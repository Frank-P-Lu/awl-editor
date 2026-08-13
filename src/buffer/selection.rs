//! SELECTION + CURSOR PLACEMENT — the mark / region model, the raw cursor
//! setters, and the line/col conversions that sit beside them. `set_mark` /
//! `clear_mark` / `set_anchor` manage the selection anchor; `selection_range` /
//! `selection_line_col` read it; `set_cursor` / `set_cursor_visual` move the caret
//! WITHOUT disturbing the mark (so a Shift+motion or mouse drag extends a region);
//! `delete_selection` / `copy_region` / `kill_region` act on it. Carved out of
//! `buffer.rs` verbatim — these stay inherent methods on [`Buffer`].

use super::{Buffer, is_word_char};

impl Buffer {
    // --- Selection (mark / region) ----------------------------------------

    /// The pointer WORD around `idx` — what a DOUBLE-CLICK selects, and the unit
    /// a word-granularity drag extends by. Code keeps awl's identifier-shaped
    /// editor word ([`Self::editor_word_bounds`]). Prose on macOS first asks the
    /// local NaturalLanguage tokenizer; the portable fallback preserves the
    /// existing English class but narrows an unspaced CJK run to one extended
    /// grapheme. Word motion and word delete remain separate rules.
    pub fn word_bounds(&self, idx: usize) -> (usize, usize) {
        let len = self.rope.len_chars();
        if len == 0 {
            return (0, 0);
        }
        let idx = idx.min(len);

        if self.page_class() == crate::page::PageClass::Prose {
            #[cfg(target_os = "macos")]
            {
                // Tokenize only the touched logical line: linguistic words do
                // not cross a newline, and a pointer drag must stay O(line),
                // never clone/tokenize the whole manuscript on every move.
                let line = self.rope.char_to_line(idx);
                let line_start = self.rope.line_to_char(line);
                let line_text = self.rope.line(line).to_string();
                if let Some(bounds) = crate::word_selection::linguistic_word_bounds(
                    &line_text,
                    idx - line_start,
                    line_start,
                    len,
                    |i| self.rope.char(i),
                ) {
                    return bounds;
                }
            }

            if let Some(bounds) =
                crate::word_selection::portable_cjk_grapheme_bounds(idx, len, |i| self.rope.char(i))
            {
                return bounds;
            }
        }

        self.editor_word_bounds(idx)
    }

    /// awl's editor-style word (or run of non-word chars): alphanumeric plus
    /// underscore. Code pointer selection always uses this exact rule. Both
    /// ends snap OUTWARD to grapheme boundaries, since a class walk can stop
    /// before a combining mark and park the caret inside the visible cluster.
    pub(super) fn editor_word_bounds(&self, idx: usize) -> (usize, usize) {
        let len = self.rope.len_chars();
        if len == 0 {
            return (0, 0);
        }
        let idx = idx.min(len);
        let class_at = |i: usize| -> Option<bool> {
            if i < len {
                Some(is_word_char(self.rope.char(i)))
            } else {
                None
            }
        };
        let want = class_at(idx)
            .or_else(|| if idx > 0 { class_at(idx - 1) } else { None })
            .unwrap_or(true);
        let mut start = idx;
        while start > 0 && is_word_char(self.rope.char(start - 1)) == want {
            start -= 1;
        }
        let mut end = idx;
        while end < len && is_word_char(self.rope.char(end)) == want {
            end += 1;
        }
        (
            crate::grapheme::snap_backward(start, len, |i| self.rope.char(i)),
            crate::grapheme::snap_forward(end, len, |i| self.rope.char(i)),
        )
    }

    /// C-Space: set the mark at the current cursor (start a selection).
    pub fn set_mark(&mut self) {
        self.clear_kill_flag();
        self.anchor = Some(self.cursor);
    }

    /// C-g: clear the mark (cancel the selection). Cursor unchanged.
    pub fn clear_mark(&mut self) {
        self.anchor = None;
        // A selection change bypasses `clear_kill_flag`, so the
        // list-continuation provenance flag needs its own clear here.
        self.list_continuation_generated = false;
    }

