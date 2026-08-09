//! Markdown formatting-command toggles (align table, bold/bullet/code-block),
//! smart-newline continuation, Tab list indent/outdent, and select-all --
//! split out of the former monolithic `actions::tests` (2026-07
//! code-organization pass).

use super::super::*;
use super::{drive_act, drive_format, drive_newline, md};
use crate::overlay::OverlayKind;

#[test]
fn copy_link_destination_uses_the_catalog_action_and_existing_kill_ring() {
    let mut buffer = Buffer::from_str("see [awl](https://awl.example/docs) now\n");
    buffer.set_cursor(7); // inside the visible link label
    drive_act(&mut buffer, &Action::CopyLinkDestination);
    assert_eq!(buffer.kill_buffer(), "https://awl.example/docs");
    let before = buffer.kill_buffer().to_string();
    buffer.set_cursor(0);
    drive_act(&mut buffer, &Action::CopyLinkDestination);
    assert_eq!(
        buffer.kill_buffer(),
        before,
        "outside a link is a calm no-op"
    );
}

#[test]
fn align_table_aligns_under_caret_is_undoable_and_no_ops_outside() {
    // Action::AlignTable routes through the SAME apply_transition seam a palette/menu
    // invocation uses, so `--keys` drives it identically. A no-path buffer is
    // markdown, so the table under the caret aligns.
    let src = "intro\n| Name | V |\n|---|---|\n| a | 100 |\ntail\n";
    let mut buffer = Buffer::from_str(src);
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };

    // Caret INSIDE the table (on the body row) — align re-pads the block.
    buffer.set_cursor(buffer.line_col_to_char(3, 2));
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let before = ctx.buffer.text();
    apply_transition(&mut ctx, &Action::AlignTable, false).primary();
    let after = ctx.buffer.text();
    assert_ne!(after, before, "align edited the buffer");
    assert!(
        after.contains("| Name | V   |\n| ---- | --- |\n| a    | 100 |"),
        "the table block is aligned in place: {after:?}"
    );
    // The surrounding prose is untouched.
    assert!(after.starts_with("intro\n") && after.ends_with("tail\n"));

    // UNDOABLE: one Cmd-Z restores the exact pre-align source.
    ctx.buffer.undo();
    assert_eq!(
        ctx.buffer.text(),
        before,
        "undo restores the pre-align source"
    );

    // NO-OP outside a table: caret on the prose intro line does nothing.
    ctx.buffer.set_cursor(0);
    let untouched = ctx.buffer.text();
    let eff = apply_transition(&mut ctx, &Action::AlignTable, false).primary();
    assert_eq!(eff, Effect::None, "align outside a table is a calm no-op");
    assert_eq!(ctx.buffer.text(), untouched, "…and edits nothing");
    assert!(!ctx.buffer.can_undo(), "…so there is nothing to undo");
}

#[test]
fn bold_toggle_through_apply_transition_is_one_undoable_edit() {
    // Cmd-P → "Bold" routes Action::Bold through the SAME apply_transition seam a key /
    // `--keys` invocation rides. Select "quick" (cols 4..9) and toggle bold.
    let mut b = drive_format("the quick fox", Some(4), 9, &Action::Bold);
    assert_eq!(b.text(), "the **quick** fox", "bold wrapped the selection");
    // The selection covers the same visible text, inside the delimiters.
    assert_eq!(b.selection_range(), Some((6, 11)));
    // ONE undo restores the exact pre-toggle text (a full-buffer replace never
    // coalesces — the whole toggle is a single atomic group).
    b.undo();
    assert_eq!(b.text(), "the quick fox", "one Cmd-Z reverts the toggle");
}

