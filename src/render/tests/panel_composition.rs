//! Composition laws for Settings, Themes, Find, and Replace. These assert the
//! authored grouping at the same shaped/hit-test seams the frame consumes.

use super::super::*;
use super::pixeldiff::{Region, diff_region, render_frame};
use super::{headless_dqp, view};

const W: u32 = 1200;
const H: u32 = 800;

/// The Settings separator exists only when both workspace regions are visible.
/// Shipped density intentionally stages this capped pane, so the composition
/// law uses the explicit compact-density counterfactual and restores it on
/// unwind rather than leaving a process-global override behind after a panic.
struct CompactDensity;

impl CompactDensity {
    fn install() -> Self {
        crate::render::overrides::set_overlay_density_test_override(Some(
            crate::render::overrides::TypeDensity {
                scale: 0.75,
                leading: 0.0,
            },
        ));
        Self
    }
}

impl Drop for CompactDensity {
    fn drop(&mut self) {
        crate::render::overrides::set_overlay_density_test_override(None);
    }
}

fn theme_view() -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_theme_picker = true;
    v.overlay_theme_chrome = Some(crate::theme::active_index());
    v.overlay_query_field = true;
    v.overlay_query_focused = true;
    v.overlay_query_placeholder = Some("Search themes".into());
    v.overlay_title = "themes".into();
    v.overlay_items = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
    v.overlay_bindings = vec![String::new(); v.overlay_items.len()];
    v.overlay_selected = crate::theme::active_index();
    v.overlay_window_rows = v.overlay_items.len();
    v.overlay_hint = crate::overlay::OverlayKind::Theme.hint();
    v
}

fn settings_view() -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_workspace = true;
    v.overlay_rows_primary = false;
    // This fixture grades Settings' rows composition. The production default
    // starts on Categories at shipped density, so enter the content stage.
    v.overlay_detail_focus = true;
    v.overlay_query_field = true;
    v.overlay_query_focused = false;
    v.overlay_query_placeholder = Some("Search settings".into());
    v.overlay_title = "settings".into();
    v.overlay_lens = vec![("All".into(), true), ("Writing".into(), false)];
    v.overlay_items = (0..31).map(|i| format!("Setting {i:02}")).collect();
    v.overlay_bindings = (0..31).map(|i| format!("value {i:02}")).collect();
    v.overlay_window_rows = 31;
    v.overlay_hint = crate::overlay::OverlayKind::Settings.hint();
    v
}

