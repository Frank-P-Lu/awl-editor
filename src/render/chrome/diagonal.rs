//! Shared diagonal row composition for summoned row surfaces.
//!
//! The authored quantities below are logical pixels. This module is their one
//! logical-to-device boundary: layout and line geometry multiply by display DPI
//! together, while rasterization remains the selection shader's concern.

use super::*;

const ROW_STEP_LOGICAL: f32 = 7.0;
const SPINE_WEIGHT_LOGICAL: f32 = 1.5;
const SPINE_CORNER_LOGICAL: f32 = 0.75;
const ATTACHMENT_BAND_INSET_LOGICAL: f32 = 44.0;
const CLUSTER_CONNECTOR_LOGICAL: f32 = 10.0;
const SELECTED_OUTWARD_LOGICAL: f32 = 4.0;
const SELECTED_SPINE_WEIGHT_LOGICAL: f32 = 3.0;

/// THE RESPONSIVE BOUND on the spine's total travel, as a fraction of the side
/// territory the card actually has (its band less the attachment inset, the
/// connector and the selected row's outward step). It is a bound, never the
/// travel itself: an ordinary card affords the authored per-row step outright
/// and this never binds; a genuinely cramped one gives up rake proportionally
/// rather than collapsing to an upright line.
///
/// It is deliberately a property of the SURFACE alone. Sizing the travel from
/// the widest row currently on screen — the first cut — made the spine's whole
/// ANGLE a function of the scroll position: a long filename scrolling out of a
/// picker swung the line from nearly upright to a full rake, moving every row
/// with it (item 222).
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

/// The one measured row-cluster layout shared by diagonal text, accessory
/// upload, Range geometry, and the planner's clickable row-side span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct DiagonalClusterRail {
    direction: theme::DiagonalDirection,
    /// The row's whole territory beside the spine — the CLUSTER BUDGET, a
    /// property of the card alone. The label runs from the spine end of it and
    /// the accessory is right-aligned to the far end, exactly as an upright
    /// world's name and chord share one text column. Sizing the rail from the
    /// rows instead is what let a scroll move it.
    cluster_w: f32,
    accessory_w: f32,
    connector: f32,
    spine_start: f32,
    spine_step: f32,
    span: RowSpan,
    selected_display: Option<usize>,
    selected_shift: f32,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(in crate::render) struct DiagonalClusterProbe {
    pub cluster_w: f32,
    pub accessory_w: f32,
    pub span: RowSpan,
    rail: DiagonalClusterRail,
}

#[cfg(test)]
impl DiagonalClusterProbe {
    pub(in crate::render) fn label_left(self, display: usize) -> f32 {
        self.rail.label_left(display)
    }

    pub(in crate::render) fn accessory_left(self, display: usize) -> f32 {
        self.rail.accessory_left(display)
    }

    pub(in crate::render) fn accessory_right(self, display: usize) -> f32 {
        self.rail.accessory_right(display)
    }

    pub(in crate::render) fn selected_offset(self) -> (f32, f32) {
        self.rail.selected_offset()
    }

    /// Where display row `display`'s SPINE segment stands — the composition's
    /// stationary surface, independent of anything a row measures.
    pub(in crate::render) fn spine_x(self, display: usize) -> f32 {
        self.rail.spine_x(display)
    }
}

impl DiagonalClusterRail {
    fn new(
        composition: DiagonalComposition,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        selected_display: Option<usize>,
        cluster_w: f32,
        accessory_w: f32,
    ) -> Self {
        let band_x = geom.band_x();
        let band_right = band_x + geom.band_w();
        let cluster_w = cluster_w.max(0.0);
        let accessory_w = accessory_w.max(0.0).min(cluster_w);
        let rows = plan.rows().len().saturating_sub(1) as f32;
        let inset = attachment_inset(composition, geom);
        // THE SPINE IS A FIXED SURFACE-RELATIVE LINE. Its travel is reserved off
        // the card's own side territory — never off the rows in front of it — so
        // filtering and scrolling leave the rake exactly where it was, and the
        // cluster elides into what is left (`diagonal_cluster_budget` subtracts
        // the same reservation). A selected cluster's one outward step is held
        // back at the far end, so a last visible selected row cannot push its
        // accessory past the card clip.
        let step = if rows > 0.0 {
            spine_travel(composition, geom, plan.rows().len()) / rows
        } else {
            0.0
        };
        let (spine_start, spine_step, span) = match composition.direction {
            theme::DiagonalDirection::Descending => (
                band_x + inset,
                step,
                RowSpan {
                    dx: inset,
                    dw: 0.0,
                    dx_per_row: step,
                    dw_per_row: 0.0,
                },
            ),
            theme::DiagonalDirection::Ascending => (
                band_right - inset,
                -step,
                RowSpan {
                    dx: 0.0,
                    dw: -inset,
                    dx_per_row: 0.0,
                    dw_per_row: -step,
                },
            ),
        };
        Self {
            direction: composition.direction,
            cluster_w,
            accessory_w,
            connector: composition.connector,
            spine_start,
            spine_step,
            span,
            selected_display,
            selected_shift: composition.selected_outward,
        }
    }

