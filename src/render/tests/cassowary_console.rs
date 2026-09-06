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
        CardShape::Chamfered {
            top_cut_px: 0.0,
            bottom_cut_px: 11.0,
        },
        "the console's docked seam edge (top) is square; the free edge (bottom) \
         keeps the shared chamfer"
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
    v.overlay_title = "commands".to_string();
    v.overlay_items = (0..12).map(|i| format!("Command {i}")).collect();
    v.overlay_bindings = (0..12).map(|i| format!("C-{i}")).collect();
    v.overlay_selected = 3;
    v.overlay_hint = "type to filter   ↵ choose   ←/→ category   esc close".into();
    v.overlay_lens = [
        "All", "Files", "Navigate", "Format", "View", "Tools", "Settings", "Recent",
    ]
    .iter()
    .enumerate()
    .map(|(i, label)| (label.to_string(), i == active))
    .collect();
    v
}

fn core_ink_pixels(pixels: &[[u8; 4]], width: u32, rect: [f32; 4], ink: [u8; 4]) -> usize {
    let x0 = rect[0].floor().max(0.0) as u32;
    let y0 = rect[1].floor().max(0.0) as u32;
    let x1 = (rect[0] + rect[2]).ceil().min(width as f32) as u32;
    let y1 = (rect[1] + rect[3])
        .ceil()
        .min((pixels.len() as u32 / width) as f32) as u32;
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| pixels[(y * width + x) as usize]))
        .filter(|px| {
            px[..3]
                .iter()
                .zip(ink[..3].iter())
                .all(|(a, b)| a.abs_diff(*b) <= 44)
        })
        .count()
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
            let docked_label = p
                .docked_facet_buffer
                .layout_runs()
                .next()
                .expect("the dock owns a separately shaped navigation strip");
            assert!(
                !docked_label.glyphs.is_empty()
                    && docked_label.line_w <= p.overlay_geometry(w).text_w,
                "the complete category strip has ink and fits the card"
            );
            // The fill deliberately OVERLAPS the pane edge rather than meeting
            // it exactly — it bridges the card's own border ring so the tab's
            // mouth reads continuous with the card ground (the pixel-level
            // seam law lives in `docked_tab_seam.rs`; this is the geometry
            // half, byte-for-byte against the same overlap the draw path uses).
            let seam_overlap = p.docked_tab_seam_overlap_probe();
            assert!(
                (tab[1] + tab[3] - (card_y + seam_overlap)).abs() < 0.01,
                "active tab's fill overlaps the pane edge by exactly the seam \
                 merge margin, got bottom {} vs card_y {card_y} + {seam_overlap}",
                tab[1] + tab[3]
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

            // APPEARANCE, not state: every inactive label must still put real
            // phosphor ink on the docked navigation line. The bug retained all
            // five hit spans while clipping the original strip wholesale, so
            // geometry-only assertions stayed green as four labels vanished.
            let pixels = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let ink = theme::faint().rgba_bytes();
            for idx in (0..p.overlay_lens.len()).filter(|idx| *idx != 1) {
                let label = &console_view(1).overlay_lens[idx].0;
                let mut hits = 0usize;
                let (left, right) = lens_hit_span(&p, idx, dock.center()).unwrap_or_else(|| {
                    panic!(
                        "{logical_w}x{logical_h}@{dpi}x: inactive facet {label:?} lost its hit span"
                    )
                });
                hits +=
                    core_ink_pixels(&pixels, w, [left, dock.top, right - left, dock.height], ink);
                assert!(
                    hits >= 2,
                    "{logical_w}x{logical_h}@{dpi}x: inactive facet {label:?} has no visible ink"
                );
            }
            cells += 1;
        }
    }
    assert_eq!(cells, 6, "narrow/wide × 1x/2x sweep is enrolled");
}

fn lens_hit_span(p: &TextPipeline, index: usize, y: f32) -> Option<(f32, f32)> {
    let card = p.overlay_card_rect()?;
    let mut xs = (card[0].floor() as i32..(card[0] + card[2]).ceil() as i32)
        .filter(|x| p.overlay_lens_at(*x as f32 + 0.5, y) == Some(index));
    let first = xs.next()? as f32;
    let last = xs.last().unwrap_or(first as i32) as f32 + 1.0;
    Some((first, last))
}

