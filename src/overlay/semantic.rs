//! What a picker owes a semantic consumer: stable row identity, and a way to
//! set the query as a whole rather than one keystroke at a time.
//!
//! The browser build carries none of this. The snapshot TYPES are shared —
//! a capture sidecar's `semantic` field is part of the schema on every
//! platform — but every producer and consumer is native: the AccessKit
//! adapter, the live-`App` fold, and `--semantic-json`. Web accessibility
//! needs a DOM mirror behind the canvas (AccessKit has no canvas adapter), so
//! these are legitimately unused there rather than unfinished.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

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
