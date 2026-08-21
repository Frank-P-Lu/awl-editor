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

    pub(in crate::app) fn session_buffers(&self) -> Vec<(PathBuf, crate::session::BufferPos)> {
        let mut buffers = Vec::new();
        if let Some(active) = self.active.as_ref()
            && let Some(path) = active.buffer.path()
        {
            let (line, col) = active.buffer.cursor_line_col();
            buffers.push((
                path.to_path_buf(),
                crate::session::BufferPos {
                    line,
                    col,
                    scroll: active.extra.scroll.row,
                    scroll_px_q: active.extra.scroll.px_q,
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
