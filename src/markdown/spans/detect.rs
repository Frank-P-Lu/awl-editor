//! Per-line markdown detection: fence language labels, thematic breaks,
//! list-item shape, frontmatter extent, and word/reading-time counting.

use super::kind::MdKind;
use crate::markdown::ConcealKind;
use std::ops::Range;

/// A fenced code block's OPENING FENCE LINE's recognized LANGUAGE, from the raw
/// line text alone (e.g. `` "```rust" `` -> `Some(Lang::Rust)`, `` "```" `` or
/// `` "```made-up" `` -> `None`) — the render-only counterpart of [`break_kind`]:
/// `md_spans` only marks WHERE a fence lives ([`crate::markdown::ConcealKind::Fence`]),
/// never which glyph/label the render should show for it, so the render re-derives
/// the label straight from the line the SAME way `break_kind` re-derives the
/// thematic-break ornament. Delegates the actual name→language gate to
/// [`crate::syntax::Lang::from_info`] — THE SAME gate `spans` uses to decide
/// whether a fence's body gets `CodeSyntax` highlighting at all — so the drawn quiet
/// LABEL and the fence's own syntax highlighting can never disagree about which
/// language a fence names. Skips up to 3 leading indent spaces (CommonMark's fence-
/// indent allowance) then a run of 3+ matching `` ` `` or `~` fence characters;
/// anything short of that (no fence, an indented code block, an unmatched run) is
/// `None`. Pure + total.
pub fn fence_line_lang(line: &str) -> Option<crate::syntax::Lang> {
    let mut chars = line.chars();
    let mut rest = line;
    for _ in 0..3 {
        match chars.clone().next() {
            Some(' ') => {
                chars.next();
                rest = chars.as_str();
            }
            _ => break,
        }
    }
    let fence_char = rest.chars().next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }
    let run = rest.chars().take_while(|&c| c == fence_char).count();
    if run < 3 {
        return None;
    }
    let info = &rest[run..];
    crate::syntax::Lang::from_info(info)
}

/// True when `line` is a CommonMark THEMATIC BREAK: after up to 3 leading spaces, a
/// run of THREE-OR-MORE matching `-`, `_`, or `*`, separated/surrounded only by
/// spaces or tabs, and nothing else. This is the bare-text heuristic
/// [`crate::render::spans::md_line_scale`] uses to grow a break line's row to fit the
/// bigger ornament glyph — sized by the active world's
/// [`crate::theme::Theme::ornament_scale`] — the size counterpart of the leading-`#`
/// heading scan (a per-line grow that never needs the whole parse). Pure + total.
///
/// A qualifying dash underline is NOT a false positive: awl has no setext
/// headings, `spans` promotes it to a real `Rule` and this scan agrees. The
/// remaining gap: `---` in a fenced code block, per `md_line_scale`'s `confirmed_rule`.
pub fn is_thematic_break(line: &str) -> bool {
    let t = line.trim_matches(|c| c == ' ' || c == '\t');
    // The run char is the first non-space glyph; every non-space char must match it,
    // and there must be at least three of them.
    let mut run_char: Option<char> = None;
    let mut count = 0usize;
    for ch in t.chars() {
        match ch {
            ' ' | '\t' => {}
            '-' | '_' | '*' => {
                match run_char {
                    None => run_char = Some(ch),
                    Some(rc) if rc == ch => {}
                    Some(_) => return false, // mixed run chars => not a break
                }
                count += 1;
            }
            _ => return false, // any other glyph disqualifies the line
        }
    }
    count >= 3
}

