//! The rotated LABEL pipeline: a short shaped run painted at an arbitrary
//! screen axis.
//!
//! glyphon 0.11 carries no transform anywhere. `TextArea` exposes
//! `left/top/scale/bounds/default_color/custom_glyphs`, `CustomGlyph` exposes
//! `left/top/width/height`, and neither has a rotation, a skew or a matrix — so
//! every run the document layer draws is upright, and a world that wants a
//! turned or slanted cue has no way to ask for one. This module is that
//! capability, and only that: ONE short run, ONE composed coverage mask, ONE
//! quad rotated onto a unit axis.
//!
//! It is deliberately NOT a text-transform framework and cannot become one. The
//! mask holds a single layout run ([`mask::LabelMask::compose`] reads
//! `layout_runs().next()` and stops), so there is no line breaking, no
//! per-line alignment, no wrapping and no selection — a paragraph is
//! structurally unreachable from here. The document layer stays the one prose
//! renderer; this draws labels.
//!
//! The shape is [`crate::caret_glyph`]'s, one level up: that module caches ONE
//! glyph's swash coverage in an R8 texture and paints an accent through it;
//! this caches a whole run's and paints a colour (or a gradient along the
//! baseline) through it. The rotation itself is the axis rotation
//! `shaders/caret.wgsl` already performs in its vertex stage, applied to a
//! glyph mask instead of a rounded rect — the same step
//! [`crate::selection::SelectionPipeline::prepare_rotated`] took for selection
//! quads.
//!
//! World-neutral by construction: nothing here reads a theme. A world's
//! expression — which string, which axis, which colours, where — is theme data
//! its caller supplies.

// ITEM 221 landed the first real (non-test) caller — the 90° flush-left
// secondary heading — and reaches `draw`/`prepare`/`clear`/`ink`/`matches`/
// `compose`/`label_axis_deg`/`label_bounds` directly. The rest of the surface
// (`is_drawn`, `label_local`, `label_hit`, `LabelMask::size`) is genuinely
// unused by product code today: it belongs to item 224's still-pending
// slanted Magpie expression, or (`label_hit`) to making the cue interactive,
// which nothing asks for yet. Kept as an allowance on the whole module rather
// than item-by-item so this doesn't have to be re-litigated on every partial
// landing; trim it once every item here has a real caller.
#![allow(dead_code)]

pub mod geometry;
pub mod mask;

#[cfg(test)]
mod tests;

use geometry::unit_axis;
use mask::LabelMask;

/// Per-quad instance data. MUST match the `Instance` struct layout in
/// `shaders/rotated_label.wgsl`.
#[repr(C)]
#[derive(Clone, Copy)]
struct LabelInstance {
    /// Screen pixel of the run's pen origin (left edge of the first glyph, on
    /// the baseline).
    origin: [f32; 2],
    /// Unit axis the baseline advances along.
    axis: [f32; 2],
    /// The mask's ink box in the run's own frame (min corner + size).
    ink_min: [f32; 2],
    ink_size: [f32; 2],
    /// Linear colour at the run's start and end — a gradient ALONG the
    /// baseline. Equal values give a flat label.
    color_a: [f32; 3],
    color_b: [f32; 3],
    alpha: f32,
}

/// Uniform globals. MUST match `Globals` in the WGSL.
#[repr(C)]
#[derive(Clone, Copy)]
struct Globals {
    viewport: [f32; 2],
    _pad: [f32; 2],
}

/// One rotated label: a single instanced quad sampling one composed run mask.
///
/// This is the CAPABILITY slice — the two world expressions that need it (a
/// 90° flush-left secondary heading, item 221's Cassowary Files/etc. cue; a
/// slanted location cue, item 224's still-pending Magpie expression) are
/// separately specified and separately verifiable. Every world that draws no
/// rotated text stays byte-identical: `render/layers.rs`'s
/// `prepare_rotated_location_label` parks this pipeline (`clear()`) whenever
/// `theme::LocationStyle` is not `RotatedRail`.
pub struct RotatedLabelPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    /// Rebuilt each `prepare`: the bound mask texture changes with the label.
    bind_group: Option<wgpu::BindGroup>,
    globals_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    sampler: wgpu::Sampler,
    instance_count: u32,
}

