//! The first-run document: the document half
//! of a launch with nothing remembered.
//!
//! `main/run/location.rs::resolve_launch_context` owns the FOLDER half of the
//! same decision — branch 3 of its precedence law is "bare launch, nothing
//! remembered: open the configured `default_folder`". This module owns what is
//! IN that folder on the very first launch: one ordinary Markdown file,
//! `welcome.md`, seeded from `samples/welcome.md` and opened as the active
//! buffer. `PHILOSOPHY.md` §1 states the product rule it implements — *"First
//! launch opens one real Markdown file that is both welcome and tutorial. The
//! user learns awl by reading and editing inside the actual editor, not by
//! completing a modal tour."*
//!
//! # Why there is no welcome MODE, and no welcome state
//!
//! The seeded document is a file, and nothing else. It is not a buffer flavour,
//! not an overlay, not a workspace, and it carries no marker the editor can
//! read afterwards. Everything downstream — autosave, history, session restore,
//! rename, delete — treats it exactly like a file the user opened from disk,
//! because that is all it is. That is what makes "no special state leaks into
//! later sessions" a STRUCTURAL claim rather than a promise: after
//! [`resolve_first_run_document`] returns, there is no first-run concept left
//! anywhere in the process.
//!
//! # The two facts this module does hold
//!
//! 1. **Write-if-absent** ([`seed`]) — a `welcome.md` already in the folder is
//!    never overwritten, byte for byte, mirroring the web build's own
//!    [`crate::fs::seed_write_if_absent`] law. A returning user who edited it
//!    keeps every character.
//! 2. **A one-shot marker** ([`mark`]/[`marked`]) — `<data_root>/welcomed`.
//!    Without it, a profile whose session kill-switch is off
//!    (`session_restore = false`) would re-seed on every single launch, since
//!    "nothing remembered" is then permanently true. The marker is a plain
//!    file with a plain sentence in it: deleting it is the supported way to
//!    ask for the welcome again.
//!
//! # Capture gate
//!
//! This module is reachable from exactly ONE place — `run::launch_windowed`
//! (`main/run/location.rs`), the windowed-launch door, beside the FOLDER half of
//! the same decision — and `firstrun::tests::
//! the_first_run_door_has_exactly_one_production_call_site` holds that. Every
//! headless capture mode resolves its own root through the explicit-only
//! `run::resolve_root` and never comes near this file: a capture invocation is not a
//! "terminal/desktop launch" in the product sense, and must stay byte-identical
//! whether or not the developer running it has ever launched awl.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use crate::commands::Platform;
use crate::convention::Convention;

/// The seeded document's leaf name, in the active folder. An ordinary name a
/// user can rename or delete without awl noticing.
pub(crate) const WELCOME_FILE: &str = "welcome.md";

/// The one-shot marker's leaf name under [`crate::fs::data_root`].
const MARKER_FILE: &str = "welcomed";

/// The marker's own contents — a sentence, not a magic byte, so a person who
/// finds it in their data directory learns what it is and how to undo it.
const MARKER_TEXT: &str = "awl has opened its welcome document once.\n\
                           Delete this file to be shown it again on the next launch with \
                           nothing to resume.\n";

/// Where the marker lives: beside the session file and the scratch stash, in
/// awl's data root — never in `config.toml`, which is the user's own file.
pub(crate) fn marker_path() -> PathBuf {
    crate::fs::data_root().join(MARKER_FILE)
}

/// Has this profile already been shown the welcome document?
pub(crate) fn marked() -> bool {
    crate::fs::active().exists(&marker_path())
}

/// Record that it has. Best-effort: a data root that cannot be written leaves
/// the profile un-marked, which costs a re-seed attempt next launch and never
/// costs a byte of the user's text (see [`seed`]'s write-if-absent law).
pub(crate) fn mark() {
    let path = marker_path();
    if let Some(parent) = path.parent() {
        let _ = crate::fs::active().create_dir_all(parent);
    }
    let _ = crate::fs::write_atomic(&path, MARKER_TEXT.as_bytes());
}

/// PURE. Is this launch a first run in the product sense?
///
/// True only when the user asked for nothing in particular and awl has nothing
/// to give them back: no explicit `--root`, no file or directory argument, no
/// remembered session folder, and no marker from a previous launch. Mirrors
/// branch 3 of `run::resolve_launch_context`'s precedence law exactly — the two
/// are the folder half and the document half of one decision, so they take the
/// same branch on the same inputs.
pub(crate) fn is_first_run(
    root: &Option<PathBuf>,
    file: &Option<PathBuf>,
    remembered: Option<&Path>,
    marked: bool,
) -> bool {
    root.is_none() && file.is_none() && remembered.is_none() && !marked
}

/// WRITE-IF-ABSENT. Materialise `samples/welcome.md` at `path`, with its
/// `{{key:}}`/`{{cmd:}}` tokens resolved for `convention`+`platform` through
/// [`crate::keytoken::render_key_tokens`] — the same seam, and the same reason,
/// as the web build's seeding: a chord glyph baked into the file at authoring
/// time would be a lie on the other convention, so the bytes are rendered at
/// the moment they are written for the machine that will read them.
///
/// A file already at `path` is left completely untouched and reported as
/// `Ok(())`: the caller opens it either way, so a user who edited their
/// welcome, or who put their own `welcome.md` in that folder first, opens
/// THEIRS.
pub(crate) fn seed(path: &Path, convention: Convention, platform: Platform) -> std::io::Result<()> {
    let fs = crate::fs::active();
    if fs.exists(path) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs.create_dir_all(parent)?;
    }
    let text = crate::keytoken::render_key_tokens(WELCOME_MD, convention, platform);
    // Through the durable-write owner, like every buffer save and the scratch
    // stash: this document is the user's manuscript from the moment it lands.
    crate::fs::write_atomic(path, text.as_bytes())
}

/// `samples/welcome.md`'s bytes. The `include_str!` itself lives in the one
/// embed owner, [`crate::embedded_docs`].
const WELCOME_MD: &str = crate::embedded_docs::WELCOME_MD;

/// THE DOOR. Given the launch as the windowed arm parsed it, return the file
/// that launch should open — `file` unchanged for every launch that is not a
/// first run, or the freshly seeded welcome document for the one that is.
///
/// `active_root` is `resolve_launch_context`'s own answer, which on a first run
/// IS the resolved `default_folder`; taking it as a parameter rather than
/// re-deriving it is what stops the document from ever landing somewhere the
/// folder half did not choose.
///
/// Failing to seed is not an error worth stopping a launch for: awl falls back
/// to the ordinary scratch buffer, the profile stays un-marked, and the next
/// launch tries again.
pub(crate) fn resolve_first_run_document(
    file: Option<PathBuf>,
    root: &Option<PathBuf>,
    remembered: Option<&Path>,
    active_root: &Path,
    convention: Convention,
    platform: Platform,
) -> Option<PathBuf> {
    if !is_first_run(root, &file, remembered, marked()) {
        return file;
    }
    let path = active_root.join(WELCOME_FILE);
    match seed(&path, convention, platform) {
        Ok(()) => {
            mark();
            Some(path)
        }
        Err(_) => file,
    }
}

#[cfg(test)]
mod tests;
