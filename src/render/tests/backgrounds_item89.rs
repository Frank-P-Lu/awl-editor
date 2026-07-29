//! ITEM 89 — REAL-PIXEL FIELD laws for `Background::Zigzag`, the correctness
//! repair of item 86's chevron margin ground.
//!
//! **Defect 1 — the field did not tile.** Item 86's shader computed the
//! chevron's centerline as `center = tri(rx) * amp` and its coverage as
//! `abs(ry - center)` — a function of the ALONG-travel coordinate alone. That
//! describes ONE continuous "V" curve embedded in the plane: the teeth repeat
//! along the travel direction, but nothing repeats the curve ACROSS it, so a
//! page margin showed a single wandering stroke with large blank areas, and a
//! taller window did not gain a second row — it only let that same lone stroke
//! travel further before running off the bottom. Item 89 folds `(ry - center)`
//! through a row pitch, turning the single curve into the infinite family
//! `center + k * pitch`.
//!
//! **Defect 2 — the tiled field still did not COVER, and the law could not see
//! it.** Item 89's first cut set `pitch = period_px` (a square lattice in the
//! travel frame). Row `k`'s ribbon only ever visits `ry` in
//! `[k*pitch - amp - t, k*pitch + amp + t]`, so whenever `2*amp + 2*t <
//! period_px` the field carried a hard BLANK LANE of `period_px - 2*amp - 2*t`
//! px that NO chevron entered at ANY `rx` — on the then-authored Gumtree
//! (250/85) a ~70px lane every 250px. A short window's narrow margin could sit
//! wholly inside one: measured at the app's real adaptive column geometry, an
//! 80x182px band of a 1600x600 right margin and a 27x234px cell of a 1400x700
//! left margin at literally ZERO deviation. That shipped green because the
//! occupancy law of the day graded ONE fixed geometry (a 1200x800 canvas with
//! a synthetic 350/500 column) and never varied the aspect ratio — a law that
//! only tests the geometry its author happened to pick is how this class of
//! bug survives. Both halves are repaired here: the shader derives the pitch
//! from the profile (`row_h = 2*amp + thickness`, so consecutive rows ABUT by
//! construction at ANY dials — see `theme::Background::zigzag_row_pitch_px`),
//! and the occupancy law below SWEEPS window geometry.
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
//! **ITEM 100 — the teeth.** Item 89's own independent re-verify reverted the
//! pitch rule to the broken `row_h = period_px` and five of the thirteen laws
//! here went red — but the HEADLINE one,
//! [`zigzag_field_covers_every_margin_cell_across_a_viewport_sweep_on_both_worlds`],
//! survived it, at 1.43x its floor against 5.48x healthy. Its grid was a 3x3
//! partition of each margin — cells up to ~116x266 px, coarse enough that a
//! field with hard 38px blank lanes still dropped material into every one of
//! them. A headline law that survives the exact regression it names is
//! decorative. It is regraded here on a grid DERIVED from the field's own
//! geometry (see [`occupancy_cell_px`]) — 4712 cells across the same sweep
//! instead of 324, at the finest granularity the field's own claimed rhythm
//! can fill — and on a per-cell ink floor stated as a fraction of the field's
//! own structural areal density rather than a bare non-zero. With the shader's
//! pitch rule reverted it now fails on 1595 blank cells.
//!
//! Item 100 also records the RESIDUAL that grid deliberately does not grade:
//! below the minimal cell, voids DO exist and
//! [`zigzag_sub_pitch_voids_stay_isolated_pockets_never_a_lane`] pins their
//! shape (isolated, never adjacent) and population rather than hiding them.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use super::super::*;
use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use super::{headless_dqp, view, view_md};

/// The deterministic single-canvas scan surface used by the laws that are
/// about the FIELD's own shape rather than about a page margin (row rhythm,
/// height scaling, column exclusion, determinism): `capture::CANVAS_WIDTH` x
/// `CANVAS_HEIGHT`, the project's own "ordinary window" frame, with a page
/// column parked mid-canvas so both margins are wide enough (350px) to scan.
const W: u32 = 1200;
const H: u32 = 800;
const COL_LEFT: f32 = 350.0;
const COL_W: f32 = 500.0;
/// Per-pixel total-channel deviation from the mark-free pass that counts as
/// real material. The differential oracle cancels the dither exactly, so this
/// only has to clear 8-bit quantization.
const INK_FLOOR: i32 = 3;

// ---------------------------------------------------------------------------
// THE VIEWPORT SWEEP — the repair's own headline. Several HEIGHTS and several
// ASPECT RATIOS, both worlds, at the app's REAL adaptive column geometry.
// ---------------------------------------------------------------------------

/// `(window_w, window_h, page measure)` triples the occupancy law grades.
///
/// Heights run 500..1600 and aspect ratios 0.56..2.37, so no single shape can
/// dominate; the two entries the verified defect was measured at (1600x600 and
/// 1400x700, at the wide 86-char measure the report used) lead the list and are
/// named in their own comments. The MEASURE is swept alongside the window
/// because it — not the window width — is what sets how much margin there is
/// to fill: at a fixed 1400px window a 60-char measure leaves 236px margins and
/// a 100-char one leaves 16px slivers.
pub(super) const SWEEP: [(u32, u32, usize); 12] = [
    (1600, 600, 86), // VERIFIED DEFECT: right margin x=[1418,1600), 80px blank band.
    (1400, 700, 86), // VERIFIED DEFECT: left margin cell x=[53,80) y=[233,467), zero ink.
    (1500, 500, 86), // the shortest swept viewport — the sweep's smallest cells.
    (1200, 800, 70), // the canonical capture canvas at the default measure.
    (1200, 800, 86),
    (2000, 1000, 86),
    (1920, 1080, 70),
    (2560, 1080, 100), // ultrawide, and the widest measure (thin margins).
    (1280, 1024, 70),
    (1100, 900, 60),  // a small window at a narrow measure — the widest margins.
    (1000, 1400, 70), // portrait.
    (900, 1600, 60),  // tall portrait.
];

/// The floor under either edge of a graded cell (device px) — the granularity
/// the verified defect was reported at (a 26px-wide cell of an 80px margin),
/// and roughly `Dots`' own 24px lattice cell, this whisper-mark family's
/// reference for "fine-grained". A world whose derived cell
/// ([`occupancy_cell_px`]) comes out finer than this on an axis is graded at
/// this instead: below it a "cell" stops being a patch of ground a reader
/// could notice and becomes a few pixels of stroke.
const MIN_CELL: u32 = 26;
/// A cell must carry at least this FRACTION of the ink an average patch of the
/// same field carries — see [`cell_ink_floor`], which turns it into a pixel
/// count against the field's own structural areal density. A twentieth: the
/// minimal cell is, by construction, the smallest rect one row's ribbon can
/// cross, and a ribbon that clips only a corner of it legitimately leaves far
/// less than the mean (measured tightest across the sweep: 1.7x this floor on
/// Gumtree, 2.2x on Quokka).
const CELL_INK_FRAC_OF_STRUCTURAL: f32 = 0.05;
/// ...but never fewer than this many pixels, so a small cell cannot pass on a
/// handful of stray antialiased pixels.
const MIN_INKED_FLOOR: usize = 8;

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
    let bare = crate::background::BgDesc {
        density: 0.0,
        ..inked
    };
    // `drift` is item 87's Waves-only phase slot — inert `0.0` for this static
    // ground, and structurally unable to touch Zigzag's `params` dials.
    let a = render_bg(device, queue, inked, w, h, col_left, col_w, 0.0);
    let b = render_bg(device, queue, bare, w, h, col_left, col_w, 0.0);
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| {
            (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>()
        })
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

