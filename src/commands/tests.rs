use super::*;

#[test]
fn every_defaults_toml_slug_names_a_real_catalog_command() {
    for slug_in_file in crate::keymap_defaults::command_defaults().keys() {
        assert!(
            COMMAND_SEED.iter().any(|c| &slug(c.name) == slug_in_file),
            "assets/keymap-defaults.toml names {slug_in_file:?}, which is not a commands::COMMAND_SEED slug"
        );
    }
}

#[test]
fn every_catalog_command_appears_in_the_defaults_toml_or_is_unbound() {
    let defaults = crate::keymap_defaults::command_defaults();
    for c in COMMAND_SEED.iter() {
        assert!(
            defaults.contains_key(&slug(c.name)),
            "{:?} (slug {:?}) has no entry in assets/keymap-defaults.toml — every catalog \
                 command must appear there, even if unbound (both slots empty)",
            c.name,
            slug(c.name)
        );
    }
}

#[test]
fn defaults_toml_has_no_stale_slugs_and_no_duplicates() {
    let defaults = crate::keymap_defaults::command_defaults();
    assert_eq!(
        defaults.len(),
        COMMAND_SEED.len(),
        "assets/keymap-defaults.toml's entry count must equal the catalog's — an orphaned \
             or duplicated slug would slip past the pure set-membership checks alone"
    );
}

#[test]
fn commands_splices_the_embedded_defaults_verbatim() {
    // THE SINGLE-SOURCE LAW, checked directly: `COMMANDS[i].native`/`.emacs`
    // is EXACTLY what `assets/keymap-defaults.toml` names for that command's
    // slug (never a residual literal from `COMMAND_SEED`, which carries only
    // `""` placeholders in both slots by construction).
    let defaults = crate::keymap_defaults::command_defaults();
    for c in COMMANDS.iter() {
        let (native, emacs) = defaults.get(&slug(c.name)).cloned().unwrap_or_default();
        assert_eq!(
            c.native, native,
            "{:?}'s native slot must come from the embedded defaults",
            c.name
        );
        assert_eq!(
            c.emacs, emacs,
            "{:?}'s emacs slot must come from the embedded defaults",
            c.name
        );
    }
}

#[test]
fn command_seed_itself_carries_no_residual_chord_literals() {
    // A belt-and-suspenders structural check: `COMMAND_SEED`'s own
    // `native`/`emacs` fields (never read by anything but the `COMMANDS`
    // splice above) must stay blank placeholders — a stray literal chord
    // reintroduced there would silently be DISCARDED by the splice (which
    // always overwrites both fields), so this catches the authoring mistake
    // even though it would otherwise have zero runtime effect.
    for c in COMMAND_SEED.iter() {
        assert_eq!(
            c.native, "",
            "{:?}: COMMAND_SEED must not carry a literal native chord",
            c.name
        );
        assert_eq!(
            c.emacs, "",
            "{:?}: COMMAND_SEED must not carry a literal emacs chord",
            c.name
        );
    }
}

#[test]
fn catalog_non_empty_and_named() {
    assert!(
        !COMMANDS.is_empty(),
        "the command catalog must list commands"
    );
    for c in COMMANDS.iter() {
        assert!(!c.name.trim().is_empty(), "command needs a display name");
    }
    const PALETTE_ONLY: &[&str] = &[
        "Keybindings…",
        "Caret style…",
        "Dictionary…",
        "Keymap…",
        "Toggle spellcheck",
        "Toggle writing nits",
        "Reset page width",
        "About",
        "Credits",
        "Lifetime stats",
        "Writing streaks",
        "Line endings…",
        "Align table",
        "Tag document language",
        "Report a Problem",
        "Download file",
        "Check for Updates",
        "Toggle typewriter scroll",
        "Toggle menu bar",
        "Keep tutorial…",
        "Keep version…",
        "Clean unused assets…",
        "Compare with version…",
        "Open file…",
        "Open folder…",
        "Move…",
        "Rename note…",
        "Duplicate note",
        "Save a Copy…",
        "Move file to Trash",
        "Reveal in file manager",
        "Copy file path",
        "Toggle page mode",
        "Toggle caret style",
        "Widen page",
        "Narrow page",
        "Toggle debug",
        "Delete word forward",
        "Delete word backward",
        "Blockquote",
        "Bullet list",
        "Numbered list",
        "Heading",
        "Cycle heading",
        "Code block",
        "Highlight",
        "Strikethrough",
        "Insert footnote",
        "Insert table…",
        "Export as Word…",
        "Export as HTML…",
        "Export as PDF…",
        "Copy link destination",
        "Review the change",
        "Save your version",
        "Use disk version",
    ];
    for c in COMMANDS.iter() {
        if !PALETTE_ONLY.contains(&c.name) {
            assert!(
                !join_slots(c.native, c.emacs).is_empty(),
                "command {} needs at least one binding slot",
                c.name
            );
        }
    }
    assert_eq!(names().len(), COMMANDS.len());
    assert_eq!(bindings().len(), COMMANDS.len());
}

#[test]
fn public_destination_catalog_has_one_goto_and_no_retired_project_wording() {
    let names: Vec<&str> = COMMANDS.iter().map(|command| command.name).collect();
    for required in ["Go to…", "Open file…", "Open folder…"] {
        assert!(
            names.contains(&required),
            "missing public destination door {required}"
        );
    }
    for retired in [
        "Go to file…",
        "Switch project…",
        "Recent projects…",
        "Browse files…",
        "Go to heading…",
    ] {
        assert!(
            !names.contains(&retired),
            "retired public wording survived: {retired}"
        );
    }
    assert_eq!(
        names.iter().filter(|name| **name == "Go to…").count(),
        1,
        "the catalog has one unified typed destination surface"
    );
}

#[test]
fn tag_document_language_is_an_immediate_palette_command() {
    let command = COMMANDS
        .iter()
        .find(|command| command.action == Action::TagDocumentLanguage)
        .expect("TagDocumentLanguage is enrolled in the command catalog");
    assert_eq!(command.name, "Tag document language");
    assert!(
        !command.name.ends_with('…'),
        "this command applies immediately; an ellipsis would falsely promise a sub-picker"
    );
    assert_eq!(command.native, "");
    assert_eq!(command.emacs, "");
}

#[test]
fn every_popover_button_fires_a_catalog_command() {
    for b in crate::popover::PopoverButton::ALL {
        let action = b.action();
        assert!(
            COMMANDS.iter().any(|c| c.action == action),
            "format popover button {b:?} fires {action:?}, which is not a catalog \
                 command — every popover button must route through an existing catalog Action"
        );
    }
}

#[test]
fn command_facets_land_on_all_home_then_offer_every_task_category() {
    assert_eq!(COMMAND_FACETS.strip[0].id, "all");
    assert!(COMMAND_FACETS.strip[0].sections.is_empty());
    let ids: Vec<&str> = COMMAND_FACETS.strip.iter().map(|f| f.id).collect();
    assert_eq!(
        ids,
        vec![
            "all", "files", "navigate", "format", "view", "tools", "settings", "recent"
        ]
    );
}

#[test]
fn menu_section_buckets_known_commands() {
    assert_eq!(menu_section("Save"), Some("File"));
    assert_eq!(menu_section("New document"), Some("File"));
    assert_eq!(menu_section("Export as PDF…"), Some("File"));
    assert_eq!(menu_section("Export as Word…"), Some("File"));
    assert_eq!(menu_section("Export as HTML…"), Some("File"));
    assert_eq!(menu_section("Copy"), Some("Edit"));
    assert_eq!(menu_section("Select all"), Some("Edit"));
    assert_eq!(menu_section("Switch theme…"), Some("View"));
    assert_eq!(menu_section("Toggle debug"), Some("View"));
    assert_eq!(menu_section("Quit"), None);
    assert_eq!(menu_section("About"), None);
    assert_eq!(menu_section("Settings"), None);
    for name in FILE_COMMANDS
        .iter()
        .chain(EDIT_COMMANDS)
        .chain(VIEW_COMMANDS)
    {
        assert!(
            COMMANDS.iter().any(|c| &c.name == name),
            "menu-section name {name:?} is not a catalog command"
        );
    }
}

#[test]
fn command_bucket_routes_each_lens() {
    assert_eq!(command_bucket(FacetItem::new("Save"), 1), Some("Files"));
    assert_eq!(command_bucket(FacetItem::new("Copy"), 1), None);
    assert_eq!(
        command_bucket(FacetItem::new("Forward word"), 2),
        Some("Navigate")
    );
    assert_eq!(command_bucket(FacetItem::new("Copy"), 3), Some("Format"));
    assert_eq!(
        command_bucket(FacetItem::new("Switch theme…"), 4),
        Some("View")
    );
    assert_eq!(command_bucket(FacetItem::new("Credits"), 5), Some("Tools"));
    assert_eq!(
        command_bucket(FacetItem::new("Keybindings…"), 6),
        Some("Settings")
    );
    let mut recent = FacetItem::new("Undo");
    recent.recent = true;
    assert_eq!(command_bucket(recent, 7), Some("Recent"));
    assert_eq!(command_bucket(FacetItem::new("Undo"), 7), None); // not flagged
    // The All home (index 0) never groups.
    assert_eq!(command_bucket(FacetItem::new("Save"), 0), None);
}

#[test]
fn recent_mru_records_newest_first_deduped_and_capped() {
    // RECENT is a process-wide global WRITER — take the ONE reentrant guard
    // so a parallel test can't interleave its clear/record/read (the
    // CLAUDE.md flake tripwire: every `cfg(test)` global writer acquires
    // `testlock::serial()`; with one lock there's no order to invert).
    let _l = crate::testlock::serial();
    clear_recent();
    assert!(recent_indices().is_empty(), "fresh process starts empty");
    record_recent(&Action::Undo);
    record_recent(&Action::Redo);
    record_recent(&Action::Undo); // re-run moves it to front, no dup
    let undo = COMMANDS
        .iter()
        .position(|c| c.action == Action::Undo)
        .unwrap();
    let redo = COMMANDS
        .iter()
        .position(|c| c.action == Action::Redo)
        .unwrap();
    assert_eq!(recent_indices(), vec![undo, redo]);
    clear_recent(); // leave no residue for other tests reading the global
}

