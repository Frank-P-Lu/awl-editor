//! build.rs — the ONE place a build-time FACT about this binary is captured.
//!
//! Today that is exactly one fact: `AWL_GIT_COMMIT`, the short commit this
//! binary was built from, read from the working tree's own git. It exists so
//! the macOS About window (`src/mac_about/`) can state a real provenance line
//! instead of a plausible-looking one.
//!
//! **The honesty rule this file exists to keep:** when git cannot answer —
//! no `git` on PATH, a source tarball with no repository, a vendored build —
//! this script emits NOTHING, `option_env!("AWL_GIT_COMMIT")` is `None`, and
//! the About window simply omits the commit line. There is no "unknown"
//! placeholder and no fallback string, because a placeholder in a facts block
//! reads as a fact. Absence is the honest answer.
//!
//! **Determinism:** no timestamp, no hostname, no build counter — nothing that
//! would make two builds of the same tree differ. The commit is a property of
//! the source, not of the machine.
//!
//! **Staleness:** cargo only reruns a build script when a declared input
//! changes, so the rerun triggers below name the two files git actually
//! rewrites when the checked-out commit moves — `HEAD` (branch switches,
//! detached checkouts) and the resolved ref file (a new commit on the current
//! branch). Both are resolved through `git rev-parse`, so this works the same
//! in a linked worktree (whose git dir is `.git/worktrees/<name>`) as in a
//! primary checkout.

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) else {
        // No git (or not a repository). Emit nothing — see the honesty rule.
        return;
    };
    let git_dir = Path::new(&git_dir);

    // Rerun when the checked-out commit moves. `HEAD` covers a branch switch;
    // the symbolic ref's own file covers a commit landing on the branch we are
    // already on. A detached HEAD has no symbolic ref — `HEAD` alone is then
    // the complete trigger.
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    if let Some(reference) = git(&["symbolic-ref", "--quiet", "HEAD"]) {
        // `git rev-parse --git-path <ref>` resolves refs/heads/x to wherever
        // this repository actually stores it (a linked worktree's HEAD ref
        // still lives in the COMMON dir, not the per-worktree one).
        if let Some(ref_path) = git(&["rev-parse", "--git-path", &reference]) {
            println!("cargo:rerun-if-changed={ref_path}");
        }
    }

    if let Some(commit) = git(&["rev-parse", "--short=12", "HEAD"]) {
        println!("cargo:rustc-env=AWL_GIT_COMMIT={commit}");
    }
}

/// Run `git <args>` and return its trimmed stdout, or `None` if git is absent,
/// fails, or answers with nothing. Never panics and never fails the build: a
/// missing commit fact is a missing LINE, not a broken build.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}
