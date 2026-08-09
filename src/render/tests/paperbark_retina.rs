//! ITEM 201 — Paperbark's Deckle ground on a real Retina display, restored.
//!
//! **The regression.** Item 186 made every ground's authored composition
//! LOGICAL, so a matched-logical 1x/2x pair now draws the identical
//! composition — correct, and item 186's own roster sweep
//! (`ground_space.rs`) proves it holds for Paperbark exactly as it
//! does for every other world. But `period_px`/`wander_px` were never
//! misclassified: item 158 tuned their NUMBERS (94.0 / 13.0) by eye against
//! the PRE-186 convention, where they were consumed as PHYSICAL pixels. On
//! whatever display that live approval happened on — almost certainly the
//! author's own Retina Mac, given item 158's note that the face gate closed
//! after "the real release-GPU Room capture" — the composition actually
//! judged was HALF those numbers (a real device divides a physical-pixel
//! quantity by its own ratio for free). Item 186 stopped that free halving
//! without retuning the dial, so the shipped 2x render went from the
//! approved ~47-logical-px lane pitch to the un-halved ~94 — half the lane
//! density, read as "unclear" by the user. **This is tuning debt exposed by
//! the migration, not a misclassification**: `theme::ground_space`'s
//! `DECKLE_STRATA` table is correct exactly as written, and this item does
//! not touch it, the shader, or the shared composition/sampling mechanism —
//! only Paperbark's own two numbers, halved to restore what was approved
//! (`src/theme/worlds.rs`'s `PAPERBARK` literal, `period_px: 47.0,
//! wander_px: 6.5`, preserving item 158's wander:pitch ratio).
//!
//! **The two traps item 186 already found, inherited rather than
//! rediscovered:** per-pixel difference is the wrong composition oracle (the
//! smallest marks legitimately read finer at 2x); and edge-skirt PIXEL counts
//! can pass their own mutation at whisper contrast (a wider ramp gets gentler
//! per pixel and can vanish under 8-bit quantization instead of reading as
//! wider). So this file measures contour separation and lane-interior tone
//! count off the DIFFERENTIAL `mark_field` oracle (item 86/89/158's own,
//! immune to the gradient/dither), and measures edge width as a mean RUN
//! LENGTH on a deliberately high-contrast Strata literal — Paperbark itself
//! is a whisper-contrast cream world, so its own tones are not trusted for a
//! feather-width claim until proven not to be the whisper-contrast trap.

use super::bands_waves::{bg_desc_for, headless_dq, render_bg_scaled};
use super::deckle_ground::{MarginStats, margin_stats};
use super::zigzag_ground::margins;
use crate::background::BgDesc;
use crate::theme::{self, Background, Weave};

fn paperbark_bg() -> Background {
    theme::PAPERBARK.background
}

/// The PRE-201 dial, reconstructed as an explicit literal (not read from any
/// live state) — the exact numbers item 158 shipped and item 186 carried
/// forward unchanged, used ONLY as the "regressed" comparison arm below.
fn pre_201_dial() -> Background {
    match paperbark_bg() {
        Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            density,
            ..
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            period_px: 94.0,
            wander_px: 13.0,
            density,
        },
        _ => unreachable!("Paperbark ships Background::Deckle"),
    }
}

/// The differential mark field at an explicit device ratio — `mark_field`
/// (item 89) hardcodes `scale=1.0` inside `render_bg`, which cannot see a real
/// Retina render at all; this is the scale-aware sibling item 201 needs.
#[allow(clippy::too_many_arguments)]
fn mark_field_scaled(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: Background,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    scale: f32,
) -> Vec<i32> {
    let inked = bg_desc_for(bg);
    let bare = BgDesc {
        density: 0.0,
        ..inked
    };
    let a = render_bg_scaled(device, queue, inked, w, h, col_left, col_w, 0.0, scale);
    let b = render_bg_scaled(device, queue, bare, w, h, col_left, col_w, 0.0, scale);
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| {
            (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>()
        })
        .collect()
}

