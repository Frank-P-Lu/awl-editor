//! THE SELECTED-ROW MARK'S SIDE, AND ITS PER-WORLD FORM.
//!
//! These laws replace the ones that graded the mark's TURN. The turn is gone —
//! the mark is upright, and what carries direction is where it stands: on the
//! row's OUTER edge, away from the spine, pointing back into the row. So the
//! claims that had a subject in a rotating mark ("the vertex is pinned to the
//! spine at every turn", "Down and Up settle at mirrored tilts", "the turn
//! progresses on injected dt", "Reduce Motion settles it instantly") do not
//! survive as weaker versions of themselves; they are replaced by claims about
//! the side, the mirror, drawn↔hit-test agreement, and the two worlds' own
//! authored marks.
//!
//! # The oracle, and why it is not a per-world branch
//!
//! The composition mirrors on ONE signed quantity, and the row planner publishes
//! it: a `RowSpan` steps the row's LEFT edge in (`dx > 0`) on a descending world
//! and its RIGHT edge in (`dw < 0`) on an ascending one, and exactly one of the
//! two is ever nonzero. So `dx + dw` IS the signed inset, and the direction the
//! mark must lie in is its sign — read off the plan the frame drew from, never
//! re-derived here and never keyed to a world's name. A law that named the two
//! worlds would pass on a mark hard-coded to each of them, which is the defect.
//!
//! # The axes
//!
//! Every `OverlayKind` (no wildcard), both diagonal worlds taken from the
//! ROSTER, 1× and 2× DPI, four canvases including two narrow enough to stage a
//! workspace, and four list shapes (empty, short, full, scrolled). Every
//! `SettingId × SettingKind` gets its own sweep, because a range row draws a
//! rail inside the row and is the one row content that could collide with the
//! mark's lane.
//!
//! # What is NOT here
//!
//! MOTION. A `Diagonal` world draws no selection band at all
//! (`overlay_selection_rects` returns nothing for it) and ships
//! `MotionJuice::CALM`, so there is no existing ease for the mark to ride and
//! `overlay_band_drawn` is the identity on it. Gliding the mark between rows
//! would mean a new positional animator — machinery — so it is not built, and no
//! law here claims feel.

use super::super::*;
use super::pixeldiff;
use super::pixeldiff::render_frame;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};

/// The canvases every sweep runs. Two comfortably wide, two narrow enough to
/// stage a workspace — the mark's lane is taken out of the card's own side
/// territory, so a cramped card is where a reservation error shows first.
const CANVASES: &[(u32, u32)] = &[(1400, 900), (1100, 760), (860, 900), (620, 820)];

/// EVERY WORLD THAT AUTHORS A DIAGONAL SPINE, and its authored mark — read off
/// the roster, so a third one enrols by shipping.
fn diagonal_worlds() -> Vec<(&'static str, theme::DiagonalSpine)> {
    let mut out = Vec::new();
    for world in theme::THEMES {
        match world.render_caps.list_style {
            theme::ListStyle::Diagonal(spine) => out.push((world.name, spine)),
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Ruled(_) => {}
        }
    }
    assert!(
        out.len() >= 2,
        "the roster sweep found {} diagonal worlds — it is not reading the roster \
         it thinks it is",
        out.len()
    );
    out
}

/// A card of `kind` with `n` candidate rows and a chord column, shaped the way
/// `sync_view` shapes one. `scroll` puts the window part-way down a long list,
/// which is the case that once moved the whole composition sideways.
fn marked_view(kind: OverlayKind, n: usize, scroll: usize) -> ViewState {
    marked_view_at(kind, n, scroll, 1.0)
}

/// The same card at an explicit ZOOM — the other half of the one scale boundary
/// the mark's lane passes through, and a `ViewState` field rather than a
/// pipeline setter because that is the door production drives it through.
fn marked_view_at(kind: OverlayKind, n: usize, scroll: usize, zoom: f32) -> ViewState {
    let mut v = view("hello world\nsecond line\nthird line\n", 0, 0);
    v.zoom = zoom;
    v.overlay_active = true;
    v.overlay_title = kind.title().to_string();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = (scroll + n / 2).min(n.saturating_sub(1));
    v.overlay_scroll = scroll;
    v.overlay_hint = "type to filter".into();
    if crate::facets::scheme(kind).is_some() {
        v.overlay_lens = vec![
            ("All".into(), true),
            ("File".into(), false),
            ("Edit".into(), false),
        ];
    }
    if let Some(shape) = kind.workspace_shape() {
        v.overlay_workspace = true;
        v.overlay_rows_primary = shape.rows_are_primary();
        if v.overlay_lens.is_empty() {
            v.overlay_lens = vec![("All".into(), true), ("Editor".into(), false)];
        }
        v.overlay_detail_focus = true;
    }
    if kind == OverlayKind::Spell {
        v.overlay_spell = Some((0, 0, 5));
        v.overlay_items = (0..n.min(5)).map(|i| format!("suggest{i}")).collect();
        v.overlay_bindings = Vec::new();
        v.overlay_selected = v.overlay_items.len() / 2;
        v.overlay_scroll = 0;
        v.overlay_hint = String::new();
        v.overlay_lens = Vec::new();
        v.overlay_workspace = false;
    }
    v
}

