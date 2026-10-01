//! Cached tight coverage bounds for short shaped marks and control labels.
//! Physical glyph identity owns the raster; callers own role, size and slot.

use super::geometry::InkBox;
use glyphon::{Buffer as GlyphBuffer, CacheKey, FontSystem, SwashCache, SwashContent};
use std::collections::HashMap;
use std::ops::Range;

/// Physical glyph identities and pen positions in the buffer's text-origin
/// frame, including each shaped baseline. This differs from mask::run_key,
/// whose one-run label keys are baseline-relative for rotation.
pub type PositionedGlyphKey = Vec<(CacheKey, i32, i32)>;

pub fn buffer_key(buffer: &GlyphBuffer) -> PositionedGlyphKey {
    positioned_key(buffer, [0.0, 0.0], |_, _| true)
}

/// Line-local byte range, matching ControlSpan and cosmic-text glyph starts.
pub fn span_key(buffer: &GlyphBuffer, row: usize, bytes: Range<usize>) -> PositionedGlyphKey {
    span_key_at(buffer, row, bytes, [0.0, 0.0])
}

/// Match the physical glyph keys at a real TextArea origin, including its
/// subpixel position; bounds are returned in that screen coordinate frame.
pub fn span_key_at(
    buffer: &GlyphBuffer,
    row: usize,
    bytes: Range<usize>,
    origin: [f32; 2],
) -> PositionedGlyphKey {
    positioned_key(buffer, origin, |line, start| {
        line == row && bytes.contains(&start)
    })
}

fn positioned_key(
    buffer: &GlyphBuffer,
    origin: [f32; 2],
    contains: impl Fn(usize, usize) -> bool,
) -> PositionedGlyphKey {
    let mut keys = Vec::new();
    for run in buffer.layout_runs() {
        let baseline = run.line_y.round() as i32;
        for glyph in run.glyphs.iter().filter(|g| contains(run.line_i, g.start)) {
            let physical = glyph.physical((origin[0], origin[1]), 1.0);
            keys.push((physical.cache_key, physical.x, physical.y + baseline));
        }
    }
    keys
}

/// One font-system lifetime's optical bounds, sharing SwashCache's key space.
/// Stores no font data or textures; whitespace and missing masks cache None.
#[derive(Default)]
pub struct InkBoundsCache {
    glyphs: HashMap<CacheKey, Option<[i32; 4]>>,
}

impl InkBoundsCache {
    /// Union tight nonzero mask bounds in the caller's pixel frame.
    /// Each tuple is (physical cache key, glyph pen x, glyph pen y).
    pub fn bounds(
        &mut self,
        fonts: &mut FontSystem,
        swash: &mut SwashCache,
        glyphs: &[(CacheKey, i32, i32)],
    ) -> Option<InkBox> {
        let mut union: Option<[i32; 4]> = None;
        for &(key, x, y) in glyphs {
            let own = *self.glyphs.entry(key).or_insert_with(|| {
                let image = swash.get_image(fonts, key).as_ref()?;
                if image.content != SwashContent::Mask {
                    return None;
                }
                let [left, top, width, height] =
                    mask_bounds(image.placement.width, image.placement.height, &image.data)?;
                Some([
                    image.placement.left + left,
                    top - image.placement.top,
                    width,
                    height,
                ])
            });
            let Some([left, top, width, height]) = own else {
                continue;
            };
            let next = [x + left, y + top, x + left + width, y + top + height];
            union = Some(match union {
                Some(old) => [
                    old[0].min(next[0]),
                    old[1].min(next[1]),
                    old[2].max(next[2]),
                    old[3].max(next[3]),
                ],
                None => next,
            });
        }
        union.map(|[left, top, right, bottom]| {
            [
                left as f32,
                top as f32,
                (right - left) as f32,
                (bottom - top) as f32,
            ]
        })
    }
}

/// A composed label already caches its result by physical run identity.
/// This convenience path shares the same coverage scanner and Swash raster.
pub fn bounds(
    fonts: &mut FontSystem,
    swash: &mut SwashCache,
    glyphs: &[(CacheKey, i32, i32)],
) -> Option<InkBox> {
    InkBoundsCache::default().bounds(fonts, swash, glyphs)
}

/// Tight [left, top, width, height] in a row-major one-channel mask.
/// Transparent allocation padding is excluded; allocation/UVs stay separate.
pub fn mask_bounds(width: u32, height: u32, data: &[u8]) -> Option<[i32; 4]> {
    let len = (width as usize).checked_mul(height as usize)?;
    if width == 0 || height == 0 || data.len() < len {
        return None;
    }
    let (mut left, mut top) = (width as i32, height as i32);
    let (mut right, mut bottom) = (0, 0);
    for (row, pixels) in data[..len].chunks_exact(width as usize).enumerate() {
        for (col, &alpha) in pixels.iter().enumerate() {
            if alpha == 0 {
                continue;
            }
            left = left.min(col as i32);
            top = top.min(row as i32);
            right = right.max(col as i32 + 1);
            bottom = bottom.max(row as i32 + 1);
        }
    }
    (right > left && bottom > top).then_some([left, top, right - left, bottom - top])
}