#[test]
fn action_for_name_matches_label_and_slug() {
    assert_eq!(action_for_name("Switch theme"), Some(Action::OpenThemeMenu));
    assert_eq!(action_for_name("switch_theme"), Some(Action::OpenThemeMenu));
    assert_eq!(action_for_name("go_to"), Some(Action::OpenGoto));
    assert_eq!(action_for_name("settings"), Some(Action::OpenSettingsMenu));
    assert_eq!(action_for_name("Toggle debug"), Some(Action::ToggleDebug));
    assert_eq!(action_for_name("toggle_debug"), Some(Action::ToggleDebug));
    assert_eq!(
        action_for_name("Toggle outline"),
        Some(Action::ToggleOutline)
    );
    assert_eq!(
        action_for_name("toggle_outline"),
        Some(Action::ToggleOutline)
    );
    assert_eq!(
        action_for_name("Toggle spellcheck"),
        Some(Action::ToggleSpellcheck)
    );
    assert_eq!(
        action_for_name("toggle_spellcheck"),
        Some(Action::ToggleSpellcheck)
    );
    // The held stats HUD is NOT a palette command — it is a momentary HOLD-to-peek, so
    // a discrete selection (with no key-release to dismiss it) would leave it stuck on.
    // It is summoned ONLY by the held Option-Cmd-I chord (`keymap.rs`), never
    // the catalog.
    assert_eq!(action_for_name("Stats HUD"), None);
    assert_eq!(action_for_name("stats_hud"), None);
    assert_eq!(action_for_name("nope"), None);
}

/// LAW: the "Notes"/two-desk-flip command is COMPLETELY gone —
/// no catalog row named "Notes", no `Action`/`Effect` variant reachable
/// through the rebinder by that name or the old `notes` slug. A future
/// command literally named "Notes" (unlikely, but the point of a
/// no-wildcard sweep is to never assume) would need a NEW, deliberate
/// entry here — this test is grep-forced, not name-coincidental, since it
/// also asserts the retired slug resolves to nothing.
#[test]
fn notes_project_flip_command_and_slug_are_fully_retired() {
    assert!(
        !COMMANDS.iter().any(|c| c.name == "Notes"),
        "no catalog row is named \"Notes\" (the retired two-desk flip)"
    );
    assert_eq!(action_for_name("Notes"), None);
    assert_eq!(
        action_for_name("notes"),
        None,
        "the retired [keys] rebind slug resolves to nothing"
    );
}

#[test]
fn a_trailing_ellipsis_never_forks_a_config_key() {
    // THE ELLIPSIS GATE: the `…` picker suffix is DISPLAY-ONLY — `slug` strips it,
    // so a command shown as "Switch theme…" keys under exactly `switch_theme`, the
    // SAME key a `[keys]` entry or the menu-routing table derives. This law pins
    // that a `…` can never fork a second config key.
    for c in COMMANDS.iter() {
        let s = slug(c.name);
        assert!(
            !s.contains('…'),
            "{}: slug must not carry the ellipsis: {s:?}",
            c.name
        );
        let bare = c.name.trim_end_matches('…').trim();
        assert_eq!(
            slug(bare),
            s,
            "{}: bare and suffixed forms must slug the same",
            c.name
        );
        assert_eq!(
            action_for_name(c.name),
            Some(c.action.clone()),
            "{}: suffixed rebind",
            c.name
        );
        assert_eq!(
            action_for_name(bare),
            Some(c.action.clone()),
            "{}: bare rebind",
            c.name
        );
    }
    assert_eq!(slug("Switch theme…"), "switch_theme");
    assert_eq!(slug("Switch theme"), "switch_theme");
    assert_eq!(
        action_for_name("switch_theme…"),
        Some(Action::OpenThemeMenu)
    );
}

/// CONVENTION-PARAMETRIC glyph helper for these two tests: glyphify a literal
/// chord SPEC (an override value, taken literally — never Cmd→Ctrl
/// translated, per `effective_binding_for`'s own doc) through the SAME two
/// pure resolvers it calls, for whichever convention is ambient.
fn glyph(spec: &str) -> String {
    match Convention::current() {
        Convention::Mac => crate::keyspec::mac_glyph_chord(spec),
        Convention::Linux => crate::keyspec::linux_glyph_chord(spec),
    }
}

fn label_for(name: &str) -> String {
    let c = COMMANDS.iter().find(|c| c.name == name).unwrap();
    resolved_native_label(c, Convention::current())
}

#[test]
fn effective_bindings_reflect_overrides() {
    // No config: effective == default labels — a MAC-ONLY invariant.
    // `bindings()`/`join_slots` is explicitly documented as "the Mac
    // baseline" (always mac glyphs, never convention-resolved), while
    // `effective_bindings` IS convention-resolved (`Convention::current()`
    // via `effective_binding_for`) — so the two agree only when the ambient
    // convention actually IS Mac; under Linux they correctly diverge (Ctrl
    // word labels vs. the mac-glyph baseline) BY DESIGN.
    if Convention::current() == Convention::Mac {
        assert_eq!(effective_bindings(&[], &[]), bindings());
    }
    let keys = vec![("switch_theme".to_string(), vec!["C-t".to_string()])];
    let eff = effective_bindings(&keys, &[]);
    let i = COMMANDS
        .iter()
        .position(|c| c.name == "Switch theme…")
        .unwrap();
    assert_eq!(eff[i], glyph("C-t"));
    let bad = vec![("switch_theme".to_string(), vec!["C-frobnicate".to_string()])];
    let eff = effective_bindings(&bad, &[]);
    assert_eq!(eff[i], label_for("Switch theme…"));
}

#[test]
fn effective_bindings_show_both_slots() {
    let i = COMMANDS.iter().position(|c| c.name == "Save").unwrap();
    assert_eq!(bindings()[i], "⌘S");
    let z = COMMANDS.iter().position(|c| c.name == "Zoom in").unwrap();
    assert_eq!(bindings()[z], "⌘=");
    let g = COMMANDS.iter().position(|c| c.name == "Go to…").unwrap();
    assert_eq!(bindings()[g], "⌘O");
    let cut = COMMANDS.iter().position(|c| c.name == "Cut").unwrap();
    assert_eq!(bindings()[cut], "⌘X · C-w");
    let s = COMMANDS.iter().position(|c| c.name == "Settings…").unwrap();
    assert_eq!(bindings()[s], "⌘,");
    let keys = vec![(
        "save".to_string(),
        vec!["Cmd-S".to_string(), "C-x C-s".to_string()],
    )];
    assert_eq!(
        effective_bindings(&keys, &[])[i],
        format!("{} · C-x C-s", glyph("Cmd-S"))
    );
    let mixed = vec![(
        "save".to_string(),
        vec!["Cmd-S".to_string(), "C-frobnicate".to_string()],
    )];
    assert_eq!(effective_bindings(&mixed, &[])[i], glyph("Cmd-S"));
}

#[test]
fn settings_command_present() {
    assert!(
        COMMANDS
            .iter()
            .any(|c| c.action == Action::OpenSettingsMenu)
    );
}

#[test]
fn line_endings_command_present_and_rebindable() {
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Line endings…")
        .expect("Line endings… must be in the catalog");
    assert_eq!(c.native, "");
    assert_eq!(c.emacs, "");
    assert_eq!(c.action, Action::ConvertLineEndings);
    assert_eq!(
        action_for_name("Line endings…"),
        Some(Action::ConvertLineEndings)
    );
    assert_eq!(
        action_for_name("line_endings"),
        Some(Action::ConvertLineEndings)
    );
}

/// **THE COMMAND-PALETTE REBIND LAW.** Command palette used to be a hand-written
/// resolver arm in `keymap::resolve`, invisible to `[keys]` entirely — a user
/// rebind was silently ignored (`action_for_name` had no catalog row to find).
/// Now that it is a real catalog command, a `[keys] command_palette = [...]`
/// entry must dispatch through the SAME override machinery every other command
/// uses, on both conventions, and its untouched DEFAULT chord must keep firing
/// alongside the rebind (`apply_overrides`'s documented additive behavior).
///
/// MUTATION TARGET: drop the `Command` literal and its `assets/keymap-
/// defaults.toml` row together (the item's own prescribed mutation) and
/// `action_for_name("command_palette")` returns `None`, so the override below
/// is silently skipped (`config [keys]: unknown action "command_palette";
/// ignored`, `apply_overrides`'s own leniency) and this law's dispatch
/// assertion fails by name — proving the rebind path, not just presence.
#[test]
fn command_palette_command_present_and_rebindable_via_keys_override() {
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Command palette…")
        .expect("Command palette… must be in the catalog");
    assert_eq!(c.native, "Cmd-P");
    assert_eq!(c.emacs, "");
    assert_eq!(c.action, Action::OpenCommandPalette);
    assert_eq!(
        action_for_name("Command palette…"),
        Some(Action::OpenCommandPalette)
    );
    assert_eq!(
        action_for_name("command_palette"),
        Some(Action::OpenCommandPalette)
    );

    // A `[keys]` rebind to a chord neither convention's default table claims,
    // on EACH convention — the override is documented convention-agnostic
    // (taken literally, never Cmd->Ctrl translated).
    let keys = vec![("command_palette".to_string(), vec!["Cmd-Alt-9".to_string()])];
    for convention in [Convention::Mac, Convention::Linux] {
        let mut km = crate::keymap::KeymapState::with_overrides_and_convention(&keys, convention);
        let (key, mods) = crate::keyspec::parse_chord("Cmd-Alt-9").expect("Cmd-Alt-9 parses");
        assert_eq!(
            km.resolve(&key, &mods),
            Action::OpenCommandPalette,
            "the [keys] rebind must dispatch under {convention:?}"
        );
        // Additive: the untouched default chord still fires too — Cmd-P on
        // Mac, its Cmd->Ctrl translation on Linux (`commands::resolved_native`).
        let default_chord = crate::commands::resolved_native(c, convention);
        assert_eq!(
            resolve_chord_under(&default_chord, convention),
            Action::OpenCommandPalette,
            "the default chord {default_chord:?} must keep firing alongside \
             the rebind under {convention:?}"
        );
    }
}

