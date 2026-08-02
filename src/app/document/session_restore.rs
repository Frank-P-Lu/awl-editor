//! Native session snapshots and restoration for document slots.

use super::*;

impl DocumentSession {
    pub(in crate::app) fn session_buffers(&self) -> Vec<(PathBuf, crate::session::BufferPos)> {
        let mut buffers = Vec::new();
        if let Some(path) = self.active.buffer.path() {
            let (line, col) = self.active.buffer.cursor_line_col();
            buffers.push((
                path.to_path_buf(),
                crate::session::BufferPos {
                    line,
                    col,
                    scroll: self.active.extra.scroll.row,
                    scroll_px_q: self.active.extra.scroll.px_q,
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
        mtime: Option<crate::fs::Metadata>,
    ) {
        let mut buffer = Buffer::from_file(path);
        apply_restored_pos(&mut buffer, pos);
        let version = buffer.version();
        self.active = crate::buffers::Entry {
            buffer,
            extra: BufferExtra {
                scroll: crate::render::ScrollPos {
                    row: pos.scroll,
                    px_q: pos.scroll_px_q,
                },
                doc_saved_version: Some(version),
                disk_mtime: mtime,
                caret_synced_version: version,
                ..Default::default()
            },
        };
    }

    pub(in crate::app) fn restore_background(
        &mut self,
        path: &Path,
        pos: crate::session::BufferPos,
        mtime: Option<crate::fs::Metadata>,
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
            disk_mtime: mtime,
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
