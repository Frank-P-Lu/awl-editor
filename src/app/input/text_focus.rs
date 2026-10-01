//! One recipient for platform text, its composition, and its candidate geometry.

use crate::app::*;
use crate::textbox::{TextBox, TextField, TextInputId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) enum TextTarget {
    None,
    Document(crate::buffers::BufferKey),
    Field {
        surface: TextInputId,
        field: TextField,
    },
}

impl App {
    pub(in crate::app) fn focused_text_target(&self) -> TextTarget {
        use crate::app::workspace::Layer;
        if crate::card::modal_card_open() || crate::menubar::open_menu().is_some() {
            return TextTarget::None;
        }
        match self.workspace_state.layer() {
            Layer::Overlay | Layer::Workspace => {
                let Some(card) = self.workspace_state.overlay() else {
                    return TextTarget::None;
                };
                let Some(field) = card.text_field() else {
                    return TextTarget::None;
                };
                if field == TextField::PickerQuery {
                    if card.files_mode && card.files_focus != crate::overlay::FilesFocus::Query {
                        return TextTarget::None;
                    }
                    if self
                        .workspace_state
                        .journey()
                        .settings_focus()
                        .is_some_and(|focus| {
                            focus != crate::overlay::workspace::SettingsFocus::Search
                        })
                    {
                        return TextTarget::None;
                    }
                }
                TextTarget::Field {
                    surface: card.text_input_id,
                    field,
                }
            }
            Layer::Search => {
                let search = self
                    .workspace_state
                    .search()
                    .expect("search layer has its panel");
                TextTarget::Field {
                    surface: search.text_input_id,
                    field: search.text_field(),
                }
            }
            Layer::Editor | Layer::Popover => self
                .document
                .active_key()
                .map(TextTarget::Document)
                .unwrap_or(TextTarget::None),
        }
    }

    /// Files deliberately hands typing from its controls back to the query.
    pub(in crate::app) fn focus_platform_text(&mut self) {
        if self.input.keyboard.ime_target.is_some() {
            return;
        }
        if let Some(card) = self.workspace_state.overlay_mut() {
            if card.files_mode {
                card.files_focus = crate::overlay::FilesFocus::Query;
            }
            if card.kind == crate::overlay::OverlayKind::Settings && card.value_edit.is_none() {
                self.workspace_state
                    .focus_settings_if_open(crate::overlay::workspace::SettingsFocus::Search);
            }
        }
    }

    /// Keep the cancelled recipient as a tombstone until the platform retires
    /// it. A late commit after Escape must not become document text.
    pub(in crate::app) fn reconcile_text_focus(&mut self) {
        let target = self.focused_text_target();
        if self
            .input
            .keyboard
            .ime_target
            .as_ref()
            .is_some_and(|old| *old != target)
        {
            self.input.keyboard.preedit.clear();
            self.input.keyboard.ime_target = Some(TextTarget::None);
        }
    }

    pub(in crate::app) fn focused_text_box(&self) -> Option<&TextBox> {
        let TextTarget::Field { field, .. } = self.focused_text_target() else {
            return None;
        };
        match field {
            TextField::FindQuery | TextField::ReplaceText => {
                self.workspace_state.search().map(|s| s.text_box())
            }
            TextField::PickerQuery
            | TextField::Rename
            | TextField::InsertLink
            | TextField::KeepVersion
            | TextField::SettingsValue => self
                .workspace_state
                .overlay()
                .and_then(|o| o.text_box(field)),
        }
    }
}
