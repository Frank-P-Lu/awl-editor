//! Shader-source contracts for the spelling-wave's endpoint taper and the
//! shared zero-amplitude writing-nit path: a hard-chopped end reads fine at
//! normal size but abruptly clipped when the underline is enlarged, so the
//! STROKE WIDTH eases toward a small rounded tip, leaving the sine curve's
//! own amplitude/period/phase and the middle stroke untouched.

/// The taper eases STROKE WIDTH toward the tip; the wave's own centerline
/// (amplitude) is untouched by it. `wave_y`'s formula appears exactly twice —
/// once for the fragment's own phase, once for the round cap's endpoint
/// anchor phase — and both are the SAME full-amplitude expression, never
/// scaled down by the taper's `eased`/`u` terms. An amplitude taper reads as
/// a mark that starts flat and swells, a different (and previously
/// forbidden) mark from the one approved here.
///
/// This is the source-level half of the contract — the pixels are pinned by
/// `render::tests::nits::spell_squiggle_keeps_full_amplitude_at_its_ends`
/// (unchanged: the curve itself still reaches full amplitude everywhere) and
/// the new width-taper + presence-floor laws in that same module.
#[test]
fn squiggle_tip_taper_moves_stroke_width_never_the_wave_centerline() {
    let shader = include_str!("../../shaders/spellunderline.wgsl");
    assert_eq!(
        shader.matches("in.center.y - in.amp * cos(").count(),
        2,
        "expected exactly the two full-amplitude curve evaluations (the fragment's own phase, \
         and the round cap's endpoint anchor) — a different count means the taper touched \
         amplitude instead of stroke width"
    );
    assert!(
        !shader.contains("amp * eased")
            && !shader.contains("eased * in.amp")
            && !shader.contains("amp * u")
            && !shader.contains("u * in.amp"),
        "the taper's ease variable must never multiply amplitude — width only"
    );
    assert!(
        shader.contains("half_w = mix(tip_half, half_w, eased)"),
        "the taper eases the STROKE WIDTH (half_w), not the wave amplitude"
    );
}

/// The tip never fades to nothing: `tip_half` is a FLOORED fraction of the
/// full stroke, not a fade target that can reach zero — a presence floor, not
/// a taste knob. And the finish stays a pure geometry change: no blur, and
/// (checked below) no opacity ramp layered on top of the width taper.
#[test]
fn squiggle_tip_has_a_nonzero_presence_floor_and_no_blur() {
    let shader = include_str!("../../shaders/spellunderline.wgsl");
    assert!(
        shader.contains("max(half_w * TIP_FRACTION, TIP_FLOOR_PX)"),
        "the tip half-width is floored, so it can never taper to invisible"
    );
    assert!(
        !shader.to_lowercase().contains("blur"),
        "the squiggle finish is a geometric taper, not a blur"
    );
}

/// The zero-amplitude writing-nit keeps its ORIGINAL two-sided opacity fade,
/// gated OFF the new wavy-only taper path and otherwise byte-identical to
/// before this taper existed — the wavy squiggle is the one scoped to
/// change; the nit's flat tick is a deliberate, separate design choice left
/// alone (it has no crest to taper toward, and the taper math assumes a
/// period).
#[test]
fn nit_end_fade_stays_gated_to_the_zero_amplitude_path_and_is_unmodified() {
    let shader = include_str!("../../shaders/spellunderline.wgsl");
    let gate = shader
        .find("if (in.amp <= 0.0) {")
        .expect("the nit's opacity fade is gated on zero (or negative) amplitude");
    let left_fade = shader
        .find("a = a * smoothstep(left - 0.5, left + edge, in.px.x);")
        .expect("the left-edge fade remains, byte-identical");
    let right_fade = shader
        .find("a = a * (1.0 - smoothstep(right - edge, right + 0.5, in.px.x));")
        .expect("the right-edge fade remains, byte-identical");
    assert!(
        left_fade > gate && right_fade > gate,
        "both fades must sit inside the zero-amplitude gate, not the shared/unconditional path \
         the wavy squiggle also runs"
    );
    // No OTHER alpha multiply exists ahead of the gate: the squiggle's own
    // (amp > 0) path never re-applies an opacity fade on top of its width
    // taper — CLAUDE.md's hazard is exactly a treatment that fades ink at its
    // ends, so the taper must be the ONLY endpoint treatment the wavy mark
    // gets.
    let before_gate = &shader[..gate];
    assert!(
        !before_gate.contains("a = a *"),
        "the squiggle path must not carry its own opacity fade on top of the width taper — an \
         end that fades AND thins would satisfy a contrast floor by disappearing rather than by \
         being legible"
    );
}
