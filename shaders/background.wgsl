// Background / MARGIN gradient shader (PAGE MODE, N++ figure/ground).
//
// Draws ONE fullscreen triangle whose fragment splits the canvas into the calm
// PAGE column and the styled MARGINS:
//   * inside the column rect [col_left, col_left+col_w) -> alpha 0, so the flat
//     base_100 clear shows through perfectly (the page reads as a clean shape).
//   * outside (the margins) -> a per-world gradient (mix(from,to,t) along `dir`),
//     alpha 1, painting the ground the page floats on.
//
// Almost entirely static (no per-doc/per-glyph input ever reaches this
// pipeline). The ONE exception (item 87) is `drift` — Bombora's WAVES
// phase drift, a single scalar the host uploads each frame from the SAME
// shared ambient clock the lava lamp and twinkling stars ride; it is `0.0`
// for every non-Waves ground and for every headless capture (that clock never
// advances there), so the render stays exactly as byte-deterministic as
// before at rest. The gradient colors arrive in LINEAR space (the render
// target is sRGB; the host converts the per-world theme bytes before
// upload), like the selection shader.
//
// When page mode is OFF the host passes col_w == viewport width, so the column
// covers everything and the margins vanish — identical to the old flat clear.

struct Globals {
    // Framebuffer size in physical pixels.
    viewport: vec2<f32>,
    // Page column left edge + width, in physical pixels.
    col_left: f32,
    col_w: f32,
    // Gradient endpoints (LINEAR rgb; a is the margin opacity, normally 1).
    // NOTE: named `c_from`/`c_to` — `from` is a reserved keyword in WGSL.
    c_from: vec4<f32>,
    c_to: vec4<f32>,
    // Unit gradient direction in UV space (e.g. (0,1)=vertical, (.7,.7)=diagonal).
    // For Stripes this is (cos angle, sin angle), so the gradient runs ALONG the
    // stripe angle.
    dir: vec2<f32>,
    // Procedural margin ground: 0=plain gradient, 1=dots, 2=starfield,
    // 3=pinstripe, 4=stripes, 5=bands, 6=waves, 7=zigzag, 8=organic,
    // 9=deckle. Matches `Background::shader_id` in src/theme/model.rs.
    shader: u32,
    // WAVES phase-drift, in radians (item 87) — a DEDICATED scalar in what was
    // `pad` after `shader`. `0.0` for every non-Waves ground and every
    // settled/headless frame (so those renders stay byte-identical); shader 6
    // reads it as `g.drift` in `waves_rgb`. Kept OUT of `params` because item
    // 86's Zigzag (shader 7) already uses all four `params` slots — routing the
    // drift through `params.z` would zero a Zigzag world's amplitude every
    // frame. Must byte-match `Globals.drift` in src/background.rs.
    drift: f32,
    // Mark/band tint (LINEAR rgb; a is the max coverage of the marks/band). For
    // shader 5/6 (Bands/Waves) this is the MIDDLE of three authored tones —
    // `c_from`/`c_pat`/`c_to` read as tones 0/1/2 (see `bands_rgb`/`waves_rgb`).
    // For shader 7 (Zigzag) this is the chevron mark's own tint, same role as
    // Dots/Pinstripe's `tint`.
    c_pat: vec4<f32>,
    // Per-ground params — the SAME four slots read with a DIFFERENT meaning
    // per shader (exactly one ground is ever active at a time, so there is no
    // real collision): params.x = Dots proximity flag (0/1) OR shader 7's
    // chevron repeat wavelength `period_px` (item 86); params.y = the
    // Stripes/Bands angle (radians) OR shader 7's own chevron travel angle;
    // params.z = shader 7's chevron amplitude `amplitude_px`; params.w =
    // shader 7's extra coverage multiplier `density`. Shader 9 (Deckle, item
    // 158) reads all four with its OWN meanings — lane pitch / wander
    // amplitude / density / weave; see `deckle_rgb`. Shader 8 (Organic) reads
    // params.z as its own theme-owned ARRANGEMENT scalar (`theme::
    // Arrangement`): `0.0` = the rounded MASSES field, `1.0` = the crisp
    // three-object FINDS field; see `organic_rgb`. All four are 0 for
    // every ground this round didn't touch, so those grounds take their
    // exact original code path. (Waves' item-87 drift is NOT here — it rides
    // the dedicated `drift` slot above.)
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> g: Globals;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    // Pixel position of the fragment (top-left origin).
    @location(0) px: vec2<f32>,
};

// A single oversized triangle covering the whole clip space.
var<private> VERTS: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0), vec2<f32>( 3.0, -1.0), vec2<f32>(-1.0,  3.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    let ndc = VERTS[vid];
    var out: VsOut;
    out.clip = vec4<f32>(ndc, 0.0, 1.0);
    // Map clip [-1,1] (y-up) back to pixels (y-down, top-left origin).
    out.px = vec2<f32>(
        (ndc.x * 0.5 + 0.5) * g.viewport.x,
        (0.5 - ndc.y * 0.5) * g.viewport.y,
    );
    return out;
}

// A scalar hash in [0,1) from a 2D integer-ish cell id. Deterministic (no clock),
// so the starfield is byte-stable across captures.
fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Proximity to the PAGE-COLUMN boundary as an intensity in [0,1]: 1.0 right at
// the page edge, decaying outward into the margin (exp falloff). Drives the
// Stripes band + the proximity-scaled Dots — the "play area radiates into the
// ground" feel. Pure pixel math, no time.
const EDGE_FALLOFF: f32 = 90.0;

// ZIGZAG (shader 7) — the chevron ribbon's own stroke half-width as a fraction
// of the authored `amplitude_px`, so a broader profile draws a proportionally
// bolder ribbon (never an absolute px thickness that a scaled-up field would
// leave hairline). It ALSO derives the field's row pitch through the abutment
// rule (`row_h = 2*amp + thickness`, see `pattern_coverage`'s `shader == 7u`
// branch) — hence a named constant rather than an inline literal: the host
// mirror in `render/tests/backgrounds_item89.rs` pins the same number, and a
// grep-law asserts this line still reads it.
const ZIGZAG_STROKE_FRAC: f32 = 0.10;

fn edge_intensity(px: vec2<f32>) -> f32 {
    var d = 0.0;
    if (px.x < g.col_left) {
        d = g.col_left - px.x;
    } else {
        d = px.x - (g.col_left + g.col_w);
    }
    return exp(-max(d, 0.0) / EDGE_FALLOFF);
}

