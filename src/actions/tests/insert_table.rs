//! `Action::InsertTable` end-to-end journeys through the real
//! `apply_transition` seam — mirrors `link_flow.rs`'s shape (its own closest
//! sibling: both open a minibuffer sub-state, both commit by mutating the
//! buffer directly rather than through a generic `OverlayAccept`).

use super::super::*;
use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::{
    DEFAULT_COLS, DEFAULT_ROWS, Journey, MAX_COLS, MAX_ROWS, OverlayKind, OverlayState,
};

fn drive(buffer: &mut Buffer, journey: &mut Journey, action: Action) {
    let mut shift_selecting = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut make_overlay = |_kind: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to =
        |_kind: OverlayKind, _root: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer,
        shift_selecting: &mut shift_selecting,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, &action, false).primary();
}

/// KEYBOARD JOURNEY: summon, sculpt rows/cols with the arrow-mapped actions,
/// commit with `Enter` as ONE undoable edit, and land the caret in the first
/// header cell -- the exact shape a `--keys` chord replay drives.
#[test]
fn keyboard_journey_sculpts_and_inserts_landing_in_the_first_header_cell() {
    let mut buffer = Buffer::from_str("intro\n");
    buffer.set_cursor(buffer.line_col_to_char(0, 5)); // end of "intro"
    let before_version = buffer.version();
    let mut journey = Journey::default();

    drive(&mut buffer, &mut journey, Action::InsertTable);
    let ov = journey.card().expect("the picker opens");
    assert_eq!(
        ov.table_dims_target(),
        Some((DEFAULT_ROWS, DEFAULT_COLS)),
        "seeded at the modest default"
    );

    // ↓ ↓ sculpts rows +2, → sculpts cols +1.
    drive(&mut buffer, &mut journey, Action::NextLine);
    drive(&mut buffer, &mut journey, Action::NextLine);
    drive(&mut buffer, &mut journey, Action::ForwardChar);
    let (rows, cols) = journey
        .card()
        .expect("still open while sculpting")
        .table_dims_target()
        .expect("a table-dims edit is active");
    assert_eq!((rows, cols), (DEFAULT_ROWS + 2, DEFAULT_COLS + 1));

    drive(&mut buffer, &mut journey, Action::Newline);
    assert!(journey.card().is_none(), "commit closes the picker");
    assert!(
        buffer.version() > before_version,
        "the commit is a real edit"
    );

    let table = crate::markdown::build_table(rows, cols);
    let text = buffer.text();
    assert_eq!(
        text,
        format!("intro\n\n{table}\n\n"),
        "a blank line opens before the table (the caret sat right after \
         \"intro\", before its own trailing newline) and one closes it \
         after (the original single trailing newline becomes a full blank \
         line, since a table must sit in its own GFM block)"
    );

    // Caret lands right after the table's own leading "| " -- the first
    // header cell.
    let table_start = text.find(&table).unwrap();
    let table_start_chars = text[..table_start].chars().count();
    assert_eq!(
        buffer.cursor_char(),
        table_start_chars + crate::markdown::FIRST_CELL_OFFSET
    );

    buffer.undo();
    assert_eq!(
        buffer.text(),
        "intro\n",
        "one undo restores every source byte"
    );
}

/// ARROW-KEY CLAMPING through the real seam: driving past either bound never
/// panics and holds at `MIN_DIM`/`MAX_ROWS`/`MAX_COLS`.
#[test]
fn keyboard_journey_clamps_at_both_bounds_through_the_real_seam() {
    let mut buffer = Buffer::from_str("");
    let mut journey = Journey::default();
    drive(&mut buffer, &mut journey, Action::InsertTable);

    for _ in 0..(MAX_ROWS + 5) {
        drive(&mut buffer, &mut journey, Action::NextLine);
    }
    for _ in 0..(MAX_COLS + 5) {
        drive(&mut buffer, &mut journey, Action::ForwardChar);
    }
    assert_eq!(
        journey.card().unwrap().table_dims_target(),
        Some((MAX_ROWS, MAX_COLS))
    );

    for _ in 0..(MAX_ROWS + MAX_COLS + 10) {
        drive(&mut buffer, &mut journey, Action::PreviousLine);
        drive(&mut buffer, &mut journey, Action::BackwardChar);
    }
    assert_eq!(
        journey.card().unwrap().table_dims_target(),
        Some((crate::overlay::MIN_DIM, crate::overlay::MIN_DIM))
    );
}

