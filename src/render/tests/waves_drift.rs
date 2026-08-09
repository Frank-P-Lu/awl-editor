//! BOMBORA'S WAVE PHASE DRIFT, real-pixel end-to-end proofs (the
//! render-side half of `theme::tests::bombora_wave_drift_schedules_zero_
//! frames_under_every_freeze_condition` / `background::waves_drift_tests`).
//! Mirrors `stars.rs`'s twinkle-diff idiom exactly: render the SAME Bombora
//! scene at two points on the shared ambient clock and diff the frames —
//! every changed pixel IS a drift pixel (nothing else in a static page reads
//! the ambient clock), so "unchanged at the frozen/settled phase, genuinely
//! different after the clock advances, confined to the margins, and every
//! OTHER GPU-instanced layer stays byte-identical in count" proves
//! determinism + liveness + boundedness in one sweep.

use super::super::*;
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
        label: Some("awl waves-drift encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

/// LAW (theme-crossing / headless-capture determinism): two INDEPENDENT
/// end-to-end captures of Bombora at the SAME frozen phase (the pipeline's
/// construction default — a headless capture never ticks the ambient clock)
/// are byte-identical, and every other GPU-instanced layer's count is
/// unaffected. Proves "a theme crossing into Bombora begins at a stable,
/// deterministic phase" and "deterministic headless capture freezes it".
#[test]
fn bombora_settled_captures_are_byte_identical() {
    let _g = crate::testlock::serial();
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping bombora_settled_captures_are_byte_identical: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(24); // narrow column -> wide, wave-bearing margins

    theme::set_active_by_name("Bombora").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    let frame_a = render_frame(&device, &queue, &mut p, W, H);
    let sel_a = p.selection_pipeline.instance_count();
    let frame_b = render_frame(&device, &queue, &mut p, W, H);
    let sel_b = p.selection_pipeline.instance_count();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    assert_eq!(
        sel_a, sel_b,
        "no other GPU-instanced layer's count may drift"
    );
    assert_eq!(
        frame_a, frame_b,
        "two captures of the SAME settled/frozen Bombora scene must be byte-identical — \
         the wave drift is a pure function of the shared ambient phase, which a headless \
         capture never advances"
    );
}

/// LAW (liveness + boundedness): advancing the SAME shared ambient clock the
/// lava lamp and twinkling stars ride (`TextPipeline::advance_lava`, the App's
/// own bounded per-tick step) visibly changes Bombora's margin waves — a
/// non-vacuous witness that the drift is wired into the real
/// render path, not just the isolated `BackgroundPipeline` unit tests — while
/// EVERY other GPU-instanced layer (selection, the one glyph/quad the tiny
/// fixture draws) stays IDENTICAL in count: the drift costs nothing beyond
/// one uniform float upload on the SAME single fullscreen-triangle draw call
/// (`BackgroundPipeline::draw`'s literal `pass.draw(0..3, 0..1)`, unchanged by
/// this round), never a new instance per glyph or per doc line.
#[test]
fn bombora_waves_visibly_drift_after_the_ambient_clock_advances_and_stay_bounded() {
    let _g = crate::testlock::serial();
    const W: u32 = 900;
    const H: u32 = 600;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping bombora_waves_visibly_drift_after_the_ambient_clock_advances_and_stay_bounded: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    crate::page::set_page_on(true);
    crate::page::set_measure(24);

    theme::set_active_by_name("Bombora").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    let frame_a = render_frame(&device, &queue, &mut p, W, H);
    let sel_a = p.selection_pipeline.instance_count();

    // Advance the shared ambient clock through the App's own bounded tick
    // step (each call clamps to one 100 ms step) to a genuinely different
    // mid-drift composition — a SMALL number of ticks (the drift is meant to
    // read as "very slow, almost imperceptible" over the whole ~67s loop, not
    // a fast scroll).
    for _ in 0..400 {
        p.advance_lava(crate::lava::LAVA_TICK_SECONDS);
    }
    let frame_b = render_frame(&device, &queue, &mut p, W, H);
    let sel_b = p.selection_pipeline.instance_count();

    let col_left = p.column_left();
    let col_right = col_left + p.column_width();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);

    assert_eq!(
        sel_a, sel_b,
        "the drift must not grow or shrink any OTHER layer's instance count — \
         it lives entirely in the background pipeline's one uniform float"
    );

    let mut changed = 0usize;
    for y in 0..H as usize {
        for x in 0..W as usize {
            let a = frame_a[y * W as usize + x];
            let b = frame_b[y * W as usize + x];
            if a == b {
                continue;
            }
            changed += 1;
            let xf = x as f32;
            assert!(
                !(xf >= col_left && xf < col_right),
                "a wave-drift-diff pixel at ({x}, {y}) sits INSIDE the writing column \
                 [{col_left}, {col_right}) — the wave ground must never bleed into the \
                 page column"
            );
        }
    }
    assert!(
        changed > 50,
        "the sea must genuinely drift between two well-separated ambient-clock phases \
         (only {changed} pixels changed) — item 87's drift is not vacuous"
    );
}
