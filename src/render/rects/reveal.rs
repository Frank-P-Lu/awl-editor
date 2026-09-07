//! WHETHER A LINE IS SHOWING ITS RAW SOURCE — the one owner of that
//! question for whole-line consumers, and the reason it is a module rather
//! than two more methods in `rects.rs`: that file is far past its size
//! ceiling, and this is a self-contained rule with its own subject.
//!
//! Two readers ask it about a thematic break — the fleuron's own draw gate
//! (`rule_lines`) and the nit underline's conceal check — and when they
//! derived it separately they disagreed: a selection-revealed `---` drew its
//! markup with the nit under it suppressed. `wysiwyg_reveals` is the same
//! rule at SPAN scope; this is its line-scoped form.

use super::*;

impl TextPipeline {
    /// The byte extent of every line the ACTIVE SELECTION touches — computed
    /// ONCE by a caller that is about to ask [`Self::line_is_revealed`] about
    /// several lines, since deriving it per line re-walks the rope.
    pub(super) fn selection_touch(&self) -> Option<std::ops::Range<usize>> {
        selection_touch_bytes(
            self.selection,
            |li| self.line_doc_byte_start(li),
            |li| {
                self.buffer
                    .lines
                    .get(li)
                    .map_or(0, |line| line.text().len())
            },
        )
    }

    /// THE reveal test for a whole LINE — caret on it, or the selection
    /// touching it — the same "caret line OR selection touch" rule
    /// [`crate::render::spans::wysiwyg_reveals`] applies to a span. ONE owner, because
    /// two readers of a line's reveal state that derive it separately drift:
    /// the rule ornament's own draw gate and the nit underline's conceal check
    /// each answer "is this thematic break showing its raw source", and a
    /// widening applied to one alone leaves a revealed `---` line drawing its
    /// markup with the nit under it suppressed.
    pub(super) fn line_is_revealed(
        &self,
        li: usize,
        selection_touch: Option<&std::ops::Range<usize>>,
    ) -> bool {
        if li == self.cursor_line {
            return true;
        }
        let start = self.line_doc_byte_start(li);
        let end = start + self.buffer.lines.get(li).map_or(0, |l| l.text().len());
        selection_touches(selection_touch, &(start..end))
    }
}
