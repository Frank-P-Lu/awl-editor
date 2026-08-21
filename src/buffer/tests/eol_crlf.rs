use super::*;

// --- LINE ENDINGS (VS Code model): normalize-on-load, restore-on-save ------
//
// RESOLVED (was the CRLF / lone-CR / U+2028 divergence). ropey now counts
// LF-ONLY — its `unicode_lines`/`cr_lines` features are OFF (Cargo.toml), so
// `len_lines`/`char_to_line`/`line_to_char` recognize a break at '\n' and
// NOWHERE else. `Buffer::from_file` NORMALIZES every '\r\n' to '\n' before the
// text enters the rope while remembering the file's `Eol`, and a save restores
// it ([`Buffer::disk_bytes`]). Two consequences, both proven below:
//   (a) the buffer is purely '\n'-based, so it AGREES with the '\n'-only
//       renderer by construction — no CRLF/lone-CR line-model divergence;
//   (b) a lone '\r' / NEL / LS / PS is ordinary CONTENT, never a line break.
// A CRLF file therefore round-trips byte-for-byte; a lone '\r' is preserved
// verbatim inside its line. (`from_str` — a raw, un-normalizing constructor —
// is characterized separately: a '\r' forced in that way is now CONTENT too,
// since counting is LF-only.)

#[test]
fn eol_detect_picks_the_dominant_ending() {
    assert_eq!(Eol::detect(""), Eol::Lf, "empty file → LF default");
    assert_eq!(Eol::detect("no newline at all"), Eol::Lf);
    assert_eq!(Eol::detect("a\nb\nc\n"), Eol::Lf);
    assert_eq!(Eol::detect("a\r\nb\r\nc\r\n"), Eol::Crlf);
    // MIXED: the MAJORITY ending wins. 3 CRLF vs 1 lone LF → CRLF.
    assert_eq!(Eol::detect("a\r\nb\r\nc\r\nd\ne"), Eol::Crlf);
    // MIXED: 1 CRLF vs 2 lone LF → LF (CRLF is the minority).
    assert_eq!(Eol::detect("a\r\nb\nc\nd"), Eol::Lf);
    // A TIE falls to LF, the conservative default.
    assert_eq!(Eol::detect("a\r\nb\n"), Eol::Lf);
    // A lone '\r' is NOT a '\r\n' pair — it never counts toward CRLF.
    assert_eq!(Eol::detect("a\rb\nc\n"), Eol::Lf);
}

#[test]
fn raw_crlf_via_from_str_counts_lf_only_cr_is_content() {
    // `from_str` does NOT normalize (only `from_file` does), so it can force a
    // '\r' into the rope. LF-only counting means that '\r' is CONTENT, not a
    // break: "abc\r\ndef\r\n" is 3 lines (the two '\n'), and line 0 is "abc\r".
    let mut buf = b("abc\r\ndef\r\n");
    assert_eq!(buf.line_count(), 3, "LF-only: the two '\\n' make 3 lines");

    // C-e on line 0 runs to just before the '\n' (past the content '\r').
    buf.line_end_motion();
    assert_eq!(buf.cursor_char(), 4);
    assert_eq!(buf.cursor_line_col(), (0, 4));

    // Typing there does NOT create a new line — the '\r' is inert content, so
    // the count stays 3 (the pre-fix model wrongly made the orphaned CR a
    // break and reported 4). This is the resolved divergence.
    buf.insert_char('X');
    assert_eq!(buf.text(), "abc\rX\ndef\r\n");
    assert_eq!(buf.line_count(), 3, "the content '\\r' is never a break");
}

#[test]
fn lf_file_loads_and_saves_byte_identical() {
    // REGRESSION: a Unix-ended file is unchanged in every respect — detected
    // LF, rope byte-for-byte, and re-saved byte-for-byte (the pre-round path).
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let path = std::path::PathBuf::from("/docs/unix.md");
    let raw = "alpha\nbeta\ngamma\n";
    let mem = crate::fs::InMemoryFs::new().with_file(&path, raw);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(buf.eol(), Eol::Lf);
        assert_eq!(buf.text(), raw, "LF rope is byte-identical");
        buf.save().unwrap();
        assert_eq!(
            mem.read(&path).unwrap(),
            raw.as_bytes(),
            "LF save unchanged"
        );
    });
}

