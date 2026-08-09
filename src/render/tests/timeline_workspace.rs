//! THE TIMELINE HALF OF THE COMPARISON WORKSPACE.
//!
//! `comparison_composite` owns the CONTENT region — that the relocated
//! document is visible inside it and nowhere else. This file owns the other
//! region and the seam between them:
//!
//! 1. **THE TWO REGIONS NEVER OVERLAP.** A timeline row's own drawn/clickable
//!    span and the comparison's box are disjoint, over the world roster and the
//!    whole geometry range. This is queue item 116's headline pixel law and it is
//!    asserted from the two owners the pixels came from, not from the arithmetic
//!    that placed them.
//! 2. **DRAWN IS CLICKABLE.** Every timeline row resolves under the pointer at the
//!    slot it is drawn in — through the ORDINARY candidate-row hit-test, with no
//!    rail function involved, because `geom.rail` is `None` whenever rows are
//!    primary.
//! 3. **THE FOOTER FITS ITS OWN BAND.** The timeline's column is narrow by design
//!    and its footer rides that column rather than the wide pane beside it, so a
//!    hint measured against the wrong box clips. The last round's vision smoke
//!    found exactly this shape — a footer running off a card while the whole suite
//!    was green — which is why it is a law and not a look.

use super::super::TextPipeline;
use super::{comparison_view, headless_dqp};

/// The LOGICAL window sizes this file sweeps — wide enough to stage both regions,
/// narrow enough to stage them one at a time, tall and short. A swept cell's
/// PHYSICAL canvas is `logical * dpi`, so every cell is a window a real session
/// can actually be in; a physical canvas that DPI shrinks below the app's own
/// enforced minimum is not a geometry the product has to survive, and grading one
/// would be grading the fixture.
const CANVASES: [(u32, u32); 4] = [(1200, 800), (1600, 1000), (900, 700), (760, 620)];

/// A rendered comparison workspace with real timeline rows, whose primary column
/// is genuinely narrow relative to its comparison.
fn timeline_view(rows: usize) -> crate::render::ViewState {
    let mut v = comparison_view("# Transcript\n\nSome compared prose here.\n", 0, 0);
    v.overlay_items = (0..rows)
        .map(|i| format!("{i} hr ago · edited \"A heading somewhere in the draft\""))
        .collect();
    v.overlay_bindings = (0..rows).map(|i| format!("+{i} −{i}")).collect();
    v
}

