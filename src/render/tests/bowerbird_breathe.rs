//! ITEM 244 — Bowerbird's organic ground stops translating and gains a
//! per-element COMPANION value breathe. Real-pixel end-to-end proofs;
//! supersedes this file's former self (a prior round's field-translation
//! amplitude fix), which this round retired outright — the field no longer
//! translates at all, so that prior round's own perceptibility-floor claim
//! ("the field must move at least N px") is now the OPPOSITE of what this
//! ground must do.
//!
//! **The measured defect this item fixed (recorded here, not just in the
//! commit message):** `shaders/background.wgsl`'s pre-244 `organic_rgb` took
//! `sin(g.drift)` for X and `cos(g.drift * 0.73)` for Y. Across the shared
//! clock's wrap, X was continuous (`sin(TAU) == sin(0) == 0`) but Y jumped
//! `cos(0.73*TAU) = -0.125333` to `cos(0.0) = 1.000000` — a 1.125333
//! normalised-unit discontinuity, ~21.9px vertically at Bowerbird's shipped
//! `scale_px: 195.0` — in ONE FRAME, every ~67s. `src/background/waves.rs`
//! already states the house law this dodged: `WAVE_DRIFT_CYCLES` must be an
//! INTEGER so a drift "meets its own endpoint exactly where the clock
//! wraps." `0.73` is not, and Organic was the one ambient consumer breaking
//! it. The wrap-continuity LAW itself (unconditional, owed regardless of
//! this round's redesign) lives in `ambient_wrap_law.rs`.
//!
//! **The redesign (user design decision, 2026-08-04):** every other ambient
//! ground earns its motion from its subject — Bombora is a sea, Currawong a
//! star field, Kite a travelling grid. Bowerbird's ground is `Finds`, a
//! "COLLECTED-TREASURE" arrangement — one anchor, one companion, one
//! cut-out, DELIBERATELY PLACED and then left alone. A bower is an
//! arrangement, not a current. So `organic_rgb` no longer computes a `drift`
//! vec2 at all (both terms deleted, not retuned), and the ground keeps a
//! life of its own through a per-element VALUE breathe on the COMPANION role
//! (`kind_b`) alone instead — see `organic_finds_rgb`'s own doc comment for
//! the mechanism (a `mix` between two of the world's existing three tones,
//! envelope shaped like `stars.rs:185`'s twinkle, integer rate, seeded
//! per-cell phase offset).

use super::super::*;
use super::bands_waves::{bg_desc_for, headless_dq, render_bg_ambient};
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};
use crate::background::AmbientUpload;

fn render_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    p.prepare(device, queue, w, h).unwrap();
    let (texture, tview) = offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl bowerbird-breathe encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

fn organic_at(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    phase: f32,
    w: u32,
    h: u32,
    left: f32,
    col: f32,
) -> Vec<[u8; 4]> {
    render_bg_ambient(
        device,
        queue,
        bg_desc_for(theme::BOWERBIRD.background),
        w,
        h,
        left,
        col,
        AmbientUpload {
            organic_phase: phase,
            ..Default::default()
        },
        1.0,
    )
}

