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
    /// list-nav this law leaves untouched), and the selected row walks
    /// down with it. What must not move is the window's own SIZE — a
    /// facet/list-style composition with more header overhead or a taller row
    /// pitch draws fewer lines in the same card height, which is exactly the
    /// "how many rows are visible" jitter the reader reported.
    capacity: usize,
}

fn snapshot(p: &mut TextPipeline, ov: &OverlayState) -> ChromeSnapshot {
    snapshot_with_theme_identity(p, ov, true)
}

fn snapshot_with_theme_identity(
    p: &mut TextPipeline,
    ov: &OverlayState,
    theme_picker: bool,
) -> ChromeSnapshot {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_theme_picker = theme_picker;
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

fn snapshot_at_zoom(p: &mut TextPipeline, ov: &OverlayState, zoom: f32) -> ChromeSnapshot {
    let mut v = view("hello world\n", 0, 0);
    v.zoom = zoom;
    v.overlay_active = true;
    v.overlay_theme_picker = true;
    v.overlay_items = ov.item_strings();
    v.overlay_selected = ov.selected;
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_align = Some(ov.align);
    p.sync_theme();
    p.set_view(&v);
    let card_rect = p.overlay_card_rect().expect("theme picker card");
    let (_, lines, _, _, _) = p.overlay_window_report().expect("theme picker report");
    ChromeSnapshot {
        card_rect,
        list_style: crate::render::effective_list_style(),
        facet_style: crate::render::effective_facet_style(),
        pane_split: crate::render::effective_pane_split(),
        row_pitch: p.overlay_lh(),
        capacity: lines,
    }
}

#[test]
fn every_opener_and_document_zoom_share_one_theme_picker_geometry() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(concat!(
            "skipping every_opener_and_document_zoom_share_one_theme_picker_geometry: ",
            "no wgpu adapter"
        ));
        return;
    };
    let names: Vec<String> = crate::theme::world_names()
        .into_iter()
        .map(str::to_string)
        .collect();
    let mut expected = None;
    let mut cells = 0usize;
    for opener in crate::theme::THEMES {
        crate::theme::set_active_by_name(opener.name).unwrap();
        let overlay = OverlayState::new_theme(names.clone(), crate::theme::active_index());
        for zoom in [0.5, 1.0, 2.0] {
            let got = snapshot_at_zoom(&mut p, &overlay, zoom);
            assert_eq!(
                got.card_rect[2], 545.0,
                "{}/{zoom}: the ordinary Themes card has one 545-logical width",
                opener.name
            );
            if let Some(expected) = expected {
                assert_eq!(
                    got, expected,
                    "{}/{zoom}: opener or document zoom moved the Themes card",
                    opener.name
                );
            } else {
                expected = Some(got);
            }
            cells += 1;
        }
    }
    assert_eq!(cells, crate::theme::THEMES.len() * 3);
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
fn preview_updates_persistent_chrome_face_while_the_picker_face_stays_pinned() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let (opener, destination) = theme::THEMES
        .iter()
        .enumerate()
        .flat_map(|(opener_index, opener)| {
            theme::THEMES
                .iter()
                .skip(opener_index + 1)
                .map(move |destination| (opener, destination))
        })
        .find(|(opener, destination)| {
            opener.font != destination.font && opener.base_100 != destination.base_100
        })
        .expect("the roster must vary both its body face and page palette");

    theme::set_active_by_name(opener.name).unwrap();
    let names: Vec<String> = theme::THEMES
        .iter()
        .map(|theme| theme.name.to_string())
        .collect();
    let mut overlay = OverlayState::new_theme(names, theme::active_index());
    let persistent_at_open = format!("{:?}", panel_attrs());
    let picker_at_open = format!("{:?}", overlay_panel_attrs());

    step_to(&mut overlay, destination.name);

    let persistent_at_preview = format!("{:?}", panel_attrs());
    let picker_at_preview = format!("{:?}", overlay_panel_attrs());
    assert_ne!(
        persistent_at_preview, persistent_at_open,
        "{} -> {}: persistent chrome behind Themes kept the opener face",
        opener.name, destination.name
    );
    assert!(
        persistent_at_preview.contains(destination.font),
        "persistent chrome must adopt destination face {} (got {persistent_at_preview})",
        destination.font
    );
    assert_eq!(
        picker_at_preview, picker_at_open,
        "the Themes card must retain its opener face"
    );
    let picker_theme = crate::render::overlay_chrome_theme();
    assert_eq!(picker_theme.font, opener.font);
    assert_eq!(picker_theme.base_100, opener.base_100);
    assert_eq!(theme::active().font, destination.font);
    assert_eq!(theme::active().base_100, destination.base_100);
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
            "world {name}: the picker's own chrome must not move \
             (summoned={summoned:?}, now={snap:?})"
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

