//! Structural SPLICES over a GFM table's SOURCE: insert or delete a row or a
//! column ([`TableVerb`], [`table_splice`]). awl is a source editor — these
//! rewrite the plain-text block, they do not drive a grid model, and every
//! splice re-emits through [`super::tables::align_table`], the ONE padder, so a
//! spliced table can never disagree with what an ordinary re-pad would produce.
//!
//! The parse is the table module's own row/cell parser
//! ([`super::tables::split_row_cells`], which honors a `\|` escape) and the
//! per-column alignment markers are read and re-emitted through
//! [`super::tables::parse_col_align`], so a column that MOVES (or a column
//! deleted from beside it) carries its `:` markers with it.
//!
//! The caret lands as a LOGICAL [`TableCaretCell`] rather than a byte offset,
//! for the same reason the auto-align path does: the re-pad shifts every offset
//! on the row, and only (cell, intra-cell offset) survives it.

use super::table_caret::TableCaretCell;
use super::tables::{ColAlign, align_table, is_separator_row, parse_col_align, split_row_cells};

/// The GFM header-separator's fixed position inside a table block — the second
/// line, always. [`super::tables::align_table`] and
/// [`super::tables::push_table_markup`] both pin it by INDEX for the same
/// reason (a body cell of literal `---` is then never mistaken for it), and a
/// splice that disagreed with the padder about which line is structural would
/// re-emit the table inside out.
const SEP_LINE: usize = 1;

/// One structural table edit. Row verbs move whole source lines; column verbs
/// splice one cell into (or out of) EVERY row including the header and the
/// separator.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TableVerb {
    InsertRowAbove,
    InsertRowBelow,
    InsertColumnLeft,
    InsertColumnRight,
    DeleteRow,
    DeleteColumn,
}

/// Why a verb DECLINED — a deliberate refusal with something to say, never a
/// silent no-op. Each one names a fact about GFM's own shape, not a limitation
/// of the splice.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TableRefusal {
    /// The block's second line is not a header-separator row, so it has no
    /// header/body structure to splice. (`table_block_lines` only demands a
    /// separator SOMEWHERE in the run.)
    NotAGfmTable,
    /// Insert-row-above with the caret on the header (or its separator): a GFM
    /// table's header IS its first row — there is no position above it.
    HeaderIsFirstRow,
    /// Delete-row with the caret on the header (or its separator): removing
    /// either leaves a run of pipes that is no longer a table. Emptying a
    /// table's BODY is fine — a header + separator alone is valid GFM — so the
    /// last body row deletes happily; the table itself is deleted with
    /// ordinary text editing.
    HeaderRowIsStructural,
    /// Delete-column on a one-column table: a zero-column table is not a table.
    OnlyColumn,
}

impl TableRefusal {
    /// The writer-visible sentence. One owner, so the notice, the tests and any
    /// prose about the refusal read the same words.
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::NotAGfmTable => "that table has no header-separator row",
            Self::HeaderIsFirstRow => "a table's header is already its first row",
            Self::HeaderRowIsStructural => "a table's header row can't be deleted",
            Self::OnlyColumn => "a table needs at least one column",
        }
    }
}

/// A spliced table block: its full new SOURCE (already aligned), plus where the
/// caret should land — the block-relative LINE and the logical
/// [`TableCaretCell`] on it.
///
/// `caret.offset` is always 0: every verb changes what is under the caret (a
/// fresh blank cell, or a different row's content at the same index), so
/// preserving an intra-cell offset would preserve a position that no longer
/// means anything. Landing at the start of the target cell is the predictable
/// answer, and it is the one a writer can type into immediately.
pub(crate) struct TableSplice {
    pub text: String,
    pub row: usize,
    pub caret: TableCaretCell,
}

