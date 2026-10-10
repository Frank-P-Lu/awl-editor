//! Unsaved text held after external changes, under Awl's data root.
//!
//! One active conflict record follows the unresolved document. Explicitly
//! closing a dirty deleted document durably retains its latest text by path
//! identity before releasing the buffer. A later conflict cannot overwrite it.
//! Opening that path adopts the retained manuscript through the same conflict
//! owner; resolving that document clears only its own retained record.
//!
//! # The format, and why it is hand-rolled
//!
//! ```text
//! awl-unresolved-change 1
//! /absolute/path/to/the/users/file.md
//! …the user's text, verbatim, to the end of the file…
//! ```
//!
//! Two header lines then raw bytes. TOML would have to escape or quote the
//! document, which means a decoder bug can silently corrupt a manuscript — the
//! precise failure this file exists to prevent. Here the text is stored as
//! itself: any decoder that finds the two header lines recovers the remainder
//! byte-for-byte, and one that does not find them recovers nothing and says so,
//! rather than half-parsing. `session.toml` is hand-rolled for a related reason
//! and is the local precedent.
//!
//! A path containing a newline cannot be represented and is refused at encode
//! time rather than written unreadably; [`encode`] returns `None` and the caller
//! keeps the conflict in memory. That is a real if vanishing case, and losing
//! the record is survivable — losing it *silently* is not.

use std::path::{Path, PathBuf};

/// The first line of a well-formed record. Bumping the trailing number
/// invalidates every older record, which is correct: a record whose layout this
/// build does not understand must be ignored, never guessed at.
const MAGIC: &str = "awl-unresolved-change 1";

/// THE ONE RECORD PATH. Singular by design — see the module doc. Beside
/// `scratch.md` and `session.toml` under the same machine-state root, never
/// among the user's own documents.
pub fn record_path() -> PathBuf {
    crate::fs::data_root().join("unresolved-change.md")
}

/// An unresolved external change: which file, and the text awl is holding for
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// The user's file — the path awl has stopped writing to.
    pub path: PathBuf,
    /// The user's unsaved text. The whole document, not a diff: a diff would
    /// need the base it was taken against to still be on disk, and the entire
    /// premise here is that the disk moved.
    pub text: String,
}

/// Serialise. `None` when the path cannot be represented (it contains a
/// newline) — refused rather than written in a form that would decode as
/// something else.
pub fn encode(record: &Record) -> Option<String> {
    let path = record.path.to_str()?;
    if path.contains('\n') || path.is_empty() {
        return None;
    }
    Some(format!("{MAGIC}\n{path}\n{}", record.text))
}

/// Parse. `None` for anything that is not exactly this format — a truncated
/// write, a file from a future awl, or an unrelated file that happens to sit at
/// the path. A half-understood record is worse than none, because the caller
/// would restore a partial document over the user's real one.
pub fn decode(raw: &str) -> Option<Record> {
    let (magic, rest) = raw.split_once('\n')?;
    if magic != MAGIC {
        return None;
    }
    let (path, text) = rest.split_once('\n')?;
    if path.is_empty() {
        return None;
    }
    Some(Record {
        path: PathBuf::from(path),
        text: text.to_string(),
    })
}

/// Preserve the record atomically, replacing different durable bytes only on
/// success. An exact persisted record needs no redundant publication. Callers
/// must not treat a failed write as preservation of current text or permission
/// to discard it. Recovery never saves the original file.
pub fn write(record: &Record) -> bool {
    let Some(body) = encode(record) else {
        return false;
    };
    let path = record_path();
    let fs = crate::fs::active();
    // Exact persisted bytes already preserve this path and manuscript. An old
    // or unreadable record never handles a newer edit or a failed write.
    if fs.read_to_string(&path).ok().as_deref() == Some(body.as_str()) {
        return true;
    }
    if let Some(parent) = path.parent() {
        let _ = fs.create_dir_all(parent);
    }
    crate::durable::write(crate::durable::Owner::Recovery, &path, body.as_bytes()).is_ok()
}

