//! Document hit testing, click selection, navigation, and selection dragging.

use crate::app::input::DragGranularity;
use crate::app::*;

impl App {
    pub(in crate::app) fn pointer_over_writing_column(&self) -> bool {
        self.frame.gpu().is_some_and(|gpu| {
            gpu.pipeline
                .over_writing_column(self.input.pointer.cursor_px.0)
        })
    }

    pub(in crate::app) fn hit_test_line_col(&self) -> (usize, usize) {
        let (px, py) = self.input.pointer.cursor_px;
        let gpu = self
            .frame
            .gpu()
            .expect("pointer hit testing requires the live GPU text pipeline");
        gpu.pipeline.hit_test_scroll(px, py, self.document.scroll())
    }

    /// Resolve shaped coordinates through the buffer's fold remap and grapheme snap.
    pub(in crate::app) fn hit_test_char(&self) -> usize {
        let (line, col) = self.hit_test_line_col();
        self.document.buffer().hit_char(line, col)
    }

    fn fold_affordance_at_pointer(&self) -> Option<usize> {
        if !self.document.buffer().has_folds() {
            return None;
        }
        let (line, col) = self.hit_test_line_col();
        self.document.buffer().fold_tail_hit(line, col)
    }

    /// Return the full-document heading under a revealed chevron. Cursor feedback
    /// and presses share this hit test; unavailable geometry returns `None`.
    pub(in crate::app) fn fold_chevron_at_pointer(&self) -> Option<usize> {
        let (px, py) = self.input.pointer.cursor_px;
        let filtered = self.frame.gpu()?.pipeline.fold_chevron_hit(px, py)?;
        Some(self.document.buffer().visible_line_to_full(filtered))
    }

    /// Share click cadence between document presses and page-edge resizing.
    pub(in crate::app) fn bump_click_count(&mut self) -> u32 {
        self.input.pointer.bump_click_count(self.frame.now())
    }

    /// Handle a primary press in the writing column. Margin presses are ignored
    /// because hit testing clamps them to text endpoints; an existing drag may
    /// still extend into a margin. Shift-click preserves the selection anchor.
    pub(in crate::app) fn on_press(&mut self, shift: bool, over_writing_column: bool) {
        if !over_writing_column {
            return;
        }
        // Fold controls consume plain presses before text selection.
        if !shift && let Some(h) = self.fold_chevron_at_pointer() {
            self.document.seal_undo_group();
            self.document.toggle_fold_at_line(h);
            self.document.clear_mark();
            return;
        }
        // The collapsed tail unfolds; heading text still places the caret.
        if !shift && let Some(h) = self.fold_affordance_at_pointer() {
            self.document.seal_undo_group();
            self.document.unfold_at(h);
            self.document.clear_mark();
            return;
        }
        let idx = self.hit_test_char();
        self.press_at_char(idx, shift);
    }

    /// Selection-state half of a document press after the live pipeline has
    /// resolved its shaped pixel position to a document character.
    pub(in crate::app) fn press_at_char(&mut self, idx: usize, shift: bool) {
        let click_count = self.bump_click_count();
        // A click is a non-edit gesture: seal the open undo group so text typed
        // after relocating the cursor is its own undo step.
        self.document.seal_undo_group();
        self.input.pointer.begin_text_drag();
        match click_count {
            1 if shift => {
                // Seed an absent anchor before moving the caret.
                self.input.pointer.drag_granularity = DragGranularity::Char;
                if self.document.buffer().anchor_char().is_none() {
                    self.document
                        .set_anchor(self.document.buffer().cursor_char());
                }
                self.document.set_cursor(idx);
                self.document.set_shift_selecting(true);
            }
            1 => {
                self.input.pointer.drag_granularity = DragGranularity::Char;
                self.document.set_cursor(idx);
                self.document.clear_mark();
                self.document.set_anchor(idx);
                self.document.set_shift_selecting(false);
            }
            2 => {
                self.input.pointer.drag_granularity = DragGranularity::Word;
                let (s, e) = self.document.buffer().word_bounds(idx);
                self.document.select_range(s, e);
            }
            _ => {
                self.input.pointer.drag_granularity = DragGranularity::Line;
                let (s, e) = self.document.buffer().line_bounds(idx);
                self.document.select_range(s, e);
            }
        }
        // Placements must reveal any folds intersecting the selection.
        self.document.reveal_placement();
    }

    /// Map a fold-filtered outline row to the full-document line navigation expects.
    pub(in crate::app) fn outline_row_target_line(&self, filtered_line: usize) -> usize {
        self.document.buffer().visible_line_to_full(filtered_line)
    }