/// Per-margin measurements at a LOGICAL `(w, h, col_left, col_w)` window,
/// rendered at device ratio `scale` (physical canvas scaled up accordingly) —
/// the matched-logical-canvas idiom item 186's own sweep uses. `min_run` is
/// the DEVICE-pixel run-length floor `mean_lane_width_px` filters transient
/// edge-ramp steps with — scale it with the device ratio so a 2x render (real
/// device pixels) is held to the same LOGICAL floor as 1x.
#[allow(clippy::too_many_arguments)]
fn margin_measurements_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: Background,
    w: u32,
    h: u32,
    col_left: u32,
    col_w: u32,
    scale: f32,
) -> Vec<(u32, MarginStats, f32)> {
    let (pw, ph) = (
        (w as f32 * scale).round() as u32,
        (h as f32 * scale).round() as u32,
    );
    let (cl, cw) = (col_left as f32 * scale, col_w as f32 * scale);
    let field = mark_field_scaled(device, queue, bg, pw, ph, cl, cw, scale);
    let min_run = (8.0 * scale).round() as u32;
    margins(pw, cl, cw)
        .iter()
        .map(|&(mx0, mx1)| {
            (
                mx1 - mx0,
                margin_stats(&field, pw, ph, mx0, mx1),
                mean_lane_width_px(&field, pw, ph, mx0, mx1, min_run),
            )
        })
        .collect()
}

/// Mean lane pitch, measured directly off the differential field as the mean
/// width of its own long flat RUNS — deliberately NOT `MarginStats::bands`,
/// which counts every quantization step during a smooth edge ramp as its own
/// "band" (a torn boundary crosses several `/6` buckets one pixel at a time)
/// and so wildly overcounts true lane crossings. A run must clear `min_run`
/// (item 158's own `lane_tones` threshold, scaled to this pitch) to count as
/// lane BODY rather than transient ramp — the same filter idea, applied to
/// counting runs instead of collecting their distinct values.
fn mean_lane_width_px(field: &[i32], w: u32, h: u32, mx0: u32, mx1: u32, min_run: u32) -> f32 {
    let mut widths: Vec<u32> = Vec::new();
    for y in (h / 8..h.saturating_sub(h / 8)).step_by((h as usize / 12).max(1)) {
        let mut last: Option<i32> = None;
        let mut run = 0u32;
        let close = |v: i32, len: u32, widths: &mut Vec<u32>| {
            let _ = v;
            if len >= min_run {
                widths.push(len);
            }
        };
        for x in mx0..mx1 {
            let q = field[(y * w + x) as usize] / 6;
            match last {
                Some(l) if l == q => run += 1,
                Some(l) => {
                    close(l, run, &mut widths);
                    run = 1;
                }
                None => run = 1,
            }
            last = Some(q);
        }
        if let Some(l) = last {
            close(l, run, &mut widths);
        }
    }
    if widths.is_empty() {
        return f32::INFINITY;
    }
    widths.iter().sum::<u32>() as f32 / widths.len() as f32
}

// ---------------------------------------------------------------------------
// 1. CONTOUR SEPARATION + LANE-INTERIOR TONE COUNT, matched 1x/2x
// ---------------------------------------------------------------------------

/// The logical window every headline measurement below shares: generous
/// enough that BOTH margins clear several lane pitches even at the finer,
/// restored density.
const W: u32 = 1400;
const H: u32 = 900;
const COL_LEFT: u32 = 350;
const COL_W: u32 = 500;

/// Item 158's own authored pitch, restored (`PAPERBARK.background.period_px()`
/// mirrors this — asserted equal below so the two cannot silently drift).
const RESTORED_PERIOD_PX: f32 = 47.0;

