use super::*;

/// Dense chrome size; every overlay geometry owner consumes this scale.
pub(in crate::render) const OVERLAY_UI_SCALE: f32 = 0.85;

pub(in crate::render) const CARD_EDGE_INSET_FLOOR: f32 = 10.0;

pub(in crate::render) fn overlay_rail_inset(ww: f32) -> f32 {
    (ww / 3.0 - CARD_MAX_W * 0.5).max(0.0)
}
pub(in crate::render) const CARD_MAX_W: f32 = 520.0;
pub(in crate::render) const CARD_MAX_W_FACETED: f32 = 600.0;
pub(in crate::render) const CARD_CONTENT_MIN_W: f32 = 160.0;

/// Query-to-results breathing room, shared by flat and faceted cards.
const OVERLAY_QUERY_BEAT: f32 = 1.55;

const OVERLAY_HINT_ROW: f32 = 0.70;

const OVERLAY_FOOTER_PAD: f32 = 2.0;

pub(in crate::render) fn overlay_card_box_policy(
    anchor: theme::CardAnchor,
    ww: f32,
    desired_w: f32,
) -> (f32, f32) {
    let floor = CARD_EDGE_INSET_FLOOR;
    let full = overlay_rail_inset(ww);
    let cw = desired_w.min((ww - 2.0 * floor).max(0.0));
    let free = (ww - cw).max(0.0);
    let anchored_max = (ww - floor - cw).max(floor);
    let left = match anchor {
        theme::CardAnchor::TopCenter => free * 0.5,
        theme::CardAnchor::TopLeft => full.min(anchored_max).max(floor).min(free),
        theme::CardAnchor::Inset { x_frac } => {
            let span = (ww - cw - 2.0 * full).max(0.0);
            (full + x_frac.clamp(0.0, 1.0) * span)
                .min(anchored_max)
                .max(floor)
                .min(free)
        }
        theme::CardAnchor::TopRight => {
            let span = (ww - cw - 2.0 * full).max(0.0);
            (full + span).min(anchored_max).max(floor).min(free)
        }
    };
    (left, cw)
}

pub(in crate::render) fn overlay_card_fill_regime(ww: f32, desired_w: f32) -> bool {
    desired_w > (ww - 2.0 * CARD_EDGE_INSET_FLOOR).max(0.0)
}

impl TextPipeline {
    pub(in crate::render) fn overlay_metrics(&self) -> GlyphMetrics {
        let m = self.metrics;
        let scale = crate::render::effective_overlay_scale();
        GlyphMetrics::new(
            m.font_size * scale,
            m.line_height * scale
                + crate::render::effective_overlay_leading()
                + self.overlay_row_gap(),
        )
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
        match crate::render::effective_list_style() {
            theme::ListStyle::Bars { gap, .. } => gap.max(0.0),
            theme::ListStyle::Pane | theme::ListStyle::Diagonal(_) => 0.0,
        }
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
        match crate::render::effective_list_style() {
            theme::ListStyle::Bars { .. } => BAR_SIDE_INSET + BAR_TEXT_PAD,
            theme::ListStyle::Pane | theme::ListStyle::Diagonal(_) => 12.0,
        }
    }

