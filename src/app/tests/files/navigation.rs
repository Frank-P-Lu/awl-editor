//! Directory journeys through the live App's input-gathering and action seam.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use crate::overlay::{FilesFocus, OverlayState};

fn fixture() -> InMemoryFs {
    InMemoryFs::new()
        .with_file("/proj/alpha.md", "alpha stays untouched\n")
        .with_file("/proj/beta.md", "beta\n")
        .with_file("/proj/research/field-notes.md", "field notes\n")
        .with_file("/proj/research/deeper/detail.md", "detail\n")
        .with_file("/proj/unsupported/image.png", "image")
        .with_dir("/proj/empty-folder")
}

fn apply(app: &mut App, action: Action) {
    let exit = crate::app::schedule::RecordingExit::new();
    app.apply(action, false, &exit, crate::stats::Door::Chord);
    assert!(!exit.exit_requested());
}

fn card(app: &App) -> &OverlayState {
    app.workspace_state.overlay().expect("Files stays open")
}

fn visible(app: &App) -> Vec<&str> {
    let card = card(app);
    card.items
        .iter()
        .map(|&index| card.rows[index].accept.as_str())
        .collect()
}

fn choose(app: &mut App, path: &str, action: Action) {
    let card = app.workspace_state.overlay_mut().unwrap();
    card.selected = card
        .items
        .iter()
        .position(|&index| card.rows[index].accept == path)
        .unwrap_or_else(|| panic!("{path} must be visible"));
    card.files_focus = FilesFocus::Choices;
    apply(app, action);
}

fn ascend(app: &mut App, focus: FilesFocus, action: Action) {
    app.workspace_state.overlay_mut().unwrap().files_focus = focus;
    apply(app, action);
}

fn assert_root(app: &App) {
    assert_eq!(card(app).browse_dir, None);
    assert_eq!(card(app).files_location().as_deref(), Some("proj"));
    assert_eq!(card(app).notice, "");
    let rows = visible(app);
    for path in [
        "alpha.md",
        "beta.md",
        "research",
        "empty-folder",
        "unsupported",
    ] {
        assert!(rows.contains(&path), "root must show {path}: {rows:?}");
    }
    assert!(!rows.contains(&"research/field-notes.md"));
}

#[test]
fn every_live_folder_route_retains_files_across_repeated_descents_and_ascents() {
    let _serial = crate::testlock::serial();
    for default_root in [false, true] {
        for enter in [Action::Newline, Action::ForwardChar] {
            for (focus, up) in [
                (FilesFocus::Up, Action::Newline),
                (FilesFocus::Up, Action::BackwardChar),
                (FilesFocus::Choices, Action::BackwardChar),
            ] {
                let fs = CountingFs::new(fixture());
                let _fs = crate::fs::FsGuard::install(Arc::new(fs.clone()));
                let mut cfg = Config::empty();
                cfg.default_folder =
                    Some(PathBuf::from(if default_root { "/proj" } else { "/other" }));
                let mut app = app_on(Some(PathBuf::from("/proj/alpha.md")), "/proj", cfg);
                app.project_location.recent_files = vec![PathBuf::from("/proj/beta.md")];
                let text = app.document.buffer().text();
                fs.clear_reads();
                apply(&mut app, Action::OpenGoto);
                assert_root(&app);
                for _ in 0..2 {
                    choose(&mut app, "empty-folder", enter.clone());
                    assert_eq!(card(&app).browse_dir.as_deref(), Some("empty-folder"));
                    assert_eq!(card(&app).notice, "this folder is empty");
                    assert!(visible(&app).is_empty());
                    ascend(&mut app, focus, up.clone());
                    assert_root(&app);
                    choose(&mut app, "research", enter.clone());
                    assert_eq!(card(&app).notice, "");
                    assert!(visible(&app).contains(&"research/field-notes.md"));
                    choose(&mut app, "research/deeper", enter.clone());
                    assert_eq!(visible(&app), ["research/deeper/detail.md"]);
                    ascend(&mut app, focus, up.clone());
                    assert_eq!(card(&app).browse_dir.as_deref(), Some("research"));
                    assert!(visible(&app).contains(&"research/field-notes.md"));
                    ascend(&mut app, focus, up.clone());
                    assert_root(&app);
                }
                assert_eq!(app.document.buffer().text(), text);
                assert_eq!(app.project_location.root, PathBuf::from("/proj"));
                assert!(
                    fs.reads().is_empty(),
                    "navigation must not read file contents"
                );
            }
        }
    }
}

