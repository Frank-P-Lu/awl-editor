//! THE ONE GESTURE THAT REORDERS THE STABLE OPEN ORDER: a pointer drag on a
//! margin stack row. Split out of the working set's own core model file
//! (already at its natural size before this landed) because a reorder is a
//! cohesive unit — a row→group-slot resolver shared with the drag target,
//! the in-group clamp, and the splice itself — rather than three more
//! methods scattered through the core model file.
//!
//! Everything here is pure and GPU-free: the live pointer machinery
//! (`app/input/gutter.rs`) resolves real pixel geometry to row INDICES and
//! hands them to [`WorkingSet::reorder_target`]/[`WorkingSet::reorder_in_group`];
//! nothing in this file knows a window exists.

use std::path::Path;

use crate::buffers::BufferKey;

use super::panel::{DrawnRow, Panel, PanelRow};
use super::{OpenFile, WorkingSet};

impl WorkingSet {
    /// THE `self.files` SLOT a drawn RESTING-STACK row names, window-aware —
    /// the fix for a real bug [`WorkingSet::stack_rows`]'s own hold-still
    /// window exposed: that method draws `group[start..]`, so row 0 of the
    /// drawn stack is `group[start]`, not `group[0]`, whenever the window has
    /// slid away from the top. A resolver that indexed `group(root)` directly
    /// (this method's predecessor) agreed with the draw only while `start`
    /// happened to be `0` — dormant until a root's group ever grew past
    /// [`super::RESTING_FILES`] and the active file forced a slide.
    ///
    /// Deliberately answers for a group of ONE file too (`row: 0`): the
    /// single-file identity line resolves through this exact door
    /// (`GutterLine::Name` → row 0,
    /// `render::chrome::gutter_hit::stack_hit_from_plan`) even though
    /// [`WorkingSet::stack_rows`] itself draws no STACK for a group that small
    /// — "is a row drawn" and "is a stack drawn" are different questions, and
    /// only the second one gates on group length.
    pub fn resting_row_index(&self, root: &Path, row: usize) -> Option<usize> {
        let group = self.group(root);
        if group.is_empty() {
            return None;
        }
        let start = self.resting_start(root, &group);
        group.get(start + row).copied()
    }

    /// The group-relative slot `key` currently occupies within `root`'s own
    /// group — the origin half of a drag, resolved once the row under the
    /// press has already named a file (through [`WorkingSet::resting_row_index`]
    /// or [`WorkingSet::expanded_row_open_file`]) rather than re-deriving it
    /// from a row index a second way.
    pub fn group_index_of(&self, root: &Path, key: &BufferKey) -> Option<usize> {
        self.group(root)
            .iter()
            .position(|&at| self.files[at].key == *key)
    }

