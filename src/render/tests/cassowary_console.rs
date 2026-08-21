//! Cassowary's operations-console composition, expressed only through shared
//! renderer capabilities. These first laws pin the pre-existing seams before
//! the visual treatment grows: the world adopts the card-backed list, and the
//! locator reports state without repeating the active category label.

use super::super::*;
use super::{headless_dqp, view};
use crate::theme::{FacetStyle, ListStyle, LocationStyle, THEMES, Theme};

fn cassowary() -> &'static Theme {
    THEMES
        .iter()
        .find(|world| world.name == "Cassowary")
        .expect("Cassowary is in the authored roster")
}

#[test]
fn cassowary_authors_the_shared_docked_active_facet_treatment() {
    assert_eq!(
        cassowary().render_caps.facet_style,
        FacetStyle::DockedTab,
        "the active category joins the console border instead of floating as a chip"
    );
}

#[test]
fn cassowary_promotes_the_development_pane_composition_into_authored_data() {
    let caps = cassowary().render_caps;
    assert_eq!(
        caps.list_style,
        ListStyle::Pane,
        "the operations console is one card-backed pane, not floating row bars"
    );
}

#[test]
fn indexed_locator_can_report_only_the_real_active_category_index() {
    let style = match cassowary().render_caps.location_style {
        LocationStyle::RotatedRail(style) => style,
        other => panic!("Cassowary must keep its rotated location rail, got {other:?}"),
    };
    assert_eq!(
        crate::render::rotated_location::format_location_text(style, "Files", Some(2)),
        Some("02".to_string()),
        "the active tab owns the label; the side locator says only its real index"
    );
    assert_eq!(
        crate::render::rotated_location::format_location_text(style, "Files", None),
        None,
        "an indexed locator never invents an index"
    );
}

fn console_view(active: usize) -> ViewState {
    let mut v = view("# Console\n\nThe document stays behind the frame.\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = (0..12).map(|i| format!("Command {i}")).collect();
    v.overlay_bindings = (0..12).map(|i| format!("C-{i}")).collect();
    v.overlay_selected = 3;
    v.overlay_hint = "type to filter   ↵ choose   ←/→ category   esc close".into();
    v.overlay_lens = ["All", "Files", "Navigate", "Format", "View"]
        .iter()
        .enumerate()
        .map(|(i, label)| (label.to_string(), i == active))
        .collect();
    v
}

#[test]
fn docked_facet_draw_hit_and_pane_edge_are_one_geometry_across_canvas_and_dpi() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");

    let mut cells = 0;
    for dpi in [1.0f32, 2.0f32] {
        for (logical_w, logical_h) in [(760u32, 620u32), (1200, 800), (1600, 1000)] {
            let (w, h) = (
                (logical_w as f32 * dpi) as u32,
                (logical_h as f32 * dpi) as u32,
            );
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping docked-facet geometry: no wgpu adapter");
                return;
            };
            p.set_dpi(dpi);
            p.set_view(&console_view(1));
            p.prepare(&device, &queue, w, h).unwrap();

            let (dock, [_, card_y, _, _]) = p
                .docked_facet_geometry_probe()
                .expect("DockedTab resolves a drawn/hit band");
            assert!(
                (dock.bottom() - card_y).abs() < 0.01,
                "{logical_w}x{logical_h}@{dpi}x: dock bottom {} must be the pane top {card_y}",
                dock.bottom()
            );
            assert!(
                dock.top >= 0.0,
                "narrow degradation keeps the dock on-canvas"
            );

            let tab = p.overlay_theme_underline.expect("active tab fill");
            assert!(
                (tab[1] + tab[3] - card_y).abs() < 0.01,
                "active tab's fill joins the same pane edge"
            );
            let x = tab[0] + tab[2] * 0.5;
            assert_eq!(
                p.overlay_lens_at(x, dock.center()),
                Some(1),
                "the translated shaped label remains its own hit target"
            );
            assert!(
                p.overlay_lens_at(x, dock.top - 0.1).is_none(),
                "outside the drawn dock is outside its hit span"
            );
            cells += 1;
        }
    }
    assert_eq!(cells, 6, "narrow/wide × 1x/2x sweep is enrolled");
}

#[test]
fn docked_facet_is_reusable_data_not_a_cassowary_identity_branch() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Saltpan").expect("synthetic carrier ships");
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping synthetic docked-facet carrier: no wgpu adapter");
        return;
    };
    set_facet_style_test_override(Some(FacetStyle::DockedTab));
    set_list_style_test_override(Some(ListStyle::Pane));
    set_pane_split_test_override(Some(theme::PaneSplit::Unified));
    p.set_view(&console_view(2));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(p.docked_facet_geometry_probe().is_some());
    let tab = p
        .overlay_theme_underline
        .expect("synthetic carrier draws the tab");
    assert_eq!(
        p.overlay_lens_at(tab[0] + tab[2] * 0.5, tab[1] + tab[3] * 0.5),
        Some(2)
    );
    set_facet_style_test_override(None);
    set_list_style_test_override(None);
    set_pane_split_test_override(None);
}
