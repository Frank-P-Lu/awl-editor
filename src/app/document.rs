//! The live document session: an optional active whole slot, every backgrounded
//! slot, the previous-buffer target, and the shared spell checker. `None` is the
//! honest state after the last document closes.
//!
//! `BufferExtra` is deliberately private here.  A buffer and every cache keyed
//! to its identity/version move as one `Entry` when parked or activated.  The
//! only mutable `Buffer` projection is [`DocumentSession::action_buffer_mut`],
//! used by the shared action core; every other caller uses a named document or
//! cache transition.

use crate::app::*;
use std::path::Path;

mod cache;
mod edit;
mod entries;
mod naming;
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
    /// What awl last SAW at the buffer's own path — stat AND content digest
    /// (`crate::external`). A stat alone cannot answer the question this field
    /// exists for; see that module's doc.
    disk_baseline: crate::external::Seen,
    /// The same observation for the persistent scratch stash.
    scratch_baseline: crate::external::Seen,
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
    active: Option<crate::buffers::Entry<BufferExtra>>,
    registry: crate::buffers::BufferRegistry<BufferExtra>,
    previous: Option<crate::buffers::BufferKey>,
    spell: Option<crate::spell::SpellChecker>,
    /// The VISIBLE order and each open file's own project root
    /// ([`crate::workingset`]). It sits beside `registry` rather than inside it
    /// because the two answer different questions: `registry` is MRU because it
    /// evicts, this is stable because it is drawn. Kept in step here — the one
    /// place both are mutated — so no consumer can see one updated and the other
    /// not.
    working: crate::workingset::WorkingSet,
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
    fn active_entry_mut(&mut self) -> &mut crate::buffers::Entry<BufferExtra> {
        self.active.as_mut().expect("active document")
    }

    /// The sole whole-slot removal door. Parking, close, and explicit empty
    /// session restore all route through it so adding buffer-owned state cannot
    /// leave one transition holding a partial document.
    fn take_active(&mut self) -> Option<crate::buffers::Entry<BufferExtra>> {
        self.active.take()
    }

    /// The sole registry removal door, shared by ordinary activation and the
    /// close successor transition.
    fn take_parked(
        &mut self,
        key: &crate::buffers::BufferKey,
    ) -> Option<crate::buffers::Entry<BufferExtra>> {
        self.registry.take(key)
    }

    pub(in crate::app) fn new(
        buffer: Buffer,
        disk_baseline: crate::external::Seen,
        scratch_baseline: crate::external::Seen,
    ) -> Self {
        let initial_version = buffer.version();
        let active = crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                caret_synced_version: initial_version,
                doc_saved_version: Some(initial_version),
                disk_baseline,
                scratch_saved_version: Some(initial_version),
                scratch_baseline,
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
            active: Some(active),
            registry: Default::default(),
            previous: None,
            spell,
            working: Default::default(),
        }
    }

    /// Enrol whatever buffer is active into the working set, under `active_root`.
    ///
    /// Called once at startup, AFTER every startup decision that can swap the
    /// active buffer has settled (the scratch-stash restore and session restore
    /// both can). Enrolling in `new` instead would register a buffer the launch
    /// then replaces, and the margin would name a document nobody opened.
    pub(in crate::app) fn enrol_active(&mut self, active_root: &Path) {
        let Some(active) = self.active.as_ref() else {
            return;
        };
        let key = crate::buffers::BufferKey::of(&active.buffer);
        let path = active.buffer.path().map(Path::to_path_buf);
        let root = match path.as_deref() {
            Some(p) => crate::workingset::root_for(p, active_root, None),
            None => active_root.to_path_buf(),
        };
        self.working.open(key, path, root);
    }

    pub(in crate::app) fn working_set(&self) -> &crate::workingset::WorkingSet {
        &self.working
    }

    /// The margin's own transient panel state (expand/collapse/scroll) is
    /// UI, not order or root truth, but it lives on the same owner rather than
    /// on root `App` (`app/tests/domains.rs::root_app_does_not_grow`'s field
    /// ceiling) — and mutating it never touches `active`/`registry`/`spell`,
    /// so it cannot desync the invariants those fields hold.
    pub(in crate::app) fn working_set_mut(&mut self) -> &mut crate::workingset::WorkingSet {
        &mut self.working
    }

    pub(in crate::app) fn poll_autosave(
        &mut self,
        now: Instant,
        idle: std::time::Duration,
    ) -> AutosavePoll {
        let Some(active) = self.active.as_mut() else {
            return AutosavePoll::Idle;
        };
        let Some(dirty) = active.extra.doc_autosave_at else {
            return AutosavePoll::Idle;
        };
        if now.saturating_duration_since(dirty) >= idle {
            active.extra.doc_autosave_at = None;
            AutosavePoll::Due
        } else {
            AutosavePoll::WaitingUntil(dirty + idle)
        }
    }

    pub(in crate::app) fn has_active(&self) -> bool {
        self.active.is_some()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn active_is_pathless(&self) -> bool {
        self.buffer_opt()
            .is_some_and(|buffer| buffer.path().is_none())
    }

    /// **DOES ANY BUFFER BEHIND THE ACTIVE ONE WANT THE MARGIN OUTLINE'S
    /// RAIL?** Handed to the renderer each `sync_view`
    /// (`ViewState::set_wants_outline_rail`) so the adaptive column belongs to
    /// the room, not to whichever file is on screen.
    pub(in crate::app) fn parked_wants_rail(&self) -> bool {
        self.registry.backgrounded_wants_rail()
    }

    pub(in crate::app) fn active_is_markdown(&self) -> bool {
        self.buffer_opt().is_some_and(Buffer::is_markdown)
    }

    pub(in crate::app) fn buffer_opt(&self) -> Option<&Buffer> {
        self.active.as_ref().map(|active| &active.buffer)
    }

    pub(in crate::app) fn buffer(&self) -> &Buffer {
        &self
            .active
            .as_ref()
            .expect("active-document-only path reached with no document")
            .buffer
    }

    /// The one mutable-buffer loan — the insertion-door census' `TextDoor::ActionCore`
    /// (`app/input/text_door.rs`), fenced to its single `app/apply.rs` call site by a
    /// source law. Its gate is deliberately one layer up: `actions::intercept_action`
    /// consumes every action before a buffer verb runs while a card or a summoned field
    /// is up, at the ACTION level because a menu key equivalent never becomes a key.
    /// `None` is the explicit no-document state; the action core uses an inert
    /// transition buffer solely while the Go-to card is active.
    pub(in crate::app) fn action_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.active.as_mut().map(|active| &mut active.buffer)
    }

    fn park_active(&mut self) {
        let Some(mut outgoing) = self.take_active() else {
            return;
        };
        let key = crate::buffers::BufferKey::of(&outgoing.buffer);
        outgoing.buffer.take_list_continuation_generated();
        self.registry.park(key, outgoing);
    }

    fn activate(&mut self, key: &crate::buffers::BufferKey) -> bool {
        let Some(mut entry) = self.take_parked(key) else {
            return false;
        };
        entry.buffer.take_list_continuation_generated();
        self.active = Some(entry);
        true
    }

    pub(in crate::app) fn open_path(
        &mut self,
        path: &Path,
        disk_baseline: crate::external::Seen,
        active_root: &Path,
    ) -> OpenPath {
        let key = crate::buffers::BufferKey::path(path);
        if self
            .active
            .as_ref()
            .map(|active| crate::buffers::BufferKey::of(&active.buffer))
            == Some(key.clone())
        {
            return OpenPath::AlreadyActive;
        }
        // The working set is updated on the SAME transition as the registry, so
        // the drawn order and the parked buffers cannot disagree. `root_for`
        // decides ownership from the file rather than the moment — see its doc
        // for why the active root must not simply win.
        let remembered = self
            .working
            .index_of(&key)
            .map(|at| self.working.files()[at].root.clone());
        let root = crate::workingset::root_for(path, active_root, remembered.as_deref());
        self.working
            .open(key.clone(), Some(path.to_path_buf()), root);
        self.previous = self
            .active
            .as_ref()
            .map(|active| crate::buffers::BufferKey::of(&active.buffer));
        self.park_active();
        if self.activate(&key) {
            return OpenPath::Reactivated;
        }
        let buffer = Buffer::from_file(path);
        let version = buffer.version();
        self.active = Some(crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                disk_baseline,
                doc_saved_version: Some(version),
                caret_synced_version: version,
                ..Default::default()
            },
        });
        OpenPath::Fresh
    }

    /// RELOAD the active buffer from its own file, keeping the caret's LINE and
    /// COLUMN and the scroll position. The clean-buffer arm of the external
    /// change guard: nothing of the user's is at stake, so the disk simply wins.
    ///
    /// The undo timeline goes with the old buffer, deliberately. It described
    /// edits against text that is no longer what the file says, so replaying it
    /// would reconstruct a document that never existed on either side. A clean
    /// buffer had nothing on that timeline the user could want back anyway.
    ///
    /// Returns `false` for a path-less buffer, which has no file to reload from.
    pub(in crate::app) fn reload_active_from_disk(&mut self, seen: crate::external::Seen) -> bool {
        let Some(path) = self
            .active
            .as_ref()
            .and_then(|active| active.buffer.path())
            .map(Path::to_path_buf)
        else {
            return false;
        };
        let active = self.active.as_ref().expect("checked above");
        let (line, col) = self
            .active
            .as_ref()
            .expect("checked above")
            .buffer
            .char_to_line_col(active.buffer.cursor_char());
        let scroll = active.extra.scroll;
        let mut buffer = Buffer::from_file(&path);
        // Both are clamped by the buffer, so a file that shrank leaves the
        // caret at the new end rather than past it.
        let idx = buffer.line_col_to_char(line, col);
        buffer.clear_mark();
        buffer.set_cursor(idx);
        let version = buffer.version();
        self.active = Some(crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                scroll,
                doc_saved_version: Some(version),
                disk_baseline: seen,
                caret_synced_version: version,
                ..Default::default()
            },
        });
        true
    }

    pub(in crate::app) fn start_fresh_document(&mut self, root: PathBuf) {
        self.previous = self
            .active
            .as_ref()
            .map(|active| crate::buffers::BufferKey::of(&active.buffer));
        self.park_active();
        let mut buffer = Buffer::scratch();
        buffer.start_fresh_doc(root.clone());
        let version = buffer.version();
        self.active = Some(crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                caret_synced_version: version,
                ..Default::default()
            },
        });
        let key = crate::buffers::BufferKey::of(self.buffer());
        self.working.open(key, None, root);
    }

    /// Commit the visible identity transition after a fresh document's naming
    /// write has succeeded. The working-set slot stays in place; only its key
    /// and now-real path change.
    pub(in crate::app) fn rekey_active_after_naming(&mut self) {
        let Some(path) = self
            .buffer_opt()
            .and_then(Buffer::path)
            .map(Path::to_path_buf)
        else {
            return;
        };
        let key = crate::buffers::BufferKey::path(&path);
        self.working.rekey_active(key, Some(path));
    }

    #[cfg(test)]
    pub(in crate::app) fn previous_path(&self) -> Option<PathBuf> {
        self.previous
            .as_ref()
            .and_then(crate::buffers::BufferKey::path_buf)
    }

    pub(in crate::app) fn previous_key(&self) -> Option<crate::buffers::BufferKey> {
        self.previous.clone()
    }

    pub(in crate::app) fn intercept_search_key(
        &mut self,
        search: &mut Option<crate::search::SearchState>,
        logical: &winit::keyboard::Key,
        mods: winit::keyboard::ModifiersState,
    ) -> Option<crate::caret::RecoilDir> {
        crate::search::keys::intercept(search, &mut self.active_entry_mut().buffer, logical, mods)
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn open_count(&self) -> usize {
        self.registry.len() + usize::from(self.active.is_some())
    }

    // Named document mutations outside the shared action core.
    pub(in crate::app) fn set_cursor(&mut self, idx: usize) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .set_cursor(idx);
    }
    pub(in crate::app) fn clear_mark(&mut self) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .clear_mark();
    }
    pub(in crate::app) fn set_anchor(&mut self, idx: usize) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .set_anchor(idx);
    }
    pub(in crate::app) fn select_range(&mut self, start: usize, end: usize) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .select_range(start, end);
    }
    pub(in crate::app) fn seal_undo_group(&mut self) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .seal_undo_group();
    }
    pub(in crate::app) fn reveal_placement(&mut self) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .reveal_placement();
    }
    pub(in crate::app) fn toggle_fold_at_line(&mut self, line: usize) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .toggle_fold_at_line(line);
    }
    pub(in crate::app) fn unfold_at(&mut self, line: usize) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .unfold_at(line);
    }
    pub(in crate::app) fn set_path(&mut self, path: PathBuf) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .set_path(path);
    }
    pub(in crate::app) fn set_note_dir(&mut self, path: PathBuf) {
        self.active_entry_mut().buffer.set_note_dir(path);
    }
    pub(in crate::app) fn set_kill(&mut self, text: &str) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .set_kill(text);
    }
}
