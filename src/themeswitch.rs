//! src/themeswitch.rs — THE THEME-SWITCH TRANSACTION-LATENCY readout (DEBUG-mode,
//! LIVE-ONLY): completed theme changes retained in a five-second wall-clock window,
//! with the slowest transaction's per-phase breakdown so the dominant cost NAMES
//! ITSELF instead of being guessed.
//!
//! WHAT IT REPORTS (drawn as three extra lines in the debug panel, `debug.rs`, only
//! after a real switch has been measured):
//!   * `theme latest N ms` — the most recently completed input-to-settled-present
//!     transaction.
//!   * `theme worst N ms` — the slowest completed transaction still inside the
//!     trailing five-second wall-clock window: the honest "how long until it
//!     settled" number.
//!   * `wait W · font X · reshape Y · rowgeom Z · sched S · atlas A · acquire Q · present P`
//!     — each
//!     phase's own duration (ms), for that `theme worst` transaction, in wall-clock
//!     order. The roster spans the WHOLE transaction, not only its work:
//!       - `wait`    — the interval between the input and the start of the reshape
//!         work. Nothing is deliberately deferred anymore, so it now reads
//!         near-zero on every switch — a scheduling fact, not a cost, and naming
//!         it (rather than folding it silently into `reshape`) is still the
//!         point — see the coverage note below.
//!       - `font`    — adopt the new world's effective face + rewrap the document to it
//!         (`sync_theme_font`'s pre-shape reconfigure; cosmic-text loads the
//!         face lazily, so its file-load cost is amortized into `reshape`/`atlas`).
//!       - `reshape` — re-lay every line's attrs + shape the whole document in the new face.
//!       - `rowgeom` — recompute the variable-row visual-geometry cache.
//!       - `sched`   — reshape done to the START of the frame that carries it: the
//!         redraw request's own trip through the event loop (winit dispatch + vsync).
//!       - `atlas`   — the settled frame's `prepare` span (rasterize + upload the new
//!         face's glyphs into the atlas; on a switch frame this dominates prepare).
//!       - `acquire` — the wait for a free drawable (`get_current_texture`), which sits
//!         between the prepare and present spans and belongs to neither.
//!       - `present` — that frame's encode + submit + present (the reshaped doc reaches screen).
//!
//! THE ROSTER MUST SPAN THE WHOLE TRANSACTION, AND [`MIN_PHASE_COVERAGE`] ENFORCES IT.
//! A breakdown of WORK phases alone can sit under a headline it does not remotely add
//! up to: a live HUD once read `theme latest 103.6 ms` over five columns summing to
//! 2.1 ms — 1.8% of its own number. That is worse than no breakdown, because it invites
//! tuning whichever of the small numbers is largest while the rest of the transaction
//! is somewhere no column looks. The three segments nothing timed were the DELIBERATE
//! debounce wait, the redraw request's trip through the event loop, and the wait for a
//! drawable; `Wait`, `Schedule` and `Acquire` name them. The floor then makes any
//! FUTURE gap self-reporting: whenever the recorded phases fall below that fraction of
//! the headline, [`breakdown_readout`] appends its own `unaccounted N.N` term, so the
//! next blind spot announces itself in the readout instead of hiding under it.
//!
//! THE PURE / LIVE SPLIT (mirrors `debug.rs`'s readout functions). This module reads
//! NO clock: phase accumulation and window reporting are fed caller-owned timestamps
//! plus synthetic-or-real millis, then formatted purely below. The module is therefore
//! unit-testable with a fake clock, and the readout is STRUCTURALLY ABSENT from a
//! headless capture: [`settle_lines`] returns an EMPTY vec for `None` — the ONLY value
//! a capture ever holds, because the live App never feeds a switch on the deterministic
//! path (the reshape timers live behind `debug_on()` + the live App, exactly like the
//! frametime/autosave/gpu readouts). A `--debug` screenshot is therefore byte-identical
//! to before this feature: no data → no lines. Real milliseconds remain LIVE-ONLY and
//! need human confirmation on a live run.

/// The named phases of a theme-switch settle, in wall-clock order. Each names a
/// real segment of the switch work so the dominant cost identifies itself in the
/// breakdown line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchPhase {
    /// The wait between the input and the start of the reshape work. Reads
    /// near-zero on every switch, since nothing is deliberately deferred. A
    /// scheduling FACT, not a cost.
    Wait,
    /// Adopt the new world's effective face + rewrap the document to it.
    Font,
    /// Re-lay every line's attrs + shape the whole document in the new face.
    Reshape,
    /// Recompute the variable-row visual-geometry cache.
    RowGeom,
    /// Reshape done to the START of the frame that carries it: the redraw
    /// request's own trip through the event loop (winit dispatch + vsync).
    Schedule,
    /// Rasterize + upload the new face's glyphs into the atlas (the settled
    /// frame's `prepare` span).
    Atlas,
    /// Wait for a free drawable (`get_current_texture`) — a whole vsync interval
    /// on a cold loop, and the largest single segment of a settled switch on a
    /// small document.
    Acquire,
    /// Encode + submit + present the reshaped frame (the first — settled — present).
    Present,
}

