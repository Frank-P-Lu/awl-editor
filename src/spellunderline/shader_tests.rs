//! Shader-source contracts for the spelling-wave finish and the shared
//! zero-amplitude writing-nit path.

#[test]
fn spelling_wave_tapers_geometry_and_reserves_opacity_fade_for_straight_nits() {
    let shader = include_str!("../../shaders/spellunderline.wgsl");
    assert!(
        shader.contains("taper = clamp(min(from_left, from_right) / half_cycle"),
        "both spelling endpoints must taper geometrically over a half-cycle"
    );
    assert!(
        shader.contains("let beyond_end = max(max(left - in.px.x, in.px.x - right), 0.0)"),
        "finite spelling curves need endpoint distance for rounded caps"
    );
    let fade = shader
        .find("a = a * smoothstep(left - 0.5")
        .expect("straight nit fade remains");
    let straight = shader
        .find("if (in.amp == 0.0)")
        .expect("straight nit branch remains");
    assert!(
        fade > straight,
        "opacity fading is enrolled only for the byte-identical zero-amplitude nit path"
    );
}
