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

/// The room's own anchor radius, in pixels, at the canonical `H` — the
/// depth band the fold is authored (and tapered) around. Mirrors
/// `WARP_SECTION_ROOM_FRAC` in `shaders/background.wgsl`.
const ANCHOR_PX: f32 = 0.432 * H as f32;

/// Mirrors of the shader's own ring-spacing formula (`rpo`/`core` in
/// `warped_grid_rgba`), duplicated here the way `warp_tunnel.rs` already
/// duplicates them for its OWN room-centred scan — so a ring's radius can be
/// checked against the CLOSED FORM directly. That indirection is load-
/// bearing here specifically: a naive "same raw radius from one point on
/// both flanks" comparison is not merely a narrow search range, it is
/// GEOMETRICALLY UNSATISFIABLE for a corner-resting axis. With the axis at
/// `VpCorner::TopLeft` (320, 240) on the canonical 1600x1000 canvas, the
/// farthest on-canvas point with `x < COL_LEFT` is the far corner of the
/// left-margin rectangle at ~824px, while the NEAREST on-canvas point with
/// `x > COL_LEFT + COL_W` is ~954px away — the two flanks' reachable-radius
/// windows are disjoint, so no `r` exists that reaches both, independent of
/// anything the shader draws. The closed form sidesteps this: each flank is
/// searched over its OWN full reachable range independently, and what is
/// compared is whether the strongest ring found on EACH flank lands on an
/// integer level set of the SAME single-axis formula — the thing a genuine
/// "two cameras" defect (a second, differently-placed axis for one flank)
/// could not fake, because computing depth from the wrong point does not
/// land on this formula's integers.
const RING_PITCH_AT: f32 = 0.8333333;
const RPO_MIN: f32 = 3.0;
const RPO_MAX: f32 = 20.0;
const CORE_FRAC: f32 = 0.055;
/// Mirrors `WARP_FOLD_TAPER_LO` — the depth (in `|depth0 - travel|` units)
/// within which the shader's own `fold_taper` stays at full strength.
const FOLD_TAPER_LO: f32 = 3.0;

fn rpo_for(bg: theme::Background) -> f32 {
    let spacing = match bg {
        theme::Background::WarpedGrid { spacing_px, .. } => spacing_px,
        _ => unreachable!("only WarpedGrid worlds reach this law"),
    };
    (RING_PITCH_AT * ANCHOR_PX * std::f32::consts::LN_2 / spacing).clamp(RPO_MIN, RPO_MAX)
}

/// The fold-free, zero-travel `depth0` at radius `r` from the axis —
/// `rpo * log2(anchor / u)` with the SAME `u = max(r, core)` clamp
/// `warped_grid_rgba` applies. A ring (major or minor) is drawn wherever
/// this is arbitrarily close to an integer.
fn depth0_at(r: f32, rpo: f32) -> f32 {
    let core = CORE_FRAC * ANCHOR_PX;
    rpo * (ANCHOR_PX / r.max(core)).log2()
}

/// A radius below which individual rings are not expected to read as
/// distinct marks at all, so a candidate pair there is not a signal this law
/// can grade regardless of axis correctness — not a tuned tolerance on this
/// law's own numbers. The shader deliberately BLURS the lattice together
/// once its projected spacing nears `WARP_ALIAS_FADE_HI_PX` (9 logical px),
/// to keep a converging grid off a rasteriser's own moire threshold; ring
/// spacing near a given radius is `~= r * ln2 / rpo`, so this solves that
/// for `r` at a comfortable multiple (20px) of the shader's own floor.
fn min_gradeable_radius(rpo: f32) -> f32 {
    20.0 * rpo / std::f32::consts::LN_2
}

