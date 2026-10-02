//! Range controls reserve real pixel territory before their names are seated.

use super::*;

impl TextPipeline {
    /// Settings controls share one measured value lane. Fit every caption
    /// against its width so a long checkbox label cannot drop the whole lane;
    /// range captions additionally reserve the rail's authored room.
    pub(super) fn fit_overlay_control_names(
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
        let value_w = self.widest_right_px();
        loop {
            let primary = self.overlay_row_primary_px(geom);
            let mut changed = false;
            for row in plan.rows() {
                let Some(item) = row.item else {
                    continue;
                };
                let room = if self.overlay_ranges.get(item).copied().flatten().is_some() {
                    rowlayout::rail_min_room(row.height)
                } else {
                    rowlayout::GAP_CHARS as f32 * self.overlay_char_width()
                };
                let Some(caption) = rows.get_mut(row.display) else {
                    continue;
                };
                let measured = primary.get(&row.display).copied().unwrap_or(0.0);
                let budget = (text_w - value_w - room - 0.5).max(0.0);
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
