//! The inline formatting family — ONE owner for how each kind's grammar binds
//! its delimiters to a payload: choosing them, recognizing them, stripping them.

use super::{ActionCtx, Effect, FormatResult, NoticeEffect, sel_range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InlineKind {
    Bold,
    Italic,
    InlineCode,
    Highlight,
    Strikethrough,
}

/// How a kind's delimiters bind to what they wrap. The distinction is the whole
/// reason a single `delim()` string was not enough: two grammars, not five.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grammar {
    /// A PROSE payload behind a fixed delimiter. Edge whitespace is pushed
    /// OUTSIDE the delimiters, because a CommonMark/GFM emphasis run flanked by
    /// whitespace on its inner side is neither left- nor right-flanking: `**a **`
    /// is four literal asterisks around plain text, not bold. `==`'s own scan
    /// (`markdown::spans::markers::equals_runs`) carries no flanking rule and
    /// would tolerate the space, but a wash that runs one blank past its last
    /// word is the same wart, so all four prose kinds trim alike.
    Prose(&'static str),
    /// A LITERAL payload: whitespace is content and stays inside. The fence is a
    /// backtick RUN long enough to clear every run within the payload, and it
    /// gains one space of padding on each side when the payload's own edge is a
    /// backtick — unpadded, the fence and the payload's edge run merge into one
    /// longer run and nothing parses as a code span at all.
    CodeSpan,
}

impl InlineKind {
    fn grammar(self) -> Grammar {
        match self {
            InlineKind::Bold => Grammar::Prose("**"),
            InlineKind::Italic => Grammar::Prose("*"),
            InlineKind::InlineCode => Grammar::CodeSpan,
            InlineKind::Highlight => Grammar::Prose("=="),
            InlineKind::Strikethrough => Grammar::Prose("~~"),
        }
    }
}

/// What a refusal SAYS. Two of the three shapes below are invisible from the
/// screen — a backtick one character outside the selection, a `==` run the
/// parser will never pair across an inline construct — so a silent no-op reads
/// as a dead command. The same reasoning `NO_TABLE_UNDER_CARET` and
/// `EXPORT_REQUIRES_MARKDOWN` record: a refusal with something to say. The
/// whitespace-only selection is deliberately NOT routed here — that one the
/// reader can already see.
const NO_VALID_MARKUP: &str = "markdown can't mark that up";

pub(in crate::actions) fn apply_inline_format(ctx: &mut ActionCtx, kind: InlineKind) -> Effect {
    if !ctx.buffer.is_markdown() {
        return Effect::None;
    }
    let text = ctx.buffer.text();
    let anchor = ctx.buffer.anchor_char();
    let cursor = ctx.buffer.cursor_char();
    match inline_toggle(kind, &text, anchor, cursor) {
        InlineToggle::Edit(r) => {
            ctx.buffer.apply_format(&r.text, r.anchor, r.cursor);
            Effect::None
        }
        InlineToggle::Nothing => Effect::None,
        InlineToggle::NoValidOutput => {
            Effect::Notice(NoticeEffect::Sticky(NO_VALID_MARKUP.to_string()))
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The raw char span an inline toggle starts from: the selection, or the word
/// under the caret, or (with neither) a bare caret (`want_caret`).
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

/// The payload span `kind` will actually carry, with the kind's own grammar
/// applied to the edges. The ONE resolution shared by the toggle (which EDITS)
/// and the popover's lit oracle ([`inline_active`], which LIGHTS), so the two can
/// never disagree about WHICH characters a button reads.
///
/// `None` = a prose selection holding nothing but whitespace: no emphasis run can
/// wrap it, so the command is a calm no-op rather than a producer of literal
/// asterisks.
fn payload_span(
    kind: InlineKind,
    chars: &[char],
    anchor: Option<usize>,
    cursor: usize,
) -> Option<(usize, usize, bool)> {
    let (mut ws, mut we, want_caret) = inline_span(chars, anchor, cursor);
    if matches!(kind.grammar(), Grammar::Prose(_)) {
        while ws < we && chars[ws].is_whitespace() {
            ws += 1;
        }
        while we > ws && chars[we - 1].is_whitespace() {
            we -= 1;
        }
        if ws == we && !want_caret {
            return None;
        }
    }
    Some((ws, we, want_caret))
}

/// The delimiter pair that carries exactly `payload` under `kind`'s grammar.
fn delims(kind: InlineKind, payload: &[char]) -> (Vec<char>, Vec<char>) {
    match kind.grammar() {
        Grammar::Prose(d) => (d.chars().collect(), d.chars().collect()),
        Grammar::CodeSpan => {
            let mut open = vec!['`'; code_fence_len(payload)];
            let mut close = open.clone();
            if code_needs_pad(payload) {
                open.push(' ');
                close.insert(0, ' ');
            }
            (open, close)
        }
    }
}

/// The shortest backtick run that cannot collide with a run inside `payload` —
/// a code span closes on the first run of EXACTLY its own length, so a fence one
/// longer than any run it encloses is the shortest one that reaches the end.
fn code_fence_len(payload: &[char]) -> usize {
    let mut runs: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < payload.len() {
        if payload[i] == '`' {
            let start = i;
            while i < payload.len() && payload[i] == '`' {
                i += 1;
            }
            runs.push(i - start);
        } else {
            i += 1;
        }
    }
    let mut n = 1;
    while runs.contains(&n) {
        n += 1;
    }
    n
}

/// Whether `payload` needs a space of padding inside its fence. Only a backtick
/// at either edge does: it would otherwise fuse with the fence's own run. An edge
/// SPACE does not — a code span whose source begins and ends with one loses that
/// pair to CommonMark's normalization, but awl styles the source bytes, so
/// padding here would show the reader spaces they never selected.
fn code_needs_pad(payload: &[char]) -> bool {
    payload.first() == Some(&'`') || payload.last() == Some(&'`')
}

fn run_before(chars: &[char], i: usize) -> usize {
    let mut n = 0;
    while n < i && chars[i - 1 - n] == '`' {
        n += 1;
    }
    n
}

fn run_after(chars: &[char], i: usize) -> usize {
    let mut n = 0;
    while i + n < chars.len() && chars[i + n] == '`' {
        n += 1;
    }
    n
}

/// An already-wrapped construct found around or inside a payload span: the char
/// range of the WHOLE construct (`outer`) and of its literal payload (`inner`).
/// Stripping is the one edit `outer` → `inner`, whichever way the delimiters sat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Wrapped {
    outer: (usize, usize),
    inner: (usize, usize),
}

/// WHERE `kind`'s delimiters sit relative to span `[ws, we)`, when it is already
/// wrapped — the ONE definition of "already formatted" shared by the toggle (it
/// STRIPs) and the popover's lit oracle ([`inline_active`], it LIGHTS), so they
/// can never disagree.
fn inline_wrap(
    kind: InlineKind,
    chars: &[char],
    text: &str,
    ws: usize,
    we: usize,
) -> Option<Wrapped> {
    let w = match kind.grammar() {
        Grammar::Prose(d) => prose_wrap(d, chars, ws, we)?,
        Grammar::CodeSpan => code_wrap(chars, ws, we)?,
    };
    // An empty payload is this command's own `****`/`` `` `` and needs no
    // confirmation; anything else must be what the real parser SEES, so pressing
    // I inside `**bold**` falls through to WRAP → `***bold***` rather than
    // stripping a `*` that is really half of a bold fence.
    if w.inner.0 == w.inner.1 || content_is_kind(kind, text, w.inner) {
        Some(w)
    } else {
        None
    }
}

fn prose_wrap(d: &str, chars: &[char], ws: usize, we: usize) -> Option<Wrapped> {
    let d: Vec<char> = d.chars().collect();
    let dl = d.len();
    let eq = |from: usize| from + dl <= chars.len() && chars[from..from + dl] == d[..];
    if ws >= dl && we + dl <= chars.len() && eq(ws - dl) && eq(we) {
        Some(Wrapped {
            outer: (ws - dl, we + dl),
            inner: (ws, we),
        })
    } else if we - ws >= 2 * dl && eq(ws) && eq(we - dl) {
        Some(Wrapped {
            outer: (ws, we),
            inner: (ws + dl, we - dl),
        })
    } else {
        None
    }
}

/// A code span's fence is variable-length and may be padded, so recognizing one
/// mirrors [`delims`] rather than comparing a fixed string: the padding is
/// admitted exactly where this command would have emitted it for THIS payload.
fn code_wrap(chars: &[char], ws: usize, we: usize) -> Option<Wrapped> {
    let pad = usize::from(code_needs_pad(&chars[ws..we]));
    if ws >= pad
        && we + pad <= chars.len()
        && (pad == 0 || (chars[ws - 1] == ' ' && chars[we] == ' '))
    {
        let (l, r) = (ws - pad, we + pad);
        let (open, close) = (run_before(chars, l), run_after(chars, r));
        if open > 0 && open == close {
            return Some(Wrapped {
                outer: (l - open, r + close),
                inner: (ws, we),
            });
        }
    }
    let n = run_after(chars, ws);
    if n == 0 || n != run_before(chars, we) || we - ws < 2 * n {
        return None;
    }
    let (rs, re) = (ws + n, we - n);
    if re - rs >= 2
        && chars[rs] == ' '
        && chars[re - 1] == ' '
        && code_needs_pad(&chars[rs + 1..re - 1])
    {
        return Some(Wrapped {
            outer: (ws, we),
            inner: (rs + 1, re - 1),
        });
    }
    Some(Wrapped {
        outer: (ws, we),
        inner: (rs, re),
    })
}

/// Does the real parse already wear `kind` over the payload at `inner`?
///
/// Asked at the payload's two ENDS rather than at its midpoint: a nested
/// construct owns the bytes it covers, so `**a `tick` b**` samples a CODE span in
/// the middle and would read as "not bold" over content that is.
///
/// TWO PARSER VIEWS, not one, because a styling span is not evidence of absence.
/// `markdown::spans` reports what a byte WEARS, and a byte only wears what a
/// prose `Event::Text` carried — so `` **`y`** `` (no Text event inside at all)
/// and a bolded word inside a link or heading (the context outranks emphasis)
/// both report "not bold" while being bold. `emphasis_content_spans` answers the
/// structural question for exactly that family. The two are unioned rather than
/// alternated: `Code`/`Highlight` live only in the first, emphasis in both.
fn content_is_kind(kind: InlineKind, text: &str, inner: (usize, usize)) -> bool {
    let (ws, we) = inner;
    let ends = [
        char_to_byte(text, ws),
        char_to_byte(text, we.saturating_sub(1)),
    ];
    let spans = crate::markdown::spans(text);
    let structural = crate::markdown::emphasis_content_spans(text);
    let wears = |b: usize| {
        spans
            .iter()
            .chain(structural.iter())
            .any(|(r, k)| r.contains(&b) && kind_matches_span(kind, *k))
    };
    if ends.iter().all(|&b| wears(b)) {
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
        && text[ends[0]..char_to_byte(text, we)].contains('\n')
        && !spans.iter().any(|(r, k)| {
            ends.iter().any(|b| r.contains(b))
                && matches!(k, crate::markdown::MdKind::Code { inline: false })
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
    payload_span(kind, &chars, anchor, cursor)
        .and_then(|(ws, we, _)| inline_wrap(kind, &chars, text, ws, we))
        .is_some()
}

/// What one press of an inline toggle does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum InlineToggle {
    /// The edit to apply.
    Edit(FormatResult),
    /// No edit and nothing to say: a prose selection holding only whitespace,
    /// which the reader can already see is empty.
    Nothing,
    /// No edit, and a REASON worth saying: every output this grammar could emit
    /// here would be literal characters rather than the construct asked for.
    NoValidOutput,
}

/// WOULD THE WRAP THIS COMMAND IS ABOUT TO EMIT ACTUALLY MEAN `kind`?
///
/// Asked of the emitted text through the same oracle that recognizes a strip,
/// so the two directions cannot hold different opinions about what the document
/// says. Three shapes fail it, and all three predate the fence work:
///
/// - a document backtick immediately OUTSIDE a code payload (`` x`y ``, select
///   `y`) — the flank check below, because no fence length can escape it: the
///   opening run becomes `n + 1` while the closing stays `n`, at every `n`;
/// - a `==highlight==` whose payload holds any inline construct — the `==` scan
///   sees one `Event::Text` at a time and never pairs a marker across a code
///   span, so the delimiters would sit in the document as literal `=`;
/// - a prose selection crossing a block boundary, where CommonMark has no
///   emphasis run to give.
///
/// The one shape that PASSES while the parser withholds confirmation is a code
/// span across a block boundary, and it passes through `content_is_kind`'s own
/// documented fallback rather than an exception here.
fn wrap_means_kind(
    kind: InlineKind,
    chars: &[char],
    ws: usize,
    we: usize,
    out: &str,
    payload: (usize, usize),
) -> bool {
    if matches!(kind.grammar(), Grammar::CodeSpan)
        && (chars.get(ws.wrapping_sub(1)) == Some(&'`') || chars.get(we) == Some(&'`'))
    {
        return false;
    }
    content_is_kind(kind, out, payload)
}

pub(super) fn inline_toggle(
    kind: InlineKind,
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
) -> InlineToggle {
    let chars: Vec<char> = text.chars().collect();
    let Some((ws, we, want_caret)) = payload_span(kind, &chars, anchor, cursor) else {
        return InlineToggle::Nothing;
    };

    if let Some(w) = inline_wrap(kind, &chars, text, ws, we) {
        let mut out: Vec<char> = Vec::with_capacity(chars.len());
        out.extend_from_slice(&chars[..w.outer.0]);
        out.extend_from_slice(&chars[w.inner.0..w.inner.1]);
        out.extend_from_slice(&chars[w.outer.1..]);
        let a = w.outer.0;
        let c = a + (w.inner.1 - w.inner.0);
        return InlineToggle::Edit(finish_inline(out, a, c));
    }

    let (open, close) = delims(kind, &chars[ws..we]);
    let mut out: Vec<char> = Vec::with_capacity(chars.len() + open.len() + close.len());
    out.extend_from_slice(&chars[..ws]);
    out.extend_from_slice(&open);
    out.extend_from_slice(&chars[ws..we]);
    out.extend_from_slice(&close);
    out.extend_from_slice(&chars[we..]);
    let (a, c) = (ws + open.len(), we + open.len());
    let text: String = out.into_iter().collect();
    // An EMPTY payload is this command's own `****`/`` `` ``, which no parser
    // reports as anything — the caret case, exempt by construction rather than
    // by a threshold.
    if a < c && !wrap_means_kind(kind, &chars, ws, we, &text, (a, c)) {
        return InlineToggle::NoValidOutput;
    }
    if want_caret {
        InlineToggle::Edit(FormatResult {
            text,
            anchor: None,
            cursor: c,
        })
    } else {
        InlineToggle::Edit(finish_inline_text(text, a, c))
    }
}

fn finish_inline(out: Vec<char>, a: usize, c: usize) -> FormatResult {
    finish_inline_text(out.into_iter().collect(), a, c)
}

fn finish_inline_text(text: String, a: usize, c: usize) -> FormatResult {
    if a == c {
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