/// ONE CELL'S READING of where the mark stands, off the SAME rail the draw used
/// and the SAME plan the pointer inverse reads. `None` when this cell drew no
/// diagonal row list at all (an empty card, a staged workspace region) — which
/// the callers count, so "nothing was graded" can never pass for a sweep.
struct MarkReading {
    /// The signed inset the ROW PLANNER published for this cluster: `+` when the
    /// row's left edge steps in, `-` when its right edge does. The one dial.
    inset: f32,
    spine_x: f32,
    label_anchor: f32,
    resting_label_anchor: f32,
    accessory_anchor: f32,
    /// The row's own measured NAME ink — what `mark_span` seats the vertex
    /// just past.
    ink_w: f32,
    /// The row's own measured ACCESSORY ink (a chord, a value, a Range
    /// readout), `0.0` when the row draws none — what `mark_span` holds the
    /// mark clear of, never the shared column's reserved width.
    accessory_ink_w: f32,
    vertex: f32,
    arm: f32,
    /// The ITEM the selected row carries — what `overlay_row_at` answers with.
    row_item: usize,
    /// The selected row's own clickable span, as the POINTER INVERSE bounds it
    /// (`OverlayRowPlan::row_at`: the card span stepped by the row's own
    /// `dx`/`dw`), and its vertical slot.
    row_left: f32,
    row_right: f32,
    row_top: f32,
    row_bottom: f32,
    /// The card the whole composition must stay inside.
    card: [f32; 4],
}

fn read_mark(p: &TextPipeline, cw: u32) -> Option<MarkReading> {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let sel = plan.selected_display()?;
    let row = plan.rows().iter().find(|r| r.display == sel)?;
    row.item?;
    let probe = p.diagonal_cluster_probe()?;
    let span = probe.span;
    let ink_w = p
        .overlay_row_primary_px(&geom)
        .get(&sel)
        .copied()
        .unwrap_or(0.0);
    let accessory_ink_w = p
        .overlay_row_secondary_px(&plan)
        .get(&sel)
        .copied()
        .unwrap_or(0.0);
    let (vertex, arm) = probe.mark_span(sel, ink_w, accessory_ink_w);
    let (card_x0, card_x1) = plan.card_x_span();
    Some(MarkReading {
        inset: span.dx + span.dw,
        spine_x: probe.spine_x(sel),
        label_anchor: probe.label_anchor(sel),
        // The label end at the row's RESTING position — the selected row's own
        // outward shift removed — so a box drawn against it excludes the label's
        // first glyph in BOTH selection states. Without that the shift's own 4px
        // of moved ink reads as the mark.
        resting_label_anchor: probe.label_anchor(sel) - probe.selected_offset().0,
        accessory_anchor: probe.accessory_anchor(sel),
        ink_w,
        accessory_ink_w,
        vertex,
        arm,
        row_item: row.item?,
        row_left: card_x0 + row.dx,
        row_right: card_x1 + row.dw,
        row_top: row.top,
        row_bottom: row.bottom(),
        card: p.overlay_card_rect().unwrap_or([0.0, 0.0, 0.0, 0.0]),
    })
}