fn assert_settings_focus_regions(p: &mut TextPipeline, device: &wgpu::Device, queue: &wgpu::Queue) {
    let mut category = settings_view();
    category.overlay_detail_focus = false;
    p.set_view(&category);
    p.prepare(device, queue, W, H).unwrap();
    let categories = render_frame(p, device, queue, W, H);
    let rail = p
        .workspace_rail_mark_probe()
        .expect("active Settings category mark");
    let row = p.overlay_row_geometry().unwrap().rows[0];
    let mut search = settings_view();
    search.overlay_detail_focus = false;
    search.overlay_query_focused = true;
    p.set_view(&search);
    p.prepare(device, queue, W, H).unwrap();
    let search_pixels = render_frame(p, device, queue, W, H);
    let geom = p.overlay_geometry(W);
    let search_field = p
        .overlay_composition_quads(&geom, &p.overlay_row_plan(&geom))
        .0[0];
    let mut controls = settings_view();
    controls.overlay_rows_focused = true;
    controls.overlay_detail_focus = true;
    p.set_view(&controls);
    p.prepare(device, queue, W, H).unwrap();
    let control_pixels = render_frame(p, device, queue, W, H);
    for (name, a, b, region) in [
        (
            "category/search rail",
            &categories,
            &search_pixels,
            Region::new(rail[0], rail[1], rail[2], rail[3]),
        ),
        (
            "category/search field",
            &categories,
            &search_pixels,
            Region::new(
                search_field[0],
                search_field[1],
                search_field[2],
                search_field[3],
            ),
        ),
        (
            "category/controls row",
            &categories,
            &control_pixels,
            Region::new(row.x, row.y, row.w, row.h),
        ),
    ] {
        let diff = diff_region(a, b, W as i64, H as i64, region);
        assert!(
            diff.differing > 20 && diff.max_channel_delta >= 12,
            "{name} focus cue was not visible: {diff:?}"
        );
    }

    // Magpie's Diagonal workspace is deliberately bare: its spine and
    // connector read directly against the crisp room, rather than inventing a
    // Pane card for Settings. The no-frost claim belongs to that composition.
    crate::theme::set_active_by_name("Magpie").unwrap();
    p.sync_theme();
    p.set_view(&controls);
    p.prepare(device, queue, W, H).unwrap();
    assert_eq!(
        p.frost_mode(),
        None,
        "Magpie Settings must not blur the document"
    );
    assert_eq!(
        p.panel_card.instance_count(),
        0,
        "Magpie's Diagonal Settings must not grow a Pane card"
    );

    // Force the Pane composition for the same Settings workspace. Its
    // card is the real opaque backing owner, so prove it with a document A/B:
    // dense prose must reach the exact safe interior with the workspace absent,
    // then disappear completely when the one-surface workspace is present. A
    // mere instance count would stay green for a removed or transparent fill.
    crate::theme::set_active_by_name("Bowerbird").unwrap();
    p.sync_theme();
    crate::render::set_list_style_test_override(Some(crate::theme::ListStyle::Pane));
    let mut dense = settings_view();
    dense.overlay_rows_focused = true;
    dense.text = super::frost_feather::DENSE.into();
    p.set_view(&dense);
    p.prepare(device, queue, W, H).unwrap();
    let dense_workspace = render_frame(p, device, queue, W, H);
    let card = p.overlay_pane_fills_probe();
    assert_eq!(card.len(), 1, "Pane Settings owns one workspace backing");
    assert_eq!(
        p.panel_card.instance_count(),
        1,
        "Pane Settings uploads its one workspace backing through panel_card"
    );
    let card = card[0];
    let inset = p.overlay_card_opaque_inset_probe(card);
    let interior = Region::new(
        card[0] + inset,
        card[1] + inset,
        card[2] - 2.0 * inset,
        card[3] - 2.0 * inset,
    );

    let mut empty = settings_view();
    empty.overlay_rows_focused = true;
    p.set_view(&empty);
    p.prepare(device, queue, W, H).unwrap();
    let empty_workspace = render_frame(p, device, queue, W, H);

    let mut uncovered_dense = dense;
    uncovered_dense.overlay_active = false;
    p.set_view(&uncovered_dense);
    p.prepare(device, queue, W, H).unwrap();
    let bare_dense = render_frame(p, device, queue, W, H);
    let mut uncovered_empty = empty;
    uncovered_empty.overlay_active = false;
    p.set_view(&uncovered_empty);
    p.prepare(device, queue, W, H).unwrap();
    let bare_empty = render_frame(p, device, queue, W, H);
    let uncovered = diff_region(&bare_dense, &bare_empty, W as i64, H as i64, interior);
    assert_eq!(
        diff_region(
            &dense_workspace,
            &empty_workspace,
            W as i64,
            H as i64,
            interior
        )
        .differing,
        0,
        "Pane Settings must mask document changes throughout its opaque interior"
    );
    assert!(
        uncovered.differing > 100,
        "fixture must put document ink behind the workspace interior before opacity is graded: \
         {uncovered:?}"
    );
    crate::render::set_list_style_test_override(None);
}

