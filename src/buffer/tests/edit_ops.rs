use super::*;

#[test]
fn insert_and_delete() {
    let mut buf = b("");
    buf.insert_char('h');
    buf.insert_char('i');
    assert_eq!(buf.text(), "hi");
    assert_eq!(buf.cursor_char(), 2);
    buf.delete_backward();
    assert_eq!(buf.text(), "h");
    buf.backward_char();
    buf.delete_forward();
    assert_eq!(buf.text(), "");
}

#[test]
fn delete_word_forward_mid_line() {
    // M-d at a word start deletes exactly that word (leaves trailing space).
    let mut buf = b("foo bar baz");
    buf.delete_word_forward();
    assert_eq!(buf.text(), " bar baz");
    assert_eq!(buf.cursor_char(), 0); // cursor stays; text collapsed to meet it
}

#[test]
fn delete_word_forward_stops_at_word_end() {
    // Mid-word, M-d removes only the rest of the current word — not the next.
    let mut buf = b("foo bar");
    buf.forward_char(); // cursor after 'f'
    buf.delete_word_forward();
    assert_eq!(buf.text(), "f bar");
    assert_eq!(buf.cursor_char(), 1);
}

#[test]
fn delete_word_forward_skips_leading_whitespace() {
    // Like M-f, it skips a run of non-word chars, then eats the word.
    let mut buf = b("foo   bar baz");
    for _ in 0..3 {
        buf.forward_char(); // cursor at the first space (col 3)
    }
    buf.delete_word_forward();
    assert_eq!(buf.text(), "foo baz"); // "   bar" removed
    assert_eq!(buf.cursor_char(), 3);
}

#[test]
fn delete_word_forward_end_of_buffer_is_noop() {
    let mut buf = b("foo");
    buf.buffer_end();
    buf.delete_word_forward(); // no panic, no over-delete
    assert_eq!(buf.text(), "foo");
    assert_eq!(buf.cursor_char(), 3);
}

#[test]
fn delete_word_forward_is_char_safe() {
    // Multi-byte chars are word chars indexed by CHAR, so no byte-boundary panic.
    let mut buf = b("café wörld");
    buf.delete_word_forward();
    assert_eq!(buf.text(), " wörld");
    assert_eq!(buf.cursor_char(), 0);
}

#[test]
fn delete_word_forward_yank_round_trip() {
    // The killed word lands in the kill buffer, so C-y brings it back.
    let mut buf = b("foo bar");
    buf.delete_word_forward();
    assert_eq!(buf.text(), " bar");
    buf.yank();
    assert_eq!(buf.text(), "foo bar");
}

#[test]
fn consecutive_word_kills_forward_accumulate() {
    // M-d M-d must ACCUMULATE both words in the kill ring (not overwrite),
    // so C-y brings back EVERYTHING killed in the run — the same append
    // precedent as consecutive C-k, respecting forward order.
    let mut buf = b("alpha beta gamma");
    buf.delete_word_forward(); // kills "alpha"
    assert_eq!(buf.kill_buffer(), "alpha");
    buf.delete_word_forward(); // kills " beta", APPENDS
    assert_eq!(buf.kill_buffer(), "alpha beta");
    assert_eq!(buf.text(), " gamma");
    buf.yank();
    assert_eq!(buf.text(), "alpha beta gamma");
}

#[test]
fn consecutive_word_kills_backward_accumulate() {
    // M-Backspace M-Backspace accumulates in reading order (a BACKWARD kill
    // PREPENDS), so C-y restores both words left-to-right rather than only
    // the last-killed one.
    let mut buf = b("alpha beta");
    buf.buffer_end();
    buf.delete_word_backward(); // kills "beta"
    assert_eq!(buf.kill_buffer(), "beta");
    buf.delete_word_backward(); // kills "alpha ", PREPENDS
    assert_eq!(buf.kill_buffer(), "alpha beta");
    assert_eq!(buf.text(), "");
    buf.yank();
    assert_eq!(buf.text(), "alpha beta");
}

