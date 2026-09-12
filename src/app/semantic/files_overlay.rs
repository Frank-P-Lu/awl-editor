//! Accessibility projection for Files' focusable regions.

use super::*;
use crate::overlay::{FilesFocus, OverlayState, workspace::SettingsFocus};

pub(super) fn query_focused(overlay: &OverlayState, settings_focus: Option<SettingsFocus>) -> bool {
    if let Some(focus) = settings_focus {
        return focus == SettingsFocus::Search;
    }
    if contextual_text_field_owns_focus(overlay) {
        return true;
    }
    overlay.files_mode && overlay.files_focus == FilesFocus::Query
}

pub(super) fn row_focused(
    overlay: &OverlayState,
    corpus: usize,
    visible: usize,
    settings_focus: Option<SettingsFocus>,
) -> bool {
    if visible != overlay.selected {
        return false;
    }
    if contextual_text_field_owns_focus(overlay) {
        return false;
    }
    if let Some(focus) = settings_focus {
        return focus == SettingsFocus::Controls;
    }
    if let Some(shape) = overlay.workspace_shape() {
        let rows_focused = overlay.detail_focus != shape.rows_are_primary();
        return rows_focused;
    }
    if !overlay.files_mode {
        return true;
    }
    overlay.files_focus == FilesFocus::Choices && corpus < overlay.rows.len()
}

/// Publish the category rail Settings actually draws. Its active category and
/// keyboard focus are separate facts: entering the rows keeps the category
/// selected without continuing to tell assistive technology it owns focus.
pub(super) fn append_workspace_rail(
    overlay: &OverlayState,
    settings_focus: Option<SettingsFocus>,
    dialog_id: &str,
    dialog: &mut SemanticNode,
    nodes: &mut Vec<SemanticNode>,
) {
    let Some(shape) = overlay.workspace_shape() else {
        return;
    };
    if shape.rows_are_primary() {
        return;
    }
    let rail_id = format!("{dialog_id}.rail");
    let mut rail = SemanticNode::new(&rail_id, SemanticRole::ListBox, "Categories");
    for (index, (label, active)) in overlay.lens_strip().into_iter().enumerate() {
        let id = workspace_rail_row_id(dialog_id, index);
        let mut item = SemanticNode::new(&id, SemanticRole::Option, label);
        item.focusable = true;
        item.selected = Some(active);
        item.focused = active
            && settings_focus
                .map(|focus| focus == SettingsFocus::Categories)
                .unwrap_or(!overlay.detail_focus);
        item.actions = vec![SemanticAction::Focus, SemanticAction::Click];
        rail.children.push(id);
        nodes.push(item);
    }
    dialog.children.push(rail_id);
    nodes.push(rail);
}

pub(super) fn append_controls(
    overlay: &OverlayState,
    dialog_id: &str,
    dialog: &mut SemanticNode,
    nodes: &mut Vec<SemanticNode>,
) {
    if !overlay.files_mode {
        return;
    }
    let root = if overlay.files_root_name.is_empty() {
        "folder"
    } else {
        &overlay.files_root_name
    };
    let destination = overlay
        .browse_dir
        .as_deref()
        .filter(|dir| !dir.is_empty())
        .map(|dir| format!("{root}/{dir}"))
        .unwrap_or_else(|| root.to_string());
    for (id, name, target, active) in [
        (
            "files",
            "Files".to_string(),
            FilesFocus::Files,
            overlay.facet_lens == 0,
        ),
        (
            "recent",
            "Recent".to_string(),
            FilesFocus::Recent,
            overlay.facet_lens == 1,
        ),
        (
            "up",
            "Up".to_string(),
            FilesFocus::Up,
            overlay.browse_dir.is_some(),
        ),
        (
            "change-folder",
            "Change folder".to_string(),
            FilesFocus::ChangeFolder,
            true,
        ),
        (
            "new-document",
            format!("New document in {destination}"),
            FilesFocus::NewDocument,
            true,
        ),
    ] {
        if id == "up" && !active {
            continue;
        }
        let node_id = format!("{dialog_id}.{id}");
        let mut node = SemanticNode::new(&node_id, SemanticRole::Button, name);
        node.focusable = true;
        node.focused = overlay.files_focus == target;
        node.selected = matches!(id, "files" | "recent").then_some(active);
        node.actions = vec![SemanticAction::Focus, SemanticAction::Click];
        dialog.children.push(node_id);
        nodes.push(node);
    }
}

pub(super) fn focus_id(
    overlay: &OverlayState,
    dialog_id: &str,
    query_id: String,
    settings_focus: Option<SettingsFocus>,
) -> String {
    if contextual_text_field_owns_focus(overlay) {
        return query_id;
    }
    if let Some(focus) = settings_focus {
        match focus {
            SettingsFocus::Categories => workspace_rail_row_id(dialog_id, overlay.facet_lens),
            SettingsFocus::Search => query_id,
            SettingsFocus::Controls => selected_row_id(overlay, dialog_id).unwrap_or(query_id),
        }
    } else if overlay.files_mode {
        match overlay.files_focus {
            FilesFocus::Query => query_id,
            FilesFocus::Files => format!("{dialog_id}.files"),
            FilesFocus::Recent => format!("{dialog_id}.recent"),
            FilesFocus::Up => format!("{dialog_id}.up"),
            FilesFocus::ChangeFolder => format!("{dialog_id}.change-folder"),
            FilesFocus::NewDocument => format!("{dialog_id}.new-document"),
            FilesFocus::Choices => selected_row_id(overlay, dialog_id).unwrap_or(query_id),
        }
    } else if let Some(shape) = overlay.workspace_shape() {
        if overlay.detail_focus && shape.rows_are_primary() {
            DOCUMENT_ID.to_string()
        } else if !overlay.detail_focus && !shape.rows_are_primary() {
            workspace_rail_row_id(dialog_id, overlay.facet_lens)
        } else {
            selected_row_id(overlay, dialog_id).unwrap_or(query_id)
        }
    } else {
        selected_row_id(overlay, dialog_id).unwrap_or(query_id)
    }
}

pub(super) fn control_focus(id: &str, overlay: &OverlayState) -> Option<FilesFocus> {
    if !overlay.files_mode {
        return None;
    }
    let prefix = format!("overlay.{}.", overlay.kind.as_str());
    Some(match id.strip_prefix(&prefix)? {
        "files" => FilesFocus::Files,
        "recent" => FilesFocus::Recent,
        "up" if overlay.browse_dir.is_some() => FilesFocus::Up,
        "change-folder" => FilesFocus::ChangeFolder,
        "new-document" => FilesFocus::NewDocument,
        _ => return None,
    })
}

pub(super) fn workspace_rail_row_id(dialog_id: &str, index: usize) -> String {
    format!("{dialog_id}.rail.{index}")
}

/// Contextual fields whose text entry remains the keyboard recipient while
/// their action row stays selected. This is an input fact, deliberately
/// independent of the renderer's backdrop-composition roster.
fn contextual_text_field_owns_focus(overlay: &OverlayState) -> bool {
    overlay.kind == crate::overlay::OverlayKind::Command || overlay.link_edit.is_some()
}

fn selected_row_id(overlay: &OverlayState, dialog_id: &str) -> Option<String> {
    overlay
        .selected_corpus_index()
        .map(|corpus| format!("{dialog_id}.row.{corpus}"))
}
