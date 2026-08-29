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

    /// Seat the shaped facet line PAST a `Split` composition's own visible
    /// seam, clear of the lower surface's own rim — the same relocation
    /// [`Self::docked_facet_band`] does for `DockedTab` (above the card
    /// entirely), for the composition that instead claims part of the
    /// strip's OWN box from underneath it. `strip_band()`'s box is folded
    /// with the whole query beat (`header_gap`) so cosmic-text's half-leading
    /// centres the label ink in it, but a `Split` composition's own seam
    /// (`OverlayRowPlan::split_bounds`) falls INSIDE that same box — so the
    /// natural centring places real ink only a few px past the seam's own
    /// rim, reading as clipped against it. This seat instead optically
    /// centres a PLAIN (uninflated) line in whatever room is actually left
    /// past the seam, `[gap_bottom, first_top]` — real, roster-measured
    /// breathing room on both sides of the ink rather than a symmetric split
    /// of a box that starts before the plate the reader actually sees.
    ///
    /// `None` off [`split_seam_active`]'s own gate (a flat card, a workspace,
    /// `DockedTab`, `Unified`, or a `Bars`/`Diagonal`/`Ruled` world with no
    /// plate to seam) — every one of those keeps reading `strip_band()`
    /// directly, unmoved.
    pub(in crate::render) fn floating_strip_band(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<crate::render::plan::PlannedHeader> {
        if !split_seam_active(geom) {
            return None;
        }
        let strip = plan.strip_band()?;
        let (_, gap_bottom) = plan.split_bounds()?;
        let bottom = strip.bottom();
        let lh = self.overlay_lh().min(strip.height);
        // Room is whatever is REALLY left between the seam and the candidate
        // band, never assumed — a starved narrow/short card can leave less
        // than one plain line here, and `height` shrinks to fit rather than
        // the seat overshooting `bottom` (which `strip.bottom()` shares with
        // `plan.first_top()`: sacred, never pushed by this seat).
        let room = (bottom - gap_bottom).max(0.0);
        let height = lh.min(room);
        let top = (gap_bottom + (room - height) * 0.5).min(bottom - height);
        Some(crate::render::plan::PlannedHeader {
            line: strip.line,
            top,
            height,
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

    pub(super) fn shape_docked_facet_strip(&mut self, geom: &OverlayGeom, scale: f32) {
        // This one buffer serves TWO relocation seats — `DockedTab` (above the
        // card) and a `Split` composition's own seam (past the lower surface's
        // rim) — never both on the same world (`floating_strip_band` excludes
        // `DockedTab` by construction), so one shaped pass covers either.
        let relocated = facet_strip_is_docked() || split_seam_active(geom);
        let metrics = self.overlay_metrics();
        self.docked_facet_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.docked_facet_buffer
            .set_size(&mut self.font_system, None, None);
        self.docked_facet_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let fs = self.metrics.font_size * crate::render::effective_overlay_scale() * scale;
        let mut spans = Vec::new();
        if relocated {
            for (idx, (label, active)) in geom.strip.iter().enumerate() {
                if idx > 0 {
                    spans.push((
                        super::strip_gap(),
                        chrome_attrs()
                            .color(theme::faint().to_glyphon())
                            .metrics(GlyphMetrics::new(fs, self.overlay_lh())),
                    ));
                }
                spans.push((
                    label.as_str(),
                    chrome_attrs()
                        .color(if *active {
                            theme::base_content().to_glyphon()
                        } else {
                            theme::muted().to_glyphon()
                        })
                        .metrics(GlyphMetrics::new(fs, self.overlay_lh())),
                ));
            }
        }
        self.docked_facet_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &panel_attrs(),
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
        let docked = self.docked_facet_band(&geom, &plan);
        let floating = self.floating_strip_band(&geom, &plan);
        let strip = docked.or(floating).or_else(|| plan.strip_band())?;
        if !strip.contains(py) {
            return None;
        }
        // THE SAME SEAT THE DRAW PATH USED: a `DockedTab` strip, and a
        // `Split` composition's relocated strip, both draw in their own
        // buffer at the card's plain text edge (`push_docked_facet_areas`
        // always seats either there, off `panel_buffer`'s own head band —
        // and neither composition is ever a banded/diagonal cluster,
        // `split_seam_active` requires `ListBacking::Card`, which
        // `ListStyle::Diagonal` never is). Every OTHER facet style draws the
        // strip INSIDE `panel_buffer`'s head band, seated at
        // `overlay_head_left` — the card's text edge on an upright world, but
        // right-aligned to the text column on an ascending diagonal cluster
        // (Magpie). Reading `geom.text_left` unconditionally here missed that
        // second seat: a click on a drawn label could resolve to no lens, or
        // the wrong one, on any banded world.
        let seat = if docked.is_some() || floating.is_some() {
            geom.text_left
        } else {
            self.overlay_head_left(&geom, &plan)
        };
        let want = px - seat;
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

/// Redirect the strip's line OUT of `panel_buffer`'s ordinary stack into a
/// dedicated, independently-seated buffer, whichever relocation claimed it
/// (`seat`: `docked_facet_band` above the card, or `floating_strip_band` past
/// a `Split` composition's own seam) — the panel buffer still draws every
/// OTHER header/candidate line, carved around the strip's original box so
/// nothing double-draws.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_docked_facet_areas<'a>(
    areas: &mut Vec<TextArea<'a>>,
    panel_buffer: &'a GlyphBuffer,
    docked_facet_buffer: &'a GlyphBuffer,
    seat: Option<crate::render::plan::PlannedHeader>,
    original: Option<crate::render::plan::PlannedHeader>,
    text_left: f32,
    text_top: f32,
    clip_left: i32,
    clip_right: i32,
    height: u32,
    ink: glyphon::Color,
) -> bool {
    let Some((dock, original)) = seat.zip(original) else {
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
    }
    // The dock owns a dedicated buffer for the complete strip. Uploading the
    // panel buffer three times with disjoint clips made glyphon enrollment
    // order-dependent: a typed query or a non-first active facet could drop
    // the tail. One buffer and one area make the whole navigation line stable.
    areas.push(TextArea {
        buffer: docked_facet_buffer,
        left: text_left,
        top: dock.top,
        scale: 1.0,
        bounds: TextBounds {
            left: clip_left,
            top: dock.top.max(0.0) as i32,
            right: clip_right,
            bottom: dock.bottom().min(height as f32) as i32,
        },
        default_color: ink,
        custom_glyphs: &[],
    });
    true
}
