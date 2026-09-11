//! Decoded semantic requests, applied through the App's existing owners.
//!
//! Nothing here mutates the rope, an overlay or a setting directly: every arm
//! ends in `App::apply` (the one keymap transition owner) or in the owning
//! state's own verb. An assistive technology therefore drives exactly the
//! transitions a keyboard drives, with the same undo, stats and redraw.
//!
//! Every arm returns whether the request was HANDLED. That boolean is what
//! `advertised_actions_all_drive_a_real_transition` sweeps: a node that
//! advertises an action nothing routes is worse than a node that advertises
//! nothing, because an assistive technology will offer it to the user.

use super::*;

impl App {
    pub(in crate::app) fn apply_semantic_request(&mut self, request: SemanticRequest) -> bool {
        match request {
            SemanticRequest::Focus { id } => self.focus_semantic_node(&id),
            SemanticRequest::Click { id } => self.click_semantic_node(&id),
            SemanticRequest::SetTextSelection { id, anchor, focus } if id == DOCUMENT_ID => {
                // While the document node is showing a substituted transcript
                // (History/Conflict/Credits), a selection asked for selects
                // THAT text, never the hidden buffer underneath it — checked
                // against the projection that is actually published right
                // now, the same snapshot the request was decoded against,
                // rather than re-asking the live `App` (which can disagree
                // for one frame across a crossing). The buffer's cursor/
                // anchor are never touched from this arm: that is exactly the
                // leak `sync_document` closed for every OTHER reader-facing
                // question, one door further in.
                if let Some(projection) = self.frame.accessibility_projection_mut()
                    && projection.showing_transcript()
                {
                    projection.set_transcript_selection(anchor, focus);
                    self.request_frame();
                    return true;
                }
                if !self.document.has_active() {
                    return false;
                }
                let text = self.document.buffer().text();
                let anchor = crate::semantic::grapheme_to_char(&text, anchor);
                let focus = crate::semantic::grapheme_to_char(&text, focus);
                self.document.clear_mark();
                self.document.set_anchor(anchor);
                self.document.set_cursor(focus);
                self.sync_view(true);
                self.request_frame();
                true
            }
            SemanticRequest::ReplaceSelectedText { id, value } if id == DOCUMENT_ID => {
                // THE CENSUS DOOR (`app/input/text_door.rs`): an assistive
                // technology drives exactly the transitions a keyboard drives,
                // and the keyboard cannot write into a read-only prose surface.
                if !self.write_document_text(
                    TextDoor::AssistiveReplaceSelection,
                    TextEdit::Insert(&value),
                ) {
                    return false;
                }
                self.sync_view(true);
                self.request_frame();
                true
            }
            SemanticRequest::SetValue { id, value } if id == DOCUMENT_ID => {
                if !self.document.has_active() {
                    return false;
                }
                let len = self.document.buffer().text().chars().count();
                if !self.write_document_text(
                    TextDoor::AssistiveSetValue,
                    TextEdit::ReplaceRange {
                        start: 0,
                        end: len,
                        text: &value,
                    },
                ) {
                    return false;
                }
                self.sync_view(true);
                self.request_frame();
                true
            }
            SemanticRequest::SetValue { id, value } => self.set_semantic_value(&id, &value),
            SemanticRequest::Increment { id } => self.step_semantic_node(&id, Action::ForwardChar),
            SemanticRequest::Decrement { id } => self.step_semantic_node(&id, Action::BackwardChar),
            SemanticRequest::Expand { id } => self.set_menu_expanded(&id, true),
            SemanticRequest::Collapse { id } => self.set_menu_expanded(&id, false),
            SemanticRequest::SetTextSelection { .. }
            | SemanticRequest::ReplaceSelectedText { .. } => false,
        }
    }

    pub(super) fn apply_semantic_action(&mut self, action: Action) {
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

    fn workspace_rail_target_position(&self, id: &str) -> Option<usize> {
        let overlay = self.workspace_state.journey().card()?;
        let shape = overlay.workspace_shape()?;
        if shape.rows_are_primary() {
            return None;
        }
        let prefix = format!("overlay.{}.rail.", overlay.kind.as_str());
        let index = id.strip_prefix(&prefix)?.parse::<usize>().ok()?;
        (index < overlay.lens_strip().len()).then_some(index)
    }

    fn focus_semantic_node(&mut self, id: &str) -> bool {
        if id == START_NEW_ID || id == START_GOTO_ID {
            return !self.document.has_active();
        }
        if id == DOCUMENT_ID {
            if self.workspace_state.overlay().is_some_and(|overlay| {
                overlay
                    .workspace_shape()
                    .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary)
                    && overlay.comparison_request().is_some()
            }) {
                self.workspace_state.focus_workspace_detail();
                self.sync_view(true);
                self.request_frame();
            }
            return true;
        }
        if id == super::SEARCH_QUERY_ID || id == super::SEARCH_REPLACE_ID {
            let Some(search) = self.workspace_state.search_mut() else {
                return false;
            };
            if id == super::SEARCH_REPLACE_ID {
                search.focus_replacement();
            } else {
                search.focus_query();
            }
            self.sync_view(true);
            self.request_frame();
            return true;
        }
        if id.ends_with(".query") {
            let focus_workspace_detail = {
                let Some(overlay) = self.workspace_state.overlay_mut() else {
                    return false;
                };
                let expected = format!("overlay.{}.query", overlay.kind.as_str());
                if id != expected {
                    return false;
                }
                if overlay.files_mode {
                    overlay.files_focus = crate::overlay::FilesFocus::Query;
                }
                overlay
                    .workspace_shape()
                    .is_some_and(|shape| !shape.rows_are_primary())
            };
            if focus_workspace_detail {
                self.workspace_state.focus_workspace_detail();
            }
            self.sync_view(true);
            self.request_frame();
            return true;
        }
        if let Some(target) = self.workspace_rail_target_position(id) {
            if let Some(overlay) = self.workspace_state.overlay_mut() {
                overlay.set_facet_lens(target);
            }
            self.workspace_state.focus_workspace_primary();
            self.refresh_deep_file_status();
            self.sync_view(true);
            self.request_frame();
            return true;
        }
        let Some(target) = self.overlay_target_position(id) else {
            return false;
        };
        if let Some(shape) = self
            .workspace_state
            .overlay()
            .and_then(|overlay| overlay.workspace_shape())
        {
            if shape.rows_are_primary() {
                self.workspace_state.focus_workspace_primary();
            } else {
                self.workspace_state.focus_workspace_detail();
            }
        }
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
        true
    }

