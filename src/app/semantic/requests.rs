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
    pub(super) fn apply_semantic_request(&mut self, request: SemanticRequest) -> bool {
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
                true
            }
            SemanticRequest::ReplaceSelectedText { id, value } if id == DOCUMENT_ID => {
                self.document.insert_text(&value);
                self.sync_view(true);
                self.request_frame();
                true
            }
            SemanticRequest::SetValue { id, value } if id == DOCUMENT_ID => {
                let len = self.document.buffer().text().chars().count();
                self.document.replace_char_range(0, len, &value);
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

    fn focus_semantic_node(&mut self, id: &str) -> bool {
        if id == DOCUMENT_ID {
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
        let Some(target) = self.overlay_target_position(id) else {
            return false;
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
        true
    }

    fn click_semantic_node(&mut self, id: &str) -> bool {
        if id == super::SEARCH_CASE_ID {
            let text = self.document.buffer().text();
            let Some(search) = self.workspace_state.search_mut() else {
                return false;
            };
            search.toggle_case(&text);
            self.sync_view(true);
            self.request_frame();
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
        if let Some((menu, item)) = self.menu_item_indices(id) {
            crate::menubar::set_open(None);
            let action = crate::menu::roster().get(menu).and_then(|menu| {
                crate::menu::dropdown_action(menu, item, self.document.buffer().is_markdown())
            });
            match action {
                Some(action) => self.apply_semantic_action(action),
                // An inert row (separator, OS-predefined, a disabled markdown
                // item) is still a real row on screen; closing the dropdown is
                // exactly what clicking it does.
                None => self.sync_view(true),
            }
            self.request_frame();
            return true;
        }
        if let Some(index) = self.menu_title_index(id) {
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
        let Some(index) = self.menu_title_index(id) else {
            return false;
        };
        crate::menubar::set_open(expanded.then_some(index));
        self.sync_view(true);
        self.request_frame();
        true
    }

    fn set_semantic_value(&mut self, id: &str, value: &str) -> bool {
        let document_text = self.document.buffer().text();
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
            overlay.set_query_text(value);
        } else {
            return false;
        }
        self.sync_view(true);
        self.request_frame();
        true
    }
}
