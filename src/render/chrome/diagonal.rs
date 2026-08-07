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

/// The air between the row cluster's OUTER end and the mark that stands beyond
/// it. Its own quantity rather than a second reading of [`CLUSTER_CONNECTOR`]:
/// that one is a connector — it joins the name to the spine and its length is
/// the join — where this is a gap, and the mark it holds off is authored
/// per world.
const MARKER_GAP: Logical = Logical(7.0);

/// The mark's vertical inset inside its row, at BOTH ends, before the world's
/// own [`theme::DiagonalMark::aperture`] narrows it further. A logical length,
/// so the mark keeps its proportion on a Retina panel instead of shrinking to a
/// hairline gap of device pixels.
const MARKER_ROW_INSET: Logical = Logical(2.0);

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
    /// The selected mark's authored stroke, gap, reach and row inset, resolved
    /// at the same boundary as every length above. `mark_weight` and
    /// `mark_reach` come from the WORLD (`theme::DiagonalMark`); the gap and the
    /// inset are the composition's own.
    pub mark_weight: f32,
    pub mark_gap: f32,
    pub mark_reach: f32,
    pub mark_row_inset: f32,
    /// `aperture` is dimensionless and therefore does NOT pass the scale
    /// boundary — it is a fraction of the row it is applied to.
    pub mark_aperture: f32,
}

impl DiagonalComposition {
    /// THE SIDE TERRITORY THE MARK ITSELF OWNS, beyond the cluster's outer end:
    /// the gap it stands off by plus its own full horizontal extent. Every term
    /// that reserves room for the composition reads this one owner, so a world
    /// whose mark is authored wider cannot silently push its own ink off the
    /// card's edge.
    pub fn mark_lane(self) -> f32 {
        self.mark_gap + self.mark_reach * 2.0
    }
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
        - composition.selected_outward
        - composition.mark_lane())
    .max(0.0);
    (composition.row_step.abs() * steps).min(room * TRAVEL_MAX_BAND_FRACTION)
}

/// THE SELECTED ROW'S MARK — a CHEVRON, and the one owner of its geometry: it
/// IS the [`crate::selection::chevron_arms`] shared owner at a derived
/// parameterization, rather than a shape that merely resembles it (see
/// `render/tests/marker_chevron_owner_item247.rs`'s Law 1, which binds the two
/// point-for-point).
///
/// Its VERTEX sits at `vertex_x` — the mark's row-facing end, one gap outward
/// from the cluster's outer edge — midway between `top` and `bottom`, and its
/// arms open AWAY from the row to `arm_x`. The mark is upright: it does not
/// turn, so the shape is the owner's at a zero turn and the two worlds differ
/// only in the SIGN of `vertex_x - arm_x`, which is the same signed dial the
/// cluster itself mirrors on.
///
/// `reach` and `spread` are the arm-end pair expressed in the owner's terms:
/// `reach` signed half the vertex-to-arm distance (so a Descending world's
/// outward-right mark and an Ascending world's outward-left one are the same
/// expression with opposite sign), `spread` half the mark's own vertical span.
///
/// Pure — no device, no clock, no theme — so a law can grade the shape this
/// frame would actually draw. The property that identifies the mark is that
/// NEITHER arm is axis-aligned for any nonzero span and nonzero reach; the
/// tick-plus-connector pair this replaced drew one vertical segment and one
/// horizontal one, and both spanned the same bounding box, so a law that counts
/// instances or measures extent cannot tell the two shapes apart.
pub(in crate::render) fn selected_chevron(
    vertex_x: f32,
    arm_x: f32,
    top: f32,
    bottom: f32,
    thickness: f32,
) -> [([f32; 2], [f32; 2], [f32; 2]); 2] {
    let reach = (vertex_x - arm_x) * 0.5;
    let spread = (top - bottom) * 0.5;
    let center = [(vertex_x + arm_x) * 0.5, (top + bottom) * 0.5];
    crate::selection::chevron_arms(center, reach, spread, 0.0, thickness)
}

