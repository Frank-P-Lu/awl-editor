//! Platform compositions belong to the focused field, never its background document.

use super::summoned_field_actions::{app, doc_state, field_text, seeded, summon};
use super::*;
use crate::textbox::TextField;
use std::sync::Arc;

pub(super) fn files_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "Cmd-o",
        crate::convention::Convention::Linux => "C-o",
    }
}

#[test]
fn ime_files_query_filters_one_japanese_commit_and_preserves_document_and_source() {
    let _g = crate::testlock::serial();
    let fs = seeded().with_file(PathBuf::from("/proj/日本語.md"), "seed");
    let _fs = crate::fs::FsGuard::install(Arc::new(fs));
    for convention in [
        crate::keymap::KeymapFlavor::Native,
        crate::keymap::KeymapFlavor::Emacs,
    ] {
        let mut app = app();
        app.apply_keymap_flavor(convention);
        app.press_spec_headless(files_chord()).unwrap();
        assert!(app.workspace_state.overlay().unwrap().files_mode);
        let before = doc_state(&app);
        let bytes = app.document.buffer().disk_bytes();
        app.on_ime(Ime::Preedit("にほんご".into(), Some((12, 12))));
        assert!(app.workspace_state.overlay().unwrap().query.is_empty());
        app.on_ime(Ime::Preedit(String::new(), None));
        app.on_ime(Ime::Commit("日本語".into()));
        let card = app.workspace_state.overlay().unwrap();
        assert_eq!(card.query.text(), "日本語");
        assert_eq!(card.items.len(), 1);
        assert_eq!(card.selected_value(), Some("日本語.md"));
        assert_eq!(doc_state(&app), before);
        assert_eq!(app.document.buffer().disk_bytes(), bytes);
        assert_eq!(
            crate::fs::active()
                .read(&PathBuf::from("/proj/draft.md"))
                .unwrap(),
            before.0.as_bytes()
        );
        app.press_spec_headless(&format!("Escape {} d", files_chord()))
            .unwrap();
        assert_eq!(app.workspace_state.overlay().unwrap().query.text(), "d");
        assert_eq!(doc_state(&app), before);
    }
}

#[test]
fn ime_commit_replaces_each_focused_field_once_without_editing_the_document() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        let mut app = app();
        summon(&mut app, field);
        let before = doc_state(&app);
        select_field(&mut app, field);
        let text = match field {
            TextField::SettingsValue => "125",
            TextField::PickerQuery
            | TextField::Rename
            | TextField::InsertLink
            | TextField::KeepVersion
            | TextField::FindQuery
            | TextField::ReplaceText => "日本語",
        };
        app.on_ime(Ime::Commit(text.to_string()));
        assert_eq!(
            doc_state(&app),
            before,
            "{field:?}: IME edited the background document"
        );
        assert_eq!(
            field_text(&app, field).as_deref(),
            Some(text),
            "{field:?}: commit must replace the field selection exactly once"
        );
    }
}

/// Selected input is a legal state in every textbox. Only Files and Find expose
/// SelectAll as an action; this fixture does not broaden other cards' grammar.
pub(super) fn select_field(app: &mut App, field: TextField) {
    match field {
        TextField::FindQuery | TextField::ReplaceText => {
            app.apply(
                Action::SelectAll,
                false,
                &crate::app::schedule::RecordingExit::default(),
                crate::stats::Door::Menu,
            );
        }
        TextField::PickerQuery => app
            .workspace_state
            .overlay_mut()
            .unwrap()
            .query
            .select_all(),
        TextField::Rename => app
            .workspace_state
            .overlay_mut()
            .unwrap()
            .rename_edit
            .as_mut()
            .unwrap()
            .input
            .select_all(),
        TextField::InsertLink => app
            .workspace_state
            .overlay_mut()
            .unwrap()
            .link_edit
            .as_mut()
            .unwrap()
            .input
            .select_all(),
        TextField::KeepVersion => app
            .workspace_state
            .overlay_mut()
            .unwrap()
            .keep_edit
            .as_mut()
            .unwrap()
            .input
            .select_all(),
        TextField::SettingsValue => app
            .workspace_state
            .overlay_mut()
            .unwrap()
            .value_edit
            .as_mut()
            .unwrap()
            .input
            .select_all(),
    }
}

