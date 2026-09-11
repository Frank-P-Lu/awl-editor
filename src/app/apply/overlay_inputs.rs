//! Live filesystem and document inputs gathered before the action core borrows
//! the active buffer mutably.

use crate::app::*;

type SpellTarget = (Vec<String>, (usize, usize, usize), String);

pub(super) struct OverlayInputs {
    pub(super) spell_target: Option<SpellTarget>,
    pub(super) history_entries: Vec<crate::history::TimelineRow>,
    pub(super) assets: Vec<crate::assets::Orphan>,
    pub(super) user_words: Vec<String>,
    pub(super) row_gates: crate::commands::RowGates,
    pub(super) search_root: std::path::PathBuf,
    pub(super) search_corpus: Vec<(String, String)>,
}

pub(super) struct GotoInputs {
    pub(super) goto_corpus: Vec<String>,
    pub(super) goto_times: Vec<String>,
    pub(super) goto_open: Vec<usize>,
    pub(super) goto_recent: Vec<usize>,
    pub(super) goto_headings: Vec<(String, usize)>,
    pub(super) goto_line_count: usize,
}

/// `path` root-relativized against `root` (`/`-separated, matching every
/// `goto_corpus` entry's own spelling) — the ONE comparison owner for
/// `gather_goto_inputs`'s two identity checks below. `path` is canonicalized
/// FIRST, through the same [`crate::buffers::normalize_path`] `root` (already
/// canonical — [`crate::app::ProjectLocation::new`]/`App::set_root`) was
/// resolved through: `Buffer::path()` and a persisted `recent_files` entry
/// both carry whatever spelling they were OPENED under (a raw CLI argument, a
/// native file-chooser result, an alias-joined path) and are never themselves
/// canonicalized at rest — see `workingset::root_for`'s doc for why the
/// STORED path stays the display spelling and the canonicalizing happens at
/// the comparison instead. Without this, a symlinked/firmlinked spelling of a
/// file genuinely under `root` (macOS's `/tmp` -> `/private/tmp`, the same
/// alias class `App::set_root` already resolves) fails `strip_prefix` and the
/// file silently drops out of both the active-file marker and the Recent lens
/// bucket, though it is really there.
fn root_relative(path: &std::path::Path, root: &std::path::Path) -> Option<String> {
    crate::buffers::normalize_path(path)
        .strip_prefix(root)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
}

impl App {
    pub(super) fn gather_goto_inputs(&mut self, action: &Action) -> GotoInputs {
        if matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenAssetClean
                | Action::OpenSearchFolder
        ) {
            self.rescan_file_index();
        }
        let location = &self.project_location;
        let recency_now = (location.root
            == crate::buffers::normalize_path(&self.config.default_folder))
        .then(crate::clock::system_now);
        let (goto_corpus, goto_times) =
            crate::index::with_recency(&location.root, location.file_index.clone(), recency_now);
        let active_rel = self
            .document
            .buffer_opt()
            .and_then(|buffer| buffer.path())
            .and_then(|path| root_relative(path, &location.root));
        let goto_open = goto_corpus
            .iter()
            .enumerate()
            .filter(|(_, candidate)| Some(*candidate) == active_rel.as_ref())
            .map(|(index, _)| index)
            .collect();
        let goto_recent = location
            .recent_files
            .iter()
            .filter_map(|path| root_relative(path, &location.root))
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

