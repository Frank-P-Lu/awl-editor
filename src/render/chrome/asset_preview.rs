//! THE ASSET CLEANER's live PREVIEW PANEL — a second coordinated region
//! beside the "Clean unused assets…" picker's row list (PHILOSOPHY §1: "One
//! task may contain several coordinated regions"), showing the highlighted
//! orphan's own image so the choice of what to trash is informed rather than
//! made from an opaque filename alone.
//!
//! Gated on [`TextPipeline::overlay_asset_preview`] (`Option<PathBuf>`,
//! mirroring [`crate::render::ViewState::overlay_asset_preview`]) — an
//! `Option` the render layer reads, exactly the way the contextual spell
//! popup gates on `overlay_spell` and the dimension picker on
//! `overlay_table_dims`. `ViewState` carries no `OverlayKind` at all; the
//! gate is the row's own `RowMeta::Asset`
//! ([`crate::overlay::OverlayState::selected_asset_path`]), never a kind
//! string threaded down here.
//!
//! Reuses the ONE inline-image decode/texture path
//! ([`image_cache`](crate::render::image_cache)) — never a second decoder —
//! decoding only the CURRENTLY SELECTED orphan (never the whole roster:
//! nothing here ever calls `ensure` for a row that isn't highlighted). A
//! later selection with the SAME mtime is a cache hit through that one
//! owner, so moving the highlight back to a row already previewed this
//! session costs nothing.
//!
//! Draws through its OWN dedicated trio (`asset_preview_panel` /
//! `asset_preview_image` / `asset_preview_text_renderer`) rather than the
//! shared float-panel quads (`float_shadow`/`float_border`/`float_card`) the
//! search panel and the caret-preview panel already claim in frames this
//! picker can be open in — a second claimant on that shared model in the
//! same frame is a real conflict, not a reuse.

use super::*;

/// The gap between the picker's own card and this panel.
const ASSET_PREVIEW_GAP: Logical = Logical(16.0);
/// The panel's own inner pad, around the image / the can't-decode text.
const ASSET_PREVIEW_PAD: Logical = Logical(14.0);
/// The panel's own corner rounding — a value distinct from (but visually
/// matching) inline images' own `IMAGE_CORNER_PX`, which stays private to
/// `render/layers.rs`.
const ASSET_PREVIEW_CORNER: Logical = Logical(4.0);
/// The panel's width band: never wider than this, so the LIST — not the
/// preview — stays the primary surface (the item's own constraint).
const ASSET_PREVIEW_MAX_W: Logical = Logical(220.0);
/// Below this much genuine room beside the card, the panel draws NOTHING —
/// the list never yields room to the preview, the preview yields to the
/// list. See [`TextPipeline::asset_preview_rect`]'s narrow-canvas degrade.
const ASSET_PREVIEW_MIN_W: Logical = Logical(96.0);
/// The can't-decode statement — honest and plain, never a blank that reads
/// as a bug. Such files remain trashable; this only says the picker cannot
/// show what one looks like.
const CANT_DECODE_STATEMENT: &str = "Can't preview this file";

impl TextPipeline {
    /// The panel's PLANNED rect `[x, y, w, h]` for the sidecar (`overlay.
    /// asset_preview`, schema `/211`), or `None` off the Asset Cleaner / when
    /// the canvas has no room for it — mirrors [`Self::overlay_card_rect`]'s
    /// own shape, reading the SAME [`Self::asset_preview_rect`] the panel
    /// actually draws from, so a Verify clause's sample point can never
    /// disagree with the pixels.
    pub fn asset_preview_report(&self) -> Option<[f32; 4]> {
        if !self.overlay_active {
            return None;
        }
        self.asset_preview_rect(self.window_w as u32)
    }