/// The average on-flank ink at exactly one radius (`want_left` selects the
/// flank): every angle whose `(x, y)` lands on that flank contributes, so a
/// genuine RING (fold: 0.0 makes it an exact circle, lighting up EVERY angle
/// simultaneously at its own radius) reads as a real elevation, while a RAIL
/// (a radial spoke, lighting up only the one angle it runs along) is diluted
/// by the hundreds of other angles that see none of it. `None` when too
/// little of the flank is reachable at this radius to average meaningfully.
fn avg_ink_at_radius(f: &[i32], axis: (f32, f32), r: f32, want_left: bool) -> Option<f32> {
    let mut vals: Vec<f32> = Vec::new();
    for i in 0..720 {
        let theta = std::f32::consts::TAU * i as f32 / 720.0;
        let x = axis.0 + r * theta.cos();
        let y = axis.1 + r * theta.sin();
        let on_side = if want_left { x < COL_LEFT } else { x > COL_LEFT + COL_W };
        if !on_side {
            continue;
        }
        let v = ink_at(f, W, H, x, y);
        if v.is_nan() {
            continue;
        }
        vals.push(v);
    }
    // MEDIAN, not mean: a RAIL (a radial spoke) contaminates only the
    // handful of angles it runs along, and a mean lets those few
    // high-value samples drag the whole estimate; the median of a
    // genuinely elevated RING (which lights up almost every angle at its
    // own radius) is unaffected by a few outliers either way. The floor is
    // 40, not 100: a flank near the far edge of its own reachable range
    // (e.g. the flank OPPOSITE a corner-resting axis) can top out around
    // 90-95 on-flank samples at ANY radius, and 100 silently threw away
    // every one of those genuinely gradeable rings.
    if vals.len() < 40 {
        return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(vals[vals.len() / 2])
}

/// The inverse of [`depth0_at`]: the radius at which the fold-free, zero-
/// travel model reports exactly `depth0`.
fn radius_for_depth0(depth0: f32, rpo: f32) -> f32 {
    ANCHOR_PX * 2f32.powf(-depth0 / rpo)
}

/// PREDICT THEN VERIFY, rather than detect-then-explain: blindly hunting for
/// local maxima in a noisy per-radius average and then asking whether any of
/// them happens to be near an integer is close to VACUOUS — with enough
/// spurious bumps (partial-arc sampling noise, a residual rail, an
/// intersection), a generous per-peak tolerance is satisfied almost
/// regardless of the axis (measured: a deliberately WRONG axis matched a
/// bump to within 0.024 of an integer purely by chance among 17 candidates).
/// This instead uses the closed form to name where a ring and its
/// neighbouring TROUGH (exactly half a ring apart) OUGHT to be, and checks
/// that real pixels bear that out: a genuine ring reads as a clear
/// peak-over-trough there; a wrong axis's predicted pair does not correlate
/// with the real ring/trough structure, so the ratio collapses toward 1.
/// Returns `(checked, confirmed)` over every integer ring index whose
/// predicted radius (and trough) both fall on the measurable flank.
fn ring_hit_rate(
    f: &[i32],
    axis: (f32, f32),
    rpo: f32,
    r_lo: f32,
    r_hi: f32,
    want_left: bool,
) -> (usize, usize) {
    // depth0 DECREASES as r increases, so the smallest r carries the
    // largest depth0.
    let d_hi = depth0_at(r_lo, rpo);
    let d_lo = depth0_at(r_hi, rpo);
    let mut checked = 0usize;
    let mut confirmed = 0usize;
    let mut n = d_lo.floor() as i32;
    while n as f32 <= d_hi.ceil() {
        let r_ring = radius_for_depth0(n as f32, rpo);
        let r_trough = radius_for_depth0(n as f32 + 0.5, rpo);
        let pair = (
            avg_ink_at_radius(f, axis, r_ring, want_left),
            avg_ink_at_radius(f, axis, r_trough, want_left),
        );
        if let (Some(ring_v), Some(trough_v)) = pair {
            checked += 1;
            if std::env::var("AWL_DEBUG_RING_HIT").is_ok() {
                eprintln!(
                    "n={n} r_ring={r_ring:.1} ring_v={ring_v:.2} r_trough={r_trough:.1} trough_v={trough_v:.2}"
                );
            }
            if ring_v > INK_FLOOR as f32 && ring_v > trough_v * 1.5 {
                confirmed += 1;
            }
        }
        n += 1;
    }
    (checked, confirmed)
}

/// ONE AXIS, EVERY ROAM STATE: on EACH flank, a strong majority of the
/// single-axis closed form's own predicted ring/trough pairs must read as
/// real peak-over-trough in the rendered pixels — proven at rest in each
/// corner and, load-bearingly, at a genuine mid-transit blend. Tested
/// against the fold-free profile (`with_fold(kite(), 0.0)`) to isolate the
/// axis/projection machinery from the fold's own angular shape, the same
/// split `warp_tunnel.rs` and `warp_one_tunnel.rs` already draw for their
/// own axis claims. See [`ring_hit_rate`]'s own doc for why this predicts
/// then verifies rather than detecting peaks and asking whether any happen
/// to match — the mutation proof
/// (`mutation_proof_a_wrong_axis_fails_the_level_set_check`) measured the
/// earlier "any peak within a loose tolerance" shape as satisfiable by a
/// WRONG axis purely by chance.
#[test]
fn one_axis_holds_under_every_roam_state_including_mid_transit() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let bg = with_fold(kite(), 0.0);
    let rpo = rpo_for(bg);
    let max_r = ((W * W + H * H) as f64).sqrt() as f32;
    for (name, axis_frac) in sample_poses() {
        let f = roam_field(&device, &queue, bg, 0.0, axis_frac);
        let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
        for (side, want_left) in [("left", true), ("right", false)] {
            let (checked, confirmed) = ring_hit_rate(&f, axis, rpo, min_gradeable_radius(rpo), max_r, want_left);
            assert!(
                checked >= 3,
                "{name} {side}: too few of the closed form's own predicted ring/trough pairs \
                 fell on the measurable part of that flank about the resolved axis {axis:?} \
                 ({checked} checked) — not enough of the flank to grade"
            );
            assert!(
                confirmed * 10 >= checked * 7,
                "{name} {side}: only {confirmed}/{checked} of the shared single-axis field's \
                 own predicted ring/trough pairs read as a real peak-over-trough about the \
                 resolved axis {axis:?} — the two flanks are not windows onto the same tube"
            );
        }
    }
}

