use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn replay_keys_goto_open_file_closes_all_no_overlay() {
    let _tg = crate::testlock::serial();
    let mut buffer = Buffer::scratch();
    let corpus = vec!["doc-fixture.md".to_string()];
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("s-o RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    assert!(
        res.journey.card().is_none(),
        "opening a file closes the overlay to the buffer"
    );
    assert_eq!(
        res.accept,
        Some((
            crate::overlay::OverlayKind::Goto,
            "doc-fixture.md".to_string()
        )),
        "the file open still fired",
    );
}

#[test]
fn replay_keys_goto_hides_dotfiles_until_file_visibility_is_all() {
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let mut buffer = Buffer::scratch();
    let corpus = vec![
        ".gitignore".to_string(),
        ".env".to_string(),
        "doc-fixture.md".to_string(),
        "src/main.rs".to_string(),
    ];
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("s-o").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    let ov = res.journey.card().expect("goto overlay open");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Goto);
    assert!(!crate::file_visibility::all_on());
    let shown = ov.item_strings();
    assert!(
        !shown.iter().any(|s| s == ".gitignore"),
        "dotfile hidden by default: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s == ".env"),
        ".env stays visible: {shown:?}"
    );
    assert!(shown.iter().any(|s| s == "doc-fixture.md"));
    crate::file_visibility::set_all_on(true);
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-o").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    let ov = res.journey.card().expect("goto overlay open under All");
    assert!(
        crate::file_visibility::all_on(),
        "File visibility All reveals dotfiles"
    );
    assert!(
        ov.item_strings().iter().any(|s| s == ".gitignore"),
        "dotfile shown under All: {:?}",
        ov.item_strings()
    );
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn replay_keys_asset_cleaner_lists_only_the_orphans_from_the_scan() {
    use std::sync::Arc;
    let root = PathBuf::from("/proj");
    let mem = crate::fs::InMemoryFs::new()
        .with_file("/proj/doc.md", "text\n![a](assets/used.png)\n")
        .with_file("/proj/assets/used.png", "U")
        .with_file("/proj/assets/orphan.png", "OO");
    let corpus = vec![
        "assets/orphan.png".to_string(),
        "assets/used.png".to_string(),
        "doc.md".to_string(),
    ];
    crate::fs::with_fs(Arc::new(mem), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-p c l e a n RET").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &corpus,
            &root,
            None,
            &Config::empty(),
            None,
        );
        let ov = res
            .journey
            .card()
            .expect("asset cleaner open after the palette chain");
        assert_eq!(ov.kind, crate::overlay::OverlayKind::Assets);
        assert_eq!(ov.item_strings(), vec!["orphan.png"]);
        assert_eq!(ov.item_bindings(), vec!["2 B · assets"]);
        assert_eq!(ov.kind.as_str(), "assets");
    });
}

#[test]
fn replay_keys_project_hides_dotfolders_marks_git_tag() {
    use std::sync::Arc;
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let ws = PathBuf::from("/ws");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir("/ws/.claude")
        .with_dir("/ws/.git") // junk-filtered before the overlay ever sees it
        .with_dir("/ws/plain")
        .with_dir("/ws/repo")
        .with_dir("/ws/repo/.git"); // marks `repo` a git repo
    crate::fs::with_fs(Arc::new(mem), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-S-p").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &[],
            &ws,
            Some(ws.as_path()),
            &Config::empty(),
            None,
        );
        let ov = res.journey.card().expect("switch-project overlay open");
        assert_eq!(ov.kind, crate::overlay::OverlayKind::Project);
        assert!(!crate::file_visibility::all_on());
        let shown = ov.item_strings();
        assert!(
            shown.iter().any(|s| s == "."),
            "'.' accept row kept: {shown:?}"
        );
        assert!(
            !shown.iter().any(|s| s.starts_with(".claude")),
            "dotfolder hidden: {shown:?}"
        );
        assert!(
            !shown.iter().any(|s| s.starts_with(".git")),
            "junk .git hidden: {shown:?}"
        );
        assert!(
            shown.iter().any(|s| s.starts_with("plain")),
            "plain shown: {shown:?}"
        );
        assert!(
            shown.iter().any(|s| s.starts_with("repo")),
            "repo shown: {shown:?}"
        );
        assert!(
            shown.iter().all(|s| !s.contains('•')),
            "no name bullet: {shown:?}"
        );
        let tags = ov.item_git_tags();
        let ipos = |name: &str| shown.iter().position(|s| s.starts_with(name)).unwrap();
        assert_eq!(tags[ipos("repo")], "git", "repo is git-tagged");
        assert_eq!(tags[ipos("plain")], "", "plain folder has no tag");

        // File visibility All reveals the overlay-hidden dotfolder (`.claude`);
        // junk `.git` stays hidden (it never reaches the overlay corpus).
        crate::file_visibility::set_all_on(true);
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-S-p").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &[],
            &ws,
            Some(ws.as_path()),
            &Config::empty(),
            None,
        );
        let ov = res.journey.card().expect("project overlay open under All");
        assert!(
            crate::file_visibility::all_on(),
            "File visibility All reveals dotfolders"
        );
        let revealed = ov.item_strings();
        assert!(
            revealed.iter().any(|s| s.starts_with(".claude")),
            "revealed: {revealed:?}"
        );
        assert!(
            revealed.iter().any(|s| s == "."),
            "'.' still present after reveal"
        );
    });
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn replay_keys_drives_rebind_menu_capture() {
    // The GAME-STYLE REBIND MENU, driven entirely through the headless replay:
    // Cmd-P → "keyb" → Enter opens the Keybindings menu, "undo" filters to Undo,
    // Enter starts a capture (ChooseMode), Enter begins recording (KEY), and a
    // plain 'q' is captured → committed (the menu's NOTICE reflects the binding).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y b RET u n d o RET RET q").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .journey
        .card()
        .expect("the rebind menu stays open after a commit");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Keybindings);
    assert!(
        ov.capture.is_none(),
        "capture closed after committing the key"
    );
    assert_eq!(
        ov.notice, "bound undo -> q",
        "notice reflects the captured binding"
    );
}

