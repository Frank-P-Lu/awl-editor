//! src/workingset.rs — the VISIBLE working set: which files are open, in what
//! order the margin draws them, and which project root each one belongs to.
//!
//! This is deliberately NOT [`crate::buffers::BufferRegistry`]. That registry is
//! MRU-ordered because its job is eviction — index 0 is the last-resort victim,
//! and a dirty buffer is never discarded. Ordering a *visible* surface that way
//! would rearrange the margin on every switch, under a pointer that is reaching
//! for a row. So the two orders are different questions with different owners:
//! the registry answers "what may I drop under memory pressure", this answers
//! "what does the reader see, where, and does it stay there".
//!
//! Three properties the rest of the app depends on:
//!
//! * **Stable open order.** A file takes a slot when it is first opened and
//!   keeps it until it is closed. Re-activating an already-open file changes
//!   only [`WorkingSet::active`], never the order.
//! * **Every open file remembers its own root.** The active project root owns Go
//!   to's corpus, New document, Move and export destinations and the bottom
//!   identity's folder line — and a buffer opened under one root can still be
//!   activated while another is current. Without a remembered root per buffer,
//!   that transition leaves the document and the folder identity describing two
//!   different places, with nothing able to tell which is wrong.
//! * **A row reads by its ROOT-RELATIVE path, not its leaf.** Two files named
//!   `notes.md` under different subfolders are not the same row and must not
//!   read as though they were.

use std::path::{Path, PathBuf};

use crate::buffers::BufferKey;

mod prototype;
pub use prototype::{PrototypeReport, PrototypeSpec, prototype_move_from_env, prototype_move_rows};

mod reorder;

/// One member of the visible working set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenFile {
    /// The registry identity — the same key `BufferRegistry` parks under, so a
    /// row and a parked buffer can never drift apart.
    pub key: BufferKey,
    /// The file's own path. `None` for the path-less scratch surface, which has
    /// a slot (it is open) but no location to name.
    pub path: Option<PathBuf>,
    /// The project root this buffer was opened under. Restored as the active
    /// root whenever this file is activated — see the module doc.
    pub root: PathBuf,
}

/// The label and removal halves of the model are law-tested here and consumed
/// by the margin stack itself, which lands separately — the design decision that
/// owns this surface asks for the resting stack to be judged from captures
/// before its drawing and pointer machinery is wired. Each allow is scoped to
/// one item rather than the module, so an unused method anywhere else in the
/// crate still fails the build, and they come off with the first consumer.
#[cfg_attr(not(test), allow(dead_code))]
impl OpenFile {
    /// The leaf the row draws in normal ink: the file name, or `"scratch"` for
    /// the path-less surface.
    pub fn leaf(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "scratch".to_string())
    }

    /// The row's QUIETER half: the file's parent, relative to its own root, with
    /// a trailing separator (`"journal/"`). `None` when the file sits directly
    /// under the root — there is no location to add, and drawing an empty span
    /// would reserve width for nothing.
    ///
    /// Deliberately relative to `self.root` rather than to whatever root happens
    /// to be active: a row in another project's group still describes where that
    /// file actually lives.
    pub fn parent_label(&self) -> Option<String> {
        let path = self.path.as_deref()?;
        let rel = path.strip_prefix(&self.root).ok()?;
        let parent = rel.parent()?;
        if parent.as_os_str().is_empty() {
            return None;
        }
        let mut s = parent.to_string_lossy().replace('\\', "/");
        s.push('/');
        Some(s)
    }
}

/// Elide a [`OpenFile::parent_label`] to `budget` characters, keeping the
/// NEAREST-TO-ROOT segment and dropping the middle: `research/sources/drafts/`
/// becomes `research/…/`.
///
/// Which end survives is the whole decision. Truncating the tail
/// (`research/sou…`) keeps characters and destroys the fact — the reader learns
/// the file is somewhere under `research`, which the first segment already said,
/// and loses that anything was elided at all. Keeping the first segment plus an
/// explicit ellipsis says both that there is more depth and where the branch
/// started, in fewer characters than the truncation it replaces.
///
/// Returns `None` when even `a/…/` cannot fit, so the caller draws no location
/// rather than a misleading fragment of one.
#[cfg_attr(not(test), allow(dead_code))]
pub fn fit_parent(label: &str, budget: usize) -> Option<String> {
    if label.chars().count() <= budget {
        return Some(label.to_string());
    }
    let first = label.split('/').next().unwrap_or_default();
    if first.is_empty() {
        return None;
    }
    let elided = format!("{first}/…/");
    if elided.chars().count() <= budget {
        Some(elided)
    } else {
        None
    }
}

