//! Pure, atomic markdown block and inline formatting toggles.

use super::*;

mod footnotes;
pub(super) use footnotes::apply_insert_footnote;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FormatResult {
    pub text: String,
    pub anchor: Option<usize>,
    pub cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockKind {
    Blockquote,
    Bullet,
    Numbered,
    Task,
    Heading,
    CodeBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineKind {
    Bold,
    Italic,
    InlineCode,
    Highlight,
    Strikethrough,
}

impl InlineKind {
    fn delim(self) -> &'static str {
        match self {
            InlineKind::Bold => "**",
            InlineKind::Italic => "*",
            InlineKind::InlineCode => "`",
            InlineKind::Highlight => "==",
            InlineKind::Strikethrough => "~~",
        }
    }
}

/// Run a BLOCK toggle over the caret line / selection and apply it as one undoable
/// edit. A markdown-only command (a `.rs`/`.txt` buffer is left untouched — block
/// markup would corrupt code), and a calm no-op when the transform changes nothing.
pub(super) fn apply_block_format(ctx: &mut ActionCtx, kind: BlockKind) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    let r = block_toggle(kind, &text, anchor, cursor);
    ctx.buffer.apply_format(&r.text, r.anchor, r.cursor);
}

pub(super) fn apply_inline_format(ctx: &mut ActionCtx, kind: InlineKind) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    let r = inline_toggle(kind, &text, anchor, cursor);
    ctx.buffer.apply_format(&r.text, r.anchor, r.cursor);
}

// --- Shared line helpers ----------------------------------------------------

fn split_lines(text: &str) -> Vec<String> {
    text.split('\n').map(str::to_string).collect()
}

fn line_start_char(lines: &[String], l: usize) -> usize {
    lines[..l].iter().map(|s| s.chars().count() + 1).sum()
}

fn char_to_line_col(lines: &[String], idx: usize) -> (usize, usize) {
    let mut acc = 0;
    for (l, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        if idx <= acc + len {
            return (l, idx - acc);
        }
        acc += len + 1; // + the newline
    }
    let last = lines.len() - 1;
    (last, lines[last].chars().count())
}

pub(super) fn sel_range(anchor: Option<usize>, cursor: usize) -> (usize, usize, bool) {
    match anchor {
        Some(a) if a != cursor => (a.min(cursor), a.max(cursor), true),
        _ => (cursor, cursor, false),
    }
}

fn indent_len(line: &[char]) -> usize {
    line.iter().take_while(|&&c| c == ' ' || c == '\t').count()
}

fn starts_with_at(line: &[char], from: usize, pat: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    from + p.len() <= line.len() && line[from..from + p.len()] == p[..]
}

fn numbered_prefix_len(line: &[char], from: usize) -> Option<usize> {
    let mut d = from;
    while d < line.len() && line[d].is_ascii_digit() {
        d += 1;
    }
    if d > from && d + 1 < line.len() && matches!(line[d], '.' | ')') && line[d + 1] == ' ' {
        Some((d - from) + 2)
    } else {
        None
    }
}

fn block_prefix(kind: BlockKind, seq: usize) -> String {
    match kind {
        BlockKind::Blockquote => "> ".to_string(),
        BlockKind::Bullet => "- ".to_string(),
        BlockKind::Numbered => format!("{seq}. "),
        BlockKind::Task => "- [ ] ".to_string(),
        BlockKind::Heading => "# ".to_string(),
        BlockKind::CodeBlock => String::new(), // handled by the fenced-wrapper branch
    }
}

fn present_prefix_len(kind: BlockKind, line: &[char], ind: usize) -> Option<usize> {
    match kind {
        BlockKind::Blockquote => starts_with_at(line, ind, "> ").then_some(2),
        BlockKind::Bullet => starts_with_at(line, ind, "- ").then_some(2),
        BlockKind::Numbered => numbered_prefix_len(line, ind),
        BlockKind::Task => {
            if starts_with_at(line, ind, "- [ ] ")
                || starts_with_at(line, ind, "- [x] ")
                || starts_with_at(line, ind, "- [X] ")
            {
                Some(6)
            } else {
                None
            }
        }
        BlockKind::Heading => starts_with_at(line, ind, "# ").then_some(2),
        BlockKind::CodeBlock => None,
    }
}

