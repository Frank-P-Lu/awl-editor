//! The focused field's render snapshot and its single projection into chrome.

use super::ViewState;
use crate::textbox::TextField;

#[derive(Clone, Debug)]
pub struct FieldInput {
    pub field: TextField,
    pub text: String,
    pub caret: usize,
    pub selection: Option<(usize, usize)>,
    pub preedit: Option<(usize, usize)>,
    pub row: Option<usize>,
}

impl ViewState {
    /// Both live frames and App captures consume the same focused-field snapshot.
    pub(crate) fn apply_field_input(&mut self, input: FieldInput) {
        self.preedit.clear();
        match input.field {
            TextField::PickerQuery
            | TextField::Rename
            | TextField::InsertLink
            | TextField::KeepVersion => {
                self.overlay_query = input.text.clone();
                self.overlay_query_caret = input.caret;
                self.overlay_query_selection = input.selection;
                self.overlay_query_field = true;
                self.overlay_query_focused = true;
            }
            TextField::SettingsValue => {
                self.overlay_query_focused = false;
                if let Some(row) = input.row
                    && let Some(binding) = self.overlay_bindings.get_mut(row)
                {
                    *binding = input.text.clone();
                }
            }
            TextField::FindQuery => {
                self.search_query = input.text.clone();
                self.search_query_caret = input.caret;
                self.search_field_selection = input.selection;
            }
            TextField::ReplaceText => {
                self.search_replacement = input.text.clone();
                self.search_replacement_caret = input.caret;
                self.search_field_selection = input.selection;
            }
        }
        self.field_input = Some(input);
    }
}
