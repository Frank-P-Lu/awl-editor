//! The INSERT-TABLE dimension picker's own geometry, draw, and hit-test —
//! the FOURTH dedicated [`TextPipeline::overlay_geometry`] arm (after
//! spell/workspace/theme), because this card's content is a drawn GRID, not
//! a candidate row list. Gated on [`TextPipeline::overlay_table_dims`]
//! (`Option<(rows, cols)>`), exactly the way the contextual spell popup gates
//! on `overlay_spell` — an `Option` the render layer reads, never a kind
//! string (`ViewState` carries no `OverlayKind` field at all).
//!
//! **One shared geometry function, [`TextPipeline::table_dims_cell_rect`],
//! computes every cell's paint rect AND is what the hit-test walks** — the
//! drawn grid and the clickable grid cannot disagree because they are the
//! same rects, read twice.

use super::*;

/// The grid ALWAYS draws at its full [`crate::overlay::MAX_ROWS`] ×
/// [`crate::overlay::MAX_COLS`] extent — only which cells paint FILLED vs.
/// EMPTY changes as the picker is sculpted (the Word/Docs insert-table grid
/// convention: a fixed visual grid, a growing highlighted region).
const CELL: Logical = Logical(26.0);
const CELL_GAP: Logical = Logical(3.0);
/// The breathing gap between the grid's bottom edge and the readout hint
/// line below it — spent as this geometry's own `header_gap` (see
/// [`TextPipeline::table_dims_overlay_geometry`]'s doc for why that field is
/// the right one to repurpose).
const GRID_HINT_GAP: Logical = Logical(10.0);

impl TextPipeline {
    /// The drawn grid's own `(width, height)` in device px, independent of
    /// any [`OverlayGeom`] — needed BEFORE one exists, to size the card.
    fn table_dims_grid_extent(&self) -> (f32, f32) {
        let cell = self.metrics.px(CELL);
        let gap = self.metrics.px(CELL_GAP);
        let cols = crate::overlay::MAX_COLS as f32;
        let rows = crate::overlay::MAX_ROWS as f32;
        (
            cols * cell + (cols - 1.0) * gap,
            rows * cell + (rows - 1.0) * gap,
        )
    }

    /// THE SHARED CELL GEOMETRY: cell `(row, col)`'s paint rect `[x, y, w,
    /// h]`, 0-based from the grid's top-left (`geom.text_left`/`text_top`).
    /// Both [`Self::prepare_table_dims_grid`] (paint) and
    /// [`Self::table_dims_cell_at`] (hit-test) call this and NOTHING else —
    /// see the module doc.
    pub(in crate::render) fn table_dims_cell_rect(
        &self,
        geom: &OverlayGeom,
        row: usize,
        col: usize,
    ) -> [f32; 4] {
        let cell = self.metrics.px(CELL);
        let gap = self.metrics.px(CELL_GAP);
        let x = geom.text_left + col as f32 * (cell + gap);
        let y = geom.text_top + row as f32 * (cell + gap);
        [x, y, cell, cell]
    }

    /// The dimension picker's dedicated geometry arm, gated by
    /// [`Self::overlay_table_dims`] in [`Self::overlay_geometry`] exactly
    /// like [`Self::spell_overlay_geometry`]'s own gate. No header rows, no
    /// query line, no candidate list at all (`n_items`/`visible` stay `0`) —
    /// the grid occupies the space a header normally would, spent as
    /// `header_gap` (the SAME field `overlay_header_gap` already means
    /// "vertical room between the header and the row band"), so the hint
    /// readout — the ONE row this card's row-plan carries — lands right
    /// below the grid through the ordinary [`Self::overlay_card_h`]/scene-
    /// planner machinery with no new positioning logic.
    pub(in crate::render) fn table_dims_overlay_geometry(&self, width: u32) -> OverlayGeom {
        let m = self.metrics;
        let pad = m.px(CARD_PAD);
        let margin = m.px(CARD_MARGIN);
        let (grid_w, grid_h) = self.table_dims_grid_extent();
        let header_gap = grid_h + m.px(GRID_HINT_GAP);
        let hint_rows = 1;

        let desired_w = grid_w + 2.0 * pad;
        let (card_x, card_w) = self.overlay_card_box(width, desired_w);
        let card_y = margin + m.px(CARD_TOP_DROP) + self.menubar_reserve();
        let card_h = self.overlay_card_h(hint_rows, header_gap, hint_rows, 0, pad);
        let card_y = card_y + self.overlay_entrance_offset();
        let text_left = card_x + pad;
        let text_top = card_y + pad;

        OverlayGeom {
            visible: 0,
            top_idx: 0,
            n_items: 0,
            hint: self.overlay_hint.clone(),
            hint_rows,
            header_rows: 0,
            header_gap,
            empty: None,
            card_x,
            card_y,
            card_w,
            card_h,
            text_left,
            text_top,
            text_w: card_w - 2.0 * pad,
            cue_above: None,
            cue_below: None,
            cue_reserved: false,
            ..OverlayGeom::base()
        }
    }

    /// Build the grid's quads (filled ink for every cell inside the live
    /// `rows × cols`, a faint wash for the rest) and upload them — a no-op
    /// clear when the picker is not open, so a stray call after close still
    /// leaves nothing drawn (`park_overlay` also parks this pipeline
    /// directly, belt-and-braces, per its own doc).
    pub(in crate::render) fn prepare_table_dims_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let Some((rows, cols)) = self.overlay_table_dims else {
            self.table_dims_cells
                .prepare(device, queue, width, height, &[]);
            return;
        };
        let geom = self.table_dims_overlay_geometry(width);
        let filled = theme::base_content().rgba_bytes();
        let empty = theme::base_200().rgba_bytes();
        let mut quads = Vec::with_capacity(crate::overlay::MAX_ROWS * crate::overlay::MAX_COLS);
        for row in 0..crate::overlay::MAX_ROWS {
            for col in 0..crate::overlay::MAX_COLS {
                let rect = self.table_dims_cell_rect(&geom, row, col);
                let color = if row < rows && col < cols { filled } else { empty };
                quads.push((rect, color));
            }
        }
        self.table_dims_cells
            .prepare_multicolor(device, queue, width, height, &quads);
    }

    /// Hit-test a pointer at PHYSICAL `(px, py)` against the drawn grid,
    /// returning the 0-based `(row, col)` cell it lands on. `None` when the
    /// picker is not open or the pointer is off every cell — reads the SAME
    /// [`Self::table_dims_cell_rect`] the paint loop draws from, so a click
    /// can never land on a cell the grid does not actually show there.
    pub fn table_dims_cell_at(&self, px: f32, py: f32) -> Option<(usize, usize)> {
        self.overlay_table_dims?;
        let geom = self.table_dims_overlay_geometry(self.window_w as u32);
        for row in 0..crate::overlay::MAX_ROWS {
            for col in 0..crate::overlay::MAX_COLS {
                let [x, y, w, h] = self.table_dims_cell_rect(&geom, row, col);
                if px >= x && px < x + w && py >= y && py < y + h {
                    return Some((row, col));
                }
            }
        }
        None
    }
}