/// LAW 1 + 2 — THE TWO REGIONS NEVER OVERLAP, AND EVERY TIMELINE ROW IS CLICKABLE
/// WHERE IT IS DRAWN.
///
/// The oracles are deliberately different objects: the comparison's box comes
/// from `comparison_viewport` (which the document layer's four geometry owners
/// read), and a row's box comes from the ROW PLAN (which the draw emitters and
/// the pointer hit-test read). Nothing here recomputes either — a law whose
/// oracle re-derives the code under test is tautological.
#[test]
fn the_timeline_and_the_comparison_never_overlap_and_every_row_is_clickable() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_timeline_and_the_comparison_never_overlap: no adapter");
        return;
    };
    let mut graded = 0usize;
    let mut rows_graded = 0usize;
    let mut wide_cells = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        for (cw, ch) in CANVASES {
            for dpi in [1.0f32, 2.0] {
                let (cw, ch) = ((cw as f32 * dpi) as u32, (ch as f32 * dpi) as u32);
                p.set_dpi(dpi);
                p.set_size(cw as f32, ch as f32);
                p.set_view(&timeline_view(6));
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{} {cw}x{ch} dpi={dpi}", world.name);
                let geom = p.workspace_geometry(cw);
                let plan = p.overlay_row_plan(&geom);

                // THE RAIL FUNCTIONS ARE NOT THE TIMELINE'S. `geom.rail` is
                // `None` whenever rows are primary, so a hit-test that reached
                // for the rail would answer nothing at all — asserted rather than
                // assumed, because "it happens to work" and "it is wired the way
                // the handoff said" are different claims.
                assert!(
                    p.workspace_rail_box(&geom, &plan).is_none(),
                    "{ctx}: a timeline shape has no LABEL rail — its rows fill the primary \
                     column instead, so the rail functions must decline"
                );

                let Some([vx, vy, vw, vh]) = p.comparison_viewport() else {
                    // The narrow stage draws one region at a time; with the
                    // timeline focused there is no comparison on screen and
                    // nothing to overlap. Still grade the rows.
                    rows_graded += grade_rows_are_clickable(&p, &plan, &ctx);
                    graded += 1;
                    continue;
                };
                wide_cells += 1;
                for row in plan.rows() {
                    // A row's OWN span, offsets folded in — the diagonal
                    // compositions step a row's edges, and the band's undisplaced
                    // span is not where such a row is drawn or clicked.
                    let (x0, x1) = row_span(&plan, row);
                    let (top, bottom) = (row.top, row.bottom());
                    let overlaps_x = x1 > vx && x0 < vx + vw;
                    let overlaps_y = bottom > vy && top < vy + vh;
                    assert!(
                        !(overlaps_x && overlaps_y),
                        "{ctx}: timeline row {} occupies x {x0}..{x1} y {top}..{bottom}, \
                         which intersects the comparison region {:?}. The two regions of one \
                         task may share a search, a selection grammar and a back path — they \
                         may not share pixels",
                        row.display,
                        [vx, vy, vw, vh]
                    );
                }
                rows_graded += grade_rows_are_clickable(&p, &plan, &ctx);
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(
        graded >= crate::theme::THEMES.len() * 4,
        "the sweep must reach every world at several geometries, got {graded}"
    );
    assert!(
        rows_graded > 200,
        "the sweep must grade real rows, got {rows_graded}"
    );
    // NON-VACUITY: the overlap half only means something on a cell that draws
    // BOTH regions. A sweep that only ever staged them would prove nothing.
    assert!(
        wide_cells > 20,
        "the sweep must reach cells that draw both regions at once, got {wide_cells}"
    );
}

/// DRAWN IS CLICKABLE, through the ordinary candidate-row hit-test. Returns how
/// many rows were graded.
fn grade_rows_are_clickable(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    ctx: &str,
) -> usize {
    let mut n = 0usize;
    for row in plan.rows() {
        let (x0, x1) = row_span(plan, row);
        let mid_y = row.top + row.height * 0.5;
        for x in [x0 + 1.0, (x0 + x1) * 0.5, x1 - 1.0] {
            assert_eq!(
                p.overlay_row_at(x, mid_y),
                row.item,
                "{ctx}: timeline row {} is drawn in slot {}..{} but a pointer at \
                 ({x}, {mid_y}) resolves differently — DESIGN.md §8: drawn geometry and \
                 hit-test geometry have one owner",
                row.display,
                row.top,
                row.bottom()
            );
        }
        n += 1;
    }
    n
}

/// ONE ROW'S OWN HORIZONTAL SPAN: the content band's, stepped by the row's own
/// planned offsets. The diagonal row compositions move a row's left and right
/// edges independently, and both the draw emitters and `row_at` read those
/// offsets — so a law that tested the band's undisplaced span would be testing a
/// box neither of them uses.
fn row_span(
    plan: &crate::render::plan::OverlayRowPlan,
    row: &crate::render::plan::PlannedRow,
) -> (f32, f32) {
    let (x0, x1) = plan.card_x_span();
    (x0 + row.dx, x1 + row.dw)
}

/// LAW 3 — THE FOOTER FITS THE COLUMN IT RIDES.
///
/// On the rail shape the footer rides the wide content pane, so its width is
/// never the narrow column's problem. On the timeline shape it rides the TIMELINE,
/// which is bounded to a fraction of the workspace's interior — so a column
/// measured from its rows alone clips the line that teaches the keys. Graded from
/// the SHAPED footer's own pixels through the one footer-measure owner, never
/// from the hint STRING's character count.
///
/// Swept over the world roster (a display face that shapes wider than the mean
/// glyph estimate is exactly what a character count would miss) and the whole
/// canvas range, because the bound is a fraction of a canvas-derived interior.
#[test]
fn the_timeline_footer_fits_its_own_column_in_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_timeline_footer_fits_its_own_column: no adapter");
        return;
    };
    let mut graded = 0usize;
    let mut worst = 0.0f32;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        for (cw, ch) in CANVASES {
            for dpi in [1.0f32, 2.0] {
                for rows in [0usize, 6] {
                    let (cw, ch) = ((cw as f32 * dpi) as u32, (ch as f32 * dpi) as u32);
                    p.set_dpi(dpi);
                    p.set_size(cw as f32, ch as f32);
                    let mut v = timeline_view(rows);
                    // The REAL footer line the timeline stage advertises, not a
                    // fixture string: this law is about that sentence fitting.
                    v.overlay_hint = crate::overlay::format_hint(
                        &crate::overlay::OverlayKind::History.rail_hint_actions(),
                    );
                    if rows == 0 {
                        v.overlay_empty = Some(
                            crate::overlay::OverlayKind::History
                                .empty_corpus_message()
                                .to_string(),
                        );
                        v.overlay_bindings = Vec::new();
                    }
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let (footer_px, text_w) = p.overlay_footer_fit_probe(cw);
                    let ctx = format!("{} {cw}x{ch} dpi={dpi} rows={rows}", world.name);
                    assert!(
                        footer_px > 1.0,
                        "{ctx}: the timeline's footer must actually shape glyphs, or this \
                         law grades nothing"
                    );
                    assert!(
                        footer_px <= text_w,
                        "{ctx}: the timeline footer shapes {footer_px:.1}px into a column \
                         {text_w:.1}px wide and clips. The footer rides the TIMELINE on \
                         this shape, not the pane beside it, so the column has to be \
                         measured with the footer in its corpus — or the line that \
                         teaches the keys is the one that gets cut"
                    );
                    worst = worst.max(footer_px / text_w.max(1.0));
                    graded += 1;
                }
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(
        graded >= crate::theme::THEMES.len() * 8,
        "the sweep must reach every world at every canvas, got {graded}"
    );
    // NON-VACUITY, from the other end: if the worst cell in the whole sweep uses
    // a small fraction of its column, the fixture has stopped being able to
    // reproduce a clip at all and this law would stay green through a real one.
    assert!(
        worst > 0.5,
        "the tightest swept cell fills only {worst:.2} of its column — the fixture can no \
         longer reproduce the clip this law is named for"
    );
}

/// LAW 4 — AN EMPTY TIMELINE DOES NOT STAND THE LIVE DOCUMENT UP INSIDE THE
/// WORKSPACE.
///
/// The SHAPE says there is a comparison region; the PAYLOAD says there is
/// something in it, and the two are not the same fact. A timeline can be up with
/// nothing to compare — an empty history, or a query that filters every version
/// away — and on those frames the text the pipeline is handed is the user's OWN
/// document. Relocating it into the comparison's place would put the live
/// document up as a third readable layer inside the workspace, which is the
/// composition this whole surface exists to remove.
///
/// The two arms differ in EXACTLY the payload flag, so the claim is about that
/// fact and nothing else; and the second half is a PIXEL claim, because "the
/// viewport is `None`" alone would still be green if the document layer had found
/// another way in.
#[test]
fn a_timeline_with_nothing_to_compare_leaves_the_document_where_it_was() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_timeline_with_nothing_to_compare: no adapter");
        return;
    };
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();

        let arm = |payload: bool| {
            let mut v = timeline_view(6);
            v.text = super::comparison_composite::sample_transcript();
            v.overlay_comparison = payload;
            v
        };
        let (with, without) = (arm(true), arm(false));

        p.set_view(&with);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        assert!(
            p.comparison_viewport().is_some(),
            "{}: precondition — with a payload the document relocates",
            world.name
        );
        let relocated = super::pixeldiff::render_frame(&mut p, &device, &queue, 1200, 800);

        p.set_view(&without);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        assert!(
            p.comparison_viewport().is_none(),
            "{}: with NOTHING to compare the document must not be relocated — the text on \
             such a frame is the user's own document, not a transcript",
            world.name
        );
        let parked = super::pixeldiff::render_frame(&mut p, &device, &queue, 1200, 800);

        // THE PIXELS AGREE WITH THE GATE. Two frames whose only difference is the
        // payload flag must differ somewhere: on one the document is drawn inside
        // the workspace, on the other it is not drawn there at all. A gate that
        // changed no pixel would be a gate that changed nothing.
        assert!(
            relocated != parked,
            "{}: turning the comparison's payload off changed no pixel — the document is \
             either being relocated on both frames or on neither",
            world.name
        );
        graded += 1;
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert_eq!(
        graded,
        crate::theme::THEMES.len(),
        "every world must be graded"
    );
}

