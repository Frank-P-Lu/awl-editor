//! Rotated-quad spine primitives: pure geometry with no device or clock, consumed by
//! [`super::SelectionPipeline::prepare_rotated`].

use super::UPRIGHT_AXIS;

/// A rotated rounded-rect spine segment's `(center, half_size,
/// axis)`, for [`SelectionPipeline::prepare_rotated`]: a bar running from
/// `from` to `to`, `thickness_px` wide. Pure geometry, no clock, no device —
/// `axis` degenerates to the inert `(1.0, 0.0)` (upright) when `from == to`
/// (a zero-length segment has no direction to normalize), so a pathological
/// input can never hand the shader a NaN.
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
pub fn narrowed_spine_corner_px(corner_px: f32, half_len: f32, half_thick: f32) -> f32 {
    corner_px
        .min(half_len.max(0.0))
        .min(half_thick.max(0.0))
        .max(0.0)
}

/// THE ROTATABLE CHEVRON — the ONE owner of the mark's shape, for every surface
/// that draws a chevron out of rotated quads. Two [`spine_segment`] arms meeting
/// at a vertex DERIVED from the arm ends, so the mark's two halves cannot drift
/// out of symmetry: there is no second quantity that could disagree.
///
/// `turn_deg` is the one input that decides which way the mark reads. At `0.0`
/// the vertex sits `reach` along `+x` from `center` and both arms trail back to
/// `reach` along `-x`, `spread` apart across it — a `›`. Every other angle is
/// the same shape rigidly rotated, so the mark is directional AT REST and a law
/// can grade the ANGLE rather than the instance count (a chevron is two segments
/// at every turn, which is exactly what a counting law cannot see).
///
/// `reach` and `spread` are SIGNED: negating `reach` is the half-turn that
/// mirrors the mark across `center`, so a composition that already mirrors on a
/// world's own sign can carry the mark with it rather than branching again.
///
/// The turn pivots on `center`, the midpoint between vertex and arm line — the
/// right pivot for a mark centred in its own box. A caller that instead needs a
/// FIXED vertex (a mark anchored to a line it must not leave) passes
/// `center = vertex - reach * (cos θ, sin θ)`, which turns the same shape about
/// the vertex with no second entry point.
///
/// Pure — no device, no clock, no theme — so a law can grade the exact shape a
/// frame would draw at any turn.
pub fn chevron_arms(
    center: [f32; 2],
    reach: f32,
    spread: f32,
    turn_deg: f32,
    thickness: f32,
) -> [([f32; 2], [f32; 2], [f32; 2]); 2] {
    let (s, c) = turn_deg.to_radians().sin_cos();
    let u = [c, s];
    let p = [-s, c];
    let vertex = [center[0] + u[0] * reach, center[1] + u[1] * reach];
    let back = [center[0] - u[0] * reach, center[1] - u[1] * reach];
    let arm_a = [back[0] + p[0] * spread, back[1] + p[1] * spread];
    let arm_b = [back[0] - p[0] * spread, back[1] - p[1] * spread];
    [
        spine_segment(vertex, arm_a, thickness),
        spine_segment(vertex, arm_b, thickness),
    ]
}
