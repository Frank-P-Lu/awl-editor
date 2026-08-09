use super::*;

#[test]
fn parse_sel_orders_endpoints_and_rejects_malformed() {
    assert_eq!(parse_sel("0:0-2:3").unwrap(), ((0, 0), (2, 3)));
    assert_eq!(parse_sel("2:3-0:0").unwrap(), ((0, 0), (2, 3)));
    assert_eq!(parse_sel(" 1:2 - 1:5 ").unwrap(), ((1, 2), (1, 5)));
    parse_sel("0:0").unwrap_err();
    parse_sel("00-23").unwrap_err();
    parse_sel("a:b-c:d").unwrap_err();
}

#[test]
fn parse_steps_reads_ms_and_rejects_junk() {
    assert_eq!(parse_steps("0,16,50,150").unwrap(), vec![0, 16, 50, 150]);
    // Whitespace + trailing/empty entries are tolerated.
    assert_eq!(parse_steps(" 0 , 30 ,").unwrap(), vec![0, 30]);
    // Empty / all-blank / non-numeric are errors.
    parse_steps("").unwrap_err();
    parse_steps("  ,  ").unwrap_err();
    parse_steps("0,x,2").unwrap_err();
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn parse_soak_duration_accepts_short_runs_and_rejects_non_positive() {
    assert_eq!(
        parse_soak_seconds("0.25").unwrap(),
        std::time::Duration::from_millis(250)
    );
    assert_eq!(
        parse_soak_seconds("900").unwrap(),
        crate::soak_gpu::DEFAULT_DURATION
    );
    for bad in ["0", "-1", "NaN", "inf", "nope"] {
        assert!(parse_soak_seconds(bad).is_err(), "{bad}");
    }
}

#[test]
fn parse_size_accepts_both_separators_and_rejects_zero() {
    assert_eq!(parse_size("2400x1600").unwrap(), (2400, 1600));
    assert_eq!(parse_size("800X600").unwrap(), (800, 600));
    // Missing separator, zero dimension, non-numeric are errors.
    parse_size("1200").unwrap_err();
    parse_size("0x600").unwrap_err();
    parse_size("800x0").unwrap_err();
    parse_size("axb").unwrap_err();
}

#[test]
fn parse_held_dir_accepts_aliases_and_rejects_bad() {
    assert!(parse_held_dir("left").unwrap() == capture::HeldDir::Left);
    assert!(parse_held_dir("L").unwrap() == capture::HeldDir::Left);
    assert!(parse_held_dir("RIGHT").unwrap() == capture::HeldDir::Right);
    assert!(parse_held_dir("u").unwrap() == capture::HeldDir::Up);
    assert!(parse_held_dir("Down").unwrap() == capture::HeldDir::Down);
    assert!(parse_held_dir("sideways").is_err());
    assert!(parse_held_dir("").is_err());
}

#[test]
fn parse_dpi_requires_finite_positive() {
    assert_eq!(parse_dpi("2.0").unwrap(), 2.0);
    assert_eq!(parse_dpi(" 1 ").unwrap(), 1.0);
    // Zero, negative, non-finite, and non-numeric are all errors (mirrors
    // parse_size's non-zero guard).
    parse_dpi("0").unwrap_err();
    parse_dpi("-1.5").unwrap_err();
    parse_dpi("inf").unwrap_err();
    parse_dpi("nan").unwrap_err();
    parse_dpi("x").unwrap_err();
}

#[test]
fn parse_zoom_requires_finite_positive() {
    assert_eq!(parse_zoom("1.6").unwrap(), 1.6);
    assert_eq!(parse_zoom(" 0.5 ").unwrap(), 0.5);
    // Zero, negative, non-finite, and non-numeric are all errors (mirrors
    // parse_dpi's guard) — a NaN factor would otherwise poison every
    // zoom-derived metric downstream.
    parse_zoom("0").unwrap_err();
    parse_zoom("-1").unwrap_err();
    parse_zoom("inf").unwrap_err();
    parse_zoom("nan").unwrap_err();
    parse_zoom("x").unwrap_err();
}

#[test]
fn clamp_zoom_never_returns_non_finite() {
    // The LAST line of defence behind the --zoom / config seams above:
    // `render::clamp_zoom` must yield a finite in-range factor for ANY input.
    // (Tested here beside the zoom-flag seam; render/tests/geometry.rs owns
    // the geometry suite.) NaN — the propagating poison — falls back to the 1.0
    // default; ±inf saturates through the ordinary clamp.
    use crate::range::ZOOM;
    use crate::render::clamp_zoom;
    let (zmin, zmax) = (ZOOM.min, ZOOM.max);
    for z in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -7.0, 1e30] {
        let c = clamp_zoom(z);
        assert!(
            c.is_finite() && (zmin..=zmax).contains(&c),
            "clamp_zoom({z}) -> {c} must be finite in [{zmin}, {zmax}]"
        );
    }
    assert_eq!(clamp_zoom(f32::NAN), 1.0, "NaN falls back to the default");
    assert_eq!(clamp_zoom(f32::INFINITY), zmax, "+inf saturates high");
    assert_eq!(clamp_zoom(f32::NEG_INFINITY), zmin, "-inf saturates low");
    // A normal factor still step-rounds + clamps exactly as before.
    assert!(
        (clamp_zoom(1.234) - 1.2).abs() < 1e-5,
        "step rounding unchanged"
    );
    assert_eq!(clamp_zoom(9.0), zmax);
    assert_eq!(clamp_zoom(0.0), zmin);
}

