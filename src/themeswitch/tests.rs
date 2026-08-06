//! Laws for the theme-switch transaction readout: the phase roster, the
//! breakdown line's exact text, the coverage floor that keeps the roster
//! honest, and [`SWITCH_WINDOW`]'s own eviction arithmetic under a fake
//! clock. Carved out of `themeswitch.rs` to keep that module under its ceiling.

use super::*;

/// A REAL transaction, read off the HUD of a live release run (2026-08-03,
/// `--live-script`, Kite→Mulga, 30ms-apart stepped burst over the 1896-line
/// `benches/fixtures/long_bullets.md`). Deliberately the COALESCING arm — the
/// one whose headline is dominated by a deliberate 101 ms wait — because that
/// is the shape a work-phases-only roster cannot describe at all, and the
/// shape the coverage floor most needs to hold on. Every law below that needs
/// "a real transaction" uses this one: the fixture is a measurement, not an
/// invention, and its eight numbers sum to its own headline.
fn measured_item241_transaction() -> (SwitchPhases, f32) {
    let mut p = SwitchPhases::default();
    p.record(SwitchPhase::Wait, 101.1);
    p.record(SwitchPhase::Font, 0.0);
    p.record(SwitchPhase::Reshape, 27.2);
    p.record(SwitchPhase::RowGeom, 0.9);
    p.record(SwitchPhase::Schedule, 1.0);
    p.record(SwitchPhase::Atlas, 12.2);
    p.record(SwitchPhase::Acquire, 0.1);
    p.record(SwitchPhase::Present, 0.5);
    (p, 143.0)
}

#[test]
fn record_and_get_roundtrip_per_phase() {
    let mut p = SwitchPhases::default();
    // A fresh accumulator has nothing recorded.
    for ph in SwitchPhase::ORDER {
        assert_eq!(p.get(ph), None);
    }
    p.record(SwitchPhase::Wait, 100.0);
    p.record(SwitchPhase::Font, 0.2);
    p.record(SwitchPhase::Reshape, 6.8);
    p.record(SwitchPhase::RowGeom, 0.9);
    p.record(SwitchPhase::Schedule, 1.5);
    p.record(SwitchPhase::Atlas, 2.0);
    p.record(SwitchPhase::Acquire, 8.3);
    p.record(SwitchPhase::Present, 0.6);
    assert_eq!(p.get(SwitchPhase::Wait), Some(100.0));
    assert_eq!(p.get(SwitchPhase::Font), Some(0.2));
    assert_eq!(p.get(SwitchPhase::Reshape), Some(6.8));
    assert_eq!(p.get(SwitchPhase::RowGeom), Some(0.9));
    assert_eq!(p.get(SwitchPhase::Schedule), Some(1.5));
    assert_eq!(p.get(SwitchPhase::Atlas), Some(2.0));
    assert_eq!(p.get(SwitchPhase::Acquire), Some(8.3));
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
    p.record(SwitchPhase::Wait, 0.0);
    p.record(SwitchPhase::Font, 0.2);
    p.record(SwitchPhase::Reshape, 6.8);
    p.record(SwitchPhase::RowGeom, 0.9);
    p.record(SwitchPhase::Schedule, 1.5);
    p.record(SwitchPhase::Atlas, 2.0);
    p.record(SwitchPhase::Acquire, 8.3);
    p.record(SwitchPhase::Present, 0.6);
    assert_eq!(
        breakdown_readout(&p, 20.3),
        "wait 0.0 · font 0.2 · reshape 6.8 · rowgeom 0.9 · sched 1.5 · atlas 2.0 · \
         acquire 8.3 · present 0.6"
    );
}

#[test]
fn breakdown_readout_shows_dash_for_an_unrecorded_phase() {
    // A partial accumulator (only the reshape-side phases recorded, e.g. a switch
    // whose present frame was skipped) shows `—` for the missing present-side ones.
    let mut p = SwitchPhases::default();
    p.record(SwitchPhase::Wait, 0.0);
    p.record(SwitchPhase::Font, 0.1);
    p.record(SwitchPhase::Reshape, 5.0);
    p.record(SwitchPhase::RowGeom, 0.8);
    assert_eq!(
        breakdown_readout(&p, 6.0),
        "wait 0.0 · font 0.1 · reshape 5.0 · rowgeom 0.8 · sched — · atlas — · \
         acquire — · present —"
    );
}

// ── The roster must cover the transaction it describes ───────────────────

