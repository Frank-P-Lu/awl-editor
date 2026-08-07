//! FROSTED-BACKDROP BLUR — the cached, cheap defocus behind a full-takeover overlay.
//!
//! Today a full overlay (command palette, go-to, outline, keybindings, spell) dimmed
//! the document behind it with a neutral grey scrim, which muted the theme's hues.
//! This replaces that with a real wgpu post-process: when such an overlay opens we
//! render the document ONCE to an offscreen texture, DOWNSAMPLE it to quarter
//! resolution, run a couple of separable-Gaussian ping-pong passes, and composite
//! the frosted result as the backdrop. The blur PRESERVES hue (a defocus, not a
//! desaturation — the whole point); a small dim toward the theme's OWN `base_100`
//! lets the doc recede a value without going neutral.
//!
//! "Do the effect, do it cheap" (PHILOSOPHY): the blur is precomputed + CACHED. The
//! owner ([`super::TextPipeline`]) recomputes only when the captured doc / size /
//! theme actually changes (it tracks a signature); a settled, unchanged
//! overlay-open frame just re-composites the already-blurred quarter texture, so an
//! idle overlay stays 0% CPU (DESIGN §6). It is DETERMINISTIC (no clock) — a pure
//! pixel function of the captured doc — so an overlay capture is byte-stable.
//!
//! TWO EXTENTS, ONE EFFECT. A full-takeover overlay frosts the WHOLE canvas; a CRISP
//! picker frosts only ITS OWN FOOTPRINT — a feathered shape that LEANS with the
//! composition drawn inside it. Which, why, the shape's arithmetic and the uniform that
//! carries it all live in [`extent`]; this file is the GPU plumbing both arms share.

mod extent;

pub use extent::{BlurSurface, Footprint, Frost, footprint_box, footprint_frost_applies};
use extent::{
    DOWNSAMPLE, U, bytes_of, capped_doc_size, downsample_for, footprint_bound,
    footprint_feather_px, scissor_px,
};
/// The authored feather width, the reach it gives the skirt, and the SHIPPING mask itself,
/// re-exported for the laws that grade the drawn edge against them — so a render-tier
/// measurement cannot compare the pixels to a retyped number, or to a second copy of the
/// shape's arithmetic, that has drifted from what the shader was handed.
#[cfg(test)]
pub(crate) use extent::{FOOTPRINT_FEATHER_PX, footprint_mask_for, footprint_skirt_px};

use wgpu::util::DeviceExt;

/// Number of separable-Gaussian ping-pong ROUNDS (each round = one horizontal + one
/// vertical 9-tap pass). Two rounds on the quarter-res target read as a soft frost
/// without smearing the hues into mud.
const BLUR_ROUNDS: u32 = 2;

/// THE COMPOSITE TARGET'S BLEND. The downsample and Gaussian passes each overwrite
/// their whole target and stay unblended; the composite carries the footprint's
/// feathered coverage in its ALPHA, so it must blend.
///
/// [`Frost::Full`] needs no second pipeline: its mask is exactly `1.0` at every pixel,
/// and at `srcA == 1` this equation is `src * 1 + dst * 0` — the replace `blend: None`
/// performed. That is asserted rather than assumed
/// (`the_full_frosts_composite_is_destination_independent`).
const COMP_BLEND: Option<wgpu::BlendState> = Some(wgpu::BlendState::ALPHA_BLENDING);