fn is_fence(line: &str) -> bool {
    crate::markdown::is_fence_line(line)
}

fn block_toggle(kind: BlockKind, text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    let lines = split_lines(text);
    let (s, e, has_sel) = sel_range(anchor, cursor);
    let (first, _) = char_to_line_col(&lines, s);
    let (mut last, end_col) = char_to_line_col(&lines, e);
    if has_sel && last > first && end_col == 0 {
        last -= 1;
    }

    if kind == BlockKind::CodeBlock {
        return code_block_toggle(&lines, first, last, has_sel);
    }

    let chars: Vec<Vec<char>> = lines.iter().map(|s| s.chars().collect()).collect();
    let nonempty: Vec<usize> = (first..=last)
        .filter(|&l| !lines[l].trim().is_empty())
        .collect();
    let all_prefixed = !nonempty.is_empty()
        && nonempty
            .iter()
            .all(|&l| present_prefix_len(kind, &chars[l], indent_len(&chars[l])).is_some());
    let strip = all_prefixed;

    let mut new_lines = lines.clone();
    let mut first_op: (i64, usize) = (0, 0); // (signed delta, indentation position)
    let mut seq = 0usize;
    for l in first..=last {
        if lines[l].trim().is_empty() {
            continue; // blank line: never prefixed / stripped
        }
        let line = &chars[l];
        let ind = indent_len(line);
        let (rebuilt, delta, at): (String, i64, usize) = if strip {
            let plen = present_prefix_len(kind, line, ind).unwrap_or(0);
            let mut v: Vec<char> = line[..ind].to_vec();
            v.extend_from_slice(&line[ind + plen..]);
            (v.into_iter().collect(), -(plen as i64), ind)
        } else {
            seq += 1;
            let prefix = block_prefix(kind, seq);
            let mut v: Vec<char> = line[..ind].to_vec();
            v.extend(prefix.chars());
            v.extend_from_slice(&line[ind..]);
            (v.into_iter().collect(), prefix.chars().count() as i64, ind)
        };
        if l == first {
            first_op = (delta, at);
        }
        new_lines[l] = rebuilt;
    }

    let new_text = new_lines.join("\n");
    let (anchor, cursor) = if has_sel {
        let a = line_start_char(&new_lines, first);
        let c = line_start_char(&new_lines, last) + new_lines[last].chars().count();
        (Some(a), c)
    } else {
        let (_, col) = char_to_line_col(&lines, cursor);
        let (delta, at) = first_op;
        let new_col = remap_col(col, delta, at);
        (None, line_start_char(&new_lines, first) + new_col)
    };
    FormatResult {
        text: new_text,
        anchor,
        cursor,
    }
}

fn remap_col(col: usize, delta: i64, at: usize) -> usize {
    if delta > 0 {
        if col >= at { col + delta as usize } else { col }
    } else if delta < 0 {
        let plen = (-delta) as usize;
        if col <= at {
            col
        } else if col >= at + plen {
            col - plen
        } else {
            at
        }
    } else {
        col
    }
}

