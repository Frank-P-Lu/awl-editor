//! Per-construct span pushers: the small `spans()` helpers that mark
//! heading/link/quote/list/task/highlight/inline-code markup.

use super::detect::{bare_url_ranges, bare_url_split, list_item};
use super::kind::MdKind;
use crate::markdown::ConcealKind;
use std::ops::Range;

/// Dim the `n`-byte inline delimiters at each end of `range` (`*`/`_` → n=1,
/// `**`/`__`/`~~` → n=2). No-op if the range is too short to hold both. WYSIWYG-
/// concealable as `ck` ([`ConcealKind::Emphasis`] for bold/italic,
/// [`ConcealKind::Strikethrough`] for `~~`): the delimiters hide off the caret's
/// line, leaving the styled content alone.
pub(super) fn push_delim(
    out: &mut Vec<(Range<usize>, MdKind)>,
    range: &Range<usize>,
    n: usize,
    ck: ConcealKind,
) {
    if range.end.saturating_sub(range.start) >= 2 * n {
        let k = MdKind::ConcealMarkup(ck);
        out.push((range.start..range.start + n, k));
        out.push((range.end - n..range.end, k));
    }
}

/// Dim a heading's leading `#`s (+ the space after), and any ATX closing `#`s.
/// WYSIWYG-concealable ([`ConcealKind::Heading`]): both marker runs hide off the
/// caret's line, leaving the sized title alone.
pub(super) fn push_heading_markers(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let k = MdKind::ConcealMarkup(ConcealKind::Heading);
    let s = &text[range.clone()];
    let b = s.as_bytes();
    // Leading: optional indent whitespace, the `#` run, then the spaces after.
    let mut i = 0;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    let mut h = i;
    while h < b.len() && b[h] == b'#' {
        h += 1;
    }
    if h > i {
        // Include trailing spaces between the hashes and the title.
        let mut j = h;
        while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
            j += 1;
        }
        out.push((range.start..range.start + j, k));
    }
    // Trailing ATX close: spaces then a `#` run at the very end of the line.
    let mut e = b.len();
    while e > 0 && (b[e - 1] == b' ' || b[e - 1] == b'\t' || b[e - 1] == b'\n') {
        e -= 1;
    }
    let mut c = e;
    while c > 0 && b[c - 1] == b'#' {
        c -= 1;
    }
    if c < e {
        // Pull in a space before the closing hashes if present.
        let mut s0 = c;
        while s0 > 0 && (b[s0 - 1] == b' ' || b[s0 - 1] == b'\t') {
            s0 -= 1;
        }
        out.push((range.start + s0..range.start + e, k));
    }
}

/// Conceal a link's MARKUP plumbing — the opening `[` and the whole `](url)`
/// tail — as WYSIWYG-concealable [`ConcealKind::Link`] spans, leaving the visible
/// link TEXT untouched (the inner `Event::Text` styles it `LinkText`, full content
/// ink). `range` is the whole `[text](url)` reference. The text/plumbing split is
/// the FIRST `](` in the source: everything before it is `[text`, everything from
/// it on is the `](url…)` tail. Off the caret's line the two plumbing runs hide to
/// zero-width so only the text shows; on the line they reveal for editing.
///
/// A reference-style (`[text][ref]` / `[text]`) or otherwise malformed link has no
/// `](`, so it falls back to a single plain, NON-concealing [`MdKind::Markup`] span
/// over the whole range — byte-identical to the pre-WYSIWYG-links rendering (dim
/// brackets, content-ink text), never a mis-conceal.
pub(super) fn push_link_markers(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let s = &text[range.clone()];
    // The `](` separating the visible text from the destination. Requires the
    // source to actually open with `[` (an inline link always does).
    if s.starts_with('[')
        && let Some(rel) = s.find("](")
    {
        let k = MdKind::ConcealMarkup(ConcealKind::Link);
        // Opening `[`.
        out.push((range.start..range.start + 1, k));
        // The `](url…)` tail — closing bracket, parens, destination + title.
        out.push((range.start + rel..range.end, k));
        return;
    }
    // Reference / malformed: dim the whole thing, no conceal (as before).
    out.push((range.clone(), MdKind::Markup));
}

/// Dim + WYSIWYG-conceal the leading `>` quote markers (+ a following space) on
/// every line of a blockquote range, including nested `>>`. Each line's whole
/// marker run is ONE [`ConcealKind::Blockquote`] span (LINE-scoped): dim like plain
/// `Markup` with WYSIWYG off, concealed to zero-width off the caret's line with it
/// on. Nested markers on one line share that line's run, so they conceal together.
/// The block's affordance off-caret is the renderer's margin-hung pull-quote mark.
pub(super) fn push_quote_markers(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let s = &text[range.clone()];
    let b = s.as_bytes();
    let mut line_start = 0usize;
    let mut i = 0usize;
    while i <= b.len() {
        if i == b.len() || b[i] == b'\n' {
            // Scan this line's leading `[ \t]*(> ?)+` marker run.
            let mut k = line_start;
            while k < i && (b[k] == b' ' || b[k] == b'\t') {
                k += 1;
            }
            let mut last = k;
            while k < i && b[k] == b'>' {
                k += 1;
                if k < i && (b[k] == b' ' || b[k] == b'\t') {
                    k += 1;
                }
                last = k;
            }
            if last > line_start {
                out.push((
                    range.start + line_start..range.start + last,
                    MdKind::ConcealMarkup(ConcealKind::Blockquote),
                ));
            }
            line_start = i + 1;
        }
        i += 1;
    }
}