#[test]
fn paperbark_contour_separation_matches_the_restored_density_at_1x_and_2x() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping paperbark_contour_separation_matches_the_restored_density: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let bg = paperbark_bg();
    assert_eq!(
        bg.period_px(),
        RESTORED_PERIOD_PX,
        "PAPERBARK's authored period_px drifted from what this law was calibrated against"
    );

    let one_x = margin_measurements_at(&device, &queue, bg, W, H, COL_LEFT, COL_W, 1.0);
    let two_x = margin_measurements_at(&device, &queue, bg, W, H, COL_LEFT, COL_W, 2.0);

    // A margin at least two lane pitches wide is required to grade lane-tone
    // layering (item 158's own "two lanes' worth" rule); this window's
    // margins (350px, ~550px at COL_W=500 W=1400) both clear that at 47px.
    let mut worst_sep_1x = 0.0f32;
    let mut worst_sep_2x_logical = 0.0f32;
    for ((_, s1, sep_1x), (_, s2, sep_2x)) in one_x.iter().zip(two_x.iter()) {
        assert!(
            s1.lane_tones >= 2 && s2.lane_tones >= 2,
            "a margin at least two lane pitches wide must show at least two DISTINCT lane-\
             interior tones at BOTH densities (1x {} tones, 2x {} tones) — item 158's collapse \
             law's own lesson: a boundary alone (spread/bands) is not layered paper",
            s1.lane_tones,
            s2.lane_tones,
        );
        let (sep_1x, sep_2x_logical) = (*sep_1x, sep_2x / 2.0);
        // CONTOUR SEPARATION, restored: the 1x margin's own pitch must sit
        // close to the authored period_px — the exact regression's numeric
        // signature (pre-201 this would measure ~94, twice the target).
        assert!(
            (sep_1x - RESTORED_PERIOD_PX).abs() < RESTORED_PERIOD_PX * 0.35,
            "1x contour separation measures {sep_1x:.1} device px against the restored \
             period_px {RESTORED_PERIOD_PX:.1} — Paperbark's Retina regression's exact \
             numeric signature is this measurement drifting back toward double"
        );
        // MATCHED-LOGICAL COMPOSITION: the 2x margin's separation, expressed
        // back in logical units, must agree with the 1x measurement — item
        // 186's rule, re-proven specifically for Paperbark's own contour
        // spacing rather than assumed from the general roster sweep.
        assert!(
            (sep_1x - sep_2x_logical).abs() < RESTORED_PERIOD_PX * 0.35,
            "contour separation is not matched between 1x ({sep_1x:.1} device px) and the \
             scale-normalized 2x measurement ({sep_2x_logical:.1} logical px) — Paperbark's \
             lane pitch is display-density-dependent again"
        );
        worst_sep_1x = worst_sep_1x.max((sep_1x - RESTORED_PERIOD_PX).abs());
        worst_sep_2x_logical = worst_sep_2x_logical.max((sep_1x - sep_2x_logical).abs());
    }
    eprintln!(
        "item-201 contour separation: worst |1x - period_px| {worst_sep_1x:.2}px, worst \
         |1x - 2x_logical| {worst_sep_2x_logical:.2}px (period_px {RESTORED_PERIOD_PX})"
    );
}

// ---------------------------------------------------------------------------
// 2. EDGE RUN LENGTH — the deckle boundary is COMPOSITION, so it must widen
//    with the device ratio (the OPPOSITE of a sampling feather), measured on
//    a high-contrast literal per the whisper-contrast trap in this file's doc.
// ---------------------------------------------------------------------------

/// The Strata weave at Paperbark's restored dials, but with tones pulled far
/// apart — the item-186 "high-contrast literal" idiom
/// (`ground_space::finds_high_contrast`), required because Paperbark's
/// own whisper tones are exactly the contrast regime where a run-length
/// measurement can be fooled by 8-bit quantization.
/// Deliberately NOT Paperbark's own (tight, 47px) pitch: the deckle edge is a
/// fixed FRACTION of the lane (`DECKLE_EDGE_LO..HI`, ~0.06 of it), so at 47px
/// its 1x transition zone is only ~2.8 device px wide — close enough to the
/// sampling grid's own resolution that discretization noise, not the ramp's
/// real width, dominates `mean_edge_ramp_px`'s measurement (the exact
/// under-sampling failure mode, distinct from but as real as item 186's
/// documented whisper-CONTRAST trap). `DECKLE_MAX_PERIOD_PX` gives the ramp
/// room (~9.6px at 1x) to measure cleanly; this checks the FAMILY property
/// (the edge is composition, not Paperbark's own specific dial).
fn deckle_high_contrast() -> Background {
    Background::Deckle {
        ground: theme::Srgb::rgb(0x10, 0x10, 0x12),
        layer: theme::Srgb::rgb(0xF4, 0xF2, 0xEC),
        deckle: theme::Srgb::rgb(0x88, 0x86, 0x82),
        weave: Weave::Strata,
        period_px: theme::DECKLE_MAX_PERIOD_PX,
        wander_px: theme::DECKLE_MAX_PERIOD_PX * 0.1383,
        density: 1.0,
    }
}

