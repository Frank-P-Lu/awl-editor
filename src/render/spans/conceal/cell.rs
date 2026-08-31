//! Per-cell inline styling for a rendered GFM table grid cell.

use super::*;

/// Build the per-cell `AttrsList` for a GFM table GRID cell (the tables-v1 styled,
/// off-cursor render — `layers::prepare_table_grid`). A cell is a SMALL INLINE
/// markdown context: it may carry `**bold**` / `*italic*` / `` `code` `` /
/// `==highlight==`, but no block construct. This reuses the EXACT inline-styling
/// seam prose uses — [`crate::markdown::spans`] on the cell substring, then
/// [`super::add_md_line_spans`] (content styling: real bundled Bold weight, italic,
/// mono+tint inline code) followed by [`super::add_wysiwyg_conceal_spans`] (the
/// emphasis / code / highlight DELIMITERS collapse to true zero-width) — so a cell
/// styles identically to the same run in body prose, with the raw markers gone from
/// both the pixels AND the shaped WIDTH (the concealed advance is sub-pixel, so a
/// caller measuring `run.line_w` after shaping sizes the column to the styled
/// content, not the raw source). `line_height` is the cell row's own height,
/// threaded into the zero-width conceal so the concealed markers never shrink the
/// row (the same contract `build_line_attrs` honors). A grid cell is ALWAYS the
/// off-cursor styled form — the caret's OWN table parks the grid and reveals raw
/// source a level up (`prepare_table_grid`), so `conceal_off_cursor = true` and
/// `cursor_byte` is irrelevant (a cell has no fenced block). A cell with NO inline
/// markup yields an empty span set → the returned list is `base` alone, so a
/// plain cell shapes BYTE-IDENTICALLY to the pre-styling `set_text(cell, base)`.
/// Gated implicitly on `wysiwyg_on()` (the caller only builds a grid when it is
/// on; `add_wysiwyg_conceal_spans` also self-gates), so nothing conceals off.
pub(in crate::render) fn cell_inline_attrs(
    base: &Attrs<'static>,
    line_height: f32,
    cell: &str,
) -> glyphon::cosmic_text::AttrsList {
    let md_spans = crate::markdown::spans(cell);
    let mut al = glyphon::cosmic_text::AttrsList::new(base);
    add_md_line_spans(&mut al, cell, 0, base, &md_spans);
    add_wysiwyg_conceal_spans(
        &mut al,
        cell,
        0,
        base,
        &md_spans,
        true,
        0,
        line_height,
        None,
        None,
        None,
    );
    al
}
