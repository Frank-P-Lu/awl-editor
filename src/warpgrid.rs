//! THE ROUTE — the deterministic camera journey the warped-grid ground travels.
//!
//! This module is the ONE owner of the route: the shader receives a finished
//! steering pose (yaw / pitch / forward travel) through the background uniform
//! and carries no route arithmetic of its own, so there is no host/GPU mirror
//! here to drift out of lockstep (the `lava.rs` / `dither.rs` mirrors exist
//! because the GPU is their only consumer; the route's consumer is this file).
//!
//! Everything is pure, so every promise the motion makes — continuous travel, a
//! long steady direction before each turn, zero velocity at every join, no
//! acceleration, no catch-up after a delayed wake, an invisible wrap — is a unit
//! test rather than a live impression.
//!
//! The route rides the SHARED ambient tick (`crate::lava::ambient_tick_dt`, the
//! App's single sparse `WaitUntil`), so it inherits the blur/move/resize pause,
//! the `ambient_motion` setting, the Reduce-Motion freeze and the headless
//! `t = 0` pin without any scheduling machinery of its own. It keeps its own
//! phase ACCUMULATOR only because its loop is minutes long where the lava lamp's
//! is seconds: one tick, two accumulators.

use std::sync::OnceLock;

/// One authored direction lasts this long before the ease into the next. It is
/// deliberately NOT 60: a route whose legs land on the minute reads as a clock,
/// and the item's own contract forbids an "obvious one-minute restart".
pub const ROUTE_LEG_SECONDS: f32 = 58.0;

/// How much of a leg is the STEADY hold, before the ease begins. The remainder
/// is the transition. Without a hold the pose eases continuously from one target
/// to the next and never reads as a *long direction* at all — it reads as a slow
/// wander.
pub const ROUTE_HOLD_FRAC: f32 = 0.55;

/// The authored directions, in order — `(yaw, pitch)` radians-ish steering, each
/// held for [`ROUTE_HOLD_FRAC`] of its leg and then eased into the next. Slot 0
/// is repeated implicitly as the wrap destination, so the loop closes on a
/// straight leg and the several-minute repeat carries no steering seam.
///
/// A bend is never followed directly by its opposite: every turn returns through
/// a straight leg first, so the journey has no hard reversal. The magnitudes are
/// deliberately unequal (`-0.74` against `+0.70`, `-0.52` against `+0.48`) so
/// the two halves of the route do not read as one mirrored figure.
const TARGETS: [(f32, f32); ROUTE_LEGS] = [
    (0.00, 0.00),   // straight
    (-0.74, 0.00),  // a long left bend
    (-0.18, -0.52), // climbing out of it
    (0.00, 0.00),   // straight again
    (0.70, 0.00),   // a long right bend
    (0.20, 0.48),   // descending out of it
    (0.00, 0.00),   // straight, into the wrap
];

pub const ROUTE_LEGS: usize = 7;

/// The whole deterministic route, in seconds — 7 min 6 s. Long enough that the
/// repeat is invisible in practice; the wrap is nonetheless made EXACT rather
/// than merely long (see [`FORWARD_CELLS_PER_LOOP`]).
pub const ROUTE_LOOP_SECONDS: f32 = ROUTE_LEG_SECONDS * ROUTE_LEGS as f32;

/// Minor grid cells of forward travel per complete route loop.
///
/// **It MUST be a multiple of [`MAJOR_EVERY`].** Forward travel enters the
/// shader as a pure SUBTRACTION from the ring coordinate, whose minor lattice
/// has period 1 but whose every-fifth-line MAJOR classification has period 5 —
/// so an integer that is not a multiple of five wraps the minor lines back onto
/// themselves while rotating which of them are major, and the loop endpoint
/// carries a visible one-cell hierarchy jump. 65 keeps both invariant, making
/// the wrap byte-exact.
pub const FORWARD_CELLS_PER_LOOP: f32 = 65.0;

/// Every fifth line is the strong one — the modulus [`FORWARD_CELLS_PER_LOOP`]
/// must respect. The GPU is its only runtime consumer (`WARP_MAJOR_EVERY` in
/// `shaders/background.wgsl`), so like the `lava`/`dither` shader mirrors this
/// exists for the laws alone and is held in lockstep by a grep-law.
#[cfg(test)]
pub const MAJOR_EVERY: f32 = 5.0;

/// The composed still: Reduce Motion, `ambient_motion` off, and every headless
/// capture render exactly this pose — a straight leg at the start of its hold,
/// so the settled frame is the world's most legible composition rather than
/// an arbitrary mid-turn.
pub const FROZEN_PHASE: f32 = 0.0;

