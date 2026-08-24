//! FOLD RENDER LAW: a collapsed section's hidden lines are dropped from the shaped
//! text, so they contribute ZERO visual rows (and zero height) — the row simply is
//! not laid out. Driven through the SAME `fold::apply_to_view` seam the live
//! `sync_view` and the headless capture use, then shaped by a real headless
//! pipeline so the geometry is the true one.

use super::{headless_pipeline, view, view_md};
use crate::render::FoldTail;
use std::collections::BTreeSet;

// Two sibling sections, no soft-wrap:
//   0 # A / 1 a1 / 2 a2 / 3 # B / 4 b1
const DOC: &str = "# A\na1\na2\n# B\nb1";

/// Fold the given heading lines of `DOC` and return the `(hidden mask, tails,
/// folds)` the live `sync_view` / capture builders feed
/// [`crate::fold::apply_to_view`].
fn fold(headings: &[usize]) -> (Vec<bool>, Vec<(usize, usize)>, BTreeSet<usize>) {
    let levels = crate::fold::heading_levels(DOC, true);
    let folds: BTreeSet<usize> = headings.iter().copied().collect();
    (
        crate::fold::hidden_lines(&levels, &folds),
        crate::fold::fold_tails(&levels, &folds),
        folds,
    )
}

#[test]
fn a_folded_section_contributes_zero_visual_rows() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold render law");
        return;
    };

    // UNFOLDED: all five logical lines shape to five visual rows (no wrap).
    let unfolded = view_md(DOC, 0, 0);
    p.set_view(&unfolded);
    let rows_unfolded = p.total_visual_rows();
    assert_eq!(rows_unfolded, 5, "five lines, five visual rows unfolded");

    // FOLD # A (line 0): its section is lines 1..=2 (a1, a2). Feed the hidden mask
    // through the shared fold seam — exactly what the app/capture builders do.
    let (hidden, tails, folds) = fold(&[0]);
    let mut folded = view_md(DOC, 0, 0);
    crate::fold::apply_to_view(&mut folded, &hidden, &tails, &folds);
    // The two hidden lines are gone from the shaped text (so they cannot lay out).
    assert_eq!(folded.text, "# A\n# B\nb1");
    p.set_view(&folded);
    let rows_folded = p.total_visual_rows();
    assert_eq!(
        rows_folded, 3,
        "the two hidden lines contribute ZERO visual rows"
    );
    assert_eq!(
        rows_unfolded - rows_folded,
        2,
        "exactly the folded section's line count is removed from the layout"
    );
}

// The quiet "… N lines" TAIL on a collapsed heading: it carries the
// CORRECT hidden count, rides the heading's OWN row (adds no row / never disturbs
// the zero-height hidden-row law), and hangs to the RIGHT of the heading text.
#[test]
fn fold_tail_rides_the_heading_row_with_the_correct_count() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-tail law");
        return;
    };

    // FOLD # A (line 0): hides a1, a2 → the tail reads "… 2 lines" on the heading.
    let (hidden, tails, folds) = fold(&[0]);
    let mut folded = view_md(DOC, 0, 0);
    crate::fold::apply_to_view(&mut folded, &hidden, &tails, &folds);
    // The view records the tail on the heading's FILTERED row (0) with count 2.
    assert_eq!(folded.fold_tails, vec![FoldTail { line: 0, hidden: 2 }]);

    p.set_view(&folded);
    // The tail is an ORNAMENT, not a shaped line: the folded doc is still exactly 3
    // visual rows — the tail added NONE (the zero-height hidden-row law is intact).
    assert_eq!(
        p.total_visual_rows(),
        3,
        "the tail rides the heading row; it adds no visual row"
    );

    // ONE mark, on the heading's own row, with the right N, past the heading text.
    let marks = p.fold_tail_marks();
    assert_eq!(marks.len(), 1, "one tail for the one collapsed heading");
    let (baseline, left, n, line) = marks[0];
    assert_eq!(n, 2, "the tail's N is the section's hidden-line count");
    assert_eq!(line, 0, "the tail hangs on the filtered heading row");
    // The mark's `f32` slot is the heading's REAL shaped BASELINE (the
    // draw pass then subtracts the tail's OWN shaped `line_y` from this), not the
    // row's top — baseline-aligned, not merely centered in the tall heading row.
    assert_eq!(
        baseline,
        p.line_ornament_baseline(0),
        "the tail's placement baseline is the heading row's own REAL shaped baseline"
    );
    assert!(
        left > p.text_left(),
        "the tail sits to the RIGHT of the heading text, not over it"
    );
}

