//! Deterministic route choreography for the warped-grid ambient background.
//!
//! The live App advances this phase from the existing sparse ambient cadence.
//! Nothing here reads a clock: focus/resize holds, ambient-motion off, Reduce
//! Motion, and headless capture all resolve through the same explicit phase.

use std::sync::OnceLock;

/// Each authored direction lasts about a minute before easing into the next.
pub const ROUTE_LEG_SECONDS: f32 = 58.0;
pub const ROUTE_LEGS: usize = 6;
pub const ROUTE_LOOP_SECONDS: f32 = ROUTE_LEG_SECONDS * ROUTE_LEGS as f32;

/// The route advances an integer number of projected minor-grid cells per
/// complete loop, so both the steering pose and forward travel meet their
/// starting values at the same endpoint.
pub const FORWARD_CELLS_PER_LOOP: f32 = 64.0;

pub const FROZEN_PHASE: f32 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoutePose {
    pub yaw: f32,
    pub pitch: f32,
    pub forward_cells: f32,
}

const TARGETS: [(f32, f32); ROUTE_LEGS] = [
    (0.0, 0.0),   // straight
    (-0.72, 0.0), // long left bend
    (0.0, -0.58), // climb
    (0.68, 0.0),  // long right bend
    (0.0, 0.56),  // descend
    (0.0, 0.0),   // straight before the seamless wrap
];

fn smoothstep01(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn route_pose(phase_seconds: f32) -> RoutePose {
    let phase = phase_seconds.rem_euclid(ROUTE_LOOP_SECONDS);
    let leg_f = phase / ROUTE_LEG_SECONDS;
    let leg = (leg_f.floor() as usize).min(ROUTE_LEGS - 1);
    let next = (leg + 1) % ROUTE_LEGS;
    let t = smoothstep01(leg_f - leg as f32);
    let (yaw0, pitch0) = TARGETS[leg];
    let (yaw1, pitch1) = TARGETS[next];
    RoutePose {
        yaw: yaw0 + (yaw1 - yaw0) * t,
        pitch: pitch0 + (pitch1 - pitch0) * t,
        forward_cells: phase / ROUTE_LOOP_SECONDS * FORWARD_CELLS_PER_LOOP,
    }
}

/// Advance by at most one sparse ambient step. A late event-loop wake never
/// catches the tunnel up in one visible jump.
pub fn advance_phase(phase_seconds: f32, dt: f32) -> f32 {
    let bounded = dt.clamp(0.0, crate::lava::LAVA_TICK_SECONDS);
    (phase_seconds + bounded).rem_euclid(ROUTE_LOOP_SECONDS)
}

pub fn phase_for(live: f32, reduced: bool, forced: Option<f32>) -> f32 {
    if let Some(phase) = forced {
        phase.rem_euclid(ROUTE_LOOP_SECONDS)
    } else if reduced {
        FROZEN_PHASE
    } else {
        live.rem_euclid(ROUTE_LOOP_SECONDS)
    }
}

/// Gallery-only pose selector. It is deliberately environment-only rather
/// than a user setting: the authored route is the product.
pub fn env_phase() -> Option<f32> {
    static VALUE: OnceLock<Option<f32>> = OnceLock::new();
    *VALUE.get_or_init(|| {
        let raw = std::env::var("AWL_WARP_GRID_POSE").ok()?;
        let phase = match raw.trim().to_ascii_lowercase().as_str() {
            "straight" | "still" | "paused" => 0.0,
            "left" => ROUTE_LEG_SECONDS,
            "climb" | "up" => ROUTE_LEG_SECONDS * 2.0,
            "right" => ROUTE_LEG_SECONDS * 3.0,
            "descent" | "descend" | "down" => ROUTE_LEG_SECONDS * 4.0,
            other => other.parse::<f32>().ok()?,
        };
        Some(phase.rem_euclid(ROUTE_LOOP_SECONDS))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_route_hits_every_named_direction_and_wraps_without_a_pose_seam() {
        let straight = route_pose(0.0);
        let left = route_pose(ROUTE_LEG_SECONDS);
        let climb = route_pose(ROUTE_LEG_SECONDS * 2.0);
        let right = route_pose(ROUTE_LEG_SECONDS * 3.0);
        let descent = route_pose(ROUTE_LEG_SECONDS * 4.0);
        assert_eq!((straight.yaw, straight.pitch), (0.0, 0.0));
        assert!(left.yaw < -0.7 && left.pitch.abs() < 1e-6);
        assert!(climb.pitch < -0.5 && climb.yaw.abs() < 1e-6);
        assert!(right.yaw > 0.6 && right.pitch.abs() < 1e-6);
        assert!(descent.pitch > 0.5 && descent.yaw.abs() < 1e-6);

        let end = route_pose(ROUTE_LOOP_SECONDS);
        assert_eq!(straight, end);
    }

    #[test]
    fn every_leg_boundary_has_zero_velocity_and_no_hard_reversal() {
        let epsilon = 0.01;
        for leg in 0..ROUTE_LEGS {
            let boundary = leg as f32 * ROUTE_LEG_SECONDS;
            let before = route_pose(boundary - epsilon);
            let at = route_pose(boundary);
            let after = route_pose(boundary + epsilon);
            assert!((before.yaw - at.yaw).abs() < 1e-4);
            assert!((after.yaw - at.yaw).abs() < 1e-4);
            assert!((before.pitch - at.pitch).abs() < 1e-4);
            assert!((after.pitch - at.pitch).abs() < 1e-4);
        }
    }

    #[test]
    fn delayed_tick_advances_only_one_sparse_step() {
        let ordinary = advance_phase(12.0, crate::lava::LAVA_TICK_SECONDS);
        let delayed = advance_phase(12.0, 8.0);
        assert_eq!(ordinary, delayed);
    }

    #[test]
    fn reduced_motion_and_headless_start_share_the_composed_still() {
        assert_eq!(phase_for(123.0, true, None), FROZEN_PHASE);
        assert_eq!(phase_for(FROZEN_PHASE, false, None), FROZEN_PHASE);
        assert_eq!(
            phase_for(0.0, false, Some(ROUTE_LEG_SECONDS)),
            ROUTE_LEG_SECONDS
        );
    }
}