/// A finished steering pose. The renderer uploads these three scalars and the
/// shader does no route arithmetic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoutePose {
    /// Left (negative) / right (positive) steering.
    pub yaw: f32,
    /// Climb (negative) / descent (positive) steering.
    pub pitch: f32,
    /// Forward travel, in minor grid cells. STRICTLY LINEAR in phase — the
    /// camera never accelerates, so no part of the journey can grab attention
    /// by speeding up.
    pub forward_cells: f32,
}

/// Hermite ease with zero velocity at both ends.
fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The pose at `phase_seconds` into the route. Pure and total: any finite input,
/// including a negative one, resolves through `rem_euclid`.
pub fn route_pose(phase_seconds: f32) -> RoutePose {
    let phase = phase_seconds.rem_euclid(ROUTE_LOOP_SECONDS);
    let leg_f = phase / ROUTE_LEG_SECONDS;
    let leg = (leg_f.floor() as usize).min(ROUTE_LEGS - 1);
    let within = leg_f - leg as f32;
    // The hold, then the ease. `within <= HOLD` is exactly the steady stretch.
    let t = smoothstep01((within - ROUTE_HOLD_FRAC) / (1.0 - ROUTE_HOLD_FRAC));
    let (yaw0, pitch0) = TARGETS[leg];
    let (yaw1, pitch1) = TARGETS[(leg + 1) % ROUTE_LEGS];
    RoutePose {
        yaw: yaw0 + (yaw1 - yaw0) * t,
        pitch: pitch0 + (pitch1 - pitch0) * t,
        forward_cells: phase / ROUTE_LOOP_SECONDS * FORWARD_CELLS_PER_LOOP,
    }
}

/// Advance the route by one BOUNDED ambient step, wrapping at the loop.
///
/// The bound is [`crate::lava::ambient_tick_dt`] — the same owner the lava lamp
/// uses — so a delayed macOS event-loop wake (a window drag, a stalled
/// compositor, a machine coming back from a long blur) can never replay the
/// accumulated wall time as one visible lurch. That is precisely the item's
/// "resumes without catch-up" promise, and it is a property of this function
/// rather than of the App's scheduling.
pub fn advance_phase(phase_seconds: f32, dt: f32) -> f32 {
    (phase_seconds + crate::lava::ambient_tick_dt(dt)).rem_euclid(ROUTE_LOOP_SECONDS)
}

/// The EFFECTIVE render phase — the same resolver shape as
/// `crate::lava::lava_phase_for`: the dev gallery override wins outright, else
/// Reduce Motion pins the composed still, else the App-driven stored phase
/// (which is [`FROZEN_PHASE`] in every headless capture, since nothing ticks
/// the clock there).
pub fn phase_for(stored: f32, reduced: bool, env: Option<f32>) -> f32 {
    match env {
        Some(e) => e.rem_euclid(ROUTE_LOOP_SECONDS),
        None if reduced => FROZEN_PHASE,
        None => stored,
    }
}

// --- The dev-only gallery knob (AWL_WARP_POSE=...) ---------------------------
//
// Mirrors `AWL_LAVA` / `AWL_STARS_PHASE` / `AWL_WAVES_PHASE` exactly: read ONCE
// at startup, memoized, a TOTAL no-op unless set. It exists because a headless
// `--screenshot` never ticks the clock, so without it no mid-route pose is
// reachable at all and the motion could not be proven over real pixels.
//
//   AWL_WARP_POSE=straight|left|climb|right|descent|wrap   (a named pose)
//   AWL_WARP_POSE=<seconds>                                (any raw phase)
//
// The named poses are the mid-HOLD of each leg — the steadiest point of that
// direction — not a leg boundary, where the pose is by construction identical
// to its neighbour's start.
fn parse_pose(raw: &str) -> Option<f32> {
    let mid_hold = |leg: usize| ROUTE_LEG_SECONDS * (leg as f32 + ROUTE_HOLD_FRAC * 0.5);
    match raw.trim().to_ascii_lowercase().as_str() {
        "straight" | "still" | "settled" => Some(FROZEN_PHASE),
        "left" => Some(mid_hold(1)),
        "climb" | "up" => Some(mid_hold(2)),
        "right" => Some(mid_hold(4)),
        "descent" | "descend" | "down" => Some(mid_hold(5)),
        "wrap" => Some(ROUTE_LOOP_SECONDS),
        other => {
            let p: f32 = other.parse().ok()?;
            p.is_finite().then_some(p)
        }
    }
}

