//! Selected overlay rows and facets: bands, bars, motion, marks, and draw probes.
//!
//! Carved out of [`super::overlay`] verbatim, no behaviour change. `TextPipeline`
//! lives in [`crate::render`], of which this is a descendant module, so these
//! methods keep full access to its private GPU fields; Rust merges the inherent
//! `impl TextPipeline` blocks across the module tree, so splitting the file is a
//! pure physical carve — the chrome pixels are byte-identical. See [`super`].

use super::*;

const FACET_CHIP_RADIUS: f32 = 6.0;

#[cfg(test)]
pub(in crate::render) struct OverlayYProbe {
    pub lh: f32,
    pub band_top: f32,
    pub sel_disp: usize,
    pub caret_center: f32,
    pub query_line_top: f32,
    pub query_line_height: f32,
    pub query_baseline: f32,
    pub primary: std::collections::BTreeMap<usize, f32>,
    pub secondary: std::collections::BTreeMap<usize, f32>,
    pub strip_baseline: Option<f32>,
    pub strip_line_bottom: Option<f32>,
    pub strip_underline_y: Option<f32>,
}

impl TextPipeline {
    /// ARM B LIVING-BAND PROBE — the DISPLAY rows the moving band covers THIS
    /// frame (see [`livingband::covered_rows`]), so the shaper flips those rows'
    /// ink to the on-band pole instead of the static selected row ("ink rides
    /// the band, not the state"). `None` on every ordinary run (env unset, a
    /// Bars or empty picker, or no selection), where the shaper is byte-identical
    /// (the old `overlay_selected` flip). Applies to BOTH the flat and the FACETED
    /// (Cmd-P palette / Settings) layouts — the target row is placed through the
    /// shared [`Self::overlay_selected_display_line`] owner so it matches the fill exactly
    /// on either. Reads the SAME phase + rects owner (`living_band_phase` /
    /// `living_band_rects`) `overlay_draw_card` draws from, so the flipped rows can
    /// never disagree with the fill's position — the exact phase-seam fix the
    /// outcome audit demands. The target row is placed through
    /// [`Self::overlay_selected_display_line`] — the ONE owner also read by the
    /// selected-band fill and the secondary right-column recolor — so the band, the
    /// hint recolor, and the flipped ink can never read a different row.
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
        let sel_disp = self.overlay_selected_display_line(geom)?;
        let lh = self.overlay_lh();
        let target = overlay_row_top(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            sel_disp,
            lh,
        );
        let (from, to, t) = self.living_band_phase(motion, target, lh);
        let (primary, echo, _cross) =
            self.living_band_rects(motion, from, to, t, geom.card_x, geom.card_w, lh);
        let bands: Vec<crate::render::livingband::BandRect> = primary
            .iter()
            .chain(echo.iter())
            .map(|r| crate::render::livingband::BandRect {
                top: r[1],
                height: r[3],
            })
            .collect();
        let first_top = overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, 0, lh);
        Some(crate::render::livingband::covered_rows(
            &bands,
            first_top,
            lh,
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
        let target = self
            .overlay_selected_display_line(geom)
            .expect("a selected row");
        let lh = self.overlay_lh();
        let first_top = overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, 0, lh);
        let sel_top = overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, target, lh);
        let (from, to, t) = self.living_band_phase(motion, sel_top, lh);
        let (primary, _echo, _cross) =
            self.living_band_rects(motion, from, to, t, geom.card_x, geom.card_w, lh);
        (covered, target, first_top, lh, primary[0])
    }

    /// TEST HOOK: total shaped glyphs the overlay text renderer would draw this
    /// frame (summed across the name buffer's layout runs). `0` once
    /// [`Self::park_overlay`] has emptied it — the assertion that a closed
    /// overlay carries no stale palette glyphs into the next frame.
    #[cfg(test)]
    pub(in crate::render) fn overlay_text_glyph_count(&self) -> usize {
        self.panel_buffer
            .layout_runs()
            .map(|r| r.glyphs.len())
            .sum()
    }

    /// ITEM 112 TEST HOOK — the absolute canvas box occupied by the shaped
    /// glyph CELLS on one primary overlay line. This is deliberately read from
    /// `panel_buffer`, the buffer the draw pass uploads, rather than rebuilt
    /// from row arithmetic: ordering and drawn↔hit-test laws can point at a
    /// title, facet, candidate, or footer glyph that actually exists and ask
    /// the production hit-test owners what that same point means.
    #[cfg(test)]
    pub(in crate::render) fn overlay_line_glyph_box(&self, line_i: usize) -> Option<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        let mut x0 = f32::INFINITY;
        let mut x1 = f32::NEG_INFINITY;
        let mut y0 = f32::INFINITY;
        let mut y1 = f32::NEG_INFINITY;
        for run in self
            .panel_buffer
            .layout_runs()
            .filter(|r| r.line_i == line_i)
        {
            if run.glyphs.is_empty() {
                continue;
            }
            let run_x0 = run.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
            let run_x1 = run
                .glyphs
                .iter()
                .map(|g| g.x + g.w)
                .fold(f32::NEG_INFINITY, f32::max);
            x0 = x0.min(geom.text_left + run_x0);
            x1 = x1.max(geom.text_left + run_x1);
            y0 = y0.min(geom.text_top + run.line_top);
            y1 = y1.max(geom.text_top + run.line_top + run.line_height);
        }
        (x0.is_finite() && x1 > x0 && y0.is_finite() && y1 > y0).then_some([
            x0,
            y0,
            x1 - x0,
            y1 - y0,
        ])
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_row_y_probe(&self) -> OverlayYProbe {
        use std::collections::BTreeMap;
        let geom = self.overlay_geometry(self.window_w as u32);
        let lh = self.overlay_lh();
        let header_rows = geom.header_rows;
        let last = header_rows
            + if geom.theme {
                geom.plan.len()
            } else {
                geom.visible
            };
        let mut primary = BTreeMap::new();
        for run in self.panel_buffer.layout_runs() {
            let li = run.line_i;
            if li >= header_rows && li < last {
                primary.insert(li - header_rows, geom.text_top + run.line_top);
            }
        }
        let sec_top = overlay_secondary_top(geom.text_top, geom.header_gap);
        let mut secondary = BTreeMap::new();
        for run in self.panel_bind_buffer.layout_runs() {
            let li = run.line_i;
            if li >= header_rows && li < last {
                secondary.insert(li - header_rows, sec_top + run.line_top);
            }
        }
        let sel_disp = if geom.theme {
            geom.plan
                .iter()
                .position(|l| matches!(l, ThemeLine::Item(i) if *i == self.overlay_selected))
                .unwrap_or(0)
        } else {
            self.overlay_selected.saturating_sub(geom.top_idx)
        };
        let band_top = overlay_row_top(geom.text_top, header_rows, geom.header_gap, sel_disp, lh);
        let mut strip_baseline = None;
        let mut strip_line_bottom = None;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i == 1 {
                strip_baseline = Some(geom.text_top + run.line_y);
                strip_line_bottom = Some(geom.text_top + run.line_top + run.line_height);
                break;
            }
        }
        let strip_underline_y = self.overlay_theme_underline.map(|q| q[1]);
        let query_run = self.panel_buffer.layout_runs().next();
        let query_line_height = query_run
            .as_ref()
            .map(|r| r.line_height)
            .unwrap_or_else(|| self.overlay_lh());
        let query_line_top = query_run
            .as_ref()
            .map(|r| geom.text_top + r.line_top)
            .unwrap_or(geom.text_top);
        let query_baseline = query_run
            .as_ref()
            .map(|r| geom.text_top + r.line_y)
            .unwrap_or(geom.text_top);
        OverlayYProbe {
            lh,
            band_top,
            sel_disp,
            caret_center: overlay_query_center(geom.text_top, query_line_height),
            query_line_top,
            query_line_height,
            query_baseline,
            primary,
            secondary,
            strip_baseline,
            strip_line_bottom,
            strip_underline_y,
        }
    }

    pub(in crate::render) fn overlay_pane_fills(&self, geom: &OverlayGeom) -> Vec<[f32; 4]> {
        let full = [geom.card_x, geom.card_y, geom.card_w, geom.card_h];
        if !matches!(
            crate::render::effective_pane_split(),
            theme::PaneSplit::Split
        ) {
            return vec![full];
        }
        let Some((gap_top, gap_bottom)) = super::overlay_split_bounds(
            geom.text_top,
            geom.header_rows,
            geom.header_gap,
            self.overlay_lh(),
        ) else {
            return vec![full];
        };
        let card_bottom = geom.card_y + geom.card_h;
        if gap_top > geom.card_y && gap_bottom < card_bottom && gap_bottom > gap_top {
            vec![
                [geom.card_x, geom.card_y, geom.card_w, gap_top - geom.card_y],
                [
                    geom.card_x,
                    gap_bottom,
                    geom.card_w,
                    card_bottom - gap_bottom,
                ],
            ]
        } else {
            vec![full]
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_pane_fills_probe(&self) -> Vec<[f32; 4]> {
        let geom = self.overlay_geometry(self.window_w as u32);
        self.overlay_pane_fills(&geom)
    }

    pub(super) fn overlay_draw_card(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
    ) {
        let lh = self.overlay_lh();
        let list_style = crate::render::effective_list_style();
        let spell = self.overlay_spell.is_some();
        let card_rect = [geom.card_x, geom.card_y, geom.card_w, geom.card_h];
        let backing = list_style.list_backing(spell);
        match backing {
            theme::ListBacking::BarePlates => {
                self.panel_shadow.prepare(device, queue, width, height, &[]);
                self.panel_border.prepare(device, queue, width, height, &[]);
            }
            theme::ListBacking::Card if spell => {
                let (chamfer_px, texture) = self.card_shape_texture(&[card_rect]);
                self.claim_float_panel(card_rect, FloatElevation::Rimmed, chamfer_px, texture);
                self.panel_card.prepare(device, queue, width, height, &[]);
                self.panel_shadow.prepare(device, queue, width, height, &[]);
                self.panel_border.prepare(device, queue, width, height, &[]);
            }
            theme::ListBacking::Card => {
                let fills = self.overlay_pane_fills(geom);
                self.prepare_panel_card_elevation(device, queue, width, height, &fills);
            }
        }

        // Selected-row highlight: a VALUE BAND, the next rung up the surface ladder
        // past the card's `base_300` (`theme::surface_selected`), set per-frame so a
        // live theme switch reskins it. Figure/ground by VALUE — not the cool
        // `selection` hue, not the amber accent (DESIGN §3/§5). The selected name
        // stays content ink, readable on the band. The band sits `header_rows` lines
        // below the card top (past the query line, if any), matching the shaped rows.
        //
        // TRUE 1-BIT WORLDS (`render_caps.selection_style ==
        // SelectionStyle::InverseVideo`): a flat fill would need SOME token
        // between `base_300`/`base_content` (both pure black/white here) to read
        // as "selected without erasing the row's own text" — no such token
        // exists on a one-bit world. The answer is a SOLID `base_content`
        // (white) band with the selected row's own glyphs recolored to solid
        // `base_300` (black) up in the shaper (`selected_ink`, threaded through
        // `overlay_shape_text`) — a hard black-on-white pair, gamma-independent
        // and CRISP. This supersedes the earlier framebuffer invert of the row
        // (`overlay_rows_invert`, retired): a `1 - dst` flip of the antialiased
        // near-white row text landed at a faint mid-grey (the Wagtail
        // selected-row low-contrast bug — see `HighlightTreatment::InverseFill`).
        // Both regimes now drive the ONE `overlay_rows` fill pipeline; the band
        // COLOR is the only thing that differs, so "prepare neither / draw text
        // that can't be read" is unreachable.
        // The `ValueBand` band VALUE is the PALETTE-COMPOSITION round's
        // strengthened, calm-by-VALUE band (`effective_overlay_selrow_band`, one
        // ramp step past the shared `surface_selected`; the gallery A/Bs it and
        // the old band is one line away — see that fn's REVERT note). Never a hue
        // (DESIGN §3/§5); the distinguishability sweep polices it.
        let band_color = match theme::active()
            .highlight_treatment(crate::render::effective_overlay_selrow_band())
        {
            theme::HighlightTreatment::ValueBand(color) => color,
            theme::HighlightTreatment::InverseFill { band, .. } => band,
        };
        self.overlay_rows.set_color(band_color.rgba_bytes());
        let sel_disp: Option<usize> = self.overlay_selected_display_line(geom);
        // PER-ITEM LIST SURFACES round: `Pane` (default) draws the byte-identical
        // full-width selected BAND; `Bars` gives each candidate row its own
        // rounded surface (unselected → `overlay_bars`, quiet; selected →
        // `overlay_rows`, brighter + `grow_px` wider) with the gap already folded
        // into `lh`. The row-y owner `overlay_row_top` feeds BOTH so bars and text
        // agree on every row; the hit-test rides the same `lh`, so a click in a
        // gap maps to the nearest row (no dead zones).
        // `list_style` computed once at the top of this fn (drives the pane-drop).
        // ITEM 45: the selected-bar growth mirror follows the FROZEN alignment (via
        // the ONE owner), so it composes with the (also-frozen) card placement and
        // never flips mid-preview when the active world changes.
        let mirror = crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth();
        // ARM B LIVING BAND (`AWL_LIVING_BAND`): the selection band's morph /
        // two-shape choreography. Ships ON (calm MORPH) — `None` only when the knob
        // is `off`. Pane-only; when active it OWNS the band rects (the ordinary
        // `overlay_band_drawn` slide is skipped for that frame). A settled frame is
        // byte-identical to the ordinary band (MORPH is calm-at-rest and
        // `living_band_phase` settles every capture / Reduce-Motion frame).
        let motion = crate::render::livingband::overlay_motion_force();
        let sel_target: Option<f32> = sel_disp.map(|disp| {
            overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, disp, lh)
        });
        let living: Option<(crate::render::livingband::MotionForce, f32, f32, f32)> =
            match (motion, sel_target) {
                (Some(force), Some(target)) if matches!(list_style, theme::ListStyle::Pane) => {
                    let (from, to, t) = self.living_band_phase(force, target, lh);
                    Some((force, from, to, t))
                }
                _ => None,
            };
        let sel_top: Option<f32> = match (living.is_some(), sel_target) {
            (true, _) => None,
            (false, Some(target)) => Some(self.overlay_band_drawn(target)),
            (false, None) => None,
        };
        let mut cross_rects: Vec<[f32; 4]> = Vec::new();
        let (sel_rects, bar_rects): (Vec<[f32; 4]>, Vec<[f32; 4]>) = match list_style {
            theme::ListStyle::Pane => {
                if let Some((force, from, to, t)) = living {
                    let (primary, echo, cross) =
                        self.living_band_rects(force, from, to, t, geom.card_x, geom.card_w, lh);
                    self.overlay_bars.set_corner(2.5);
                    self.overlay_bars
                        .set_color(theme::surface_selected().rgba_bytes());
                    self.overlay_cross.set_corner(2.5);
                    self.overlay_cross
                        .set_color(theme::overlay_band_overlap().rgba_bytes());
                    cross_rects = cross;
                    (primary, echo)
                } else {
                    let rects = match (sel_disp, sel_top) {
                        (Some(disp), Some(top)) => {
                            let dx = self.overlay_slant_dx(disp);
                            vec![[geom.card_x + dx, top, geom.card_w - dx, lh]]
                        }
                        _ => Vec::new(),
                    };
                    (rects, Vec::new())
                }
            }
            theme::ListStyle::Bars {
                radius,
                gap,
                grow_px,
                extent,
                coverage,
            } => {
                let r = radius.max(0.0);
                let g = gap.max(0.0);
                let bar_h = (lh - g).max(1.0);
                let hug = extent.hugs();
                let primary_px = if hug {
                    self.overlay_row_primary_px(geom)
                } else {
                    std::collections::BTreeMap::new()
                };
                let chord_px = if hug && !extent.inline_shortcut() && self.overlay_right_shown {
                    self.overlay_row_secondary_px(geom)
                } else {
                    std::collections::BTreeMap::new()
                };
                let span_of = |k: usize| -> (f32, f32) {
                    if hug {
                        super::bar_hug_span(
                            geom.card_x,
                            geom.card_w,
                            geom.text_left,
                            primary_px.get(&k).copied().unwrap_or(0.0),
                        )
                    } else {
                        super::bar_full_span(geom.card_x, geom.card_w)
                    }
                };
                let bar_off = g * 0.5;
                self.overlay_rows.set_corner(r);
                self.overlay_bars.set_corner(r);
                self.overlay_rows.set_stroke(0.0);
                self.overlay_bars.set_stroke(0.0);
                self.overlay_bars
                    .set_color(theme::overlay_bar_unselected().rgba_bytes());
                let item_rows: Vec<usize> = if geom.theme {
                    geom.plan
                        .iter()
                        .enumerate()
                        .filter_map(|(k, l)| matches!(l, ThemeLine::Item(_)).then_some(k))
                        .collect()
                } else {
                    (0..geom.visible).collect()
                };
                let mut unsel: Vec<[f32; 4]> = match coverage {
                    theme::BarCoverage::SelectedOnly => Vec::new(),
                    theme::BarCoverage::All => item_rows
                        .iter()
                        .copied()
                        .filter(|k| Some(*k) != sel_disp)
                        .map(|k| {
                            let top = overlay_row_top(
                                geom.text_top,
                                geom.header_rows,
                                geom.header_gap,
                                k,
                                lh,
                            );
                            let (x, w) = span_of(k);
                            let (x, w) = slant_bar_span(x, w, hug, self.overlay_slant_dx(k));
                            [x, top + bar_off, w, bar_h]
                        })
                        .collect(),
                };
                if geom.hint_rows + geom.footer_rows > 0 {
                    let content_rows = if geom.theme {
                        geom.plan.len()
                    } else {
                        geom.visible + geom.empty.is_some() as usize
                    };
                    let footer_hug = hug.then(|| {
                        (
                            geom.text_left,
                            self.overlay_footer_content_px(geom, content_rows),
                        )
                    });
                    unsel.push(super::footer_plate_rect(
                        geom.text_top,
                        geom.header_rows,
                        geom.header_gap,
                        content_rows,
                        lh,
                        geom.card_x,
                        geom.card_w,
                        geom.card_y + geom.card_h,
                        footer_hug,
                    ));
                }
                if geom.theme {
                    for (k, line) in geom.plan.iter().enumerate() {
                        if !matches!(line, ThemeLine::Header(_)) {
                            continue;
                        }
                        let top = overlay_row_top(
                            geom.text_top,
                            geom.header_rows,
                            geom.header_gap,
                            k,
                            lh,
                        );
                        let (x, w) = span_of(k);
                        unsel.push([x, top + bar_off, w, bar_h]);
                    }
                    unsel.extend(self.overlay_strip_tab_plates.iter().copied());
                }
                let sel = match (sel_disp, sel_top) {
                    (Some(disp), Some(top)) => {
                        let (bx, bw) = span_of(disp);
                        let (bx, bw) = slant_bar_span(bx, bw, hug, self.overlay_slant_dx(disp));
                        // GROW-POP (choreography 4): the `grow_px` ledge eases in on
                        // each selection move via the ONE `overlay_grow_progress`
                        // owner. Full `grow_px` in every capture (byte-identical).
                        let grow = grow_px * self.overlay_grow_progress();
                        let (x, w) = super::grow_span(bx, bw, grow, mirror);
                        vec![[x, top + bar_off, w.max(1.0), bar_h]]
                    }
                    _ => Vec::new(),
                };
                let mut sel = sel;
                if !chord_px.is_empty() {
                    let (fx, fw) = super::bar_full_span(geom.card_x, geom.card_w);
                    let full_right = fx + fw;
                    let chord_right = geom.text_left + geom.text_w;
                    let chord_plate = |k: usize, top: f32| -> [f32; 4] {
                        let w_c = chord_px.get(&k).copied().unwrap_or(0.0);
                        let right = (chord_right + super::BAR_TEXT_PAD).min(full_right);
                        let plate_w = w_c + 2.0 * super::BAR_TEXT_PAD;
                        let left = (right - plate_w).max(fx);
                        [left, top + bar_off, (right - left).max(1.0), bar_h]
                    };
                    let row_top = |k: usize| {
                        overlay_row_top(geom.text_top, geom.header_rows, geom.header_gap, k, lh)
                    };
                    for &k in &item_rows {
                        if !chord_px.contains_key(&k) {
                            continue;
                        }
                        if Some(k) == sel_disp {
                            sel.push(chord_plate(k, row_top(k)));
                        } else if coverage == theme::BarCoverage::All {
                            unsel.push(chord_plate(k, row_top(k)));
                        }
                    }
                }
                (sel, unsel)
            }
        };
        if backing == theme::ListBacking::BarePlates {
            const SCRIM_PAD: f32 = 2.0;
            let radius = match list_style {
                theme::ListStyle::Bars { radius, .. } => radius.max(0.0),
                theme::ListStyle::Pane => 0.0,
            };
            let scrims: Vec<[f32; 4]> = bar_rects
                .iter()
                .chain(sel_rects.iter())
                .map(|&[x, y, w, h]| {
                    [
                        x - SCRIM_PAD,
                        y - SCRIM_PAD,
                        w + 2.0 * SCRIM_PAD,
                        h + 2.0 * SCRIM_PAD,
                    ]
                })
                .collect();
            self.panel_card.set_corner(radius + SCRIM_PAD);
            self.panel_card
                .set_color(theme::overlay_bars_scrim().rgba_bytes());
            self.panel_card
                .prepare(device, queue, width, height, &scrims);
        }
        self.overlay_bars
            .prepare(device, queue, width, height, &bar_rects);
        self.overlay_rows
            .prepare(device, queue, width, height, &sel_rects);
        self.overlay_cross
            .prepare(device, queue, width, height, &cross_rects);
        // ITEM 94 — THE RANGE ROW'S RAIL. Every visible range row's track / fill /
        // thumb, resolved by the ONE rail owner (`overlay_rails`, which the pointer
        // hit-test reads too — so the control is clickable exactly where it is
        // drawn). EMPTY for every other card (both pipelines park → byte-identical).
        //
        // INK: two quiet rungs, never the amber accent (DESIGN §3) — `faint` for the
        // track, `muted` for the fill + thumb. When the SELECTED row carries a rail
        // AND the highlight band would wash `muted` out, the fill/thumb flip through
        // the ONE `theme::selected_row_secondary_ink` owner — the SAME mechanism the
        // value TEXT beside it already uses, so rail and number stay legible together
        // on every world rather than either growing its own contrast rule.
        let rails = self.overlay_rails(geom);
        let (mut track_rects, mut thumb_rects): (Vec<[f32; 4]>, Vec<[f32; 4]>) =
            (Vec::new(), Vec::new());
        for (_item, rail) in &rails {
            track_rects.push(rail.track);
            if rail.fill[2] > 0.0 {
                thumb_rects.push(rail.fill);
            }
            thumb_rects.push(rail.thumb);
        }
        let selected_rail = rails.iter().any(|(item, _)| {
            Some(*item) == sel_disp.and_then(|k| self.overlay_item_at_row(geom, k))
        });
        let thumb_ink = if selected_rail && super::selected_secondary_on_band() {
            match theme::active()
                .highlight_treatment(crate::render::effective_overlay_selrow_band())
            {
                theme::HighlightTreatment::InverseFill { ink, .. } => ink,
                theme::HighlightTreatment::ValueBand(b) => theme::selected_row_secondary_ink(b),
            }
        } else {
            theme::muted()
        };
        self.overlay_range_track
            .set_color(theme::faint().rgba_bytes());
        self.overlay_range_thumb.set_color(thumb_ink.rgba_bytes());
        self.overlay_range_track
            .prepare(device, queue, width, height, &track_rects);
        self.overlay_range_thumb
            .prepare(device, queue, width, height, &thumb_rects);
        // FACETED STRIP active-lens mark: the rect the shaper recorded (its SHAPE
        // set by `facet_style` — hairline underline / band / active chip); a
        // non-theme card parks it empty (so a stale rect never lingers).
        let underline: Vec<[f32; 4]> = if geom.theme {
            self.overlay_theme_underline.iter().copied().collect()
        } else {
            Vec::new()
        };
        // PER-ITEM LIST SURFACES round: `Text` (default) keeps the content-ink
        // hairline byte-identically; `Band` recolors the ACTIVE mark to the
        // selected-row band VALUE (never amber) and rounds it into a pill.
        // V6 P5 [`theme::FacetStyle::Chips`] — REAL chips (third attempt): the
        // ACTIVE label rides `overlay_lens_underline` as a FILLED value pill
        // (same as `Band`), and EACH INACTIVE label draws a GHOST pill — a MUTED
        // hairline STROKE — via `overlay_facet_ghost`. Both are recorded from the
        // SAME shaped strip glyphs (in `overlay_shape_theme`), so the skin can't
        // disagree with the hit-test.
        let facet_style = crate::render::effective_facet_style();
        let mut ghosts: Vec<[f32; 4]> = Vec::new();
        let band = match theme::active()
            .highlight_treatment(crate::render::effective_overlay_selrow_band())
        {
            theme::HighlightTreatment::ValueBand(c) => c,
            theme::HighlightTreatment::InverseFill { band, .. } => band,
        };
        match facet_style {
            theme::FacetStyle::Text => {}
            theme::FacetStyle::Band => {
                self.overlay_lens_underline.set_color(band.rgba_bytes());
                self.overlay_lens_underline.set_corner(FACET_CHIP_RADIUS);
                self.overlay_lens_underline.set_stroke(0.0);
            }
            theme::FacetStyle::Chips(v) => {
                use theme::ChipVariant as V;
                let content = theme::base_content();
                let muted = theme::muted();
                let stroke = crate::render::BAR_OUTLINE_STROKE_PX;
                let (a_fill, a_corner, a_stroke): ([u8; 4], f32, f32) = match v {
                    V::Hairline => (band.rgba_bytes(), FACET_CHIP_RADIUS, 0.0),
                    V::FilledActive => (content.rgba_bytes(), FACET_CHIP_RADIUS, 0.0),
                    V::Underline => (content.rgba_bytes(), 1.75, 0.0),
                    V::Bracket => (content.rgba_bytes(), 0.0, 0.0),
                };
                self.overlay_lens_underline.set_color(a_fill);
                self.overlay_lens_underline.set_corner(a_corner);
                self.overlay_lens_underline.set_stroke(a_stroke);
                let (g_color, g_corner, g_stroke): ([u8; 4], f32, f32) = match v {
                    V::Hairline => (muted.rgba_bytes(), FACET_CHIP_RADIUS, stroke),
                    V::Bracket => (content.rgba_bytes(), 0.0, 0.0),
                    V::FilledActive | V::Underline => {
                        (muted.rgba_bytes(), FACET_CHIP_RADIUS, stroke)
                    }
                };
                self.overlay_facet_ghost.set_color(g_color);
                self.overlay_facet_ghost.set_corner(g_corner);
                self.overlay_facet_ghost.set_stroke(g_stroke);
                if geom.theme {
                    ghosts = self.overlay_theme_facet_ghosts.clone();
                }
            }
        }
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &underline);
        if !matches!(facet_style, theme::FacetStyle::Chips(_)) {
            self.overlay_facet_ghost.set_corner(FACET_CHIP_RADIUS);
            self.overlay_facet_ghost
                .set_stroke(crate::render::BAR_OUTLINE_STROKE_PX);
        }
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &ghosts);
    }
}
