use super::*;

impl<'a> ReplaySession<'a> {
    pub(super) fn interpret_effect(
        &mut self,
        action: &Action,
        chord: &crate::keyspec::Chord,
        effect: actions::Effect,
        work: &mut actions::EffectWorklist,
        pending_return_to: &mut Option<crate::overlay::OverlayKind>,
    ) -> Result<()> {
        self.classify_effect(action, chord, &effect)?;
        if self.interpret_headless_effect(&effect) {
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
            | actions::Effect::Notice(_)
            | actions::Effect::Render(_)
            | actions::Effect::SettingToggle { .. }
            | actions::Effect::SettingValueCommit { .. }
            | actions::Effect::SettingPathPick { .. } => {
                unreachable!("typed effects are owned by interpret_headless_effect")
            }
            // An unsupported platform chooser is already recorded above. It
            // has no headless continuation; permissive replay calmly consumes
            // it after preserving the typed skip in the sidecar.
            actions::Effect::Surface(_) => {}
            actions::Effect::InsertDate => {
                let (y, m, d) = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
                let text = crate::dateformat::active_format().format(y, m, d);
                self.buffer.insert_text(&text);
            }
            actions::Effect::OverlayAccept(kind, value) => {
                if kind == crate::overlay::OverlayKind::Goto {
                    self.switch_to_goto_target(&value);
                }
                // SWITCH-PROJECT: re-scope root/workspace/corpus
                // to the ACCEPTED root BEFORE recording the accept, so every chord
                // the caller applies afterward (Cmd-O, Browse, the asset scan) reads
                // the new tree — the sidecar's own re-derivation (`run::project_info`,
                // the same owner) and this session's internal state can no longer disagree.
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
            | actions::Effect::Export(_, _)
            | actions::Effect::CheckForUpdates
            | actions::Effect::TrashAsset { .. }
            | actions::Effect::RenameNoteCommit { .. }
            | actions::Effect::DuplicateNote
            | actions::Effect::SaveCopy
            | actions::Effect::RevealInFileManager(_)
            | actions::Effect::AddToDictionary(_)
            | actions::Effect::None => {}
        }
        Ok(())
    }
}
