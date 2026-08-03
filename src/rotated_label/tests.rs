//! THE ROTATED-LABEL FRAME LAWS — the PURE tier.
//!
//! The capability has two ways to be wrong, and they are graded in two places.
//! This file grades the run's frame — the axis, the quad, the bounds, the
//! hit test — with no GPU in the room. Its oracles are deliberately different
//! computations from the code they grade: the quad is checked against a
//! measured side length and a dot product, the hit test against a polygon
//! winding sweep over the quad's own corners, and the bounds against a dense
//! sample of the quad's interior. A law that re-ran `label_local` to check
//! `label_local` would prove nothing.
//!
//! The PIXELS — coverage, ink mass and legibility at each angle, at 1× and 2×
//! DPI — are graded in `render::tests`, where the shared headless device and
//! the real font system live.

use super::geometry::*;

/// Magpie's slant. Its diagonal row spine advances `ROW_STEP_LOGICAL` (7 px)
/// sideways for every line of row height it descends; at the 32 px row height
/// `render::tests` builds its canvas around, that is a run 12.3° off vertical,
/// i.e. 77.7° off horizontal. Signed both ways because the world roster carries
/// both a Descending and an Ascending diagonal.
const MAGPIE_SLANT_DEG: f32 = 77.66;

/// Every angle the capability is graded at: upright, both quarter turns, the
/// half turn a mirrored cue takes, and Magpie's slant in both directions.
const GRADED_ANGLES: [f32; 7] = [
    0.0,
    90.0,
    270.0,
    180.0,
    MAGPIE_SLANT_DEG,
    -MAGPIE_SLANT_DEG,
    180.0 - MAGPIE_SLANT_DEG,
];

// ───────────────────────── the pure frame ─────────────────────────

/// The inert case is EXACT, not approximately exact. `label_axis_deg(0)` must
/// be the identical upright axis the selection and caret pipelines already
/// carry — one owner for "not rotated" — and the three other quadrant axes must
/// be exact too, because only an exact quarter turn maps the mask's texel grid
/// onto the pixel grid without a resample.
#[test]
fn the_quadrant_axes_are_exact_and_the_upright_one_is_the_shared_inert_axis() {
    assert_eq!(label_axis_deg(0.0), crate::selection::UPRIGHT_AXIS);
    assert_eq!(label_axis_deg(90.0), [0.0, -1.0]);
    assert_eq!(label_axis_deg(180.0), [-1.0, 0.0]);
    assert_eq!(label_axis_deg(270.0), [0.0, 1.0]);
    // Wrapping is exact too: a caller that accumulates a turn must not fall off
    // the exact path at 360° or below zero.
    assert_eq!(label_axis_deg(360.0), crate::selection::UPRIGHT_AXIS);
    assert_eq!(label_axis_deg(-90.0), [0.0, 1.0]);
    assert_eq!(label_axis_deg(-270.0), [0.0, -1.0]);
}

/// SWEEP THE WHOLE CIRCLE, not the angles anyone pictured: at every whole
/// degree the axis is a unit vector, its perpendicular is a unit vector, and
/// the two are orthogonal — the basis the shader rebuilds per vertex.
#[test]
fn every_degree_of_turn_yields_an_orthonormal_run_frame() {
    for deg in -720..=720 {
        let a = label_axis_deg(deg as f32);
        let p = label_perp(a);
        let len_a = (a[0] * a[0] + a[1] * a[1]).sqrt();
        let len_p = (p[0] * p[0] + p[1] * p[1]).sqrt();
        let dot = a[0] * p[0] + a[1] * p[1];
        assert!(
            (len_a - 1.0).abs() < 1e-5 && (len_p - 1.0).abs() < 1e-5 && dot.abs() < 1e-5,
            "{deg}°: axis {a:?} len {len_a}, perp {p:?} len {len_p}, dot {dot}"
        );
    }
    // A degenerate axis can never reach the shader as a NaN.
    assert_eq!(unit_axis([0.0, 0.0]), crate::selection::UPRIGHT_AXIS);
    assert_eq!(unit_axis([f32::NAN, 1.0]), crate::selection::UPRIGHT_AXIS);
    assert_eq!(label_axis_deg(f32::NAN), crate::selection::UPRIGHT_AXIS);
}