/// LAW (the redesign's headline claim, real pixels): the field's own
/// SILHOUETTE — which pixels carry ink at all — must be IDENTICAL at every
/// ambient phase; only the ink's VALUE inside that fixed silhouette may
/// change. Proven via the differential oracle every ground family in this
/// file already uses (`density: 0.0` collapses to the flat ground exactly,
/// independent of phase — the multiplicative breathe formula's own
/// invariant, see `organic_finds_rgb`'s doc): render the flat reference
/// once, render two well-separated mid-cycle phases against it, and require
/// the two "differs from flat" MASKS to match pixel-for-pixel even though
/// the pixel VALUES inside that mask are free to differ.
#[test]
fn bowerbird_organic_field_never_translates_across_the_ambient_clock() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bowerbird_organic_field_never_translates_across_the_ambient_clock: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, left, col) = (900u32, 600u32, 220.0, 460.0);

    let mut flat_bg = bg_desc_for(theme::BOWERBIRD.background);
    flat_bg.density = 0.0;
    let flat = render_bg_ambient(
        &device,
        &queue,
        flat_bg,
        w,
        h,
        left,
        col,
        AmbientUpload::default(),
        1.0,
    );

    let frame_a = organic_at(&device, &queue, 0.31, w, h, left, col);
    let frame_b = organic_at(&device, &queue, 1.74, w, h, left, col);

    let mask = |frame: &[[u8; 4]]| -> Vec<bool> {
        frame
            .iter()
            .zip(flat.iter())
            .map(|(p, f)| {
                let d: i32 = (0..3).map(|k| (p[k] as i32 - f[k] as i32).abs()).sum();
                d >= 3
            })
            .collect()
    };
    let mask_a = mask(&frame_a);
    let mask_b = mask(&frame_b);

    // Not a bit-exact mask comparison: a companion pixel right at the
    // shader's own antialiased edge (`finds_fill`'s sub-pixel feather) blends
    // ground and ink in a fraction that itself depends on `d_b`, so the
    // breathe can nudge a HANDFUL of edge pixels across the >=3 threshold
    // either way without the SHAPE actually moving — the same AA tolerance
    // this codebase already grants elsewhere (e.g. `dither.rs`'s "a small,
    // scattered minority attributable to glyph anti-aliasing"). A genuine
    // translation would instead flip a shape's entire leading/trailing edge —
    // hundreds to thousands of pixels — not a thin fringe.
    let mask_diff = mask_a
        .iter()
        .zip(mask_b.iter())
        .filter(|(a, b)| a != b)
        .count();
    let ink_pixels = mask_a.iter().filter(|&&b| b).count().max(1);
    assert!(
        ink_pixels > (w * h / 100) as usize,
        "sanity: too few ink-bearing pixels found ({ink_pixels}) to trust the mask comparison"
    );
    assert!(
        mask_diff * 20 < ink_pixels,
        "the set of ink-bearing pixels moved between two ambient phases ({mask_diff} of \
         {ink_pixels} ink pixels flipped, over the AA-edge tolerance) — the FIELD must hold \
         perfectly still (a bower is an arrangement, deliberately placed and left alone); only \
         the companion's own VALUE may change"
    );

    // Non-vacuity: the two frames must actually differ SOMEWHERE inside that
    // fixed silhouette — otherwise "never translates" would be trivially true
    // of a ground that also never breathes.
    let mut differed_inside_mask = 0usize;
    for i in 0..frame_a.len() {
        if mask_a[i] && frame_a[i] != frame_b[i] {
            differed_inside_mask += 1;
        }
    }
    assert!(
        differed_inside_mask > 50,
        "the companion breathe must visibly change SOME ink pixel's value between two \
         well-separated phases (only {differed_inside_mask} did) — a silhouette that never \
         moves AND never changes value would just be item 244's own dead deletion, not the \
         breathe it was replaced with"
    );
}

