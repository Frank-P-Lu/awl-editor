//! THE THEME-PICKER WORLD-JUMP HAZARD IS REAL, against the actual
//! render pipeline (not merely hypothesized): a keyboard-driven crossing
//! (`preview_move`, the same call `actions/overlay_nav.rs`'s Up/Down arms run)
//! RE-ANCHORS the card to the destination world's own rail — a fixed
//! physical pixel that hit-tests to a candidate row before the crossing can
//! hit-test to NOTHING (off the relocated card) after it, with the pointer
//! never having traveled a pixel. This is the geometry half of the item-85 law;
//! the gate itself (`OverlayState::hover_at`, composed with whatever hit-test
//! the pipeline returns) is proven separately and purely in
//! `overlay::tests::hover_at_gates_on_real_pointer_motion_not_a_relayout_hit_test_change`
//! — including the STRONGER "cascades onto a genuinely different row" form of
//! the hazard, which that pure test can construct directly via an injected hit
//! (this pipeline's row-Y formula is anchor-independent, so a same-rail
//! crossing never moves the row math — only a rail CROSSING does, and that
//! crossing's dominant, always-reproducible effect is relocating the card's
//! X-range, i.e. moving a pixel from "on some row" to "on no row"). This file
//! exists to show that pure test isn't fighting a strawman: real
//! `TextPipeline::overlay_row_at` geometry really does move under a stationary
//! pixel across a real, deliberate world crossing.

use super::super::*;
use super::{headless_pipeline, view};
use crate::overlay::OverlayState;

/// Cross the theme picker's SELECTION to `name` via the DELIBERATE-move owner
/// (`preview_move` = preview + re-anchor) — the exact call the keyboard nav path
/// runs. Mirrors `reanchor_crossing_law::cross_to` (duplicated locally: that
/// helper is private to its own test module).
fn cross_to(ov: &mut OverlayState, name: &str) {
    let ci = ov
        .rows
        .iter()
        .position(|r| r.accept == name)
        .expect("world in corpus");
    let pos = ov
        .items
        .iter()
        .position(|&i| i == ci)
        .expect("world visible on the flat lens");
    ov.selected = pos;
    crate::actions::preview_move(ov);
}

/// A FLAT (non-faceted) picker view at `ov`'s current alignment — no
/// `overlay_lens`, so this renders through the plain `overlay_geometry` path
/// (never `theme_overlay_geometry`), the same minimal shape
/// `reanchor_crossing_law::picker_view` uses for its own card-rect law.
fn picker_view(ov: &OverlayState) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = (0..8).map(|i| format!("Row {i}")).collect();
    v.overlay_selected = 0;
    v.overlay_align = Some(ov.align);
    v
}

const WW: f32 = 1200.0;
const WH: f32 = 800.0;

#[test]
fn a_deliberate_world_crossing_can_move_a_stationary_pixels_hit_test_row() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_deliberate_world_crossing_can_move_a_stationary_pixels_hit_test_row: no wgpu adapter"
        );
        return;
    };
    crate::render::set_card_anchor_test_override(None); // each world's OWN data
    let restore = theme::active().name;

    // GUARD the world-data premise: Wagtail is the LEFT rail world, Cassowary the
    // RIGHT — the widest possible rail crossing, so a
    // stationary pixel over Wagtail's card is guaranteed to fall well outside
    // Cassowary's on a 1200px canvas.
    let anchor_of = |name: &str| {
        theme::THEMES
            .iter()
            .find(|t| t.name == name)
            .unwrap()
            .render_caps
            .card_anchor
    };
    assert_eq!(anchor_of("Wagtail"), theme::CardAnchor::TopLeft);
    assert_eq!(anchor_of("Cassowary"), theme::CardAnchor::TopRight);

    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());

    p.set_size(WW, WH);

    cross_to(&mut ov, "Wagtail");
    assert_eq!(
        ov.align,
        theme::CardAnchor::TopLeft,
        "keyboard nav re-anchored to Wagtail's rail"
    );
    p.sync_theme();
    let v1 = picker_view(&ov);
    p.set_view(&v1);
    let [cx1, cy1, cw1, ch1] = p.overlay_card_rect().expect("Wagtail card");
    // A candidate row's midpoint, well inside Wagtail's own left-hugging card.
    let (px, py) = (cx1 + cw1 * 0.5, cy1 + ch1 * 0.5);
    let hit_before = p.overlay_row_at(px, py);
    assert!(
        hit_before.is_some(),
        "the probe pixel must start ON a real candidate row"
    );

    // THE WORLD JUMP: a deliberate keyboard crossing to Cassowary — the picker's
    // own re-layout, exactly `actions::overlay_nav::preview_move` (Down/Up)
    // applies. The pointer's PHYSICAL position (px, py) never moves.
    cross_to(&mut ov, "Cassowary");
    assert_eq!(
        ov.align,
        theme::CardAnchor::TopRight,
        "the crossing re-anchored to Cassowary's rail"
    );
    p.sync_theme();
    let v2 = picker_view(&ov);
    p.set_view(&v2);
    let hit_after = p.overlay_row_at(px, py);

    // THE HAZARD, proven against real geometry: the SAME stationary pixel that
    // hit a real row before the crossing hits NOTHING after it — the card moved
    // clean out from under the unmoved pointer. This is exactly the class of
    // event `OverlayState::hover_at` must refuse to read as "the user hovered
    // row X" (there is no row X here at all anymore) — its stronger sibling,
    // cascading onto a genuinely DIFFERENT real row, is proven directly (with an
    // injected hit, since this pipeline's row-Y formula is rail-independent) in
    // `overlay::tests::hover_at_gates_on_real_pointer_motion_not_a_relayout_hit_test_change`.
    assert_ne!(
        hit_before, hit_after,
        "a deliberate rail crossing must move what a stationary pixel hits — \
         the geometry hazard item 85's real-motion gate guards against"
    );
    assert_eq!(
        hit_after, None,
        "Wagtail's row midpoint sits outside Cassowary's right-hugging card"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}
