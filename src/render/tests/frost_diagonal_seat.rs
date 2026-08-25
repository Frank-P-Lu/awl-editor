//! THE SHIPPED DIAGONAL DEFAULTS — the top-seated frost and the mirror-seated query,
//! graded as the frame actually draws them rather than through an audition env gate
//! (there is no gate left to read: both are the shipped path now).
//!
//! Two symptoms named the composition's own defect: a document heading straddled by the
//! frost's own top face read as melting half-legible ink, and a query caret seated at
//! the text edge on a world whose every row anchors the opposite side left ~900 physical
//! px of dead air between the caret and the first candidate. Both are graded here in the
//! terms the defect was named in — a row that reads as split, a caret that reads as far —
//! swept over [`super::diagonal_worlds`] (derived from `ListStyle::Diagonal`, never a
//! named pair) at both DPIs.

use super::super::*;
use super::diagonal_worlds;
use super::frost_feather::{DENSE, theme_picker};
use super::headless_dqp;

/// The coverage floor under a pixel the frost claims to cover — the same value
/// `frost_parallelogram`/`frost_width` float theirs at, and the same arithmetic reason:
/// the box lands exactly on the outermost surface's own edge, where the mask is exactly
/// `1.0`, and a `smoothstep` over a 28 logical px feather is still 0.996 a pixel out.
const ROW_FROST_FLOOR: f32 = 0.9;

/// A mask reading this far under the floor is the OTHER thing a sample can be: sharp,
/// live document, not a rounding wobble near `1.0`. Used only to prove a sample outside
/// the frost's reach actually reads as uncovered, so the floor check above cannot be
/// satisfied by a mask function that always answers `1.0`.
const SHARP_CEILING: f32 = 0.1;

fn typed_query_picker(text: &str) -> ViewState {
    let mut v = theme_picker(text);
    v.overlay_query = "mangrove".to_string();
    v.overlay_query_caret = v.overlay_query.chars().count();
    v
}

// ---------------------------------------------------------------------------
// LAW 1 — the frost's top face never sits below the canvas top on a diagonal
// composition, so no document row it draws over can straddle it.
// ---------------------------------------------------------------------------

