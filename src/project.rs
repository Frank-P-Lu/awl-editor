//! The active PROJECT: a root directory plus a tiny, read-only git status probe.
//!
//! Exactly one project is active at a time; its root scopes the go-to file
//! index. Git awareness is intentionally minimal and READ-ONLY — we never run a
//! mutating git command. We surface only the name, the branch, and whether the
//! worktree is dirty (reported in the capture sidecar's `project` block).

use std::path::{Path, PathBuf};

/// The resolved active project. `name` is the root's final path component (what
/// a developer calls the project). `branch`/`dirty` are populated for git roots
/// via `git rev-parse` / `git status --porcelain`; both are quiet read-only
/// probes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub root: PathBuf,
    pub name: String,
    pub is_git: bool,
    pub branch: Option<String>,
    pub dirty: bool,
}

/// **THE ONE RULE FOR WHAT A FOLDER IS CALLED** — its final path component,
/// falling back to the whole path when there is none (`/`).
///
/// Two surfaces say a folder's name out loud and they must agree: the
/// persistent gutter names the ACTIVE project ([`Project::name`]), and the
/// switch-project card's accept-this-folder row names the folder that row would
/// pick ([`crate::overlay::here_folder_label`]). A second derivation is how the
/// gutter and the card come to call one directory two different things.
///
/// Deliberately pure — no filesystem read, no git probe. [`Project::resolve`] is
/// the door that does those; a picker row is rebuilt per directory level and
/// cannot afford a subprocess.
pub fn folder_name(root: &Path) -> String {
    root.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string())
}

impl Project {
    /// Resolve the project for `root`: name from the path, git branch/dirty from
    /// the read-only probes (no-ops for a non-git root).
    pub fn resolve(root: &Path) -> Self {
        let name = folder_name(root);
        let is_git = crate::fs::active().exists(&root.join(".git"));
        let (branch, dirty) = if is_git {
            (git_branch(root), git_dirty(root))
        } else {
            (None, false)
        };
        Self {
            root: root.to_path_buf(),
            name,
            is_git,
            branch,
            dirty,
        }
    }
}

/// Current branch via `git rev-parse --abbrev-ref HEAD`. `None` on any failure
/// or a detached HEAD ("HEAD").
fn git_branch(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if b.is_empty() || b == "HEAD" {
        None
    } else {
        Some(b)
    }
}

/// Dirty == `git status --porcelain` produces ANY output.
fn git_dirty(root: &Path) -> bool {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain"])
        .output();
    match out {
        Ok(o) if o.status.success() => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gutter's project NAME and the switch-project card's
    /// accept-this-folder row are the same sentence about the same directory,
    /// so they are one rule. This sweeps the shapes that rule has to answer —
    /// an ordinary root, a trailing slash, a dotted name, the filesystem root
    /// with no final component, and a relative path — and asserts
    /// `Project::resolve` reports exactly what the shared owner returns.
    #[test]
    fn the_project_name_is_the_shared_folder_name_rule() {
        use std::sync::Arc;
        let cases = [
            "/plain",
            "/a/b/notes",
            "/a/b/notes/",
            "/a/.config",
            "/",
            "rel",
        ];
        let mut checked = 0usize;
        for case in cases {
            let p = PathBuf::from(case);
            let mem = crate::fs::InMemoryFs::new().with_dir(&p);
            let resolved = crate::fs::with_fs(Arc::new(mem), || Project::resolve(&p));
            assert_eq!(
                resolved.name,
                folder_name(&p),
                "{case:?}: the gutter's name and the shared owner's must agree"
            );
            assert!(!resolved.name.is_empty(), "{case:?}: named nothing");
            checked += 1;
        }
        assert_eq!(checked, 6, "the sweep lost cells");
        // The two ends of the rule, spelled out so a change to either is
        // visible here rather than only in a caller: the final component
        // ordinarily, the whole path when there is none.
        assert_eq!(folder_name(Path::new("/a/b/notes")), "notes");
        assert_eq!(folder_name(Path::new("/")), "/");
    }

    #[test]
    fn non_git_root_has_no_branch() {
        // The `.git` probe routes through the FILESYSTEM SEAM; an InMemoryFs dir with
        // no `.git` resolves as a non-git project (no real disk).
        use std::sync::Arc;
        let p = PathBuf::from("/plain");
        let mem = crate::fs::InMemoryFs::new().with_dir(&p);
        crate::fs::with_fs(Arc::new(mem), || {
            let proj = Project::resolve(&p);
            assert!(!proj.is_git);
            assert!(proj.branch.is_none());
            assert!(!proj.dirty);
        });
    }
}
