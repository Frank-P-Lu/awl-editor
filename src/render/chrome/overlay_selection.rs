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
    line_height: f32,
    bar_height: f32,
    bar_offset: f32,
    primary_px: std::collections::BTreeMap<usize, f32>,
    chord_px: std::collections::BTreeMap<usize, f32>,
    item_rows: Vec<usize>,
}

impl OverlayBarLayout {
    fn span(&self, geom: &OverlayGeom, row: usize) -> (f32, f32) {
        if self.extent.hugs() {
            bar_hug_span(
                geom.card_x,
                geom.card_w,
                geom.text_left,
                self.primary_px.get(&row).copied().unwrap_or(0.0),
            )
        } else {
            bar_full_span(geom.card_x, geom.card_w)
        }
    }

    fn row_top(&self, geom: &OverlayGeom, row: usize) -> f32 {
        overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            row,
            self.line_height,
        )
    }

    fn append_chord_plates(
        &self,
        geom: &OverlayGeom,
        selected_row: Option<usize>,
        selected: &mut Vec<[f32; 4]>,
        unselected: &mut Vec<[f32; 4]>,
    ) {
        if self.chord_px.is_empty() {
            return;
        }
        let (full_x, full_width) = bar_full_span(geom.card_x, geom.card_w);
        let full_right = full_x + full_width;
        let chord_right = geom.text_left + geom.text_w;
        for &row in &self.item_rows {
            let Some(width) = self.chord_px.get(&row).copied() else {
                continue;
            };
            let right = (chord_right + BAR_TEXT_PAD).min(full_right);
            let plate_width = width + 2.0 * BAR_TEXT_PAD;
            let left = (right - plate_width).max(full_x);
            let rect = [
                left,
                self.row_top(geom, row) + self.bar_offset,
                (right - left).max(1.0),
                self.bar_height,
            ];
            if Some(row) == selected_row {
                selected.push(rect);
            } else if self.coverage == theme::BarCoverage::All {
                unselected.push(rect);
            }
        }
    }
}

