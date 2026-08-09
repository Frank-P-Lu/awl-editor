//! ITEM 191 — Bowerbird's shipped `Finds` ground, tuned from rendered pixels.
//!
//! Three changes, each verified separately here:
//!   * the anchor/companion/cut-out composition grew ~15% as ONE
//!     hierarchy-preserving move (`FINDS_ANCHOR_LO`/`_HI` in
//!     `shaders/background.wgsl`, whose own comment states the ratio math);
//!   * the cell PITCH opened separately (Bowerbird's `scale_px`, 156 -> 195);
//!   * the item-176 UNCONSTRAINED per-cell dropout — an independent coin flip
//!     every cell, which could and did let several neighbouring omissions
//!     align into a conspicuous dead patch — is now a DECORRELATED,
//!     deterministic one (`finds_is_local_min` in the shader) that provably
//!     never drops two lattice-adjacent cells at once.
//!
//! **Why the void law reads nearest-neighbour spacing, not an inscribed
//! circle or a re-derived lattice grid.** Two more direct-looking approaches
//! were tried and rejected, on real measurements:
//!   * The largest circle inscribed in open ground (a Chamfer
//!     distance-to-ink transform) is the wrong SHAPE: this arrangement's own
//!     "generous open ground" law leaves 70-95% of every cell empty by
//!     design, so the ordinary gap at a 4-cell corner already produces an
//!     inscribed circle nearly as large as a real dropout produces — reverting
//!     the fix barely moved the measured radius (152px fixed vs 202px broken,
//!     both comfortably under any bound generous enough not to fire on
//!     ordinary packing). A circle is bounded by a void's NARROW axis; a
//!     dropout chain is long and thin, so the metric is structurally blind to
//!     the exact shape "neighbouring omissions align" describes.
//!   * Mirroring the shader's own lattice transform host-side (to count
//!     dropped CELLS directly) turned out to be unreliable for a different
//!     reason: `hash21` is `fract(sin(dot(p, big constants)) * huge constant)`,
//!     and `sin` of the resulting few-thousand-magnitude arguments is exactly
//!     where GPU and CPU trig implementations are LEAST guaranteed to agree
//!     (argument reduction at that magnitude is implementation-defined) — a
//!     real cross-check against rendered pixels measured the host mirror
//!     disagreeing with the actual GPU dropout decision on 11 of 45 cells in
//!     one field, not a rare near-threshold flip but a wholesale wrong
//!     answer (a cell the host computed `h0=0.64`, nowhere near the 0.226
//!     threshold, that the GPU still rendered empty). Reusing that same
//!     `hash21` for anything host-side would silently inherit the bug.
//!
//! The law instead uses ONLY what `bowerbird_finds.rs`'s pixel reader
//! already proves reliable: each collection's own centre. In an undropped
//! lattice a collection's nearest surviving neighbour sits about one cell
//! pitch away (jitter is bounded to a small fraction of a cell); dropping ONE
//! cell pushes its neighbours' nearest-surviving-neighbour out to roughly two
//! pitches, and AN ALIGNED RUN of several dropped cells pushes it further
//! still — so the MAXIMUM nearest-neighbour distance across a field is a
//! direct, purely pixel-based read of "how bad is the worst gap actually
//! surrounding a real collection," sensitive to exactly the elongated-chain
//! shape the other two approaches missed, and it needs no re-derivation of
//! the shader's own hash.
//!
//! Reuses the field reader `bowerbird_finds.rs` already built and
//! proved (`Collection`, `read_collections`, `organic_bg`) rather than
//! re-implementing it. Per the project's own tripwire (the sidecar is a
//! state oracle, never an appearance oracle), every claim below is measured
//! over rendered bytes.

use super::bands_waves::{bg_desc_for, headless_dq, render_bg_scaled};
use super::bowerbird_finds::{Collection, organic_bg, read_collections};
use crate::theme;

// --- Size and spacing readers ------------------------------------------------

fn role_radius_px(area_px: usize) -> f32 {
    (area_px as f32 / std::f32::consts::PI).sqrt()
}