/// The frosted-backdrop post-process: three fragment pipelines (downsample / blur /
/// composite) over a shared fullscreen-triangle vertex + bind group, plus the
/// lazily-sized offscreen textures they ping-pong through.
pub struct BlurBackdrop {
    down_pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    comp_pipeline: wgpu::RenderPipeline,
    bind_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// The target texture format (the surface / offscreen format the pipelines were
    /// built for); the lazily-created textures must match it.
    format: wgpu::TextureFormat,
    /// The size the current textures + bind groups were built for; `None` until the
    /// first [`Self::ensure`].
    size: Option<(u32, u32)>,
    /// The downsample factor the current textures were built for ([`downsample_for`]).
    /// Part of the recreate key alongside `size`, because a DPI change alters the
    /// quarter-res working size without altering the surface's.
    ds: u32,
    /// WHERE this frame's composite lands ([`Frost`]). Set by [`Self::ensure`] and read
    /// by [`Self::draw_backdrop`], so the extent that was sized and dimmed is the
    /// extent that gets drawn. `Full` until the first `ensure` — the historical
    /// fullscreen composite is the safe default for a caller that never scopes.
    frost: Frost,
    /// The footprint feather in PHYSICAL px, resolved at [`Self::ensure`] from the
    /// surface's DPI. Stored beside `frost` because [`Self::draw_backdrop`] must bound
    /// the SAME feathered shape the uniform describes, and it has no DPI of its own.
    feather: f32,
    /// The captured document (full-res render target + sample source).
    doc: Option<wgpu::Texture>,
    doc_view: Option<wgpu::TextureView>,
    /// Quarter-res ping-pong pair.
    qa_view: Option<wgpu::TextureView>,
    qb_view: Option<wgpu::TextureView>,
    /// Per-pass uniform buffers (one value each — a single uniform can't carry
    /// distinct per-pass values within one encoder submit, so each pass owns its own).
    u_down: Option<wgpu::Buffer>,
    u_blur_h: Option<wgpu::Buffer>,
    u_blur_v: Option<wgpu::Buffer>,
    u_comp: Option<wgpu::Buffer>,
    /// Per-source bind groups: down samples the doc, the H passes sample `qa`, the V
    /// passes sample `qb`, the composite samples the final `qa`.
    bg_down: Option<wgpu::BindGroup>,
    bg_h: Option<wgpu::BindGroup>,
    bg_v: Option<wgpu::BindGroup>,
    bg_comp: Option<wgpu::BindGroup>,
}