impl TextPipeline {
    /// Display rows covered by the living Pane band this frame. The shaper
    /// reads this same selection owner, so on-band ink and fill share one phase.
    pub(in crate::render) fn living_covered_rows(
        &mut self,
        geom: &OverlayGeom,
    ) -> Option<Vec<usize>> {
        let motion = crate::render::livingband::overlay_motion_force()?;
        if !matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Pane
        ) {
            return None;
        }
        let selected_row = self.overlay_selected_display_line(geom)?;
        let line_height = self.overlay_lh();
        let target = overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            selected_row,
            line_height,
        );
        let (from, to, t) = self.living_band_phase(motion, target, line_height);
        let (primary, echo, _) =
            self.living_band_rects(motion, from, to, t, geom.card_x, geom.card_w, line_height);
        let bands = primary
            .iter()
            .chain(echo.iter())
            .map(|rect| crate::render::livingband::BandRect {
                top: rect[1],
                height: rect[3],
            })
            .collect::<Vec<_>>();
        let first_top = overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            0,
            line_height,
        );
        Some(crate::render::livingband::covered_rows(
            &bands,
            first_top,
            line_height,
            geom.visible,
        ))
    }

    #[cfg(test)]
    pub(in crate::render) fn living_probe_geom(
        &mut self,
        geom: &OverlayGeom,
    ) -> (Vec<usize>, usize, f32, f32, [f32; 4]) {
        let motion = crate::render::livingband::overlay_motion_force()
            .expect("living_probe_geom needs the motion probe armed");
        let covered = self.living_covered_rows(geom).unwrap_or_default();
        let selected_row = self
            .overlay_selected_display_line(geom)
            .expect("a selected row");
        let line_height = self.overlay_lh();
        let first_top = overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            0,
            line_height,
        );
        let selected_top = overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            selected_row,
            line_height,
        );
        let (from, to, t) = self.living_band_phase(motion, selected_top, line_height);
        let (primary, _, _) =
            self.living_band_rects(motion, from, to, t, geom.card_x, geom.card_w, line_height);
        (covered, selected_row, first_top, line_height, primary[0])
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn overlay_prepare_selection(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        list_style: theme::ListStyle,
        backing: theme::ListBacking,
        selected_row: Option<usize>,
    ) {
        let band_color = match theme::active()
            .highlight_treatment(crate::render::effective_overlay_selrow_band())
        {
            theme::HighlightTreatment::ValueBand(color) => color,
            theme::HighlightTreatment::InverseFill { band, .. } => band,
        };
        self.overlay_rows.set_color(band_color.rgba_bytes());
        let rects = match list_style {
            theme::ListStyle::Pane => self.overlay_pane_selection(geom, selected_row),
            theme::ListStyle::Bars {
                radius,
                gap,
                grow_px,
                extent,
                coverage,
            } => self.overlay_bar_selection(
                geom,
                selected_row,
                radius,
                gap,
                grow_px,
                extent,
                coverage,
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

    fn overlay_pane_selection(
        &mut self,
        geom: &OverlayGeom,
        selected_row: Option<usize>,
    ) -> OverlaySelectionRects {
        let line_height = self.overlay_lh();
        let target = selected_row.map(|row| {
            overlay_row_top(
                geom.text_top,
                geom.header_rows,
                geom.header_gap,
                row,
                line_height,
            )
        });
        if let (Some(force), Some(target)) =
            (crate::render::livingband::overlay_motion_force(), target)
        {
            let (from, to, t) = self.living_band_phase(force, target, line_height);
            let (selected, unselected, cross) =
                self.living_band_rects(force, from, to, t, geom.card_x, geom.card_w, line_height);
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
        let selected = match (selected_row, target) {
            (Some(row), Some(target)) => {
                let top = self.overlay_band_drawn(target);
                let dx = self.overlay_slant_dx(row);
                vec![[geom.card_x + dx, top, geom.card_w - dx, line_height]]
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
        selected_row: Option<usize>,
        radius: f32,
        gap: f32,
        grow_px: f32,
        extent: theme::BarExtent,
        coverage: theme::BarCoverage,
    ) -> OverlaySelectionRects {
        let layout = self.overlay_bar_layout(geom, radius, gap, grow_px, extent, coverage);
        self.overlay_rows.set_corner(layout.radius);
        self.overlay_bars.set_corner(layout.radius);
        self.overlay_rows.set_stroke(0.0);
        self.overlay_bars.set_stroke(0.0);
        self.overlay_bars
            .set_color(theme::overlay_bar_unselected().rgba_bytes());
        let mut unselected = self.overlay_unselected_bar_rects(geom, &layout, selected_row);
        let mut selected = self.overlay_selected_bar_rects(geom, &layout, selected_row);
        layout.append_chord_plates(geom, selected_row, &mut selected, &mut unselected);
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
        radius: f32,
        gap: f32,
        grow_px: f32,
        extent: theme::BarExtent,
        coverage: theme::BarCoverage,
    ) -> OverlayBarLayout {
        let line_height = self.overlay_lh();
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
        let item_rows = if geom.theme {
            geom.plan
                .iter()
                .enumerate()
                .filter_map(|(row, line)| matches!(line, ThemeLine::Item(_)).then_some(row))
                .collect()
        } else {
            (0..geom.visible).collect()
        };
        OverlayBarLayout {
            radius: radius.max(0.0),
            grow_px,
            extent,
            coverage,
            line_height,
            bar_height: (line_height - gap).max(1.0),
            bar_offset: gap * 0.5,
            primary_px,
            chord_px,
            item_rows,
        }
    }

    fn overlay_unselected_bar_rects(
        &self,
        geom: &OverlayGeom,
        layout: &OverlayBarLayout,
        selected_row: Option<usize>,
    ) -> Vec<[f32; 4]> {
        let mut rects = match layout.coverage {
            theme::BarCoverage::SelectedOnly => Vec::new(),
            theme::BarCoverage::All => layout
                .item_rows
                .iter()
                .copied()
                .filter(|row| Some(*row) != selected_row)
                .map(|row| self.overlay_bar_plate(geom, layout, row))
                .collect(),
        };
        if geom.hint_rows + geom.footer_rows > 0 {
            let content_rows = if geom.theme {
                geom.plan.len()
            } else {
                geom.visible + geom.empty.is_some() as usize
            };
            let footer_hug = layout.extent.hugs().then(|| {
                (
                    geom.text_left,
                    self.overlay_footer_content_px(geom, content_rows),
                )
            });
            rects.push(footer_plate_rect(
                geom.text_top,
                geom.header_rows,
                geom.header_gap,
                content_rows,
                layout.line_height,
                geom.card_x,
                geom.card_w,
                geom.card_y + geom.card_h,
                footer_hug,
            ));
        }
        if geom.theme {
            for (row, line) in geom.plan.iter().enumerate() {
                if matches!(line, ThemeLine::Header(_)) {
                    let top = layout.row_top(geom, row);
                    let (x, width) = layout.span(geom, row);
                    rects.push([x, top + layout.bar_offset, width, layout.bar_height]);
                }
            }
            rects.extend(self.overlay_strip_tab_plates.iter().copied());
        }
        rects
    }

    fn overlay_bar_plate(
        &self,
        geom: &OverlayGeom,
        layout: &OverlayBarLayout,
        row: usize,
    ) -> [f32; 4] {
        let top = layout.row_top(geom, row);
        let (x, width) = layout.span(geom, row);
        let (x, width) = slant_bar_span(x, width, layout.extent.hugs(), self.overlay_slant_dx(row));
        [x, top + layout.bar_offset, width, layout.bar_height]
    }

    fn overlay_selected_bar_rects(
        &mut self,
        geom: &OverlayGeom,
        layout: &OverlayBarLayout,
        selected_row: Option<usize>,
    ) -> Vec<[f32; 4]> {
        let Some(row) = selected_row else {
            return Vec::new();
        };
        let target = layout.row_top(geom, row);
        let top = self.overlay_band_drawn(target);
        let (x, width) = layout.span(geom, row);
        let (x, width) = slant_bar_span(x, width, layout.extent.hugs(), self.overlay_slant_dx(row));
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