// The count tracks the ACTUAL hidden extent: folding the deeper section # B (which
// hides only its single body line) reads "… 1 line", and a nested fold's tail is
// suppressed while its parent is folded (the parent's count already covers it).
#[test]
fn fold_tail_count_tracks_the_hidden_extent() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-tail count law");
        return;
    };

    // FOLD # B (line 3): its section is line 4 (b1) only → "… 1 line". Only that one
    // line below it hides, so # B keeps its full-doc row 3 in the filtered document.
    let (hidden, tails, folds) = fold(&[3]);
    let mut folded = view_md(DOC, 0, 0);
    crate::fold::apply_to_view(&mut folded, &hidden, &tails, &folds);
    assert_eq!(folded.fold_tails, vec![FoldTail { line: 3, hidden: 1 }]);
    p.set_view(&folded);
    let marks = p.fold_tail_marks();
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].2, 1, "# B hides exactly one line");
    assert_eq!(marks[0].3, 3, "# B renders on filtered row 3");
}

// GALLERY-FOUND REGRESSION: a collapsed heading long enough to visually
// WRAP used to hang its tail off the FLATTENED end-of-line x
// (`line_glyph_xs(line).last()`, which deliberately offsets each wrapped row's
// glyphs to continue past the previous one for callers that don't care which row
// a column lands on) — landing the tail comfortably past the actual column, off
// in the page's right margin, utterly disconnected from the heading it annotates.
// The fix reads the FIRST VISUAL ROW's own row-LOCAL end x
// (`visual_rows(line)[0]`, never offset across rows), matching where the tail's
// BASELINE already sits (always the first row's — see `fold_tail_marks`'s doc).
#[test]
fn fold_tail_hangs_after_the_first_visual_row_when_the_heading_wraps() {
    // Acquire and pin BEFORE pipeline creation. Construction reads the page
    // globals to choose wrap geometry, so locking only after `headless_pipeline`
    // leaves a predecessor's sticky custom measure observable here.
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-tail wrap regression law");
        return;
    };
    // PAGE MODE globals are process-wide (`crate::page::page_on`/`measure`, which
    // `text_wrap_width`/`text_left` below read live): serialize with every other
    // page-touching test, mirroring this file's own `fold_chevron_*` siblings.
    // Without this, dozens of siblings elsewhere (`render::tests::outline::*` in
    // particular) that correctly hold `serial()` while their OWN body runs at
    // `measure(40)` still race a reader that never takes the lock at all — the
    // guard only excludes OTHER lock-holders, so an unguarded reader sails
    // straight through their locked critical section and catches page geometry
    // mid-mutation. Confirmed empirically: paired alone with any one of six
    // different `outline::` tests (each independently correctly-guarded), this
    // test still fails ~1/25 runs with the exact reported signature
    // (`ceiling=844.80005`, i.e. a `measure(40)` mid-flight read).
    // A single H1 line long enough to wrap at the default 1200px canvas (H1's
    // scaled-up glyphs make even a fairly ordinary sentence wrap).
    let doc = "# A rather long section heading that keeps going for quite a while indeed so it wraps here\nbody one\nbody two\n";
    let levels = crate::fold::heading_levels(doc, true);
    let folds: BTreeSet<usize> = [0].into_iter().collect();
    let hidden = crate::fold::hidden_lines(&levels, &folds);
    let tails = crate::fold::fold_tails(&levels, &folds);

    let mut view = view_md(doc, 0, 0);
    crate::fold::apply_to_view(&mut view, &hidden, &tails, &folds);
    p.set_view(&view);

    // Fixture self-check: the heading genuinely wraps to more than one visual row
    // (else this test cannot witness the bug it guards against).
    let rows = p.visual_rows(0);
    assert!(
        rows.len() > 1,
        "fixture must actually wrap the heading to >1 visual row, got {}",
        rows.len()
    );
    // The OLD (buggy) placement's `end` — the flattened, cumulative-across-rows x —
    // is what the fix must NOT use: it lands far past the actual column.
    let flattened_end = p.line_glyph_xs(0).last().copied().unwrap_or(0.0);
    let buggy_left = p.text_left() + flattened_end;

    let marks = p.fold_tail_marks();
    assert_eq!(marks.len(), 1);
    let (_, left, _, _) = marks[0];

    // FIX: the tail stays within a first-row-sized budget — comfortably inside the
    // actual wrap width, nowhere near the buggy flattened placement.
    let ceiling = p.text_left() + p.text_wrap_width();
    assert!(
        left <= ceiling,
        "the tail must hang within the actual column, not past it: left={left} ceiling={ceiling}"
    );
    assert!(
        left < buggy_left - p.metrics.char_width,
        "the tail must NOT land at the old flattened (cumulative-across-wrapped-rows) x: \
         left={left} buggy_flattened_left={buggy_left}"
    );
}