/// THE WHOLE SIDE CLAIM on one cell, asserted against the planner's own signed
/// inset. Every comparison below is multiplied by `inset`, so the same lines
/// grade both mirrors and there is nowhere for a per-world constant to hide:
/// swap the sign in `DiagonalClusterRail::mark_span` and every cell fails.
fn assert_mark_is_outboard(r: &MarkReading, ctx: &str) {
    let s = r.inset.signum();
    assert!(
        r.inset.abs() > 0.5,
        "{ctx}: the planner published no signed inset ({}) — this law's whole \
         oracle is that sign, so a zero makes every claim below vacuous",
        r.inset
    );
    // THE HEADLINE: the mark stands past the row's own NAME, on the side away
    // from the spine — never at the cluster's whole reserved width, which is
    // the stranded placement this law replaced.
    assert!(
        (r.vertex - r.label_anchor) * s > 0.0,
        "{ctx}: the mark's vertex ({}) is not outboard of the row's own label \
         end ({}) — it is on the SPINE side of the row, or sitting on the label \
         itself (inset sign {s}, spine at {})",
        r.vertex,
        r.label_anchor,
        r.spine_x
    );
    // NON-VACUITY: the vertex actually moved past the NAME's own measured ink,
    // not just past the label's bare anchor — the reading this law would get
    // if `mark_span` silently dropped `ink_w` on the floor. Skipped when the
    // accessory clamp is live: it is free to pull the vertex back inside the
    // name's own ink on a row too cramped to afford the full reach, which is
    // the clamp working, not the ink going unread.
    if r.accessory_ink_w <= 0.0 {
        // `(vertex - label_anchor) * s` is the SIGNED outboard distance — always
        // non-negative once the headline claim above holds. `ink_w` is a plain
        // unsigned width (never itself multiplied by `s`), so it is compared
        // against that distance directly.
        let past_label = (r.vertex - r.label_anchor) * s;
        assert!(
            past_label >= r.ink_w - 0.51,
            "{ctx}: the vertex ({}) sits only {past_label} past the label end \
             ({}) — the row's own measured name ink ({}) is not reaching the \
             placement, which is indistinguishable from every row sharing one \
             fixed reach",
            r.vertex,
            r.label_anchor,
            r.ink_w
        );
        // …and it stands only a SEATING GAP past that ink, not the cluster's whole
        // reserved width — the far placement this law replaced. 60 device px covers
        // the authored gap at every DPI and zoom this sweep runs while sitting far
        // under any real accessory-column reach, so this floor is what actually
        // catches a `mark_span` reverted to its old `accessory_anchor`-based reach.
        let past_ink = past_label - r.ink_w;
        assert!(
            past_ink <= 60.0,
            "{ctx}: the vertex sits {past_ink:.1} px past the row's own name ink \
             ({}) — far more than a seating gap, which reads as the mark \
             standing at the cluster's whole reserved width again rather than \
             just past the name",
            r.ink_w
        );
    }
    // THE MARK POINTS BACK INTO THE ROW: the vertex is its inner end and the
    // arms open outward, into the card's margin.
    assert!(
        (r.arm - r.vertex) * s > 0.0,
        "{ctx}: the arms ({}) must open AWAY from the row from the vertex ({}) — a \
         mark whose vertex is outboard points out of the card",
        r.arm,
        r.vertex
    );
    // NO COLLISION with the row's OWN accessory ink — a chord, a value or a
    // Range readout — when it draws one. A row with none has nothing on this
    // side to collide with, so the check is gated on real ink rather than the
    // shared column's reserved width, which the mark is free to cross when
    // this row leaves it empty.
    if r.accessory_ink_w > 0.0 {
        let far = r.accessory_anchor - r.accessory_ink_w * s;
        let (alo, ahi) = (r.accessory_anchor.min(far), r.accessory_anchor.max(far));
        let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
        assert!(
            hi <= alo + 0.01 || lo >= ahi - 0.01,
            "{ctx}: the mark [{lo}, {hi}] overlaps its own row's accessory ink \
             [{alo}, {ahi}] ({} px wide) — the clamp meant to hold it clear did \
             not",
            r.accessory_ink_w
        );
    }
    // NO CLIPPING: both abscissae stay inside the card.
    let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
    assert!(
        lo >= r.card[0] - 0.51 && hi <= r.card[0] + r.card[2] + 0.51,
        "{ctx}: the mark [{lo}, {hi}] leaves the card [{}, {}]",
        r.card[0],
        r.card[0] + r.card[2]
    );
}

// ---------------------------------------------------------------------------
// LAW 1 — the side, over every `OverlayKind` × world × DPI × canvas × list shape
// ---------------------------------------------------------------------------

