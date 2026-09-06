//! The six STRUCTURAL table verbs at the `apply_transition` seam — the same
//! door a palette pick, a menu click and a `--keys` replay all come through.
//!
//! The pure splice's own boundary sweep lives in `markdown::table_edit`; what
//! is asserted here is what only the action layer owns: the shared
//! caret-in-table gate, the sealed one-undo-per-verb group, where the caret
//! lands in the real buffer, and the refusal notices reaching the effect
//! channel the sidecar publishes.

use super::super::*;
use super::{drive_act, drive_act_effect, md};

/// Every structural verb, so a sweep cannot quietly omit one.
const VERBS: [Action; 6] = [
    Action::TableInsertRowAbove,
    Action::TableInsertRowBelow,
    Action::TableInsertColumnLeft,
    Action::TableInsertColumnRight,
    Action::TableDeleteRow,
    Action::TableDeleteColumn,
];

/// prose, a table, prose — the caret starts on the body row `| a | 100 |`.
fn doc() -> Buffer {
    let src = "intro\n| Name | V |\n| --- | --- |\n| a | 100 |\n| b | 2 |\ntail\n";
    let mut b = md(src, 0);
    b.set_cursor(b.line_col_to_char(3, 2));
    b
}

fn notice_text(effect: &Effect) -> Option<&str> {
    match effect {
        Effect::Notice(notice) => notice.message(),
        _ => None,
    }
}

/// EVERY verb is ONE sealed undo group: a single Cmd-Z restores the exact
/// pre-verb source, and the edit that follows it is its own step rather than
/// coalescing backwards into the verb. Swept over all six, so a verb that
/// splits its work into two edits (or lets the auto-re-pad fire as a second
/// one) fails here.
#[test]
fn every_verb_is_one_undoable_edit_and_never_coalesces_with_what_follows() {
    for verb in VERBS {
        let mut b = doc();
        let before = b.text();
        let effect = drive_act_effect(&mut b, &verb);
        assert_eq!(effect, Effect::None, "{verb:?} refused unexpectedly");
        let after = b.text();
        assert_ne!(after, before, "{verb:?} edited nothing");
        assert!(
            after.starts_with("intro\n") && after.ends_with("tail\n"),
            "{verb:?} disturbed the surrounding prose: {after:?}"
        );
        // A keystroke straight after the verb is its own step…
        drive_act(&mut b, &Action::InsertChar('z'));
        b.undo();
        assert_eq!(
            b.text(),
            after,
            "{verb:?}: undo #1 takes back only the typing"
        );
        // …and ONE more Cmd-Z takes the whole verb back.
        b.undo();
        assert_eq!(
            b.text(),
            before,
            "{verb:?}: undo #2 restores the pre-verb source"
        );
        assert!(!b.can_undo(), "{verb:?}: nothing else was recorded");
    }
}

/// The caret lands IN the cell the verb opened, ready to type — asserted by
/// typing and reading back where the character actually landed, not by
/// trusting a column number.
#[test]
fn the_caret_lands_in_the_cell_the_verb_opened() {
    // Insert row below, from the body row: the caret is in the new row's first
    // cell.
    let mut b = doc();
    drive_act(&mut b, &Action::TableInsertRowBelow);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        crate::markdown::split_row_cells(&b.line_text(4)),
        vec!["x".to_string(), String::new()],
        "the new row is line 4 and 'x' went into its first cell: {:?}",
        b.text()
    );

    // Insert column right, caret in column 0: the caret is in the new column 1.
    let mut b = doc();
    drive_act(&mut b, &Action::TableInsertColumnRight);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        crate::markdown::split_row_cells(&b.line_text(3)),
        vec!["a".to_string(), "x".into(), "100".into()],
        "the blank column opened right of 'a' and took the caret: {:?}",
        b.text()
    );

    // Insert column left, same caret: the new column is 0 and 'a' moved right.
    let mut b = doc();
    drive_act(&mut b, &Action::TableInsertColumnLeft);
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        crate::markdown::split_row_cells(&b.line_text(3)),
        vec!["x".to_string(), "a".into(), "100".into()]
    );

    // Delete row: the row below slides up under the caret, same column.
    let mut b = doc();
    drive_act(&mut b, &Action::TableDeleteRow);
    assert_eq!(b.cursor_line_col().0, 3, "the caret stays on line 3");
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        crate::markdown::split_row_cells(&b.line_text(3)),
        vec!["xb".to_string(), "2".into()],
        "the caret sits at the START of the row that slid up: {:?}",
        b.text()
    );
}

/// OFF a table — on prose, on a pipe-bearing line that is not a table, and on
/// a non-markdown buffer — every verb refuses OUT LOUD and edits nothing. A
/// palette row that is always listed owes the writer a reason, not silence.
#[test]
fn off_a_table_every_verb_refuses_out_loud_and_edits_nothing() {
    // (what, source, path, caret line, caret column)
    let cases: [(&str, &str, &str, usize, usize); 3] = [
        (
            "prose",
            "intro\n| Name | V |\n| --- | --- |\n| a | 100 |\ntail\n",
            "note.md",
            0,
            2,
        ),
        (
            "a pipe-bearing line with no separator row",
            "a | b\nc | d\n",
            "note.md",
            0,
            2,
        ),
        (
            "a non-markdown buffer",
            "| a | b |\n| --- | --- |\n| 1 | 2 |\n",
            "notes.txt",
            2,
            2,
        ),
    ];
    for (what, src, path, line, col) in cases {
        for verb in VERBS {
            let mut b = Buffer::from_str(src);
            b.set_path(std::path::PathBuf::from(path));
            b.set_cursor(b.line_col_to_char(line, col));
            let before = b.text();
            let effect = drive_act_effect(&mut b, &verb);
            assert_eq!(
                notice_text(&effect),
                Some("put the caret in a table first"),
                "{verb:?} on {what} said nothing"
            );
            assert_eq!(b.text(), before, "{verb:?} on {what} edited the buffer");
            assert!(!b.can_undo(), "{verb:?} on {what} recorded undo history");
        }
    }
}

