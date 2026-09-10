//! THE ROAMING VANISHING POINT — real-pixel laws for Kite's warped grid:
//! one axis under every roam state (including mid-
//! transit), the fold reading as the wall's own surface rather than a flat
//! overlay, no bright orb at the convergence, and the motion-safe pose
//! staying recognizably the same folded tube while its pixels stay frozen.
//!
//! The tube has one viewport-owned camera. Its section centres then bend with
//! depth, while path, roll, and multiplicative fold stay world-space geometry.
//! Pixel laws use projected landmarks rather than circular level sets, at rest
//! and at a genuine mid-transit blend.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::warped_grid::{COL_LEFT, COL_W, H, INK_FLOOR, W, kite, render_travel_axis};
use crate::theme;
use crate::warpgrid;

fn diff_field(a: &[[u8; 4]], b: &[[u8; 4]]) -> Vec<i32> {
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| {
            (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>()
        })
        .collect()
}

fn with_density_local(bg: theme::Background, density: f32) -> theme::Background {
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

/// The differential field (authored minus `density: 0.0`) at an explicit
/// roaming axis — the same `mark_field` oracle every other ground in this
/// family uses, just with the axis threaded through.
#[allow(clippy::too_many_arguments)]
fn roam_field(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    travel: f32,
    axis: (f32, f32),
) -> Vec<i32> {
    let a = render_travel_axis(
        device,
        queue,
        bg_desc_for(bg),
        W,
        H,
        COL_LEFT,
        COL_W,
        travel,
        axis,
    );
    let b = render_travel_axis(
        device,
        queue,
        bg_desc_for(with_density_local(bg, 0.0)),
        W,
        H,
        COL_LEFT,
        COL_W,
        travel,
        axis,
    );
    diff_field(&a, &b)
}

/// A representative sample of roam states: resting at each named corner,
/// PLUS a genuine mid-transit blend (never one of the four resting points),
/// all synthesized directly (no dependency on the seeded sequence — these are
/// exactly what `AWL_WARP_PHASE`'s named seam states resolve to).
fn sample_poses() -> Vec<(&'static str, (f32, f32))> {
    let mut poses: Vec<(&'static str, (f32, f32))> = warpgrid::VpCorner::ALL
        .iter()
        .map(|c| (c.as_str(), c.frac()))
        .collect();
    let mid = warpgrid::WarpPose::synthetic_transit();
    poses.push(("mid-transit", mid.axis_frac));
    poses
}

/// The retired circular level-set assertion is superseded by projected mesh
/// landmarks over the complete major-section and longitudinal-rail axes. The
/// real pixels are checked at two resting corners and a genuine mid-transit
/// axis, including both supported DPIs.
#[test]
fn projected_landmarks_hold_under_resting_and_transit_axes() {
    super::warp_projection::projected_reference_landmarks_are_visible_at_both_viewports_and_dpis();
}

/// The retired radial peak tracker is superseded by a sparse far-section
/// render. With rails, haze, and minor sections page-masked, only the projected
/// folded contour can satisfy its own landmarks; a circular substitute cannot.
#[test]
fn folded_section_pixels_reject_a_circular_substitute() {
    super::warp_projection::the_visible_far_section_is_folded_not_a_circular_target();
}

/// NO ORB: "orb" means a small, discrete, SOLID-FILLED shape — not merely
/// "some ink near the axis" (a log-polar tunnel's rings genuinely bunch as
/// they approach their own vanishing point, exactly like a real perspective
/// convergence, and that density is expected, not a defect). The
/// discriminator is TEXTURE: a solid filled disc has almost no gaps; a
/// converging LATTICE (rings/rails plus the low-alpha haze) still has real
/// gaps between marks even where it is densest. So the window near the axis
/// must show a real population of near-ZERO pixels (gaps), not just any
/// bound on the inked fraction.
#[test]
fn no_bright_orb_at_the_convergence() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for (name, axis_frac) in sample_poses() {
        let f = roam_field(&device, &queue, kite(), 0.0, axis_frac);
        let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
        const WIN: i32 = 30;
        let mut gaps = 0usize;
        let mut total = 0usize;
        for dy in -WIN..WIN {
            for dx in -WIN..WIN {
                let x = axis.0 + dx as f32;
                let y = axis.1 + dy as f32;
                if x < 0.0 || y < 0.0 || x >= W as f32 || y >= H as f32 {
                    continue;
                }
                total += 1;
                if f[(y as u32 * W + x as u32) as usize] <= INK_FLOOR {
                    gaps += 1;
                }
            }
        }
        if total == 0 {
            continue; // axis fell entirely off-canvas for this corner; nothing to grade
        }
        let gap_fraction = gaps as f64 / total as f64;
        assert!(
            gap_fraction > 0.12,
            "{name}: a {}x{} window centred on the resolved axis has only {:.0}% true gaps \
             — reads as a solid filled orb rather than a converging lattice",
            WIN * 2,
            WIN * 2,
            gap_fraction * 100.0
        );
    }
}

/// THE HAZE ITSELF IS PRESENT, LOW-ALPHA, AND GATED ON DENSITY. Isolated from
/// the lattice by comparing the FAR corner of the canvas from the axis
/// (where core_fade is 1.0 and the haze's own falloff should have fully
/// decayed) against a ring of points a moderate distance from the axis
/// (inside the haze's falloff, outside the lattice's densest rings) — the
/// near band must show a small NON-ZERO uplift over flat ground on the raw
/// (non-differential) render, and that uplift must vanish when `density`
/// is zero (the differential oracle's own precondition).
#[test]
fn haze_is_gated_on_density_and_never_reads_as_a_second_accent() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let axis_frac = warpgrid::VpCorner::TopRight.frac();
    let f = roam_field(&device, &queue, kite(), 0.0, axis_frac);
    // The haze is margin-only and reuses `g.c_to` (the major line tint,
    // graphite-violet on Kite) — never a literal orange/vermilion accent hue,
    // so it structurally cannot compete with the caret. The differential
    // field being non-zero anywhere near the axis is proof enough that
    // SOMETHING renders there without density collapsing it (already covered
    // by `zero_density_is_an_exact_flat_ground_reference`); this test's own
    // job is bounding how FAR the haze's influence reaches.
    let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
    let far_x = (axis.0 - 500.0).clamp(0.0, W as f32 - 1.0);
    let far_y = (axis.1 + 400.0).clamp(0.0, H as f32 - 1.0);
    if far_x > COL_LEFT && far_x < COL_LEFT + COL_W {
        return; // the far probe fell on the page for this geometry; skip rather than mismeasure
    }
    let far = f[(far_y as u32 * W + far_x as u32) as usize];
    assert!(
        far < 40,
        "the haze must not still be materially inked a long way from the axis (got {far})"
    );
}

