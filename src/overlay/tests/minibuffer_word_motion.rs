use super::*;

// ── WORD-OPS ROUND (b): ⌥⌫ word-delete in the minibuffer ────────────────────
// Every overlay input (the fuzzy query + the Rename / Link / Keep / Settings-
// value edits) deletes a WHOLE trailing word on ⌥⌫, routed through the ONE
// document-buffer boundary owner (`buffer::word_delete_backward_boundary`) via
// the shared `TextBox::delete_word_back` — so the palette can never
// disagree with the text about where a word ends. (Plain L/R still drive list
// navigation; WORD motion — Ctrl/Opt-arrow, `ForwardWord`/
// `BackwardWord` — to the query's own caret instead, since plain arrows are
// claimed by lens/descend/list.)

#[test]
fn query_word_delete_removes_a_trailing_word_not_a_char() {
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    for c in "foo bar baz".chars() {
        ov.push(c);
    }
    assert_eq!(ov.query, "foo bar baz");
    ov.pop_word(); // ⌥⌫ removes the trailing word "baz"
    assert_eq!(ov.query, "foo bar ");
    ov.pop_word(); // and its whitespace + the next word
    assert_eq!(ov.query, "foo ");
    ov.pop_word();
    assert_eq!(ov.query, "");
    ov.pop_word(); // NO-OP on an empty query (never panics / underflows)
    assert_eq!(ov.query, "");
    // Plain ⌫ still removes ONE char — the split is real.
    ov.push('a');
    ov.push('b');
    ov.pop();
    assert_eq!(ov.query, "a");
}

#[test]
fn rename_minibuffer_word_delete() {
    let mut ov = OverlayState::new_rename("hello world".to_string());
    // "hello world" carries no extension, so the WHOLE name arrives
    // pre-selected (see `rename_seeds_stem_selected_extension_untouched`
    // below) -- collapse it first (a plain caret motion, the ⌥⌫ word-delete
    // rule's OWN coverage lives here, not the "replace on delete" one) so
    // this test still exercises `delete_word_back`'s boundary rule the way
    // it always has.
    ov.rename_edit_char_right();
    assert_eq!(ov.rows[0].accept, "hello world");
    ov.rename_edit_pop_word();
    // The word-deleted value mirrors into corpus[0] (the visible editable row).
    assert_eq!(ov.rows[0].accept, "hello ");
    ov.rename_edit_pop_word();
    assert_eq!(ov.rows[0].accept, "");
}

// ── RENAME SEEDED SELECTION: stem selected, extension untouched ─────────────
// The file-manager rename convention. `Path::file_stem` gets every shape
// right with no special case: a normal `name.ext` selects `name`;
// no extension AND a dotfile (whose own `file_stem` IS the whole name, per
// `Path`'s documented rule) select the WHOLE name; `archive.tar.gz` selects
// `archive.tar`, stripping only the LAST extension, same as a file manager.

