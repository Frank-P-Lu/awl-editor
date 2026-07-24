//! src/app/files/notes.rs — the TWO-DESK "Notes" flip (item 59):
//! `Action::NotesFlip`'s impure apply (the pure toggle-target decision,
//! [`super::notes_flip_target`], lives in `files/mod.rs`, unit-tested
//! standalone), plus its two buffer-swap halves — a fresh untitled quick-note
//! (`start_fresh_note`, shared with `open::new_note`'s C-x n) and the
//! pathless scratch restore (`restore_scratch_desk`). Split out of the former
//! `app/files.rs` monolith (item 56).

use crate::app::*;
use super::active::{BufferExtra, DeskReturn};
use super::{notes_flip_target, NotesFlipTarget};

impl App {

    /// "Notes" (`Action::NotesFlip`, palette-only, user-decided 2026-07-22):
    /// a TWO-DESK FLIP between the current project and `notes_root` (item 59).
    /// The pure toggle DECISION lives in [`notes_flip_target`] (unit-tested
    /// standalone, no `App` needed); this is its impure APPLY.
    ///
    /// The flip switches the WHOLE writing context, not just the root: entering
    /// Notes parks the current buffer, re-scopes the root, and RESTORES the last
    /// active notes buffer with its full caret/selection/scroll/fold view (via
    /// `buffer_registry`) — or opens a FRESH untitled quick-note on a first-ever
    /// visit rather than choosing an arbitrary existing file. Flipping back
    /// restores the exact prior project AND its active buffer/view. Dirty
    /// buffers are PARKED (a dirty note is named+saved by `set_root`'s
    /// `flush_note` first, then parked), never discarded or spuriously saved.
    ///
    /// The root change GATES the buffer swap as ONE transaction: `set_root`
    /// returning `false` (a cancelled MAS grant) leaves BOTH desks and the
    /// remembered return target untouched. Like the C-x n jump, the visit is
    /// TRANSIENT — it calls [`Self::set_root`] directly rather than
    /// [`Self::switch_project`], so it never persists the STICKY project root
    /// (a bare relaunch still reopens whatever was genuinely active) nor pushes
    /// the recent-projects MRU (Notes is not a "recent project" you switched to).
    pub(in crate::app) fn notes_flip(&mut self) {
        let previous = self.notes_return.as_ref().map(|d| d.root.as_path());
        match notes_flip_target(&self.root, &self.notes_root, previous) {
            // Nothing to flip to (no usable notes_root) or nothing to flip
            // BACK to (already home, nothing remembered) — a quiet no-op,
            // exactly like `last_buffer_toggle`'s own "nothing opened before".
            NotesFlipTarget::Inert | NotesFlipTarget::AlreadyHome => {}
            NotesFlipTarget::Enter { target, remember } => {
                // The notes root may not exist yet on a fresh machine — create
                // it lazily, exactly like `new_note` does before its own jump.
                let _ = crate::fs::active().create_dir_all(&target);
                // ONE TRANSACTION: the root change gates everything below. A
                // denied grant returns here with both desks + the return memory
                // untouched (`notes_last_file` is not consumed until success).
                if !self.set_root(target) {
                    return;
                }
                // `set_root` flushed the leaving (home) buffer, so a dirty
                // home NOTE now carries its derived path — snapshot the home
                // desk AFTER, so that just-named file is what we return to.
                // Captured from `Buffer::path()` (the sole authoritative
                // path, item 56) BEFORE the swap below parks it.
                let return_desk = DeskReturn {
                    root: remember,
                    file: self.active.buffer.path().map(|p| p.to_path_buf()),
                };
                // Restore the notes desk's last active buffer (full view via
                // the registry), or open a fresh untitled note on a first-ever
                // visit. Either activation parks the home buffer under its key.
                match self.notes_last_file.take() {
                    Some(f) => self.load_path(f),
                    None => self.start_fresh_note(),
                }
                self.notes_return = Some(return_desk);
            }
            NotesFlipTarget::Back { target } => {
                // `target` is the remembered home root; take the full desk.
                let Some(ret) = self.notes_return.take() else { return };
                debug_assert_eq!(ret.root, target, "Back target must be the remembered home root");
                if !self.set_root(ret.root.clone()) {
                    // Denied: restore the memory, leave both desks untouched.
                    self.notes_return = Some(ret);
                    return;
                }
                // `set_root` flushed the leaving notes buffer (naming a dirty
                // note), so `Buffer::path()` now reflects the notes desk's real
                // active file — remember it for the next visit (an unnamed/empty
                // note leaves `None` → a fresh note opens next time). Captured
                // BEFORE the swap below parks it.
                self.notes_last_file = self.active.buffer.path().map(|p| p.to_path_buf());
                match ret.file {
                    Some(f) => self.load_path(f),
                    None => self.restore_scratch_desk(),
                }
            }
        }
    }


