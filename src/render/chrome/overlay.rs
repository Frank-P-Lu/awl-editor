use super::*;

impl TextPipeline {
    pub(in crate::render) fn overlay_metrics(&self) -> GlyphMetrics {
        let m = self.metrics;
        let scale = crate::render::effective_overlay_scale();
        GlyphMetrics::new(m.font_size * scale, self.overlay_lh())
    }

    /// PER-ITEM LIST SURFACES round — the vertical GAP (device px) opened
    /// between candidate rows under [`theme::ListStyle::Bars`]; `0.0` under
    /// `Pane` (byte-identical). It is folded into the ONE overlay row-pitch
    /// owner [`Self::overlay_lh`] (and thus into `overlay_metrics`), so the card
    /// height, the shaped text spread, the selected band, and the pointer
    /// hit-test all widen the row pitch TOGETHER — bars and text can never
    /// disagree about a row's y (round A's y-agreement law holds by
    /// construction). The bar surfaces then draw `lh - gap` tall, leaving the
    /// gap as the space between them.
    pub(in crate::render) fn overlay_row_gap(&self) -> f32 {
        let gap = match crate::render::effective_list_style() {
            theme::ListStyle::Bars => Logical(crate::render::effective_bar_config().gap.max(0.0)),
            // The same quantity one style down: a `Ruled` list separates its
            // rows with a rule and the AIR either side of it, so the air rides
            // the one row-pitch owner exactly as a plate gap does — text, rules,
            // card height and hit-test widen together and cannot disagree.
            theme::ListStyle::Ruled(_) => RULE_ROW_AIR,
            theme::ListStyle::Pane | theme::ListStyle::Diagonal(_) => Logical(0.0),
        };
        self.metrics.px(gap)
    }

    /// The overlay's EXTRA leading, resolved. A theme-authored length like any
    /// other: it is summed with a dpi-scaled line height inside
    /// [`Self::overlay_lh`], so leaving it raw made the one quantity the tree
    /// treats as logical drift out of proportion across displays.
    pub(in crate::render) fn overlay_leading(&self) -> f32 {
        self.metrics
            .px(Logical(crate::render::effective_overlay_leading()))
    }

    /// PER-ITEM LIST SURFACES round — the horizontal inset (device px) the row
    /// TEXT column holds from the layout bound (`card_x` .. `card_x + card_w`).
    /// `Pane` keeps the historical `12` pad (byte-identical). `Bars` insets
    /// `BAR_SIDE_INSET + BAR_TEXT_PAD` so the glyphs sit a comfortable pad INSIDE
    /// each bar's edge (the user's "bar text needs real left padding" refit),
    /// symmetric so the secondary chord column mirrors it inside the bar's right
    /// edge. The ONE owner both `overlay_geometry` and `theme_overlay_geometry`
    /// read for `text_left`/`text_w`, so shaping, hit-test, caret, and the
    /// right-aligned chords all inset together.
    pub(in crate::render) fn overlay_text_hpad(&self) -> f32 {
        let l = match crate::render::effective_list_style() {
            theme::ListStyle::Bars => {
                return self.metrics.px(BAR_SIDE_INSET) + self.metrics.px(BAR_TEXT_PAD);
            }
            // The inset a `Ruled` list holds is its GUTTER — the column its
            // selection mark hangs in and the margin its heavy rule runs out
            // into. Symmetric, so the secondary column mirrors it.
            theme::ListStyle::Ruled(_) => RULES_TEXT_HPAD,
            theme::ListStyle::Pane | theme::ListStyle::Diagonal(_) => PANE_TEXT_HPAD,
        };
        self.metrics.px(l)
    }

    /// The overlay row LINE HEIGHT — the single-owner metric the card height, the
    /// row-Y, the hit-test, and the
    /// selected-row band all read, so a click always lands on the row it highlights.
    pub(in crate::render) fn overlay_lh(&self) -> f32 {
        self.metrics.line_height * crate::render::effective_overlay_scale()
            + self.overlay_leading()
            + self.overlay_row_gap()
    }