#[test]
fn bullet_list_toggle_through_apply_transition_round_trips_and_undoes() {
    // Select the two content lines (cols 0..4 over "a\nb\n") and toggle a bullet list.
    let mut b = drive_format("a\nb\nc\n", Some(0), 4, &Action::ToggleBulletList);
    assert_eq!(b.text(), "- a\n- b\nc\n", "every selected line is prefixed");
    // A second dispatch (the selection now spans the prefixed lines) strips them.
    let re = drive_format(
        &b.text(),
        b.selection_range().map(|(s, _)| s),
        b.selection_range().unwrap().1,
        &Action::ToggleBulletList,
    );
    assert_eq!(re.text(), "a\nb\nc\n", "re-toggle strips the bullets back");
    // And one undo of the FIRST toggle restores the plain lines.
    b.undo();
    assert_eq!(b.text(), "a\nb\nc\n", "one Cmd-Z reverts the bullet toggle");
}

#[test]
fn code_block_toggle_through_apply_transition_wraps_and_undoes() {
    let mut b = drive_format("let x = 1;\n", None, 3, &Action::ToggleCodeBlock);
    assert_eq!(
        b.text(),
        "```\nlet x = 1;\n```\n",
        "the caret line is fenced"
    );
    b.undo();
    assert_eq!(b.text(), "let x = 1;\n", "one Cmd-Z reverts the fence");
}

#[test]
fn heading_toggle_is_a_noop_on_a_code_buffer() {
    // Formatting commands are markdown-only: a `.rs` buffer is never touched
    // (block markup would corrupt code). No edit → nothing to undo.
    use std::path::PathBuf;
    let mut buffer = Buffer::from_str("fn main() {}\n");
    buffer.set_path(PathBuf::from("/tmp/x.rs"));
    buffer.set_cursor(0);
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    apply_transition(&mut ctx, &Action::ToggleHeading, false).primary();
    assert_eq!(
        ctx.buffer.text(),
        "fn main() {}\n",
        "a code buffer is left untouched"
    );
    assert!(!ctx.buffer.can_undo(), "no edit was recorded");
}

#[test]
fn smart_newline_continues_lists_quotes_and_indent() {
    // Unordered bullet carries to the new line.
    let mut b = md("- a", 3);
    drive_newline(&mut b);
    assert_eq!(b.text(), "- a\n- ");
    assert_eq!(b.cursor_char(), 6);

    // Ordered list AUTO-INCREMENTS the number.
    let mut b = md("1. first", 8);
    drive_newline(&mut b);
    assert_eq!(b.text(), "1. first\n2. ");

    // A double-digit ordered marker keeps counting and preserves the delimiter.
    let mut b = md("9) nine", 7);
    drive_newline(&mut b);
    assert_eq!(b.text(), "9) nine\n10) ");

    // Blockquote continues with the same '>' run.
    let mut b = md("> quote", 7);
    drive_newline(&mut b);
    assert_eq!(b.text(), "> quote\n> ");

    // Leading indentation is preserved on a plain Enter.
    let mut b = md("    code", 8);
    drive_newline(&mut b);
    assert_eq!(b.text(), "    code\n    ");
}

#[test]
fn smart_newline_continues_an_unchecked_task_item() {
    // A TASK item (checked or not) continues like a bullet, but carries the
    // checkbox forward ALWAYS UNCHECKED — never `[x]`, even continuing a checked
    // item (a fresh continuation line is new, unfinished work).
    let mut b = md("- [ ] buy milk", 14);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "- [ ] buy milk\n- [ ] ",
        "the checkbox continues, unchecked"
    );

    let mut b = md("- [x] done already", 19);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "- [x] done already\n- [ ] ",
        "continuing a CHECKED item still opens an UNCHECKED one"
    );

    // Nested (indented) task items keep their indent too.
    let mut b = md("  - [ ] nested", 14);
    drive_newline(&mut b);
    assert_eq!(b.text(), "  - [ ] nested\n  - [ ] ");
}

#[test]
fn smart_newline_empty_blockquote_always_ends_the_block() {
    // Blockquotes sit OUTSIDE the provenance law: an empty BLOCKQUOTE
    // unconditionally strips the dangling `>` run and ends, regardless of the
    // marker's origin (a directly-constructed `md()` buffer, exactly like bytes
    // loaded from disk, carries no generated-provenance flag either way).
    let mut b = md("> ", 2);
    drive_newline(&mut b);
    assert_eq!(b.text(), "");
    assert_eq!(b.cursor_char(), 0);
}

