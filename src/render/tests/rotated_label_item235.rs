//! ITEM 235 — the ROTATED LABEL's PIXELS: does a turned run actually reach the
//! screen, and is it still legible when it gets there?
//!
//! The run's frame (axis, quad, bounds, hit test) is graded purely in
//! `rotated_label::tests`. This file grades what a rotated label LOOKS LIKE,
//! because legibility is the point of a label and legibility is not a property
//! anyone can assert — the sidecar has famously reported a row selected while
//! it rendered invisible.
//!
//! THE ORACLE IS NOT THE ROTATION. Ground truth is the CPU-composed coverage
//! image (`rotated_label::mask::compose_run`) — the run rasterised upright by
//! the same swash cache glyphon draws through, an image no rotation code
//! touches. Every rendered angle is DEROTATED back onto that image's own grid
//! by an inverse rotation written out longhand below, from the angle in
//! degrees, never by calling the product's own `label_local`. Then the two
//! images are compared pixel for pixel.
//!
//! The numbers that come out are the legibility measurement:
//!
//! - `ink` — total coverage rendered, as a fraction of the truth's. A rotation
//!   that smears or drops ink moves this off 1.
//! - `mae` — mean absolute coverage error per texel.
//! - `core` — mean rendered coverage where the truth is a solid stroke
//!   (≥ 0.9), as a fraction of what the truth has there. This is how BLACK the
//!   letters still are, graded against the upright render rather than against
//!   an ideal the upright render does not reach either.
//! - `gap` — mean rendered coverage where the truth is empty. This is how much
//!   ink has bled into the counters and the space between stems.
//! - `contrast` — the stroke-to-gap coverage separation, again as a fraction of
//!   the truth's own. A blurred label loses it from both ends at once, and it
//!   is the number that says whether the letters are still separable.
//!
//! Graded at every angle the two blocked world expressions need — the quarter
//! turn, the half turn a mirrored cue takes, and the diagonal worlds' slant in
//! both directions — at 1× and 2× DPI.

use super::super::*;
use super::{dither, headless_dqp};
use crate::rotated_label::RotatedLabelPipeline;
use crate::rotated_label::geometry::{InkBox, label_axis_deg, label_bounds};
use crate::rotated_label::mask::{LabelMask, compose_run};

/// The string the flush-left vertical heading actually needs, so the numbers
/// are measured on the real label rather than a convenient one.
const LABEL: &str = "Files";

/// The diagonal worlds' slant. Their row spine advances 7 logical px sideways
/// per 32 px row it descends — 12.3° off vertical, so 77.7° off horizontal.
const SLANT_DEG: f32 = 77.66;

/// Every angle the pixels are graded at. Signed slants because the roster
/// carries both an ascending and a descending diagonal, and 180° because a
/// mirrored cue turns through it.
const GRADED_ANGLES: [f32; 7] = [
    0.0,
    90.0,
    270.0,
    180.0,
    SLANT_DEG,
    -SLANT_DEG,
    180.0 - SLANT_DEG,
];

/// Canvas edge. Big enough that the longest run fits at any angle with room
/// for the bounds check to see empty pixels on all four sides.
const CANVAS: u32 = 256;

/// The pen origin, integral so the UPRIGHT draw lands fragment centres exactly
/// on texel centres — that is what makes the 0° arm an exactness claim rather
/// than a resampling one.
const ORIGIN: [f32; 2] = [96.0, 148.0];

/// `core` and `contrast` are RATIOS against the truth's own numbers, never
/// absolutes: swash's anti-aliased strokes do not average full coverage even
/// upright, and a law that demanded they did would be grading against an ideal
/// nothing reaches.
struct Measured {
    ink: f32,
    mae: f32,
    core: f32,
    gap: f32,
    contrast: f32,
}

impl std::fmt::Display for Measured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ink {:.4} mae {:.4} core {:.4} gap {:.4} contrast {:.4}",
            self.ink, self.mae, self.core, self.gap, self.contrast
        )
    }
}

