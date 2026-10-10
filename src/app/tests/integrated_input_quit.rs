//! Typed manuscript edits and formatting retain their history across refused exits.
use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use winit::keyboard::{Key, ModifiersState, SmolStr};

const SOURCE: &str = "- parent\n  - abc child\n\nanchor\n";
const COMPOSED: &str = "- parent\n  - 日本語 child\n\nanchor\n";
const TYPED: &str = "- parent\n  - 日本語e\u{301}👩‍💻 child\n\nanchor\n";
const FENCED: &str = "- parent\n  - ```\n    日本語e\u{301}👩‍💻 child\n    ```\n\nanchor\n";

fn type_key(app: &mut App, text: &str) {
    app.press_chord_headless(
        &crate::keyspec::Chord {
            spec: text.into(),
            key: Key::Character(SmolStr::new(text)),
            mods: ModifiersState::empty().into(),
        },
        &schedule::RecordingExit::default(),
    );
}

fn apply(app: &mut App, action: Action) {
    app.apply(
        action,
        false,
        &schedule::RecordingExit::default(),
        crate::stats::Door::Menu,
    );
}

fn request(app: &mut App, window: bool) -> bool {
    let exit = schedule::RecordingExit::new();
    let accepted = if window {
        app.request_document_exit(&exit)
    } else {
        app.apply(Action::Quit, false, &exit, crate::stats::Door::Menu)
    };
    assert_eq!(accepted, exit.exit_requested());
    accepted
}

fn text_snapshot(app: &App) -> Vec<(crate::buffers::BufferKey, String)> {
    let active = app.document.active_key();
    app.document
        .open_entry_keys()
        .into_iter()
        .map(|key| {
            let text = if active.as_ref() == Some(&key) {
                app.document.buffer().text()
            } else {
                app.document.parked_text(&key).unwrap()
            };
            (key, text)
        })
        .collect()
}

fn assert_refused_exits_preserve(
    app: &mut App,
    mem: &InMemoryFs,
    a: &std::path::Path,
    b: &std::path::Path,
) {
    let _guard = crate::testlock::serial();
    let key = app.document.active_key();
    let before = text_snapshot(app);
    let cursor = app.document.buffer().cursor_char();
    let selection = app.document.buffer().selection_range();
    for _ in 0..3 {
        for window in [false, true] {
            assert!(!request(app, window));
            assert_eq!(text_snapshot(app), before);
            assert_eq!(app.document.active_key(), key);
            assert_eq!(app.document.buffer().cursor_char(), cursor);
            assert_eq!(app.document.buffer().selection_range(), selection);
            assert_eq!(mem.read(a).unwrap(), SOURCE.as_bytes());
            assert_eq!(mem.read(b).unwrap(), b"other\n");
        }
    }
}

fn assert_editing_history_roundtrip(app: &mut App, caret: usize, emitted_cursor: usize) {
    let _guard = crate::testlock::serial();
    for expected in [TYPED, COMPOSED, SOURCE] {
        apply(app, Action::Undo);
        assert_eq!(app.document.buffer().text(), expected);
        assert_eq!(app.document.buffer().selection_range(), None);
    }
    assert_eq!(app.document.buffer().cursor_char(), caret);
    assert!(!app.document.buffer().can_undo());
    for expected in [COMPOSED, TYPED, FENCED] {
        apply(app, Action::Redo);
        assert_eq!(app.document.buffer().text(), expected);
    }
    assert_eq!(app.document.buffer().cursor_char(), emitted_cursor);
}

