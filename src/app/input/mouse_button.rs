//! Mouse-button dispatch, including the no-document start actions.

use crate::app::*;

impl App {
    pub(in crate::app) fn on_mouse_input(
        &mut self,
        exit: &dyn schedule::Exit,
        state: ElementState,
        button: MouseButton,
    ) {
        if state == ElementState::Pressed
            && matches!(button, MouseButton::Left | MouseButton::Right)
        {
            self.stamp_input();
            self.feed_peek(crate::peek::PeekStimulus::Interrupt);
        }
        // Summoned cards own any mouse press, matching the key intercept.
        if state == ElementState::Pressed
            && matches!(button, MouseButton::Left | MouseButton::Right)
            && crate::card::dismiss_summoned_card()
        {
            self.sync_view(true);
            self.request_frame();
            return;
        }
        if button == MouseButton::Right {
            if state == ElementState::Pressed && self.document.has_active() {
                let over_writing_column = self.pointer_over_writing_column();
                self.on_right_press(exit, over_writing_column);
            }
            return;
        }
        if button != MouseButton::Left {
            return;
        }
        match state {
            ElementState::Pressed => self.on_left_press(exit),
            ElementState::Released if self.input.pointer.range_drag.is_some() => {
                self.end_range_drag();
            }
            ElementState::Released if self.input.pointer.image_resizing.is_some() => {
                self.end_image_resize();
            }
            ElementState::Released if self.input.pointer.page_resizing => {
                self.end_page_resize();
            }
            ElementState::Released => self.on_left_release(),
        }
        self.request_frame();
    }

    fn on_left_press(&mut self, exit: &dyn schedule::Exit) {
        if self.menubar_press(exit) {
            self.resync_pointer_derived_state();
            self.request_frame();
            return;
        }
        if !self.document.has_active() && !self.workspace_state.overlay_open() {
            let (px, py) = self.input.pointer.cursor_px;
            if let Some(action) = self
                .frame
                .gpu()
                .and_then(|gpu| gpu.pipeline.start_action_at(px, py))
            {
                let _ = self.apply(action, false, exit, crate::stats::Door::Chord);
            }
            self.resync_pointer_derived_state();
            self.request_frame();
            return;
        }
        if self
            .input
            .keyboard
            .mods
            .state()
            .contains(ModifiersState::SUPER)
            && self.workspace_state.pickers_clear()
            && self.pointer_over_writing_column()
            && self.follow_link_at_pointer()
        {
            return;
        }
        if self.workspace_state.popover_holds_attention() {
            let (px, py) = self.input.pointer.cursor_px;
            let hit = self
                .frame
                .gpu()
                .and_then(|g| g.pipeline.popover_hit(px, py));
            if let Some(button) = hit {
                let _ = self.apply(button.action(), false, exit, crate::stats::Door::Chord);
                self.sync_view(true);
                self.request_frame();
                return;
            }
            if self
                .frame
                .gpu()
                .is_some_and(|g| g.pipeline.over_popover(px, py))
            {
                return;
            }
            self.workspace_state.dismiss_popover();
        }
        if self.workspace_state.overlay_open() {
            self.overlay_click(exit);
        } else if !(self.workspace_state.search_active() && self.panel_click()
            || self.begin_image_resize_if_hovering()
            || self.begin_page_resize_if_hovering(exit))
            && !self.outline_click()
            && !self.gutter_stack_click()
        {
            let shift = self
                .input
                .keyboard
                .mods
                .state()
                .contains(ModifiersState::SHIFT);
            let over_writing_column = self.pointer_over_writing_column();
            self.on_press(shift, over_writing_column);
            if over_writing_column {
                self.sync_view(true);
                self.resync_pointer_derived_state();
            }
        }
    }

    fn on_left_release(&mut self) {
        if !self.document.has_active() {
            self.resync_pointer_derived_state();
            self.request_frame();
            return;
        }
        self.input.finish_text_drag();
        self.resync_pointer_derived_state();
        if !self.document.buffer().has_selection() {
            self.document.clear_mark();
        }
        let eligible = crate::popover::popover_on()
            && self.document.buffer().has_selection()
            && self.document.buffer().is_markdown();
        self.workspace_state.summon_popover(eligible);
        self.sync_view(true);
    }
}
