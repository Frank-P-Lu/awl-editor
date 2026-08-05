//! THE ROW BUILDERS — every fact in the reference, read from the owner that
//! already holds it.
//!
//! Nothing in this file restates a roster. A command's name and chords come
//! from `commands::COMMANDS` (the same catalog the palette draws), a settings
//! row from `settings::SETTINGS`, a config key's default from the range spec /
//! toggle default that the running app itself reads, a world from
//! `theme::THEMES`, a markdown construct's coverage from `MdKind::tag`. What IS
//! authored here is the *presentation* — a column heading, a construct's
//! example line, a type's human name — and each authored list is pinned by a
//! law in `super::law` that fails when the roster it presents grows past it.

use super::Block;
use super::emit::Table;

pub(super) fn block(caption: Option<&str>, note: Option<&str>, table: Table) -> Block {
    Block {
        caption: caption.map(str::to_string),
        note: note.map(str::to_string),
        table,
    }
}

pub(super) mod commands;
pub(super) mod config;
pub(super) mod markdown;
pub(super) mod settings;
pub(super) mod worlds;

pub(super) use commands::{commands, synthetic_name};
pub(super) use config::{CONFIG_NON_KEYS, config, config_default, documented_config_keys};
pub(super) use markdown::{conceal_facts_for, documented_tags, markdown};
pub(super) use settings::{SETTINGS_DISPATCH_ONLY_KEYS, config_key_of, settings};
pub(super) use worlds::worlds;