#[test]
fn follow_link_command_present_and_rebindable() {
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Follow link")
        .expect("Follow link must be in the catalog");
    assert_eq!(c.native, "");
    assert_eq!(c.emacs, "C-c C-o");
    assert_eq!(c.action, Action::FollowLink);
    assert_eq!(action_for_name("Follow link"), Some(Action::FollowLink));
    assert_eq!(action_for_name("follow_link"), Some(Action::FollowLink));
    // The default `C-c C-o` chord parses AND resolves to FollowLink through a
    // fresh MAC-convention keymap (the C-c prefix path) — the catalog/keymap
    // agreement sweep relies on this, pinned here explicitly too. Mac-pinned
    // deliberately: under `Convention::Linux`, bare Ctrl-C is displaced to
    // native Copy (`LINUX_DISPLACED_LETTERS` includes 'c'), so the `C-c`
    // prefix never arms there — that displacement is its own contract, see
    // `keymap.rs`'s collision table doc.
    crate::keymap::parse_binding("C-c C-o").unwrap();
    assert_eq!(
        resolve_chord_under("C-c C-o", Convention::Mac),
        Action::FollowLink
    );
}

#[test]
fn report_problem_command_present_and_rebindable() {
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Report a Problem")
        .expect("Report a Problem must be in the catalog");
    assert_eq!(c.native, "");
    assert_eq!(c.emacs, "");
    assert_eq!(c.action, Action::ReportProblem);
    assert!(
        !c.native_only,
        "Report a Problem must be available on the web build too"
    );
    assert_eq!(
        action_for_name("Report a Problem"),
        Some(Action::ReportProblem)
    );
    assert_eq!(
        action_for_name("report_a_problem"),
        Some(Action::ReportProblem)
    );
}

#[test]
fn check_for_updates_command_present_rebindable_and_native_only() {
    // "Check for Updates" is a real palette command (no default chord, like
    // Report a Problem/Settings/About) backed by `Action::CheckForUpdates`,
    // `native_only: true` (the web build updates by deploy, so a "check"
    // command is meaningless there — it must NOT appear in the web view),
    // and independently rebindable via `[keys] check_for_updates`.
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Check for Updates")
        .expect("Check for Updates must be in the catalog");
    assert_eq!(c.native, "");
    assert_eq!(c.emacs, "");
    assert_eq!(c.action, Action::CheckForUpdates);
    assert!(
        c.native_only,
        "Check for Updates must be hidden on the web build"
    );
    assert!(!c.available_on(Platform::Web));
    assert!(c.available_on(Platform::Native));
    assert_eq!(
        action_for_name("Check for Updates"),
        Some(Action::CheckForUpdates)
    );
    assert_eq!(
        action_for_name("check_for_updates"),
        Some(Action::CheckForUpdates)
    );
}

#[test]
fn toggle_writing_nits_command_present_and_rebindable() {
    let c = COMMANDS
        .iter()
        .find(|c| c.name == "Toggle writing nits")
        .expect("the Toggle writing nits command must be in the catalog");
    assert_eq!(c.native, "");
    assert_eq!(c.emacs, "");
    assert_eq!(c.action, Action::ToggleWritingNits);
    assert_eq!(
        action_for_name("Toggle writing nits"),
        Some(Action::ToggleWritingNits)
    );
    assert_eq!(
        action_for_name("toggle_writing_nits"),
        Some(Action::ToggleWritingNits)
    );
}

#[test]
fn clipboard_and_select_all_in_catalog_with_real_bindings() {
    let find = |name: &str| COMMANDS.iter().find(|c| c.name == name).unwrap();
    let copy = find("Copy");
    assert_eq!(copy.action, Action::CopyRegion);
    assert_eq!((copy.native, copy.emacs), ("Cmd-C", ""));
    let cut = find("Cut");
    assert_eq!(cut.action, Action::KillRegion);
    assert_eq!((cut.native, cut.emacs), ("Cmd-X", "C-w"));
    let paste = find("Paste");
    assert_eq!(paste.action, Action::Yank);
    assert_eq!((paste.native, paste.emacs), ("Cmd-V", "C-y"));
    let all = find("Select all");
    assert_eq!(all.action, Action::SelectAll);
    assert_eq!((all.native, all.emacs), ("Cmd-A", ""));
    assert_eq!(action_for_name("copy"), Some(Action::CopyRegion));
    assert_eq!(action_for_name("select_all"), Some(Action::SelectAll));
}

#[test]
fn keybindings_command_present_and_rebindable() {
    assert!(COMMANDS.iter().any(|c| c.action == Action::OpenKeybindings));
    assert_eq!(
        action_for_name("Keybindings"),
        Some(Action::OpenKeybindings)
    );
    assert_eq!(
        action_for_name("keybindings"),
        Some(Action::OpenKeybindings)
    );
}

#[test]
fn version_history_command_present_and_rebindable() {
    assert!(COMMANDS.iter().any(|c| c.action == Action::OpenHistory));
    assert_eq!(
        action_for_name("Version history…"),
        Some(Action::OpenHistory)
    );
    assert_eq!(
        action_for_name("version_history"),
        Some(Action::OpenHistory)
    );
    let cmd = COMMANDS
        .iter()
        .find(|c| c.action == Action::OpenHistory)
        .unwrap();
    assert_eq!(cmd.native, "Cmd-S-h");
}

#[test]
fn keep_version_command_present_named_and_rebindable() {
    assert!(COMMANDS.iter().any(|c| c.action == Action::KeepVersion));
    assert_eq!(action_for_name("Keep version…"), Some(Action::KeepVersion));
    assert_eq!(action_for_name("Keep version"), Some(Action::KeepVersion));
    assert_eq!(action_for_name("keep_version"), Some(Action::KeepVersion));
    let cmd = COMMANDS
        .iter()
        .find(|c| c.action == Action::KeepVersion)
        .unwrap();
    assert_eq!(cmd.native, "", "palette-only — no default chord");
    assert_eq!(cmd.emacs, "");
}

#[test]
fn binding_conflict_finds_canonical_clash() {
    assert_eq!(binding_conflict("C-s", "undo", &[]), Some("Search forward"));
    assert_eq!(
        binding_conflict("Ctrl-s", "undo", &[]),
        Some("Search forward")
    );
    assert_eq!(binding_conflict("C-s", "search_forward", &[]), None);
    assert_eq!(binding_conflict("C-j", "undo", &[]), None);
    let keys = vec![("save".to_string(), vec!["C-j".to_string()])];
    assert_eq!(binding_conflict("C-j", "undo", &keys), Some("Save"));
    // An unparseable spec never conflicts.
    assert_eq!(binding_conflict("C-frobnicate", "undo", &[]), None);
}

#[test]
fn markdown_formatting_commands_are_all_present_named_and_rebindable() {
    let formatting: &[(&str, Action, &str)] = &[
        ("Blockquote", Action::ToggleBlockquote, ""),
        ("Bullet list", Action::ToggleBulletList, ""),
        ("Numbered list", Action::ToggleNumberedList, ""),
        ("Task list", Action::ToggleTaskList, "Cmd-S-l"),
        ("Heading", Action::ToggleHeading, ""),
        ("Code block", Action::ToggleCodeBlock, ""),
        ("Bold", Action::Bold, "Cmd-B"),
        ("Italic", Action::Italic, "Cmd-I"),
        ("Inline code", Action::InlineCode, "Cmd-E"),
        ("Highlight", Action::Highlight, ""),
        ("Strikethrough", Action::Strikethrough, ""),
    ];
    for (name, action, native) in formatting {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.name == *name)
            .unwrap_or_else(|| panic!("formatting command {name:?} missing from catalog"));
        assert_eq!(&cmd.action, action, "{name}: catalog action");
        assert_eq!(cmd.native, *native, "{name}: native chord slot");
        assert_eq!(
            cmd.emacs, "",
            "{name}: emacs slot is left empty for the user"
        );
        assert_eq!(
            action_for_name(name),
            Some(action.clone()),
            "{name}: label rebind"
        );
        assert_eq!(
            action_for_name(&slug(name)),
            Some(action.clone()),
            "{name}: slug rebind"
        );
    }
    assert_eq!(binding_conflict("Cmd-B", "bold", &[]), None);
    assert_eq!(binding_conflict("Cmd-I", "italic", &[]), None);
    assert_eq!(binding_conflict("Cmd-E", "inline_code", &[]), None);
    assert_eq!(binding_conflict("Cmd-S-l", "task_list", &[]), None);
    let eff = effective_bindings(&[], &[]);
    let bold = COMMANDS.iter().position(|c| c.name == "Bold").unwrap();
    let ital = COMMANDS.iter().position(|c| c.name == "Italic").unwrap();
    let code = COMMANDS
        .iter()
        .position(|c| c.name == "Inline code")
        .unwrap();
    let task = COMMANDS.iter().position(|c| c.name == "Task list").unwrap();
    let convention = Convention::current();
    assert_eq!(
        eff[bold],
        resolved_native_label(&COMMANDS[bold], convention)
    );
    assert_eq!(
        eff[ital],
        resolved_native_label(&COMMANDS[ital], convention)
    );
    assert_eq!(
        eff[code],
        resolved_native_label(&COMMANDS[code], convention)
    );
    assert_eq!(
        eff[task],
        resolved_native_label(&COMMANDS[task], convention)
    );
}

