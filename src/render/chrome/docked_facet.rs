use super::*;

impl TextPipeline {
    /// Seat the shaped facet line immediately above the card. The grouped
    /// strip's planned box also owns the beat below its glyph line, so only
    /// its glyph-bearing line docks; otherwise the tab outgrows a narrow
    /// canvas's top margin.
    pub(in crate::render) fn docked_facet_band(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<crate::render::plan::PlannedHeader> {
        let strip = plan.strip_band()?;
        let dock_h = self.overlay_lh().min(strip.height);
        matches!(
            crate::render::effective_facet_style(),
            theme::FacetStyle::DockedTab
        )
        .then_some(crate::render::plan::PlannedHeader {
            line: strip.line,
            top: (geom.card_y - dock_h).max(0.0),
            height: dock_h.min(geom.card_y),
        })
    }

    #[cfg(test)]
    pub(in crate::render) fn docked_facet_geometry_probe(
        &self,
    ) -> Option<(crate::render::plan::PlannedHeader, [f32; 4])> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        self.docked_facet_band(&geom, &plan)
            .map(|dock| (dock, [geom.card_x, geom.card_y, geom.card_w, geom.card_h]))
    }

    pub(super) fn shape_docked_facet_label(&mut self, geom: &OverlayGeom) {
        let label = if matches!(
            crate::render::effective_facet_style(),
            theme::FacetStyle::DockedTab
        ) {
            geom.strip
                .iter()
                .find_map(|(label, active)| active.then_some(label.as_str()))
                .unwrap_or("")
        } else {
            ""
        };
        let metrics = self.overlay_metrics();
        self.docked_facet_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.docked_facet_buffer
            .set_size(&mut self.font_system, None, None);
        self.docked_facet_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        self.docked_facet_buffer.set_text(
            &mut self.font_system,
            label,
            &panel_attrs().color(theme::base_content().to_glyphon()),
            Shaping::Advanced,
            None,
        );
        self.docked_facet_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }

    /// Hit-test a pointer against the same shaped facet line the dock moves.
    pub fn overlay_lens_at(&self, px: f32, py: f32) -> Option<usize> {
        if !self.overlay_active || self.overlay_lens.is_empty() {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        if !geom.theme || px < geom.card_x || px > geom.card_x + geom.card_w {
            return None;
        }
        let plan = self.overlay_row_plan(&geom);
        let strip = self
            .docked_facet_band(&geom, &plan)
            .or_else(|| plan.strip_band())?;
        if !strip.contains(py) {
            return None;
        }
        let want = px - geom.text_left;
        let mut hit = None;
        for run in self
            .panel_buffer
            .layout_runs()
            .filter(|run| run.line_i == 1)
        {
            let mut text = String::from("\n");
            let mut ranges = Vec::new();
            for (idx, (label, _)) in self.overlay_lens.iter().enumerate() {
                if idx > 0 {
                    text.push_str(super::strip_gap());
                }
                let start = text.len();
                text.push_str(label);
                ranges.push((idx, start..text.len()));
            }
            for glyph in run
                .glyphs
                .iter()
                .filter(|g| want >= g.x && want < g.x + g.w)
            {
                let byte = glyph.start + 1;
                for (idx, range) in &ranges {
                    if range.contains(&byte) {
                        hit = Some(*idx);
                    }
                }
            }
        }
        hit
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_docked_facet_areas<'a>(
    areas: &mut Vec<TextArea<'a>>,
    panel_buffer: &'a GlyphBuffer,
    docked_facet_buffer: &'a GlyphBuffer,
    dock: Option<crate::render::plan::PlannedHeader>,
    tab: Option<[f32; 4]>,
    original: Option<crate::render::plan::PlannedHeader>,
    text_left: f32,
    text_top: f32,
    clip_left: i32,
    clip_right: i32,
    height: u32,
    ink: glyphon::Color,
) -> bool {
    let Some(((dock, tab), original)) = dock.zip(tab).zip(original) else {
        return false;
    };
    {
        let mut push = |top: f32, clip_top: f32, clip_bottom: f32| {
            areas.push(TextArea {
                buffer: panel_buffer,
                left: text_left,
                top,
                scale: 1.0,
                bounds: TextBounds {
                    left: clip_left,
                    top: clip_top.max(0.0) as i32,
                    right: clip_right,
                    bottom: clip_bottom.min(height as f32) as i32,
                },
                default_color: ink,
                custom_glyphs: &[],
            });
        };
        push(text_top, 0.0, original.top);
        push(text_top, original.bottom(), height as f32);

        // Move the complete shaped strip with the active tab. Its active span
        // is camouflaged in the tab surface and redrawn once below. Keeping
        // the quiet labels in one TextArea avoids GlyphBuffer multi-clip
        // behavior dropping the tail after the active label.
        let strip_baseline = panel_buffer
            .layout_runs()
            .find(|run| run.line_i == original.line)
            .map_or(original.top - text_top, |run| run.line_y);
        let dock_baseline = docked_facet_buffer
            .layout_runs()
            .next()
            .map_or(0.0, |run| run.line_y);
        let moved_top = dock.top + dock_baseline - strip_baseline;
        let clip_top = dock.top.max(0.0) as i32;
        let clip_bottom = dock.bottom().min(height as f32) as i32;
        areas.push(TextArea {
            buffer: panel_buffer,
            left: text_left,
            top: moved_top,
            scale: 1.0,
            bounds: TextBounds {
                left: clip_left,
                top: clip_top,
                right: clip_right,
                bottom: clip_bottom,
            },
            default_color: ink,
            custom_glyphs: &[],
        });
    }
    let label_w = docked_facet_buffer
        .layout_runs()
        .next()
        .map_or(0.0, |run| run.line_w);
    areas.push(TextArea {
        buffer: docked_facet_buffer,
        left: tab[0] + (tab[2] - label_w) * 0.5,
        top: dock.top,
        scale: 1.0,
        bounds: TextBounds {
            left: tab[0].max(0.0) as i32,
            top: dock.top.max(0.0) as i32,
            right: (tab[0] + tab[2]).min(clip_right as f32) as i32,
            bottom: dock.bottom().min(height as f32) as i32,
        },
        default_color: ink,
        custom_glyphs: &[],
    });
    true
}
