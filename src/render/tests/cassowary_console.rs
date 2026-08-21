//! Cassowary's operations-console composition, expressed only through shared
//! renderer capabilities. These first laws pin the pre-existing seams before
//! the visual treatment grows: the world adopts the card-backed list, and the
//! locator reports state without repeating the active category label.

use super::super::*;
use super::{headless_dqp, view};
use crate::theme::{
    CardShape, FacetStyle, ListStyle, LocationStyle, PlacardPlacement, SummonedMaterial, THEMES,
    Theme,
};

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
fn cassowary_authors_a_submerged_placard_and_chamfered_console() {
    let caps = cassowary().render_caps;
    assert_eq!(
        caps.placard_placement,
        PlacardPlacement::Bleed {
            x_em: 0.0,
            y_em: 0.34,
        },
        "the COMMANDS placard deliberately leaves the bottom of the viewport"
    );
    assert_eq!(
        caps.card_shape,
        CardShape::Chamfered { cut_px: 11.0 },
        "the console keeps the shared chamfered card geometry"
    );
}

#[test]
fn cassowary_authors_the_static_shared_scanline_material() {
    assert_eq!(
        cassowary().render_caps.summoned_material,
        SummonedMaterial::Scanlines {
            pitch_px: 4.0,
            line_px: 1.0,
            strength: 0.12,
        }
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

#[test]
fn submerged_placard_geometry_is_drawn_reported_and_rail_consumed_across_dpi() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let mut logical_crops = Vec::new();

    for dpi in [1.0f32, 2.0f32] {
        let (w, h) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping submerged placard geometry: no wgpu adapter");
            return;
        };
        p.set_dpi(dpi);
        p.set_view(&console_view(1));
        p.prepare(&device, &queue, w, h).unwrap();
        let geom = p.overlay_geometry(w);
        let (_, py, _, ph) = p
            .overlay_shape_placard(&geom)
            .expect("Cassowary's COMMANDS placard draws at this canvas");
        let crop = py + ph - h as f32;
        assert!(
            crop > 2.0 * dpi,
            "the authored bottom bleed must produce a visible crop at {dpi}x, got {crop}px"
        );
        assert!(
            p.rotated_rail_probe(&geom).is_some(),
            "the index rail consumes the same displaced placard geometry"
        );
        logical_crops.push(crop / dpi);
    }
    assert!(
        (logical_crops[0] - logical_crops[1]).abs() < 8.0,
        "em-relative crop depth stays logically stable across DPI: {logical_crops:?}"
    );
}

#[test]
fn placard_bleed_is_reusable_data_on_a_synthetic_world() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Saltpan").expect("synthetic carrier ships");
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping synthetic placard placement: no wgpu adapter");
        return;
    };
    set_title_style_test_override(Some(theme::TitleStyle::Placard {
        corner: theme::PlacardCorner::BL,
        scale: 3.0,
        ink: theme::PlacardInk::Bold,
    }));
    p.set_view(&console_view(0));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);

    set_placard_placement_test_override(Some(PlacardPlacement::Contained));
    let contained = p
        .overlay_shape_placard(&geom)
        .expect("synthetic placard draws");
    set_placard_placement_test_override(Some(PlacardPlacement::Bleed {
        x_em: 0.0,
        y_em: 0.5,
    }));
    let bled = p
        .overlay_shape_placard(&geom)
        .expect("synthetic bled placard draws");
    let font_size = contained.3 / 1.1;
    assert!((bled.1 - contained.1 - 0.5 * font_size).abs() < 0.01);
    assert_eq!(bled.0, contained.0, "zero x-em leaves the x anchor alone");

    set_placard_placement_test_override(None);
    set_title_style_test_override(None);
}

#[test]
fn scanline_material_is_reusable_static_data_with_one_absolute_phase() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Saltpan").expect("synthetic carrier ships");
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping synthetic scanline material: no wgpu adapter");
        return;
    };
    set_title_style_test_override(Some(theme::TitleStyle::Placard {
        corner: theme::PlacardCorner::BL,
        scale: 3.0,
        ink: theme::PlacardInk::Bold,
    }));
    set_summoned_material_test_override(Some(SummonedMaterial::Scanlines {
        pitch_px: 5.0,
        line_px: 1.25,
        strength: 0.2,
    }));
    p.set_view(&console_view(0));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(p.panel_material.instance_count(), 1);
    assert_eq!(p.placard_material.instance_count(), 1);
    assert_eq!(p.panel_material.scanlines(), Some((0.2, 5.0, 1.25)));
    assert_eq!(p.placard_material.scanlines(), p.panel_material.scanlines());

    let shader = include_str!("../../../shaders/selection.wgsl");
    assert!(shader.contains("let phase = in.px.y"));
    assert!(
        !shader.contains("g.time"),
        "the material shader owns no clock"
    );
    assert!(
        !shader.contains("random("),
        "the material owns no randomness"
    );
    set_summoned_material_test_override(None);
    set_title_style_test_override(None);
}
