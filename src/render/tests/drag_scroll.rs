//! Scroll-on-drag: dragging a text selection past the writing column's
//! top/bottom edge used to stop at one screenful (`on_drag` -> `hit_test_scroll`
//! only ever resolved to whatever was ALREADY shaped on screen; nothing
//! advanced `scroll`). These laws cover the two pure/pipeline-testable halves
//! of the fix: the overshoot-to-rate curve on its own, then the composed tick
//! (overshoot -> rate -> `scroll_by_px` -> `hit_test_scroll`) that
//! `App::step_drag_scroll` (`app/input/mouse.rs`) drives — the App-level wire
//! itself has no GPU in a hermetic `App` test, so its own law is that it
//! degrades to a no-op without one (`app/input/tests.rs`).

use super::super::scroll::drag_scroll_rate;
use super::super::*;
use super::{H, headless_pipeline, view};

// --- The pure overshoot -> rate curve, no GPU required -----------------

#[test]
fn drag_scroll_rate_is_zero_inside_the_dead_zone() {
    for overshoot in [0.0, 1.0, 3.0, 6.0] {
        assert_eq!(
            drag_scroll_rate(overshoot),
            0.0,
            "overshoot {overshoot} sits inside the dead zone and must not scroll"
        );
    }
}

#[test]
fn drag_scroll_rate_is_positive_and_monotonically_increasing_past_the_dead_zone() {
    let samples = [7.0, 20.0, 50.0, 90.0, 140.0, 300.0, 5000.0];
    let mut prev = 0.0;
    for overshoot in samples {
        let rate = drag_scroll_rate(overshoot);
        assert!(
            rate > 0.0,
            "overshoot {overshoot} is past the dead zone and must scroll (rate {rate})"
        );
        assert!(
            rate >= prev,
            "the curve must never slow down as overshoot grows: {prev} then {rate} at {overshoot}"
        );
        prev = rate;
    }
}

#[test]
fn drag_scroll_rate_is_capped() {
    let at_ramp_end = drag_scroll_rate(1_000_000.0);
    let just_past_ramp = drag_scroll_rate(139.9);
    assert!(
        at_ramp_end >= just_past_ramp,
        "the cap must be the curve's maximum, not a value it can exceed then fall from"
    );
    // Two overshoots both past the ramp must read identically — the cap is
    // flat, not a further (even slower) climb.
    assert_eq!(
        drag_scroll_rate(10_000.0),
        drag_scroll_rate(1_000_000.0),
        "past the ramp the rate must be held flat at the cap"
    );
}

#[test]
fn drag_scroll_rate_reads_overshoot_magnitude_only() {
    // The caller's SIGN carries direction (up vs down); the curve itself is
    // symmetric — an "above the top edge" overshoot must scroll exactly as
    // fast as the same distance "below the bottom edge" would.
    assert_eq!(drag_scroll_rate(-42.0), drag_scroll_rate(42.0));
}

// --- Overshoot geometry: the writing column's screen-fixed band --------

#[test]
fn overshoot_is_zero_strictly_inside_the_writing_column_band() {
    let _g = crate::testlock::serial();
    let Some(p) = headless_pipeline() else {
        eprintln!("skipping overshoot_is_zero_strictly_inside_the_writing_column_band: no wgpu");
        return;
    };
    let top = p.text_origin_top();
    assert_eq!(
        p.drag_scroll_overshoot(top, H),
        0.0,
        "top edge itself is in-band"
    );
    assert_eq!(
        p.drag_scroll_overshoot((top + H) / 2.0, H),
        0.0,
        "the middle of the column is in-band"
    );
    assert_eq!(
        p.drag_scroll_overshoot(H, H),
        0.0,
        "bottom edge itself is in-band"
    );
}

#[test]
fn overshoot_is_signed_by_which_edge_the_pointer_crossed() {
    let _g = crate::testlock::serial();
    let Some(p) = headless_pipeline() else {
        eprintln!("skipping overshoot_is_signed_by_which_edge_the_pointer_crossed: no wgpu");
        return;
    };
    let top = p.text_origin_top();
    let above = p.drag_scroll_overshoot(top - 25.0, H);
    let below = p.drag_scroll_overshoot(H + 25.0, H);
    assert!(
        above < 0.0,
        "above the column must read negative, got {above}"
    );
    assert!(
        below > 0.0,
        "below the column must read positive, got {below}"
    );
    assert_eq!(above.abs(), 25.0);
    assert_eq!(below, 25.0);
}

