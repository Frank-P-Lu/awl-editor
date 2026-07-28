//! ITEM 86 — REAL-PIXEL proofs for `Background::Zigzag`, the repeating
//! chevron mark ground that replaced Quokka's dot grid AND Gumtree's
//! grass-bands field in one light-worlds taste round. The round's own brief:
//! the two worlds must NOT look like recolours of one asset — vary scale,
//! profile/direction, spacing, and contrast through world DATA read by the
//! ONE renderer (`shaders/background.wgsl`'s `pattern_coverage`, shader id
//! 7). Mirrors `backgrounds_item69.rs`'s pattern — drive `BackgroundPipeline`
//! directly (the purest reachable seam, no text/markdown involved) and read
//! the real GPU output back, reusing its `headless_dq` device helper rather
//! than re-deriving it, and item 89's `mark_field` differential oracle for the
//! one pixel measurement left here.
//!
//! **Post item-89 note:** this ground was reopened as a CORRECTNESS repair —
//! item 86's chevron repeated its teeth ALONG one travel line but never tiled
//! that line ACROSS the margin field, so a page margin carried a single
//! wandering stroke with large blank areas; and the fold that first tiled it
//! stacked rows every `period_px`, which left a blank LANE between rows
//! wherever the excursion did not span the period. The field laws (per-cell
//! occupancy SWEPT over viewport geometry, the no-blank-lane law, the abutment
//! theorem, row rhythm, height scaling, column exclusion, determinism) all live
//! in `backgrounds_item89.rs`; the per-world dials were re-derived there too
//! (Quokka 100/24 unchanged, Gumtree 170/60). What stays here is item 86's
//! DESIGN brief — the roster, and the four dials' authored distinctness.
//!
//! Per the project tripwire (the sidecar is a STATE oracle, never an
//! APPEARANCE oracle), every contrast/distinctness/column-exclusion claim
//! here is proven by PIXEL arithmetic over the rendered bytes.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use super::backgrounds_item69::headless_dq;
use crate::theme;

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
            theme::Background::Organic { .. } => "organic",
            theme::Background::WarpedGrid { .. } => "warped-grid",
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
    let all_distinct = a.0 != b.0 && a.1 != b.1 && a.2 != b.2 && a.3 != b.3;
    assert!(
        !all_distinct,
        "identical dials must NOT pass the distinctness check"
    );
}

