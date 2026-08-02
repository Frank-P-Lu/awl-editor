//! GPU selection-highlight quads, drawn beneath text and caret.

const CORNER_RADIUS: f32 = 2.5;

/// Per-quad instance: a rectangle center + half-size in pixels, plus the shared
/// RGBA color. MUST match `Instance` in the WGSL.
///
/// `axis` (item 131b) is the unit rotation axis (cos, sin) the quad's vertex
/// positions are rotated onto — `(1.0, 0.0)` is upright, exactly mirroring
/// `caret.wgsl`'s own `axis` field. [`SelectionPipeline::prepare`] and
/// [`SelectionPipeline::prepare_multicolor`] both upload the inert default for
/// every instance, so every existing consumer stays byte-identical; only
/// [`SelectionPipeline::prepare_rotated`] ever uploads a real one.
#[repr(C)]
#[derive(Clone, Copy)]
struct SelInstance {
    center: [f32; 2],
    half: [f32; 2],
    color: [f32; 4],
    axis: [f32; 2],
}

/// The inert, upright rotation axis every non-rotated `SelInstance` carries.
pub(crate) const UPRIGHT_AXIS: [f32; 2] = [1.0, 0.0];

/// Uniform globals. MUST match `Globals` in the WGSL.
#[repr(C)]
#[derive(Clone, Copy)]
struct Globals {
    viewport: [f32; 2],
    corner: f32,
    dither: f32,
    stroke: f32,
    /// DITHER CELL (CHUNK round) — see `shaders/selection.wgsl`'s `Globals`:
    /// the edge, in PHYSICAL pixels, of ONE Bayer cell the dither branch snaps
    /// to before its lookup. `1.0` (the construction default) is the exact
    /// pre-chunk per-pixel stipple — byte-identical for every consumer that
    /// never raises it (the placard stipple, the always-on page frame). THE
    /// ONE WAGTAIL HIGHLIGHT TEXTURE's three consumers raise it to ~2 logical
    /// px via [`Self::set_dither_cell`]. Unused by an `fs_invert` pipeline.
    cell: f32,
    chamfer: f32,
    halftone: f32,
    halftone_angle: f32,
    halftone_cell: f32,
    /// Std140 tail padding so `dot_color` (a vec4, 16-byte aligned) lands on
    /// a 16-byte boundary — MUST match the equal-sized `_pad2: vec2<f32>` in
    /// the WGSL `Globals` (see that struct's doc for the exact byte math).
    _pad2: [f32; 2],
    /// HALFTONE dot ink (item 70), LINEAR RGBA — derived Rust-side from the
    /// theme's own surface ladder (`theme::derive::card_texture_ink`), never
    /// a raw/amber literal. `[0.0; 4]` (construction default, fully
    /// transparent) is a no-op paired with `halftone == 0.0`. (Item 71 once
    /// shared this field with a second, JAGGED-WAVE texture — Bowerbird's
    /// own woven card identity — retired outright by item 86; `dot_color`
    /// ends the struct at byte 64, already a multiple of the largest
    /// member's 16-byte alignment, so no further tail padding is needed.)
    dot_color: [f32; 4],
}