impl SwitchPhase {
    /// Every phase in wall-clock order — the breakdown line's fixed column order,
    /// and the roster [`SwitchPhases::recorded_ms`] sums over. It must span the
    /// WHOLE transaction: a roster naming only the work phases leaves the bulk of
    /// its own headline outside every column, which is how this one grew.
    pub const ORDER: [SwitchPhase; 8] = [
        SwitchPhase::Wait,
        SwitchPhase::Font,
        SwitchPhase::Reshape,
        SwitchPhase::RowGeom,
        SwitchPhase::Schedule,
        SwitchPhase::Atlas,
        SwitchPhase::Acquire,
        SwitchPhase::Present,
    ];

    /// The compact label the breakdown line uses for this phase.
    pub fn label(self) -> &'static str {
        match self {
            SwitchPhase::Wait => "wait",
            SwitchPhase::Font => "font",
            SwitchPhase::Reshape => "reshape",
            SwitchPhase::RowGeom => "rowgeom",
            SwitchPhase::Schedule => "sched",
            SwitchPhase::Atlas => "atlas",
            SwitchPhase::Acquire => "acquire",
            SwitchPhase::Present => "present",
        }
    }
}

/// A once-per-switch PHASE ACCUMULATOR: the live theme-switch path stamps an
/// `Instant` at each phase boundary and records the elapsed millis here. Reads NO
/// clock itself — the caller owns every `Instant` — so it is fully unit-testable
/// with synthetic durations and structurally inert on the headless path (which
/// never constructs one). A phase left unrecorded reads back as `None` and shows a
/// `—` in the breakdown.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SwitchPhases {
    wait: Option<f32>,
    font: Option<f32>,
    reshape: Option<f32>,
    row_geom: Option<f32>,
    schedule: Option<f32>,
    atlas: Option<f32>,
    acquire: Option<f32>,
    present: Option<f32>,
}

/// THE COVERAGE FLOOR. The recorded phases must account for at least this
/// fraction of the transaction they sit under, or the breakdown is not a
/// breakdown — it is a handful of small numbers beside a large one. Below the
/// floor, [`breakdown_readout`] names the shortfall out loud rather than letting
/// it pass as tidy columns; `phase_roster_covers_a_real_transaction` is the law,
/// and its non-vacuity proof is a real readout that failed this way (2.1 ms of
/// phases under a 117.2 ms headline — 1.8%).
///
/// Set at 0.80 rather than 1.0 deliberately: `Atlas` and `Present` are measured
/// on the GPU's own perf stamps while the headline is measured on the App clock,
/// and the remainder of the settled frame (animation advance, `sync_view`) is
/// genuinely not one of the named phases. The floor bounds how much may go
/// unnamed; it does not pretend nothing can.
pub const MIN_PHASE_COVERAGE: f32 = 0.80;

impl SwitchPhases {
    /// Record (or overwrite) one phase's own duration, in milliseconds.
    pub fn record(&mut self, phase: SwitchPhase, ms: f32) {
        *match phase {
            SwitchPhase::Wait => &mut self.wait,
            SwitchPhase::Font => &mut self.font,
            SwitchPhase::Reshape => &mut self.reshape,
            SwitchPhase::RowGeom => &mut self.row_geom,
            SwitchPhase::Schedule => &mut self.schedule,
            SwitchPhase::Atlas => &mut self.atlas,
            SwitchPhase::Acquire => &mut self.acquire,
            SwitchPhase::Present => &mut self.present,
        } = Some(ms);
    }

    /// This phase's recorded duration (ms), or `None` if it was never recorded.
    pub fn get(&self, phase: SwitchPhase) -> Option<f32> {
        match phase {
            SwitchPhase::Wait => self.wait,
            SwitchPhase::Font => self.font,
            SwitchPhase::Reshape => self.reshape,
            SwitchPhase::RowGeom => self.row_geom,
            SwitchPhase::Schedule => self.schedule,
            SwitchPhase::Atlas => self.atlas,
            SwitchPhase::Acquire => self.acquire,
            SwitchPhase::Present => self.present,
        }
    }

    /// The total of every RECORDED phase (ms) over the whole [`SwitchPhase::ORDER`]
    /// roster — an unrecorded phase contributes nothing, exactly as it shows `—`.
    pub fn recorded_ms(&self) -> f32 {
        SwitchPhase::ORDER.iter().filter_map(|&p| self.get(p)).sum()
    }

    /// What fraction of a `total_ms` transaction the recorded phases account for.
    /// A non-positive total has no meaningful coverage (`None`) — the caller then
    /// has nothing to report a shortfall against.
    pub fn coverage(&self, total_ms: f32) -> Option<f32> {
        (total_ms > 0.0).then(|| self.recorded_ms() / total_ms)
    }