#[test]
fn smart_newline_empty_list_item_of_unknown_provenance_is_preserved() {
    // The ordered-list rule, GENERALIZED to numbered and task
    // items alongside bullets: Enter on an EMPTY list marker of ANY provenance
    // OTHER than "awl's own immediately preceding continuation" PRESERVES the
    // marker byte-semantically and opens a fresh PLAIN line below — it does NOT
    // strip the marker, and it does NOT emit a second one. A buffer built
    // directly via `md()` carries no generated-provenance flag, exactly like
    // bytes loaded from disk or typed by hand — see the dedicated keystroke-driven
    // tests below for the flag's actual set/clear lifecycle.
    for marker in ['-', '*', '+'] {
        let src = format!("{marker} a\n{marker} ");
        let mut b = md(&src, 6);
        drive_newline(&mut b);
        assert_eq!(
            b.text(),
            format!("{marker} a\n{marker} \n"),
            "the `{marker} ` bullet stays intact and a plain line opens below"
        );
        // Caret is at column 0 of the NEW empty line (char 7 over the 7-char body).
        assert_eq!(
            b.cursor_char(),
            7,
            "caret parks on the new plain line, col 0"
        );
        let (line, col) = b.cursor_line_col();
        assert_eq!(
            (line, col),
            (2, 0),
            "caret on the fresh line below the bullet"
        );
    }

    // The lone bullet at document START, no preceding sibling: still preserved.
    let mut b = md("- ", 2);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "- \n",
        "a lone empty bullet is preserved, plain line below"
    );
    assert_eq!(b.cursor_char(), 3);

    // NESTED empty bullet: the indent + `- ` marker are ALL preserved intact.
    let mut b = md("  - ", 4);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "  - \n",
        "the nested bullet (indent + marker) is preserved"
    );
    assert_eq!(b.cursor_char(), 5);
    assert_eq!(b.cursor_line_col(), (1, 0));

    // An empty ORDERED item of unknown provenance is ALSO preserved
    // (this REVERSES the old unconditional "ends the
    // block" behavior for the non-generated case).
    let mut b = md("1. ", 3);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "1. \n",
        "the ordered marker is preserved, plain line below"
    );
    assert_eq!(b.cursor_char(), 4);
    assert_eq!(b.cursor_line_col(), (1, 0));

    // A NESTED empty ordered item: indent + marker both preserved.
    let mut b = md("  1. ", 5);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "  1. \n",
        "the nested ordered marker is preserved intact"
    );

    // An empty TASK item (unchecked box, no text) is preserved too.
    let mut b = md("- [ ] ", 6);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "- [ ] \n",
        "the empty checkbox is preserved, plain line below"
    );

    // …and a CHECKED empty task item preserves the checked state.
    let mut b = md("- [x] ", 6);
    drive_newline(&mut b);
    assert_eq!(b.text(), "- [x] \n", "the checked box is preserved as-is");
}

