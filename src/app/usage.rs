//! src/app/usage.rs — THE LOCAL USAGE LEDGER (`UsageLedger`); the ownership
//! map is `docs/app-domains.md`.
//!
//! awl keeps two never-uploaded personal records: the LIFETIME ODOMETER
//! (`crate::stats` — keystrokes, honest writing time, files touched, caret
//! travel, and the silent command-usage ledger the discoverability surfaces
//! rank) and the WRITING STREAKS record (`crate::streaks` — the per-day word
//! delta the heatmap card draws). The pure stores, their arithmetic and their
//! (de)serializers live in those two modules; this one owns the LIVE STATE
//! around them — the flush ledger, the sampling anchors, and the privacy gate.
//!
//! Native only (`cfg(not(target_arch = "wasm32"))`), mirroring the daemon /
//! session-restore gate, and constructed ONLY by the live `App`: a headless
//! `--screenshot` / `--keys` capture never builds one, so it is structurally
//! incapable of touching `stats.toml` or `streaks.toml`
//! (`main::run::tests::headless_replay_never_touches_the_stats_file`).
//!
//! ## Why the two records are ONE domain
//!
//! They are not two features that happen to sit side by side. They share ONE
//! privacy kill switch (the single `stats` config toggle — both are private,
//! native-only, never-uploaded personal state), ONE flush cadence (the same
//! idle / blur / switch / quit triggers the autosave engine uses), and ONE
//! buffer-swap event: each holds a per-buffer SAMPLING ANCHOR that has to be
//! retired when the active document changes — the odometer's caret-travel
//! anchor (two documents' pixel coordinates are incomparable) and the streaks
//! word baseline (an opened file's existing words are not "writing").
//!
//! ## The invariants that were held by convention
//!
//! 1. **A record mutation and its unflushed-changes stamp.** Raising the
//!    odometer's `_dirty` flag was hand-written beside four separate
//!    `self.stats.record_*` calls, and the streaks flag beside one more. A
//!    sixth recording path that forgot the stamp would accrue in memory and be
//!    dropped at quit, silently and only for the user who hit it. The private
//!    [`dirtying`] module removes the possibility at COMPILE time rather than
//!    by review: outside it neither store is reachable as `&mut` and neither
//!    flag is writable at all, and its one door stamps from the closure's own
//!    [`dirtying::Changed`] report instead of from the caller remembering.
//!    `the_usage_records_have_exactly_one_dirty_stamping_site` fences the
//!    inside of that module, which the type system cannot.
//! 2. **The privacy gate.** The `stats` config toggle was re-read at eight
//!    sites across two modules, where a tracking hook that forgot the gate was
//!    a privacy defect one missing `if` away. Every transition below that can
//!    record or persist takes a [`Recording`] value instead, and
//!    `ConfigurationRuntime::usage_recording` is the one site under `src/app/`
//!    that reads the toggle at all — pinned by
//!    `the_usage_privacy_gate_has_exactly_one_reader` in `app/tests/domains.rs`.
//!
//! ## Where the ODOMETER and the STREAKS halves legitimately differ
//!
//! The two flushes are NOT interchangeable, and three call sites deliberately
//! run only one of them. The streaks delta is BUFFER-SCOPED — the word count
//! now minus the count at the last sample of *this* document — so it must be
//! sampled BEFORE the active buffer changes or the subtraction spans two
//! unrelated documents. The odometer's counters are app-global and have no such
//! deadline. Hence [`UsageLedger::flush_writing`] alone on the pre-swap path,
//! and both on the idle / blur / quit triggers. Preserved exactly.

use crate::clock::Instant;
use std::path::PathBuf;

/// THE PRIVACY GATE as a value. Every transition that can record or persist
/// takes one, so the tracking decision is made where the configuration lives
/// (`ConfigurationRuntime::usage_recording`) rather than re-derived per hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum Recording {
    /// Local usage tracking is on (`stats = true`, the default).
    On,
    /// The kill switch is off: nothing accrues and nothing is ever written.
    Off,
}

