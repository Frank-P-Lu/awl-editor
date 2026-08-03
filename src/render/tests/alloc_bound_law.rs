//! THE ALLOCATION LAWS — what the render suite may hold on the shared device,
//! measured in a number that travels across backends.
//!
//! The growth was first found with RSS under a fixed 4 GiB container ceiling:
//! `render::tests::` walks memory monotonically to an OOM kill. RSS cannot be
//! the oracle — the same suite peaks at 448 MiB on the dev host's Metal and
//! never dies, so a law written against it grades the container's ceiling and
//! refuses to run where there is no container.
//! [`crate::gpu_alloc`] supplies the replacement: wgpu-hal's own live
//! buffer/texture/texture-view counts, which every backend awl ships maintains.
//!
//! Measured with that counter over `render::tests::` at `--test-threads=1`
//! before [`crate::test_gpu`] swept anything: live objects on the shared device
//! climbed to **160 201**, in a sawtooth that fell back to 3 only at the
//! handful of tests that read pixels back and so polled the device as a side
//! effect. Nothing bounded it but which tests were in the run and in what
//! order.
//!
//! Three laws, because there are three separable ways this comes undone:
//!
//! 1. **the instrument stops working** — drop wgpu's `counters` feature and
//!    every counter below reads a constant zero, which passes laws 2 and 3
//!    forever without ever having measured anything;
//!
//! 2. **one test starts costing more** — a helper grows its per-test GPU
//!    footprint;
//!
//! 3. **the suite stops reclaiming** — the footprint is fine but nothing hands
//!    it back, and N tests hold N times one test.
//!
//! Law 3 is the one worth the most, and it is the one with a trap in it. The
//! obvious workload — build a pipeline, draw it, submit, read the pixels back —
//! reclaims TWICE on its own behalf: `read_pixels` polls, and `Queue::submit`
//! maintains the device on its way out. This file's first draft did both and
//! law 3 stayed GREEN with the reclaim it exists to check deleted. See
//! [`one_test_shaped_workload`] for what it must therefore not do.
//!
//! ⚠️ None of this is evidence about the hosted-macOS CI hang. The container
//! deaths were prompt `OOMKilled=true` SIGKILLs; the hosted runner parks forever
//! with memory flat. Different failure mode, and bounding this is not proven to
//! prevent that.

use super::super::*;
use super::{dither, headless_dqp};
use crate::gpu_alloc;

/// A square small enough that twenty of them cost nothing and large enough to
/// be a real attachment.
const W: u32 = 64;
const H: u32 = 64;

/// Live objects one test-shaped workload may hold — measured at **6** on the
/// dev host's Metal (the `glyphon::Cache`/`TextAtlas` pair the suite builds per
/// call is 2 textures + 2 buffers; the offscreen attachment and the background
/// pipeline's uniform are the rest), with headroom for a backend that counts
/// texture views or splits a uniform differently. A helper that starts
/// allocating a per-test resource nobody noticed lands here.
const PER_TEST_CEILING: i64 = 24;

/// A workload must move the counter by at least this much or the ceiling above
/// is being met by a workload that allocates nothing.
const MIN_FOOTPRINT: i64 = 3;

/// Slack on the accumulation law. A non-blocking sweep cannot free what an
/// in-flight submission still owns, so the tail of a sweep legitimately lags by
/// a workload or two; anything past that is accumulation.
const ACCUMULATION_SLACK: i64 = 2 * PER_TEST_CEILING;

