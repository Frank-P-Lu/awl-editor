//! SCROLLING A PICKER MOVES THE LIST, AND NOTHING ELSE.
//!
//! Mangrove's diagonal spine swung sideways whenever a picker's list scrolled.
//! Nothing about the spine was drawn wrong: its whole GEOMETRY was derived from
//! the rows in front of it. `DiagonalClusterRail::new` sized the per-row travel
//! from the side territory LEFT OVER after the widest VISIBLE row's cluster, and
//! the visible set is exactly what a scroll changes — so a long filename
//! scrolling out of a picker swung the line from nearly upright to a full rake,
//! and every row moved with it. Both diagonal worlds had it, in mirror image.
//! Its two smaller siblings had the same shape: the accessory rail and, on an
//! ascending world, the whole cluster hanging off it, stepped sideways when a
//! longer chord scrolled in; and a right-anchored card's content hug
//! re-measured the visible window, translating the whole card by a measured
//! 31.1 px on Mangrove and Cassowary alike whenever its width bound.
//!
//! Three quantities changed owner, all of them toward the SURFACE:
//!   * the spine's travel is now `spine_travel` — the authored per-row step over
//!     the drawn rows, bounded by a fraction of the card's own side territory,
//!     with no row in the formula at all;
//!   * `diagonal_cluster_budget` subtracts that same reservation, so a row's
//!     elision is the same at every scroll position;
//!   * the cluster's reserved label and accessory extents, and a right-anchored
//!     card's hug width, come off the WHOLE candidate roster — a picker's width
//!     is a property of its content, and a scroll position is not content.
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

/// THE EMPIRICAL WORST CASE, and the shape that actually reproduced: a picker
/// whose widest rows sit at the TOP. Scrolling past them shrinks the widest
/// visible label enormously, which is precisely the input the spine's travel
/// used to read. A roster of uniform-width rows would have hidden the defect
/// completely — the real `⌘P` palette nearly does, which is why it is swept
/// beside this one rather than instead of it.
fn varied_view() -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "go to";
    v.overlay_items = (0..40)
        .map(|i| match i {
            0..=1 => format!("a-considerably-longer-note-name-{i}-here.md"),
            _ => format!("n{i:02}.md"),
        })
        .collect();
    v.overlay_bindings = (0..40)
        .map(|i| {
            if i % 7 == 0 {
                "C-c C-o".into()
            } else {
                String::new()
            }
        })
        .collect();
    v.overlay_selected = 0;
    v.overlay_window_rows = 12;
    v
}

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

