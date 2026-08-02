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
        selected_offset: None,
        selected_display: None,
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

// ===== THE HEADER BAND =======================================================
//
// The query/title INPUT line and the grouped family's lens STRIP. Four owners
// used to answer questions about these two boxes from loose scalars in
// `render/chrome` — `overlay_secondary_top`, `overlay_split_bounds`,
// `overlay_strip_band` and `overlay_query_center`, now all DELETED. These are
// the pure laws that replace them; every oracle below is derived INDEPENDENTLY
// of the accessor it grades (a law that asks the same function twice is
// tautological — the lesson the first family's `livingband::covered_rows` taught).

/// A plan with `header_rows` header lines, `header_gap` of beat and two
/// candidate rows.
fn headered(header_rows: usize, header_gap: f32, lh: f32) -> super::OverlayRowPlan {
    plan_overlay_rows(&OverlayRowPlanInput {
        card_x: CARD_X,
        card_w: CARD_W,
        text_top: TEXT_TOP,
        lh,
        header_gap,
        header_rows,
        visible: 2,
        top_idx: 0,
        n_items: 2,
        selected: 0,
        empty_rows: 0,
        lines: None,
        dx_per_row: 0.0,
        cluster_span: None,
        selected_offset: None,
        selected_display: None,
    })
}

/// THE HEADER BAND TILES, WITH NO HOLE AND NO OVERLAP, FROM `text_top` DOWN TO
/// THE FIRST CANDIDATE ROW — and the query BEAT lives inside the LAST header
/// line's own box, never as a gap between boxes.
///
/// The oracle is independent: each line's expected top is accumulated by
/// SUMMING the previous heights (never by calling the plan's own `top`), and the
/// band's closure is checked against `row_top(0)`, which the row family owns.
/// Swept over every header count a picker can reach (0 spell / 1 flat + workspace
/// / 2 grouped) plus a 3 nobody ships, four beats including zero, and four row
/// pitches spanning the zoom band.
#[test]
fn the_header_band_tiles_from_text_top_to_the_first_candidate_row() {
    for header_rows in 0usize..=3 {
        for &gap in &[0.0f32, 5.0, 42.0, 84.3] {
            for &lh in &[12.0f32, 24.0, 27.2, 81.6] {
                let plan = headered(header_rows, gap, lh);
                let ctx = format!("hdr={header_rows} gap={gap} lh={lh}");
                let heads = plan.header_lines();
                assert_eq!(heads.len(), header_rows, "{ctx}: one box per header line");

                // Independent accumulation: line 0 starts at text_top, each next
                // line starts where the previous one ended.
                let mut cursor = TEXT_TOP;
                for (i, head) in heads.iter().enumerate() {
                    assert_eq!(head.line, i, "{ctx}: header line {i} names itself");
                    assert!(
                        (head.top - cursor).abs() < 1e-3,
                        "{ctx}: header line {i} starts at {} but line {} ended at {cursor} \
                         — the band has a hole or an overlap",
                        head.top,
                        i.saturating_sub(1)
                    );
                    let want_h = if i + 1 == header_rows { lh + gap } else { lh };
                    assert!(
                        (head.height - want_h).abs() < 1e-3,
                        "{ctx}: header line {i} is {} tall, want {want_h} — the beat \
                         belongs to the LAST header line's own box",
                        head.height
                    );
                    cursor = head.bottom();
                }
                // …and the band closes exactly on the candidate band's first row,
                // graded against the ROW family's own owner.
                //
                // WITH NO HEADER LINE there is no box for the beat to live in, so
                // the closure claim is scoped to a card that has one. Production
                // never constructs the other combination — `header_rows == 0` is
                // exactly the contextual spell popup, which sets `header_gap` to
                // `0.0` in the same breath (`overlay_geometry`'s `contextual`
                // arm) — and the zero-beat cell of this very sweep proves the two
                // agree there.
                let first = plan
                    .row_top(0)
                    .unwrap_or_else(|| panic!("{ctx}: two rows were planned"));
                if header_rows > 0 {
                    assert!(
                        (cursor - first).abs() < 1e-3,
                        "{ctx}: the header band ends at {cursor} but the first candidate \
                         row is planned at {first}"
                    );
                } else {
                    assert!(
                        (first - (TEXT_TOP + gap)).abs() < 1e-3,
                        "{ctx}: a headerless card's band starts at its own text top"
                    );
                }
                // The query field is the FIRST header line; the strip is the LAST,
                // and only exists once there are two.
                assert_eq!(
                    plan.query_band().map(|f| f.top),
                    heads.first().map(|h| h.top),
                    "{ctx}: the query field is header line 0"
                );
                assert_eq!(
                    plan.strip_band().map(|s| s.top),
                    (header_rows >= 2).then(|| heads[header_rows - 1].top),
                    "{ctx}: the lens strip is the LAST header line of a card that has \
                     more than one"
                );
            }
        }
    }
}

/// THE SECONDARY COLUMN AND THE CANDIDATE BAND SHARE ONE Y-ORIGIN. The right
/// column is a uniform-`lh` buffer leading with `header_rows` empty lines, so
/// uploading it at `secondary_top()` must land label `r` exactly on `row_top(r)`.
///
/// The oracle walks the buffer's OWN model — `origin + leading_lines * lh` —
/// rather than asking the plan a second time.
#[test]
fn the_secondary_origin_lands_every_label_on_its_own_planned_row() {
    for header_rows in 0usize..=2 {
        for &gap in &[0.0f32, 5.0, 42.0] {
            for &lh in &[12.0f32, 27.2, 81.6] {
                let plan = headered(header_rows, gap, lh);
                let origin = plan.secondary_top();
                assert_eq!(
                    plan.header_rows(),
                    header_rows,
                    "the plan must report the leading-line count the column pads with"
                );
                for r in 0usize..2 {
                    let label_top = origin + (plan.header_rows() + r) as f32 * lh;
                    let band_top = plan.row_top(r).unwrap();
                    assert!(
                        (label_top - band_top).abs() < 1e-3,
                        "hdr={header_rows} gap={gap} lh={lh}: secondary label {r} lands \
                         at {label_top}, its band at {band_top} — the shortcut would ride \
                         off its own row"
                    );
                }
            }
        }
    }
}

