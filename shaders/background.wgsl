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
    // 3=pinstripe, 4=stripes, 5=bands, 6=waves, 7=zigzag, 8=organic.
    // 9 is the isolated, disposable Paperbark A–E trial and is never returned
    // by `Background::shader_id` or enrolled in the shipping world roster.
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
    // shader 7's extra coverage multiplier `density`. All four are 0 for
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

// 8: cut-paper blobs. Three differently-offset rounded cell fields make
// large masses, islands, and droplets; subtracting a small inner field leaves
// occasional holes. The only time input is the shared, slow ambient phase.
fn organic_rgb(px: vec2<f32>) -> vec3<f32> {
    let s = max(g.params.x, 32.0);
    let d = clamp(g.params.y, 0.0, 1.0);
    let drift = vec2<f32>(
        sin(g.drift) * max(s * ORGANIC_DRIFT_X_FRAC, ORGANIC_DRIFT_MIN_X_PX),
        cos(g.drift * 0.73) * max(s * ORGANIC_DRIFT_Y_FRAC, ORGANIC_DRIFT_MIN_Y_PX),
    );
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

// 9: DISPOSABLE PAPERBARK MATERIAL STUDY. `params.x` selects exactly one of
// five fully-static treatments. All five consume the same three provisional
// material tones and the same page-column mask; no clock, texture, asset, or
// world identity enters this shader. This whole branch is deleted after the
// user's A–E choice rather than becoming five permanent backgrounds.
fn paperbark_broad_sheets(px: vec2<f32>) -> vec3<f32> {
    let warp = sin(px.x * 0.0053) * 19.0 + sin(px.x * 0.017 + 1.7) * 6.0;
    let q = (px.y + warp) / 190.0;
    let lane = fract(q);
    let body = smoothstep(0.035, 0.14, lane) * (1.0 - smoothstep(0.72, 0.97, lane));
    let lift = 1.0 - smoothstep(0.018, 0.085, lane);
    let alternating = mix(0.45, 0.72, hash21(vec2<f32>(floor(q), 3.0)));
    let sheet = mix(g.c_from.rgb, g.c_to.rgb, body * alternating);
    return mix(sheet, g.c_pat.rgb, lift * 0.22);
}

fn paperbark_deckled_strata(px: vec2<f32>) -> vec3<f32> {
    var distance = g.col_left - px.x;
    if (px.x > g.col_left + g.col_w) {
        distance = px.x - (g.col_left + g.col_w);
    }
    let torn = sin(px.y * 0.017) * 13.0 + sin(px.y * 0.053 + distance * 0.011) * 4.5;
    let q = max(distance + torn, 0.0) / 94.0;
    let lane = fract(q);
    let layer = floor(q);
    let value = mix(0.22, 0.70, hash21(vec2<f32>(layer, 11.0)));
    let edge = 1.0 - smoothstep(0.015, 0.075, min(lane, 1.0 - lane));
    let strata = mix(g.c_from.rgb, g.c_to.rgb, value);
    return mix(strata, g.c_pat.rgb, edge * 0.20);
}

fn paperbark_loose_fibres(px: vec2<f32>) -> vec3<f32> {
    let row = floor(px.y / 88.0);
    let seed = hash21(vec2<f32>(row, 23.0));
    let center = (row + 0.18 + seed * 0.64) * 88.0;
    let fibre_y = center + sin(px.x * 0.006 + seed * 6.28318) * (9.0 + seed * 16.0);
    let long_fibre = (1.0 - smoothstep(0.7, 2.2, abs(px.y - fibre_y))) * step(0.34, seed);

    let diagonal_coord = px.y + px.x * 0.24;
    let vein_row = floor(diagonal_coord / 146.0);
    let vein_seed = hash21(vec2<f32>(vein_row, 47.0));
    let vein_center = (vein_row + 0.28 + vein_seed * 0.44) * 146.0;
    let vein = (1.0 - smoothstep(0.5, 1.45, abs(diagonal_coord - vein_center)))
        * step(0.64, vein_seed);

    let paper = mix(g.c_from.rgb, g.c_to.rgb, 0.10);
    let translucent = mix(paper, g.c_to.rgb, long_fibre * 0.38);
    return mix(translucent, g.c_pat.rgb, vein * 0.26);
}

fn paperbark_relief_print(px: vec2<f32>) -> vec3<f32> {
    let cell_size = vec2<f32>(230.0, 176.0);
    let cell = floor(px / cell_size);
    let stagger = vec2<f32>(select(0.0, 0.47, i32(cell.y) % 2 == 1), 0.0);
    let local = fract(px / cell_size + stagger) - vec2<f32>(0.5);
    let jitter = vec2<f32>(
        hash21(cell + vec2<f32>(3.0, 7.0)),
        hash21(cell + vec2<f32>(13.0, 2.0)),
    ) - vec2<f32>(0.5);
    let p = local - jitter * 0.11;
    let block_distance = max(abs(p.x) * 0.78, abs(p.y));
    let block = 1.0 - smoothstep(0.31, 0.38, block_distance);
    let grain = hash21(floor(px / 6.0) + cell * 5.0);
    let dry = mix(0.48, 1.0, step(0.22, grain));
    let print = block * dry;
    let edge = smoothstep(0.28, 0.33, block_distance)
        * (1.0 - smoothstep(0.37, 0.42, block_distance));
    let ink = mix(g.c_from.rgb, g.c_to.rgb, print * 0.78);
    return mix(ink, g.c_pat.rgb, edge * 0.16);
}

fn paperbark_peeling_curls(px: vec2<f32>) -> vec3<f32> {
    let cell_size = vec2<f32>(310.0, 184.0);
    let cell = floor(px / cell_size);
    let local = fract(px / cell_size) - vec2<f32>(0.5);
    let seed = hash21(cell + vec2<f32>(29.0, 5.0));
    let x = local.x + (seed - 0.5) * 0.14;
    let arch = -0.17 * (1.0 - clamp((x / 0.40) * (x / 0.40), 0.0, 1.0));
    let curve = arch + sin(x * 8.0 + seed * 2.2) * 0.035 + (seed - 0.5) * 0.20;
    let bounded = 1.0 - smoothstep(0.37, 0.44, abs(x));
    let ribbon = (1.0 - smoothstep(0.016, 0.034, abs(local.y - curve))) * bounded;
    let shadow = (1.0 - smoothstep(0.022, 0.052, abs(local.y - curve - 0.050))) * bounded;
    let tip_center = vec2<f32>(select(-0.39, 0.39, seed > 0.5), curve);
    let tip = 1.0 - smoothstep(0.025, 0.060, length(local - tip_center));
    let paper = mix(g.c_from.rgb, g.c_to.rgb, ribbon * 0.72 + tip * 0.48);
    return mix(paper, g.c_pat.rgb, shadow * (1.0 - ribbon) * 0.20);
}

fn paperbark_rgb(px: vec2<f32>) -> vec3<f32> {
    let profile = u32(round(clamp(g.params.x, 0.0, 4.0)));
    if (profile == 0u) { return paperbark_broad_sheets(px); }
    if (profile == 1u) { return paperbark_deckled_strata(px); }
    if (profile == 2u) { return paperbark_loose_fibres(px); }
    if (profile == 3u) { return paperbark_relief_print(px); }
    return paperbark_peeling_curls(px);
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
    if (g.shader == 9u) { return vec4<f32>(paperbark_rgb(in.px), 1.0); }
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