// Linear proximity to the PAGE-COLUMN boundary, normalized across the FULL
// margin width: 1.0 right at the page edge, 0.0 out at the viewport edge. Unlike
// `edge_intensity` (a fast exp band), this ramps over the WHOLE margin, so a dot
// RADIUS keyed off it reads its size gradient across the entire ground instead
// of collapsing into one full-size band at the edge. Pure pixel math, no time.
fn edge_proximity(px: vec2<f32>) -> f32 {
    var d = 0.0;
    var span = 1.0;
    if (px.x < g.col_left) {
        d = g.col_left - px.x;
        span = max(g.col_left, 1.0);
    } else {
        d = px.x - (g.col_left + g.col_w);
        span = max(g.viewport.x - (g.col_left + g.col_w), 1.0);
    }
    return clamp(1.0 - d / span, 0.0, 1.0);
}

// Coverage [0,1] of the assigned margin ground at pixel `px`. All grounds are
// pure functions of pixel coordinates — STATIC, no time. Tuned to whisper.
fn pattern_coverage(px: vec2<f32>) -> f32 {
    // --- 1: DOTS — a grid of round dots; `params.x` flips proximity scaling. ---
    if (g.shader == 1u) {
        let cell = 24.0;
        let c = fract(px / cell) - vec2<f32>(0.5, 0.5);
        let d = length(c * cell);
        if (g.params.x > 0.5) {
            // edge=true: the dot RADIUS scales with page proximity — a FULL, fat
            // dot hugging the page boundary, SHRINKING to ~28% out at the far
            // margin (the N++ reference look). SIZE carries the gradient (keyed
            // off the linear, full-margin `edge_proximity`, NOT the fast exp
            // band), so the alpha only floors GENTLY — far dots stay visible-small
            // instead of dissolving before their size can read.
            let p = edge_proximity(px);
            let radius = mix(0.85, 3.0, p); // ~28% far -> a full fat dot at the edge
            let dot = 1.0 - smoothstep(radius, radius + 0.9, d);
            let alpha = mix(0.5, 1.0, p);   // gentle falloff; brightest hugging the page
            return dot * alpha;
        }
        // edge=false: today's UNIFORM ~1.4px dots with a 1px feather (unchanged).
        return 1.0 - smoothstep(1.4, 2.4, d);
    }
    // --- 4: STRIPES — diagonal stripes in a bright band hugging the page edge,
    // dissolving outward into the gradient (the N++ look). The band peaks at the
    // boundary (edge_intensity) and the stripes run perpendicular to `dir`. ---
    if (g.shader == 4u) {
        let a = g.params.y;
        // Coordinate across the stripes (perpendicular bands give the diagonal look).
        let coord = px.x * cos(a) + px.y * sin(a);
        let period = 13.0;
        let f = abs(fract(coord / period) - 0.5) * period; // px distance to a stripe
        let line = 1.0 - smoothstep(2.0, 3.5, f);          // ~bright diagonal stripe
        return line * edge_intensity(px);                  // dissolve outward
    }
    // --- 3: PINSTRIPE — fine vertical parallel lines (ledger / print rules). ---
    if (g.shader == 3u) {
        let period = 9.0;
        let x = abs(fract(px.x / period) - 0.5) * period; // px distance to line
        return 1.0 - smoothstep(0.5, 1.2, x);
    }
    // --- 2: STARFIELD — scattered dots + the occasional 4-point sparkle. ---
    if (g.shader == 2u) {
        let cell = 34.0;
        let id = floor(px / cell);
        let local = fract(px / cell);
        // Per-cell jittered star position + a presence roll (only some cells lit).
        let jx = hash21(id + vec2<f32>(1.0, 0.0));
        let jy = hash21(id + vec2<f32>(0.0, 7.0));
        let present = hash21(id + vec2<f32>(3.0, 5.0));
        let star = vec2<f32>(jx, jy);
        let dpx = (local - star) * cell;
        let r = length(dpx);
        // A small round dot for every lit cell.
        var cov = (1.0 - smoothstep(0.7, 1.7, r)) * step(0.55, present);
        // The brightest ~1/6 cells also get a thin 4-point sparkle cross.
        if (present > 0.84) {
            let cross = (1.0 - smoothstep(0.4, 1.0, abs(dpx.x))) * (1.0 - smoothstep(2.5, 4.5, abs(dpx.y)))
                      + (1.0 - smoothstep(0.4, 1.0, abs(dpx.y))) * (1.0 - smoothstep(2.5, 4.5, abs(dpx.x)));
            cov = max(cov, clamp(cross, 0.0, 1.0));
        }
        return cov;
    }
    // --- 7: ZIGZAG — a TILED field of repeating chevron ("V") rows,
    // whisper-composited over the gradient like Dots/Pinstripe (item 86;
    // FIELD-tiled by item 89). Four independently authored dials (see
    // `Background::Zigzag`'s own doc): `period` is the chevron's repeat
    // wavelength ALONG its travel (the SCALE dial — the tooth wavelength
    // ALONE; the row-to-row pitch ACROSS travel is DERIVED, see the abutment
    // rule below), `amp` its peak excursion across travel (the PROFILE dial —
    // which also derives the stroke's own thickness AND, through the
    // abutment rule, the row pitch), `a` the direction the chevrons
    // themselves travel (the DIRECTION dial, independent of the base
    // gradient's own `dir`), and `dens` an extra per-world coverage
    // multiplier (the CONTRAST dial) stacked with the shared
    // `PATTERN_MAX_COVERAGE` ceiling every mark ground already carries. ---
    if (g.shader == 7u) {
        let period = max(g.params.x, 1.0);
        let a = g.params.y;
        let amp = max(g.params.z, 0.0);
        // A negative density selects the data-authored filled-band treatment;
        // its magnitude remains the ordinary contrast dial. The uniform has no
        // spare slot, so this preserves Gumtree's positive-density upload bits.
        let banded = g.params.w < 0.0;
        let dens = clamp(abs(g.params.w), 0.0, 1.0);
        let ca = cos(a);
        let sa = sin(a);
        // Rotate into the chevron's own travel frame: `rx` runs ALONG the
        // travel direction (the triangle-wave meander axis), `ry` across it
        // (the axis the "V" excursion is measured on, and the axis the rows
        // stack along).
        let rx = px.x * ca + px.y * sa;
        let ry = -px.x * sa + px.y * ca;
        // A broad triangle wave of `rx` (period `period`), in [-1, 1] — the
        // SAME fold `jagged_wave_band` (`shaders/selection.wgsl`, item 71,
        // retired) used: `fract` + a fold gives the sharp "jagged" corners a
        // chevron needs.
        let tri = abs(fract(rx / period) * 2.0 - 1.0) * 2.0 - 1.0;
        let center = tri * amp;
        let thickness = max(amp * ZIGZAG_STROKE_FRAC, 1.2);
        // ITEM 89 — THE FIELD FOLD. `center` is a function of `rx` ALONE, so
        // `abs(ry - center)` (item 86's original) describes ONE continuous
        // chevron LINE embedded in the plane: teeth repeat along travel, but
        // nothing repeats it ACROSS travel, so a tall margin showed one
        // wandering stroke with large blank areas and a taller window only
        // let that single stroke travel further before running off-canvas.
        // Folding `(ry - center)` through the row pitch turns that one line
        // into the INFINITE FAMILY `center + k * row_h` — a genuinely tiled
        // Mario-like zigzag field that covers any viewport, at any height,
        // with the same row rhythm.
        //
        // ITEM 89-FIX — THE ABUTMENT RULE, and why the pitch is NOT `period`.
        // Item 89's first cut set `row_h = period` (a square lattice in the
        // travel frame). That tiles, but it does NOT COVER: row `k`'s ribbon
        // only ever visits `ry` in `[k*row_h - amp - t, k*row_h + amp + t]`,
        // so when `2*amp + 2*t < period` the field carries a VOID BAND of
        // `period - 2*amp - 2*t` px between every pair of neighbouring rows
        // that NO chevron enters at ANY `rx` — a hard blank lane across the
        // whole margin. On Gumtree's authored 250/85 that lane was ~70px
        // wide, and a short window's narrow margin could land inside one
        // (measured: a 182x80 band of a 1600x600 right margin at literally
        // zero deviation).
        //
        // So the pitch is DERIVED from the profile instead: `row_h = 2*amp +
        // thickness` makes each row's ribbon sweep exactly one pitch wide,
        // and its ribbon CORE (`d <= 0.6*t`) sweep `2*amp + 1.2*t` — a pitch
        // PLUS `0.2*t` of overlap. Consecutive rows therefore ABUT (strictly
        // overlap) across the travel axis for ANY authored dials, so a void
        // band is impossible BY CONSTRUCTION rather than by tuning: over the
        // PLANE the covering property no longer depends on the
        // period/amplitude ratio or on the angle. (ITEM 100 — the one bound:
        // a VIEWPORT sees a rotated rectangle of that plane, so it inherits
        // the guarantee only while it holds a whole tooth of travel; a canvas
        // narrower than one `period` never sweeps the full excursion. See
        // `Background::zigzag_row_pitch_px`'s doc comment — no shipping
        // world's tooth is within 4x of that regime.) It also retires
        // item 89's row-collision clamp — abutting rows cannot smear together
        // (each ribbon is `2*t` wide inside a `2*amp + t` lane), so there is
        // nothing left to guard.
        let row_h = 2.0 * amp + thickness;
        let u = (ry - center) / row_h;
        // Signed distance to the NEAREST row line (px): fold to [-0.5, 0.5).
        let d = abs(fract(u + 0.5) - 0.5) * row_h;
        let line = 1.0 - smoothstep(thickness * 0.6, thickness, d);
        if (banded) {
            // One filled chevron lane followed by one untouched lane. `u` is
            // continuous in viewport space, so both margins share one rhythm.
            return select(0.0, dens, fract(u) < 0.5);
        }
        return line * dens;
    }
    // 0: plain gradient — no marks.
    return 0.0;
}

