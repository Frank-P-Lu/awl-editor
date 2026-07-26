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
//!     trailing five-second wall-clock window. A debounced preview includes the user's
//!     pause before reshape: it is the honest "how long until it settled" number.
//!   * `font X · reshape Y · rowgeom Z · atlas W · present P` — each WORK phase's own
//!     duration (ms), for that `theme worst` transaction, in wall-clock order:
//!       - `font`    — adopt the new world's effective face + rewrap the document to it
//!                    (`sync_theme_font`'s pre-shape reconfigure; cosmic-text loads the
//!                    face lazily, so its file-load cost is amortized into `reshape`/`atlas`).
//!       - `reshape` — re-lay every line's attrs + shape the whole document in the new face.
//!       - `rowgeom` — recompute the variable-row visual-geometry cache.
//!       - `atlas`   — the settled frame's `prepare` span (rasterize + upload the new
//!                    face's glyphs into the atlas; on a switch frame this dominates prepare).
//!       - `present` — that frame's encode + submit + present (the reshaped doc reaches screen).
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
    /// Adopt the new world's effective face + rewrap the document to it.
    Font,
    /// Re-lay every line's attrs + shape the whole document in the new face.
    Reshape,
    /// Recompute the variable-row visual-geometry cache.
    RowGeom,
    /// Rasterize + upload the new face's glyphs into the atlas (the settled
    /// frame's `prepare` span).
    Atlas,
    /// Encode + submit + present the reshaped frame (the first — settled — present).
    Present,
}

impl SwitchPhase {
    /// The five phases in wall-clock order — the breakdown line's fixed column order.
    pub const ORDER: [SwitchPhase; 5] = [
        SwitchPhase::Font,
        SwitchPhase::Reshape,
        SwitchPhase::RowGeom,
        SwitchPhase::Atlas,
        SwitchPhase::Present,
    ];

    /// The compact label the breakdown line uses for this phase.
    pub fn label(self) -> &'static str {
        match self {
            SwitchPhase::Font => "font",
            SwitchPhase::Reshape => "reshape",
            SwitchPhase::RowGeom => "rowgeom",
            SwitchPhase::Atlas => "atlas",
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
    font: Option<f32>,
    reshape: Option<f32>,
    row_geom: Option<f32>,
    atlas: Option<f32>,
    present: Option<f32>,
}

impl SwitchPhases {
    /// Record (or overwrite) one phase's own duration, in milliseconds.
    pub fn record(&mut self, phase: SwitchPhase, ms: f32) {
        *match phase {
            SwitchPhase::Font => &mut self.font,
            SwitchPhase::Reshape => &mut self.reshape,
            SwitchPhase::RowGeom => &mut self.row_geom,
            SwitchPhase::Atlas => &mut self.atlas,
            SwitchPhase::Present => &mut self.present,
        } = Some(ms);
    }

