//! THE LIVE-`App` EVENT→PRESENT TRACE ASSERTION.
//!
//! The defect ("picker selection appears to advance only every second input,
//! with no transition" — Firetail 2026-07-17, Settings 2026-07-26, Commands
//! 2026-08-01) escaped three rounds of green logical-tier laws because every
//! one of them asked the shared core "where did `selected` end up". The core
//! was always right. What was wrong was a link FURTHER DOWN a chain nothing
//! observed end to end.
//!
//! So this law observes the chain itself, through the flight recorder
//! (`crate::probe`) that a user's own repro session writes — the same trace
//! points, the same order, read back as data. Driving is
//! [`App::press_spec_headless`]: real chords through the real keymap into the
//! real `App::apply` (tier 2, `docs/harness-reach.md`).
//!
//! WHAT THIS TIER CAN AND CANNOT SEE, stated rather than glossed. A hermetic
//! `App` has no window and no `Gpu`, so the chain's last three links —
//! `request_frame` (gated on a live window), `prepare_highlight` (emitted from
//! `prepare`, which only a real `Gpu::redraw` runs) and `present` — cannot
//! appear here and are NOT asserted here. They are covered at their own owners:
//! `hybrid_band_snap::a_move_onto_a_settled_band_reports_the_ease_advance_could_not_see`
//! (the prepared highlight endpoint + the ease the pre-prepare `advance` cannot
//! see) and
//! `app::tests::lifecycle::a_prepare_time_band_activity_keeps_the_loop_hot_by_itself`
//! (the follow-up-frame decision that was actually wrong). What THIS law owns is
//! the front half: winit receipt → keymap resolve → `App::apply` → the logical
//! selection, and specifically that each accepted navigation input moves the
//! selection by EXACTLY ONE reachable row — the "dropped input", "repeated
//! input" and "state advanced twice" hypotheses the item names, ruled out by
//! measurement rather than by assumption.

use super::*;
use std::sync::Arc;

/// A hermetic `App` over an `InMemoryFs` — never the real disk, and never a
/// real root to index (`App::new` must never receive `/` and walk the whole
/// filesystem).
fn seeded_fs() -> crate::fs::InMemoryFs {
    crate::fs::InMemoryFs::new()
        .with_dir("/ws")
        .with_dir("/ws/proj")
        .with_dir("/cfg")
}

/// The palette chord for the convention this pass runs under — `native-gate.sh`
/// runs the suite once per convention and the binding differs.
fn palette_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-p",
        crate::convention::Convention::Linux => "C-p",
    }
}

fn nav_app() -> App {
    app_on(
        None,
        "/ws/proj",
        Config {
            path: std::path::PathBuf::from("/cfg/config.toml"),
            workspace: Some(std::path::PathBuf::from("/ws")),
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..Config::empty()
        },
    )
}

/// Read the recorder's lines back, trimming the `+<ms> ` stamp each carries.
fn trace_lines(path: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| match l.split_once("ms ") {
            Some((head, rest)) if head.starts_with('+') => rest.to_string(),
            _ => l.to_string(),
        })
        .collect()
}

/// Drive `spec` into a hermetic `App` with the flight recorder armed, and return
/// its trace. Panics rather than returning an error: a spec that will not parse
/// is a broken law, not a finding.
fn traced(spec: &str) -> Vec<String> {
    // The recorder writes to the REAL disk (it is the user's black box, not an
    // `fs::FileSystem` consumer), so its file lives in a `ScratchDir` that
    // cleans up on every path, panic included.
    let scratch = crate::testscratch::ScratchDir::new(std::env::temp_dir().join(format!(
        "awl-nav-trace-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    )));
    let path = scratch.join("flight.log");
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded_fs()));
    let mut app = nav_app();
    crate::probe::arm_flight_for_test(&path);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        app.press_spec_headless(spec).expect("the spec parses");
        trace_lines(&path)
    }));
    crate::probe::disarm_flight_for_test();
    drop(scratch);
    out.unwrap_or_else(|e| std::panic::resume_unwind(e))
}

/// Every `apply` line's `sel … -> …` pair, as `(before, after)` selected
/// indices, for the lines whose action is `action`. Parsed out of the recorder's
/// own text on purpose — a helper that recomputed the selection from `App` state
/// would be asserting against itself.
fn selection_steps(lines: &[String], action: &str) -> Vec<(Option<usize>, Option<usize>)> {
    let idx = |s: &str| -> Option<usize> {
        // "Some((Command, 3, 108, 0))" -> 3
        let inner = s.strip_prefix("Some((")?;
        inner.split(',').nth(1)?.trim().parse().ok()
    };
    lines
        .iter()
        .filter_map(|l| {
            let rest = l.strip_prefix(&format!("apply {action} sel "))?;
            let (before, after) = rest.split_once(" -> ")?;
            Some((idx(before), idx(after)))
        })
        .collect()
}