// --- 5: BANDS — EXACTLY THREE broad, tone-on-tone diagonal bands spanning the
// WHOLE margin field (cut-paper grass, not a repeating stripe-tile). Unlike
// `pattern_coverage`'s whisper-marks-over-a-gradient grounds, this (and
// `waves_rgb` below) computes the FINAL rgb directly — the three tones ARE the
// field, not a low-coverage overlay — so `fs_main` branches to it before the
// base-gradient/dither/pattern-overlay pipeline runs (see the early-return
// there). Pure function of pixel position: static, no time, no assets. ---
//
// `coord` projects `px` onto the band direction (`params.y` = angle, the same
// slot Stripes uses); `extent` is that SAME projection of the full viewport
// rect, so `t = coord / extent` lands in [0,1] over the WHOLE canvas — the two
// boundaries at 1/3 and 2/3 are therefore FRACTIONS of the viewport, not a
// fixed pixel period, so a narrower/wider page CROPS OR SCALES the identical
// three-band field instead of tiling more stripes into it. A small smoothstep
// (`aa`, ~1.5px in `t`-space) feathers each boundary — "crisp-but-quiet": a
// tight edge, but between three low-mutual-contrast ladder rungs.
// Shared by `bands_rgb`/`waves_rgb`: three world tones (c_from/c_pat/c_to),
// split at two boundaries along `coord` with a shared antialias half-width
// `aa` — the ONE owner of the "two-boundary tri-tone mix" both fields do,
// only their boundary/coord math differs.
fn tri_tone_mix(coord: f32, b1: f32, b2: f32, aa: f32) -> vec3<f32> {
    let m1 = smoothstep(b1 - aa, b1 + aa, coord);
    let m2 = smoothstep(b2 - aa, b2 + aa, coord);
    let tone01 = mix(g.c_from.rgb, g.c_pat.rgb, m1);
    return mix(tone01, g.c_to.rgb, m2);
}