/// THE OCCUPANCY GRID'S CELL, derived from the FIELD's own geometry rather
/// than picked — `(width, height)` in device px, floored at [`MIN_CELL`] per
/// axis.
///
/// A chevron field is a family of ribbons stacked one row pitch `P` apart
/// along the ACROSS-TRAVEL axis. So the smallest axis-aligned rect that can be
/// asked to hold a whole row is the one whose longest chord in that direction
/// reaches `P`: travelling along `ry` = `(-sin a, cos a)` from inside a `w x h`
/// rect you leave through the vertical edge after `w/|sin a|` and through the
/// horizontal edge after `h/|cos a|`, so the longest across-travel chord is
/// `min(w/|sin a|, h/|cos a|)`, and it reaches `P` exactly when
/// `w >= P*|sin a|` AND `h >= P*|cos a|`. That pair IS the cell: Quokka's steep
/// 54° field grades 41x30 px, Gumtree's near-horizontal 15° one 33x122 px.
///
/// This is the whole reason the grid is not simply "as fine as possible".
/// ANYTHING SMALLER IS UNSATISFIABLE BY ANY SPARSE ROW FIELD — a narrower or
/// shorter rect can lie wholly between two neighbouring ribbons no matter how
/// perfect the coverage — so a law grading finer would be pinning noise, and a
/// law grading coarser (item 89's 3x3, up to 116x266 px) is the slack the
/// broken pitch rule hid inside. The measured population at exactly this
/// granularity is 4712 cells over the sweep, ZERO of them blank; at the broken
/// pitch rule, 1595 blank.
///
/// Both inputs come from the ONE owner each (`Background::zigzag_row_pitch_px`
/// — the host mirror of the shader's abutment rule — and `Background::angle`),
/// never a re-derivation here.
fn occupancy_cell_px(bg: theme::Background) -> (u32, u32) {
    let p = bg.zigzag_row_pitch_px();
    let (sa, ca) = (bg.angle().sin().abs(), bg.angle().cos().abs());
    (
        ((p * sa).ceil() as u32).max(MIN_CELL),
        ((p * ca).ceil() as u32).max(MIN_CELL),
    )
}

/// The pixel count a `cw x ch` cell of this field must carry to count as
/// occupied: [`CELL_INK_FRAC_OF_STRUCTURAL`] of the ink an average patch of
/// the same field carries, floored at [`MIN_INKED_FLOOR`].
///
/// The field's STRUCTURAL areal ink density is a closed form, and — the point
/// of stating the floor this way — a CONSTANT the abutment rule fixes: a
/// ribbon is `2*t` wide across a pitch of `2*amp + t`, so the inked fraction of
/// the plane is `2t/(2*amp + t)`, which is `~9.5%` for ANY dials once `t` is
/// `0.10*amp` (both shipping worlds, and every future one that stays off the
/// `1.2px` stroke floor). So this floor scales itself with a world's contrast
/// and dials instead of pinning one authored geometry's pixel count.
fn cell_ink_floor(bg: theme::Background, cw: u32, ch: u32) -> usize {
    let structural = 2.0 * bg.zigzag_stroke_px() / bg.zigzag_row_pitch_px().max(1.0);
    let need = structural * CELL_INK_FRAC_OF_STRUCTURAL * (cw * ch) as f32;
    (need.round() as usize).max(MIN_INKED_FLOOR)
}

/// The cell rects of one margin's occupancy grid, as `(x0, x1, y0, y1)` in
/// device px: a uniform tiling in cells of EXACTLY `cw x ch` (never a stretched
/// remainder — the cell size is the law's whole premise), with the last column
/// and row pulled flush against the far edge so they overlap their neighbour
/// instead of leaving an ungraded strip. Returns nothing at all for a margin
/// narrower than one cell (a hairline sliver of ground, not a field).
fn cells(x0: u32, x1: u32, h: u32, cw: u32, ch: u32) -> Vec<(u32, u32, u32, u32)> {
    let span = x1.saturating_sub(x0);
    if span < cw || h < ch {
        return Vec::new();
    }
    let (ncol, nrow) = (span.div_ceil(cw), h.div_ceil(ch));
    (0..nrow)
        .flat_map(|r| {
            let cy0 = (r * ch).min(h - ch);
            (0..ncol).map(move |c| {
                let cx0 = (x0 + c * cw).min(x1 - cw);
                (cx0, cx0 + cw, cy0, cy0 + ch)
            })
        })
        .collect()
}

/// `(inked pixels, local peak)` over a cell of the differential field.
fn cell_stats(field: &[i32], w: u32, cell: (u32, u32, u32, u32)) -> (usize, i32) {
    let (x0, x1, y0, y1) = cell;
    let mut inked = 0usize;
    let mut peak = 0i32;
    for y in y0..y1 {
        for x in x0..x1 {
            let v = field[(y * w + x) as usize];
            if v >= INK_FLOOR {
                inked += 1;
            }
            peak = peak.max(v);
        }
    }
    (inked, peak)
}

/// The two page margins `(x0, x1)` a column at `[col_left, col_left+col_w)`
/// leaves in a `w`-wide canvas.
pub(super) fn margins(w: u32, col_left: f32, col_w: f32) -> [(u32, u32); 2] {
    let l = col_left.max(0.0) as u32;
    let r = ((col_left + col_w).ceil() as u32).min(w);
    [(0, l.min(w)), (r, w)]
}

