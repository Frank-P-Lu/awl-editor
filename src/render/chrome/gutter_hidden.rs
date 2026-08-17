//! THE GUTTER'S HIDDEN ARM, plus the doc-dimming predicate that has always sat
//! beside it.
//!
//! Neither belongs to the block's shape, which is what [`super::gutter`] owns —
//! one is what the pipeline does when there is no block at all, and the other is
//! not about the gutter in the first place. They live here so that file stays
//! about the lines it draws.

use super::*;

impl TextPipeline {
    /// The HIDDEN arm: park an empty buffer off-screen so nothing draws and a
    /// non-page (or unnamed, or too-narrow) capture stays byte-identical to what
    /// it rendered before this chrome existed.
    pub(super) fn park_gutter_offscreen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: TextBounds,
        muted: glyphon::Color,
    ) -> anyhow::Result<()> {
        let line_height = self.metrics.line_height;
        self.gutter_buffer
            .set_size(&mut self.font_system, Some(1.0), Some(line_height));
        self.gutter_buffer.set_text(
            &mut self.font_system,
            "",
            &panel_attrs().color(muted),
            Shaping::Advanced,
            None,
        );
        self.gutter_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let area = TextArea {
            buffer: &self.gutter_buffer,
            left: 0.0,
            top: -1000.0,
            scale: 1.0,
            bounds,
            default_color: muted,
            custom_glyphs: &[],
        };
        self.gutter_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon gutter prepare failed: {e:?}"))
    }

    /// True when a FULL-takeover overlay is up and the document RECEDES behind it (the
    /// cached frosted-blur backdrop is active over the whole canvas). False for the
    /// search SPLIT panel / no overlay (the doc stays bright), for the crisp THEME/CARET
    /// pickers (the doc stays crisp so the live theme colours / caret preview read
    /// honestly), for the POINTER-ANCHORED menu (it frosts its own footprint at most, and
    /// a footprint dims by nothing), AND for the contextual SPELL panel (a small float
    /// popup at the word — it recedes nothing). Reported in the sidecar as `dim_overlay`.
    ///
    /// ONE OWNER with the frost's own full-arm gate (`overlay_blur`): the sidecar's field
    /// and the pass that draws the dim answer one question, and a second copy of the rule
    /// is how the report comes to disagree with the pixels.
    pub fn dims_doc(&self) -> bool {
        self.overlay_blur()
    }
}
