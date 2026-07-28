use super::*;

/// The table has the audited 28 rows (including the Keybindings sub-menu and
/// two Advanced actions), plus Date format, File visibility (item 77), and
/// Scroll sensitivity (item 90). Every display name is UNIQUE (it is both the
/// fuzzy corpus and value-readout key). The exact count is asserted below so
/// an added/removed row must touch this comment deliberately.
#[test]
fn settings_table_names_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for r in SETTINGS {
        assert!(seen.insert(r.name), "duplicate setting name: {}", r.name);
    }
    assert_eq!(SETTINGS.len(), seen.len());
    assert_eq!(
        SETTINGS.len(),
        31,
        "corpus size changed — update this count deliberately (and the doc comments \
             at the top of settings.rs) rather than let it drift"
    );
}

/// SINGLE-OWNER LAW: every setting's category is a real lens SECTION on the
/// strip (so it is reachable under a refinement lens), and every refinement
/// lens's section is a real category (so no lens is dead). Keeps [`SETTINGS`]
/// and [`SETTINGS_FACET_STRIP`] in lockstep — a new category fails until the
/// lens exists, and vice versa.
#[test]
fn every_setting_category_is_a_lens() {
    let lens_sections: Vec<&str> = SETTINGS_FACET_STRIP
        .iter()
        .skip(1) // skip the All home (no sections)
        .filter_map(|f| f.sections.first().copied())
        .collect();
    for r in SETTINGS {
        assert!(
            lens_sections.contains(&r.category),
            "setting {:?} has category {:?} with no matching lens",
            r.name,
            r.category
        );
    }
    for section in &lens_sections {
        assert!(
            SETTINGS.iter().any(|r| r.category == *section),
            "lens section {section:?} has no settings"
        );
    }
}

#[test]
fn settings_bucket_routes_each_lens() {
    for (idx, lens) in SETTINGS_FACET_STRIP.iter().enumerate().skip(1) {
        let section = lens.sections[0];
        for r in SETTINGS {
            let placed = settings_bucket(FacetItem::new(r.name), idx);
            if r.category == section {
                assert_eq!(
                    placed,
                    Some(section),
                    "{} should be under {section}",
                    r.name
                );
            } else {
                assert_eq!(placed, None, "{} should NOT be under {section}", r.name);
            }
        }
    }
}

/// Every table row yields a value readout without hitting the drift fallthrough
/// — the readout `match` and the table can never silently disagree. TOGGLE /
/// PICKER / VALUE / PATH rows carry a non-empty value; SUBMENU / ACTION
/// rows are deliberately blank (affordances, not settings).
#[test]
fn every_setting_has_a_value_readout() {
    let values = SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 0.8,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    };
    for r in SETTINGS {
        let v = value_for(r, &values);
        match r.kind {
            SettingKind::Submenu | SettingKind::Action => {
                assert!(v.is_empty(), "{} is an affordance, no value", r.name);
            }
            _ => assert!(!v.is_empty(), "{} must have a value readout", r.name),
        }
    }
}

/// INTERACTION LAW: every TOGGLE row resolves a config key (so Enter can flip +
/// persist it) and every NON-toggle row resolves NONE — the `SettingKind::Toggle`
/// discriminant and [`toggle_key`] can never disagree about what is flippable.
#[test]
fn every_toggle_has_a_config_key_and_nothing_else_does() {
    for r in SETTINGS {
        match r.kind {
            SettingKind::Toggle => assert!(
                toggle_key(r.id).is_some(),
                "toggle {:?} has no config key",
                r.name
            ),
            _ => assert!(
                toggle_key(r.id).is_none(),
                "non-toggle {:?} resolved a toggle key",
                r.name
            ),
        }
    }
}

#[test]
fn pickers_and_submenus_open_a_sub_overlay_and_nothing_else_does() {
    for r in SETTINGS {
        match r.kind {
            SettingKind::Picker | SettingKind::Submenu => assert!(
                sub_overlay(r.id).is_some(),
                "{:?} ({:?}) opens no sub-overlay",
                r.name,
                r.kind
            ),
            _ => assert!(
                sub_overlay(r.id).is_none(),
                "{:?} unexpectedly opens a sub-overlay",
                r.name
            ),
        }
    }
}

