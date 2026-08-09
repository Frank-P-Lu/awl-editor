//! The PORTABLE GPU-allocation oracle: how many wgpu objects are alive on a
//! device right now, in a number that means the same thing on Metal, Vulkan,
//! WebGL2 and D3D12 — and a runtime PROBE that refuses to let the laws above it
//! believe that claim without measuring it on the backend actually present.
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
//! ## The unit: BUFFERS AND TEXTURE VIEWS, and deliberately not textures
//!
//! `wgpu_types::HalCounters` exposes `buffers`, `textures` and `texture_views`
//! as the three object classes every backend nominally maintains, plus
//! `buffer_memory` / `texture_memory`, which only vulkan and dx12 maintain
//! (metal and gles read a flat zero, so a byte-valued law is silently vacuous on
//! two of awl's four backends). The first draft of this module therefore summed
//! all three counts. **That draft failed on CI's lavapipe**, and the reason is
//! not a subtlety — it is an outright accounting defect in wgpu-hal 29.0.3's
//! Vulkan backend:
//!
//! | backend (`wgpu-hal/src/…`) | `create_texture` | `destroy_texture` |
//! |---|---|---|
//! | metal (`device.rs:517,531`) | `textures.add(1)` | `textures.sub(1)` |
//! | gles (`device.rs:977,1012`) | `textures.add(1)` | `textures.sub(1)` |
//! | dx12 (`device.rs:578,595`) | `textures.add(1)` | `textures.sub(1)` |
//! | **vulkan** (`device.rs:1052,1125`) | **nothing at all** | `textures.sub(1)` |
//!
//! Vulkan's `create_texture` never increments; its `destroy_texture` still
//! decrements; the only `textures.add(1)` on that backend is `add_raw_texture`,
//! which is for externally-owned images. So on Vulkan the live texture count
//! **starts at zero and walks downward by one per texture the process creates
//! and drops.** Both halves of CI's failure message fall straight out of that:
//! creating a texture moved the count by `0`, and the absolute reading was
//! `textures -8`. Nothing about lavapipe, nothing about software rasterisers —
//! a missing line, on every Vulkan device there is.
//!
//! `buffers` and `texture_views` are add/sub-balanced on all four backends, so
//! their sum is a count of live hal objects with the SAME meaning on every
//! backend awl ships. It is not a count of caller-visible Rust handles: a view
//! whose caller drops the backing `Texture` can keep more than one hal view
//! live. The probe below measures that lifetime shape instead of assuming its
//! multiplier. [`GpuLive::portable`] is the unit's one owner.
//!
//! ⚠️ **The gap this leaves, measured rather than argued.** Views are created
//! one-for-one with textures here (`dither::offscreen` returns a texture *and*
//! its view; glyphon's `TextAtlas` makes a view per atlas texture), but a view
//! is not a stand-in for a texture at the instant the count is READ. Nothing in
//! `PendingWrites` pins a view — the map holds `dst_buffers` and `dst_textures`
//! only — so a dropped view is destroyed promptly while the texture it looked at
//! stays pinned. Measured on Metal, one settled test-shaped workload leaves
//! **7 buffers, 0 views, 2 textures**; with the sweep in
//! `test_gpu::arrive` deleted, twenty workloads leave **87 buffers, 0 views, 42
//! textures**.
//!
//! So the unit sees the BUFFER half of the pin and not the texture half: about
//! four countable objects per unreclaimed workload against two it cannot count.
//! The two halves move in lockstep for this suite, so the accumulation law fires
//! on the growth — but it undercounts it, and **a leak that pinned only textures
//! and no buffers would be invisible here.** That is the price of a unit that has
//! to mean the same thing on a Vulkan device whose texture counter runs
//! backwards, and it is the price this round chose to pay rather than ship a
//! third number that is only true on one backend.
//!
//! ## Why READING THE SOURCE IS NOT ENOUGH, and what [`probe`] is for
//!
//! The table above is read off wgpu-hal 29.0.3, and so was the first draft's
//! (correct, as far as it went) portability argument. The draft still shipped a
//! non-portable oracle, because a source reading is a claim about a version and
//! a backend, made on a host that runs neither. [`probe`] converts the claim
//! into a measurement taken on whatever backend the gate is actually running:
//! it creates one object of each class, watches the counter, and reports the
//! observed delta. The laws assert over the classes that RESPOND, not over the
//! classes a table says should. When upstream fixes Vulkan's `create_texture`,
//! the probe notices without anyone editing this file.
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
//! polls.
//!
//! ⚠️ This module is an INSTRUMENT for the suite's accumulation, not evidence
//! about the hosted-macOS CI hang. Every container death measured under the
//! ceiling above was a prompt SIGKILL with `OOMKilled=true`; the hosted runner
//! parks forever with memory flat. Different failure mode. Bounding what this
//! counts is worth doing on its own terms and is not a fix for that.

