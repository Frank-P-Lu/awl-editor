//! The ACTIVE summoned surfaces — the ones that own the keyboard.
//!
//! Each fold returns the id that holds focus, which is what keeps the
//! exactly-one-focus invariant a property of the ladder rather than of a
//! bookkeeping field: `Layer` picks exactly one fold, and that fold names
//! exactly one focused node.

use super::*;

impl App {
    pub(super) fn fold_search(&self, nodes: &mut Vec<SemanticNode>) -> String {
        let Some(search) = self.workspace_state.search() else {
            return DOCUMENT_ID.to_string();
        };
        let mut dialog = SemanticNode::new(SEARCH_ID, SemanticRole::Dialog, "Find and replace");

        let query_focused = !search.is_editing_replacement();
        let mut query = SemanticNode::new(SEARCH_QUERY_ID, SemanticRole::TextInput, "Find");
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
        dialog.children.push(SEARCH_QUERY_ID.to_string());
        nodes.push(query);

        let mut case = SemanticNode::new(SEARCH_CASE_ID, SemanticRole::CheckBox, "Match case");
        case.checked = Some(search.is_case_sensitive());
        case.focusable = true;
        case.actions = vec![SemanticAction::Click];
        dialog.children.push(SEARCH_CASE_ID.to_string());
        nodes.push(case);

        if search.is_replace_active() {
            let mut replacement =
                SemanticNode::new(SEARCH_REPLACE_ID, SemanticRole::TextInput, "Replace with");
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
            dialog.children.push(SEARCH_REPLACE_ID.to_string());
            nodes.push(replacement);
        }

        dialog.description = Some(match search.current_index() {
            Some(current) => format!("{} of {} matches", current + 1, search.hit_count()),
            None => "No matches".to_string(),
        });
        nodes.push(dialog);
        nodes[0].children.push(SEARCH_ID.to_string());
        if query_focused {
            SEARCH_QUERY_ID.to_string()
        } else {
            SEARCH_REPLACE_ID.to_string()
        }
    }

    pub(super) fn fold_overlay(&self, nodes: &mut Vec<SemanticNode>) -> String {
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
        // No Focus: a picker's query box ALWAYS has the keyboard — every
        // keystroke goes there while the card is up, and there is no
        // transition that could make it more focused than it already is.
        // Advertising one would hand an assistive technology an action that
        // does nothing.
        query.actions = vec![SemanticAction::SetValue];

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

    pub(super) fn fold_popover(&self, nodes: &mut Vec<SemanticNode>) -> String {
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
            button.actions = vec![SemanticAction::Click];
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
}

/// A Settings row is not an option in a list — it is the control it renders.
/// Every other picker's rows are options, and the match is exhaustive over the
/// setting kinds so a new kind cannot fall through to a wrong role.
fn overlay_row_semantics(
    kind: crate::overlay::OverlayKind,
    corpus: usize,
) -> (SemanticRole, Vec<SemanticAction>) {
    use crate::settings::SettingKind;
    if kind == crate::overlay::OverlayKind::Settings
        && let Some(setting) = crate::settings::visible_rows().get(corpus)
    {
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
