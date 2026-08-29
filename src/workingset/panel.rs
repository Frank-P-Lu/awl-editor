//! The margin's transient EXPANDED/GROUPED panel — the overflow row's own
//! scrollable view over every open file, headed by root. Split out of the
//! parent module to keep [`super::WorkingSet`]'s own file under this repo's
//! production line ceiling; the panel's fields and methods stay logically
//! part of `WorkingSet` (a second `impl` block, reaching the parent's private
//! fields the way any descendant module can) rather than a separate type.

use std::path::{Path, PathBuf};

use super::{OpenFile, StackRow, StackRowKind, WorkingSet};

/// THE EXPANDED/GROUPED PANEL'S OWN SCROLLABLE VIEWPORT, in total drawn rows
/// (file rows and folder headings together) — the number judged in the
/// working-set residual-3 gallery pass.
pub const EXPANDED_VIEWPORT: usize = 8;

/// The margin's OWN transient UI state: is the reader looking at the resting
/// five-row stack, or the panel it expands into? Lives beside the order and
/// root state it presents rather than on `App` (which has a hard field
/// ceiling, `app/tests/domains.rs::root_app_does_not_grow`) — and the module's
/// own contract, "what does the reader see, where, and does it stay there",
/// already covers a transient viewport as much as it covers the stable order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum Panel {
    #[default]
    Resting,
    /// `scroll` is the FIRST drawn row's index into [`WorkingSet::expanded_full`]
    /// (headings and files counted together) — never re-derived from the
    /// active file once the panel is open, so a reader's own wheel motion is
    /// never fought (see [`WorkingSet::scroll_expanded`]'s doc).
    Expanded { scroll: usize },
}

/// One row of the EXPANDED panel's full, unwindowed content — every open
/// file, headed by its root, in the SAME first-seen root order the judged
/// gallery's `Grouped` prototype used. Kept distinct from [`StackRow`]: this
/// carries the row's real identity ([`OpenFile`] index or root), which a
/// click needs to resolve and a drawn [`StackRow`] deliberately does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PanelRow {
    Group(PathBuf, bool),
    File(usize),
}

/// One row of the panel's DRAWN WINDOW — what [`WorkingSet::expanded_rows`]
/// turns into a [`StackRow`] and what every drag/reorder resolution maps
/// back to an [`WorkingSet::expanded_full`] position. Distinct from
/// [`PanelRow`] (the unwindowed content) because a drawn window can hold rows
/// `expanded_full` never does: a pinned STICKY heading — a duplicate of a
/// real heading that has already scrolled off, inserted so every visible
/// File row's group stays nameable from the window alone — and a passive
/// scroll-position cue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DrawnRow {
    /// A real `expanded_full` entry, carried alongside its own index there —
    /// the position a drag onto this row resolves to.
    Full(usize, PanelRow),
    /// `origin` is the REAL heading's own `expanded_full` index — dragging
    /// onto the pinned duplicate lands exactly where dragging onto that real
    /// heading would.
    Sticky(usize, PathBuf, bool),
    /// The panel's own passive scroll-position cue — `up: true` at the
    /// window's own first slot when FILE rows are hidden above it, `up:
    /// false` at the last slot when files remain below. `hidden` is a FILE
    /// count ([`window_edge_counts`](crate::render::chrome::window_edge_counts)'s
    /// own contract), never a display-row count — a project heading is not
    /// an item. Names no `expanded_full` position: never a drop target.
    Overflow { up: bool, hidden: usize },
}

impl DrawnRow {
    /// The [`WorkingSet::expanded_full`] position a drag onto this row
    /// resolves to. `None` for the passive overflow cue, which names no real
    /// position — [`WorkingSet::reorder_target`]'s caller can never actually
    /// hand this in: the hit-test every real drag row is resolved through
    /// already filters the cue out before a caller sees it
    /// (`gutter_hit::stack_hit_from_plan`), so this is a documented
    /// impossible case, not a silent wrong answer.
    pub(super) fn full_index(&self) -> Option<usize> {
        match self {
            DrawnRow::Full(i, _) | DrawnRow::Sticky(i, _, _) => Some(*i),
            DrawnRow::Overflow { .. } => None,
        }
    }
}