fn input_format_exit_case(ime: bool, reverse: bool, parked: bool) {
    let _guard = crate::testlock::serial();
    let a = PathBuf::from("/integrated/draft.md");
    let b = PathBuf::from("/integrated/other.md");
    let mem = InMemoryFs::new()
        .with_file(&a, SOURCE)
        .with_file(&b, "other\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let mut app = app_on(
        Some(a.clone()),
        "/integrated",
        Config {
            autosave: Some(false),
            session_restore: Some(false),
            ..Config::empty()
        },
    );
    let start = app.document.buffer().line_col_to_char(1, 4);
    let end = start + 3;
    let (anchor, caret) = if reverse { (end, start) } else { (start, end) };
    app.document.select_range(anchor, caret);
    if ime {
        app.on_ime(Ime::Preedit("にほん".into(), Some((3, 3))));
        assert_eq!(app.document.buffer().text(), SOURCE);
        app.on_ime(Ime::Commit("日本語".into()));
    } else {
        type_key(&mut app, "日本語");
    }
    assert_eq!(app.document.buffer().text(), COMPOSED);
    type_key(&mut app, "e\u{301}👩‍💻");
    assert_eq!(app.document.buffer().text(), TYPED);
    apply(&mut app, Action::ToggleCodeBlock);
    assert_eq!(app.document.buffer().text(), FENCED);
    let emitted_cursor = app.document.buffer().cursor_char();
    let emitted_selection = app.document.buffer().selection_range();
    assert!(emitted_selection.is_some());
    if parked {
        assert!(app.load_path(b.clone()));
    }
    assert_refused_exits_preserve(&mut app, &mem, &a, &b);
    if parked {
        assert!(app.load_path(a.clone()));
    }
    assert_eq!(app.document.buffer().text(), FENCED);
    assert_eq!(app.document.buffer().cursor_char(), emitted_cursor);
    assert_eq!(app.document.buffer().selection_range(), emitted_selection);
    assert_editing_history_roundtrip(&mut app, caret, emitted_cursor);
    app.manual_save();
    assert_eq!(mem.read(&a).unwrap(), FENCED.as_bytes());
    assert!(request(&mut app, false));
    assert!(request(&mut app, true));
}

#[test]
fn complete_input_then_nested_code_survives_refused_exit_and_parked_undo_redo() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    for ime in [false, true] {
        for reverse in [false, true] {
            for parked in [false, true] {
                input_format_exit_case(ime, reverse, parked);
            }
        }
    }
}

fn assert_original_unsaved(app: &App) {
    let _guard = crate::testlock::serial();
    assert!(app.is_document_dirty());
    let active = app.document.active_key().unwrap();
    assert!(app.document.close_facts(&active).unwrap().unsaved);
}

#[test]
fn selected_input_refreshes_conflict_recovery_without_acknowledging_original_save() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    for ime in [false, true] {
        for window in [false, true] {
            let path = PathBuf::from("/integrated/conflict.md");
            let mem = InMemoryFs::new().with_file(&path, SOURCE);
            let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
            let mut app = app_on(
                Some(path.clone()),
                "/integrated",
                Config {
                    autosave: Some(false),
                    session_restore: Some(false),
                    ..Config::empty()
                },
            );
            app.document.set_text(COMPOSED);
            mem.write(&path, b"external editor\n").unwrap();
            app.manual_save();
            assert!(app.change_unresolved());
            assert_eq!(crate::recovery::read_for(&path).unwrap().text, COMPOSED);
            let start = app.document.buffer().line_col_to_char(1, 4);
            app.document.select_range(start, start + 3);
            if ime {
                app.on_ime(Ime::Commit("最新e\u{301}👩‍💻".into()));
            } else {
                type_key(&mut app, "最新e\u{301}👩‍💻");
            }
            let latest = app.document.buffer().text();
            assert_eq!(latest, "- parent\n  - 最新e\u{301}👩‍💻 child\n\nanchor\n");
            assert!(request(&mut app, window));
            assert_original_unsaved(&app);
            assert_eq!(crate::recovery::read_for(&path).unwrap().text, latest);
            assert_eq!(mem.read(&path).unwrap(), b"external editor\n");
            assert!(app.document.buffer().is_dirty());
            apply(&mut app, Action::Undo);
            assert_eq!(app.document.buffer().text(), COMPOSED);
            apply(&mut app, Action::Redo);
            assert_eq!(app.document.buffer().text(), latest);
            assert!(request(&mut app, window));
            assert_original_unsaved(&app);
            assert_eq!(crate::recovery::read_for(&path).unwrap().text, latest);
        }
    }
}
