//! Plan-level geometry laws — no device, no shaper, no theme.
//!
//! These are the laws item 174 exists to make possible: presentation decisions
//! asserted directly against the planner's own output rather than inferred from
//! pixels. The device-level companions (drawn ↔ hit-test ↔ sidecar identity over
//! the real pipeline and the real world roster) live in
//! `render/tests/overlay_plan_law.rs`.

use super::overlay_rows::{OverlayRowPlanInput, PlanLine, fit_item_rows, plan_overlay_rows};

const CARD_X: f32 = 420.0;
const CARD_W: f32 = 360.0;
const TEXT_TOP: f32 = 64.0;
const LH: f32 = 24.0;

fn flat(
    visible: usize,
    top_idx: usize,
    n_items: usize,
    header_rows: usize,
) -> OverlayRowPlanInput<'static> {
    OverlayRowPlanInput {
        card_x: CARD_X,
        card_w: CARD_W,
        text_top: TEXT_TOP,
        lh: LH,
        header_gap: 0.0,
        header_rows,
        visible,
        top_idx,
        n_items,
        selected: 0,
        empty_rows: 0,
        lines: None,
        dx_per_row: 0.0,
        cluster_span: None,
    }
}

fn hit(px: f32, py: f32, visible: usize, top_idx: usize, n: usize) -> Option<usize> {
    plan_overlay_rows(&flat(visible, top_idx, n, 1)).row_at(px, py)
}

fn hit_spell(px: f32, py: f32, visible: usize, top_idx: usize, n: usize) -> Option<usize> {
    plan_overlay_rows(&flat(visible, top_idx, n, 0)).row_at(px, py)
}

#[test]
fn pointer_maps_to_the_row_under_it() {
    assert_eq!(hit(500.0, 88.0, 5, 2, 8), Some(2)); // top of row 0
    assert_eq!(hit(500.0, 100.0, 5, 2, 8), Some(2)); // mid row 0
    assert_eq!(hit(500.0, 112.0, 5, 2, 8), Some(3)); // row 1
    assert_eq!(hit(500.0, 200.0, 5, 2, 8), Some(6));
}

#[test]
fn query_row_and_above_are_not_rows() {
    assert_eq!(hit(500.0, 70.0, 5, 2, 8), None);
    assert_eq!(hit(500.0, 0.0, 5, 2, 8), None);
}

#[test]
fn below_the_last_visible_row_is_none() {
    assert_eq!(hit(500.0, 210.0, 5, 2, 8), None);
}

#[test]
fn off_the_card_horizontally_is_none() {
    assert_eq!(hit(419.0, 100.0, 5, 2, 8), None); // left of card
    assert_eq!(hit(781.0, 100.0, 5, 2, 8), None); // right of card
    assert_eq!(hit(420.0, 100.0, 5, 2, 8), Some(2));
    assert_eq!(hit(780.0, 100.0, 5, 2, 8), Some(2));
}

#[test]
fn empty_list_never_hits() {
    assert_eq!(hit(500.0, 100.0, 0, 0, 0), None);
}

#[test]
fn spell_panel_rows_start_at_the_top_no_query_line() {
    assert_eq!(hit_spell(500.0, 64.0, 4, 0, 4), Some(0)); // top of row 0
    assert_eq!(hit_spell(500.0, 70.0, 4, 0, 4), Some(0)); // still row 0
    assert_eq!(hit_spell(500.0, 88.0, 4, 0, 4), Some(1)); // row 1
    assert_eq!(hit_spell(500.0, 63.0, 4, 0, 4), None); // above the panel text
}

#[test]
fn a_visible_row_past_the_corpus_end_clamps_to_none() {
    assert_eq!(hit(500.0, 88.0, 5, 2, 5), Some(2)); // vis 0 -> idx 2
    assert_eq!(hit(500.0, 150.0, 5, 2, 5), Some(4)); // vis 2 -> idx 4 (last valid)
    assert_eq!(hit(500.0, 160.0, 5, 2, 5), None); // vis 3 -> idx 5 >= 5
}