#[test]
fn value_and_path_keys_track_their_kinds() {
    for r in SETTINGS {
        match r.kind {
            SettingKind::Value | SettingKind::Range => {
                assert!(value_key(r.id).is_some(), "value {:?} has no key", r.name);
                assert!(
                    path_key(r.id).is_none(),
                    "value {:?} resolved a path key",
                    r.name
                );
            }
            SettingKind::Path => {
                assert!(path_key(r.id).is_some(), "path {:?} has no key", r.name);
                assert!(
                    value_key(r.id).is_none(),
                    "path {:?} resolved a value key",
                    r.name
                );
            }
            SettingKind::Toggle
            | SettingKind::Picker
            | SettingKind::Submenu
            | SettingKind::Action => {
                assert!(
                    value_key(r.id).is_none(),
                    "{:?} resolved a value key",
                    r.name
                );
                assert!(path_key(r.id).is_none(), "{:?} resolved a path key", r.name);
            }
        }
    }
}

/// ITEM 94 — THE KIND CONTRACT SWEEP, no wildcard anywhere: for EVERY row,
/// exactly which of the five behaviour maps must resolve, declared per
/// [`SettingKind`]. A future kind fails to COMPILE here until it declares its
/// whole contract; a row wired into the wrong map fails at RUN time. This is the
/// one place all five maps are checked against each other, so `range_spec` can
/// never quietly grow (or lose) a member the way a `_ => None` fallthrough allows.
#[test]
fn every_setting_kind_declares_its_whole_behaviour_contract() {
    let _g = crate::testlock::serial();
    for r in SETTINGS {
        let (toggle, value, path, sub, range) = match r.kind {
            SettingKind::Toggle => (true, false, false, false, false),
            SettingKind::Picker => (false, false, false, true, false),
            SettingKind::Value => (false, true, false, false, false),
            SettingKind::Range => (false, true, false, false, true),
            SettingKind::Path => (false, false, true, false, false),
            SettingKind::Submenu => (false, false, false, true, false),
            SettingKind::Action => (false, false, false, false, false),
        };
        assert_eq!(
            toggle_key(r.id).is_some(),
            toggle,
            "{:?}: toggle_key",
            r.name
        );
        assert_eq!(value_key(r.id).is_some(), value, "{:?}: value_key", r.name);
        assert_eq!(path_key(r.id).is_some(), path, "{:?}: path_key", r.name);
        assert_eq!(
            sub_overlay(r.id).is_some(),
            sub,
            "{:?}: sub_overlay",
            r.name
        );
        assert_eq!(
            range_spec(r.id).is_some(),
            range,
            "{:?}: range_spec",
            r.name
        );
    }
}

/// ITEM 94 — a RANGE row resolves EVERYTHING its interaction needs, and no other
/// row resolves ANY of it: the authored spec, a live value in the gathered
/// readout inputs, a config key to persist under, and a rail cell for the drawn
/// thumb. The four maps sweep together so a range row can never be half-wired
/// (a spec with no live value would step a number nothing reads).
#[test]
fn every_range_row_is_wired_end_to_end_and_nothing_else_is() {
    let _g = crate::testlock::serial();
    let values = probe_values();
    for r in SETTINGS {
        let is_range = r.kind == SettingKind::Range;
        assert_eq!(range_spec(r.id).is_some(), is_range, "{:?}: spec", r.name);
        assert_eq!(
            range_value(r.id, &values).is_some(),
            is_range,
            "{:?}: value",
            r.name
        );
        assert_eq!(
            range_cell(r, &values).is_some(),
            is_range,
            "{:?}: cell",
            r.name
        );
        if is_range {
            let spec = range_spec(r.id).unwrap();
            let v = range_value(r.id, &values).unwrap();
            let cell = range_cell(r, &values).unwrap();
            assert_eq!(
                cell.id, r.id,
                "{:?}: the cell carries its own identity",
                r.name
            );
            assert_eq!(
                cell.step,
                spec.step_of(v),
                "{:?}: the cell is the spec's step",
                r.name
            );
            assert_eq!(
                value_for(r, &values),
                spec.format(spec.value_of_step(cell.step)),
                "{:?}: the value cell and the thumb disagree",
                r.name
            );
            assert!(
                value_key(r.id).is_some(),
                "{:?}: nothing to persist under",
                r.name
            );
        }
    }
}

