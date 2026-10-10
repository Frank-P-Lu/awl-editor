//! Emphasis-family delimiter and content ownership shared by render and toggles.

use super::*;

/// One EMPHASIS-FAMILY inline construct — the three whose delimiters sit at
/// both ends of pulldown's own range and whose content is ordinary prose.
///
/// ONE table, read by the render walk ([`spans`], which dims the delimiters)
/// and by [`emphasis_content_spans`] (which reports what those delimiters
/// enclose), so the two can never disagree about where a construct's content
/// starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Emphasis {
    Strong,
    Em,
    Strike,
}

impl Emphasis {
    /// Bytes of delimiter at EACH end of the construct's own range.
    const fn delim(self) -> usize {
        match self {
            Emphasis::Em => 1,
            Emphasis::Strong | Emphasis::Strike => 2,
        }
    }

    const fn conceal(self) -> ConcealKind {
        match self {
            Emphasis::Strong | Emphasis::Em => ConcealKind::Emphasis,
            Emphasis::Strike => ConcealKind::Strikethrough,
        }
    }

    /// What the construct MEANS over its content, in the same `MdKind`
    /// vocabulary the render walk speaks.
    const fn content_kind(self) -> MdKind {
        match self {
            Emphasis::Strong => MdKind::Bold,
            Emphasis::Em => MdKind::Italic,
            Emphasis::Strike => MdKind::Strikethrough,
        }
    }

    pub(super) fn push_delim(self, out: &mut Vec<(Range<usize>, MdKind)>, range: &Range<usize>) {
        push_delim(out, range, self.delim(), self.conceal());
    }

    /// The construct's CONTENT extent inside its own delimiters — `None` when
    /// the range cannot hold both delimiters and a byte between them.
    fn content_range(self, range: &Range<usize>) -> Option<Range<usize>> {
        let d = self.delim();
        (range.end.saturating_sub(range.start) > 2 * d).then(|| range.start + d..range.end - d)
    }
}

/// The CONTENT byte extent of every emphasis-family construct the real parser
/// reports, whether or not a prose `Event::Text` survives inside it.
///
/// [`spans`] answers what a byte WEARS, and a byte can only wear what a Text
/// event carried. Two shapes defeat that: a payload that is entirely a code
/// span (`` **`y`** ``) emits `Event::Code` and no `Event::Text` at all, so it
/// carries no `Bold` span while being unambiguously bold; and a bolded word
/// inside a link, heading, quote or checked task wears that context's own kind
/// instead, because [`inline_kind`] ranks the context above emphasis. The
/// formatting toggles need the STRUCTURAL question — which construct COVERS
/// these bytes — which survives both.
///
/// Shares this module's own `strike_engaged` gate (so a single-tilde `~x~` is
/// inert here exactly as it is in the render), the [`Emphasis`] table's
/// delimiter widths, and `PARSE_OPTIONS`, so the two walks read one grammar.
pub fn emphasis_content_spans(text: &str) -> Vec<(Range<usize>, MdKind)> {
    use pulldown_cmark::{Event, Parser, Tag};

    let (body, offset) = match crate::frontmatter::detect(text) {
        Some(fm) => (&text[fm.range.end..], fm.range.end),
        None => (text, 0),
    };
    let mut out = Vec::new();
    for (ev, range) in Parser::new_ext(body, crate::markdown::PARSE_OPTIONS).into_offset_iter() {
        let construct = match ev {
            Event::Start(Tag::Strong) => Emphasis::Strong,
            Event::Start(Tag::Emphasis) => Emphasis::Em,
            Event::Start(Tag::Strikethrough) if strike_engaged(&body[range.clone()]) => {
                Emphasis::Strike
            }
            _ => continue,
        };
        if let Some(inner) = construct.content_range(&range) {
            out.push((
                inner.start + offset..inner.end + offset,
                construct.content_kind(),
            ));
        }
    }
    out
}
