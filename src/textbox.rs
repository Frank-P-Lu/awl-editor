//! ONE SHARED SINGLE-LINE TEXTBOX MODEL: text + CHAR-index caret +
//! motion/edit/word rules, shared by the 7 end-only single-line fields this
//! model routes through it — picker query, Rename, Insert-link URL,
//! Keep-version name, Settings value, Find query, Replace text (see
//! [`TextField::ALL`]). Pure text + caret + motion — NO char filtering (a
//! Settings digit/`.`/`%` gate, a Rename `/`-reject), NO refilter/recompute/
//! commit; those stay owned by each surface (`overlay::capture`,
//! `overlay::nav`, `search::mod` respectively).
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
//! `_forward` (and the minibuffer word-delete,
//! `overlay::nav::truncate_trailing_word`) already share. Wiring motion to
//! the delete rule (or vice versa) makes a textbox's opt-arrow disagree with
//! the document's own M-b/M-f.

use crate::buffer::{
    word_backward_boundary, word_delete_backward_boundary, word_delete_forward_boundary,
    word_forward_boundary,
};

/// A single-line text field: its content plus a CHAR-index caret, plus an
/// OPTIONAL selection anchor. Shared by every end-only minibuffer field (see
/// the module doc) so motion/edit/word rules exist in exactly ONE place —
/// "same behavior ⇒ same code".
///
/// SELECTION MECHANICS (minimal by design — only [`Self::seeded_selecting_prefix`]
/// ever arms one today, for the Rename minibuffer's seeded stem): `anchor` is
/// the selection's OTHER edge, `caret` is always the ACTIVE edge. The
/// invariant every mutator below upholds is structural, not defensive:
/// `anchor` is `None` whenever there is nothing selected, and is NEVER left
/// equal to `caret` (a zero-width "selection" is the same as no selection, so
/// every op that would produce one clears `anchor` instead). Two rules cover
/// every existing method:
///   * an EDIT op (`insert`, `delete_back`, `delete_forward`,
///     `delete_word_back`, `delete_word_forward`) deletes the active
///     selection FIRST (the file-manager "type/delete replaces the
///     selection" convention) and never compounds it with its own char/word
///     rule;
///   * a MOTION op (`char_left`/`right`, `word_left`/`right`, `set_caret`)
///     COLLAPSES an active selection to the edge the motion implies, rather
///     than stepping further from either edge.
///
/// A field with no selection (every one of the other 6 minibuffer fields, and
/// Rename itself after its first keystroke) never sets `anchor`, so every
/// rule above is a no-op for it — this is purely additive.
///
/// Deliberately UNIMPLEMENTED (no current caller needs it): shift-to-extend,
/// mouse-drag-select, copy/cut of a selection. Adding one of those later is a
/// new caller of `anchor`, not a change to the mechanics above.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    /// CHAR index into `text`, always in `0..=text.chars().count()`. The
    /// ACTIVE selection edge (see the struct doc) when `anchor` is `Some`.
    caret: usize,
    /// The selection's OTHER edge, or `None` when nothing is selected. Never
    /// equal to `caret` — see the struct doc's invariant.
    anchor: Option<usize>,
}

impl TextBox {
    /// An empty field, caret at 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// A field pre-filled with `s`, caret at the END — the seeding every
    /// existing seeded minibuffer uses (Rename / Insert-link /
    /// Settings all start from the current value, caret ready to backspace
    /// it; only Keep-version seeds empty, via [`Self::new`]).
    pub fn seeded(s: &str) -> Self {
        Self {
            text: s.to_string(),
            caret: s.chars().count(),
            anchor: None,
        }
    }

