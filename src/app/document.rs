//! The live document session: one active whole slot, every backgrounded slot,
//! the previous-buffer target, and the shared spell checker.
//!
//! `BufferExtra` is deliberately private here.  A buffer and every cache keyed
//! to its identity/version move as one `Entry` when parked or activated.  The
//! only mutable `Buffer` projection is [`DocumentSession::action_buffer_mut`],
//! used by the shared action core; every other caller uses a named document or
//! cache transition.

use crate::app::*;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct BufferExtra {
    shift_selecting: bool,
    scroll: crate::render::ScrollPos,
    spell_cache: Vec<crate::spell::SpellVerdict>,
    spell_checked_version: Option<u64>,
    sync_text_cache: Option<(u64, String)>,
    caret_synced_version: u64,
    doc_saved_version: Option<u64>,
    scratch_saved_version: Option<u64>,
    disk_mtime: Option<crate::fs::Metadata>,
    scratch_mtime: Option<crate::fs::Metadata>,
    doc_autosave_at: Option<Instant>,
    history_preview: Option<(String, String)>,
    history_scroll_before: Option<crate::render::ScrollPos>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) struct TestExtraProjection(BufferExtra);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum OpenPath {
    AlreadyActive,
    Reactivated,
    Fresh,
}

pub(in crate::app) struct DocumentSession {
    active: crate::buffers::Entry<BufferExtra>,
    registry: crate::buffers::BufferRegistry<BufferExtra>,
    previous: Option<PathBuf>,
    spell: Option<crate::spell::SpellChecker>,
}

