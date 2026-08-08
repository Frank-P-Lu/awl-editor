//! THE PIXEL-DIFF HELPER — the LAW ROUND's structural answer to the Wagtail
//! invisible-picker-row bug (six render surfaces shipped invisible across
//! three rounds while every MECHANISM-shaped test — `instance_count == 1`,
//! `dither() > 0.0`, … — passed green; a fully-transparent quad satisfies
//! every one of those assertions). CLAUDE.md's harness section names the
//! rule this file exists to make cheap to follow: **"the sidecar is a STATE
//! oracle, not an APPEARANCE oracle" — appearance-class properties
//! ("visible", "distinct", "legible") must be asserted over the PNG's
//! pixels, never inferred from state.** Before this file, doing that meant
//! hand-rolling a readback + a bespoke pixel loop per test (see `dither.rs`'s
//! own `offscreen`/`read_pixels`, and `one_bit.rs`'s several hand-inlined
//! sampling loops) — this module makes the OUTCOME assertion itself one line:
//! `assert_perceptibly_different(..)` / `assert_identical(..)`.
//!
//! Deterministic, no clock, no filesystem — pure arithmetic over two
//! already-rendered `Vec<[u8;4]>` pixel buffers (the same row-major shape
//! `dither::read_pixels` returns). Doesn't render anything itself; callers
//! still drive `TextPipeline::prepare`/`render` + `dither::{offscreen,
//! read_pixels}` (or the `render_region` convenience wrapper below) exactly
//! as `one_bit.rs`/`dither.rs` already do — this module is the assertion
//! layer on top, not a new rendering path.

use super::super::*;
use super::dither;

/// This pixel's `(L*, a*, b*)` in CIE L\*a\*b\* (D65).
///
/// The sRGB decode is [`theme::Srgb`]'s own — the tree's one sRGB EOTF — rather
/// than a local copy, so a perceptual oracle and the colour the product actually
/// hands the GPU cannot disagree about what "linear" means.
pub(super) fn lab(p: [u8; 4]) -> (f64, f64, f64) {
    let lin = crate::theme::srgb_channel_to_linear;
    let (r, g, b) = (lin(p[0]), lin(p[1]), lin(p[2]));
    // sRGB → CIE XYZ (D65), then XYZ → Lab against the D65 white point.
    let x = (0.4124 * r + 0.3576 * g + 0.1805 * b) / 0.95047;
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let z = (0.0193 * r + 0.1192 * g + 0.9505 * b) / 1.08883;
    let f = |t: f64| {
        if t > 0.008_856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x), f(y), f(z));
    (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
}

/// **CIE 1976 ΔE — the PERCEPTUAL distance between two drawn pixels, and this
/// tree's settled oracle for "can this surface be seen".** One owner, because
/// every appearance floor that asks that question must ask it the same way.
///
/// Neither a WCAG contrast ratio nor a luminance difference answers it, and both
/// were tried before this. **A ratio and a `|ΔY|` each collapse in the dark**,
/// where a plainly visible step between two near-black surfaces measures almost
/// nothing — and both are LUMINANCE-ONLY, so they call a plate that differs from
/// its page in hue or chroma invisible. Potoroo's sticky plate sits ΔL\* 0.87
/// from its page and is unmistakable on screen, the difference almost entirely in
/// b\* (44 against 0); a luminance floor called that invisible and demanded a
/// product change that would have made a legible surface worse.
///
/// For scale: ΔE ≈ 2.3 is the classic just-noticeable difference.
pub(super) fn delta_e(a: [u8; 4], b: [u8; 4]) -> f64 {
    let (l1, a1, b1) = lab(a);
    let (l2, a2, b2) = lab(b);
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

/// A rectangular pixel region in canvas (device-pixel) coordinates. `x`/`y`
/// are the top-left corner; `w`/`h` extend right/down. Coordinates are
/// clamped to the buffer's own bounds by `diff_region`/`sample_region`, so a
/// region that runs slightly past a computed edge (rounding, a `-1`/`+1`
/// overhang like the border-ring tests already use) never panics.
#[derive(Clone, Copy, Debug)]
pub(super) struct Region {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

impl Region {
    pub(super) fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Region {
            x: x as i64,
            y: y as i64,
            w: w as i64,
            h: h as i64,
        }
    }
    /// The whole canvas.
    pub(super) fn canvas(width: i64, height: i64) -> Self {
        Region {
            x: 0,
            y: 0,
            w: width,
            h: height,
        }
    }
}

/// Measured difference between two same-sized pixel buffers over `region`:
/// how many of the region's pixels differ at all, the region's total pixel
/// count, and the largest single-channel delta observed anywhere in it
/// (0 if the region is byte-identical).
#[derive(Clone, Copy, Debug)]
pub(super) struct DiffReport {
    pub differing: usize,
    pub total: usize,
    pub max_channel_delta: u8,
}

impl DiffReport {
    pub(super) fn differing_fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.differing as f32 / self.total as f32
        }
    }
}

