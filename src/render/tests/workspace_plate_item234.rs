//! ITEM 234 — A WORKSPACE'S ROW TEXT SAT OUTSIDE ITS OWN PLATE.
//!
//! Both other overlay families lay their row text out `overlay_text_hpad()`
//! inside the band the row surfaces span, and on a `Bars` world that number is
//! `BAR_SIDE_INSET + BAR_TEXT_PAD` for one reason: `bar_full_span` insets the
//! plate `BAR_SIDE_INSET` from the same band, so the leftover `BAR_TEXT_PAD` is
//! the air between a plate's edge and the glyphs it backs. The WORKSPACE family
//! laid its rows out on the bare band, which put the text `BAR_SIDE_INSET`
//! OUTSIDE its own plate at BOTH edges — the first glyph of every row label cut
//! by the plate's left edge, and the right-aligned VALUE hanging past its right
//! one. That right-hand half is the reported "Block" plate cutting its final
//! `k` on Cassowary; it is the same 8px on every world the plates are visible
//! on, and the left-hand half was on all twenty.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};

fn values() -> crate::settings::SettingsValues {
    crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.0,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    }
}

/// A REAL Settings workspace at lens `lens`, folded the way `App::sync_view`
/// folds one — every `SettingId` with its real `SettingKind`'s value cell, so
/// the value column under test is the product's own, not a fixture's.
fn settings_view(lens: usize) -> ViewState {
    let vals = values();
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov.set_facet_lens(lens);
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_workspace = true;
    v.overlay_rows_primary = false;
    v.overlay_title = OverlayKind::Settings.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_hint = "↑/↓ category   ↵ settings   esc close".into();
    v.overlay_selected = ov.selected;
    v
}

/// **THE ROW TEXT SITS INSIDE ITS OWN PLATE.** Swept over the whole world
/// roster (the reported world is one of twenty, and the two neighbouring items
/// this one was bundled with were both reported world-specific and both turned
/// out universal), every `SettingId` category lens, and the widths where the
/// clip bites — including the narrowest the workspace still draws both regions
/// at.
///
/// THREE ARMS:
///  1. THE BOUND, from the geometry owner: the row box must clear the plate's
///     own span by `BAR_TEXT_PAD` at both edges, exactly as a contextual card's
///     does. On a `Pane` world there is no plate and the same `overlay_text_hpad`
///     owner supplies that world's own pad, so the bound is stated once.
///  2. NON-VACUITY, the retired rule written out inline: the row box WAS the
///     bare band, and that box overruns the plate at both edges by
///     `BAR_SIDE_INSET` — the measured 8px of the report.
///  3. THE PIXELS, on the reported symptom: no ink of the value column may fall
///     outside the plate that backs it.
#[test]
fn a_workspace_rows_text_sits_inside_its_own_plate_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping workspace plate law: no wgpu adapter");
        return;
    };
    let lenses = crate::facets::scheme(OverlayKind::Settings)
        .expect("Settings facets")
        .strip
        .len();
    assert!(lenses >= 7, "the Settings category roster shrank: {lenses}");

    let mut retired_overrun = 0.0f32;
    let mut graded = 0usize;
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let bars = matches!(
            theme::active().render_caps.list_style,
            theme::ListStyle::Bars { .. }
        );
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for logical_w in [900.0f32, 1200.0, 1600.0] {
                let (cw, ch) = (
                    (logical_w * dpi).round() as u32,
                    (800.0 * dpi).round() as u32,
                );
                p.set_size(cw as f32, ch as f32);
                for lens in 0..lenses {
                    let v = settings_view(lens);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let geom = p.overlay_geometry(cw);
                    if geom.visible_probe() == 0 {
                        continue;
                    }
                    let ctx = format!("{world}@{dpi}x{logical_w}/lens{lens}");
                    let hpad = p.overlay_text_hpad();
                    let (bar_x, bar_w) = crate::render::chrome::bar_full_span_probe(
                        geom.band_x_probe(),
                        geom.band_w_probe(),
                    );
                    let pad = hpad - (bar_x - geom.band_x_probe());

                    // --- ARM 1: THE BOUND -----------------------------------
                    assert!(
                        geom.text_left >= bar_x + pad - 0.51,
                        "{ctx}: the row text starts at {:.1}, left of its own plate's \
                         {:.1} + its pad {pad:.1}",
                        geom.text_left,
                        bar_x
                    );
                    assert!(
                        geom.text_left + geom.text_w <= bar_x + bar_w - pad + 0.51,
                        "{ctx}: the row text ends at {:.1}, right of its own plate's \
                         {:.1} less its pad {pad:.1} — this is the value hanging past \
                         the plate that backs it",
                        geom.text_left + geom.text_w,
                        bar_x + bar_w
                    );

                    // --- ARM 2: THE RETIRED RULE ----------------------------
                    // `text_left`/`text_w` used to BE the band, with no pad at all.
                    let (retired_left, retired_w) = (geom.band_x_probe(), geom.band_w_probe());
                    let overrun =
                        (bar_x - retired_left).max(retired_left + retired_w - (bar_x + bar_w));
                    retired_overrun = retired_overrun.max(overrun / dpi);
                    assert!(
                        overrun > 1.0,
                        "{ctx}: the retired rule no longer overruns the plate — this cell \
                         cannot witness the defect"
                    );
                    if bars {
                        graded += 1;
                    }
                }
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded >= 100,
        "the bound graded only {graded} plate-bearing cells"
    );
    assert!(
        (retired_overrun - 8.0).abs() < 0.51,
        "the retired rule's overrun measured {retired_overrun:.2} logical px; the report \
         and the code both say it is BAR_SIDE_INSET (8.0)"
    );
}