/// THE MARK IS ON THE ROW'S OUTER EDGE, AND THE SIDE COMES FROM THE PLANNER.
///
/// Non-vacuity has three arms, because a sweep of negative-ish geometric claims
/// is the easiest kind to satisfy by rendering nothing: the cell count is
/// asserted, every `OverlayKind` must have been reached, and the two mirrors must
/// have produced OPPOSITE inset signs — so a run that only ever saw one world
/// (or one in which the mirror had collapsed) fails rather than passing quietly.
#[test]
fn the_selected_mark_stands_on_the_row_outer_edge_in_every_kind_and_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the_selected_mark_stands_on_the_row_outer_edge: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut kinds_seen = std::collections::BTreeSet::new();
    let mut signs_seen: Vec<(&str, f32)> = Vec::new();

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        let mut world_sign: Option<f32> = None;
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for kind in OverlayKind::ALL {
                    // Empty, short, full and scrolled — the four list shapes a
                    // fixed surface-relative spine has to survive.
                    for &(n, scroll) in &[(0usize, 0usize), (3, 0), (24, 0), (24, 9)] {
                        p.set_view(&marked_view(kind, n, scroll));
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let ctx =
                            format!("{world} {kind:?} n={n} scroll={scroll} dpi={dpi} {cw}x{ch}");
                        let Some(r) = read_mark(&p, cw) else {
                            continue;
                        };
                        assert_mark_is_outboard(&r, &ctx);
                        let s = r.inset.signum();
                        match world_sign {
                            None => world_sign = Some(s),
                            Some(prev) => assert_eq!(
                                prev, s,
                                "{ctx}: one world must mirror ONE way — its inset sign \
                                 flipped between cells"
                            ),
                        }
                        kinds_seen.insert(format!("{kind:?}"));
                        graded += 1;
                    }
                }
            }
        }
        signs_seen.push((
            world,
            world_sign.expect("a diagonal world graded at least one cell"),
        ));
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);

    let want_kinds: std::collections::BTreeSet<String> = OverlayKind::ALL
        .into_iter()
        .map(|k| format!("{k:?}"))
        .collect();
    assert_eq!(
        kinds_seen,
        want_kinds,
        "every OverlayKind must have drawn a marked diagonal row somewhere in the \
         sweep — missing {:?}",
        want_kinds.difference(&kinds_seen).collect::<Vec<_>>()
    );
    assert!(
        graded > 400,
        "the sweep must grade a real corpus of cells, got {graded}"
    );
    // THE MIRROR ITSELF: at least two worlds, and they do not agree.
    let distinct: std::collections::BTreeSet<String> =
        signs_seen.iter().map(|(_, s)| format!("{s}")).collect();
    assert!(
        distinct.len() >= 2,
        "the rostered diagonal worlds all mirror the SAME way ({signs_seen:?}) — this \
         law cannot tell a mirrored mark from a hard-coded one"
    );
}

// ---------------------------------------------------------------------------
// LAW 2 — drawn ↔ hit-test agreement
// ---------------------------------------------------------------------------

/// A CLICK ON THE MARK SELECTS THE ROW THE MARK IS ON.
///
/// The mark is the outermost ink a diagonal row owns, so it is the first thing a
/// reader aims at and the easiest thing to draw outside the row's own clickable
/// span. Graded through `overlay_row_at` — the production pointer inverse, not a
/// re-derivation — at the mark's vertex, at its arm end, and at its centre, over
/// both worlds, both DPIs, every canvas, and a zoom, because the mark's lane is
/// a logical length and zoom is the other half of the one scale boundary.
#[test]
fn the_mark_lies_inside_its_own_row_clickable_span_at_every_scale() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the_mark_lies_inside_its_own_row_clickable_span: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    let mut graded = 0usize;

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for zoom in [1.0f32, 1.4] {
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for &(lw, lh) in CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for kind in [
                        OverlayKind::Command,
                        OverlayKind::Goto,
                        OverlayKind::Settings,
                    ] {
                        p.set_view(&marked_view_at(kind, 18, 4, zoom));
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let Some(r) = read_mark(&p, cw) else {
                            continue;
                        };
                        let ctx = format!("{world} {kind:?} zoom={zoom} dpi={dpi} {cw}x{ch}");
                        // CONTAINMENT: the drawn mark is inside the row surface
                        // the pointer inverse accepts for that row.
                        let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
                        assert!(
                            lo >= r.row_left - 0.51 && hi <= r.row_right + 0.51,
                            "{ctx}: the mark [{lo}, {hi}] leaves its own row's clickable \
                             span [{}, {}] — the outermost ink a row owns must still be \
                             that row's to click",
                            r.row_left,
                            r.row_right
                        );
                        // AGREEMENT: the inverse answers with the marked row at
                        // three points along the mark.
                        let mid_y = (r.row_top + r.row_bottom) * 0.5;
                        for (what, x) in [
                            ("vertex", r.vertex),
                            ("arm", r.arm),
                            ("centre", (r.vertex + r.arm) * 0.5),
                        ] {
                            assert_eq!(
                                p.overlay_row_at(x, mid_y),
                                Some(r.row_item),
                                "{ctx}: pointing at the mark's {what} (x={x}, y={mid_y}) must \
                                 hit the row it marks"
                            );
                        }
                        graded += 1;
                    }
                }
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 40,
        "the agreement sweep graded only {graded} cells"
    );
}

