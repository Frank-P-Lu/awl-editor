//! src/app/persistence.rs — THE APP-GLOBAL SAVE LEDGER (`PersistenceRuntime`,
//! whose ownership map is `docs/app-domains.md`).
//!
//! Saving in awl has two halves. The PER-BUFFER half — `doc_saved_version`,
//! `scratch_saved_version`, `disk_mtime`, `scratch_mtime`, `doc_autosave_at` —
//! correctly lives in `files::BufferExtra` and travels with the active slot
//! (that split keeps this state buffer-local). This module owns the APP-GLOBAL half:
//!
//!  - the FRESH-DOCUMENT (unnamed, Cmd-N) autosave ledger: its debounce stamp
//!    and the buffer version it last wrote,
//!  - the two save-feedback clocks the Debug panel and the held HUD read,
//!  - the window title's dirty-state cache.
//!
//! ## Per-identity note ledgers
//!
//! Every unnamed fresh buffer has a [`crate::buffers::BufferKey::Fresh`] key,
//! and therefore its own debounce, handled version, and first-failure flag.
//! Buffer versions restart at zero, so identity is part of this cache key.
//!
//! A naming attempt records the current version BEFORE the write. On failure,
//! that exact version is deliberately not retried on every idle tick: the first
//! failure gets one calm notice, and another edit (a new version) arms one new
//! attempt. Manual Save remains an immediate explicit retry. On success, the
//! Fresh ledger is retired in the same re-key transition that installs Path.

use crate::clock::Instant;

#[cfg(not(target_arch = "wasm32"))]
mod fault_probe;

#[cfg(not(target_arch = "wasm32"))]
impl super::App {
    pub(crate) fn run_persistence_fault_probe(
        operation: &str,
        args: &[std::path::PathBuf],
    ) -> anyhow::Result<()> {
        fault_probe::run(operation, args)
    }
}

/// The app-global save ledger. Fields are private: every write is a named
/// transition, so the two fields of the fresh-document ledger cannot drift
/// apart.
#[derive(Default)]
pub(in crate::app) struct PersistenceRuntime {
    /// When the active FRESH DOCUMENT last changed and a debounced auto-name
    /// save is pending; the write fires after `AUTOSAVE_DEBOUNCE` of quiet in
    /// `about_to_wait` (live only — headless never schedules this).
    /// `None` = nothing pending.
    note_ledgers: std::collections::HashMap<crate::buffers::BufferKey, NoteLedger>,
    /// When the AUTOSAVE ENGINE last wrote successfully THIS session (the
    /// document autosave OR the scratch stash) — stamped ONLY through
    /// [`Self::record_engine_write`], i.e. exclusively inside
    /// `autosave_doc_now`/`stash_scratch_now`'s `Ok` arms, past the clobber
    /// guard. So the Debug panel's `autosave saved · Ns ago` line can never
    /// claim a write the engine did not just make. `None` before the first
    /// successful write.
    engine_last_write: Option<Instant>,
    /// When ANY successful write landed this session — a manual save, the
    /// scratch→note conversion, a fresh document's auto-name save, or the
    /// autosave engine. Feeds the held HUD's SAVED stat.
    last_save_at: Option<Instant>,
    /// SAVE-FEEDBACK: the dirty-state the window title/titlebar last
    /// rendered, so `sync_view` re-titles only on an actual clean↔dirty FLIP
    /// rather than formatting a string and making an OS call every keystroke.
    title_dirty: bool,
    /// THE ONE UNRESOLVED EXTERNAL CHANGE, or `None`. App-global and singular
    /// on purpose: while one is open every door that would leave the document
    /// — switch, rename, move, Finish — is refused, so a second conflict is
    /// unreachable rather than merely unlikely. That is what makes "one
    /// recovery record" ([`crate::recovery`]) a structural fact instead of a
    /// convention.
    unresolved: Option<UnresolvedChange>,
    /// Has the user already been sent back to the conflict once by trying to
    /// quit? A second Quit proceeds. Refusing forever would trap someone whose
    /// only way out is a resolution they may not want to make yet — and it is
    /// unnecessary, because the recovery record has already made quitting
    /// lossless by the time this is consulted.
    quit_deferred_once: bool,
}