#[test]
fn smart_newline_no_guess_provenance_law() {
    // THE LAW: a lone empty list marker's Enter behavior depends on
    // WHERE it came from, not on its bytes — identical bytes never let the
    // second Enter guess "generated". Driven through the REAL `Action::Newline`
    // dispatch (exactly what `--keys` replays), never by hand-assembling text.

    // (1) A lone `- ` LOADED FROM DISK (here: constructed directly, identical
    // bytes to a generated line, but NOT reached via awl's own continuation) —
    // Enter PRESERVES the marker and opens a plain line below.
    let mut loaded = md("- ", 2);
    drive_act(&mut loaded, &Action::Newline);
    assert_eq!(
        loaded.text(),
        "- \n",
        "a loaded/typed empty marker is preserved"
    );
    assert_eq!(loaded.cursor_line_col(), (1, 0));

    // (2) The SAME bytes, but reached by awl's OWN continuation: type "- a",
    // Enter (generates the bare "- " continuation line), Enter again —
    // immediately, nothing intervening — must EXIT the list instead (the
    // ordinary "Enter twice" gesture), even though the buffer momentarily held
    // the EXACT SAME "- a\n- " text as scenario (1)'s sibling case.
    let mut generated = md("", 0);
    drive_act(&mut generated, &Action::InsertChar('-'));
    drive_act(&mut generated, &Action::InsertChar(' '));
    drive_act(&mut generated, &Action::InsertChar('a'));
    drive_act(&mut generated, &Action::Newline); // continuation: "- a\n- "
    assert_eq!(
        generated.text(),
        "- a\n- ",
        "the continuation opened a bare empty marker"
    );
    drive_act(&mut generated, &Action::Newline); // the "second Enter"
    assert_eq!(
        generated.text(),
        "- a\n",
        "Enter on AWL'S OWN generated empty continuation ends the list"
    );
    assert_eq!(
        generated.cursor_char(),
        4,
        "caret parks at the now-blank line"
    );

    // (3) Non-vacuous by construction: (1) and (2) reach the SAME "- a\n- "
    // intermediate text via different routes and DIVERGE on the very next
    // Enter — a law that guessed from bytes alone (the pre-item-78 rule) could
    // not tell them apart and would preserve both.

    // (4) Any INTERVENING action between the generating Enter and the next one
    // clears the provenance — the second Enter then falls back to "preserve",
    // exactly like scenario (1). `acts` returns the caret to col 2 (the empty
    // marker's end, needed for the probe Enter to see the marker at all rather
    // than "caret inside it") via ANOTHER motion, so every step in the sequence
    // is itself a legitimate intervening op — never a flag-blind reposition.
    let intervenes = |acts: &[Action]| {
        let mut b = md("", 0);
        drive_act(&mut b, &Action::InsertChar('-'));
        drive_act(&mut b, &Action::InsertChar(' '));
        drive_act(&mut b, &Action::InsertChar('a'));
        drive_act(&mut b, &Action::Newline); // "- a\n- ", flag SET, caret at col 2
        for act in acts {
            drive_act(&mut b, act);
        }
        drive_act(&mut b, &Action::Newline);
        b.text()
    };
    assert_eq!(
        intervenes(&[Action::BackwardChar, Action::ForwardChar]),
        "- a\n- \n",
        "an intervening motion clears the flag — the marker is preserved, not ended"
    );
    assert_eq!(
        intervenes(&[Action::InsertChar('x'), Action::DeleteBackward]),
        "- a\n- \n",
        "intervening typing (even undone back to the identical bytes) clears the flag"
    );
    assert_eq!(
        intervenes(&[Action::SetMark]),
        "- a\n- \n",
        "a selection change (C-Space) clears the flag too — caret untouched by SetMark"
    );

    // (5) Undo THEN redo also clears it, even though redo restores the exact
    // generated bytes — the provenance is gone, not the text.
    let mut undo_redo = md("", 0);
    drive_act(&mut undo_redo, &Action::InsertChar('-'));
    drive_act(&mut undo_redo, &Action::InsertChar(' '));
    drive_act(&mut undo_redo, &Action::InsertChar('a'));
    drive_act(&mut undo_redo, &Action::Newline); // "- a\n- ", flag SET
    undo_redo.undo();
    undo_redo.redo(); // back to "- a\n- ", byte-identical, provenance gone
    assert_eq!(
        undo_redo.text(),
        "- a\n- ",
        "redo restored the identical bytes"
    );
    drive_act(&mut undo_redo, &Action::Newline);
    assert_eq!(
        undo_redo.text(),
        "- a\n- \n",
        "undo/redo cleared the flag — the marker is preserved, not ended"
    );

    // (6) C-g (Cancel → clear_mark) also clears it, independent of any motion.
    let mut cancel = md("", 0);
    drive_act(&mut cancel, &Action::InsertChar('-'));
    drive_act(&mut cancel, &Action::InsertChar(' '));
    drive_act(&mut cancel, &Action::InsertChar('a'));
    drive_act(&mut cancel, &Action::Newline); // "- a\n- ", flag SET
    drive_act(&mut cancel, &Action::Cancel);
    drive_act(&mut cancel, &Action::Newline);
    assert_eq!(
        cancel.text(),
        "- a\n- \n",
        "C-g clears the flag too — preserved, not ended"
    );
}

