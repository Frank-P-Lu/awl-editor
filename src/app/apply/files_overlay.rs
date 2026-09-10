//! Live filesystem inputs for constructing and releveling the Files surface.

use std::path::{Path, PathBuf};

use crate::overlay::{BuildCtx, OverlayKind, OverlayState};

pub(super) struct FilesOverlayBuilder {
    root: PathBuf,
    corpus: Vec<String>,
    open: Vec<usize>,
    recent: Vec<usize>,
}

impl FilesOverlayBuilder {
    pub(super) fn new(root: PathBuf, ctx: &BuildCtx<'_>) -> Self {
        Self {
            root,
            corpus: ctx.goto_corpus.clone(),
            open: ctx.goto_open.clone(),
            recent: ctx.goto_recent.clone(),
        }
    }

    pub(super) fn build(&self, kind: OverlayKind, ctx: &BuildCtx<'_>) -> Option<OverlayState> {
        let overlay = crate::overlay::build(kind, ctx)?;
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
            let overlay = OverlayState::new_files(
                self.corpus.clone(),
                self.open.clone(),
                self.recent.clone(),
                rel.clone(),
            );
            return Some(self.attach_level(overlay, rel.as_deref()));
        }
        crate::overlay::browse_level(kind, rel, &self.root, workspace, recent_projects)
    }

    fn attach_level(&self, mut overlay: OverlayState, rel: Option<&str>) -> OverlayState {
        let prefix = rel
            .filter(|path| !path.is_empty())
            .map(|path| format!("{path}/"));
        let level = crate::index::try_list_dir_level(&self.root, rel);
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
