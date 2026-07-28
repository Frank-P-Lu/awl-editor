//! ITEM 10 — ONE SHARED SINGLE-LINE TEXTBOX MODEL: text + CHAR-index caret +
//! motion/edit/word rules, shared by the 7 end-only single-line fields this
//! item routes through it — picker query, Rename, Insert-link URL,
//! Keep-version name, Settings value, Find query, Replace text (see
//! [`TextField::ALL`]). Pure text + caret + motion — NO char filtering (a
//! Settings digit/`.`/`%` gate, a Rename `/`-reject), NO refilter/recompute/
//! commit; those stay owned by each surface (`overlay::capture`,
//! `overlay::nav`, `search::mod` respectively) exactly as before item 10.
//!
//! CHAR-INDEX DISCIPLINE: `caret` is a CHAR index into `text`
//! (`0..=text.chars().count()`), NEVER a byte offset — `String::insert` /
//! `replace_range` take BYTE indices, so every splice below converts
//! explicitly via `char_indices` (`Self::byte_of`). This is the Unicode trap
//! the parity tests in `textbox/tests.rs` guard: CJK / combining marks / emoji
//! are multi-byte in UTF-8, and a byte offset used as a caret position panics
//! (or silently splits a multibyte char) the first time a field holds one. The
//! caret STEP is a separate rule again — one extended grapheme cluster, owned
//! by [`crate::grapheme`] and shared with the document buffer.
//!
//! TWO DISTINCT WORD RULES, never conflated (see `buffer.rs`'s own doc on
//! [`crate::buffer::word_delete_backward_boundary`]): word MOTION
//! ([`TextBox::word_left`] / [`TextBox::word_right`], Ctrl/Opt-arrow)
//! delegates to the SAME [`crate::buffer::word_forward_boundary`] /
//! [`crate::buffer::word_backward_boundary`] free fns the document buffer's
//! own `Buffer::forward_word` / `backward_word` use; word DELETE
//! ([`TextBox::delete_word_back`] / [`TextBox::delete_word_forward`],
//! Opt-Backspace / Opt-forward-Delete) delegates to the SEPARATE
//! `word_delete_*_boundary` owners the document's `delete_word_backward` /
//! `_forward` (and the pre-item-10 minibuffer word-delete,
//! `overlay::nav::truncate_trailing_word`) already share. Wiring motion to
//! the delete rule (or vice versa) makes a textbox's opt-arrow disagree with
//! the document's own M-b/M-f — the item's headline trap.

use crate::buffer::{
    word_backward_boundary, word_delete_backward_boundary, word_delete_forward_boundary,
    word_forward_boundary,
};

/// A single-line text field: its content plus a CHAR-index caret. Shared by
/// every end-only minibuffer field (see the module doc) so motion/edit/word
/// rules exist in exactly ONE place — "same behavior ⇒ same code".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    /// CHAR index into `text`, always in `0..=text.chars().count()`.
    caret: usize,
}

