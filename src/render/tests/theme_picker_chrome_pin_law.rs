//! THE HEADLINE LAW: a full arrow sweep of the WHOLE roster holds the theme
//! picker's own card fixed — rect, list style, facet style, pane split, row
//! pitch and the visible-row window all identical frame to frame — while the
//! world behind it (the page ground a viewer actually sees around the card)
//! changes on every step. Enrolment is the roster itself
//! (`theme::THEMES`, all of it, not a named subset) so a future world cannot
//! silently dodge the sweep, and the roster's own non-Pane members
//! (Cassowary/Kite/Bars/Ruled families) are exactly what makes this
//! non-vacuous: without the pin, those are the worlds whose composition
//! genuinely differs from wherever the picker was summoned.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayState;

/// One frame's worth of the axes this law holds fixed.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ChromeSnapshot {
    card_rect: [f32; 4],
    list_style: theme::ListStyle,
    facet_style: theme::FacetStyle,
    pane_split: theme::PaneSplit,
    row_pitch: f32,
    /// The window's drawn CAPACITY — how many display lines it shows at
    /// once. Deliberately excludes both `top` (the scroll offset) and
    /// `sel_row`: as the sweep steps the selection down the roster, the
    /// window legitimately SCROLLS to keep it in view (ordinary, kind-agnostic
    /// list-nav — item 609 does not touch it), and the selected row walks
    /// down with it. What must not move is the window's own SIZE — a
    /// facet/list-style composition with more header overhead or a taller row
    /// pitch draws fewer lines in the same card height, which is exactly the
    /// "how many rows are visible" jitter the reader reported.
    capacity: usize,
}

fn snapshot(p: &mut TextPipeline, ov: &OverlayState) -> ChromeSnapshot {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = ov.item_strings();
    v.overlay_selected = ov.selected;
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_align = Some(ov.align);
    p.sync_theme();
    p.set_view(&v);
    let card_rect = p.overlay_card_rect().expect("theme picker card");
    let (_top, lines, _sel_row, _card_h, _canvas_h) = p
        .overlay_window_report()
        .expect("theme picker window report");
    ChromeSnapshot {
        card_rect,
        list_style: crate::render::effective_list_style(),
        facet_style: crate::render::effective_facet_style(),
        pane_split: crate::render::effective_pane_split(),
        row_pitch: p.overlay_lh(),
        capacity: lines,
    }
}

/// Move the picker's selection to `name` via the DELIBERATE-move door
/// (`preview_move`), the exact call the keyboard-nav path runs.
fn step_to(ov: &mut OverlayState, name: &str) {
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

#[test]
fn full_roster_arrow_sweep_holds_the_card_fixed() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping full_roster_arrow_sweep_holds_the_card_fixed: no wgpu adapter");
        return;
    };
    crate::render::set_card_anchor_test_override(None);
    crate::render::set_list_style_test_override(None);
    crate::render::set_facet_style_test_override(None);
    let restore = theme::active().name;

    // NON-VACUITY (enrolment): the roster genuinely carries more than one
    // `ListStyle` — named here, not assumed, so the sweep below cannot pass
    // by coincidence of an all-Pane roster.
    let first_style = theme::THEMES[0].render_caps.list_style;
    assert!(
        theme::THEMES
            .iter()
            .any(|t| t.render_caps.list_style != first_style),
        "the roster must carry more than one ListStyle for this law to mean anything"
    );

    theme::set_active_by_name("Tawny").unwrap();
    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names.clone(), theme::active_index());
    let summoned = snapshot(&mut p, &ov);

    let mut worlds_seen_ground: Vec<theme::Srgb> = Vec::new();
    for name in &names {
        step_to(&mut ov, name);
        let snap = snapshot(&mut p, &ov);
        assert_eq!(
            snap, summoned,
            "world {name}: the picker's own chrome must not move (summoned={summoned:?}, now={snap:?})"
        );
        worlds_seen_ground.push(theme::active().base_100);
    }

    // PRESENCE: the page ground genuinely differed across the sweep — the
    // fixed-chrome law is not satisfied by a roster that never really changed
    // anything. `base_100` is read straight off the world driving the ground
    // shader; the sweep must not have painted the same ground colour the
    // whole way through.
    let first_ground = worlds_seen_ground[0];
    assert!(
        worlds_seen_ground.iter().any(|g| *g != first_ground),
        "the ground behind the card must have changed across the sweep — every \
         step reported the same base_100 ({first_ground:?})"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
    crate::render::set_list_style_test_override(None);
    crate::render::set_facet_style_test_override(None);
}

