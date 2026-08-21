//! Capture-only projections for the working-set design decision.
//!
//! The production model owns stable order, file/root truth and the active slot.
//! This module auditions three presentations of that SAME state without making
//! any of them the product rule. It is reached only while folding a
//! `--screenshot-app` artifact and only when the sealed prototype environment
//! key is present; an ordinary live frame never asks it a question.

use std::path::Path;

use super::{StackRow, StackRowKind, WorkingSet};

const RESTING_FILES: usize = 5;
const EXPANDED_FILES: usize = 8;

/// Capture-only candidate rows for the already-existing Move destination card.
/// The production navigator currently expresses "move here" only in its footer
/// and has no discoverable new-folder row; this makes both alternatives visible
/// for the user-judgment capture without changing the live action grammar.
pub fn prototype_move_rows(existing_folders: &[String]) -> Vec<String> {
    let mut rows = Vec::with_capacity(existing_folders.len() + 2);
    rows.push("Move here".to_string());
    rows.push("New folder…".to_string());
    rows.extend(existing_folders.iter().cloned());
    rows
}

pub fn prototype_move_from_env() -> bool {
    std::env::var("AWL_WORKING_SET_PROTOTYPE_MOVE").as_deref() == Ok("1")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrototypeSpec {
    Collapsed { hover: Option<usize> },
    Expanded { scroll: usize, hover: Option<usize> },
    Grouped { hover: Option<usize> },
}

impl PrototypeSpec {
    /// Read the sealed prototype pose. This is deliberately not a config key or
    /// CLI flag: it cannot become persisted product state by accident.
    pub fn from_env() -> Option<Self> {
        let mode = std::env::var("AWL_WORKING_SET_PROTOTYPE").ok()?;
        let hover = std::env::var("AWL_WORKING_SET_PROTOTYPE_HOVER")
            .ok()
            .and_then(|v| v.parse().ok());
        match mode.as_str() {
            "collapsed" => Some(Self::Collapsed { hover }),
            "expanded" => Some(Self::Expanded {
                scroll: std::env::var("AWL_WORKING_SET_PROTOTYPE_SCROLL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                hover,
            }),
            "grouped" => Some(Self::Grouped { hover }),
            _ => None,
        }
    }

    pub fn mode(self) -> &'static str {
        match self {
            Self::Collapsed { .. } => "collapsed",
            Self::Expanded { .. } => "expanded",
            Self::Grouped { .. } => "grouped",
        }
    }

    fn hover(self) -> Option<usize> {
        match self {
            Self::Collapsed { hover } | Self::Expanded { hover, .. } | Self::Grouped { hover } => {
                hover
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrototypeReport {
    pub mode: &'static str,
    pub total_open: usize,
    pub total_file_rows: usize,
    pub visible_file_rows: usize,
    pub hidden: usize,
    pub scroll: usize,
    pub viewport: usize,
    pub active_row: Option<usize>,
    pub hovered_row: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrototypeView {
    pub rows: Vec<StackRow>,
    pub report: PrototypeReport,
}

impl WorkingSet {
    /// Project the real working set into one audition pose.
    ///
    /// `Collapsed` uses one concrete candidate for the undecided window rule:
    /// keep the first five stable slots until the active file would leave, then
    /// move the smallest contiguous stable-order window that includes it. The
    /// name and report make this a candidate to judge, not a silently shipped
    /// rule.
    pub fn prototype_view(&self, spec: PrototypeSpec) -> PrototypeView {
        match spec {
            PrototypeSpec::Collapsed { .. } => self.prototype_collapsed(spec),
            PrototypeSpec::Expanded { scroll, .. } => self.prototype_expanded(spec, scroll),
            PrototypeSpec::Grouped { .. } => self.prototype_grouped(spec),
        }
    }

    fn prototype_collapsed(&self, spec: PrototypeSpec) -> PrototypeView {
        let group = self
            .active_root()
            .map(|root| self.group(root))
            .unwrap_or_default();
        let active_in_group = self
            .active_index()
            .and_then(|active| group.iter().position(|at| *at == active));
        let max_start = group.len().saturating_sub(RESTING_FILES);
        let start = active_in_group
            .map(|active| active.saturating_sub(RESTING_FILES - 1).min(max_start))
            .unwrap_or(0);
        let visible = &group[start..(start + RESTING_FILES).min(group.len())];
        let mut rows = visible
            .iter()
            .map(|at| self.file_row(*at))
            .collect::<Vec<_>>();
        let hidden = self.len().saturating_sub(visible.len());
        if hidden > 0 {
            rows.push(StackRow {
                leaf: format!("+ {hidden} more…"),
                kind: StackRowKind::More { hidden },
                ..StackRow::default()
            });
        }
        self.finish(spec, rows, group.len(), hidden, start, RESTING_FILES)
    }

    fn prototype_expanded(&self, spec: PrototypeSpec, requested_scroll: usize) -> PrototypeView {
        let group = self
            .active_root()
            .map(|root| self.group(root))
            .unwrap_or_default();
        let max_scroll = group.len().saturating_sub(EXPANDED_FILES);
        let scroll = requested_scroll.min(max_scroll);
        let visible = &group[scroll..(scroll + EXPANDED_FILES).min(group.len())];
        let rows = visible
            .iter()
            .map(|at| self.file_row(*at))
            .collect::<Vec<_>>();
        self.finish(
            spec,
            rows,
            group.len(),
            group.len().saturating_sub(visible.len()),
            scroll,
            EXPANDED_FILES,
        )
    }

    fn prototype_grouped(&self, spec: PrototypeSpec) -> PrototypeView {
        let active_root = self.active_root();
        let mut roots = Vec::new();
        for file in self.files() {
            if !roots
                .iter()
                .any(|root: &&Path| *root == file.root.as_path())
            {
                roots.push(file.root.as_path());
            }
        }
        let mut rows = Vec::new();
        for root in roots {
            let active = active_root == Some(root);
            rows.push(StackRow {
                leaf: crate::project::folder_name(root),
                kind: StackRowKind::Group { active },
                ..StackRow::default()
            });
            rows.extend(self.group(root).into_iter().map(|at| self.file_row(at)));
        }
        self.finish(spec, rows, self.len(), 0, 0, self.len())
    }

    fn file_row(&self, at: usize) -> StackRow {
        StackRow {
            leaf: self.files[at].leaf(),
            parent: self.files[at].parent_label().unwrap_or_default(),
            active: self.active_index() == Some(at),
            kind: StackRowKind::File,
            prototype_hovered: false,
        }
    }

    fn finish(
        &self,
        spec: PrototypeSpec,
        mut rows: Vec<StackRow>,
        total_file_rows: usize,
        hidden: usize,
        scroll: usize,
        viewport: usize,
    ) -> PrototypeView {
        let hover = spec.hover().filter(|at| {
            rows.get(*at)
                .is_some_and(|row| matches!(row.kind, StackRowKind::File))
        });
        if let Some(at) = hover {
            rows[at].prototype_hovered = true;
        }
        let active_row = rows.iter().position(|row| row.active);
        let visible_file_rows = rows
            .iter()
            .filter(|row| matches!(row.kind, StackRowKind::File))
            .count();
        PrototypeView {
            rows,
            report: PrototypeReport {
                mode: spec.mode(),
                total_open: self.len(),
                total_file_rows,
                visible_file_rows,
                hidden,
                scroll,
                viewport,
                active_row,
                hovered_row: hover,
            },
        }
    }
}

#[cfg(test)]
mod tests;