#[test]
fn crlf_file_normalizes_on_load_and_round_trips_byte_for_byte() {
    // The headline: a Windows-ended file loads with a PURELY '\n' rope (no CR
    // survives) tagged `Eol::Crlf`, and a save restores '\r\n' so the on-disk
    // bytes are IDENTICAL to what was loaded.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let path = std::path::PathBuf::from("/docs/win.md");
    let raw = "# Title\r\nline two\r\nline three\r\n";
    let mem = crate::fs::InMemoryFs::new().with_file(&path, raw);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(buf.eol(), Eol::Crlf, "detected CRLF");
        assert_eq!(
            buf.text(),
            "# Title\nline two\nline three\n",
            "rope is LF-only"
        );
        assert!(!buf.text().contains('\r'), "no CR ever enters the rope");
        // Buffer and the '\n'-only renderer now agree: 4 lines (trailing '\n').
        assert_eq!(buf.line_count(), 4);
        buf.save().unwrap();
        assert_eq!(
            mem.read(&path).unwrap(),
            raw.as_bytes(),
            "CRLF round-trips byte-for-byte"
        );
    });
}

#[test]
fn setext_headings_round_trip_byte_for_byte() {
    // The file is plain text and setext is valid CommonMark — a setext heading
    // not reading as a heading (nor growing its underline's row) changes what awl
    // DRAWS, never what it STORES. Load a document mixing an ATX heading, a
    // setext heading (`===` underline), a real thematic break, and a
    // setext-underline that's a bare `---` (the dash form is the one
    // `is_thematic_break`'s documented false positive can confuse with a real
    // break) — then a no-edit save must reproduce the EXACT original bytes.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let path = std::path::PathBuf::from("/docs/setext.md");
    let raw = "# ATX Heading\n\nSetext Title\n------------\n\nAnother Setext\n\
               ==============\n\n---\n\nbody\n";
    let mem = crate::fs::InMemoryFs::new().with_file(&path, raw);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(
            buf.disk_bytes(),
            raw.as_bytes(),
            "load produces byte-identical disk_bytes with no edits"
        );
        buf.save().unwrap();
        assert_eq!(
            mem.read(&path).unwrap(),
            raw.as_bytes(),
            "a no-edit save of a setext-bearing document reproduces the exact original bytes"
        );
    });
}

#[test]
fn mixed_eol_file_picks_dominant_and_normalizes_all_lines() {
    // 3 CRLF vs 1 lone LF → dominant CRLF. On load, EVERY ending (the lone LF
    // included) becomes '\n' in the rope; on save, EVERY line is re-emitted
    // CRLF — a VS Code-style normalize (so it deliberately does NOT preserve
    // the original minority '\n').
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let path = std::path::PathBuf::from("/docs/mixed.md");
    let raw = "a\r\nb\r\nc\r\nd\ne";
    let mem = crate::fs::InMemoryFs::new().with_file(&path, raw);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(buf.eol(), Eol::Crlf, "CRLF is the majority");
        assert_eq!(buf.text(), "a\nb\nc\nd\ne", "all endings normalized to LF");
        assert!(!buf.text().contains('\r'));
        buf.save().unwrap();
        assert_eq!(
            mem.read(&path).unwrap(),
            b"a\r\nb\r\nc\r\nd\r\ne",
            "save re-emits CRLF uniformly (normalize, not preserve)"
        );
    });
}