    pub(in crate::render) fn overlay_char_width(&self) -> f32 {
        self.metrics.char_width * crate::render::effective_overlay_scale()
    }

    pub(in crate::render) fn overlay_card_box(&self, width: u32, desired_w: f32) -> (f32, f32) {
        overlay_card_box_policy(
            crate::render::resolve_overlay_anchor(self.overlay_align),
            width as f32,
            desired_w,
            self.metrics.scale,
            self.metrics.dpi,
        )
    }

    /// THE ONE OWNER of the summoned card's WIDE desired width (device px) at the
    /// CURRENT zoom/DPI: the base cap ([`CARD_MAX_W`] / [`CARD_MAX_W_FACETED`],
    /// tuned for the 1:1 capture canvas) GROWN by [`Metrics::scale`] so
    /// the card widens WITH the glyphs.
    ///
    /// Without this the cap stayed an unzoomed 520/600 while the overlay text
    /// DOUBLED under zoom — the card read proportionally half as wide, and
    /// [`rowlayout`]'s primary-cell elision + the footer's yield fired even though
    /// the WINDOW had abundant room (the zoom-blind card bug: at 200% every
    /// palette row came back "Go t…ile…", "Comp…ion…"). The window clamp in
    /// [`overlay_card_box_policy`] still bounds the result, so a card never
    /// overruns the window — it just stops eliding when there IS room, and enters
    /// the fill regime only when the window GENUINELY lacks it. Both geometry
    /// paths ([`Self::overlay_geometry`], [`TextPipeline::theme_overlay_geometry`])
    /// and the fill-regime fold read this ONE scaled width, so the width and the
    /// fold threshold can never drift.
    ///
    /// GROW-ONLY (`scale.max(dpi)`, i.e. `dpi · zoom.max(1.0)`): ZOOM only ever
    /// WIDENS the base cap, and the DENSITY always carries it, so the resolved
    /// cap is one LOGICAL width on every panel. The bug it fixes is a high-zoom
    /// COLLAPSE, so it touches exactly the `zoom > 1.0` regime; at the authored
    /// default (zoom 1.0) and every zoom ≤ 1.0 the card holds the base cap at
    /// whatever the display's ratio is (a slightly-roomier-than-proportional card
    /// at low zoom, which never clips).
    ///
    /// The floor is stated against `dpi` rather than a bare `1.0` because the two
    /// differ on a below-default retina stress cell: at `zoom 0.8,
    /// dpi 2` a `1.0` floor never binds, so the card resolved to `545 · 1.6` =
    /// 872 device px — 436 LOGICAL px against the 545 a 1× reader sees at the
    /// same logical window. The elision budget, and therefore how much of a
    /// command name a row can show, was a property of the reader's display.
    pub(in crate::render) fn overlay_card_desired_w(&self, base: LogicalGrowOnly) -> f32 {
        self.metrics.px_grow_only(base)
    }

    pub(in crate::render) fn overlay_right_anchored(&self) -> bool {
        crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth()
    }

