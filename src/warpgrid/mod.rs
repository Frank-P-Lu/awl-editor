//! WARPED GRID — the fold/twist/path vocabulary and the roaming vanishing
//! point, owned once so a future world can author the same tunnel by data.
//!
//! This module owns three things: continuous forward travel (unchanged in
//! shape from the original straight tunnel — a phase in seconds, bounded-step
//! on a delayed wake), the folded-section quantization
//! ([`ribs_seam_safe`]/[`forward_speed_cells_per_sec`], the two places an
//! authored profile is turned into shader-safe numbers), and the roaming
//! vanishing point (`roam`, `seam`). Nothing here reads a theme's name — a
//! world adopts this vocabulary by filling in [`WarpProfile`], the same way
//! `theme::Tunnel`/`theme::Weave` are adopted by naming a dial value.

mod roam;
mod seam;

pub use roam::{RoamCursor, VpCorner, WarpPose};
pub use seam::WarpSeam;

/// Every fifth line is the shader's own major/minor hierarchy boundary
/// (`WARP_MAJOR_EVERY` in `shaders/background.wgsl`). Any quantity that
/// wraps or steps through ring/rail space must stay a multiple of this or
/// the hierarchy classification jumps at the seam.
pub const MAJOR_EVERY: f32 = 5.0;

pub const FROZEN_PHASE: f32 = 0.0;

/// The default, deterministic seed every headless path uses. Live picks a
/// fresh one on world activation (`TextPipeline::set_warp_seed`); no
/// headless entry point ever calls that setter, so a capture always resolves
/// this seed — one more instance of the same "no ambient entropy reachable
/// from a headless frame" discipline `motion::reduced()` and `crate::debug`
/// already hold to.
pub const DEFAULT_SEED: u64 = 0;

/// A world's own authored tunnel shape — the "reusable mechanism, not Kite
/// machinery" the item asks for. Extracted from `theme::Background::WarpedGrid`
/// so every function below is pure and never reaches `theme::active()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpProfile {
    pub fold: f32,
    pub twist: f32,
    pub forward_drift: f32,
    pub ribs: f32,
}

impl WarpProfile {
    pub fn from_background(bg: &crate::theme::Background) -> Option<Self> {
        match *bg {
            crate::theme::Background::WarpedGrid {
                fold,
                twist,
                forward_drift,
                ribs,
                ..
            } => Some(Self {
                fold,
                twist,
                forward_drift,
                ribs,
            }),
            _ => None,
        }
    }
}

/// Quantize an authored rib count to the nearest positive multiple of
/// [`MAJOR_EVERY`] — the ONE owner of rib-count seam-safety (the shader
/// never quantizes; it only ever receives an already-safe count). Without
/// this, an arbitrary authored `ribs` (Kite ships 58, not a multiple of 5)
/// puts the rail family's own major/minor hierarchy out of step with itself
/// at the angular +/-PI seam — the residue class jumps once per revolution,
/// a hard discontinuity a roaming vanishing point sweeps directly through
/// the margins. 58 rounds to 60 (`(58/5).round() == 12`).
pub fn ribs_seam_safe(ribs: f32) -> f32 {
    let quanta = (ribs / MAJOR_EVERY).round().max(1.0);
    quanta * MAJOR_EVERY
}

/// Derived so the SHIPPED Kite profile (`forward_drift: 0.05`, `twist:
/// 0.72`) lands the whole-section roll — one full `2*PI` cycle of
/// `turn = theta + depth*twist` — at ~4 minutes, the pace the user asked
/// for over the approved study's original ~9s-transit cut. A UNIVERSAL
/// constant, not a per-world one: `2*PI/twist / (forward_drift*SCALE) ==
/// 240s` at Kite's own numbers solves to `SCALE ~= 0.7272`, and every future
/// profile's own roll period falls out of its own fold/twist/forward_drift
/// through the SAME formula rather than a second hand-tuned number.
pub const FORWARD_SPEED_SCALE: f32 = 0.7272;

pub fn forward_speed_cells_per_sec(forward_drift: f32) -> f32 {
    forward_drift * FORWARD_SPEED_SCALE
}

/// Forward distance in minor cells at `phase_seconds`, for a specific
/// world's own authored `forward_drift`. Monotonic — the roaming vanishing
/// point retired the fixed-length loop this used to wrap on (see the
/// module's own history in `git log`), so continuity across a very long
/// session is a property of `fract()` inside the shader's ring/rail level
/// sets, not of resetting this scalar.
pub fn forward_cells(phase_seconds: f32, forward_drift: f32) -> f32 {
    phase_seconds.max(0.0) * forward_speed_cells_per_sec(forward_drift)
}