#[test]
fn lone_cr_nel_ls_ps_are_content_not_line_breaks() {
    // The documented lone-CR (and NEL / LS / PS) decision: these are CONTENT,
    // never breaks — the buffer now AGREES with the '\n'-only renderer instead
    // of diverging (the pre-round model counted each as its own break).
    for sep in ["\r", "\u{0085}", "\u{2028}", "\u{2029}"] {
        let text = format!("ab{sep}cd");
        let mut buf = b(&text);
        assert_eq!(buf.line_count(), 1, "{sep:?} is content, so one line");
        // C-e runs to the true end of the single 5-char line ("ab_cd"), past
        // the separator (which is inert content).
        buf.set_cursor(0);
        buf.line_end_motion();
        assert_eq!(buf.cursor_line_col(), (0, 5), "{sep:?}: C-e reaches col 5");
        // One C-k takes the WHOLE line, separator included (it's just content).
        let mut buf = b(&text);
        buf.kill_line();
        assert_eq!(buf.text(), "", "{sep:?}: the whole line is one kill");
        assert_eq!(
            buf.kill_buffer(),
            text,
            "{sep:?}: separator killed as content"
        );
    }
}

#[test]
fn lone_cr_file_is_preserved_verbatim_and_round_trips() {
    // A lone '\r' in a LOADED file stays literal content (not a CRLF signal,
    // not a break), and round-trips byte-for-byte through an LF save.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let path = std::path::PathBuf::from("/docs/cr.md");
    let raw = "ab\rcd\nef\n";
    let mem = crate::fs::InMemoryFs::new().with_file(&path, raw);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(buf.eol(), Eol::Lf, "a lone CR never signals CRLF");
        assert_eq!(buf.text(), raw, "the lone CR is preserved verbatim");
        assert_eq!(buf.line_count(), 3, "LF-only: the two '\\n' make 3 lines");
        buf.set_cursor(0);
        buf.line_end_motion();
        assert_eq!(
            buf.cursor_line_col(),
            (0, 5),
            "C-e runs past the content CR"
        );
        buf.save().unwrap();
        assert_eq!(mem.read(&path).unwrap(), raw.as_bytes(), "byte-for-byte");
    });
}

#[test]
fn caret_column_over_former_crlf_matches_the_lf_equivalent() {
    // The SAME document, once CRLF-ended and once LF-ended. After load both
    // ropes are pure '\n', so EVERY caret / motion result is identical — there
    // is no '\r' in the rope for the caret to land "inside".
    use std::sync::Arc;
    let crlf_path = std::path::PathBuf::from("/docs/w.md");
    let lf_path = std::path::PathBuf::from("/docs/u.md");
    let mem = crate::fs::InMemoryFs::new()
        .with_file(&crlf_path, "hello\r\nworld\r\n")
        .with_file(&lf_path, "hello\nworld\n");
    crate::fs::with_fs(Arc::new(mem), || {
        let mut win = Buffer::from_file(&crlf_path);
        let mut nix = Buffer::from_file(&lf_path);
        assert_eq!(win.eol(), Eol::Crlf);
        assert_eq!(nix.eol(), Eol::Lf);
        assert_eq!(win.text(), nix.text(), "identical rope content after load");
        // C-e on line 0: both land at col 5 (end of "hello"), never on a CR.
        win.set_cursor(0);
        nix.set_cursor(0);
        win.line_end_motion();
        nix.line_end_motion();
        assert_eq!(win.cursor_line_col(), (0, 5));
        assert_eq!(win.cursor_line_col(), nix.cursor_line_col());
        assert_eq!(win.cursor_char(), nix.cursor_char(), "same absolute index");
        // Vertical motion onto line 1 also matches exactly.
        win.next_line();
        nix.next_line();
        assert_eq!(win.cursor_line_col(), nix.cursor_line_col());
        assert_eq!(win.cursor_char(), nix.cursor_char());
    });
}

