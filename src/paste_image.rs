//! PASTE-IMAGE (native, LIVE-App-only): when the OS clipboard holds an IMAGE
//! rather than text, awl saves it as a PNG into an `assets/` folder beside the
//! document (the Typora/Obsidian convention) and inserts a markdown image
//! reference at the caret as ONE undoable edit. The PURE pieces — the next free
//! filename, the RGBA→PNG encode, and the save-location resolution live here so
//! they are testable without a real clipboard or disk; the LIVE glue reads
//! arboard and persists through [`persist_png`]; the returned reference
//! re-enters the shared action core as a continuation.
//!
//! NAMING: the filename stem comes from the DOCUMENT, not a counter — a paste
//! into `trip-notes.md` writes `trip-notes-1.png`, so an assets folder full of
//! pasted images reads like the notes they illustrate instead of an opaque run
//! of `pasted-N.png`. [`paste_stem`] derives the stem (sanitized via
//! [`sanitize_stem`]) and [`next_pasted_name`] probes for the first free `N`
//! under it — unrelated stems (an old `pasted-` run alongside a new
//! `trip-notes-` one) probe independently in the same directory, since each
//! only ever matches its OWN prefix.
//!
//! DETERMINISM: nothing here reads a clock or randomness — the unique filename
//! is derived by PROBING the assets dir (`<stem>-1.png`, `<stem>-2.png`, …), a
//! pure function of (stem, directory listing). The whole feature is gated off
//! the headless capture (the OS clipboard image path never runs under
//! `--screenshot` / `--keys`), so a default capture stays byte-identical.
//!
//! NO-PATH BUFFER (settled): a path-less buffer has no directory to hang
//! `assets/` off of, so `App::paste_image_reference` triggers the notes system's OWN
//! auto-name save FIRST (`App::ensure_note_named_before_paste`, `app/files/`)
//! before ever reaching [`assets_dir`]/[`image_ref`]/[`paste_stem`] below — the
//! paste lands beside a real, notes-root file (and inherits ITS stem) rather
//! than this module's absolute data-root fallback whenever that save succeeds.
//! Their `None` arms remain exactly as the LAST-RESORT fallback for the one
//! case that save can't name: a truly EMPTY buffer with no first line to derive
//! a filename from — [`paste_stem`] falls back to the literal `pasted` stem
//! there, so that one case keeps today's `pasted-N.png` names unchanged.

use std::path::{Path, PathBuf};

/// The stem used when no document stem is available: a truly empty, path-less
/// buffer (the one case [`App::ensure_note_named_before_paste`] can't rescue).
/// Keeps today's `pasted-<N>.png` naming for that last-resort case.
const FALLBACK_STEM: &str = "pasted";

/// Unicode-scalar cap on a sanitized stem. Kept in CHARS (not bytes) so
/// truncation always lands on a scalar boundary — never splitting a multi-byte
/// UTF-8 encoding — while still leaving comfortable headroom under the
/// 255-BYTE `NAME_MAX` class of limits (ext4, APFS) shared by the platforms
/// awl targets: even an all-CJK stem (3 bytes/char in UTF-8) at this cap plus
/// the `-<N>.png` suffix stays well under 255 bytes.
const MAX_STEM_CHARS: usize = 80;