/// MUTATION PROOF: withholding the typed Themes identity makes the same
/// crossing move the card — proving the laws above hold because the render
/// path enrols this card in the pin, not because the fixture never crosses
/// compositions.
#[test]
fn without_the_typed_theme_identity_the_sweep_would_move_the_card() {
    let _g = crate::testlock::serial();
    let Some((_device, _queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping typed-Theme-identity mutation law: no wgpu adapter");
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

    // THE MUTATION: withhold the typed identity which makes `set_view`
    // reassert the picker pin. The restore guard keeps the direct unpin below
    // local even if the assertion unwinds.
    let _pin_restore = crate::render::PickerChromePinRestore::capture();
    crate::render::unpin_picker_chrome();

    step_to(&mut ov, target);
    let unpinned = snapshot_with_theme_identity(&mut p, &ov, false);

    assert_ne!(
        unpinned, summoned,
        "with the pin dropped, crossing into {target} (a non-Pane world) must \
         move the card — the mutation the laws above are proven against"
    );

    theme::set_active_by_name(restore).unwrap();
    crate::render::set_card_anchor_test_override(None);
}

/// THE LEAK `PickerChromePinRestore` CLOSES: the mutation proof above steps
/// outside the self-healing overlay-construction lifecycle for one span (a
/// raw `unpin_picker_chrome()` with no matching summon), so a fixture that
/// forces the pin directly and dies inside that span — before its own
/// cleanup runs — used to leave a CONCRETE world index for whatever the
/// harness schedules onto this worker thread next, exactly the shape a
/// leaked forced `ListStyle` once corrupted an unrelated law with. Forces
/// the identical shape directly (pin, then panic) and checks the pin
/// afterward on the SAME thread, which is where a thread-local leak of this
/// kind would actually land.
#[test]
fn a_panic_after_a_direct_pin_does_not_leak_it_to_the_next_reader_on_this_thread() {
    let _g = crate::testlock::serial();
    assert_eq!(
        crate::render::picker_chrome_pin_probe(),
        None,
        "fixture assumption: nothing pinned the picker chrome ambiently entering this test"
    );
    let died = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _g = crate::testlock::serial();
        let _pin_restore = crate::render::PickerChromePinRestore::capture();
        crate::render::set_picker_chrome_pin_for_test(Some(0));
        panic!("a fixture that pinned the picker chrome directly and died before unpinning");
    }));
    assert!(died.is_err(), "the fixture above must have panicked");
    assert_eq!(
        crate::render::picker_chrome_pin_probe(),
        None,
        "PickerChromePinRestore must put the pin back on the unwinding path, or the \
         NEXT reader on this worker thread inherits a concrete world index nobody chose"
    );
}

#[test]
fn dismissing_themes_releases_its_font_and_palette_pin() {
    let _g = crate::testlock::serial();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let Some((_device, _queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping dismissing_themes_releases_its_font_and_palette_pin: no wgpu adapter");
        return;
    };
    let names = crate::theme::world_names()
        .into_iter()
        .map(str::to_string)
        .collect();
    let _overlay = OverlayState::new_theme(names, crate::theme::active_index());
    assert!(crate::render::picker_chrome_pin_probe().is_some());

    pipeline.set_view(&view("document only\n", 0, 0));
    assert_eq!(
        crate::render::picker_chrome_pin_probe(),
        None,
        "a no-overlay frame must restore ordinary chrome to the active world's font and palette"
    );
}

#[test]
fn a_direct_non_theme_view_cannot_inherit_an_earlier_themes_pin() {
    let _g = crate::testlock::serial();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let Some((_device, _queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping non-theme pin reset: no wgpu adapter");
        return;
    };
    crate::render::set_picker_chrome_pin_for_test(Some(0));
    let mut ordinary = view("document\n", 0, 0);
    ordinary.overlay_active = true;
    ordinary.overlay_items = vec!["ordinary command".into()];
    ordinary.overlay_theme_picker = false;
    ordinary.overlay_theme_chrome = None;
    pipeline.set_view(&ordinary);
    assert_eq!(
        crate::render::picker_chrome_pin_probe(),
        None,
        "a raw non-Theme ViewState must clear stale Themes composition"
    );
}