/// Dim a list item's leading marker (`-`/`*`/`+` or `1.`/`1)`), plus its INDENT and
/// its trailing space.
///
/// pulldown's `Tag::Item` `range` starts at the MARKER CHARACTER itself, NEVER at
/// the line's own start — so for a NESTED item (`  - text`, `it.indent > 0`) the
/// leading indent spaces sit BEFORE `range.start` and are invisible to a scan over
/// `text[range.clone()]` alone. A top-level item has `indent == 0`, so this was
/// masked there (the marker IS the line start) — only nesting exposed the gap: the
/// nested marker's own span silently excluded its 2(+)-space indent, leaving those
/// bytes with NO markdown span at all (the "space missing" mis-highlight the notes
/// named — harmless for bare whitespace on its own, but a real, general nested-list
/// gap, not an image-specific one).
///
/// FIX: re-derive the marker from the item's own LINE (walk back to the preceding
/// `\n`, or byte 0), through the SAME shared [`list_item`] line-scanner the bullet
/// reveal-conceal / Tab-Shift-Tab indent / depth-cycle machinery already use — one
/// owner, so the marker's span can never again disagree with what those already
/// treat as "the marker". The pushed span now covers the WHOLE prefix
/// (indent + marker + its required space), matching `list_item`'s own
/// `0..content` shape exactly.
pub(super) fn push_list_marker(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let line_start = text[..range.start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[range.start..]
        .find('\n')
        .map_or(text.len(), |i| range.start + i);
    let line = &text[line_start..line_end];
    let Some(it) = list_item(line) else {
        return; // not a recognizable marker line (shouldn't happen for a real Item)
    };
    // From the LINE's own start (byte 0, covering any indent) through `content`
    // (past the marker + its required space) — `it.indent` is only the marker
    // CHARACTER's own offset, not where the span should START.
    if it.content > 0 {
        out.push((line_start..line_start + it.content, MdKind::ListMarker));
    }
}

/// Style a task checkbox: the `[ ]`/`[x]` marker `range` (from the `TaskListMarker`
/// event) plus the single space that follows it, so the whole checkbox + gap reads
/// as one unit. `checked` selects the open/closed [`MdKind::Task`] role.
pub(super) fn push_task_marker(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
    checked: bool,
) {
    let mut end = range.end;
    let b = text.as_bytes();
    if end < b.len() && (b[end] == b' ' || b[end] == b'\t') {
        end += 1;
    }
    out.push((range.start..end, MdKind::Task(checked)));
}

/// Byte ranges of every ISOLATED two-`=` run in `s` — a valid `==` delimiter
/// candidate for [`push_highlight_spans`]. "Isolated" means the byte immediately
/// before AND after the pair (if any) is NOT itself `=`, so a run of exactly 1
/// (`=`), 3 (`===`), or 4+ (`====`) equals yields ZERO candidates at any offset
/// within it — every position in a longer run fails the "not `=`" check on one
/// side or the other. This single rule is what makes a bare `=` meaningless
/// (never a run of 2) and what makes an adjacent `====` inert (no candidate
/// anywhere in it) — no special-casing either edge case separately. Pure, O(n).
///
/// THE ONE OWNER of the `==highlight==` delimiter gate. `==` is NOT a CommonMark
/// construct (pulldown emits it as literal text), so the RENDER
/// ([`push_highlight_spans`] here) and the EXPORT
/// ([`crate::export::model::split_highlight`]) each hand-roll the same
/// after-the-fact scan — and they MUST agree on which `=` runs count, or an inert
/// `===` / bare `=` would highlight in one path but not the other (the exact
/// two-owner shape that produced the `~x~` strike divergence). Sharing this pub
/// owner is what keeps them byte-for-byte identical — see the
/// `render_export_highlight_agree` law test. Both callers pair the returned
/// candidates greedily (open `k`, close `k+1`), so the pairing stays local to
/// each while the candidate SET is shared.
pub fn equals_runs(s: &str) -> Vec<Range<usize>> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < b.len() {
        if b[i] == b'='
            && b[i + 1] == b'='
            && (i == 0 || b[i - 1] != b'=')
            && (i + 2 >= b.len() || b[i + 2] != b'=')
        {
            out.push(i..i + 2);
            i += 2; // consume the whole marker; never rescan its bytes
        } else {
            i += 1;
        }
    }
    out
}

