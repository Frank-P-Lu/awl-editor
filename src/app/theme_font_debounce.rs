//! ITEM 202's LEADING-EDGE-PLUS-TRAILING-COALESCE rule for the theme-picker
//! preview's deferred font reshape. Extracted out of `app.rs` (2026-08-01
//! review): this is a pure scheduling decision — no `self`, no GPU, no clock
//! of its own — and belongs to the scheduling seam, not to `App`.
//!
//! A flat trailing-only debounce window forces one number to answer two
//! different questions. At `0` (item 37b) every step reshapes: fast for an
//! isolated step (~39ms, item 37b's own measured figure) but a rapid run
//! queues behind itself on the single main thread (measured live: p95
//! movement-latency 18.9ms over a 30ms-apart six-step burst). Raising the flat
//! window to `100` (this item's first landing) coalesces the burst to one
//! reshape, but punishes the isolated case too (~124ms), reopening the exact
//! "felt theme-switch freeze" item 37b's own commit was about.
//!
//! [`theme_font_reshape_decision`] answers the two questions separately: an
//! ISOLATED step (nothing has reshaped in the last `window`, and nothing is
//! already pending) reshapes IMMEDIATELY — item 37b's ~39ms path, untouched. A
//! step arriving WITHIN `window` of the last reshape, or while one is already
//! pending, is a BURST CONTINUATION and coalesces into the caller's own
//! trailing debounce instead — an N-step burst pays one leading reshape plus
//! one trailing settle, never N and never the isolated case's cost either.
//!
//! The caller (`App::retint_theme_preview`) owns the GPU-gated execution of
//! whichever verdict this returns, and `App::theme_font_last_reshape_at` is
//! the leading edge's own clock — stamped by the immediate path, a
//! commit/revert (`App::retint_theme_now`), and a coalesced settle firing
//! (`App::apply_deferred_theme_font`) alike. `window` is the caller's
//! `theme_font_debounce()` (`app.rs`; `AWL_THEME_FONT_DEBOUNCE_MS` is its A/B
//! override, governing both halves of this rule at once — see docs/fonts.md
//! for the live before/after numbers on both axes).
//!
//! ## THE TEST READS THE WORK, NOT THE CLOCK ALONE
//!
//! The rule above once answered "should this step wait?" from elapsed time by
//! itself, against a cost model that was a CONSTANT calibrated when an isolated
//! reshape cost ~39ms. It is not a constant. Measured on one machine
//! (`--bench-theme-burst`, release, 2026-08-03) one `sync_theme_font` costs
//! **0.2ms** on a seven-line document, **12.0ms** on CLAUDE.md and **24.4ms**
//! on a 1896-line fixture — a hundredfold spread the window cannot see. On the
//! cheap end the arithmetic is absurd: a user arrowing Kite→Mulga measured a
//! **105.4ms** settle over **0.2ms** of reshape work (live, `--live-script`,
//! release; the headline tracked `AWL_THEME_FONT_DEBOUNCE_MS` one-for-one at
//! 50/100/250ms, so ~96% of the transaction was deliberate waiting). Coalescing
//! bought nothing there — there was nothing to coalesce.
//!
//! So the leading-edge test takes `last_reshape_cost`: the RESHAPE-SIDE work the
//! previous real reshape actually cost, measured by the same
//! `sync_theme_font_timed` door every live reshape runs through
//! (`SwitchPhases::reshape_side_ms`). A reshape measured CHEAPER than
//! `cheap_reshape` reads ISOLATED however recently it happened, because the
//! queueing hazard the coalesce exists to avoid does not exist at that cost.
//! Anything dearer — or, decisively, anything **unmeasured** (`None`) — keeps
//! the elapsed-time rule above exactly as written. Silence is never read as
//! cheap: an unmeasured reshape is the conservative case and coalesces.
//!
//! **This does not undo the burst coalescing.** The regression a zero window
//! caused was N *expensive* reshapes queueing on the one main thread; the cheap
//! gate cannot reach that case, because the cost that opens it is the very cost
//! that makes queueing harmless. A document whose reshape is worth deferring
//! still pays exactly one leading reshape plus one trailing settle for an N-step
//! run, and `--bench-theme-burst`'s reshape-count witness is unmoved (the bench
//! drives `TextPipeline` directly and never consults this rule at all).