/// Walk `region` (clamped to `[0,width) x [0,height)`) over two row-major
/// `width`x`height` pixel buffers and measure how much they differ. A pixel
/// counts as "differing" if ANY of its four channels differ at all; the
/// report's `max_channel_delta` is the single largest per-channel |a-b| seen
/// anywhere in the region, over any channel of any pixel.
pub(super) fn diff_region(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    width: i64,
    height: i64,
    region: Region,
) -> DiffReport {
    assert_eq!(
        a.len(),
        b.len(),
        "diff_region: buffers must be the same size"
    );
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(width);
    let y1 = (region.y + region.h).min(height);
    let mut differing = 0usize;
    let mut total = 0usize;
    let mut max_delta: u8 = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width + x) as usize;
            total += 1;
            let pa = a[i];
            let pb = b[i];
            let mut this_max = 0u8;
            let mut differs = false;
            for c in 0..4 {
                let d = pa[c].abs_diff(pb[c]);
                this_max = this_max.max(d);
                if d != 0 {
                    differs = true;
                }
            }
            if differs {
                differing += 1;
            }
            max_delta = max_delta.max(this_max);
        }
    }
    DiffReport {
        differing,
        total,
        max_channel_delta: max_delta,
    }
}

/// The floor a `DiffReport` must clear to count as "perceptibly different" —
/// BOTH a minimum FRACTION of the region's pixels must differ at all (guards
/// against a single stray anti-aliased pixel counting as "different") AND
/// the largest single-channel delta anywhere in the region must clear a
/// minimum magnitude (guards against a fraction of barely-different pixels —
/// e.g. sub-pixel rounding noise — counting as a real visual change).
/// `DEFAULT` is deliberately conservative: real UI state changes (a fill
/// band, an inverted row, a moved highlight) clear it by a wide margin;
/// genuine anti-aliasing noise between two otherwise-identical renders does
/// not.
#[derive(Clone, Copy, Debug)]
pub(super) struct DistinguishFloor {
    pub min_fraction: f32,
    pub min_max_delta: u8,
}

impl DistinguishFloor {
    pub(super) const DEFAULT: DistinguishFloor = DistinguishFloor {
        min_fraction: 0.01,
        min_max_delta: 12,
    };
}

