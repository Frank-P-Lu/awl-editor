//! WHERE AM I WORKING — the launch root, the workspace scope, the capture
//! sidecar's project block, and `ReplaySession`'s own
//! re-scope on a Switch-project accept. One owner each, lifted out of
//! `run.rs` whole — this module IS the location-derivation owner, so a
//! derivation belongs here even when the struct it mutates is declared in
//! the parent file.
//!
//! The live `App` derives the same three facts in `App::resync_project_location`
//! and shares [`resolve_workspace`] with this file. Keeping the capture's own
//! derivation in ONE place here is what stops the two from drifting again — see
//! [`project_info`], `docs/harness-reach.md`, and queue items 180/183/189.

use std::path::PathBuf;

use super::ReplaySession;
use crate::app;
use crate::capture;
use crate::config::Config;

pub(crate) fn resolve_root(root: &Option<PathBuf>, file: &Option<PathBuf>) -> PathBuf {
    if let Some(r) = root {
        return r.clone();
    }
    if let Some(f) = file {
        if crate::fs::active().is_dir(f) {
            return f.clone();
        }
        if let Some(p) = f.parent()
            && !p.as_os_str().is_empty()
        {
            return p.to_path_buf();
        }
    }
    crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// THE ONE launch-precedence law for the WINDOWED launch door
/// only (`Mode::Windowed` in [`run`]):
///
/// 1. **EXPLICIT TARGET WINS** — `--root`, or a file/dir argument (`awl .`
///    included) — delegates to [`resolve_root`], unaffected by anything
///    remembered.
/// 2. **ARGUMENT-FREE LAUNCH RESTORES** — bare `awl`: the remembered active
///    folder (`remembered`, from `crate::session::remembered_root`, gated by
///    the caller on `Config::session_restore_on()`) wins if there is one.
/// 3. **FIRST RUN** — bare launch, nothing remembered: an explicitly
///    configured default folder is authoritative. Without one, the live App
///    starts from its own recoverable data area; `firstrun` supplies Welcome
///    through the scratch stash and never creates `~/notes`.
///
/// The DOCUMENT half of a bare launch (which file becomes active, the rest
/// parked behind it) is owned by `App::apply_session_restore`, reading the
/// SAME underlying session state — see `app/session.rs`'s module doc for why
/// the two halves can never disagree.
pub(crate) fn resolve_launch_context(
    root: &Option<PathBuf>,
    file: &Option<PathBuf>,
    remembered: Option<&std::path::Path>,
    default_folder: &std::path::Path,
    default_folder_configured: bool,
) -> PathBuf {
    if root.is_some() || file.is_some() {
        return resolve_root(root, file);
    }
    match remembered {
        Some(p) => p.to_path_buf(),
        None if default_folder_configured => default_folder.to_path_buf(),
        None => crate::fs::data_root(),
    }
}

/// THE ONE DERIVATION of a capture's [`capture::ProjectInfo`] from a root — the
/// headless twin of `App::resync_project_location`, and for the same reason.
///
/// A capture reports the project location TWICE: once from the launch root, and
/// again if the replay accepts a Project-picker row (`Effect::OverlayAccept(
/// Project, ..)` — the same effect that reaches `App::switch_project` live).
/// Those were two hand-rolled sites, and the second re-derived `name`/`branch`/
/// `dirty` from the new root while carrying the OLD root's `workspace` forward:
/// a stale workspace defect exactly, in the harness's own copy of the rule. A
/// capture of a Switch-project therefore
/// reported a workspace the running editor no longer had — an oracle that lies
/// about the very transition it is asked to witness.
///
/// One builder, both sites. `workspace` is the ALREADY-FOLDED flag-over-config
/// value (`main/args.rs`), matching what `App` folds for itself; the fallback to
/// `root.parent()` stays in the shared [`resolve_workspace`] both paths call.
pub(crate) fn project_info(
    root: &std::path::Path,
    workspace: &Option<PathBuf>,
    default_folder: Option<&std::path::Path>,
    config: &Config,
) -> capture::ProjectInfo {
    let proj = crate::project::Project::resolve(root);
    capture::ProjectInfo {
        root: root.to_path_buf(),
        name: proj.name,
        branch: proj.branch,
        dirty: proj.dirty,
        default_folder: default_folder.map(|p| p.to_path_buf()),
        workspace: Some(resolve_workspace(workspace, root)),
        keymap_flavor: config.keymap_flavor().config_name(),
    }
}

pub(crate) fn resolve_workspace(workspace: &Option<PathBuf>, root: &std::path::Path) -> PathBuf {
    if let Some(w) = workspace {
        return w.clone();
    }
    match root.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => root.to_path_buf(),
    }
}