/// Shape `LABEL` at `px` into a one-line buffer — the caller's job in the real
/// product too: the label capability shapes nothing itself.
fn label_buffer(font_system: &mut FontSystem, px: f32) -> GlyphBuffer {
    let mut buf = GlyphBuffer::new(font_system, GlyphMetrics::new(px, px * 1.35));
    buf.set_size(font_system, Some(CANVAS as f32), Some(px * 2.0));
    buf.set_wrap(font_system, glyphon::Wrap::None);
    buf.set_text(
        font_system,
        LABEL,
        &Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
        None,
    );
    buf.shape_until_scroll(font_system, false);
    buf
}

/// Draw one label at `deg` alone on a transparent canvas and read the ALPHA
/// channel back as coverage in `[0, 1]`.
///
/// Alpha, not colour: the target is sRGB-encoded but its alpha channel is not,
/// so the byte that comes back IS the coverage the shader wrote, with no
/// transfer function in the way.
fn render_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mask: &LabelMask,
    deg: f32,
    gradient: bool,
) -> (Vec<f32>, Vec<[u8; 4]>) {
    let mut pipe = RotatedLabelPipeline::new(device, dither::FMT);
    let (a, b) = if gradient {
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0])
    } else {
        ([1.0, 1.0, 1.0], [1.0, 1.0, 1.0])
    };
    pipe.prepare(
        device,
        queue,
        CANVAS,
        CANVAS,
        mask,
        ORIGIN,
        label_axis_deg(deg),
        a,
        b,
        1.0,
    );
    assert!(pipe.is_drawn(), "{deg}°: nothing prepared");
    let (texture, view) = dither::offscreen(device, CANVAS, CANVAS);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl rotated-label test encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl rotated-label test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pipe.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));
    let rgba = dither::read_pixels(device, queue, &texture, CANVAS, CANVAS);
    let alpha = rgba.iter().map(|p| p[3] as f32 / 255.0).collect();
    (alpha, rgba)
}