#[derive(Default)]
struct NoteLedger {
    debounce_at: Option<Instant>,
    saved_version: Option<u64>,
    failure_reported: bool,
}

/// A file that changed on disk while awl held unsaved edits for it. Both texts
/// are real work; awl holds the editable one and stops writing to the path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) struct UnresolvedChange {
    /// The user's file — the path awl has stopped writing to.
    pub(in crate::app) path: std::path::PathBuf,
    /// What the disk said when the conflict was raised, or `None` when the file
    /// was DELETED — in which case there is no disk version to take, and the
    /// resolution offering it must decline rather than write an empty document.
    pub(in crate::app) theirs: Option<String>,
}

impl PersistenceRuntime {
    // ─── THE FRESH-DOCUMENT LEDGER (one rule, one owner) ─────────────────

    /// Is the fresh document OWED a write at `version`?
    ///
    /// **The sole spelling** of the per-key handled-version comparison. The caller
    /// still supplies "is this buffer an unnamed fresh document" — that is the
    /// buffer's own fact, not this ledger's.
    pub(in crate::app) fn note_write_owed(
        &self,
        key: &crate::buffers::BufferKey,
        version: u64,
    ) -> bool {
        self.note_ledgers.get(key).and_then(|s| s.saved_version) != Some(version)
    }

    /// Record that `version` has been handled by the fresh-document autosave,
    /// and retire any pending debounce in the SAME transition.
    ///
    /// The pairing is the point: three separate sites used to write the two
    /// fields as two statements, and a fourth site (`autosave_note`) wrote only
    /// the version and relied on its callers to clear the timer.
    pub(in crate::app) fn record_note_write(
        &mut self,
        key: crate::buffers::BufferKey,
        version: u64,
    ) {
        let ledger = self.note_ledgers.entry(key).or_default();
        ledger.saved_version = Some(version);
        ledger.debounce_at = None;
    }

    /// A buffer swap to a BRAND-NEW fresh document: forget the version AND the
    /// timer, so the new document is owed its first write at whatever version
    /// it starts from — including a version the PREVIOUS document already
    /// recorded. Clearing only the timer here is the defect this transition
    /// exists to make unwritable; see this module's doc.
    pub(in crate::app) fn reset_for_fresh_document(&mut self, key: crate::buffers::BufferKey) {
        self.note_ledgers.insert(key, NoteLedger::default());
    }

    /// Arm the fresh-document debounce at `now` (re-stamping slides the
    /// deadline, which is what collapses a typing burst into one write).
    pub(in crate::app) fn arm_note_debounce(
        &mut self,
        key: crate::buffers::BufferKey,
        now: Instant,
    ) {
        self.note_ledgers.entry(key).or_default().debounce_at = Some(now);
    }

    /// Retire a pending debounce WITHOUT recording a version — the flush path,
    /// which is about to perform the write itself and will record the version
    /// through [`Self::record_note_write`].
    pub(in crate::app) fn disarm_note_debounce(&mut self, key: &crate::buffers::BufferKey) {
        if let Some(ledger) = self.note_ledgers.get_mut(key) {
            ledger.debounce_at = None;
        }
    }

    /// The armed debounce's deadline for a `window` of quiet, or `None` when
    /// nothing is pending. The scheduler needs the instant itself (for its one
    /// `WaitUntil`), so this returns the deadline rather than a bool.
    pub(in crate::app) fn note_debounce_deadline(
        &self,
        key: &crate::buffers::BufferKey,
        window: std::time::Duration,
    ) -> Option<Instant> {
        self.note_ledgers
            .get(key)
            .and_then(|ledger| ledger.debounce_at)
            .map(|dirty| dirty + window)
    }