    /// The panel's own rect `[x, y, w, h]`, beside the picker's card at the
    /// SAME `card_y`/`card_h` — a coordinated region, not a second card
    /// hunting for its own place. `None` when there is nothing to preview
    /// ([`Self::overlay_asset_preview`] empty) OR when the canvas has no
    /// genuine room for it beside the card ([`ASSET_PREVIEW_MIN_W`]).
    ///
    /// Recomputes [`Self::overlay_geometry`] — cheap, since the ROW PLAN
    /// (the budgeted per-frame `OverlayRowPlan`) is a SEPARATE build this
    /// never touches, the identical precedent `table_dims_cell_at`'s own
    /// hit-test recomputation already sets.
    pub(in crate::render) fn asset_preview_rect(&self, width: u32) -> Option<[f32; 4]> {
        self.overlay_asset_preview.as_ref()?;
        let geom = self.overlay_geometry(width);
        let gap = self.metrics.px(ASSET_PREVIEW_GAP);
        let margin = self.metrics.px(CARD_MARGIN);
        let max_w = self.metrics.px(ASSET_PREVIEW_MAX_W);
        let min_w = self.metrics.px(ASSET_PREVIEW_MIN_W);
        let avail = width as f32 - (geom.card_x + geom.card_w) - gap - margin;
        if avail < min_w {
            return None;
        }
        Some([
            geom.card_x + geom.card_w + gap,
            geom.card_y,
            avail.min(max_w),
            geom.card_h,
        ])
    }

    /// Decode the SELECTED orphan (through the one inline-image cache) and
    /// draw the panel: the thumbnail CONTAIN-fit inside it when the file
    /// decodes, or the honest can't-decode statement (name, size, a plain
    /// sentence) when it does not — never a blank. A no-op park whenever
    /// [`Self::asset_preview_rect`] has nothing to show (every kind but
    /// Assets, or a canvas with no room for it), so a default frame and
    /// every other picker's frame stay byte-identical.
    pub(in crate::render) fn prepare_asset_preview(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let (Some(rect), Some(path)) = (
            self.asset_preview_rect(width),
            self.overlay_asset_preview.clone(),
        ) else {
            return self.park_asset_preview(device, queue, width, height);
        };
        // Read only inside the native decode block below — wasm has no
        // decoder at all (`image_cache` is native-only), so every orphan is
        // the honest can't-decode state there by construction.
        #[cfg(target_arch = "wasm32")]
        let _ = &path;
        let pad = self.metrics.px(ASSET_PREVIEW_PAD);
        let inner = [
            rect[0] + pad,
            rect[1] + pad,
            (rect[2] - 2.0 * pad).max(1.0),
            (rect[3] - 2.0 * pad).max(1.0),
        ];
        let corner = self.metrics.px(ASSET_PREVIEW_CORNER);
        self.asset_preview_panel
            .set_color(theme::base_200().rgba_bytes());
        self.asset_preview_panel.set_corner(corner);
        self.asset_preview_panel
            .prepare(device, queue, width, height, &[rect]);

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::render::image_cache::{ImageCache, ImageState};
            let max_dim = device.limits().max_texture_dimension_2d;
            let is_ready = matches!(
                self.image_cache
                    .ensure(device, queue, &path, inner[2], max_dim),
                ImageState::Ready { .. }
            );
            if is_ready {
                let key = ImageCache::canonical_key(&path);
                let dst = self
                    .image_cache
                    .intrinsic(&key)
                    .map(|dims| crate::render::image_cache::contain_fit(dims, inner[2], inner[3]))
                    .map(|(dw, dh)| {
                        [
                            inner[0] + (inner[2] - dw) * 0.5,
                            inner[1] + (inner[3] - dh) * 0.5,
                            dw,
                            dh,
                        ]
                    });
                if let (Some(dst), Some(view)) = (dst, self.image_cache.view(&key)) {
                    self.asset_preview_image.prepare(
                        device,
                        queue,
                        width,
                        height,
                        &[crate::image_pipeline::PlacedImage {
                            dst,
                            alpha: 1.0,
                            corner,
                            view,
                        }],
                    );
                    self.park_asset_preview_text(device, queue, width, height)?;
                    return Ok(());
                }
            }
        }

