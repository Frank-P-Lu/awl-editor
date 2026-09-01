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
mod tests {
    use super::*;

    #[test]
    fn next_pasted_name_probes_deterministically() {
        // Empty dir → the first name.
        assert_eq!(next_pasted_name("pasted", &[]), "pasted-1.png");
        // One taken → the next.
        assert_eq!(
            next_pasted_name("pasted", &["pasted-1.png".to_string()]),
            "pasted-2.png"
        );
        // A run taken → the first free above it.
        assert_eq!(
            next_pasted_name(
                "pasted",
                &[
                    "pasted-1.png".to_string(),
                    "pasted-2.png".to_string(),
                    "pasted-3.png".to_string(),
                ]
            ),
            "pasted-4.png"
        );
        // A GAP is filled, not skipped (probes from 1 up).
        assert_eq!(
            next_pasted_name(
                "pasted",
                &["pasted-2.png".to_string(), "pasted-3.png".to_string()]
            ),
            "pasted-1.png"
        );
        // Unrelated files are ignored.
        assert_eq!(
            next_pasted_name(
                "pasted",
                &["notes.md".to_string(), "pasted-1.png".to_string()]
            ),
            "pasted-2.png"
        );
    }

    #[test]
    fn next_pasted_name_uses_the_given_stem() {
        // A doc-derived stem probes its OWN `<stem>-N.png` sequence.
        assert_eq!(next_pasted_name("trip-notes", &[]), "trip-notes-1.png");
        assert_eq!(
            next_pasted_name("trip-notes", &["trip-notes-1.png".to_string()]),
            "trip-notes-2.png"
        );
    }

    #[test]
    fn next_pasted_name_probes_two_stems_independently_in_the_same_listing() {
        // The OLD `pasted-` run and a NEW doc-derived `trip-notes-` run share
        // one directory listing but never collide: each stem only matches
        // candidates carrying its own exact prefix.
        let existing = vec![
            "pasted-1.png".to_string(),
            "pasted-2.png".to_string(),
            "trip-notes-1.png".to_string(),
        ];
        assert_eq!(next_pasted_name("pasted", &existing), "pasted-3.png");
        assert_eq!(
            next_pasted_name("trip-notes", &existing),
            "trip-notes-2.png"
        );
    }

    #[test]
    fn sanitize_stem_replaces_separators_whitespace_and_parens() {
        // Path separators would change what directory a leaf name means.
        assert_eq!(sanitize_stem("a/b\\c"), "a-b-c");
        // Whitespace breaks `parse_image_source`'s destination-token split.
        assert_eq!(sanitize_stem("trip notes"), "trip-notes");
        // A run of unsafe chars collapses to ONE `-`, not one per char.
        assert_eq!(sanitize_stem("trip   notes"), "trip-notes");
        // Parens terminate a bare markdown destination early.
        assert_eq!(sanitize_stem("resume (v2)"), "resume-v2");
    }

    #[test]
    fn sanitize_stem_keeps_internal_dots_but_trims_leading_and_trailing() {
        // Internal dots survive (a version-like stem stays readable).
        assert_eq!(sanitize_stem("v1.2.3"), "v1.2.3");
        // A dotfile-style stem doesn't produce a hidden pasted image.
        assert_eq!(sanitize_stem(".config"), "config");
        // Trailing dot trimmed too (would otherwise abut "-1.png" oddly).
        assert_eq!(sanitize_stem("notes."), "notes");
    }

    #[test]
    fn sanitize_stem_keeps_cjk_and_other_non_ascii_untransliterated() {
        assert_eq!(sanitize_stem("旅行笔记"), "旅行笔记");
        // Mixed script + ASCII, with a separator between.
        assert_eq!(sanitize_stem("café notes"), "café-notes");
    }

