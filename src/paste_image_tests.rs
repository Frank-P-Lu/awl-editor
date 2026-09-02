use super::*;

#[test]
fn next_named_asset_probes_deterministically_at_the_clipboard_paste_extension() {
    // Empty dir → the first name.
    assert_eq!(next_named_asset("pasted", "png", &[]), "pasted-1.png");
    // One taken → the next.
    assert_eq!(
        next_named_asset("pasted", "png", &["pasted-1.png".to_string()]),
        "pasted-2.png"
    );
    // A run taken → the first free above it.
    assert_eq!(
        next_named_asset(
            "pasted",
            "png",
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
        next_named_asset(
            "pasted",
            "png",
            &["pasted-2.png".to_string(), "pasted-3.png".to_string()]
        ),
        "pasted-1.png"
    );
    // Unrelated files are ignored.
    assert_eq!(
        next_named_asset(
            "pasted",
            "png",
            &["notes.md".to_string(), "pasted-1.png".to_string()]
        ),
        "pasted-2.png"
    );
}

#[test]
fn next_named_asset_uses_the_given_stem() {
    // A doc-derived stem probes its OWN `<stem>-N.png` sequence.
    assert_eq!(
        next_named_asset("trip-notes", "png", &[]),
        "trip-notes-1.png"
    );
    assert_eq!(
        next_named_asset("trip-notes", "png", &["trip-notes-1.png".to_string()]),
        "trip-notes-2.png"
    );
}

#[test]
fn next_named_asset_probes_two_stems_independently_in_the_same_listing() {
    // The OLD `pasted-` run and a NEW doc-derived `trip-notes-` run share
    // one directory listing but never collide: each stem only matches
    // candidates carrying its own exact prefix.
    let existing = vec![
        "pasted-1.png".to_string(),
        "pasted-2.png".to_string(),
        "trip-notes-1.png".to_string(),
    ];
    assert_eq!(next_named_asset("pasted", "png", &existing), "pasted-3.png");
    assert_eq!(
        next_named_asset("trip-notes", "png", &existing),
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

// ── extension-preserving naming (dropped images reuse this owner too) ─────

#[test]
fn next_named_asset_probes_extensions_independently_under_the_same_stem() {
    assert_eq!(
        next_named_asset("trip-notes", "jpg", &[]),
        "trip-notes-1.jpg"
    );
    assert_eq!(
        next_named_asset(
            "trip-notes",
            "jpg",
            &[
                "trip-notes-1.jpg".to_string(),
                "trip-notes-1.png".to_string()
            ]
        ),
        "trip-notes-2.jpg",
        "a same-stem different-extension file does not block this extension's own run"
    );
}

#[test]
fn persist_bytes_preserves_the_dropped_extension_and_probes_independently_of_a_png_run() {
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let _guard = crate::testlock::serial();
    let mem = crate::fs::InMemoryFs::new().with_dir("/notes/assets");
    mem.write(Path::new("/notes/assets/trip-notes-1.png"), b"pasted")
        .unwrap();
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let reference = persist_bytes(
            Some(Path::new("/notes/trip-notes.md")),
            Path::new("/data"),
            b"jpeg-bytes",
            "jpg",
        );
        // Same stem as the existing pasted PNG, but its own `.jpg` run — the
        // existing `-1.png` never blocks `-1.jpg`.
        assert_eq!(reference.as_deref(), Some("assets/trip-notes-1.jpg"));
        assert_eq!(
            mem.read(Path::new("/notes/assets/trip-notes-1.jpg"))
                .unwrap(),
            b"jpeg-bytes",
            "the dropped file's own bytes are copied verbatim, never re-encoded"
        );
        assert_eq!(
            mem.read(Path::new("/notes/assets/trip-notes-1.png"))
                .unwrap(),
            b"pasted",
            "the pre-existing pasted-N.png is untouched"
        );
    });
}
