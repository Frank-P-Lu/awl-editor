use super::overlay_clamp::window_plan;
use super::*;

mod spans;
use crate::render::rotated_location::LOCATION_SCALE;

/// Pixels the active-lens UNDERLINE sits BELOW the strip run's shaped baseline
/// (`overlay_shape_theme`). Small so the rule hugs the label — enough to clear
/// the baseline for every chrome/mono/display face without striking the glyphs.
const UNDERLINE_BASELINE_DROP: Logical = Logical(2.0);

/// Stroke weight of the `FacetStyle::Text` active-lens hairline. `Logical` +
/// `Metrics::px` is the same pixel-space boundary every other chrome length
/// in this file crosses, so this stroke scales with every other term of its
/// own rect (position, span) rather than staying a fixed device size.
const TEXT_MARK_THICKNESS: Logical = Logical(1.5);

/// Stroke weight of the `FacetStyle::Chips(ChipVariant::Underline)` active-lens
/// mark — thicker than the plain `Text` hairline so it still reads as a chip
/// rather than a rule, but crossing the same `Logical` + `Metrics::px`
/// boundary.
const UNDERLINE_CHIP_THICKNESS: Logical = Logical(3.5);

impl TextPipeline {
    /// The shared docked-facet line: the shaped strip keeps its ordinary x
    /// spans and typography, but its line box seats immediately above the
    /// card so its bottom edge and the pane's top edge are the same edge.
    pub(in crate::render) fn docked_facet_band(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<crate::render::plan::PlannedHeader> {
        let strip = plan.strip_band()?;
        // The grouped strip's planned box also owns the calm beat below its
        // glyph line. Dock only the glyph-bearing line: carrying that folded
        // beat above the pane makes the tab taller than the available top
        // margin on a narrow canvas.
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

    /// THEME PICKER display plan: the candidate-area sequence of section HEADERS +
    /// world ROWS, from the parallel `overlay_sections`. A header is emitted before a
    /// row whenever its section differs from the previous row's (so contiguous groups
    /// get one header each); the All lens / non-grouped rows emit no headers. Section
    /// labels are uppercased for the faint header display. Shared by the geometry,
    /// shaping, selected-band, and hit-test so they can never disagree.
    ///
    /// **A SECTION WHOSE LABEL IS THE CARD'S OWN LOCATION IS NOT A SECTION.**
    /// When the group being headed is the very place the picker is
    /// (`overlay_location`, the active lens), that line is the SECOND LEVEL of
    /// the card's heading hierarchy rather than chrome dividing a list into
    /// parts, and is planned as [`PlanLine::Location`] so the shaper can say so.
    /// Its SLOT is unchanged: the defect was a heading in a list's voice.
    pub(in crate::render) fn theme_plan(&self) -> Vec<PlanLine> {
        let mut out = Vec::with_capacity(self.overlay_items.len());
        let mut prev: Option<String> = None;
        let location = self.overlay_location.as_deref();
        for i in 0..self.overlay_items.len() {
            let sect = self
                .overlay_sections
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !sect.is_empty() && prev.as_deref() != Some(sect) {
                out.push(match location == Some(sect) {
                    true => PlanLine::Location(sect.to_string()),
                    false => PlanLine::Header(sect.to_uppercase()),
                });
            }
            out.push(PlanLine::Item(i));
            prev = if sect.is_empty() {
                None
            } else {
                Some(sect.to_string())
            };
        }
        out
    }

    pub(super) fn theme_overlay_geometry(&self, width: u32) -> OverlayGeom {
        let lh = self.overlay_lh();
        // THE SAME THREE TOKENS THE FLAT FAMILY PLACES ITS CARD WITH — they were
        // a second copy of the same literals here, which is how the grouped
        // card kept a physical 24px of vertical pad on a retina panel while
        // every other quantity in its own height doubled.
        let pad = self.metrics.px(super::CARD_PAD);
        let margin = self.metrics.px(super::CARD_MARGIN);
        let n_items = self.overlay_items.len();
        let full_plan = self.theme_plan();
        let mut hint = self.overlay_hint.clone();
        let hint_rows = if hint.is_empty() { 0 } else { 1 };
        // See `overlay_hint_gap_rows`'s own doc (`chrome/mod.rs`) — the ONE owner
        // this grouped family shares with the flat family (`overlay.rs`) and the
        // workspace family (`workspace.rs`), so a hint can't stop sitting flush
        // against the last row in one family while still doing it in another.
        let mut hint_gap_rows = overlay_hint_gap_rows(hint_rows);
        let (footer, footer_rows) = self.overlay_footer_lines();
        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;
        let header_rows = 2;
        let header_gap = self.overlay_header_gap();
        let card_y = margin + self.metrics.px(super::CARD_TOP_DROP) + self.menubar_reserve();
        let total_headers = full_plan.len() - n_items;
        // Strip + hint + footer here, at `min_items: 0`; the SECTION headers are
        // charged to the drawn WINDOW (`fit_sectioned_item_rows`).
        let chrome_rows = header_rows + hint_gap_rows + hint_rows + empty_rows + footer_rows;
        // THE ONE HEIGHT-CLAMP OWNER, shared with the flat family.
        let avail_px = (self.window_h - card_y - margin - 2.0 * pad - header_gap).max(lh);
        let item_cap = self.overlay_sectioned_item_cap(avail_px, lh, chrome_rows, total_headers, 0);
        let (item_top, item_visible) = scroll_window(
            n_items,
            self.overlay_selected,
            self.overlay_scroll,
            item_cap,
        );
        let plan = window_plan(&full_plan, item_top, item_top + item_visible);
        let mut total_rows =
            header_rows + plan.len() + empty_rows + hint_gap_rows + hint_rows + footer_rows;
        // Wider than the flat pickers so the whole lens strip (Time … All) fits on
        // one line even on a WIDE mono world face without the far-right All clipping
        // — via the SAME horizontal-box owner (edge inset + narrow-window fallback),
        // just a slightly wider cap ([`CARD_MAX_W_FACETED`]). Scaled to the current
        // zoom/DPI by the ONE owner [`TextPipeline::overlay_card_desired_w`], so the
        // faceted card (the Cmd-P palette) grows WITH the glyphs like the flat one —
        // otherwise the 600 cap stayed unzoomed while the text doubled and every row
        // elided (the zoom-blind card bug).
        // Content-hug for a RIGHT-ANCHORED faceted card (via the ONE
        // `overlay_desired_w` owner), the wide `CARD_MAX_W_FACETED` cap otherwise.
        let desired_w = self.overlay_desired_w(super::CARD_MAX_W_FACETED);
        let (card_x, card_w) = self.overlay_card_box(width, desired_w);
        let card_narrow =
            super::overlay_card_fill_regime(width as f32, desired_w, self.metrics.scale);
        let hpad = self.overlay_text_hpad();
        let text_w = card_w - 2.0 * hpad;
        hint = super::hint_yielding_explanation(&hint, width as f32 / self.metrics.scale.max(0.01));
        let mut card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        // The hint gap is decorative breathing room, not load-bearing chrome:
        // in the starvation corner (a sectioned card's own fixed
        // header/hint/footer overhead, at the `min_items: 0` floor, already
        // outgrowing the canvas at an extreme zoom), drop it rather than push
        // the card past the canvas — the same degrade the flat family's own
        // arm takes (`overlay.rs::overlay_geometry`).
        if card_y + card_h > self.window_h + 0.01 && hint_gap_rows > 0 {
            total_rows -= hint_gap_rows;
            hint_gap_rows = 0;
            card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        }
        let card_y = card_y + self.overlay_entrance_offset();
        let text_left = card_x + hpad;
        let text_top = card_y + pad;
        OverlayGeom {
            // The DRAWN window: `visible` = candidate DISPLAY LINES shown (headers + item
            // rows), `top_idx` = the first ITEM shown (0 when the whole list fits). The
            // theme draw/hit-test read `plan` directly (already the windowed slice), so
            // these feed the sidecar window report, not the row math.
            visible: plan.len(),
            top_idx: item_top,
            n_items,
            hint,
            hint_rows,
            hint_gap_rows,
            footer,
            footer_rows,
            theme: true,
            strip: self.overlay_lens.clone(),
            plan,
            header_rows,
            header_gap,
            empty,
            card_x,
            card_y,
            card_w,
            card_h,
            text_left,
            text_top,
            text_w,
            row_text: None,
            card_narrow,
            // The GROUPED CARD floats over the document, so it has no navigation
            // rail and its content band IS its card (`OverlayGeom::band_x`).
            workspace: false,
            rail: None,
            pane_x: 0.0,
            pane_w: 0.0,
            rows_focused: false,
        }
    }

    /// FACETED PICKER: hit-test a pointer against the lens STRIP (display line 1),
    /// returning the STRIP INDEX (into the picker's [`crate::facets::FacetScheme::strip`])
    /// the label under `(px, py)` selects — so a CLICK on a lens switches the facet (the
    /// pointing counterpart to LEFT/RIGHT). `None` off the strip row, off the card, or for
    /// a non-faceting overlay. Uses the same per-lens byte ranges the shaper laid out, read
    /// back from the shaped strip glyphs so the hit lands on the same label the eye sees.
    pub fn overlay_lens_at(&self, px: f32, py: f32) -> Option<usize> {
        if !self.overlay_active || self.overlay_lens.is_empty() {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        if !geom.theme || px < geom.card_x || px > geom.card_x + geom.card_w {
            return None;
        }
        // The strip's clickable band is its PLANNED line box — the same object
        // the mark centre and the strip's own glyph metrics read. A pointer entry
        // point has no frame to ride, so it plans freshly, still O(visible).
        let plan = self.overlay_row_plan(&geom);
        let strip = self
            .docked_facet_band(&geom, &plan)
            .or_else(|| plan.strip_band())?;
        if !strip.contains(py) {
            return None;
        }
        let want = px - geom.text_left;
        let mut hit: Option<usize> = None;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i != 1 {
                continue;
            }
            let mut s = String::from("\n");
            let mut ranges: Vec<(usize, std::ops::Range<usize>)> = Vec::new();
            for (idx, (lbl, _)) in self.overlay_lens.iter().enumerate() {
                if idx > 0 {
                    s.push_str(super::strip_gap());
                }
                let a = s.len();
                s.push_str(lbl);
                ranges.push((idx, a..s.len()));
            }
            for g in run.glyphs.iter() {
                if want >= g.x && want < g.x + g.w {
                    let b = g.start + 1;
                    for (idx, r) in ranges.iter() {
                        if b >= r.start && b < r.end {
                            hit = Some(*idx);
                        }
                    }
                }
            }
        }
        hit
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn overlay_shape_theme(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
        trailing: &[String],
        elide: bool,
    ) -> bool {
        let mut strip_s = String::from("\n");
        let mut label_ranges: Vec<(std::ops::Range<usize>, bool)> = Vec::new();
        let mut sep_ranges: Vec<std::ops::Range<usize>> = Vec::new();
        let mut active_range: Option<std::ops::Range<usize>> = None;
        for (idx, (lbl, active)) in geom.strip.iter().enumerate() {
            if idx > 0 {
                let s = strip_s.len();
                strip_s.push_str(super::strip_gap());
                sep_ranges.push(s..strip_s.len());
            }
            let s = strip_s.len();
            strip_s.push_str(lbl);
            let r = s..strip_s.len();
            if *active {
                active_range = Some(r.clone());
            }
            label_ranges.push((r, *active));
        }
        let active_ink = match crate::render::effective_facet_style() {
            theme::FacetStyle::Chips(theme::ChipVariant::FilledActive) => {
                theme::base_300().to_glyphon()
            }
            _ => ink,
        };
        self.shape_theme_spans(
            geom,
            plan,
            ink,
            active_ink,
            muted,
            selected_ink,
            vis,
            &strip_s,
            &label_ranges,
            &sep_ranges,
            trailing,
            1.0,
            elide,
        );
        let strip_w = self.theme_strip_px();
        if strip_w > geom.text_w {
            let scale = (geom.text_w / strip_w).max(0.5);
            self.shape_theme_spans(
                geom,
                plan,
                ink,
                active_ink,
                muted,
                selected_ink,
                vis,
                &strip_s,
                &label_ranges,
                &sep_ranges,
                trailing,
                scale,
                elide,
            );
        }
        // Record the active-lens mark from the shaped strip glyphs (line 1). Line-1
        // glyphs are byte-indexed WITHIN the strip line's own text — the leading "\n" in
        // `strip_s` split the lines — so a label's line-relative range is its `strip_s`
        // range shifted back by that one "\n" byte. The MARK'S SHAPE is the
        // PER-ITEM LIST SURFACES round's `facet_style`:
        //   - `Text`   (default, byte-identical) — a hairline UNDERLINE under the
        //     active label.
        //   - `Band`   — a rounded value PILL behind the active label (the killed
        //     `Chips` skin's ghost-pill-per-label was dropped; only the active
        //     mark draws).
        // The x-spans come from the SAME shaped glyphs the strip hit-test reads, so
        // the skin can never disagree with where a label is clicked.
        // Scan line 1 for a strip-range's glyph x-span (min_x, max_x) + the shaped
        // baseline (C2 y-owner), `None` if empty.
        let span_of = |buf: &GlyphBuffer, r: &std::ops::Range<usize>| -> Option<(f32, f32, f32)> {
            let (a, b) = (r.start.saturating_sub(1), r.end.saturating_sub(1));
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            // Y-OWNER FIX (COMPOSITION-C2): the underline y must be read from the
            // strip run's SHAPED BASELINE, never a fixed `2*lh` formula. The strip
            // row is inflated by `header_gap` (a taller line box), and the label
            // may shape at a display/mono CHROME face whose baseline sits high in
            // that box — so `text_top + 2*lh - 3` landed MID-GLYPH (the underline
            // struck through "File" on Tawny/Firetail). `run.line_y` is the real
            // baseline in buffer space (same `geom.text_top + run.line_*` mapping
            // the primary/secondary columns use); the underline sits a hair BELOW
            // it for every face. The strip's responsive fold reshapes into the
            // same `panel_buffer`, so this reads the FINAL (possibly scaled) run.
            let mut baseline = f32::MIN;
            for run in buf.layout_runs() {
                if run.line_i != 1 {
                    continue;
                }
                baseline = baseline.max(run.line_y);
                for g in run.glyphs.iter() {
                    if g.start >= a && g.start < b {
                        min_x = min_x.min(g.x);
                        max_x = max_x.max(g.x + g.w);
                    }
                }
            }
            (max_x > min_x && baseline > f32::MIN).then_some((min_x, max_x, baseline))
        };
        let facet_style = crate::render::effective_facet_style();
        let scale = self.metrics.scale;
        const CHIP_HPAD: Logical = Logical(6.0);
        const CHIP_VPAD: Logical = Logical(2.0);
        let chip_hpad = self.metrics.px(CHIP_HPAD);
        let underline_drop = self.metrics.px(UNDERLINE_BASELINE_DROP);
        let strip_text_lh = self.metrics.line_height * crate::render::effective_overlay_scale();
        let chip_h = (strip_text_lh - 2.0 * self.metrics.px(CHIP_VPAD)).max(1.0);
        // PLATE FLOOR: `strip.center()` treats the strip's whole
        // folded header-line box as free room, but a `Split` composition's own
        // visible seam falls INSIDE that same box — `BREATHE_FRAC` +
        // `SPLIT_GAP_FRAC` leave the query beat's own plate starting only at
        // `split_bounds().1`, not the box's geometric top (the fold is what
        // puts the beat there at all). A pill/tick centred on the box then
        // draws above the plate it is meant to sit inside: the filled chip's
        // own plate running flush into the strip band's top. Only
        // `ListStyle::Pane` ever draws that plate at all — mirroring the one
        // gate `overlay_prepare_card_backing` itself reads before ever calling
        // `overlay_pane_fills` — and only `PaneSplit::Split` carves a seam out
        // of it; every other composition floors at the box's own top, which
        // leaves `s.center()` untouched: BYTE-IDENTICAL off either gate (every
        // `Bars`/`Diagonal`/`Rules` world, and Cassowary's `Unified` Bars).
        let mark_cy = self
            .docked_facet_band(geom, plan)
            .or_else(|| plan.strip_band())
            .map_or(geom.text_top, |s| {
                let plate_top = if crate::render::effective_list_style().list_backing(false)
                    == theme::ListBacking::Card
                    && matches!(
                        crate::render::effective_pane_split(),
                        theme::PaneSplit::Split
                    ) {
                    plan.split_bounds()
                        .map_or(s.top, |(_, gap_bottom)| gap_bottom.max(s.top))
                } else {
                    s.top
                };
                s.center().max(plate_top + chip_h * 0.5)
            });
        let pill_px = |left: f32, right: f32| -> [f32; 4] {
            [
                geom.text_left + left,
                mark_cy - chip_h * 0.5,
                (right - left).max(1.0),
                chip_h,
            ]
        };
        let corner_ticks = |l: f32, r: f32| -> Vec<[f32; 4]> {
            const TICK_L: Logical = Logical(6.0); // arm length
            const TH_L: Logical = Logical(1.6); // arm thickness
            let (tick, th) = (TICK_L.px(scale), TH_L.px(scale));
            let top = mark_cy - chip_h * 0.5;
            let bot = mark_cy + chip_h * 0.5;
            let x0 = geom.text_left + l;
            let x1 = geom.text_left + r;
            vec![
                [x0, top, tick, th],
                [x0, top, th, tick], // TL
                [x1 - tick, top, tick, th],
                [x1 - th, top, th, tick], // TR
                [x0, bot - th, tick, th],
                [x0, bot - tick, th, tick], // BL
                [x1 - tick, bot - th, tick, th],
                [x1 - th, bot - tick, th, tick], // BR
            ]
        };
        // The active mark rect (single-rect skins) + the ghost/tick collection. THE
        // ONE owner: every skin below reads the SAME shaped glyph spans the strip
        // hit-test does, so the mark can never disagree with where a label is clicked.
        let mut ghosts: Vec<[f32; 4]> = Vec::new();
        let inactive_pills = || -> Vec<[f32; 4]> {
            let mut v = Vec::new();
            for (r, active) in &label_ranges {
                if *active {
                    continue;
                }
                if let Some((min_x, max_x, _)) = span_of(&self.panel_buffer, r) {
                    v.push(pill_px(min_x - chip_hpad, max_x + chip_hpad));
                }
            }
            v
        };
        self.overlay_theme_underline = active_range.as_ref().and_then(|ar| {
            let (min_x, max_x, baseline) = span_of(&self.panel_buffer, ar)?;
            match facet_style {
                theme::FacetStyle::Text => {
                    let y = geom.text_top + baseline + underline_drop;
                    Some([
                        geom.text_left + min_x,
                        y,
                        max_x - min_x,
                        self.metrics.px(TEXT_MARK_THICKNESS),
                    ])
                }
                theme::FacetStyle::Band => Some(pill_px(min_x - chip_hpad, max_x + chip_hpad)),
                theme::FacetStyle::DockedTab => Some([
                    geom.text_left + min_x - chip_hpad,
                    geom.card_y - chip_h,
                    max_x - min_x + 2.0 * chip_hpad,
                    chip_h,
                ]),
                theme::FacetStyle::Chips(v) => match v {
                    theme::ChipVariant::Hairline | theme::ChipVariant::FilledActive => {
                        if matches!(v, theme::ChipVariant::Hairline) {
                            ghosts = inactive_pills();
                        }
                        Some(pill_px(min_x - chip_hpad, max_x + chip_hpad))
                    }
                    theme::ChipVariant::Underline => {
                        let y = geom.text_top + baseline + underline_drop;
                        Some([
                            geom.text_left + min_x,
                            y,
                            max_x - min_x,
                            self.metrics.px(UNDERLINE_CHIP_THICKNESS),
                        ])
                    }
                    theme::ChipVariant::Bracket => {
                        ghosts = corner_ticks(min_x - chip_hpad, max_x + chip_hpad);
                        None
                    }
                },
            }
        });
        if matches!(facet_style, theme::FacetStyle::DockedTab)
            && let Some(tab) = self.overlay_theme_underline
        {
            ghosts.push(tab);
        }
        self.overlay_theme_facet_ghosts = ghosts;
        // A tab PILL is a plate, so `Rules` is deliberately absent — it draws
        // none anywhere. (`Diagonal` is on the yes side here and the no side of
        // `draws_row_plates`: it computes pills nothing consumes.)
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars | theme::ListStyle::Diagonal(_)
        );
        self.overlay_strip_tab_plates = if bars {
            label_ranges
                .iter()
                .filter_map(|(r, _active)| {
                    span_of(&self.panel_buffer, r)
                        .map(|(min_x, max_x, _)| pill_px(min_x - chip_hpad, max_x + chip_hpad))
                })
                .collect()
        } else {
            Vec::new()
        };
        false
    }
    #[allow(clippy::too_many_arguments)]
    fn shape_theme_spans(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        active_ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
        strip_s: &str,
        label_ranges: &[(std::ops::Range<usize>, bool)],
        sep_ranges: &[std::ops::Range<usize>],
        trailing: &[String],
        strip_scale: f32,
        elide: bool,
    ) {
        let fitted_hint = self.overlay_fitted_hint(geom);
        // Per-line font sizes ride the overlay UI base (`OVERLAY_UI_SCALE`), and their
        // LINE HEIGHTS stay the uniform UI row height (`overlay_lh`) so the plan line
        // offsets, the selected band, and the underline `y` never drift from a per-span
        // metric taller than the row.
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let sigil = "› ";
        let slant = crate::render::overlay_slant();
        let slant_tax = slant
            .map(|s| crate::render::slant_max_offset(&s, geom.plan.len()))
            .unwrap_or(0.0);
        let char_w = self.overlay_char_width();
        let total_chars = if char_w > 0.0 {
            (((geom.text_w - slant_tax).max(0.0)) / char_w).floor() as usize
        } else {
            usize::MAX
        };
        let row_budget = rowlayout::full_budget(total_chars);
        let fitted: Vec<Option<String>> = geom
            .plan
            .iter()
            .map(|line| match line {
                PlanLine::Location(_) | PlanLine::Header(_) => None,
                PlanLine::Item(i) => {
                    let name = self.overlay_items.get(*i).map(|s| s.as_str()).unwrap_or("");
                    Some(if elide {
                        rowlayout::fit_primary(name, row_budget).to_string()
                    } else {
                        name.to_string()
                    })
                }
            })
            .collect();

        let title_prefix = self.overlay_title_prefix(geom);
        let mut spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        if title_prefix.is_empty() {
            spans.push((sigil, mk(muted)));
        } else {
            spans.push((title_prefix.as_str(), chrome_attrs().color(muted)));
        }
        spans.push((self.overlay_query.as_str(), mk(ink)));
        // Strip line: active label in full ink, others muted, separators + the "\n"
        // faint. One ordered pass over `strip_s` so the spans tile the line in byte
        // order (rich-text concatenates spans in push order). The label/separator
        // spans carry the strip font size at the `strip_lh` (= `lh + header_gap`)
        // row height; the leading "\n" keeps the buffer's UI font size so the strip
        // row's font stays scale-invariant.
        self.push_theme_strip_spans(
            &mut spans,
            plan,
            spans::ThemeStripSpec {
                text: strip_s,
                labels: label_ranges,
                separators: sep_ranges,
                scale: strip_scale,
            },
            active_ink,
            muted,
        );
        self.push_theme_plan_spans(
            &mut spans,
            geom,
            &fitted,
            trailing,
            OverlaySpanInks {
                ink,
                muted,
                selected: selected_ink,
            },
            vis,
        );
        if let Some(msg) = &geom.empty {
            spans.push(("\n", mk(muted)));
            spans.push((msg.as_str(), mk(muted)));
        }
        if geom.hint_rows > 0 {
            self.push_overlay_hint_spans(
                &mut spans,
                fitted_hint.as_str(),
                muted,
                geom.hint_gap_rows,
                geom.header_rows > 0 || !geom.plan.is_empty() || geom.empty.is_some(),
            );
        }
        self.panel_buffer
            .set_size(&mut self.font_system, Some(geom.text_w), Some(geom.card_h));
        self.panel_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let default_attrs = base.clone().color(ink);
        self.panel_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }

    fn theme_strip_px(&self) -> f32 {
        let mut w = 0.0f32;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i == 1 {
                w = w.max(run.line_w);
            }
        }
        w
    }
}
