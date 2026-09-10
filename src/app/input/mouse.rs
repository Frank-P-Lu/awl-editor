//! Pointer event dispatch. Grabbed gestures precede hover; wheel routing keeps
//! working-set and table consumption ahead of overlay, zoom, and document scroll.

use super::wheel::*;
use crate::app::*;

mod document;
mod feedback;
mod overlay;
mod scroll;
mod surfaces;

impl App {
    /// Dispatch grabbed gestures before hover and text selection, then refresh
    /// pointer-derived cursor and hover feedback once.
    pub(in crate::app) fn on_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.input.pointer.cursor_px = (position.x as f32, position.y as f32);
        let prev_pointer_hide = self.input.pointer.pointer_hide;
        self.input.pointer.pointer_hide = crate::pointer_hide::on_mouse_move(prev_pointer_hide);
        if let Some(visible) = crate::pointer_hide::os_visibility_change(
            prev_pointer_hide,
            self.input.pointer.pointer_hide,
        ) && let Some(gpu) = self.frame.gpu()
        {
            gpu.window.set_cursor_visible(visible);
        }
        if self.input.pointer.range_drag.is_some() {
            // A live SETTINGS RAIL SCRUB owns the pointer outright (it is
            // a grabbed control): the value tracks the pointer through the range
            // spec, and the hover below must NOT also re-select rows under the
            // travelling pointer mid-gesture.
            self.on_range_drag();
        } else if self.input.pointer.row_drag.is_some() {
            // A press-armed WORKING-SET ROW DRAG owns the pointer outright,
            // the same way a rail scrub does: no row hover/switch may fire
            // under the travelling pointer until release settles the
            // gesture (`RowDrag`'s own doc).
            self.on_row_drag();
        } else if self.input.pointer.query_drag {
            // A press landed on the query field itself: every move scrubs its
            // caret, never a row hover — checked ahead of `overlay_open` below
            // so a query drag can't be read as a row hover crossing rows the
            // pointer is no longer over.
            self.on_query_drag();
        } else if self.workspace_state.overlay_open() {
            self.overlay_hover();
        } else if self.input.pointer.page_resizing {
            self.on_page_resize_drag();
        } else if self.input.pointer.image_resizing.is_some() {
            self.on_image_resize_drag();
        } else if self.input.pointer.dragging {
            // Reflow under a stationary pointer must not arm a text drag.
            if self.input.pointer.arm_text_drag_if_moved() {
                self.on_drag();
                self.sync_view(true);
                self.request_frame();
            }
        }
        self.resync_pointer_derived_state();
    }

    /// Abort row dragging and clear hover feedback when the pointer leaves.
    pub(in crate::app) fn on_cursor_left(&mut self) {
        // An in-flight row drag cannot survive the pointer leaving the
        // window — there is no release to resolve it against, so it aborts
        // rather than replaying a stale click or reorder on the next press.
        self.abort_row_drag();
        if self.clear_pointer_hover_state() {
            self.request_frame();
        }
    }

    pub(in crate::app) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        self.stamp_input();
        // A summoned MODAL card owns any press, but SWALLOWS a wheel outright.
        if crate::card::modal_card_open() {
            return;
        }
        if !self.document.has_active() && !self.workspace_state.overlay_open() {
            return;
        }
        if self.try_working_set_panel_scroll(delta) {
            return;
        }
        let zoom_mod = scroll_zoom_intent(self.input.keyboard.mods.state());
        if self.try_horizontal_table_pan(delta, zoom_mod) {
            return;
        }
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * WHEEL_LINES_PER_NOTCH,
            MouseScrollDelta::PixelDelta(p) => {
                accumulate_picker_pixels(&mut self.input.pointer.scroll_px_accum, p.y as f32)
            }
        };
        if self.workspace_state.overlay_open() {
            self.try_overlay_wheel_route(lines);
        } else if zoom_mod {
            self.try_zoom_wheel(lines);
        } else {
            self.scroll_document_wheel(delta);
        }
        self.request_frame();
    }
}