#[test]
fn empty_results_keep_the_active_tab_and_commands_placard_visible() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping Cassowary empty-result appearance: no wgpu adapter");
        return;
    };
    let mut v = console_view(0);
    v.overlay_items.clear();
    v.overlay_bindings.clear();
    v.overlay_empty = Some("no matches".into());
    v.overlay_selected = 0;
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();

    let tab = p
        .overlay_theme_underline
        .expect("empty result keeps active tab");
    let placard = p
        .overlay_shape_placard(&p.overlay_geometry(w))
        .expect("empty result keeps COMMANDS placard");
    let pixels = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    let ink = theme::base_content().rgba_bytes();
    assert!(
        core_ink_pixels(&pixels, w, tab, ink) >= 3,
        "active All tab has state but no visible label ink"
    );
    let visible_placard = [
        placard.0,
        placard.1.max(0.0),
        placard.2,
        (h as f32 - placard.1.max(0.0)).min(placard.3).max(0.0),
    ];
    let placard_ink = theme::placard_ink(theme::PlacardInk::Bold).rgba_bytes();
    assert!(
        core_ink_pixels(&pixels, w, visible_placard, placard_ink) >= 24,
        "COMMANDS placard has geometry but no visible phosphor ink"
    );
}

#[test]
fn typed_filter_keeps_every_active_tab_complete_and_commands_placard_visible() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping Cassowary typed-filter appearance: no wgpu adapter");
        return;
    };
    let mut enrolled = 0;
    for active in 0..console_view(0).overlay_lens.len() {
        let mut v = console_view(active);
        v.overlay_query = "save".into();
        v.overlay_items = vec!["Save".into(), "Save a Copy…".into()];
        v.overlay_bindings = vec!["⌘S".into(), String::new()];
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();

        let dock = p
            .docked_facet_geometry_probe()
            .expect("typed filter keeps dock")
            .0;
        let pixels = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);
        for idx in 0..v.overlay_lens.len() {
            let (left, right) = lens_hit_span(&p, idx, dock.center())
                .unwrap_or_else(|| panic!("active {active}: facet {idx} lost its hit span"));
            let expected = if idx == active {
                theme::base_content().rgba_bytes()
            } else {
                theme::faint().rgba_bytes()
            };
            assert!(
                core_ink_pixels(
                    &pixels,
                    w,
                    [left, dock.top, right - left, dock.height],
                    expected
                ) >= 2,
                "active {active}: facet {idx} has no visible ink under a typed query"
            );
        }
        let placard = p
            .overlay_shape_placard(&p.overlay_geometry(w))
            .expect("typed filter keeps COMMANDS placard");
        assert!(
            core_ink_pixels(
                &pixels,
                w,
                [placard.0, placard.1.max(0.0), placard.2, placard.3],
                theme::placard_ink(theme::PlacardInk::Bold).rgba_bytes(),
            ) >= 24,
            "active {active}: typed filter loses COMMANDS placard ink"
        );
        enrolled += 1;
    }
    assert_eq!(enrolled, 8, "every active facet is appearance-enrolled");
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
    assert_eq!(
        p.overlay_facet_material.instance_count(),
        0,
        "DockedTab alone does not invent a material on a Flat world"
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
    assert_eq!(
        p.overlay_facet_material.instance_count(),
        0,
        "scanlines do not invent a tab on a non-DockedTab world"
    );
    assert_eq!(p.panel_material.scanlines(), Some((0.2, 5.0, 1.25)));
    assert_eq!(p.placard_material.scanlines(), p.panel_material.scanlines());
    for frame in 0..3 {
        assert!(
            !p.advance(1.0 / 60.0),
            "static scanlines must request no render follow-up on idle frame {frame}"
        );
    }

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

/// Run explicitly with
/// `cargo test --release --bin awl cassowary_static_material_release_cost`
/// `-- --ignored --nocapture`.
/// This is a report, not a timing threshold: hosted and local GPUs are unlike,
/// but the two arms share one process/device/frame and differ only by the
/// material capability, so the printed delta honestly isolates its local cost.
#[test]
#[ignore = "release-mode GPU cost report"]
fn cassowary_static_material_release_cost() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping Cassowary material cost: no wgpu adapter");
        return;
    };
    p.set_view(&console_view(1));

    let sample = |p: &mut TextPipeline, material, rounds: usize| -> Vec<u128> {
        set_summoned_material_test_override(Some(material));
        (0..rounds)
            .map(|_| {
                let start = std::time::Instant::now();
                p.prepare(&device, &queue, w, h).unwrap();
                let _ = super::pixeldiff::render_frame(p, &device, &queue, w, h);
                start.elapsed().as_nanos()
            })
            .collect()
    };
    let _ = sample(&mut p, SummonedMaterial::Flat, 8);
    let _ = sample(
        &mut p,
        SummonedMaterial::Scanlines {
            pitch_px: 4.0,
            line_px: 1.0,
            strength: 0.12,
        },
        8,
    );
    let median = |mut values: Vec<u128>| {
        values.sort_unstable();
        values[values.len() / 2]
    };
    let flat = median(sample(&mut p, SummonedMaterial::Flat, 40));
    let scanlines = median(sample(
        &mut p,
        SummonedMaterial::Scanlines {
            pitch_px: 4.0,
            line_px: 1.0,
            strength: 0.12,
        },
        40,
    ));
    set_summoned_material_test_override(None);
    eprintln!(
        "cassowary material release cost: flat={:.3}ms scanlines={:.3}ms \
         delta={:+.3}ms (median of 40 serialized 1200x800 \
         prepare+render+readback frames)",
        flat as f64 / 1_000_000.0,
        scanlines as f64 / 1_000_000.0,
        (scanlines as i128 - flat as i128) as f64 / 1_000_000.0,
    );
}

