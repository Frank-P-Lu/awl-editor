//! The one process-wide headless `(Device, Queue)` shared by every render test.
//!
//! `wgpu::Device`/`Queue` are Arc-backed handles, so callers get a cheap clone
//! of one lazily-created pair instead of standing up an adapter and device
//! apiece (~64ms each). Callers still build their own `Cache`/`TextPipeline`.
//!
//! `None` on a machine with no adapter — cached too, so the probe runs once and
//! callers keep their existing skip. A test needing non-default features/limits,
//! or exercising device loss, must create its own device instead.

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

/// A cloned handle to the shared test device+queue, or `None` on a GPU-less
/// machine — the cheap refcount-bump replacement for the old per-test
/// `request_adapter`/`request_device` dance. Callers that need a device with
/// non-default features/limits, or that test device loss, must NOT use this —
/// they need a device of their own.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn shared_device_queue() -> Option<(wgpu::Device, wgpu::Queue)> {
    cached().clone()
}
