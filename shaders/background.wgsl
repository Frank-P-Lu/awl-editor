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
// pipeline). The ONE exception is `drift` — Bombora's WAVES
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
    // Procedural margin ground: 0=plain gradient, 1=dots, 2=VACANT (the
    // retired scattered-star ground; an unissued id draws the plain gradient),
    // 3=pinstripe, 4=stripes, 5=bands, 6=waves, 7=zigzag, 8=organic,
    // 9=deckle, 10=warped-grid. Matches `Background::shader_id` in
    // src/theme/ground.rs, which explains why a retired id is vacated rather
    // than reused: every number here is a WIRE value, so renumbering repaints
    // worlds.
    shader: u32,
    // WAVES phase-drift, in radians — a DEDICATED scalar in what was
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
    // chevron repeat wavelength `period_px`; params.y = the
    // Stripes/Bands angle (radians) OR shader 7's own chevron travel angle;
    // params.z = shader 7's chevron amplitude `amplitude_px`; params.w =
    // shader 7's extra coverage multiplier `density`. Shader 9 (Deckle, item
    // 158) reads all four with its OWN meanings — lane pitch / wander
    // amplitude / density / weave; see `deckle_rgb`. Shader 8 (Organic) reads
    // x/y alone — it once read params.z as a theme-owned ARRANGEMENT scalar,
    // but the ground draws ONE arrangement now and that slot is inert; see
    // `organic_rgb`. All four are 0 for
    // every ground this round didn't touch, so those grounds take their
    // exact original code path. (Waves' item-87 drift is NOT here — it rides
    // the dedicated `drift` slot above.) Shader 10 (WarpedGrid) reads
    // x/y/w as projected minor-cell spacing / coverage density / framing.
    params: vec4<f32>,
    // Warped-grid travel in minor cells, resolved on the host. Zero for every
    // other ground. Must byte-match `Globals.warp_travel` in src/background.rs.
    warp_travel: f32,
    // PHYSICAL PIXELS PER LOGICAL PIXEL (the display's device ratio:
    // 1.0 on a 1:1 screen, 2.0 on a Retina one). The host uploads it from the
    // SAME `scale_factor` the window reports (`--capture-dpi` headlessly);
    // deliberately NOT folded with the user's text zoom, because the ground is a
    // property of the Room, not of the type size. Every COMPOSITION quantity
    // below is divided through this before it is used; every SAMPLING quantity
    // is not. See `to_logical` / `sampling_feather` and the classification table
    // in `src/theme/ground.rs`, which is the authority this file mirrors.
    scale: f32,
    // Organic's own per-frame breathe phase, in CYCLES (the raw
    // shared-clock phase, NOT radians — unlike `drift` above, nothing
    // pre-multiplies this by TAU). `0.0` for every non-Organic ground and
    // every settled/headless frame. Read directly by `organic_finds_rgb`'s
    // companion (`kind_b`) value-breathe, whose envelope shape matches
    // `stars.rs:185`'s `rate * phase / LAVA_LOOP_CYCLES`. Must byte-match
    // `Globals.organic_phase` in src/background.rs. Replaces the field
    // TRANSLATION `organic_rgb` used to compute from `drift` —
    // deleted outright, so the ground no longer translates at all.
    organic_phase: f32,
    // std140 tail padding — a uniform struct is rounded up to its 16-byte
    // alignment, and the Rust mirror must allocate the same bytes. A LONE
    // SCALAR, never folded into a `vec2`/`vec3`: a vec2 carries 8-byte
    // alignment and a vec3 16-byte, either of which would move this tail and
    // the struct's total size, and wgpu validates the binding against that
    // size (it does, by name, the moment this drifts).
    pad0: f32,
};

@group(0) @binding(0) var<uniform> g: Globals;

// --- THE TWO COORDINATE SPACES OF A PROCEDURAL GROUND ---
//
// A ground carries two structurally different classes of authored number, and
// before this item both were physical pixels by accident of the coordinate the
// fragment shader happens to run in:
//
//   * COMPOSITION — a cell, a pitch, a mark size, a wander, a falloff reach.
//     These describe WHAT THE USER SEES: how many elements a margin holds and
//     how large each reads. They must live in LOGICAL pixels, so two matched
//     logical canvases show the same world composition at 1x and at 2x. Every
//     one of them is evaluated against `to_logical(px)`.
//   * SAMPLING — an antialias feather, a dither cell. These describe HOW THE
//     SAMPLE GRID RESOLVES that composition, so they are properties of the
//     device pixel and must stay PHYSICAL: a 2x display resolves the SAME
//     composition more finely, which is exactly the benefit of the density.
//     A feather used inside logical-space math is converted back through
//     `sampling_feather`, so its width on the glass never moves.
//
// A feather is NOT a small composition quantity: converting one would make a
// crisp edge blurrier on a better display, which is the opposite of the fix.
// `src/theme/ground.rs`'s `Background::authored_quantities` names every one of
// these numbers, its class, and why; a grep-law holds the two in lockstep.
fn dpr() -> f32 {
    return max(g.scale, 0.01);
}
// A fragment's position in LOGICAL pixels — the space every composition
// quantity below is authored in.
fn to_logical(p: vec2<f32>) -> vec2<f32> {
    return p / dpr();
}
// A SAMPLING feather authored in PHYSICAL pixels, expressed in the logical
// units the composition math runs in. Its width on the glass is `physical_px`
// at every device ratio — that invariance is the whole point of the call.
fn sampling_feather(physical_px: f32) -> f32 {
    return physical_px / dpr();
}
fn viewport_l() -> vec2<f32> {
    return g.viewport / dpr();
}
fn col_left_l() -> f32 {
    return g.col_left / dpr();
}
fn col_w_l() -> f32 {
    return g.col_w / dpr();
}

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
// so every seeded field built on it is byte-stable across captures.
fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

// Proximity to the PAGE-COLUMN boundary as an intensity in [0,1]: 1.0 right at
// the page edge, decaying outward into the margin (exp falloff). Drives the
// Stripes band + the proximity-scaled Dots — the "play area radiates into the
// ground" feel. Pure pixel math, no time. COMPOSITION (logical px): how far the
// band radiates is a thing the eye measures against the margin, not against the
// sample grid.
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