/// One test's shape: allocate, look, drop. Nothing is submitted and nothing is
/// read back.
///
/// The device is reached through the REAL door, so the bound under test is the
/// one on that door and not one this file invented. What it allocates is what an
/// ordinary render test allocates — a full `TextPipeline`, which is where the
/// suite's per-call `glyphon::Cache` and `TextAtlas` come from, an offscreen
/// colour attachment and its view, and a world's background pipeline with its
/// uniforms.
///
/// ⚠️ **THE TWO THINGS IT MUST NOT DO, both found by mutation rather than by
/// reasoning.** `dither::read_pixels` polls with `wait_indefinitely`; and
/// `Queue::submit` maintains the device on its way out. Both are reclaims. The
/// first draft of this file drew a pass and submitted it, and law 3 stayed GREEN
/// with `test_gpu`'s reclaim deleted — the workload was doing the reclaiming its
/// own law was there to check for. A workload that allocates and drops without
/// ever touching the queue is also the shape of the tests that actually
/// accumulate: the ones that build a pipeline, read geometry off it and return.
fn one_test_shaped_workload(bg: theme::Background) {
    let Some((device, queue, _pipeline)) = headless_dqp(W as f32, H as f32) else {
        return;
    };
    let (_texture, _tview) = dither::offscreen(&device, W, H);
    let mut ground = crate::background::BackgroundPipeline::new(
        &device,
        dither::FMT,
        super::backgrounds_item69::bg_desc_for(bg),
    );
    ground.prepare(&queue, W, H, 0.0, 0.0, Default::default(), 1.0);
}

/// Settle the device and report what it is holding, so a measurement starts
/// from a real floor rather than from whatever the previous test in this
/// process left in flight.
///
/// The wait is BOUNDED and lives only here, in the law's own setup.
/// `gpu_alloc::reclaim` — the thing on the shared path — stays non-blocking on
/// purpose (an *indefinite* wait is where the hosted-macOS CI threads park), but
/// a non-blocking sweep cannot free what an unfinished submission still owns,
/// and a floor that still contains the previous workload's pending objects made
/// this law's first draft measure a footprint of **-1**. So: wait for the queue
/// to drain, then sweep, then read.
fn settled(device: &wgpu::Device, queue: &wgpu::Queue) -> i64 {
    gpu_alloc::reclaim(device, queue);
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(10)),
    });
    gpu_alloc::reclaim(device, queue);
    gpu_alloc::live(device).total()
}

/// Law 1 — THE INSTRUMENT. Creating one texture must move the counter by
/// exactly one texture, on whatever backend this gate is running.
///
/// Without wgpu's `counters` feature `InternalCounter::read` is a `0` constant
/// with no storage behind it, so laws 2 and 3 would pass on every tree forever
/// and nobody would find out until a container died again. This law is what
/// makes the other two mean something, and it is the reason `Cargo.toml` names
/// the feature.
#[test]
fn a_texture_moves_the_portable_allocation_counter() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = crate::test_gpu::shared_device_queue() else {
        eprintln!("skipping a_texture_moves_the_portable_allocation_counter: no wgpu adapter");
        return;
    };
    let before = gpu_alloc::live(&device);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl alloc-bound-law probe"),
        size: wgpu::Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: dither::FMT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let after = gpu_alloc::live(&device);
    assert_eq!(
        after.since(before).textures,
        1,
        "creating one wgpu texture moved wgpu-hal's live texture count by {} instead of 1 \
         (before: {before}; after: {after}). Either wgpu's `counters` feature is no longer \
         enabled in Cargo.toml — in which case every counter here reads a constant zero and \
         the allocation laws below are vacuous — or this backend does not maintain the \
         counters the portable oracle is built on.",
        after.since(before).textures,
    );
    drop(texture);
    // NON-VACUOUS in the other direction: the counter must come DOWN too, or a
    // law that watches for growth is watching a number that only ever grows.
    let reclaimed = settled(&device, &queue);
    assert!(
        reclaimed <= before.total(),
        "a dropped texture left the live count at {reclaimed}, above the {} it started from — \
         `gpu_alloc::reclaim` is supposed to be what actually hands a dropped resource back \
         to the device",
        before.total(),
    );
}

