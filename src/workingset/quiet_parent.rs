//! THE ONE PARENT-ELIDING PRIMITIVE and its companion ancestor finder — split
//! out of `workingset.rs` to keep that file under this repo's production
//! line ceiling. Two callers share it: [`super::OpenFile::parent_label`]
//! (a file's own parent, relative to its root) and
//! [`super::panel::group_parent_label`] (a project root, relative to the
//! deepest ancestor it shares with a same-leaf rival) — the same
//! strip-then-format rule, one level apart, so neither owns a second copy.

use std::path::{Path, PathBuf};

/// `full`'s path relative to `base`, with a trailing separator
/// (`"journal/"`), or `None` when `full` does not live under `base` or names
/// `base` itself — there is no location to add, and drawing an empty span
/// would reserve width for nothing.
pub(super) fn quiet_relative_label(full: &Path, base: &Path) -> Option<String> {
    let rel = full.strip_prefix(base).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    let mut s = rel.to_string_lossy().replace('\\', "/");
    s.push('/');
    Some(s)
}

/// The deepest ancestor `a` and `b` share, component-wise. Empty (`""`) when
/// they share nothing but a root, or a filesystem root component when even
/// that differs (e.g. two Windows drives) — [`quiet_relative_label`] then
/// answers `None` for either, the honest "no shared context to name" case.
pub(super) fn common_ancestor(a: &Path, b: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for (ca, cb) in a.components().zip(b.components()) {
        if ca != cb {
            break;
        }
        out.push(ca.as_os_str());
    }
    out
}
