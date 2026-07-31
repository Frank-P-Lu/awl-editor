//! ITEM 163 — Bowerbird's organic ground drift, real-pixel end-to-end proofs.
//!
//! **The defect, and why the pre-163 laws missed it:**
//! `organic_phase_sweep_stays_cool_and_the_density_mutation_goes_red`
//! (`backgrounds_item117.rs`) and `bowerbird_organic_schedules_zero_frames_
//! under_every_freeze_condition` (`theme::tests`) both PASSED on the shipped
//! shader while a live user reported "I literally don't see any drift" —
//! because neither law measured HOW FAR anything moved, only THAT some pixel
//! changed or THAT the scheduler armed. This file adds the missing measure.
//!
//! **Diagnosis (recorded here, not just in the commit message, since a wrong
//! premise is the most valuable finding a diagnosis step can return):** the
//! live ambient scheduler was never the bug. `TextPipeline::
//! prepare_background_layer` (`render/layers.rs`) already resolves a NONZERO
//! `drift` for `effective_background().is_organic()` from the SAME shared
//! `waves_render_phase()` clock the lava lamp and Bombora's waves ride, and
//! `Theme::has_ambient_tick` already includes `background.is_organic()` so
//! Bowerbird's world genuinely arms the App's ambient `WaitUntil` tick under
//! default settings (`bowerbird_organic_schedules_zero_frames_under_every_
//! freeze_condition` proves the arm/freeze truth table; `
//! bowerbird_organic_drift_is_wired_end_to_end` below proves it with real
//! rendered pixels, not just the scheduling predicate). The actual bug was
//! authored AMPLITUDE: `shaders/background.wgsl`'s `organic_rgb` translated
//! the whole field by only `sin(g.drift)*5.0, cos(g.drift*0.73)*4.0` — under
//! 4% of Bowerbird's own 156px `scale_px` cell — so the scheduler was doing
//! real work every ~100ms and the result was still invisible on a real
//! glance-back. Item 163's fix scales the drift to a FRACTION of the cell
//! size (`ORGANIC_DRIFT_X_FRAC`/`_Y`, ~13%/10%) instead of a fixed pixel
//! count, without touching the scheduler, the shared clock, the gates, or
//! the blob-shape math at all.

use super::super::*;
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};

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
        label: Some("awl bowerbird-drift encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

/// LAW (liveness, real pixels, the live-scheduler half of the diagnosis):
/// advancing the SAME shared ambient clock the lava lamp/stars/waves ride
/// (`TextPipeline::advance_lava`) through the real `is_organic()` branch in
/// `prepare_background_layer` visibly changes a real Bowerbird page, while
/// two captures at the SAME settled phase stay byte-identical and every
/// other GPU-instanced layer's count is untouched — proving the scheduler
/// genuinely reaches Bowerbird's ground (ruling out "the clock never
/// advances Bowerbird" as the explanation) independent of how big the
/// resulting displacement is (that's the separate law below).
#[test]
fn bowerbird_organic_drift_is_wired_end_to_end() {
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping bowerbird_organic_drift_is_wired_end_to_end: no wgpu adapter");
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

    // Settled determinism: two captures with no tick in between are byte-
    // identical (a headless capture never ticks, so a theme crossing INTO
    // Bowerbird begins at this same stable frame every time).
    let settled_a = render_frame(&device, &queue, &mut p, W, H);
    let sel_settled = p.selection_pipeline.instance_count();
    let settled_b = render_frame(&device, &queue, &mut p, W, H);
    assert_eq!(
        settled_a, settled_b,
        "two captures of the SAME settled Bowerbird scene must be byte-identical"
    );

    // Advance the real ambient clock through the App's own bounded per-tick
    // step (never the caret's hot per-frame Poll loop).
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
        "the ground's drift must not grow or shrink any OTHER layer's instance count"
    );

    let mut changed = 0usize;
    for y in 0..H as usize {
        for x in 0..W as usize {
            if settled_a[y * W as usize + x] != moved[y * W as usize + x] {
                changed += 1;
                let xf = x as f32;
                assert!(
                    !(xf >= col_left && xf < col_right),
                    "a drift-diff pixel at ({x}, {y}) sits INSIDE the writing column \
                     [{col_left}, {col_right}) — the organic ground must never bleed \
                     into the static page"
                );
            }
        }
    }
    assert!(
        changed > 200,
        "the shared ambient clock must genuinely move Bowerbird's ground after 167 real \
         ticks (only {changed} pixels changed) — the scheduler is not reaching Bowerbird"
    );
}

/// A `[x0, x1)` band's mean BLUE channel per column, averaged over every row
/// in `[0, h)` — Bowerbird's three tones (`0x26`, `0x33`, `0x49`) separate
/// most in blue, and averaging over the full column height marginalizes out
/// the field's (smaller) vertical drift component so a purely horizontal
/// correlation search stays meaningful despite the field also moving in y.
fn blue_profile(pixels: &[[u8; 4]], w: u32, h: u32, x0: u32, x1: u32) -> Vec<f32> {
    (x0..x1)
        .map(|x| {
            let sum: u32 = (0..h).map(|y| pixels[(y * w + x) as usize][2] as u32).sum();
            sum as f32 / h as f32
        })
        .collect()
}

