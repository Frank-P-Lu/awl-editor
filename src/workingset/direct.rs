//! The compact margin's file window, grouped by the shared root projection.

use std::path::{Path, PathBuf};

use super::panel::{PanelRow, group_stack_row};
use super::{RESTING_FILES, StackRow, WorkingSet};

/// Labels and identity are cached together; frame projection touches only the
/// visible file slots and headings, even with many open roots.
#[derive(Clone, Debug, Default)]
pub(super) struct Projection {
    order: Vec<usize>,
    headings: std::collections::HashMap<PathBuf, StackRow>,
    files: Vec<StackRow>,
}

impl WorkingSet {
    /// Rebuild only when working-set identity, order or activation changes.
    pub(super) fn refresh_direct(&mut self) {
        let full = self.expanded_full();
        let roots: Vec<&Path> = full
            .iter()
            .filter_map(|row| match row {
                PanelRow::Group(root, _) => Some(root.as_path()),
                PanelRow::File(_) => None,
            })
            .collect();
        let order = full
            .iter()
            .filter_map(|row| match row {
                PanelRow::File(at) => Some(*at),
                PanelRow::Group(..) => None,
            })
            .collect();
        let headings = full
            .iter()
            .filter_map(|row| match row {
                PanelRow::Group(root, active) => {
                    Some((root.clone(), group_stack_row(root, *active, &roots)))
                }
                PanelRow::File(_) => None,
            })
            .collect();
        let files = (0..self.files.len()).map(|at| self.file_row(at)).collect();
        self.direct_projection = Projection {
            order,
            headings,
            files,
        };
    }

    /// Keep an activated file visible with the smallest necessary slide.
    /// Ordinary scrolling never re-centres on it.
    pub(super) fn reveal_direct(&mut self) {
        self.refresh_direct();
        let Some(active) = self.active else {
            self.direct_scroll = 0;
            return;
        };
        let Some(position) = self
            .direct_projection
            .order
            .iter()
            .position(|&at| at == active)
        else {
            return;
        };
        let max = self.files.len().saturating_sub(RESTING_FILES);
        self.direct_scroll = self.direct_scroll.min(max);
        if position < self.direct_scroll {
            self.direct_scroll = position;
        } else if position >= self.direct_scroll + RESTING_FILES {
            self.direct_scroll = (position + 1 - RESTING_FILES).min(max);
        }
    }

    /// Five file slots, with each visible root's heading when several roots
    /// are open. Headings do not consume file slots or acquire file identity.
    pub(super) fn direct_window(&self) -> Vec<PanelRow> {
        let grouped = self.direct_projection.headings.len() > 1;
        let mut previous = None;
        let mut window = Vec::new();
        for &at in self
            .direct_projection
            .order
            .iter()
            .skip(self.direct_scroll)
            .take(RESTING_FILES)
        {
            let root = &self.files[at].root;
            if grouped && previous != Some(root) {
                window.push(PanelRow::Group(
                    root.clone(),
                    Some(root.as_path()) == self.active_root(),
                ));
            }
            window.push(PanelRow::File(at));
            previous = Some(root);
        }
        window
    }

    /// Roots and their file order share the expanded panel's projection.
    pub fn direct_rows(&self, _root: &Path) -> Vec<StackRow> {
        if self.files.len() < 2 {
            return Vec::new();
        }
        self.direct_window()
            .iter()
            .map(|row| match row {
                PanelRow::Group(root, _) => self.direct_projection.headings[root].clone(),
                PanelRow::File(at) => self.direct_projection.files[*at].clone(),
            })
            .collect()
    }

    /// Move by file slots without changing the active file.
    pub fn scroll_direct(&mut self, delta: isize) {
        let max = self.files.len().saturating_sub(RESTING_FILES);
        self.direct_scroll = self.direct_scroll.saturating_add_signed(delta).min(max);
    }

    /// Resolve clicks through the same projection that draws each row.
    pub fn direct_row_index(&self, row: usize) -> Option<usize> {
        match self.direct_window().get(row)? {
            PanelRow::File(at) => Some(*at),
            PanelRow::Group(..) => None,
        }
    }

    pub fn direct_row_group_root(&self, row: usize) -> Option<PathBuf> {
        match self.direct_window().get(row)? {
            PanelRow::Group(root, _) => Some(root.clone()),
            PanelRow::File(_) => None,
        }
    }
}