    /// Swap in a fresh empty quick-NOTE buffer as the active buffer — the
    /// buffer-swap half of C-x n, factored out of [`Self::new_note`] so the
    /// two-desk Notes flip's first-ever visit opens the SAME fresh note without
    /// re-running the root change. The caller has ALREADY re-scoped the root to
    /// `notes_root` (via `set_root`). Parks the leaving buffer under its key.
    pub(super) fn start_fresh_note(&mut self) {
        // Captured BEFORE `park_active_buffer` below moves the slot away
        // (`Buffer::path()` is the sole authoritative path, item 56).
        self.prev_file = self.active.buffer.path().map(|p| p.to_path_buf());
        // WRITING STREAKS: sample the LEAVING buffer's word-delta before it is
        // replaced by the fresh note (the anchor is reset below), so words
        // written in it are recorded before the swap (native only; gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush();
        // PARK the buffer we are leaving (registered under its own identity if
        // it has one) exactly like `load_path`, so a later C-x b / reopen finds
        // it live rather than re-reading disk.
        self.park_active_buffer();
        // `park_active_buffer` already left `self.active` a fresh
        // `Entry{buffer: Buffer::scratch(), extra: BufferExtra::default()}`
        // placeholder — start the note in place on that already-complete slot.
        self.active.buffer.start_note(self.notes_root.clone());
        self.search = None;
        self.preedit.clear();
        self.active.extra.caret_synced_version = self.active.buffer.version();
        self.autosave_saved_version = None;
        self.autosave_dirty_at = None;
        // STICKY PAGE WIDTH: a fresh note is always markdown (PROSE), so this
        // re-applies `page_width_prose` regardless of what the leaving buffer's
        // kind was — mirrors `load_path`'s own resync.
        self.sync_page_measure();
        // LIFETIME STATS: a fresh note is a buffer swap — drop the caret-travel
        // anchor so its first caret sample re-anchors (see `load_path`).
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        // WRITING STREAKS: a fresh note is an awl-CREATED buffer born empty, so
        // anchor EAGERLY at its birth count (0) rather than lazily — otherwise the
        // words typed before the first idle flush would be anchored away on that
        // flush (the anchor-swallow bug). See `streaks_anchor_now` vs the lazy
        // `streaks_reset_baseline` an OPENED file uses.
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_anchor_now();
        self.update_title();
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }


    /// Restore the pathless SCRATCH buffer as the active buffer — the pathless
    /// sibling of [`Self::load_path`], used when a Notes flip BACK returns to a
    /// home desk that had NO open file (a bare-launch scratch surface). Parks
    /// the current (notes) buffer, then brings the registry's `Scratch` entry
    /// (its edits + full view state) back, or a fresh scratch if it was evicted.
    fn restore_scratch_desk(&mut self) {
        self.flush_note();
        self.autosave_flush();
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_flush();
        // Captured BEFORE `park_active_buffer` below moves the slot away.
        self.prev_file = self.active.buffer.path().map(|p| p.to_path_buf());
        self.park_active_buffer();
        if !self.activate_from_registry(&crate::buffers::BufferKey::Scratch) {
            self.active = crate::buffers::Entry {
                buffer: Buffer::scratch(),
                extra: BufferExtra::default(),
            };
            self.active.extra.doc_saved_version = Some(self.active.buffer.version());
            self.active.extra.caret_synced_version = self.active.buffer.version();
        }
        self.search = None;
        self.preedit.clear();
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_reset_baseline();
        self.sync_page_measure();
        self.update_title();
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }
}
