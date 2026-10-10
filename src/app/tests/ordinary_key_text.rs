//! Complete ordinary text keys use the same admitted action and field owners.

use super::ime_selected_commit::{DOC, fire, fixture};
use super::summoned_field_actions::{doc_state, field_text, summon};
use super::*;
use std::sync::Arc;
use winit::keyboard::{Key, ModifiersState, SmolStr};

fn press(app: &mut App, text: &str, mods: ModifiersState) {
    app.press_chord_headless(
        &crate::keyspec::Chord {
            spec: text.into(),
            key: Key::Character(SmolStr::new(text)),
            mods: mods.into(),
        },
        &schedule::RecordingExit::default(),
    );
}

fn fs_fixture() -> crate::fs::FsGuard {
    crate::fs::FsGuard::install(Arc::new(
        crate::fs::InMemoryFs::new().with_file(PathBuf::from("/proj/draft.md"), DOC),
    ))
}

#[test]
fn ordinary_key_preserves_complete_selected_text_and_one_undo_redo() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for reverse in [false, true] {
        for text in ["x", "日本語", "e\u{301}", "👩‍💻", "a b\n\t\rc"] {
            let mut app = fixture();
            let (anchor, caret) = if reverse { (4, 1) } else { (1, 4) };
            app.document.select_range(anchor, caret);
            let version = app.document.buffer().version();
            let eol = app.document.buffer().eol();
            press(&mut app, text, ModifiersState::empty());
            let expected = format!("前{text}後\n");
            assert_eq!(app.document.buffer().text(), expected);
            assert_eq!(app.document.buffer().version(), version + 1);
            assert_eq!(
                app.document.buffer().cursor_char(),
                1 + text.chars().count()
            );
            assert_eq!(app.document.buffer().selection_range(), None);
            assert_eq!(
                app.document.buffer().eol(),
                eol,
                "typing changes no save policy"
            );
            fire(&mut app, Action::Undo);
            assert_eq!(app.document.buffer().text(), DOC);
            assert_eq!(app.document.buffer().cursor_char(), caret);
            assert!(!app.document.buffer().can_undo());
            fire(&mut app, Action::Redo);
            assert_eq!(app.document.buffer().text(), expected);
        }
    }
}

#[test]
fn selected_ime_then_multiscalar_key_has_two_complete_undo_redo_steps() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for reverse in [false, true] {
        let mut app = fixture();
        app.document
            .select_range(if reverse { 4 } else { 1 }, if reverse { 1 } else { 4 });
        let version = app.document.buffer().version();
        app.on_ime(Ime::Preedit("にほんご".into(), None));
        app.on_ime(Ime::Commit("日本語".into()));
        assert_eq!(app.document.buffer().text(), "前日本語後\n");
        assert_eq!(app.document.buffer().cursor_char(), 4);
        assert_eq!(app.document.buffer().version(), version + 1);
        press(&mut app, "e\u{301}👩‍💻", ModifiersState::empty());
        assert_eq!(app.document.buffer().text(), "前日本語e\u{301}👩‍💻後\n");
        assert_eq!(app.document.buffer().cursor_char(), 9);
        // An unselected ordinary run retains one content revision per scalar.
        assert_eq!(app.document.buffer().version(), version + 6);
        for (expected, caret) in [("前日本語後\n", 4), (DOC, if reverse { 1 } else { 4 })] {
            fire(&mut app, Action::Undo);
            assert_eq!(app.document.buffer().text(), expected);
            assert_eq!(app.document.buffer().cursor_char(), caret);
            assert_eq!(app.document.buffer().selection_range(), None);
        }
        assert!(!app.document.buffer().can_undo());
        for expected in ["前日本語後\n", "前日本語e\u{301}👩‍💻後\n"] {
            fire(&mut app, Action::Redo);
            assert_eq!(app.document.buffer().text(), expected);
        }
    }
}

#[test]
fn unselected_ime_and_ordinary_key_retain_distinct_whitespace_typing_groups() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    let mut app = fixture();
    app.on_ime(Ime::Commit("a b".into()));
    press(&mut app, "c d", ModifiersState::empty());
    assert_eq!(app.document.buffer().text(), format!("a bc d{DOC}"));
    for prefix in ["a bc ", "a ", ""] {
        fire(&mut app, Action::Undo);
        assert_eq!(app.document.buffer().text(), format!("{prefix}{DOC}"));
    }
    for prefix in ["a ", "a bc ", "a bc d"] {
        fire(&mut app, Action::Redo);
        assert_eq!(app.document.buffer().text(), format!("{prefix}{DOC}"));
    }
}