#[test]
fn word_kill_then_move_starts_a_fresh_kill() {
    // A non-kill command between word-kills resets the kill flag, so the
    // next kill REPLACES the ring rather than accumulating (Emacs semantics).
    let mut buf = b("alpha beta gamma");
    buf.delete_word_forward(); // kills "alpha"
    assert_eq!(buf.kill_buffer(), "alpha");
    buf.forward_char(); // a motion resets the kill flag
    buf.delete_word_forward(); // fresh kill, REPLACES
    assert_eq!(buf.kill_buffer(), "beta");
}

#[test]
fn insert_newline_splits() {
    let mut buf = b("helloworld");
    for _ in 0..5 {
        buf.forward_char();
    }
    buf.insert_newline();
    assert_eq!(buf.text(), "hello\nworld");
    assert_eq!(buf.cursor_line_col(), (1, 0));
}

#[test]
fn tab_inserts_spaces_to_next_stop() {
    let mut buf = b("");
    buf.insert_tab();
    assert_eq!(buf.text(), "    "); // col 0 -> a full 4-wide tab
    let mut buf2 = b("ab");
    buf2.buffer_end(); // col 2
    buf2.insert_tab();
    assert_eq!(buf2.text(), "ab  "); // 2 spaces to reach the next stop
}

#[test]
fn tab_is_a_single_undo() {
    let mut buf = b("x");
    buf.buffer_end(); // col 1
    buf.insert_tab(); // 3 spaces to the next stop
    assert_eq!(buf.text(), "x   ");
    buf.undo();
    assert_eq!(buf.text(), "x");
}

#[test]
fn insert_text_lands_the_literal_string_at_the_cursor() {
    let mut buf = b("hello world");
    for _ in 0..5 {
        buf.forward_char(); // caret after "hello"
    }
    buf.insert_text(", 22/07/26,");
    assert_eq!(buf.text(), "hello, 22/07/26, world");
}

#[test]
fn insert_text_replaces_an_active_selection() {
    let mut buf = b("hello world");
    buf.select_range(0, 5); // "hello"
    buf.insert_text("goodbye");
    assert_eq!(buf.text(), "goodbye world");
    assert!(
        !buf.has_selection(),
        "the selection is consumed by the insert"
    );
}

/// "ONE undoable edit": Insert Date's whole contract. A single Cmd-Z
/// removes the ENTIRE inserted string in one step, regardless of length.
#[test]
fn insert_text_is_a_single_undo() {
    let mut buf = b("x");
    buf.buffer_end();
    buf.insert_text("2026-07-22");
    assert_eq!(buf.text(), "x2026-07-22");
    buf.undo();
    assert_eq!(buf.text(), "x");
}

/// The sealing discipline: `insert_text` never coalesces with adjacent
/// typing on EITHER side — undoing after typing more text still removes
/// ONLY the date, and typing right before it doesn't merge into its
/// group either (an earlier bug class `apply_format`'s own sealing
/// avoids — see its doc).
#[test]
fn insert_text_never_coalesces_with_adjacent_typing() {
    let mut buf = b("");
    buf.insert_char('a'); // ordinary typing, opens an Insert group
    buf.insert_text("2026-07-22"); // the discrete date insert
    buf.insert_char('b'); // ordinary typing resumes right after
    assert_eq!(buf.text(), "a2026-07-22b");
    buf.undo(); // removes ONLY "b"
    assert_eq!(buf.text(), "a2026-07-22");
    buf.undo(); // removes ONLY the date
    assert_eq!(buf.text(), "a");
    buf.undo(); // removes ONLY "a"
    assert_eq!(buf.text(), "");
}

#[test]
fn insert_text_empty_string_is_a_no_op() {
    let mut buf = b("hello");
    let before = buf.version();
    buf.insert_text("");
    assert_eq!(buf.text(), "hello");
    assert_eq!(buf.version(), before, "an empty insert records no edit");
}
