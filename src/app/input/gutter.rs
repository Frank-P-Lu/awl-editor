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
    /// THE OPEN FILE A DRAWN STACK ROW NAMES, resolved against WHICHEVER of
    /// the two margin presentations is currently drawn.
    ///
    /// The RESTING stack resolves `row` through the SAME `group(root)` filter
    /// [`crate::workingset::WorkingSet::stack_rows`] built its rows from — and
    /// through the same root: the ACTIVE FILE's remembered root, which is what
    /// the renderer asked for. Resolving against `project_location.root`
    /// instead would agree almost always and name a different file exactly
    /// when a cross-root buffer is active, which is the case the remembered
    /// root exists for.
    ///
    /// The EXPANDED panel resolves `row` through
    /// [`crate::workingset::WorkingSet::expanded_row_open_file`] instead — a
    /// DIFFERENT index space (the panel's own scrolled, multi-root row list,
    /// never `group(root)`), because the drawn rows there are not one root's
    /// group. `None` for a heading row, a row past the end of whichever list is
    /// drawn, and the path-less SCRATCH row when the caller asked for a path.
    fn gutter_stack_row_file(&self, row: usize) -> Option<&crate::workingset::OpenFile> {
        let working = self.document.working_set();
        if working.is_expanded() {
            return working.expanded_row_open_file(row);
        }
        let root = working.active_root()?;
        let at = *working.group(root).get(row)?;
        working.files().get(at)
    }

    /// Switching uses the row's key, so this path-only projection remains only
    /// for tests and path assertions.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::app) fn gutter_stack_row_path(&self, row: usize) -> Option<PathBuf> {
        self.gutter_stack_row_file(row)?.path.clone()
    }

    /// THE BUFFER A DRAWN STACK ROW NAMES — the CLOSE route's own resolution.
    ///
    /// Resolved through the exact same door as [`Self::gutter_stack_row_path`],
    /// because the two must agree about which file a given row index is: a
    /// switch and a close that disagreed by one would close the file next to
    /// the one the pointer was over.
    ///
    /// Unlike the path route this answers for the SCRATCH row too. Its registry
    /// identity is the activation and close handle.
    pub(in crate::app) fn gutter_stack_row_key(
        &self,
        row: usize,
    ) -> Option<crate::buffers::BufferKey> {
        Some(self.gutter_stack_row_file(row)?.key.clone())
    }

    /// CLICK-TO-SWITCH/EXPAND/COLLAPSE on the working-set margin: hit-test the
    /// pointer against the stack's OWN row geometry
    /// (`TextPipeline::gutter_stack_hit`, which folds in the whole
    /// shown/hidden gate — no page mode, no name, an open overlay, a margin
    /// under the floor and a single-file margin all return `None`) and act on
    /// what the row NAMES:
    ///
    /// * a FILE row's switch half opens that file (through
    ///   [`App::activate_open_buffer`] — THE existing already-open door, which
    ///   already restores the file's own remembered root via
    ///   `finish_buffer_activation`, so a cross-root row needs no second
    ///   restoration path here);
    /// * a FILE row's right-edge CLOSE zone routes to [`App::close_buffer`] —
    ///   the one removal owner;
    /// * the one `More` row EXPANDS the transient scrollable panel
    ///   ([`crate::workingset::WorkingSet::expand`]);
    /// * a `Group` heading is inert (filtered out before this function ever
    ///   sees it — `render::chrome::gutter_hit::stack_hit_from_plan`);
    /// * a press that misses the block ENTIRELY while the panel is open is a
    ///   CLICK-AWAY: it collapses the panel and is swallowed, mirroring the
    ///   awl-drawn menu bar's own click-away contract
    ///   (`app/input/mouse.rs::menubar_press`) rather than inventing a second
    ///   one.
    ///
    /// Returns whether the press was consumed by the margin, so the caller
    /// skips the document press. A row with no file to open still returns
    /// `true`: the press was consumed by the margin, and falling through would
    /// place the caret from a click the reader aimed at chrome.
    ///
    /// A close targets the NAMED buffer: an inactive row closes its own file
    /// without first activating it, so the reader's document does not change
    /// underneath a click aimed at a different one.
    ///
    /// A SINGLE-FILE margin resolves through this exact door too
    /// (`TextPipeline::gutter_stack_hit` answers `row: 0` for the identity
    /// line's own close zone) — closing it is the pointer's route to the same
    /// zero-document start surface ⌘W already reaches, since the sole open
    /// file is always the active one.
    pub(in crate::app) fn gutter_stack_click(&mut self) -> bool {
        let (px, py) = self.input.pointer.cursor_px;
        let hit = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.gutter_stack_hit(px, py, g.config.height));
        let Some(hit) = hit else {
            if self.document.working_set().is_expanded() {
                self.document.working_set_mut().collapse();
                self.sync_view(true);
                self.request_frame();
                return true;
            }
            return false;
        };
        match hit.kind {
            crate::workingset::StackRowKind::More { .. } => {
                self.document.working_set_mut().expand();
                self.sync_view(true);
                self.request_frame();
            }
            // Filtered out before the hit-test ever answers a row for it
            // (`stack_hit_from_plan`); kept as a named, no-op arm rather than a
            // wildcard so a future row kind cannot fall silently through here.
            crate::workingset::StackRowKind::Group { .. } => {}
            crate::workingset::StackRowKind::File => {
                if hit.is_close() {
                    if let Some(key) = self.gutter_stack_row_key(hit.row) {
                        self.close_buffer(key);
                    }
                } else if let Some(key) = self.gutter_stack_row_key(hit.row) {
                    // Resolve BEFORE collapsing: the expanded panel's own
                    // scrolled index space is what `hit.row` was resolved
                    // against, so the panel must still be open when the key
                    // above is looked up.
                    if self.document.working_set().is_expanded() {
                        self.document.working_set_mut().collapse();
                    }
                    self.activate_open_buffer(key);
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests;
