use super::super::*;
use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::{Journey, OverlayKind, OverlayState};

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

/// Full flow: Cmd-K on a selection → type a URL → Enter commits as one
/// undoable edit → Cmd-Z restores the exact pre-edit text and cursor.
#[test]
fn full_wrap_flow_commits_one_undoable_edit_and_undo_restores_exactly() {
    let mut buffer = Buffer::from_str("hello world");
    buffer.select_range(0, 5);
    let before_version = buffer.version();
    let mut journey = Journey::default();

    drive(&mut buffer, &mut journey, Action::InsertLink);
    let overlay = journey.card_mut().expect("overlay must open");
    for ch in "https://example.com".chars() {
        overlay.link_edit_push(ch);
    }
    drive(&mut buffer, &mut journey, Action::Newline);

    assert!(journey.card().is_none(), "commit closes the overlay");
    assert_eq!(buffer.text(), "[hello](https://example.com) world");
    assert!(
        buffer.version() > before_version,
        "the commit is a real edit"
    );
    buffer.undo();
    assert_eq!(buffer.text(), "hello world");
    assert_eq!(
        buffer.cursor_char(),
        5,
        "undo restores the pre-commit cursor; format history carries no anchor"
    );
}

#[test]
fn esc_cancels_with_no_buffer_change() {
    let mut buffer = Buffer::from_str("hello world");
    buffer.set_cursor(5);
    let mut journey = Journey::default();

    drive(&mut buffer, &mut journey, Action::InsertLink);
    assert!(journey.card().is_some());
    drive(&mut buffer, &mut journey, Action::Cancel);

    assert!(journey.card().is_none(), "Esc closes the minibuffer");
    assert_eq!(
        buffer.text(),
        "hello world",
        "cancel never edits the buffer"
    );
}
