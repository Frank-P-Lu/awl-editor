//! The compact margin's direct window over every open file.

use std::path::Path;

use super::{RESTING_FILES, StackRow, WorkingSet};

impl WorkingSet {
    /// Keep an activated file visible with the smallest necessary slide.
    /// Ordinary scrolling uses a separate door and never re-centres on it.
    pub(super) fn reveal_direct(&mut self) {
        let Some(active) = self.active else {
            self.direct_scroll = 0;
            return;
        };
        let max = self.files.len().saturating_sub(RESTING_FILES);
        self.direct_scroll = self.direct_scroll.min(max);
        if active < self.direct_scroll {
            self.direct_scroll = active;
        } else if active >= self.direct_scroll + RESTING_FILES {
            self.direct_scroll = (active + 1 - RESTING_FILES).min(max);
        }
    }

    /// The compact margin shows every open file in stable order. A file from
    /// another root carries that root's name in its quiet location label.
    pub fn direct_rows(&self, root: &Path) -> Vec<StackRow> {
        if self.files.len() < 2 {
            return Vec::new();
        }
        let end = (self.direct_scroll + RESTING_FILES).min(self.files.len());
        (self.direct_scroll..end)
            .map(|at| {
                let mut row = self.file_row(at);
                let file = &self.files[at];
                if file.root != root {
                    row.parent =
                        format!("{}/{}", crate::project::folder_name(&file.root), row.parent);
                }
                row
            })
            .collect()
    }

    /// Move the compact window by file rows, without changing the active file.
    pub fn scroll_direct(&mut self, delta: isize) {
        let max = self.files.len().saturating_sub(RESTING_FILES);
        self.direct_scroll = self.direct_scroll.saturating_add_signed(delta).min(max);
    }

    /// Resolve the drawn row through the same window `direct_rows` displays.
    pub fn direct_row_index(&self, row: usize) -> Option<usize> {
        if row >= RESTING_FILES {
            return None;
        }
        let at = self.direct_scroll + row;
        (at < self.files.len()).then_some(at)
    }
}
