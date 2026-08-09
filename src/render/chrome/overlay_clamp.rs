//! The ONE height-clamp owner every candidate-row window (flat,
//! grouped, and the spell popup) routes through: the per-kind row count
//! (`overlay_window_rows`) further bounded to what the card's own available
//! pixels actually fit. Split out of `overlay.rs` (a grandfathered file at its
//! own code-health high-water mark) so this round adds a new, small,
//! unmarked file rather than growing an already-at-its-ceiling one. The shared
//! footer helper lives here for the same reason — `overlay.rs`
//! and `theme_picker.rs` are both already at their own ceiling.

use super::*;

impl TextPipeline {
    /// Caps a candidate window to what `avail_px` fits at row pitch `lh`,
    /// given `overhead_rows` non-item display lines — never above the
    /// per-kind cap (`overlay_window_rows`). Before this existed the flat
    /// family had no such bound at all: a big flat corpus (the theme picker's
    /// own world roster, `overlay_window_rows() == theme::THEMES.len()`) drew
    /// a card taller than the canvas (`card_h: 934` against `canvas_h: 800`).
    ///
    /// `min_items` is `fit_item_rows`'s own family floor — see its doc for why
    /// the FLAT family and the GROUPED family pass different
    /// values here.
    pub(super) fn overlay_item_cap(
        &self,
        avail_px: f32,
        lh: f32,
        overhead_rows: usize,
        min_items: usize,
    ) -> usize {
        self.overlay_window_rows
            .max(1)
            .min(fit_item_rows(avail_px, lh, overhead_rows, min_items))
    }

    /// TEST-ONLY: the bottom MARGIN a card contractually leaves below itself.
    /// That strip is canvas the card may never spend, so a law asking whether
    /// an EMPTY candidate band was forced by the canvas must not count it as
    /// free room — the two rows such a band is missing have to fit above it.
    #[cfg(test)]
    pub(in crate::render) fn overlay_card_margin(&self) -> f32 {
        self.metrics.px(super::overlay::CARD_MARGIN)
    }

    /// The GROUPED family's counterpart, routed through
    /// [`fit_sectioned_item_rows`] so its own section headers are charged for
    /// the window that will actually be drawn rather than for every section in
    /// the list. Same per-kind ceiling, same family floor; the only difference
    /// is which header count the budget pays for.
    pub(super) fn overlay_sectioned_item_cap(
        &self,
        avail_px: f32,
        lh: f32,
        chrome_rows: usize,
        total_headers: usize,
        min_items: usize,
    ) -> usize {
        self.overlay_window_rows.max(1).min(fit_sectioned_item_rows(
            avail_px,
            lh,
            chrome_rows,
            total_headers,
            min_items,
        ))
    }

    /// Resolves the FLAT family's item window in one call: caps it to what
    /// the canvas fits before sliding it through the shared `scroll_window`
    /// owner. `overlay_geometry`'s flat path and the spell popup each read
    /// this with their own `avail_px` (the popup's is whichever side of the
    /// anchoring word is roomier, not a fixed card_y/margin budget). Both
    /// pass `min_items: 1` — a flat/contextual card's own fixed overhead
    /// (one query line, no lens strip) never grows past what a real canvas
    /// holds, so it always attempts to show its own selection.
    pub(super) fn overlay_flat_window(
        &self,
        n_items: usize,
        avail_px: f32,
        overhead_rows: usize,
    ) -> (usize, usize) {
        let item_cap = self.overlay_item_cap(avail_px, self.overlay_lh(), overhead_rows, 1);
        scroll_window(
            n_items,
            self.overlay_selected,
            self.overlay_scroll,
            item_cap,
        )
    }

    /// The FOOTER band (the keybindings tips) as `(lines, display
    /// rows)`, the ONE owner both geometry families read so a footer can
    /// never be sized against `avail_px` in one family and left out of the
    /// other (the grouped family used to omit it entirely). `keybindings_tips`
    /// only ever carries content while the Keybindings overlay is open (a
    /// FLAT kind, `app/stats.rs::sync_discoverability`), so this is empty for
    /// every GROUPED kind today — wiring it through the shared owner anyway
    /// means a future grouped footer is counted for free rather than
    /// silently reopening this item's exact defect.
    pub(in crate::render) fn overlay_footer_lines(&self) -> (Vec<String>, usize) {
        let footer = self.keybindings_tips.clone();
        let footer_rows = if footer.is_empty() {
            0
        } else {
            footer.len() + 1
        };
        (footer, footer_rows)
    }
}

/// Slice a full display plan (headers + item rows, from [`TextPipeline::theme_plan`]) to
/// the ITEM window `[lo, hi)`: keep every `Item(i)` with `lo ≤ i < hi`, and re-hang the
/// SECTION HEADER above the first surviving item of each section (a header whose whole
/// section fell outside the window is dropped). Items in the window form a contiguous run
/// in the plan (the plan is built in item-index order), so one forward pass — carrying
/// the most-recent header until an in-window item consumes it — yields the correct
/// header→rows grouping for the visible slice. A window at the start of a section shows
/// that section's header at the top (`a section header at the window top is fine`).
pub(super) fn window_plan(full: &[PlanLine], lo: usize, hi: usize) -> Vec<PlanLine> {
    let mut out: Vec<PlanLine> = Vec::new();
    let mut pending: Option<&PlanLine> = None;
    for line in full {
        match line {
            PlanLine::Location(_) | PlanLine::Header(_) => pending = Some(line),
            PlanLine::Item(i) => {
                if *i >= lo && *i < hi {
                    if let Some(h) = pending.take() {
                        out.push(h.clone());
                    }
                    out.push(line.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{PlanLine, window_plan};

    fn sample_plan() -> Vec<PlanLine> {
        vec![
            PlanLine::Header("A".into()),
            PlanLine::Item(0),
            PlanLine::Item(1),
            PlanLine::Item(2),
            PlanLine::Header("B".into()),
            PlanLine::Item(3),
            PlanLine::Item(4),
        ]
    }

    fn shape(plan: &[PlanLine]) -> Vec<String> {
        plan.iter()
            .map(|l| match l {
                PlanLine::Location(l) => format!("@{l}"),
                PlanLine::Header(h) => format!("#{h}"),
                PlanLine::Item(i) => format!("i{i}"),
            })
            .collect()
    }

    #[test]
    fn window_plan_returns_the_full_plan_when_it_fits() {
        assert_eq!(
            shape(&window_plan(&sample_plan(), 0, 5)),
            shape(&sample_plan())
        );
    }

    #[test]
    fn window_plan_keeps_only_touched_sections_headers() {
        assert_eq!(
            shape(&window_plan(&sample_plan(), 2, 4)),
            vec!["#A", "i2", "#B", "i3"]
        );
        assert_eq!(
            shape(&window_plan(&sample_plan(), 3, 5)),
            vec!["#B", "i3", "i4"]
        );
    }

    #[test]
    fn window_plan_header_at_window_top_and_no_duplicates() {
        assert_eq!(
            shape(&window_plan(&sample_plan(), 1, 3)),
            vec!["#A", "i1", "i2"]
        );
    }

    #[test]
    fn window_plan_empty_range_is_empty() {
        assert!(window_plan(&sample_plan(), 9, 9).is_empty());
        assert!(window_plan(&sample_plan(), 5, 5).is_empty());
    }
}
