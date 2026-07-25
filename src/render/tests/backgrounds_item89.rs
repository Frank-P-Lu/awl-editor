//! ITEM 89 — REAL-PIXEL FIELD laws for `Background::Zigzag`, the correctness
//! repair of item 86's chevron margin ground.
//!
//! **The defect these laws exist to forbid.** Item 86's shader computed the
//! chevron's centerline as `center = tri(rx) * amp` and its coverage as
//! `abs(ry - center)` — a function of the ALONG-travel coordinate alone. That
//! describes ONE continuous "V" curve embedded in the plane: the teeth repeat
//! along the travel direction, but nothing repeats the curve ACROSS it, so a
//! page margin showed a single wandering stroke with large blank areas, and a
//! taller window did not gain a second row — it only let that same lone stroke
//! travel further before running off the bottom. Item 86's own tests missed it
//! because their positive half was "SOME pattern pixel exists somewhere in the
//! margin", which one stroke satisfies trivially. Item 89 folds `(ry - center)`
//! through the row period, turning the single curve into the infinite family
//! `center + k * row_h` — a genuinely tiled Mario-like zigzag field.
//!
//! **The oracle.** Every appearance claim here is PIXEL arithmetic (the project
//! tripwire: the sidecar is a state oracle, never an appearance oracle), and it
//! is DIFFERENTIAL: the same world is rendered twice, once as authored and once
//! with `density: 0.0` (which zeroes the mark's coverage exactly —
//! `mix(rgb, tint, 0.0) == rgb` — and touches nothing else), then subtracted.
//! The gradient, its ordered dither, and the sRGB quantization are bit-identical
//! between the two passes, so the difference image is the MARK ALONE, with no
//! mirrored color math to drift out of lockstep. `INK_FLOOR` is the per-pixel
//! total-channel difference that counts as material.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use crate::theme;
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};

/// The deterministic scan canvas: `capture::CANVAS_WIDTH`x`CANVAS_HEIGHT`, the
/// project's own "ordinary window" frame (every ground in this family authors
/// its dials in unscaled device px against it — `Dots`' 24px cell, `Pinstripe`'s
/// 9px rule, `Waves`' 22px scallop), with a page column parked mid-canvas so
/// BOTH margins are wide enough (350px) to grid meaningfully.
const W: u32 = 1200;
const H: u32 = 800;
const COL_LEFT: f32 = 350.0;
const COL_W: f32 = 500.0;
/// The two margins the column leaves, as `[x0, x1)` device-px spans.
const MARGINS: [(u32, u32); 2] = [(0, 350), (850, 1200)];
/// Per-pixel total-channel deviation from the mark-free pass that counts as
/// real material. The differential oracle cancels the dither exactly, so this
/// only has to clear 8-bit quantization.
const INK_FLOOR: i32 = 3;
/// The occupancy grid: `GRID` x `GRID` cells per margin.
const GRID: u32 = 3;
/// Inked pixels a cell must hold to count as "visibly contains material" —
/// ~1% of a 116x266 cell, against 3.7%-7.9% actually measured on the two
/// shipping worlds (deliberate slack: the law is "no blank region", not a
/// pinned coverage number).
const MIN_INKED_PER_CELL: usize = 300;

/// The two worlds that wear this ground (the exhaustive no-wildcard roster law
/// lives in `backgrounds_item86.rs`).
fn zigzag_worlds() -> [(&'static str, theme::Background); 2] {
    [
        ("Quokka", theme::QUOKKA.background),
        ("Gumtree", theme::GUMTREE.background),
    ]
}

/// The DIFFERENTIAL mark field: per-pixel total-channel deviation between the
/// world as authored and the same world with its mark coverage zeroed. See the
/// module doc — this isolates the chevron ink from the gradient/dither exactly,
/// with no host-side color mirror. The ONE owner of that measurement: item 86's
/// sibling module reuses it for its CONTRAST law rather than re-deriving a
/// weaker distance-from-the-gradient-endpoints proxy.
pub(super) fn mark_field(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
) -> Vec<i32> {
    let inked = bg_desc_for(bg);
    let bare = crate::background::BgDesc { density: 0.0, ..inked };
    // `drift` is item 87's Waves-only phase slot — inert `0.0` for this static
    // ground, and structurally unable to touch Zigzag's `params` dials.
    let a = render_bg(device, queue, inked, w, h, col_left, col_w, 0.0);
    let b = render_bg(device, queue, bare, w, h, col_left, col_w, 0.0);
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| (0..3).map(|k| (p[k] as i32 - q[k] as i32).abs()).sum::<i32>())
        .collect()
}

