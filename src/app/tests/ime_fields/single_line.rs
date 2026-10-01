//! Platform commits obey the same single-line boundary as external paste.

use super::*;

#[test]
fn ime_commits_strip_every_line_separator_in_each_focused_field() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        let mut app = app();
        summon(&mut app, field);
        select_field(&mut app, field);
        let document = doc_state(&app);
        app.on_ime(Ime::Preedit("125".into(), None));
        app.on_ime(Ime::Preedit(String::new(), None));
        app.on_ime(Ime::Commit("1\u{2028}2\u{2029}5\r\n\u{85}".into()));
        assert_eq!(field_text(&app, field).as_deref(), Some("125"), "{field:?}");
        assert_eq!(doc_state(&app), document, "{field:?}");
    }
}

#[test]
fn ime_commits_with_only_line_separators_preserve_field_selection() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        let mut app = app();
        summon(&mut app, field);
        select_field(&mut app, field);
        app.on_ime(Ime::Commit("125".into()));
        select_field(&mut app, field);
        let before = app.focused_text_box().unwrap().clone();
        assert_eq!(before.selection_range(), Some((0, 3)), "{field:?}");
        let document = doc_state(&app);
        app.on_ime(Ime::Commit("\r\n\t\u{85}\u{2028}\u{2029}".into()));
        assert_eq!(app.focused_text_box(), Some(&before), "{field:?}");
        assert_eq!(doc_state(&app), document, "{field:?}");
    }
}
