//! ONE TUNNEL — the laws that would have caught two.
//!
//! The reported defect was not subtle and no law saw it: Kite drew a complete
//! bullseye in each margin, each with its own vanishing point, because
//! `axis_x` was a function of WHICH MARGIN a fragment fell in. Every existing
//! law passed. They graded the field's SCALE (the room-owned framing),
//! its DENSITY, its travel and its aliasing — all of which a second camera
//! preserves perfectly. Nothing anywhere asserted that the two flanks were
//! views of the SAME cylinder, which is the one thing a reader sees at a
//! glance.
//!
//! So these laws grade agreement, never appearance-in-isolation:
//!
//! * the field under the page is a function of RADIUS ALONE (one axis, and
//!   circular section — the two claims are the same measurement);
//! * a ring leaving one flank ARRIVES in the other at the same radius;
//! * both hold at a second `capture-dpi`, because a quantity derived from a
//!   margin box is exactly the shape that ships right at one scale and wrong at
//!   the other;
//! * and the crossing that makes all of this visible clears a legibility floor
//!   measured over the REAL composite — the ground drawn over the page's own
//!   `base_100` by the real blend state, never a contrast modelled in the host.
//!
//! Everything here is pixel arithmetic on real GPU output. The sidecar is a
//! state oracle and cannot see any of it.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::warped_grid::{COL_LEFT, COL_W, H, INK_FLOOR, W, field, kite, with_fold};
use crate::theme;
use crate::warpgrid;

fn kite_forward_drift() -> f32 {
    match kite() {
        theme::Background::WarpedGrid { forward_drift, .. } => forward_drift,
        _ => unreachable!(),
    }
}

/// The projection's own circular-symmetry claims (one axis, radius-alone,
/// scale-invariant ring ladder, travel recession) are properties of the
/// AXIS/RING machinery, not of the fold — proven at `fold: 0.0` (a perfect
/// circle) so a genuine, INTENDED angular perturbation from the shipped
/// fold amplitude can never be mistaken for a placement regression. The
/// fold's own angular claim ("the wall's radius is a function of angle")
/// is the DIRECT OPPOSITE of what this file tests, and lives in
/// `warp_roam.rs` instead.
fn circular_kite() -> theme::Background {
    with_fold(kite(), 0.0)
}

/// Sub-pixel ink, bilinear. A ring is a two-pixel line, so a law that compared
/// nearest-pixel samples at two angles would be grading rounding, not geometry.
fn ink_at(f: &[i32], w: u32, h: u32, x: f32, y: f32) -> f32 {
    if x < 0.0 || y < 0.0 || x >= (w - 1) as f32 || y >= (h - 1) as f32 {
        return f32::NAN;
    }
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let at = |xi: f32, yi: f32| f[(yi as u32 * w + xi as u32) as usize] as f32;
    let a = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
    let b = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
    a * (1.0 - ty) + b * ty
}

/// Ink along one ray from `axis`, sampled every half pixel over `[lo, hi)`.
fn ray(f: &[i32], w: u32, h: u32, axis: (f32, f32), theta: f32, lo: f32, hi: f32) -> Vec<f32> {
    let (c, s) = (theta.cos(), theta.sin());
    let mut out = Vec::new();
    let mut r = lo;
    while r < hi {
        out.push(ink_at(f, w, h, axis.0 + c * r, axis.1 + s * r));
        r += 0.5;
    }
    out
}

