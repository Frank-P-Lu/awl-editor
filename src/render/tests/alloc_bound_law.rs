//! THE ALLOCATION LAWS — what the render suite may hold on the shared device,
//! measured in a number that travels across backends.
//!
//! The growth was first found with RSS under a fixed 4 GiB container ceiling:
//! `render::tests::` walks memory monotonically to an OOM kill. RSS cannot be
//! the oracle — the same suite peaks at 448 MiB on the dev host's Metal and
//! never dies, so a law written against it grades the container's ceiling and
//! refuses to run where there is no container. [`crate::gpu_alloc`] supplies the
//! replacement: wgpu-hal's own live buffer and texture-view counts.
//!
//! ## The redesign, and what the first draft got wrong
//!
//! An earlier version of this file summed `buffers + textures + texture_views`
//! and justified the choice from wgpu-hal 29.0.3's sources, which say all three
//! are maintained by every backend. It passed on the dev host's Metal, landed,
//! and failed on CI's lavapipe under both conventions with its own message
//! naming the cause: creating a texture moved the live texture count by `0`, and
//! the absolute reading was `textures -8`. Neither is a lavapipe quirk.
//! wgpu-hal 29.0.3's **Vulkan `create_texture` simply omits
//! `counters.textures.add(1)`** while `destroy_texture` still subtracts, so on
//! every Vulkan device the live texture count walks downward from zero. The
//! table was right about metal, gles and dx12 and wrong about the fourth row.
//!
//! Two things changed as a result, and they are the point of this round:
//!
//! - **The unit dropped `textures`.** `buffers` and `texture_views` are
//!   add/sub-balanced on all four backends and map one-for-one from wgpu-core
//!   resources, so their sum means the same thing everywhere.
//!   [`crate::gpu_alloc::GpuLive::portable`] owns it. The cost is stated in that
//!   module: a texture allocated with no view is invisible here.
//! - **The portability claim is now MEASURED, not read.** Every law below runs
//!   [`crate::gpu_alloc::probe`] first and asserts over the classes that
//!   actually responded on the backend present. A source table is a claim about
//!   a version; a probe is a measurement of the machine.
//!
//! ## Four laws, because there are four separable ways this comes undone
//!
//! 1. **the instrument stops working** — drop wgpu's `counters` feature and
//!    every counter reads a constant zero, which passes every bound below
//!    forever without ever having measured anything;
//! 2. **a counter the bound trusts is not conservative** — the `textures`
//!    anomaly again, on some other class or some later wgpu: a count that can
//!    read negative cannot support a bound in either direction;
//! 3. **one test starts costing more** — a helper grows its per-test GPU
//!    footprint;
//! 4. **the suite stops reclaiming** — the footprint is fine but nothing hands
//!    it back, and N tests hold N times one test.
//!
//! Law 4 is the one worth the most, and it is the one with a trap in it. The
//! obvious workload — build a pipeline, draw it, submit, read the pixels back —
//! reclaims TWICE on its own behalf: `read_pixels` polls, and `Queue::submit`
//! maintains the device on its way out. This file's first draft did both and the
//! accumulation law stayed GREEN with the reclaim it exists to check deleted.
//! See [`one_test_shaped_workload`] for what it must therefore not do.
//!
//! ⚠️ None of this is evidence about the hosted-macOS CI hang. The container
//! deaths were prompt `OOMKilled=true` SIGKILLs; the hosted runner parks forever
//! with memory flat. Different failure mode, and bounding this is not proven to
//! prevent that.

use super::super::*;
use super::{dither, headless_dqp};
use crate::gpu_alloc::{self, Class};

/// A square small enough that twenty of them cost nothing and large enough to
/// be a real attachment.
const W: u32 = 64;
const H: u32 = 64;

/// Live portable objects one test-shaped workload may hold. The unit is
/// buffers + texture views, which is backend-independent by construction (one
/// wgpu-core buffer is one hal buffer, one wgpu-core view is one hal view), so
/// this is one number for every backend rather than a per-backend table.
/// Headroom over the measured footprint is deliberate but bounded: a helper
/// that starts allocating a per-test resource nobody noticed lands here.
const PER_TEST_CEILING: i64 = 24;