pub struct SelectionPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_cap: usize,
    instance_count: u32,
    color: [f32; 4],
    /// DITHER MODE density uploaded into `Globals::dither` each `prepare`
    /// (`0.0` = off, the pre-round behavior). Meaningless on an invert
    /// pipeline, where `fs_invert` never reads the field.
    dither: f32,
    corner: f32,
    stroke: f32,
    /// DITHER CELL edge (PHYSICAL px) uploaded into `Globals::cell` each
    /// `prepare` — the CHUNK round's Bayer-quantization block size. `1.0` (the
    /// construction default) is the pre-chunk per-pixel stipple, byte-identical
    /// for every consumer that never calls [`Self::set_dither_cell`]. THE ONE
    /// WAGTAIL HIGHLIGHT TEXTURE's three consumers raise it to ~2 logical px
    /// (`render::spans::wagtail_stipple_cell_px`, Retina-aware) so the stipple
    /// reads as deliberate dithered pixels rather than fine noise. Meaningless
    /// on an `fs_invert` pipeline.
    dither_cell: f32,
    /// Chamfer depth (px) uploaded into `Globals::chamfer` each `prepare`
    /// (item 70). `0.0` (construction default) is the ORIGINAL rounded-rect
    /// silhouette — byte-identical for every pipeline that never calls
    /// [`Self::set_chamfer`] (every world but Quokka's card family).
    chamfer: f32,
    halftone: f32,
    halftone_angle: f32,
    halftone_cell: f32,
    /// HALFTONE dot ink (LINEAR RGBA) uploaded into `Globals::dot_color`
    /// (item 70) — set via [`Self::set_halftone`], always a theme-ladder
    /// derived color (see that fn's doc). (Item 71 once shared this field
    /// with a second, JAGGED-WAVE texture via a `set_wave` sibling —
    /// Bowerbird's own woven card identity — retired outright by item 86.)
    dot_color: [f32; 4],
}

fn ordinary_blend() -> wgpu::BlendState {
    wgpu::BlendState {
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
    }
}

/// TRUE INVERSE-VIDEO's blend state: per channel, `result = (1 - dst) * src`
/// (color: `src_factor: OneMinusDst, dst_factor: Zero`) — combined with
/// `fs_invert` always writing `src = (1,1,1)`, this computes an exact
/// `result = 1 - dst`, the classic 1-bit "flip every channel" invert. The
/// alpha channel is left untouched (`src_factor: Zero, dst_factor: One`) —
/// the invert is a color-only operation; `OneMinusDst` is a standard wgpu
/// `BlendFactor` (verified against the pinned `wgpu = "=29.0.3"`,
/// `wgpu-types-29.0.3/src/render.rs`'s `BlendFactor` enum — `Dst = 6`,
/// `OneMinusDst = 7` — and it maps to `GL_ONE_MINUS_DST_COLOR`, a factor
/// WebGL2/OpenGL ES 3.0 have supported since core, so the wasm/WebGL2
/// fallback build gets the identical blend math).
fn invert_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::OneMinusDst,
            dst_factor: wgpu::BlendFactor::Zero,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    }
}

/// The one `selection.wgsl` module. `TextPipeline::new` stands up ~25
/// selection pipelines whose only real variation is blend state and fragment
/// entry point — and the entry point is a `create_render_pipeline` parameter,
/// not a module one — so a single module serves them all, and the WGSL is
/// translated to the backend's shading language once per device instead of
/// once per pipeline. Every `SelectionPipeline` constructor takes the module
/// by reference so there is no path that recompiles it. `gpu_cache` holds the
/// same rule across the whole process: the translation now happens once per
/// device rather than once per `TextPipeline`.
pub fn selection_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    crate::gpu_cache::shader(device, crate::gpu_cache::Shader::Selection)
}

