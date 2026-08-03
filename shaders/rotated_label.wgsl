// Rotated-label shader: paints ONE short text run at an arbitrary screen axis.
//
// glyphon 0.11 has no transform anywhere — `TextArea` carries left/top/scale/
// bounds and `CustomGlyph` carries left/top/width/height, so an upright run is
// the only thing the document layer can draw. This pipeline is the LABEL escape
// hatch: the run's whole coverage is composed ONCE on the CPU into a single R8
// mask (the same swash cache glyphon rasterizes from), and that mask is painted
// through ONE quad whose vertices are rotated onto `axis` — exactly the axis
// rotation `caret.wgsl`'s vertex stage already performs for the caret streak,
// applied to a glyph mask instead of a rounded rect.
//
// The mask is the run's OWN unrotated raster, so glyph spacing, hinting and
// anti-aliasing are byte-for-byte what an upright render produces; rotation is
// a single bilinear resample of that image and nothing else. There is no
// per-glyph transform and no second shaper: a label is one image.
//
// Coordinates are in PIXELS. The run has its own frame: `u` runs along the
// baseline in the advance direction, `v` runs DOWN from the baseline (so an
// ascender is negative `v`). `axis` maps `u` onto the screen and its +90°
// rotation maps `v`, which is what makes `(1, 0)` the inert upright case.

struct Globals {
    // Framebuffer size in physical pixels.
    viewport: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var mask: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct Instance {
    // Screen pixel the run's PEN ORIGIN (left edge of the first glyph, on the
    // baseline) sits at.
    @location(0) origin: vec2<f32>,
    // Unit axis the baseline advances along. (1, 0) = upright, left to right.
    @location(1) axis: vec2<f32>,
    // The composed mask's ink box in the run's own frame: min corner (u, v)
    // and size, in pixels. `v_min` is negative for a run with any ascender.
    @location(2) ink_min: vec2<f32>,
    @location(3) ink_size: vec2<f32>,
    // Linear colour at the run's START and END — a gradient ALONG the baseline,
    // so a world whose visual language is a gradient line gets one for free.
    // Both equal = a flat label.
    @location(4) color_a: vec3<f32>,
    @location(5) color_b: vec3<f32>,
    // Overall alpha multiplier.
    @location(6) alpha: f32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Mask uv, exactly the unit-quad corner: the quad IS the mask's ink box, so
    // an upright draw maps fragment centres onto texel centres 1:1.
    @location(0) uv: vec2<f32>,
    @location(1) color_a: vec3<f32>,
    @location(2) color_b: vec3<f32>,
    @location(3) alpha: f32,
};

// Unit quad corners (two triangles) in [0,1].
var<private> CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: Instance) -> VsOut {
    let corner = CORNERS[vid];
    // The corner in the RUN'S OWN frame, then rotated onto the screen axis.
    let local = inst.ink_min + corner * inst.ink_size;
    let ax = normalize(inst.axis);
    // +90° of `ax` in y-down pixel space: for the upright axis this is (0, 1),
    // i.e. screen DOWN, which is what makes `v` mean "below the baseline".
    let perp = vec2<f32>(-ax.y, ax.x);
    let px = inst.origin + local.x * ax + local.y * perp;

    // Pixel -> clip space. y flips (pixels are y-down, clip is y-up).
    let ndc = vec2<f32>(
        px.x / g.viewport.x * 2.0 - 1.0,
        1.0 - px.y / g.viewport.y * 2.0,
    );

    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = corner;
    out.color_a = inst.color_a;
    out.color_b = inst.color_b;
    out.alpha = inst.alpha;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // The run's own composed coverage. The mask carries a one-pixel transparent
    // border (the CPU compositor pads it), so the quad's rasterised edge — which
    // has no anti-aliasing of its own on a rotated quad — always lands on zero
    // coverage instead of cutting a stem off square.
    let cov = textureSampleLevel(mask, samp, in.uv, 0.0).r;
    // The gradient runs ALONG the baseline, in the run's own frame, so it slants
    // with the label rather than staying screen-horizontal.
    let color = mix(in.color_a, in.color_b, in.uv.x);
    let a = clamp(cov, 0.0, 1.0) * in.alpha;
    return vec4<f32>(color, a);
}