// --- The composed tick: overshoot -> rate -> scroll -> hit-test --------

#[test]
fn drag_scroll_step_declines_in_band_and_on_the_first_tick() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping drag_scroll_step_declines_in_band_and_on_the_first_tick: no wgpu");
        return;
    };
    let text = "one\n".repeat(200);
    p.set_view(&view(&text, 0, 0));
    let scroll = ScrollPos::default();
    let px = p.text_left();
    let in_band_py = p.text_origin_top() + 10.0;
    assert!(
        p.drag_scroll_step(scroll, px, in_band_py, H, 0.5).is_none(),
        "a pointer inside the band must not advance scroll"
    );
    let beyond_py = H + 60.0;
    assert!(
        p.drag_scroll_step(scroll, px, beyond_py, H, 0.0).is_none(),
        "dt <= 0 (the drag's first tick past the edge) must not advance scroll"
    );
    assert!(
        p.drag_scroll_step(scroll, px, beyond_py, H, -1.0).is_none(),
        "a negative dt must not advance scroll"
    );
}

/// THE HEADLINE LAW: a drag held past the bottom edge must be able to reach
/// content beyond one screenful — the exact gap `on_drag` -> `hit_test_scroll`
/// used to close off (nothing ever advanced `scroll`, so the hit-test could
/// never resolve past whatever a single screen already showed). Synthetic
/// steps stand in for the live clock (the headless path has no clock of its
/// own): each iteration mirrors one `App::step_drag_scroll` call with a fixed
/// injected `dt`, exactly like `capture::capture_timeline`'s injected-dt
/// stepper advances the caret spring deterministically.
#[test]
fn a_held_drag_past_the_bottom_edge_scrolls_and_extends_past_one_screenful() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_held_drag_past_the_bottom_edge_scrolls_and_extends_past_one_screenful: \
             no wgpu"
        );
        return;
    };
    let lines = 400;
    let text: String = (0..lines).map(|i| format!("row {i}\n")).collect();
    p.set_view(&view(&text, 0, 0));

    // One screenful, in whole rows, at this pipeline's own metrics — the
    // ceiling a drag could reach BEFORE this fix (H is tall enough, and every
    // line here is a single visual row, so this is also the line-index
    // ceiling).
    let one_screenful_rows = (H / p.metrics.line_height).ceil() as usize;

    let px = p.text_left();
    // Comfortably past the ramp (see `drag_scroll_rate_is_capped`), so the
    // rate is pinned at its cap and the law does not depend on retuning the
    // dead zone / ramp width.
    let py = H + 200.0;
    let dt = 0.1_f32;

    let mut scroll = ScrollPos::default();
    let mut last_line = 0usize;
    let mut prev_row = 0usize;
    for step in 0..40 {
        let (new_scroll, line, _col) = p
            .drag_scroll_step(scroll, px, py, H, dt)
            .unwrap_or_else(|| panic!("step {step}: pointer beyond the edge must keep scrolling"));
        assert!(
            new_scroll.row >= prev_row,
            "step {step}: scroll must never move backward while held beyond the same edge"
        );
        prev_row = new_scroll.row;
        scroll = new_scroll;
        last_line = line;
    }

    assert!(
        scroll.row > one_screenful_rows,
        "40 held ticks at a pinned-cap rate must scroll well past one screenful \
         ({one_screenful_rows} rows) — settled at row {}",
        scroll.row
    );
    assert!(
        last_line > one_screenful_rows,
        "the hit-tested line under the (unmoved) pointer must land beyond one screenful \
         once the content has scrolled under it — settled at line {last_line}"
    );
}

/// NON-VACUITY WITNESS for the headline law above (CLAUDE.md: prove a law
/// isn't satisfiable by deleting its own subject). Clamping
/// [`drag_scroll_rate`]'s body to unconditionally `return 0.0;` collapses
/// every overshoot to a zero rate, so `drag_scroll_step` declines forever
/// (`rate <= 0.0` is one of its own decline conditions) and the headline
/// law's `unwrap_or_else` panics on its very first step — scroll never
/// advances at all under that mutation. This law pins the floor a rate curve
/// permanently clamped to zero would violate.
#[test]
fn drag_scroll_rate_is_not_vacuously_zero_for_a_real_overshoot() {
    assert!(
        drag_scroll_rate(60.0) > 0.0,
        "a real overshoot must earn a nonzero rate — a rate permanently clamped to zero \
         is exactly the regression this whole file exists to catch"
    );
}
