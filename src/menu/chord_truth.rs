//! **THE DRAWN MENU'S CHORD COLUMN TELLS THE TRUTH.** The
//! awl-rendered menu bar's secondary column used to read
//! `commands::resolved_native_label_truthful` directly — config-blind by that
//! function's own doc, which expects a caller that knows the user's config to
//! layer it on top. Nothing did: under `keymap = "emacs"` the column printed a
//! Ctrl-letter beside Copy/Cut/Paste/Save/… that the resolver actually
//! dispatches to an emacs binding elsewhere, and a `[keys]` rebind never
//! touched the label at all. `commands::menu_native_label` is the fix — the
//! ONE owner both this file's laws and `menu::item_chord`/`item_chord_for_id`
//! now route through, sharing `commands::native_label_effective` with the
//! palette's own `join_slots_truthful`.
//!
//! Split into its own file (mirroring [`super::ellipsis_law`]) rather than
//! grown inline: `menu.rs` already sits well above the natural 500-line
//! ceiling on a recorded exception, and a self-contained roster-wide law is
//! exactly the shape that exception exists for, not a reason to widen it
//! further.

use super::*;

/// LAW: swept over the WHOLE roster (never a hand-picked command list)
/// crossed with BOTH `keymap` flavors, the drawn menu bar's chord column
/// (`commands::menu_native_label`, what `item_chord`/`item_chord_for_id` call
/// under `Convention::current()`) shows a command's resolved Linux chord if
/// and only if a REAL `crate::keymap::KeymapState` — built under the SAME
/// flavor's composed `Config::effective_linux_keep()` — actually dispatches
/// that exact keystroke to the row's own `Action`. A chord the resolver hands
/// to a DIFFERENT action (or drops as `Ignore`/`BeginPrefix`) must render an
/// EMPTY cell, never that Ctrl-letter beside the wrong command.
///
/// Non-vacuity: `emacs`'s composed keep-list is a strict superset of
/// `native`'s (`Config::effective_linux_keep`'s own emacs-preset union), so
/// this asserts the WIDENED suppression set is non-empty and that every
/// command in it still shows its real chord under the `native` flavor —
/// proving the sweep actually exercises both directions rather than only
/// ever finding the unconditional `C-k` floor. Mutation-proven: reverting
/// `commands::menu_native_label` to call `resolved_native_label_truthful`
/// directly (the pre-fix, config-blind path) fails this law by name on the
/// first emacs-displaced command it reaches (`New document`'s `Ctrl+N`,
/// which the emacs flavor's resolver sends to `NextLine` instead).
#[test]
fn menu_chord_column_agrees_with_real_linux_dispatch_under_both_flavors() {
    let mut suppressed_by_flavor: std::collections::HashMap<&str, Vec<&str>> =
        std::collections::HashMap::new();
    for flavor in ["native", "emacs"] {
        let mut cfg = crate::config::Config::empty();
        cfg.keymap = Some(flavor.to_string());
        let keep = cfg.effective_linux_keep();

        let mut km =
            crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Linux);
        km.apply_linux_keep(&keep);

        let mut suppressed = Vec::new();
        let mut checked = 0usize;
        for section in SECTIONS {
            for r in *section {
                let c = commands::COMMANDS
                    .iter()
                    .find(|cmd| cmd.name == r.command)
                    .unwrap_or_else(|| panic!("menu id {:?} names no catalog command", r.id));
                let native = commands::resolved_native(c, crate::convention::Convention::Linux);
                if native.trim().is_empty() {
                    continue;
                }
                let Ok((key, mods)) = crate::keyspec::parse_chord(&native) else {
                    continue;
                };
                checked += 1;
                let dispatched = km.resolve(&key, &mods);
                let rendered = commands::menu_native_label(
                    c,
                    &[],
                    &keep,
                    crate::convention::Convention::Linux,
                    commands::Platform::Native,
                );
                if dispatched == c.action {
                    assert!(
                        !rendered.is_empty(),
                        "flavor {flavor:?}: {} dispatches {native:?} to its own action \
                         but the menu column rendered an empty cell",
                        c.name
                    );
                } else {
                    assert!(
                        rendered.is_empty(),
                        "flavor {flavor:?}: {} would render {rendered:?} for {native:?}, \
                         but the real resolver dispatches that chord to {dispatched:?} \
                         instead — a false chord beside the wrong command",
                        c.name
                    );
                    suppressed.push(c.name);
                }
            }
        }
        assert!(
            checked > 10,
            "flavor {flavor:?}: fewer than 10 routed commands carry a Linux chord — \
             the sweep is not exercising the roster"
        );
        suppressed_by_flavor.insert(flavor, suppressed);
    }

    let native_set: std::collections::HashSet<&str> =
        suppressed_by_flavor["native"].iter().copied().collect();
    let emacs_only: Vec<&str> = suppressed_by_flavor["emacs"]
        .iter()
        .copied()
        .filter(|name| !native_set.contains(name))
        .collect();
    assert!(
        !emacs_only.is_empty(),
        "the emacs flavor's keep-list must widen suppression over at least one command \
         beyond native's unconditional C-k floor, or this law never exercises the axis \
         it exists to catch"
    );
}

/// LAW: a `[keys]` rebind updates the menu's chord column, not just the
/// palette's. Picks a routed command with a real Linux chord (`New
/// document`, `Cmd-N` -> `Ctrl+N`), overrides it, and requires the rendered
/// label to show the OVERRIDE — never the stale static default — with the
/// config-free default shown beside it as the control.
#[test]
fn menu_chord_column_reflects_a_keys_override() {
    let c = commands::COMMANDS
        .iter()
        .find(|c| c.name == "New document")
        .unwrap();
    let default = commands::menu_native_label(
        c,
        &[],
        &[],
        crate::convention::Convention::Linux,
        commands::Platform::Native,
    );
    assert_eq!(default, "Ctrl+N", "control: the config-free default chord");

    let overrides = vec![("New document".to_string(), vec!["C-y".to_string()])];
    let overridden = commands::menu_native_label(
        c,
        &overrides,
        &[],
        crate::convention::Convention::Linux,
        commands::Platform::Native,
    );
    assert_eq!(
        overridden, "Ctrl+Y",
        "a [keys] rebind must reach the menu's chord column"
    );
}
