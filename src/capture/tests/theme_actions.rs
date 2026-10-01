//! `overlay.theme_actions` is the public form of the Theme card's two action
//! hit targets. This law crosses the real render + serializer boundary so the
//! sidecar cannot publish rects for a card that did not shape them, omit either
//! rect from Theme, or leave stale rects behind after the overlay changes.

use super::super::*;
use super::{adapter_available, sidecar};
use crate::buffer::Buffer;
use crate::overlay::{OverlayKind, OverlayState};
use crate::testscratch::ScratchDir;

fn overlay_opts(ov: &OverlayState) -> CaptureOpts {
    let mut opts = CaptureOpts {
        ..CaptureOpts::default()
    };
    opts.overlay = Some(OverlayInfo {
        align: crate::render::effective_card_anchor(),
        chrome_theme: None,
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title().to_string(),
        query: ov.query.text().to_string(),
        query_caret: ov.query.caret(),
        query_selection: None,
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        match_highlights: ov.item_match_highlights(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        files_location: ov.files_location(),
        files_surface: ov.files_mode,
        files_query_focused: ov.files_mode && ov.files_focus == crate::overlay::FilesFocus::Query,
        settings_focus: None,
        browse_dir: ov.browse_dir.clone(),
        return_to: None,
        spell_target: None,
        table_dims: None,
        context_anchor: None,
        asset_preview: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        empty: None,
        show_hidden: false,
    });
    opts
}

#[test]
fn theme_actions_are_two_physical_rects_only_on_the_theme_overlay() {
    if !adapter_available() {
        eprintln!("skipping theme-actions sidecar law: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let scratch = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-capture-theme-actions-{}", std::process::id())),
    );
    let buffer = Buffer::from_str("theme action sidecar\n");

    let names = crate::theme::THEMES
        .iter()
        .map(|theme| theme.name.to_string())
        .collect();
    let theme = OverlayState::new_theme(names, crate::theme::active_index());
    let theme_png = scratch.join("theme.png");
    capture_with(&theme_png, &buffer, &overlay_opts(&theme)).expect("Theme capture renders");
    let theme_sidecar = sidecar(
        &std::fs::read_to_string(theme_png.with_extension("json")).expect("Theme sidecar exists"),
    );
    assert_eq!(theme_sidecar["schema"], serde_json::json!(schema_plain()));
    let actions = theme_sidecar["overlay"]["theme_actions"]
        .as_object()
        .expect("Theme publishes its action rects");
    let mut keys: Vec<_> = actions.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["cancel", "switch"]);
    for name in ["switch", "cancel"] {
        let rect = actions[name]
            .as_array()
            .unwrap_or_else(|| panic!("{name} is a physical [x,y,w,h] rect"));
        assert_eq!(rect.len(), 4, "{name} rect has four coordinates");
        let [x, y, w, h] = std::array::from_fn(|i| {
            rect[i]
                .as_f64()
                .unwrap_or_else(|| panic!("{name}[{i}] is finite"))
        });
        assert!(
            x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite(),
            "{name} rect is finite"
        );
        assert!(w > 0.0 && h > 0.0, "{name} rect has positive area");
        assert!(
            x >= 0.0 && y >= 0.0 && x + w <= 1200.0 && y + h <= 800.0,
            "{name} rect is expressed in the capture's physical pixel space: {rect:?}"
        );
    }
    assert_ne!(actions["switch"], actions["cancel"]);

    let command = OverlayState::new(OverlayKind::Command, vec!["Save".into()], vec![], vec![]);
    let command_png = scratch.join("command.png");
    capture_with(&command_png, &buffer, &overlay_opts(&command))
        .expect("non-Theme overlay capture renders");
    let command_sidecar = sidecar(
        &std::fs::read_to_string(command_png.with_extension("json"))
            .expect("non-Theme sidecar exists"),
    );
    assert!(command_sidecar["overlay"]["theme_actions"].is_null());

    let plain_png = scratch.join("plain.png");
    capture_with(&plain_png, &buffer, &CaptureOpts::default())
        .expect("overlay-free capture renders");
    let plain_sidecar = sidecar(
        &std::fs::read_to_string(plain_png.with_extension("json"))
            .expect("overlay-free sidecar exists"),
    );
    assert!(plain_sidecar["overlay"]["theme_actions"].is_null());
}
