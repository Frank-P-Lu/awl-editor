//! Queries and inverses over one planned overlay row band.

use super::{OverlayRowPlan, PlannedRow};

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
    /// Mirrors [`super::overlay_rows::OverlayRowPlanInput::cue_above_rows`] —
    /// the SECONDARY (chord/bind) buffer's own leading-empties count adds this
    /// too, exactly as it already adds [`Self::billed_header_rows`]: the
    /// above-edge cue shifts the PRIMARY buffer's row 0 down by one line
    /// (`OverlayGeom::shaped_first_row_line`), and a right column built
    /// against the unshifted count would then bind chord `k` to name `k`'s
    /// row a line early.
    pub(in crate::render) fn cue_above_rows(&self) -> usize {
        self.cue_above_rows
    }
    /// Display lines that precede the footer: the candidate band, the
    /// empty-state notice, AND the below-edge count cue — the one owner every
    /// consumer of "how far down does content run" (the footer plate's seat,
    /// the sidecar) reads, so a cue line can't seat a footer plate on top of
    /// its own glyphs the way the empty-state notice once did before it was
    /// counted here.
    pub(in crate::render) fn content_rows(&self) -> usize {
        self.rows.len() + self.empty_rows + self.cue_below_rows
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

    pub(in crate::render) fn row_dx(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dx)
    }
    pub(in crate::render) fn row_dw(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dw)
    }

    /// **THE ONE ROW X-SPAN**, inclusive, in the same canvas-pixel space the
    /// pointer arrives in: `[card_x + dx, card_x + card_w + dw]`.
    ///
    /// It is a named accessor rather than an expression because it now has more
    /// than one reader — the pointer inverse below and the published geometry
    /// report ([`super::row_report`]) — and two spellings of the same span is the
    /// drift a staggering composition already caused once, when the draw
    /// emitters applied a row's offset and `row_at` kept testing the card's
    /// undisplaced edges.
    pub(in crate::render) fn row_x_span(&self, display: usize) -> Option<(f32, f32)> {
        let row = self.rows.get(display)?;
        Some((self.card_x + row.dx, self.card_x + self.card_w + row.dw))
    }

    /// The pointer inverse reads the exact horizontal span the planned row draws.
    pub(in crate::render) fn row_at(&self, px: f32, py: f32) -> Option<usize> {
        let display = self.display_at(py)?;
        let (x0, x1) = self.row_x_span(display)?;
        (px >= x0 && px <= x1)
            .then_some(self.rows.get(display)?.item)
            .flatten()
    }

    #[cfg(test)]
    pub(in crate::render) fn card_x_span(&self) -> (f32, f32) {
        (self.card_x, self.card_x + self.card_w)
    }
}