impl DocumentSession {
    pub(in crate::app) fn new(
        buffer: Buffer,
        disk_mtime: Option<crate::fs::Metadata>,
        scratch_mtime: Option<crate::fs::Metadata>,
    ) -> Self {
        let initial_version = buffer.version();
        let active = crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                caret_synced_version: initial_version,
                doc_saved_version: Some(initial_version),
                disk_mtime,
                scratch_saved_version: Some(initial_version),
                scratch_mtime,
                ..Default::default()
            },
        };
        let spell = match crate::spell::SpellChecker::new(crate::spell::active_variant()) {
            Ok(sc) => Some(sc),
            Err(e) => {
                eprintln!("spell-check disabled: {e}");
                None
            }
        };
        Self {
            active,
            registry: Default::default(),
            previous: None,
            spell,
        }
    }

    pub(in crate::app) fn buffer(&self) -> &Buffer {
        &self.active.buffer
    }

    /// The one mutable-buffer loan, fenced to `app/apply.rs` by a source law.
    pub(in crate::app) fn action_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.active.buffer
    }

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

    pub(in crate::app) fn disk_mtime(&self) -> Option<crate::fs::Metadata> {
        self.active.extra.disk_mtime
    }

    pub(in crate::app) fn scratch_mtime(&self) -> Option<crate::fs::Metadata> {
        self.active.extra.scratch_mtime
    }

    pub(in crate::app) fn record_document_saved(
        &mut self,
        version: u64,
        mtime: Option<crate::fs::Metadata>,
    ) {
        self.active.extra.doc_saved_version = Some(version);
        self.active.extra.disk_mtime = mtime;
        self.active.extra.caret_synced_version = version;
    }

    pub(in crate::app) fn acknowledge_document_version(&mut self, version: u64) {
        self.active.extra.doc_saved_version = Some(version);
    }

    pub(in crate::app) fn record_scratch_saved(
        &mut self,
        version: u64,
        mtime: Option<crate::fs::Metadata>,
    ) {
        self.active.extra.scratch_saved_version = Some(version);
        self.active.extra.scratch_mtime = mtime;
    }

    pub(in crate::app) fn clear_scratch_saved(&mut self) {
        self.active.extra.scratch_saved_version = None;
        self.active.extra.scratch_mtime = None;
    }

    pub(in crate::app) fn doc_autosave_at(&self) -> Option<Instant> {
        self.active.extra.doc_autosave_at
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

    fn park_active(&mut self) {
        let Some(key) = crate::buffers::BufferKey::of(&self.active.buffer) else {
            return;
        };
        self.active.buffer.take_list_continuation_generated();
        let outgoing = std::mem::replace(
            &mut self.active,
            crate::buffers::Entry {
                buffer: Buffer::scratch(),
                extra: BufferExtra::default(),
            },
        );
        self.registry.park(key, outgoing);
    }

    fn activate(&mut self, key: &crate::buffers::BufferKey) -> bool {
        let Some(mut entry) = self.registry.take(key) else {
            return false;
        };
        entry.buffer.take_list_continuation_generated();
        self.active = entry;
        true
    }

    pub(in crate::app) fn open_path(
        &mut self,
        path: &Path,
        disk_mtime: Option<crate::fs::Metadata>,
    ) -> OpenPath {
        let key = crate::buffers::BufferKey::path(path);
        if self
            .active
            .buffer
            .path()
            .map(crate::buffers::BufferKey::path)
            == Some(key.clone())
        {
            return OpenPath::AlreadyActive;
        }
        self.previous = self.active.buffer.path().map(Path::to_path_buf);
        self.park_active();
        if self.activate(&key) {
            return OpenPath::Reactivated;
        }
        let buffer = Buffer::from_file(path);
        let version = buffer.version();
        self.active = crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                disk_mtime,
                doc_saved_version: Some(version),
                caret_synced_version: version,
                ..Default::default()
            },
        };
        OpenPath::Fresh
    }

    pub(in crate::app) fn start_fresh_document(&mut self, root: PathBuf) {
        self.previous = self.active.buffer.path().map(Path::to_path_buf);
        self.park_active();
        self.active.buffer.start_fresh_doc(root);
        self.active.extra.caret_synced_version = self.active.buffer.version();
    }

    pub(in crate::app) fn previous_path(&self) -> Option<PathBuf> {
        self.previous.clone()
    }

    pub(in crate::app) fn intercept_search_key(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
        logical: &winit::keyboard::Key,
        mods: winit::keyboard::ModifiersState,
    ) -> Option<crate::caret::RecoilDir> {
        crate::search::keys::intercept(search, &mut self.active.buffer, logical, mods)
    }

    pub(in crate::app) fn open_count(&self) -> usize {
        self.registry.len() + 1
    }

    #[cfg(test)]
    pub(in crate::app) fn contains_background(&self, key: &crate::buffers::BufferKey) -> bool {
        self.registry.contains(key)
    }

    #[cfg(test)]
    pub(in crate::app) fn replace_buffer(&mut self, buffer: Buffer) {
        self.active.buffer = buffer;
        self.active.extra = BufferExtra::default();
        self.active.extra.caret_synced_version = self.active.buffer.version();
    }

    #[cfg(test)]
    pub(in crate::app) fn undo(&mut self) {
        self.active.buffer.undo();
    }
    #[cfg(test)]
    pub(in crate::app) fn set_mark(&mut self) {
        self.active.buffer.set_mark();
    }
    #[cfg(test)]
    pub(in crate::app) fn toggle_fold_at_cursor(&mut self) {
        self.active.buffer.toggle_fold_at_cursor();
    }
    #[cfg(test)]
    pub(in crate::app) fn mark_list_continuation_generated(&mut self) {
        self.active.buffer.mark_list_continuation_generated();
    }
    #[cfg(test)]
    pub(in crate::app) fn take_list_continuation_generated(&mut self) -> bool {
        self.active.buffer.take_list_continuation_generated()
    }
    #[cfg(test)]
    pub(in crate::app) fn start_fresh_for_test(&mut self, root: PathBuf) {
        self.active.buffer.start_fresh_doc(root);
    }

    #[cfg(test)]
    fn extra(&self) -> &BufferExtra {
        &self.active.extra
    }

    #[cfg(test)]
    pub(in crate::app) fn sync_text_cached(&self) -> bool {
        self.extra().sync_text_cache.is_some()
    }
    #[cfg(test)]
    pub(in crate::app) fn caret_synced_version(&self) -> u64 {
        self.extra().caret_synced_version
    }
    #[cfg(test)]
    pub(in crate::app) fn history_preview_value(&self) -> Option<(String, String)> {
        self.extra().history_preview.clone()
    }
    #[cfg(test)]
    pub(in crate::app) fn history_scroll_before(&self) -> Option<crate::render::ScrollPos> {
        self.extra().history_scroll_before
    }

    #[cfg(test)]
    pub(in crate::app) fn seed_round_trip_extra(&mut self) {
        self.active.extra.shift_selecting = true;
        self.active.extra.scroll = crate::render::ScrollPos { row: 11, px_q: 29 };
        self.recompute_spell_cache();
        self.active.extra.sync_text_cache =
            Some((self.active.buffer.version(), self.active.buffer.text()));
        self.active.extra.caret_synced_version = 999;
        self.active.extra.doc_saved_version = Some(777);
        self.active.extra.scratch_saved_version = Some(888);
        self.active.extra.disk_mtime = Some(crate::fs::Metadata {
            modified: None,
            len: Some(101),
        });
        self.active.extra.scratch_mtime = Some(crate::fs::Metadata {
            modified: None,
            len: Some(202),
        });
        self.active.extra.doc_autosave_at = None;
        self.active.extra.history_preview = Some(("42".to_string(), "old text".to_string()));
        self.active.extra.history_scroll_before = Some(crate::render::ScrollPos::at_row(55));
    }

    #[cfg(test)]
    pub(in crate::app) fn round_trip_extra_signature(&self) -> TestExtraProjection {
        TestExtraProjection(self.active.extra.clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn session_buffers(&self) -> Vec<(PathBuf, crate::session::BufferPos)> {
        let mut buffers = Vec::new();
        if let Some(path) = self.active.buffer.path() {
            let (line, col) = self.active.buffer.cursor_line_col();
            buffers.push((
                path.to_path_buf(),
                crate::session::BufferPos {
                    line,
                    col,
                    scroll: self.active.extra.scroll.row,
                    scroll_px_q: self.active.extra.scroll.px_q,
                },
            ));
        }
        for (_key, entry) in self.registry.iter() {
            let Some(path) = entry.buffer.path() else {
                continue;
            };
            let (line, col) = entry.buffer.cursor_line_col();
            buffers.push((
                path.to_path_buf(),
                crate::session::BufferPos {
                    line,
                    col,
                    scroll: entry.extra.scroll.row,
                    scroll_px_q: entry.extra.scroll.px_q,
                },
            ));
        }
        buffers
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn restore_active(
        &mut self,
        path: &Path,
        pos: crate::session::BufferPos,
        mtime: Option<crate::fs::Metadata>,
    ) {
        let mut buffer = Buffer::from_file(path);
        apply_restored_pos(&mut buffer, pos);
        let version = buffer.version();
        self.active = crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                scroll: crate::render::ScrollPos {
                    row: pos.scroll,
                    px_q: pos.scroll_px_q,
                },
                doc_saved_version: Some(version),
                disk_mtime: mtime,
                caret_synced_version: version,
                ..Default::default()
            },
        };
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn restore_background(
        &mut self,
        path: &Path,
        pos: crate::session::BufferPos,
        mtime: Option<crate::fs::Metadata>,
    ) {
        let mut buffer = Buffer::from_file(path);
        apply_restored_pos(&mut buffer, pos);
        let version = buffer.version();
        let extra = BufferExtra {
            scroll: crate::render::ScrollPos {
                row: pos.scroll,
                px_q: pos.scroll_px_q,
            },
            doc_saved_version: Some(version),
            disk_mtime: mtime,
            caret_synced_version: version,
            ..Default::default()
        };
        self.registry.park(
            crate::buffers::BufferKey::path(path),
            crate::buffers::Entry { buffer, extra },
        );
    }

    // Named document mutations outside the shared action core.
    pub(in crate::app) fn set_cursor(&mut self, idx: usize) {
        self.active.buffer.set_cursor(idx);
    }
    pub(in crate::app) fn clear_mark(&mut self) {
        self.active.buffer.clear_mark();
    }
    pub(in crate::app) fn set_anchor(&mut self, idx: usize) {
        self.active.buffer.set_anchor(idx);
    }
    pub(in crate::app) fn select_range(&mut self, start: usize, end: usize) {
        self.active.buffer.select_range(start, end);
    }
    pub(in crate::app) fn seal_undo_group(&mut self) {
        self.active.buffer.seal_undo_group();
    }
    pub(in crate::app) fn reveal_placement(&mut self) {
        self.active.buffer.reveal_placement();
    }
    pub(in crate::app) fn toggle_fold_at_line(&mut self, line: usize) {
        self.active.buffer.toggle_fold_at_line(line);
    }
    pub(in crate::app) fn unfold_at(&mut self, line: usize) {
        self.active.buffer.unfold_at(line);
    }
    pub(in crate::app) fn insert_char(&mut self, ch: char) {
        self.active.buffer.insert_char(ch);
    }
    pub(in crate::app) fn insert_text(&mut self, text: &str) {
        self.active.buffer.insert_text(text);
    }
    pub(in crate::app) fn replace_char_range(&mut self, start: usize, end: usize, text: &str) {
        self.active.buffer.replace_char_range(start, end, text);
    }
    pub(in crate::app) fn set_text(&mut self, text: &str) {
        self.active.buffer.set_text(text);
    }
    pub(in crate::app) fn set_path(&mut self, path: PathBuf) {
        self.active.buffer.set_path(path);
    }
    pub(in crate::app) fn set_note_dir(&mut self, path: PathBuf) {
        self.active.buffer.set_note_dir(path);
    }
    pub(in crate::app) fn set_kill(&mut self, text: &str) {
        self.active.buffer.set_kill(text);
    }
    pub(in crate::app) fn save(&mut self) -> anyhow::Result<()> {
        self.active.buffer.save()
    }
    pub(in crate::app) fn save_into_folder(&mut self, folder: &Path) -> anyhow::Result<()> {
        self.active.buffer.save_into_folder(folder)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_restored_pos(buffer: &mut Buffer, pos: crate::session::BufferPos) {
    let idx = buffer.line_col_to_char(pos.line, pos.col);
    buffer.clear_mark();
    buffer.set_cursor(idx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn every_buffer_extra_field_round_trips_a_b_a_b_c_a() {
        let _guard = crate::testlock::serial();
        let a = PathBuf::from("/session/a.md");
        let b = PathBuf::from("/session/b.md");
        let c = PathBuf::from("/session/c.md");
        let fs = crate::fs::InMemoryFs::new()
            .with_file(&a, "helo alpha\n")
            .with_file(&b, "bravo\n")
            .with_file(&c, "charlie\n");
        let _fs = crate::fs::FsGuard::install(Arc::new(fs));
        let mut session = DocumentSession::new(Buffer::from_file(&a), None, None);

        session.seed_round_trip_extra();
        session.active.extra.doc_autosave_at = Some(Instant::now());
        let expected = session.active.extra.clone();
        assert!(
            !expected.spell_cache.is_empty(),
            "fixture must exercise spell cache"
        );

        assert_eq!(session.open_path(&b, None), OpenPath::Fresh);
        assert_eq!(session.open_path(&a, None), OpenPath::Reactivated);
        assert_eq!(session.active.extra, expected, "A -> B -> A");
        assert_eq!(session.open_path(&b, None), OpenPath::Reactivated);
        assert_eq!(session.open_path(&c, None), OpenPath::Fresh);
        assert_eq!(session.open_path(&a, None), OpenPath::Reactivated);
        assert_eq!(session.active.extra, expected, "A -> B -> C -> A");
    }
}
