//! Duplicate-current-file ownership.

use crate::app::*;
use std::path::Path;

impl App {
    /// Copy the current live bytes to a no-clobber sibling and open the copy.
    /// The original is parked through the ordinary load door, so unsaved edits
    /// remain reachable and the copy begins a distinct history timeline.
    pub(in crate::app) fn duplicate_current_file(&mut self) {
        let Some(old) = self.document.buffer().path().map(Path::to_path_buf) else {
            return;
        };
        self.flush_note();
        self.autosave_flush();
        if self.refuse_while_unresolved() {
            return;
        }
        let bytes = self.document.buffer().disk_bytes();
        let dir = old.parent().map(Path::to_path_buf).unwrap_or_default();
        let stem = old
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = old
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        match crate::buffer::write_new_unique(
            crate::durable::Owner::ManualSave,
            &dir,
            &stem,
            &ext,
            &bytes,
            |candidate| self.document.path_is_claimed_by_other(candidate),
        ) {
            Ok(new_path) => {
                self.load_path(new_path);
                self.set_toast_notice("duplicated");
            }
            Err(error) => self.set_sticky_notice(format!("duplicate failed: {error}")),
        }
        self.request_frame();
    }
}