/// THE COVERAGE LAW. A real, fully-recorded transaction's phases must account
/// for at least [`MIN_PHASE_COVERAGE`] of its own headline — and the readout
/// must therefore carry NO `unaccounted` term. Swept over the whole
/// `SwitchPhase::ORDER` roster (every phase recorded, none assumed), against
/// the transaction this item was raised on.
#[test]
fn phase_roster_covers_a_real_transaction() {
    let (phases, total) = measured_item241_transaction();
    for p in SwitchPhase::ORDER {
        assert!(
            phases.get(p).is_some(),
            "{} is in ORDER but the measured transaction never records it — a \
             phase nobody feeds is a column of `—` and a hole in the coverage",
            p.label()
        );
    }
    let coverage = phases
        .coverage(total)
        .expect("a positive total has coverage");
    assert!(
        coverage >= MIN_PHASE_COVERAGE,
        "the phase roster accounts for only {:.1}% of its own {total:.1} ms \
         headline (floor {:.0}%): {} — a breakdown that does not add up to the \
         number it sits under names nothing",
        coverage * 100.0,
        MIN_PHASE_COVERAGE * 100.0,
        breakdown_readout(&phases, total)
    );
    assert!(
        !breakdown_readout(&phases, total).contains("unaccounted"),
        "a covered transaction must not carry a shortfall term"
    );
}

/// NON-VACUITY, against the exact readout that raised this rule: a
/// work-phases-only roster under a user's own 117.2 ms headline. The law above
/// must go RED on this, and the readout must name the gap out loud —
/// otherwise the coverage floor is decoration.
#[test]
fn the_shipped_2026_08_03_blind_spot_fails_the_coverage_floor() {
    // The reported HUD line: `font 0.0 · reshape 0.1 · rowgeom 0.0 · atlas
    // 1.8 · present 0.2` beneath `theme worst 117.2 ms`. No `Wait`, no
    // `Schedule`, no `Acquire` — the segments carrying the other ~115 ms.
    let mut p = SwitchPhases::default();
    p.record(SwitchPhase::Font, 0.0);
    p.record(SwitchPhase::Reshape, 0.1);
    p.record(SwitchPhase::RowGeom, 0.0);
    p.record(SwitchPhase::Atlas, 1.8);
    p.record(SwitchPhase::Present, 0.2);
    let total = 117.2;
    let coverage = p.coverage(total).unwrap();
    assert!(
        coverage < MIN_PHASE_COVERAGE,
        "the pre-241 roster covered {:.1}% of its headline and the floor let it pass",
        coverage * 100.0
    );
    let missing = p
        .shortfall_ms(total)
        .expect("below the floor, so a shortfall");
    assert!(
        (missing - 115.1).abs() < 0.05,
        "the unnamed remainder is the whole defect: expected ~115.1 ms, got {missing:.1}"
    );
    assert!(
        breakdown_readout(&p, total).ends_with("· unaccounted 115.1"),
        "the readout must name its own blind spot: {}",
        breakdown_readout(&p, total)
    );
}

/// A transaction with NO total (a degenerate zero) has no coverage claim to
/// make and must not manufacture a shortfall — the guard cannot divide by zero
/// into a false accusation.
#[test]
fn a_zero_total_transaction_makes_no_coverage_claim() {
    let (phases, _) = measured_item241_transaction();
    assert_eq!(phases.coverage(0.0), None);
    assert_eq!(phases.shortfall_ms(0.0), None);
    assert!(!breakdown_readout(&phases, 0.0).contains("unaccounted"));
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
    p.record(SwitchPhase::Wait, 143.0);
    p.record(SwitchPhase::Font, 0.2);
    p.record(SwitchPhase::Reshape, 6.8);
    p.record(SwitchPhase::RowGeom, 0.9);
    p.record(SwitchPhase::Schedule, 1.5);
    p.record(SwitchPhase::Atlas, 2.0);
    p.record(SwitchPhase::Acquire, 0.7);
    p.record(SwitchPhase::Present, 0.6);
    // The BREAKDOWN describes the WORST transaction, so its coverage is graded
    // against the worst headline (155.2), never the latest — a shortfall term
    // keyed to the wrong number would accuse the wrong transaction.
    assert_eq!(
        settle_lines(Some(SwitchReport {
            latest: CompletedSwitch::new(12.0, p),
            worst: CompletedSwitch::new(155.2, p),
        })),
        vec![
            "theme latest 12.0 ms".to_string(),
            "theme worst 155.2 ms".to_string(),
            "wait 143.0 · font 0.2 · reshape 6.8 · rowgeom 0.9 · sched 1.5 · atlas 2.0 · \
             acquire 0.7 · present 0.6"
                .to_string(),
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
fn fake_clock_exact_switch_window_boundary_survives_then_expires() {
    let clock = fake_clock();
    let t0 = crate::clock::Clock::now(&clock);
    let mut history = SwitchHistory::default();
    history.insert(t0, 42.0, switch(42.0));
    clock.advance(SWITCH_WINDOW);
    assert!(
        history.report(crate::clock::Clock::now(&clock)).is_some(),
        "exactly SWITCH_WINDOW stays"
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
