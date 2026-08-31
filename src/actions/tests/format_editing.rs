//! Markdown formatting-command toggles (align table, bold/bullet/code-block),
//! smart-newline continuation, Tab list indent/outdent, and select-all --
//! split out of the former monolithic `actions::tests` (2026-07
//! code-organization pass).

use super::super::*;
use super::{drive_act, drive_act_effect, drive_format, drive_newline, md};
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
fn auto_align_fires_on_row_leave_but_not_while_still_editing_the_row() {
    // Type raw, misaligned source into a cell -- the table stays visibly ragged
    // (no realign) as long as the caret hasn't left the row it's typing in.
    let mut buffer = md("|Name|V|\n|---|---|\n|a|100|\n|b|2|\n", 0);
    buffer.set_cursor(buffer.line_col_to_char(2, 2)); // right after "a" on row 2
    drive_act(&mut buffer, &Action::InsertChar('b'));
    assert_eq!(
        buffer.line_text(2),
        "|ab|100|",
        "no realign mid-row, even though the table is now visibly ragged"
    );
    drive_act(&mut buffer, &Action::InsertChar('c'));
    assert_eq!(
        buffer.line_text(2),
        "|abc|100|",
        "a second keystroke on the SAME row still doesn't trigger it"
    );
    let misaligned_full = buffer.text();

    // Leaving the row (Down arrow, Action::NextLine) is the trigger: the whole
    // table snaps to Prettier alignment, computed fresh over the CURRENT (typed)
    // content -- reusing the exact same `align_table` the manual command uses.
    drive_act(&mut buffer, &Action::NextLine);
    let expected =
        crate::markdown::align_table("|Name|V|\n|---|---|\n|abc|100|\n|b|2|");
    assert_eq!(
        buffer.text(),
        format!("{expected}\n"),
        "leaving the row snapped the whole table into Prettier alignment"
    );
    assert_ne!(buffer.text(), misaligned_full, "the row-leave actually edited the source");
}

#[test]
fn auto_align_undoes_as_its_own_step_revealing_the_raw_typed_edit() {
    let mut buffer = md("|Name|V|\n|---|---|\n|a|100|\n|b|2|\n", 0);
    buffer.set_cursor(buffer.line_col_to_char(2, 2));
    drive_act(&mut buffer, &Action::InsertChar('b'));
    drive_act(&mut buffer, &Action::InsertChar('c'));
    let misaligned_full = buffer.text();
    let original_full = "|Name|V|\n|---|---|\n|a|100|\n|b|2|\n".to_string();

    drive_act(&mut buffer, &Action::NextLine); // triggers the auto-align
    assert_ne!(buffer.text(), misaligned_full, "the align actually changed the source");

    // ONE undo reveals the user's last raw edit -- the auto-align is its own
    // sealed group, never merged with (and never discarding) the "bc" typing
    // that preceded it.
    buffer.undo();
    assert_eq!(
        buffer.text(),
        misaligned_full,
        "one undo restores the pre-align, still-typed (misaligned) content"
    );

    // A second undo removes the coalesced "bc" typing itself, back to the
    // untouched original -- confirming the align's group was truly separate
    // rather than having silently swallowed part of the typing on the way in.
    buffer.undo();
    assert_eq!(
        buffer.text(),
        original_full,
        "a second undo removes the typing, unaffected by the align that followed it"
    );
}

