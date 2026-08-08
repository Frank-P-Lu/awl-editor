//! THE DIAGONAL COMPOSITION IN REAL PIXELS — five laws, five oracles.
//!
//! The composition already has geometry laws: the mark's side reads off the
//! planner's signed inset, the mark's shape is a chevron, the authored quantities
//! pass one scale boundary, and an upright world reserves nothing. Every one of
//! those grades a NUMBER the frame computed. **None of them grades what was
//! drawn**, and this tree's own history is that a mechanism can fire while the
//! pixels do not move — six render surfaces once shipped invisible while
//! `instance_count == 1` passed on all of them.
//!
//! So each law below asserts over bytes that came back off the GPU, and each has
//! its own oracle rather than five readings of one:
//!
//! 1. **ORIENTATION** — the drawn spine LEANS, monotonically, in the direction the
//!    world authored, and the drawn line is the declared line.
//! 2. **LINE CONTINUITY** — the spine is ONE line: every scanline between the first
//!    drawn row's centre and the last's carries its ink, with no gap at any row
//!    boundary.
//! 3. **THE INSET ATTACHMENT BAND** — the spine stands a FIXED distance from the
//!    card's own inboard edge, and that distance is a property of the SURFACE: it
//!    does not move when the row corpus changes under it.
//! 4. **THE FIXED LABEL–CONTROL GAP** — both lanes are pinned to the spine, so the
//!    room between a row's name and its control is the CARD's number and not the
//!    row's; and a long name never closes that room to nothing.
//! 5. **PLACARD / ROW NON-OVERLAP** — the room's wordmark never lands under a row's
//!    ink. A bare-plate card draws no opaque surface of its own, so a wordmark that
//!    reached the rows would show through them.
//!
//! # How the spine is FOUND, and why it is not read out of the geometry
//!
//! A law that asks "is there ink at the x the code says" cannot fail on a spine
//! drawn at the wrong x by the same code. So the spine is located by SEARCH: it is
//! the first ink band inward from the card's own INBOARD edge — the side away from
//! the row cluster, which is named by the row planner's signed inset (`dx + dw`,
//! exactly one of which is ever nonzero) and by nothing else. On both worlds that
//! strip is otherwise empty; the nearest other ink is the row's name, a full
//! connector further out.
//!
//! # Every number is a length or a comparison
//!
//! No law here asserts a byte value. Positions are logical/device lengths, which a
//! rasterizer does not get to disagree about, and every ink test is a PERCEPTUAL
//! distance from the ground the same frame drew.

use super::super::*;
use super::pixeldiff;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// The perceptual distance from the card's own ground at which a pixel counts as
/// INK. Far under the roster's real signal — the spine is drawn in `muted`, which
/// measures ΔE 41 (Mangrove) and 51 (Magpie) from each world's page — and far over
/// the antialiasing of a card edge.
const INK_DE: f64 = 12.0;

/// Two measured grounds are the same surface below this — the classic JND.
const SAME_GROUND: f64 = 2.3;

/// EVERY WORLD THAT AUTHORS A DIAGONAL SPINE, and the direction it authored. Read
/// off the roster, so a third world enrols by shipping rather than by being listed.
fn diagonal_worlds() -> Vec<(&'static str, theme::DiagonalDirection)> {
    let mut out = Vec::new();
    for world in theme::THEMES {
        match world.render_caps.list_style {
            theme::ListStyle::Diagonal(spine) => out.push((world.name, spine.direction)),
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {}
        }
    }
    assert!(
        out.len() >= 2,
        "the roster sweep found {} diagonal worlds — it is not reading the roster it \
         thinks it is",
        out.len()
    );
    out
}

/// A card with a deliberately WIDE spread of name widths and chord widths — the
/// axis a lane seated off a row's own content moves on. `scroll` puts the window
/// part-way down, which is the case that once slid the whole composition sideways.
fn spread_view(kind: OverlayKind, n: usize, scroll: usize, selected: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\nthird line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = (0..n)
        .map(|i| match i % 4 {
            0 => format!("go {i}"),
            1 => format!("a considerably longer candidate label number {i}"),
            2 => format!("middling label {i}"),
            _ => format!("an even longer name that will certainly need eliding, number {i}"),
        })
        .collect();
    // Chord widths from two characters to seven, so the accessory lane's own
    // position cannot be a reading of any one row's chord. Not wider: a chord
    // column the card cannot seat is yielded WHOLE by `rowlayout`, and a card with
    // no control lane at all is a different law's subject.
    v.overlay_bindings = (0..n)
        .map(|i| match i % 3 {
            0 => "F1".to_string(),
            1 => "C-s".to_string(),
            _ => "C-x C-s".to_string(),
        })
        .collect();
    v.overlay_selected = selected.min(n.saturating_sub(1));
    v.overlay_scroll = scroll;
    v.overlay_hint = "type to filter".into();
    v
}