// ITEM 163: the shipped drift translated the whole field by only ~5x4px
// (`sin(g.drift)*5.0, cos(g.drift*0.73)*4.0`) — against a `scale_px` cell of
// 156px that reads as under 4% of a cell, so a live user glancing back after
// a minute genuinely could not see it move even though every phase-motion
// law (below) already passed: those laws proved SOME pixel changed, never
// that the change was big enough to register. `ORGANIC_DRIFT_X_FRAC`/`_Y`
// scale the drift to a FRACTION of the cell size `s` instead of a fixed
// pixel count, so the authored displacement stays proportionate if a future
// Organic world ever ships a different `scale_px`. Both fractions stay well
// under 0.5 (never sliding a whole cell's width in one throw) and the two
// axes keep DIFFERENT magnitudes/frequencies (`sin(g.drift)` vs
// `cos(g.drift * 0.73)`, unchanged) so the composition reads as one cohesive
// collage sliding a lazy, slightly elliptical path — never a rigid ping-pong
// on a single line. `cell`/`local` are translated TOGETHER (both read
// `px + drift`), the same "pan an infinite tiled field" shape as before: a
// blob's identity is a hash of its (translated) integer cell, so panning
// only ever reveals more of the SAME static authored composition — no blob
// spawns, dissolves, or deforms; the shapes are pure functions of position,
// exactly as before, just sampled through a bigger window.
//
// `ORGANIC_DRIFT_MIN_X_PX`/`_Y` are a FLOOR under that same fraction, not a
// second independent tunable: a pure fraction alone reintroduces item 163's
// own bug at a small `s` — `s`'s own defensive clamp above bottoms out at
// 32.0, where `32.0 * ORGANIC_DRIFT_X_FRAC` is a ~4px flashback to the
// pre-163 amplitude. The floor guarantees a real, on-screen displacement at
// ANY reachable `s` (Bowerbird's shipped 156px included, where the fraction
// alone already clears it and wins the `max`), so a future Organic world
// authored with a smaller cell can't silently ship the exact defect this
// item fixed.
const ORGANIC_DRIFT_X_FRAC: f32 = 0.13;
const ORGANIC_DRIFT_Y_FRAC: f32 = 0.10;
const ORGANIC_DRIFT_MIN_X_PX: f32 = 12.0;
const ORGANIC_DRIFT_MIN_Y_PX: f32 = 9.0;

// --- FINDS: the COLLECTED-TREASURE arrangement (`theme::Arrangement::Finds`,
// params.z >= 0.5). One cell draws one deliberately arranged collection of
// THREE crisp objects — a large ANCHOR, a smaller COMPANION offset across its
// edge, and a tiny CUT-OUT punched back to the open ground — mixed from
// circles, squares and triangles. Scale hierarchy, offset, rotation, overlap
// and tone assignment are all seeded from the cell's own identity, so they
// vary collection to collection while the three roles never do. ---
//
// The ranges below are chosen so the role ordering (visible anchor area >
// visible companion area > cut-out area) holds for EVERY reachable hash
// combination, not on average:
//   * all three kinds enclose the SAME area at a given nominal radius (the
//     kind constants equalize it), so a role's area follows from its radius
//     alone whatever kinds its hashes drew;
//   * the companion is at least FINDS_COMPANION_LO of the anchor radius and
//     the cut-out at most FINDS_ACCENT_HI, and
//     `ACCENT_HI^2 < COMPANION_LO^2 - ACCENT_HI^2`, so even a cut-out landing
//     wholly inside the companion cannot invert that pair;
//   * the companion centre sits within FINDS_OFFSET_HI anchor-radii, under the
//     sum of the two shapes' SMALLEST reaches (their inradii, the triangle's
//     being the smallest of the three), so the pair always overlaps and a
//     collection is one connected arrangement rather than scattered pieces;
//   * the cut-out sits on the far side of the anchor centre from the
//     companion and inside the anchor's own inradius, so it always reads as a
//     hole through the collection and never as a fourth object.
// A collection reaches at most `JITTER/2 + ANCHOR_HI * (OFFSET_HI +
// COMPANION_HI * 1.5551)` ~ 0.44 of a cell from its centre — the 1.5551 is the
// triangle's circumradius per unit nominal radius, the largest reach of the
// three kinds — so it stays comfortably inside the half cell, neighbouring
// collections never merge, and each one is separately readable on the open
// ground between them.
const FINDS_SQUARE_HALF: f32 = 0.8862269; // (2a)^2 == pi*r^2
const FINDS_TRI_HALF_SIDE: f32 = 1.3468; // sqrt(3)*h^2 == pi*r^2
const FINDS_TRI_INRADIUS: f32 = 0.7776; // the smallest inradius of the three kinds
const FINDS_ANCHOR_LO: f32 = 0.150;
const FINDS_ANCHOR_HI: f32 = 0.195;
const FINDS_COMPANION_LO: f32 = 0.46;
const FINDS_COMPANION_HI: f32 = 0.56;
const FINDS_OFFSET_LO: f32 = 0.80;
const FINDS_OFFSET_HI: f32 = 1.02;
const FINDS_ACCENT_LO: f32 = 0.20;
const FINDS_ACCENT_HI: f32 = 0.26;
const FINDS_ACCENT_OFFSET_LO: f32 = 0.10;
const FINDS_ACCENT_OFFSET_HI: f32 = 0.34;
const FINDS_JITTER: f32 = 0.15;
const FINDS_LATTICE_ANGLE: f32 = 0.42;
const FINDS_DROPOUT: f32 = 0.10;
const FINDS_TAU: f32 = 6.2831855;
// The feather half-width, in PHYSICAL pixels. Crisp is the whole point, so
// this is a fixed sub-pixel skirt rather than a fraction of a cell: the same
// hard edge resolves without stair-stepping at 1x and at 2x, and at any cell
// scale a future Organic world might author.
const FINDS_EDGE_AA_PX: f32 = 0.75;
// FINDS declares its own cell FLOOR, the way Deckle declares a lane pitch
// floor: the cut-out is a FRACTION of the anchor, so below this scale it falls
// under a pixel and a collection aliases into speckle instead of reading as
// three arranged objects. A property of the shader, not of the dial pair.
const FINDS_MIN_SCALE_PX: f32 = 96.0;

fn finds_rot(p: vec2<f32>, a: f32) -> vec2<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec2<f32>(c * p.x + s * p.y, -s * p.x + c * p.y);
}

fn sd_square(p: vec2<f32>, h: f32) -> f32 {
    let q = abs(p) - vec2<f32>(h, h);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
}

// Exact signed distance to an equilateral triangle of half-side `h`.
fn sd_triangle(p: vec2<f32>, h: f32) -> f32 {
    let k = 1.7320508;
    var q = vec2<f32>(abs(p.x) - h, p.y + h / k);
    if (q.x + k * q.y > 0.0) {
        q = vec2<f32>(q.x - k * q.y, -k * q.x - q.y) * 0.5;
    }
    q.x = q.x - clamp(q.x, -2.0 * h, 0.0);
    return -length(q) * sign(q.y);
}

