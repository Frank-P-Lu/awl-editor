use super::BgDesc;

/// Pack the mutually exclusive per-ground controls into the shared param slots.
pub(super) fn ground_params(desc: &BgDesc) -> [f32; 4] {
    match desc.shader {
        // Organic: cell scale + density + the authored arrangement.
        8 => [desc.period_px, desc.density, desc.profile, 0.0],
        // Deckle mode is total over Weave × DeckleAnchor: viewport Strata = 0,
        // Fibres = 1 (anchor ignored), page-relative Strata = 2.
        9 => [
            desc.period_px,
            desc.amplitude_px,
            desc.density,
            if desc.profile >= 0.5 {
                1.0
            } else {
                2.0 * desc.deckle_anchor
            },
        ],
        // Warped grid: projected minor-cell spacing, coverage, and framing.
        10 => [desc.period_px, desc.density, 0.0, desc.tunnel],
        _ => {
            let edge_period = if desc.edge { 1.0 } else { 0.0 } + desc.period_px;
            let signed_density = if desc.banded { -1.0 } else { 1.0 } * desc.density;
            [edge_period, desc.angle, desc.amplitude_px, signed_density]
        }
    }
}
