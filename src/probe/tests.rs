//! Tests for the live probe, carved out of `probe.rs` so the module's own
//! production size measures production code. Test files are exempt from the
//! file-size ceilings by `code-health.py`'s `production()` rule.

use super::*;

#[test]
fn parse_covers_every_verb_and_appends_the_terminating_quit() {
    let steps = parse_script("keys Cmd-T Down; sleep 250; shot dwell-1").expect("parses");
    assert_eq!(steps.len(), 4, "keys + sleep + shot + the appended quit");
    match &steps[0] {
        Step::Keys(chords) => assert_eq!(chords.len(), 2),
        other => panic!("expected Keys, got {other:?}"),
    }
    assert_eq!(steps[1], Step::Sleep(250));
    assert_eq!(steps[2], Step::Shot("dwell-1".into()));
    assert_eq!(steps[3], Step::Quit, "a script always terminates");
}

#[test]
fn parse_keeps_an_explicit_trailing_quit_single() {
    let steps = parse_script("keys Down; quit").expect("parses");
    assert_eq!(steps.len(), 2);
    assert_eq!(steps.last(), Some(&Step::Quit));
}

#[test]
fn parse_covers_mouse_move_and_wheel() {
    let steps = parse_script("move 900 640; wheel -2; wheel 1").expect("parses");
    assert_eq!(steps[0], Step::MouseMove(900.0, 640.0));
    assert_eq!(steps[1], Step::Wheel(-2.0));
    assert_eq!(steps[2], Step::Wheel(1.0));
    assert_eq!(steps.last(), Some(&Step::Quit), "still terminates");
    for bad in ["move 900", "move a b", "move 1 2 3", "wheel nudge"] {
        assert!(parse_script(bad).is_err(), "{bad:?} should be rejected");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn probe_window_is_smaller_than_the_center_stage_default() {
    assert!(
        std::hint::black_box(PROBE_LOGICAL_W) < 1200.0 && PROBE_LOGICAL_H < 800.0,
        "probe window {PROBE_LOGICAL_W}x{PROBE_LOGICAL_H} must be smaller than the 1200x800 default"
    );
    assert!(
        std::hint::black_box(PROBE_LOGICAL_W) >= 640.0 && PROBE_LOGICAL_H >= 400.0,
        "probe window must stay large enough to render a real page + picker"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn flight_recorder_arms_and_appends_a_stamped_line() {
    let _g = crate::testlock::serial();
    let path = std::env::temp_dir().join(format!("awl-flight-test-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    assert!(!flight_active(), "flight starts disarmed");
    assert!(
        !live_active(),
        "no probe in a unit test, so recording() == flight_active()"
    );
    arm_flight(&path);
    assert!(flight_active(), "arming flips the flag");
    assert!(
        recording(),
        "recording() is true under the flight recorder alone"
    );
    trace(format_args!("preview Galah -> Magpie {}", 42));
    let body = std::fs::read_to_string(&path).expect("the flight file exists");
    assert!(
        body.contains("preview Galah -> Magpie 42"),
        "the traced line landed in the black box, got:\n{body}"
    );
    assert!(
        body.contains("flight-recorder armed"),
        "the header line is present:\n{body}"
    );
    assert!(
        body.lines()
            .all(|l| l.starts_with("+") && l.contains("ms ")),
        "every line carries the +<ms> stamp:\n{body}"
    );
    FLIGHT_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut s) = FLIGHT_SINK.lock() {
        *s = None;
    }
    let _ = std::fs::remove_file(&path);
    assert!(!recording(), "disarmed again — no leak into sibling tests");
}

#[test]
fn parse_rejects_the_malformed_forms_by_name() {
    for (spec, needle) in [
        ("", "empty script"),
        ("dance", "unknown step"),
        ("keys", "needs a chord spec"),
        ("sleep soon", "needs ms"),
        ("shot ../escape", "shot"),
        ("keys NotAChord-", "chord"),
    ] {
        let err = parse_script(spec)
            .expect_err(spec)
            .to_string()
            .to_lowercase();
        assert!(
            err.contains(&needle.to_lowercase()),
            "{spec:?} should fail mentioning {needle:?}, got: {err}"
        );
    }
}

#[test]
fn parse_covers_the_latency_step() {
    let steps = parse_script("keys Down; latency").expect("parses");
    assert_eq!(steps[1], Step::Latency);
    assert_eq!(steps.last(), Some(&Step::Quit), "still terminates");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn movement_latency_mark_and_present_produce_a_sample_and_distribution() {
    let _g = crate::testlock::serial();
    if let Ok(mut p) = LATENCY_PENDING.lock() {
        p.clear();
    }
    if let Ok(mut s) = LATENCY_SAMPLES.lock() {
        s.clear();
    }
    if let Ok(mut b) = BAND_SETTLE_PENDING.lock() {
        *b = None;
    }

    let path = std::env::temp_dir().join(format!("awl-latency-test-{}.log", std::process::id()));
    let _ = std::fs::remove_file(&path);
    arm_flight(&path);
    assert!(
        recording(),
        "the flight recorder alone is enough to arm `recording()`"
    );

    note_presented_frame();
    assert!(latency_distribution().is_none(), "nothing sampled yet");

    mark_movement_input();
    assert!(
        !LATENCY_PENDING.lock().unwrap().is_empty(),
        "the mark armed the clock"
    );
    assert!(
        BAND_SETTLE_PENDING.lock().unwrap().is_some(),
        "the same movement arms the decorative-band endpoint"
    );
    note_band_phase(0.4);
    assert!(
        BAND_SETTLE_PENDING.lock().unwrap().is_some(),
        "an intermediate prepared phase stays pending"
    );
    note_band_phase(1.0);
    assert!(
        BAND_SETTLE_PENDING.lock().unwrap().is_none(),
        "the prepared final phase closes input-to-band-settled"
    );
    std::thread::sleep(std::time::Duration::from_millis(2));
    note_presented_frame();
    assert!(
        LATENCY_PENDING.lock().unwrap().is_empty(),
        "closing out clears the pending mark"
    );

    let dist = latency_distribution().expect("one sample now recorded");
    assert!(
        dist.starts_with("n=1"),
        "exactly one sample so far, got: {dist}"
    );
    for field in ["min=", "p50=", "p95=", "max="] {
        assert!(dist.contains(field), "{dist:?} is missing {field:?}");
    }

    let body = std::fs::read_to_string(&path).expect("the flight file exists");
    assert!(
        body.contains("movement-latency"),
        "the sample traced into the black box:\n{body}"
    );
    assert!(
        body.contains("theme-band phase=0.400") && body.contains("theme-band input-to-settled"),
        "the recorder retains the prepared band phase and endpoint:\n{body}"
    );

    note_presented_frame();
    assert!(
        latency_distribution().unwrap().starts_with("n=1"),
        "a no-op note must not add a sample"
    );

    FLIGHT_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut s) = FLIGHT_SINK.lock() {
        *s = None;
    }
    if let Ok(mut p) = LATENCY_PENDING.lock() {
        p.clear();
    }
    if let Ok(mut s) = LATENCY_SAMPLES.lock() {
        s.clear();
    }
    if let Ok(mut b) = BAND_SETTLE_PENDING.lock() {
        *b = None;
    }
    let _ = std::fs::remove_file(&path);
    assert!(!recording(), "disarmed again — no leak into sibling tests");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn movement_latency_is_a_no_op_outside_recording() {
    let _g = crate::testlock::serial();
    assert!(!recording(), "no probe/flight armed in a plain unit test");
    if let Ok(mut p) = LATENCY_PENDING.lock() {
        p.clear();
    }
    if let Ok(mut s) = LATENCY_SAMPLES.lock() {
        s.clear();
    }
    if let Ok(mut b) = BAND_SETTLE_PENDING.lock() {
        *b = None;
    }
    mark_movement_input();
    assert!(
        LATENCY_PENDING.lock().unwrap().is_empty(),
        "a mark outside recording never arms"
    );
    note_presented_frame();
    note_band_phase(1.0);
    assert!(latency_distribution().is_none());
    assert!(BAND_SETTLE_PENDING.lock().unwrap().is_none());
}

/// A single overwritten slot reports `n=1` for a burst of N reshaping
/// inputs regardless of N — the exact shape that once undercounted a real
/// burst's cost by roughly two orders of magnitude against the
/// `--bench-theme-burst` figure, which never touches this probe.
///
/// Sweeps burst LENGTH rather than one hand-picked count: a single-slot-vs-
/// queue bug is invisible at `n=1` and only some off-by-one variants would
/// show at `n=2`; this checks 1, 2, 3, 8, and 9 (`--bench-theme-burst`'s own
/// burst length).
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn movement_latency_burst_of_n_reports_n_not_one() {
    let _g = crate::testlock::serial();
    let path =
        std::env::temp_dir().join(format!("awl-latency-burst-test-{}.log", std::process::id()));

    for n in [1usize, 2, 3, 8, 9] {
        if let Ok(mut p) = LATENCY_PENDING.lock() {
            p.clear();
        }
        if let Ok(mut s) = LATENCY_SAMPLES.lock() {
            s.clear();
        }
        let _ = std::fs::remove_file(&path);
        arm_flight(&path);
        assert!(recording(), "flight recorder arms recording() for n={n}");

        // ZERO-GAP shape: every mark fires before ANY of the burst's
        // presents close one out — the exact arrival order a fast
        // arrow-key burst produces when input outruns the frame loop, and
        // the shape the historical "n=1 for 8 inputs" bias was measured
        // under. A queue must survive this ordering, not just the tidy
        // alternating one; an overwriting single slot collapses it to a
        // single sample no matter how many presents follow.
        for _ in 0..n {
            mark_movement_input();
        }
        for _ in 0..n {
            note_presented_frame();
        }

        let dist = latency_distribution().unwrap_or_default();
        assert!(
            dist.starts_with(&format!("n={n}")),
            "a burst of {n} reshaping inputs must report n={n}, got: {dist:?}"
        );
        assert!(
            LATENCY_PENDING.lock().unwrap().is_empty(),
            "every mark in the n={n} burst was closed out, none left dangling"
        );

        FLIGHT_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
        if let Ok(mut s) = FLIGHT_SINK.lock() {
            *s = None;
        }
    }

    if let Ok(mut p) = LATENCY_PENDING.lock() {
        p.clear();
    }
    if let Ok(mut s) = LATENCY_SAMPLES.lock() {
        s.clear();
    }
    if let Ok(mut b) = BAND_SETTLE_PENDING.lock() {
        *b = None;
    }
    let _ = std::fs::remove_file(&path);
    assert!(!recording(), "disarmed again — no leak into sibling tests");
}

/// THE PROBE'S REFERENCE MUST BE THE LIVE `App`, ON THE WINDOW'S OWN CANVAS.
///
/// The bug this law is named for: the matrix graded a real window (harness tier 3)
/// against a bare `--screenshot` replay of the shared core (tier 1), on a canvas
/// this script chose rather than the one the window rendered on. Both halves are
/// wrong, and neither fails loudly — the whole picture is simply a different SIZE,
/// so nearly every uniform block still matches and the mismatch surfaces as a
/// handful of blocks where a glyph landed somewhere the reference has bare page.
/// That reads exactly like a stale/vanished surface and was chased as one.
///
/// So: `ref_for` goes through `--screenshot-app` (tier 2 — a real headless `App`,
/// which makes the same launch-time render decisions the window did), and takes
/// its canvas from the `surface=WxH dpi=S` the shot line reports.
///
/// Launch and replay now share [`crate::range::ZOOM.default`], but equal zoom is
/// not permission to collapse the tiers: launch-time App decisions and future
/// configured state still belong to the live-App reference. This law therefore
/// keeps grading like with like after the size mismatch itself is gone.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn probe_reference_is_the_live_app_on_the_windows_own_canvas() {
    let script = probe_script_source();
    let body = between(&script, "ref_for() {", "\n}\n").expect("ref_for is defined in the script");

    assert!(
        body.contains("--screenshot-app"),
        "live-probe.sh's ref_for must render its reference through the live-`App` \
         door (--screenshot-app), not the replay core: equal default zoom does not \
         make the replay core a launch-time App oracle. ref_for was:\n{body}",
    );
    assert!(
        !body.contains("--screenshot "),
        "no bare --screenshot (replay-core) reference may remain in ref_for:\n{body}"
    );
    assert!(
        body.contains("--capture-dpi"),
        "ref_for must pin the reference's dpi — layout is not invariant under \
         trading surface pixels for dpi, so a reference at the window's LOGICAL \
         size and dpi 1 is not the HiDPI window's picture:\n{body}"
    );
    assert!(
        !body.contains("PROBE_CANVAS"),
        "ref_for must take its canvas from the window's reported surface, not from \
         this script's own PROBE_CANVAS:\n{body}"
    );

    let check = between(&script, "check_shot() {", "\n}\n").expect("check_shot is defined");
    for field in ["surface=", "dpi="] {
        assert!(
            check.contains(field),
            "check_shot must read `{field}` off the shot line and hand it to ref_for \
             (the window is the authority on its own canvas):\n{check}"
        );
    }
}

/// The `LIVE-PROBE shot …` tail the app WRITES is the tail the script READS.
/// A rename on either side would not fail loudly — the script would extract an
/// empty canvas — so the field names are pinned rather than trusted.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn probe_shot_lines_carry_the_canvas_fields_the_script_parses() {
    let tag = crate::app::probe::surface_tag(1800, 1200, 2.0);
    assert_eq!(
        tag, "surface=1800x1200 dpi=2",
        "the wire format, spelled once"
    );

    let script = probe_script_source();
    let check = between(&script, "check_shot() {", "\n}\n").expect("check_shot is defined");
    for field in ["surface=", "dpi="] {
        assert!(
            tag.contains(field) && check.contains(field),
            "producer (`app::probe::surface_tag` -> {tag:?}) and consumer \
             (live-probe.sh check_shot) must agree on `{field}`"
        );
    }
}

/// The script's `PROBE_CANVAS` sizes the probe WINDOW and must stay in lockstep
/// with [`PROBE_LOGICAL_W`]/[`PROBE_LOGICAL_H`]. Previously a comment asking for
/// lockstep and a claim that a drift would surface on its own; it now fails here.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn probe_canvas_in_the_script_matches_the_window_constants() {
    let script = probe_script_source();
    let want = format!(
        "PROBE_CANVAS=\"{}x{}\"",
        PROBE_LOGICAL_W as u32, PROBE_LOGICAL_H as u32
    );
    assert!(
        script.contains(&want),
        "live-probe.sh must declare {want} to match PROBE_LOGICAL_W/H"
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn probe_script_source() -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("live-probe.sh");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The body of a shell function: everything between `head` and the first line that
/// is exactly `}` after it. Returns `None` if the opener is absent, so a renamed
/// function fails the law by name instead of vacuously passing on an empty body.
#[cfg(not(target_arch = "wasm32"))]
fn between(haystack: &str, head: &str, close: &str) -> Option<String> {
    let start = haystack.find(head)? + head.len();
    let rest = &haystack[start..];
    let end = rest.find(close)?;
    Some(rest[..end].to_string())
}
