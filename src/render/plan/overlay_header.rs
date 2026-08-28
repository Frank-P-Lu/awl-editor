//! The planned HEADER BAND of a summoned overlay card.
//!
//! The display lines ABOVE the candidate band: the query/title INPUT field every
//! takeover picker draws, and the grouped family's lens STRIP under it. One
//! module owns where they sit, how tall they are, and everything derived from
//! that — the caret's centre, the pointer's band, the secondary column's upload
//! origin, and the split-pane composition's visible gap.

use super::OverlayRowPlan;
use super::overlay_rows::OverlayRowPlanInput;

/// The visible-BACKGROUND strip between a split Pane card's two surfaces is this
/// fraction of the query BEAT tall. Glyph-free by the half-leading
/// CENTRING bound: an inflated line box centres its glyph run, so the run's far
/// edge clears the band's near edge as long as its own font height stays under
/// `lh + header_gap·(1 - 2·frac)` — a bound against OVERLAP, not against
/// reading clear of it. MEASURED (pixel arithmetic over a real capture, not
/// the formula alone): at 0.4 the facet strip's own ink starts a bare ~3
/// physical px below the lower surface's rim at dpi 1 — real, non-overlapping,
/// but on a `Bordered` world (a literal black-on-white rim) that reads as
/// touching. Lowered for real breathing room while leaving `BREATHE_FRAC` —
/// the query box's OWN already-tuned symmetric breathing — untouched: only the
/// gap's own thickness shrinks, its START position does not move, and neither
/// `first_top` nor `card_h` reads this constant at all, so no row rhythm or
/// card height moves. **Floored above a lower value that was tried first**:
/// `chip_plate_floor`'s own mark-floor proves it bites by reconstructing the
/// naive pre-fix centre and showing it draws above the lower surface's plate
/// — that proof goes vacuous once the plate's own top (this fraction) pulls
/// far enough ahead of the naturally-centred mark, which this module's own
/// tests pin down between 0.35 and 0.25. This value keeps that non-vacuity
/// intact while still buying the facet strip real, measured clearance.
/// Reverting to the historical 0.4 is one line, plus the dependent literal
/// reconstructions in `tests/split_pane.rs` and `plan/tests.rs` (both
/// intentionally re-derive the fraction as an independent oracle rather than
/// reading the constant back).
pub(super) const SPLIT_GAP_FRAC: f32 = 0.35;

/// A split card's upper surface borrows this fraction of the SAME
/// already-proven-safe slack as symmetric breathing room below the query box
/// before the visible gap starts, so the query stops reading bottom-heavy inside
/// its own small strip.
pub(super) const BREATHE_FRAC: f32 = 0.2;

/// **THE HEADER BAND'S OWN RUN** — the distance from a card's `text_top` down to
/// its candidate band's `first_top`, for a card with `header_rows` display lines
/// above the band at pitch `lh` and a query beat of `header_gap`.
///
/// ONE owner, because this number is asked for from three unrelated places and
/// re-summing it at each is how they drift: the row planner's forward y
/// arithmetic (`row_top`), the header boxes below (whose LAST box closes exactly
/// on it), and a summoned workspace's relocated document viewport, which opens at the
/// same line the candidate band does. A workspace whose lens moved into
/// its header carries TWO header lines, so a consumer that re-summed `lh +
/// header_gap` for itself would seat the comparison a whole line high the moment
/// that second line appeared.
///
/// `header_rows == 0` (the contextual spell popup) still carries its own zero
/// beat, so this stays the plain offset in every family.
pub(in crate::render) fn header_band_height(header_rows: usize, lh: f32, header_gap: f32) -> f32 {
    header_rows as f32 * lh + header_gap
}

/// **WHERE THE BEAT STANDS.** The query BEAT is negative space BEFORE the first
/// candidate; the only question the band has to answer is whose LINE BOX owns
/// it. A card carrying a lens STRIP folds it into the strip's box, because
/// cosmic-text centres a line's glyph run in its box and that half-leading is
/// exactly what seats the strip below the split seam, clear of the query bar.
/// A card whose ONLY header line IS the query field has no such line to seat:
/// folding the beat there centres the field's glyphs in a box that runs all the
/// way to the first candidate, dropping them the better part of a row below the
/// query bar's own surface and opening a blank strip ABOVE them. So the beat
/// stands on its own after the header lines instead, and the shaper emits it as
/// a glyph-free line rather than as line-height on a line that has glyphs.
///
/// Reduced to one sentence: **THE QUERY FIELD'S BOX IS ALWAYS EXACTLY ONE LINE**,
/// and the beat closes the band from whichever box follows it.
pub(in crate::render) fn beat_stands_alone(header_rows: usize, header_gap: f32) -> bool {
    header_rows <= 1 && header_gap > 0.0
}

