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

/// How far the `DockedTab` active plate's fill overlaps the card's own top
/// edge — enough to paint over the card's flat, dpi-unscaled border hairline
/// (`FLOAT_BORDER_RING_PX`) and its AA feather, so no border pixel survives
/// the tab's mouth. `Logical` + `Metrics::px` like the rest of this rect, so
/// the overlap only grows relative to that flat hairline as dpi/zoom rise.
const DOCKED_TAB_SEAM_OVERLAP: Logical = Logical(2.0);

impl TextPipeline {
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
    /// parts, and is planned as [`PlanLine::Location`] so the shaper can say so
    /// — UNLESS the active `LocationStyle` composes independently of any row
    /// (`LocationStyle::needs_plan_row`), in which case no line is planned for
    /// it at all: a style that draws itself off the card (`RotatedRail`) has
    /// no glyphs and no anchor to seat there, so charging it a row would only
    /// vacate one. A style that still anchors its OWN cue to that row's
    /// geometry (`Raked`) keeps the line; its slot is otherwise unchanged —
    /// the original defect was a heading in a list's voice, not the row's
    /// existence.
    pub(in crate::render) fn theme_plan(&self) -> Vec<PlanLine> {
        let mut out = Vec::with_capacity(self.overlay_items.len());
        let mut prev: Option<String> = None;
        let location = self.overlay_location.as_deref();
        let location_needs_row = theme::active().render_caps.location_style.needs_plan_row();
        for i in 0..self.overlay_items.len() {
            let sect = self
                .overlay_sections
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !sect.is_empty() && prev.as_deref() != Some(sect) {
                match location == Some(sect) {
                    true if location_needs_row => out.push(PlanLine::Location(sect.to_string())),
                    true => {}
                    false => out.push(PlanLine::Header(sect.to_uppercase())),
                }
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

    /// The grouped family's starvation-degrade arm for the count cue:
    /// re-window `full_plan` at `(item_top, item_visible)` (already re-derived
    /// by the caller with the cue's own overhead dropped) and recompute the
    /// row/height totals that follow from it. Split out of
    /// `theme_overlay_geometry` purely to keep that function under its own
    /// line budget — every input here is a value that function already
    /// resolved, so this owns no policy of its own.
    #[allow(clippy::too_many_arguments)]
    fn theme_window_without_cue(
        &self,
        full_plan: &[PlanLine],
        (item_top, item_visible): (usize, usize),
        billed_header_rows: usize,
        empty_rows: usize,
        hint_rows: usize,
        hint_gap_rows: usize,
        footer_rows: usize,
        header_gap: f32,
        pad: f32,
    ) -> (usize, Vec<PlanLine>, f32) {
        let plan = window_plan(full_plan, item_top, item_top + item_visible);
        let total_rows =
            billed_header_rows + plan.len() + empty_rows + hint_gap_rows + hint_rows + footer_rows;
        let card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        (item_top, plan, card_h)
    }

    /// The grouped family's starvation-degrade arm for the header/query gap —
    /// the same kind of decorative, non-load-bearing chrome
    /// [`Self::theme_window_without_cue`] already degrades, dropped rather
    /// than pushing the card past the canvas.
    fn theme_card_h_without_header_gap(
        &self,
        total_rows: usize,
        hint_gap_rows: usize,
        header_gap: f32,
        hint_rows: usize,
        pad: f32,
    ) -> (usize, f32) {
        let total_rows = total_rows - hint_gap_rows;
        let card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, 0, pad);
        (0, card_h)
    }

    /// Where the grouped card's billed header-row count and top-left origin
    /// resolve to. Split out of `theme_overlay_geometry` purely to keep it
    /// under its own line budget; it owns no policy of its own.
    fn theme_card_placement(&self, header_rows: usize, margin: f32) -> (usize, f32) {
        // The QUERY line and the lens STRIP each own a header box (so
        // `strip_band()`/`docked_facet_band()` and every `line_i == 1` reader
        // — the hit-test, the tab-mark spans, the clip carve — keep a strip
        // line to read), but under `FacetStyle::DockedTab` the strip draws
        // OUTSIDE the card (`docked_facet_band`), so its box charges no `lh`
        // toward the rows/height math below — only the box COUNT stays 2,
        // never the billed space. Derived from the facet style's own data,
        // not this world's name, so any future `DockedTab` world reclaims
        // the same row.
        let billed_header_rows = header_rows - facet_strip_is_docked() as usize;
        let card_y = margin + self.metrics.px(super::CARD_TOP_DROP) + self.menubar_reserve();
        (billed_header_rows, card_y)
    }

    /// The grouped card's own box: its x/width (content-hugging on a
    /// right-anchored card, the wide `CARD_MAX_W_FACETED` cap otherwise), its
    /// fill regime, and the text column the two shapers read. Split out of
    /// `theme_overlay_geometry` purely to keep it under its own line budget.
    fn theme_card_box(&self, width: u32) -> (f32, f32, bool, f32, f32) {
        let desired_w = self.overlay_desired_w(super::CARD_MAX_W_FACETED);
        let (card_x, card_w) = self.overlay_card_box(width, desired_w);
        let card_narrow =
            super::overlay_card_fill_regime(width as f32, desired_w, self.metrics.scale);
        let hpad = self.overlay_text_hpad();
        let text_w = card_w - 2.0 * hpad;
        (card_x, card_w, card_narrow, hpad, text_w)
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
        // See `overlay_hint_gap_rows`'s own doc (`chrome/mod.rs`) — the ONE owner
        // this grouped family shares with the flat family (`overlay.rs`) and the
        // workspace family (`workspace.rs`), so a hint can't stop sitting flush
        // against the last row in one family while still doing it in another.
        let (mut hint, hint_rows, mut hint_gap_rows, footer, footer_rows, empty, empty_rows) =
            self.overlay_chrome_inventory(n_items);
        let header_rows = 2;
        let (billed_header_rows, card_y) = self.theme_card_placement(header_rows, margin);
        let header_gap = self.overlay_header_gap();
        let total_headers = full_plan.len() - n_items;
        // Strip + hint + footer here, at `min_items: 0`; the SECTION headers are
        // charged to the drawn WINDOW (`fit_sectioned_item_rows`).
        let chrome_rows = billed_header_rows + hint_gap_rows + hint_rows + empty_rows + footer_rows;
        // THE ONE HEIGHT-CLAMP OWNER, shared with the flat family.
        let avail_px = (self.window_h - card_y - margin - 2.0 * pad - header_gap).max(lh);
        // The cue's own fixed point (`resolve_window_and_cue`): `item_top`/
        // `item_visible` are ITEM counts straight off `scroll_window`, read
        // BEFORE `window_plan` turns them into a display-line count that
        // also bills section headers — passing THAT count here would
        // double-charge every header as a hidden item.
        let fit_window = |chrome_rows: usize| {
            let item_cap =
                self.overlay_sectioned_item_cap(avail_px, lh, chrome_rows, total_headers, 0);
            scroll_window(
                n_items,
                self.overlay_selected,
                self.overlay_scroll,
                item_cap,
            )
        };
        let (mut item_top, item_visible, mut cue_above, mut cue_below, mut cue_rows) =
            super::overlay_clamp::resolve_window_and_cue(n_items, |extra| {
                fit_window(chrome_rows + extra)
            });
        let mut plan = window_plan(&full_plan, item_top, item_top + item_visible);
        let total_rows = billed_header_rows
            + plan.len()
            + empty_rows
            + hint_gap_rows
            + hint_rows
            + footer_rows
            + cue_rows;
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
        let (card_x, card_w, card_narrow, hpad, text_w) = self.theme_card_box(width);
        hint = super::hint_yielding_explanation(&hint, width as f32 / self.metrics.scale.max(0.01));
        let mut card_h = self.overlay_card_h(total_rows, header_gap, hint_rows, hint_gap_rows, pad);
        if card_y + card_h > self.window_h + 0.01 && hint_gap_rows > 0 {
            (hint_gap_rows, card_h) = self.theme_card_h_without_header_gap(
                total_rows,
                hint_gap_rows,
                header_gap,
                hint_rows,
                pad,
            );
        }
        // The count cue is the SAME kind of decorative, non-load-bearing chrome
        // the hint gap already degrades above — in the same starvation corner,
        // drop it and re-fit the window at the ORIGINAL (uninflated) overhead
        // rather than let two rows of ambient position-keeping force a card
        // past its own canvas. Re-derives the window (never re-uses the
        // now-stale one) because freeing the cue's reserved overhead can let
        // the corpus fit a real item that overhead was crowding out.
        if card_y + card_h > self.window_h + 0.01 && cue_rows > 0 {
            (cue_above, cue_below, cue_rows) = (None, None, 0);
            (item_top, plan, card_h) = self.theme_window_without_cue(
                &full_plan,
                fit_window(chrome_rows),
                billed_header_rows,
                empty_rows,
                hint_rows,
                hint_gap_rows,
                footer_rows,
                header_gap,
                pad,
            );
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
            cue_above,
            cue_below,
            cue_reserved: cue_rows > 0,
        }
    }

    pub(super) fn overlay_shape_theme(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        inks: OverlaySpanInks,
        vis: &VisualSelection,
        trailing: &[String],
        elide: bool,
    ) -> bool {
        let ink = inks.ink;
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
        let mut strip_scale = 1.0;
        self.shape_theme_spans(
            geom,
            plan,
            active_ink,
            inks,
            vis,
            spans::ThemeStripSpec {
                text: &strip_s,
                labels: &label_ranges,
                separators: &sep_ranges,
                scale: 1.0,
            },
            trailing,
            elide,
        );
        let strip_w = self.theme_strip_px();
        if strip_w > geom.text_w {
            strip_scale = (geom.text_w / strip_w).max(0.5);
            self.shape_theme_spans(
                geom,
                plan,
                active_ink,
                inks,
                vis,
                spans::ThemeStripSpec {
                    text: &strip_s,
                    labels: &label_ranges,
                    separators: &sep_ranges,
                    scale: strip_scale,
                },
                trailing,
                elide,
            );
        }
        // RELOCATED SEAT (computed before any mark/glyph reading below, and
        // before `shape_docked_facet_strip` runs so BOTH read the frame it
        // produces): `DockedTab` moves the strip above the card, and a
        // `Split` composition's own seam moves it PAST the lower surface's
        // rim (`split_seam_active`/`floating_strip_band`) — either way the
        // relocated buffer, not `panel_buffer`'s own head band, is what
        // actually draws, so every mark below must read glyphs FROM there
        // too, or a mark computed against the un-relocated position would
        // disagree with where the label it marks actually renders.
        let dock_seat = self
            .docked_facet_band(geom, plan)
            .or_else(|| self.floating_strip_band(geom, plan));
        self.shape_docked_facet_strip(geom, strip_scale);
        // Record the active-lens mark from the shaped strip glyphs (line 1 of
        // `panel_buffer`, or line 0 of the relocated `docked_facet_buffer`).
        // Line-1 glyphs are byte-indexed WITHIN the strip line's own text —
        // the leading "\n" in `strip_s` split the lines — so a label's
        // line-relative range is its `strip_s` range shifted back by that one
        // "\n" byte; `docked_facet_buffer` carries no leading "\n" at all
        // (`shape_docked_facet_strip`'s own spans), so the SAME shift lands
        // it on that buffer's line 0 too. The MARK'S SHAPE is the PER-ITEM
        // LIST SURFACES round's `facet_style`:
        //   - `Text`   (default, byte-identical) — a hairline UNDERLINE under the
        //     active label.
        //   - `Band`   — a rounded value PILL behind the active label (the killed
        //     `Chips` skin's ghost-pill-per-label was dropped; only the active
        //     mark draws).
        // The x-spans come from the SAME shaped glyphs the strip hit-test reads, so
        // the skin can never disagree with where a label is clicked.
        // Scan `line_i` for a strip-range's glyph x-span (min_x, max_x) + the shaped
        // baseline (C2 y-owner), `None` if empty.
        let span_of = |buf: &GlyphBuffer,
                        line_i: usize,
                        r: &std::ops::Range<usize>|
         -> Option<(f32, f32, f32)> {
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
            // the primary/secondary columns use, or `dock_seat.top + run.line_y`
            // for a relocated buffer, whose own `line_top` is always `0.0` — a
            // single un-stacked line); the underline sits a hair BELOW it for
            // every face.
            let mut baseline = f32::MIN;
            for run in buf.layout_runs() {
                if run.line_i != line_i {
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
        // Whichever buffer actually draws the strip this frame — the ONE
        // reader every mark below shares, so a relocated label can never be
        // marked against the position it was relocated FROM.
        let mark_span = |r: &std::ops::Range<usize>| -> Option<(f32, f32, f32)> {
            if dock_seat.is_some() {
                span_of(&self.docked_facet_buffer, 0, r)
            } else {
                span_of(&self.panel_buffer, 1, r)
            }
        };
        // The absolute canvas origin `mark_span`'s baseline is relative to:
        // `panel_buffer`'s own stacked head band (`geom.text_top`) when
        // nothing relocated the strip, else the relocated seat's own top —
        // the buffer `push_docked_facet_areas` actually uploads it at.
        let mark_origin = dock_seat.map_or(geom.text_top, |s| s.top);
        let facet_style = crate::render::effective_facet_style();
        let scale = self.metrics.scale;
        // NO SEAT YET. Every mark rect below is computed in the strip's own
        // BUFFER-LOCAL x (as if seated at 0) and shifted to its real seat —
        // the same `overlay_head_left(geom, plan)` the emitter uses for the
        // head band's `TextArea` — only once [`Self::theme_reseat_marks`]
        // runs, later this frame. It cannot be added here: this shaping pass
        // runs BEFORE `self.diagonal_cluster` is resolved for the frame (the
        // cluster's own measurement reads THIS pass's shaped row widths), so
        // `overlay_head_left` would still see last frame's — or no — cluster
        // and silently answer with the upright seat on a banded world. Adding
        // it here once produced exactly that: correct on every world this
        // pass could see resolved, wrong on the one whose resolution hadn't
        // happened yet.
        const CHIP_HPAD: Logical = Logical(6.0);
        const CHIP_VPAD: Logical = Logical(2.0);
        let chip_hpad = self.metrics.px(CHIP_HPAD);
        let underline_drop = self.metrics.px(UNDERLINE_BASELINE_DROP);
        let strip_text_lh = self.metrics.line_height * crate::render::effective_overlay_scale();
        let chip_h = (strip_text_lh - 2.0 * self.metrics.px(CHIP_VPAD)).max(1.0);
        // RELOCATED SEAT (`dock_seat`, computed above): `strip_band()` is the
        // strip's PLAIN folded header-line box, centred by cosmic-text's own
        // half-leading — free room only when nothing else claims part of it.
        // Two compositions claim part of it and supply their OWN seat
        // instead (`DockedTab` above the card, a `Split` composition's own
        // seam past the lower surface's rim) — every mark here reads
        // whichever seat is active, so a pill/tick can never draw above a
        // plate it is meant to sit inside. `dock_seat` is `None` on every
        // other composition, leaving `s.center()` untouched: BYTE-IDENTICAL
        // off either gate (every `Bars`/`Diagonal`/`Ruled` world, and
        // Cassowary's `Unified`).
        let mark_cy = dock_seat
            .or_else(|| plan.strip_band())
            .map_or(geom.text_top, |s| s.center());
        let pill_px = |left: f32, right: f32| -> [f32; 4] {
            [
                left,
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
            let x0 = l;
            let x1 = r;
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
                if let Some((min_x, max_x, _)) = mark_span(r) {
                    v.push(pill_px(min_x - chip_hpad, max_x + chip_hpad));
                }
            }
            v
        };
        self.overlay_theme_underline = active_range.as_ref().and_then(|ar| {
            let (min_x, max_x, baseline) = mark_span(ar)?;
            match facet_style {
                theme::FacetStyle::Text => {
                    let y = mark_origin + baseline + underline_drop;
                    Some([
                        min_x,
                        y,
                        max_x - min_x,
                        self.metrics.px(TEXT_MARK_THICKNESS),
                    ])
                }
                theme::FacetStyle::Band => Some(pill_px(min_x - chip_hpad, max_x + chip_hpad)),
                theme::FacetStyle::DockedTab => {
                    let tab = [
                        min_x - chip_hpad,
                        geom.card_y - chip_h,
                        max_x - min_x + 2.0 * chip_hpad,
                        chip_h,
                    ];
                    // THE TAB'S MOUTH: the ghost ring frames the tab at its true
                    // bounds (its own bottom-edge stroke lands on the card's top
                    // border). The PLATE below is deliberately taller — it
                    // overlaps that stroke AND the card's own border sliver with
                    // the card's own ground color, so the active facet reads
                    // continuous with the card instead of a chip floating above it.
                    ghosts.push(tab);
                    let seam_overlap = self.metrics.px(DOCKED_TAB_SEAM_OVERLAP);
                    Some([tab[0], tab[1], tab[2], tab[3] + seam_overlap])
                }
                theme::FacetStyle::Chips(v) => match v {
                    theme::ChipVariant::Hairline | theme::ChipVariant::FilledActive => {
                        if matches!(v, theme::ChipVariant::Hairline) {
                            ghosts = inactive_pills();
                        }
                        Some(pill_px(min_x - chip_hpad, max_x + chip_hpad))
                    }
                    theme::ChipVariant::Underline => {
                        let y = mark_origin + baseline + underline_drop;
                        Some([
                            min_x,
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
        self.overlay_theme_facet_ghosts = ghosts;
        // A tab PILL is a plate, so `Ruled` is deliberately absent — it draws
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
                    mark_span(r).map(|(min_x, max_x, _)| pill_px(min_x - chip_hpad, max_x + chip_hpad))
                })
                .collect()
        } else {
            Vec::new()
        };
        false
    }

    /// THE ONE PLACE THE STRIP MARKS' X GAINS A SEAT — called once per frame,
    /// after `self.diagonal_cluster` has resolved (`overlay_draw::prepare_overlay`,
    /// right after `resolve_diagonal_cluster`, before anything downstream reads
    /// these rects). `overlay_shape_theme` above records every mark rect
    /// BUFFER-LOCAL (as if seated at 0), because it runs earlier in the same
    /// frame than the cluster it would otherwise need to ask
    /// [`Self::overlay_head_left`] for — this is the single, later point where
    /// that seat is actually known, so it is added exactly once, to exactly
    /// the rects `overlay_shape_theme` left local. A no-op (adds `0.0`) on any
    /// contextual card (`geom.theme` false — no strip, no marks recorded)
    /// or any upright world, where `overlay_head_left` already answers
    /// `geom.text_left`.
    pub(super) fn theme_reseat_marks(&mut self, geom: &OverlayGeom, plan: &OverlayRowPlan) {
        if !geom.theme {
            return;
        }
        let seat = self.overlay_head_left(geom, plan);
        if let Some(r) = self.overlay_theme_underline.as_mut() {
            r[0] += seat;
        }
        for r in self.overlay_theme_facet_ghosts.iter_mut() {
            r[0] += seat;
        }
        for r in self.overlay_strip_tab_plates.iter_mut() {
            r[0] += seat;
        }
    }

    /// TEST HOOK: the DockedTab active-plate seam overlap this frame's
    /// `Metrics` resolves `DOCKED_TAB_SEAM_OVERLAP` to — so a law can assert
    /// the drawn geometry against the SAME scaled value the draw path used
    /// rather than a re-derived guess.
    #[cfg(test)]
    pub(in crate::render) fn docked_tab_seam_overlap_probe(&self) -> f32 {
        self.metrics.px(DOCKED_TAB_SEAM_OVERLAP)
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_theme_spans(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        active_ink: glyphon::Color,
        inks: OverlaySpanInks,
        vis: &VisualSelection,
        strip: spans::ThemeStripSpec,
        trailing: &[String],
        elide: bool,
    ) {
        let OverlaySpanInks { ink, muted, .. } = inks;
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
        self.push_theme_strip_spans(&mut spans, plan, strip, active_ink, muted);
        // The ABOVE-edge count cue's own line, opened between the strip and
        // the first candidate row by `plan_overlay_rows`'s `first_top` shift
        // (`OverlayRowPlanInput::cue_above_rows`, fed from `geom.cue_reserved`
        // — the scroll-INVARIANT reservation, never `geom.cue_above.is_some()`
        // itself, so this line's existence cannot appear or vanish as the
        // reader scrolls through an already-open card). `push_theme_plan_spans`
        // always opens ITS OWN leading "\n" for display line 0, so this needs
        // no `content_before` bookkeeping. Blank (a bare space) when this
        // edge's own `cue_above` has nothing to say at the CURRENT scroll.
        // Owned `String`s, not temporaries: `spans` borrows past this
        // function's own scope (into `set_rich_text`), so the cue text must
        // live at least as long as `title_prefix` does.
        let cue_above_text = geom.cue_above.map(|n| super::edge_cue_text(true, n));
        if geom.cue_reserved {
            spans.push(("\n", mk(muted)));
            spans.push((cue_above_text.as_deref().unwrap_or(" "), mk(muted)));
        }
        self.push_theme_plan_spans(&mut spans, geom, &fitted, trailing, inks, vis);
        if let Some(msg) = &geom.empty {
            spans.push(("\n", mk(muted)));
            spans.push((msg.as_str(), mk(muted)));
        }
        // The BELOW-edge cue, directly under the last drawn line of
        // `geom.plan` (or the empty-state notice, sharing that band —
        // `content_rows`'s own ordering: rows, then the notice, then this;
        // mutually exclusive in practice, since the reservation only ever
        // fires while `n_items > 0`), ahead of the hint/footer that may
        // follow it. Same reservation-vs-content split as above.
        let cue_below_text = geom.cue_below.map(|n| super::edge_cue_text(false, n));
        if geom.cue_reserved {
            spans.push(("\n", mk(muted)));
            spans.push((cue_below_text.as_deref().unwrap_or(" "), mk(muted)));
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
