//! Live-state fold into the renderer-independent semantic snapshot.

use super::*;
use crate::semantic::{
    DOCUMENT_ID, DOCUMENT_TEXT_ID, ROOT_ID, SemanticAction, SemanticNode, SemanticRole,
    SemanticSelection, SemanticSnapshot,
};

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

    fn apply_semantic_request(&mut self, request: crate::semantic::SemanticRequest) {
        use crate::semantic::SemanticRequest;
        match request {
            SemanticRequest::Focus { id } => self.focus_semantic_node(&id),
            SemanticRequest::Click { id } => self.click_semantic_node(&id),
            SemanticRequest::SetTextSelection { id, anchor, focus } if id == DOCUMENT_ID => {
                let text = self.document.buffer().text();
                let anchor = crate::semantic::grapheme_to_char(&text, anchor);
                let focus = crate::semantic::grapheme_to_char(&text, focus);
                self.document.clear_mark();
                self.document.set_anchor(anchor);
                self.document.set_cursor(focus);
                self.sync_view(true);
                self.request_frame();
            }
            SemanticRequest::ReplaceSelectedText { id, value } if id == DOCUMENT_ID => {
                self.document.insert_text(&value);
                self.sync_view(true);
                self.request_frame();
            }
            SemanticRequest::SetValue { id, value } if id == DOCUMENT_ID => {
                let len = self.document.buffer().text().chars().count();
                self.document.replace_char_range(0, len, &value);
                self.sync_view(true);
                self.request_frame();
            }
            SemanticRequest::SetValue { id, value } => {
                self.set_semantic_value(&id, &value);
            }
            SemanticRequest::Increment { id } => {
                self.focus_semantic_node(&id);
                self.apply_semantic_action(Action::ForwardChar);
            }
            SemanticRequest::Decrement { id } => {
                self.focus_semantic_node(&id);
                self.apply_semantic_action(Action::BackwardChar);
            }
            SemanticRequest::Expand { id } => {
                self.focus_semantic_node(&id);
                self.apply_semantic_action(Action::Newline);
            }
            SemanticRequest::Collapse { .. } => self.apply_semantic_action(Action::Cancel),
            SemanticRequest::SetTextSelection { .. }
            | SemanticRequest::ReplaceSelectedText { .. } => {}
        }
    }

    fn apply_semantic_action(&mut self, action: Action) {
        let exit = schedule::RecordingExit::new();
        self.apply(action, false, &exit, crate::stats::Door::Menu);
    }

    fn overlay_target_position(&self, id: &str) -> Option<usize> {
        let overlay = self.workspace_state.journey().card()?;
        let kind = overlay.kind.as_str();
        overlay
            .item_corpus_indices()
            .iter()
            .position(|corpus| id == format!("overlay.{kind}.row.{corpus}"))
    }

    fn focus_semantic_node(&mut self, id: &str) {
        if id == DOCUMENT_ID {
            return;
        }
        if id == "search.query" || id == "search.replacement" {
            if let Some(search) = self.workspace_state.search_mut() {
                if id == "search.replacement" {
                    search.focus_replacement();
                } else {
                    search.focus_query();
                }
                self.sync_view(true);
                self.request_frame();
            }
            return;
        }
        let Some(target) = self.overlay_target_position(id) else {
            return;
        };
        let current = self
            .workspace_state
            .journey()
            .card()
            .map(|overlay| overlay.selected)
            .unwrap_or(0);
        let (action, count) = if target >= current {
            (Action::NextLine, target - current)
        } else {
            (Action::PreviousLine, current - target)
        };
        for _ in 0..count {
            self.apply_semantic_action(action.clone());
        }
    }

    fn click_semantic_node(&mut self, id: &str) {
        if id == "search.case-sensitive" {
            let text = self.document.buffer().text();
            if let Some(search) = self.workspace_state.search_mut() {
                search.toggle_case(&text);
                self.sync_view(true);
                self.request_frame();
            }
            return;
        }
        if let Some(name) = id.strip_prefix("format-popover.") {
            let button = match name {
                "bold" => crate::popover::PopoverButton::Bold,
                "italic" => crate::popover::PopoverButton::Italic,
                "highlight" => crate::popover::PopoverButton::Highlight,
                "code" => crate::popover::PopoverButton::Code,
                "strike" => crate::popover::PopoverButton::Strike,
                "heading" => crate::popover::PopoverButton::Heading,
                "link" => crate::popover::PopoverButton::Link,
                _ => return,
            };
            self.apply_semantic_action(button.action());
            return;
        }
        if self.overlay_target_position(id).is_some() {
            self.focus_semantic_node(id);
            self.apply_semantic_action(Action::Newline);
        }
    }

    fn set_semantic_value(&mut self, id: &str, value: &str) {
        let document_text = self.document.buffer().text();
        if id == "search.query" {
            if let Some(search) = self.workspace_state.search_mut() {
                search.set_query_text(value, &document_text);
            }
        } else if id == "search.replacement" {
            if let Some(search) = self.workspace_state.search_mut() {
                search.set_replacement_text(value);
            }
        } else if id.ends_with(".query")
            && let Some(overlay) = self.workspace_state.overlay_mut()
        {
            overlay.set_query_text(value);
        } else {
            return;
        }
        self.sync_view(true);
        self.request_frame();
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

        let focus_id = match layer {
            workspace::Layer::Editor => DOCUMENT_ID.to_string(),
            workspace::Layer::Popover => self.fold_popover(&mut nodes),
            workspace::Layer::Search => self.fold_search(&mut nodes),
            workspace::Layer::Workspace | workspace::Layer::Overlay => {
                self.fold_overlay(&mut nodes)
            }
        };

        if let Some(message) = self.frame.notice().text() {
            let notice_id = "notice".to_string();
            nodes.push(SemanticNode::new(&notice_id, SemanticRole::Status, message));
            nodes[0].children.push(notice_id);
        }

        debug_assert_eq!(nodes.iter().filter(|node| node.focused).count(), 1);
        SemanticSnapshot {
            schema: crate::semantic::SCHEMA,
            root_id: ROOT_ID.to_string(),
            focus_id,
            nodes,
        }
    }

    fn fold_search(&self, nodes: &mut Vec<SemanticNode>) -> String {
        let Some(search) = self.workspace_state.search() else {
            return DOCUMENT_ID.to_string();
        };
        let dialog_id = "search";
        let query_id = "search.query";
        let replace_id = "search.replacement";
        let case_id = "search.case-sensitive";
        let mut dialog = SemanticNode::new(dialog_id, SemanticRole::Dialog, "Find and replace");

        let query_focused = !search.is_editing_replacement();
        let mut query = SemanticNode::new(query_id, SemanticRole::TextInput, "Find");
        query.value = Some(search.query().to_string());
        query.character_lengths = crate::semantic::grapheme_lengths(search.query());
        let caret = crate::semantic::char_to_grapheme(search.query(), search.query_caret());
        query.selection = Some(SemanticSelection {
            anchor: caret,
            focus: caret,
        });
        query.focusable = true;
        query.focused = query_focused;
        query.editable = true;
        query.actions = vec![SemanticAction::Focus, SemanticAction::SetValue];
        dialog.children.push(query_id.to_string());
        nodes.push(query);

        let mut case = SemanticNode::new(case_id, SemanticRole::CheckBox, "Match case");
        case.checked = Some(search.is_case_sensitive());
        case.focusable = true;
        case.actions = vec![SemanticAction::Focus, SemanticAction::Click];
        dialog.children.push(case_id.to_string());
        nodes.push(case);

        if search.is_replace_active() {
            let mut replacement =
                SemanticNode::new(replace_id, SemanticRole::TextInput, "Replace with");
            replacement.value = Some(search.replacement().to_string());
            replacement.character_lengths = crate::semantic::grapheme_lengths(search.replacement());
            let caret =
                crate::semantic::char_to_grapheme(search.replacement(), search.replacement_caret());
            replacement.selection = Some(SemanticSelection {
                anchor: caret,
                focus: caret,
            });
            replacement.focusable = true;
            replacement.focused = !query_focused;
            replacement.editable = true;
            replacement.actions = vec![SemanticAction::Focus, SemanticAction::SetValue];
            dialog.children.push(replace_id.to_string());
            nodes.push(replacement);
        }

        dialog.description = Some(match search.current_index() {
            Some(current) => format!("{} of {} matches", current + 1, search.hit_count()),
            None => "No matches".to_string(),
        });
        nodes.push(dialog);
        nodes[0].children.push(dialog_id.to_string());
        if query_focused {
            query_id.to_string()
        } else {
            replace_id.to_string()
        }
    }

    fn fold_overlay(&self, nodes: &mut Vec<SemanticNode>) -> String {
        let Some(overlay) = self.workspace_state.journey().card() else {
            return DOCUMENT_ID.to_string();
        };
        let kind = overlay.kind.as_str();
        let dialog_id = format!("overlay.{kind}");
        let query_id = format!("{dialog_id}.query");
        let list_id = format!("{dialog_id}.rows");
        let mut dialog = SemanticNode::new(&dialog_id, SemanticRole::Dialog, overlay.kind.title());
        dialog.description = Some(overlay.foot_hint());

        let mut query = SemanticNode::new(&query_id, SemanticRole::TextInput, overlay.kind.title());
        query.value = Some(overlay.query.text().to_string());
        query.character_lengths = crate::semantic::grapheme_lengths(overlay.query.text());
        let caret = crate::semantic::char_to_grapheme(overlay.query.text(), overlay.query.caret());
        query.selection = Some(SemanticSelection {
            anchor: caret,
            focus: caret,
        });
        query.focusable = true;
        query.editable = true;
        query.actions = vec![SemanticAction::Focus, SemanticAction::SetValue];

        let mut list = SemanticNode::new(&list_id, SemanticRole::ListBox, overlay.kind.title());
        let labels = overlay.item_strings();
        let values = overlay.item_bindings();
        for (visible, (&corpus, label)) in overlay
            .item_corpus_indices()
            .iter()
            .zip(labels.iter())
            .enumerate()
        {
            let row_id = format!("{dialog_id}.row.{corpus}");
            let (role, actions) = overlay_row_semantics(overlay.kind, corpus);
            let mut row = SemanticNode::new(&row_id, role, label);
            row.value = values.get(visible).filter(|v| !v.is_empty()).cloned();
            row.selected = Some(visible == overlay.selected);
            row.focusable = true;
            row.focused = visible == overlay.selected;
            row.actions = actions;
            list.children.push(row_id);
            nodes.push(row);
        }
        if let Some(empty) = overlay.empty_notice() {
            let empty_id = format!("{dialog_id}.empty");
            list.children.push(empty_id.clone());
            nodes.push(SemanticNode::new(empty_id, SemanticRole::Status, empty));
        }
        if !overlay.notice.is_empty() {
            let notice_id = format!("{dialog_id}.notice");
            dialog.children.push(notice_id.clone());
            nodes.push(SemanticNode::new(
                notice_id,
                SemanticRole::Status,
                &overlay.notice,
            ));
        }
        dialog.children.push(query_id.clone());
        dialog.children.push(list_id.clone());
        if labels.is_empty() {
            query.focused = true;
        }
        nodes.push(query);
        nodes.push(list);
        nodes.push(dialog);
        nodes[0].children.push(dialog_id.clone());

        overlay
            .selected_corpus_index()
            .map(|corpus| format!("{dialog_id}.row.{corpus}"))
            .unwrap_or(query_id)
    }

    fn fold_popover(&self, nodes: &mut Vec<SemanticNode>) -> String {
        let dialog_id = "format-popover";
        let mut dialog = SemanticNode::new(dialog_id, SemanticRole::Dialog, "Formatting");
        let buffer = self.document.buffer();
        let model = actions::popover::plan(
            &buffer.text(),
            buffer.anchor_char(),
            buffer.cursor_char(),
            buffer.is_markdown(),
        );
        for state in model.into_iter().flat_map(|model| model.buttons) {
            let id = format!("{dialog_id}.{:?}", state.button).to_ascii_lowercase();
            let mut button =
                SemanticNode::new(&id, SemanticRole::Button, popover_name(state.button));
            button.value = state.active.then(|| "on".to_string());
            button.focusable = true;
            button.actions = vec![SemanticAction::Focus, SemanticAction::Click];
            dialog.children.push(id);
            nodes.push(button);
        }
        let focus = dialog
            .children
            .first()
            .cloned()
            .unwrap_or_else(|| DOCUMENT_ID.to_string());
        if let Some(node) = nodes.iter_mut().find(|node| node.id == focus) {
            node.focused = true;
        }
        nodes.push(dialog);
        nodes[0].children.push(dialog_id.to_string());
        focus
    }

    #[cfg(test)]
    pub(crate) fn set_semantic_text_for_test(&mut self, text: &str) {
        self.document.set_text(text);
        self.document.set_cursor(text.chars().count());
    }
}