/// The card's own ground: the modal exact colour over the row band, which on a
/// bare-plate world is the page and on a plated one is the plate — whatever is
/// actually there, never a recomputation. `skip` (the placard's box, where one is
/// in play) is excluded so a wordmark cannot become the "ground" it is being
/// compared against.
fn ground_of(
    pixels: &[[u8; 4]],
    cw: i64,
    plan: &crate::render::plan::OverlayRowPlan,
    card: [f32; 4],
    skip: Option<[f32; 4]>,
) -> Option<[u8; 4]> {
    use std::collections::HashMap;
    let (Some(first), Some(last)) = (plan.rows().first(), plan.rows().last()) else {
        return None;
    };
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    let (x0, x1) = ((card[0] + 2.0) as i64, (card[0] + card[2] - 2.0) as i64);
    for y in (first.top as i64)..(last.bottom() as i64) {
        for x in x0..x1 {
            if let Some(s) = skip
                && (x as f32) >= s[0]
                && (x as f32) < s[0] + s[2]
                && (y as f32) >= s[1]
                && (y as f32) < s[1] + s[3]
            {
                continue;
            }
            let idx = (y * cw + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .filter(|_| true)
}

/// Whether column `x` carries ink anywhere in the scanline range `y0..y1`.
fn ink_col(pixels: &[[u8; 4]], cw: i64, x: i64, y0: i64, y1: i64, ground: [u8; 4]) -> bool {
    for y in y0..y1 {
        let idx = (y * cw + x) as usize;
        if idx < pixels.len() && pixeldiff::delta_e(pixels[idx], ground) > INK_DE {
            return true;
        }
    }
    false
}

/// THE SPINE, FOUND BY SEARCH: the centre of the first ink band inward from the
/// card's INBOARD edge — the side the planner's signed inset names — measured in
/// the three scanlines around `y_mid`. Three, not the row's whole slot: the spine
/// travels sideways as it descends, so a full-slot band's centre carries a bias of
/// a quarter of the row step and could not be compared against a declared abscissa
/// at all.
///
/// `s` is the inset's sign: `+1` when the row's LEFT edge steps in (the spine
/// stands near the card's left edge), `-1` when its right edge does.
fn find_spine(
    pixels: &[[u8; 4]],
    cw: i64,
    card: [f32; 4],
    s: f32,
    y_mid: f32,
    ground: [u8; 4],
    reach: f32,
) -> Option<f32> {
    let (y0, y1) = ((y_mid - 1.0) as i64, (y_mid + 2.0) as i64);
    let start = if s > 0.0 {
        card[0] + 1.0
    } else {
        card[0] + card[2] - 2.0
    };
    let steps = reach.max(1.0) as i64;
    let mut first: Option<i64> = None;
    let mut last: Option<i64> = None;
    for k in 0..steps {
        let x = (start + k as f32 * s) as i64;
        if x < 0 || x >= cw {
            break;
        }
        if ink_col(pixels, cw, x, y0, y1, ground) {
            if first.is_none() {
                first = Some(x);
            }
            last = Some(x);
        } else if first.is_some() {
            break;
        }
    }
    match (first, last) {
        (Some(a), Some(b)) => Some((a + b) as f32 * 0.5),
        _ => None,
    }
}

/// One drawn row's composition, in real pixels.
struct DrawnRow {
    display: usize,
    top: f32,
    bottom: f32,
    mid: f32,
    /// The spine's drawn ink centre in this row's slot.
    spine: f32,
    /// The declared abscissa the frame's own rail says the spine stands at.
    declared_spine: f32,
}

/// The card's inboard edge — the side the spine stands on, named by the planner's
/// signed inset rather than by a world.
fn inboard_edge(card: [f32; 4], s: f32) -> f32 {
    if s > 0.0 { card[0] } else { card[0] + card[2] }
}

/// Read every drawn row's spine, plus the frame's own signed inset. `None` when
/// this cell drew no diagonal row list at all.
fn read_rows(
    p: &TextPipeline,
    pixels: &[[u8; 4]],
    cw: u32,
    ground: [u8; 4],
) -> Option<(f32, [f32; 4], Vec<DrawnRow>)> {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let probe = p.diagonal_cluster_probe()?;
    let card = p.overlay_card_rect()?;
    let span = probe.span;
    let s = (span.dx + span.dw).signum();
    let reach = card[2] * 0.5;
    let mut out = Vec::new();
    for row in plan.rows() {
        if row.item.is_none() {
            continue;
        }
        let mid = row.top + row.height * 0.5;
        let Some(spine) = find_spine(pixels, cw as i64, card, s, mid, ground, reach) else {
            continue;
        };
        out.push(DrawnRow {
            display: row.display,
            top: row.top,
            bottom: row.bottom(),
            mid,
            spine,
            declared_spine: probe.spine_x(row.display),
        });
    }
    (out.len() >= 2).then_some((s, card, out))
}

/// The canvases every sweep runs. Two comfortably wide, two narrow enough that the
/// composition's own reservations start yielding.
const CANVASES: &[(u32, u32)] = &[(1400, 900), (1200, 800), (900, 760), (700, 820)];

/// Render one cell and hand back everything the laws read, or `None` when the cell
/// drew no diagonal list.
struct Cell {
    s: f32,
    card: [f32; 4],
    rows: Vec<DrawnRow>,
    ground: [u8; 4],
    pixels: Vec<[u8; 4]>,
    cw: u32,
}

fn render_cell(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cw: u32,
    ch: u32,
    v: &ViewState,
) -> Option<Cell> {
    p.set_view(v);
    p.prepare(device, queue, cw, ch).ok()?;
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let card = p.overlay_card_rect()?;
    let pixels = pixeldiff::render_frame(p, device, queue, cw, ch);
    let ground = ground_of(&pixels, cw as i64, &plan, card, None)?;
    let (s, card, rows) = read_rows(p, &pixels, cw, ground)?;
    Some(Cell {
        s,
        card,
        rows,
        ground,
        pixels,
        cw,
    })
}

// ---------------------------------------------------------------------------
// LAW 1 — ORIENTATION
// ---------------------------------------------------------------------------

/// **THE DRAWN SPINE LEANS THE WAY THE WORLD AUTHORED IT.**
///
/// Three claims, and the third is the one that stops the first two going vacuous:
///
/// * the ink found by search sits where the frame's own rail says the spine is
///   (within 2 device px), so the pixels and the geometry are ONE line;
/// * every consecutive pair of drawn rows steps in the authored direction's sign —
///   not merely "the ends differ", which a spine that wandered and came back would
///   satisfy;
/// * **PRESENCE / non-degeneracy:** the measured travel clears both an absolute
///   floor and 60% of the frame's own measured step budget. An UPRIGHT spine — the
///   degenerate composition, and the one a broken travel term produces — has travel
///   zero and passes any monotonicity claim vacuously.
///
/// And the two mirrors must have produced OPPOSITE travel signs, or the law cannot
/// tell a mirrored spine from a hard-coded one.
#[test]
fn the_drawn_spine_leans_monotonically_in_its_authored_direction() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the drawn-spine orientation law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut leaning = 0usize;
    let mut signs: Vec<(&str, f32)> = Vec::new();

    for (world, direction) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        let want = direction.sign();
        let mut world_travel: Option<f32> = None;
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for &(lw, lh) in CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for &(n, scroll) in &[(6usize, 0usize), (12, 0), (24, 0), (24, 9)] {
                        let v = spread_view(OverlayKind::Command, n, scroll, n / 2);
                        let Some(cell) = render_cell(&mut p, &device, &queue, cw, ch, &v) else {
                            continue;
                        };
                        let ctx =
                            format!("{world} bar={bar} dpi={dpi} {cw}x{ch} n={n} scroll={scroll}");

                        // ONE LINE: the searched ink is the declared spine.
                        for r in &cell.rows {
                            assert!(
                                (r.spine - r.declared_spine).abs() <= 2.0 * dpi,
                                "{ctx}: row {}'s spine ink was found at x {:.1}, but the rail the \
                                 frame drew from puts it at {:.1} — the drawn line and the \
                                 declared line are not the same line",
                                r.display,
                                r.spine,
                                r.declared_spine
                            );
                        }

                        let first = cell.rows.first().expect("read_rows returned >= 2 rows");
                        let last = cell.rows.last().expect("read_rows returned >= 2 rows");
                        let travel = last.spine - first.spine;
                        let budget = last.declared_spine - first.declared_spine;

                        // A card cramped enough to yield its whole rake draws a
                        // near-upright spine that no pixel grid can resolve. Those
                        // cells are counted OUT rather than passed quietly.
                        if budget.abs() >= 3.0 {
                            for w in cell.rows.windows(2) {
                                let step = w[1].spine - w[0].spine;
                                assert!(
                                    step * want > 0.0,
                                    "{ctx}: rows {} → {} stepped {step:+.1} px, against the \
                                     authored {direction:?} sign {want:+} — the drawn spine does \
                                     not lean the way this world authored it",
                                    w[0].display,
                                    w[1].display
                                );
                            }
                            assert!(
                                travel * want >= 3.0 && travel.abs() >= budget.abs() * 0.6,
                                "{ctx}: the drawn spine travelled {travel:+.1} px across {} rows \
                                 against a declared budget of {budget:+.1} — a spine that does \
                                 not travel is an UPRIGHT line, and every monotonicity claim \
                                 above is vacuous on it",
                                cell.rows.len()
                            );
                            match world_travel {
                                None => world_travel = Some(travel.signum()),
                                Some(prev) => assert_eq!(
                                    prev,
                                    travel.signum(),
                                    "{ctx}: one world leans ONE way — its drawn travel sign \
                                     flipped between cells"
                                ),
                            }
                            leaning += 1;
                        }
                        graded += 1;
                    }
                }
            }
        }
        signs.push((
            world,
            world_travel.expect("a diagonal world drew at least one leaning card"),
        ));
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(
        graded > 100,
        "the orientation sweep graded only {graded} cells"
    );
    assert!(
        leaning > 80,
        "only {leaning} of {graded} cells drew a resolvable rake — the strict arm is running \
         on too small a corpus to mean anything"
    );
    let distinct: std::collections::BTreeSet<String> =
        signs.iter().map(|(_, s)| format!("{s}")).collect();
    assert!(
        distinct.len() >= 2,
        "every rostered diagonal world's spine leans the SAME way in pixels ({signs:?}) — this \
         law cannot tell a mirrored composition from a hard-coded one"
    );
}

