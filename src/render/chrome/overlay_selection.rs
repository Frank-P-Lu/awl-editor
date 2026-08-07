//! Selected-row surface preparation for overlay cards.

use super::*;

#[derive(Default)]
pub(super) struct OverlaySelectionRects {
    pub(super) selected: Vec<[f32; 4]>,
    pub(super) unselected: Vec<[f32; 4]>,
    pub(super) cross: Vec<[f32; 4]>,
    /// The footer plate's own rect (also in `unselected`), so its rim can be
    /// seated without re-identifying it. `None` off `Bars` or with no footer.
    pub(super) footer_plate: Option<[f32; 4]>,
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
    /// The frame's `zoom * dpi`, so every span this layout resolves reads the
    /// one scale the card was placed at.
    scale: f32,
}

/// Apply each travelling band's own planned two-sided span. This stays beside
/// the selection emitter so no choreography can smuggle in a second row-offset
/// calculation.
fn apply_living_row_spans(plan: &OverlayRowPlan, rects: &mut [[f32; 4]]) {
    for r in rects {
        if let Some(row) = plan.display_nearest(r[1] + r[3] * 0.5) {
            let dx = plan.row_dx(row);
            let dw = plan.row_dw(row);
            r[0] += dx;
            r[2] += dw - dx;
        }
    }
}

