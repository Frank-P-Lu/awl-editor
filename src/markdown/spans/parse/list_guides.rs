//! Guide-only ancestry: deliberately independent of rich-marker concealment.
use super::*;
use pulldown_cmark::{Event, Tag, TagEnd};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ListGuideLine {
    pub line: usize,
    pub source_indent: usize,
    pub marker: bool,
    /// Midpoints between successive, parser-confirmed parent/child columns.
    pub columns: Vec<f32>,
    /// Source rows of the parser-confirmed parent markers, in column order.
    pub parents: Vec<usize>,
}

pub(super) struct ListGuideCollector<'a> {
    text: &'a str,
    offset: usize,
    starts: Vec<usize>,
    items: Vec<Option<(usize, usize)>>,
    blocked: usize,
    lines: std::collections::BTreeMap<usize, ListGuideLine>,
}

impl<'a> ListGuideCollector<'a> {
    pub fn new(text: &'a str, offset: usize) -> Self {
        Self {
            text,
            offset,
            starts: std::iter::once(0)
                .chain(text.match_indices('\n').map(|(i, _)| i + 1))
                .collect(),
            items: Vec::new(),
            blocked: 0,
            lines: Default::default(),
        }
    }

    pub fn on_event(&mut self, event: &Event<'_>, range: &Range<usize>) {
        match event {
            Event::Start(Tag::Item) => {
                let line = self.line_at(range.start);
                let item = list_item(self.line(line));
                self.items
                    .push(item.map(|it| (line + self.offset, it.indent)));
                if self.blocked == 0 {
                    self.record(line, true);
                }
            }
            Event::End(TagEnd::Item) => {
                self.items.pop();
            }
            Event::Start(
                Tag::CodeBlock(_) | Tag::BlockQuote(_) | Tag::Heading { .. } | Tag::Table(_),
            ) => {
                self.blocked += 1;
                // Item Start precedes its opening block: retract that marker
                // row when the first content is itself an excluded construct.
                self.lines
                    .remove(&(self.line_at(range.start) + self.offset));
            }
            Event::End(
                TagEnd::CodeBlock | TagEnd::BlockQuote(_) | TagEnd::Heading(_) | TagEnd::Table,
            ) => self.blocked = self.blocked.saturating_sub(1),
            Event::Text(_)
            | Event::Code(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
                if self.blocked == 0 && !range.is_empty() =>
            {
                for line in self.line_at(range.start)..=self.line_at(range.end - 1) {
                    self.record(line, false);
                }
            }
            _ => {}
        }
    }

    fn line_at(&self, byte: usize) -> usize {
        self.starts
            .partition_point(|start| *start <= byte)
            .saturating_sub(1)
    }

    fn line(&self, index: usize) -> &'a str {
        &self.text[self.starts[index]
            ..self
                .starts
                .get(index + 1)
                .map_or(self.text.len(), |next| next - 1)]
    }

    fn record(&mut self, line: usize, marker: bool) {
        // An unrecognized ancestor is a barrier, never an invented depth.
        if self.items.len() < 2 || self.items.iter().any(Option::is_none) {
            return;
        }
        let columns = self
            .items
            .windows(2)
            .filter_map(|pair| {
                let ((_, parent), (_, child)) = (pair[0]?, pair[1]?);
                (child > parent).then_some((parent + child) as f32 * 0.5)
            })
            .collect::<Vec<_>>();
        if columns.is_empty() {
            return;
        }
        let parents = self
            .items
            .windows(2)
            .filter_map(|pair| {
                let ((line, parent), (_, child)) = (pair[0]?, pair[1]?);
                (child > parent).then_some(line)
            })
            .collect();
        let source_indent = self
            .line(line)
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
        self.lines
            .entry(line + self.offset)
            .and_modify(|entry| entry.marker |= marker)
            .or_insert(ListGuideLine {
                line: line + self.offset,
                source_indent,
                marker,
                columns,
                parents,
            });
    }

    pub fn finish(self) -> Vec<ListGuideLine> {
        self.lines.into_values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guides_follow_parser_ancestry_across_mixed_items() {
        let _serial = crate::testlock::serial();
        let text = concat!(
            "10. Parent\n    - Child\n      - [ ] Task\n",
            "        continuation\n    - Sibling\n\nOutside\n",
        );
        let (_, _, guides) = spans_with_list_layout(text);
        assert_eq!(
            guides
                .iter()
                .map(|g| (g.line, g.columns.clone()))
                .collect::<Vec<_>>(),
            vec![
                (1, vec![2.0]),
                (2, vec![2.0, 5.0]),
                (3, vec![2.0, 5.0]),
                (4, vec![2.0]),
            ]
        );
    }

    #[test]
    fn guides_exclude_code_quotes_and_false_indentation() {
        let _serial = crate::testlock::serial();
        for text in [
            "  - Root\n",
            "```\n- Parent\n  - Fake\n```\n",
            "> - Parent\n>   - Child\n",
        ] {
            assert!(spans_with_list_layout(text).2.is_empty(), "{text}");
        }
        let guides = spans_with_list_layout("- Parent\n  - Child\nlazy prose\n").2;
        assert_eq!(guides[1].source_indent, 0);
        assert_eq!(guides[1].columns, vec![1.0]);
    }

    #[test]
    fn guides_exclude_items_opening_with_block_constructs() {
        let _serial = crate::testlock::serial();
        for text in [
            "- Parent\n  - # Heading\n",
            "- Parent\n  - > Quote\n",
            "- Parent\n  - ```\n    code\n    ```\n",
        ] {
            assert!(spans_with_list_layout(text).2.is_empty(), "{text}");
        }
    }

    #[test]
    fn empty_tasks_and_frontmatter_keep_document_coordinates() {
        let _serial = crate::testlock::serial();
        let guides =
            spans_with_list_layout("---\ntitle: Test\n---\n- Parent\n  - [ ] \n  - [x] Done\n").2;
        assert_eq!(
            guides.iter().map(|g| g.line).collect::<Vec<_>>(),
            vec![4, 5]
        );
        assert!(guides.iter().all(|g| g.marker && g.columns == vec![1.0]));
    }
}