#[test]
fn themes_is_one_surface_with_a_field_and_two_clickable_footer_actions() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _pin = crate::render::PickerChromePinRestore::capture();
    crate::theme::set_active_by_name("Bowerbird").unwrap();
    crate::render::pin_picker_chrome();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Themes composition law: no wgpu adapter");
        return;
    };
    p.set_view(&theme_view());
    p.prepare(&device, &queue, W, H).unwrap();

    assert_eq!(
        p.overlay_pane_fills_probe().len(),
        1,
        "Themes must be one card, not detached query and list plates"
    );
    assert_eq!(p.panel_buffer.lines[0].text(), "Themes   Search themes");
    let geom = p.overlay_geometry(W);
    let plan = p.overlay_row_plan(&geom);
    let (fills, borders) = p.overlay_composition_quads(&geom, &plan);
    assert_eq!(fills.len(), 3, "query field plus Switch and Cancel fills");
    assert_eq!(borders.len(), 12, "four hairlines around each control");
    assert_eq!(
        p.overlay_range_thumb.instance_count(),
        3,
        "the frame must upload all three control fills"
    );
    assert_eq!(
        p.overlay_range_track.instance_count(),
        12,
        "the frame must upload every control hairline"
    );

    let (switch, cancel) = p
        .theme_panel_action_report()
        .expect("Themes publishes its two action rectangles");
    assert!(
        switch[0] + switch[2] + 4.0 <= cancel[0],
        "Switch and Cancel must be visibly separate controls: {switch:?} {cancel:?}"
    );
    let (fill, border) = p.overlay_composition_inks();
    let chrome = crate::render::overlay_chrome_theme();
    assert_eq!(fill, chrome.base_300.rgba_bytes());
    assert_eq!(border, chrome.muted.rgba_bytes());
    let center = |[x, y, w, h]: [f32; 4]| (x + w * 0.5, y + h * 0.5);
    let (sx, sy) = center(switch);
    let (cx, cy) = center(cancel);
    assert_eq!(
        p.theme_panel_action_at(sx, sy),
        Some(crate::render::chrome::overlay_composition::ThemePanelAction::Switch)
    );
    assert_eq!(
        p.theme_panel_action_at(cx, cy),
        Some(crate::render::chrome::overlay_composition::ThemePanelAction::Cancel)
    );

    let before = render_frame(&mut p, &device, &queue, W, H);
    crate::theme::set_active_by_name("Mulga").unwrap();
    p.sync_theme();
    p.prepare(&device, &queue, W, H).unwrap();
    assert_eq!(p.overlay_geometry(W).card_probe(), geom.card_probe());
    assert_eq!(p.theme_panel_action_report(), Some((switch, cancel)));
    assert_eq!(p.overlay_composition_inks(), (fill, border));
    let after = render_frame(&mut p, &device, &queue, W, H);
    let actions = Region::new(
        switch[0],
        switch[1],
        cancel[0] + cancel[2] - switch[0],
        switch[3].max(cancel[3]),
    );
    assert_eq!(
        diff_region(&before, &after, W as i64, H as i64, actions).differing,
        0,
        "previewing a world must not recolor or reshape the chooser controls"
    );
}

#[test]
fn settings_title_field_and_region_separator_are_distinct() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _density = CompactDensity::install();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Settings composition law: no wgpu adapter");
        return;
    };
    let mut category = settings_view();
    category.overlay_detail_focus = false;
    p.set_view(&category);
    p.prepare(&device, &queue, W, H).unwrap();

    assert_eq!(p.panel_buffer.lines[0].text(), "Settings   Search settings");
    let geom = p.overlay_geometry(W);
    let plan = p.overlay_row_plan(&geom);
    let (fills, borders) = p.overlay_composition_quads(&geom, &plan);
    assert_eq!(fills.len(), 1, "Settings has one recognizable search field");
    assert_eq!(
        borders.len(),
        5,
        "field outline plus rail/content separator"
    );
    assert_eq!(
        p.overlay_range_thumb.instance_count(),
        1,
        "the frame must upload the Settings field fill"
    );
    assert_eq!(
        p.overlay_range_track.instance_count(),
        5,
        "the frame must upload the field outline and workspace separator"
    );
    let field = fills[0];
    let field_y = field[1] + field[3] * 0.5;
    assert_eq!(
        p.overlay_query_char_at(geom.text_left + 2.0, field_y),
        None,
        "the Settings title is not part of the search field"
    );
    assert_eq!(
        p.overlay_query_char_at(field[0] + field[2] * 0.5, field_y),
        Some(0),
        "the field itself remains clickable while Categories owns focus"
    );
    let (_, _, _, card_h, _) = p.overlay_window_report().unwrap();
    let (cue_above, cue_below) = p.overlay_edge_cue_report().unwrap();
    assert_eq!(cue_above, None);
    assert_eq!(cue_below, Some(31 - geom.visible_probe()));
    assert!(
        p.panel_buffer
            .lines
            .iter()
            .any(|line| line.text().contains("↓")),
        "the clipped Settings roster must disclose continuation"
    );
    let card = geom.card_probe();
    assert!(
        card[2] < W as f32 * 0.8,
        "wide Settings slab was not capped: {card:?}"
    );
    assert!((card[0] - (W as f32 - card[2]) * 0.5).abs() < 0.1);
    assert_eq!(card[3], card_h);
    assert_eq!(
        p.frost_mode(),
        None,
        "Settings must leave its framed document crisp"
    );

    assert_settings_focus_regions(&mut p, &device, &queue);
}