/// THE DISTINCTNESS LAW (data half): every one of the four authored dials —
/// `period_px` (scale/spacing), `amplitude_px` (profile), `angle`
/// (direction), `density` (contrast) — differs between Quokka's and
/// Gumtree's `Zigzag`, and Gumtree's is the "broader and quieter" of the
/// pair per the round's own brief (broader spacing, lower contrast).
#[test]
fn zigzag_dials_are_measurably_distinct_between_quokka_and_gumtree() {
    let (
        theme::Background::Zigzag {
            period_px: qp,
            amplitude_px: qa,
            angle: qang,
            density: qd,
            ..
        },
        theme::Background::Zigzag {
            period_px: gp,
            amplitude_px: ga,
            angle: gang,
            density: gd,
            ..
        },
    ) = (theme::QUOKKA.background, theme::GUMTREE.background)
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

#[test]
fn quokka_alone_uses_horizontal_filled_zigzag_bands() {
    assert!(theme::QUOKKA.background.zigzag_banded());
    assert!(!theme::GUMTREE.background.zigzag_banded());
    assert_eq!(theme::QUOKKA.background.angle(), 0.0);
    assert_eq!(theme::GUMTREE.background.angle(), 0.26);
}

// ---------------------------------------------------------------------------
// REAL-PIXEL LAWS
// ---------------------------------------------------------------------------

// SUPERSEDED BY ITEM 89: this file's original column-exclusion law lived here.
// Its negative half (nothing paints inside the page column) was sound; its
// POSITIVE half was `margin_has_mark = true` if ANY single pixel of a strided
// margin scan differed from the two gradient endpoints — which one wandering
// chevron stroke satisfies trivially, and which is exactly why item 86's
// non-tiling field (60-95% of a tall margin blank) shipped green. Both halves
// now live, strengthened and differential, in `backgrounds_item89.rs`
// (`zigzag_contributes_zero_ink_inside_the_writing_column_on_both_worlds` +
// the 18-cell occupancy grid). The determinism law that followed it moved
// there too, widened to two canvas sizes.

/// THE DISTINCTNESS LAW (real-pixel half, CONTRAST): over the SAME canvas
/// geometry, Quokka's higher-`density` chevron field lays down BOLDER ink than
/// Gumtree's lower-`density` one — the real-GPU-output confirmation of the
/// data-level `density` inequality above (never trusted from the struct
/// literal alone — the Wagtail lesson). Measured through item 89's
/// DIFFERENTIAL oracle (`backgrounds_item89::mark_field`: the world rendered
/// as authored minus the same world with its mark coverage zeroed), so the
/// number really is the MARK's own peak deviation. The original form of this
/// law measured each pixel's distance from the nearer gradient ENDPOINT, which
/// on a two-tone gradient is dominated by the mid-gradient tone itself (up to
/// half the endpoint span — 24 on Gumtree, against a mark that only reaches
/// 11), i.e. it compared the two worlds' gradient spans as much as their
/// marks; the differential field has no such confound.
///
/// Peak deviation (not total marked-pixel AREA) is the right proxy for
/// "bolder/quieter": `line * density` peaks exactly AT `density` on a
/// chevron's own centerline regardless of the ribbon's width, so this isolates
/// the CONTRAST dial from the PROFILE dial's own (also authored, also
/// distinct) effect on ribbon thickness / marked area.
#[test]
fn quokka_zigzag_reads_higher_contrast_than_gumtrees_over_real_pixels() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping quokka_zigzag_reads_higher_contrast_than_gumtrees_over_real_pixels: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);

    // No page hole (col_w = 0): the WHOLE canvas is margin, the purest scan
    // surface for a peak-intensity measurement.
    let peak_ink = |bg: theme::Background| -> i32 {
        super::backgrounds_item89::mark_field(&device, &queue, bg, w, h, 0.0, 0.0)
            .into_iter()
            .max()
            .unwrap_or(0)
    };

    let quokka_peak = peak_ink(theme::QUOKKA.background);
    let gumtree_peak = peak_ink(theme::GUMTREE.background);
    assert!(quokka_peak > 0, "Quokka's zigzag must reach SOME real ink");
    assert!(
        gumtree_peak > 0,
        "Gumtree's zigzag must reach SOME real ink"
    );
    assert!(
        quokka_peak >= gumtree_peak + 8,
        "Quokka's louder zigzag (peak mark deviation {quokka_peak}) must retain a material \
         contrast lead over Gumtree ({gumtree_peak})"
    );
}

/// THE VISIBILITY FLOOR: Gumtree remains a quiet ground, but its mark cannot
/// return to the imperceptible pre-item-108 blend. This is deliberately a
/// differential real-pixel floor (the same world with density zeroed is
/// subtracted), across the narrow and generous page geometries represented in
/// the review dashboard. It pins authored visibility without changing any
/// shared Zigzag geometry or tint machinery.
fn gumtree_visibility_floor(field: &[i32]) -> i32 {
    field.iter().copied().max().unwrap_or(0)
}

#[test]
fn gumtree_zigzag_is_visibly_present_across_dashboard_geometries() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping gumtree_zigzag_is_visibly_present_across_dashboard_geometries: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    for (w, h, col_left, col_w) in [(900, 700, 125.0, 650.0), (1800, 1000, 350.0, 1100.0)] {
        let field = super::backgrounds_item89::mark_field(
            &device,
            &queue,
            theme::GUMTREE.background,
            w,
            h,
            col_left,
            col_w,
        );
        let peak = gumtree_visibility_floor(&field);
        assert!(
            peak >= 18,
            "Gumtree {w}x{h}: mark peak deviation {peak} must clear the visible-background floor 18"
        );
    }
}

#[test]
fn gumtree_visibility_floor_rejects_the_imperceptible_density_mutation() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let theme::Background::Zigzag {
        from,
        to,
        dir,
        tint,
        period_px,
        amplitude_px,
        angle,
        ..
    } = theme::GUMTREE.background
    else {
        unreachable!("Gumtree must remain a Zigzag world");
    };
    let imperceptible = theme::Background::Zigzag {
        from,
        to,
        dir,
        tint,
        period_px,
        amplitude_px,
        angle,
        density: 0.20,
        banded: false,
    };
    let field = super::backgrounds_item89::mark_field(
        &device,
        &queue,
        imperceptible,
        900,
        700,
        125.0,
        650.0,
    );
    let peak = gumtree_visibility_floor(&field);
    assert!(
        peak < 18,
        "mutation witness must remain below the visibility floor, got {peak}"
    );
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
        assert!(
            bg.amplitude_px() > 0.0,
            "{name}: amplitude_px must be positive"
        );
        assert!(
            (0.0..=1.0).contains(&bg.density()),
            "{name}: density must sit in [0,1]"
        );
    }
}
