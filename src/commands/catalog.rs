use super::Command;
use std::sync::LazyLock;

mod editing;
mod navigation;
mod tools;
mod trash;

fn clone_command(command: &Command) -> Command {
    Command {
        name: command.name,
        action: command.action.clone(),
        native: command.native,
        emacs: command.emacs,
        native_only: command.native_only,
        web_only: command.web_only,
        description: command.description,
    }
}

/// The one ordered catalog source. The three slices are intentionally
/// concatenated here so every existing caller keeps the same corpus index and
/// display order; the file boundaries are a size split, never a semantic one.
pub(super) static COMMAND_SEED: LazyLock<Vec<Command>> = LazyLock::new(|| {
    navigation::COMMANDS
        .iter()
        .chain(tools::COMMANDS)
        .chain(editing::COMMANDS)
        .map(clone_command)
        .collect()
});