/// The concrete predecessor/victim order. The retina schema capture
/// once exited at exactly `(page_on=true, measure=80)`; this calls the fold
/// victim immediately after installing that signature. The victim's pin must
/// set the prose default before it constructs its pipeline and restore the
/// predecessor signature when it leaves.
#[test]
fn retina_measure_predecessor_cannot_contaminate_fold_tail_victim() {
    // This is an intentional product-style predecessor: it models the legacy
    // retina writer's sticky exit, while the nested victim still owns its own
    // PagePin. Use the existing reentrant product door, never a second mutex.
    let _predecessor = crate::testlock::product();
    let _incoming = crate::page::PagePin::snapshot();
    for pair in 0..40 {
        crate::page::set_page_on(true);
        crate::page::set_measure(80);
        assert_eq!(
            (crate::page::page_on(), crate::page::measure()),
            (true, 80),
            "pair {pair}: the predecessor really installs the retired signature"
        );

        fold_tail_hangs_after_the_first_visual_row_when_the_heading_wraps();

        assert_eq!(
            (crate::page::page_on(), crate::page::measure()),
            (true, 80),
            "pair {pair}: the victim restores its predecessor exactly; its own pipeline used the pinned prose default"
        );
    }
}

// The expand CHEVRON is a SUMMONED
// affordance: shown only when the caret is on the collapsed heading (the
// headlessly-reachable arm; hover is live-only). It now hangs IMMEDIATELY LEFT of
// the heading — OUTSIDE the editable text advance, in the writing column's own
// leading pad — never sharing the tail's (unmoved, right-of-text) slot. The tail
// never hides.
#[test]
fn fold_chevron_reveals_only_when_the_caret_is_on_the_collapsed_heading() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron law");
        return;
    };
    // PAGE MODE globals are process-wide; serialize with every other page test and
    // restore the default so a later test never inherits this one's setting.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    let (hidden, tails, folds) = fold(&[0]); // fold # A; its tail hangs on filtered row 0

    // Caret ON the collapsed heading (# A, full line 0 → filtered row 0, where folding
    // parks it): the chevron reveals on that row, LEFT of the heading text.
    let mut on = view_md(DOC, 0, 0);
    crate::fold::apply_to_view(&mut on, &hidden, &tails, &folds);
    assert_eq!(
        on.cursor_line, 0,
        "caret on the folded heading's filtered row"
    );
    p.set_view(&on);
    let ch = p.fold_chevron_marks();
    assert_eq!(ch.len(), 1, "the caret-on-heading chevron reveals");
    assert_eq!(ch[0].2, 0, "on the heading's own filtered row");
    assert_eq!(
        ch[0].0,
        p.line_ornament_top(0) + p.visual_rows(0)[0].line_height * 0.5,
        "the chevron's placement target is the heading's real shaped-row centre"
    );
    assert!(
        ch[0].1 < p.text_left(),
        "the chevron sits OUTSIDE the editable text advance, strictly left of text_left \
         (got {} vs text_left {})",
        ch[0].1,
        p.text_left()
    );
    let tail = p.fold_tail_marks();
    assert!(
        ch[0].1 < tail[0].1,
        "the chevron (left margin) sits LEFT of the tail (right of the heading text)"
    );

    // Caret OFF the heading (on b1, full line 4 → a different filtered row): NO chevron,
    // but the tail is still shown — the tail is unconditional, the chevron summoned.
    let mut off = view_md(DOC, 4, 0);
    crate::fold::apply_to_view(&mut off, &hidden, &tails, &folds);
    assert_ne!(off.cursor_line, 0, "caret is not on the collapsed heading");
    p.set_view(&off);
    assert!(
        p.fold_chevron_marks().is_empty(),
        "no chevron when the caret is not on the heading (and no pointer to hover) — \
         rest state shows no chevron"
    );
    assert_eq!(
        p.fold_tail_marks().len(),
        1,
        "the tail is always shown, chevron or not"
    );
    crate::page::set_page_on(true);
}

