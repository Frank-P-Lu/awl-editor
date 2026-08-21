use super::*;

impl App {
    /// With no document, the canvas has only its two start actions. A summoned
    /// Go-to card continues through the ordinary core using the transition-only
    /// inert buffer until accepting a real file installs the active document.
    pub(super) fn reject_without_document(&self, action: &Action) -> bool {
        !self.document.has_active()
            && !self.workspace_state.overlay_open()
            && !matches!(
                action,
                Action::NewDocument | Action::OpenGoto | Action::Quit
            )
    }
}
