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

use super::{OverlayRowPlan, PlannedRow, RowLanes};
use crate::render::TextPipeline;

/// One published candidate row: the rectangle that is simultaneously DRAWN,
/// CLICKABLE, and REPORTED.
///
/// `x .. x + w` is the row's own inclusive pointer span (`row_x_span`), so a
/// pointer at `x + w * 0.5` selects `item` and a pointer outside it does not.
/// `item` is `None` for a display line that carries no selectable item — the
/// grouped family's section headings and the card's own secondary location line.
///
/// WHICH ROW IS SELECTED IS DELIBERATELY ABSENT. The sidecar already reports it
/// once, as `window.sel_row`, resolved through the owner that also colours the
/// band — so a second answer here could only be the plan's LOGICAL row, which is
/// a different fact (the line Enter activates) that disagrees with the drawn one
/// for the length of every selection move. Publishing both without saying which
/// is which would put two selections in one sidecar, and this block exists to
/// make drawn-versus-published agreement assertable rather than ambiguous. Ask
/// `window.sel_row` for the selection and these rects for the geometry.
///
/// `lanes` carries the row's ACCESSORY CLUSTER, absent lane by absent lane: the
/// name's seated ink, the value's, and the rail's own track and pointer band.
/// Each is `None` exactly when the frame drew nothing there, so a card that
/// yielded its whole accessory column publishes the absence rather than the
/// width the column would have wanted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlannedRowRect {
    pub display: usize,
    pub item: Option<usize>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub lanes: RowLanes,
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
    pub rows: Vec<PlannedRowRect>,
}

impl OverlayRowPlan {
    /// Project this plan into its published form. A projection, never a second
    /// derivation: no arithmetic here that the plan does not already own.
    ///
    /// `lanes` is asked of the caller rather than derived, for the reason the
    /// whole projection exists: a row's lanes are seated by the accessors the
    /// DRAW reads, which need the measured buffers and the world's mirror — none
    /// of which a plan carries. Resolving them here would have meant a second
    /// spelling of the seat.
    pub(super) fn geometry_report(
        &self,
        lanes: impl Fn(&PlannedRow) -> RowLanes,
    ) -> OverlayRowGeometry {
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
                    lanes: lanes(row),
                }
            })
            .collect();
        OverlayRowGeometry {
            band_x: self.card_x,
            band_w: self.card_w,
            first_top: self.first_top(),
            pitch: self.lh(),
            footer_top: self.footer_top(),
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
        let plan = self.overlay_row_plan(&geom);
        let lanes = self.overlay_row_lanes(&geom, &plan);
        Some(plan.geometry_report(|row| {
            lanes.get(&row.display).copied().unwrap_or_default()
        }))
    }
}
