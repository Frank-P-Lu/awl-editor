//! GFM table actions: opening and committing the dimension picker, plus the
//! Enter and Tab source-editing gestures. Insert mirrors `actions/link.rs`'s
//! shape (a pure transform producing a [`format::FormatResult`], applied as ONE atomic
//! edit via `Buffer::apply_format`) for the same "same behavior, same code"
//! reason: an insert-table commit is "replace nothing with a new block, then
//! land the cursor sensibly", exactly like Insert-link and Insert-footnote.

use super::format::FormatResult;
use super::*;
use crate::overlay::OverlayState;

fn table_row_source(columns: usize) -> String {
    format!("|{}", " |".repeat(columns.max(1)))
}

/// Bare Enter inserts a correctly-columned source row immediately below a real
/// GFM table row. Shift-Enter deliberately bypasses this owner in `actions.rs`.
pub(super) fn table_newline(ctx: &mut ActionCtx) -> bool {
    if !ctx.buffer.is_markdown() || ctx.buffer.has_selection() {
        return false;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    let (line, _) = ctx.buffer.cursor_line_col();
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, line) else {
        return false;
    };
    let columns = (start..end)
        .filter(|&row| row != start + 1)
        .map(|row| crate::markdown::split_row_cells(lines[row]).len())
        .max()
        .unwrap_or(1);
    // Header + separator are inseparable GFM structure: inserting "below the
    // header" means the first body row, below its separator.
    let below = if line == start { start + 1 } else { line };
    let at = ctx.buffer.line_col_to_char(below, usize::MAX);
    let row = table_row_source(columns);
    ctx.buffer.replace_char_range(at, at, &format!("\n{row}"));
    ctx.buffer.set_cursor(at + 3);
    true
}

/// Tab/Shift-Tab walk real table cells in source order and select the target's
/// trimmed content, so typing replaces that cell like a spreadsheet. An empty
/// cell is a bare caret at its raw content start (immediately after the opening
/// pipe), because there is no content to select. Reaching the forward end appends
/// one scaffold row; non-table source and arbitrary selections keep the
/// established list-indentation behavior.
pub(super) fn table_tab(ctx: &mut ActionCtx, forward: bool) -> bool {
    if !ctx.buffer.is_markdown() {
        return false;
    }
    let text = ctx.buffer.text();
    let lines: Vec<&str> = text.split('\n').collect();
    let (line, col) = ctx.buffer.cursor_line_col();
    let Some((start, end)) = crate::markdown::table_block_lines(&lines, line) else {
        return false;
    };
    let mut cells = Vec::new();
    for (row, line_text) in lines.iter().enumerate().take(end).skip(start) {
        if row == start + 1 {
            continue;
        }
        for range in crate::markdown::table_cell_ranges(line_text) {
            let start_col = line_text[..range.start].chars().count();
            let end_col = line_text[..range.end].chars().count();
            let start_char = ctx.buffer.line_col_to_char(row, start_col);
            let end_char = ctx.buffer.line_col_to_char(row, end_col);
            cells.push((row, start_col, start_char, end_char));
        }
    }
    if cells.is_empty() {
        return false;
    }
    let current = if let Some(selection) = ctx.buffer.selection_range() {
        let Some(i) = cells
            .iter()
            .position(|&(_, _, start_char, end_char)| selection == (start_char, end_char))
        else {
            // Only an exact cell selection stays in table navigation. Arbitrary
            // regions retain ordinary Tab's established replace/indent behavior.
            return false;
        };
        i
    } else {
        cells
            .iter()
            .rposition(|&(row, start_col, _, _)| row == line && start_col <= col)
            .or_else(|| cells.iter().position(|&(row, _, _, _)| row == line))
            .unwrap_or_else(|| if forward { 0 } else { cells.len() - 1 })
    };
    if forward && current + 1 == cells.len() {
        let columns = cells
            .iter()
            .filter(|(row, _, _, _)| *row == start)
            .count()
            .max(1);
        let at = ctx.buffer.line_col_to_char(end - 1, usize::MAX);
        ctx.buffer
            .replace_char_range(at, at, &format!("\n{}", table_row_source(columns)));
        ctx.buffer.clear_mark();
        ctx.buffer.set_cursor(ctx.buffer.line_col_to_char(end, 1));
    } else {
        let next = if forward {
            current + 1
        } else {
            current.saturating_sub(1)
        };
        let (_, _, start_char, end_char) = cells[next];
        ctx.buffer.select_range(start_char, end_char);
    }
    true
}