fn code_block_toggle(lines: &[String], first: usize, last: usize, _has_sel: bool) -> FormatResult {
    let already = last > first && is_fence(&lines[first]) && is_fence(&lines[last]);
    if already {
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() - 2);
        new_lines.extend_from_slice(&lines[..first]);
        new_lines.extend_from_slice(&lines[first + 1..last]);
        new_lines.extend_from_slice(&lines[last + 1..]);
        let inner = last - first - 1; // body line count
        let new_text = new_lines.join("\n");
        let (anchor, cursor) = if inner > 0 {
            let a = line_start_char(&new_lines, first);
            let body_last = first + inner - 1;
            let c = line_start_char(&new_lines, body_last) + new_lines[body_last].chars().count();
            (Some(a), c)
        } else {
            (None, line_start_char(&new_lines, first))
        };
        FormatResult {
            text: new_text,
            anchor,
            cursor,
        }
    } else {
        let mut new_lines: Vec<String> = Vec::with_capacity(lines.len() + 2);
        new_lines.extend_from_slice(&lines[..first]);
        new_lines.push("```".to_string());
        new_lines.extend_from_slice(&lines[first..=last]);
        new_lines.push("```".to_string());
        new_lines.extend_from_slice(&lines[last + 1..]);
        let new_text = new_lines.join("\n");
        let close = last + 2; // index of the closing fence in new_lines
        let a = line_start_char(&new_lines, first);
        let c = line_start_char(&new_lines, close) + new_lines[close].chars().count();
        FormatResult {
            text: new_text,
            anchor: Some(a),
            cursor: c,
        }
    }
}

fn line_heading_level(line: &[char], ind: usize) -> usize {
    let mut h = ind;
    while h < line.len() && line[h] == '#' {
        h += 1;
    }
    let n = h - ind;
    if (1..=6).contains(&n) && h < line.len() && line[h] == ' ' {
        n
    } else {
        0
    }
}

fn heading_prefix_char_len(line: &[char], ind: usize) -> usize {
    let lvl = line_heading_level(line, ind);
    if lvl > 0 { lvl + 1 } else { 0 }
}

fn next_heading_level(cur: usize) -> usize {
    match cur {
        0 => 1,
        1 => 2,
        2 => 3,
        _ => 0,
    }
}

pub(crate) fn heading_level(text: &str, anchor: Option<usize>, cursor: usize) -> usize {
    let lines = split_lines(text);
    let (s, e, has_sel) = sel_range(anchor, cursor);
    let (first, _) = char_to_line_col(&lines, s);
    let (mut last, end_col) = char_to_line_col(&lines, e);
    if has_sel && last > first && end_col == 0 {
        last -= 1;
    }
    let chars: Vec<Vec<char>> = lines.iter().map(|s| s.chars().collect()).collect();
    (first..=last)
        .find(|&l| !lines[l].trim().is_empty())
        .map(|l| line_heading_level(&chars[l], indent_len(&chars[l])))
        .unwrap_or(0)
}

pub(super) fn apply_heading_cycle(ctx: &mut ActionCtx) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    let r = heading_cycle(&text, anchor, cursor);
    ctx.buffer.apply_format(&r.text, r.anchor, r.cursor);
}

