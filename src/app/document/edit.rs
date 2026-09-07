//! The ONE text edit `DocumentSession` exposes, and the four raw mutators it
//! dispatches to.
//!
//! Split out of `document.rs` so the four privates and their door sit together:
//! they are module-private on purpose, and this is the module they are private
//! to. Nothing in `crate::app` can edit the rope except through
//! [`DocumentSession::apply_text_edit`], whose only production caller is
//! `App::write_document_text` — which requires a named `TextDoor`. That is the
//! insertion-door census' forcing function (`app/input/text_door.rs`).

use super::*;

impl DocumentSession {
    /// **THE ONE TEXT EDIT VISIBLE OUTSIDE THIS MODULE.** Its four raw mutators
    /// below are module-private, so nothing in `crate::app` can edit the rope
    /// except through here — and the only production caller is
    /// [`App::write_document_text`], which requires a named
    /// [`crate::app::TextDoor`]. That is the census's forcing function: a new
    /// insertion door cannot reach the buffer without enrolling.
    ///
    /// Pinned to that single call site by `app::tests::insertion_census`, the
    /// same way `action_buffer_mut` is pinned to the action core.
    pub(in crate::app) fn apply_text_edit(&mut self, edit: crate::app::TextEdit<'_>) {
        match edit {
            crate::app::TextEdit::Char(ch) => self.edit_insert_char(ch),
            crate::app::TextEdit::Insert(text) => self.edit_insert_text(text),
            crate::app::TextEdit::ReplaceRange { start, end, text } => {
                self.edit_replace_char_range(start, end, text)
            }
            crate::app::TextEdit::Whole(text) => self.edit_set_text(text),
        }
    }

    fn edit_insert_char(&mut self, ch: char) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .insert_char(ch);
    }
    fn edit_insert_text(&mut self, text: &str) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .insert_text(text);
    }
    fn edit_replace_char_range(&mut self, start: usize, end: usize, text: &str) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .replace_char_range(start, end, text);
    }
    fn edit_set_text(&mut self, text: &str) {
        self.active
            .as_mut()
            .expect("active document")
            .buffer
            .set_text(text);
    }
}