#[test]
fn visible_range_cells_are_parallel_to_the_visible_rows() {
    let _g = crate::testlock::serial();
    let values = probe_values();
    let cells = visible_range_cells(&values);
    let rows = visible_rows();
    assert_eq!(
        cells.len(),
        rows.len(),
        "the rail column is parallel to the rows"
    );
    for (row, cell) in rows.iter().zip(&cells) {
        assert_eq!(
            cell.is_some(),
            row.kind == SettingKind::Range,
            "{:?} carries the wrong rail state",
            row.name
        );
    }
    assert_eq!(
        cells.iter().filter(|c| c.is_some()).count(),
        SETTINGS
            .iter()
            .filter(|r| r.kind == SettingKind::Range)
            .count()
    );
}

/// Only bounded numeric continuous judgments get a rail. This no-wildcard
/// roster makes every other row state why it is not a Range, so a future kind
/// change cannot silently broaden the interaction grammar.
#[test]
fn the_complete_settings_roster_has_an_explicit_range_decision() {
    let _g = crate::testlock::serial();
    for row in SETTINGS {
        let exclusion = match row.id {
            SettingId::PageWidthProse
            | SettingId::PageWidthCode
            | SettingId::Zoom
            | SettingId::ScrollSensitivity => None,
            SettingId::CaretStyle
            | SettingId::DateFormat
            | SettingId::Theme
            | SettingId::Dictionary
            | SettingId::CjkReadsAs
            | SettingId::Keymap => Some("discrete choice"),
            SettingId::PageMode
            | SettingId::TypewriterScroll
            | SettingId::ReduceMotion
            | SettingId::Wysiwyg
            | SettingId::FormatPopover
            | SettingId::InlineImages
            | SettingId::CodeLigatures
            | SettingId::Outline
            | SettingId::MenuBar
            | SettingId::Spellcheck
            | SettingId::WritingNits
            | SettingId::FileVisibility
            | SettingId::Autosave
            | SettingId::LocalHistory
            | SettingId::SessionRestore => Some("boolean toggle"),
            SettingId::DefaultFolder | SettingId::ProjectsFolder | SettingId::ProjectRoot => {
                Some("path picker")
            }
            SettingId::Keybindings => Some("submenu"),
            SettingId::ReportProblem | SettingId::EditConfigAsText => Some("action"),
        };
        assert_eq!(
            row.kind == SettingKind::Range,
            exclusion.is_none(),
            "{}: Range suitability disagrees with its explicit decision ({exclusion:?})",
            row.name
        );
    }
}

#[test]
fn the_zoom_range_is_the_authored_fifty_to_three_hundred_percent_linear_rail() {
    let _g = crate::testlock::serial();
    let spec = range_spec(SettingId::Zoom).expect("Zoom is a range row");
    assert_eq!(
        (spec.min, spec.max, spec.step, spec.default),
        (0.5, 3.0, 0.1, 1.0)
    );
    assert_eq!(spec.map, crate::range::RailMap::Linear);
    assert_eq!(spec.unit, crate::range::Unit::Percent);
    assert_eq!(spec.step_count(), 26, "50%..300% in 10-point steps");
    assert_eq!(spec.format(0.5), "50%");
    assert_eq!(spec.format(3.0), "300%");
    assert_eq!(parse_zoom("140%"), Some(spec.value_of_step(14)));
    assert_eq!(spec.frac_of(0.5), 0.0);
    assert!((spec.frac_of(3.0) - 1.0).abs() < 1e-5);
    assert!((spec.frac_of(1.0) - 0.2).abs() < 1e-5);
}

/// ITEM 94 — every zoom DOOR lands on the same authored grid: the ⌘±/rail
/// `stepped` owner, the wheel's own call, `clamp_zoom` (which `--zoom`, a config
/// load and `set_zoom` all run through) and the typed exact entry. No input path
/// computes a parallel value — the DONE criterion, asserted.
#[test]
fn every_zoom_door_lands_on_the_same_authored_grid() {
    let _g = crate::testlock::serial();
    let spec = range_spec(SettingId::Zoom).unwrap();
    for k in spec.min_step()..=spec.max_step() {
        let v = spec.value_of_step(k);
        assert_eq!(
            crate::render::clamp_zoom(v).to_bits(),
            v.to_bits(),
            "clamp_zoom({v})"
        );
        assert_eq!(
            parse_zoom(&spec.format(v)),
            Some(v),
            "typing {v}'s own readout"
        );
        if k < spec.max_step() {
            assert_eq!(spec.stepped(v, 1), spec.value_of_step(k + 1));
        }
        let f = spec.frac_of_step(k);
        assert_eq!(
            spec.value_at_frac(f).to_bits(),
            v.to_bits(),
            "the rail at {f}"
        );
    }
}

