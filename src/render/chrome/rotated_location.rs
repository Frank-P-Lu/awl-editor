//! The faceted card's own half of the location cue: which plan line it is,
//! where its row sits, and (for `Raked`) which diagonal composition it
//! belongs to. The mask compose, the shrink-to-fit budget, and the shared
//! preparation call are `TextPipeline::prepare_rotated_location_label`
//! (`render/rotated_location.rs`), which knows nothing about facets, plans, or
//! diagonals — only a string, a rectangle, an axis and two colours.
//! `OverlayGeom`/`OverlayRowPlan` are `chrome`-private, so this is the one
//! place that reads them for the cue.

use super::*;
use crate::render::rotated_location::ROTATED_LOCATION_INSET_PX;

impl TextPipeline {
    /// Read THIS frame's location line (the shared row planner's
    /// `PlanLine::Location`, still the row-plan's own single slot) and, on any
    /// world whose style paints it itself (`draws_inline() == false`), hand
    /// its text and its row band to the rotated-label capability — placed and
    /// coloured per that style below. `header_gap` (the query beat's own calm
    /// divider between the lens strip and the candidate band) is shared by
    /// both styles' shrink-to-fit budget.
    pub(super) fn prepare_overlay_rotated_location(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) {
        let style = theme::active().render_caps.location_style;
        let cue = (!style.draws_inline())
            .then(|| {
                geom.plan
                    .iter()
                    .enumerate()
                    .find_map(|(display, line)| match line {
                        PlanLine::Location(l) => Some((display, l.clone())),
                        _ => None,
                    })
            })
            .flatten()
            .and_then(|(display, label)| plan.rows().get(display).map(|row| (label, *row)));
        let Some((label, row)) = cue else {
            self.rotated_label_pipeline.clear();
            return;
        };

        // The (flush_x, inset_px, axis_deg, color_a, color_b) tuple is the
        // whole difference between the two world expressions — everything
        // else routes through the one shared preparation below.
        let placement = match style {
            theme::LocationStyle::Inline => None, // excluded by `draws_inline()` above
            theme::LocationStyle::RotatedRail => {
                let muted = srgb_u8_to_linear3(theme::muted().rgba_bytes());
                Some((geom.card_x, ROTATED_LOCATION_INSET_PX, 90.0, muted, muted))
            }
            theme::LocationStyle::Raked => self.diagonal_cluster.map(|cluster| {
                // THE MEASURED step, not `DiagonalComposition::row_step` — see
                // `location_axis_deg`'s own doc for why reading the narrow-card
                // yield here is what keeps the cue and the spine beside it
                // from disagreeing on a card too tight for the authored step.
                let axis_deg = super::diagonal::location_axis_deg(cluster.spine_step(), row.height);
                let muted = srgb_u8_to_linear3(theme::muted().rgba_bytes());
                let ink = srgb_u8_to_linear3(theme::base_content().rgba_bytes());
                (geom.text_left + row.dx, 0.0, axis_deg, muted, ink)
            }),
        };

        match placement {
            Some((flush_x, inset_px, axis_deg, color_a, color_b)) => self
                .prepare_rotated_location_label(
                    device,
                    queue,
                    width,
                    height,
                    &label,
                    flush_x,
                    inset_px,
                    row.top,
                    row.height,
                    geom.header_gap,
                    axis_deg,
                    color_a,
                    color_b,
                ),
            // `Raked` with no measured diagonal cluster cannot happen on a
            // shipping world (only a `ListStyle::Diagonal` world is ever
            // assigned `Raked`, and `resolve_diagonal_cluster` always runs
            // before this), but a probe/force path could disagree with the
            // theme's own data — park rather than paint from stale data.
            None => self.rotated_label_pipeline.clear(),
        }
    }
}
