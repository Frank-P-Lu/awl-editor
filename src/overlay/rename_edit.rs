//! The RENAME minibuffer sub-state: `RenameEdit` and its `OverlayState` verbs,
//! split out of `overlay::capture` to keep that file under its file-size mark —
//! same module, own file, no ownership change.

use super::{OverlayKind, OverlayState};
use crate::textbox::TextBox;

/// NOTES VERBS round: the live RENAME minibuffer sub-state — the current typed
/// filename plus the original (for the prompt's "unchanged" no-op check). Pure +
/// serialisable, mirroring [`ValueEdit`]'s exact shape but WITHOUT the numeric/`.`/`%`
/// filter (a filename accepts any character except the path separator `/`, which
/// would let a typed name silently escape into a different directory). While it is
/// `Some`, the Rename overlay OWNS every key at the intercept level (any printable
/// char extends `input`, Backspace deletes, Enter commits, Esc cancels) — see
/// [`super::overlay_nav`]'s `rename_edit`-first check (`actions/overlay_nav.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameEdit {
    /// The text typed so far + its CHAR-index caret, seeded (caret at
    /// END) from the file's current name. The `/`-REJECT filter stays here in
    /// [`OverlayState::rename_edit_push`] — `TextBox::insert` itself accepts any char.
    pub input: TextBox,
    /// The name at edit start (unused by the core beyond equality — the CALLER's
    /// own "unchanged input is a no-op" gate reads it via `Effect::RenameNoteCommit`
    /// naming the typed value; kept here so a future cancel-restore path, or a test,
    /// never has to re-derive it).
    pub orig: String,
}

impl RenameEdit {
    /// The dim PROMPT line the card shows while renaming, surfaced to the sidecar's
    /// `overlay.hint` via [`OverlayState::foot_hint`] — exactly the seam the
    /// Keybindings capture's own `Capture::prompt` rides, so the minibuffer's typing
    /// state is `--keys`-verifiable with ZERO new sidecar plumbing.
    pub fn prompt(&self) -> String {
        format!(
            "rename to: {}   Enter commit   Esc cancel",
            self.input.text()
        )
    }
}

impl OverlayState {
    /// NOTES VERBS round: RENAME the current file — build the fresh minibuffer
    /// state, pre-filled with `current_name` (which becomes the single editable
    /// row's primary cell too — corpus and `rename_edit.input` start in lockstep so
    /// the very first frame already shows the seeded name, not an empty row) with
    /// the STEM selected and the extension untouched — the file-manager rename
    /// convention: the first keystroke replaces the stem outright, and a bare
    /// Backspace/Delete never eats into the extension by accident.
    /// `Path::file_stem` already gets this right for every shape without a
    /// special case — a normal `name.ext` selects `name`; no extension or a
    /// dotfile (`.gitignore`) selects the WHOLE name (its own `file_stem` IS the
    /// whole name); `archive.tar.gz` selects `archive.tar`, stripping only the
    /// last extension, same as a file manager. An empty name selects nothing
    /// (`file_stem` is `None`), so a bare Enter/typing-from-scratch caller is
    /// unaffected — the same seed [`TextBox::seeded`] always produced.
    pub fn new_rename(current_name: String) -> Self {
        let mut s = Self::new_marked(
            OverlayKind::Rename,
            vec![current_name.clone()],
            vec![false],
            vec![false],
            Vec::new(),
            Vec::new(),
            None,
        );
        let stem_len = std::path::Path::new(&current_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.chars().count())
            .unwrap_or(0);
        s.rename_edit = Some(RenameEdit {
            input: TextBox::seeded_selecting_prefix(&current_name, stem_len),
            orig: current_name,
        });
        s.rename_edit_mirror();
        s
    }

    /// RENAME MINIBUFFER: mirror the live `rename_edit.input` — text, caret AND
    /// selection — into `corpus[0]`'s primary cell (RENAME has no separate
    /// `bindings` secondary column, so the live-typed name IS the primary cell)
    /// and into `query`, the ONE field the render path already tracks a
    /// per-character caret/selection box for (`render::chrome::overlay_draw
    /// ::overlay_query_caret_box`) — so the seeded selection this module arms is
    /// actually visible, with zero new caret-geometry plumbing. `query` is
    /// otherwise unused by Rename (no fuzzy filtering happens on it); every
    /// mutator below re-derives it wholesale from `rename_edit.input` rather than
    /// being mutated directly, so the two can never drift. A no-op when no
    /// rename edit is active.
    fn rename_edit_mirror(&mut self) {
        let Some(re) = self.rename_edit.as_ref() else {
            return;
        };
        let text = re.input.text().to_string();
        if let Some(row) = self.rows.get_mut(0) {
            row.accept = text;
        }
        self.query = re.input.clone();
    }

