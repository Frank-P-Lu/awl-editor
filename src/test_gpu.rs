//! The one process-wide headless `(Device, Queue)` shared by every render test.
//!
//! `wgpu::Device`/`Queue` are Arc-backed handles, so callers get a cheap clone
//! of one lazily-created pair instead of standing up an adapter and device
//! apiece (~64ms each). Callers still build their own `Cache`/`TextPipeline`.
//!
//! `None` on a machine with no adapter — cached too, so the probe runs once and
//! callers keep their existing skip. A test needing non-default features/limits,
//! or exercising device loss, must create its own device instead.
//!
//! ## The shared device is PROCESS-GLOBAL STATE, and is guarded like the rest
//!
//! One device shared by every render test is one population of live wgpu
//! objects and one set of wgpu-hal counters, mutated by whichever tests happen
//! to be running. `cargo test` runs in parallel, so that is the same race as
//! the active theme or the swappable fs backend, and it gets the same cure:
//! [`arrive`] — the single choke point both doors go through — ASSERTS that the
//! caller holds [`crate::testlock::serial`], so an unguarded caller panics by
//! name on its first run instead of flaking someone else's measurement.
//!
//! The assertion is measured, not precautionary. Two hundred and forty-five
//! render tests reached this device without the guard, and against the two
//! allocation laws in `render/tests/alloc_bound_law.rs` — which read those
//! process-global counters as a baseline — they were a rolling corruption of
//! the number being measured. The symptom was a law that passed ALONE, passed
//! in the FULL suite, and failed under `cargo test render::` with a *varying*
//! count of failures: the accumulation trace stepped from 6 live objects to 158
//! between two consecutive workloads of a sweep that allocates four apiece.
//! Nothing in that reading was about the suite's GPU footprint. Such a law can
//! never fail CI, which runs unfiltered, and fails a developer every time,
//! because a filter is how you work inside a directory.
//!
//! **The guard must be held for as long as the test holds anything allocated
//! here, not merely across the call to a door.** A `TextPipeline` dropped at the
//! closing brace still moves these counters when it drops; that is why the
//! obligation is on the test (`let _g = crate::testlock::serial();` first line,
//! so it drops last) and cannot be discharged by having the door take the lock
//! and give it back.

/// No shared device on wasm: the test runner is Node, which has no adapter to
/// hand out, and `wgpu`'s handles are not `Sync` there — so the cache below
/// cannot exist as a static at all. `None` is the truthful answer, and it is
/// the one every caller already handles by skipping.
#[cfg(target_arch = "wasm32")]
pub(crate) fn shared_device_queue() -> Option<(wgpu::Device, wgpu::Queue)> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

/// Lazily create (once) and cache the shared headless device+queue, or `None`
/// if no wgpu adapter is available. Never re-probed after the first call.
#[cfg(not(target_arch = "wasm32"))]
fn cached() -> &'static Option<(wgpu::Device, wgpu::Queue)> {
    static GPU: OnceLock<Option<(wgpu::Device, wgpu::Queue)>> = OnceLock::new();
    GPU.get_or_init(|| {
        pollster::block_on(async {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .ok()?;
            adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("awl shared test device"),
                    ..Default::default()
                })
                .await
                .ok()
        })
    })
}

/// Is there an adapter on this machine at all — asked WITHOUT reaching the
/// device.
///
/// The `skipping …: no wgpu adapter` guard clause at the top of a GPU test is a
/// question about the machine, not a use of the device, and it is asked BEFORE
/// the test takes [`crate::testlock::serial`] (there is nothing to serialize
/// yet). Answering it through [`shared_device_queue`] would run [`arrive`] —
/// a submit, a poll and a counter mutation — outside every guard in the process,
/// on some sixty capture tests. This answers it from the cache alone: no
/// arrival, no sweep, nothing for a concurrent measurement to see.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn adapter_present() -> bool {
    cached().is_some()
}

/// No adapter on wasm — see [`shared_device_queue`].
#[cfg(target_arch = "wasm32")]
pub(crate) fn adapter_present() -> bool {
    false
}