fn probe_values() -> SettingsValues {
    SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.4,
        scroll_sensitivity: 2.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    }
}

/// Typed numerics parse and clamp through their Range owners.
#[test]
fn value_parse_and_clamp_are_sane() {
    let width = &crate::range::PAGE_WIDTH_PROSE;
    assert_eq!(width.parse("45"), Some(45.0));
    assert_eq!(width.parse("5"), Some(width.min), "a tiny width clamps up");
    assert_eq!(
        width.parse("9000"),
        Some(width.max),
        "a huge width clamps down"
    );

    assert_eq!(parse_zoom("80%"), Some(0.8));
    assert_eq!(parse_zoom("1.5"), Some(1.5));
    assert_eq!(
        parse_zoom("125"),
        Some(crate::render::clamp_zoom(1.25)),
        "an integer-ish value reads as a percent"
    );
    assert_eq!(parse_zoom("5000%"), Some(crate::range::ZOOM.max));
    assert_eq!(
        parse_zoom("10%"),
        Some(crate::range::ZOOM.min),
        "10% -> 0.1 clamps up to the floor"
    );
    assert_eq!(parse_zoom("oops"), None);
    assert_eq!(parse_zoom(""), None);
}

/// A few concrete value cells match the process-global / gathered owners
/// (the readout reads the SAME truth the renderer does). Outline reads its
/// PROCESS GLOBAL (the renderer's owner), not a gathered config copy — the
/// every-toggle-dispatches sweep's fix — so it is flipped here under the
/// one test guard and restored.
#[test]
fn value_cells_read_the_live_owners() {
    let _g = crate::testlock::serial();
    let values = SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 0.8,
        scroll_sensitivity: 1.0,
        ..Default::default()
    };
    let find = |name: &str| *SETTINGS.iter().find(|r| r.name == name).unwrap();
    assert_eq!(value_for(&find("Page width (prose)"), &values), "70");
    assert_eq!(value_for(&find("Page width (code)"), &values), "100");
    assert_eq!(value_for(&find("Zoom"), &values), "80%");
    let outline0 = crate::outline::outline_on();
    crate::outline::set_outline_on(true);
    assert_eq!(value_for(&find("Outline"), &values), "on");
    crate::outline::set_outline_on(false);
    assert_eq!(value_for(&find("Outline"), &values), "off");
    crate::outline::set_outline_on(outline0);
    assert_eq!(
        value_for(&find("Theme"), &values),
        crate::theme::active().name
    );
}

#[test]
fn date_format_row_is_a_picker_and_previews_today() {
    let _g = crate::testlock::serial();
    let row = row_of(SettingId::DateFormat);
    assert_eq!(row.kind, SettingKind::Picker);
    assert_eq!(sub_overlay(row.id), Some(crate::overlay::OverlayKind::Date));
    assert_eq!(toggle_key(row.id), None, "a picker row has no toggle key");

    let saved = crate::dateformat::active_format();
    let values = SettingsValues {
        today_ymd: (2009, 3, 7),
        ..Default::default()
    };
    crate::dateformat::set_active_format(crate::dateformat::DateFormat::DdMmYy);
    assert_eq!(value_for(&row, &values), "07/03/09");
    crate::dateformat::set_active_format(crate::dateformat::DateFormat::Iso);
    assert_eq!(value_for(&row, &values), "2009-03-07");
    crate::dateformat::set_active_format(crate::dateformat::DateFormat::DMonthYyyy);
    assert_eq!(value_for(&row, &values), "7 March 2009");
    crate::dateformat::set_active_format(saved); // restore, no leak to another test
}