/// Assert that `region` (same coordinates in both buffers, both sized
/// `width`x`height`) is PERCEPTIBLY DIFFERENT between renders `a` and `b` —
/// the one-line replacement for "does state-on actually look different from
/// state-off". Fails loud with the measured numbers on a miss, so a
/// regression reads as "the highlight band stopped painting" rather than a
/// bare `assert!` false.
pub(super) fn assert_perceptibly_different(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    width: i64,
    height: i64,
    region: Region,
    floor: DistinguishFloor,
    label: &str,
) {
    let report = diff_region(a, b, width, height, region);
    assert!(
        report.total > 0,
        "{label}: region is empty ({region:?}) — nothing to compare"
    );
    let frac = report.differing_fraction();
    assert!(
        frac >= floor.min_fraction && report.max_channel_delta >= floor.min_max_delta,
        "{label}: expected a PERCEPTIBLE difference in {region:?} but got \
         differing_fraction={frac:.4} (floor {:.4}), max_channel_delta={} (floor {}) \
         over {} pixels — the two states render the SAME here, exactly the shape of \
         the Wagtail invisible-picker-row bug (a mechanism fired, the pixels didn't move)",
        floor.min_fraction,
        report.max_channel_delta,
        floor.min_max_delta,
        report.total,
    );
}

/// The inverse assertion: `region` must be BYTE-IDENTICAL between `a` and
/// `b` — used to prove a refactor changed nothing observable (the enum-shape
/// refactor in this round proves Wagtail + a control world render pixel-for-
/// pixel identical before/after).
pub(super) fn assert_identical(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    width: i64,
    height: i64,
    region: Region,
    label: &str,
) {
    let report = diff_region(a, b, width, height, region);
    assert!(
        report.total > 0,
        "{label}: region is empty ({region:?}) — nothing to compare"
    );
    assert_eq!(
        report.differing, 0,
        "{label}: expected byte-identical pixels in {region:?}, but {} of {} pixels differ \
         (max_channel_delta={})",
        report.differing, report.total, report.max_channel_delta,
    );
}

/// The classic CIE ΔE just-noticeable difference — one named constant so
/// every ΔE-based floor in this file cites the same number rather than
/// re-deriving it (see [`delta_e`]'s own doc for why ΔE, not a byte delta or
/// a WCAG ratio, is this tree's oracle for "can this be seen").
const CLASSIC_JND: f64 = 2.3;

/// How many pixels a pair of SAME-POSITION buffers moves at least
/// [`CLASSIC_JND`] apart over `region`, and the single largest ΔE observed
/// anywhere in it — the perceptual analogue of [`DiffReport`]'s `(fraction,
/// max_channel_delta)` pair, for a caller comparing two renders perceptually
/// rather than byte-exact (a byte comparison of two GPU renders is a claim
/// about the rasterizer, not the product).
pub(super) fn pairwise_delta_e_report(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    width: i64,
    height: i64,
    region: Region,
) -> (usize, f64) {
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(width);
    let y1 = (region.y + region.h).min(height);
    let mut covered = 0usize;
    let mut peak = 0.0f64;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width + x) as usize;
            let d = delta_e(a[i], b[i]);
            if d >= CLASSIC_JND {
                covered += 1;
            }
            peak = peak.max(d);
        }
    }
    (covered, peak)
}

/// At least this many pixels must clear the JND for a pair to count as
/// distinct — one lucky anti-aliased pixel cannot stand in for a real shape
/// difference. Mirrors `marker_side_item303.rs`'s own `MARK_MIN_COVERED_CELLS`
/// (also 4, chosen there for the identical reason: a thin stroke against a
/// mostly-empty ground must not pass on population size alone).
pub(super) const PAIRWISE_MIN_COVERED_PX: usize = 4;

/// The pair's single largest ΔE must clear this — comfortably past the
/// classic 2.3 JND, the same order of margin this tree's own ΔE ceilings
/// already sit at rather than the JND itself (item 346 shipped its presence
/// floor with margin at both ends, never flush against the JND).
pub(super) const PAIRWISE_MIN_PEAK_DELTA_E: f64 = 6.0;