fn finds_shape(p: vec2<f32>, kind: u32, r: f32) -> f32 {
    if (kind == 0u) { return length(p) - r; }
    if (kind == 1u) { return sd_square(p, r * FINDS_SQUARE_HALF); }
    return sd_triangle(p, r * FINDS_TRI_HALF_SIDE);
}

// The SDFs above are in CELL units; `* s` puts them in physical pixels, where
// the feather is a fixed fraction of one pixel at any scale or device ratio.
fn finds_fill(sd_cell: f32, s: f32) -> f32 {
    return 1.0 - smoothstep(-FINDS_EDGE_AA_PX, FINDS_EDGE_AA_PX, sd_cell * s);
}

fn organic_finds_rgb(px: vec2<f32>, s: f32, d: f32, drift: vec2<f32>) -> vec3<f32> {
    // The lattice is ROTATED and then per-row sheared: the scattered rhythm
    // survives, its rows and columns do not, so the field never resolves into
    // the visible grid a plain `floor(px / s)` would draw.
    let w = px + drift;
    let ca = cos(FINDS_LATTICE_ANGLE);
    let sa = sin(FINDS_LATTICE_ANGLE);
    let q = vec2<f32>(w.x * ca - w.y * sa, w.x * sa + w.y * ca) / s;
    let row = floor(q.y);
    let qx = q.x + hash21(vec2<f32>(row, 91.0));
    let cell = vec2<f32>(floor(qx), row);
    let local = vec2<f32>(fract(qx), fract(q.y)) - vec2<f32>(0.5, 0.5);

    let h0 = hash21(cell + vec2<f32>(19.0, 5.0));
    // Some cells hold nothing: open ground is part of the arrangement.
    if (h0 < FINDS_DROPOUT) {
        return g.c_from.rgb;
    }
    let h1 = hash21(cell + vec2<f32>(3.0, 41.0));
    let h2 = hash21(cell + vec2<f32>(57.0, 13.0));
    let h3 = hash21(cell + vec2<f32>(7.0, 71.0));
    let h4 = hash21(cell + vec2<f32>(31.0, 23.0));
    let h5 = hash21(cell + vec2<f32>(11.0, 89.0));

    let centre = (vec2<f32>(fract(h0 * 17.0), fract(h1 * 13.0)) - vec2<f32>(0.5, 0.5)) * FINDS_JITTER;
    let r_a = mix(FINDS_ANCHOR_LO, FINDS_ANCHOR_HI, h1);
    let r_b = r_a * mix(FINDS_COMPANION_LO, FINDS_COMPANION_HI, h4);
    let r_c = r_a * mix(FINDS_ACCENT_LO, FINDS_ACCENT_HI, h5);
    let kind_a = u32(floor(fract(h2 * 5.0) * 3.0));
    let kind_b = (kind_a + 1u + u32(floor(fract(h3 * 7.0) * 2.0))) % 3u;
    let kind_c = u32(floor(fract(h4 * 11.0) * 3.0));
    let phi = h2 * FINDS_TAU;
    let arm = vec2<f32>(cos(phi), sin(phi));
    let c_b = centre + arm * (r_a * mix(FINDS_OFFSET_LO, FINDS_OFFSET_HI, fract(h3 * 3.0)));
    let c_c = centre - arm * (r_a * FINDS_TRI_INRADIUS
        * mix(FINDS_ACCENT_OFFSET_LO, FINDS_ACCENT_OFFSET_HI, fract(h5 * 3.0)));

    let cov_a = finds_fill(
        finds_shape(finds_rot(local - centre, fract(h3 * 23.0) * FINDS_TAU), kind_a, r_a), s);
    let cov_b = finds_fill(
        finds_shape(finds_rot(local - c_b, fract(h4 * 29.0) * FINDS_TAU), kind_b, r_b), s);
    let cov_c = finds_fill(
        finds_shape(finds_rot(local - c_c, fract(h5 * 19.0) * FINDS_TAU), kind_c, r_c), s);

    // The cut-out is removed from BOTH pieces, so a collection reads as one
    // arranged thing with a hole through it rather than three stacked marks —
    // and the hole returns to EXACTLY the open ground at any density, which is
    // what lets a pixel law find it as an enclosed region of ground tone.
    let keep = 1.0 - cov_c;
    let swap = fract(h2 * 31.0) > 0.5;
    // Each piece carries its OWN opaque ink, already mixed toward the ground by
    // the authored density, and is then composited by coverage alone. Blending
    // the second piece through the first instead would give the overlap a
    // fourth, half-transparent value — a stack of films rather than one
    // arrangement of cut objects, and the exact haze this arrangement replaces.
    // `density == 0.0` still collapses both inks onto the ground exactly.
    let ink_a = mix(g.c_from.rgb, select(g.c_pat.rgb, g.c_to.rgb, swap), d);
    let ink_b = mix(g.c_from.rgb, select(g.c_to.rgb, g.c_pat.rgb, swap), d);
    var rgb = mix(g.c_from.rgb, ink_a, cov_a * keep);
    return mix(rgb, ink_b, cov_b * keep);
}

// 8: the ORGANIC ground, in either theme-owned arrangement (params.z). MASSES
// (0.0) is cut-paper blobs: three differently-offset rounded cell fields make
// large masses, islands, and droplets, and subtracting a small inner field
// leaves occasional holes. FINDS (1.0) is the crisp collected-treasure field
// above. The only time input either takes is the shared, slow ambient phase,
// and it enters as ONE whole-field translation before the lattice is derived —
// so a shape is a pure function of its cell, and the field can only pan, never
// morph, spawn, dissolve, or animate one object of a collection on its own.
fn organic_rgb(px: vec2<f32>) -> vec3<f32> {
    let finds = g.params.z >= 0.5;
    let s = max(g.params.x, select(32.0, FINDS_MIN_SCALE_PX, finds));
    let d = clamp(g.params.y, 0.0, 1.0);
    let drift = vec2<f32>(
        sin(g.drift) * max(s * ORGANIC_DRIFT_X_FRAC, ORGANIC_DRIFT_MIN_X_PX),
        cos(g.drift * 0.73) * max(s * ORGANIC_DRIFT_Y_FRAC, ORGANIC_DRIFT_MIN_Y_PX),
    );
    if (finds) {
        return organic_finds_rgb(px, s, d, drift);
    }
    let cell = floor((px + drift) / s);
    let local = fract((px + drift) / s) - vec2<f32>(0.5);
    let jitter = vec2<f32>(hash21(cell), hash21(cell + vec2<f32>(7.0, 3.0))) - vec2<f32>(0.5);
    let mass = 1.0 - smoothstep(0.20, 0.42, length(local - jitter * 0.22));
    let island = 1.0 - smoothstep(0.09, 0.22, length(local + jitter * 0.35));
    let hole = 1.0 - smoothstep(0.045, 0.10, length(local - jitter * 0.52));
    let tone = mix(g.c_from.rgb, g.c_pat.rgb, mass * d);
    let with_island = mix(tone, g.c_to.rgb, island * d * 0.72);
    return mix(with_island, g.c_from.rgb, hole * mass * 0.65);
}