    pub(in crate::render) fn span(self) -> RowSpan {
        self.span
    }

    fn spine_x(self, display: usize) -> f32 {
        self.spine_start + self.spine_step * display as f32
    }

    fn shift(self, display: usize) -> f32 {
        let shift = if self.selected_display == Some(display) {
            self.selected_shift
        } else {
            0.0
        };
        shift * self.direction.sign()
    }

    pub(in crate::render) fn selected_offset(self) -> (f32, f32) {
        let shift = self.selected_shift * self.direction.sign();
        (shift, shift)
    }

    pub(in crate::render) fn row_plan(
        self,
    ) -> (Option<RowSpan>, Option<(f32, f32)>, Option<usize>) {
        (
            Some(self.span()),
            Some(self.selected_offset()),
            self.selected_display,
        )
    }

    fn spine(self, plan: &OverlayRowPlan) -> Option<([f32; 2], [f32; 2])> {
        let first = plan.rows().first()?;
        let last = plan.rows().last()?;
        Some((
            [self.spine_x(first.display), first.top + first.height * 0.5],
            [self.spine_x(last.display), last.top + last.height * 0.5],
        ))
    }

    pub(in crate::render) fn label_left(self, display: usize) -> f32 {
        let spine = self.spine_x(display) + self.shift(display);
        match self.direction {
            theme::DiagonalDirection::Descending => spine + self.connector,
            theme::DiagonalDirection::Ascending => spine - self.connector - self.cluster_w,
        }
    }

    /// The far end of the row's territory — where the accessory column's ink
    /// ends, right-aligned into it the way an upright card right-aligns its
    /// chord to the card's own text edge.
    pub(in crate::render) fn accessory_right(self, display: usize) -> f32 {
        self.label_left(display) + self.cluster_w
    }

    #[cfg(test)]
    pub(in crate::render) fn accessory_left(self, display: usize) -> f32 {
        self.accessory_right(display) - self.accessory_w
    }

    pub(in crate::render) fn accessory_w(self) -> f32 {
        self.accessory_w
    }
}

/// The attachment band's inset — the authored value, yielding on a card too
/// narrow to seat it and still leave the far half free.
fn attachment_inset(composition: DiagonalComposition, geom: &OverlayGeom) -> f32 {
    composition
        .attachment_inset
        .min((geom.band_w() * 0.5 - composition.connector).max(0.0))
}

/// The spine's TOTAL horizontal travel across the drawn rows — the authored
/// per-row step over the rows there are, bounded by
/// [`TRAVEL_MAX_BAND_FRACTION`] of the card's own side territory. A pure
/// function of the composition and the card: no row, no label, no scroll
/// position enters it.
fn spine_travel(composition: DiagonalComposition, geom: &OverlayGeom, rows: usize) -> f32 {
    let steps = rows.saturating_sub(1) as f32;
    let room = (geom.band_w()
        - attachment_inset(composition, geom)
        - composition.connector
        - composition.selected_outward)
        .max(0.0);
    (composition.row_step.abs() * steps).min(room * TRAVEL_MAX_BAND_FRACTION)
}

impl DiagonalComposition {
    /// Resolve every authored quantity at the single logical→device boundary.
    pub fn resolve(direction: theme::DiagonalDirection, dpi: f32) -> Self {
        let scale = dpi.max(1.0);
        Self {
            direction,
            row_step: direction.sign() * ROW_STEP_LOGICAL * scale,
            spine_weight: SPINE_WEIGHT_LOGICAL * scale,
            spine_corner: SPINE_CORNER_LOGICAL * scale,
            attachment_inset: ATTACHMENT_BAND_INSET_LOGICAL * scale,
            connector: CLUSTER_CONNECTOR_LOGICAL * scale,
            selected_outward: SELECTED_OUTWARD_LOGICAL * scale,
            selected_spine_weight: SELECTED_SPINE_WEIGHT_LOGICAL * scale,
        }
    }
}

impl TextPipeline {
    /// THE SIDE TERRITORY a diagonal card owes its composition, beyond the row
    /// cluster itself: the attachment inset the spine stands on, the connector
    /// from spine to cluster, the selected row's outward step, and the travel the
    /// deepest row is stepped by.
    ///
    /// A content-hugging card (item 51) sizes itself from its measured rows, and
    /// a card measured from rows ALONE is exactly one cluster wide — leaving the
    /// composition nothing. `DiagonalClusterRail::new` then spends the card's
    /// whole band on inset + connector + cluster, so `available_travel` collapses
    /// to zero and the spine stands upright, and the shaper's own
    /// `diagonal_cluster_budget` cuts the same territory back out of `text_w`, so
    /// `rowlayout::fits` fails and the secondary column — the key chords — yields
    /// entirely. Reserving it here is what makes a hugged diagonal card able to
    /// hold what the composition draws into it.
    ///
    /// `0.0` on every upright world, so their hug width is untouched.
    /// `rows` is the plan's own drawn row count — the SAME count
    /// [`DiagonalClusterRail::new`] divides its travel across, so the reserve and
    /// the spend can never be derived from two different windows.
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