// ---------------------------------------------------------------------------
// LAW 3 — real pixels: the ink moved sides
// ---------------------------------------------------------------------------

/// Pixels differing between two frames inside `[x0, x1) × [y0, y1)`.
fn differ_in(a: &[[u8; 4]], b: &[[u8; 4]], w: i64, x0: f32, x1: f32, y0: f32, y1: f32) -> usize {
    let (xa, xb) = (x0.floor().max(0.0) as i64, x1.ceil().max(0.0) as i64);
    let (ya, yb) = (y0.floor().max(0.0) as i64, y1.ceil().max(0.0) as i64);
    let mut n = 0;
    for y in ya..yb {
        for x in xa..xb {
            let i = (y * w + x) as usize;
            if i < a.len() && i < b.len() && a[i] != b[i] {
                n += 1;
            }
        }
    }
    n
}

/// THE LANE'S PERCEPTUAL READING of the same box [`differ_in`] counts bytes in,
/// as `(travel, span, covered)`: how far the darkest cell in the lane MOVED from
/// its own unselected ground, how far that same cell would have had to move to
/// wear the mark's ink outright, and how many cells ended up nearer the ink than
/// their own ground.
///
/// Every quantity is a distance between two readings of the SAME cell, and the
/// claims built on them are ratios and counts — never a byte compared to a theme
/// constant, so none of it is a claim about the rasterizer. `ink` is read from
/// the same owner the draw sets its colour from.
fn lane_presence(
    on: &[[u8; 4]],
    off: &[[u8; 4]],
    ink: [u8; 4],
    w: i64,
    x: (f32, f32),
    y: (f32, f32),
) -> (f64, f64, usize) {
    let (xa, xb) = (x.0.floor().max(0.0) as i64, x.1.ceil().max(0.0) as i64);
    let (ya, yb) = (y.0.floor().max(0.0) as i64, y.1.ceil().max(0.0) as i64);
    let (mut travel, mut span, mut covered) = (0.0f64, 0.0f64, 0usize);
    for row in ya..yb {
        for col in xa..xb {
            let i = (row * w + col) as usize;
            if i >= on.len() || i >= off.len() {
                continue;
            }
            let moved = pixeldiff::delta_e(on[i], off[i]);
            if moved > travel {
                travel = moved;
                span = pixeldiff::delta_e(off[i], ink);
            }
            if pixeldiff::delta_e(on[i], ink) < moved {
                covered += 1;
            }
        }
    }
    (travel, span, covered)
}

/// HOW FAR THE DARKEST CELL MUST TRAVEL from its own ground toward the mark's
/// ink, as a FRACTION of the distance between them — so the claim is the same
/// claim on a dark world and a light one, and on a backend that antialiases
/// differently. Measured on this host: the shipped hairline travels 0.67 of that
/// span at 1× and 1.00 at 2×, the crisp mark 1.00 at both, and a mark thinned to
/// an eighth of the hairline's stroke — a wash, still drawn, still moving bytes —
/// travels 0.30 and 0.38. The half-way line separates them with margin at both
/// ends, and it is the coverage below which a stroke has stopped being ink.
const MARK_MIN_TRAVEL_FRACTION: f64 = 0.5;

/// AND HOW MANY CELLS MUST GET THERE, so one lucky sample cannot stand in for a
/// shape. Measured: 10 cells for the thinnest authorship at the coarsest scale,
/// 395 for the heaviest at the finest — and exactly ZERO for the wash, which is
/// what makes any positive floor a real separator and lets this one sit well
/// under the tightest real reading.
const MARK_MIN_COVERED_CELLS: usize = 4;

