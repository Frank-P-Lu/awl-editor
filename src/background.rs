//! The MARGIN-gradient background pipeline (PAGE MODE). Draws ONE fullscreen
//! triangle BEFORE the selection / text passes: the calm page column is left
//! untouched (the fragment outputs alpha 0 there, so the base_100 clear shows)
//! and the surrounding MARGINS carry a per-world gradient, so the page reads as
//! a clean shape floating on a styled ground (N++ figure/ground).
//!
//! It mirrors [`crate::selection::SelectionPipeline`]'s structure (std140-friendly
//! globals, a tiny local bytemuck shim, the same straight-alpha over-blend) but is
//! vertex-free: the triangle is generated from `vertex_index`, so there is no
//! instance buffer — the draw call is a LITERAL `pass.draw(0..3, 0..1)` (see
//! [`BackgroundPipeline::draw`]), never scaled by document length or visible
//! content. Colors arrive as sRGB theme bytes and are converted to linear here
//! (the render target is sRGB). Almost entirely static: no per-doc/per-glyph
//! input ever reaches this pipeline. The ONE exception (item 87) is a single
//! scalar `drift` uniform — [`Background::Waves`]' very slow phase drift,
//! riding the SAME shared ambient clock the lava lamp and twinkling stars use
//! ([`crate::lava::lava_phase_for`] via `TextPipeline::waves_render_phase`) —
//! `0.0` for every world/frame that isn't an active `Waves` ground mid-tick, so
//! the headless capture (which never advances that clock) stays exactly as
//! byte-deterministic as before.

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
    /// Procedural ground discriminant: 0=gradient, 1=dots, 2=starfield,
    /// 3=pinstripe, 4=stripes, 5=bands, 6=waves, 7=zigzag (see
    /// `Background::shader_id`).
    shader: u32,
    /// WAVES phase-drift, in radians (item 87) — a DEDICATED per-frame slot
    /// occupying what was `_pad` after `shader` (offset 60; std140-safe — a
    /// scalar packed after `shader`'s scalar, so `pat`'s vec4 still lands on
    /// its 16-byte boundary at 64 and the struct stays 96 bytes). `0.0` for
    /// every non-Waves ground and every settled/headless frame, so those
    /// renders stay byte-identical to the pre-item-87 shape. Kept OFF `params`
    /// ON PURPOSE (merge reconcile with item 86): Zigzag already owns all four
    /// `params` slots — routing the drift through one of them would flatten a
    /// Zigzag world's dial to 0.0 every frame — so the drift gets its own
    /// storage here. Read by `waves_rgb` as `g.drift`.
    drift: f32,
    /// Mark/band tint (linear rgb) + its max coverage in `a`.
    pat: [f32; 4],
    /// Extra per-ground params — the SAME four slots read with different
    /// per-shader meanings (exactly one ground is ever active, so there is
    /// no real overlap): `params.x` = edge/proximity flag (0/1, Dots) OR
    /// Zigzag's `period_px` (item 86 — the two never coexist, since `edge`
    /// is always `false` off a Zigzag world and `period_px` is always `0.0`
    /// off a Dots world, so the two contributions simply SUM); `params.y` =
    /// stripe/band angle in radians (Stripes, Bands) OR Zigzag's own chevron
    /// travel angle; `params.z` = Zigzag's `amplitude_px`; `params.w` =
    /// Zigzag's `density`. Bands/Waves read `from`/`to`/`tint` above as
    /// their three authored TONES (not a gradient) — and Waves reads its
    /// phase drift from the dedicated `drift` slot above — so neither needs a
    /// `params` slot.
    params: [f32; 4],
}

/// A flat, host-side descriptor of a world's [`crate::theme::Background`] — the
/// sRGB bytes + shader discriminant + the per-ground params the pipeline needs.
/// Built from `theme::background()` in render.rs (the linear conversion happens
/// here, in [`BackgroundPipeline`]).
#[derive(Clone, Copy)]
pub struct BgDesc {
    /// Gradient START endpoint (sRGB rgba bytes).
    pub from: [u8; 4],
    /// Gradient END endpoint (sRGB rgba bytes).
    pub to: [u8; 4],
    /// Gradient direction in UV space (for Stripes: derived from the angle).
    pub dir: (f32, f32),
    /// Ground discriminant (`Background::shader_id`).
    pub shader: u32,
    /// Mark/band tint (sRGB rgb bytes; inert for a plain gradient).
    pub tint: [u8; 3],
    /// Proximity-scaling flag (Dots only).
    pub edge: bool,
    /// Stripe/Bands angle, or Zigzag's own chevron travel angle, in radians
    /// (0 for every other ground).
    pub angle: f32,
    /// Zigzag's chevron repeat wavelength ALONG its travel — and (item 89)
    /// the row-to-row spacing ACROSS it, the field tiling on a square lattice
    /// in the travel frame — device px (item 86; `0.0` for every other
    /// ground).
    pub period_px: f32,
    /// Zigzag's chevron peak excursion across its travel, device px (item 86;
    /// `0.0` for every other ground).
    pub amplitude_px: f32,
    /// Zigzag's extra coverage multiplier `[0,1]` (item 86; `0.0` for every
    /// other ground).
    pub density: f32,
}

