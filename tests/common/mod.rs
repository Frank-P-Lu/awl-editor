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

use std::ops::Deref;
use std::path::{Path, PathBuf};
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

/// A monotonic counter folded into [`shared_sandbox`]'s path, so concurrent
/// test threads in one binary each get their OWN directory rather than racing
/// to wipe-then-recreate a shared one (queue item 168: a [`ScratchDir`] owns
/// exactly one directory, so a genuinely shared one is not this guard's job).
static SANDBOX_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// An EMPTY sandbox dir for spawns that have no tempdir of their own to hang a
/// config off (`--help`, `--list-worlds`, the storyboard runner, which all
/// write their outputs elsewhere) — nothing is ever written into it, it exists
/// only to give [`config_path_in`] a real parent. Bind the return value to a
/// variable that outlives the spawned child (its `Drop` removes the directory);
/// letting it drop immediately is also fine, since an ABSENT config path
/// resolves to defaults exactly like an empty one ([`awl`]'s doc).
pub fn shared_sandbox() -> ScratchDir {
    let seq = SANDBOX_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "awl-test-config-sandbox-{}-{seq}",
        std::process::id()
    ));
    ScratchDir::new(dir)
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

/// THE one owner of an on-disk scratch directory these integration tests
/// create under `std::env::temp_dir()` (queue item 168). A DUPLICATE of
/// `src/testscratch.rs`'s type of the same name and shape — unavoidable,
/// since this crate ships no `[lib]` target (`Cargo.toml`; the binary tests
/// spawn via `CARGO_BIN_EXE_awl` above), so a `tests/*.rs` integration binary
/// has no door into the main crate's `#[cfg(test)]` internals at all. Keep the
/// two in lockstep by hand; see `src/testscratch.rs`'s module doc for the
/// leak this closes and why removal on drop (not an explicit end-of-function
/// call) is the fix.
///
/// `tests/fault_kill9.rs` is the one NAMED exception in spirit, not in code:
/// it `SIGKILL`s a CHILD inside the pre-rename window of an atomic write, so
/// the child cannot clean up its own half-written sibling file — but the
/// PARENT test still wraps its directory in a `ScratchDir` like every other
/// fixture, and the guard's recursive removal deletes that orphaned file
/// along with everything else when the parent's test function returns.
pub struct ScratchDir(PathBuf);

impl ScratchDir {
    /// Create a fresh directory at `path`, wiping any stale leftover first
    /// (a prior crashed run, or one left by code that predates this guard).
    pub fn new(path: PathBuf) -> Self {
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch dir is creatable");
        ScratchDir(path)
    }

    /// A cross-process-safe alternative to [`ScratchDir::new`]: ATOMICALLY
    /// claims the first unused `<root>/<stem>-<suffix>` directory via
    /// `create_dir` (which fails if the path already exists) instead of
    /// wiping first. For a fixture where a same-named directory might still
    /// be owned by ANOTHER live or crashed test process — this integration
    /// binary has no door into the main crate's in-process
    /// `testlock::serial()`, so a blind remove-then-create is not exclusive
    /// ownership (`tests/hermetic_canary.rs`, the one caller). Never removes
    /// an existing directory.
    pub fn claim(root: &Path, stem: &str) -> Self {
        for suffix in 0_u64.. {
            let dir = root.join(format!("{stem}-{suffix}"));
            match std::fs::create_dir(&dir) {
                Ok(()) => return ScratchDir(dir),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("failed to claim scratch dir {}: {e}", dir.display()),
            }
        }
        unreachable!("the u64 suffix space is inexhaustible in one test run")
    }
}

impl Deref for ScratchDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for ScratchDir {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for ScratchDir {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