/// The sign the whole capability turns on: 90° must read BOTTOM TO TOP (the
/// flush-left vertical heading), not top to bottom, and a descender must fall
/// to the RIGHT of the baseline. Get this backwards and every label is
/// mirrored — which still passes a round-trip law, because a round trip
/// cancels its own error.
#[test]
fn a_quarter_turn_reads_bottom_to_top_with_descenders_to_the_right() {
    let origin = [100.0, 100.0];
    let axis = label_axis_deg(90.0);
    // Advancing 10px along the baseline goes UP the screen.
    let advanced = label_point(origin, axis, [10.0, 0.0]);
    assert_eq!(advanced, [100.0, 90.0], "the run must advance upward");
    // 10px BELOW the baseline in the run's own frame goes RIGHT on screen.
    let below = label_point(origin, axis, [0.0, 10.0]);
    assert_eq!(below, [110.0, 100.0], "descenders must fall to the right");
    // And the upright case is the untouched identity.
    let up = label_axis_deg(0.0);
    assert_eq!(label_point(origin, up, [10.0, 4.0]), [110.0, 104.0]);
}

/// The quad is a RIGID motion of the ink box: at every graded angle its four
/// sides measure exactly the box's own sides and its corners are square.
/// Measured with lengths and dot products, never by re-running the placement.
#[test]
fn the_quad_is_a_rigid_motion_of_the_ink_box_at_every_graded_angle() {
    let ink: InkBox = [-3.0, -12.0, 47.0, 19.0];
    for deg in GRADED_ANGLES {
        let q = label_quad([200.0, 150.0], label_axis_deg(deg), ink);
        let side = |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
        assert!((side(q[0], q[1]) - ink[2]).abs() < 1e-3, "{deg}°: top side");
        assert!(
            (side(q[3], q[2]) - ink[2]).abs() < 1e-3,
            "{deg}°: bottom side"
        );
        assert!(
            (side(q[0], q[3]) - ink[3]).abs() < 1e-3,
            "{deg}°: left side"
        );
        assert!(
            (side(q[1], q[2]) - ink[3]).abs() < 1e-3,
            "{deg}°: right side"
        );
        let e1 = [q[1][0] - q[0][0], q[1][1] - q[0][1]];
        let e2 = [q[3][0] - q[0][0], q[3][1] - q[0][1]];
        assert!(
            (e1[0] * e2[0] + e1[1] * e2[1]).abs() < 1e-2,
            "{deg}°: corner is not square"
        );
    }
}

/// The AABB a consumer reserves space with really is the quad's own extent:
/// every corner sits inside it, and the box is TIGHT — each of its four edges
/// is touched. A bounds that merely contained the quad would let a cue reserve
/// a gutter twice as wide as it needs.
#[test]
fn the_bounds_are_the_tight_axis_aligned_extent_of_the_quad() {
    let ink: InkBox = [-3.0, -12.0, 47.0, 19.0];
    for deg in GRADED_ANGLES {
        let axis = label_axis_deg(deg);
        let q = label_quad([200.0, 150.0], axis, ink);
        let b = label_bounds([200.0, 150.0], axis, ink);
        for c in q {
            assert!(
                c[0] >= b[0] - 1e-3
                    && c[0] <= b[0] + b[2] + 1e-3
                    && c[1] >= b[1] - 1e-3
                    && c[1] <= b[1] + b[3] + 1e-3,
                "{deg}°: corner {c:?} outside bounds {b:?}"
            );
        }
        let touches = |v: f32, target: f32| (v - target).abs() < 1e-3;
        assert!(q.iter().any(|c| touches(c[0], b[0])), "{deg}°: left slack");
        assert!(q.iter().any(|c| touches(c[1], b[1])), "{deg}°: top slack");
        assert!(
            q.iter().any(|c| touches(c[0], b[0] + b[2])),
            "{deg}°: right slack"
        );
        assert!(
            q.iter().any(|c| touches(c[1], b[1] + b[3])),
            "{deg}°: bottom slack"
        );
    }
    // A quarter turn TRANSPOSES the box — the property a flush-left vertical
    // heading's gutter is sized by.
    let b0 = label_bounds([200.0, 150.0], label_axis_deg(0.0), ink);
    let b90 = label_bounds([200.0, 150.0], label_axis_deg(90.0), ink);
    assert!((b0[2] - b90[3]).abs() < 1e-3 && (b0[3] - b90[2]).abs() < 1e-3);
}

