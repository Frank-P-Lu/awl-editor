//! THE PICKER SWEEP — what one theme-picker arrow costs, per world, at one
//! scroll depth.
//!
//! Held apart from the frame profiler next door because it measures a different
//! subject: not "how long does a frame take" but "how long does the user wait
//! between the arrow and the frame", and how much of the document had to be
//! shaped first. That second quantity is what makes the sweep depth-sensitive —
//! see [`picker_sweep`].

use super::*;

/// The device, queue, shader cache and render target the burst profiler draws
/// through — one borrow instead of four at every call.
#[derive(Clone, Copy)]
struct BurstGpu<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    cache: &'a Cache,
    target_view: &'a wgpu::TextureView,
}

/// What one picker arrow cost, and how much of the document it had shaped at the
/// two moments that matter: when its frame was presented, and when its step ended.
struct PickerStep {
    face: &'static str,
    shape_ms: f64,
    frame_ms: f64,
    tail_ms: f64,
    at_present: usize,
    settled: usize,
    /// Did the step DECLARE a debt — it narrowed the shaping budget, so a
    /// `finish_shape_tail` ran?
    owed: bool,
    /// Did the step actually DEFER any rows? The split buys nothing where it did
    /// not, and how often it does is a function of SCROLL DEPTH. Kept as its own
    /// question rather than inferred from `owed`: see [`picker_step`].
    narrowed: bool,
}

impl PickerStep {
    fn to_present(&self) -> f64 {
        self.shape_ms + self.frame_ms
    }
    fn whole_step(&self) -> f64 {
        self.to_present() + self.tail_ms
    }
}

/// ONE arrow of the sweep: the reshape at `reach`, the frame it produces, and the
/// off-screen tail that the same step pays afterwards. `None` when this hop has no
/// reshape to do at all (only reachable when the sweep re-enters the world it
/// started in), which is not an arrow worth timing.
///
/// WITNESSED IN ROWS, not just millis — CLAUDE.md's own tripwire: this bench once
/// "measured" 5 ms while nothing reshaped. The reshape counter is checked against
/// `needs_theme_reshape`, the arm must have shaped the number of rows its reach
/// implies, and the step must END owing nothing whichever arm ran.
fn picker_step(
    gpu: &BurstGpu<'_>,
    p: &mut TextPipeline,
    view: &ViewState,
    reach: ShapeReach,
    world: crate::theme::Theme,
) -> anyhow::Result<Option<PickerStep>> {
    crate::theme::set_active_by_name(world.name);
    let face = crate::theme::active().font;
    let must_reshape = p.needs_theme_reshape();
    let before = p.reshape_count;
    let t0 = Instant::now();
    p.sync_theme_colors();
    p.sync_theme_font(reach);
    let shape_ms = t0.elapsed().as_secs_f64() * 1e3;
    assert_reshape_witness(
        &format!("picker step to {} ({face})", world.name),
        before,
        p.reshape_count,
        must_reshape,
    )?;
    if !must_reshape {
        return Ok(None);
    }
    p.set_view(view);
    let frame = burst_frame(p, gpu.device, gpu.queue, gpu.target_view, true)?;
    // The frame is presented; everything below is the same step's second half.
    let at_present = p.total_visual_rows();
    let t1 = Instant::now();
    let paid = p.finish_shape_tail();
    let tail_ms = t1.elapsed().as_secs_f64() * 1e3;
    let settled = p.total_visual_rows();
    // The settled count is NOT stable across worlds and must not be pinned to one:
    // the wrap WIDTH is face-independent but the glyph ADVANCES inside it are not,
    // so a proportional face fits a different number of characters per row than a
    // mono one and the document genuinely has a different number of visual rows in
    // each world. What must hold per hop is that nothing is owed. The cross-arm
    // claim — that both reaches settle on the SAME geometry for a given world — is
    // `theme_preview_shape_law`'s, where a control pipeline makes it provable.
    ensure!(
        !p.shape_tail_owed(),
        "step to {} ended still owing a shaping tail",
        world.name
    );
    if matches!(reach, ShapeReach::Whole) {
        ensure!(
            !paid && at_present == settled,
            "the whole-document arm presented {at_present} of {settled} rows (tail \
             paid: {paid}) — this arm must never narrow"
        );
    }
    // The split arm is NOT required to narrow HERE, because whether it can is the
    // very thing this sweep measures: the presentable budget runs from the
    // document's FIRST row (cosmic-text fills from `buffer.scroll`, which awl holds
    // at 0), so it shortens with depth and covers the whole document at the end.
    // The teeth go where the shipped claim is — the document top, in
    // [`picker_sweep`].
    //
    // WHAT MUST HOLD AT EVERY DEPTH is the CORRECTNESS direction: a step that
    // deferred rows must have declared the debt that pays them, or those rows carry
    // no geometry past the step. The converse is a COST claim, not a correctness
    // one, and it is deliberately only printed: `presentable_reach_height` decides
    // whether to narrow BEFORE the shape, from the last fully-shaped pass's own
    // document height, so a reshape that changes that height can still leave a
    // debt with nothing behind it near the crossover depth. The counts below say
    // how often.
    ensure!(
        at_present >= settled || paid,
        "the step to {} deferred rows ({at_present} of {settled} shaped at the \
         present) without declaring the debt that pays them — nothing would ever \
         shape the rest",
        world.name
    );
    Ok(Some(PickerStep {
        face,
        shape_ms,
        frame_ms: frame.total,
        tail_ms,
        at_present,
        settled,
        owed: paid,
        narrowed: at_present < settled,
    }))
}