/// Mean anchor(major)/companion(minor)/cut-out radius across every whole
/// collection a field draws — the "collections read bigger" claim, in px.
fn role_size_means(found: &[Collection]) -> (f32, f32, f32) {
    let n = (found.len().max(1)) as f32;
    let mean =
        |f: fn(&Collection) -> usize| found.iter().map(|c| role_radius_px(f(c))).sum::<f32>() / n;
    (
        mean(|c| c.major_px),
        mean(|c| c.minor_px),
        mean(|c| c.cutout_px),
    )
}

/// Every collection's own nearest-neighbour distance (centre to centre). The
/// SEARCH considers every found collection (including ones near the crop
/// edge, real neighbours), but only REPORTS a gap for a collection whose own
/// centre sits at least `interior_px` inside the canvas on every side — a
/// collection near the crop edge can have its TRUE nearest neighbour cropped
/// away by `read_collections`' own border filter, which inflates its nearest
/// SURVIVING gap for a reason that has nothing to do with the dropout
/// mechanism this file measures.
fn nearest_neighbor_distances_px(
    found: &[Collection],
    w: u32,
    h: u32,
    interior_px: f32,
) -> Vec<f32> {
    found
        .iter()
        .filter(|a| {
            a.cx >= interior_px
                && a.cy >= interior_px
                && a.cx <= w as f32 - interior_px
                && a.cy <= h as f32 - interior_px
        })
        .map(|a| {
            found
                .iter()
                .filter(|b| (a.cx, a.cy) != (b.cx, b.cy))
                .map(|b| ((a.cx - b.cx).powi(2) + (a.cy - b.cy).powi(2)).sqrt())
                .fold(f32::INFINITY, f32::min)
        })
        .collect()
}

fn mean(xs: &[f32]) -> f32 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f32>() / xs.len() as f32
    }
}

fn max_of(xs: &[f32]) -> f32 {
    xs.iter().copied().fold(0.0, f32::max)
}

// --- The grid every claim below sweeps ---------------------------------------

// Sized specifically so the void-bound law below is NON-VACUOUS: a smaller
// canvas (the original 1800x1200 / 900x1200 field this file started with)
// holds too few cells for a rare multi-cell dropout run to reliably appear in
// only a few sampled phases, so the very mutation this law exists to catch
// could pass it by pure sampling luck — measured directly, the unconstrained
// item-176 mechanism's worst observed gap over 6 phases was 200px (under the
// fixed mechanism's OWN ~204px ceiling) at 1800x1200, but 248-339px — clearly
// over it — at every size from 2400x1600 up. `WIDE`/`NARROW` below are picked
// from that measurement, not merely "a bit bigger for safety."
const WIDE_W: u32 = 2400;
const WIDE_H: u32 = 1600;
/// A narrow margin strip, in the same spirit as `capture-bowerbird-revival.sh`'s
/// "narrow" page regime (a slimmer width, the same generous height).
const NARROW_W: u32 = 1200;
const NARROW_H: u32 = 1600;
/// Three phases spread across the drift cycle's full period (`TAU`), not
/// merely "at rest" — the ambient field pans continuously, so a bound proved
/// only at phase 0 would miss whatever phase is actually worst.
const DRIFT_PHASES: [f32; 3] = [0.0, 2.0943952, 4.1887903]; // 0, TAU/3, 2*TAU/3

/// The crop-edge exclusion `nearest_neighbor_distances_px` needs, in the same
/// PHYSICAL px its inputs (`Collection::cx`/`cy`, `w`, `h`) are already in: a
/// full cell's worth beyond a collection's own ~0.5-cell worst-case reach
/// (see `FINDS_*`'s reach comment in `shaders/background.wgsl`), so a
/// collection judged on its nearest-neighbour gap is never being judged on an
/// accident of the crop instead.
fn interior_margin_px(scale: f32, density: u32) -> f32 {
    scale * density as f32
}