/// Convert a project heading — natural or [`DrawnRow::Sticky`]'s pinned
/// duplicate — into its drawn [`StackRow`]. The ONE owner both
/// [`WorkingSet::expanded_rows`] arms read, so a heading cannot be spelled
/// two different ways depending on whether it scrolled into view or was
/// pinned there.
fn group_stack_row(root: &Path, active: bool, roots: &[&Path]) -> StackRow {
    StackRow {
        // Trailing `/` for FOLDER identity — the same rule `row_display`
        // already applies to a picker's own folder rows (`row.is_dir` /
        // `RowMeta::GotoFolder`); a project heading is a folder every time,
        // so it always carries one rather than reading as a bare
        // (file-like) name.
        leaf: format!("{}/", crate::project::folder_name(root)),
        parent: group_parent_label(root, roots).unwrap_or_default(),
        // The OUTER field, not just the kind's own copy: this is the one
        // `active: bool` marker `stack_spans`' ink match reads for "is this
        // the current row" (`StackRow::file_row` sets the same field for a
        // File row) — a heading that only carried the nested copy read as
        // never current to it. `plate_rects` deliberately does NOT read this
        // field for a Group row: the plate is always the active file's,
        // never the project's own heading.
        active,
        kind: StackRowKind::Group { active },
    }
}

/// **A GROUP HEADING'S OWN QUIET PARENT** — a disambiguating segment
/// (`"research/"` vs `"archive/"`) for a root whose LEAF name collides with
/// another root drawn in the same panel (two different projects both named
/// `notes`), `None` when no other drawn root shares this leaf.
///
/// Routed through [`super::quiet_parent::common_ancestor`] + the SAME
/// strip-then-format primitive [`OpenFile::parent_label`] uses
/// ([`super::quiet_parent::quiet_relative_label`]) rather than a second
/// parent-eliding mechanism: the base here is the deepest ancestor `root`
/// shares with EVERY other same-leaf root (folded pairwise), one level up
/// from a file's own root-relative parent, but the same "strip, drop if
/// empty, add `/`" rule.
///
/// A root that sits DIRECTLY under that shared ancestor (e.g. `~/notes`
/// beside `~/archive/notes`) strips to nothing and draws no quiet label of
/// its own — still distinguishable, because its same-leaf rival gets one.
pub(super) fn group_parent_label(root: &Path, all_roots: &[&Path]) -> Option<String> {
    let leaf = crate::project::folder_name(root);
    let mut colliding = all_roots
        .iter()
        .copied()
        .filter(|r| crate::project::folder_name(r) == leaf);
    let first = colliding.next()?;
    let base = colliding.fold(first.to_path_buf(), |acc, r| {
        super::quiet_parent::common_ancestor(&acc, r)
    });
    // `root`'s own PARENT against the shared base — one level up from a
    // file's own `path.parent()` against its root, the same rule
    // `quiet_relative_label` already applies. A leaf with no collision
    // (`colliding` held only `root` itself) folds `base` to `root`, and
    // `root.parent()` is never `root` itself, so this falls through to
    // `None` the same way an empty relative path would.
    super::quiet_parent::quiet_relative_label(root.parent()?, &base)
}

#[cfg_attr(not(test), allow(dead_code))]
impl WorkingSet {
    /// Is the reader looking at the resting stack, or the panel it expands
    /// into?
    pub fn is_expanded(&self) -> bool {
        matches!(self.panel, Panel::Expanded { .. })
    }

    /// OPEN THE EXPANDED PANEL, scrolled so the active row is visible (the
    /// browser-tab reveal-on-open convention this surface's brief names) — a
    /// no-op below two open files, since there is then no overflow row that
    /// could have summoned it.
    pub fn expand(&mut self) {
        if self.len() < 2 {
            return;
        }
        self.panel = Panel::Expanded {
            scroll: self.expanded_reveal_scroll(),
        };
    }

