//! One replayed chord: resolve, apply, and settle.
//!
//! Resolution owns the search-before-keymap ordering and derives shift intent
//! once. Application owns the shared depth-first effect worklist. Each action
//! inside that worklist still crosses the single [`actions::apply_transition`]
//! seam after gathering the caller-owned overlay inputs it needs.

use super::*;
#[path = "chord/files_overlay.rs"]
mod files_overlay;

struct ResolvedChord {
    action: Action,
    shift: bool,
}

impl ReplaySession<'_> {
    pub(crate) fn apply_chord(&mut self, chord: &crate::keyspec::Chord) -> Result<()> {
        let Some(resolved) = self.resolve_chord(chord)? else {
            return Ok(());
        };
        self.apply_resolved_chord(chord, resolved)?;
        self.arm_hover_baseline();
        Ok(())
    }

    /// Search sees the chord before the keymap; a keymap prefix yields no
    /// action; otherwise shift-selection intent is derived once from the first
    /// resolved action and the physical key, then carried through nested
    /// palette re-dispatch unchanged.
    fn resolve_chord(&mut self, chord: &crate::keyspec::Chord) -> Result<Option<ResolvedChord>> {
        if self.search.is_some() {
            let _ = crate::search::keys::intercept(
                &mut self.search,
                self.buffer,
                &chord.key,
                chord.mods.state(),
            );
            self.record_search_trace(chord);
            return Ok(None);
        }

        let Some(action) = self.resolver.resolve(chord)? else {
            self.record_prefix_trace(chord);
            return Ok(None);
        };
        let shift = chord
            .mods
            .state()
            .contains(winit::keyboard::ModifiersState::SHIFT)
            && crate::app::motion_honors_shift_select(&action, &chord.key);
        Ok(Some(ResolvedChord { action, shift }))
    }

    /// Run one resolved chord through the shared depth-first worklist. A nested
    /// `RunAction` is applied before the outer transition's remaining effects;
    /// only the worklist owns that ordering.
    fn apply_resolved_chord(
        &mut self,
        chord: &crate::keyspec::Chord,
        resolved: ResolvedChord,
    ) -> Result<()> {
        let mut work = actions::EffectWorklist::root(resolved.action);
        let mut pending_return_to = None;
        while let Some(item) = work.next() {
            match item {
                actions::EffectWorkItem::Action(action) => self.apply_action_transition(
                    chord,
                    action,
                    resolved.shift,
                    &mut work,
                    &mut pending_return_to,
                ),
                actions::EffectWorkItem::Effect { owner, effect } => {
                    self.interpret_effect(&owner, chord, effect, &mut work, &mut pending_return_to)?
                }
            }
        }
        Ok(())
    }

    fn gather_goto_input(&self, action: &Action) -> crate::overlay::GotoInputs {
        let headings = if matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenOutline
        ) && self.buffer.is_markdown()
        {
            crate::markdown::headings(&self.buffer.text())
                .into_iter()
                .map(|heading| (heading.label(), heading.line))
                .collect()
        } else {
            Vec::new()
        };
        crate::overlay::GotoInputs {
            corpus: self.corpus.to_vec(),
            open: Vec::new(),
            recent: Vec::new(),
            times: Vec::new(),
            headings,
            line_count: matches!(
                action,
                Action::OpenGoto
                    | Action::OpenProject
                    | Action::OpenRecentProjects
                    | Action::OpenOutline
            )
            .then(|| self.buffer.line_count())
            .unwrap_or_default(),
        }
    }

    fn gather_spell_target(&self, action: &Action) -> Option<crate::overlay::SpellSuggestTarget> {
        if !matches!(action, Action::OpenSpellSuggest) {
            return None;
        }
        self.spell.as_ref().and_then(|checker| {
            let (line, col) = self.buffer.cursor_line_col();
            checker
                .suggest_at(&self.buffer.text(), line, col, self.buffer.syntax_lang())
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
        })
    }

    fn gather_history_entries(&self, action: &Action) -> Vec<crate::history::TimelineRow> {
        if !matches!(action, Action::OpenHistory | Action::CompareVersion) {
            return Vec::new();
        }
        crate::history::source_path(self.buffer.path(), self.buffer.is_unnamed_fresh())
            .map(|path| {
                crate::history::timeline_rows(
                    &path,
                    &self.buffer.text(),
                    crate::history::now_millis(),
                )
            })
            .unwrap_or_default()
    }

    fn gather_assets(&self, action: &Action) -> Vec<crate::assets::Orphan> {
        if !matches!(action, Action::OpenAssetClean) {
            return Vec::new();
        }
        crate::assets::scan(&self.root, &self.corpus)
    }

    /// Replay's checker has no ambient personal dictionary. The empty roster is
    /// intentional: ordinary captures stay hermetic while `--screenshot-app`
    /// reaches the App's loaded dictionary.
    fn gather_user_words(&self, action: &Action) -> Vec<String> {
        if !matches!(action, Action::OpenUserWords) {
            return Vec::new();
        }
        // The replay's OWN checker, and it is ALWAYS EMPTY here: `SpellChecker`
        // is constructed with no personal dictionary and only the live `App`
        // ever fills one (`App::load_user_dictionary` is the sole caller of
        // `set_user_words`). So this arm carries the live gather's SHAPE, not
        // its content — a tier-1 `--keys` capture photographs an empty word
        // list whatever `dictionary.txt` holds, and the picker's rows are
        // reachable only through `--screenshot-app`. That is a decision, not an
        // oversight: a replay reading the ambient word list would photograph
        // whoever ran it. `docs/harness-reach.md` records the ceiling and
        // `capture::tests::personal_dictionary_journey` holds it there.
        self.spell
            .as_ref()
            .map(|checker| checker.user_words_sorted())
            .unwrap_or_default()
    }

    // SEARCH IN FOLDER's headless twin of the live gather above: same budget,
    // same `crate::fs` seam, so a `--keys` capture sees the real corpus.
    fn gather_search_corpus(&self, action: &Action) -> Vec<(String, String)> {
        if !matches!(action, Action::OpenSearchFolder) {
            return Vec::new();
        }
        let root = self.root.clone();
        crate::search_folder::load_corpus(
            &self.corpus,
            &crate::search_folder::SearchBudget::default(),
            |rel| {
                crate::fs::active()
                    .read_to_string(&crate::index::resolve(&root, rel))
                    .ok()
            },
        )
    }

    /// Apply one action through the sole pure transition seam, then append its
    /// effects to the current depth-first worklist. The pending palette
    /// breadcrumb is attributed between the core transition and its effects,
    /// matching the live interpreter's ordering.
    fn apply_action_transition(
        &mut self,
        chord: &crate::keyspec::Chord,
        action: Action,
        shift: bool,
        work: &mut actions::EffectWorklist,
        pending_return_to: &mut Option<crate::overlay::OverlayKind>,
    ) {
        if let Some(oracle) = self.oracle.as_deref_mut() {
            oracle.refresh(self.buffer, self.zoom);
        }
        let effective_keep = self.config.effective_linux_keep();
        let settings_values = || {
            crate::settings::SettingsValues::gather(
                self.config,
                &self.root,
                self.zoom,
                crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
            )
        };
        let picker_kind = crate::overlay::picker_kind_for(
            &action,
            self.journey.parked_kind(),
            self.journey.card(),
        );
        let picker_input = match picker_kind {
            Some(crate::overlay::OverlayKind::Goto) => Some(crate::overlay::PickerInput::Goto(
                self.gather_goto_input(&action),
            )),
            Some(crate::overlay::OverlayKind::Theme) => Some(crate::overlay::PickerInput::Theme),
            Some(crate::overlay::OverlayKind::Caret) => Some(crate::overlay::PickerInput::Caret),
            Some(crate::overlay::OverlayKind::Dictionary) => {
                Some(crate::overlay::PickerInput::Dictionary)
            }
            Some(crate::overlay::OverlayKind::CjkLang | crate::overlay::OverlayKind::Date) => {
                let values = settings_values();
                Some(crate::overlay::PickerInput::Settings(values))
            }
            Some(crate::overlay::OverlayKind::Keymap) => {
                Some(crate::overlay::PickerInput::Keymap {
                    configured: self.config.keymap.clone().unwrap_or_default(),
                })
            }
            Some(crate::overlay::OverlayKind::Command) => Some(
                crate::overlay::PickerInput::Command(crate::overlay::CommandInputs {
                    bindings: crate::overlay::BindingInputs {
                        keys: &self.config.keys,
                        linux_keep: &effective_keep,
                        keymap_flavor: self.config.keymap_flavor(),
                    },
                    settings_values: settings_values(),
                    row_gates: Default::default(),
                }),
            ),
            Some(crate::overlay::OverlayKind::Keybindings) => Some(
                crate::overlay::PickerInput::Keybindings(crate::overlay::BindingInputs {
                    keys: &self.config.keys,
                    linux_keep: &effective_keep,
                    keymap_flavor: self.config.keymap_flavor(),
                }),
            ),
            Some(crate::overlay::OverlayKind::Spell) => Some(crate::overlay::PickerInput::Spell(
                self.gather_spell_target(&action),
            )),
            Some(crate::overlay::OverlayKind::History) => Some(
                crate::overlay::PickerInput::History(crate::overlay::HistoryInputs {
                    entries: self.gather_history_entries(&action),
                    now: None,
                    session_start: None,
                    subject_name: self.buffer.display_name(),
                }),
            ),
            Some(crate::overlay::OverlayKind::Settings) => {
                Some(crate::overlay::PickerInput::Settings(settings_values()))
            }
            Some(crate::overlay::OverlayKind::Assets) => Some(crate::overlay::PickerInput::Assets(
                self.gather_assets(&action),
            )),
            Some(crate::overlay::OverlayKind::UserWords) => Some(
                crate::overlay::PickerInput::UserWords(self.gather_user_words(&action)),
            ),
            Some(crate::overlay::OverlayKind::SearchFolder) => Some(
                crate::overlay::PickerInput::SearchFolder(crate::overlay::SearchFolderInputs {
                    root: self.root.clone(),
                    corpus: self.gather_search_corpus(&action),
                }),
            ),
            Some(crate::overlay::OverlayKind::Credits) => {
                Some(crate::overlay::PickerInput::Credits)
            }
            Some(
                crate::overlay::OverlayKind::Browse
                | crate::overlay::OverlayKind::MoveDest
                | crate::overlay::OverlayKind::ExportDest
                | crate::overlay::OverlayKind::Project
                | crate::overlay::OverlayKind::ProjectBrowse
                | crate::overlay::OverlayKind::Rename
                | crate::overlay::OverlayKind::InsertLink
                | crate::overlay::OverlayKind::KeepName
                | crate::overlay::OverlayKind::Context
                | crate::overlay::OverlayKind::TableDims
                | crate::overlay::OverlayKind::Conflict,
            )
            | None => None,
        };
        let files_builder =
            files_overlay::ReplayFilesBuilder::new(&self.root, &self.workspace, &self.corpus);
        let mut make_overlay = |kind| {
            picker_input
                .as_ref()
                .and_then(|input| files_builder.build(kind, input))
        };
        let mut browse_to = |kind, rel| files_builder.browse(kind, rel);
        let mut ctx = actions::ActionCtx {
            buffer: &mut *self.buffer,
            shift_selecting: &mut self.shift_selecting,
            zoom: &mut self.zoom,
            search: &mut self.search,
            scroll_page_lines: 20,
            journey: &mut self.journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: self.oracle.as_deref().map(|oracle| oracle.as_oracle()),
        };
        let transition = actions::apply_transition(&mut ctx, &action, shift);
        let primary = transition.primary();
        self.record_action_trace(chord, &action, &primary);
        self.journey.attribute_launch(pending_return_to.take());
        work.expand(action, transition);
    }

    fn arm_hover_baseline(&mut self) {
        if let Some(overlay) = self.journey.card_mut() {
            overlay.arm_hover_baseline(self.cursor_px.0, self.cursor_px.1);
        }
    }
}