use crate::clock::Instant;
use std::time::Duration;

/// The outcome of [`theme_font_reshape_decision`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemeFontReshapeDecision {
    /// Reshape right now — an isolated step, nothing recent to coalesce with.
    Immediate,
    /// Fold into the pending/sliding trailing debounce — a burst continuation.
    Coalesce,
}

/// A step is ISOLATED — reshape [`ThemeFontReshapeDecision::Immediate`] — when
/// no reshape is currently `pending` AND the last reshape is either absent, at
/// least `window` old, or **measured cheaper than `cheap_reshape`**; otherwise
/// it is a burst continuation and must [`ThemeFontReshapeDecision::Coalesce`]
/// (never re-enter the immediate path while one is already pending, and never
/// treat two reshapes closer than `window` apart as two separate isolated
/// steps unless the work itself says there is nothing to save).
///
/// `pending.is_none()` is a HARD precondition of every isolated verdict,
/// including the cheap one: two steps racing into the immediate path while a
/// deferral is in flight would reshape twice and leave the deferral to reshape
/// a third time. `last_reshape_cost` of `None` means UNMEASURED, and never
/// opens the cheap gate — see the module doc.
pub(crate) fn theme_font_reshape_decision(
    pending: Option<Instant>,
    last_reshape_at: Option<Instant>,
    last_reshape_cost: Option<Duration>,
    now: Instant,
    window: Duration,
    cheap_reshape: Duration,
) -> ThemeFontReshapeDecision {
    let cooled_down = last_reshape_at.is_none_or(|t| now.saturating_duration_since(t) >= window);
    let nothing_to_save = last_reshape_cost.is_some_and(|cost| cost < cheap_reshape);
    let isolated = pending.is_none() && (cooled_down || nothing_to_save);
    if isolated {
        ThemeFontReshapeDecision::Immediate
    } else {
        ThemeFontReshapeDecision::Coalesce
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock as _;

    /// The cost threshold every law here reads (`app.rs`'s own default), so a
    /// change to the constant moves the laws with it instead of past them.
    fn cheap() -> Duration {
        crate::app::theme_font_cheap_reshape()
    }

    /// An EXPENSIVE last reshape — measured, and dear enough that coalescing is
    /// worth its wait. `--bench-theme-burst` measures 12.0ms on CLAUDE.md and
    /// 24.4ms on a 1896-line fixture; this is the smaller of the two.
    fn expensive() -> Option<Duration> {
        Some(Duration::from_micros(12_000))
    }

    /// A CHEAP last reshape — the seven-line probe fixture's own measured 0.2ms,
    /// the case that produced a 105.4ms live settle over 0.2ms of work.
    fn measured_cheap() -> Option<Duration> {
        Some(Duration::from_micros(200))
    }

    #[test]
    fn nothing_pending_and_no_prior_reshape_is_isolated() {
        let t = crate::clock::VirtualClock::new().now();
        assert_eq!(
            theme_font_reshape_decision(None, None, None, t, Duration::from_millis(100), cheap()),
            ThemeFontReshapeDecision::Immediate,
            "the very first preview step of a session has nothing to coalesce with"
        );
    }

    #[test]
    fn a_reshape_at_or_past_the_window_reads_isolated() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        let window = Duration::from_millis(100);
        clock.advance_ms(window.as_millis() as u64); // exactly at the boundary
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(None, Some(last), expensive(), now, window, cheap()),
            ThemeFontReshapeDecision::Immediate,
            "`debounce_due`'s own `>=` semantics say the boundary itself counts as \
             isolated — this law pins that the leading-edge check agrees"
        );
    }

    #[test]
    fn a_reshape_inside_the_window_is_a_burst_continuation() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        let window = Duration::from_millis(100);
        clock.advance_ms(window.as_millis() as u64 - 1);
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(None, Some(last), expensive(), now, window, cheap()),
            ThemeFontReshapeDecision::Coalesce,
            "one millisecond short of the window must still coalesce, not reshape again"
        );
    }

    #[test]
    fn a_pending_coalesce_never_re_enters_the_immediate_path() {
        // Even if the LAST REAL reshape is ancient, a step that arrives while one
        // is already PENDING must coalesce — two steps racing to both go
        // "immediate" would defeat the whole point (two reshapes instead of one).
        let clock = crate::clock::VirtualClock::new();
        let window = Duration::from_millis(100);
        let ancient = clock.now();
        clock.advance_ms(10 * window.as_millis() as u64);
        let pending_since = clock.now();
        clock.advance_ms(1);
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(
                Some(pending_since),
                Some(ancient),
                expensive(),
                now,
                window,
                cheap()
            ),
            ThemeFontReshapeDecision::Coalesce,
            "a step must coalesce whenever one is already pending, regardless of \
             how long ago the last REAL reshape was"
        );
    }

    // ── The cost input ──────────────────────────────────────────────────────

    /// THE REPORTED CASE: a step arriving deep inside the
    /// window, whose previous reshape was MEASURED at 0.2ms. The elapsed clock
    /// says "burst continuation"; the work says there is nothing to save by
    /// waiting, and the work wins — otherwise this step buys a 100ms trailing
    /// settle for 0.2ms of avoided reshaping, which is the whole defect.
    #[test]
    fn a_measured_cheap_reshape_makes_the_next_step_immediate_inside_the_window() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        let window = Duration::from_millis(100);
        clock.advance_ms(30); // the live-measured arrowing cadence
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(None, Some(last), measured_cheap(), now, window, cheap()),
            ThemeFontReshapeDecision::Immediate,
            "a 0.2ms reshape 30ms ago is not a queueing hazard; deferring this step \
             costs the user the whole trailing window to save nothing"
        );
    }

    /// THE OTHER HALF, and the one that keeps the burst coalescing intact: the
    /// SAME timing with an EXPENSIVE measured cost must still coalesce. Swept
    /// across the whole interesting cost axis rather than one hand-picked pair —
    /// every cost at or
    /// above the threshold coalesces, every cost below it goes immediate, and the
    /// boundary itself belongs to the coalescing side (`<`, not `<=`).
    #[test]
    fn the_cheap_gate_opens_strictly_below_the_threshold_and_nowhere_above() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        let window = Duration::from_millis(100);
        clock.advance_ms(30);
        let now = clock.now();
        let threshold = cheap();
        let micros = threshold.as_micros() as u64;
        // 0.05x .. 4x the threshold in fortieths, plus the boundary exactly.
        let mut costs: Vec<u64> = (1..=80).map(|k| micros * k / 20).collect();
        costs.push(micros);
        for c in costs {
            let cost = Duration::from_micros(c);
            let want = if cost < threshold {
                ThemeFontReshapeDecision::Immediate
            } else {
                ThemeFontReshapeDecision::Coalesce
            };
            assert_eq!(
                theme_font_reshape_decision(None, Some(last), Some(cost), now, window, threshold),
                want,
                "a {c}us reshape against a {micros}us threshold decided the wrong way"
            );
        }
    }

    /// UNMEASURED IS NOT CHEAP. `None` is what every path that could not time its
    /// reshape reports — no GPU, a headless `App`, a reshape that found no work.
    /// Reading silence as "cheap" would send every such step down the immediate
    /// path and hand the N-reshape burst straight back.
    #[test]
    fn an_unmeasured_reshape_cost_keeps_the_elapsed_time_rule() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        let window = Duration::from_millis(100);
        clock.advance_ms(30);
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(None, Some(last), None, now, window, cheap()),
            ThemeFontReshapeDecision::Coalesce,
            "an unmeasured cost must fall back to the elapsed-time rule, never open \
             the cheap gate"
        );
    }

    /// The cheap gate does NOT outrank the pending guard. A cheap measured cost
    /// while a deferral is already in flight must still coalesce: the pending
    /// reshape is going to happen anyway, and reshaping now as well would do the
    /// same work twice.
    #[test]
    fn a_cheap_cost_still_yields_to_a_pending_deferral() {
        let clock = crate::clock::VirtualClock::new();
        let last = clock.now();
        clock.advance_ms(10);
        let pending_since = clock.now();
        clock.advance_ms(20);
        let now = clock.now();
        assert_eq!(
            theme_font_reshape_decision(
                Some(pending_since),
                Some(last),
                measured_cheap(),
                now,
                Duration::from_millis(100),
                cheap()
            ),
            ThemeFontReshapeDecision::Coalesce,
            "`pending.is_none()` is a hard precondition of EVERY isolated verdict"
        );
    }
}
