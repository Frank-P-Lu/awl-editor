use super::*;

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
                    let _ = self.buffer.save_into_folder(&self.root);
                }
            }
            actions::PersistenceEffect::Save(actions::SaveKind::Finish) => {
                if self.filesystem == crate::replay::FilesystemCapability::Isolated {
                    let _ = self.buffer.save();
                }
            }
            // Live-App-only: the conflict this settles is latched on the App's
            // own per-buffer baseline, which a replay never builds. Classified
            // Unsupported, so a strict replay aborts naming it rather than
            // pretending to resolve something it cannot see.
            actions::PersistenceEffect::ResolveExternalChange(_)
            | actions::PersistenceEffect::ReviewExternalChange => {}
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
            // Replay owns one buffer and no working set, so neither switching
            // away nor removing an entry has anything to act on. Both are
            // classified Unsupported rather than silently doing nothing.
            actions::BufferEffect::Previous | actions::BufferEffect::CloseActive => {}
            actions::BufferEffect::NewDocument => {
                self.start_fresh_document();
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
            actions::BufferEffect::OpenGuide => {
                *self.buffer = Buffer::from_str(&crate::guide::render(
                    crate::convention::Convention::current(),
                    crate::commands::Platform::current(),
                ));
            }
            actions::BufferEffect::OpenReference => {
                *self.buffer = Buffer::from_str(crate::reference_doc::REFERENCE_MD);
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
            actions::Effect::Surface(surface) => match surface {
                actions::SurfaceEffect::ShowAbout => {
                    crate::about::set_open(true);
                    true
                }
                actions::SurfaceEffect::OpenFileChooser
                | actions::SurfaceEffect::OpenFolderChooser => false,
            },
            actions::Effect::SettingToggle { key } => {
                self.interpret_setting_toggle(key);
                true
            }
            actions::Effect::SettingValueCommit { key, value } => {
                self.interpret_setting_value_commit(key, value);
                true
            }
            actions::Effect::SettingPathPick { key, path } => {
                self.interpret_setting_path_pick(key, path);
                true
            }
            // THE CALM NOTICE, latched rather than swallowed. This arm used to
            // discard every notice while still reporting the effect as APPLIED,
            // so a headless capture of an action whose only user-visible result
            // IS its notice — a refused Export, a failed rename — photographed
            // nothing and recorded no skip either. A replay has no clock, so a
            // Toast latched here never expires; that matches a GPU-less live
            // `App`, whose `set_toast_notice` also arms no deadline.
            actions::Effect::Notice(notice) => {
                // `Clear` is the one arm carrying neither a message nor a kind;
                // both accessors answer over the same no-wildcard match, so a
                // new arm is a compile error in `actions::effects` rather than a
                // silently dropped notice here.
                self.notice = match (notice.message(), notice.kind()) {
                    (Some(text), Some(kind)) => Some((text.to_string(), kind)),
                    _ => None,
                };
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
            | actions::Effect::Export(_, _)
            | actions::Effect::CheckForUpdates
            | actions::Effect::CopyPulse
            | actions::Effect::SettingRangeStep { .. }
            | actions::Effect::TrashAsset { .. }
            | actions::Effect::RenameNoteCommit { .. }
            | actions::Effect::DuplicateNote
            | actions::Effect::InsertDate => false,
        }
    }
}