/// ONE full arrow-sweep of the theme picker, at one [`ShapeReach`], from one
/// scroll position.
///
/// The route is `theme::THEMES` in its own order, because that is exactly what
/// `overlay::build`'s Theme arm hands the picker and therefore exactly what
/// holding Down walks. `reach` picks which step shape is being timed:
///
/// * [`ShapeReach::Whole`] — today's step. One whole-document reshape, then the
///   frame. `to-present` is the whole thing.
/// * [`ShapeReach::Presentable`] — the split step. Shape what the frame can paint,
///   present, then pay the off-screen tail. `to-present` is what the user waits
///   for; `step` is the same total work as the `Whole` arm.
///
/// `scroll_frac` matters and is swept rather than assumed: the presentable budget
/// runs from the document's FIRST row down past the viewport (cosmic-text fills
/// from the buffer top), so the deeper the scroll the less tail there is to defer
/// and the smaller the saving. At the document's end there is none at all.
fn picker_sweep(
    gpu: &BurstGpu<'_>,
    buffer: &Buffer,
    misspelled: &[crate::spell::Misspelling],
    reach: ShapeReach,
    scroll_frac: f32,
) -> anyhow::Result<()> {
    crate::theme::set_active_by_name("Mangrove");
    let mut p = TextPipeline::new(gpu.device, gpu.queue, gpu.cache, FORMAT);
    p.set_size(BURST_WIDTH as f32, BURST_HEIGHT as f32);
    p.set_dpi(DPI);
    let mut view = live_view(buffer, misspelled.to_vec());
    view.zoom = BURST_ZOOM;
    p.set_view(&view);
    for _ in 0..10 {
        burst_frame(&mut p, gpu.device, gpu.queue, gpu.target_view, false)?;
    }
    // The settled row count is read AFTER the warm frames, not before them: the
    // first prepare's reveal-on-cursor conceal re-lays the markdown markup and the
    // document's visual rows move under it. Reading it early pins a count the editor
    // never actually sits at.
    let total_rows = p.total_visual_rows();
    // Canonicalize the fraction through the REAL clamp rather than trusting it: at
    // `1.0` it names a row past the last one, and `max_scroll` is precisely what
    // decides where the document's END sits. Going through `scroll_by_px` also
    // means the deepest sample is a position the editor would actually rest at.
    let start = p.scroll_by_px(
        crate::render::ScrollPos::at_row(
            (((total_rows as f32) * scroll_frac) as usize).min(total_rows.saturating_sub(1)),
        ),
        0.0,
        BURST_HEIGHT as f32,
    );
    let start_row = start.row;
    view.scroll = start;
    p.set_view(&view);
    for _ in 0..10 {
        burst_frame(&mut p, gpu.device, gpu.queue, gpu.target_view, false)?;
    }
    let whole_rows = p.total_visual_rows();
    let viewport_rows = BURST_HEIGHT as f32 / p.metrics.line_height;
    // NON-VACUITY: a sweep over a document the window already fits has no
    // off-screen tail to defer and would time two identical arms.
    ensure!(
        whole_rows as f32 > 4.0 * viewport_rows,
        "fixture is {whole_rows} rows against a {viewport_rows:.0}-row window — it no \
         longer overflows, and this sweep would measure nothing"
    );

    let arm = match reach {
        ShapeReach::Whole => "whole document, then present (today)",
        ShapeReach::Presentable => "present, then the tail — same step",
    };
    println!();
    println!(
        "---- picker sweep · {arm} · scroll row {start_row}/{whole_rows} ({:.0}%) ----",
        scroll_frac * 100.0
    );
    println!(
        "{:>10} | {:>21} | {:>9} | {:>9} | {:>10} | {:>9} | {:>9} | {:>11}",
        "world", "face", "shape", "frame", "to-present", "tail", "step", "rows shaped"
    );

    let (mut to_present, mut step_total, mut hops) = (0.0f64, 0.0f64, 0usize);
    let (mut narrowed, mut owed) = (0usize, 0usize);
    let (mut shaped_at_present, mut shaped_settled) = (0usize, 0usize);
    for world in crate::theme::THEMES {
        let Some(s) = picker_step(gpu, &mut p, &view, reach, world)? else {
            continue;
        };
        hops += 1;
        narrowed += usize::from(s.narrowed);
        owed += usize::from(s.owed);
        shaped_at_present += s.at_present;
        shaped_settled += s.settled;
        to_present += s.to_present();
        step_total += s.whole_step();
        println!(
            "{:>10} | {:>21} | {:>8.1}ms | {:>8.1}ms | {:>9.1}ms | \
             {:>8.1}ms | {:>8.1}ms | {:>5}/{:<5}",
            world.name,
            s.face,
            s.shape_ms,
            s.frame_ms,
            s.to_present(),
            s.tail_ms,
            s.whole_step(),
            s.at_present,
            s.settled
        );
    }
    println!(
        "  {hops} arrows · input->present {to_present:.1}ms total, {:.1}ms mean · whole \
         step {step_total:.1}ms total, {:.1}ms mean",
        to_present / hops as f64,
        step_total / hops as f64
    );
    // The ROWS witness for the depth curve: how much of the document the arm
    // actually had to shape before it could present. This is the quantity the
    // millis are downstream of, and the one that makes the split's shrinking
    // return legible rather than inferred from timings.
    println!(
        "  deferred rows on {narrowed}/{hops} arrows (debt declared on {owed}) · \
         shaped-before-present {:.0}% of the settled document",
        100.0 * shaped_at_present as f64 / shaped_settled.max(1) as f64
    );
    // NON-VACUITY, placed where the shipped claim is: at the document TOP the split
    // must defer something on every arrow, or this arm is the whole-document step
    // under another name. At DEPTH it is expected to defer less and eventually
    // nothing — that shrinking is the measurement, not a failure.
    assert_depth_curve(reach, scroll_frac, hops, narrowed, owed)
}

