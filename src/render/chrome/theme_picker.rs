use super::overlay_clamp::window_plan;
use super::*;

/// Pixels the active-lens UNDERLINE sits BELOW the strip run's shaped baseline
/// (`overlay_shape_theme`). Small so the rule hugs the label — enough to clear
/// the baseline for every chrome/mono/display face without striking the glyphs.
const UNDERLINE_BASELINE_DROP: f32 = 2.0;

impl TextPipeline {
    /// THEME PICKER display plan: the candidate-area sequence of section HEADERS +
    /// world ROWS, from the parallel `overlay_sections`. A header is emitted before a
    /// row whenever its section differs from the previous row's (so contiguous groups
    /// get one header each); the All lens / non-grouped rows emit no headers. Section
    /// labels are uppercased for the faint header display. Shared by the geometry,
    /// shaping, selected-band, and hit-test so they can never disagree.
    pub(in crate::render) fn theme_plan(&self) -> Vec<PlanLine> {
        let mut out = Vec::with_capacity(self.overlay_items.len());
        let mut prev: Option<String> = None;
        for i in 0..self.overlay_items.len() {
            let sect = self
                .overlay_sections
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !sect.is_empty() && prev.as_deref() != Some(sect) {
                out.push(PlanLine::Header(sect.to_uppercase()));
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
        let pad = 12.0;
        let margin = 12.0;
        let n_items = self.overlay_items.len();
        let full_plan = self.theme_plan();
        let hint = self.overlay_hint.clone();
        let hint_rows = if hint.is_empty() { 0 } else { 1 };
        let (footer, footer_rows) = self.overlay_footer_lines();
        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;
        let header_rows = 2;
        let header_gap = self.overlay_header_gap();
        let card_y = margin + 40.0 + self.menubar_reserve();
        let total_headers = full_plan.len() - n_items;
        // ITEM 184 — strip + headers + footer count here; `min_items: 0`
        // empties the band rather than overrun it (`fit_item_rows`'s doc).
        let chrome_rows = header_rows + hint_rows + empty_rows + footer_rows;
        // ITEM 181 — THE ONE HEIGHT-CLAMP OWNER, shared with the flat family.
        let avail_px = (self.window_h - card_y - margin - 2.0 * pad - header_gap).max(lh);
        let item_cap = self.overlay_item_cap(avail_px, lh, chrome_rows + total_headers, 0);
        let (item_top, item_visible) = scroll_window(
            n_items,
            self.overlay_selected,
            self.overlay_scroll,
            item_cap,
        );
        let plan = window_plan(&full_plan, item_top, item_top + item_visible);
        let total_rows = header_rows + plan.len() + empty_rows + hint_rows + footer_rows;
        // Wider than the flat pickers so the whole lens strip (Time … All) fits on
        // one line even on a WIDE mono world face without the far-right All clipping
        // — via the SAME horizontal-box owner (edge inset + narrow-window fallback),
        // just a slightly wider cap ([`CARD_MAX_W_FACETED`]). Scaled to the current
        // zoom/DPI by the ONE owner [`TextPipeline::overlay_card_desired_w`], so the
        // faceted card (the Cmd-P palette) grows WITH the glyphs like the flat one —
        // otherwise the 600 cap stayed unzoomed while the text doubled and every row
        // elided (the zoom-blind card bug).
        // ITEM 51: content-hug for a RIGHT-ANCHORED faceted card (via the ONE
        // `overlay_desired_w` owner), the wide `CARD_MAX_W_FACETED` cap otherwise.
        let desired_w = self.overlay_desired_w(super::overlay::CARD_MAX_W_FACETED);
        let (card_x, card_w) = self.overlay_card_box(width, desired_w);
        let card_narrow = super::overlay::overlay_card_fill_regime(width as f32, desired_w);
        let hpad = self.overlay_text_hpad();
        let text_w = card_w - 2.0 * hpad;
        let card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, pad);
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
            card_narrow,
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
        let (strip_top, strip_lh) = self.overlay_strip_band(&geom);
        if py < strip_top || py >= strip_top + strip_lh {
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
                if idx == 0 {
                    continue; // the All home is not a drawn label
                }
                if idx > 1 {
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
            if idx == 0 {
                continue; // the All home is the flat corpus, not a drawn label
            }
            if idx > 1 {
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
        const CHIP_HPAD: f32 = 6.0;
        const CHIP_VPAD: f32 = 2.0;
        let (strip_top, strip_lh) = self.overlay_strip_band(geom);
        let mark_cy = strip_top + strip_lh * 0.5;
        let strip_text_lh = self.metrics.line_height * crate::render::effective_overlay_scale();
        let chip_h = (strip_text_lh - 2.0 * CHIP_VPAD).max(1.0);
        let pill_px = |left: f32, right: f32| -> [f32; 4] {
            [
                geom.text_left + left,
                mark_cy - chip_h * 0.5,
                (right - left).max(1.0),
                chip_h,
            ]
        };
        let corner_ticks = |l: f32, r: f32| -> Vec<[f32; 4]> {
            const TICK: f32 = 6.0; // arm length
            const TH: f32 = 1.6; // arm thickness
            let top = mark_cy - chip_h * 0.5;
            let bot = mark_cy + chip_h * 0.5;
            let x0 = geom.text_left + l;
            let x1 = geom.text_left + r;
            vec![
                [x0, top, TICK, TH],
                [x0, top, TH, TICK], // TL
                [x1 - TICK, top, TICK, TH],
                [x1 - TH, top, TH, TICK], // TR
                [x0, bot - TH, TICK, TH],
                [x0, bot - TICK, TH, TICK], // BL
                [x1 - TICK, bot - TH, TICK, TH],
                [x1 - TH, bot - TICK, TH, TICK], // BR
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
                    v.push(pill_px(min_x - CHIP_HPAD, max_x + CHIP_HPAD));
                }
            }
            v
        };
        self.overlay_theme_underline = active_range.as_ref().and_then(|ar| {
            let (min_x, max_x, baseline) = span_of(&self.panel_buffer, ar)?;
            match facet_style {
                theme::FacetStyle::Text => {
                    let y = geom.text_top + baseline + UNDERLINE_BASELINE_DROP;
                    Some([geom.text_left + min_x, y, max_x - min_x, 1.5])
                }
                theme::FacetStyle::Band => Some(pill_px(min_x - CHIP_HPAD, max_x + CHIP_HPAD)),
                theme::FacetStyle::Chips(v) => match v {
                    theme::ChipVariant::Hairline | theme::ChipVariant::FilledActive => {
                        if matches!(v, theme::ChipVariant::Hairline) {
                            ghosts = inactive_pills();
                        }
                        Some(pill_px(min_x - CHIP_HPAD, max_x + CHIP_HPAD))
                    }
                    theme::ChipVariant::Underline => {
                        let y = geom.text_top + baseline + UNDERLINE_BASELINE_DROP;
                        Some([geom.text_left + min_x, y, max_x - min_x, 3.5])
                    }
                    theme::ChipVariant::Bracket => {
                        ghosts = corner_ticks(min_x - CHIP_HPAD, max_x + CHIP_HPAD);
                        None
                    }
                },
            }
        });
        self.overlay_theme_facet_ghosts = ghosts;
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars { .. }
        );
        self.overlay_strip_tab_plates = if bars {
            label_ranges
                .iter()
                .filter_map(|(r, _active)| {
                    span_of(&self.panel_buffer, r)
                        .map(|(min_x, max_x, _)| pill_px(min_x - CHIP_HPAD, max_x + CHIP_HPAD))
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
        let m = self.metrics;
        let faint = theme::faint().to_glyphon();
        let label = crate::markdown::type_scale::LABEL;
        // Per-line font sizes ride the overlay UI base (`OVERLAY_UI_SCALE`), and their
        // LINE HEIGHTS stay the uniform UI row height (`overlay_lh`) so the plan line
        // offsets, the selected band, and the underline `y` never drift from a per-span
        // metric taller than the row.
        let ui = crate::render::effective_overlay_scale();
        let lh = self.overlay_lh();
        let header_metrics = GlyphMetrics::new(m.font_size * ui * label, lh);
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
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
                PlanLine::Header(_) => None,
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
        {
            let mut cursor = 0usize;
            let mut pushes: Vec<(std::ops::Range<usize>, glyphon::Color)> = Vec::new();
            for (r, active) in label_ranges {
                pushes.push((r.clone(), if *active { active_ink } else { muted }));
            }
            for r in sep_ranges {
                pushes.push((r.clone(), faint));
            }
            pushes.sort_by_key(|(r, _)| r.start);
            // The strip row's HEIGHT is inflated by `header_gap` (PALETTE-COMPOSITION
            // round) so the calm divider space falls after the lens strip, before the
            // section-grouped rows — uniform with the flat pickers' query-line gap.
            // The plan-line offsets, selected band, and underline all fold the same
            // gap in through `overlay_row_top`, so nothing below the strip drifts.
            //
            // The gap MUST ride the strip line's REAL LABEL glyphs, NOT its leading
            // "\n": cosmic-text sizes a line from the glyphs ON it, and the "\n" is a
            // BREAK that terminates the PRIOR (query) line — its own metrics never
            // grow the strip line, so inflating only the "\n" moved the selected BAND
            // (which reads `header_gap` off `overlay_row_top`) down a half-row while
            // the TEXT stayed put. That half-row band/text drift was invisible under
            // a gentle value band but clipped the top of the selected row's own
            // glyphs once a 1-bit world drew them as solid black on a white band
            // (the Wagtail selected-row bug's second half). `strip_lh` on the labels
            // makes text and band agree; the "\n" keeps the row's scale-invariant
            // baseline size.
            let strip_lh = self.overlay_strip_band(geom).1;
            spans.push((
                &strip_s[0..1],
                mk(faint).metrics(GlyphMetrics::new(m.font_size * ui, lh)),
            ));
            cursor += 1;
            for (r, c) in pushes {
                debug_assert_eq!(r.start, cursor, "strip spans must tile the line");
                cursor = r.end;
                let fs = if strip_scale < 1.0 {
                    m.font_size * ui * strip_scale
                } else {
                    m.font_size * ui
                };
                spans.push((
                    &strip_s[r],
                    chrome_attrs()
                        .color(c)
                        .metrics(GlyphMetrics::new(fs, strip_lh)),
                ));
            }
        }
        let slant_italic = slant.map(|s| s.italic).unwrap_or(false);
        let rk = |c| {
            if slant_italic {
                mk(c).style(glyphon::cosmic_text::Style::Italic)
            } else {
                mk(c)
            }
        };
        for (idx, (line, fit)) in geom.plan.iter().zip(fitted.iter()).enumerate() {
            spans.push(("\n", mk(ink)));
            match line {
                PlanLine::Header(h) => {
                    spans.push((h.as_str(), mk(faint).metrics(header_metrics)));
                }
                PlanLine::Item(_) => {
                    let flip = vis.reads_selected(idx);
                    let c = match selected_ink {
                        Some(c) if flip => c,
                        _ => ink,
                    };
                    spans.push((fit.as_deref().unwrap_or(""), rk(c)));
                    if let Some(t) = trailing.get(idx).filter(|t| !t.is_empty()) {
                        push_symbol_split(&mut spans, t, || mk(muted), || sym(muted));
                    }
                }
            }
        }
        if let Some(msg) = &geom.empty {
            spans.push(("\n", mk(muted)));
            spans.push((msg.as_str(), mk(muted)));
        }
        if geom.hint_rows > 0 {
            self.push_overlay_hint_spans(&mut spans, geom.hint.as_str(), muted);
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
