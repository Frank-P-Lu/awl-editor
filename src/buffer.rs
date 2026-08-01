use std::borrow::Cow;
use std::path::{Path, PathBuf};

use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Eol {
    #[default]
    Lf,
    Crlf,
}

impl Eol {
    pub fn detect(s: &str) -> Eol {
        // Two vectorized passes over the raw bytes rather than two scalar ones;
        // `\r\n` cannot overlap itself, so counting it non-overlapping matches
        // the `match_indices` semantics this replaced.
        let b = s.as_bytes();
        let total_lf = memchr::memchr_iter(b'\n', b).count();
        // Every '\n' immediately preceded by a '\r' is a CRLF pair.
        let crlf = memchr::memmem::find_iter(b, b"\r\n").count();
        let lone_lf = total_lf - crlf;
        if crlf > lone_lf { Eol::Crlf } else { Eol::Lf }
    }

    /// Encode text for disk without doubling CRLF introduced by other input paths.
    pub fn encode<'a>(&self, lf_text: &'a str) -> Cow<'a, str> {
        match self {
            Eol::Lf => Cow::Borrowed(lf_text),
            Eol::Crlf if lf_text.contains('\n') => {
                Cow::Owned(normalize_eol(lf_text).replace('\n', "\r\n"))
            }
            Eol::Crlf => Cow::Borrowed(lf_text),
        }
    }

    /// The short UI label for this ending — `"LF"` / `"CRLF"` — shown by the held
    /// stats HUD's LINE ENDINGS row and named in the capture sidecar's `hud.eol`
    /// field. A pure function, so it is deterministic and capture-safe.
    pub fn label(&self) -> &'static str {
        match self {
            Eol::Lf => "LF",
            Eol::Crlf => "CRLF",
        }
    }

    /// The OTHER ending — the target of the "Line endings…" toggle
    /// (`Lf`↔`Crlf`). awl recognizes exactly two, so a toggle is total.
    pub fn toggled(&self) -> Eol {
        match self {
            Eol::Lf => Eol::Crlf,
            Eol::Crlf => Eol::Lf,
        }
    }
}