#[test]
fn caret_lands_on_the_same_logical_cell_after_auto_align_not_a_raw_offset() {
    // Widen row 2's first cell (typed raw, misaligned) so realigning grows
    // column 0's width -- everything to its right on every row shifts, which is
    // exactly what turns a raw column index into the wrong cell.
    let src = "|N|V|\n|---|---|\n|a|100|\n|bbb|2|\n";
    let mut buffer = md(src, 0);
    buffer.set_cursor(buffer.line_col_to_char(2, 2)); // right after "a"
    for c in "ZZZZ".chars() {
        drive_act(&mut buffer, &Action::InsertChar(c));
    }
    assert_eq!(buffer.line_text(2), "|aZZZZ|100|");
    let pre_align_col = buffer.cursor_line_col().1;

    // What Down's own column-clamp (`min(goal, target_len)`, `buffer/motion.rs`'s
    // `vertical`) lands on over row 3's PRE-align text -- computed independently
    // of the production caret-preservation code under test, so the comparison
    // below isn't just restating it.
    let pre_row3 = "|bbb|2|";
    let expected_pre_col = pre_align_col.min(pre_row3.chars().count());
    let expected_pos = crate::markdown::locate_table_caret(pre_row3, expected_pre_col);

    // Leaving the row triggers the auto-align; Down's own landing decided which
    // cell the caret was over the instant before the re-pad ran.
    drive_act(&mut buffer, &Action::NextLine);

    let (row, col) = buffer.cursor_line_col();
    assert_eq!(row, 3, "the caret is still on row 3 after the align rewrote it");
    let realigned_row3 = buffer.line_text(3);
    assert_ne!(realigned_row3, pre_row3, "row 3 actually got re-padded");
    let actual_pos = crate::markdown::locate_table_caret(&realigned_row3, col);
    assert_eq!(
        actual_pos, expected_pos,
        "the caret preserved its logical cell + offset across the re-pad, not a \
         raw column that now points at padding or a different cell"
    );
}

#[test]
fn tag_document_language_is_one_undoable_edit_and_never_writes_a_second_block() {
    // The EXPLICIT door that replaced the open-time auto-stamp. It routes
    // through the same `apply_transition` seam a palette accept rides, so
    // `--keys` drives it identically live and in replay.
    let _g = crate::testlock::serial();
    let original = "# 你好\n\nこれは日本語です。你好。\n";
    let mut b = Buffer::from_str(original);
    let effect = drive_act_effect(&mut b, &Action::TagDocumentLanguage);
    assert_eq!(
        effect,
        Effect::Notice(NoticeEffect::Toast(
            "Document language: Japanese".to_string()
        )),
        "the applied language is acknowledged by writer-facing name through the toast channel"
    );
    assert_eq!(
        b.text(),
        format!("---\nlang: ja\n---\n{original}"),
        "the detected tag is stamped at byte 0"
    );
    assert!(
        b.is_dirty(),
        "the explicit metadata edit participates in save"
    );
    assert_eq!(
        b.disk_bytes(),
        format!("---\nlang: ja\n---\n{original}").as_bytes(),
        "the next save writes the frontmatter exactly once"
    );
    // ONE undoable edit: Cmd-Z restores the pre-tag document exactly.
    b.undo();
    assert_eq!(b.text(), original, "one undo removes the whole block");

    // Re-running it never adds a SECOND block — the gate is the presence of a
    // frontmatter block, not a one-shot flag.
    let effect = drive_act_effect(&mut b, &Action::TagDocumentLanguage);
    assert!(matches!(effect, Effect::Notice(NoticeEffect::Toast(_))));
    let once = b.text();
    let effect = drive_act_effect(&mut b, &Action::TagDocumentLanguage);
    assert_eq!(b.text(), once, "a tagged document is never re-tagged");
    assert_eq!(
        effect,
        Effect::None,
        "a no-op never claims that a language was applied"
    );
}

#[test]
fn tag_document_language_reads_the_live_ambiguity_ladder_and_no_ops_without_cjk() {
    // AMBIGUOUS HAN follows the LIVE ladder — the same global the Settings
    // "Ambiguous CJK reads as" row reads and the CJK picker promotes — so the
    // stamp a user asks for agrees with the tiebreak they set. Probed on BOTH
    // sides of the condition, with the pair required to differ.
    use crate::frontmatter::{DEFAULT_CJK_PRIORITY, Lang, cjk_priority, set_cjk_priority};
    let _g = crate::testlock::serial();
    let restore = cjk_priority();
    let han_only = "汉字漢字\n";

    set_cjk_priority(&DEFAULT_CJK_PRIORITY);
    let mut ja = Buffer::from_str(han_only);
    drive_act(&mut ja, &Action::TagDocumentLanguage);

    set_cjk_priority(&[Lang::ZhHans, Lang::Ja, Lang::ZhHant, Lang::Ko]);
    let mut zh = Buffer::from_str(han_only);
    drive_act(&mut zh, &Action::TagDocumentLanguage);

    assert_eq!(ja.text(), format!("---\nlang: ja\n---\n{han_only}"));
    assert_eq!(zh.text(), format!("---\nlang: zh-Hans\n---\n{han_only}"));
    assert_ne!(
        ja.text(),
        zh.text(),
        "the ladder must actually decide the stamped tag"
    );

    // An UNAMBIGUOUS script ignores the ladder outright (kana is Japanese
    // however the tiebreak is ordered).
    let mut kana = Buffer::from_str("かな\n");
    drive_act(&mut kana, &Action::TagDocumentLanguage);
    assert_eq!(kana.text(), "---\nlang: ja\n---\nかな\n");

    // NO CJK: nothing to name, so nothing is written and nothing is undoable.
    let latin = "Just some ordinary English prose.\n";
    let mut plain = Buffer::from_str(latin);
    let eff = drive_act_effect(&mut plain, &Action::TagDocumentLanguage);
    assert_eq!(eff, Effect::None);
    assert_eq!(plain.text(), latin, "a pure-Latin document is never tagged");
    assert!(!plain.can_undo(), "…so there is nothing to undo");

    set_cjk_priority(&restore);
}

