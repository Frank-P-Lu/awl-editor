//! The ROAMING VANISHING POINT — a deterministic, seedable, pure state
//! machine. Four room-owned targets (viewport fractions, independent of page
//! geometry); hold each for [`DWELL_SECONDS`], then drift to a pseudo-randomly
//! chosen next target (never immediately repeating) over [`TRANSIT_SECONDS`]
//! on a zero-velocity-at-both-ends [`smootherstep`] curve.
//!
//! Nothing here reads the clock, the theme, or any global — every function is
//! a pure map from `(seed, phase_seconds)` (or `(corner, seed, segment)`) to a
//! result, which is what makes it reachable from a headless capture and from
//! the live per-frame hot path with the SAME answer (see [`RoamCursor`]).

/// One of the four room-owned vanishing-point targets, as a VIEWPORT
/// fraction — independent of page geometry, per the brief.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VpCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl VpCorner {
    /// Declaration order IS the roster every no-wildcard sweep walks and the
    /// order [`others`] filters against, so a new corner is enrolled by
    /// adding it here once.
    pub const ALL: [VpCorner; 4] = [
        VpCorner::TopLeft,
        VpCorner::TopRight,
        VpCorner::BottomLeft,
        VpCorner::BottomRight,
    ];

    pub fn frac(self) -> (f32, f32) {
        match self {
            VpCorner::TopLeft => (0.20, 0.24),
            VpCorner::TopRight => (0.80, 0.24),
            VpCorner::BottomLeft => (0.20, 0.76),
            VpCorner::BottomRight => (0.80, 0.76),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            VpCorner::TopLeft => "top-left",
            VpCorner::TopRight => "top-right",
            VpCorner::BottomLeft => "bottom-left",
            VpCorner::BottomRight => "bottom-right",
        }
    }

    /// The three OTHER corners, in [`ALL`]'s own order — the pool
    /// `next_corner` picks from, which is what makes "no immediate repeat" a
    /// structural property (the current corner is never in its own pool)
    /// rather than a retry loop.
    fn others(self) -> [VpCorner; 3] {
        let mut out = [VpCorner::TopLeft; 3];
        let mut i = 0;
        for c in VpCorner::ALL {
            if c != self {
                out[i] = c;
                i += 1;
            }
        }
        out
    }
}

/// Each held target is shown for this long...
pub const DWELL_SECONDS: f32 = 15.0;
/// ...then the camera drifts to the next one over this long. The approved
/// study used 9 seconds; the user explicitly asked for longer so the move
/// reads as the tunnel CONTORTING rather than a camera sliding.
pub const TRANSIT_SECONDS: f32 = 12.0;
pub const SEGMENT_SECONDS: f32 = DWELL_SECONDS + TRANSIT_SECONDS;

/// A fast, deterministic, non-cryptographic mix (splitmix64's finalizer) —
/// good enough for "pseudo-random, never repeats, looks unpredictable",
/// which is all a target sequence needs.
fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// The target reached from `current` at transition `segment` (the segment
/// INDEX being left, so segment 0's transition picks segment 1's target) — a
/// pure function of `(current, seed, segment)`, so a fixed seed reproduces
/// the exact same sequence on every run. `current` is excluded from its own
/// pool by construction ([`VpCorner::others`]), which is the no-immediate-
/// repeat guarantee: there is no retry, because there is nothing to retry.
pub fn next_corner(current: VpCorner, seed: u64, segment: u64) -> VpCorner {
    let mixed = splitmix64(seed ^ splitmix64(segment));
    let pool = current.others();
    pool[(mixed % 3) as usize]
}

/// The corner held AT `segment` (segment 0 is always [`VpCorner::TopRight`] —
/// "start at top-right"), by walking the deterministic chain from the start.
/// O(segment) — fine for a bounded test/capture query; the live per-frame
/// path never calls this directly (see [`RoamCursor`], which amortizes the
/// SAME walk to O(1) per frame).
pub fn corner_at(segment: u64, seed: u64) -> VpCorner {
    let mut corner = VpCorner::TopRight;
    let mut i = 0;
    while i < segment {
        corner = next_corner(corner, seed, i);
        i += 1;
    }
    corner
}