/// **ONE OWNER of "can a human tell candidate A from candidate B in the
/// artifact they were handed"** — the check this tree did not have until
/// item 349's vision smoke could tell "before" from four candidate marks but
/// not the four apart from EACH OTHER, because each was cropped to its own
/// bounding box at ~77x39 device px and upscaled independently. That
/// independent crop-and-rescale is what erased the very quantity (vertex
/// angle) the candidates differed in: two candidates occupying different
/// logical footprints, each blown up to fill the same thumbnail cell, share
/// no ruler — a genuinely different angle can render pixel-for-pixel
/// identical once each side has been independently renormalized to fit.
///
/// So this check does NOT crop or rescale anything itself. Candidates arrive
/// **already composited into ONE shared `width`x`height` frame at ONE
/// zoom/dpi** — the coordinate space a human actually looks at — and every
/// buffer failing to match that shared size is refused rather than silently
/// skipped, because a mismatched size IS the own-bounding-box-crop defect,
/// caught before it can launder a real difference away.
///
/// Two obligations precede any pairwise reading, so the check cannot pass by
/// grading nothing:
/// - `candidates.len() >= 2` — "every adjacent pair is distinct" is
///   vacuously true of a set with fewer than two members, and this repo has
///   shipped laws satisfiable by deleting their own subject.
/// - every buffer is exactly `width * height` long.
///
/// Then every **adjacent** pair — the order the gallery lays them out in,
/// the order a reader's eye actually compares — must clear BOTH
/// [`PAIRWISE_MIN_COVERED_PX`] pixels past the JND and a peak ΔE of
/// [`PAIRWISE_MIN_PEAK_DELTA_E`] (see their own docs for why two numbers, not
/// one). `candidates` is whatever slice the caller renders and names; this
/// function enrols nothing of its own and keeps no name list, so a caller
/// building a REAL comparison gallery drives it from wherever it already
/// declares its own candidates.
pub(super) fn assert_pairwise_distinct(
    candidates: &[(&str, &[[u8; 4]])],
    width: i64,
    height: i64,
    label: &str,
) {
    assert!(
        candidates.len() >= 2,
        "{label}: a comparison set of {} member(s) is not a comparison — \
         pairwise distinctness is vacuously true of a set with fewer than \
         two members",
        candidates.len()
    );
    let region = Region::canvas(width, height);
    let expect_len = (width * height) as usize;
    for pair in candidates.windows(2) {
        let (name_a, buf_a) = pair[0];
        let (name_b, buf_b) = pair[1];
        assert_eq!(
            buf_a.len(),
            expect_len,
            "{label}: candidate {name_a:?} is not {width}x{height} px — every \
             candidate must be rendered into the SAME shared frame at the \
             artifact's own scale, never cropped to its own bounding box and \
             rescaled independently (that renormalization is what hid a real \
             vertex-angle difference in the gallery item 350 was named for)"
        );
        assert_eq!(
            buf_b.len(),
            expect_len,
            "{label}: candidate {name_b:?} is not {width}x{height} px — see \
             {name_a:?}'s message above"
        );
        let (covered, peak) = pairwise_delta_e_report(buf_a, buf_b, width, height, region);
        assert!(
            covered >= PAIRWISE_MIN_COVERED_PX && peak >= PAIRWISE_MIN_PEAK_DELTA_E,
            "{label}: candidates {name_a:?} and {name_b:?} are not \
             distinguishable at this artifact's own {width}x{height} scale — \
             {covered} px cleared the {CLASSIC_JND} JND (floor \
             {PAIRWISE_MIN_COVERED_PX}), peak ΔE {peak:.2} (floor \
             {PAIRWISE_MIN_PEAK_DELTA_E}). A separation that is real at full \
             render resolution and invisible here is exactly the gap this \
             check exists to close."
        );
    }
}

