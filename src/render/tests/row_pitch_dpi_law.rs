//! THE ROW PITCH SURVIVES A DISPLAY-SCALE CHANGE OVER AN ALREADY-SHAPED DOCUMENT.
//!
//! A window dragged from a 1x panel to a 2x one — or a system scale change under
//! the running app — arrives as one winit `ScaleFactorChanged`, and the app answers
//! it with exactly two calls: `TextPipeline::set_dpi` and then a view push. That is
//! the ONLY door that changes the scale of a pipeline already holding a shaped
//! document; every capture door builds a fresh pipeline and sets its DPI before any
//! text, so no capture can see this axis and none of them is affected.
//!
//! **WHAT A RELAYOUT DOES NOT REPAIR.** `set_dpi` re-sets the buffer's metrics, which
//! re-lays every line — but a line's own attrs may carry an ABSOLUTE per-span
//! line-height, and an absolute value survives the relayout and then wins over the
//! buffer's new metrics. Two constructs carry one: a heading's scaled span metrics,
//! and every WYSIWYG conceal, whose zero-width override deliberately pairs its
//! near-zero font size with the line's own line-height. Only the heading was
//! restyled on a scale change, so a heading-less markdown document with any
//! concealed markup kept its row pitch at the value it shaped at — measured at
//! 32.0 for dpi 1.5, 2 AND 3 alike, while a plain line in the SAME document
//! correctly reported 48, 64 and 96.
//!
//! **WHY IT READS AS THE PRODUCT SHRINKING.** `caret_band_scale` is
//! `row_height / metrics.line_height`, so a stale row height over live metrics is a
//! ratio that FALLS as density rises: the inline-code pill's band measured 28px at
//! every tier, where the correct band is 28 x the display factor. The pill's
//! x-extents tracked the new metrics the whole time, so the defect presents as a
//! band that shrinks relative to its own text — the wrong sign, which is how the
//! hazard announces itself.
//!
//! **THE SHAPE OF EACH CLAIM.** Every claim drives the live event's own call pair
//! against an already-shaped document and compares it to a FRESH pipeline at the
//! same tier — a differential oracle, so no claim is satisfiable by a degenerate
//! value. Each also pins the quantity absolutely (`LINE_HEIGHT * dpi`, the tier-1
//! band divided by the display factor) so a repair that makes both orders equally
//! wrong still fails. The tier list carries a FRACTIONAL member because a repair
//! keyed to an integer factor passes at 2 and 3 and fails at 1.5.

use super::super::*;
use super::{headless_dqp, view};

/// The tiers every claim is graded at. 1.5 is a real macOS scale and is here
/// deliberately.
const TIERS: [f32; 4] = [1.0, 1.5, 2.0, 3.0];

/// The LOGICAL window every tier is held at, so a comparison across tiers compares
/// the same window rather than two different ones.
const LOGICAL_W: f32 = 1200.0;
const LOGICAL_H: f32 = 800.0;

/// A heading-LESS markdown document carrying concealed markup: the case only the
/// conceal override pins, and the one the heading gate never covered. Short lines,
/// so neither tier's wrap width can reflow them.
const CONCEALED_MD: &str = "`code` at line start\nplain second line\n";

/// The same document with a heading, so the sweep covers the construct the old gate
/// DID answer alongside the one it did not.
const HEADING_MD: &str = "# Heading one\n\n`code` and prose\n";

/// Plain prose: no absolute per-span metrics anywhere, the arm that was already
/// correct and must stay so.
const PLAIN: &str = "hello world\nsecond line\n";

fn md_view(text: &str, cursor_line: usize) -> ViewState {
    let mut v = view(text, cursor_line, 0);
    v.is_markdown = true;
    v
}

/// A FRESH pipeline at `dpi`, DPI set before any document — the capture path's and
/// app startup's own order, and the reference every claim measures against.
fn fresh_at(dpi: f32) -> Option<(wgpu::Device, wgpu::Queue, TextPipeline)> {
    let (device, queue, mut p) = headless_dqp(LOGICAL_W * dpi, LOGICAL_H * dpi)?;
    p.set_dpi(dpi);
    p.set_size(LOGICAL_W * dpi, LOGICAL_H * dpi);
    Some((device, queue, p))
}