/// LAW 5 — A TRANSCRIPT WITH NOWHERE TO BE IS DRAWN NOWHERE.
///
/// The substitution and the region are two facts, and the narrow stage separates
/// them: with the timeline focused, the comparison is off screen while the pushed
/// text is still the transcript. Left alone the document layer falls back to the
/// PAGE column and draws it there — behind a workspace that is not showing a
/// comparison at all — and, on a blur-eligible world, frosts it into the backdrop
/// as well, which on a bare-plates world is the most prominent thing on screen.
///
/// The oracle is a differential in the transcript's own PROSE, so every differing
/// pixel is transcript ink and the card, the rows and the ground all cancel. The
/// claim is that there are none: not "the viewport is `None`", which the fallback
/// would satisfy while drawing.
#[test]
fn a_parked_transcript_reaches_no_pixel_of_the_frame() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_parked_transcript_reaches_no_pixel: no adapter");
        return;
    };
    // A canvas narrow enough to STAGE the two regions: the timeline holds focus,
    // so the comparison is not on screen and the transcript is parked.
    let (cw, ch) = (620u32, 720u32);
    let mut graded = 0usize;
    let mut blur_worlds = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        p.set_size(cw as f32, ch as f32);
        let arm = |body: &str| {
            let mut v = timeline_view(6);
            v.text = format!("# {body}\n\n{body} {body} {body}\n\n{body} again\n");
            v.is_markdown = true;
            v
        };
        p.set_view(&arm("Paragraph"));
        p.prepare(&device, &queue, cw, ch).unwrap();
        if p.comparison_viewport().is_some() {
            // This world's metrics still fit both regions here; the parked state
            // is the one under test, so skip rather than grade the wrong thing.
            continue;
        }
        assert!(
            p.document_is_a_transcript(),
            "{}: precondition — the substitution must be in force",
            world.name
        );
        let a = super::pixeldiff::render_frame(&mut p, &device, &queue, cw, ch);
        if p.backdrop_blur() {
            blur_worlds += 1;
        }
        p.set_view(&arm("Zzzzzzzzz"));
        p.prepare(&device, &queue, cw, ch).unwrap();
        let b = super::pixeldiff::render_frame(&mut p, &device, &queue, cw, ch);
        let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
        assert_eq!(
            differing, 0,
            "{}: {differing} pixels changed when the PARKED transcript's prose changed. \
             A transcript whose region is off screen must not be drawn — not at the page \
             column it falls back to, and not into the offscreen capture the blur frosts",
            world.name
        );
        graded += 1;
    }
    p.set_size(1200.0, 800.0);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    assert!(
        graded >= crate::theme::THEMES.len() / 2,
        "the staged geometry must be reached on most of the roster, got {graded}"
    );
    // NON-VACUITY on the axis that hides: the frosted-backdrop half only means
    // something if blur-eligible worlds are actually among the graded cells.
    assert!(
        blur_worlds >= 2,
        "the frosted-backdrop arm must see blur-eligible worlds, got {blur_worlds}"
    );
}
