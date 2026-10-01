//! Platform IME composition lifecycle and its live redraw door.

use super::TextTarget;
use crate::app::*;
use crate::textbox::TextField;

impl App {
    /// Store transient composition without touching the buffer; commit inserts
    /// the finalized text and retires the preedit.
    pub(in crate::app) fn handle_ime(&mut self, ime: Ime) {
        self.reconcile_text_focus();
        match ime {
            Ime::Enabled => {
                self.input.keyboard.ime_enabled = true;
                self.input.keyboard.ime_target = None;
                self.input.keyboard.ime_preedit_ended = false;
                self.input.keyboard.preedit.clear();
            }
            Ime::Disabled => {
                self.input.keyboard.ime_enabled = false;
                self.input.keyboard.preedit.clear();
                self.input.keyboard.ime_target = None;
                self.input.keyboard.ime_preedit_ended = false;
                self.input.keyboard.preedit_cursor = 0;
            }
            Ime::Preedit(text, cursor) => {
                if text.is_empty() {
                    // Platforms can retire marked text immediately BEFORE its
                    // commit. Keep its recipient across that empty preedit.
                    self.input.keyboard.preedit.clear();
                    self.input.keyboard.ime_preedit_ended = true;
                    return;
                }
                // An empty preedit can also cancel without committing. A new
                // marked run starts a fresh composition, even when the platform
                // keeps IME enabled across cancellation and a focus change.
                if self.input.keyboard.ime_preedit_ended {
                    self.input.keyboard.ime_target = None;
                    self.input.keyboard.ime_preedit_ended = false;
                }
                self.focus_platform_text();
                let target = self.focused_text_target();
                let owner = self.input.keyboard.ime_target.get_or_insert(target.clone());
                if *owner != target || *owner == TextTarget::None {
                    return;
                }
                let byte = cursor.map(|(start, _)| start).unwrap_or(text.len());
                self.input.keyboard.preedit_cursor = text
                    .char_indices()
                    .take_while(|(index, _)| *index < byte)
                    .count();
                self.input.keyboard.preedit = text;
            }
            Ime::Commit(text) => {
                self.input.keyboard.preedit.clear();
                self.input.keyboard.ime_preedit_ended = false;
                self.focus_platform_text();
                let target = self.focused_text_target();
                let owner = self
                    .input
                    .keyboard
                    .ime_target
                    .take()
                    .unwrap_or(target.clone());
                if owner != target {
                    return;
                }
                match owner {
                    TextTarget::None => {}
                    TextTarget::Document(_) => {
                        // The document keeps its normal typing/undo door.
                        for c in text.chars() {
                            if !self.write_document_text(TextDoor::Ime, TextEdit::Char(c)) {
                                return;
                            }
                        }
                    }
                    TextTarget::Field { field, .. } => self.commit_field_text(field, &text),
                }
            }
        }
    }

    fn commit_field_text(&mut self, field: TextField, text: &str) {
        let text = crate::textbox::single_line(text);
        if text.is_empty() {
            return;
        }
        match field {
            TextField::FindQuery | TextField::ReplaceText => {
                if let Some(search) = self.workspace_state.search_mut() {
                    self.document.commit_search_text(search, &text);
                }
            }
            TextField::PickerQuery
            | TextField::Rename
            | TextField::InsertLink
            | TextField::KeepVersion
            | TextField::SettingsValue => {
                let prev = crate::theme::active();
                let Some(card) = self.workspace_state.overlay_mut() else {
                    return;
                };
                card.commit_text(field, &text);
                let theme = card.kind == crate::overlay::OverlayKind::Theme;
                if field == TextField::PickerQuery {
                    crate::actions::preview_move(card);
                }
                if theme {
                    self.retint_theme_preview(prev);
                }
            }
        }
    }

    pub(in crate::app) fn on_ime(&mut self, ime: Ime) {
        self.handle_ime(ime);
        self.sync_view(true);
        self.request_frame();
    }
}