// NO-OVERLAP PIXEL LAW: the chevron is a SEPARATE ornament — never part of
// the shaped document glyph run — so revealing it must never shift the heading's
// own glyph x-positions vs the no-chevron REST state. Isolated via HOVER (not the
// caret): landing the CARET on a heading ALSO reveals its raw WYSIWYG markdown
// markup (PHILOSOPHY.md's "any line shows raw markdown while the caret is on
// it"), which genuinely DOES change that line's glyph advances (CLAUDE.md's own
// tripwire: "Conceal reveal changes glyph advances, not just color") — a real,
// unrelated effect that would otherwise contaminate this law. Hover triggers the
// SAME `chevron_revealed` arm without touching conceal state, so comparing
// rest-vs-hover (the pixel law's own phrasing) isolates the chevron alone.
// Compared via the SAME real-shaped-glyph source the caret/hit-test/selection all
// read ([`TextPipeline::line_glyph_xs`]), not re-derived pixels.
#[test]
fn fold_chevron_reveal_never_shifts_the_heading_glyph_positions() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron no-overlap law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    let (hidden, tails, folds) = fold(&[0]); // fold # A
    // Caret parked away from the heading for BOTH states (line 4, "b1") — WYSIWYG
    // conceal on the heading line is therefore identical in both; only hover varies.
    let mut view = view_md(DOC, 4, 0);
    crate::fold::apply_to_view(&mut view, &hidden, &tails, &folds);

    // REST: no hover, chevron absent.
    p.set_view(&view);
    p.set_hover_line(None);
    assert!(p.fold_chevron_marks().is_empty(), "rest state: no chevron");
    let xs_rest = p.line_glyph_xs(0);
    let top_rest = p.line_ornament_top(0);
    let rows_rest = p.total_visual_rows();

    // HOVER on the collapsed heading's row: chevron revealed.
    p.set_hover_line(Some(0));
    assert_eq!(
        p.fold_chevron_marks().len(),
        1,
        "chevron revealed: hovering the heading"
    );
    let xs_reveal = p.line_glyph_xs(0);
    let top_reveal = p.line_ornament_top(0);
    let rows_reveal = p.total_visual_rows();

    assert_eq!(
        xs_rest, xs_reveal,
        "the heading's own shaped glyph x-boundaries must be IDENTICAL whether or \
         not the chevron is revealed (it lives outside the text advance entirely)"
    );
    assert_eq!(
        top_rest, top_reveal,
        "the heading row's top never moves either"
    );
    assert_eq!(
        rows_rest, rows_reveal,
        "revealing the chevron adds no visual row"
    );
    crate::page::set_page_on(true);
}