// ---------------------------------------------------------------------------
// LAW 2 — LINE CONTINUITY
// ---------------------------------------------------------------------------

/// **THE SPINE IS ONE LINE.** Every scanline between the first drawn row's centre
/// and the last's carries the spine's ink, within a few pixels of the line through
/// the two measured endpoints.
///
/// This is the claim a geometry probe structurally cannot make: `cluster.spine()`
/// returns two endpoints, and a draw that emitted one segment per ROW would return
/// the same two and satisfy every existing law while leaving a gap at each row
/// boundary. The oracle is therefore a walk down the canvas, and its failure
/// message names the longest gap and where it started, because a gap at a row
/// boundary and a gap in the middle of a row are different defects.
///
/// PRESENCE: the walk must cover a real span (the row band is hundreds of
/// scanlines) and every scanline must hit — a spine drawn nowhere has no gaps
/// either, so the found count is asserted against the walked count.
#[test]
fn the_spine_is_one_continuous_line_down_the_whole_row_band() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the spine-continuity law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut scanlines = 0usize;

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for &(lw, lh) in CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for &(n, scroll) in &[(12usize, 0usize), (24, 0), (24, 9)] {
                        let v = spread_view(OverlayKind::Command, n, scroll, n / 2);
                        let Some(cell) = render_cell(&mut p, &device, &queue, cw, ch, &v) else {
                            continue;
                        };
                        let ctx =
                            format!("{world} bar={bar} dpi={dpi} {cw}x{ch} n={n} scroll={scroll}");
                        let first = cell.rows.first().expect(">= 2 rows");
                        let last = cell.rows.last().expect(">= 2 rows");
                        // The walk covers the segment's INTERIOR. Its two end caps
                        // are a rounded 1.5px stroke terminating exactly on a row
                        // centre, so the outermost scanline carries sub-threshold
                        // coverage — that is the cap's antialiasing, not a break in
                        // the line, and a law that counted it would be grading the
                        // rasterizer's corner rounding.
                        let (y0, y1) = (first.mid.ceil() as i64 + 1, last.mid.floor() as i64 - 1);
                        if y1 - y0 < 40 {
                            continue;
                        }
                        // The line through the two MEASURED endpoints — not the
                        // declared one, so this law still fails on a spine drawn
                        // continuously in the wrong place only via law 1's arm and
                        // never accidentally passes because the two agree.
                        let dx = (last.spine - first.spine) / (y1 - y0) as f32;
                        let tol = (3.0 * dpi).ceil() as i64;
                        let mut worst_gap = 0i64;
                        let mut worst_at = 0i64;
                        let mut run = 0i64;
                        let mut hit = 0usize;
                        for y in y0..=y1 {
                            let x = first.spine + dx * (y - y0) as f32;
                            let lo = (x as i64 - tol).max(0);
                            let hi = (x as i64 + tol + 1).min(cell.cw as i64);
                            let found = (lo..hi).any(|xx| {
                                ink_col(&cell.pixels, cell.cw as i64, xx, y, y + 1, cell.ground)
                            });
                            if found {
                                hit += 1;
                                run = 0;
                            } else {
                                run += 1;
                                if run > worst_gap {
                                    worst_gap = run;
                                    worst_at = y;
                                }
                            }
                        }
                        let walked = (y1 - y0 + 1) as usize;
                        assert_eq!(
                            hit,
                            walked,
                            "{ctx}: the spine is BROKEN — {} of {walked} scanlines between the \
                             first row's centre (y {y0}) and the last's (y {y1}) carry no ink \
                             within ±{tol}px of the line through its own two ends. Longest gap \
                             {worst_gap} scanlines, ending at y {worst_at} (row pitch {:.1}). A \
                             per-row spine emits exactly this and satisfies every geometry law.",
                            walked - hit,
                            last.bottom - last.top
                        );
                        scanlines += walked;
                        graded += 1;
                    }
                }
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(
        graded > 40,
        "the continuity sweep graded only {graded} cells"
    );
    assert!(
        scanlines > 8000,
        "the walk covered only {scanlines} scanlines in total — too short a span to see a \
         row-boundary gap"
    );
}

