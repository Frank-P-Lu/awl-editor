//! **THE PLANNED ROW BAND AS A PUBLISHED FACT** — the third reading of the one
//! planned object, for the capture sidecar.
//!
//! The draw emitters read the plan, the pointer hit-test inverts the plan, and
//! until now the harness could read neither: a test that wanted to know where a
//! picker row sat had to recover it from the PNG's pixels, which is an
//! appearance oracle answering a geometry question. This module is the third
//! reader, and it is deliberately nothing but a projection — every number it
//! carries is read off [`OverlayRowPlan`] through the same accessors the draw
//! and the pointer use ([`OverlayRowPlan::row_x_span`] above all), so a sidecar
//! rect that disagrees with the ink is impossible by construction rather than by
//! review.
//!
//! **THE COORDINATE SPACE IS PHYSICAL (DEVICE) PIXELS**, the space the whole
//! overlay geometry family already speaks: the same space `overlay_row_at`
//! accepts a pointer in, the same space the sidecar's `layout.rows[].top` and
//! `overlay.window.card_h` are already in, and the same space the PNG's own
//! pixels are in. A `WxH` capture at `--capture-dpi N` is a `(W/N)x(H/N)`
//! LOGICAL window, so every figure here scales with N — which is exactly what
//! makes it a usable oracle for a device-pixel bug, and exactly why it must not
//! be silently divided by anything on the way out.
//!
//! **WHICH PATH THIS RUNS ON.** Nothing here is reachable from a frame: the one
//! entry point is [`TextPipeline::overlay_row_geometry`], whose only callers are
//! the sidecar writer (once per capture) and the laws. It allocates one `Vec` of
//! one rect per PLANNED DISPLAY LINE — never one per corpus item — so it
//! inherits the planner's own O(visible) bound rather than adding a walk.

use super::OverlayRowPlan;
use crate::render::TextPipeline;

/// One published candidate row: the rectangle that is simultaneously DRAWN,
/// CLICKABLE, and REPORTED.
///
/// `x .. x + w` is the row's own inclusive pointer span (`row_x_span`), so a
/// pointer at `x + w * 0.5` selects `item` and a pointer outside it does not.
/// `item` is `None` for a display line that carries no selectable item — the
/// grouped family's section headings and the card's own secondary location line
/// — and `selected` marks the one line the plan reports as selected, which is
/// the line Enter activates rather than wherever an animated band currently sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlannedRowRect {
    pub display: usize,
    pub item: Option<usize>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub selected: bool,
}

/// The whole planned candidate band, published. `band_x`/`band_w` are the
/// content band every row is stepped in from; `first_top` and `pitch` are the
/// band's origin and row pitch; `footer_top` is where the candidate area ends
/// and the foot hint begins (the candidate rows PLUS an empty-state notice line,
/// which occupies a slot but plans no row).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OverlayRowGeometry {
    pub band_x: f32,
    pub band_w: f32,
    pub first_top: f32,
    pub pitch: f32,
    pub footer_top: f32,
    pub selected_display: Option<usize>,
    pub rows: Vec<PlannedRowRect>,
}

impl OverlayRowPlan {
    /// Project this plan into its published form. A projection, never a second
    /// derivation: no arithmetic here that the plan does not already own.
    pub(super) fn geometry_report(&self) -> OverlayRowGeometry {
        let selected_display = self.selected_display();
        let rows = self
            .rows()
            .iter()
            .map(|row| {
                let (x0, x1) = self
                    .row_x_span(row.display)
                    .expect("a row of this plan has a span");
                PlannedRowRect {
                    display: row.display,
                    item: row.item,
                    x: x0,
                    y: row.top,
                    w: x1 - x0,
                    h: row.height,
                    selected: Some(row.display) == selected_display,
                }
            })
            .collect();
        OverlayRowGeometry {
            band_x: self.card_x,
            band_w: self.card_w,
            first_top: self.first_top(),
            pitch: self.lh(),
            footer_top: self.footer_top(),
            selected_display,
            rows,
        }
    }
}

impl TextPipeline {
    /// **THE SIDECAR'S ROW GEOMETRY**, or `None` when no overlay is summoned.
    ///
    /// Builds this frame's plan through the ONE planning seam
    /// (`overlay_row_plan`) — the same call `overlay_row_at` and
    /// `overlay_window_report` make — and projects it. Fresh rather than
    /// retained, for the same reason the pointer entry point is: there is no
    /// plan cache to key, so no `buffer.version()` collision can serve a stale
    /// band across a buffer swap.
    pub(crate) fn overlay_row_geometry(&self) -> Option<OverlayRowGeometry> {
        if !self.overlay_active {
            return None;
        }
        let geom = self.overlay_geometry(self.window_w as u32);
        Some(self.overlay_row_plan(&geom).geometry_report())
    }
}