/// Bilinear coverage at a fractional canvas point; 0 outside the canvas.
fn sample(alpha: &[f32], p: [f32; 2]) -> f32 {
    let (fx, fy) = (p[0] - 0.5, p[1] - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let at = |x: f32, y: f32| -> f32 {
        if x < 0.0 || y < 0.0 || x >= CANVAS as f32 || y >= CANVAS as f32 {
            return 0.0;
        }
        alpha[(y as u32 * CANVAS + x as u32) as usize]
    };
    let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
    let bot = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
    top * (1.0 - ty) + bot * ty
}

/// Derotate the rendered canvas back onto the truth image's own grid and grade
/// it. The inverse rotation is built HERE, longhand from `deg`, so a sign error
/// in the product cannot cancel against the oracle.
fn measure(alpha: &[f32], truth: &[u8], mw: u32, mh: u32, ink: InkBox, deg: f32) -> Measured {
    // Screen pixels are y-down, so a reader's counter-clockwise turn is a
    // negative turn here — the same fact the product states, arrived at
    // independently and NOT snapped to the exact quadrant values, so the two
    // are not the same arithmetic.
    let (s, c) = (-deg.to_radians()).sin_cos();
    let (ax, perp) = ([c, s], [-s, c]);

    let (mut sum_t, mut sum_r, mut sum_e) = (0.0f32, 0.0f32, 0.0f32);
    let (mut core, mut core_truth, mut n_core) = (0.0f32, 0.0f32, 0usize);
    let (mut gap, mut n_gap) = (0.0f32, 0usize);
    for j in 0..mh {
        for i in 0..mw {
            let u = ink[0] + i as f32 + 0.5;
            let v = ink[1] + j as f32 + 0.5;
            let p = [
                ORIGIN[0] + u * ax[0] + v * perp[0],
                ORIGIN[1] + u * ax[1] + v * perp[1],
            ];
            let r = sample(alpha, p);
            let t = truth[(j * mw + i) as usize] as f32 / 255.0;
            sum_t += t;
            sum_r += r;
            sum_e += (r - t).abs();
            if t >= 0.9 {
                core += r;
                core_truth += t;
                n_core += 1;
            } else if t == 0.0 {
                gap += r;
                n_gap += 1;
            }
        }
    }
    let n = (mw * mh) as f32;
    assert!(
        n_core > 20 && n_gap > 20,
        "{deg}°: the truth image has no strokes ({n_core}) or no gaps ({n_gap}) to grade"
    );
    let core = core / n_core as f32;
    let gap = gap / n_gap as f32;
    let core_truth = core_truth / n_core as f32;
    Measured {
        ink: sum_r / sum_t.max(1e-6),
        mae: sum_e / n,
        core: core / core_truth,
        gap,
        // The truth's own gap coverage is zero by construction, so its contrast
        // IS `core_truth` and the ratio is the separation that survived.
        contrast: (core - gap) / core_truth,
    }
}

/// THE LEGIBILITY LAW. At every graded angle and both DPIs, the rendered label
/// must still be the composed run: its ink mass preserved, its strokes still
/// solid, its counters and inter-stem gaps still empty.
///
/// The thresholds are the MEASURED worst case with margin, not aspirations —
/// a label that merely "appears" would sail past a loose bound and fail a
/// reader. The 0° arm is held far tighter than the rest because it is not a
/// resample at all: the quad exactly covers the mask, so the composed coverage
/// must arrive on screen essentially byte for byte.
#[test]
fn a_rotated_label_stays_legible_at_every_graded_angle_and_both_dpis() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(CANVAS as f32, CANVAS as f32) else {
        eprintln!("skipping a_rotated_label_stays_legible: no wgpu adapter");
        return;
    };
    // 1× and 2×: a label is rasterised at PHYSICAL pixels, so a retina display
    // is a bigger font size and a bigger mask, not a scaled quad.
    for dpi in [1.0f32, 2.0] {
        let buf = label_buffer(&mut p.font_system, 15.0 * dpi);
        let (truth, ink, mw, mh) = compose_run(&mut p.font_system, &mut p.swash_cache, &buf)
            .expect("the label composes to real ink");
        let mask = LabelMask::compose(
            &device,
            &queue,
            &mut p.font_system,
            &mut p.swash_cache,
            &buf,
        )
        .expect("the label uploads");
        assert_eq!(mask.size(), (mw, mh));
        assert_eq!(mask.ink(), ink);
        // Measure every angle FIRST, print the whole table, then grade it: a
        // failure at one angle must not hide the numbers at the others.
        let measured: Vec<(f32, Measured)> = GRADED_ANGLES
            .iter()
            .map(|&deg| {
                let (alpha, _) = render_at(&device, &queue, &mask, deg, false);
                (deg, measure(&alpha, &truth, mw, mh, ink, deg))
            })
            .collect();
        for (deg, m) in &measured {
            eprintln!("rotated label {dpi}x mask {mw}x{mh} {deg:>7.2} deg  {m}");
        }
        for (deg, m) in &measured {
            let quadrant = deg.rem_euclid(90.0) < 1e-3;
            let (ink_tol, mae_max, core_min, gap_max, contrast_min) = if *deg == 0.0 {
                (0.005, 0.002, 0.999, 0.002, 0.999)
            } else if quadrant {
                (0.02, 0.02, 0.98, 0.02, 0.96)
            } else {
                (0.02, 0.13, 0.65, 0.08, 0.60)
            };
            let ctx = format!("{dpi}x {deg} deg: {m}");
            assert!((m.ink - 1.0).abs() <= ink_tol, "ink mass moved — {ctx}");
            assert!(m.mae <= mae_max, "coverage error — {ctx}");
            assert!(m.core >= core_min, "strokes went hollow — {ctx}");
            assert!(m.gap <= gap_max, "ink bled into the gaps — {ctx}");
            assert!(
                m.contrast >= contrast_min,
                "strokes stopped separating from their gaps — {ctx}"
            );
        }
    }
}