// ---------------------------------------------------------------------------
// LAW 3 — THE INSET ATTACHMENT BAND
// ---------------------------------------------------------------------------

/// **THE SPINE STANDS ON A FIXED SURFACE-RELATIVE INSET.**
///
/// The composition's promise is that filtering and scrolling never move the spine:
/// it is seated off the CARD, and the row corpus elides into what is left. So the
/// oracle holds the window and the world still and changes only the content —
/// six row counts, a scroll, and a filtered-down list — then measures the drawn
/// inset from the card's own inboard edge in each and requires ONE number.
///
/// The card itself is allowed to move and resize between those frames (it hugs its
/// rows); that is exactly why the measurement is RELATIVE to the card's edge and
/// not an absolute abscissa.
///
/// PRESENCE, the companion every geometry floor needs: the inset must be a REAL
/// band. A spine seated flush on the card's edge, or one that lost its attachment
/// term altogether, would hold "the inset never changes" perfectly at zero.
#[test]
fn the_attachment_inset_is_a_property_of_the_surface_not_of_the_row_corpus() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the attachment-inset law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    // The content variations. Every one of them changes the row corpus; none of
    // them changes the window, the world or the DPI.
    let corpus: &[(usize, usize)] = &[(3, 0), (6, 0), (12, 0), (24, 0), (24, 9), (24, 15)];
    let mut graded = 0usize;
    let mut inset_floor_graded = 0usize;

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for &(lw, lh) in CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    let mut seen: Vec<(usize, usize, f32, f32)> = Vec::new();
                    for &(n, scroll) in corpus {
                        let v = spread_view(OverlayKind::Command, n, scroll, n / 2);
                        let Some(cell) = render_cell(&mut p, &device, &queue, cw, ch, &v) else {
                            continue;
                        };
                        let first = cell.rows.first().expect(">= 2 rows");
                        let edge = inboard_edge(cell.card, cell.s);
                        seen.push((n, scroll, (first.spine - edge) * cell.s, cell.card[2]));
                    }
                    if seen.len() < 3 {
                        continue;
                    }
                    let ctx = format!("{world} bar={bar} dpi={dpi} {cw}x{ch}");
                    let (base_n, base_scroll, base_inset, base_w) = seen[0];
                    for &(n, scroll, inset, card_w) in &seen[1..] {
                        assert!(
                            (inset - base_inset).abs() <= 1.5 * dpi,
                            "{ctx}: the drawn attachment inset MOVED with the content — \
                             {base_inset:.1} px from the card's inboard edge at n={base_n} \
                             scroll={base_scroll} (card {base_w:.0} wide), {inset:.1} px at \
                             n={n} scroll={scroll} (card {card_w:.0} wide). The spine is seated \
                             off the rows in front of it, not off the surface."
                        );
                    }
                    // PRESENCE: a real band, not a spine on the card's edge. The
                    // floor sits under the roster's tightest real value (44 logical
                    // px authored, and this is the DEVICE reading) with room for a
                    // cramped card's own yield.
                    assert!(
                        base_inset >= 12.0 * dpi,
                        "{ctx}: the spine stands only {base_inset:.1} device px inside the \
                         card's own edge — there is no attachment band for it to stand on, \
                         and 'the inset never changes' is satisfied at zero"
                    );
                    inset_floor_graded += 1;
                    graded += seen.len();
                }
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(graded > 100, "the inset sweep graded only {graded} frames");
    assert!(
        inset_floor_graded > 20,
        "only {inset_floor_graded} cells reached the presence floor"
    );
}