    /// Return to the resting stack. Idempotent.
    pub fn collapse(&mut self) {
        self.panel = Panel::Resting;
    }

    /// SCROLL THE EXPANDED PANEL by `delta` rows (negative toward the top),
    /// clamped to `[0, max]` — the panel's own bounds, never toward the active
    /// row. A no-op while the panel is not open. This is the ONE door a
    /// reader's own wheel/trackpad motion moves the panel through, and it never
    /// re-centres on the active file the way [`Self::on_active_changed`]'s
    /// reveal does — the two clauses in this surface's brief ("opens scrolled
    /// so the active row is visible" / "a user's own scroll is never fought")
    /// are not in tension: the first is a bound, the second is that this
    /// function never moves `scroll` toward anything but where the caller
    /// asked.
    pub fn scroll_expanded(&mut self, delta: isize) {
        let Panel::Expanded { scroll } = self.panel else {
            return;
        };
        let max = self.expanded_max_scroll(&self.expanded_full()) as isize;
        let next = (scroll as isize + delta).clamp(0, max.max(0)) as usize;
        self.panel = Panel::Expanded { scroll: next };
    }

    /// THE EXPANDED PANEL'S FULL, UNWINDOWED CONTENT: every open file, headed
    /// by its root, roots in FIRST-SEEN order (the same order the judged
    /// gallery's `Grouped` prototype drew them in — `grouped-saltpan.png`
    /// heads `notebook` before `atlas` because `notebook`'s files were opened
    /// first).
    pub(super) fn expanded_full(&self) -> Vec<PanelRow> {
        let mut roots: Vec<&Path> = Vec::new();
        for f in &self.files {
            if !roots.contains(&f.root.as_path()) {
                roots.push(f.root.as_path());
            }
        }
        let active_root = self.active_root();
        let mut rows = Vec::with_capacity(self.files.len() + roots.len());
        for root in roots {
            rows.push(PanelRow::Group(
                root.to_path_buf(),
                Some(root) == active_root,
            ));
            for at in self.group(root) {
                rows.push(PanelRow::File(at));
            }
        }
        rows
    }

    /// The minimal-jump scroll that brings the active row into the DRAWN
    /// window — found directly rather than in closed form, the same reason
    /// [`Self::expanded_max_scroll`] is: a pinned sticky heading or an
    /// overflow cue can make the window narrower than [`EXPANDED_VIEWPORT`]
    /// depending on `scroll` itself, so "smallest `s` whose own window
    /// contains the active row" is not a fixed offset once that varies. `0`
    /// when nothing is active (there is no row to reveal).
    pub(super) fn expanded_reveal_scroll(&self) -> usize {
        let full = self.expanded_full();
        let Some(active) = self.active else {
            return 0;
        };
        if !full
            .iter()
            .any(|row| matches!(row, PanelRow::File(at) if *at == active))
        {
            return 0;
        }
        let max_scroll = self.expanded_max_scroll(&full);
        (0..=max_scroll)
            .find(|&s| {
                self.expanded_window_at(&full, s).iter().any(
                    |row| matches!(row, DrawnRow::Full(_, PanelRow::File(at)) if *at == active),
                )
            })
            .unwrap_or(max_scroll)
    }

    /// THE LARGEST SCROLL WORTH HOLDING. A plain `full.len() - EXPANDED_VIEWPORT`
    /// bound is correct only for a constant-size window; this panel's own
    /// windows can show FEWER files than that whenever a sticky heading or an
    /// overflow cue is pinned, which can strand a group's own last file just
    /// past the old bound (measured: ten files under one root,
    /// `EXPANDED_VIEWPORT = 8` — the old bound clamped scroll at 3, and the
    /// sticky-widened window at scroll 3 showed files 2..8, never reaching
    /// file 9). Found directly rather than in closed form: the smallest
    /// scroll whose own window already reaches the end — no `↓ N more` cue —
    /// since scrolling further reveals nothing new.
    fn expanded_max_scroll(&self, full: &[PanelRow]) -> usize {
        if full.len() <= EXPANDED_VIEWPORT {
            return 0;
        }
        (0..full.len())
            .find(|&s| {
                !self
                    .expanded_window_at(full, s)
                    .iter()
                    .any(|row| matches!(row, DrawnRow::Overflow { up: false, .. }))
            })
            .unwrap_or_else(|| full.len().saturating_sub(1))
    }