/// The front half of the chain, for one tap: the winit/keymap receipt exists,
/// it resolved to the navigation action, and `App::apply` moved the selection by
/// exactly one row. Swept over a run of taps so a defect that only shows on the
/// SECOND input (the whole shape of this report) cannot hide behind a
/// single-step assertion.
#[test]
fn every_navigation_tap_resolves_once_and_advances_exactly_one_reachable_row() {
    let _g = crate::testlock::serial();
    const TAPS: usize = 6;
    let downs = std::iter::repeat_n("Down", TAPS)
        .collect::<Vec<_>>()
        .join(" ");
    let lines = traced(&format!("{} {downs}", palette_chord()));

    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("resolve -> OpenCommandPalette")),
        "the palette chord reached the keymap; trace was:\n{}",
        lines.join("\n")
    );
    let resolves = lines
        .iter()
        .filter(|l| l.starts_with("resolve -> NextLine"))
        .count();
    assert_eq!(
        resolves,
        TAPS,
        "exactly one keymap resolve per tap — no dropped and no repeated input; trace:\n{}",
        lines.join("\n")
    );

    let steps = selection_steps(&lines, "NextLine");
    assert_eq!(
        steps.len(),
        TAPS,
        "exactly one `apply NextLine` per tap — the state advances once, never twice; trace:\n{}",
        lines.join("\n")
    );
    for (i, (before, after)) in steps.iter().enumerate() {
        assert_eq!(
            (*before, *after),
            (Some(i), Some(i + 1)),
            "tap {} must step the selection {i} -> {}, got {before:?} -> {after:?}; trace:\n{}",
            i + 1,
            i + 1,
            lines.join("\n")
        );
    }
}

/// RAPID ALTERNATION — the cadence the item names alongside taps and held
/// repeat. Up/Down/Up/Down from a row in the middle must land back where it
/// started after each PAIR, and every single input must still be exactly one
/// reachable row. An every-other-input defect in the state tier would show here
/// as a drift; it does not, which is what sends the hunt downstream.
#[test]
fn rapid_alternating_up_down_moves_exactly_one_row_per_input_and_never_drifts() {
    let _g = crate::testlock::serial();
    let lines = traced(&format!(
        "{} Down Down Down Up Down Up Down Up",
        palette_chord()
    ));

    let mut at: Option<usize> = None;
    let mut inputs = 0usize;
    for line in &lines {
        for action in ["NextLine", "PreviousLine"] {
            let Some(rest) = line.strip_prefix(&format!("apply {action} sel ")) else {
                continue;
            };
            let Some((before, after)) = rest.split_once(" -> ") else {
                continue;
            };
            let idx = |s: &str| -> Option<usize> {
                s.strip_prefix("Some((")?
                    .split(',')
                    .nth(1)?
                    .trim()
                    .parse()
                    .ok()
            };
            let (b, a) = (idx(before), idx(after));
            if let Some(prev) = at {
                assert_eq!(
                    b,
                    Some(prev),
                    "each input starts from where the previous one left the selection; trace:\n{}",
                    lines.join("\n")
                );
            }
            let (b, a) = (b.expect("a card is open"), a.expect("a card is open"));
            let want = if action == "NextLine" { b + 1 } else { b - 1 };
            assert_eq!(
                a,
                want,
                "{action} must move exactly one reachable row (from {b}); trace:\n{}",
                lines.join("\n")
            );
            at = Some(a);
            inputs += 1;
        }
    }
    assert_eq!(inputs, 8, "every one of the eight inputs was accepted");
    assert_eq!(
        at,
        Some(2),
        "three Down (row 3) then Up/Down/Up/Down/Up lands on row 2 — every input \
         counted, none doubled, none lost"
    );
}

/// ANTI-VACUITY. The whole law rests on the recorder actually writing the chain
/// while `recording()` is armed — and writing NOTHING when it is not, which is
/// the property that keeps a normal launch byte-identical. A silently disarmed
/// recorder would make every assertion above pass over an empty file.
#[test]
fn the_chain_is_written_only_while_the_recorder_is_armed() {
    let _g = crate::testlock::serial();
    assert!(
        !crate::probe::recording(),
        "no probe or recorder is armed in a plain unit test"
    );
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded_fs()));
    let mut app = nav_app();
    app.press_spec_headless(&format!("{} Down", palette_chord()))
        .expect("parses");
    drop(_fs);

    let lines = traced(&format!("{} Down", palette_chord()));
    for needle in [
        "resolve -> OpenCommandPalette",
        "resolve -> NextLine",
        "apply NextLine sel ",
    ] {
        assert!(
            lines.iter().any(|l| l.starts_with(needle)),
            "the armed recorder writes {needle:?}; trace:\n{}",
            lines.join("\n")
        );
    }
    assert!(
        !crate::probe::recording(),
        "the recorder is put away again — no leak into a sibling test"
    );
}
