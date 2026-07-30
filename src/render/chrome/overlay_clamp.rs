//! ITEM 181 — the ONE height-clamp owner every candidate-row window (flat,
//! grouped, and the spell popup) routes through: the per-kind row count
//! (`overlay_window_rows`) further bounded to what the card's own available
//! pixels actually fit. Split out of `overlay.rs` (a grandfathered file at its
//! own code-health high-water mark) so this round adds a new, small,
//! unmarked file rather than growing an already-at-its-ceiling one.

use super::*;

impl TextPipeline {
    /// Caps a candidate window to what `avail_px` fits at row pitch `lh`,
    /// given `overhead_rows` non-item display lines — never above the
    /// per-kind cap (`overlay_window_rows`). Before this existed the flat
    /// family had no such bound at all: a big flat corpus (the theme picker's
    /// own world roster, `overlay_window_rows() == theme::THEMES.len()`) drew
    /// a card taller than the canvas (`card_h: 934` against `canvas_h: 800`).
    pub(super) fn overlay_item_cap(&self, avail_px: f32, lh: f32, overhead_rows: usize) -> usize {
        self.overlay_window_rows
            .max(1)
            .min(fit_item_rows(avail_px, lh, overhead_rows))
    }

    /// Resolves the FLAT family's item window in one call: caps it to what
    /// the canvas fits before sliding it through the shared `scroll_window`
    /// owner. `overlay_geometry`'s flat path and the spell popup each read
    /// this with their own `avail_px` (the popup's is whichever side of the
    /// anchoring word is roomier, not a fixed card_y/margin budget).
    pub(super) fn overlay_flat_window(
        &self,
        n_items: usize,
        avail_px: f32,
        overhead_rows: usize,
    ) -> (usize, usize) {
        let item_cap = self.overlay_item_cap(avail_px, self.overlay_lh(), overhead_rows);
        scroll_window(
            n_items,
            self.overlay_selected,
            self.overlay_scroll,
            item_cap,
        )
    }
}