/// The "Ambiguous CJK reads as" row is a Picker (opening
/// `OverlayKind::CjkLang`), and its value cell shows the live ladder's
/// FRONT language in WRITER WORDS ("Japanese"), never the raw BCP 47 code
/// ("ja") — the whole point of the row growing up from `SettingKind::List`.
#[test]
fn cjk_row_is_a_picker_with_a_writer_word_value_cell() {
    let _g = crate::testlock::serial();
    let row = row_of(SettingId::CjkReadsAs);
    assert_eq!(row.kind, SettingKind::Picker);
    assert_eq!(
        sub_overlay(row.id),
        Some(crate::overlay::OverlayKind::CjkLang)
    );

    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
    assert_eq!(value_for(&row, &SettingsValues::default()), "Japanese");

    crate::frontmatter::set_cjk_priority(&crate::frontmatter::promote_cjk_priority(
        crate::frontmatter::Lang::Ko,
    ));
    assert_eq!(value_for(&row, &SettingsValues::default()), "Korean");

    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
}

#[test]
fn visible_rows_native_is_the_full_table() {
    assert_eq!(
        visible_rows_on(crate::commands::Platform::Native).len(),
        SETTINGS.len()
    );
    assert_eq!(
        visible_names(),
        names(),
        "native: visible_names must match the full table"
    );
}

/// On `Web`, EVERY row is now visible too — "Edit config as text" stopped
/// hiding once `main::wasm_start` started loading a real `config.toml` over
/// `WebFs` (`fs::web_config_path`), so `App::open_settings`'s empty-path guard
/// never fires there anymore.
#[test]
fn visible_rows_web_is_also_the_full_table() {
    let web = visible_rows_on(crate::commands::Platform::Web);
    assert_eq!(web.len(), SETTINGS.len());
    assert!(web.iter().any(|r| r.name == "Edit config as text"));
}

/// INDEX COHERENCE: `visible_names()`/`visible_value_cells()` stay parallel to
/// `visible_rows()` on THIS platform — a picker row's index always names the
/// SAME row across all three, so `settings_accept`'s `visible_rows()[ci]` lookup
/// can never mis-map.
#[test]
fn visible_names_and_value_cells_are_parallel_to_visible_rows() {
    let rows = visible_rows();
    let names = visible_names();
    let cells = visible_value_cells(&SettingsValues::default());
    assert_eq!(names.len(), rows.len());
    assert_eq!(cells.len(), rows.len());
    for (i, r) in rows.iter().enumerate() {
        assert_eq!(names[i], r.name);
    }
}

#[test]
fn every_covered_by_pair_names_a_real_row_and_a_real_command() {
    for (row_id, cmd_name) in COVERED_BY {
        assert!(
            SETTINGS.iter().any(|r| r.id == *row_id),
            "COVERED_BY names no real settings row: {row_id:?}"
        );
        assert!(
            crate::commands::COMMANDS
                .iter()
                .any(|c| c.name == *cmd_name),
            "COVERED_BY names no real catalog command: {cmd_name:?}"
        );
    }
}

#[test]
fn covered_by_picker_rows_open_the_same_overlay_as_their_command() {
    use crate::keymap::Action;
    use crate::overlay::OverlayKind;
    for (row_id, cmd_name) in COVERED_BY {
        let row = row_of(*row_id);
        if !matches!(row.kind, SettingKind::Picker | SettingKind::Submenu) {
            continue;
        }
        let cmd = crate::commands::COMMANDS
            .iter()
            .find(|c| c.name == *cmd_name)
            .unwrap();
        let expected = match &cmd.action {
            Action::OpenThemeMenu => OverlayKind::Theme,
            Action::OpenCaretMenu => OverlayKind::Caret,
            Action::OpenDictionaryMenu => OverlayKind::Dictionary,
            Action::OpenKeybindings => OverlayKind::Keybindings,
            other => panic!(
                "{cmd_name:?} covers {row_id:?} but its action {other:?} \
                                  isn't a known overlay-opening arm — add it here"
            ),
        };
        assert_eq!(
            sub_overlay(row.id),
            Some(expected),
            "{row_id:?} and {cmd_name:?} must open the same overlay"
        );
    }
}

