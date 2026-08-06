//! Tests for `render.rs`'s sRGB-to-linear tint converter (the frosted-blur
//! composite's alpha-dropped `[f32; 3]` variant).

use super::super::*;

/// **BIT-IDENTITY, OVER EVERY BYTE.** `srgb_u8_to_linear3` used to carry its
/// own inline per-channel loop; it now calls `theme::srgb_channel_to_linear_f32`.
/// This is the pre-refactor formula, written out independently (mirrors
/// `background::tests`'s identical law) so a regression in the shared owner
/// cannot also hide from the test meant to catch it.
#[test]
fn srgb_u8_to_linear3_is_bit_identical_to_the_pre_refactor_formula_over_every_byte() {
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
        let c = srgb_u8_to_linear3([v, v, v, v]);
        for (i, ch) in c.iter().enumerate() {
            assert_eq!(
                ch.to_bits(),
                want.to_bits(),
                "byte {v} channel {i}: got {ch} ({:#010x}), want {want} ({:#010x})",
                ch.to_bits(),
                want.to_bits()
            );
        }
    }
}
