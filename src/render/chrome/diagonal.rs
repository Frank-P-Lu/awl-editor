//! Shared diagonal row composition for summoned row surfaces.
//!
//! The authored quantities below are logical pixels — and they now say so in
//! their TYPE rather than in a `_LOGICAL` suffix, which was the right instinct
//! with an unenforceable mechanism. They resolve through the one owner every
//! other chrome length passes, so the spine tracks zoom as well as DPI instead
//! of carrying a second, DPI-only, grow-only scale of its own.

use super::*;
use crate::render::rowlayout::ColumnFlow;

const ROW_STEP: Logical = Logical(7.0);
const SPINE_WEIGHT: Logical = Logical(1.5);
const SPINE_CORNER: Logical = Logical(0.75);
const ATTACHMENT_BAND_INSET: Logical = Logical(44.0);
const CLUSTER_CONNECTOR: Logical = Logical(10.0);
const SELECTED_OUTWARD: Logical = Logical(4.0);
const SELECTED_SPINE_WEIGHT: Logical = Logical(3.0);

/// THE RESPONSIVE BOUND on the spine's total travel, as a fraction of the side
/// territory the card has. A bound, never the travel itself: an ordinary card
/// affords the authored per-row step outright, a cramped one gives up rake
/// proportionally rather than collapsing to an upright line. A property of the
/// SURFACE alone — sized from the widest row on screen, the spine's whole ANGLE
/// became a function of the scroll position.
const TRAVEL_MAX_BAND_FRACTION: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct DiagonalComposition {
    pub direction: theme::DiagonalDirection,
    pub row_step: f32,
    pub spine_weight: f32,
    pub spine_corner: f32,
    pub attachment_inset: f32,
    pub connector: f32,
    pub selected_outward: f32,
    pub selected_spine_weight: f32,
}

/// The attachment band's inset, yielding on a card too narrow to seat it and
/// still leave the far half free.
fn attachment_inset(composition: DiagonalComposition, geom: &OverlayGeom) -> f32 {
    composition
        .attachment_inset
        .min((geom.band_w() * 0.5 - composition.connector).max(0.0))
}

/// The spine's TOTAL horizontal travel across the drawn rows: the authored
/// per-row step, bounded by [`TRAVEL_MAX_BAND_FRACTION`] of the card's side
/// territory. No row, no label and no scroll position enters it.
fn spine_travel(composition: DiagonalComposition, geom: &OverlayGeom, rows: usize) -> f32 {
    let steps = rows.saturating_sub(1) as f32;
    let room = (geom.band_w()
        - attachment_inset(composition, geom)
        - composition.connector
        - composition.selected_outward)
        .max(0.0);
    (composition.row_step.abs() * steps).min(room * TRAVEL_MAX_BAND_FRACTION)
}

/// THE SELECTED ROW'S MARK — a CHEVRON, and the one owner of its geometry.
///
/// Its vertex sits on the spine at `spine_x`, midway between the two arm ends,
/// and its arms open to `arm_x` at `top` and `bottom`. Deriving the vertex FROM
/// the arm ends (rather than taking a row centre as a third argument) makes the
/// mirror structural: the two arms cannot drift out of symmetry, because there
/// is no second quantity that could disagree.
///
/// Pure — no device, no clock, no theme — so a law can grade the shape this
/// frame would actually draw. The property that identifies the mark is that
/// NEITHER arm is axis-aligned for any row of nonzero height and nonzero reach;
/// the tick-plus-connector pair this replaced drew one vertical segment and one
/// horizontal one, and both spanned the same bounding box, so a law that counts
/// instances or measures extent cannot tell the two shapes apart.
pub(in crate::render) fn selected_chevron(
    spine_x: f32,
    arm_x: f32,
    top: f32,
    bottom: f32,
    thickness: f32,
) -> [([f32; 2], [f32; 2], [f32; 2]); 2] {
    let vertex = [spine_x, (top + bottom) * 0.5];
    [
        crate::selection::spine_segment(vertex, [arm_x, top], thickness),
        crate::selection::spine_segment(vertex, [arm_x, bottom], thickness),
    ]
}

