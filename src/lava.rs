//! Animated viewport-space lava ground. The shader and pure Rust field helpers
//! must stay aligned. Ticks run only for active lava worlds; reduced motion and
//! headless capture use fixed phases for determinism.

use crate::theme::{Background, LavaEdge, Srgb};
use std::sync::OnceLock;

// --- CADENCE / PHASE constants ------------------------------------------------

/// Ambient tick period; inactive worlds schedule no lava frames.
pub const LAVA_TICK_MS: u64 = 100;

/// Phase advance rate in cycles per second.
pub const LAVA_SPEED: f32 = 0.03;

/// The WHOLE field's period in phase cycles. Vertical bob repeats after one
/// cycle, but horizontal sway uses half-frequency and repeats only after two;
/// wrapping at two is therefore the first phase where every blob center meets
/// its own starting point.
pub const LAVA_LOOP_CYCLES: f32 = 2.0;

/// One fixed ambient advancement step. A delayed event-loop wake (notably while
/// macOS is dragging the window) may report much more wall time than this, but
/// the lamp advances by at most one sparse-tick step: it drifts instead of
/// catching up in one visible jump.
pub const LAVA_TICK_SECONDS: f32 = LAVA_TICK_MS as f32 / 1000.0;

/// The FROZEN phase: what the lamp settles to under Reduce Motion, and the fixed
/// phase a headless capture always renders (t=0, deterministic). The base blob
/// layout ([`BACKDROP_BLOBS`]) is authored so this phase reads as a settled mid
/// composition, so the one frozen frame serves BOTH the accessibility freeze
/// and the capture — matching the caret-demo `settle()` precedent (`render.rs`).
pub const LAVA_FROZEN_PHASE: f32 = 0.0;

/// MARGINS-ONLY mask feather WIDTH (px): how far into the margin, starting from
/// the column edge, the field ramps 0 → full strength. Comfortably inside a
/// modest margin. TASTE TUNABLE — flagged for live review.
pub const MARGIN_GAP_PX: f32 = 28.0;

/// Maximum blobs the shader's uniform carries — exactly [`BACKDROP_BLOBS`]'s
/// population, so every slot is authored and live.
pub const MAX_BLOBS: usize = 12;

/// ONE continuous backdrop field, authored in viewport UV and wholly independent
/// of the page column. Each row is `[cx, cy, r, w]`: center in viewport UV,
/// radius as a fraction of viewport height, and field weight. Several blobs sit
/// behind the ordinary page footprint on purpose; widening/narrowing the page
/// only occludes/reveals this same composition instead of manufacturing two
/// separately-sized side lamps. SHARED by both lava worlds (Firetail + Mangrove
/// read the same field — a per-world fork would be machinery the design doesn't
/// need). The user-approved twelve-body population: a wide large/medium/
/// satellite hierarchy (mean radius ~0.131, firmer than the prior eight-body
/// field's ~0.146) keeps the large masses legible while the satellites make
/// each margin read as a field rather than two banks, exposing a healthier
/// population in both margins than the eight-body predecessor did.
pub const BACKDROP_BLOBS: [[f32; 4]; 12] = [
    [0.06, 0.14, 0.130, 0.92],
    [0.18, 0.38, 0.165, 1.00],
    // cy calibrated to 0.75 (from the trial's 0.67) so this satellite's own
    // reach seeds the bottom-left GUTTER's frost pocket — a fixed corner probe
    // (`theme::tests::gutter_frost_pill_keeps_ink_contrast_on_every_lava_world`)
    // the trial never exercised (it always read the eight-body control field).
    // Radius/weight, and every other body, are the approved trial data verbatim.
    [0.07, 0.75, 0.140, 0.95],
    [0.23, 0.86, 0.104, 0.86],
    [0.35, 0.58, 0.145, 1.00],
    [0.62, 0.29, 0.138, 1.00],
    [0.76, 0.08, 0.100, 0.82],
    [0.88, 0.31, 0.156, 1.00],
    [0.95, 0.59, 0.123, 0.92],
    [0.78, 0.78, 0.140, 0.96],
    [0.61, 0.88, 0.098, 0.84],
    [0.48, 0.50, 0.133, 0.95],
];

/// The shader's horizontal-sway multiplier (`shaders/lava.wgsl`'s `blob_center`
/// reads it as `g.anim.y`): firms the silhouette and keeps the lamp's ambient
/// drift reading mainly VERTICAL, down from a prior field's full (1.0)
/// horizontal amplitude. MUST match the same constant folded into
/// [`animated_center`] below, the pure mirror every geometry law reads.
pub const LAVA_HORIZONTAL_SWAY: f32 = 0.52;

// --- FROST constants (the shipped headed-doc treatment) -----------------------
//
// The user's design: on a lava world a HEADED doc keeps BOTH margins alive (the
// lamp animates in the left margin too), and the drawn margin ink (the outline
// entries + the bottom-left gutter) sits over a FROSTED FIELD — the SMOOTH
// metaball field softened + a value dim — so the dim ink keeps its contrast while
// the lamp stays fully alive around it. This REPLACES the old whole-left-margin
// CARVE (which flattened the entire rail to ground). All TASTE TUNABLE — flagged
// for live review, named like `THEME_FONT_DEBOUNCE`.
//
// ORGANIC GLYPH-SEEDED FIELD (item 32): the frost is NOT a union of rectangular
// pills. Every visible margin glyph SEEDS a small close halo; the halos are
// summed into ONE continuous scalar field and thresholded ([`frost_coverage`]),
// so neighbouring seeds MERGE in ANY direction wherever their softened halos
// overlap — nearby words / rows naturally become a larger organic island, a run
// of consecutive headings joins while a group-gapped one separates, with NO
// artificial per-row separation and NO zoom breakpoint (the topology falls out of
// the continuous field, never an `if zoom >= X` mode switch). Mangrove and
// Firetail share the SAME shape recipe; only the palette-derived colour differs.

