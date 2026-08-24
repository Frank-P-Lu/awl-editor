//! THE MID-QUERY GATE: `ForwardChar`/`BackwardChar`/`LineStart`(`BufferStart`)/
//! `LineEnd`(`BufferEnd`) carry the list-nav overloads (lens cycle, row jump)
//! while the query caret sits AT REST at its own end, and fall through to the
//! query field's own char motion / Home-End the instant the caret sits
//! anywhere else — see `OverlayState::query_at_rest`'s doc and
//! `actions::overlay_nav::mid_query_motion`. Driven through the real
//! `apply_transition` overlay intercept, so a `--keys` replay reaches the same
//! code these tests do.
//!
//! Every fixture here types a TWO-CHAR query so a caret of 1 is genuinely
//! interior (neither 0 nor the length) — a one-char query has no such
//! position, and an earlier draft of this file silently tested "at rest"
//! twice under the name "mid-query" for exactly that reason.

use super::super::*;
use super::drive;
use crate::overlay::OverlayKind;

/// Fixture names all share `"at"` so a two-char typed query keeps the whole
/// roster non-empty at every step below.
fn goto_overlay_sharing_at() -> crate::overlay::Journey {
    crate::overlay::Journey::seeded(Some(OverlayState::new(
        OverlayKind::Goto,
        vec!["cats.md".into(), "scatter.md".into(), "educate.md".into()],
        vec![],
        vec![],
    )))
}

fn type_at(overlay: &mut crate::overlay::Journey, accept: &mut Option<(OverlayKind, String)>) {
    drive(overlay, accept, &Action::InsertChar('a'));
    drive(overlay, accept, &Action::InsertChar('t'));
}

#[test]
fn typing_always_leaves_the_query_caret_at_rest() {
    let mut overlay = goto_overlay_sharing_at();
    let mut accept = None;
    type_at(&mut overlay, &mut accept);
    let ov = overlay.card().unwrap();
    assert_eq!(ov.query.text(), "at");
    assert_eq!(ov.query.caret(), 2);
    assert!(
        ov.query_at_rest(),
        "ordinary typing always leaves the caret at the query's own end"
    );
}

#[test]
fn forward_and_backward_char_cycle_the_lens_at_rest_but_step_the_caret_mid_query() {
    let mut overlay = goto_overlay_sharing_at();
    let mut accept = None;
    type_at(&mut overlay, &mut accept);
    assert!(overlay.card().unwrap().query_at_rest());
    let lens_at_rest = overlay.card().unwrap().active_facet_id();

    // AT REST: ForwardChar keeps its existing list-nav meaning (lens cycle),
    // unchanged.
    drive(&mut overlay, &mut accept, &Action::ForwardChar);
    let after_rest_step = overlay.card().unwrap().active_facet_id();
    assert_ne!(
        after_rest_step, lens_at_rest,
        "ForwardChar at rest must still cycle the lens"
    );
    assert_eq!(
        overlay.card().unwrap().query.caret(),
        2,
        "a list-nav ForwardChar must not also move the query caret"
    );

    // Simulate a click landing mid-query — genuinely interior: caret 1 of a
    // length-2 query is neither the start nor the end.
    overlay.card_mut().unwrap().query_set_caret(1);
    assert!(!overlay.card().unwrap().query_at_rest());
    let lens_mid_query = overlay.card().unwrap().active_facet_id();

    // MID-QUERY: ForwardChar is now the field's own char motion, not the lens.
    drive(&mut overlay, &mut accept, &Action::ForwardChar);
    let ov = overlay.card().unwrap();
    assert_eq!(
        ov.active_facet_id(),
        lens_mid_query,
        "mid-query ForwardChar must not cycle the lens"
    );
    assert_eq!(
        ov.query.caret(),
        2,
        "mid-query ForwardChar steps the caret one char right"
    );
    assert!(
        ov.query_at_rest(),
        "stepping right from the last interior position lands at the end"
    );

    // Reaching the end again restores the list-nav reading on the VERY NEXT
    // keypress.
    drive(&mut overlay, &mut accept, &Action::ForwardChar);
    assert_ne!(
        overlay.card().unwrap().active_facet_id(),
        lens_mid_query,
        "once the caret is at rest again, ForwardChar resumes cycling the lens"
    );

    // BackwardChar mirrors it: land mid-query (caret 1, still interior),
    // then step back left to 0.
    overlay.card_mut().unwrap().query_set_caret(1);
    assert!(!overlay.card().unwrap().query_at_rest());
    let lens_before_back = overlay.card().unwrap().active_facet_id();
    drive(&mut overlay, &mut accept, &Action::BackwardChar);
    let ov = overlay.card().unwrap();
    assert_eq!(
        ov.active_facet_id(),
        lens_before_back,
        "mid-query BackwardChar must not cycle the lens"
    );
    assert_eq!(
        ov.query.caret(),
        0,
        "BackwardChar steps the caret one char left"
    );
    assert!(
        !ov.query_at_rest(),
        "caret 0 of a length-2 query is not at rest"
    );
}