// --- 9: DECKLE — THE HANDMADE-PAPER MATERIAL FIELD (item 158). ---
//
// A family of quasi-random CONTOUR LANES through the margins. Each lane is
// seeded from its own index, so no two neighbours agree; a two-tone sine
// profile WANDERS every lane, so none of them is a ruled line. Entirely
// static: a pure function of the fragment position, the page column, three
// authored tones and three authored dials. No clock, no texture, no asset, and
// no world identity ever reaches this branch.
//
// ONE authored `weave` (params.w) picks which of two profiles a world wears.
// Both read the SAME tones and the SAME dials; a second world adopts the other
// profile by writing one word in its own theme literal (`theme::Weave`), which
// is why there is no world name anywhere below:
//
//   * STRATA (weave 0) — lanes indexed on DISTANCE FROM THE PAGE COLUMN, so
//     the contours gather around the writing page and mirror across it. Each
//     lane is FILLED at its own seeded value and its boundary carries the torn
//     deckle tint. Paperbark's ground.
//   * FIBRES (weave 1) — lanes indexed on screen y, drawn as thin translucent
//     STROKES with seeded dropouts, plus a sparser diagonal vein family.
//     Reusable, currently unassigned.
//
// DIALS: params.x = the lane pitch (`period_px`), params.y = the wander
// amplitude (`wander_px`), params.z = the ONE coverage/contrast multiplier
// (`density`), params.w = the weave.
//
// `density == 0.0` collapses BOTH profiles to their flat ground EXACTLY — the
// lane values converge on DECKLE_MID and every tint drops out. That is not a
// nicety: it is the differential oracle every pixel law for this ground
// measures against (item 86's `mark_field` idiom), so the gradient, dither and
// 8-bit quantization cancel and what remains is the material alone.

// The lane-value midpoint `density == 0` flattens to, and the value half-range
// per unit density. Together they reproduce the authored strata spread
// (`mix(0.22, 0.70, seed)` at density 0.20) while keeping the flat-at-zero
// property above. Mirrored by `theme::model`'s DECKLE_* consts.
const DECKLE_MID: f32 = 0.46;
const DECKLE_SPREAD_GAIN: f32 = 1.2;
// The deckle EDGE, as a fraction of a lane — a torn boundary, not a rule.
const DECKLE_EDGE_LO: f32 = 0.015;
const DECKLE_EDGE_HI: f32 = 0.075;
// The wander profile: a coarse tear plus a finer one that leans with distance
// from the page, so the lanes never repeat exactly down a tall margin. FIXED
// family character (the Waves-tier precedent) — only its AMPLITUDE is authored.
const DECKLE_WANDER_FREQ: f32 = 0.017;
const DECKLE_WANDER_FINE_FREQ: f32 = 0.053;
const DECKLE_WANDER_FINE_FRAC: f32 = 0.34615385;
const DECKLE_WANDER_SKEW: f32 = 0.011;
// FIBRES: stroke frequency, half-width ramp, the seeded dropout gate, and the
// two coverage gains that ride the shared `density` dial.
const DECKLE_FIBRE_FREQ: f32 = 0.006;
const DECKLE_FIBRE_HALF_LO: f32 = 0.7;
const DECKLE_FIBRE_HALF_HI: f32 = 2.2;
const DECKLE_FIBRE_KEEP: f32 = 0.34;
const DECKLE_FIBRE_LIFT_GAIN: f32 = 1.9;
const DECKLE_FIBRE_GROUND: f32 = 0.10;
const DECKLE_FIBRE_WANDER_BASE: f32 = 0.36;
const DECKLE_FIBRE_WANDER_SEED: f32 = 0.64;
const DECKLE_VEIN_PITCH_FRAC: f32 = 1.6590909;
const DECKLE_VEIN_SKEW: f32 = 0.24;
const DECKLE_VEIN_KEEP: f32 = 0.64;
const DECKLE_VEIN_GAIN: f32 = 1.3;
const DECKLE_VEIN_HALF_LO: f32 = 0.5;
const DECKLE_VEIN_HALF_HI: f32 = 1.45;
const DECKLE_TAU: f32 = 6.2831855;
// The lane pitch FLOOR. The deckle edge is a FRACTION of a lane, so below this
// the boundary falls under a pixel and the field aliases into moire instead of
// reading as paper. Enforced HERE (a property of the shader, not of the dial
// pair — item 89's abutment lesson) and mirrored by `theme::DECKLE_MIN_PERIOD_PX`.
const DECKLE_MIN_PITCH_PX: f32 = 40.0;
// The weave threshold `theme::Weave::mode` writes either side of.
const DECKLE_WEAVE_FIBRES: f32 = 0.5;

// Legacy mutation arm only: distance to the page edge is precisely the
// border-decoration behaviour item 175 rejects. `deckle_strata` chooses the
// stable viewport coordinate by default; this stays named so the pixel law can
// restore the defect and prove it goes red.
fn deckle_page_distance(px: vec2<f32>) -> f32 {
    if (px.x > g.col_left + g.col_w) {
        return px.x - (g.col_left + g.col_w);
    }
    return g.col_left - px.x;
}

// The Room/viewport owner: a page-width drag moves only the opaque mask above
// this field. The viewport centre is stable under page-width dragging and under
// the adaptive-column shift, so an exposed screen point cannot translate,
// stretch, reseed, or reflow its paper contours.
fn deckle_viewport_distance(px: vec2<f32>) -> f32 {
    return abs(px.x - g.viewport.x * 0.5);
}