fn overlay_row_semantics(
    kind: crate::overlay::OverlayKind,
    corpus: usize,
) -> (SemanticRole, Vec<SemanticAction>) {
    use crate::settings::SettingKind;
    if kind == crate::overlay::OverlayKind::Settings {
        if let Some(setting) = crate::settings::visible_rows().get(corpus) {
            return match setting.kind {
                SettingKind::Toggle => (
                    SemanticRole::CheckBox,
                    vec![SemanticAction::Focus, SemanticAction::Click],
                ),
                SettingKind::Range => (
                    SemanticRole::Slider,
                    vec![
                        SemanticAction::Focus,
                        SemanticAction::Increment,
                        SemanticAction::Decrement,
                    ],
                ),
                SettingKind::Value | SettingKind::Path => (
                    SemanticRole::TextInput,
                    vec![SemanticAction::Focus, SemanticAction::Click],
                ),
                SettingKind::Picker | SettingKind::Submenu | SettingKind::Action => (
                    SemanticRole::Button,
                    vec![SemanticAction::Focus, SemanticAction::Click],
                ),
            };
        }
    }
    (
        SemanticRole::Option,
        vec![SemanticAction::Focus, SemanticAction::Click],
    )
}

fn popover_name(button: crate::popover::PopoverButton) -> &'static str {
    match button {
        crate::popover::PopoverButton::Bold => "Bold",
        crate::popover::PopoverButton::Italic => "Italic",
        crate::popover::PopoverButton::Highlight => "Highlight",
        crate::popover::PopoverButton::Code => "Inline code",
        crate::popover::PopoverButton::Strike => "Strikethrough",
        crate::popover::PopoverButton::Heading => "Heading",
        crate::popover::PopoverButton::Link => "Link",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_markdown_snapshot_has_one_focus_and_grapheme_selection() {
        let _guard = crate::testlock::serial();
        let mut app = App::new_hermetic(None, PathBuf::from("/"), Config::empty());
        app.set_semantic_text_for_test("e\u{301} 👨‍👩‍👧‍👦 🇯🇵");
        let snapshot = app.semantic_snapshot();
        assert_eq!(snapshot.focus_id, DOCUMENT_ID);
        assert_eq!(snapshot.nodes.iter().filter(|node| node.focused).count(), 1);
        let text = snapshot
            .nodes
            .iter()
            .find(|node| node.id == DOCUMENT_TEXT_ID)
            .unwrap();
        assert_eq!(text.character_lengths.len(), 5);
        let document = snapshot
            .nodes
            .iter()
            .find(|node| node.id == DOCUMENT_ID)
            .unwrap();
        assert_eq!(document.selection.unwrap().focus, 5);
    }

    /// A second frame-side caller of `semantic_snapshot()` would reintroduce
    /// the per-frame whole-document cost that `refresh_accessibility`'s gate
    /// exists to prevent — and it would do so silently, because the equality
    /// dedup downstream still suppresses the *update*, just not the *build*.
    /// So the call sites are enumerated: three are the snapshot's declared
    /// consumers, and the fourth is the gate itself.
    #[test]
    fn semantic_snapshot_has_no_ungated_frame_side_caller() {
        let mut found: Vec<String> = Vec::new();
        let mut stack = vec![PathBuf::from("src")];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("src is readable") {
                let path = entry.expect("dir entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().is_none_or(|ext| ext != "rs") {
                    continue;
                }
                let source = std::fs::read_to_string(&path).expect("source is utf-8");
                // Only production lines: a test may build a snapshot freely,
                // and prose naming the function is not a call.
                let production = source
                    .split_once("#[cfg(test)]")
                    .map_or(source.as_str(), |(before, _)| before);
                if production.lines().any(|line| {
                    line.contains("semantic_snapshot()") && !line.trim().starts_with("//")
                }) {
                    found.push(path.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        found.sort();
        let mut sanctioned = vec![
            // The live-App sidecar embeds the same snapshot.
            "src/app/capture_state.rs".to_string(),
            // The gate, plus the AccessKit activation and action handlers.
            "src/app/semantic.rs".to_string(),
            // `--semantic-json` prints it.
            "src/main/run/live_app.rs".to_string(),
        ];
        sanctioned.sort();
        assert_eq!(
            found, sanctioned,
            "a new caller of semantic_snapshot() must justify its per-frame cost",
        );
    }

    #[test]
    fn document_ids_survive_edits() {
        let _guard = crate::testlock::serial();
        let mut app = App::new_hermetic(None, PathBuf::from("/"), Config::empty());
        let before: Vec<String> = app
            .semantic_snapshot()
            .nodes
            .into_iter()
            .map(|n| n.id)
            .collect();
        app.document.insert_text("# hello");
        let after: Vec<String> = app
            .semantic_snapshot()
            .nodes
            .into_iter()
            .map(|n| n.id)
            .collect();
        assert_eq!(before, after);
    }
}