/// Lay out one box per header display line, stacked from `text_top` at the
/// candidate band's own pitch, with the query BEAT folded into the LAST box
/// exactly as the shaper folds it into that line's own glyph metrics — unless
/// that box is the query field's own ([`beat_stands_alone`]), in which case the
/// beat closes the band as its own glyph-free run. Either way the band closes on
/// `first_top`, so the query field, the lens strip and the first candidate row
/// cannot disagree about where one ends and the next begins.
///
/// The last box's height is whatever [`header_band_height`] has left over rather
/// than a second `lh + header_gap`, so the boxes and the band's run are one
/// derivation.
pub(super) fn plan_header_band(input: &OverlayRowPlanInput<'_>) -> Vec<PlannedHeader> {
    let n = input.header_rows;
    // `band_bottom` — and so `first_top`, via the SAME `header_band_height`
    // call `row_top` makes — reads the BILLED count, not the box count: a
    // docked facet strip still gets its own box below (so `strip_band()`
    // keeps a real line to hand `docked_facet_band`), but that box's OWN
    // height comes out of whatever `band_bottom` leaves over, never a fixed
    // `lh` of its own. `spans.rs` shapes the strip's glyph line at exactly
    // this box's height (`plan.strip_band().height`), so the shrink reaches
    // the shaped buffer through the one plan both read, never a second edit.
    let band_bottom =
        input.text_top + header_band_height(input.billed_header_rows, input.lh, input.header_gap);
    let folded = !beat_stands_alone(n, input.header_gap);
    (0..n)
        .map(|line| {
            let top = input.text_top + line as f32 * input.lh;
            PlannedHeader {
                line,
                top,
                height: match folded && line + 1 == n {
                    true => band_bottom - top,
                    false => input.lh,
                },
            }
        })
        .collect()
}

/// ONE PLANNED HEADER LINE — a display line ABOVE the candidate band: the
/// query/title INPUT line every summoned picker draws, and (on the grouped
/// family) the lens STRIP under it.
///
/// `top`/`height` are the LINE BOX in canvas px — the box cosmic-text half-leads
/// this line's glyphs into, which is what makes it the box the caret is centred
/// in, the box a pointer must fall inside to be "on the field", and the box the
/// split-pane composition carves its visible gap out of.
///
/// THE BEAT IS STRUCTURAL, never a leading newline. The query BEAT
/// (`header_gap`, `overlay_header_gap`'s calm slab of negative space before the
/// first candidate) either inflates the LAST header line's real glyph metrics
/// (`shape_theme_spans`'s `strip_lh` on a grouped card's line 1) or stands as
/// its own glyph-free line after them ([`beat_stands_alone`] — a flat card,
/// whose one header line is the query field itself). So header line `i` is
/// exactly `lh` tall, except a folded last line, which is `lh + header_gap`; the
/// band always closes on the candidate band's `first_top`, by construction.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::render) struct PlannedHeader {
    pub line: usize,
    pub top: f32,
    pub height: f32,
}

impl PlannedHeader {
    pub(in crate::render) fn bottom(&self) -> f32 {
        self.top + self.height
    }
    /// The y a glyph run half-led into this box centres on — where the query
    /// caret rides and where the strip's active mark is centred.
    pub(in crate::render) fn center(&self) -> f32 {
        self.top + self.height * 0.5
    }
    pub(in crate::render) fn contains(&self, py: f32) -> bool {
        py >= self.top && py < self.bottom()
    }
}

impl OverlayRowPlan {
    // --- THE HEADER BAND ------------------------------------------------------
    //
    // The three answers — the query field's box, the lens strip's box, and the
    // split-pane gap carved between the two surfaces — are one planned object
    // read three ways, never three separate owners in `render/chrome`.
    // `overlay_secondary_top`, `overlay_split_bounds`,
    // `overlay_strip_band` and `overlay_query_center` are GONE from
    // `render/chrome`; a consumer cannot re-derive a header line's position,
    // only read it off the plan the pixels came from.

