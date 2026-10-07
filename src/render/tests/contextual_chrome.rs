//! Commands + Insert-link are contextual choices: their shared card geometry,
//! authored world composition, and hit regions stay unchanged, while the Room
//! outside them remains readable instead of receiving full-canvas frost.

use super::super::*;
use super::pixeldiff::render_frame;
use super::{headless_dqp, view};
use std::collections::BTreeSet;

const DENSE: &str = "Context remains readable around this summoned choice.\n\
The sentence behind the card still matters to the action.\n\
Commands answer what to do without hiding the document.\n\
Links need the words around the destination for orientation.\n\
More visible prose keeps the pixel oracle non-vacuous.\n\
The shared chrome may cover only its own bounded footprint.\n\
Lines below the card remain crisp and legible.\n\
Nothing relocates attention into a separate workspace.\n\
The room stays present.\n";

fn style_name(style: theme::ListStyle) -> &'static str {
    match style {
        theme::ListStyle::Pane => "Pane",
        theme::ListStyle::Bars => "Bars",
        theme::ListStyle::Diagonal(_) => "Diagonal",
        theme::ListStyle::Ruled(_) => "Ruled",
    }
}

fn contextual_view(kind: crate::overlay::OverlayKind, text: &str) -> ViewState {
    assert!(matches!(
        kind,
        crate::overlay::OverlayKind::Command | crate::overlay::OverlayKind::InsertLink
    ));
    let mut v = view(text, 0, 0);
    v.overlay_active = true;
    v.overlay_retains_room = kind.retains_readable_room();
    v.overlay_title = kind.title().to_string();
    v.overlay_hint = kind.hint();
    v.overlay_window_rows = 12;
    match kind {
        crate::overlay::OverlayKind::Command => {
            v.overlay_items = crate::commands::names();
            v.overlay_bindings =
                crate::commands::effective_bindings(&[], &[], crate::keymap::KeymapFlavor::Native);
            v.overlay_lens = crate::facets::scheme(kind)
                .map(|scheme| scheme.strip_labels(0))
                .unwrap_or_default();
        }
        crate::overlay::OverlayKind::InsertLink => {
            v.overlay_query_placeholder = kind.field_placeholder().map(str::to_string);
            v.overlay_items = vec!["↵  insert link".to_string()];
            v.overlay_bindings = vec![String::new()];
        }
        _ => unreachable!(),
    }
    v
}

fn assert_contextual_frost(
    world: &theme::Theme,
    actual: Option<crate::render::blur::Frost>,
    ctx: &str,
) -> (usize, usize) {
    let flat = world.render_caps.backdrop == theme::Backdrop::Flat;
    let bare = crate::render::blur::footprint_frost_applies(world.render_caps.list_style);
    match (flat, bare, actual) {
        (true, _, None) | (false, false, None) => (0, 1),
        (false, true, Some(crate::render::blur::Frost::Footprint(foot))) => {
            assert!(foot.rect[2] > 0.0 && foot.rect[3] > 0.0, "{ctx}");
            (1, 0)
        }
        (_, _, actual) => {
            panic!("{ctx}: expected flat={flat} bare={bare}, got {actual:?}")
        }
    }
}

fn assert_room_ink_visible(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    kind: crate::overlay::OverlayKind,
    size: (u32, u32),
    dpi: f32,
    ctx: &str,
) {
    let (w, h) = size;
    let frost = p.frost_mode();
    let card = p.overlay_card_rect().expect("contextual card");
    // The shaped and blank rows can draw different document carets. Remove
    // those pixels so only prose can witness the Room beyond the card.
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let with_prose = render_frame(p, device, queue, w, h);

    let blank = "\n".repeat(DENSE.lines().count());
    p.set_view(&contextual_view(kind, &blank));
    p.prepare(device, queue, w, h).unwrap();
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let without_prose = render_frame(p, device, queue, w, h);

    let step = dpi.round().max(1.0) as usize;
    // Thin face ink may land only on an odd device-pixel phase at 2x density.
    // Visit every phase, stopping at the first real prose pixel.
    let visible = (0..step).any(|phase_y| {
        (0..step).any(|phase_x| {
            (phase_y..h as usize).step_by(step).any(|y| {
                (phase_x..w as usize).step_by(step).any(|x| {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let in_card = px >= card[0]
                        && px < card[0] + card[2]
                        && py >= card[1]
                        && py < card[1] + card[3];
                    let in_frost = frost.is_some_and(|mode| {
                        crate::render::blur::footprint_mask_for(mode, dpi, px, py) > 0.0
                    });
                    !in_card
                        && !in_frost
                        && with_prose[y * w as usize + x] != without_prose[y * w as usize + x]
                })
            })
        })
    });
    assert!(visible, "{ctx}: no prose pixels survived outside chrome");
}