/// The BYTE RANGE of a SETEXT H2 heading's own UNDERLINE, if that underline
/// independently qualifies as a thematic break ([`is_thematic_break`]) — the seam
/// [`crate::markdown::spans`]' `Tag::Heading` arm uses to promote `a\n---` to a real
/// `Rule` span without touching the title line. `range` is pulldown's own heading
/// range, which always ends on the underline's own line (its last line, whether the
/// title is one physical line or several); walking back to the last `\n` finds it
/// without re-deriving line boundaries from scratch. Leading indent on the
/// underline is excluded from the returned range (matching a real `Event::Rule`'s
/// own range), so [`crate::render::spans::add_rule_conceal_span`] conceals exactly
/// the dashes, never the indent. `None` when the underline is too short to be a
/// break — CommonMark accepts a bare single `-` for a setext H2, which stays plain
/// body text, unchanged.
pub(super) fn setext_break_range(text: &str, range: &Range<usize>) -> Option<Range<usize>> {
    let body = &text[range.clone()];
    let core = body.strip_suffix('\n').unwrap_or(body);
    let line_start = core.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &core[line_start..];
    if !is_thematic_break(line) {
        return None;
    }
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let start = range.start + line_start + indent;
    Some(start..range.start + core.len())
}

/// The number of leading-indent SPACES that make up ONE nesting level for a
/// markdown list. awl's list model is "every 2 spaces = one level" (see
/// [`ListItem::depth`]); this is the single place that ratio lives, shared by the
/// depth derivation (rendering) and the Tab/Shift-Tab indent step (editing). A tab
/// in the leading run counts as one unit toward `indent`, same as a space — see
/// [`list_item`].
pub const LIST_INDENT: usize = 2;

/// A detected markdown LIST ITEM on ONE line — the SHARED list-detection primitive
/// behind the depth-derived bullet GLYPH (rendering, `spans.rs`/`rects.rs`), the
/// Tab/Shift-Tab indent/outdent EDIT (`actions.rs`/`buffer`), AND the Enter-list-
/// continuation decision (`actions::edit::smart_newline_for`, which layers its own
/// checkbox-suffix and blockquote handling on top). Pure per-line scan (no full
/// parse), matching the per-line precedent of [`crate::render::spans::md_line_scale`]:
/// optional leading spaces OR TABS, then either an unordered marker (`-`/`*`/`+`) or
/// an ordered one (digits + `.`/`)`), then a REQUIRED single space. Byte offsets are
/// into the line; since a leading space and a leading tab are each one byte, `indent`
/// is both the leading-indent CHAR COUNT and the marker char's byte offset. See
/// [`list_item`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// Leading-indent (space or tab) count == the byte/char offset of the marker
    /// character.
    pub indent: usize,
    /// True for an ordered (`1.`) item; false for an unordered (`-`) bullet. Only
    /// unordered items get a depth-cycling bullet glyph — ordered keep their number.
    pub ordered: bool,
    /// Byte offset where the item's CONTENT begins (after the marker + its space).
    pub content: usize,
    /// True when the item has no content (just the marker + optional trailing
    /// whitespace) — the "empty item" whose Enter behavior is special: an ordered /
    /// blockquote item ENDS the block, an unordered bullet is PRESERVED with a plain
    /// line opened below (see `actions::edit::smart_newline_for`).
    pub empty: bool,
}

impl ListItem {
    /// Nesting depth = leading spaces / [`LIST_INDENT`] (every 2 spaces one level).
    pub fn depth(&self) -> usize {
        self.indent / LIST_INDENT
    }
}

/// Detect a markdown list item on `line` — the SHARED detection used by the bullet
/// glyph, its reveal-on-cursor concealment, the Tab/Shift-Tab indent edit, and the
/// Enter-list-continuation decision. Recognizes, after optional leading spaces OR
/// TABS (CommonMark's tab-as-indent — mixing the two is fine, each counts as one
/// unit of `indent`), an unordered marker (`-`/`*`/`+`) or an ordered one (a digit
/// run + `.`/`)`), each REQUIRING a single following space (so a bare `-` or
/// `12 monkeys` is NOT a list). Returns `None` for a non-list line. Pure.
pub fn list_item(line: &str) -> Option<ListItem> {
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    let indent = i;
    if i >= b.len() {
        return None;
    }
    let ordered = if matches!(b[i], b'-' | b'*' | b'+') {
        i += 1;
        false
    } else if b[i].is_ascii_digit() {
        let d0 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i > d0 && i < b.len() && (b[i] == b'.' || b[i] == b')') {
            i += 1;
            true
        } else {
            return None;
        }
    } else {
        return None;
    };
    // A real list item's marker is followed by a single space.
    if i >= b.len() || b[i] != b' ' {
        return None;
    }
    i += 1;
    let content = i;
    let empty = line[content..].chars().all(|c| c.is_whitespace());
    Some(ListItem {
        indent,
        ordered,
        content,
        empty,
    })
}

