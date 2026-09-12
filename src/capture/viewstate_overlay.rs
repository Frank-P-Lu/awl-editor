//! Projection of serialized capture overlay state into the renderer's `ViewState`.

use crate::buffer::Buffer;
use crate::overlay::OverlayKind;
use crate::render::ViewState;

use super::opts::{CaptureOpts, OverlayInfo};

pub(super) fn fold(view: &mut ViewState, buffer: &Buffer, opts: &CaptureOpts, search_active: bool) {
    let kind = opts
        .overlay
        .as_ref()
        .and_then(|overlay| OverlayKind::from_mode(overlay.mode));
    fold_identity(view, buffer, opts, kind, search_active);
    fold_roster(view, opts, kind);
    fold_workspace_focus(view, opts, kind);
    fold_anchors_and_preview(view, opts, kind);
}

fn fold_identity(
    view: &mut ViewState,
    buffer: &Buffer,
    opts: &CaptureOpts,
    kind: Option<OverlayKind>,
    search_active: bool,
) {
    view.overlay_active = opts.overlay.as_ref().map(|o| o.active).unwrap_or(false);
    view.overlay_align = opts.overlay.as_ref().map(|o| o.align);
    view.overlay_theme_chrome = opts.overlay.as_ref().and_then(|o| o.chrome_theme);
    fold_popover(view, buffer, opts, search_active);

    view.overlay_crisp = kind.is_some_and(OverlayKind::keeps_backdrop_crisp);
    view.overlay_retains_room = kind.is_some_and(OverlayKind::retains_readable_room);
    (view.overlay_files_surface, view.overlay_files_location) =
        files_projection(opts.overlay.as_ref());
    view.overlay_theme_picker = kind == Some(OverlayKind::Theme);
    view.overlay_query = opts
        .overlay
        .as_ref()
        .map(|o| o.query.clone())
        .unwrap_or_default();
    view.overlay_query_caret = opts
        .overlay
        .as_ref()
        .map(|o| o.query_caret)
        .unwrap_or_else(|| view.overlay_query.chars().count());
    view.overlay_query_selection = opts.overlay.as_ref().and_then(|o| o.query_selection);
    view.overlay_query_placeholder =
        kind.and_then(|overlay_kind| overlay_kind.field_placeholder().map(str::to_string));
    view.overlay_title = opts
        .overlay
        .as_ref()
        .filter(|_| kind.is_none_or(OverlayKind::draws_title_prefix))
        .map(|o| o.title.clone())
        .unwrap_or_default();
    view.overlay_row_path_splits = kind.map(OverlayKind::row_path_splits).unwrap_or(false);
}

fn fold_popover(view: &mut ViewState, buffer: &Buffer, opts: &CaptureOpts, search_active: bool) {
    if crate::popover::popover_on()
        && !search_active
        && !view.overlay_active
        && (opts.force_popover || std::env::var_os("AWL_POPOVER").is_some())
        && let Some(((l0, c0), (l1, c1))) = view.selection
    {
        let anchor = buffer.line_col_to_char(l0, c0);
        let cursor = buffer.line_col_to_char(l1, c1);
        view.popover = crate::actions::popover::plan(
            &buffer.text(),
            Some(anchor),
            cursor,
            buffer.is_markdown(),
        );
    }
}

fn files_projection(overlay: Option<&OverlayInfo>) -> (bool, String) {
    let active = overlay.is_some_and(|overlay| {
        OverlayKind::from_mode(overlay.mode) == Some(OverlayKind::Goto) && overlay.files_surface
    });
    let location = active
        .then(|| overlay.and_then(|overlay| overlay.files_location.clone()))
        .flatten()
        .unwrap_or_default();
    (active, location)
}

fn fold_roster(view: &mut ViewState, opts: &CaptureOpts, kind: Option<OverlayKind>) {
    view.overlay_items = opts
        .overlay
        .as_ref()
        .map(|o| o.items.clone())
        .unwrap_or_default();
    view.overlay_hug_roster = opts.overlay_hug_roster.clone();
    view.overlay_empty = opts.overlay.as_ref().and_then(|o| o.empty.clone());
    view.overlay_bindings = opts
        .overlay
        .as_ref()
        .map(|o| o.bindings.clone())
        .unwrap_or_default();
    view.overlay_ranges = opts
        .overlay
        .as_ref()
        .map(|o| o.ranges.clone())
        .unwrap_or_default();
    view.overlay_match_highlights = match_highlights_projection(opts.overlay.as_ref());
    view.overlay_git = opts
        .overlay
        .as_ref()
        .map(|o| o.git.clone())
        .unwrap_or_default();
    view.overlay_selected = opts.overlay.as_ref().map(|o| o.selected_index).unwrap_or(0);

    let window_rows = kind.map(OverlayKind::window_rows).unwrap_or(12);
    view.overlay_window_rows = window_rows;
    view.overlay_scroll = if kind == Some(OverlayKind::Theme) {
        0
    } else {
        view.overlay_selected.saturating_sub(window_rows - 1)
    };
    view.overlay_hint = opts
        .overlay
        .as_ref()
        .map(|o| o.hint.clone())
        .unwrap_or_default();
    fold_facets(view, opts, kind == Some(OverlayKind::Theme));
}

fn match_highlights_projection(overlay: Option<&OverlayInfo>) -> Vec<Option<(usize, usize)>> {
    overlay
        .map(|o| o.match_highlights.clone())
        .unwrap_or_default()
}

