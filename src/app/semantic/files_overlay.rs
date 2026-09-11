//! Accessibility projection for Files' focusable regions.

use super::*;
use crate::overlay::{FilesFocus, OverlayState, RowMeta};

pub(super) fn query_focused(overlay: &OverlayState) -> bool {
    if contextual_text_field_owns_focus(overlay) {
        return true;
    }
    overlay.files_mode && overlay.files_focus == FilesFocus::Query
}

pub(super) fn row_focused(overlay: &OverlayState, corpus: usize, visible: usize) -> bool {
    if visible != overlay.selected {
        return false;
    }
    if contextual_text_field_owns_focus(overlay) {
        return false;
    }
    if !overlay.files_mode {
        return true;
    }
    match overlay.files_focus {
        FilesFocus::Choices => !matches!(
            overlay.rows[corpus].meta,
            RowMeta::FolderChooser | RowMeta::NewDocument
        ),
        FilesFocus::ChangeFolder => matches!(overlay.rows[corpus].meta, RowMeta::FolderChooser),
        FilesFocus::NewDocument => matches!(overlay.rows[corpus].meta, RowMeta::NewDocument),
        _ => false,
    }
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
    for (id, name, target, active) in [
        ("files", "Files", FilesFocus::Files, overlay.facet_lens == 0),
        (
            "recent",
            "Recent",
            FilesFocus::Recent,
            overlay.facet_lens == 1,
        ),
        ("up", "Up", FilesFocus::Up, overlay.browse_dir.is_some()),
    ] {
        if id == "up" && !active {
            continue;
        }
        let node_id = format!("{dialog_id}.{id}");
        let mut node = SemanticNode::new(&node_id, SemanticRole::Button, name);
        node.focusable = true;
        node.focused = overlay.files_focus == target;
        node.selected = (id != "up").then_some(active);
        node.actions = vec![SemanticAction::Focus, SemanticAction::Click];
        dialog.children.push(node_id);
        nodes.push(node);
    }
}

pub(super) fn focus_id(overlay: &OverlayState, dialog_id: &str, query_id: String) -> String {
    if contextual_text_field_owns_focus(overlay) {
        return query_id;
    }
    if overlay.files_mode {
        match overlay.files_focus {
            FilesFocus::Query => query_id,
            FilesFocus::Files => format!("{dialog_id}.files"),
            FilesFocus::Recent => format!("{dialog_id}.recent"),
            FilesFocus::Up => format!("{dialog_id}.up"),
            _ => selected_row_id(overlay, dialog_id).unwrap_or(query_id),
        }
    } else {
        selected_row_id(overlay, dialog_id).unwrap_or(query_id)
    }
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
