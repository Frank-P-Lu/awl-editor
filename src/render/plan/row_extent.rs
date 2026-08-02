//! THE HORIZONTAL HALF OF A ROW PLAN — how far each display line is stepped in
//! from either edge of the content band, and the one place that arithmetic runs.
//!
//! It is its own module because a card's vertical band and its horizontal extent
//! become knowable at DIFFERENT moments in a frame. The row sequence and its
//! slots fall straight out of the already-resolved geometry; a MEASURED diagonal
//! cluster cannot exist until the rows it measures have been shaped, and those
//! rows are shaped against the plan itself. So the frame builds its plan, lets
//! the measurement happen, and then COMPLETES the same plan
//! ([`OverlayRowPlan::complete_row_extent`]) rather than building a second one —
//! a second plan would be a second answer to "where is row 3", kept agreeing by
//! discipline alone.

use super::OverlayRowPlan;
use super::overlay_rows::{OverlayRowPlanInput, PlannedRow};

/// The row-side span for a measured diagonal cluster.  The cluster owner
/// resolves this from the same shaped label/accessory measurements that draw
/// the text; the planner is still the one place that turns it into row bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct RowSpan {
    pub dx: f32,
    pub dw: f32,
    pub dx_per_row: f32,
    pub dw_per_row: f32,
}

/// Everything a plan needs to answer "how far is display line `n` stepped in".
/// Carried as a value so the frame's own seam and the planner's build path read
/// exactly the same four numbers.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::render) struct RowExtent {
    pub dx_per_row: f32,
    pub cluster_span: Option<RowSpan>,
    pub selected_offset: Option<(f32, f32)>,
    pub selected_display: Option<usize>,
}

impl RowExtent {
    pub(super) fn of(input: &OverlayRowPlanInput<'_>) -> Self {
        Self {
            dx_per_row: input.dx_per_row,
            cluster_span: input.cluster_span,
            selected_offset: input.selected_offset,
            selected_display: input.selected_display,
        }
    }
}

/// THE ROW-EXTENT ARITHMETIC. Assigns each row's `dx`/`dw` OUTRIGHT, never
/// incrementally, so applying an extent to a plan that already carries one lands
/// on exactly the numbers a fresh plan would — which is what lets a frame
/// complete its one plan instead of growing a second.
///
/// The ONE signed step splits into the two-sided extent here and only here: the
/// positive part grows `dx` (left edge steps in), the negative part grows `dw`
/// (right edge steps in). Exactly one of the two is ever nonzero for a given
/// `dx_per_row`, which is what lets both mirrored compositions share one input
/// without a second knob. A row's horizontal extent comes off the same display
/// index its vertical slot does, so a staggering composition cannot leave the
/// draw and the hit-test reading two different arithmetics.
pub(super) fn apply_row_extent(rows: &mut [PlannedRow], extent: &RowExtent) {
    let (base_dx, base_dw, dx_step, dw_step) = extent.cluster_span.map_or_else(
        || {
            (
                0.0,
                0.0,
                extent.dx_per_row.max(0.0),
                extent.dx_per_row.min(0.0),
            )
        },
        |span| (span.dx, span.dw, span.dx_per_row, span.dw_per_row),
    );
    for row in rows.iter_mut() {
        row.dx = base_dx + dx_step * row.display as f32;
        row.dw = base_dw + dw_step * row.display as f32;
    }
    if let Some((dx, dw)) = extent.selected_offset
        && let Some(row) = rows
            .iter_mut()
            .find(|row| Some(row.display) == extent.selected_display)
    {
        row.dx += dx;
        row.dw += dw;
    }
}

/// A measured cluster as plan input: its row span, its selected row's outward
/// step, and which display line that is. `(None, None, None)` on every upright
/// world, which is what keeps them inert.
pub(in crate::render) type ClusterExtent = (Option<RowSpan>, Option<(f32, f32)>, Option<usize>);

impl OverlayRowPlan {
    /// COMPLETE this frame's plan with the MEASURED half of its extent — the half
    /// that could not be known when it was built — in place, through the same
    /// arithmetic a fresh build runs, so one plan carries one answer and the
    /// per-frame plan COUNT stays a meaningful witness.
    pub(in crate::render) fn complete_row_extent(&mut self, cluster: ClusterExtent) {
        let (cluster_span, selected_offset, selected_display) = cluster;
        apply_row_extent(
            &mut self.rows,
            &RowExtent {
                dx_per_row: self.dx_per_row,
                cluster_span,
                selected_offset,
                selected_display,
            },
        );
    }
}