/// THE HEADLINE PLAN LAW: the drawn slot and the interaction geometry are the
/// same object, over every header count, every header gap, and every row — not a
/// forward formula checked against a hand-written inverse.
///
/// The swept axis is the one the pre-plan code got away with: the two directions
/// used to be separate free functions taking five loose scalars each, and every
/// call site re-assembled the arguments itself.
#[test]
fn every_planned_row_slot_hit_tests_back_to_its_own_row() {
    for &header_rows in &[0usize, 1, 2, 3] {
        for &gap in &[0.0f32, 5.0, 13.0, 63.55] {
            for &lh in &[1.0f32, 12.0, 24.0, 41.0, 99.5] {
                for &visible in &[1usize, 2, 5, 12] {
                    for &top_idx in &[0usize, 3, 40] {
                        let mut input = flat(visible, top_idx, top_idx + visible, header_rows);
                        input.header_gap = gap;
                        input.lh = lh;
                        let plan = plan_overlay_rows(&input);
                        assert_eq!(plan.candidate_rows(), visible);
                        for row in plan.rows() {
                            let item = row.item.expect("a full window carries every item");
                            // Every y strictly inside the drawn slot resolves to
                            // this row's own item, at both card edges.
                            for frac in [0.0f32, 0.001, 0.5, 0.999] {
                                let y = row.top + frac * row.height;
                                for x in [CARD_X, CARD_X + CARD_W * 0.5, CARD_X + CARD_W] {
                                    assert_eq!(
                                        plan.row_at(x, y),
                                        Some(item),
                                        "header_rows {header_rows} gap {gap} lh {lh} \
                                         visible {visible} top_idx {top_idx}: display \
                                         {} slot [{}, {}) must hit-test to item {item} \
                                         at ({x}, {y})",
                                        row.display,
                                        row.top,
                                        row.bottom(),
                                    );
                                }
                            }
                            // And the slots tile the band with no gap and no overlap.
                            assert_eq!(plan.row_top(row.display), Some(row.top));
                            if row.display > 0 {
                                assert_eq!(plan.rows()[row.display - 1].bottom(), row.top);
                            } else {
                                assert_eq!(row.top, plan.first_top());
                            }
                        }
                        assert_eq!(plan.band_bottom(), plan.first_top() + visible as f32 * lh);
                    }
                }
            }
        }
    }
}

/// A GROUPED plan's section headers push item rows down and reject a click. The
/// axis: every header POSITION in the sequence, including leading, interior,
/// trailing and consecutive headers.
#[test]
fn grouped_plan_headers_hold_a_row_slot_and_accept_no_click() {
    let shapes: &[&[Option<usize>]] = &[
        &[None, Some(0), Some(1)],
        &[Some(0), None, Some(1)],
        &[Some(0), Some(1), None],
        &[None, None, Some(0)],
        &[None, Some(0), None, Some(1), None],
    ];
    for shape in shapes {
        let lines: Vec<PlanLine> = shape
            .iter()
            .map(|s| match s {
                Some(i) => PlanLine::Item(*i),
                None => PlanLine::Header("SECTION".into()),
            })
            .collect();
        let n_items = shape.iter().filter(|s| s.is_some()).count();
        let mut input = flat(0, 0, n_items, 2);
        input.lines = Some(&lines);
        let plan = plan_overlay_rows(&input);
        assert_eq!(plan.candidate_rows(), shape.len(), "{shape:?}");
        for (k, want) in shape.iter().enumerate() {
            let row = plan.rows()[k];
            assert_eq!(row.item, *want, "{shape:?} display {k}");
            let mid = row.top + row.height * 0.5;
            assert_eq!(
                plan.row_at(CARD_X + 1.0, mid),
                *want,
                "{shape:?} display {k}: a header must accept no click and an item row must"
            );
        }
    }
}

