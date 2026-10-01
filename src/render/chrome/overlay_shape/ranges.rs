//! Range controls reserve real pixel territory before their names are seated.

use super::*;

impl TextPipeline {
    /// Refine only visible range captions against their measured readout and
    /// the rail's authored room. Re-shaping the real row buffer preserves every
    /// font, highlight and baseline decision; non-range captions stay intact.
    pub(super) fn fit_overlay_range_names(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        inks: OverlaySpanInks,
        vis: &VisualSelection,
        rows: &mut [String],
        text_w: f32,
    ) {
        if self.overlay_ranges.is_empty() {
            return;
        }
        let secondary = self.overlay_row_secondary_px(plan);
        loop {
            let primary = self.overlay_row_primary_px(geom);
            let mut changed = false;
            for row in plan.rows() {
                let Some(item) = row.item else {
                    continue;
                };
                if self.overlay_ranges.get(item).copied().flatten().is_none() {
                    continue;
                }
                let Some(caption) = rows.get_mut(row.display) else {
                    continue;
                };
                let measured = primary.get(&row.display).copied().unwrap_or(0.0);
                let value_w = secondary.get(&row.display).copied().unwrap_or(0.0);
                let budget =
                    (text_w - value_w - rowlayout::rail_min_room(row.height) - 0.5).max(0.0);
                let chars = caption.chars().count();
                if measured > budget && chars > 4 {
                    // The ratio reaches the likely fit in one pass. Each further
                    // pass strictly shrinks the short, already-elided caption.
                    let next =
                        ((chars as f32 * budget / measured).floor() as usize).clamp(4, chars - 1);
                    *caption = rowlayout::fit_primary(caption, next);
                    changed = true;
                }
            }
            if !changed {
                return;
            }
            self.shape_overlay_names(geom, plan, inks, vis, rows, &[]);
        }
    }
}
