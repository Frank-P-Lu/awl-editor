//! ITEM 222 — SCROLLING A PICKER MOVES THE LIST, AND NOTHING ELSE.
//!
//! Mangrove's diagonal spine slid sideways whenever the command list scrolled.
//! The spine was never the thing that moved: THE CARD WAS. A right-anchored card
//! hugs its measured content (item 51), that measurement read the widest row of
//! the VISIBLE WINDOW, and the window is exactly what a scroll changes — so a
//! longer chord scrolling into view re-hugged the card, and a card pinned by its
//! right edge to the interior rail can only grow LEFTWARD. Everything composed
//! against the card — border, ground, rows and, most visibly, the raking spine —
//! translated with it. Both right-anchored worlds had it (Mangrove by 31.1 px
//! and Cassowary by exactly the same 31.1 px over the same trajectory); Mangrove
//! is simply the one where a long straight line makes the motion impossible to
//! miss.
//!
//! The hug width now comes off the WHOLE candidate roster
//! (`measure_roster_primary_px` / `measure_roster_secondary_px`), because a hug
//! width is a property of a picker's content and a scroll position is not
//! content.
//!
//! The law is a scroll TRAJECTORY, and its pixel claim is per row and
//! per surface: the strip of each row that belongs to the SURFACE — the card
//! ground, its border and the spine that rakes across it — must be byte-identical
//! at every scroll position, while the strip that belongs to the LIST must
//! actually change (or the law would pass on a picker that never scrolled).

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::render::plan::RowSpan;

/// The trajectory: the selection walks down the roster with the window, so the
/// SELECTED row stays display 0 at every step. Everything that tracks selection
/// — the band, the bright local spine segment, the outward step — is therefore
/// held still by construction, and the only thing left that could move is what
/// this law is about.
const TRAJECTORY: [usize; 6] = [0, 1, 3, 7, 12, 20];

fn palette_view() -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = crate::commands::names();
    v.overlay_bindings = crate::commands::effective_bindings(&[], &[]);
    v.overlay_selected = 0;
    v.overlay_window_rows = 12;
    v.overlay_lens = crate::facets::scheme(crate::overlay::OverlayKind::Command)
        .map(|s| {
            s.strip
                .iter()
                .enumerate()
                .map(|(i, f)| (f.label.to_string(), i == 0))
                .collect()
        })
        .unwrap_or_default();
    v
}

/// Split display row `d` into the SURFACE half and the LIST half, in canvas
/// coordinates, with NO WILDCARD over the row compositions: a diagonal world's
/// cluster sits on one side of its spine and the surface owns the other, an
/// upright world's rows start at the card's own text edge.
fn split_row(p: &TextPipeline, w: u32, d: usize) -> (pixeldiff::Region, pixeldiff::Region) {
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let row = plan.rows()[d];
    let (top, height) = (row.top, (row.bottom() - row.top).max(1.0));
    let card = p.overlay_card_rect().expect("the palette card");
    let (x0, x1) = (card[0], card[0] + card[2]);
    let cut = match crate::render::effective_list_style() {
        theme::ListStyle::Diagonal(theme::DiagonalDirection::Descending) => {
            let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
            (probe.label_left(d), true)
        }
        theme::ListStyle::Diagonal(theme::DiagonalDirection::Ascending) => {
            let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
            (probe.accessory_right(d), false)
        }
        theme::ListStyle::Pane | theme::ListStyle::Bars { .. } => {
            (geom.text_left + plan.row_dx(d), true)
        }
    };
    // A ONE-PIXEL SEAM at the boundary belongs to neither half: a glyph shaped
    // hard against its column's edge antialiases a fraction past it, and a law
    // that counted that as the surface moving would be reporting the list's own
    // ink. The seam is smaller than any real translation this law exists to
    // catch (the reported motion was 31 px).
    const SEAM: f32 = 2.0;
    let (edge, surface_is_left) = cut;
    match surface_is_left {
        // Surface to the LEFT of the row's own content, list to the right.
        true => (
            pixeldiff::Region::new(x0, top, (edge - SEAM - x0).max(1.0), height),
            pixeldiff::Region::new(edge, top, (x1 - edge).max(1.0), height),
        ),
        // Mirrored: an ascending world's clusters hug the left of their spine.
        false => (
            pixeldiff::Region::new(edge + SEAM, top, (x1 - edge - SEAM).max(1.0), height),
            pixeldiff::Region::new(x0, top, (edge - x0).max(1.0), height),
        ),
    }
}