/// HIT AGREEMENT. No rotated run is interactive today — both expressions this
/// unblocks are read, not pressed — but a consumer that ever makes one
/// interactive, or that must prove a cue steals no press from the rows beside
/// it, needs `label_hit` to agree with the shape actually drawn.
///
/// The oracle is a POLYGON WINDING sweep over `label_quad`'s own corners: a
/// point is inside a convex quad when it is on the same side of all four
/// edges. That is a different computation from `label_hit`'s inverse rotation,
/// so the two cannot be wrong together — and the sweep runs over a dense grid
/// that covers the rotated box AND the axis-aligned bounds around it, which is
/// where a hit test that lazily used the bounds would be caught.
#[test]
fn the_hit_test_agrees_with_the_drawn_quad_over_a_dense_grid() {
    let ink: InkBox = [-3.0, -12.0, 47.0, 19.0];
    let origin = [200.0, 150.0];
    for deg in GRADED_ANGLES {
        let axis = label_axis_deg(deg);
        let quad = label_quad(origin, axis, ink);
        let b = label_bounds(origin, axis, ink);
        let mut inside = 0usize;
        let mut outside = 0usize;
        let mut checked = 0usize;
        // Sweep a margin BEYOND the bounds so points outside every candidate
        // shape are graded too.
        let (x0, y0) = (b[0] - 6.0, b[1] - 6.0);
        let (x1, y1) = (b[0] + b[2] + 6.0, b[1] + b[3] + 6.0);
        let mut y = y0;
        while y <= y1 {
            let mut x = x0;
            while x <= x1 {
                let p = [x, y];
                let want = winding_inside(&quad, p);
                let got = label_hit(origin, axis, ink, p);
                // Points within half a pixel of an edge are genuinely
                // ambiguous under two different formulations; skip only those.
                if edge_distance(&quad, p) > 0.5 {
                    checked += 1;
                    assert_eq!(got, want, "{deg}°: hit disagreement at {p:?}");
                    if want { inside += 1 } else { outside += 1 }
                }
                x += 0.5;
            }
            y += 0.5;
        }
        // NON-VACUOUS: the sweep must actually have straddled the boundary.
        assert!(
            inside > 500 && outside > 500,
            "{deg}°: sweep saw {inside} inside / {outside} outside of {checked}"
        );
    }
    // The rotated hit box is NOT the axis-aligned one: at 45° the bounds'
    // corner is far outside the run. A hit test that took the cheap path would
    // pass every assertion above only if this were false.
    let axis = label_axis_deg(45.0);
    let b = label_bounds(origin, axis, ink);
    assert!(
        !label_hit(origin, axis, ink, [b[0] + 0.5, b[1] + 0.5]),
        "the bounds' own corner must not be a hit at 45°"
    );
}

/// Whether `p` is inside the convex quad `q` — the INDEPENDENT oracle for
/// [`label_hit`]. Same-side-of-every-edge, by cross product; it never rotates
/// anything.
fn winding_inside(q: &[[f32; 2]; 4], p: [f32; 2]) -> bool {
    let mut sign = 0i32;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let s = if cross > 0.0 {
            1
        } else if cross < 0.0 {
            -1
        } else {
            0
        };
        if s != 0 {
            if sign == 0 {
                sign = s;
            } else if sign != s {
                return false;
            }
        }
    }
    true
}

/// Distance from `p` to the nearest of the quad's four edges, for skipping the
/// genuinely ambiguous band either formulation may round either way.
fn edge_distance(q: &[[f32; 2]; 4], p: [f32; 2]) -> f32 {
    let mut best = f32::INFINITY;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let d = [b[0] - a[0], b[1] - a[1]];
        let len2 = d[0] * d[0] + d[1] * d[1];
        let t = if len2 > 0.0 {
            (((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let c = [a[0] + t * d[0], a[1] + t * d[1]];
        best = best.min((p[0] - c[0]).hypot(p[1] - c[1]));
    }
    best
}