/// Normalize a freshly-read file string to the buffer's pure-`\n` model: strip the
/// `\r` from every `\r\n` pair so no CRLF ever enters the rope. A LONE `\r` (or
/// NEL / LS / PS) is left untouched — it is ordinary content, not a line break
/// (the VS Code model). Allocation-light: a file with no `\r` at all borrows.
fn normalize_eol(s: &str) -> Cow<'_, str> {
    if s.contains('\r') {
        Cow::Owned(s.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(s)
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The ONE owner of the backward word-delete boundary (⌥⌫ / M-Backspace): from
/// char index `cursor`, first consume any trailing WHITESPACE run LEFT, then
/// delete exactly ONE token — a run of WORD chars if a word char now sits before
/// the caret, else a run of PUNCTUATION (non-word, non-whitespace) chars. Return
/// the char index the deletion should stop at.
///
/// This matches native macOS Option-Delete (verified 2026-07-22): a caret sitting
/// after a punctuation run deletes only that run, NOT the word before it — so
/// `abc ...⎸` leaves `abc ` (the word survives), while `abc def⎸` still deletes
/// only `def` and `abc def ⎸` deletes `def ` (a word plus the space that
/// introduced it). Punctuation and a word are DISTINCT token classes and never
/// delete together in one stroke; only leading whitespace folds into the token it
/// precedes. The old rule (skip-nonword-then-word) over-deleted the word after a
/// trailing punctuation run — the reported `abc ...⎸` bug.
///
/// `char_at(i)` yields the char at 0-based char index `i` (`i < cursor` always).
/// Abstract over the storage so the rope-backed [`Buffer::delete_word_backward`]
/// and the overlay minibuffer (a `String`) share this rule instead of duplicating
/// it.
pub(crate) fn word_delete_backward_boundary(
    cursor: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    let mut i = cursor;
    while i > 0 && char_at(i - 1).is_whitespace() {
        i -= 1;
    }
    if i == 0 {
        return 0;
    }
    if is_word_char(char_at(i - 1)) {
        while i > 0 && is_word_char(char_at(i - 1)) {
            i -= 1;
        }
    } else {
        // Punctuation: non-word AND non-whitespace (whitespace was consumed above
        // and a word char ends the run), so this never crosses into an adjacent word.
        while i > 0 && !is_word_char(char_at(i - 1)) && !char_at(i - 1).is_whitespace() {
            i -= 1;
        }
    }
    i
}

/// The ONE owner of the forward word-delete boundary (⌥+forward-Delete /
/// DeleteWordForward): from char index `cursor`, first consume any LEADING
/// WHITESPACE run to the RIGHT, then delete exactly ONE token — a run of WORD
/// chars if a word char now sits at the caret, else a run of PUNCTUATION
/// (non-word, non-whitespace) chars. Return the char index the deletion should
/// stop at.
///
/// The exact forward mirror of [`word_delete_backward_boundary`]: ONE token
/// class per stroke, so `⎸... abc` removes only the `...` run (the word `abc`
/// survives, leaving ` abc`), exactly as `... abc⎸` ⌥⌫ removes only `abc`.
/// Punctuation and a word are DISTINCT classes that never delete together; only
/// the whitespace that INTRODUCES a token folds into it. The old rule
/// (skip-nonword-then-word) over-deleted BOTH the punct run and the word after
/// it in one stroke — the forward twin of the backward bug item 3(a) fixed.
///
/// `char_at(i)` yields the char at 0-based char index `i` (`cursor <= i < len`).
pub(crate) fn word_delete_forward_boundary(
    cursor: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    let mut j = cursor;
    while j < len && char_at(j).is_whitespace() {
        j += 1;
    }
    if j == len {
        return len;
    }
    if is_word_char(char_at(j)) {
        while j < len && is_word_char(char_at(j)) {
            j += 1;
        }
    } else {
        // Punctuation: non-word AND non-whitespace (whitespace was consumed above
        // and a word char ends the run), so this never crosses into an adjacent word.
        while j < len && !is_word_char(char_at(j)) && !char_at(j).is_whitespace() {
            j += 1;
        }
    }
    j
}

/// The ONE owner of the WORD-MOTION forward boundary (M-f / Ctrl/Opt-Right /
/// [`crate::buffer::motion`]'s `Buffer::forward_word`): from char index
/// `cursor`, skip any run of NON-word chars, then skip the WORD-char run that
/// follows, returning the char index reached. `len` is the char length of the
/// text; `char_at(i)` yields the char at `i` (`cursor <= i < len`).
///
/// DISTINCT from [`word_delete_forward_boundary`] — motion and delete are
/// different rules (skip-then-skip vs. one-token-plus-its-introducing-
/// whitespace) that must never be conflated; see that function's own doc.
/// Abstracted over the storage so [`Buffer::forward_word`] (rope-backed) and
/// [`crate::textbox::TextBox::word_right`] (a plain `String`) share the SAME
/// rule instead of the textbox silently drifting from the document's own M-f.
///
/// The word rule runs over CHARS and then the answer is snapped OUTWARD to a
/// grapheme-cluster boundary ([`crate::grapheme`]). It must be: `is_word_char`
/// asks `char::is_alphanumeric`, which is `Alphabetic | Nd | Nl | No` and so
/// says NO to a combining acute (`Mn`) and to a variation selector — the run
/// therefore ends BETWEEN a base letter and the mark drawn on top of it, at a
/// position that does not exist on screen. Snapping changes the answer only
/// where it was interior to a cluster, never which chars count as a word.
pub(crate) fn word_forward_boundary(
    cursor: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    let mut i = cursor;
    while i < len && !is_word_char(char_at(i)) {
        i += 1;
    }
    while i < len && is_word_char(char_at(i)) {
        i += 1;
    }
    crate::grapheme::snap_forward(i, len, char_at)
}

/// The ONE owner of the WORD-MOTION backward boundary — the exact mirror of
/// [`word_forward_boundary`], snapped outward to the LEFT for the same reason
/// (a Devanagari conjunct's virama is non-word and sits mid-cluster), and
/// shared by [`Buffer::backward_word`] and [`crate::textbox::TextBox::word_left`].
/// `char_at(i)` yields the char at `i` (`0 <= i < cursor`); the walk never
/// reaches `cursor` itself, so the caret doubles as the snap's `len`.
pub(crate) fn word_backward_boundary(cursor: usize, char_at: impl Fn(usize) -> char) -> usize {
    let mut i = cursor;
    while i > 0 && !is_word_char(char_at(i - 1)) {
        i -= 1;
    }
    while i > 0 && is_word_char(char_at(i - 1)) {
        i -= 1;
    }
    crate::grapheme::snap_backward(i, cursor, char_at)
}

/// One recorded edit, the unit of undo. We store the CHANGE (op-based history),
/// not a whole-document snapshot, so memory is proportional to what was edited.
/// At char index `start`, the text `removed` was replaced by the text `inserted`.
/// `cursor_before` is where the cursor sat before the edit (restored on undo);
/// `cursor_after` is where it landed after (restored on redo). Inverting an edit
/// (undo) re-inserts `removed` in place of `inserted` and restores `cursor_before`.
#[derive(Clone, Debug)]
struct Edit {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: usize,
    cursor_after: usize,
}

/// The direction of the last recorded edit, used for coalescing. An insertion
/// run and a deletion run never merge into the same group.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EditKind {
    Insert,
    Delete,
}

pub struct Buffer {
    rope: Rope,
    cursor: usize,
    goal_col: Option<usize>,
    goal_x: Option<f32>,
    /// CARET WRAP AFFINITY: which visual row the caret RENDERS on when its column
    /// sits exactly on a SHARED soft-wrap boundary (see [`crate::caret::Affinity`]).
    /// `Upstream` (upper row's trailing edge) is set ONLY by a visual line-end
    /// motion (C-e / End / Cmd-Right); every other motion / edit clears it back to
    /// `Downstream`, exactly like `goal_x`'s lifecycle — so it only survives on a
    /// caret parked at a visual-row end. The buffer carries it opaquely; the render
    /// pipeline reads it (via [`Self::affinity`]) to disambiguate the two legit
    /// renders of the boundary column.
    affinity: crate::caret::Affinity,
    /// LIST-CONTINUATION PROVENANCE (item 78, short-lived): true for exactly one
    /// beat after `actions::edit::smart_newline`'s list-item Continue arm opens a
    /// BARE, otherwise-empty bullet/numbered/task continuation line (nothing
    /// carried over from the split line) — so the very next smart-newline
    /// decision on THIS line can tell "awl just generated this empty marker"
    /// apart from "this empty marker's bytes came from anywhere else" (typed,
    /// loaded from disk, undone/redone back into place, or reached after any
    /// other edit/motion/selection change). Read-and-cleared by
    /// [`Self::take_list_continuation_generated`]; the SET is the one place in
    /// `smart_newline`. Cleared — never left to go stale — by every one of this
    /// buffer's mutation/motion choke points (`apply_edit`, `clear_kill_flag`,
    /// `undo`/`redo`, `clear_mark`, `set_cursor_visual`) and by a buffer
    /// park/activate swap (`app/files/active.rs`), so identical bytes loaded
    /// from disk — or reached by ANY route other than that one immediately
    /// preceding continuation — are never guessed to be generated.
    list_continuation_generated: bool,
    path: Option<PathBuf>,
    /// This buffer's line-ending discipline (the VS Code model): the rope is
    /// ALWAYS pure-`\n`, and `eol` remembers the file's original ending so a save
    /// restores it byte-for-byte ([`Self::disk_bytes`]). Detected on load
    /// ([`Self::from_file`]); [`Eol::Lf`] for a fresh / scratch / note buffer.
    eol: Eol,
    /// QUICK NOTE target directory: set when this buffer is a freshly-summoned
    /// scrap note (C-x n) that has not been named yet. While `path` is `None` and
    /// this is `Some`, the first `save()` DERIVES the filename from the buffer's
    /// first non-empty line (slugified) under this directory — "capture first,
    /// name later". Stays set after the first save so the windowed app keeps
    /// auto-saving the note; the filename then LOCKS (save writes the bound path).
    /// `None` for ordinary files and scratch buffers (which never auto-name).
    note_dir: Option<PathBuf>,
    kill: String,
    last_was_kill: bool,
    dirty: bool,
    anchor: Option<usize>,
    version: u64,
    /// Undo stack: completed (and the in-progress) edit groups, oldest first.
    /// Each group is a run of coalesced [`Edit`]s applied together; one undo pops
    /// and inverts the whole top group. A fresh edit may extend the top group (see
    /// coalescing rules in [`record_edit`]) or push a new one.
    undo_stack: Vec<Vec<Edit>>,
    /// Redo stack: groups popped by undo, ready to re-apply. Cleared by any NEW
    /// edit (linear, modern-editor history — undo is not itself undoable).
    redo_stack: Vec<Vec<Edit>>,
    /// True when the top undo group is "open" and a contiguous same-direction edit
    /// may coalesce into it. Sealed (set false) by [`seal_undo_group`] after any
    /// non-edit command, and internally when a group-breaking edit occurs.
    undo_group_open: bool,
    last_edit_kind: Option<EditKind>,
    /// COLLAPSED SECTIONS (view state, never file content): the set of ATX heading
    /// LOGICAL LINES whose sections are folded. Pure in-memory render state for the
    /// app run — it survives a buffer switch (the whole `Buffer` parks in the
    /// registry) but is NOT serialized to disk / session and is NOT on the undo
    /// timeline (undo replays rope `Edit`s, never this field). Empty for the
    /// overwhelming common case, so every fold read short-circuits to a no-op. The
    /// section extent + auto-expand rules live in [`crate::fold`]; this buffer owns
    /// only the set + the caret-relative gestures over it.
    folds: std::collections::BTreeSet<usize>,
}

impl Buffer {
    pub fn scratch() -> Self {
        Self::from_rope(Rope::new(), None)
    }

    /// Load a file into a buffer. A missing file yields an empty buffer bound to
    /// that path (so the first Cmd-S creates it), matching mg behavior.
    ///
    /// LINE ENDINGS (VS Code model): the file's DOMINANT ending is detected
    /// ([`Eol::detect`]) and remembered, then every `\r\n` is normalized to `\n`
    /// ([`normalize_eol`]) BEFORE the text enters the rope — so the buffer is
    /// purely `\n`-based and agrees with the `\n`-only renderer by construction.
    /// A save restores the remembered ending ([`Self::disk_bytes`]), so a CRLF
    /// file round-trips byte-for-byte. A missing file defaults to [`Eol::Lf`].
    pub fn from_file(path: &Path) -> Self {
        let (rope, eol) = match crate::fs::active().read_to_string(path) {
            Ok(s) => (Rope::from_str(&normalize_eol(&s)), Eol::detect(&s)),
            Err(_) => (Rope::new(), Eol::Lf),
        };
        let mut buf = Self::from_rope(rope, Some(path.to_path_buf()));
        buf.eol = eol;
        buf
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        Self::from_rope(Rope::from_str(s), None)
    }

    fn from_rope(rope: Rope, path: Option<PathBuf>) -> Self {
        Self {
            rope,
            cursor: 0,
            goal_col: None,
            goal_x: None,
            affinity: crate::caret::Affinity::Downstream,
            list_continuation_generated: false,
            path,
            eol: Eol::Lf,
            note_dir: None,
            kill: String::new(),
            last_was_kill: false,
            dirty: false,
            anchor: None,
            version: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_group_open: false,
            last_edit_kind: None,
            folds: std::collections::BTreeSet::new(),
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// This buffer's line-ending discipline (see [`Eol`]). The rope is always
    /// pure-`\n`; this reports what a save will restore.
    pub fn eol(&self) -> Eol {
        self.eol
    }

    /// Switch this buffer's line-ending discipline (the palette's "Convert Line
    /// Endings" command calls this). The rope is UNCHANGED — it is always pure
    /// `\n`; only the on-disk encoding differs — so this is metadata, not a text
    /// edit. Design choice (documented): EOL is NOT part of the undo history, and
    /// Cmd-Z does not restore it (mirroring VS Code, where the ending is a
    /// document-level setting, not an undoable edit; the rope content is
    /// byte-identical either way, so there is nothing in the text for undo to
    /// restore). A real change bumps `version` + marks the buffer dirty so the
    /// autosave engine rewrites the file with the new ending on the next flush; a
    /// no-op switch (same ending) leaves everything untouched.
    pub fn set_eol(&mut self, eol: Eol) {
        if self.eol == eol {
            return;
        }
        self.eol = eol;
        self.dirty = true;
        self.version += 1;
    }

    /// The buffer's content encoded to its ON-DISK byte form: the pure-`\n` rope
    /// string with this buffer's [`Eol`] restored ([`Eol::encode`]). The ONE owner
    /// of "buffer content → disk bytes" — every save path routes through it (manual
    /// [`Self::save`], the autosave engine, the scratch stash), so a CRLF file is
    /// rewritten with `\r\n` and an LF file is byte-identical to today. Distinct
    /// from [`Self::text`], which is the internal pure-`\n` view every other reader
    /// (spell / search / markdown / render) wants.
    pub fn disk_bytes(&self) -> Vec<u8> {
        let text = self.rope.to_string();
        match self.eol.encode(&text) {
            // Lf (or a `\n`-free Crlf buffer): reuse the rope string's own buffer.
            Cow::Borrowed(_) => text.into_bytes(),
            // Crlf with real `\n`s: the freshly-rewritten `\r\n` string.
            Cow::Owned(s) => s.into_bytes(),
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(p) = &self.path {
            return p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "scratch".to_string());
        }
        let stem = match first_nonempty_line(&self.rope.to_string()) {
            Some(line) => note_stem(line),
            None => "scratch".to_string(),
        };
        format!("{stem}.md")
    }

    /// True when this buffer is a MARKDOWN document. awl is a prose-first writing
    /// app, so the rule is unified and prose-leaning: a buffer with NO path — the
    /// bare SCRATCH launch surface or an unnamed FRESH DOCUMENT — defaults to
    /// markdown, styling `# title` / **bold** as you type on the blank writing
    /// surface; a SAVED file is markdown only by its `.md` / `.markdown` extension
    /// (case-insensitive). So a `.rs` / `.txt` / `.env` file (a path with a
    /// non-markdown extension) stays NOT markdown — code/.env files always open
    /// WITH a path, so they are unaffected. (The no-path arm subsumes
    /// [`Self::is_unnamed_fresh`] — a fresh document is always unsaved-then-`.md`
    /// — and is what makes it read as markdown from the first keystroke, before
    /// its first save derives a `.md` path.)
    /// Gates the renderer's markdown styling pass. Syntax highlighting stays
    /// path-based ([`Self::syntax_lang`]), so a no-path buffer reports no code
    /// language and is never code-highlighted — markdown and code remain mutually
    /// exclusive even for the scratch surface.
    pub fn is_markdown(&self) -> bool {
        match self.path.as_deref() {
            None => true,
            Some(p) => p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                .unwrap_or(false),
        }
    }

    /// The CODE language for syntax highlighting, or `None` when this buffer must
    /// NOT be highlighted — decided purely by the file extension via
    /// [`crate::syntax::Lang::from_path`]. The gate excludes `.env`, `.md`/
    /// `.markdown` (own markdown styling), `.txt`, and any unrecognized / scratch
    /// buffer, so those render byte-identically. Markdown and code are mutually
    /// exclusive: a `.md` buffer is [`Self::is_markdown`] with no `syntax_lang`.
    pub fn syntax_lang(&self) -> Option<crate::syntax::Lang> {
        self.path
            .as_deref()
            .and_then(crate::syntax::Lang::from_path)
    }

    /// Which STICKY page-width CLASS this buffer draws its measure from — see
    /// [`crate::page::PageClass`]. Delegates to the ONE classifier
    /// (`PageClass::of_syntax`), driven by [`Self::syntax_lang`], so it can
    /// never disagree with the syntax-highlighting gate: a recognized CODE
    /// file is `Code`; markdown / the no-path scratch-or-note surface / an
    /// unrecognized plain-text file is `Prose`.
    pub fn page_class(&self) -> crate::page::PageClass {
        crate::page::PageClass::of_syntax(self.syntax_lang())
    }

    pub fn set_path(&mut self, p: PathBuf) {
        self.path = Some(p);
    }

    /// Mark this buffer as a freshly-summoned, UNNAMED document living under
    /// `dir`: it has no filename yet; the first non-empty line names it ONCE, on
    /// the first material save ([`Self::save`] then clears this — item 76's
    /// one-shot naming law: a LATER title edit never re-triggers a rename, since
    /// [`Self::is_unnamed_fresh`] is false from that first save on).
    pub fn set_note_dir(&mut self, dir: PathBuf) {
        self.note_dir = Some(dir);
    }

    pub fn is_unnamed_fresh(&self) -> bool {
        self.note_dir.is_some()
    }

    pub fn start_fresh_doc(&mut self, dir: PathBuf) {
        *self = Self::from_rope(Rope::new(), None);
        self.note_dir = Some(dir);
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn kill_buffer(&self) -> &str {
        &self.kill
    }

    /// Pure setter for the kill ring's top entry. Used by the app's clipboard
    /// bridge to load an external OS-clipboard value before a yank. Overwrites
    /// (does not append) and MUST NOT touch `last_was_kill`: loading an external
    /// value is not a kill, so a subsequent C-k must start a fresh kill rather
    /// than chaining onto this. No winit/gpu/arboard here — buffer stays pure.
    ///
    /// NORMALIZES any `\r\n` the external source used to the rope's pure-`\n`
    /// invariant ([`normalize_eol`], matching [`Self::from_file`]) — a pasted
    /// Windows/CRLF clipboard value can therefore never introduce a real `\r\n`
    /// into the rope on yank (which a `Crlf` save would otherwise double-encode).
    /// A lone `\r` stays content (the established lone-CR decision).
    pub fn set_kill(&mut self, s: &str) {
        self.kill.clear();
        self.kill.push_str(&normalize_eol(s));
    }

    pub fn cursor_line_col(&self) -> (usize, usize) {
        let line = self.rope.char_to_line(self.cursor);
        let line_start = self.rope.line_to_char(line);
        (line, self.cursor - line_start)
    }

    pub fn folds(&self) -> &std::collections::BTreeSet<usize> {
        &self.folds
    }

    pub fn has_folds(&self) -> bool {
        !self.folds.is_empty()
    }

    /// The per-logical-line HIDDEN mask for the current fold set — `true` where a
    /// line is collapsed inside a folded section (never the heading line itself).
    /// Empty-of-`true` when nothing is folded. The render builds its fold-filtered
    /// text from this ([`crate::fold::Filter`]).
    pub fn hidden_lines(&self) -> Vec<bool> {
        if self.folds.is_empty() {
            return Vec::new();
        }
        let levels = self.heading_levels();
        crate::fold::hidden_lines(&levels, &self.folds)
    }

    fn heading_levels(&self) -> Vec<u8> {
        crate::fold::heading_levels(&self.text(), self.is_markdown())
    }

    pub fn fold_tails(&self) -> Vec<(usize, usize)> {
        if self.folds.is_empty() {
            return Vec::new();
        }
        let levels = self.heading_levels();
        crate::fold::fold_tails(&levels, &self.folds)
    }

    pub fn visible_line_to_full(&self, visible_line: usize) -> usize {
        if self.folds.is_empty() {
            return visible_line;
        }
        let levels = self.heading_levels();
        let hidden = crate::fold::hidden_lines(&levels, &self.folds);
        crate::fold::visible_to_full(&hidden, visible_line)
    }

    /// CLICK-TO-EXPAND hit test: given a pointer's VISIBLE `(line, col)` (as the
    /// render's hit-test yields), return the FULL-document heading line to EXPAND when
    /// the click landed on a collapsed heading's "… N lines" TAIL (past the heading's
    /// own character length — `col >= line_len`). `None` when nothing is folded, the
    /// clicked visible line is not a collapsed heading, or the click is ON the heading
    /// text (which places the caret for editing, unchanged). Column-based (no pixel
    /// geometry): a click past the last glyph is unambiguously "the affordance", never
    /// content. **item 65 note:** the expand CHEVRON moved to the LEFT margin (a
    /// summoned VISUAL cue); this hit region — and this fn's behavior — is UNCHANGED
    /// by that move. **item 81 update:** the chevron IS now its own (separate,
    /// left-margin, pixel-hit) click target — see [`Self::toggle_fold_at_line`] — so a
    /// collapsed heading keeps TWO generous expand doors: this tail region, still
    /// unchanged, plus the chevron. Clicking anywhere past the heading text (where the
    /// tail still hangs) keeps expanding exactly as before.
    pub fn fold_tail_hit(&self, visible_line: usize, col: usize) -> Option<usize> {
        if self.folds.is_empty() {
            return None;
        }
        let full = self.visible_line_to_full(visible_line);
        if !self.folds.contains(&full) {
            return None; // not a collapsed heading's row
        }
        (col >= self.line_len(full)).then_some(full)
    }

    pub fn unfold_at(&mut self, heading_line: usize) -> bool {
        if self.folds.remove(&heading_line) {
            self.set_cursor(self.line_start(heading_line));
            true
        } else {
            false
        }
    }

    pub fn toggle_fold_at_cursor(&mut self) -> Option<usize> {
        let levels = self.heading_levels();
        let (line, _) = self.cursor_line_col();
        let h = crate::fold::toggle_at(&levels, &mut self.folds, line)?;
        if self.folds.contains(&h) {
            self.set_cursor(self.line_start(h));
        }
        Some(h)
    }

    /// THE FOLD CHEVRON's own click target (item 81): toggle the fold on EXACTLY
    /// `heading_line` — fold it if open, unfold it if folded — regardless of where
    /// the caret currently sits. The ONE owner BOTH directions of a chevron click
    /// share (`crate::fold::toggle_heading`, the same function
    /// [`Self::toggle_fold_at_cursor`]'s own `toggle_at` routes through), unlike
    /// [`Self::unfold_at`] (the tail's expand-only click target). On a FOLD (not an
    /// unfold), the caret parks on the heading line — mirrors
    /// `toggle_fold_at_cursor`'s "never leave the caret inside the section it just
    /// hid" rule; an UNFOLD leaves the caret exactly where it was. Returns `false`
    /// (no-op) when `heading_line` does not name a real heading (a stale line after
    /// an edit, or an out-of-range index) — a click can never invent a fold.
    pub fn toggle_fold_at_line(&mut self, heading_line: usize) -> bool {
        let levels = self.heading_levels();
        if !crate::fold::toggle_heading(&levels, &mut self.folds, heading_line) {
            return false;
        }
        if self.folds.contains(&heading_line) {
            self.set_cursor(self.line_start(heading_line));
        }
        true
    }

    pub fn collapse_other_sections(&mut self) {
        if !self.is_markdown() {
            return;
        }
        let levels = self.heading_levels();
        let (line, _) = self.cursor_line_col();
        self.folds = crate::fold::collapse_others(&levels, line);
    }

    /// THE ONE REVEALED CARET/SELECTION PLACEMENT OWNER. Every gesture that PLACES
    /// the caret or a selection directly — action motion, single / shift / double /
    /// triple click and drag endpoints, search next/previous, heading & line jumps,
    /// and their headless-replay twins — routes through here (never through the two
    /// private halves below) so a caret or selection can never be left logically
    /// inside a hidden row. It reveals every fold hiding the caret line, every fold
    /// the selection would span invisibly, and prunes stale entries. Idempotent; a
    /// cheap no-op when nothing is folded. Deliberate NON-revealing setup seams — the
    /// low-level [`Buffer::set_cursor`], and the fold gestures that park the caret on
    /// a still-visible heading ([`Buffer::toggle_fold_at_cursor`] /
    /// [`Buffer::collapse_other_sections`] / [`Buffer::unfold_at`]) — do NOT call
    /// this. Returns true when the fold set changed.
    pub fn reveal_placement(&mut self) -> bool {
        if self.folds.is_empty() {
            return false;
        }
        let mut changed = self.reveal_cursor();
        changed |= self.reveal_selection();
        changed
    }

    /// AUTO-EXPAND: reveal any fold that hides the caret line (and prune stale
    /// entries whose heading was edited away). Cheap no-op when nothing is folded.
    /// PRIVATE — one half of [`Buffer::reveal_placement`], the single door every
    /// placement seam routes through; call that, never this. Returns true when the
    /// fold set changed. See [`crate::fold::expand_containing`].
    fn reveal_cursor(&mut self) -> bool {
        if self.folds.is_empty() {
            return false;
        }
        let levels = self.heading_levels();
        let mut changed = crate::fold::prune_stale(&levels, &mut self.folds);
        let (line, _) = self.cursor_line_col();
        changed |= crate::fold::expand_containing(&levels, &mut self.folds, line);
        changed
    }

    /// AUTO-EXPAND: reveal any fold the active selection would span INVISIBLY, so a
    /// selection never crosses hidden lines. No-op when nothing is folded or there
    /// is no selection. PRIVATE — the other half of [`Buffer::reveal_placement`];
    /// route through that. See [`crate::fold::expand_range`].
    fn reveal_selection(&mut self) -> bool {
        if self.folds.is_empty() {
            return false;
        }
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        let lo = self.rope.char_to_line(start);
        let hi = self.rope.char_to_line(end);
        let levels = self.heading_levels();
        crate::fold::expand_range(&levels, &mut self.folds, lo, hi)
    }

    #[allow(dead_code)]
    pub fn cursor_char(&self) -> usize {
        self.cursor
    }

    pub fn cursor_byte(&self) -> usize {
        self.rope.char_to_byte(self.cursor)
    }

    pub fn char_to_byte(&self, ch: usize) -> usize {
        self.rope.char_to_byte(ch.min(self.rope.len_chars()))
    }

    fn line_start(&self, line: usize) -> usize {
        self.rope.line_to_char(line)
    }

    fn line_len(&self, line: usize) -> usize {
        let total_lines = self.rope.len_lines();
        if line >= total_lines {
            return 0;
        }
        let start = self.rope.line_to_char(line);
        let end = if line + 1 < total_lines {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        let mut len = end - start;
        if len > 0 {
            let last = self.rope.char(end - 1);
            if last == '\n' {
                len -= 1;
            }
        }
        len
    }

    fn clear_kill_flag(&mut self) {
        self.last_was_kill = false;
        self.goal_x = None;
        // A plain motion / edit also drops any caret wrap-affinity: the caret is no
        // longer parked at a visual-row END, so at a shared boundary it renders on
        // the LOWER row again (the default bias). The visual line-END motion re-sets
        // `Upstream` AFTER its `set_cursor`, so only that survives (see
        // `crate::caret::Affinity`).
        self.affinity = crate::caret::Affinity::Downstream;
        // Item 78: any plain motion / edit routed through here is exactly the kind
        // of "something intervened" this provenance flag must not survive.
        self.list_continuation_generated = false;
    }

    pub(crate) fn take_list_continuation_generated(&mut self) -> bool {
        std::mem::take(&mut self.list_continuation_generated)
    }

    pub(crate) fn mark_list_continuation_generated(&mut self) {
        self.list_continuation_generated = true;
    }

    /// The caret's current wrap AFFINITY (which visual row it renders on at a
    /// shared soft-wrap boundary — see [`crate::caret::Affinity`]). Read by the
    /// render pipeline's caret placement; `Downstream` for any caret not parked at
    /// a visual-row end.
    pub fn affinity(&self) -> crate::caret::Affinity {
        self.affinity
    }

    /// Mark the caret's wrap AFFINITY. Called ONLY by the visual line-END motion
    /// (C-e / End / Cmd-Right) with `Upstream`, AFTER `set_cursor` has parked the
    /// caret at the boundary column (so it survives that call's clear). Every other
    /// motion / edit resets it to `Downstream` through `clear_kill_flag` /
    /// `set_cursor_visual` / `apply_edit`.
    pub fn set_affinity(&mut self, affinity: crate::caret::Affinity) {
        self.affinity = affinity;
    }

    /// The word (or the run of non-word chars) around `idx` — what a
    /// DOUBLE-CLICK selects, and the unit a word-granularity drag extends by.
    /// Both ends are snapped OUTWARD to grapheme-cluster boundaries, since the
    /// caret lands on one of them: the char-class walk alone ends a word before
    /// a trailing combining mark, which would park the caret inside the `é` it
    /// just selected.
    pub fn word_bounds(&self, idx: usize) -> (usize, usize) {
        let len = self.rope.len_chars();
        if len == 0 {
            return (0, 0);
        }
        let idx = idx.min(len);
        let class_at = |i: usize| -> Option<bool> {
            if i < len {
                Some(is_word_char(self.rope.char(i)))
            } else {
                None
            }
        };
        let want = class_at(idx)
            .or_else(|| if idx > 0 { class_at(idx - 1) } else { None })
            .unwrap_or(true);
        let mut start = idx;
        while start > 0 && is_word_char(self.rope.char(start - 1)) == want {
            start -= 1;
        }
        let mut end = idx;
        while end < len && is_word_char(self.rope.char(end)) == want {
            end += 1;
        }
        (
            crate::grapheme::snap_backward(start, len, |i| self.rope.char(i)),
            crate::grapheme::snap_forward(end, len, |i| self.rope.char(i)),
        )
    }

    pub fn line_bounds(&self, idx: usize) -> (usize, usize) {
        let idx = idx.min(self.rope.len_chars());
        let line = self.rope.char_to_line(idx);
        let start = self.line_start(line);
        let total_lines = self.rope.len_lines();
        let end = if line + 1 < total_lines {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        (start, end)
    }

    /// Replace the ENTIRE buffer contents with `new` as ONE atomic, undoable edit,
    /// then seal the group so it is its own undo step. The cursor lands at the end
    /// of the inserted text (callers that care reposition it afterward). Used by
    /// find-and-replace, which computes the post-replace document wholesale; a
    /// no-op replacement (identical text) is the caller's to skip.
    pub fn set_text(&mut self, new: &str) {
        self.clear_kill_flag();
        self.goal_col = None;
        self.anchor = None;
        let before = self.cursor;
        let len = self.rope.len_chars();
        let after = new.chars().count();
        self.apply_edit(0, len, new, before, after);
        self.seal_undo_group();
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        self.clear_kill_flag();
        self.goal_col = None;
        let max = self.rope.len_chars();
        self.anchor = Some(start.min(max));
        self.cursor = end.min(max);
    }

    /// Save to the bound path. For an UNNAMED FRESH DOCUMENT that has not been
    /// named yet (`path` is None but `note_dir` is set), DERIVE the filename from
    /// the first non-empty line — slugified, collision-suffixed — under
    /// `note_dir`, bind it, and write there; an EMPTY document bails (no file
    /// written, no litter). Returns Err if there is no path and no name can be
    /// derived.
    ///
    /// **ONE-SHOT NAMING (item 76):** this is the ONLY place a fresh document's
    /// filename is ever derived. Once bound, `note_dir` is cleared in the SAME
    /// step — [`Self::is_unnamed_fresh`] is false from this call on, so it reads
    /// as an ORDINARY pathed file thereafter. A later edit to the first line
    /// never re-derives or renames it (the old LIVE-rename-to-title behavior is
    /// retired — Rename is now the one, explicit, generic verb for that).
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.path.is_none()
            && let Some(dir) = self.note_dir.clone()
        {
            let text = self.rope.to_string();
            match first_nonempty_line(&text) {
                Some(line) => {
                    let stem = note_stem(line);
                    crate::fs::active().create_dir_all(&dir)?;
                    let path = unique_path(&dir, &stem, "md");
                    self.path = Some(path);
                    // ONE-SHOT: the name is derived exactly once — clear the
                    // fresh-document marker so a later first-line edit never
                    // re-triggers a rename.
                    self.note_dir = None;
                }
                // A truly empty document (no non-whitespace anywhere) is
                // NEVER written — no litter.
                None => anyhow::bail!("empty note: nothing to save yet"),
            }
        }
        match &self.path {
            Some(p) => {
                // ATOMIC: temp sibling + rename, so a crash mid-save leaves the
                // old file or the new one — never a truncated half-write. The
                // buffer's remembered line ending is restored here ([`disk_bytes`]),
                // so a CRLF file round-trips byte-for-byte.
                crate::fs::write_atomic(p, &self.disk_bytes())?;
                self.dirty = false;
                Ok(())
            }
            None => anyhow::bail!("no file bound to this buffer (scratch)"),
        }
    }

    /// SAVE-FEEDBACK round: manual save on the TRUE scratch surface (no path,
    /// never a fresh-document marker) converts it into an unnamed fresh document
    /// bound to `folder` FIRST, then saves — reusing the exact auto-name recipe
    /// [`Self::set_note_dir`] + [`Self::save`] already give Cmd-N (the same one
    /// `App::ensure_note_named_before_paste` established for the paste-image
    /// door, generalized here to manual save). A buffer that is ALREADY an
    /// unnamed fresh document, or already pathed, is left untouched — this only
    /// ever promotes a true scratch buffer, and only once (`is_unnamed_fresh()`
    /// is true from then on, so a second call is a plain `save()`). `folder`
    /// need not exist yet: creating it is best-effort (mirroring
    /// `App::new_document`); if it truly can't be created or written to, that
    /// failure surfaces as the same `Err` `save` already returns, for the caller
    /// to turn into a calm notice — never a terminal print.
    pub fn save_into_folder(&mut self, folder: &Path) -> anyhow::Result<()> {
        if !self.is_unnamed_fresh() {
            let _ = crate::fs::active().create_dir_all(folder);
            self.set_note_dir(folder.to_path_buf());
        }
        self.save()
    }
}

mod selection;

mod motion;

/// UNDO / REDO ENGINE — the `apply_edit` mutation choke point + the op-based
/// history (coalescing) + undo / redo / seal. Inherent methods on [`Buffer`],
/// carved out verbatim; `apply_edit` is `pub(super)` for the edit / selection
/// modules + this root.
mod undo;

mod edit;
#[allow(unused_imports)]
pub use edit::is_url;

mod notes;
pub use notes::*;

#[cfg(test)]
mod tests;
