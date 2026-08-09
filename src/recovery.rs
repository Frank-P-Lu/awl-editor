//! src/recovery.rs — THE UNRESOLVED-CHANGE RECORD: exactly one, under awl's own
//! data root, holding the one thing that exists nowhere else.
//!
//! When a file changes on disk while awl holds unsaved edits, both versions are
//! real work and neither may be destroyed. The disk version is safe by
//! construction — awl simply stops writing to that path. The user's version is
//! safe only for as long as the process lives, and a process can be killed.
//!
//! So the unsaved text is written here, atomically, for exactly as long as the
//! conflict is unresolved. It is deleted the moment the user resolves, either
//! way.
//!
//! # What this is deliberately NOT
//!
//! Not a versioning system, not a backup, not a crash-recovery service, and not
//! a second copy of the user's file. There is **one** record, it exists **only**
//! while a conflict is open, and it holds **one** document. Local history
//! ([`crate::history`]) already owns "what did this file used to say"; the
//! scratch stash already owns "what was I typing with no file open". This owns
//! the narrow gap between them: text the user typed, that awl has been forbidden
//! to write to its own file, that would otherwise live only in RAM.
//!
//! It follows the scratch stash's pattern exactly — one path under
//! [`crate::fs::data_root`], one [`crate::fs::write_atomic`], read once at
//! startup — because that pattern is already proven for the same job, and a
//! second mechanism for the same job is how the two drift.
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

/// WRITE THE RECORD, atomically, replacing whatever was there. Best-effort by
/// signature: the caller is always in the middle of something more important
/// (a save being held, a quit in progress) and a failed record must never
/// escalate into a failed anything-else. The `bool` is for the laws that need to
/// assert the write happened.
pub fn write(record: &Record) -> bool {
    let Some(body) = encode(record) else {
        return false;
    };
    let path = record_path();
    let fs = crate::fs::active();
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
pub fn clear() {
    let _ = crate::fs::active().remove_file(&record_path());
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