    /// The UNACCOUNTED milliseconds of a `total_ms` transaction, but ONLY when
    /// coverage has fallen below [`MIN_PHASE_COVERAGE`] — the one predicate both
    /// the readout and its law read, so the line a user screenshots and the test
    /// that guards it can never disagree about where the floor is.
    pub fn shortfall_ms(&self, total_ms: f32) -> Option<f32> {
        let coverage = self.coverage(total_ms)?;
        (coverage < MIN_PHASE_COVERAGE).then(|| total_ms - self.recorded_ms())
    }
}

use std::collections::VecDeque;
use std::time::Duration;

use crate::clock::Instant;

/// The wall-clock diagnostic window. Unlike [`crate::debug::COST_WINDOW`], this is
/// time rather than a number of frames: a hundred cheap caret frames must not erase
/// the switch hitch that prompted the investigation.
pub const SWITCH_WINDOW: Duration = Duration::from_secs(5);

/// One completed input-to-settled-present theme transaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompletedSwitch {
    total_ms: f32,
    phases: SwitchPhases,
}

impl CompletedSwitch {
    pub fn new(total_ms: f32, phases: SwitchPhases) -> Self {
        Self { total_ms, phases }
    }

    pub fn total_ms(self) -> f32 {
        self.total_ms
    }

    pub fn phases(self) -> SwitchPhases {
        self.phases
    }
}

/// The two facts the panel needs: the most recently completed transaction and the
/// slowest completed transaction still inside the wall-clock window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchReport {
    pub latest: CompletedSwitch,
    pub worst: CompletedSwitch,
}

/// Bounded, timestamped switch history. It is deliberately separate from
/// [`crate::debug::CostRing`]: a switch is an interaction transaction across event
/// turns, not a frame. Call [`Self::report`] on redraws that already occur; it never
/// schedules work or owns a tick.
#[derive(Debug, Default)]
pub struct SwitchHistory {
    entries: VecDeque<(Instant, CompletedSwitch)>,
}

impl SwitchHistory {
    pub fn insert(&mut self, completed_at: Instant, total_ms: f32, phases: SwitchPhases) {
        self.entries
            .push_back((completed_at, CompletedSwitch::new(total_ms, phases)));
        self.evict(completed_at);
    }

    /// Return the live report at `now`, evicting transactions whose age is strictly
    /// greater than five seconds. Exactly five seconds remains readable.
    pub fn report(&mut self, now: Instant) -> Option<SwitchReport> {
        self.evict(now);
        let latest = self.entries.back().map(|(_, entry)| *entry)?;
        let worst = self
            .entries
            .iter()
            .map(|(_, entry)| *entry)
            .max_by(|a, b| a.total_ms.total_cmp(&b.total_ms))?;
        Some(SwitchReport { latest, worst })
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Whether no completed transaction remains. The Debug-off closeout uses this so
    /// a history-only diagnostic session cannot escape the ordinary clear path.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn evict(&mut self, now: Instant) {
        while self.entries.front().is_some_and(|(at, _)| {
            now.checked_duration_since(*at)
                .is_some_and(|age| age > SWITCH_WINDOW)
        }) {
            self.entries.pop_front();
        }
    }
}

/// The debug-panel LINES for completed theme switches: the latest transaction, the
/// recent worst transaction, then that worst transaction's phase breakdown. Returns
/// an EMPTY vec when no switch remains in the diagnostic window.
///
/// `None` is the ONLY value the headless capture ever holds (the live App never
/// feeds a switch on the deterministic path), so an empty vec is what keeps the
/// readout STRUCTURALLY ABSENT from a `--debug` screenshot — no data, no lines, a
/// byte-identical capture. The determinism law rests on this: it is asserted
/// directly in the tests.
pub fn settle_lines(measured: Option<SwitchReport>) -> Vec<String> {
    let Some(report) = measured else {
        return Vec::new();
    };
    vec![
        format!("theme latest {:.1} ms", report.latest.total_ms()),
        format!("theme worst {:.1} ms", report.worst.total_ms()),
        breakdown_readout(&report.worst.phases(), report.worst.total_ms()),
    ]
}

/// The once-per-switch PHASE BREAKDOWN line: each phase's own duration in
/// wall-clock order (`SwitchPhase::ORDER`), `·`-separated, so the dominant cost
/// names itself. A phase with no recorded duration shows `—`.
///
/// THE LINE POLICES ITS OWN HONESTY. `total_ms` is the headline this breakdown
/// sits under, and whenever the recorded phases cover less than
/// [`MIN_PHASE_COVERAGE`] of it the line appends a final `unaccounted N.N` term.
/// A breakdown of 2.1 ms beneath a 117.2 ms headline prints `unaccounted 115.1`
/// and names itself on sight instead of reading as tidy columns.
pub fn breakdown_readout(phases: &SwitchPhases, total_ms: f32) -> String {
    let mut parts: Vec<String> = SwitchPhase::ORDER
        .iter()
        .map(|&p| match phases.get(p) {
            Some(ms) => format!("{} {:.1}", p.label(), ms),
            None => format!("{} —", p.label()),
        })
        .collect();
    if let Some(missing) = phases.shortfall_ms(total_ms) {
        parts.push(format!("unaccounted {missing:.1}"));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests;