/// REQUIRED LAW (1/2): `new_rename` seeds the EXACT selection range + caret
/// position `Path::file_stem` implies, swept over the four shapes the
/// convention has to agree with a file manager on. Also asserts `query` --
/// the ONE field the render path tracks a per-character caret/selection box
/// for -- mirrors the seed from frame one, so the selection this module arms
/// is not a fact only `rename_edit.input` itself can see.
#[test]
fn rename_seeds_stem_selected_extension_untouched() {
    // (name, expected selection range, expected caret position).
    type Case = (&'static str, Option<(usize, usize)>, usize);
    let cases: [Case; 4] = [
        // A normal `name.ext`: the stem alone is selected.
        ("fukushima-trip.md", Some((0, 14)), 14),
        // No extension at all: the WHOLE name is selected (nothing to
        // protect from a bare Backspace/Delete).
        ("README", Some((0, 6)), 6),
        // A dotfile: its OWN `file_stem` is the whole name (Path's rule for
        // a name that begins with '.' and has no other '.' within), so the
        // whole name is selected here too.
        (".gitignore", Some((0, 10)), 10),
        // Multiple dots: only the LAST extension is excluded.
        ("archive.tar.gz", Some((0, 11)), 11),
    ];
    for (name, expected_selection, expected_caret) in cases {
        let ov = OverlayState::new_rename(name.to_string());
        let re = ov.rename_edit.as_ref().expect("rename edit armed");
        assert_eq!(
            re.input.selection_range(),
            expected_selection,
            "{name}: seeded selection range"
        );
        assert_eq!(
            re.input.caret(),
            expected_caret,
            "{name}: seeded caret position"
        );
        assert_eq!(
            ov.query,
            re.input.clone(),
            "{name}: query mirrors the seed from frame one"
        );
    }
}

/// An EMPTY name (the "unchanged input is a no-op" shape a caller passes
/// when there is nothing to rename FROM) seeds no selection at all --
/// `Path::file_stem` on an empty path is `None`, and [`TextBox::
/// seeded_selecting_prefix`]'s own zero-width guard keeps `anchor` at
/// `None` rather than arming a phantom selection.
#[test]
fn rename_seeds_no_selection_for_an_empty_name() {
    let ov = OverlayState::new_rename(String::new());
    let re = ov.rename_edit.as_ref().expect("rename edit armed");
    assert_eq!(re.input.selection_range(), None);
    assert_eq!(re.input.caret(), 0);
}

#[test]
fn link_minibuffer_word_delete() {
    let mut ov = OverlayState::new_link_edit(
        "http://a.com/path".to_string(),
        LinkEditMode::Empty { at: 0 },
    );
    ov.link_edit_pop_word(); // drops the trailing "path" segment, keeps the "/"
    assert_eq!(ov.rows[0].accept, "http://a.com/");
}

#[test]
fn keep_minibuffer_word_delete() {
    let mut ov = OverlayState::new_keep_name();
    for c in "my great note".chars() {
        ov.keep_edit_push(c);
    }
    ov.keep_edit_pop_word();
    assert_eq!(ov.rows[0].accept, "my great ");
}

// ── ONE SHARED TEXTBOX MODEL ───────────────────────────────────────────────

/// C — PICKER QUERY: word MOTION moves the caret WITHOUT refiltering (the
/// items list, selection, and scroll stay untouched — motion is not an edit),
/// and a subsequent INSERT splices at that mid-string caret (not appended at
/// the end), which THEN refilters as usual.
#[test]
fn picker_query_word_motion_does_not_refilter_then_insert_splices_mid_string() {
    let mut ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    for c in "readme".chars() {
        ov.push(c);
    }
    assert_eq!(ov.query, "readme");
    let items_before = ov.items.clone();
    let selected_before = ov.selected;
    // Word-left twice lands the caret at the START ("readme" is one word).
    ov.query.word_left();
    assert_eq!(
        ov.query.caret(),
        0,
        "word_left walks to the start of the one word"
    );
    // Pure motion: the ranked list is untouched.
    assert_eq!(ov.items, items_before, "caret motion never refilters");
    assert_eq!(ov.selected, selected_before);
    // Insert at the (now START) caret splices, not appends.
    ov.push('x');
    assert_eq!(
        ov.query, "xreadme",
        "insert lands at the caret, not the end"
    );
}

/// C — RENAME MINIBUFFER: the `/`-reject filter still applies at a
/// MID-STRING caret, not just at the end.
#[test]
fn rename_minibuffer_rejects_slash_mid_string() {
    let mut ov = OverlayState::new_rename("hello".to_string());
    ov.rename_edit_word_left(); // caret -> 0 (one word)
    ov.rename_edit_push('/'); // rejected everywhere, including mid-string
    assert_eq!(
        ov.rows[0].accept, "hello",
        "the / is rejected, caret position or not"
    );
    ov.rename_edit_push('X');
    assert_eq!(
        ov.rows[0].accept, "Xhello",
        "a normal char still splices at the caret"
    );
}

/// C — LINK MINIBUFFER: no character filter, INCLUDING `/`, at a mid-string
/// caret (a URL legitimately contains it).
#[test]
fn link_minibuffer_accepts_slash_mid_string() {
    let mut ov = OverlayState::new_link_edit("ab".to_string(), LinkEditMode::Empty { at: 0 });
    ov.link_edit_char_left(); // caret between 'a' and 'b'
    ov.link_edit_push('/');
    assert_eq!(ov.rows[0].accept, "a/b");
}

/// C — KEEP-VERSION MINIBUFFER: an empty (or whitespace-only) input still
/// commits to `None` (the plain, nameless keep) even after caret motion that
/// never actually inserted anything.
#[test]
fn keep_minibuffer_empty_after_motion_still_targets_none() {
    let mut ov = OverlayState::new_keep_name();
    ov.keep_edit_word_left(); // no-op motion on an empty field
    ov.keep_edit_char_right(); // no-op motion (nothing to move onto)
    assert_eq!(
        ov.keep_edit_target(),
        Some(None),
        "empty input is still the nameless keep"
    );
    for c in "  ".chars() {
        ov.keep_edit_push(c);
    }
    assert_eq!(
        ov.keep_edit_target(),
        Some(None),
        "whitespace-only input is ALSO the nameless keep"
    );
}

/// C — SETTINGS VALUE EDIT: the digit/`.`/`%` filter still applies at a
/// MID-STRING caret (word-left from a seeded value lands the caret before
/// the digits), and Esc still restores the ORIGINAL cell.
#[test]
fn settings_value_edit_filters_mid_string_and_esc_restores() {
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        vec!["Zoom".to_string()],
        vec![],
        vec![],
    );
    ov.set_secondaries(vec!["100%".to_string()]);
    ov.start_value_edit("zoom".to_string(), "Zoom".to_string());
    assert_eq!(
        ov.value_edit.as_ref().unwrap().input.caret(),
        4,
        "seeded caret at the end"
    );
    ov.value_edit_word_left();
    assert_eq!(ov.value_edit.as_ref().unwrap().input.caret(), 0);
    // A letter is rejected at the mid-string (here, start) caret too.
    ov.value_edit_push('x');
    assert_eq!(
        ov.rows[0].secondary, "100%",
        "non-digit/./% rejected at any caret position"
    );
    // A digit splices at the caret.
    ov.value_edit_push('9');
    assert_eq!(ov.rows[0].secondary, "9100%");
    // Esc restores the cell to what it showed before the edit began.
    ov.value_edit_cancel();
    assert_eq!(
        ov.rows[0].secondary, "100%",
        "Esc restores the original value"
    );
    assert!(ov.value_edit.is_none());
}

