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
        // Summoned cards own any mouse press, matching the key intercept —
        // except the Writing-streaks card, whose own surface PAGES instead
        // of dismissing (see `press_with_card_open`); a press outside it
        // still dismisses, exactly like About/Lifetime everywhere else.
        // MIDDLE is in this match (and not in the stamp/peek one above): it can
        // carry the follow gesture, so a summoned card must own it exactly as
        // it owns a left or right press — otherwise a middle-click would follow
        // a link in the document hidden behind the card.
        if state == ElementState::Pressed
            && matches!(
                button,
                MouseButton::Left | MouseButton::Right | MouseButton::Middle
            )
        {
            if crate::streaks::streaks_open() {
                let (px, py) = self.input.pointer.cursor_px;
                let on_card = crate::card::point_in_card(
                    px,
                    py,
                    self.frame
                        .gpu()
                        .and_then(|g| g.pipeline.streaks_card_rect()),
                );
                self.press_with_card_open(on_card);
                return;
            }
            if crate::card::dismiss_summoned_card() {
                self.sync_view(true);
                self.request_frame();
                return;
            }
        }
        if button == MouseButton::Right {
            if state == ElementState::Pressed && self.document.has_active() {
                let over_writing_column = self.pointer_over_writing_column();
                self.on_right_press(exit, over_writing_column);
            }
            return;
        }
        // A NON-PRIMARY button may still be a follow gesture — the Linux emacs
        // flavor seeds middle-click (mouse-2) the same way it seeds the Meta
        // and `C-x` key layers, through the one roster in `keymap::platform`.
        // Inert everywhere else, so this is a no-op on Mac and under `native`.
        if button == MouseButton::Middle {
            if state == ElementState::Pressed {
                self.press_follow_gesture(crate::keymap::PointerButton::Middle);
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
            ElementState::Released if self.input.pointer.row_drag.is_some() => {
                self.end_row_drag();
            }
            ElementState::Released if self.input.pointer.image_resizing.is_some() => {
                self.end_image_resize();
            }
            ElementState::Released if self.input.pointer.page_resizing => {
                self.end_page_resize();
            }
            ElementState::Released if self.input.pointer.query_drag => {
                self.input.pointer.query_drag = false;
            }
            ElementState::Released => self.on_left_release(),
        }
        self.request_frame();
    }

    /// What a press does while the Writing-streaks card is up, given only
    /// whether it landed ON the card's own drawn surface: `true` PAGES —
    /// flips heatmap⇄cumulative, the exact same flip the ←/→ key intercept
    /// performs (`streaks::toggle_view`) — and leaves the card open; `false`
    /// DISMISSES it, through the one shared owner every other summoned-card
    /// press closes through. Split out from the real geometry lookup (`on_card`,
    /// read through `self.frame.gpu()` in [`Self::on_mouse_input`] — live-only
    /// like every other pixel-position hit-test in this file: `popover_hit`,
    /// `start_action_at`) so this decision is testable given just the hit-test's
    /// answer, with no renderer at all.
    pub(in crate::app) fn press_with_card_open(&mut self, on_card: bool) {
        if on_card {
            crate::streaks::toggle_view();
        } else {
            crate::card::dismiss_summoned_card();
        }
        self.sync_view(true);
        self.request_frame();
    }

    /// THE POINTER DOOR onto the follow affordance, shared by every button that
    /// can carry one. Asks `keymap::follows_link` — the one selection point over
    /// the per-convention/per-flavor gesture roster — whether THIS button with
    /// THESE modifiers follows, and only then reaches the follow seam. Returns
    /// whether the press was spent following, so the caller swallows it.
    ///
    /// The two guards are the ones the ⌘-click affordance has always carried:
    /// no summoned picker may be up (its scrim owns the press), and the pointer
    /// must be over the writing column (the margins are not the document).
    fn press_follow_gesture(&mut self, button: crate::keymap::PointerButton) -> bool {
        if !self.document.has_active() {
            return false;
        }
        let follows = crate::keymap::follows_link(
            crate::convention::Convention::current(),
            self.config.keymap_flavor(),
            button,
            self.input.keyboard.mods.state(),
        );
        follows
            && self.workspace_state.pickers_clear()
            && self.pointer_over_writing_column()
            && self.follow_link_at_pointer()
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
        if self.press_follow_gesture(crate::keymap::PointerButton::Primary) {
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