// `px` is LOGICAL, so the column bounds it is measured against are
// taken in logical units too.
fn edge_intensity(px: vec2<f32>) -> f32 {
    var d = 0.0;
    if (px.x < col_left_l()) {
        d = col_left_l() - px.x;
    } else {
        d = px.x - (col_left_l() + col_w_l());
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
    if (px.x < col_left_l()) {
        d = col_left_l() - px.x;
        span = max(col_left_l(), 1.0);
    } else {
        d = px.x - (col_left_l() + col_w_l());
        span = max(viewport_l().x - (col_left_l() + col_w_l()), 1.0);
    }
    return clamp(1.0 - d / span, 0.0, 1.0);
}

// Coverage [0,1] of the assigned margin ground at LOGICAL pixel `px`
// — the caller divides through the device ratio, so every cell/period/mark size
// below is a logical quantity and the composition is the same at 1x and 2x).
// All grounds are pure functions of position — STATIC, no time. Tuned to
// whisper.
fn pattern_coverage(px: vec2<f32>) -> f32 {
    // --- 1: DOTS — a grid of round dots; `params.x` flips proximity scaling. ---
    if (g.shader == 1u) {
        // COMPOSITION: the dot lattice pitch.
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
            // COMPOSITION: the dot's own radius, in logical px. SAMPLING: the
            // 0.9px skirt that resolves its rim.
            let radius = mix(0.85, 3.0, p); // ~28% far -> a full fat dot at the edge
            let dot = 1.0 - smoothstep(radius, radius + sampling_feather(0.9), d);
            let alpha = mix(0.5, 1.0, p);   // gentle falloff; brightest hugging the page
            return dot * alpha;
        }
        // edge=false: UNIFORM ~1.4 LOGICAL px dots (COMPOSITION) with a 1
        // PHYSICAL px feather (SAMPLING) — at 1x this is the original
        // `smoothstep(1.4, 2.4, d)`, byte for byte.
        return 1.0 - smoothstep(1.4, 1.4 + sampling_feather(1.0), d);
    }
    // --- 4: STRIPES — diagonal stripes in a bright band hugging the page edge,
    // dissolving outward into the gradient (the N++ look). The band peaks at the
    // boundary (edge_intensity) and the stripes run perpendicular to `dir`. ---
    if (g.shader == 4u) {
        let a = g.params.y;
        // Coordinate across the stripes (perpendicular bands give the diagonal look).
        let coord = px.x * cos(a) + px.y * sin(a);
        // COMPOSITION: the stripe pitch and the stripe's own half-width.
        // SAMPLING: the 1.5px skirt that resolves its edge.
        let period = 13.0;
        let f = abs(fract(coord / period) - 0.5) * period; // logical px to a stripe
        let line = 1.0 - smoothstep(2.0, 2.0 + sampling_feather(1.5), f);
        return line * edge_intensity(px);                  // dissolve outward
    }
    // --- 3: PINSTRIPE — fine vertical parallel lines (ledger / print rules). ---
    if (g.shader == 3u) {
        // COMPOSITION: the rule pitch and half-width. SAMPLING: the 0.7px skirt.
        let period = 9.0;
        let x = abs(fract(px.x / period) - 0.5) * period; // logical px to a line
        return 1.0 - smoothstep(0.5, 0.5 + sampling_feather(0.7), x);
    }
    // --- 2: VACANT. The scattered-star ground that held this id is retired;
    // the id stays unissued rather than being handed to a neighbour, because
    // every id below is a wire value (see `Background::shader_id`). ---
    // --- 7: ZIGZAG — a TILED field of repeating chevron ("V") rows,
    // whisper-composited over the gradient like Dots/Pinstripe;
    // FIELD-tiled. Four independently authored dials (see
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
    // Every one of Zigzag's numbers is COMPOSITION and therefore
    // logical — `period`, `amp`, and the derived `thickness`. The stroke's soft
    // edge is `thickness * 0.6 .. thickness`, a PROPORTION of the ribbon rather
    // than a fixed skirt, so it is part of the drawn profile and not a sampling
    // feather; and the abutment rule folds `thickness` straight into the row
    // PITCH, so a thickness in physical px would make the field's own pitch
    // density-dependent — the exact defect this item closes.
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
        // SAME fold `jagged_wave_band` (`shaders/selection.wgsl`,
        // retired) used: `fract` + a fold gives the sharp "jagged" corners a
        // chevron needs.
        let tri = abs(fract(rx / period) * 2.0 - 1.0) * 2.0 - 1.0;
        let center = tri * amp;
        let thickness = max(amp * ZIGZAG_STROKE_FRAC, 1.2);
        // THE FIELD FOLD. `center` is a function of `rx` ALONE, so
        // `abs(ry - center)` describes ONE continuous
        // chevron LINE embedded in the plane: teeth repeat along travel, but
        // nothing repeats it ACROSS travel, so a tall margin showed one
        // wandering stroke with large blank areas and a taller window only
        // let that single stroke travel further before running off-canvas.
        // Folding `(ry - center)` through the row pitch turns that one line
        // into the INFINITE FAMILY `center + k * row_h` — a genuinely tiled
        // Mario-like zigzag field that covers any viewport, at any height,
        // with the same row rhythm.
        //
        // THE ABUTMENT RULE, and why the pitch is NOT `period`. A first cut
        // set `row_h = period` (a square lattice in the
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
        // period/amplitude ratio or on the angle. (The one bound:
        // a VIEWPORT sees a rotated rectangle of that plane, so it inherits
        // the guarantee only while it holds a whole tooth of travel; a canvas
        // narrower than one `period` never sweeps the full excursion. See
        // `Background::zigzag_row_pitch_px`'s doc comment — no shipping
        // world's tooth is within 4x of that regime.) It also retires
        // the row-collision clamp — abutting rows cannot smear together
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

// THE FIELD TRANSLATION IS GONE. The former `drift` vec2 (a
// `sin(g.drift)`/`cos(g.drift * 0.73)` field pan, both terms) is deleted
// outright, not retuned: the `0.73` term was DISCONTINUOUS across the shared
// clock's wrap (`cos(0.73*TAU) != cos(0.0)`, a 1.125-normalised-unit jump —
// ~22px vertically at Bowerbird's `scale_px`), and the user's own design call
// (2026-08-04) was that a bower's ARRANGEMENT reads better held still than
// panned: "objects deliberately placed and then left alone." The ground's one
// remaining ambient input is the FINDS companion's own value breathe, below.
//
// --- FINDS: the COLLECTED-TREASURE arrangement, and the ONLY one this ground
// draws. One cell draws one deliberately arranged collection of
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
// COMPANION_HI * 1.5551)` of a cell from its centre — the 1.5551 is the
// triangle's circumradius per unit nominal radius, the largest reach of the
// three kinds. Raising `ANCHOR_HI` (below) for composition growth raises this
// worst-case reach from ~0.44 to ~0.499 —
// still inside the half cell, so neighbouring collections still never merge,
// but the margin is now thin rather than generous, so this claim is a
// SWEPT PIXEL LAW (`render::tests::bowerbird_finds_item176`'s three-role
// hierarchy check — a merge would fuse two collections' ink and fail it), not
// a comment asserted on the strength of the arithmetic alone.
const FINDS_SQUARE_HALF: f32 = 0.8862269; // (2a)^2 == pi*r^2
const FINDS_TRI_HALF_SIDE: f32 = 1.3468; // sqrt(3)*h^2 == pi*r^2
const FINDS_TRI_INRADIUS: f32 = 0.7776; // the smallest inradius of the three kinds
// The anchor's own nominal-radius range is 1.15x its former
// values, ONE hierarchy-preserving move: the companion and cut-out are both
// authored as FRACTIONS of the anchor's radius (`r_b`/`r_c` below), and their
// offsets are fractions of the anchor's radius too, so scaling the anchor
// alone carries the whole collection — all three roles, and their spacing —
// up by the same 15% without retuning a single ratio. Measured effect: about
// 15% more linear size (~32% more inked area) per collection, exactly the
// "more physical presence" the user asked for, with the role ordering and
// every overlap/enclosure guarantee in the comment above untouched (they are
// stated in ratios, which this change never touches).
const FINDS_ANCHOR_LO: f32 = 0.1725;
const FINDS_ANCHOR_HI: f32 = 0.22425;
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
// The WINNING hash's threshold, not a per-cell rate (see
// `finds_is_local_min` below). Raised from the item-176 0.10 so the
// decorrelated mechanism still draws roughly one breathing cell in ten:
// a cell's own hash is one of 9 i.i.d. draws (itself plus its full
// neighbourhood), so P(cell empty) = P(its draw is both the neighbourhood's
// minimum AND under this threshold) = (1-(1-t)^9)/9, solved for the
// item-176 target of 0.10 at t ~= 0.226.
const FINDS_DROPOUT: f32 = 0.226;
const FINDS_TAU: f32 = 6.2831855;
// The feather half-width, in PHYSICAL pixels. Crisp is the whole point, so
// this is a fixed sub-pixel skirt rather than a fraction of a cell: the same
// hard edge resolves without stair-stepping at 1x and at 2x, and at any cell
// scale a future Organic world might author.
//
// THIS IS THE CANONICAL SAMPLING QUANTITY. Every
// composition number around it became logical; this one stays physical, and
// `finds_fill` converts it INTO the logical space the SDFs are evaluated in so
// that its width on the glass is 0.75px at 1x and 0.75px at 2x. Converting it
// would make the same edge blurrier on a better display — the opposite of what
// a feather is for.
const FINDS_EDGE_AA_PX: f32 = 0.75;
// FINDS declares its own cell FLOOR, the way Deckle declares a lane pitch
// floor: the cut-out is a FRACTION of the anchor, so below this scale it falls
// under a pixel and a collection aliases into speckle instead of reading as
// three arranged objects. A property of the shader, not of the dial pair.
//
// LOGICAL, even though its MOTIVATION is a sampling one. It is a
// floor on a COMPOSITION quantity (the cell), and a floor applied in physical
// px would clamp a small authored cell differently at 1x and 2x, putting the
// composition back under the display's control at exactly the sizes the floor
// exists to protect. Clamping in logical px is also the conservative reading:
// at 2x the floor's own 96 logical px carry 192 device pixels of detail, which
// is strictly more resolution than the number was calibrated against.
const FINDS_MIN_SCALE_PX: f32 = 96.0;

// THE COMPANION'S VALUE BREATHE, replacing the deleted field
// translation (`organic_rgb`'s own comment). Only the COMPANION role
// (`kind_b`, drawn as `ink_b` below) breathes; the anchor and cut-out stay
// static — selection by SHAPE KIND and the cut-out role were both
// considered and rejected: kinds clump because they are
// seeded per element, triangles are the highest-salience shape, and the
// cut-out is the smallest element and risks falling under perceptible).
//
// The breathe is a `mix` between the SAME two of the world's three tones
// `ink_b` already draws from (`c_from` and whichever of `c_to`/`c_pat` the
// per-cell `swap` roll picked) — no new palette data — by scaling the
// authored density fraction `d` by a small multiplier instead of introducing
// a second blend axis. MULTIPLICATIVE, not additive, on purpose: it is what
// keeps `density == 0.0` collapsing to the flat ground EXACTLY (`d * mult ==
// 0` for any `mult`), the same differential-oracle invariant every other
// ground family in this file holds
// (`render::tests::bowerbird_finds_item176::finds_density_zero_is_exactly_
// the_flat_ground`) — an ADDITIVE nudge would have let a live breathe phase
// draw a companion at `density: 0.0`, silently breaking that law.
//
// The envelope reuses `stars.rs:185`'s own shape: a cycle position
// `u = fract(rate * phase / LAVA_LOOP_CYCLES + offset)`, with an INTEGER
// `rate` (so `u` meets its own endpoint exactly at the shared clock's wrap —
// the house law this item's own defect broke) and a decorrelated per-cell
// `offset` (so neighbouring companions never breathe in unison). A raised
// cosine turns `u` into a smooth 0..1 pulse — continuous at the wrap because
// `cos` of a 2*pi-periodic argument cannot jump — slow and soft by
// construction, so it reads as a breathe, never a flash.
//
// `ORGANIC_LOOP_CYCLES` MUST byte-match `crate::lava::LAVA_LOOP_CYCLES`
// (2.0) — the same shared-clock loop length `g.organic_phase` wraps at.
const ORGANIC_LOOP_CYCLES: f32 = 2.0;
// The same seconds-scale cadence `stars.rs`'s `TWINKLE_RATE_MIN`/`_STEPS`
// authors (3..=8 whole cycles per ~67s loop = one breath every ~8-22s):
// TASTE TUNABLE, reusing an already-validated "seconds-scale, never
// flicker" band rather than a fresh, untested one.
const ORGANIC_BREATHE_RATE_MIN: f32 = 3.0;
const ORGANIC_BREATHE_RATE_STEPS: f32 = 6.0;
// The density MULTIPLIER's peak-to-peak swing (`d_b` ranges
// `d * [1 - AMOUNT/2, 1 + AMOUNT/2]`) — a fraction of the authored density,
// itself a fraction of the tone gap. TASTE TUNABLE, flagged for live review:
// measured peak per-channel movement at Bowerbird's shipped tones/density is
// a real but modest ~17 sRGB levels (never the full tone gap, and reached
// only at a companion's own breathe peak, once every ~8-22s) — comfortably
// past "I literally don't see it" (the measured failure mode at a
// ~2-level swing) while a raised-cosine envelope over a multi-second cycle
// keeps it a breathe, not a flash. The live `--release` sitting is what
// actually settles this number, not the pixel floor alone.
const ORGANIC_BREATHE_AMOUNT: f32 = 1.2;

// THE VOID-BOUND DROPOUT. The prior mechanism drew a cell empty
// on an UNCONSTRAINED per-cell coin flip (`hash21(cell) < 0.10`), independent
// cell to cell. Independent flips have no memory of their neighbours, so nothing
// stopped a run of several adjacent cells from all landing empty at once — rare
// for any one pair, but a real margin holds thousands of cells, and the user's
// own verdict named the result: conspicuous dead patches where the omissions
// happened to align.
//
// The fix keeps the decision fully deterministic and local (no clock, no
// stored state — still a pure function of `cell`) but decorrelates
// neighbours: a cell may become empty only when its own hash is the STRICT
// MINIMUM among its full 3x3 Moore neighbourhood (itself plus all 8
// neighbours). Two mutually-adjacent cells can never both win that
// comparison — `h0(A) < h0(B)` and `h0(B) < h0(A)` cannot both hold — so no
// two lattice-adjacent cells are EVER simultaneously empty, which bounds the
// largest contiguous void to roughly one cell's own reach, structurally,
// however large the field is (proved over real pixels by
// `render::tests::bowerbird_spacing_item191`).
//
// The neighbour lookup reuses the exact lattice-cell index space
// `organic_finds_rgb` computes `cell` in. Only the horizontal axis is
// row-sheared (`qx = q.x + hash21(row, 91.0)`, a per-row offset in [0,1)), so
// the shear DELTA between any two adjacent rows sits strictly inside (-1, 1)
// — the true nearest cell one row up or down always rounds to one of
// `{col-1, col, col+1}`, so the 3x3 window below is exact, not an
// approximation that could miss a genuine geometric neighbour.
fn finds_h0(cell: vec2<f32>) -> f32 {
    return hash21(cell + vec2<f32>(19.0, 5.0));
}

fn finds_is_local_min(cell: vec2<f32>, h0: f32) -> bool {
    var min_other = finds_h0(cell + vec2<f32>(-1.0, -1.0));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(0.0, -1.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(1.0, -1.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(-1.0, 0.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(1.0, 0.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(-1.0, 1.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(0.0, 1.0)));
    min_other = min(min_other, finds_h0(cell + vec2<f32>(1.0, 1.0)));
    return h0 < min_other;
}

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

// The SDFs above are in CELL units; `* s` puts them in LOGICAL pixels (item
// 186 — `s` is the logical cell scale). The feather is authored in PHYSICAL
// pixels and converted into that logical space, so the transition band measures
// `2 * FINDS_EDGE_AA_PX` device pixels wide on the glass at EVERY device ratio
// and at any cell scale.
fn finds_fill(sd_cell: f32, s: f32) -> f32 {
    let aa = sampling_feather(FINDS_EDGE_AA_PX);
    return 1.0 - smoothstep(-aa, aa, sd_cell * s);
}

fn organic_finds_rgb(px: vec2<f32>, s: f32, d: f32) -> vec3<f32> {
    // The lattice is ROTATED and then per-row sheared: the scattered rhythm
    // survives, its rows and columns do not, so the field never resolves into
    // the visible grid a plain `floor(px / s)` would draw. No
    // longer translated by any per-frame drift — a bower is an arrangement,
    // deliberately placed and then left alone.
    let w = px;
    let ca = cos(FINDS_LATTICE_ANGLE);
    let sa = sin(FINDS_LATTICE_ANGLE);
    let q = vec2<f32>(w.x * ca - w.y * sa, w.x * sa + w.y * ca) / s;
    let row = floor(q.y);
    let qx = q.x + hash21(vec2<f32>(row, 91.0));
    let cell = vec2<f32>(floor(qx), row);
    let local = vec2<f32>(fract(qx), fract(q.y)) - vec2<f32>(0.5, 0.5);

    let h0 = finds_h0(cell);
    // Some cells hold nothing: open ground is part of the arrangement. Item
    // 191 added the local-minimum gate (`finds_is_local_min`) so this can
    // never fire on two lattice-adjacent cells at once — see the comment on
    // that function.
    if (h0 < FINDS_DROPOUT && finds_is_local_min(cell, h0)) {
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

    // THE COMPANION'S OWN VALUE BREATHE (see the constants' doc
    // above). A decorrelated per-cell roll off `h6` (a fresh salt, clear of
    // every other draw this cell already made) picks an INTEGER rate and a
    // phase offset; `breathe` is a smooth 0..1 pulse of that roll against the
    // shared clock's raw phase (`g.organic_phase`, cycles — never radians).
    // Nudging `d` by it (never swapping which tone `ink_b` targets) keeps the
    // breathe on the SAME two tones `ink_a`'s sibling companion already mixes.
    let h6 = hash21(cell + vec2<f32>(43.0, 97.0));
    let breathe_offset = fract(h6 * 61.8034);
    let breathe_rate = ORGANIC_BREATHE_RATE_MIN
        + floor(fract(h6 * 7.0) * ORGANIC_BREATHE_RATE_STEPS);
    let breathe_u = fract(breathe_rate * g.organic_phase / ORGANIC_LOOP_CYCLES + breathe_offset);
    let breathe = 0.5 - 0.5 * cos(breathe_u * FINDS_TAU);
    let breathe_mult = 1.0 + (breathe - 0.5) * ORGANIC_BREATHE_AMOUNT;
    let d_b = clamp(d * breathe_mult, 0.0, 1.0);
    let ink_b = mix(g.c_from.rgb, select(g.c_to.rgb, g.c_pat.rgb, swap), d_b);

    var rgb = mix(g.c_from.rgb, ink_a, cov_a * keep);
    return mix(rgb, ink_b, cov_b * keep);
}

// 8: the ORGANIC ground — the crisp collected-treasure FINDS field above, and
// nothing else. It opened with a second arrangement, cut-paper MASSES (three
// differently-offset rounded cell fields making large masses, islands and
// droplets, with a small inner field subtracted to leave occasional holes),
// selected by a `params.z` scalar. No world reached for it, so the arm, its
// scalar and its dial enum went together — a shader branch serving zero worlds
// is the same infrastructure smell as a one-arm enum, and deleting the enum
// alone would have banked none of it. `params.z` is inert here now.
// The field is entirely STATIC: a shape is a pure function of its
// cell and nothing pans, morphs, spawns, or dissolves it. Its one remaining
// ambient input is the companion's own per-element value breathe
// (`organic_finds_rgb`'s own doc), which changes a drawn object's TONE, never
// its position. `px` is LOGICAL, so `s` — the authored cell — is a
// logical cell, and every fraction-of-a-cell threshold follows it for free.
fn organic_rgb(px: vec2<f32>) -> vec3<f32> {
    let s = max(g.params.x, FINDS_MIN_SCALE_PX);
    let d = clamp(g.params.y, 0.0, 1.0);
    return organic_finds_rgb(px, s, d);
}

// --- 9: DECKLE — THE HANDMADE-PAPER MATERIAL FIELD. ---
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
//
// DIALS: params.x = the lane pitch (`period_px`), params.y = the wander
// amplitude (`wander_px`), params.z = the ONE coverage/contrast multiplier
// (`density`), params.w = the weave.
//
// This whole field is COMPOSITION and runs in LOGICAL pixels — the
// pitch, the wander, the wander FREQUENCIES (per logical px), the fibre and
// vein half-width ramps, and the pitch floor. Deckle carries no sampling
// feather at all: a lane boundary's softness is `DECKLE_EDGE_LO..HI`, a
// FRACTION of a lane, so it is already relative to the composition and widens
// with it. That is the point of the class distinction — a torn paper edge is a
// drawn thing, not a resolve of the sample grid.
//
// `density == 0.0` collapses BOTH profiles to their flat ground EXACTLY — the
// lane values converge on DECKLE_MID and every tint drops out. That is not a
// nicety: it is the differential oracle every pixel law for this ground
// measures against (`mark_field` idiom), so the gradient, dither and
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
// The lane pitch FLOOR, in LOGICAL px. The deckle edge is a FRACTION of a lane,
// so below this the boundary falls under a pixel and the field aliases into
// moire instead of reading as paper. Enforced HERE (a property of the shader,
// not of the dial pair — the abutment lesson) and mirrored by
// `theme::DECKLE_MIN_PERIOD_PX`. Logical for the same reason
// `FINDS_MIN_SCALE_PX` is: a floor on a composition quantity is itself a
// composition quantity, or the clamp hands the composition back to the display.
const DECKLE_MIN_PITCH_PX: f32 = 40.0;
// The weave threshold `theme::Weave::mode` writes either side of.
const DECKLE_WEAVE_FIBRES: f32 = 0.5;

// The Room/viewport owner, and the ONLY one: a page-width drag moves only the
// opaque mask above this field. The viewport centre is stable under page-width
// dragging and under the adaptive-column shift, so an exposed screen point
// cannot translate, stretch, reseed, or reflow its paper contours. Measuring
// the distance from the PAGE EDGE instead is precisely the border-decoration
// behaviour the wallpaper law rejects; that arm is gone, and the wallpaper law now
// states the property directly — the field is provably NOT invariant under the
// displacement a page-anchored owner would have introduced — instead of
// demonstrating it by keeping the rejected code alive to fail.
fn deckle_viewport_distance(px: vec2<f32>) -> f32 {
    return abs(px.x - viewport_l().x * 0.5);
}

fn deckle_strata(px: vec2<f32>, pitch: f32, wander: f32, density: f32) -> vec3<f32> {
    let d = deckle_viewport_distance(px);
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
    if (g.params.w >= DECKLE_WEAVE_FIBRES) {
        return deckle_fibres(px, pitch, wander, density);
    }
    return deckle_strata(px, pitch, wander, density);
}

// AUDIT FINDING: the plain corner-to-corner projection
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
//
// Bands has NO composition quantity in pixels at all — every boundary
// is a FRACTION of the viewport, so the same three bands span any canvas at any
// density by construction (this ground was already density-independent, and the
// item's premise does not reach it). Its one pixel number is the 1.5px boundary
// feather, which is SAMPLING and stays physical; it is converted into the
// normalized `t` space through the same owner every other feather uses.
const BANDS_MARGIN_SPAN: f32 = 1.35;
fn bands_rgb(px: vec2<f32>) -> vec3<f32> {
    let a = g.params.y;
    let dir = vec2<f32>(cos(a), sin(a));
    let vp = viewport_l();
    let center = vp * 0.5;
    let extent = max(dot(vp, dir), 1.0) * BANDS_MARGIN_SPAN;
    let t = clamp(dot(px - center, dir) / extent + 0.5, 0.0, 1.0);
    let aa = sampling_feather(1.5) / extent;
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
// DRIFT: `b1`'s phase ADVANCES by `drift`, `b2`'s RETARDS by the
// SAME amount — equal magnitude, opposite sign (see `src/background.rs`'s
// module doc for the full derivation of why opposite-sign is the one choice
// that avoids the whole field sliding as a single rigid "sheet": a same-sign
// drift on both curves is provably an exact horizontal translation of the
// entire composition, including the middle tier, so it produces ZERO relative
// motion between tiers). Under opposite signs each OUTER tier (top/bottom)
// sweeps with its own single boundary curve's sign, while the MIDDLE tier —
// bounded by both — visibly shears/breathes counter to them: the sea reads as
// independently layered swells, never a sheet translating behind the margin.
//
// The scallop AMPLITUDE and WAVELENGTH are COMPOSITION — they say how
// tall and how wide a swell reads — so both are logical (`WAVE_FREQ` is
// radians per LOGICAL px, and `px`/`viewport` arrive logical). The tier
// boundaries themselves are viewport THIRDS, already dimensionless. The 1.5px
// boundary feather is SAMPLING and stays physical.
const WAVE_AMP: f32 = 22.0;
const WAVE_FREQ: f32 = 0.024166097; // 2*pi / 260px — wide, shallow scallops
const WAVE_PHASE_1: f32 = 0.0;
const WAVE_PHASE_2: f32 = 2.4;
fn waves_rgb(px: vec2<f32>) -> vec3<f32> {
    let drift = g.drift;
    let vh = viewport_l().y;
    let b1 = vh * (1.0 / 3.0) + WAVE_AMP * sin(px.x * WAVE_FREQ + WAVE_PHASE_1 + drift);
    let b2 = vh * (2.0 / 3.0) + WAVE_AMP * sin(px.x * WAVE_FREQ + WAVE_PHASE_2 - drift);
    return tri_tone_mix(px.y, b1, b2, sampling_feather(1.5));
}

// --- 10: WARPED GRID — ONE camera, ONE projected cylinder, cropped at the page
// (the projection is recomposed). ---
//
// There is no geometry, no depth buffer and no 3-D engine here: the whole
// tunnel is a closed-form ray cast done per fragment.
//
// The camera travels straight through a circular tube. A ring is a level set of
// projected radius; a rail is a level set of polar angle.
//
// The cylinder never rescales. `WARP_SECTION_ROOM_FRAC` sizes the anchor
// ring against the ROOM's own height and nothing else, and the section is a
// CIRCLE — no aspect, no affine fit, one isotropic space — so its projected
// aspect ratio is the constant 1.00 at every page width the adaptive column can
// reach. The page width moves nothing at all.
//
// ONE AXIS, AT THE ROOM'S OWN CENTRE, AND THAT IS THE WHOLE OF THE PLACEMENT.
// An axis that is a function of WHICH MARGIN a fragment falls in gives each
// margin its own vanishing point, and a reader sees exactly what that is: two
// tunnels, side by side, disagreeing across the page. One axis at `vp.x * 0.5`
// is the whole fix, and it costs only the reason the old placement existed —
// it puts the vanishing point BEHIND THE PAGE. That is the composition: the
// thing you are travelling toward is hidden behind your own writing, and each
// margin carries one flank of the single tube around it.
//
// The placement takes no page term and no margin term, so the page column can
// only MASK this field. Both flanks are equidistant from the axis at every page
// width, by construction rather than by tuning.
//
// THE FIELD CROSSES THE PAGE, SIMPLIFIED TO ITS SCAFFOLD. Two flanks with
// nothing between them are still two pictures as far as the eye is concerned;
// the evidence that they are ONE tube is that a ring leaving the left margin
// ARRIVES in the right one at the same height. So the page column no longer
// punches this ground away — it veils the field to `WARP_PAGE_VEIL` and retires
// the minor lattice, leaving the major scaffold alone to make the crossing.
// That simplification is not a new idea: it is the same one `WARP_NARROW_LO_PX`
// already applies as a margin narrows. Prose keeps its figure/ground because
// the veil is held by a legibility floor measured over the rendered page, not
// by taste alone — see `render/tests/warp_one_tunnel_item268.rs`.

// The anchor ring's projected RADIUS, as a fraction of the room's height. The
// section is a circle, so this is the whole of its size and shape. 0.432 is
// round 1's own anchor at the narrowest page on the canonical 1600x1000 canvas
// (`3 * page_half` at measure 20 = 432px), which is the composition the live
// review approved.
const WARP_SECTION_ROOM_FRAC: f32 = 0.432;
// THE UNDER-PAGE VEIL: what fraction of its full strength the scaffold keeps
// once it crosses onto the page. It is the only quantity in this ground that
// draws where prose does, so it is the only one bounded by a LEGIBILITY floor
// rather than by composition alone — `warp_one_tunnel_item268.rs` samples the
// rendered page and asserts every pixel of it clears 4.5:1 against the world's
// own body ink, which is the same floor the syntax roles are already held to.
//
// The floor is not what set this number, though; it has room to spare. The
// number is set by what the crossing is FOR — it is evidence, not decoration.
// A ring has to be traceable from one flank to the other and must not compete
// with a line of text, and the minor lattice retiring at the page edge is what
// buys the rest of the quiet.
const WARP_PAGE_VEIL: f32 = 0.13;
// How far the crossing takes, in LOGICAL px, measured from the page edge: the
// distance over which the minor lattice retires and the veil settles. Composition
// (the Pinstripe/Zigzag rule), so a 2x display draws the same crossing.
const WARP_PAGE_EASE_PX: f32 = 52.0;
// Round 2's own placement, alive only inside the `MarginPlaced` arm: the margin
// widths, in anchors, between which its window slid, and how far in from the
// page edge the axis landed once it had.
const WARP_WINDOW_FULL: f32 = 1.0;
const WARP_WINDOW_TIGHT: f32 = 0.35;
const WARP_WINDOW_STRADDLE: f32 = 0.4;
// Rings per octave of depth is DERIVED from the authored `spacing_px`, which is
// the projected pitch of the minor rings at `WARP_RING_PITCH_AT` of the anchor
// radius — a fixed place on the fixed section, so the lattice's own scale is as
// constant as the section's. Bounded so a very short or very tall room cannot
// drive the lattice into a knot or into three lonely rings.
//
// THE REFERENCE POINT MOVED WHEN THE AXIS DID, and it had to. `spacing_px` is
// only meaningful where the reader is actually looking. With an axis parked in
// each margin, that was the near field — a third of the anchor. With ONE axis at
// the room's centre the near field is behind the page, and what the margins show
// is the band from the page edge out to the room's corner: roughly 1.1 to 2.2
// anchors on the canonical canvas. Leaving the reference at a third of the anchor
// honours the authored pitch at a radius no reader can see and hands the margins
// three lonely arcs. So this is the same design re-derived against the geometry
// that replaced it — the authored pitch is realised in the middle of the band
// the reader has.
const WARP_RING_PITCH_AT: f32 = 0.8333333;
const WARP_RPO_MIN: f32 = 3.0;
// The ceiling rises with the reference point, for the same reason: it exists to
// bite at extreme rooms, and against the new reference the old value would begin
// biting on an ordinary 4K-at-1x desktop. Ring DENSITY is not what makes a
// converging lattice unsafe — `alias_fade` and `WARP_CORE_FRAC` are the moire
// defences, and neither one reads this bound.
const WARP_RPO_MAX: f32 = 20.0;
const WARP_LN2: f32 = 0.6931472;
// Every fifth line is the strong one. The polar angle's own seam is at +/-PI,
// which maps to +/-WARP_RAILS_PER_HALF_TURN: unless that integer is a multiple
// of the modulus, the rail beside the seam is classed major on one side and
// minor on the other and draws a hard discontinuity.
const WARP_RAILS_PER_HALF_TURN: f32 = 10.0;
const WARP_MAJOR_EVERY: f32 = 5.0;
// Drawn line WEIGHTS — composition, in logical px (the Pinstripe/Zigzag rule).
const WARP_MINOR_HALF_PX: f32 = 0.45;
const WARP_MAJOR_HALF_PX: f32 = 1.00;
// The antialias skirt — SAMPLING, in physical px, so a better display resolves
// the same line more finely.
const WARP_AA_PX: f32 = 1.0;
// The lattice fades out BELOW this projected spacing, in LOGICAL pixels, so a
// converging grid can never reach the regime where a Retina or WebGL2
// rasteriser turns it into moire. It is also what quietly dissolves the
// LOGICAL, although its motivation is a sampling one: this bound decides HOW
// DEEP the tunnel is drawn, so in physical pixels the display would choose how
// much of the world the reader sees — the coordinate contract resolves that tension the
// same way for `FINDS_MIN_SCALE_PX` and `DECKLE_MIN_PITCH_PX`. It is also the
// conservative reading: at 2x these logical pixels carry twice the device
// samples, so the moire it exists to prevent is further away, not nearer.
const WARP_ALIAS_FADE_LO_PX: f32 = 4.5;
const WARP_ALIAS_FADE_HI_PX: f32 = 9.0;
// Quiet beside the page: no mark reaches the page edge, so nothing competes
// with prose at the boundary the eye reads across.
const WARP_EDGE_QUIET_PX: f32 = 10.0;
const WARP_EDGE_FADE_MAX_PX: f32 = 56.0;
// Narrow-margin simplification: the minor lattice retires as the margin
// narrows, leaving the major scaffold alone.
const WARP_NARROW_LO_PX: f32 = 84.0;
const WARP_NARROW_HI_PX: f32 = 210.0;
// THE CAMERA NEVER REACHES THE FAR END. A floor under the projected radius,
// as a fraction of the anchor, BOUNDS both lattices' projected pitch (ring
// pitch grows as `u*ln2/rpo`, rail pitch as `pi*u/rails`) and keeps the fixed
// deepest visible rings resolvable. No-moire is a property of this expression;
// the alias fade is a second line of defence.
const WARP_CORE_FRAC: f32 = 0.055;
// Both families retire INTO the far end rather than crowding into a knot: the
// far end of a real tunnel is haze, not a solid lattice.
const WARP_CORE_FADE_LO: f32 = 1.0;
const WARP_CORE_FADE_HI: f32 = 4.0;
// Mutation arms for page-derived scale, margin-derived placement, and reversed
// travel. Each threshold occupies its own unit-wide band.
const WARP_TUNNEL_PAGE_SCALED: f32 = 0.5;
const WARP_TUNNEL_MARGIN_PLACED: f32 = 1.5;
const WARP_TUNNEL_REVERSED: f32 = 2.5;
// Round 1's authored numbers, alive only inside the `PageScaled` arm: the anchor
// ring's diameter in page widths, and where its flank crossed the page edge as a
// fraction of the room's height.
const WARP_PAGE_SCALED_RATIO: f32 = 3.0;
const WARP_PAGE_SCALED_FIT: f32 = 0.42;
// Anti-aliased distance to the nearest integer level set of `coord`, in units of
// its own screen-space gradient — so one expression draws a line of constant
// width at every projected spacing, near and far.
//
// THE ONE PLACE THIS FILE MEASURES IN DEVICE PIXELS ON PURPOSE:
// `fwidth` differentiates against the RASTERISER's grid whatever space its
// argument was computed in, so `d` here is PHYSICAL however logical the
// coordinate is. The two quantities meet it from their own sides — the drawn
// WEIGHT is composition and converts UP into that space, the AA skirt is
// sampling and is already in it — which is what keeps a 2x display drawing the
// same line, more finely, rather than a line half as wide.
fn warp_line(coord: f32, half_px: f32) -> f32 {
    let fw = max(fwidth(coord), 0.0001);
    let d = abs(fract(coord + 0.5) - 0.5) / fw;
    let half_phys = half_px * dpr();
    return 1.0 - smoothstep(half_phys, half_phys + WARP_AA_PX, d);
}

// Every fifth line is the strong one: classify the NEAREST integer level set,
// so a fragment's own hierarchy agrees with the line it is drawing.
fn warp_is_major(coord: f32) -> f32 {
    let i = round(coord);
    let m = abs(i - WARP_MAJOR_EVERY * round(i / WARP_MAJOR_EVERY));
    return 1.0 - step(0.5, m);
}

// WHERE THE ONE AXIS SITS: the room's own centre, and nothing else is consulted.
// The single owner of the placement. It takes no page argument, no margin
// argument and — the whole point — no SIDE argument, so there is exactly one
// vanishing point in the room and both flanks are windows onto it.
fn warp_room_axis(vp_x: f32) -> f32 {
    return vp_x * 0.5;
}

// ROUND 2'S PLACEMENT, kept as data for the `MarginPlaced` arm: the distance the
// axis fell INSIDE the page edge, as a function of the margin's OWN width.
// Positive hid the axis behind the page; negative brought it out into the
// margin. See `theme::Tunnel`.
fn warp_window_hide(span: f32, page_half: f32, anchor: f32) -> f32 {
    let full = smoothstep(WARP_WINDOW_TIGHT, WARP_WINDOW_FULL, span / max(anchor, 1.0));
    return mix(-WARP_WINDOW_STRADDLE * span, page_half, full);
}

// `in_page` is decided ONCE, by the caller, in the PHYSICAL space the host
// measured the column in — the same test that punches every other ground away.
// Re-deciding it here against the logical bounds would put a half-pixel seam
// between the punch and the veil at any scale factor but 1.
fn warped_grid_rgba(p: vec2<f32>, in_page: bool) -> vec4<f32> {
    let vp = viewport_l();
    let cl = col_left_l();
    let cw = col_w_l();
    let spacing = max(g.params.x, 8.0);
    let density = clamp(g.params.y, 0.0, 1.0);

    // THE ONE CAMERA, AT ONE CONSTANT SCALE. The section is a circle sized
    // against the ROOM alone and the horizon is the room's middle, so no page
    // width can rescale, flatten or re-shape the cylinder.
    let page_half = max(cw * 0.5, 1.0);
    let mode = g.params.w;
    let page_scaled = mode >= WARP_TUNNEL_PAGE_SCALED && mode < WARP_TUNNEL_MARGIN_PLACED;
    let margin_placed = mode >= WARP_TUNNEL_MARGIN_PLACED && mode < WARP_TUNNEL_REVERSED;
    let reversed = mode >= WARP_TUNNEL_REVERSED;
    var anchor = WARP_SECTION_ROOM_FRAC * max(vp.y, 1.0);
    var aspect = 1.0;
    if (page_scaled) {
        // Mutation arm: scale and flatten the section from the page column.
        anchor = WARP_PAGE_SCALED_RATIO * page_half;
        let flank = sqrt(max(anchor * anchor - page_half * page_half, 1.0));
        aspect = clamp(flank / max(WARP_PAGE_SCALED_FIT * vp.y, 1.0), 1.0, 4.0);
    }
    // THE ONE AXIS. Under the shipping profile the placement reads the ROOM
    // alone — not the page, not the margin, and not which side this fragment
    // fell on — so the page column below is used for NOTHING but the legibility
    // masks and the crossing. The two page-derived placements, which DO give
    // each margin its own axis, are the mutation arms.
    let col_right = cl + cw;
    let on_right = p.x >= col_right;
    let span = max(select(cl, vp.x - col_right, on_right), 1.0);
    let hide = select(warp_window_hide(span, page_half, anchor), page_half, page_scaled);
    let placed = select(cl + hide, col_right - hide, on_right);
    let axis_x = select(
        warp_room_axis(vp.x),
        placed,
        page_scaled || margin_placed,
    );
    let axis = vec2<f32>(axis_x, vp.y * 0.5);
    // The tunnel's own space. Under the shipping profile it is the glass's own
    // space too — the section is a circle and the projection isotropic, which is
    // what makes "the aspect ratio is invariant" true by construction rather
    // than by tuning. Only `PageScaled` puts an affine transform here.
    let q = vec2<f32>(p.x - axis.x, (p.y - axis.y) * aspect);

    // Straight tube: the projected radius and polar angle are direct.
    let core = WARP_CORE_FRAC * anchor;
    let w = q;
    let u_raw = length(w);
    let u = max(u_raw, core);

    // Rings are level sets of `log(u)`, and forward travel is one ADDITION.
    //
    // The ring labelled `n` is drawn where
    // `rpo*log2(anchor/u) + Z = n`, i.e. at `u = anchor * 2^((Z - n)/rpo)`.
    // `warp_travel` strictly increases with time, so
    // every ring's projected radius GROWS and the lattice sweeps outward past
    // the reader — which is approach. Subtracting `Z`, as this line did before,
    // shrinks every radius toward the axis instead: the rings converge into the
    // far end and the world reads as travelling backwards. `Tunnel::Reversed`
    // is that sign kept as data.
    let rpo = clamp(
        WARP_RING_PITCH_AT * anchor * WARP_LN2 / spacing,
        WARP_RPO_MIN,
        WARP_RPO_MAX,
    );
    let travel = select(g.warp_travel, -g.warp_travel, reversed);
    let ring = rpo * log2(anchor / u) + travel;
    let rail = atan2(w.y, w.x) * (WARP_RAILS_PER_HALF_TURN / 3.14159265);

    // The four families, kept APART until the masks have had their say — because
    // only one of them crosses the page. A RING is the depth cue: it is a closed
    // curve around the axis, so a ring that leaves the left flank has to arrive
    // in the right one, and that arrival is the whole evidence that there is one
    // tube. A RAIL is radial; it runs INTO the page rather than across it, and
    // the two rails through the axis would draw a full-width horizontal and a
    // full-height vertical straight through the prose — chrome, not depth. So
    // the rails retire at the page edge with the minor lattice.
    let ring_major = warp_is_major(ring);
    let rail_major = warp_is_major(rail);
    let core_fade = smoothstep(core * WARP_CORE_FADE_LO, core * WARP_CORE_FADE_HI, u_raw);
    let ring_hi = warp_line(ring, WARP_MAJOR_HALF_PX) * ring_major * core_fade;
    let rail_hi = warp_line(rail, WARP_MAJOR_HALF_PX) * rail_major * core_fade;
    let lattice = max(
        warp_line(ring, WARP_MINOR_HALF_PX) * (1.0 - ring_major),
        warp_line(rail, WARP_MINOR_HALF_PX) * (1.0 - rail_major),
    ) * core_fade;

    // Legibility masks, on the SAME side test the window placement already made
    // — how far into a margin this fragment is, and how wide that margin is.
    // Masks on one field, never a second camera.
    let edge_d = max(select(cl - p.x, p.x - col_right, on_right), 0.0);
    // THE CROSSING, as ONE signed profile over the whole room. `sd` is the
    // distance to the nearer page edge: positive out in a margin, negative under
    // the page. The margin ramp keeps its shape; what changed is its FLOOR. It
    // used to reach zero at the page edge because the page edge was the end of
    // the world — now the field continues past it, so the floor is the veil, and
    // the stroke that leaves the left flank is the same stroke that arrives in
    // the right one. A ramp that touched zero in between would break exactly the
    // continuity this ground exists to show.
    let depth_in = max(min(p.x - cl, col_right - p.x), 0.0);
    let sd = select(edge_d, -depth_in, in_page);
    // THE RAMP LIVES ENTIRELY IN THE MARGIN. It starts AT the page edge, so the
    // page carries exactly the veil everywhere and nothing bleeds inward at half
    // strength across the first inch of prose — where a line of text starts.
    let edge_fade = mix(
        WARP_PAGE_VEIL,
        1.0,
        smoothstep(
            0.0,
            WARP_EDGE_QUIET_PX + min(WARP_EDGE_FADE_MAX_PX, span * 0.5),
            sd,
        ),
    );
    // WHAT CROSSES IS THE MAJOR RINGS ALONE: the minor lattice and the whole
    // rail family are absent from the page entirely and fade in over the first
    // `WARP_PAGE_EASE_PX` of MARGIN, so the only marks that ever share space
    // with prose are the sparse concentric arcs.
    let margin_only = smoothstep(0.0, WARP_PAGE_EASE_PX, sd);
    let major = max(ring_hi, rail_hi * margin_only);
    let minor = lattice * margin_only;
    // The projected spacing of whichever lattice is finer here. `fwidth` is a
    // device-grid derivative (see `warp_line`), so the reciprocal is physical and
    // is divided back into the LOGICAL space the bound is authored in.
    let finest_px = 1.0 / (max(max(fwidth(ring), fwidth(rail)), 0.0001) * dpr());
    let alias_fade = smoothstep(WARP_ALIAS_FADE_LO_PX, WARP_ALIAS_FADE_HI_PX, finest_px);
    let narrow_fade = smoothstep(WARP_NARROW_LO_PX, WARP_NARROW_HI_PX, span);

    let minor_cov =
        clamp(minor * alias_fade * narrow_fade * edge_fade * density * 0.60, 0.0, 1.0);
    let major_cov = clamp(major * alias_fade * edge_fade * density * 0.88, 0.0, 1.0);
    if (in_page) {
        // OVER the page's own flat clear, in straight alpha — this ground is the
        // one that does not punch. There is no `c_from` here: the page's tone is
        // the page's business, and the field only tints it by the coverage it
        // actually drew.
        let a = clamp(minor_cov + major_cov, 0.0, 1.0);
        return vec4<f32>(select(g.c_pat.rgb, g.c_to.rgb, major_cov >= minor_cov), a);
    }
    let with_minor = mix(g.c_from.rgb, g.c_pat.rgb, minor_cov);
    return vec4<f32>(mix(with_minor, g.c_to.rgb, major_cov), 1.0);
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
// Called with the PHYSICAL fragment position, deliberately. This is
// the purest SAMPLING quantity in the file — a threshold matrix whose job is to
// perturb each DEVICE pixel by half a quantization step before the render
// target rounds it to 8 bits. Tiling it in logical px would put four device
// pixels on one threshold at 2x and hand the banding back.
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
    // WARPED GRID is the one ground that declines the punch — its whole subject
    // is a single tube whose axis is behind the page, so it VEILS the column
    // instead of vanishing at it (see `warped_grid_rgba`). Every other ground
    // keeps the hard punch, unchanged.
    let in_page = in.px.x >= g.col_left && in.px.x < g.col_left + g.col_w;
    if (in_page && g.shader != 10u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    // THE ONE conversion. Every ground below composes in LOGICAL
    // pixels; the PAGE-COLUMN punch above and the dither below deliberately do
    // not (the column is a physical geometry fact the host measured, and the
    // dither belongs to the device grid).
    let lp = to_logical(in.px);
    // 5/6: BANDS / WAVES compute their own final rgb directly (three opaque
    // authored tones ARE the field) — bypass the gradient/dither/pattern-
    // overlay pipeline below entirely, which every OTHER ground still takes
    // unchanged (byte-identical).
    if (g.shader == 5u) {
        return vec4<f32>(bands_rgb(lp), 1.0);
    }
    if (g.shader == 6u) {
        return vec4<f32>(waves_rgb(lp), 1.0);
    }
    if (g.shader == 8u) { return vec4<f32>(organic_rgb(lp), 1.0); }
    if (g.shader == 9u) { return vec4<f32>(deckle_rgb(lp), 1.0); }
    if (g.shader == 10u) { return warped_grid_rgba(lp, in_page); }
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
    let cov = pattern_coverage(lp) * g.c_pat.a;
    rgb = mix(rgb, g.c_pat.rgb, cov);
    return vec4<f32>(rgb, a);
}