    /// THE DRAG TARGET a drawn row `row` (of WHICHEVER presentation is
    /// currently shown — resting stack or expanded panel) names, for
    /// `origin_root`'s own group — ALWAYS a valid group-relative index, `0`
    /// when the group is empty. **In-group only is enforced HERE**, at the one
    /// place a live drag asks "where does this land", so no caller can forget
    /// the clamp: a row outside `origin_root`'s own block never returns an
    /// index a drag could use to cross into another root's group.
    ///
    /// The RESTING stack draws only the active root's own group, so every row
    /// it shows already belongs to `origin_root` by construction — window-aware
    /// via [`WorkingSet::resting_start`], the same computation
    /// [`WorkingSet::stack_rows`] draws from, so drag and draw can never
    /// disagree about which group slot a given row names.
    ///
    /// The EXPANDED panel mixes roots ([`WorkingSet::expanded_full`]), so a row
    /// outside `origin_root`'s own contiguous block (its heading plus every
    /// one of its own files, always contiguous — see that method's doc)
    /// clamps DIRECTIONALLY to whichever edge of the block the pointer sits on
    /// the near side of: at or above the block's own heading clamps to its
    /// top (group index `0`); at or below its last file clamps to its bottom
    /// (`group.len() - 1`). A row that lands ON the block itself (its heading
    /// or one of its own files) resolves to the exact group slot drawn there.
    ///
    /// `row` is a DRAWN-window index — [`WorkingSet::expanded_window`]'s own
    /// space, not a plain offset from `scroll` — because item 518's pinned
    /// sticky heading can sit ahead of real content and shift that
    /// correspondence. Resolved through [`super::panel::DrawnRow::full_index`],
    /// the SAME mapping [`WorkingSet::expanded_rows`] draws from, so a drag
    /// can never target a different `expanded_full` position than the row
    /// the pointer is actually over.
    pub fn reorder_target(&self, origin_root: &Path, row: usize) -> usize {
        let group = self.group(origin_root);
        let last = group.len().saturating_sub(1);
        if group.is_empty() {
            return 0;
        }
        let Panel::Expanded { .. } = self.panel else {
            let start = self.resting_start(origin_root, &group);
            return (start + row).min(last);
        };
        let full = self.expanded_full();
        let Some(block_start) = full
            .iter()
            .position(|r| matches!(r, PanelRow::Group(root, _) if root == origin_root))
        else {
            return last;
        };
        // A row past the drawn window names no `expanded_full` position of
        // its own — clamp past the block's own end, same as any row below it.
        let absolute = self
            .expanded_window()
            .get(row)
            .and_then(DrawnRow::full_index)
            .unwrap_or(full.len());
        if absolute <= block_start {
            0
        } else {
            // Inside the block, or past its own last file: either way this is
            // "group index `absolute - (heading's own slot)`", clamped — past
            // the block's own end that arithmetic already lands on `last`.
            absolute.saturating_sub(block_start + 1).min(last)
        }
    }

    /// USER-DRIVEN drag-and-drop reorder within one root's group — the ONE
    /// gesture that changes the stable open order (see the module doc:
    /// activation, closing and windowing never do). `from`/`to` are
    /// group-relative slots into `self.group(root)`; the move is a
    /// `Vec::remove` + `Vec::insert` on the group's OWN sequence — `to` is
    /// where the moved file lands in the group's FINAL order, the ordinary
    /// "move to this position" convention (mirrored by, for one, Angular
    /// CDK's `moveItemInArray`). A `from`/`to` both inside the same original
    /// slot leaves every row untouched; `to` clamps into range rather than
    /// panicking on a stale drop target.
    ///
    /// Only positions BELONGING TO `root`'s own group move: every other
    /// root's files keep the exact absolute slot they held in `self.files`
    /// before the call, so a stable open order's promised interleaving with
    /// other projects' rows survives the reorder unchanged — "in-group only"
    /// holds at the STORAGE layer, not only at the gesture layer that calls
    /// this.
    ///
    /// Never touches [`WorkingSet::active_index`]'s FILE identity, even when
    /// the active file is the one that moved: only its absolute slot (and,
    /// through [`WorkingSet::recompute_resting_window`], its position inside
    /// the hold-still window) can shift. A drag reorders the row it grabs; it
    /// does not also activate it, so a background row can be reordered
    /// without disturbing what the reader is looking at.
    pub fn reorder_in_group(&mut self, root: &Path, from: usize, to: usize) {
        let positions = self.group(root);
        if positions.len() < 2 || from >= positions.len() {
            return;
        }
        let to = to.min(positions.len() - 1);
        if from == to {
            return;
        }
        let active_key = self.active.map(|i| self.files[i].key.clone());
        let mut entries: Vec<OpenFile> =
            positions.iter().map(|&at| self.files[at].clone()).collect();
        let moved = entries.remove(from);
        entries.insert(to, moved);
        for (&slot, entry) in positions.iter().zip(entries) {
            self.files[slot] = entry;
        }
        if let Some(key) = active_key {
            self.active = self.files.iter().position(|f| f.key == key);
        }
        self.recompute_resting_window();
    }
}

// Exhaustively tested from `workingset/tests.rs` alongside the rest of the
// model's own laws (shared fixtures — `ten`, `foreign_interleaved` — live
// there too), not a sibling `reorder/tests.rs`: the laws exercise this
// module's methods together with `stack_rows`/`expanded_rows` from the CORE
// file, so one test file reading both is truer to what each law actually
// proves than splitting by implementation file would be.
