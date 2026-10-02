//! Shared padded rectangular caret dimensions.
//!
//! The geometry owner supplies real raster ink. This owner turns it into the
//! visible Block body, with no punctuation or world identity branch.

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
/// resolved through `px`, which every caller supplies as the STORED
/// [`super::Metrics::scale`] — the one `zoom * dpi` factor `CARET_INK_PAD` meets
/// too. Deriving that factor back out of an already-scaled length is a second
/// source for the same number and not a bit-equal one; `caret_scale_law` measures
/// the disagreement and refuses the derivation. Area is a SEPARATE family, not
/// `Logical`: doubling the display factor quadruples an area, not doubles it, so
/// resolving it through a length's one-multiply door would under-scale it by
/// exactly one factor of `scale` — invisible at `--capture-dpi 1`, the same
/// failure shape `Logical` exists to close for lengths.
pub(super) const CARET_VISUAL_BODY_MIN_W: Logical = Logical(6.5);
pub(super) const CARET_VISUAL_BODY_MIN_H: Logical = Logical(12.0);
pub(super) const CARET_VISUAL_BODY_MIN_AREA: Area = Area(96.0);
/// Side and bottom clearance around the complete shaped grapheme ink.
pub(super) const CARET_BLOCK_INK_PAD: Logical = Logical(2.5);
/// Tighter top clearance; shared by every glyph, document, and picker.
pub(super) const CARET_BLOCK_TOP_PAD: Logical = Logical(1.25);
/// Soft rounding with a full raster-ink safety margin inside each curved corner.
pub(super) const CARET_BLOCK_CORNER_RADIUS: Logical = Logical(4.5);

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
/// `w` carries [`CARET_INK_PAD_W`] the same way `h` already carries
/// [`CARET_INK_PAD`] — a uniform margin around the anchored glyph's own ink,
/// never a per-glyph raster read, so a narrow letter and a wide one both grow
/// by the identical two pads before either floor below ever runs.
pub(super) fn caret_visual_body_dims(ink: InkBox, px: f32) -> (f32, f32) {
    caret_visual_body_dims_with_pad(ink, px, 2.0 * CARET_INK_PAD.px(px), CARET_INK_PAD_W.px(px))
}

/// A shared padded ink rectangle for the document and picker sample.
pub(super) fn caret_block_body_dims(ink: InkBox, px: f32) -> (f32, f32) {
    let vertical = CARET_BLOCK_TOP_PAD.px(px) + CARET_BLOCK_INK_PAD.px(px);
    caret_visual_body_dims_with_pad(ink, px, vertical, CARET_BLOCK_INK_PAD.px(px))
}

/// Center below the ink midpoint so only the top edge loses clearance.
pub(super) fn caret_block_center_offset(ink: InkBox, px: f32) -> f32 {
    ink.height * 0.5 + (CARET_BLOCK_INK_PAD.px(px) - CARET_BLOCK_TOP_PAD.px(px)) * 0.5
}

/// Clamp the authored rounding to the same resting rectangle in every consumer.
pub(super) fn caret_block_corner(w: f32, h: f32, px: f32) -> f32 {
    CARET_BLOCK_CORNER_RADIUS.px(px).min(w * 0.5).min(h * 0.5)
}

fn caret_visual_body_dims_with_pad(
    ink: InkBox,
    px: f32,
    vertical_pad: f32,
    horizontal_pad: f32,
) -> (f32, f32) {
    let mut w = (ink.width + 2.0 * horizontal_pad).max(CARET_VISUAL_BODY_MIN_W.px(px));
    let mut h = (ink.height + vertical_pad).max(CARET_VISUAL_BODY_MIN_H.px(px));
    let min_area = CARET_VISUAL_BODY_MIN_AREA.px2(px);
    if w * h < min_area {
        let grow = (min_area / (w * h)).sqrt();
        w *= grow;
        h *= grow;
    }
    (w, h)
}
