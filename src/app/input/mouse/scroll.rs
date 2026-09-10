//! Wheel routing for working sets, tables, overlays, zoom, and document scrolling.

use super::super::wheel::*;
use crate::app::*;

impl App {
    fn wheel_scroll_px(&mut self, pixels: f32) {
        if let Some(gpu) = self.frame.gpu() {
            let scroll =
                gpu.pipeline
                    .scroll_by_px(self.document.scroll(), pixels, gpu.config.height as f32);
            self.document.set_scroll(scroll);
        }
    }

    /// A wheel over the EXPANDED WORKING-SET PANEL scrolls it, not the
    /// document — hit-tested against the SAME rect the lava carve uses.
    pub(super) fn try_working_set_panel_scroll(&mut self, delta: MouseScrollDelta) -> bool {
        if !self.document.working_set().is_expanded() {
            return false;
        }
        let (px, py) = self.input.pointer.cursor_px;
        let over_panel = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.gutter_stack_bounds(g.config.height))
            .is_some_and(|[x, y, w, h]| px >= x && px < x + w && py >= y && py < y + h);
        if !over_panel {
            return false;
        }
        let lines = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * WHEEL_LINES_PER_NOTCH,
            MouseScrollDelta::PixelDelta(p) => {
                accumulate_picker_pixels(&mut self.input.pointer.scroll_px_accum, p.y as f32)
            }
        };
        if lines.abs() >= 1.0 {
            let delta = -lines.round() as isize; // wheel up = toward the top
            self.document.working_set_mut().scroll_expanded(delta);
            self.sync_view(false);
        }
        self.request_frame();
        true
    }

    /// A horizontal-dominant packet tries the live TABLE's own pan first; a decline falls through.
    pub(super) fn try_horizontal_table_pan(
        &mut self,
        delta: MouseScrollDelta,
        zoom_mod: bool,
    ) -> bool {
        if zoom_mod || self.workspace_state.overlay_open() {
            return false;
        }
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                (x * WHEEL_PIXELS_PER_LINE, y * WHEEL_PIXELS_PER_LINE)
            }
            MouseScrollDelta::PixelDelta(p) => pixel_wheel_axes(
                p.x as f32,
                p.y as f32,
                self.input.pointer.scroll_sensitivity,
            ),
        };
        if dx.abs() > dy.abs() * 1.2 && dx.abs() > 0.5 {
            let (px, py) = self.input.pointer.cursor_px;
            let scroll = self.document.scroll();
            if let Some(gpu) = self.frame.gpu_mut()
                && gpu.pipeline.try_table_pan(px, py, scroll, dx)
            {
                self.request_frame();
                return true;
            }
        }
        false
    }

    /// With an overlay open: its comparison region beside its rows, asked
    /// kind-neutrally via `comparison_request()` (never a named-kind check —
    /// one used to read `kind == History` and missed the next such surface),
    /// or else the overlay's own rows.
    pub(super) fn try_overlay_wheel_route(&mut self, lines: f32) {
        if lines.abs() < 1.0 {
            return;
        }
        let diff_wheel = self
            .workspace_state
            .overlay()
            .map(|o| o.comparison_request().is_some())
            .unwrap_or(false)
            && !self
                .frame
                .gpu()
                .and_then(|g| g.pipeline.overlay_card_rect())
                .map(|[x, y, w, h]| {
                    let (px, py) = self.input.pointer.cursor_px;
                    px >= x && px < x + w && py >= y && py < y + h
                })
                .unwrap_or(false);
        if diff_wheel {
            let delta = -lines.round() as isize; // wheel up = toward the top
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.diff_scroll = ov.diff_scroll.saturating_add_signed(delta);
            }
            self.sync_view(false);
        } else {
            self.overlay_wheel(lines);
        }
    }

    /// One AUTHORED step per notch, anchored on the pointer only when the zoom moved.
    pub(super) fn try_zoom_wheel(&mut self, lines: f32) {
        if lines.abs() < 1.0 {
            return;
        }
        let dir = lines.signum();
        let before = self.frame.zoom();
        self.set_zoom(crate::range::ZOOM.stepped(self.frame.zoom(), dir as i32));
        if self.frame.zoom() != before {
            self.arm_zoom_anchor_pointer();
        }
        self.feed_peek(crate::peek::PeekStimulus::Interrupt);
    }

    /// Nothing else claimed it: scroll the writing column.
    pub(super) fn scroll_document_wheel(&mut self, delta: MouseScrollDelta) {
        let px = match delta {
            MouseScrollDelta::PixelDelta(p) => {
                pixel_wheel_document_px(p.y as f32, self.input.pointer.scroll_sensitivity)
            }
            MouseScrollDelta::LineDelta(_, y) => {
                line_wheel_document_px(y, self.frame.zoom(), self.frame.dpi())
            }
        };
        self.wheel_scroll_px(px);
        self.sync_view(false);
    }
}