impl DiagonalComposition {
    /// Resolve every authored quantity at the ONE logical→device boundary
    /// (`zoom * dpi`), the same `scale` the text beside the spine was sized at.
    pub fn resolve(direction: theme::DiagonalDirection, scale: f32) -> Self {
        Self {
            direction,
            row_step: direction.sign() * ROW_STEP.px(scale),
            spine_weight: SPINE_WEIGHT.px(scale),
            spine_corner: SPINE_CORNER.px(scale),
            attachment_inset: ATTACHMENT_BAND_INSET.px(scale),
            connector: CLUSTER_CONNECTOR.px(scale),
            selected_outward: SELECTED_OUTWARD.px(scale),
            selected_spine_weight: SELECTED_SPINE_WEIGHT.px(scale),
        }
    }
}

mod cluster;
mod location;
#[cfg(test)]
pub(in crate::render) use cluster::DiagonalClusterProbe;
pub(in crate::render) use cluster::DiagonalClusterRail;
use cluster::label_flow_of;
pub(in crate::render) use location::location_axis_deg;

impl TextPipeline {
    /// THE SIDE TERRITORY a diagonal card owes its composition beyond the row
    /// cluster: the attachment inset the spine stands on, the connector, the
    /// selected row's outward step and the deepest row's travel.
    ///
    /// A card that hugs its measured ROWS is exactly one cluster wide and leaves
    /// the composition nothing: the travel collapses to zero (an upright spine)
    /// and `diagonal_cluster_budget` cuts the same territory back out of `text_w`
    /// until `rowlayout::fits` drops the key chords entirely. `0.0` on every
    /// upright world, so their hug width is untouched. `rows` is the plan's own
    /// drawn count — the same one the travel is divided across.
    pub(in crate::render) fn diagonal_side_reserve_px(&self, rows: usize) -> f32 {
        let Some(composition) = active(self) else {
            return 0.0;
        };
        let rows = rows.saturating_sub(1) as f32;
        composition.attachment_inset
            + composition.connector
            + composition.selected_outward
            + composition.row_step.abs() * rows
    }

    /// The width a diagonal row's CLUSTER may occupy — the band less the
    /// attachment inset, the connector, the reserved travel and the selected
    /// row's outward step. Every term is a property of the card, so a row's
    /// elision is the same number at every scroll position and every filter.
    pub(in crate::render) fn diagonal_cluster_budget(
        &self,
        geom: &OverlayGeom,
        rows: usize,
    ) -> Option<f32> {
        let composition = active(self)?;
        let inset = attachment_inset(composition, geom);
        // Anchored to the BAND (the spine stands on `band_x + inset`), clipped by
        // the TEXT column, one `hpad` narrower at each edge — without that term
        // the deepest row's accessory lost its last glyph to the clip.
        Some(
            geom.text_w.min(
                (geom.band_w()
                    - inset
                    - composition.connector
                    - spine_travel(composition, geom, rows)
                    - composition.selected_outward
                    - self.overlay_text_hpad())
                .max(0.0),
            ),
        )
    }

    /// THE ONE READ of the measured cluster AS PLAN INPUT — its row span, its
    /// selected row's outward step, and which display line that is. A frame builds
    /// its plan BEFORE the cluster exists and completes it after; the standalone
    /// pointer/report entry points, with no frame to ride, plan against whatever
    /// the last drawn frame measured. Both doors ask this one question.
    pub(in crate::render) fn diagonal_row_extent(&self) -> ClusterExtent {
        self.diagonal_cluster
            .map_or((None, None, None), DiagonalClusterRail::row_plan)
    }

    pub(super) fn resolve_diagonal_cluster(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
    ) -> Option<DiagonalClusterRail> {
        let composition = active(self)?;
        let cluster_w = self.diagonal_cluster_budget(geom, plan.rows().len())?;
        // The accessory column's own INK width — how far in from the rail's far
        // edge a chord, value or Range readout reaches. Only rails and hit bands
        // read it; the column's outer edge is the rail's, and does not move.
        let secondary = self.overlay_row_secondary_px(geom);
        let mut accessory_w = plan
            .rows()
            .iter()
            .map(|row| secondary.get(&row.display).copied().unwrap_or(0.0))
            .fold(0.0, f32::max);
        for row in plan.rows() {
            let Some(item) = row.item else {
                continue;
            };
            if self.overlay_ranges.get(item).copied().flatten().is_some() {
                let value_w = secondary.get(&row.display).copied().unwrap_or(0.0);
                accessory_w = accessory_w.max(
                    value_w + crate::render::rowlayout::rail_accessory_width(self.overlay_lh()),
                );
            }
        }
        Some(DiagonalClusterRail::new(
            composition,
            geom,
            plan,
            vis.rows().first().copied(),
            cluster_w,
            accessory_w,
        ))
    }