/// The horizontal shift (in the profile's own px units) that best aligns `b`
/// onto `a` by minimizing mean squared difference over the overlapping span,
/// searched over `[-max_shift, max_shift]`. A brute-force 1-D correlation —
/// the deterministic "phase-delta pixel arithmetic" a perceptibility floor
/// needs, without pulling in a signal-processing dependency for one law.
fn best_shift(a: &[f32], b: &[f32], max_shift: i32) -> i32 {
    let n = a.len() as i32;
    let mut best = 0i32;
    let mut best_mse = f32::INFINITY;
    for shift in -max_shift..=max_shift {
        let mut sse = 0f32;
        let mut count = 0i32;
        for i in 0..n {
            let j = i + shift;
            if j < 0 || j >= n {
                continue;
            }
            let d = a[i as usize] - b[j as usize];
            sse += d * d;
            count += 1;
        }
        if count < n / 2 {
            continue; // require a real majority overlap, not an edge sliver
        }
        let mse = sse / count as f32;
        if mse < best_mse {
            best_mse = mse;
            best = shift;
        }
    }
    best
}

/// THE MISSING LAW (item 163): a perceptibility FLOOR in pixels, not just a
/// "some pixel changed" witness. Two explicitly chosen points on the shared
/// ambient clock — the settled phase (`drift = 0`) and 167 ticks later
/// (`g.drift` lands within a whisper of π/2, putting `sin(g.drift)` near its
/// own maximum — the single best-separated point on the curve to measure the
/// AUTHORED peak, chosen deliberately rather than an arbitrary tick count) —
/// are rendered through the real `BackgroundPipeline`, and the LEFT margin's
/// horizontal blue-channel profile at each phase is cross-correlated to
/// recover how far the field actually walked, in px.
///
/// `PERCEPTIBILITY_FLOOR_PX = 10.0`: the pre-163 shader's authored ceiling
/// was 5.0px (`sin(g.drift) * 5.0`) — this floor sits strictly above that
/// entire old range, so the law is a real regression gate, not a restatement
/// of the new constant. It sits comfortably below the new formula's own
/// ~20px peak (`156.0 * ORGANIC_DRIFT_X_FRAC` with `ORGANIC_DRIFT_X_FRAC =
/// 0.13`), leaving headroom for the correlation search's own measurement
/// slop instead of pinning the exact authored value.
const PERCEPTIBILITY_FLOOR_PX: f32 = 10.0;

#[test]
fn bowerbird_organic_drift_clears_a_perceptibility_floor_over_the_ambient_cycle() {
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping bowerbird_organic_drift_clears_a_perceptibility_floor: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(24); // narrow column -> wide margins to measure in

    theme::set_active_by_name("Bowerbird").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    let frame_a = render_frame(&device, &queue, &mut p, W, H);
    for _ in 0..167 {
        p.advance_lava(crate::lava::LAVA_TICK_SECONDS);
    }
    let frame_b = render_frame(&device, &queue, &mut p, W, H);

    let col_left = p.column_left();
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    let left_band = col_left.floor().max(1.0) as u32;
    let profile_a = blue_profile(&frame_a, W, H, 0, left_band);
    let profile_b = blue_profile(&frame_b, W, H, 0, left_band);
    let shift = best_shift(&profile_a, &profile_b, 40).unsigned_abs() as f32;

    assert!(
        shift >= PERCEPTIBILITY_FLOOR_PX,
        "Bowerbird's organic ground moved only {shift}px between two well-separated ambient \
         phases (settled vs. 167 ticks later) — under the {PERCEPTIBILITY_FLOOR_PX}px \
         perceptibility floor a writer glancing back after roughly a minute cannot register \
         the drift as real motion"
    );
}