    pub(super) fn gather_picker_input<'a>(
        &mut self,
        action: &Action,
        project_root: &std::path::Path,
        config_keys: &'a [(String, Vec<String>)],
        config_linux_keep: &'a [String],
        config_keymap_flavor: crate::keymap::KeymapFlavor,
    ) -> Option<crate::overlay::PickerInput<'a>> {
        let kind = crate::overlay::picker_kind_for(
            action,
            self.workspace_state.journey().parked_kind(),
            self.workspace_state
                .overlay()
                .is_some_and(|overlay| overlay.kind == crate::overlay::OverlayKind::Settings),
        )?;
        if matches!(
            kind,
            crate::overlay::OverlayKind::Goto
                | crate::overlay::OverlayKind::Spell
                | crate::overlay::OverlayKind::History
                | crate::overlay::OverlayKind::Assets
                | crate::overlay::OverlayKind::UserWords
                | crate::overlay::OverlayKind::SearchFolder
        ) {
            self.gather_content_picker_input(kind, action)
        } else {
            self.gather_config_picker_input(
                kind,
                action,
                project_root,
                config_keys,
                config_linux_keep,
                config_keymap_flavor,
            )
        }
    }

    fn gather_content_picker_input<'a>(
        &mut self,
        kind: crate::overlay::OverlayKind,
        action: &Action,
    ) -> Option<crate::overlay::PickerInput<'a>> {
        match kind {
            crate::overlay::OverlayKind::Goto => {
                let GotoInputs {
                    goto_corpus,
                    goto_times,
                    goto_open,
                    goto_recent,
                    goto_headings,
                    goto_line_count,
                } = self.gather_goto_inputs(action);
                Some(crate::overlay::PickerInput::Goto(
                    crate::overlay::GotoInputs {
                        corpus: goto_corpus,
                        open: goto_open,
                        recent: goto_recent,
                        times: goto_times,
                        headings: goto_headings,
                        line_count: goto_line_count,
                    },
                ))
            }
            crate::overlay::OverlayKind::Spell => {
                let OverlayInputs { spell_target, .. } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::Spell(spell_target))
            }
            crate::overlay::OverlayKind::History => {
                let OverlayInputs {
                    history_entries, ..
                } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::History(
                    crate::overlay::HistoryInputs {
                        entries: history_entries,
                        now: Some(crate::history::now_millis()),
                        session_start: crate::history::session_epoch_ms(),
                    },
                ))
            }
            crate::overlay::OverlayKind::Assets => {
                let OverlayInputs { assets, .. } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::Assets(assets))
            }
            crate::overlay::OverlayKind::UserWords => {
                let OverlayInputs { user_words, .. } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::UserWords(user_words))
            }
            crate::overlay::OverlayKind::SearchFolder => {
                let OverlayInputs {
                    search_root,
                    search_corpus,
                    ..
                } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::SearchFolder(
                    crate::overlay::SearchFolderInputs {
                        root: search_root,
                        corpus: search_corpus,
                    },
                ))
            }
            _ => unreachable!("content picker routing admitted {kind:?}"),
        }
    }

    fn gather_config_picker_input<'a>(
        &mut self,
        kind: crate::overlay::OverlayKind,
        action: &Action,
        project_root: &std::path::Path,
        config_keys: &'a [(String, Vec<String>)],
        config_linux_keep: &'a [String],
        config_keymap_flavor: crate::keymap::KeymapFlavor,
    ) -> Option<crate::overlay::PickerInput<'a>> {
        let settings_values = |app: &Self| {
            crate::settings::SettingsValues::gather(
                &app.config,
                project_root,
                app.frame.zoom(),
                crate::dateformat::today_from_system_clock(),
            )
        };
        match kind {
            crate::overlay::OverlayKind::Theme => Some(crate::overlay::PickerInput::Theme),
            crate::overlay::OverlayKind::Caret => Some(crate::overlay::PickerInput::Caret),
            crate::overlay::OverlayKind::Dictionary => {
                Some(crate::overlay::PickerInput::Dictionary)
            }
            crate::overlay::OverlayKind::Keymap => Some(crate::overlay::PickerInput::Keymap {
                configured: self.config.keymap.clone().unwrap_or_default(),
            }),
            crate::overlay::OverlayKind::Command => {
                let OverlayInputs { row_gates, .. } = self.gather_overlay_inputs(action);
                Some(crate::overlay::PickerInput::Command(
                    crate::overlay::CommandInputs {
                        bindings: crate::overlay::BindingInputs {
                            keys: config_keys,
                            linux_keep: config_linux_keep,
                            keymap_flavor: config_keymap_flavor,
                        },
                        settings_values: settings_values(self),
                        row_gates,
                    },
                ))
            }
            crate::overlay::OverlayKind::Keybindings => Some(
                crate::overlay::PickerInput::Keybindings(crate::overlay::BindingInputs {
                    keys: config_keys,
                    linux_keep: config_linux_keep,
                    keymap_flavor: config_keymap_flavor,
                }),
            ),
            crate::overlay::OverlayKind::Settings => {
                Some(crate::overlay::PickerInput::Settings(settings_values(self)))
            }
            crate::overlay::OverlayKind::Credits => Some(crate::overlay::PickerInput::Credits),
            crate::overlay::OverlayKind::Goto
            | crate::overlay::OverlayKind::Spell
            | crate::overlay::OverlayKind::History
            | crate::overlay::OverlayKind::Assets
            | crate::overlay::OverlayKind::UserWords
            | crate::overlay::OverlayKind::SearchFolder => {
                unreachable!("content picker routing bypassed its owner")
            }
            crate::overlay::OverlayKind::CjkLang
            | crate::overlay::OverlayKind::Date
            | crate::overlay::OverlayKind::Browse
            | crate::overlay::OverlayKind::MoveDest
            | crate::overlay::OverlayKind::ExportDest
            | crate::overlay::OverlayKind::Project
            | crate::overlay::OverlayKind::ProjectBrowse
            | crate::overlay::OverlayKind::Rename
            | crate::overlay::OverlayKind::InsertLink
            | crate::overlay::OverlayKind::KeepName
            | crate::overlay::OverlayKind::Context
            | crate::overlay::OverlayKind::TableDims
            | crate::overlay::OverlayKind::Conflict => None,
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
        // The live checker's own personal set, gathered only when the picker
        // was summoned. No filesystem rescan: the file was folded into the
        // checker at launch and every add/forget keeps the two in step.
        let user_words = if matches!(action, Action::OpenUserWords) {
            self.document.user_words_sorted()
        } else {
            Vec::new()
        };
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
        let has_waiter = self
            .document
            .buffer_opt()
            .map(crate::buffers::BufferKey::of)
            .is_some_and(|key| {
                self.wait_conns
                    .get(&key)
                    .is_some_and(|waiters| !waiters.is_empty())
            });
        #[cfg(any(target_arch = "wasm32", feature = "mas"))]
        let has_waiter = false;
        // SEARCH IN FOLDER: read every candidate file's content, bounded, ONLY
        // when the search binding fired -- reading a whole project's text is
        // pure waste otherwise. `file_index` is already fresh (the rescan
        // above, shared with Goto/Assets); `refilter` re-matches this same
        // loaded corpus against the query on every keystroke, never re-reading
        // disk (`crate::search_folder`'s own module doc).
        let (search_root, search_corpus) = if matches!(action, Action::OpenSearchFolder) {
            let root = self.project_location.root.clone();
            let files = self.project_location.file_index.clone();
            let corpus = crate::search_folder::load_corpus(
                &files,
                &crate::search_folder::SearchBudget::default(),
                |rel| {
                    crate::fs::active()
                        .read_to_string(&crate::index::resolve(&root, rel))
                        .ok()
                },
            );
            (root, corpus)
        } else {
            (std::path::PathBuf::new(), Vec::new())
        };
        OverlayInputs {
            spell_target,
            history_entries,
            assets,
            user_words,
            row_gates: crate::commands::RowGates {
                has_waiter,
                change_unresolved: self.change_unresolved(),
                named_file: self
                    .document
                    .buffer_opt()
                    .is_some_and(|buffer| buffer.path().is_some()),
            },
            search_root,
            search_corpus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::root_relative;

    /// **A THIRD INSTANCE of 512(b)'s bug class**, in the Go-to picker's
    /// active-file marker and Recent-lens membership: `gather_goto_inputs`
    /// used to `strip_prefix` a raw `Buffer::path()`/`recent_files` entry —
    /// whatever spelling the file was opened under — against the already-
    /// canonical `location.root`, so a symlinked/firmlinked alias of a file
    /// genuinely under the root (macOS's `/tmp` -> `/private/tmp`, the same
    /// class `App::set_root` already resolves) silently failed the prefix
    /// match: the active file lost its ranking bias, and a recent file
    /// dropped out of the Recent lens bucket entirely (`overlay::filter`'s
    /// `recent` flag gates both ranking AND facet-bucket membership, not
    /// only tiebreak order).
    ///
    /// Real symlinked directory (`normalize_path` reaches real disk directly
    /// via `std::fs::canonicalize`, so a fabricated pair of strings would
    /// prove nothing) — the same precedent
    /// `buffer_key_path_resolves_a_symlinked_directory_to_the_real_path` /
    /// `a_symlinked_alias_of_a_root_normalizes_to_the_same_root_for_answer`
    /// already establish for the sibling identities this bug class keeps
    /// turning up in.
    #[test]
    #[cfg(unix)]
    fn root_relative_resolves_a_symlinked_alias_to_the_canonical_roots_relative_spelling() {
        let _guard = crate::testlock::serial();
        let base = crate::testscratch::ScratchDir::new(std::env::temp_dir().join(format!(
            "awl-goto-root-relative-alias-{}",
            std::process::id()
        )));
        let real_dir = base.join("real");
        let link_dir = base.join("link");
        std::fs::create_dir_all(real_dir.join("sub")).unwrap();
        std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();
        std::fs::write(real_dir.join("sub/a.md"), "a").unwrap();

        let canon_root = crate::buffers::normalize_path(&real_dir);
        // The file reached THROUGH THE ALIAS (never typed under the real
        // spelling) must still relativize against the canonical root, or
        // an open buffer / recent-files entry that happened to be opened
        // via the alias would drop out of both the active marker and the
        // Recent lens.
        let via_alias = link_dir.join("sub/a.md");
        assert_eq!(
            root_relative(&via_alias, &canon_root),
            Some("sub/a.md".to_string()),
            "a file reached through a symlinked alias of the root must still \
             resolve to its canonical root-relative spelling"
        );
        // Non-vacuity: prove the alias and the real spelling actually differ
        // as raw strings, or the assertion above could pass by coincidence
        // rather than by the canonicalization actually running.
        assert_ne!(
            via_alias,
            real_dir.join("sub/a.md"),
            "the alias spelling must genuinely differ from the real one, or this \
             law never exercises the canonicalization it exists to prove"
        );
    }

    #[test]
    fn root_relative_is_none_for_a_path_genuinely_outside_the_root() {
        assert_eq!(
            root_relative(
                std::path::Path::new("/elsewhere/a.md"),
                std::path::Path::new("/root")
            ),
            None
        );
    }
}