#[test]
fn set_eol_flips_encoding_is_metadata_not_an_undoable_edit() {
    // Switching the ending is a DOCUMENT-LEVEL setting, not a text edit: the
    // rope content is untouched, only `disk_bytes` differs. A real switch bumps
    // `version` + dirties (so the autosave engine rewrites); a no-op switch is
    // inert; and undo does NOT restore the ending (it is off the timeline).
    let mut buf = Buffer::from_str("a\nb\n");
    assert_eq!(buf.eol(), Eol::Lf);
    assert_eq!(buf.disk_bytes(), b"a\nb\n");
    let v0 = buf.version();
    buf.set_eol(Eol::Crlf);
    assert_eq!(buf.eol(), Eol::Crlf);
    assert_eq!(buf.text(), "a\nb\n", "rope content is untouched");
    assert_eq!(buf.disk_bytes(), b"a\r\nb\r\n", "on-disk encoding flipped");
    assert!(buf.is_dirty());
    assert!(buf.version() > v0, "version bumped so autosave picks it up");
    // A no-op switch (same ending) changes nothing.
    let v1 = buf.version();
    buf.set_eol(Eol::Crlf);
    assert_eq!(buf.version(), v1, "same ending: no version bump");

    // Undo reverts the TEXT edit but leaves the EOL where it was set — the
    // ending is metadata, not part of the undo history (documented choice).
    let mut buf2 = Buffer::from_str("x");
    buf2.insert_char('y'); // one undoable text edit
    buf2.set_eol(Eol::Crlf);
    buf2.undo();
    assert_eq!(buf2.text(), "x", "undo reverts the text edit");
    assert_eq!(
        buf2.eol(),
        Eol::Crlf,
        "EOL is metadata — undo does not restore it"
    );
}

#[test]
fn crlf_encode_is_idempotent_never_double_encodes_existing_crlf() {
    // REGRESSION (data loss, commit f953392): `disk_bytes` used to re-encode
    // '\n' -> '\r\n' assuming the rope was pure-'\n'. If a real '\r\n' reached
    // the rope by a door OTHER than `from_file` (a pasted CRLF clipboard, a
    // history restore), an `Eol::Crlf` save DOUBLE-encoded it to '\r\r\n' —
    // byte corruption. `from_str` (raw, un-normalizing) forces that state.
    let mut buf = Buffer::from_str("a\r\nb");
    buf.set_eol(Eol::Crlf);
    assert_eq!(
        buf.disk_bytes(),
        b"a\r\nb",
        "existing \\r\\n encodes to a SINGLE \\r\\n, never \\r\\r\\n"
    );
    // A pure-'\n' rope (the normal invariant) still encodes exactly once.
    let mut buf2 = Buffer::from_str("a\nb\n");
    buf2.set_eol(Eol::Crlf);
    assert_eq!(
        buf2.disk_bytes(),
        b"a\r\nb\r\n",
        "pure-\\n encodes as before"
    );
    // A lone '\r' (no '\n' after it) stays CONTENT, never sprouts a '\n'.
    let mut buf3 = Buffer::from_str("a\rb\nc");
    buf3.set_eol(Eol::Crlf);
    assert_eq!(
        buf3.disk_bytes(),
        b"a\rb\r\nc",
        "lone \\r preserved; only the real break becomes \\r\\n"
    );
}

#[test]
fn pasted_crlf_round_trips_to_a_single_crlf_per_line_never_doubled() {
    // The entry-door half of the fix: an external CRLF clipboard value pasted
    // (set_kill + yank) into a `Crlf` buffer must round-trip to ONE '\r\n' per
    // line on save, never '\r\r\n'. `set_kill` normalizes '\r\n' -> '\n' on the
    // way in (matching `from_file`), so the rope stays purely '\n'.
    let mut buf = Buffer::from_str("");
    buf.set_eol(Eol::Crlf);
    buf.set_kill("x\r\ny"); // an external Windows-clipboard value
    assert_eq!(
        buf.kill_buffer(),
        "x\ny",
        "\\r\\n normalized out of the kill ring"
    );
    buf.yank();
    assert_eq!(buf.text(), "x\ny", "the rope is purely '\\n' after paste");
    assert!(!buf.text().contains('\r'), "no CR ever enters the rope");
    assert_eq!(
        buf.disk_bytes(),
        b"x\r\ny",
        "a single \\r\\n per line on save, never doubled"
    );
    // A lone '\r' in the pasted value is content and survives the normalize.
    let mut buf2 = Buffer::from_str("");
    buf2.set_kill("p\rq");
    assert_eq!(
        buf2.kill_buffer(),
        "p\rq",
        "lone \\r stays content in the kill ring"
    );
}