/// Law 2 — ONE TEST'S FOOTPRINT. What a single render test holds on the shared
/// device stays small, and is not zero.
///
/// The upper bound is the half that fails when the suite grows its per-test
/// allocation; the lower bound is what stops the upper one from being met by a
/// workload that quietly stopped allocating.
#[test]
fn one_render_test_allocates_a_bounded_number_of_gpu_objects() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = crate::test_gpu::shared_device_queue() else {
        eprintln!(
            "skipping one_render_test_allocates_a_bounded_number_of_gpu_objects: no wgpu adapter"
        );
        return;
    };
    // A warmup first: the process-wide program cache and the shared device's
    // own one-time buffers are not this test's footprint, and whichever test
    // ran first in this process may or may not have already paid for them.
    one_test_shaped_workload(theme::THEMES[theme::DEFAULT_THEME].background);
    let base = settled(&device, &queue);

    one_test_shaped_workload(theme::THEMES[theme::DEFAULT_THEME].background);
    let footprint = gpu_alloc::live(&device).total() - base;

    assert!(
        footprint >= MIN_FOOTPRINT,
        "a whole test-shaped workload — a `TextPipeline` with its own `glyphon::Cache` and \
         `TextAtlas`, an offscreen attachment, a background pipeline and a submitted draw — \
         moved wgpu-hal's live object count by only {footprint}. The bound below is then \
         being met by a workload that allocates nothing, which is not the same as a suite \
         that allocates little.",
    );
    assert!(
        footprint <= PER_TEST_CEILING,
        "one render test now holds {footprint} live wgpu objects on the shared device, past \
         the {PER_TEST_CEILING} this suite is allowed. Every test in `render::tests::` pays \
         this, so a helper that grew its per-test GPU footprint grew the whole suite's. \
         Detail: {}",
        gpu_alloc::live(&device),
    );
}

/// Law 3 — THE ACCUMULATION. Twenty tests must hold what one test holds.
///
/// The sweep is over the whole `THEMES` roster rather than a world someone
/// picked, so a twenty-first world is measured the day it lands and cannot dodge
/// this by not being on a list. The roster is also the axis the commit that
/// first showed the growth moved, which makes it the axis worth sweeping even
/// though the bound itself is roster-size independent: it compares the end of
/// the sweep against the START of the same sweep, so adding worlds cannot loosen
/// or tighten it.
#[test]
fn the_render_suite_does_not_accumulate_gpu_objects_across_tests() {
    let _g = crate::testlock::serial();
    let Some((device, _queue)) = crate::test_gpu::shared_device_queue() else {
        eprintln!(
            "skipping the_render_suite_does_not_accumulate_gpu_objects_across_tests: \
             no wgpu adapter"
        );
        return;
    };
    let worlds = theme::THEMES;
    let mut samples: Vec<i64> = Vec::with_capacity(worlds.len());
    for t in worlds.iter() {
        one_test_shaped_workload(t.background);
        samples.push(gpu_alloc::live(&device).total());
    }
    let first = samples[0];
    let last = *samples.last().expect("THEMES is never empty");
    let peak = *samples.iter().max().expect("THEMES is never empty");

    // NON-VACUOUS: the sweep really did allocate. If every workload were a
    // no-op the flatness below would hold for the wrong reason.
    assert!(
        first >= MIN_FOOTPRINT,
        "the first workload of the sweep left only {first} live wgpu objects on the shared \
         device — the sweep is not allocating, so the accumulation bound below is measuring \
         nothing",
    );
    assert!(
        peak <= first + ACCUMULATION_SLACK,
        "{} test-shaped workloads in a row peaked at {peak} live wgpu objects on the shared \
         device, having started at {first} after the very first one — so the suite is holding \
         roughly {:.1} workloads' worth rather than one, and `render::tests::` accumulates \
         until the process ends (measured at 160 201 objects before `test_gpu` swept on \
         arrival). Per-sample trace: {samples:?}",
        worlds.len(),
        peak as f64 / first.max(1) as f64,
    );
    // The end of the sweep specifically, not just its peak: a slow leak that
    // only shows up late is the shape this suite actually had.
    assert!(
        last <= first + ACCUMULATION_SLACK,
        "the {}th test-shaped workload left {last} live wgpu objects on the shared device \
         against {first} after the first — the suite reclaims less than it allocates. \
         Per-sample trace: {samples:?}",
        worlds.len(),
    );
}