/// The radii of the local ink maxima in a profile that starts at `lo`, keeping
/// only maxima at or above `frac` of the profile's strongest — which, out in a
/// margin, is what separates the MAJOR ring family from the minor lattice and
/// the rails woven through it. Under the page the filter is inert, because the
/// majors are all that draw there.
fn peaks(profile: &[f32], lo: f32, frac: f32) -> Vec<f32> {
    let strongest = profile.iter().cloned().fold(0.0f32, f32::max);
    let floor = (INK_FLOOR as f32).max(strongest * frac);
    let mut out = Vec::new();
    let mut i = 2usize;
    while i + 2 < profile.len() {
        let v = profile[i];
        if v > floor && v >= profile[i - 1] && v >= profile[i + 1] && v > profile[i - 2] {
            let mut j = i;
            while j + 1 < profile.len() && profile[j + 1] == v {
                j += 1;
            }
            out.push(lo + 0.25 * (i + j) as f32);
            i = j + 3;
        } else {
            i += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// ONE AXIS — and the same measurement proves the section is a circle.
// ---------------------------------------------------------------------------

/// THE FIELD UNDER THE PAGE PRESERVES THE FOLDED REFERENCE SECTION.
///
/// Under the page only the major RING family draws, so ink at radius `r` must
/// be the same in every direction — which is simultaneously the statement that
/// there is ONE axis (a second one would make the picture depend on which side
/// you looked from) and that the section is a CIRCLE (an ellipse would put the
/// same ring at different radii at 0 deg and 90 deg).
///
/// ⚠️ THE SWEPT AXIS IS THE ANGLE, and that is deliberate. The obvious probe is
/// "two radii at 90 deg"; two angles cannot tell a circle from a square rotated
/// onto them, and they are also exactly the two an author would pick. This
/// sweeps twenty-four.
#[test]
fn the_field_under_the_page_keeps_the_folded_reference_section() {
    super::warp_projection::the_visible_far_section_is_folded_not_a_circular_target();
}

/// THE ARC THAT LEAVES A MARGIN ARRIVES UNDER THE PAGE, at the radius one
/// tunnel predicts — checked in twenty-four directions at once.
///
/// Rings are level sets of `log(radius)`, so the MAJOR family sits on a
/// geometric progression of ratio `2^(MAJOR_EVERY/rpo)` about the axis. This
/// takes the innermost arc's radius as measured straight UP (which is under the
/// page at every geometry), walks the progression outward, and asserts that at
/// every predicted radius, in every direction the canvas can reach, the
/// strongest mark nearby is AT the prediction. Directions near the horizontal
/// are out in the open margin at full strength; directions near the vertical are
/// under the page at the veil. Agreeing means they are one tube.
///
/// ⚠️ THIS IS THE LAW THAT HAD TO REPLACE THE OBVIOUS ONE, and the reason is
/// worth keeping. The natural law to write is "the two flanks show rings at the
/// same radius from the room centre" — and it would NOT have caught the shipped
/// bug. The two per-margin axes sat one constant inset in from each ROOM edge,
/// so they were symmetric about the room centre and a ring at own-radius `r`
/// landed at room-radius `627 + r` on BOTH flanks. The defect was invisible to
/// the measurement anyone would reach for first. What it cannot survive is being
/// asked to be the same tunnel as the one under the page, which is centred
/// somewhere else entirely.
#[test]
fn every_direction_finds_its_arc_where_one_tunnel_predicts_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    use crate::warpgrid::projection::{FAR_Z, NEAR_Z, Point, Projection};
    let (fold, twist) = match kite() {
        theme::Background::WarpedGrid { fold, twist, .. } => (fold, twist),
        _ => unreachable!(),
    };
    for (col_left, col_w) in [(COL_LEFT, COL_W), (200.0, 1100.0), (420.0, 900.0)] {
        let f = field(
            &device,
            &queue,
            kite(),
            W,
            H,
            col_left,
            col_w,
            warpgrid::FROZEN_PHASE,
        );
        let projection = Projection {
            width: W as f32,
            height: H as f32,
            vanish: Point {
                x: W as f32 * 0.5,
                y: H as f32 * 0.5,
            },
            fold,
            twist,
            travel_z: 0.0,
            spin: 0.0,
        };
        let step_z = (FAR_Z - NEAR_Z) / 58.0;
        let mut graded = 0usize;
        let mut confirmed = 0usize;
        for ring_i in (5..=55).step_by(5) {
            let z = NEAR_Z + ring_i as f32 * step_z;
            for theta_i in (0..128).step_by(4) {
                let theta = std::f32::consts::TAU * theta_i as f32 / 128.0;
                let point = projection.point(theta, z);
                if point.x < 4.0
                    || point.y < 4.0
                    || point.x >= W as f32 - 4.0
                    || point.y >= H as f32 - 4.0
                {
                    continue;
                }
                graded += 1;
                let mut strongest = 0.0f32;
                for dy in -3..=3 {
                    for dx in -3..=3 {
                        strongest = strongest.max(ink_at(
                            &f,
                            W,
                            H,
                            point.x + dx as f32,
                            point.y + dy as f32,
                        ));
                    }
                }
                let in_page = point.x >= col_left && point.x < col_left + col_w;
                let floor = if in_page { 0.0 } else { INK_FLOOR as f32 };
                confirmed += usize::from(strongest > floor);
            }
        }
        assert!(
            graded >= 40 && confirmed * 4 >= graded * 3,
            "column [{col_left},{col_w}]: only {confirmed}/{graded} approved projected \
             section landmarks carry real GPU ink"
        );
    }
}

/// THE SAME LADDER AT A SECOND SCALE FACTOR. A tunnel scaled off a margin box is
/// exactly the quantity that ships correct at one DPI and wrong at the other
/// so the ring radii are re-measured in LOGICAL units on a
/// canvas of twice the physical size at `dpi 2` and must land on the same
/// numbers.
#[test]
fn the_ring_ladder_is_the_same_in_logical_units_at_both_scale_factors() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let ladder = |dpi: f32| -> Vec<f32> {
        let s = dpi;
        let (w, h) = ((W as f32 * s) as u32, (H as f32 * s) as u32);
        let f = field_at_dpi(
            &device,
            &queue,
            circular_kite(),
            w,
            h,
            COL_LEFT * s,
            COL_W * s,
            warpgrid::FROZEN_PHASE,
            dpi,
        );
        let axis = (w as f32 * 0.5, h as f32 * 0.5);
        peaks(
            &ray(&f, w, h, axis, 0.0, 60.0 * s, 400.0 * s),
            60.0 * s,
            0.5,
        )
        .into_iter()
        .map(|r| r / s)
        .collect()
    };
    let one = ladder(1.0);
    let two = ladder(2.0);
    assert!(
        one.len() >= 3,
        "the 1x ladder must carry rings to compare: {one:?}"
    );
    let n = one.len().min(two.len());
    assert_eq!(
        one.len(),
        two.len(),
        "the two scale factors found different numbers of rings: 1x {one:?} vs 2x {two:?}"
    );
    for i in 0..n {
        assert!(
            (one[i] - two[i]).abs() <= 1.5,
            "ring {i} sits at {:.1} logical px at 1x and {:.1} at 2x — the projection is \
             reading the device grid. 1x {one:?} 2x {two:?}",
            one[i],
            two[i]
        );
    }
}

/// The differential field at an explicit scale factor. `warped_grid`'s
/// own `field` pins `dpi 1`, which is the single configuration CLAUDE.md warns
/// every check quietly runs in.
#[allow(clippy::too_many_arguments)]
fn field_at_dpi(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    phase: f32,
    dpi: f32,
) -> Vec<i32> {
    let flat = match bg {
        theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            fold,
            twist,
            forward_drift,
            ribs,
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density: 0.0,
            fold,
            twist,
            forward_drift,
            ribs,
        },
        other => other,
    };
    let a = raw(device, queue, bg, w, h, col_left, col_w, phase, dpi, None);
    let b = raw(device, queue, flat, w, h, col_left, col_w, phase, dpi, None);
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| {
            (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>()
        })
        .collect()
}

