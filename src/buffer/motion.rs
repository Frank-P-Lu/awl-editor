//! CURSOR MOTION — the non-mutating caret movements: C-f / C-b char motion,
//! C-a / C-e line ends, C-n / C-p vertical motion (keeping a goal column across
//! short lines), M-< / M-> buffer ends, and M-f / M-b word motion. Each clears the
//! kill flag like mg. Carved out of `buffer.rs` verbatim — inherent methods on
//! [`Buffer`].

use super::Buffer;

impl Buffer {
    // --- Motion -----------------------------------------------------------

    /// One CHARACTER right — one extended grapheme cluster, via the shared
    /// boundary owner [`crate::grapheme`], so a base and its combining marks
    /// cross together and the caret never parks inside a glyph.
    pub fn forward_char(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor =
            crate::grapheme::next_cluster_boundary(self.cursor, self.rope.len_chars(), |i| {
                self.rope.char(i)
            });
    }

    /// One CHARACTER left — the mirror of [`Self::forward_char`], same owner.
    pub fn backward_char(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor = crate::grapheme::prev_cluster_boundary(self.cursor, |i| self.rope.char(i));
    }

    pub fn line_start_motion(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_start(line);
    }

    pub fn line_end_motion(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        let (line, _) = self.cursor_line_col();
        self.cursor = self.line_start(line) + self.line_len(line);
    }

    pub fn next_line(&mut self) {
        self.vertical(1);
    }

    pub fn previous_line(&mut self) {
        self.vertical(-1);
    }

    /// Move the cursor `delta` lines (negative = up), preserving the goal column.
    fn vertical(&mut self, delta: isize) {
        self.clear_kill_flag();
        let (line, col) = self.cursor_line_col();
        let goal = self.goal_col.unwrap_or(col);
        let target_line = line as isize + delta;
        if target_line < 0 {
            // At top: go to start of first line but keep goal column.
            self.cursor = 0;
            self.goal_col = Some(goal);
            return;
        }
        let last_line = self.line_count() - 1;
        let target_line = (target_line as usize).min(last_line);
        let target_len = self.line_len(target_line);
        let target_col = goal.min(target_len);
        self.cursor = self.line_start(target_line) + target_col;
        self.goal_col = Some(goal);
    }

    pub fn buffer_start(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor = 0;
    }

    pub fn buffer_end(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor = self.rope.len_chars();
    }

    pub fn forward_word(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        // ONE owner of the word-MOTION boundary (`super::word_forward_boundary`),
        // shared with `crate::textbox::TextBox::word_right` — see that fn's doc
        // for why this must stay distinct from the word-DELETE rule.
        self.cursor =
            super::word_forward_boundary(self.cursor, self.rope.len_chars(), |i| self.rope.char(i));
    }

    pub fn backward_word(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor = super::word_backward_boundary(self.cursor, |i| self.rope.char(i));
    }

    /// M-e: to the start of the following sentence — see
    /// [`crate::buffer::sentence`]'s module doc for the UAX #29 rule and why
    /// that lands past a terminator's own trailing whitespace.
    pub fn forward_sentence(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor = super::sentence_forward_boundary(self.cursor, self.rope.len_chars(), |i| {
            self.rope.char(i)
        });
    }

    /// M-a: to the start of the current sentence, or the previous one if the
    /// cursor already sits at a sentence start — the exact mirror of
    /// [`Self::forward_sentence`].
    pub fn backward_sentence(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.cursor =
            super::sentence_backward_boundary(self.cursor, self.rope.len_chars(), |i| {
                self.rope.char(i)
            });
    }
}
