//! Projected-tube landmark laws.
//!
//! The CPU mirror does not stand in for rendering: it supplies reference
//! coordinates, then these laws ask the real GPU pixels for ink at those
//! coordinates. A circular polar substitute can satisfy generic coverage and
//! contrast laws; it cannot put marks on this folded, displaced, rolled mesh.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::warped_grid::{INK_FLOOR, kite, render_travel_axis_dpi};
use crate::theme;
use crate::warpgrid::projection::{BODY_Z, NEAR_Z, Point, Projection, RAILS};

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

#[derive(Clone)]
struct Field {
    px: Vec<i32>,
    physical_w: u32,
    physical_h: u32,
    dpi: f32,
}

fn ink_in_logical_band(field: &Field, x0: f32, x1: f32) -> usize {
    let start = (x0 * field.dpi).max(0.0).round() as u32;
    let end = (x1 * field.dpi).min(field.physical_w as f32).round() as u32;
    (0..field.physical_h)
        .flat_map(|y| (start..end).map(move |x| (y, x)))
        .filter(|&(y, x)| field.px[(y * field.physical_w + x) as usize] > INK_FLOOR)
        .count()
}

fn percentile(values: &mut [i32], pct: usize) -> i32 {
    values.sort_unstable();
    values[(values.len() - 1) * pct / 100]
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
        let step_z = (BODY_Z - NEAR_Z) / 58.0;
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
                let z = NEAR_Z + (BODY_Z - NEAR_Z) * segment_i as f32 / 92.0;
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
                pixels.strongest_near(p.point(theta, BODY_Z), 2) > INK_FLOOR
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

/// The tube continues beyond its original endpoint, with lighter distant ink.
#[test]
fn distant_sections_continue_with_lighter_ink() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for dpi in [1.0, 2.0] {
        let axis = (0.80, 0.24);
        let mut desc = bg_desc_for(kite());
        desc.warp_ribs = 10.0;
        desc.density = 1.0;
        let pixels = field_desc(&device, &queue, 1200, 800, 0.0, 1200.0, dpi, axis, desc);
        let p = projection(1200, 800, axis);
        let strengths = |ring: f32| {
            let z = NEAR_Z + ring * (BODY_Z - NEAR_Z) / 10.0;
            (0..128)
                .step_by(2)
                .map(|i| {
                    pixels.strongest_near(p.point(std::f32::consts::TAU * i as f32 / 128.0, z), 1)
                })
                .collect::<Vec<_>>()
        };
        let near = percentile(&mut strengths(5.0), 50);
        let far = percentile(&mut strengths(15.0), 50);
        assert!(far >= 2, "dpi={dpi}: extended section lacks ink: {far}");
        assert!(
            far * 3 < near * 2,
            "dpi={dpi}: distant ink {far} competes with near ink {near}"
        );
    }
}

/// The bounded projection is not required to fill every arbitrary sliver of a
/// panoramic page. It is required to remain one visible surface in both margins
/// of the two supported capture geometries, at rest and in transit.
#[test]
pub(super) fn projected_surface_marks_both_supported_margins() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for (w, h, col_left, col_w, dpi, axis) in [
        (1200, 800, 312.0, 576.0, 1.0, (0.80, 0.24)),
        (1200, 800, 312.0, 576.0, 2.0, (0.50, 0.50)),
        (1600, 1000, 418.0, 763.2, 1.0, (0.20, 0.76)),
        (1600, 1000, 418.0, 763.2, 2.0, (0.80, 0.24)),
    ] {
        let pixels = field(&device, &queue, w, h, col_left, col_w, dpi, axis);
        let counts = [
            ink_in_logical_band(&pixels, 0.0, col_left),
            ink_in_logical_band(&pixels, col_left + col_w, w as f32),
        ];
        for (side, count) in ["left", "right"].into_iter().zip(counts) {
            assert!(
                count >= 500,
                "{w}x{h}@{dpi} axis={axis:?} {side} margin has only {count} projected pixels"
            );
        }

        // Non-vacuity: the same detector rejects the exact blank-margin defect.
        let mut blank_left = pixels.clone();
        let left_end = (col_left * dpi).round() as u32;
        for y in 0..blank_left.physical_h {
            for x in 0..left_end {
                blank_left.px[(y * blank_left.physical_w + x) as usize] = 0;
            }
        }
        assert_eq!(ink_in_logical_band(&blank_left, 0.0, col_left), 0);
    }
}

