//! The rotated label's PURE geometry: no device, no clock, no font system.
//!
//! A label has its own frame. `u` runs along the baseline in the advance
//! direction; `v` runs DOWN from the baseline, so an ascender is negative `v`
//! and a descender positive. [`label_axis_deg`] maps a reader's protractor
//! angle onto the unit screen axis `u` follows, and everything else here is one
//! rotation of that frame onto (or off) the screen — the same two-vector
//! `(ax, perp)` basis `caret.wgsl` and `shaders/rotated_label.wgsl` build in
//! their vertex stages, so the CPU can answer "where is this label, and what
//! does it cover" without a GPU in the room.

use crate::selection::UPRIGHT_AXIS;

/// An ink box in the run's own frame: `[u_min, v_min, width, height]`, pixels.
pub type InkBox = [f32; 4];

/// The unit screen axis a label's baseline advances along, for a protractor
/// angle a reader would name: 0° reads left to right, 90° reads bottom to top
/// (the flush-left vertical heading), 180° upside down, 270° top to bottom.
///
/// Screen pixels are y-DOWN, so a counter-clockwise turn as the reader sees it
/// is a NEGATIVE turn in pixel coordinates — that sign is the whole subtlety
/// here, and getting it backwards mirrors a label rather than rotating it.
pub fn label_axis_deg(deg: f32) -> [f32; 2] {
    let turn = if deg.is_finite() {
        deg.rem_euclid(360.0)
    } else {
        0.0
    };
    // The four quadrant axes are returned EXACTLY instead of through the
    // transcendentals. `cos(90°)` in f32 is 4.4e-8, not 0, and only an exact
    // axis makes a quarter turn a lossless texel transpose of the run's mask
    // rather than a bilinear resample sitting fractionally off the grid. 90° is
    // the angle a rotated label is most often asked for, so it is the one that
    // must not pay for a rounding error.
    if turn == 0.0 {
        UPRIGHT_AXIS
    } else if turn == 90.0 {
        [0.0, -1.0]
    } else if turn == 180.0 {
        [-1.0, 0.0]
    } else if turn == 270.0 {
        [0.0, 1.0]
    } else {
        let (sin, cos) = (-turn.to_radians()).sin_cos();
        [cos, sin]
    }
}

/// The axis `v` (down from the baseline) follows: `axis` turned +90° in y-down
/// pixel space, so the upright axis yields screen-down and the frame stays
/// right-handed as the label turns.
pub fn label_perp(axis: [f32; 2]) -> [f32; 2] {
    let a = unit_axis(axis);
    [-a[1], a[0]]
}

/// A point in the run's own frame, placed on the screen.
pub fn label_point(origin: [f32; 2], axis: [f32; 2], local: [f32; 2]) -> [f32; 2] {
    let a = unit_axis(axis);
    let p = label_perp(a);
    [
        origin[0] + local[0] * a[0] + local[1] * p[0],
        origin[1] + local[0] * a[1] + local[1] * p[1],
    ]
}

/// A screen point mapped back INTO the run's own frame — the exact inverse of
/// [`label_point`]. The basis is orthonormal, so the inverse is a pair of dot
/// products and never a matrix solve.
pub fn label_local(origin: [f32; 2], axis: [f32; 2], screen: [f32; 2]) -> [f32; 2] {
    let a = unit_axis(axis);
    let p = label_perp(a);
    let d = [screen[0] - origin[0], screen[1] - origin[1]];
    [d[0] * a[0] + d[1] * a[1], d[0] * p[0] + d[1] * p[1]]
}

/// The label's four screen-space corners, in `ink`'s own corner order:
/// `(u_min, v_min)`, `(u_max, v_min)`, `(u_max, v_max)`, `(u_min, v_max)` —
/// exactly the quad `shaders/rotated_label.wgsl` rasterises.
pub fn label_quad(origin: [f32; 2], axis: [f32; 2], ink: InkBox) -> [[f32; 2]; 4] {
    let (u0, v0) = (ink[0], ink[1]);
    let (u1, v1) = (ink[0] + ink[2], ink[1] + ink[3]);
    [
        label_point(origin, axis, [u0, v0]),
        label_point(origin, axis, [u1, v0]),
        label_point(origin, axis, [u1, v1]),
        label_point(origin, axis, [u0, v1]),
    ]
}

/// The axis-aligned screen bounds `[x, y, w, h]` the label occupies — what a
/// consumer needs to prove a cue does not overlap the rows beside it, and to
/// reserve the gutter it stands in.
pub fn label_bounds(origin: [f32; 2], axis: [f32; 2], ink: InkBox) -> [f32; 4] {
    let quad = label_quad(origin, axis, ink);
    let mut min = quad[0];
    let mut max = quad[0];
    for c in &quad[1..] {
        min = [min[0].min(c[0]), min[1].min(c[1])];
        max = [max[0].max(c[0]), max[1].max(c[1])];
    }
    [min[0], min[1], max[0] - min[0], max[1] - min[1]]
}

/// Whether a screen point lands on the label's own rotated box — the hit test
/// a consumer would need IF a rotated run were ever made interactive.
///
/// No rotated run is interactive today: the two expressions this capability
/// unblocks are a secondary heading and a location cue, and both are read, not
/// pressed. This exists so the answer is DERIVED from the same frame the shader
/// draws through rather than approximated by the axis-aligned
/// [`label_bounds`] — which, at 90°, over-claims the corners of a diagonal run
/// by a wide margin and would steal presses from the rows beside it.
pub fn label_hit(origin: [f32; 2], axis: [f32; 2], ink: InkBox, screen: [f32; 2]) -> bool {
    let l = label_local(origin, axis, screen);
    l[0] >= ink[0] && l[0] <= ink[0] + ink[2] && l[1] >= ink[1] && l[1] <= ink[1] + ink[3]
}

/// Normalise, degenerating to the inert upright axis on a zero/non-finite
/// input so a pathological caller can never put a NaN in front of the shader —
/// the same guard [`crate::selection::spine_segment`] applies to a zero-length
/// segment.
///
/// The ONE owner: [`super::RotatedLabelPipeline::prepare`] normalises through
/// this same function before uploading, so the geometry a consumer measures
/// with and the quad the GPU rasterises cannot disagree about the direction.
pub fn unit_axis(axis: [f32; 2]) -> [f32; 2] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt();
    if len.is_finite() && len > 1e-6 {
        [axis[0] / len, axis[1] / len]
    } else {
        UPRIGHT_AXIS
    }
}
