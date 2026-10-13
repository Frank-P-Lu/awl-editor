//! Retain empty source lines omitted by the CommonMark event stream.
use super::{Block, Frame, accept_block};
use pulldown_cmark::{Event, TagEnd};

#[derive(Default)]
pub(super) struct SourceSpacing {
    cursor: usize,
}

impl SourceSpacing {
    pub fn before(
        &mut self,
        src: &str,
        event: &Event<'_>,
        range: &std::ops::Range<usize>,
        stack: &mut [Frame],
    ) {
        if !matches!(event, Event::End(_)) {
            self.advance(src, range.start, stack);
        }
        if !matches!(event, Event::Start(_) | Event::End(_)) {
            self.consume(range.end);
        }
        // Item gaps belong to the item, before it closes into a List frame.
        if matches!(event, Event::End(TagEnd::Item)) {
            self.advance(src, range.end, stack);
        }
    }

    pub fn consume(&mut self, end: usize) {
        self.cursor = self.cursor.max(end);
    }
    pub fn finish_event(&mut self, src: &str, end: usize, stack: &mut [Frame]) {
        self.advance(src, end, stack);
    }
    pub fn advance(&mut self, src: &str, end: usize, stack: &mut [Frame]) {
        if end <= self.cursor {
            return;
        }
        let begins_midline = self.cursor > 0 && src.as_bytes()[self.cursor - 1] != b'\n';
        let count = src[self.cursor..end]
            .split_inclusive('\n')
            .enumerate()
            .filter(|(index, line)| {
                if (*index == 0 && begins_midline) || !line.ends_with('\n') {
                    return false;
                }
                // Empty quote lines carry only their container prefixes.
                line.chars().all(|c| c.is_whitespace() || c == '>')
            })
            .count();
        if count > 0 && !matches!(stack.last(), Some(Frame::List { .. })) {
            accept_block(stack, Block::BlankLines(count));
        }
        self.cursor = end;
    }
}
