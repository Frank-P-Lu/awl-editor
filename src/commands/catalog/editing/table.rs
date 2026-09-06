//! Table-specific entries in the editing command catalog.

use super::{Action, Command};

pub(super) const INSERT_TABLE: Command = Command {
    name: "Insert table…",
    action: Action::InsertTable,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Summon the dimension picker: sculpt rows/columns, then insert a fresh GFM table.",
    ),
};

/// The six STRUCTURAL verbs over the GFM table under the caret. Palette-only
/// (no default chord, like Align table): each is a real `Action`, independently
/// rebindable via `[keys]`. Off a table they refuse out loud — a notice naming
/// what they need — rather than doing nothing quietly.
pub(super) const INSERT_ROW_ABOVE: Command = Command {
    name: "Insert row above",
    action: Action::TableInsertRowAbove,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Open a blank row above the caret's row in the table under the caret."),
};

pub(super) const INSERT_ROW_BELOW: Command = Command {
    name: "Insert row below",
    action: Action::TableInsertRowBelow,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Open a blank row below the caret's row in the table under the caret."),
};

pub(super) const INSERT_COLUMN_LEFT: Command = Command {
    name: "Insert column left",
    action: Action::TableInsertColumnLeft,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Open a blank column left of the caret's column, in every row of the table."),
};

pub(super) const INSERT_COLUMN_RIGHT: Command = Command {
    name: "Insert column right",
    action: Action::TableInsertColumnRight,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Open a blank column right of the caret's column, in every row of the table.",
    ),
};

pub(super) const DELETE_ROW: Command = Command {
    name: "Delete row",
    action: Action::TableDeleteRow,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Remove the caret's own row from the table under the caret."),
};

pub(super) const DELETE_COLUMN: Command = Command {
    name: "Delete column",
    action: Action::TableDeleteColumn,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Remove the caret's own column — its cell on every row — from the table."),
};
