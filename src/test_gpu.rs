//! THE one process-wide headless `(Device, Queue)` for tests (item 138's sibling:
//! GPU device-creation cost, not the actions/apply/schedule work that item owns).
//!
//! Every render/pipeline test used to run its own `Instance::new` ->
//! `request_adapter` -> `request_device` dance — a real Metal/Vulkan device
//! stand-up, paid fresh by every single test regardless of what it asserted.
//! Measured at ~430ms/test over 613 `render::tests::` tests (326s of the suite's
//! 8m51s), that setup cost dwarfed the work under test.
//!
//! `wgpu::Device` and `wgpu::Queue` are `Clone + Send + Sync` Arc-backed handles
//! (cloning bumps a refcount, no GPU work), so ONE device is created lazily on
//! first use and every caller gets a cheap clone of the same pair. Callers still
//! build their OWN [`glyphon::Cache`] / `TextPipeline` / pipeline-under-test —
//! only the Instance/Adapter/Device/Queue stand-up is shared.
//!
//! `None` on a GPU-less machine (CI, a headless box with no adapter) — cached
//! too, so a GPU-less run still pays the probe exactly once, and every caller
//! must keep degrading to its existing clean skip.

use std::sync::OnceLock;

/// Lazily create (once) and cache the shared headless device+queue, or `None`
/// if no wgpu adapter is available. Never re-probed after the first call.
fn cached() -> &'static Option<(wgpu::Device, wgpu::Queue)> {
    static GPU: OnceLock<Option<(wgpu::Device, wgpu::Queue)>> = OnceLock::new();
    GPU.get_or_init(|| {
        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
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

/// A cloned handle to the shared test device+queue, or `None` on a GPU-less
/// machine — the cheap refcount-bump replacement for the old per-test
/// `request_adapter`/`request_device` dance. Callers that need a device with
/// non-default features/limits, or that test device loss, must NOT use this —
/// they need a device of their own.
pub(crate) fn shared_device_queue() -> Option<(wgpu::Device, wgpu::Queue)> {
    cached().clone()
}