impl SelectionPipeline {
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        srgba: [u8; 4],
    ) -> Self {
        Self::build(
            device,
            shader,
            format,
            srgba,
            "selection.ordinary",
            "fs_main",
            CORNER_RADIUS,
            ordinary_blend(),
        )
    }

    /// TRUE INVERSE-VIDEO SELECTION (one-bit worlds only — see
    /// `worlds.rs::WAGTAIL`'s doc comment + THEMES.md's 1-bit section for the
    /// full history of why this replaces the old "punch outline"
    /// mechanism). Built with its OWN `wgpu::RenderPipeline` object (blend
    /// state is baked in at construction, so this could not share the
    /// ordinary pipeline) using a `OneMinusDst`/`Zero` color blend —
    /// `shaders/selection.wgsl`'s `fs_invert` doc derives the exact math.
    /// Always draws pure opaque white regardless of the active theme's
    /// tokens (the blend trick needs `src == 1.0` exactly to compute a true
    /// `1 - dst`) — `set_color`/`set_dither`/`prepare_pulsed` are meaningless
    /// here and simply never called on an instance built this way. Starts
    /// with `corner = 0.0` (a hard rectangle — the right shape for a
    /// SELECTION range); a CARET-flavored instance calls [`Self::set_corner`]
    /// each frame to draw a rounded (if aliased) silhouette instead — see
    /// `shaders/selection.wgsl`'s `fs_invert` doc for the mechanism.
    pub fn new_invert(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self::build(
            device,
            shader,
            format,
            [255, 255, 255, 255],
            "selection.invert",
            "fs_invert",
            0.0,
            invert_blend(),
        )
    }

    /// The shared pipeline-construction body: every field the two public
    /// constructors differ on (fragment entry point, corner radius, blend
    /// state) is a parameter here — everything else (bind group layout,
    /// vertex buffer layout, instance buffer) is identical code, so the two
    /// pipeline "flavors" cannot drift apart by construction.
    fn build(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        srgba: [u8; 4],
        // `key` names the PROGRAM this flavor compiles to, and must change
        // whenever anything baked into the `wgpu::RenderPipeline` changes —
        // the entry point and the blend state, here. Everything else the two
        // flavors differ on (color, corner radius, dither) is a uniform, not
        // pipeline state, so it stays per-instance and out of the key.
        key: &'static str,
        entry_point: &str,
        corner: f32,
        blend: wgpu::BlendState,
    ) -> Self {
        let bind_group_layout = crate::gpu_cache::bind_group_layout("selection", || {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("selection globals layout"),
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
            })
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selection globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("selection globals bind"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let pipeline = crate::gpu_cache::render_pipeline(key, format, || {
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("selection pipeline layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

            let instance_layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<SelInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 8,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 16,
                        shader_location: 2,
                    },
                    // ITEM 131b — `axis`, at the END of the struct so every offset
                    // above is untouched (no existing attribute moves).
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 32,
                        shader_location: 3,
                    },
                ],
            };

            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("selection pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    buffers: &[instance_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
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
            })
        });

        let instance_cap = 64;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selection instances"),
            size: (instance_cap * std::mem::size_of::<SelInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group,
            globals_buf,
            instance_buf,
            instance_cap,
            instance_count: 0,
            color: srgba_u8_to_linear(srgba),
            dither: 0.0,
            corner,
            stroke: 0.0,
            dither_cell: 1.0,
            chamfer: 0.0,
            halftone: 0.0,
            halftone_angle: 0.0,
            halftone_cell: 6.0,
            dot_color: [0.0; 4],
        }
    }

    pub fn set_color(&mut self, srgba: [u8; 4]) {
        self.color = srgba_u8_to_linear(srgba);
    }

    /// Switch DITHER MODE on/off (density `0.0` = off, the ordinary soft
    /// fill — else THE ONE WAGTAIL HIGHLIGHT TEXTURE at that density). Called
    /// from `sync_theme_colors` every theme switch (a switch FROM a one-bit
    /// world must reset this back to `0.0`, not merely leave it stale).
    pub fn set_dither(&mut self, density: f32) {
        self.dither = density;
    }

    /// Set the DITHER CELL edge (PHYSICAL px) the NEXT `prepare` uploads into
    /// `Globals::cell` (CHUNK round). `1.0` restores the pre-chunk per-pixel
    /// stipple (byte-identical to a pipeline that never calls this); `> 1.0`
    /// coarsens the Bayer stipple into `cell`x`cell`-px blocks so it reads as
    /// deliberate dithered pixels. Clamped to `>= 1.0` so a stray sub-pixel
    /// value can never divide-by-zero in the shader. Fed from
    /// `render::spans::wagtail_stipple_cell_px` at THE ONE WAGTAIL HIGHLIGHT
    /// TEXTURE's three consumers (construction + every re-tint + a DPI change);
    /// meaningless (never called) on the placard/page-frame dither consumers,
    /// which keep the fine per-pixel stipple.
    pub fn set_dither_cell(&mut self, cell: f32) {
        self.dither_cell = cell.max(1.0);
    }

    /// The current DITHER CELL edge (`1.0` = the fine per-pixel stipple). A
    /// cheap headless assertion hook, mirroring [`Self::dither`] (used by the
    /// Override the rounded-rect corner radius (px) the NEXT `prepare` call
    /// uploads into `Globals::corner`. Meaningless (never called) on the
    /// ORDINARY fill pipeline (its `CORNER_RADIUS` is fixed at construction
    /// and never needs to move) or on `selection_invert` (a selection range
    /// is a rectangle, not a rounded-rect — leaving `corner` at its `0.0`
    /// construction default IS the "stay rectangular" contract). The ONE
    /// real caller is the 1-BIT CARET ROUND's `caret_invert`
    /// (`render/layers.rs::prepare_caret_block`), which passes in the SAME
    /// already-computed, already-zoom/squash-animated radius the ORDINARY
    /// (non-one-bit) caret pipeline draws with — one Rust-side owner for the
    /// number, never a second constant; see `shaders/selection.wgsl`'s
    /// `fs_invert` doc for how the shader spends it.
    pub fn set_corner(&mut self, corner: f32) {
        self.corner = corner;
    }

    /// Set the OUTLINE / STROKE width (px) the NEXT `prepare` uploads into
    /// `Globals::stroke` (V6 P5 round). `0.0` restores the SOLID fill
    /// (byte-identical to a pipeline that never calls this); `> 0.0` turns the
    /// quad into a hairline RING that wide just inside its edge — the
    /// `FacetStyle::Chips` ghost pills. Set every frame from the draw path so a
    /// mode change never leaves it stale.
    pub fn set_stroke(&mut self, stroke: f32) {
        self.stroke = stroke.max(0.0);
    }

    /// Set the CHAMFER depth (px, item 70) the NEXT `prepare` uploads into
    /// `Globals::chamfer`. `0.0` (the construction default, and the value
    /// every pipeline but Quokka's card family carries) restores the ORIGINAL
    /// rounded-rect silhouette — byte-identical. `> 0.0` cuts a crisp 45°
    /// corner that deep, replacing the rounded corner (see
    /// `shaders/selection.wgsl`'s `sd_card_rect`). Set every frame from the
    /// draw path (`render::chrome::effective_card_shape`), narrow-reduced
    /// there, never re-derived here.
    pub fn set_chamfer(&mut self, chamfer_px: f32) {
        self.chamfer = chamfer_px.max(0.0);
    }

    /// The current CHAMFER depth (`0.0` = the rounded-rect silhouette). A
    /// cheap headless assertion hook mirroring [`Self::stroke`] (used by the
    /// Set the HALFTONE dot texture (item 70) the NEXT `prepare` uploads into
    /// `Globals::halftone`/`halftone_angle`/`halftone_cell`/`dot_color`.
    /// `density <= 0.0` disables the texture entirely (`Globals::halftone`
    /// stays `0.0`, `fs_main`'s composite is skipped outright — byte-identical
    /// to a pipeline that never calls this, the default every pipeline but
    /// Quokka's card FILL carries). `ink` MUST already be a theme-ladder
    /// derived sRGBA color (`theme::derive::card_texture_ink`) — this setter
    /// does no derivation of its own, it only converts to linear (mirroring
    /// [`Self::set_color`]) and uploads what the caller computed.
    pub fn set_halftone(&mut self, density: f32, angle_rad: f32, cell_px: f32, ink: [u8; 4]) {
        self.halftone = density.clamp(0.0, 1.0);
        self.halftone_angle = angle_rad;
        self.halftone_cell = cell_px.max(1.0);
        self.dot_color = srgba_u8_to_linear(ink);
    }

    /// The current HALFTONE density ceiling (`0.0` = off). A cheap headless
    /// assertion hook mirroring [`Self::dither`] (used by the render tests;
    /// How many quad instances the last `prepare` uploaded (0 = nothing drawn). A cheap
    /// headless assertion hook for "is this summoned rect present this frame?" (used by
    /// the render tests; no non-test caller in the shipping binary).
    #[allow(dead_code)]
    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    /// The current DITHER MODE density (`0.0` = off — the ordinary alpha
    /// fill). A cheap headless assertion hook, mirroring [`Self::instance_count`]
    /// (used by the render tests; no non-test caller in the shipping binary).
    #[allow(dead_code)]
    pub fn dither(&self) -> f32 {
        self.dither
    }

    /// The current OUTLINE / STROKE width (`0.0` = solid fill). A cheap headless
    /// assertion hook for the `FacetStyle::Chips` ghost-pill law test; no non-test
    /// caller in the shipping binary.
    #[allow(dead_code)]
    pub fn stroke(&self) -> f32 {
        self.stroke
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rects: &[[f32; 4]],
    ) {
        self.prepare_with_color(device, queue, width, height, rects, self.color);
    }

    /// COPY PULSE: build instances exactly like [`Self::prepare`], but blend the
    /// STORED base `color` toward `peak_srgba` (a brighter tint in the SAME hue
    /// family — see `render::copy_pulse_peak_srgba`) by `(1.0 - settle)`. `settle`
    /// in `[0, 1]`: `1.0` draws EXACTLY the base color — byte-identical to
    /// `prepare` (the short-circuit below skips the blend arithmetic entirely, so
    /// there is no floating-point drift at rest either) — `0.0` draws fully
    /// `peak_srgba`. Never mutates the stored base `color`: a live theme switch's
    /// [`Self::set_color`] stays the single source of truth, and the very next
    /// settled frame (`settle >= 1.0`) reverts automatically with no extra
    /// bookkeeping on either side.
    // Selection upload mirrors wgpu's explicit device/queue and animation inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_pulsed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rects: &[[f32; 4]],
        peak_srgba: [u8; 4],
        settle: f32,
    ) {
        let settle = settle.clamp(0.0, 1.0);
        if settle >= 1.0 {
            self.prepare(device, queue, width, height, rects);
            return;
        }
        let peak = srgba_u8_to_linear(peak_srgba);
        let color = lerp4(peak, self.color, settle);
        self.prepare_with_color(device, queue, width, height, rects, color);
    }

    /// The shared body of [`Self::prepare`] / [`Self::prepare_pulsed`]: build +
    /// upload instances from `rects`, tinted with the given (already-linear)
    /// `color` — NOT necessarily the stored `self.color`, so the copy-pulse blend
    /// never has to mutate persistent state to draw an ephemeral frame.
    fn prepare_with_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rects: &[[f32; 4]],
        color: [f32; 4],
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            corner: self.corner,
            dither: self.dither,
            stroke: self.stroke,
            cell: self.dither_cell,
            chamfer: self.chamfer,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: [0.0; 2],
            dot_color: self.dot_color,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck_lite::bytes_of(&globals));

        let mut instances: Vec<SelInstance> = Vec::with_capacity(rects.len());
        for r in rects {
            let (x, y, w, h) = (r[0], r[1], r[2], r[3]);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            instances.push(SelInstance {
                center: [x + w * 0.5, y + h * 0.5],
                half: [w * 0.5, h * 0.5],
                color,
                axis: UPRIGHT_AXIS,
            });
        }

        self.upload_instances(device, queue, &instances);
        self.instance_count = instances.len() as u32;
    }

    /// Build instances from PER-QUAD-COLORED rectangles — each `([x,y,w,h], srgba)`
    /// its own fill (the WRITING-STREAKS heatmap, where every calendar square carries
    /// a different intensity tint off the world's value ladder). Unlike [`Self::prepare`]
    /// (one shared color for every quad) this spends the per-instance `color` field the
    /// WGSL already forwards, so no shader change is needed. Globals (corner/dither/
    /// stroke) still apply uniformly. An empty slice draws nothing.
    pub fn prepare_multicolor(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        quads: &[([f32; 4], [u8; 4])],
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            corner: self.corner,
            dither: self.dither,
            stroke: self.stroke,
            cell: self.dither_cell,
            chamfer: self.chamfer,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: [0.0; 2],
            dot_color: self.dot_color,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck_lite::bytes_of(&globals));

        let mut instances: Vec<SelInstance> = Vec::with_capacity(quads.len());
        for (r, srgba) in quads {
            let (x, y, w, h) = (r[0], r[1], r[2], r[3]);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            instances.push(SelInstance {
                center: [x + w * 0.5, y + h * 0.5],
                half: [w * 0.5, h * 0.5],
                color: srgba_u8_to_linear(*srgba),
                axis: UPRIGHT_AXIS,
            });
        }
        self.upload_instances(device, queue, &instances);
        self.instance_count = instances.len() as u32;
    }

    /// The rotated rounded-rect emitter. Every other builder
    /// (`prepare`, `prepare_multicolor`) draws axis-aligned quads; this is the
    /// one door that can draw a quad seated at an angle — the primitive a
    /// crisp diagonal spine needs, since every overlay quad pipeline is
    /// otherwise axis-aligned. `quads` is `(center, half_size, axis)` per
    /// instance, in the SAME pixel units `prepare`'s `[x, y, w, h]` uses (just
    /// centered rather than corner-anchored, since a rotated quad has no
    /// axis-aligned corner to anchor from). Globals (corner/dither/stroke/
    /// chamfer/halftone) still apply uniformly across the batch, exactly as
    /// they do for `prepare`. An `axis` of `(0.0, 0.0)` (degenerate — nothing
    /// to normalize) is not this function's job to guard against; construct
    /// instances through [`spine_segment`], which never emits one.
    ///
    /// No production caller uses it yet. The tests preserve the upright-axis
    /// equivalence and verify that a non-upright axis rotates the quad.
    #[allow(dead_code)]
    pub fn prepare_rotated(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        quads: &[([f32; 2], [f32; 2], [f32; 2])],
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            corner: self.corner,
            dither: self.dither,
            stroke: self.stroke,
            cell: self.dither_cell,
            chamfer: self.chamfer,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: [0.0; 2],
            dot_color: self.dot_color,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck_lite::bytes_of(&globals));

        let color = self.color;
        let instances: Vec<SelInstance> = quads
            .iter()
            .filter(|(_, half, _)| half[0] > 0.0 && half[1] > 0.0)
            .map(|&(center, half, axis)| SelInstance {
                center,
                half,
                color,
                axis,
            })
            .collect();
        self.upload_instances(device, queue, &instances);
        self.instance_count = instances.len() as u32;
    }

    fn upload_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[SelInstance],
    ) {
        if instances.len() > self.instance_cap {
            self.instance_cap = instances.len().next_power_of_two();
            // Size the new buffer to the FULL capacity — NOT just the current
            // contents. A later frame whose count is ≤ instance_cap but > the
            // count at grow-time would otherwise overrun this buffer (the
            // write_buffer path below never resizes). This is the fix for the
            // wgpu "Copy … would overrun the Destination buffer" validation panic.
            self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("selection instances"),
                size: (self.instance_cap * std::mem::size_of::<SelInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buf, 0, bytemuck_lite::cast_slice(instances));
        }
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instance_buf.slice(..));
        pass.draw(0..6, 0..self.instance_count);
    }
}

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

    pub fn cast_slice<T: Pod>(s: &[T]) -> &[u8] {
        unsafe { core::slice::from_raw_parts(s.as_ptr() as *const u8, core::mem::size_of_val(s)) }
    }
}

unsafe impl bytemuck_lite::Pod for SelInstance {}
unsafe impl bytemuck_lite::Pod for Globals {}

fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Convert an 8-bit sRGB RGBA quad to linear-light floats for the shader (the
/// render target is sRGB, so the GPU expects linear color it re-encodes on
/// write). Alpha is linear already.
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

mod spine;
// No non-test caller yet — item 131c is these primitives' named first
// consumer, so the re-export is unused exactly as the functions are.
#[allow(unused_imports)]
pub use spine::{narrowed_spine_corner_px, spine_segment};

#[cfg(test)]
mod tests;
