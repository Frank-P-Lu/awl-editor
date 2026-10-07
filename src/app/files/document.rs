//! Fresh-document activation and caret placement after navigation.

use crate::app::*;

impl App {
    /// Park the current buffer and activate an unnamed document in the current folder.
    pub(super) fn start_fresh_document(&mut self) {
        self.start_fresh_document_at(self.project_location.root.clone());
    }

    /// Both fresh-document doors share the leave boundary and activation work.
    pub(super) fn start_fresh_document_at(&mut self, destination: PathBuf) {
        if self.refuse_while_unresolved() {
            return;
        }
        self.flush_note();
        self.autosave_flush();
        if self.refuse_while_unresolved() {
            return;
        }
        if self.is_document_dirty() {
            self.set_sticky_notice(
                "Changes are still unsaved — save before starting another document",
            );
            self.request_frame();
            return;
        }
        let _ = crate::fs::active().create_dir_all(&destination);
        // WRITING STREAKS: sample the LEAVING buffer's word-delta before it is
        // replaced by the fresh document (the anchor is reset below), so words
        // written in it are recorded before the swap (native only; gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        if self.document.has_active() {
            self.streaks_flush();
        }
        // PARK the buffer we are leaving (registered under its own identity if
        // it has one) exactly like `load_path`, so a later C-x b / reopen finds
        // it live rather than re-reading disk.
        self.document.start_fresh_document(destination);
        self.workspace_state.close_search();
        self.input.clear_preedit();
        self.persistence
            .reset_for_fresh_document(self.document.active_key().expect("fresh document key"));
        // STICKY PAGE WIDTH: a fresh document is always markdown (PROSE), so this
        // re-applies `page_width_prose` regardless of what the leaving buffer's
        // kind was — mirrors `load_path`'s own resync.
        self.sync_page_measure();
        // LIFETIME STATS: a fresh document is a buffer swap — drop the
        // caret-travel anchor so its first caret sample re-anchors (see
        // `load_path`).
        #[cfg(not(target_arch = "wasm32"))]
        self.stats_reset_caret_anchor();
        // WRITING STREAKS: a fresh document is an awl-CREATED buffer born
        // empty, so anchor EAGERLY at its birth count (0) rather than lazily —
        // otherwise the words typed before the first idle flush would be
        // anchored away on that flush (the anchor-swallow bug). See
        // `streaks_anchor_now` vs the lazy `streaks_reset_baseline` an OPENED
        // file uses.
        #[cfg(not(target_arch = "wasm32"))]
        self.streaks_anchor_now();
        self.update_title();
        self.sync_view(true);
        self.request_frame();
    }
}

impl App {
    /// SEARCH-IN-FOLDER's own door: open `rel` (root-relative, `open_rel`'s
    /// own resolve) AND land the caret at `line`/`col` -- but ONLY if the open
    /// actually succeeds (unlike `open_rel`, which discards `load_path`'s
    /// bool; a refused open here -- classified unsupported, deleted since the
    /// search ran -- must never jump the caret inside whatever buffer was
    /// already active).
    pub(in crate::app) fn open_path_at_line(&mut self, rel: &str, line: usize, col: usize) {
        let path = crate::index::resolve(&self.project_location.root, rel);
        if self.load_path(path) {
            self.jump_to_line_col(line, col);
        }
    }

    pub(in crate::app) fn jump_to_line(&mut self, line: usize) {
        self.jump_to_line_col(line, 0);
    }

    /// `jump_to_line`'s own column-aware generalization -- search-in-folder's
    /// door lands the caret exactly on the match, not just the line start.
    pub(in crate::app) fn jump_to_line_col(&mut self, line: usize, col: usize) {
        let idx = self.document.buffer().line_col_to_char(line, col);
        self.document.clear_mark();
        self.document.set_cursor(idx);
        // REVEALED PLACEMENT (folds): a heading Go-to / margin-outline jump may target
        // a line hidden inside a collapsed section — route through the ONE placement
        // owner so the landing line is revealed, never left inside a fold. A cheap
        // no-op unless a section is folded.
        self.document.reveal_placement();
        self.document.set_shift_selecting(false);
        self.sync_view(true);
        self.request_frame();
    }
}

impl App {
    /// The event-loop shutdown owner, shared with persistence regressions.
    pub(in crate::app) fn flush_documents_for_shutdown(&mut self) {
        self.flush_note();
        self.autosave_flush();
        // THE UNRESOLVED CHANGE'S LAST WRITE. `autosave_flush` above refreshes
        // the record whenever the engine would have written the file, but it
        // short-circuits on a version it has already acknowledged — so this
        // makes the guarantee unconditional at the one moment it stops being
        // repeatable. The window close button reaches here without passing the
        // Quit deferral, which is exactly why the record cannot depend on it.
        if let Some(path) = self.persistence.unresolved().map(|u| u.path.clone()) {
            self.write_recovery_record(&path);
        }
        // SESSION RESTORE: the final safety net, mirroring the autosave flush
        // right above it (native only; kill-switch gated inside).
        #[cfg(not(target_arch = "wasm32"))]
        self.session_flush();
    }
}
