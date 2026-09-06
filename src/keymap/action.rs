#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ForwardChar,
    BackwardChar,
    NextLine,
    PreviousLine,
    LineStart,
    LineEnd,
    ForwardWord,
    BackwardWord,
    /// M-e (Linux `emacs` flavor Meta seed; no default on Mac/native — see
    /// docs/config.md): to the start of the following sentence, UAX #29
    /// segmentation. See `buffer::sentence`'s module doc for the rule.
    ForwardSentence,
    /// M-a, the mirror of [`Action::ForwardSentence`].
    BackwardSentence,
    BufferStart,
    BufferEnd,
    InsertChar(char),
    Newline,
    /// SHIFT-HELD ACCEPT (`⇧↵`) — the deliberate, footer-taught
    /// "yes, really" (restores a History row; bare `Enter` no longer does).
    /// Resolved in [`KeymapState::resolve_named`], never a catalog chord. IN
    /// THE EDITOR it rides the exact same arm as [`Action::Newline`] — see
    /// `actions::tests::alternate_accept`.
    AcceptAlternate,
    InsertTab,
    Outdent,
    /// ⌥↑ (Option-Up): swap the caret's LOGICAL line — or every line an active
    /// selection touches, moved as one block — with the line immediately
    /// above, caret and selection riding the moved text (columns preserved).
    /// A LOGICAL-line move: wrapped visual rows never reorder independently,
    /// since the buffer has no notion of a visual row at all. Applied as ONE
    /// atomic replace (never coalesces — see `Buffer::apply_edit`'s
    /// `record_edit`), so a single undo restores the pre-move order even
    /// right after a run of typing, while two separate moves stay two undo
    /// steps. A calm no-op at the first line (nothing above to swap with; no
    /// version bump, nothing to undo). Inside a table this is a plain source
    /// edit — the landed row-leave re-pad
    /// (`crate::actions::auto_align_table_on_row_leave`) re-aligns the grid
    /// afterward exactly like any other row-changing action, with no
    /// special-casing here. A numbered list's literal ordinals move WITH
    /// their lines and are not renumbered by this action (mirroring
    /// Tab/Shift-Tab's own no-renumber precedent); re-invoking "Numbered
    /// list" resequences them, same as it does for any other reordering.
    /// See `Buffer::move_line_up`.
    MoveLineUp,
    /// The downward mirror of [`Action::MoveLineUp`]: swaps with the line
    /// immediately below; a calm no-op at the last line. See
    /// `Buffer::move_line_down`.
    MoveLineDown,
    DeleteBackward,
    DeleteWordBackward,
    DeleteWordForward,
    /// M-k (Linux `emacs` flavor Meta seed): kill to the start of the
    /// following sentence — see `buffer::sentence`'s module doc.
    DeleteSentenceForward,
    /// The mirror of [`Action::DeleteSentenceForward`]; no default chord
    /// (palette + `[keys]`, the word-delete precedent).
    DeleteSentenceBackward,
    DeleteToLineStart,
    DeleteForward,
    KillLine,
    Yank,
    YankText,
    InsertImageReference(String),
    /// Undo the last edit group (Cmd+Z / C-/).
    Undo,
    Redo,
    SetMark,
    CopyRegion,
    KillRegion,
    SelectAll,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    PageScrollDown,
    PageScrollUp,
    Save,
    /// Write a snapshot of the current document to a separately chosen path,
    /// without adopting that path as this buffer's identity.
    SaveCopy,
    /// Move the current named document to the operating system's recoverable
    /// Trash. The live document owner performs the external-change and
    /// working-set gates before it asks the platform backend to move anything.
    TrashFile,
    Quit,
    SearchForward,
    SearchBackward,
    /// Cmd-R (headline) / Cmd-Option-F (legacy): summon the find-and-replace panel
    /// — the SAME panel as isearch, with the labeled REPLACE row revealed (a MODE of
    /// the one panel, no separate chrome). A fresh open shows the replace row but
    /// keeps focus on the FIND field; pressing Cmd-R again while the panel is open
    /// jumps focus into the replacement (consumed by the shared search-key seam,
    /// `crate::search::keys::intercept`, on both drivers). Tab switches fields.
    OpenReplace,
    Cancel,
    OpenThemeMenu,
    OpenCommandPalette,
    OpenOutline,
    OpenSpellSuggest,
    ToggleCaretMode,
    OpenCaretMenu,
    OpenDictionaryMenu,
    /// Palette "Keymap…" (hidden on `Convention::Mac`, see
    /// `commands::row_hidden` — the flavor is structurally inert there):
    /// summon the native/emacs keymap-flavor picker, mirroring
    /// `OpenCaretMenu`'s shape.
    OpenKeymapMenu,
    /// Cmd-P → "Toggle spellcheck": flip the GLOBAL spell-check on/off (default
    /// ON — the escape hatch for no-squiggles-ever people). OFF silences EVERY
    /// squiggle (prose comments and code strings alike, per `spell.rs`'s ONE
    /// owner gate) and turns `Cmd-;` / a right-click into a calm no-op. A real
    /// `Action` (not the `writing_nits` sentinel hack) so it round-trips through
    /// `RunAction` unambiguously; render-only (no buffer change), sticky
    /// (persisted like `writing_nits`). No default chord (palette-summoned);
    /// rebindable via `[keys]`. See `spell.rs`.
    ToggleSpellcheck,
    TogglePageMode,
    PageWider,
    PageNarrower,
    /// RESET PAGE WIDTH — snap the measure back to the ACTIVE buffer's OWN built-in
    /// default (see [`crate::page::PageClass::default_measure`] — 70 prose / 100
    /// code) and CLEAR the sticky `page_width_prose`/`page_width_code` config
    /// override matching that SAME class entirely (back to `None`, which already
    /// means "use the built-in default"), so a future default change flows through
    /// instead of pinning a stale value. The "there's no easy way back" fix for
    /// [`PageWider`]/[`PageNarrower`]. No default chord — reachable via the palette
    /// ("Reset page width") and a DOUBLE-CLICK on the draggable page edge
    /// (pointing-not-buttons); rebindable via `[keys]`. Render-only (re-wraps).
    PageReset,
    ToggleDebug,
    ToggleOutline,
    /// "Fold section" (Cmd-Shift-E / C-c C-f): TOGGLE the collapse of the markdown SECTION
    /// enclosing the caret — the heading plus its body + nested subheadings hide to a
    /// quiet one-line summary; toggling again re-expands. VIEW state only (the rope is
    /// untouched, never on the undo timeline); a no-op off a heading / on a
    /// non-markdown buffer. See `fold.rs` + `buffer::toggle_fold_at_cursor`.
    ToggleFold,
    CollapseOtherSections,
    ToggleMenuBar,
    ToggleTypewriter,
    ToggleWritingNits,
    ShowStatsHud,
    About,
    LifetimeStats,
    WritingStreaks,
    /// Palette "Line endings…": TOGGLE the active buffer's line-ending
    /// discipline (`LF`↔`CRLF`, [`crate::buffer::Eol`]) — the rope is byte-identical
    /// either way (always pure `\n`); only the ON-DISK encoding a save restores
    /// differs. Document-level metadata, NOT an undoable edit (Cmd-Z does not
    /// restore it, mirroring VS Code); it marks the buffer dirty + bumps `version`
    /// so autosave rewrites with the new ending. No default chord — the palette IS
    /// its entry point, like Settings/About. See `buffer.rs`'s `set_eol`.
    ConvertLineEndings,
    /// Palette "Align table": RE-PAD the GFM markdown table under the caret so its
    /// `|` line up (Prettier-style monospace alignment of the SOURCE — awl never
    /// draws a grid). Finds the table block around the caret, replaces it via
    /// [`crate::markdown::align_table`] as ONE undoable edit (Cmd-Z restores the
    /// pre-align source); a calm no-op when the caret is not in a table. No default
    /// chord — the palette IS its entry point (like Settings/About); a real
    /// `Action`, independently rebindable via `[keys]`. See `markdown/`.
    AlignTable,
    /// Palette "Insert row above" / "Insert row below": open one blank source
    /// row in the GFM table under the caret, above or below the caret's own
    /// row, as ONE undoable edit. The header and its separator are one
    /// structural unit — "below" the header means the first BODY row, and
    /// "above" it is refused with a notice, because a GFM table's header IS
    /// its first row. Palette-only, like Align table; a real `Action`,
    /// independently rebindable via `[keys]`. See
    /// `crate::markdown::table_splice`.
    TableInsertRowAbove,
    /// Insert-row-below's counterpart — see [`Action::TableInsertRowAbove`].
    TableInsertRowBelow,
    /// Palette "Insert column left" / "Insert column right": splice one blank
    /// cell into EVERY row of the table under the caret (the header and the
    /// alignment separator included) beside the caret's own column, as ONE
    /// undoable edit. Existing columns carry their `:` alignment markers with
    /// them; the new column takes none.
    TableInsertColumnLeft,
    /// Insert-column-left's counterpart — see [`Action::TableInsertColumnLeft`].
    TableInsertColumnRight,
    /// Palette "Delete row": remove the caret's own table row as ONE undoable
    /// edit. Refused with a notice on the header and on its separator (either
    /// one leaves a run of pipes that is no longer a table); emptying the BODY
    /// is fine, since a header + separator alone is valid GFM.
    TableDeleteRow,
    /// Palette "Delete column": remove the caret's own table column — its cell
    /// on every row plus its alignment marker — as ONE undoable edit. Refused
    /// with a notice on a one-column table, which has no column to spare.
    TableDeleteColumn,
    /// Palette "Tag document language": write a `lang:` frontmatter tag naming
    /// the document's detected CJK language, as ONE undoable edit at byte 0 —
    /// the ONLY door in awl that adds that tag. A calm no-op on a non-markdown
    /// buffer, on a document that already carries a frontmatter block, and on
    /// a document with no CJK in it. No default chord — the palette IS its
    /// entry point (like Settings/About/Align table); a real `Action`,
    /// independently rebindable via `[keys]`. See
    /// `crate::actions::edit::tag_document_language`.
    TagDocumentLanguage,
    /// Palette "Report a Problem": compose a `mailto:` link to the maintainer —
    /// subject `"awl problem report (v…)"`, a calm what-happened template body,
    /// and (if a local crash log exists) that log's PATH with a "please attach
    /// this file" line (`mailto:` cannot attach a file; the body never inlines
    /// the log's own content — see the crash-visibility privacy law). The pure
    /// core can't reach the crash-log directory or the OS mail client, so it
    /// signals [`crate::actions::Effect::ReportProblem`] for the live App to
    /// compose (`crashlog::report_problem_mailto`) and open through the SAME
    /// OS-handoff seam `Action::FollowLink` uses (`App::follow_link`). No
    /// document content is ever touched. No default chord (palette-only, like
    /// Settings/About); `native_only: false` — available on the web build too.
    /// Headless replay never opens anything (live-App-only). See `crashlog.rs`.
    ReportProblem,
    /// Palette "Download file" (WEB-ONLY — `web_only: true`, the inverse of
    /// `native_only`): export the ACTIVE buffer's text as a browser download —
    /// `Blob` + object URL + a synthetic `<a download>` click (`web_export.rs`).
    /// A native user already has a real file on real disk (this command is hidden
    /// there entirely — see `commands.rs`'s `web_only` field); on the web build it
    /// is the escape hatch for the no-real-filesystem/no-OS-clipboard sandbox (see
    /// WEB.md). The pure core can't touch `web_sys` (no DOM handoff seam in
    /// `ActionCtx`), so it signals a bare request
    /// ([`crate::actions::Effect::DownloadFile`]) for the live App to perform.
    /// Escapes the SCRATCH buffer too (its virtual `display_name()`). No default
    /// chord (palette-only, like Settings/About); rebindable via `[keys]`.
    /// LIVE-APP-ONLY: headless `--keys` replay never touches the DOM, so it is a
    /// no-op there — a settled capture stays byte-identical. See `web_export.rs`.
    DownloadFile,
    /// Palette "Check for Updates": the app NEVER phones home — this composes
    /// ONE static URL (the site's `/check` page, carrying `CARGO_PKG_VERSION`
    /// as a `?v=` query param — [`crate::updates::check_url`]) and hands it to
    /// the OS browser through the SAME OS-handoff seam `Action::FollowLink` /
    /// `Action::ReportProblem` use (`App::follow_link`); the actual
    /// version-comparison happens in the browser, against a static
    /// `version.json` the site regenerates at deploy — never a fetch from this
    /// binary. The pure core can't reach the fs/OS-handoff, so it signals
    /// [`crate::actions::Effect::CheckForUpdates`] for the live App to (a)
    /// record a LOCAL "last checked" marker (best-effort,
    /// `updates::record_checked`) and (b) open the browser. No document
    /// content is ever touched. No default chord (palette-only, like Report a
    /// Problem/Settings/About); `native_only: true` — the web build updates by
    /// deploy, so the command is meaningless there. Headless replay never
    /// writes the marker or opens anything (live-App-only, mirroring
    /// `ReportProblem`/`FollowLink`). See `updates.rs`.
    CheckForUpdates,
    ToggleBlockquote,
    ToggleBulletList,
    ToggleNumberedList,
    ToggleTaskList,
    ToggleHeading,
    HeadingCycle,
    ToggleCodeBlock,
    Bold,
    Italic,
    InlineCode,
    Highlight,
    Strikethrough,
    /// Palette/menu formatting command: insert a collision-free `[^N]`
    /// reference at the selection end (or caret), append its `[^N]: `
    /// definition, and leave the caret ready to type the note. Markdown-only,
    /// one undoable source edit, no hidden document model.
    InsertFootnote,
    ExportWord,
    ExportHtml,
    ExportPdf,
    OpenGoto,
    OpenProject,
    #[allow(dead_code)] // retained as a replay compatibility action; no public door constructs it.
    OpenRecentProjects,
    OpenBrowse,
    OpenFolder,
    LastBuffer,
    NewDocument,
    /// Summon the persistent scratch surface as the active document — reading
    /// its stash back exactly like a bare relaunch would, so a closed scratch
    /// is reachable again without restarting. The one in-session door back
    /// after `Action::FinishBuffer` / a stack-row close discards it.
    OpenScratch,
    KeepTutorial,
    MoveFile,
    OpenRenameNote,
    DuplicateNote,
    /// Point the platform's own file viewer at the active document's file —
    /// the generalized [`crate::mac_chrome::reveal_in_file_viewer`] handoff
    /// export already used, reached here for any named document. A path-less
    /// scratch buffer has nowhere to reveal: the pure core resolves this to
    /// `Effect::None` rather than signaling a handoff with no target.
    RevealInFileManager,
    /// Copy the active document's absolute native path onto the kill ring —
    /// the same shape as [`Action::CopyLinkDestination`] (set-kill, then the
    /// ordinary `WriteKillRing` effect mirrors it to the OS clipboard). A
    /// no-op for a path-less scratch buffer.
    CopyFilePath,
    #[allow(dead_code)] // next-phase: fired by the settings menu's "Edit config as text" row.
    OpenSettings,
    OpenSettingsMenu,
    OpenKeybindings,
    /// Palette "Credits": summon the read-only CREDITS VIEWER
    /// (`OverlayKind::Credits`) — never a buffer swap. See `credits.rs`.
    OpenCredits,
    OpenHistory,
    /// THE WRITER'S DIFF (palette "Compare with version…", markdown buffers only):
    /// open the READ-ONLY prose-diff view comparing the CURRENT buffer against a
    /// past version — the marked-up manuscript (`crate::prosediff`: struck deletions,
    /// washed insertions, moves, folds). From the BUFFER (no overlay) it compares
    /// against the most-recent version (a loose file's newest history snapshot, or a
    /// git-managed file's HEAD via `git show`); from the open HISTORY picker it
    /// compares against the HIGHLIGHTED row. Esc returns to the live document exactly
    /// (the buffer is never touched). No default chord (a palette command like
    /// Version history / Settings), rebindable via `[keys] compare_with_version`.
    CompareVersion,
    /// Clean unused assets (summon by name, Cmd-P): open the ASSET CLEANER — a
    /// summoned, transient picker listing the ORPHAN image files under the active
    /// project (an image under an `assets/` directory that no document references, per
    /// [`crate::assets::scan`]). Enter on a row moves that file to the macOS TRASH
    /// (recoverable — never `rm`; the row leaves the list, the picker stays open). No
    /// default chord (a palette command, "Clean unused assets…", like Settings/About),
    /// rebindable via `[keys] clean_unused_assets`. See `overlay/`
    /// (`OverlayKind::Assets`) + `assets.rs`.
    OpenAssetClean,
    /// Search in folder… (summon by name, Cmd-P): open the FULL-TEXT SEARCH
    /// picker over the active folder — type a query, see matching lines
    /// grouped by file (`OverlayKind::SearchFolder`), Enter opens that file at
    /// the match through the same combined door every other "open" effect
    /// resolves through (`Effect::OpenPathAtLine`). Complements Cmd-F/Cmd-R
    /// (in-buffer only) and Go to… (names/headings, never content). No
    /// default chord — ⌘⇧F is already `search_backward` — rebindable via
    /// `[keys] search_in_folder`. See `overlay/` (`OverlayKind::SearchFolder`)
    /// + `search_folder.rs`.
    OpenSearchFolder,
    KeepVersion,
    /// READ the unresolved external change: summon the conflict workspace, whose
    /// three read-only views — Differences, Your version, Version on disk — show
    /// one at a time beside the list naming them. Changes nothing: `Esc` returns
    /// to editing with the conflict still open. Palette-only, and its row is
    /// hidden unless a conflict is open (`app/files/external.rs`), exactly like
    /// the two resolutions it is read before.
    ReviewChange,
    /// Settle an unresolved external change by writing the buffer over the file,
    /// after rechecking the disk. Palette-only, and its row is hidden unless a
    /// conflict is open (`app/files/external.rs`).
    ResolveKeepMine,
    /// Settle it the other way: replace the buffer with the file, as ONE
    /// undoable edit. Palette-only and hidden the same way.
    ResolveTakeTheirs,
    /// FINISH the active buffer (the emacsclient "server-edit" convention; the emacs
    /// `C-x #` default is retired, so it is palette-only now): save it, notify any
    /// daemon `--wait` client waiting on it, and
    /// switch to the previously-open buffer (the same swap [`Action::LastBuffer`]
    /// performs). The core only does the SAVE (identically to [`Action::Save`], so
    /// history/mtime bookkeeping stays on one door); the daemon-notify + buffer-swap
    /// are caller-level (the pure core can't reach the daemon, and headless replay has
    /// none to notify). Also a palette command ("Finish file"), rebindable via
    /// `[keys]`. See `crate::daemon`.
    FinishBuffer,
    /// Cmd-click a markdown link (the advertised mouse affordance), or the emacs
    /// `C-c C-o` chord (the org-mode "open link at point" convention, kept): if the
    /// caret sits inside a markdown link, open that link's URL in the default browser.
    /// The pure core extracts the URL ([`crate::markdown::link_at`]) and signals it
    /// back as [`crate::actions::Effect::FollowLink`]; the live App performs the OS
    /// browser handoff (a user-initiated launch, not an app network fetch — the
    /// zero-network invariant holds). A caret outside every link is a calm no-op.
    /// Headless replay never opens a browser.
    FollowLink,
    CopyLinkDestination,
    InsertLink,
    /// Palette "Insert table…" (markdown only): summon the keyboard-first
    /// DIMENSION PICKER — a small drawn grid `↑/↓`/`←/→` sculpt (rows/columns),
    /// typed digits parse forgivingly (`3x4`/`3 4`), a click on the same drawn
    /// grid picks a cell outright. `↵` inserts a fresh GFM table (header row +
    /// separator + blank body rows) at the caret and lands the caret in the
    /// first header cell; `Esc` cancels. No default chord (like Align table);
    /// rebindable via `[keys]`. See `overlay::TableDimsEdit` +
    /// `markdown::build_table`.
    InsertTable,
    InsertDate,
    BeginPrefix,
    Ignore,
}
impl Action {
    pub fn is_motion(&self) -> bool {
        matches!(
            self,
            Action::ForwardChar
                | Action::BackwardChar
                | Action::NextLine
                | Action::PreviousLine
                | Action::LineStart
                | Action::LineEnd
                | Action::ForwardWord
                | Action::BackwardWord
                | Action::ForwardSentence
                | Action::BackwardSentence
                | Action::BufferStart
                | Action::BufferEnd
        )
    }

