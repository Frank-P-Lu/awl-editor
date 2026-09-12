//! Parsed enrollment for rich unordered-list and task ornaments.

use super::{detect::list_item, kind::MdKind};
use std::ops::Range;

/// One unordered list row that the rich-preview renderer may replace with an
/// ornament. The raw line scanner supplies the marker geometry, while parsed
/// [`MdKind::ListMarker`] membership proves that the same bytes are markdown
/// rather than list-shaped text inside a code block. A task state is attached
/// only when its parsed marker begins exactly where this list prefix ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RichListItem {
    pub marker_col: usize,
    pub depth: usize,
    pub task: Option<bool>,
}

/// Classify an unordered list row for rich preview through one shared owner.
/// Concealment and ornament painting both call this function, so either both
/// replace the source marker or neither does.
pub fn rich_unordered_list_item<'a>(
    line: &str,
    line_doc_start: usize,
    spans: impl IntoIterator<Item = &'a (Range<usize>, MdKind)>,
) -> Option<RichListItem> {
    let item = list_item(line)?;
    if item.ordered {
        return None;
    }
    let prefix_end = line_doc_start + item.content;
    let line_end = line_doc_start + line.len();
    let mut parsed_list_marker = false;
    let mut task = None;
    for (range, kind) in spans {
        match *kind {
            MdKind::ListMarker if range.start == line_doc_start && range.end >= prefix_end => {
                parsed_list_marker = true;
            }
            MdKind::Task(checked) if range.start == prefix_end && range.end <= line_end => {
                task = Some(checked);
            }
            _ => {}
        }
    }
    parsed_list_marker.then_some(RichListItem {
        marker_col: item.indent,
        depth: item.depth(),
        task,
    })
}
