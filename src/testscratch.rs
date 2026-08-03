//! `testscratch` — THE one owner of an ON-DISK scratch directory a test
//! creates under `std::env::temp_dir()` (queue item 168).
//!
//! THE LEAK THIS CLOSES. Every fixture that needed a real directory (a
//! spawned-binary capture, a socket path, a save-to-disk round trip) used to
//! write its own three-line idiom: `remove_dir_all` any stale leftover,
//! `create_dir_all` a fresh one, do the test, `remove_dir_all` again at the
//! end of the function. That closing call is an EXPLICIT statement on the
//! happy path — it does not run when the test panics (a failed `assert!`),
//! returns early via `?`, or is killed. `$TMPDIR` accumulated 2,768 `awl-*`
//! directories (9.2 GB) this way; several fixtures (`tests/frost_rail_pixels.rs`,
//! `tests/bullet_blank_line_nit_pixels.rs`, `tests/common::shared_sandbox`)
//! never even reached a closing call — every run left one behind regardless
//! of pass or fail.
//!
//! THE FIX IS STRUCTURAL, NOT DISCIPLINE. [`ScratchDir`] owns exactly one
//! directory: created fresh in [`ScratchDir::new`] (a stale same-named leftover
//! is wiped first), removed recursively in its `Drop` impl. `Drop` runs on
//! every unwind path — a panic, an early `return`/`?`, or a normal fall-through
//! — so cleanup survives all three by construction, the same guarantee
//! `testlock::SerialGuard` and `fs::FsGuard` already give the process-global
//! test state. Deref/`AsRef<Path>` let it stand in for the `PathBuf` it wraps
//! at almost every call site.
//!
//! THE NAMED EXCEPTION: `tests/fault_kill9.rs`. It deliberately `SIGKILL`s a
//! CHILD process inside `fs::write_atomic`'s pre-rename window, so the child
//! cannot clean up its own `.<name>.awl-tmp` sibling file by construction —
//! that is the property under test, and "fixing" it would destroy the
//! rehearsal. Ownership of the directory belongs to the PARENT test process,
//! which still wraps it in a [`ScratchDir`] like everyone else: the guard's
//! recursive removal deletes the child's orphaned pre-rename file along with
//! everything else in the directory when the parent's test function returns,
//! regardless of how the child died.

#![cfg(test)]

use std::ops::Deref;
use std::path::{Path, PathBuf};

/// An RAII scratch directory. [`ScratchDir::new`] wipes any stale leftover at
/// `path` (a prior crashed run, or a leftover from before this guard existed)
/// and creates it fresh; dropping the guard removes it recursively. Wrap the
/// full path — including the `-{pid}` suffix most fixtures already compute —
/// so the on-disk naming a fixture is known by (and any doc that names a
/// specific `awl-*` directory) is unchanged.
pub(crate) struct ScratchDir(PathBuf);

impl ScratchDir {
    /// Create a fresh directory at `path`, removing anything already there.
    pub(crate) fn new(path: PathBuf) -> Self {
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("scratch dir is creatable");
        ScratchDir(path)
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

impl std::fmt::Debug for ScratchDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stem(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "awl-testscratch-selftest-{tag}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn drop_removes_the_directory_on_the_happy_path() {
        let path = stem("happy");
        let dir = ScratchDir::new(path.clone());
        std::fs::write(dir.join("f.txt"), b"x").unwrap();
        assert!(path.is_dir());
        drop(dir);
        assert!(!path.exists(), "the guard removes its directory on drop");
    }

    #[test]
    fn drop_removes_the_directory_when_the_scope_unwinds_from_a_panic() {
        let path = stem("panic");
        let caught = std::panic::catch_unwind(|| {
            let dir = ScratchDir::new(path.clone());
            std::fs::write(dir.join("f.txt"), b"x").unwrap();
            panic!("deliberate: prove Drop still fires on unwind");
        });
        assert!(caught.is_err());
        assert!(
            !path.exists(),
            "a panicking scope still drops (and thus removes) the guard"
        );
    }

    #[test]
    fn drop_removes_the_directory_when_the_caller_returns_early() {
        // Distinct from the happy-path test above: this models the exact
        // shape most fixtures use — `let dir = ScratchDir::new(..); if guard {
        // return; }` — an early `?`/`return` this guard must survive, not
        // just a normal fall-through. (Rust's `Drop` does not actually
        // distinguish an early `return` from falling off a function's end —
        // both are ordinary, non-unwinding scope exits — so this and the
        // happy-path test above jointly cover every non-panicking exit; the
        // panic test above covers the third, unwinding one.)
        fn early_return_after_creating(path: &Path) -> &'static str {
            let _dir = ScratchDir::new(path.to_path_buf());
            if true {
                return "returned early";
            }
            unreachable!()
        }
        let path = stem("early-return");
        assert_eq!(early_return_after_creating(&path), "returned early");
        assert!(
            !path.exists(),
            "the guard must drop (and remove its directory) across an early return, \
             not only at the end of the function"
        );
    }