/// THE FIELD-OCCUPANCY LAW, SWEPT OVER VIEWPORT GEOMETRY — the repair's
/// headline claim, and the half of it that matters most: coverage must hold
/// INDEPENDENT of the window's height and aspect ratio.
///
/// For each of [`SWEEP`]'s twelve `(window, measure)` shapes, on BOTH worlds:
/// resolve the page column through the app's OWN adaptive-placement owner
/// (`TextPipeline::column_left`/`column_width` — the live geometry every
/// downstream reader composes, not a synthetic constant), tile each resulting
/// margin in cells of exactly [`occupancy_cell_px`] — the finest granularity
/// the field's own claimed row pitch can fill — and require EVERY cell to hold
/// real chevron material: at least [`cell_ink_floor`] inked pixels (a fraction
/// of the ink an average patch of the SAME field carries, not a bare non-zero)
/// AND a local peak at least half the field's own global peak (an actual ribbon
/// core crosses it, not a feather tail leaking in from a neighbour).
///
/// **ITEM 100 — why the grid moved.** As item 89 shipped it, this law graded a
/// 3x3 partition of each margin: 324 cells of up to 116x266 px, each needing 1%
/// of its own area inked. Item 89's independent re-verify reverted the shader's
/// pitch rule to the broken `row_h = period_px` — a field with a hard 38px lane
/// no chevron enters — and this law STILL PASSED, at 1.43x its floor against
/// 5.48x healthy: cells that big catch a row somewhere no matter how sparse the
/// field gets. The two laws that did catch it were the blank-LANE law and the
/// evenness bound. So the headline claim is now graded at the cell the field's
/// own geometry defines (4712 cells, ~7-40x smaller), where the same revert
/// leaves 1595 cells with literally zero ink.
///
/// What this law deliberately does NOT claim: coverage BELOW that cell. See
/// [`zigzag_sub_pitch_voids_stay_isolated_pockets_never_a_lane`], which grades
/// exactly that residual instead of leaving it unspoken.
#[test]
fn zigzag_field_covers_every_margin_cell_across_a_viewport_sweep_on_both_worlds() {
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping zigzag_field_covers_every_margin_cell_across_a_viewport_sweep_on_both_worlds: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_theme = theme::active().name;
    crate::page::set_page_on(true);

    let mut graded = 0usize;
    let mut tightest = f32::INFINITY;
    for (ww, wh, measure) in SWEEP {
        crate::page::set_measure(measure);
        p.set_size(ww as f32, wh as f32);
        for (name, bg) in zigzag_worlds() {
            theme::set_active_by_name(name).unwrap();
            p.sync_theme();
            // A heading-free prose view: the adaptive column's WIDE/symmetric
            // regime (the margin outline wants no rail), which is the geometry
            // the defect was verified at and the harder of the two — a
            // rail-shifted column trades a tiny right margin for a very wide
            // left one, and wide margins are the easy case. The RAIL regime is
            // swept by its own law below.
            p.set_view(&view("some plain prose here, no headings at all\n", 0, 0));
            let (col_left, col_w) = (p.column_left(), p.column_width());
            assert!(
                col_w > 0.0 && col_left >= 0.0,
                "{name}: {ww}x{wh}@{measure} produced a nonsense column {col_left}/{col_w}"
            );
            let field = mark_field(&device, &queue, bg, ww, wh, col_left, col_w);
            let peak = field.iter().copied().max().unwrap_or(0);
            assert!(
                peak >= 6,
                "{name}: {ww}x{wh}@{measure} — the zigzag field must reach real ink (peak {peak})"
            );
            let (cw, ch) = occupancy_cell_px(bg);
            let need = cell_ink_floor(bg, cw, ch);
            for (mi, (mx0, mx1)) in margins(ww, col_left, col_w).iter().enumerate() {
                for cell in cells(*mx0, *mx1, wh, cw, ch) {
                    let (x0, x1, y0, y1) = cell;
                    let (inked, cell_peak) = cell_stats(&field, ww, cell);
                    graded += 1;
                    tightest = tightest.min(inked as f32 / need as f32);
                    assert!(
                        inked >= need,
                        "{name}: {ww}x{wh}@{measure} margin {mi} cell x[{x0},{x1}) \
                         y[{y0},{y1}) ({cw}x{ch}) is BLANK ({inked} inked px, need {need}) — \
                         the zigzag field must tile every margin at every viewport shape, not \
                         only at the one geometry its author picked, and at the cell its own \
                         row pitch defines, not at whatever grid happens to hide the gaps"
                    );
                    assert!(
                        cell_peak * 2 >= peak,
                        "{name}: {ww}x{wh}@{measure} margin {mi} cell x[{x0},{x1}) \
                         y[{y0},{y1}) only catches a feather (local peak {cell_peak} vs field \
                         peak {peak}) — no chevron ribbon core crosses it"
                    );
                }
            }
        }
    }
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    theme::set_active_by_name(was_theme).unwrap();

    // NON-VACUITY: the sweep really did grade a substantial population of
    // cells (a geometry list that silently produced no margins would pass
    // every assertion above). Measured 4712 at item 100's derived cell — the
    // bound is set well under it so an authored dial change may move the cell
    // size without tripping a bookkeeping assert, but a grid that collapsed
    // back toward item 89's 324 could not survive it.
    assert!(
        graded >= 2_000,
        "the viewport sweep graded only {graded} margin cells — it is not exercising \
         the geometry axis it exists for at the granularity item 100 set"
    );
    eprintln!(
        "zigzag sweep: {graded} margin cells graded, tightest occupancy {tightest:.2}x the floor"
    );
}

/// THE RAIL-REGIME ARM of the sweep: the adaptive column's OTHER placement
/// regime. On a markdown buffer with headings the margin outline claims a rail,
/// so `TextPipeline::column_left` shifts the column right — a very wide left
/// margin against a thin right one, an ASYMMETRY the heading-free arm above
/// never produces. Graded by the same cells, from the same live owner.
#[test]
fn zigzag_field_covers_every_margin_cell_in_the_outline_rail_regime_too() {
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping zigzag_field_covers_every_margin_cell_in_the_outline_rail_regime_too: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_outline = crate::outline::outline_on();
    let was_theme = theme::active().name;
    crate::page::set_page_on(true);
    crate::outline::set_outline_on(true);

    let mut shifted = 0usize;
    for (ww, wh, measure) in SWEEP {
        crate::page::set_measure(measure);
        p.set_size(ww as f32, wh as f32);
        for (name, bg) in zigzag_worlds() {
            theme::set_active_by_name(name).unwrap();
            p.sync_theme();
            p.set_view(&view_md("# One\n\ntext\n\n## Two\n\nmore\n", 0, 0));
            let (col_left, col_w) = (p.column_left(), p.column_width());
            // Only the geometries where the rail actually MOVED the column are
            // this law's business; elsewhere it would re-run the arm above.
            p.set_view(&view("plain\n", 0, 0));
            let symmetric = p.column_left();
            p.set_view(&view_md("# One\n\ntext\n\n## Two\n\nmore\n", 0, 0));
            if (col_left - symmetric).abs() < 1.0 {
                continue;
            }
            shifted += 1;
            let field = mark_field(&device, &queue, bg, ww, wh, col_left, col_w);
            let peak = field.iter().copied().max().unwrap_or(0);
            let (cw, ch) = occupancy_cell_px(bg);
            let need = cell_ink_floor(bg, cw, ch);
            for (mi, (mx0, mx1)) in margins(ww, col_left, col_w).iter().enumerate() {
                for cell in cells(*mx0, *mx1, wh, cw, ch) {
                    let (x0, x1, y0, y1) = cell;
                    let (inked, cell_peak) = cell_stats(&field, ww, cell);
                    assert!(
                        inked >= need,
                        "{name}: RAIL {ww}x{wh}@{measure} margin {mi} cell x[{x0},{x1}) \
                         y[{y0},{y1}) is BLANK ({inked} inked px, need {need})"
                    );
                    assert!(
                        cell_peak * 2 >= peak,
                        "{name}: RAIL {ww}x{wh}@{measure} margin {mi} cell x[{x0},{x1}) \
                         y[{y0},{y1}) only catches a feather ({cell_peak} vs {peak})"
                    );
                }
            }
        }
    }
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    crate::outline::set_outline_on(was_outline);
    theme::set_active_by_name(was_theme).unwrap();
    assert!(
        shifted > 0,
        "no swept geometry actually entered the outline-rail regime — this arm proves nothing"
    );
}

// ---------------------------------------------------------------------------
// THE BLANK-LANE LAW — the defect's own signature, measured directly.
// ---------------------------------------------------------------------------