    /// EVERY planned header line, in draw order. Read by the head band's own ink box,
    /// which must span the widest ink of every line riding that one `TextArea` — so a
    /// third header line enrols by existing rather than by someone remembering to add it
    /// beside [`Self::query_band`] and [`Self::strip_band`].
    pub(in crate::render) fn header_lines(&self) -> &[PlannedHeader] {
        &self.headers
    }

    /// How many `lh`-tall rows the header band actually BILLS before the
    /// candidate band starts — see [`super::overlay_rows::OverlayRowPlanInput
    /// ::billed_header_rows`]'s own doc. Paired with [`Self::secondary_top`]:
    /// a right-column buffer leads with exactly this many empty lines, and the
    /// two must come from one object or the column's origin and its own
    /// leading can disagree.
    pub(in crate::render) fn billed_header_rows(&self) -> usize {
        self.billed_header_rows
    }

    /// THE QUERY FIELD's own line box — where its glyphs are half-led, where its
    /// caret is centred, and the band a pointer must be inside to be "on the
    /// field". `None` for the contextual spell popup, which has no query line at
    /// all (`header_rows == 0`).
    ///
    /// The field's box is exactly ONE line on every family ([`beat_stands_alone`]):
    /// the caret it centres, the pointer band it defines and the glyphs the
    /// shaper half-leads into it are then one box, and the beat that follows is
    /// not part of the field.
    pub(in crate::render) fn query_band(&self) -> Option<PlannedHeader> {
        self.headers.first().copied()
    }

    /// THE BEAT'S OWN GLYPH-FREE LINE — its height, when the beat stands after
    /// the header lines rather than inside the last one. `None` when it is
    /// folded (a grouped card's lens strip) or when there is no beat at all.
    ///
    /// DERIVED from the planned boxes, never re-decided: whatever run the header
    /// lines leave between their last bottom and the candidate band's `first_top`
    /// IS the beat, so the shaper cannot emit a spacer the band did not plan.
    pub(in crate::render) fn beat_line(&self) -> Option<f32> {
        let last = self.headers.last()?;
        let slack = self.first_top() - last.bottom();
        (slack > 0.0).then_some(slack)
    }

    /// THE LENS STRIP's own line box on a GROUPED card — the last header line,
    /// which is the one the beat inflates. `None` on a flat picker or the spell
    /// popup (no strip is drawn), which is exactly when `geom.theme` is false.
    pub(in crate::render) fn strip_band(&self) -> Option<PlannedHeader> {
        (self.headers.len() >= 2).then(|| self.headers[self.headers.len() - 1])
    }

    /// The device-px TOP a uniform-line-height RIGHT-COLUMN buffer must be
    /// uploaded at so its chord/time labels — which lead with
    /// `billed_header_rows` empty lines — land EXACTLY on this plan's
    /// candidate band. The two share ONE y-origin by the invariant
    /// `secondary_top() + (billed_header_rows + r) * lh == row_top(r)`; the
    /// leading empties supply `billed_header_rows * lh` and this supplies the
    /// beat.
    ///
    /// THE COMPOSITION-ROUND BUG this closes: the beat is folded into the primary
    /// column (its inflated header line) AND the band/hit-test (through the
    /// planned row slots), but the right column was once uploaded flush at
    /// `text_top` — so every shortcut rode `header_gap` HIGH of its row.
    pub(in crate::render) fn secondary_top(&self) -> f32 {
        self.text_top + self.header_gap
    }

    /// SPLIT-PANE COMPOSITION — the vertical bounds `(gap_top,
    /// gap_bottom)` of the visible-BACKGROUND strip between a split Pane card's
    /// two surfaces, or `None` when there is no header to split off (the
    /// contextual spell popup, or a zero query beat). The UPPER surface owns
    /// `[card_y, gap_top]` (the title/query INPUT line); the LOWER surface owns
    /// `[gap_bottom, card_bottom]` (the facets / section-headers + candidate rows
    /// + footer). The world's own background shows through between them.
    ///
    /// THE SEAM HANGS FROM THE QUERY FIELD'S OWN BOTTOM EDGE, on every family —
    /// one rule, no arms. The upper surface holds the query INPUT and nothing
    /// else, so it closes [`BREATHE_FRAC`] of the beat below the field's box as
    /// symmetric breathing (the field then sits with a pad above it and a pad
    /// below it inside its own bar), and the visible band is
    /// [`SPLIT_GAP_FRAC`] of the beat tall. Everything below — a lens strip, the
    /// candidate rows, the footer — belongs to the lower surface. On a GROUPED
    /// card that is the same number the retired `strip.top + breathe` produced,
    /// because the strip's own top IS the field's bottom.
    ///
    /// IT IS CARVED OUT OF THE BEAT, the negative space between the query field's
    /// glyphs and the first candidate, so no glyph falls in the gap and no text
    /// moves; it is a pure FILL change. `first_top` is sacred and untouched
    /// (moving it would shift every row below).
    pub(in crate::render) fn split_bounds(&self) -> Option<(f32, f32)> {
        let field = self.headers.first()?;
        if self.header_gap <= 0.0 {
            return None;
        }
        let upper_bottom = field.bottom() + self.header_gap * BREATHE_FRAC;
        Some((
            upper_bottom,
            upper_bottom + self.header_gap * SPLIT_GAP_FRAC,
        ))
    }

