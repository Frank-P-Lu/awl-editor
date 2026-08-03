//! The label's coverage mask: ONE short shaped run composed into ONE R8 image.
//!
//! [`crate::caret_glyph::GlyphMask`] caches ONE glyph's swash coverage keyed by
//! its [`CacheKey`]; this is the same cache one level up — a whole run's
//! coverage, composed on the CPU in the run's OWN unrotated frame, keyed by the
//! exact physical glyphs that produced it. Composing on the CPU is what keeps
//! this a label capability rather than a second prose renderer: the shaping,
//! hinting, spacing and anti-aliasing are byte-for-byte an upright render's,
//! and the GPU only ever sees one image and one quad.
//!
//! ONE RUN, deliberately. [`LabelMask::compose`] reads `layout_runs().next()`
//! and stops. A label that wrapped would need line breaking, per-line
//! alignment and a per-line axis — which is the document layer's job, and the
//! document layer stays the one prose renderer.

use glyphon::{Buffer as GlyphBuffer, CacheKey, FontSystem, SwashCache, SwashContent};

use super::geometry::InkBox;

/// The transparent border, in pixels, the composed image carries on every side.
///
/// A rotated quad has no anti-aliasing of its own — the rasteriser either
/// covers a pixel or it does not — so a stem whose coverage runs right to the
/// mask's edge would be cut off square along the quad boundary. One row of
/// guaranteed-zero coverage moves that hard edge off the ink.
pub const MASK_PAD: u32 = 1;

/// The physical glyphs a run composed from: `(cache key, x, y)` in the run's
/// own pixel frame. This IS the mask's identity — the cache key already folds
/// glyph id, font, size and subpixel bin, so anything that would rasterise
/// differently compares differently.
pub type LabelKey = Vec<(CacheKey, i32, i32)>;

/// One short run's composed coverage, uploaded to a single R8 texture.
pub struct LabelMask {
    key: LabelKey,
    /// Owns the GPU texture; `view` is what the bind group samples, but the
    /// texture must outlive it, so this stays as an RAII guard.
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    ink: InkBox,
    size: (u32, u32),
}

impl LabelMask {
    /// The run's ink box in its own frame — `[u_min, v_min, width, height]`,
    /// with `v` measured DOWN from the baseline, so `v_min` is negative for any
    /// run with an ascender. Includes [`MASK_PAD`] on every side.
    pub fn ink(&self) -> InkBox {
        self.ink
    }

    /// The composed image's pixel size, padding included.
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Whether this mask already describes `buffer`'s first run — the cache
    /// test a caller makes before paying for a re-composition.
    pub fn matches(&self, buffer: &GlyphBuffer) -> bool {
        self.key == run_key(buffer)
    }

    /// Compose `buffer`'s FIRST layout run into one coverage image and upload
    /// it. `None` when the run has no rasterisable ink at all (an empty label,
    /// or a run of whitespace) — there is nothing to draw and no texture worth
    /// allocating.
    ///
    /// A glyph swash hands back as anything but [`SwashContent::Mask`] (a
    /// colour emoji) is skipped: a label is one ink, and a coverage mask has
    /// nowhere to put a second colour.
    pub fn compose(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        buffer: &GlyphBuffer,
    ) -> Option<Self> {
        let key = run_key(buffer);
        let (data, ink, w, h) = compose_run(font_system, swash_cache, buffer)?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rotated label mask"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                // R8 = 1 byte/px; the composed image is tightly packed.
                bytes_per_row: Some(w),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Some(Self {
            key,
            texture,
            view,
            ink,
            size: (w, h),
        })
    }
}

/// `buffer`'s FIRST layout run as physical glyphs in the run's own frame.
pub fn run_key(buffer: &GlyphBuffer) -> LabelKey {
    let Some(run) = buffer.layout_runs().next() else {
        return Vec::new();
    };
    run.glyphs
        .iter()
        .map(|g| {
            let p = g.physical((0.0, 0.0), 1.0);
            (p.cache_key, p.x, p.y)
        })
        .collect()
}

/// The composed coverage for `buffer`'s first run: `(row-major R8 coverage, ink
/// box, width, height)`.
///
/// Split out of [`LabelMask::compose`] because this image is the GROUND TRUTH a
/// rotated render is graded against, and grading needs it without a device in
/// hand. It is not the code under test — the rotation is — so a law may read it
/// freely.
pub fn compose_run(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    buffer: &GlyphBuffer,
) -> Option<(Vec<u8>, InkBox, u32, u32)> {
    compose_coverage(font_system, swash_cache, &run_key(buffer))
}

fn compose_coverage(
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    key: &LabelKey,
) -> Option<(Vec<u8>, InkBox, u32, u32)> {
    // Rasterise every glyph first: the union box is not known until they are
    // all placed, and a run short enough to be a label is short enough to hold.
    let mut cells: Vec<(i32, i32, u32, u32, Vec<u8>)> = Vec::new();
    for &(cache_key, gx, gy) in key {
        let Some(image) = swash_cache.get_image_uncached(font_system, cache_key) else {
            continue;
        };
        if image.content != SwashContent::Mask {
            continue;
        }
        let (w, h) = (image.placement.width, image.placement.height);
        if w == 0 || h == 0 || image.data.len() < (w as usize) * (h as usize) {
            continue;
        }
        // The swash placement box hangs off the pen origin: `left` is +x from
        // the pen, `top` the pixels ABOVE the baseline — the same convention
        // `TextPipeline::caret_glyph_geometry` reads it in.
        cells.push((
            gx + image.placement.left,
            gy - image.placement.top,
            w,
            h,
            image.data,
        ));
    }
    if cells.is_empty() {
        return None;
    }

    let pad = MASK_PAD as i32;
    let min_x = cells.iter().map(|c| c.0).min()? - pad;
    let min_y = cells.iter().map(|c| c.1).min()? - pad;
    let max_x = cells.iter().map(|c| c.0 + c.2 as i32).max()? + pad;
    let max_y = cells.iter().map(|c| c.1 + c.3 as i32).max()? + pad;
    let w = (max_x - min_x).max(1) as u32;
    let h = (max_y - min_y).max(1) as u32;

    let mut data = vec![0u8; (w as usize) * (h as usize)];
    for (cx, cy, cw, ch, src) in &cells {
        for row in 0..*ch {
            let dy = (cy - min_y) + row as i32;
            if dy < 0 || dy >= h as i32 {
                continue;
            }
            for col in 0..*cw {
                let dx = (cx - min_x) + col as i32;
                if dx < 0 || dx >= w as i32 {
                    continue;
                }
                let s = src[(row * cw + col) as usize];
                let d = &mut data[dy as usize * w as usize + dx as usize];
                // MAX, not a sum: two glyphs whose boxes overlap (an accent, a
                // kerned pair) must union their coverage, never saturate it
                // into a blob.
                *d = (*d).max(s);
            }
        }
    }

    let ink = [min_x as f32, min_y as f32, w as f32, h as f32];
    Some((data, ink, w, h))
}