    #[cfg(test)]
    pub(in crate::render) fn diagonal_cluster_probe(&self) -> Option<DiagonalClusterProbe> {
        self.diagonal_cluster.map(DiagonalClusterProbe::of)
    }
}

/// WHICH WAY THIS WORLD'S ACCESSORY COLUMN GROWS, answered from the world alone
/// so the chord column can be SHAPED before any cluster has been measured — the
/// shaping pass runs before `resolve_diagonal_cluster` can exist. Upright worlds
/// keep the right-aligned secondary column every card has always had.
pub(in crate::render) fn accessory_flow(pipeline: &TextPipeline) -> ColumnFlow {
    match active(pipeline) {
        None => ColumnFlow::Leftward,
        Some(composition) => label_flow_of(composition.direction).mirrored(),
    }
}

pub(in crate::render) fn active(pipeline: &TextPipeline) -> Option<DiagonalComposition> {
    match crate::render::effective_list_style() {
        theme::ListStyle::Diagonal(direction) => Some(DiagonalComposition::resolve(
            direction,
            pipeline.metrics.scale,
        )),
        // `Rules` also arranges with drawn lines and is deliberately not this:
        // a spine is one geometry the rows hang off, a rule is a boundary.
        theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => None,
    }
}

impl TextPipeline {
    pub(super) fn prepare_diagonal_spine(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        plan: &OverlayRowPlan,
        vis: &VisualSelection,
    ) {
        let Some(composition) = active(self) else {
            self.overlay_spine
                .prepare_rotated(device, queue, width, height, &[]);
            self.overlay_spine_selected
                .prepare_rotated(device, queue, width, height, &[]);
            return;
        };
        let Some(cluster) = self.diagonal_cluster else {
            self.overlay_spine
                .prepare_rotated(device, queue, width, height, &[]);
            self.overlay_spine_selected
                .prepare_rotated(device, queue, width, height, &[]);
            return;
        };
        let Some((start, end)) = cluster.spine(plan) else {
            self.overlay_spine
                .prepare_rotated(device, queue, width, height, &[]);
            self.overlay_spine_selected
                .prepare_rotated(device, queue, width, height, &[]);
            return;
        };
        self.overlay_spine.set_corner(composition.spine_corner);
        self.overlay_spine.set_color(theme::muted().rgba_bytes());
        let segment = crate::selection::spine_segment(start, end, composition.spine_weight);
        self.overlay_spine
            .prepare_rotated(device, queue, width, height, &[segment]);

        let selected_segments = plan
            .rows()
            .iter()
            .filter(|row| vis.reads_selected(row.display))
            .flat_map(|row| {
                // The chevron inscribes exactly the bounding box the previous
                // tick-plus-connector pair spanned — the same outward reach, the
                // same row inset at both ends — so every term that reserves
                // territory for this mark (`diagonal_side_reserve_px`,
                // `diagonal_cluster_budget`, `spine_travel`) still describes the
                // drawn shape and needs no compensating adjustment.
                //
                // The arm reaches the cluster's SPINE end — the edge the row's own
                // name hugs — READ FROM the cluster rather than re-derived, so it
                // keeps the selected row's outward shift and mirrors with the
                // cluster, opening right on a Descending world and left on an
                // Ascending one, without a second sign living here.
                selected_chevron(
                    cluster.spine_x(row.display),
                    cluster.label_anchor(row.display),
                    row.top + 2.0,
                    row.bottom() - 2.0,
                    composition.selected_spine_weight,
                )
            })
            .collect::<Vec<_>>();
        self.overlay_spine_selected
            .set_corner(composition.spine_corner);
        self.overlay_spine_selected
            .set_color(theme::base_content().rgba_bytes());
        self.overlay_spine_selected.prepare_rotated(
            device,
            queue,
            width,
            height,
            &selected_segments,
        );
    }
}