/// WHICH ROOT A FILE BELONGS TO, decided once per open.
///
/// The naive answer — "whatever root is active when it opens" — is wrong in the
/// exact case this whole mechanism exists for. Re-activating a file that lives
/// under another root would re-stamp it with the CURRENT root, so the memory
/// needed to restore its project is destroyed by the very transition that needs
/// it, and the second switch back reads as correct while naming the wrong
/// folder.
///
/// So the rule is about the file, not the moment: a file keeps the root it was
/// opened under for as long as it still lives beneath it, falls back to the
/// active root when that root contains it, and otherwise stands on its own
/// parent directory rather than borrowing a root it is not inside.
pub fn root_for(path: &Path, active_root: &Path, remembered: Option<&Path>) -> PathBuf {
    if let Some(r) = remembered
        && path.starts_with(r)
    {
        return r.to_path_buf();
    }
    if path.starts_with(active_root) {
        return active_root.to_path_buf();
    }
    path.parent().unwrap_or(path).to_path_buf()
}

/// ONE DRAWN ROW of the margin's resting stack, already reduced to the two
/// pieces of text a row shows and which one of them is the reader's current
/// file. Deliberately a projection rather than a borrow of [`OpenFile`]: the
/// renderer never asks the working set a question mid-frame, so a row cannot
/// answer one thing to the draw and another to the hit-test.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StackRowKind {
    /// A real open file. This is the only row kind that may show the close mark
    /// or carry the active-file plate.
    #[default]
    File,
    /// The collapsed view's single generic overflow affordance.
    More { hidden: usize },
    /// A project heading in the expanded cross-project prototype.
    Group { active: bool },
}

/// One projected row in the margin stack.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct StackRow {
    /// The file name, in the row's normal ink.
    pub leaf: String,
    /// The root-relative parent with its trailing separator (`"journal/"`), in
    /// quieter ink. Empty when the file sits directly under the root.
    pub parent: String,
    /// Is this the file the reader is currently editing?
    pub active: bool,
    /// Whether this row is a file, the one overflow affordance, or a project
    /// heading. [`WorkingSet::stack_rows`] (the resting stack) emits `File` and,
    /// once the active root's group overflows [`RESTING_FILES`], one trailing
    /// `More`; [`WorkingSet::expanded_rows`] (the transient scrollable panel)
    /// emits `File` and `Group` heading rows.
    pub kind: StackRowKind,
    /// A sealed capture-prototype hover pose. The live app's real pointer still
    /// rides the renderer's `gutter_stack_hover`; this bit exists only so a
    /// windowless `--screenshot-app` run can photograph the already-shipped
    /// close mark without fabricating pointer input.
    pub prototype_hovered: bool,
}

/// THE RESTING STACK'S OWN ROW CAP — the number of FILE rows the collapsed
/// margin draws before folding the rest behind one `+ N more…` row. Fixed at
/// the number the user judged in the item 444 residual-3 gallery
/// (`captures/item-444-residual3/README.md`); [`prototype::PrototypeSpec`]'s
/// own candidate reuses this constant rather than a second literal, so the
/// judged number and the shipped one cannot drift apart.
pub const RESTING_FILES: usize = 5;

/// THE EXPANDED/GROUPED PANEL'S OWN SCROLLABLE VIEWPORT, in total drawn rows
/// (file rows and folder headings together) — the number judged in the same
/// gallery pass.
pub const EXPANDED_VIEWPORT: usize = 8;

