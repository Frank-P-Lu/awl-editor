//! Shader-source contracts for the spelling-wave finish and the shared
//! zero-amplitude writing-nit path.

/// The wave's ends are an OPACITY fade, never a geometric one. An amplitude
/// taper reads as a mark that starts flat and swells, which is a different
/// mark; the shipped one is one continuous ripple that simply fades out.
///
/// This is the source-level half of the contract — the pixels are pinned by
/// `render::tests::nits::spell_squiggle_keeps_full_amplitude_at_its_ends`,
/// which is the law that fails when the geometry tapers again.
#[test]
fn spelling_wave_ends_fade_by_opacity_and_never_by_amplitude() {
    let shader = include_str!("../../shaders/spellunderline.wgsl");
    assert!(
        !shader.contains("taper"),
        "the wave's amplitude must reach both ends undiminished — no geometric taper"
    );
    let fade = shader
        .find("a = a * smoothstep(left - 0.5")
        .expect("the end fade remains");
    assert!(
        shader.contains("a = a * (1.0 - smoothstep(right - edge, right + 0.5, in.px.x))"),
        "both ends fade, not just the left one"
    );
    // The fade is SHARED. A branch would re-split the two paths, and the wavy
    // side is the one that would silently lose its soft finish.
    assert!(
        !shader[..fade].contains("if (in.amp == 0.0)"),
        "the end fade is shared by the wavy spelling mark and the straight nit, \
         not gated onto the zero-amplitude path"
    );
}