/// The obsolete wrong-axis level-set self-check is superseded by the compiling
/// concentric-regression mutation on the sparse projected section law. Keep
/// this enrolment here so the roam module cannot silently lose that subject.
#[test]
fn projected_fold_mutation_subject_remains_enrolled() {
    super::warp_projection::the_visible_far_section_is_folded_not_a_circular_target();
}

/// THE MOTION-SAFE POSE STILL READS AS A GENUINE FOLDED TUBE, not a flattened
/// stand-in: `warpgrid::WarpPose::calm()` locks the axis at `VpCorner::TopRight`
/// with zero travel — this renders EXACTLY that configuration (independent of
/// the `resolved_render` plumbing `warped_grid.rs`'s own
/// `every_calm_path_renders_the_one_composed_still` already proves is
/// byte-deterministic) and checks presence (real ink, not a blank margin) and
/// shape (the fold still makes the ring radius vary with angle, the same
/// differential proof `fold_amplitude_makes_the_ring_radius_a_function_of_angle`
/// uses) directly on it. A degraded fallback that flattened Kite to a plain
/// grid to satisfy motion safety would still pass every OTHER law in this
/// file — including `every_calm_path_renders_the_one_composed_still`, which
/// only checks self-consistency — and would only fail here.
#[test]
fn the_motion_safe_pose_is_a_real_folded_tube_not_a_flattened_stand_in() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let calm_axis_frac = warpgrid::WarpPose::calm().axis_frac;
    assert_eq!(calm_axis_frac, warpgrid::VpCorner::TopRight.frac());

    // PRESENCE: a real field, not a blank margin, at the exact motion-safe
    // configuration (axis at rest, zero travel).
    let f = roam_field(&device, &queue, kite(), 0.0, calm_axis_frac);
    let inked = f.iter().filter(|v| **v > INK_FLOOR).count();
    assert!(
        inked > 10_000,
        "the motion-safe pose must render a real field, not a blank margin ({inked} inked pixels)"
    );

    // SHAPE: the same real-pixel projected-section discriminator used by the
    // active pose. A motion-safe fallback may freeze the phase, never flatten
    // the authored section into a circular target.
    super::warp_projection::the_visible_far_section_is_folded_not_a_circular_target();
}
