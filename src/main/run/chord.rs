//! One replayed chord: resolve, apply, and settle.
//!
//! Resolution owns the search-before-keymap ordering and derives shift intent
//! once. Application owns the shared depth-first effect worklist. Each action
//! inside that worklist still crosses the single [`actions::apply_transition`]
//! seam after gathering the caller-owned overlay inputs it needs.

use super::*;

struct ResolvedChord {
    action: Action,
    shift: bool,
}

use crate::overlay::SpellSuggestTarget;

struct ReplayActionInputs {
    goto_headings: Vec<(String, usize)>,
    goto_line_count: usize,
    spell_target: Option<SpellSuggestTarget>,
    history_entries: Vec<crate::history::TimelineRow>,
    assets: Vec<crate::assets::Orphan>,
    goto_folders: Vec<(String, bool)>,
    goto_recent_folders: Vec<String>,
    settings_values: crate::settings::SettingsValues,
    search_corpus: Vec<(String, String)>,
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

    /// Gather the data whose cost or meaning depends on the action being
    /// applied. This is replay's input half of the shared overlay builder; it
    /// performs no editor transition itself.
    fn gather_action_inputs(&self, action: &Action) -> ReplayActionInputs {
        let goto_headings = if matches!(action, Action::OpenGoto | Action::OpenOutline)
            && self.buffer.is_markdown()
        {
            crate::markdown::headings(&self.buffer.text())
                .into_iter()
                .map(|heading| (heading.label(), heading.line))
                .collect()
        } else {
            Vec::new()
        };
        // Go to Line's numeric companion: ANY buffer, not only markdown --
        // the same summon gate as `goto_headings`, minus the markdown check.
        let goto_line_count = if matches!(action, Action::OpenGoto | Action::OpenOutline) {
            self.buffer.line_count()
        } else {
            0
        };
        let spell_target = if matches!(action, Action::OpenSpellSuggest) {
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
        } else {
            None
        };
        let history_entries = if matches!(action, Action::OpenHistory | Action::CompareVersion) {
            crate::history::source_path(self.buffer.path(), self.buffer.is_unnamed_fresh())
                .map(|path| {
                    crate::history::timeline_rows(
                        &path,
                        &self.buffer.text(),
                        crate::history::now_millis(),
                    )
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let assets = if matches!(action, Action::OpenAssetClean) {
            crate::assets::scan(&self.root, &self.corpus)
        } else {
            Vec::new()
        };
        let (goto_folders, goto_recent_folders) = if matches!(
            action,
            Action::OpenGoto
                | Action::OpenProject
                | Action::OpenRecentProjects
                | Action::OpenOutline
        ) {
            crate::overlay::goto_folder_roster(Some(&self.workspace), &[])
        } else {
            (Vec::new(), Vec::new())
        };
        // SEARCH IN FOLDER's headless twin of the live gather above: same
        // budget, same `crate::fs` seam, so a `--keys` capture sees the real
        // corpus a live summon would.
        let search_corpus = if matches!(action, Action::OpenSearchFolder) {
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
        } else {
            Vec::new()
        };
        ReplayActionInputs {
            goto_headings,
            goto_line_count,
            spell_target,
            history_entries,
            assets,
            goto_folders,
            goto_recent_folders,
            settings_values: crate::settings::SettingsValues::gather(
                self.config,
                &self.root,
                self.zoom,
                crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
            ),
            search_corpus,
        }
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
        let inputs = self.gather_action_inputs(&action);
        let effective_keep = self.config.effective_linux_keep();
        let build_ctx = crate::overlay::BuildCtx {
            goto_corpus: self.corpus.to_vec(),
            goto_open: Vec::new(),
            goto_recent: Vec::new(),
            goto_times: Vec::new(),
            config_keys: &self.config.keys,
            config_linux_keep: &effective_keep,
            config_keymap_flavor: self.config.keymap_flavor(),
            goto_headings: inputs.goto_headings,
            goto_line_count: inputs.goto_line_count,
            goto_folders: inputs.goto_folders,
            goto_recent_folders: inputs.goto_recent_folders,
            spell_target: inputs.spell_target,
            history_entries: inputs.history_entries,
            history_now: None,
            history_session_start: None,
            settings_values: inputs.settings_values,
            assets: inputs.assets,
            // Headless replay is daemon-free, so Finish file stays hidden.
            row_gates: Default::default(),
            search_root: self.root.clone(),
            search_corpus: inputs.search_corpus,
        };
        let mut make_overlay =
            |kind: crate::overlay::OverlayKind| crate::overlay::build(kind, &build_ctx);
        let (root, workspace) = (self.root.as_path(), Some(self.workspace.as_path()));
        let mut browse_to = |kind: crate::overlay::OverlayKind, rel: Option<String>| {
            // Recent projects are persisted live-only state; replay supplies an
            // empty roster so captures remain deterministic.
            crate::overlay::browse_level(kind, rel, root, workspace, &[])
        };
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