        // MISSING / CAN'T-DECODE (or no decoder compiled at all, on wasm):
        // the orphan that fails to decode is the MOST important one to see
        // honestly — draw the panel's calm statement instead of a blank.
        self.asset_preview_image.clear();
        let name = self
            .overlay_items
            .get(self.overlay_selected)
            .cloned()
            .unwrap_or_default();
        let secondary = self
            .overlay_bindings
            .get(self.overlay_selected)
            .cloned()
            .unwrap_or_default();
        let buffers = self.build_asset_preview_missing_text_buffers(&name, &secondary, inner);
        let bounds = self.clip_text_bounds(TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        });
        let areas: Vec<TextArea> = buffers
            .iter()
            .map(|(buf, left, top, color)| TextArea {
                buffer: buf,
                left: *left,
                top: *top,
                scale: 1.0,
                bounds,
                default_color: *color,
                custom_glyphs: &[],
            })
            .collect();
        self.asset_preview_text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon asset preview render failed: {e:?}"))?;
        Ok(())
    }

    /// Park every asset-preview pipeline empty — the frame after the Assets
    /// picker closes (or narrows below [`ASSET_PREVIEW_MIN_W`]) carries no
    /// stale thumbnail or statement.
    pub(in crate::render) fn park_asset_preview(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.asset_preview_panel
            .prepare(device, queue, width, height, &[]);
        self.asset_preview_image.clear();
        self.park_asset_preview_text(device, queue, width, height)
    }

    fn park_asset_preview_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let _ = (width, height);
        self.asset_preview_text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                Vec::<TextArea>::new(),
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon asset preview park failed: {e:?}"))?;
        Ok(())
    }

    /// Shape the can't-decode state's three lines (name / size / the plain
    /// statement) centred inside `inner`, mirroring
    /// `build_missing_placeholder_text_buffers`'s inline-image precedent —
    /// same label scale, same doc-attrs ink ladder (muted primary, faint
    /// secondary). An empty `name`/`secondary` (a bare-fixture test, or a
    /// row whose secondary column is genuinely blank) simply omits that
    /// line rather than shaping nothing.
    fn build_asset_preview_missing_text_buffers(
        &mut self,
        name: &str,
        secondary: &str,
        inner: [f32; 4],
    ) -> Vec<(GlyphBuffer, f32, f32, glyphon::Color)> {
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let gm = GlyphMetrics::new(m.font_size * label, m.line_height * label);
        let line_h = m.line_height * label;
        let muted = theme::muted().to_glyphon();
        let faint = theme::faint().to_glyphon();
        let center = Some(glyphon::cosmic_text::Align::Center);
        let lines: [(&str, glyphon::Color); 3] = [
            (name, muted),
            (secondary, faint),
            (CANT_DECODE_STATEMENT, faint),
        ];
        let box_w = inner[2].max(1.0);
        let rows = lines.iter().filter(|(t, _)| !t.is_empty()).count().max(1);
        let block_h = line_h * rows as f32;
        let start_y = inner[1] + (inner[3] - block_h).max(0.0) * 0.5;
        let mut buffers = Vec::with_capacity(lines.len());
        let mut row = 0.0;
        for (text, color) in lines {
            if text.is_empty() {
                continue;
            }
            let attrs = self.doc_attrs().color(color);
            let mut buf = GlyphBuffer::new(&mut self.font_system, gm);
            buf.set_size(&mut self.font_system, Some(box_w), Some(line_h));
            buf.set_text(
                &mut self.font_system,
                text,
                &attrs,
                Shaping::Advanced,
                center,
            );
            buf.shape_until_scroll(&mut self.font_system, false);
            buffers.push((buf, inner[0], start_y + line_h * row, color));
            row += 1.0;
        }
        buffers
    }
}
