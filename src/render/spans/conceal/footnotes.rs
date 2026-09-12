//! Footnote source concealment and its corresponding painted-number slot.

use super::*;

/// Add the forced leading slot for one concealed footnote span, returning
/// whether this was a real footnote reference or definition.
#[allow(clippy::too_many_arguments)]
pub(super) fn add_footnote_conceal_spans(
    al: &mut glyphon::cosmic_text::AttrsList,
    line_text: &str,
    line_doc_start: usize,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    range: &std::ops::Range<usize>,
    lo: usize,
    hi: usize,
    hidden: &Attrs<'static>,
    advances: SubstituteAdvances,
) -> bool {
    use crate::markdown::MdKind;
    let number = md_spans.iter().find_map(|(span, kind)| {
        if span.start < range.end && range.start < span.end {
            match kind {
                MdKind::FootnoteReference(number) | MdKind::FootnoteDefinition(number) => {
                    Some(*number)
                }
                _ => None,
            }
        } else {
            None
        }
    });
    let Some(number) = number else {
        return false;
    };

    // The source collapses, but the drawn superscript needs a real caret/hit-test
    // cell and following prose must begin after it. Force the first concealed
    // scalar to the number's OWN shaped advance plus one calm gap
    // (`SubstituteAdvances::footnote_slot`); the remaining source stays truly
    // zero-width. A row-metric fraction cannot serve this: it under-reserved a
    // two-digit number on six worlds and over-reserved a heading row on all of
    // them, because the number always paints at BODY size whatever row it lands
    // on.
    super::substitutes::add_reserved_conceal_spans(
        al,
        line_text,
        line_doc_start,
        lo..hi,
        hidden,
        advances.footnote_slot(number) / CONCEAL_ZERO_WIDTH_FONT_SIZE,
    );
    true
}