/// The tallest run of consecutive scanlines in `[y0, y1)` on which NO pixel of
/// `[x0, x1)` reaches the ribbon `core` — i.e. the height of the widest fully
/// BLANK horizontal band of the field. This is exactly the measurement the
/// verified defect report made (an 80px band of a 1600x600 right margin at zero
/// deviation).
fn widest_blank_band(field: &[i32], w: u32, x0: u32, x1: u32, y0: u32, y1: u32, core: i32) -> u32 {
    let mut widest = 0u32;
    let mut run = 0u32;
    for y in y0..y1 {
        let any = (x0..x1).any(|x| field[(y * w + x) as usize] >= core);
        if any {
            run = 0;
        } else {
            run += 1;
            widest = widest.max(run);
        }
    }
    widest
}

/// THE MARGIN-EVENNESS BOUND (device px). The abutment rule kills the field's
/// blank LANE — the band no chevron enters at ANY x. It cannot kill the
/// ordinary gap BETWEEN two neighbouring rows, and no sparse row field can: a
/// margin only 80px wide gives a shallow ribbon too little x to travel a whole
/// pitch in, so some horizontal band of it inevitably falls between two rows.
/// What the repair buys there is a much EVENER rhythm. Measured over this
/// file's whole viewport sweep, the widest fully-blank horizontal band of any
/// margin (worst case both times: the 80px-wide left margin at 1400x700):
///
/// | field | widest blank margin band |
/// |---|---|
/// | item 89 as it shipped in 49a84d5 (period lattice, 250/85) | 174px |
/// | the same pitch rule at TODAY's dials (170/60) — the controlled arm | 102px |
/// | shipped abutment rule, 170/60 | 57px (60px at real GPU pixels) |
/// | shipped abutment rule, Quokka 100/24 | 19px |
///
/// This constant PINS that gain so a future dial edit cannot quietly give it
/// back. Law: [`zigzag_margin_bands_stay_even_across_the_sweep`].
const MAX_BLANK_BAND_PX: u32 = 72;

/// The widest run of consecutive 1px ACROSS-TRAVEL (`ry`) bins that no ribbon
/// core reaches — the field's blank LANE, measured on the axis the rows
/// actually stack along rather than on screen `y` (at any nonzero travel angle
/// a horizontal scanline sweeps hundreds of px of `ry`, so a screen-`y` scan
/// cannot see a lane at all — the trap this helper exists to avoid).
///
/// The outermost bins are dropped: only the corners of the canvas map there, so
/// they are sampled far too thinly to judge.
fn widest_blank_ry_lane(field: &[i32], w: u32, h: u32, angle: f32, core: i32) -> u32 {
    let (ca, sa) = (angle.cos(), angle.sin());
    let ry_of = |x: f32, y: f32| -x * sa + y * ca;
    let corners = [
        ry_of(0.0, 0.0),
        ry_of(w as f32, 0.0),
        ry_of(0.0, h as f32),
        ry_of(w as f32, h as f32),
    ];
    let lo = corners.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = corners.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let n = (hi - lo).ceil().max(1.0) as usize + 1;
    let mut hit = vec![false; n];
    let mut sampled = vec![0u32; n];
    for y in 0..h {
        for x in 0..w {
            let b = (ry_of(x as f32, y as f32) - lo).round() as usize;
            let b = b.min(n - 1);
            sampled[b] += 1;
            if field[(y * w + x) as usize] >= core {
                hit[b] = true;
            }
        }
    }
    // Judge only the well-sampled interior (at least 1/4 of the median bin's
    // population) — the extreme bins are single-corner slivers.
    let mut pops: Vec<u32> = sampled.iter().copied().filter(|&v| v > 0).collect();
    pops.sort_unstable();
    let floor = pops.get(pops.len() / 2).copied().unwrap_or(0) / 4;
    let mut widest = 0u32;
    let mut run = 0u32;
    for i in 0..n {
        if sampled[i] < floor.max(1) {
            run = 0;
            continue;
        }
        if hit[i] {
            run = 0;
        } else {
            run += 1;
            widest = widest.max(run);
        }
    }
    widest
}

/// THE NO-BLANK-LANE LAW at real pixels: over a FULL canvas of margin (no page
/// column — the surface the field's own periodicity is a statement about),
/// there is not one value of the across-travel coordinate the chevron family
/// fails to reach, on either world, at four canvas shapes. This is the pixel
/// form of the abutment rule: each row's ribbon core sweeps `2*amp + 1.2*t` of
/// the across-travel axis against a pitch of `2*amp + t`, so consecutive rows
/// OVERLAP and the family covers every value of `ry`.
///
/// Item 89's first cut (pitch = `period_px`) left a lane of
/// `period_px - 2*amp - 2*t` — the host mirror below reproduces it, so this
/// law is proven capable of failing.
#[test]
fn the_zigzag_family_leaves_no_blank_lane_across_its_travel_axis() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping the_zigzag_family_leaves_no_blank_lane_across_its_travel_axis: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        for (w, h) in [(1200u32, 800u32), (1600, 600), (900, 1600), (2000, 1000)] {
            let field = mark_field(&device, &queue, bg, w, h, 0.0, 0.0);
            let peak = field.iter().copied().max().unwrap_or(0);
            let core = (peak / 2).max(INK_FLOOR);
            let lane = widest_blank_ry_lane(&field, w, h, bg.angle(), core);
            assert_eq!(
                lane, 0,
                "{name}: {w}x{h} — the chevron family misses {lane}px of its own across-travel \
                 axis entirely (a blank LANE). The row pitch must abut the profile's own \
                 excursion (`Background::zigzag_row_pitch_px`), never exceed it"
            );
        }
    }
}