fn projected_edge_crossings(
    pixels: &Field,
    p: Projection,
    edge: f32,
    page_is_right: bool,
) -> (usize, usize) {
    let step_z = (BODY_Z - NEAR_Z) / 58.0;
    let mut enrolled = 0usize;
    let mut confirmed = 0usize;
    for ring_i in (5..=55).step_by(5) {
        let z = NEAR_Z + ring_i as f32 * step_z;
        for theta_i in 0..128 {
            let a = p.point(std::f32::consts::TAU * theta_i as f32 / 128.0, z);
            let b = p.point(std::f32::consts::TAU * (theta_i + 1) as f32 / 128.0, z);
            if (a.x - edge) * (b.x - edge) > 0.0 || (b.x - a.x).abs() < 2.0 {
                continue;
            }
            let t = (edge - a.x) / (b.x - a.x);
            let y = a.y + (b.y - a.y) * t;
            if !(8.0..p.height - 8.0).contains(&y) {
                continue;
            }
            let slope = (b.y - a.y) / (b.x - a.x);
            let left = Point {
                x: edge - 4.0,
                y: y - 4.0 * slope,
            };
            let right = Point {
                x: edge + 4.0,
                y: y + 4.0 * slope,
            };
            let (outside, inside) = if page_is_right {
                (left, right)
            } else {
                (right, left)
            };
            enrolled += 1;
            confirmed += usize::from(
                pixels.strongest_near(outside, 2) > 0 && pixels.strongest_near(inside, 2) > 0,
            );
        }
    }
    (enrolled, confirmed)
}

/// Continuity is evaluated where projected major sections actually cross each
/// page edge, not by averaging a fixed horizontal strip that a curved contour
/// may legitimately miss.
#[test]
pub(super) fn projected_major_sections_cross_both_page_edges() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let (w, h, col_left, col_w, axis) = (1200, 800, 312.0, 576.0, (0.80, 0.24));
    let pixels = field(&device, &queue, w, h, col_left, col_w, 1.0, axis);
    let p = projection(w, h, axis);
    for (name, edge, page_is_right) in
        [("left", col_left, true), ("right", col_left + col_w, false)]
    {
        let (enrolled, confirmed) = projected_edge_crossings(&pixels, p, edge, page_is_right);
        assert!(
            enrolled >= 2 && confirmed * 4 >= enrolled * 3,
            "{name} edge: only {confirmed}/{enrolled} projected section crossings continue"
        );

        let mut severed = pixels.clone();
        let edge_px = (edge * severed.dpi).round() as u32;
        let (x0, x1) = if page_is_right {
            (0, edge_px)
        } else {
            (edge_px, severed.physical_w)
        };
        for y in 0..severed.physical_h {
            for x in x0..x1 {
                severed.px[(y * severed.physical_w + x) as usize] = 0;
            }
        }
        let (_, mutated) = projected_edge_crossings(&severed, p, edge, page_is_right);
        assert!(
            mutated * 4 < enrolled * 3,
            "{name} edge detector did not reject a deliberately severed margin"
        );
    }
}

fn page_and_margin_strengths(
    pixels: &Field,
    p: Projection,
    col_left: f32,
    col_w: f32,
) -> (Vec<i32>, Vec<i32>) {
    let mut page = Vec::new();
    let mut margin = Vec::new();
    let step_z = (BODY_Z - NEAR_Z) / 58.0;
    for ring_i in (5..=55).step_by(5) {
        let z = NEAR_Z + ring_i as f32 * step_z;
        for theta_i in (0..128).step_by(2) {
            let point = p.point(std::f32::consts::TAU * theta_i as f32 / 128.0, z);
            if !on_canvas(point, p.width as u32, p.height as u32) {
                continue;
            }
            let strength = pixels.strongest_near(point, 2);
            if point.x > col_left + 24.0 && point.x < col_left + col_w - 24.0 {
                page.push(strength);
            } else if point.x < col_left - 48.0 || point.x > col_left + col_w + 48.0 {
                margin.push(strength);
            }
        }
    }
    (page, margin)
}

/// The under-page major-section veil is present but materially quieter than
/// the same projected family in open margin. Percentiles over known landmarks
/// avoid grading accidental line intersections as the entire treatment.
#[test]
pub(super) fn projected_page_veil_is_present_and_quiet() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for (w, h, col_left, col_w, dpi, axis) in [
        (1200, 800, 312.0, 576.0, 1.0, (0.80, 0.24)),
        (1200, 800, 312.0, 576.0, 2.0, (0.50, 0.50)),
        (1600, 1000, 418.0, 763.2, 1.0, (0.20, 0.76)),
    ] {
        let pixels = field(&device, &queue, w, h, col_left, col_w, dpi, axis);
        let p = projection(w, h, axis);
        let (mut page, mut margin) = page_and_margin_strengths(&pixels, p, col_left, col_w);
        assert!(page.len() >= 20 && margin.len() >= 20);
        let page_p90 = percentile(&mut page, 90);
        let margin_p90 = percentile(&mut margin, 90);
        assert!(
            page_p90 > 0 && page_p90 * 3 <= margin_p90 * 2,
            "{w}x{h}@{dpi}: projected page p90 {page_p90}, open margin p90 {margin_p90}"
        );

        let mut unmasked = pixels.clone();
        let x0 = (col_left * dpi).round() as u32;
        let x1 = ((col_left + col_w) * dpi).round() as u32;
        for y in 0..unmasked.physical_h {
            for x in x0..x1.min(unmasked.physical_w) {
                unmasked.px[(y * unmasked.physical_w + x) as usize] = 255;
            }
        }
        let (mut loud_page, mut same_margin) =
            page_and_margin_strengths(&unmasked, p, col_left, col_w);
        assert!(
            percentile(&mut loud_page, 90) * 3 > percentile(&mut same_margin, 90) * 2,
            "page-quiet detector accepted a deliberately unmasked page"
        );
    }
}
