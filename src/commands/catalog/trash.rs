use super::Command;
use crate::keymap::Action;

pub(super) const COMMAND: Command = Command {
    name: "Move file to Trash",
    action: Action::TrashFile,
    native: "",
    emacs: "",
    native_only: true,
    web_only: false,
    description: Some(concat!(
        "Move a clean file to the operating system's recoverable Trash. ",
        "Save or resolve any changes first, then invoke this command again.",
    )),
};