/// C — SETTINGS VALUE EDIT: Enter's commit target reads the CURRENT typed
/// value regardless of where the caret ended up.
#[test]
fn settings_value_edit_commit_target_reads_current_value_after_motion() {
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        vec!["Zoom".to_string()],
        vec![],
        vec![],
    );
    ov.set_secondaries(vec!["50%".to_string()]);
    ov.start_value_edit("zoom".to_string(), "Zoom".to_string());
    ov.value_edit_char_left(); // caret before the trailing '%'
    ov.value_edit_push('5'); // splices before the '%': "50%" -> "505%"
    assert_eq!(
        ov.value_edit_target(),
        Some(("zoom".to_string(), "505%".to_string()))
    );
}

/// B (per-surface) — UNICODE round-trip through the REAL Rename minibuffer:
/// a CJK name inserts, backspaces, and word-deletes exactly like the plain-
/// ASCII case, proving the char-index splice discipline holds end-to-end
/// through `overlay::capture`, not just inside `TextBox` directly.
#[test]
fn rename_minibuffer_handles_multibyte_text_end_to_end() {
    let mut ov = OverlayState::new_rename(String::new());
    for c in "日本語".chars() {
        ov.rename_edit_push(c);
    }
    assert_eq!(ov.rows[0].accept, "日本語");
    ov.rename_edit_char_left();
    ov.rename_edit_push('X');
    assert_eq!(ov.rows[0].accept, "日本X語");
    ov.rename_edit_pop();
    assert_eq!(ov.rows[0].accept, "日本語");
}