/// REAL PIXELS — SELECTING A ROW PAINTS INK IN ITS OUTER LANE AND LEAVES THE
/// SPINE-SIDE CONNECTOR GAP UNTOUCHED.
///
/// The sidecar and the geometry probes are state oracles; this is the appearance
/// oracle, and it is the law that fails on the shipped defect rather than on a
/// renamed field. Two frames of the same card differing only in WHICH row is
/// selected are compared inside two boxes on the graded row: the mark's own lane
/// beyond the cluster's outer end, and the connector gap at the spine end where
/// the mark used to be drawn. The first must gain ink; the second must not.
///
/// # The lane's PERCEPTUAL floor, and why a count of moved bytes needed one
///
/// A mark authored per world is authored to be *lighter* in an editorial
/// register, and "lighter" runs all the way down to nothing. Neither reading
/// that existed before could see that: this law counted BYTES that moved, and a
/// whole lane shifted by one level passes it, while
/// [`each_diagonal_world_paints_its_own_authored_mark`] grades the ORDER of two
/// worlds' ink, which two marks scaled together toward invisibility still
/// satisfy. So the lane also answers to [`MARK_PEAK_DE_FLOOR`] and
/// [`MARK_PAST_JND_FLOOR`] against its own ground, which is what makes a thinner
/// mark a *findable* one rather than an absent one.
///
/// Swept over both scales, because a hairline authored in logical pixels is
/// thinnest in device pixels at 1x — the scale every capture defaults to, and so
/// the scale at which a vanishing stroke would look correct.
#[test]
fn selecting_a_row_paints_the_outer_lane_and_leaves_the_spine_gap_clear() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping selecting_a_row_paints_the_outer_lane: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    let mut graded = 0usize;

    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);

            // The GRADED row is 3; the two frames select 3 and then 5, so row 3
            // is marked in one and plain in the other with nothing else moved.
            let mut on = marked_view(OverlayKind::Command, 12, 0);
            on.overlay_selected = 3;
            let mut off = marked_view(OverlayKind::Command, 12, 0);
            off.overlay_selected = 5;

            p.set_view(&on);
            p.prepare(&device, &queue, cw, ch).unwrap();
            let r = read_mark(&p, cw).expect("a marked diagonal row");
            let frame_on = render_frame(&mut p, &device, &queue, cw, ch);

            p.set_view(&off);
            p.prepare(&device, &queue, cw, ch).unwrap();
            let frame_off = render_frame(&mut p, &device, &queue, cw, ch);

            let ctx = format!("{world} dpi={dpi}");
            let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
            let lane = differ_in(
                &frame_on,
                &frame_off,
                cw as i64,
                lo - 1.0,
                hi + 1.0,
                r.row_top,
                r.row_bottom,
            );
            assert!(
                lane > 8,
                "{ctx}: selecting the row painted only {lane} differing px in its OUTER \
                 lane [{lo}, {hi}] — the mark is not being drawn where the geometry says \
                 it is, and no state probe can see that"
            );

            // THE SAME BOX, READ PERCEPTUALLY. `lane` counts cells that
            // moved; these two say the movement is ink.
            let (travel, span, covered) = lane_presence(
                &frame_on,
                &frame_off,
                theme::base_content().rgba_bytes(),
                cw as i64,
                (lo - 1.0, hi + 1.0),
                (r.row_top, r.row_bottom),
            );
            let reached = if span > 0.0 { travel / span } else { 0.0 };
            assert!(
                reached >= MARK_MIN_TRAVEL_FRACTION,
                "{ctx}: the mark's darkest cell travelled ΔE {travel:.2} of the {span:.2} \
                 between its own unselected ground and the ink the mark is painted in — \
                 {reached:.2} of the way, under the {MARK_MIN_TRAVEL_FRACTION} floor. This \
                 world's mark has been thinned past being drawn, and neither {lane} moved \
                 bytes nor an ORDER against the other world's ink can tell that from a \
                 stroke"
            );
            assert!(
                covered >= MARK_MIN_COVERED_CELLS,
                "{ctx}: only {covered} cells in the mark's lane ended nearer the ink than \
                 their own ground (floor {MARK_MIN_COVERED_CELLS}; darkest cell reached \
                 {reached:.2}) — whatever is drawn there is not a shape a reader can find"
            );

            // The spine-side connector gap: strictly between the spine and the
            // cluster's RESTING label end, inset by a pixel and a half at each
            // end so the spine's own antialiasing and the label's first glyph
            // are excluded in both selection states.
            let (glo, ghi) = (
                r.spine_x.min(r.resting_label_anchor) + 1.5,
                r.spine_x.max(r.resting_label_anchor) - 1.5,
            );
            if ghi > glo {
                let gap = differ_in(
                    &frame_on,
                    &frame_off,
                    cw as i64,
                    glo,
                    ghi,
                    r.row_top + 1.0,
                    r.row_bottom - 1.0,
                );
                assert!(
                    gap <= 2,
                    "{ctx}: {gap} px changed in the SPINE-side connector gap [{glo}, {ghi}] \
                     — the mark is still being drawn against the spine, which is the side \
                     it was moved off"
                );
            }
            graded += 1;
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded >= 4, "the pixel sweep graded only {graded} cells");
}