    pub(in crate::render) fn diagonal_is_active(&self) -> bool {
        active(self).is_some()
    }

    /// The width a diagonal row's CLUSTER may occupy — the band less the
    /// attachment inset, the connector, the spine's reserved travel and the
    /// selected row's outward step. Every term is a property of the card, so the
    /// budget (and therefore every row's elision) is the same number at every
    /// scroll position and under every filter.
    pub(in crate::render) fn diagonal_cluster_budget(
        &self,
        geom: &OverlayGeom,
        rows: usize,
    ) -> Option<f32> {
        let composition = active(self)?;
        let inset = attachment_inset(composition, geom);
        // The rail is anchored to the BAND (the spine stands on `band_x + inset`)
        // but it is clipped by the TEXT column, which is one `hpad` narrower at
        // each edge. Without that term the deepest row's accessory ran past the
        // clip and its last glyph was cut off the card.
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
        // The accessory column's own INK width — how far left of the rail's far
        // edge a chord, a value or a Range readout reaches. Only the rails and
        // hit bands read it; the column's right edge is the rail's, and does not
        // move with it.
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
        self.diagonal_cluster.map(|rail| DiagonalClusterProbe {
            cluster_w: rail.cluster_w,
            accessory_w: rail.accessory_w,
            span: rail.span,
            rail,
        })
    }
}

pub(in crate::render) fn active(pipeline: &TextPipeline) -> Option<DiagonalComposition> {
    match crate::render::effective_list_style() {
        theme::ListStyle::Diagonal(direction) => {
            Some(DiagonalComposition::resolve(direction, pipeline.dpi))
        }
        theme::ListStyle::Pane | theme::ListStyle::Bars { .. } => None,
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
                let spine_x = cluster.spine_x(row.display);
                let mid_y = row.top + row.height * 0.5;
                let local = crate::selection::spine_segment(
                    [spine_x, row.top + 2.0],
                    [spine_x, row.bottom() - 2.0],
                    composition.selected_spine_weight,
                );
                let connector_end = match composition.direction {
                    theme::DiagonalDirection::Descending => {
                        [cluster.label_left(row.display), mid_y]
                    }
                    theme::DiagonalDirection::Ascending => {
                        [cluster.accessory_right(row.display), mid_y]
                    }
                };
                [
                    local,
                    crate::selection::spine_segment(
                        [spine_x, mid_y],
                        connector_end,
                        composition.selected_spine_weight,
                    ),
                ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_registry_scales_every_diagonal_quantity_once_with_dpi() {
        let one = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 1.0);
        let two = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 2.0);
        assert_eq!(two.row_step, one.row_step * 2.0);
        assert_eq!(two.spine_weight, one.spine_weight * 2.0);
        assert_eq!(two.spine_corner, one.spine_corner * 2.0);
        assert_eq!(two.attachment_inset, one.attachment_inset * 2.0);
        assert_eq!(two.selected_outward, one.selected_outward * 2.0);
        assert_eq!(two.selected_spine_weight, one.selected_spine_weight * 2.0);
        assert!(
            one.row_step > 0.0,
            "the diagonal law must not pass on an inert composition"
        );
    }

    #[test]
    fn one_composition_owns_the_two_mirrored_orientations() {
        let down = DiagonalComposition::resolve(theme::DiagonalDirection::Descending, 1.0);
        let up = DiagonalComposition::resolve(theme::DiagonalDirection::Ascending, 1.0);
        assert_eq!(down.row_step, -up.row_step);
        assert_eq!(down.spine_weight, up.spine_weight);
        assert_eq!(down.spine_corner, up.spine_corner);
        assert_eq!(down.attachment_inset, up.attachment_inset);
        assert_eq!(down.selected_outward, up.selected_outward);
        assert_eq!(down.selected_spine_weight, up.selected_spine_weight);
        assert!(
            down.selected_spine_weight > down.spine_weight,
            "the selected local spine must visibly thicken over its resting stroke"
        );
    }

    #[test]
    fn only_world_data_names_mangrove_and_magpie() {
        let _g = crate::testlock::serial();
        let renderer = include_str!("diagonal.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(!renderer.contains("Mangrove"));
        assert!(!renderer.contains("Magpie"));

        let mut diagonal = Vec::new();
        for world in theme::THEMES {
            match world.render_caps.list_style {
                theme::ListStyle::Diagonal(direction) => diagonal.push((world.name, direction)),
                theme::ListStyle::Pane | theme::ListStyle::Bars { .. } => {}
            }
        }
        assert_eq!(
            diagonal,
            vec![
                ("Mangrove", theme::DiagonalDirection::Descending),
                ("Magpie", theme::DiagonalDirection::Ascending),
            ]
        );
        for name in ["Mangrove", "Magpie"] {
            theme::set_active_by_name(name).unwrap();
            assert_ne!(
                theme::muted(),
                theme::primary(),
                "{name}: spine ink is never the accent"
            );
            assert_ne!(
                theme::muted(),
                theme::base_content(),
                "{name}: the selected local spine must brighten over the resting muted ink"
            );
        }
        theme::set_active(theme::DEFAULT_THEME);
    }
}