    /// Returns true only for the first failed naming autosave of this buffer.
    pub(in crate::app) fn first_note_failure(&mut self, key: crate::buffers::BufferKey) -> bool {
        let ledger = self.note_ledgers.entry(key).or_default();
        let first = !ledger.failure_reported;
        ledger.failure_reported = true;
        first
    }

    pub(in crate::app) fn clear_note_failure(&mut self, key: &crate::buffers::BufferKey) {
        if let Some(ledger) = self.note_ledgers.get_mut(key) {
            ledger.failure_reported = false;
        }
    }

    pub(in crate::app) fn retire_note_ledger(&mut self, key: &crate::buffers::BufferKey) {
        self.note_ledgers.remove(key);
    }

    #[cfg(test)]
    pub(in crate::app) fn has_note_ledger(&self, key: &crate::buffers::BufferKey) -> bool {
        self.note_ledgers.contains_key(key)
    }

    // ─── SAVE-FEEDBACK CLOCKS ────────────────────────────────────────────

    /// A successful write of ANY kind landed at `now`.
    pub(in crate::app) fn record_save(&mut self, now: Instant) {
        self.last_save_at = Some(now);
    }

    /// The AUTOSAVE ENGINE itself wrote successfully at `now`. Stamps the
    /// engine's own clock AND the any-kind clock, which the two engine `Ok`
    /// arms (`autosave_doc_now`, `stash_scratch_now`) used to do as two
    /// statements each — four lines that had to stay in lockstep.
    pub(in crate::app) fn record_engine_write(&mut self, now: Instant) {
        self.engine_last_write = Some(now);
        self.record_save(now);
    }

