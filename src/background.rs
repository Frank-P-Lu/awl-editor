/// Uniform globals. MUST match `Globals` in `shaders/background.wgsl`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Globals {
    viewport: [f32; 2],
    col_left: f32,
    col_w: f32,
    from: [f32; 4],
    to: [f32; 4],
    dir: [f32; 2],
    shader: u32,
    /// Waves phase drift. Its dedicated std140 slot keeps Zigzag's parameter slots intact.
    drift: f32,
    pat: [f32; 4],
    params: [f32; 4],
}

/// A flat, host-side descriptor of a world's [`crate::theme::Background`] — the
/// sRGB bytes + shader discriminant + the per-ground params the pipeline needs.
/// Built from `theme::background()` in render.rs (the linear conversion happens
/// here, in [`BackgroundPipeline`]).
#[derive(Clone, Copy)]
pub struct BgDesc {
    pub from: [u8; 4],
    pub to: [u8; 4],
    pub dir: (f32, f32),
    pub shader: u32,
    pub tint: [u8; 3],
    pub edge: bool,
    pub angle: f32,
    pub period_px: f32,
    pub amplitude_px: f32,
    pub density: f32,
    pub banded: bool,
}

/// The margin-gradient render pipeline: a single fullscreen triangle alpha-blended
/// over the cleared background, before selection + text.
pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    from: [f32; 4],
    to: [f32; 4],
    dir: [f32; 2],
    shader: u32,
    pat: [f32; 4],
    params: [f32; 4],
}

/// Max coverage the margin pattern's marks reach (the shader multiplies the
/// per-pixel coverage by this). Kept low so the dots / stars / stripes whisper
/// and the page column stays the clear figure.
const PATTERN_MAX_COVERAGE: f32 = 0.55;

// --- ITEM 87: WAVES PHASE DRIFT (the shared ambient clock's third consumer) ---
//
// Bombora's wave-tier boundaries ride a single scalar DRIFT (radians),
// uploaded through the DEDICATED `Globals.drift` slot (NOT `params` — item 86's
// Zigzag owns all four `params` slots; see that field's doc). The two boundary
// curves drift with EQUAL MAGNITUDE and OPPOSITE SIGN: the top/middle
// boundary advances by `+drift`, the middle/bottom boundary by `-drift`. A
// SAME-SIGN drift on both curves is mathematically an EXACT rigid horizontal
// translation of the whole three-tier field (`sin(x*F+P+d)` for both curves
// is identical to evaluating the undrifted field at `x + d/F` — the field's
// shape literally never changes, only its position) — a "one sheet" slide
// where every tier, middle included, shares IDENTICAL motion, precisely the
// outcome item 87 asks NOT to produce. The opposite-sign choice is the only
// one that breaks that rigid-translation identity: each OUTER tier (top,
// bottom) is bounded by exactly one of the two curves and sweeps with that
// curve's own sign, while the MIDDLE tier — bounded by BOTH, one advancing
// and one retarding — visibly shears/breathes counter to them, so the sea
// reads as independently layered swells rather than a sheet sliding behind
// the margin. `WAVE_DRIFT_CYCLES` is an INTEGER (the twinkling-stars'
// "integer cycles per ambient loop" law, THEMES.md's ambient-stars section):
// the drift completes an EXACT number of full turns over one shared-clock
// loop (`crate::lava::LAVA_LOOP_CYCLES`), so it meets its own endpoint
// exactly where the clock wraps — seamless, no pop. `1.0` is the slowest
// non-zero integer choice (one full 2*pi sweep — one WAVE wavelength of
// crest travel — over the ~67s loop), matching "very slow, almost
// imperceptible." Pure; MUST match `shaders/background.wgsl`'s own `drift`
// read off `g.drift` and its `waves_rgb`'s
// `WAVE_AMP`/`WAVE_FREQ`/`WAVE_PHASE_1`/`WAVE_PHASE_2`.
//
// ITEM 89 SCOPE NOTE: the four SHAPE constants below, and the
// [`waves_boundaries`] mirror that reads them, exist ONLY so those tier-geometry
// laws can be unit-tested without a GPU — the shipping renderer reads the
// WGSL's own copies (`shaders/background.wgsl`'s `waves_rgb`), never these. They
// are therefore `#[cfg(test)]`-gated and module-PRIVATE (no cross-module test
// calls them; `render/tests/backgrounds_item69.rs` only mentions `WAVE_AMP` in a
// comment) rather than carrying an `allow(dead_code)` that would let a genuinely
// dead future constant hide here. `WAVE_DRIFT_CYCLES` stays ungated: it feeds
// the RUNTIME [`waves_drift_radians`].
#[cfg(test)]
const WAVE_AMP: f32 = 22.0;
#[cfg(test)]
const WAVE_FREQ: f32 = 0.024166097;
#[cfg(test)]
const WAVE_PHASE_1: f32 = 0.0;
#[cfg(test)]
const WAVE_PHASE_2: f32 = 2.4;
const WAVE_DRIFT_CYCLES: f32 = 1.0;

/// The WAVES drift, in radians, for the shared ambient `phase` (cycles,
/// `[0, LAVA_LOOP_CYCLES)`) — `0.0` at `phase == 0.0` (the frozen/settled/
/// headless-capture phase, so a theme crossing INTO Bombora, and every
/// headless capture, renders the EXACT pre-item-87 static composition). Pure.
/// See the module doc above for the seamless-wrap derivation.
pub fn waves_drift_radians(phase: f32) -> f32 {
    phase * std::f32::consts::TAU * WAVE_DRIFT_CYCLES / crate::lava::LAVA_LOOP_CYCLES
}

