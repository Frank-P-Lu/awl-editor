use super::super::*;
use super::keyspec;
use crate::testscratch::ScratchDir;

#[test]
fn both_capture_doors_report_provisional_fresh_identity_as_untitled() {
    let _serial = crate::testlock::serial();
    let mut fresh = Buffer::scratch();
    fresh.start_fresh_doc(std::path::PathBuf::from("/notes"));

    let folded = capture_fold::buffers_info(2, Some(&fresh));
    assert_eq!(folded.open, 2);
    assert_eq!(folded.active.as_deref(), Some("untitled"));

    let scratch = capture_fold::buffers_info(1, Some(&Buffer::scratch()));
    assert_eq!(scratch.active.as_deref(), Some("scratch"));
}

fn assert_shared_fold_keeps_faceted_hug_roster(
    current: &std::path::Path,
    root: &std::path::Path,
    config: &Config,
    headings: &str,
) {
    let mut buffer = Buffer::from_file(current);
    let corpus = crate::index::build_index(root);
    let mut keymap =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = ReplaySession::new(
        ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        root,
        Some(root),
        config,
        None,
        &mut keymap,
    );
    for chord in crate::keyspec::parse_chords(headings).expect("headings chords") {
        session
            .apply_chord(&chord)
            .expect("shared-fold chord applies");
    }
    let summon_roster = session
        .journey()
        .card()
        .and_then(crate::overlay::OverlayState::hug_roster)
        .expect("real Go-to summon has an unlensed hug corpus");
    let folded = fold_capture_state(&session, project_info(root, &None, None, config));
    let folded_roster = folded
        .overlay_hug_roster
        .as_ref()
        .expect("shared capture fold preserves the Go-to hug corpus");
    assert!(std::sync::Arc::ptr_eq(&summon_roster, folded_roster));
    assert!(
        folded_roster
            .primary
            .iter()
            .any(|row| row.contains("very-long-project-folder-name")),
        "shared fold carries the wide All corpus, not the narrow Headings roster"
    );
}

/// Both ordinary capture doors must carry the summon-time faceted corpus into
/// their settled `ViewState`.  The storyboard/live fold has a typed seam;
/// one-shot `--screenshot --keys` still owns its short replay fold locally.
/// Drive the real Go-to path through each so deleting either enrolment makes
/// the wide All corpus disappear when Headings is selected.
#[test]
fn faceted_hug_roster_survives_both_one_shot_and_shared_capture_folds() {
    let _serial = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-faceted-hug-fold-{}", std::process::id())),
    );
    let root = dir.join("workspace");
    let long_folder = root.join("archive/very-long-project-folder-name");
    std::fs::create_dir_all(&long_folder).expect("seeded root");
    let current = root.join("current.md");
    std::fs::write(&current, "# Current\n").expect("seed current file");
    std::fs::write(long_folder.join("old.md"), "# Old\n").expect("seed nested file");
    let config = Config::empty();
    let all = "s-p g o Space t o Enter";
    let headings = "s-p g o Space t o Enter Right Right";

    assert_shared_fold_keeps_faceted_hug_roster(&current, &root, &config, headings);

    if capture::build_oracle(&Buffer::from_file(&current), &CaptureOpts::default()).is_none() {
        eprintln!("skipping faceted-hug capture fold pixels: no wgpu adapter");
        return;
    }
    let capture = |label: &str, spec: &str| {
        let out = dir.join(format!("{label}.png"));
        capture_screenshot(
            out.clone(),
            Some(current.clone()),
            CaptureOpts::default(),
            keyspec::parse_keys(spec).expect("one-shot chords"),
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
            Some(root.clone()),
            Some(root.clone()),
            dir.join("outside-corpus-config"),
            Config::empty(),
            true,
        )
        .expect("strict one-shot capture succeeds");
        serde_json::from_str::<serde_json::Value>(
            &std::fs::read_to_string(out.with_extension("json")).expect("sidecar"),
        )
        .expect("sidecar parses")
    };
    let band = |sidecar: &serde_json::Value| {
        let band = &sidecar["overlay"]["window"]["band"];
        (
            band["x"].as_f64().expect("band x"),
            band["w"].as_f64().expect("band w"),
        )
    };
    let mirrors: Vec<&str> = crate::theme::THEMES
        .iter()
        .filter(|world| world.render_caps.card_anchor.mirrors_growth())
        .map(|world| world.name)
        .collect();
    assert!(
        !mirrors.is_empty(),
        "the shipping roster must enroll at least one right-anchored card"
    );
    for world in mirrors {
        let _world = crate::theme::WorldPin::world(world).expect("enrolled world exists");
        let all_json = capture(&format!("{world}-all"), all);
        let headings_json = capture(&format!("{world}-headings"), headings);
        assert_eq!(
            all_json["overlay"]["lens"], "all",
            "[{world}] one-shot starts at All"
        );
        assert_eq!(
            headings_json["overlay"]["lens"], "headings",
            "[{world}] one-shot navigation reaches Headings"
        );
        let all_band = band(&all_json);
        let headings_band = band(&headings_json);
        assert!(
            (all_band.0 - headings_band.0).abs() <= 0.01
                && (all_band.1 - headings_band.1).abs() <= 0.01,
            "[{world}] one-shot fold lost the summon-time hug corpus across All -> Headings: \
             all={all_band:?}, headings={headings_band:?}"
        );
    }
}

