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
    DeleteBackward,
    DeleteWordBackward,
    DeleteWordForward,
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
    KeepTutorial,
    MoveFile,
    OpenRenameNote,
    DuplicateNote,
    #[allow(dead_code)] // next-phase: fired by the settings menu's "Edit config as text" row.
    OpenSettings,
    OpenSettingsMenu,
    OpenKeybindings,
    /// Palette "Credits": summon the read-only CREDITS VIEWER
    /// (`OverlayKind::Credits`) — never a buffer swap, unlike
    /// Guide/Reference below. See `credits.rs`.
    OpenCredits,
    OpenGuide,
    /// Palette "Reference": open the embedded `REFERENCE.md` into the buffer,
    /// exactly like Guide — the second bundled document through the same
    /// `App::open_bundled_doc` owner (`app/files/open.rs`). No default
    /// chord (palette-only, like Guide/Settings/About); rebindable via
    /// `[keys]`. See `reference_doc.rs`.
    OpenReference,
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
                | Action::DeleteBackward
                | Action::DeleteWordBackward
                | Action::DeleteWordForward
                | Action::DeleteToLineStart
                | Action::DeleteForward
                | Action::KillLine
                | Action::Yank
                | Action::YankText
                | Action::InsertImageReference(_)
                | Action::KillRegion
                | Action::AlignTable
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
        )
    }
}