/// One of the knob's own NAMED poses, resolved to a route phase — so a law
/// sweeping "every route pose" sweeps the same vocabulary `AWL_WARP_POSE`
/// offers, rather than inventing a second list that could drift from it.
#[cfg(test)]
pub fn named_pose(name: &str) -> Option<f32> {
    parse_pose(name)
}

/// `AWL_WARP_POSE`'s parsed value, or `None` (every normal + headless run).
pub fn env_phase() -> Option<f32> {
    static ONCE: OnceLock<Option<f32>> = OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_WARP_POSE")
            .ok()
            .as_deref()
            .and_then(parse_pose)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose_at(name: &str) -> RoutePose {
        route_pose(parse_pose(name).expect("named pose"))
    }

    /// Every named direction in the item's own vocabulary is genuinely reachable
    /// and reads as that direction — not merely a distinct number.
    #[test]
    fn every_named_direction_is_reachable_and_steers_the_way_it_is_named() {
        let straight = pose_at("straight");
        assert!(
            straight.yaw.abs() < 1e-6 && straight.pitch.abs() < 1e-6,
            "straight must be straight, got {straight:?}"
        );
        let left = pose_at("left");
        assert!(left.yaw < -0.5, "left must steer left, got {left:?}");
        assert!(left.pitch.abs() < 1e-6, "left must not climb, got {left:?}");
        let climb = pose_at("climb");
        assert!(climb.pitch < -0.4, "climb must climb, got {climb:?}");
        let right = pose_at("right");
        assert!(right.yaw > 0.5, "right must steer right, got {right:?}");
        let descent = pose_at("descent");
        assert!(descent.pitch > 0.4, "descent must descend, got {descent:?}");
        // Left and right are opposite SIDES, and climb/descent opposite signs —
        // the cross-margin coherence law downstream depends on this.
        assert!(left.yaw * right.yaw < 0.0);
        assert!(climb.pitch * descent.pitch < 0.0);
    }

    /// The visible contract is uninterrupted travel, so the loop endpoint must
    /// be indistinguishable from the start — in the POSE and, because forward
    /// travel is a multiple of the major modulus, in the GRID's own lattice and
    /// line hierarchy too.
    #[test]
    fn the_route_wraps_with_no_pose_seam_and_no_line_hierarchy_jump() {
        let a = route_pose(0.0);
        let b = route_pose(ROUTE_LOOP_SECONDS);
        assert_eq!(a.yaw, b.yaw, "yaw must close");
        assert_eq!(a.pitch, b.pitch, "pitch must close");
        let advanced = route_pose(ROUTE_LOOP_SECONDS * (1.0 - 1e-6)).forward_cells;
        assert!(
            (advanced - FORWARD_CELLS_PER_LOOP).abs() < 1e-2,
            "one loop must travel exactly {FORWARD_CELLS_PER_LOOP} cells, got {advanced}"
        );
        // THE LOAD-BEARING ARITHMETIC: an integer multiple of the major modulus.
        // A non-multiple wraps the minor lattice while rotating which lines are
        // major, and the loop shows a one-cell hierarchy jump.
        let cells = FORWARD_CELLS_PER_LOOP;
        assert_eq!(
            cells % MAJOR_EVERY,
            0.0,
            "forward travel per loop ({cells}) must be a multiple of the major \
             modulus ({MAJOR_EVERY}) or the wrap rotates the line hierarchy"
        );
    }

    /// No cut, no jump, no attention-grabbing acceleration: the pose has zero
    /// velocity at every leg boundary AND across the whole steady hold, and the
    /// forward speed is exactly constant everywhere.
    #[test]
    fn every_leg_boundary_holds_still_and_forward_speed_never_changes() {
        let eps = 0.02_f32;
        for leg in 0..ROUTE_LEGS {
            let b = ROUTE_LEG_SECONDS * leg as f32;
            let before = route_pose(b - eps);
            let at = route_pose(b);
            let after = route_pose(b + eps);
            for (label, x, y) in [
                ("yaw", before.yaw, at.yaw),
                ("pitch", before.pitch, at.pitch),
            ] {
                assert!(
                    (x - y).abs() < 1e-4,
                    "leg {leg} boundary must arrive at rest ({label}: {x} -> {y})"
                );
            }
            for (label, x, y) in [("yaw", at.yaw, after.yaw), ("pitch", at.pitch, after.pitch)] {
                assert!(
                    (x - y).abs() < 1e-4,
                    "leg {leg} boundary must leave at rest ({label}: {x} -> {y})"
                );
            }
        }
        // Constant forward speed, sampled across the whole route.
        let mut speeds = Vec::new();
        let n = 200;
        for i in 0..n {
            let t0 = ROUTE_LOOP_SECONDS * i as f32 / n as f32;
            let t1 = ROUTE_LOOP_SECONDS * (i as f32 + 0.5) / n as f32;
            speeds.push((route_pose(t1).forward_cells - route_pose(t0).forward_cells) / (t1 - t0));
        }
        let lo = speeds.iter().cloned().fold(f32::MAX, f32::min);
        let hi = speeds.iter().cloned().fold(f32::MIN, f32::max);
        assert!(
            (hi - lo).abs() < 1e-4,
            "forward travel must never accelerate (speed {lo}..{hi} cells/s)"
        );
    }

    /// A LONG steady direction, not a continuous wander: each leg genuinely
    /// holds its authored pose for [`ROUTE_HOLD_FRAC`] of its length.
    #[test]
    fn each_leg_holds_one_steady_direction_before_it_eases() {
        for leg in 0..ROUTE_LEGS {
            let base = ROUTE_LEG_SECONDS * leg as f32;
            let start = route_pose(base);
            let hold_end = route_pose(base + ROUTE_LEG_SECONDS * (ROUTE_HOLD_FRAC - 1e-4));
            assert!(
                (start.yaw - hold_end.yaw).abs() < 1e-5
                    && (start.pitch - hold_end.pitch).abs() < 1e-5,
                "leg {leg} must hold one direction across its whole hold \
                 ({start:?} vs {hold_end:?})"
            );
            let held = ROUTE_LEG_SECONDS * ROUTE_HOLD_FRAC;
            assert!(
                held > 25.0,
                "a 'long direction' must hold for tens of seconds, got {held}s"
            );
        }
    }

    /// No hard reversal: the steering never crosses from a committed bend
    /// straight into its opposite — it always passes through a near-straight
    /// stretch first.
    #[test]
    fn the_route_never_reverses_hard_from_one_bend_into_its_opposite() {
        let step = 0.25_f32;
        let mut committed: Option<f32> = None; // sign of the last committed yaw
        let mut n = 0;
        let mut t = 0.0;
        while t < ROUTE_LOOP_SECONDS {
            let yaw = route_pose(t).yaw;
            if yaw.abs() > 0.45 {
                if let Some(prev) = committed {
                    assert!(
                        prev * yaw > 0.0,
                        "yaw jumped from a committed {prev} bend to {yaw} at {t}s \
                         without passing through straight"
                    );
                }
                committed = Some(yaw.signum());
                n += 1;
            } else if yaw.abs() < 0.08 {
                committed = None; // straightened out; the next bend may go either way
            }
            t += step;
        }
        assert!(n > 0, "the route must actually commit to bends");
    }

    /// Losing focus pauses in place and resumes WITHOUT catch-up: a long
    /// delayed wake advances by exactly one sparse ambient step, never the
    /// accumulated wall time.
    #[test]
    fn a_delayed_wake_advances_one_sparse_step_never_the_lost_wall_time() {
        let from = 31.5_f32;
        let one = advance_phase(from, crate::lava::LAVA_TICK_SECONDS);
        for delayed in [0.4_f32, 2.0, 12.0, 600.0] {
            assert_eq!(
                advance_phase(from, delayed),
                one,
                "a {delayed}s stall must still advance exactly one ambient step"
            );
        }
        // And it really does advance on an ordinary wake (non-vacuity).
        assert!(one > from, "an ordinary tick must advance the route");
    }

    /// Reduce Motion and the headless capture share ONE composed still, and the
    /// gallery knob overrides both.
    #[test]
    fn reduce_motion_and_headless_share_the_one_composed_still() {
        assert_eq!(phase_for(123.4, true, None), FROZEN_PHASE);
        assert_eq!(phase_for(FROZEN_PHASE, false, None), FROZEN_PHASE);
        assert_eq!(phase_for(123.4, false, None), 123.4);
        assert_eq!(phase_for(123.4, true, Some(58.0)), 58.0);
        // The override is wrapped, so an out-of-range gallery value is total.
        assert_eq!(phase_for(0.0, false, Some(ROUTE_LOOP_SECONDS + 10.0)), 10.0);
    }

    /// The knob's whole named vocabulary parses, and an unset/garbage value is
    /// a no-op rather than a panic.
    #[test]
    fn the_gallery_knob_parses_every_named_pose_and_rejects_garbage() {
        for name in [
            "straight", "still", "settled", "left", "climb", "up", "right", "descent", "descend",
            "down", "wrap", "116.0", "  LEFT  ",
        ] {
            assert!(parse_pose(name).is_some(), "{name:?} must parse");
        }
        for bad in ["", "sideways", "nan", "loop-de-loop"] {
            assert!(parse_pose(bad).is_none(), "{bad:?} must not parse");
        }
    }
}