fn fold_facets(view: &mut ViewState, opts: &CaptureOpts, theme_panel: bool) {
    view.overlay_lens = opts
        .overlay
        .as_ref()
        .map(|o| o.lens_strip.clone())
        .unwrap_or_default();
    if theme_panel && view.overlay_lens.is_empty() && std::env::var("AWL_THEME_LENS_DEMO").is_ok() {
        view.overlay_lens = vec![
            ("All".to_string(), false),
            ("Warm".to_string(), true),
            ("Cool".to_string(), false),
            ("Light".to_string(), false),
            ("Dark".to_string(), false),
        ];
    }
    view.overlay_sections = opts
        .overlay
        .as_ref()
        .map(|o| o.sections.clone())
        .unwrap_or_default();
    view.overlay_location = crate::facets::strip_location(&view.overlay_lens).map(str::to_string);
}

fn fold_workspace_focus(view: &mut ViewState, opts: &CaptureOpts, kind: Option<OverlayKind>) {
    view.overlay_workspace = opts.overlay.as_ref().map(|o| o.workspace).unwrap_or(false);
    view.overlay_rows_primary = opts
        .overlay
        .as_ref()
        .filter(|o| o.workspace)
        .and(kind)
        .and_then(OverlayKind::workspace_shape)
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    view.overlay_comparison = opts.preview_text.is_some();
    view.overlay_query_field = kind.is_none_or(OverlayKind::offers_query);
    view.overlay_query_focused = opts.overlay.as_ref().is_none_or(|overlay| {
        if kind == Some(OverlayKind::Settings) {
            overlay.settings_focus == Some("search")
        } else {
            !view.overlay_files_surface || overlay.files_query_focused
        }
    });
    view.overlay_detail_focus = opts
        .overlay
        .as_ref()
        .map(|o| o.detail_focus)
        .unwrap_or(false);
    view.overlay_rows_focused = opts.overlay.as_ref().is_none_or(|overlay| {
        if kind == Some(OverlayKind::Settings) {
            overlay.settings_focus == Some("controls")
        } else if view.overlay_files_surface {
            !overlay.files_query_focused
        } else {
            true
        }
    });
}

fn fold_anchors_and_preview(view: &mut ViewState, opts: &CaptureOpts, kind: Option<OverlayKind>) {
    view.overlay_spell = opts.overlay.as_ref().and_then(|o| o.spell_target);
    view.overlay_table_dims = opts.overlay.as_ref().and_then(|o| o.table_dims);
    view.overlay_context_anchor = opts.overlay.as_ref().and_then(|o| o.context_anchor);
    view.overlay_asset_preview = opts.overlay.as_ref().and_then(|o| o.asset_preview.clone());
    view.caret_preview = opts
        .overlay
        .as_ref()
        .filter(|_| kind == Some(OverlayKind::Caret))
        .and_then(|o| {
            o.items
                .get(o.selected_index)
                .and_then(|name| crate::caret::CaretMode::from_label(name))
        });
}

#[cfg(test)]
mod files_projection_tests {
    use super::*;

    fn goto(lens: Option<&'static str>, files_surface: bool) -> OverlayInfo {
        OverlayInfo {
            active: true,
            mode: "goto",
            align: crate::theme::CardAnchor::TopCenter,
            chrome_theme: None,
            query: String::new(),
            query_caret: 0,
            query_selection: None,
            items: Vec::new(),
            empty: None,
            bindings: Vec::new(),
            ranges: Vec::new(),
            match_highlights: Vec::new(),
            git: Vec::new(),
            selected_index: 0,
            hint: String::new(),
            files_location: Some("project/notes".into()),
            files_surface,
            files_query_focused: true,
            settings_focus: None,
            browse_dir: None,
            spell_target: None,
            table_dims: None,
            context_anchor: None,
            asset_preview: None,
            capture: None,
            notice: String::new(),
            lens,
            lens_strip: Vec::new(),
            sections: Vec::new(),
            preview_id: None,
            preview_view: None,
            workspace: false,
            detail_focus: false,
            diff_scroll: 0,
            show_hidden: false,
            return_to: None,
            title: "go to".into(),
        }
    }

    #[test]
    fn dedicated_files_identity_not_a_shared_lens_name_controls_projection() {
        for lens in ["files", "recent"] {
            let overlay = goto(Some(lens), true);
            assert_eq!(
                files_projection(Some(&overlay)),
                (true, "project/notes".into()),
                "dedicated Files {lens} lens carries its header and focus surface"
            );
        }

        for lens in [
            None,
            Some("all"),
            Some("files"),
            Some("headings"),
            Some("folders"),
            Some("recent"),
        ] {
            let overlay = goto(lens, false);
            assert_eq!(
                files_projection(Some(&overlay)),
                (false, String::new()),
                "generic Go to {lens:?} must not acquire the dedicated Files header"
            );
        }
    }

    #[test]
    fn search_match_ranges_survive_capture_projection() {
        let mut overlay = goto(None, false);
        overlay.match_highlights = vec![Some((18, 28))];
        assert_eq!(
            match_highlights_projection(Some(&overlay)),
            vec![Some((18, 28))],
            "the final capture view must retain the live row match range"
        );
        assert!(match_highlights_projection(None).is_empty());
    }
}