/// The split arm's own NON-VACUITY, at the two depths where the shipped claim is
/// specific enough to have teeth. Held apart from [`picker_sweep`] because it is
/// the only part of that function that asserts rather than measures, and because
/// the two ends want opposite things.
///
/// At the document TOP every arrow must defer something, or this arm is the
/// whole-document step under another name. At the document END the debt must
/// follow the DEFERRAL rather than `full_shape_height`'s deliberate over-estimate:
/// a decider that answers the HEIGHT question declares one on every single arrow
/// here (a budget that already reaches the last row is still far under the
/// over-estimate) and buys two whole-document relayouts per arrow with it. The
/// bound there is `< hops` rather than a count, because how many arrows still
/// genuinely defer at max scroll is a property of the fixture and of how much each
/// world's face moves the wrapped-row count.
fn assert_depth_curve(
    reach: ShapeReach,
    scroll_frac: f32,
    hops: usize,
    narrowed: usize,
    owed: usize,
) -> anyhow::Result<()> {
    if !matches!(reach, ShapeReach::Presentable) {
        return Ok(());
    }
    if scroll_frac == 0.0 {
        ensure!(
            narrowed == hops,
            "at the document top the split arm deferred nothing on {} of {hops} \
             arrows — it is measuring the whole-document step under another name",
            hops - narrowed
        );
    }
    if scroll_frac == 1.0 {
        ensure!(
            owed < hops,
            "at the document end the split arm declared a debt on all {hops} arrows \
             while deferring rows on {narrowed} — the debt is being decided against \
             `full_shape_height`'s deliberate over-estimate rather than against the \
             document"
        );
    }
    Ok(())
}

/// Every picker sweep the theme-burst profiler runs: both reaches, at each of
/// TOP / HALF / END.
///
/// The route within a sweep is `theme::THEMES` in its own order, because that is
/// literally what pressing Down through the theme card walks. Each arm gets its own
/// fresh pipeline so neither inherits the other's warm shaping caches.
///
/// The DEPTH axis is swept rather than assumed, and the end is the sample that
/// matters: the presentable budget is measured from the document's FIRST row, so it
/// is exactly there that the split has nothing left to defer.
pub(super) fn run(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &Cache,
    target_view: &wgpu::TextureView,
    buffer: &Buffer,
    misspelled: &[crate::spell::Misspelling],
) -> anyhow::Result<()> {
    let gpu = BurstGpu {
        device,
        queue,
        cache,
        target_view,
    };
    for scroll_frac in [0.0f32, 0.5, 1.0] {
        for reach in [ShapeReach::Whole, ShapeReach::Presentable] {
            picker_sweep(&gpu, buffer, misspelled, reach, scroll_frac)?;
        }
    }
    Ok(())
}