/// Summon the DIMENSION PICKER over the editor. Markdown-only, like every
/// other formatting command (Insert-link, Insert-footnote, Align table) — a
/// calm no-op on a non-markdown buffer.
pub(super) fn open_insert_table(ctx: &mut ActionCtx) {
    if !ctx.buffer.is_markdown() {
        return;
    }
    ctx.journey.enter(Some(OverlayState::new_table_dims()));
}

/// Build the fresh table's `FormatResult`: a `rows`×`cols` GFM table
/// ([`crate::markdown::build_table`]) inserted at `anchor.map_or(cursor,
/// |a| a.max(cursor))` — the same insertion-point convention
/// `actions::format::footnotes::insert_footnote` uses, so an active selection
/// never gets silently swallowed into the table. Blank lines are inserted
/// around the table wherever the surrounding text doesn't already provide
/// one, so a table dropped mid-paragraph still parses as its own GFM block
/// rather than a lazy continuation of the line before/after it. The caret
/// lands in the FIRST HEADER CELL — right after the table's own leading
/// `"| "` (`markdown::FIRST_CELL_OFFSET`). Pure; no clock, no allocation
/// beyond the output string.
pub(super) fn insert_table_at(
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
    rows: usize,
    cols: usize,
) -> FormatResult {
    let insert = anchor.map_or(cursor, |anchor| anchor.max(cursor));
    let chars: Vec<char> = text.chars().collect();
    let insert = insert.min(chars.len());
    let before: String = chars[..insert].iter().collect();
    let after: String = chars[insert..].iter().collect();

    let lead = if before.is_empty() || before.ends_with("\n\n") {
        ""
    } else if before.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let table = crate::markdown::build_table(rows, cols);
    let trail = if after.is_empty() || after.starts_with("\n\n") {
        ""
    } else if after.starts_with('\n') {
        "\n"
    } else {
        "\n\n"
    };

    let table_start = before.chars().count() + lead.chars().count();
    let mut out =
        String::with_capacity(before.len() + lead.len() + table.len() + trail.len() + after.len());
    out.push_str(&before);
    out.push_str(lead);
    out.push_str(&table);
    out.push_str(trail);
    out.push_str(&after);

    FormatResult {
        text: out,
        anchor: None,
        cursor: table_start + crate::markdown::FIRST_CELL_OFFSET,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_at_bare_cursor_with_blank_lines_and_lands_in_the_first_header_cell() {
        let text = "before\nmiddle\nafter";
        let cursor = "before\nmiddle".chars().count();
        let result = insert_table_at(text, None, cursor, 3, 2);
        let table = crate::markdown::build_table(3, 2);
        assert_eq!(
            result.text,
            format!("before\nmiddle\n\n{table}\n\nafter"),
            "blank lines open on both sides of a mid-paragraph insertion"
        );
        // The caret lands right after the table's own leading "| ".
        let table_start = result.text.find(&table).unwrap();
        let table_start_chars = result.text[..table_start].chars().count();
        assert_eq!(
            result.cursor,
            table_start_chars + crate::markdown::FIRST_CELL_OFFSET
        );
        let out_chars: Vec<char> = result.text.chars().collect();
        let at = result.cursor;
        assert_eq!(out_chars[at - 2..at].iter().collect::<String>(), "| ");
    }

    #[test]
    fn at_document_start_and_end_adds_no_leading_or_trailing_blank_line() {
        let result = insert_table_at("", None, 0, 1, 1);
        assert_eq!(result.text, crate::markdown::build_table(1, 1));
    }

    #[test]
    fn already_blank_boundaries_are_not_doubled() {
        let text = "para\n\n\n\nafter";
        // Cursor sits right after the run of blank lines and before "after".
        let cursor = text.find("after").unwrap();
        let result = insert_table_at(text, None, cursor, 1, 1);
        // `before` already ends "\n\n" (two-plus blank lines) -- no extra lead.
        assert!(result.text.starts_with(&format!(
            "para\n\n\n\n{}",
            crate::markdown::build_table(1, 1)
        )));
    }

    #[test]
    fn a_selection_end_wins_over_a_caret_left_of_it() {
        // anchor (selection start) at 0, cursor (caret) parked at 3 -- the
        // FURTHER end of the selection is where the table lands, mirroring
        // insert_footnote's `anchor.max(cursor)`.
        let text = "abcdef";
        let result = insert_table_at(text, Some(0), 3, 1, 1);
        assert!(
            result
                .text
                .starts_with(&format!("abc\n\n{}", crate::markdown::build_table(1, 1)))
        );
    }
}
