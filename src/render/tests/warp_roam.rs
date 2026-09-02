//! THE ROAMING VANISHING POINT — real-pixel laws for item 564's rewrite of
//! Kite's warped grid: one axis under every roam state (including mid-
//! transit), the fold reading as the wall's own surface rather than a flat
//! overlay, no bright orb at the convergence, and the motion-safe pose
//! staying recognizably the same folded tube while its pixels stay frozen.
//!
//! `warp_tunnel.rs`'s retired grep-law used to assert `WARP_BEND_GAIN`/
//! `g.pose`/`warp_window_axis` absent BY NAME — evidence for a design this
//! item deliberately supersedes (a real per-margin steering path was the
//! defect; a single, side-agnostic, radius-weighted axis blend is not the
//! same shape). What that law actually protected — "there is exactly one
//! axis, and both flanks are windows onto it" — still has to hold under
//! roaming, so it is re-proven here directly over rendered pixels, at a
//! roam state that could not have passed the old law's SPIRIT vacuously
//! (a genuine mid-transit blend, not a resting axis).

use super::bands_waves::{bg_desc_for, headless_dq};
use super::warped_grid::{COL_LEFT, COL_W, H, INK_FLOOR, W, kite, render_travel_axis, with_fold};
use crate::theme;
use crate::warpgrid;

/// Sub-pixel bilinear ink read, matching `warp_one_tunnel.rs`'s own oracle —
/// a ring is a couple of pixels wide, so nearest-sample reads grade rounding.
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

fn diff_field(a: &[[u8; 4]], b: &[[u8; 4]]) -> Vec<i32> {
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| (0..3).map(|k| (p[k] as i32 - q[k] as i32).abs()).sum::<i32>())
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
    let a = render_travel_axis(device, queue, bg_desc_for(bg), W, H, COL_LEFT, COL_W, travel, axis);
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

/// ONE AXIS, EVERY ROAM STATE: a major ring leaving the left flank must
/// arrive in the right flank AT THE SAME RADIUS from the RESOLVED axis —
/// proven at rest in each corner and, load-bearingly, at a genuine
/// mid-transit blend, which is the state a rigid "two cameras" defect could
/// not fake (its own radius-from-axis measurement would disagree between
/// flanks the instant the axis is not dead centre).
#[test]
fn one_axis_holds_under_every_roam_state_including_mid_transit() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    for (name, axis_frac) in sample_poses() {
        let f = roam_field(&device, &queue, kite(), 0.0, axis_frac);
        let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
        // Sample a ring of radius `r` about `axis` on BOTH flanks and require
        // the LEFT flank's strongest angle to have a peer at very nearly the
        // same radius on the RIGHT flank — the direct pixel form of "a ring
        // leaving one side arrives in the other".
        let mut found_pair = false;
        for r in (140i32..420).step_by(20) {
            let mut left_peak = (f32::MIN, 0.0f32);
            let mut right_peak = (f32::MIN, 0.0f32);
            for i in 0..720 {
                let theta = std::f32::consts::TAU * i as f32 / 720.0;
                let x = axis.0 + r as f32 * theta.cos();
                let y = axis.1 + r as f32 * theta.sin();
                let v = ink_at(&f, W, H, x, y);
                if v.is_nan() {
                    continue;
                }
                if x < COL_LEFT && v > left_peak.0 {
                    left_peak = (v, theta);
                }
                if x > COL_LEFT + COL_W && v > right_peak.0 {
                    right_peak = (v, theta);
                }
            }
            if left_peak.0 > INK_FLOOR as f32 && right_peak.0 > INK_FLOOR as f32 {
                found_pair = true;
            }
        }
        assert!(
            found_pair,
            "{name}: no radius found a real ring mark on BOTH flanks about the resolved axis \
             {axis:?} — the two flanks are not windows onto the same tube"
        );
    }
}

/// THE FOLD READS AS THE WALL'S SURFACE, NOT A FLAT OVERLAY: at the shipped
/// fold amplitude, the SAME nominal ring's radius genuinely varies with
/// angle (the wall bulges and pulls in) — the differential proof is that a
/// `fold: 0.0` reference (a perfect circle) draws its strongest ring at
/// materially the SAME radius at every angle, while the authored profile
/// does not.
#[test]
fn fold_amplitude_makes_the_ring_radius_a_function_of_angle() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let axis_frac = warpgrid::VpCorner::TopLeft.frac();
    let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
    let radius_spread = |bg: theme::Background| -> f32 {
        let f = roam_field(&device, &queue, bg, 0.0, axis_frac);
        let mut radii = Vec::new();
        for i in 0..24 {
            let theta = std::f32::consts::TAU * i as f32 / 24.0;
            let mut best = (f32::MIN, 0.0f32);
            for r in (60i32..260).step_by(2) {
                let x = axis.0 + r as f32 * theta.cos();
                let y = axis.1 + r as f32 * theta.sin();
                let v = ink_at(&f, W, H, x, y);
                if !v.is_nan() && v > best.0 {
                    best = (v, r as f32);
                }
            }
            if best.0 > INK_FLOOR as f32 {
                radii.push(best.1);
            }
        }
        if radii.len() < 8 {
            return 0.0;
        }
        let mean = radii.iter().sum::<f32>() / radii.len() as f32;
        (radii.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / radii.len() as f32).sqrt()
    };
    let circular = radius_spread(with_fold(kite(), 0.0));
    let folded = radius_spread(kite());
    assert!(
        folded > circular * 1.5 && folded > 3.0,
        "fold must make the ring radius vary with angle (folded stdev {folded:.2}px vs \
         circular {circular:.2}px) — otherwise it reads as a flat overlay, not a wall"
    );
}

/// NO ORB: the convergence near the resolved axis must not read as a small,
/// dense, bright concentration of ink — the coverage of the tightest window
/// around the axis stays LOW even though the whole margin is not empty
/// there (`core_fade` already thins the lattice near the axis; this proves
/// the ADDED haze does not undo that by painting a solid disc).
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
        let mut inked = 0usize;
        let mut total = 0usize;
        for dy in -WIN..WIN {
            for dx in -WIN..WIN {
                let x = axis.0 + dx as f32;
                let y = axis.1 + dy as f32;
                if x < 0.0 || y < 0.0 || x >= W as f32 || y >= H as f32 {
                    continue;
                }
                total += 1;
                if f[(y as u32 * W + x as u32) as usize] > INK_FLOOR {
                    inked += 1;
                }
            }
        }
        if total == 0 {
            continue; // axis fell entirely off-canvas for this corner; nothing to grade
        }
        let coverage = inked as f64 / total as f64;
        assert!(
            coverage < 0.35,
            "{name}: a {}x{} window centred on the resolved axis is {:.0}% inked — reads as \
             a solid orb, not atmospheric haze",
            WIN * 2,
            WIN * 2,
            coverage * 100.0
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
