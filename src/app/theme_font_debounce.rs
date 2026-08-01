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

/// A step is ISOLATED — reshape [`ThemeFontReshapeDecision::Immediate`] —
/// when no reshape is currently `pending` AND either nothing has reshaped yet
/// or the last reshape is at least `window` old; otherwise it is a burst
/// continuation and must [`ThemeFontReshapeDecision::Coalesce`] (never
/// re-enter the immediate path while one is already pending, and never treat
/// two reshapes closer than `window` apart as two separate isolated steps).
pub(crate) fn theme_font_reshape_decision(
    pending: Option<Instant>,
    last_reshape_at: Option<Instant>,
    now: Instant,
    window: Duration,
) -> ThemeFontReshapeDecision {
    let isolated = pending.is_none()
        && last_reshape_at.is_none_or(|t| now.saturating_duration_since(t) >= window);
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

    #[test]
    fn nothing_pending_and_no_prior_reshape_is_isolated() {
        let t = crate::clock::VirtualClock::new().now();
        assert_eq!(
            theme_font_reshape_decision(None, None, t, Duration::from_millis(100)),
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
            theme_font_reshape_decision(None, Some(last), now, window),
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
            theme_font_reshape_decision(None, Some(last), now, window),
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
            theme_font_reshape_decision(Some(pending_since), Some(ancient), now, window),
            ThemeFontReshapeDecision::Coalesce,
            "a step must coalesce whenever one is already pending, regardless of \
             how long ago the last REAL reshape was"
        );
    }
}
