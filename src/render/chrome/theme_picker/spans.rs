use super::*;

pub(super) struct ThemeStripSpec<'a> {
    pub(super) text: &'a str,
    pub(super) labels: &'a [(std::ops::Range<usize>, bool)],
    pub(super) separators: &'a [std::ops::Range<usize>],
    pub(super) scale: f32,
}

impl TextPipeline {
    pub(super) fn push_theme_strip_spans<'a>(
        &self,
        spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
        plan: &OverlayRowPlan,
        strip: ThemeStripSpec<'a>,
        active_ink: glyphon::Color,
        muted: glyphon::Color,
    ) {
        let m = self.metrics;
        let ui = crate::render::effective_overlay_scale();
        let lh = self.overlay_lh();
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let faint = theme::faint().to_glyphon();
        let mut cursor = 0usize;
        let mut pushes: Vec<(std::ops::Range<usize>, glyphon::Color)> = strip
            .labels
            .iter()
            .map(|(range, active)| (range.clone(), if *active { active_ink } else { muted }))
            .chain(strip.separators.iter().cloned().map(|range| (range, faint)))
            .collect();
        pushes.sort_by_key(|(range, _)| range.start);
        // The strip's own PLANNED box height carries the query beat. Inflation
        // rides the real label glyphs rather than the leading line break because
        // cosmic-text sizes a line from the glyphs on it.
        let strip_lh = plan.strip_band().map_or(lh, |band| band.height);
        spans.push((
            &strip.text[0..1],
            mk(faint).metrics(GlyphMetrics::new(m.font_size * ui, lh)),
        ));
        cursor += 1;
        for (range, color) in pushes {
            debug_assert_eq!(range.start, cursor, "strip spans must tile the line");
            cursor = range.end;
            let fs = m.font_size * ui * strip.scale.min(1.0);
            spans.push((
                &strip.text[range],
                chrome_attrs()
                    .color(color)
                    .metrics(GlyphMetrics::new(fs, strip_lh)),
            ));
        }
    }

    pub(super) fn push_theme_plan_spans<'a>(
        &self,
        spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
        geom: &'a OverlayGeom,
        fitted: &'a [Option<String>],
        trailing: &'a [String],
        inks: OverlaySpanInks,
        vis: &VisualSelection,
    ) {
        let OverlaySpanInks {
            ink,
            muted,
            selected: selected_ink,
        } = inks;
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
