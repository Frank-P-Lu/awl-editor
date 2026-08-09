use super::super::*;
use super::keyspec;
use crate::testscratch::ScratchDir;

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
            "awl-item189-switch-goto-{convention:?}-{}",
            std::process::id()
        )));
        std::fs::create_dir_all(dir.join("old-ws/proj-a")).unwrap();
        std::fs::create_dir_all(dir.join("old-ws/sibling")).unwrap();
        std::fs::create_dir_all(dir.join("new-ws/other")).unwrap();
        std::fs::create_dir_all(dir.join("new-ws/proj-b")).unwrap();
        std::fs::write(dir.join("old-ws/proj-a/keep.md"), "keep").unwrap();
        std::fs::write(dir.join("new-ws/proj-b/target.md"), "target").unwrap();

        let (switch_project, open_goto) = match convention {
            crate::convention::Convention::Mac => ("s-S-p", "s-o"),
            crate::convention::Convention::Linux => ("C-S-p", "C-o"),
        };
        // Backspace ascends above the old workspace to `dir`; Enter descends
        // into `new-ws`; Down moves off `other` onto `proj-b`; Enter descends
        // into it; the next Enter accepts the drilled-in directory as the new
        // root — the exact navigation the model test drives. The final
        // chord opens Goto in the (now, or not yet, re-scoped) session.
        let spec = format!("{switch_project} Backspace Enter Down Enter Enter {open_goto}");
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
            dir.join("new-ws/proj-b").to_string_lossy(),
            "[{convention:?}] the sidecar's accepted root is the new project (item 183's half)"
        );
        let items: Vec<String> = v["overlay"]["items"]
            .as_array()
            .unwrap_or_else(|| panic!("[{convention:?}] goto overlay open"))
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            items,
            vec!["target.md".to_string()],
            "[{convention:?}] Cmd-O after Switch-project must list the NEW root's \
             files, not the launch root's — a `keep.md` row here is item 189's \
             stale-corpus bug"
        );
    }
}