fn deckle_strata(px: vec2<f32>, pitch: f32, wander: f32, density: f32) -> vec3<f32> {
    let d = select(deckle_viewport_distance(px), deckle_page_distance(px), g.params.w >= 1.5);
    let torn = sin(px.y * DECKLE_WANDER_FREQ) * wander
        + sin(px.y * DECKLE_WANDER_FINE_FREQ + d * DECKLE_WANDER_SKEW)
            * wander * DECKLE_WANDER_FINE_FRAC;
    let q = max(d + torn, 0.0) / pitch;
    let lane = fract(q);
    let seed = hash21(vec2<f32>(floor(q), 11.0));
    let value = clamp(
        DECKLE_MID + (seed - 0.5) * 2.0 * density * DECKLE_SPREAD_GAIN,
        0.0,
        1.0,
    );
    let edge = 1.0 - smoothstep(DECKLE_EDGE_LO, DECKLE_EDGE_HI, min(lane, 1.0 - lane));
    let strata = mix(g.c_from.rgb, g.c_to.rgb, value);
    return mix(strata, g.c_pat.rgb, edge * density);
}

fn deckle_fibres(px: vec2<f32>, pitch: f32, wander: f32, density: f32) -> vec3<f32> {
    let row = floor(px.y / pitch);
    let seed = hash21(vec2<f32>(row, 23.0));
    let center = (row + 0.18 + seed * 0.64) * pitch;
    let fibre_y = center + sin(px.x * DECKLE_FIBRE_FREQ + seed * DECKLE_TAU)
        * wander * (DECKLE_FIBRE_WANDER_BASE + seed * DECKLE_FIBRE_WANDER_SEED);
    let fibre = (1.0 - smoothstep(DECKLE_FIBRE_HALF_LO, DECKLE_FIBRE_HALF_HI, abs(px.y - fibre_y)))
        * step(DECKLE_FIBRE_KEEP, seed);

    let vein_pitch = pitch * DECKLE_VEIN_PITCH_FRAC;
    let along = px.y + px.x * DECKLE_VEIN_SKEW;
    let vein_row = floor(along / vein_pitch);
    let vein_seed = hash21(vec2<f32>(vein_row, 47.0));
    let vein_center = (vein_row + 0.28 + vein_seed * 0.44) * vein_pitch;
    let vein = (1.0 - smoothstep(DECKLE_VEIN_HALF_LO, DECKLE_VEIN_HALF_HI, abs(along - vein_center)))
        * step(DECKLE_VEIN_KEEP, vein_seed);

    let paper = mix(g.c_from.rgb, g.c_to.rgb, DECKLE_FIBRE_GROUND);
    let lifted = mix(paper, g.c_to.rgb, fibre * density * DECKLE_FIBRE_LIFT_GAIN);
    return mix(lifted, g.c_pat.rgb, vein * density * DECKLE_VEIN_GAIN);
}

fn deckle_rgb(px: vec2<f32>) -> vec3<f32> {
    let pitch = max(g.params.x, DECKLE_MIN_PITCH_PX);
    let wander = g.params.y;
    let density = g.params.z;
    if (g.params.w >= DECKLE_WEAVE_FIBRES && g.params.w < 1.5) {
        return deckle_fibres(px, pitch, wander, density);
    }
    return deckle_strata(px, pitch, wander, density);
}

// ITEM 69 FOLLOW-UP (audit finding): the plain corner-to-corner projection
// below reads fine at a NARROW or SQUARE canvas, but at a wide CANONICAL
// aspect (~1200x800) the projection is dominated by the width term, so a
// fixed-pixel margin sliver near x=0 or x=viewport.x sees only a sliver of
// the full [0,1] range and NEVER crosses a boundary — both margins degrade to
// one flat tone, even though the mid-field scanline still shows all three
// bands. Re-anchoring the coordinate on the VIEWPORT'S OWN CENTER (instead of
// the top-left corner) makes the left/right margins mirror-symmetric, and
// `BANDS_MARGIN_SPAN` widens the span past the plain diagonal so each
// margin's crossing lands back inside the visible canvas at canonical size —
// the same crop/scale behavior the type's own doc promises for responsive
// views. Tuned so canonical's margins each catch a crossing while the
// existing mid-field ">15% per band" law still holds comfortably.
const BANDS_MARGIN_SPAN: f32 = 1.35;
fn bands_rgb(px: vec2<f32>) -> vec3<f32> {
    let a = g.params.y;
    let dir = vec2<f32>(cos(a), sin(a));
    let center = g.viewport * 0.5;
    let extent = max(dot(g.viewport, dir), 1.0) * BANDS_MARGIN_SPAN;
    let t = clamp(dot(px - center, dir) / extent + 0.5, 0.0, 1.0);
    let aa = 1.5 / extent;
    return tri_tone_mix(t, 1.0 / 3.0, 2.0 / 3.0, aa);
}

// --- 6: WAVES — THREE stacked, NON-OVERLAPPING shallow wave tiers (wide
// scalloped crests, horizontally phase-offset tier-to-tier so they read as
// layered swells, never a grid). Tier geometry (amplitude/wavelength/phase) is
// a FIXED constant, never per-world data — every `Waves` world shares this
// exact shape (only the three tones differ). Pure function of `px` PLUS one
// scalar, `g.drift` — the item-87 phase DRIFT (radians), `0.0` at rest so
// the settled/headless-capture render is byte-identical to the pre-drift
// shape. ---
//
// The viewport height splits into thirds; each of the two boundaries between
// tiers is that third's y plus a sine wobble in x (a "scallop"), with tier 2's
// boundary carrying a DIFFERENT phase than tier 1's so the two crest-lines
// visibly drift apart ("layer") instead of tracking each other like a grid.
// The wobble amplitude is held well under a third of the viewport height for
// any real window, so the two boundaries never cross — the three tiers stay
// NON-OVERLAPPING by construction (drift never changes the amplitude, only a
// crest's x-position, so this bound holds at every phase too).
//
// DRIFT (item 87): `b1`'s phase ADVANCES by `drift`, `b2`'s RETARDS by the
// SAME amount — equal magnitude, opposite sign (see `src/background.rs`'s
// module doc for the full derivation of why opposite-sign is the one choice
// that avoids the whole field sliding as a single rigid "sheet": a same-sign
// drift on both curves is provably an exact horizontal translation of the
// entire composition, including the middle tier, so it produces ZERO relative
// motion between tiers). Under opposite signs each OUTER tier (top/bottom)
// sweeps with its own single boundary curve's sign, while the MIDDLE tier —
// bounded by both — visibly shears/breathes counter to them: the sea reads as
// independently layered swells, never a sheet translating behind the margin.
const WAVE_AMP: f32 = 22.0;
const WAVE_FREQ: f32 = 0.024166097; // 2*pi / 260px — wide, shallow scallops
const WAVE_PHASE_1: f32 = 0.0;
const WAVE_PHASE_2: f32 = 2.4;
fn waves_rgb(px: vec2<f32>) -> vec3<f32> {
    let drift = g.drift;
    let b1 = g.viewport.y * (1.0 / 3.0) + WAVE_AMP * sin(px.x * WAVE_FREQ + WAVE_PHASE_1 + drift);
    let b2 = g.viewport.y * (2.0 / 3.0) + WAVE_AMP * sin(px.x * WAVE_FREQ + WAVE_PHASE_2 - drift);
    return tri_tone_mix(px.y, b1, b2, 1.5);
}

