//! The bottom-left gutter's own lava frost seeds. Split out of `gutter.rs`
//! to keep that file near its natural size — the geometry here is a second,
//! device-free question ("where does the frost breathe") answered off the
//! exact same [`TextPipeline::gutter_layout`] the block itself draws from.

use super::*;

impl TextPipeline {
    /// THE ORGANIC FROST SEEDS for the bottom-left GUTTER (the shipped lava
    /// treatment): the filename + project lines each seed halos `[x0, x1, yc, r]`
    /// (device px) hugging their RIGHT-aligned ink near the column, so they join the
    /// SAME summed field the outline feeds ([`TextPipeline::prepare_lava_layer`]) —
    /// a warm organic whisper under the stack instead of the old full-width
    /// rectangle. Seeds hug the ACTUAL ink (each line's width, right-aligned to
    /// `avail`) rather than the whole `[0, avail]` box. `None`-empty when the gutter
    /// is HIDDEN. Rides the SAME [`Self::gutter_layout`] owner + the shared
    /// [`crate::render::frost_seed_radius`] / [`crate::render::push_text_seeds`] the
    /// outline uses, so both surfaces (and both worlds) seed identically.
    pub(in crate::render) fn gutter_frost_seeds(&self, height: u32) -> Vec<[f32; 4]> {
        let Some(layout) = self.gutter_layout() else {
            return Vec::new();
        };
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        if row_h <= 0.0 {
            return Vec::new();
        }
        let r_row = crate::render::frost_seed_radius(
            row_h,
            crate::lava::FROST_FEATHER_PX,
            self.metrics.zoom,
            self.dpi,
        );
        let skirt =
            crate::lava::frost_px(crate::lava::FROST_FEATHER_PX, self.metrics.zoom, self.dpi);
        let pad_x =
            crate::lava::frost_px(crate::lava::FROST_PILL_PAD_X, self.metrics.zoom, self.dpi);
        // The two stacked LABEL rows, bottom-anchored at the SAME named inset
        // (mirrors `prepare_gutter` / `gutter_carve_rect`, and the corner
        // readouts): name over project. Each line is RIGHT-aligned within
        // `[0, avail]`, so its ink hugs the column at the right edge.
        let stack = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            row_h,
            layout.lines().len(),
            self.metrics.px_physical(super::readout::CANVAS_INSET),
            super::gutter::GUTTER_CARVE_BREATH.0,
        );
        // The gutter's own LABEL advance (its glyphs are the doc advance × LABEL).
        let label_char_w = self.metrics.char_width * label;
        let push_line = |seeds: &mut Vec<[f32; 4]>, text: &str, row: f32| {
            if text.is_empty() {
                return;
            }
            let w = (text.chars().count() as f32 * label_char_w).min(layout.avail);
            let yc = stack.rows[row as usize][1] + row_h * 0.5;
            crate::render::push_text_seeds(
                seeds,
                layout.avail - w - pad_x,
                w + 2.0 * pad_x,
                yc,
                r_row,
                skirt,
                text,
            );
        };
        let mut seeds = Vec::new();
        for (row, (text, _)) in layout.lines().into_iter().enumerate() {
            push_line(&mut seeds, text, row as f32);
        }
        seeds
    }
}
