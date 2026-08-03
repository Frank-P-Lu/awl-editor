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

    pub(in crate::render) fn overlay_bar_rects_probe(&mut self) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let vis = self.resolve_visual_selection(&geom, &plan);
        match crate::render::effective_list_style() {
            theme::ListStyle::Bars {
                radius,
                gap,
                grow_px,
                extent,
                coverage,
            } => {
                let r = self.overlay_bar_selection(
                    &geom, &plan, &vis, radius, gap, grow_px, extent, coverage,
                );
                (r.selected, r.unselected)
            }
            theme::ListStyle::Pane => (Vec::new(), Vec::new()),
            theme::ListStyle::Diagonal(_) => {
                let r = self.overlay_bar_selection(
                    &geom,
                    &plan,
                    &vis,
                    6.0,
                    10.0,
                    24.0,
                    theme::BarExtent::HugLabel,
                    theme::BarCoverage::All,
                );
                (r.selected, r.unselected)
            }
        }
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
