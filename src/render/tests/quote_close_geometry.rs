//! THE CLOSING PULL-QUOTE MARK'S GEOMETRY — unit-level, at the purest reachable
//! seam (`TextPipeline::quote_mark_geometry_for_test`), independent of a GPU
//! render. The opening mark ("66") is untouched by this item and keeps its own
//! coverage; these laws are about the CLOSING mark ("99") alone: its x follows
//! the block's own widest ink edge, its y never rises above that row's own top,
//! and a wide-wrap block yields the mark inside the column instead of letting
//! it escape past the text edge.

use super::{headless_dqp, headless_pipeline, view};
use crate::render::rects::QuoteSide;

fn close_mark(p: &mut crate::render::TextPipeline) -> (f32, f32, f32) {
    let marks = p.quote_mark_geometry_for_test();
    let (top, left, width, _) = *marks
        .iter()
        .find(|&&(_, _, _, side)| side == QuoteSide::Close)
        .expect("a closing mark");
    (top, left, width)
}

/// NARROW WRAP: an ordinary short one-line quote, nowhere near the column's own
/// edge. The closing mark's x must sit exactly one gap past the row's own
/// ink-right — read from the SAME real shaped-row geometry the production path
/// reads (`quote_close_row_end_x`), not re-derived from the mark's own
/// placement — and its top must never rise above the row's own top.
#[test]
fn close_mark_follows_ink_at_narrow_wrap() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping close_mark_follows_ink_at_narrow_wrap: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);
    let mut v = view("> a short quote\n\ntail\n", 2, 0);
    v.is_markdown = true;
    p.set_view(&v);

    let ink_right = p.quote_close_row_end_x(0);
    let row_top = p.doc_top() + p.visual_rows(0).last().expect("rows").line_top;
    let (top, left, _width) = close_mark(&mut p);

    let gap = p.metrics.font_size * 0.5;
    assert!(
        (left - (ink_right + gap)).abs() < 1.0,
        "an unclamped narrow row must sit one (roughly half-em) gap past its own \
         ink: x {left}, ink_right {ink_right}, expected ~{}",
        ink_right + gap
    );
    assert!(
        top >= row_top - 0.5,
        "the closing mark's box rose above its own row's top: {top} < {row_top} \
         — it will read as belonging to the row above"
    );
    crate::page::set_page_on(was_page);
}

/// WIDE WRAP: a blockquote body long enough that its last visual row's own ink
/// runs up against the column's own wrap edge. The clamp must still yield the
/// mark INSIDE the column — never past `text_right` — rather than escaping the
/// page.
///
/// A single arbitrarily-chosen repeat count is not enough here: greedy word
/// wrap fills every row BUT the last as full as it can, while the LAST row's
/// own fill is whatever happens to be left over — for one repeat count that
/// leftover can sit far short of `text_right`, leaving the clamp's `.min(...)`
/// arm never actually taken and this law "passing" without ever exercising
/// the branch it names. So this sweeps a fine-grained filler over many total
/// word counts and keeps the configuration whose last row comes CLOSEST to
/// the column's own right edge — the true empirical worst case — and proves
/// the clamp genuinely bound by requiring the mark's own right edge to land
/// (not just stay under) at that edge.
#[test]
fn close_mark_clamps_inside_the_column_at_the_widest_wrap() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping close_mark_clamps_inside_the_column_at_the_widest_wrap: no wgpu adapter"
        );
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);

    let text_right = p.column_left() + p.column_width();
    let wrap_right = p.text_left() + p.text_wrap_width();
    let mut best_ink_right = f32::MIN;
    let mut best_right_edge = f32::MIN;
    for reps in 40..=160 {
        // A short two-letter unit gives fine-grained control over how full the
        // final row lands, unlike a long multi-word phrase whose leftover
        // jumps in bigger, coarser steps.
        let long = format!("> {}\n\ntail\n", "wq ".repeat(reps));
        let mut v = view(&long, 2, 0);
        v.is_markdown = true;
        p.set_view(&v);
        if p.visual_rows(0).len() < 2 {
            continue; // not wrapped yet at this length
        }
        let ink_right = p.quote_close_row_end_x(0);
        best_ink_right = best_ink_right.max(ink_right);
        let (_top, left, width) = close_mark(&mut p);
        assert!(
            left + width <= text_right + 0.5,
            "the closing mark escaped the column at reps={reps}: right edge {} \
             past text_right {text_right}",
            left + width
        );
        best_right_edge = best_right_edge.max(left + width);
    }
    assert!(
        best_ink_right > wrap_right - 40.0,
        "the sweep never brought the block's last row within 40px of the \
         column's own right edge (closest {best_ink_right} vs {text_right}) \
         — this law needs a configuration that actually reaches the clamp \
         boundary, not just headroom under it"
    );
    assert!(
        best_right_edge > wrap_right && best_right_edge <= text_right + 0.5,
        "the sweep's tightest configuration never pinned the mark's own right \
         edge to the column's right edge ({best_right_edge} vs {text_right}) \
         — the clamp's `.min(...)` arm may never actually bind"
    );
    crate::page::set_page_on(was_page);
}