/// Number of SEPARATE inked bands a vertical scanline at `x` crosses — the
/// direct count of chevron ROWS stacked down the margin (each row the scanline
/// meets contributes one run; the gaps between rows separate them).
///
/// A SCHMITT trigger, not a bare threshold: a run OPENS on a ribbon core (half
/// the field's own peak deviation — relative, so the two worlds' very different
/// contrasts are measured the same way) and only CLOSES on true zero. The field
/// is exactly 0 between rows (coverage outside a ribbon's smoothstep is 0.0, and
/// the differential oracle cancels the dither), so the hysteresis costs nothing
/// there — but it stops a quiet world's ~8-bit-thin ribbon from splitting into
/// two phantom "rows" wherever quantization dips its interior by one step.
fn row_crossings(field: &[i32], w: u32, h: u32, x: u32, peak: i32) -> usize {
    let core = (peak / 2).max(INK_FLOOR);
    let mut runs = 0usize;
    let mut inside = false;
    for y in 0..h {
        let v = field[(y * w + x) as usize];
        if !inside && v >= core {
            runs += 1;
            inside = true;
        } else if inside && v == 0 {
            inside = false;
        }
    }
    runs
}

/// The cell rects of one margin's `GRID`x`GRID` occupancy grid, as
/// `(x0, x1, y0, y1)` in device px.
fn cells(x0: u32, x1: u32, h: u32) -> Vec<(u32, u32, u32, u32)> {
    let cw = (x1 - x0) / GRID;
    let ch = h / GRID;
    (0..GRID)
        .flat_map(|r| {
            (0..GRID).map(move |c| (x0 + c * cw, x0 + (c + 1) * cw, r * ch, (r + 1) * ch))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// THE OCCUPANCY LAW — the item's headline claim.
// ---------------------------------------------------------------------------

/// THE FIELD-OCCUPANCY LAW: partition EACH of the two page margins into a 3x3
/// grid of substantial cells (116x266 device px) and require EVERY one of the 18
/// cells, on BOTH worlds, to hold real chevron material — not merely a nonzero
/// pixel, but `MIN_INKED_PER_CELL` inked pixels AND a local peak at least half
/// the field's own global peak (i.e. an actual ribbon core crosses the cell, not
/// a feather tail leaking in from a neighbour). This is the law item 86 lacked:
/// its positive half was satisfied by one stroke anywhere in the margin, so a
/// field that was 60-95% blank passed.
#[test]
fn zigzag_field_covers_every_cell_of_both_page_margins_on_both_worlds() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_field_covers_every_cell_of_both_page_margins_on_both_worlds: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        let field = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let peak = field.iter().copied().max().unwrap_or(0);
        assert!(peak >= 6, "{name}: the zigzag field must reach real ink (peak {peak})");
        for (mi, (mx0, mx1)) in MARGINS.iter().enumerate() {
            for (x0, x1, y0, y1) in cells(*mx0, *mx1, H) {
                let mut inked = 0usize;
                let mut cell_peak = 0i32;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let v = field[(y * W + x) as usize];
                        if v >= INK_FLOOR {
                            inked += 1;
                        }
                        cell_peak = cell_peak.max(v);
                    }
                }
                assert!(
                    inked >= MIN_INKED_PER_CELL,
                    "{name}: margin {mi} cell x[{x0},{x1}) y[{y0},{y1}) is BLANK \
                     ({inked} inked px, need {MIN_INKED_PER_CELL}) — the zigzag field \
                     must tile the whole margin, not wander through part of it"
                );
                assert!(
                    cell_peak * 2 >= peak,
                    "{name}: margin {mi} cell x[{x0},{x1}) y[{y0},{y1}) only catches a \
                     feather (local peak {cell_peak} vs field peak {peak}) — no chevron \
                     ribbon core crosses it"
                );
            }
        }
    }
}

