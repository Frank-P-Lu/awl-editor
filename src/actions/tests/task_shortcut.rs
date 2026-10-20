use super::*;

#[test]
fn task_shortcut_converts_only_typed_completion_in_parser_owned_list_context() {
    let _g = crate::testlock::serial();
    for (source, line, expected) in [
        ("- []", 0, "- [ ] "),
        ("* []", 0, "* [ ] "),
        ("+ []", 0, "+ [ ] "),
        ("- outer\n  - []", 1, "- outer\n  - [ ] "),
        ("# 日本語\n- []", 1, "# 日本語\n- [ ] "),
    ] {
        for action in [Action::InsertChar(' '), Action::InsertText(" ".into())] {
            let mut buffer = Buffer::from_str(source);
            buffer.set_cursor(buffer.line_col_to_char(line, usize::MAX));
            drive_act(&mut buffer, &action);
            assert_eq!(buffer.text(), expected);
            assert_eq!(
                buffer.cursor_line_col().1,
                expected.lines().last().unwrap().chars().count()
            );
            buffer.undo();
            assert_eq!(
                buffer.text(),
                format!("{source} "),
                "one undo restores the typed literal"
            );
            buffer.redo();
            assert_eq!(buffer.text(), expected);
        }
    }
}

#[test]
fn task_shortcut_keeps_code_literals_paste_and_existing_task_spelling() {
    let _g = crate::testlock::serial();
    for (source, line) in [
        ("[]", 0),
        ("plain []", 0),
        ("- literal []", 0),
        ("1. []", 0),
        ("> - []", 0),
        ("    - []", 0),
        ("```\n- []\n```", 1),
        ("~~~\n- []\n~~~", 1),
        ("- [ ]", 0),
    ] {
        let mut buffer = Buffer::from_str(source);
        let cursor = buffer.line_col_to_char(line, usize::MAX);
        buffer.set_cursor(cursor);
        drive_act(&mut buffer, &Action::InsertChar(' '));
        let mut expected = source.to_string();
        expected.insert(buffer.char_to_byte(cursor), ' ');
        assert_eq!(buffer.text(), expected, "{source:?}");
    }
    let mut buffer = Buffer::from_str("- []");
    buffer.buffer_end();
    drive_act(&mut buffer, &Action::PasteText(" ".into()));
    assert_eq!(buffer.text(), "- [] ");
}

#[test]
fn task_shortcut_preserves_selection_bulk_typing_plain_text_and_body_undo() {
    let _g = crate::testlock::serial();
    let mut plain = Buffer::from_str("- []");
    plain.set_path("/notes/literal.txt".into());
    plain.buffer_end();
    drive_act(&mut plain, &Action::InsertChar(' '));
    assert_eq!(plain.text(), "- [] ");

    let mut selected = Buffer::from_str("- []x");
    selected.buffer_end();
    selected.set_anchor(4);
    drive_act(&mut selected, &Action::InsertText(" ".into()));
    assert_eq!(selected.text(), "- [] ");
    selected.undo();
    assert_eq!(selected.text(), "- []x");

    for text in ["[] ", "- [] ", " body"] {
        let mut bulk = Buffer::from_str("- ");
        bulk.buffer_end();
        drive_act(&mut bulk, &Action::InsertText(text.into()));
        assert_eq!(bulk.text(), format!("- {text}"));
    }
    let mut typed = Buffer::from_str("");
    for c in "- [] ".chars() {
        drive_act(&mut typed, &Action::InsertChar(c));
    }
    assert_eq!(typed.text(), "- [ ] ");
    drive_act(&mut typed, &Action::InsertText("日本語".into()));
    assert_eq!(typed.text(), "- [ ] 日本語");
    typed.undo();
    assert_eq!(typed.text(), "- [ ] ");
    typed.undo();
    assert_eq!(typed.text(), "- [] ");
    typed.redo();
    assert_eq!(typed.text(), "- [ ] ");
    drive_act(&mut typed, &Action::InsertText("todo".into()));
    drive_act(&mut typed, &Action::Newline);
    assert_eq!(typed.text(), "- [ ] todo\n- [ ] ");
}