// ---------------------------------------------------------------------------
// LAW 4 — THE FIXED LABEL–CONTROL GAP
// ---------------------------------------------------------------------------

/// One unselected row's two lanes, in drawn pixels, measured OUTBOARD of the spine.
struct Lanes {
    /// The name's ink edge nearest the spine.
    label_near: f32,
    /// The name's ink edge furthest from the spine.
    label_far: f32,
    /// The control's ink edge nearest the spine.
    control_near: f32,
    /// The control's ink edge furthest from the spine.
    control_far: f32,
    /// Whether the name's ink reached the strip's own near boundary — in which case
    /// the reading is a clip rather than a measurement, and the caller says so
    /// loudly instead of comparing boundary against boundary on every row.
    near_clipped: bool,
}

/// Read a row's two ink blocks between the spine and the cluster's outer end. The
/// row must NOT be the selected one: the selected row carries the mark in the same
/// strip and steps outward, and both belong to a different law.
///
/// The blocks are separated by taking the WIDEST ink-free band in the strip — the
/// name/control gap is wider than any inter-glyph gap by construction, and this is
/// measured rather than assumed by the `MIN_GAP` arm below.
fn read_lanes(cell: &Cell, r: &DrawnRow, near: f32, far: f32) -> Option<Lanes> {
    let s = cell.s;
    let (a, b) = (near, far);
    let (lo, hi) = (a.min(b), a.max(b));
    let bands = pixeldiff::ink_column_bands(
        &cell.pixels,
        cell.cw as i64,
        lo as i64,
        hi as i64,
        (r.top + 1.0) as i64,
        (r.bottom - 1.0) as i64,
        cell.ground,
        (INK_DE as u8).max(1),
    );
    let ink: Vec<(i64, i64)> = bands
        .iter()
        .filter(|b| b.ink)
        .map(|b| (b.x0, b.x1))
        .collect();
    if ink.len() < 2 {
        return None;
    }
    // The widest INK-FREE band strictly between two ink bands.
    let mut split = 0usize;
    let mut widest = -1i64;
    for i in 0..ink.len() - 1 {
        let gap = ink[i + 1].0 - ink[i].1;
        if gap > widest {
            widest = gap;
            split = i;
        }
    }
    // In canvas order the blocks are (near-spine, far-from-spine) when the
    // composition grows rightward and reversed when it grows leftward.
    let near_block = if s > 0.0 {
        (ink[0].0 as f32, ink[split].1 as f32)
    } else {
        (ink[ink.len() - 1].1 as f32, ink[split + 1].0 as f32)
    };
    let far_block = if s > 0.0 {
        (ink[split + 1].0 as f32, ink[ink.len() - 1].1 as f32)
    } else {
        (ink[split].1 as f32, ink[0].0 as f32)
    };
    let boundary = if s > 0.0 { lo as i64 } else { hi as i64 - 1 };
    Some(Lanes {
        label_near: near_block.0,
        label_far: near_block.1,
        control_near: far_block.0,
        control_far: far_block.1,
        near_clipped: (near_block.0 as i64 - boundary).abs() <= 1,
    })
}

