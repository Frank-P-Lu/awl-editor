//! The buffer's side of the accessibility run table.
//!
//! Three of these four are the whole coupling between the rope and
//! [`crate::semantic::runs::RunTable`]: read the lines an edit covers before it
//! happens, resplice the table after it, and read one run's text back. They are
//! `pub(super)` because `buffer/undo.rs`'s three rope-mutation sites are the
//! only callers there may be — a fourth would be a mutation the table never
//! heard about, and `buffer/run_law.rs` is what would notice.

use super::Buffer;

impl Buffer {
    /// This buffer's stable per-line run identity.
    pub fn runs(&self) -> &crate::semantic::runs::RunTable {
        &self.runs
    }

    /// One accessibility RUN's text: the line INCLUDING its trailing `\n` where
    /// it has one. That is what makes the concatenation of every run the
    /// document byte for byte, so adding up per-run grapheme counts gives the
    /// same answer as segmenting the whole document — the identity the
    /// projection's offset arithmetic rests on. Distinct from
    /// [`Buffer::line_text`], which stops before the newline because
    /// smart-newline wants the block prefix, not the break.
    pub fn run_text(&self, line: usize) -> String {
        if line >= self.rope.len_lines() {
            return String::new();
        }
        self.rope.line(line).to_string()
    }

    /// The lines an edit over the char range `start..end` covers, as table
    /// indices, read BEFORE the rope is mutated.
    pub(super) fn line_span_of(&self, start: usize, end: usize) -> (usize, usize) {
        let clamp = |index: usize| self.rope.char_to_line(index.min(self.rope.len_chars()));
        let first = clamp(start);
        (first, clamp(end).max(first))
    }

    /// Record that the lines `first..=last` were replaced by `inserted`.
    pub(super) fn resplice_runs(&mut self, first: usize, last: usize, inserted: &str) {
        self.runs
            .splice(first, last, inserted.matches('\n').count() + 1);
        debug_assert_eq!(
            self.runs.runs().len(),
            self.rope.len_lines().max(1),
            "the run table drifted from the rope's line count",
        );
    }
}