    pub(in crate::render) fn overlay_desired_w(&self, base_cap: LogicalGrowOnly) -> f32 {
        let scaled = self.overlay_card_desired_w(base_cap);
        if self.overlay_right_anchored() && self.overlay_content_w > 0.0 {
            let floor = self.metrics.px_grow_only(CARD_CONTENT_MIN_W).min(scaled);
            self.overlay_content_w.clamp(floor, scaled)
        } else {
            scaled
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_elided_candidates(&self, width: u32) -> Vec<String> {
        let geom = self.overlay_geometry(width);
        let cw = self.overlay_char_width();
        let total_chars = if cw > 0.0 {
            (geom.text_w / cw).floor() as usize
        } else {
            usize::MAX
        };
        let budget = rowlayout::full_budget(total_chars);
        self.overlay_items
            .iter()
            .filter(|item| rowlayout::fit_primary(item.as_str(), budget).as_str() != item.as_str())
            .cloned()
            .collect()
    }

    /// THE QUERY-INPUT BEAT token (device px): the calm slab of negative space
    /// inserted after the header rows (query + optional lens strip) and before
    /// the candidate list, on the palette AND every faceted picker uniformly (the
    /// divider is negative space, never a drawn rule). Sized off the overlay row
    /// height so it scales with zoom/DPI like every other overlay metric.
    ///
    /// [`OVERLAY_QUERY_BEAT`] sets it above ~0.55
    /// of a row — a clearer beat between the input line and the first result,
    /// still short of the "fat lip" of a whole blank row. It is STRUCTURAL, not a
    /// leading newline (the f2cb656 tripwire): the shaper inflates the last
    /// header line's REAL glyph metrics by exactly this, and the band, primary
    /// name, secondary chord, hit-test, and caret all fold it in through the ONE
    /// y-owner, the scene planner — so text
    /// and band move together, never a half-row split. Both geometry owners read
    /// this; the contextual spell popup passes `0.0` (no header to divide from).
    /// LIVE-ONLY taste: whether the widened beat reads right needs a human eye.
    pub(in crate::render) fn overlay_header_gap(&self) -> f32 {
        (self.overlay_lh() * OVERLAY_QUERY_BEAT.0).round()
    }

    pub(in crate::render) fn overlay_hint_h(&self) -> f32 {
        (self.overlay_lh() * OVERLAY_HINT_ROW.0).round()
    }

    /// The blank separator's own (shorter still) row height.
    pub(in crate::render) fn overlay_hint_gap_h(&self) -> f32 {
        (self.overlay_lh() * OVERLAY_HINT_GAP_ROW.0).round()
    }

    /// Reclaims the dead space `hint_rows` (`overlay_hint_h`-tall) and
    /// `gap_rows` (`overlay_hint_gap_h`-tall) COMPACT rows leave behind in a
    /// row budget that allocated each of them a full `overlay_lh` slot — the
    /// hint's own row and the blank separator ahead of it, each
    /// reclaimed at its OWN compact height rather than one borrowing the
    /// other's. ONE trailing [`OVERLAY_FOOTER_PAD`] survives regardless of how
    /// many compact rows there are, never one per row: the pad is the
    /// breathing room below the LAST compact row, not a tax per row, so the
    /// gap row's own reclaim can't eat into the chin below the hint.
    pub(in crate::render) fn overlay_footer_reclaim(
        &self,
        hint_rows: usize,
        gap_rows: usize,
    ) -> f32 {
        if hint_rows == 0 && gap_rows == 0 {
            return 0.0;
        }
        let pad = self.metrics.px(OVERLAY_FOOTER_PAD);
        let hint_slack = hint_rows as f32 * (self.overlay_lh() - self.overlay_hint_h()).max(0.0);
        let gap_slack = gap_rows as f32 * (self.overlay_lh() - self.overlay_hint_gap_h()).max(0.0);
        (hint_slack + gap_slack - pad).max(0.0)
    }

    pub(in crate::render) fn overlay_card_h(
        &self,
        total_rows: usize,
        header_gap: f32,
        hint_rows: usize,
        gap_rows: usize,
        pad: f32,
    ) -> f32 {
        total_rows as f32 * self.overlay_lh() + header_gap + 2.0 * pad
            - self.overlay_footer_reclaim(hint_rows, gap_rows)
    }

    pub(in crate::render) fn overlay_right_labels(&self) -> &[String] {
        if !self.overlay_bindings.is_empty() {
            &self.overlay_bindings
        } else if !self.overlay_times.is_empty() {
            &self.overlay_times
        } else {
            &self.overlay_git
        }
    }

    #[cfg(test)]
    pub(in crate::render) fn overlay_geom_is_faceted(&self, geom: &OverlayGeom) -> bool {
        geom.theme
    }

    pub(in crate::render) fn overlay_geometry(&self, width: u32) -> OverlayGeom {
        if let Some((line, start_col, end_col)) = self.overlay_spell {
            return self.spell_overlay_geometry(width, line, start_col, end_col);
        }
        // The THIRD family. Checked before the faceted one because a
        // workspace's rail IS its facet strip, stood on its end: the same data
        // reaches a different presentation, and there is no card to place.
        if self.overlay_is_workspace() {
            return self.workspace_geometry(width);
        }
        if !self.overlay_lens.is_empty() {
            return self.theme_overlay_geometry(width);
        }
        let pad = self.metrics.px(CARD_PAD);
        let margin = self.metrics.px(CARD_MARGIN);
        let n_items = self.overlay_items.len();

        let mut hint = self.overlay_hint.clone();
        let hint_rows = if hint.is_empty() { 0 } else { 1 };
        let mut hint_gap_rows = overlay_hint_gap_rows(hint_rows);

        let (footer, footer_rows) = self.overlay_footer_lines();

        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;

        let contextual = self.overlay_contextual();
        let header_rows = usize::from(!contextual); // contextual rows need no query field
        // PALETTE-COMPOSITION round: a calm gap after the query header, before the
        // candidate list (negative space as the divider). Grows the card by exactly
        // this and offsets the candidate band/hit-test through the planned rows.
        let header_gap = if contextual {
            0.0
        } else {
            self.overlay_header_gap()
        };
        let card_y = self
            .overlay_context_anchor
            .map(|(_, y)| y + self.metrics.px(CONTEXT_ANCHOR_DROP))
            .unwrap_or(margin + self.metrics.px(CARD_TOP_DROP) + self.menubar_reserve());
        // Cap the item window to what the canvas fits, same owner the
        // grouped family reads (`theme_overlay_geometry`).
        let avail_px = if contextual {
            (self.window_h - 2.0 * margin).max(self.overlay_lh())
        } else {
            (self.window_h - card_y - margin - 2.0 * pad - header_gap).max(self.overlay_lh())
        };
        let chrome_rows = header_rows + hint_gap_rows + hint_rows + empty_rows + footer_rows;
        let (top_idx, visible) = self.overlay_flat_window(n_items, avail_px, chrome_rows);
        let mut total_rows =
            header_rows + visible + empty_rows + hint_gap_rows + hint_rows + footer_rows;
        let desired_w = self.overlay_desired_w(CARD_MAX_W);
        let (mut card_x, card_w) = self.overlay_card_box(width, desired_w);
        if let Some((x, _)) = self.overlay_context_anchor {
            let floor = self.metrics.px(CARD_EDGE_INSET_FLOOR);
            card_x = x.clamp(floor, (width as f32 - card_w - floor).max(floor));
        }
        let card_narrow = overlay_card_fill_regime(width as f32, desired_w, self.metrics.scale);
        let hpad = self.overlay_text_hpad();
        let text_w = card_w - 2.0 * hpad;
        hint = hint_yielding_explanation(&hint, width as f32 / self.metrics.scale.max(0.01));
        let mut card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        // The gap is decorative breathing room, not load-bearing chrome: in the
        // starvation corner (a `min_items: 1` floor still
        // outgrows the canvas at an extreme zoom — the flat family's own
        // arm), drop it rather than push the card past the canvas the way a
        // sectioned header's own overhead already degrades at the grouped
        // family's `min_items: 0` floor. `total_rows`/`card_h` are the only
        // two callers see, so this can't drift from the struct below.
        if !contextual && card_y + card_h > self.window_h + 0.01 && hint_gap_rows > 0 {
            total_rows -= hint_gap_rows;
            hint_gap_rows = 0;
            card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        }
        let card_y = if contextual {
            card_y.clamp(margin, (self.window_h - card_h - margin).max(margin))
        } else {
            card_y
        } + self.overlay_entrance_offset();
        let text_left = card_x + hpad;
        let text_top = card_y + pad;
        OverlayGeom {
            visible,
            top_idx,
            n_items,
            hint,
            hint_rows,
            hint_gap_rows,
            footer,
            footer_rows,
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
            ..OverlayGeom::base()
        }
    }

    /// Geometry for the contextual SPELL panel: a small floating popup anchored just
    /// below the misspelled `(line, start_col, end_col)` word — no query line, no foot
    /// hint, just the suggestion rows. The card's LEFT edge aligns to the word start
    /// and its TOP hangs a hair below the word's screen rect (computed from the SAME
    /// advance-aware visual-row layout the squiggle under the word uses, so the panel
    /// tracks the word at any wrap / scroll / zoom). Clamped to stay on-canvas — it
    /// flips ABOVE the word when there is no room below.
    fn spell_overlay_geometry(
        &self,
        width: u32,
        line: usize,
        start_col: usize,
        end_col: usize,
    ) -> OverlayGeom {
        let m = self.metrics;
        let pad = self.metrics.px(SPELL_PAD);
        let margin = self.metrics.px(SPELL_MARGIN);
        let gap = self.metrics.px(SPELL_WORD_GAP);
        let n_items = self.overlay_items.len();
        let header_rows = 0;
        let hint = String::new();
        let hint_rows = 0;
        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };

        let (word_x, word_top, _word_w, word_h) = self.spell_word_rect(line, start_col, end_col);

        // The popup's own budget: whichever side (above/below the
        // word) is roomier.
        let below_avail = self.window_h - (word_top + word_h + gap) - margin;
        let above_avail = word_top - gap - margin;
        let avail_px = below_avail.max(above_avail).max(self.overlay_lh());
        let (top_idx, visible) =
            self.overlay_flat_window(n_items, avail_px, header_rows + hint_rows);

        // Width fits the WIDEST suggestion ROW: its SHAPED width measured into
        // `overlay_spell_w`, not the anchor word. A short misspelling therefore cannot
        // make a card too narrow for its longer corrections.
        //
        // Non-Diagonal compositions also measure the row in the elision's own units.
        // The shaped width is in the UI face, while row elision divides `text_w` by the
        // document mono `char_width` and charges the primary for the ellipsis cell and
        // the blank but present secondary column carried by each spell row.
        // A wide-mono world can therefore need more width than the UI-face measurement.
        // The grid floor covers the widest row plus the cells `rowlayout::plan` reserves:
        // one ellipsis, the secondary-column gap, and one rounding cell so integer
        // `floor(text_w / char_width)` clears the target when f32 division lands just
        // under a whole cell. That expression remains byte-identical for Pane, Bars and
        // Ruled and still permits a genuinely overlong row to elide at the card cap.
        //
        // Diagonal rows occupy a measured cluster beside their rake instead. Charging
        // both the document grid and the rake created the broad empty span, while the
        // phantom empty secondary column could still shorten the Add action. The shared
        // spell-popup policy owns that typed boundary, its composition reserve, and the
        // matching measured-fit decision used by shaping.
        // The caller supplies measurements only; style arithmetic does not remain here.
        // This keeps every spell geometry consumer on the same typed decision.
        let widest_chars = self
            .overlay_items
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0);
        let grid_slack = 1 + crate::render::rowlayout::GAP_CHARS + 1;
        let char_grid_w = (widest_chars + grid_slack) as f32 * m.char_width;
        let measured_w = if self.overlay_spell_w > 0.0 {
            self.overlay_spell_w
        } else {
            char_grid_w
        };
        let rows = header_rows + visible.max(1) + hint_rows;
        // The MIN/MAX bounds are tuned for the 1:1 capture canvas; GROW them with the
        // current zoom/DPI (the SAME grow-only `LogicalGrowOnly` the takeover
        // card's width uses) so a long correction isn't clamped to an unzoomed cap
        // while its shaped `content_w` doubled under zoom — the zoom-blind card bug,
        // contextual sibling. Grow-only (`scale.max(dpi)`): at every zoom ≤ 1.0 (including
        // the authored default) the band holds its authored LOGICAL width on any
        // panel, so the popup's clamp is not a function of the display. The MAX is
        // wide enough to hold a whole add-to-dictionary row for an ordinary word
        // (~24 mono cells) so it never elides at wide width; a genuinely
        // long adversarial word still overruns it and elides — a small popup, never a
        // takeover card.
        let framed_w = self.spell_framed_width(rows, measured_w, char_grid_w, pad);
        let card_w = framed_w
            .clamp(
                self.metrics.px_grow_only(SPELL_MIN_W),
                self.metrics.px_grow_only(SPELL_MAX_W),
            )
            .min(width as f32 - 2.0 * margin);
        let text_w = card_w - 2.0 * pad;
        let card_h = self.overlay_card_h(rows, 0.0, 0, 0, pad);

        let [card_x, card_y] = crate::render::plan::plan_spell_anchor(
            [width as f32, self.window_h],
            [word_x, word_top, _word_w, word_h],
            [card_w, card_h],
            margin,
            gap,
        );
        let text_left = card_x + pad;
        let text_top = card_y + pad;
        OverlayGeom {
            visible,
            top_idx,
            n_items,
            hint,
            hint_rows,
            hint_gap_rows: 0,
            header_rows,
            empty,
            card_x,
            card_y,
            card_w,
            card_h,
            text_left,
            text_top,
            text_w,
            ..OverlayGeom::base()
        }
    }

