//! ONE TUNNEL (item 268) — the laws that would have caught two.
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
use super::warped_grid::{COL_LEFT, COL_W, H, INK_FLOOR, W, field, kite};
use crate::theme;
use crate::warpgrid;

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

/// The canonical room's single axis and its page half-width.
fn axis_and_page_half(w: u32, h: u32, col_left: f32, col_w: f32) -> ((f32, f32), f32) {
    let axis = (w as f32 * 0.5, h as f32 * 0.5);
    // How far the page reaches from the axis on its NEARER side — the radius
    // out to which every direction is still under the page.
    let half = (axis.0 - col_left).min(col_left + col_w - axis.0);
    (axis, half.max(1.0))
}

// ---------------------------------------------------------------------------
// ONE AXIS — and the same measurement proves the section is a circle.
// ---------------------------------------------------------------------------

/// THE FIELD UNDER THE PAGE IS A FUNCTION OF RADIUS ALONE.
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
fn the_field_under_the_page_is_a_function_of_radius_alone() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    // Centred and deliberately OFF-CENTRE columns: an axis that read the page or
    // the margin would survive the symmetric case and die here.
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
        let (axis, page_half) = axis_and_page_half(W, H, col_left, col_w);
        // Clear of the core haze at the near end and of the page edge at the far
        // end, so every sampled direction is in the uniform veiled band.
        let (lo, hi) = (100.0f32, page_half - 12.0);
        assert!(hi > lo + 80.0, "the under-page band must be worth sweeping");

        let angles: Vec<f32> = (0..24)
            .map(|i| std::f32::consts::TAU * i as f32 / 24.0)
            .collect();
        let profiles: Vec<Vec<f32>> = angles
            .iter()
            .map(|t| ray(&f, W, H, axis, *t, lo, hi))
            .collect();
        let n = profiles.iter().map(Vec::len).min().unwrap_or(0);
        assert!(n > 100, "profiles too short to grade: {n}");

        // WHAT IS COMPARED IS THE LADDER OF RING RADII PER DIRECTION, never the
        // ink pointwise. A ring is a one-pixel line, so two directions sampled at
        // the same radius land at different sub-pixel offsets across the raster
        // and their ink differs by everything while the geometry is identical —
        // a pointwise comparison grades the rounding, not the section.
        let ladders: Vec<Vec<f32>> = profiles.iter().map(|p| peaks(p, lo, 0.5)).collect();
        let rings = ladders[0].len();
        assert!(
            rings >= 3,
            "the under-page band must carry at least three rings, found {rings}: {:?}",
            ladders[0]
        );
        for (k, l) in ladders.iter().enumerate() {
            assert_eq!(
                l.len(),
                rings,
                "column [{col_left},{col_w}]: direction {k} ({:.0} deg) finds {} rings \
                 against {rings} straight along +x — {l:?} vs {:?}",
                angles[k].to_degrees(),
                l.len(),
                ladders[0]
            );
            for i in 0..rings {
                assert!(
                    (l[i] - ladders[0][i]).abs() <= 2.0,
                    "column [{col_left},{col_w}]: ring {i} sits at {:.1} straight along +x \
                     and at {:.1} at {:.0} deg — the section is not circular, or the two \
                     flanks are not looking at one axis",
                    ladders[0][i],
                    l[i],
                    angles[k].to_degrees()
                );
            }
        }
    }
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
    // `rpo` and the major modulus mirror `background.wgsl`; the ratio is the
    // projection's own, never a number fitted to the output.
    let anchor = 0.432f32 * H as f32;
    let spacing = match kite() {
        theme::Background::WarpedGrid { spacing_px, .. } => spacing_px,
        _ => unreachable!(),
    };
    let rpo = (0.8333333 * anchor * std::f32::consts::LN_2 / spacing).clamp(3.0, 20.0);
    let step = (warpgrid::MAJOR_EVERY / rpo).exp2();

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
        let axis = (W as f32 * 0.5, H as f32 * 0.5);
        // Straight up is under the page at every column this sweeps, so the seed
        // arc is read at the veil, in the one direction no page width can move.
        let up = ray(&f, W, H, axis, -std::f32::consts::FRAC_PI_2, 100.0, 420.0);
        let seed = peaks(&up, 100.0, 0.5);
        assert!(
            seed.len() >= 2,
            "column [{col_left},{col_w}]: no seed ladder straight up — {seed:?}"
        );
        let r0 = seed[0];

        let mut graded = 0usize;
        let mut r = r0;
        while r < 900.0 {
            for k in 0..24 {
                // OFF THE RAILS, DELIBERATELY. `WARP_RAILS_PER_HALF_TURN` puts a
                // MAJOR rail at 0, 90, 180 and 270 degrees, and a ray fired
                // straight along one runs inside it for its whole length — every
                // window then reads rail ink and the measurement says nothing
                // about rings. The half-step offset keeps all twenty-four
                // directions between rails.
                let theta = std::f32::consts::TAU * (k as f32 + 0.5) / 24.0;
                let win = (0.12 * r).max(8.0);
                // THE PAGE-EDGE RAMP IS NOT GRADED, and the reason is the same one
                // that keeps `the_field_under_the_page_is_a_function_of_radius_alone`
                // clear of it: inside that band the field is transitioning between
                // the veil and full strength, so a window straddling it compares one
                // arc at one strength against another at a different one and grades
                // the RAMP. The two uniform regions either side of it are where a
                // geometric claim can be read at all.
                let sx = axis.0 + theta.cos() * r;
                if (sx - col_left).abs() < 90.0 || (sx - (col_left + col_w)).abs() < 90.0 {
                    continue;
                }
                let prof = ray(&f, W, H, axis, theta, r - win, r + win);
                if prof.iter().any(|v| v.is_nan()) {
                    continue; // this direction runs off the canvas at this radius
                }
                // THE TOLERANCE SCALES WITH THE RADIUS, because the prediction
                // is a PRODUCT: the seed arc is located to about a pixel and each
                // step multiplies that error along with everything else, so by the
                // fourth ring a fixed few-pixel window has drifted off the arc it
                // was aimed at. Half the local ring spacing is the bound that
                // matters — the window must never be wide enough to admit the
                // neighbouring ring — and at this ratio it stays far inside it.
                let half = 3.0 + 0.012 * r;
                let idx = |x: f32| ((x - (r - win)) / 0.5) as usize;
                let (lo, hi) = (idx(r - half), idx(r + half).min(prof.len() - 1));
                let on = prof[lo..=hi].iter().cloned().fold(0.0f32, f32::max);
                let off = prof[..idx(r - half - 2.0)]
                    .iter()
                    .chain(prof[idx(r + half + 2.0).min(prof.len() - 1)..].iter())
                    .cloned()
                    .fold(0.0f32, f32::max);
                assert!(
                    on > INK_FLOOR as f32,
                    "column [{col_left},{col_w}]: nothing at radius {r:.1} at {:.0} deg — \
                     one tunnel predicts an arc there and this direction has none",
                    theta.to_degrees()
                );
                // The 0.85 is quantization slack, not a weakened claim: at the
                // veil an arc reads about 53 total-channel units and two arcs a
                // window apart can tie within a couple of them. A ring centred
                // somewhere else does not miss by two units — it misses by the
                // whole difference between full strength and blank ground.
                assert!(
                    on >= off * 0.85,
                    "column [{col_left},{col_w}]: at {:.0} deg the strongest mark near \
                     radius {r:.1} is {off:.0} OFF the prediction against {on:.0} on it — \
                     this direction's arcs are centred somewhere else, which is what two \
                     tunnels look like",
                    theta.to_degrees()
                );
                graded += 1;
            }
            r *= step;
        }
        assert!(
            graded >= 40,
            "column [{col_left},{col_w}]: only {graded} direction/radius cells graded"
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
            kite(),
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
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density: 0.0,
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
            warp_travel: warpgrid::forward_cells(phase),
            ..Default::default()
        },
        dpi,
    );
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item268-warp encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl item268-warp pass"),
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
        for phase in [warpgrid::FROZEN_PHASE, warpgrid::LOOP_SECONDS * 0.41] {
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