fn assert_workspace_continuation_scroll(
    p: &TextPipeline,
    name: &str,
) -> ([f32; 4], f32, f32, usize) {
    let geom = p.overlay_geometry(W);
    let card = geom.card_probe();
    let rows = p
        .overlay_row_geometry()
        .expect("workspace row report after prepare");
    let rail_top = p
        .workspace_rail_probe(W)
        .rows
        .first()
        .copied()
        .flatten()
        .expect("Settings rail row 0")[1];
    let (top, visible, _, _, _) = p
        .overlay_window_report()
        .expect("workspace sidecar window report");
    let sidecar_cues = p
        .overlay_edge_cue_report()
        .expect("workspace sidecar cue report");
    let shaped_cues = p.overlay_cue_lines(W);
    assert_eq!(visible, rows.rows.len(), "{name}: visible rows");
    assert!(top + visible <= 31, "{name}: window stays in corpus");
    assert_eq!(
        sidecar_cues.0.is_some(),
        shaped_cues.0.is_some(),
        "{name}: above cue"
    );
    assert_eq!(
        sidecar_cues.1.is_some(),
        shaped_cues.1.is_some(),
        "{name}: below cue"
    );

    let lines: Vec<&str> = p
        .panel_buffer
        .lines
        .iter()
        .map(|line| line.text())
        .collect();
    let footer = lines
        .get(lines.len().saturating_sub(3)..)
        .expect("three-line continuation footer");
    let footer_cue_lines = [lines.len() - 2, lines.len() - 1];
    assert_eq!(footer[0], "", "{name}: separator line");
    assert_eq!(
        footer[1],
        sidecar_cues.0.map_or_else(
            || " ".into(),
            |n| crate::render::chrome::edge_cue_text(true, n),
        ),
        "{name}: fixed above-edge slot"
    );
    assert_eq!(
        footer[2],
        sidecar_cues.1.map_or_else(
            || " ".into(),
            |n| crate::render::chrome::edge_cue_text(false, n),
        ),
        "{name}: fixed below-edge slot"
    );
    if name == "middle" {
        let (above, below) = sidecar_cues;
        assert_eq!(above, Some(top));
        assert_eq!(below, Some(31 - top - visible));
        for cue in [footer[1], footer[2]] {
            assert_eq!(
                lines.iter().filter(|line| **line == cue).count(),
                1,
                "middle scroll shapes each footer cue exactly once"
            );
        }
        assert!(
            shaped_cues.0.is_some() && shaped_cues.1.is_some(),
            "middle scroll shapes both cue texts"
        );
        for line_i in footer_cue_lines {
            let run = p
                .panel_buffer
                .layout_runs()
                .find(|run| run.line_i == line_i)
                .expect("a shaped cue has a layout run");
            let bottom = geom.text_top + run.line_top + run.line_height;
            assert!(
                bottom <= card[1] + card[3] + 0.01,
                "middle cue line {line_i} escapes the workspace card: {bottom} > {}",
                card[1] + card[3]
            );
        }
    }
    (card, rail_top, rows.first_top, visible)
}