#[test]
fn smart_newline_empty_bullet_preserve_is_one_undo_group() {
    // The whole gesture is ONE atomic undo group: a single Cmd-Z restores the
    // pre-Enter state (the empty bullet, caret at its end), and a redo re-applies it.
    let mut b = md("- a\n- ", 6);
    drive_newline(&mut b);
    assert_eq!(b.text(), "- a\n- \n");
    let after = b.text();

    b.undo();
    assert_eq!(
        b.text(),
        "- a\n- ",
        "one undo removes exactly the opened line"
    );
    assert_eq!(
        b.cursor_char(),
        6,
        "caret restored to the end of the empty bullet"
    );

    b.redo();
    assert_eq!(
        b.text(),
        after,
        "redo re-opens the plain line under the preserved bullet"
    );
    assert_eq!(b.cursor_char(), 7);

    // Driven as the real keystroke stream `- <space> Enter`: typing then Enter must
    // still leave the preserved bullet + a writable plain line, undoable as sensible
    // chunks (the Enter is its own group — it never coalesces the marker away).
    let mut b = md("", 0);
    drive_act(&mut b, &Action::InsertChar('-'));
    drive_act(&mut b, &Action::InsertChar(' '));
    drive_act(&mut b, &Action::Newline);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        b.text(),
        "- \nx",
        "typed bullet preserved; `x` lands on the plain line"
    );
    assert_eq!(b.cursor_char(), 4);
    b.undo(); // remove the x
    assert_eq!(
        b.text(),
        "- \n",
        "undo the typed x, bullet + plain line intact"
    );
    b.undo(); // remove the opened line
    assert_eq!(
        b.text(),
        "- ",
        "undo the Enter — the preserved bullet remains"
    );
}

#[test]
fn smart_newline_empty_bullet_preserve_round_trips_crlf() {
    // CRLF restoration: a buffer whose EOL is CRLF preserves the empty bullet and
    // opens a plain line, and `disk_bytes` restores CRLF on EVERY line — including
    // the freshly opened one. The rope stays pure `\n` (EOL is document metadata).
    let mut b = md("- a\n- ", 6);
    b.set_eol(crate::buffer::Eol::Crlf);
    drive_newline(&mut b);
    assert_eq!(
        b.text(),
        "- a\n- \n",
        "the rope preserves the bullet + a plain line"
    );
    let disk = String::from_utf8(b.disk_bytes()).unwrap();
    assert_eq!(
        disk, "- a\r\n- \r\n",
        "save restores CRLF on every line, opened one too"
    );
}

#[test]
fn smart_newline_is_markdown_only() {
    // A non-markdown buffer (a path with a non-md extension) gets a PLAIN
    // newline — no marker continuation — so `.rs` / `.txt` editing is
    // byte-identical. (A no-path scratch buffer is now the prose-first writing
    // surface and DOES continue markers; only a saved non-md file opts out.)
    let mut b = Buffer::from_str("- a");
    b.set_path(std::path::PathBuf::from("code.rs"));
    b.set_cursor(3);
    drive_newline(&mut b);
    assert_eq!(b.text(), "- a\n");
    assert_eq!(b.cursor_char(), 4);
}

#[test]
fn tab_indents_a_list_line_and_shift_tab_outdents() {
    // TAB on a bullet indents one level (+2 leading spaces); the depth glyph is
    // derived downstream, so only the text changes here.
    let mut b = md("- item", 6);
    drive_act(&mut b, &Action::InsertTab);
    assert_eq!(b.text(), "  - item");
    // The caret rides with the content (+2).
    assert_eq!(b.cursor_char(), 8);

    // SHIFT-TAB outdents it back (−2, clamped at 0 so a second one is a no-op).
    drive_act(&mut b, &Action::Outdent);
    assert_eq!(b.text(), "- item");
    let v = b.version();
    drive_act(&mut b, &Action::Outdent);
    assert_eq!(b.text(), "- item", "outdent clamps at column 0");
    assert_eq!(b.version(), v, "a clamped outdent makes no edit");
}