/// The single most-common "ink" pixel (any pixel differing from `bg` by more
/// than `threshold` in any channel) over `region` — `None` if the region has
/// no ink at all. A glyph's own anti-aliased edge pixels are a minority next
/// to its solid fill, so the MODE (not the mean, which an edge's partial
/// coverage would drag toward `bg`) is a robust stand-in for "what color is
/// this text drawn in", usable to compare two DIFFERENT regions' text ink
/// for an exact real-pixel match (e.g. "does a table cell's ink match
/// ordinary body prose's ink") without needing to hand-pick a single glyph
/// pixel. Threshold + region semantics mirror [`diff_region`].
pub(super) fn dominant_ink_color(
    pixels: &[[u8; 4]],
    width: i64,
    height: i64,
    region: Region,
    bg: [u8; 4],
    threshold: u8,
) -> Option<[u8; 4]> {
    use std::collections::HashMap;
    let x0 = region.x.max(0);
    let y0 = region.y.max(0);
    let x1 = (region.x + region.w).min(width);
    let y1 = (region.y + region.h).min(height);
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in y0..y1 {
        for x in x0..x1 {
            let p = pixels[(y * width + x) as usize];
            let d = p[0]
                .abs_diff(bg[0])
                .max(p[1].abs_diff(bg[1]))
                .max(p[2].abs_diff(bg[2]));
            if d > threshold {
                *counts.entry(p).or_insert(0) += 1;
            }
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(p, _)| p)
}

/// One horizontal INK/gap band in a single-row column scan — [`ink_column_bands`]'s
/// output unit. `x0..=x1` is an inclusive canvas-column range (both ends real
/// columns, matching the loop below).
#[derive(Clone, Copy, Debug)]
pub(super) struct ColBand {
    pub ink: bool,
    // Read via `{:?}` in a failing assertion's message (dead-code analysis
    // ignores `Debug`-only field reads); also here for a future caller that
    // wants a band's own width, not just its ink/gap classification.
    #[allow(dead_code)]
    pub x0: i64,
    #[allow(dead_code)]
    pub x1: i64,
}

/// Collapse the column range `[x0,x1)` of the row band `[y0,y1)` into
/// alternating ink/gap [`ColBand`]s: a column counts as "ink" if ANY pixel in
/// its `y0..y1` span differs from `bg` by more than `threshold` in any single
/// channel. The theme-QA round's own bullet-padding probe (a hand-rolled
/// Python script over the capture PNG) promoted here so a padding/touching
/// law can assert "how many separate ink blobs sit in this strip" — a
/// direct, appearance-level test of "does the bullet glyph touch the text
/// that follows it" (a single merged band == touching; two bands with a real
/// gap between them == not touching) — rather than inferring it from
/// geometry. Real GPU pixels, the Wagtail lesson (CLAUDE.md's harness
/// section): appearance is proven over bytes, never state.
// Pixel-band probes keep their scan bounds and threshold explicit for readable laws.
#[allow(clippy::too_many_arguments)]
pub(super) fn ink_column_bands(
    pixels: &[[u8; 4]],
    width: i64,
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
    bg: [u8; 4],
    threshold: u8,
) -> Vec<ColBand> {
    let mut bands: Vec<ColBand> = Vec::new();
    let mut cur: Option<bool> = None;
    let mut start = x0;
    let mut x = x0;
    while x < x1 {
        let mut ink = false;
        for y in y0..y1 {
            let idx = y * width + x;
            if idx < 0 || idx as usize >= pixels.len() {
                continue;
            }
            let p = pixels[idx as usize];
            let d = p[0]
                .abs_diff(bg[0])
                .max(p[1].abs_diff(bg[1]))
                .max(p[2].abs_diff(bg[2]));
            if d > threshold {
                ink = true;
                break;
            }
        }
        match cur {
            None => {
                cur = Some(ink);
                start = x;
            }
            Some(c) if c != ink => {
                bands.push(ColBand {
                    ink: c,
                    x0: start,
                    x1: x - 1,
                });
                cur = Some(ink);
                start = x;
            }
            _ => {}
        }
        x += 1;
    }
    if let Some(c) = cur {
        bands.push(ColBand {
            ink: c,
            x0: start,
            x1: x1 - 1,
        });
    }
    bands
}

/// [`ink_column_bands`]'s ROW-axis mirror: collapse the row range `[y0,y1)`
/// of the column band `[x0,x1)` into alternating ink/gap [`ColBand`]s (its
/// `x0`/`x1` fields hold this scan's row range instead — the type is
/// axis-neutral, an ink/gap band with a `lo..=hi` extent regardless of which
/// axis produced it). A row counts as "ink" if ANY pixel in its `x0..x1` span
/// differs from `bg` by more than `threshold` in any single channel. Where
/// [`ink_column_bands`] answers "does this glyph touch that one" (a
/// horizontal question), this answers "how tall is the BLANK GAP between two
/// blocks" (a vertical one) — the theme-QA round's heading-spacing law: a
/// no-bold world's gap AROUND a heading must read measurably taller than the
/// gap between two ordinary body paragraphs, at real pixels.
// Pixel-band probes keep their scan bounds and threshold explicit for readable laws.
#[allow(clippy::too_many_arguments)]
pub(super) fn ink_row_bands(
    pixels: &[[u8; 4]],
    width: i64,
    height: i64,
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
    bg: [u8; 4],
    threshold: u8,
) -> Vec<ColBand> {
    let mut bands: Vec<ColBand> = Vec::new();
    let mut cur: Option<bool> = None;
    let mut start = y0;
    let mut y = y0;
    while y < y1 {
        let mut ink = false;
        if y >= 0 && y < height {
            for x in x0..x1 {
                let idx = y * width + x;
                if idx < 0 || idx as usize >= pixels.len() {
                    continue;
                }
                let p = pixels[idx as usize];
                let d = p[0]
                    .abs_diff(bg[0])
                    .max(p[1].abs_diff(bg[1]))
                    .max(p[2].abs_diff(bg[2]));
                if d > threshold {
                    ink = true;
                    break;
                }
            }
        }
        match cur {
            None => {
                cur = Some(ink);
                start = y;
            }
            Some(c) if c != ink => {
                bands.push(ColBand {
                    ink: c,
                    x0: start,
                    x1: y - 1,
                });
                cur = Some(ink);
                start = y;
            }
            _ => {}
        }
        y += 1;
    }
    if let Some(c) = cur {
        bands.push(ColBand {
            ink: c,
            x0: start,
            x1: y1 - 1,
        });
    }
    bands
}

/// Render the pipeline's CURRENT prepared state (whatever the caller already
/// set via `set_view`/`prepare`) to an offscreen `width`x`height` texture and
/// read it back as a flat row-major `Vec<[u8;4]>` — the exact `dither`-module
/// readback dance every real-pixel test in this tree already hand-rolls,
/// pulled out to ONE call so a NEW real-pixel test doesn't have to re-inline
/// it a third/fourth time (mirrors `dither.rs`'s own doc note on why the
/// FIRST such duplication, versus `capture/gpu.rs`, is itself accepted).
pub(super) fn render_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = dither::offscreen(device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl pixeldiff-test encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    dither::read_pixels(device, queue, &texture, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_region_counts_differing_pixels_and_max_delta() {
        let w = 4i64;
        let h = 2i64;
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        let mut b = a.clone();
        b[0] = [10, 0, 0, 255]; // one pixel differs by 10
        b[5] = [0, 0, 0, 255]; // identical
        let report = diff_region(&a, &b, w, h, Region::canvas(w, h));
        assert_eq!(report.total, 8);
        assert_eq!(report.differing, 1);
        assert_eq!(report.max_channel_delta, 10);
    }

    #[test]
    fn region_clamps_to_buffer_bounds_never_panics() {
        let w = 4i64;
        let h = 4i64;
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        let b = a.clone();
        // A region hanging off every edge — must clamp, not panic or underflow.
        let report = diff_region(
            &a,
            &b,
            w,
            h,
            Region {
                x: -2,
                y: -2,
                w: 100,
                h: 100,
            },
        );
        assert_eq!(report.total, 16);
        assert_eq!(report.differing, 0);
    }

    #[test]
    fn assert_perceptibly_different_passes_on_a_real_change() {
        let w = 4i64;
        let h = 4i64;
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        let mut b = a.clone();
        for p in b.iter_mut() {
            *p = [255, 255, 255, 255];
        }
        assert_perceptibly_different(
            &a,
            &b,
            w,
            h,
            Region::canvas(w, h),
            DistinguishFloor::DEFAULT,
            "test fixture",
        );
    }

    #[test]
    #[should_panic(expected = "expected a PERCEPTIBLE difference")]
    fn assert_perceptibly_different_fails_when_nothing_moved() {
        let w = 4i64;
        let h = 4i64;
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        let b = a.clone();
        assert_perceptibly_different(
            &a,
            &b,
            w,
            h,
            Region::canvas(w, h),
            DistinguishFloor::DEFAULT,
            "test fixture",
        );
    }

    #[test]
    fn assert_identical_passes_on_byte_identical_buffers() {
        let w = 4i64;
        let h = 4i64;
        let a = vec![[12u8, 34, 56, 255]; (w * h) as usize];
        let b = a.clone();
        assert_identical(&a, &b, w, h, Region::canvas(w, h), "test fixture");
    }

    #[test]
    #[should_panic(expected = "expected byte-identical pixels")]
    fn assert_identical_fails_on_any_difference() {
        let w = 4i64;
        let h = 4i64;
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        let mut b = a.clone();
        b[3] = [1, 0, 0, 255];
        assert_identical(&a, &b, w, h, Region::canvas(w, h), "test fixture");
    }

    #[test]
    fn assert_pairwise_distinct_passes_on_genuinely_different_candidates() {
        let (w, h) = (4i64, 4i64);
        let n = (w * h) as usize;
        let black = vec![[0u8, 0, 0, 255]; n];
        let white = vec![[255u8, 255, 255, 255]; n];
        // A, B, C alternate — every ADJACENT pair (A/B, B/C) must clear the
        // floor; this does not require the non-adjacent A/C pair to.
        assert_pairwise_distinct(
            &[
                ("A", black.as_slice()),
                ("B", white.as_slice()),
                ("C", black.as_slice()),
            ],
            w,
            h,
            "test fixture",
        );
    }

    #[test]
    #[should_panic(expected = "a comparison set of 1 member(s) is not a comparison")]
    fn assert_pairwise_distinct_refuses_a_set_of_one() {
        let (w, h) = (4i64, 4i64);
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        assert_pairwise_distinct(&[("A", a.as_slice())], w, h, "test fixture");
    }

    #[test]
    #[should_panic(expected = "is not 4x4 px")]
    fn assert_pairwise_distinct_refuses_a_mismatched_buffer_size() {
        let (w, h) = (4i64, 4i64);
        let a = vec![[0u8, 0, 0, 255]; (w * h) as usize];
        // A candidate cropped to its own (smaller) bounding box rather than
        // rendered into the shared frame — the exact renormalization defect
        // this check exists to refuse rather than silently skip.
        let b = vec![[255u8, 255, 255, 255]; 3];
        assert_pairwise_distinct(
            &[("A", a.as_slice()), ("B", b.as_slice())],
            w,
            h,
            "test fixture",
        );
    }

    /// THE NATURAL MUTATION: make two adjacent candidates identical and
    /// confirm the check refuses them, by name.
    #[test]
    #[should_panic(expected = "candidates \"B\" and \"C\" are not distinguishable")]
    fn assert_pairwise_distinct_refuses_two_identical_candidates() {
        let (w, h) = (4i64, 4i64);
        let n = (w * h) as usize;
        let black = vec![[0u8, 0, 0, 255]; n];
        let white = vec![[255u8, 255, 255, 255]; n];
        let mutated_c = white.clone(); // MUTATION: C collapsed onto B
        assert_pairwise_distinct(
            &[
                ("A", black.as_slice()),
                ("B", white.as_slice()),
                ("C", mutated_c.as_slice()),
            ],
            w,
            h,
            "test fixture",
        );
    }

    /// THE SCALE CLAIM, proven rather than asserted in a doc comment: the
    /// SAME real angle difference that this check finds at a workable frame
    /// size can fail to clear the floor once minified hard enough — which is
    /// exactly the shape of item 349's defect (a real vertex-angle
    /// difference, invisible in a ~77x39 device-px crop). Two candidates
    /// differ only in a diagonal edge's slope; downsampled by box-averaging
    /// (the same blending a thumbnail resize performs) to a small enough
    /// frame, the edge's few differing pixels dilute below this check's own
    /// floor, at the SAME floor constants used at the fine scale — proving
    /// "measure at the artifact's own scale" is not a slogan: the verdict
    /// itself flips with the frame size. Measured on this host: 40x40 reads
    /// covered=78, peak ΔE 93.68; minified to 4x4 it still clears both floors
    /// (covered=6, peak 8.36) and even 3x3 barely clears them (covered=4,
    /// peak 6.60) — so the frame this test minifies to is 2x2, where the
    /// SAME real difference reads covered=2, peak ΔE 4.16, under both floors.
    #[test]
    fn assert_pairwise_distinct_a_real_angle_difference_can_vanish_when_minified() {
        let (w, h) = (40i64, 40i64);
        // Candidate A: a shallow edge (rise 1 for every 2 columns).
        // Candidate B: a steep edge (rise 1 per column) — a substantial,
        // genuinely different vertex angle (~26 degrees apart).
        let draw = |steep: bool| -> Vec<[u8; 4]> {
            let mut buf = vec![[255u8, 255, 255, 255]; (w * h) as usize];
            for x in 0..w {
                let y = if steep { x } else { x / 2 };
                if y < h {
                    buf[(y * w + x) as usize] = [20, 20, 20, 255];
                }
            }
            buf
        };
        let a = draw(false);
        let b = draw(true);

        // AT THE FULL 40x40 FRAME: a real, findable difference.
        assert_pairwise_distinct(
            &[("shallow", a.as_slice()), ("steep", b.as_slice())],
            w,
            h,
            "fine scale",
        );

        // MINIFIED by box-averaging to a small shared frame — still a SHARED
        // frame (never an independent per-candidate crop), just a much
        // coarser one, which is the load-bearing axis item 350 named.
        let minify = |buf: &[[u8; 4]], tw: i64, th: i64| -> Vec<[u8; 4]> {
            let mut out = vec![[0u8; 4]; (tw * th) as usize];
            let (bw, bh) = (w / tw, h / th);
            for ty in 0..th {
                for tx in 0..tw {
                    let mut sum = [0u32; 3];
                    let mut count = 0u32;
                    for sy in (ty * bh)..((ty + 1) * bh) {
                        for sx in (tx * bw)..((tx + 1) * bw) {
                            let p = buf[(sy * w + sx) as usize];
                            for c in 0..3 {
                                sum[c] += p[c] as u32;
                            }
                            count += 1;
                        }
                    }
                    out[(ty * tw + tx) as usize] = [
                        (sum[0] / count) as u8,
                        (sum[1] / count) as u8,
                        (sum[2] / count) as u8,
                        255,
                    ];
                }
            }
            out
        };
        let (tw, th) = (2i64, 2i64);
        let a_small = minify(&a, tw, th);
        let b_small = minify(&b, tw, th);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_pairwise_distinct(
                &[
                    ("shallow", a_small.as_slice()),
                    ("steep", b_small.as_slice()),
                ],
                tw,
                th,
                "coarse scale",
            )
        }));
        assert!(
            result.is_err(),
            "a 40x40 real angle difference, box-averaged down to a 2x2 shared \
             frame, must NOT still read as distinguishable at this check's own \
             floor — if it does, this test cannot demonstrate the load-bearing \
             claim that scale changes the verdict"
        );
    }
}