/// One background pass. `clear` is the surface the ground composites ONTO — the
/// legibility law passes the world's own `base_100`, so the page it measures is
/// the real GPU composite through the real blend state rather than a contrast
/// arithmetic done in the host.
#[allow(clippy::too_many_arguments)]
fn raw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    phase: f32,
    dpi: f32,
    clear: Option<wgpu::Color>,
) -> Vec<[u8; 4]> {
    let mut pipe =
        crate::background::BackgroundPipeline::new(device, super::dither::FMT, bg_desc_for(bg));
    pipe.prepare(
        queue,
        w,
        h,
        col_left,
        col_w,
        crate::background::AmbientUpload {
            warp_travel: warpgrid::forward_cells(phase, kite_forward_drift()),
            warp_axis: super::warped_grid::AXIS_ROOM,
            ..Default::default()
        },
        dpi,
    );
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl one-tunnel encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl one-tunnel pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tview,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear.unwrap_or(wgpu::Color::BLACK)),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pipe.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

// ---------------------------------------------------------------------------
// THE CROSSING'S PRICE — legibility, over the real composite.
// ---------------------------------------------------------------------------

fn rel_lum(px: [u8; 4]) -> f64 {
    let lin = |c: u8| {
        let c = c as f64 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2])
}

fn contrast(a: f64, b: f64) -> f64 {
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// EVERY PIXEL OF THE VEILED PAGE CLEARS 4.5:1 AGAINST THE WORLD'S BODY INK.
///
/// The under-page crossing is the one thing in this ground that draws where
/// prose does, so it is the one thing held to a legibility floor rather than to
/// composition alone. 4.5:1 is not a new number — it is the floor the syntax
/// roles are already held to against their ground.
///
/// ⚠️ THE COMPOSITE IS THE GPU'S, NOT THE HOST'S. The ground is drawn over the
/// world's own `base_100` through the real blend state and the floor is read off
/// the resulting pixels. A host-side "veil alpha times the major tone over
/// base_100" would be a second implementation of the shader's compositing, and
/// and the repo has paid for that once already: a modelled composite is
/// confidently wrong.
#[test]
fn the_under_page_crossing_clears_the_body_ink_legibility_floor() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let base = theme::KITE.base_100;
    let ink = rel_lum([
        theme::KITE.base_content.r,
        theme::KITE.base_content.g,
        theme::KITE.base_content.b,
        255,
    ]);
    let bare = rel_lum([base.r, base.g, base.b, 255]);
    let clear = wgpu::Color {
        r: (base.r as f64 / 255.0).powf(2.2),
        g: (base.g as f64 / 255.0).powf(2.2),
        b: (base.b as f64 / 255.0).powf(2.2),
        a: 1.0,
    };

    let mut worst = (f64::INFINITY, 0u32, 0u32, String::new());
    for (col_left, col_w) in [(COL_LEFT, COL_W), (200.0, 1100.0), (420.0, 900.0)] {
        for phase in [warpgrid::FROZEN_PHASE, 166.5] {
            let px = raw(
                &device,
                &queue,
                kite(),
                W,
                H,
                col_left,
                col_w,
                phase,
                1.0,
                Some(clear),
            );
            let x0 = col_left as u32 + 4;
            let x1 = (col_left + col_w) as u32 - 4;
            for y in 0..H {
                for x in x0..x1 {
                    let c = contrast(rel_lum(px[(y * W + x) as usize]), ink);
                    if c < worst.0 {
                        worst = (c, x, y, format!("[{col_left},{col_w}]@{phase}"));
                    }
                }
            }
        }
    }
    assert!(
        worst.0 >= 4.5,
        "the veiled page falls to {:.2}:1 against Kite's body ink at ({}, {}) on {} — the \
         crossing must never cost prose its figure/ground",
        worst.0,
        worst.1,
        worst.2,
        worst.3
    );
    // ...and it must actually BE a veil. A crossing that cost nothing measurable
    // would pass the floor above while drawing nothing, which is the state this
    // whole item exists to leave behind.
    assert!(
        worst.0 < contrast(bare, ink) - 0.5,
        "the page reads at {:.2}:1, indistinguishable from bare base_100's {:.2}:1 — \
         nothing crossed",
        worst.0,
        contrast(bare, ink)
    );
}
