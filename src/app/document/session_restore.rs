//! Native session snapshots and restoration for document slots.

use super::*;

impl DocumentSession {
    /// Restore an explicitly empty previous session. The launch scratch exists
    /// only to keep first launch unchanged; it is not enrolled as a document in
    /// the restored working set.
    pub(in crate::app) fn restore_no_document(&mut self) {
        if let Some(mut active) = self.take_active() {
            active.buffer.take_list_continuation_generated();
        }
        self.registry = crate::buffers::BufferRegistry::default();
        self.working = crate::workingset::WorkingSet::default();
    }

    /// Every open PATHED file's remembered position, in the SAME order
    /// [`crate::workingset::WorkingSet`] draws the margin from — the one
    /// order every consumer (the resting stack, the expanded panel, a drag)
    /// reads, so session restore is not a second, silently-MRU-ordered list.
    /// Iterating `registry.iter()` directly (the old shape) read the
    /// registry's own MRU order instead, which drifts from the drawn order
    /// the moment a background row is switched to and back — the working
    /// set's whole reason for existing separately from the registry (this
    /// module's own doc).
    pub(in crate::app) fn session_buffers(&self) -> Vec<(PathBuf, crate::session::BufferPos)> {
        self.working
            .files()
            .iter()
            .filter_map(|file| {
                let path = file.path.clone()?;
                let pos = self.entry_pos(&file.key)?;
                Some((path, pos))
            })
            .collect()
    }

    /// The cursor/scroll a session save records for `key` — the active entry
    /// when `key` names it, else the parked one. `None` only if a working-set
    /// row somehow names neither, which the removal owner's "both halves or
    /// neither" discipline (`entries.rs::discard`) should make unreachable.
    fn entry_pos(&self, key: &crate::buffers::BufferKey) -> Option<crate::session::BufferPos> {
        let entry = if self.active_key().as_ref() == Some(key) {
            self.active.as_ref()
        } else {
            self.registry.get(key)
        }?;
        let (line, col) = entry.buffer.cursor_line_col();
        Some(crate::session::BufferPos {
            line,
            col,
            scroll: entry.extra.scroll.row,
            scroll_px_q: entry.extra.scroll.px_q,
        })
    }

    pub(in crate::app) fn restore_active(
        &mut self,
        path: &Path,
        pos: crate::session::BufferPos,
        seen: crate::external::Seen,
    ) {
        let mut buffer = Buffer::from_file(path);
        apply_restored_pos(&mut buffer, pos);
        let version = buffer.version();
        self.active = Some(crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                scroll: crate::render::ScrollPos {
                    row: pos.scroll,
                    px_q: pos.scroll_px_q,
                },
                doc_saved_version: Some(version),
                disk_baseline: seen,
                caret_synced_version: version,
                ..Default::default()
            },
        });
    }

    pub(in crate::app) fn restore_background(
        &mut self,
        path: &Path,
        pos: crate::session::BufferPos,
        seen: crate::external::Seen,
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
            disk_baseline: seen,
            caret_synced_version: version,
            ..Default::default()
        };
        self.registry.park(
            crate::buffers::BufferKey::path(path),
            crate::buffers::Entry { buffer, extra },
        );
    }
}

fn apply_restored_pos(buffer: &mut Buffer, pos: crate::session::BufferPos) {
    let idx = buffer.line_col_to_char(pos.line, pos.col);
    buffer.clear_mark();
    buffer.set_cursor(idx);
}
