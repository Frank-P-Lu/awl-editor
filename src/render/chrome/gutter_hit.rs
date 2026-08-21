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
/// that row. `row` indexes the DRAWN STACK — the same list
/// `crate::workingset::WorkingSet::stack_rows` produced — never the block's
/// line numbering, which the optional `changed elsewhere` line shifts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GutterStackHit {
    pub row: usize,
    pub intent: RowIntent,
}

/// Resolve a pointer against an already-planned block. Kept pure so the live
/// hover/click enrolment can be swept without constructing a GPU pipeline; the
/// production method below supplies the exact layout/plan it draws.
pub(super) fn stack_hit_from_plan(
    layout: &GutterLayout,
    plan: &crate::render::plan::GutterStackPlan,
    px: f32,
    py: f32,
) -> Option<GutterStackHit> {
    let line = plan.hit_row(px, py)?;
    let GutterLine::File(row) = layout.lines().get(line)?.1 else {
        return None;
    };
    if !matches!(
        layout.files.get(row)?.kind,
        crate::workingset::StackRowKind::File
    ) {
        return None;
    }
    Some(GutterStackHit {
        row,
        intent: gutter_stack::row_intent(*plan.rows.get(line)?, px),
    })
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
    /// are laid from. Both routes below read it, so neither hit-tests against
    /// geometry the other does not.
    fn gutter_hit_plan(
        &self,
        height: u32,
    ) -> Option<(GutterLayout, crate::render::plan::GutterStackPlan)> {
        let layout = self.gutter_layout()?;
        let plan = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            self.metrics.line_height * crate::markdown::type_scale::LABEL,
            layout.lines().len(),
            self.metrics.px_physical(super::readout::CANVAS_INSET),
            super::gutter::GUTTER_CARVE_BREATH.0,
        );
        Some((layout, plan))
    }

    /// Hit-test the two identity rows from the exact layout that draws them.
    pub fn gutter_context_target(
        &self,
        px: f32,
        py: f32,
        height: u32,
    ) -> Option<crate::context_menu::ContextTarget> {
        let (layout, plan) = self.gutter_hit_plan(height)?;
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

    /// CLICK-TO-SWITCH on a working-set row: which drawn stack row the pointer
    /// is over, and whether it is over that row's close zone.
    ///
    /// `None` for every line that is not a working-set row — the `changed
    /// elsewhere` affordance, the project line, and the LONE filename of a
    /// single-file margin, which is not a stack and has nothing to switch to.
    /// So a one-file window's pointer behaviour is exactly what it was before
    /// this surface existed.
    pub fn gutter_stack_hit(&self, px: f32, py: f32, height: u32) -> Option<GutterStackHit> {
        let (layout, plan) = self.gutter_hit_plan(height)?;
        stack_hit_from_plan(&layout, &plan, px, py)
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