fn present(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue, dpi: f32) {
    p.prepare(
        device,
        queue,
        (LOGICAL_W * dpi) as u32,
        (LOGICAL_H * dpi) as u32,
    )
    .expect("a headless frame prepares");
    p.atlas.trim();
}

fn line_pitches(p: &TextPipeline, text: &str) -> Vec<f32> {
    (0..text.lines().count())
        .map(|li| p.visual_rows(li)[0].line_height)
        .collect()
}

/// The row pitch of every logical line of `text`, from a pipeline shaped at dpi 1
/// and THEN moved to `dpi` — the live `ScaleFactorChanged` sequence (`set_dpi`, the
/// surface resize, one view push) over an already-shaped document.
fn pitches_after_scale_change(text: &str, cursor_line: usize, dpi: f32) -> Option<Vec<f32>> {
    let (device, queue, mut p) = headless_dqp(LOGICAL_W, LOGICAL_H)?;
    p.set_size(LOGICAL_W, LOGICAL_H);
    let v = md_view(text, cursor_line);
    p.set_view(&v);
    present(&mut p, &device, &queue, 1.0);
    // ENROLMENT: the document really was shaped at the OTHER scale first. Without
    // this the "already-shaped" premise of the whole file is unproven.
    let before = p.visual_rows(1)[0].line_height;
    assert!(
        (before - LINE_HEIGHT).abs() < 1e-3,
        "enrolment: line 1 must be shaped at LINE_HEIGHT before the move, got {before}"
    );
    p.set_dpi(dpi);
    p.set_size(LOGICAL_W * dpi, LOGICAL_H * dpi);
    p.set_view(&v);
    present(&mut p, &device, &queue, dpi);
    Some(line_pitches(&p, text))
}

/// The same pitches from a pipeline born at `dpi`.
fn pitches_fresh(text: &str, cursor_line: usize, dpi: f32) -> Option<Vec<f32>> {
    let (device, queue, mut p) = fresh_at(dpi)?;
    let v = md_view(text, cursor_line);
    p.set_view(&v);
    present(&mut p, &device, &queue, dpi);
    Some(line_pitches(&p, text))
}

