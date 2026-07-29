//! PASTE-IMAGE (native, LIVE-App-only): when the OS clipboard holds an IMAGE
//! rather than text, awl saves it as a PNG into an `assets/` folder beside the
//! document (the Typora/Obsidian convention) and inserts a markdown image
//! reference at the caret as ONE undoable edit. The PURE pieces — the next free
//! filename, the RGBA→PNG encode, and the save-location resolution live here so
//! they are testable without a real clipboard or disk; the LIVE glue reads
//! arboard and persists through [`persist_png`]; the returned reference
//! re-enters the shared action core as a continuation.
//!
//! DETERMINISM: nothing here reads a clock or randomness — the unique filename
//! is derived by PROBING the assets dir (`pasted-1.png`, `pasted-2.png`, …), a
//! pure function of the directory listing. The whole feature is gated off the
//! headless capture (the OS clipboard image path never runs under `--screenshot`
//! / `--keys`), so a default capture stays byte-identical.
//!
//! NO-PATH BUFFER (settled): a path-less buffer has no directory to hang
//! `assets/` off of, so `App::paste_image_reference` triggers the notes system's OWN
//! auto-name save FIRST (`App::ensure_note_named_before_paste`, `app/files/`)
//! before ever reaching [`assets_dir`]/[`image_ref`] below — the paste lands
//! beside a real, notes-root file rather than this module's absolute data-root
//! fallback whenever that save succeeds. [`assets_dir`]'s/[`image_ref`]'s `None`
//! arms remain exactly as the LAST-RESORT fallback for the one case that save
//! can't name: a truly EMPTY buffer with no first line to derive a filename from.

use std::path::{Path, PathBuf};

/// The stem every pasted image name carries: `pasted-<N>.png`.
const PASTED_STEM: &str = "pasted-";

