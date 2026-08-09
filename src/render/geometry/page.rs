//! ITEM 116b — THE PAGE'S OWN COLUMN ON THE CANVAS.
//!
//! Two ideas of "the writing column" exist and they are not the same. The
//! DOCUMENT's column relocates when a summoned workspace hands the document
//! layer a comparison viewport (`render::chrome::comparison`); the PAGE's own
//! column on the canvas never does — the ground punch, the star margin band,
//! the frost seed key, the sidecar's `page` block and the draggable page edges
//! all describe the backdrop, which the relocation leaves exactly where it was.
//!
//! This module is that second idea, kept in one named place so it cannot become
//! a parallel geometry scattered through the tree.
//! [`TextPipeline::page_column_left`] / [`TextPipeline::page_column_width`] stay
//! crate-render-private and their call sites are ENUMERATED by
//! `render::tests::comparison_viewport::
//! the_unrelocated_page_column_has_exactly_the_named_consumers`.
//! Every other consumer reads `column_left()`/`column_width()` and follows the
//! document, or yields with the margin-orientation family
//! (`TextPipeline::margin_orientation_yields`).

use super::*;

impl TextPipeline {
    /// THE PAGE's OWN EDGE PAD in physical px — the ONE resolution of the authored
    /// [`PAGE_MIN_PAD`] that every pipeline-side reader shares with the pure policies
    /// in `geometry::column` (which take the same `dpi` explicitly, because they have
    /// no `&self`). The margin OUTLINE reads it too: the rail's left inset must be the
    /// SAME quantity `adaptive_column_left` granted the rail room for, and the
    /// outline's own hit band must start where the block is drawn — three readers of
    /// one pad, so a click can never miss a rail it can see.
    ///
    /// `self.dpi`, NOT `self.metrics.scale`: see the module doc on
    /// [`page_column_advance`]'s zoom-stripped space.
    pub(in crate::render) fn edge_pad(&self) -> f32 {
        PAGE_MIN_PAD.px(self.dpi)
    }

    /// THE PAGE's own column on the CANVAS, never relocated — the module-private
    /// bypass item 116b's one owner leaves for the two consumers that genuinely
    /// mean the backdrop rather than the document: [`Self::page_geometry`] (the
    /// ground punch, the star margin band, the frost seed key) and the sidecar's
    /// `page` block through it. Everything else stays on [`Self::column_width`]
    /// and follows the document.
    pub(in crate::render) fn page_column_width(&self) -> f32 {
        column_width_for(
            self.window_w,
            self.page_advance(),
            crate::page::page_on(),
            crate::page::measure(),
            self.dpi,
        )
    }

    /// The canvas-side twin of [`Self::page_column_width`]. WIDE: a
    /// byte-identical passthrough to [`column_left_for`]. NARROW + the margin
    /// outline wanting its rail ([`Self::outline_wants_rail`]): shifts right per
    /// [`adaptive_column_left`]'s pressure test. Zoom-independent (driven by
    /// [`Self::page_advance`]).
    pub(in crate::render) fn page_column_left(&self) -> f32 {
        let label = crate::markdown::type_scale::LABEL;
        let char_width = self.page_advance();
        adaptive_column_left(
            self.window_w,
            char_width,
            crate::page::page_on(),
            crate::page::measure(),
            self.outline_wants_rail(),
            rowlayout::OUTLINE_PREFERRED_CHARS as f32 * self.metrics.char_width * label,
            rowlayout::OUTLINE_MIN_CHARS as f32 * self.metrics.char_width * label,
            self.metrics.char_width * crate::render::chrome::MARGIN_COLUMN_GAP_CHARS.0,
            self.dpi,
        )
    }

    /// THE PAGE ON THE CANVAS — page state, measure, and the page column's own
    /// unrelocated left/width. Read by the ground punch, the star margin band,
    /// the frost seed key and the sidecar's `page` block: all of them describe
    /// the BACKDROP, which item 116b's relocated document viewport deliberately
    /// does not move. (The document's own column is [`Self::column_left`].)
    pub fn page_geometry(&self) -> (bool, usize, f32, f32) {
        (
            crate::page::page_on(),
            crate::page::measure(),
            self.page_column_left(),
            self.page_column_width(),
        )
    }

    /// DIRECT-MANIPULATION resize — is the pointer at `pointer_x` (physical px)
    /// hovering a DRAGGABLE page-column edge? True whenever page mode is ON and the
    /// pointer is within [`PAGE_RESIZE_GRAB_PX`] of a DRAWN column edge — including a
    /// COLLAPSED page whose margins sit at the [`PAGE_MIN_PAD`] floor. The edge is
    /// the affordance whether or not there is margin left to give: dragging INWARD
    /// from a collapsed column must still narrow the measure (else the user is locked
    /// out — the widen-past-capacity lockout bug, 2026-07-15). The pure proximity
    /// test is [`page_boundary_hit`]. The live app reads this to flip the OS cursor
    /// to a resize glyph and to decide whether a press begins a width drag instead of
    /// a text selection.
    pub fn page_resize_hover(&self, pointer_x: f32) -> bool {
        self.page_resize_edge_at(pointer_x).is_some()
    }

    /// ITEM 116b — a DRAGGABLE page edge is margin orientation in the direct-
    /// manipulation register: it asks "how wide is my page?" of a document the
    /// user is writing. While the document layer is relocated into a read-only
    /// comparison ([`Self::margin_orientation_yields`]) there is no such page on
    /// screen, so the affordance yields with the rest of the margin family
    /// rather than arming a measure drag against the comparison pane's edge.
    pub fn page_resize_edge_at(&self, pointer_x: f32) -> Option<ResizeEdge> {
        if self.margin_orientation_yields() {
            return None;
        }
        page_resize_edge_hit(
            crate::page::page_on(),
            self.page_column_left(),
            self.page_column_width(),
            pointer_x,
            self.metrics.px(PAGE_RESIZE_GRAB_PX),
        )
    }

    pub fn over_writing_column(&self, pointer_x: f32) -> bool {
        in_writing_column(pointer_x, self.column_left(), self.column_width())
    }

    pub fn page_resize_measure_at(&self, pointer_x: f32, edge: ResizeEdge, anchor_x: f32) -> usize {
        page_resize_measure_anchored(self.page_advance(), pointer_x, anchor_x, edge)
    }
}
