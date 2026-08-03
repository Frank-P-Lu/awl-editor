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
/// TWO ARMS:
///  1. THE BOUND, from the geometry owner: the row box must clear the plate's
///     own span by `BAR_TEXT_PAD` at both edges, exactly as a contextual card's
///     does. `Pane` and `Diagonal` worlds draw no plate (item 236 — the
///     bare-plate roster is not the plate-drawing roster, and Mangrove and
///     Magpie are `Diagonal`), and there the same claim degenerates to the pure
///     one it always was: `text_left` clears the BAND by `overlay_text_hpad`,
///     the one owner of that pad on every world. `bar_full_span` is arithmetic
///     over the band, not emitter output, so no cell here grades a synthesized
///     plate — but only the plate-bearing cells count toward `graded`.
///  2. NON-VACUITY, the retired rule written out inline: the row box WAS the
///     bare band, and that box overruns the plate at both edges by
///     `BAR_SIDE_INSET` — the measured 8px of the report. **This arm is weaker
///     than it reads**: `bar_full_span` is `(band_x + INSET, band_w - 2*INSET)`,
///     so the overrun it measures is `BAR_SIDE_INSET` identically, in every
///     cell, with no world, width, lens or DPI entering it. It pins the
///     report's 8px against the constant and nothing else; arm 1 is what
///     actually sweeps.
///
/// The pixel evidence for this claim is the sibling law below, which grades the
/// shaper's own glyph runs against the emitter's own quads on the worlds that
/// really draw plates.
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

/// **THE REPORTED SYMPTOM, PER ROW, IN THE SHAPER'S OWN GLYPH WIDTHS.**
/// Cassowary's Caret style row shows `Block` and its plate is the one that cut
/// the final `k`. This is that claim for EVERY `SettingId` of every category, at
/// the widths where it bites, on every world that draws plates: the value run
/// the shaper laid out must sit inside the plate the frame drew behind it, with
/// `BAR_TEXT_PAD` of air at each edge.
///
/// THE ORACLE IS THE EMITTER'S OWN QUADS, not the geometry under test: the
/// plates come from `overlay_bar_rects_probe`, which runs the real
/// `overlay_bar_selection`/`append_chord_plates` emitters, and the claim is the
/// one that broke — an accessory plate's right edge must REACH the right-aligned
/// value column it backs, plus its pad. Before the fix that edge was clamped to
/// `bar_full_span`'s inset and landed `BAR_SIDE_INSET` SHORT of the column.
///
/// AN ABSOLUTE PIXEL SCAVENGE WAS TRIED FIRST AND RETIRED: "ink near the plate"
/// cannot tell a value's glyph from a rail, a card edge or a neighbouring
/// descender, and it reported a clip on Galah under the full suite that no
/// value's glyph had caused. The pixel evidence for this round is the
/// before/after capture matrix and the live-`App` shot, not a scavenging oracle.
///
/// THE ROSTER IS THE WORLDS THAT DRAW PLATES, WHICH IS NOT THE BARE-PLATE
/// ROSTER. `list_backing == BarePlates` has five members and two of them —
/// Mangrove and Magpie — are `ListStyle::Diagonal`, which draws a spine and no
/// plate at all. `overlay_bar_rects_probe` used to SYNTHESIZE rects for them at
/// invented dials; item 236 made it refuse, so the substitution this comment
/// warned about is now a panic rather than a warning.
#[test]
fn every_settings_value_sits_inside_its_own_plate_on_every_plated_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping workspace value-plate law: no wgpu adapter");
        return;
    };
    let plated: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.list_style, theme::ListStyle::Bars { .. }))
        .map(|t| t.name)
        .collect();
    assert_eq!(plated, ["Galah", "Firetail", "Cassowary"]);
    let lenses = crate::facets::scheme(OverlayKind::Settings)
        .expect("Settings facets")
        .strip
        .len();

    let mut graded = 0usize;
    let mut tightest = f32::MAX;
    for world in &plated {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for logical_w in [900.0f32, 1200.0, 1600.0] {
            p.set_size(logical_w, 800.0);
            let cw = logical_w as u32;
            for lens in 0..lenses {
                p.set_view(&settings_view(lens));
                p.prepare(&device, &queue, cw, 800).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let (sel, unsel) = p.overlay_bar_rects_probe();
                let plates: Vec<[f32; 4]> = sel.into_iter().chain(unsel).collect();
                let column_right = geom.text_left + geom.text_w;
                let band_right = geom.band_x_probe() + geom.band_w_probe();
                for row in plan.rows().iter().filter(|r| r.item.is_some()) {
                    // The ACCESSORY plate on this row: the one in the band's right
                    // half, which is the value column's own.
                    let Some(plate) = plates
                        .iter()
                        .copied()
                        .filter(|r| (r[1] - row.top).abs() < row.height)
                        .filter(|r| r[0] > geom.band_x_probe() + geom.band_w_probe() * 0.5)
                        .max_by(|a, b| a[0].total_cmp(&b[0]))
                    else {
                        continue;
                    };
                    let ctx = format!("{world}@{logical_w}/lens{lens}/row{}", row.display);
                    let reach = plate[0] + plate[2] - column_right;
                    tightest = tightest.min(reach);
                    assert!(
                        reach >= crate::render::chrome::BAR_TEXT_PAD - 0.51,
                        "{ctx}: the value column's right edge is {column_right:.1} and its \
                         plate ends at {:.1} — {reach:.1}px where the plate owes it {:.1}. \
                         This is the plate measured against the wrong right bound, and it \
                         is the reported clipped glyph.",
                        plate[0] + plate[2],
                        crate::render::chrome::BAR_TEXT_PAD
                    );
                    assert!(
                        plate[0] + plate[2] <= band_right + 0.51,
                        "{ctx}: the plate runs past the content band's own edge \
                         ({:.1} > {band_right:.1})",
                        plate[0] + plate[2]
                    );
                    graded += 1;
                }
            }
        }
    }
    p.set_size(1200.0, 800.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded >= 300,
        "the value-plate law graded only {graded} rows — it must reach every \
         SettingId of every category at every swept width"
    );
    assert!(
        tightest.is_finite(),
        "nothing was measured; tightest clearance {tightest}"
    );
}