    /// RENAME MINIBUFFER: insert `c` at the caret UNLESS it is `/` — a path
    /// separator would let a typed name silently escape into a different
    /// directory, which this verb (a same-directory rename) never does; every other
    /// character is accepted (unlike `value_edit_push`'s digit-only filter — a
    /// filename is free text). The FILTER stays here — `TextBox::insert` itself
    /// accepts any char. A no-op when no rename edit is active.
    pub fn rename_edit_push(&mut self, c: char) {
        let Some(re) = self.rename_edit.as_mut() else {
            return;
        };
        if c != '/' {
            re.input.insert(c);
        }
        self.rename_edit_mirror();
    }

    /// RENAME MINIBUFFER: ⌥⌫ word-delete — drop the trailing word (the word-DELETE
    /// rule), mirroring the change into `corpus[0]`. A no-op when no rename edit is
    /// active.
    pub fn rename_edit_pop_word(&mut self) {
        let Some(re) = self.rename_edit.as_mut() else {
            return;
        };
        re.input.delete_word_back();
        self.rename_edit_mirror();
    }

    /// RENAME MINIBUFFER: delete the char before the caret, mirroring the change
    /// into `corpus[0]`. A no-op when no rename edit is active.
    pub fn rename_edit_pop(&mut self) {
        let Some(re) = self.rename_edit.as_mut() else {
            return;
        };
        re.input.delete_back();
        self.rename_edit_mirror();
    }

    /// RENAME MINIBUFFER char/word motion + forward word-delete. Each still
    /// calls [`Self::rename_edit_mirror`] even though the TEXT never changes —
    /// a motion can COLLAPSE the seeded selection or move the caret, and the
    /// mirrored `query` field is what the render path's caret/selection box
    /// actually reads. A no-op when no rename edit is active.
    pub fn rename_edit_char_left(&mut self) {
        if let Some(re) = self.rename_edit.as_mut() {
            re.input.char_left();
        }
        self.rename_edit_mirror();
    }
    pub fn rename_edit_char_right(&mut self) {
        if let Some(re) = self.rename_edit.as_mut() {
            re.input.char_right();
        }
        self.rename_edit_mirror();
    }
    pub fn rename_edit_word_left(&mut self) {
        if let Some(re) = self.rename_edit.as_mut() {
            re.input.word_left();
        }
        self.rename_edit_mirror();
    }
    pub fn rename_edit_word_right(&mut self) {
        if let Some(re) = self.rename_edit.as_mut() {
            re.input.word_right();
        }
        self.rename_edit_mirror();
    }
    pub fn rename_edit_delete_word_forward(&mut self) {
        let Some(re) = self.rename_edit.as_mut() else {
            return;
        };
        re.input.delete_word_forward();
        self.rename_edit_mirror();
    }

    /// RENAME MINIBUFFER click-to-place: the [`OverlayState::query_set_caret`]
    /// door reads back a click's column, but the SEEDED SELECTION lives on
    /// `rename_edit.input`, not `query` (a mirror — see
    /// [`Self::rename_edit_mirror`]) — a click routed straight at `query`
    /// would place the caret there only for the mirror to snap it back on the
    /// next keystroke. Placing it here first, THEN mirroring, keeps the click
    /// authoritative. A no-op when no rename edit is active.
    pub(super) fn rename_edit_set_caret(&mut self, at: usize) {
        if let Some(re) = self.rename_edit.as_mut() {
            re.input.set_caret(at);
        }
        self.rename_edit_mirror();
    }

    /// RENAME MINIBUFFER commit target: the typed filename, consumed when Enter
    /// commits. `None` when no rename edit is active.
    pub fn rename_edit_target(&self) -> Option<String> {
        self.rename_edit
            .as_ref()
            .map(|re| re.input.text().to_string())
    }
}