#[test]
fn links_v2_command_is_present_named_and_rebindable() {
    let cmd = COMMANDS
        .iter()
        .find(|c| c.name == "Insert link…")
        .expect("Insert link… missing from catalog");
    assert_eq!(cmd.action, Action::InsertLink);
    assert_eq!(cmd.native, "Cmd-K");
    assert_eq!(cmd.emacs, "");
    assert!(!cmd.native_only, "Insert link… is available on web too");
    assert_eq!(action_for_name("Insert link…"), Some(Action::InsertLink));
    assert_eq!(
        action_for_name(&slug("Insert link…")),
        Some(Action::InsertLink)
    );
    assert_eq!(binding_conflict("Cmd-K", "insert_link", &[]), None);
}

fn resolve_chord_under(spec: &str, convention: Convention) -> Action {
    let mut km = crate::keymap::KeymapState::new_with_convention(convention);
    let mut last = Action::Ignore;
    for tok in spec.split_whitespace() {
        let (key, mods) = crate::keyspec::parse_chord(tok)
            .unwrap_or_else(|e| panic!("catalog chord {spec:?} failed to parse: {e}"));
        last = km.resolve(&key, &mods);
    }
    last
}

#[test]
fn catalog_and_keymap_agree_on_every_default_chord() {
    // THE AGREEMENT SWEEP: the catalog's binding labels and the keymap's
    // dispatch are now SEEDED FROM ONE SOURCE (assets/keymap-defaults.toml).
    // On the chord-VALUE axis this loop is therefore a round-trip — it can
    // no longer catch a wrong default chord, because dispatch and
    // expectation read the same parse. What it STILL genuinely verifies is
    // the SEED-TO-DISPATCH ROUND-TRIP (every seeded slot actually reaches
    // `resolve` and fires its command, so `[keys]` can always address it)
    // PLUS the hand-written Linux POLICY layer below (translation, override,
    // displacement, keep) which is NOT seeded from the TOML. The VALUE
    // oracle — "this specific command resolves to this specific chord" — is
    // the checked-in literal snapshots
    // (`keymap::tests::mac_convention_is_byte_identical_to_the_pre_round_table`
    // and `keymap::tests::catalog_chord_snapshot_is_frozen`), NOT this sweep.
    //
    // CONVENTION-PROOF (per-convention, not just whichever is ambient):
    // `c.native` is always stored in MAC-LITERAL form ("Cmd-O") — under
    // `Convention::Linux` the chord that ACTUALLY fires is the one
    // `commands::resolved_native` computes (a translated/overridden Ctrl
    // chord, per `LINUX_NATIVE_OVERRIDE`/`translate_native_for_linux`), so the
    // native half is checked by resolving THAT translated chord under each
    // convention in turn — never the literal mac string against a Linux
    // keymap (which would never fire native_down at all, see
    // `KeymapState::native_down`'s Super-vs-Ctrl split). The emacs half is
    // OS-agnostic text ("C-s") and is checked directly under BOTH
    // conventions, EXCEPT where `keymap::linux_displaces_emacs_default` says
    // Linux's native layer displaces it (`LINUX_DISPLACED_LETTERS`) — that
    // displacement is its own exhaustively law-tested contract
    // (`keymap::tests::linux_collision_table_matches_the_documented_displaced_list`),
    // not something this sweep should re-assert. SYMMETRICALLY, the NATIVE
    // half skips a chord the DEFAULT (config-free) Linux keep-list holds
    // back (`keymap::linux_builtin_keep()` — Insert link's Ctrl-K, which
    // yields to kill-line out of the box; the insert-link-yields round) —
    // (`keymap::tests::out_of_the_box_linux_ctrl_k_is_kill_line_under_both_keymap_flavors`),
    // and the labels never advertise the chord there either
    // (`insert_link_has_no_visible_linux_binding_out_of_the_box_mac_shows_cmd_k`).
    let default_linux_keep = crate::config::Config::empty().effective_linux_keep();
    for c in COMMANDS.iter() {
        for convention in [Convention::Mac, Convention::Linux] {
            if !c.native.trim().is_empty() {
                let resolved = resolved_native(c, convention);
                let kept_back = convention == Convention::Linux
                    && crate::keymap::linux_keeps_chord(&default_linux_keep, &resolved);
                if !resolved.trim().is_empty() && !kept_back {
                    assert!(
                        crate::keymap::parse_binding(&resolved).is_ok(),
                        "{}: {:?}'s resolved native chord {resolved:?} must parse via \
                         parse_binding",
                        c.name,
                        convention
                    );
                    assert_eq!(
                        resolve_chord_under(&resolved, convention),
                        c.action,
                        "{}: {:?}'s resolved native chord {resolved:?} must resolve to the \
                         catalog action",
                        c.name,
                        convention
                    );
                }
            }
            if !c.emacs.trim().is_empty() {
                assert!(
                    crate::keymap::parse_binding(c.emacs).is_ok(),
                    "{}: emacs default {:?} must parse via parse_binding",
                    c.name,
                    c.emacs
                );
                if convention == Convention::Linux
                    && crate::keymap::linux_displaces_emacs_default(c.emacs, &[])
                {
                    continue; // displaced by native on Linux — covered by keymap.rs's own law test.
                }
                assert_eq!(
                    resolve_chord_under(c.emacs, convention),
                    c.action,
                    "{}: {:?}'s emacs default {:?} must resolve to the catalog action",
                    c.name,
                    convention,
                    c.emacs
                );
            }
        }
        assert_eq!(
            action_for_name(&slug(c.name)),
            Some(c.action.clone()),
            "{}: slug round-trip through action_for_name",
            c.name
        );
    }
}