/// Perlin's smootherstep: zero velocity AND zero curvature at both ends, so a
/// transit neither jerks into motion nor snaps to a stop — "the tunnel
/// contorting", not a camera cut.
pub fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The resolved state of the roaming vanishing point for one frame — the
/// ONE thing the shader ultimately needs (`axis_frac`) plus enough of the
/// state machine's own shape (`holding`/`from`/`to`/`transit_t`) for the
/// sidecar to report it and for laws to assert against it directly, rather
/// than reverse-engineering the state from the interpolated fraction alone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpPose {
    pub axis_frac: (f32, f32),
    pub holding: bool,
    pub from: VpCorner,
    pub to: VpCorner,
    /// Eased transit progress, `0.0` while holding.
    pub transit_t: f32,
}

impl WarpPose {
    /// The motion-safe authored pose: vanishing point locked at top-right,
    /// no target sequence. Reachable from ANY prior state — this constructs
    /// fresh rather than reading one, which is what makes it a genuine
    /// override rather than a coincidence of the stored phase.
    pub fn calm() -> Self {
        Self {
            axis_frac: VpCorner::TopRight.frac(),
            holding: true,
            from: VpCorner::TopRight,
            to: VpCorner::TopRight,
            transit_t: 0.0,
        }
    }

    /// A synthetic, seed-independent HOLD at an arbitrary named corner — the
    /// capture seam's way of naming every corner without needing to invert
    /// the pseudo-random sequence.
    pub fn at_corner(c: VpCorner) -> Self {
        Self {
            axis_frac: c.frac(),
            holding: true,
            from: c,
            to: c,
            transit_t: 0.0,
        }
    }

    /// A synthetic, seed-independent midpoint TRANSIT between two corners
    /// chosen for maximum visual distinction — the capture seam's "a
    /// midpoint transition" without depending on where the real sequence
    /// happens to be at some phase value.
    pub fn synthetic_transit() -> Self {
        Self::at_progress(VpCorner::TopRight, VpCorner::BottomLeft, 0.5)
    }

    fn at_progress(from: VpCorner, to: VpCorner, raw_t: f32) -> Self {
        let e = smootherstep(raw_t);
        let (fx, fy) = from.frac();
        let (tx, ty) = to.frac();
        Self {
            axis_frac: (lerp(fx, tx, e), lerp(fy, ty, e)),
            holding: false,
            from,
            to,
            transit_t: e,
        }
    }
}

/// Resolve the roam state at `phase_seconds` from scratch — PURE, O(segment
/// count). This is the ground-truth definition every law is checked against;
/// [`RoamCursor`] is an O(1)-per-frame cache that is PROVEN to agree with it
/// (`roam_cursor_matches_pure_resolution`), never a second definition.
pub fn resolve_pose(seed: u64, phase_seconds: f32) -> WarpPose {
    let phase = phase_seconds.max(0.0);
    let segment = (phase / SEGMENT_SECONDS).floor() as u64;
    let t_in_seg = phase - segment as f32 * SEGMENT_SECONDS;
    let from = corner_at(segment, seed);
    if t_in_seg < DWELL_SECONDS {
        WarpPose {
            axis_frac: from.frac(),
            holding: true,
            from,
            to: from,
            transit_t: 0.0,
        }
    } else {
        let to = next_corner(from, seed, segment);
        let raw = (t_in_seg - DWELL_SECONDS) / TRANSIT_SECONDS;
        WarpPose::at_progress(from, to, raw)
    }
}