/// TYPED-DIGIT JOURNEY: real `InsertChar`/`DeleteBackward` actions drive the
/// forgiving `RxC` parse, and the drawn readout tracks it live (surfaced via
/// `OverlayState::foot_hint`, so a sidecar capture sees the same string).
#[test]
fn typed_digit_journey_parses_forgivingly_and_commits_the_typed_size() {
    let mut buffer = Buffer::from_str("");
    let mut journey = Journey::default();
    drive(&mut buffer, &mut journey, Action::InsertTable);

    for c in "5x3".chars() {
        drive(&mut buffer, &mut journey, Action::InsertChar(c));
    }
    assert_eq!(journey.card().unwrap().table_dims_target(), Some((5, 3)));
    assert_eq!(
        journey.card().unwrap().foot_hint(),
        "5 × 3 table   ↵ insert   Esc cancel"
    );

    // Backspace the trailing digit; the incomplete "5x" leaves 5x3 standing.
    drive(&mut buffer, &mut journey, Action::DeleteBackward);
    assert_eq!(journey.card().unwrap().table_dims_target(), Some((5, 3)));

    drive(&mut buffer, &mut journey, Action::Newline);
    let table = crate::markdown::build_table(5, 3);
    assert_eq!(
        buffer.text(),
        table,
        "empty document: no padding blank lines"
    );
}

/// `Esc` CANCELS with zero buffer change and no undo entry -- mirrors
/// `link_flow.rs`'s own cancel law exactly.
#[test]
fn esc_cancels_with_no_buffer_change() {
    let mut buffer = Buffer::from_str("hello world");
    buffer.set_cursor(5);
    let mut journey = Journey::default();

    drive(&mut buffer, &mut journey, Action::InsertTable);
    assert!(journey.card().is_some());
    drive(&mut buffer, &mut journey, Action::NextLine); // sculpt before cancelling
    drive(&mut buffer, &mut journey, Action::Cancel);

    assert!(journey.card().is_none(), "Esc closes the picker");
    assert_eq!(
        buffer.text(),
        "hello world",
        "cancel never edits the buffer"
    );
    assert!(!buffer.can_undo(), "…so there is nothing to undo");
}

/// A POINTER PICK is the mouse's own route to the exact state an arrow-key
/// sculpt reaches (`OverlayState::table_dims_pick`, wired to a real click in
/// `app/input/mouse.rs::overlay_click` -- the live gesture itself has no
/// `--keys` vocabulary and is flagged for human confirmation, but the STATE
/// MUTATION it drives is this seam). Picking a cell then committing with
/// `Enter` inserts exactly that size.
#[test]
fn pointer_pick_sets_dims_from_the_clicked_cell_and_commits_on_enter() {
    let mut buffer = Buffer::from_str("");
    let mut journey = Journey::default();
    drive(&mut buffer, &mut journey, Action::InsertTable);

    journey.card_mut().unwrap().table_dims_pick(3, 6); // 0-based -> 4 rows x 7 cols
    assert_eq!(journey.card().unwrap().table_dims_target(), Some((4, 7)));

    drive(&mut buffer, &mut journey, Action::Newline);
    assert_eq!(buffer.text(), crate::markdown::build_table(4, 7));
}

/// Markdown-only, like every other formatting command: a non-markdown buffer
/// never opens the picker.
#[test]
fn is_a_calm_no_op_off_markdown() {
    let mut buffer = Buffer::from_str("plain code content");
    buffer.set_path(std::path::PathBuf::from("/tmp/x.rs"));
    assert!(!buffer.is_markdown());
    let mut journey = Journey::default();
    drive(&mut buffer, &mut journey, Action::InsertTable);
    assert!(
        journey.card().is_none(),
        "no picker opens off a non-markdown buffer"
    );
}