#[test]
fn no_two_catalog_commands_share_a_default_chord() {
    // PAIRWISE default-chord conflicts, compared CANONICALLY through the same
    // `binding_conflict` the rebind menu gates on (so `Cmd-S` == `s-s`
    // spellings clash too). An INTENTIONALLY shared chord would be allow-
    // listed here as a (command, command) pair with a comment explaining the
    // share — today there are NONE, so the list is empty and every default
    // chord belongs to exactly one command.
    const INTENTIONALLY_SHARED: &[(&str, &str)] = &[];
    for c in COMMANDS.iter() {
        for chord in [c.native, c.emacs] {
            if chord.trim().is_empty() {
                continue;
            }
            if let Some(other) = binding_conflict(chord, &slug(c.name), &[]) {
                let allowlisted = INTENTIONALLY_SHARED
                    .iter()
                    .any(|(a, b)| (*a == c.name && *b == other) || (*a == other && *b == c.name));
                assert!(
                    allowlisted,
                    "default chord {chord:?} is bound to BOTH {:?} and {other:?} \
                         (not in the intentional-share allowlist)",
                    c.name
                );
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn slug_for_action_and_has_native_chord_key_the_usage_ledger() {
    assert_eq!(slug_for_action(&Action::OpenGoto).as_deref(), Some("go_to"));
    assert_eq!(
        slug_for_action(&Action::OpenThemeMenu).as_deref(),
        Some("switch_theme")
    );
    assert_eq!(
        slug_for_action(&Action::ForwardChar),
        Some("forward_char".to_string())
    );
    assert_eq!(slug_for_action(&Action::InsertChar('x')), None);
    assert_eq!(slug_for_action(&Action::BeginPrefix), None);
    assert!(has_native_chord("go_to"), "Go to… carries Cmd-O");
    assert!(has_native_chord("save"), "Save carries Cmd-S");
    assert!(
        has_native_chord("settings"),
        "Settings… now carries Cmd-, (P1)"
    );
    assert!(!has_native_chord("open_file"), "Open file… is palette-only");
    assert!(!has_native_chord("about"), "About is palette-only");
    assert!(
        !has_native_chord("reset_page_width"),
        "Reset page width is palette-only"
    );
    assert!(!has_native_chord("no_such_command"), "unknown slug: false");
    assert!(has_native_chord(&slug_for_action(&Action::Save).unwrap()));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn peek_row_resolves_native_chord_and_name_or_none_for_palette_only() {
    assert_eq!(
        peek_row_for_slug("go_to"),
        Some(crate::peek::PeekRow {
            chord: label_for("Go to…"),
            name: "Go to".into()
        })
    );
    assert_eq!(
        peek_row_for_slug("switch_theme"),
        Some(crate::peek::PeekRow {
            chord: label_for("Switch theme…"),
            name: "Switch theme".into()
        })
    );
    // A palette-only command (no native chord to teach) → None, so it never
    // surfaces as a peek/footer row even if slow-door usage ranks it.
    assert_eq!(peek_row_for_slug("about"), None);
    assert_eq!(
        peek_row_for_slug("settings"),
        Some(crate::peek::PeekRow {
            chord: label_for("Settings…"),
            name: "Settings".into()
        })
    );
    assert_eq!(peek_row_for_slug("no_such_command"), None);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn catalog_motions_are_exactly_the_curated_navigation_set() {
    // THE MOTION SPLIT (user-decided 2026-07-10, superseding the original
    // all-motions exclusion; WIDENED by the emacs-hands-on-Linux round to the
    // last four bare-control nav motions — char forward/back, line up/down —
    // so `[keys]` can finally rebind C-f/C-b/C-n/C-p at all). Every motion
    // `Action::is_motion` names is now a catalog row (palette-visible +
    // rebindable); the split that remains is self-insertion, which never
    // enters the catalog. Kept as a NO-WILDCARD-style completeness sweep
    // (rather than deleting it now that the split is "all of them") so a
    // FUTURE motion added to `is_motion` without a matching catalog row still
    // fails this test loudly, exactly like before.
    const NAVIGATION_MOTIONS: &[Action] = &[
        Action::ForwardChar,
        Action::BackwardChar,
        Action::NextLine,
        Action::PreviousLine,
        Action::ForwardWord,
        Action::BackwardWord,
        Action::LineStart,
        Action::LineEnd,
        Action::BufferStart,
        Action::BufferEnd,
    ];
    for c in COMMANDS.iter() {
        if c.action.is_motion() {
            assert!(
                NAVIGATION_MOTIONS.contains(&c.action),
                "{}: a motion outside the curated navigation set entered the catalog",
                c.name
            );
        }
        assert!(
            !matches!(c.action, Action::InsertChar(_)),
            "{} self-inserts; excluded",
            c.name
        );
    }
    for m in NAVIGATION_MOTIONS {
        assert!(
            COMMANDS.iter().any(|c| &c.action == m),
            "curated navigation motion {m:?} missing from the catalog"
        );
    }
    for m in NAVIGATION_MOTIONS {
        assert!(
            m.is_motion(),
            "{m:?} listed as a navigation motion but is_motion() is false"
        );
    }
}

#[test]
fn motion_commands_are_all_present_named_and_rebindable() {
    let motions: &[(&str, Action, &str, &str)] = &[
        ("Forward word", Action::ForwardWord, "M-Right", ""),
        ("Backward word", Action::BackwardWord, "M-Left", ""),
        ("Line start", Action::LineStart, "Cmd-Left", "C-a"),
        ("Line end", Action::LineEnd, "Cmd-Right", "C-e"),
        ("Document start", Action::BufferStart, "Cmd-Up", ""),
        ("Document end", Action::BufferEnd, "Cmd-Down", ""),
    ];
    for (name, action, native, emacs) in motions {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.name == *name)
            .unwrap_or_else(|| panic!("motion command {name:?} missing from catalog"));
        assert_eq!(&cmd.action, action, "{name}: catalog action");
        assert_eq!(cmd.native, *native, "{name}: native chord slot");
        assert_eq!(cmd.emacs, *emacs, "{name}: emacs chord slot");
        assert_eq!(
            action_for_name(name),
            Some(action.clone()),
            "{name}: label rebind"
        );
        assert_eq!(
            action_for_name(&slug(name)),
            Some(action.clone()),
            "{name}: slug rebind"
        );
    }
    for spec in ["M-f", "M-b"] {
        assert!(
            crate::keymap::parse_binding(spec).is_ok(),
            "{spec:?} must parse"
        );
    }
    assert_eq!(binding_conflict("M-f", "forward_word", &[]), None);
    assert_eq!(binding_conflict("M-b", "backward_word", &[]), None);
    let keys = vec![("forward_word".to_string(), vec!["M-f".to_string()])];
    let i = COMMANDS
        .iter()
        .position(|c| c.name == "Forward word")
        .unwrap();
    assert_eq!(effective_bindings(&keys, &[])[i], glyph("M-f"));
}

#[test]
fn word_delete_commands_are_catalog_rows_and_rebindable() {
    let deletes: &[(&str, Action)] = &[
        ("Delete word forward", Action::DeleteWordForward),
        ("Delete word backward", Action::DeleteWordBackward),
    ];
    for (name, action) in deletes {
        let cmd = COMMANDS
            .iter()
            .find(|c| c.name == *name)
            .unwrap_or_else(|| panic!("word-delete command {name:?} missing from catalog"));
        assert_eq!(&cmd.action, action, "{name}: catalog action");
        assert_eq!(cmd.native, "", "{name}: native slot empty by default");
        assert_eq!(cmd.emacs, "", "{name}: emacs slot empty by default");
        assert_eq!(
            action_for_name(name),
            Some(action.clone()),
            "{name}: label rebind"
        );
        assert_eq!(
            action_for_name(&slug(name)),
            Some(action.clone()),
            "{name}: slug rebind"
        );
    }
    assert_eq!(
        action_for_name("delete_word_forward"),
        Some(Action::DeleteWordForward)
    );
    assert_eq!(
        action_for_name("delete_word_backward"),
        Some(Action::DeleteWordBackward)
    );
    assert!(
        crate::keymap::parse_binding("M-d").is_ok(),
        "M-d must parse"
    );
    assert_eq!(binding_conflict("M-d", "delete_word_forward", &[]), None);
    let keys = vec![("delete_word_forward".to_string(), vec!["M-d".to_string()])];
    let i = COMMANDS
        .iter()
        .position(|c| c.name == "Delete word forward")
        .unwrap();
    assert_eq!(effective_bindings(&keys, &[])[i], glyph("M-d"));
}

const HIDE_ON_WEB: &[&str] = &[
    "Quit",
    "Finish file",
    "Version history…",
    "Compare with version…",
    "Keep version…",
    "Review the change",
    "Save your version",
    "Use disk version",
    "Lifetime stats",
    "Writing streaks",
    "Clean unused assets…",
    "Check for Updates",
    "Keep tutorial…",
    "Export as PDF…",
    "Save a Copy…",
    "Move file to Trash",
    "Reveal in file manager",
    "Copy file path",
];

const HIDE_ON_NATIVE: &[&str] = &["Download file"];

#[test]
fn hide_list_is_exactly_the_native_only_commands() {
    let flagged: std::collections::HashSet<&str> = COMMANDS
        .iter()
        .filter(|c| c.native_only)
        .map(|c| c.name)
        .collect();
    let listed: std::collections::HashSet<&str> = HIDE_ON_WEB.iter().copied().collect();
    assert_eq!(
        flagged, listed,
        "native_only flags and the hide list must match exactly"
    );
}

#[test]
fn inverse_hide_list_is_exactly_the_web_only_commands() {
    let flagged: std::collections::HashSet<&str> = COMMANDS
        .iter()
        .filter(|c| c.web_only)
        .map(|c| c.name)
        .collect();
    let listed: std::collections::HashSet<&str> = HIDE_ON_NATIVE.iter().copied().collect();
    assert_eq!(
        flagged, listed,
        "web_only flags and the inverse hide list must match exactly"
    );
}

#[test]
fn no_command_is_flagged_unavailable_on_both_platforms() {
    for c in COMMANDS.iter() {
        assert!(
            !(c.native_only && c.web_only),
            "{}: native_only and web_only can never both be true (available nowhere)",
            c.name
        );
    }
}

#[test]
fn web_only_commands_are_unavailable_on_native_available_on_web() {
    for name in HIDE_ON_NATIVE {
        let c = COMMANDS
            .iter()
            .find(|c| &c.name == name)
            .unwrap_or_else(|| panic!("{name}: missing"));
        assert!(
            !c.available_on(Platform::Native),
            "{name}: must be hidden on native"
        );
        assert!(
            c.available_on(Platform::Web),
            "{name}: must stay available on web"
        );
    }
}

#[test]
fn hide_listed_commands_are_unavailable_on_web_available_on_native() {
    for name in HIDE_ON_WEB {
        let c = COMMANDS
            .iter()
            .find(|c| &c.name == name)
            .unwrap_or_else(|| panic!("{name}: missing"));
        assert!(
            !c.available_on(Platform::Web),
            "{name}: must be hidden on web"
        );
        assert!(
            c.available_on(Platform::Native),
            "{name}: must stay available natively"
        );
    }
}

#[test]
fn every_other_command_is_available_on_both_platforms() {
    for c in COMMANDS.iter() {
        if HIDE_ON_WEB.contains(&c.name) || HIDE_ON_NATIVE.contains(&c.name) {
            continue;
        }
        assert!(
            c.available_on(Platform::Web),
            "{}: unexpectedly hidden on web",
            c.name
        );
        assert!(
            c.available_on(Platform::Native),
            "{}: unexpectedly hidden on native",
            c.name
        );
    }
}

#[test]
fn platform_current_is_native_under_a_native_test_binary() {
    // `cargo test` is never a wasm32 target, so `Platform::current()` reads
    // Native here — the compiled-platform door and the explicit-platform door
    // agree on THIS binary by construction.
    assert_eq!(Platform::current(), Platform::Native);
}

#[test]
fn visible_on_native_drops_exactly_the_inverse_hide_list_and_nothing_else() {
    let native = visible_on(Platform::Native);
    assert_eq!(native.len(), COMMANDS.len() - HIDE_ON_NATIVE.len());
    // Order is otherwise preserved exactly (filtering, never reordering).
    let expected: Vec<&str> = COMMANDS
        .iter()
        .map(|c| c.name)
        .filter(|n| !HIDE_ON_NATIVE.contains(n))
        .collect();
    let actual: Vec<&str> = native.iter().map(|c| c.name).collect();
    assert_eq!(
        actual, expected,
        "native visible() must preserve catalog order exactly"
    );
    for name in HIDE_ON_NATIVE {
        assert!(
            !native.iter().any(|c| &c.name == name),
            "{name}: leaked into the native view"
        );
    }
    assert_eq!(visible().len(), visible_on(Platform::Native).len());
}

#[test]
fn visible_on_web_drops_exactly_the_hide_list_and_nothing_else() {
    let web = visible_on(Platform::Web);
    assert_eq!(web.len(), COMMANDS.len() - HIDE_ON_WEB.len());
    for c in &web {
        assert!(
            !HIDE_ON_WEB.contains(&c.name),
            "{}: should have been hidden on web",
            c.name
        );
    }
    for name in HIDE_ON_WEB {
        assert!(
            !web.iter().any(|c| &c.name == name),
            "{name}: leaked into the web view"
        );
    }
    for name in HIDE_ON_NATIVE {
        assert!(
            web.iter().any(|c| &c.name == name),
            "{name}: missing from the web view"
        );
    }
}

/// INDEX-COHERENCE LAW for the filtered palette/rebind-menu corpus: for every
/// row `i` in `visible()`, `visible_action_of(i)` / `visible_slug_of(i)` /
/// `visible_name_of(i)` all name THAT SAME row's command — never a raw
/// `COMMANDS[i]` (which would silently mis-map once rows are hidden). Checked on
/// both platforms explicitly (`visible_on`), not just the native-compiled
/// `visible()`, so the web-filtered corpus's own index coherence is pinned too.
#[test]
fn visible_corpus_index_coherence_holds_on_both_platforms() {
    for platform in [Platform::Native, Platform::Web] {
        let filtered = visible_on(platform);
        let names: Vec<String> = filtered.iter().map(|c| c.name.to_string()).collect();
        let actions: Vec<Action> = filtered.iter().map(|c| c.action.clone()).collect();
        for (i, (name, action)) in names.iter().zip(actions.iter()).enumerate() {
            let c = filtered[i];
            assert_eq!(
                &c.name.to_string(),
                name,
                "row {i}: name must match its own filtered slot"
            );
            assert_eq!(
                &c.action, action,
                "row {i}: action must match its own filtered slot"
            );
        }
    }
    let corpus = visible();
    for (i, command) in corpus.iter().enumerate() {
        assert_eq!(
            visible_action_of(i),
            command.action,
            "row {i}: visible_action_of drift"
        );
        assert_eq!(
            visible_slug_of(i),
            slug(command.name),
            "row {i}: visible_slug_of drift"
        );
        assert_eq!(
            visible_name_of(i),
            corpus[i].name,
            "row {i}: visible_name_of drift"
        );
    }
}

#[test]
fn visible_names_and_bindings_are_parallel_and_match_visible() {
    let corpus = visible();
    let names = visible_names();
    let binds = visible_effective_bindings(&[], &[]);
    assert_eq!(names.len(), corpus.len());
    assert_eq!(binds.len(), corpus.len());
    for (i, c) in corpus.iter().enumerate() {
        assert_eq!(names[i], c.name);
    }
}

#[test]
fn visible_hidden_mask_gates_finish_buffer_on_the_live_waiter_fact_alone() {
    // Reveal/Copy-path's own `named_file` gate has its own law right below
    // this one; hold it `true` here so this test's roster stays scoped to
    // the waiter/conflict facts it names.
    let always_named = RowGates {
        named_file: true,
        ..Default::default()
    };
    let corpus = visible();
    let idx = corpus
        .iter()
        .position(|c| c.action == Action::FinishBuffer)
        .expect("FinishBuffer is a real catalog row");

    let mask_no_waiter = visible_hidden_mask(always_named);
    assert_eq!(
        mask_no_waiter.len(),
        corpus.len(),
        "mask is parallel to visible()"
    );
    assert!(
        mask_no_waiter[idx],
        "FinishBuffer must be hidden with no waiter"
    );
    // THE WHOLE RUNTIME-GATED SET, by name, under "no live fact is true" — so a
    // row that quietly grows a gate has to be named here rather than absorbed
    // into a count. "Keymap…" is filtered out here (and below): it is ALSO
    // runtime-gated (`row_hidden`'s `OpenKeymapMenu` arm), but on
    // `Convention`, a process-frozen fact this test does not force — the
    // separate assertion right after this one names it explicitly instead of
    // baking one ambient convention's answer into an exact-list literal.
    // "Move file to Trash" joins the list off macOS: its gate reads the
    // compile-frozen host (`row_hidden_on_host`'s `host_os != "macos"` arm,
    // asserted for both hosts by name in
    // `non_macos_hides_trash_from_palette_and_rejects_direct_dispatch`), so the
    // expected list derives from the same frozen fact — the shape the
    // named-file law below already uses.
    let trash_off_macos = (!cfg!(target_os = "macos")).then_some("Move file to Trash");
    let hidden_now = hidden_row_names(&corpus, &mask_no_waiter);
    assert_eq!(
        hidden_now,
        trash_off_macos
            .into_iter()
            .chain([
                "Finish file",
                "Review the change",
                "Save your version",
                "Use disk version"
            ])
            .collect::<Vec<_>>(),
        "exactly these rows are runtime-gated on a RowGates fact, and each on its own live fact"
    );
    let keymap_idx = corpus
        .iter()
        .position(|c| c.name == "Keymap…")
        .expect("Keymap… is a real catalog row");
    assert_eq!(
        mask_no_waiter[keymap_idx],
        crate::convention::Convention::current() != crate::convention::Convention::Linux,
        "Keymap… is runtime-gated on Convention, not a RowGates fact"
    );

    let mask_waiting = visible_hidden_mask(RowGates {
        has_waiter: true,
        ..always_named
    });
    assert!(
        !mask_waiting[idx],
        "FinishBuffer must show while a waiter is active"
    );
    // The waiter fact unmasks the waiter row ALONE — it must not also reveal a
    // row gated on some other fact, which is the drift a single bool invited.
    assert_eq!(
        hidden_row_names(&corpus, &mask_waiting),
        trash_off_macos
            .into_iter()
            .chain(["Review the change", "Save your version", "Use disk version"])
            .collect::<Vec<_>>(),
        "the waiter fact gates the waiter row and nothing else"
    );

    // …and the conflict fact, symmetrically.
    let mask_conflicted = visible_hidden_mask(RowGates {
        change_unresolved: true,
        ..always_named
    });
    assert_eq!(
        hidden_row_names(&corpus, &mask_conflicted),
        trash_off_macos
            .into_iter()
            .chain(["Finish file"])
            .collect::<Vec<_>>(),
        "an open conflict reveals both resolutions and nothing else"
    );
}

/// The catalog names a `visible_hidden_mask` result actually hides, in
/// catalog order — the one reader every `RowGates` fact-isolation law in this
/// file shares, dropping "Keymap…" (runtime-gated on `Convention`, not a
/// `RowGates` fact — see the assertion right after this helper's first call).
fn hidden_row_names<'a>(corpus: &[&'a Command], mask: &[bool]) -> Vec<&'a str> {
    corpus
        .iter()
        .zip(mask)
        .filter(|&(_, &h)| h)
        .map(|(c, _)| c.name)
        .filter(|&name| name != "Keymap…")
        .collect()
}

/// Reveal/Copy-path's own live fact, isolated from the waiter/conflict laws
/// above the same way they are isolated from it: every other `RowGates` fact
/// held at its "nothing live" default while only `named_file` varies.
///
/// MUTATION TARGET: change `row_hidden`'s `RevealInFileManager |
/// Action::CopyFilePath` arm to an unconditional `false` and this fails by
/// name — a scratch document would then offer both rows in the palette.
#[test]
fn visible_hidden_mask_gates_reveal_and_copy_path_on_the_named_file_fact_alone() {
    let corpus = visible();
    let hidden_unnamed = hidden_row_names(&corpus, &visible_hidden_mask(Default::default()));
    assert!(
        hidden_unnamed.contains(&"Reveal in file manager")
            && hidden_unnamed.contains(&"Copy file path"),
        "an unnamed scratch document must hide both rows: {hidden_unnamed:?}"
    );

    let mask_named = visible_hidden_mask(RowGates {
        named_file: true,
        ..Default::default()
    });
    let hidden_named = hidden_row_names(&corpus, &mask_named);
    assert!(
        !hidden_named.contains(&"Reveal in file manager")
            && !hidden_named.contains(&"Copy file path"),
        "a named document must show both rows: {hidden_named:?}"
    );
    assert_eq!(
        hidden_named,
        (!cfg!(target_os = "macos"))
            .then_some("Move file to Trash")
            .into_iter()
            .chain([
                "Finish file",
                "Review the change",
                "Save your version",
                "Use disk version",
            ])
            .collect::<Vec<_>>(),
        "the named-file fact gates Reveal/Copy-path and nothing else"
    );
}

#[test]
fn non_macos_hides_trash_from_palette_and_rejects_direct_dispatch() {
    let gates = RowGates {
        named_file: true,
        ..Default::default()
    };
    assert!(row_hidden_on_host(&Action::TrashFile, gates, "linux"));
    assert!(!action_available_on_host(
        &Action::TrashFile,
        Platform::Native,
        "linux"
    ));
    assert!(!row_hidden_on_host(&Action::TrashFile, gates, "macos"));
    assert!(action_available_on_host(
        &Action::TrashFile,
        Platform::Native,
        "macos"
    ));
}

#[test]
fn action_available_gates_hidden_actions_only_on_web() {
    assert!(!action_available(&Action::Quit, Platform::Web));
    assert!(action_available(&Action::Quit, Platform::Native));
    assert!(!action_available(&Action::FinishBuffer, Platform::Web));
    assert!(action_available(&Action::OpenKeybindings, Platform::Web));
    assert!(action_available(&Action::Save, Platform::Web));
    assert!(action_available(&Action::Save, Platform::Native));
    assert!(action_available(&Action::ForwardChar, Platform::Web));
    assert!(action_available(&Action::InsertChar('x'), Platform::Web));
}

#[test]
fn visible_recent_indices_drops_hidden_catalog_entries_and_translates_the_rest() {
    // RECENT is a process-wide global WRITER — take the ONE reentrant guard
    // (see the sibling test above; the CLAUDE.md flake tripwire).
    let _l = crate::testlock::serial();
    clear_recent();
    record_recent(&Action::Undo);
    record_recent(&Action::Quit); // a hidden-on-web command
    record_recent(&Action::Redo);
    let vis = visible_recent_indices();
    assert_eq!(vis.len(), 3);
    let corpus = visible();
    let redo_row = corpus
        .iter()
        .position(|c| c.action == Action::Redo)
        .unwrap();
    assert_eq!(vis[0], redo_row, "most-recent-first order preserved");
    clear_recent();
}

/// THE HARD LAW: on `Convention::Mac` + `Platform::Native` (a plain macOS
/// native build) neither Tier 2 (web-reserved) nor Tier 3 (Linux-displaced)
/// can ever fire, so [`join_slots_truthful`] must be BYTE-IDENTICAL to the
/// pre-round `join_slots(c.native, c.emacs)` for EVERY catalog command.
#[test]
fn mac_native_label_truth_is_byte_identical_to_join_slots() {
    for c in COMMANDS.iter() {
        assert_eq!(
            join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
            join_slots(c.native, c.emacs),
            "{} diverged from the pre-round Mac-native label",
            c.name
        );
    }
}

#[test]
fn web_reserved_native_chord_shows_its_web_alternate() {
    let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
    let switch_theme = COMMANDS.iter().find(|c| c.name == "Switch theme…").unwrap();
    for c in [new_document, switch_theme] {
        assert_eq!(
            c.emacs.trim(),
            "",
            "{} must have no emacs slot for this test's claim",
            c.name
        );
        for convention in [Convention::Mac, Convention::Linux] {
            let label = resolved_native_label_truthful(c, convention, Platform::Web);
            assert!(
                !label.is_empty(),
                "{}: web alternate must not be blank ({convention:?})",
                c.name
            );
            assert_ne!(
                label,
                resolved_native_label(c, convention),
                "{}: the web label must be the ALTERNATE, not the (reserved) native one",
                c.name
            );
            assert_eq!(
                join_slots_truthful(c, convention, Platform::Web, &[]),
                label
            );
            assert_eq!(
                resolved_native_label_truthful(c, convention, Platform::Native),
                resolved_native_label(c, convention)
            );
        }
    }
}

#[test]
fn web_alternate_labels_are_convention_keyed() {
    let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
    let switch_theme = COMMANDS.iter().find(|c| c.name == "Switch theme…").unwrap();
    assert_eq!(
        resolved_native_label_truthful(new_document, Convention::Mac, Platform::Web),
        "\u{2303}J"
    );
    assert_eq!(
        resolved_native_label_truthful(switch_theme, Convention::Mac, Platform::Web),
        "\u{2303}T"
    );
    assert_eq!(
        resolved_native_label_truthful(new_document, Convention::Linux, Platform::Web),
        "Alt+N"
    );
    assert_eq!(
        resolved_native_label_truthful(switch_theme, Convention::Linux, Platform::Web),
        "Alt+T"
    );
}

#[test]
fn exactly_new_note_and_switch_theme_are_web_reserved_and_available() {
    let mut hit: Vec<&str> = COMMANDS
        .iter()
        .filter(|c| c.available_on(Platform::Web))
        .filter(|c| {
            [Convention::Mac, Convention::Linux]
                .iter()
                .any(|conv| crate::webreserved::is_reserved(&resolved_native(c, *conv), *conv))
        })
        .map(|c| c.name)
        .collect();
    hit.sort_unstable();
    assert_eq!(hit, vec!["New document", "Switch theme…"]);
}

#[test]
fn web_alternate_keys_is_inert_on_native_and_populated_on_web() {
    assert_eq!(
        web_alternate_keys(&[], Convention::Mac, Platform::Native),
        Vec::new()
    );
    let mut on_web = web_alternate_keys(&[], Convention::Mac, Platform::Web);
    on_web.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        on_web,
        vec![
            ("new_document".to_string(), vec!["C-j".to_string()]),
            ("switch_theme".to_string(), vec!["C-t".to_string()])
        ]
    );
    let mut on_web_linux = web_alternate_keys(&[], Convention::Linux, Platform::Web);
    on_web_linux.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        on_web_linux,
        vec![
            ("new_document".to_string(), vec!["M-n".to_string()]),
            ("switch_theme".to_string(), vec!["M-t".to_string()])
        ]
    );
}