impl TextBox {
    /// An empty field, caret at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// A field pre-filled with `s`, caret at the END — the seeding every
    /// existing minibuffer used before item 10 (Rename / Insert-link /
    /// Settings all start from the current value, caret ready to backspace
    /// it; only Keep-version seeds empty, via [`Self::new`]).
    pub fn seeded(s: &str) -> Self {
        Self {
            text: s.to_string(),
            caret: s.chars().count(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn len_chars(&self) -> usize {
        self.text.chars().count()
    }

    /// Move the caret to `at`, CLAMPED to `[0, len_chars]` — never panics on
    /// an out-of-range request. Not yet wired to a live surface (no field
    /// currently jumps its caret to an arbitrary position); exercised by the
    /// parity/unit tests and kept for a future click-to-place caller.
    #[allow(dead_code)]
    pub fn set_caret(&mut self, at: usize) {
        self.caret = at.min(self.len_chars());
    }

    /// The BYTE offset of CHAR index `idx` within `text` (`idx` may equal
    /// `len_chars()`, yielding `text.len()`) — the ONE char->byte conversion
    /// every splice below routes through, so a multibyte field (CJK /
    /// combining / emoji) never panics on a byte-misaligned
    /// `String::insert` / `replace_range` (the Unicode trap this module's
    /// doc names).
    fn byte_of(&self, idx: usize) -> usize {
        if idx == 0 {
            return 0;
        }
        self.text
            .char_indices()
            .nth(idx)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    /// Insert `c` at the caret and advance past it. Accepts ANY char — no
    /// filtering (a Settings digit gate / Rename `/`-reject is the CALLER's
    /// job, applied before this is reached; see the module doc).
    pub fn insert(&mut self, c: char) {
        let b = self.byte_of(self.caret);
        self.text.insert(b, c);
        self.caret += 1;
    }

    /// Backspace: delete the CHARACTER before the caret — one extended
    /// grapheme cluster, through the same [`crate::grapheme`] owner the
    /// document buffer's own Backspace uses. A no-op at the start.
    pub fn delete_back(&mut self) {
        if self.caret == 0 {
            return;
        }
        let new_caret = self.prev_boundary();
        let end = self.byte_of(self.caret);
        let start = self.byte_of(new_caret);
        self.text.replace_range(start..end, "");
        self.caret = new_caret;
    }

    /// Forward-delete: remove the CHARACTER at the caret — one extended
    /// grapheme cluster, same owner as [`Self::delete_back`]. A no-op at the
    /// end. Not yet wired to a live surface (none of the 7 fields bind a plain
    /// forward-Delete — only the word-delete variant, `delete_word_forward`,
    /// is claimed); kept for API completeness + its boundary-safety test.
    #[allow(dead_code)]
    pub fn delete_forward(&mut self) {
        if self.caret >= self.len_chars() {
            return;
        }
        let start = self.byte_of(self.caret);
        let end = self.byte_of(self.next_boundary());
        self.text.replace_range(start..end, "");
        // Caret unchanged: the following char slides up to meet it.
    }

    /// The next / previous extended-grapheme-cluster boundary around the caret,
    /// from the ONE owner ([`crate::grapheme`]) the document
    /// [`Buffer`](crate::buffer::Buffer) steps by — so a minibuffer's arrows
    /// and Backspace cross a combining pair or an emoji ZWJ sequence exactly as
    /// the document's do. `textbox/tests.rs`'s parity table is where that
    /// agreement is proven.
    fn next_boundary(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        crate::grapheme::next_cluster_boundary(self.caret, chars.len(), |i| chars[i])
    }

    fn prev_boundary(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        crate::grapheme::prev_cluster_boundary(self.caret, |i| chars[i])
    }

    /// One CHARACTER left — one grapheme cluster.
    pub fn char_left(&mut self) {
        self.caret = self.prev_boundary();
    }

    /// One CHARACTER right — one grapheme cluster.
    pub fn char_right(&mut self) {
        self.caret = self.next_boundary();
    }

    /// WORD motion right — delegates to the SAME boundary rule
    /// [`Buffer::forward_word`](crate::buffer::Buffer::forward_word) uses
    /// (skip non-word, then skip word). NEVER the word-DELETE boundary — see
    /// the module doc's "two word rules" trap.
    pub fn word_right(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        self.caret = word_forward_boundary(self.caret, chars.len(), |i| chars[i]);
    }

    /// WORD motion left — the exact mirror of [`Self::word_right`].
    pub fn word_left(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        self.caret = word_backward_boundary(self.caret, |i| chars[i]);
    }

    /// WORD delete backward (⌥⌫) — the SAME token-class rule the document
    /// buffer's `delete_word_backward` uses, NOT the motion rule above.
    pub fn delete_word_back(&mut self) {
        if self.caret == 0 {
            return;
        }
        // Only the chars BEFORE the caret matter to the backward rule.
        let chars: Vec<char> = self.text.chars().take(self.caret).collect();
        let new_caret = word_delete_backward_boundary(self.caret, |i| chars[i]);
        let start = self.byte_of(new_caret);
        let end = self.byte_of(self.caret);
        self.text.replace_range(start..end, "");
        self.caret = new_caret;
    }

    /// WORD delete forward (⌥+forward-Delete) — the exact mirror of
    /// [`Self::delete_word_back`].
    pub fn delete_word_forward(&mut self) {
        let chars: Vec<char> = self.text.chars().collect();
        let len = chars.len();
        if self.caret >= len {
            return;
        }
        let stop = word_delete_forward_boundary(self.caret, len, |i| chars[i]);
        let start = self.byte_of(self.caret);
        let end = self.byte_of(stop);
        self.text.replace_range(start..end, "");
        // Caret unchanged: it sits at the start of what was just deleted.
    }
}

impl PartialEq<&str> for TextBox {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl PartialEq<str> for TextBox {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}

enum_with_all! {
    /// Every end-only single-line surface routes through [`TextBox`].
    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextField {
        /// The summoned-picker fuzzy query (Goto / Command / Theme / …).
        PickerQuery,
        /// The Rename minibuffer's typed filename.
        Rename,
        /// The Cmd-K Insert-link minibuffer's typed URL.
        InsertLink,
        /// The "Keep version…" minibuffer's typed (optional) name.
        KeepVersion,
        /// The Settings menu's inline numeric VALUE edit (page width / zoom).
        SettingsValue,
        /// The find/replace panel's search query.
        FindQuery,
        /// The find/replace panel's replacement text.
        ReplaceText,
    }
}

#[cfg(test)]
mod tests;