#[test]
fn replay_keys_rebind_menu_recording_state_visible() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y b RET s a v e RET Down RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.journey.card().expect("menu open");
    let cap = ov.capture.clone().expect("a capture is in progress");
    assert_eq!(cap.cmd_name, "Save");
    assert_eq!(cap.stage, crate::overlay::CaptureStage::Recording);
    assert!(
        cap.chord_mode,
        "Down selected the CHORD row before recording"
    );
    assert!(cap.captured.is_empty(), "no combo pressed yet");
}

#[test]
fn replay_keys_settings_cjk_picker_round_trips_headlessly() {
    let _g = crate::testlock::serial();
    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("s-p s e t t i n g s RET a m b i g u o u s RET Down Down Down RET")
            .unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);

    assert_eq!(
        crate::frontmatter::cjk_priority(),
        vec![
            crate::frontmatter::Lang::Ko,
            crate::frontmatter::Lang::Ja,
            crate::frontmatter::Lang::ZhHans,
            crate::frontmatter::Lang::ZhHant,
        ],
    );

    let ov = res
        .journey
        .card()
        .expect("popped back to the Settings menu, not closed");
    assert_eq!(
        ov.kind,
        crate::overlay::OverlayKind::Settings,
        "back at Settings"
    );
    assert_eq!(
        res.journey.parked_kind(),
        None,
        "single-level: no N-deep stack"
    );
    let row_idx = crate::settings::SETTINGS
        .iter()
        .position(|r| r.name == "Ambiguous CJK reads as")
        .unwrap();
    assert_eq!(
        ov.rows[row_idx].secondary, "Korean",
        "the re-summoned Settings menu's value cell is FRESH, in writer-words"
    );

    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
}

/// THE SUSPEND/RETURN JOURNEY, DRIVEN ENTIRELY BY `--keys`. Open the
/// Settings workspace from the palette, filter DOWN THE LIST to a row that is
/// not row 0, descend into its child audition, and cancel: the workspace
/// resumes ON THAT ROW, with the filter that found it.
///
/// The pre-item-173 breadcrumb re-summoned the parent FRESH and dropped both,
/// so this replay used to land on "Caret style" with an empty query no matter
/// which row you came from. The action path and the replay path are the same
/// seam, so this is the parity witness as well as the restoration one.
#[test]
fn replay_keys_a_cancelled_settings_child_resumes_on_the_row_it_left() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p s e t t i n g s RET t h e m e RET Esc").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);

    let ov = res
        .journey
        .card()
        .expect("the workspace resumed rather than closing to the buffer");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Settings);
    assert_eq!(
        ov.selected_value(),
        Some("Theme"),
        "resumed on the row the child was opened from, not on row 0"
    );
    assert_eq!(
        ov.query.text(),
        "theme",
        "and with the filter that found it"
    );
    assert_eq!(res.journey.parked_kind(), None, "single-level");
    crate::theme::set_active(0);
}

/// The SAME journey stopped one chord earlier: while the child audition is up,
/// the SIDECAR reports which surface is parked beneath it. `overlay.return_to`
/// keeps its published name and now reads the lifecycle's parked parent, so an
/// agent probe can see the suspension without a new schema field.
#[test]
fn replay_keys_the_sidecar_reports_the_parked_workspace_under_a_child() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p s e t t i n g s RET t h e m e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.journey.card().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Theme),
        "the child audition is up"
    );
    let (info, _preview, _diff) =
        crate::run::overlay_capture_info(&res.journey, &buffer).expect("a card is up");
    assert_eq!(
        info.return_to,
        Some("settings"),
        "the sidecar names the parked workspace"
    );
    assert_eq!(info.mode, "theme");
    crate::theme::set_active(0);
}