/// The per-cell verdict: the card held still, every row's SURFACE strip is
/// byte-identical at every scroll position, and the LIST halves really moved.
fn assert_still(
    at: &str,
    rects: &[[f32; 4]],
    frames: &[Vec<[u8; 4]>],
    splits: &[(pixeldiff::Region, pixeldiff::Region)],
    w: u32,
    h: u32,
) {
    for (i, rect) in rects.iter().enumerate().skip(1) {
        assert_eq!(
            *rect, rects[0],
            "{at}: the card moved between scroll {} and {} — {:?} -> {rect:?}",
            TRAJECTORY[0], TRAJECTORY[i], rects[0],
        );
    }
    let mut moved_rows = 0usize;
    for (d, (surface, list)) in splits.iter().enumerate() {
        for (i, frame) in frames.iter().enumerate().skip(1) {
            let report = pixeldiff::diff_region(&frames[0], frame, w as i64, h as i64, *surface);
            assert_eq!(
                report.differing,
                0,
                "{at}: row {d}'s SURFACE strip changed between scroll {} and {} \
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
    // NON-VACUITY: the list really did scroll. Without this the law would pass
    // just as happily on a picker that ignored every scroll key.
    assert!(
        moved_rows >= splits.len() / 2,
        "{at}: only {moved_rows} of {} rows changed content over the trajectory — \
         the list did not scroll, so the surface claim above proves nothing",
        splits.len(),
    );
}

/// The headline law, over a diagonal world AND an upright control that shares
/// the same right-anchored hug mechanism, AND a plain centred world.
///
/// Every region is computed ONCE, from the first frame, and reused verbatim at
/// every later scroll position. That is the whole point: if the card translates,
/// a fixed canvas region stops holding the same thing and the law goes red —
/// which a per-frame region would have hidden by translating with it.
///
/// THE AXIS THAT ALMOST GOT AWAY IS THE CANVAS. A right-anchored card only HUGS
/// while its content fits inside the width cap; on a roomy canvas both
/// right-anchored worlds sit at the cap, the hug never binds, and every claim
/// below holds no matter what the measurement does. The first cut of this law
/// ran at one comfortable canvas and stayed green under the exact regression it
/// is named for. It now sweeps canvases and REFUSES to pass unless the hug
/// actually bound somewhere.
const CANVASES: [(u32, u32); 3] = [(1200, 800), (1400, 900), (1040, 760)];

#[test]
fn scrolling_a_picker_moves_only_its_list_never_its_surface() {
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping scrolling_a_picker_moves_only_its_list: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let mut visible_width_swings = 0usize;

    for (fixture, build) in [
        (
            "goto (widest rows at the top)",
            varied_view as fn() -> ViewState,
        ),
        (
            "the real command palette",
            palette_view as fn() -> ViewState,
        ),
    ] {
        for (w, h) in CANVASES {
            p.set_size(w as f32, h as f32);
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

                let mut widest_visible: Vec<f32> = Vec::new();
                for scroll in TRAJECTORY {
                    let mut v = build();
                    v.overlay_scroll = scroll;
                    v.overlay_selected = scroll;
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();
                    rects.push(p.overlay_card_rect().expect("the palette card"));
                    // The quantity that USED to move the spine: the widest primary
                    // cell currently on screen. Recorded so the law can prove it
                    // really varied over this trajectory.
                    let geom = p.overlay_geometry(w);
                    widest_visible.push(
                        p.overlay_row_primary_px(&geom)
                            .values()
                            .copied()
                            .fold(0.0, f32::max),
                    );
                    if splits.is_none() {
                        let rows = p.overlay_row_plan(&geom).rows().len();
                        splits = Some((0..rows).map(|d| split_row(&p, w, d)).collect());
                    }
                    frames.push(pixeldiff::render_frame(&mut p, &device, &queue, w, h));
                }
                let splits = splits.expect("the first frame's row split");
                let at = format!("{fixture} / {world} @ {w}x{h}");
                let lo = widest_visible.iter().copied().fold(f32::INFINITY, f32::min);
                let hi = widest_visible
                    .iter()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max);
                visible_width_swings += usize::from(hi - lo > 40.0);

                assert_still(&at, &rects, &frames, &splits, w, h);
            }
        }
    }
    theme::set_active(theme::DEFAULT_THEME);

    // 4. NON-VACUITY, the second kind, and the one the first cut of this law did
    //    not have: the WIDEST VISIBLE ROW — the input every moved quantity used
    //    to read — must genuinely swing over the trajectory somewhere in the
    //    sweep. A roster of uniform rows satisfies every claim above no matter
    //    what the composition reads.
    assert!(
        visible_width_swings > 0,
        "no swept cell saw the widest visible row change by more than 40 px — the \
         input that used to move the spine never varied, so the sweep proves nothing"
    );
}

/// THE DIAGONAL'S OWN GEOMETRY IS SCROLL-INVARIANT — the spine's rake, its
/// attachment band and its per-row travel are the same numbers at every scroll
/// position. The pixel law above proves the ink did not move; this names WHICH
/// quantity would have, so a regression reads as "the step changed" rather than
/// as an anonymous pixel diff.
///
/// It reads BOTH cluster edges as well as the spine, so the rail the labels and
/// the chords hang on is pinned in the same breath: on a descending world the
/// label column's left edge and the accessory column's right edge, mirrored on
/// an ascending one.
#[test]
fn the_diagonal_spine_geometry_does_not_read_the_scroll_position() {
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_diagonal_spine_geometry_does_not_read_scroll: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();

    for (fixture, build) in [
        (
            "goto (widest rows at the top)",
            varied_view as fn() -> ViewState,
        ),
        (
            "the real command palette",
            palette_view as fn() -> ViewState,
        ),
    ] {
        for ((w, h), world) in CANVASES
            .into_iter()
            .flat_map(|c| ["Mangrove", "Magpie"].map(|n| (c, n)))
        {
            p.set_size(w as f32, h as f32);
            theme::set_active_by_name(world).unwrap();
            let mut seen: Option<(RowSpan, f32, f32, f32, f32)> = None;
            for scroll in TRAJECTORY {
                let mut v = build();
                v.overlay_scroll = scroll;
                v.overlay_selected = scroll;
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
                let now = (
                    probe.span,
                    probe.spine_x(0),
                    probe.spine_x(4),
                    probe.label_left(4),
                    probe.accessory_right(4),
                );
                match seen {
                    None => {
                        assert!(
                            probe.span.dx_per_row != 0.0 || probe.span.dw_per_row != 0.0,
                            "{fixture} / {world} @ {w}x{h}: the spine has no rake \
                             at all — a zero step makes every scroll-invariance \
                             claim below vacuous"
                        );
                        seen = Some(now);
                    }
                    Some(first) => assert_eq!(
                        now, first,
                        "{fixture} / {world} @ {w}x{h}: the diagonal moved at \
                         scroll {scroll} — {first:?} -> {now:?}"
                    ),
                }
            }
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
}
