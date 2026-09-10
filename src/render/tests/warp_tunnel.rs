//! Fixed-framing and forward-travel laws for Kite's warped grid.

use super::bands_waves::headless_dq;
use super::warped_grid::{H, INK_FLOOR, W, field, kite, with_tunnel};
use crate::{theme, warpgrid};

const OUTER_BAND: u32 = 180;

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

#[test]
fn shader_has_fixed_geometry_and_no_steering_path() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for present in [
        "@vertex\nfn vs_tunnel(",
        "let radius = warp_radius(theta, world_z, fold, twist);",
        concat!(
            "return centre + (warp_path(world_z) + radius * ",
            "vec2<f32>(cos(angle), sin(angle))) * scale;"
        ),
        "let theta = WARP_TAU * f32(rail_i) / f32(WARP_RAIL_SLOTS);",
        "const WARP_RING_SEGMENTS: u32 = 128u;",
        "const WARP_RAIL_SLOTS: u32 = 24u;",
    ] {
        assert!(wgsl.contains(present), "missing `{present}`");
    }
    for absent in [
        "WARP_BEND_GAIN",
        "WARP_SOLVE_STEPS",
        "warp_surface_coord",
        "warped_grid_inverse_rgba",
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