#[test]
fn static_material_enrolls_the_complete_overlay_surface_roster() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping material surface roster: no wgpu adapter");
        return;
    };
    let mut enrolled = 0;
    let mut faceted = 0;
    let mut flat = 0;
    let mut contextual = 0;
    let mut workspace = 0;
    let mut fingerprints = std::collections::BTreeSet::new();
    for kind in crate::overlay::OverlayKind::ALL {
        match kind {
            crate::overlay::OverlayKind::Goto
            | crate::overlay::OverlayKind::Project
            | crate::overlay::OverlayKind::ProjectBrowse
            | crate::overlay::OverlayKind::Browse
            | crate::overlay::OverlayKind::Theme
            | crate::overlay::OverlayKind::Caret
            | crate::overlay::OverlayKind::Dictionary
            | crate::overlay::OverlayKind::CjkLang
            | crate::overlay::OverlayKind::Date
            | crate::overlay::OverlayKind::Keymap
            | crate::overlay::OverlayKind::MoveDest
            | crate::overlay::OverlayKind::Command
            | crate::overlay::OverlayKind::Spell
            | crate::overlay::OverlayKind::Keybindings
            | crate::overlay::OverlayKind::History
            | crate::overlay::OverlayKind::Conflict
            | crate::overlay::OverlayKind::Credits
            | crate::overlay::OverlayKind::Settings
            | crate::overlay::OverlayKind::Assets
            | crate::overlay::OverlayKind::UserWords
            | crate::overlay::OverlayKind::Rename
            | crate::overlay::OverlayKind::InsertLink
            | crate::overlay::OverlayKind::KeepName
            | crate::overlay::OverlayKind::Context
            | crate::overlay::OverlayKind::ExportDest
            | crate::overlay::OverlayKind::TableDims
            | crate::overlay::OverlayKind::SearchFolder => {}
        }
        let mut v = view("teh\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = if kind.draws_title_prefix() {
            kind.title().to_string()
        } else {
            "".to_string()
        };
        v.overlay_window_rows = kind.window_rows();
        v.overlay_items = vec!["Alpha".into(), "Omega".into()];
        v.overlay_bindings = vec!["C-a".into(), "C-o".into()];
        v.overlay_hint = kind.hint();
        if kind == crate::overlay::OverlayKind::Spell {
            v.overlay_spell = Some((0, 0, 3));
            v.overlay_title = "".to_string();
            v.overlay_hint.clear();
            contextual += 1;
        } else if kind.workspace_shape().is_some() {
            v.overlay_workspace = true;
            v.overlay_lens = crate::facets::scheme(kind)
                .map(|scheme| scheme.strip_labels(0))
                .unwrap_or_default();
            workspace += 1;
        } else if let Some(scheme) = crate::facets::scheme(kind) {
            v.overlay_lens = scheme.strip_labels(0);
            faceted += 1;
        } else {
            flat += 1;
        }
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        assert_eq!(p.panel_material.instance_count(), 1, "{kind:?} pane");
        let geom = p.overlay_geometry(1200);
        let has_placard = p.overlay_shape_placard(&geom).is_some();
        assert_eq!(
            p.placard_material.instance_count(),
            u32::from(has_placard),
            "{kind:?}: material follows the placard geometry this real surface actually owns"
        );
        let card = p.overlay_card_rect().expect("real surface has geometry");
        fingerprints.insert((
            geom.header_rows,
            !v.overlay_lens.is_empty(),
            v.overlay_workspace,
            v.overlay_spell.is_some(),
            (card[0].round() as i32, card[1].round() as i32),
        ));
        enrolled += 1;
    }
    assert_eq!(enrolled, crate::overlay::OverlayKind::ALL.len());
    assert!(
        faceted > 0 && flat > 0 && contextual == 1 && workspace > 0,
        "real roster must enroll every production surface family: \
         faceted={faceted} flat={flat} contextual={contextual} \
         workspace={workspace}"
    );
    assert!(
        fingerprints.len() >= 4,
        "the sweep must reach distinct production geometries, not replay one \
         fake console view: {fingerprints:?}"
    );
}