/// THE FOLD READS AS THE WALL'S SURFACE, NOT A FLAT OVERLAY: at the shipped
/// fold amplitude, the SAME nominal ring's radius genuinely varies with
/// angle (the wall bulges and pulls in) — the differential proof is that a
/// `fold: 0.0` reference (a perfect circle) draws its strongest ring at
/// materially the SAME radius at every angle, while the authored profile
/// does not. Sampled in the depth band the fold is actually tapered around
/// (near the room's own anchor radius — see `WARP_FOLD_TAPER_LO/HI` in the
/// shader): far outside that band the fold is DELIBERATELY inert (the
/// taper that keeps a fold-induced caustic out of the open margin), so
/// sampling there would test the taper, not the fold.
#[test]
fn fold_amplitude_makes_the_ring_radius_a_function_of_angle() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let axis_frac = warpgrid::VpCorner::TopLeft.frac();
    let axis = (axis_frac.0 * W as f32, axis_frac.1 * H as f32);
    // THE INITIAL SEARCH MUST LAND INSIDE THE TAPER'S FULL-STRENGTH BAND, not
    // merely somewhere in a wide window around the anchor. `WARP_FOLD_TAPER_LO`
    // (mirrored below) is where the shader's own `fold_taper` STARTS falling
    // off `abs(depth0 - travel)` away from the ring the camera is level with;
    // a window as wide as `anchor * [0.65, 1.35]` spans several rings on
    // EITHER side of that band, and the single strongest peak in it can be
    // one already tapered to a quarter strength (measured: at `fold_taper`'s
    // own `abs(depth0)==5`, `fold_eff` is already down to ~26% of authored
    // `fold` — nearly a flat overlay by construction, not by a broken
    // formula). Deriving the window from the SAME closed form
    // (`radius_for_depth0`/`rpo_for`) this file already carries for the
    // axis law keeps it a property of the shader's own taper, not a second
    // hand-picked range.
    let rpo = rpo_for(kite());
    let search_lo = radius_for_depth0(FOLD_TAPER_LO, rpo);
    let search_hi = radius_for_depth0(-FOLD_TAPER_LO, rpo);
    // A NARROW window straddling the same tracked ring at every angle — a
    // broad open search (as a first draft of this law used) can jump onto a
    // DIFFERENT ring at different angles even for a perfectly circular
    // reference, which measures the ring LADDER's own spacing rather than
    // any one ring's shape. The tracked radius starts from theta=0's own
    // strongest peak in a wide first pass, then every subsequent angle
    // searches only a small band around the PREVIOUS angle's found radius —
    // the ring is continuous, so this cannot skip to a neighbour. STEPS=192
    // and a +/-6px window (calibrated against this exact axis/profile, not
    // guessed): a wider window or coarser angular step both let a single
    // rail-intersection blip survive into the tracked list and inflate the
    // FOLD-FREE reference's own stdev to within noise of the folded one —
    // measured up to 3.26px of pure tracking noise at a +/-15px window,
    // against a real fold-driven signal of ~4.7-5.3px regardless of these
    // settings; this configuration pushes the noise floor down to ~0.7px
    // while leaving the genuine signal essentially unchanged.
    const STEPS: u32 = 192;
    const WINDOW_PX: f32 = 6.0;
    let track = |bg: theme::Background| -> Vec<f32> {
        let f = roam_field(&device, &queue, bg, 0.0, axis_frac);
        let peak_in = |theta: f32, r_lo: f32, r_hi: f32| -> Option<f32> {
            let mut best = (f32::MIN, 0.0f32);
            let mut r = r_lo;
            while r < r_hi {
                let x = axis.0 + r * theta.cos();
                let y = axis.1 + r * theta.sin();
                let v = ink_at(&f, W, H, x, y);
                if !v.is_nan() && v > best.0 {
                    best = (v, r);
                }
                r += 0.5;
            }
            (best.0 > INK_FLOOR as f32).then_some(best.1)
        };
        let mut current = match peak_in(0.0, search_lo, search_hi) {
            Some(r) => r,
            None => return Vec::new(),
        };
        let mut radii = vec![current];
        for i in 1..STEPS {
            let theta = std::f32::consts::TAU * i as f32 / STEPS as f32;
            match peak_in(theta, current - WINDOW_PX, current + WINDOW_PX) {
                Some(r) => {
                    radii.push(r);
                    current = r;
                }
                None => return radii, // lost the ring (e.g. under the page) — use what we have
            }
        }
        radii
    };
    let spread = |radii: &[f32]| -> f32 {
        if radii.len() < (STEPS / 4) as usize {
            return 0.0;
        }
        let mean = radii.iter().sum::<f32>() / radii.len() as f32;
        (radii.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / radii.len() as f32).sqrt()
    };
    let circular = spread(&track(with_fold(kite(), 0.0)));
    let folded = spread(&track(kite()));
    assert!(
        folded > circular + 3.0,
        "fold must make the ring radius vary with angle near the anchor depth (folded stdev \
         {folded:.2}px vs circular {circular:.2}px, tracked continuously) — otherwise it \
         reads as a flat overlay, not a wall"
    );
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