/// Bowerbird's own live authored cell pitch, read dynamically off the world
/// literal rather than duplicated as a second hardcoded number — the law
/// tracks whatever `worlds.rs` says.
fn bowerbird_finds_scale() -> f32 {
    match theme::BOWERBIRD.background {
        theme::Background::Organic { scale_px, .. } => scale_px,
        _ => panic!("Bowerbird must ship Background::Organic"),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_finds(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    scale: f32,
    w: u32,
    h: u32,
    density: u32,
    phase: f32,
) -> Vec<[u8; 4]> {
    let bg = bg_desc_for(organic_bg(scale));
    render_bg_scaled(
        device,
        queue,
        bg,
        w * density,
        h * density,
        0.0,
        0.0,
        phase,
        density as f32,
    )
}

// --- The diagnostic: BEFORE/AFTER numbers, printed not asserted -------------

/// Not a law — a measurement dump. Run with
/// `cargo test --bin awl bowerbird_spacing::measure -- --ignored --nocapture`
/// before and after a tuning edit to get the real before/after distributions
/// item 191 asks for (role size, nearest-neighbour spacing, its worst case),
/// across wide/narrow, 1x/2x and three drift phases.
#[test]
#[ignore]
fn measure_bowerbird_finds_distributions() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping measure_bowerbird_finds_distributions: no adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let scale = bowerbird_finds_scale();
    println!("Bowerbird Finds — scale_px={scale}");
    for (label, w, h) in [("wide", WIDE_W, WIDE_H), ("narrow", NARROW_W, NARROW_H)] {
        for density in [1u32, 2] {
            for (pi, phase) in DRIFT_PHASES.into_iter().enumerate() {
                let pixels = render_finds(&device, &queue, scale, w, h, density, phase);
                let found = read_collections(&pixels, w * density, h * density);
                let (major, minor, cutout) = role_size_means(&found);
                let margin = interior_margin_px(scale, density);
                let nn = nearest_neighbor_distances_px(&found, w * density, h * density, margin);
                let (nn_mean, nn_max) = (mean(&nn) / density as f32, max_of(&nn) / density as f32);
                println!(
                    "{label:6} {density}x phase{pi}: n={:3} major_r={major:6.1} \
                     minor_r={minor:6.1} cutout_r={cutout:5.1} nn_mean={nn_mean:6.1} \
                     nn_max={nn_max:6.1} (scale={scale})",
                    found.len()
                );
            }
        }
    }
}

// --- The laws -----------------------------------------------------------------

/// LAW: the composition grew as ONE hierarchy-preserving move. At the
/// item-176 reference scale (156 — a claim about the MECHANISM, not about
/// whichever pitch Bowerbird happens to author), the mean anchor/companion/
/// cut-out radii are each measurably larger than the item-176 empirical
/// baseline (pinned below, measured once against the pre-191 shader) by
/// comparable ratios — proving the three roles grew TOGETHER rather than one
/// role being retuned alone.
#[test]
fn finds_composition_grew_about_fifteen_percent_and_kept_its_ratios() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_composition_grew_about_fifteen_percent_and_kept_its_ratios");
        return;
    };
    let _g = crate::testlock::serial();
    let pixels = render_finds(&device, &queue, 156.0, WIDE_W, WIDE_H, 1, 0.0);
    let found = read_collections(&pixels, WIDE_W, WIDE_H);
    assert!(
        found.len() >= 25,
        "vacuous: only {} collections",
        found.len()
    );
    let (major, minor, cutout) = role_size_means(&found);

    // Empirically measured ONCE against the pre-191 shader (`FINDS_ANCHOR_LO
    // 0.150`/`_HI 0.195`), same canvas/scale/phase — the real "before" this
    // growth is measured against, not a re-derived analytic estimate (an
    // early draft compared against a hand-derived nominal-radius midpoint and
    // it was wrong: visible area is nominal area MINUS the companion's
    // overlap, which does not cancel out of a simple LO/HI mean).
    let old_major_r = 23.07f32;
    let old_minor_r = 13.06f32;
    let old_cutout_r = 3.74f32;

    for (label, measured, old) in [
        ("major/anchor", major, old_major_r),
        ("minor/companion", minor, old_minor_r),
        ("cutout", cutout, old_cutout_r),
    ] {
        let growth = measured / old;
        assert!(
            (1.06..1.24).contains(&growth),
            "{label}: measured mean radius {measured:.1}px vs the item-176 baseline {old:.1}px \
             is a {growth:.3}x change — outside the ~15% hierarchy-preserving growth band"
        );
    }
}