/// The selected display line: a grouped plan's headers push it down; a flat
/// window's is its window offset, and a transient list SHRINK (selection past
/// the window) clamps instead of overflowing.
#[test]
fn selected_display_line_tracks_both_families_and_clamps_defensively() {
    // Flat: the window offset.
    let mut input = flat(5, 10, 40, 1);
    input.selected = 13;
    assert_eq!(plan_overlay_rows(&input).selected_display(), Some(3));
    // Flat, selection BEHIND the window (a stale scroll hint): saturates to 0.
    input.selected = 4;
    assert_eq!(plan_overlay_rows(&input).selected_display(), Some(0));
    // Flat, selection PAST the window (a transient list shrink): clamps to the
    // last visible row rather than pointing off the plan.
    input.selected = 99;
    assert_eq!(plan_overlay_rows(&input).selected_display(), Some(4));
    // No items at all: no selected line.
    let empty = flat(0, 0, 0, 1);
    assert_eq!(plan_overlay_rows(&empty).selected_display(), None);
    // Grouped: two headers above item 2 push it to display 4.
    let lines = vec![
        PlanLine::Header("A".into()),
        PlanLine::Item(0),
        PlanLine::Item(1),
        PlanLine::Header("B".into()),
        PlanLine::Item(2),
    ];
    let mut g = flat(0, 0, 3, 2);
    g.lines = Some(&lines);
    g.selected = 2;
    let plan = plan_overlay_rows(&g);
    assert_eq!(plan.selected_display(), Some(4));
    assert_eq!(plan.item_at(4), Some(2));
}

/// THE FOOTER SEAM — one owner of "how many display lines precede the footer".
/// The empty-state NOTICE occupies a content line, so a card showing one starts
/// its footer a row lower. Both families answer identically; before the plan the
/// grouped family omitted the notice and drew its footer plate over it.
#[test]
fn the_empty_state_notice_occupies_a_content_line_in_both_families() {
    // Flat, no rows, one notice.
    let mut f = flat(0, 0, 0, 1);
    f.empty_rows = 1;
    let plan = plan_overlay_rows(&f);
    assert_eq!(plan.candidate_rows(), 0);
    assert_eq!(plan.content_rows(), 1);
    assert_eq!(plan.footer_top(), plan.first_top() + LH);

    // Grouped, no rows, one notice — the same answer.
    let lines: Vec<PlanLine> = Vec::new();
    let mut g = flat(0, 0, 0, 2);
    g.empty_rows = 1;
    g.lines = Some(&lines);
    let gplan = plan_overlay_rows(&g);
    assert_eq!(gplan.candidate_rows(), 0);
    assert_eq!(gplan.content_rows(), 1);
    assert_eq!(gplan.footer_top(), gplan.first_top() + LH);

    // With rows and no notice the footer sits directly under the band.
    let full = plan_overlay_rows(&flat(7, 0, 7, 1));
    assert_eq!(full.content_rows(), 7);
    assert_eq!(full.footer_top(), full.band_bottom());
}

/// O(VISIBLE), NOT O(DOC): the plan holds one row per DISPLAY LINE the card
/// shows, never one per item in the corpus. A 200,000-row Go-to picker plans the
/// twelve rows on screen.
#[test]
fn plan_rows_are_bounded_by_the_window_not_the_corpus() {
    for &n_items in &[12usize, 1_000, 200_000] {
        for &visible in &[1usize, 12] {
            let plan = plan_overlay_rows(&flat(visible, 0, n_items, 1));
            assert_eq!(
                plan.rows().len(),
                visible,
                "a {n_items}-item picker showing {visible} rows must plan exactly \
                 {visible} rows — planning the corpus is the O(doc) frame this \
                 module exists to refuse"
            );
        }
    }
}

/// A degenerate row pitch cannot be inverted, and must not divide by zero.
#[test]
fn a_zero_row_pitch_answers_no_pointer() {
    let mut input = flat(5, 0, 5, 1);
    input.lh = 0.0;
    let plan = plan_overlay_rows(&input);
    assert_eq!(plan.display_at(plan.first_top() + 1.0), None);
    assert_eq!(plan.row_at(CARD_X + 1.0, plan.first_top() + 1.0), None);
}

