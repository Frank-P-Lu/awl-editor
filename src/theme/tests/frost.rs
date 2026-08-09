use super::super::*;

/// THE FROST PILL CONTRAST LAW.
/// The shipped headed-doc treatment is per-entry FROST pills: behind each
/// outline entry the lava renders a softened (blurred SMOOTH-field) sample
/// value-DIMMED toward the flat ground (`crate::lava::frost_pixel` /
/// `crate::lava::FROST_DIM`), while the lamp stays fully alive between and around
/// the pills. This law proves the DIM outline ink stays legible over that frosted
/// pill ground at EVERY animation phase, in two halves:
///
/// (1) PHASE SWEEP (64 phases × a pill-region grid in the left margin): the ACTUAL
///     frosted pixel — the pure-Rust shader mirror `frost_field` → `frost_pixel` —
///     clears the ink-ladder floors against the outline's inks: the `faint` (every
///     non-current) entry at redmean >= 100, the `base_content` current row at >=
///     150. Proven over COMPOSITED PIXELS, never sidecar state (the Wagtail
///     invisible-picker-row lesson). WITNESSED non-vacuous: some sampled frost
///     pixel genuinely differs from the flat ground (the lamp reads THROUGH the
///     frost — it is a softened lamp, not the old flat carve).
///
/// (2) PHASE-FREE WORST BOUND: the brightest a frost pill can ever reach is
///     `mix(blob_hi, ground, FROST_DIM)` (the softened field bounded by blob_hi,
///     then dimmed) — proving the ink clears THAT covers every phase by
///     construction, a belt-and-braces guard the sweep can't miss.
///
/// The `Background` match is NO-WILDCARD: a future ground variant must decide its
/// frost story here or fail to compile.
#[test]
fn outline_frost_pills_keep_ink_contrast_on_every_lava_world() {
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    // Representative page geometry (the 1600x1000 gallery canvas). Frost pills sit
    // in the LEFT margin (x well below col_left), hugging the outline entries.
    let vp = (1600.0f32, 1000.0f32);
    for t in THEMES.iter() {
        // NO-WILDCARD: a future ground variant must decide its frost story here.
        let (ground, blob_lo, blob_hi) = match t.background {
            // Every non-lava ground carries no lava — no frost.
            Background::Gradient { .. }
            | Background::Dots { .. }
            | Background::Pinstripe { .. }
            | Background::Stripes { .. }
            | Background::Bands { .. }
            | Background::Waves { .. }
            | Background::Zigzag { .. }
            | Background::Organic { .. }
            | Background::Deckle { .. }
            // A moving ground, but not a LAVA one — no frost seeds.
            | Background::WarpedGrid { .. } => continue,
            Background::Lava {
                ground,
                blob_lo,
                blob_hi,
                ..
            } => (ground, blob_lo, blob_hi),
        };
        let blur = crate::lava::FROST_BLUR_PX;
        let dim = crate::lava::FROST_DIM;
        assert_eq!(
            ground, t.base_100,
            "{}: frost ground must be base_100",
            t.name
        );

        // (1) Phase sweep × a pill-region grid: the ACTUAL frost pixel clears the
        //     ink-ladder floors, and the lamp genuinely reads through the frost.
        let mut witnessed_alive = false;
        for step in 0..64 {
            let phase = step as f32 * crate::lava::LAVA_LOOP_CYCLES / 64.0;
            // Left-margin pill band: x below the column, y across the outline rows.
            for xi in 0..24 {
                let x = 80.0 + (270.0 - 80.0) * (xi as f32 + 0.5) / 24.0;
                for y in [150.0, 320.0, 500.0, 680.0, 850.0] {
                    let field = crate::lava::frost_field(
                        (x, y),
                        vp,
                        &crate::lava::BACKDROP_BLOBS,
                        phase,
                        blur,
                    );
                    let px = crate::lava::frost_pixel(field, ground, blob_lo, blob_hi, dim);
                    let dimd = redmean(t.faint, px);
                    assert!(
                        dimd >= 100.0,
                        "{}: faint outline ink only {dimd:.1} redmean from the frost pill \
                         at x={x} y={y} phase={phase} (under the ink-ladder floor)",
                        t.name
                    );
                    let lit = redmean(t.base_content, px);
                    assert!(
                        lit >= 150.0,
                        "{}: the current outline row only {lit:.1} redmean from the frost \
                         pill at x={x} y={y} phase={phase}",
                        t.name
                    );
                    if (px.r, px.g, px.b) != (ground.r, ground.g, ground.b) {
                        witnessed_alive = true;
                    }
                }
            }
        }
        assert!(
            witnessed_alive,
            "{}: no sampled frost pixel differs from the flat ground — the frost is a \
             vacuous flat carve, not a softened LIVING lamp",
            t.name
        );

        // (2) PHASE-FREE WORST BOUND: mix(blob_hi, ground, dim) is the brightest a
        //     frost pill can reach; the ink clears the floors against it, so every
        //     phase is covered by construction.
        let worst = crate::lava::frost_pixel(1.0, ground, blob_lo, blob_hi, dim);
        assert!(
            redmean(t.faint, worst) >= 100.0,
            "{}: faint ink only {:.1} redmean from the WORST frost pill (phase-free bound)",
            t.name,
            redmean(t.faint, worst)
        );
        assert!(
            redmean(t.base_content, worst) >= 150.0,
            "{}: current row only {:.1} redmean from the worst frost pill",
            t.name,
            redmean(t.base_content, worst)
        );
    }
}

