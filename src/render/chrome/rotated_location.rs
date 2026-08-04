//! The faceted card's own half of the rotated secondary-location cue: which
//! plan line it is, and where its row sits. The mask compose, the axis, and
//! the shrink-to-fit budget are `TextPipeline::prepare_rotated_location_label`
//! (`render/rotated_location.rs`), which knows nothing about facets or plans
//! — only a string and a rectangle. `OverlayGeom`/`OverlayRowPlan` are
//! `chrome`-private, so this is the one place that reads them for the cue.

use super::*;

impl TextPipeline {
    /// Read THIS frame's location line (the shared row planner's
    /// `PlanLine::Location`, still the row-plan's own single slot) and, on a
    /// `RotatedRail` world, hand its text, its row band, and the ONE known
    /// blank stretch above it (`geom.header_gap` — the "calm divider" the
    /// query beat carves between the lens strip and the candidate band,
    /// deliberately empty on every faceted card) to the rotated-label
    /// capability.
    pub(super) fn prepare_overlay_rotated_location(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) {
        let cue = (theme::active().render_caps.location_style == theme::LocationStyle::RotatedRail)
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
            .and_then(|(display, label)| {
                plan.rows()
                    .get(display)
                    .map(|row| (label, row.top, row.height))
            });
        match cue {
            Some((label, row_top, row_height)) => self.prepare_rotated_location_label(
                device,
                queue,
                width,
                height,
                &label,
                geom.card_x,
                row_top,
                row_height,
                geom.header_gap,
            ),
            None => self.rotated_label_pipeline.clear(),
        }
    }
}