/// THE TOOTH BOUND on the no-lane guarantee (item 100) — the fixture behind
/// the honest wording of `Background::zigzag_row_pitch_px`'s own doc comment.
///
/// That comment used to claim the abutment rule leaves no blank lane "at ANY
/// dials, at any angle, at any viewport size or aspect". The first two thirds
/// are provable and proven (over the PLANE: [`the_abutment_rule_forbids_a_blank_lane_at_every_dial_setting`]).
/// The last clause was false, and this is the counterexample: a viewport sees a
/// ROTATED RECTANGLE of that plane, not a whole strip of it, so it inherits the
/// guarantee only when it actually holds the travel the excursion needs. Give
/// the field a tooth wider than the canvas's own travel extent — `period_px`
/// 2000 against the 1365px a 1200x800 canvas spans along a 15° travel axis —
/// and the triangle wave never completes a cycle in view: part of the
/// across-travel axis is never visited, measured at a 78px blank lane.
///
/// The regime is bounded by the TOOTH, and `theme::ZIGZAG_MAX_ROW_PITCH_PX`
/// does not bound the tooth (it bounds the PITCH, which the tooth no longer
/// sets). So the second half of this law is the authored guard that keeps
/// shipping worlds far outside it: every `Zigzag` world's `period_px` must sit
/// under a quarter of the travel extent of the SHORTEST viewport this file
/// sweeps — Gumtree's 170px tooth against a 394px allowance, Quokka's 100px
/// against 320px.
#[test]
fn the_no_lane_guarantee_is_bounded_by_the_tooth_the_viewport_can_hold() {
    // The data half needs no GPU, so it runs on an adapter-less machine too.
    let travel = |w: u32, h: u32, a: f32| w as f32 * a.cos().abs() + h as f32 * a.sin().abs();
    let mut seen = 0usize;
    for t in theme::THEMES {
        if !matches!(t.background, theme::Background::Zigzag { .. }) {
            continue;
        }
        seen += 1;
        let bg = t.background;
        let shortest = SWEEP
            .iter()
            .map(|(w, h, _)| travel(*w, *h, bg.angle()))
            .fold(f32::INFINITY, f32::min);
        assert!(
            bg.period_px() * 4.0 <= shortest,
            "{}: a {}px tooth against only {shortest:.0}px of travel on the sweep's shortest \
             viewport — a field whose tooth the viewport cannot comfortably hold never sweeps \
             its full excursion in view, and the abutment rule's no-lane guarantee (a claim \
             about the PLANE) stops transferring to the margin",
            t.name,
            bg.period_px()
        );
    }
    assert!(
        seen >= 2,
        "expected both Zigzag worlds in the roster (saw {seen})"
    );

    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping the GPU half of the_no_lane_guarantee_is_bounded_by_the_tooth_the_viewport_can_hold: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    // An OFF-ROSTER dial pair, deliberately: this is the boundary of the
    // guarantee, not a world anyone ships.
    let huge_tooth = theme::Background::Zigzag {
        from: theme::Srgb::rgb(20, 30, 20),
        to: theme::Srgb::rgb(40, 60, 40),
        dir: (0.0, 1.0),
        tint: theme::Srgb::rgb(240, 240, 240),
        period_px: 2000.0,
        amplitude_px: 150.0,
        angle: 0.26,
        density: 0.6,
        banded: false,
    };
    let (w, h) = (W, H);
    let field = mark_field(&device, &queue, huge_tooth, w, h, 0.0, 0.0);
    let peak = field.iter().copied().max().unwrap_or(0);
    let lane = widest_blank_ry_lane(&field, w, h, huge_tooth.angle(), (peak / 2).max(INK_FLOOR));
    assert!(
        lane >= 30,
        "a {}px tooth on a {w}x{h} canvas ({:.0}px of travel) left only a {lane}px lane — this \
         fixture exists to hold `zigzag_row_pitch_px`'s doc comment honest about the ONE regime \
         the abutment rule does not cover; if the regime is gone, the comment must change with it",
        huge_tooth.period_px(),
        travel(w, h, huge_tooth.angle())
    );
    eprintln!("zigzag tooth bound: a 2000px tooth on {w}x{h} leaves a {lane}px lane");
}

/// The sub-pitch grid the RESIDUAL is graded on: [`MIN_CELL`] wide by this
/// tall, the granularity item 89's re-verify reported its pockets at. It is
/// deliberately FINER than [`occupancy_cell_px`] on at least one axis for both
/// worlds (Quokka's cell needs 41px of width, Gumtree's 122px of height), i.e.
/// finer than any sparse row field can guarantee.
const SUB_PITCH_CELL_H: u32 = 100;
/// The share of sub-pitch cells that may come up blank, per world. Measured
/// across the sweep: Quokka 0.21% (3 of 1424), Gumtree 4.71% (67 of 1424); at
/// the broken `row_h = period_px` pitch rule, 15.4% and 18.9%.
const MAX_SUB_PITCH_VOID_FRAC: f32 = 0.08;

/// THE RESIDUAL, PINNED (item 100) — what the occupancy law deliberately does
/// not claim, measured rather than left unspoken.
///
/// Grade the same swept margins on a grid FINER than the field's own row pitch
/// can fill ([`MIN_CELL`] x [`SUB_PITCH_CELL_H`]) and blank cells do appear:
/// ~30x100px lens-shaped pockets between two neighbouring ribbons, flanked by
/// ink on both sides. Item 89's re-verify named three of them on Gumtree
/// (1200x800@m70 `x[32,64) y[300,400)`, 1400x700@m86 `x[1346,1373) y[0,100)`
/// and `x[1373,1400) y[500,600)`); this sweep's own tiling finds 67 of 1424,
/// the same shape at more offsets, and 3 on Quokka.
///
/// **The decision, stated rather than tuned away: PINNED as a bounded
/// residual, not failed on.** A pocket that size is not a lane and not a
/// coverage defect — it is what a SPARSE ROW field IS below its own pitch, and
/// no shader change can remove it: shrinking the pitch enough to fill a
/// sub-pitch cell means a denser field, which is an authored TASTE dial
/// (`amplitude_px`), not a correctness rule. Failing here would therefore
/// pressure the next dial edit toward density for a law's sake. What CAN be
/// held, and is held here, is the residual's SHAPE and SIZE:
///
/// - **isolated** — no blank cell touches another, in either axis, anywhere in
///   the sweep. A void may be a pocket; it may never join its neighbour into a
///   band, a column, or a lane. (Measured: longest blank run 1 cell both ways;
///   at the broken pitch rule, 2 vertically and 3 horizontally.)
/// - **rare** — at most [`MAX_SUB_PITCH_VOID_FRAC`] of a world's sub-pitch
///   cells.
///
/// Both bounds go red at the broken pitch rule, so this is a second detector
/// rather than a place to file the inconvenient cells. Zero pockets is allowed
/// (a future denser dial pair is an improvement, not a violation) — only the
/// doc's recorded numbers would go stale, and the printed line below reports
/// them every run.
#[test]
fn zigzag_sub_pitch_voids_stay_isolated_pockets_never_a_lane() {
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping zigzag_sub_pitch_voids_stay_isolated_pockets_never_a_lane: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_theme = theme::active().name;
    crate::page::set_page_on(true);

    for (name, bg) in zigzag_worlds() {
        let (mut total, mut blank) = (0usize, 0usize);
        for (ww, wh, measure) in SWEEP {
            crate::page::set_measure(measure);
            p.set_size(ww as f32, wh as f32);
            theme::set_active_by_name(name).unwrap();
            p.sync_theme();
            p.set_view(&view("some plain prose here, no headings at all\n", 0, 0));
            let (col_left, col_w) = (p.column_left(), p.column_width());
            let field = mark_field(&device, &queue, bg, ww, wh, col_left, col_w);
            for (mi, (mx0, mx1)) in margins(ww, col_left, col_w).iter().enumerate() {
                // A PLAIN tiling here (no flush-to-edge overlap): adjacency is
                // the claim, so neighbouring cells must not share pixels.
                let (ncol, nrow) = (mx1.saturating_sub(*mx0) / MIN_CELL, wh / SUB_PITCH_CELL_H);
                if ncol == 0 || nrow == 0 {
                    continue;
                }
                let mut void = vec![false; (ncol * nrow) as usize];
                for r in 0..nrow {
                    for c in 0..ncol {
                        let cell = (
                            mx0 + c * MIN_CELL,
                            mx0 + (c + 1) * MIN_CELL,
                            r * SUB_PITCH_CELL_H,
                            (r + 1) * SUB_PITCH_CELL_H,
                        );
                        total += 1;
                        if cell_stats(&field, ww, cell).0 == 0 {
                            blank += 1;
                            void[(r * ncol + c) as usize] = true;
                        }
                    }
                }
                for r in 0..nrow {
                    for c in 0..ncol {
                        if !void[(r * ncol + c) as usize] {
                            continue;
                        }
                        let right = c + 1 < ncol && void[(r * ncol + c + 1) as usize];
                        let below = r + 1 < nrow && void[((r + 1) * ncol + c) as usize];
                        assert!(
                            !right && !below,
                            "{name}: {ww}x{wh}@{measure} margin {mi} — the blank sub-pitch cell \
                             at column {c} row {r} (x[{},{}) y[{},{})) has a blank neighbour \
                             ({}{}). A residual void may be an isolated pocket between two \
                             ribbons; two touching voids are a BAND, and a band is the blank \
                             lane the abutment rule exists to forbid",
                            mx0 + c * MIN_CELL,
                            mx0 + (c + 1) * MIN_CELL,
                            r * SUB_PITCH_CELL_H,
                            (r + 1) * SUB_PITCH_CELL_H,
                            if right { "to its right" } else { "" },
                            if below { " below it" } else { "" },
                        );
                    }
                }
            }
        }
        assert!(
            total >= 1_000,
            "{name}: only {total} sub-pitch cells graded — this law is not measuring the \
             residual it exists to bound"
        );
        let frac = blank as f32 / total as f32;
        eprintln!(
            "zigzag sub-pitch residual on {name}: {blank} blank of {total} cells ({:.2}%)",
            100.0 * frac
        );
        assert!(
            frac <= MAX_SUB_PITCH_VOID_FRAC,
            "{name}: {blank} of {total} sub-pitch cells ({:.2}%) are blank, past the \
             {:.0}% bound item 100 pinned the residual at — the pockets stopped being a \
             residual and became the field's character",
            100.0 * frac,
            100.0 * MAX_SUB_PITCH_VOID_FRAC
        );
    }
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    theme::set_active_by_name(was_theme).unwrap();
}

