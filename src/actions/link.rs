//! LINKS V2 — the pure logic behind Cmd-K (`Action::InsertLink`): deciding which
//! [`LinkEditMode`] a press lands in (from buffer state alone), and building the
//! final whole-buffer text once the URL minibuffer commits. Mirrors
//! `actions/format.rs`'s shape (pure transform in, [`FormatResult`]-style out,
//! applied as ONE atomic edit via `Buffer::apply_format`) — the same "same
//! behavior, same code" reasoning: an insert-link edit and a format toggle are
//! both "replace a byte range with a new wrapped/inserted string, then land the
//! cursor sensibly", so they share the apply primitive even though their pure
//! transforms live in separate, purpose-named modules (mirroring the block/inline
//! split within `format.rs` itself).

use super::format;
use super::*;
use crate::overlay::{LinkEditMode, OverlayState};

/// Char index of byte offset `byte` into `text` — the one conversion seam this
/// module needs (`markdown::link_at_full` returns DOCUMENT byte offsets, matching
/// pulldown's own coordinate space; `Buffer::apply_format`/`replace_char_range`
/// want CHAR indices). O(byte) but called at most once per Cmd-K press, never
/// per-frame.
fn byte_to_char(text: &str, byte: usize) -> usize {
    text.as_bytes()[..byte.min(text.len())]
        .iter()
        .filter(|&&b| (b & 0xC0) != 0x80) // count UTF-8 lead bytes only
        .count()
}

/// LINKS V2: decide the [`LinkEditMode`] + prefill URL for a fresh Cmd-K press,
/// purely from `text` + the selection/cursor CHAR state (no buffer mutation, so
/// this is safe to call speculatively). `kill_head` is the clipboard/kill-ring
/// head (`Buffer::kill_buffer`) — used as the prefill IFF it looks like a URL
/// ([`crate::buffer::is_url`]); the nice-touch "you probably want to paste this"
/// seed the task asked for, flagged as a taste call (a URL sitting in the kill
/// ring might be stale / not what the user means to link to — logged for live
/// review, not hidden).
///
/// Three cases, in priority order:
///   1. An ACTIVE SELECTION wraps that exact span: `[selection](url)`.
///   2. No selection, but the caret sits INSIDE an existing link
///      ([`crate::markdown::link_at_full`]): EDIT mode — re-prompt with that
///      link's own current URL (not the kill head — editing an existing link
///      should show what's there, not overwrite the prefill with something
///      unrelated), rewriting the same range on commit.
///   3. Neither: insert empty `[](url)` markup at the caret.
pub(super) fn plan(
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
    kill_head: &str,
) -> (LinkEditMode, String) {
    let (s, e, has_sel) = crate::actions::format::sel_range(anchor, cursor);
    let url_prefill = || {
        if crate::buffer::is_url(kill_head) {
            kill_head.to_string()
        } else {
            String::new()
        }
    };
    if has_sel {
        let wrapped: String = text.chars().skip(s).take(e - s).collect();
        return (
            LinkEditMode::WithText {
                start: s,
                end: e,
                text: wrapped,
            },
            url_prefill(),
        );
    }
    let byte = text
        .char_indices()
        .nth(cursor)
        .map(|(b, _)| b)
        .unwrap_or(text.len());
    if let Some(link) = crate::markdown::link_at_full(text, byte) {
        let start = byte_to_char(text, link.start);
        let end = byte_to_char(text, link.end);
        return (
            LinkEditMode::Existing {
                start,
                end,
                source_text: link.link_text,
            },
            link.url,
        );
    }
    (LinkEditMode::Empty { at: cursor }, url_prefill())
}