/// CLAIM 1 — A DISPLAY-SCALE CHANGE OVER A SHAPED DOCUMENT LEAVES THE SAME ROW
/// PITCH A FRESH PIPELINE AT THAT SCALE WOULD HAVE PRODUCED.
///
/// Swept over the tiers AND over the document axis the old gate partitioned:
/// concealed-but-heading-less markdown (the arm that was stale), markdown with a
/// heading (the arm the gate covered), and plain prose (the arm with no absolute
/// span metrics at all). The fresh-pipeline comparison is the differential half; the
/// body-line pin is the absolute half.
#[test]
fn the_row_pitch_after_a_display_scale_change_matches_a_fresh_pipeline() {
    let _g = crate::testlock::serial();
    let prev_page = crate::page::page_on();
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the row-pitch scale law: no wgpu adapter");
        crate::page::set_page_on(prev_page);
        return;
    }
    assert!(
        TIERS.iter().any(|t| t.fract() != 0.0),
        "the tier list must carry a fractional scale, else an integer-only repair passes"
    );
    let mut graded = 0usize;
    // Each case names its document, the caret line, and the line whose pitch is the
    // BODY pitch — the one that must equal `LINE_HEIGHT * dpi` exactly.
    let cases: [(&str, &str, usize, usize); 3] = [
        ("concealed markdown, no heading", CONCEALED_MD, 1, 1),
        ("markdown with a heading", HEADING_MD, 2, 2),
        ("plain prose", PLAIN, 0, 1),
    ];
    for (name, text, cursor_line, body_line) in cases {
        for &dpi in &TIERS {
            let Some(moved) = pitches_after_scale_change(text, cursor_line, dpi) else {
                continue;
            };
            let Some(reference) = pitches_fresh(text, cursor_line, dpi) else {
                continue;
            };
            assert_eq!(
                moved.len(),
                reference.len(),
                "{name} @ dpi {dpi}: the two orders must see the same lines"
            );
            for (li, (m, r)) in moved.iter().zip(reference.iter()).enumerate() {
                assert!(
                    (m - r).abs() < 1e-3,
                    "{name} @ dpi {dpi}: line {li} row pitch is {m} after the scale \
                     change but {r} on a pipeline born at that scale — the line's own \
                     absolute span metrics were not rebuilt (moved {moved:?} vs fresh \
                     {reference:?})"
                );
            }
            let body = moved[body_line];
            let want = LINE_HEIGHT * dpi;
            assert!(
                (body - want).abs() < 1e-3,
                "{name} @ dpi {dpi}: the body row pitch is {body}, want {want} \
                 (LINE_HEIGHT x the display factor) — a repair that makes both orders \
                 equally wrong still fails here"
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        cases.len() * TIERS.len(),
        "every document x tier cell must be graded"
    );
    crate::page::set_page_on(prev_page);
}

/// CLAIM 2 — THE BAND DERIVED FROM THE ROW GROWS WITH DENSITY, AND BY THE DISPLAY
/// FACTOR.
///
/// The inline-code pill's band is `caret_band_scale`'s quotient of the row height
/// and the live metrics, so it is the reading that made the defect visible. Three
/// halves, all required: the band exists at tier 1 (a presence floor — an invariance
/// claim over a zero band is satisfied by deleting the pill), the band divided by
/// the display factor is INVARIANT, and it is strictly MONOTONIC in the display
/// factor, which pins the sign the defect got backwards.
#[test]
fn the_pill_band_after_a_display_scale_change_grows_with_the_display_factor() {
    let _g = crate::testlock::serial();
    let prev_page = crate::page::page_on();
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the pill-band scale law: no wgpu adapter");
        crate::page::set_page_on(prev_page);
        return;
    }
    let v = md_view(CONCEALED_MD, 1);
    let mut bands: Vec<(f32, f32)> = Vec::new();
    for &dpi in &TIERS {
        let Some((device, queue, mut p)) = headless_dqp(LOGICAL_W, LOGICAL_H) else {
            continue;
        };
        p.set_size(LOGICAL_W, LOGICAL_H);
        p.set_view(&v);
        present(&mut p, &device, &queue, 1.0);
        // ENROLMENT: the subject must EXIST at tier 1, or the claim is a statement
        // about an empty list.
        assert_eq!(
            p.code_pill_rects().len(),
            1,
            "enrolment: the fixture must draw exactly one inline-code pill at dpi 1"
        );
        p.set_dpi(dpi);
        p.set_size(LOGICAL_W * dpi, LOGICAL_H * dpi);
        p.set_view(&v);
        present(&mut p, &device, &queue, dpi);
        let rects = p.code_pill_rects();
        assert_eq!(
            rects.len(),
            1,
            "dpi {dpi}: the pill must survive the scale change: {rects:?}"
        );
        // Divide the pill's OWN inset back out: what is left is the row band, the
        // quantity a stale pitch corrupts.
        let band = rects[0][3] - 2.0 * p.metrics.px(CODE_PILL_INSET_Y);
        bands.push((dpi, band));
    }
    assert_eq!(bands.len(), TIERS.len(), "every tier must be graded");
    let (_, base) = bands[0];
    assert!(
        base > 1.0,
        "presence floor: the tier-1 band must be a real band, got {base}"
    );
    for &(dpi, band) in &bands {
        let logical = band / dpi;
        assert!(
            (logical - base).abs() < 0.75,
            "dpi {dpi}: the pill's band is {band}, i.e. {logical} logical against the \
             tier-1 band of {base} — a band derived from a stale row pitch holds its \
             DEVICE size and reads as shrinking (all tiers: {bands:?})"
        );
    }
    for pair in bands.windows(2) {
        let (lo_dpi, lo) = pair[0];
        let (hi_dpi, hi) = pair[1];
        assert!(
            hi > lo,
            "the band must GROW from dpi {lo_dpi} ({lo}) to dpi {hi_dpi} ({hi}) — the \
             defect's signature is the opposite sign (all tiers: {bands:?})"
        );
    }
    crate::page::set_page_on(prev_page);
}

