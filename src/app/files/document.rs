//! src/app/files/document.rs — the FRESH DOCUMENT buffer-swap (Cmd-N /
//! `open::new_document`'s buffer-swap half). The old two-desk
//! project-flip command that used to live in this file (with it, the
//! notes-desk scratch-restore helper) — there is now exactly ONE active
//! folder (`App::root`), so there is nothing to flip between. Split out of the
//! former `app/files.rs` monolith and renamed from `notes.rs`.

use crate::app::*;

impl App {
    /// Swap in a fresh, unnamed document buffer as the active buffer — the
    /// buffer-swap half of Cmd-N (`open::new_document`). The caller has NOT
    /// changed the root: a fresh document is created IN the current active
    /// folder (`self.root`), never a separate notes home. Parks the leaving
    /// buffer under its key.
    pub(super) fn start_fresh_document(&mut self) {
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
        self.document
            .start_fresh_document(self.project_location.root.clone());
        self.workspace_state.close_search();
        self.input.clear_preedit();
        self.persistence.reset_for_fresh_document();
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
