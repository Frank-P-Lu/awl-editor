//! Post-action synchronization between a summoned overlay and live pointer state.

use crate::app::*;

impl App {
    pub(super) fn sync_overlay_after_core(
        &mut self,
        overlay_was_open: bool,
        pointer: crate::app::input::RestingPointer,
    ) {
        // Re-anchor after every action so an incidental CursorMoved cannot let a
        // motionless pointer steal a keyboard-driven overlay selection.
        if let Some(overlay) = self.workspace_state.overlay_mut() {
            let (px, py) = pointer.px();
            overlay.arm_hover_baseline(px, py);
        }
        self.refresh_deep_file_status();
        let overlay_open = self.workspace_state.overlay_open();
        if overlay_was_open && !overlay_open {
            // The open->closed edge, not the button release, owns the query
            // drag's lifecycle from here: the press that armed it may still be
            // held when a keyboard action closes the overlay out from under it.
            self.input.clear_query_drag();
        }
        if overlay_open != overlay_was_open {
            self.resync_pointer_derived_state();
        }
    }

    /// Reconcile the selected filename/path result through one App owner.
    /// Keyboard, wheel, pointer lens, and semantic query doors all call here;
    /// the overlay caches the verdict, so view-only syncs never reread bytes.
    pub(in crate::app) fn refresh_deep_file_status(&mut self) {
        let root = self.project_location.root.clone();
        if let Some(overlay) = self.workspace_state.overlay_mut() {
            overlay.refresh_selected_deep_file_status(&root);
        }
    }
}
