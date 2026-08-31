//! CARET-PRESERVING table-row math: express a caret's column on a GFM table
//! row as a LOGICAL position ([`TableCaretCell`], `locate_table_caret`) that
//! survives [`super::tables::align_table`]'s re-padding — and the inverse
//! (`table_caret_col`) that resolves it back to a raw column on a realigned
//! row. Split out of `tables.rs` (which owns the alignment/layout math
//! itself) purely to keep that file under its size ceiling; this pair is used
//! by `actions::edit::auto_align_table_on_row_leave`, the auto-align trigger.

use super::tables::table_cell_ranges;

/// Where a caret sits within one table ROW, independent of exact byte/char
/// offsets: which 0-based CELL its column falls in, and how many CHARS from
/// the start of that cell's TRIMMED content. [`super::tables::align_table`]
/// only ever rewrites a row's surrounding padding, never a cell's own
/// content, so this position is exactly the invariant that survives a
/// re-pad — see [`table_caret_col`], its inverse, which is how the auto-align
/// caller (`actions::edit::auto_align_table_on_row_leave`) keeps the caret on
/// the same logical cell/offset across the replace.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct TableCaretCell {
    pub cell: usize,
    pub offset: usize,
}

/// Locate a CHAR column `col` on table row `line` as a [`TableCaretCell`]. A
/// column resting inside a cell's own trimmed content reports that exact
/// offset; one resting on the delimiting `|` or its surrounding padding
/// clamps to the NEAREST cell edge (the end of the cell before it, or the
/// start of the cell after) rather than reporting a position outside any
/// cell — so every column has an answer, even mid-realign. A pipeless `line`
/// is treated as one whole cell.
pub(crate) fn locate_table_caret(line: &str, col: usize) -> TableCaretCell {
    let ranges = table_cell_ranges(line);
    if ranges.is_empty() {
        return TableCaretCell {
            cell: 0,
            offset: col,
        };
    }
    let byte_col = char_col_to_byte(line, col);
    let last = ranges.len() - 1;
    for (i, r) in ranges.iter().enumerate() {
        if byte_col <= r.end || i == last {
            let clamped = byte_col.clamp(r.start, r.end);
            let offset = byte_to_char_col(line, clamped) - byte_to_char_col(line, r.start);
            return TableCaretCell { cell: i, offset };
        }
    }
    unreachable!("the `i == last` arm above always matches by the final iteration");
}

/// The inverse of [`locate_table_caret`]: the CHAR column on `line` (typically
/// a freshly REALIGNED row) for a logical [`TableCaretCell`] position — the
/// start of that cell's trimmed content plus `offset` chars, clamped to the
/// cell's length (unchanged by alignment, since it never edits content — the
/// clamp only guards a ragged row whose cell genuinely got shorter/longer
/// content some other way). A `cell` past `line`'s own column count clamps to
/// the last cell — alignment can only ever GAIN trailing cells (ragged-row
/// padding), never lose one a caret was already inside.
pub(crate) fn table_caret_col(line: &str, pos: TableCaretCell) -> usize {
    let ranges = table_cell_ranges(line);
    if ranges.is_empty() {
        return pos.offset;
    }
    let r = &ranges[pos.cell.min(ranges.len() - 1)];
    let start_col = byte_to_char_col(line, r.start);
    let cell_len = byte_to_char_col(line, r.end) - start_col;
    start_col + pos.offset.min(cell_len)
}

/// Byte offset of CHAR column `col` on `line`, clamped to `line`'s length.
fn char_col_to_byte(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map(|(b, _)| b)
        .unwrap_or(line.len())
}

/// CHAR column of byte offset `byte` on `line` — the inverse of
/// [`char_col_to_byte`], for a `byte` that always lands on a char boundary
/// (every offset this pair feeds it comes from [`table_cell_ranges`], which
/// derives its own boundaries from `char_indices`/`trim_start`/`trim_end` and
/// so never splits a multi-byte scalar).
fn byte_to_char_col(line: &str, byte: usize) -> usize {
    line[..byte].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_and_resolve_round_trip_through_a_realign() {
        // "bbb" is cell 0; landing mid-content (offset 1, between the two b's)
        // must come back to the same (cell, offset) after the row gains extra
        // padding elsewhere — the byte offset moves, the logical position does
        // not.
        let raw = "|bbb|2|";
        let pos = locate_table_caret(raw, 2); // between the two b's
        assert_eq!(pos, TableCaretCell { cell: 0, offset: 1 });
        let padded = "| bbb   | 2 |"; // column 0 widened by an unrelated row
        let col = table_caret_col(padded, pos);
        let back = locate_table_caret(padded, col);
        assert_eq!(back, pos, "round-trips through a wider realigned row");
    }

    #[test]
    fn a_column_on_the_delimiter_clamps_to_the_nearest_cell_edge() {
        let raw = "|bbb|2|";
        // col 4 sits exactly on the pipe between the two cells -- it clamps to
        // the END of the cell before it (cell 0's trimmed length is 3), per the
        // documented "nearest cell edge, scanning left to right" rule.
        let pos = locate_table_caret(raw, 4);
        assert_eq!(pos, TableCaretCell { cell: 0, offset: 3 });
    }

    #[test]
    fn a_pipeless_line_is_treated_as_one_whole_cell() {
        let pos = locate_table_caret("plain text", 5);
        assert_eq!(pos, TableCaretCell { cell: 0, offset: 5 });
        assert_eq!(table_caret_col("plain text", pos), 5);
    }
}
