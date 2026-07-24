//! ITEM 86 — REAL-PIXEL proofs for `Background::Zigzag`, the repeating
//! chevron mark ground that replaced Quokka's dot grid AND Gumtree's
//! grass-bands field in one light-worlds taste round. The round's own brief:
//! the two worlds must NOT look like recolours of one asset — vary scale,
//! profile/direction, spacing, and contrast through world DATA read by the
//! ONE renderer (`shaders/background.wgsl`'s `pattern_coverage`, shader id
//! 7). Mirrors `backgrounds_item69.rs`'s pattern — drive `BackgroundPipeline`
//! directly (the purest reachable seam, no text/markdown involved) and read
//! the real GPU output back, reusing its `headless_dq`/`bg_desc_for`/
//! `render_bg` trio rather than re-deriving them.
//!
//! Per the project tripwire (the sidecar is a STATE oracle, never an
//! APPEARANCE oracle), every contrast/distinctness/column-exclusion claim
//! here is proven by PIXEL arithmetic over the rendered bytes.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use crate::theme;
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};

// ---------------------------------------------------------------------------
// STRUCTURAL ROSTER — exhaustive, no wildcard (background-ground half of the
// round's required law; the card-cap half already lives in
// `card_texture_shape.rs::card_caps_are_flat_rectangular_for_every_world_but_quokka`).
// ---------------------------------------------------------------------------

/// EXHAUSTIVE, no-wildcard match over the closed [`theme::Background`] enum:
/// `Zigzag` ships on Quokka and Gumtree ALONE, every other world ships
/// something else. A newly added `Background` variant must extend this match
/// (compile error) before it can silently dodge the sweep.
#[test]
fn zigzag_ships_on_quokka_and_gumtree_alone_no_wildcard() {
    for t in theme::THEMES {
        let kind = match t.background {
            theme::Background::Gradient { .. } => "gradient",
            theme::Background::Dots { .. } => "dots",
            theme::Background::Starfield { .. } => "starfield",
            theme::Background::Pinstripe { .. } => "pinstripe",
            theme::Background::Stripes { .. } => "stripes",
            theme::Background::Lava { .. } => "lava",
            theme::Background::Bands { .. } => "bands",
            theme::Background::Waves { .. } => "waves",
            theme::Background::Zigzag { .. } => "zigzag",
        };
        match t.name {
            "Quokka" | "Gumtree" => {
                assert_eq!(kind, "zigzag", "{} must ship Background::Zigzag", t.name)
            }
            _ => assert_ne!(
                kind, "zigzag",
                "{} must NOT ship Background::Zigzag (item 86 is Quokka/Gumtree-only)",
                t.name
            ),
        }
    }
}

/// NON-VACUITY SELF-PROOF: the exact four-dial inequality check
/// [`zigzag_dials_are_measurably_distinct_between_quokka_and_gumtree`] below
/// uses, run here against a pair of DELIBERATELY IDENTICAL literals — proving
/// the law is capable of failing (a copy-pasted recolor would trip it), not
/// just capable of passing the real authored data.
#[test]
fn distinctness_check_fails_on_identical_dials_proving_it_is_non_vacuous() {
    let a = (50.0f32, 10.0f32, 0.95f32, 0.60f32);
    let b = a; // an identical copy — exactly the "recolor of one asset" bug.
    let all_distinct =
        a.0 != b.0 && a.1 != b.1 && a.2 != b.2 && a.3 != b.3;
    assert!(!all_distinct, "identical dials must NOT pass the distinctness check");
}

