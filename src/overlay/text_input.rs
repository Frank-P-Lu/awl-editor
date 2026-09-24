//! Focused textbox ownership for platform text input, independent of key actions.

use super::{OverlayKind, OverlayState};
use crate::textbox::{TextBox, TextField};

impl OverlayState {
    pub(crate) fn text_field(&self) -> Option<TextField> {
        if self.capture.is_some() {
            return None;
        }
        match self.kind {
            OverlayKind::Rename => self.rename_edit.as_ref().map(|_| TextField::Rename),
            OverlayKind::InsertLink => self.link_edit.as_ref().map(|_| TextField::InsertLink),
            OverlayKind::KeepName => self.keep_edit.as_ref().map(|_| TextField::KeepVersion),
            OverlayKind::Settings if self.value_edit.is_some() => Some(TextField::SettingsValue),
            OverlayKind::History | OverlayKind::Conflict => {
                (!self.detail_focus).then_some(TextField::PickerQuery)
            }
            OverlayKind::Spell
            | OverlayKind::Context
            | OverlayKind::TableDims
            | OverlayKind::Credits => None,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::SearchFolder
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Settings => Some(TextField::PickerQuery),
        }
    }

    pub(crate) fn text_box(&self, field: TextField) -> Option<&TextBox> {
        match field {
            TextField::PickerQuery => Some(&self.query),
            TextField::Rename => self.rename_edit.as_ref().map(|e| &e.input),
            TextField::InsertLink => self.link_edit.as_ref().map(|e| &e.input),
            TextField::KeepVersion => self.keep_edit.as_ref().map(|e| &e.input),
            TextField::SettingsValue => self.value_edit.as_ref().map(|e| &e.input),
            TextField::FindQuery | TextField::ReplaceText => None,
        }
    }

    /// Keep validation, mirroring and filtering at the same owners as typing.
    /// A composition is text, so characters cannot become navigation actions.
    pub(crate) fn commit_text(&mut self, field: TextField, text: &str) {
        match field {
            TextField::PickerQuery => {
                self.query.insert_text(text);
                self.selected = 0;
                self.scroll = 0;
                self.refilter();
            }
            TextField::Rename => {
                for c in text.chars() {
                    self.rename_edit_push(c);
                }
            }
            TextField::InsertLink => {
                for c in text.chars() {
                    self.link_edit_push(c);
                }
            }
            TextField::KeepVersion => {
                for c in text.chars() {
                    self.keep_edit_push(c);
                }
            }
            TextField::SettingsValue => {
                for c in text.chars() {
                    self.value_edit_push(c);
                }
            }
            TextField::FindQuery | TextField::ReplaceText => unreachable!("search owns its fields"),
        }
    }
}
