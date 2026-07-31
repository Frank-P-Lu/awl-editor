mod params;
mod waves;
use params::ground_params;
pub(crate) use waves::{env_phase, waves_drift_radians};

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
    /// The warped grid's finished steering pose — `[yaw, pitch, forward_cells, 0]`,
    /// resolved by `crate::warpgrid::route_pose` on the HOST so the shader carries
    /// no route arithmetic to drift out of lockstep. All zero for every other
    /// ground, on every frame, so their render is byte-identical.
    pose: [f32; 4],
    /// ITEM 186 — physical pixels per logical pixel (the display's device
    /// ratio). The shader divides every COMPOSITION quantity through this and
    /// leaves every SAMPLING quantity alone; `theme::ground`'s
    /// `Background::authored_quantities` is the table that says which is which.
    scale: f32,
    /// std140 tail padding: a uniform struct is rounded up to a multiple of its
    /// 16-byte alignment, and wgpu validates the binding against that size.
    _pad: [f32; 3],
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
    /// The ground's theme-owned profile dial (`Background::profile_mode`).
    pub profile: f32,
    /// Deckle's stable-room coordinate owner; inert off that ground.
    pub deckle_anchor: f32,
    /// WARPED GRID's tunnel-placement scalar; INERT `0.0` off that ground
    /// (`Background::tunnel_mode`), so no other world's upload changes shape.
    pub tunnel: f32,
}

/// The PER-FRAME ambient scalars the background pass carries — everything about
/// this frame that is not the world's own authored data. Grouping them is not
/// cosmetic: it keeps `prepare`'s signature honest as the shared ambient clock
/// gains consumers, and every field is `0.0` for a ground that does not read it,
/// so a static world's upload is byte-identical whatever the clock holds.
#[derive(Clone, Copy, Default)]
pub struct AmbientUpload {
    /// WAVES / ORGANIC phase drift, in radians (item 87 / item 163).
    pub drift: f32,
    /// WARPED GRID's finished steering pose — `[yaw, pitch, forward_cells]`,
    /// resolved by `crate::warpgrid::route_pose` on the host (item 132).
    pub pose: [f32; 3],
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

    /// `scale` (item 186) is the display's device ratio — PHYSICAL pixels per
    /// LOGICAL pixel, `1.0` on a 1:1 screen, `2.0` on a Retina one. It is the
    /// window's own `scale_factor` (`--capture-dpi` headlessly) and deliberately
    /// NOT the pipeline's `metrics.scale` (zoom x dpi): the ground belongs to the
    /// Room, not to the type size, and has never scaled with the user's zoom.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        col_left: f32,
        col_w: f32,
        ambient: AmbientUpload,
        scale: f32,
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            col_left,
            col_w,
            from: self.from,
            to: self.to,
            dir: self.dir,
            shader: self.shader,
            drift: ambient.drift,
            pat: self.pat,
            params: self.params,
            pose: [ambient.pose[0], ambient.pose[1], ambient.pose[2], 0.0],
            scale,
            _pad: [0.0; 3],
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