// --- ITEM 181 — the height-clamp owner ---------------------------------------

/// The ordinary case: enough `avail_px` for some rows but not the whole ask,
/// after the overhead (non-item) rows are paid for. `min_items` doesn't bind
/// here for either family — both floors give the same answer.
#[test]
fn fit_item_rows_divides_the_budget_after_overhead() {
    // 10 rows of pitch 20 fit in 200px; 3 are overhead, so 7 items fit.
    assert_eq!(fit_item_rows(200.0, 20.0, 3, 1), 7);
    assert_eq!(fit_item_rows(200.0, 20.0, 3, 0), 7);
    // A partial row's pixels don't count — floor, not round.
    assert_eq!(fit_item_rows(199.9, 20.0, 3, 1), 6);
}

/// THE FLAT/SPELL FLOOR (`min_items: 1`): a card always attempts at least one
/// item row, however small `avail_px` is — this is the theme-picker
/// regression itself (item 181): a big flat corpus must lose rows to the
/// clamp, never collapse to zero and never keep every row regardless of the
/// canvas.
#[test]
fn fit_item_rows_floors_at_one_even_when_nothing_fits() {
    assert_eq!(fit_item_rows(0.0, 20.0, 0, 1), 1);
    assert_eq!(fit_item_rows(5.0, 20.0, 0, 1), 1);
    assert_eq!(fit_item_rows(20.0, 20.0, 10, 1), 1); // overhead alone exceeds the budget
}

/// THE GROUPED FLOOR (`min_items: 0`, item 184): when the fixed chrome
/// overhead ALONE already meets or exceeds what `avail_px` holds — the
/// residual item 181's own doc named "a chrome-overhead sizing question, not
/// a row-count one" — the grouped family shows an empty candidate band
/// rather than a forced row that would overrun the canvas. Wherever the
/// overhead does NOT already consume the whole budget this is unchanged from
/// the `min_items: 1` case (`saturating_sub` alone already clears 1), so
/// every already-fitting grouped picker is untouched — only the exact
/// pathological case answers differently.
#[test]
fn fit_item_rows_allows_zero_once_the_groups_own_chrome_overruns_the_budget() {
    assert_eq!(fit_item_rows(20.0, 20.0, 10, 0), 0); // overhead alone exceeds the budget
    assert_eq!(fit_item_rows(0.0, 20.0, 0, 0), 0);
    // Not vacuous the other direction either: an ordinary budget still fits
    // items even with `min_items: 0`.
    assert_eq!(fit_item_rows(200.0, 20.0, 3, 0), 7);
}

/// A degenerate row pitch cannot be divided by; the clamp still answers a
/// usable, non-panicking row count — the family's own floor, not a bare `1`.
#[test]
fn fit_item_rows_guards_a_zero_row_pitch() {
    assert_eq!(fit_item_rows(500.0, 0.0, 2, 1), 1);
    assert_eq!(fit_item_rows(500.0, 0.0, 2, 0), 0);
}

/// NON-VACUITY: the clamp genuinely BINDS below a big per-kind row cap once
/// the corpus and the canvas disagree — the shape `OverlayKind::Theme` hits
/// (`window_rows() == THEMES.len()`, 19, with no canvas awareness at all
/// before item 181: `card_h: 934` against `canvas_h: 800` at the theme
/// picker's own default geometry). A row pitch and overhead in the same
/// ballpark as the real overlay metrics, on the default 800px canvas.
#[test]
fn fit_item_rows_binds_below_a_big_per_kind_cap_once_the_canvas_cannot_hold_it() {
    let per_kind_cap = 19usize; // `OverlayKind::Theme::window_rows()` == THEMES.len()
    let avail_px = 550.0; // canvas minus card_y/margin/pad/header_gap, in the real overlay's range
    let fit = fit_item_rows(avail_px, 27.0, 3, 1); // lh, chrome_rows in the real overlay's range
    assert!(
        fit < per_kind_cap,
        "the canvas-derived fit ({fit}) must bind below the per-kind cap \
         ({per_kind_cap}) — otherwise this fixture no longer reproduces the \
         reported regression"
    );
}