/// The margin's OWN transient UI state: is the reader looking at the resting
/// five-row stack, or the panel it expands into? Lives beside the order and
/// root state it presents rather than on `App` (which has a hard field
/// ceiling, `app/tests/domains.rs::root_app_does_not_grow`) — and the module's
/// own contract, "what does the reader see, where, and does it stay there",
/// already covers a transient viewport as much as it covers the stable order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum Panel {
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
enum PanelRow {
    Group(PathBuf, bool),
    File(usize),
}

/// The open files, in the order the margin draws them, plus which one is active.
///
/// Empty after the last document closes; that is the zero-document state's own
/// representation rather than a fake unnamed buffer.
#[derive(Clone, Debug, Default)]
pub struct WorkingSet {
    files: Vec<OpenFile>,
    active: Option<usize>,
    /// The RESTING stack's own hold-still window: the root it was last
    /// computed for, and the first visible slot within that root's group.
    /// `None` until the first activation. See [`Self::recompute_resting_window`].
    resting_window: Option<(PathBuf, usize)>,
    panel: Panel,
}

#[cfg_attr(not(test), allow(dead_code))]
impl WorkingSet {
    /// Open `key` under `root` (or re-activate it if already open) and make it
    /// active. Returns its slot.
    ///
    /// An already-open file KEEPS ITS SLOT — the whole point of the stable
    /// order. A re-open under a different root updates the remembered root
    /// without moving the row, because the file's location is what changed, not
    /// the reader's map of the margin.
    pub fn open(&mut self, key: BufferKey, path: Option<PathBuf>, root: PathBuf) -> usize {
        let at = match self.files.iter().position(|f| f.key == key) {
            Some(at) => {
                self.files[at].root = root;
                self.files[at].path = path;
                at
            }
            None => {
                self.files.push(OpenFile { key, path, root });
                self.files.len() - 1
            }
        };
        self.active = Some(at);
        self.on_active_changed();
        at
    }

    /// Remove the file at `at`, returning it. The active slot follows the
    /// surviving neighbour rather than resetting to zero: closing row 3 of 5
    /// should leave the reader looking at row 3's replacement, not jump the
    /// margin back to the top.
    pub fn close(&mut self, at: usize) -> Option<OpenFile> {
        if at >= self.files.len() {
            return None;
        }
        let gone = self.files.remove(at);
        self.active = match self.active {
            _ if self.files.is_empty() => None,
            Some(a) if a > at => Some(a - 1),
            Some(a) if a == at => Some(a.min(self.files.len() - 1)),
            other => other,
        };
        self.on_active_changed();
        Some(gone)
    }

    /// Remove the file carrying `key`, wherever it sits. The pointer route:
    /// closing an inactive row closes THAT named buffer without first
    /// activating it.
    pub fn close_key(&mut self, key: &BufferKey) -> Option<OpenFile> {
        let at = self.files.iter().position(|f| &f.key == key)?;
        self.close(at)
    }

    /// Make the file at `at` active without disturbing the order. `false` if the
    /// slot does not exist.
    pub fn set_active(&mut self, at: usize) -> bool {
        if at >= self.files.len() {
            return false;
        }
        self.active = Some(at);
        self.on_active_changed();
        true
    }

    /// Recompute everything that depends on WHICH slot is active, after any
    /// mutation that can change it (open/re-activate, switch, close). Two
    /// independent recomputations, each a no-op when it does not apply:
    ///
    /// * The resting stack's hold-still window ([`Self::recompute_resting_window`]).
    /// * The expanded panel's reveal — "any activation re-reveals it"
    ///   (`CLAUDE.md`'s brief for this surface): if the panel is open when the
    ///   active slot changes, its scroll re-centres on the new active row
    ///   through the SAME minimal-jump formula opening it uses
    ///   ([`Self::expanded_reveal_scroll`]), rather than staying wherever a
    ///   PREVIOUS reader's scroll left it. A working set that drops below two
    ///   files closes the panel outright — there is nothing left to expand.
    fn on_active_changed(&mut self) {
        self.recompute_resting_window();
        if self.len() < 2 {
            self.panel = Panel::Resting;
        } else if matches!(self.panel, Panel::Expanded { .. }) {
            self.panel = Panel::Expanded {
                scroll: self.expanded_reveal_scroll(),
            };
        }
    }