/// THE MARGIN-EVENNESS LAW, swept: over every geometry in [`SWEEP`], on both
/// worlds, at the app's real column geometry, no margin carries a fully-blank
/// horizontal band taller than [`MAX_BLANK_BAND_PX`].
///
/// This is the companion the occupancy grid alone cannot give: a grid says
/// every CELL holds material, which a field with one tall stripe of nothing
/// straddling a cell boundary could still satisfy. Measured exactly the way the
/// verification report measured the defect it names (scan the margin's full
/// width per scanline; count the longest run with no material anywhere on it).
#[test]
fn zigzag_margin_bands_stay_even_across_the_sweep() {
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping zigzag_margin_bands_stay_even_across_the_sweep: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_theme = theme::active().name;
    crate::page::set_page_on(true);

    let mut worst = (0u32, String::new());
    for (ww, wh, measure) in SWEEP {
        crate::page::set_measure(measure);
        p.set_size(ww as f32, wh as f32);
        for (name, bg) in zigzag_worlds() {
            theme::set_active_by_name(name).unwrap();
            p.sync_theme();
            p.set_view(&view("some plain prose here, no headings at all\n", 0, 0));
            let (col_left, col_w) = (p.column_left(), p.column_width());
            let field = mark_field(&device, &queue, bg, ww, wh, col_left, col_w);
            for (mi, (mx0, mx1)) in margins(ww, col_left, col_w).iter().enumerate() {
                if mx1.saturating_sub(*mx0) < MIN_CELL {
                    continue;
                }
                let band = widest_blank_band(&field, ww, *mx0, *mx1, 0, wh, INK_FLOOR);
                if band > worst.0 {
                    worst = (band, format!("{name} {ww}x{wh}@{measure} margin {mi}"));
                }
            }
        }
    }
    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    theme::set_active_by_name(was_theme).unwrap();
    eprintln!(
        "zigzag widest blank margin band: {}px ({})",
        worst.0, worst.1
    );
    assert!(
        worst.0 <= MAX_BLANK_BAND_PX,
        "{} carries a {}px fully-blank horizontal band — past the {MAX_BLANK_BAND_PX}px \
         evenness bound item 89's coverage repair bought",
        worst.1,
        worst.0
    );
}

// ---------------------------------------------------------------------------
// FIELD-SHAPE laws on the deterministic canvas.
// ---------------------------------------------------------------------------

/// THE ROW-RHYTHM LAW (the two worlds' authored characters): on the
/// deterministic 1200x800 canvas Gumtree's broad field crosses a mid-margin
/// vertical scanline as a handful of broad rows and Quokka's tighter one as
/// distinctly MORE. Ranges, not pinned numbers — the claim is the READING
/// ("broad rows on Gumtree; Quokka tighter"), and the strict inequality is what
/// keeps the two worlds from converging on one rhythm.
///
/// The bands moved with the coverage repair, and the move is the honest record
/// of its cost: the row pitch is no longer free to be as broad as the tooth
/// wavelength (that freedom WAS the blank lane), so Gumtree reads ~5 broad rows
/// down an ordinary window where item 89's first cut read ~3. Its BROADNESS now
/// lives in the tooth wavelength (170px against Quokka's 100px) and in a row
/// pitch still 2.5x Quokka's. (The two numbers read 480px and 2.7x until item
/// 100 — a stale record of a dial pair that never shipped.)
#[test]
fn zigzag_reads_as_broad_rows_on_gumtree_and_a_tighter_field_on_quokka() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping zigzag_reads_as_broad_rows_on_gumtree_and_a_tighter_field_on_quokka: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    // The expected band travels WITH the world, so there is no name match in a
    // render-side assertion (the design-is-data discipline `theme_caps_law`
    // enforces for real render code, kept here by habit).
    let expected = [
        ("Quokka", theme::QUOKKA.background, 15..=18),
        ("Gumtree", theme::GUMTREE.background, 5..=9),
    ];
    let mut tightest = Vec::new();
    for (name, bg, band) in expected {
        let field = mark_field(&device, &queue, bg, W, H, COL_LEFT, COL_W);
        let peak = field.iter().copied().max().unwrap_or(0);
        // Both margins, mid-cell scanlines: the rhythm is a property of the
        // field, not of one lucky x.
        let xs = [40u32, 175, 310, 890, 1025, 1160];
        let n: Vec<usize> = xs
            .iter()
            .map(|&x| row_crossings(&field, W, H, x, peak))
            .collect();
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
        tightest[0],
        tightest[1]
    );
}

#[test]
fn quokka_filled_bands_alternate_over_real_pixels_and_match_the_host_phase() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let field = mark_field(&device, &queue, theme::QUOKKA.background, W, H, 0.0, 0.0);
    let x = W / 2;
    let mut runs = Vec::new();
    let mut host_runs = Vec::new();
    let mut last = None;
    let mut last_host = None;
    for y in 0..H {
        let gpu = field[(y * W + x) as usize] >= INK_FLOOR;
        let host = zigzag_coverage(
            x as f32,
            y as f32,
            theme::QUOKKA.background,
            PitchRule::Abut,
        ) >= theme::QUOKKA.background.density() * 0.5;
        if last != Some(gpu) {
            runs.push(gpu);
            last = Some(gpu);
        }
        if last_host != Some(host) {
            host_runs.push(host);
            last_host = Some(host);
        }
    }
    assert!(
        runs.len() >= 12,
        "expected repeated filled/untouched Quokka lanes, got {runs:?}"
    );
    assert!(
        runs.windows(2).all(|w| w[0] != w[1]),
        "Quokka lanes must alternate: {runs:?}"
    );
    assert!(
        runs.contains(&true) && runs.contains(&false),
        "both filled and untouched lanes must exist"
    );
    assert!(
        (runs.len() as isize - host_runs.len() as isize).abs() <= 1,
        "host/GPU cadence: gpu={runs:?}, host={host_runs:?}"
    );
}

