//! Laws over the structural table splices. Pure functions, no globals — the
//! same seam `table_caret.rs`'s own tests sit at.
//!
//! The sweeps here are deliberately EXHAUSTIVE over the boundary axis (every
//! row × every column × every verb, on every table shape that has a boundary:
//! 1×1, one-row, one-column, ragged) rather than a handful of hand-picked
//! positions, because a splice's bugs live at its ends.

use super::*;
use crate::markdown::table_caret::locate_table_caret;
use crate::markdown::tables::{ColAlign, align_table, parse_col_align, split_row_cells};

/// Every verb, so a sweep can never quietly omit one (no wildcard: a new
/// [`TableVerb`] fails to compile here until it is enrolled).
const VERBS: [TableVerb; 6] = [
    TableVerb::InsertRowAbove,
    TableVerb::InsertRowBelow,
    TableVerb::InsertColumnLeft,
    TableVerb::InsertColumnRight,
    TableVerb::DeleteRow,
    TableVerb::DeleteColumn,
];

fn verb_name(verb: TableVerb) -> &'static str {
    match verb {
        TableVerb::InsertRowAbove => "InsertRowAbove",
        TableVerb::InsertRowBelow => "InsertRowBelow",
        TableVerb::InsertColumnLeft => "InsertColumnLeft",
        TableVerb::InsertColumnRight => "InsertColumnRight",
        TableVerb::DeleteRow => "DeleteRow",
        TableVerb::DeleteColumn => "DeleteColumn",
    }
}

/// The per-column alignment a block actually carries, read back off its
/// SEPARATOR line the same way the padder does.
fn aligns_of(block: &str) -> Vec<ColAlign> {
    let sep = block.split('\n').nth(1).expect("a table has a separator");
    split_row_cells(sep)
        .iter()
        .map(|cell| parse_col_align(cell))
        .collect()
}

/// Every content row's trimmed cells, separator lifted out — the grid a reader
/// sees.
fn grid_of(block: &str) -> Vec<Vec<String>> {
    block
        .split('\n')
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, line)| split_row_cells(line))
        .collect()
}

fn splice(block: &str, row: usize, cell: usize, verb: TableVerb) -> TableSplice {
    table_splice(block, row, TableCaretCell { cell, offset: 0 }, verb)
        .unwrap_or_else(|refusal| panic!("{} refused: {refusal:?}", verb_name(verb)))
}

/// A 2-column × 3-content-row table with a distinct marker per column, so a
/// column that moves is visibly traceable.
const T2X3: &str = "| a | b |\n| :-- | --: |\n| 1 | 2 |\n| 3 | 4 |";

// ---------------------------------------------------------------------------
// HEADLINE LAW: alignment markers survive every splice, at every position.
// ---------------------------------------------------------------------------

/// Per-column `:` markers are DATA that rides its column: inserting a column
/// shifts every marker to its right, deleting one takes only its own marker,
/// and a row splice touches none of them. Swept over every verb × every caret
/// row × every caret column of a four-column table carrying all four distinct
/// alignments, so no marker value and no position is left un-probed.
#[test]
fn alignment_markers_ride_their_columns_through_every_splice_position() {
    // One column per ColAlign variant, in a fixed order, so a shift by one is
    // never masked by two neighbours agreeing.
    let block =
        "| n | l | r | c |\n| --- | :-- | --: | :-: |\n| 1 | 2 | 3 | 4 |\n| 5 | 6 | 7 | 8 |";
    let base = [
        ColAlign::None,
        ColAlign::Left,
        ColAlign::Right,
        ColAlign::Center,
    ];
    assert_eq!(aligns_of(block), base, "the fixture's own markers parse");
    let rows = block.split('\n').count(); // 4 source lines: header, sep, 2 body
    for verb in VERBS {
        for row in 0..rows {
            for cell in 0..base.len() {
                let Ok(out) = table_splice(block, row, TableCaretCell { cell, offset: 0 }, verb)
                else {
                    continue; // a documented refusal; its own law covers it
                };
                let got = aligns_of(&out.text);
                let want: Vec<ColAlign> = match verb {
                    TableVerb::InsertRowAbove
                    | TableVerb::InsertRowBelow
                    | TableVerb::DeleteRow => base.to_vec(),
                    TableVerb::InsertColumnLeft => {
                        let mut w = base.to_vec();
                        w.insert(cell, ColAlign::None);
                        w
                    }
                    TableVerb::InsertColumnRight => {
                        let mut w = base.to_vec();
                        w.insert(cell + 1, ColAlign::None);
                        w
                    }
                    TableVerb::DeleteColumn => {
                        let mut w = base.to_vec();
                        w.remove(cell);
                        w
                    }
                };
                assert_eq!(
                    got,
                    want,
                    "{} at row {row} cell {cell} moved the alignment markers\n{out_text}",
                    verb_name(verb),
                    out_text = out.text
                );
            }
        }
    }
}