/// LINKS V2 COMMIT: build the new whole-buffer text + cursor/anchor to restore,
/// given the (already-decided) `mode` and the typed `url`. Mirrors
/// `format::FormatResult`'s exact shape so the caller applies it the same way
/// (`Buffer::apply_format`) — an empty `url` is still applied verbatim (an empty
/// `[text]()`/`[]()"` is a harmless, correctable markdown oddity, not worth a
/// silent-cancel special case the user didn't ask for).
pub(super) fn commit(text: &str, mode: &LinkEditMode, url: &str) -> format::FormatResult {
    let chars: Vec<char> = text.chars().collect();
    match mode {
        LinkEditMode::WithText {
            start,
            end,
            text: inner,
        } => {
            let start = (*start).min(chars.len());
            let end = (*end).min(chars.len()).max(start);
            let mut out = String::new();
            out.extend(&chars[..start]);
            let link = serialized_literal(inner, url);
            out.push_str(&link);
            out.extend(&chars[end..]);
            let cursor = start + link.chars().count();
            format::FormatResult {
                text: out,
                anchor: None,
                cursor,
            }
        }
        LinkEditMode::Existing {
            start,
            end,
            source_text,
        } => {
            let start = (*start).min(chars.len());
            let end = (*end).min(chars.len()).max(start);
            let mut out = String::new();
            out.extend(&chars[..start]);
            let link = serialized_source(source_text, url);
            out.push_str(&link);
            out.extend(&chars[end..]);
            let cursor = start + link.chars().count();
            format::FormatResult {
                text: out,
                anchor: None,
                cursor,
            }
        }
        LinkEditMode::Empty { at } => {
            let at = (*at).min(chars.len());
            let mut out = String::new();
            out.extend(&chars[..at]);
            out.push_str(&serialized_literal("", url));
            out.extend(&chars[at..]);
            // Caret lands BETWEEN the brackets, ready to type the link text.
            format::FormatResult {
                text: out,
                anchor: None,
                cursor: at + 1,
            }
        }
    }
}

/// The one serializer family for a Markdown link. Literal prose is escaped;
/// source already parsed from an existing link is carried byte-for-byte.
fn serialized_literal(text: &str, url: &str) -> String {
    let label = text
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]");
    serialized_source(&label, url)
}

fn serialized_source(source_text: &str, url: &str) -> String {
    let destination = url
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");
    format!("[{source_text}]({destination})")
}

/// Pure paste conversion. The caller has already established that this is a
/// Markdown buffer and that `url` has the conservative URL shape. Context is
/// delegated to markdown's parser-backed owner: code and existing-link source
/// remain ordinary literal paste.
pub(crate) fn paste_over_selection(
    text: &str,
    start: usize,
    end: usize,
    url: &str,
) -> Option<(String, Option<usize>, usize)> {
    if start >= end || !crate::markdown::link_paste_is_safe(text, start, end) {
        return None;
    }
    let selected: String = text.chars().skip(start).take(end - start).collect();
    let replacement = serialized_literal(&selected, url);
    Some((
        replace_chars(text, start, end, &replacement),
        None,
        start + replacement.chars().count(),
    ))
}

fn replace_chars(text: &str, start: usize, end: usize, replacement: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len()).max(start);
    let mut out = String::new();
    out.extend(&chars[..start]);
    out.push_str(replacement);
    out.extend(&chars[end..]);
    out
}

