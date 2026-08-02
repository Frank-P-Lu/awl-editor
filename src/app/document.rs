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

mod cache;
#[cfg(not(target_arch = "wasm32"))]
mod session_restore;
#[cfg(test)]
mod tests;

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

/// The document-owned result of polling its autosave timer. A due result has
/// already consumed the arm, so the scheduler cannot fire the same deadline
/// twice or mutate a buffer-scoped cache directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum AutosavePoll {
    Idle,
    WaitingUntil(Instant),
    Due,
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

    pub(in crate::app) fn poll_autosave(
        &mut self,
        now: Instant,
        idle: std::time::Duration,
    ) -> AutosavePoll {
        let Some(dirty) = self.active.extra.doc_autosave_at else {
            return AutosavePoll::Idle;
        };
        if now.saturating_duration_since(dirty) >= idle {
            self.active.extra.doc_autosave_at = None;
            AutosavePoll::Due
        } else {
            AutosavePoll::WaitingUntil(dirty + idle)
        }
    }

    pub(in crate::app) fn buffer(&self) -> &Buffer {
        &self.active.buffer
    }

    /// The one mutable-buffer loan, fenced to `app/apply.rs` by a source law.
    pub(in crate::app) fn action_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.active.buffer
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

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn open_count(&self) -> usize {
        self.registry.len() + 1
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