    fn spell_word_rect(
        &self,
        line: usize,
        start_col: usize,
        end_col: usize,
    ) -> (f32, f32, f32, f32) {
        let m = self.metrics;
        let doc_top = self.doc_top();
        let rows = self.visual_rows(line);
        let row = pick_row(&rows, start_col);
        let char_count = row.xs.len().saturating_sub(1);
        let s = start_col.min(char_count);
        let e = end_col.min(char_count).max(s);
        let (x, w) = row_x_span(row, self.text_left(), s, e, m.char_width);
        let top = doc_top + row.line_top;
        (x, top, w, row.line_height)
    }

    /// Hit-test a pointer at PHYSICAL `(px, py)` against the SUMMONED overlay's
    /// candidate ROWS, returning the `items` index of the row it lands on — the value
    /// to assign to `overlay_selected` / [`crate::overlay::OverlayState::selected`] — or
    /// `None` when the pointer is off the card, on the query line, on the foot hint, or
    /// below the last visible row. It reads the SAME [`Self::overlay_geometry`] the rows
    /// are rendered from, so a hovered/clicked row can NEVER disagree with the
    /// highlighted one. This is the ONE reusable mechanic behind mouse-selecting EVERY
    /// picker kind (go-to / command / browse / theme / keybindings / spell / caret /
    /// outline / project / move-dest) — the overlay intercept is kind-agnostic, so
    /// `input.rs` maps a pointer to a row here and then drives the same selection-move +
    /// accept the keyboard does.
    /// The summoned overlay card's rectangle `[x, y, w, h]` for this frame, or `None`
    /// when no overlay is open — the centered takeover card vs. the contextual SPELL
    /// panel anchored at the misspelled word — from the SAME [`Self::overlay_geometry`]
    /// the card renders from. Used by `input.rs` for the CLICK-AWAY hit-test (a left
    /// click OUTSIDE this rect dismisses the overlay) and by headless tests to assert
    /// WHERE the card sits.
    pub fn overlay_card_rect(&self) -> Option<[f32; 4]> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        Some([geom.card_x, geom.card_y, geom.card_w, geom.card_h])
    }

    /// The SUMMONED overlay's drawn scroll-WINDOW for the sidecar, or `None` when no
    /// overlay is open: `(top, lines, sel_row, card_h, canvas_h)` — the first candidate
    /// ITEM shown (`top`), the number of candidate DISPLAY LINES actually drawn (`lines`:
    /// headers + rows for the grouped/faceted path, rows for the flat path), the 0-based
    /// position of the SELECTED row AMONG those drawn candidate lines (`sel_row`), and the
    /// card / canvas heights. Lets a headless test assert the card is BOUNDED (`card_h ≤
    /// canvas_h`) and the selection stays visible (`sel_row < lines`) — the two guarantees
    /// the windowing exists to keep. Reads the SAME [`Self::overlay_geometry`] the card
    /// renders from, so the report can never claim a window the pixels don't show.
    pub fn overlay_window_report(&self) -> Option<(usize, usize, usize, f32, f32)> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let canvas_h = self.window_h;
        // The sidecar reports the PLAN, not a parallel count: `lines`
        // is the planned candidate band's own length and `sel_row` its planned
        // selected display line. A report can no longer claim a window the drawn
        // rows and the hit-test don't share.
        let plan = self.overlay_row_plan(&geom);
        Some((
            geom.top_idx,
            plan.candidate_rows(),
            plan.selected_display().unwrap_or(0),
            geom.card_h,
            canvas_h,
        ))
    }

    /// THE ONE PLANNING SEAM for the candidate-row band. Hands the
    /// already-resolved [`OverlayGeom`] (card box + window + header metrics) and
    /// the row pitch to the device-free scene planner, which emits one
    /// [`PlannedRow`] per candidate display line and its interaction geometry.
    ///
    /// Every downstream consumer — the selected band, the bar plates, the chord
    /// plates, the footer plate, the range rails, the text clip bands, the pointer
    /// hit-test, and the sidecar window report — reads the resulting plan; none of
    /// them may compute a row's y. It is built once per overlay frame in
    /// `prepare_overlay` and threaded down, and freshly (still O(visible)) by the
    /// standalone pointer/report entry points, which have no frame to ride.
    pub(in crate::render) fn overlay_row_plan(&self, geom: &OverlayGeom) -> OverlayRowPlan {
        let (cluster_span, selected_offset, selected_display) = self.diagonal_row_extent();
        plan_overlay_rows(&OverlayRowPlanInput {
            card_x: geom.band_x(),
            card_w: geom.band_w(),
            text_top: geom.text_top,
            lh: self.overlay_lh(),
            header_gap: geom.header_gap,
            header_rows: geom.header_rows,
            visible: geom.visible,
            top_idx: geom.top_idx,
            n_items: geom.n_items,
            selected: self.overlay_selected,
            empty_rows: geom.empty.is_some() as usize,
            lines: geom.theme.then_some(geom.plan.as_slice()),
            dx_per_row: self.overlay_row_dx_step(),
            cluster_span,
            selected_offset,
            selected_display,
        })
    }

    /// THE RAIL GEOMETRY OWNER for the whole card: every VISIBLE range
    /// row's `(item index, rail)` pair, resolved through `rowlayout::rail_geom`
    /// against the SAME shaped-glyph measurements the value column draws from
    /// (`overlay_row_secondary_px` / `overlay_row_primary_px`) and the SAME row-y
    /// owner (the row plan) the highlight band uses.
    ///
    /// EMPTY unless the card genuinely carries rails AND the secondary column was
    /// granted (`overlay_right_shown` — a rail beside a yielded value column would
    /// be a control with no readout), so every other picker is byte-identical.
    /// Called by BOTH the draw path and the pointer hit-test — a rail is clickable
    /// exactly where it is drawn, by construction.
    pub(in crate::render) fn overlay_rails(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Vec<(usize, crate::render::rowlayout::Rail)> {
        if self.overlay_ranges.is_empty() || !self.overlay_right_shown {
            return Vec::new();
        }
        let secondary = self.overlay_row_secondary_px(geom);
        let primary = self.overlay_row_primary_px(geom);
        let cluster = self.diagonal_cluster;
        let mut out = Vec::new();
        for row in plan.rows() {
            let Some(item) = row.item else {
                continue;
            };
            let Some(Some(frac)) = self.overlay_ranges.get(item).copied() else {
                continue;
            };
            let k = row.display;
            let value_w = secondary.get(&k).copied().unwrap_or(0.0);
            let label_w = primary.get(&k).copied().unwrap_or(0.0);
            let flow = self.overlay_accessory_flow();
            // WHERE the accessory hangs is the lane owner's one answer
            // (`overlay_accessory_anchor`, which the accessory upload, the frost's
            // surface list and the sidecar's own projection all ask); only how much
            // room is LEFT beside it still differs by composition — a spined card
            // reserves it off the cluster, an upright one off what the row leaves.
            let anchor = self.overlay_accessory_anchor(geom, k);
            let avail = match cluster {
                Some(cluster) => cluster.accessory_w(),
                None => (anchor - value_w) - (geom.text_left + label_w),
            };
            if let Some(rail) = crate::render::rowlayout::rail_geom(
                anchor, flow, value_w, avail, row.top, row.height, frac,
            ) {
                out.push((item, rail));
            }
        }
        out
    }

    pub fn overlay_range_at(&self, px: f32, py: f32) -> Option<(usize, f32)> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        self.overlay_rails(&geom, &plan)
            .into_iter()
            .find_map(|(item, rail)| {
                crate::render::rowlayout::rail_hit(&rail, px, py).then(|| {
                    (
                        item,
                        crate::render::rowlayout::rail_frac_at(px, rail.x0, rail.x1),
                    )
                })
            })
    }

    pub fn overlay_range_scale(&self, item: usize) -> Option<(f32, f32)> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        self.overlay_rails(&geom, &plan)
            .into_iter()
            .find_map(|(i, rail)| (i == item).then_some((rail.x0, rail.x1)))
    }

    /// The `overlay_items` index a pointer at PHYSICAL `(px, py)` selects — the
    /// value to assign to [`crate::overlay::OverlayState::selected`] — resolved by
    /// INVERTING the very [`PlannedRow`] slots the rows are drawn from. Both card
    /// families answer through the one planner, so a hovered/clicked row can never
    /// disagree with the highlighted one, and the faceted card's section headers
    /// (which carry no item) reject a click by construction.
    ///
    /// Deliberately NOT part of the item-164 visual-selection transaction: a click
    /// accepts the row under the pointer on the frame it lands, however far behind
    /// the animated band happens to be.
    pub fn overlay_row_at(&self, px: f32, py: f32) -> Option<usize> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        self.overlay_row_plan(&geom).row_at(px, py)
    }

    /// THE ONE HOVER-RESOLUTION SEAM: hit-test `(px, py)` against
    /// this pipeline's live overlay row geometry ([`Self::overlay_row_at`]),
    /// then run the result through [`crate::overlay::OverlayState::hover_at`]'s
    /// real-motion + movement-slop gate. Both `App::overlay_hover` (the live
    /// `CursorMoved` path, `app/input/mouse.rs`) and the headless `--keys`
    /// pointer-replay step (`ReplaySession::apply_move`, `main/run.rs`, via
    /// `capture::OraclePipeline::resolve_overlay_hover`, which wraps the
    /// SAME [`crate::render::TextPipeline`] this method is on) route through
    /// this single function — "what counts as a hover move" can never drift
    /// between live input and a scripted reproduction, and a new caller can't
    /// grow a second, hand-rolled hit-test-then-hover_at pair.
    pub fn resolve_overlay_hover(
        &self,
        overlay: &mut crate::overlay::OverlayState,
        px: f32,
        py: f32,
    ) -> bool {
        let hit = self.overlay_row_at(px, py);
        overlay.hover_at(px, py, hit)
    }

    /// Hit-test a pointer at PHYSICAL `(px, py)` against the SUMMONED overlay's
    /// editable QUERY-INPUT line — the `› query` filter field every flat/nav/theme
    /// picker draws on top. Returns `true` when the pointer sits inside the
    /// field's own PLANNED line box, within the card's x-bounds. The contextual
    /// SPELL panel has NO query line (`header_rows == 0`), so the plan carries no
    /// query band and this always returns `false`. Used by
    /// `input.rs::sync_cursor_icon` to give the field the I-beam.
    ///
    /// **THE DRIFT THIS CLOSES.** Reading the bare row pitch here — `text_top ..
    /// text_top + lh` — describes the wrong box on the FLAT family, whose field
    /// is `lh + header_gap` tall (the beat is the BOTTOM of the field's own box,
    /// not a row after it) with its ink half-led LOW inside it. On the shipping
    /// default at 1200x800 the field draws `[64.0, 133.2]`, caret at 98.6 and
    /// baseline at 106.0, against a pointer band ending at 91.2: the I-beam sat
    /// in empty air above the text, missing by 7.4px at 1x and 14.8px at 2x. The
    /// GROUPED family was right by accident (its beat inflates the lens strip
    /// instead), which is how a parallel calculation survives review — it agrees
    /// on the arm somebody looked at.
    pub fn over_overlay_query(&self, px: f32, py: f32) -> bool {
        if !self.overlay_active {
            return false;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        let Some(field) = self.overlay_row_plan(&geom).query_band() else {
            return false;
        };
        px >= geom.card_x && px <= geom.card_x + geom.card_w && field.contains(py)
    }
}
