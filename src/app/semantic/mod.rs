//! Live-state fold into the renderer-independent semantic snapshot.
//!
//! One owner, three consumers: the native AccessKit tree, `--semantic-json`,
//! and the live-App capture sidecar all read the value `semantic_snapshot`
//! returns. Surfaces are folded in two families — the ACTIVE ones that own the
//! keyboard (`surfaces.rs`, selected by the `Layer` ladder) and the PASSIVE
//! ones that are announced without taking focus (`passive.rs`). Requests come
//! back through `requests.rs`, which ends every arm in an existing owner.

use super::*;
use crate::semantic::{
    DOCUMENT_ID, DOCUMENT_TEXT_ID, ROOT_ID, SemanticAction, SemanticNode, SemanticRequest,
    SemanticRole, SemanticSelection, SemanticSnapshot,
};

mod passive;
mod requests;
mod surfaces;

/// Ids that both a fold and a request arm must spell identically.
const SEARCH_ID: &str = "search";
const SEARCH_QUERY_ID: &str = "search.query";
const SEARCH_REPLACE_ID: &str = "search.replacement";
const SEARCH_CASE_ID: &str = "search.case-sensitive";
const WHICHKEY_ID: &str = "whichkey";
const MENUBAR_ID: &str = "menubar";
const NOTICE_ID: &str = "notice";

impl App {
    pub(in crate::app) fn handle_accessibility_event(&mut self, event: accesskit_winit::Event) {
        match event.window_event {
            accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.frame.set_accessibility_active(true);
                let snapshot = self.semantic_snapshot();
                self.frame.update_accessibility(snapshot, true);
            }
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                let snapshot = self.semantic_snapshot();
                if let Some(request) = crate::semantic::native::decode_request(&snapshot, request) {
                    self.apply_semantic_request(request);
                }
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.frame.set_accessibility_active(false);
            }
        }
    }

    /// The ONE live door from a scheduled frame to the platform tree. The gate
    /// sits here rather than inside the runtime because BUILDING the snapshot
    /// is the expense being avoided, and only the caller can decline to build
    /// it: a snapshot is a whole-rope `String` plus a UAX #29 pass over it,
    /// which an ordinary no-screen-reader frame must never pay.
    pub(in crate::app) fn refresh_accessibility(&mut self) {
        if !self.frame.accessibility_wants_snapshot() {
            return;
        }
        let snapshot = self.semantic_snapshot();
        self.frame.update_accessibility(snapshot, false);
    }

    pub(crate) fn semantic_snapshot(&self) -> SemanticSnapshot {
        let buffer = self.document.buffer();
        let text = buffer.text();
        let layer = self.workspace_state.layer();
        let document_focused = matches!(layer, workspace::Layer::Editor);
        let document_name = buffer
            .path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled document".to_string());

        let anchor_char = buffer.anchor_char().unwrap_or(buffer.cursor_char());
        let selection = SemanticSelection {
            anchor: crate::semantic::char_to_grapheme(&text, anchor_char),
            focus: crate::semantic::char_to_grapheme(&text, buffer.cursor_char()),
        };
        let mut text_node = SemanticNode::new(DOCUMENT_TEXT_ID, SemanticRole::Text, "Markdown");
        text_node.value = Some(text.clone());
        text_node.character_lengths = crate::semantic::grapheme_lengths(&text);

        let mut document = SemanticNode::new(DOCUMENT_ID, SemanticRole::Document, document_name);
        document.children.push(DOCUMENT_TEXT_ID.to_string());
        document.selection = Some(selection);
        document.focusable = true;
        document.focused = document_focused;
        document.editable = true;
        document.multiline = true;
        document.actions = vec![
            SemanticAction::Focus,
            SemanticAction::SetTextSelection,
            SemanticAction::ReplaceSelectedText,
            SemanticAction::SetValue,
        ];
        let mut root = SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl");
        root.children.push(DOCUMENT_ID.to_string());
        let mut nodes = vec![root, document, text_node];

        // Exactly one ACTIVE surface owns the keyboard, and the ladder — not a
        // field — is what says which. No wildcard arm: a sixth rung must be
        // placed here before it compiles.
        let focus_id = match layer {
            workspace::Layer::Editor => DOCUMENT_ID.to_string(),
            workspace::Layer::Popover => self.fold_popover(&mut nodes),
            workspace::Layer::Search => self.fold_search(&mut nodes),
            workspace::Layer::Workspace | workspace::Layer::Overlay => {
                self.fold_overlay(&mut nodes)
            }
        };

        // PASSIVE surfaces ride on top of whatever holds focus and never take
        // it: any number may be up at once without disturbing the invariant
        // below.
        self.fold_passive(&mut nodes, &text);

        if let Some(message) = self.frame.notice().text() {
            nodes.push(SemanticNode::new(NOTICE_ID, SemanticRole::Status, message));
            nodes[0].children.push(NOTICE_ID.to_string());
        }

        debug_assert_eq!(nodes.iter().filter(|node| node.focused).count(), 1);
        SemanticSnapshot {
            schema: crate::semantic::SCHEMA.to_string(),
            root_id: ROOT_ID.to_string(),
            focus_id,
            nodes,
        }
    }

    #[cfg(test)]
    pub(crate) fn set_semantic_text_for_test(&mut self, text: &str) {
        self.document.set_text(text);
        self.document.set_cursor(text.chars().count());
    }

    /// Put this App on a named surface. The projection laws live in
    /// `semantic::native`, which cannot reach `WorkspaceState`'s private
    /// transitions, so the fixtures are named here beside the folds they
    /// exercise.
    #[cfg(test)]
    pub(crate) fn install_semantic_fixture_for_test(&mut self, surface: &str) {
        match surface {
            "editor" => {}
            "overlay" => {
                self.workspace_state
                    .install_overlay_for_test(crate::overlay::OverlayState::new(
                        crate::overlay::OverlayKind::Command,
                        vec!["alpha".to_string(), "beta".to_string()],
                        Vec::new(),
                        Vec::new(),
                    ))
            }
            "search" => {
                self.workspace_state
                    .install_search_for_test(crate::search::SearchState::start(
                        0,
                        crate::search::Direction::Forward,
                    ));
            }
            other => panic!("unknown semantic fixture {other}"),
        }
    }
}

#[cfg(test)]
mod passive_roster;
#[cfg(test)]
mod tests;
