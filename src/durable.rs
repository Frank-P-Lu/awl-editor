//! Preserve malformed app-owned TOML stores before lenient fallback overwrites
//! them. The user-edited config remains exempt.
//!
//! [`load_toml_store`] is the ONE shared loader every TOML-backed store
//! (`session`, `stats`, `recents`, `mas::GrantStore`) calls: it distinguishes
//! absence from a present file whose TOML syntax failed to parse, or
//! it isn't even valid UTF-8" (a real corruption signal — preserved). A
//! A valid TOML file with a missing/wrong-typed field takes the lenient path
//! every `from_toml` already handles on purpose (an old store missing
//! a newly-added field) without creating `.corrupt-*` siblings.
//!
//! **Config is EXEMPT from the sibling-copy rule (documented, not an
//! oversight):** `config.toml` is the user's own hand-edited file — a parse
//! failure there is almost always a typo the user just made, not disk
//! corruption, and `Config::load` already has its own correct-for-that-case
//! behavior (keep the prior in-memory values + show a notice, never
//! silently reset to defaults — see `config/model.rs`). Backing up a
//! `.corrupt-*` sibling on every fat-fingered edit would litter the config
//! dir for a case that isn't data loss at all (the user's editor buffer +
//! undo history still has their intended text). So `config::write` keeps
//! routing every write through [`crate::fs::write_atomic`] (the PART 1
//! durability fix) but never calls [`preserve_corrupt`].
//!
//! **The history log is a SEPARATE format (not TOML)** and gets its own
//! corruption check colocated with its own parser
//! (`history::store::read_log`) — see that module for the "does this log
//! look trustworthy" logic — but calls the SAME [`preserve_corrupt`] here.
//! Scratch's one possible parse failure (invalid UTF-8) is checked in `app.rs`.

pub(crate) mod owner;
pub use owner::{Owner, write};
use std::path::Path;

/// How many `.corrupt-*` siblings a single store keeps — a generous but
/// bounded window (mirrors `crashlog::MAX_CRASH_LOGS`'s "look back across a
/// bad week, never an unbounded pile" reasoning, just narrower: a corrupt
/// store is a much rarer event than a crash).
pub const CORRUPT_BACKUP_KEEP: usize = 5;

/// The corrupt-backup sibling's file name for a store whose own file name is
/// `name`, stamped at `now_ms` (millis since the Unix epoch) plus `seq` (a
/// per-process MONOTONIC disambiguator — see [`next_seq`]). Both are
/// zero-padded to a fixed width so a plain lexical sort of file names IS a
/// chronological sort (millis since epoch comfortably fits in 20 digits for
/// millennia; `seq` in 10) — the same "sortable by construction" trick
/// `crashlog::utc_timestamp` uses with zero-padded date fields. `seq` alone
/// (not `now_ms` alone) is what actually GUARANTEES uniqueness: two corrupt
/// loads landing in the same wall-clock millisecond — entirely realistic
/// under a tight burst, e.g. this module's own prune test — would otherwise
/// collide on the SAME file name and silently overwrite one backup with the
/// next, defeating the whole "keep the newest N" contract before it even
/// starts.
pub fn corrupt_backup_name(name: &str, now_ms: u128, seq: u64) -> String {
    format!("{name}.corrupt-{now_ms:020}-{seq:010}")
}

/// The next value in a process-wide MONOTONIC counter, used purely to
/// disambiguate [`corrupt_backup_name`] when two backups land in the same
/// millisecond (see that function's doc). Never reset, never wraps in any
/// realistic process lifetime (`u64`).
fn next_seq() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// PURE: given the file names present in a store's directory, return the
/// `.corrupt-*` siblings of `stem` to DELETE so at most `keep` newest
/// survive. Sorts lexically (chronological by construction — see
/// [`corrupt_backup_name`]) and never touches any file that isn't a
/// `<stem>.corrupt-*` sibling, so it can't accidentally prune the live store
/// file or another store's siblings sharing the same directory.
pub fn corrupt_siblings_to_prune(names: &[String], stem: &str, keep: usize) -> Vec<String> {
    let prefix = format!("{stem}.corrupt-");
    let mut matching: Vec<&String> = names.iter().filter(|n| n.starts_with(&prefix)).collect();
    matching.sort();
    if matching.len() <= keep {
        return Vec::new();
    }
    matching[..matching.len() - keep]
        .iter()
        .map(|s| (*s).clone())
        .collect()
}

/// PRESERVE a corrupt store's raw bytes: write them to a timestamped sibling
/// beside `path`, then prune down to [`CORRUPT_BACKUP_KEEP`] newest. Called
/// ONLY when a load found the file PRESENT but unparseable/undecodable —
/// NEVER when the file is simply absent (every call site here is gated on
/// that distinction; see [`load_toml_store`] and `history::store::read_log`).
///
/// Best-effort throughout: a failure to write the backup or to list/prune
/// the directory is swallowed (the lenient load this protects must proceed
/// regardless — losing the ABILITY to recover a corrupt file is far better
/// than losing the EDITOR over a filesystem hiccup while trying to save one).
pub fn preserve_corrupt(path: &Path, raw: &[u8]) {
    let fs = crate::fs::active();
    let Some(parent) = path.parent() else { return };
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return;
    };
    let now_ms = crate::clock::system_now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let backup_path = parent.join(corrupt_backup_name(&name, now_ms, next_seq()));
    let _ = fs.write(&backup_path, raw);
    if let Ok(entries) = fs.read_dir(parent) {
        let existing: Vec<String> = entries
            .into_iter()
            .filter(|e| e.is_file)
            .map(|e| e.name)
            .collect();
        for stale in corrupt_siblings_to_prune(&existing, &name, CORRUPT_BACKUP_KEEP) {
            let _ = fs.remove_file(&parent.join(&stale));
        }
    }
}

/// The SHARED loader for every TOML-backed store (`session`, `stats`,
/// `recents`, `mas::GrantStore`): reads `path` through the active
/// `FileSystem`, and — ONLY when the file is PRESENT — checks whether it
/// preserves before handing it to `parse` (each store's own `from_toml`,
/// which stays exactly as lenient about individual FIELDS as before this
/// round: a valid-but-incomplete table is not corruption).
///
/// Three outcomes:
///   - file absent (`NotFound`) → `T::default()`, nothing preserved (there
///     was nothing to lose).
///   - file present, valid UTF-8, but its TOML SYNTAX fails to parse → the
///     raw text is preserved, then `parse("")`-equivalent proceeds (in
///     practice every `from_toml` returns `T::default()` on unparseable
///     input, so this is `T::default()` too — but routed through the same
///     `parse` closure so a future looser recovery strategy stays a
///     one-function change).
///   - file present but not valid UTF-8 at all (`read_to_string` errors on
///     something other than `NotFound`) → the RAW BYTES are preserved
///     (best-effort re-read via `fs.read`), and `T::default()`.
pub fn load_toml_store<T: Default>(path: &Path, parse: impl FnOnce(&str) -> T) -> T {
    let fs = crate::fs::active();
    match fs.read_to_string(path) {
        Ok(src) => {
            if src.parse::<toml::Table>().is_err() {
                preserve_corrupt(path, src.as_bytes());
            }
            parse(&src)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => T::default(),
        Err(_) => {
            // Present but not valid UTF-8 (or some other read failure short
            // of NotFound): try a raw byte read to preserve what we can.
            if let Ok(raw) = fs.read(path) {
                preserve_corrupt(path, &raw);
            }
            T::default()
        }
    }
}

#[cfg(test)]
mod tests;