/// **THE ROOM BETWEEN A NAME AND ITS CONTROL IS THE CARD'S NUMBER, NOT THE ROW'S.**
///
/// Both lanes hang off the spine: the name on the spine end at one connector, the
/// control on the cluster's outer end. So on every row of one card the name's
/// spine-side edge and the control's outer edge sit at the SAME two offsets from
/// the drawn spine — and the room between them is therefore one number for the
/// whole card, which is what makes a filter or a scroll unable to shuffle the
/// column sideways.
///
/// The axis is CONTENT, deliberately: the sweep's names run from two characters to
/// seventy and its chords from two to eleven, so a lane seated off either one moves
/// row to row and fails. A lane seated off the card does not.
///
/// Two arms, and the second is the presence companion:
///
/// * **FIXED** — both offsets are constant across the drawn rows.
/// * **A REAL GAP** — the clear distance between the name's outer ink and the
///   control's inner ink clears a floor on EVERY row, including the ones whose
///   names had to elide. "The lanes are at fixed offsets" is perfectly true of a
///   card whose long names run straight into their chords; that is the defect a
///   constancy claim alone cannot see.
#[test]
fn both_row_lanes_hang_off_the_spine_so_the_name_control_gap_is_the_cards_own() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the label/control lane law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut rows_graded = 0usize;
    let mut spread_seen = 0usize;
    let mut tightest = f32::MAX;

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                // Wide canvases only: a card too narrow to grant the accessory
                // column at all draws no control lane, and `rowlayout`'s own yield
                // is a different law's subject.
                for &(lw, lh) in &[(1400u32, 900u32), (1200, 800)] {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for &(n, scroll) in &[(12usize, 0usize), (24, 0), (24, 9)] {
                        let selected = (n / 2).min(n - 1);
                        let v = spread_view(OverlayKind::Command, n, scroll, selected);
                        let Some(cell) = render_cell(&mut p, &device, &queue, cw, ch, &v) else {
                            continue;
                        };
                        if !p.overlay_right_column_shown() {
                            continue;
                        }
                        let geom = p.overlay_geometry(cw);
                        let plan = p.overlay_row_plan(&geom);
                        let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
                        let vis_selected = plan.selected_display();
                        let ctx =
                            format!("{world} bar={bar} dpi={dpi} {cw}x{ch} n={n} scroll={scroll}");

                        let mut measured: Vec<(usize, f32, f32, f32, f32)> = Vec::new();
                        for r in &cell.rows {
                            if Some(r.display) == vis_selected {
                                continue; // carries the mark and the outward step
                            }
                            // The strip stops at the cluster's own outer end, so no
                            // margin chrome can enter the reading.
                            let far = probe.accessory_anchor(r.display) + 2.0 * cell.s;
                            // The strip starts OUTBOARD of the spine's own sideways
                            // travel: the line is diagonal, so inside one row's slot
                            // its ink already reaches half a row step past its centre
                            // abscissa, and a strip that began at a fixed few pixels
                            // read the spine's own tail as the name's first glyph.
                            let clear = probe.spine_step().abs() * 0.5 + 3.0 * dpi;
                            let near = r.spine + clear * cell.s;
                            let Some(l) = read_lanes(&cell, r, near, far) else {
                                continue;
                            };
                            assert!(
                                !l.near_clipped,
                                "{ctx}: row {}'s name ink reaches the reading strip's own near                                  boundary ({near:.1}, {clear:.1} px outboard of the drawn spine)                                  — the measurement is a CLIP, so every offset below would be                                  compared boundary against boundary and the constancy arm would                                  pass on any layout at all",
                                r.display
                            );
                            let name_off = (l.label_near - r.spine) * cell.s;
                            let control_off = (l.control_far - r.spine) * cell.s;
                            let gap = (l.control_near - l.label_far) * cell.s;
                            let name_span = (l.label_far - l.label_near).abs();
                            measured.push((r.display, name_off, control_off, gap, name_span));
                        }
                        if measured.len() < 4 {
                            continue;
                        }

                        // FIXED: both offsets are one number for the card. The
                        // tolerance is a glyph's own side bearing, which differs
                        // between a name starting 'g' and one starting 'a'.
                        let tol = 4.0 * dpi;
                        let (r0, n0, c0, _, _) = measured[0];
                        for &(rd, name_off, control_off, _, _) in &measured[1..] {
                            assert!(
                                (name_off - n0).abs() <= tol,
                                "{ctx}: the NAME lane moved with the row's content — row {r0} \
                                 starts {n0:.1} px outboard of its spine, row {rd} starts \
                                 {name_off:.1} px outboard of its own. The name is seated off \
                                 its own width, not off the composition."
                            );
                            assert!(
                                (control_off - c0).abs() <= tol,
                                "{ctx}: the CONTROL lane moved with the row's content — row {r0} \
                                 ends {c0:.1} px outboard of its spine, row {rd} ends \
                                 {control_off:.1} px outboard of its own. A chord column seated \
                                 off the row rather than the card shuffles sideways on every \
                                 filter keystroke."
                            );
                        }

                        // A REAL GAP on every row, elided ones included.
                        // ⚠️ CALIBRATED, not chosen. A floor loose enough to be
                        // obviously safe is a floor no defect can trip: at 2 device
                        // px this arm survived the name budget swallowing the whole
                        // gap allowance. Measured over this sweep, the shipped
                        // tightest name/control gap is 57 device px at 1x, and
                        // collapsing the layout's own gap budget takes it to 32 — so
                        // the floor sits in that interval, with 17 px of headroom
                        // over the shipped reading and 8 under the defect's.
                        let min_gap = 40.0 * dpi;
                        for &(rd, _, _, gap, _) in &measured {
                            assert!(
                                gap >= min_gap,
                                "{ctx}: row {rd}'s name runs to within {gap:.1} device px of its \
                                 control (floor {min_gap:.1}) — the lanes are at fixed offsets \
                                 and the room between them has still been spent, which is what a \
                                 constancy claim alone cannot see"
                            );
                            tightest = tightest.min(gap);
                        }

                        // NON-VACUITY for the constancy arms: this cell's own names
                        // must really differ in drawn width. A lane law swept over
                        // one name length proves nothing about a lane seated off the
                        // name, which is the defect both arms are named for.
                        let widest = measured.iter().map(|m| m.4).fold(0.0f32, f32::max);
                        let narrowest = measured.iter().map(|m| m.4).fold(f32::MAX, f32::min);
                        if widest >= narrowest * 3.0 {
                            spread_seen += 1;
                        }
                        rows_graded += measured.len();
                        graded += 1;
                    }
                }
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(graded > 8, "the lane sweep graded only {graded} cells");
    assert!(
        rows_graded > 80,
        "the lane sweep graded only {rows_graded} rows"
    );
    // NON-VACUITY for the gap arm: the corpus must contain rows whose names got
    // close to their control. A sweep of short names would clear any floor.
    assert!(
        spread_seen > 4,
        "only {spread_seen} graded cells drew names whose widest was 3x the narrowest — the \
         content axis both constancy arms are named for is not actually present in this sweep \
         (tightest name/control gap seen: {tightest:.1} device px)"
    );
}

