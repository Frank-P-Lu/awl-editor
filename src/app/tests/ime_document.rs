//! Document composition projects its platform cursor without editing saved text.
use super::summoned_field_actions::{app, doc_state, seeded, summon};
use super::*;
use crate::textbox::TextField;
use std::sync::Arc;

fn projected(app: &App) -> ViewState {
    let mut view = ViewState::base();
    view.preedit = app.input.preedit().into();
    app.project_text_input(&mut view);
    view
}

#[test]
fn document_preedit_cursor_reaches_live_and_capture_projection_without_edits() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let mut app = app();
    app.document.set_cursor(2);
    let before = doc_state(&app);
    let disk = app.document.buffer().disk_bytes();
    let cases = [
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 2),
        (6, 2),
        (7, 3),
        (9, 3),
        (99, 3),
    ];
    for (byte, expected) in cases {
        app.on_ime(Ime::Preedit("にほん".into(), Some((byte, 99))));
        let view = projected(&app);
        assert_eq!(view.preedit, "にほん");
        assert_eq!(view.preedit_cursor, Some(expected), "byte {byte}");
        let opts = app.capture_opts();
        assert_eq!(opts.preedit.as_deref(), Some("にほん"));
        assert_eq!(opts.preedit_cursor, view.preedit_cursor);
        assert_eq!(doc_state(&app), before);
        assert_eq!(app.document.buffer().disk_bytes(), disk);
    }
    app.on_ime(Ime::Preedit("にほん".into(), None));
    assert_eq!(projected(&app).preedit_cursor, Some(3));
    app.on_ime(Ime::Preedit(String::new(), None));
    assert_eq!(projected(&app).preedit_cursor, None);
    assert_eq!(app.capture_opts().preedit_cursor, None);
    app.on_ime(Ime::Commit("日本".into()));
    assert_eq!(projected(&app).preedit_cursor, None);
    app.apply(
        Action::Undo,
        false,
        &crate::app::schedule::RecordingExit::default(),
        crate::stats::Door::Menu,
    );
    assert_eq!(app.document.buffer().text(), before.0);
}

#[test]
fn document_preedit_cursor_is_inert_after_disable_and_field_projection() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    let mut app = app();
    let before = doc_state(&app);
    app.on_ime(Ime::Preedit("にほん".into(), Some((3, 3))));
    app.on_ime(Ime::Disabled);
    assert_eq!(projected(&app).preedit_cursor, None);
    assert_eq!(app.capture_opts().preedit_cursor, None);
    assert_eq!(doc_state(&app), before);
    summon(&mut app, TextField::FindQuery);
    super::ime_fields::select_field(&mut app, TextField::FindQuery);
    app.on_ime(Ime::Preedit("にほん".into(), Some((3, 3))));
    let view = projected(&app);
    assert!(view.preedit.is_empty());
    assert_eq!(view.preedit_cursor, None);
    assert_eq!(view.field_input.unwrap().caret, 1);
    assert_eq!(app.capture_opts().preedit_cursor, None);
    assert_eq!(doc_state(&app), before);
    let mut preview = ViewState::base();
    app.project_text_input(&mut preview);
    assert_eq!(preview.preedit_cursor, None);
}
