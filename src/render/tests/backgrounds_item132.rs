//! Kite's warped-grid laws. State/data claims use exhaustive enum and roster
//! sweeps; appearance claims use differential arithmetic over the real GPU
//! output rather than trusting the sidecar.

use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use crate::{theme, warpgrid};

fn rgb_delta(a: [u8; 4], b: [u8; 4]) -> u32 {
    (0..3)
        .map(|channel| (a[channel] as i32 - b[channel] as i32).unsigned_abs())
        .sum()
}

fn changed_field(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    col_left: f32,
    col_width: f32,
    phase: f32,
) -> (Vec<[u8; 4]>, Vec<[u8; 4]>) {
    let authored = bg_desc_for(theme::KITE.background);
    let mut flat = authored;
    flat.density = 0.0;
    (
        render_bg(
            device, queue, authored, width, height, col_left, col_width, phase,
        ),
        render_bg(
            device, queue, flat, width, height, col_left, col_width, phase,
        ),
    )
}

#[test]
fn warped_grid_is_kite_alone_no_wildcard() {
    for world in theme::THEMES {
        let is_warped_grid = match world.background {
            theme::Background::Gradient { .. } => false,
            theme::Background::Dots { .. } => false,
            theme::Background::Starfield { .. } => false,
            theme::Background::Pinstripe { .. } => false,
            theme::Background::Stripes { .. } => false,
            theme::Background::Lava { .. } => false,
            theme::Background::Bands { .. } => false,
            theme::Background::Waves { .. } => false,
            theme::Background::Zigzag { .. } => false,
            theme::Background::Organic { .. } => false,
            theme::Background::WarpedGrid { .. } => true,
        };
        assert_eq!(
            is_warped_grid,
            world.name == "Kite",
            "{} warped-grid roster",
            world.name
        );
    }
}

#[test]
fn kite_owns_distinct_data_not_an_identity_branch() {
    let theme::Background::WarpedGrid {
        tones,
        spacing_px,
        density,
        curvature,
    } = theme::KITE.background
    else {
        panic!("Kite must select the reusable WarpedGrid capability");
    };
    assert!((36.0..=96.0).contains(&spacing_px));
    assert!((0.4..=0.9).contains(&density));
    assert!((0.5..=1.2).contains(&curvature));
    assert!(tones.windows(2).all(|pair| pair[0] != pair[1]));
    assert_eq!(theme::KITE.font, "Fira Sans");
    assert_eq!(
        theme::THEMES
            .iter()
            .filter(|world| world.font == "Fira Sans")
            .map(|world| world.name)
            .collect::<Vec<_>>(),
        ["Kite"],
        "the already-bundled OFL face gives Kite a distinct Latin voice"
    );

    let shader = include_str!("../../../shaders/background.wgsl");
    assert!(
        shader.contains("if (g.shader == 9u) { return vec4<f32>(warped_grid_rgb(in.px), 1.0); }")
    );
    assert!(
        !shader.contains("Kite"),
        "the reusable renderer must not branch on world identity"
    );
}

#[test]
fn shader_route_and_line_hierarchy_mirror_the_pure_owner() {
    let shader = include_str!("../../../shaders/background.wgsl");
    for source_fact in [
        "let leg_seconds = 58.0;",
        "let loop_seconds = 348.0;",
        "phase / loop_seconds * 64.0",
        "round(nearest / 5.0) * 5.0",
        "let ring_major_line = warp_line(ring_coord, 0.82) * ring_major;",
        "let ring_minor_line = warp_line(ring_coord, 0.34);",
        "let alias_fade = smoothstep(2.8, 4.6, projected_minor_px);",
    ] {
        assert!(
            shader.contains(source_fact),
            "WGSL/Rust route or hierarchy mirror drifted at `{source_fact}`"
        );
    }
    assert_eq!(warpgrid::ROUTE_LEG_SECONDS, 58.0);
    assert_eq!(warpgrid::ROUTE_LOOP_SECONDS, 348.0);
    assert_eq!(warpgrid::FORWARD_CELLS_PER_LOOP, 64.0);
}

#[test]
fn warped_grid_occupies_both_margins_at_room_frame_and_scale_geometries_only() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping warped-grid GPU geometry law: no wgpu adapter");
        return;
    };
    let _guard = crate::testlock::serial();
    // Narrow Room, canonical Room, wide/2x Frame, and zoom-like wider page.
    for (width, height, left, col) in [
        (800, 600, 40.0, 720.0),
        (1200, 800, 250.0, 700.0),
        (1800, 1000, 350.0, 1100.0),
        (2400, 1600, 430.0, 1540.0),
    ] {
        let (field, flat) = changed_field(&device, &queue, width, height, left, col, 0.0);
        let delta = |x: u32, y: u32| {
            rgb_delta(
                field[(y * width + x) as usize],
                flat[(y * width + x) as usize],
            )
        };
        let left_marks = (0..left as u32)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .filter(|&(x, y)| delta(x, y) >= 12)
            .count();
        let right_marks = ((left + col) as u32..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .filter(|&(x, y)| delta(x, y) >= 12)
            .count();
        assert!(
            left_marks > (height / 10) as usize && right_marks > (height / 10) as usize,
            "{width}x{height}: both Frame slices must retain a readable scaffold ({left_marks}/{right_marks})"
        );
        assert!(
            (left as u32..(left + col) as u32).all(|x| (0..height)
                .all(|y| field[(y * width + x) as usize] == flat[(y * width + x) as usize])),
            "{width}x{height}: warped-grid ink entered the opaque page column"
        );
    }
}

