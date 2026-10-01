//! Field-state receipts for opt-in live input investigation.
//! Queries and document bodies are omitted; actions can contain typed characters.

use crate::app::App;

impl App {
    pub(in crate::app) fn trace_input_state(
        &self,
        phase: &str,
        action: &crate::keymap::Action,
        door: crate::stats::Door,
    ) {
        if !crate::probe::recording() {
            return;
        }
        let buffer = self.document.buffer_opt();
        let overlay = self.workspace_state.overlay().map(|overlay| {
            (
                overlay.kind,
                overlay.files_focus,
                overlay.query.text().len(),
                overlay.query.caret(),
                overlay.query.selection_range(),
            )
        });
        crate::probe::trace(format_args!(
            "input-state {phase} door={door:?} action={action:?} mods={:?} \
             ime={} preedit_bytes={} doc_version={:?} doc_cursor={:?} doc_selection={:?} \
             overlay(kind,files_focus,query_bytes,caret,selection)={overlay:?}",
            self.input.keyboard.mods.state(),
            self.input.keyboard.ime_enabled,
            self.input.keyboard.preedit.len(),
            buffer.map(|buffer| buffer.version()),
            buffer.map(|buffer| buffer.cursor_char()),
            buffer.and_then(|buffer| buffer.selection_range()),
        ));
    }
}
