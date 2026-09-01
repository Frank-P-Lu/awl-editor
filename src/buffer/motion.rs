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
        self.cursor = super::sentence_backward_boundary(self.cursor, self.rope.len_chars(), |i| {
            self.rope.char(i)
        });
    }

    /// M-k / Delete sentence forward: kill from the cursor to the start of
    /// the following sentence (into the kill ring, same accumulation
    /// precedent as [`Self::delete_word_forward`]) — the sentence-motion
    /// sibling of the word deletes above, sharing
    /// [`super::sentence_forward_boundary`] with [`Self::forward_sentence`]
    /// so a kill always lands exactly where the motion would. With an active
    /// selection, delete that instead.
    pub fn delete_sentence_forward(&mut self) {
        self.goal_col = None;
        if self.delete_selection() {
            self.last_was_kill = false;
            return;
        }
        let len = self.rope.len_chars();
        let j = super::sentence_forward_boundary(self.cursor, len, |k| self.rope.char(k));
        if j > self.cursor {
            let killed = self.rope.slice(self.cursor..j).to_string();
            if self.last_was_kill {
                self.kill.push_str(&killed);
            } else {
                self.kill = killed;
            }
            let before = self.cursor;
            self.seal_undo_group();
            self.apply_edit(self.cursor, j - self.cursor, "", before, before);
            self.seal_undo_group();
            self.last_was_kill = true;
        } else {
            self.last_was_kill = false;
        }
    }

    /// Delete sentence backward (no default chord — reachable via the
    /// palette and `[keys]`, the word-delete precedent): kill from the start
    /// of the current sentence up to the cursor, the exact mirror of
    /// [`Self::delete_sentence_forward`], sharing
    /// [`super::sentence_backward_boundary`] with [`Self::backward_sentence`].
    /// With an active selection, delete that instead.
    pub fn delete_sentence_backward(&mut self) {
        self.goal_col = None;
        if self.delete_selection() {
            self.last_was_kill = false;
            return;
        }
        let len = self.rope.len_chars();
        let i = super::sentence_backward_boundary(self.cursor, len, |k| self.rope.char(k));
        if i < self.cursor {
            let killed = self.rope.slice(i..self.cursor).to_string();
            if self.last_was_kill {
                let mut acc = killed;
                acc.push_str(&self.kill);
                self.kill = acc;
            } else {
                self.kill = killed;
            }
            let before = self.cursor;
            self.seal_undo_group();
            self.apply_edit(i, self.cursor - i, "", before, i);
            self.seal_undo_group();
            self.last_was_kill = true;
        } else {
            self.last_was_kill = false;
        }
    }
}
