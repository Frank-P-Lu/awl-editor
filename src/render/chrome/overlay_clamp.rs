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
    /// per-kind cap (`overlay_window_rows`). Without the cap the flat family
    /// has no such bound at all: a big flat corpus (the theme picker's own
    /// world roster, `overlay_window_rows() == theme::THEMES.len()`) draws a
    /// card taller than the canvas — measured `card_h: 934` against
    /// `canvas_h: 800`.
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
        self.metrics.px(super::CARD_MARGIN)
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

    /// Resolve a workspace's candidate window after reserving its fixed chrome
    /// in pixels. Unlike a floating flat card, a workspace has a fixed card
    /// height and a compact teaching footer whose two lines are not full row
    /// pitches. The footer is navigation, so it is reserved first and the
    /// candidate band may become empty at the enforced minimum geometry.
    pub(super) fn overlay_workspace_window(
        &self,
        n_items: usize,
        item_cap: usize,
    ) -> (usize, usize) {
        let item_cap = self.overlay_window_rows.max(1).min(item_cap);
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

/// **THE COUNT CUE'S BUDGET FIXED POINT**, shared by every geometry family
/// (flat, grouped, workspace).
///
/// **THE RESERVATION IS SYMMETRIC AND SCROLL-INVARIANT — NEVER "HOWEVER MANY
/// EDGES CLIP RIGHT NOW".** `scroll_window`'s own `visible` (`resolve`'s
/// second return) already ignores `top`/the selection — it is a pure function
/// of the corpus size and the window's byte budget — so "is this
/// corpus/canvas/query combination windowed AT ALL" is itself scroll-
/// invariant, decided once by `resolve(0)`. Whether the CURRENTLY VISIBLE
/// slice happens to clip above, below, both or neither is NOT: scrolling from
/// the top (clips only below) to the middle (clips both) of the SAME windowed
/// corpus changes which edges are non-empty every frame. An earlier cut of
/// this function charged `resolve` exactly as many extra rows as the
/// CURRENTLY clipped edges (0, 1 or 2) — so the reserved overhead, and with it
/// `first_top`/`card_h`/every real row's own Y, changed shape as the user
/// scrolled through nothing but the SAME already-open card: a scroll that
/// crossed from "only below clips" to "both clip" grew the card under the
/// reader's cursor (`render/tests/palette_scroll_anchor.rs`'s own law, which
/// exists for exactly this class of defect on an unrelated composition and
/// caught this one immediately). So: once `resolve(0)` shows the corpus is
/// windowed at all, BOTH edges' rows are reserved unconditionally
/// (`resolve(2)`) — worst case, but the ONLY value that cannot change with
/// scroll — and the actual per-edge CONTENT (whether a slot draws text or sits
/// blank this frame) is read off THAT fixed window via [`window_edge_counts`],
/// never fed back into the reservation itself.
///
/// `resolve` closes over whatever the family's own fit function needs
/// (`avail_px`, `lh`, the family's fixed chrome, its `min_items` floor) and
/// maps "how many EXTRA overhead rows to reserve for the cue" to `(top,
/// visible)` — so this stays ignorant of `fit_item_rows` /
/// `fit_sectioned_item_rows` / `fit_workspace_item_rows` and cannot drift
/// from whichever one a family actually reads.
///
/// **THE CUE MAY NEVER BE THE THING THAT EMPTIES THE WINDOW.** Reserving the
/// two rows can push a family at its own `min_items: 0` floor (the grouped
/// card's own starvation arm) to answer with zero items — so a corpus that
/// comfortably showed a row without the cue could lose its LAST row to the
/// cue's own reserved lines. That trade is never worth making: a passive
/// position cue is not worth an empty candidate band. So when the reserved
/// pass's `visible` drops to zero while the naive (uncued) pass had at least
/// one, this returns the NAIVE window with no cue reservation at all — the
/// corpus still shows what little of itself fits, silently, rather than
/// nothing plus an announcement that there was more.
///
/// Returns `(top, visible, cue_above, cue_below, reserved)` — `reserved` is
/// the fixed row COUNT (`0` or `2`) every caller charges to its own
/// `total_rows`/`first_top` math, kept deliberately separate from
/// `cue_above`/`cue_below` (`Option<usize>`, the per-edge CONTENT for THIS
/// frame's scroll position) so a caller can never accidentally derive the
/// scroll-varying content count back into the reservation.
pub(super) fn resolve_window_and_cue(
    n_items: usize,
    resolve: impl Fn(usize) -> (usize, usize),
) -> (usize, usize, Option<usize>, Option<usize>, usize) {
    let (top0, visible0) = resolve(0);
    if visible0 >= n_items {
        return (top0, visible0, None, None, 0);
    }
    let (top, visible) = resolve(2);
    if visible == 0 {
        return (top0, visible0, None, None, 0);
    }
    let (above, below) = window_edge_counts(top, visible, n_items);
    (top, visible, above, below, 2)
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
    use super::{PlanLine, resolve_window_and_cue, window_plan};

    /// **THE RESERVATION'S OWN BOUNDARY.** A reported gap at the head of an
    /// UNCLAMPED theme-picker list (every world in the roster shown at once)
    /// was hypothesized as
    /// "a reserved marker row that stays reserved even when nothing is
    /// clamped." That hypothesis is FALSE for shipped code — the head gap is
    /// entirely `OVERLAY_QUERY_BEAT`'s deliberate query divider, not a stray
    /// reservation — but proving that took a fixture the existing
    /// `render/tests/edge_count_cue_law.rs` roster sweep does not
    /// carry: its own tall-fits cell caps every kind's corpus at
    /// `window_rows().clamp(1, 3)`, so for the theme picker
    /// (`window_rows() == 20`) it exercises `n_items = 3` — miles from the
    /// cap, where two reserved-but-unneeded rows have so much slack neither
    /// edge ever clips and its own assertion (`(above, below) == (None,
    /// None)`) stays green whether or not `reserved` itself is nonzero. It
    /// checks the cue's TEXT, never the RESERVATION.
    ///
    /// This law reads `reserved` directly — the one field the other law
    /// never inspects — swept across the exact boundary a small fixture
    /// cannot reach: `n_items` from just under to just over each `cap`,
    /// including `n_items == cap` (the theme picker's own shipped shape,
    /// `n_items == window_rows()`).
    ///
    /// MUTATION-PROVEN: deleting `resolve_window_and_cue`'s `visible0 >=
    /// n_items` early return (so every call falls through to the
    /// `resolve(2)` branch and unconditionally reserves) failed this law
    /// immediately at `cap=1, n_items=1` — while `edge_count_cue_law.rs`'s
    /// own tall-fits cells stayed green throughout, because their
    /// `n_items=3` fixture never
    /// reaches a cap tight enough for the always-on reservation to displace a
    /// real item or fail its `(None, None)` check.
    #[test]
    fn reservation_never_fires_when_the_corpus_fits_at_or_under_the_cap() {
        for cap0 in [1usize, 2, 3, 5, 8, 20, 41] {
            for n_items in cap0.saturating_sub(2)..=(cap0 + 2) {
                let resolve = |extra: usize| {
                    let item_cap = cap0.saturating_sub(extra).max(1);
                    (0, n_items.min(item_cap))
                };
                let (top, visible, above, below, reserved) =
                    resolve_window_and_cue(n_items, resolve);
                if n_items <= cap0 {
                    assert_eq!(
                        reserved, 0,
                        "cap0={cap0} n_items={n_items}: the corpus fits without any \
                         help from the reservation, so no row may be reserved for it \
                         (got top={top} visible={visible} above={above:?} below={below:?})"
                    );
                    assert_eq!(
                        (above, below),
                        (None, None),
                        "cap0={cap0} n_items={n_items}: a corpus that fits shows no edge cue"
                    );
                    assert_eq!(
                        visible, n_items,
                        "cap0={cap0} n_items={n_items}: a corpus that fits shows every item"
                    );
                }
            }
        }
    }

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