/// CLAIM 3 — THE REPAIR IS PAID ONCE PER SCALE CHANGE, NEVER PER FRAME.
///
/// The restyle a scale change now performs unconditionally is only sound because the
/// frame path never reaches it. Counted, not reasoned about, and counted on TWO
/// witnesses because one of them cannot see the work: `reshape_count` tracks the
/// preedit-shaping seam only, so a restyle moves it by ZERO — the witness that a
/// restyle DID run is the row-geometry GENERATION, which `restyle_all_lines`
/// invalidates. An idle frame must move neither.
#[test]
fn the_scale_change_restyle_costs_no_reshape_on_an_idle_frame() {
    let _g = crate::testlock::serial();
    let prev_page = crate::page::page_on();
    crate::page::set_page_on(false);
    crate::markdown::set_wysiwyg_on(true);
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping the scale-change reshape-count law: no wgpu adapter");
        crate::page::set_page_on(prev_page);
        return;
    }
    const IDLE_FRAMES: usize = 8;
    // A scale change re-wraps, re-styles and re-shapes the document by design; what
    // it may NOT do is grow with the number of frames that follow it.
    const MAX_RESHAPES_PER_SCALE_CHANGE: u64 = 4;
    const MAX_INVALIDATIONS_PER_SCALE_CHANGE: u64 = 8;
    let dpi = 2.0;
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL_W, LOGICAL_H) else {
        crate::page::set_page_on(prev_page);
        return;
    };
    p.set_size(LOGICAL_W, LOGICAL_H);
    let v = md_view(CONCEALED_MD, 1);
    p.set_view(&v);
    present(&mut p, &device, &queue, 1.0);
    let at_rest = p.reshape_count;
    let gen_at_rest = p.row_geom.generation();
    p.set_dpi(dpi);
    p.set_size(LOGICAL_W * dpi, LOGICAL_H * dpi);
    p.set_view(&v);
    present(&mut p, &device, &queue, dpi);
    let after_move = p.reshape_count;
    let gen_after_move = p.row_geom.generation();
    let move_cost = after_move - at_rest;
    let move_invalidations = gen_after_move - gen_at_rest;
    assert!(
        move_cost <= MAX_RESHAPES_PER_SCALE_CHANGE,
        "one display-scale change cost {move_cost} reshapes, over the \
         {MAX_RESHAPES_PER_SCALE_CHANGE} budget"
    );
    assert!(
        move_invalidations <= MAX_INVALIDATIONS_PER_SCALE_CHANGE,
        "one display-scale change cost {move_invalidations} row-geometry \
         invalidations, over the {MAX_INVALIDATIONS_PER_SCALE_CHANGE} budget"
    );
    // The move must have done SOME row-geometry work, or the two idle assertions
    // below are comparing a repair that never happened against itself.
    assert!(
        move_invalidations > 0,
        "enrolment: the scale change must invalidate the row geometry at least once"
    );
    // ENROLMENT: the pitch really did move, so the frames below idle over a REPAIRED
    // document rather than over one the repair never touched. Read the CONCEALED
    // line, not the plain one — the plain line tracks the scale either way, so an
    // enrolment taken there is satisfied by the defect.
    let pitch = p.visual_rows(0)[0].line_height;
    assert!(
        (pitch - LINE_HEIGHT * dpi).abs() < 1e-3,
        "enrolment: the scale change must have repaired the pitch, got {pitch}"
    );
    for frame in 0..IDLE_FRAMES {
        p.set_view(&v);
        present(&mut p, &device, &queue, dpi);
        let now = p.reshape_count;
        assert_eq!(
            now, after_move,
            "idle frame {frame} after the scale change reshaped: {now} vs {after_move} \
             — the scale-change restyle must not have migrated onto the frame path"
        );
        let gen_now = p.row_geom.generation();
        assert_eq!(
            gen_now, gen_after_move,
            "idle frame {frame} after the scale change invalidated the row geometry: \
             {gen_now} vs {gen_after_move} — a restyle ran on the frame path, which is \
             O(doc) per frame"
        );
    }
    crate::page::set_page_on(prev_page);
}
