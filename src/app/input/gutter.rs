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

/// A press-armed candidate DRAG on a working-set `File` row — snapshotted at
/// press ([`App::gutter_stack_click`]'s File-switch arm) and carried until
/// release ([`App::end_row_drag`]), mirroring [`super::drags::RangeDrag`]'s
/// own "arm at press, resolve every move, settle on release" shape.
///
/// A press on this row does NOT switch the file immediately — unlike the
/// close zone and the `More` row, which stay instant — because it does not
/// yet know whether the gesture is a click or the start of a drag. Deferring
/// mirrors [`super::mouse::press_at_char`]'s own phantom-selection-click fix
/// for TEXT (`PointerInput::arm_text_drag_if_moved`): switching immediately
/// on press, the way the resting/expanded panel's own reveal logic can, would
/// let content relocate under a still-down pointer before the drag even
/// starts (`on_active_changed`'s expanded-panel reveal moves `scroll`, not
/// just the highlighted row) — the exact "content relocating under a
/// stationary pointer" class this codebase already fights elsewhere.
#[derive(Clone, Debug)]
pub(super) struct RowDrag {
    /// The dragged file's identity — resolved once, at press. Its OWN root
    /// (the group a drop clamps into) is re-derived from this key on release
    /// through the same door [`App::gutter_stack_row_drop`] uses, rather than
    /// snapshotted here too — one owner of "which root does this file
    /// belong to", not two copies that could drift if the file's remembered
    /// root ever changed mid-gesture.
    pub(super) key: crate::buffers::BufferKey,
    pub(super) press_px: (f32, f32),
    /// The drawn row the press landed on, in whichever presentation was
    /// showing at press time (resting stack or expanded panel) — carried
    /// rather than re-hit-tested, so a NEVER-armed release can replay the
    /// deferred click through the EXACT row the press already resolved.
    pub(super) press_row: usize,
    /// Has the pointer travelled past [`super::DRAG_ARM_SLOP_PX`] since
    /// press? `false` the whole gesture means "this was a click".
    pub(super) armed: bool,
    /// The CURRENT drop target, group-relative — updated on every armed move
    /// through [`crate::workingset::WorkingSet::reorder_target`], so release
    /// only ever replays a target that door already clamped into the
    /// dragged file's own group.
    pub(super) target: usize,
}

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
        // Window-aware: `stack_rows` draws `group(root)[start..]`, so row 0 of
        // the drawn stack is `group[start]`, not `group[0]`, once the
        // hold-still window has slid away from the top
        // (`WorkingSet::resting_row_index`'s own doc).
        let at = working.resting_row_index(root, row)?;
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

    /// **GIVEN A RECOGNIZED DRAG-AND-DROP**: move the file the drawn row
    /// `from_row` names to the drop slot the drawn row `to_row` currently
    /// indicates, within its own root's group. GPU-free — both rows resolve
    /// through the same row→file/row→slot doors the click and hover routes
    /// already use ([`Self::gutter_stack_row_file`],
    /// [`crate::workingset::WorkingSet::reorder_target`]), so the live
    /// pointer machinery ([`Self::on_row_drag`], [`Self::end_row_drag`]) only
    /// has to resolve two row indices from real pixel geometry (live-only,
    /// `docs/harness-reach.md` — no capture door drives a pointer) and hand
    /// them here. WHERE a drop lands, and that it never crosses out of the
    /// dragged file's own group, is proved at THIS seam instead — without a
    /// window.
    ///
    /// `false` when `from_row` names no file (a `More`/`Group` row, or a row
    /// past the drawn window); the caller is expected to have already gated
    /// on `hit.kind == StackRowKind::File` and `!hit.is_close()` before ever
    /// arming a drag, exactly as [`Self::gutter_stack_click`] does for an
    /// ordinary switch.
    pub(in crate::app) fn gutter_stack_row_drop(&mut self, from_row: usize, to_row: usize) -> bool {
        let Some((key, root)) = self
            .gutter_stack_row_file(from_row)
            .map(|f| (f.key.clone(), f.root.clone()))
        else {
            return false;
        };
        let working = self.document.working_set();
        let Some(from) = working.group_index_of(&root, &key) else {
            return false;
        };
        let to = working.reorder_target(&root, to_row);
        self.document
            .working_set_mut()
            .reorder_in_group(&root, from, to);
        self.sync_view(true);
        self.request_frame();
        true
    }

    /// CLICK-TO-SWITCH/EXPAND/COLLAPSE, and the DRAG-ARM, on the working-set
    /// margin: hit-test the pointer against the stack's OWN row geometry
    /// (`TextPipeline::gutter_stack_hit`, which folds in the whole
    /// shown/hidden gate — no page mode, no name, an open overlay, a margin
    /// under the floor and a single-file margin all return `None`) and act on
    /// what the row NAMES:
    ///
    /// * a FILE row's switch half ARMS a [`RowDrag`] rather than switching on
    ///   the spot — the deferral [`RowDrag`]'s own doc explains. Release
    ///   ([`Self::end_row_drag`]) either replays the switch (never armed) or
    ///   performs the reorder (armed past the drag slop);
    /// * a FILE row's right-edge CLOSE zone routes to [`App::close_buffer`]
    ///   IMMEDIATELY, on press — the one removal owner, and never a drag
    ///   handle: pressing the close mark always means close;
    /// * the one `More` row EXPANDS the transient scrollable panel
    ///   ([`crate::workingset::WorkingSet::expand`]), immediately;
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
    /// file is always the active one. Its switch half still arms a
    /// [`RowDrag`], harmlessly: [`crate::workingset::WorkingSet::reorder_in_group`]
    /// is a no-op below two files, so a lone row cannot be dragged anywhere —
    /// only clicked, on release.
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
                } else if let Some(file) = self.gutter_stack_row_file(hit.row) {
                    self.input.pointer.row_drag = Some(RowDrag {
                        key: file.key.clone(),
                        press_px: (px, py),
                        press_row: hit.row,
                        armed: false,
                        target: hit.row,
                    });
                }
            }
        }
        true
    }

    /// LIVE row-drag step: arm the drag once the pointer has travelled past
    /// [`super::DRAG_ARM_SLOP_PX`] (mirroring text-selection's own
    /// phantom-click fix), then, once armed, re-resolve the drop target and
    /// mirror it into the pipeline's quiet insertion indicator. A no-op
    /// before the slop is crossed — exactly like a text drag, a still-armed
    /// press stays a pending click until real motion says otherwise.
    pub(in crate::app) fn on_row_drag(&mut self) {
        let Some(mut drag) = self.input.pointer.row_drag.clone() else {
            return;
        };
        if !drag.armed {
            let (px0, py0) = drag.press_px;
            let (px, py) = self.input.pointer.cursor_px;
            let (dx, dy) = (px - px0, py - py0);
            if dx * dx + dy * dy > crate::app::DRAG_ARM_SLOP_PX.powi(2) {
                drag.armed = true;
            }
        }
        if drag.armed {
            drag.target = self.drag_target_row();
            if let Some(gpu) = self.frame.gpu_mut() {
                gpu.pipeline.set_gutter_drag_indicator(Some(drag.target));
            }
            self.request_frame();
        }
        self.input.pointer.row_drag = Some(drag);
    }

    /// THE DRAWN ROW under the live pointer, for the drag target — reuses the
    /// SAME hit-test the click/hover routes read
    /// ([`crate::render::TextPipeline::gutter_stack_hit`]) so a drop can never
    /// disagree with what the reader sees under the pointer. That hit-test
    /// answers `None` off a real row (above/below the block, or over a
    /// `Group` heading, which carries no row target of its own —
    /// `render::chrome::gutter_hit::stack_hit_from_plan`); this falls back to
    /// the block's own top/bottom edge when the pointer is clearly above or
    /// below it, and otherwise leaves the PREVIOUS target alone (never
    /// guesses a row from partial geometry) — [`crate::workingset::WorkingSet::reorder_target`]
    /// clamps whatever row this returns into the dragged file's own group
    /// regardless, so an imprecise mid-panel guess here can degrade the
    /// indicator's exact resting slot but never let a drop cross groups.
    fn drag_target_row(&self) -> usize {
        let (px, py) = self.input.pointer.cursor_px;
        let Some(gpu) = self.frame.gpu() else {
            return 0;
        };
        if let Some(hit) = gpu.pipeline.gutter_stack_hit(px, py, gpu.config.height) {
            return hit.row;
        }
        match gpu.pipeline.gutter_stack_bounds(gpu.config.height) {
            Some([_, y, _, _]) if py < y => 0,
            Some(_) => usize::MAX / 2,
            None => self
                .input
                .pointer
                .row_drag
                .as_ref()
                .map(|d| d.target)
                .unwrap_or(0),
        }
    }

    /// FINISH a row-drag gesture on button RELEASE: an UNARMED drag (the
    /// pointer never crossed the slop) replays the deferred switch through
    /// the exact row the press already resolved — `gutter_stack_click`'s own
    /// former immediate behavior, just moved to release; an ARMED drag
    /// performs the reorder instead, through [`Self::gutter_stack_row_drop`],
    /// and does NOT also activate the dragged row (a drag reorders; it does
    /// not switch — [`crate::workingset::WorkingSet::reorder_in_group`]'s own
    /// doc). Either way, clears the pipeline's insertion indicator and the
    /// drag state.
    pub(in crate::app) fn end_row_drag(&mut self) {
        let Some(drag) = self.input.pointer.row_drag.take() else {
            return;
        };
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_gutter_drag_indicator(None);
        }
        if drag.armed {
            self.gutter_stack_row_drop(drag.press_row, drag.target);
        } else if self.document.working_set().index_of(&drag.key).is_some() {
            // The KEY was already resolved at press (`RowDrag::key`), so
            // release needs no row re-resolution at all — unlike the old
            // press-time immediate switch, which had to resolve the key
            // BEFORE collapsing the panel because the panel's own scrolled
            // index space was what the row had been resolved against. The
            // `index_of` check above is a defensive no-op guard (a drag
            // cannot close anything) rather than a load-bearing one.
            if self.document.working_set().is_expanded() {
                self.document.working_set_mut().collapse();
            }
            self.activate_open_buffer(drag.key);
        }
        self.resync_pointer_derived_state();
        self.request_frame();
    }

    /// Abort an in-flight row drag without performing either the click or the
    /// reorder — the pointer left the window, or the view it was resolved
    /// against changed underneath it (Escape collapsing an open panel mid-drag).
    /// A no-op when no drag is live.
    pub(in crate::app) fn abort_row_drag(&mut self) {
        if self.input.pointer.row_drag.take().is_some()
            && let Some(gpu) = self.frame.gpu_mut()
        {
            gpu.pipeline.set_gutter_drag_indicator(None);
        }
    }
}

#[cfg(test)]
mod tests;
