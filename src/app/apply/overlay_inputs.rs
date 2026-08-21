//! Live filesystem and document inputs gathered before the action core borrows
//! the active buffer mutably.

use crate::app::*;

type SpellTarget = (Vec<String>, (usize, usize, usize), String);

pub(super) struct OverlayInputs {
    pub(super) spell_target: Option<SpellTarget>,
    pub(super) history_entries: Vec<crate::history::TimelineRow>,
    pub(super) assets: Vec<crate::assets::Orphan>,
    pub(super) row_gates: crate::commands::RowGates,
}

pub(super) struct GotoInputs {
    pub(super) goto_corpus: Vec<String>,
    pub(super) goto_times: Vec<String>,
    pub(super) goto_open: Vec<usize>,
    pub(super) goto_recent: Vec<usize>,
    pub(super) goto_headings: Vec<(String, usize)>,
    pub(super) goto_line_count: usize,
}

impl App {
    pub(super) fn gather_goto_inputs(&mut self, action: &Action) -> GotoInputs {
        if matches!(action, Action::OpenGoto | Action::OpenAssetClean) {
            self.rescan_file_index();
        }
        let location = &self.project_location;
        let recency_now =
            (location.root == self.config.default_folder).then(crate::clock::system_now);
        let (goto_corpus, goto_times) =
            crate::index::with_recency(&location.root, location.file_index.clone(), recency_now);
        let active_rel = self
            .document
            .buffer_opt()
            .and_then(|buffer| buffer.path())
            .and_then(|path| {
                path.strip_prefix(&location.root)
                    .ok()
                    .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            });
        let goto_open = goto_corpus
            .iter()
            .enumerate()
            .filter(|(_, candidate)| Some(*candidate) == active_rel.as_ref())
            .map(|(index, _)| index)
            .collect();
        let goto_recent = location
            .recent_files
            .iter()
            .filter_map(|path| path.strip_prefix(&location.root).ok())
            .map(|rel| rel.to_string_lossy().replace('\\', "/"))
            .filter_map(|rel| goto_corpus.iter().position(|candidate| *candidate == rel))
            .collect();
        let goto_headings = if matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenOutline
        ) && self
            .document
            .buffer_opt()
            .is_some_and(crate::buffer::Buffer::is_markdown)
        {
            crate::markdown::headings(&self.document.buffer_opt().unwrap().text())
                .into_iter()
                .map(|heading| (heading.label(), heading.line))
                .collect()
        } else {
            Vec::new()
        };
        // Go to Line's numeric companion: ANY buffer, not only markdown --
        // the same summon gate as `goto_headings`, minus the markdown check.
        let goto_line_count = if matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenOutline
        ) {
            self.document
                .buffer_opt()
                .map_or(0, crate::buffer::Buffer::line_count)
        } else {
            0
        };
        GotoInputs {
            goto_corpus,
            goto_times,
            goto_open,
            goto_recent,
            goto_headings,
            goto_line_count,
        }
    }

    pub(super) fn gather_overlay_inputs(&mut self, action: &Action) -> OverlayInputs {
        let spell_target =
            if matches!(action, Action::OpenSpellSuggest) && self.document.has_active() {
                let (line, col) = self.document.buffer().cursor_line_col();
                self.document
                    .spell_suggestion_target(line, col)
                    .map(|target| {
                        (
                            target.suggestions,
                            (
                                target.misspelling.line,
                                target.misspelling.start_col,
                                target.misspelling.end_col,
                            ),
                            target.word,
                        )
                    })
            } else {
                None
            };
        let history_entries = if matches!(action, Action::OpenHistory | Action::CompareVersion)
            && self.document.has_active()
        {
            crate::history::source_path(
                self.document.buffer().path(),
                self.document.buffer().is_unnamed_fresh(),
            )
            .map(|path| {
                crate::history::timeline_rows(
                    &path,
                    &self.document.buffer().text(),
                    crate::history::now_millis(),
                )
            })
            .unwrap_or_default()
        } else {
            Vec::new()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let assets = if matches!(action, Action::OpenAssetClean) {
            crate::assets::scan(
                &self.project_location.root,
                &self.project_location.file_index,
            )
        } else {
            Vec::new()
        };
        #[cfg(target_arch = "wasm32")]
        let assets = Vec::new();
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        let has_waiter = self
            .document
            .buffer_opt()
            .and_then(crate::buffers::BufferKey::of)
            .is_some_and(|key| {
                self.wait_conns
                    .get(&key)
                    .is_some_and(|waiters| !waiters.is_empty())
            });
        #[cfg(any(target_arch = "wasm32", feature = "mas"))]
        let has_waiter = false;
        OverlayInputs {
            spell_target,
            history_entries,
            assets,
            row_gates: crate::commands::RowGates {
                has_waiter,
                change_unresolved: self.change_unresolved(),
                named_file: self
                    .document
                    .buffer_opt()
                    .is_some_and(|buffer| buffer.path().is_some()),
            },
        }
    }

    pub(super) fn gather_goto_folders(
        &self,
        action: &Action,
    ) -> (Vec<(String, bool)>, Vec<String>) {
        if !matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenOutline
        ) {
            return (Vec::new(), Vec::new());
        }
        let recent: Vec<String> = self
            .project_location
            .recent_projects
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        crate::overlay::goto_folder_roster(self.project_location.workspace_root.as_deref(), &recent)
    }
}
