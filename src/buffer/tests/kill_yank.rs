use super::*;

#[test]
fn kill_line_to_eol() {
    let mut buf = b("hello world\nsecond");
    for _ in 0..6 {
        buf.forward_char();
    }
    buf.kill_line();
    assert_eq!(buf.text(), "hello \nsecond");
    assert_eq!(buf.kill_buffer(), "world");
}

#[test]
fn kill_line_at_eol_kills_newline() {
    let mut buf = b("hello\nworld");
    buf.line_end_motion(); // end of "hello", before '\n'
    buf.kill_line(); // kills the newline -> join
    assert_eq!(buf.text(), "helloworld");
}

#[test]
fn delete_to_line_start_removes_back_to_line_start_undoable_no_kill_ring() {
    // Cmd-⌫: remove from the caret back to the LOGICAL line start, caret lands
    // there. It does NOT touch the kill ring (a delete, not a cut) and is one
    // undoable step.
    let mut buf = b("hello world\nsecond");
    for _ in 0..6 {
        buf.forward_char(); // caret just after "hello " (col 6)
    }
    buf.delete_to_line_start();
    assert_eq!(buf.text(), "world\nsecond");
    assert_eq!(buf.cursor_char(), 0, "caret lands at the line start");
    assert_eq!(
        buf.kill_buffer(),
        "",
        "delete-to-line-start never fills the kill ring"
    );
    buf.undo();
    assert_eq!(
        buf.text(),
        "hello world\nsecond",
        "one undoable step restores the prefix"
    );

    // On a LATER line it stops at THAT line's start (never crosses the newline).
    let mut buf = b("alpha\nbeta gamma");
    buf.next_line(); // line 1, col 0
    for _ in 0..5 {
        buf.forward_char(); // after "beta " (col 5)
    }
    buf.delete_to_line_start();
    assert_eq!(buf.text(), "alpha\ngamma");
    // At column 0 it is a calm no-op — the version does not bump.
    let v = buf.version();
    buf.delete_to_line_start();
    assert_eq!(buf.text(), "alpha\ngamma");
    assert_eq!(
        buf.version(),
        v,
        "no-op at the line start leaves the version untouched"
    );
}

#[test]
fn consecutive_kills_append() {
    let mut buf = b("hello world\n");
    // kill "hello world" then the newline, accumulating in kill buffer
    buf.kill_line();
    assert_eq!(buf.kill_buffer(), "hello world");
    buf.kill_line(); // at eol now -> kills newline, appends
    assert_eq!(buf.kill_buffer(), "hello world\n");
    assert_eq!(buf.text(), "");
}

#[test]
fn consecutive_kills_coalesce_into_one_undo_group() {
    // C-k C-k (kill the line's content, then its newline) is ONE user
    // gesture, so a single C-/ restores it fully — even though the first
    // kill removed whitespace-bearing text (which normally seals a group).
    let mut buf = b("foo bar baz\nsecond");
    buf.kill_line(); // kill "foo bar baz"
    buf.kill_line(); // at eol -> kill the newline, joining "second"
    assert_eq!(buf.text(), "second");
    buf.undo(); // ONE undo must restore the whole kill run
    assert_eq!(buf.text(), "foo bar baz\nsecond");
    assert!(
        !buf.can_undo(),
        "the kill run should be a single undo group"
    );
}

#[test]
fn kill_then_move_resets_accumulation() {
    let mut buf = b("aaa\nbbb");
    buf.kill_line(); // kill "aaa", kill="aaa"
    assert_eq!(buf.kill_buffer(), "aaa");
    buf.forward_char(); // a motion resets the kill flag
    buf.line_end_motion();
    buf.kill_line(); // now on the (joined) tail; fresh kill, not appended
    assert_ne!(buf.kill_buffer(), "aaa\n");
}

#[test]
fn yank_inserts_kill_buffer() {
    let mut buf = b("hello world");
    for _ in 0..6 {
        buf.forward_char();
    }
    buf.kill_line(); // kill "world"
    buf.buffer_start();
    buf.yank();
    assert_eq!(buf.text(), "worldhello ");
    assert_eq!(buf.cursor_char(), 5);
}

#[test]
fn kill_and_yank_roundtrip() {
    let mut buf = b("line one\nline two");
    buf.kill_line(); // kill "line one"
    buf.delete_forward(); // remove the leftover newline
    // buffer now "line two", kill = "line one"
    buf.buffer_end();
    buf.insert_newline();
    buf.yank();
    assert_eq!(buf.text(), "line two\nline one");
}

#[test]
fn dirty_flag_tracks_edits() {
    let mut buf = b("x");
    assert!(!buf.is_dirty());
    buf.forward_char();
    assert!(!buf.is_dirty()); // motion doesn't dirty
    buf.insert_char('y');
    assert!(buf.is_dirty());
}