/// Detect `==marked==` runs within ONE text event's range (`range`, into the
/// document `text`) and push `Markup` (the `==` delimiters) + `Highlight` (the
/// marked content) spans onto `out`. NOT CommonMark — there is no `==` construct
/// in the spec; this ships the de-facto Obsidian/Typora/iA convention.
///
/// Delimiter candidates come from [`equals_runs`] (isolated two-`=` runs only).
/// They pair up GREEDILY, consuming two consecutive candidates at a time:
/// candidate `k` opens, candidate `k+1` closes. A trailing UNPAIRED candidate (an
/// odd one out at the end of the list) is simply left as literal `=` characters —
/// the "unclosed `==`" case: no span, no panic, just plain text. A candidate pair
/// separated by a `\n` is rejected too (NO CROSS-LINE SPANS): the open is
/// discarded as literal and the rejected close is retried as a fresh open against
/// the NEXT candidate, so `a==\nb==c==` still highlights `c` from the trailing
/// pair. In practice a soft-wrapped paragraph already arrives as separate `Text`
/// events split at the line break (pulldown emits `Event::SoftBreak` between
/// them, never embedding the `\n` in a `Text` range), so this mostly guards a
/// defensive edge the parser doesn't otherwise produce — see the direct
/// [`push_highlight_spans`] unit test that constructs one by hand.
/// WYSIWYG-concealable ([`ConcealKind::Highlight`]): the `==` delimiters hide off
/// the caret's line — the wash stroke IS the affordance once they do.
pub(in crate::markdown) fn push_highlight_spans(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let s = &text[range.clone()];
    let markers = equals_runs(s);
    let k_markup = MdKind::ConcealMarkup(ConcealKind::Highlight);
    let mut k = 0usize;
    while k + 1 < markers.len() {
        let open = markers[k].clone();
        let close = markers[k + 1].clone();
        if s[open.end..close.start].contains('\n') {
            k += 1; // no cross-line spans: discard `open`, retry `close` as a new open
            continue;
        }
        out.push((range.start + open.start..range.start + open.end, k_markup));
        out.push((
            range.start + open.end..range.start + close.start,
            MdKind::Highlight,
        ));
        out.push((range.start + close.start..range.start + close.end, k_markup));
        k += 2;
    }
}

/// Detect bare `scheme://…` URLs within ONE text-event's range (`range`, into
/// the document `text`) and push their flanking [`ConcealKind::BareUrl`] spans
/// — a SCHEME span always, a TAIL span only when the URL carries a path/query
/// beyond its authority (see [`bare_url_split`]). The authority itself
/// (host[:port]) gets no span at all: it stays real, always-visible content
/// ink, exactly the way a link's visible text stays untouched by its own
/// flanking [`ConcealKind::Link`] plumbing — this is that same precedent, one
/// event later. Detection candidates come from [`bare_url_ranges`], which never
/// crosses a whitespace boundary, so multiple URLs in one run are each split
/// and pushed independently.
pub(super) fn push_bare_url_spans(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let s = &text[range.clone()];
    let k = MdKind::ConcealMarkup(ConcealKind::BareUrl);
    for url_rel in bare_url_ranges(s) {
        let url = &s[url_rel.clone()];
        let (scheme_rel, tail_rel) = bare_url_split(url);
        let base = range.start + url_rel.start;
        if scheme_rel.end > scheme_rel.start {
            out.push((base + scheme_rel.start..base + scheme_rel.end, k));
        }
        if let Some(tail_rel) = tail_rel {
            out.push((base + tail_rel.start..base + tail_rel.end, k));
        }
    }
}

/// Inline `` `code` ``: dim the matching backtick runs at each end, mono-tint the
/// inner slice. The backticks are WYSIWYG-concealable ([`ConcealKind::Code`]); the
/// content span is `MdKind::Code { inline: true }` — the renderer washes it with a
/// small pill (see `render::rects::ensure_code_pill_protos`), unlike a block body.
pub(super) fn push_inline_code(
    out: &mut Vec<(Range<usize>, MdKind)>,
    text: &str,
    range: &Range<usize>,
) {
    let s = &text[range.clone()];
    let b = s.as_bytes();
    let open = b.iter().take_while(|&&c| c == b'`').count();
    let close = b.iter().rev().take_while(|&&c| c == b'`').count();
    if open == 0 || open + close > b.len() {
        // Degenerate (shouldn't happen for a Code event) — tint the whole thing.
        out.push((range.clone(), MdKind::Code { inline: true }));
        return;
    }
    let k_markup = MdKind::ConcealMarkup(ConcealKind::Code);
    out.push((range.start..range.start + open, k_markup));
    out.push((range.end - close..range.end, k_markup));
    if range.start + open < range.end - close {
        out.push((
            range.start + open..range.end - close,
            MdKind::Code { inline: true },
        ));
    }
}