#[test]
fn ordinary_text_keys_reach_every_existing_focused_field_without_editing_document() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for field in crate::textbox::TextField::ALL {
        let mut app = fixture();
        summon(&mut app, field);
        super::ime_fields::select_field(&mut app, field);
        let before = doc_state(&app);
        let text = if field == crate::textbox::TextField::SettingsValue {
            "125"
        } else {
            "日本語e\u{301}👩‍💻"
        };
        press(&mut app, text, ModifiersState::empty());
        assert_eq!(field_text(&app, field).as_deref(), Some(text), "{field:?}");
        assert_eq!(doc_state(&app), before, "{field:?}: background document");
    }
}

#[test]
fn multiscalar_query_recomputes_once_and_replacement_never_moves_match() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    let mut app = fixture();
    summon(&mut app, crate::textbox::TextField::FindQuery);
    super::ime_fields::select_field(&mut app, crate::textbox::TextField::FindQuery);
    let version = app.document.buffer().version();
    let _ = crate::textbox::work::take();
    press(&mut app, "abc", ModifiersState::empty());
    assert_eq!(app.document.buffer().cursor_char(), 1);
    assert_eq!(app.document.buffer().version(), version);
    let counts = crate::textbox::work::take();
    assert_eq!(counts[0], 1, "one field splice");
    assert_eq!(counts[2], 1, "one query recomputation");
    summon(&mut app, crate::textbox::TextField::ReplaceText);
    super::ime_fields::select_field(&mut app, crate::textbox::TextField::ReplaceText);
    let before = doc_state(&app);
    let _ = crate::textbox::work::take();
    press(&mut app, "日本語", ModifiersState::empty());
    assert_eq!(doc_state(&app), before);
    assert_eq!(
        crate::textbox::work::take()[2],
        0,
        "replacement is not searched"
    );
}

#[test]
fn selected_combined_events_remain_blocked_by_read_only_focus() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for reverse in [false, true] {
        let mut app = fixture();
        app.document
            .select_range(if reverse { 4 } else { 1 }, if reverse { 1 } else { 4 });
        app.on_ime(Ime::Preedit("候補".into(), None));
        app.workspace_state
            .install_overlay_for_test(crate::overlay::OverlayState::new_credits());
        let before = doc_state(&app);
        app.on_ime(Ime::Commit("日本語".into()));
        press(&mut app, "e\u{301}👩‍💻", ModifiersState::empty());
        assert_eq!(doc_state(&app), before);
        app.workspace_state.dismiss_pickers();
        fire(&mut app, Action::Undo);
        assert_eq!(app.document.buffer().text(), DOC);
        assert!(
            !app.document.buffer().can_redo(),
            "refused events create no history"
        );
    }
}

#[test]
fn ordinary_empty_and_modifier_refused_keys_do_not_replace_selection() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for text in ["", "\n日本", "\t日本", "\r日本"] {
        let mut app = fixture();
        app.document.select_range(1, 4);
        let before = doc_state(&app);
        press(&mut app, text, ModifiersState::empty());
        assert_eq!(doc_state(&app), before, "{text:?}");
    }
    for mods in [
        ModifiersState::CONTROL,
        ModifiersState::SUPER,
        ModifiersState::CONTROL | ModifiersState::SUPER,
    ] {
        let mut app = fixture();
        app.document.select_range(1, 4);
        let before = doc_state(&app);
        press(&mut app, "日本語", mods);
        assert_eq!(doc_state(&app), before, "{mods:?}");
    }
}