    fn click_semantic_node(&mut self, id: &str) -> bool {
        if id == START_NEW_ID {
            self.apply_semantic_action(Action::NewDocument);
            return true;
        }
        if id == START_GOTO_ID {
            self.apply_semantic_action(Action::OpenGoto);
            return true;
        }
        if let Some(target) = self.workspace_rail_target_position(id) {
            if let Some(overlay) = self.workspace_state.overlay_mut() {
                overlay.set_facet_lens(target);
            }
            self.workspace_state.focus_workspace_detail();
            self.refresh_deep_file_status();
            self.sync_view(true);
            self.request_frame();
            return true;
        }
        let search_control = match id {
            super::SEARCH_CASE_ID => Some(crate::search::PanelControl::CaseToggle),
            super::SEARCH_PREVIOUS_ID => Some(crate::search::PanelControl::NavPrev),
            super::SEARCH_NEXT_ID => Some(crate::search::PanelControl::NavNext),
            super::SEARCH_REPLACE_BUTTON_ID => Some(crate::search::PanelControl::ReplaceButton),
            super::SEARCH_REPLACE_ALL_ID => Some(crate::search::PanelControl::ReplaceAllButton),
            _ => None,
        };
        if let Some(control) = search_control {
            if self.workspace_state.search().is_none() || !self.document.has_active() {
                return false;
            }
            self.apply_semantic_action(Action::SearchPanel(control));
            return true;
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
                _ => return false,
            };
            self.apply_semantic_action(button.action());
            return true;
        }
        if let Some((menu, item)) = passive::menu_item_indices(id) {
            crate::menubar::set_open(None);
            let action = crate::menu::roster().get(menu).and_then(|menu| {
                crate::menu::dropdown_action(menu, item, self.document.active_is_markdown())
            });
            match action {
                Some(action) if !self.reject_menu_without_document(&action) => {
                    self.apply_semantic_action(action);
                }
                Some(_) => self.sync_view(true),
                // An inert row (separator, OS-predefined, a disabled markdown
                // item) is still a real row on screen; closing the dropdown is
                // exactly what clicking it does.
                None => self.sync_view(true),
            }
            self.request_frame();
            return true;
        }
        if let Some(index) = passive::menu_title_index(id) {
            // The same toggle a press on the title performs (`menubar_press`).
            crate::menubar::toggle_open(index);
            self.workspace_state.dismiss_pickers();
            self.sync_view(true);
            self.request_frame();
            return true;
        }
        if self.overlay_target_position(id).is_some() {
            self.focus_semantic_node(id);
            self.apply_semantic_action(Action::Newline);
            return true;
        }
        false
    }

    /// A slider step. Only a row that really exists steps — otherwise the
    /// caret would quietly walk the document instead.
    fn step_semantic_node(&mut self, id: &str, action: Action) -> bool {
        if self.overlay_target_position(id).is_none() {
            return false;
        }
        self.focus_semantic_node(id);
        self.apply_semantic_action(action);
        true
    }

    fn set_menu_expanded(&mut self, id: &str, expanded: bool) -> bool {
        let Some(index) = passive::menu_title_index(id) else {
            return false;
        };
        crate::menubar::set_open(expanded.then_some(index));
        self.sync_view(true);
        self.request_frame();
        true
    }

    fn set_semantic_value(&mut self, id: &str, value: &str) -> bool {
        let document_text = self
            .document
            .buffer_opt()
            .map(Buffer::text)
            .unwrap_or_default();
        if id == super::SEARCH_QUERY_ID {
            let Some(search) = self.workspace_state.search_mut() else {
                return false;
            };
            search.set_query_text(value, &document_text);
        } else if id == super::SEARCH_REPLACE_ID {
            let Some(search) = self.workspace_state.search_mut() else {
                return false;
            };
            search.set_replacement_text(value);
        } else if id.ends_with(".query") {
            let Some(overlay) = self.workspace_state.overlay_mut() else {
                return false;
            };
            overlay.set_semantic_query_text(value);
        } else {
            return false;
        }
        self.refresh_deep_file_status();
        self.sync_view(true);
        self.request_frame();
        true
    }
}
