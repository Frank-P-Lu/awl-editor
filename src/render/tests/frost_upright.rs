//! UPRIGHT PLATE-HUGGING FROST — tightness against an independent subject.
//!
//! `overlay_drawn_surfaces` once called the full Bars pointer band a drawn surface, so
//! the old tightness law agreed perfectly with a 545 px frost over ~397 px of plates.
//! These laws derive enrolment from the roster, tightness from the row emitter plus
//! frost-free pixels, and blur presence from a normal/suppressed pixel pair.

use super::super::*;
use super::frost_card_ink::luma;
use super::frost_feather::{DENSE, render_frame, theme_picker};
use super::{headless_dqp, view_md};

const CARD_INK_DELTA: f32 = 3.0;
const PIXEL_CONTAINMENT_ALLOWANCE_PX: f32 = 4.0;
/// 24 px of selection-independent ledge plus 4 px of raster disagreement.
const UPPER_ALLOWANCE_PX: f32 = 28.0;
const FROST_PRESENCE_DELTA: f32 = 1.5;

fn typed_picker(text: &str) -> ViewState {
    let mut v = theme_picker(text);
    v.overlay_query = "mangrove".to_string();
    v.overlay_query_caret = v.overlay_query.chars().count();
    v
}

fn pointer_menu(text: &str, dpi: f32) -> ViewState {
    let state = crate::context_menu::ContextState {
        has_selection: true,
        link: false,
        heading: false,
        heading_folded: false,
        misspelled: false,
        named_file: true,
    };
    let rows = crate::context_menu::rows(
        crate::context_menu::ContextTarget::Selection,
        state,
        crate::commands::Platform::Native,
    );
    let mut v = view_md(text, 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Context.title().to_string();
    v.overlay_items = rows.iter().map(|r| r.label.to_string()).collect();
    v.overlay_bindings = vec![String::new(); rows.len()];
    v.overlay_selected = 0;
    v.overlay_context_anchor = Some((300.0 * dpi, 260.0 * dpi));
    v
}

fn footprint_rect(frost: crate::render::blur::Frost, label: &str) -> [f32; 4] {
    match frost {
        crate::render::blur::Frost::Footprint(f) => f.rect,
        other => panic!("{label}: expected the footprint arm, got {other:?}"),
    }
}

/// Grade one real card against the row emitter, independent of the frost's surface
/// census. Pixels independently prove containment and non-vanishing blur.
fn grade_hugged_card(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    geometry: (u32, u32, f32),
    view: ViewState,
    label: &str,
) -> ([f32; 4], [f32; 4], u64) {
    let (w, h, dpi) = geometry;
    p.set_view(&view);
    let frosted = render_frame(device, queue, p, w, h);
    let rect = footprint_rect(p.frost_mode().expect("the Bars footprint"), label);
    let card = p.overlay_card_rect().expect("the open overlay card");
    let plates = p.overlay_row_surfaces_probe();
    assert!(!plates.is_empty(), "{label}: no Bars plate was emitted");
    let (mut sl, mut st, mut sr, mut sb) = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    for [x, y, width, height] in plates {
        sl = sl.min(x);
        st = st.min(y);
        sr = sr.max(x + width);
        sb = sb.max(y + height);
    }
    let surface = [sl, st, sr - sl, sb - st];

    crate::render::blur::set_frost_suppressed(true);
    p.set_view(&view);
    let open = render_frame(device, queue, p, w, h);
    let placard =
        p.overlay_shape_placard(&p.overlay_geometry(w))
            .unwrap_or((f32::NAN, f32::NAN, 0.0, 0.0));
    let mut shut_view = view;
    shut_view.overlay_active = false;
    p.set_view(&shut_view);
    let shut = render_frame(device, queue, p, w, h);
    crate::render::blur::set_frost_suppressed(false);

    let (mut left, mut top, mut right, mut bottom) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    let mut ink = vec![false; (w * h) as usize];
    let mut ink_count = 0u64;
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let i = (y * w as i64 + x) as usize;
            if (luma(open[i]) - luma(shut[i])).abs() < CARD_INK_DELTA {
                continue;
            }
            let (fx, fy) = (x as f32, y as f32);
            if fx < card[0] || fx > card[0] + card[2] || fy < card[1] || fy > card[1] + card[3] {
                continue;
            }
            if fx >= placard.0
                && fx <= placard.0 + placard.2
                && fy >= placard.1
                && fy <= placard.1 + placard.3
            {
                continue;
            }
            ink[i] = true;
            ink_count += 1;
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
    }
    assert!(
        ink_count > 2_000 && right > left && bottom > top,
        "{label}: only {ink_count} card-ink pixels — the bounds are vacuous"
    );
    let pixel = [
        left as f32,
        top as f32,
        (right - left + 1) as f32,
        (bottom - top + 1) as f32,
    ];

    let mut presence = 0u64;
    for y in rect[1].floor().max(0.0) as i64..(rect[1] + rect[3]).ceil().min(h as f32) as i64 {
        for x in rect[0].floor().max(0.0) as i64..(rect[0] + rect[2]).ceil().min(w as f32) as i64 {
            let i = (y * w as i64 + x) as usize;
            if !ink[i] && (luma(frosted[i]) - luma(open[i])).abs() >= FROST_PRESENCE_DELTA {
                presence += 1;
            }
        }
    }

    let allowance = PIXEL_CONTAINMENT_ALLOWANCE_PX * dpi;
    let (fr, fb) = (rect[0] + rect[2], rect[1] + rect[3]);
    let (pr, pb) = (pixel[0] + pixel[2], pixel[1] + pixel[3]);
    assert!(
        pixel[0] >= rect[0] - allowance
            && pixel[1] >= rect[1] - allowance
            && pr <= fr + allowance
            && pb <= fb + allowance,
        "{label}: pixel card ink {pixel:?} escapes frost core {rect:?}"
    );

    // The emitter owns the often-faint footer plate; pixels own upright query/header
    // ink above the first plate. Their union is independent of the frost census.
    let expected_left = surface[0].min(pixel[0]);
    let expected_top = surface[1].min(pixel[1]);
    let expected_right = (surface[0] + surface[2]).max(pixel[0] + pixel[2]);
    let expected_bottom = (surface[1] + surface[3]).max(pixel[1] + pixel[3]);
    let expected = [
        expected_left,
        expected_top,
        expected_right - expected_left,
        expected_bottom - expected_top,
    ];
    let excess_x = (rect[2] - expected[2]) / dpi;
    let excess_y = (rect[3] - expected[3]) / dpi;
    assert!(
        excess_x <= UPPER_ALLOWANCE_PX && excess_y <= UPPER_ALLOWANCE_PX,
        "{label}: hugged Bars frost {rect:?} exceeds the independent plate/pixel union \
         {expected:?} by ({excess_x:.1}, {excess_y:.1}) logical px; this is the loose \
         interaction/card box, not the drawn plate composition"
    );
    assert!(
        presence > (500.0 * dpi * dpi) as u64,
        "{label}: only {presence} non-card pixels changed by at least \
         {FROST_PRESENCE_DELTA} luma — a vanishing frost satisfies tightness"
    );
    (rect, expected, presence)
}

