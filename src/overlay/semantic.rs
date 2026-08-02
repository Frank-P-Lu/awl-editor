//! What a picker owes a semantic consumer: stable row identity, and a way to
//! set the query as a whole rather than one keystroke at a time.

use super::OverlayState;

impl OverlayState {
    /// Replace the query outright — the semantic `SetValue` a screen reader or
    /// an agent sends, which arrives as a finished string rather than as the
    /// per-character `push`/`pop` the keyboard path uses. Resets the selection
    /// and scroll exactly as typing into an empty box would.
    pub(crate) fn set_query_text(&mut self, value: &str) {
        self.query = crate::textbox::TextBox::seeded(value);
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    /// Stable corpus identities for the currently visible rows, parallel to
    /// [`Self::item_strings`]. A semantic consumer must never key a row by its
    /// filtered display position: typing one character would rename every row
    /// below the first match and make assistive focus jump to another control.
    pub fn item_corpus_indices(&self) -> &[usize] {
        &self.items
    }
}
