//! Projected-tube landmark laws.
//!
//! The CPU mirror does not stand in for rendering: it supplies reference
//! coordinates, then these laws ask the real GPU pixels for ink at those
//! coordinates. A circular polar substitute can satisfy generic coverage and
//! contrast laws; it cannot put marks on this folded, displaced, rolled mesh.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::warped_grid::{INK_FLOOR, kite, render_travel_axis_dpi};
use crate::theme;
use crate::warpgrid::projection::{FAR_Z, NEAR_Z, Point, Projection, RAILS};

fn with_density(bg: theme::Background, density: f32) -> theme::Background {
    match bg {
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
            density,
            fold,
            twist,
            forward_drift,
            ribs,
        },
        other => other,
    }
}

struct Field {
    px: Vec<i32>,
    physical_w: u32,
    physical_h: u32,
    dpi: f32,
}

impl Field {
    fn strongest_near(&self, p: Point, radius_logical: i32) -> i32 {
        let x = (p.x * self.dpi).round() as i32;
        let y = (p.y * self.dpi).round() as i32;
        let r = (radius_logical as f32 * self.dpi).ceil() as i32;
        let mut strongest = 0;
        for yy in (y - r).max(0)..=(y + r).min(self.physical_h as i32 - 1) {
            for xx in (x - r).max(0)..=(x + r).min(self.physical_w as i32 - 1) {
                strongest =
                    strongest.max(self.px[(yy as u32 * self.physical_w + xx as u32) as usize]);
            }
        }
        strongest
    }
}

#[allow(clippy::too_many_arguments)]
fn field(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    logical_w: u32,
    logical_h: u32,
    col_left: f32,
    col_w: f32,
    dpi: f32,
    axis: (f32, f32),
) -> Field {
    let physical_w = (logical_w as f32 * dpi) as u32;
    let physical_h = (logical_h as f32 * dpi) as u32;
    let render = |bg| {
        render_travel_axis_dpi(
            device,
            queue,
            bg_desc_for(bg),
            physical_w,
            physical_h,
            col_left * dpi,
            col_w * dpi,
            0.0,
            axis,
            dpi,
        )
    };
    let a = render(kite());
    let b = render(with_density(kite(), 0.0));
    Field {
        px: a
            .iter()
            .zip(b.iter())
            .map(|(p, q)| {
                (0..3)
                    .map(|channel| (p[channel] as i32 - q[channel] as i32).abs())
                    .sum()
            })
            .collect(),
        physical_w,
        physical_h,
        dpi,
    }
}

#[allow(clippy::too_many_arguments)]
fn field_desc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    logical_w: u32,
    logical_h: u32,
    col_left: f32,
    col_w: f32,
    dpi: f32,
    axis: (f32, f32),
    mut desc: crate::background::BgDesc,
) -> Field {
    let physical_w = (logical_w as f32 * dpi) as u32;
    let physical_h = (logical_h as f32 * dpi) as u32;
    let a = render_travel_axis_dpi(
        device,
        queue,
        desc,
        physical_w,
        physical_h,
        col_left * dpi,
        col_w * dpi,
        0.0,
        axis,
        dpi,
    );
    desc.density = 0.0;
    let b = render_travel_axis_dpi(
        device,
        queue,
        desc,
        physical_w,
        physical_h,
        col_left * dpi,
        col_w * dpi,
        0.0,
        axis,
        dpi,
    );
    Field {
        px: a
            .iter()
            .zip(b.iter())
            .map(|(p, q)| {
                (0..3)
                    .map(|channel| (p[channel] as i32 - q[channel] as i32).abs())
                    .sum()
            })
            .collect(),
        physical_w,
        physical_h,
        dpi,
    }
}

fn projection(logical_w: u32, logical_h: u32, axis: (f32, f32)) -> Projection {
    let (fold, twist) = match kite() {
        theme::Background::WarpedGrid { fold, twist, .. } => (fold, twist),
        _ => unreachable!(),
    };
    Projection {
        width: logical_w as f32,
        height: logical_h as f32,
        vanish: Point {
            x: axis.0 * logical_w as f32,
            y: axis.1 * logical_h as f32,
        },
        fold,
        twist,
        travel_z: 0.0,
        spin: 0.0,
    }
}

