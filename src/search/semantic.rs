//! What the find/replace panel owes a semantic consumer: whole-string writes
//! into either field, which is the shape a `SetValue` request arrives in.
//!
//! The browser build carries none of this. The snapshot TYPES are shared —
//! a capture sidecar's `semantic` field is part of the schema on every
//! platform — but every producer and consumer is native: the AccessKit
//! adapter, the live-`App` fold, and `--semantic-json`. Web accessibility
//! needs a DOM mirror behind the canvas (AccessKit has no canvas adapter), so
//! these are legitimately unused there rather than unfinished.
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use super::{SearchState, TextBox};

impl SearchState {
    /// Replace the query outright and re-run the search, exactly as if the
    /// whole string had been typed. Moves editing focus to the query, because
    /// that is the field the caller just wrote.
    pub(crate) fn set_query_text(&mut self, value: &str, haystack: &str) {
        self.query = TextBox::seeded(value);
        self.editing_replacement = false;
        self.recompute(haystack);
    }

    /// Replace the replacement text outright. No re-search: the replacement is
    /// not part of the match query.
    pub(crate) fn set_replacement_text(&mut self, value: &str) {
        self.replacement = TextBox::seeded(value);
        self.editing_replacement = true;
    }
}
