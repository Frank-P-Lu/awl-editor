use super::Command;
use crate::keymap::Action;
mod footnotes;
use footnotes::INSERT_FOOTNOTE;
mod table;
use table::{
    DELETE_COLUMN, DELETE_ROW, INSERT_COLUMN_LEFT, INSERT_COLUMN_RIGHT, INSERT_ROW_ABOVE,
    INSERT_ROW_BELOW, INSERT_TABLE,
};
mod link;
use link::INSERT_LINK;
mod move_lines;
mod sentence;
pub(super) static COMMANDS: &[Command] = &[
    Command {
        name: "Blockquote",
        action: Action::ToggleBlockquote,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle a `> ` blockquote prefix on the caret line or each line of the selection.",
        ),
    },
    Command {
        name: "Bullet list",
        action: Action::ToggleBulletList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle a `- ` bullet marker on the caret line or each line of the selection.",
        ),
    },
    Command {
        name: "Numbered list",
        action: Action::ToggleNumberedList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle a numbered-list marker on the line or selection, renumbering sequentially.",
        ),
    },
    Command {
        name: "Task list",
        action: Action::ToggleTaskList,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle a `- [ ] ` task checkbox on the caret line or each line of the selection.",
        ),
    },
    Command {
        name: "Heading",
        action: Action::ToggleHeading,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle a level-1 `# ` heading marker on the caret line."),
    },
    Command {
        name: "Cycle heading",
        action: Action::HeadingCycle,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Cycle the caret line's heading level 1 → 2 → 3 → plain text."),
    },
    Command {
        name: "Code block",
        action: Action::ToggleCodeBlock,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Wrap the caret line or selection in a fenced code block, unwrapping if fenced.",
        ),
    },
    Command {
        name: "Bold",
        action: Action::Bold,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle `**bold**` on the selection or word at the caret."),
    },
    Command {
        name: "Italic",
        action: Action::Italic,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Toggle `*italic*` on the selection or word at the caret."),
    },
    Command {
        name: "Inline code",
        action: Action::InlineCode,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle `` `inline code` `` markup around the selection or the word at the caret.",
        ),
    },
    Command {
        name: "Highlight",
        action: Action::Highlight,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle `==highlight==` markup around the selection or the word at the caret.",
        ),
    },
    Command {
        name: "Strikethrough",
        action: Action::Strikethrough,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Toggle `~~strikethrough~~` markup around the selection or the word at the caret.",
        ),
    },
    INSERT_FOOTNOTE,
    Command {
        name: "Export as Word…",
        action: Action::ExportWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Export as `.docx`; markdown buffers only, folder chosen on native."),
    },
    Command {
        name: "Export as HTML…",
        action: Action::ExportHtml,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Export as `.html`; markdown buffers only, folder chosen on native."),
    },
    Command {
        name: "Export as PDF…",
        action: Action::ExportPdf,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Choose a folder, then export as `.pdf`; markdown buffers only, native builds only.",
        ),
    },
    INSERT_LINK,
    INSERT_TABLE,
    INSERT_ROW_ABOVE,
    INSERT_ROW_BELOW,
    INSERT_COLUMN_LEFT,
    INSERT_COLUMN_RIGHT,
    DELETE_ROW,
    DELETE_COLUMN,
    Command {
        name: "Save",
        action: Action::Save,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Save the buffer to disk."),
    },
    Command {
        name: "Review the change",
        action: Action::ReviewChange,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Show an unresolved change: differences, your version, disk version. Changes nothing.",
        ),
    },
    Command {
        name: "Save your version",
        action: Action::ResolveKeepMine,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Settle an unresolved external change by writing the buffer over the file on disk.",
        ),
    },
    Command {
        name: "Use disk version",
        action: Action::ResolveTakeTheirs,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some(
            "Settle an unresolved change by replacing the buffer with the disk file, as one edit.",
        ),
    },
    Command {
        name: "Quit",
        action: Action::Quit,
        native: "",
        emacs: "",
        native_only: true,
        web_only: false,
        description: Some("Quit the application."),
    },
    Command {
        name: "Search forward",
        action: Action::SearchForward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Open incremental search (prefilled from selection or last query), forward.",
        ),
    },
    Command {
        name: "Search backward",
        action: Action::SearchBackward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Open incremental search (prefilled from selection or last query), backward.",
        ),
    },
    Command {
        name: "Find and replace…",
        action: Action::OpenReplace,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Open the search panel with its replace row revealed."),
    },
    Command {
        name: "Undo",
        action: Action::Undo,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Undo the last edit group."),
    },
    Command {
        name: "Redo",
        action: Action::Redo,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Redo the last undone edit group."),
    },
    Command {
        name: "Copy",
        action: Action::CopyRegion,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Copy the selection to the kill buffer, leaving the text and clearing the mark.",
        ),
    },
    Command {
        name: "Cut",
        action: Action::KillRegion,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Cut the selection into the kill buffer and remove it from the buffer."),
    },
    Command {
        name: "Paste",
        action: Action::Yank,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Insert the OS clipboard's content — an image reference if it holds one, else text.",
        ),
    },
    Command {
        name: "Select all",
        action: Action::SelectAll,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Select the entire buffer."),
    },
    Command {
        name: "Zoom in",
        action: Action::ZoomIn,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Step the editor's zoom level up."),
    },
    Command {
        name: "Zoom out",
        action: Action::ZoomOut,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Step the editor's zoom level down."),
    },
    Command {
        name: "Reset zoom",
        action: Action::ZoomReset,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Reset the editor's zoom level to its default."),
    },
    // MOTION COMMANDS (user-decided 2026-07-10, superseding the original all-motions
    // exclusion — see the module doc): the curated NAVIGATION motions are catalog rows
    // so they show in Cmd-P + the Keybindings rebind menu and are REBINDABLE via
    // `[keys]`. The concrete ask this serves: reclaiming the retired Option-letter
    // word motion — `forward_word = ["M-Right", "M-f"]` / `backward_word =
    // ["M-Left", "M-b"]` — which macOS reserves for typing by DEFAULT (the platform
    // rule that retired the M-letter layer) but a config line may deliberately opt
    // back in. Each row shows its REAL default chord (both slots fire; a config
    // override is ADDITIVE); the emacs slots left empty by that retirement stay
    // empty for the user to fill — never re-shipped. Line start/end keep their
    // surviving bare-control second slots (C-a / C-e), now visible + teachable.
    Command {
        name: "Forward word",
        action: Action::ForwardWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret forward one word."),
    },
    Command {
        name: "Backward word",
        action: Action::BackwardWord,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret backward one word."),
    },
    sentence::SENTENCE_FORWARD,
    sentence::SENTENCE_BACKWARD,
    Command {
        name: "Line start",
        action: Action::LineStart,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Move the caret to the start of the visual line (logical without an oracle).",
        ),
    },
    Command {
        name: "Line end",
        action: Action::LineEnd,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Move the caret to the end of the visual line (logical line without a layout oracle).",
        ),
    },
    Command {
        name: "Document start",
        action: Action::BufferStart,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret to the start of the document."),
    },
    Command {
        name: "Document end",
        action: Action::BufferEnd,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret to the end of the document."),
    },
    Command {
        name: "Forward char",
        action: Action::ForwardChar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret forward one character."),
    },
    Command {
        name: "Backward char",
        action: Action::BackwardChar,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Move the caret backward one character."),
    },
    Command {
        name: "Next line",
        action: Action::NextLine,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Move the caret down one visual line, following soft wraps and a sticky goal column.",
        ),
    },
    Command {
        name: "Previous line",
        action: Action::PreviousLine,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Move the caret up one visual line, following soft wraps and a sticky goal column.",
        ),
    },
    move_lines::MOVE_LINE_UP,
    move_lines::MOVE_LINE_DOWN,
    // WORD-DELETE, the mutating siblings of the word MOTIONS above — catalog rows
    // so `[keys]` can reach them (`delete_word_forward = "M-d"` reclaims the
    // classic emacs kill-word; `delete_word_backward = "M-Backspace"`). Both slots
    // are EMPTY by default: the SURVIVING default chords (⌥⌫ back, ⌥Delete / C-⌫ /
    // C-Delete forward) are dispatched by `keymap::resolve_named`'s static
    // NamedKey arms, NOT a catalog chord — the SAME split the plain char/line
    // motions above use (arrows fire from static arms; the catalog row exists only
    // to show in Cmd-P + be rebindable). The retired M-letter emacs slot stays
    // empty for the user to fill, never re-shipped (the word-motion precedent).
    Command {
        name: "Delete word forward",
        action: Action::DeleteWordForward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Delete the word or punctuation run after the caret; a selection deletes instead.",
        ),
    },
    Command {
        name: "Delete word backward",
        action: Action::DeleteWordBackward,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some(
            "Delete the word or punctuation run before the caret; a selection deletes instead.",
        ),
    },
    sentence::DELETE_SENTENCE_FORWARD,
    sentence::DELETE_SENTENCE_BACKWARD,
    Command {
        name: "Settings…",
        action: Action::OpenSettingsMenu,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the settings picker."),
    },
    Command {
        name: "Keybindings…",
        action: Action::OpenKeybindings,
        native: "",
        emacs: "",
        native_only: false,
        web_only: false,
        description: Some("Summon the keybindings rebind menu."),
    },
];
