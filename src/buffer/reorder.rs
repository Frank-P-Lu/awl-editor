//! MOVE LINE UP/DOWN (⌥↑/⌥↓) — swap the caret's logical line, or every line an
//! active selection touches (moved as one block), with its immediate
//! neighbor above/below. A LOGICAL-line operation: the buffer has no notion
//! of a visual (wrapped) row at all, so a long soft-wrapped line always
//! moves as one atomic unit.

use super::Buffer;

impl Buffer {
    /// ⌥↑: swap the caret line — or every line the selection touches — with
    /// the line above. A calm no-op at the first line.
    pub fn move_line_up(&mut self) {
        self.move_lines(true);
    }

    /// ⌥↓: the downward mirror of [`Self::move_line_up`]. A calm no-op at
    /// the last line.
    pub fn move_line_down(&mut self) {
        self.move_lines(false);
    }

    /// Shared engine. Determines the affected LOGICAL line range (the caret
    /// line, or every line an active selection touches — mirroring
    /// `reindent`'s own "a selection ending at column 0 does NOT pull in
    /// that trailing line" rule), then rotates that range plus its one
    /// neighbor by exactly one position and applies the whole span as ONE
    /// atomic replace ([`Self::apply_edit`]) — a replace never coalesces
    /// (`record_edit`'s `is_replace` gate), so a single undo restores the
    /// pre-move order regardless of what ran immediately before it, and two
    /// separate moves stay two undo steps. Returns `false` (no edit, no
    /// version bump) when there is no neighbor to swap with.
    fn move_lines(&mut self, up: bool) -> bool {
        self.clear_kill_flag();
        self.goal_col = None;

        let sel = self.selection_range();
        // (start_line, end_line): the block's logical line span. `end_at_next_col0`
        // records whether the selection's LATER endpoint sits at column 0 of the
        // line right after the block (a whole-line Shift+Down-style selection) —
        // that endpoint is excluded from the block itself but its "hangs off the
        // block's tail" SHAPE must survive the move.
        let (start_line, end_line, has_sel, end_at_next_col0) = match sel {
            Some((s, e)) => {
                let (l0, _) = self.char_to_line_col(s);
                let (mut l1, c1) = self.char_to_line_col(e);
                let carried = c1 == 0 && l1 > l0;
                if carried {
                    l1 -= 1;
                }
                (l0, l1, true, carried)
            }
            None => {
                let (l, _) = self.cursor_line_col();
                (l, l, false, false)
            }
        };

        let total_lines = self.line_count();
        if up {
            if start_line == 0 {
                return false; // first line: nothing above to swap with
            }
        } else if end_line + 1 >= total_lines {
            return false; // last line: nothing below to swap with
        }

        let neighbor = if up { start_line - 1 } else { end_line + 1 };
        let (range_first, range_last) = if up {
            (neighbor, end_line)
        } else {
            (start_line, neighbor)
        };

        // Rotate: the neighbor moves to the opposite end of the range; every
        // other line shifts one position toward the neighbor's old slot.
        let mut lines: Vec<String> = (range_first..=range_last)
            .map(|l| self.line_text(l))
            .collect();
        if up {
            let n = lines.remove(0);
            lines.push(n);
        } else {
            let n = lines.pop().expect("range_first..=range_last is non-empty");
            lines.insert(0, n);
        }
        let new_block = lines.join("\n");

        let block_start = self.line_start(range_first);
        let block_end = self.line_start(range_last) + self.line_len(range_last);

        let delta: isize = if up { -1 } else { 1 };
        let new_start_line = (start_line as isize + delta) as usize;
        let new_end_line = (end_line as isize + delta) as usize;

        // Endpoint columns, read BEFORE the edit (each line's own content is
        // unchanged by the move — only its position shifts — so a column
        // captured now stays correct against the same line's new position).
        let s_col = sel.map(|(s, _)| self.char_to_line_col(s).1);
        let e_col = if end_at_next_col0 {
            0
        } else {
            sel.map(|(_, e)| self.char_to_line_col(e).1).unwrap_or(0)
        };
        let cur_col = self.char_to_line_col(self.cursor).1;
        let anchor_is_end = self.anchor.is_some_and(|a| a > self.cursor);

        // `cursor_after` is a placeholder (the real landing spot depends on the
        // POST-edit line layout, computed and set explicitly just below) —
        // `block_start` keeps it a valid in-range position rather than a bare
        // magic number in the interim.
        let before = self.cursor;
        self.apply_edit(
            block_start,
            block_end - block_start,
            &new_block,
            before,
            block_start,
        );

        if has_sel {
            let new_s = self.line_col_to_char(new_start_line, s_col.unwrap());
            let new_e = if end_at_next_col0 {
                self.line_col_to_char(new_end_line + 1, 0)
            } else {
                self.line_col_to_char(new_end_line, e_col)
            };
            if anchor_is_end {
                self.anchor = Some(new_e);
                self.cursor = new_s;
            } else {
                self.anchor = Some(new_s);
                self.cursor = new_e;
            }
        } else {
            self.cursor = self.line_col_to_char(new_start_line, cur_col);
            self.anchor = None;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn moves_the_caret_line_up_past_its_neighbor_and_rides_the_caret() {
        let mut b = Buffer::from_str("one\ntwo\nthree\n");
        b.set_cursor(b.line_col_to_char(1, 2)); // "tw|o"
        b.move_line_up();
        assert_eq!(b.text(), "two\none\nthree\n");
        assert_eq!(b.cursor_line_col(), (0, 2), "caret rides the moved text");
    }

    #[test]
    fn moves_the_caret_line_down_past_its_neighbor_and_rides_the_caret() {
        let mut b = Buffer::from_str("one\ntwo\nthree\n");
        b.set_cursor(b.line_col_to_char(1, 2));
        b.move_line_down();
        assert_eq!(b.text(), "one\nthree\ntwo\n");
        assert_eq!(b.cursor_line_col(), (2, 2), "caret rides the moved text");
    }

    #[test]
    fn first_line_up_is_a_calm_no_op() {
        let mut b = Buffer::from_str("one\ntwo\n");
        b.set_cursor(1);
        let before = b.text();
        b.move_line_up();
        assert_eq!(b.text(), before, "no edit at the first line");
        assert!(!b.can_undo(), "no version bump means nothing to undo");
        assert_eq!(b.cursor_char(), 1, "caret untouched");
    }

    #[test]
    fn last_line_down_is_a_calm_no_op() {
        // A trailing '\n' counts as its own (empty) final line -- ropey's
        // `len_lines` convention, matching `eol_crlf.rs`'s own pins ("abc\r\ndef\r\n"
        // is 3 lines, not 2). The TRUE last line here is that trailing empty
        // one, reachable exactly like Cmd-Down/buffer_end would land on it.
        let mut b = Buffer::from_str("one\ntwo\n");
        assert_eq!(b.line_count(), 3, "the trailing newline is its own last line");
        b.set_cursor(b.line_col_to_char(2, 0)); // caret on the trailing blank line
        let before = b.text();
        b.move_line_down();
        assert_eq!(b.text(), before, "no edit at the true last line");
        assert!(!b.can_undo());
    }

    #[test]
    fn moving_the_last_content_line_down_past_a_trailing_blank_swaps_into_it() {
        // The caret sits on "two", the last line of VISIBLE content, but one
        // real (reachable, swappable) line short of the buffer's true end --
        // the trailing newline's own empty line. Moving down is a real,
        // meaningful edit: "two" becomes the true last line (no trailing
        // newline of its own) and a blank line is left where it was.
        let mut b = Buffer::from_str("one\ntwo\n");
        b.set_cursor(b.line_col_to_char(1, 1)); // caret on "two", NOT the last line
        b.move_line_down();
        assert_eq!(b.text(), "one\n\ntwo");
        assert_eq!(b.cursor_line_col(), (2, 1), "caret rides \"two\" to its new line");
    }

    #[test]
    fn last_line_down_is_a_calm_no_op_with_no_trailing_newline() {
        // The final line carries no trailing '\n' -- still the last line,
        // still nothing below it to swap with.
        let mut b = Buffer::from_str("one\ntwo");
        b.set_cursor(b.line_col_to_char(1, 1));
        let before = b.text();
        b.move_line_down();
        assert_eq!(b.text(), before);
        assert!(!b.can_undo());
    }

    #[test]
    fn moving_the_final_line_without_a_trailing_newline_up_carries_the_missing_newline() {
        // "two" is the last line and has no trailing '\n'. Moving it up
        // relocates "no trailing newline" to whichever line ends up last --
        // a structural fact about which line IS last, not about specific text.
        let mut b = Buffer::from_str("one\ntwo");
        b.set_cursor(b.line_col_to_char(1, 1));
        b.move_line_up();
        assert_eq!(b.text(), "two\none", "no trailing newline after the move either");
        assert_eq!(b.cursor_line_col(), (0, 1));
    }

    #[test]
    fn block_selection_moves_as_one_unit_columns_preserved() {
        let mut b = Buffer::from_str("alpha\nbeta\ngamma\ndelta\n");
        // Select "beta" + "gamma" (lines 1..=2), caret at gamma's end (forward
        // selection: anchor at line1 col0, cursor at line2 col5).
        b.set_cursor(b.line_col_to_char(1, 0));
        b.set_mark();
        b.set_cursor(b.line_col_to_char(2, 5));
        b.move_line_down();
        assert_eq!(b.text(), "alpha\ndelta\nbeta\ngamma\n");
        assert_eq!(
            b.selection_line_col(),
            Some(((2, 0), (3, 5))),
            "the whole block rides together, columns preserved"
        );
    }

    #[test]
    fn block_selection_with_whole_line_shift_down_shape_moves_and_keeps_its_shape() {
        // Emulates Shift+Down repeatedly: selection END sits at column 0 of
        // the line AFTER the block (l0=0 through end of line1, expressed as
        // anchor@(0,0) .. cursor@(2,0)).
        let mut b = Buffer::from_str("one\ntwo\nthree\nfour\n");
        b.set_cursor(0);
        b.set_mark();
        b.set_cursor(b.line_col_to_char(2, 0)); // selects "one\ntwo\n" wholly
        b.move_line_down();
        assert_eq!(b.text(), "three\none\ntwo\nfour\n");
        assert_eq!(
            b.selection_line_col(),
            Some(((1, 0), (3, 0))),
            "the whole-line selection shape survives the move"
        );
    }

    #[test]
    fn block_move_against_buffer_start_is_a_no_op() {
        let mut b = Buffer::from_str("one\ntwo\nthree\n");
        b.set_cursor(0);
        b.set_mark();
        b.set_cursor(b.line_col_to_char(1, 3));
        let before = b.text();
        b.move_line_up();
        assert_eq!(before, b.text(), "block already touches the first line");
        assert!(!b.can_undo());
    }

    #[test]
    fn block_move_against_buffer_end_is_a_no_op() {
        // No trailing newline, so "three" -- the selection's real endpoint,
        // stopping exactly at its own end rather than carrying into a
        // following line -- IS the buffer's true last line, no phantom
        // trailing blank in the way.
        let mut b = Buffer::from_str("one\ntwo\nthree");
        assert_eq!(b.line_count(), 3, "no trailing newline: three real lines");
        b.set_cursor(b.line_col_to_char(1, 0));
        b.set_mark();
        b.set_cursor(b.line_col_to_char(2, 5)); // end of "three", not into a 4th line
        let before = b.text();
        b.move_line_down();
        assert_eq!(before, b.text(), "block already touches the last line");
        assert!(!b.can_undo());
    }

    #[test]
    fn two_separate_moves_undo_as_two_steps() {
        let mut b = Buffer::from_str("one\ntwo\nthree\n");
        b.set_cursor(b.line_col_to_char(2, 0));
        b.move_line_up(); // one\nthree\ntwo\n
        b.move_line_up(); // three\none\ntwo\n
        assert_eq!(b.text(), "three\none\ntwo\n");
        b.undo();
        assert_eq!(b.text(), "one\nthree\ntwo\n", "one undo reverts one move");
        b.undo();
        assert_eq!(b.text(), "one\ntwo\nthree\n", "the other undo reverts the other");
    }

    #[test]
    fn a_block_move_undoes_as_one_step() {
        let mut b = Buffer::from_str("one\ntwo\nthree\nfour\n");
        b.set_cursor(b.line_col_to_char(1, 0));
        b.set_mark();
        b.set_cursor(b.line_col_to_char(2, 5));
        b.move_line_down();
        assert_eq!(b.text(), "one\nfour\ntwo\nthree\n");
        b.undo();
        assert_eq!(
            b.text(),
            "one\ntwo\nthree\nfour\n",
            "the whole block move is ONE undo step"
        );
    }

    #[test]
    fn a_long_wrapping_length_line_moves_as_one_logical_unit() {
        // The buffer has no notion of a visual row, so a line long enough to
        // soft-wrap several times over is still exactly one logical line here.
        let long = "w".repeat(400);
        let mut b = Buffer::from_str(&format!("{long}\nshort\n"));
        b.set_cursor(0);
        b.move_line_down();
        assert_eq!(b.text(), format!("short\n{long}\n"));
        let (row, _) = b.cursor_line_col();
        assert_eq!(row, 1, "the whole long line moved as one unit");
        assert_eq!(b.line_text(1).chars().count(), 400);
    }

    #[test]
    fn move_resets_the_sticky_goal_column_like_any_other_edit() {
        // `next_line`'s flat (no-oracle) vertical motion is what a headless
        // unit test without a layout oracle exercises; it sticks `goal_col`
        // (private field, reachable here as a descendant of `crate::buffer`)
        // across a run of vertical moves.
        let mut b = Buffer::from_str("one two\nthree\nfour five\n");
        b.set_cursor(b.line_col_to_char(0, 5));
        b.next_line(); // establishes a sticky goal column for the run
        assert!(b.goal_col.is_some(), "setup: a goal column is now sticky");
        b.move_line_down();
        assert!(
            b.goal_col.is_none(),
            "an edit resets the goal column exactly like every other buffer mutation"
        );
    }

    #[test]
    fn identical_neighbor_lines_still_move_the_caret() {
        // Adjacent lines with byte-identical content: the SWAP is still a
        // real edit (the caret must still travel), even though the text
        // before/after this specific pair looks unchanged.
        let mut b = Buffer::from_str("same\nsame\nother\n");
        b.set_cursor(b.line_col_to_char(0, 2));
        b.move_line_down();
        assert_eq!(b.cursor_line_col(), (1, 2), "caret followed its own line down");
    }
}
