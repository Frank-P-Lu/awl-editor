use super::*;

/// A parsed inline-image span, before layout assigns its display size or any
/// mixed-line forcing. Keeping discovery separate lets the layout pass own all
/// size-dependent decisions.
pub(super) struct FoundImageSpan {
    pub(super) range: std::ops::Range<usize>,
    pub(super) image: crate::markdown::ImageRef,
    pub(super) line: usize,
    pub(super) line_start: usize,
    pub(super) line_end: usize,
}

impl TextPipeline {
    pub(super) fn find_inline_image_spans(
        text: &str,
        md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
    ) -> Vec<FoundImageSpan> {
        use crate::markdown::{ConcealKind, MdKind};

        md_spans
            .iter()
            .filter(|(_, kind)| matches!(kind, MdKind::ConcealMarkup(ConcealKind::Image)))
            .filter_map(|(range, _)| {
                let image = text
                    .get(range.clone())
                    .and_then(crate::markdown::parse_image_source)?;
                let line = text[..range.start].bytes().filter(|&b| b == b'\n').count();
                let line_start = text[..range.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = text[range.end..]
                    .find('\n')
                    .map(|i| range.end + i)
                    .unwrap_or(text.len());
                Some(FoundImageSpan {
                    range: range.clone(),
                    image,
                    line,
                    line_start,
                    line_end,
                })
            })
            .collect()
    }
}