/// Roster-derived plate carriers × 1×/2× × both menu-bar states × both geometry
/// families. The upper bound and the presence floor are a pair.
#[test]
fn upright_plate_hugging_frost_tracks_drawn_surfaces_in_both_geometry_families() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();
    let worlds: Vec<&str> = crate::theme::THEMES
        .iter()
        .filter(|t| {
            t.render_caps.list_style.draws_row_plates()
                && crate::theme::BarConfig::SHIPPED.extent.hugs()
        })
        .map(|t| t.name)
        .collect();
    assert!(!worlds.is_empty(), "no hugged plate composition enrolled");

    let mut families = std::collections::BTreeSet::new();
    let mut worst = (f32::NEG_INFINITY, f32::NEG_INFINITY, String::new());
    for world in worlds {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    crate::theme::set_active(entry);
                    return;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                for (family, view) in [
                    ("picker", typed_picker(DENSE)),
                    ("pointer-menu", pointer_menu(DENSE, dpi)),
                ] {
                    let label = format!("{world}/{family} @ {dpi}x bar {bar}");
                    let (frost, drawn, presence) =
                        grade_hugged_card(&device, &queue, &mut p, (w, h, dpi), view, &label);
                    let excess = ((frost[2] - drawn[2]) / dpi, (frost[3] - drawn[3]) / dpi);
                    if excess.0.max(excess.1) > worst.0.max(worst.1) {
                        worst = (excess.0, excess.1, label.clone());
                    }
                    eprintln!(
                        "MEASURED {label}: frost {:.1}×{:.1}, drawn union {:.1}×{:.1}, \
                         excess {:.1}×{:.1}, {presence} presence pixels",
                        frost[2] / dpi,
                        frost[3] / dpi,
                        drawn[2] / dpi,
                        drawn[3] / dpi,
                        excess.0,
                        excess.1,
                    );
                    families.insert(family);
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(entry);
    assert_eq!(
        families,
        std::collections::BTreeSet::from(["picker", "pointer-menu"])
    );
    eprintln!(
        "ROSTER WORST upright excess {:.1}×{:.1} logical at {}",
        worst.0, worst.1, worst.2
    );
}

/// Whole roster × both card families, plus the FullWidth Bars control. Enrolment comes
/// from each world's composition; no world name stands in for Pane/Bars/Ruled/Diagonal.
#[test]
fn every_composition_keeps_its_picker_and_pointer_menu_footprint_contract() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let (w, h) = (1200u32, 900u32);
    let mut styles = std::collections::BTreeSet::new();
    let mut bars_world = None;
    for t in crate::theme::THEMES {
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            crate::theme::set_active(entry);
            return;
        };
        crate::theme::set_active_by_name(t.name).unwrap();
        let style = t.render_caps.list_style;
        let style_name = match style {
            crate::theme::ListStyle::Pane => "Pane",
            crate::theme::ListStyle::Bars => {
                bars_world.get_or_insert(t.name);
                "Bars"
            }
            crate::theme::ListStyle::Ruled(_) => "Ruled",
            crate::theme::ListStyle::Diagonal(_) => "Diagonal",
        };
        styles.insert(style_name);
        for (family, view) in [
            ("picker", typed_picker(DENSE)),
            ("pointer-menu", pointer_menu(DENSE, 1.0)),
        ] {
            p.set_view(&view);
            let _ = render_frame(&device, &queue, &mut p, w, h);
            let frost = p.frost_mode();
            let excluded = t.render_caps.backdrop == crate::theme::Backdrop::Flat
                || matches!(style, crate::theme::ListStyle::Pane);
            assert_eq!(
                frost.is_none(),
                excluded,
                "{}/{family}: {style_name} routing disagrees with its roster composition",
                t.name
            );
        }
    }
    assert_eq!(
        styles,
        std::collections::BTreeSet::from(["Bars", "Diagonal", "Pane", "Ruled"])
    );

    let bars_world = bars_world.expect("the roster has a Bars carrier");
    crate::theme::set_active_by_name(bars_world).unwrap();
    crate::render::set_bar_config_test_override(Some(crate::theme::BarConfig {
        extent: crate::theme::BarExtent::FullWidth,
        ..crate::theme::BarConfig::SHIPPED
    }));
    for (family, view) in [
        ("picker", typed_picker(DENSE)),
        ("pointer-menu", pointer_menu(DENSE, 1.0)),
    ] {
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            crate::render::set_bar_config_test_override(None);
            crate::theme::set_active(entry);
            return;
        };
        p.set_view(&view);
        let _ = render_frame(&device, &queue, &mut p, w, h);
        let card = p.overlay_card_rect().expect("the Bars card");
        let frost = footprint_rect(p.frost_mode().expect("the Bars footprint"), family);
        assert!(
            (card[2] - frost[2]).abs() <= 16.0 && frost[1] == card[1] && frost[3] == card[3],
            "{bars_world}/{family}: FullWidth Bars must keep the band and vertical card; \
             got frost {frost:?}, card {card:?}"
        );
    }
    crate::render::set_bar_config_test_override(None);
    crate::theme::set_active(entry);
}
