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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::render) struct DiagonalComposition {
    pub direction: theme::DiagonalDirection,
    pub row_step: f32,
    pub spine_weight: f32,
    pub spine_corner: f32,
    pub attachment_inset: f32,
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
        }
    }

    fn attachment_x(self, geom: &OverlayGeom) -> f32 {
        let inset = self
            .attachment_inset
            .min((geom.band_w() * 0.5 - self.spine_weight).max(0.0));
        match self.direction {
            theme::DiagonalDirection::Descending => geom.band_x() + inset,
            theme::DiagonalDirection::Ascending => geom.band_x() + geom.band_w() - inset,
        }
    }

    /// The continuous spine through the fixed surface-relative row samples.
    pub fn spine(self, geom: &OverlayGeom, plan: &OverlayRowPlan) -> Option<([f32; 2], [f32; 2])> {
        let first = plan.rows().first()?;
        let last = plan.rows().last()?;
        let x0 = self.attachment_x(geom);
        let x1 = x0 + self.row_step * last.display.saturating_sub(first.display) as f32;
        Some((
            [x0, first.top + first.height * 0.5],
            [x1, last.top + last.height * 0.5],
        ))
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
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) {
        let Some(composition) = active(self) else {
            self.overlay_spine
                .prepare_rotated(device, queue, width, height, &[]);
            return;
        };
        let Some((start, end)) = composition.spine(geom, plan) else {
            self.overlay_spine
                .prepare_rotated(device, queue, width, height, &[]);
            return;
        };
        self.overlay_spine.set_corner(composition.spine_corner);
        self.overlay_spine.set_color(theme::muted().rgba_bytes());
        let segment = crate::selection::spine_segment(start, end, composition.spine_weight);
        self.overlay_spine
            .prepare_rotated(device, queue, width, height, &[segment]);
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
        }
        theme::set_active(theme::DEFAULT_THEME);
    }
}