// Graceful-hide: the chevron needs room in the writing column's own
// leading pad ([`TextPipeline::text_left`] minus [`TextPipeline::column_left`]).
// Edge-to-edge (page mode off) that pad is exactly zero, so the chevron would
// otherwise land ON TOP of the heading's own first glyph — instead it hides
// entirely, mirroring the outline's / gutter's own no-room floors. The tail is
// UNAFFECTED (it hangs to the right, in the always-available wrap width), so the
// collapsed state stays legible either way.
#[test]
fn fold_chevron_hides_gracefully_with_no_room_edge_to_edge() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron no-room law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(false); // edge-to-edge: text_left == column_left, zero pad
    let (hidden, tails, folds) = fold(&[0]);
    let mut on = view_md(DOC, 0, 0);
    crate::fold::apply_to_view(&mut on, &hidden, &tails, &folds);
    p.set_view(&on);
    assert!(
        (p.text_left() - p.column_left()).abs() < 0.01,
        "edge-to-edge has no writing-column leading pad to hang the chevron in"
    );
    assert!(
        p.fold_chevron_marks().is_empty(),
        "no room => the chevron gracefully hides even with the caret on the heading"
    );
    assert_eq!(
        p.fold_tail_marks().len(),
        1,
        "the tail is unaffected by the chevron's room gate — still shows the count"
    );
    crate::page::set_page_on(true);
}

// The fold chevron is REVEALED (and clickable) on an EXPANDED
// heading too, not merely a collapsed one's tail row. Pre-fix, `fold_chevron_marks`
// read only `fold_tails` (empty whenever nothing is folded), so hovering an
// unfolded heading never showed anything — this test's middle assertion FAILS
// against that old behavior.
#[test]
fn fold_chevron_reveals_and_hits_on_an_expanded_heading_too() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron expanded-heading law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);

    // UNFOLDED DOC, caret away from the heading: rest state shows no chevron.
    let view = view_md(DOC, 4, 0);
    p.set_view(&view);
    p.set_hover_line(None);
    assert!(
        p.fold_chevron_marks().is_empty(),
        "rest state: an expanded heading shows no chevron either"
    );

    // HOVER the expanded heading's row (line 0, never folded): its chevron
    // reveals — the NEW behavior.
    p.set_hover_line(Some(0));
    let marks = p.fold_chevron_marks();
    assert_eq!(
        marks.len(),
        1,
        "hovering an EXPANDED heading reveals its chevron too, not just a \
         collapsed one's — fails pre-item-81 (fold_chevron_marks read fold_tails, \
         empty when nothing is folded)"
    );
    let (_, left, line) = marks[0];
    assert_eq!(line, 0);

    // The revealed chevron's own pixel box hit-tests to this heading line.
    let top = p.line_ornament_top(line);
    assert_eq!(
        p.fold_chevron_hit(left + 1.0, top + 1.0),
        Some(0),
        "the chevron's own pixel box hit-tests to the heading"
    );

    // Off the chevron's narrow lane — over the heading's own TEXT instead — is
    // NOT a hit: a click there must place the caret, never toggle the fold.
    assert_eq!(
        p.fold_chevron_hit(p.text_left() + 5.0, top + 1.0),
        None,
        "a click on the heading's own text is not the chevron's hit region"
    );
    // Above/below the row entirely is not a hit either.
    assert_eq!(p.fold_chevron_hit(left + 1.0, top - 50.0), None);
    crate::page::set_page_on(true);
}

