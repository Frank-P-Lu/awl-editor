//! WAVES phase-drift laws — split out of `background.rs`'s former
//! inline `mod waves_drift_tests` to make room
//! under the file's own size ratchet without touching the drift math itself);
//! every test's NAME is unchanged, only which file its source lives in moved.
use super::waves::{WAVE_FREQ, waves_boundaries};
use super::*;

#[test]
fn drift_is_zero_at_the_settled_phase() {
    assert_eq!(waves_drift_radians(0.0), 0.0);
}

#[test]
fn drift_wraps_seamlessly_at_the_shared_clocks_loop_endpoint() {
    let h = 900.0;
    for x in [0.0, 137.0, 512.0, 1801.0_f32] {
        let start = waves_boundaries(x, h, waves_drift_radians(0.0));
        let end = waves_boundaries(x, h, waves_drift_radians(crate::lava::LAVA_LOOP_CYCLES));
        assert!(
            (start.0 - end.0).abs() < 1e-2,
            "b1 seamless at the wrap: {start:?} vs {end:?}"
        );
        assert!(
            (start.1 - end.1).abs() < 1e-2,
            "b2 seamless at the wrap: {start:?} vs {end:?}"
        );
    }
}

#[test]
fn boundaries_never_cross_at_any_drift_phase() {
    let h = 900.0;
    for step in 0..20 {
        let phase = step as f32 * crate::lava::LAVA_LOOP_CYCLES / 20.0;
        let drift = waves_drift_radians(phase);
        for x in (0..2000).step_by(97) {
            let (b1, b2) = waves_boundaries(x as f32, h, drift);
            assert!(
                b1 < b2,
                "tiers never cross at drift={drift}, x={x}: b1={b1} b2={b2}"
            );
        }
    }
}

#[test]
fn drift_is_not_a_rigid_one_sheet_translation() {
    let h = 900.0;
    let d = 0.7_f32;
    let shift = d / WAVE_FREQ;
    let (b1_d, b2_d) = waves_boundaries(123.0, h, d);
    let (b1_static_shifted, b2_static_shifted) = waves_boundaries(123.0 + shift, h, 0.0);
    assert!(
        (b1_d - b1_static_shifted).abs() < 1e-2,
        "b1 alone is a pure phase shift by d/FREQ: {b1_d} vs {b1_static_shifted}"
    );
    assert!(
        (b2_d - b2_static_shifted).abs() > 1.0,
        "b2 does NOT follow b1's shift -- the field is genuinely layered \
             (counter-moving), not one rigid sheet: {b2_d} vs {b2_static_shifted}"
    );
}

#[test]
fn nonzero_drift_actually_moves_the_boundaries() {
    let h = 900.0;
    let (b1_0, b2_0) = waves_boundaries(50.0, h, 0.0);
    let (b1_d, b2_d) = waves_boundaries(50.0, h, 1.1);
    assert!((b1_0 - b1_d).abs() > 0.5, "b1 moves under drift");
    assert!((b2_0 - b2_d).abs() > 0.5, "b2 moves under drift");
}

/// **BIT-IDENTITY, OVER EVERY BYTE.** `srgba_u8_to_linear` used to carry its
/// own inline per-channel loop; it now calls `theme::srgb_channel_to_linear_f32`.
/// This is the pre-refactor formula, written out independently (the pattern
/// `theme::tests::clear`'s own round-trip oracle uses) rather than re-imported,
/// so a regression in the shared owner cannot also hide from the very test
/// meant to catch it. `f32` throughout, matching every shader-side call site —
/// a `f64`-computed-then-cast value is measurably NOT bit-identical here (up
/// to 6 ULP apart on 214/256 bytes), which is exactly why the owner has an
/// `f32`-width entry point at all rather than one shared `f64` function.
#[test]
fn srgba_u8_to_linear_is_bit_identical_to_the_pre_refactor_formula_over_every_byte() {
    fn reference_channel(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    for v in 0u8..=255 {
        let want = reference_channel(v);
        let c = srgba_u8_to_linear([v, v, v, v]);
        for (i, ch) in c.iter().take(3).enumerate() {
            assert_eq!(
                ch.to_bits(),
                want.to_bits(),
                "byte {v} channel {i}: got {ch} ({:#010x}), want {want} ({:#010x})",
                ch.to_bits(),
                want.to_bits()
            );
        }
        assert_eq!(c[3], v as f32 / 255.0, "alpha stays a linear passthrough");
    }
}
