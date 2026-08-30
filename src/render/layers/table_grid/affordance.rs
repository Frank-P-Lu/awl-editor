//! Display-only geometry for empty table cells.

use super::*;

pub(super) fn empty_cell_affordance(cell: Option<&String>) -> bool {
    cell.is_none_or(String::is_empty)
}

pub(super) fn place_empty_cell_affordances(
    placed: &mut TablePlacement<'_>,
    meta: &TableMeta,
    shaped: &TableGridShaped,
    xray_lines: &[usize],
    context: TablePlacementContext,
    pan: f32,
    line_top: &impl Fn(usize) -> f32,
) {
    for (doc_line, cells) in &meta.grid_rows {
        if !table_decoration_visible(meta, xray_lines, *doc_line) {
            continue;
        }
        for column in 0..meta.ncols {
            if !empty_cell_affordance(cells.get(column)) {
                continue;
            }
            let left = context.text_left + shaped.col_x[column] - pan + context.pad;
            let width = (shaped.col_w[column] - 2.0 * context.pad).max(0.0);
            let top = line_top(*doc_line) + context.line_height * 0.28;
            if width > 0.0 {
                placed
                    .empty_rects
                    .push([left, top, width, context.line_height * 0.44]);
            }
        }
    }
}