fn px_at(pixels: &[[u8; 4]], w: i64, x: i64, y: i64) -> [u8; 4] {
    pixels[(y * w + x) as usize]
}

/// THE CORNER LAW: the console's docked seam edge (top, where the facet strip
/// lives) is a SQUARE corner — sampled inward it must read as card fill, not
/// page ground — while the free bottom edge keeps the shared 45° cut, reading
/// the SAME discriminator `card_texture_shape.rs` proves against Quokka
/// (`ex + ey < cut` lands outside the fill). Both halves are asserted, so a
/// regression toward EITHER "still chamfered on top" or "no longer chamfered
/// on bottom" goes red — a presence floor paired with an absence floor, not
/// one alone. Swept across dpi so a chrome-padding-class bug (CLAUDE.md's
/// `--capture-dpi 1` tripwire) can't hide at the one scale a capture defaults to.
#[test]
fn cassowary_console_top_square_bottom_chamfered_across_dpi() {
    let _guard = crate::testlock::serial();
    let _world = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let card_fill = cassowary().base_300.rgba_bytes();
    let near = |a: [u8; 4], b: [u8; 4]| (0..3).all(|k| (a[k] as i16 - b[k] as i16).abs() <= 4);
    let mut cells = 0;
    for dpi in [1.0f32, 2.0f32] {
        let (logical_w, logical_h) = (1200u32, 800u32);
        let (w, h) = (
            (logical_w as f32 * dpi) as u32,
            (logical_h as f32 * dpi) as u32,
        );
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping cassowary corner-shape law: no wgpu adapter");
            return;
        };
        p.set_dpi(dpi);
        p.set_view(&console_view(1));
        p.prepare(&device, &queue, w, h).unwrap();
        let [cx, cy, _cw, ch] = p.overlay_card_rect().expect("console card is open");
        let pixels = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);

        // 5 LOGICAL px inward on both axes: well inside a chamfer's cut
        // (5+5=10 < the authored 11px) and trivially inside a square corner.
        let inset = (5.0 * dpi) as i64;
        let top_left = px_at(&pixels, w as i64, cx as i64 + inset, cy as i64 + inset);
        assert!(
            near(top_left, card_fill),
            "{dpi}x: the docked seam's top-left corner, {inset}px inward, must be \
             SQUARE (card fill), got {top_left:?} vs fill {card_fill:?}"
        );
        let bottom_left = px_at(
            &pixels,
            w as i64,
            cx as i64 + inset,
            (cy + ch) as i64 - inset,
        );
        assert!(
            !near(bottom_left, card_fill),
            "{dpi}x: the free bottom-left corner, {inset}px inward, must still show \
             the authored 45° cut (page ground), got {bottom_left:?} which reads as \
             card fill {card_fill:?}"
        );
        cells += 1;
    }
    assert_eq!(cells, 2, "1x/2x dpi sweep is enrolled");
}

