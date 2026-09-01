//! Tests for `main.rs` (the `--keys` replay harness, launch-context
//! resolution, and headless-safety laws) -- split by SUBJECT (the 2026-08
//! code-organization pass, following `render/tests/`'s established shape)
//! out of one 4026-line `run::tests` file (`main.rs`'s `#[path =
//! "main/run.rs"] mod run;`, whose own `mod tests;` is the module this
//! directory's file location has always answered to) into this
//! `main/tests/` directory -- every test's NAME is unchanged, only its
//! module path grew one segment (`run::tests::foo` ->
//! `run::tests::<subject>::foo`). `use super::*;` here still resolves to
//! the crate root exactly as before the split. The replay harness itself
//! (`keyspec::parse_keys`,
//! `replay_keys`, `replay_keys_mode`, `replay_keys_mode_isolated`) is used
//! by nearly every test in every subject file, so it stays HERE rather than
//! duplicated or arbitrarily assigned to one subject; each child module
//! reaches it via a targeted `use super::{..};` for whichever of it (and
//! this module's own other shared helpers) it actually calls.
use super::*;

mod buffer_switching;
mod capture_scenarios;
mod caret_mode;
mod credits_capture;
mod goto_project;
mod headless_safety;
mod history;
mod insert_link;
mod launch_context;
mod minibuffers;
mod move_lines;
mod page_geometry;
mod page_measure;
mod palette;
mod project_info_literals;
mod project_switching;
mod replay_ownership;
mod replay_warnings;
mod search;
mod sentence_motion;
mod settings_persist;
mod shift_select;
mod visual_motion;

// CONVENTION-PROOF SHADOWS: this whole file's `--keys` replay tests hardcode
// MAC-form literal specs ("Cmd-S-h", "s-p", a bare "C-n"/"C-x" whose letter
// Linux's collision table displaces, …) — pinning resolution to
// `Convention::Mac` is the honest fix (these tests document specifically
// what a MAC-convention chord does; Linux's own displacement/collision
// behavior is separately, exhaustively law-tested in `keymap.rs`). Chord
// PARSING is now convention-free (`parse_chords` never touches the keymap),
// so the pinning moved WITH resolution into the replay loop: these local
// `replay_keys`/`replay_keys_mode` wrappers SHADOW the module-level fns
// (a local item wins over a glob import) and supply a Mac-pinned
// `KeymapState`, so none of the ~60 call sites below needed rewriting. The
// local `keyspec` module keeps the old call shape for the same reason.
mod keyspec {
    pub fn parse_keys(spec: &str) -> anyhow::Result<Vec<crate::keyspec::Chord>> {
        crate::keyspec::parse_chords(spec)
    }
}

#[allow(clippy::too_many_arguments)]
fn replay_keys_mode(
    mode: crate::replay::Mode,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
) -> Result<ReplayResult> {
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    super::replay_keys_mode(
        mode,
        crate::replay::FilesystemCapability::None,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
        &mut km,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(not(target_arch = "wasm32"))]
fn replay_keys_mode_isolated(
    mode: crate::replay::Mode,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
) -> Result<ReplayResult> {
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    super::replay_keys_mode(
        mode,
        crate::replay::FilesystemCapability::Isolated,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
        &mut km,
    )
}

#[allow(clippy::too_many_arguments)]
fn replay_keys(
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
) -> ReplayResult {
    match replay_keys_mode(
        crate::replay::Mode::Permissive,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
    ) {
        Ok(res) => res,
        Err(e) => unreachable!("permissive replay never aborts: {e}"),
    }
}
