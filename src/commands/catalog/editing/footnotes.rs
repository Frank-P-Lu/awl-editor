//! Footnote-specific entries in the editing command catalog.

use super::{Action, Command};

pub(super) const INSERT_FOOTNOTE: Command = Command {
    name: "Insert footnote",
    action: Action::InsertFootnote,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Insert a collision-free footnote reference and definition, then type the note.",
    ),
};