impl<'a> ReplaySession<'a> {
    /// The capture sidecar's project block for this session's CURRENT location.
    /// A storyboard asks for this afresh at every rendered step, so a
    /// Switch-project accept cannot leave later frames carrying the launch
    /// root's identity. The free [`project_info`] builder remains the one owner
    /// of the derivation; this method only supplies the session-private inputs
    /// that were re-scoped together by [`Self::resync_project_location`].
    pub(crate) fn current_project_info(
        &self,
        default_folder: Option<&std::path::Path>,
    ) -> capture::ProjectInfo {
        project_info(
            &self.root,
            &self.workspace_flag,
            default_folder,
            self.config,
        )
    }

    /// THE ONE RE-SCOPING OWNER — re-derive `root`,
    /// `workspace`, and the file `corpus` for a NEW project root, the
    /// session's mirror of the live `App::resync_project_location`
    /// (`app/files/open.rs`) and for the identical reason: before this fn
    /// existed, a Switch-project accept re-derived the SIDECAR's project
    /// block through [`project_info`] but left these three
    /// fields fixed at their launch values, so a chord applied after the
    /// accept — a Cmd-O opening Goto against `corpus`, a Browse summon
    /// against `root`/`workspace` — silently kept testing the OLD tree.
    ///
    /// Called ONLY from the `OverlayAccept(Project, ..)` arm in
    /// `effect_interpreter.rs`, immediately before `self.accept` is set —
    /// `pub(super)` rather than `pub`, so no consumer outside `run` can read
    /// a stale copy of any of the three by reaching around it. `workspace_flag`
    /// re-runs the SAME [`resolve_workspace`] the constructor used, against
    /// the NEW root: an explicit `--workspace` stays pinned across the
    /// switch; an unset one re-derives the new root's parent, covering both
    /// the same-parent coincidence and the no-parent (filesystem-root) edge
    /// stale-workspace defect, rather than carrying the OLD resolved value forward.
    pub(super) fn resync_project_location(&mut self, new_root: PathBuf) {
        self.corpus = crate::index::build_index(&new_root);
        self.workspace = resolve_workspace(&self.workspace_flag, &new_root);
        self.root = new_root;
    }
}

/// THE WINDOWED LAUNCH DOOR — the whole body of `run`'s `Mode::Windowed` arm,
/// which is this module's subject twice over: it resolves WHERE the launch
/// works ([`resolve_launch_context`], the folder half) and WHICH document it
/// opens (`crate::firstrun`, the document half), then hands both to
/// `app::run`. Lifted out of `run.rs` whole; nothing about the sequence
/// changed in the move.
#[allow(clippy::too_many_arguments)]
pub(crate) fn launch_windowed(
    file: Option<PathBuf>,
    root: Option<PathBuf>,
    workspace: Option<PathBuf>,
    default_folder: Option<PathBuf>,
    config: Config,
    wait: bool,
    live: Option<crate::probe::LiveScript>,
) -> anyhow::Result<()> {
    // THE ONE LAUNCH-PRECEDENCE LAW: explicit --root/file wins;
    // else a bare launch restores the remembered active folder (the
    // session's one owner, native + kill-switch gated); else (first run,
    // or the switch is off) the configured default folder.
    #[cfg(not(target_arch = "wasm32"))]
    let remembered = if config.session_restore_on() {
        crate::session::remembered_root()
    } else {
        None
    };
    #[cfg(target_arch = "wasm32")]
    let remembered: Option<PathBuf> = None; // session is native-only
    let default_folder_resolved = crate::args::resolve_default_folder(
        &default_folder
            .clone()
            .or_else(|| config.default_folder.clone()),
    );
    let default_folder_configured = default_folder.is_some() || config.default_folder.is_some();
    let active_root = resolve_launch_context(
        &root,
        &file,
        remembered.as_deref(),
        &default_folder_resolved,
        default_folder_configured,
    );
    // THE DOCUMENT HALF OF THE SAME LAW: a launch that took
    // branch 3 above — nothing asked for, nothing remembered, never
    // welcomed — opens one real Markdown file in that folder instead of
    // an empty scratch buffer. `crate::firstrun` seeds it write-if-
    // absent and marks the profile; from here down it is an ordinary
    // file argument, which is the whole point (see that module's
    // header: there is no welcome state for a later session to leak).
    #[cfg(not(target_arch = "wasm32"))]
    let file = crate::firstrun::resolve_first_run_document(
        file,
        &root,
        remembered.as_deref(),
        &active_root,
        default_folder_configured,
        crate::convention::Convention::current(),
        crate::commands::Platform::current(),
    );
    // Pass the RAW flags + config; `App::new` folds them (flag > config >
    // default) and re-folds on a live config reload. `wait` (native-only,
    // the single-instance daemon's `--wait`) rides straight through, as
    // does `live` (the `--live-script` probe — see `crate::probe`).
    #[cfg(not(target_arch = "wasm32"))]
    {
        app::run(
            file,
            active_root,
            workspace,
            default_folder,
            config,
            wait,
            None,
            live,
        )
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = live; // native-live-only; parsed as None on wasm
        app::run(file, active_root, workspace, default_folder, config, wait)
    }
}