#[test]
fn palette_language_tag_capture_photographs_the_toast_it_reports() {
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-language-toast-{}", std::process::id())),
    );
    let fixture = dir.join("note.md");
    std::fs::write(&fixture, "# 你好\n\nこれは日本語です。\n").unwrap();
    if capture::build_oracle(&Buffer::from_file(&fixture), &CaptureOpts::default()).is_none() {
        eprintln!("skipping language-toast capture: no wgpu adapter");
        return;
    }

    let quiet = dir.join("quiet.png");
    capture_screenshot(
        quiet.clone(),
        Some(fixture.clone()),
        CaptureOpts::default(),
        Vec::new(),
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("quiet capture succeeds");

    let noticed = dir.join("noticed.png");
    capture_screenshot(
        noticed.clone(),
        Some(fixture),
        CaptureOpts::default(),
        keyspec::parse_keys("s-p t a g Space d o c u m e n t Space l a n g u a g e Enter").unwrap(),
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("language-tag capture succeeds");

    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(noticed.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        sidecar["notice"],
        serde_json::json!({
            "text": "Document language: Japanese",
            "kind": "toast"
        }),
        "the one-shot capture fold must carry the nested palette action's notice to the renderer"
    );
    assert!(
        sidecar["text"]
            .as_str()
            .unwrap()
            .starts_with("---\nlang: ja\n---\n"),
        "the same artifact reports the metadata edit the toast acknowledges"
    );
    assert_ne!(
        std::fs::read(quiet).unwrap(),
        std::fs::read(noticed).unwrap(),
        "PRESENCE: the language toast must change the frame's actual pixels"
    );
}

#[test]
fn capture_scenario_search_replace_replay_lands_in_the_sidecar_search_block() {
    // SIDECAR EVIDENCE for the shared search/replace routing: one `--keys`
    // spec drives open + query typing + replace-field reveal + replacement
    // typing + replace-one through the REAL `capture_screenshot` seam, and
    // every operation's outcome is assertable from the sidecar `search`
    // block + `text` — the round's done-criteria witness. Real disk +
    // capture -> hold the fs TEST_LOCK like the sticky-root test above.
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-search-replay-{}", std::process::id())),
    );
    let fixture = dir.join("doc.txt");
    std::fs::write(&fixture, "line one\nline two\nline three\n").unwrap();
    let out = dir.join("cap.png");
    let keys = keyspec::parse_keys("C-s l i n e Tab r o w Enter").unwrap();
    capture_screenshot(
        out.clone(),
        Some(fixture),
        CaptureOpts::default(),
        keys,
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        true, // strict: the whole spec must ride real seams
    )
    .expect("capture succeeds");
    let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let s = &v["search"];
    assert_eq!(s["query"].as_str().unwrap(), "line", "typed query");
    assert!(s["active"].as_bool().unwrap());
    assert!(
        s["replace_active"].as_bool().unwrap(),
        "Tab revealed the row"
    );
    assert_eq!(
        s["replacement"].as_str().unwrap(),
        "row",
        "typed replacement"
    );
    assert!(
        s["editing_replacement"].as_bool().unwrap(),
        "focus is in the field"
    );
    assert_eq!(
        s["hit_count"].as_u64().unwrap(),
        2,
        "one of three matches was replaced"
    );
    assert!(
        v["text"].as_str().unwrap().starts_with("row one\nline two"),
        "replace-one swapped exactly the first match"
    );
    assert_eq!(
        v["cursor"]["line"].as_u64().unwrap(),
        1,
        "caret advanced to the next match"
    );
    assert_eq!(v["cursor"]["col"].as_u64().unwrap(), 0);
}

