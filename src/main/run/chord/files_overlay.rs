//! Hermetic filesystem inputs for constructing and releveling replay's Files surface.

use std::path::Path;

use crate::overlay::{OverlayKind, OverlayState, PickerInput};

pub(super) struct ReplayFilesBuilder<'a> {
    root: &'a Path,
    workspace: &'a Path,
    corpus: &'a [String],
}

impl<'a> ReplayFilesBuilder<'a> {
    pub(super) fn new(root: &'a Path, workspace: &'a Path, corpus: &'a [String]) -> Self {
        Self {
            root,
            workspace,
            corpus,
        }
    }

    pub(super) fn build(&self, kind: OverlayKind, input: &PickerInput<'_>) -> Option<OverlayState> {
        let overlay = crate::overlay::build_for(kind, input)?;
        Some(if kind == OverlayKind::Goto {
            self.attach_level(overlay, None)
        } else {
            overlay
        })
    }

    pub(super) fn browse(&self, kind: OverlayKind, rel: Option<String>) -> Option<OverlayState> {
        if kind == OverlayKind::Goto {
            let overlay =
                OverlayState::new_files(self.corpus.to_vec(), Vec::new(), Vec::new(), rel.clone());
            return Some(self.attach_level(overlay, rel.as_deref()));
        }
        // Recent projects are persisted live-only state; replay supplies an
        // empty roster so captures remain deterministic.
        crate::overlay::browse_level(kind, rel, self.root, Some(self.workspace), &[])
    }

    fn attach_level(&self, mut overlay: OverlayState, rel: Option<&str>) -> OverlayState {
        let prefix = rel
            .filter(|path| !path.is_empty())
            .map(|path| format!("{path}/"));
        let level = crate::index::try_list_dir_level(self.root, rel);
        let non_text =
            crate::overlay::non_text_level_files(rel, level.as_deref().unwrap_or_default());
        overlay.exclude_files(&non_text);
        overlay.attach_file_directories(
            level
                .clone()
                .unwrap_or_default()
                .into_iter()
                .filter(|entry| entry.is_dir)
                .map(|entry| match &prefix {
                    Some(prefix) => format!("{prefix}{}", entry.name),
                    None => entry.name,
                })
                .collect(),
        );
        overlay.set_files_level_state(level.as_deref());
        overlay
    }
}