impl RotatedLabelPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = crate::gpu_cache::shader(device, crate::gpu_cache::Shader::RotatedLabel);

        let bind_group_layout = crate::gpu_cache::bind_group_layout("rotated_label", || {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("rotated label layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rotated label globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bilinear, clamped: the quad exactly covers the mask, so an upright
        // draw lands fragment centres on texel centres and the filter returns
        // the stored coverage unchanged; a turned one reconstructs between
        // them. Clamping matters only at the very edge, which the mask's own
        // transparent border already holds at zero.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rotated label sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let pipeline = build_pipeline(device, format, &shader, &bind_group_layout);

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rotated label instances"),
            size: std::mem::size_of::<LabelInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            globals_buf,
            instance_buf,
            sampler,
            instance_count: 0,
        }
    }

    /// Mark this pipeline as drawing nothing this frame.
    pub fn clear(&mut self) {
        self.instance_count = 0;
    }

    /// Whether the last prepare left a label to draw. Mirrors
    /// [`crate::caret_glyph::CaretGlyphPipeline::is_drawn`].
    pub fn is_drawn(&self) -> bool {
        self.instance_count > 0
    }

    /// Place `mask` with its pen origin at `origin`, running along `axis`.
    ///
    /// `color_a`/`color_b` are LINEAR and interpolate along the baseline, so a
    /// world whose visual language is a gradient line gets one without a second
    /// code path; pass the same value twice for a flat label.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        mask: &LabelMask,
        origin: [f32; 2],
        axis: [f32; 2],
        color_a: [f32; 3],
        color_b: [f32; 3],
        alpha: f32,
    ) {
        let globals = Globals {
            viewport: [width as f32, height as f32],
            _pad: [0.0, 0.0],
        };
        queue.write_buffer(&self.globals_buf, 0, crate::caret::bytes_of_pod(&globals));

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rotated label bind"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.globals_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(mask.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        }));

        let ink = mask.ink();
        // The axis is normalised HERE as well as in the vertex stage, so the
        // CPU-side geometry a consumer measures with (`label_bounds`,
        // `label_hit`) and the quad the GPU rasterises can never disagree about
        // which direction the run runs.
        let axis = unit_axis(axis);
        let inst = LabelInstance {
            origin,
            axis,
            ink_min: [ink[0], ink[1]],
            ink_size: [ink[2], ink[3]],
            color_a,
            color_b,
            alpha,
        };
        queue.write_buffer(&self.instance_buf, 0, crate::caret::bytes_of_pod(&inst));
        self.instance_count = 1;
    }

    /// Record the label draw.
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.instance_count == 0 {
            return;
        }
        let Some(bg) = self.bind_group.as_ref() else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.set_vertex_buffer(0, self.instance_buf.slice(..));
        pass.draw(0..6, 0..self.instance_count);
    }
}

/// Build (or reuse) the render pipeline itself. Its own function so
/// `RotatedLabelPipeline::new` stays a readable list of the objects a caller
/// actually owns — the descriptor below owns nothing and never changes.
fn build_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    shader: &wgpu::ShaderModule,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    crate::gpu_cache::render_pipeline("rotated_label", format, || {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rotated label pipeline layout"),
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LabelInstance>() as u64,
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
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 16,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 24,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 32,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 44,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 56,
                    shader_location: 6,
                },
            ],
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rotated label pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[instance_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Straight-alpha over-blend, matching the glyph-
                    // silhouette caret so the run's anti-aliased coverage
                    // composites softly onto whatever is under it.
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
                // A label turned past 180° winds the other way; without
                // this the quad would vanish at exactly the angles a
                // mirrored cue needs.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    })
}