/// THE ROW-RHYTHM LAW (the board's "roughly three broad visible zigzag rows at
/// an ordinary window height", plus the two worlds' authored characters): on the
/// deterministic 1200x800 canvas Gumtree's broad field crosses a mid-margin
/// vertical scanline as ~3 rows and Quokka's tighter one as distinctly MORE.
/// Ranges, not pinned numbers — the claim is the reading ("about three broad
/// rows; Quokka tighter"), and the strict inequality is what keeps the two
/// worlds from converging on one rhythm.
#[test]
fn zigzag_reads_as_about_three_broad_rows_on_gumtree_and_a_tighter_field_on_quokka() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_reads_as_about_three_broad_rows_on_gumtree_and_a_tighter_field_on_quokka: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    // The expected band travels WITH the world, so there is no name match in a
    // render-side assertion (the design-is-data discipline `theme_caps_law`
    // enforces for real render code, kept here by habit).
    let expected = [
        ("Quokka", theme::QUOKKA.background, 5..=9),
        ("Gumtree", theme::GUMTREE.background, 3..=5),
    ];
    let mut tightest = Vec::new();
    for (name, bg, band) in expected {
        let field = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let peak = field.iter().copied().max().unwrap_or(0);
        // Both margins, mid-cell scanlines: the rhythm is a property of the
        // field, not of one lucky x.
        let xs = [40u32, 175, 310, 890, 1025, 1160];
        let n: Vec<usize> = xs.iter().map(|&x| row_crossings(&field, W, H, x, peak)).collect();
        let lo = *n.iter().min().unwrap();
        let hi = *n.iter().max().unwrap();
        assert!(
            band.contains(&lo) && band.contains(&hi),
            "{name}: chevron rows per 800px scanline {n:?} must sit in {band:?}"
        );
        tightest.push(lo);
    }
    assert!(
        tightest[0] > tightest[1],
        "Quokka's field must read TIGHTER (more chevron rows per window: {} vs Gumtree's {})",
        tightest[0], tightest[1]
    );
}

/// THE HEIGHT-SCALING LAW — the defect's own signature, inverted. Item 86's
/// single stroke did not repeat across the field, so DOUBLING the canvas height
/// bought no extra rows (the same lone stroke merely travelled further before
/// leaving the canvas: measured row counts stayed put while 62% of a tall
/// margin read blank). A tiled field's row count instead scales with the
/// height it is given: doubling the canvas must roughly double the rows.
#[test]
fn zigzag_row_count_scales_with_the_canvas_height() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_row_count_scales_with_the_canvas_height: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        let short = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let tall = mark_field(&device, &queue, bg, W, H * 2, COL_LEFT, COL_W);
        let peak = short.iter().copied().max().unwrap_or(0);
        let tall_peak = tall.iter().copied().max().unwrap_or(0);
        for x in [175u32, 1025] {
            let a = row_crossings(&short, W, H, x, peak);
            let b = row_crossings(&tall, W, H * 2, x, tall_peak);
            assert!(
                b as f32 >= 1.6 * a as f32,
                "{name}: x={x} — a canvas twice as tall shows {b} chevron rows against {a}; \
                 a tiled field must gain rows with the height (item 86's single stroke did not)"
            );
            assert!(
                b as f32 <= 2.4 * a as f32,
                "{name}: x={x} — {b} rows against {a} on half the height: the row rhythm must \
                 be a CONSTANT spacing, not one that accelerates with the viewport"
            );
        }
    }
}

/// THE COLUMN-EXCLUSION LAW: the chevron field contributes EXACTLY ZERO ink
/// inside the writing column, on both worlds — the differential field is
/// identically 0 over every column pixel (the mark cannot even reach there), and
/// every column pixel is still the untouched framebuffer clear (the shader's own
/// alpha-0 hole, punched before any per-shader branch runs). Asserted alongside
/// a real-material check in the margins so it can never pass vacuously on an
/// empty render.
#[test]
fn zigzag_contributes_zero_ink_inside_the_writing_column_on_both_worlds() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_contributes_zero_ink_inside_the_writing_column_on_both_worlds: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    // Straight-alpha blend at src-alpha 0 leaves the framebuffer's own CLEAR
    // value untouched (`result = dst`) — `render_bg` clears to opaque black, so
    // an untouched column pixel reads `[0,0,0,255]`, NOT a literal `[0,0,0,0]`.
    const CLEARED: [u8; 4] = [0, 0, 0, 255];
    for (name, bg) in zigzag_worlds() {
        let field = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let pixels = render_bg(&device, &queue, bg_desc_for(bg), W, H, COL_LEFT, COL_W, 0.0);
        let mut margin_inked = 0usize;
        for y in 0..H {
            for x in 0..W {
                let idx = (y * W + x) as usize;
                let inside = (x as f32) >= COL_LEFT && (x as f32) < COL_LEFT + COL_W;
                if inside {
                    assert_eq!(
                        field[idx], 0,
                        "{name}: the zigzag mark reached ({x},{y}) INSIDE the writing column"
                    );
                    assert_eq!(
                        pixels[idx], CLEARED,
                        "{name}: ({x},{y}) inside the writing column is not the untouched clear"
                    );
                } else if field[idx] >= INK_FLOOR {
                    margin_inked += 1;
                }
            }
        }
        // Non-vacuity: the pass really did paint a field in the margins.
        assert!(
            margin_inked > 10_000,
            "{name}: only {margin_inked} inked margin pixels — the exclusion law must not pass \
             on an empty render"
        );
    }
}