    /// THE PANEL'S DRAWN WINDOW at the CURRENT scroll, clamped to
    /// [`Self::expanded_max_scroll`]. Empty while the panel is not open.
    pub(super) fn expanded_window(&self) -> Vec<DrawnRow> {
        let Panel::Expanded { scroll } = self.panel else {
            return Vec::new();
        };
        let full = self.expanded_full();
        if full.is_empty() {
            return Vec::new();
        }
        let scroll = scroll.min(self.expanded_max_scroll(&full));
        self.expanded_window_at(&full, scroll)
    }

    /// THE PANEL'S DRAWN WINDOW at an ARBITRARY `scroll` — the one builder
    /// [`Self::expanded_window`] (drawing) and [`Self::expanded_max_scroll`]/
    /// [`Self::expanded_reveal_scroll`] (searching over candidate scrolls)
    /// all read, so a window built for a SEARCH candidate and the window
    /// actually drawn can never disagree about their own shape.
    ///
    /// Reserves, in order, the `↑ N more` cue (whenever any FILE precedes
    /// `scroll`) and the pinned sticky heading (whenever the first content
    /// row would otherwise be a bare File) — the overflow line ahead of the
    /// heading, so the two affordances never fight for one slot — then fills
    /// the rest with content from `full[scroll..]`, reserving one more slot
    /// at the end for the `↓ N more` cue if any FILE would still be left
    /// unshown. Every reservation costs a real viewport slot: the drawn total
    /// never exceeds [`EXPANDED_VIEWPORT`].
    fn expanded_window_at(&self, full: &[PanelRow], scroll: usize) -> Vec<DrawnRow> {
        if full.is_empty() {
            return Vec::new();
        }
        let n_items = self.files.len();
        // The FILE-ordinal position of the window's first shown item — the
        // `top` [`crate::render::chrome::window_edge_counts`] wants, never a
        // display-row count: a project heading is not an item.
        let top_files = full[..scroll]
            .iter()
            .filter(|r| matches!(r, PanelRow::File(_)))
            .count();
        let sticky = expanded_sticky_origin(self, full, scroll);
        let reserved_top = usize::from(top_files > 0) + usize::from(sticky.is_some());
        let content_budget = EXPANDED_VIEWPORT.saturating_sub(reserved_top);

        // How many ROWS of `full[scroll..]` fit in `budget`, and how many of
        // those rows are FILES — the `visible` item count
        // [`crate::render::chrome::window_edge_counts`] wants.
        let content_at = |budget: usize| -> (usize, usize) {
            let mut rows = 0usize;
            let mut files = 0usize;
            for row in &full[scroll..] {
                if rows >= budget {
                    break;
                }
                rows += 1;
                if matches!(row, PanelRow::File(_)) {
                    files += 1;
                }
            }
            (rows, files)
        };
        let (mut rows_shown, mut files_shown) = content_at(content_budget);
        let (top, below) =
            crate::render::chrome::window_edge_counts(top_files, files_shown, n_items);
        if below.is_some() {
            (rows_shown, files_shown) = content_at(content_budget.saturating_sub(1));
        }
        // Re-asked after the possible budget cut above: cutting one more
        // content row can only ever ADD to what is hidden below, never
        // remove the need for the cue it just made room for.
        let below = if below.is_some() {
            crate::render::chrome::window_edge_counts(top_files, files_shown, n_items).1
        } else {
            below
        };

        let mut window = Vec::with_capacity(EXPANDED_VIEWPORT);
        if let Some(hidden) = top {
            window.push(DrawnRow::Overflow { up: true, hidden });
        }
        if let Some(origin) = sticky {
            let PanelRow::Group(root, active) = &full[origin] else {
                unreachable!("expanded_sticky_origin only ever names a Group row");
            };
            window.push(DrawnRow::Sticky(origin, root.clone(), *active));
        }
        window.extend(
            full[scroll..(scroll + rows_shown).min(full.len())]
                .iter()
                .enumerate()
                .map(|(i, row)| DrawnRow::Full(scroll + i, row.clone())),
        );
        if let Some(hidden) = below {
            window.push(DrawnRow::Overflow { up: false, hidden });
        }
        window
    }

