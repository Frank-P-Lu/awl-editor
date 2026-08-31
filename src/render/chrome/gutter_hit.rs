//! THE GUTTER'S POINTER ROUTES — what a click at `(px, py)` in the bottom-left
//! identity block is aiming at.
//!
//! Separate from [`super::gutter`] (which owns the block) and
//! [`super::gutter_stack`] (which owns a row) because a pointer asks a third
//! question neither of them does: not "what is drawn" but "what did the reader
//! mean". Both routes below start from the SAME
//! [`TextPipeline::gutter_layout`] the frame was composed from, so a target can
//! never sit where nothing is drawn — including the hidden arms, which that one
//! owner already answers `None` for (no page mode, no name, an open overlay, a
//! margin under the floor).

use super::gutter::GutterLine;
use super::gutter_stack::RowIntent;
use super::*;

/// WHICH WORKING-SET ROW the pointer is over, and what it is aiming at within
/// that row. `row` indexes `crate::workingset::WorkingSet::group(root)` — the
/// SAME slots `stack_rows` draws from — never the block's line numbering,
/// which the optional `changed elsewhere`/project lines shift. A single-file
/// margin has no drawn stack, but `group(root)` still holds exactly the one
/// slot the identity line names, so [`GutterLine::Name`] resolves to `row: 0`
/// through the identical index the App's row→file lookups already read
/// (`App::gutter_stack_row_key`) — one door for both shapes, not a second one
/// for the single-file case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GutterStackHit {
    pub row: usize,
    /// The row's own [`crate::workingset::StackRowKind`] — the pointer route's
    /// answer to "what kind of row is this", read off the SAME drawn
    /// [`StackRow`](crate::workingset::StackRow) `row` indexes rather than
    /// re-derived. A `More` row never carries a close mark
    /// ([`super::gutter_stack::stack_spans`]'s own gate); a `Group` heading's
    /// own mark closes its whole group rather than itself. Either way a
    /// caller branches on `kind` BEFORE trusting [`Self::is_close`] — `intent`
    /// is purely geometric and does not know what a close means for this row.
    pub kind: crate::workingset::StackRowKind,
    pub intent: RowIntent,
}

/// Resolve a pointer against an already-planned block. Kept pure so the live
/// hover/click enrolment can be swept without constructing a GPU pipeline; the
/// production method below supplies the exact layout/plan/char-width it draws.
///
/// `label_char_w` is the LABEL-scale advance the close zone's ink-width
/// estimate is built from — the SAME quantity [`gutter_stack::plate_rects`]
/// already multiplies a row's char count by, so a target derived here can
/// never disagree with the fill drawn from that other door.
pub(super) fn stack_hit_from_plan(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    label_char_w: f32,
    px: f32,
    py: f32,
) -> Option<GutterStackHit> {
    let line = plan.hit_row(px, py)?;
    let band = *plan.rows.get(line)?;
    let mark_chars = gutter_stack::CLOSE_MARK_TEXT.chars().count();
    // Resolve WHICH row/kind this line is, and how many characters of ink it
    // shapes, BEFORE classifying the pointer: the close zone now anchors on
    // that row's own ink width (`row_intent`'s own doc), so the row has to be
    // known first rather than intent computed off the band alone.
    let (row, kind, chars) = match layout.lines().get(line)?.1 {
        GutterLine::File(row) => {
            let file = layout.files.get(row)?;
            (row, file.kind, file.text.chars().count())
        }
        // The single-file identity names the same lone slot `group(root)`
        // would draw as row 0 of a stack, whether or not the margin was ever
        // wide enough to draw one — so it enrols in the SAME close/switch
        // geometry a working-set row does rather than staying an inert label.
        GutterLine::Name => (
            0,
            crate::workingset::StackRowKind::File,
            layout.name.chars().count(),
        ),
        GutterLine::Project | GutterLine::Changed => return None,
    };
    let text_w = (chars + mark_chars) as f32 * label_char_w;
    let intent = gutter_stack::row_intent(band, text_w, px);
    match kind {
        // A project HEADING carries no switch target of its own (the
        // design boundary: mark the active group, never make the
        // heading itself a target) — a press elsewhere on it stays
        // click-away, exactly as it always was. Its own close zone is
        // the one exception: closing a heading closes its whole group,
        // so it enrols for that intent alone.
        crate::workingset::StackRowKind::Group { .. } if intent == RowIntent::Close => {}
        crate::workingset::StackRowKind::Group { .. } => return None,
        // A passive scroll-position cue is inert in both halves.
        crate::workingset::StackRowKind::Overflow { .. } => return None,
        _ => {}
    }
    Some(GutterStackHit { row, kind, intent })
}

impl GutterStackHit {
    /// Did the pointer land on the row's CLOSE zone rather than its switch
    /// half? Asked here rather than by re-exporting [`RowIntent`] for the App to
    /// match on: the classifier stays private to the renderer that lays the row
    /// out, and the pointer route reads one predicate off the hit it already
    /// holds instead of a second copy of the same arithmetic.
    pub fn is_close(&self) -> bool {
        matches!(self.intent, RowIntent::Close)
    }
}

