//! tests/common/mod.rs — THE ONE OWNER of "a spawned `awl` child never resolves
//! its config through the DEVELOPER'S OWN dotfiles" (queue item 93).
//!
//! ROOT CAUSE THIS MODULE EXISTS FOR. `config::config_path()`
//! (`src/config/model.rs`) walks a LADDER: explicit `--config` → `$AWL_CONFIG`
//! → `$XDG_CONFIG_HOME/awl/config.toml` → `$HOME/.config/awl/config.toml`.
//! Every binary-spawning test used to write `.env_remove("AWL_CONFIG")` under
//! the name of isolation — which is exactly BACKWARDS: removing the variable
//! drops the override and lets the child fall all the way through to the
//! developer's real `~/.config/awl/config.toml`. A personal `zoom = 1.5` there
//! silently rescaled every pixel metric in the child and turned
//! `tests/bullet_blank_line_nit_pixels.rs` and `tests/frost_rail_pixels.rs` red
//! on one box while CI (no dotfiles) stayed green — a phantom "regression" with
//! no product change behind it. `env_remove` is the ONE value for this variable
//! that guarantees fall-through; it must never be used for isolation again.
//!
//! THE RULE. A spawned child's config source is DECLARED, never inherited: the
//! first ladder rung it can reach must land inside a test-owned sandbox. There
//! are exactly two shapes of that, both below — [`awl`] pins the `$AWL_CONFIG`
//! rung, [`awl_in_home`] pins the `$HOME`/`$XDG_CONFIG_HOME` rungs (for the
//! canary, which tests the fall-through itself). Every `tests/*.rs` routes
//! through one of them; `tests/spawn_config_law.rs` is the law that no new test
//! can name the binary any other way.

// Each test target uses only the doors it needs; the unused ones are not dead.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

/// A `Command` for the real `awl` binary with its CONFIG LADDER PINNED to
/// `sandbox`.
///
/// `$AWL_CONFIG` is SET (never removed) to a path inside `sandbox`, so
/// `config_path()` short-circuits on its first env rung and the developer's
/// `$XDG_CONFIG_HOME` / `$HOME` are unreachable. The file does NOT need to
/// exist: config loading is total, and an absent path yields the pure built-in
/// defaults — which is what a hermetic capture wants. A test that needs a
/// specific setting writes that TOML at [`config_path_in`] first.
///
/// The user dictionary is derived from the config path's parent
/// (`config::user_dictionary_path`), so pinning this rung sandboxes that too.
pub fn awl(sandbox: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_awl"));
    cmd.env("AWL_CONFIG", config_path_in(sandbox));
    cmd
}

/// The config file [`awl`] points a child at, so a test that wants real
/// settings can write them there first. Named distinctly from `config.toml` so
/// it cannot collide with a sandbox that also carries an `.config/awl/` tree.
pub fn config_path_in(sandbox: &Path) -> std::path::PathBuf {
    sandbox.join("awl-sandbox-config.toml")
}

/// A process-wide, EMPTY sandbox dir for spawns that have no tempdir of their
/// own to hang a config off (`--help`, `--list-worlds`, the storyboard runner,
/// which all write their outputs elsewhere). `create_dir_all` is idempotent and
/// race-free, so parallel test threads in one binary share it safely — nothing
/// is ever written into it, it exists only to give [`config_path_in`] a real
/// parent.
pub fn shared_sandbox() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("awl-test-config-sandbox-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// The CANARY shape: pin the ladder one rung LOWER — `$HOME` and
/// `$XDG_CONFIG_HOME` inside `home`, with `$AWL_CONFIG` deliberately REMOVED so
/// resolution actually walks down into the canary tree. That fall-through is
/// precisely what `tests/hermetic_canary.rs` is measuring, so here (and only
/// here) the removal is the point rather than the bug. Still hermetic by the
/// module's rule: the rung the child lands on is inside a test-owned sandbox,
/// never the developer's real dirs.
pub fn awl_in_home(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_awl"));
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("AWL_CONFIG");
    cmd
}