/// THE ONE-OWNER LAW: every console layer that draws a card-shaped quad this
/// frame carries the SAME chamfer pair `card_shape_texture` resolves for its
/// OWN rect — never a per-layer opinion invented independently (the bug this
/// round names: `overlay_material.rs` once hardcoded the placard's chamfer to
/// `0.0` regardless of the card's real shape). Derived from the roster
/// (`CardShape::Chamfered` adopters), not pinned to Cassowary by name, so a
/// future adopter is swept the moment it authors the shape.
#[test]
fn every_console_layer_shares_the_corner_mask_owner() {
    let _guard = crate::testlock::serial();
    let adopters: Vec<&str> = THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.card_shape, CardShape::Chamfered { .. }))
        .map(|t| t.name)
        .collect();
    assert!(
        adopters.len() >= 2,
        "the corner-mask law needs at least two adopters to prove it isn't \
         reading one world's own accidental agreement: {adopters:?}"
    );
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping corner-mask ownership law: no wgpu adapter");
        return;
    };
    let mut checked_panel = 0;
    let mut checked_scanlines = 0;
    let mut checked_placard = 0;
    for name in adopters {
        let _world = theme::WorldPin::world(name).expect("adopter ships");
        p.set_view(&console_view(1));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let card = p.overlay_card_rect().expect("card is open");
        let (expected, _) = p.card_shape_texture(&[card]);
        assert_eq!(
            p.panel_card.chamfer(),
            (expected.top, expected.bottom),
            "{name}: panel fill disagrees with the corner-mask owner"
        );
        assert_eq!(
            p.panel_shadow.chamfer(),
            (expected.top, expected.bottom),
            "{name}: panel shadow disagrees with the corner-mask owner"
        );
        assert_eq!(
            p.panel_border.chamfer(),
            (expected.top, expected.bottom),
            "{name}: panel border disagrees with the corner-mask owner"
        );
        checked_panel += 1;

        if p.panel_material.instance_count() > 0 {
            assert_eq!(
                p.panel_material.chamfer(),
                (expected.top, expected.bottom),
                "{name}: scanline material disagrees with the corner-mask owner"
            );
            checked_scanlines += 1;
        }
        if p.placard_material.instance_count() > 0 {
            let geom = p.overlay_geometry(1200);
            let placard_rect = p
                .overlay_shape_placard(&geom)
                .map(|(x, y, w, h)| [x, y, w, h])
                .expect("placard_material drew an instance so its rect exists");
            let (placard_expected, _) = p.card_shape_texture(&[placard_rect]);
            assert_eq!(
                p.placard_material.chamfer(),
                (placard_expected.top, placard_expected.bottom),
                "{name}: placard disagrees with the corner-mask owner resolved \
                 against its OWN rect"
            );
            checked_placard += 1;
        }
    }
    assert_eq!(checked_panel, 2, "both adopters carry a card-backed panel");
    assert!(
        checked_scanlines >= 1,
        "no adopter enrolled the scanline material arm"
    );
    assert!(
        checked_placard >= 1,
        "no adopter enrolled the placard arm — the bug this law exists to catch \
         can only hide if this stays zero"
    );
}
