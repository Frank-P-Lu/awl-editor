use super::super::*;
use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::{Journey, OverlayKind, OverlayState};

fn drive(buffer: &mut Buffer, action: Action) {
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = Journey::default();
    let mut make = |_kind: OverlayKind| -> Option<OverlayState> { None };
    let mut browse = |_kind: OverlayKind, _root: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make,
        browse_to: &mut browse,
        oracle: None,
    };
    let _ = apply_transition(&mut ctx, &action, false);
}

#[test]
fn table_tab_walks_containing_cell_then_appends_and_undoes_as_one_edit() {
    let mut b = Buffer::from_str("| α | b\\|c |\n|---|---|\n| d | e |\n");
    b.set_cursor(b.line_col_to_char(0, 3));
    drive(&mut b, Action::InsertTab);
    assert_eq!(
        b.cursor_line_col(),
        (0, 6),
        "inside the first cell advances once"
    );
    drive(&mut b, Action::InsertTab);
    assert_eq!(b.cursor_line_col().0, 2, "escaped pipe remains in its cell");
    b.set_cursor(b.line_col_to_char(2, 3));
    drive(&mut b, Action::InsertTab);
    b.set_cursor(b.line_col_to_char(2, 7));
    drive(&mut b, Action::InsertTab);
    assert!(
        b.text().contains("\n| | |"),
        "last cell appends a scaffold row"
    );
    drive(&mut b, Action::Undo);
    assert!(
        !b.text().contains("\n| | |\n|"),
        "append is one undoable edit"
    );
}

#[test]
fn table_enter_keeps_header_and_separator_adjacent_and_shift_enter_splits() {
    let mut b = Buffer::from_str("| h |\n|---|\n| b |\n");
    b.set_cursor(b.line_col_to_char(0, 3));
    drive(&mut b, Action::Newline);
    assert!(b.text().starts_with("| h |\n|---|\n| |\n"));
    let mut plain = Buffer::from_str("word");
    plain.set_cursor(2);
    drive(&mut plain, Action::InsertTab);
    assert_eq!(plain.text(), "wo  rd", "ordinary Tab is unchanged");
    drive(&mut plain, Action::AcceptAlternate);
    assert_eq!(
        plain.text(),
        "wo  \nrd",
        "Shift-Enter remains a literal split"
    );
}

#[test]
fn table_dispatch_sweeps_one_column_ragged_unicode_and_selection_contexts() {
    let mut one = Buffer::from_str("| 東京 |\n|---|\n| x |\n");
    one.set_cursor(one.line_col_to_char(0, 3));
    drive(&mut one, Action::InsertTab);
    assert_eq!(
        one.cursor_line_col().0,
        2,
        "one-column header wraps to body"
    );
    drive(&mut one, Action::Outdent);
    assert_eq!(one.cursor_line_col().0, 0, "Shift-Tab wraps backwards");

    let mut ragged = Buffer::from_str("| a | b |\n|---|---|\n| 終 |\n");
    ragged.set_cursor(ragged.line_col_to_char(2, 3));
    drive(&mut ragged, Action::Newline);
    assert!(
        ragged.text().contains("| 終 |\n| | |"),
        "ragged row gets full scaffold width"
    );

    let mut selected = Buffer::from_str("| a |\n|---|\n| b |\n");
    selected.set_cursor(0);
    selected.set_mark();
    selected.set_cursor(selected.line_col_to_char(2, 3));
    let mut ordinary = Buffer::from_str("| a |\n|---|\n| b |\n");
    ordinary.set_cursor(0);
    ordinary.set_mark();
    ordinary.set_cursor(ordinary.line_col_to_char(2, 3));
    drive(&mut selected, Action::InsertTab);
    drive(&mut ordinary, Action::InsertTab);
    assert_eq!(
        selected.text(),
        ordinary.text(),
        "selection keeps ordinary Tab behavior"
    );
}