/// A column splice moves CONTENT with its markers, not only the markers — the
/// companion presence check, so the law above cannot be satisfied by a table
/// whose cells all emptied out.
#[test]
fn a_column_splice_carries_its_cell_contents_too() {
    let block = "| a | b | c |\n| --- | --- | --- |\n| 1 | 2 | 3 |";
    let out = splice(block, 2, 1, TableVerb::InsertColumnLeft);
    assert_eq!(
        grid_of(&out.text),
        vec![
            vec!["a".to_string(), String::new(), "b".into(), "c".into()],
            vec!["1".to_string(), String::new(), "2".into(), "3".into()],
        ],
        "the blank column opens at index 1 and pushes b/c right"
    );
    let out = splice(block, 2, 1, TableVerb::DeleteColumn);
    assert_eq!(
        grid_of(&out.text),
        vec![
            vec!["a".to_string(), "c".into()],
            vec!["1".to_string(), "3".into()],
        ],
        "only column 1 leaves; a and c close up"
    );
}

// ---------------------------------------------------------------------------
// Row verbs, swept over every row of a real table.
// ---------------------------------------------------------------------------

/// Insert-row-below works from EVERY row — header, separator and every body
/// row — and always opens exactly one blank row in the right place, with the
/// caret in its first cell. From the header or its separator the new row is
/// the FIRST BODY row (below the separator), the same "header and separator
/// are one unit" rule `table_newline` follows.
#[test]
fn insert_row_below_opens_one_blank_row_from_every_row_including_the_header() {
    for row in 0..4 {
        let out = splice(T2X3, row, 0, TableVerb::InsertRowBelow);
        let grid = grid_of(&out.text);
        assert_eq!(grid.len(), 4, "one row added (from row {row})");
        // Which grid row is the blank one: below the header (grid 1) for rows
        // 0/1, below body row n for the rest.
        let want_blank = if row <= 1 { 1 } else { row - 1 + 1 };
        assert_eq!(
            grid[want_blank],
            vec![String::new(), String::new()],
            "the blank row lands at grid row {want_blank} (from row {row}): {:?}",
            out.text
        );
        assert_eq!(
            out.row,
            want_blank + 1,
            "the caret's SOURCE line skips the separator (from row {row})"
        );
        assert_eq!(out.caret, TableCaretCell { cell: 0, offset: 0 });
        // The pre-existing content is intact and in order.
        let content: Vec<Vec<String>> = grid
            .iter()
            .filter(|r| r.iter().any(|c| !c.is_empty()))
            .cloned()
            .collect();
        assert_eq!(
            content,
            grid_of(T2X3),
            "no existing row was disturbed (from row {row})"
        );
    }
}

/// Insert-row-above works from every BODY row and refuses on the header and on
/// its separator, naming the reason.
#[test]
fn insert_row_above_refuses_only_on_the_header_unit() {
    for row in [0usize, 1] {
        assert_eq!(
            table_splice(
                T2X3,
                row,
                TableCaretCell { cell: 0, offset: 0 },
                TableVerb::InsertRowAbove
            )
            .err(),
            Some(TableRefusal::HeaderIsFirstRow),
            "row {row} is the header unit"
        );
    }
    for row in 2..4 {
        let out = splice(T2X3, row, 0, TableVerb::InsertRowAbove);
        let grid = grid_of(&out.text);
        assert_eq!(grid.len(), 4);
        let want_blank = row - 1;
        assert_eq!(
            grid[want_blank],
            vec![String::new(), String::new()],
            "the blank row takes the caret row's place (from row {row})"
        );
        assert_eq!(out.row, row, "the caret follows the new blank row");
        assert_eq!(out.caret, TableCaretCell { cell: 0, offset: 0 });
    }
}