#[test]
fn navigated_files_keep_root_search_recent_and_active_identity() {
    let _serial = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(fixture()));
    let mut app = app_on(
        Some(PathBuf::from("/proj/alpha.md")),
        "/proj",
        Config::empty(),
    );
    app.project_location.recent_files = vec![
        PathBuf::from("/proj/beta.md"),
        PathBuf::from("/proj/research/field-notes.md"),
    ];
    apply(&mut app, Action::OpenGoto);
    choose(&mut app, "research", Action::ForwardChar);
    apply(&mut app, Action::PasteText("beta".into()));
    assert_eq!(visible(&app), ["beta.md"]);
    apply(&mut app, Action::BackwardChar);
    assert_eq!(card(&app).browse_dir.as_deref(), Some("research"));
    apply(&mut app, Action::SelectAll);
    apply(&mut app, Action::DeleteBackward);
    assert!(visible(&app).contains(&"research/field-notes.md"));
    ascend(&mut app, FilesFocus::Up, Action::Newline);
    assert_root(&app);
    let open: Vec<_> = card(&app)
        .open
        .iter()
        .map(|&i| card(&app).rows[i].accept.as_str())
        .collect();
    assert_eq!(open, ["alpha.md"]);
    app.workspace_state.overlay_mut().unwrap().files_focus = FilesFocus::Recent;
    apply(&mut app, Action::Newline);
    let recent = visible(&app);
    assert_eq!(recent.len(), 2);
    assert!(recent.contains(&"beta.md"));
    assert!(recent.contains(&"research/field-notes.md"));
    app.workspace_state.overlay_mut().unwrap().files_focus = FilesFocus::Files;
    apply(&mut app, Action::Newline);
    assert_root(&app);
}

#[test]
fn folder_outcomes_reset_and_reopening_refreshes_the_cached_roster() {
    let _serial = crate::testlock::serial();
    let mem = fixture();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(PathBuf::from("/proj/alpha.md")),
        "/proj",
        Config::empty(),
    );
    apply(&mut app, Action::OpenGoto);
    choose(&mut app, "unsupported", Action::Newline);
    assert_eq!(card(&app).notice, "no supported files in this folder");
    assert!(visible(&app).is_empty());
    ascend(&mut app, FilesFocus::Up, Action::Newline);
    assert_root(&app);
    mem.write(
        std::path::Path::new("/proj/new.md"),
        b"created outside awl\n",
    )
    .unwrap();
    choose(&mut app, "empty-folder", Action::Newline);
    ascend(&mut app, FilesFocus::Up, Action::Newline);
    assert!(
        !visible(&app).contains(&"new.md"),
        "navigation uses the summon-time index"
    );
    apply(&mut app, Action::Cancel);
    assert!(!app.workspace_state.overlay_open());
    apply(&mut app, Action::OpenGoto);
    assert!(
        visible(&app).contains(&"new.md"),
        "a new summon rescans the root"
    );
    choose(&mut app, "research", Action::Newline);
    choose(&mut app, "research/field-notes.md", Action::Newline);
    assert!(
        !app.workspace_state.overlay_open(),
        "only file acceptance dismisses Files"
    );
    assert_eq!(
        app.document.buffer().path(),
        Some(std::path::Path::new("/proj/research/field-notes.md"))
    );
    assert_eq!(
        mem.read_to_string(std::path::Path::new("/proj/alpha.md"))
            .unwrap(),
        "alpha stays untouched\n"
    );
}

const FILES_FOCI: [FilesFocus; 7] = [
    FilesFocus::Up,
    FilesFocus::Query,
    FilesFocus::Files,
    FilesFocus::Recent,
    FilesFocus::Choices,
    FilesFocus::ChangeFolder,
    FilesFocus::NewDocument,
];

fn row_position(app: &App, path: &str) -> usize {
    let card = card(app);
    card.items
        .iter()
        .position(|&i| card.rows[i].accept == path)
        .unwrap()
}

