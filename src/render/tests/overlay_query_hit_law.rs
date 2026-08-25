//! THE QUERY FIELD'S OWN CLICK-TO-PLACE LAW (DESIGN.md §8: "a surface that
//! looks clickable must be clickable where it is drawn").
//! `TextPipeline::overlay_query_char_at` is the pointer's door into the query
//! field; this file proves it is the exact geometric INVERSE of
//! `overlay_query_caret_box` (the door the CARET draws through): a probe at
//! any caret's own drawn box resolves back to that same char index, over the
//! query roster, a title-prefixed card, and both DPIs.
//!
//! MUTATION PROOF: neuter `overlay_query_char_at` to return `None`
//! unconditionally — an unconditionally swallowed click, off a row, with no
//! caret placement — and every round-trip assertion below goes red —
//! `Some(caret)` can never equal `None`.

use super::super::*;
use super::{headless_dqp, view};

fn query_view(text: &str, query: &str, title: &'static str) -> ViewState {
    let mut v = view(text, 0, 0);
    v.overlay_active = true;
    v.overlay_title = title.to_string();
    v.overlay_items = vec!["row one".into(), "row two".into(), "row three".into()];
    v.overlay_selected = 0;
    v.overlay_hint = "type to filter".into();
    v.overlay_query = query.to_string();
    v.overlay_query_caret = query.chars().count();
    v
}

/// THE HEADLINE LAW: every char boundary in the query — over a short/empty/
/// accented/CJK roster, an untitled and a titled card, both DPIs — is
/// clickable exactly where its own caret is drawn.
#[test]
fn a_click_at_any_drawn_caret_position_resolves_back_to_that_same_char_index() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping a_click_at_any_drawn_caret_position_resolves_back_to_that_same_char_index: \
             no wgpu adapter"
        );
        return;
    };
    let queries = ["", "a", "hello world", "héllo wörld", "日本語のクエリ"];
    let titles = ["", "go to"];
    let mut graded = 0usize;
    for &q in &queries {
        let len = q.chars().count();
        for &title in &titles {
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                let cw = (1200.0 * dpi).round() as u32;
                let ch = (800.0 * dpi).round() as u32;
                // `window_w`/`window_h` (what `overlay_query_char_at` itself
                // reads to rebuild its own geometry) come from `set_size`, not
                // `prepare` — both must move together at a new DPI or the hit
                // test rebuilds geometry at the STALE canvas while this loop
                // reads boxes from the fresh one.
                p.set_size(cw as f32, ch as f32);
                let v = query_view("hello\n", q, title);
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                for caret in 0..=len {
                    // The shaped run doesn't depend on the caret, only the box
                    // reading it — so re-stamping the field and re-reading the
                    // box is exact without a second `prepare()` per caret.
                    p.overlay_query_caret = caret;
                    let [x, y, w, h] = p
                        .overlay_query_caret_box(&geom, &plan)
                        .expect("an active flat picker with a title draws a query caret box");
                    let mid_y = y + h * 0.5;
                    // Just inside the caret box's own LEFT edge — the caret is
                    // drawn at the char's true boundary, but its box WIDTH
                    // (`m.caret_w`) can be a near-full glyph cell (a block
                    // caret): probing the box's CENTER then straddles the
                    // glyph's own midpoint and the round-to-nearer-half rule
                    // below answers the neighbor instead.
                    let probe_x = x + 0.75_f32.min(w * 0.5);
                    assert_eq!(
                        p.overlay_query_char_at(probe_x, mid_y),
                        Some(caret),
                        "q={q:?} title={title:?} dpi={dpi}: a click at caret {caret}'s own \
                         drawn box ({probe_x:.2},{mid_y:.2}) resolved to a different char index"
                    );
                    graded += 1;
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 50,
        "the sweep must grade many caret positions across the roster, got {graded}"
    );
}

/// THE SAME GATE THE I-BEAM READS: `overlay_query_char_at` must agree with
/// `over_overlay_query` about WHICH pixels are on the field — `Some(_)` here
/// exactly where the I-beam shows there, never a wider or narrower box that
/// could drift the two apart.
#[test]
fn the_hit_test_and_the_i_beam_gate_agree_on_every_probed_pixel() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_hit_test_and_the_i_beam_gate_agree_on_every_probed_pixel: \
             no wgpu adapter"
        );
        return;
    };
    let v = query_view("hello\n", "hello world", "go to");
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);
    let mut graded = 0usize;
    // A grid across the whole card plus a margin, so the sweep crosses every
    // edge of the field's own box in both axes, not just its interior.
    let (card_x0, card_x1) = plan.card_x_span();
    let x0 = card_x0 - 20.0;
    let x1 = card_x1 + 20.0;
    let y0 = geom.card_y - 20.0;
    let y1 = geom.card_y + geom.card_h + 20.0;
    let mut px = x0;
    while px <= x1 {
        let mut py = y0;
        while py <= y1 {
            assert_eq!(
                p.overlay_query_char_at(px, py).is_some(),
                p.over_overlay_query(px, py),
                "({px:.1},{py:.1}): the click hit-test and the I-beam gate disagree about \
                 whether the pointer is on the query field"
            );
            graded += 1;
            py += 7.0;
        }
        px += 7.0;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 300,
        "the grid sweep must probe many pixels, got {graded}"
    );
}

/// A card with no query line (the contextual spell popup) offers nothing to
/// click into — no caret box, no hit.
#[test]
fn a_card_with_no_query_line_has_no_query_hit() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_card_with_no_query_line_has_no_query_hit: no wgpu adapter");
        return;
    };
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_spell = Some((0, 0, 5));
    v.overlay_items = vec!["suggestion".into()];
    v.overlay_selected = 0;
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);
    assert!(
        plan.query_band().is_none(),
        "the contextual spell popup plans no query band"
    );
    let (card_x0, card_x1) = plan.card_x_span();
    for probe_x in [card_x0, (card_x0 + card_x1) * 0.5] {
        for probe_y in [geom.card_y, geom.card_y + geom.card_h * 0.5] {
            assert_eq!(p.overlay_query_char_at(probe_x, probe_y), None);
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
}

/// No summoned overlay at all: nothing to click into.
#[test]
fn no_overlay_open_has_no_query_hit() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping no_overlay_open_has_no_query_hit: no wgpu adapter");
        return;
    };
    let v = view("hello\n", 0, 0);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(p.overlay_query_char_at(600.0, 400.0), None);
    theme::set_active(theme::DEFAULT_THEME);
}