/// Coverage at run-local `(u, v)` in the composed truth, bilinear with
/// clamp-to-edge — the same reconstruction the sampler performs, written here
/// from the image alone.
fn truth_at(truth: &[u8], mw: u32, mh: u32, ink: InkBox, u: f32, v: f32) -> f32 {
    let (fx, fy) = (u - ink[0] - 0.5, v - ink[1] - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let at = |x: f32, y: f32| -> f32 {
        let x = (x.max(0.0) as u32).min(mw - 1);
        let y = (y.max(0.0) as u32).min(mh - 1);
        truth[(y * mw + x) as usize] as f32 / 255.0
    };
    let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
    let bot = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
    top * (1.0 - ty) + bot * ty
}

/// The canvas an IDEAL single bilinear rotation of the truth would produce:
/// for every pixel centre, map back into the run's frame and reconstruct. The
/// rigid raster rotation is DEFINED by this, and it is written here from the
/// angle in degrees and the composed image alone — no product code runs.
fn cpu_rotate(truth: &[u8], mw: u32, mh: u32, ink: InkBox, deg: f32) -> Vec<f32> {
    let (s, c) = (-deg.to_radians()).sin_cos();
    let (ax, perp) = ([c, s], [-s, c]);
    let mut out = vec![0.0f32; (CANVAS * CANVAS) as usize];
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            let d = [x as f32 + 0.5 - ORIGIN[0], y as f32 + 0.5 - ORIGIN[1]];
            let u = d[0] * ax[0] + d[1] * ax[1];
            let v = d[0] * perp[0] + d[1] * perp[1];
            // Outside the quad the rasteriser writes nothing at all.
            if u < ink[0] || u > ink[0] + ink[2] || v < ink[1] || v > ink[1] + ink[3] {
                continue;
            }
            out[(y * CANVAS + x) as usize] = truth_at(truth, mw, mh, ink, u, v);
        }
    }
    out
}

/// THE ROTATION IS OPTIMAL FOR ITS CLASS — the law that says what the slant
/// numbers above MEAN.
///
/// A rigid rotation of a raster costs one resample; nothing can avoid it
/// without more source resolution. The question a legibility figure cannot
/// answer on its own is whether the softening measured at a slant is that
/// unavoidable cost or an implementation losing ink on top of it. This law
/// answers it: the rendered canvas is compared, pixel for pixel, against the
/// canvas an ideal bilinear rigid rotation produces, computed on the CPU from
/// the composed image and the angle alone.
///
/// It is also the strongest possible check of the shader's uv mapping. An
/// off-by-half-texel uv, a quad whose extent did not match the mask, an
/// unnormalised axis, or a perpendicular pointing the wrong way all move
/// coverage the reference does not move.
#[test]
fn the_rendered_rotation_matches_an_ideal_bilinear_resample() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(CANVAS as f32, CANVAS as f32) else {
        eprintln!("skipping the_rendered_rotation_matches_an_ideal_bilinear_resample: no adapter");
        return;
    };
    for dpi in [1.0f32, 2.0] {
        let buf = label_buffer(&mut p.font_system, 15.0 * dpi);
        let (truth, ink, mw, mh) = compose_run(&mut p.font_system, &mut p.swash_cache, &buf)
            .expect("the label composes to real ink");
        let mask = LabelMask::compose(
            &device,
            &queue,
            &mut p.font_system,
            &mut p.swash_cache,
            &buf,
        )
        .expect("the label uploads");
        for deg in GRADED_ANGLES {
            let (alpha, _) = render_at(&device, &queue, &mask, deg, false);
            let want = cpu_rotate(&truth, mw, mh, ink, deg);
            let mut worst = 0.0f32;
            let mut sum = 0.0f32;
            let mut lit = 0usize;
            for (g, w) in alpha.iter().zip(want.iter()) {
                worst = worst.max((g - w).abs());
                sum += (g - w).abs();
                if *w > 0.02 {
                    lit += 1;
                }
            }
            let mean = sum / (CANVAS * CANVAS) as f32;
            eprintln!(
                "rotated label vs ideal {dpi}x {deg:>7.2} deg  \
worst {worst:.4} mean {mean:.6} reference-lit {lit}"
            );
            // NON-VACUOUS: the reference must actually have ink on it, or a
            // pipeline that drew nothing would match a canvas of zeros.
            assert!(
                lit > 150,
                "{dpi}x {deg}: reference has only {lit} lit pixels"
            );
            // The residual is 8-bit quantisation on both sides plus the
            // sampler's fixed-point filter weights, and a fragment either side
            // of the quad's own edge; nothing structural fits under it.
            assert!(
                worst <= 0.02 && mean <= 0.0005,
                "{dpi}x {deg} deg: the render departs from an ideal bilinear rotation — \
                 worst {worst:.4}, mean {mean:.6}"
            );
        }
    }
}