fn heading_cycle(text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    let lines = split_lines(text);
    let (s, e, has_sel) = sel_range(anchor, cursor);
    let (first, _) = char_to_line_col(&lines, s);
    let (mut last, end_col) = char_to_line_col(&lines, e);
    if has_sel && last > first && end_col == 0 {
        last -= 1;
    }
    let chars: Vec<Vec<char>> = lines.iter().map(|s| s.chars().collect()).collect();
    let cur = (first..=last)
        .find(|&l| !lines[l].trim().is_empty())
        .map(|l| line_heading_level(&chars[l], indent_len(&chars[l])))
        .unwrap_or(0);
    let target = next_heading_level(cur);
    let prefix: String = if target > 0 {
        format!("{} ", "#".repeat(target))
    } else {
        String::new()
    };
    let new_len = prefix.chars().count();

    let mut new_lines = lines.clone();
    let mut first_op: (i64, usize, usize) = (0, 0, 0);
    for l in first..=last {
        if lines[l].trim().is_empty() {
            continue;
        }
        let line = &chars[l];
        let ind = indent_len(line);
        let existing = heading_prefix_char_len(line, ind);
        let mut v: Vec<char> = line[..ind].to_vec();
        v.extend(prefix.chars());
        v.extend_from_slice(&line[ind + existing..]);
        if l == first {
            first_op = (new_len as i64 - existing as i64, ind, existing);
        }
        new_lines[l] = v.into_iter().collect();
    }

    let new_text = new_lines.join("\n");
    let (anchor, cursor) = if has_sel {
        let a = line_start_char(&new_lines, first);
        let c = line_start_char(&new_lines, last) + new_lines[last].chars().count();
        (Some(a), c)
    } else {
        let (_, col) = char_to_line_col(&lines, cursor);
        let (_, at, existing) = first_op;
        let stripped = remap_col(col, -(existing as i64), at);
        let new_col = remap_col(stripped, new_len as i64, at);
        (None, line_start_char(&new_lines, first) + new_col)
    };
    FormatResult {
        text: new_text,
        anchor,
        cursor,
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The char span an inline toggle acts on: the selection, or the word under the
/// caret, or (with neither) a bare caret (`want_caret`). Shared by [`inline_toggle`]
/// and the popover's [`inline_active`] so the two can never disagree on WHICH span
/// a button reads.
fn inline_span(chars: &[char], anchor: Option<usize>, cursor: usize) -> (usize, usize, bool) {
    let (s, e, has_sel) = sel_range(anchor, cursor);
    if has_sel {
        return (s, e, false);
    }
    let mut a = cursor;
    while a > 0 && is_word_char(chars[a - 1]) {
        a -= 1;
    }
    let mut b = cursor;
    while b < chars.len() && is_word_char(chars[b]) {
        b += 1;
    }
    if b > a {
        (a, b, false)
    } else {
        (cursor, cursor, true) // no word → empty-delimiter insert
    }
}

/// WHERE `kind`'s delimiters sit relative to span `[ws, we)`, when it is already
/// wrapped — the ONE definition of "already formatted" shared by the toggle (it
/// STRIPs) and the popover's lit oracle ([`inline_active`], it LIGHTS), so they
/// can never disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineWrap {
    Surrounding,
    Enclosing,
}

fn inline_wrap(
    kind: InlineKind,
    chars: &[char],
    text: &str,
    ws: usize,
    we: usize,
) -> Option<InlineWrap> {
    let d: Vec<char> = kind.delim().chars().collect();
    let dl = d.len();
    let eq = |from: usize| from + dl <= chars.len() && chars[from..from + dl] == d[..];
    let (plan, content_empty) = if ws >= dl && we + dl <= chars.len() && eq(ws - dl) && eq(we) {
        (InlineWrap::Surrounding, ws == we)
    } else if we - ws >= 2 * dl && eq(ws) && eq(we - dl) {
        (InlineWrap::Enclosing, we - ws == 2 * dl)
    } else {
        return None;
    };
    if content_empty || content_is_kind(kind, text, ws, we) {
        Some(plan)
    } else {
        None
    }
}

fn content_is_kind(kind: InlineKind, text: &str, ws: usize, we: usize) -> bool {
    let mid_char = ws + (we - ws) / 2;
    let mid_byte = char_to_byte(text, mid_char);
    let spans = crate::markdown::spans(text);
    if spans
        .iter()
        .any(|(r, k)| r.contains(&mid_byte) && kind_matches_span(kind, *k))
    {
        return true;
    }
    // FALLBACK, InlineCode + a newline in the span only: a genuine CommonMark
    // code SPAN cannot cross a paragraph/block boundary (a blank line, or a
    // line a list/heading marker turns into its own block), so the real parser
    // never confirms one whose wrapped content crosses that boundary — even
    // though the backticks this command inserted are still sitting right
    // there. Backtick has no sibling delimiter to disambiguate against (unlike
    // `*` vs `**`), so recognizing the strip doesn't need positive
    // confirmation here; it only needs to rule out the one real false
    // positive, a literal backtick that is source text INSIDE an actual
    // fenced/indented code block, where the toggle must never mistake code-
    // body characters for its own markup. Gated on a literal `\n` in the span
    // so a same-line flanked pair (`` `a` or `b` ``, selecting " or ") still
    // requires the positive match above and is never merged.
    kind == InlineKind::InlineCode
        && text[char_to_byte(text, ws)..char_to_byte(text, we)].contains('\n')
        && !spans.iter().any(|(r, k)| {
            r.contains(&mid_byte) && matches!(k, crate::markdown::MdKind::Code { inline: false })
        })
}

fn kind_matches_span(kind: InlineKind, k: crate::markdown::MdKind) -> bool {
    use crate::markdown::MdKind;
    match kind {
        InlineKind::Bold => matches!(k, MdKind::Bold | MdKind::BoldItalic),
        InlineKind::Italic => matches!(k, MdKind::Italic | MdKind::BoldItalic),
        InlineKind::InlineCode => matches!(k, MdKind::Code { inline: true }),
        InlineKind::Highlight => matches!(k, MdKind::Highlight),
        InlineKind::Strikethrough => matches!(k, MdKind::Strikethrough),
    }
}

fn char_to_byte(text: &str, char_idx: usize) -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(text.len())
}