/// A cloned handle to the shared test device+queue, or `None` on a GPU-less
/// machine — the cheap refcount-bump replacement for the old per-test
/// `request_adapter`/`request_device` dance. Callers that need a device with
/// non-default features/limits, or that test device loss, must NOT use this —
/// they need a device of their own.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn shared_device_queue() -> Option<(wgpu::Device, wgpu::Queue)> {
    let pair = cached().clone();
    if let Some((device, queue)) = pair.as_ref() {
        arrive(device, queue);
    }
    pair
}

/// THE BOUND. Everything the last caller dropped is handed back to the device
/// before this one starts allocating, so what the suite holds at any instant is
/// one test's worth and not the run's.
///
/// This function and [`with_shared_programs`] are the only two ways a test
/// reaches the shared device, so calling it here needs no roster of tests and
/// no roster of resources: whatever a test allocated, it allocated on this
/// device, and whatever it dropped is swept at the next arrival.
///
/// **Why the sweep is needed at all.** A render test drops its
/// `glyphon::Cache`, `TextAtlas`, offscreen texture and readback buffer at the
/// closing brace and none of it goes away — `prepare()` staged writes to them,
/// and `wgpu_core` pins an `Arc` of every write destination until something
/// submits. Most render tests never submit; they read geometry off the CPU side
/// and return. Measured over `render::tests::` at `--test-threads=1` before
/// this call existed: live wgpu-hal objects on the shared device climbed to
/// **160 201**, sawtoothing back to 3 only at the handful of tests that happen
/// to read pixels back and so submit and poll as a side effect. That is a
/// per-test accumulation with nothing bounding it but which tests happen to be
/// in the run and in what order. [`crate::gpu_alloc::reclaim`] holds the
/// mechanism.
///
/// ⚠️ Bounding this is worth doing on its own terms and is **not** a fix for,
/// or evidence about, the hosted-macOS CI hang — that is a park-forever with
/// memory flat, a different failure mode from the container's prompt
/// `OOMKilled=true`. See [`crate::gpu_alloc`].
#[cfg(not(target_arch = "wasm32"))]
fn arrive(device: &wgpu::Device, queue: &wgpu::Queue) {
    assert!(
        crate::testlock::currently_held(),
        "a test reached the process-wide shared wgpu device without holding \
         `crate::testlock::serial()`. The shared device is process-global state — one object \
         population and one set of wgpu-hal counters — so an unguarded caller races every other \
         test on it exactly as an unguarded theme or fs caller would, and the allocation laws in \
         `render/tests/alloc_bound_law.rs` read those counters as their baseline. Add \
         `let _g = crate::testlock::serial();` as this test's FIRST statement, so it drops LAST \
         and the guard still covers the drop of everything allocated here — a `TextPipeline` \
         dropped at the closing brace moves these counters on its way out. (If this test needs \
         a device of its own — non-default features or limits, or device loss — build one \
         rather than taking this door; see the module doc.)"
    );
    ARRIVALS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let held = traced().then(|| crate::gpu_alloc::live(device));
    crate::gpu_alloc::reclaim(device, queue);
    if let Some(held) = held {
        trace(device, held);
    }
}

/// Every arrival at the shared device, counted — the WITNESS that lets a law
/// which reads the process-global counters prove it had them to itself.
///
/// The assertion in [`arrive`] is what makes exclusivity true; this is what lets
/// a measurement CHECK that it was, instead of trusting it. The two are not the
/// same guarantee: the assertion can only speak for callers that come through
/// this door, and a law whose baseline is quietly wrong reports a bound
/// violation it cannot substantiate — indistinguishable, from inside the bound,
/// from a real regression.
#[cfg(not(target_arch = "wasm32"))]
static ARRIVALS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// How many times any test has reached the shared device in this process.
///
/// A caller reads this before and after a measurement window and compares the
/// difference against the number of arrivals it made itself; anything else came
/// from somewhere the caller does not control. See
/// `render::tests::alloc_bound_law`'s `refuse_unless_exclusive`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn arrivals() -> u64 {
    ARRIVALS.load(std::sync::atomic::Ordering::SeqCst)
}

