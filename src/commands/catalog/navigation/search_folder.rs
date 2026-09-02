use super::{Action, Command};

pub(super) const SEARCH_FOLDER: Command = Command {
    name: "Search in folder…",
    action: Action::OpenSearchFolder,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Full-text search over every file in the active folder — matching lines grouped by file.",
    ),
};
