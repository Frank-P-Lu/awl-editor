//! Personal-dictionary picker construction and row retirement.

use super::{OverlayKind, OverlayState};

impl OverlayState {
    /// The words the personal dictionary holds, one row each, in the order the
    /// caller gathered them (`SpellChecker::user_words_sorted` — alphabetical).
    ///
    /// The word IS the accept string, so `↵` on a row hands the effect exactly
    /// the text to forget; there is no per-row payload and therefore no
    /// `RowMeta` of its own.
    pub fn new_user_words(words: Vec<String>) -> Self {
        let n = words.len();
        Self::new_marked(
            OverlayKind::UserWords,
            words,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        )
    }

    /// Retire one word's row from the OPEN card, mirroring
    /// [`Self::remove_asset_row`]: this picker's accept is destructive and keeps
    /// the card up, so the list has to shrink under the highlight rather than
    /// being rebuilt from a fresh scan.
    pub fn remove_user_word_row(&mut self, word: &str) -> bool {
        let Some(index) = self.rows.iter().position(|row| row.accept == word) else {
            return false;
        };
        self.rows.remove(index);
        self.refilter();
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        true
    }
}
