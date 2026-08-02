//! Cache, spell, and persistence state transitions for the active slot.

use super::*;

impl DocumentSession {
    pub(in crate::app) fn shift_selecting(&self) -> bool {
        self.active.extra.shift_selecting
    }

    pub(in crate::app) fn set_shift_selecting(&mut self, value: bool) {
        self.active.extra.shift_selecting = value;
    }

    pub(in crate::app) fn scroll(&self) -> crate::render::ScrollPos {
        self.active.extra.scroll
    }

    pub(in crate::app) fn set_scroll(&mut self, scroll: crate::render::ScrollPos) {
        self.active.extra.scroll = scroll;
    }

    pub(in crate::app) fn spell_cache(&self) -> &[crate::spell::SpellVerdict] {
        &self.active.extra.spell_cache
    }

    pub(in crate::app) fn spell_checked_version(&self) -> Option<u64> {
        self.active.extra.spell_checked_version
    }

    pub(in crate::app) fn invalidate_spell_cache(&mut self) {
        self.active.extra.spell_checked_version = None;
    }

    pub(in crate::app) fn recompute_spell_cache(&mut self) {
        let Some(spell) = self.spell.as_ref() else {
            return;
        };
        let text = self.active.buffer.text();
        let spans = spell.misspellings_for(&text, self.active.buffer.syntax_lang());
        self.active.extra.spell_cache = crate::spell::keyed(&text, spans);
        self.active.extra.spell_checked_version = Some(self.active.buffer.version());
    }

    pub(in crate::app) fn spell_enabled(&self) -> bool {
        self.spell.is_some()
    }

    #[cfg(test)]
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn spell_check(&self, word: &str) -> Option<bool> {
        self.spell.as_ref().map(|spell| spell.check(word))
    }

    pub(in crate::app) fn spell_suggestion_target(
        &self,
        line: usize,
        col: usize,
    ) -> Option<crate::spell::SuggestionTarget> {
        let spell = self.spell.as_ref()?;
        spell.suggest_at(
            &self.active.buffer.text(),
            line,
            col,
            self.active.buffer.syntax_lang(),
        )
    }

    pub(in crate::app) fn replace_spell_checker(&mut self, variant: crate::spell::DictVariant) {
        self.spell = match crate::spell::SpellChecker::new(variant) {
            Ok(sc) => Some(sc),
            Err(e) => {
                eprintln!("dictionary switch failed: {e}");
                None
            }
        };
        self.invalidate_spell_cache();
    }

    pub(in crate::app) fn set_user_words(&mut self, words: Vec<String>) {
        if let Some(spell) = self.spell.as_mut() {
            spell.set_user_words(words);
        }
    }

    pub(in crate::app) fn add_user_word(&mut self, word: &str) -> bool {
        self.spell
            .as_mut()
            .map(|spell| spell.add_user_word(word))
            .unwrap_or(false)
    }

    pub(in crate::app) fn sync_text(&mut self) -> String {
        let version = self.active.buffer.version();
        match &self.active.extra.sync_text_cache {
            Some((cached, text)) if *cached == version => text.clone(),
            _ => {
                let text = self.active.buffer.text();
                self.active.extra.sync_text_cache = Some((version, text.clone()));
                text
            }
        }
    }

    pub(in crate::app) fn caret_was_synced_at(&mut self, version: u64) -> bool {
        let changed = self.active.extra.caret_synced_version != version;
        self.active.extra.caret_synced_version = version;
        changed
    }

    pub(in crate::app) fn doc_saved_version(&self) -> Option<u64> {
        self.active.extra.doc_saved_version
    }

    pub(in crate::app) fn scratch_saved_version(&self) -> Option<u64> {
        self.active.extra.scratch_saved_version
    }

    pub(in crate::app) fn disk_baseline(&self) -> crate::external::Seen {
        self.active.extra.disk_baseline
    }

    pub(in crate::app) fn scratch_baseline(&self) -> crate::external::Seen {
        self.active.extra.scratch_baseline
    }

    /// ADOPT a fresh observation of the document's path WITHOUT claiming the
    /// buffer was saved. The clean-reload and the conflict-resolution doors both
    /// need this: they have looked at the disk and must not be told about the
    /// same change twice, but the buffer's saved-version bookkeeping is theirs
    /// to set separately.
    pub(in crate::app) fn adopt_disk_baseline(&mut self, seen: crate::external::Seen) {
        self.active.extra.disk_baseline = seen;
    }

    pub(in crate::app) fn record_document_saved(
        &mut self,
        version: u64,
        seen: crate::external::Seen,
    ) {
        self.active.extra.doc_saved_version = Some(version);
        self.active.extra.disk_baseline = seen;
        self.active.extra.caret_synced_version = version;
    }

    pub(in crate::app) fn acknowledge_document_version(&mut self, version: u64) {
        self.active.extra.doc_saved_version = Some(version);
    }

    pub(in crate::app) fn record_scratch_saved(
        &mut self,
        version: u64,
        seen: crate::external::Seen,
    ) {
        self.active.extra.scratch_saved_version = Some(version);
        self.active.extra.scratch_baseline = seen;
    }

    pub(in crate::app) fn clear_scratch_saved(&mut self) {
        self.active.extra.scratch_saved_version = None;
        self.active.extra.scratch_baseline = crate::external::Seen::Absent;
    }

    pub(in crate::app) fn arm_doc_autosave(&mut self, at: Instant) {
        self.active.extra.doc_autosave_at = Some(at);
    }

    pub(in crate::app) fn disarm_doc_autosave(&mut self) {
        self.active.extra.doc_autosave_at = None;
    }

    pub(in crate::app) fn history_preview(&self, id: &str) -> Option<&str> {
        let (cached, text) = self.active.extra.history_preview.as_ref()?;
        (cached == id).then_some(text.as_str())
    }

    pub(in crate::app) fn set_history_preview(&mut self, id: String, text: String) {
        self.active.extra.history_preview = Some((id, text));
    }

    pub(in crate::app) fn remember_history_scroll(&mut self) {
        self.active.extra.history_scroll_before = Some(self.active.extra.scroll);
    }

    pub(in crate::app) fn close_history(&mut self, accepted: bool) {
        if accepted {
            self.active.extra.history_scroll_before = None;
        } else if let Some(scroll) = self.active.extra.history_scroll_before.take() {
            self.active.extra.scroll = scroll;
        }
        self.active.extra.history_preview = None;
    }
}