/// THE LABEL LANDS WHERE THE GEOMETRY SAYS. The ink a consumer measures with
/// `label_bounds` — to reserve a gutter, or to prove a cue overlaps no row —
/// must be the ink actually drawn.
///
/// This is what pins the ORIENTATION absolutely. The legibility law derotates,
/// and a derotation can in principle cancel a mirrored render; a bounding box
/// measured straight off the canvas cannot. A flipped `perp`, a positive
/// instead of negative turn, or an axis and perpendicular swapped all move the
/// ink somewhere this law is looking and the legibility law is not.
#[test]
fn the_drawn_ink_fills_the_bounds_the_geometry_predicts() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(CANVAS as f32, CANVAS as f32) else {
        eprintln!("skipping the_drawn_ink_fills_the_bounds: no wgpu adapter");
        return;
    };
    let buf = label_buffer(&mut p.font_system, 15.0);
    let mask = LabelMask::compose(
        &device,
        &queue,
        &mut p.font_system,
        &mut p.swash_cache,
        &buf,
    )
    .expect("the label uploads");
    for deg in GRADED_ANGLES {
        let (alpha, _) = render_at(&device, &queue, &mask, deg, false);
        let b = label_bounds(ORIGIN, label_axis_deg(deg), mask.ink());
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut lit = 0usize;
        for y in 0..CANVAS {
            for x in 0..CANVAS {
                if alpha[(y * CANVAS + x) as usize] > 0.05 {
                    lit += 1;
                    x0 = x0.min(x as f32);
                    y0 = y0.min(y as f32);
                    x1 = x1.max(x as f32 + 1.0);
                    y1 = y1.max(y as f32 + 1.0);
                }
            }
        }
        assert!(lit > 100, "{deg}°: only {lit} lit pixels — nothing drew");
        // CONTAINED: the mask's one-pixel transparent border guarantees no ink
        // reaches the quad's own edge, so a small outward slack is the only
        // tolerance needed.
        assert!(
            x0 >= b[0] - 1.5 && y0 >= b[1] - 1.5,
            "{deg}°: ink starts at ({x0}, {y0}), bounds {b:?}"
        );
        assert!(
            x1 <= b[0] + b[2] + 1.5 && y1 <= b[1] + b[3] + 1.5,
            "{deg}°: ink ends at ({x1}, {y1}), bounds {b:?}"
        );
        // TIGHT: the bounds are not allowed to be a generous box the ink rattles
        // around in. A quarter turn of a wide, short run makes the two extents
        // very different, so this is where a "bounds" that forgot to rotate
        // would be caught.
        assert!(
            (x1 - x0) >= b[2] - 6.0 && (y1 - y0) >= b[3] - 6.0,
            "{deg}°: ink measures {}×{}, bounds claim {}×{}",
            x1 - x0,
            y1 - y0,
            b[2],
            b[3]
        );
    }
}

