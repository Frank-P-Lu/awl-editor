use super::*;

pub(super) fn chord_trace(
    chord: &str,
    action: &Action,
    classified: &crate::replay::Classified,
) -> crate::storyboard::ChordTrace {
    crate::storyboard::ChordTrace {
        chord: chord.to_string(),
        action: Some(format!("{action:?}")),
        effect: classified.name.to_string(),
        class: match &classified.class {
            crate::replay::EffectClass::Applied => "applied",
            crate::replay::EffectClass::Intercepted { .. } => "intercepted",
            crate::replay::EffectClass::Unsupported { .. } => "unsupported",
        },
        detail: match &classified.class {
            crate::replay::EffectClass::Intercepted { detail } => detail.clone(),
            crate::replay::EffectClass::Applied
            | crate::replay::EffectClass::Unsupported { .. } => String::new(),
        },
    }
}

impl ReplayPolicy {
    pub(crate) fn isolated() -> Self {
        Self {
            mode: crate::replay::Mode::Strict,
            filesystem: crate::replay::FilesystemCapability::Isolated,
        }
    }
}

/// Strict capture is the sole screenshot door allowed to mint isolated
/// filesystem authority. Ordinary capture calls `replay_keys`, whose
/// signature has no capability parameter and always supplies `None`.
#[allow(clippy::too_many_arguments)]
pub(super) fn replay_keys_strict(
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
    km: &mut crate::keymap::KeymapState,
) -> Result<ReplayResult> {
    let policy = ReplayPolicy::isolated();
    replay_keys_mode(
        policy.mode,
        policy.filesystem,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
        km,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn capture_replay(
    strict: bool,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
    km: &mut crate::keymap::KeymapState,
) -> Result<ReplayResult> {
    if strict {
        replay_keys_strict(buffer, keys, corpus, root, workspace, config, oracle, km)
    } else {
        Ok(replay_keys(
            buffer, keys, corpus, root, workspace, config, oracle, km,
        ))
    }
}

impl<'a> ReplaySession<'a> {
    fn interpret_persistence(&mut self, persistence: &actions::PersistenceEffect) {
        match persistence {
            actions::PersistenceEffect::Save(actions::SaveKind::Manual) => {
                if self.filesystem == crate::replay::FilesystemCapability::Isolated {
                    // The caller handed this replay an isolated, test-owned
                    // filesystem. Manual save performs the live scratch-
                    // promotion/plain-save split only through that sandbox.
                    let _ = self.buffer.save_into_folder(self.root);
                }
            }
            actions::PersistenceEffect::Save(actions::SaveKind::Finish) => {
                if self.filesystem == crate::replay::FilesystemCapability::Isolated {
                    let _ = self.buffer.save();
                }
            }
            actions::PersistenceEffect::Preference(preference) => match preference {
                actions::PreferenceEffect::CaretMode
                | actions::PreferenceEffect::PageMode
                | actions::PreferenceEffect::PageWidth
                | actions::PreferenceEffect::PageReset
                | actions::PreferenceEffect::Outline
                | actions::PreferenceEffect::MenuBar
                | actions::PreferenceEffect::Typewriter
                | actions::PreferenceEffect::Spellcheck
                | actions::PreferenceEffect::WritingNits
                | actions::PreferenceEffect::WritingStreaks => {}
            },
        }
    }

    fn interpret_buffer(&mut self, buffer: &actions::BufferEffect) {
        match buffer {
            actions::BufferEffect::Previous { .. } => {}
            actions::BufferEffect::NewDocument => {
                park_active(self.buffer, &mut self.registry);
                self.buffer.start_fresh_doc(self.root.to_path_buf());
            }
            actions::BufferEffect::OpenSettings => {
                // Existing, explicitly configured files may be read. Absence
                // stays absence: replay never materializes a default config.
                if !self.config.path.as_os_str().is_empty()
                    && crate::fs::active().exists(&self.config.path)
                {
                    *self.buffer = Buffer::from_file(&self.config.path);
                }
            }
            actions::BufferEffect::OpenCredits => {
                *self.buffer = Buffer::from_str(crate::credits::CREDITS_MD);
            }
            actions::BufferEffect::OpenGuide => {
                *self.buffer = Buffer::from_str(&crate::guide::render(
                    crate::convention::Convention::current(),
                    crate::commands::Platform::current(),
                ));
            }
        }
    }

    /// Interpret the closed typed-effect families for headless replay. Returning
    /// `true` means this owner handled the effect and the legacy applied-effect
    /// match must not see it. Nested matches deliberately have no wildcard:
    /// extending any typed vocabulary fails compilation here and in the live
    /// interpreter until both owners make a conscious routing decision.
    pub(super) fn interpret_headless_effect(&mut self, effect: &actions::Effect) -> bool {
        match effect {
            actions::Effect::Persistence(persistence) => {
                self.interpret_persistence(persistence);
                true
            }
            actions::Effect::Clipboard(clipboard) => {
                match clipboard {
                    actions::ClipboardEffect::WriteKillRing => {}
                    actions::ClipboardEffect::PasteImage => return false,
                }
                true
            }
            actions::Effect::Buffer(buffer) => {
                self.interpret_buffer(buffer);
                true
            }
            actions::Effect::Daemon(daemon) => {
                match daemon {
                    actions::DaemonEffect::NotifyFinished => {}
                }
                true
            }
            actions::Effect::Surface(actions::SurfaceEffect::ShowAbout) => {
                crate::about::set_open(true);
                true
            }
            actions::Effect::Notice(notice) => {
                match notice {
                    actions::NoticeEffect::Toast(_)
                    | actions::NoticeEffect::Sticky(_)
                    | actions::NoticeEffect::Clear => {}
                }
                true
            }
            actions::Effect::Render(render) => {
                match render {
                    actions::RenderEffect::SyncView { .. }
                    | actions::RenderEffect::Reshape
                    | actions::RenderEffect::ZoomChanged
                    | actions::RenderEffect::Redraw
                    | actions::RenderEffect::EditStreak => {}
                }
                true
            }
            actions::Effect::None
            | actions::Effect::Quit
            | actions::Effect::RunAction(_)
            | actions::Effect::OverlayAccept(_, _)
            | actions::Effect::JumpToLine(_)
            | actions::Effect::AddToDictionary(_)
            | actions::Effect::RebindCommit { .. }
            | actions::Effect::RebindReset { .. }
            | actions::Effect::Recoil(_)
            | actions::Effect::TypeImpact
            | actions::Effect::DeleteSquash
            | actions::Effect::Gulp
            | actions::Effect::LineLand
            | actions::Effect::KeepVersion { .. }
            | actions::Effect::FollowLink(_)
            | actions::Effect::ReportProblem
            | actions::Effect::DownloadFile
            | actions::Effect::Export(_)
            | actions::Effect::CheckForUpdates
            | actions::Effect::CopyPulse
            | actions::Effect::SettingToggle { .. }
            | actions::Effect::SettingValueCommit { .. }
            | actions::Effect::SettingPathPick { .. }
            | actions::Effect::SettingRangeStep { .. }
            | actions::Effect::TrashAsset { .. }
            | actions::Effect::RenameNoteCommit { .. }
            | actions::Effect::DuplicateNote
            | actions::Effect::InsertDate => false,
        }
    }
}