/// DETERMINISM: two independent renders of the identical desc are byte-for-byte
/// equal, at two canvas sizes (the field is a pure function of the pixel
/// coordinate — no clock, no randomness, and item 87's `drift` slot is
/// structurally unreachable from this ground). Supersedes item 86's
/// single-size version.
#[test]
fn zigzag_renders_byte_identically_across_independent_draws_at_two_canvas_sizes() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping zigzag_renders_byte_identically_across_independent_draws_at_two_canvas_sizes: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        let desc = bg_desc_for(bg);
        for (w, h, cl, cw) in [(W, H, COL_LEFT, COL_W), (900u32, 1600u32, 200.0f32, 400.0f32)] {
            let a = render_bg(&device, &queue, desc, w, h, cl, cw, 0.0);
            let b = render_bg(&device, &queue, desc, w, h, cl, cw, 0.0);
            assert_eq!(a, b, "{name}: two draws of the identical desc diverged at {w}x{h}");
        }
    }
}

// ---------------------------------------------------------------------------
// THE SHADER MIRROR — used ONLY to prove the occupancy grid above is capable of
// FAILING, and kept in lockstep with the GPU by its own law.
// ---------------------------------------------------------------------------

/// A minimal host-side mirror of `pattern_coverage`'s `shader == 7u` branch
/// (`shaders/background.wgsl`), in BOTH shapes: `tiled = false` is item 86's
/// original single-curve `abs(ry - center)`, `tiled = true` is item 89's folded
/// field. Its ONLY job is the non-vacuity self-proof below (showing the grid
/// catches the untiled shape); every real appearance claim in this file is
/// measured off the GPU. Held in lockstep by
/// [`the_host_mirror_agrees_with_the_gpus_own_row_rhythm`].
fn zigzag_coverage(x: f32, y: f32, bg: theme::Background, tiled: bool) -> f32 {
    let period = bg.period_px().max(1.0);
    let a = bg.angle();
    let amp = bg.amplitude_px().clamp(0.0, period * 0.40);
    let dens = bg.density().clamp(0.0, 1.0);
    let (ca, sa) = (a.cos(), a.sin());
    let rx = x * ca + y * sa;
    let ry = -x * sa + y * ca;
    let fr = |v: f32| v - v.floor();
    let tri = (fr(rx / period) * 2.0 - 1.0).abs() * 2.0 - 1.0;
    let center = tri * amp;
    let d = if tiled {
        (fr((ry - center) / period + 0.5) - 0.5).abs() * period
    } else {
        (ry - center).abs()
    };
    let thickness = (amp * 0.10).max(1.2);
    let t = ((d - thickness * 0.6) / (thickness - thickness * 0.6)).clamp(0.0, 1.0);
    let line = 1.0 - t * t * (3.0 - 2.0 * t);
    line * dens
}

