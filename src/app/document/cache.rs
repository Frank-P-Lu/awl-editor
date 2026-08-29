//! Cache, spell, and persistence state transitions for the active slot.

use super::*;

impl DocumentSession {
    pub(in crate::app) fn shift_selecting(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.extra.shift_selecting)
    }

    pub(in crate::app) fn set_shift_selecting(&mut self, value: bool) {
        if let Some(active) = self.active.as_mut() {
            active.extra.shift_selecting = value;
        }
    }

    pub(in crate::app) fn scroll(&self) -> crate::render::ScrollPos {
        self.active
            .as_ref()
            .map(|active| active.extra.scroll)
            .unwrap_or_default()
    }

    pub(in crate::app) fn set_scroll(&mut self, scroll: crate::render::ScrollPos) {
        if let Some(active) = self.active.as_mut() {
            active.extra.scroll = scroll;
        }
    }

    pub(in crate::app) fn spell_cache(&self) -> &[crate::spell::SpellVerdict] {
        self.active
            .as_ref()
            .map(|active| active.extra.spell_cache.as_slice())
            .unwrap_or(&[])
    }

    pub(in crate::app) fn spell_checked_version(&self) -> Option<u64> {
        self.active
            .as_ref()
            .and_then(|active| active.extra.spell_checked_version)
    }

    pub(in crate::app) fn invalidate_spell_cache(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.extra.spell_checked_version = None;
        }
    }

    pub(in crate::app) fn recompute_spell_cache(&mut self) {
        let Some(spell) = self.spell.as_ref() else {
            return;
        };
        let Some(active) = self.active.as_mut() else {
            return;
        };
        let text = active.buffer.text();
        let spans = spell.misspellings_for(&text, active.buffer.syntax_lang());
        active.extra.spell_cache = crate::spell::keyed(&text, spans);
        active.extra.spell_checked_version = Some(active.buffer.version());
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
        let active = self.active.as_ref()?;
        spell.suggest_at(
            &active.buffer.text(),
            line,
            col,
            active.buffer.syntax_lang(),
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
        let Some(active) = self.active.as_mut() else {
            return String::new();
        };
        let version = active.buffer.version();
        match &active.extra.sync_text_cache {
            Some((cached, text)) if *cached == version => text.clone(),
            _ => {
                let text = active.buffer.text();
                active.extra.sync_text_cache = Some((version, text.clone()));
                text
            }
        }
    }

    pub(in crate::app) fn caret_was_synced_at(&mut self, version: u64) -> bool {
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        let changed = active.extra.caret_synced_version != version;
        active.extra.caret_synced_version = version;
        changed
    }

    pub(in crate::app) fn doc_saved_version(&self) -> Option<u64> {
        self.active
            .as_ref()
            .and_then(|active| active.extra.doc_saved_version)
    }

    pub(in crate::app) fn scratch_saved_version(&self) -> Option<u64> {
        self.active
            .as_ref()
            .and_then(|active| active.extra.scratch_saved_version)
    }

    pub(in crate::app) fn disk_baseline(&self) -> crate::external::Seen {
        self.active
            .as_ref()
            .map(|active| active.extra.disk_baseline)
            .unwrap_or_default()
    }

    pub(in crate::app) fn scratch_baseline(&self) -> crate::external::Seen {
        self.active
            .as_ref()
            .map(|active| active.extra.scratch_baseline)
            .unwrap_or_default()
    }

    /// ADOPT a fresh observation of the document's path WITHOUT claiming the
    /// buffer was saved. The clean-reload and the conflict-resolution doors both
    /// need this: they have looked at the disk and must not be told about the
    /// same change twice, but the buffer's saved-version bookkeeping is theirs
    /// to set separately.
    pub(in crate::app) fn adopt_disk_baseline(&mut self, seen: crate::external::Seen) {
        if let Some(active) = self.active.as_mut() {
            active.extra.disk_baseline = seen;
        }
    }

    pub(in crate::app) fn record_document_saved(
        &mut self,
        version: u64,
        seen: crate::external::Seen,
    ) {
        let active = self.active_entry_mut();
        active.extra.doc_saved_version = Some(version);
        active.extra.disk_baseline = seen;
        active.extra.caret_synced_version = version;
    }

    pub(in crate::app) fn acknowledge_document_version(&mut self, version: u64) {
        self.active_entry_mut().extra.doc_saved_version = Some(version);
    }

    pub(in crate::app) fn record_scratch_saved(
        &mut self,
        version: u64,
        seen: crate::external::Seen,
    ) {
        let active = self.active_entry_mut();
        active.extra.scratch_saved_version = Some(version);
        active.extra.scratch_baseline = seen;
    }

    pub(in crate::app) fn clear_scratch_saved(&mut self) {
        let active = self.active_entry_mut();
        active.extra.scratch_saved_version = None;
        active.extra.scratch_baseline = crate::external::Seen::Absent;
    }

    /// Swap in a RESTORED true scratch buffer — the state-layer half of
    /// `App::open_scratch`'s "closed this session" arm. Stamps `version`
    /// already-saved against `baseline` like [`Self::new`] does, and enrols
    /// the working-set row itself (unlike a still-unnamed fresh document,
    /// scratch has a stable identity from the moment it exists, and
    /// `park_active` re-homes an existing row rather than creating one).
    pub(in crate::app) fn open_scratch(
        &mut self,
        buffer: Buffer,
        baseline: crate::external::Seen,
        root: PathBuf,
    ) {
        self.previous = self
            .active
            .as_ref()
            .and_then(|active| active.buffer.path())
            .map(Path::to_path_buf);
        self.park_active();
        let version = buffer.version();
        self.active = Some(crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                caret_synced_version: version,
                scratch_saved_version: Some(version),
                scratch_baseline: baseline,
                ..Default::default()
            },
        });
        self.working
            .open(crate::buffers::BufferKey::Scratch, None, root);
    }

    pub(in crate::app) fn arm_doc_autosave(&mut self, at: Instant) {
        if let Some(active) = self.active.as_mut() {
            active.extra.doc_autosave_at = Some(at);
        }
    }

    pub(in crate::app) fn disarm_doc_autosave(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.extra.doc_autosave_at = None;
        }
    }

    pub(in crate::app) fn history_preview(&self, id: &str) -> Option<&str> {
        let (cached, text) = self.active.as_ref()?.extra.history_preview.as_ref()?;
        (cached == id).then_some(text.as_str())
    }

    pub(in crate::app) fn set_history_preview(&mut self, id: String, text: String) {
        if let Some(active) = self.active.as_mut() {
            active.extra.history_preview = Some((id, text));
        }
    }

    pub(in crate::app) fn remember_history_scroll(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.extra.history_scroll_before = Some(active.extra.scroll);
        }
    }

    pub(in crate::app) fn close_history(&mut self, accepted: bool) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if accepted {
            active.extra.history_scroll_before = None;
        } else if let Some(scroll) = active.extra.history_scroll_before.take() {
            active.extra.scroll = scroll;
        }
        active.extra.history_preview = None;
    }
}
