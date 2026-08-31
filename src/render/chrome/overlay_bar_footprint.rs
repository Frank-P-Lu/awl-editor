//! Bars plate envelope used by the footprint frost.

use super::*;

impl TextPipeline {
    /// THE SELECTION-INDEPENDENT ENVELOPE OF EVERY PLATE THIS BARS CARD CAN DRAW.
    ///
    /// The backdrop cache must not change shape merely because the selected row moved,
    /// while a hugged composition still owes frost only to its real plates rather than
    /// to the wider pointer band. This asks the draw path's layout owner for every base
    /// plate, adds the selected ledge at this frame's grow phase to every row that could
    /// carry it, and includes accessory and footer plates. `SelectedOnly` changes which
    /// plates draw now, not the selection-independent envelope the frost must back.
    pub(super) fn overlay_bar_footprint_rects(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        cfg: theme::BarConfig,
    ) -> Vec<[f32; 4]> {
        let mut envelope_cfg = cfg;
        envelope_cfg.coverage = theme::BarCoverage::All;
        let layout = self.overlay_bar_layout(geom, plan, envelope_cfg);
        let vis = VisualSelection::default();
        let (mut rects, _) = self.overlay_unselected_bar_rects(geom, plan, &layout, &vis);

        let grow = layout.grow_px * self.overlay_grow_progress();
        let mirror = crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth();
        rects.extend(plan.rows().iter().filter(|r| r.item.is_some()).map(|r| {
            let [x, y, width, height] = self.overlay_bar_plate(geom, &layout, r);
            let (x, width) = grow_span(x, width, grow, mirror);
            [x, y, width.max(1.0), height]
        }));

        let mut chord = Vec::new();
        layout.append_chord_plates(geom, plan, &vis, &mut chord, &mut rects);
        rects.extend(chord);
        rects
    }
}
