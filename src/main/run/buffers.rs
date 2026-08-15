//! Active-buffer switching for headless replay.
//!
//! The live App and replay share [`crate::buffers::BufferRegistry`]. This
//! module owns replay's two transitions that leave the active buffer: opening
//! a Go-to target and starting a fresh document. Both park the whole departing
//! buffer before installing its replacement, so cursor, edits, and undo state
//! travel together.

use super::*;

impl ReplaySession<'_> {
    fn park_active_buffer(&mut self) {
        let Some(key) = crate::buffers::BufferKey::of(self.buffer) else {
            return;
        };
        let old = std::mem::replace(self.buffer, Buffer::scratch());
        self.registry.park(
            key,
            crate::buffers::Entry {
                buffer: old,
                extra: (),
            },
        );
    }

    pub(super) fn switch_to_goto_target(&mut self, value: &str) {
        let path = crate::index::resolve(&self.root, value);
        let new_key = crate::buffers::BufferKey::path(&path);
        if crate::buffers::BufferKey::of(self.buffer).as_ref() == Some(&new_key) {
            return;
        }

        self.park_active_buffer();
        *self.buffer = match self.registry.take(&new_key) {
            Some(entry) => entry.buffer,
            None => Buffer::from_file(&path),
        };
        crate::page::set_measure(self.config.measure_for(self.buffer.page_class()));
    }

    pub(super) fn start_fresh_document(&mut self) {
        self.park_active_buffer();
        self.buffer.start_fresh_doc(self.root.clone());
    }
}