    #[test]
    fn sanitize_stem_caps_absurdly_long_names_on_a_char_boundary() {
        // 200 ASCII chars: capped to MAX_STEM_CHARS, no panic, no partial byte.
        let long = "a".repeat(200);
        let stem = sanitize_stem(&long);
        assert_eq!(stem.chars().count(), MAX_STEM_CHARS);
        assert!(stem.chars().all(|c| c == 'a'));

        // 200 CJK chars (3 bytes each in UTF-8): same char cap, and the
        // truncation lands on a scalar boundary (the string is valid UTF-8
        // by construction since `chars().take(..)` only ever collects whole
        // scalars).
        let long_cjk = "旅".repeat(200);
        let stem_cjk = sanitize_stem(&long_cjk);
        assert_eq!(stem_cjk.chars().count(), MAX_STEM_CHARS);
    }

    #[test]
    fn sanitize_stem_falls_back_when_nothing_survives() {
        // Entirely separators/whitespace/dots → nothing left after trimming.
        assert_eq!(sanitize_stem("///   ..."), "pasted");
        assert_eq!(sanitize_stem(""), "pasted");
    }

    #[test]
    fn paste_stem_derives_from_the_doc_file_stem() {
        assert_eq!(
            paste_stem(Some(Path::new("/notes/trip-notes.md"))),
            "trip-notes"
        );
        // The sanitization rule applies to the derived stem too.
        assert_eq!(
            paste_stem(Some(Path::new("/notes/trip notes (draft).md"))),
            "trip-notes-draft"
        );
    }

    #[test]
    fn paste_stem_falls_back_to_pasted_with_no_doc_path() {
        // The one case `ensure_note_named_before_paste` can't rescue: a truly
        // empty, path-less buffer. Keeps today's `pasted-N.png` naming.
        assert_eq!(paste_stem(None), "pasted");
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
            // The document is `trip-notes.md` — the persisted name and the
            // returned reference are both derived from ITS stem, not a bare
            // counter.
            let reference = persist_png(
                Some(Path::new("/notes/trip-notes.md")),
                Path::new("/data"),
                b"png",
            );
            assert_eq!(reference.as_deref(), Some("assets/trip-notes-1.png"));
            assert_eq!(
                mem.read(Path::new("/notes/assets/trip-notes-1.png"))
                    .unwrap(),
                b"png"
            );
            assert!(
                !mem.exists(Path::new("/notes/trip-notes.md")),
                "the external image transaction never edits or saves the document"
            );
        });
    }

    #[test]
    fn persist_png_probes_the_doc_stem_independently_of_an_existing_pasted_run() {
        use crate::fs::FileSystem;
        use std::sync::Arc;

        let _guard = crate::testlock::serial();
        // An OLD `pasted-1.png` already lives in assets/ from before this
        // feature; a fresh paste into `trip-notes.md` must land at
        // `trip-notes-1.png`, not skip ahead because of the unrelated stem.
        let mem = crate::fs::InMemoryFs::new().with_dir("/notes/assets");
        mem.write(Path::new("/notes/assets/pasted-1.png"), b"old")
            .unwrap();
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let reference = persist_png(
                Some(Path::new("/notes/trip-notes.md")),
                Path::new("/data"),
                b"new",
            );
            assert_eq!(reference.as_deref(), Some("assets/trip-notes-1.png"));
            assert_eq!(
                mem.read(Path::new("/notes/assets/pasted-1.png")).unwrap(),
                b"old",
                "the pre-existing pasted-N.png is untouched"
            );
        });
    }

    #[test]
    fn persist_png_falls_back_to_pasted_for_a_path_less_buffer() {
        use std::sync::Arc;

        let _guard = crate::testlock::serial();
        let mem = crate::fs::InMemoryFs::new().with_dir("/data");
        crate::fs::with_fs(Arc::new(mem.clone()), || {
            let reference = persist_png(None, Path::new("/data"), b"png");
            let reference = reference.expect("a path-less buffer still persists via data_root");
            assert!(
                reference.ends_with("assets/pasted-1.png"),
                "no doc stem available → today's pasted-N.png naming: {reference:?}"
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