/// **Config still trumps everything:** a user `[keys]` entry for "New
/// note" suppresses ITS web alternate entirely (the user's own chosen
/// chord is never shadowed), while "Switch theme…"'s alternate — untouched
/// by the user's config — still appears.
#[test]
fn web_alternate_keys_skips_a_command_the_user_has_already_rebound() {
    let existing = vec![("new_document".to_string(), vec!["C-x C-n".to_string()])];
    let on_web = web_alternate_keys(&existing, Convention::Mac, Platform::Web);
    assert!(
        !on_web.iter().any(|(name, _)| name == "new_document"),
        "user's own new_document rebind must not be shadowed"
    );
    assert!(
        on_web.iter().any(|(name, _)| name == "switch_theme"),
        "switch_theme's alternate is still added"
    );
}

#[test]
fn web_alternate_keys_dispatch_the_real_action_on_web() {
    let keys = web_alternate_keys(&[], Convention::Mac, Platform::Web);
    let mut km = crate::keymap::KeymapState::with_overrides(&keys);
    let (key, mods) = crate::keyspec::parse_chord("C-j").expect("C-j parses");
    assert_eq!(km.resolve(&key, &mods), Action::NewDocument);
    let (key, mods) = crate::keyspec::parse_chord("C-t").expect("C-t parses");
    assert_eq!(km.resolve(&key, &mods), Action::OpenThemeMenu);
}

