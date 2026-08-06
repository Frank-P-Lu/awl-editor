use super::super::*;
use super::*;

#[test]
fn is_url_recognizes_http_https_and_rejects_prose_and_paths() {
    // Real URLs.
    assert!(is_url("https://example.com"));
    assert!(is_url("http://example.com/the/essay?q=1#frag"));
    assert!(is_url("ftp://host/file"));
    // NOT URLs: plain prose, a bare path, an interior-space string, a bare
    // scheme with no host, an empty string, a multi-line clipboard.
    assert!(!is_url("the essay"));
    assert!(!is_url("https://has a space"));
    assert!(!is_url("/Users/alex/notes.md"));
    assert!(!is_url("./relative/path"));
    assert!(!is_url("http://")); // nothing after `://`
    assert!(!is_url("://nohost"));
    assert!(!is_url("example.com")); // no scheme
    assert!(!is_url(""));
    assert!(!is_url("https://a\nhttps://b"));
}

#[test]
fn paste_url_over_selection_in_markdown_wraps_as_one_undoable_link() {
    // Markdown buffer (no path => markdown). Select "the essay", paste a URL.
    let mut buf = b("the essay");
    buf.set_kill("https://example.com");
    buf.select_range(0, 9); // select the whole "the essay"
    buf.yank();
    assert_eq!(buf.text(), "[the essay](https://example.com)");
    // ONE undoable edit: Cmd-Z restores the original text (the selection).
    buf.undo();
    assert_eq!(buf.text(), "the essay");
    assert!(!buf.can_undo());
}

#[test]
fn paste_url_with_no_selection_is_a_normal_paste() {
    // No selection: URL is inserted verbatim, never wrapped.
    let mut buf = b("");
    buf.set_kill("https://example.com");
    buf.yank();
    assert_eq!(buf.text(), "https://example.com");
}

#[test]
fn paste_nonurl_over_selection_is_a_normal_replace() {
    let mut buf = b("the essay");
    buf.set_kill("some prose");
    buf.select_range(0, 9);
    buf.yank();
    assert_eq!(buf.text(), "some prose");
}

#[test]
fn paste_url_over_selection_in_code_buffer_is_a_normal_replace() {
    // A `.rs` buffer is NOT markdown: a URL over a selection stays a plain
    // replace — never `[x](url)` in code.
    let mut buf = b("the essay");
    buf.set_path(std::path::PathBuf::from("/tmp/x.rs"));
    assert!(!buf.is_markdown());
    buf.set_kill("https://example.com");
    buf.select_range(0, 9);
    buf.yank();
    assert_eq!(buf.text(), "https://example.com");
}

#[test]
fn word_bounds_on_word_char() {
    let buf = b("foo bar.baz");
    // idx 5 is inside "bar"
    assert_eq!(buf.word_bounds(5), (4, 7));
    // idx 0 inside "foo"
    assert_eq!(buf.word_bounds(0), (0, 3));
    // idx at the space (3) -> the run of non-word chars [3,4)
    assert_eq!(buf.word_bounds(3), (3, 4));
}

#[test]
fn line_bounds_includes_newline() {
    let buf = b("aaa\nbbb\nccc");
    // line 1 ("bbb") spans chars [4,8) including its trailing newline
    assert_eq!(buf.line_bounds(5), (4, 8));
    // last line has no trailing newline
    assert_eq!(buf.line_bounds(9), (8, 11));
}

#[test]
fn line_col_to_char_roundtrips() {
    let buf = b("hello\nworld\n!");
    for &idx in &[0usize, 3, 5, 6, 9, 11, 12] {
        let (l, c) = buf.char_to_line_col(idx);
        assert_eq!(buf.line_col_to_char(l, c), idx, "roundtrip at {idx}");
    }
}

// --- Click / drag selection-collapse tests ----------------------------
// These model the exact buffer API sequence the app's mouse handlers and
// motion-extend path use, so a plain click can never leave a phantom
// selection that a later bare motion would extend.

/// A single click places the cursor and (to support a future drag) sets the
/// anchor at the same index. The press-time state has NO visible selection
/// (anchor == cursor), so the release-time collapse must clear the anchor,
/// after which a bare motion just moves the cursor without selecting.
#[test]
fn plain_click_then_motion_does_not_select() {
    let mut buf = b("line0\nline1\nline2");
    buf.buffer_end(); // pretend we clicked near the end
    let idx = buf.cursor_char();
    // on_press, single click:
    buf.set_cursor(idx);
    buf.clear_mark();
    buf.set_anchor(idx); // anchor == cursor: no visible selection yet
    assert!(!buf.has_selection());
    // Released with no drag: the app collapses the lingering anchor when
    // has_selection() is false.
    if !buf.has_selection() {
        buf.clear_mark();
    }
    assert_eq!(buf.anchor_char(), None, "plain click must clear the anchor");
    // A bare motion (e.g. C-p / PreviousLine) must NOT create a selection.
    buf.previous_line();
    assert!(
        !buf.has_selection(),
        "bare motion after plain click selected"
    );
    assert_eq!(buf.selection_range(), None);
}

/// A click-DRAG (cursor moves away from the press-time anchor) leaves a real
/// selection, so the release-time collapse must preserve it.
#[test]
fn click_drag_still_selects() {
    let mut buf = b("hello world");
    // on_press at 0:
    buf.set_cursor(0);
    buf.clear_mark();
    buf.set_anchor(0);
    // on_drag (Char granularity) to idx 5:
    buf.set_cursor(5);
    assert!(buf.has_selection());
    // Released: has_selection() is true -> anchor preserved.
    if !buf.has_selection() {
        buf.clear_mark();
    }
    assert!(buf.has_selection(), "click-drag selection was dropped");
    assert_eq!(buf.selection_range(), Some((0, 5)));
}

/// An explicit mark (C-Space / SetMark) followed by a motion must still
/// extend the region (Emacs `mg` sticky behavior) — the click-collapse fix
/// only touches the mouse-release path, never the keyboard mark path.
#[test]
fn mark_then_motion_still_extends_after_click_fix() {
    let mut buf = b("hello world");
    // simulate a prior plain click leaving a clean (no-anchor) state:
    buf.set_cursor(0);
    buf.clear_mark();
    assert_eq!(buf.anchor_char(), None);
    // C-Space:
    buf.set_mark();
    // motion extends:
    for _ in 0..5 {
        buf.forward_char();
    }
    assert!(buf.has_selection());
    assert_eq!(buf.selection_range(), Some((0, 5)));
}
