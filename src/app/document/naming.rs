//! Live-session naming and save ownership.

use super::DocumentSession;
use std::path::Path;

impl DocumentSession {
    pub(in crate::app) fn save(&mut self) -> anyhow::Result<()> {
        self.save_owned(crate::durable::Owner::ManualSave)
    }

    pub(in crate::app) fn save_owned(
        &mut self,
        owner: crate::durable::Owner,
    ) -> anyhow::Result<()> {
        let reserved = self.other_live_path_keys();
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .save_owned_avoiding(owner, |candidate| {
                reserved.contains(&crate::buffers::BufferKey::path(candidate))
            })
    }

    pub(in crate::app) fn save_into_folder(&mut self, folder: &Path) -> anyhow::Result<()> {
        let reserved = self.other_live_path_keys();
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .save_into_folder_avoiding(folder, |candidate| {
                reserved.contains(&crate::buffers::BufferKey::path(candidate))
            })
    }

    /// Snapshot every normalized Path key owned by a buffer OTHER than the
    /// active one. Naming/move/duplicate allocate against this set and disk
    /// existence through `buffer::unique_path_avoiding` before writing.
    fn other_live_path_keys(&self) -> std::collections::HashSet<crate::buffers::BufferKey> {
        let active = self.active_key();
        self.working
            .files()
            .iter()
            .filter(|file| Some(&file.key) != active.as_ref())
            .filter(|file| matches!(file.key, crate::buffers::BufferKey::Path(_)))
            .map(|file| file.key.clone())
            .collect()
    }

    pub(in crate::app) fn path_is_claimed_by_other(&self, candidate: &Path) -> bool {
        self.other_live_path_keys()
            .contains(&crate::buffers::BufferKey::path(candidate))
    }
}
