//! Live filesystem inputs for constructing and releveling the Files surface.

use std::path::{Path, PathBuf};

use crate::overlay::{GotoInputs, OverlayKind, OverlayState, PickerInput};

use super::overlay_inputs::root_relative;
use crate::app::location::ProjectLocation;

pub(super) struct FilesOverlayBuilder<'a> {
    location: &'a ProjectLocation,
    default_folder: &'a Path,
    active_path: Option<PathBuf>,
}

impl<'a> FilesOverlayBuilder<'a> {
    pub(super) fn new(
        location: &'a ProjectLocation,
        default_folder: &'a Path,
        active_path: Option<PathBuf>,
    ) -> Self {
        Self {
            location,
            default_folder,
            active_path,
        }
    }

    /// Summon and directory navigation share the cached root-wide roster.
    /// Gather lazily: ordinary query edits neither copy it nor read metadata.
    pub(super) fn inputs(&self) -> GotoInputs {
        let location = self.location;
        let recency_now = (location.root == crate::buffers::normalize_path(self.default_folder))
            .then(crate::clock::system_now);
        let (corpus, times) =
            crate::index::with_recency(&location.root, location.file_index.clone(), recency_now);
        let active_rel = self
            .active_path
            .as_deref()
            .and_then(|path| root_relative(path, &location.root));
        let open = corpus
            .iter()
            .enumerate()
            .filter(|(_, candidate)| Some(*candidate) == active_rel.as_ref())
            .map(|(index, _)| index)
            .collect();
        let recent = location
            .recent_files
            .iter()
            .filter_map(|path| root_relative(path, &location.root))
            .filter_map(|rel| corpus.iter().position(|candidate| *candidate == rel))
            .collect();
        GotoInputs {
            corpus,
            open,
            recent,
            times,
            headings: Vec::new(),
            line_count: 0,
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

    pub(super) fn browse(
        &self,
        kind: OverlayKind,
        rel: Option<String>,
        workspace: Option<&Path>,
        recent_projects: &[String],
    ) -> Option<OverlayState> {
        if kind == OverlayKind::Goto {
            let input = self.inputs();
            let mut overlay =
                OverlayState::new_files(input.corpus, input.open, input.recent, rel.clone());
            overlay.set_times(input.times);
            return Some(self.attach_level(overlay, rel.as_deref()));
        }
        crate::overlay::browse_level(kind, rel, &self.location.root, workspace, recent_projects)
    }

    fn attach_level(&self, mut overlay: OverlayState, rel: Option<&str>) -> OverlayState {
        overlay.set_files_root_name(crate::project::folder_name(&self.location.root));
        let prefix = rel
            .filter(|path| !path.is_empty())
            .map(|path| format!("{path}/"));
        let level = crate::index::try_list_dir_level(&self.location.root, rel);
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
