//! THE THEME-PICKER WORLD-JUMP HOVER HAZARD IS RETIRED, against the actual
//! render pipeline (not merely hypothesized): a deliberate keyboard-driven
//! crossing (`preview_move`, the same call `actions/overlay_nav.rs`'s Up/Down
//! arms run) used to RE-ANCHOR the card to the destination world's own
//! rail — a fixed physical pixel could hit-test to a candidate row before the
//! crossing and to NOTHING (off the relocated card) after it, with the
//! pointer never having traveled a pixel. Since the picker's own chrome is now
//! PINNED for the life of its summon (`crate::render::pin_picker_chrome`),
//! that specific hazard cannot occur any more: a world crossing changes the
//! preview behind the card, never the card's own geometry.
//!
//! This file proves the retirement against real geometry — the same
//! `TextPipeline::overlay_row_at` seam the old hazard test drove — rather
//! than merely asserting the pin exists. The GENERIC hover-gate law (`hover_at`
//! must not synthesize a selection change from a relayout under a stationary
//! pointer, whatever causes the relayout) is proven separately and purely in
//! `overlay::tests::hover_at_gates_on_real_pointer_motion_not_a_relayout_hit_test_change`,
//! and stays valid for causes other than a theme-picker world crossing (a
//! window resize, say) — this file's own claim is narrower and stronger: for
//! the theme picker specifically, the crossing this file drives no longer
//! relocates anything for that gate to have to guard against.

use super::super::*;
use super::{headless_pipeline, view};
use crate::overlay::OverlayState;

/// Cross the theme picker's SELECTION to `name` via the DELIBERATE-move owner
/// (`preview_move`) — the exact call the keyboard nav path runs.
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
fn a_deliberate_world_crossing_no_longer_moves_a_stationary_pixels_hit_test_row() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_deliberate_world_crossing_no_longer_moves_a_stationary_pixels_hit_test_row: \
             no wgpu adapter"
        );
        return;
    };
    crate::render::set_card_anchor_test_override(None); // each world's OWN data
    let restore = theme::active().name;

    // GUARD the world-data premise: Wagtail is the LEFT rail world, Cassowary the
    // RIGHT — the widest possible rail crossing on a 1200px canvas, so a
    // stationary pixel over Wagtail's card would (pre-609) fall well outside
    // Cassowary's card once the crossing re-anchored it.
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

    theme::set_active_by_name("Wagtail").unwrap();
    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());

    p.set_size(WW, WH);

    cross_to(&mut ov, "Wagtail");
    assert_eq!(
        ov.align,
        theme::CardAnchor::TopLeft,
        "the picker summoned (and stays) on Wagtail's rail"
    );
    p.sync_theme();
    let v1 = picker_view(&ov);
    p.set_view(&v1);
    let [cx1, cy1, cw1, ch1] = p.overlay_card_rect().expect("Wagtail card");
    // A candidate row's midpoint, well inside the summoned card.
    let (px, py) = (cx1 + cw1 * 0.5, cy1 + ch1 * 0.5);
    let hit_before = p.overlay_row_at(px, py);
    assert!(
        hit_before.is_some(),
        "the probe pixel must start ON a real candidate row"
    );

    // THE WORLD JUMP: a deliberate keyboard crossing to Cassowary — exactly
    // `actions::overlay_nav::preview_move` (Down/Up) applies. The pointer's
    // PHYSICAL position (px, py) never moves.
    cross_to(&mut ov, "Cassowary");
    assert_eq!(
        theme::active().name,
        "Cassowary",
        "the crossing applied Cassowary live"
    );
    assert_eq!(
        ov.align,
        theme::CardAnchor::TopLeft,
        "the card's own chrome stays pinned to the SUMMONED (Wagtail) rail, \
         not Cassowary's, across the crossing"
    );
    p.sync_theme();
    let v2 = picker_view(&ov);
    p.set_view(&v2);
    let hit_after = p.overlay_row_at(px, py);

    // THE RETIREMENT, proven against real geometry: the SAME stationary pixel
    // hits the SAME row before and after the crossing — the card never moved,
    // so there is no hit-test hazard left for `OverlayState::hover_at` to have
    // to guard against on this path.
    assert_eq!(
        hit_before, hit_after,
        "a deliberate rail crossing during a theme-picker preview must NOT move \
         what a stationary pixel hits, now that the picker's own chrome is pinned"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}

/// NON-VACUOUS: reverting to the pre-609 unpinned read (a bare
/// `theme::active().render_caps.card_anchor`, ignoring the summon-time pin)
/// makes the SAME crossing move the stationary pixel's hit-test row —
/// proving the test above is exercising the pin, not a fixture that never
/// really crosses rails.
#[test]
fn without_the_pin_the_same_crossing_would_move_the_hit_test_row() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping without_the_pin_the_same_crossing_would_move_the_hit_test_row: \
             no wgpu adapter"
        );
        return;
    };
    crate::render::set_card_anchor_test_override(None);
    let restore = theme::active().name;

    p.set_size(WW, WH);

    // Simulate the pre-609 unpinned render read directly: build the SAME flat
    // picker view at each world's OWN live rail (no `OverlayState`/pin involved
    // at all — this is exactly what every render consumer read before the pin
    // existed) and show the stationary pixel's hit changes across the crossing.
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = (0..8).map(|i| format!("Row {i}")).collect();
    v.overlay_selected = 0;

    v.overlay_align = Some(theme::CardAnchor::TopLeft); // Wagtail's own rail
    p.sync_theme();
    p.set_view(&v);
    let [cx1, cy1, cw1, ch1] = p.overlay_card_rect().expect("left card");
    let (px, py) = (cx1 + cw1 * 0.5, cy1 + ch1 * 0.5);
    let hit_before = p.overlay_row_at(px, py);
    assert!(hit_before.is_some());

    v.overlay_align = Some(theme::CardAnchor::TopRight); // Cassowary's own rail
    p.set_view(&v);
    let hit_after = p.overlay_row_at(px, py);

    assert_ne!(
        hit_before, hit_after,
        "an unpinned rail crossing really does move what a stationary pixel \
         hits (hit_before={hit_before:?}, hit_after={hit_after:?}) — the pin \
         is what the test above is proving holds it still"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}
