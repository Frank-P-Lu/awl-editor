//! Move-line-up/down entries in the editing command catalog.

use super::{Action, Command};

pub(super) const MOVE_LINE_UP: Command = Command {
    name: "Move line up",
    action: Action::MoveLineUp,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Swap the caret line — or every line a selection touches — with the line above.",
    ),
};

pub(super) const MOVE_LINE_DOWN: Command = Command {
    name: "Move line down",
    action: Action::MoveLineDown,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Swap the caret line — or every line a selection touches — with the line below.",
    ),
};
