use super::Command;
use crate::keymap::Action;

pub(super) static COMMANDS: &[Command] = &[
    // COMMAND PALETTE: the catalog's own front door, now catalogued rather than
    // a hand-written resolver arm — so it carries a GUIDE row, a `[keys]`
    // rebind, and a menu item. EMACS SLOT DELIBERATELY EMPTY: a catalog emacs
    // slot fires on Mac too, where Option belongs to accent typing, so no
    // default Meta binding ships here (a future Linux Meta-layer binding seeds
    // through that separate machinery, never this slot).
    Command {
        name: "Command palette…",
        action: Action::OpenCommandPalette,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the command palette, searchable across every catalog command."),
    },
    Command {
        name: "Go to…",
        action: Action::OpenGoto,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Find files, headings, folders, and recent destinations."),
    },
    Command {
        name: "Open file…",
        action: Action::OpenBrowse,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Choose a file with the platform file chooser."),
    },
    Command {
        name: "Open folder…",
        action: Action::OpenFolder,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Choose the active writing folder with the platform folder chooser."),
    },
    Command {
        name: "Spell suggestions…",
        action: Action::OpenSpellSuggest,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon spelling suggestions for the misspelled word at the caret."),
    },
    Command {
        name: "Version history…",
        action: Action::OpenHistory,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Summon the version timeline — git log if tracked, saved snapshots otherwise.",
        ),
    },
    Command {
        name: "Compare with version…",
        action: Action::CompareVersion,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Open the read-only prose diff comparing the current buffer against a past version.",
        ),
    },
    Command {
        name: "Clean unused assets…",
        action: Action::OpenAssetClean,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Summon the list of orphaned image files under the project, for moving to the trash.",
        ),
    },
    Command {
        name: "Keep version…",
        action: Action::KeepVersion,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Prompt for a name, then record the buffer text as a pinned history snapshot under it.",
        ),
    },
    Command {
        name: "Last file",
        action: Action::LastBuffer,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Switch to the previously open file; a no-op with nothing to switch back to.",
        ),
    },
    Command {
        name: "New document",
        action: Action::NewDocument,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Start a new, empty document in the current project folder."),
    },
    Command {
        name: "Keep tutorial…",
        action: Action::KeepTutorial,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Mark the tutorial to be saved once a folder is chosen, opening the project switcher.",
        ),
    },
    Command {
        name: "Move…",
        action: Action::MoveFile,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Summon the destination browser to move the current file to another folder.",
        ),
    },
    Command {
        name: "Rename note…",
        action: Action::OpenRenameNote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Open the rename prompt, seeded with the current file's name."),
    },
    Command {
        name: "Duplicate note",
        action: Action::DuplicateNote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Save a copy of the file beside it, deduplicated, and switch to editing the copy.",
        ),
    },
    Command {
        name: "Save a Copy…",
        action: Action::SaveCopy,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Choose a destination for a snapshot while keeping the current file open and \
             unchanged.",
        ),
    },
    // REVEAL / COPY PATH: the platform-neutral catalog name — the visible
    // label a macOS surface may narrow to "Reveal in Finder" is a display
    // decision made where it is shown (`context_menu::reveal_label`), never
    // baked into this name (see that function's doc for why: the generated
    // GUIDE.md/REFERENCE.md tables regenerate on whichever host's CI job
    // runs them, and a host-dependent name would make those doc-drift laws
    // disagree with themselves between the mac and linux jobs). Native-only:
    // a web build has no real filesystem to reveal or an absolute native
    // path to copy.
    Command {
        name: "Reveal in file manager",
        action: Action::RevealInFileManager,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Show the current document's file in the platform's file manager (Finder on macOS).",
        ),
    },
    Command {
        name: "Copy file path",
        action: Action::CopyFilePath,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some("Copy the current document's absolute file path to the clipboard."),
    },
    // FINISH FILE: the emacsclient "server-edit" convention — save, notify any
    // daemon `--wait` client, and CLOSE the file, removing it from the working
    // set. The emacs `C-x #` default is retired; Cmd-W is its native slot now.
    // It is still non-destructive under stray muscle memory, but for a
    // different reason than it used to be: it no longer merely parks the
    // buffer, it removes it — and what makes that safe is the lossless gate it
    // closes through (`app::files::close`), which saves first and REFUSES
    // outright rather than discard a buffer whose file moved underneath it.
    // Closing the last open file removes nothing, since there is no
    // zero-document state to land in. NATIVE-ONLY: the daemon handoff it
    // notifies has no web analog. See `crate::daemon`. (Action stays
    // `FinishBuffer`.)
    Command {
        name: "Finish file",
        action: Action::FinishBuffer,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some("Save the file, notify any daemon `--wait` client, and close it."),
    },
    Command {
        name: "Follow link",
        action: Action::FollowLink,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Open the caret's markdown link, or jump from a footnote reference to its definition.",
        ),
    },
    Command {
        name: "Copy link destination",
        action: Action::CopyLinkDestination,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Copy the URL of the markdown link under the caret to the kill buffer."),
    },
    Command {
        name: "Switch theme…",
        action: Action::OpenThemeMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the theme (world) picker."),
    },
    Command {
        name: "Caret style…",
        action: Action::OpenCaretMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the caret style picker."),
    },
    Command {
        name: "Dictionary…",
        action: Action::OpenDictionaryMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the spelling dictionary picker."),
    },
    Command {
        // Hidden on `Convention::Mac` (`commands::row_hidden`): native macOS
        // ⌘ bindings double-fire the emacs slot regardless of this flavor, so
        // the picker has nothing to choose there — see `KeymapFlavor`'s doc.
        name: "Keymap…",
        action: Action::OpenKeymapMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the keymap flavor picker (native/emacs)."),
    },
    Command {
        name: "Toggle spellcheck",
        action: Action::ToggleSpellcheck,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Flip spellcheck on or off globally, silencing every squiggle when off."),
    },
    Command {
        name: "Toggle caret style",
        action: Action::ToggleCaretMode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Cycle to the next caret style."),
    },
    Command {
        name: "Toggle page mode",
        action: Action::TogglePageMode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle between the centered writing column and full window width."),
    },
    Command {
        name: "Toggle writing nits",
        action: Action::ToggleWritingNits,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle the writing-nits style underlines on or off."),
    },
    Command {
        name: "Widen page",
        action: Action::PageWider,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Widen the page column by one step."),
    },
    Command {
        name: "Narrow page",
        action: Action::PageNarrower,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Narrow the page column by one step."),
    },
    Command {
        name: "Reset page width",
        action: Action::PageReset,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Reset the page column to the buffer's default width, clearing any override.",
        ),
    },
    Command {
        name: "Toggle debug",
        action: Action::ToggleDebug,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle the debug overlay."),
    },
    Command {
        name: "Toggle outline",
        action: Action::ToggleOutline,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle the heading outline panel."),
    },
    // FOLD SECTION: collapse/expand the markdown section under the caret (view state,
    // never file content). Default Cmd-. / C-c C-f; rebindable via config `[keys]`.
    Command {
        name: "Fold section",
        action: Action::ToggleFold,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle collapse of the section under the caret; view state, not on the undo timeline.",
        ),
    },
    Command {
        name: "Collapse other sections",
        action: Action::CollapseOtherSections,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Collapse every markdown section except the one under the caret."),
    },
    Command {
        name: "Toggle typewriter scroll",
        action: Action::ToggleTypewriter,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle keeping the caret vertically centered as you type."),
    },
    Command {
        name: "Toggle menu bar",
        action: Action::ToggleMenuBar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle the menu bar's visibility."),
    },
    Command {
        name: "About",
        action: Action::About,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Show the About panel."),
    },
    Command {
        name: "Credits",
        action: Action::OpenCredits,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Open the bundled Credits document in a read-only viewer."),
    },
    Command {
        name: "Lifetime stats",
        action: Action::LifetimeStats,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some("Open the lifetime writing statistics panel."),
    },
    Command {
        name: "Writing streaks",
        action: Action::WritingStreaks,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some("Open the writing-streaks panel (per-day heatmap and cumulative total)."),
    },
    // LINE ENDINGS: toggle the active file's on-disk ending (LF <-> CRLF). No default
    // chord — the palette IS its entry point (a rare command, like Settings/About); a
    // real `Action` (`ConvertLineEndings`), independently rebindable via `[keys]`.
    Command {
        name: "Line endings…",
        action: Action::ConvertLineEndings,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle the file's on-disk line ending between LF and CRLF; not on the undo timeline.",
        ),
    },
    // ALIGN TABLE: re-pad the GFM table under the caret so its `|` line up (source
    // alignment, never a drawn grid). No default chord — the palette IS its entry
    // point (like Settings/About); a real `Action`, independently rebindable.
    Command {
        name: "Align table",
        action: Action::AlignTable,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Re-pad the GFM table under the caret so its `|` columns line up."),
    },
    // TAG DOCUMENT LANGUAGE: the one door that writes a `lang:` frontmatter tag,
    // and it opens only when the user asks. Opening a document never edits it, so
    // the detection that used to stamp on open is now this explicit command. No
    // default chord — the palette IS its entry point (like Align table).
    Command {
        name: "Tag document language",
        action: Action::TagDocumentLanguage,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Add a `lang:` frontmatter tag naming this document's detected CJK language.",
        ),
    },
    Command {
        name: "Insert Date",
        action: Action::InsertDate,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Insert today's date at the caret, in the configured date format."),
    },
];
