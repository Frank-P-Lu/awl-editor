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

mod direct;
mod panel;
mod prototype;
pub use prototype::{prototype_move_from_env, prototype_move_rows};

mod quiet_parent;
mod reorder;

mod stackrow;
pub use stackrow::{StackRow, StackRowKind};

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
        match (&self.path, &self.key) {
            (Some(path), _) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "scratch".to_string()),
            (None, BufferKey::Fresh(_)) => "untitled".to_string(),
            (None, BufferKey::Scratch | BufferKey::Path(_)) => "scratch".to_string(),
        }
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
        quiet_parent::quiet_relative_label(path.parent()?, &self.root)
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
///
/// `path` is canonicalized before every `starts_with` comparison, through the
/// SAME [`crate::buffers::normalize_path`] the root-identity owners already
/// route `active_root` through — otherwise it disagrees in spelling with an
/// alias-resolved root (macOS's `/var` -> `/private/var`) and falls through
/// to "orphan", a real regression. A fictional path round-trips unchanged.
pub fn root_for(path: &Path, active_root: &Path, remembered: Option<&Path>) -> PathBuf {
    let path = crate::buffers::normalize_path(path);
    if let Some(r) = remembered
        && path.starts_with(r)
    {
        return r.to_path_buf();
    }
    if path.starts_with(active_root) {
        return active_root.to_path_buf();
    }
    path.parent().unwrap_or(&path).to_path_buf()
}

/// The compact margin's maximum number of simultaneously drawn file rows.
pub const RESTING_FILES: usize = 5;

/// The open files, in the order the margin draws them, plus which one is active.
///
/// Empty after the last document closes; that is the zero-document state's own
/// representation rather than a fake unnamed buffer.
#[derive(Clone, Debug, Default)]
pub struct WorkingSet {
    files: Vec<OpenFile>,
    active: Option<usize>,
    /// First file in the compact, directly scrollable margin window.
    direct_scroll: usize,
    direct_projection: direct::Projection,
    panel: panel::Panel,
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

    /// An explicit folder choice adopts open files beneath it. Files outside
    /// that scope keep their own root, order, active slot, and buffer identity.
    pub fn rescope(&mut self, root: &Path) {
        for file in &mut self.files {
            if file
                .path
                .as_deref()
                .is_some_and(|path| crate::buffers::normalize_path(path).starts_with(root))
            {
                file.root = root.to_path_buf();
            }
        }
        self.on_active_changed();
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

    /// Re-point the ACTIVE file's identity + path after it moves ON DISK —
    /// [`Self::open`] cannot do this: its lookup is BY KEY, and a move's new
    /// (path-derived) key never equals the old one, so it would read the
    /// moved file as a DIFFERENT file and push a second row instead of
    /// updating this one. A move never changes which root owns the slot, so
    /// this updates IN PLACE: the row stays exactly where it was, and its
    /// quiet parent label (derived live from `path`/`root`, see
    /// [`OpenFile::parent_label`]) reads correctly on the next draw. A no-op
    /// when nothing is active.
    pub fn rekey_active(&mut self, key: BufferKey, path: Option<PathBuf>) {
        if let Some(file) = self.active.and_then(|at| self.files.get_mut(at)) {
            file.key = key;
            file.path = path;
            self.refresh_direct();
        }
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

    /// Reveal the active file in the compact window after an activation.
    /// The expanded panel's own view follows if one was explicitly opened.
    fn on_active_changed(&mut self) {
        self.reveal_direct();
        if self.len() < 2 {
            self.panel = panel::Panel::Resting;
        } else if matches!(self.panel, panel::Panel::Expanded { .. }) {
            self.panel = panel::Panel::Expanded {
                scroll: self.expanded_reveal_scroll(),
            };
        }
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

    /// The one row projection both the compact and expanded views use, so
    /// they cannot
    /// describe the same open file differently.
    fn file_row(&self, at: usize) -> StackRow {
        let leaf = match self.files[at].key {
            BufferKey::Fresh(_) => {
                let fresh: Vec<usize> = self
                    .files
                    .iter()
                    .enumerate()
                    .filter_map(|(i, file)| matches!(file.key, BufferKey::Fresh(_)).then_some(i))
                    .collect();
                if fresh.len() <= 1 || fresh.first() == Some(&at) {
                    "untitled".to_string()
                } else {
                    let rank = fresh.iter().position(|&i| i == at).unwrap_or(0) + 1;
                    format!("untitled {rank}")
                }
            }
            _ => self.files[at].leaf(),
        };
        StackRow {
            leaf,
            parent: self.files[at].parent_label().unwrap_or_default(),
            active: self.active == Some(at),
            kind: StackRowKind::File,
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
            self.direct_rows(root)
        }
    }
}

#[cfg(test)]
mod tests;