#[test]
fn multiscalar_key_resolution_preserves_control_and_prefix_guards() {
    let _guard = crate::testlock::serial();
    for convention in [
        crate::convention::Convention::Mac,
        crate::convention::Convention::Linux,
    ] {
        let mut keymap = crate::keymap::KeymapState::new_with_convention(convention);
        keymap.apply_linux_keep(&["C-x".into()]);
        let key = |text: &str| Key::Character(SmolStr::new(text));
        assert_eq!(
            keymap.resolve(&key("日本語"), &ModifiersState::empty().into()),
            Action::InsertText("日本語".into())
        );
        assert_eq!(
            keymap.resolve(&key("x"), &ModifiersState::empty().into()),
            Action::InsertChar('x')
        );
        assert_eq!(
            keymap.resolve(&key("k日本"), &ModifiersState::CONTROL.into()),
            Action::KillLine
        );
        assert_eq!(
            keymap.resolve(&key("h日本"), &ModifiersState::SUPER.into()),
            Action::Ignore
        );
        assert_eq!(
            keymap.resolve(&key("x"), &ModifiersState::CONTROL.into()),
            Action::BeginPrefix
        );
        assert_eq!(
            keymap.resolve(&key("日本語"), &ModifiersState::empty().into()),
            Action::Cancel
        );
        assert!(!keymap.in_prefix());
        assert_eq!(
            keymap.resolve(&key("日本語"), &ModifiersState::empty().into()),
            Action::InsertText("日本語".into())
        );
    }
}

#[test]
fn ordinary_text_moves_every_files_control_to_the_existing_query_owner() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    use crate::overlay::FilesFocus;
    for focus in [
        FilesFocus::Query,
        FilesFocus::Files,
        FilesFocus::Recent,
        FilesFocus::Up,
        FilesFocus::Choices,
        FilesFocus::ChangeFolder,
        FilesFocus::NewDocument,
    ] {
        let mut app = fixture();
        app.press_spec_headless(super::ime_fields::files_chord())
            .unwrap();
        app.workspace_state.overlay_mut().unwrap().files_focus = focus;
        let before = doc_state(&app);
        press(&mut app, "日本語", ModifiersState::empty());
        let card = app.workspace_state.overlay().unwrap();
        assert_eq!(card.files_focus, FilesFocus::Query);
        assert_eq!(card.query.text(), "日本語");
        assert_eq!(doc_state(&app), before);
    }
}

#[test]
fn mixed_unselected_events_keep_coalescing_across_text_payloads() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    let mut app = fixture();
    press(&mut app, "x", ModifiersState::empty());
    app.on_ime(Ime::Commit("日本".into()));
    press(&mut app, "e\u{301}", ModifiersState::empty());
    app.on_ime(Ime::Commit("a b".into()));
    press(&mut app, "y", ModifiersState::empty());
    assert_eq!(
        app.document.buffer().text(),
        format!("x日本e\u{301}a by{DOC}")
    );
    assert_eq!(app.document.buffer().cursor_char(), 9);
    for (prefix, caret) in [("x日本e\u{301}a ", 7), ("", 0)] {
        fire(&mut app, Action::Undo);
        assert_eq!(app.document.buffer().text(), format!("{prefix}{DOC}"));
        assert_eq!(app.document.buffer().cursor_char(), caret);
    }
    assert!(!app.document.buffer().can_undo());
    for (prefix, caret) in [("x日本e\u{301}a ", 7), ("x日本e\u{301}a by", 9)] {
        fire(&mut app, Action::Redo);
        assert_eq!(app.document.buffer().text(), format!("{prefix}{DOC}"));
        assert_eq!(app.document.buffer().cursor_char(), caret);
    }
}

#[test]
fn complete_keys_leave_config_bindings_and_alt_find_guards_in_charge() {
    let _guard = crate::testlock::serial();
    let _toggles = crate::testlock::misc::TogglesRestore::capture();
    let _fs = fs_fixture();
    for convention in [
        crate::convention::Convention::Mac,
        crate::convention::Convention::Linux,
    ] {
        let mut keymap = crate::keymap::KeymapState::new_with_convention(convention);
        keymap.apply_overrides(&[("quit".into(), vec!["日".into()])]);
        assert_eq!(
            keymap.resolve(
                &Key::Character(SmolStr::new("日")),
                &ModifiersState::empty().into()
            ),
            Action::Quit
        );
        assert_eq!(
            keymap.resolve(
                &Key::Character(SmolStr::new("日本語")),
                &ModifiersState::empty().into()
            ),
            Action::InsertText("日本語".into()),
            "full-key lookup does not reinterpret the first scalar as a binding"
        );
    }
    let mut app = fixture();
    summon(&mut app, crate::textbox::TextField::FindQuery);
    super::ime_fields::select_field(&mut app, crate::textbox::TextField::FindQuery);
    let before = field_text(&app, crate::textbox::TextField::FindQuery);
    press(&mut app, "日本語", ModifiersState::ALT);
    assert_eq!(
        field_text(&app, crate::textbox::TextField::FindQuery),
        before
    );
}
