//! The redraw-request door.
//!
//! A redraw is a frame-scheduling request, not a rendering operation. GPU and
//! window ownership deliberately stay where they are; this module only keeps
//! winit's raw request verb behind one narrow App transition.

use super::*;

impl App {
    pub(in crate::app) fn request_frame(&self) {
        if let Some(gpu) = self.gpu.as_ref() {
            request_window(&gpu.window);
        }
    }
}

/// Window-owning recovery paths cannot borrow an assembled `App` yet. They
/// still cross the same door instead of calling winit directly.
pub(in crate::app) fn request_window(window: &Window) {
    window.request_redraw();
}