#[test]
fn covered_by_toggle_rows_flip_the_same_global_as_their_command() {
    use crate::keymap::Action;
    let _g = crate::testlock::serial();
    let values = SettingsValues::default();
    for (row_id, cmd_name) in COVERED_BY {
        let row = row_of(*row_id);
        if row.kind != SettingKind::Toggle {
            continue;
        }
        let cmd = crate::commands::COMMANDS
            .iter()
            .find(|c| c.name == *cmd_name)
            .unwrap();
        let flip = || match &cmd.action {
            Action::TogglePageMode => crate::page::toggle(),
            Action::ToggleTypewriter => crate::typewriter::toggle(),
            Action::ToggleOutline => crate::outline::toggle(),
            Action::ToggleMenuBar => crate::menubar::toggle(),
            Action::ToggleSpellcheck => crate::spell::toggle(),
            Action::ToggleWritingNits => crate::nits::toggle(),
            other => panic!(
                "{cmd_name:?} covers {row_id:?} but its action {other:?} \
                                  isn't a known global-flipping arm — add it here"
            ),
        };
        let before = value_for(&row, &values);
        flip();
        let after = value_for(&row, &values);
        assert_ne!(
            before, after,
            "{row_id:?}'s value must flip when {cmd_name:?} fires"
        );
        flip(); // restore, so this test never leaks state to another.
        assert_eq!(
            value_for(&row, &values),
            before,
            "flip must be a true toggle"
        );
    }
}

#[test]
fn covered_rows_are_excluded_from_the_palette_on_both_platforms() {
    use crate::commands::Platform;
    for platform in [Platform::Native, Platform::Web] {
        let palette = palette_rows_on(platform);
        for (row_id, cmd_name) in COVERED_BY {
            if crate::commands::available_by_name(cmd_name, platform) {
                assert!(
                    !palette.iter().any(|r| r.id == *row_id),
                    "{row_id:?} must not appear in the {platform:?} palette union \
                         while {cmd_name:?} covers it there"
                );
            }
        }
    }
}

/// A covered row stays FULLY FUNCTIONAL inside the Settings menu itself —
/// this fix only trims the PALETTE corpus, never `visible_rows`.
#[test]
fn covered_rows_stay_in_the_settings_menu_unaffected() {
    for (row_id, _) in COVERED_BY {
        assert!(
            visible_rows().iter().any(|r| r.id == *row_id),
            "{row_id:?} must remain reachable from the Settings menu"
        );
    }
}

/// THE REAPPEARANCE CASE (a covered row whose covering command is
/// PLATFORM-HIDDEN): tested against the pure decision fn directly with a real
/// `native_only` command standing in for a hypothetical covering command,
/// since none of today's ten real `COVERED_BY` commands happen to be
/// platform-scoped. `Native` (where the stand-in command IS available) hides
/// the row exactly like a real covered pair; `Web` (where it's hidden) lets
/// the row REAPPEAR — the door is never entirely lost.
#[test]
fn covered_row_reappears_in_the_palette_if_its_command_is_platform_hidden() {
    use crate::commands::Platform;
    let stand_in = "Version history…";
    assert!(crate::commands::available_by_name(
        stand_in,
        Platform::Native
    ));
    assert!(!crate::commands::available_by_name(stand_in, Platform::Web));

    assert!(
        !row_visible_in_palette(Some(stand_in), Platform::Native),
        "covered + command available -> hidden"
    );
    assert!(
        row_visible_in_palette(Some(stand_in), Platform::Web),
        "covered + command platform-hidden -> the row REAPPEARS, door never lost"
    );
    assert!(row_visible_in_palette(None, Platform::Native));
    assert!(row_visible_in_palette(None, Platform::Web));
}

/// STRONGER DEDUPE LAW: for every sub-overlay kind a settings Picker/Submenu
/// row can ever open ([`sub_overlay`]'s own closed range), the palette union
/// has EXACTLY ONE door to it — either the uncovered settings row, or the
/// covering command (never both, and — since every such kind names a real
/// row today — never neither). A future settings row sharing a destination
/// with an existing command fails this test until it's added to
/// [`COVERED_BY`].
#[test]
fn no_two_palette_doors_open_the_same_settings_sub_overlay() {
    use crate::keymap::Action;
    use crate::overlay::OverlayKind;
    let kinds = [
        OverlayKind::Caret,
        OverlayKind::Theme,
        OverlayKind::Dictionary,
        OverlayKind::CjkLang,
        OverlayKind::Keybindings,
    ];
    let command_opens = |a: &Action| match a {
        Action::OpenCaretMenu => Some(OverlayKind::Caret),
        Action::OpenThemeMenu => Some(OverlayKind::Theme),
        Action::OpenDictionaryMenu => Some(OverlayKind::Dictionary),
        Action::OpenKeybindings => Some(OverlayKind::Keybindings),
        _ => None,
    };
    let palette = palette_rows();
    for kind in kinds {
        let command_doors = crate::commands::visible()
            .into_iter()
            .filter(|c| command_opens(&c.action) == Some(kind))
            .count();
        let row_doors = palette
            .iter()
            .filter(|r| sub_overlay(r.id) == Some(kind))
            .count();
        assert_eq!(
            command_doors + row_doors,
            1,
            "{kind:?} must have exactly one palette door (commands={command_doors}, rows={row_doors})"
        );
    }
}