/// TIER 2, the fallback half: a SYNTHETIC command whose native chord is
/// web-reserved but which ALSO carries a surviving emacs slot falls back
/// to that slot on the web — never a blank label when a truthful door
/// remains.
#[test]
fn web_reserved_native_chord_falls_back_to_a_surviving_emacs_slot() {
    let synthetic = Command {
        name: "Synthetic",
        action: Action::Ignore,
        native: "Cmd-N",
        emacs: "C-k",
        native_only: false,
        web_only: false,
        description: None,
    };
    assert_eq!(
        join_slots_truthful(&synthetic, Convention::Mac, Platform::Web, &[]),
        "C-k"
    );
    assert_eq!(
        join_slots_truthful(&synthetic, Convention::Linux, Platform::Web, &[]),
        "C-k"
    );
    assert_eq!(
        join_slots_truthful(&synthetic, Convention::Mac, Platform::Native, &[]),
        "⌘N · C-k"
    );
}

#[test]
fn linux_web_reserved_uses_the_ctrl_translated_form() {
    let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
    assert_eq!(resolved_native(new_document, Convention::Linux), "C-n");
    assert!(crate::webreserved::is_reserved("C-n", Convention::Linux));
    assert_eq!(
        resolved_native_label_truthful(new_document, Convention::Linux, Platform::Web),
        "Alt+N"
    );
}

/// TIER 3: "Search forward" (native Cmd-F, emacs `C-s`) under
/// `Convention::Linux` — Ctrl-S is claimed by Save, so the emacs slot is
/// displaced and must NOT appear in the joined label, on EITHER platform
/// (the collision is a dispatch-table property, not a web-only one).
#[test]
fn linux_displaced_emacs_default_never_shown_on_either_platform() {
    let search = COMMANDS
        .iter()
        .find(|c| c.name == "Search forward")
        .unwrap();
    for platform in [Platform::Native, Platform::Web] {
        let label = join_slots_truthful(search, Convention::Linux, platform, &[]);
        assert_eq!(
            label, "Ctrl+F",
            "displaced C-s must not appear (platform {platform:?})"
        );
    }
    // Mac convention: the emacs slot is UNCHANGED (Ctrl never reads native
    // there), so the old joined form survives on both platforms.
    assert_eq!(
        join_slots_truthful(search, Convention::Mac, Platform::Native, &[]),
        "⌘F · C-s"
    );
}

/// TIER 3, the prefix-sequence edge case: "Follow link"'s emacs default is
/// the two-key `"C-c C-o"` sequence — Ctrl-C now resolves straight to Copy
/// on Linux, so the WHOLE sequence is displaced (never arms), and Follow
/// link has no native slot either — the joined label goes fully blank.
#[test]
fn linux_displaces_a_prefix_sequence_by_its_first_key() {
    let follow = COMMANDS.iter().find(|c| c.name == "Follow link").unwrap();
    assert_eq!(follow.native.trim(), "");
    assert_eq!(follow.emacs, "C-c C-o");
    assert_eq!(
        join_slots_truthful(follow, Convention::Linux, Platform::Native, &[]),
        ""
    );
    assert_eq!(
        join_slots_truthful(follow, Convention::Mac, Platform::Native, &[]),
        "C-c C-o"
    );
}

/// TIER 3, the non-displaced control: "Undo"'s emacs slot `C-/` is a
/// non-letter chord outside the displaced-letter set entirely, so it
/// survives Linux exactly like Mac.
#[test]
fn non_displaced_emacs_default_survives_linux() {
    let undo = COMMANDS.iter().find(|c| c.name == "Undo").unwrap();
    assert_eq!(
        join_slots_truthful(undo, Convention::Linux, Platform::Native, &[]),
        "Ctrl+Z · C-/"
    );
}

/// THE LABEL-TRUTH LAW, swept over the WHOLE catalog × every (convention,
/// platform) pair: [`resolved_native_label_truthful`] is empty whenever
/// [`crate::webreserved::is_reserved`] says so, and the joined label never
/// contains a Linux-displaced emacs default as one of its `·`-separated
/// tokens. A future collision must be explicitly accounted for.
#[test]
fn label_truth_law_holds_across_the_whole_catalog() {
    for c in COMMANDS.iter() {
        for convention in [Convention::Mac, Convention::Linux] {
            for platform in [Platform::Native, Platform::Web] {
                let native_resolved = resolved_native(c, convention);
                let reserved = platform == Platform::Web
                    && crate::webreserved::is_reserved(&native_resolved, convention);
                if reserved {
                    let label = resolved_native_label_truthful(c, convention, platform);
                    let native_label = resolved_native_label(c, convention);
                    assert_ne!(
                        label, native_label,
                        "{}: reserved native chord {native_resolved:?} still shown verbatim \
                         ({convention:?}/{platform:?})",
                        c.name
                    );
                    // Either a web alternate (non-blank) or blank (no alternate defined) — but
                    // never the reserved native chord itself.
                    if let Some(alt) = chords::web_alternate_for(c, convention) {
                        let expect = match convention {
                            Convention::Mac => crate::keyspec::mac_glyph_chord(alt),
                            Convention::Linux => crate::keyspec::linux_glyph_chord(alt),
                        };
                        assert_eq!(
                            label, expect,
                            "{}: web alternate label mismatch ({convention:?}/{platform:?})",
                            c.name
                        );
                    } else {
                        assert_eq!(
                            label, "",
                            "{}: no alternate defined, label should be blank \
                             ({convention:?}/{platform:?})",
                            c.name
                        );
                    }
                }
                let displaced = convention == Convention::Linux
                    && crate::keymap::linux_displaces_emacs_default(c.emacs, &[]);
                if displaced {
                    let label = join_slots_truthful(c, convention, platform, &[]);
                    assert!(
                        !label.split(" · ").any(|tok| tok == c.emacs),
                        "{}: displaced emacs default {:?} still shown \
                         ({convention:?}/{platform:?}) — label was {label:?}",
                        c.name,
                        c.emacs
                    );
                }
            }
        }
    }
}