// The dev-only gallery knob (AWL_WAVES_PHASE=<f32>): mirrors `AWL_LAVA`/
// `AWL_STARS_PHASE` exactly (read once, memoized, a total no-op unless set —
// a headless capture never ticks the clock, so this never touches
// determinism there). Drives BOTH consumers of `waves_render_phase` — Bombora's
// wave drift AND Bowerbird's organic drift (item 163) — one shared clock, one
// knob. Lets a gallery/before-after shot reach a real mid-drift composition.
fn parse_waves_phase(raw: &str) -> Option<f32> {
    let p: f32 = raw.trim().parse().ok()?;
    p.is_finite().then_some(p)
}

/// `AWL_WAVES_PHASE`'s parsed value, or `None` (every normal + headless run).
/// Consumed by `TextPipeline::waves_render_phase` (env wins outright).
pub fn env_phase() -> Option<f32> {
    static ONCE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_WAVES_PHASE")
            .ok()
            .as_deref()
            .and_then(parse_waves_phase)
    })
}

/// The Rust MIRROR of `shaders/background.wgsl`'s `waves_rgb` boundary math —
/// the top/middle boundary `b1` (top third of the viewport height, plus the
/// scallop sine, phase-ADVANCED by `drift`) and the middle/bottom boundary
/// `b2` (bottom third, phase-RETARDED by `drift` — the opposite sign).
/// `viewport_h` in px; returns `(b1, b2)` in px. MUST stay in lockstep with
/// the shader; unit-tested here without a GPU (the `lava.rs`/`dither.rs`
/// shader-mirror idiom). TEST-ONLY and module-private (item 89) — the runtime
/// path reads the WGSL's own copy of this math; see the scope note above the
/// `WAVE_*` constants.
#[cfg(test)]
fn waves_boundaries(x: f32, viewport_h: f32, drift: f32) -> (f32, f32) {
    let b1 = viewport_h * (1.0 / 3.0) + WAVE_AMP * (x * WAVE_FREQ + WAVE_PHASE_1 + drift).sin();
    let b2 = viewport_h * (2.0 / 3.0) + WAVE_AMP * (x * WAVE_FREQ + WAVE_PHASE_2 - drift).sin();
    (b1, b2)
}

impl BackgroundPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, desc: BgDesc) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("background shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/background.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("background globals layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("background globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("background globals bind"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("background pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("background pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            globals_buf,
            from: srgba_u8_to_linear(desc.from),
            to: srgba_u8_to_linear(desc.to),
            dir: [desc.dir.0, desc.dir.1],
            shader: desc.shader,
            pat: pattern_tint(desc.tint),
            params: ground_params(&desc),
        }
    }

    pub fn set_gradient(&mut self, desc: BgDesc) {
        self.from = srgba_u8_to_linear(desc.from);
        self.to = srgba_u8_to_linear(desc.to);
        self.dir = [desc.dir.0, desc.dir.1];
        self.shader = desc.shader;
        self.pat = pattern_tint(desc.tint);
        self.params = ground_params(&desc);
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        col_left: f32,
        col_w: f32,
        drift: f32,
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            col_left,
            col_w,
            from: self.from,
            to: self.to,
            dir: self.dir,
            shader: self.shader,
            drift,
            pat: self.pat,
            params: self.params,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck_lite::bytes_of(&globals));
    }

    /// Record the fullscreen-triangle draw into an open render pass, FIRST (right
    /// after the clear, before selection + text).
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Convert an 8-bit sRGB RGBA quad to linear-light floats for the shader (the
/// render target is sRGB, so the GPU expects linear color it re-encodes on
/// write). Alpha is linear already. Same as the selection pipeline's converter.
fn srgba_u8_to_linear(c: [u8; 4]) -> [f32; 4] {
    fn ch(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    [ch(c[0]), ch(c[1]), ch(c[2]), c[3] as f32 / 255.0]
}

/// Convert an opaque 8-bit sRGB pattern tint to linear rgb + bake the max
/// coverage into `a` (the shader multiplies its per-pixel coverage by this).
fn pattern_tint(c: [u8; 3]) -> [f32; 4] {
    let lin = srgba_u8_to_linear([c[0], c[1], c[2], 0xFF]);
    [lin[0], lin[1], lin[2], PATTERN_MAX_COVERAGE]
}

/// Pack the per-ground params the shader reads: `x` = the Dots proximity flag
/// Pack the mutually exclusive ground controls into the shared parameter slots.
fn ground_params(desc: &BgDesc) -> [f32; 4] {
    if desc.shader == 8 {
        return [desc.period_px, desc.density, 0.0, 0.0];
    }
    [
        if desc.edge { 1.0 } else { 0.0 } + desc.period_px,
        desc.angle,
        desc.amplitude_px,
        if desc.banded {
            -desc.density
        } else {
            desc.density
        },
    ]
}
// ---------------------------------------------------------------------------
// Minimal local Pod/bytemuck shim (same approach as selection.rs, no extra crate).
// ---------------------------------------------------------------------------
mod bytemuck_lite {
    /// Marker for types safe to reinterpret as bytes.
    ///
    /// # Safety
    /// Implementors must have a stable layout with no padding and only
    /// plain-old-data fields.
    pub unsafe trait Pod: Copy + 'static {}

    pub fn bytes_of<T: Pod>(t: &T) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((t as *const T) as *const u8, core::mem::size_of::<T>())
        }
    }
}

unsafe impl bytemuck_lite::Pod for Globals {}

#[cfg(test)]
mod tests;