    /// THE HOLD-STILL / MINIMAL-SLIDE LAW.
    ///
    /// The gallery's rejected candidate (`prototype::PrototypeSpec::Collapsed`,
    /// unchanged — it stays on the record as the law's own red-arm reference)
    /// re-derives the resting window from nothing but the active file's index
    /// EVERY time, which is what let an already-visible row jump across the
    /// window on the very next activation (`collapsed-jitter.png`). This is
    /// STATEFUL instead: the window remembered here only MOVES when the newly
    /// active file has left it, and then by the minimum distance that brings
    /// it back — never re-centring on a file the reader was already looking
    /// at.
    ///
    /// A window computed for a DIFFERENT root (or none yet) falls back to the
    /// same fresh reveal the rejected candidate used — there is no PREVIOUS
    /// window to hold still against the first time a root is visited.
    fn recompute_resting_window(&mut self) {
        let Some(active) = self.active else {
            self.resting_window = None;
            return;
        };
        let root = self.files[active].root.clone();
        let group = self.group(&root);
        let Some(active_in_group) = group.iter().position(|&at| at == active) else {
            self.resting_window = None;
            return;
        };
        let max_start = group.len().saturating_sub(RESTING_FILES);
        let start = match &self.resting_window {
            Some((prev_root, prev_start)) if *prev_root == root => {
                let prev_start = (*prev_start).min(max_start);
                if active_in_group >= prev_start && active_in_group < prev_start + RESTING_FILES {
                    // HOLD STILL: the newly active file is already inside the
                    // drawn window, so nothing about it moves.
                    prev_start
                } else if active_in_group < prev_start {
                    // SLIDE UP by exactly enough to reveal it at the window's
                    // own top edge — never further.
                    active_in_group
                } else {
                    // SLIDE DOWN by exactly enough to reveal it at the
                    // window's own bottom edge.
                    (active_in_group + 1).saturating_sub(RESTING_FILES)
                }
            }
            _ => active_in_group
                .saturating_sub(RESTING_FILES.saturating_sub(1))
                .min(max_start),
        };
        self.resting_window = Some((root, start.min(max_start)));
    }

    pub fn files(&self) -> &[OpenFile] {
        &self.files
    }