/// **THE REPORTED SYMPTOM, IN PIXELS.** Cassowary's Caret style row shows the
/// widest value the Settings roster carries at its default (`Block`), and its
/// plate is the one that cut the final `k`. Every ink column of that value must
/// now lie inside the plate that backs it.
///
/// THE ROSTER IS THE WORLDS THAT DRAW PLATES, WHICH IS NOT THE BARE-PLATE
/// ROSTER. `list_backing == BarePlates` has five members and two of them —
/// Mangrove and Magpie — are `ListStyle::Diagonal`, which draws a spine and no
/// plate at all; `overlay_bar_rects_probe` SYNTHESIZES bar rects for them at
/// invented dials so other laws can reason about a row's span. Grading real ink
/// against a rect that is never drawn measures nothing, and reports a clip on a
/// world with no plate to clip against.
#[test]
fn the_widest_settings_value_draws_no_ink_outside_its_plate_on_any_plated_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping workspace value-ink law: no wgpu adapter");
        return;
    };
    let bare: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.list_style, theme::ListStyle::Bars { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(bare, ["Galah", "Firetail", "Cassowary"]);

    let luma = |c: [u8; 4]| 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
    let mut graded: Vec<String> = Vec::new();
    for world in &bare {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        p.set_size(1200.0, 800.0);
        p.set_view(&settings_view(0));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        let Some(first) = plan.rows().iter().find(|r| r.item == Some(0)) else {
            continue;
        };
        let (sel, unsel) = p.overlay_bar_rects_probe();
        // The VALUE plate on this row: the right-most plate whose band is the row's.
        let Some(value_plate) = sel
            .iter()
            .chain(unsel.iter())
            .copied()
            .filter(|r| (r[1] - first.top).abs() < first.height)
            .filter(|r| r[0] > geom.band_x_probe() + geom.band_w_probe() * 0.5)
            .max_by(|a, b| a[0].total_cmp(&b[0]))
        else {
            continue;
        };
        let (texture, tview) = super::dither::offscreen(&device, 1200, 800);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl item234 value ink encoder"),
        });
        p.render(&mut encoder, &tview).unwrap();
        queue.submit(Some(encoder.finish()));
        let pixels = super::dither::read_pixels(&device, &queue, &texture, 1200, 800);

        let y0 = value_plate[1].round().max(0.0) as usize;
        let y1 = (value_plate[1] + value_plate[3]).round().min(799.0) as usize;
        let plate_right = value_plate[0] + value_plate[2];
        // Card ground just beyond the plate, on the same rows.
        let gx = (plate_right + 14.0).round().min(1199.0) as usize;
        let ground: f32 = ((y0 + y1) / 2..(y0 + y1) / 2 + 1)
            .map(|y| luma(pixels[y * 1200 + gx]))
            .sum();
        let mut outside = Vec::new();
        for x in (plate_right.ceil() as usize + 1)..(plate_right as usize + 13).min(1199) {
            for y in y0..y1 {
                if (luma(pixels[y * 1200 + x]) - ground).abs() > 24.0 {
                    outside.push(x);
                    break;
                }
            }
        }
        assert!(
            outside.is_empty(),
            "{world}: the value's ink reaches columns {outside:?}, outside its own plate \
             which ends at {plate_right:.1} — this is the reported clipped glyph"
        );
        graded.push((*world).to_string());
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        graded.len(),
        3,
        "the value-ink law must grade every plate-drawing world: {graded:?}"
    );
}