/// Delete-row removes exactly the caret's own row from every body position,
/// refuses on the header unit, and — deleting the LAST body row — leaves a
/// header + separator, which is still a valid GFM table.
#[test]
fn delete_row_empties_the_body_but_never_the_table() {
    for row in [0usize, 1] {
        assert_eq!(
            table_splice(
                T2X3,
                row,
                TableCaretCell { cell: 0, offset: 0 },
                TableVerb::DeleteRow
            )
            .err(),
            Some(TableRefusal::HeaderRowIsStructural),
            "row {row} is the header unit"
        );
    }
    // Body rows: each one deletes itself, leaving the other.
    let out = splice(T2X3, 2, 1, TableVerb::DeleteRow);
    assert_eq!(
        grid_of(&out.text),
        vec![
            vec!["a".to_string(), "b".into()],
            vec!["3".to_string(), "4".into()]
        ]
    );
    assert_eq!(out.row, 2, "the row below slid up into the caret's place");
    assert_eq!(out.caret, TableCaretCell { cell: 1, offset: 0 });
    let out = splice(T2X3, 3, 0, TableVerb::DeleteRow);
    assert_eq!(
        grid_of(&out.text),
        vec![
            vec!["a".to_string(), "b".into()],
            vec!["1".to_string(), "2".into()]
        ]
    );
    assert_eq!(out.row, 2, "the caret stays on the new last row");

    // Emptying the body entirely: delete both rows, one after the other.
    let once = splice(T2X3, 2, 0, TableVerb::DeleteRow);
    let twice = splice(&once.text, once.row, 0, TableVerb::DeleteRow);
    assert_eq!(
        twice.text.split('\n').count(),
        2,
        "a header + separator survives: {:?}",
        twice.text
    );
    assert_eq!(twice.row, 0, "the caret falls back onto the header");
    assert_eq!(aligns_of(&twice.text), aligns_of(T2X3), "markers survive");
    assert_eq!(
        table_splice(
            &twice.text,
            0,
            TableCaretCell { cell: 0, offset: 0 },
            TableVerb::DeleteRow
        )
        .err(),
        Some(TableRefusal::HeaderRowIsStructural),
        "a body-less table declines to delete itself"
    );
}

// ---------------------------------------------------------------------------
// Degenerate shapes: 1x1, one column, one row.
// ---------------------------------------------------------------------------

/// The smallest legal table (one header cell, no body). Every verb either does
/// the obvious thing or refuses for a named reason — nothing panics, nothing
/// produces a non-table.
#[test]
fn a_one_by_one_table_answers_every_verb() {
    let block = "| a |\n| --- |";
    assert_eq!(
        table_splice(
            block,
            0,
            TableCaretCell { cell: 0, offset: 0 },
            TableVerb::InsertRowAbove
        )
        .err(),
        Some(TableRefusal::HeaderIsFirstRow)
    );
    assert_eq!(
        table_splice(
            block,
            0,
            TableCaretCell { cell: 0, offset: 0 },
            TableVerb::DeleteRow
        )
        .err(),
        Some(TableRefusal::HeaderRowIsStructural)
    );
    assert_eq!(
        table_splice(
            block,
            0,
            TableCaretCell { cell: 0, offset: 0 },
            TableVerb::DeleteColumn
        )
        .err(),
        Some(TableRefusal::OnlyColumn),
        "deleting the only column would leave no table"
    );
    let out = splice(block, 0, 0, TableVerb::InsertRowBelow);
    assert_eq!(
        grid_of(&out.text),
        vec![vec!["a".to_string()], vec![String::new()]]
    );
    assert_eq!(out.row, 2);
    let out = splice(block, 0, 0, TableVerb::InsertColumnRight);
    assert_eq!(
        grid_of(&out.text),
        vec![vec!["a".to_string(), String::new()]]
    );
    assert_eq!(out.caret, TableCaretCell { cell: 1, offset: 0 });
    let out = splice(block, 0, 0, TableVerb::InsertColumnLeft);
    assert_eq!(
        grid_of(&out.text),
        vec![vec![String::new(), "a".to_string()]]
    );
    assert_eq!(out.caret, TableCaretCell { cell: 0, offset: 0 });
}

