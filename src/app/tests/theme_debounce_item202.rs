//! ITEM 202 — the theme-picker MOVEMENT-LATENCY regression, repair round.
//!
//! **The bug (item 37b, 2026-07-24 — item 202's own first landing didn't fix
//! this, only re-tuned it):** a FLAT trailing-only debounce forces one number
//! to answer two different questions. At `THEME_FONT_DEBOUNCE_DEFAULT_MS == 0`
//! (37b), `debounce_due`'s `now - dirty >= 0` is trivially true at the SAME
//! instant `theme_font_at` is stamped, so EVERY preview step reshapes —
//! isolated steps stay fast (~39ms, matching 37b's own measured figure) but a
//! rapid run pays the reshape once per step and later steps queue behind
//! earlier ones on the single main thread (measured live: p95 movement-latency
//! rose to 18.9ms over a 6-step, 30ms-apart burst). Raising the flat window to
//! 100ms (this item's first landing) coalesces the burst (one reshape, not
//! six) but punishes the ISOLATED case too — settle rises to ~124ms, reopening
//! the exact "felt theme-switch freeze" 37b's own commit was about.
//!
//! **The fix:** `crate::app::theme_font_debounce::theme_font_reshape_decision`
//! replaces the flat window with LEADING-EDGE-PLUS-TRAILING-COALESCE (that
//! module's own doc has the pure-function mechanism and its four exhaustive
//! laws — session start, the `>=` window boundary, inside-window coalescing,
//! and "never re-enter Immediate while one is pending"). This file covers what
//! that module cannot: the END-TO-END wiring through the REAL `App` scheduling
//! body, and the shared window constant's own same-tick structural law.
//!
//! LIVE evidence this round is built against (measured 2026-08-01,
//! `--live-script`, real compositor, release build, screen confirmed unlocked
//! at capture time): isolated single-step settle ~39ms at a flat 0ms window
//! (37b's own untouched fast path); a 30ms-apart six-step burst at a flat
//! 100ms window produced exactly ONE trailing reshape with colors-only
//! movement-latency p50=5.1ms/p95=8.9ms (the coalescing half of this fix,
//! already proven live); the isolated-vs-burst leading-edge SPLIT itself is a
//! same-day repair on top of that live-proven coalescing and is verified here
//! at the purest reachable seam, since a live re-measurement needs an unlocked
//! display this session did not have throughout (`docs/harness-reach.md`).
//!
//! This file drives the REAL `about_to_wait_impl` scheduling body
//! (`App::step_scheduling`) under a `VirtualClock`, exactly like
//! `which_key.rs`'s frame-loop law. `App::apply_deferred_theme_font` stamps
//! `theme_font_last_reshape_at` unconditionally, before its GPU-gated reshape
//! call, so the trailing half of the mechanism is exercised for real here,
//! not stood in for.

use super::*;
use crate::clock::Clock as _;

/// Mirrors exactly what `retint_theme_preview`'s GPU-gated block does with the
/// decision — the ONLY thing this skips is the real `sync_theme_font()`/
/// `sync_theme_colors()` GPU calls, which have no bearing on the scheduling
/// question these laws pin (a hermetic `App` has no `gpu`; see the module doc).
fn simulate_preview_step(app: &mut App, now: crate::clock::Instant) -> ThemeFontReshapeDecision {
    let decision = theme_font_reshape_decision(
        app.theme_font_at,
        app.theme_font_last_reshape_at,
        now,
        theme_font_debounce(),
    );
    match decision {
        ThemeFontReshapeDecision::Immediate => {
            app.theme_font_last_reshape_at = Some(now);
            app.theme_font_at = None;
        }
        ThemeFontReshapeDecision::Coalesce => {
            app.theme_font_at = Some(now);
        }
    }
    decision
}

// ── End-to-end: an isolated step vs. a rapid burst, driven through the real
//    scheduling body (`App::step_scheduling`) under a `VirtualClock` ────────