/// THE SHIPPED HEADED-DOC TREATMENT. `true` (the user's pick — "both margins
/// alive on every doc") = the glyph-seeded FROST field, the lamp animating in both
/// margins. Flip this ONE const to `false` to revert to the OLD whole-left-margin
/// CARVE (the `rail` global + [`rail_dist_outside`] + `lava_rail_carved` stay
/// wired for exactly this one-line data revert): frost turns off and a headed
/// doc's whole left margin flattens back to the flat page ground.
pub const FROST_RAIL_DEFAULT: bool = true;

/// The VALUE DIM inside the frost field: how far the softened field is mixed back
/// toward the flat page `ground` (0 = the raw softened lamp, 1 = pure flat
/// ground). Sized so the dim margin ink clears its ink-ladder contrast floor
/// against the field at EVERY animation phase (law
/// `outline_frost_pills_keep_ink_contrast_on_every_lava_world`), while a whisper
/// of the softened lamp still reads behind the text.
pub const FROST_DIM: f32 = 0.65;

/// The frost BLUR kernel spacing (logical px, zoom/DPI-scaled by the caller): the
/// per-tap offset of the 3×3 cross [`frost_field`] averages the SMOOTH field over.
/// Averaging the raw undithered field (never the posterized color) is the
/// Mangrove REQUIREMENT — blurring the Bayer grid makes cross moiré (the
/// documented palette-blur lesson), so the frost samples the field, not the
/// dither.
pub const FROST_BLUR_PX: f32 = 5.0;

/// The frost SEED HALO SKIRT (logical px, zoom/DPI-scaled): a soft px pad added to
/// each seed's glyph-derived halo radius, so the field blends into the live lamp
/// instead of drawing a hard boundary. (Formerly the rectangular pill's edge
/// feather; now the halo skirt of the organic field.)
pub const FROST_FEATHER_PX: f32 = 7.0;

/// The horizontal padding (logical px, zoom/DPI-scaled) the seeded run extends past
/// each end of its shaped text extent — the "comfortable padding" that hugs the
/// text without clipping its antialiased edge.
pub const FROST_PILL_PAD_X: f32 = 6.0;

/// The vertical inset of a frost seed row from its full line box, as a fraction of
/// the row height (top AND bottom) — so seed y-centres hug the text band. (Kept
/// for the coarse outline-ink exclusion rects the STARS layer reads.)
pub const FROST_PILL_INSET_Y_FRAC: f32 = 0.1;

/// THE SEED HALO RADIUS as a fraction of the LABEL row height — the "small close
/// halo" each glyph seeds, DERIVED FROM THE ACTUAL ZOOMED GLYPH GEOMETRY (the row
/// height IS the zoomed line box). Tuned with [`FROST_ISO`] so glyph seeds within
/// a run always merge, consecutive rows (pitch one row height) merge into a larger
/// island, and a group-gapped row (pitch 1.5 rows) tends to separate — the natural
/// island structure, with no explicit per-row rule.
pub const FROST_SEED_RADIUS_FRAC: f32 = 0.62;

/// THE PUNCTUATION-AWARE RUN RADIUS FRACTION (item 61): a run's halo radius is
/// bounded by its OWN measured ink half-width (times this fraction) plus the
/// skirt, so a run whose ink is narrow relative to the row-height radius — an
/// isolated `&`, a single-letter heading — gets a halo sized to ITS OWN
/// geometry instead of the full row-height fraction ([`FROST_SEED_RADIUS_FRAC`]),
/// which otherwise dwarfs a short run into a disproportionate round bump (a
/// seed's `(1 - (d/r)^2)^2` falloff is symmetric, so a near-zero-ink run at a
/// large `r` reads as a bare circle centred on the glyph, not a close hug). A
/// long/normal run's ink half-width already exceeds this bound, so `min()` is a
/// no-op there — the row-height radius (and its row/nearby-run merge behaviour)
/// is UNCHANGED for ordinary text. See [`crate::render::push_text_seeds`].
pub const FROST_RUN_INK_RADIUS_FRAC: f32 = 0.5;

/// THE BOUNDED END-PAD CEILING (item 61), expressed as a multiple of the skirt
/// ([`FROST_FEATHER_PX`], zoom/DPI-scaled): the ABSOLUTE most a run's halo may
/// reach past its own measured ink, independent of the row-height radius
/// ([`FROST_SEED_RADIUS_FRAC`] × row height). Without this ceiling a run's
/// end-of-label overshoot grows with the row's own line-height (tall margin
/// type, a deep heading ladder rung) with no relation to how long the label
/// actually is — a long single-run label ("Button-free", no internal
/// whitespace to break it into smaller runs) would carry the SAME oversized
/// skirt past its final glyph as a one-character run. Tuned so a typical
/// multi-word row's own radius sits at or under the ceiling (byte-identical
/// merge behaviour for ordinary text — see `docs/render.md`), while a tall
/// margin type / large zoom no longer inflates the endcap without bound.
pub const FROST_END_RADIUS_SKIRTS: f32 = 3.0;

