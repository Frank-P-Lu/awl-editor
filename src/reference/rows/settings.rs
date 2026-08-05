//! The settings section's rows: every Settings-overlay row and the
//! `config.toml` key it persists under — the one place a DISPATCH-only key is
//! separated from a key a user can actually write.

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// Every row of the Settings overlay, in the order the overlay shows them, with
/// the `config.toml` key each one persists under.
pub(crate) fn settings() -> Vec<Block> {
    let mut t = Table::new(&["Setting", "Group", "Control", "config.toml key"]);
    for r in crate::settings::SETTINGS {
        t.push(vec![
            Cell::text(r.name),
            Cell::text(r.category),
            Cell::text(control_label(r.kind)),
            Cell::code_or_dash(config_key_of(r.id).unwrap_or("")),
        ]);
    }
    vec![block(None, None, t)]
}

/// What the row's Enter key does. NO WILDCARD: a new `SettingKind` fails to
/// compile here until it says how it is edited.
fn control_label(k: crate::settings::SettingKind) -> &'static str {
    use crate::settings::SettingKind;
    match k {
        SettingKind::Toggle => "On/off",
        SettingKind::Picker => "Opens a picker",
        SettingKind::Value => "Typed value",
        SettingKind::Range => "Numeric rail",
        SettingKind::Path => "Picks a folder",
        SettingKind::Submenu => "Opens a submenu",
        SettingKind::Action => "Runs a command",
    }
}

/// A settings-row key that ROUTES rather than persists. `path_key` doubles as a
/// dispatch key: `App::setting_path_pick` intercepts `project_root` and performs
/// a switch-project instead of writing a pref, and `Config` has had no such field
/// since that key was retired. Documenting it as a `config.toml` key would tell a
/// reader to write a line the loader has never read.
///
/// `super::law::every_settings_row_key_is_a_real_config_key` pins this list from
/// both ends: no other row may name a key `Config` lacks, and an entry here that
/// no longer routes anything is stale.
pub(crate) const SETTINGS_DISPATCH_ONLY_KEYS: &[&str] = &["project_root"];

/// The `config.toml` key a settings row writes, from the three no-wildcard maps
/// that already own that question, minus the dispatch-only keys above. A row
/// with no key persists nothing.
pub(crate) fn config_key_of(id: crate::settings::SettingId) -> Option<&'static str> {
    let key = crate::settings::toggle_key(id)
        .or_else(|| crate::settings::value_key(id))
        .or_else(|| crate::settings::path_key(id))?;
    (!SETTINGS_DISPATCH_ONLY_KEYS.contains(&key)).then_some(key)
}