/// LAW: the companion breathe is genuinely VISIBLE across a phase sweep, and
/// neighbouring companions are NOT in phase with each other — the seeded
/// per-cell offset (`organic_finds_rgb`'s `h6` roll) actually desynchronises
/// them, real pixels, not merely a claim about the hash. Method: render a
/// phase sweep, keep only pixels whose value genuinely moves across it (by
/// construction, exactly the companion-covered pixels — the anchor, cut-out,
/// and open ground never read `g.organic_phase` at all), sample WIDELY
/// spaced survivors (spacing exceeds a single companion's own footprint, so
/// distinct samples land on distinct companions rather than the same one
/// twice), and require their trajectories NOT all be near-perfectly
/// correlated — the exact signature two companions sharing one rate+offset
/// would leave (an in-phase pair's trajectories are identical up to the
/// per-cell tone/density scale, so their correlation is ~1.0 exactly).
#[test]
fn bowerbird_companion_breathe_is_visible_and_neighbours_desynchronise() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bowerbird_companion_breathe_is_visible_and_neighbours_desynchronise: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, left, col) = (900u32, 600u32, 0.0, 0.0); // no page hole: every pixel is margin

    const N_PHASES: usize = 20;
    let wrap = crate::lava::LAVA_LOOP_CYCLES;
    let frames: Vec<Vec<[u8; 4]>> = (0..N_PHASES)
        .map(|i| {
            let phase = wrap * (i as f32) / (N_PHASES as f32);
            organic_at(&device, &queue, phase, w, h, left, col)
        })
        .collect();

    // Scan EVERY pixel (not a sparse grid — a companion's own footprint is a
    // small fraction of the canvas, and a fixed-stride grid can easily land
    // entirely on ground or on the same handful of collections by chance) for
    // genuine phase-variance: by construction only companion-covered pixels
    // ever move at all (the anchor, cut-out, and open ground never read
    // `g.organic_phase`), so this recovers every companion the sweep touches.
    let mut varying: Vec<u32> = Vec::new();
    for idx in 0..(w * h) as usize {
        let mut sum = 0f32;
        let mut sumsq = 0f32;
        for f in &frames {
            let v = f[idx][2] as f32;
            sum += v;
            sumsq += v * v;
        }
        let n = frames.len() as f32;
        let mean = sum / n;
        let var = sumsq / n - mean * mean;
        if var > 0.3 {
            varying.push(idx as u32);
        }
    }
    assert!(
        varying.len() > 20,
        "too few phase-varying pixels found ({}) — the breathe is not visibly reaching real \
         pixels",
        varying.len()
    );

    // Bucket survivors spatially (bucket > a single companion's own
    // footprint, comfortably under Bowerbird's 195px cell) and keep one
    // representative trajectory per bucket, so distinct samples land on
    // DISTINCT companions rather than the same one many times over.
    const BUCKET: u32 = 80;
    let mut reps: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
    for idx in varying {
        let (x, y) = (idx % w, idx / w);
        reps.entry((x / BUCKET, y / BUCKET)).or_insert(idx as usize);
    }
    let trajectories: Vec<Vec<f32>> = reps
        .values()
        .map(|&idx| frames.iter().map(|f| f[idx][2] as f32).collect())
        .collect();
    assert!(
        trajectories.len() >= 4,
        "too few distinct breathing companions sampled ({}) — need several to prove \
         desynchronisation",
        trajectories.len()
    );

    fn pearson(a: &[f32], b: &[f32]) -> f32 {
        let n = a.len() as f32;
        let ma = a.iter().sum::<f32>() / n;
        let mb = b.iter().sum::<f32>() / n;
        let mut cov = 0f32;
        let mut va = 0f32;
        let mut vb = 0f32;
        for i in 0..a.len() {
            let da = a[i] - ma;
            let db = b[i] - mb;
            cov += da * db;
            va += da * da;
            vb += db * db;
        }
        cov / (va.sqrt() * vb.sqrt()).max(1e-6)
    }

    let mut min_corr = f32::INFINITY;
    let mut any_pair = false;
    for i in 0..trajectories.len() {
        for j in (i + 1)..trajectories.len() {
            any_pair = true;
            let c = pearson(&trajectories[i], &trajectories[j]);
            min_corr = min_corr.min(c);
        }
    }
    assert!(any_pair, "need at least two varying samples to compare");
    assert!(
        min_corr < 0.9,
        "every sampled companion's breathe trajectory correlates >= 0.9 with every other \
         (tightest pair: {min_corr:.4}) — neighbours are breathing IN LOCKSTEP, not \
         desynchronised by their own seeded phase offset"
    );
}

/// LAW (worst-phase value/hue bounds, re-verified post-244): the ground must
/// stay in Bowerbird's cool navy value band at EVERY companion-breathe phase,
/// not just the settled one — the multiplicative amplitude is small by
/// design, but this is the real regression gate against a future amplitude
/// bump quietly pushing a companion's mixed value warm or bright.
#[test]
fn bowerbird_organic_worst_breathe_phase_stays_cool_and_off_the_page() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bowerbird_organic_worst_breathe_phase_stays_cool_and_off_the_page: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, left, col) = (900u32, 600u32, 220.0, 460.0);
    let wrap = crate::lava::LAVA_LOOP_CYCLES;
    for i in 0..24 {
        let phase = wrap * (i as f32) / 24.0;
        let pixels = organic_at(&device, &queue, phase, w, h, left, col);
        for (idx, p) in pixels.iter().enumerate() {
            let x = (idx as u32) % w;
            if (x as f32) >= left && (x as f32) < left + col {
                assert_eq!(
                    [p[0], p[1], p[2]],
                    [0, 0, 0],
                    "phase {phase}: organic ink entered the page column at x={x}"
                );
                continue;
            }
            assert!(
                p[2] >= p[0] && p[0] < 90,
                "phase {phase}: warm/bright margin pixel {p:?} — the ground must stay cool"
            );
        }
    }
}

