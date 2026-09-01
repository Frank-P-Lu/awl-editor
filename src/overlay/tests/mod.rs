//! Tests for the `overlay` module (pickers, navigators, capture/value-edit
//! sub-states, faceting, elision) -- split by SUBJECT out of one 3433-line
//! `overlay::tests` file into this `overlay/tests/` directory -- every
//! test's NAME is unchanged, only its module path grew one segment
//! (`overlay::tests::foo` -> `overlay::tests::<subject>::foo`).
//! `use super::*;` here still resolves to the `overlay` root exactly as
//! before the split; each child module re-derives overlay access directly
//! via its own `use super::super::*;` plus `use super::*;` for whichever of
//! this module's shared test helpers (`corpus`, `orphan`, `history_rows`) it
//! actually calls.

use super::*;

fn corpus() -> Vec<String> {
    vec![
        ".env".to_string(),
        "README.md".to_string(),
        "src/lib.rs".to_string(),
        "src/main.rs".to_string(),
    ]
}

fn orphan(rel: &str, size: u64) -> crate::assets::Orphan {
    let (name, parent) = match rel.rsplit_once('/') {
        Some((d, n)) => (n.to_string(), d.to_string()),
        None => (rel.to_string(), String::new()),
    };
    crate::assets::Orphan {
        rel: rel.to_string(),
        name,
        parent,
        size: Some(size),
        abs: std::path::PathBuf::from("/proj").join(rel),
    }
}

/// Three history rows newest-first, exercising both WHICH shapes (a git
/// subject, an edited-heading description) and an empty which.
fn history_rows() -> Vec<crate::history::TimelineRow> {
    let row = |when: &str, which: &str, counts: &str, id: &str| crate::history::TimelineRow {
        when: when.to_string(),
        which: which.to_string(),
        counts: counts.to_string(),
        id: id.to_string(),
        timestamp: id.parse().unwrap_or(0),
        pinned: false,
        name: None,
    };
    vec![
        row("just now", "fix: the engine", "+0 −0", "300"),
        row("2 min ago", "edited \"Two flows\"", "+0 −1", "200"),
        row("1 hr ago", "", "+1 −2", "100"),
    ]
}

mod assets;
mod caret_date_link;
mod command_palette;
mod elision_and_browse;
mod flat_pickers;
mod goto_headings;
mod goto_line;
mod hints;
mod history_picker;
mod hover_keyboard_nav;
mod keybindings_capture;
mod kind_roster_laws;
mod minibuffer_word_motion;
mod move_dest;
mod picker_visibility;
mod project;
mod row_meta_laws;
mod settings_rail;
mod spell;
mod unified_goto;