impl TextPipeline {
    /// The block's planner rows for this frame, off the SAME layout the glyphs
    /// are laid from, plus the LABEL-scale char width the close zone's ink
    /// estimate needs (`stack_hit_from_plan`'s own doc). Both routes below
    /// read it, so neither hit-tests against geometry the other does not.
    fn gutter_hit_plan(
        &self,
        height: u32,
    ) -> Option<(GutterLayout, crate::render::plan::GutterStackPlan, f32)> {
        let layout = self.gutter_layout()?;
        let plan = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            self.metrics.line_height * crate::markdown::type_scale::LABEL,
            layout.lines().len(),
            self.metrics.px_physical(super::readout::CANVAS_INSET),
            super::gutter::GUTTER_CARVE_BREATH.0,
        );
        let label_char_w = self.metrics.char_width * crate::markdown::type_scale::LABEL;
        Some((layout, plan, label_char_w))
    }

    /// THE MARGIN STACK'S OWN BOUNDING BAND `[left, top, right, bottom]` — the
    /// same rect the lava carve uses ([`Self::gutter_carve_rect`]), exposed
    /// here because the wheel route needs the same region a scroll gesture
    /// must land inside to move the expanded panel rather than the document
    /// sitting behind it. `None` when the block is hidden.
    pub fn gutter_stack_bounds(&self, height: u32) -> Option<[f32; 4]> {
        self.gutter_carve_rect(height)
    }

    /// Hit-test the two identity rows from the exact layout that draws them.
    pub fn gutter_context_target(
        &self,
        px: f32,
        py: f32,
        height: u32,
    ) -> Option<crate::context_menu::ContextTarget> {
        let (layout, plan, _) = self.gutter_hit_plan(height)?;
        // Hit-test against the SAME ordered line list the block is drawn from, so
        // an added line can never shift a target silently: the affordance itself
        // is a LABEL, not a target — it names a state, and the two things you can
        // do about that state are named palette rows, not a click here.
        let row = plan.hit_row(px, py)?;
        match layout.lines().get(row)?.1 {
            GutterLine::Name => Some(crate::context_menu::ContextTarget::Filename),
            // The ACTIVE row is the same target the lone filename is — it names
            // the same buffer, so the identity menu it has always opened keeps
            // working when the identity widens. An INACTIVE row names a buffer
            // that is not the active one, and every action on that menu operates
            // on the active document; returning `Filename` here would point the
            // reader at one file and rename another. Until a row can carry its
            // own named-buffer target, it carries none.
            GutterLine::File(at) => layout
                .files
                .get(at)
                .filter(|line| line.active)
                .map(|_| crate::context_menu::ContextTarget::Filename),
            GutterLine::Project => Some(crate::context_menu::ContextTarget::Folder),
            // The affordance is a LABEL, not a target: it names a state, and the
            // three things you can do about that state are named palette rows.
            GutterLine::Changed => None,
        }
    }

    /// CLICK-TO-SWITCH/CLOSE on an identity row: which row the pointer is
    /// over (`group(root)`-indexed), and whether it is over that row's close
    /// zone.
    ///
    /// `None` for every line that names no file — the `changed elsewhere`
    /// affordance and the project heading. The LONE filename of a single-file
    /// margin now resolves too (`row: 0`, [`GutterLine::Name`] in
    /// [`stack_hit_from_plan`]): it carries the same close mark a working-set
    /// row does, so a one-file window's pointer accepts a close exactly where
    /// the mark is drawn. Switching a single-file row is a no-op past the
    /// resolve (there is nowhere else to switch to), never a second code path.
    pub fn gutter_stack_hit(&self, px: f32, py: f32, height: u32) -> Option<GutterStackHit> {
        let (layout, plan, label_char_w) = self.gutter_hit_plan(height)?;
        stack_hit_from_plan(&layout, &plan, label_char_w, px, py)
    }

    /// Mirror the working-set row under the LIVE pointer into render state.
    ///
    /// The hit itself comes from [`Self::gutter_stack_hit`], so hover enrollment,
    /// click routing and the drawn close mark all read one row/zone geometry.
    /// Returns whether the visible hover state changed, allowing the App to ask
    /// for exactly one repaint on entry, zone crossing, row crossing or exit.
    pub fn resolve_gutter_stack_hover(&mut self, px: f32, py: f32, height: u32) -> bool {
        let next = self.gutter_stack_hit(px, py, height);
        if self.gutter_stack_hover == next {
            return false;
        }
        self.gutter_stack_hover = next;
        true
    }

    /// Clear a live close-mark reveal when the pointer leaves the window.
    pub fn clear_gutter_stack_hover(&mut self) -> bool {
        self.gutter_stack_hover.take().is_some()
    }
}

#[cfg(test)]
mod tests;
