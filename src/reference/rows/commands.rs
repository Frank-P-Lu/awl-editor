//! The commands section's rows: every catalog command grouped by the
//! palette's own task taxonomy, plus the two chords the keymap matches outside
//! the catalog.

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// Every catalog command, grouped by the palette's own task taxonomy, with the
/// DEFAULT (config-free) chord it answers to under each convention.
///
/// Both columns are asked for EXPLICITLY (`Convention::Mac` / `Convention::Linux`,
/// `Platform::Native` throughout) rather than read from the running host, so the
/// document a macOS developer generates and the document Linux CI checks are the
/// same bytes. The Linux column's keep-list is `Config::empty()`'s — the default
/// composition — so a command left unbound on a stock Linux install shows an
/// empty cell rather than a chord no such install would honour.
pub(crate) fn commands() -> Vec<Block> {
    let keep = crate::config::Config::empty().effective_linux_keep();
    let mut out = Vec::new();
    for cat in crate::commands::TaskCategory::ALL {
        let mut t = Table::new(&["Command", "What it does", "macOS", "Linux", "Builds"]);
        for c in crate::commands::COMMANDS.iter() {
            if crate::commands::task_category_of(c.name) != Some(cat) {
                continue;
            }
            let mac = crate::commands::join_slots_truthful(
                c,
                crate::convention::Convention::Mac,
                crate::commands::Platform::Native,
                &[],
            );
            let linux = crate::commands::join_slots_truthful(
                c,
                crate::convention::Convention::Linux,
                crate::commands::Platform::Native,
                &keep,
            );
            t.push(vec![
                Cell::text(c.name),
                Cell::text_or_dash(c.description),
                Cell::code_or_dash(&mac),
                Cell::code_or_dash(&linux),
                Cell::text(builds_label(c)),
            ]);
        }
        if !t.is_empty() {
            out.push(block(Some(cat.label()), None, t));
        }
    }
    out.push(synthetic_chords());
    out
}

/// The chord the keymap matches DIRECTLY, outside the catalog: the held stats
/// HUD (a hold-only panel a discrete palette selection cannot dismiss). Read
/// from `keytoken::SYNTHETIC` — the same roster the docs' chord tokens resolve
/// through — so a second one cannot appear undocumented. The command palette
/// used to sit here too; it is a real catalog row now, so it appears in the
/// ordinary per-category tables above instead.
fn synthetic_chords() -> Block {
    let mut t = Table::new(&["Chord for", "macOS", "Linux"]);
    for (slug, _, _) in crate::keytoken::SYNTHETIC {
        let label = |c| {
            crate::keytoken::key_token_label(slug, c, crate::commands::Platform::Native)
                .unwrap_or_default()
        };
        t.push(vec![
            Cell::text(synthetic_name(slug)),
            Cell::code_or_dash(&label(crate::convention::Convention::Mac)),
            Cell::code_or_dash(&label(crate::convention::Convention::Linux)),
        ]);
    }
    block(
        Some("Chords with no command"),
        Some("This one is matched by the keymap directly and cannot be rebound."),
        t,
    )
}

/// The display name for a synthetic slug. `super::law` sweeps
/// `keytoken::SYNTHETIC` and fails by name for a slug with no entry here.
pub(crate) fn synthetic_name(slug: &str) -> &'static str {
    match slug {
        "stats_hud" => "Held stats HUD",
        other => panic!(
            "synthetic chord slug `{other}` has no display name in \
             `reference::rows::synthetic_name` — a chord the keymap matches \
             directly and the reference cannot name"
        ),
    }
}

/// Which builds carry a command. `native_only` and `web_only` are DATA on the
/// catalog row, so this is a readout, not a judgement.
fn builds_label(c: &crate::commands::Command) -> &'static str {
    match (c.native_only, c.web_only) {
        (false, false) => "Native, browser",
        (true, false) => "Native",
        (false, true) => "Browser",
        (true, true) => "None",
    }
}