/// The margin-gradient render pipeline: a single fullscreen triangle alpha-blended
/// over the cleared background, before selection + text.
pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    /// Linear-space gradient endpoints + direction, re-tinted on a theme switch.
    from: [f32; 4],
    to: [f32; 4],
    dir: [f32; 2],
    /// Procedural margin ground + its linear mark/band tint (re-set on a theme
    /// switch), plus the per-ground params (edge flag / stripe angle).
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
                    // Straight-alpha over-blend: the margins composite onto the
                    // base_100 clear, the page (alpha 0) leaves it untouched.
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

    /// Re-tint the gradient + ground to a new world (a live theme switch). The
    /// next `prepare` uploads it.
    pub fn set_gradient(&mut self, desc: BgDesc) {
        self.from = srgba_u8_to_linear(desc.from);
        self.to = srgba_u8_to_linear(desc.to);
        self.dir = [desc.dir.0, desc.dir.1];
        self.shader = desc.shader;
        self.pat = pattern_tint(desc.tint);
        self.params = ground_params(&desc);
    }

    /// Upload the per-frame globals: the viewport + the page column rect (in
    /// physical pixels). When page mode is OFF the caller passes `col_w == width`
    /// so the column covers the whole canvas and the margins vanish. `drift`
    /// (item 87) is the WAVES phase-drift, in radians — the caller
    /// (`TextPipeline::prepare_background_layer`) passes `0.0` for every
    /// non-Waves ground, so this is the ONLY per-frame input that can vary this
    /// pipeline's output at all, and only for `Background::Waves`.
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
/// (0/1) SUMMED with Zigzag's `period_px` (item 86 — the two grounds never
/// coexist, so this is never a real collision, just two mutually-exclusive
/// contributions sharing one slot), `y` = the Stripes/Bands angle in radians
/// OR Zigzag's own chevron travel angle, `z` = Zigzag's `amplitude_px`, `w` =
/// Zigzag's `density`. For every ground this round didn't touch, `period_px`/
/// `amplitude_px`/`density` are all `0.0`, so `x`/`z`/`w` reduce to exactly
/// their pre-round values (`edge` alone / `0.0` / `0.0`) — a byte-identical
/// render. NOTE (merge reconcile with item 87): Waves' phase drift does NOT
/// pass through here — it rides the dedicated `Globals.drift` slot uploaded
/// per-frame by [`BackgroundPipeline::prepare`], so a Zigzag world's
/// `amplitude_px` in `z` is never overwritten.
fn ground_params(desc: &BgDesc) -> [f32; 4] {
    [
        if desc.edge { 1.0 } else { 0.0 } + desc.period_px,
        desc.angle,
        desc.amplitude_px,
        desc.density,
    ]
}

// ---------------------------------------------------------------------------
// Minimal local Pod/bytemuck shim (same approach as selection.rs, no extra crate).
// ---------------------------------------------------------------------------
mod bytemuck_lite {
    /// Marker for types safe to reinterpret as bytes.
    ///
    /// # Safety
    /// Implementors must be `#[repr(C)]`, contain no padding, and consist only
    /// of plain-old-data fields (here: f32 arrays/scalars).
    pub unsafe trait Pod: Copy + 'static {}

    pub fn bytes_of<T: Pod>(t: &T) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((t as *const T) as *const u8, core::mem::size_of::<T>())
        }
    }
}

unsafe impl bytemuck_lite::Pod for Globals {}

#[cfg(test)]
mod waves_drift_tests {
    use super::*;

    /// LAW: the settled/headless-capture phase (`0.0`) drives ZERO drift — the
    /// static composition at rest is byte-identical to the pre-item-87 shape.
    /// A theme crossing INTO Bombora starts here (the shared clock's own
    /// frozen phase), never a random jump.
    #[test]
    fn drift_is_zero_at_the_settled_phase() {
        assert_eq!(waves_drift_radians(0.0), 0.0);
    }