impl Recording {
    /// From the resolved `stats` config toggle. The ONE producer.
    pub(in crate::app) fn from_config(on: bool) -> Self {
        if on { Self::On } else { Self::Off }
    }

    fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

/// The pairing of a persisted record with its unflushed-changes flag, in its
/// own module so the flag is unreachable from the rest of this file.
///
/// **The whole point is that the flag cannot be forgotten.** `Dirtying`'s
/// fields are private to these few lines, so nowhere else in the program —
/// not even in the owner immediately below — does a `&mut` to either store or
/// a write to either flag exist. The only mutable door is [`Dirtying::record`],
/// which stamps from the closure's [`Changed`] report. The previous shape was a
/// store field and a `_dirty` field side by side on root `App`, paired by hand
/// at five call sites and checkable only by reading all five.
mod dirtying {
    /// Did a recording transition actually change the store? Returned by the
    /// closure [`Dirtying::record`] runs, so the unflushed-changes stamp
    /// follows from the store's own report — a deduped re-open of an
    /// already-seen file says [`Changed::No`] and leaves a quiet quit writing
    /// nothing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) enum Changed {
        Yes,
        No,
    }

    /// A persisted record and its unflushed-changes flag as ONE value.
    pub(super) struct Dirtying<T> {
        value: T,
        /// Whether `value` holds increments the last flush did not write, so a
        /// flush with nothing new skips the atomic write entirely.
        dirty: bool,
    }

    impl<T> Dirtying<T> {
        pub(super) fn new(value: T) -> Self {
            Self {
                value,
                dirty: false,
            }
        }

        pub(super) fn get(&self) -> &T {
            &self.value
        }

        /// THE ONLY `&mut` DOOR. Runs `f` against the store and stamps the
        /// record dirty iff `f` reports a real change.
        pub(super) fn record<R>(&mut self, f: impl FnOnce(&mut T) -> (Changed, R)) -> R {
            let (changed, out) = f(&mut self.value);
            if changed == Changed::Yes {
                self.dirty = true;
            }
            out
        }

        /// Persist through `write` iff something changed since the last flush.
        /// A failed write is reported and the flag still cleared — the shipped
        /// behaviour: a broken data directory must not turn every later idle
        /// tick into another failing write and another line of stderr.
        pub(super) fn flush(&mut self, what: &str, write: impl FnOnce(&T) -> std::io::Result<()>) {
            if !self.dirty {
                return;
            }
            if let Err(e) = write(&self.value) {
                eprintln!("{what} save failed: {e}");
            }
            self.dirty = false;
        }

        #[cfg(test)]
        pub(super) fn is_dirty(&self) -> bool {
            self.dirty
        }
    }
}

use dirtying::{Changed, Dirtying};

/// The live state around awl's two private local-usage records. Every field is
/// private; every write is a named transition.
pub(in crate::app) struct UsageLedger {
    /// The lifetime odometer + silent command-usage ledger, loaded at launch.
    odometer: Dirtying<crate::stats::Stats>,
    /// The session-timer origin, stamped from the SAME clock the frame runtime
    /// owns, so a deterministic clock would govern the active-writing odometer
    /// too.
    origin: Instant,
    /// Millis-since-`origin` of the previous keystroke — the `last` side of the
    /// capped active-writing interval. `None` until the first press.
    last_input_ms: Option<u64>,
    /// The caret's last-sampled DOCUMENT-space position (scroll-independent),
    /// for the travel accumulator. `None` until the first sample, and dropped
    /// on a buffer swap ([`Self::reset_caret_anchor`]) so the jump between two
    /// documents' incomparable coordinate spaces is never counted as travel.
    last_caret_xy: Option<(f32, f32)>,
    /// The logical `(line, col)` at that same sample. Travel is added only when
    /// THIS changed, so a pure scroll or a heading reshape refreshes the anchor
    /// without faking distance.
    last_cursor: Option<(usize, usize)>,
    /// The per-day writing record, loaded at launch.
    writing: Dirtying<crate::streaks::Streaks>,
    /// The active buffer's word count at the last streaks sample — the `last`
    /// side of the per-flush word DELTA. `None` = anchor lazily on the next
    /// flush, so an opened document's existing words are anchored rather than
    /// counted as freshly written.
    word_baseline: Option<usize>,
}