impl SettingId {
    #[allow(dead_code)]
    fn witness(self) {
        match self {
            SettingId::CaretStyle
            | SettingId::PageMode
            | SettingId::TypewriterScroll
            | SettingId::ReduceMotion
            | SettingId::PageWidthProse
            | SettingId::PageWidthCode
            | SettingId::Zoom
            | SettingId::ScrollSensitivity
            | SettingId::DateFormat
            | SettingId::Theme
            | SettingId::Wysiwyg
            | SettingId::FormatPopover
            | SettingId::InlineImages
            | SettingId::CodeLigatures
            | SettingId::Outline
            | SettingId::MenuBar
            | SettingId::Spellcheck
            | SettingId::Dictionary
            | SettingId::WritingNits
            | SettingId::CjkReadsAs
            | SettingId::DefaultFolder
            | SettingId::ProjectsFolder
            | SettingId::ProjectRoot
            | SettingId::FileVisibility
            | SettingId::Autosave
            | SettingId::LocalHistory
            | SettingId::SessionRestore
            | SettingId::Keymap
            | SettingId::Keybindings
            | SettingId::ReportProblem
            | SettingId::EditConfigAsText => {}
        }
    }
}

#[test]
fn every_setting_id_maps_1_to_1_to_the_registry() {
    let roster: &[SettingId] = &[
        SettingId::CaretStyle,
        SettingId::PageMode,
        SettingId::TypewriterScroll,
        SettingId::ReduceMotion,
        SettingId::PageWidthProse,
        SettingId::PageWidthCode,
        SettingId::Zoom,
        SettingId::ScrollSensitivity,
        SettingId::DateFormat,
        SettingId::Theme,
        SettingId::Wysiwyg,
        SettingId::FormatPopover,
        SettingId::InlineImages,
        SettingId::CodeLigatures,
        SettingId::Outline,
        SettingId::MenuBar,
        SettingId::Spellcheck,
        SettingId::Dictionary,
        SettingId::WritingNits,
        SettingId::CjkReadsAs,
        SettingId::DefaultFolder,
        SettingId::ProjectsFolder,
        SettingId::ProjectRoot,
        SettingId::FileVisibility,
        SettingId::Autosave,
        SettingId::LocalHistory,
        SettingId::SessionRestore,
        SettingId::Keymap,
        SettingId::Keybindings,
        SettingId::ReportProblem,
        SettingId::EditConfigAsText,
    ];
    roster.iter().for_each(|id| id.witness());
    assert_eq!(
        roster.len(),
        31,
        "the hand-listed roster changed size — update deliberately"
    );
    assert_eq!(roster.len(), SETTINGS.len(), "roster/registry size drifted");

    let mut seen = std::collections::HashSet::new();
    for r in SETTINGS {
        assert!(
            seen.insert(r.id),
            "duplicate SettingId in SETTINGS: {:?}",
            r.id
        );
    }
    assert_eq!(
        seen.len(),
        SETTINGS.len(),
        "every SETTINGS row has a UNIQUE id"
    );

    for id in roster {
        assert!(
            SETTINGS.iter().any(|r| r.id == *id),
            "SettingId::{id:?} names no SETTINGS row"
        );
    }
    for r in SETTINGS {
        assert_eq!(
            row_of(r.id).name,
            r.name,
            "row_of round-trip failed for {:?}",
            r.id
        );
    }
}