/// THE GUTTER FROST PILL CONTRAST LAW. The bottom-left page-mode
/// GUTTER (`TextPipeline::prepare_gutter` — the filename/project stack) must
/// NOT hard-carve its corner out of the lava mask: that drops the band to the
/// flat, DARKEST page ground (`base_100`) — an ugly geometric dark pocket
/// beside the much lighter writing column and below the margin's own blob
/// peaks (worst on Firetail, ground lum ~12 vs column ~60). It rides the SAME
/// organic FROST FIELD the outline does (`TextPipeline::gutter_frost_seeds` →
/// `prepare_lava_layer`'s `seeds`): the lamp renders SOFTENED (a blurred
/// SMOOTH-field sample, `crate::lava::frost_field`) and value-DIMMED toward the
/// flat ground (`crate::lava::frost_pixel` / `FROST_DIM`), so the dim gutter ink
/// keeps its contrast while the lamp reads THROUGH — a warm whisper, not a dead
/// flat rectangle. Four halves:
///
/// (1) LEGIBILITY over the FROST pill at EVERY phase (64 phases × an in-pill grid):
///     the ACTUAL frosted pixel (the pure-Rust shader mirror `frost_field` →
///     `frost_pixel`) clears the ink-ladder floors against the gutter's own inks —
///     the `faint` project line at redmean >= 100, the `muted` filename line at
///     >= 150. Proven over COMPOSITED PIXELS, never sidecar state (the Wagtail
///     invisible-picker-row lesson).
///
/// (2) THE DARK REGION IS FIXED — the lamp reads THROUGH: WITNESSED non-vacuous —
///     some sampled frost pixel genuinely differs from the flat ground, so the old
///     dead-flat dark pocket is gone (a softened living lamp, not a carve).
///
/// (3) PHASE-FREE WORST BOUND: the brightest a pill can ever reach is
///     `frost_pixel(1.0, ..)` = `mix(blob_hi, ground, FROST_DIM)`; the ink clears
///     the floors against THAT, so every phase is covered by construction.
///
/// (4) THE FROST IS LOCAL — both margins keep their lamp: the organic coverage
///     (`frost_coverage`) is solid OVER the gutter seed's ink and exactly 0 far
///     from every seed (the left margin high above the band, the whole right
///     margin), so nothing carves and the rest of both margins stay their live
///     lamp. The gutter seed geometry is pinned at the render seam by
///     `render::tests::outline::gutter_frost_seeds_follow_gutter_visibility`.
///
/// The `Background` match is NO-WILDCARD: a future ground variant must decide its
/// frost story here or fail to compile. A static-ground world carries no lava, so
/// it `continue`s — no frost, byte-identical (the unaffected-worlds guarantee).
fn frost_redmean(a: Srgb, b: Srgb) -> f32 {
    let rbar = (a.r as f32 + b.r as f32) * 0.5;
    let (dr, dg, db) = (
        a.r as f32 - b.r as f32,
        a.g as f32 - b.g as f32,
        a.b as f32 - b.b as f32,
    );
    let squared =
        (2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db;
    squared.sqrt()
}

#[test]
fn gutter_frost_pill_keeps_ink_contrast_on_every_lava_world() {
    let vp = (1600.0f32, 1000.0f32);
    let gutter_seed = [40.0f32, 250.0, 930.0, 40.0];
    for t in THEMES.iter() {
        let (ground, blob_lo, blob_hi) = match t.background {
            Background::Gradient { .. }
            | Background::Dots { .. }
            | Background::Pinstripe { .. }
            | Background::Stripes { .. }
            | Background::Bands { .. }
            | Background::Waves { .. }
            | Background::Zigzag { .. }
            | Background::Organic { .. }
            | Background::Deckle { .. }
            | Background::WarpedGrid { .. } => continue,
            Background::Lava {
                ground,
                blob_lo,
                blob_hi,
                ..
            } => (ground, blob_lo, blob_hi),
        };
        let blur = crate::lava::FROST_BLUR_PX;
        let dim = crate::lava::FROST_DIM;
        assert_eq!(
            ground, t.base_100,
            "{}: frost ground must be base_100",
            t.name
        );

        // (1)+(2) Phase sweep × in-pill grid: the ACTUAL frost pixel clears the
        //         gutter ink floors, AND the lamp genuinely reads through the frost
        //         (the dark pocket is gone).
        let mut witnessed_alive = false;
        for step in 0..64 {
            let phase = step as f32 * crate::lava::LAVA_LOOP_CYCLES / 64.0;
            for xi in 0..16 {
                // x strictly INSIDE the pill, past its right-face feather.
                let x = 12.0 + (235.0 - 12.0) * (xi as f32 + 0.5) / 16.0;
                for y in [860.0, 900.0, 940.0, 980.0] {
                    let field = crate::lava::frost_field(
                        (x, y),
                        vp,
                        &crate::lava::BACKDROP_BLOBS,
                        phase,
                        blur,
                    );
                    let px = crate::lava::frost_pixel(field, ground, blob_lo, blob_hi, dim);
                    let project = frost_redmean(t.faint, px);
                    assert!(
                        project >= 100.0,
                        "{}: the gutter's faint project line only {project:.1} redmean from \
                         the frost pill at x={x} y={y} phase={phase} (under the ink-ladder floor)",
                        t.name
                    );
                    let name = frost_redmean(t.muted, px);
                    assert!(
                        name >= 150.0,
                        "{}: the gutter's muted filename only {name:.1} redmean from the frost \
                         pill at x={x} y={y} phase={phase}",
                        t.name
                    );
                    if (px.r, px.g, px.b) != (ground.r, ground.g, ground.b) {
                        witnessed_alive = true;
                    }
                }
            }
        }
        assert!(
            witnessed_alive,
            "{}: no sampled gutter frost pixel differs from the flat ground — the dark pocket \
             would still be a dead-flat carve, not a softened LIVING lamp",
            t.name
        );

        // PHASE-FREE WORST BOUND: the brightest pill phase clears the ink floors
        //     against it, so every phase is covered by construction.
        let worst = crate::lava::frost_pixel(1.0, ground, blob_lo, blob_hi, dim);
        assert!(
            frost_redmean(t.faint, worst) >= 100.0,
            "{}: faint project ink only {:.1} redmean from the WORST gutter frost pill \
             (phase-free bound)",
            t.name,
            frost_redmean(t.faint, worst)
        );
        assert!(
            frost_redmean(t.muted, worst) >= 150.0,
            "{}: muted filename ink only {:.1} redmean from the worst gutter frost pill",
            t.name,
            frost_redmean(t.muted, worst)
        );

        // THE FROST IS LOCAL: organic coverage is solid over the gutter seed's
        //     ink (non-vacuous) and exactly
        //     zero far from every seed (the left margin high above the band, and the
        //     whole right margin), so nothing is carved and the rest of both margins
        //     are untouched. `frost_coverage` sums the seed halos and thresholds them.
        assert!(
            crate::lava::frost_coverage(120.0, 930.0, &[gutter_seed]) > 0.99,
            "{}: the gutter seed does not frost its own ink (vacuous)",
            t.name
        );
        for (x, y) in [
            (150.0, 400.0),  // left margin, far above the band
            (200.0, 200.0),  // left margin, far above the band
            (1320.0, 930.0), // right margin, at the band's y
            (1560.0, 970.0), // right margin, deep bottom
        ] {
            assert_eq!(
                crate::lava::frost_coverage(x, y, &[gutter_seed]),
                0.0,
                "{}: frost leaked far from the gutter seed at x={x} y={y} \
                 (not local — a margin lost its lamp)",
                t.name
            );
        }
    }
}

/// Frost has one renderer-owned recipe. Its three constants remain well formed;
/// the lava-background gate is the only capability axis.
#[test]
fn frost_recipe_is_one_renderer_owned_lava_recipe() {
    assert!((0.0..=1.0).contains(&crate::lava::FROST_DIM));
    assert!(std::hint::black_box(crate::lava::FROST_BLUR_PX) > 0.0);
    assert!(std::hint::black_box(crate::lava::FROST_FEATHER_PX) >= 0.0);
    assert!(THEMES.iter().any(|t| t.background.is_lava()));
}

/// THE FOLD-AFFORDANCE CAPABILITY law (mirrors
/// `frost_recipe_is_a_per_world_capability_defaulting_to_the_shipped_lava_
/// values` immediately above — same shape, same reasoning): [`model::FoldAfford`]
/// is data, never a per-world code path.
///
/// (1) DEFAULT IS INERT: `RenderCaps::DEFAULT.fold_afford` is `FoldAfford::
///     DEFAULT` (`0.0`/`0.0`) — the bare `faint`/`muted` ladder rung.
/// (2) WELL-FORMED PER WORLD: every world's two lifts sit in `[0.0, 1.0]` (a
///     [`Srgb::lerp`] factor — out of range is meaningless, not merely ugly).
/// (3) ONLY A LAVA WORLD DIALS OFF-DEFAULT: a non-lava world has no glow-lit
///     column to compensate for, so it MUST stay at `FoldAfford::DEFAULT` — a
///     static-ground world quietly picking up a lift would be undetectable
///     drift, not a conscious taste call. At least one lava world DOES dial
///     off-default (the capability has a live consumer, not vacuous data).
#[test]
fn fold_afford_is_a_per_world_capability_inert_off_the_lava_worlds() {
    use crate::theme::FoldAfford;
    assert_eq!(
        RenderCaps::DEFAULT.fold_afford,
        FoldAfford::DEFAULT,
        "the DEFAULT caps carry the inert (0.0/0.0) fold-afford lift"
    );
    let mut saw_dialed = false;
    for t in THEMES.iter() {
        let f = t.render_caps.fold_afford;
        assert!(
            (0.0..=1.0).contains(&f.chevron_lift),
            "{}: fold_afford.chevron_lift {} out of [0,1]",
            t.name,
            f.chevron_lift
        );
        assert!(
            (0.0..=1.0).contains(&f.tail_lift),
            "{}: fold_afford.tail_lift {} out of [0,1]",
            t.name,
            f.tail_lift
        );
        if t.background.is_lava() {
            if f != FoldAfford::DEFAULT {
                saw_dialed = true;
            }
        } else {
            assert_eq!(
                f,
                FoldAfford::DEFAULT,
                "{}: a non-lava world has no glow-lit column to compensate for — \
                 fold_afford must stay the inert default",
                t.name
            );
        }
    }
    assert!(
        saw_dialed,
        "a lava world dials its own fold-afford lift (the capability has a live consumer)"
    );
}