/// TIER 4 (emacs-hands-on-Linux): "Forward char" (no native slot, emacs
/// `C-f`) is normally Linux-DISPLACED by "Search forward"'s native Ctrl-F.
/// A `linux_keep_emacs = ["C-f"]` config UN-displaces it (its emacs label
/// reappears) AND suppresses "Search forward"'s own native label for that
/// SAME chord — the two-sided fix, checked on both commands at once so
/// they can never drift apart.
#[test]
fn linux_keep_emacs_restores_the_emacs_label_and_suppresses_the_native_one() {
    let keep = vec!["C-f".to_string()];
    let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
    let search = COMMANDS
        .iter()
        .find(|c| c.name == "Search forward")
        .unwrap();

    assert_eq!(
        join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &[]),
        ""
    );
    assert_eq!(
        join_slots_truthful(search, Convention::Linux, Platform::Native, &[]),
        "Ctrl+F"
    );

    // WITH the keep-list: Forward char shows its kept emacs chord; Search
    // forward's native Ctrl+F vanishes (it no longer actually fires there),
    // leaving only Search forward's OWN un-displaced... wait, C-s IS still
    // displaced by Save's native Ctrl-S (unrelated to this keep entry), so
    // Search forward's label goes fully blank — it has NO chord that fires
    // on Linux once C-f is given back to Forward char.
    assert_eq!(
        join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &keep),
        "C-f"
    );
    assert_eq!(
        join_slots_truthful(search, Convention::Linux, Platform::Native, &keep),
        ""
    );

    assert_eq!(
        join_slots_truthful(forward_char, Convention::Mac, Platform::Native, &keep),
        join_slots_truthful(forward_char, Convention::Mac, Platform::Native, &[]),
    );
    assert_eq!(
        join_slots_truthful(search, Convention::Mac, Platform::Native, &keep),
        join_slots_truthful(search, Convention::Mac, Platform::Native, &[]),
    );
}

#[test]
fn linux_keep_emacs_is_a_per_chord_door_not_a_policy_flip() {
    let keep = vec!["C-f".to_string()];
    let next_line = COMMANDS.iter().find(|c| c.name == "Next line").unwrap();
    assert_eq!(
        join_slots_truthful(next_line, Convention::Linux, Platform::Native, &keep),
        ""
    );
    let new_document = COMMANDS.iter().find(|c| c.name == "New document").unwrap();
    assert_eq!(
        join_slots_truthful(new_document, Convention::Linux, Platform::Native, &keep),
        "Ctrl+N"
    );
}

/// `effective_bindings`/`visible_effective_bindings` (the palette/rebind-menu
/// doors) thread the keep-list all the way through — not just the pure
/// `join_slots_truthful` unit.
#[test]
fn effective_bindings_reflects_the_linux_keep_emacs_list() {
    if Convention::current() != Convention::Linux {
        return;
    }
    let keep = vec!["C-f".to_string()];
    let i = COMMANDS
        .iter()
        .position(|c| c.name == "Forward char")
        .unwrap();
    assert_eq!(effective_bindings(&[], &[])[i], "");
    assert_eq!(effective_bindings(&[], &keep)[i], "C-f");
}

#[test]
fn linux_keep_emacs_is_inert_on_mac_for_the_whole_catalog() {
    let keep = vec![
        "C-f".to_string(),
        "C-b".to_string(),
        "C-n".to_string(),
        "C-p".to_string(),
    ];
    for c in COMMANDS.iter() {
        assert_eq!(
            join_slots_truthful(c, Convention::Mac, Platform::Native, &keep),
            join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
            "{}: linux_keep_emacs must be inert on Mac",
            c.name
        );
    }
}

/// TIER 4, WHOLE-PRESET FLAVOR: the same two-sided label fix
/// [`linux_keep_emacs_restores_the_emacs_label_and_suppresses_the_native_one`]
/// exercises for a hand-picked `["C-f"]`, now exercised for the FULL emacs
/// flavor preset (`keymap::linux_emacs_preset_keep`) — "Forward char" gets
/// its emacs `C-f` label back and "Search forward" loses the native
/// `Ctrl+F` claim it would otherwise show. UNLIKE the hand-picked case,
/// "Search forward" does NOT go blank here: its OWN emacs default (`C-s`)
/// is ALSO in the whole preset (the letter `s` is displaced too, by
/// Save's native Ctrl-S), so Save's native claim is suppressed right back
/// and Search forward's bare `C-s` reappears — the whole-preset's actual
/// shape, every displaced letter reverting to its own emacs owner at once.
#[test]
fn keymap_flavor_emacs_preset_restores_labels_two_sided() {
    let preset = crate::keymap::linux_emacs_preset_keep();
    let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
    let search = COMMANDS
        .iter()
        .find(|c| c.name == "Search forward")
        .unwrap();
    let save = COMMANDS.iter().find(|c| c.name == "Save").unwrap();
    assert_eq!(
        join_slots_truthful(forward_char, Convention::Linux, Platform::Native, &preset),
        "C-f"
    );
    assert_eq!(
        join_slots_truthful(search, Convention::Linux, Platform::Native, &preset),
        "C-s"
    );
    assert_eq!(
        join_slots_truthful(save, Convention::Linux, Platform::Native, &preset),
        ""
    );
}

#[test]
fn keymap_flavor_emacs_preset_is_inert_on_mac_for_the_whole_catalog() {
    let preset = crate::keymap::linux_emacs_preset_keep();
    for c in COMMANDS.iter() {
        assert_eq!(
            join_slots_truthful(c, Convention::Mac, Platform::Native, &preset),
            join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
            "{}: the emacs keymap flavor must be inert on Mac",
            c.name
        );
    }
}

/// `Config::effective_linux_keep` is the ONE composition owner both dispatch
/// (`keymap.rs`) and labels (`join_slots_truthful`, via this module) read —
/// pinning that a `keymap = "emacs"` config produces the SAME label as
/// passing the preset directly, so the two can never drift.
#[test]
fn config_effective_linux_keep_feeds_join_slots_truthful_identically_to_the_bare_preset() {
    let mut cfg = crate::config::Config::empty();
    cfg.keymap = Some("emacs".to_string());
    let via_config = cfg.effective_linux_keep();
    let bare_preset = crate::keymap::linux_emacs_preset_keep();
    let forward_char = COMMANDS.iter().find(|c| c.name == "Forward char").unwrap();
    assert_eq!(
        join_slots_truthful(
            forward_char,
            Convention::Linux,
            Platform::Native,
            &via_config
        ),
        join_slots_truthful(
            forward_char,
            Convention::Linux,
            Platform::Native,
            &bare_preset
        ),
    );
}

/// HARD LAW (b): Insert link's VISIBLE effective binding is EMPTY on Linux —
/// out of the box, no user config, under BOTH keymap flavors — while Mac
/// still shows Cmd-K (the `keymap` flavor is a Linux-only concept; Mac's
/// label is unaffected regardless). Drives the SAME `Config::
/// effective_linux_keep()` composition the live palette/rebind-menu read,
/// so a label surface can never advertise a Linux chord that dispatch (see
/// `keymap::tests::out_of_the_box_linux_ctrl_k_is_kill_line_under_both_
/// keymap_flavors`) would never actually honor.
#[test]
fn insert_link_has_no_visible_linux_binding_out_of_the_box_mac_shows_cmd_k() {
    let insert_link = COMMANDS.iter().find(|c| c.name == "Insert link…").unwrap();
    for flavor in ["native", "emacs"] {
        let mut cfg = crate::config::Config::empty();
        cfg.keymap = Some(flavor.to_string());
        let keep = cfg.effective_linux_keep();
        assert_eq!(
            join_slots_truthful(insert_link, Convention::Linux, Platform::Native, &keep),
            "",
            "Insert link must show no Linux chord out of the box under keymap={flavor:?}"
        );
        assert_eq!(
            join_slots_truthful(insert_link, Convention::Mac, Platform::Native, &keep),
            "⌘K",
            "Mac must still show Cmd-K under keymap={flavor:?} (the keep list is Linux-only)"
        );
    }
}

#[test]
fn effective_linux_keep_builtin_floor_is_inert_on_mac_for_the_whole_catalog() {
    for flavor in ["native", "emacs"] {
        let mut cfg = crate::config::Config::empty();
        cfg.keymap = Some(flavor.to_string());
        let keep = cfg.effective_linux_keep();
        for c in COMMANDS.iter() {
            assert_eq!(
                join_slots_truthful(c, Convention::Mac, Platform::Native, &keep),
                join_slots_truthful(c, Convention::Mac, Platform::Native, &[]),
                "{}: the built-in keep floor must be inert on Mac (keymap={flavor:?})",
                c.name
            );
        }
    }
}
