//! FRESH-DOCUMENT NAMING + FILE MOVES — the pure helpers behind a document's
//! one-shot auto-naming and the C-x m move: `first_nonempty_line` (a fresh
//! document's working title), `note_stem` / `slug_core` (title -> filename
//! stem), and `unique_path` / `move_file` (no-clobber path selection + true
//! moves over the filesystem seam). Free functions carved out of `buffer.rs`
//! verbatim; glob-re-exported from the module root so the `crate::buffer::*`
//! call sites resolve unchanged. The LIVE-rename-to-title machinery
//! (`rename_to_stem`/`stem_matches_slug`) is retired — naming is now one-shot,
//! at the first material save only (see `Buffer::save`'s doc).

use std::path::{Path, PathBuf};

/// A calm filename budget with room for `.md`, atomic `.{name}.awl-tmp`,
/// quarantine (~37 bytes), and ordinary collision suffixes under NAME_MAX=255.
pub const NOTE_STEM_MAX_BYTES: usize = 72;

/// The first line of `text` with non-whitespace content (trimmed), or `None` when
/// the text is empty / all blank. This is a quick note's working TITLE.
pub fn first_nonempty_line(text: &str) -> Option<&str> {
    text.lines().map(|l| l.trim()).find(|l| !l.is_empty())
}

/// The filename STEM a note's first `line` derives to: its [`slug_core`], or the
/// "scratch" placeholder when the line has no slug-able (alphanumeric) content.
/// Shared by the FIRST naming save and live-rename so both agree on the name.
pub fn note_stem(line: &str) -> String {
    let mut s = slug_core(line);
    if s.len() > NOTE_STEM_MAX_BYTES {
        let mut end = NOTE_STEM_MAX_BYTES;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        let prefix = &s[..end];
        end = prefix.rfind('-').filter(|&at| at > 0).unwrap_or(end);
        s.truncate(end);
        while s.ends_with('-') {
            s.pop();
        }
    }
    if s.is_empty() {
        "scratch".to_string()
    } else {
        s
    }
}

/// The raw slug for `line`: lowercase alphanumerics with non-alphanumeric runs
/// collapsed to single dashes (edges trimmed). Returns an EMPTY string when the
/// line has no alphanumeric content, so the caller ([`note_stem`]) decides the
/// fallback. A single word stays a single word ("foo" -> "foo").
fn slug_core(line: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in line.chars() {
        if c.is_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else {
            pending_dash = true;
        }
    }
    out
}

/// MOVE the file at `old` into `dest_dir`, KEEPING its filename: create the
/// destination directory if needed, never clobber an existing same-named file
/// there (append a numeric suffix on collision), and `std::fs::rename` (a true
/// move, not a copy). Returns the new path; an already-in-place move is a no-op
/// returning `old`. This is the only file-WRITE the move feature performs, scoped
/// to the current note (the C-x m fence: create + move, nothing else).
pub fn move_file(old: &Path, dest_dir: &Path) -> std::io::Result<PathBuf> {
    crate::fs::active().create_dir_all(dest_dir)?;
    let filename = old
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    let natural = dest_dir.join(&filename);
    if natural == old {
        return Ok(old.to_path_buf()); // already there
    }
    let new_path = if crate::fs::active().exists(&natural) {
        let p = Path::new(&filename);
        let stem = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = p
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        unique_path(dest_dir, &stem, &ext)
    } else {
        natural
    };
    crate::fs::active().rename(old, &new_path)?;
    Ok(new_path)
}

/// A NON-CLOBBERING path in `dir` for `stem`.`ext` (`ext` empty = no extension):
/// returns `<dir>/<stem>.<ext>` if free, else the first free `<stem>-2.<ext>`,
/// `<stem>-3.<ext>`, … So a note title collision (or a move into a folder that
/// already holds a same-named file) appends a short numeric suffix rather than
/// overwriting.
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let name = |suffix: Option<u32>| -> String {
        let base = match suffix {
            None => stem.to_string(),
            Some(n) => format!("{stem}-{n}"),
        };
        if ext.is_empty() {
            base
        } else {
            format!("{base}.{ext}")
        }
    };
    let mut candidate = dir.join(name(None));
    let mut n = 2u32;
    while crate::fs::active().exists(&candidate) {
        candidate = dir.join(name(Some(n)));
        n += 1;
    }
    candidate
}