#[test]
fn an_isolated_step_reshapes_immediately_and_never_schedules_a_trailing_settle() {
    let _serial = crate::testlock::serial();
    let clock = crate::clock::VirtualClock::new();
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.set_clock(Box::new(clock.clone()));
    let sched = RecordingScheduler::new();

    clock.advance_ms(1000); // nothing has ever reshaped; a lone tap
    let decision = simulate_preview_step(&mut app, clock.now());
    assert_eq!(
        decision,
        ThemeFontReshapeDecision::Immediate,
        "an isolated step must reshape now — this is item 37b's untouched \
         ~39ms fast path, and it must not regress back to the flat-debounce \
         shape that punished every step equally"
    );
    assert!(
        app.theme_font_at.is_none(),
        "an immediate reshape leaves nothing pending"
    );

    // Nothing further arrives. The scheduling body must find nothing to do —
    // a SECOND reshape here would silently double the isolated-step cost.
    sched.begin_step();
    app.step_scheduling(&sched);
    assert!(
        app.theme_font_at.is_none(),
        "an isolated step's settle must not schedule a redundant trailing reshape"
    );
}

#[test]
fn a_rapid_burst_pays_one_leading_reshape_and_one_trailing_settle_not_six() {
    let _serial = crate::testlock::serial();
    let clock = crate::clock::VirtualClock::new();
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.set_clock(Box::new(clock.clone()));
    let sched = RecordingScheduler::new();

    // Six steps, 30ms apart — the exact live-measured burst cadence (module
    // doc). Track how many steps landed `Immediate` vs `Coalesce`.
    let mut immediate = 0u32;
    let mut coalesced = 0u32;
    for _ in 0..6 {
        clock.advance_ms(30);
        match simulate_preview_step(&mut app, clock.now()) {
            ThemeFontReshapeDecision::Immediate => immediate += 1,
            ThemeFontReshapeDecision::Coalesce => coalesced += 1,
        }
        sched.begin_step();
        app.step_scheduling(&sched);
    }
    assert_eq!(
        (immediate, coalesced),
        (1, 5),
        "exactly the FIRST step of a burst is isolated (nothing preceded it); \
         every step after it must coalesce, not reshape again — a regression \
         to the flat 0ms shape makes every step `Immediate` (6, 0); a \
         regression to a flat trailing-only shape makes every step `Coalesce` \
         (0, 6)"
    );
    assert!(
        app.theme_font_at.is_some(),
        "the burst's tail must still be pending immediately after its last step"
    );

    // Let the trailing window fully elapse with no further input.
    clock.advance_ms(theme_font_debounce().as_millis() as u64 + 1);
    sched.begin_step();
    app.step_scheduling(&sched);
    assert!(
        app.theme_font_at.is_none(),
        "the coalesced tail must settle once the burst goes quiet"
    );
    assert!(
        app.theme_font_last_reshape_at.is_some(),
        "the trailing settle counts as a real reshape for the NEXT step's leading-edge decision"
    );
    // Total reshapes for this 6-step burst: 1 (leading) + 1 (trailing) = 2 —
    // dramatically better than the old flat-0ms shape's 6, and the isolated
    // step's own cost (tested above) is completely unaffected by this burst.
}

// ── The scheduling constant itself ──────────────────────────────────────────

/// STRUCTURAL regression pin: the debounce window itself must be large enough
/// that `debounce_due` cannot be satisfied on the SAME instant it was armed —
/// the trailing half of the mechanism still needs this (it is what makes a
/// burst's tail coalesce at all rather than firing every step, same as
/// `theme_font_debounce::a_reshape_at_or_past_the_window_reads_isolated`'s own
/// boundary law for the leading-edge half). A nonzero
/// `THEME_FONT_DEBOUNCE_DEFAULT_MS` is ALSO a compile-time invariant (`app.rs`'s
/// own `const _: () = assert!(..)`); this law covers the non-const half — the
/// real, env-overridable predicate.
#[test]
fn the_default_theme_font_debounce_cannot_fire_on_the_same_tick_it_was_armed() {
    let now = crate::clock::VirtualClock::new().now();
    assert!(
        !debounce_due(now, theme_font_debounce(), now),
        "the default debounce fired at elapsed=0 — a same-tick-satisfiable \
         window makes every step read as `Coalesce`-vs-`Immediate` on identical \
         timestamps meaningless, and collapses the trailing coalesce back into \
         firing every step"
    );
}
