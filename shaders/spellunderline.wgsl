// Spell-check squiggle shader: draws each misspelled word's underline as a
// wavy (cosine) red line inside a band quad. The vertex stage expands a unit quad
// to the band + a small margin so the antialiased stroke is not clipped. The
// fragment stage evaluates the distance from the pixel to the wave curve
//   y = -amp * cos(x * 2*pi / period)
// (taken about the band's vertical center) and shades a soft, ~`thickness`-wide
// antialiased stroke. Drawn UNDER the text so glyphs stay crisp on top.
//
// PHASE: the wave BEGINS AT ITS TOP under the word's first glyph. `x0`
// is the band's left edge (the first glyph), so at `px.x == x0` the phase is 0 and
// `-cos(0) == -1` puts the curve at `center.y - amp` — the crest (top, since y is
// screen-DOWN). A plain `sin` would start at the vertical center (a zero-crossing)
// and dive DOWN first; the cosine start lands a crest right under the first letter.
// The wave keeps FULL amplitude to both ends — the mark reads as one continuous
// chunky ripple, and the CENTERLINE never moves — but the STROKE WIDTH eases
// down to a small rounded tip over the last quarter-wavelength, so an end
// landing mid-crest lifts off like a brush stroke instead of hard-cutting.
// This applies only to the wavy mark (`amp > 0`); the flat writing-nit tick
// (`amp == 0`, see below) keeps its original two-sided OPACITY fade unchanged
// — a nit has no crest to taper toward, and the taper math assumes a period.
//
// Coordinates are in PIXELS (top-left origin). `viewport` maps pixel space to
// clip space ([-1,1], y-up) in the vertex stage, identical to selection.wgsl.
//
// NOTE: the half-size field is named `hsize` (not `half`) because `half` is a
// reserved type keyword in Metal Shading Language and breaks WGSL->MSL codegen.