#[test]
fn route_poses_change_one_coherent_field_and_turns_open_opposite_sides() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping warped-grid GPU route law: no wgpu adapter");
        return;
    };
    let _guard = crate::testlock::serial();
    let (width, height, left, col) = (1400, 900, 300.0, 800.0);
    let desc = bg_desc_for(theme::KITE.background);
    let phases = [
        0.0,
        warpgrid::ROUTE_LEG_SECONDS,
        warpgrid::ROUTE_LEG_SECONDS * 2.0,
        warpgrid::ROUTE_LEG_SECONDS * 3.0,
        warpgrid::ROUTE_LEG_SECONDS * 4.0,
    ];
    let frames: Vec<Vec<[u8; 4]>> = phases
        .iter()
        .map(|&phase| render_bg(&device, &queue, desc, width, height, left, col, phase))
        .collect();
    let mut flat_desc = desc;
    flat_desc.density = 0.0;
    for (&phase, frame) in phases.iter().zip(frames.iter()) {
        let flat = render_bg(&device, &queue, flat_desc, width, height, left, col, phase);
        assert!(
            (left as u32..(left + col) as u32).all(|x| (0..height)
                .all(|y| frame[(y * width + x) as usize] == flat[(y * width + x) as usize])),
            "route pose {phase}s painted through the page column"
        );
    }
    for (phase, frame) in phases.iter().zip(frames.iter()).skip(1) {
        let changed = frames[0]
            .iter()
            .zip(frame.iter())
            .enumerate()
            .filter(|(index, (a, b))| {
                let x = (*index as u32) % width;
                ((x as f32) < left || (x as f32) >= left + col) && rgb_delta(**a, **b) >= 6
            })
            .count();
        assert!(
            changed > 8_000,
            "named route pose {phase}s did not visibly transform both slices ({changed} pixels)"
        );
    }

    let transition_count = |frame: &[[u8; 4]], x0: u32, x1: u32| -> usize {
        let y = height / 2 + 91;
        (x0 + 1..x1)
            .filter(|&x| {
                let a = frame[(y * width + x - 1) as usize];
                let b = frame[(y * width + x) as usize];
                rgb_delta(a, b) >= 10
            })
            .count()
    };
    let left_turn = &frames[1];
    let right_turn = &frames[3];
    let left_slice_when_left = transition_count(left_turn, 0, left as u32);
    let right_slice_when_left = transition_count(left_turn, (left + col) as u32, width);
    let left_slice_when_right = transition_count(right_turn, 0, left as u32);
    let right_slice_when_right = transition_count(right_turn, (left + col) as u32, width);
    assert!(
        left_slice_when_left > left_slice_when_right
            && right_slice_when_right > right_slice_when_left,
        "turn compression must swap sides as one field: left {left_slice_when_left}/{left_slice_when_right}, right {right_slice_when_left}/{right_slice_when_right}"
    );
}

#[test]
fn major_minor_value_ladder_edge_fade_and_alias_guard_are_visible_in_pixels() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping warped-grid GPU value law: no wgpu adapter");
        return;
    };
    let _guard = crate::testlock::serial();
    let (width, height, left, col) = (1400, 900, 300.0, 800.0);
    let (field, flat) = changed_field(&device, &queue, width, height, left, col, 0.0);
    let delta_at = |x: u32, y: u32| {
        rgb_delta(
            field[(y * width + x) as usize],
            flat[(y * width + x) as usize],
        )
    };
    let margin_deltas: Vec<u32> = (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .filter(|&(x, _)| (x as f32) < left || (x as f32) >= left + col)
        .map(|(x, y)| delta_at(x, y))
        .collect();
    assert!(
        margin_deltas.iter().filter(|&&delta| delta >= 180).count() > 500,
        "major rails must occupy the deep graphite rung"
    );
    assert!(
        margin_deltas
            .iter()
            .filter(|&&delta| (30..150).contains(&delta))
            .count()
            > 2_000,
        "minor rails must occupy their quieter indigo rung"
    );

    let near_page: u64 = (0..height)
        .map(|y| delta_at(left as u32 - 3, y) as u64)
        .sum();
    let far_page: u64 = (0..height)
        .map(|y| delta_at(left as u32 - 70, y) as u64)
        .sum();
    assert!(
        near_page * 3 < far_page,
        "the grid must recede beside the page edge ({near_page} vs {far_page})"
    );

    // An aliasing regression presents as isolated one-pixel dark/light flips.
    // fwidth suppression may leave a few intersections, never a checker field.
    let mut isolated = 0usize;
    let mut marked = 0usize;
    for y in 1..height - 1 {
        for x in 1..left as u32 - 1 {
            let here = delta_at(x, y) >= 24;
            if here {
                marked += 1;
                let horizontal = delta_at(x - 1, y) >= 24 || delta_at(x + 1, y) >= 24;
                let vertical = delta_at(x, y - 1) >= 24 || delta_at(x, y + 1) >= 24;
                isolated += usize::from(!horizontal && !vertical);
            }
        }
    }
    assert!(marked > 2_000);
    assert!(
        isolated * 100 < marked * 2,
        "minor-grid suppression regressed into isolated shimmer pixels ({isolated}/{marked})"
    );
}

#[test]
fn ambient_gates_freeze_kites_long_phase_without_catchup() {
    assert!(theme::KITE.has_ambient_motion());
    assert!(theme::KITE.has_ambient_tick());
    assert_eq!(
        warpgrid::phase_for(227.0, true, None),
        warpgrid::FROZEN_PHASE
    );
    assert_eq!(
        warpgrid::advance_phase(18.0, 12.0),
        warpgrid::advance_phase(18.0, crate::lava::LAVA_TICK_SECONDS),
        "a refocus/resume wake must not catch the route up"
    );
}