/// The wgpu-hal object counts on one device at one instant. All three classes
/// are carried because a diagnostic that hides the miscounted one cannot
/// explain itself; only [`Self::portable`] is what a bound is written against.
///
/// Signed because [`wgpu_types::InternalCounter`] is — and because, as the
/// module doc records, one of these classes really does go negative.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuLive {
    pub(crate) buffers: i64,
    pub(crate) textures: i64,
    pub(crate) texture_views: i64,
}

/// The three object classes, as an axis a law can sweep by name rather than by
/// three copies of the same assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    Buffers,
    Textures,
    TextureViews,
}

impl Class {
    /// Every class, so a sweep cannot silently omit one.
    pub(crate) const ALL: [Class; 3] = [Class::Buffers, Class::Textures, Class::TextureViews];

    /// The two classes whose accounting is balanced on metal, vulkan, gles AND
    /// dx12 — the ones [`GpuLive::portable`] sums, and the ones a backend must
    /// actually maintain for the laws above to mean anything.
    pub(crate) const PORTABLE: [Class; 2] = [Class::Buffers, Class::TextureViews];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Class::Buffers => "buffers",
            Class::Textures => "textures",
            Class::TextureViews => "texture_views",
        }
    }
}

impl GpuLive {
    /// This snapshot's count for one class.
    pub(crate) fn get(self, class: Class) -> i64 {
        match class {
            Class::Buffers => self.buffers,
            Class::Textures => self.textures,
            Class::TextureViews => self.texture_views,
        }
    }

    /// THE UNIT. The classes every backend counts the same way — see the
    /// module doc for why `textures` is not among them.
    pub(crate) fn portable(self) -> i64 {
        Class::PORTABLE.iter().map(|&c| self.get(c)).sum()
    }

    /// `self - earlier`, per class.
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
            "{} portable objects (buffers {}, views {}; textures {} — not in the unit)",
            self.portable(),
            self.buffers,
            self.texture_views,
            self.textures,
        )
    }
}

/// What one object of each class did to its counter on THIS backend: the delta
/// observed across a single create. `1` is a class that responds; `0` is a class
/// this backend does not maintain (or the whole `counters` feature compiled
/// out); anything else is a backend counting in units nobody here understands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Probe {
    pub(crate) created: GpuLive,
    /// Live view-counter delta after creating one view and dropping the
    /// caller's backing `Texture`, while retaining only the `TextureView`.
    view_only_pin: i64,
}

impl Probe {
    /// The observed create-delta for one class.
    pub(crate) fn delta(self, class: Class) -> i64 {
        self.created.get(class)
    }

    /// Does this backend maintain `class` at all?
    pub(crate) fn responds(self, class: Class) -> bool {
        self.delta(class) == 1
    }

    /// No class moved at all — the signature of wgpu's `counters` feature being
    /// off, where every `read()` is a `0` constant with no storage behind it.
    pub(crate) fn instrument_is_dead(self) -> bool {
        Class::ALL.iter().all(|&c| self.delta(c) == 0)
    }

    /// Does retaining only a caller-visible view remain observable?
    pub(crate) fn view_only_pin(self) -> i64 {
        self.view_only_pin
    }
}

impl std::fmt::Display for Probe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for c in Class::ALL {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{} {:+}", c.name(), self.delta(c))?;
        }
        write!(f, "; view-only pin {:+}", self.view_only_pin)
    }
}

/// Snapshot `device`'s live wgpu-hal object counts.
///
/// Requires wgpu's `counters` feature. Without it [`wgpu_types::InternalCounter`]
/// compiles to nothing and every `read()` here returns a constant `0` — a law
/// written against a silent zero passes forever, which is why [`probe`] exists
/// and why the first law in `alloc_bound_law.rs` asserts the instrument responds
/// before anything else asserts a bound.
///
/// `Cargo.toml` turns the feature on for EVERY build rather than only for
/// tests. The reader here is test-only because nothing in the product reads it
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

/// MEASURE the instrument, on the backend that is actually here.
///
/// Creates one buffer, one texture and one view of that texture, reading the
/// counters between each, then drops the caller's texture while retaining its
/// view and measures that lifetime too. This is the whole difference
/// between this module and its reverted first draft: the draft asserted a
/// portability table read off wgpu-hal's sources, on a host that runs only one
/// of the four backends in it, and CI falsified the table's `textures` row on
/// first contact. A claim about a backend is measured on that backend or it is
/// not made.
///
/// Allocations are the smallest the API permits — the point is which counter
/// moves, not by how much. The view is created last and on the probe's own
/// texture, because a view cannot exist without one; the texture's own delta is
/// read before the view is made, so the two classes never share a measurement.
pub(crate) fn probe(device: &wgpu::Device) -> Probe {
    let before_buffer = live(device);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("awl gpu-alloc probe buffer"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let after_buffer = live(device);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl gpu-alloc probe texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let after_texture = live(device);

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let after_view = live(device);

    drop(texture);
    let with_view_only = live(device);
    drop(view);
    drop(buffer);

    Probe {
        created: GpuLive {
            buffers: after_buffer.since(before_buffer).buffers,
            textures: after_texture.since(after_buffer).textures,
            texture_views: after_view.since(after_texture).texture_views,
        },
        view_only_pin: with_view_only.since(after_texture).texture_views,
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