/// HEADLINE LAW (item 55): renaming a row's DISPLAY LABEL changes NO
/// behavior — every resolver (`toggle_key`/`value_key`/`path_key`/
/// `sub_overlay`/`value_for`, INCLUDING the value readout, the subtle one
/// per the item-55 plan) switches on the row's typed `id`, never its
/// `name`. FAILS before item 55 (when these resolvers matched on
/// `row.name`): confirmed non-vacuous by construction — this literally
/// builds a relabeled COPY of each row and re-runs every resolver against
/// it, so a regression back to name-keyed matching reintroduces the
/// failure immediately (a `SettingRow` is `Copy`, so `relabeled` and `r`
/// are two independent values sharing only the `id`).
#[test]
fn a_label_edit_changes_no_behavior() {
    let _g = crate::testlock::serial();
    let values = SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 0.8,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    };
    for r in SETTINGS {
        let relabeled = SettingRow {
            name: "nonsense zzqx label",
            ..*r
        };
        assert_eq!(
            toggle_key(relabeled.id),
            toggle_key(r.id),
            "{:?}: toggle_key drifted on a label-only edit",
            r.name
        );
        assert_eq!(
            value_key(relabeled.id),
            value_key(r.id),
            "{:?}: value_key drifted on a label-only edit",
            r.name
        );
        assert_eq!(
            path_key(relabeled.id),
            path_key(r.id),
            "{:?}: path_key drifted on a label-only edit",
            r.name
        );
        assert_eq!(
            sub_overlay(relabeled.id),
            sub_overlay(r.id),
            "{:?}: sub_overlay drifted on a label-only edit",
            r.name
        );
        assert_eq!(
            value_for(&relabeled, &values),
            value_for(r, &values),
            "{:?}: value_for drifted on a label-only edit",
            r.name
        );
    }
}

#[test]
fn action_kind_rows_are_exactly_report_problem_and_edit_config_as_text() {
    let action_ids: std::collections::HashSet<SettingId> = SETTINGS
        .iter()
        .filter(|r| r.kind == SettingKind::Action)
        .map(|r| r.id)
        .collect();
    assert_eq!(
        action_ids,
        std::collections::HashSet::from([SettingId::ReportProblem, SettingId::EditConfigAsText])
    );
}

#[test]
fn typed_ids_still_emit_the_legacy_wire_keys() {
    assert_eq!(toggle_key(SettingId::PageMode), Some("page_mode"));
    assert_eq!(
        toggle_key(SettingId::TypewriterScroll),
        Some("typewriter_scroll")
    );
    assert_eq!(toggle_key(SettingId::ReduceMotion), Some("reduce_motion"));
    assert_eq!(toggle_key(SettingId::Wysiwyg), Some("wysiwyg"));
    assert_eq!(toggle_key(SettingId::FormatPopover), Some("popover"));
    assert_eq!(toggle_key(SettingId::InlineImages), Some("inline_images"));
    assert_eq!(toggle_key(SettingId::CodeLigatures), Some("code_ligatures"));
    assert_eq!(toggle_key(SettingId::Outline), Some("outline"));
    assert_eq!(toggle_key(SettingId::MenuBar), Some("menu_bar"));
    assert_eq!(toggle_key(SettingId::Spellcheck), Some("spellcheck"));
    assert_eq!(toggle_key(SettingId::WritingNits), Some("writing_nits"));
    assert_eq!(toggle_key(SettingId::Autosave), Some("autosave"));
    assert_eq!(toggle_key(SettingId::LocalHistory), Some("history"));
    assert_eq!(
        toggle_key(SettingId::SessionRestore),
        Some("session_restore")
    );
    assert_eq!(toggle_key(SettingId::Keymap), Some("keymap"));
    assert_eq!(
        toggle_key(SettingId::DateFormat),
        None,
        "a picker row has no toggle key"
    );

    assert_eq!(
        value_key(SettingId::PageWidthProse),
        Some("page_width_prose")
    );
    assert_eq!(value_key(SettingId::PageWidthCode), Some("page_width_code"));
    assert_eq!(value_key(SettingId::Zoom), Some("zoom"));

    assert_eq!(path_key(SettingId::DefaultFolder), Some("default_folder"));
    assert_eq!(path_key(SettingId::ProjectsFolder), Some("workspace"));
    assert_eq!(path_key(SettingId::ProjectRoot), Some("project_root"));
}
