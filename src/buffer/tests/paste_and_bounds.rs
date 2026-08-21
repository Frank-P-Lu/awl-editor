use super::*;

#[test]
fn is_url_enrols_http_https_mailto_and_rejects_other_paste_shapes() {
    // Real URLs.
    assert!(is_url("https://example.com"));
    assert!(is_url("http://example.com/the/essay?q=1#frag"));
    assert!(is_url("mailto:writer@example.com"));
    assert!(is_url("HTTPS://EXAMPLE.COM/path"));
    assert!(is_url("https://[2001:db8::1]:8443/path"));
    assert!(is_url("MAILTO:Writer@example.com?subject=hello"));
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
    assert!(!is_url("ftp://host/file"));
    assert!(!is_url("mailto:writer"));
    for malformed in [
        "https:///path",
        "https://?query",
        "https://#fragment",
        "https://---",
        "https://:443/path",
        "https://user@:443/path",
        "https://[::1]bad/path",
        "https://[::1]:bad/path",
        "mailto:@",
        "mailto:writer@",
        "mailto:writer@@example.com",
    ] {
        assert!(
            !is_url(malformed),
            "{malformed:?} has no plausible authority"
        );
    }
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
    buf.redo();
    assert_eq!(buf.text(), "[the essay](https://example.com)");
}

#[test]
fn paste_url_sweeps_unicode_mailto_direction_and_markdown_escaping() {
    let mut buf = b("before \u{732b}] (\\) after");
    // Reverse selection direction must use the same source range.
    buf.select_range(13, 7);
    buf.set_kill("mailto:writer@example.com");
    buf.yank();
    assert_eq!(
        buf.text(),
        "before [\u{732b}\\] (\\\\)](mailto:writer@example.com) after"
    );

    let mut destination = b("words");
    destination.select_range(0, 5);
    destination.set_kill("https://example.com/a_(b)\\c");
    destination.yank();
    assert_eq!(
        destination.text(),
        "[words](https://example.com/a_\\(b\\)\\\\c)"
    );
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
    for replacement in ["some prose", "./relative/path", "https://a\nhttps://b"] {
        let mut buf = b("the essay");
        buf.set_kill(replacement);
        buf.select_range(0, 9);
        buf.yank();
        assert_eq!(
            buf.text(),
            replacement,
            "{replacement:?} stays byte-for-byte literal"
        );
    }
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
fn paste_url_stays_literal_in_markdown_code_and_existing_link_source() {
    for (text, start, end, expected) in [
        ("`words`", 0, 1, "https://new.examplewords`"),
        ("`words`", 1, 6, "`https://new.example`"),
        (
            "```rust\nwords\n```",
            8,
            13,
            "```rust\nhttps://new.example\n```",
        ),
        (
            "[words](https://old.example)",
            1,
            6,
            "[https://new.example](https://old.example)",
        ),
    ] {
        let mut buf = b(text);
        buf.select_range(start, end);
        buf.set_kill("https://new.example");
        buf.yank();
        assert_eq!(
            buf.text(),
            expected,
            "{text:?} must remain a literal paste context"
        );
    }
}

#[test]
fn paste_url_stays_literal_in_image_source_and_frontmatter() {
    for (text, selected, expected) in [
        ("![alt](old.png)", "alt", "![https://new.example](old.png)"),
        (
            "---\ntitle: words\n---\nbody",
            "words",
            "---\ntitle: https://new.example\n---\nbody",
        ),
    ] {
        let start_byte = text.find(selected).unwrap();
        let start = text[..start_byte].chars().count();
        let end = start + selected.chars().count();
        let mut buf = b(text);
        buf.select_range(start, end);
        buf.set_kill("https://new.example");
        buf.yank();
        assert_eq!(
            buf.text(),
            expected,
            "MUTATION TRAP: parser-owned non-prose source must take literal paste"
        );
    }
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

#[cfg(target_os = "macos")]
#[test]
fn prose_word_bounds_use_the_macos_linguistic_token() {
    let buf = b("大幅に構成が変わっており");
    assert_eq!(
        buf.word_bounds(3),
        (3, 5),
        "macOS prose selects 構成, not the whole unspaced phrase"
    );
}

#[test]
fn portable_prose_cjk_fallback_is_one_grapheme() {
    let text = "前半。後半";
    assert_eq!(
        crate::word_selection::portable_cjk_grapheme_bounds(0, text.chars().count(), |i| text
            .chars()
            .nth(i)
            .unwrap(),),
        Some((0, 1)),
        "an unspaced CJK run does not become one editor-style word"
    );
}

#[test]
fn portable_editor_words_keep_english_and_code_snake_case() {
    let prose = b("alpha beta");
    assert_eq!(prose.editor_word_bounds(1), (0, 5));
    assert_eq!(prose.editor_word_bounds(7), (6, 10));

    let mut code = b("snake_case next");
    code.set_path(std::path::PathBuf::from("example.rs"));
    assert_eq!(code.word_bounds(5), (0, 10));
    assert_eq!(code.word_bounds(12), (11, 15));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_prose_linguistic_selection_covers_real_writing_shapes() {
    fn prose(text: &str, idx: usize) -> (usize, usize) {
        b(text).word_bounds(idx)
    }

    assert_eq!(prose("don't stop", 3), (0, 5), "ASCII apostrophe");
    assert_eq!(prose("don’t stop", 3), (0, 5), "curly apostrophe");
    assert_eq!(prose("state-of-the-art", 6), (6, 8), "hyphenated prose");
    assert_eq!(
        prose("https://example.com/a-b", 10),
        (8, 15),
        "URL component"
    );
    assert_eq!(prose("**bold**", 3), (2, 6), "Markdown content");
    assert_eq!(prose("**bold**", 0), (0, 2), "Markdown punctuation");
    assert_eq!(prose("hello...world", 5), (5, 8), "punctuation run");
    assert_eq!(prose("👩🏽‍💻 next", 0), (0, 4), "emoji cluster");
    assert_eq!(prose("cafe\u{301} next", 2), (0, 5), "combining cluster");
    assert_eq!(
        prose("Englishと日本語mixed", 8),
        (8, 10),
        "mixed-script Japanese compound"
    );

    let mut plain_text = b("大幅に構成が変わっており");
    plain_text.set_path(std::path::PathBuf::from("essay.txt"));
    assert_eq!(plain_text.word_bounds(3), (3, 5), "plain-text prose");

    let mut code = b("大幅に構成が変わっており snake_case");
    code.set_path(std::path::PathBuf::from("example.rs"));
    assert_eq!(code.word_bounds(3), (0, 12), "code keeps editor CJK run");
    assert_eq!(code.word_bounds(18), (13, 23), "code keeps snake_case");
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