/// THE ANCHOR LAW (fix for the reported defect: "on the multi-line case the 99
/// rode above the row and read as belonging to the row above"). This is the
/// one case that separates "anchored to the last VISUAL row" from "anchored to
/// the first" or from the mark's own floor alone: a SINGLE logical line, long
/// enough to soft-wrap into several visual rows, so the block's last logical
/// line has its own first row well above its own last row. A regression that
/// reads the wrong row's baseline (but keeps the correct floor, which only
/// pins the mark to the last row's TOP, not its baseline) still passes at the
/// floor while landing visibly higher than intended — this law catches that by
/// requiring the resolved top to sit inside the LAST row's own band, clearly
/// separated from the first row's.
#[test]
fn close_mark_anchors_to_the_last_wrapped_row_not_the_first() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping close_mark_anchors_to_the_last_wrapped_row_not_the_first: no wgpu adapter"
        );
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);
    let long = format!("> {}\n\ntail\n", "wrapping quote body ".repeat(40));
    let mut v = view(&long, 2, 0);
    v.is_markdown = true;
    p.set_view(&v);
    let rows = p.visual_rows(0);
    assert!(
        rows.len() > 1,
        "fixture failed to soft-wrap: {} visual row(s) — this law needs a real \
         multi-row SINGLE logical line",
        rows.len()
    );
    let first_row_top = p.doc_top() + rows.first().expect("rows").line_top;
    let last_row_top = p.doc_top() + rows.last().expect("rows").line_top;
    let last_row_height = rows.last().expect("rows").line_height;
    assert!(
        last_row_top > first_row_top + 0.5,
        "fixture's first and last rows coincide: {first_row_top} == {last_row_top}"
    );

    let (top, _left, _width) = close_mark(&mut p);
    assert!(
        top >= last_row_top - 0.5,
        "the closing mark's box rose above its own (last) row's top: {top} < \
         {last_row_top}"
    );
    assert!(
        top <= last_row_top + last_row_height + 0.5,
        "the closing mark's box fell below its own row's band: {top} > \
         {last_row_top} + {last_row_height}"
    );
    assert!(
        top - first_row_top > last_row_height * 0.5,
        "the closing mark sits within half a row-height of the FIRST wrapped \
         row's top ({top} vs first row top {first_row_top}) — it reads as \
         belonging to the row above rather than to its own last row \
         ({last_row_top})"
    );
    crate::page::set_page_on(was_page);
}

#[test]
fn closing_quote_uses_its_own_block_edge_and_final_row_at_every_zoom() {
    let _guard = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        return;
    };
    crate::page::set_page_on(true);
    crate::markdown::set_wysiwyg_on(true);
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        for zoom in [0.5, 1.0, 2.0] {
            for document in [
                concat!(
                    "> A long opening sentence that occupies the quote block.\n",
                    "> dog.\n\ntail\n"
                ),
                "> Short.\n\ntail\n",
                concat!(
                    "> One long logical line wrapping through the block ",
                    "with a much shorter final word dog.\n\ntail\n"
                ),
                concat!(
                    "> A long opening sentence that occupies the quote block.\n",
                    "> dog.\n\n> Short block.\n\ntail\n"
                ),
            ] {
                let tail = document.lines().count() - 1;
                let mut v = view(document, tail, 0);
                v.is_markdown = true;
                v.zoom = zoom;
                p.set_view(&v);
                p.prepare(&device, &queue, 1200, 800)
                    .expect("prepared quote frame");
                let report = p.layout_report().expect("shaped layout");
                let last = if document.starts_with("> A long") {
                    1
                } else {
                    0
                };
                let edge = report
                    .rows
                    .iter()
                    .filter(|row| row.logical_line <= last)
                    .flat_map(|row| row.xs.iter().copied())
                    .fold(0.0_f32, f32::max);
                let last_top = p.doc_top() + p.visual_rows(last).last().unwrap().line_top;
                let (top, left, width) = close_mark(&mut p);
                assert!(
                    left >= edge - 0.5,
                    "closing mark overlaps owning block: {left} < {edge} @ {zoom}"
                );
                assert!(left + width <= p.column_left() + p.column_width() + 0.5);
                assert!(top >= last_top - 0.5);
                if document.contains("> Short block.") {
                    let marks = p.quote_mark_geometry_for_test();
                    let closes: Vec<_> = marks
                        .iter()
                        .filter(|mark| mark.3 == QuoteSide::Close)
                        .collect();
                    let opens: Vec<_> = marks
                        .iter()
                        .filter(|mark| mark.3 == QuoteSide::Open)
                        .collect();
                    assert_eq!(closes.len(), 2);
                    assert_eq!(opens.len(), 2);
                    assert!(closes[0].1 > closes[1].1, "each quote owns its block width");
                    assert_eq!(
                        opens[0].1, opens[1].1,
                        "opening marks retain their shared left gutter"
                    );
                }
                assert_eq!(
                    p.layout_report().expect("shaped layout").rows.len(),
                    report.rows.len(),
                    "ornament added a text row"
                );
                if last == 1 {
                    let last_edge = report
                        .rows
                        .iter()
                        .filter(|row| row.logical_line == last)
                        .flat_map(|row| row.xs.iter().copied())
                        .fold(0.0_f32, f32::max);
                    assert!(
                        left > last_edge + p.metrics.font_size,
                        "mark still follows short final word"
                    );
                }
            }
        }
    }
}