/// `Action::InsertLink` dispatch: markdown buffers only (a calm no-op elsewhere,
/// matching the formatting toggles' own availability honesty), summon the
/// minibuffer via [`plan`]. The actual edit happens on Enter, inside the modal
/// intercept (`actions/overlay_nav.rs`) — see [`commit`].
pub(super) fn open_insert_link(ctx: &mut ActionCtx) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    let kill = ctx.buffer.kill_buffer().to_string();
    let (mode, prefill) = plan(&text, anchor, cursor, &kill);
    ctx.journey
        .enter(Some(OverlayState::new_link_edit(prefill, mode)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::overlay::OverlayKind;

    // --- plan(): mode + prefill decision ------------------------------------

    #[test]
    fn plan_with_selection_wraps_it() {
        let (mode, prefill) = plan("hello world", Some(0), 5, "");
        assert_eq!(
            mode,
            LinkEditMode::WithText {
                start: 0,
                end: 5,
                text: "hello".to_string()
            }
        );
        assert_eq!(prefill, "");
    }

    #[test]
    fn plan_with_selection_prefills_from_a_url_looking_kill_head() {
        let (_, prefill) = plan("hello world", Some(0), 5, "https://example.com");
        assert_eq!(prefill, "https://example.com");
    }

    #[test]
    fn link_serializer_escapes_source_without_changing_visible_words() {
        let result = commit(
            "cat] (\\)",
            &LinkEditMode::WithText {
                start: 0,
                end: 8,
                text: "cat] (\\)".to_string(),
            },
            "https://example.com/a_(b)\\c",
        );
        assert_eq!(
            result.text,
            "[cat\\] (\\\\)](https://example.com/a_\\(b\\)\\\\c)"
        );
    }

    #[test]
    fn plan_with_selection_ignores_a_non_url_kill_head() {
        let (_, prefill) = plan("hello world", Some(0), 5, "just some prose");
        assert_eq!(prefill, "");
    }

    #[test]
    fn plan_with_no_selection_and_no_link_inserts_empty() {
        let (mode, prefill) = plan("hello world", None, 5, "");
        assert_eq!(mode, LinkEditMode::Empty { at: 5 });
        assert_eq!(prefill, "");
    }

    #[test]
    fn plan_with_no_selection_and_no_link_prefills_from_url_kill_head() {
        let (mode, prefill) = plan("hello world", None, 5, "https://x.test/y");
        assert_eq!(mode, LinkEditMode::Empty { at: 5 });
        assert_eq!(prefill, "https://x.test/y");
    }

    #[test]
    fn plan_with_caret_inside_an_existing_link_is_edit_mode() {
        let text = "see [the text](https://old.example/path) here";
        // Caret inside "the text".
        let cursor = text.find("text").unwrap();
        let (mode, prefill) = plan(text, None, cursor, "https://irrelevant.kill.head");
        let start = text.find('[').unwrap();
        let end = text.find(')').unwrap() + 1;
        assert_eq!(
            mode,
            LinkEditMode::Existing {
                start,
                end,
                source_text: "the text".to_string()
            }
        );
        // Prefill is the EXISTING link's URL, never the kill head.
        assert_eq!(prefill, "https://old.example/path");
    }

    #[test]
    fn plan_with_caret_outside_any_link_is_not_edit_mode() {
        let text = "before [link](https://x.test) after";
        let cursor = 3; // inside "before", nowhere near the link
        let (mode, _) = plan(text, None, cursor, "");
        assert_eq!(mode, LinkEditMode::Empty { at: 3 });
    }

    // --- commit(): the pure text build ---------------------------------------

    #[test]
    fn commit_with_text_wraps_as_markdown_link() {
        let text = "hello world";
        let mode = LinkEditMode::WithText {
            start: 0,
            end: 5,
            text: "hello".to_string(),
        };
        let r = commit(text, &mode, "https://example.com");
        assert_eq!(r.text, "[hello](https://example.com) world");
        assert_eq!(r.anchor, None);
        // Cursor lands right after the closing paren.
        assert_eq!(&r.text[..r.cursor], "[hello](https://example.com)");
    }

    #[test]
    fn commit_empty_inserts_brackets_with_caret_between_them() {
        let text = "hello world";
        let mode = LinkEditMode::Empty { at: 5 };
        let r = commit(text, &mode, "https://example.com");
        assert_eq!(r.text, "hello[](https://example.com) world");
        // Caret sits BETWEEN the brackets, ready to type the link text.
        assert_eq!(r.cursor, 6);
        assert_eq!(&r.text[5..7], "[]");
    }

    #[test]
    fn commit_edit_mode_rewrites_the_url_preserving_the_link_text() {
        let text = "see [the text](https://old.example/path) here";
        let start = text.find('[').unwrap();
        let end = text.find(')').unwrap() + 1;
        let mode = LinkEditMode::Existing {
            start,
            end,
            source_text: "the text".to_string(),
        };
        let r = commit(text, &mode, "https://new.example/path");
        assert_eq!(r.text, "see [the text](https://new.example/path) here");
    }

    #[test]
    fn existing_link_rewrite_preserves_already_escaped_raw_label_source() {
        let text = r"see [a\]b\\c](https://old.example) here";
        let cursor = text.find("b\\").unwrap();
        let (mode, _) = plan(text, None, cursor, "");
        let LinkEditMode::Existing {
            ref source_text, ..
        } = mode
        else {
            panic!("parsed existing link must retain a source-labelled edit mode");
        };
        assert_eq!(source_text, r"a\]b\\c");

        let result = commit(text, &mode, "https://new.example");
        assert_eq!(
            result.text, r"see [a\]b\\c](https://new.example) here",
            "MUTATION TRAP: existing raw escapes are never escaped a second time"
        );
        assert_eq!(
            result.cursor,
            result.text.find(" here").unwrap(),
            "caret lands after the rewritten link"
        );
    }

    // --- open_insert_link(): the full apply_transition dispatch ---------------------

    fn drive_open(text: &str, anchor: Option<usize>, cursor: usize) -> crate::overlay::Journey {
        let mut buffer = Buffer::from_str(text);
        buffer.set_cursor(cursor);
        if let Some(a) = anchor {
            buffer.select_range(a, cursor);
        }
        let mut shift_selecting = false;
        let mut zoom = 1.0;
        let mut search = None;
        let mut journey = crate::overlay::Journey::default();
        let mut make_overlay = |_k: OverlayKind| -> Option<crate::overlay::OverlayState> { None };
        let mut browse_to =
            |_k: OverlayKind, _r: Option<String>| -> Option<crate::overlay::OverlayState> { None };
        let mut ctx = ActionCtx {
            buffer: &mut buffer,
            shift_selecting: &mut shift_selecting,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: None,
        };
        apply_transition(&mut ctx, &Action::InsertLink, false).primary();
        journey
    }

    #[test]
    fn insert_link_opens_the_minibuffer_on_a_markdown_buffer() {
        let journey = drive_open("hello world", None, 5);
        let ov = journey.card().expect("overlay must open");
        assert_eq!(ov.kind, OverlayKind::InsertLink);
        assert!(ov.link_edit.is_some());
    }

    #[test]
    fn insert_link_is_a_calm_no_op_on_a_non_markdown_buffer() {
        let mut buffer = Buffer::from_str("fn main() {}");
        buffer.set_path(std::path::PathBuf::from("x.rs"));
        buffer.set_cursor(3);
        assert!(!buffer.is_markdown());
        let mut shift_selecting = false;
        let mut zoom = 1.0;
        let mut search = None;
        let mut journey = crate::overlay::Journey::default();
        let mut make_overlay = |_k: OverlayKind| -> Option<crate::overlay::OverlayState> { None };
        let mut browse_to =
            |_k: OverlayKind, _r: Option<String>| -> Option<crate::overlay::OverlayState> { None };
        let mut ctx = ActionCtx {
            buffer: &mut buffer,
            shift_selecting: &mut shift_selecting,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: None,
        };
        apply_transition(&mut ctx, &Action::InsertLink, false).primary();
        assert!(
            journey.card().is_none(),
            "a non-markdown buffer must not open the link minibuffer"
        );
    }

    // --- byte_to_char ----------------------------------------------------------

    #[test]
    fn byte_to_char_handles_multibyte_prefixes() {
        // "héllo" — 'é' is 2 bytes, so byte offset 3 (right after 'é') is char 2.
        let text = "héllo";
        assert_eq!(byte_to_char(text, 0), 0);
        assert_eq!(byte_to_char(text, 3), 2);
        assert_eq!(byte_to_char(text, text.len()), text.chars().count());
    }
}
