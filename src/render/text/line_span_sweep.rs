//! Line-local Markdown spans in original paint order.

use crate::markdown::MdKind;
use std::ops::Range;

type Span = (Range<usize>, MdKind);

/// Walk ascending logical lines without rescanning the document's spans on
/// each line. The byte ranges stay in document coordinates (fence reveal
/// still needs the whole block); only the unrelated ranges are dropped.
pub(super) struct LineSpanSweep<'a> {
    spans: &'a [Span],
    by_start: Vec<usize>,
    entered: usize,
    active: Vec<usize>,
    local: Vec<Span>,
}

impl<'a> LineSpanSweep<'a> {
    pub(super) fn new(spans: &'a [Span]) -> Self {
        let mut by_start: Vec<_> = (0..spans.len()).collect();
        by_start.sort_unstable_by_key(|&i| (spans[i].0.start, i));
        Self {
            spans,
            by_start,
            entered: 0,
            active: Vec::new(),
            local: Vec::new(),
        }
    }

    /// Retain the former refresh predicate, allowing a span that begins at
    /// the line's ending byte. Consumers decide whether that span paints any
    /// glyphs. Only the temporary local selection is put back in parse order:
    /// applying overlapping styles in byte-start order can change their ink.
    pub(super) fn for_line(&mut self, start: usize, end: usize) -> &[Span] {
        self.active.retain(|&i| self.spans[i].0.end > start);
        while let Some(&i) = self.by_start.get(self.entered) {
            let range = &self.spans[i].0;
            if range.start > end {
                break;
            }
            self.entered += 1;
            if range.end > start {
                self.active.push(i);
            }
        }
        self.active.sort_unstable();
        self.local.clear();
        self.local
            .extend(self.active.iter().map(|&i| self.spans[i].clone()));
        &self.local
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::ConcealKind;

    #[test]
    fn sweep_matches_full_overlap_filter_without_reordering_paint() {
        let _g = crate::testlock::serial();
        let spans = [
            (20..25, MdKind::Rule),
            (1..23, MdKind::ConcealMarkup(ConcealKind::Fence)),
            (4..8, MdKind::Code { inline: false }),
            (4..8, MdKind::Bold),
            (0..1, MdKind::Quote),
            (9..12, MdKind::Italic),
            (17..18, MdKind::Rule),
        ];
        let mut sweep = LineSpanSweep::new(&spans);
        for (start, end) in [
            (0, 4),
            (5, 5),
            (6, 9),
            (10, 16),
            (17, 20),
            (21, 29),
            (30, 42),
        ] {
            let expected: Vec<_> = spans
                .iter()
                .filter(|(range, _)| range.start <= end && range.end > start)
                .cloned()
                .collect();
            assert_eq!(sweep.for_line(start, end), expected, "line {start}..{end}");
        }
    }
}