/// The mean width of a TRANSITION run in the DIFFERENTIAL field — deliberately
/// not `mean_edge_ramp_px` (item 186's own idiom, built for Finds' hard SDF
/// edge) applied to raw COMPOSITED pixels: a probe row showed the ordered
/// Bayer dither, identical noise magnitude at both densities, fragments a
/// WIDE proportional ramp (Deckle's, unlike Finds' ~1.5px-total feather) into
/// several short sub-runs whenever a single device-pixel step's true color
/// change is small enough for dither to occasionally round two neighbours to
/// the same 8-bit value mid-ramp — worse at 2x, where each step covers LESS
/// of the transition. The differential field cancels dither exactly (item
/// 89/158's own oracle: identical in the inked and bare passes, so it
/// subtracts out), so run lengths measured on it are not subject to that
/// fragmentation. `min_step` filters residual dither/quantization noise in
/// otherwise-flat lane interiors, scaled with the device ratio exactly as
/// `margin_measurements_at`'s `min_run` is.
fn mean_transition_run_px(field: &[i32], w: u32, h: u32, mx0: u32, mx1: u32, min_step: i32) -> f32 {
    let mut total = 0u64;
    let mut runs = 0u64;
    for y in (h / 8..h.saturating_sub(h / 8)).step_by((h as usize / 12).max(1)) {
        let mut run = 0u64;
        for x in mx0..mx1.saturating_sub(1) {
            let d = (field[(y * w + x) as usize] - field[(y * w + x + 1) as usize]).abs();
            if d >= min_step {
                run += 1;
            } else if run > 0 {
                total += run;
                runs += 1;
                run = 0;
            }
        }
        if run > 0 {
            total += run;
            runs += 1;
        }
    }
    if runs == 0 {
        return 0.0;
    }
    total as f32 / runs as f32
}

#[test]
fn paperbark_edge_run_length_widens_with_device_ratio_because_it_is_composition() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping paperbark_edge_run_length_widens_with_device_ratio: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = deckle_high_contrast();
    let one_x = mark_field_scaled(
        &device,
        &queue,
        bg,
        W,
        H,
        COL_LEFT as f32,
        COL_W as f32,
        1.0,
    );
    let two_x = mark_field_scaled(
        &device,
        &queue,
        bg,
        W * 2,
        H * 2,
        (COL_LEFT * 2) as f32,
        (COL_W * 2) as f32,
        2.0,
    );
    let r1 = mean_transition_run_px(&one_x, W, H, 0, COL_LEFT, 2);
    let r2 = mean_transition_run_px(&two_x, W * 2, H * 2, 0, COL_LEFT * 2, 4);
    let ratio = r2 / r1.max(f32::EPSILON);
    // The OPPOSITE bound from Finds' crisp-edge law: Deckle's torn boundary
    // is a FRACTION of the lane (composition), not a fixed device skirt, so
    // it must widen roughly 2x in device px at 2x — a feather wrongly held
    // physical here would hold this ratio near 1.0 instead.
    assert!(
        (1.5..=2.6).contains(&ratio),
        "Deckle's edge run length measures {r1:.2} device px at 1x and {r2:.2} at 2x — a \
         ratio of {ratio:.2}, where a COMPOSITION edge (a fraction of the lane) widens ~2x \
         and a wrongly-physical feather would hold ~1.0. `DECKLE_EDGE_LO/HI` must stay a \
         lane fraction, never a device-pixel skirt."
    );
    eprintln!("item-201 edge run length: {r1:.2}px @1x, {r2:.2}px @2x (ratio {ratio:.2})");
}