/// One line per device acquisition on `AWL_GPU_ALLOC_TRACE=1`, and nothing at
/// all otherwise — the raw material every accumulation measurement in this
/// module's doc is built on.
///
/// `held` is what the PREVIOUS test left on the device; `kept` is what survives
/// [`arrive`]'s sweep. The pair is the whole diagnostic: `held` large and `kept`
/// small is one heavy test being cleaned up after, which is fine; `kept`
/// climbing across acquisitions is the suite accumulating, which is not.
///
/// Every render test reaches the shared device through this function or through
/// [`with_shared_programs`], so a trace indexed by call number is a trace
/// indexed by test, with no roster of tests to keep by hand. Run it as
/// `AWL_GPU_ALLOC_TRACE=1 cargo test --bin awl render::tests:: -- \
/// --test-threads=1 --nocapture` and the samples interleave with libtest's own
/// `test … ok` lines on one stream, in order.
///
/// The `themes`/`bgwgsl` stamp is a PROVENANCE ASSERTION, not decoration. A
/// cross-commit pass here has already scored the same binary twice, because two
/// trees extracted within the same second and Cargo reused the artifacts. Both
/// fields are compile-time constants of the tree that built the binary, and the
/// world roster and the size of `background.wgsl` are exactly what a cross-commit
/// comparison of this suite tends to vary — so a trace that claims to come from
/// a tree it did not says so in its own first field.
#[cfg(not(target_arch = "wasm32"))]
fn traced() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("AWL_GPU_ALLOC_TRACE").is_some())
}

#[cfg(not(target_arch = "wasm32"))]
fn trace(device: &wgpu::Device, held: crate::gpu_alloc::GpuLive) {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    // The backend's own counter behaviour, measured once and repeated on every
    // line: a trace read on one backend and compared against a trace read on
    // another is only meaningful alongside which counters each one maintains.
    static PROBE: OnceLock<crate::gpu_alloc::Probe> = OnceLock::new();
    let probe = *PROBE.get_or_init(|| crate::gpu_alloc::probe(device));
    let kept = crate::gpu_alloc::live(device);
    println!(
        "gpu-alloc-trace acquire={} held={} kept={} buffers={} views={} textures={} \
         probe=[{probe}] themes={} bgwgsl={}",
        N.fetch_add(1, Ordering::Relaxed) + 1,
        held.portable(),
        kept.portable(),
        kept.buffers,
        kept.texture_views,
        kept.textures,
        crate::theme::THEMES.len(),
        include_str!("../shaders/background.wgsl").len(),
    );
}

/// Run `f` against the shared device+queue with `gpu_cache` armed, or `None` on
/// a GPU-less machine.
///
/// This is the ONE caller of [`crate::gpu_cache::scoped`], and the reason the
/// cache can be sound at all. `f` receives the shared device rather than
/// choosing one, so nothing built inside it can belong to a different device —
/// which matters because `wgpu::Device`'s `PartialEq` reports two separately
/// requested, simultaneously live devices as EQUAL (measured, wgpu 29.0.3), so
/// a device-keyed cache cannot be written. Identity here is a property of the
/// call shape, not of a comparison.
///
/// The shared device is created once and never dropped, so the programs cached
/// against it stay valid for the process.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn with_shared_programs<R>(
    f: impl FnOnce(&wgpu::Device, &wgpu::Queue) -> R,
) -> Option<R> {
    let (device, queue) = cached().as_ref()?;
    arrive(device, queue);
    Some(crate::gpu_cache::scoped(|| f(device, queue)))
}

#[cfg(all(test, target_arch = "wasm32"))]
pub(crate) fn with_shared_programs<R>(
    _f: impl FnOnce(&wgpu::Device, &wgpu::Queue) -> R,
) -> Option<R> {
    None
}