#[test]
fn tag_document_language_never_touches_a_non_markdown_buffer() {
    // Frontmatter is a markdown/notes convention: literal `---`/`lang:` text
    // at the top of a `.rs` file is corruption, not metadata.
    use crate::fs::InMemoryFs;
    let _g = crate::testlock::serial();
    let p = std::path::PathBuf::from("/proj/main.rs");
    let src = "fn main() {\n    println!(\"こんにちは\");\n}\n";
    let _fs =
        crate::fs::FsGuard::install(std::sync::Arc::new(InMemoryFs::new().with_file(&p, src)));
    let mut b = Buffer::from_file(&p);
    assert!(!b.is_markdown(), "arranged: a code buffer");
    drive_act(&mut b, &Action::TagDocumentLanguage);
    assert_eq!(b.text(), src, "a code file is never tagged");
    assert!(!b.can_undo());
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

/// THE REGRESSION LAW: the popover's Code button does not own a
/// private edit path — `PopoverButton::action()` is the SAME catalog Action
/// the keyboard/palette route fires (law-tested,
/// `commands::tests::every_popover_button_fires_a_catalog_command`), and the
/// live click handler dispatches it through the identical `App::apply` seam
/// (`app/input/mouse.rs`: `self.apply(button.action(), …)`). So driving
/// `PopoverButton::Code.action()` through `apply_transition` here IS the real
/// popover route, not a stand-in for it — premise-checked before the fix
/// landed: a bare-two-line selection already round-tripped on `main`; the
/// selection that reproduced the report crosses a BLANK line, where a
/// backtick-delimited code span cannot exist in CommonMark at all, so the
/// shared parser-backed "already wrapped" check could never confirm what the
/// FIRST press had just inserted.
#[test]
fn inline_code_popover_route_two_presses_round_trip_with_undo_redo() {
    let action = crate::popover::PopoverButton::Code.action();
    assert_eq!(
        action,
        Action::InlineCode,
        "the popover fires the catalog Action"
    );

    let src = "one\n\ntwo\n"; // the wrapped selection crosses a blank line
    let mut b = drive_format(src, Some(0), 8, &action);
    assert_eq!(b.text(), "`one\n\ntwo`\n", "first press wraps");
    let (a1, c1) = b
        .selection_range()
        .expect("selection over the wrapped text");

    // The popover's OWN lit oracle — the exact predicate the render row reads
    // every frame — must agree the button is active on the wrapped selection;
    // a dark button here is the visible half of the same defect.
    let plan = crate::actions::popover::plan(&b.text(), Some(a1), c1, true)
        .expect("a markdown buffer summons the popover");
    let code_lit = plan
        .buttons
        .iter()
        .find(|s| s.button == crate::popover::PopoverButton::Code)
        .expect("Code is a popover button")
        .active;
    assert!(code_lit, "code button must be LIT on the wrapped selection");

    // Second press, through the SAME real dispatch: strips back exactly.
    let re = drive_format(&b.text(), Some(a1), c1, &action);
    assert_eq!(
        re.text(),
        src,
        "second press strips back to the exact original"
    );
    assert_eq!(re.selection_range(), Some((0, 8)), "selection restored");

    // ONE undo of the FIRST press's toggle restores the pre-wrap text (the
    // whole replace-and-reselect is one atomic group); redo re-applies it.
    b.undo();
    assert_eq!(b.text(), src, "one Cmd-Z reverts the wrap");
    b.redo();
    assert_eq!(
        b.text(),
        "`one\n\ntwo`\n",
        "redo re-applies the same single edit"
    );
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
fn tab_indents_a_tab_indented_list_line_not_a_soft_tab() {
    // Regression: `list_item`'s indent scan used to accept spaces only, so a
    // TAB-indented bullet failed the Tab/Shift-Tab list gate
    // (`selection_or_cursor_on_list`) and fell through to a soft-tab insert —
    // even though Enter on the SAME line happily continued it as a list
    // (`smart_newline_for`'s own indent scan already accepted tabs). Driven
    // through the real `apply_transition` seam exactly as `--keys "Tab"` would.
    let mut b = md("\t- item", 7); // caret at end of line
    drive_act(&mut b, &Action::InsertTab);
    assert_eq!(
        b.text(),
        "  \t- item",
        "Tab indents the tab-indented bullet by one nesting level, not a soft-tab"
    );

    // Same gate, ordered and task variants — every marker kind list_item covers.
    let mut ordered = md("\t1. item", 8);
    drive_act(&mut ordered, &Action::InsertTab);
    assert_eq!(
        ordered.text(),
        "  \t1. item",
        "tab-indented ordered item indents"
    );

    let mut task = md("\t- [ ] item", 11);
    drive_act(&mut task, &Action::InsertTab);
    assert_eq!(
        task.text(),
        "  \t- [ ] item",
        "tab-indented task item indents"
    );
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
fn tab_gate_and_enter_continuation_agree_on_is_list_line() {
    // Parity law: `list_item` is the documented SHARED list-detection primitive
    // behind BOTH the Tab/Shift-Tab indent gate (`selection_or_cursor_on_list`,
    // which calls it directly) and the Enter continuation (`smart_newline_for`,
    // which now routes its marker detection through it too). Sweep a corpus of
    // tab-indented bullets/ordered/task items and near-misses and assert the two
    // consumers can never disagree on whether a line IS a list item. Caret is
    // parked at the line's own end, which — for any real list marker — always
    // sits at or past the marker (so `smart_newline_for` never declines on a
    // caret-inside-marker technicality here).
    let corpus = [
        "- item",
        "* item",
        "+ item",
        "\t- tab-indented bullet",
        "\t\t- double-tab-indented bullet",
        "  - space-indented bullet",
        "\t* tab-indented star",
        "\t+ tab-indented plus",
        " \t- mixed space-then-tab bullet",
        "\t - tab-then-space bullet",
        "1. ordered",
        "12) ordered paren",
        "\t1. tab-indented ordered",
        "\t12) tab-indented ordered paren",
        "- [ ] task",
        "- [x] task checked",
        "\t- [ ] tab-indented task",
        "\t- [X] tab-indented task, capital X",
        "-", // bare dash, no trailing space — not a list
        "-nope",
        "12 monkeys", // digits + space with no '.'/')' delimiter — not a list
        "just prose",
        "",
        "\t", // only indentation, nothing else
        "\tjust indented prose",
        "\t\t\tdeeply indented prose",
    ];
    for line in corpus {
        let is_list = crate::markdown::list_item(line).is_some();
        let col = line.chars().count();
        let enter_is_list = matches!(
            smart_newline_for(line, col),
            Some(SmartNewline::ContinueListItem { .. }) | Some(SmartNewline::EmptyListItem { .. })
        );
        assert_eq!(
            is_list, enter_is_list,
            "list_item and smart_newline_for disagree on is-list for {line:?}"
        );
    }
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

#[test]
fn insert_footnote_is_one_undoable_edit_through_the_real_action_seam() {
    let source = "A note here.";
    let mut buffer = drive_format(source, None, 6, &Action::InsertFootnote);
    assert_eq!(buffer.text(), "A note[^1] here.\n\n[^1]: ");
    assert!(buffer.can_undo());
    buffer.undo();
    assert_eq!(buffer.text(), source, "one undo restores every source byte");
}