#[test]
fn workspace_continuation_footer_is_three_shaped_lines_at_every_scroll_position() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _density = CompactDensity::install();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping workspace continuation footer law: no wgpu adapter");
        return;
    };
    let mut stable: Option<([f32; 4], f32, f32, usize)> = None;
    for (name, selected, scroll) in [("top", 0, 0), ("middle", 15, 8), ("bottom", 30, 30)] {
        let mut v = settings_view();
        v.overlay_selected = selected;
        v.overlay_scroll = scroll;
        p.set_view(&v);
        p.prepare(&device, &queue, W, H).unwrap();
        assert!(
            p.workspace_is_wide(W),
            "{name}: the continuation fixture requires the compact two-region Settings composition"
        );
        let split = p.workspace_rail_probe(W);
        assert!(
            split.rail.is_some() && split.visible > 0,
            "{name}: the compact fixture must show both its rail and rows"
        );
        let now = assert_workspace_continuation_scroll(&p, name);
        match stable {
            None => stable = Some(now),
            Some(first) => assert_eq!(
                now, first,
                "{name}: scrolling may change only the window and cue contents, \
                 not card/rail/row origins"
            ),
        }
    }
}

fn assert_search_composition(p: &TextPipeline, replace: bool) {
    use crate::render::chrome::PanelHit;

    let geometry = p.panel_geometry().expect("active panel geometry");
    let [x, y, w, h] = geometry.card;
    assert!((420.0..=480.0).contains(&w), "compact search card: {w}");
    let mut expected = vec![
        ("find_field", PanelHit::Find),
        ("close", PanelHit::Close),
        ("nav_prev", PanelHit::NavPrev),
        ("nav_next", PanelHit::NavNext),
        ("case_toggle", PanelHit::CaseToggle),
        ("reveal_replace", PanelHit::RevealReplace),
    ];
    if replace {
        expected.extend([
            ("replace_field", PanelHit::Replace),
            ("replace_button", PanelHit::ReplaceButton),
            ("replace_all_button", PanelHit::ReplaceAllButton),
        ]);
    }
    assert_eq!(geometry.controls.len(), expected.len(), "control roster");
    for (name, hit) in expected {
        let control = geometry
            .controls
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing {name}"));
        let [cx, cy, cw, ch] = control.rect;
        assert!(cw > 0.0 && ch > 0.0, "{name}: empty target");
        assert!(
            cx >= x - 0.5 && cy >= y - 0.5 && cx + cw <= x + w + 0.5 && cy + ch <= y + h + 0.5,
            "{name} leaves its card: {:?} in {:?}",
            control.rect,
            geometry.card
        );
        assert_eq!(
            p.panel_hit(cx + cw * 0.5, cy + ch * 0.5),
            Some(hit),
            "{name}"
        );
    }
    for (i, a) in geometry.controls.iter().enumerate() {
        for b in &geometry.controls[i + 1..] {
            let overlap_x =
                (a.rect[0] + a.rect[2]).min(b.rect[0] + b.rect[2]) - a.rect[0].max(b.rect[0]);
            let overlap_y =
                (a.rect[1] + a.rect[3]).min(b.rect[1] + b.rect[3]) - a.rect[1].max(b.rect[1]);
            assert!(
                overlap_x <= 0.5 || overlap_y <= 0.5,
                "{} and {} targets overlap: {:?} / {:?}",
                a.name,
                b.name,
                a.rect,
                b.rect
            );
        }
    }
    let text = p
        .panel_buffer
        .lines
        .iter()
        .map(|line| line.text())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("Match case"),
        "checkbox label missing: {text:?}"
    );
    assert!(
        text.contains(&crate::keyspec::PANEL_CLOSE.label()),
        "close shortcut hint missing: {text:?}"
    );
    assert!(text.contains("Replace"), "disclosure caption is present");
}

#[test]
fn find_and_replace_compose_distinct_clickable_controls_across_worlds() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Find/Replace composition law: no wgpu adapter");
        return;
    };
    let mut v = view("quiet prose\n", 0, 0);
    v.search_active = true;
    v.search_query = "quiet".into();
    v.search_query_caret = 5;
    v.search_matches = vec![((0, 0), (0, 5))];
    v.search_current = Some(0);
    v.search_replacement = "peace".into();
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for replace in [false, true] {
            v.search_replace_active = replace;
            p.set_view(&v);
            p.prepare(&device, &queue, W, H).unwrap();
            assert_search_composition(&p, replace);
        }
    }
}