/// LAW (liveness, real pixels): advancing the SAME shared ambient clock the
/// lava lamp/stars/waves ride (`TextPipeline::advance_lava`) through the real
/// `is_organic()` branch in `prepare_background_layer` visibly changes a real
/// Bowerbird page's companion breathe, while two captures at the SAME
/// settled phase stay byte-identical and every other GPU-instanced layer's
/// count is untouched — proving the scheduler still genuinely reaches
/// Bowerbird post-244 (the field-translation code path this used to prove is
/// gone; the companion breathe is what the clock now drives).
#[test]
fn bowerbird_organic_breathe_is_wired_end_to_end() {
    let _g = crate::testlock::serial();
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping bowerbird_organic_breathe_is_wired_end_to_end: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(24);

    theme::set_active_by_name("Bowerbird").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    let settled_a = render_frame(&device, &queue, &mut p, W, H);
    let sel_settled = p.selection_pipeline.instance_count();
    let settled_b = render_frame(&device, &queue, &mut p, W, H);
    assert_eq!(
        settled_a, settled_b,
        "two captures of the SAME settled Bowerbird scene must be byte-identical"
    );

    for _ in 0..167 {
        p.advance_lava(crate::lava::LAVA_TICK_SECONDS);
    }
    let moved = render_frame(&device, &queue, &mut p, W, H);
    let sel_moved = p.selection_pipeline.instance_count();

    let col_left = p.column_left();
    let col_right = col_left + p.column_width();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    assert_eq!(
        sel_settled, sel_moved,
        "the ground's breathe must not grow or shrink any OTHER layer's instance count"
    );

    let mut changed = 0usize;
    for y in 0..H as usize {
        for x in 0..W as usize {
            if settled_a[y * W as usize + x] != moved[y * W as usize + x] {
                changed += 1;
                let xf = x as f32;
                assert!(
                    !(xf >= col_left && xf < col_right),
                    "a breathe-diff pixel at ({x}, {y}) sits INSIDE the writing column \
                     [{col_left}, {col_right}) — the organic ground must never bleed into the \
                     static page"
                );
            }
        }
    }
    assert!(
        changed > 20,
        "the shared ambient clock must genuinely change Bowerbird's ground after 167 real \
         ticks (only {changed} pixels changed) — the scheduler is not reaching Bowerbird"
    );
}

/// LAW (gates, real pixels): Reduce Motion freezes the render to the settled
/// composition EVEN IF the shared phase field itself has already advanced —
/// `TextPipeline::waves_render_phase` reads `crate::motion::reduced()` at
/// render time, not at tick time, so this holds independent of exactly when
/// in a session the user (or the Settings toggle) flips the preference.
#[test]
fn bowerbird_organic_reduce_motion_freezes_pixels_despite_an_advanced_clock() {
    let _g = crate::testlock::serial();
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping bowerbird_organic_reduce_motion_freezes_pixels_despite_an_advanced_clock: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let was_reduced = crate::motion::reduced();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(24);

    theme::set_active_by_name("Bowerbird").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    crate::motion::set_reduced(false);
    let settled = render_frame(&device, &queue, &mut p, W, H);

    for _ in 0..500 {
        p.advance_lava(crate::lava::LAVA_TICK_SECONDS);
    }
    crate::motion::set_reduced(true);
    let frozen = render_frame(&device, &queue, &mut p, W, H);

    crate::motion::set_reduced(was_reduced);
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    assert_eq!(
        settled, frozen,
        "Reduce Motion must render the SAME settled composition as an untouched clock, even \
         after 500 ticks of accumulated phase — the gate lives at render time"
    );
}