    /// This phase's recorded duration (ms), or `None` if it was never recorded.
    pub fn get(&self, phase: SwitchPhase) -> Option<f32> {
        match phase {
            SwitchPhase::Font => self.font,
            SwitchPhase::Reshape => self.reshape,
            SwitchPhase::RowGeom => self.row_geom,
            SwitchPhase::Atlas => self.atlas,
            SwitchPhase::Present => self.present,
        }
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
        breakdown_readout(&report.worst.phases()),
    ]
}

/// The once-per-switch PHASE BREAKDOWN line: each phase's own duration in
/// wall-clock order (`SwitchPhase::ORDER`), `·`-separated, so the dominant cost
/// names itself. A phase with no recorded duration shows `—`.
pub fn breakdown_readout(phases: &SwitchPhases) -> String {
    let parts: Vec<String> = SwitchPhase::ORDER
        .iter()
        .map(|&p| match phases.get(p) {
            Some(ms) => format!("{} {:.1}", p.label(), ms),
            None => format!("{} —", p.label()),
        })
        .collect();
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_get_roundtrip_per_phase() {
        let mut p = SwitchPhases::default();
        // A fresh accumulator has nothing recorded.
        for ph in SwitchPhase::ORDER {
            assert_eq!(p.get(ph), None);
        }
        p.record(SwitchPhase::Font, 0.2);
        p.record(SwitchPhase::Reshape, 6.8);
        p.record(SwitchPhase::RowGeom, 0.9);
        p.record(SwitchPhase::Atlas, 2.0);
        p.record(SwitchPhase::Present, 0.6);
        assert_eq!(p.get(SwitchPhase::Font), Some(0.2));
        assert_eq!(p.get(SwitchPhase::Reshape), Some(6.8));
        assert_eq!(p.get(SwitchPhase::RowGeom), Some(0.9));
        assert_eq!(p.get(SwitchPhase::Atlas), Some(2.0));
        assert_eq!(p.get(SwitchPhase::Present), Some(0.6));
        // Recording again overwrites (a phase is measured once per switch).
        p.record(SwitchPhase::Reshape, 4.1);
        assert_eq!(p.get(SwitchPhase::Reshape), Some(4.1));
    }

    #[test]
    fn breakdown_readout_names_each_phase_in_order() {
        // Feed SYNTHETIC durations (no clock) and assert the exact formatted line —
        // the phases appear in wall-clock order, the dominant cost (reshape) visible.
        let mut p = SwitchPhases::default();
        p.record(SwitchPhase::Font, 0.2);
        p.record(SwitchPhase::Reshape, 6.8);
        p.record(SwitchPhase::RowGeom, 0.9);
        p.record(SwitchPhase::Atlas, 2.0);
        p.record(SwitchPhase::Present, 0.6);
        assert_eq!(
            breakdown_readout(&p),
            "font 0.2 · reshape 6.8 · rowgeom 0.9 · atlas 2.0 · present 0.6"
        );
    }

    #[test]
    fn breakdown_readout_shows_dash_for_an_unrecorded_phase() {
        // A partial accumulator (only the reshape-side phases recorded, e.g. a switch
        // whose present frame was skipped) shows `—` for the missing present-side ones.
        let mut p = SwitchPhases::default();
        p.record(SwitchPhase::Font, 0.1);
        p.record(SwitchPhase::Reshape, 5.0);
        p.record(SwitchPhase::RowGeom, 0.8);
        assert_eq!(
            breakdown_readout(&p),
            "font 0.1 · reshape 5.0 · rowgeom 0.8 · atlas — · present —"
        );
    }

    #[test]
    fn settle_lines_are_absent_without_a_measured_switch() {
        // DETERMINISM LAW (formatting seam): the `None` value — the ONLY value a
        // headless capture ever holds, since the live App never feeds a switch on the
        // deterministic path — yields ZERO lines. No data, no readout: a `--debug`
        // screenshot stays byte-identical to before this feature.
        assert_eq!(settle_lines(None), Vec::<String>::new());
    }

    #[test]
    fn settle_lines_are_the_headline_then_the_breakdown() {
        let mut p = SwitchPhases::default();
        p.record(SwitchPhase::Font, 0.2);
        p.record(SwitchPhase::Reshape, 6.8);
        p.record(SwitchPhase::RowGeom, 0.9);
        p.record(SwitchPhase::Atlas, 2.0);
        p.record(SwitchPhase::Present, 0.6);
        assert_eq!(
            settle_lines(Some(SwitchReport {
                latest: CompletedSwitch::new(12.0, p),
                worst: CompletedSwitch::new(155.2, p),
            })),
            vec![
                "theme latest 12.0 ms".to_string(),
                "theme worst 155.2 ms".to_string(),
                "font 0.2 · reshape 6.8 · rowgeom 0.9 · atlas 2.0 · present 0.6".to_string(),
            ]
        );
    }

    fn fake_clock() -> crate::clock::VirtualClock {
        crate::clock::VirtualClock::new()
    }

    fn switch(ms: f32) -> SwitchPhases {
        let mut phases = SwitchPhases::default();
        phases.record(SwitchPhase::Reshape, ms - 1.0);
        phases.record(SwitchPhase::Present, 1.0);
        phases
    }

    #[test]
    fn fake_clock_insertion_reports_latest_and_worst() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 42.0, switch(42.0));
        let report = history.report(t0).expect("inserted transaction survives");
        assert_eq!(report.latest.total_ms(), 42.0);
        assert_eq!(report.worst.total_ms(), 42.0);
    }

    #[test]
    fn fake_clock_rolling_max_keeps_the_worst_transactions_breakdown() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 42.0, switch(42.0));
        clock.advance_ms(1);
        let t1 = crate::clock::Clock::now(&clock);
        history.insert(t1, 9.0, switch(9.0));
        let report = history.report(t1).unwrap();
        assert_eq!(report.latest.total_ms(), 9.0);
        assert_eq!(report.worst.total_ms(), 42.0);
        assert_eq!(report.worst.phases().get(SwitchPhase::Reshape), Some(41.0));
    }

    #[test]
    fn fake_clock_exact_five_second_boundary_survives_then_expires() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 42.0, switch(42.0));
        clock.advance(SWITCH_WINDOW);
        assert!(
            history.report(crate::clock::Clock::now(&clock)).is_some(),
            "five seconds exactly stays"
        );
        clock.advance(Duration::from_nanos(1));
        assert!(history.report(crate::clock::Clock::now(&clock)).is_none());
    }

    #[test]
    fn fake_clock_newer_cheaper_switch_does_not_erase_peak() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 42.0, switch(42.0));
        clock.advance(Duration::from_secs(1));
        let t1 = crate::clock::Clock::now(&clock);
        history.insert(t1, 4.0, switch(4.0));
        let report = history.report(t1).unwrap();
        assert_eq!(report.latest.total_ms(), 4.0);
        assert_eq!(report.worst.total_ms(), 42.0);
    }

    #[test]
    fn fake_clock_newer_worse_switch_replaces_peak() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 4.0, switch(4.0));
        clock.advance(Duration::from_secs(1));
        let t1 = crate::clock::Clock::now(&clock);
        history.insert(t1, 42.0, switch(42.0));
        let report = history.report(t1).unwrap();
        assert_eq!(report.latest.total_ms(), 42.0);
        assert_eq!(report.worst.total_ms(), 42.0);
    }

    #[test]
    fn explicit_debug_off_clear_forgets_the_session() {
        let clock = fake_clock();
        let t0 = crate::clock::Clock::now(&clock);
        let mut history = SwitchHistory::default();
        history.insert(t0, 42.0, switch(42.0));
        history.clear();
        assert!(history.report(t0).is_none());
    }
}