fn on_canvas_margin(p: Point, w: u32, h: u32, col_left: f32, col_w: f32) -> bool {
    p.x >= 4.0
        && p.y >= 4.0
        && p.x < w as f32 - 4.0
        && p.y < h as f32 - 4.0
        && (p.x < col_left - 4.0 || p.x >= col_left + col_w + 4.0)
}

fn on_canvas(p: Point, w: u32, h: u32) -> bool {
    p.x >= 4.0 && p.y >= 4.0 && p.x < w as f32 - 4.0 && p.y < h as f32 - 4.0
}

#[test]
pub(super) fn projected_reference_landmarks_are_visible_at_both_viewports_and_dpis() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for (w, h, col_left, col_w, dpi, axis) in [
        (1200, 800, 312.0, 576.0, 1.0, (0.80, 0.24)),
        (1600, 1000, 324.0, 950.0, 1.0, (0.20, 0.76)),
        (1200, 800, 312.0, 576.0, 2.0, (0.50, 0.50)),
    ] {
        let pixels = field(&device, &queue, w, h, col_left, col_w, dpi, axis);
        let p = projection(w, h, axis);
        let step_z = (FAR_Z - NEAR_Z) / 58.0;
        let mut seen = 0usize;
        let mut enrolled = 0usize;

        // Every fifth cross-section is the major scaffold. Sample mesh vertices,
        // not an imagined radial proxy, over the whole angular axis.
        for ring_i in (5..=55).step_by(5) {
            let z = NEAR_Z + ring_i as f32 * step_z;
            for theta_i in (0..128).step_by(4) {
                let point = p.point(std::f32::consts::TAU * theta_i as f32 / 128.0, z);
                if on_canvas(point, w, h) {
                    enrolled += 1;
                    let in_page = point.x >= col_left && point.x < col_left + col_w;
                    let floor = if in_page { 0 } else { INK_FLOOR };
                    seen += usize::from(pixels.strongest_near(point, 3) > floor);
                }
            }
        }

        // Fixed-theta longitudinal rails use the same projected points. This is
        // the visible-pixel witness for path displacement, roll, and curvature.
        for rail_i in 0..RAILS {
            let theta = std::f32::consts::TAU * rail_i as f32 / RAILS as f32;
            for segment_i in (4..88).step_by(7) {
                let z = NEAR_Z + (FAR_Z - NEAR_Z) * segment_i as f32 / 92.0;
                let point = p.point(theta, z);
                if on_canvas_margin(point, w, h, col_left, col_w) {
                    enrolled += 1;
                    seen += usize::from(pixels.strongest_near(point, 3) > INK_FLOOR);
                }
            }
        }

        assert!(
            enrolled > 40,
            "{w}x{h}@{dpi} axis={axis:?}: landmark enrolment is unexpectedly empty"
        );
        let hit_rate = seen as f32 / enrolled as f32;
        assert!(
            hit_rate >= 0.75,
            "{w}x{h}@{dpi} axis={axis:?}: only {seen}/{enrolled} ({:.1}%) \
             approved projected landmarks carry visible ink",
            hit_rate * 100.0
        );
    }
}

/// A sparse under-page render isolates one far major section: rails and minor
/// sections are page-masked, and the convergence haze is margin-only. The real
/// folded reference contour must therefore carry substantially more ink than
/// the rejected circular substitute at the same nominal depth.
#[test]
pub(super) fn the_visible_far_section_is_folded_not_a_circular_target() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let (w, h, col_left, col_w, axis) = (1200, 800, 300.0, 600.0, (0.5, 0.5));
    let mut desc = bg_desc_for(kite());
    desc.warp_ribs = 5.0;
    desc.density = 1.0;
    let pixels = field_desc(&device, &queue, w, h, col_left, col_w, 1.0, axis, desc);
    let folded = projection(w, h, axis);
    let circular = Projection {
        fold: 0.0,
        ..folded
    };
    let hits = |p: Projection| {
        (0..128)
            .step_by(2)
            .filter(|&theta_i| {
                let theta = std::f32::consts::TAU * theta_i as f32 / 128.0;
                pixels.strongest_near(p.point(theta, FAR_Z), 2) > INK_FLOOR
            })
            .count()
    };
    let folded_hits = hits(folded);
    let circular_hits = hits(circular);
    assert!(
        folded_hits >= 42 && folded_hits >= circular_hits + 15,
        "folded section {folded_hits}/64 hits, circular substitute {circular_hits}/64"
    );
}
