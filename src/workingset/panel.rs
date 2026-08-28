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
        let max = self.expanded_full().len().saturating_sub(EXPANDED_VIEWPORT) as isize;
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

    /// The minimal-jump scroll that brings the active row into
    /// [`EXPANDED_VIEWPORT`] rows of the panel — mirrors the resting window's
    /// own fresh-reveal formula (`stack_rows`' fallback arm), over the FULL
    /// grouped list rather than one root's group. `0` when nothing is active
    /// (there is no row to reveal).
    pub(super) fn expanded_reveal_scroll(&self) -> usize {
        let full = self.expanded_full();
        let max_scroll = full.len().saturating_sub(EXPANDED_VIEWPORT);
        let active_row = self.active.and_then(|active| {
            full.iter()
                .position(|row| matches!(row, PanelRow::File(at) if *at == active))
        });
        active_row
            .map(|a| {
                a.saturating_sub(EXPANDED_VIEWPORT.saturating_sub(1))
                    .min(max_scroll)
            })
            .unwrap_or(0)
    }

    /// THE EXPANDED PANEL'S DRAWN ROWS: [`EXPANDED_VIEWPORT`] rows of
    /// [`Self::expanded_full`] starting at the current scroll, clamped to
    /// bounds at READ time (never trusting a stored `scroll` that a close
    /// since made too large — [`Self::on_active_changed`] already recomputes
    /// it on most closes, but this stays correct even if that changes). Empty
    /// while the panel is not open.
    pub fn expanded_rows(&self) -> Vec<StackRow> {
        let Panel::Expanded { scroll } = self.panel else {
            return Vec::new();
        };
        let full = self.expanded_full();
        let max_scroll = full.len().saturating_sub(EXPANDED_VIEWPORT);
        let scroll = scroll.min(max_scroll);
        let roots: Vec<&Path> = full
            .iter()
            .filter_map(|row| match row {
                PanelRow::Group(root, _) => Some(root.as_path()),
                PanelRow::File(_) => None,
            })
            .collect();
        full[scroll..(scroll + EXPANDED_VIEWPORT).min(full.len())]
            .iter()
            .map(|row| match row {
                PanelRow::Group(root, active) => StackRow {
                    // Trailing `/` for FOLDER identity — the same rule
                    // `row_display` already applies to a picker's own folder
                    // rows (`row.is_dir` / `RowMeta::GotoFolder`); a project
                    // heading is a folder every time, so it always carries
                    // one rather than reading as a bare (file-like) name.
                    leaf: format!("{}/", crate::project::folder_name(root)),
                    parent: group_parent_label(root, &roots).unwrap_or_default(),
                    // The OUTER field, not just the kind's own copy: this is
                    // the one `active: bool` marker `plate_rects` and every
                    // reader outside `stack_spans`' ink match already reads
                    // for "is this the current row" (`StackRow::file_row`
                    // sets the same field for a File row) — a heading that
                    // only carried the nested copy read as never current to
                    // any of them.
                    active: *active,
                    kind: StackRowKind::Group { active: *active },
                },
                PanelRow::File(at) => self.file_row(*at),
            })
            .collect()
    }

    /// THE FILE a drawn EXPANDED-PANEL row names, or `None` for a heading row
    /// or a row past the panel's own drawn window. The click-resolution
    /// counterpart to [`Self::expanded_rows`] — resolved through the exact
    /// same windowed slice, so a click can never name a file a different row
    /// than the one drawn under the pointer.
    pub fn expanded_row_open_file(&self, row: usize) -> Option<&OpenFile> {
        let Panel::Expanded { scroll } = self.panel else {
            return None;
        };
        let full = self.expanded_full();
        let max_scroll = full.len().saturating_sub(EXPANDED_VIEWPORT);
        let scroll = scroll.min(max_scroll);
        match full.get(scroll + row)? {
            PanelRow::File(at) => self.files.get(*at),
            PanelRow::Group(..) => None,
        }
    }
}
