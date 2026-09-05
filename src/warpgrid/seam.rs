//! `AWL_WARP_PHASE` — the deterministic capture seam over the roaming tunnel.
//! Extended from a bare seconds value (the old straight/circular tunnel's
//! only knob) to name every state a capture needs: a specific corner, a
//! midpoint transition, the ring-hierarchy wrap, or the motion-safe calm
//! pose — without depending on the pseudo-random sequence (see
//! `roam::WarpPose::at_corner`/`synthetic_transit`, which resolve these
//! directly rather than trying to invert the sequence for a given phase).
//!
//! Optional; unset in every normal run and every ordinary capture. Never
//! read anywhere but the render seam that resolves [`super::WarpPose`].

use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WarpSeam {
    /// The motion-safe authored calm pose.
    Calm,
    /// A synthetic, sequence-independent hold at one named corner.
    Corner(super::VpCorner),
    /// A synthetic, sequence-independent midpoint transition.
    Transit,
    /// The ring-hierarchy's own repeat point for the active profile.
    Wrap,
    /// A raw phase override, in seconds, through the real sequence.
    Seconds(f32),
}

fn parse(raw: &str) -> Option<WarpSeam> {
    use super::VpCorner::*;
    match raw.trim().to_ascii_lowercase().as_str() {
        "still" | "settled" | "start" | "calm" => Some(WarpSeam::Calm),
        "top-left" | "tl" => Some(WarpSeam::Corner(TopLeft)),
        "top-right" | "tr" => Some(WarpSeam::Corner(TopRight)),
        "bottom-left" | "bl" => Some(WarpSeam::Corner(BottomLeft)),
        "bottom-right" | "br" => Some(WarpSeam::Corner(BottomRight)),
        "transit" => Some(WarpSeam::Transit),
        "wrap" => Some(WarpSeam::Wrap),
        other => {
            let seconds: f32 = other.parse().ok()?;
            seconds.is_finite().then_some(WarpSeam::Seconds(seconds))
        }
    }
}

/// Optional gallery/capture seam. Normal runs and captures leave it unset.
pub fn env_seam() -> Option<WarpSeam> {
    static ONCE: OnceLock<Option<WarpSeam>> = OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_WARP_PHASE")
            .ok()
            .as_deref()
            .and_then(parse)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::warpgrid::VpCorner;

    #[test]
    fn every_named_state_parses() {
        for (raw, want) in [
            ("still", WarpSeam::Calm),
            ("settled", WarpSeam::Calm),
            ("start", WarpSeam::Calm),
            ("calm", WarpSeam::Calm),
            ("top-left", WarpSeam::Corner(VpCorner::TopLeft)),
            ("tl", WarpSeam::Corner(VpCorner::TopLeft)),
            ("top-right", WarpSeam::Corner(VpCorner::TopRight)),
            ("bottom-left", WarpSeam::Corner(VpCorner::BottomLeft)),
            ("bottom-right", WarpSeam::Corner(VpCorner::BottomRight)),
            ("br", WarpSeam::Corner(VpCorner::BottomRight)),
            ("transit", WarpSeam::Transit),
            ("wrap", WarpSeam::Wrap),
            ("116.0", WarpSeam::Seconds(116.0)),
            ("0", WarpSeam::Seconds(0.0)),
        ] {
            assert_eq!(parse(raw), Some(want), "raw={raw}");
        }
    }

    #[test]
    fn case_and_whitespace_insensitive() {
        assert_eq!(
            parse("  Top-Right  "),
            Some(WarpSeam::Corner(VpCorner::TopRight))
        );
        assert_eq!(parse("CALM"), Some(WarpSeam::Calm));
    }

    #[test]
    fn garbage_parses_to_none() {
        for raw in ["", "left", "nan", "top-middle"] {
            assert_eq!(parse(raw), None, "raw={raw}");
        }
    }
}
