//! One preservation boundary before keyboard/menu Quit or window-close.

use crate::app::*;

impl App {
    /// Refuse exit while any open entry lacks a saved original or a current
    /// active-conflict recovery record. Never remove or activate an entry here.
    fn prepare_document_exit(&mut self) -> bool {
        if self.config.autosave_on() {
            self.flush_note();
            self.autosave_flush();
        }
        let held = self
            .persistence
            .unresolved()
            .map(|change| change.path.clone());
        if let Some(path) = &held
            && !self.write_recovery_record(path)
        {
            self.set_sticky_notice("Recovery could not be saved — Awl stays open");
            self.request_frame();
            return false;
        }
        let active = self.document.active_key();
        for key in self.document.open_entry_keys() {
            let Some(facts) = self.document.close_facts(&key) else {
                self.set_sticky_notice("An open document could not be checked — Awl stays open");
                self.request_frame();
                return false;
            };
            let recovered = active.as_ref() == Some(&key)
                && facts
                    .path
                    .as_ref()
                    .is_some_and(|path| held.as_ref() == Some(path));
            if facts.unsaved && !recovered {
                self.set_sticky_notice("Unsaved changes — save each open document before quitting");
                self.request_frame();
                return false;
            }
        }
        true
    }

    /// The only document-aware exit request, shared by action and window doors.
    pub(in crate::app) fn request_document_exit(&mut self, exit: &dyn schedule::Exit) -> bool {
        if !self.prepare_document_exit() {
            return false;
        }
        exit.exit();
        true
    }
}