/// PRESENCE, at the pixel: a real render of the picker at the FIRST and a
/// LATER step in the sweep shows a different pixel just outside the card's
/// own left edge (the page ground), even though the card itself never moved.
#[test]
fn the_ground_outside_the_card_really_does_repaint_across_the_sweep() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_ground_outside_the_card_really_does_repaint_across_the_sweep: \
             no wgpu adapter"
        );
        return;
    };
    crate::render::set_card_anchor_test_override(None);
    let restore = theme::active().name;
    theme::set_active_by_name("Tawny").unwrap();

    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());

    let render_corner = |p: &mut TextPipeline, ov: &OverlayState| -> [u8; 4] {
        let mut v = view("hello world\n", 0, 0);
        v.overlay_active = true;
        v.overlay_items = ov.item_strings();
        v.overlay_selected = ov.selected;
        v.overlay_lens = ov.lens_strip();
        v.overlay_sections = ov.item_sections();
        v.overlay_align = Some(ov.align);
        p.sync_theme();
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let (texture, tview) = super::dither::offscreen(&device, 1200, 800);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl theme-picker-pin-law encoder"),
        });
        p.render(&mut encoder, &tview).unwrap();
        queue.submit(Some(encoder.finish()));
        let pixels = super::dither::read_pixels(&device, &queue, &texture, 1200, 800);
        // The card is TopCenter on Tawny (a comfortable interior rail): the
        // canvas's own top-left corner, 4px in, is page ground on every world
        // in the roster — never inside the card's own footprint.
        pixels[(4u32 * 1200 + 4) as usize]
    };

    let before = render_corner(&mut p, &ov);
    // Cross to a world whose GROUND genuinely differs from Tawny's own.
    step_to(&mut ov, "Wagtail");
    let after = render_corner(&mut p, &ov);

    assert_ne!(
        before, after,
        "the page ground behind the fixed card must repaint across a world crossing \
         (before={before:?}, after={after:?})"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}

/// MUTATION PROOF: unpinning the picker's chrome mid-sweep (bypassing
/// `pin_picker_chrome`, exactly the pre-item-609 behaviour) makes the very
/// same crossing move the card — proving the laws above hold because of the
/// pin, not because this fixture never really crosses compositions.
#[test]
fn without_the_pin_the_sweep_would_move_the_card() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping without_the_pin_the_sweep_would_move_the_card: no wgpu adapter");
        return;
    };
    crate::render::set_card_anchor_test_override(None);
    let restore = theme::active().name;
    theme::set_active_by_name("Tawny").unwrap();

    let names: Vec<String> = theme::THEMES.iter().map(|t| t.name.to_string()).collect();
    let mut ov = OverlayState::new_theme(names, theme::active_index());
    assert!(
        crate::render::picker_chrome_pin_probe().is_some(),
        "summoning a Theme overlay must pin the picker's chrome"
    );
    let summoned = snapshot(&mut p, &ov);

    // Find the FIRST world in roster order whose own `ListStyle` differs from
    // the summoned (Tawny/Pane) one — named by the roster itself, never assumed.
    let target = theme::THEMES
        .iter()
        .find(|t| t.render_caps.list_style != theme::ListStyle::Pane)
        .expect("the roster carries at least one non-Pane world")
        .name;

    // THE MUTATION: drop the pin the same way `preview_move` would if
    // `pin_picker_chrome` had never been wired up (never call it again after
    // this — the point is to observe the picker's composition WITHOUT it).
    crate::render::unpin_picker_chrome();

    step_to(&mut ov, target);
    let unpinned = snapshot(&mut p, &ov);

    assert_ne!(
        unpinned, summoned,
        "with the pin dropped, crossing into {target} (a non-Pane world) must \
         move the card — the mutation the laws above are proven against"
    );

    // Restore the pin so this test does not leak an unpinned state into
    // whatever the harness runs next.
    crate::render::pin_picker_chrome();
    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}
