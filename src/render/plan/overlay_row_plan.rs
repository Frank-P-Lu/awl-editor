//! Queries and inverses over one planned overlay row band.

use super::overlay_rows::{FACETED_BREATHE_FRAC, SPLIT_GAP_FRAC};
use super::{OverlayRowPlan, PlannedHeader, PlannedRow};

impl OverlayRowPlan {
    pub(in crate::render) fn rows(&self) -> &[PlannedRow] {
        &self.rows
    }
    pub(in crate::render) fn row_top(&self, display: usize) -> Option<f32> {
        self.rows.get(display).map(|r| r.top)
    }
    pub(in crate::render) fn item_at(&self, display: usize) -> Option<usize> {
        self.rows.get(display).and_then(|r| r.item)
    }
    pub(in crate::render) fn candidate_rows(&self) -> usize {
        self.rows.len()
    }
    pub(in crate::render) fn content_rows(&self) -> usize {
        self.rows.len() + self.empty_rows
    }
    pub(in crate::render) fn first_top(&self) -> f32 {
        self.first_top
    }
    pub(in crate::render) fn lh(&self) -> f32 {
        self.lh
    }
    pub(in crate::render) fn band_bottom(&self) -> f32 {
        self.first_top + self.rows.len() as f32 * self.lh
    }
    pub(in crate::render) fn footer_top(&self) -> f32 {
        self.first_top + self.content_rows() as f32 * self.lh
    }
    pub(in crate::render) fn selected_display(&self) -> Option<usize> {
        self.selected_display
    }

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

    /// SPLIT-PANE COMPOSITION (item 50) — the vertical bounds `(gap_top,
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
    ///     beat as symmetric breathing below the query box (item 83).
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

    pub(in crate::render) fn row_dx(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dx)
    }
    pub(in crate::render) fn row_dw(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dw)
    }

    /// The pointer inverse reads the exact horizontal span the planned row draws.
    pub(in crate::render) fn row_at(&self, px: f32, py: f32) -> Option<usize> {
        let row = self.rows.get(self.display_at(py)?)?;
        (px >= self.card_x + row.dx && px <= self.card_x + self.card_w + row.dw)
            .then_some(row.item)
            .flatten()
    }

    #[cfg(test)]
    pub(in crate::render) fn card_x_span(&self) -> (f32, f32) {
        (self.card_x, self.card_x + self.card_w)
    }
}
