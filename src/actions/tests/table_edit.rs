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
        b.selected_text().as_deref(),
        Some("b\\|c"),
        "inside the first cell selects the next cell once"
    );
    drive(&mut b, Action::InsertTab);
    assert_eq!(b.cursor_line_col().0, 2, "escaped pipe remains in its cell");
    b.clear_mark();
    b.set_cursor(b.line_col_to_char(2, 3));
    drive(&mut b, Action::InsertTab);
    b.clear_mark();
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
fn table_tab_selects_content_and_empty_cells_land_after_the_opening_pipe() {
    let mut filled = Buffer::from_str("| one | two |\n|---|---|\n| a | b |\n");
    filled.set_cursor(filled.line_col_to_char(0, 3));
    drive(&mut filled, Action::InsertTab);
    assert_eq!(
        filled.selected_text().as_deref(),
        Some("two"),
        "Tab selects the target cell's trimmed content"
    );
    drive(&mut filled, Action::InsertChar('X'));
    assert_eq!(
        filled.line_text(0),
        "| one | X |",
        "typing replaces the selected cell instead of its pipes or padding"
    );

    let mut empty = Buffer::from_str("| filled |     |\n|---|---|\n| a | b |\n");
    empty.set_cursor(empty.line_col_to_char(0, 3));
    drive(&mut empty, Action::InsertTab);
    assert_eq!(
        empty.cursor_line_col(),
        (0, 10),
        "an empty target lands immediately after its opening pipe"
    );
    assert_eq!(
        empty.selection_range(),
        None,
        "an empty cell is a bare caret"
    );
    drive(&mut empty, Action::InsertChar('X'));
    assert_eq!(
        empty.line_text(0),
        "| filled |X     |",
        "typing starts inside the cell, never against its closing pipe"
    );
}

#[test]
fn table_tab_selection_walks_symmetrically_and_append_starts_inside_the_new_cell() {
    let mut b = Buffer::from_str("| first | last |\n|---|---|\n| a | b |\n");
    b.set_cursor(b.line_col_to_char(0, 3));
    drive(&mut b, Action::InsertTab);
    assert_eq!(b.selected_text().as_deref(), Some("last"));
    drive(&mut b, Action::Outdent);
    assert_eq!(
        b.selected_text().as_deref(),
        Some("first"),
        "Shift-Tab walks back from the selection Tab created"
    );
    drive(&mut b, Action::Outdent);
    assert_eq!(
        b.selected_text().as_deref(),
        Some("first"),
        "Shift-Tab on the first cell stays at the first cell"
    );

    b.clear_mark();
    b.set_cursor(b.line_col_to_char(2, 7));
    drive(&mut b, Action::InsertTab);
    assert_eq!(
        b.cursor_line_col(),
        (3, 1),
        "the appended row starts just inside its first opening pipe"
    );
    assert_eq!(
        b.selection_range(),
        None,
        "the appended empty cell is a caret"
    );
    drive(&mut b, Action::InsertChar('X'));
    assert!(
        b.text().contains("\n|X | |"),
        "typing fills the scaffold cell without replacing a pipe"
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