/// THE SPLIT-PANE GAP IS CARVED OUT OF THE LAST HEADER LINE'S OWN BOX. It never
/// crosses into the candidate band, never starts above that box, and is exactly
/// `SPLIT_GAP_FRAC` of the beat tall — for BOTH arms (flat hangs it from the
/// box's bottom edge, grouped from its top edge plus the item-83 breathe).
///
/// The containment oracle is the header band accumulated independently (as
/// above), not `split_bounds`' own inputs.
#[test]
fn the_split_gap_stays_inside_the_last_header_lines_box() {
    for &gap in &[5.0f32, 42.0, 84.3] {
        for &lh in &[12.0f32, 27.2, 81.6] {
            for header_rows in [1usize, 2] {
                let plan = headered(header_rows, gap, lh);
                let ctx = format!("hdr={header_rows} gap={gap} lh={lh}");
                let (gt, gb) = plan
                    .split_bounds()
                    .unwrap_or_else(|| panic!("{ctx}: a card with a header and a beat splits"));
                let beat_box = plan.header_lines()[header_rows - 1];
                assert!(gb > gt, "{ctx}: the gap is a real band");
                assert!(
                    (gb - gt - gap * 0.4).abs() < 1e-3,
                    "{ctx}: the gap is 0.4 of the beat, got {}",
                    gb - gt
                );
                assert!(
                    gt >= beat_box.top - 1e-3 && gb <= beat_box.bottom() + 1e-3,
                    "{ctx}: the gap [{gt}, {gb}] escapes the last header line's box \
                     [{}, {}]",
                    beat_box.top,
                    beat_box.bottom()
                );
                // Never into the candidate band.
                assert!(
                    gb <= plan.row_top(0).unwrap() + 1e-3,
                    "{ctx}: the gap runs into the first candidate row"
                );
                // The two arms differ, and differ in the documented direction:
                // flat hangs from the bottom edge, grouped from the top.
                if header_rows == 1 {
                    assert!(
                        (gb - beat_box.bottom()).abs() < 1e-3,
                        "{ctx}: the flat arm's gap is flush against the first row"
                    );
                } else {
                    assert!(
                        (gt - (beat_box.top + gap * 0.2)).abs() < 1e-3,
                        "{ctx}: the grouped arm's gap hangs one breathe below the query box"
                    );
                }
            }
        }
    }
}

/// A CARD WITH NO QUERY LINE PLANS NO HEADER BAND AT ALL — the contextual spell
/// popup. Nothing to centre a caret in, nothing to hit-test, nothing to split.
#[test]
fn a_contextual_popup_plans_no_header_band() {
    for &lh in &[12.0f32, 27.2] {
        let plan = headered(0, 0.0, lh);
        assert!(plan.header_lines().is_empty());
        assert_eq!(plan.query_band(), None);
        assert_eq!(plan.strip_band(), None);
        assert_eq!(plan.split_bounds(), None);
        assert_eq!(plan.header_rows(), 0);
        // The candidate band starts at the card's own text top, with no beat.
        assert!((plan.row_top(0).unwrap() - TEXT_TOP).abs() < 1e-3);
        // A zero BEAT never splits either, even with a header line.
        assert_eq!(headered(1, 0.0, lh).split_bounds(), None);
    }
}

/// THE QUERY FIELD'S BOX IS NOT THE ROW PITCH — the arithmetic shape of the
/// live defect this family closed. On the FLAT family (one header line) the beat
/// inflates the query line itself, so a consumer reading the bare `lh` describes
/// a box that stops well short of the field's own ink; on the GROUPED family
/// (two lines) the beat inflates the STRIP instead, so the same wrong reading
/// happens to be right. A parallel calculation agrees on the arm somebody looked
/// at — which is why this is asserted as a DIFFERENCE, by name.
#[test]
fn the_flat_query_box_is_taller_than_the_row_pitch_and_the_grouped_one_is_not() {
    let (gap, lh) = (42.0f32, 27.2f32);
    let flat_field = headered(1, gap, lh).query_band().unwrap();
    assert!(
        (flat_field.height - (lh + gap)).abs() < 1e-3,
        "the flat query box carries the beat: {} vs {}",
        flat_field.height,
        lh + gap
    );
    assert!(
        flat_field.center() > flat_field.top + lh,
        "the flat field's own centre ({}) sits BELOW a bare-`lh` band ending at {} — \
         a consumer reading the row pitch misses the caret entirely",
        flat_field.center(),
        flat_field.top + lh
    );
    let grouped = headered(2, gap, lh);
    let grouped_field = grouped.query_band().unwrap();
    assert!(
        (grouped_field.height - lh).abs() < 1e-3,
        "the grouped query box is a plain row: {}",
        grouped_field.height
    );
    assert!(
        (grouped.strip_band().unwrap().height - (lh + gap)).abs() < 1e-3,
        "…and the grouped family's beat inflates its STRIP instead"
    );
    // `contains` is the pointer's own predicate: half-open, so the boundary
    // belongs to exactly one box.
    assert!(flat_field.contains(flat_field.top));
    assert!(!flat_field.contains(flat_field.bottom()));
    assert!(!flat_field.contains(flat_field.top - 0.01));
}