/// MUTATION PROOF (temporary, see the build report): this law's own
/// `frac < 0.2` threshold is not vacuously satisfiable — checking a REAL
/// rendered field's left flank against a WRONG axis (the axis a genuine
/// "two cameras" bug would leave the OTHER flank computed from) must not
/// find a matching level set. Demonstrates the law has teeth without
/// requiring an actual shader regression.
#[test]
fn mutation_proof_a_wrong_axis_fails_the_level_set_check() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let bg = with_fold(kite(), 0.0);
    let rpo = rpo_for(bg);
    let max_r = ((W * W + H * H) as f64).sqrt() as f32;
    let true_axis_frac = warpgrid::VpCorner::TopLeft.frac();
    let f = roam_field(&device, &queue, bg, 0.0, true_axis_frac);
    let true_axis = (true_axis_frac.0 * W as f32, true_axis_frac.1 * H as f32);
    let (checked, confirmed) = ring_hit_rate(&f, true_axis, rpo, min_gradeable_radius(rpo), max_r, true);
    eprintln!("correct left flank: {confirmed}/{checked} predicted rings confirmed");
    assert!(
        checked >= 3 && confirmed * 10 >= checked * 7,
        "the CORRECT axis itself failed its own hit-rate floor ({confirmed}/{checked}) — the \
         law's model or its confirmation ratio is miscalibrated, not just its mutation arm"
    );
    let wrong_axis_frac = warpgrid::VpCorner::TopRight.frac();
    let wrong_axis = (wrong_axis_frac.0 * W as f32, wrong_axis_frac.1 * H as f32);
    let (w_checked, w_confirmed) = ring_hit_rate(&f, wrong_axis, rpo, min_gradeable_radius(rpo), max_r, true);
    eprintln!("wrong left flank:   {w_confirmed}/{w_checked} predicted rings confirmed");
    assert!(
        w_checked < 3 || w_confirmed * 10 < w_checked * 7,
        "a deliberately wrong axis ({wrong_axis:?} instead of {true_axis:?}) unexpectedly hit \
         this law's own confirmation floor ({w_confirmed}/{w_checked}) — this law would not \
         catch a real 'two cameras' regression"
    );
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

    // SHAPE: the fold still reads as the wall's surface at this exact pose —
    // same tracking technique as `fold_amplitude_makes_the_ring_radius_a_function_of_angle`,
    // just at the calm axis/travel instead of a roamed corner.
    let axis = (calm_axis_frac.0 * W as f32, calm_axis_frac.1 * H as f32);
    let rpo = rpo_for(kite());
    let search_lo = radius_for_depth0(FOLD_TAPER_LO, rpo);
    let search_hi = radius_for_depth0(-FOLD_TAPER_LO, rpo);
    const STEPS: u32 = 192;
    const WINDOW_PX: f32 = 6.0;
    let track = |bg: theme::Background| -> Vec<f32> {
        let f = roam_field(&device, &queue, bg, 0.0, calm_axis_frac);
        let peak_in = |theta: f32, r_lo: f32, r_hi: f32| -> Option<f32> {
            let mut best = (f32::MIN, 0.0f32);
            let mut r = r_lo;
            while r < r_hi {
                let x = axis.0 + r * theta.cos();
                let y = axis.1 + r * theta.sin();
                let v = ink_at(&f, W, H, x, y);
                if !v.is_nan() && v > best.0 {
                    best = (v, r);
                }
                r += 0.5;
            }
            (best.0 > INK_FLOOR as f32).then_some(best.1)
        };
        // Start pointing back toward the canvas interior (PI), not 0: unlike
        // `fold_amplitude_makes_the_ring_radius_a_function_of_angle`'s
        // TopLeft axis (where theta=0 heads toward the page), the calm
        // pose's axis sits just inside the RIGHT margin, so theta=0 exits
        // the canvas immediately and finds nothing to track. And SUBTRACT
        // the sweep (rather than add, as the TopLeft law does from its own
        // theta=0) so the first steps head toward the room's spacious lower
        // half rather than its cramped upper edge: TopRight's axis sits the
        // SAME 240px from the top as TopLeft's does, but `sin(pi + eps) < 0`
        // while `sin(eps) > 0`, so mirroring the increment's SIGN — not just
        // offsetting its start — is what keeps the two laws' sweeps
        // direction-equivalent relative to their own geometry.
        let mut current = match peak_in(std::f32::consts::PI, search_lo, search_hi) {
            Some(r) => r,
            None => return Vec::new(),
        };
        let mut radii = vec![current];
        for i in 1..STEPS {
            let theta = std::f32::consts::PI - std::f32::consts::TAU * i as f32 / STEPS as f32;
            match peak_in(theta, current - WINDOW_PX, current + WINDOW_PX) {
                Some(r) => {
                    radii.push(r);
                    current = r;
                }
                None => return radii,
            }
        }
        radii
    };
    let spread = |radii: &[f32]| -> f32 {
        if radii.len() < (STEPS / 4) as usize {
            return 0.0;
        }
        let mean = radii.iter().sum::<f32>() / radii.len() as f32;
        (radii.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / radii.len() as f32).sqrt()
    };
    let circular = spread(&track(with_fold(kite(), 0.0)));
    let folded = spread(&track(kite()));
    assert!(
        folded > circular + 3.0,
        "the motion-safe pose's own ring radius must still vary with angle (folded stdev \
         {folded:.2}px vs circular {circular:.2}px) — otherwise the calm pose reads as a flat \
         overlay, not the same folded tube"
    );
}

