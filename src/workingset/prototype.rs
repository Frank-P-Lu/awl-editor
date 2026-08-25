//! The working set's Move-navigator audition door.
//!
//! The stack's own overflow/grouping presentations were auditioned here too,
//! once — that decision is shipped (the real `WorkingSet::stack_rows` and
//! `WorkingSet::expanded_rows` are the product now). What remains is the
//! Move-navigator sub-scope, still undecided and awaiting its own round: a
//! sealed environment door, read only while folding a `--screenshot-app`
//! artifact, that projects one candidate row set onto the existing Move
//! destination card without changing the live action grammar. An ordinary
//! live frame never asks it a question.

/// Capture-only candidate rows for the already-existing Move destination card.
/// The production navigator currently expresses "move here" only in its footer
/// and has no discoverable new-folder row; this makes both alternatives visible
/// for the user-judgment capture without changing the live action grammar.
pub fn prototype_move_rows(existing_folders: &[String]) -> Vec<String> {
    let mut rows = Vec::with_capacity(existing_folders.len() + 2);
    rows.push("Move here".to_string());
    rows.push("New folder…".to_string());
    rows.extend(existing_folders.iter().cloned());
    rows
}

pub fn prototype_move_from_env() -> bool {
    std::env::var("AWL_WORKING_SET_PROTOTYPE_MOVE").as_deref() == Ok("1")
}

#[cfg(test)]
mod tests;