fn covered_by_chrome(
    card: [f32; 4],
    frost: Option<crate::render::blur::Frost>,
    px: f32,
    py: f32,
) -> bool {
    const SHADOW_GUARD: f32 = 48.0;
    (px >= card[0] - SHADOW_GUARD
        && px < card[0] + card[2] + SHADOW_GUARD
        && py >= card[1] - SHADOW_GUARD
        && py < card[1] + card[3] + SHADOW_GUARD)
        || frost
            .is_some_and(|mode| crate::render::blur::footprint_mask_for(mode, 1.0, px, py) > 0.0)
}

fn assert_sweep_enrollment(
    styles: BTreeSet<&'static str>,
    footprinted: usize,
    unfrosted: usize,
    controls: usize,
    graded: usize,
) {
    assert_eq!(
        styles,
        BTreeSet::from(["Bars", "Diagonal", "Pane", "Ruled"])
    );
    assert!(
        footprinted > 0,
        "no bare composition reached footprint frost"
    );
    assert!(
        unfrosted > 0,
        "no card-backed/flat composition reached no frost"
    );
    assert!(controls > 0, "the full-takeover mutation arm never ran");
    assert_eq!(graded, crate::theme::THEMES.len() * 2 * 2 * 2);
}

/// The production world roster × both contextual surfaces × narrow/wide ×
/// 1x/2x density. Geometry and hit-testing continue through the shared plan;
/// the only changed axis is the typed takeover decision. The control mutation
/// clears that one bit and must recover Full frost on every non-flat world.
#[test]
fn contextual_chrome_retains_the_room_across_world_composition_geometry_and_density() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping contextual chrome roster law: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        unreachable!("adapter presence and shared test device must agree");
    };
    let mut styles = BTreeSet::new();
    let mut footprinted = 0usize;
    let mut unfrosted = 0usize;
    let mut controls = 0usize;
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        styles.insert(style_name(world.render_caps.list_style));
        for kind in [
            crate::overlay::OverlayKind::Command,
            crate::overlay::OverlayKind::InsertLink,
        ] {
            for (logical_w, logical_h) in [(700.0f32, 560.0f32), (1200.0, 800.0)] {
                for dpi in [1.0f32, 2.0] {
                    let (w, h) = (
                        (logical_w * dpi).round() as u32,
                        (logical_h * dpi).round() as u32,
                    );
                    p.set_dpi(dpi);
                    p.set_size(w as f32, h as f32);
                    let local = contextual_view(kind, DENSE);
                    assert!(
                        local.overlay_retains_room,
                        "{kind:?} lost its typed projection"
                    );
                    assert!(!local.overlay_crisp, "{kind:?} impersonated a preview");
                    p.set_view(&local);
                    p.prepare(&device, &queue, w, h).unwrap();
                    let ctx = format!("{} {kind:?} {logical_w}x{logical_h} dpi={dpi}", world.name);
                    assert!(!p.dims_doc(), "{ctx}: contextual chrome dimmed the Room");
                    let flat = world.render_caps.backdrop == theme::Backdrop::Flat;
                    let (foot, none) = assert_contextual_frost(&world, p.frost_mode(), &ctx);
                    footprinted += foot;
                    unfrosted += none;
                    assert_room_ink_visible(&mut p, &device, &queue, kind, (w, h), dpi, &ctx);

                    let geom = p.overlay_geometry(w);
                    let plan = p.overlay_row_plan(&geom);
                    let query = plan.query_band().expect("both cards own a query field");
                    assert!(
                        p.over_overlay_query(
                            geom.card_probe()[0] + 1.0,
                            query.top + query.height * 0.5,
                        ),
                        "{ctx}: the drawn field is not clickable",
                    );
                    let row = plan
                        .rows()
                        .iter()
                        .find(|row| row.item.is_some())
                        .expect("both cards own an action/result row");
                    let (x0, x1) = plan.row_x_span(row.display).expect("row has a hit span");
                    assert_eq!(
                        p.overlay_row_at((x0 + x1) * 0.5, row.top + row.height * 0.5),
                        row.item,
                        "{ctx}: shared drawn and interactive row geometry diverged",
                    );

                    let mut takeover = local;
                    takeover.overlay_retains_room = false;
                    p.set_view(&takeover);
                    p.prepare(&device, &queue, w, h).unwrap();
                    if flat {
                        assert_eq!(p.frost_mode(), None, "{ctx}: one-bit stays crisp");
                    } else {
                        assert_eq!(
                            p.frost_mode(),
                            Some(crate::render::blur::Frost::Full),
                            "{ctx}: clearing the typed reason did not restore takeover frost",
                        );
                        assert!(p.dims_doc(), "{ctx}: mutation control did not dim");
                        controls += 1;
                    }
                    graded += 1;
                }
            }
        }
    }
    p.set_dpi(1.0);
    assert_sweep_enrollment(styles, footprinted, unfrosted, controls, graded);
}

