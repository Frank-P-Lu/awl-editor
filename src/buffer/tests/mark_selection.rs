use super::*;

// --- Selection tests --------------------------------------------------

#[test]
fn set_mark_then_motion_extends_region() {
    let mut buf = b("hello world");
    buf.set_mark(); // anchor at 0
    for _ in 0..5 {
        buf.forward_char();
    }
    // region is [0,5) = "hello"
    assert_eq!(buf.selection_range(), Some((0, 5)));
    assert!(buf.has_selection());
}

#[test]
fn clear_mark_drops_selection() {
    let mut buf = b("abc");
    buf.set_mark();
    buf.forward_char();
    assert!(buf.has_selection());
    buf.clear_mark();
    assert!(!buf.has_selection());
    assert_eq!(buf.selection_range(), None);
}

#[test]
fn selected_text_reads_the_active_region_ordered() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char();
    }
    assert_eq!(buf.selected_text().as_deref(), Some("hello"));
    // Ordered regardless of which end the cursor sits at.
    buf.clear_mark();
    buf.buffer_end();
    buf.set_mark();
    for _ in 0.."world".len() {
        buf.backward_char();
    }
    assert_eq!(buf.selected_text().as_deref(), Some("world"));
    // No selection => None.
    buf.clear_mark();
    assert_eq!(buf.selected_text(), None);
}

#[test]
fn selection_orders_endpoints_when_cursor_before_anchor() {
    let mut buf = b("abcdef");
    buf.buffer_end(); // cursor at 6
    buf.set_mark(); // anchor at 6
    for _ in 0..3 {
        buf.backward_char(); // cursor at 3, anchor 6
    }
    assert_eq!(buf.selection_range(), Some((3, 6))); // ordered
}

#[test]
fn selection_span_across_lines() {
    // "line0\nline1\nline2": anchor mid-line0, cursor mid-line2.
    let mut buf = b("line0\nline1\nline2");
    for _ in 0..2 {
        buf.forward_char(); // cursor at col 2 line 0
    }
    buf.set_mark();
    // move to line 2 col 3
    buf.next_line();
    buf.next_line();
    buf.line_start_motion();
    for _ in 0..3 {
        buf.forward_char();
    }
    let ((l0, c0), (l1, c1)) = buf.selection_line_col().unwrap();
    assert_eq!((l0, c0), (0, 2));
    assert_eq!((l1, c1), (2, 3));
}

#[test]
fn kill_region_cuts_and_fills_kill_buffer() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char(); // select "hello"
    }
    buf.kill_region();
    assert_eq!(buf.text(), " world");
    assert_eq!(buf.kill_buffer(), "hello");
    assert_eq!(buf.cursor_char(), 0);
    assert!(!buf.has_selection());
}

#[test]
fn kill_line_clears_a_backward_mark_no_oob_slice() {
    // Regression: C-k with a BACKWARD active mark (anchor AFTER the cursor)
    // used to leave `anchor` dangling past the rope's shrunk end, so the
    // next selection-consuming op sliced out of bounds and panicked in
    // ropey. C-k must deactivate the region (Emacs semantics).
    let mut buf = b("hello world");
    buf.buffer_end(); // cursor at 11
    buf.set_mark(); // anchor at 11
    buf.buffer_start(); // cursor at 0, anchor 11 (backward selection)
    assert_eq!(buf.selection_range(), Some((0, 11)));
    buf.kill_line(); // kills "hello world" -> rope now empty
    assert_eq!(buf.text(), "");
    assert!(!buf.has_selection(), "C-k deactivates the region");
    assert_eq!(buf.anchor_char(), None);
    // The op that used to panic: a copy with the stale backward mark.
    buf.copy_region(); // must NOT panic (no OOB slice)
    assert_eq!(buf.selection_range(), None);
}

#[test]
fn kill_line_clears_a_forward_mark_too() {
    // Control: a FORWARD mark (anchor BEFORE cursor) is likewise cleared.
    let mut buf = b("hello world");
    buf.set_mark(); // anchor at 0
    buf.buffer_end(); // cursor at 11, anchor 0 (forward selection)
    assert_eq!(buf.selection_range(), Some((0, 11)));
    buf.kill_line(); // at eol -> nothing to kill, but region deactivates
    assert!(!buf.has_selection(), "C-k deactivates the region");
    assert_eq!(buf.anchor_char(), None);
    buf.copy_region(); // must NOT panic
    assert_eq!(buf.selection_range(), None);
}

#[test]
fn set_kill_roundtrips_through_kill_buffer() {
    let mut buf = b("");
    buf.set_kill("hello");
    assert_eq!(buf.kill_buffer(), "hello");
    // overwrites, does not append
    buf.set_kill("world");
    assert_eq!(buf.kill_buffer(), "world");
    // empty is allowed and clears
    buf.set_kill("");
    assert_eq!(buf.kill_buffer(), "");
}

#[test]
fn set_kill_does_not_chain_with_kill_line() {
    // set_kill must NOT set last_was_kill, so a following C-k must REPLACE
    // (fresh kill), not append to, the value we set.
    let mut buf = b("abc\n");
    buf.set_kill("EXTERNAL");
    buf.kill_line(); // cursor at start of line -> kills "abc"
    assert_eq!(buf.kill_buffer(), "abc"); // replaced, NOT "EXTERNALabc"
}

#[test]
fn copy_region_keeps_text() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char();
    }
    buf.copy_region();
    assert_eq!(buf.text(), "hello world"); // unchanged
    assert_eq!(buf.kill_buffer(), "hello");
    assert!(!buf.has_selection()); // mark cleared by copy
}

#[test]
fn kill_then_yank_region_roundtrip() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char();
    }
    buf.kill_region(); // buffer " world", kill "hello"
    buf.buffer_end();
    buf.yank();
    assert_eq!(buf.text(), " worldhello");
}

#[test]
fn typing_replaces_selection() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char(); // select "hello"
    }
    buf.insert_char('X');
    assert_eq!(buf.text(), "X world");
    assert!(!buf.has_selection());
    assert_eq!(buf.cursor_char(), 1);
}

#[test]
fn backspace_deletes_selection() {
    let mut buf = b("hello world");
    buf.set_mark();
    for _ in 0..5 {
        buf.forward_char();
    }
    buf.delete_backward();
    assert_eq!(buf.text(), " world");
    assert!(!buf.has_selection());
}

#[test]
fn yank_replaces_selection() {
    let mut buf = b("hello world");
    // put "XX" in kill buffer via kill_region of a throwaway
    buf.select_range(0, 0);
    buf.kill = "XX".to_string();
    buf.select_range(0, 5); // select "hello"
    buf.yank();
    assert_eq!(buf.text(), "XX world");
}
