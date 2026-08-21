//! src/app/input/gutter.rs — CLICK-TO-SWITCH on the margin's working set.
//!
//! The bottom-left identity widens into a stack of the project's open files;
//! this is the pointer half of that surface. It sits beside [`super::mouse`]
//! rather than inside it because the question is a routing one — which OPEN
//! FILE a row names — and answering it needs the working set rather than
//! anything about the document under the pointer.
//!
//! Split exactly the way [`super::mouse`]'s outline route is: the pixel half is
//! live-only (a hit-test needs a real renderer, and no capture door drives a
//! pointer — `docs/harness-reach.md`), so the ROW→FILE half is a separate,
//! GPU-free method that a unit law can drive.

use crate::app::*;

impl App {
    /// THE FILE A DRAWN STACK ROW NAMES, or `None` when the row names no file
    /// on disk.
    ///
    /// `row` indexes the drawn stack, so it is resolved through the SAME
    /// `group(root)` filter [`crate::workingset::WorkingSet::stack_rows`] built
    /// those rows from — and through the same root: the ACTIVE FILE's
    /// remembered root, which is what the renderer asked for. Resolving against
    /// `project_location.root` instead would agree almost always and name a
    /// different file exactly when a cross-root buffer is active, which is the
    /// case the remembered root exists for.
    ///
    /// `None` for the path-less SCRATCH row. Switching uses the row's key, so
    /// this path-only projection remains only for tests and path assertions.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::app) fn gutter_stack_row_path(&self, row: usize) -> Option<PathBuf> {
        let working = self.document.working_set();
        let root = working.active_root()?;
        let at = *working.group(root).get(row)?;
        working.files().get(at)?.path.clone()
    }

    /// THE BUFFER A DRAWN STACK ROW NAMES — the CLOSE route's own resolution.
    ///
    /// Resolved through the same `group(root)` filter and the same remembered
    /// root as [`Self::gutter_stack_row_path`], because the two must agree about
    /// which file a given row index is: a switch and a close that disagreed by
    /// one would close the file next to the one the pointer was over.
    ///
    /// Unlike the path route this answers for the SCRATCH row too. Its registry
    /// identity is the activation and close handle.
    pub(in crate::app) fn gutter_stack_row_key(
        &self,
        row: usize,
    ) -> Option<crate::buffers::BufferKey> {
        let working = self.document.working_set();
        let root = working.active_root()?;
        let at = *working.group(root).get(row)?;
        Some(working.files().get(at)?.key.clone())
    }

    /// CLICK-TO-SWITCH on a working-set row: hit-test the pointer against the
    /// stack's OWN row geometry (`TextPipeline::gutter_stack_hit`, which folds
    /// in the whole shown/hidden gate — no page mode, no name, an open overlay,
    /// a margin under the floor and a single-file margin all return `None`) and,
    /// on a hit, open that row's file.
    ///
    /// Through [`App::load_path`] — THE file-open door every picker selection,
    /// the Last-file toggle and the daemon handoff already share — so a row
    /// switch is the same transition as opening the file any other way, down to
    /// the arriving document restoring its own project root. Inventing a second
    /// switching path here would be a second answer to "what does opening a
    /// file mean".
    ///
    /// Returns whether the press landed on a row, so the caller skips the
    /// document press. A row with no file to open still returns `true`: the
    /// press was consumed by the margin, and falling through would place the
    /// caret from a click the reader aimed at chrome.
    ///
    /// The row's RIGHT-EDGE close zone routes to [`App::close_buffer`] — the one
    /// removal owner — and the rest of the band still switches. Both meanings
    /// come from [`crate::render::chrome::gutter_stack::row_intent`], the same
    /// classifier any drawn affordance reads, so what the pointer accepts and
    /// what the reader is shown cannot disagree once that affordance exists.
    ///
    /// A close targets the NAMED buffer: an inactive row closes its own file
    /// without first activating it, so the reader's document does not change
    /// underneath a click aimed at a different one.
    pub(in crate::app) fn gutter_stack_click(&mut self) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let hit = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.gutter_stack_hit(px, py, g.config.height));
        let Some(hit) = hit else {
            return false;
        };
        if hit.is_close() {
            if let Some(key) = self.gutter_stack_row_key(hit.row) {
                self.close_buffer(key);
            }
        } else if let Some(key) = self.gutter_stack_row_key(hit.row) {
            self.activate_open_buffer(key);
        }
        true
    }
}

#[cfg(test)]
mod tests;
