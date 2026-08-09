//! The ambient wrap-continuity law — the owed, UNCONDITIONAL replacement for
//! `organic_ground.rs`'s deleted `organic_phase_moves_and_wraps_without_
//! a_catchup_jump`.
//!
//! **Why the old law was vacuous — the sharpest instance of this class found
//! in this repo, and the reason the pop shipped green.** It (a) called
//! `waves_drift_radians`, a function Organic never read — Organic computed
//! its own drift inline, `render/layers.rs`'s `is_organic()` arm — so it
//! guarded the wrong owner entirely; (b) it never applied the `0.73` term
//! that actually broke; and (c) what remained asserted `sin`/`cos`'s own
//! 2*pi-periodicity, a fact of trigonometry rather than of anything awl
//! authors — it could not fail for any value of any constant in this repo.
//!
//! **This law's shape is different on all three axes.** It sweeps EVERY
//! ambient ground whose per-frame term rides `background.wgsl`'s `Globals`
//! uniform across the shared clock's own wrap point
//! (`crate::lava::LAVA_LOOP_CYCLES`, where `crate::lava::advance_phase`'s
//! `rem_euclid` returns the phase to exactly `0.0`) — Bombora's WAVES drift
//! (`g.drift`, radians, owned by `crate::background::waves_drift_radians`)
//! and Bowerbird's ORGANIC companion breathe (`g.organic_phase`, raw cycles,
//! read directly by `organic_finds_rgb` — no Rust owner at all). It asserts
//! continuity in the value the SHADER actually evaluates — real GPU pixels
//! from the real `BackgroundPipeline`, via the direct-injection seam
//! `render_bg_ambient` — never a Rust mirror the shader may not call (the
//! exact substitution that made the old law guard nothing).
//!
//! Stars (`crate::stars::brightness`) and WarpedGrid's travel are
//! deliberately OUT of this sweep: a star's per-frame value is computed in
//! Rust and uploaded VERBATIM as the instance's own vertex color (no further
//! shader transform to go discontinuous in), already covered at that exact
//! seam by `stars::tests::twinkle_is_seamless_across_the_ambient_loop_wrap`;
//! WarpedGrid's travel is monotonic camera motion, never wrapped against
//! `LAVA_LOOP_CYCLES` at all (`docs/render.md`'s own note: "this one
//! travels"), so "continuity at the wrap" is not a claim that applies to it.

use super::bands_waves::{bg_desc_for, headless_dq, render_bg_ambient};
use crate::background::AmbientUpload;
use crate::theme;

const WRAP: f32 = crate::lava::LAVA_LOOP_CYCLES;

/// The wrap tolerance, in per-channel u8 units. NOT zero: `f32::consts::TAU`
/// is not bit-exact `2*pi`, so a real GPU `sin`/`cos` evaluated at a large
/// multiple of it (`waves_drift_radians(LAVA_LOOP_CYCLES)` lands near `TAU`
/// itself) differs from its value at `0.0` by a few ULPs — harmless noise
/// that a handful of boundary pixels can carry across an 8-bit quantization
/// edge after `smoothstep`/`mix`. `WRAP_TOLERANCE` is set to comfortably
/// clear that noise floor while sitting FAR under the historical defect's own
/// signature: the deleted `0.73` term jumped `cos` by 1.125333 NORMALISED
/// units — over 280 u8 levels — across the whole field at once, not a few
/// boundary pixels by a handful of levels.
const WRAP_TOLERANCE: i32 = 4;