/// The document's FRONTMATTER block END byte, if `md_spans` carries a
/// `ConcealMarkup(Frontmatter)` span (always spanning byte `0..end`) — the
/// SHARED exclusion point for word-count/reading-time
/// ([`render::chrome::TextPipeline::word_count`](crate::render::TextPipeline)),
/// writing-nits (`render/rects.rs::ensure_nit_protos`), and — indirectly, via
/// its own [`crate::frontmatter::detect`] call — spell-check
/// (`spell::SpellChecker::misspellings_for`). `None` when the document has no
/// frontmatter block (the exclusion is then a no-op everywhere it's used).
pub fn frontmatter_end(md_spans: &[(Range<usize>, MdKind)]) -> Option<usize> {
    md_spans.iter().find_map(|(r, k)| {
        matches!(k, MdKind::ConcealMarkup(ConcealKind::Frontmatter)).then_some(r.end)
    })
}

/// The words-per-minute used to turn a WORD count into a reading-time estimate
/// (200 wpm, the conventional silent-prose rate) — the SINGLE place it's
/// defined; a character count paces itself via `CountUnit::pace_per_minute`.
pub const READING_WPM: usize = 200;

/// Count words in `text` — whitespace-separated tokens. A blank document is 0.
/// Pure + cheap; markup characters ride along with their word (`**bold**` counts as
/// one). No production caller: [`crate::card::figures::word_count`] is the one
/// every readout/streak-ledger surface now goes through (CJK-aware — see its
/// own doc comment for why a plain whitespace split undercounts unspaced
/// scripts). This stays as the documented naive baseline `card::figures`'
/// own token-counting doc comment contrasts itself against, pinned by its
/// own test.
#[cfg_attr(not(test), allow(dead_code))]
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Estimate reading time in WHOLE minutes for `words` at `pace_per_minute`,
/// rounded UP so any prose reads `1 min` minimum; 0 words → `0`.
pub fn reading_time_min(words: usize, pace_per_minute: usize) -> usize {
    if words == 0 {
        0
    } else {
        words.div_ceil(pace_per_minute)
    }
}

/// THE strikethrough ENGAGEMENT gate — awl's exactly-two-tilde rule, the ONE
/// place BOTH the live renderer ([`spans`]) and every exporter
/// ([`crate::export`]'s `model::parse`) decide whether a pulldown
/// `Tag::Strikethrough` span actually strikes. pulldown's GFM option ALSO parses
/// single-tilde `~x~`; awl deliberately keeps that INERT (the `==` exactly-two
/// precedent — the format command and the writer's-diff serializer both emit
/// `~~`), so prose like `2~3 weeks` or a stray `~word~` never silently strikes.
/// `src` is the whole `~~…~~` span slice (from the offset iterator); ENGAGED iff
/// its LEADING tilde run is EXACTLY two (GFM guarantees the closing run matches).
/// A `~~~` run is a block-level FENCE and never reaches an inline strikethrough
/// tag, so it can't reach this gate. Pure + total. Sharing this ONE owner is what
/// keeps the RENDER (inert `~x~` → no strike span) and the EXPORT (inert `~x~` →
/// no `<del>`/`<w:strike/>`) from disagreeing — the render/export strike
/// divergence this fixed.
pub fn strike_engaged(src: &str) -> bool {
    src.bytes().take_while(|&b| b == b'~').count() == 2
}
