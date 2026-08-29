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