    /// Jump to the heading under a visible outline row. The upstream dispatcher
    /// lets an overlay consume the press first.
    pub(in crate::app) fn outline_click(&mut self) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let line = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.outline_hit_line(px, py, g.config.height));
        if let Some(line) = line {
            self.jump_to_line(self.outline_row_target_line(line));
            true
        } else {
            false
        }
    }

    /// Follow the pointer target through the shared keyboard follow/effect path.
    /// Return whether the press was consumed, without moving the caret.
    pub(in crate::app) fn follow_link_at_pointer(&mut self) -> bool {
        let byte = self.document.buffer().char_to_byte(self.hit_test_char());
        let effect = crate::actions::follow::follow_effect(self.document.buffer(), byte);
        if effect == crate::actions::Effect::None {
            return false;
        }
        self.apply_live_effect(effect);
        true
    }

    /// Is the byte under the pointer followable? Asked off the same seam the
    /// click uses, so the hand cannot promise a follow the press would not do.
    pub(in crate::app) fn followable_at_pointer(&self) -> bool {
        let byte = self.document.buffer().char_to_byte(self.hit_test_char());
        crate::actions::follow::follow_effect(self.document.buffer(), byte)
            != crate::actions::Effect::None
    }

    /// Extend text selection, scrolling first when the pointer overshoots the
    /// writing band. The idle scheduler also drives [`Self::step_drag_scroll`].
    pub(in crate::app) fn on_drag(&mut self) {
        if !self.input.pointer.dragging {
            return;
        }
        let Some(_) = self.frame.gpu() else {
            return;
        };
        if self.step_drag_scroll() {
            return;
        }
        let idx = self.hit_test_char();
        self.drag_to_char(idx);
    }

    /// Shared scheduling guard: a text drag must be armed and its overshoot must
    /// earn a nonzero scroll rate. A stationary press or dead-zone hover never arms
    /// idle scrolling; unavailable geometry also declines.
    pub(in crate::app) fn drag_scroll_primed(&self) -> bool {
        if !self.input.pointer.dragging || !self.input.pointer.drag_armed {
            return false;
        }
        let (_, py) = self.input.pointer.cursor_px;
        let Some(gpu) = self.frame.gpu() else {
            return false;
        };
        gpu.pipeline
            .drag_scroll_active(py, gpu.config.height as f32)
    }

    /// Advance by elapsed time and extend selection under the scrolled content.
    /// Both pointer movement and the idle scheduler call this owner. The first tick
    /// primes the clock without scrolling; `false` lets the caller use its normal hit test.
    pub(in crate::app) fn step_drag_scroll(&mut self) -> bool {
        if !self.drag_scroll_primed() {
            self.input.pointer.clear_drag_scroll_tick();
            return false;
        }
        let (px, py) = self.input.pointer.cursor_px;
        let scroll = self.document.scroll();
        let now = self.frame.now();
        let dt = self.input.pointer.drag_scroll_tick_dt(now).as_secs_f32();
        let Some(gpu) = self.frame.gpu() else {
            return false;
        };
        let height = gpu.config.height as f32;
        let Some((new_scroll, line, col)) =
            gpu.pipeline.drag_scroll_step(scroll, px, py, height, dt)
        else {
            return false;
        };
        self.document.set_scroll(new_scroll);
        let idx = self.document.buffer().hit_char(line, col);
        self.drag_to_char(idx);
        true
    }

    /// Selection-state half of a text drag after the live pipeline has resolved
    /// the pointer to a document character.
    pub(in crate::app) fn drag_to_char(&mut self, idx: usize) {
        match self.input.pointer.drag_granularity {
            DragGranularity::Char => self.document.set_cursor(idx),
            DragGranularity::Word => {
                let anchor = self.document.buffer().anchor_char().unwrap_or(idx);
                let (ws, we) = self.document.buffer().word_bounds(idx);
                if idx >= anchor {
                    self.document.set_cursor(we);
                } else {
                    self.document.set_cursor(ws);
                }
            }
            DragGranularity::Line => {
                let anchor = self.document.buffer().anchor_char().unwrap_or(idx);
                let (ls, le) = self.document.buffer().line_bounds(idx);
                if idx >= anchor {
                    self.document.set_cursor(le);
                } else {
                    self.document.set_cursor(ls);
                }
            }
        }
        // Do not leave a selection spanning hidden content.
        self.document.reveal_placement();
    }
}