    /// A field pre-filled with `s`, with characters `[0, prefix_len)`
    /// SELECTED (anchor at 0) and the caret at `prefix_len` — the Rename
    /// minibuffer's file-manager convention: the editable STEM arrives
    /// pre-selected so the very first keystroke replaces it outright, while
    /// an untouched tail (a file extension) sits past the caret rather than
    /// inside the selection. `prefix_len` is CLAMPED to `s`'s char count. A
    /// `prefix_len` of `0` seeds NO selection (`anchor` stays `None`) — the
    /// struct doc's "never a zero-width selection" invariant, kept here at
    /// the one place a selection is born rather than papered over later.
    pub fn seeded_selecting_prefix(s: &str, prefix_len: usize) -> Self {
        let caret = prefix_len.min(s.chars().count());
        Self {
            text: s.to_string(),
            caret,
            anchor: (caret > 0).then_some(0),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn caret(&self) -> usize {
        self.caret
    }

    /// The active selection as CHAR indices `(start, end)`, `start <= end`,
    /// or `None` when there is none. Never `Some((x, x))` — see the struct
    /// doc's invariant; a caller never has to special-case a zero-width
    /// range.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        (a != self.caret).then(|| (a.min(self.caret), a.max(self.caret)))
    }

    /// Deletes the active selection (if any), leaving the caret at its START
    /// with no anchor. Every EDIT op below calls this FIRST — the
    /// file-manager "type/delete replaces the selection" rule — and returns
    /// early on `true` rather than compounding with its own char/word rule.
    fn delete_selection_if_any(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        let sb = self.byte_of(start);
        let eb = self.byte_of(end);
        self.text.replace_range(sb..eb, "");
        self.caret = start;
        self.anchor = None;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn len_chars(&self) -> usize {
        self.text.chars().count()
    }

    /// Move the caret to `at`, CLAMPED to `[0, len_chars]` — never panics on
    /// an out-of-range request. The click-to-place / drag-scrub door for the
    /// picker query field ([`crate::overlay::OverlayState::query_set_caret`])
    /// and the mid-query Home/End split ([`crate::overlay::OverlayState::
    /// query_home`] / `query_end`).
    pub fn set_caret(&mut self, at: usize) {
        self.caret = at.min(self.len_chars());
        self.anchor = None;
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
        self.delete_selection_if_any();
        let b = self.byte_of(self.caret);
        self.text.insert(b, c);
        self.caret += 1;
    }

    /// Backspace: delete the CHARACTER before the caret — one extended
    /// grapheme cluster, through the same [`crate::grapheme`] owner the
    /// document buffer's own Backspace uses. A no-op at the start. Replaces
    /// (never compounds with) an active selection — see the struct doc.
    pub fn delete_back(&mut self) {
        if self.delete_selection_if_any() {
            return;
        }
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
        if self.delete_selection_if_any() {
            return;
        }
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

    /// One CHARACTER left — one grapheme cluster. COLLAPSES an active
    /// selection to its start instead of stepping past it — see the struct
    /// doc's MOTION rule.
    pub fn char_left(&mut self) {
        if let Some((start, _)) = self.selection_range() {
            self.caret = start;
            self.anchor = None;
            return;
        }
        self.caret = self.prev_boundary();
    }

    /// One CHARACTER right — one grapheme cluster. Collapses an active
    /// selection to its end, mirroring [`Self::char_left`].
    pub fn char_right(&mut self) {
        if let Some((_, end)) = self.selection_range() {
            self.caret = end;
            self.anchor = None;
            return;
        }
        self.caret = self.next_boundary();
    }

    /// WORD motion right — delegates to the SAME boundary rule
    /// [`Buffer::forward_word`](crate::buffer::Buffer::forward_word) uses
    /// (skip non-word, then skip word). NEVER the word-DELETE boundary — see
    /// the module doc's "two word rules" trap. Collapses an active selection
    /// to its end rather than stepping a word past it, mirroring
    /// [`Self::char_right`].
    pub fn word_right(&mut self) {
        if let Some((_, end)) = self.selection_range() {
            self.caret = end;
            self.anchor = None;
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        self.caret = word_forward_boundary(self.caret, chars.len(), |i| chars[i]);
    }

    /// WORD motion left — the exact mirror of [`Self::word_right`],
    /// including its selection-collapse rule.
    pub fn word_left(&mut self) {
        if let Some((start, _)) = self.selection_range() {
            self.caret = start;
            self.anchor = None;
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        self.caret = word_backward_boundary(self.caret, |i| chars[i]);
    }

    /// WORD delete backward (⌥⌫) — the SAME token-class rule the document
    /// buffer's `delete_word_backward` uses, NOT the motion rule above.
    pub fn delete_word_back(&mut self) {
        if self.delete_selection_if_any() {
            return;
        }
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
        if self.delete_selection_if_any() {
            return;
        }
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