#[test]
fn tab_indents_an_ordered_list_without_renumbering() {
    // Ordered items indent too (Tab/Shift-Tab), and we do NOT auto-renumber.
    let mut b = md("1. first", 8);
    drive_act(&mut b, &Action::InsertTab);
    assert_eq!(
        b.text(),
        "  1. first",
        "ordered item indents, number unchanged"
    );
    drive_act(&mut b, &Action::Outdent);
    assert_eq!(b.text(), "1. first");
}

#[test]
fn tab_off_a_list_inserts_spaces_not_an_indent() {
    // On a plain prose line Tab keeps the existing soft-tab (to the next 4-stop),
    // so non-list editing is unchanged.
    let mut b = md("hello", 5);
    drive_act(&mut b, &Action::InsertTab);
    assert_eq!(b.text(), "hello   ", "col 5 => 3 spaces to the next 4-stop");
}

#[test]
fn tab_indents_all_selected_list_lines() {
    // A selection spanning three bullets: one Tab indents them ALL as one undo step.
    let mut b = md("- a\n- b\n- c", 0);
    b.set_mark(); // anchor at 0
    b.set_cursor(b.text().chars().count()); // extend to end => whole doc selected
    drive_act(&mut b, &Action::InsertTab);
    assert_eq!(
        b.text(),
        "  - a\n  - b\n  - c",
        "every selected bullet indents"
    );
    // One undo restores the whole block (the indent is atomic).
    b.undo();
    assert_eq!(
        b.text(),
        "- a\n- b\n- c",
        "the block indent is one atomic undo"
    );

    // Shift-Tab outdents a whole selection back, on an already-indented block.
    let mut b = md("  - a\n  - b\n  - c", 0);
    b.set_mark();
    b.set_cursor(b.text().chars().count());
    drive_act(&mut b, &Action::Outdent);
    assert_eq!(b.text(), "- a\n- b\n- c", "every selected bullet outdents");
}

#[test]
fn select_all_selects_the_whole_buffer_region() {
    // A multi-line buffer with the cursor parked mid-document.
    let mut b = Buffer::from_str("alpha\nbeta\ngamma\n");
    let len = b.text().chars().count();
    b.set_cursor(3); // somewhere in the middle, no mark
    assert!(!b.has_selection());

    drive_act(&mut b, &Action::SelectAll);

    // Mark at document start, point at document end => the whole doc is the region.
    assert!(b.has_selection());
    assert_eq!(b.anchor_char(), Some(0));
    assert_eq!(b.cursor_char(), len);
    assert_eq!(b.selection_range(), Some((0, len)));
    // Endpoints span from (line 0, col 0) to the last line's last col.
    let ((l0, c0), (l1, _c1)) = b.selection_line_col().unwrap();
    assert_eq!((l0, c0), (0, 0), "region starts at document start");
    assert_eq!(l1, b.line_count() - 1, "region ends on the last line");
}

#[test]
fn select_all_on_empty_buffer_is_a_safe_no_op() {
    // An EMPTY buffer: select-all must not panic and leaves an empty region
    // (anchor == cursor == 0), so nothing is "selected".
    let mut b = Buffer::from_str("");
    drive_act(&mut b, &Action::SelectAll);
    assert!(
        !b.has_selection(),
        "empty buffer => empty region, not a selection"
    );
    assert_eq!(b.cursor_char(), 0);
    assert_eq!(b.selection_range(), None);
}

#[test]
fn kill_region_after_select_all_empties_the_buffer() {
    // Cmd-A then C-w (cut) removes the ENTIRE document.
    let mut b = Buffer::from_str("one\ntwo\nthree\n");
    drive_act(&mut b, &Action::SelectAll);
    drive_act(&mut b, &Action::KillRegion);
    assert_eq!(b.text(), "", "select-all + cut empties the buffer");
    assert!(!b.has_selection());
    // The cut text is in the kill buffer, so a yank restores the whole doc.
    drive_act(&mut b, &Action::YankText);
    assert_eq!(
        b.text(),
        "one\ntwo\nthree\n",
        "the cut whole-doc yanks back"
    );
}

