//! Folder navigation keeps its persistent location separate from the editable filter.

use super::*;

pub(super) const LOCATION_GROUP_GAP: Rows = Rows(1.5);
pub(super) const ACTION_GROUP_GAP: Rows = Rows(0.65);
// Redistribute the existing path band: more air above, less below, no row lost.
const LOCATION_LEADING: Rows = Rows(0.5);

impl TextPipeline {
    fn folder_location_leading(&self, height: f32) -> f32 {
        (self.overlay_lh() * LOCATION_LEADING.0).min((height - self.overlay_lh()).max(0.0))
    }

    pub(super) fn fit_folder_location(
        &mut self,
        geom: &OverlayGeom,
        font_size: f32,
        plan: &OverlayRowPlan,
    ) -> Option<f32> {
        let location = self.overlay_folder_location.clone()?;
        let height = plan.header_lines().get(1)?.height;
        let prepared = crate::overlay::PreparedDirectoryPath::new(&location);
        let metrics = GlyphMetrics::new(
            font_size * crate::markdown::type_scale::LABEL,
            height - self.folder_location_leading(height),
        );
        let attrs = overlay_panel_attrs().metrics(metrics);
        let budgets = super::overlay_shape::files_location_fit_budgets(
            prepared.len(),
            prepared.leaf_identity_budget(),
        );
        let measure = self
            .overlay_folder_buffer
            .get_or_insert_with(|| GlyphBuffer::new(&mut self.font_system, metrics));
        for budget in budgets {
            let shown = prepared.elide(budget);
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
        Some(height)
    }

    /// A noneditable display area inside the path's one planned header budget.
    pub(super) fn folder_location_area(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        bounds: TextBounds,
    ) -> Option<(f32, f32, TextBounds)> {
        self.overlay_folder_location.as_ref()?;
        let line = plan.header_lines().get(1)?;
        Some((
            geom.text_left,
            line.top + self.folder_location_leading(line.height),
            TextBounds {
                top: (line.top.ceil() as i32).max(bounds.top),
                bottom: (line.bottom().floor() as i32).min(bounds.bottom),
                ..bounds
            },
        ))
    }

    #[cfg(test)]
    pub(super) fn folder_location_glyph_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        let line = plan.header_lines().get(1)?;
        let run = self.overlay_folder_buffer.as_ref()?.layout_runs().next()?;
        let x0 = run.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
        let x1 = run.glyphs.iter().map(|g| g.x + g.w).fold(0.0, f32::max);
        (x0.is_finite() && x1 > x0).then_some([
            geom.text_left + x0,
            line.top + self.folder_location_leading(line.height) + run.line_top,
            x1 - x0,
            run.line_height,
        ])
    }
}

/// Reserve the path's header band; its independently shaped text is uploaded above.
pub(super) fn push_location_spans<'b>(
    spans: &mut Vec<(&str, glyphon::Attrs<'b>)>,
    height: Option<f32>,
    font_size: f32,
    attrs: glyphon::Attrs<'b>,
) {
    if let Some(height) = height {
        spans.push(("\n", attrs.clone()));
        spans.push((
            " ",
            attrs.metrics(GlyphMetrics::new(
                font_size * crate::markdown::type_scale::LABEL,
                height,
            )),
        ));
    }
}

/// Upload the independently shaped location without borrowing other pipeline owners.
pub(super) fn push_folder_location_area<'a>(
    areas: &mut Vec<TextArea<'a>>,
    buffer: Option<&'a GlyphBuffer>,
    geometry: Option<(f32, f32, TextBounds)>,
    muted: glyphon::Color,
) {
    if let Some(buffer) = buffer
        && let Some((left, top, bounds)) = geometry
    {
        areas.push(TextArea {
            buffer,
            left,
            top,
            scale: 1.0,
            bounds,
            default_color: muted,
            custom_glyphs: &[],
        });
    }
}