/// THE SHARED GATE. Align table and the structural verbs answer the SAME
/// question about where a table is: swept over every line of a document that
/// mixes prose, a pipe-bearing run with no separator, and a real (deliberately
/// misaligned) table, the line on which Align table finds work is exactly the
/// line on which a verb does NOT report "no table". A second caret-in-table
/// predicate anywhere would disagree on one of these lines.
#[test]
fn align_table_and_the_structural_verbs_share_one_caret_in_table_gate() {
    // Line 0 prose; 1-2 a pipe run with NO separator (not a table); 3 blank;
    // 4-7 a real, misaligned table; 8 prose.
    let src = "intro\na | b\nc | d\n\n|Name|V|\n|---|---|\n|a|100|\n|b|2|\ntail\n";
    let lines = src.split('\n').count();
    let mut reached = Vec::new();
    for line in 0..lines {
        let mut align = md(src, 0);
        align.set_cursor(align.line_col_to_char(line, 0));
        let before = align.text();
        drive_act(&mut align, &Action::AlignTable);
        let align_reached = align.text() != before;

        for verb in VERBS {
            let mut b = md(src, 0);
            b.set_cursor(b.line_col_to_char(line, 0));
            let effect = drive_act_effect(&mut b, &verb);
            let verb_reached = notice_text(&effect) != Some("put the caret in a table first");
            assert_eq!(
                verb_reached,
                align_reached,
                "line {line} ({:?}): Align table reached it = {align_reached}, {verb:?} reached \
                 it = {verb_reached} — the two are reading different gates",
                src.split('\n').nth(line).unwrap_or("")
            );
        }
        if align_reached {
            reached.push(line);
        }
    }
    assert_eq!(
        reached,
        vec![4, 5, 6, 7],
        "the gate enrolled the wrong lines — this law would be vacuous if it enrolled none"
    );
}

/// A verb DECLINED by the table's own shape reports the pure splice's own
/// sentence, unchanged — one owner of the words, so a notice can never drift
/// from the rule it describes. Swept over every refusal the six verbs can
/// raise.
#[test]
fn every_structural_refusal_surfaces_the_splices_own_sentence() {
    use crate::markdown::TableRefusal;
    // (caret line, verb, expected refusal) over a two-column, two-body-row
    // table plus a one-column table for the last-column case.
    let table = "| Name | V |\n| --- | --- |\n| a | 100 |\n| b | 2 |\n";
    let one_col = "| Name |\n| --- |\n| a |\n";
    let cases: [(&str, usize, Action, TableRefusal); 5] = [
        (
            table,
            0,
            Action::TableInsertRowAbove,
            TableRefusal::HeaderIsFirstRow,
        ),
        (
            table,
            1,
            Action::TableInsertRowAbove,
            TableRefusal::HeaderIsFirstRow,
        ),
        (
            table,
            0,
            Action::TableDeleteRow,
            TableRefusal::HeaderRowIsStructural,
        ),
        (
            table,
            1,
            Action::TableDeleteRow,
            TableRefusal::HeaderRowIsStructural,
        ),
        (
            one_col,
            2,
            Action::TableDeleteColumn,
            TableRefusal::OnlyColumn,
        ),
    ];
    for (src, line, verb, want) in cases {
        let mut b = md(src, 0);
        b.set_cursor(b.line_col_to_char(line, 1));
        let before = b.text();
        let effect = drive_act_effect(&mut b, &verb);
        assert_eq!(
            notice_text(&effect),
            Some(TableRefusal::message(want)),
            "{verb:?} on line {line} of {src:?}"
        );
        assert_eq!(b.text(), before, "a refusal edits nothing");
        assert!(!b.can_undo(), "a refusal records no undo history");
    }
}

/// The auto-re-pad on row-leave does not fight a splice: after any verb the
/// block is already what the padder would produce, so nothing re-pads it a
/// second time — checked by driving a caret motion OUT of the table afterwards
/// and finding the text unchanged.
#[test]
fn a_caret_leaving_the_table_after_a_verb_finds_nothing_left_to_re_pad() {
    for verb in VERBS {
        let mut b = doc();
        drive_act(&mut b, &verb);
        let settled = b.text();
        // Walk the caret off the table's row (and then out of the block
        // entirely) — the row-leave trigger fires on each step.
        for _ in 0..4 {
            drive_act(&mut b, &Action::NextLine);
        }
        assert_eq!(
            b.text(),
            settled,
            "{verb:?}: the auto-re-pad rewrote the spliced block"
        );
    }
}

/// A verb reached from a SELECTION does not silently swallow it: the selection
/// is cleared and the caret lands in the verb's own cell, exactly as it does
/// from a bare caret.
#[test]
fn a_verb_clears_a_selection_rather_than_editing_around_it() {
    let mut b = doc();
    let start = b.line_col_to_char(3, 2);
    b.select_range(start, b.line_col_to_char(3, 3));
    drive_act(&mut b, &Action::TableInsertRowBelow);
    assert!(!b.has_selection(), "the selection is gone");
    drive_act(&mut b, &Action::InsertChar('x'));
    assert_eq!(
        crate::markdown::split_row_cells(&b.line_text(4)),
        vec!["x".to_string(), String::new()],
        "and the caret is in the new row: {:?}",
        b.text()
    );
}