    pub fn path_for(&self, key: &BufferKey) -> Option<&Path> {
        self.files
            .iter()
            .find(|file| &file.key == key)
            .and_then(|file| file.path.as_deref())
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    pub fn active_file(&self) -> Option<&OpenFile> {
        self.files.get(self.active?)
    }

    /// The root the active file remembers — the one a switch must restore, and
    /// the one the bottom identity's folder line should be naming.
    pub fn active_root(&self) -> Option<&Path> {
        Some(self.active_file()?.root.as_path())
    }

    pub fn index_of(&self, key: &BufferKey) -> Option<usize> {
        self.files.iter().position(|f| &f.key == key)
    }

    /// THE ROWS THE MARGIN DRAWS for `root`'s group — and **empty whenever that
    /// group holds fewer than two files.**
    ///
    /// That emptiness is the whole one-file contract, and it lives HERE so it
    /// lives exactly once. The bottom identity widens into a stack only when
    /// there is a working set to show; with a single file open the renderer is
    /// handed nothing, takes the same path it took before this surface existed,
    /// and draws the same bytes. A second `len() <= 1` guard further down would
    /// be a second place for that rule to be true, and the day they disagree the
    /// margin grows a row for a set of one.
    ///
    /// The count that GATES the stack is the ACTIVE ROOT'S GROUP, never
    /// [`Self::len`]: a file parked under another project must not summon a
    /// stack in this one. Once gated, the window shown is bounded to
    /// [`RESTING_FILES`] rows through [`Self::recompute_resting_window`]'s
    /// hold-still/minimal-slide state, with one trailing `+ N more…` row
    /// whenever anything is hidden — counting every open buffer this window
    /// does not draw, in this root's own overflow AND every other root alike,
    /// since that row is the margin's one door to all of them (the panel this
    /// overflow row expands: [`Self::expanded_rows`]).
    /// THE RESTING STACK'S OWN WINDOW START for `root`'s group, in
    /// group-relative units — the ONE computation [`Self::stack_rows`] draws
    /// from and [`Self::resting_row_index`] resolves a pointer against, so a
    /// click, a drag, and the drawn window can never disagree about which
    /// group slot row 0 names. Mirrors [`Self::recompute_resting_window`]'s own
    /// fallback formula (a window computed for a DIFFERENT root, or none yet,
    /// falls back to the same fresh reveal that method's own last arm uses),
    /// but never WRITES `resting_window` — this is the read-only half, asked
    /// by any caller that needs the number without mutating state.
    fn resting_start(&self, root: &Path, group: &[usize]) -> usize {
        let max_start = group.len().saturating_sub(RESTING_FILES);
        match &self.resting_window {
            Some((r, s)) if r.as_path() == root => (*s).min(max_start),
            _ => {
                let active_in_group = self
                    .active_index()
                    .and_then(|active| group.iter().position(|&at| at == active));
                active_in_group
                    .map(|a| {
                        a.saturating_sub(RESTING_FILES.saturating_sub(1))
                            .min(max_start)
                    })
                    .unwrap_or(0)
            }
        }
    }

    pub fn stack_rows(&self, root: &Path) -> Vec<StackRow> {
        let group = self.group(root);
        if group.len() < 2 {
            return Vec::new();
        }
        let start = self.resting_start(root, &group);
        let visible = &group[start..(start + RESTING_FILES).min(group.len())];
        let mut rows: Vec<StackRow> = visible.iter().map(|&at| self.file_row(at)).collect();
        let hidden = self.len().saturating_sub(visible.len());
        if hidden > 0 {
            rows.push(StackRow {
                leaf: format!("+ {hidden} more…"),
                kind: StackRowKind::More { hidden },
                ..StackRow::default()
            });
        }
        rows
    }

    /// The one row projection both [`Self::stack_rows`] and
    /// [`Self::expanded_rows`] build a `File` row from, so the two views cannot
    /// describe the same open file differently.
    fn file_row(&self, at: usize) -> StackRow {
        StackRow {
            leaf: self.files[at].leaf(),
            parent: self.files[at].parent_label().unwrap_or_default(),
            active: self.active == Some(at),
            kind: StackRowKind::File,
            prototype_hovered: false,
        }
    }

    /// The slots whose files belong to `root` — the resting stack's own group.
    /// Order is preserved, so a group is a filtered view of the stable order
    /// rather than a second ordering to keep in sync.
    pub fn group(&self, root: &Path) -> Vec<usize> {
        self.files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.root == root)
            .map(|(i, _)| i)
            .collect()
    }

    /// THE MARGIN'S ONE ROW SOURCE: the resting stack, or the expanded panel it
    /// opens into, whichever the reader is currently looking at. Both live
    /// call sites that draw the margin (`app/viewstate.rs`'s `sync_view`,
    /// `app/capture_state.rs`'s live-App capture fold) route through this ONE
    /// owner instead of each asking [`Self::is_expanded`] on its own — the
    /// shape "two call sites, one condition" is exactly how a live frame and
    /// its own capture drift apart the day only one of them is edited.
    pub fn margin_rows(&self, root: &Path) -> Vec<StackRow> {
        if self.is_expanded() {
            self.expanded_rows()
        } else {
            self.stack_rows(root)
        }
    }

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
    fn expanded_full(&self) -> Vec<PanelRow> {
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
    fn expanded_reveal_scroll(&self) -> usize {
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
        full[scroll..(scroll + EXPANDED_VIEWPORT).min(full.len())]
            .iter()
            .map(|row| match row {
                PanelRow::Group(root, active) => StackRow {
                    leaf: crate::project::folder_name(root),
                    kind: StackRowKind::Group { active: *active },
                    ..StackRow::default()
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

#[cfg(test)]
mod tests;
