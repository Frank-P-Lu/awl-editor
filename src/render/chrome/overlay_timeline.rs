//! History timeline metadata fitting inside the shared overlay row lanes.

use super::*;

/// Leads the first label with exactly the planner's header-row count. A
/// contextual card has zero headers, so it must gain no synthetic blank line.
pub(super) fn right_bind_lines<'a>(
    header_rows: usize,
    labels: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    labels
        .enumerate()
        .map(|(k, label)| {
            let leads = if k == 0 { header_rows } else { 1 };
            format!("{}{label}", "\n".repeat(leads))
        })
        .collect()
}

impl TextPipeline {
    /// History metadata is authored content, not an all-or-nothing shortcut.
    /// Grant each row what its own primary leaves, then end-ellipsize against
    /// shaped pixels until the two lanes genuinely fit.
    pub(super) fn shape_timeline_right_to_fit(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        muted: glyphon::Color,
        vis: &VisualSelection,
        elide: bool,
    ) -> Option<bool> {
        if !(geom.workspace && self.overlay_rows_primary && elide) {
            return None;
        }
        let right_labels = self.overlay_right_labels();
        let labels: Vec<String> = geom
            .plan
            .iter()
            .map(|line| match line {
                PlanLine::Item(i) => right_labels.get(*i).cloned().unwrap_or_default(),
                PlanLine::Location(_) | PlanLine::Header(_) => String::new(),
            })
            .collect();
        let primary = self.overlay_row_primary_px(geom);
        let gap_px = rowlayout::GAP_CHARS as f32 * self.overlay_char_width();
        let char_w = self.overlay_char_width().max(1.0);
        let mut fitted: Vec<String> = labels
            .iter()
            .enumerate()
            .map(|(display, label)| {
                let left = primary.get(&display).copied().unwrap_or(0.0) + gap_px;
                let chars = ((geom.text_w - left).max(0.0) / char_w).floor() as usize;
                rowlayout::fit_primary_end(label, chars)
            })
            .collect();

        loop {
            let lines =
                right_bind_lines(plan.billed_header_rows(), fitted.iter().map(String::as_str));
            self.shape_overlay_right(geom, ink, muted, vis, &lines);
            let secondary = self.overlay_row_secondary_px(plan);
            let mut changed = false;
            for (display, label) in fitted.iter_mut().enumerate() {
                let used = primary.get(&display).copied().unwrap_or(0.0)
                    + gap_px
                    + secondary.get(&display).copied().unwrap_or(0.0);
                if used > geom.text_w + 0.01 && !label.is_empty() {
                    *label = rowlayout::fit_primary_end(label, label.chars().count() - 1);
                    changed = true;
                }
            }
            if !changed {
                self.overlay_right_shown = fitted.iter().any(|s| !s.is_empty());
                return Some(self.overlay_right_shown);
            }
        }
    }
}
