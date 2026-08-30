//! FRESH-DOCUMENT NAMING + FILE MOVES — the pure helpers behind a document's
//! one-shot auto-naming and the C-x m move: `first_nonempty_line` (a fresh
//! document's working title), `note_stem` / `slug_core` (title -> filename
//! stem), and `unique_path` / `move_file` (no-clobber path selection + true
//! moves over the filesystem seam). Free functions carved out of `buffer.rs`
//! verbatim; glob-re-exported from the module root so the `crate::buffer::*`
//! call sites resolve unchanged. The LIVE-rename-to-title machinery
//! (`rename_to_stem`/`stem_matches_slug`) is retired — naming is now one-shot,
//! at the first material save only (see `Buffer::save`'s doc).

use super::Buffer;
use ropey::Rope;
use std::path::{Path, PathBuf};

/// A calm filename budget. With `.md` (3 bytes), the largest `u32` collision
/// suffix (11), the longest per-attempt atomic temp decoration (41), and a
/// deliberately reserved 64-byte quarantine decoration, the worst component
/// is 191 bytes — 64 bytes below the 255-byte component limit on advertised macOS and
/// Linux filesystems. Smaller filesystem limits can still reject a name; the
/// naming transaction leaves buffer identity intact on that ordinary error.
pub const NOTE_STEM_MAX_BYTES: usize = 72;

impl Buffer {
    pub fn display_name(&self) -> String {
        if let Some(p) = &self.path {
            return p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "scratch".to_string());
        }
        if self.fresh_id.is_some() {
            return "untitled".to_string();
        }
        let stem = match first_nonempty_line(&self.rope.to_string()) {
            Some(line) => note_stem(line),
            None => "scratch".to_string(),
        };
        format!("{stem}.md")
    }

    /// Mark this buffer as a freshly-summoned, UNNAMED document living under
    /// `dir`: it has no filename yet; the first non-empty line names it ONCE, on
    /// the first material save ([`Self::save`] then clears this — the
    /// one-shot naming law: a LATER title edit never re-triggers a rename, since
    /// [`Self::is_unnamed_fresh`] is false from that first save on).
    pub fn set_note_dir(&mut self, dir: PathBuf) {
        self.note_dir = Some(dir);
    }

    pub fn is_unnamed_fresh(&self) -> bool {
        self.note_dir.is_some()
    }

    /// A just-created fresh buffer contains no user work and has no edit/undo
    /// history to protect. It may close without forcing an impossible naming
    /// save. Once any edit dirties it, even if the visible text returns empty,
    /// the normal save/refusal gate owns the close.
    pub(crate) fn is_discardable_empty_fresh(&self) -> bool {
        self.is_unnamed_fresh() && !self.dirty && self.rope.len_chars() == 0
    }

    pub(crate) fn fresh_id(&self) -> Option<u64> {
        self.fresh_id
    }

    pub fn start_fresh_doc(&mut self, dir: PathBuf) {
        static NEXT_FRESH_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        *self = Self::from_rope(Rope::new(), None);
        self.note_dir = Some(dir);
        self.fresh_id = Some(NEXT_FRESH_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    }
}

/// The first line of `text` with non-whitespace content (trimmed), or `None` when
/// the text is empty / all blank. This is a quick note's working TITLE.
pub fn first_nonempty_line(text: &str) -> Option<&str> {
    text.lines().map(|l| l.trim()).find(|l| !l.is_empty())
}

/// The filename STEM a note's first `line` derives to: its [`slug_core`], or the
/// "scratch" placeholder when the line has no slug-able (alphanumeric) content.
/// Every caller inherits one UTF-8-byte cap. Prefer a complete dash-delimited
/// word; a single long word falls back to the last valid scalar boundary.
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
#[cfg(test)]
pub fn move_file(old: &Path, dest_dir: &Path) -> std::io::Result<PathBuf> {
    move_file_avoiding(old, dest_dir, |_| false)
}

/// The live-session move: disk paths and identities already claimed by another
/// open buffer share the same unavailable predicate.
pub(crate) fn move_file_avoiding(
    old: &Path,
    dest_dir: &Path,
    mut unavailable: impl FnMut(&Path) -> bool,
) -> std::io::Result<PathBuf> {
    crate::fs::active().create_dir_all(dest_dir)?;
    let filename = old
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    let natural = dest_dir.join(&filename);
    if natural == old {
        return Ok(old.to_path_buf()); // already there
    }
    let p = Path::new(&filename);
    let stem = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = p
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    loop {
        let new_path = unique_path_avoiding(dest_dir, &stem, &ext, &mut unavailable);
        match crate::fs::active().rename_no_replace(old, &new_path) {
            Ok(()) => return Ok(new_path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let new_path = unique_path_avoiding(dest_dir, &stem, &ext, &mut unavailable);
        crate::fs::active().rename(old, &new_path)?;
        Ok(new_path)
    }
}

/// A NON-CLOBBERING path in `dir` for `stem`.`ext` (`ext` empty = no extension):
/// returns `<dir>/<stem>.<ext>` if free, else the first free `<stem>-2.<ext>`,
/// `<stem>-3.<ext>`, … So a note title collision (or a move into a folder that
/// already holds a same-named file) appends a short numeric suffix rather than
/// overwriting.
#[cfg(test)]
pub fn unique_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    unique_path_avoiding(dir, stem, ext, |_| false)
}

/// The one no-clobber allocator. A candidate is unavailable when it exists on
/// disk OR when `reserved` says a live buffer already owns its normalized key.
pub(crate) fn unique_path_avoiding(
    dir: &Path,
    stem: &str,
    ext: &str,
    mut reserved: impl FnMut(&Path) -> bool,
) -> PathBuf {
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
    while crate::fs::active().exists(&candidate) || reserved(&candidate) {
        candidate = dir.join(name(Some(n)));
        n += 1;
    }
    candidate
}

/// Publish a new file under the shared disk-plus-live allocator. Native uses
/// create-if-absent publication and retries the suffix if another creator wins
/// after selection; the browser filesystem has no concurrent native publisher.
pub(crate) fn write_new_unique(
    owner: crate::durable::Owner,
    dir: &Path,
    stem: &str,
    ext: &str,
    data: &[u8],
    mut unavailable: impl FnMut(&Path) -> bool,
) -> std::io::Result<PathBuf> {
    crate::fs::active().create_dir_all(dir)?;
    #[cfg(not(target_arch = "wasm32"))]
    loop {
        let candidate = unique_path_avoiding(dir, stem, ext, &mut unavailable);
        match crate::durable::write_new(owner, &candidate, data) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let candidate = unique_path_avoiding(dir, stem, ext, &mut unavailable);
        crate::durable::write(owner, &candidate, data)?;
        Ok(candidate)
    }
}