/// The headline law, over a diagonal world AND an upright control that shares
/// the same right-anchored hug mechanism, AND a plain centred world.
///
/// Every region is computed ONCE, from the first frame, and reused verbatim at
/// every later scroll position. That is the whole point: if the card translates,
/// a fixed canvas region stops holding the same thing and the law goes red —
/// which a per-frame region would have hidden by translating with it.
#[test]
fn scrolling_a_picker_moves_only_its_list_never_its_surface() {
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping scrolling_a_picker_moves_only_its_list: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    for world in [
        "Mangrove",  // the reported world: diagonal + right-anchored
        "Cassowary", // the control: upright Bars, the SAME right-anchored hug
        "Magpie",    // the mirrored diagonal, left-anchored
        "Tawny",     // a plain centred Pane world
    ] {
        theme::set_active_by_name(world).unwrap();

        let mut frames: Vec<Vec<[u8; 4]>> = Vec::new();
        let mut rects: Vec<[f32; 4]> = Vec::new();
        let mut splits: Option<Vec<(pixeldiff::Region, pixeldiff::Region)>> = None;

        for scroll in TRAJECTORY {
            let mut v = palette_view();
            v.overlay_scroll = scroll;
            v.overlay_selected = scroll;
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            rects.push(p.overlay_card_rect().expect("the palette card"));
            if splits.is_none() {
                let rows = p.overlay_row_plan(&p.overlay_geometry(w)).rows().len();
                splits = Some((0..rows).map(|d| split_row(&p, w, d)).collect());
            }
            frames.push(pixeldiff::render_frame(&mut p, &device, &queue, w, h));
        }
        let splits = splits.expect("the first frame's row split");

        // 1. THE CARD ITSELF DOES NOT MOVE OR RESIZE.
        for (i, rect) in rects.iter().enumerate().skip(1) {
            assert_eq!(
                *rect, rects[0],
                "{world}: the card moved between scroll {} and {} — {:?} -> {rect:?}",
                TRAJECTORY[0], TRAJECTORY[i], rects[0],
            );
        }

        // 2. EVERY ROW'S SURFACE STRIP IS BYTE-IDENTICAL AT EVERY SCROLL POSITION.
        let mut moved_rows = 0usize;
        for (d, (surface, list)) in splits.iter().enumerate() {
            for (i, frame) in frames.iter().enumerate().skip(1) {
                let report =
                    pixeldiff::diff_region(&frames[0], frame, w as i64, h as i64, *surface);
                assert_eq!(
                    report.differing,
                    0,
                    "{world}: row {d}'s SURFACE strip changed between scroll {} and {} \
                     ({} of {} px, max channel delta {}) — the ground, border or spine \
                     moved with the list",
                    TRAJECTORY[0],
                    TRAJECTORY[i],
                    report.differing,
                    report.total,
                    report.max_channel_delta,
                );
            }
            let listed = pixeldiff::diff_region(
                &frames[0],
                frames.last().unwrap(),
                w as i64,
                h as i64,
                *list,
            );
            moved_rows += usize::from(listed.differing > 0);
        }

        // 3. NON-VACUITY: the list really did scroll. Without this the law would
        //    pass just as happily on a picker that ignored every scroll key.
        assert!(
            moved_rows >= splits.len() / 2,
            "{world}: only {moved_rows} of {} rows changed content over the trajectory — \
             the list did not scroll, so the surface claim above proves nothing",
            splits.len(),
        );
    }
    theme::set_active(theme::DEFAULT_THEME);
}

/// THE DIAGONAL'S OWN GEOMETRY IS SCROLL-INVARIANT — the spine's rake, its
/// attachment band and its per-row travel are the same numbers at every scroll
/// position. The pixel law above proves the ink did not move; this names WHICH
/// quantity would have, so a regression reads as "the step changed" rather than
/// as an anonymous pixel diff.
///
/// WHAT THIS DOES NOT CLAIM, deliberately: the cluster's own LABEL extent is
/// still the widest label of the VISIBLE window, so on an ASCENDING world (whose
/// clusters hang from the right of their spine) a row's label LEFT edge still
/// steps when a wider label scrolls in — measured at ~22 px over this
/// trajectory on Magpie. That is the measured cluster rail item 131 reserves
/// "across the visible set", and widening it to the roster is item 131d's
/// slice, not this one's: the accessory extent is reserved here because it is
/// what moved the SPINE-anchored rail, and the label extent interacts with
/// elision and the band budget in a way that is a composition decision.
#[test]
fn the_diagonal_spine_geometry_does_not_read_the_scroll_position() {
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_diagonal_spine_geometry_does_not_read_scroll: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    for world in ["Mangrove", "Magpie"] {
        theme::set_active_by_name(world).unwrap();
        let mut seen: Option<(RowSpan, f32, f32)> = None;
        for scroll in TRAJECTORY {
            let mut v = palette_view();
            v.overlay_scroll = scroll;
            v.overlay_selected = scroll;
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
            let now = (probe.span, probe.spine_x(0), probe.spine_x(4));
            match seen {
                None => {
                    assert!(
                        probe.span.dx_per_row != 0.0 || probe.span.dw_per_row != 0.0,
                        "{world}: the spine has no rake at all — a zero step makes every \
                         scroll-invariance claim below vacuous"
                    );
                    seen = Some(now);
                }
                Some(first) => assert_eq!(
                    now, first,
                    "{world}: the diagonal moved at scroll {scroll} — {first:?} -> {now:?}"
                ),
            }
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
}
