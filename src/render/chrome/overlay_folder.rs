//! Folder navigation keeps the current location separate from the editable filter.

use super::*;

pub(super) const LOCATION_GROUP_GAP: Rows = Rows(1.0);
pub(super) const ACTION_GROUP_GAP: Rows = Rows(0.65);

impl TextPipeline {
    pub(super) fn fit_folder_location(
        &mut self,
        geom: &OverlayGeom,
        font_size: f32,
        plan: &OverlayRowPlan,
    ) -> Option<(String, f32)> {
        let location = self.overlay_folder_location.clone()?;
        let height = plan.header_lines().get(1)?.height;
        let prepared = crate::overlay::PreparedDirectoryPath::new(&location);
        let metrics = GlyphMetrics::new(font_size * crate::markdown::type_scale::LABEL, height);
        let attrs = overlay_panel_attrs().metrics(metrics);
        let budgets = super::overlay_shape::files_location_fit_budgets(
            prepared.len(),
            prepared.leaf_identity_budget(),
        );
        let mut shown = String::new();
        for budget in budgets {
            shown = prepared.elide(budget);
            let measure = &mut self.workspace_hint_measure_buffer;
            measure.set_metrics(&mut self.font_system, metrics);
            measure.set_size(&mut self.font_system, None, None);
            measure.set_wrap(&mut self.font_system, Wrap::None);
            measure.set_text(
                &mut self.font_system,
                &shown,
                &attrs,
                Shaping::Advanced,
                None,
            );
            measure.shape_until_scroll(&mut self.font_system, false);
            if measure.layout_runs().all(|run| run.line_w <= geom.text_w) {
                break;
            }
        }
        Some((shown, height))
    }
}