/// THE TOP SEAT, AND THE ROW IT WAS NAMED FOR.
///
/// Geometric half: [`blur::footprint_seat_top`] always answers `y ≤ 0` on a diagonal
/// composition, and no drawn document row ever starts above `y = 0` — so the top face
/// can never fall inside a row's own vertical span. Pixel half: the fixture's own H1
/// (`DENSE`'s first line, "# The feathered footprint") is sampled at its own top,
/// middle and bottom, at an x the frost's shape reaches — all three must clear the same
/// coverage floor `frost_parallelogram`/`frost_width` already ship, which is what "does
/// not straddle" means in drawn pixels rather than in the rect alone.
#[test]
fn the_diagonal_top_face_never_splits_the_document_row_it_used_to_straddle() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1600.0, 900.0) else {
        eprintln!("skipping the_diagonal_top_face_never_splits_the_document_row: no adapter");
        return;
    };
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    let mut seated_below_zero = 0usize;

    for world in diagonal_worlds() {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in &[(1200u32, 900u32), (1600, 900)] {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                p.set_view(&typed_query_picker(DENSE));
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{world} dpi={dpi} {cw}x{ch}");
                let card = p.overlay_card_rect().expect("a crisp picker has a card");
                let frost = p.frost_mode().expect("an enrolled diagonal world frosts");
                let rect = match frost {
                    crate::render::blur::Frost::Footprint(f) => f.rect,
                    other => panic!("{ctx}: expected the footprint arm, got {other:?}"),
                };

                // NON-VACUITY: the card's own top must sit well below the canvas top,
                // or this cell cannot tell a seated face from an unseated one (a card
                // already opening at the window's edge would pass either way).
                assert!(
                    card[1] > 5.0,
                    "{ctx}: the card's own top {} is already near the canvas top",
                    card[1]
                );
                assert!(
                    rect[1] <= 0.5,
                    "{ctx}: the frost's top face sits at {} — below the canvas top, so a \
                     document row above the card can straddle it (the H1-melting defect \
                     this seat fixed)",
                    rect[1]
                );
                if rect[1] <= 0.0 {
                    seated_below_zero += 1;
                }

                // THE ROW ITSELF: the fixture's own H1 is document row 0. Its canvas y
                // comes from the SAME accessor the render path reads, never a second
                // measurement.
                let row_top = p.doc_top() + p.row_top_px(0);
                let row_bottom = row_top + p.row_height_px(0);
                assert!(
                    row_bottom > row_top,
                    "{ctx}: document row 0 measured zero height"
                );
                let sample_x = rect[0] + rect[2] * 0.5;
                for (name, y) in [
                    ("top", row_top),
                    ("mid", (row_top + row_bottom) * 0.5),
                    ("bottom", row_bottom),
                ] {
                    let m = crate::render::blur::footprint_mask_for(frost, dpi, sample_x, y);
                    assert!(
                        m >= ROW_FROST_FLOOR,
                        "{ctx}: the H1's own row reads {m:.4} frost coverage at its {name} \
                         ({sample_x:.1},{y:.1}), under the floor {ROW_FROST_FLOOR} — the row \
                         is split between sharp and blurred, the melting defect this seat \
                         exists to remove"
                    );
                }
                // AND THE MASK FUNCTION ACTUALLY DISCRIMINATES: a point comfortably
                // outside the shape's reach reads sharp, so the floor above is not
                // satisfied by a mask that always answers `1.0`.
                let outside = crate::render::blur::footprint_mask_for(
                    frost,
                    dpi,
                    (rect[0] - 400.0 * dpi).max(0.0),
                    row_top,
                );
                assert!(
                    outside <= SHARP_CEILING,
                    "{ctx}: a sample 400 logical px outside the frost's reach still reads \
                     {outside:.4} coverage — this law's floor check is vacuous"
                );
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    crate::theme::set_active(entry);
    assert!(graded >= 8, "the sweep graded only {graded} cells");
    assert!(
        seated_below_zero == graded,
        "{seated_below_zero}/{graded} cells actually seated the top face at or above the \
         canvas top — the seat is not reaching every diagonal world"
    );
}

// ---------------------------------------------------------------------------
// LAW 2 — the query-to-first-item distance is bounded on every diagonal world.
// ---------------------------------------------------------------------------

/// One cell's reading of the query caret and the first candidate row's own label start.
fn read_query_gap(p: &TextPipeline, cw: u32) -> Option<(f32, f32)> {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let caret = p.overlay_query_caret_box(&geom, &plan)?;
    let first = plan.rows().iter().find(|r| r.item.is_some())?;
    let cluster = p.diagonal_cluster_probe()?;
    let ink_w = p
        .overlay_row_primary_px(&geom)
        .get(&first.display)
        .copied()
        .unwrap_or(0.0);
    let label_x = cluster.label_origin(first.display, ink_w);
    Some((caret[0], label_x))
}

/// THE ITEM'S OWN DIAGNOSIS, BOUNDED: the query caret never sits absurdly far from the
/// first candidate row. Measured on this fixture: 102.7 logical px worst-case with the
/// mirror-seated query shipped, 311.8 with it reverted to the text edge on every world
/// (`ColumnFlow::Leftward` forced to answer `geom.text_left`, the pre-pick placement) —
/// so 200 sits with real margin above the shipped worst and well under the broken one,
/// and is not satisfied by deleting the fix.
const QUERY_GAP_CEILING_LOGICAL_PX: f32 = 200.0;

#[test]
fn the_query_caret_never_sits_absurdly_far_from_the_first_candidate() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1600.0, 900.0) else {
        eprintln!("skipping the_query_caret_never_sits_absurdly_far: no adapter");
        return;
    };
    let entry = crate::theme::active_index();
    let mut graded = 0usize;
    let mut worst: Vec<(String, f32)> = Vec::new();

    for world in diagonal_worlds() {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in &[(1200u32, 900u32), (1600, 900)] {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                p.set_view(&typed_query_picker(DENSE));
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{world} dpi={dpi} {cw}x{ch}");
                let Some((caret_x, label_x)) = read_query_gap(&p, cw) else {
                    continue;
                };
                let gap_logical = (caret_x - label_x).abs() / dpi;
                worst.push((ctx.clone(), gap_logical));
                assert!(
                    gap_logical < QUERY_GAP_CEILING_LOGICAL_PX,
                    "{ctx}: the query caret sits {gap_logical:.1} logical px from the first \
                     candidate's own label start — at or past the item's own ~900 PHYSICAL \
                     px baseline once converted, the query-to-first-item sprawl this item \
                     was named for"
                );
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    crate::theme::set_active(entry);
    assert!(graded >= 8, "the sweep graded only {graded} cells");
    let (worst_ctx, worst_gap) = worst
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .cloned()
        .expect("graded at least one cell");
    eprintln!("QUERY GAP worst: {worst_gap:.1} logical px at {worst_ctx}");
}
