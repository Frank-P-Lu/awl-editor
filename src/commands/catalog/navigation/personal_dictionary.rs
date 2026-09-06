use super::{Action, Command};

/// Lives in its own file for the same reason `search_folder` does: the
/// navigation catalog sits under the file-size ceiling only because its longer
/// entries are siblings rather than inline literals.
pub(super) const PERSONAL_DICTIONARY: Command = Command {
    name: "Personal dictionary…",
    action: Action::OpenUserWords,
    native: "",
    emacs: "",
    // Works on web too: with no config directory the word list is the session's
    // own, and forgetting a word still un-silences it.
    native_only: false,
    web_only: false,
    description: Some(
        "List the words you have added to spell-check; accept a row to forget that word.",
    ),
};