// --- item 131a: the signed step splits into a two-sided extent ---------------

/// THE SIGN SPLIT, at the pure planner level (no device, no pipeline): a
/// POSITIVE `dx_per_row` plans a growing `dx` with `dw` held at exactly `0.0`
/// (Mangrove's shape — left edge steps in, right edge flush); a NEGATIVE
/// `dx_per_row` plans a growing-negative `dw` with `dx` held at exactly `0.0`
/// (Magpie's mirror — right edge steps in, left edge flush). Never both at
/// once for a single signed input, by construction — the reason one field
/// suffices for two mirrored compositions.
#[test]
fn a_positive_step_plans_dx_only_and_a_negative_step_plans_dw_only() {
    let mut mangrove = flat(4, 0, 4, 0);
    mangrove.dx_per_row = 6.0;
    let plan = plan_overlay_rows(&mangrove);
    for (i, row) in plan.rows().iter().enumerate() {
        assert_eq!(
            row.dx,
            6.0 * i as f32,
            "row {i}: dx must be the positive stair"
        );
        assert_eq!(
            row.dw, 0.0,
            "row {i}: dw must stay zero under a positive step"
        );
    }

    let mut magpie = flat(4, 0, 4, 0);
    magpie.dx_per_row = -6.0;
    let plan = plan_overlay_rows(&magpie);
    for (i, row) in plan.rows().iter().enumerate() {
        assert_eq!(
            row.dx, 0.0,
            "row {i}: dx must stay zero under a negative step"
        );
        assert_eq!(
            row.dw,
            -6.0 * i as f32,
            "row {i}: dw must be the negative stair"
        );
    }
}

/// `row_at`'s x-test against the two-sided extent, directly (no pipeline): a
/// row's clickable span is `[card_x + dx, card_x + card_w + dw]`. Proven on
/// BOTH mirror directions from one fixture — the shape that would have caught
/// a `row_at` still hard-coded to the card's own right edge (pre-131a).
#[test]
fn row_at_bounds_by_the_two_sided_extent_on_both_mirrors() {
    let mut magpie = flat(3, 0, 3, 0);
    magpie.dx_per_row = -10.0; // row 2's right edge retreats 20px
    let plan = plan_overlay_rows(&magpie);
    let (x0, x1) = plan.card_x_span();
    let mid_y = plan.row_top(2).unwrap() + plan.lh() * 0.5;
    // Row 2's own right edge is `x1 - 20`; past it is bare card, not row 2.
    assert_eq!(plan.row_at(x1 - 20.5, mid_y), Some(2));
    assert_eq!(
        plan.row_at(x1 - 19.5, mid_y),
        None,
        "a pointer inside the retreated strip (drawn nowhere) must not select row 2"
    );
    // The LEFT edge is untouched under this mirror — still clickable at x0.
    assert_eq!(plan.row_at(x0 + 0.5, mid_y), Some(2));
}

/// DETERMINISM: the planner is a pure function of its inputs — the same input
/// plans the same rows every time, and nothing about it reads a clock.
#[test]
fn planning_is_deterministic() {
    let lines = vec![
        PlanLine::Header("A".into()),
        PlanLine::Item(0),
        PlanLine::Item(1),
    ];
    let mut input = flat(0, 0, 2, 2);
    input.lines = Some(&lines);
    input.header_gap = 63.55;
    input.lh = 41.0;
    let a = plan_overlay_rows(&input);
    let b = plan_overlay_rows(&input);
    assert_eq!(a.rows(), b.rows());
    assert_eq!(a.first_top(), b.first_top());
    assert_eq!(a.selected_display(), b.selected_display());
    assert_eq!(a.card_x_span(), b.card_x_span());
}