// ---------------------------------------------------------------------------
// LAW 5 — PLACARD / ROW NON-OVERLAP
// ---------------------------------------------------------------------------

/// **THE ROOM'S WORDMARK NEVER LANDS UNDER A ROW.**
///
/// The placard is anchored to the WINDOW's corner while the card is anchored to
/// its own, and which corner the wordmark takes is derived from the card's anchor
/// (`derived_placard_corner`). A diagonal world draws BARE plates — no opaque
/// surface of its own under the rows — so a wordmark that reached them would show
/// through, competing with the row text at full size.
///
/// The axis is therefore the card's ANCHOR, swept through every variant including
/// both halves of `Inset`, crossed with window sizes small enough that a
/// fit-to-canvas wordmark is as wide as the room.
///
/// Three arms:
///
/// * **DISJOINT** — the placard's own box and every row's cluster-plus-mark box
///   share no area. A geometric claim in device px, which no rasterizer disputes.
/// * **THE WORDMARK IS THERE** — real ink inside the placard's box. Disjointness
///   is otherwise satisfied perfectly by a wordmark that stopped drawing.
/// * **AND THE ROWS SIT ON THE CARD'S OWN GROUND** — the modal surface inside each
///   row's cluster box matches the card's own ground measured OUTSIDE the placard's
///   box. This is the arm that fails if the wordmark bleeds under the rows without
///   the two declared boxes overlapping, which is the interesting version of the
///   defect.
#[test]
fn the_room_wordmark_never_lands_under_a_diagonal_row() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the placard/row non-overlap law: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let anchors = [
        theme::CardAnchor::TopLeft,
        theme::CardAnchor::TopCenter,
        theme::CardAnchor::TopRight,
        theme::CardAnchor::Inset { x_frac: 0.12 },
        theme::CardAnchor::Inset { x_frac: 0.72 },
    ];
    let mut graded = 0usize;
    let mut placard_seen = 0usize;
    let mut anchors_seen = std::collections::BTreeSet::new();

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for bar in [false, true] {
            crate::menubar::set_menu_bar_on(bar);
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for &(lw, lh) in CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for anchor in anchors {
                        crate::render::overrides::set_card_anchor_test_override(Some(anchor));
                        let v = spread_view(OverlayKind::Command, 18, 0, 9);
                        let Some(cell) = render_cell(&mut p, &device, &queue, cw, ch, &v) else {
                            continue;
                        };
                        let geom = p.overlay_geometry(cw);
                        let plan = p.overlay_row_plan(&geom);
                        let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
                        let Some((px, py, pw, ph)) = p.overlay_shape_placard(&geom) else {
                            continue; // a narrow card draws no wordmark at all
                        };
                        let placard = [px, py, pw, ph];
                        let ctx = format!("{world} bar={bar} dpi={dpi} {cw}x{ch} {anchor:?}");
                        anchors_seen.insert(format!("{anchor:?}"));

                        // THE WORDMARK IS THERE.
                        let mut wordmark_ink = 0usize;
                        for y in (py.max(0.0) as i64)..((py + ph) as i64).min(ch as i64) {
                            for x in (px.max(0.0) as i64)..((px + pw) as i64).min(cw as i64) {
                                let idx = (y * cw as i64 + x) as usize;
                                if idx < cell.pixels.len()
                                    && pixeldiff::delta_e(cell.pixels[idx], cell.ground) > 1.0
                                {
                                    wordmark_ink += 1;
                                }
                            }
                        }
                        assert!(
                            wordmark_ink > 200,
                            "{ctx}: only {wordmark_ink} pixels inside the wordmark's own box \
                             {placard:?} differ from the ground at all — a placard that stopped \
                             drawing satisfies every non-overlap claim below"
                        );
                        placard_seen += 1;

                        // The card's own ground, measured OUTSIDE the wordmark's box.
                        let Some(card_ground) =
                            ground_of(&cell.pixels, cw as i64, &plan, cell.card, Some(placard))
                        else {
                            continue;
                        };

                        for r in &cell.rows {
                            let (cl, cr) = probe.cluster_span(r.display);
                            let (mv, ma) = probe.mark_span(r.display);
                            let lo = cl.min(cr).min(mv).min(ma);
                            let hi = cl.max(cr).max(mv).max(ma);
                            let row_box = [lo, r.top, hi - lo, r.bottom - r.top];

                            // DISJOINT.
                            let overlap_x =
                                (row_box[0] + row_box[2]).min(px + pw) - row_box[0].max(px);
                            let overlap_y =
                                (row_box[1] + row_box[3]).min(py + ph) - row_box[1].max(py);
                            assert!(
                                overlap_x <= 0.0 || overlap_y <= 0.0,
                                "{ctx}: the room's wordmark {placard:?} overlaps row {}'s own ink \
                                 box {row_box:?} by {overlap_x:.1}x{overlap_y:.1} device px — the \
                                 card draws bare plates, so the mark shows THROUGH the row text",
                                r.display
                            );

                            // AND THE ROW SITS ON THE CARD'S OWN GROUND.
                            let region = pixeldiff::Region::new(
                                row_box[0],
                                row_box[1] + 1.0,
                                row_box[2],
                                row_box[3] - 2.0,
                            );
                            let local = row_mode(&cell.pixels, cw as i64, region);
                            let de = pixeldiff::delta_e(local, card_ground);
                            assert!(
                                de < SAME_GROUND,
                                "{ctx}: row {}'s own ground reads {local:?}, ΔE {de:.2} from the \
                                 card's ground away from the wordmark ({card_ground:?}) — \
                                 something is drawn under this row that is not under the rest of \
                                 the card, and the two declared boxes do not overlap",
                                r.display
                            );
                        }
                        graded += 1;
                    }
                }
            }
        }
    }

    crate::render::overrides::set_card_anchor_test_override(None);
    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    assert_eq!(
        anchors_seen.len(),
        anchors.len(),
        "every card anchor must have staged a wordmark somewhere in the sweep — the wordmark's \
         corner is DERIVED from the anchor, so a missed anchor is a missed corner. Saw \
         {anchors_seen:?}"
    );
    assert!(graded > 40, "the wordmark sweep graded only {graded} cells");
    assert_eq!(
        graded, placard_seen,
        "every graded cell must have cleared the wordmark presence arm"
    );
}

/// The modal exact colour over a region.
fn row_mode(pixels: &[[u8; 4]], cw: i64, r: pixeldiff::Region) -> [u8; 4] {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * cw + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}
