//! THE EDIT MENU'S OWN ACTIONS, as one derived roster.
//!
//! These are the document verbs macOS installs REAL key equivalents for
//! (`native::accelerator_for_id`), and AppKit answers a key equivalent from
//! the main menu BEFORE the key window sees the event — so every one of them
//! can arrive at `App::apply` while a summoned text-entry surface holds the
//! caret, through a door no keymap guard is standing at.
//!
//! Split into its own file rather than added to `menu.rs`, which is at its
//! file-size ceiling.

use super::{RosterItem, roster};
use crate::keymap::Action;

/// Undo, Redo, Cut, Copy, Paste, Select all — as THIS BUILD actually routes
/// them, read off the SHIPPED roster (`menu::roster`) rather than re-listed.
/// The Edit menu's own top-level routed rows are exactly `EDIT_ITEMS`; the
/// Markdown vocabulary sits under it as a `Submenu` and is deliberately not
/// swept here.
///
/// The routing law over the summoned fields takes its verb axis from this, so
/// a seventh Edit row enrols the day it is added rather than the day someone
/// remembers to widen a literal.
#[cfg_attr(not(test), allow(dead_code))]
pub fn edit_menu_actions() -> Vec<Action> {
    roster()
        .into_iter()
        .find(|m| m.title == "Edit")
        .map(|m| {
            m.items
                .iter()
                .filter_map(|item| match item {
                    RosterItem::Routed { id, .. } => super::resolve(id),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The roster above is read off the built menu; this is the DATA it must
/// agree with. Two independent paths to the same answer — a filter over
/// `roster()` and the `EDIT_ITEMS` table it is built from — so a roster
/// filter that silently stopped matching (a row promoted to a submenu, an
/// id renamed) is caught rather than shrinking the sweep.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;
    use crate::menu::EDIT_ITEMS;

    #[test]
    fn the_edit_verb_roster_matches_the_table_it_is_built_from() {
        let from_roster = edit_menu_actions();
        let from_table: Vec<Action> = EDIT_ITEMS
            .iter()
            .filter_map(|r| commands::action_for_name(r.command))
            .collect();
        assert_eq!(
            from_roster, from_table,
            "the Edit menu's shipped rows and the EDIT_ITEMS table disagree"
        );
        assert!(
            from_roster.contains(&Action::SelectAll) && from_roster.len() >= 6,
            "the Edit verb roster lost members: {from_roster:?}"
        );
    }
}
