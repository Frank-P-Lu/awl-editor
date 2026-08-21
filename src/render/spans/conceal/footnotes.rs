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
    line_height: f32,
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
    // scalar to the same conservative slot the decoration geometry law grades;
    // the remaining source stays truly zero-width.
    let first_len = line_text[(lo - line_doc_start)..]
        .chars()
        .next()
        .map_or(0, char::len_utf8);
    let first_end = (lo + first_len).min(hi);
    if first_end > lo {
        let slot = footnote_number_slot(number, line_height);
        let forcing = hidden
            .clone()
            .letter_spacing(slot / CONCEAL_ZERO_WIDTH_FONT_SIZE);
        al.add_span(
            (lo - line_doc_start)..(first_end - line_doc_start),
            &forcing,
        );
    }
    if first_end < hi {
        al.add_span((first_end - line_doc_start)..(hi - line_doc_start), hidden);
    }
    true
}

/// Width reserved in the shaped document for one painted footnote number.
/// Digits beyond the first grow the slot; the fixed tail is the calm gap before
/// following prose. Derived only from the row metric so it scales across DPI,
/// zoom, heading rows, and every world without a rasterizer-specific constant.
pub(in crate::render) fn footnote_number_slot(number: usize, line_height: f32) -> f32 {
    let digits = number.max(1).ilog10() as f32 + 1.0;
    line_height * (0.34 + (digits - 1.0) * 0.20)
}