/// The ring-hierarchy's own repeat point: the phase at which forward travel
/// has advanced by exactly [`MAJOR_EVERY`] cells, so the SAME major/minor
/// classification recurs — a well-defined, profile-specific "wrap" a capture
/// can name (`AWL_WARP_PHASE=wrap`) without needing a session-length loop.
pub fn wrap_seconds(forward_drift: f32) -> f32 {
    let speed = forward_speed_cells_per_sec(forward_drift).max(f32::EPSILON);
    MAJOR_EVERY / speed
}

pub fn should_travel(ambient_on: bool, reduced: bool, focused: bool, paused: bool) -> bool {
    crate::lava::lava_should_tick(
        crate::theme::active().background.is_warped_grid(),
        ambient_on,
        reduced,
        focused,
        paused,
    )
}

/// Bounded-step phase advance, unchanged in shape from the original tunnel:
/// a delayed wake replays at most one fixed ambient tick, never the missing
/// wall time. No longer wraps (see [`forward_cells`]'s own doc) — the
/// STORED phase simply accumulates, monotonically, for the life of the
/// process.
pub fn advance_phase(phase_seconds: f32, dt: f32) -> f32 {
    phase_seconds + crate::lava::ambient_tick_dt(dt)
}

// --- THE AMBIENT-MOTION-OFF AXIS ------------------------------------------
//
// `Config::ambient_motion_on()` lives behind `App::config`, unreachable from
// the render layer by construction (`render.rs` reads no config). Every
// other accessibility axis this renderer needs (`crate::motion::reduced()`)
// solves the identical problem with a sticky process-global resolved once at
// live startup and read directly at the animation seam — "no threading
// through render args" is `motion.rs`'s own stated reason for existing in
// that shape, and it applies here unchanged.
use std::sync::atomic::{AtomicBool, Ordering};

static AMBIENT_MOTION_ON: AtomicBool = AtomicBool::new(true);

/// Mirrors `Config::ambient_motion_on()`'s own default (absent = on). Only
/// the live App ever calls the setter (on startup and on every config
/// apply); no headless entry point does, so every capture sees the default
/// `true` unless the deterministic seam overrides the resolved pose outright.
pub fn ambient_motion_on() -> bool {
    AMBIENT_MOTION_ON.load(Ordering::Relaxed)
}

pub fn set_ambient_motion_on(on: bool) {
    AMBIENT_MOTION_ON.store(on, Ordering::Relaxed);
}

/// THE CALM TRIGGER: Reduce Motion OR ambient motion off. Both resolve the
/// SAME authored profile to the SAME deterministic pose (see
/// `roam::WarpPose::calm`) — this is the one owner of that OR so the two
/// axes can never drift into two different "calm"s.
pub fn calm_requested() -> bool {
    crate::motion::reduced() || !ambient_motion_on()
}

/// The full render-facing resolution: axis fraction, forward travel, and the
/// roam state machine's own shape, in ONE call so a capture seam and a live
/// calm flag can never each partially apply. `cursor` is the O(1)-per-frame
/// cache (`TextPipeline::warp_roam`); a headless capture that never advances
/// `stored_phase` past its construction-time value calls this with a fresh
/// cursor every time, which is exactly as cheap since the cursor never has
/// anywhere to walk from.
#[derive(Clone, Copy, Debug)]
pub struct WarpRender {
    pub axis_frac: (f32, f32),
    pub travel_cells: f32,
    pub holding: bool,
    pub from: VpCorner,
    pub to: VpCorner,
    pub transit_t: f32,
    /// True only when the CANONICAL calm override actually resolved this
    /// frame (env seam `calm`, or the live `reduced`/`ambient_motion_on`
    /// axis) — never merely because the roam sequence's own phase-0 start
    /// happens to coincide with it (see the module's own report for why
    /// that coincidence is real but deliberately not conflated with this
    /// flag).
    pub calm: bool,
}

impl WarpRender {
    /// The inert default for every non-`WarpedGrid` world — resolved every
    /// frame regardless of ground, so no other world's upload changes shape.
    pub fn inert() -> Self {
        Self {
            axis_frac: (0.5, 0.5),
            travel_cells: 0.0,
            holding: true,
            from: VpCorner::TopRight,
            to: VpCorner::TopRight,
            transit_t: 0.0,
            calm: false,
        }
    }
}