// BANDING KILL — the classic 8x8 ordered (Bayer) dither matrix, values 0..64.
// A pure function of PIXEL POSITION alone (no time, no random), so the headless
// capture stays deterministic. Rust mirror + full derivation notes:
// `src/render/dither.rs` (kept in sync by hand — see that file's module doc for
// why a small cross-language duplication is the accepted answer here).
var<private> BAYER8: array<u32, 64> = array<u32, 64>(
     0u, 32u,  8u, 40u,  2u, 34u, 10u, 42u,
    48u, 16u, 56u, 24u, 50u, 18u, 58u, 26u,
    12u, 44u,  4u, 36u, 14u, 46u,  6u, 38u,
    60u, 28u, 52u, 20u, 62u, 30u, 54u, 22u,
     3u, 35u, 11u, 43u,  1u, 33u,  9u, 41u,
    51u, 19u, 59u, 27u, 49u, 17u, 57u, 25u,
    15u, 47u,  7u, 39u, 13u, 45u,  5u, 37u,
    63u, 31u, 55u, 23u, 61u, 29u, 53u, 21u,
);

// The Bayer threshold at pixel `px`, normalized to [0,1) — tiles every 8px.
fn bayer_threshold01(px: vec2<f32>) -> f32 {
    let x = u32(floor(px.x)) % 8u;
    let y = u32(floor(px.y)) % 8u;
    return f32(BAYER8[y * 8u + x]) / 64.0;
}

// sRGB transfer function (encode: linear -> sRGB, decode: sRGB -> linear),
// applied per-channel. NEEDED for the dither below: the render target is
// `Rgba8UnormSrgb`, so the GPU auto-encodes this shader's LINEAR output to
// sRGB and quantizes THAT to 8 bits on write — a dither meant to land "at
// ±half an 8-bit step before quantization" must therefore perturb the
// SRGB-ENCODED value (the space that's actually rounded to a byte), not the
// linear one: the sRGB curve is steep near black, so a fixed linear-space
// nudge would land as a much LARGER swing in the shadows than in the
// highlights (confirmed empirically: it broke the round's own ≤1-LSB law —
// see `render::tests::dither`). Encoding here, dithering, then decoding back
// to linear before `return` makes the GPU's own re-encode land exactly where
// intended, channel by channel.
fn srgb_encode1(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return c * 12.92;
    }
    return 1.055 * pow(c, 1.0 / 2.4) - 0.055;
}
fn srgb_decode1(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Inside the page column: punch a hole so the flat base_100 clear shows.
    if (in.px.x >= g.col_left && in.px.x < g.col_left + g.col_w) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // 5/6: BANDS / WAVES compute their own final rgb directly (three opaque
    // authored tones ARE the field) — bypass the gradient/dither/pattern-
    // overlay pipeline below entirely, which every OTHER ground still takes
    // unchanged (byte-identical).
    if (g.shader == 5u) {
        return vec4<f32>(bands_rgb(in.px), 1.0);
    }
    if (g.shader == 6u) {
        return vec4<f32>(waves_rgb(in.px), 1.0);
    }
    if (g.shader == 8u) { return vec4<f32>(organic_rgb(in.px), 1.0); }
    if (g.shader == 9u) { return vec4<f32>(deckle_rgb(in.px), 1.0); }
    // Margin: evaluate the gradient along `dir`. UV is centered so the diagonal
    // worlds read symmetrically; t is clamped to [0,1].
    let uv = in.px / g.viewport;
    let t = clamp(dot(uv - vec2<f32>(0.5, 0.5), g.dir) + 0.5, 0.0, 1.0);
    var rgb = mix(g.c_from.rgb, g.c_to.rgb, t);
    let a = mix(g.c_from.a, g.c_to.a, t);
    // BANDING KILL: an ordered ±half-8-bit-step dither, added in sRGB-ENCODED
    // space (see `srgb_encode1`'s doc for why) BEFORE the GPU quantizes it to
    // the 8-bit render target — imperceptible as its own texture, breaks up
    // the visible banding a smooth `mix()` produces across a wide gradient. A
    // FLAT gradient (from == to, e.g. Wagtail's one-bit background) is an
    // EXACT no-op — any nonzero nudge on a pure #000000/#FFFFFF would round
    // to a forbidden third value under the one-bit law, so this is gated,
    // not merely small.
    let flat = all(g.c_from.rgb == g.c_to.rgb) && (g.c_from.a == g.c_to.a);
    if (!flat) {
        let offset = (bayer_threshold01(in.px) - 0.5) / 255.0;
        let srgb = vec3<f32>(srgb_encode1(rgb.x), srgb_encode1(rgb.y), srgb_encode1(rgb.z));
        let dithered = clamp(srgb + vec3<f32>(offset, offset, offset), vec3<f32>(0.0), vec3<f32>(1.0));
        rgb = vec3<f32>(srgb_decode1(dithered.x), srgb_decode1(dithered.y), srgb_decode1(dithered.z));
    }
    // Overlay the procedural pattern: mix the dim tint in at a low coverage so the
    // marks whisper and the page column stays the clear figure.
    let cov = pattern_coverage(in.px) * g.c_pat.a;
    rgb = mix(rgb, g.c_pat.rgb, cov);
    return vec4<f32>(rgb, a);
}