    /// True when this action mutates buffer content and records undo history.
    pub fn is_edit(&self) -> bool {
        matches!(
            self,
            Action::InsertChar(_)
                | Action::Newline
                | Action::AcceptAlternate
                | Action::InsertTab
                | Action::Outdent
                | Action::MoveLineUp
                | Action::MoveLineDown
                | Action::DeleteBackward
                | Action::DeleteWordBackward
                | Action::DeleteWordForward
                | Action::DeleteSentenceForward
                | Action::DeleteSentenceBackward
                | Action::DeleteToLineStart
                | Action::DeleteForward
                | Action::KillLine
                | Action::Yank
                | Action::YankText
                | Action::InsertImageReference(_)
                | Action::KillRegion
                | Action::AlignTable
                | Action::TableInsertRowAbove
                | Action::TableInsertRowBelow
                | Action::TableInsertColumnLeft
                | Action::TableInsertColumnRight
                | Action::TableDeleteRow
                | Action::TableDeleteColumn
                | Action::TagDocumentLanguage
                | Action::ToggleBlockquote
                | Action::ToggleBulletList
                | Action::ToggleNumberedList
                | Action::ToggleTaskList
                | Action::ToggleHeading
                | Action::HeadingCycle
                | Action::ToggleCodeBlock
                | Action::Bold
                | Action::Italic
                | Action::InlineCode
                | Action::Highlight
                | Action::Strikethrough
                | Action::InsertFootnote
        )
    }
}