#[test]
fn ime_preedit_empty_preedit_commit_and_disable_keep_one_recipient() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        let mut app = app();
        summon(&mut app, field);
        let before = doc_state(&app);
        let old = field_text(&app, field);
        app.on_ime(Ime::Enabled);
        app.on_ime(Ime::Preedit("にほん".into(), Some((3, 3))));
        assert_eq!(
            field_text(&app, field),
            old,
            "preedit is transient: {field:?}"
        );
        let mut view = ViewState::base();
        app.project_text_input(&mut view);
        assert!(
            view.preedit.is_empty(),
            "field composition reached the document renderer"
        );
        let shown = view.field_input.unwrap();
        assert_eq!(shown.field, field);
        assert_eq!(shown.preedit.map(|(s, e)| e - s), Some(3));
        app.on_ime(Ime::Preedit(String::new(), None));
        assert!(app.input.preedit().is_empty());
        app.on_ime(Ime::Commit("日本".into()));
        assert_eq!(doc_state(&app), before, "{field:?}");
        if field != TextField::SettingsValue {
            assert!(field_text(&app, field).unwrap().contains("日本"));
        } else {
            assert_eq!(
                field_text(&app, field),
                old,
                "numeric filtering must survive IME"
            );
        }
        app.on_ime(Ime::Preedit("語".into(), None));
        app.on_ime(Ime::Disabled);
        assert!(app.input.preedit().is_empty());
    }
}

#[test]
fn ime_late_commit_cannot_cross_dismissal_or_reopened_surface_identity() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for reopen in [false, true] {
        let mut app = app();
        summon(&mut app, TextField::PickerQuery);
        let before = doc_state(&app);
        app.on_ime(Ime::Preedit("にほん".into(), None));
        app.workspace_state.dismiss_pickers();
        if reopen {
            summon(&mut app, TextField::PickerQuery);
        }
        let old = field_text(&app, TextField::PickerQuery);
        app.on_ime(Ime::Preedit(String::new(), None));
        app.on_ime(Ime::Commit("日本".into()));
        assert_eq!(doc_state(&app), before);
        assert_eq!(field_text(&app, TextField::PickerQuery), old);
    }
}

#[test]
fn ime_cannot_cross_document_to_field_or_find_to_replace_focus() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let mut app = app();
    let before = doc_state(&app);
    app.on_ime(Ime::Preedit("にほん".into(), None));
    summon(&mut app, TextField::FindQuery);
    let old = field_text(&app, TextField::FindQuery);
    app.on_ime(Ime::Commit("日本".into()));
    assert_eq!(field_text(&app, TextField::FindQuery), old);
    assert_eq!(doc_state(&app), before);
    app.on_ime(Ime::Preedit("にほん".into(), None));
    app.workspace_state
        .search_mut()
        .unwrap()
        .focus_replacement();
    app.on_ime(Ime::Preedit(String::new(), None));
    app.on_ime(Ime::Commit("日本".into()));
    assert_eq!(
        field_text(&app, TextField::ReplaceText).as_deref(),
        Some("")
    );
    assert_eq!(doc_state(&app), before);
    app.on_ime(Ime::Enabled);
    app.on_ime(Ime::Commit("新規".into()));
    assert_eq!(
        field_text(&app, TextField::ReplaceText).as_deref(),
        Some("新規")
    );
}

#[test]
fn ime_document_composition_keeps_typing_and_undo_semantics() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let mut app = app();
    let before = app.document.buffer().text();
    app.on_ime(Ime::Preedit("にほん".into(), Some((3, 3))));
    assert_eq!(app.document.buffer().text(), before);
    app.on_ime(Ime::Preedit(String::new(), None));
    app.on_ime(Ime::Commit("日本".into()));
    assert_eq!(app.document.buffer().text(), format!("日本{before}"));
    app.apply(
        Action::Undo,
        false,
        &crate::app::schedule::RecordingExit::default(),
        crate::stats::Door::Menu,
    );
    assert_eq!(app.document.buffer().text(), before);
}

#[test]
fn ime_every_non_text_surface_refuses_background_edits() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    use crate::overlay::{OverlayKind, OverlayState};
    for kind in OverlayKind::ALL {
        let text = match kind {
            OverlayKind::Credits
            | OverlayKind::Context
            | OverlayKind::Spell
            | OverlayKind::TableDims
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName => false,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::SearchFolder
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Settings
            | OverlayKind::History
            | OverlayKind::Conflict => true,
        };
        let mut app = app();
        app.workspace_state
            .install_overlay_for_test(OverlayState::new(kind, vec!["日本".into()], vec![], vec![]));
        let before = doc_state(&app);
        app.on_ime(Ime::Preedit("にほん".into(), None));
        app.on_ime(Ime::Commit("日本".into()));
        assert_eq!(doc_state(&app), before, "{kind:?}");
        assert_eq!(
            app.workspace_state.overlay().unwrap().query.text(),
            if text { "日本" } else { "" },
            "{kind:?}"
        );
    }
    for set_open in [
        crate::about::set_open,
        crate::lifetime::set_open,
        crate::streaks::set_open,
    ] {
        let mut app = app();
        let before = doc_state(&app);
        set_open(true);
        app.on_ime(Ime::Commit("日本".into()));
        set_open(false);
        assert_eq!(doc_state(&app), before);
    }
}
