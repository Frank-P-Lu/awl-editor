use super::BgDesc;

/// Pack the mutually exclusive ground controls into the shared shader slots.
pub(super) fn ground(desc: &BgDesc) -> [f32; 4] {
    if desc.shader == 8 {
        return [desc.period_px, desc.density, 0.0, 0.0];
    }
    if desc.shader == 9 {
        return [desc.period_px, desc.density, desc.amplitude_px, 0.0];
    }
    [
        if desc.edge { 1.0 } else { 0.0 } + desc.period_px,
        desc.angle,
        desc.amplitude_px,
        if desc.banded {
            -desc.density
        } else {
            desc.density
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warped_grid_params_land_in_the_three_slots_the_shader_reads() {
        let desc = BgDesc {
            from: [0; 4],
            to: [0; 4],
            dir: (0.0, 1.0),
            shader: 9,
            tint: [0; 3],
            edge: false,
            angle: 0.0,
            period_px: 54.0,
            amplitude_px: 0.9,
            density: 0.78,
            banded: false,
        };
        assert_eq!(ground(&desc), [54.0, 0.78, 0.9, 0.0]);
    }
}
