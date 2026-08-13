use super::*;

impl TextPipeline {
    pub(super) fn push_theme_plan_spans<'a>(
        &self,
        spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
        geom: &'a OverlayGeom,
        fitted: &'a [Option<String>],
        trailing: &'a [String],
        ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
    ) {
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let lh = self.overlay_lh();
        let ui = crate::render::effective_overlay_scale();
        let header_metrics = GlyphMetrics::new(
            self.metrics.font_size * ui * crate::markdown::type_scale::LABEL,
            lh,
        );
        let location_metrics = GlyphMetrics::new(self.metrics.font_size * ui * LOCATION_SCALE, lh);
        let slant_italic = crate::render::overlay_slant().is_some_and(|s| s.italic);
        let row_attrs = |c| {
            if slant_italic {
                mk(c).style(glyphon::cosmic_text::Style::Italic)
            } else {
                mk(c)
            }
        };
        for (idx, (line, fit)) in geom.plan.iter().zip(fitted.iter()).enumerate() {
            spans.push(("\n", mk(ink)));
            match line {
                PlanLine::Location(label) => {
                    if theme::active().render_caps.location_style.draws_inline() {
                        spans.push((
                            label.as_str(),
                            chrome_attrs().color(muted).metrics(location_metrics),
                        ));
                    }
                }
                PlanLine::Header(label) => {
                    spans.push((
                        label.as_str(),
                        mk(theme::faint().to_glyphon()).metrics(header_metrics),
                    ));
                }
                PlanLine::Item(_) => {
                    let color = match selected_ink {
                        Some(color) if vis.reads_selected(idx) => color,
                        _ => ink,
                    };
                    spans.push((fit.as_deref().unwrap_or(""), row_attrs(color)));
                    if let Some(cell) = trailing.get(idx).filter(|cell| !cell.is_empty()) {
                        push_symbol_split(spans, cell, || mk(muted), || sym(muted));
                    }
                }
            }
        }
    }
}
