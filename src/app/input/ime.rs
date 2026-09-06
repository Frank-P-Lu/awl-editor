//! Platform IME composition lifecycle and its live redraw door.

use crate::app::*;

impl App {
    /// Store transient composition without touching the buffer; commit inserts
    /// the finalized text and retires the preedit.
    pub(in crate::app) fn handle_ime(&mut self, ime: Ime) {
        match ime {
            Ime::Enabled => self.input.keyboard.ime_enabled = true,
            Ime::Disabled => {
                self.input.keyboard.ime_enabled = false;
                self.input.keyboard.preedit.clear();
            }
            Ime::Preedit(text, _cursor) => self.input.keyboard.preedit = text,
            Ime::Commit(text) => {
                self.input.keyboard.preedit.clear();
                // THE CENSUS DOOR (`app/input/text_door.rs`). This door never
                // resolves through the keymap, so the action intercept that
                // shuts every chord path cannot see it: a committed composition
                // arriving while a READ-ONLY prose surface is up would edit the
                // buffer hidden behind the transcript. Per CHARACTER, so a
                // commit coalesces into the open undo group exactly as typing
                // does.
                for c in text.chars() {
                    if !self.write_document_text(TextDoor::Ime, TextEdit::Char(c)) {
                        return;
                    }
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
