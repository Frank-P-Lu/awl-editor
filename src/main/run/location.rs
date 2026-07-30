//! WHERE AM I WORKING — the launch root, the workspace scope, and the capture
//! sidecar's project block. One owner each, lifted out of `run.rs` whole.
//!
//! The live `App` derives the same three facts in `App::resync_project_location`
//! and shares [`resolve_workspace`] with this file. Keeping the capture's own
//! derivation in ONE place here is what stops the two from drifting again — see
//! [`project_info`], `docs/harness-reach.md`, and queue items 180/183.

use std::path::PathBuf;

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

/// THE ONE launch-precedence law (item 76), for the WINDOWED launch door
/// only (`Mode::Windowed` in [`run`]):
///
/// 1. **EXPLICIT TARGET WINS** — `--root`, or a file/dir argument (`awl .`
///    included) — delegates to [`resolve_root`], unaffected by anything
///    remembered.
/// 2. **ARGUMENT-FREE LAUNCH RESTORES** — bare `awl`: the remembered active
///    folder (`remembered`, from `crate::session::remembered_root`, gated by
///    the caller on `Config::session_restore_on()`) wins if there is one.
/// 3. **FIRST RUN** — bare launch, nothing remembered (a fresh install, or
///    the session kill-switch is off): `default_folder` (the resolved
///    `--default-folder`/config/`~/notes` value).
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
) -> PathBuf {
    if root.is_some() || file.is_some() {
        return resolve_root(root, file);
    }
    match remembered {
        Some(p) => p.to_path_buf(),
        None => default_folder.to_path_buf(),
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
/// item 180's defect exactly, in the harness's own copy of the rule, still live
/// after item 180 fixed the App. A capture of a Switch-project therefore
/// reported a workspace the running editor no longer had — an oracle that lies
/// about the very transition it is asked to witness (queue item 183).
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
