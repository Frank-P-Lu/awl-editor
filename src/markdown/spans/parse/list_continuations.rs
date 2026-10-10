//! Parser-owned continuation lines collected beside the styling event fold.

use super::*;
use pulldown_cmark::{Event, Tag, TagEnd};

#[derive(Clone, Copy)]
struct ListItemContext {
    marker_line: usize,
    /// `None` is an ownership barrier for an ordered or unrecognized item.
    marker_col: Option<usize>,
}

pub(super) struct ListContinuationCollector<'a> {
    text: &'a str,
    body_offset: usize,
    line_offset: usize,
    line_starts: Vec<usize>,
    items: Vec<ListItemContext>,
    continuations: std::collections::BTreeMap<usize, ListContinuation>,
    code_blocks: u32,
    quotes: u32,
    headings: u32,
    tables: u32,
}

impl<'a> ListContinuationCollector<'a> {
    pub(super) fn new(text: &'a str, body_offset: usize, line_offset: usize) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Self {
            text,
            body_offset,
            line_offset,
            line_starts,
            items: Vec::new(),
            continuations: std::collections::BTreeMap::new(),
            code_blocks: 0,
            quotes: 0,
            headings: 0,
            tables: 0,
        }
    }

    /// Observe the exact event already consumed by the styling fold. Inline
    /// leaf ranges cover prose of every shape without a second Markdown parse.
    pub(super) fn on_event(&mut self, event: &Event<'_>, range: &Range<usize>) {
        match event {
            Event::Start(Tag::Item) => self.start_item(range.start),
            Event::End(TagEnd::Item) => {
                self.items.pop();
            }
            Event::Start(Tag::CodeBlock(_)) => self.code_blocks += 1,
            Event::End(TagEnd::CodeBlock) => self.code_blocks = self.code_blocks.saturating_sub(1),
            Event::Start(Tag::BlockQuote(_)) => self.quotes += 1,
            Event::End(TagEnd::BlockQuote(_)) => self.quotes = self.quotes.saturating_sub(1),
            Event::Start(Tag::Heading { .. }) => self.headings += 1,
            Event::End(TagEnd::Heading(_)) => self.headings = self.headings.saturating_sub(1),
            Event::Start(Tag::Table(_)) => self.tables += 1,
            Event::End(TagEnd::Table) => self.tables = self.tables.saturating_sub(1),
            Event::Text(_)
            | Event::Code(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
                if self.accepts_prose() =>
            {
                self.record_prose_lines(range);
            }
            _ => {}
        }
    }

    pub(super) fn finish(self) -> Vec<ListContinuation> {
        self.continuations.into_values().collect()
    }

    fn accepts_prose(&self) -> bool {
        !self.items.is_empty()
            && self.code_blocks == 0
            && self.quotes == 0
            && self.headings == 0
            && self.tables == 0
    }

    fn start_item(&mut self, byte: usize) {
        let marker_line = line_index_at(&self.line_starts, byte);
        let line_start = self.line_starts[marker_line];
        let line_end = self
            .line_starts
            .get(marker_line + 1)
            .map_or(self.text.len(), |next| next.saturating_sub(1));
        let marker_col = list_item(&self.text[line_start..line_end])
            .filter(|item| !item.ordered)
            .map(|item| item.indent);
        self.items.push(ListItemContext {
            marker_line,
            marker_col,
        });
    }

    fn record_prose_lines(&mut self, range: &Range<usize>) {
        let Some(owner) = self.items.last().copied() else {
            return;
        };
        let Some(marker_col) = owner.marker_col else {
            return;
        };
        if range.is_empty() {
            return;
        }
        let first = line_index_at(&self.line_starts, range.start);
        let last = line_index_at(&self.line_starts, range.end.saturating_sub(1));
        for body_line in first..=last {
            if body_line == owner.marker_line {
                continue;
            }
            let line_start = self.line_starts[body_line];
            let line_end = self
                .line_starts
                .get(body_line + 1)
                .map_or(self.text.len(), |next| next.saturating_sub(1));
            let line = &self.text[line_start..line_end];
            if line.trim().is_empty() || list_item(line).is_some() {
                continue;
            }
            let source_indent = line
                .as_bytes()
                .iter()
                .take_while(|byte| matches!(**byte, b' ' | b'\t'))
                .count();
            self.continuations
                .entry(self.body_offset + line_start)
                .or_insert(ListContinuation {
                    line: self.line_offset + body_line,
                    line_doc_start: self.body_offset + line_start,
                    source_indent,
                    marker_col,
                });
        }
    }
}

fn line_index_at(line_starts: &[usize], byte: usize) -> usize {
    line_starts
        .partition_point(|start| *start <= byte)
        .saturating_sub(1)
}