impl UsageLedger {
    /// Load both records through the same `FileSystem` seam the recent-* MRUs
    /// use (each degrades to an empty store on a fresh install) and start the
    /// active-writing session clock.
    pub(in crate::app) fn load(origin: Instant) -> Self {
        Self {
            odometer: Dirtying::new(crate::stats::load(&crate::stats::stats_path())),
            origin,
            last_input_ms: None,
            last_caret_xy: None,
            last_cursor: None,
            writing: Dirtying::new(crate::streaks::load(&crate::streaks::streaks_path())),
            word_baseline: None,
        }
    }

    // ── THE ODOMETER HALF ────────────────────────────────────────────────

    /// Record ONE keyboard press. `printable` is whether the press resolved to
    /// an `Action::InsertChar` (a real character written). Bumps `keystrokes`
    /// (+ `chars_typed` when printable) and folds the capped active-writing
    /// interval into the total and the active world's bucket, stamping this
    /// press as the next interval's `last`.
    pub(in crate::app) fn note_keystroke(
        &mut self,
        recording: Recording,
        now: Instant,
        printable: bool,
    ) {
        if !recording.is_on() {
            return;
        }
        let now_ms = now.duration_since(self.origin).as_millis() as u64;
        let world = crate::theme::active().name;
        let last = self.last_input_ms;
        self.odometer.record(|s| {
            s.record_keystroke(printable, world, last, now_ms);
            (Changed::Yes, ())
        });
        self.last_input_ms = Some(now_ms);
    }

    /// Sample the caret and accumulate its DOCUMENT-space travel. Distance is
    /// added ONLY when the logical cursor changed since the last sample; the
    /// anchor is refreshed either way.
    ///
    /// `sample` is a thunk, not a value, and deliberately so: it reads the GPU
    /// pipeline's caret position and queries the rope for `(line, col)` on
    /// EVERY `sync_view`, and the shipped behaviour never pays for either with
    /// tracking off. It yields `None` when the GPU is not up yet — nothing to
    /// read a caret position from.
    pub(in crate::app) fn track_caret(
        &mut self,
        recording: Recording,
        sample: impl FnOnce() -> Option<((f32, f32), (usize, usize))>,
    ) {
        if !recording.is_on() {
            return;
        }
        let Some((xy, cursor)) = sample() else {
            return;
        };
        if let (Some(prev_xy), Some(prev_cur)) = (self.last_caret_xy, self.last_cursor)
            && cursor != prev_cur
        {
            self.odometer.record(|s| {
                s.record_caret_move(prev_xy, xy);
                (Changed::Yes, ())
            });
        }
        self.last_caret_xy = Some(xy);
        self.last_cursor = Some(cursor);
    }

    /// Record a file OPEN into the distinct-files set. A re-open of an
    /// already-seen path is inert and never re-marks the record dirty — the
    /// store's own dedupe verdict IS the [`Changed`] report.
    pub(in crate::app) fn touch_file(&mut self, recording: Recording, path: PathBuf) {
        if !recording.is_on() {
            return;
        }
        self.odometer.record(|s| {
            let added = s.touch_file(path);
            (if added { Changed::Yes } else { Changed::No }, ())
        });
    }

    /// Drop the caret-travel anchor across a BUFFER SWAP, so the first sample
    /// in the arriving document re-anchors instead of counting the jump between
    /// two incomparable coordinate spaces as travel.
    pub(in crate::app) fn reset_caret_anchor(&mut self) {
        self.last_caret_xy = None;
        self.last_cursor = None;
    }

