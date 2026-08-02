//! The one owner of every GPU object a pipeline builds that is pure CODE, and
//! the cache that stops the test suite rebuilding those objects per instance.
//!
//! A `wgpu::ShaderModule`, a `wgpu::RenderPipeline` and a
//! `wgpu::BindGroupLayout` are programs and shapes: nothing a caller can write
//! to, nothing that carries a theme, a uniform, an instance or a frame. The
//! things a caller CAN mutate — uniform buffers, bind groups, instance buffers
//! and the CPU-side tints uploaded through them — stay per-instance and are
//! never cached here. That split is what makes sharing safe: two pipeline
//! instances built from one cached program cannot see each other's state,
//! because the program holds none. A world a test asks for still arrives
//! through the same uniforms it always did.
//!
//! `selection.rs` already states the rule for its own module ("the WGSL is
//! translated to the backend's shading language once per device instead of
//! once per pipeline"). This module holds that rule one level up, where the
//! cost actually lives. `TextPipeline::new` builds 8 shader modules and ~33
//! render pipelines, and the LIVE APP pays that ONCE PER LAUNCH while the test
//! suite paid it 795 times per process: 1675 `background.wgsl` translations —
//! 1346 lines of WGSL — and roughly 26 000 render pipelines against ONE
//! device, measured on the dev host at 44 ms per `TextPipeline::new`, 39.9 s of
//! a single `cargo test --bin awl` run. On hosted macOS the Metal stack is
//! virtualised and the runner has three vCPUs, and that same churn is where the
//! `mac (build + test)` job wedged.
//!
//! ## Why the cache is SCOPED and not keyed by device
//!
//! The obvious shape — a process-wide map keyed by `wgpu::Device` — cannot be
//! written, because `wgpu::Device`'s `PartialEq` is not an identity. Measured
//! against wgpu 29.0.3 on this repo's own Metal adapter: two SEPARATELY
//! REQUESTED, SIMULTANEOUSLY LIVE devices compare EQUAL. A cache that trusted
//! it handed one device's `BindGroupLayout` to another, and 648 of 3616 unit
//! tests died inside `wgpu-core` with `BindGroupLayout[Id(6,1)] does not
//! exist`.
//!
//! So identity is established by CONSTRUCTION instead of by comparison. The
//! cache is armed only inside [`scoped`], and the only caller of `scoped` is
//! [`crate::test_gpu::with_shared_programs`], which hands its closure the ONE
//! process-wide shared test device — there is no way to run that closure
//! against a different one. Outside `scoped`, every function here is a plain
//! pass-through, so the live app, `--screenshot`, the benches and every test
//! that stands up its own device build exactly what they built before.
//!
//! The cache itself is process-wide behind a mutex, and deliberately NOT
//! thread-local: libtest gives every test its OWN thread, so a thread-local
//! cache is a fresh cache 3616 times over — measured, it left `builds` at
//! 86 061 and `TextPipeline::new` at 46 ms, i.e. no change at all. Only the
//! ARMED flag is per-thread, because the scope it tracks is per call.

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// The WGSL sources, one variant each. Owning every `include_str!` here is what
/// makes `create_shader_module` a single-owner call — `gpu_cache_law` greps
/// `src/` and fails on any other caller.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Shader {
    Background,
    Blur,
    Caret,
    CaretGlyph,
    Image,
    Lava,
    Selection,
    SpellUnderline,
}

impl Shader {
    fn label(self) -> &'static str {
        match self {
            Shader::Background => "background shader",
            Shader::Blur => "blur shader",
            Shader::Caret => "caret shader",
            Shader::CaretGlyph => "caret glyph shader",
            Shader::Image => "image shader",
            Shader::Lava => "lava shader",
            Shader::Selection => "selection shader",
            Shader::SpellUnderline => "spell underline shader",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Shader::Background => include_str!("../shaders/background.wgsl"),
            Shader::Blur => include_str!("../shaders/blur.wgsl"),
            Shader::Caret => include_str!("../shaders/caret.wgsl"),
            Shader::CaretGlyph => include_str!("../shaders/caret_glyph.wgsl"),
            Shader::Image => include_str!("../shaders/image.wgsl"),
            Shader::Lava => include_str!("../shaders/lava.wgsl"),
            Shader::Selection => include_str!("../shaders/selection.wgsl"),
            Shader::SpellUnderline => include_str!("../shaders/spellunderline.wgsl"),
        }
    }
}

fn compile(device: &wgpu::Device, which: Shader) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(which.label()),
        source: wgpu::ShaderSource::Wgsl(which.source().into()),
    })
}

