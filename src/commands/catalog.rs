use super::{Action, Command};
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

/// Stable configuration identity for renamed catalog rows.
pub fn command_slug(command: &Command) -> String {
    match &command.action {
        Action::OpenGoto => "go_to".to_string(),
        Action::OpenFolder => "open_folder".to_string(),
        _ => super::slug(command.name),
    }
}

pub fn action_for_name(name: &str) -> Option<Action> {
    let want = super::slug(name);
    super::COMMANDS
        .iter()
        .find(|command| command_slug(command) == want || super::slug(command.name) == want)
        .map(|command| command.action.clone())
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
