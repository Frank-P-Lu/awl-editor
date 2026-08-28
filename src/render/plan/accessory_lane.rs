//! **THE ROW'S TWO TEXT LANES AND ITS RAIL** — one owner for WHERE a candidate
//! row's NAME ink hangs and WHERE its ACCESSORY ink hangs, whichever composition
//! the card is drawn in.
//!
//! A row is a name at one end of a cluster and an accessory at the other, and
//! both ends move together: an upright card hangs its name off the text column's
//! left edge and its value off the far right one, while a spined card hangs the
//! name on the spine end and the accessory on the outer end, mirroring as a unit.
//! That is ONE rule with two arms, and it was spelled out three times — once in
//! the seat the shaped names are uploaded through, once in the rail owner, once
//! in the surface list the frost measures its box from. Three copies of a rule
//! is three chances for a published number to disagree with a drawn one, which
//! is the whole defect the scene planner exists to close, so the arms live here
//! and every reader asks.
//!
//! **WHAT IS MEASURED HERE IS NOTHING.** The ink WIDTHS still come from the
//! shaped buffers the frame really uploaded (`overlay_row_primary_px` /
//! `overlay_row_secondary_px`), and the rail still comes from its own owner
//! (`overlay_rails`, which the pointer hit-test reads). This module only answers
//! WHERE a width that has already been measured is seated.
//!
//! **PRESENCE IS THE DRAW'S OWN ANSWER, NOT A GUESS.** A lane is reported only
//! when the frame draws it: the accessory lane is gated on `overlay_right_shown`
//! exactly as the accessory upload and the frost's surface list are, so a card
//! that yielded its whole accessory column reports no value lane and no rail
//! rather than reporting where they WOULD have gone. A width a frame did not
//! draw is not geometry.

use super::{OverlayRowPlan, PlannedRow};
use crate::render::TextPipeline;
use crate::render::chrome::OverlayGeom;
use crate::render::rowlayout::ColumnFlow;
use std::collections::BTreeMap;

/// One text lane's seated ink: `x .. x + w`, in the physical pixels the rest of
/// the overlay geometry family speaks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Lane {
    pub x: f32,
    pub w: f32,
}

/// A Range row's rail: the DRAWN track `x .. x + w`, and the deliberately more
/// generous band a pointer is accepted in (`hit_x .. hit_x + hit_w`). Both come
/// off the one `Rail` the draw path and the pointer hit-test share, so the pair
/// publishes the drawn/clickable distinction rather than flattening it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RailLane {
    pub x: f32,
    pub w: f32,
    pub hit_x: f32,
    pub hit_w: f32,
}

/// The accessory cluster of one candidate row, as far as the frame drew it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RowLanes {
    pub label: Option<Lane>,
    pub value: Option<Lane>,
    pub rail: Option<RailLane>,
}

impl TextPipeline {
    /// **THE ACCESSORY COLUMN'S ANCHOR** for display row `display` — the end of
    /// the row the value text and the rail both hang on and grow back inward
    /// from. A spined card asks its measured cluster (whose own outward sign
    /// already carries the mirror and the selected row's outward step); an
    /// upright card's accessory hangs on the far edge of its text column.
    pub(in crate::render) fn overlay_accessory_anchor(
        &self,
        geom: &OverlayGeom,
        display: usize,
    ) -> f32 {
        match self.diagonal_cluster {
            Some(cluster) => cluster.accessory_anchor(display),
            None => geom.row_text_left() + geom.row_text_w(),
        }
    }

    /// That column's `(left, right)` for ink `w` wide — the anchor plus the
    /// world's own accessory [`ColumnFlow`], applied once here so a mirrored
    /// card cannot seat its value one way and report it the other.
    pub(in crate::render) fn overlay_accessory_span(
        &self,
        geom: &OverlayGeom,
        display: usize,
        w: f32,
    ) -> (f32, f32) {
        self.overlay_accessory_flow()
            .span(self.overlay_accessory_anchor(geom, display), w)
    }

    pub(in crate::render) fn overlay_accessory_flow(&self) -> ColumnFlow {
        crate::render::chrome::diagonal::accessory_flow(self)
    }

    /// **WHERE A ROW'S NAME INK BEGINS.** A spined card's name hangs on the
    /// spine end, so on a mirrored world its origin is a function of the ink it
    /// measures — hence `ink_w`, which is `None` for the callers that have not
    /// paid for the measurement and do not need it (an upright card's names
    /// begin at the text column, stepped by the row's own planned offset,
    /// whatever they measure).
    pub(in crate::render) fn overlay_label_origin(
        &self,
        geom: &OverlayGeom,
        row: &PlannedRow,
        ink_w: Option<f32>,
    ) -> f32 {
        match (self.diagonal_cluster, ink_w) {
            (Some(cluster), Some(w)) => cluster.label_origin(row.display, w),
            _ => geom.row_text_left() + row.dx,
        }
    }

    /// Every planned row's lanes, keyed by display line — the projection's own
    /// input, and the ONE place the three measured sources are joined.
    ///
    /// **NOT ON THE FRAME PATH.** Its only caller is the sidecar's own entry
    /// point; the draw path reads the accessors above directly, per surface, as
    /// it always did. One entry per PLANNED DISPLAY LINE, never one per corpus
    /// item, so it inherits the planner's O(visible) bound.
    pub(in crate::render) fn overlay_row_lanes(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> BTreeMap<usize, RowLanes> {
        let primary = self.overlay_row_primary_px(geom);
        // The accessory buffer keeps its last shaped labels even on a frame that
        // yielded the column, so the GATE is the emitter's own
        // `overlay_right_shown` rather than "is there a measurement" — the same
        // question the accessory upload and the frost's surface list ask.
        let secondary = self
            .overlay_right_shown
            .then(|| self.overlay_row_secondary_px(plan));
        let rails: BTreeMap<usize, crate::render::rowlayout::Rail> = self
            .overlay_rails(geom, plan)
            .into_iter()
            .filter_map(|(item, rail)| {
                plan.rows()
                    .iter()
                    .find(|row| row.item == Some(item))
                    .map(|row| (row.display, rail))
            })
            .collect();
        let mut out = BTreeMap::new();
        for row in plan.rows() {
            let k = row.display;
            let label = primary.get(&k).copied().filter(|w| *w > 0.0).map(|w| Lane {
                x: self.overlay_label_origin(geom, row, Some(w)),
                w,
            });
            let value = secondary
                .as_ref()
                .and_then(|m| m.get(&k).copied())
                .filter(|w| *w > 0.0)
                .map(|w| {
                    let (l, r) = self.overlay_accessory_span(geom, k, w);
                    Lane { x: l, w: r - l }
                });
            let rail = rails.get(&k).map(|rail| RailLane {
                x: rail.track[0],
                w: rail.track[2],
                hit_x: rail.hit[0],
                hit_w: rail.hit[2],
            });
            out.insert(k, RowLanes { label, value, rail });
        }
        out
    }
}