impl OverlayBarLayout {
    fn span(&self, geom: &OverlayGeom, row: usize) -> (f32, f32) {
        if self.extent.hugs() {
            bar_hug_span(
                geom.band_x(),
                geom.band_w(),
                geom.text_left,
                self.primary_px.get(&row).copied().unwrap_or(0.0),
                self.scale,
            )
        } else {
            bar_full_span(geom.band_x(), geom.band_w(), self.scale)
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
        let (full_x, full_width) = bar_full_span(geom.band_x(), geom.band_w(), self.scale);
        let text_pad = BAR_TEXT_PAD.px(self.scale);
        let full_right = full_x + full_width;
        let chord_right = geom.text_left + geom.text_w;
        for planned in plan.rows().iter().filter(|r| r.item.is_some()) {
            let row = planned.display;
            let Some(width) = self.chord_px.get(&row).copied() else {
                continue;
            };
            let right = (chord_right + text_pad).min(full_right);
            let plate_width = width + 2.0 * text_pad;
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
        let rects = self.overlay_selection_rects(geom, plan, vis, list_style);
        if backing == theme::ListBacking::BarePlates {
            self.overlay_prepare_bar_scrims(device, queue, width, height, list_style, &rects);
        }
        self.overlay_bars
            .prepare(device, queue, width, height, &rects.unselected);
        self.overlay_rows
            .prepare(device, queue, width, height, &rects.selected);
        self.overlay_cross
            .prepare(device, queue, width, height, &rects.cross);
        // ITEM 308 — the footer plate's RIM: its rect grown one px on every
        // side, drawn under it (mirrors `notice_rim`/`notice_plate`). Resolved
        // fresh from the live theme every frame, so no `sync_theme_colors` entry.
        let rim_rects: Vec<[f32; 4]> = rects
            .footer_plate
            .map(|[x, y, w, h]| [x - 1.0, y - 1.0, w + 2.0, h + 2.0])
            .into_iter()
            .collect();
        self.footer_plate_rim
            .set_color(theme::overlay_footer_plate_rim().rgba_bytes());
        self.footer_plate_rim
            .prepare(device, queue, width, height, &rim_rects);
    }

    /// THE ONE PLACE a list style becomes row surfaces. `overlay_prepare_selection`
    /// and the test probe both come through here, so a law cannot grade a shape
    /// this frame would not have drawn.
    pub(super) fn overlay_selection_rects(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
        list_style: theme::ListStyle,
    ) -> OverlaySelectionRects {
        match list_style {
            theme::ListStyle::Pane => self.overlay_pane_selection(geom, plan, vis),
            theme::ListStyle::Bars => {
                self.overlay_bar_selection(geom, plan, vis, crate::render::effective_bar_config())
            }
            // Diagonal selection is the bright CHEVRON prepared by its measured
            // composition owner (`diagonal::selected_chevron`). It deliberately has
            // no row-fill fallback: the planned outward span remains the pointer and
            // text truth while the line, rather than a poster bar, carries focus.
            theme::ListStyle::Diagonal(_) => OverlaySelectionRects::default(),
            theme::ListStyle::Rules(mark) => self.overlay_rules_selection(geom, plan, vis, mark),
        }
    }

    /// ITEM 164 — the Pane band's quads for this frame, from the ALREADY-RESOLVED
    /// [`VisualSelection`]. It re-runs no animator: the travel/phase and the drawn
    /// top were decided once, at the transaction, so the fill cannot land on a
    /// different row from the ink that was shaped against it.
    pub(super) fn overlay_pane_selection(
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
            // Every travelling shape owns the planned row nearest ITS centre.
            // This makes the non-default TwoShape echo precise: it no longer
            // borrows the logical target's diagonal offset while visibly sitting
            // on another row. The planner remains the one shared geometry source
            // for the band, row text, and pointer inverse.
            apply_living_row_spans(plan, &mut selected);
            apply_living_row_spans(plan, &mut unselected);
            apply_living_row_spans(plan, &mut cross);
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
                footer_plate: None,
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

    pub(super) fn overlay_bar_selection(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
        cfg: theme::BarConfig,
    ) -> OverlaySelectionRects {
        let layout = self.overlay_bar_layout(geom, plan, cfg);
        self.overlay_rows.set_corner(layout.radius);
        self.overlay_bars.set_corner(layout.radius);
        self.overlay_rows.set_stroke(0.0);
        self.overlay_bars.set_stroke(0.0);
        self.overlay_bars
            .set_color(theme::overlay_bar_unselected().rgba_bytes());
        let (mut unselected, footer_plate) =
            self.overlay_unselected_bar_rects(geom, plan, &layout, vis);
        let mut selected = self.overlay_selected_bar_rects(geom, plan, &layout, vis);
        layout.append_chord_plates(geom, plan, vis, &mut selected, &mut unselected);
        OverlaySelectionRects {
            selected,
            unselected,
            cross: Vec::new(),
            footer_plate,
        }
    }

    fn overlay_bar_layout(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        cfg: theme::BarConfig,
    ) -> OverlayBarLayout {
        let line_height = plan.lh();
        // The gap arrives as the config's own dial; the plate it separates is
        // seated inside a row pitch that already resolved the same number.
        let gap = self.metrics.px(Logical(cfg.gap.max(0.0)));
        let hugs = cfg.extent.hugs();
        let primary_px = if hugs {
            self.overlay_row_primary_px(geom)
        } else {
            std::collections::BTreeMap::new()
        };
        let chord_px = if hugs && !cfg.extent.inline_shortcut() && self.overlay_right_shown {
            self.overlay_row_secondary_px(geom)
        } else {
            std::collections::BTreeMap::new()
        };
        OverlayBarLayout {
            // The plate's own corner and its outward growth are config-owned
            // LENGTHS, resolved at the same boundary the gap between the plates
            // already passes — so a bar keeps its shape at every scale.
            radius: self.metrics.px(Logical(cfg.radius.max(0.0))),
            grow_px: self.metrics.px(Logical(cfg.grow_px)),
            extent: cfg.extent,
            coverage: cfg.coverage,
            bar_height: (line_height - gap).max(1.0),
            bar_offset: gap * 0.5,
            primary_px,
            chord_px,
            scale: self.metrics.scale,
        }
    }

    /// Returns the unselected bar rects, plus the footer plate's own rect
    /// separately (`None` if this frame draws no footer).
    fn overlay_unselected_bar_rects(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        layout: &OverlayBarLayout,
        vis: &VisualSelection,
    ) -> (Vec<[f32; 4]>, Option<[f32; 4]>) {
        let mut rects = match layout.coverage {
            theme::BarCoverage::SelectedOnly => Vec::new(),
            theme::BarCoverage::All => plan
                .rows()
                .iter()
                .filter(|r| r.item.is_some() && !vis.reads_selected(r.display))
                .map(|r| self.overlay_bar_plate(geom, layout, r))
                .collect(),
        };
        let footer_plate = if geom.hint_rows + geom.footer_rows > 0 {
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
            // THE PLATE BACKS THE FOOTER, not "everything below it".
            // Running it to the card's bottom edge is right for a card that HUGS
            // its content: the plate closes the card, and the two are the same
            // line. A WORKSPACE's card comes from the CANVAS instead, so there is
            // no bottom edge to close — the same rule paints a slab as tall as
            // whatever vertical space the rows did not use, hanging below the
            // footer's own glyphs with nothing in it. There the plate takes the
            // FOOTER'S OWN BAND, whose height is the row pitch LESS the amount a
            // footer line is shorter than a row — `overlay_footer_reclaim`, the
            // one owner of that difference, which the card height already reads.
            // PLUS the blank row `overlay_hint_gap_rows` reserves ahead of the
            // hint, at its own (item 293) compact height and reclaimed
            // alongside the hint's: omitting it here left this plate ending a
            // row above the hint text it exists to back, on any workspace
            // card carrying Bars plates.
            let gap_rows = overlay_hint_gap_rows(geom.hint_rows);
            let footer_band = (geom.hint_rows + gap_rows + geom.footer_rows) as f32
                * self.overlay_lh()
                - self.overlay_footer_reclaim(geom.hint_rows, gap_rows);
            let card_bottom = geom.card_y + geom.card_h;
            let plate_bottom = match geom.workspace {
                true => (plan.footer_top() + footer_band).min(card_bottom),
                false => card_bottom,
            };
            let plate = footer_plate_rect(
                plan.footer_top(),
                geom.band_x(),
                geom.band_w(),
                plate_bottom,
                footer_hug,
                self.metrics.scale,
            );
            rects.push(plate);
            Some(plate)
        } else {
            None
        };
        if geom.theme {
            // ITEM 316 — `item.is_none()` is a HEADER (always real glyphs) or
            // the card's own LOCATION line, glyph-free wherever
            // `LocationStyle::draws_inline()` is false (Cassowary's
            // `RotatedRail`, composed off-card). Reads the shaper's own gate,
            // not a named world, so it tracks any future non-inline style.
            let location_inline = theme::active().render_caps.location_style.draws_inline();
            for r in plan.rows().iter().filter(|r| r.item.is_none()) {
                let off = !location_inline
                    && matches!(geom.plan.get(r.display), Some(PlanLine::Location(_)));
                if off {
                    continue;
                }
                let (x, width) = layout.span(geom, r.display);
                rects.push([x, r.top + layout.bar_offset, width, layout.bar_height]);
            }
            rects.extend(self.overlay_strip_tab_plates.iter().copied());
        }
        (rects, footer_plate)
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
        /// A corner for no scrim at all on `Diagonal` (it emits no plates), kept
        /// so the shared pipeline is still prepared each frame.
        const DIAGONAL_SCRIM_CORNER: Logical = Logical(6.0);
        let pad = self.metrics.px(BAR_SCRIM_PAD);
        // The `BarePlates` gate above is the CARD's question, not the row's, and
        // that is deliberate here even though the same name misleads a plate
        // claim: every bare-plate world must have `panel_card` prepared each
        // frame, or a stale instance from the previous world survives into it.
        // `Diagonal` therefore arrives with an EMPTY `rects`, so its radius is a
        // corner for no scrim at all — never let that number be read as an
        // authored dial.
        let radius = match list_style {
            theme::ListStyle::Bars => self.metrics.px(Logical(
                crate::render::effective_bar_config().radius.max(0.0),
            )),
            theme::ListStyle::Diagonal(_) => self.metrics.px(DIAGONAL_SCRIM_CORNER),
            // Like `Diagonal`, a corner for no scrim at all — `Rules` emits
            // rules, not plates, and the gate below declines to pad them.
            theme::ListStyle::Rules(_) => 0.0,
            theme::ListStyle::Pane => 0.0,
        };
        // A SCRIM IS A PLATE'S OWN PAD, so only a plate-drawing style has one.
        // `Diagonal` reached here with an empty `rects` and was already
        // scrimless by accident of that emptiness; `Rules` emits real quads that
        // are emphatically not plates, so the question is asked directly.
        // Padding a hairline by `BAR_SCRIM_PAD` on every side is precisely how a
        // rule would become the object this style exists to not draw.
        let scrims = match list_style.draws_row_plates() {
            false => Vec::new(),
            true => rects
                .unselected
                .iter()
                .chain(rects.selected.iter())
                .map(|&plate| bar_scrim_rect(plate, pad))
                .collect::<Vec<_>>(),
        };
        self.panel_card.set_corner(radius + pad);
        self.panel_card
            .set_color(theme::overlay_bars_scrim().rgba_bytes());
        self.panel_card
            .prepare(device, queue, width, height, &scrims);
    }
}

#[cfg(test)]
mod tests;