    /// When any write last landed (the held HUD's SAVED stat).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn last_save_at(&self) -> Option<Instant> {
        self.last_save_at
    }

    /// When the autosave engine last wrote (the Debug panel's line).
    pub(in crate::app) fn engine_last_write_at(&self) -> Option<Instant> {
        self.engine_last_write
    }

    // ─── THE UNRESOLVED EXTERNAL CHANGE (one, or none) ───────────────────

    /// The open conflict, if there is one. Every gated door reads this and
    /// nothing else, so "is this document resolvable right now" has one answer.
    pub(in crate::app) fn unresolved(&self) -> Option<&UnresolvedChange> {
        self.unresolved.as_ref()
    }

    /// Is a conflict open for `path` specifically? A conflict belongs to one
    /// file; a door acting on a DIFFERENT file is not gated by it, which
    /// matters because the gated doors are reached from surfaces that name
    /// their own target.
    pub(in crate::app) fn unresolved_for(&self, path: &std::path::Path) -> bool {
        self.unresolved.as_ref().is_some_and(|u| u.path == path)
    }

    /// LATCH a conflict. Replaces any previous one — see the field's doc for
    /// why a second is unreachable rather than merely rare.
    pub(in crate::app) fn set_unresolved(&mut self, change: UnresolvedChange) {
        self.unresolved = Some(change);
        self.quit_deferred_once = false;
    }

    /// RESOLVED — the only way out, taken by both resolutions and by nothing
    /// else. Returns what was latched so the caller can act on it.
    pub(in crate::app) fn take_unresolved(&mut self) -> Option<UnresolvedChange> {
        self.quit_deferred_once = false;
        self.unresolved.take()
    }

    /// Should a Quit be sent back to the conflict? True exactly once per
    /// latched conflict: the first attempt is deferred so the user is told,
    /// every attempt after it proceeds. Consuming the flag here — rather than
    /// asking and clearing at the call site — is what keeps "exactly once"
    /// from depending on the caller remembering to clear it.
    pub(in crate::app) fn defer_quit_for_conflict(&mut self) -> bool {
        if self.unresolved.is_none() || self.quit_deferred_once {
            return false;
        }
        self.quit_deferred_once = true;
        true
    }

    // ─── THE TITLE DIRTY-STATE CACHE ─────────────────────────────────────

    /// Does the window title need re-rendering for `dirty`? True only on an
    /// actual clean↔dirty flip.
    pub(in crate::app) fn title_cache_stale(&self, dirty: bool) -> bool {
        self.title_dirty != dirty
    }

    /// Record what the title just rendered. Called by `update_title` — the ONE
    /// writer — so any caller of it keeps this cache honest, not just
    /// `sync_view`'s own comparison.
    pub(in crate::app) fn record_title(&mut self, dirty: bool) {
        self.title_dirty = dirty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// A deterministic origin: one `VirtualClock` read, then pure offsets. The
    /// clock is the App's one time owner (`crate::clock`), so these tests never
    /// touch a wall clock — the fake-clock determinism law applies here too.
    fn origin() -> Instant {
        crate::clock::Clock::now(&crate::clock::VirtualClock::new())
    }

    /// THE VERSION-COLLISION LAW.
    ///
    /// The axis swept is the one the version-keyed cache actually fails on:
    /// **version reuse across two documents**, not "does a new document need
    /// saving". Buffer versions restart at 0 per open, so document B's version
    /// 1 is indistinguishable from document A's version 1 by value alone — and
    /// `reset_for_fresh_document` is the ONLY thing standing between that and a
    /// swallowed first save.
    ///
    /// Swept over the values that collide, including the 0 restart and the
    /// saturation end, rather than one hand-picked pair.
    #[test]
    fn the_fresh_document_ledger_forgets_the_version_not_just_the_timer() {
        let t0 = origin();
        for version in [0u64, 1, 2, 7, 4096, u64::MAX] {
            let mut p = PersistenceRuntime::default();
            let a = crate::buffers::BufferKey::Fresh(1);
            let b = crate::buffers::BufferKey::Fresh(2);
            // A brand-new ledger owes a write at every version.
            assert!(
                p.note_write_owed(&a, version),
                "an untouched ledger must owe a write at version {version}"
            );

            // Document A records this version.
            p.arm_note_debounce(a.clone(), t0);
            p.record_note_write(a.clone(), version);
            assert!(
                !p.note_write_owed(&a, version),
                "the recorded version must not be owed again ({version})"
            );
            assert_eq!(
                p.note_debounce_deadline(&a, Duration::from_millis(400)),
                None,
                "recording a write must retire the debounce in the same \
                 transition ({version})"
            );

            // Cmd-N: document B starts fresh and reaches the SAME version.
            p.reset_for_fresh_document(b.clone());
            assert!(
                p.note_write_owed(&b, version),
                "after a fresh-document reset, version {version} must be owed \
                 again — document B's counter collided with document A's and the \
                 new document's first save was swallowed"
            );
            assert_eq!(
                p.note_debounce_deadline(&b, Duration::from_millis(400)),
                None,
                "a fresh document starts with no pending write ({version})"
            );
        }
    }

    /// `disarm_note_debounce` is the flush path's tool: it retires the timer
    /// and deliberately does NOT record a version, because the caller is about
    /// to perform the write. Confusing the two would make `flush_note` mark the
    /// content saved without saving it.
    #[test]
    fn disarming_the_debounce_does_not_claim_the_write() {
        let mut p = PersistenceRuntime::default();
        let key = crate::buffers::BufferKey::Fresh(1);
        p.arm_note_debounce(key.clone(), origin());
        p.disarm_note_debounce(&key);
        assert_eq!(
            p.note_debounce_deadline(&key, Duration::from_millis(400)),
            None
        );
        assert!(
            p.note_write_owed(&key, 3),
            "disarming the timer must never make the content look saved"
        );
    }

    /// A failed automatic naming attempt handles one VERSION, not the whole
    /// buffer. This is the anti-churn policy: an idle loop does not hammer an
    /// unwritable folder or repeat its notice, while the next edit earns one
    /// new attempt. Manual Save bypasses this owed-write scheduler entirely.
    #[test]
    fn a_failed_note_version_waits_for_an_edit_and_reports_once() {
        let mut p = PersistenceRuntime::default();
        let key = crate::buffers::BufferKey::Fresh(41);
        assert!(p.note_write_owed(&key, 7));

        // `autosave_note` records immediately before attempting the write.
        p.record_note_write(key.clone(), 7);
        assert!(p.first_note_failure(key.clone()));
        assert!(
            !p.first_note_failure(key.clone()),
            "no debounce notice spam"
        );
        assert!(
            !p.note_write_owed(&key, 7),
            "the same failed version is not retried by every idle tick"
        );
        assert!(
            p.note_write_owed(&key, 8),
            "the next edit creates a new version and one new attempt"
        );
    }

    #[test]
    fn successful_rekey_retires_only_the_named_fresh_ledger() {
        let mut p = PersistenceRuntime::default();
        let named = crate::buffers::BufferKey::Fresh(1);
        let other = crate::buffers::BufferKey::Fresh(2);
        p.record_note_write(named.clone(), 3);
        p.record_note_write(other.clone(), 3);

        p.retire_note_ledger(&named);

        assert!(!p.has_note_ledger(&named));
        assert!(p.has_note_ledger(&other));
    }

    /// The debounce deadline is the arm instant plus the window, and a
    /// re-stamp slides it — the burst-collapsing behavior the whole debounce
    /// exists for.
    #[test]
    fn re_arming_the_note_debounce_slides_the_deadline() {
        let window = Duration::from_millis(400);
        let t0 = origin();
        let mut p = PersistenceRuntime::default();
        let key = crate::buffers::BufferKey::Fresh(1);
        assert_eq!(p.note_debounce_deadline(&key, window), None);
        p.arm_note_debounce(key.clone(), t0);
        assert_eq!(p.note_debounce_deadline(&key, window), Some(t0 + window));
        p.arm_note_debounce(key.clone(), t0 + Duration::from_millis(150));
        assert_eq!(
            p.note_debounce_deadline(&key, window),
            Some(t0 + Duration::from_millis(550)),
            "a fresh keystroke must slide the deadline, not keep the old one"
        );
    }

    /// An ENGINE write stamps both clocks; an ordinary save stamps only the
    /// any-kind clock. The Debug panel's `autosave saved · Ns ago` line reads
    /// the engine clock specifically, so a manual Cmd-S must never make it
    /// claim the engine wrote.
    #[test]
    fn only_the_engine_stamps_the_engine_clock() {
        let t0 = origin();
        let ten = t0 + Duration::from_millis(10);
        let twenty = t0 + Duration::from_millis(20);
        let mut p = PersistenceRuntime::default();
        assert_eq!(p.last_save_at(), None);
        assert_eq!(p.engine_last_write_at(), None);

        p.record_save(ten);
        assert_eq!(p.last_save_at(), Some(ten));
        assert_eq!(
            p.engine_last_write_at(),
            None,
            "a manual save must not make the Debug panel claim an engine write"
        );

        p.record_engine_write(twenty);
        assert_eq!(p.engine_last_write_at(), Some(twenty));
        assert_eq!(
            p.last_save_at(),
            Some(twenty),
            "an engine write is also a save; the two clocks moved in lockstep at \
             both former call sites and must keep doing so"
        );
    }

    /// The title cache reports stale only on a real flip, in both directions.
    #[test]
    fn the_title_cache_is_stale_only_on_a_flip() {
        let mut p = PersistenceRuntime::default();
        // A fresh App starts clean, so "clean" is not stale and "dirty" is.
        assert!(!p.title_cache_stale(false));
        assert!(p.title_cache_stale(true));
        p.record_title(true);
        assert!(!p.title_cache_stale(true));
        assert!(p.title_cache_stale(false));
        p.record_title(false);
        assert!(!p.title_cache_stale(false));
    }
}