/// Pixel presence companion: one roster-derived representative of every list
/// composition must still draw real document ink outside the card/frost mask.
/// Comparing dense prose with an otherwise identical blank document isolates
/// the Room from the world's ground and the unchanged chrome.
#[test]
fn every_contextual_composition_leaves_real_document_ink_visible_outside_its_footprint() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping contextual chrome pixel law: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    let mut representatives = Vec::new();
    for label in ["Pane", "Bars", "Diagonal", "Ruled"] {
        let world = crate::theme::THEMES
            .iter()
            .find(|world| style_name(world.render_caps.list_style) == label)
            .unwrap_or_else(|| panic!("no {label} world enrolled"));
        representatives.push(world.name);
    }
    let (w, h) = (1000u32, 760u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        unreachable!("adapter presence and shared test device must agree");
    };
    let blank = "\n".repeat(DENSE.lines().count());
    let mut graded = 0usize;
    for world in representatives {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for kind in [
            crate::overlay::OverlayKind::Command,
            crate::overlay::OverlayKind::InsertLink,
        ] {
            let dense = contextual_view(kind, DENSE);
            p.set_view(&dense);
            p.prepare(&device, &queue, w, h).unwrap();
            let frost = p.frost_mode();
            let card = p.overlay_card_rect().expect("contextual card");
            let with_prose = render_frame(&mut p, &device, &queue, w, h);

            let empty = contextual_view(kind, &blank);
            p.set_view(&empty);
            p.prepare(&device, &queue, w, h).unwrap();
            let without_prose = render_frame(&mut p, &device, &queue, w, h);
            let mut visible = 0usize;
            for y in 0..h {
                for x in 0..w {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let in_card = px >= card[0]
                        && px < card[0] + card[2]
                        && py >= card[1]
                        && py < card[1] + card[3];
                    let in_frost = frost.is_some_and(|mode| {
                        crate::render::blur::footprint_mask_for(mode, 1.0, px, py) > 0.0
                    });
                    if !in_card
                        && !in_frost
                        && with_prose[(y * w + x) as usize] != without_prose[(y * w + x) as usize]
                    {
                        visible += 1;
                    }
                }
            }
            assert!(
                visible > 100,
                "{world} {kind:?}: only {visible} document pixels survived outside chrome",
            );
            graded += 1;
        }
    }
    assert_eq!(graded, 8, "four compositions × two contextual cards");
}

/// Placard worlds keep their authored card composition, but these two brief
/// contextual fields fold the title into the query line. The named-vs-untitled
/// differential must have no ink outside either card/frost footprint plus its
/// shadow allowance, proving the remote wordmark is absent rather than dimmer.
#[test]
fn contextual_titles_are_modest_and_integrated_on_every_placard_world() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping contextual title law: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    let placard_worlds: Vec<theme::Theme> = crate::theme::THEMES
        .iter()
        .copied()
        .filter(|world| {
            matches!(
                world.render_caps.title_style,
                theme::TitleStyle::Placard { .. }
            )
        })
        .collect();
    assert!(!placard_worlds.is_empty(), "no placard world enrolled");
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        unreachable!("adapter presence and shared test device must agree");
    };
    let mut graded = 0usize;
    for world in placard_worlds {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for width in [700u32, 1200] {
            p.set_size(width as f32, 800.0);
            for kind in [
                crate::overlay::OverlayKind::Command,
                crate::overlay::OverlayKind::InsertLink,
            ] {
                let named = contextual_view(kind, DENSE);
                p.set_view(&named);
                p.prepare(&device, &queue, width, 800).unwrap();
                let geom = p.overlay_geometry(width);
                assert!(
                    p.overlay_shape_placard(&geom).is_none(),
                    "{} {kind:?} width={width}: remote placard geometry survived",
                    world.name,
                );
                let query = p.panel_buffer.lines[0].text();
                let prefix = format!("{}   ", kind.title());
                assert!(
                    query.starts_with(&prefix),
                    "{} {kind:?} width={width}: {query:?} does not start with {prefix:?}",
                    world.name,
                );
                let named_card = p.overlay_card_rect().unwrap();
                let named_frost = p.frost_mode();
                let named_pixels = render_frame(&mut p, &device, &queue, width, 800);

                let mut untitled = named;
                untitled.overlay_title.clear();
                p.set_view(&untitled);
                p.prepare(&device, &queue, width, 800).unwrap();
                let untitled_card = p.overlay_card_rect().unwrap();
                let untitled_frost = p.frost_mode();
                let untitled_pixels = render_frame(&mut p, &device, &queue, width, 800);
                let mut remote_delta = 0usize;
                for y in 0..800u32 {
                    for x in 0..width {
                        let px = x as f32 + 0.5;
                        let py = y as f32 + 0.5;
                        if !covered_by_chrome(named_card, named_frost, px, py)
                            && !covered_by_chrome(untitled_card, untitled_frost, px, py)
                            && named_pixels[(y * width + x) as usize]
                                != untitled_pixels[(y * width + x) as usize]
                        {
                            remote_delta += 1;
                        }
                    }
                }
                assert_eq!(
                    remote_delta, 0,
                    "{} {kind:?} width={width}: remote title left {remote_delta} pixels",
                    world.name,
                );
                graded += 1;
            }
        }
    }
    assert_eq!(
        graded,
        crate::theme::THEMES
            .iter()
            .filter(|world| matches!(
                world.render_caps.title_style,
                theme::TitleStyle::Placard { .. }
            ))
            .count()
            * 4
    );
}