// The item-65 no-overlap pixel law, extended to an EXPANDED heading: revealing
// its chevron via hover must never shift the heading's own shaped glyph
// positions, row top, or visual-row count — it is a left-margin ornament outside
// the shaped text run, exactly like the collapsed case.
#[test]
fn fold_chevron_reveal_on_an_expanded_heading_never_shifts_glyph_positions() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron expanded no-overlap law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    // Caret away from the heading in BOTH states, so WYSIWYG conceal on the
    // heading line never varies — only hover does (mirrors the collapsed-case law).
    let view = view_md(DOC, 4, 0);

    p.set_view(&view);
    p.set_hover_line(None);
    assert!(p.fold_chevron_marks().is_empty(), "rest state: no chevron");
    let xs_rest = p.line_glyph_xs(0);
    let top_rest = p.line_ornament_top(0);
    let rows_rest = p.total_visual_rows();

    p.set_hover_line(Some(0));
    assert_eq!(
        p.fold_chevron_marks().len(),
        1,
        "chevron revealed: hovering the heading"
    );
    let xs_reveal = p.line_glyph_xs(0);
    let top_reveal = p.line_ornament_top(0);
    let rows_reveal = p.total_visual_rows();

    assert_eq!(
        xs_rest, xs_reveal,
        "the expanded heading's shaped glyph x-boundaries must be IDENTICAL \
         whether or not the chevron is revealed"
    );
    assert_eq!(
        top_rest, top_reveal,
        "the heading row's top never moves either"
    );
    assert_eq!(
        rows_rest, rows_reveal,
        "revealing the chevron adds no visual row"
    );
    crate::page::set_page_on(true);
}

// LAW (required): the chevron's hit region resolves for a heading whichever way
// it currently faces, and toggling through the ONE owner (`crate::fold::
// toggle_heading`) flips it correctly BOTH times — an expanded heading's
// "collapse" click and a collapsed heading's "expand" click are never two
// separate code paths that could drift. Non-vacuous: pre-item-81, the first
// (expanded) hit-test below returns `None` (no chevron existed to hit at all),
// so this test fails before the fix exactly like the reveal law above.
#[test]
fn fold_chevron_hit_toggles_through_one_owner_both_directions() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron one-owner law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    let levels = crate::fold::heading_levels(DOC, true);
    let mut folds: BTreeSet<usize> = BTreeSet::new();

    // EXPANDED: hover heading 0, hit its chevron, toggle via the one owner.
    let view = view_md(DOC, 4, 0);
    p.set_view(&view);
    p.set_hover_line(Some(0));
    let marks = p.fold_chevron_marks();
    assert_eq!(
        marks.len(),
        1,
        "expanded heading's chevron must be revealed to hit it"
    );
    let (_, left, line) = marks[0];
    let top = p.line_ornament_top(line);
    let hit = p
        .fold_chevron_hit(left + 1.0, top + 1.0)
        .expect("the expanded chevron's own pixel box must hit-test");
    assert!(
        crate::fold::toggle_heading(&levels, &mut folds, hit),
        "hit resolves to a real heading line"
    );
    assert!(folds.contains(&0), "the expanded heading is now folded");

    // COLLAPSED: rebuild the folded view through the SAME real fold seam, re-hover
    // the same row, hit its chevron again, toggle via the SAME owner.
    let tails = crate::fold::fold_tails(&levels, &folds);
    let hidden = crate::fold::hidden_lines(&levels, &folds);
    let mut folded_view = view_md(DOC, 4, 0);
    crate::fold::apply_to_view(&mut folded_view, &hidden, &tails, &folds);
    p.set_view(&folded_view);
    p.set_hover_line(Some(0));
    let marks2 = p.fold_chevron_marks();
    assert_eq!(
        marks2.len(),
        1,
        "collapsed heading's chevron must be revealed to hit it"
    );
    let (_, left2, line2) = marks2[0];
    let top2 = p.line_ornament_top(line2);
    let hit2 = p
        .fold_chevron_hit(left2 + 1.0, top2 + 1.0)
        .expect("the collapsed chevron's own pixel box must hit-test");
    assert!(crate::fold::toggle_heading(&levels, &mut folds, hit2));
    assert!(
        !folds.contains(&0),
        "the SAME owner reverses it — one function, both directions"
    );
    crate::page::set_page_on(true);
}

