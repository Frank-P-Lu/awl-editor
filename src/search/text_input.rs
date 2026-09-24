use super::SearchState;
use crate::textbox::{TextBox, TextField};

impl SearchState {
    pub(crate) fn text_field(&self) -> TextField {
        if self.editing_replacement {
            TextField::ReplaceText
        } else {
            TextField::FindQuery
        }
    }

    pub(crate) fn text_box(&self) -> &TextBox {
        if self.editing_replacement {
            &self.replacement
        } else {
            &self.query
        }
    }

    pub(crate) fn commit_text(&mut self, text: &str, haystack: &str) {
        if self.editing_replacement {
            self.replacement.insert_text(text);
        } else {
            self.query.insert_text(text);
            self.recompute(haystack);
        }
    }
}