    /// THE EXPANDED PANEL'S DRAWN ROWS, converted from [`Self::expanded_window`]
    /// — [`group_stack_row`] renders a natural heading and its pinned sticky
    /// duplicate identically, since both name the same real project. Empty
    /// while the panel is not open.
    pub fn expanded_rows(&self) -> Vec<StackRow> {
        let window = self.expanded_window();
        if window.is_empty() {
            return Vec::new();
        }
        let full = self.expanded_full();
        let roots: Vec<&Path> = full
            .iter()
            .filter_map(|row| match row {
                PanelRow::Group(root, _) => Some(root.as_path()),
                PanelRow::File(_) => None,
            })
            .collect();
        window
            .iter()
            .map(|row| match row {
                DrawnRow::Full(_, PanelRow::Group(root, active)) => {
                    group_stack_row(root, *active, &roots)
                }
                DrawnRow::Sticky(_, root, active) => group_stack_row(root, *active, &roots),
                DrawnRow::Full(_, PanelRow::File(at)) => self.file_row(*at),
                DrawnRow::Overflow { up, hidden } => StackRow {
                    leaf: crate::render::chrome::edge_cue_text(*up, *hidden),
                    parent: String::new(),
                    active: false,
                    kind: StackRowKind::Overflow {
                        up: *up,
                        hidden: *hidden,
                    },
                },
            })
            .collect()
    }

    /// THE FILE a drawn EXPANDED-PANEL row names, or `None` for a heading row
    /// or a row past the panel's own drawn window. The click-resolution
    /// counterpart to [`Self::expanded_rows`] — resolved through the exact
    /// same [`Self::expanded_window`], so a click can never name a file a
    /// different row than the one drawn under the pointer.
    pub fn expanded_row_open_file(&self, row: usize) -> Option<&OpenFile> {
        match self.expanded_window().get(row)? {
            DrawnRow::Full(_, PanelRow::File(at)) => self.files.get(*at),
            DrawnRow::Full(_, PanelRow::Group(..))
            | DrawnRow::Sticky(..)
            | DrawnRow::Overflow { .. } => None,
        }
    }
}

/// Whether the window opening at `scroll` needs the pinned sticky heading —
/// true exactly when the first row it would otherwise draw is a bare File,
/// its own group's heading having already scrolled off above.
fn expanded_sticky_needed(full: &[PanelRow], scroll: usize) -> bool {
    matches!(full.get(scroll), Some(PanelRow::File(_)))
}

/// The pinned heading's OWN `expanded_full` index, when [`expanded_sticky_needed`]
/// says one is needed — always found, since `expanded_full` places every
/// group's heading immediately before its own files (`expanded_full`'s doc),
/// so scanning backward from `scroll` for `File(at)`'s own root always lands
/// on a real `Group` entry.
fn expanded_sticky_origin(ws: &WorkingSet, full: &[PanelRow], scroll: usize) -> Option<usize> {
    let PanelRow::File(at) = full.get(scroll)? else {
        return None;
    };
    if !expanded_sticky_needed(full, scroll) {
        return None;
    }
    let root = ws.files[*at].root.clone();
    full[..scroll]
        .iter()
        .rposition(|r| matches!(r, PanelRow::Group(r2, _) if *r2 == root))
}
