//! Fixed-framing and forward-travel laws for Kite's warped grid.

use super::backgrounds_item69::headless_dq;
use super::backgrounds_item132::{H, INK_FLOOR, W, field, kite, with_tunnel};
use crate::{theme, warpgrid};

const OUTER_BAND: u32 = 180;
const SECTION_ROOM_FRAC: f32 = 0.432;
// Mirrors of `background.wgsl`. `WINDOW_INSET` went with the placement it
// belonged to: the two per-margin axes were replaced by ONE at the room's own
// centre, so a radial scan sweeps about `(W/2, H/2)` and the axis is no longer a
// function of the anchor at all. `RING_PITCH_AT` and `RPO_MAX` moved with it —
// see the shader's own note on why the reference point had to follow the axis
// outward.
const RING_PITCH_AT: f32 = 0.8333333;
const RPO_MIN: f32 = 3.0;
const RPO_MAX: f32 = 20.0;

fn outer_pixels(frame: &[i32], col_left: f32, col_w: f32) -> Vec<i32> {
    let col_right = col_left + col_w;
    assert!(col_left >= OUTER_BAND as f32);
    assert!(W as f32 - col_right >= OUTER_BAND as f32);
    let mut out = Vec::with_capacity((2 * OUTER_BAND * H) as usize);
    for y in 0..H {
        for x in 0..OUTER_BAND {
            out.push(frame[(y * W + x) as usize]);
        }
        for x in W - OUTER_BAND..W {
            out.push(frame[(y * W + x) as usize]);
        }
    }
    out
}

fn framing_samples(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
) -> Vec<Vec<i32>> {
    // Both centred and deliberately asymmetric columns. Every margin is wider
    // than the narrow/edge fades, so the outer bands expose the same room field.
    [
        (300.0, 1000.0),
        (400.0, 800.0),
        (500.0, 600.0),
        (360.0, 760.0),
    ]
    .into_iter()
    .map(|(left, width)| {
        let frame = field(device, queue, bg, W, H, left, width, warpgrid::FROZEN_PHASE);
        outer_pixels(&frame, left, width)
    })
    .collect()
}

#[test]
fn fixed_framing_is_room_owned_across_page_width_and_offset() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let frames = framing_samples(&device, &queue, kite());
    for frame in &frames[1..] {
        let changed = frame.iter().zip(&frames[0]).filter(|(a, b)| a != b).count();
        assert_eq!(
            changed, 0,
            "page width or offset changed {changed} pixels in the room-owned outer field"
        );
    }
    assert!(
        frames[0].iter().filter(|v| **v > INK_FLOOR).count() > 10_000,
        "the invariant band must contain a real field"
    );
}

#[test]
fn page_owned_framing_mutations_break_the_width_invariant() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for mutation in [theme::Tunnel::PageScaled, theme::Tunnel::MarginPlaced] {
        let frames = framing_samples(&device, &queue, with_tunnel(kite(), mutation));
        let changed = frames[1..]
            .iter()
            .map(|frame| frame.iter().zip(&frames[0]).filter(|(a, b)| a != b).count())
            .max()
            .unwrap_or(0);
        assert!(
            changed > 1_000,
            "{mutation:?} must visibly reframe the outer field; only {changed} pixels changed"
        );
    }
}

fn travel_phase(cells: f32) -> f32 {
    cells / warpgrid::FORWARD_CELLS_PER_LOOP * warpgrid::LOOP_SECONDS
}

fn strongest_major_ring_in(frame: &[i32], radii: std::ops::Range<i32>) -> f32 {
    // THE ONE AXIS: the room's own centre, which is the whole of the placement.
    // A scan that still swept about a per-margin axis would be measuring a
    // picture the shader no longer draws.
    let axis = (W as f32 * 0.5, H as f32 * 0.5);
    let mut best = (f64::MIN, 0.0);
    for radius in radii {
        let mut score = 0.0;
        let mut samples = 0usize;
        for i in 0..192 {
            let theta = std::f32::consts::TAU * i as f32 / 192.0;
            let x = (axis.0 + radius as f32 * theta.cos()).round() as i32;
            let y = (axis.1 + radius as f32 * theta.sin()).round() as i32;
            if x < 0 || x >= W as i32 || y < 0 || y >= H as i32 {
                continue;
            }
            score += frame[(y as u32 * W + x as u32) as usize] as f64;
            samples += 1;
        }
        score /= samples.max(1) as f64;
        if score > best.0 {
            best = (score, radius as f32);
        }
    }
    assert!(best.0 > INK_FLOOR as f64, "radial scan found no ring");
    best.1
}

fn ring_ladder(device: &wgpu::Device, queue: &wgpu::Queue, bg: theme::Background) -> Vec<f32> {
    let mut radii = Vec::new();
    for cells in [0.0, 0.4, 0.8] {
        let frame = field(device, queue, bg, W, H, 324.0, 950.0, travel_phase(cells));
        // THE SCAN BAND MOVED OUTWARD WITH THE AXIS. With one axis at the room's
        // centre the near field is under the page at `WARP_PAGE_VEIL`, so a ladder
        // read there would be grading a veiled fraction of the field. This band sits where
        // the circle passes through BOTH margins at full strength, and its width
        // is set by the ring family's own spacing at that radius (~220px between
        // consecutive majors), so exactly one major can fall inside it.
        let range = radii.last().map_or(560..760, |last: &f32| {
            (*last as i32 - 45).max(1)..(*last as i32 + 45)
        });
        radii.push(strongest_major_ring_in(&frame, range));
    }
    radii
}

#[test]
fn forward_travel_grows_the_projected_rings_at_the_authored_rate() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let radii = ring_ladder(&device, &queue, kite());
    assert!(radii.windows(2).all(|w| w[1] > w[0]), "{radii:?}");

    let spacing = match kite() {
        theme::Background::WarpedGrid { spacing_px, .. } => spacing_px,
        _ => unreachable!(),
    };
    let anchor = SECTION_ROOM_FRAC * H as f32;
    let rpo = (RING_PITCH_AT * anchor * std::f32::consts::LN_2 / spacing).clamp(RPO_MIN, RPO_MAX);
    let measured = (radii[2] / radii[0]).log2() / 0.8;
    let expected = 1.0 / rpo;
    assert!(
        (measured - expected).abs() <= expected * 0.28,
        "rings grew {measured:.3} octaves/cell; expected {expected:.3}: {radii:?}"
    );
}

#[test]
fn reversed_travel_mutation_recedes() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let radii = ring_ladder(
        &device,
        &queue,
        with_tunnel(kite(), theme::Tunnel::Reversed),
    );
    assert!(radii.windows(2).all(|w| w[1] < w[0]), "{radii:?}");
}

#[test]
fn shader_has_fixed_geometry_and_no_steering_path() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for present in [
        "let w = q;",
        "let travel = select(g.warp_travel, -g.warp_travel, reversed);",
        // The placement owner takes a viewport width and NOTHING else — no page,
        // no margin, and above all no side. That signature is what makes "one
        // vanishing point" a property of the code rather than of a tuning.
        "fn warp_room_axis(vp_x: f32) -> f32 {",
    ] {
        assert!(wgsl.contains(present), "missing `{present}`");
    }
    for absent in [
        "WARP_BEND_GAIN",
        "WARP_SOLVE_STEPS",
        "g.pose",
        "per_margin",
        "warp_window_axis",
        "WARP_WINDOW_INSET",
    ] {
        assert!(
            !wgsl.contains(absent),
            "obsolete steering path `{absent}` remains"
        );
    }
}
