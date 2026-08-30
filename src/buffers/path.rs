//! Lenient normalized-path helpers for live buffer identity.

use std::path::{Component, Path, PathBuf};

pub(super) fn lexically_collapse(abs: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in abs.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve the longest existing ancestor, then restore the absent tail.
pub(super) fn canonicalize_lenient(clean: &Path) -> PathBuf {
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut ancestor = clean;
    loop {
        let Some(parent) = ancestor.parent() else {
            return clean.to_path_buf();
        };
        if let Some(name) = ancestor.file_name() {
            tail.push(name.to_os_string());
        }
        ancestor = parent;
        if let Ok(canon_ancestor) = std::fs::canonicalize(ancestor) {
            let mut out = canon_ancestor;
            for comp in tail.iter().rev() {
                out.push(comp);
            }
            return out;
        }
    }
}