struct Globals {
    // Framebuffer size in physical pixels.
    viewport: vec2<f32>,
    pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;

struct Instance {
    // Center of the band, in pixels.
    @location(0) center: vec2<f32>,
    // Half-size (width/2, height/2) of the band, in pixels.
    @location(1) hsize: vec2<f32>,
    // Pixel x of the band's LEFT edge (wave phase anchor).
    @location(2) x0: f32,
    // Sine amplitude (px).
    @location(3) amp: f32,
    // Sine period (px).
    @location(4) period: f32,
    // Stroke thickness (px).
    @location(5) thickness: f32,
    // Linear RGBA color.
    @location(6) color: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Pixel position of this fragment (absolute, for the wave phase).
    @location(0) px: vec2<f32>,
    // Band center (px) so the wave is taken about the vertical mid-line.
    @location(1) center: vec2<f32>,
    @location(2) hsize: vec2<f32>,
    @location(3) x0: f32,
    @location(4) amp: f32,
    @location(5) period: f32,
    @location(6) thickness: f32,
    @location(7) color: vec4<f32>,
};

const PI: f32 = 3.14159265;

// Tip-taper tuning (squiggle only, `amp > 0`). The tip is a fraction of the
// full stroke, floored so it can never fully vanish — the floor matches the
// 0.5px minimum `SpellUnderlinePipeline::prepare` already clamps `thickness`
// to (`src/spellunderline.rs`), so the tip is never thinner than the thinnest
// full-width stroke this same pipeline already ships elsewhere.
const TIP_FRACTION: f32 = 0.35;
const TIP_FLOOR_PX: f32 = 0.5;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: Instance) -> VsOut {
    let corner = QUAD_NDC[vid];
    // Margin so the antialiased stroke + wave crests + tip-taper end caps are
    // not clipped by the quad itself (the band height already includes the
    // amplitude, but pad for AA). The X margin grows with stroke thickness so
    // a large zoom's rounded end cap (bounded by the same tip radius the
    // fragment stage computes) always has room to render, not just the small
    // flat 2px pad that was enough before an end could bulge past the band.
    let tip_half_bound = max(inst.thickness * 0.5 * TIP_FRACTION, TIP_FLOOR_PX);
    let margin_x = max(2.0, tip_half_bound + 1.5);
    let extent = inst.hsize + vec2<f32>(margin_x, 2.0);
    let local = corner * extent;
    let px = inst.center + local;

    let ndc = vec2<f32>(
        px.x / g.viewport.x * 2.0 - 1.0,
        1.0 - px.y / g.viewport.y * 2.0,
    );

    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    out.px = px;
    out.center = inst.center;
    out.hsize = inst.hsize;
    out.x0 = inst.x0;
    out.amp = inst.amp;
    out.period = inst.period;
    out.thickness = inst.thickness;
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Horizontal extent of the band (the word's own span).
    let left = in.center.x - in.hsize.x;
    let right = in.center.x + in.hsize.x;
    let span = right - left;
    let phase = (in.px.x - in.x0) * (2.0 * PI / in.period);
    // Curve height about the band's vertical center. `-cos` so the wave BEGINS at
    // its TOP (crest) under the first glyph (phase 0 → center.y - amp); see the
    // header note. UNCHANGED by the taper below — only the stroke width varies,
    // never the centerline.
    let wave_y = in.center.y - in.amp * cos(phase);

    // Slope of `-amp*cos(phase)` is `+amp*(…)*sin(phase)`; see below.
    let dydx = in.amp * (2.0 * PI / in.period) * sin(phase);

    var dist: f32;
    var half_w = in.thickness * 0.5;

    if (in.amp > 0.0) {
        // THICKNESS TAPER: ease the stroke's half-width down to a small
        // nonzero tip over the last `taper_len` (a quarter wavelength,
        // shortened for a span too short to hold one — never more than half
        // the span, so a one-character word still tapers instead of
        // overlapping its own opposite end). `u` is 0 exactly at the word's
        // boundary and 1 by one taper length inward; the cubic ease
        // (`3u²-2u³`) is the same smoothstep shape the antialiasing below
        // already uses, so the taper reads as one continuous easing, not a
        // kink. `tip_half` is a PRESENCE floor, not a fade target: the tip
        // never thins past it, so the end stays a locatable dot rather than
        // vanishing.
        let tip_half = max(half_w * TIP_FRACTION, TIP_FLOOR_PX);
        let taper_len = min(in.period * 0.25, span * 0.5);
        let local_x = clamp(in.px.x - left, 0.0, span);
        let d = min(local_x, span - local_x);
        let u = clamp(d / max(taper_len, 0.0001), 0.0, 1.0);
        let eased = u * u * (3.0 - 2.0 * u);
        half_w = mix(tip_half, half_w, eased);

        if (in.px.x < left || in.px.x > right) {
            // Past the word's own boundary: this is the ROUNDED CAP, not a
            // re-creation of the old hard clip. Distance is to the curve's
            // own endpoint (still at full amplitude — the crest/trough the
            // wave actually reaches at x=left/right), so the cap is a small
            // circle of radius `tip_half` centered exactly where the curve
            // stops, matching a round line-cap. Deliberately NOT used inside
            // [left, right]: there, the slope-compensated perpendicular
            // distance below is the correct measure on the wave's steep
            // sections, and a point-distance approximation would fatten them.
            let anchor_x = select(right, left, in.px.x < left);
            let anchor_phase = (anchor_x - in.x0) * (2.0 * PI / in.period);
            let anchor_y = in.center.y - in.amp * cos(anchor_phase);
            dist = length(in.px - vec2<f32>(anchor_x, anchor_y));
        } else {
            // Perpendicular distance to the curve: divide the vertical gap by
            // the local slope magnitude sqrt(1 + dy/dx^2), which keeps the
            // stroke an even width even on the steep parts of the wave (a
            // plain vertical |dy| would fatten the flats and thin the slopes).
            dist = abs(in.px.y - wave_y) / sqrt(1.0 + dydx * dydx);
        }
    } else {
        dist = abs(in.px.y - wave_y) / sqrt(1.0 + dydx * dydx);
    }

    // Antialiased stroke with a ~1px feather around the (possibly tapered)
    // half-width.
    var a = 1.0 - smoothstep(half_w - 0.75, half_w + 0.75, dist);

    if (in.amp <= 0.0) {
        // Zero-amplitude writing-nit: UNCHANGED. Fade the very ends by
        // OPACITY only, exactly as before the taper above existed — the flat
        // tick has no crest to lift off of, so it keeps its original soft cut
        // rather than the wave's geometric taper.
        let edge = 1.5;
        a = a * smoothstep(left - 0.5, left + edge, in.px.x);
        a = a * (1.0 - smoothstep(right - edge, right + 0.5, in.px.x));
    }

    a = clamp(a, 0.0, 1.0) * in.color.a;
    return vec4<f32>(in.color.rgb, a);
}
