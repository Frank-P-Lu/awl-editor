//! `toggle` — the ONE process-global boolean-flag mechanism behind awl's sticky
//! on/off preferences (outline, spellcheck, writing nits, the debug panel,
//! typewriter scroll, file visibility, page mode, the menu bar, the format
//! popover, reduce-motion, code ligatures, WYSIWYG conceal, inline images,
//! the which-key force-show probe, …). Each of those modules used to
//! hand-roll an identical `AtomicBool` + relaxed-load/store pair, each with
//! its OWN copy (or its own ABSENCE) of the `testlock` write discipline.
//! [`Toggle`] is that one mechanism; every consumer still owns its OWN
//! verb-named public surface (`outline_on`/`set_outline_on`/`toggle`, …) and
//! its own default, doc comment, and tests — only the storage + guard
//! collapse to one place. The sweep law below (and its allow-list) names the
//! few sticky flags that deliberately stayed on a raw `AtomicBool`.
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
//! its OWN `crate::testlock::serial()` guard first — [`Toggle`]'s assert then
//! finds the lock already held (the guard is reentrant) rather than taking it
//! itself, so the multi-step window stays one continuous acquisition.

use std::sync::atomic::{AtomicBool, Ordering};

/// A process-global boolean flag: one `AtomicBool` behind a relaxed
/// load/store, the one shape every sticky on/off preference uses.
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

    // ── THE SWEEP LAW ───────────────────────────────────────────────────────
    //
    // A fresh `static FOO_ON: AtomicBool` outside this file is exactly the
    // reimplemented-quadruplet bug this module retired: a new sticky flag
    // must either route through `Toggle`, or join `ALLOWED_RAW_ATOMIC_BOOL`
    // below with a NAMED reason. No wildcard: every hit not in the allow-list
    // fails the test, so a new holdout can't dodge the sweep by silence.

    /// Files allowed to declare a raw `static _: AtomicBool`, and why each
    /// one is a genuinely different mechanism rather than an undodged toggle.
    /// `card.rs`'s `CardFlag` (about/lifetime/streaks' shared open-flag
    /// mechanism, its own precedent) and `testlock.rs`'s own concurrency-test
    /// fixture don't need an entry: neither wraps a BARE `static _: AtomicBool`
    /// (the pattern below scans for), so they can never appear in `hits`.
    const ALLOWED_RAW_ATOMIC_BOOL: &[(&str, &str)] = &[
        (
            "render/blur/suppress.rs",
            "SUPPRESSED cannot exist in a ship build at all: `mod suppress` is \
            `cfg(test)` and `frost_mode`'s branch on it carries the same attribute, \
            so there is no shipped state here to make sticky. It is the test-only \
            door that gives a completeness law two frames differing ONLY by the \
            card's own drawing -- deliberately outside `testlock::serial()`, and \
            restored by its caller on every exit path including the unwinding one, \
            the same discipline the menu-bar arm follows in that test family",
        ),
        (
            "streaks.rs",
            "CUMULATIVE rides CardFlag's OWN (deliberately \
            unguarded) discipline — `set_open` resets it in the same \
            unguarded window `CardFlag::set_open` writes in, and it has no \
            direct external setter (only the XOR-only `toggle_view`)",
        ),
        (
            "hud.rs",
            "HUD_HELD is TRANSIENT interaction state (held while a key \
            is down, no config binding, no persistence) — the same category \
            as menubar::OPEN_MENU, not a sticky preference",
        ),
        (
            "probe.rs",
            "LIVE_ACTIVE / FLIGHT_ACTIVE — the flight recorder, a \
            localized single-concern cluster whose two flags are read and \
            written together by one owner, not independent preferences",
        ),
        (
            "crashlog.rs",
            "HOOK_INSTALLED is a one-way install witness (no \
            setter, no toggle, never flips back) — a startup latch, not a \
            get/set/toggle preference",
        ),
    ];

    fn scan_dir(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, usize)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(root, &path, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            // Skip this file itself: its OWN test text below quotes the
            // scanned-for pattern in prose/string literals, which would
            // otherwise self-match as a false positive.
            if path.file_name().and_then(|n| n.to_str()) == Some("toggle.rs") {
                continue;
            }
            scan_file(root, &path, out);
        }
    }

    /// Mirrors `render::overrides::tests::scan_file`: skips `#[cfg(test)]`-gated
    /// bodies, so a test-only fixture's own `AtomicBool` doesn't self-match.
    /// Reports each hit keyed by its path RELATIVE to `root` (`src/`) — a bare
    /// filename like `mod.rs` is ambiguous across submodule directories.
    fn scan_file(root: &std::path::Path, path: &std::path::Path, out: &mut Vec<(String, usize)>) {
        #[derive(Clone, Copy, PartialEq)]
        enum State {
            Normal,
            AfterCfgTest,
            InSkippedBlock(i32),
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let name = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let mut state = State::Normal;
        for (i, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            state = match state {
                State::Normal => {
                    if trimmed.starts_with("#[cfg(test)") || trimmed.starts_with("#[cfg(all(test") {
                        State::AfterCfgTest
                    } else {
                        if !trimmed.starts_with("//")
                            && trimmed.contains("static ")
                            && trimmed.contains("AtomicBool")
                        {
                            out.push((name.clone(), i + 1));
                        }
                        State::Normal
                    }
                }
                State::AfterCfgTest => {
                    if trimmed.starts_with("#[") {
                        State::AfterCfgTest
                    } else if line.contains('{') {
                        let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                        if d <= 0 {
                            State::Normal
                        } else {
                            State::InSkippedBlock(d)
                        }
                    } else if trimmed.ends_with(';') {
                        State::Normal
                    } else {
                        State::AfterCfgTest
                    }
                }
                State::InSkippedBlock(depth) => {
                    let d =
                        depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        State::Normal
                    } else {
                        State::InSkippedBlock(d)
                    }
                }
            };
        }
    }

    #[test]
    fn every_sticky_atomic_bool_routes_through_toggle_or_is_named_here() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut hits = Vec::new();
        scan_dir(&root, &root, &mut hits);

        let stray: Vec<_> = hits
            .iter()
            .filter(|(f, _)| !ALLOWED_RAW_ATOMIC_BOOL.iter().any(|(name, _)| name == f))
            .collect();
        assert!(
            stray.is_empty(),
            "a raw `static _: AtomicBool` outside `crate::toggle::Toggle` and the \
             named allow-list is exactly the reimplemented-quadruplet bug this \
             module retired — route it through `Toggle`, or add it to \
             `ALLOWED_RAW_ATOMIC_BOOL` with a reason. offending lines:\n{}",
            stray
                .iter()
                .map(|(f, l)| format!("  {f}:{l}"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Non-vacuous: every allow-list entry must actually still exist, or
        // the list is stale (and should shrink — a converted flag must be
        // REMOVED from here, not just left unenforced).
        for (file, reason) in ALLOWED_RAW_ATOMIC_BOOL {
            assert!(
                hits.iter().any(|(f, _)| f == file),
                "{file} is on the allow-list ({reason}) but no longer declares a raw \
                 AtomicBool — remove it from the list instead of leaving it stale"
            );
        }
    }
}
