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
/// brackets need width, dashes need height, and commas need all three. Width
/// and height are `Logical` — the same pixel space chrome's own pads live in —
/// resolved through the scale the caller already recovered from the metrics
/// (`px = m.caret_h / CARET_H`, exactly [`super::Metrics::scale`] because
/// [`super::Metrics::with_dpi`] built `caret_h` that way, the same recovery
/// `CARET_INK_PAD` already uses). Area is a SEPARATE family, not `Logical`:
/// doubling the display factor quadruples an area rather than doubling it, so
/// resolving it through a length's one-multiply door would under-scale it by
/// exactly one factor of `scale` — invisible at `--capture-dpi 1`, the same
/// failure shape `Logical` exists to close for lengths.
pub(super) const CARET_VISUAL_BODY_MIN_W: Logical = Logical(6.5);
pub(super) const CARET_VISUAL_BODY_MIN_H: Logical = Logical(12.0);
pub(super) const CARET_VISUAL_BODY_MIN_AREA: Area = Area(96.0);

/// A quantity in SQUARE logical pixels — an AREA floor, not a length. The
/// newtype carries no `.px()`, only [`Self::px2`], so an area constant cannot
/// reach a comparison through the same one-multiply door a length uses and
/// come out scaled by only one factor of `scale`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(super) struct Area(pub f32);

impl Area {
    /// The one multiply, applied twice: `scale * scale`, the factor an AREA
    /// wants when every linear quantity beside it scales by `scale` alone.
    pub(super) fn px2(self, scale: f32) -> f32 {
        self.0 * scale * scale
    }
}

/// Apply the authored floor without flattening ordinary glyph-responsive carets.
pub(super) fn caret_visual_body_dims(ink: InkBox, px: f32) -> (f32, f32) {
    let mut w = ink.width.max(CARET_VISUAL_BODY_MIN_W.px(px));
    let mut h = (ink.height + 2.0 * CARET_INK_PAD.px(px)).max(CARET_VISUAL_BODY_MIN_H.px(px));
    let min_area = CARET_VISUAL_BODY_MIN_AREA.px2(px);
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
        let ink = self.caret_anchor_ink_box();
        let needs_body = ink.is_some_and(|ink| {
            let px = self.metrics.caret_h / CARET_H;
            let (w, _) = caret_visual_body_dims(ink, px);
            let (baseline, ascent, font) = self.caret_row_metrics();
            let (_, h) = self.caret_cell_vertical_from_ink(ink, baseline, ascent, font, px);
            w > ink.width + f32::EPSILON
                || h > ink.height + 2.0 * CARET_INK_PAD.px(px) + f32::EPSILON
        });
        if needs_body {
            self.prepare_caret_block(device, queue, width, height);
            // The support body is the accent (`primary`), drawn behind the glyph,
            // on an ordinary Normal-caret-style world (a Filled/InverseVideo world
            // folds Morph to Block before this function is ever reached — see
            // `folds_morph_to_block`), where `primary` is a genuine accent distinct
            // from the page's own text ink. The glyph therefore needs NO contrast
            // correction at all: ordinary prose ink (`base_content`) already reads
            // over the accent body exactly as it reads over any other surface —
            // that is the whole "one accent, ink by value" rule (DESIGN.md).
            // `primary_content` must NOT be read here: it is the colour authored to
            // sit ON TOP of a FILLED ink-caret block, where `primary ==
            // base_content` and a second copy of the same ink would vanish into it
            // (see `prepare_caret_block`'s `CaretBlockStyle::Filled` arm, the only
            // other caller of `primary_content` on this path) — reading it on an
            // ordinary world recolours every landed mark in a colour with no
            // relation to the page's own ink or the world's accent, and on a world
            // where `primary_content` happens to sit close to the page ground, the
            // glyph reads as nearly swallowed.
            self.caret_glyph_pipeline
                .set_color(theme::base_content().rgb_bytes());
        } else {
            self.caret_pipeline.prepare_empty();
            self.caret_glyph_pipeline
                .set_color(theme::primary().rgb_bytes());
        }
    }
}