#[test]
fn pointer_row_activation_reclaims_focus_after_up_and_every_files_control() {
    let _serial = crate::testlock::serial();
    for focus in FILES_FOCI {
        let mem = fixture();
        let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
        let mut app = app_on(
            Some(PathBuf::from("/proj/alpha.md")),
            "/proj",
            Config::empty(),
        );
        let exit = crate::app::schedule::RecordingExit::new();
        apply(&mut app, Action::OpenGoto);
        choose(&mut app, "empty-folder", Action::Newline);
        ascend(&mut app, FilesFocus::Up, Action::Newline);
        assert_root(&app);
        for _ in 0..2 {
            app.workspace_state.overlay_mut().unwrap().files_focus = focus;
            app.activate_overlay_row(row_position(&app, "research"), &exit);
            assert_eq!(
                card(&app).browse_dir.as_deref(),
                Some("research"),
                "prior focus {focus:?}"
            );
            assert_eq!(card(&app).files_focus, FilesFocus::Choices);
            assert!(visible(&app).contains(&"research/field-notes.md"));
            ascend(&mut app, FilesFocus::Up, Action::Newline);
            assert_root(&app);
        }
        app.workspace_state.overlay_mut().unwrap().files_focus = focus;
        app.activate_overlay_row(row_position(&app, "beta.md"), &exit);
        assert!(
            !app.workspace_state.overlay_open(),
            "file click accepts, prior focus {focus:?}"
        );
        assert_eq!(
            app.document.buffer().path(),
            Some(std::path::Path::new("/proj/beta.md"))
        );
        assert_eq!(
            mem.read_to_string(std::path::Path::new("/proj/alpha.md"))
                .unwrap(),
            "alpha stays untouched\n"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn semantic_row_targeting_sets_exact_selection_and_choices_focus_before_acceptance() {
    let _serial = crate::testlock::serial();
    for focus in FILES_FOCI {
        for already_selected in [false, true] {
            let _fs = crate::fs::FsGuard::install(Arc::new(fixture()));
            let mut app = app_on(
                Some(PathBuf::from("/proj/alpha.md")),
                "/proj",
                Config::empty(),
            );
            apply(&mut app, Action::OpenGoto);
            let target = row_position(&app, "research");
            let overlay = app.workspace_state.overlay_mut().unwrap();
            overlay.files_focus = focus;
            overlay.selected = if already_selected {
                target
            } else {
                (target + 1) % overlay.items.len()
            };
            let id = format!("overlay.goto.row.{}", overlay.items[target]);
            assert!(
                app.apply_semantic_request(crate::semantic::SemanticRequest::Focus {
                    id: id.clone()
                })
            );
            assert_eq!(card(&app).selected, target, "prior focus {focus:?}");
            assert_eq!(
                card(&app).files_focus,
                FilesFocus::Choices,
                "prior focus {focus:?}"
            );
            assert_eq!(card(&app).browse_dir, None, "focus alone does not accept");
            let overlay = app.workspace_state.overlay_mut().unwrap();
            overlay.files_focus = focus;
            overlay.selected = if already_selected {
                target
            } else {
                (target + 1) % overlay.items.len()
            };
            assert!(app.apply_semantic_request(crate::semantic::SemanticRequest::Click { id }));
            assert_eq!(card(&app).browse_dir.as_deref(), Some("research"));
            assert!(visible(&app).contains(&"research/field-notes.md"));
        }
    }
}

#[test]
fn stale_pointer_row_index_does_not_dispatch_a_files_control_action() {
    let _serial = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(fixture()));
    let mut app = app_on(
        Some(PathBuf::from("/proj/alpha.md")),
        "/proj",
        Config::empty(),
    );
    apply(&mut app, Action::OpenGoto);
    app.workspace_state.overlay_mut().unwrap().files_focus = FilesFocus::NewDocument;
    let exit = crate::app::schedule::RecordingExit::new();
    app.activate_overlay_row(card(&app).items.len(), &exit);
    assert_root(&app);
    assert_eq!(
        app.document.buffer().path(),
        Some(std::path::Path::new("/proj/alpha.md"))
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn semantic_files_focus_reveals_a_deep_row_and_retains_its_scroll_window() {
    let _serial = crate::testlock::serial();
    let mem = InMemoryFs::new();
    let window = crate::overlay::OverlayKind::Goto.window_rows();
    for index in 0..window * 3 {
        mem.write(
            &PathBuf::from(format!("/proj/note-{index:03}.md")),
            b"text\n",
        )
        .unwrap();
    }
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let mut app = app_on(None, "/proj", Config::empty());
    apply(&mut app, Action::OpenGoto);
    let deep = window * 2;
    for (target, expected_scroll) in [
        (deep, deep + 1 - window),
        (deep - 1, deep + 1 - window),
        (0, 0),
    ] {
        app.workspace_state.overlay_mut().unwrap().files_focus = FilesFocus::Recent;
        let id = format!("overlay.goto.row.{}", card(&app).items[target]);
        assert!(app.apply_semantic_request(crate::semantic::SemanticRequest::Focus { id }));
        assert_eq!(card(&app).selected, target);
        assert_eq!(card(&app).files_focus, FilesFocus::Choices);
        assert_eq!(card(&app).scroll, expected_scroll, "target {target}");
        assert_eq!(card(&app).browse_dir, None);
    }
}