impl DiagonalComposition {
    /// Resolve every authored quantity at the ONE logical→device boundary
    /// (`zoom * dpi`), the same `scale` the text beside the spine was sized at.
    /// The world's own mark is authored in the same space and passes the same
    /// boundary; only its dimensionless aperture does not.
    pub fn resolve(spine: theme::DiagonalSpine, scale: f32) -> Self {
        let direction = spine.direction;
        Self {
            direction,
            row_step: direction.sign() * ROW_STEP.px(scale),
            spine_weight: SPINE_WEIGHT.px(scale),
            spine_corner: SPINE_CORNER.px(scale),
            attachment_inset: ATTACHMENT_BAND_INSET.px(scale),
            connector: CLUSTER_CONNECTOR.px(scale),
            selected_outward: SELECTED_OUTWARD.px(scale),
            mark_weight: Logical(spine.mark.weight).px(scale),
            mark_gap: MARKER_GAP.px(scale),
            mark_reach: Logical(spine.mark.reach).px(scale),
            mark_row_inset: MARKER_ROW_INSET.px(scale),
            mark_aperture: spine.mark.aperture,
        }
    }

    /// THE MARK'S OWN VERTICAL SPAN inside a planned row: the row's inset
    /// height narrowed by the world's aperture, centred on the row. Returned as
    /// `(top, bottom)` for [`selected_chevron`] — the one place the aperture is
    /// applied, so an authored fraction cannot mean one thing in the draw and
    /// another in a probe.
    pub fn mark_span_y(self, row_top: f32, row_height: f32) -> (f32, f32) {
        let mid = row_top + row_height * 0.5;
        let half =
            ((row_height - self.mark_row_inset * 2.0).max(0.0) * 0.5 * self.mark_aperture).max(0.0);
        (mid - half, mid + half)
    }
}

mod cluster;
mod location;
mod offband;
#[cfg(test)]
pub(in crate::render) use cluster::DiagonalClusterProbe;
pub(in crate::render) use cluster::DiagonalClusterRail;
use cluster::label_flow_of;
pub(in crate::render) use location::location_axis_deg;
#[cfg(test)]
pub(in crate::render) use offband::FOOT_CONTINUES_THE_LEAN;

impl TextPipeline {
    /// THE SIDE TERRITORY a diagonal card owes its composition beyond the row
    /// cluster: the attachment inset the spine stands on, the connector, the
    /// selected row's outward step, the mark's own lane beyond the cluster's
    /// outer end, and the deepest row's travel.
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
            + composition.mark_lane()
            + composition.row_step.abs() * rows
    }

    /// The width a diagonal row's CLUSTER may occupy — the band less the
    /// attachment inset, the connector, the reserved travel, the selected row's
    /// outward step and the mark's lane. Every term is a property of the card,
    /// so a row's elision is the same number at every scroll position and every
    /// filter.
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
                    - composition.mark_lane()
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
        theme::ListStyle::Diagonal(spine) => {
            Some(DiagonalComposition::resolve(spine, pipeline.metrics.scale))
        }
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
                // THE MARK STANDS ON THE ROW'S OUTER EDGE, away from the spine —
                // pointing back INTO the row rather than out of the card — and
                // both of its abscissae are READ FROM the cluster
                // (`DiagonalClusterRail::mark_span`) rather than re-derived here.
                // That is what mirrors it: the cluster's own outward sign already
                // put its accessory end on the card side, so the mark inherits
                // the mirror and the selected row's outward shift together, with
                // no second sign and no world branch living in the draw.
                let (top, bottom) = composition.mark_span_y(row.top, row.height);
                let (vertex_x, arm_x) = cluster.mark_span(row.display);
                selected_chevron(vertex_x, arm_x, top, bottom, composition.mark_weight)
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