#[test]
fn gumtree_outline_upload_remains_byte_identical_to_the_unbanded_descriptor() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let field = mark_field(
        &device,
        &queue,
        theme::GUMTREE.background,
        W,
        H,
        COL_LEFT,
        COL_W,
    );
    let theme::Background::Zigzag {
        from,
        to,
        dir,
        tint,
        period_px,
        amplitude_px,
        angle,
        density,
        ..
    } = theme::GUMTREE.background
    else {
        unreachable!()
    };
    let explicit_outline = theme::Background::Zigzag {
        from,
        to,
        dir,
        tint,
        period_px,
        amplitude_px,
        angle,
        density,
        banded: false,
    };
    assert_eq!(
        field,
        mark_field(&device, &queue, explicit_outline, W, H, COL_LEFT, COL_W)
    );
}

/// THE HEIGHT-SCALING LAW — the original defect's own signature, inverted. Item
/// 86's single stroke did not repeat across the field, so DOUBLING the canvas
/// height bought no extra rows (the same lone stroke merely travelled further
/// before leaving the canvas: measured row counts stayed put while 62% of a tall
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
        eprintln!(
            "skipping zigzag_contributes_zero_ink_inside_the_writing_column_on_both_worlds: no wgpu adapter"
        );
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
        eprintln!(
            "skipping zigzag_renders_byte_identically_across_independent_draws_at_two_canvas_sizes: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    for (name, bg) in zigzag_worlds() {
        let desc = bg_desc_for(bg);
        for (w, h, cl, cw) in [
            (W, H, COL_LEFT, COL_W),
            (900u32, 1600u32, 200.0f32, 400.0f32),
        ] {
            let a = render_bg(&device, &queue, desc, w, h, cl, cw, 0.0);
            let b = render_bg(&device, &queue, desc, w, h, cl, cw, 0.0);
            assert_eq!(
                a, b,
                "{name}: two draws of the identical desc diverged at {w}x{h}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// THE SHADER MIRROR — the host-side model of the coverage function, used for
// the ABUTMENT theorem (which is a claim about ALL dials, not just two worlds'
// authored pair) and for the non-vacuity proofs. Kept in lockstep with the GPU
// by its own law.
// ---------------------------------------------------------------------------

/// Which rule sets the field's row pitch. `Abut` is what ships (the pitch is
/// derived from the profile, `theme::Background::zigzag_row_pitch_px`);
/// `PeriodLattice` is item 89's FIRST cut (pitch = `period_px`), kept ONLY so
/// the laws above can be proven capable of catching the defect it caused.
#[derive(Clone, Copy, PartialEq)]
enum PitchRule {
    Abut,
    PeriodLattice,
}

/// A minimal host-side mirror of `pattern_coverage`'s `shader == 7u` branch
/// (`shaders/background.wgsl`). Every real appearance claim in this file is
/// measured off the GPU; this exists for the all-dials theorem and the
/// non-vacuity self-proofs. Held in lockstep by
/// [`the_host_mirror_agrees_with_the_gpus_own_row_rhythm`].
fn zigzag_coverage(x: f32, y: f32, bg: theme::Background, rule: PitchRule) -> f32 {
    let period = bg.period_px().max(1.0);
    let a = bg.angle();
    let amp = bg.amplitude_px().max(0.0);
    let dens = bg.density().clamp(0.0, 1.0);
    let (ca, sa) = (a.cos(), a.sin());
    let rx = x * ca + y * sa;
    let ry = -x * sa + y * ca;
    let fr = |v: f32| v - v.floor();
    let tri = (fr(rx / period) * 2.0 - 1.0).abs() * 2.0 - 1.0;
    let center = tri * amp;
    let thickness = bg.zigzag_stroke_px();
    let row_h = match rule {
        PitchRule::Abut => bg.zigzag_row_pitch_px(),
        PitchRule::PeriodLattice => period,
    };
    let d = (fr((ry - center) / row_h + 0.5) - 0.5).abs() * row_h;
    if bg.zigzag_banded() {
        return if fr((ry - center) / row_h) < 0.5 {
            dens
        } else {
            0.0
        };
    }
    let t = ((d - thickness * 0.6) / (thickness - thickness * 0.6)).clamp(0.0, 1.0);
    let line = 1.0 - t * t * (3.0 - 2.0 * t);
    line * dens
}

/// THE ABUTMENT THEOREM, over ARBITRARY dials — the structural half of the
/// repair, and the reason it is not a tuning. For a wide deterministic sweep of
/// `(period_px, amplitude_px, angle)` (including degenerate corners: a zero
/// profile, a hairline profile under the stroke floor, an excursion many times
/// the wavelength), the ribbon CORE's own across-travel sweep
/// (`2*amp + 1.2*thickness`) is at least the row pitch (`2*amp + thickness`) —
/// so consecutive rows always overlap and a blank lane is IMPOSSIBLE, at any
/// dials, any angle, any viewport. Item 89's first cut satisfied this for NO
/// dial pair with `period_px > 2*amp + 2*thickness`, which is where both
/// shipping worlds sat.
#[test]
fn the_abutment_rule_forbids_a_blank_lane_at_every_dial_setting() {
    let mut checked = 0usize;
    let mut first_cut_lanes = 0usize;
    for pi in 0..40u32 {
        for ai in 0..40u32 {
            let period = 1.0 + pi as f32 * 25.0;
            let amp = ai as f32 * 12.5;
            let bg = theme::Background::Zigzag {
                from: theme::Srgb::rgb(0, 0, 0),
                to: theme::Srgb::rgb(0, 0, 0),
                dir: (0.0, 1.0),
                tint: theme::Srgb::rgb(0, 0, 0),
                period_px: period,
                amplitude_px: amp,
                angle: 0.1 + (pi % 7) as f32 * 0.2,
                density: 0.5,
                banded: false,
            };
            let t = bg.zigzag_stroke_px();
            let pitch = bg.zigzag_row_pitch_px();
            let core_sweep = 2.0 * (amp + 0.6 * t);
            assert!(
                core_sweep >= pitch - 1e-4,
                "period {period} / amplitude {amp}: the ribbon core sweeps {core_sweep} of a \
                 {pitch} pitch — a blank lane of {} px opens between rows",
                pitch - core_sweep
            );
            checked += 1;
            // The first cut's lane, at these same dials — the witness that the
            // property above is a REAL constraint and not a tautology.
            if period - core_sweep > 1.0 {
                first_cut_lanes += 1;
            }
        }
    }
    assert!(
        checked >= 1_000,
        "the dial sweep must be substantial (checked {checked})"
    );
    assert!(
        first_cut_lanes * 3 >= checked,
        "item 89's first cut (pitch = period_px) must leave a blank lane across a large part \
         of the dial space ({first_cut_lanes} of {checked}) — otherwise this theorem proves \
         nothing"
    );
}

/// NON-VACUITY, in two parts, through the host mirror in BOTH pitch rules with
/// Gumtree's OWN authored dials.
///
/// **(a) The LANE.** Measured on the across-travel axis the rows stack along
/// (the same axis [`widest_blank_ry_lane`] scans), the first cut leaves a band
/// no chevron enters at any point of the plane; the shipped abutment rule
/// leaves NONE. So [`the_zigzag_family_leaves_no_blank_lane_across_its_travel_axis`]
/// is a discriminating check, not a formality any chevron would pass.
///
/// **(b) The MARGIN band.** Restricted to the 182px-wide 1600x600 right margin
/// (`x=[1418,1600)`) the verification report measured, the first cut's widest
/// fully-blank band is at least twice the shipped field's. Stated as a RATIO,
/// not "the shipped field leaves none": inside a margin that narrow a shallow
/// ribbon has too little x to travel a whole pitch, so an ordinary
/// between-rows gap is unavoidable there for ANY sparse row field — the honest
/// claim is that it got much smaller and much more regular, which is what
/// [`zigzag_margin_bands_stay_even_across_the_sweep`] pins.
#[test]
fn the_first_cuts_pitch_rule_reopens_the_verified_blank_band_and_abutment_closes_it() {
    let bg = theme::GUMTREE.background;
    let core = 0.5 * bg.density();
    let widest = |rule: PitchRule, x0: u32, x1: u32, h: u32| -> u32 {
        let mut widest = 0u32;
        let mut run = 0u32;
        for y in 0..h {
            let any = (x0..x1).any(|x| zigzag_coverage(x as f32, y as f32, bg, rule) >= core);
            if any {
                run = 0;
            } else {
                run += 1;
                widest = widest.max(run);
            }
        }
        widest
    };
    // (a) the LANE, on the across-travel axis: walk `ry` directly at a fixed
    // `rx` sample set — the mirror is a pure function, so this needs no image.
    let lane = |rule: PitchRule| -> u32 {
        let (ca, sa) = (bg.angle().cos(), bg.angle().sin());
        let mut widest = 0u32;
        let mut run = 0u32;
        for ry in 0..1200u32 {
            let any = (0..240).any(|k| {
                let rx = k as f32 * 4.0;
                let (x, y) = (rx * ca - ry as f32 * sa, rx * sa + ry as f32 * ca);
                zigzag_coverage(x, y, bg, rule) >= core
            });
            if any {
                run = 0;
            } else {
                run += 1;
                widest = widest.max(run);
            }
        }
        widest
    };
    let lane_first_cut = lane(PitchRule::PeriodLattice);
    let lane_shipped = lane(PitchRule::Abut);
    assert!(
        lane_first_cut >= 30,
        "the first cut's pitch rule must leave a real blank LANE across a full scan surface \
         (got {lane_first_cut}px) — else this file's blank-lane laws cannot have caught it"
    );
    assert_eq!(
        lane_shipped, 0,
        "the shipped abutment rule still leaves a {lane_shipped}px blank lane on the \
         across-travel axis — the pitch must never exceed the profile's own excursion"
    );
    // (b) the verified MARGIN band, as a ratio.
    let band_first_cut = widest(PitchRule::PeriodLattice, 1418, 1600, 600);
    let band_shipped = widest(PitchRule::Abut, 1418, 1600, 600);
    assert!(
        band_shipped * 2 <= band_first_cut,
        "the verified 1600x600 right margin still carries a {band_shipped}px blank band \
         against the first cut's {band_first_cut}px — the repair must at least halve it"
    );
}

/// LOCKSTEP: the host mirror above must agree with the REAL GPU field on the
/// row rhythm it measures — the same vertical scanlines, the same relative
/// half-peak criterion, within one row (antialiasing and 8-bit quantization
/// legitimately move a single boundary). Without this, a mirror that drifted
/// from the shader would leave the non-vacuity proofs arguing about code that no
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
                let cov = zigzag_coverage(x as f32, y as f32, bg, PitchRule::Abut);
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

/// THE ROW-PITCH BOUND — the one authored discipline the abutment rule does NOT
/// give for free. Abutment guarantees the field has no blank LANE at any dials;
/// it says nothing about the pitch being fine-grained relative to a PAGE MARGIN,
/// and a pitch wider than a margin cell would let an individual cell fall
/// between two rows even though the lane itself is covered. Every shipping
/// `Zigzag` world therefore authors a profile whose derived pitch clears
/// `theme::ZIGZAG_MAX_ROW_PITCH_PX` — the bound the viewport sweep's own
/// smallest cell (~27x166 px at 1500x500) sets.
#[test]
fn authored_zigzag_row_pitch_stays_fine_grained_against_a_page_margin() {
    let mut seen = 0usize;
    for t in theme::THEMES {
        if !matches!(t.background, theme::Background::Zigzag { .. }) {
            continue;
        }
        seen += 1;
        let pitch = t.background.zigzag_row_pitch_px();
        assert!(
            pitch > 0.0 && pitch <= theme::ZIGZAG_MAX_ROW_PITCH_PX,
            "{}: derived zigzag row pitch {pitch}px is outside (0, {}] — a field whose rows sit \
             further apart than a margin cell can leave that cell blank at a short window",
            t.name,
            theme::ZIGZAG_MAX_ROW_PITCH_PX
        );
        // The pitch is DERIVED, never authored: it must equal the abutment rule
        // applied to this world's own two dials.
        let expect = 2.0 * t.background.amplitude_px() + t.background.zigzag_stroke_px();
        assert!(
            (pitch - expect).abs() < 1e-4,
            "{}: row pitch {pitch} is not the abutment rule's {expect}",
            t.name
        );
    }
    assert!(
        seen >= 2,
        "expected both Zigzag worlds in the roster (saw {seen})"
    );
}

/// STRUCTURAL TRIPWIRE on the shader source itself, so neither half of the fix
/// can be dropped in a refactor on a machine where the GPU laws above skip: the
/// `shader == 7u` branch must fold the ACROSS-travel coordinate through a row
/// pitch, that pitch must be the ABUTMENT rule (not `period`), the stroke
/// fraction must be the constant the host mirror pins, and item 86's unfolded
/// distance must stay gone.
#[test]
fn the_wgsl_zigzag_branch_abuts_its_rows() {
    let src: String = include_str!("../../../shaders/background.wgsl")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    for needle in [
        "letthickness=max(amp*ZIGZAG_STROKE_FRAC,1.2);",
        "letrow_h=2.0*amp+thickness;",
        "letu=(ry-center)/row_h;",
        "abs(fract(u+0.5)-0.5)*row_h",
        "constZIGZAG_STROKE_FRAC:f32=0.10;",
    ] {
        assert!(
            src.contains(needle),
            "shaders/background.wgsl lost the zigzag field fold / abutment rule (missing \
             `{needle}`) — the ground would fall back to one wandering stroke, or to rows that \
             leave a blank lane between them (item 89)"
        );
    }
    assert!(
        !src.contains("letd=abs(ry-center);"),
        "shaders/background.wgsl reintroduced item 86's UNFOLDED chevron distance"
    );
    assert!(
        !src.contains("letrow_h=period;"),
        "shaders/background.wgsl reintroduced item 89's first-cut PERIOD lattice pitch — the \
         rule that left a blank lane of `period - 2*amp - 2*thickness` between rows"
    );
    // The host mirror's own constant must be the shader's.
    assert!(
        (theme::ZIGZAG_STROKE_FRAC - 0.10).abs() < 1e-6
            && (theme::ZIGZAG_MIN_STROKE_PX - 1.2).abs() < 1e-6,
        "theme::ZIGZAG_STROKE_FRAC/ZIGZAG_MIN_STROKE_PX drifted from the WGSL literals above"
    );
}
