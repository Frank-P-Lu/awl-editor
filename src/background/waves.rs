//! Waves' phase-drift owner — the shared ambient clock's third
//! consumer, plus the shader-mirror the drift laws are stated against.
//!
//! The drift is a self-contained sub-vocabulary (one runtime
//! function, one env knob, one test mirror) with a single call site in
//! `render/layers.rs`, and it shares nothing with the pipeline's wgpu plumbing.
// Bombora's wave-tier boundaries ride a single scalar DRIFT (radians),
// uploaded through the DEDICATED `Globals.drift` slot (NOT `params`: Zigzag owns
// all four `params` slots; see that field's doc). The two boundary
// curves drift with EQUAL MAGNITUDE and OPPOSITE SIGN: the top/middle
// boundary advances by `+drift`, the middle/bottom boundary by `-drift`. A
// SAME-SIGN drift on both curves is mathematically an EXACT rigid horizontal
// translation of the whole three-tier field (`sin(x*F+P+d)` for both curves
// is identical to evaluating the undrifted field at `x + d/F` — the field's
// shape literally never changes, only its position) — a "one sheet" slide
// where every tier, middle included, shares IDENTICAL motion, precisely the
// outcome the background must not produce. The opposite-sign choice is the only
// one that breaks that rigid-translation identity: each OUTER tier (top,
// bottom) is bounded by exactly one of the two curves and sweeps with that
// curve's own sign, while the MIDDLE tier — bounded by BOTH, one advancing
// and one retarding — visibly shears/breathes counter to them, so the sea
// reads as independently layered swells rather than a sheet sliding behind
// the margin. `WAVE_DRIFT_CYCLES` is an INTEGER (the twinkling-stars'
// "integer cycles per ambient loop" law, THEMES.md's ambient-stars section):
// the drift completes an EXACT number of full turns over one shared-clock
// loop (`crate::lava::LAVA_LOOP_CYCLES`), so it meets its own endpoint
// exactly where the clock wraps — seamless, no pop. `1.0` is the slowest
// non-zero integer choice (one full 2*pi sweep — one WAVE wavelength of
// crest travel — over the ~67s loop), matching "very slow, almost
// imperceptible." Pure; MUST match `shaders/background.wgsl`'s own `drift`
// read off `g.drift` and its `waves_rgb`'s
// `WAVE_AMP`/`WAVE_FREQ`/`WAVE_PHASE_1`/`WAVE_PHASE_2`.
//
// The four shape constants below, and the
// [`waves_boundaries`] mirror that reads them, exist ONLY so those tier-geometry
// laws can be unit-tested without a GPU — the shipping renderer reads the
// WGSL's own copies (`shaders/background.wgsl`'s `waves_rgb`), never these. They
// are therefore `#[cfg(test)]`-gated and module-PRIVATE (no cross-module test
// calls them; `render/tests/bands_waves.rs` only mentions `WAVE_AMP` in a
// comment) rather than carrying an `allow(dead_code)` that would hide a genuinely
// dead constant. `WAVE_DRIFT_CYCLES` stays ungated: it feeds
// the RUNTIME [`waves_drift_radians`].
#[cfg(test)]
pub(super) const WAVE_AMP: f32 = 22.0;
#[cfg(test)]
pub(super) const WAVE_FREQ: f32 = 0.024166097;
#[cfg(test)]
pub(super) const WAVE_PHASE_1: f32 = 0.0;
#[cfg(test)]
pub(super) const WAVE_PHASE_2: f32 = 2.4;
const WAVE_DRIFT_CYCLES: f32 = 1.0;

/// The WAVES drift, in radians, for the shared ambient `phase` (cycles,
/// `[0, LAVA_LOOP_CYCLES)`) — `0.0` at `phase == 0.0` (the frozen/settled/
/// headless-capture phase, so a theme crossing INTO Bombora, and every
/// headless capture, renders the static composition). Pure.
/// See the module doc above for the seamless-wrap derivation.
pub(crate) fn waves_drift_radians(phase: f32) -> f32 {
    phase * std::f32::consts::TAU * WAVE_DRIFT_CYCLES / crate::lava::LAVA_LOOP_CYCLES
}

// The dev-only gallery knob (AWL_WAVES_PHASE=<f32>): mirrors `AWL_LAVA`/
// `AWL_STARS_PHASE` exactly (read once, memoized, a total no-op unless set —
// a headless capture never ticks the clock, so this never touches
// determinism there). Drives BOTH consumers of `waves_render_phase` — Bombora's
// wave drift and Bowerbird's companion value-breathe (the ground itself no
// longer translates) — one shared clock, one knob. Lets a gallery/before-after
// shot reach a real mid-cycle composition.
fn parse_waves_phase(raw: &str) -> Option<f32> {
    let p: f32 = raw.trim().parse().ok()?;
    p.is_finite().then_some(p)
}

/// `AWL_WAVES_PHASE`'s parsed value, or `None` (every normal + headless run).
/// Consumed by `TextPipeline::waves_render_phase` (env wins outright).
pub(crate) fn env_phase() -> Option<f32> {
    static ONCE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_WAVES_PHASE")
            .ok()
            .as_deref()
            .and_then(parse_waves_phase)
    })
}

/// The Rust MIRROR of `shaders/background.wgsl`'s `waves_rgb` boundary math —
/// the top/middle boundary `b1` (top third of the viewport height, plus the
/// scallop sine, phase-ADVANCED by `drift`) and the middle/bottom boundary
/// `b2` (bottom third, phase-RETARDED by `drift` — the opposite sign).
/// `viewport_h` in px; returns `(b1, b2)` in px. MUST stay in lockstep with
/// the shader; unit-tested here without a GPU (the `lava.rs`/`dither.rs`
/// shader-mirror idiom). Test-only and module-private — the runtime
/// path reads the WGSL's own copy of this math; see the scope note above the
/// `WAVE_*` constants.
#[cfg(test)]
pub(super) fn waves_boundaries(x: f32, viewport_h: f32, drift: f32) -> (f32, f32) {
    let b1 = viewport_h * (1.0 / 3.0) + WAVE_AMP * (x * WAVE_FREQ + WAVE_PHASE_1 + drift).sin();
    let b2 = viewport_h * (2.0 / 3.0) + WAVE_AMP * (x * WAVE_FREQ + WAVE_PHASE_2 - drift).sin();
    (b1, b2)
}
