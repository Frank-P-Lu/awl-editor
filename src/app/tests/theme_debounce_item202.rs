//! ITEM 202 — the theme-picker MOVEMENT-LATENCY regression. The shared
//! `theme_font_at` debounce (`app.rs`'s `theme_font_debounce`), shipped at
//! `THEME_FONT_DEBOUNCE_DEFAULT_MS == 0` (item 37b), let `debounce_due` fire on
//! the SAME `about_to_wait` pass that armed it — `now - dirty >= 0` is
//! trivially true at elapsed 0 — collapsing "instant colors, ONE deferred
//! reshape at rest" back into "colors + a full reshape, synchronously, every
//! single navigation step" for every input modality that funnels through
//! `App::retint_theme_preview` (keyboard nav, pointer hover, wheel — one
//! shared owner; `docs/render.md`, `docs/fonts.md`).
//!
//! LIVE evidence (measured 2026-08-01 via `--live-script` against the real
//! compositor, real release binary, a six-step rapid burst 30ms apart — the
//! real held-arrow-key / fast-wheel-sweep cadence, on both an ambient world
//! (Mangrove) and a static one (Saltpan), so the effect is not ambient-tick
//! contention): at 0ms, 6 steps produced 6 separate `deferred_reshape applied`
//! events and later steps queued behind earlier ones (p95 movement-latency
//! rose to 18.9ms, worse than an isolated single step's 12ms); at 100ms, the
//! SAME burst produced exactly ONE reshape at rest and every step's
//! colors-only present stayed fast and consistent (p50 5.1ms, p95 8.9ms — both
//! LOWER than 0ms's numbers, because no step queued behind a previous step's
//! in-flight reshape). The wheel path reproduced the identical pattern (4
//! reshapes over 6 wheel notches at 0ms; 1 at 100ms) — confirming the shared
//! owner is genuinely shared, not keyboard-specific.
//!
//! This is a live-only demonstration (`docs/harness-reach.md`: a real GPU
//! window and compositor). The two laws below pin the exact SCHEDULING
//! MECHANISM responsible, at the purest reachable seam. The first drives the
//! REAL `about_to_wait_impl` body (`App::step_scheduling`) under a
//! `VirtualClock`, exactly like `which_key.rs`'s frame-loop law — `theme_font_at`
//! is stamped directly (mirroring `retint_theme_preview`'s own re-stamp) rather
//! than through a live GPU-gated preview, because the SCHEDULING DECISION
//! downstream of that stamp has no GPU dependency of its own
//! (`App::apply_deferred_theme_font` degrades to a harmless no-op on the
//! reshape/redraw calls when `gpu` is `None`, so only the timing is exercised
//! here — the real reshape work stays live-only). The second is a pure-function
//! regression pin on the constant itself.

use super::*;
use crate::clock::Clock as _;

/// A rapid theme-preview BURST — steps closer together than the debounce —
/// must coalesce to exactly ONE deferred reshape settle, fired only once the
/// burst goes quiet, never once per step.
#[test]
fn a_rapid_preview_burst_coalesces_to_one_settle_not_one_per_step() {
    let _serial = crate::testlock::serial();
    let clock = crate::clock::VirtualClock::new();
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    app.set_clock(Box::new(clock.clone()));
    let sched = RecordingScheduler::new();

    // Six steps, 30ms apart — the real burst cadence the live trace above
    // measured (a held arrow key / a fast wheel sweep). Each RE-STAMPS
    // `theme_font_at`, mirroring `retint_theme_preview`'s own re-stamp on every
    // further preview step (docs/fonts.md: "each further preview step
    // RE-STAMPS `theme_font_at`, sliding the deadline").
    let mut settled_mid_burst = 0u32;
    for _ in 0..6 {
        clock.advance_ms(30);
        app.theme_font_at = Some(clock.now());
        sched.begin_step();
        app.step_scheduling(&sched);
        if app.theme_font_at.is_none() {
            settled_mid_burst += 1;
        }
    }
    assert_eq!(
        settled_mid_burst, 0,
        "a step arriving inside the debounce window must not settle early — \
         every re-stamp should still find the deadline pending. At \
         THEME_FONT_DEBOUNCE_DEFAULT_MS == 0 this fails on the very first \
         step: `debounce_due` is trivially true at elapsed 0, so the deferred \
         reshape fires synchronously on every step instead of coalescing \
         (item 202's actual regression)."
    );
    assert!(
        app.theme_font_at.is_some(),
        "the burst must still be pending immediately after its last step"
    );

    // Let the debounce window fully elapse with no further input.
    clock.advance_ms(theme_font_debounce().as_millis() as u64 + 1);
    sched.begin_step();
    app.step_scheduling(&sched);
    assert!(
        app.theme_font_at.is_none(),
        "the deferred reshape must settle once the burst goes quiet"
    );
}

/// STRUCTURAL regression pin for item 202's actual defect: the debounce
/// constant itself must be large enough that `debounce_due` cannot be
/// satisfied on the SAME instant it was armed — the exact condition that let
/// the old `THEME_FONT_DEBOUNCE_DEFAULT_MS == 0` default apply the deferred
/// reshape synchronously inside the very `about_to_wait` pass that stamped
/// `theme_font_at`, before that step's own colors-only redraw ever reached
/// `RedrawRequested`. `THEME_FONT_DEBOUNCE_DEFAULT_MS > 0` itself is a
/// compile-time invariant (`app.rs`'s own `const _: () = assert!(..)`, right
/// beside the constant — a reversion to 0 fails the BUILD); this law covers
/// the half that is NOT a compile-time fact — that the real predicate,
/// against the real (env-overridable) `theme_font_debounce()`, genuinely does
/// not fire at elapsed 0 — with no live compositor needed to observe the
/// stutter.
#[test]
fn the_default_theme_font_debounce_cannot_fire_on_the_same_tick_it_was_armed() {
    let now = crate::clock::VirtualClock::new().now();
    assert!(
        !debounce_due(now, theme_font_debounce(), now),
        "the default debounce fired at elapsed=0 — this is item 202's exact \
         defect: a same-tick-satisfiable debounce collapses the deferred \
         reshape back into the colors-only redraw it was split off from, \
         blocking every navigation step's own present behind a full reshape"
    );
}