/// The next free `pasted-<N>.png` name given the leaf names ALREADY in the assets
/// directory — the smallest `N >= 1` whose `pasted-N.png` is not present. Pure
/// over the listing (no clock / no random), so the same directory state always
/// yields the same name: `["pasted-1.png"] → "pasted-2.png"`, `[] → "pasted-1.png"`,
/// gaps are filled (`["pasted-2.png"] → "pasted-1.png"`).
pub fn next_pasted_name(existing: &[String]) -> String {
    let mut n: usize = 1;
    loop {
        let candidate = format!("{PASTED_STEM}{n}.png");
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
    let filename = next_pasted_name(&existing);
    crate::fs::write_atomic(&dir.join(&filename), png).ok()?;
    Some(image_ref(doc_path, data_root, &filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_pasted_name_probes_deterministically() {
        // Empty dir → the first name.
        assert_eq!(next_pasted_name(&[]), "pasted-1.png");
        // One taken → the next.
        assert_eq!(
            next_pasted_name(&["pasted-1.png".to_string()]),
            "pasted-2.png"
        );
        // A run taken → the first free above it.
        assert_eq!(
            next_pasted_name(&[
                "pasted-1.png".to_string(),
                "pasted-2.png".to_string(),
                "pasted-3.png".to_string(),
            ]),
            "pasted-4.png"
        );
        // A GAP is filled, not skipped (probes from 1 up).
        assert_eq!(
            next_pasted_name(&["pasted-2.png".to_string(), "pasted-3.png".to_string()]),
            "pasted-1.png"
        );
        // Unrelated files are ignored.
        assert_eq!(
            next_pasted_name(&["notes.md".to_string(), "pasted-1.png".to_string()]),
            "pasted-2.png"
        );
    }

    #[test]
    fn encode_rgba_png_makes_valid_png_bytes() {
        // A 2x1 RGBA image: one red pixel, one green pixel.
        let rgba = [255u8, 0, 0, 255, 0, 255, 0, 255];
        let png = encode_rgba_png(2, 1, &rgba).expect("valid RGBA encodes");
        // The 8-byte PNG signature.
        assert_eq!(
            &png[..8],
            &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']
        );
        // Round-trips back to the same dimensions through the decoder.
        let decoded = image::load_from_memory(&png).expect("re-decodes");
        assert_eq!((decoded.width(), decoded.height()), (2, 1));
    }

    #[test]
    fn encode_rgba_png_rejects_bad_input_without_panic() {
        // Length mismatch (needs 8 bytes for 2x1) → None, not a panic.
        assert!(encode_rgba_png(2, 1, &[0u8; 4]).is_none());
        // Zero dimension → None.
        assert!(encode_rgba_png(0, 4, &[]).is_none());
        assert!(encode_rgba_png(4, 0, &[]).is_none());
    }

    #[test]
    fn assets_dir_resolves_doc_relative_vs_scratch_data_dir() {
        let data_root = Path::new("/home/u/.local/share/awl");
        // Doc HAS a path → assets/ beside the doc.
        let doc = PathBuf::from("/home/u/notes/journal.md");
        assert_eq!(
            assets_dir(Some(&doc), data_root),
            PathBuf::from("/home/u/notes/assets")
        );
        // No path (scratch) → assets/ under the data root.
        assert_eq!(
            assets_dir(None, data_root),
            PathBuf::from("/home/u/.local/share/awl/assets")
        );
    }

    #[test]
    fn image_ref_is_relative_for_a_doc_and_absolute_for_scratch() {
        let data_root = Path::new("/home/u/.local/share/awl");
        let doc = PathBuf::from("/home/u/notes/journal.md");
        // Doc-relative — portable beside the file.
        assert_eq!(
            image_ref(Some(&doc), data_root, "pasted-1.png"),
            "assets/pasted-1.png"
        );
        // Scratch → absolute, so it resolves before the doc is saved anywhere.
        assert_eq!(
            image_ref(None, data_root, "pasted-1.png"),
            "/home/u/.local/share/awl/assets/pasted-1.png"
        );
    }

    #[test]
    fn persist_png_writes_only_the_image_and_returns_a_core_continuation_value() {
        use crate::fs::FileSystem;
        use std::sync::Arc;

        let _guard = crate::testlock::serial();
        let mem = crate::fs::InMemoryFs::new().with_dir("/notes");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let reference = persist_png(Some(Path::new("/notes/a.md")), Path::new("/data"), b"png");
            assert_eq!(reference.as_deref(), Some("assets/pasted-1.png"));
            assert_eq!(
                mem.read(Path::new("/notes/assets/pasted-1.png")).unwrap(),
                b"png"
            );
            assert!(
                !mem.exists(Path::new("/notes/a.md")),
                "the external image transaction never edits or saves the document"
            );
        });
    }

    #[test]
    fn persist_png_failure_returns_text_fallback_without_a_reference() {
        use std::sync::Arc;

        let _guard = crate::testlock::serial();
        crate::fs::with_fs(Arc::new(crate::fs::UnwritableFs), || {
            assert_eq!(
                persist_png(Some(Path::new("/notes/a.md")), Path::new("/data"), b"png"),
                None,
                "a failed external write must select the text-yank fallback"
            );
        });
    }

    /// The insert lands as ONE undoable edit through the real buffer seam
    /// (`replace_char_range`, the exact core continuation call makes): the
    /// ref text appears, and a single Cmd-Z (`undo`) restores the prior text +
    /// cursor. The live clipboard read is live-only; this proves the insert half.
    #[test]
    fn inserted_ref_is_one_undoable_edit_over_the_real_buffer() {
        use crate::buffer::Buffer;
        // Caret at end of an existing prose line (mid-line → leading newline).
        let mut b = Buffer::from_str("hello");
        b.set_cursor(5);
        let reference = image_ref(None, Path::new("/data"), "pasted-1.png");
        let text = crate::actions::image_reference_text(false, &reference);
        let at = b.cursor_char();
        b.replace_char_range(at, at, &text);
        assert_eq!(b.text(), "hello\n![](/data/assets/pasted-1.png)\n");
        // ONE undo restores exactly the prior text.
        b.undo();
        assert_eq!(b.text(), "hello");
        assert_eq!(b.cursor_char(), 5);
    }

    /// LOCKS the "no `|W`" contract end to end: a pasted ref — even for a huge
    /// retina-native-pixel image — parses back with `width_hint: None` through the
    /// SAME `markdown::parse_image_source` the renderer reads, so display sizing
    /// falls back to fit-to-column (never the raw native pixel width). A `|W` hint
    /// is reserved for the drag-resize write-back, never paste.
    #[test]
    fn pasted_ref_never_stamps_a_width() {
        // A retina screenshot's native width, the exact shape this round guards
        // against (`![|2241](assets/pasted-3.png)` was the reported bug).
        let reference = "assets/pasted-3.png";
        let text = crate::actions::image_reference_text(true, reference);
        assert_eq!(text, "![](assets/pasted-3.png)\n");
        assert!(
            !text.contains('|'),
            "no width hint delimiter anywhere in the inserted ref: {text:?}"
        );
        let src = text.trim_end_matches('\n');
        let parsed = crate::markdown::parse_image_source(src).expect("a well-formed image ref");
        assert_eq!(parsed.width_hint, None, "paste never stamps a width hint");
        assert_eq!(parsed.path, reference);
    }

    #[test]
    fn insert_text_puts_the_ref_on_its_own_line() {
        // At line start: no leading newline, trailing newline for the fresh line.
        assert_eq!(
            crate::actions::image_reference_text(true, "assets/pasted-1.png"),
            "![](assets/pasted-1.png)\n"
        );
        // Mid-line: a leading newline pushes the ref onto its own line.
        assert_eq!(
            crate::actions::image_reference_text(false, "assets/pasted-1.png"),
            "\n![](assets/pasted-1.png)\n"
        );
    }
}