/// Apply `verb` to the GFM table `block` (its raw source lines joined by `\n`),
/// with the caret on block-relative line `row` at logical position `caret`.
///
/// Contract:
/// - **Ragged rows normalize.** The grid's column count is the MAX cell count
///   across every line (the separator included), and short rows gain empty
///   cells — exactly the normalization
///   [`super::tables::align_table`] already performs on every re-pad, not a
///   second rule.
/// - **Alignment is carried, never recomputed.** Each column's `:` markers ride
///   its index; an inserted column takes [`ColAlign::None`]; a deleted column
///   takes its marker with it and every other column keeps its own.
/// - **Outer pipes are normalized** (`a | b` becomes `| a | b |`) because the
///   output goes through the one padder — the same thing Align table does.
/// - The header and its separator are ONE structural unit: a caret on either
///   answers "the header row" for every verb.
/// - Pure; no clock, no globals.
pub(crate) fn table_splice(
    block: &str,
    row: usize,
    caret: TableCaretCell,
    verb: TableVerb,
) -> Result<TableSplice, TableRefusal> {
    let lines: Vec<&str> = block.split('\n').collect();
    if lines.len() <= SEP_LINE || !is_separator_row(lines[SEP_LINE]) {
        return Err(TableRefusal::NotAGfmTable);
    }
    // The GRID is every CONTENT row, header first, with the separator lifted
    // out into `aligns` — so a row index here is "which row of the table",
    // never "which source line", and the structural line can't be spliced.
    let mut grid: Vec<Vec<String>> = lines
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != SEP_LINE)
        .map(|(_, line)| split_row_cells(line))
        .collect();
    let sep_cells = split_row_cells(lines[SEP_LINE]);
    let ncols = grid
        .iter()
        .map(Vec::len)
        .chain(std::iter::once(sep_cells.len()))
        .max()
        .unwrap_or(1)
        .max(1);
    for cells in grid.iter_mut() {
        cells.resize(ncols, String::new());
    }
    let mut aligns: Vec<ColAlign> = (0..ncols)
        .map(|c| {
            sep_cells
                .get(c)
                .map(|cell| parse_col_align(cell))
                .unwrap_or(ColAlign::None)
        })
        .collect();

    // The caret's GRID row and column, clamped into the normalized grid.
    let g = if row <= SEP_LINE { 0 } else { row - 1 };
    let g = g.min(grid.len() - 1);
    let c = caret.cell.min(ncols - 1);

    let (grid_row, cell) = match verb {
        TableVerb::InsertRowAbove => {
            if g == 0 {
                return Err(TableRefusal::HeaderIsFirstRow);
            }
            grid.insert(g, vec![String::new(); ncols]);
            (g, 0)
        }
        TableVerb::InsertRowBelow => {
            grid.insert(g + 1, vec![String::new(); ncols]);
            (g + 1, 0)
        }
        TableVerb::DeleteRow => {
            if g == 0 {
                return Err(TableRefusal::HeaderRowIsStructural);
            }
            grid.remove(g);
            // The row below slides up into the caret's place; deleting the LAST
            // body row leaves the caret on the new last row (the header, if the
            // body is now empty).
            (g.min(grid.len() - 1), c)
        }
        TableVerb::InsertColumnLeft => {
            for cells in grid.iter_mut() {
                cells.insert(c, String::new());
            }
            aligns.insert(c, ColAlign::None);
            (g, c)
        }
        TableVerb::InsertColumnRight => {
            for cells in grid.iter_mut() {
                cells.insert(c + 1, String::new());
            }
            aligns.insert(c + 1, ColAlign::None);
            (g, c + 1)
        }
        TableVerb::DeleteColumn => {
            if ncols == 1 {
                return Err(TableRefusal::OnlyColumn);
            }
            for cells in grid.iter_mut() {
                cells.remove(c);
            }
            aligns.remove(c);
            // Deleting the last column lands the caret on the new last one.
            (g, c.min(ncols - 2))
        }
    };

    Ok(TableSplice {
        text: emit(&grid, &aligns),
        // Back to a SOURCE line: the header is line 0, every body row sits one
        // past its grid index because the separator lives between them.
        row: if grid_row == 0 { 0 } else { grid_row + 1 },
        caret: TableCaretCell { cell, offset: 0 },
    })
}

/// Re-emit the grid as GFM source, re-padded by the one padder. The raw lines
/// built here are deliberately minimal (single-space cells, three-character
/// separator stubs): [`align_table`] owns every width decision, so this
/// function can never become a second opinion about padding.
fn emit(grid: &[Vec<String>], aligns: &[ColAlign]) -> String {
    let mut lines = Vec::with_capacity(grid.len() + 1);
    lines.push(row_source(&grid[0]));
    lines.push(sep_source(aligns));
    for cells in &grid[1..] {
        lines.push(row_source(cells));
    }
    align_table(&lines.join("\n"))
}

/// One data row's raw source: `| a | b |`. Cell content is re-emitted verbatim,
/// so a `\|` escape (and anything else the cell parser kept) survives.
fn row_source(cells: &[String]) -> String {
    let mut s = String::from("|");
    for cell in cells {
        s.push(' ');
        s.push_str(cell);
        s.push(' ');
        s.push('|');
    }
    s
}

/// The header-separator's raw source at each column's alignment. Widths are
/// [`align_table`]'s business; these stubs only carry the `:` markers.
fn sep_source(aligns: &[ColAlign]) -> String {
    let mut s = String::from("|");
    for align in aligns {
        s.push_str(match align {
            ColAlign::None => "---",
            ColAlign::Left => ":--",
            ColAlign::Right => "--:",
            ColAlign::Center => ":-:",
        });
        s.push('|');
    }
    s
}

#[cfg(test)]
mod tests;