    /// Record ONE command dispatch, attributed to the `door` it came through.
    /// MOTIONS never reach the ledger even when the catalog lists them
    /// (navigation is not a "command" for discoverability, and without the gate
    /// every arrow press would key a row AND dirty the record); a non-catalog
    /// action yields no slug and allocates nothing, which is what keeps the hot
    /// typing path free.
    pub(in crate::app) fn note_dispatch(
        &mut self,
        recording: Recording,
        action: &crate::keymap::Action,
        door: crate::stats::Door,
    ) {
        if !recording.is_on() {
            return;
        }
        if action.is_motion() {
            return;
        }
        let Some(slug) = crate::commands::slug_for_action(action) else {
            return;
        };
        self.odometer.record(|s| {
            s.record_command(slug, door);
            (Changed::Yes, ())
        });
    }

    /// The LIFETIME-ODOMETER snapshot for the held HUD's rows, or `None` when
    /// tracking is off — so the rows honestly read as the `"—"` placeholder
    /// rather than a misleading row of zeros.
    pub(in crate::app) fn hud_snapshot(
        &self,
        recording: Recording,
    ) -> Option<crate::hud::HudStats> {
        if !recording.is_on() {
            return None;
        }
        let s = self.odometer.get();
        Some(crate::hud::HudStats {
            chars_typed: s.chars_typed,
            active_writing_ms: s.active_writing_ms,
            files_touched: s.files_touched_count(),
            caret_distance_px: s.caret_distance_px,
            world: s.most_used_world().map(|(name, _)| name.to_string()),
        })
    }

    /// The HOLD-⌘ peek's personalized rows: the top-[`crate::peek::PEEK_ROWS`]
    /// graduation candidates resolved to chord+name. Empty on a fresh install
    /// → the pipeline falls back to the curated starter six.
    pub(in crate::app) fn peek_rows(&self) -> Vec<crate::peek::PeekRow> {
        self.graduation_rows(crate::peek::PEEK_ROWS)
    }

    /// The Keybindings footer's "your top 3" tips — the SAME ranking as
    /// [`Self::peek_rows`], formatted as `"⌘O  Go to file"` one-liners.
    pub(in crate::app) fn keybinding_tips(&self) -> Vec<String> {
        self.graduation_rows(3)
            .into_iter()
            .map(|r| format!("{}  {}", r.chord, r.name))
            .collect()
    }

    /// The one graduation query both discoverability surfaces read, so the peek
    /// and the footer can never rank by different rules.
    fn graduation_rows(&self, n: usize) -> Vec<crate::peek::PeekRow> {
        self.odometer
            .get()
            .graduation_candidates(crate::commands::has_native_chord, n)
            .iter()
            .filter_map(|(slug, _)| crate::commands::peek_row_for_slug(slug))
            .collect()
    }

    /// Flush the odometer to `stats.toml` ATOMICALLY. A no-op when tracking is
    /// off OR nothing changed since the last flush, so a quiet blur or quit
    /// writes nothing.
    pub(in crate::app) fn flush_odometer(&mut self, recording: Recording) {
        if !recording.is_on() {
            return;
        }
        self.odometer.flush("stats", |s| {
            crate::stats::save(&crate::stats::stats_path(), s)
        });
    }

    // ── THE WRITING-STREAKS HALF ─────────────────────────────────────────

    /// Drop the word-delta anchor to LAZY across a buffer swap into an OPENED
    /// FILE, so the arriving document's existing words re-anchor on the next
    /// flush rather than counting as freshly written.
    pub(in crate::app) fn reset_word_baseline(&mut self) {
        self.word_baseline = None;
    }

    /// Anchor the word-delta baseline EAGERLY at `words` — the seam for an
    /// awl-CREATED buffer (a new note, or the birth / restored-stash scratch),
    /// whose birth content must not count as freshly written yet whose first
    /// post-birth keystrokes, typed BEFORE the first idle flush, must. A lazy
    /// anchor would sample at the already-typed count on that first flush and
    /// lose the whole window (the anchor-swallow bug).
    pub(in crate::app) fn anchor_words(&mut self, words: usize) {
        self.word_baseline = Some(words);
    }

