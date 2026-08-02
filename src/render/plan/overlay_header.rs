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
/// `lh + header_gap·(1 - 2·frac)` — comfortably true for every body face at 0.4.
pub(super) const SPLIT_GAP_FRAC: f32 = 0.4;

/// A GROUPED card's upper surface borrows this fraction of the SAME
/// already-proven-safe slack as symmetric breathing room below the query box
/// before the visible gap starts, so the query stops reading bottom-heavy inside
/// its own small strip. The FLAT arm is already at its ceiling (its gap sits
/// flush against the first candidate row) and takes no breathe.
pub(super) const FACETED_BREATHE_FRAC: f32 = 0.2;

/// **THE HEADER BAND'S OWN RUN** — the distance from a card's `text_top` down to
/// its candidate band's `first_top`, for a card with `header_rows` display lines
/// above the band at pitch `lh` and a query beat of `header_gap`.
///
/// ONE owner, because this number is asked for from three unrelated places and
/// used to be re-summed at each: the row planner's forward y arithmetic
/// (`row_top`), the header boxes below (whose LAST box closes exactly on it), and
/// — item 116d — a summoned workspace's relocated document viewport, which opens
/// at the same line the candidate band does. A workspace whose lens moved into
/// its header carries TWO header lines, so a consumer that re-summed `lh +
/// header_gap` for itself would seat the comparison a whole line high the moment
/// that second line appeared.
///
/// `header_rows == 0` (the contextual spell popup) still carries its own zero
/// beat, so this stays the plain offset in every family.
pub(in crate::render) fn header_band_height(header_rows: usize, lh: f32, header_gap: f32) -> f32 {
    header_rows as f32 * lh + header_gap
}

/// Lay out one box per header display line, stacked from `text_top` at the
/// candidate band's own pitch, with the query BEAT folded into the LAST box
/// exactly as the shaper folds it into that line's own glyph metrics — so the
/// band closes on `first_top` and the query field, the lens strip and the first
/// candidate row cannot disagree about where one ends and the next begins.
///
/// The last box's height is whatever [`header_band_height`] has left over rather
/// than a second `lh + header_gap`, so the boxes and the band's run are one
/// derivation.
pub(super) fn plan_header_band(input: &OverlayRowPlanInput<'_>) -> Vec<PlannedHeader> {
    let n = input.header_rows;
    let band_bottom = input.text_top + header_band_height(n, input.lh, input.header_gap);
    (0..n)
        .map(|line| {
            let top = input.text_top + line as f32 * input.lh;
            PlannedHeader {
                line,
                top,
                height: match line + 1 == n {
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
/// THE BEAT LIVES IN THE LAST HEADER LINE'S BOX, not between the lines. The
/// query BEAT (`header_gap`, `overlay_header_gap`'s calm slab of negative space
/// before the first candidate) is STRUCTURAL: the shaper inflates the LAST
/// header line's real glyph metrics by exactly it (`shape_overlay_names`'s
/// `header_lh` on a flat card's line 0; `shape_theme_spans`'s `strip_lh` on a
/// grouped card's line 1) rather than emitting a blank line. So header line `i`
/// is `lh` tall except the last, which is `lh + header_gap` — and the last
/// line's own BOTTOM is the candidate band's `first_top`, by construction.
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
    // The three answers that used to be three separate owners in
    // `render/chrome` — the query field's box, the lens strip's box, and the
    // split-pane gap carved between the two surfaces — are one planned object
    // read three ways. `overlay_secondary_top`, `overlay_split_bounds`,
    // `overlay_strip_band` and `overlay_query_center` are GONE from
    // `render/chrome`; a consumer cannot re-derive a header line's position,
    // only read it off the plan the pixels came from.

    #[cfg(test)]
    pub(in crate::render) fn header_lines(&self) -> &[PlannedHeader] {
        &self.headers
    }

    /// How many display lines the card draws ABOVE its candidate band. Paired
    /// with [`Self::secondary_top`]: a right-column buffer leads with exactly
    /// this many empty lines, and the two must come from one object or the
    /// column's origin and its own leading can disagree.
    pub(in crate::render) fn header_rows(&self) -> usize {
        self.headers.len()
    }

    /// THE QUERY FIELD's own line box — where its glyphs are half-led, where its
    /// caret is centred, and the band a pointer must be inside to be "on the
    /// field". `None` for the contextual spell popup, which has no query line at
    /// all (`header_rows == 0`).
    ///
    /// On a FLAT picker this box is `lh + header_gap` tall: the query BEAT is the
    /// bottom of the field's own box, not a separate row. That is why reading the
    /// bare `lh` here was a real drift — see `over_overlay_query`.
    pub(in crate::render) fn query_band(&self) -> Option<PlannedHeader> {
        self.headers.first().copied()
    }

    /// THE LENS STRIP's own line box on a GROUPED card — the last header line,
    /// which is the one the beat inflates. `None` on a flat picker or the spell
    /// popup (no strip is drawn), which is exactly when `geom.theme` is false.
    pub(in crate::render) fn strip_band(&self) -> Option<PlannedHeader> {
        (self.headers.len() >= 2).then(|| self.headers[self.headers.len() - 1])
    }

    /// The device-px TOP a uniform-line-height RIGHT-COLUMN buffer must be
    /// uploaded at so its chord/time labels — which lead with `header_rows` empty
    /// lines — land EXACTLY on this plan's candidate band. The two share ONE
    /// y-origin by the invariant `secondary_top() + (header_rows + r) * lh ==
    /// row_top(r)`; the leading empties supply `header_rows * lh` and this
    /// supplies the beat.
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
    /// BOTH ARMS ARE CARVED OUT OF THE LAST HEADER LINE'S OWN BOX — the negative
    /// space of the query beat — so no glyph falls in the gap and no text moves;
    /// it is a pure FILL change. They differ only in which EDGE of that box the
    /// band hangs from:
    ///
    ///   * FLAT (one header line): the beat is the BOTTOM of the query box, so
    ///     the band sits flush against the box's bottom edge — which is the first
    ///     candidate row's top, and sacred (moving it would shift every row
    ///     below).
    ///   * GROUPED (query line + lens strip): the beat is the BOTTOM of the STRIP
    ///     box, but the surface seam belongs above the strip, so the band hangs
    ///     from the strip box's TOP edge — plus [`FACETED_BREATHE_FRAC`] of the
    ///     beat as symmetric breathing below the query box.
    pub(in crate::render) fn split_bounds(&self) -> Option<(f32, f32)> {
        let head = self.headers.last()?;
        if self.header_gap <= 0.0 {
            return None;
        }
        let gap = self.header_gap * SPLIT_GAP_FRAC;
        if self.headers.len() == 1 {
            let lower_top = head.bottom();
            Some((lower_top - gap, lower_top))
        } else {
            let upper_bottom = head.top + self.header_gap * FACETED_BREATHE_FRAC;
            Some((upper_bottom, upper_bottom + gap))
        }
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