/// Every per-channel delta between two same-sized frames stays within
/// [`WRAP_TOLERANCE`], AND the number of pixels carrying ANY nonzero delta at
/// all stays a small minority — floating-point noise near a wrap shows up as
/// a scattered handful of boundary pixels nudged by a couple of levels, never
/// a field-wide shift.
fn assert_wrap_is_continuous(a: &[[u8; 4]], b: &[[u8; 4]], label: &str) {
    assert_eq!(a.len(), b.len(), "{label}: frame size mismatch");
    let mut max_delta = 0i32;
    let mut nonzero = 0usize;
    for (pa, pb) in a.iter().zip(b.iter()) {
        let d = (0..3)
            .map(|k| (pa[k] as i32 - pb[k] as i32).abs())
            .max()
            .unwrap();
        max_delta = max_delta.max(d);
        if d > 0 {
            nonzero += 1;
        }
    }
    assert!(
        max_delta <= WRAP_TOLERANCE,
        "{label}: a pixel differs by {max_delta} channel levels between phase 0.0 and the \
         shared clock's own wrap point ({WRAP}) — over the {WRAP_TOLERANCE}-level \
         floating-point noise floor, i.e. a REAL discontinuity, not rounding: a \
         discontinuous drift-to-shader term pops one frame wide right at the loop seam"
    );
    assert!(
        nonzero < a.len() / 20,
        "{label}: {nonzero} of {} pixels differ at all between phase 0.0 and the wrap — floating \
         point noise near a wrap is a scattered few boundary pixels, not a fifth of the field",
        a.len()
    );
}

#[test]
fn every_ambient_consumers_shader_term_is_continuous_across_the_wrap() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping every_ambient_consumers_shader_term_is_continuous_across_the_wrap: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h) = (900u32, 600u32);

    // WAVES (Bombora) — `g.drift` is radians, pre-mapped on the host by the
    // real production function every frame actually calls.
    let waves_at = |phase: f32| {
        let drift = crate::background::waves_drift_radians(phase);
        render_bg_ambient(
            &device,
            &queue,
            bg_desc_for(theme::BOMBORA.background),
            w,
            h,
            0.0,
            0.0,
            AmbientUpload {
                drift,
                ..Default::default()
            },
            1.0,
        )
    };
    let waves_0 = waves_at(0.0);
    let waves_wrap = waves_at(WRAP);
    assert_wrap_is_continuous(&waves_0, &waves_wrap, "Bombora (waves)");

    // ORGANIC (Bowerbird) — `g.organic_phase` is raw CYCLES, read directly by
    // `organic_finds_rgb`; there is no Rust-side pre-transform to call at all.
    let organic_at = |phase: f32| {
        render_bg_ambient(
            &device,
            &queue,
            bg_desc_for(theme::BOWERBIRD.background),
            w,
            h,
            0.0,
            0.0,
            AmbientUpload {
                organic_phase: phase,
                ..Default::default()
            },
            1.0,
        )
    };
    let organic_0 = organic_at(0.0);
    let organic_wrap = organic_at(WRAP);
    assert_wrap_is_continuous(
        &organic_0,
        &organic_wrap,
        "Bowerbird (organic companion breathe)",
    );

    // Non-vacuity, the other direction: a genuinely MID-cycle phase must
    // differ from the settled frame for BOTH consumers, well past the wrap's
    // own noise floor — so "always identical" (e.g. an accidentally-inert
    // upload) cannot masquerade as "continuous".
    let mid = WRAP * 0.5;
    let waves_mid = waves_at(mid);
    let organic_mid = organic_at(mid);
    let big_delta = |a: &[[u8; 4]], b: &[[u8; 4]]| -> usize {
        a.iter()
            .zip(b.iter())
            .filter(|(pa, pb)| (0..3).any(|k| (pa[k] as i32 - pb[k] as i32).abs() > WRAP_TOLERANCE))
            .count()
    };
    assert!(
        big_delta(&waves_0, &waves_mid) > 50,
        "sanity: Bombora's drift term must actually move somewhere mid-cycle, well past the \
         wrap's own floating-point noise floor"
    );
    assert!(
        big_delta(&organic_0, &organic_mid) > 50,
        "sanity: Bowerbird's companion breathe must actually move somewhere mid-cycle, well \
         past the wrap's own floating-point noise floor"
    );
}
