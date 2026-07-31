//! Rotated-quad spine primitives: pure geometry with no device or clock, consumed by
//! [`super::SelectionPipeline::prepare_rotated`].

use super::UPRIGHT_AXIS;

/// A rotated rounded-rect spine segment's `(center, half_size,
/// axis)`, for [`SelectionPipeline::prepare_rotated`]: a bar running from
/// `from` to `to`, `thickness_px` wide. Pure geometry, no clock, no device —
/// `axis` degenerates to the inert `(1.0, 0.0)` (upright) when `from == to`
/// (a zero-length segment has no direction to normalize), so a pathological
/// input can never hand the shader a NaN.
///
/// No non-test caller yet — see [`SelectionPipeline::prepare_rotated`]'s doc.
#[allow(dead_code)]
pub fn spine_segment(
    from: [f32; 2],
    to: [f32; 2],
    thickness_px: f32,
) -> ([f32; 2], [f32; 2], [f32; 2]) {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let center = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5];
    let half = [len * 0.5, (thickness_px * 0.5).max(0.0)];
    let axis = if len > 1e-4 {
        [d[0] / len, d[1] / len]
    } else {
        UPRIGHT_AXIS
    };
    (center, half, axis)
}

/// Cap a spine segment's own corner radius to at most its
/// SHORTER half-extent (half its length, half its thickness), mirroring
/// `render::chrome::narrowed_chamfer_px`'s "clamp a decorative cut to the
/// shape's own geometry" shape. The shader's per-fragment SDF already clamps
/// `min(g.corner, min(hsize.x, hsize.y))` (`selection.wgsl`'s `fs_main`), so
/// this CPU-side twin is belt-and-suspenders: it lets a caller reason about
/// the drawn shape (and pick one shared `SelectionPipeline::set_corner` value
/// across a whole spine of differently-sized segments) before anything
/// reaches the GPU, rather than discovering the clamp only in the rendered
/// pixels.
///
/// No non-test caller yet — see [`SelectionPipeline::prepare_rotated`]'s doc.
#[allow(dead_code)]
pub fn narrowed_spine_corner_px(corner_px: f32, half_len: f32, half_thick: f32) -> f32 {
    corner_px
        .min(half_len.max(0.0))
        .min(half_thick.max(0.0))
        .max(0.0)
}
