//! Themes is an opaque bordered card over a sharp document. This replaces the
//! older partial-frost contract deliberately: the formerly footprint-reliant
//! roster remains the mutation witness, but a Theme card now declines both
//! full-canvas and footprint blur.

use super::super::TextPipeline;
use super::frost_feather::{DENSE, render_frame, theme_picker};
use super::headless_dqp;

fn formerly_footprint_reliant_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|world| crate::render::blur::footprint_frost_applies(world.render_caps.list_style))
        .map(|world| world.name)
        .collect()
}

fn prepare_picker(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text: &str,
    width: u32,
    height: u32,
) -> Vec<[u8; 4]> {
    let mut view = theme_picker(text);
    view.overlay_theme_picker = true;
    p.set_view(&view);
    render_frame(device, queue, p, width, height)
}

#[test]
fn themes_declines_every_blur_arm_and_keeps_an_opaque_card() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let enrolled = formerly_footprint_reliant_worlds();
    assert!(
        !enrolled.is_empty(),
        "the old footprint branch must have a real roster witness"
    );

    let (width, height) = (1200u32, 800u32);
    let Some((device, queue, mut pipeline)) = headless_dqp(width as f32, height as f32) else {
        eprintln!(
            "skipping themes_declines_every_blur_arm_and_keeps_an_opaque_card: no wgpu adapter"
        );
        return;
    };

    for opener in enrolled {
        crate::theme::set_active_by_name(opener).expect("roster opener");
        crate::render::pin_picker_chrome();
        let dense = prepare_picker(&mut pipeline, &device, &queue, DENSE, width, height);
        assert_eq!(
            pipeline.frost_mode(),
            None,
            "{opener}: Themes must use neither Full nor Footprint frost"
        );
        assert_eq!(
            crate::render::effective_card_elevation(),
            crate::theme::Elevation::Bordered,
            "{opener}: no-blur chrome must retain a real bordered backing"
        );
        let card = pipeline.overlay_card_rect().expect("Theme card");

        let empty = prepare_picker(&mut pipeline, &device, &queue, "", width, height);
        let x = (card[0] + card[2] - 10.0).floor() as usize;
        let y = (card[1] + card[3] - 10.0).floor() as usize;
        let index = y * width as usize + x;
        assert_eq!(
            dense[index], empty[index],
            "{opener}: document pixels leaked through the card's quiet lower-right pad"
        );
    }
}

#[test]
fn the_typed_theme_flag_is_the_no_blur_subject() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let Some((_device, _queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_typed_theme_flag_is_the_no_blur_subject: no wgpu adapter");
        return;
    };
    let opener = formerly_footprint_reliant_worlds()
        .into_iter()
        .next()
        .expect("a mutation witness");
    crate::theme::set_active_by_name(opener).unwrap();
    crate::render::pin_picker_chrome();
    crate::render::set_list_style_test_override(Some(crate::theme::ListStyle::Bars));

    let mut view = theme_picker(DENSE);
    view.overlay_theme_picker = false;
    pipeline.set_view(&view);
    assert!(
        matches!(
            pipeline.frost_mode(),
            Some(crate::render::blur::Frost::Footprint(_))
        ),
        "mutation: dropping the typed Theme flag must restore the old footprint branch"
    );
    crate::render::set_list_style_test_override(None);
}

#[test]
fn caret_keeps_its_local_footprint_on_the_same_enrolled_worlds() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    let Some((_device, _queue, mut pipeline)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping caret_keeps_its_local_footprint_on_the_same_enrolled_worlds: no wgpu adapter"
        );
        return;
    };
    let enrolled = formerly_footprint_reliant_worlds();
    assert!(
        !enrolled.is_empty(),
        "Caret needs a real footprint roster witness"
    );
    assert!(
        crate::overlay::OverlayKind::Caret.keeps_backdrop_crisp(),
        "the control fixture is the product's typed Caret projection"
    );
    for opener in enrolled {
        crate::theme::set_active_by_name(opener).unwrap();
        let mut view = theme_picker(DENSE);
        view.overlay_theme_picker = false;
        view.overlay_crisp = crate::overlay::OverlayKind::Caret.keeps_backdrop_crisp();
        pipeline.set_view(&view);
        assert!(
            matches!(
                pipeline.frost_mode(),
                Some(crate::render::blur::Frost::Footprint(_))
            ),
            "{opener}: Caret must retain its local footprint treatment"
        );
    }
}
