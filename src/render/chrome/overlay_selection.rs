//! Selected-row surface preparation for overlay cards.

use super::*;

#[derive(Default)]
struct OverlaySelectionRects {
    selected: Vec<[f32; 4]>,
    unselected: Vec<[f32; 4]>,
    cross: Vec<[f32; 4]>,
}

struct OverlayBarLayout {
    radius: f32,
    grow_px: f32,
    extent: theme::BarExtent,
    coverage: theme::BarCoverage,
    bar_height: f32,
    bar_offset: f32,
    primary_px: std::collections::BTreeMap<usize, f32>,
    chord_px: std::collections::BTreeMap<usize, f32>,
}

impl OverlayBarLayout {
    fn span(&self, geom: &OverlayGeom, row: usize) -> (f32, f32) {
        if self.extent.hugs() {
            bar_hug_span(
                geom.band_x(),
                geom.band_w(),
                geom.text_left,
                self.primary_px.get(&row).copied().unwrap_or(0.0),
            )
        } else {
            bar_full_span(geom.band_x(), geom.band_w())
        }
    }

    /// ITEM 164 — the shortcut PLATE behind a `Bars` chord is an ACCESSORY of the
    /// selected row's own bar, so it reads the shared [`VisualSelection`] (never
    /// the logical row) for WHICH plate carries the band colour, and rides the
    /// band's DRAWN top for WHERE that plate sits. Before the transaction it took
    /// both from state: on a sliding world the chord plate recoloured and held
    /// still a whole glide before the bar it belongs to arrived under it.
    ///
    /// ITEM 174 — the fallback top (an unselected plate) is the PLANNED row's own
    /// slot, read off the plan the bar under it was placed from.
    fn append_chord_plates(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
        selected: &mut Vec<[f32; 4]>,
        unselected: &mut Vec<[f32; 4]>,
    ) {
        if self.chord_px.is_empty() {
            return;
        }
        let (full_x, full_width) = bar_full_span(geom.band_x(), geom.band_w());
        let full_right = full_x + full_width;
        let chord_right = geom.text_left + geom.text_w;
        for planned in plan.rows().iter().filter(|r| r.item.is_some()) {
            let row = planned.display;
            let Some(width) = self.chord_px.get(&row).copied() else {
                continue;
            };
            let right = (chord_right + BAR_TEXT_PAD).min(full_right);
            let plate_width = width + 2.0 * BAR_TEXT_PAD;
            let left = (right - plate_width).max(full_x);
            let on_band = vis.reads_selected(row);
            let top = match (on_band, vis.band_top()) {
                (true, Some(drawn)) => drawn,
                _ => planned.top,
            };
            let rect = [
                left,
                top + self.bar_offset,
                (right - left).max(1.0),
                self.bar_height,
            ];
            if on_band {
                selected.push(rect);
            } else if self.coverage == theme::BarCoverage::All {
                unselected.push(rect);
            }
        }
    }
}