#[test]
fn capture_sidecar_traces_permissive_replay_skips_and_strict_writes_nothing() {
    // The sidecar, not stderr, is the capture verifier's state oracle. Drive a
    // real Move accept through the same screenshot door users invoke: its
    // settled overlay otherwise looks exactly like a successful live move.
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-replay-skips-{}", std::process::id())),
    );
    std::fs::create_dir_all(dir.join("archive")).unwrap();
    let fixture = dir.join("note.md");
    std::fs::write(&fixture, "note\n").unwrap();
    if capture::build_oracle(&Buffer::from_file(&fixture), &CaptureOpts::default()).is_none() {
        eprintln!("skipping replay-skip sidecar capture: no wgpu adapter");
        return;
    }
    let keys = keyspec::parse_keys("s-p m o v e Enter Enter").unwrap();
    let keymap =
        || crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);

    let permissive = dir.join("permissive.png");
    capture_screenshot(
        permissive.clone(),
        Some(fixture.clone()),
        CaptureOpts::default(),
        keys.clone(),
        keymap(),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("permissive replay captures past a live-only move");
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(permissive.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        sidecar["replay_skips"],
        serde_json::json!([{ "effect": "overlay_accept", "action": "Newline" }]),
        "the settled sidecar names the skipped move and its originating action"
    );
    assert!(
        fixture.exists() && !dir.join("archive/note.md").exists(),
        "the test fixture proves the move was really skipped"
    );

    let ordinary = dir.join("ordinary.png");
    capture_screenshot(
        ordinary.clone(),
        Some(fixture.clone()),
        CaptureOpts::default(),
        keyspec::parse_keys("X s-s").unwrap(),
        keymap(),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("ordinary capture succeeds");
    let ordinary_sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(ordinary.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        ordinary_sidecar["replay_skips"],
        serde_json::json!([{ "effect": "save", "action": "Save" }])
    );
    assert_eq!(
        ordinary_sidecar["text"].as_str(),
        Some("Xnote\n"),
        "the sidecar shows the in-session edit"
    );
    assert_eq!(
        std::fs::read_to_string(&fixture).unwrap(),
        "note\n",
        "ordinary capture records Save without mutating the input"
    );

    let strict = dir.join("strict.png");
    let err = capture_screenshot(
        strict.clone(),
        Some(fixture),
        CaptureOpts::default(),
        keys,
        keymap(),
        Some(dir.to_path_buf()),
        None,
        dir.join("notes"),
        Config::empty(),
        true,
    )
    .expect_err("strict replay aborts at the same move seam");
    assert!(err.to_string().contains("`overlay_accept`"), "{err}");
    assert!(!strict.exists() && !strict.with_extension("json").exists());
}

/// THE PRIMARY LAW: `ReplaySession` used to resolve `root` /
/// `workspace` / `corpus` ONCE before replay and hold them fixed for its whole
/// lifetime. The *accepted* sidecar location (`run::project_info`
/// re-derives it whole on a Project accept), but a chord applied AFTER the
/// accept still read the LAUNCH root's file index — a `Cmd-O` following a
/// Switch-project quietly listed the wrong tree while the capture reported
/// success. Drives the REAL `capture_screenshot` door in BOTH conventions,
/// mirroring `app::files::tests::
/// switch_project_driven_by_real_chords_through_apply_repoints_the_workspace`
/// (the model this test follows) — mac `s-S-p`, linux `C-S-p`, then Goto's own
/// `s-o`/`C-o`.
#[test]
fn keys_capture_switch_project_then_goto_lists_the_new_roots_files() {
    let _fs = crate::testlock::serial();
    for convention in [
        crate::convention::Convention::Mac,
        crate::convention::Convention::Linux,
    ] {
        let dir = ScratchDir::new(std::env::temp_dir().join(format!(
            "awl-switch-project-goto-{convention:?}-{}",
            std::process::id()
        )));
        std::fs::create_dir_all(dir.join("old-ws/proj-a")).unwrap();
        std::fs::create_dir_all(dir.join("old-ws/sibling")).unwrap();
        std::fs::write(dir.join("old-ws/proj-a/keep.md"), "keep").unwrap();
        std::fs::write(dir.join("old-ws/sibling/target.md"), "target").unwrap();

        let (switch_project, open_goto) = match convention {
            crate::convention::Convention::Mac => ("s-S-p", "s-o"),
            crate::convention::Convention::Linux => ("C-S-p", "C-o"),
        };
        // The switch-project picker is flat over the workspace's direct
        // children only, so there is no folder to descend into — Down moves
        // off `proj-a` onto `sibling` and Enter switches to it immediately,
        // one chord. The final chord opens Goto in the re-scoped session.
        let spec = format!("{switch_project} Down Down Enter {open_goto}");
        let keys = keyspec::parse_keys(&spec).unwrap();
        let out = dir.join("cap.png");
        capture_screenshot(
            out.clone(),
            None,
            CaptureOpts::default(),
            keys,
            crate::keymap::KeymapState::new_with_convention(convention),
            Some(dir.join("old-ws/proj-a")),
            None,
            dir.join("notes"),
            Config::empty(),
            false,
        )
        .unwrap_or_else(|e| panic!("[{convention:?}] capture succeeds: {e}"));
        let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v["project"]["root"].as_str().unwrap(),
            // Through the sidecar's own home redaction: the subject is WHICH
            // root the accept re-derived, not how an under-home path is spelled.
            crate::capture::redact::redact(&dir.join("old-ws/sibling").to_string_lossy()),
            "[{convention:?}] the sidecar's accepted root is the new project (item 183's half)"
        );
        let items: Vec<String> = v["overlay"]["items"]
            .as_array()
            .unwrap_or_else(|| panic!("[{convention:?}] goto overlay open"))
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(items.first().map(String::as_str), Some("target.md"));
        assert!(
            !items.iter().any(|item| item == "keep.md"),
            "[{convention:?}] Cmd-O after the folder switch must list the NEW root's \
             file, while its additional rows are typed folder destinations: {items:?}"
        );
    }
}

