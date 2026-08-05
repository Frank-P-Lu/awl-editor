//! `testlock` — THE one process-wide reentrant test-serialization guard.
//!
//! awl keeps a fistful of process-GLOBAL state that tests read and write: the
//! active THEME, PAGE mode/measure, the caret look, the debug / hud / peek /
//! menubar flags, the spell / nits / markdown / outline / typewriter /
//! frontmatter sticky globals, the summoned about / lifetime CARD flags, and
//! the swappable fs / trash / socket-dir / cwd backends. `cargo test` runs in
//! parallel, so two tests touching the same global race — and when each global
//! was guarded by its OWN `Mutex` with a documented acquire ORDER, a test that
//! took two of them in the wrong order could ABBA-deadlock (a real 3-way hang
//! lived in the about↔lifetime pair; the theme↔page pair produced the
//! wash-cache/geometry flake).
//!
//! The cure is STRUCTURAL: ONE lock for all of it. Every test — and every
//! `cfg(test)` global WRITER: the `page` measure setters, `apply_transition`'s
//! card-dismissal intercepts, `fs::FsGuard` / `fs::CwdGuard` / `assets`'s
//! `with_trash`, `daemon`'s socket-dir gate — acquires [`serial`]. With a
//! single lock there is no acquire order left to invert, so the ABBA class is
//! UNREPRESENTABLE.
//!
//! The one subtlety a single lock forces is REENTRANCY: a test holds the guard
//! across its whole window and then calls a writer (or drives `apply_transition`,
//! which acquires it too) on the SAME thread — so acquisition is keyed on a
//! thread-local "this thread already holds it" flag, and a nested acquire
//! returns a no-op guard instead of self-deadlocking. Only the OUTERMOST guard
//! owns the release. Poison is absorbed (`into_inner`), mirroring the old
//! raw-mutex convention: a failed assertion in one test must not cascade a
//! poisoned-lock panic into every later one.
//!
//! The cost is COARSER parallelism — every global-touching test now serializes
//! against every other — accepted deliberately (the pure, global-free unit
//! tests still run fully parallel). This is the single owner that replaced the
//! old `theme::TEST_LOCK` / `fs::TEST_LOCK` / `page::test_lock` /
//! `about`+`lifetime` composite / caret / debug / hud / … family.
//!
//! WORLD, PAGE, SPELLCHECK, RENDER-OVERRIDE and (see [`misc`]) every other
//! sticky/summoned-card/picker global CLEANLINESS are checked, not silently
//! imposed. An outermost [`serial`] guard snapshots those globals and fails
//! (after restoring them) if its test window exits dirty. The restore is
//! what makes a leak IMPOSSIBLE rather than merely reported: it also runs while
//! the window is UNWINDING, where the report is suppressed, so a fixture that
//! forces a knob and then panics — or returns early past its own reset — cannot
//! hand that value to the next test — a leaked forced `ListStyle` once made an
//! unrelated jump-hint law in another file report a clip that was not one, green
//! single-threaded and red in a wide parallel run. This is deliberately not the
//! retired ambient `WorldPin-on-serial`: a clean window performs no write, and
//! production code that owns a persistent global write takes [`product`] instead.
//! A nested product acquire under a test guard remains inside the test's checked
//! window, so it cannot punch a hole through the law. `theme::set_active` and the
//! page setters independently acquire this lock at their runtime writer choke
//! points.
//!
//! [`misc`] is the WIDER half of that same audit: a command sweep
//! that applies every `Action` fires every sticky `Toggle`, every summoned
//! card's `CardFlag`, and the caret-mode override as a side effect, and a
//! restore list sized to what the guard's exit audit happened to check —
//! rather than to the sweep's actual reach — is exactly how `debug` leaked ON
//! into the rest of the suite once already. [`misc::pins`], [`misc::leaked`]
//! and [`misc::restore`] share one field list, so the audit and the restore
//! can no longer drift apart the way the world/page/spellcheck fields above
//! and a test's own hand-restore once did.

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

#[cfg(test)]
pub(crate) mod misc;

/// The one mutex behind [`serial`]. Never touched outside this module.
static TEST_MUTEX: Mutex<()> = Mutex::new(());

thread_local! {
    /// True while THIS thread holds [`TEST_MUTEX`] via the OUTERMOST live
    /// [`SerialGuard`] — the reentrancy key.
    static HELD: Cell<bool> = const { Cell::new(false) };
    /// Runtime witness for laws that must prove production requested the
    /// product door, rather than accidentally nesting the checked test door.
    static PRODUCT_REQUESTS: Cell<usize> = const { Cell::new(0) };
}

/// The guard [`serial`] returns. `inner: None` is the reentrant (already-held)
/// case: dropping it releases nothing; only the outermost guard clears the
/// thread flag and unlocks the mutex.
pub(crate) struct SerialGuard {
    inner: Option<MutexGuard<'static, ()>>,
    world_at_entry: Option<usize>,
    page_at_entry: Option<(bool, usize)>,
    spellcheck_at_entry: Option<bool>,
    #[cfg(test)]
    overrides_at_entry: Option<crate::render::overrides::OverridePins>,
    #[cfg(test)]
    misc_at_entry: Option<misc::MiscPins>,
}