impl TextPipeline {
    #[cfg(test)]
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
        let first_top = plan.first_top();
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
            first_top,
            line_height,
            primary[0],
        )
    }

    /// TEST PROBE (item 131a) — the Pane world's selected-band rect this frame,
    /// ACTUALLY emitted by [`Self::overlay_pane_selection`] rather than
    /// rebuilt: the drawn-evidence oracle for the Pane band's own two-sided
    /// extent, mirroring [`Self::overlay_bar_rects_probe`]'s role for `Bars`.
    /// Empty when there is no living-band motion in flight and no logical
    /// selection (nothing drawn this frame), or under `Bars`.
    #[cfg(test)]
    pub(in crate::render) fn overlay_pane_rects_probe(&mut self) -> Vec<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let vis = self.resolve_visual_selection(&geom, &plan);
        match crate::render::effective_list_style() {
            theme::ListStyle::Pane => self.overlay_pane_selection(&geom, &plan, &vis).selected,
            theme::ListStyle::Bars { .. } | theme::ListStyle::Diagonal(_) => Vec::new(),
        }
    }

    /// TEST PROBE — the `(selected, unselected)` row-surface quads this frame
    /// ACTUALLY emits under a `Bars` world, read from the emitter rather than
    /// rebuilt: the only oracle for "where did the footer plate land" that cannot
    /// be satisfied by a parallel reimplementation. Empty on a `Pane` world.
    #[cfg(test)]
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn overlay_prepare_selection(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        list_style: theme::ListStyle,
        backing: theme::ListBacking,
        vis: &VisualSelection,
    ) {
        let band_color = match theme::active()
            .highlight_treatment(crate::render::effective_overlay_selrow_band())
        {
            theme::HighlightTreatment::ValueBand(color) => color,
            theme::HighlightTreatment::InverseFill { band, .. } => band,
        };
        // ITEM 114 — THE FOCUS CUE, and the whole of it. A workspace has two
        // regions that both keep a selection, so one of the two markers has to
        // say "this one is live". It stays the SAME rect in the SAME place and
        // only loses presence — figure/ground by value, not a second decoration
        // bolted on (DESIGN.md §5). Off a workspace this is the identity.
        let rgba = match geom.workspace && !geom.rows_focused {
            true => super::workspace::dimmed(band_color, super::workspace::UNFOCUSED_MARK_ALPHA),
            false => band_color.rgba_bytes(),
        };
        self.overlay_rows.set_color(rgba);
        let rects = match list_style {
            theme::ListStyle::Pane => self.overlay_pane_selection(geom, plan, vis),
            theme::ListStyle::Bars {
                radius,
                gap,
                grow_px,
                extent,
                coverage,
            } => {
                self.overlay_bar_selection(geom, plan, vis, radius, gap, grow_px, extent, coverage)
            }
            // The diagonal changes row composition, not selection language; its
            // selection treatment remains the shared poster-bar treatment.
            theme::ListStyle::Diagonal(_) => self.overlay_bar_selection(
                geom,
                plan,
                vis,
                6.0,
                10.0,
                24.0,
                theme::BarExtent::HugLabel,
                theme::BarCoverage::All,
            ),
        };
        if backing == theme::ListBacking::BarePlates {
            self.overlay_prepare_bar_scrims(device, queue, width, height, list_style, &rects);
        }
        self.overlay_bars
            .prepare(device, queue, width, height, &rects.unselected);
        self.overlay_rows
            .prepare(device, queue, width, height, &rects.selected);
        self.overlay_cross
            .prepare(device, queue, width, height, &rects.cross);
    }

    /// ITEM 164 — the Pane band's quads for this frame, from the ALREADY-RESOLVED
    /// [`VisualSelection`]. It re-runs no animator: the travel/phase and the drawn
    /// top were decided once, at the transaction, so the fill cannot land on a
    /// different row from the ink that was shaped against it.
    fn overlay_pane_selection(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
    ) -> OverlaySelectionRects {
        let line_height = plan.lh();
        if let Some((force, from, to, t)) = vis.living() {
            let (mut selected, mut unselected, mut cross) = self.living_band_rects(
                force,
                from,
                to,
                t,
                geom.band_x(),
                geom.band_w(),
                line_height,
            );
            // ITEM 131a — the living morph drew the card's raw bounds, blind to
            // any row's own two-sided extent (a gap that predates 131a: `dx`/`dw`
            // didn't exist before the seam, so this path was never WRONG for
            // anything that ever shipped — it read `0.0` implicitly by never
            // reading anything). Apply the CURRENT logical row's extent uniformly
            // to every emitted rect, the same formula the settled branch below
            // uses: exact for the shipped default (`Choreo::Morph`, one shape,
            // one row throughout the glide), and strictly better than the prior
            // "never applied" for the non-default `TwoShape` echo too — though
            // that echo's OWN row may differ from the logical target mid-glide,
            // and resolving THAT precisely is a selection/composition question
            // (item 131e), not one this plumbing fix can silently decide.
            if let Some(row) = vis.logical() {
                let dx = plan.row_dx(row);
                let dw = plan.row_dw(row);
                for r in selected
                    .iter_mut()
                    .chain(unselected.iter_mut())
                    .chain(cross.iter_mut())
                {
                    r[0] += dx;
                    r[2] += dw - dx;
                }
            }
            self.overlay_bars.set_corner(2.5);
            self.overlay_bars
                .set_color(theme::surface_selected().rgba_bytes());
            self.overlay_cross.set_corner(2.5);
            self.overlay_cross
                .set_color(theme::overlay_band_overlap().rgba_bytes());
            return OverlaySelectionRects {
                selected,
                unselected,
                cross,
            };
        }
        let selected = match (vis.logical(), vis.band_top()) {
            (Some(row), Some(top)) => {
                let dx = plan.row_dx(row);
                let dw = plan.row_dw(row);
                vec![[
                    geom.band_x() + dx,
                    top,
                    geom.band_w() + dw - dx,
                    line_height,
                ]]
            }
            _ => Vec::new(),
        };
        OverlaySelectionRects {
            selected,
            ..OverlaySelectionRects::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn overlay_bar_selection(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
        radius: f32,
        gap: f32,
        grow_px: f32,
        extent: theme::BarExtent,
        coverage: theme::BarCoverage,
    ) -> OverlaySelectionRects {
        let layout = self.overlay_bar_layout(geom, plan, radius, gap, grow_px, extent, coverage);
        self.overlay_rows.set_corner(layout.radius);
        self.overlay_bars.set_corner(layout.radius);
        self.overlay_rows.set_stroke(0.0);
        self.overlay_bars.set_stroke(0.0);
        self.overlay_bars
            .set_color(theme::overlay_bar_unselected().rgba_bytes());
        let mut unselected = self.overlay_unselected_bar_rects(geom, plan, &layout, vis);
        let mut selected = self.overlay_selected_bar_rects(geom, plan, &layout, vis);
        layout.append_chord_plates(geom, plan, vis, &mut selected, &mut unselected);
        OverlaySelectionRects {
            selected,
            unselected,
            cross: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn overlay_bar_layout(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        radius: f32,
        gap: f32,
        grow_px: f32,
        extent: theme::BarExtent,
        coverage: theme::BarCoverage,
    ) -> OverlayBarLayout {
        let line_height = plan.lh();
        let gap = gap.max(0.0);
        let hugs = extent.hugs();
        let primary_px = if hugs {
            self.overlay_row_primary_px(geom)
        } else {
            std::collections::BTreeMap::new()
        };
        let chord_px = if hugs && !extent.inline_shortcut() && self.overlay_right_shown {
            self.overlay_row_secondary_px(geom)
        } else {
            std::collections::BTreeMap::new()
        };
        OverlayBarLayout {
            radius: radius.max(0.0),
            grow_px,
            extent,
            coverage,
            bar_height: (line_height - gap).max(1.0),
            bar_offset: gap * 0.5,
            primary_px,
            chord_px,
        }
    }

    fn overlay_unselected_bar_rects(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        layout: &OverlayBarLayout,
        vis: &VisualSelection,
    ) -> Vec<[f32; 4]> {
        let mut rects = match layout.coverage {
            theme::BarCoverage::SelectedOnly => Vec::new(),
            theme::BarCoverage::All => plan
                .rows()
                .iter()
                .filter(|r| r.item.is_some() && !vis.reads_selected(r.display))
                .map(|r| self.overlay_bar_plate(geom, layout, r))
                .collect(),
        };
        if geom.hint_rows + geom.footer_rows > 0 {
            // ITEM 174 — ONE owner of "how many content rows precede the footer"
            // ([`OverlayRowPlan::content_rows`]): the candidate band PLUS an
            // empty-state notice line. The measured hug width and the plate's own
            // top now read the same number, so the plate can no longer sit a row
            // above the glyphs it backs on a card that shows a notice.
            let content_rows = plan.content_rows();
            let footer_hug = layout.extent.hugs().then(|| {
                (
                    geom.text_left,
                    self.overlay_footer_content_px(geom, content_rows),
                )
            });
            rects.push(footer_plate_rect(
                plan.footer_top(),
                geom.band_x(),
                geom.band_w(),
                geom.card_y + geom.card_h,
                footer_hug,
            ));
        }
        if geom.theme {
            for r in plan.rows().iter().filter(|r| r.item.is_none()) {
                let (x, width) = layout.span(geom, r.display);
                rects.push([x, r.top + layout.bar_offset, width, layout.bar_height]);
            }
            rects.extend(self.overlay_strip_tab_plates.iter().copied());
        }
        rects
    }

    fn overlay_bar_plate(
        &self,
        geom: &OverlayGeom,
        layout: &OverlayBarLayout,
        planned: &PlannedRow,
    ) -> [f32; 4] {
        let row = planned.display;
        let (x, width) = layout.span(geom, row);
        let (x, width) = slant_bar_span(x, width, layout.extent.hugs(), planned.dx, planned.dw);
        [x, planned.top + layout.bar_offset, width, layout.bar_height]
    }

    /// ITEM 164 — the selected BAR's quad. Its `y` is the transaction's already
    /// eased [`VisualSelection::band_top`] (this path re-runs no animator); its
    /// `x`/width still hug the LOGICAL row's own shaped label, because a hug span
    /// is a property of the row the bar is travelling TO.
    fn overlay_selected_bar_rects(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        layout: &OverlayBarLayout,
        vis: &VisualSelection,
    ) -> Vec<[f32; 4]> {
        let (Some(row), Some(top)) = (vis.logical(), vis.band_top()) else {
            return Vec::new();
        };
        let (x, width) = layout.span(geom, row);
        let (x, width) = slant_bar_span(
            x,
            width,
            layout.extent.hugs(),
            plan.row_dx(row),
            plan.row_dw(row),
        );
        let grow = layout.grow_px * self.overlay_grow_progress();
        let mirror = crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth();
        let (x, width) = grow_span(x, width, grow, mirror);
        vec![[
            x,
            top + layout.bar_offset,
            width.max(1.0),
            layout.bar_height,
        ]]
    }

    fn overlay_prepare_bar_scrims(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        list_style: theme::ListStyle,
        rects: &OverlaySelectionRects,
    ) {
        const PAD: f32 = 2.0;
        let radius = match list_style {
            theme::ListStyle::Bars { radius, .. } => radius.max(0.0),
            theme::ListStyle::Diagonal(_) => 6.0,
            theme::ListStyle::Pane => 0.0,
        };
        let scrims = rects
            .unselected
            .iter()
            .chain(rects.selected.iter())
            .map(|&[x, y, width, height]| [x - PAD, y - PAD, width + 2.0 * PAD, height + 2.0 * PAD])
            .collect::<Vec<_>>();
        self.panel_card.set_corner(radius + PAD);
        self.panel_card
            .set_color(theme::overlay_bars_scrim().rgba_bytes());
        self.panel_card
            .prepare(device, queue, width, height, &scrims);
    }
}