// ---------------------------------------------------------------------------
// LAW 4 — the per-world mark is a per-world number of pixels
// ---------------------------------------------------------------------------

/// THE MARK'S WEIGHT AND FORM ARE WORLD DATA, MEASURED IN INK.
///
/// The claim that made this round: one mark cannot serve two display faces. It
/// is graded here in drawn pixels rather than in the authored constants, because
/// a theme field nothing reads is exactly the failure a data-only law cannot
/// see. The ORDER is derived from the roster's own authored areas — lighter
/// authorship must paint less ink — never from a world's name, so the law holds
/// if the two worlds' authorship is ever exchanged.
#[test]
fn each_diagonal_world_paints_its_own_authored_mark() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping each_diagonal_world_paints_its_own_authored_mark: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    // The authored ink budget: stroke × the arm length the aperture and reach
    // imply. A proportion, not a prediction of the rasterizer — only its ORDER
    // is used.
    let authored_area = |m: theme::DiagonalMark| m.weight * (m.reach + m.aperture * 10.0);

    let mut measured: Vec<(&str, f32, usize)> = Vec::new();
    for (world, spine) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        let (cw, ch) = (1200u32, 800u32);
        p.set_size(cw as f32, ch as f32);
        let mut on = marked_view(OverlayKind::Command, 12, 0);
        on.overlay_selected = 3;
        let mut off = marked_view(OverlayKind::Command, 12, 0);
        off.overlay_selected = 5;

        p.set_view(&on);
        p.prepare(&device, &queue, cw, ch).unwrap();
        let r = read_mark(&p, cw).expect("a marked diagonal row");
        let frame_on = render_frame(&mut p, &device, &queue, cw, ch);
        p.set_view(&off);
        p.prepare(&device, &queue, cw, ch).unwrap();
        let frame_off = render_frame(&mut p, &device, &queue, cw, ch);

        let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
        let ink = differ_in(
            &frame_on,
            &frame_off,
            cw as i64,
            lo - 1.0,
            hi + 1.0,
            r.row_top,
            r.row_bottom,
        );
        measured.push((world, authored_area(spine.mark), ink));
    }

    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);

    // Every pair with DIFFERENT authorship must differ in drawn ink, in the
    // direction the authorship implies.
    let mut compared = 0usize;
    for (i, &(a_name, a_area, a_ink)) in measured.iter().enumerate() {
        for &(b_name, b_area, b_ink) in measured.iter().skip(i + 1) {
            if (a_area - b_area).abs() < 1e-6 {
                continue;
            }
            let (light, light_ink, heavy, heavy_ink) = if a_area < b_area {
                (a_name, a_ink, b_name, b_ink)
            } else {
                (b_name, b_ink, a_name, a_ink)
            };
            assert!(
                light_ink < heavy_ink,
                "{light} authors the lighter mark and yet paints {light_ink} px against \
                 {heavy}'s {heavy_ink} — the per-world authorship is not reaching the \
                 draw, which is what a shared renderer constant looks like from here"
            );
            compared += 1;
        }
    }
    assert!(
        compared >= 1,
        "no two rostered diagonal worlds author different marks, so this law compared \
         nothing: {measured:?}"
    );
}

// ---------------------------------------------------------------------------
// LAW 5 — every `SettingId × SettingKind`
// ---------------------------------------------------------------------------

/// Fold a settings workspace card into a `ViewState` the way `App::sync_view`
/// does. Local rather than shared, per this directory's own convention.
fn settings_view(ov: &OverlayState, selected: usize) -> ViewState {
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Settings.title().to_string();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    v.overlay_sections = ov.item_sections();
    v.overlay_hint = ov.foot_hint();
    v.overlay_selected = selected.min(ov.item_strings().len().saturating_sub(1));
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = ov.window_rows();
    v
}

/// A settings workspace card with `lens` selected, rows staged and focused.
fn settings_values() -> crate::settings::SettingsValues {
    super::settings_values(1.0, 1.0)
}

fn settings_card(lens: usize) -> OverlayState {
    let vals = settings_values();
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov.set_facet_lens(lens);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    journey.toggle_detail();
    journey.card().expect("the card is up").clone()
}