    /// Sample `words` and fold the DELTA since the last sample into today's
    /// record, then persist if anything changed. The FIRST sample of a buffer
    /// only ANCHORS (records nothing), so a file's pre-existing words are never
    /// counted; every later sample records the net words added since the
    /// previous one (clamped for the day total, raw kept).
    ///
    /// `words` is a thunk for the same reason [`Self::track_caret`]'s sample
    /// is: counting words allocates the whole document as a `String`, and the
    /// shipped behaviour never pays for it with tracking off.
    pub(in crate::app) fn flush_writing(
        &mut self,
        recording: Recording,
        words: impl FnOnce() -> usize,
    ) {
        if !recording.is_on() {
            return;
        }
        let words = words();
        match self.word_baseline {
            None => {
                // Anchor only — a fresh launch or a just-swapped buffer.
                // Nothing recorded (opening content is not "writing").
                self.word_baseline = Some(words);
            }
            Some(prev) => {
                let delta = words as i64 - prev as i64;
                self.word_baseline = Some(words);
                if delta != 0 {
                    let day = local_today();
                    self.writing.record(|w| {
                        w.record_delta(&day, delta);
                        (Changed::Yes, ())
                    });
                }
            }
        }
        self.writing.flush("streaks", |w| {
            crate::streaks::save(&crate::streaks::streaks_path(), w)
        });
    }

    /// The live year-VIEW for a summoned Writing streaks card, or `None` when
    /// tracking is off — the card then shows the honest placeholder rather than
    /// a misleading empty grid.
    pub(in crate::app) fn writing_view(
        &self,
        recording: Recording,
    ) -> Option<crate::streaks::StreaksView> {
        recording
            .is_on()
            .then(|| self.writing.get().view(local_today_ymd()))
    }

    // ── READ-ONLY PROJECTIONS FOR THIS DOMAIN'S OWN LAWS ─────────────────

    #[cfg(test)]
    pub(in crate::app) fn odometer(&self) -> &crate::stats::Stats {
        self.odometer.get()
    }

    #[cfg(test)]
    pub(in crate::app) fn writing(&self) -> &crate::streaks::Streaks {
        self.writing.get()
    }

    #[cfg(test)]
    pub(in crate::app) fn odometer_dirty(&self) -> bool {
        self.odometer.is_dirty()
    }
}

/// Today's LOCAL calendar day as `"YYYY-MM-DD"`.
pub(in crate::app) fn local_today() -> String {
    let (y, m, d) = local_today_ymd();
    crate::streaks::fmt_ymd(y, m, d)
}

/// Today's LOCAL calendar day as `(y, m, d)` — the tuple form the streaks
/// card's view/streak/series consume directly (no stringify + re-parse round
/// trip). Reads the wall clock plus the OS's current UTC offset, then floors to
/// a civil date via the pure model. A clock before the epoch or a null
/// `localtime_r` degrades to a 0 offset (UTC), never a panic.
pub(in crate::app) fn local_today_ymd() -> (i64, i64, i64) {
    let secs = crate::clock::system_now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    crate::streaks::civil_ymd_from_epoch_secs(secs + local_utc_offset_secs())
}

/// The OS's CURRENT UTC offset in seconds (east positive) — the one timezone
/// read the streaks day boundary needs. std has no local-offset API, so this
/// reads libc's `tm_gmtoff` via `localtime_r` on the current time. A null
/// return (never expected) degrades to UTC (0). Unsafe FFI is contained here;
/// the result feeds the pure civil-date conversion.
fn local_utc_offset_secs() -> i64 {
    // SAFETY: `time` takes a null pointer (returns the current time) and
    // `localtime_r` writes into our stack `tm`, which we zero first. Both are
    // the documented calling conventions; `tm_gmtoff` is a stable field on
    // macOS + Linux libc.
    unsafe {
        let t: libc::time_t = libc::time(std::ptr::null_mut());
        let mut tmv: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tmv).is_null() {
            return 0;
        }
        tmv.tm_gmtoff as i64
    }
}