    /// The overlay row LINE HEIGHT — the single-owner metric the card height, the
    /// row-Y ([`overlay_row_top`]), the hit-test ([`overlay_row_of`]), and the
    /// selected-row band all read, so a click always lands on the row it highlights.
    pub(in crate::render) fn overlay_lh(&self) -> f32 {
        self.metrics.line_height * crate::render::effective_overlay_scale()
            + crate::render::effective_overlay_leading()
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
        )
    }

    pub(in crate::render) fn overlay_pixel_scale(&self) -> f32 {
        self.metrics.font_size / crate::render::FONT_SIZE
    }

    /// THE ONE OWNER of the summoned card's WIDE desired width (device px) at the
    /// CURRENT zoom/DPI: the base cap ([`CARD_MAX_W`] / [`CARD_MAX_W_FACETED`],
    /// tuned for the 1:1 capture canvas) GROWN by [`Self::overlay_pixel_scale`] so
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
    /// GROW-ONLY (`scale.max(1.0)`): the scale only ever WIDENS the base cap. The
    /// bug is a high-zoom COLLAPSE, so the fix touches exactly the `zoom·dpi > 1.0`
    /// regime. At the SHIPPED default (zoom 0.8, dpi 1.0 → scale 0.8) and every
    /// scale ≤ 1.0 this is the identity — the card holds the base cap, BYTE-
    /// IDENTICAL to the pre-fix `base`-passthrough (so the 0.8 default look and
    /// every ≤1.0 capture/law are untouched; a slightly-roomier-than-proportional
    /// card at low zoom never clips).
    pub(in crate::render) fn overlay_card_desired_w(&self, base: f32) -> f32 {
        base * self.overlay_pixel_scale().max(1.0)
    }

    pub(in crate::render) fn overlay_right_anchored(&self) -> bool {
        crate::render::resolve_overlay_anchor(self.overlay_align).mirrors_growth()
    }

    pub(in crate::render) fn overlay_desired_w(&self, base_cap: f32) -> f32 {
        let scaled = self.overlay_card_desired_w(base_cap);
        if self.overlay_right_anchored() && self.overlay_content_w > 0.0 {
            let floor = (CARD_CONTENT_MIN_W * self.overlay_pixel_scale().max(1.0)).min(scaled);
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
    /// COMPOSITION ROUND (item 4) widened it from ~0.55 to [`OVERLAY_QUERY_BEAT`]
    /// of a row — a clearer beat between the input line and the first result,
    /// still short of the "fat lip" of a whole blank row. It is STRUCTURAL, not a
    /// leading newline (the f2cb656 tripwire): the shaper inflates the last
    /// header line's REAL glyph metrics by exactly this, and the band, primary
    /// name, secondary chord, hit-test, and caret all fold it in through the ONE
    /// y-owner family ([`overlay_row_top`] / [`overlay_secondary_top`]) — so text
    /// and band move together, never a half-row split. Both geometry owners read
    /// this; the contextual spell popup passes `0.0` (no header to divide from).
    /// LIVE-ONLY taste: whether the widened beat reads right needs a human eye.
    pub(in crate::render) fn overlay_header_gap(&self) -> f32 {
        (self.overlay_lh() * OVERLAY_QUERY_BEAT).round()
    }

    pub(in crate::render) fn overlay_hint_h(&self) -> f32 {
        (self.overlay_lh() * OVERLAY_HINT_ROW).round()
    }

    pub(in crate::render) fn overlay_footer_reclaim(&self, hint_rows: usize) -> f32 {
        hint_rows as f32 * (self.overlay_lh() - self.overlay_hint_h() - OVERLAY_FOOTER_PAD).max(0.0)
    }

    pub(in crate::render) fn overlay_card_h(
        &self,
        total_rows: usize,
        header_gap: f32,
        hint_rows: usize,
        pad: f32,
    ) -> f32 {
        total_rows as f32 * self.overlay_lh() + header_gap + 2.0 * pad
            - self.overlay_footer_reclaim(hint_rows)
    }

    /// THE ONE STRIP-BAND OWNER — the faceted theme picker's lens STRIP sits on
    /// display line 1, whose height is inflated to `lh + header_gap` by the query
    /// BEAT (cosmic-text half-leads the labels into that taller box, so they center
    /// below a plain `lh` band). Returns `(strip_top, strip_lh)`: the strip's top
    /// edge (`text_top + lh`) and its inflated line height. The lens hit-test
    /// ([`TextPipeline::overlay_lens_at`]), the active-facet pill center, and the
    /// strip-label glyph metrics all read THIS — so the clickable band, the pill,
    /// and the shaped glyphs can never disagree about where the strip sits (the
    /// misaligned-chip / half-row band-vs-text drift class). Flat pickers have no
    /// strip; this is meaningful only when `geom.theme`.
    pub(in crate::render) fn overlay_strip_band(&self, geom: &OverlayGeom) -> (f32, f32) {
        let lh = self.overlay_lh();
        (geom.text_top + lh, lh + geom.header_gap)
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
        // ITEM 114 — the THIRD family. Checked before the faceted one because a
        // workspace's rail IS its facet strip, stood on its end: the same data
        // reaches a different presentation, and there is no card to place.
        if self.overlay_is_workspace() {
            return self.workspace_geometry(width);
        }
        if !self.overlay_lens.is_empty() {
            return self.theme_overlay_geometry(width);
        }
        let pad = 12.0;
        let margin = 12.0;
        let n_items = self.overlay_items.len();

        let hint = self.overlay_hint.clone();
        let hint_rows = if hint.is_empty() { 0 } else { 1 };

        let (footer, footer_rows) = self.overlay_footer_lines();

        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;

        let contextual = self.overlay_context_anchor.is_some();
        let header_rows = usize::from(!contextual); // contextual rows need no query field
        // PALETTE-COMPOSITION round: a calm gap after the query header, before the
        // candidate list (negative space as the divider). Grows the card by exactly
        // this and offsets the candidate band/hit-test through `overlay_row_top`.
        let header_gap = if contextual {
            0.0
        } else {
            self.overlay_header_gap()
        };
        let card_y = self
            .overlay_context_anchor
            .map(|(_, y)| y + 4.0)
            .unwrap_or(margin + 40.0 + self.menubar_reserve());
        // ITEM 181 — cap the item window to what the canvas fits, same owner the
        // grouped family reads (`theme_overlay_geometry`).
        let avail_px = if contextual {
            (self.window_h - 2.0 * margin).max(self.overlay_lh())
        } else {
            (self.window_h - card_y - margin - 2.0 * pad - header_gap).max(self.overlay_lh())
        };
        let chrome_rows = header_rows + hint_rows + empty_rows + footer_rows;
        let (top_idx, visible) = self.overlay_flat_window(n_items, avail_px, chrome_rows);
        let total_rows = header_rows + visible + empty_rows + hint_rows + footer_rows;
        let desired_w = self.overlay_desired_w(CARD_MAX_W);
        let (mut card_x, card_w) = self.overlay_card_box(width, desired_w);
        if let Some((x, _)) = self.overlay_context_anchor {
            card_x = x.clamp(
                CARD_EDGE_INSET_FLOOR,
                (width as f32 - card_w - CARD_EDGE_INSET_FLOOR).max(CARD_EDGE_INSET_FLOOR),
            );
        }
        let card_narrow = overlay_card_fill_regime(width as f32, desired_w);
        let hpad = self.overlay_text_hpad();
        let text_w = card_w - 2.0 * hpad;
        let card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, pad);
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

    pub(in crate::render) fn measure_spell_content_w(&mut self) -> f32 {
        if self.overlay_items.is_empty() {
            return 0.0;
        }
        let ui_metrics = self.overlay_metrics();
        self.panel_buffer
            .set_metrics(&mut self.font_system, ui_metrics);
        self.panel_buffer
            .set_size(&mut self.font_system, None, None);
        let text = self.overlay_items.join("\n");
        let ink = theme::base_content().to_glyphon();
        self.panel_buffer.set_text(
            &mut self.font_system,
            &text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut max_w = 0.0_f32;
        for run in self.panel_buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
        }
        max_w
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
        let pad = 10.0;
        let margin = 8.0;
        let gap = 6.0; // the breath between the word and the panel
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

        // ITEM 181 — the popup's own budget: whichever side (above/below the
        // word) is roomier.
        let below_avail = self.window_h - (word_top + word_h + gap) - margin;
        let above_avail = word_top - gap - margin;
        let avail_px = below_avail.max(above_avail).max(self.overlay_lh());
        let (top_idx, visible) =
            self.overlay_flat_window(n_items, avail_px, header_rows + hint_rows);

        // Width: fit the WIDEST suggestion ROW — its real SHAPED width, measured into
        // `overlay_spell_w` at sync — plus padding, NOT the anchor word. So a short
        // misspelled word ("teh") can no longer make a narrow card the longer
        // corrections overflow. A calm MIN keeps a lone short suggestion from looking
        // pinched; the card stays capped small and clamped on-canvas. (Falls back to
        // the char-count estimate only if a measurement has not run yet.)
        //
        // ITEM 49 — MEASURE EVERY ROW IN THE ELISION'S OWN UNITS: the shaped
        // `overlay_spell_w` is in the UI/chrome face, but the row-elision budget
        // (`overlay_shape_text` → `rowlayout::plan`) divides `text_w` by the DOCUMENT
        // mono `char_width` and then charges the primary for the `…` cell AND the
        // (blank but PRESENT) right column the popup's per-row `overlay_bindings`
        // carries — one empty slot per suggestion + the add row. On a WIDE-mono world
        // (Firetail / Monaspace Xenon) that grid needs MORE width than the UI-face
        // shaped measurement grants — so the first-class "Add '<word>' to dictionary"
        // row elided at a WIDE width, floating in empty space (the user report). Floor
        // the content width by the char-grid width of the WIDEST row plus the cells
        // `rowlayout::plan` reserves off the top — `1` (the `…`) + `GAP_CHARS` (the
        // secondary-column gutter) — plus one more so the integer `floor(text_w /
        // char_width)` clears the target even when the f32 division lands a hair under
        // a whole cell. `max()` only GROWS the card where the grid outruns the shaped
        // width (byte-identical where the shaped face is the wider of the two); the cap
        // below still elides a genuinely over-long row as the last resort.
        let widest_chars = self
            .overlay_items
            .iter()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or(0);
        let grid_slack = 1 + crate::render::rowlayout::GAP_CHARS + 1;
        let char_grid_w = (widest_chars + grid_slack) as f32 * m.char_width;
        let content_w = if self.overlay_spell_w > 0.0 {
            self.overlay_spell_w.max(char_grid_w)
        } else {
            char_grid_w
        };
        // The MIN/MAX bounds are tuned for the 1:1 capture canvas; GROW them with the
        // current zoom/DPI (the SAME grow-only `overlay_pixel_scale` the takeover
        // card's width uses) so a long correction isn't clamped to an unzoomed cap
        // while its shaped `content_w` doubled under zoom — the zoom-blind card bug,
        // contextual sibling. Grow-only (`scale.max(1.0)`): byte-identical at every
        // scale ≤ 1.0 (the shipped 0.8 default + all captures untouched). The MAX is
        // wide enough to hold a whole add-to-dictionary row for an ordinary word
        // (~24 mono cells) so it never elides at wide width (item 49); a genuinely
        // long adversarial word still overruns it and elides — a small popup, never a
        // takeover card.
        let scale = self.overlay_pixel_scale().max(1.0);
        let card_w = (content_w + 2.0 * pad)
            .clamp(140.0 * scale, 520.0 * scale)
            .min(width as f32 - 2.0 * margin);
        let text_w = card_w - 2.0 * pad;
        let rows = header_rows + visible.max(1) + hint_rows;
        let card_h = self.overlay_card_h(rows, 0.0, 0, pad);

        let mut card_x = word_x;
        if card_x + card_w > width as f32 - margin {
            card_x = (width as f32 - margin - card_w).max(margin);
        }
        card_x = card_x.max(margin);
        let below_y = word_top + word_h + gap;
        let card_y = if below_y + card_h <= self.window_h - margin {
            below_y
        } else {
            (word_top - gap - card_h).max(margin)
        };
        let text_left = card_x + pad;
        let text_top = card_y + pad;
        OverlayGeom {
            visible,
            top_idx,
            n_items,
            hint,
            hint_rows,
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
        // ITEM 174 — the sidecar reports the PLAN, not a parallel count: `lines`
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

    /// ITEM 174 — THE ONE PLANNING SEAM for the candidate-row band. Hands the
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
        let (cluster_span, selected_offset, selected_display) = self.diagonal_cluster.map_or(
            (None, None, None),
            crate::render::chrome::diagonal::DiagonalClusterRail::row_plan,
        );
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

    /// ITEM 94 — THE RAIL GEOMETRY OWNER for the whole card: every VISIBLE range
    /// row's `(item index, rail)` pair, resolved through `rowlayout::rail_geom`
    /// against the SAME shaped-glyph measurements the value column draws from
    /// (`overlay_row_secondary_px` / `overlay_row_primary_px`) and the SAME row-y
    /// owner (`overlay_row_top`) the highlight band uses.
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
            let (text_right, avail) = match cluster {
                Some(cluster) => (cluster.accessory_right(k), cluster.accessory_w()),
                None => {
                    let text_right = geom.text_left + geom.text_w;
                    (
                        text_right,
                        (text_right - value_w) - (geom.text_left + label_w),
                    )
                }
            };
            if let Some(rail) = crate::render::rowlayout::rail_geom(
                text_right, value_w, avail, row.top, row.height, frac,
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

    /// ITEM 106 — THE ONE HOVER-RESOLUTION SEAM: hit-test `(px, py)` against
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
    /// picker draws on top (`header_rows == 1`). Returns `true` when the pointer
    /// sits on that one row, within the card's x-bounds. The contextual SPELL
    /// panel has NO query line (`header_rows == 0`), so it always returns `false`.
    /// Reads the SAME [`Self::overlay_geometry`] the query line renders from (its
    /// row is `text_top .. text_top + line_height`, the row just above the
    /// candidate window), so this can never disagree with where the field draws.
    /// Used by `input.rs::sync_cursor_icon` to give the field the I-beam.
    pub fn over_overlay_query(&self, px: f32, py: f32) -> bool {
        if !self.overlay_active {
            return false;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        if geom.header_rows == 0 {
            return false;
        }
        let lh = self.overlay_lh();
        px >= geom.card_x
            && px <= geom.card_x + geom.card_w
            && py >= geom.text_top
            && py < geom.text_top + lh
    }
}