// Non-heading rows and non-Markdown files show no fold affordance at all.
#[test]
fn no_chevron_off_a_heading_or_off_a_markdown_buffer() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron gating law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);

    // A markdown buffer, hovering a BODY (non-heading) row: no chevron.
    let md_view = view_md(DOC, 4, 0);
    p.set_view(&md_view);
    p.set_hover_line(Some(1)); // "a1", a body row
    assert!(
        p.fold_chevron_marks().is_empty(),
        "a non-heading row never shows a chevron, even hovered"
    );
    assert_eq!(
        p.fold_chevron_hit(p.column_left() + 1.0, p.line_ornament_top(1) + 1.0),
        None
    );

    // The SAME text, NOT flagged markdown: no chevron even hovering the heading row.
    let plain_view = view(DOC, 4, 0);
    p.set_view(&plain_view);
    p.set_hover_line(Some(0));
    assert!(
        p.fold_chevron_marks().is_empty(),
        "a non-markdown buffer offers no fold affordance at all"
    );
    crate::page::set_page_on(true);
}

/// DRAWN ⇔ ANNOUNCED WITH A SECTION FOLDED. The renderer used to
/// derive the card's DOCUMENT figures from the text it happened to be shaping,
/// which a fold has already filtered down to the visible lines. So the drawn
/// WORD COUNT was over the visible document while the announced one — which
/// `App::card_inputs` derives from the buffer — was over the whole buffer: two
/// surfaces disagreeing about one fact.
///
/// This drives the REAL seams on both sides. The announced side is built from a
/// real [`crate::buffer::Buffer`] through the exact expression the semantic fold
/// uses; the drawn side is a real headless pipeline fed through
/// [`crate::fold::apply_to_view`], asked for the same
/// [`crate::card::content::CardInputs`] a frame composes its card from. The
/// hand-derived fixture numbers are asserted alongside, so the law does not rest
/// on the two sides being wrong in the same way.
#[test]
fn a_folded_section_never_moves_the_card_figures_the_pipeline_draws() {
    let _g = crate::testlock::serial();
    use crate::card::figures::{DocFigures, fixture};

    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping folded-card-figures law");
        return;
    };
    let _g = crate::testlock::serial();

    // THE ANNOUNCED SIDE, verbatim from `App::card_inputs`: the buffer's own
    // text, kind and caret. A fold is view state, so the buffer never sees one.
    let mut buffer = crate::buffer::Buffer::from_str(fixture::DOC);
    buffer.set_cursor(63); // the start of `beta five six` — fixture::CARET
    assert_eq!(buffer.cursor_line_col(), fixture::CARET);
    let (cl, cc) = buffer.cursor_line_col();
    let announced = DocFigures::of(&buffer.text(), buffer.is_markdown(), cl, cc);
    assert_eq!(announced.words, fixture::WORDS, "hand-derived word count");
    assert_eq!(announced.percent, fixture::PERCENT, "hand-derived percent");

    // THE DRAWN SIDE, unfolded first — the baseline both readings already agreed
    // on, so a regression here would be a different bug.
    let mut v = view_md(fixture::DOC, fixture::CARET.0, fixture::CARET.1);
    p.set_view(&v);
    assert_eq!(p.card_inputs().doc, announced, "unfolded baseline");

    // …then with `# Alpha` collapsed, through the shared fold seam.
    let levels = crate::fold::heading_levels(fixture::DOC, true);
    let set: BTreeSet<usize> = [fixture::FOLD_HEADING].into_iter().collect();
    crate::fold::apply_to_view(
        &mut v,
        &crate::fold::hidden_lines(&levels, &set),
        &crate::fold::fold_tails(&levels, &set),
        &set,
    );
    p.set_view(&v);

    // The pipeline really is shaping the FILTERED document — otherwise nothing
    // was folded and the assertion below is free.
    assert_eq!(p.doc_text(), fixture::FOLDED);
    assert_eq!(p.cursor_line, fixture::FOLDED_CARET_LINE);

    assert_eq!(
        p.card_inputs().doc,
        announced,
        "folding a section moved the DRAWN card figures away from the announced ones",
    );
    // And the sidecar's own HUD block, which is how a capture reports the card.
    let hud = p.hud_report();
    assert_eq!(hud.words, Some(fixture::WORDS_PAIR));
    assert_eq!(hud.percent, fixture::PERCENT);
    assert_eq!(hud.lang, Some(crate::frontmatter::Lang::Ja));
    assert_eq!(p.readout_report(), Some(fixture::WORDS_PAIR));

    // The reading the bug produced, named: it is a DIFFERENT answer, so this law
    // is not green by coincidence.
    assert_eq!(
        DocFigures::of(&p.doc_text(), true, p.cursor_line, p.cursor_col).words,
        fixture::FOLDED_WORDS,
    );
}