/// THE DISTINCTNESS LAW (data half): every one of the four authored dials —
/// `period_px` (scale/spacing), `amplitude_px` (profile), `angle`
/// (direction), `density` (contrast) — differs between Quokka's and
/// Gumtree's `Zigzag`, and Gumtree's is the "broader and quieter" of the
/// pair per the round's own brief (broader spacing, lower contrast).
#[test]
fn zigzag_dials_are_measurably_distinct_between_quokka_and_gumtree() {
    let (theme::Background::Zigzag {
        period_px: qp,
        amplitude_px: qa,
        angle: qang,
        density: qd,
        ..
    }, theme::Background::Zigzag {
        period_px: gp,
        amplitude_px: ga,
        angle: gang,
        density: gd,
        ..
    }) = (theme::QUOKKA.background, theme::GUMTREE.background)
    else {
        panic!("both Quokka and Gumtree must ship Background::Zigzag");
    };
    assert_ne!(qp, gp, "period_px (scale/spacing) must differ");
    assert_ne!(qa, ga, "amplitude_px (profile) must differ");
    assert_ne!(qang, gang, "angle (direction) must differ");
    assert_ne!(qd, gd, "density (contrast) must differ");
    assert!(gp > qp, "Gumtree's period must be BROADER than Quokka's");
    assert!(gd < qd, "Gumtree's density must be QUIETER than Quokka's");
}

// ---------------------------------------------------------------------------
// REAL-PIXEL LAWS
// ---------------------------------------------------------------------------

/// THE COLUMN-EXCLUSION LAW: the zigzag mark NEVER paints inside the page
/// column, for either world — every pixel inside `[col_left, col_left+col_w)`
/// stays the exact CLEAR color (the shader's own alpha-0 hole), while the
/// margins DO show real mark pixels (a sanity check that the render is
/// actually doing something, not a vacuously-empty pass). This is also the
/// ground's own "text stays legible" proof: the writing column the glyphs
/// render into is structurally untouched by this ground, on both worlds.
#[test]
fn zigzag_pattern_never_paints_inside_the_page_column_on_either_world() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_pattern_never_paints_inside_the_page_column_on_either_world: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let (col_left, col_w) = (350.0f32, 500.0f32);

    for (name, bg) in [
        ("Quokka", theme::QUOKKA.background),
        ("Gumtree", theme::GUMTREE.background),
    ] {
        let desc = bg_desc_for(bg);
        // The shader's OWN column hole is alpha-0 (the fragment returns
        // `vec4(0,0,0,0)` inside `[col_left, col_left+col_w)`, unconditionally,
        // BEFORE any per-shader branch runs) — so "never paints inside the
        // column" reduces to "every column pixel stays the alpha-0 clear".
        let pixels = render_bg(&device, &queue, desc, w, h, col_left, col_w);
        let (from, to) = (bg.from().rgba_bytes(), bg.to().rgba_bytes());
        let is_mark = |p: [u8; 4]| {
            let near = |c: [u8; 4]| (0..3).all(|k| (p[k] as i16 - c[k] as i16).abs() <= 2);
            !near(from) && !near(to)
        };
        // Straight-alpha blend at src-alpha 0 leaves the framebuffer's own
        // CLEAR value untouched (`result = dst`) — `render_bg` clears to
        // opaque black, so an untouched column pixel reads `[0,0,0,255]`,
        // NOT a literal `[0,0,0,0]` (that would be the shader's OWN write,
        // which alpha-0 blending never actually composites in).
        const CLEARED: [u8; 4] = [0, 0, 0, 255];
        let mut column_all_clear = true;
        let mut margin_has_mark = false;
        for y in (0..h).step_by(23) {
            for x in (0..w).step_by(7) {
                let idx = (y * w + x) as usize;
                let is_col = (x as f32) >= col_left && (x as f32) < col_left + col_w;
                if is_col {
                    if pixels[idx] != CLEARED {
                        column_all_clear = false;
                    }
                } else if is_mark(pixels[idx]) {
                    margin_has_mark = true;
                }
            }
        }
        assert!(column_all_clear, "{name}: a zigzag pixel leaked inside the page column");
        assert!(margin_has_mark, "{name}: the margin must actually show the chevron mark (sanity)");
    }
}