/// EVERY SETTING ROW'S MARK IS OUTBOARD. `SettingKind` spans a toggle, a
/// picker, a range with a drawn rail, a path, a submenu and an action — six row
/// CONTENTS through one row planner. The range row is the one that could
/// genuinely differ: it draws a rail inside the row from a different owner, and
/// a law that only graded toggles would never see a mark colliding with it or
/// being pushed off the card by it.
///
/// Coverage is asserted against the registry rather than a count, so a new
/// setting cannot slip in un-swept, and against both worlds and both DPIs.
#[test]
fn every_setting_id_and_kind_carries_an_outboard_mark() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping every_setting_id_and_kind_carries_an_outboard_mark: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let names: Vec<String> = crate::settings::visible_names();
    let registry: Vec<&crate::settings::SettingRow> = crate::settings::SETTINGS
        .iter()
        .filter(|r| names.iter().any(|n| n == r.name))
        .collect();
    assert_eq!(
        registry.len(),
        names.len(),
        "the visible corpus and the registry must line up 1:1 or the coverage claim is \
         about the wrong rows"
    );

    let mut seen_ids = std::collections::BTreeSet::new();
    let mut seen_kinds = std::collections::BTreeSet::new();
    let mut graded = 0usize;
    for (world, _) in diagonal_worlds() {
        let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            // Wide and staged-narrow: a workspace's narrow staging is a second
            // geometry, not a smaller first one.
            for &(lw, lh) in &[(1400u32, 900u32), (620, 820)] {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                // Two lens categories, so a CATEGORY CHANGE is swept: the row
                // corpus changes under a fixed surface, which is exactly when a
                // content-derived lane would move.
                for lens in [0usize, 1] {
                    for (idx, row) in registry.iter().enumerate() {
                        let ov = settings_card(lens);
                        p.set_view(&settings_view(&ov, idx));
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let ctx = format!(
                            "{world} {:?} ({:?}) lens={lens} dpi={dpi} {cw}x{ch}",
                            row.id, row.kind
                        );
                        let Some(r) = read_mark(&p, cw) else {
                            continue;
                        };
                        assert_mark_is_outboard(&r, &ctx);
                        seen_ids.insert(format!("{:?}", row.id));
                        seen_kinds.insert(format!("{:?}", row.kind));
                        graded += 1;
                    }
                }
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);

    let want_ids: std::collections::BTreeSet<String> =
        registry.iter().map(|r| format!("{:?}", r.id)).collect();
    let want_kinds: std::collections::BTreeSet<String> =
        registry.iter().map(|r| format!("{:?}", r.kind)).collect();
    assert_eq!(
        seen_ids,
        want_ids,
        "every SettingId in the visible registry must have been graded — missing {:?}",
        want_ids.difference(&seen_ids).collect::<Vec<_>>()
    );
    assert_eq!(
        seen_kinds,
        want_kinds,
        "every SettingKind must have been reached — missing {:?}",
        want_kinds.difference(&seen_kinds).collect::<Vec<_>>()
    );
    assert!(
        graded > 100,
        "the settings sweep must grade every row on every cell, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 6 — the composition is structurally absent everywhere else
// ---------------------------------------------------------------------------

/// EVERY WORLD THAT IS NOT A DIAGONAL WORLD RESERVES NOTHING AND MEASURES NO
/// CLUSTER — the structural half of "the other worlds are byte-identical".
///
/// It is a type-level guarantee that only `ListStyle::Diagonal` can carry mark
/// data, so the risk is not a stray field: it is a SHARED length quietly
/// changing. This asserts the two doors every diagonal quantity passes through
/// answer inertly on the rest of the roster, and counts the worlds it graded so
/// a roster read that found nothing cannot pass.
#[test]
fn no_upright_world_reserves_diagonal_side_territory() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping no_upright_world_reserves_diagonal_side_territory: no adapter");
        return;
    };
    let mut upright = 0usize;
    let mut diagonal = 0usize;
    for world in theme::THEMES {
        let _pin = theme::WorldPin::world(world.name).expect("a rostered world sets active");
        p.sync_theme();
        p.set_view(&marked_view(OverlayKind::Command, 12, 0));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        match world.render_caps.list_style {
            theme::ListStyle::Diagonal(_) => {
                assert!(
                    p.diagonal_side_reserve_px(12) > 0.0,
                    "{}: a diagonal world reserves side territory",
                    world.name
                );
                diagonal += 1;
            }
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Ruled(_) => {
                assert_eq!(
                    p.diagonal_side_reserve_px(12),
                    0.0,
                    "{}: an upright world reserves no side territory, so the mark's new \
                     lane cannot have widened its card",
                    world.name
                );
                assert!(
                    p.diagonal_cluster_probe().is_none(),
                    "{}: an upright world measures no cluster",
                    world.name
                );
                upright += 1;
            }
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        upright >= 15 && diagonal >= 2,
        "the roster split must be real: {upright} upright, {diagonal} diagonal"
    );
}