// === GEOMETRY CHANGED UNDER AN UNMOVED POINTER ============================
//
// `App::update_fold_hover` decides which row's chevron to reveal by asking
// `hit_test_scroll(px, py, scroll)` — the SAME pixel, a DIFFERENT `scroll`,
// can answer a different document row without the pointer ever producing a
// `CursorMoved` (a wheel scroll, a keyboard-driven page-down or caret-chase).
// This proves that sensitivity at the exact primitive the resync reads, then
// the downstream effect: re-deriving the hover row from the POST-scroll
// geometry must clear a chevron that scrolled away, not leave it lit.

#[test]
fn fold_hover_tracks_the_row_under_an_unmoved_pointer_across_a_scroll() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("no GPU adapter; skipping scroll-resync law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);

    // DOC = "# A\na1\na2\n# B\nb1" — a heading on line 0, plain text below.
    // Caret parked on line 2 (body text, not a heading): `chevron_revealed`
    // also lights up on `line == cursor_line`, so the caret must sit away
    // from every heading or it would keep # A's chevron lit on its own,
    // independent of whatever `hover_line` this law drives.
    let view = view_md(DOC, 2, 0);
    p.set_view(&view);

    let px = p.text_left() + 1.0;
    let py = p.text_origin_top() + 1.0;

    // AT REST (scroll row 0): the fixed pixel resolves to line 0, the heading.
    let (line_before, _) = p.hit_test_scroll(px, py, crate::render::ScrollPos::at_row(0));
    assert_eq!(
        line_before, 0,
        "fixture sanity: unscrolled, the pixel sits on the heading"
    );

    // A SYNTHETIC SCROLL, same pointer position — no `CursorMoved` at all: the
    // SAME pixel now resolves to a DIFFERENT document row, the doc's last
    // line scrolled up under it.
    let (line_after, _) = p.hit_test_scroll(px, py, crate::render::ScrollPos::at_row(4));
    assert_ne!(
        line_after, line_before,
        "a scroll must change which row an unmoved pixel resolves to — the \
         exact geometry fact the wheel/keyboard-scroll resync depends on"
    );
    assert_eq!(
        line_after, 4,
        "fixture sanity: scrolled, the pixel sits on b1"
    );

    // THE DOWNSTREAM EFFECT: hovering the PRE-scroll row reveals # A's chevron…
    p.set_hover_line(Some(line_before));
    assert_eq!(
        p.fold_chevron_marks().len(),
        1,
        "hovering the heading row reveals its chevron"
    );
    // …but re-deriving the hover row from the POST-scroll geometry — exactly
    // what the gated resync now does every `sync_view` — hovers "b1" instead:
    // no heading there, so the chevron must clear. Before this fix nothing
    // ever re-asked this question after a scroll, so a chevron revealed
    // pre-scroll stayed lit here.
    p.set_hover_line(Some(line_after));
    assert!(
        p.fold_chevron_marks().is_empty(),
        "the row a scroll moved under the pointer is plain text — no chevron may remain lit"
    );
    crate::page::set_page_on(true);
}