#[test]
fn line_start_and_end_jump_rows_at_rest_but_move_the_caret_mid_query() {
    let mut overlay = goto_overlay_sharing_at();
    let mut accept = None;
    type_at(&mut overlay, &mut accept);
    let ov = overlay.card().unwrap();
    assert!(ov.query_at_rest());
    assert_eq!(ov.query.caret(), 2);
    assert!(
        ov.items.len() >= 2,
        "the fixture roster must stay non-empty after filtering: {:?}",
        ov.items
    );

    // AT REST: LineEnd/BufferEnd still jump the ROW selection to the last item.
    drive(&mut overlay, &mut accept, &Action::LineEnd);
    let ov = overlay.card().unwrap();
    let last_row = ov.items.len() - 1;
    assert_eq!(
        ov.selected, last_row,
        "LineEnd at rest jumps to the last row"
    );
    assert_eq!(
        ov.query.caret(),
        2,
        "a row jump must not move the query caret"
    );

    // AT REST: LineStart/BufferStart jumps back to the first row.
    drive(&mut overlay, &mut accept, &Action::LineStart);
    let ov = overlay.card().unwrap();
    assert_eq!(ov.selected, 0, "LineStart at rest jumps to the first row");
    assert_eq!(ov.query.caret(), 2);

    // Land the caret mid-query (a real click's caret-side door) — the
    // genuinely interior position, caret 1 of a length-2 query.
    overlay.card_mut().unwrap().query_set_caret(1);
    assert!(!overlay.card().unwrap().query_at_rest());
    let sel_mid_query = overlay.card().unwrap().selected;

    // MID-QUERY: LineEnd is Home/End's TEXT-FIELD half — the caret to the
    // query's own end — never a row jump.
    drive(&mut overlay, &mut accept, &Action::LineEnd);
    let ov = overlay.card().unwrap();
    assert_eq!(
        ov.selected, sel_mid_query,
        "mid-query LineEnd must not jump rows"
    );
    assert_eq!(
        ov.query.caret(),
        2,
        "mid-query LineEnd moves the caret to the query's own end"
    );
    assert!(
        ov.query_at_rest(),
        "the caret is now at the query's own end"
    );

    // Reaching the end restores the list-nav reading on the very next call:
    // LineStart now jumps rows again, matching `query_at_rest`'s doc.
    drive(&mut overlay, &mut accept, &Action::LineStart);
    let ov = overlay.card().unwrap();
    assert_eq!(
        ov.selected, 0,
        "list-nav resumes once the caret is at rest again"
    );
    assert_eq!(
        ov.query.caret(),
        2,
        "a resumed list-nav LineStart must not move the query caret"
    );

    // MID-QUERY LineStart: caret to the query's own start (Home), not row 0
    // again by coincidence — proven by landing the selection somewhere else
    // first.
    drive(&mut overlay, &mut accept, &Action::NextLine);
    let sel_before = overlay.card().unwrap().selected;
    overlay.card_mut().unwrap().query_set_caret(1);
    assert!(!overlay.card().unwrap().query_at_rest());
    drive(&mut overlay, &mut accept, &Action::LineStart);
    let ov = overlay.card().unwrap();
    assert_eq!(
        ov.selected, sel_before,
        "mid-query LineStart must not jump rows"
    );
    assert_eq!(
        ov.query.caret(),
        0,
        "mid-query LineStart moves the caret to char 0"
    );
}

/// MUTATION-PROOF SEAM: `OverlayState::query_set_caret` is the one caret-side
/// door a click (`render::TextPipeline::overlay_query_char_at` in the pointer
/// path) and this file's mid-query simulations both go through. A regression
/// that re-swallows placement — makes it a no-op, so a "click inside the
/// card off a row" leaves the caret exactly where it was — is exactly what
/// turns the laws above red: every `mid-query` assertion reduces to the
/// `at rest` behaviour it is contrasted against, because the caret never
/// actually leaves the end.
#[test]
fn query_set_caret_is_the_one_door_a_click_and_this_files_own_simulations_share() {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["alpha.md".into(), "beta.md".into()],
        vec![],
        vec![],
    );
    ov.push('a');
    ov.push('b');
    assert_eq!(ov.query.caret(), 2);
    ov.query_set_caret(0);
    assert_eq!(
        ov.query.caret(),
        0,
        "query_set_caret must actually move the caret — the click-to-place door"
    );
    assert!(!ov.query_at_rest());
}