#[test]
fn parse_measure_requires_positive() {
    assert_eq!(parse_measure("80").unwrap(), 80);
    assert_eq!(parse_measure(" 40 ").unwrap(), 40);
    // Zero and non-numeric are errors (mirrors parse_size's non-zero guard).
    parse_measure("0").unwrap_err();
    parse_measure("-1").unwrap_err();
    parse_measure("x").unwrap_err();
}

#[test]
fn resolve_capture_kind_matches_the_precedence_mode_construction_uses() {
    // No output at all is the windowed editor, whatever else was passed —
    // the flags below would all be inert without an `out` path.
    assert_eq!(
        resolve_capture_kind(false, true, true, true, true, true),
        CaptureKind::Windowed
    );
    // With an output path, held > timeline > motion > screenshot-app >
    // screenshot-frames > plain screenshot, exactly the order `Mode`
    // construction checks.
    assert_eq!(
        resolve_capture_kind(true, true, true, true, true, true),
        CaptureKind::Held
    );
    assert_eq!(
        resolve_capture_kind(true, false, true, true, true, true),
        CaptureKind::Timeline
    );
    assert_eq!(
        resolve_capture_kind(true, false, false, true, true, true),
        CaptureKind::Motion
    );
    assert_eq!(
        resolve_capture_kind(true, false, false, false, true, true),
        CaptureKind::ScreenshotApp
    );
    assert_eq!(
        resolve_capture_kind(true, false, false, false, false, true),
        CaptureKind::ScreenshotFrames
    );
    assert_eq!(
        resolve_capture_kind(true, false, false, false, false, false),
        CaptureKind::Screenshot
    );
}

#[test]
fn single_capture_mode_rejects_conflicts() {
    // Zero or one capture-mode flag is fine.
    ensure_single_capture_mode(&[]).unwrap();
    ensure_single_capture_mode(&["--screenshot"]).unwrap();
    // Two distinct modes — or the same flag twice — is a conflict.
    assert!(ensure_single_capture_mode(&["--screenshot", "--capture-held"]).is_err());
    assert!(ensure_single_capture_mode(&["--screenshot", "--screenshot"]).is_err());
    // The error names every conflicting flag.
    let msg = ensure_single_capture_mode(&["--screenshot", "--screenshot-motion"])
        .unwrap_err()
        .to_string();
    assert!(msg.contains("--screenshot") && msg.contains("--screenshot-motion"));
}