#[test]
fn type_after_select_all_replaces_the_whole_buffer() {
    // Cmd-A then typing a char replaces the ENTIRE selection with that char,
    // as one atomic edit (one undo restores the original document).
    let mut b = Buffer::from_str("keep\nnothing\nof this\n");
    drive_act(&mut b, &Action::SelectAll);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        b.text(),
        "x",
        "the whole selection is replaced by the typed char"
    );
    assert_eq!(b.cursor_char(), 1);
    b.undo();
    assert_eq!(
        b.text(),
        "keep\nnothing\nof this\n",
        "one undo restores the original"
    );
}

#[test]
fn copy_region_after_select_all_copies_all_and_keeps_text() {
    // Cmd-A then M-w (copy) leaves the text intact but stages the whole doc for
    // a yank (the mark clears, as copy_region does).
    let mut b = Buffer::from_str("copy\nme\n");
    drive_act(&mut b, &Action::SelectAll);
    drive_act(&mut b, &Action::CopyRegion);
    assert_eq!(b.text(), "copy\nme\n", "copy leaves the document unchanged");
    assert!(!b.has_selection(), "copy clears the mark");
    // Yanking at the end appends the copied whole document.
    b.buffer_end();
    drive_act(&mut b, &Action::YankText);
    assert_eq!(
        b.text(),
        "copy\nme\ncopy\nme\n",
        "the copied whole doc yanks in"
    );
}

#[test]
fn smart_newline_parser_declines_plain_and_inside_marker() {
    // Plain prose: nothing to continue.
    assert!(smart_newline_for("hello", 5).is_none());
    // Caret inside the marker (col 0 of a bullet): plain newline, no dupe.
    assert!(smart_newline_for("- item", 0).is_none());
    // A lone "-" without a trailing space is not a list yet.
    assert!(smart_newline_for("-", 1).is_none());
}

#[test]
fn dash_then_enter_leaves_a_writable_line() {
    // Regression — `-` then Enter must never strand an UNWRITABLE empty
    // item. Decided semantics (2026-07-23): a lone `-` (no trailing space) is not
    // a list yet, so Enter falls through to a PLAIN newline — the dash stays a
    // literal `-` on its own line with a fresh blank line below. Drive the whole
    // gesture through the REAL apply_transition seam exactly as `--keys "- Enter x"`
    // does (InsertChar → Newline → InsertChar), then assert the typed character
    // actually LANDED after the newline and the caret advanced onto it — i.e. the
    // new line is writable, not eaten. (`-` alone yields no `md_spans`, so nothing
    // conceals; the buffer-level writability contract is the floor this pins.)
    let mut b = md("", 0);
    drive_act(&mut b, &Action::InsertChar('-'));
    drive_act(&mut b, &Action::Newline);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        b.text(),
        "-\nx",
        "the dash stays literal and `x` lands on the new line"
    );
    // Caret sits AFTER the `x`: char 3 over "-\nx" — the line the user landed on
    // genuinely took the keystroke.
    assert_eq!(b.cursor_char(), 3, "caret advanced past the written `x`");
    let (line, col) = b.cursor_line_col();
    assert_eq!(
        (line, col),
        (1, 1),
        "caret is on the new line, one column in"
    );
}

#[test]
fn smart_newline_ordered_marker_at_usize_max_saturates_no_overflow() {
    // A pathological ordered marker of exactly `usize::MAX` parses fine, but the
    // continuation used to compute `n + 1` — which OVERFLOWS (panic in debug,
    // wrap-to-0 in release). `saturating_add(1)` pins the number at usize::MAX
    // instead: the marker simply stops counting up rather than crashing.
    let max = usize::MAX; // 18446744073709551615 on 64-bit
    let line = format!("{max}. item");
    let col = line.chars().count();
    match smart_newline_for(&line, col) {
        Some(SmartNewline::ContinueListItem { prefix, bare }) => {
            assert_eq!(
                prefix,
                format!("{max}. "),
                "the number saturates, never overflows"
            );
            assert!(
                bare,
                "nothing followed the cursor — the opened line is bare"
            );
        }
        _ => panic!("expected a continued ordered item at the usize::MAX marker"),
    }
}