/// The gradient runs along the RUN'S OWN baseline, not the screen's — what a
/// world whose visual language is a slanted gradient line needs, and the reason
/// it is one instance field rather than a second code path.
///
/// Checked at the quarter turn, where "along the run" and "along the screen"
/// are perpendicular: with red at the start and blue at the end, the ink low on
/// the screen must be red-dominant and the ink high on it blue-dominant. A
/// gradient left in screen space would split the label left-to-right instead
/// and both halves would read the same.
#[test]
fn the_gradient_runs_along_the_labels_own_baseline() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(CANVAS as f32, CANVAS as f32) else {
        eprintln!("skipping the_gradient_runs_along_the_labels_own_baseline: no wgpu adapter");
        return;
    };
    let buf = label_buffer(&mut p.font_system, 15.0);
    let mask = LabelMask::compose(
        &device,
        &queue,
        &mut p.font_system,
        &mut p.swash_cache,
        &buf,
    )
    .expect("the label uploads");
    let (_, rgba) = render_at(&device, &queue, &mask, 90.0, true);
    let b = label_bounds(ORIGIN, label_axis_deg(90.0), mask.ink());
    let mid = b[1] + b[3] * 0.5;
    let (mut low_red, mut low_blue, mut high_red, mut high_blue) = (0u64, 0u64, 0u64, 0u64);
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            let px = rgba[(y * CANVAS + x) as usize];
            if px[3] < 128 {
                continue;
            }
            // Low on the screen is the START of a run that reads bottom to top.
            if (y as f32) > mid {
                low_red += px[0] as u64;
                low_blue += px[2] as u64;
            } else {
                high_red += px[0] as u64;
                high_blue += px[2] as u64;
            }
        }
    }
    // A gradient left in SCREEN space would put both halves at almost the same
    // mix at this angle, so the bar is a real swing rather than a bare
    // inequality: each half must be at least 40% richer in its own end's hue.
    assert!(
        low_red as f32 > low_blue as f32 * 1.4,
        "the run's start must be red-dominant: red {low_red} vs blue {low_blue}"
    );
    assert!(
        high_blue as f32 > high_red as f32 * 1.4,
        "the run's end must be blue-dominant: red {high_red} vs blue {high_blue}"
    );
}

/// A label is a LABEL. The mask reads one layout run and stops, so a string
/// that wraps to two lines composes only its first — the structural reason this
/// capability cannot grow into a second prose renderer.
#[test]
fn only_the_first_layout_run_ever_composes() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(CANVAS as f32, CANVAS as f32) else {
        eprintln!("skipping only_the_first_layout_run_ever_composes: no wgpu adapter");
        return;
    };
    let one = label_buffer(&mut p.font_system, 15.0);
    let (_, ink_one, w_one, _) = compose_run(&mut p.font_system, &mut p.swash_cache, &one)
        .expect("the one-line label composes");

    // The SAME text, wrapped hard onto two lines by a narrow box.
    let mut two = GlyphBuffer::new(&mut p.font_system, GlyphMetrics::new(15.0, 20.0));
    two.set_size(&mut p.font_system, Some(6.0), Some(120.0));
    two.set_wrap(&mut p.font_system, glyphon::Wrap::Glyph);
    two.set_text(
        &mut p.font_system,
        LABEL,
        &Attrs::new().family(Family::SansSerif),
        Shaping::Advanced,
        None,
    );
    two.shape_until_scroll(&mut p.font_system, false);
    // NON-VACUOUS: the fixture must really have wrapped, or the comparison
    // below is between two identical single-line buffers and proves nothing.
    assert!(
        two.layout_runs().count() >= 2,
        "the wrap fixture must produce more than one run"
    );
    let (_, ink_two, w_two, h_two) = compose_run(&mut p.font_system, &mut p.swash_cache, &two)
        .expect("the wrapped label still composes its first run");
    assert!(
        w_two < w_one,
        "a wrapped label must compose only its first run: {w_two} px wide vs {w_one} unwrapped"
    );
    // And that first run is ONE line tall — a second line would have pushed the
    // ink box far below the baseline.
    assert!(
        ink_two[1] + h_two as f32 <= ink_one[1] + 20.0,
        "the composed box reaches a second line: {ink_two:?} vs {ink_one:?}"
    );
}
