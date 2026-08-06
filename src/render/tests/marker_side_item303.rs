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
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {}
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
    v.overlay_title = kind.title();
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
    /// The accessory column's own `(left, right)` ink box — the nearest thing the
    /// mark could collide with, and the only row content on its side.
    accessory_span: (f32, f32),
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
    let (vertex, arm) = probe.mark_span(sel);
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
        accessory_span: probe.accessory_span(sel),
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
/// inset. Every comparison below is multiplied by `inset`, so the same four
/// lines grade both mirrors and there is nowhere for a per-world constant to
/// hide: swap the sign in `DiagonalClusterRail::mark_span` and every cell fails.
fn assert_mark_is_outboard(r: &MarkReading, ctx: &str) {
    let s = r.inset.signum();
    assert!(
        r.inset.abs() > 0.5,
        "{ctx}: the planner published no signed inset ({}) — this law's whole \
         oracle is that sign, so a zero makes every claim below vacuous",
        r.inset
    );
    // The cluster's OUTER end is downstream of its spine end, along the inset's
    // own direction. This is the premise the mark inherits.
    assert!(
        (r.accessory_anchor - r.label_anchor) * s >= 0.0,
        "{ctx}: the cluster's accessory end ({}) is not outboard of its label end \
         ({}) along the planner's inset sign {s}",
        r.accessory_anchor,
        r.label_anchor
    );
    // THE HEADLINE: the mark stands beyond the cluster's outer end, on the side
    // away from the spine.
    assert!(
        (r.vertex - r.accessory_anchor) * s > 0.0,
        "{ctx}: the mark's vertex ({}) is not outboard of the cluster's outer end \
         ({}) — it is on the SPINE side of the row, which is the defect this law \
         is named for (inset sign {s}, spine at {})",
        r.vertex,
        r.accessory_anchor,
        r.spine_x
    );
    // …and therefore beyond the spine by more than the whole cluster.
    assert!(
        (r.vertex - r.spine_x) * s > (r.accessory_anchor - r.spine_x) * s,
        "{ctx}: the mark must be further from the spine than the cluster is"
    );
    // THE MARK POINTS BACK INTO THE ROW: the vertex is its inner end and the
    // arms open outward, into the card's margin.
    assert!(
        (r.arm - r.vertex) * s > 0.0,
        "{ctx}: the arms ({}) must open AWAY from the row from the vertex ({}) — a \
         mark whose vertex is outboard points out of the card",
        r.arm,
        r.vertex
    );
    // NO COLLISION with the row's own accessory column, which is the only row
    // content on the mark's side of the cluster. The two are disjoint by
    // construction — the mark's lane is reserved beyond the column's outer edge —
    // and that is asserted rather than argued, because a chord, a value readout
    // and a Range rail all land in that column at different widths.
    let (lo, hi) = (r.vertex.min(r.arm), r.vertex.max(r.arm));
    let (alo, ahi) = r.accessory_span;
    assert!(
        hi <= alo + 0.01 || lo >= ahi - 0.01,
        "{ctx}: the mark [{lo}, {hi}] overlaps the accessory column's ink box \
         [{alo}, {ahi}] — the mark's lane is meant to be reserved clear of it"
    );
    // NO CLIPPING: both abscissae stay inside the card.
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

/// REAL PIXELS — SELECTING A ROW PAINTS INK IN ITS OUTER LANE AND LEAVES THE
/// SPINE-SIDE CONNECTOR GAP UNTOUCHED.
///
/// The sidecar and the geometry probes are state oracles; this is the appearance
/// oracle, and it is the law that fails on the shipped defect rather than on a
/// renamed field. Two frames of the same card differing only in WHICH row is
/// selected are compared inside two boxes on the graded row: the mark's own lane
/// beyond the cluster's outer end, and the connector gap at the spine end where
/// the mark used to be drawn. The first must gain ink; the second must not.
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
    v.overlay_title = OverlayKind::Settings.title();
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
    crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.0,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    }
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
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Rules(_) => {
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