/// THE DISTINCTNESS LAW (real-pixel half, CONTRAST): over the SAME canvas
/// geometry, Quokka's higher-`density` chevron field reaches a HIGHER PEAK
/// deviation from its own base gradient than Gumtree's lower-`density` one —
/// the real-GPU-output confirmation of the data-level `density` inequality
/// above (never trusted from the struct literal alone — the Wagtail lesson).
/// Peak deviation (not total marked-pixel AREA) is the right proxy for
/// "bolder/quieter": `line * density` peaks exactly AT `density` at a
/// chevron's own centerline regardless of the stroke's width, so this
/// isolates the CONTRAST dial from the PROFILE dial's own (also authored,
/// also distinct) effect on stroke thickness / marked area.
#[test]
fn quokka_zigzag_reads_higher_contrast_than_gumtrees_over_real_pixels() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping quokka_zigzag_reads_higher_contrast_than_gumtrees_over_real_pixels: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);

    let peak_deviation = |bg: theme::Background| -> i32 {
        let desc = bg_desc_for(bg);
        // No page hole (col_w = 0): the WHOLE canvas is margin, the purest
        // scan surface for a peak-intensity measurement.
        let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0);
        let (from, to) = (bg.from().rgba_bytes(), bg.to().rgba_bytes());
        let dist_from_nearest_endpoint = |p: [u8; 4]| {
            let d = |c: [u8; 4]| (0..3).map(|k| (p[k] as i32 - c[k] as i32).abs()).sum::<i32>();
            d(from).min(d(to))
        };
        pixels.iter().map(|&p| dist_from_nearest_endpoint(p)).max().unwrap_or(0)
    };

    let quokka_peak = peak_deviation(theme::QUOKKA.background);
    let gumtree_peak = peak_deviation(theme::GUMTREE.background);
    assert!(quokka_peak > 0, "Quokka's zigzag must reach SOME real ink");
    assert!(gumtree_peak > 0, "Gumtree's zigzag must reach SOME real ink");
    assert!(
        quokka_peak > gumtree_peak,
        "Quokka's higher-density zigzag (peak deviation {quokka_peak}) must reach a HIGHER \
         peak contrast than Gumtree's lower-density one (peak deviation {gumtree_peak})"
    );
}

/// DETERMINISM: two independent renders of the SAME desc are byte-for-byte
/// identical — no clock, no randomness (the pipeline's `Globals` carries no
/// time uniform at all), the same static-ground promise every other margin
/// pattern already holds.
#[test]
fn zigzag_renders_byte_identically_across_two_independent_draws() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_renders_byte_identically_across_two_independent_draws: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in [
        ("Quokka", theme::QUOKKA.background),
        ("Gumtree", theme::GUMTREE.background),
    ] {
        let desc = bg_desc_for(bg);
        let a = render_bg(&device, &queue, desc, 900, 600, 200.0, 400.0);
        let b = render_bg(&device, &queue, desc, 900, 600, 200.0, 400.0);
        assert_eq!(a, b, "{name}: two draws of the identical desc diverged");
    }
}

/// SANE, POSITIVE DIALS: both worlds' `period_px`/`amplitude_px` are finite
/// and strictly positive (a zero/negative period would divide-by-zero-adjacent
/// degenerate the shader's `fract(rx / period)`, guarded by `max(.., 1.0)`
/// there, but the AUTHORED data itself should never rely on that floor), and
/// `density` sits in the documented `[0,1]` contrast range.
#[test]
fn zigzag_dials_are_sane_and_positive_on_both_worlds() {
    for (name, bg) in [
        ("Quokka", theme::QUOKKA.background),
        ("Gumtree", theme::GUMTREE.background),
    ] {
        assert!(bg.period_px() > 0.0, "{name}: period_px must be positive");
        assert!(bg.amplitude_px() > 0.0, "{name}: amplitude_px must be positive");
        assert!((0.0..=1.0).contains(&bg.density()), "{name}: density must sit in [0,1]");
    }
}