/// READ THE RECORD, if there is one. A present-but-unparseable record is
/// preserved to a `.corrupt-*` sibling before being ignored — the same treatment
/// the scratch stash gets, and for the same reason: those bytes are a
/// manuscript, and the very next [`write`] would otherwise overwrite them.
pub fn read() -> Option<Record> {
    let path = record_path();
    let raw = crate::fs::active().read_to_string(&path).ok()?;
    match decode(&raw) {
        Some(record) => Some(record),
        None => {
            crate::durable::preserve_corrupt(&path, raw.as_bytes());
            None
        }
    }
}

/// DELETE THE RECORD — called on exactly one event, the conflict being
/// resolved. Best-effort: a record that outlives its conflict is noticed at the
/// next launch and discarded there ([`matches_path`] is what makes that safe),
/// which is a far better failure than a resolve that reports an error the user
/// can do nothing about.
#[cfg(test)]
pub fn clear() {
    let _ = crate::fs::active().remove_file(&record_path());
}

/// Closed deleted documents need independent recovery ownership: the next
/// unresolved document may replace or clear the active conflict record.
fn retained_path(path: &Path) -> PathBuf {
    let identity = crate::external::digest(path.as_os_str().as_encoded_bytes());
    crate::fs::data_root()
        .join("recovery")
        .join(format!("closed-{identity:016x}.md"))
}

/// Preserve the latest explicitly closed manuscript for this identity. A
/// failed durable write refuses the close, keeping the editable buffer alive.
pub fn retain_closed(record: &Record) -> bool {
    let Some(body) = encode(record) else {
        return false;
    };
    let path = retained_path(&record.path);
    let fs = crate::fs::active();
    if let Some(parent) = path.parent()
        && fs.create_dir_all(parent).is_err()
    {
        return false;
    }
    // Do not replace an unrelated record even if path hashing collides.
    match fs.read_to_string(&path) {
        Ok(raw) if decode(&raw).is_none_or(|old| old.path != record.path) => return false,
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return false,
        Ok(_) | Err(_) => {}
    }
    crate::durable::write(crate::durable::Owner::Recovery, &path, body.as_bytes()).is_ok()
}

/// Opening the original path recovers its retained text through the same
/// conflict mechanism. Other documents cannot consume or clear this record.
pub fn read_for(path: &Path) -> Option<Record> {
    read()
        .filter(|record| matches_path(record, path))
        .or_else(|| {
            let raw = crate::fs::active()
                .read_to_string(&retained_path(path))
                .ok()?;
            decode(&raw).filter(|record| matches_path(record, path))
        })
}

/// Refuse a close if its active record cannot be removed. The caller first
/// writes the latest text to both records, so failed removal cannot hide it.
pub fn clear_active_for(path: &Path) -> bool {
    let fs = crate::fs::active();
    let active = record_path();
    match fs.read_to_string(&active) {
        Ok(raw) => match decode(&raw) {
            Some(record) if matches_path(&record, path) => match fs.remove_file(&active) {
                Ok(()) => true,
                Err(error) => error.kind() == std::io::ErrorKind::NotFound,
            },
            Some(_) => true,
            None => false,
        },
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    }
}

pub fn clear_for(path: &Path) {
    let _ = clear_active_for(path);
    let retained = retained_path(path);
    let fs = crate::fs::active();
    if fs
        .read_to_string(&retained)
        .ok()
        .and_then(|raw| decode(&raw))
        .is_some_and(|record| matches_path(&record, path))
    {
        let _ = fs.remove_file(&retained);
    }
}

/// Does this record belong to `path`? The startup restore's own guard: a record
/// left over from a different file must never have its text loaded into the
/// document the user actually opened. Compared as paths, not strings, so a
/// trailing-slash or `.`-segment difference does not read as a different file.
pub fn matches_path(record: &Record, path: &Path) -> bool {
    record.path == path
}

#[cfg(test)]
mod tests;