/// Sanitize a raw document stem (e.g. a file's `file_stem()`) into a name safe
/// to use BOTH as a filesystem leaf name and, unescaped, as the destination
/// inside a bare `![](assets/<name>.png)` markdown image reference (the exact
/// shape [`crate::actions::image_reference_text`] writes — never wrapped in
/// `<angle brackets>`). The recorded rule, and why:
///
/// - **Path separators** (`/`, `\`) would change which directory the
///   reference names, or fail outright as a filename — replaced.
/// - **Whitespace** breaks [`crate::markdown::parse_image_source`]'s own
///   destination parsing, which takes only the token before the first
///   whitespace run (`(assets/my notes-1.png)` would resolve to
///   `assets/my`) — replaced.
/// - **Parens** (`(`/`)`) terminate a bare markdown destination early — the
///   parser stops at the first unescaped `)` — so a filename containing one
///   would truncate or corrupt the reference — replaced.
/// - Any other ASCII punctuation that is special to shells, URLs, or
///   filesystem path syntax (`<>:"|?*#%&[]`) and all control characters are
///   replaced too, for the same "unescaped destination" reason.
/// - A run of replaced characters collapses to a SINGLE `-` (never dropped
///   outright), so `"trip notes (v2)"` reads as `"trip-notes-v2"` rather than
///   losing the word boundary.
/// - **Dots** are kept internally (`"v1.2.3"` stays `"v1.2.3"`) — only a
///   LEADING or TRAILING dot is trimmed after truncation, so a dotfile-style
///   document stem (`".config"`) can't produce a hidden pasted image.
/// - **Non-ASCII is KEPT, never transliterated** — CJK and other scripts are
///   `char::is_alphanumeric` and pass through unchanged (`"旅行笔记"` stays
///   `"旅行笔记"`); every platform awl targets (APFS, ext4, and the browser's
///   OPFS) accepts UTF-8 leaf names natively, so there is no real breakage to
///   transliterate away. Revisit only if a live report says otherwise.
/// - **Length is capped** at [`MAX_STEM_CHARS`] (see its own doc for the
///   byte-budget reasoning) so an absurdly long document title can't produce
///   a filename a real filesystem rejects.
/// - An empty result (the raw stem was ENTIRELY separators/whitespace/dots)
///   falls back to [`FALLBACK_STEM`], same as no doc-derived stem at all.
///
/// Pure and total — never panics, never touches the filesystem.
pub fn sanitize_stem(raw: &str) -> String {
    let mut out = String::new();
    let mut last_was_dash = false;
    for ch in raw.chars() {
        let keep = ch == '_' || ch == '.' || ch.is_alphanumeric();
        if keep {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    let capped: String = out.chars().take(MAX_STEM_CHARS).collect();
    let trimmed = capped.trim_matches(|c: char| c == '-' || c == '.');
    if trimmed.is_empty() {
        FALLBACK_STEM.to_string()
    } else {
        trimmed.to_string()
    }
}

/// The STEM a pasted image's filename is derived from: the sanitized document
/// file-stem when `doc_path` names a file (`"trip-notes.md"` → `"trip-notes"`),
/// or [`FALLBACK_STEM`] when there is none — the truly-empty-buffer case (see
/// the module doc's NO-PATH BUFFER note). A non-UTF-8 file name (not
/// reachable through awl's own save path, but not a panic either) falls back
/// the same way. Pure — no filesystem access.
pub fn paste_stem(doc_path: Option<&Path>) -> String {
    match doc_path.and_then(Path::file_stem).and_then(|s| s.to_str()) {
        Some(raw) if !raw.is_empty() => sanitize_stem(raw),
        _ => FALLBACK_STEM.to_string(),
    }
}

/// The next free `<stem>-<N>.png` name given the leaf names ALREADY in the
/// assets directory — the smallest `N >= 1` whose `<stem>-N.png` is not
/// present. Pure over (stem, listing) — no clock / no random — so the same
/// inputs always yield the same name: `("pasted", ["pasted-1.png"]) →
/// "pasted-2.png"`, `("pasted", []) → "pasted-1.png"`, gaps are filled
/// (`("pasted", ["pasted-2.png"]) → "pasted-1.png"`). Different stems probe
/// INDEPENDENTLY against the same listing — `"pasted-2.png"` never blocks
/// `"trip-notes-1.png"` — since a candidate only ever matches its own stem's
/// exact prefix.
pub fn next_pasted_name(stem: &str, existing: &[String]) -> String {
    let mut n: usize = 1;
    loop {
        let candidate = format!("{stem}-{n}.png");
        if !existing.iter().any(|name| name == &candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Encode raw RGBA8 pixels (row-major, 4 bytes/pixel, the shape arboard's
/// `ImageData` hands back) into PNG file bytes. `None` — never a panic — when the
/// dimensions are degenerate (either zero) or the buffer length disagrees with
/// `width * height * 4`, so a malformed clipboard image falls back to the normal
/// paste rather than crash. Uses the bundled `image` crate's PNG encoder (the
/// only codec feature enabled).
pub fn encode_rgba_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};

    if width == 0 || height == 0 {
        return None;
    }
    let expected = width.checked_mul(height)?.checked_mul(4)?;
    if rgba.len() != expected {
        return None;
    }
    let (w, h) = (u32::try_from(width).ok()?, u32::try_from(height).ok()?);
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(rgba, w, h, ExtendedColorType::Rgba8)
        .ok()?;
    Some(out)
}

/// Where a pasted image is SAVED for a document at `doc_path` — the `assets/`
/// folder beside the doc, or (for a no-path scratch buffer) `assets/` under the
/// passed `data_root`. Pure: the caller supplies `data_root`
/// (`crate::fs::data_root()` live) so this needs no environment.
pub fn assets_dir(doc_path: Option<&Path>, data_root: &Path) -> PathBuf {
    match doc_path.and_then(Path::parent) {
        Some(dir) => dir.join("assets"),
        None => data_root.join("assets"),
    }
}

/// The markdown image REFERENCE path for `filename` (already just a leaf name):
/// DOC-RELATIVE `assets/<name>` when the doc has a path (portable — it resolves
/// beside the file), or the ABSOLUTE `<data_root>/assets/<name>` for a no-path
/// scratch buffer (which has no directory to be relative to yet, so the absolute
/// path keeps the image resolving until the doc is saved somewhere).
pub fn image_ref(doc_path: Option<&Path>, data_root: &Path, filename: &str) -> String {
    match doc_path {
        Some(_) => format!("assets/{filename}"),
        None => assets_dir(None, data_root)
            .join(filename)
            .to_string_lossy()
            .into_owned(),
    }
}

/// Persist an already-encoded clipboard image and return the reference that a
/// shared core continuation should insert. No buffer mutation happens here.
pub fn persist_png(doc_path: Option<&Path>, data_root: &Path, png: &[u8]) -> Option<String> {
    let fs = crate::fs::active();
    let dir = assets_dir(doc_path, data_root);
    fs.create_dir_all(&dir).ok()?;
    let existing: Vec<String> = fs
        .read_dir(&dir)
        .map(|entries| entries.into_iter().map(|entry| entry.name).collect())
        .unwrap_or_default();
    let stem = paste_stem(doc_path);
    let filename = next_pasted_name(&stem, &existing);
    crate::fs::write_atomic(&dir.join(&filename), png).ok()?;
    Some(image_ref(doc_path, data_root, &filename))
}

#[cfg(test)]
#[path = "paste_image_tests.rs"]
mod tests;
