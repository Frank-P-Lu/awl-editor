//! Proportional caret vertical envelopes and glyphless neighbor borrowing.

use super::*;

impl TextPipeline {
    /// Search within the visual row and blend lifted boxes from both sides by
    /// distance. This stays O(row width), never O(document).
    pub(super) fn nearest_row_raster_box(&mut self, line: usize, col: usize) -> Option<InkBox> {
        let rows = self.visual_rows(line);
        let row = rows.get(pick_row_index_aff(&rows, col, self.caret_affinity))?;
        let (start, end) = (row.start_col, row.end_col);
        let max_back = col.saturating_sub(start);
        let max_fwd = end.saturating_sub(col + 1);
        let back = (1..=max_back).find_map(|d| self.raster_box_at(line, col - d).map(|b| (d, b)));
        let fwd = (1..=max_fwd).find_map(|d| self.raster_box_at(line, col + d).map(|b| (d, b)));
        let (_, ascent, font) = self.caret_row_metrics();
        let lift = |(d, ink)| (d, self.caret_cell_vertical_ink_box(ink, ascent, font));

        match (back.map(lift), fwd.map(lift)) {
            (Some((db, bb)), Some((df, fb))) => {
                let t = db as f32 / (db + df) as f32;
                Some(InkBox {
                    left: bb.left + (fb.left - bb.left) * t,
                    top: bb.top + (fb.top - bb.top) * t,
                    width: bb.width + (fb.width - bb.width) * t,
                    height: bb.height + (fb.height - bb.height) * t,
                })
            }
            (Some((_, box_)), None) | (None, Some((_, box_))) => Some(box_),
            (None, None) => None,
        }
    }

    /// Lift short ink into its row's measured x-height band, preserving any
    /// real ascender or descender without a punctuation table.
    fn caret_cell_vertical_ink_box(&self, ink: InkBox, ascent: f32, font: &str) -> InkBox {
        let top = ink
            .top
            .max((ascent * facepitch::x_height_ratio(font)).max(1.0));
        let descent = ink.descent();
        InkBox {
            height: top + descent,
            top,
            ..ink
        }
    }

    /// The sole proportional-cell vertical owner. Horizontal support-body
    /// dimensions remain ink-derived; only this centre and height use the
    /// row's insertion envelope.
    pub(in crate::render) fn caret_cell_vertical_from_ink(
        &self,
        ink: InkBox,
        baseline: f32,
        ascent: f32,
        font: &str,
        px: f32,
    ) -> (f32, f32) {
        let vertical = self.caret_cell_vertical_ink_box(ink, ascent, font);
        let (_, old_h) = caret_visual_body_dims(ink, px);
        let h = old_h.max(vertical.height + 2.0 * CARET_INK_PAD * px);
        (baseline - vertical.top + vertical.height * 0.5, h)
    }
}