impl Drop for SerialGuard {
    fn drop(&mut self) {
        if self.inner.is_some() {
            let mut world_leak = None;
            let mut page_leak = None;
            let mut spellcheck_leak = None;
            if let Some(before) = self.world_at_entry {
                let after = crate::theme::active_index();
                if after != before {
                    // Clean before reporting, so one failed test cannot poison
                    // whatever the harness schedules next.
                    crate::theme::set_active(before);
                    if !std::thread::panicking() {
                        world_leak = Some((before, after));
                    }
                }
            }
            if let Some((on_before, measure_before)) = self.page_at_entry {
                let on_after = crate::page::page_on();
                let measure_after = crate::page::measure();
                if (on_after, measure_after) != (on_before, measure_before) {
                    // As with worlds, restore before reporting. In particular,
                    // a capture test that returns early or unwinds after shaping
                    // at a custom measure must not poison its next reader.
                    crate::page::set_page_on(on_before);
                    crate::page::set_measure(measure_before);
                    if !std::thread::panicking() {
                        page_leak = Some(((on_before, measure_before), (on_after, measure_after)));
                    }
                }
            }
            if let Some(before) = self.spellcheck_at_entry {
                let after = crate::spell::spellcheck_on();
                if after != before {
                    crate::spell::set_spellcheck_on(before);
                    if !std::thread::panicking() {
                        spellcheck_leak = Some((before, after));
                    }
                }
            }
            #[cfg(test)]
            let mut override_leak = None;
            #[cfg(test)]
            if let Some(before) = self.overrides_at_entry.take() {
                let after = crate::render::overrides::pins();
                let leaked = crate::render::overrides::leaked_knobs(&before, &after);
                if !leaked.is_empty() {
                    // Restore before reporting, exactly as above — and note that
                    // this arm runs on the UNWINDING path too, where reporting is
                    // suppressed. That is the whole point: a fixture that forces a
                    // knob and then dies cannot hand its forced value to the next
                    // test in another file.
                    crate::render::overrides::restore_pins(&before);
                    if !std::thread::panicking() {
                        override_leak = Some(leaked);
                    }
                }
            }
            #[cfg(test)]
            let mut misc_leak = None;
            #[cfg(test)]
            if let Some(before) = self.misc_at_entry.take() {
                let after = misc::pins();
                let leaked = misc::leaked(&before, &after);
                if !leaked.is_empty() {
                    // Restore before reporting, same reasoning as the render
                    // overrides above: this arm runs on the UNWINDING path too,
                    // so a fixture that flips a toggle (a full command sweep
                    // fires EVERY one of them) and then dies cannot hand its
                    // value to the next test in another file.
                    misc::restore(&before);
                    if !std::thread::panicking() {
                        misc_leak = Some(leaked);
                    }
                }
            }
            HELD.with(|h| h.set(false));
            if let Some((before, after)) = world_leak {
                panic!(
                    "test left the active world dirty: entered at {} ({}) and exited at {} ({})",
                    before,
                    crate::theme::THEMES[before].name,
                    after,
                    crate::theme::THEMES[after].name
                );
            }
            if let Some(((on_before, measure_before), (on_after, measure_after))) = page_leak {
                panic!(
                    "test left page inputs dirty: entered at page_on={} measure={} \
                     and exited at page_on={} measure={}",
                    on_before, measure_before, on_after, measure_after
                );
            }
            if let Some((before, after)) = spellcheck_leak {
                panic!(
                    "test left spellcheck dirty: entered at {} and exited at {}",
                    before, after
                );
            }
            #[cfg(test)]
            if let Some(leaked) = override_leak {
                panic!("test left render overrides dirty: {}", leaked.join("; "));
            }
            #[cfg(test)]
            if let Some(leaked) = misc_leak {
                panic!("test left misc globals dirty: {}", leaked.join("; "));
            }
        }
    }
}

/// Acquire THE process-wide test-serialization lock: blocks until free, absorbs
/// poison, and is REENTRANT per thread (a nested acquire on a thread that
/// already holds it returns a no-op guard instead of self-deadlocking). The
/// ONLY door to the mutex.
pub(crate) fn serial() -> SerialGuard {
    acquire(true)
}

/// The same mutex and reentrancy, for a production function whose active-world
/// write is its result (the theme preview in `actions::apply_transition`). Only an
/// OUTERMOST product window skips the test-exit cleanliness check; nested under
/// [`serial`], the enclosing test window still owns and checks the outcome.
pub(crate) fn product() -> SerialGuard {
    PRODUCT_REQUESTS.with(|n| n.set(n.get() + 1));
    acquire(false)
}

#[cfg(test)]
pub(crate) fn product_requests() -> usize {
    PRODUCT_REQUESTS.with(Cell::get)
}

fn acquire(check_world: bool) -> SerialGuard {
    if HELD.with(|h| h.get()) {
        return SerialGuard {
            inner: None,
            world_at_entry: None,
            page_at_entry: None,
            spellcheck_at_entry: None,
            #[cfg(test)]
            overrides_at_entry: None,
            #[cfg(test)]
            misc_at_entry: None,
        };
    }
    let guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    HELD.with(|h| h.set(true));
    SerialGuard {
        inner: Some(guard),
        world_at_entry: check_world.then(crate::theme::active_index),
        page_at_entry: check_world.then(|| (crate::page::page_on(), crate::page::measure())),
        spellcheck_at_entry: check_world.then(crate::spell::spellcheck_on),
        // Snapshot AFTER the flag is set: `pins` reads through the ordinary
        // doors, which assert the hold.
        #[cfg(test)]
        overrides_at_entry: check_world.then(crate::render::overrides::pins),
        #[cfg(test)]
        misc_at_entry: check_world.then(misc::pins),
    }
}

/// True iff THIS thread currently holds the guard (via a live [`serial`]
/// guard). For the law tests below.
pub(crate) fn currently_held() -> bool {
    HELD.with(|h| h.get())
}

#[cfg(test)]
mod tests;