    /// Cmd-A: SELECT ALL — set the mark at document start (char 0) and place the
    /// point at document end (`len_chars`), so the ENTIRE buffer is the active
    /// region. Reuses the mark/point machinery (like a C-Space at the top then a
    /// motion to the end). On an EMPTY buffer this leaves anchor == cursor == 0, so
    /// `has_selection()` stays false and it is a calm no-op (no panic).
    pub fn select_all(&mut self) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.anchor = Some(0);
        self.cursor = self.rope.len_chars();
    }

    /// Set the mark to an explicit char index (used by mouse-press to begin a
    /// drag selection). Clamped into range.
    pub fn set_anchor(&mut self, idx: usize) {
        self.clear_kill_flag();
        self.anchor = Some(idx.min(self.rope.len_chars()));
    }

    /// True when a mark is set and spans a non-empty region.
    pub fn has_selection(&self) -> bool {
        matches!(self.anchor, Some(a) if a != self.cursor)
    }

    /// The active mark (anchor), if any. `None` = no selection.
    #[allow(dead_code)]
    pub fn anchor_char(&self) -> Option<usize> {
        self.anchor
    }

    /// The selection as an ordered `(start, end)` char range (start <= end), or
    /// `None` when there is no non-empty selection.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        match self.anchor {
            Some(a) if a != self.cursor => Some((a.min(self.cursor), a.max(self.cursor))),
            _ => None,
        }
    }

    /// The active selection's TEXT, or `None` when there is no non-empty
    /// selection. Used by Cmd-F's "search for selection" prefill (the Xcode
    /// convention, W2 of the keybinding-idiom audit — see
    /// `actions/motion.rs::start_search`); never mutates.
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        Some(self.rope.slice(start..end).to_string())
    }

    /// The selection expressed in line/col endpoints, ordered so the first
    /// endpoint is earlier in the buffer. Returns `((l0,c0),(l1,c1))` or `None`.
    /// Used by the renderer to build highlight rectangles.
    pub fn selection_line_col(&self) -> Option<((usize, usize), (usize, usize))> {
        let (start, end) = self.selection_range()?;
        Some((self.char_to_line_col(start), self.char_to_line_col(end)))
    }

    /// Convert an absolute char index to (line, col).
    pub fn char_to_line_col(&self, idx: usize) -> (usize, usize) {
        let idx = idx.min(self.rope.len_chars());
        let line = self.rope.char_to_line(idx);
        let line_start = self.rope.line_to_char(line);
        (line, idx - line_start)
    }

    /// The text of `line` EXCLUDING the trailing newline. Used by the markdown
    /// smart-newline to read the current block's prefix (list marker / blockquote
    /// / indentation) so Enter can continue or end it. Pure read; no allocation
    /// beyond the one returned line.
    pub fn line_text(&self, line: usize) -> String {
        let start = self.line_start(line);
        let len = self.line_len(line);
        self.rope.slice(start..start + len).to_string()
    }

    /// Convert a (line, col) to an absolute char index, clamping col to the
    /// line's length and line to the buffer. The inverse of [`char_to_line_col`]
    /// for in-range inputs; used by mouse hit-testing.
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        let last_line = self.line_count() - 1;
        let line = line.min(last_line);
        let len = self.line_len(line);
        self.line_start(line) + col.min(len)
    }

    /// THE ONE CLICK-TO-ROPE RESOLUTION: a hit-tested `(line, col)` — `line` in
    /// the render's FOLD-FILTERED space, `col` a char column on it — to the char
    /// index a caret may actually occupy. Two steps the pointer path must never
    /// take separately: remap the line through [`Self::visible_line_to_full`] (a
    /// fold above the click shifts every line below it), then snap the column's
    /// char index to the NEAREST grapheme-cluster boundary
    /// ([`crate::grapheme::snap_nearest`], which is also where "nearest in chars"
    /// is justified as nearest on screen), so a pointer inside a rendered `é`
    /// resolves to one side of it instead of between its letter and its accent.
    ///
    /// Every click, drag endpoint, right-press and ⌘-click link probe passes
    /// through here, via `App::hit_test_char`. Kept on the buffer rather than the
    /// app so the rule is testable without a live pointer or a GPU.
    pub fn hit_char(&self, visible_line: usize, col: usize) -> usize {
        let idx = self.line_col_to_char(self.visible_line_to_full(visible_line), col);
        crate::grapheme::snap_nearest(idx, self.rope.len_chars(), |i| self.rope.char(i))
    }

    /// Move the cursor to an absolute char index (clamped), WITHOUT touching the
    /// mark, so a Shift+motion or mouse drag extends the selection. Resets the
    /// goal column and kill flag like the other motions.
    pub fn set_cursor(&mut self, idx: usize) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.goal_x = None;
        self.cursor = idx.min(self.rope.len_chars());
    }

    /// The remembered VISUAL goal-x for visual-line vertical motion (see the
    /// `goal_x` field). `apply_transition`'s layout oracle reads this at the start of a
    /// C-n/C-p: `Some(x)` means a run of vertical moves is in progress and the
    /// caret should stay under `x`; `None` means recompute from the caret's current
    /// visual x.
    pub fn goal_x(&self) -> Option<f32> {
        self.goal_x
    }

    /// Place the caret at char index `idx` for a VISUAL vertical move, REMEMBERING
    /// `goal_x` (the TEXT_LEFT-relative pixel column to stay under across the run).
    /// Unlike [`Self::set_cursor`] this does NOT clear `goal_x`, so consecutive
    /// C-n/C-p keep the same screen column through soft wraps; like it, it leaves
    /// the mark untouched (so Shift+C-n extends the region). The next non-vertical
    /// motion or edit clears `goal_x` via `clear_kill_flag` / `apply_edit`.
    ///
    /// A vertical step aims at a PIXEL column, so `idx` arrives from the layout
    /// oracle's `col_in_row` and can name a position interior to a grapheme
    /// cluster (a cluster's chars each get a slice of its ink — see
    /// `render::assemble_glyph_xs`, and a goal-x can fall in any slice). The snap
    /// to the NEAREST boundary lives HERE, at the one sink every vertical landing
    /// passes through, rather than in the caller, so a future vertical motion
    /// cannot reintroduce a caret between a letter and its accent.
    pub fn set_cursor_visual(&mut self, idx: usize, goal_x: f32) {
        self.last_was_kill = false;
        self.goal_col = None;
        let len = self.rope.len_chars();
        self.cursor = crate::grapheme::snap_nearest(idx.min(len), len, |i| self.rope.char(i));
        self.goal_x = Some(goal_x);
        // A vertical move is not a line-END intent, so it drops any wrap-affinity
        // (mirrors `clear_kill_flag`, which this method deliberately bypasses to
        // KEEP goal_x): landing on a boundary via Up/Down renders on the LOWER row.
        self.affinity = crate::caret::Affinity::Downstream;
        // A vertical move is still MOVEMENT — this deliberately bypasses
        // `clear_kill_flag` to keep `goal_x`, but the list-continuation
        // provenance flag has no such exemption.
        self.list_continuation_generated = false;
    }

    /// Delete the active selection (if any) and place the cursor at its start.
    /// Returns true if something was deleted. Used before self-insert / yank so
    /// typing replaces the selection (modern editor behavior).
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            let before = self.cursor;
            self.anchor = None;
            self.goal_col = None;
            self.apply_edit(start, end - start, "", before, start);
            true
        } else {
            self.anchor = None;
            false
        }
    }

    /// M-w: copy the active selection into the kill buffer, leaving text intact
    /// and clearing the mark. No-op (clears mark) when there is no selection.
    pub fn copy_region(&mut self) {
        self.clear_kill_flag();
        if let Some((start, end)) = self.selection_range() {
            self.kill = self.rope.slice(start..end).to_string();
        }
        self.anchor = None;
    }

    /// C-w: kill (cut) the active selection into the kill buffer and remove it
    /// from the buffer, placing the cursor at the region start.
    pub fn kill_region(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            self.kill = self.rope.slice(start..end).to_string();
            let before = self.cursor;
            self.anchor = None;
            self.goal_col = None;
            // A region kill is its own atomic undo group.
            self.seal_undo_group();
            self.apply_edit(start, end - start, "", before, start);
            self.seal_undo_group();
        } else {
            self.anchor = None;
            self.goal_col = None;
        }
        // A region kill does not chain with C-k line kills.
        self.last_was_kill = false;
    }
}