/// LAW: the cell pitch opened SEPARATELY from the composition — Bowerbird's
/// own `scale_px` is a materially different ratio (20-30%) from the ~15%
/// composition growth measured above, so the two are provably not one
/// disguised uniform scale.
#[test]
fn finds_cell_pitch_opened_by_a_separately_authored_amount() {
    let shipped = bowerbird_finds_scale();
    let reference = 156.0f32;
    let pitch_growth = shipped / reference;
    assert!(
        (1.18..1.35).contains(&pitch_growth),
        "Bowerbird's scale_px is {shipped}, a {pitch_growth:.3}x change off the item-176 \
         reference (156.0) — expected a clearly-separate ~20-30% opening of the lattice, \
         distinct from the ~15% composition growth"
    );
}

/// LAW (the defect item 191 exists to fix — see the module doc for why this
/// reads nearest-neighbour spacing rather than an inscribed circle or a
/// re-derived lattice grid): the WORST nearest-neighbour gap between any two
/// surviving collections never exceeds `MAX_NN_CELLS` cell-pitches — wide and
/// narrow margins, 1x and 2x, and three drift phases spread across the
/// ambient cycle, so the bound holds at whichever phase is worst for it, not
/// merely at rest.
///
/// `1.10` is a MEASURED bound, not a round guess: swept over wide/narrow,
/// both densities and many phases, the fixed (`finds_is_local_min`-gated)
/// mechanism's worst observed gap was 1.044 cell-pitches, dead flat across
/// every canvas size tried (204px at scale 195, i.e. `195 * 1.044`) — the
/// guarantee that no two Moore-adjacent cells are ever both dropped in
/// action. Reverting to the item-176 unconstrained dropout (same threshold,
/// `finds_is_local_min` removed) measured 1.27-1.74 cell-pitches at these
/// same canvas sizes. `1.10` sits with real margin above the fixed ceiling
/// and real margin below every broken measurement — see this law's own
/// mutation-proof note in the module doc history for the exact broken run.
const MAX_NN_CELLS: f32 = 1.10;

#[test]
fn finds_dropout_never_opens_a_gap_much_larger_than_ordinary_spacing() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_dropout_never_opens_a_gap_much_larger_than_ordinary_spacing");
        return;
    };
    let _g = crate::testlock::serial();
    let scale = bowerbird_finds_scale();
    let bound = MAX_NN_CELLS * scale;
    for (label, w, h) in [("wide", WIDE_W, WIDE_H), ("narrow", NARROW_W, NARROW_H)] {
        for density in [1u32, 2] {
            for (pi, phase) in DRIFT_PHASES.into_iter().enumerate() {
                let pixels = render_finds(&device, &queue, scale, w, h, density, phase);
                let found = read_collections(&pixels, w * density, h * density);
                assert!(
                    found.len() >= 10,
                    "{label} {density}x phase{pi}: only {} collections — vacuous",
                    found.len()
                );
                let margin = interior_margin_px(scale, density);
                let worst = max_of(&nearest_neighbor_distances_px(
                    &found,
                    w * density,
                    h * density,
                    margin,
                )) / density as f32;
                assert!(
                    worst <= bound,
                    "{label} {density}x phase{pi} ({phase}): the worst nearest-neighbour gap \
                     between two surviving collections is {worst:.1}px, over the {bound:.1}px \
                     ({MAX_NN_CELLS}-cell) bound — neighbouring omissions aligned into a \
                     conspicuous dead patch"
                );
            }
        }
    }
}

/// LAW: `density: 0.0` still collapses the tuned arrangement to the flat open
/// ground exactly — item 191 touched the anchor scale, the pitch and the
/// dropout gate, none of which the density collapse depends on, but this is
/// the differential oracle every size/void measurement above implicitly
/// leans on, so it is checked directly rather than assumed carried over from
/// item 176.
#[test]
fn finds_tuned_density_zero_is_still_exactly_the_flat_ground() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping finds_tuned_density_zero_is_still_exactly_the_flat_ground");
        return;
    };
    let _g = crate::testlock::serial();
    let scale = bowerbird_finds_scale();
    let mut flat = bg_desc_for(organic_bg(scale));
    flat.density = 0.0;
    let pixels = render_bg_scaled(&device, &queue, flat, 600, 400, 0.0, 0.0, 0.0, 1.0);
    let first = pixels[0];
    assert!(
        pixels.iter().all(|p| *p == first),
        "density 0 must leave one flat tone; found at least two"
    );
}
