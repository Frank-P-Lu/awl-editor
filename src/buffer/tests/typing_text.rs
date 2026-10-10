//! Literal typing payloads retain selection and whitespace history laws.
use super::*;

#[test]
fn complete_typing_replaces_selection_once_and_keeps_scalar_cursor() {
    let _guard = crate::testlock::serial();
    for reverse in [false, true] {
        let mut buf = b("前abc後\n");
        let (anchor, caret) = if reverse { (4, 1) } else { (1, 4) };
        buf.select_range(anchor, caret);
        let version = buf.version();
        buf.insert_chars("e\u{301}👩‍💻\n\t\r");
        assert_eq!(buf.text(), "前e\u{301}👩‍💻\n\t\r後\n");
        assert_eq!(buf.cursor_char(), 9);
        assert_eq!(buf.version(), version + 1);
        assert!(!buf.has_selection());
        buf.undo();
        assert_eq!(buf.text(), "前abc後\n");
        assert_eq!(buf.cursor_char(), caret);
        assert!(!buf.can_undo());
        buf.redo();
        assert_eq!(buf.text(), "前e\u{301}👩‍💻\n\t\r後\n");
        assert_eq!(buf.cursor_char(), 9);
    }
}

#[test]
fn unselected_text_keeps_raw_newline_tab_and_whitespace_history() {
    let _guard = crate::testlock::serial();
    let mut buf = b("後\n");
    buf.insert_chars("a b\n\tc");
    assert_eq!(buf.text(), "a b\n\tc後\n");
    assert_eq!(buf.cursor_char(), 6);
    for prefix in ["a b\n\t", "a b\n", "a ", ""] {
        buf.undo();
        assert_eq!(buf.text(), format!("{prefix}後\n"));
    }
    assert!(!buf.can_undo());
    for prefix in ["a ", "a b\n", "a b\n\t", "a b\n\tc"] {
        buf.redo();
        assert_eq!(buf.text(), format!("{prefix}後\n"));
    }
}

#[test]
fn empty_typing_payload_preserves_selection_and_does_not_seal_typing() {
    let _guard = crate::testlock::serial();
    let mut buf = b("abc");
    buf.select_range(3, 0);
    let version = buf.version();
    buf.insert_chars("");
    assert_eq!(buf.text(), "abc");
    assert_eq!(buf.selection_range(), Some((0, 3)));
    assert_eq!(buf.cursor_char(), 0);
    assert_eq!(buf.version(), version);
    assert!(!buf.can_undo());
    let mut buf = b("");
    buf.insert_char('x');
    buf.insert_chars("");
    buf.insert_chars("日本");
    buf.undo();
    assert_eq!(buf.text(), "");
    assert!(!buf.can_undo());
}
