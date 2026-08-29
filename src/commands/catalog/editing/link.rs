//! Link-specific entries in the editing command catalog.

use super::{Action, Command};

pub(super) const INSERT_LINK: Command = Command {
    name: "Insert link…",
    action: Action::InsertLink,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Summon the URL prompt for a markdown link: wrap, edit, or insert a link at the caret.",
    ),
};