impl BlurBackdrop {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = crate::gpu_cache::shader(device, crate::gpu_cache::Shader::Blur);
        let bind_layout = crate::gpu_cache::bind_group_layout("blur", || {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blur bind layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
        // All three pipelines share the vertex + layout + (opaque) target format and
        // differ only in the fragment entry point — so `key` is what varies, and
        // it varies with exactly that. Everything a pass writes to (its uniform
        // buffer, its bind group) is built per instance below, outside the cache.
        let mk = |key: &'static str, entry: &str, label: &str, blend| {
            crate::gpu_cache::render_pipeline(key, format, || {
                let pipeline_layout =
                    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("blur pipeline layout"),
                        bind_group_layouts: &[Some(&bind_layout)],
                        immediate_size: 0,
                    });
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some(entry),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend,
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
            })
        };
        let down_pipeline = mk("blur.down", "fs_down", "blur downsample pipeline", None);
        let blur_pipeline = mk("blur.blur", "fs_blur", "blur gaussian pipeline", None);
        let comp_pipeline = mk("blur.comp", "fs_comp", "blur composite", COMP_BLEND);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("blur sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            down_pipeline,
            blur_pipeline,
            comp_pipeline,
            bind_layout,
            sampler,
            format,
            size: None,
            ds: DOWNSAMPLE,
            frost: Frost::Full,
            feather: footprint_feather_px(1.0),
            doc: None,
            doc_view: None,
            qa_view: None,
            qb_view: None,
            u_down: None,
            u_blur_h: None,
            u_blur_v: None,
            u_comp: None,
            bg_down: None,
            bg_h: None,
            bg_v: None,
            bg_comp: None,
        }
    }

    /// (Re)build the textures + bind groups for `width`×`height` at `dpi` and refresh
    /// the per-pass uniforms (sample steps + the composite `tint` toward base_100,
    /// `base100_linear`, dimmed by `frost`'s own amount). Returns `true` when the
    /// textures were RECREATED (a fresh / resized / re-scaled target), so the caller
    /// must force a recompute (the cached blur is gone). An unchanged-geometry call
    /// only re-uploads the uniforms and returns `false`.
    ///
    /// `frost` is stored for [`Self::draw_backdrop`] — the extent that was dimmed here
    /// is the extent drawn there, so a footprint can never be composited with the
    /// full-takeover dim or vice versa.
    pub fn ensure(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: BlurSurface,
        base100_linear: [f32; 3],
        frost: Frost,
    ) -> bool {
        let BlurSurface { width, height, dpi } = surface;
        self.frost = frost;
        self.feather = footprint_feather_px(dpi);
        let ds = downsample_for(dpi);
        // THE RECREATE KEY IS (size, downsample), not size alone: a DPI change moves
        // the quarter-res working size (and the blur's reach with it) while the
        // surface's own size can stay put — a display move between a 1× and a 2×
        // screen at the same logical size does exactly that.
        let recreated = self.size != Some((width, height)) || self.ds != ds;
        if recreated {
            // DROP-BEFORE-ALLOCATE: release the PREVIOUS textures/views/bind-groups
            // (set them to `None`) BEFORE creating the new ones, so a resize never has
            // the old AND new doc/qa/qb sets live at the same instant — that transient
            // double is the resize VRAM peak. The size-guard above means we only reach
            // here on a GENUINE size change, so the final resources are identical to the
            // un-dropped path; only the momentary doubling is gone.
            self.bg_down = None;
            self.bg_h = None;
            self.bg_v = None;
            self.bg_comp = None;
            self.doc = None;
            self.doc_view = None;
            self.qa_view = None;
            self.qb_view = None;
            self.u_down = None;
            self.u_blur_h = None;
            self.u_blur_v = None;
            self.u_comp = None;

            let format = self.format;
            let qw = (width / ds).max(1);
            let qh = (height / ds).max(1);
            // Cap the full-res doc capture on very-large/high-DPI surfaces (no-op at or
            // below the cap → byte-identical); a smaller target scales the whole document
            // down to fill it (see `capped_doc_size`).
            let (cw, ch) = capped_doc_size(width, height, ds);
            let mk_tex = |label: &str, w: u32, h: u32| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
            };
            let doc = mk_tex("blur doc", cw, ch);
            let qa = mk_tex("blur qa", qw, qh);
            let qb = mk_tex("blur qb", qw, qh);
            let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
            let doc_view = v(&doc);
            let qa_view = v(&qa);
            let qb_view = v(&qb);

            // Sample steps: the downsample reads the full-res doc (its 4-tap box reads
            // ONE texel of the possibly-capped doc, so its step is the capped doc's
            // texel size, not the surface's); each blur axis reads the quarter-res
            // texel along one direction.
            let u_down = self.mk_uniform(
                device,
                U::pass([1.0 / cw as f32, 1.0 / ch as f32, 0.0, 0.0]),
            );
            let u_blur_h = self.mk_uniform(device, U::pass([1.0 / qw as f32, 0.0, 0.0, 0.0]));
            let u_blur_v = self.mk_uniform(device, U::pass([0.0, 1.0 / qh as f32, 0.0, 0.0]));
            let u_comp = self.mk_uniform(device, U::comp(base100_linear, frost, dpi));

            self.bg_down = Some(self.mk_bind(device, &u_down, &doc_view));
            self.bg_h = Some(self.mk_bind(device, &u_blur_h, &qa_view));
            self.bg_v = Some(self.mk_bind(device, &u_blur_v, &qb_view));
            self.bg_comp = Some(self.mk_bind(device, &u_comp, &qa_view));

            self.doc = Some(doc);
            self.doc_view = Some(doc_view);
            self.qa_view = Some(qa_view);
            self.qb_view = Some(qb_view);
            self.u_down = Some(u_down);
            self.u_blur_h = Some(u_blur_h);
            self.u_blur_v = Some(u_blur_v);
            self.u_comp = Some(u_comp);
            self.size = Some((width, height));
            self.ds = ds;
        }
        // Refresh the composite's tint AND its extent each call (cheap) so a theme
        // change, a switch between the two frost arms — which carry different dims —
        // or a card that merely MOVED lands without a texture rebuild.
        if let Some(buf) = &self.u_comp {
            queue.write_buffer(buf, 0, bytes_of(&U::comp(base100_linear, frost, dpi)));
        }
        recreated
    }

    fn mk_uniform(&self, device: &wgpu::Device, u: U) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("blur uniform"),
            contents: bytes_of(&u),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn mk_bind(
        &self,
        device: &wgpu::Device,
        uniform: &wgpu::Buffer,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blur bind"),
            layout: &self.bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// The full-res render target the caller draws the document into (so it can be
    /// captured, then blurred). `None` before the first [`Self::ensure`].
    pub fn doc_view(&self) -> Option<&wgpu::TextureView> {
        self.doc_view.as_ref()
    }

    /// Run the downsample + the separable-Gaussian ping-pong passes into the
    /// quarter-res pair, leaving the FINAL blurred result in `qa` (which
    /// [`Self::draw_backdrop`] composites). Each pass is its own render pass on the
    /// shared encoder, so wgpu inserts the read-after-write barriers between them.
    /// The doc texture must already be drawn (an earlier pass on `doc_view`).
    pub fn encode_blur(&self, encoder: &mut wgpu::CommandEncoder) {
        let (Some(qa), Some(qb), Some(bg_down), Some(bg_h), Some(bg_v)) = (
            &self.qa_view,
            &self.qb_view,
            &self.bg_down,
            &self.bg_h,
            &self.bg_v,
        ) else {
            return;
        };
        // 1) downsample doc -> qa.
        self.pass(encoder, &self.down_pipeline, bg_down, qa);
        // 2) BLUR_ROUNDS of (H: qa -> qb, V: qb -> qa). bg_h samples qa, bg_v samples
        //    qb, so the same two bind groups serve every round.
        for _ in 0..BLUR_ROUNDS {
            self.pass(encoder, &self.blur_pipeline, bg_h, qb);
            self.pass(encoder, &self.blur_pipeline, bg_v, qa);
        }
    }

    /// One fullscreen-triangle pass: clear the target (the tri overwrites every
    /// pixel, so the clear is just a defined load) and draw with `pipeline` + `bind`.
    fn pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        bind: &wgpu::BindGroup,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blur pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Composite the cached frosted backdrop (the final blurred `qa`, upsampled +
    /// dimmed toward base_100) into an already-open render pass, over the EXTENT
    /// [`Self::ensure`] was handed. A no-op until `ensure` has run.
    ///
    /// [`Frost::Full`] draws the bare fullscreen triangle it always did, at an alpha of
    /// exactly 1.0 — a replace under [`COMP_BLEND`]. A [`Frost::Footprint`] draws the
    /// same triangle, shaped by the feathered mask in `fs_comp`, and SCISSORED to
    /// [`footprint_bound`]: outside that box the mask is provably zero, so the scissor
    /// changes nothing visible and buys the pass keeping whatever was already drawn
    /// there — the crisp document and the world's live ground — bit-for-bit, on every
    /// backend, rather than through an sRGB blend round-trip against a zero alpha.
    ///
    /// The scissor is RESET to the whole target before returning: it is pass state, not
    /// draw state, and the card, its text and the whole chrome tail are drawn into this
    /// same pass afterwards. Forgetting the reset clips the card's own rows to the frost.
    pub fn draw_backdrop<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let Some(bg) = &self.bg_comp else {
            return;
        };
        // `size` is `Some` whenever `bg_comp` is (both are set together by `ensure`).
        let (width, height) = self.size.unwrap_or((0, 0));
        let scissor = match self.frost {
            Frost::Full => None,
            Frost::Footprint(foot) => {
                match scissor_px(footprint_bound(foot, self.feather), width, height) {
                    Some(s) => Some(s),
                    // A footprint entirely off-canvas frosts nothing at all — drawing
                    // the unscissored triangle instead would frost the WHOLE page.
                    None => return,
                }
            }
        };
        if let Some((x, y, w, h)) = scissor {
            pass.set_scissor_rect(x, y, w, h);
        }
        pass.set_pipeline(&self.comp_pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.draw(0..3, 0..1);
        if scissor.is_some() {
            pass.set_scissor_rect(0, 0, width, height);
        }
    }
}

#[cfg(test)]
mod tests;