/// LAW (the axis a single-scale check would miss): the perceptibility floor
/// above only proves the fix at Bowerbird's shipped 156px `scale_px`.
/// `organic_rgb`'s own defensive clamp (`s = max(g.params.x, 32.0)`) admits
/// scales far smaller than that, and a PURE fraction-of-`s` formula goes
/// back UNDER the floor there (`32.0 * ORGANIC_DRIFT_X_FRAC` ≈ 4.2px — a
/// flashback to the pre-163 defect) — exactly why `organic_rgb` pairs the
/// fraction with an `ORGANIC_DRIFT_MIN_X_PX`/`_Y` floor underneath it. This
/// sweeps `s` from the shader's own defended minimum, through the shipped
/// default, to a much larger hypothetical cell, using a literal
/// `Background::Organic` value at each scale (the direct-drift-injection
/// seam, not tied to any one world) so the claim is about the MECHANISM, not
/// one hardcoded constant — the exact trap CLAUDE.md names for a green law
/// that only checked the case its author already had in mind.
#[test]
fn bowerbird_organic_drift_clears_the_floor_at_every_reachable_cell_scale() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bowerbird_organic_drift_clears_the_floor_at_every_reachable_cell_scale: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let (tones, arrangement, density) = match theme::BOWERBIRD.background {
        theme::Background::Organic {
            tones,
            arrangement,
            density,
            ..
        } => (tones, arrangement, density),
        _ => panic!("Bowerbird must ship Background::Organic"),
    };
    const W: u32 = 900;
    const H: u32 = 600;
    const LEFT: f32 = 220.0;
    const COL: f32 = 460.0;
    // The shader's own defended floor, the shipped default, and a much
    // larger hypothetical cell — the mechanism must hold at every one.
    for scale_px in [32.0f32, 60.0, 156.0, 400.0] {
        let bg = theme::Background::Organic {
            tones,
            arrangement,
            scale_px,
            density,
        };
        let frame_a = render_bg(&device, &queue, bg_desc_for(bg), W, H, LEFT, COL, 0.0);
        let frame_b = render_bg(
            &device,
            &queue,
            bg_desc_for(bg),
            W,
            H,
            LEFT,
            COL,
            std::f32::consts::FRAC_PI_2,
        );
        let left_band = LEFT.floor().max(1.0) as u32;
        let profile_a = blue_profile(&frame_a, W, H, 0, left_band);
        let profile_b = blue_profile(&frame_b, W, H, 0, left_band);
        // A search window past half the cell period would let an alias of
        // the tiled field masquerade as a smaller true shift; keep it under
        // that period's own half-width.
        let max_shift = ((scale_px / 2.0 - 2.0).max(8.0)) as i32;
        let shift = best_shift(&profile_a, &profile_b, max_shift).unsigned_abs() as f32;
        assert!(
            shift >= PERCEPTIBILITY_FLOOR_PX,
            "scale_px {scale_px}: moved only {shift}px between drift=0 and drift=pi/2 — under \
             the {PERCEPTIBILITY_FLOOR_PX}px perceptibility floor"
        );
    }
}

/// LAW (worst-phase value/hue/page bounds, re-verified at the NEW, larger
/// amplitude): the ground must stay in Bowerbird's cool navy value band and
/// out of the page column at EVERY phase, not just the settled one — reusing
/// `backgrounds_item69`'s direct-drift-injection seam
/// (`render_bg`/`bg_desc_for`/`headless_dq`) the way `backgrounds_item117.rs`
/// already does, swept across a full turn of `g.drift` so a bigger authored
/// amplitude can't quietly push a worst-phase pixel warm/bright or into the
/// column. Non-vacuous against amplitude specifically: this sweep already
/// existed pre-163 at the OLD tiny amplitude — its counterpart in
/// `backgrounds_item117.rs` stays green post-163, and this copy re-asserts
/// the same bound at page-column-adjacent geometry the perceptibility law
/// above uses (narrow measure, wide margins).
#[test]
fn bowerbird_organic_worst_phase_stays_cool_and_off_the_page() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bowerbird_organic_worst_phase_stays_cool_and_off_the_page: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let bg = theme::BOWERBIRD.background;
    let (w, h, left, col) = (900, 600, 220.0, 460.0);
    for i in 0..24 {
        let drift = (i as f32 / 24.0) * std::f32::consts::TAU;
        let pixels = render_bg(&device, &queue, bg_desc_for(bg), w, h, left, col, drift);
        for (idx, p) in pixels.iter().enumerate() {
            let x = (idx as u32) % w;
            if (x as f32) >= left && (x as f32) < left + col {
                // The column pixel is never touched: `fs_main` returns
                // transparent there, so the pass's own opaque BLACK clear
                // shows through unchanged (rgb stays 0; alpha is the clear's,
                // not the shader's, so it is not part of this assertion).
                assert_eq!(
                    [p[0], p[1], p[2]],
                    [0, 0, 0],
                    "drift {drift}: organic ink entered the page column at x={x}"
                );
                continue;
            }
            assert!(
                p[2] >= p[0] && p[0] < 90,
                "drift {drift}: warm/bright margin pixel {p:?} — the ground must stay cool"
            );
        }
    }
}

/// LAW (gates, real pixels): Reduce Motion freezes the render to the settled
/// composition EVEN IF the shared phase field itself has already advanced —
/// `TextPipeline::waves_render_phase` reads `crate::motion::reduced()` at
/// render time, not at tick time, so this holds independent of exactly when
/// in a session the user (or the Settings toggle) flips the preference.
/// Real-pixel proof, not just the pure `lava_phase_for` resolver check
/// (`organic_freeze_conditions_resolve_to_the_settled_phase` in
/// `backgrounds_item117.rs`) — the sidecar/pixel discipline this repo holds
/// every ambient-gate claim to.
#[test]
fn bowerbird_organic_reduce_motion_freezes_pixels_despite_an_advanced_clock() {
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

    // Simulate a clock that kept advancing (e.g. Reduce Motion flipped ON
    // mid-session, after ticks had already accumulated) — the render path,
    // not the tick path, must be the thing that freezes.
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
        "Reduce Motion must render the SAME settled composition as an untouched clock, \
         even after 500 ticks of accumulated phase — the gate lives at render time"
    );
}
