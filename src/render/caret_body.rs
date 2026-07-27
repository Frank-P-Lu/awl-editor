//! CARET BODY — the shared minimum resting silhouette for proportional carets.
//!
//! The geometry owner supplies real raster ink. This owner turns it into the
//! visible Block body and decides whether a settled Morph needs that same body
//! behind its recoloured glyph, with no punctuation or world identity branch.

use super::*;

/// Full raster ink box, relative to the glyph pen origin.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct InkBox {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl InkBox {
    pub fn descent(self) -> f32 {
        (self.height - self.top).max(0.0)
    }
}

/// The smallest visible body at zoom 1. Width, height, and area are separate:
/// brackets need width, dashes need height, and commas need all three.
pub(super) const CARET_VISUAL_BODY_MIN_W: f32 = 6.5;
pub(super) const CARET_VISUAL_BODY_MIN_H: f32 = 12.0;
pub(super) const CARET_VISUAL_BODY_MIN_AREA: f32 = 96.0;

/// Apply the authored floor without flattening ordinary glyph-responsive carets.
pub(super) fn caret_visual_body_dims(ink: InkBox, px: f32) -> (f32, f32) {
    let mut w = ink.width.max(CARET_VISUAL_BODY_MIN_W * px);
    let mut h = (ink.height + 2.0 * CARET_INK_PAD * px).max(CARET_VISUAL_BODY_MIN_H * px);
    let min_area = CARET_VISUAL_BODY_MIN_AREA * px * px;
    if w * h < min_area {
        let grow = (min_area / (w * h)).sqrt();
        w *= grow;
        h *= grow;
    }
    (w, h)
}

impl TextPipeline {
    /// The sole Morph support decision: prepare its shared body or clear a
    /// previous frame's block before the glyph silhouette draws.
    pub(super) fn prepare_morph_body_or_empty(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let needs_body = self
            .caret_anchor_ink_box()
            .map(|ink| {
                let px = self.metrics.caret_h / CARET_H;
                let (w, h) = caret_visual_body_dims(ink, px);
                w > ink.width + f32::EPSILON
                    || h > ink.height + 2.0 * CARET_INK_PAD * px + f32::EPSILON
            })
            .unwrap_or(false);
        if needs_body {
            self.prepare_caret_block(device, queue, width, height);
            // The support body is the accent itself, so an accent-coloured Morph
            // silhouette would disappear into it. Knock the inhabited glyph back
            // through in the authored primary-content colour, matching the Filled
            // block's existing covered-glyph rule.
            self.caret_glyph_pipeline
                .set_color(theme::primary_content().rgb_bytes());
        } else {
            self.caret_pipeline.prepare_empty();
            self.caret_glyph_pipeline
                .set_color(theme::primary().rgb_bytes());
        }
    }
}
