use super::*;

impl App {
    fn start_action_without_document(action: &Action) -> bool {
        matches!(
            action,
            Action::NewDocument | Action::OpenGoto | Action::Quit
        )
    }

    /// With no document, the canvas has only its two start actions. A summoned
    /// Go-to card continues through the ordinary core using the transition-only
    /// inert buffer until accepting a real file installs the active document.
    pub(in crate::app) fn reject_without_document(&self, action: &Action) -> bool {
        !self.document.has_active()
            && !self.workspace_state.overlay_open()
            && !Self::start_action_without_document(action)
    }

    /// Menu rows are global and remain addressable while Go to is open. Unlike
    /// overlay keystrokes, they never inherit the card's transition allowance:
    /// a document command such as Export still has no subject.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(in crate::app) fn reject_menu_without_document(&self, action: &Action) -> bool {
        !self.document.has_active() && !Self::start_action_without_document(action)
    }
}
