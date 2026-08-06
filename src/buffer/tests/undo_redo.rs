use super::*;

// --- Undo / redo tests ------------------------------------------------

/// Type text then undo: the buffer returns to empty and the cursor home.
#[test]
fn undo_restores_empty_after_typing() {
    let mut buf = b("");
    for c in "abc".chars() {
        buf.insert_char(c);
    }
    assert_eq!(buf.text(), "abc");
    assert!(buf.can_undo());
    buf.undo();
    assert_eq!(buf.text(), "");
    assert_eq!(buf.cursor_char(), 0);
    assert!(!buf.can_undo());
}

/// Undo then redo round-trips back to the typed text + cursor.
#[test]
fn undo_then_redo_restores_text() {
    let mut buf = b("");
    for c in "abc".chars() {
        buf.insert_char(c);
    }
    buf.undo();
    assert_eq!(buf.text(), "");
    assert!(buf.can_redo());
    buf.redo();
    assert_eq!(buf.text(), "abc");
    assert_eq!(buf.cursor_char(), 3);
    assert!(!buf.can_redo());
}

/// Typing "hello world" then ONE undo removes the last word group ("world");
/// a SECOND undo removes "hello " (word + its trailing space).
#[test]
fn undo_coalesces_per_word() {
    let mut buf = b("");
    for c in "hello world".chars() {
        buf.insert_char(c);
    }
    assert_eq!(buf.text(), "hello world");
    buf.undo();
    assert_eq!(buf.text(), "hello ");
    buf.undo();
    assert_eq!(buf.text(), "");
    assert!(!buf.can_undo());
}

/// A space is an undo boundary on BOTH sides: each word is independently
/// undoable, and the space rides with the word before it.
#[test]
fn each_word_is_its_own_group() {
    let mut buf = b("");
    for c in "one two three".chars() {
        buf.insert_char(c);
    }
    buf.undo();
    assert_eq!(buf.text(), "one two ");
    buf.undo();
    assert_eq!(buf.text(), "one ");
    buf.undo();
    assert_eq!(buf.text(), "");
}

/// Replacing a selection then undo restores the ORIGINAL selected text (one
/// atomic step), and the buffer text is exactly as before the replace.
#[test]
fn undo_restores_replaced_selection() {
    let mut buf = b("hello world");
    buf.select_range(0, 5); // select "hello"
    buf.insert_char('X'); // replace with "X"
    assert_eq!(buf.text(), "X world");
    buf.undo();
    assert_eq!(buf.text(), "hello world");
    // Cursor restored to where it was before the edit.
    assert_eq!(buf.cursor_char(), 5);
    assert!(!buf.has_selection());
}

/// Yank-over-selection then undo restores the original selected text in one
/// step.
#[test]
fn undo_restores_yank_over_selection() {
    let mut buf = b("hello world");
    buf.kill = "ZZ".to_string();
    buf.select_range(0, 5); // select "hello"
    buf.yank();
    assert_eq!(buf.text(), "ZZ world");
    buf.undo();
    assert_eq!(buf.text(), "hello world");
}

/// A NEW edit after an undo clears the redo stack (linear history).
#[test]
fn new_edit_after_undo_clears_redo() {
    let mut buf = b("");
    for c in "abc".chars() {
        buf.insert_char(c);
    }
    buf.undo();
    assert!(buf.can_redo());
    buf.insert_char('Z');
    assert_eq!(buf.text(), "Z");
    assert!(!buf.can_redo());
    buf.redo(); // no-op now
    assert_eq!(buf.text(), "Z");
}

/// Sealing the group (a non-edit command) splits a same-direction run so each
/// side is undone separately even though both were insertions.
#[test]
fn seal_splits_insertion_run() {
    let mut buf = b("");
    for c in "abc".chars() {
        buf.insert_char(c);
    }
    buf.seal_undo_group(); // simulate a cursor motion between bursts
    for c in "def".chars() {
        buf.insert_char(c);
    }
    assert_eq!(buf.text(), "abcdef");
    buf.undo();
    assert_eq!(buf.text(), "abc");
    buf.undo();
    assert_eq!(buf.text(), "");
}

/// Direction flip (insert then delete) starts a new group: undoing the delete
/// does not also undo the preceding insertions.
#[test]
fn direction_flip_starts_new_group() {
    let mut buf = b("");
    for c in "abcd".chars() {
        buf.insert_char(c);
    }
    buf.delete_backward(); // delete 'd'
    buf.delete_backward(); // delete 'c'
    assert_eq!(buf.text(), "ab");
    buf.undo(); // undoes the deletion run -> "abcd"
    assert_eq!(buf.text(), "abcd");
    buf.undo(); // undoes the insertion -> ""
    assert_eq!(buf.text(), "");
}

/// A backspace run coalesces into one undo group.
#[test]
fn backspace_run_coalesces() {
    let mut buf = b("abcdef");
    buf.buffer_end();
    buf.delete_backward();
    buf.delete_backward();
    buf.delete_backward();
    assert_eq!(buf.text(), "abc");
    buf.undo();
    assert_eq!(buf.text(), "abcdef");
    assert_eq!(buf.cursor_char(), 6);
}

/// undo/redo bump the version counter so the view/spell layer re-syncs.
#[test]
fn undo_redo_bump_version() {
    let mut buf = b("");
    buf.insert_char('a');
    let v_after_type = buf.version();
    buf.undo();
    assert!(buf.version() > v_after_type);
    let v_after_undo = buf.version();
    buf.redo();
    assert!(buf.version() > v_after_undo);
}

#[test]
fn line_col_to_char_clamps_col() {
    let buf = b("hi\nlonger");
    // col past end of line 0 clamps to end of "hi" (char index 2)
    assert_eq!(buf.line_col_to_char(0, 99), 2);
    // line past end clamps to last line
    let (l, _) = buf.char_to_line_col(buf.line_col_to_char(99, 0));
    assert_eq!(l, 1);
}