/// A workload must move the counter by at least this much or the ceiling above
/// is being met by a workload that allocates nothing. The offscreen attachment's
/// view and the background pipeline's uniform buffer are two the workload cannot
/// avoid; the atlas pair is the rest.
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
/// first draft of this file drew a pass and submitted it, and the accumulation
/// law stayed GREEN with `test_gpu`'s reclaim deleted — the workload was doing
/// the reclaiming its own law was there to check for. A workload that allocates
/// and drops without ever touching the queue is also the shape of the tests that
/// actually accumulate: the ones that build a pipeline, read geometry off it and
/// return.
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
/// The wait is BOUNDED and lives only here, in the laws' own setup.
/// `gpu_alloc::reclaim` — the thing on the shared path — stays non-blocking on
/// purpose (an *indefinite* wait is where the hosted-macOS CI threads park), but
/// a non-blocking sweep cannot free what an unfinished submission still owns,
/// and a floor that still contains the previous workload's pending objects made
/// this law's first draft measure a footprint of **-1**. So: wait for the queue
/// to drain, then sweep, then read.
fn settled(device: &wgpu::Device, queue: &wgpu::Queue) -> gpu_alloc::GpuLive {
    gpu_alloc::reclaim(device, queue);
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(10)),
    });
    gpu_alloc::reclaim(device, queue);
    gpu_alloc::live(device)
}

/// The shared device plus a measurement of what its counters actually do here,
/// or `None` on a machine with no adapter.
///
/// Every law takes this door rather than calling
/// [`crate::gpu_alloc::probe`] itself, so no law can assert a bound without
/// having first established that the number under it moves.
fn instrumented() -> Option<(wgpu::Device, wgpu::Queue, gpu_alloc::Probe)> {
    let (device, queue) = crate::test_gpu::shared_device_queue()?;
    let probe = gpu_alloc::probe(&device);
    Some((device, queue, probe))
}

/// Law 1 — THE INSTRUMENT. Creating one object of a class the bound is built on
/// must move that class's counter by exactly one, on whatever backend this gate
/// is running.
///
/// This is the law the reverted draft's equivalent got RIGHT in spirit and wrong
/// in scope: it demanded a response from `textures` too, which no Vulkan device
/// can give (wgpu-hal 29.0.3's `vulkan/device.rs` `create_texture` omits the
/// increment its `destroy_texture` pairs with). Only the classes
/// [`crate::gpu_alloc::GpuLive::portable`] actually sums are required here.
/// `textures` is still probed and still reported — it just cannot fail the gate,
/// and if a later wgpu fixes that omission the probe will say so in this law's
/// own output.
///
/// Without wgpu's `counters` feature `InternalCounter::read` is a `0` constant
/// with no storage behind it, so every bound below would pass on every tree
/// forever and nobody would find out until a container died again. That case is
/// called out by name here, and it is the reason `Cargo.toml` names the feature.
#[test]
fn the_allocation_counter_responds_on_this_backend() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, probe)) = instrumented() else {
        eprintln!("skipping the_allocation_counter_responds_on_this_backend: no wgpu adapter");
        return;
    };
    // Recorded on every run, not only on failure: this one line is what a
    // cross-backend claim about this oracle has to be checked against, and the
    // whole reason this round exists is that the previous one had no such line
    // from any backend but the author's.
    eprintln!("awl gpu-alloc probe: create-deltas {probe}");

    assert!(
        !probe.instrument_is_dead(),
        "no wgpu-hal counter moved when this probe created a buffer, a texture and a view \
         ({probe}). That is the signature of wgpu's `counters` feature being off — with it \
         off, `InternalCounter::read` is a `0` constant with no storage behind it, every \
         allocation law in this file passes vacuously forever, and the render suite's \
         allocation growth goes unwatched. Re-arm `features = [\"counters\"]` on the wgpu \
         dependency in Cargo.toml, for BOTH the native and the wasm32 target sections.",
    );
    for class in Class::PORTABLE {
        assert!(
            probe.responds(class),
            "creating one wgpu object of class `{}` moved wgpu-hal's live `{}` counter by {} \
             instead of 1 \
             (all classes: {probe}). Other counters here did move, so wgpu's `counters` \
             feature is on and this backend does not maintain `{}` the way the allocation \
             bound needs. That class must either be fixed upstream or dropped from \
             `gpu_alloc::Class::PORTABLE` — leaving it in makes the bound read as coverage \
             while guarding nothing, which is exactly how this file's first version reached \
             main.",
            class.name(),
            class.name(),
            probe.delta(class),
            class.name(),
        );
    }
}