/// NON-VACUITY SELF-PROOF for the occupancy grid: run the EXACT same 3x3-per-
/// margin geometry over item 86's untiled coverage (the mirror's
/// `tiled = false` arm) and over item 89's folded one, using each world's own
/// authored dials. The untiled shape leaves whole CELLS with literally zero
/// material — the blank margin the item reopened — while the tiled field fills
/// every cell. So the grid above is a discriminating check, not a formality
/// that any zigzag would pass.
#[test]
fn the_occupancy_grid_rejects_item_86s_untiled_single_stroke() {
    for (name, bg) in zigzag_worlds() {
        let peak = bg.density();
        for (mx0, mx1) in MARGINS {
            let mut empty_untiled = 0usize;
            for (x0, x1, y0, y1) in cells(mx0, mx1, H) {
                let count = |tiled: bool| {
                    (y0..y1)
                        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
                        .filter(|&(x, y)| {
                            zigzag_coverage(x as f32, y as f32, bg, tiled) >= 0.35 * peak
                        })
                        .count()
                };
                assert!(
                    count(true) >= MIN_INKED_PER_CELL,
                    "{name}: the TILED field must fill cell x[{x0},{x1}) y[{y0},{y1})"
                );
                if count(false) == 0 {
                    empty_untiled += 1;
                }
            }
            assert!(
                empty_untiled >= 1,
                "{name}: margin [{mx0},{mx1}) — item 86's untiled stroke must leave at least one \
                 grid cell BLANK, else this grid could not have caught the defect"
            );
        }
    }
}

/// LOCKSTEP: the host mirror above must agree with the REAL GPU field on the
/// row rhythm it measures — the same vertical scanlines, the same relative
/// half-peak criterion, within one row (antialiasing and 8-bit quantization
/// legitimately move a single boundary). Without this, a mirror that drifted
/// from the shader would leave the non-vacuity proof arguing about code that no
/// longer ships.
#[test]
fn the_host_mirror_agrees_with_the_gpus_own_row_rhythm() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping the_host_mirror_agrees_with_the_gpus_own_row_rhythm: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        let field = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let peak = field.iter().copied().max().unwrap_or(0);
        for x in [40u32, 175, 310, 890, 1025, 1160] {
            let gpu = row_crossings(&field, W, H, x, peak);
            // The mirror's own half-peak criterion: coverage peaks AT `density`
            // on a ribbon's centerline.
            let mut mirror = 0usize;
            let mut inside = false;
            for y in 0..H {
                let cov = zigzag_coverage(x as f32, y as f32, bg, true);
                if !inside && cov >= 0.5 * bg.density() {
                    mirror += 1;
                    inside = true;
                } else if inside && cov <= 0.0 {
                    inside = false;
                }
            }
            assert!(
                gpu.abs_diff(mirror) <= 1,
                "{name}: x={x} — mirror counts {mirror} chevron rows, the GPU {gpu}: the host \
                 mirror has drifted out of lockstep with shaders/background.wgsl"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// DATA + STRUCTURAL laws (no GPU needed — they hold on an adapter-less machine)
// ---------------------------------------------------------------------------

/// THE ROW-COLLISION BOUND: the shader clamps `amplitude_px` to 40% of
/// `period_px` so neighbouring chevron rows can never overlap into a solid
/// smear. Every shipping Zigzag world must author its profile INSIDE that
/// bound, so the clamp is a guard for future data and never silently rewrites
/// an authored look.
#[test]
fn authored_zigzag_amplitude_stays_inside_the_shaders_row_collision_bound() {
    for t in theme::THEMES {
        if let theme::Background::Zigzag { period_px, amplitude_px, .. } = t.background {
            assert!(
                amplitude_px <= period_px * 0.40,
                "{}: amplitude_px {amplitude_px} exceeds 40% of period_px {period_px} — the \
                 shader would clamp it, so the authored profile is not what renders",
                t.name
            );
        }
    }
}

/// STRUCTURAL TRIPWIRE on the shader source itself, so the fold cannot be
/// dropped in a refactor on a machine where the GPU laws above skip: the
/// `shader == 7u` branch must fold the ACROSS-travel coordinate through the row
/// period, and must NOT carry item 86's unfolded `abs(ry - center)`.
#[test]
fn the_wgsl_zigzag_branch_folds_the_across_travel_axis() {
    let src: String = include_str!("../../../shaders/background.wgsl")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    for needle in ["letrow_h=period;", "letu=(ry-center)/row_h;", "abs(fract(u+0.5)-0.5)*row_h"] {
        assert!(
            src.contains(needle),
            "shaders/background.wgsl lost the zigzag field fold (missing `{needle}`) — the ground \
             would fall back to one wandering stroke (item 89)"
        );
    }
    assert!(
        !src.contains("letd=abs(ry-center);"),
        "shaders/background.wgsl reintroduced item 86's UNFOLDED chevron distance"
    );
}
