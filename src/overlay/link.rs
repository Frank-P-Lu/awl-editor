/// What a committed URL is applied to. The mode is decided once when Cmd-K is
/// pressed and carried unchanged through the minibuffer flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkEditMode {
    /// Wrap literal selected prose, escaping it as a Markdown label first.
    WithText {
        start: usize,
        end: usize,
        text: String,
    },
    /// Rewrite an existing destination while retaining its raw label source.
    Existing {
        start: usize,
        end: usize,
        source_text: String,
    },
    /// Insert empty markup at `at`; the caret lands between its brackets.
    Empty { at: usize },
}