/// THE FIELD ISO LEVEL: the summed seed halo strength at which the organic frost
/// coverage crosses 0.5. A lone seed peaks at 1.0 at its core, so ISO < 1 gives a
/// halo out past the seed; two seeds each contributing ~ISO/2 in the gap between
/// them bridge into one island (the metaball neck). MUST match `shaders/lava.wgsl`.
pub const FROST_ISO: f32 = 0.55;

/// THE ISO SOFT BAND: the summed-field half-width over which coverage ramps
/// 0 → 1 around [`FROST_ISO`] — the organic edge softness (never a hard contour).
/// MUST match `shaders/lava.wgsl`.
pub const FROST_ISO_SOFT: f32 = 0.42;

/// SEED GRANULARITY (item 32 perf arm). `true` = PER-GLYPH seeds (one halo per
/// visible glyph — the ideal bumpy hug). `false` = the NAMED DEGRADATION ARM:
/// one WORD-RUN seed per whitespace-delimited run, still merging organically in
/// any direction, far fewer per-pixel seeds. Flipped to the degradation arm only
/// if the per-glyph steady frame cost regresses > 5% (item 32 STEP 3).
pub const FROST_SEED_PER_GLYPH: bool = false;

/// Convert an authored Frost dimension from logical to physical pixels. The lava
/// shader consumes physical pixels, so its blur, skirt, and pad must use the same
/// user-zoom × device-DPI scale as [`crate::render::Metrics::with_dpi`].
pub fn frost_px(logical_px: f32, zoom: f32, dpi: f32) -> f32 {
    logical_px * zoom * dpi
}

/// The MAX frost SEEDS the shader's uniform carries (`array<vec4<f32>,
/// MAX_FROST_SEEDS>`). Per-glyph seeding over a full followed outline plus the
/// gutter can reach into the low hundreds; the field clamps here (a generous cap,
/// far above any realistic margin-ink glyph budget).
pub const MAX_FROST_SEEDS: usize = 256;

/// The MAX outline-ink exclusion RECTS the STARS layer reads (one per drawn
/// outline row) — unchanged from the pre-organic-field cap, kept for that coarse
/// keep-out geometry (see [`crate::render::TextPipeline::lava_frost_pill_rects`]).
pub const MAX_FROST_PILLS: usize = 48;

#[allow(dead_code)] // shader-mirror constant (see the pure-math note below).
const TAU: f32 = std::f32::consts::TAU;

// Frost blend constants — MUST match `shaders/lava.wgsl`'s `THRESHOLD` /
// `EDGE_WIDTH` / `CORE_WIDTH` (the metaball edge/core smoothstep bands the frost
// pixel maps the softened field through).
const FROST_THRESHOLD: f32 = 0.5;
const FROST_EDGE_WIDTH: f32 = 0.12;
const FROST_CORE_WIDTH: f32 = 0.35;

// --- PURE math (the shader mirror, unit-tested) -------------------------------
//
// `#[allow(dead_code)]` on the four functions below (+ `TAU`): the REAL runtime
// math happens in `shaders/lava.wgsl`'s own copy of this exact field + mask; these
// Rust functions exist ONLY as the pure mirror `lava::tests` exercises — the
// established `render::dither`/`SelectionPipeline::instance_count` idiom for a
// test-only shader mirror. They MUST stay in lockstep with the WGSL.