/// Delete-column at every column of a table, including the LAST one — the
/// caret slides back onto the new last column rather than off the end — and
/// the refusal fires only when the last column would go.
#[test]
fn delete_column_sweeps_every_column_and_refuses_only_the_last_one() {
    let mut block = "| a | b | c |\n| :-- | --: | :-: |\n| 1 | 2 | 3 |".to_string();
    // Every column, deleted from the right-hand end inward, so the "caret was
    // on the last column" case is hit at every width down to 1.
    for want_cols in (1..3).rev() {
        let cell = want_cols; // the last column's index before this delete
        let out = splice(&block, 2, cell, TableVerb::DeleteColumn);
        assert_eq!(
            grid_of(&out.text)[0].len(),
            want_cols,
            "one column left from {block:?}"
        );
        assert_eq!(
            out.caret,
            TableCaretCell {
                cell: want_cols - 1,
                offset: 0
            },
            "the caret lands on the new last column"
        );
        block = out.text;
    }
    assert_eq!(
        table_splice(
            &block,
            2,
            TableCaretCell { cell: 0, offset: 0 },
            TableVerb::DeleteColumn
        )
        .err(),
        Some(TableRefusal::OnlyColumn)
    );
}

/// A run of pipe-bearing lines whose SECOND line is not a separator is not a
/// GFM table for splicing purposes, and every verb says so rather than
/// rewriting it inside out. (`table_block_lines` only demands a separator
/// somewhere in the run, so this shape genuinely reaches here.)
#[test]
fn a_block_whose_second_line_is_not_a_separator_is_refused_by_every_verb() {
    for block in ["| a | b |\n| c | d |\n| --- | --- |", "| a | b |"] {
        for verb in VERBS {
            assert_eq!(
                table_splice(block, 0, TableCaretCell { cell: 0, offset: 0 }, verb).err(),
                Some(TableRefusal::NotAGfmTable),
                "{} on {block:?}",
                verb_name(verb)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Content-shape axes: ragged rows, escapes, CJK, pipe-less edges.
// ---------------------------------------------------------------------------

/// Ragged rows (short AND long) normalize to the table's column count — the
/// same thing an ordinary re-pad already does — and the splice lands at the
/// right index on every row, not just the caret's own.
#[test]
fn ragged_rows_normalize_and_the_splice_reaches_every_one_of_them() {
    // Row 2 is short (one cell), row 3 is LONG (three cells) — so the column
    // count comes from a body row, not the header.
    let block = "| a | b |\n| --- | --- |\n| 1 |\n| 3 | 4 | 5 |";
    let out = splice(block, 2, 0, TableVerb::InsertColumnRight);
    assert_eq!(
        grid_of(&out.text),
        vec![
            vec!["a".to_string(), String::new(), "b".into(), String::new()],
            vec!["1".to_string(), String::new(), String::new(), String::new()],
            vec!["3".to_string(), String::new(), "4".into(), "5".into()],
        ],
        "every row is the same width and the new column opened at index 1"
    );
    assert_eq!(aligns_of(&out.text).len(), 4, "the separator widened too");
    // And a row verb normalizes the same way.
    let out = splice(block, 2, 0, TableVerb::InsertRowBelow);
    for row in grid_of(&out.text) {
        assert_eq!(row.len(), 3, "every row is 3 wide: {:?}", out.text);
    }
}

/// An escaped pipe is CELL CONTENT (GFM requires the escape even inside a code
/// span), so a splice must carry it through untouched and must not treat it as
/// a column boundary.
#[test]
fn an_escaped_pipe_stays_inside_its_cell_across_a_column_splice() {
    let block = "| a \\| b | `c \\| d` |\n| --- | --- |\n| 1 | 2 |";
    assert_eq!(
        grid_of(block)[0],
        vec!["a \\| b".to_string(), "`c \\| d`".into()],
        "the fixture is two cells, not four"
    );
    let out = splice(block, 0, 0, TableVerb::InsertColumnRight);
    assert_eq!(
        grid_of(&out.text)[0],
        vec!["a \\| b".to_string(), String::new(), "`c \\| d`".into()],
        "both escapes survive verbatim: {:?}",
        out.text
    );
    // The escaped pipes are still escaped in the SOURCE (not doubled, not eaten).
    assert_eq!(out.text.matches("\\|").count(), 2);
}

/// CJK cells: the splice hands its output to the one padder, so the wide-glyph
/// width rule is the padder's — the result is a fixed point under
/// `align_table`, which is exactly what "no second padder" means.
#[test]
fn cjk_cells_come_back_at_the_padders_own_widths() {
    let block = "| 名前 | b |\n| --- | --- |\n| 東京 | 2 |";
    let out = splice(block, 2, 1, TableVerb::InsertColumnRight);
    assert_eq!(
        align_table(&out.text),
        out.text,
        "the spliced block is already what the padder would produce: {:?}",
        out.text
    );
    assert!(out.text.contains("名前"), "content intact");
}

/// A table written without outer pipes still splices — and comes back in the
/// padder's canonical form, the same normalization Align table performs.
#[test]
fn a_table_without_outer_pipes_splices_and_normalizes() {
    let block = "a | b\n--- | ---\n1 | 2";
    let out = splice(block, 2, 0, TableVerb::InsertColumnRight);
    assert_eq!(
        out.text, "| a |   | b |\n| - | - | - |\n| 1 |   | 2 |",
        "outer pipes are added by the one padder"
    );
}

// ---------------------------------------------------------------------------
// Invariants that must hold for EVERY verb at EVERY position.
// ---------------------------------------------------------------------------

/// Whatever a verb does, its output is (a) already aligned — so the auto-re-pad
/// that fires on row-leave finds nothing to do and cannot fight the splice —
/// (b) still a GFM table with a separator on its second line, and (c) carries a
/// caret row that actually exists with a resolvable column on it.
#[test]
fn every_splice_lands_aligned_still_a_table_and_with_a_reachable_caret() {
    let fixtures = [
        T2X3,
        "| a |\n| --- |",
        "| a | b |\n| --- | --- |",                       // no body
        "| a |\n| :-: |\n| 1 |\n| 2 |\n| 3 |",            // one column, three rows
        "| a | b |\n| --- | --- |\n| 1 |\n| 3 | 4 | 5 |", // ragged
        "| 名前 | b |\n| --- | --- |\n| 東京 | 2 |",      // wide graphemes
        "|  |  |\n| --- | --- |\n|  |  |",                // all cells empty
    ];
    for block in fixtures {
        let lines = block.split('\n').count();
        let cols = grid_of(block).iter().map(Vec::len).max().unwrap_or(1);
        for verb in VERBS {
            for row in 0..lines {
                for cell in 0..cols {
                    let Ok(out) =
                        table_splice(block, row, TableCaretCell { cell, offset: 0 }, verb)
                    else {
                        continue;
                    };
                    let what = format!("{} at row {row} cell {cell} of {block:?}", verb_name(verb));
                    assert_eq!(
                        align_table(&out.text),
                        out.text,
                        "{what}: output is not a fixed point of the padder\n{:?}",
                        out.text
                    );
                    let out_lines: Vec<&str> = out.text.split('\n').collect();
                    assert!(out_lines.len() >= 2, "{what}: lost the separator");
                    assert!(
                        crate::markdown::tables::is_separator_row(out_lines[1]),
                        "{what}: line 1 is no longer a separator: {:?}",
                        out_lines[1]
                    );
                    assert!(
                        out.row < out_lines.len(),
                        "{what}: caret row {} is past the block",
                        out.row
                    );
                    assert_ne!(out.row, 1, "{what}: the caret landed on the separator");
                    // The caret's logical position resolves to a real column and
                    // round-trips back to the same cell.
                    let line = out_lines[out.row];
                    let col = crate::markdown::table_caret_col(line, out.caret);
                    assert_eq!(
                        locate_table_caret(line, col).cell,
                        out.caret.cell,
                        "{what}: the caret does not round-trip on {line:?}"
                    );
                }
            }
        }
    }
}
