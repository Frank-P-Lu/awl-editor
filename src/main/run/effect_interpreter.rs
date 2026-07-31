use super::*;

impl<'a> ReplaySession<'a> {
    fn record_effect_class(
        &mut self,
        action: &Action,
        chord: &crate::keyspec::Chord,
        effect: &actions::Effect,
    ) -> Result<bool> {
        let classified = crate::replay::classify_for(effect, self.filesystem);
        if !matches!(classified.class, crate::replay::EffectClass::Applied) {
            *self.records.last_mut().expect("this chord has a trace") =
                replay_effects::chord_trace(&chord.spec, action, &classified);
        }
        if let crate::replay::EffectClass::Intercepted { detail } = &classified.class {
            self.intercepts.push(crate::replay::Intercept {
                effect: classified.name,
                detail: detail.clone(),
            });
        }
        if self.mode == crate::replay::Mode::Strict
            && let crate::replay::EffectClass::Unsupported { .. } = classified.class
        {
            return Err(crate::replay::strict_error(action, &classified));
        }
        if self.mode == crate::replay::Mode::Permissive
            && let Some(skip) = crate::replay::permissive_skip(action, &classified)
        {
            self.replay_skips.push(skip);
        }
        if self.mode == crate::replay::Mode::Permissive
            && let Some(warning) = crate::replay::warn_line(action, &classified)
        {
            eprintln!("{warning}");
            self.warnings.push(warning);
        }
        Ok(self.interpret_headless_effect(effect))
    }

    pub(super) fn interpret_effect(
        &mut self,
        action: &Action,
        chord: &crate::keyspec::Chord,
        effect: actions::Effect,
        work: &mut actions::EffectWorklist,
        pending_return_to: &mut Option<crate::overlay::OverlayKind>,
    ) -> Result<()> {
        if self.record_effect_class(action, chord, &effect)? {
            return Ok(());
        }
        match effect {
            actions::Effect::Clipboard(actions::ClipboardEffect::PasteImage) => {
                work.descend(Action::YankText);
            }
            actions::Effect::Buffer(_)
            | actions::Effect::Persistence(_)
            | actions::Effect::Clipboard(_)
            | actions::Effect::Daemon(_)
            | actions::Effect::Surface(_)
            | actions::Effect::Notice(_)
            | actions::Effect::Render(_)
            | actions::Effect::SettingToggle { .. }
            | actions::Effect::SettingValueCommit { .. }
            | actions::Effect::SettingPathPick { .. } => {
                unreachable!("typed effects are owned by interpret_headless_effect")
            }
            actions::Effect::InsertDate => {
                let (y, m, d) = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
                let text = crate::dateformat::active_format().format(y, m, d);
                self.buffer.insert_text(&text);
            }
            actions::Effect::OverlayAccept(kind, value) => {
                if kind == crate::overlay::OverlayKind::Goto {
                    let path = crate::index::resolve(&self.root, &value);
                    let new_key = crate::buffers::BufferKey::path(&path);
                    if crate::buffers::BufferKey::of(self.buffer).as_ref() != Some(&new_key) {
                        park_active(self.buffer, &mut self.registry);
                        *self.buffer = match self.registry.take(&new_key) {
                            Some(entry) => entry.buffer,
                            None => Buffer::from_file(&path),
                        };
                        crate::page::set_measure(self.config.measure_for(self.buffer.page_class()));
                    }
                }
                // SWITCH-PROJECT (queue item 189): re-scope root/workspace/corpus
                // to the ACCEPTED root BEFORE recording the accept, so every chord
                // the caller applies afterward (Cmd-O, Browse, the asset scan) reads
                // the new tree — the sidecar's own re-derivation (`run::project_info`,
                // item 183) and this session's internal state can no longer disagree.
                if kind == crate::overlay::OverlayKind::Project {
                    self.resync_project_location(std::path::PathBuf::from(&value));
                }
                self.accept = Some((kind, value));
            }
            actions::Effect::JumpToLine(line) => {
                let idx = self.buffer.line_col_to_char(line, 0);
                self.buffer.set_cursor(idx);
                self.buffer.reveal_placement();
            }
            actions::Effect::RunAction(action) => {
                *pending_return_to = Some(crate::overlay::OverlayKind::Command);
                work.descend(action);
            }
            actions::Effect::RebindCommit { slug, binding, .. } => {
                if let Some(overlay) = self.journey.card_mut() {
                    overlay.notice = format!("bound {slug} -> {binding}");
                    overlay.capture_abort();
                }
            }
            actions::Effect::RebindReset { slug } => {
                if let Some(overlay) = self.journey.card_mut()
                    && overlay.notice.is_empty()
                {
                    overlay.notice = format!("reset {slug}");
                }
            }
            actions::Effect::Quit
            | actions::Effect::Recoil(_)
            | actions::Effect::TypeImpact
            | actions::Effect::DeleteSquash
            | actions::Effect::Gulp
            | actions::Effect::LineLand
            | actions::Effect::CopyPulse
            | actions::Effect::SettingRangeStep { .. }
            | actions::Effect::KeepVersion { .. }
            | actions::Effect::FollowLink(_)
            | actions::Effect::ReportProblem
            | actions::Effect::DownloadFile
            | actions::Effect::Export(_)
            | actions::Effect::CheckForUpdates
            | actions::Effect::TrashAsset { .. }
            | actions::Effect::RenameNoteCommit { .. }
            | actions::Effect::DuplicateNote
            | actions::Effect::AddToDictionary(_)
            | actions::Effect::None => {}
        }
        Ok(())
    }
}
