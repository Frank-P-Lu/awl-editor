//! Test-only observations of the overlay selection emitters.

use super::*;

impl TextPipeline {
    pub(in crate::render) fn living_probe_geom(
        &mut self,
        geom: &OverlayGeom,
    ) -> (Vec<usize>, usize, f32, f32, [f32; 4]) {
        let plan = self.overlay_row_plan(geom);
        let vis = self.resolve_visual_selection(geom, &plan);
        let (motion, from, to, t) = vis
            .living()
            .expect("living_probe_geom needs the motion probe armed on a Pane world");
        let selected_row = vis.logical().expect("a selected row");
        let line_height = plan.lh();
        let (primary, _, _) = self.living_band_rects(
            motion,
            from,
            to,
            t,
            geom.band_x(),
            geom.band_w(),
            line_height,
        );
        (
            vis.rows().to_vec(),
            selected_row,
            plan.first_top(),
            line_height,
            primary[0],
        )
    }

    pub(in crate::render) fn overlay_pane_rects_probe(&mut self) -> Vec<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let vis = self.resolve_visual_selection(&geom, &plan);
        match crate::render::effective_list_style() {
            theme::ListStyle::Pane => self.overlay_pane_selection(&geom, &plan, &vis).selected,
            theme::ListStyle::Bars { .. } | theme::ListStyle::Diagonal(_) => Vec::new(),
        }
    }

    /// EVERY ROW SURFACE THIS FRAME WOULD DRAW — selected, unselected and the
    /// travelling overlap — from the one production owner. Legal on every
    /// style, and empty is a real answer: a `Diagonal` world draws no row fill
    /// at all, and a law may assert exactly that.
    pub(in crate::render) fn overlay_row_surfaces_probe(&mut self) -> Vec<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let vis = self.resolve_visual_selection(&geom, &plan);
        let r =
            self.overlay_selection_rects(&geom, &plan, &vis, crate::render::effective_list_style());
        r.selected
            .into_iter()
            .chain(r.unselected)
            .chain(r.cross)
            .collect()
    }

    /// THE PLATES A PLATED WORLD DREW, and only a plated world's.
    ///
    /// Refusing is the point. `BarePlates` reads like "draws plates" and is not
    /// — it is the CARD's backing — so a law sweeping that roster reaches
    /// `Diagonal` worlds, which draw no plate at all. Answering them would mean
    /// synthesizing: calling `overlay_bar_selection` at hardcoded dials no world
    /// authors and no frame draws, and every claim graded against those quads
    /// would be a claim about an invention. A law asking for plates on a world
    /// that draws none fails here, by name, instead of going green. The honest
    /// question for those worlds is [`Self::overlay_row_surfaces_probe`].
    pub(in crate::render) fn overlay_bar_rects_probe(&mut self) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
        let style = crate::render::effective_list_style();
        assert!(
            style.draws_row_plates(),
            "overlay_bar_rects_probe: {} ({style:?}) draws no row plates — there is no \
             plate geometry to grade here, and synthesizing some would only grade an \
             invention. Sweep the plate-drawing roster (`ListStyle::draws_row_plates`), \
             or ask `overlay_row_surfaces_probe` what this world really draws.",
            theme::active().name,
        );
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let vis = self.resolve_visual_selection(&geom, &plan);
        let r = self.overlay_selection_rects(&geom, &plan, &vis, style);
        (r.selected, r.unselected)
    }
}

impl OverlayGeom {
    /// TEST-ONLY readers for the palette-location laws: the card's candidate
    /// DISPLAY LINES tagged by kind, and how many there are — so a law can
    /// assert what the band opens on without a render path exposing its plan.
    #[cfg(test)]
    pub(in crate::render) fn plan_labels_probe(&self) -> Vec<String> {
        self.plan
            .iter()
            .filter_map(|l| match l {
                PlanLine::Location(s) => Some(format!("loc:{s}")),
                PlanLine::Header(s) => Some(format!("hdr:{s}")),
                PlanLine::Item(_) => None,
            })
            .collect()
    }

    #[cfg(test)]
    pub(in crate::render) fn plan_len_probe(&self) -> usize {
        self.plan.len()
    }
}