    /// Invert a planned row slot without re-deriving its y arithmetic.
    pub(in crate::render) fn display_at(&self, py: f32) -> Option<usize> {
        (self.lh > 0.0).then(|| {
            self.rows
                .iter()
                .position(|r| py >= r.top && py < r.bottom())
        })?
    }

    /// A travelling band belongs to the planned row nearest its visual centre.
    pub(in crate::render) fn display_nearest(&self, py: f32) -> Option<usize> {
        self.rows
            .iter()
            .min_by(|a, b| {
                let da = (a.top + a.height * 0.5 - py).abs();
                let db = (b.top + b.height * 0.5 - py).abs();
                da.total_cmp(&db)
            })
            .map(|row| row.display)
    }
}
use super::fit_rows::fit_item_rows_after_px;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct WorkspaceRowFit {
    pub item_cap: usize,
    pub pad: f32,
    pub header_rows: usize,
    pub header_gap: f32,
    pub hint_gap_rows: usize,
}

/// Resolve a fixed-height workspace's candidate capacity after charging its
/// header, empty-state and compact teaching footer. This is planner-owned row
/// arithmetic: consumers receive the resolved item cap and header beat instead
/// of rebuilding a candidate-band origin from loose row counts.
#[allow(clippy::too_many_arguments)]
pub(in crate::render) fn fit_workspace_item_rows(
    card_h: f32,
    pad: f32,
    lh: f32,
    header_rows: usize,
    header_gap: f32,
    empty_rows: usize,
    footer_with_gap: f32,
    footer_without_gap: f32,
    footer_present: bool,
    min_items: usize,
) -> WorkspaceRowFit {
    let mut pad = pad;
    let mut header_rows = header_rows;
    let mut planned_gap = header_gap;
    let mut hint_gap_rows = usize::from(footer_present);
    let mut footer_reserve = footer_with_gap;
    let fixed = |pad: f32, header_rows: usize, header_gap: f32, footer_reserve: f32| {
        pad * if footer_present { 1.0 } else { 2.0 }
            + (header_rows.saturating_add(empty_rows)) as f32 * lh
            + header_gap
            + footer_reserve
    };

    if fixed(pad, header_rows, planned_gap, footer_reserve) > card_h {
        planned_gap = 0.0;
    }
    while header_rows > 0 && fixed(pad, header_rows, planned_gap, footer_reserve) > card_h {
        header_rows -= 1;
    }
    if footer_present && fixed(pad, header_rows, planned_gap, footer_reserve) > card_h {
        hint_gap_rows = 0;
        footer_reserve = footer_without_gap;
    }
    let non_pad = fixed(0.0, header_rows, planned_gap, footer_reserve);
    let pad_cost = if footer_present { pad } else { 2.0 * pad };
    if non_pad + pad_cost > card_h {
        // The footer is attached BELOW the content stack, so only the TOP pad
        // can yield here. Keeping the symmetric half would reserve a bottom pad
        // the shaper never spends and seat the teaching line past the card.
        let divisor = if footer_present { 1.0 } else { 2.0 };
        pad = ((card_h - non_pad).max(0.0) / divisor).min(pad);
    }

    let avail_px = (card_h - 2.0 * pad).max(0.0);
    let reserved_px =
        (header_rows.saturating_add(empty_rows)) as f32 * lh + planned_gap + footer_reserve;
    WorkspaceRowFit {
        item_cap: fit_item_rows_after_px(avail_px, lh, reserved_px, min_items),
        pad,
        header_rows,
        header_gap: planned_gap,
        hint_gap_rows,
    }
}