#[test]
fn unused_hooks_flags_only_what_a_mode_drops() {
    // A plain screenshot honors every hook → nothing unused.
    let all = SuppliedHooks {
        sel: true,
        zoom: true,
        scroll: true,
        preedit: true,
        search: true,
        search_case: true,
        search_replace: true,
        capture_size: true,
        capture_dpi: true,
        root: true,
        workspace: true,
        default_folder: true,
    };
    assert!(unused_hooks(CaptureKind::Screenshot, &all).is_empty());

    // Motion threads only keys/file: every other hook is dropped.
    let motion = unused_hooks(CaptureKind::Motion, &all);
    for f in [
        "--sel",
        "--zoom",
        "--scroll",
        "--preedit",
        "--search",
        "--search-case",
        "--search-replace",
        "--capture-size",
        "--capture-dpi",
        "--root",
        "--workspace",
        "--default-folder",
    ] {
        assert!(motion.contains(&f), "motion should drop {f}");
    }

    // Timeline / held carry root + canvas/dpi but still drop the per-frame
    // render hooks and workspace/default-folder.
    for kind in [CaptureKind::Timeline, CaptureKind::Held] {
        let u = unused_hooks(kind, &all);
        assert!(u.contains(&"--sel") && u.contains(&"--search-case"));
        assert!(u.contains(&"--workspace") && u.contains(&"--default-folder"));
        assert!(!u.contains(&"--root"));
        assert!(!u.contains(&"--capture-size") && !u.contains(&"--capture-dpi"));
    }

    // `--screenshot-app` honors canvas/dpi/root/workspace (real
    // `LiveAppSpec` fields), but still drops the per-frame render hooks
    // the live `App` owns via real driving, AND `--default-folder` —
    // `LiveAppSpec` has no slot for either, so a flag this door cannot
    // thread must be REFUSED here rather than silently discarded
    // downstream.
    let app = unused_hooks(CaptureKind::ScreenshotApp, &all);
    for f in [
        "--sel",
        "--zoom",
        "--scroll",
        "--preedit",
        "--search",
        "--search-case",
        "--search-replace",
        "--default-folder",
    ] {
        assert!(app.contains(&f), "--screenshot-app should drop {f}");
    }
    for f in ["--capture-size", "--capture-dpi", "--root", "--workspace"] {
        assert!(!app.contains(&f), "--screenshot-app should honor {f}");
    }

    // `--screenshot-frames` honors ONLY canvas/dpi (real `Mode::
    // ScreenshotFrames` fields) — its document is a stationary backdrop
    // loaded straight off disk, so it drops the per-frame render hooks,
    // AND root/workspace/default-folder, unlike every other real-frame
    // mode above. (`--keys` is refused by its own dedicated check, not
    // this table, so it is not part of this sweep.)
    let frames = unused_hooks(CaptureKind::ScreenshotFrames, &all);
    for f in [
        "--sel",
        "--zoom",
        "--scroll",
        "--preedit",
        "--search",
        "--search-case",
        "--search-replace",
        "--root",
        "--workspace",
        "--default-folder",
    ] {
        assert!(frames.contains(&f), "--screenshot-frames should drop {f}");
    }
    for f in ["--capture-size", "--capture-dpi"] {
        assert!(!frames.contains(&f), "--screenshot-frames should honor {f}");
    }

    // The windowed editor honors project context but not capture hooks.
    let win = unused_hooks(CaptureKind::Windowed, &all);
    assert!(win.contains(&"--sel") && win.contains(&"--capture-size"));
    assert!(!win.contains(&"--root"));
    assert!(!win.contains(&"--workspace") && !win.contains(&"--default-folder"));

    // Nothing supplied → nothing unused, for every mode.
    let none = SuppliedHooks::default();
    for kind in [
        CaptureKind::Windowed,
        CaptureKind::Screenshot,
        CaptureKind::Motion,
        CaptureKind::Timeline,
        CaptureKind::Held,
        CaptureKind::ScreenshotApp,
        CaptureKind::ScreenshotFrames,
    ] {
        assert!(unused_hooks(kind, &none).is_empty());
    }
}
