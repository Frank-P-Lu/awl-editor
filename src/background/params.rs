use super::BgDesc;

/// Warped grid's own dedicated shape slot (fold, twist, quantized ribs,
/// unused) — `[0.0;4]` for every other ground, so no other world's upload
/// changes shape.
pub(super) fn warp_shape_params(desc: &BgDesc) -> [f32; 4] {
    if desc.shader == 10 {
        [desc.warp_fold, desc.warp_twist, desc.warp_ribs, 0.0]
    } else {
        [0.0; 4]
    }
}

/// Pack the mutually exclusive per-ground controls into the shared param slots.
pub(super) fn ground_params(desc: &BgDesc) -> [f32; 4] {
    match desc.shader {
        // Organic: cell scale + density. The ground draws ONE arrangement, so
        // the profile slot is inert here.
        8 => [desc.period_px, desc.density, 0.0, 0.0],
        // Deckle mode carries the WEAVE alone: Strata = 0, Fibres = 1. The
        // coordinate owner is no longer a dial — Strata measures from the
        // viewport centre unconditionally, so nothing multiplexes a second
        // control through this slot.
        9 => [
            desc.period_px,
            desc.amplitude_px,
            desc.density,
            desc.profile,
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
