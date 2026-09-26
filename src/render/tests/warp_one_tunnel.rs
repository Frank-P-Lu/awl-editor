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
//! * the field under the page follows the same folded projected section;
//! * projected section landmarks continue across page and margin masks;
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
use super::warped_grid::{COL_LEFT, COL_W, H, INK_FLOOR, W, field, kite};
use crate::theme;
use crate::warpgrid;

fn kite_forward_drift() -> f32 {
    match kite() {
        theme::Background::WarpedGrid { forward_drift, .. } => forward_drift,
        _ => unreachable!(),
    }
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

// ---------------------------------------------------------------------------
// ONE PROJECTED SURFACE.
// ---------------------------------------------------------------------------

/// THE FIELD UNDER THE PAGE PRESERVES THE FOLDED REFERENCE SECTION.
///
/// This supersedes the radius-alone assertion: under the page only major
/// sections draw, so the sparse pixel oracle asks for the depth-dependent,
/// displaced, rolled, multiplicatively folded contour itself.
#[test]
fn the_field_under_the_page_keeps_the_folded_reference_section() {
    super::warp_projection::the_visible_far_section_is_folded_not_a_circular_target();
}

/// THE SECTION THAT LEAVES A MARGIN ARRIVES UNDER THE PAGE, at the projected
/// coordinates one surface predicts. The complete angular axis of each major
/// section is sampled against real GPU pixels across several page masks.
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
    use crate::warpgrid::projection::{BODY_Z, NEAR_Z, Point, Projection};
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
        let step_z = (BODY_Z - NEAR_Z) / 58.0;
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

/// THE SAME PROJECTED LANDMARKS AT BOTH SCALE FACTORS. This supersedes the
/// circular ray/peak ladder: the real contour and fixed-theta rails must land in
/// logical space at 1x and 2x, including the alias fade at each device scale.
#[test]
fn projected_landmarks_are_the_same_in_logical_units_at_both_scale_factors() {
    super::warp_projection::projected_reference_landmarks_are_visible_at_both_viewports_and_dpis();
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