    /// LAW: `WAVE_DRIFT_CYCLES` is an INTEGER multiple of the shared ambient
    /// clock's own loop (the twinkling-stars' "integer cycles per ambient
    /// loop" precedent, THEMES.md), so the drift's sin() argument advances by
    /// an exact multiple of TAU across one full clock loop — both boundary
    /// curves land back at their starting shape with no visible pop.
    /// NON-VACUOUS: a non-integer `WAVE_DRIFT_CYCLES` (e.g. 1.3) fails this
    /// exact assertion (verified by hand before picking the integer).
    #[test]
    fn drift_wraps_seamlessly_at_the_shared_clocks_loop_endpoint() {
        let h = 900.0;
        for x in [0.0, 137.0, 512.0, 1801.0_f32] {
            let start = waves_boundaries(x, h, waves_drift_radians(0.0));
            let end = waves_boundaries(x, h, waves_drift_radians(crate::lava::LAVA_LOOP_CYCLES));
            assert!(
                (start.0 - end.0).abs() < 1e-2,
                "b1 seamless at the wrap: {start:?} vs {end:?}"
            );
            assert!(
                (start.1 - end.1).abs() < 1e-2,
                "b2 seamless at the wrap: {start:?} vs {end:?}"
            );
        }
    }

    /// LAW: the two boundary curves never cross, at ANY drift phase (the
    /// item-69 non-overlap guarantee survives item 87's drift — the wobble
    /// amplitude is unaffected by drift, only a crest's x-position moves).
    #[test]
    fn boundaries_never_cross_at_any_drift_phase() {
        let h = 900.0;
        for step in 0..20 {
            let phase = step as f32 * crate::lava::LAVA_LOOP_CYCLES / 20.0;
            let drift = waves_drift_radians(phase);
            for x in (0..2000).step_by(97) {
                let (b1, b2) = waves_boundaries(x as f32, h, drift);
                assert!(
                    b1 < b2,
                    "tiers never cross at drift={drift}, x={x}: b1={b1} b2={b2}"
                );
            }
        }
    }

    /// LAW (the "not one sheet" proof): a SAME-SIGN drift on both boundaries
    /// would be an EXACT rigid horizontal translation of the whole field —
    /// `b1` and `b2` would both reconcile with their static (undrifted) shape
    /// under the identical coordinate shift `d/WAVE_FREQ`. This item's
    /// OPPOSITE-sign implementation shifts `b1` by `+drift` and `b2` by
    /// `-drift`: `b1` alone IS exactly that rigid shift of itself (phase is
    /// purely additive), but `b2` requires the OPPOSITE shift — so no SINGLE
    /// translation reconciles both curves simultaneously. NON-VACUOUS: with a
    /// same-sign drift (`waves_boundaries`'s `b2` using `+ drift` instead of
    /// `- drift`) this second assertion fails, because then `b2` WOULD match
    /// `b1`'s shift too (verified by hand against a same-sign variant before
    /// committing to the opposite-sign design).
    #[test]
    fn drift_is_not_a_rigid_one_sheet_translation() {
        let h = 900.0;
        let d = 0.7_f32;
        let shift = d / WAVE_FREQ;
        let (b1_d, b2_d) = waves_boundaries(123.0, h, d);
        let (b1_static_shifted, b2_static_shifted) = waves_boundaries(123.0 + shift, h, 0.0);
        assert!(
            (b1_d - b1_static_shifted).abs() < 1e-2,
            "b1 alone is a pure phase shift by d/FREQ: {b1_d} vs {b1_static_shifted}"
        );
        assert!(
            (b2_d - b2_static_shifted).abs() > 1.0,
            "b2 does NOT follow b1's shift -- the field is genuinely layered \
             (counter-moving), not one rigid sheet: {b2_d} vs {b2_static_shifted}"
        );
    }

    /// LAW: nonzero drift moves the boundaries relative to the STATIC (drift
    /// 0) shape — a non-vacuous witness that the drift term actually reaches
    /// the math (as opposed to a wiring bug that always uploads 0.0).
    #[test]
    fn nonzero_drift_actually_moves_the_boundaries() {
        let h = 900.0;
        let (b1_0, b2_0) = waves_boundaries(50.0, h, 0.0);
        let (b1_d, b2_d) = waves_boundaries(50.0, h, 1.1);
        assert!((b1_0 - b1_d).abs() > 0.5, "b1 moves under drift");
        assert!((b2_0 - b2_d).abs() > 0.5, "b2 moves under drift");
    }
}