/// WGSL-matching `smoothstep(edge0, edge1, x)`: 0 below `edge0`, 1 above `edge1`,
/// a Hermite ease between. Pure — the Rust mirror of the shader's own builtin.
#[allow(dead_code)]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 == edge1 {
        return if x < edge0 { 0.0 } else { 1.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The signed "distance outside the no-lava zones" at pixel x (px): positive out
/// in a lava-bearing margin, <= 0 inside a zone the field must vanish from.
/// Ordinarily the one zone is the writing column `[col_left, col_right]` (both
/// edges via `max()`). With the LEFT-MARGIN RAIL carved (`rail_carved` — the
/// margin OUTLINE is actually DRAWN this frame, a HEADED doc, see
/// `TextPipeline::lava_rail_carved`) the whole LEFT margin joins the no-lava
/// zone: only the RIGHT margin's distance counts, so the outline's dim entries
/// sit on the flat ground at every phase and the lamp keeps the right margin
/// (the outline hiding reclaims the full margin). The bottom-left GUTTER no
/// longer flattens the whole margin — it drives the bounded
/// [`gutter_corner_dist_outside`] carve instead, so an ordinary (gutter-only)
/// doc keeps BOTH margins. The carve also feeds the Glow treatment's
/// `could_glow` through this same distance, so no unexplained edge-bleed tints
/// the page next to a flat rail. MUST match `shaders/lava.wgsl`'s `dist_outside`.
#[allow(dead_code)]
pub fn rail_dist_outside(x: f32, col_left: f32, col_right: f32, rail_carved: bool) -> f32 {
    if rail_carved {
        x - col_right
    } else {
        (col_left - x).max(x - col_right)
    }
}

/// The MARGINS-ONLY lava coverage mask at pixel x (px): 0 inside every no-lava
/// zone ([`rail_dist_outside`] — the writing column, plus the whole left margin
/// while the outline rail is carved) and at its edge, ramping to 1 a `gap` px
/// further out. The lava is drawn at this coverage, so the field fades entirely
/// outside the zones and the page (and a carved rail) stays a clean flat
/// ground. MUST match `shaders/lava.wgsl`'s `mask`. `gap` is floored at 1.0
/// (matching the shader).
#[allow(dead_code)]
pub fn lava_mask(x: f32, col_left: f32, col_right: f32, gap: f32, rail_carved: bool) -> f32 {
    smoothstep(
        0.0,
        gap.max(1.0),
        rail_dist_outside(x, col_left, col_right, rail_carved),
    )
}

/// The plain (un-carved) column mask — [`lava_mask`] with no outline rail, kept
/// as the named identity the page-width-invariance tests read.
#[allow(dead_code)]
pub fn column_mask(x: f32, col_left: f32, col_right: f32, gap: f32) -> f32 {
    lava_mask(x, col_left, col_right, gap, false)
}

/// The signed "distance outside the GUTTER's local corner rect" at pixel
/// `(x, y)` (px): <= 0 inside the bounded bottom-left region the gutter owns,
/// positive out beyond it. `rect` is `[left, top, right, bottom]`. The per-axis
/// outside distances (negative inside the span) are combined by `max`, so the
/// result is negative iff BOTH axes are inside — the box interior — and its
/// magnitude just outside a face is the perpendicular distance to that face.
/// Unlike [`rail_dist_outside`] (which flattens the WHOLE left margin for a
/// headed doc), this carves only a bounded corner, leaving the rest of both
/// margins their lamp. MUST match `shaders/lava.wgsl`'s `gutter_dist_outside`.
#[allow(dead_code)]
pub fn gutter_corner_dist_outside(x: f32, y: f32, rect: [f32; 4]) -> f32 {
    let gx = (rect[0] - x).max(x - rect[2]);
    let gy = (rect[1] - y).max(y - rect[3]);
    gx.max(gy)
}

/// The full 2-D lava coverage mask at pixel `(x, y)`: the 1-D margin mask
/// ([`lava_mask`] — the writing column plus, when `rail_carved`, the whole left
/// margin) further multiplied by the GUTTER's local corner carve when
/// `gutter_rect` is `Some` (the field vanishes over the bounded bottom-left
/// region, feathered over `gap` px at its top/right faces). This is the exact
/// SHIP-path mirror of `shaders/lava.wgsl`'s `fs_main` mask (probe mode 0): with
/// `gutter_rect = None` it is byte-for-byte [`lava_mask`]. MUST stay in lockstep
/// with the shader.
#[allow(dead_code)]
pub fn lava_mask_2d(
    x: f32,
    y: f32,
    col_left: f32,
    col_right: f32,
    gap: f32,
    rail_carved: bool,
    gutter_rect: Option<[f32; 4]>,
) -> f32 {
    let base = lava_mask(x, col_left, col_right, gap, rail_carved);
    match gutter_rect {
        Some(r) => base * smoothstep(0.0, gap.max(1.0), gutter_corner_dist_outside(x, y, r)),
        None => base,
    }
}

/// The ANIMATED center (UV space) of base blob `i` at `phase` (in cycles) — the
/// slow lava bob, a per-blob sine keyed off the index so the lamps never move in
/// unison. Horizontal amplitude is scaled by [`LAVA_HORIZONTAL_SWAY`] so ambient
/// drift reads mainly vertical. MUST match `shaders/lava.wgsl`'s `blob_center`
/// (which reads the same multiplier off `g.anim.y`). Pure.
#[allow(dead_code)]
pub fn animated_center(
    i: usize,
    base_cx: f32,
    base_cy: f32,
    base_r: f32,
    viewport: (f32, f32),
    phase: f32,
) -> (f32, f32) {
    let fi = i as f32;
    let amp_y = 0.055 + 0.020 * (fi * 0.37).fract();
    // Horizontal sway follows the authored viewport-relative radius, so the
    // whole backdrop scales coherently with the window, never with page width.
    let aspect = viewport.1.max(1.0) / viewport.0.max(1.0);
    let amp_x = base_r * aspect * (0.18 + 0.08 * (fi * 0.61).fract()) * LAVA_HORIZONTAL_SWAY;
    let off = fi * 1.7;
    let cy = base_cy + amp_y * (phase * TAU + off).sin();
    let cx = base_cx + amp_x * (phase * TAU * 0.5 + off * 1.3).sin();
    (cx, cy)
}

/// The summed metaball FIELD at pixel `px` (physical px), the Gaussian-falloff
/// sum over the animated blobs — MUST match `shaders/lava.wgsl`'s
/// `metaball_field`. `blobs` are `[cx, cy, r, w]` base positions; `viewport` is
/// `(width, height)` px. Pure (a function of position + phase, never a clock).
#[allow(dead_code)]
pub fn metaball_field(px: (f32, f32), viewport: (f32, f32), blobs: &[[f32; 4]], phase: f32) -> f32 {
    const FIELD_K: f32 = 1.2;
    let mut total = 0.0;
    for (i, b) in blobs.iter().enumerate() {
        let (cx, cy) = animated_center(i, b[0], b[1], b[2], viewport, phase);
        let center = (cx * viewport.0, cy * viewport.1);
        let r_px = (b[2] * viewport.1).max(1.0);
        let dx = px.0 - center.0;
        let dy = px.1 - center.1;
        let dist_sq = dx * dx + dy * dy;
        total += b[3] * (-FIELD_K * dist_sq / (r_px * r_px)).exp();
    }
    total
}

// --- FROST pure math (the shader mirror, unit-tested) -------------------------
//
// The FROST treatment (behind the margin ink): a SOFTENED sample of the SMOOTH
// metaball field ([`frost_field`], a 3×3 tap average — never the dithered color,
// the Mangrove palette-blur lesson) mapped through the same edge/core blend the
// lamp uses, then value-dimmed toward the flat ground ([`frost_pixel`]). Blended
// into the live lamp by the ORGANIC glyph-seeded coverage — every seed contributes
// a soft halo ([`frost_seed_bump`]), the halos SUM into one continuous field and
// threshold ([`frost_coverage`]) so neighbours merge in any direction. All four
// MUST stay in lockstep with `shaders/lava.wgsl`.

/// The SOFTENED (blurred) metaball field at pixel `px`: [`metaball_field`]
/// averaged over a 3×3 tap cross at `blur` px spacing. Averaging the RAW field
/// (undithered) widens each blob's apparent edge without ever sampling the Bayer
/// grid — the Mangrove REQUIREMENT (blurring the ordered-dither grid makes cross
/// moiré). MUST match `shaders/lava.wgsl`'s `frost_field`. Pure.
#[allow(dead_code)]
pub fn frost_field(
    px: (f32, f32),
    viewport: (f32, f32),
    blobs: &[[f32; 4]],
    phase: f32,
    blur: f32,
) -> f32 {
    let mut acc = 0.0;
    for oy in [-blur, 0.0, blur] {
        for ox in [-blur, 0.0, blur] {
            acc += metaball_field((px.0 + ox, px.1 + oy), viewport, blobs, phase);
        }
    }
    acc / 9.0
}

/// The FROST PILL PIXEL (sRGB): the softened `field` mapped through the lamp's
/// own edge/core blend (`ground → blob_lo → blob_hi`), then VALUE-DIMMED toward
/// the flat `ground` by `dim` so the dim outline ink keeps its contrast. The
/// blend is computed in sRGB (the documented approximation the sibling lava
/// figure/ground law uses — the shader mixes in linear, but the tones are dark +
/// close so the perceptual gap is negligible; the law asserts the contrast floor
/// over these values directly). MUST match `shaders/lava.wgsl`'s frost color path.
#[allow(dead_code)]
pub fn frost_pixel(field: f32, ground: Srgb, blob_lo: Srgb, blob_hi: Srgb, dim: f32) -> Srgb {
    let edge_t = smoothstep(
        FROST_THRESHOLD - FROST_EDGE_WIDTH,
        FROST_THRESHOLD + FROST_EDGE_WIDTH,
        field,
    );
    let core_t = smoothstep(FROST_THRESHOLD, FROST_THRESHOLD + FROST_CORE_WIDTH, field);
    let lerp = |a: u8, b: u8, t: f32| -> u8 {
        (a as f32 + (b as f32 - a as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    let ch = |gc: u8, lo: u8, hi: u8| -> u8 {
        let blob = lerp(lo, hi, core_t); // blob_lo → blob_hi by core_t
        let smooth = lerp(gc, blob, edge_t); // ground → blob by edge_t
        lerp(smooth, gc, dim) // value dim back toward the flat ground
    };
    Srgb {
        r: ch(ground.r, blob_lo.r, blob_hi.r),
        g: ch(ground.g, blob_lo.g, blob_hi.g),
        b: ch(ground.b, blob_lo.b, blob_hi.b),
        a: 0xFF,
    }
}

/// THE SEED HALO BUMP at pixel `(x, y)` for one glyph/word SEED `[x0, x1, yc, r]`
/// (a horizontal run `[x0, x1]` at row centre `yc`, halo radius `r` px): a compact
/// support soft bump — `(1 - (d/r)^2)^2` where `d` is the distance to the run
/// segment (0 within the run's x-span), so it is `1` at the ink and `0` past a
/// radius, C1-smooth. Summed across seeds it is the metaball field that MERGES
/// neighbours. MUST match `shaders/lava.wgsl`'s `seed_bump`. Pure.
#[allow(dead_code)]
pub fn frost_seed_bump(x: f32, y: f32, seed: [f32; 4]) -> f32 {
    let dx = (seed[0] - x).max(x - seed[1]).max(0.0);
    let dy = y - seed[2];
    let r = seed[3].max(1.0);
    let t = (1.0 - (dx * dx + dy * dy) / (r * r)).clamp(0.0, 1.0);
    t * t
}

/// THE ORGANIC FROST COVERAGE at pixel `(x, y)`: the seed halos SUMMED into one
/// continuous field, then thresholded at [`FROST_ISO`] over the [`FROST_ISO_SOFT`]
/// band. Because the halos SUM (never a max/union), two nearby seeds bridge into
/// one island wherever their combined field clears the iso — so glyphs within a
/// run, and nearby rows, merge in ANY direction with no per-row rule. An EMPTY
/// seed list is a total no-op (0 everywhere) — the inert non-frost frame. MUST
/// match `shaders/lava.wgsl`'s seed loop. Pure.
#[allow(dead_code)]
pub fn frost_coverage(x: f32, y: f32, seeds: &[[f32; 4]]) -> f32 {
    let mut field = 0.0f32;
    for s in seeds {
        field += frost_seed_bump(x, y, *s);
    }
    smoothstep(
        FROST_ISO - FROST_ISO_SOFT,
        FROST_ISO + FROST_ISO_SOFT,
        field,
    )
}

// --- CADENCE / PHASE resolution (pure, unit-tested) ---------------------------

/// THE CADENCE GATE: may the live App arm its slow ambient lava tick THIS frame?
/// True ONLY when a lava world is active AND ambient motion is on AND motion is
/// NOT reduced AND the window is focused and unobstructed (pause on frost,
/// resize, and move). A non-lava world
/// (`active == false`) is always false, so it schedules ZERO extra frames —
/// preserving 0% idle CPU. Pure, so the whole gate is unit-testable.
pub fn lava_should_tick(
    active: bool,
    ambient_on: bool,
    reduced: bool,
    focused: bool,
    paused: bool,
) -> bool {
    active && ambient_on && !reduced && focused && !paused
}

/// THE PAUSE COMPOSITION the cadence gate's `paused` term is fed from — ONE
/// owner of "which transient live interactions hold the lamp": an active
/// RESIZE stream, an active MOVE stream, or a blur-eligible overlay (frost).
/// Any of the three holds the phase (and, since [`lava_should_tick`] is the
/// only door to `advance_lava`, the field with it) without resetting it. Pure,
/// so the OR-composition itself is law-testable — the live App reads its three
/// inputs off `resize_settle_at` / `move_settle_at` / `lava_blur_active()`.
pub fn lava_paused(resizing: bool, moving: bool, blurred: bool) -> bool {
    resizing || moving || blurred
}

// THE PREVIEW-CROSSING CLASSIFICATION IS RETIRED (2026-07-18). It once decided
// whether a theme-preview step got the present-transaction bracket by testing the
// OUTGOING/INCOMING pair against a HEAVYWEIGHT-PIPELINE boundary (ambient cadence
// or one-bit pipeline). A live probe of the reported Mangrove→Magpie gesture
// proved the classification structurally wrong: the actual LANDING step
// (`Galah→Magpie`) is same-side on both boundaries → it read `Steady` and armed
// NO bracket, so the landing frame presented unbracketed while the bracket that
// did arm (on the transient `Mangrove→Galah` boundary earlier in the nav) had
// already torn down. Three successive widenings (is_lava → ambient → one-bit)
// chased the boundary and never covered the landing. The fix is the simpler
// truth: `App::retint_theme_preview` arms the bracket on EVERY preview step
// unconditionally, and the teardown is event-ordered (it waits for the reshaped
// frame's in-bracket present) rather than a per-crossing timer. There is no
// per-pair decision left to make, so the pure fn + its `CrossingAction` enum are
// gone; `has_ambient_motion` / `is_one_bit` survive for their other callers.

/// Choose the viewport used to lay out the metaball field. During a live resize
/// the last-settled dimensions are held while the live viewport and column mask
/// continue to follow the window; the new dimensions become authoritative only
/// on settle.
pub fn field_viewport(live: [f32; 2], settled: [f32; 2]) -> [f32; 2] {
    if settled[0] > 0.0 && settled[1] > 0.0 {
        settled
    } else {
        live
    }
}

/// The blur capture consumes a smooth lava source. Ordered posterization is an
/// authored live-world treatment, but its axis-aligned grid aliases with the
/// downsampled separable frost and produces crosses; outside capture it remains
/// exactly as the world requested.
pub fn dither_for_blur(authored: bool, backdrop_blur: bool) -> bool {
    authored && !backdrop_blur
}

/// Bound an ambient wake's elapsed wall time to ONE fixed sparse tick. Normal
/// due wakes therefore advance by exactly [`LAVA_TICK_SECONDS`]; delayed wakes
/// never accumulate and replay the missing wall time as a visible catch-up jump.
/// Pure, so the macOS event-loop-stall behavior is law-testable without a window.
pub fn ambient_tick_dt(elapsed: f32) -> f32 {
    elapsed.clamp(0.0, LAVA_TICK_SECONDS)
}

/// Advance the phase by one bounded ambient step at [`LAVA_SPEED`], wrapping to
/// `[0, LAVA_LOOP_CYCLES)` so a long-running session never loses `sin` precision
/// AND the half-frequency horizontal term meets its own endpoint. Pure.
pub fn advance_phase(phase: f32, dt: f32) -> f32 {
    let p = phase + ambient_tick_dt(dt) * LAVA_SPEED;
    p.rem_euclid(LAVA_LOOP_CYCLES)
}

/// The EFFECTIVE render phase: the dev gallery `env` override wins outright
/// (frozen gallery captures); else Reduce Motion pins [`LAVA_FROZEN_PHASE`]
/// (mirroring the caret-demo `settle()` precedent); else the App-driven `stored`
/// phase (which is [`LAVA_FROZEN_PHASE`] = 0.0 in a headless capture, since the
/// capture never ticks). Pure — the whole determinism story reads off this one
/// resolver. See `TextPipeline::lava_render_phase`.
pub fn lava_phase_for(stored: f32, reduced: bool, env: Option<f32>) -> f32 {
    match env {
        Some(e) => e,
        None if reduced => LAVA_FROZEN_PHASE,
        None => stored,
    }
}

// --- The dev-only gallery knob (AWL_LAVA=...) ---------------------------------
//
// Mirrors `AWL_CJK_FORCE` / the probe's `AWL_LAVA_PROBE` exactly: read ONCE at
// startup, memoized, a total no-op unless set. Since NO world ships a lava
// background yet, this is the only way to render the lamp (it forces a
// `Background::Lava` over whatever world is active, at a FIXED phase), so a
// gallery capture can be produced for the human eyeball step. Format:
//   AWL_LAVA=<palette>:<phase>[:<edge>][:<dither>]
//   <palette> = warm | deepsea            (the probe's tuned, legibility-checked palettes)
//   <phase>   = a float (the frozen composition, e.g. 0.0 / 0.35)
//   <edge>    = hard | glow               (optional; default glow — the probe's agent pick)
//   <dither>  = dither                    (optional; the coarse Bayer print-grain)
// e.g. AWL_LAVA=deepsea:0.35:glow:dither

fn parse_spec(raw: &str) -> Option<(Background, f32)> {
    let mut parts = raw.split(':');
    let palette = parts.next()?;
    let phase: f32 = parts.next()?.parse().ok()?;
    let mut edge = LavaEdge::Glow;
    let mut dithered = false;
    for tok in parts {
        match tok {
            "hard" => edge = LavaEdge::Hard,
            "glow" => edge = LavaEdge::Glow,
            "dither" | "dithered" => dithered = true,
            "" => {}
            _ => return None,
        }
    }
    // Reuse the SHIPPED worlds' authored colors rather than carrying a second
    // probe-only copy that can drift after a palette retune. The env spec still
    // owns its requested edge/dither treatment below.
    let source = match palette {
        "warm" => crate::theme::FIRETAIL.background,
        "deepsea" => crate::theme::MANGROVE.background,
        _ => return None,
    };
    let (ground, blob_lo, blob_hi, _, _) = source.lava_params()?;
    Some((
        Background::Lava {
            ground,
            blob_lo,
            blob_hi,
            edge,
            dithered,
        },
        phase,
    ))
}

fn spec() -> &'static Option<(Background, f32)> {
    static ONCE: OnceLock<Option<(Background, f32)>> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::var("AWL_LAVA")
            .ok()
            .as_deref()
            .and_then(parse_spec)
    })
}

/// The dev gallery override [`Background::Lava`], if `AWL_LAVA` was set at startup
/// and parses. `None` (every normal + headless run) means: no override, the
/// active world's real background stands — byte-identical to before this feature.
pub fn env_override() -> Option<Background> {
    spec().as_ref().map(|(bg, _)| *bg)
}

/// The dev gallery override's FIXED phase, if `AWL_LAVA` is set. Consumed by
/// [`lava_phase_for`] (env wins outright), so a gallery capture renders exactly
/// the requested frozen composition.
pub fn env_phase() -> Option<f32> {
    spec().as_ref().map(|(_, phase)| *phase)
}

// --- The dev-only FROST-OFF gallery knob (AWL_LAVA_FROST=off) ------------------
//
// Mirrors the `AWL_LAVA` / `AWL_CJK_FORCE` precedent: read ONCE at startup,
// memoized, a TOTAL no-op unless set, so ship + headless determinism is untouched
// when absent. The ONLY knob kept — the vetoed plate/band/bleed both-sides
// auditions were deleted (the user picked FROST). `AWL_LAVA_FROST=off` turns the
// frost pills OFF so the A/B "before" (the outline sitting on the raw, unfrosted
// lamp — why frost earns its place) stays producible for a gallery.

/// Whether the dev-only `AWL_LAVA_FROST` env knob was set to `off` at startup —
/// the A/B "before" (frost pills suppressed). Read once, memoized. A no-op
/// (returns `false`) unless set, so every normal + headless run frosts by default.
fn frost_env_off() -> bool {
    static ONCE: OnceLock<bool> = OnceLock::new();
    *ONCE.get_or_init(|| {
        std::env::var("AWL_LAVA_FROST")
            .ok()
            .as_deref()
            .map(|v| v.trim().eq_ignore_ascii_case("off"))
            .unwrap_or(false)
    })
}

/// Whether per-entry FROST is active this run: the shipped default
/// ([`FROST_RAIL_DEFAULT`]) UNLESS the dev-only `AWL_LAVA_FROST=off` gallery knob
/// suppressed it. When off, a headed lava doc's outline sits on the raw lamp (the
/// A/B "before"); when the const is flipped to `false`, frost is off AND the old
/// whole-margin carve returns (see [`FROST_RAIL_DEFAULT`]).
pub fn frost_on() -> bool {
    FROST_RAIL_DEFAULT && !frost_env_off()
}

// --- The wgpu pipeline --------------------------------------------------------

/// Uniform globals. MUST match `Globals` in `shaders/lava.wgsl`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Globals {
    viewport: [f32; 2],
    field_viewport: [f32; 2],
    blob_count: u32,
    dither: u32,
    /// 1 = the margin OUTLINE is drawn this frame (a HEADED doc), so the whole
    /// LEFT margin is its rail — carved out of the field mask (the conservative
    /// FULL carve, see [`rail_dist_outside`]). The bottom-left GUTTER no longer
    /// gates this full carve; it drives the LOCAL corner carve below instead, so
    /// an ordinary (gutter-only) doc keeps BOTH margins their lamp.
    rail: u32,
    /// 1 = the bottom-left GUTTER is drawn this frame, so a bounded LOCAL corner
    /// region around it ([`Globals::gutter_rect`]) is carved out of the field
    /// mask while the rest of both margins keep the lamp. MUST match
    /// `shaders/lava.wgsl`'s `gutter` gate + [`gutter_corner_dist_outside`].
    gutter: u32,
    /// `[col_left_px, col_right_px, gap_px, mask_mode]` — `mask_mode` from
    /// [`LavaEdge::mask_mode`] (1.0 hard, 2.0 glow).
    margin: [f32; 4],
    /// `[phase, 0, 0, 0]` — phase in cycles.
    anim: [f32; 4],
    ground: [f32; 4],
    blob_lo: [f32; 4],
    blob_hi: [f32; 4],
    blobs: [[f32; 4]; MAX_BLOBS],
    /// The GUTTER's local corner carve rect `[left, top, right, bottom]` (px) —
    /// the bounded bottom-left region the field vanishes from when `gutter == 1`.
    /// All-zero when there is no gutter carve. See [`gutter_corner_dist_outside`].
    gutter_rect: [f32; 4],
    /// FROST params `[dim, blur_px, iso_unused, seed_count]`: the organic
    /// glyph-seeded field treatment (the shipped headed-doc default). `seed_count`
    /// (the trailing float) is how many of [`Globals::seeds`] are live — `0` in
    /// every non-frost frame (non-lava world, no margin ink, or `AWL_LAVA_FROST=off`),
    /// so the whole frost path is inert. See [`frost_pixel`] / [`frost_coverage`].
    frost: [f32; 4],
    /// The FROST SEEDS `[x0, x1, yc, r]` (px), one per visible margin glyph (or
    /// per word-run under the degradation arm) — the halos whose SUMMED field the
    /// lava renders FROSTED behind. Only the first `frost.w` are live (all-zero
    /// otherwise). See [`MAX_FROST_SEEDS`] / [`frost_seed_bump`].
    seeds: [[f32; 4]; MAX_FROST_SEEDS],
}