// ---------------------------------------------------------------------------
// 3. PAGE-WIDTH STABILITY — the restored density holds across margin widths,
//    at both device ratios, not just one hand-picked geometry.
// ---------------------------------------------------------------------------

#[test]
fn paperbark_lane_density_is_stable_across_page_width_at_1x_and_2x() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping paperbark_lane_density_is_stable_across_page_width: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = paperbark_bg();
    let mut graded = 0usize;
    // Margin widths spanning "barely two lanes" to "generously wide", at a
    // fixed canvas — the axis item 89/158's own lesson says to sweep, since a
    // hand-picked width can hide a defect the geometry sweep exposes.
    for col_w in [1000u32, 800, 600, 400, 250] {
        let col_left = (2000 - col_w) / 2;
        let one_x = margin_measurements_at(&device, &queue, bg, 2000, 900, col_left, col_w, 1.0);
        let two_x = margin_measurements_at(&device, &queue, bg, 2000, 900, col_left, col_w, 2.0);
        for ((w1, _, sep_1x), (_, _, sep_2x)) in one_x.iter().zip(two_x.iter()) {
            if (*w1 as f32) < RESTORED_PERIOD_PX * 2.0 {
                continue; // too narrow to grade layering, exactly item 158's own carve-out
            }
            let (sep_1x, sep_2x_logical) = (*sep_1x, sep_2x / 2.0);
            assert!(
                (sep_1x - RESTORED_PERIOD_PX).abs() < RESTORED_PERIOD_PX * 0.4,
                "col_w {col_w}: 1x separation {sep_1x:.1} strayed from the restored pitch \
                 {RESTORED_PERIOD_PX}"
            );
            assert!(
                (sep_1x - sep_2x_logical).abs() < RESTORED_PERIOD_PX * 0.4,
                "col_w {col_w}: separation is not matched between 1x ({sep_1x:.1}) and \
                 scale-normalized 2x ({sep_2x_logical:.1})"
            );
            graded += 1;
        }
    }
    assert!(
        graded >= 6,
        "the page-width sweep must actually grade margins (graded {graded})"
    );
    eprintln!("item-201 page-width stability: {graded} margins graded across 5 widths");
}

// ---------------------------------------------------------------------------
// 4. MUTATION WITNESS — the pre-201 dial, still reachable as an explicit
//    literal, must fail the headline separation law by roughly double.
// ---------------------------------------------------------------------------

/// Not a `#[test]` — a callable proof the pre-201 dial (still constructible
/// as an explicit literal above) measures roughly DOUBLE the restored
/// separation, i.e. the exact regression. Exercised by
/// `pre_201_dial_would_fail_the_separation_law`, and by hand against a real
/// reversion of `PAPERBARK`'s literal (see this item's report for the actual
/// panic text that produced).
#[test]
fn pre_201_dial_would_fail_the_separation_law() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping pre_201_dial_would_fail_the_separation_law: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let bg = pre_201_dial();
    // A wider margin than the headline law's (800px vs 350/550px): at the
    // pre-201 pitch (94px) the headline geometry holds under 4 complete
    // lanes, so its mean-run measurement is dominated by the two partial
    // lanes at the margin's own edges. 800px holds ~8.5, averaging that noise
    // out enough to state a reliable number.
    let one_x = margin_measurements_at(&device, &queue, bg, 2400, 900, 800, 800, 1.0);
    let (_, _, sep_1x) = &one_x[0];
    let sep_1x = *sep_1x;
    assert!(
        sep_1x > RESTORED_PERIOD_PX * 1.5,
        "the pre-201 dial (period_px 94.0) must measure roughly DOUBLE the restored \
         separation — it measured {sep_1x:.1} against a >{:.1} floor, which would mean the \
         reverted dial no longer reproduces the regression this item fixes",
        RESTORED_PERIOD_PX * 1.5,
    );
    eprintln!(
        "item-201 mutation witness: pre-201 dial (94.0/13.0) measures {sep_1x:.1} device px \
         separation — {:.2}x the restored {RESTORED_PERIOD_PX}",
        sep_1x / RESTORED_PERIOD_PX
    );
}