/// Law 2 — THE COUNTER IS CONSERVATIVE. No counter the bound trusts may read
/// negative on a settled device.
///
/// A create/destroy count that can go negative cannot support a bound in either
/// direction: the total it contributes to can be dragged down by one class while
/// another leaks, and the "growth" a law watches for never arrives. This is the
/// anomaly CI reported (`textures -8`) turned into a law, and it sweeps ALL
/// THREE classes rather than the two the unit uses — asserting on whichever ones
/// the probe says respond here. So on Metal, where `textures` does respond, this
/// law covers it; on Vulkan, where it does not, the class is excluded from the
/// unit and from this law together, by one measurement rather than by two
/// separate decisions that could drift apart.
#[test]
fn no_counter_the_allocation_bound_trusts_ever_reads_negative() {
    let _g = crate::testlock::serial();
    let Some((device, queue, probe)) = instrumented() else {
        eprintln!(
            "skipping no_counter_the_allocation_bound_trusts_ever_reads_negative: \
             no wgpu adapter"
        );
        return;
    };
    let live = settled(&device, &queue);
    for class in Class::ALL {
        if !probe.responds(class) {
            continue;
        }
        assert!(
            live.get(class) >= 0,
            "wgpu-hal's live `{}` count on the settled shared device reads {}, and a \
             create/destroy count can only go negative if something is decremented that was \
             never incremented. The known shape of this is wgpu-hal 29.0.3's Vulkan \
             `create_texture`, which omits `counters.textures.add(1)` while its \
             `destroy_texture` still subtracts — every texture the process creates and drops \
             walks that counter down by one. A class that does this cannot carry a bound in \
             either direction: it drags the total down while something else leaks. Drop it \
             from `gpu_alloc::Class::PORTABLE` and say so, or fix it upstream. Full reading: \
             {live}; probe: {probe}",
            class.name(),
            live.get(class),
        );
    }
}

/// Law 3 — ONE TEST'S FOOTPRINT. What a single render test holds on the shared
/// device stays small, and is not zero.
///
/// The upper bound is the half that fails when the suite grows its per-test
/// allocation; the lower bound is what stops the upper one from being met by a
/// workload that quietly stopped allocating.
#[test]
fn one_render_test_allocates_a_bounded_number_of_gpu_objects() {
    let _g = crate::testlock::serial();
    let Some((device, queue, _probe)) = instrumented() else {
        eprintln!(
            "skipping one_render_test_allocates_a_bounded_number_of_gpu_objects: no wgpu adapter"
        );
        return;
    };
    // A warmup first: the process-wide program cache and the shared device's
    // own one-time buffers are not this test's footprint, and whichever test
    // ran first in this process may or may not have already paid for them.
    one_test_shaped_workload(theme::THEMES[theme::DEFAULT_THEME].background);
    let base = settled(&device, &queue).portable();

    one_test_shaped_workload(theme::THEMES[theme::DEFAULT_THEME].background);
    let footprint = gpu_alloc::live(&device).portable() - base;
    // On every run, for the same reason law 1 reports its probe: the measured
    // number is what a cross-backend claim has to be checked against, and this
    // round exists because the previous one had no such reading from lavapipe.
    eprintln!(
        "awl gpu-alloc footprint: one workload = {footprint} portable objects \
         (base {base}, ceiling {PER_TEST_CEILING}, floor {MIN_FOOTPRINT}); held now: {}",
        gpu_alloc::live(&device)
    );

    assert!(
        footprint >= MIN_FOOTPRINT,
        "a whole test-shaped workload — a `TextPipeline` with its own `glyphon::Cache` and \
         `TextAtlas`, an offscreen attachment and its view, and a background pipeline with \
         its uniforms — moved wgpu-hal's live portable object count by only {footprint}. The \
         bound below is then being met by a workload that allocates nothing, which is not \
         the same as a suite that allocates little. Full reading: {}",
        gpu_alloc::live(&device),
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

/// Law 4 — THE ACCUMULATION. Twenty tests must hold what one test holds.
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
    let Some((device, _queue, _probe)) = instrumented() else {
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
        samples.push(gpu_alloc::live(&device).portable());
    }
    let first = samples[0];
    let last = *samples.last().expect("THEMES is never empty");
    let peak = *samples.iter().max().expect("THEMES is never empty");
    eprintln!(
        "awl gpu-alloc accumulation: {} workloads, first {first} peak {peak} last {last} \
         (slack {ACCUMULATION_SLACK}); held now: {}; trace {samples:?}",
        worlds.len(),
        gpu_alloc::live(&device),
    );

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
         until the process ends. Per-sample trace: {samples:?}",
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
