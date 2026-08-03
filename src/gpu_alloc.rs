//! The PORTABLE GPU-allocation oracle: how many wgpu objects are ALIVE on a
//! device right now, in a number that means the same thing on Metal, Vulkan,
//! WebGL2 and D3D12.
//!
//! ## Why this exists at all
//!
//! Under a fixed 4 GiB container ceiling at `--test-threads=1`, `render::tests::`
//! walks RSS monotonically to an OOM kill, and how far it gets is
//! commit-correlated. That is a real signal and a fast one, but **RSS is not an
//! oracle**. The same suite peaks at 448 MiB on the dev host's Metal and never
//! dies, so a law written against RSS grades the container's ceiling and the
//! host allocator, not the product. It also cannot run at all where there is no
//! container.
//!
//! What travels is the object count wgpu-hal itself keeps. Every backend
//! increments a counter when it creates a buffer, a texture or a texture view
//! and decrements it when it destroys one, so the number is a property of what
//! the product asked the GPU for — identical arithmetic on every backend, no
//! container, no host allocator, no ceiling.
//!
//! ## Why COUNTS and not BYTES — measured, not assumed
//!
//! `wgpu_types::HalCounters` also carries `buffer_memory` / `texture_memory` /
//! `memory_allocations`, and bytes would be the more expressive unit. They are
//! **not portable**. Read off wgpu-hal 29.0.3's own sources:
//!
//! | counter | metal | vulkan | gles | dx12 |
//! |---|---|---|---|---|
//! | `buffers` / `textures` / `texture_views` | yes | yes | yes | yes |
//! | `buffer_memory` / `texture_memory` | **no** | yes | **no** | yes |
//!
//! Metal and GLES never touch the memory counters — they read a flat zero
//! there. awl ships on Metal and on WebGL2, so a byte-valued law would be
//! silently vacuous on two of awl's four backends and loud on the other two:
//! exactly the "measures the container, not the product" failure this module
//! was written to end. Counts are the largest unit every backend actually
//! maintains, so counts are the unit.
//!
//! ## Why a LIVE count is the right shape for a leak
//!
//! These counters are decremented by wgpu-hal at DESTRUCTION, and a wgpu
//! resource is destroyed when its last internal `Arc` goes — so a live count
//! that only climbs means something is still holding references a caller
//! believes it dropped. [`reclaim`] is the other half: it lets go of the two
//! things that hold them.
//!
//! ## What actually holds them — measured against wgpu-core 29.0.3
//!
//! The obvious answer is "allocations wgpu reclaims only on poll". A poll is
//! **half** of it, and on its own it is not the half that matters here:
//!
//! - `Queue::write_buffer` and `Queue::write_texture` put an `Arc` of the
//!   DESTINATION into `wgpu_core`'s `PendingWrites` (`device/queue.rs`,
//!   `dst_buffers`/`dst_textures`), and that map is emptied in exactly one
//!   place: `pre_submit`. **A caller that stages writes and never submits pins
//!   every buffer and texture it wrote to, for the life of the device**, no
//!   matter how many times it polls.
//! - A resource still owned by an in-flight submission is released by
//!   `LifetimeTracker::triage_submissions`, which a poll drives.
//!
//! The render suite hits the first one constantly: a test builds a pipeline,
//! calls `prepare()` — which writes uniforms, and drives glyphon's atlas
//! uploads — reads geometry back off the CPU side, and returns without ever
//! submitting a frame. So [`reclaim`] submits an empty command stream *and*
//! polls. Measured: with the poll alone, twenty test-shaped workloads climbed
//! 22 → 136 live objects; with both, they stay flat.
//!
//! ⚠️ This module is an INSTRUMENT for the suite's accumulation, not evidence
//! about the hosted-macOS CI hang. Every container death measured under the
//! ceiling above was a prompt SIGKILL with `OOMKilled=true`; the hosted runner
//! parks forever with memory flat. Different failure mode. Bounding what this
//! counts is worth doing on its own terms and is not a fix for that.

/// The wgpu-hal objects alive on one device at one instant — the three classes
/// every backend counts. Signed because [`wgpu_types::InternalCounter`] is, and
/// because a delta is the thing callers actually want.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuLive {
    pub(crate) buffers: i64,
    pub(crate) textures: i64,
    pub(crate) texture_views: i64,
}

impl GpuLive {
    /// The one number a bound is written against.
    pub(crate) fn total(self) -> i64 {
        self.buffers + self.textures + self.texture_views
    }

    /// `self - earlier`, per class — what came into existence (or, negative,
    /// what was reclaimed) between two snapshots.
    pub(crate) fn since(self, earlier: Self) -> Self {
        GpuLive {
            buffers: self.buffers - earlier.buffers,
            textures: self.textures - earlier.textures,
            texture_views: self.texture_views - earlier.texture_views,
        }
    }
}

impl std::fmt::Display for GpuLive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} objects (buffers {}, textures {}, views {})",
            self.total(),
            self.buffers,
            self.textures,
            self.texture_views
        )
    }
}

/// Snapshot `device`'s live wgpu-hal object counts.
///
/// Requires wgpu's `counters` feature. Without it [`wgpu_types::InternalCounter`]
/// compiles to nothing and every `read()` here returns a constant `0` — a law
/// written against a silent zero passes forever, which is why
/// `a_texture_moves_the_portable_allocation_counter` exists and asserts the
/// instrument responds before anything else asserts a bound.
///
/// `Cargo.toml` turns the feature on for EVERY build rather than only for
/// tests. The reader below is test-only because nothing in the product reads it
/// yet, but the feature is not: gating it to dev-dependencies would compile one
/// wgpu for the test binary and a different one for the shipping binary, and
/// the difference would be in exactly the layer being measured. The cost is two
/// relaxed atomics per GPU object created and destroyed.
pub(crate) fn live(device: &wgpu::Device) -> GpuLive {
    let c = device.get_internal_counters();
    GpuLive {
        buffers: c.hal.buffers.read() as i64,
        textures: c.hal.textures.read() as i64,
        texture_views: c.hal.texture_views.read() as i64,
    }
}

/// Let `device` actually destroy what its callers have already dropped.
///
/// Both halves are needed and they are needed in this order — see the module
/// doc for what each one releases:
///
/// 1. an EMPTY submit, which is what drains `PendingWrites` and unpins every
///    buffer and texture a `write_buffer`/`write_texture` still holds an `Arc`
///    of. Ordering-safe: staged writes are applied in the order they were
///    staged, so flushing them early is exactly what the caller's own next
///    submit would have done.
/// 2. a poll, which releases what the PREVIOUS call's submission owned once its
///    fence has passed.
///
/// The poll is non-blocking on purpose. `PollType::wait_indefinitely()` is
/// where the hosted-macOS CI job's threads park forever, and this runs on the
/// path of every render test; a sweep that cannot complete now simply completes
/// at the next call, one workload later.
pub(crate) fn reclaim(device: &wgpu::Device, queue: &wgpu::Queue) {
    queue.submit(std::iter::empty());
    // A poll can legitimately report `Timeout`/`WaitSucceeded` variants and, on
    // a lost device, an error. None of that changes what a caller does next:
    // this is a hint to the driver, not a barrier.
    let _ = device.poll(wgpu::PollType::Poll);
}
