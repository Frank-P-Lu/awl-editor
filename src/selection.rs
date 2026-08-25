//! GPU selection-highlight quads, drawn beneath text and caret.

const CORNER_RADIUS: f32 = 2.5;

mod material;
mod pipeline;
use pipeline::{Flavor, build_render_pipeline};

/// Per-quad instance: a rectangle center + half-size in pixels, plus the shared
/// RGBA color. MUST match `Instance` in the WGSL.
///
/// `axis` is the unit rotation axis (cos, sin) the quad's vertex
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
    /// px via [`Self::set_dither_cell`]. Unused by an `fs_two_colour` pipeline.
    cell: f32,
    /// TOP-half / BOTTOM-half chamfer depth (px), split so a docked seam
    /// edge (top) can stay square while the free edge (bottom) keeps the
    /// cut — see the WGSL `Globals::chamfer_top`/`chamfer_bottom` doc.
    chamfer_top: f32,
    chamfer_bottom: f32,
    halftone: f32,
    halftone_angle: f32,
    halftone_cell: f32,
    /// Std140 tail padding so `dot_color` (a vec4, 16-byte aligned) lands on
    /// a 16-byte boundary — MUST match the equal-sized `_pad2: f32` in
    /// the WGSL `Globals` (see that struct's doc for the exact byte math).
    _pad2: f32,
    /// HALFTONE dot ink, LINEAR RGBA, derived Rust-side from the theme's own
    /// surface ladder (`theme::derive::card_texture_ink`), never a raw/amber
    /// literal. Fully transparent is a no-op paired with `halftone == 0.0`.
    dot_color: [f32; 4],
    /// The additive second source for a two-colour swap. Zero on ordinary
    /// pipelines; see [`SelectionPipeline::set_two_colour`].
    second_color: [f32; 4],
}

/// std140 offers no automatic padding on the Rust side — this struct hand-fills
/// every gap (see `_pad2`'s doc). A field added or reordered without recomputing
/// the WGSL-side offsets corrupts every field after it silently (same SIZE,
/// different bytes at each field); this catches a SIZE drift immediately, which
/// a same-size reorder still would not, so the Quokka/Cassowary byte-identity
/// captures remain the real oracle for field-order mistakes.
const _: () = assert!(std::mem::size_of::<Globals>() == 80);

pub struct SelectionPipeline {
    pipeline: wgpu::RenderPipeline,
    second_pipeline: Option<wgpu::RenderPipeline>,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_cap: usize,
    instance_count: u32,
    color: [f32; 4],
    /// DITHER MODE density uploaded into `Globals::dither` each `prepare`
    /// (`0.0` = off, the pre-round behavior). Meaningless on an invert
    /// pipeline, where `fs_two_colour` never reads the field.
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
    /// on an `fs_two_colour` pipeline.
    dither_cell: f32,
    /// TOP-half / BOTTOM-half chamfer depth (px) uploaded into
    /// `Globals::chamfer_top`/`chamfer_bottom` each `prepare`. `0.0` on a half
    /// (the construction default) is that half's ORIGINAL rounded-rect
    /// silhouette — byte-identical for every pipeline that never calls
    /// [`Self::set_chamfer`] (every world but Quokka/Cassowary's card family).
    chamfer_top: f32,
    chamfer_bottom: f32,
    halftone: f32,
    halftone_angle: f32,
    halftone_cell: f32,
    /// HALFTONE dot ink (LINEAR RGBA) uploaded into `Globals::dot_color`
    /// Set via [`Self::set_halftone`], always a theme-ladder-derived color
    /// (see that fn's doc). The former JAGGED-WAVE texture and its `set_wave`
    /// sibling are retired.
    dot_color: [f32; 4],
    second_color: [f32; 4],
}

/// The one `selection.wgsl` module. `TextPipeline::new` stands up ~25
/// selection pipelines whose only real variation is blend state and fragment
/// entry point — and the entry point is a `create_render_pipeline` parameter,
/// not a module one — so a single module serves them all, and the WGSL is
/// translated to the backend's shading language once per device instead of
/// once per pipeline. Every `SelectionPipeline` constructor takes the module
/// by reference so there is no path that recompiles it, and `gpu_cache` holds
/// that same rule across the process rather than per `TextPipeline`.
pub fn selection_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    crate::gpu_cache::shader(device, crate::gpu_cache::Shader::Selection)
}