/// The unified Folders lens exposes known workspace children and terminates in
/// the platform chooser fallback. Ordinary replay cannot open an OS panel, but
/// it must report that exact typed seam rather than resurrecting a second
/// switch-project stage.
#[test]
fn keys_capture_folders_lens_ends_in_the_typed_platform_fallback() {
    let _fs = crate::testlock::serial();
    for convention in [
        crate::convention::Convention::Mac,
        crate::convention::Convention::Linux,
    ] {
        let dir = ScratchDir::new(std::env::temp_dir().join(format!(
            "awl-folder-fallback-{convention:?}-{}",
            std::process::id()
        )));
        std::fs::create_dir_all(dir.join("old-ws/proj-a")).unwrap();
        std::fs::create_dir_all(dir.join("old-ws/sibling/nested")).unwrap();
        std::fs::write(dir.join("old-ws/proj-a/keep.md"), "keep").unwrap();
        std::fs::write(dir.join("old-ws/sibling/nested/deep.md"), "deep").unwrap();
        let chord = match convention {
            crate::convention::Convention::Mac => "s-S-p",
            crate::convention::Convention::Linux => "C-S-p",
        };
        let open = folder_capture(&dir, convention, chord, Some(dir.join("old-ws/proj-a")));
        assert_eq!(open["overlay"]["mode"].as_str(), Some("goto"));
        let rows: Vec<&str> = open["overlay"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row.as_str().unwrap())
            .collect();
        assert_eq!(rows.last().copied(), Some("Choose another folder…"));
        assert!(!rows.iter().any(|row| row.contains("nested")));

        let fallback = folder_capture(
            &dir,
            convention,
            &format!("{chord} Down Down Down Enter"),
            Some(dir.join("old-ws/proj-a")),
        );
        assert_eq!(
            fallback["replay_skips"][0]["effect"].as_str(),
            Some("open_folder_chooser"),
            "[{convention:?}] the terminal row reaches the typed live-only chooser seam"
        );
    }
}

fn folder_capture(
    dir: &ScratchDir,
    convention: crate::convention::Convention,
    spec: &str,
    root: Option<std::path::PathBuf>,
) -> serde_json::Value {
    let out = dir.join(format!("folder-{}.png", spec.len()));
    capture_screenshot(
        out.clone(),
        None,
        CaptureOpts::default(),
        keyspec::parse_keys(spec).unwrap(),
        crate::keymap::KeymapState::new_with_convention(convention),
        root,
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .unwrap_or_else(|e| panic!("[{convention:?}] capture of {spec:?}: {e}"));
    serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap()).unwrap()
}
