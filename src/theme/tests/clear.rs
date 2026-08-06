//! THE CLEAR COLOUR'S TRANSFER FUNCTION — [`Srgb::to_wgpu_clear`].
//!
//! `LoadOp::Clear` never reaches a fragment stage, so an sRGB-format
//! attachment applies no linear→sRGB encode to its value and consumes it as
//! linear light. A clear that hands over raw sRGB bytes therefore stores their
//! sRGB ENCODE, and the error grows toward black where the curve is steepest:
//! Currawong's authored `#060607` drew as `#2A2A2E` and Potoroo's `#1F0400` as
//! `#622200`, measured against real pixels on every world in the roster.
//!
//! These are the PURE-ARITHMETIC half of that law — no device, no adapter, so
//! they run on every host and on wasm. The DRAWN half (a real frame's page
//! pixel, at 1× and 2×, swept over `THEMES`) is
//! `render::tests::page_ground_law`. Neither subsumes the other: this one
//! proves the number is right and the pixel law proves it is wired.

use super::super::color::srgb_channel_to_linear;
use super::super::*;

/// The sRGB OETF — linear light `[0,1]` back to an sRGB channel `[0,1]`. This
/// is what the attachment's own format does on store, written out here so the
/// round trip below is asserted against the standard rather than against
/// another copy of awl's code.
fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Quantise a linear channel the way an 8-bit sRGB attachment does.
fn store_as_srgb_u8(linear: f64) -> u8 {
    (linear_to_srgb(linear) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// **THE ROUND TRIP, OVER EVERY CHANNEL VALUE THERE IS.** Sweeping all 256
/// bytes rather than the roster's own twenty grounds is the point: the roster
/// is a sample of what worlds happen to be authored today, and the defect this
/// pins is a property of the CURVE — it is negligible at `0xF6` and 36 bytes
/// wide at `0x06`, so a law that only visited light grounds would have shipped
/// green. Pure black and pure white are the curve's two fixed points and are
/// the exact values at which the broken passthrough also passes.
///
/// Asked through [`Srgb::to_wgpu_clear`] — the method the clear site actually
/// calls — rather than through the bare converter, so this grades the WIRING as
/// well as the arithmetic. Asked the other way it stayed green while the
/// production clear was reverted to a raw passthrough, which is the whole reason
/// a mutation is run before a law is believed.
#[test]
fn every_srgb_byte_survives_the_clear_and_the_attachments_encode() {
    let _g = crate::testlock::serial();
    let mut worst = (0u8, 0i32);
    for v in 0u8..=255 {
        // A GREY, so all three channels carry the same byte and a per-channel
        // mix-up inside the method cannot hide behind a lucky permutation…
        let clear = Srgb::rgb(v, v, v).to_wgpu_clear();
        for (ch, linear) in [("r", clear.r), ("g", clear.g), ("b", clear.b)] {
            let back = store_as_srgb_u8(linear);
            assert_eq!(
                back, v,
                "sRGB byte {v} handed to LoadOp::Clear ({ch}) and stored back \
                 through an sRGB attachment came out as {back} — the clear \
                 colour and the drawn pixel must be the same number"
            );
        }
        // …and a lone channel, which does catch that permutation: only `r` is
        // set, so a method that read the wrong field reports 0 here.
        let solo = Srgb::rgb(v, 0, 0).to_wgpu_clear();
        assert_eq!(
            store_as_srgb_u8(solo.r),
            v,
            "the clear's red channel must carry the token's red byte {v}"
        );
        assert_eq!(
            store_as_srgb_u8(solo.g),
            0,
            "byte {v} in red must leave green at zero"
        );
        // …and, for the report below, how far the RAW-BYTE passthrough this
        // replaced would have landed from the authored value.
        let naive = store_as_srgb_u8(v as f64 / 255.0);
        let err = (naive as i32 - v as i32).abs();
        if err > worst.1 {
            worst = (v, err);
        }
    }
    // NON-VACUITY, inline: the retired rule written out. A straight
    // `channel/255.0` passthrough — what shipped before — must MISS, and miss
    // badly, or this law is grading nothing. The worst byte is reported so the
    // failure names the size of the defect rather than only its existence.
    assert!(
        worst.1 >= 60,
        "the raw-byte passthrough this law replaced is off by at most {} bytes \
         (worst at 0x{:02X}) — if that is small, the attachment is no longer \
         applying an sRGB encode and this law has stopped discriminating",
        worst.1,
        worst.0
    );
}

/// **ONE TRANSFER FUNCTION, NOT SIX.** Every colour that reaches the GPU as a
/// vertex/uniform value is linearised by a per-pipeline converter, and the
/// clear is now linearised too — by its own function, because `theme` cannot
/// depend on a render module. So pin them together: a second definition of a
/// transfer function is a second answer waiting to disagree, and the whole bug
/// this file exists for was one path that skipped the conversion while every
/// other path did it.
///
/// `selection::srgba_u8_to_linear` is the anchor because it is the only one of
/// the five already visible outside its own module. The tolerance is `f32`
/// epsilon-scale: the shader-side converters compute in `f32` and this one in
/// `f64` (`wgpu::Color` is `f64`), which is a difference in width, not in rule.
///
/// This one grades the FUNCTION and deliberately not the wiring — it stays green
/// if the clear site stops calling it, which is exactly what its two siblings
/// are for. Read the three together or none of them.
#[test]
fn the_clear_uses_the_same_srgb_transfer_function_as_the_shader_side_converters() {
    let _g = crate::testlock::serial();
    for v in 0u8..=255 {
        let mine = srgb_channel_to_linear(v);
        let theirs = crate::selection::srgba_u8_to_linear([v, v, v, 0xFF])[0] as f64;
        assert!(
            (mine - theirs).abs() <= 1e-6,
            "byte {v}: theme's clear converter says {mine} and \
             selection::srgba_u8_to_linear says {theirs} — the tree must carry \
             ONE sRGB EOTF"
        );
    }
}

/// **THE ROSTER, NAMED.** The per-byte law above already covers every world's
/// grounds, but this one reports the worlds by name and states, in the failure
/// message, how far each authored page was from what it drew — the fact
/// THEMES.md's "`ground == base_100` keeps the flat page column and the margin
/// floor one seamless plane" depends on, and the fact any future work reasoning
/// from an authored dark token needs to be able to trust.
///
/// Enrolment is `THEMES` itself, so a twenty-first world is swept the day it
/// lands.
#[test]
fn every_worlds_authored_page_is_the_page_its_clear_stores() {
    let _g = crate::testlock::serial();
    let mut moved = 0usize;
    for t in THEMES.iter() {
        let c = t.base_100;
        let clear = c.to_wgpu_clear();
        let got = Srgb::rgb(
            store_as_srgb_u8(clear.r),
            store_as_srgb_u8(clear.g),
            store_as_srgb_u8(clear.b),
        );
        assert_eq!(
            got.hex(),
            c.hex(),
            "{}: authored page {} stores as {} — the ground is not the token",
            t.name,
            c.hex(),
            got.hex()
        );
        // How much the raw-byte passthrough moved this world, for the count below.
        let naive = Srgb::rgb(
            store_as_srgb_u8(c.r as f64 / 255.0),
            store_as_srgb_u8(c.g as f64 / 255.0),
            store_as_srgb_u8(c.b as f64 / 255.0),
        );
        if naive != c {
            moved += 1;
        }
    }
    // Wagtail's page is pure black — the curve's fixed point — so it is the one
    // world the defect could never have touched. Everything else must be
    // shown to have been affected, or this law is being asked about a bug that
    // is no longer reachable.
    assert_eq!(
        moved,
        THEMES.len() - 1,
        "the raw-byte passthrough moved {moved} of {} worlds' pages; exactly one \
         (the pure-black one-bit page) is a fixed point of the sRGB curve, so \
         any other count means the roster or the attachment format changed \
         under this law",
        THEMES.len()
    );
}
