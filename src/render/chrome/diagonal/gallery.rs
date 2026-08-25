//! CAPTURE-ONLY composition candidates for the Diagonal picker's frost
//! placement, query seat and chevron reach.
//!
//! Every knob here reads its own `AWL_DIAGONAL_GALLERY_*` env var, resolves to
//! an inert `None`/`false` on every ordinary run (nothing reads these vars
//! outside this file), and is consumed by the SAME draw path the shipped
//! composition already runs — never a second renderer. An unset env var
//! reproduces today's frame bit for bit; the point of these knobs is to let a
//! capture script audition an alternative without the alternative ever
//! becoming what a live frame draws by default. No shipped default reads
//! these functions' answers as anything but `None`/`false`.

use crate::render::chrome::OverlayGeom;

/// Which alternative frost extent this frame's capture asked for, or `None`
/// on every ordinary run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::render) enum FrostCandidate {
    /// Frost the whole canvas instead of the card's own footprint — the
    /// item's "full-canvas frost when the footprint would leave a large
    /// legible fraction sharp anyway" direction.
    Full,
    /// Seat the footprint's top face above the first document line instead
    /// of at the card's own top edge.
    TopAboveFirstLine,
}

pub(in crate::render) fn frost_candidate() -> Option<FrostCandidate> {
    match std::env::var("AWL_DIAGONAL_GALLERY_FROST").ok().as_deref() {
        Some("full") => Some(FrostCandidate::Full),
        Some("top0") => Some(FrostCandidate::TopAboveFirstLine),
        _ => None,
    }
}

/// Extend a footprint rect's top face to the canvas top (`y = 0`), which is
/// always above any document line drawn beneath a card seated near the top of
/// the window — the item's "top face seated above the first document line"
/// direction.
///
/// The naive edit (`rect[1] = 0.0; rect[3] = old_bottom`) SILENTLY MOVES THE
/// RAKING SIDE FACES: `blur::extent::footprint_dist_outside` un-shears every
/// point about the box's own vertical centre (`cy = y + h/2`), so changing
/// `y`/`h` moves `cy` and therefore slides the sheared side faces sideways at
/// every row — a candidate meant to isolate the TOP face would incidentally
/// regress the side faces, muddying the comparison the gallery exists to
/// make. Compensating `x` by `shear * (new_cy - old_cy)` keeps every side
/// face at the exact canvas x it already had; only the top boundary moves.
pub(in crate::render) fn seat_top_above_first_line(rect: [f32; 4], shear: f32) -> [f32; 4] {
    let [x, y, w, h] = rect;
    let bottom = y + h;
    let new_y = y.min(0.0);
    let new_h = (bottom - new_y).max(0.0);
    let old_cy = y + h * 0.5;
    let new_cy = new_y + new_h * 0.5;
    let new_x = x + shear * (new_cy - old_cy);
    [new_x, new_y, w, new_h]
}

/// Shorten the selected row's chevron reach so its vertex seats just past the
/// row's own measured NAME ink instead of the far edge of the row's whole
/// reserved cluster width — the item's "chevron reach shortened" direction.
pub(in crate::render) fn short_chevron_reach() -> bool {
    std::env::var("AWL_DIAGONAL_GALLERY_CHEVRON").as_deref() == Ok("short")
}

/// Right-align the query header band against the card's own text column
/// instead of seating it at the card's left text edge — the item's "query
/// seated nearer the list's anchor side" direction.
///
/// `offband.rs`'s own design note is the tension this candidate photographs
/// rather than resolves: a query field is an input, and right-aligning one on
/// a mirrored composition makes its sigil travel as the user types. A static
/// capture cannot show that travel; the README says so rather than letting
/// the screenshot imply the tradeoff was free.
pub(in crate::render) fn query_right_aligned() -> bool {
    std::env::var("AWL_DIAGONAL_GALLERY_QUERY").as_deref() == Ok("right")
}

/// Where the gallery's right-aligned query candidate would seat a head band
/// `ink_w` wide, or `None` when the candidate is not active. The default seat
/// (`geom.text_left`) is untouched by this module; every caller falls back to
/// it when this returns `None`.
pub(in crate::render) fn head_left_override(geom: &OverlayGeom, ink_w: f32) -> Option<f32> {
    if !query_right_aligned() {
        return None;
    }
    Some((geom.text_left + geom.text_w - ink_w).max(geom.text_left))
}
