//! Selected document compositions replace text through one undoable edit.

use super::summoned_field_actions::doc_state;
use super::*;
use std::sync::Arc;

pub(super) const DOC: &str = "前abc後\n";

pub(super) fn fixture() -> App {
    app_on(
        Some(PathBuf::from("/proj/draft.md")),
        "/proj",
        Config {
            autosave: Some(false),
            session_restore: Some(false),
            reduce_motion: Some(true),
            ambient_motion: Some(false),
            ..Config::empty()
        },
    )
}

pub(super) fn fire(app: &mut App, action: Action) {
    app.apply(
        action,
        false,
        &schedule::RecordingExit::default(),
        crate::stats::Door::Menu,
    );
}

#[test]
fn selected_ime_commit_preserves_raw_text_and_one_undo_redo_in_both_directions() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    for reverse in [false, true] {
        for text in ["x", "日本語", "e\u{301}", "👩‍💻", " ", "\n\t\r", "a b\n\t\rc"] {
            let mut app = fixture();
            let (anchor, caret) = if reverse { (4, 1) } else { (1, 4) };
            app.document.select_range(anchor, caret);
            let before = doc_state(&app);
            app.on_ime(Ime::Enabled);
            app.on_ime(Ime::Preedit("にほん".into(), None));
            assert_eq!(doc_state(&app), before, "preedit is transient");
            app.on_ime(Ime::Preedit(String::new(), None));
            app.on_ime(Ime::Commit(text.into()));
            let expected = format!("前{text}後\n");
            assert_eq!(app.document.buffer().text(), expected, "{text:?}");
            assert_eq!(app.document.buffer().version(), before.1 + 1);
            assert_eq!(
                app.document.buffer().cursor_char(),
                1 + text.chars().count()
            );
            assert_eq!(app.document.buffer().selection_range(), None);
            fire(&mut app, Action::Undo);
            assert_eq!(app.document.buffer().text(), DOC, "one Undo: {text:?}");
            assert_eq!(app.document.buffer().cursor_char(), caret);
            assert!(!app.document.buffer().can_undo());
            fire(&mut app, Action::Redo);
            assert_eq!(app.document.buffer().text(), expected, "one Redo: {text:?}");
            assert_eq!(
                app.document.buffer().cursor_char(),
                1 + text.chars().count()
            );
        }
    }
}

#[test]
fn empty_ime_commit_preserves_selection_and_open_typing_group() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    for reverse in [false, true] {
        let mut app = fixture();
        app.document
            .select_range(if reverse { 4 } else { 1 }, if reverse { 1 } else { 4 });
        let before = doc_state(&app);
        app.on_ime(Ime::Preedit("候補".into(), None));
        app.on_ime(Ime::Commit(String::new()));
        assert_eq!(doc_state(&app), before);
        assert!(!app.document.buffer().can_undo());
    }
    let mut app = fixture();
    fire(&mut app, Action::InsertChar('x'));
    app.on_ime(Ime::Commit(String::new()));
    fire(&mut app, Action::InsertChar('y'));
    fire(&mut app, Action::Undo);
    assert_eq!(
        app.document.buffer().text(),
        DOC,
        "empty commit must not seal typing"
    );
}

#[test]
fn selected_ime_commit_is_separate_from_typing_on_both_sides() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    for reverse in [false, true] {
        let mut app = fixture();
        app.document.set_cursor(1);
        fire(&mut app, Action::InsertChar('x'));
        app.document
            .select_range(if reverse { 5 } else { 2 }, if reverse { 2 } else { 5 });
        app.on_ime(Ime::Commit("日本".into()));
        fire(&mut app, Action::InsertChar('z'));
        for expected in ["前x日本後\n", "前xabc後\n", DOC] {
            fire(&mut app, Action::Undo);
            assert_eq!(app.document.buffer().text(), expected);
        }
        for expected in ["前xabc後\n", "前x日本後\n", "前x日本z後\n"] {
            fire(&mut app, Action::Redo);
            assert_eq!(app.document.buffer().text(), expected);
        }
    }
}

#[test]
fn unselected_ime_commit_keeps_typing_coalescing_and_whitespace_boundaries() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    let mut app = fixture();
    fire(&mut app, Action::InsertChar('x'));
    app.on_ime(Ime::Commit("日本".into()));
    fire(&mut app, Action::Undo);
    assert_eq!(
        app.document.buffer().text(),
        DOC,
        "unselected text still joins typing"
    );
    fire(&mut app, Action::Redo);
    assert_eq!(app.document.buffer().text(), format!("x日本{DOC}"));
    let mut app = fixture();
    app.on_ime(Ime::Commit("a b\n\tc".into()));
    for prefix in ["a b\n\t", "a b\n", "a ", ""] {
        fire(&mut app, Action::Undo);
        assert_eq!(app.document.buffer().text(), format!("{prefix}{DOC}"));
    }
}

#[test]
fn selected_ime_commit_still_obeys_read_only_and_missing_document_doors() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    let mut app = fixture();
    app.document.select_range(1, 4);
    app.on_ime(Ime::Preedit("候補".into(), None));
    app.workspace_state
        .install_overlay_for_test(crate::overlay::OverlayState::new_credits());
    let before = doc_state(&app);
    app.on_ime(Ime::Commit("日本語".into()));
    assert_eq!(doc_state(&app), before);
    assert!(!app.document.buffer().can_undo());
    app.workspace_state.dismiss_pickers();
    let key = app.document.active_key().unwrap();
    let _outgoing = app.document.unseat_active(&key, None).unwrap();
    assert!(!app.document.has_active());
    app.on_ime(Ime::Commit("日本語".into()));
    assert!(!app.document.has_active());
}

#[test]
fn input_fixture_restores_all_startup_toggles() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ));
    let before = crate::testlock::misc::pins();
    for reduced in [false, true] {
        for ambient in [false, true] {
            {
                let _toggles = crate::testlock::misc::TogglesRestore::capture();
                let _app = app_on(
                    Some(PathBuf::from("/proj/draft.md")),
                    "/proj",
                    Config {
                        autosave: Some(false),
                        session_restore: Some(false),
                        reduce_motion: Some(reduced),
                        ambient_motion: Some(ambient),
                        scroll_sensitivity: Some(1.5),
                        ..Config::empty()
                    },
                );
                assert_eq!(crate::motion::reduced(), reduced);
                assert_eq!(crate::warpgrid::ambient_motion_on(), ambient);
            }
            assert_eq!(crate::testlock::misc::pins(), before);
        }
    }
}
