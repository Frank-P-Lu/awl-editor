//! Composition laws for Settings, Themes, Find, and Replace. These assert the
//! authored grouping at the same shaped/hit-test seams the frame consumes.

use super::super::*;
use super::{headless_dqp, view};

const W: u32 = 1200;
const H: u32 = 800;

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
    v.overlay_workspace = true;
    v.overlay_rows_primary = false;
    v.overlay_query_field = true;
    v.overlay_query_focused = false;
    v.overlay_query_placeholder = Some("Search settings".into());
    v.overlay_title = "settings".into();
    v.overlay_lens = vec![("All".into(), true), ("Writing".into(), false)];
    v.overlay_items = vec!["Caret style".into(), "Page mode".into()];
    v.overlay_bindings = vec!["Block".into(), "on".into()];
    v.overlay_hint = crate::overlay::OverlayKind::Settings.hint();
    v
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
        .theme_panel_action_rects(&geom)
        .expect("Themes publishes its two action rectangles");
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
}

#[test]
fn settings_title_field_and_region_separator_are_distinct() {
    let _guard = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Settings composition law: no wgpu adapter");
        return;
    };
    p.set_view(&settings_view());
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
}

#[test]
fn find_and_replace_group_controls_without_inline_chord_clutter() {
    let _guard = crate::testlock::serial();
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

    for replace in [false, true] {
        v.search_replace_active = replace;
        v.search_replacement = "peace".into();
        p.set_view(&v);
        p.prepare(&device, &queue, W, H).unwrap();
        let lines: Vec<String> = p
            .panel_buffer
            .lines
            .iter()
            .map(|line| line.text().to_string())
            .collect();
        let text = lines.join("\n");
        assert!(
            !text.contains("Aa"),
            "redundant Aa marker returned: {text:?}"
        );
        assert!(
            !text.contains("Tab"),
            "inline field hint returned: {text:?}"
        );
        assert_eq!(lines.last().unwrap(), "Esc close");

        let geometry = p.panel_geometry().expect("active panel geometry");
        assert!(geometry.card[2] >= p.metrics.panel_ui().px(crate::render::chrome::PANEL_MIN_W));
        assert!(
            geometry.controls.iter().any(|c| c.name == "case_toggle"),
            "Match case must keep a visible/clickable checkbox"
        );
        assert_eq!(
            geometry.controls.iter().any(|c| c.name == "replace_button"),
            replace
        );
        assert_eq!(
            geometry
                .controls
                .iter()
                .any(|c| c.name == "replace_all_button"),
            replace
        );
    }
}