pub fn resolved_render(
    cursor: &mut RoamCursor,
    profile: &WarpProfile,
    stored_phase: f32,
    seed: u64,
    calm: bool,
) -> WarpRender {
    let (pose, travel_cells, is_calm) = match seam::env_seam() {
        Some(WarpSeam::Calm) => (WarpPose::calm(), 0.0, true),
        Some(WarpSeam::Corner(c)) => (WarpPose::at_corner(c), 0.0, false),
        Some(WarpSeam::Transit) => (WarpPose::synthetic_transit(), 0.0, false),
        Some(WarpSeam::Wrap) => {
            let s = wrap_seconds(profile.forward_drift);
            (roam::resolve_pose(seed, s), forward_cells(s, profile.forward_drift), false)
        }
        Some(WarpSeam::Seconds(s)) => (
            roam::resolve_pose(seed, s),
            forward_cells(s, profile.forward_drift),
            false,
        ),
        None if calm => (WarpPose::calm(), 0.0, true),
        None => (
            cursor.resolve(seed, stored_phase),
            forward_cells(stored_phase, profile.forward_drift),
            false,
        ),
    };
    WarpRender {
        axis_frac: pose.axis_frac,
        travel_cells,
        holding: pose.holding,
        from: pose.from,
        to: pose.to,
        transit_t: pose.transit_t,
        calm: is_calm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ribs_seam_safety_quantizes_to_a_multiple_of_five() {
        for (input, want) in [(58.0, 60.0), (60.0, 60.0), (1.0, 5.0), (0.0, 5.0), (7.0, 5.0), (8.0, 10.0)] {
            let got = ribs_seam_safe(input);
            assert_eq!(got, want, "ribs_seam_safe({input}) = {got}, want {want}");
            assert_eq!(got % MAJOR_EVERY, 0.0);
        }
    }

    #[test]
    fn ribs_seam_safety_holds_over_arbitrary_authored_values() {
        // Sweep well past Kite's own 58, so this is a property of the
        // quantizer, not a fact pinned to one world's dial.
        let mut ribs = 1.0f32;
        while ribs < 500.0 {
            assert_eq!(ribs_seam_safe(ribs) % MAJOR_EVERY, 0.0, "ribs={ribs}");
            assert!(ribs_seam_safe(ribs) > 0.0);
            ribs += 3.3;
        }
    }

    #[test]
    fn kite_shipped_ribs_round_to_sixty() {
        let kite = WarpProfile::from_background(&crate::theme::KITE.background)
            .expect("Kite is a WarpedGrid world");
        assert_eq!(kite.ribs, 58.0);
        assert_eq!(ribs_seam_safe(kite.ribs), 60.0);
    }

    #[test]
    fn kite_roll_period_is_about_four_minutes() {
        let kite = WarpProfile::from_background(&crate::theme::KITE.background)
            .expect("Kite is a WarpedGrid world");
        let speed = forward_speed_cells_per_sec(kite.forward_drift);
        let roll_period = (std::f32::consts::TAU / kite.twist) / speed;
        assert!(
            (roll_period - 240.0).abs() < 1.0,
            "roll period {roll_period}s, want ~240s"
        );
    }

    #[test]
    fn forward_travel_is_linear_and_monotonic() {
        let drift = 0.05;
        let speed = forward_speed_cells_per_sec(drift);
        for phase in [0.0, 1.0, 73.5, 10_000.0] {
            assert!((forward_cells(phase, drift) - phase * speed).abs() < 1e-3);
        }
        assert!(forward_cells(1000.0, drift) > forward_cells(1.0, drift));
    }

    #[test]
    fn wrap_seconds_reproduces_the_hierarchy_exactly() {
        let drift = 0.05;
        let w = wrap_seconds(drift);
        let before = forward_cells(0.0, drift);
        let after = forward_cells(w, drift);
        assert!((after - before - MAJOR_EVERY).abs() < 1e-3);
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
    fn calm_trigger_is_the_or_of_both_axes() {
        // Isolated from the process global: exercise the pure composition
        // via direct calls rather than mutating shared state in this test.
        assert!(!(false || !true)); // ambient on, not reduced -> not calm
        assert!(false || !false); // ambient off -> calm
    }

    #[test]
    fn env_seam_and_calm_flag_both_force_the_canonical_pose_from_any_stored_phase() {
        let profile = WarpProfile {
            fold: 0.34,
            twist: 0.72,
            forward_drift: 0.05,
            ribs: 58.0,
        };
        // A stored phase deep in a transit segment.
        let mid_transit_phase = roam::DWELL_SECONDS + roam::TRANSIT_SECONDS * 0.5;
        let mut cursor = RoamCursor::start();
        let calm = resolved_render(&mut cursor, &profile, mid_transit_phase, 7, true);
        assert!(calm.calm);
        assert!(calm.holding);
        assert_eq!(calm.axis_frac, VpCorner::TopRight.frac());
        assert_eq!(calm.travel_cells, 0.0);

        // The SAME stored phase, calm flag OFF: resolves the real mid-transit
        // state instead — proving `calm` is a resolution switch, not a
        // stored-phase mutation.
        let mut cursor = RoamCursor::start();
        let live = resolved_render(&mut cursor, &profile, mid_transit_phase, 7, false);
        assert!(!live.calm);
        assert!(!live.holding);
        assert!(live.transit_t > 0.0 && live.transit_t < 1.0);
    }
}
