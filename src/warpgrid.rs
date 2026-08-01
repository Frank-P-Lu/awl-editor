//! Continuous forward travel for the warped-grid ground.
//!
//! The projection and framing are static. This module owns only the distance
//! travelled through the lattice, its exact wrap, and the live scheduling gate.

use std::sync::OnceLock;

/// The several-minute repeat is long enough to disappear during ordinary use.
pub const LOOP_SECONDS: f32 = 406.0;

/// Minor cells travelled per loop. This must remain a multiple of the shader's
/// every-fifth-line hierarchy or the wrap changes which rings are major.
pub const FORWARD_CELLS_PER_LOOP: f32 = 65.0;

#[cfg(test)]
pub const MAJOR_EVERY: f32 = 5.0;

pub const FROZEN_PHASE: f32 = 0.0;

pub fn should_travel(ambient_on: bool, reduced: bool, focused: bool, paused: bool) -> bool {
    crate::lava::lava_should_tick(
        crate::theme::active().background.is_warped_grid(),
        ambient_on,
        reduced,
        focused,
        paused,
    )
}

/// Forward distance at `phase_seconds`. The speed is constant; only the exact
/// lattice-period wrap resets the scalar.
pub fn forward_cells(phase_seconds: f32) -> f32 {
    phase_seconds.rem_euclid(LOOP_SECONDS) / LOOP_SECONDS * FORWARD_CELLS_PER_LOOP
}

pub fn advance_phase(phase_seconds: f32, dt: f32) -> f32 {
    (phase_seconds + crate::lava::ambient_tick_dt(dt)).rem_euclid(LOOP_SECONDS)
}

pub fn phase_for(stored: f32, reduced: bool, env: Option<f32>) -> f32 {
    match env {
        Some(e) => e.rem_euclid(LOOP_SECONDS),
        None if reduced => FROZEN_PHASE,
        None => stored,
    }
}

fn parse_phase(raw: &str) -> Option<f32> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "still" | "settled" | "start" => Some(FROZEN_PHASE),
        "wrap" => Some(LOOP_SECONDS),
        other => {
            let phase: f32 = other.parse().ok()?;
            phase.is_finite().then_some(phase)
        }
    }
}

/// Optional gallery phase. Normal runs and captures leave it unset.
pub fn env_phase() -> Option<f32> {
    static ONCE: OnceLock<Option<f32>> = OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_WARP_PHASE")
            .ok()
            .as_deref()
            .and_then(parse_phase)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_travel_is_linear_and_wraps_on_the_line_hierarchy() {
        let speed = FORWARD_CELLS_PER_LOOP / LOOP_SECONDS;
        for phase in [0.0, 1.0, 73.5, LOOP_SECONDS * 0.73] {
            assert!((forward_cells(phase) - phase * speed).abs() < 1e-5);
        }
        assert_eq!(forward_cells(0.0), forward_cells(LOOP_SECONDS));
        assert_eq!(FORWARD_CELLS_PER_LOOP % MAJOR_EVERY, 0.0);
    }

    #[test]
    fn delayed_wakes_advance_one_bounded_step() {
        let from = 31.5;
        let one = advance_phase(from, crate::lava::LAVA_TICK_SECONDS);
        for delayed in [0.4, 2.0, 12.0, 600.0] {
            assert_eq!(advance_phase(from, delayed), one);
        }
        assert!(one > from);
    }

    #[test]
    fn freeze_and_gallery_phase_share_one_resolver() {
        assert_eq!(phase_for(123.4, true, None), FROZEN_PHASE);
        assert_eq!(phase_for(123.4, false, None), 123.4);
        assert_eq!(phase_for(123.4, true, Some(50.0)), 50.0);
        assert_eq!(phase_for(0.0, false, Some(LOOP_SECONDS + 10.0)), 10.0);
        for value in ["still", "settled", "start", "wrap", "116.0"] {
            assert!(parse_phase(value).is_some());
        }
        for value in ["", "left", "nan"] {
            assert!(parse_phase(value).is_none());
        }
    }
}