/// What the two flavors differ on that is BAKED INTO the compiled program, so
/// `key` (what `gpu_cache` stores it under) must change whenever the other two
/// do. Color, corner radius and dither are uniforms, not pipeline state.
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
            Flavor::ordinary(),
            CORNER_RADIUS,
        )
    }

    /// Arbitrary TWO-COLOUR inverse treatment. Its subtractive blend is baked
    /// at construction; [`Self::set_two_colour`] supplies resolved palette
    /// endpoints without a black/white assumption. Starts with `corner = 0.0`
    /// (a hard rectangle for a SELECTION range); a CARET instance calls [`Self::set_corner`]
    /// each frame to draw a rounded (if aliased) silhouette instead — see
    /// `shaders/selection.wgsl`'s `fs_two_colour` doc for the mechanism.
    pub fn new_two_colour(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        ground: [u8; 4],
        ink: [u8; 4],
    ) -> Self {
        let mut pipeline = Self::build(device, shader, format, ground, Flavor::two_colour(), 0.0);
        pipeline.set_two_colour(ground, ink);
        pipeline
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
        flavor: Flavor,
        corner: f32,
    ) -> Self {
        let Flavor {
            key,
            entry_point,
            blend,
            second,
        } = flavor;
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

        let pipeline = build_render_pipeline(
            device,
            shader,
            format,
            &bind_group_layout,
            key,
            entry_point,
            blend,
        );
        let second_pipeline = second.map(|second| {
            build_render_pipeline(
                device,
                shader,
                format,
                &bind_group_layout,
                second.key,
                second.entry_point,
                second.blend,
            )
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
            second_pipeline,
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
            chamfer_top: 0.0,
            chamfer_bottom: 0.0,
            halftone: 0.0,
            halftone_angle: 0.0,
            halftone_cell: 6.0,
            dot_color: [0.0; 4],
            second_color: [0.0; 4],
        }
    }

    pub fn set_color(&mut self, srgba: [u8; 4]) {
        self.color = srgba_u8_to_linear(srgba);
    }

    /// Resolve authored palette roles into fixed-point-safe swap passes. The
    /// first subtracts the destination from the per-channel maximum, then the
    /// second adds the minimum: `max - dst + min == ground + ink - dst`.
    pub fn set_two_colour(&mut self, ground: [u8; 4], ink: [u8; 4]) {
        let ground = srgba_u8_to_linear(ground);
        let ink = srgba_u8_to_linear(ink);
        self.color = [
            ground[0].max(ink[0]),
            ground[1].max(ink[1]),
            ground[2].max(ink[2]),
            1.0,
        ];
        self.second_color = [
            ground[0].min(ink[0]),
            ground[1].min(ink[1]),
            ground[2].min(ink[2]),
            1.0,
        ];
    }

    /// The colour this pipeline is currently carrying, in the same linear form
    /// [`Self::set_color`] stores. Test-only, and it exists because a headless
    /// capture builds its pipelines ONCE and never runs `sync_theme_colors`: a
    /// token mis-routed in the SYNC half alone repaints nothing a capture can
    /// see, and only surfaces after a LIVE theme switch. This is the seam a law
    /// reads instead of the pixels.
    #[cfg(test)]
    pub(crate) fn test_color(&self) -> [f32; 4] {
        self.color
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
    /// `fs_two_colour` doc for how the shader spends it.
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

    /// Set the TOP-half / BOTTOM-half chamfer depth in px that the NEXT
    /// `prepare` uploads into `Globals::chamfer_top`/`chamfer_bottom`. `0.0`
    /// on a half (the construction default, and the value every pipeline but
    /// Quokka/Cassowary's card family carries) restores that half's ORIGINAL
    /// rounded-rect silhouette — byte-identical. `> 0.0` cuts a crisp 45°
    /// corner that deep on that half, replacing the rounded corner (see
    /// `shaders/selection.wgsl`'s `sd_card_rect`). Set every frame from the
    /// draw path (`render::chrome::TextPipeline::card_shape_texture`, the
    /// single corner-mask owner every console layer resolves through),
    /// narrow-reduced there, never re-derived here.
    pub fn set_chamfer(&mut self, top_px: f32, bottom_px: f32) {
        self.chamfer_top = top_px.max(0.0);
        self.chamfer_bottom = bottom_px.max(0.0);
    }

    /// The chamfer pair this pipeline is currently carrying (`(top, bottom)`
    /// px), in the same units [`Self::set_chamfer`] stores. Test-only: the
    /// ENFORCEMENT hook for "every console layer resolves its corner through
    /// the same owner" — a law reads this off each of the panel/scanline/
    /// placard pipelines and asserts they agree, rather than trusting the
    /// call sites by convention.
    #[cfg(test)]
    pub(crate) fn chamfer(&self) -> (f32, f32) {
        (self.chamfer_top, self.chamfer_bottom)
    }

    /// Set the HALFTONE dot texture the NEXT `prepare` uploads into
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
            chamfer_top: self.chamfer_top,
            chamfer_bottom: self.chamfer_bottom,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: 0.0,
            dot_color: self.dot_color,
            second_color: self.second_color,
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
            chamfer_top: self.chamfer_top,
            chamfer_bottom: self.chamfer_bottom,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: 0.0,
            dot_color: self.dot_color,
            second_color: self.second_color,
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
    /// Its production consumers are the overlay's diagonal spine and selected-row
    /// mark, and the fold chevron in the writing column's own margin.
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
            chamfer_top: self.chamfer_top,
            chamfer_bottom: self.chamfer_bottom,
            halftone: self.halftone,
            halftone_angle: self.halftone_angle,
            halftone_cell: self.halftone_cell,
            _pad2: 0.0,
            dot_color: self.dot_color,
            second_color: self.second_color,
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
        if let Some(pipeline) = &self.second_pipeline {
            pass.set_pipeline(pipeline);
            pass.draw(0..6, 0..self.instance_count);
        }
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

/// Convert an 8-bit sRGB RGBA quad to linear-light floats for the shader.
pub(crate) fn srgba_u8_to_linear(c: [u8; 4]) -> [f32; 4] {
    let ch = crate::theme::srgb_channel_to_linear_f32;
    [ch(c[0]), ch(c[1]), ch(c[2]), c[3] as f32 / 255.0]
}

mod spine;
pub use spine::{chevron_arms, spine_segment};

#[cfg(test)]
mod tests;