/// The LAVA-LAMP metaball ground pipeline: one fullscreen triangle, drawn right
/// after the margin-gradient background and before every foreground layer.
/// Mirrors [`crate::background::BackgroundPipeline`]'s structure (std140-friendly
/// globals, a tiny local bytemuck shim, vertex-free draw, straight-alpha
/// over-blend). `active` is set each [`Self::prepare`]; [`Self::draw`] is a total
/// no-op while `false`, so a non-lava world draws NOTHING (byte-identical).
pub struct LavaPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    globals_buf: wgpu::Buffer,
    active: bool,
}

impl LavaPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lava shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/lava.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lava globals layout"),
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
            label: Some("lava globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lava globals bind"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lava pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lava pipeline"),
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
                // Straight-alpha over-blend (same as the background pipeline): the
                // margins composite onto the base-ground pass, the transparent
                // column (alpha 0) leaves the base_100 page clear untouched.
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
            active: false,
        }
    }

    /// Upload this frame's globals from the resolved lava `params` (`None` for
    /// every non-lava world → the pipeline goes INACTIVE and draws nothing), the
    /// live column bounds (`col_left`/`col_w` from `TextPipeline::column_left`/
    /// `column_width`, the one geometry owner), whether the whole LEFT margin is
    /// carved out of the mask this frame (`rail_carved`, from
    /// `TextPipeline::lava_rail_carved` — the margin OUTLINE's own draw gate, so
    /// the full carve can never disagree with what the frame draws), the GUTTER's
    /// bounded LOCAL corner carve rect (`gutter_rect`, `Some` iff the gutter
    /// draws — from `TextPipeline::lava_gutter_carve_rect`), the effective
    /// `phase`, the organic FROST `frost_seeds` (the visible margin glyphs' halo
    /// seeds `[x0, x1, yc, r]` — empty in every non-frost frame, so the frost path
    /// is inert) plus their `[dim, blur_px, iso]` params, and (for the one-line
    /// carve revert) whether the whole LEFT margin is carved (`rail_carved`, `false`
    /// under the frost default).
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        settled_field_viewport: [f32; 2],
        col_left: f32,
        col_w: f32,
        rail_carved: bool,
        gutter_rect: Option<[f32; 4]>,
        frost_seeds: &[[f32; 4]],
        frost_params: [f32; 3],
        params: Option<(Srgb, Srgb, Srgb, LavaEdge, bool)>,
        phase: f32,
    ) {
        let (ground, blob_lo, blob_hi, edge, dithered) = match params {
            Some(p) => p,
            None => {
                self.active = false;
                return;
            }
        };
        self.active = true;
        let mut blobs = [[0.0f32; 4]; MAX_BLOBS];
        for (dst, src) in blobs.iter_mut().zip(BACKDROP_BLOBS.iter()) {
            *dst = *src;
        }
        let globals = Globals {
            viewport: [width as f32, height as f32],
            field_viewport: field_viewport([width as f32, height as f32], settled_field_viewport),
            blob_count: BACKDROP_BLOBS.len() as u32,
            dither: dithered as u32,
            rail: rail_carved as u32,
            gutter: gutter_rect.is_some() as u32,
            margin: [col_left, col_left + col_w, MARGIN_GAP_PX, edge.mask_mode()],
            anim: [phase, LAVA_HORIZONTAL_SWAY, 0.0, 0.0],
            ground: srgb_u8_to_linear(ground),
            blob_lo: srgb_u8_to_linear(blob_lo),
            blob_hi: srgb_u8_to_linear(blob_hi),
            blobs,
            gutter_rect: gutter_rect.unwrap_or([0.0; 4]),
            frost: {
                let n = frost_seeds.len().min(MAX_FROST_SEEDS);
                [frost_params[0], frost_params[1], frost_params[2], n as f32]
            },
            seeds: {
                let mut ss = [[0.0f32; 4]; MAX_FROST_SEEDS];
                for (dst, src) in ss.iter_mut().zip(frost_seeds.iter()) {
                    *dst = *src;
                }
                ss
            },
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck_lite::bytes_of(&globals));
    }

    /// Record the fullscreen-triangle draw — a TOTAL NO-OP while inactive (no
    /// lava world / the last `prepare` saw `None`), so a non-lava frame is
    /// byte-identical to before this feature existed.
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if !self.active {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Convert an opaque sRGB u8 color to linear-light rgba for the shader (the
/// render target is sRGB). Same converter as the background pipeline's.
fn srgb_u8_to_linear(c: Srgb) -> [f32; 4] {
    fn ch(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    [ch(c.r), ch(c.g), ch(c.b), 1.0]
}

mod bytemuck_lite {
    /// # Safety
    /// Implementors must be `#[repr(C)]`, contain no padding, and consist only of
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