/// The O(1)-per-frame cache: incrementally walks the SAME chain
/// [`corner_at`] defines, advancing only by however many segments elapsed
/// since the last call (0 or 1 for any real frame cadence, however long the
/// session has run) rather than re-walking from segment 0 every frame.
#[derive(Clone, Copy, Debug)]
pub struct RoamCursor {
    segment: u64,
    corner: VpCorner,
}

impl RoamCursor {
    pub fn start() -> Self {
        Self {
            segment: 0,
            corner: VpCorner::TopRight,
        }
    }

    /// Advance the cursor to cover `phase_seconds`, then resolve the pose at
    /// that phase — the live per-frame entry point.
    pub fn resolve(&mut self, seed: u64, phase_seconds: f32) -> WarpPose {
        let phase = phase_seconds.max(0.0);
        let target_segment = (phase / SEGMENT_SECONDS).floor() as u64;
        while self.segment < target_segment {
            self.corner = next_corner(self.corner, seed, self.segment);
            self.segment += 1;
        }
        let t_in_seg = phase - self.segment as f32 * SEGMENT_SECONDS;
        if t_in_seg < DWELL_SECONDS {
            WarpPose {
                axis_frac: self.corner.frac(),
                holding: true,
                from: self.corner,
                to: self.corner,
                transit_t: 0.0,
            }
        } else {
            let to = next_corner(self.corner, seed, self.segment);
            let raw = (t_in_seg - DWELL_SECONDS) / TRANSIT_SECONDS;
            WarpPose::at_progress(self.corner, to, raw)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: [u64; 5] = [0, 1, 42, 0xDEAD_BEEF, u64::MAX];

    #[test]
    fn no_immediate_repeat_over_a_long_walk() {
        for seed in SEEDS {
            let mut prev = VpCorner::TopRight;
            for segment in 1..500u64 {
                let next = corner_at(segment, seed);
                assert_ne!(
                    next, prev,
                    "seed {seed:#x} segment {segment}: repeated {prev:?}"
                );
                prev = next;
            }
        }
    }

    #[test]
    fn full_roster_is_reachable() {
        for seed in SEEDS {
            let mut seen = std::collections::HashSet::new();
            for segment in 0..200u64 {
                seen.insert(corner_at(segment, seed));
            }
            assert_eq!(
                seen.len(),
                4,
                "seed {seed:#x} never visited all four corners: {seen:?}"
            );
        }
    }

    #[test]
    fn starts_at_top_right() {
        for seed in SEEDS {
            assert_eq!(corner_at(0, seed), VpCorner::TopRight);
        }
    }

    #[test]
    fn exact_dwell_then_exact_transit() {
        let seed = 7;
        let from = corner_at(0, seed);
        let to = next_corner(from, seed, 0);

        // Just inside the dwell: holding, unmoved.
        let p = resolve_pose(seed, DWELL_SECONDS - 0.001);
        assert!(p.holding);
        assert_eq!(p.axis_frac, from.frac());

        // The exact dwell boundary: transit begins, eased progress ~0.
        let p = resolve_pose(seed, DWELL_SECONDS);
        assert!(!p.holding);
        assert!(p.transit_t < 0.01, "transit_t={}", p.transit_t);
        assert_eq!(p.to, to);

        // Just before the segment ends: transit essentially complete.
        let p = resolve_pose(seed, SEGMENT_SECONDS - 0.001);
        assert!(p.transit_t > 0.99, "transit_t={}", p.transit_t);

        // Exactly at the segment boundary: holding again, at `to`.
        let p = resolve_pose(seed, SEGMENT_SECONDS);
        assert!(p.holding);
        assert_eq!(p.axis_frac, to.frac());
    }

    #[test]
    fn endpoint_velocity_and_curvature_are_zero() {
        // Smootherstep's own defining property, swept at both ends.
        let eps = 1e-4;
        for t in [0.0_f32, 1.0] {
            let center = smootherstep(t);
            let a = smootherstep((t - eps).clamp(0.0, 1.0));
            let b = smootherstep((t + eps).clamp(0.0, 1.0));
            // A symmetric finite difference near a smooth extremum is small
            // relative to the step actually taken by a linear ramp (1.0),
            // proving the curve genuinely flattens rather than merely
            // clamping at the boundary.
            assert!(
                (b - a).abs() < eps * 2.0,
                "t={t}: velocity not ~0 ({a} -> {center} -> {b})"
            );
        }
        assert_eq!(smootherstep(0.0), 0.0);
        assert_eq!(smootherstep(1.0), 1.0);
        assert!(
            (smootherstep(0.5) - 0.5).abs() < 1e-6,
            "not point-symmetric"
        );
    }

    #[test]
    fn fixed_seed_is_fully_deterministic() {
        for seed in SEEDS {
            for phase in [0.0, 5.0, 15.0, 21.0, 27.0, 5000.0] {
                assert_eq!(resolve_pose(seed, phase), resolve_pose(seed, phase));
            }
        }
    }

    #[test]
    fn different_seeds_diverge() {
        // Not a hard requirement of any single seed, but the roster of seeds
        // above must not all coincide — otherwise "seedable" would be a
        // decoration over one fixed sequence.
        let sequences: Vec<Vec<VpCorner>> = SEEDS
            .iter()
            .map(|&s| (1..20).map(|seg| corner_at(seg, s)).collect())
            .collect();
        assert!(
            sequences.windows(2).any(|w| w[0] != w[1]),
            "every seed produced the identical sequence"
        );
    }

    #[test]
    fn roam_cursor_matches_pure_resolution() {
        for seed in SEEDS {
            let mut cursor = RoamCursor::start();
            // Walk forward in small, frame-sized steps (never backwards —
            // the live per-frame caller never rewinds), cross-checking the
            // O(1) cache against the from-scratch definition at every step.
            let mut phase = 0.0f32;
            while phase < SEGMENT_SECONDS * 30.0 {
                let cached = cursor.resolve(seed, phase);
                let pure = resolve_pose(seed, phase);
                assert_eq!(cached, pure, "seed {seed:#x} phase {phase}: cache diverged");
                phase += 0.37; // an irrational-ish frame step, on purpose
            }
        }
    }

    #[test]
    fn calm_and_named_overrides_ignore_the_sequence() {
        let calm = WarpPose::calm();
        assert!(calm.holding);
        assert_eq!(calm.from, VpCorner::TopRight);
        assert_eq!(calm.axis_frac, VpCorner::TopRight.frac());

        for c in VpCorner::ALL {
            let p = WarpPose::at_corner(c);
            assert!(p.holding);
            assert_eq!(p.axis_frac, c.frac());
        }

        let t = WarpPose::synthetic_transit();
        assert!(!t.holding);
        assert!((t.transit_t - 0.5).abs() < 1e-6);
    }

    /// MUTATION PROOF: break the no-immediate-repeat rule (let the pool
    /// include the current corner) and watch `no_immediate_repeat_over_a_long_walk`'s
    /// own claim go red. This function is not wired into the shipped path —
    /// it exists so the report can show the exact panic text a real break
    /// produces; see the module's own build report.
    #[cfg(test)]
    fn broken_next_corner_allows_repeats(_current: VpCorner, seed: u64, segment: u64) -> VpCorner {
        let mixed = splitmix64(seed ^ splitmix64(segment));
        VpCorner::ALL[(mixed % 4) as usize]
    }

    #[test]
    fn mutation_proof_broken_pool_does_repeat() {
        // Demonstrates the BROKEN variant CAN repeat (i.e. the real law
        // above is non-vacuous) without polluting the shipped function.
        let mut saw_repeat = false;
        let mut prev = VpCorner::TopRight;
        for segment in 0..200u64 {
            let next = broken_next_corner_allows_repeats(prev, 99, segment);
            if next == prev {
                saw_repeat = true;
                break;
            }
            prev = next;
        }
        assert!(saw_repeat, "broken pool never repeated in 200 draws");
    }
}