pub(crate) fn inline_active(
    kind: InlineKind,
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let (ws, we, _) = inline_span(&chars, anchor, cursor);
    inline_wrap(kind, &chars, text, ws, we).is_some()
}

fn inline_toggle(
    kind: InlineKind,
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
) -> FormatResult {
    let chars: Vec<char> = text.chars().collect();
    let d: Vec<char> = kind.delim().chars().collect();
    let dl = d.len();

    let (ws, we, want_caret) = inline_span(&chars, anchor, cursor);

    // STRIP — the span is already wrapped by `kind`. WHERE the delimiters sit comes
    // from the ONE shared owner [`inline_wrap`] (the same the popover lights from),
    // so the toggle can never strip a `*` that is really half of a `**` bold fence:
    // pressing I inside `**bold**` falls through to WRAP → `***bold***`, never a
    // silent bold→italic degrade.
    match inline_wrap(kind, &chars, text, ws, we) {
        Some(InlineWrap::Surrounding) => {
            let mut out: Vec<char> = Vec::with_capacity(chars.len() - 2 * dl);
            out.extend_from_slice(&chars[..ws - dl]);
            out.extend_from_slice(&chars[ws..we]);
            out.extend_from_slice(&chars[we + dl..]);
            let (a, c) = (ws - dl, we - dl);
            return finish_inline(out, ws == we, a, c);
        }
        Some(InlineWrap::Enclosing) => {
            let mut out: Vec<char> = Vec::with_capacity(chars.len() - 2 * dl);
            out.extend_from_slice(&chars[..ws]);
            out.extend_from_slice(&chars[ws + dl..we - dl]);
            out.extend_from_slice(&chars[we..]);
            let (a, c) = (ws, we - 2 * dl);
            return finish_inline(out, false, a, c);
        }
        None => {}
    }

    let mut out: Vec<char> = Vec::with_capacity(chars.len() + 2 * dl);
    out.extend_from_slice(&chars[..ws]);
    out.extend_from_slice(&d);
    out.extend_from_slice(&chars[ws..we]);
    out.extend_from_slice(&d);
    out.extend_from_slice(&chars[we..]);
    if want_caret {
        let c = ws + dl;
        FormatResult {
            text: out.into_iter().collect(),
            anchor: None,
            cursor: c,
        }
    } else {
        let (a, c) = (ws + dl, we + dl);
        finish_inline(out, false, a, c)
    }
}

fn finish_inline(out: Vec<char>, empty: bool, a: usize, c: usize) -> FormatResult {
    let text: String = out.into_iter().collect();
    if empty || a == c {
        FormatResult {
            text,
            anchor: None,
            cursor: c,
        }
    } else {
        FormatResult {
            text,
            anchor: Some(a),
            cursor: c,
        }
    }
}

#[cfg(test)]
mod tests;
