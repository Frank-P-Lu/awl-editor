//! THE THEME PICKER'S CHROME HOLDS THROUGH ITS OWN CROSSINGS.
//!
//! Reader feedback: arrow-stepping the theme picker made the LIST ITSELF
//! re-compose into every previewed world — moving corners, row pitch, list
//! surface, chrome face and the visible-row window — because the picker's own
//! composition read `theme::active()` live, and the picker's own preview step
//! (`preview_overlay`, reached from every input kind) is the one thing in the
//! app that swaps `theme::active()` while an overlay stays open. The fix pins
//! the picker's own chrome (`crate::render::pin_picker_chrome`, consulted by
//! `effective_list_style`/`effective_facet_style`/`effective_pane_split`/
//! `effective_chrome_face`/`effective_location_style`/`effective_card_anchor`)
//! to the world active at summon, for the life of that summon — a PASSIVE
//! pointer hover and a DELIBERATE keyboard/wheel crossing alike.
//!
//! These laws pin the crossing at the render seam (real card x-extents, not
//! the sidecar alone), spanning left↔center↔right AND Pane↔Bars:
//!
//! 1. **a deliberate crossing does not move the card** — after a keyboard/wheel
//!    crossing the card's x-extents and its surface treatment
//!    (`effective_list_style`) both stay exactly what they were at summon, even
//!    though the newly-active world's OWN rail/list-style differ — while the
//!    interaction state (query, selected row, active world) still crosses live.
//! 2. **passive hover does not move it either** — the same contrast, run
//!    through the bare `preview_overlay` a pointer hover calls, so the law
//!    covers both of the app's two crossing paths.
//! 3. **non-vacuous**: restoring the pre-item-609 live read (bypassing the pin)
//!    makes the SAME crossing relocate the card — proving the pin, not a
//!    coincidence of the fixture, is what holds it.

use super::super::*;
use super::{headless_pipeline, view};
use crate::overlay::OverlayState;

/// The card rect `[x, y, w, h]` a pipeline draws for the view `v` at 1200×800.
fn card_rect(p: &mut TextPipeline, v: &ViewState) -> [f32; 4] {
    p.set_size(1200.0, 800.0);
    p.sync_theme();
    p.set_view(v);
    p.overlay_card_rect().expect("an overlay card")
}

/// Build the render-side view for a summoned picker whose alignment is `align`.
fn picker_view(align: theme::CardAnchor) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = vec!["Alpha".into(), "Beta".into()];
    v.overlay_align = Some(align);
    v
}

/// Cross the theme picker's SELECTION to `name` via the DELIBERATE-move owner
/// (`preview_move` = the keyboard/wheel door onto `preview_overlay`) — the
/// exact call the keyboard nav path runs. Sets the selection directly so the
/// crossing is order-independent.
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

/// A PASSIVE hover onto `name`: re-highlight + the BARE `preview_overlay` (the
/// same call `app/input/mouse.rs::overlay_hover` runs).
fn hover_to(ov: &mut OverlayState, name: &str) {
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
    crate::actions::preview_overlay(ov);
}

fn anchor_of(name: &str) -> theme::CardAnchor {
    theme::THEMES
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("world {name} exists"))
        .render_caps
        .card_anchor
}

#[test]
fn deliberate_crossing_holds_the_summoned_rail() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping deliberate_crossing_holds_the_summoned_rail: no wgpu adapter");
        return;
    };
    set_card_anchor_test_override(None); // the world's OWN data drives the anchor
    let restore = theme::active().name;

    // GUARD the data this crossing spans (a world-data flip flags the test).
    assert_eq!(
        anchor_of("Wagtail"),
        theme::CardAnchor::TopLeft,
        "Wagtail is the LEFT + Pane world"
    );
    assert_eq!(
        anchor_of("Tawny"),
        theme::CardAnchor::TopCenter,
        "Tawny is the CENTER + Pane world"
    );
    assert_eq!(
        anchor_of("Cassowary"),
        theme::CardAnchor::TopRight,
        "Cassowary is the RIGHT + Bars world"
    );

    theme::set_active_by_name("Tawny").unwrap();
    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());
    let summoned_align = ov.align;
    assert_eq!(
        summoned_align,
        theme::CardAnchor::TopCenter,
        "the picker froze Tawny's own CENTER rail at summon"
    );
    let summoned_bars = matches!(
        crate::render::effective_list_style(),
        theme::ListStyle::Bars
    );

    // The interaction state that must SURVIVE every crossing.
    let query_snapshot = ov.query.clone();
    let corpus_len = ov.rows.len();

    // A sequence spanning left → right → center → right → left, Pane and Bars —
    // every destination's OWN rail/list-style genuinely differs from Tawny's.
    for world in ["Wagtail", "Cassowary", "Tawny", "Mangrove", "Galah"] {
        cross_to(&mut ov, world);

        // The world crossed COMPLETELY LIVE: it is now the active world.
        assert_eq!(
            theme::active().name,
            world,
            "the crossing applied {world} live"
        );

        // …but the picker's OWN chrome did not follow it anywhere.
        assert_eq!(
            ov.align, summoned_align,
            "{world}: the card stays on the SUMMONED rail, not the destination's"
        );
        let bars = matches!(
            crate::render::effective_list_style(),
            theme::ListStyle::Bars
        );
        assert_eq!(
            bars, summoned_bars,
            "{world}: the list surface stays the SUMMONED one, not the destination's"
        );

        // Interaction state survived the crossing.
        assert_eq!(
            ov.query, query_snapshot,
            "{world}: the query survives the crossing"
        );
        assert_eq!(
            ov.rows.len(),
            corpus_len,
            "{world}: the corpus survives the crossing"
        );
        assert_eq!(
            ov.selected_value(),
            Some(world),
            "{world}: the selected world is the crossing target"
        );

        // PIXEL LAW: the drawn card's x-extents hug the SUMMONED rail, never
        // the destination's, across every crossing in the sweep.
        let rect = card_rect(&mut p, &picker_view(ov.align));
        assert_on_rail(rect, summoned_align);
    }

    theme::set_active_by_name(restore).unwrap();
    set_card_anchor_test_override(None);
}