// ---------------------------------------------------------------------------
// 5. SCOPE — Galah's Fibres profile and the Deckle geometry the Paperbark
//    retune was forbidden to touch stayed exactly as authored.
//
//    `density` is deliberately NOT asserted here. It is a loudness dial with
//    its own owner — a band law plus a roster-wide "the dial does material
//    work" sweep in `deckle_ground.rs`, and a byte-exact snapshot of
//    this whole ground in `loudness_map.rs` that names the world when
//    it moves. Pinning the value here as well made this file a SECOND owner of
//    someone else's constant: a later, deliberate retune of the dial (0.10 ->
//    0.12) failed this test, whose subject is the fibre GEOMETRY rather than
//    the loudness, and the failure said nothing true about the retune. The
//    fields below are the ones a Paperbark change could plausibly have
//    disturbed, and they are still pinned exactly.
// ---------------------------------------------------------------------------

#[test]
fn galahs_fibres_geometry_is_untouched_by_this_item() {
    match theme::GALAH.background {
        Background::Deckle {
            weave,
            period_px,
            wander_px,
            ..
        } => {
            assert_eq!(weave, Weave::Fibres);
            assert_eq!(period_px, 64.0);
            assert_eq!(wander_px, 8.0);
        }
        _ => panic!("Galah must still ship Background::Deckle"),
    }
}

// ---------------------------------------------------------------------------
// 6. BEFORE/AFTER GALLERY SHEET
// ---------------------------------------------------------------------------

fn save(pixels: &[[u8; 4]], w: u32, h: u32, path: &str) {
    let mut img = image::RgbaImage::new(w, h);
    for (i, px) in pixels.iter().enumerate() {
        img.put_pixel((i as u32) % w, (i as u32) / w, image::Rgba(*px));
    }
    img.save(path).unwrap();
    eprintln!("wrote {path}");
}

#[test]
fn paperbark_retina_before_after_sheet() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping paperbark_retina_before_after_sheet: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    std::fs::create_dir_all("gallery/item-201-paperbark-retina").ok();
    let (w, h, cl, cw) = (1600u32, 1000u32, 400u32, 600u32);

    // BEFORE: the pre-201 dial, rendered at the real device ratio a Retina
    // user actually sees (matched-logical 2x) — reproduces the reported
    // "unclear" ground exactly, since the dial (not the mechanism) is at
    // fault.
    let before = render_bg_scaled(
        &device,
        &queue,
        bg_desc_for(pre_201_dial()),
        w * 2,
        h * 2,
        (cl * 2) as f32,
        (cw * 2) as f32,
        0.0,
        2.0,
    );
    // AFTER: the shipped, restored dial, same matched-logical 2x geometry.
    let after = render_bg_scaled(
        &device,
        &queue,
        bg_desc_for(paperbark_bg()),
        w * 2,
        h * 2,
        (cl * 2) as f32,
        (cw * 2) as f32,
        0.0,
        2.0,
    );
    // NEIGHBOURHOOD: Galah's Fibres at the same geometry, for the vision
    // smoke's distinctness question.
    let galah = render_bg_scaled(
        &device,
        &queue,
        bg_desc_for(theme::GALAH.background),
        w * 2,
        h * 2,
        (cl * 2) as f32,
        (cw * 2) as f32,
        0.0,
        2.0,
    );

    save(
        &before,
        w * 2,
        h * 2,
        "gallery/item-201-paperbark-retina/before-2x-unclear.png",
    );
    save(
        &after,
        w * 2,
        h * 2,
        "gallery/item-201-paperbark-retina/after-2x-restored.png",
    );
    save(
        &galah,
        w * 2,
        h * 2,
        "gallery/item-201-paperbark-retina/galah-2x-neighbourhood.png",
    );
}
