//! Diagnostic-preserving reads of one directory level.

use std::path::{Path, PathBuf};

use super::is_junk_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_git: bool,
}

/// Resolve `rel` below `root`; `None` and an empty path name the root itself.
pub fn resolve_dir_level(root: &Path, rel: Option<&str>) -> PathBuf {
    match rel {
        Some(rel) if !rel.is_empty() => root.join(rel),
        _ => root.to_path_buf(),
    }
}

/// Compatibility projection for callers that treat an unreadable level as empty.
pub fn list_dir_level(root: &Path, rel: Option<&str>) -> Vec<DirEntry> {
    try_list_dir_level(root, rel).unwrap_or_default()
}

/// List directories first and files second, preserving an unreadable level as `None`.
pub fn try_list_dir_level(root: &Path, rel: Option<&str>) -> Option<Vec<DirEntry>> {
    let dir = resolve_dir_level(root, rel);
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in crate::fs::active().read_dir(&dir).ok()? {
        let name = entry.name;
        if entry.is_dir {
            if is_junk_dir(&name) {
                continue;
            }
            dirs.push(DirEntry {
                name,
                is_dir: true,
                is_git: crate::fs::active().exists(&entry.path.join(".git")),
            });
        } else if entry.is_file {
            files.push(DirEntry {
                name,
                is_dir: false,
                is_git: false,
            });
        }
    }
    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    dirs.extend(files);
    Some(dirs)
}
