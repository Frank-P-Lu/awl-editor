//! Markdown span kinds and parser output for renderer attributes.
//!
//! Split by subject into this directory: [`kind`] is the `MdKind`/`BreakKind`
//! vocabulary, [`detect`] the pure per-line detectors (`is_thematic_break`,
//! `list_item`, `fence_line_lang`, word/reading-time counting), [`parse`] the
//! pulldown-cmark walk (`spans`) itself, and [`markers`] the per-construct span
//! pushers `spans` calls into.

mod detect;
mod footnotes;
mod kind;
mod markers;
mod parse;
pub use detect::{
    LIST_INDENT, ListItem, READING_WPM, fence_line_lang, frontmatter_end, is_thematic_break,
    list_item, reading_time_min, strike_engaged, word_count,
};
pub use kind::{BreakKind, MdKind, break_kind};
pub use markers::equals_runs;
#[cfg(test)]
pub(super) use markers::push_highlight_spans;
pub use parse::spans;
