//! `toggle` — the ONE process-global boolean-flag mechanism behind awl's sticky
//! on/off preferences (outline, spellcheck, writing nits, the debug panel,
//! typewriter scroll, file visibility, page mode, the menu bar, the format
//! popover, reduce-motion, code ligatures, the which-key force-show probe, the
//! streaks card's page flip, …). Each of those modules used to hand-roll an
//! identical `AtomicBool` + relaxed-load/store pair, each with its OWN copy
//! (or its own ABSENCE) of the `testlock` write discipline. [`Toggle`] is that
//! one mechanism; every consumer still owns its OWN verb-named public surface
//! (`outline_on`/`set_outline_on`/`toggle`, …) and its own default, doc
//! comment, and tests — only the storage + guard collapse to one place.
//!
//! [`Toggle::set`] / [`Toggle::toggle`] ASSERT the caller already holds
//! `crate::testlock::serial()` (mirrors `theme::set_active`, NOT
//! `fs::active`): the write itself never blocks, and a violation panics BY
//! NAME instead of racing silently. [`Toggle::on`] is never guarded — a read
//! never asserts, matching `render::overrides`'s reasoning (see that module's
//! `TEST_OVERRIDE` doc): reads vastly outnumber the tests that actually flip a
//! flag, so guarding them would demand a far wider retrofit than this
//! mechanism is worth.
//!
//! A caller that must hold the lock across MORE than one write in a row (e.g.
//! `menubar::set_menu_bar_on` clearing the open dropdown in the same window it
//! flips the bar off, or `page::widen` reading-then-writing the measure) takes
//! its OWN `crate::testlock::serial()` guard first, exactly as it did before
//! this module existed — [`Toggle`]'s assert then finds the lock already held
//! (the guard is reentrant) rather than taking it itself, so the multi-step
//! window stays one continuous acquisition.

use std::sync::atomic::{AtomicBool, Ordering};

/// A process-global boolean flag: the identical `AtomicBool` + relaxed
/// load/store shape awl's sticky on/off preferences all shared before this
/// module existed.
pub(crate) struct Toggle(AtomicBool);

impl Toggle {
    /// A flag fixed at `default` until the first write.
    pub(crate) const fn new(default: bool) -> Self {
        Toggle(AtomicBool::new(default))
    }

    /// The current value. Never guarded — see the module doc.
    pub(crate) fn on(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// ASSERT-guarded write (mirrors `theme::set_active`): panics under
    /// `cfg(test)` if the caller isn't already holding `testlock::serial()`.
    pub(crate) fn set(&self, on: bool) {
        #[cfg(test)]
        assert!(
            crate::testlock::currently_held(),
            "a Toggle was written without holding crate::testlock::serial()"
        );
        self.0.store(on, Ordering::Relaxed);
    }

    /// ASSERT-guarded flip, returning the new value.
    pub(crate) fn toggle(&self) -> bool {
        #[cfg(test)]
        assert!(
            crate::testlock::currently_held(),
            "a Toggle was flipped without holding crate::testlock::serial()"
        );
        let next = !self.on();
        self.0.store(next, Ordering::Relaxed);
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_reads_the_constructed_default() {
        let t = Toggle::new(true);
        assert!(t.on());
        let f = Toggle::new(false);
        assert!(!f.on());
    }

    #[test]
    fn set_writes_and_toggle_flips_and_reports_the_new_value() {
        let _g = crate::testlock::serial();
        let t = Toggle::new(false);
        t.set(true);
        assert!(t.on());
        assert!(!t.toggle(), "flips to off and reports the new value");
        assert!(!t.on());
        assert!(t.toggle(), "flips back to on and reports the new value");
        assert!(t.on());
    }

    #[test]
    fn a_caller_holding_the_lock_first_reenters_cleanly() {
        // Models `menubar::set_menu_bar_on` / `page::widen`: the caller takes
        // its OWN outer guard, then calls `set`/`toggle` (which would try to
        // take it again if it asserted-and-acquired) — the reentrant guard
        // must make this a no-op, never a self-deadlock.
        let _outer = crate::testlock::serial();
        let t = Toggle::new(false);
        t.set(true);
        t.toggle();
        assert!(!t.on());
    }

    #[test]
    #[should_panic(expected = "a Toggle was written without holding crate::testlock::serial()")]
    fn set_off_guard_panics_by_name() {
        // Deliberately no `crate::testlock::serial()` guard: the non-vacuous
        // proof that `set` really enforces the discipline it claims to.
        let t = Toggle::new(false);
        t.set(true);
    }

    #[test]
    #[should_panic(expected = "a Toggle was flipped without holding crate::testlock::serial()")]
    fn toggle_off_guard_panics_by_name() {
        let t = Toggle::new(false);
        t.toggle();
    }
}