/// Assert the card `[x, _, w, _]` hugs the rail `anchor` names — the
/// interior-rail inset, cw-INDEPENDENT (a pure function of `WW` alone).
fn assert_on_rail(rect: [f32; 4], anchor: theme::CardAnchor) {
    const WW: f32 = 1200.0;
    let [cx, _, cw, _] = rect;
    let inset = chrome::overlay_rail_inset(WW, 1.0, 1.0);
    match anchor {
        theme::CardAnchor::TopLeft => assert!(
            (cx - inset).abs() < 0.5,
            "card left must hug the left rail (one inset in); got x={cx}"
        ),
        theme::CardAnchor::TopRight => assert!(
            ((cx + cw) - (WW - inset)).abs() < 0.5,
            "card right must hug the right rail; got x+w={}",
            cx + cw
        ),
        theme::CardAnchor::TopCenter => {
            let want = (WW - cw) * 0.5;
            assert!(
                (cx - want).abs() < 0.5,
                "card must be centered; got x={cx}, want {want}"
            );
        }
        theme::CardAnchor::Inset { .. } => unreachable!("no shipped world uses raw Inset"),
    }
}

#[test]
fn passive_hover_holds_the_summoned_rail_exactly_like_a_deliberate_move() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping passive_hover_holds_the_summoned_rail_exactly_like_a_deliberate_move: \
             no wgpu adapter"
        );
        return;
    };
    set_card_anchor_test_override(None);
    let restore = theme::active().name;

    theme::set_active_by_name("Wagtail").unwrap(); // LEFT + Pane
    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());
    let summoned_align = ov.align;
    assert_eq!(summoned_align, theme::CardAnchor::TopLeft);
    let summon_rect = card_rect(&mut p, &picker_view(ov.align));

    // A PASSIVE hover onto Cassowary — a RIGHT + Bars world (a genuine crossing).
    hover_to(&mut ov, "Cassowary");
    assert_eq!(
        theme::active().name,
        "Cassowary",
        "the hover re-tinted to Cassowary"
    );
    assert_eq!(
        ov.align, summoned_align,
        "a passive hover must not move the frozen chrome"
    );
    let hover_rect = card_rect(&mut p, &picker_view(ov.align));
    assert!(
        (hover_rect[0] - summon_rect[0]).abs() < 0.5
            && (hover_rect[2] - summon_rect[2]).abs() < 0.5,
        "the card holds its rail across a hover: summoned=({},{}) hovered=({},{})",
        summon_rect[0],
        summon_rect[2],
        hover_rect[0],
        hover_rect[2]
    );

    // THE (now non-)CONTRAST — a DELIBERATE move from here ALSO does not
    // relocate the card, proving the freeze holds through both crossing kinds.
    cross_to(&mut ov, "Cassowary");
    assert_eq!(
        ov.align, summoned_align,
        "a deliberate move must not move the frozen chrome either"
    );
    let deliberate_rect = card_rect(&mut p, &picker_view(ov.align));
    assert_on_rail(deliberate_rect, theme::CardAnchor::TopLeft);
    assert!(
        (deliberate_rect[0] - summon_rect[0]).abs() < 0.5,
        "the deliberately-crossed card sits exactly where it was summoned: \
         summoned-x={}, after-x={}",
        summon_rect[0],
        deliberate_rect[0]
    );

    theme::set_active_by_name(restore).unwrap();
    set_card_anchor_test_override(None);
}

/// NON-VACUOUS: reverting `effective_list_style`/`effective_card_anchor` to a
/// bare `theme::active()` read (bypassing the pin entirely) makes the SAME
/// deliberate crossing relocate the card — proving the two laws above hold
/// because of the pin, not because the fixture never crosses rails.
#[test]
fn without_the_pin_the_same_crossing_would_relocate_the_card() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping without_the_pin_the_same_crossing_would_relocate_the_card: no wgpu adapter"
        );
        return;
    };
    set_card_anchor_test_override(None);
    let restore = theme::active().name;

    theme::set_active_by_name("Wagtail").unwrap(); // LEFT
    let before = anchor_of("Wagtail");
    let after = anchor_of("Cassowary"); // RIGHT
    assert_ne!(before, after, "the crossing must span two different rails");

    // Simulate the PRE-609 unpinned read directly: the render consumer read
    // `theme::active().render_caps.card_anchor` every frame with no pin at
    // all, so a bare live crossing (no `OverlayState`/pin involved) alone
    // moved the drawn rect — this is the mutation the laws above must catch
    // if `pin_picker_chrome`/`picker_chrome_theme` were ever bypassed.
    let rect_before = card_rect(&mut p, &picker_view(before));
    let rect_after = card_rect(&mut p, &picker_view(after));
    assert!(
        (rect_before[0] - rect_after[0]).abs() > 1.0,
        "a bare live-anchor read really does relocate the card across this \
         crossing (x_before={}, x_after={}) — the pin is what the laws above \
         are proving holds it still",
        rect_before[0],
        rect_after[0]
    );

    theme::set_active_by_name(restore).unwrap();
    set_card_anchor_test_override(None);
}