    #[test]
    fn new_wipes_a_stale_leftover_directory_first() {
        let path = stem("stale");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("old.txt"), b"leftover").unwrap();
        let dir = ScratchDir::new(path.clone());
        assert!(
            !dir.join("old.txt").exists(),
            "a fresh guard starts from an empty directory, not a stale one"
        );
    }

    // --- THE NO-WILDCARD LAW (queue item 168): every OTHER file in `src/` ---

    /// A source file's CODE, with every `//`-comment cut away — mirrors
    /// `tests/spawn_config_law.rs`'s `code_only` (both laws need the same "a
    /// raw text scan would convict this file's own prose" guard). String-aware:
    /// a `//` inside a string literal is not a comment.
    fn code_only(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for line in text.lines() {
            let bytes = line.as_bytes();
            let mut in_str = false;
            let mut i = 0;
            let cut = loop {
                if i >= bytes.len() {
                    break bytes.len();
                }
                match bytes[i] {
                    b'\\' if in_str => i += 1,
                    b'"' => in_str = !in_str,
                    b'/' if !in_str && bytes.get(i + 1) == Some(&b'/') => break i,
                    _ => {}
                }
                i += 1;
            };
            out.push_str(&line[..cut]);
            out.push('\n');
        }
        out
    }

    /// Every `src/**/*.rs` file (this module's own file excepted — it is the
    /// owner, not a fixture), as `(path relative to src/, comment-stripped code)`.
    fn src_sources() -> Vec<(String, String)> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut out = Vec::new();
        walk_rs(&root, &mut out);
        out.retain(|(name, _): &(String, String)| name != "testscratch.rs");
        out.sort();
        out
    }

    fn walk_rs(dir: &Path, out: &mut Vec<(String, String)>) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk_rs(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                let name = p
                    .strip_prefix(&root)
                    .unwrap_or(&p)
                    .to_string_lossy()
                    .into_owned();
                let src = std::fs::read_to_string(&p).expect("source reads");
                out.push((name, code_only(&src)));
            }
        }
    }

    /// LAW 1: the literal call may appear in exactly this file's [`ScratchDir`]
    /// `Drop` impl. Built in two pieces so this file's own doc prose (which
    /// names the call) cannot trip its own law.
    fn banned_removal() -> String {
        format!("remove_dir{}", "_all")
    }

    #[test]
    fn only_the_guard_calls_remove_dir_all() {
        let banned = banned_removal();
        let offenders: Vec<String> = src_sources()
            .into_iter()
            .filter(|(_, text)| text.contains(&banned))
            .map(|(name, _)| name)
            .collect();
        assert!(
            offenders.is_empty(),
            "these files call remove_dir_all directly instead of letting a \
             `testscratch::ScratchDir` guard's Drop own the cleanup: {offenders:?}. \
             An explicit end-of-function remove is exactly the happy-path-only \
             idiom queue item 168 retired — it never runs on a panic or an early \
             return. Wrap the directory in `ScratchDir::new` instead."
        );
    }

    /// LAW 2: a bare, never-wrapped `std::env::temp_dir()` root. The few real
    /// exceptions, each named rather than silently tolerated:
    /// - `fs/serialization_law.rs` reads the bare system temp root with
    ///   `fs::active().is_dir(..)` — nothing is ever created there.
    /// - `main/run.rs` is `--soak-gpu`, LIVE production code (not a test).
    /// - `main/tests.rs` and `probe.rs` each build ONE loose FILE path (a PNG /
    ///   log file, never a directory) directly under the OS temp root.
    /// - `render/benchsuite/scenarios.rs` is `--bench-suite`, LIVE dev-tool
    ///   code (not a test), and only READS the bare temp root as a placeholder.
    /// - `daemon.rs` builds ONE loose `.sock` FILE path per test, cleaned via
    ///   `remove_file` (a different primitive, not `ScratchDir`'s concern).
    const UNWRAPPED_ALLOWLIST: &[&str] = &[
        "fs/serialization_law.rs",
        "main/run.rs",
        "main/tests.rs",
        "probe.rs",
        "render/benchsuite/scenarios.rs",
        // `--bench-a11y` only READS `temp_dir()` as an App root; it creates no
        // directory and writes nothing, so there is nothing to clean up. Live
        // CLI code, not a test-owned directory.
        "app/semantic/bench.rs",
        "daemon.rs",
    ];

    #[test]
    fn every_new_scratch_root_is_wrapped_in_scratch_dir() {
        let needle = "std::env::temp_dir()";
        let mut offenders: Vec<String> = Vec::new();
        for (name, text) in src_sources() {
            if UNWRAPPED_ALLOWLIST.contains(&name.as_str()) {
                continue;
            }
            let calls = text.matches(needle).count();
            let wraps = text.matches("ScratchDir::new(").count();
            if calls > wraps {
                offenders.push(format!(
                    "{name} ({calls} temp_dir() call(s), {wraps} ScratchDir::new wrap(s))"
                ));
            }
        }
        assert!(
            offenders.is_empty(),
            "these files build a scratch root off std::env::temp_dir() without \
             routing it through testscratch::ScratchDir::new: {offenders:?}. An \
             unwrapped root is either never cleaned up at all, or cleaned up only \
             on the happy path (queue item 168) — add it to UNWRAPPED_ALLOWLIST \
             here ONLY if it names a single loose file, or is live (non-test) \
             production code, never a test-owned directory."
        );
    }
}