#[derive(Default)]
struct Programs {
    shaders: HashMap<Shader, wgpu::ShaderModule>,
    layouts: HashMap<&'static str, wgpu::BindGroupLayout>,
    pipelines: HashMap<(&'static str, wgpu::TextureFormat), wgpu::RenderPipeline>,
}

thread_local! {
    /// Armed only between [`scoped`]'s entry and exit — see this module's own
    /// doc for why that scope, and not a device key, is what makes a hit sound.
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

/// Every entry here belongs to the shared test device, because [`scoped`] is
/// the only thing that lets an entry in and `test_gpu` is `scoped`'s only
/// caller. `wgpu`'s handles are `Send + Sync` on native; on wasm they are not,
/// and there is nothing to amortise there either, so this half does not exist.
#[cfg(not(target_arch = "wasm32"))]
fn programs() -> &'static Mutex<Programs> {
    static PROGRAMS: OnceLock<Mutex<Programs>> = OnceLock::new();
    PROGRAMS.get_or_init(|| Mutex::new(Programs::default()))
}

/// wasm arms nothing (`test_gpu::with_shared_programs` is `None` there), so
/// every lookup below short-circuits before it can reach this.
#[cfg(target_arch = "wasm32")]
fn programs() -> &'static Mutex<Programs> {
    unreachable!("the gpu_cache is never armed on wasm")
}

/// Every object this module actually built — cache misses and uncached
/// pass-throughs alike. `gpu_cache_law` reads it: a second `TextPipeline` on
/// the shared device must move it by ZERO.
static BUILDS: AtomicU64 = AtomicU64::new(0);

/// Arm the thread-local cache for the duration of `f`.
///
/// **Do not call this directly.** [`crate::test_gpu::with_shared_programs`] is
/// the one caller, and it is the one that can guarantee the precondition every
/// hit depends on: that everything built inside `f` is built on the ONE
/// process-wide shared test device. Arming this around a closure that touches
/// any other device would hand that device another's programs.
#[cfg(test)]
pub(crate) fn scoped<R>(f: impl FnOnce() -> R) -> R {
    let was = ARMED.with(|a| a.replace(true));
    let out = f();
    ARMED.with(|a| a.set(was));
    out
}

fn armed() -> bool {
    ARMED.with(|a| a.get())
}

#[cfg(test)]
pub(crate) fn builds() -> u64 {
    BUILDS.load(Ordering::Relaxed)
}

pub(crate) fn shader(device: &wgpu::Device, which: Shader) -> wgpu::ShaderModule {
    if !armed() {
        BUILDS.fetch_add(1, Ordering::Relaxed);
        return compile(device, which);
    }
    if let Some(m) = programs().lock().unwrap().shaders.get(&which).cloned() {
        return m;
    }
    // The lock is NOT held across the build: it would serialise every pipeline
    // construction in the process behind one mutex, and two threads racing the
    // same key simply both build and the later insert wins.
    BUILDS.fetch_add(1, Ordering::Relaxed);
    let module = compile(device, which);
    programs()
        .lock()
        .unwrap()
        .shaders
        .entry(which)
        .or_insert(module)
        .clone()
}

/// `key` names the LAYOUT, and must change whenever the descriptor does.
pub(crate) fn bind_group_layout(
    key: &'static str,
    build: impl FnOnce() -> wgpu::BindGroupLayout,
) -> wgpu::BindGroupLayout {
    if !armed() {
        BUILDS.fetch_add(1, Ordering::Relaxed);
        return build();
    }
    if let Some(l) = programs().lock().unwrap().layouts.get(key).cloned() {
        return l;
    }
    BUILDS.fetch_add(1, Ordering::Relaxed);
    let layout = build();
    programs()
        .lock()
        .unwrap()
        .layouts
        .entry(key)
        .or_insert(layout)
        .clone()
}

/// `key` names the PROGRAM, and must change whenever anything baked into the
/// `wgpu::RenderPipeline` changes — the entry point, the blend state, the
/// vertex layout. Anything a caller can later set (a color, a corner radius, a
/// dither density) is a uniform, not pipeline state, so it stays per-instance
/// and out of the key. `format` is in the key because it genuinely is part of
/// the compiled program.
pub(crate) fn render_pipeline(
    key: &'static str,
    format: wgpu::TextureFormat,
    build: impl FnOnce() -> wgpu::RenderPipeline,
) -> wgpu::RenderPipeline {
    if !armed() {
        BUILDS.fetch_add(1, Ordering::Relaxed);
        return build();
    }
    if let Some(p) = programs()
        .lock()
        .unwrap()
        .pipelines
        .get(&(key, format))
        .cloned()
    {
        return p;
    }
    BUILDS.fetch_add(1, Ordering::Relaxed);
    let pipeline = build();
    programs()
        .lock()
        .unwrap()
        .pipelines
        .entry((key, format))
        .or_insert(pipeline)
        .clone()
}
