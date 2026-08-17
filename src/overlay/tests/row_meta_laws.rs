use super::*;

// ── Typed `OverlayRow` replacing the 12 corpus-parallel arrays ─────────────

/// The flat switch-project picker AS SUMMONED: the level builder plus the door
/// row its summon seams attach (`OverlayState::attach_browse_door`), so this
/// representative produces every tag the kind declares rather than only the
/// tags the builder alone makes.
fn project_overlay() -> OverlayState {
    let mut ov =
        OverlayState::new_project("/proj".to_string(), vec![("child".to_string(), false)], &[]);
    ov.attach_browse_door();
    ov
}

/// Build a REPRESENTATIVE overlay for `kind` — enough of a real corpus that
/// every `RowMeta` variant [`OverlayKind::row_meta_roster`] declares for it
/// actually gets produced at least once (Command: a hidden row + an appended
/// settings row; Goto: a file row + an appended heading row; Spell: a
/// suggestion + the terminal add row). Used only by
/// [`every_kind_produces_only_its_declared_row_meta_roster`] below.
fn representative_overlay(kind: OverlayKind) -> OverlayState {
    match kind {
        OverlayKind::Goto => {
            let mut ov = OverlayState::new(
                kind,
                vec!["a.md".to_string(), "b.md".to_string()],
                vec![],
                vec![],
            );
            ov.attach_headings(vec![("Intro".to_string(), 3)]);
            ov
        }
        OverlayKind::Project => project_overlay(),
        OverlayKind::Browse => OverlayState::new_marked(
            kind,
            vec!["a.txt".to_string()],
            vec![false],
            vec![false],
            vec![],
            vec![],
            None,
        ),
        OverlayKind::Theme => OverlayState::new_theme(vec!["Tawny".to_string()], 0),
        OverlayKind::Caret => OverlayState::new_caret(crate::caret::CaretMode::ALL[0]),
        OverlayKind::Dictionary => OverlayState::new_dictionary(crate::spell::DictVariant::ALL[0]),
        OverlayKind::CjkLang => {
            OverlayState::new_cjk_lang(crate::frontmatter::DEFAULT_CJK_PRIORITY[0])
        }
        OverlayKind::Date => {
            OverlayState::new_date(crate::dateformat::DateFormat::ALL[0], (2024, 1, 1))
        }
        OverlayKind::Keymap => OverlayState::new_keymap(crate::keymap::KeymapFlavor::ALL[0]),
        OverlayKind::MoveDest | OverlayKind::ExportDest | OverlayKind::ProjectBrowse => {
            OverlayState::new_marked(
                kind,
                vec!["folder".to_string()],
                vec![false],
                vec![true],
                vec![],
                vec![],
                None,
            )
        }
        OverlayKind::Command => {
            let names = crate::commands::visible_names();
            let n = names.len();
            let mut hidden = vec![false; n];
            if n > 0 {
                hidden[0] = true; // touches CommandHidden
            }
            let mut ov = OverlayState::new_command(names, vec![String::new(); n], hidden);
            ov.attach_settings_rows(
                crate::settings::palette_rows(),
                crate::settings::palette_value_cells(&Default::default()),
            ); // touches CommandSetting
            ov
        }
        OverlayKind::Spell => {
            OverlayState::new_spell(vec!["the".to_string()], (0, 0, 3), "teh".to_string())
        }
        OverlayKind::Keybindings => OverlayState::new_keybindings(
            crate::commands::visible_names(),
            crate::commands::visible_effective_bindings(&[], &[]),
        ),
        OverlayKind::History => OverlayState::new_history(history_rows(), None, None),
        OverlayKind::Conflict => OverlayState::new_conflict(
            std::path::PathBuf::from("/notes/a.md"),
            Some("what the disk says\n".to_string()),
        ),
        OverlayKind::Settings => {
            let mut ov = OverlayState::new(kind, crate::settings::visible_names(), vec![], vec![]);
            ov.set_secondaries(crate::settings::visible_value_cells(&Default::default()));
            ov
        }
        OverlayKind::Assets => OverlayState::new_assets(vec![orphan("assets/a.png", 1)]),
        OverlayKind::Rename => OverlayState::new_rename("old.md".to_string()),
        OverlayKind::InsertLink => OverlayState::new_link_edit(
            String::new(),
            crate::overlay::LinkEditMode::Empty { at: 0 },
        ),
        OverlayKind::KeepName => OverlayState::new_keep_name(),
        OverlayKind::Context => crate::context_menu::overlay(
            crate::context_menu::rows(
                crate::context_menu::ContextTarget::Body,
                crate::context_menu::ContextState {
                    has_selection: false,
                    link: false,
                    heading: false,
                    heading_folded: false,
                    misspelled: false,
                    named_file: false,
                },
                crate::commands::Platform::Native,
            ),
            (10.0, 10.0),
        ),
    }
}

/// RUNTIME ROSTER LAW: build a REPRESENTATIVE overlay for EVERY
/// `OverlayKind` and assert every produced row's `meta.tag()` sits inside
/// that kind's declared [`OverlayKind::row_meta_roster`] — the RUNTIME half
/// of the no-wildcard compile-time match (which only forces a NEW kind to
/// DECLARE a roster, never that its constructors actually HONOR it). Catches
/// the "forgot to set the metadata, the row silently stayed Plain" class of
/// bug a pure compile-time sweep can't see.
#[test]
fn every_kind_produces_only_its_declared_row_meta_roster() {
    assert_eq!(
        OverlayKind::ALL.len(),
        OverlayKind::VARIANT_COUNT,
        "the runtime sweep covers the enum's generated variant count"
    );
    assert!(!OverlayKind::ALL.is_empty(), "the sweep is non-vacuous");
    for kind in OverlayKind::ALL {
        let ov = representative_overlay(kind);
        let roster = kind.row_meta_roster();
        assert!(
            !roster.is_empty(),
            "{kind:?} must declare at least one RowMeta tag"
        );
        for row in &ov.rows {
            let tag = row.meta.tag();
            assert!(
                roster.contains(&tag),
                "{kind:?} produced a row with meta tag {tag:?}, not in its \
                 declared roster {roster:?}"
            );
        }
    }
}

/// ROWMETA EXHAUSTIVENESS WITNESS: construct one instance of every
/// [`RowMeta`] variant and check [`RowMeta::tag`] maps it to the matching
/// [`RowMetaTag`]. `RowMeta::tag`'s own match is the real no-wildcard
/// compile-time guard (a future variant fails to compile there until it's
/// mapped); this witnesses the mapping is actually HONEST, not merely that
/// it compiles.
#[test]
fn row_meta_tag_maps_every_variant_correctly() {
    assert_eq!(RowMeta::Plain.tag(), RowMetaTag::Plain);
    assert_eq!(
        RowMeta::GotoFile {
            time: "5m ago".to_string()
        }
        .tag(),
        RowMetaTag::GotoFile
    );
    assert_eq!(
        RowMeta::GotoHeading { line: 3 }.tag(),
        RowMetaTag::GotoHeading
    );
    assert_eq!(RowMeta::GotoFolder.tag(), RowMetaTag::GotoFolder);
    assert_eq!(RowMeta::FolderChooser.tag(), RowMetaTag::FolderChooser);
    assert_eq!(
        RowMeta::CommandSetting {
            id: crate::settings::SettingId::Keymap
        }
        .tag(),
        RowMetaTag::CommandSetting
    );
    assert_eq!(RowMeta::CommandHidden.tag(), RowMetaTag::CommandHidden);
    assert_eq!(RowMeta::SpellAdd.tag(), RowMetaTag::SpellAdd);
    assert_eq!(
        RowMeta::History {
            id: "1".to_string(),
            ts: 0
        }
        .tag(),
        RowMetaTag::History
    );
}

/// PRESERVATION LAW — Go-to HEADING rows keep their `line` across a
/// `refilter` (a typed query re-ranks/narrows `rows` into a fresh `items`
/// view; the line must travel with ITS OWN row, never a neighbor's).
#[test]
fn goto_heading_rows_keep_their_line_across_refilter() {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["a.md".to_string(), "b.md".to_string()],
        vec![],
        vec![],
    );
    ov.attach_headings(vec![("Intro".to_string(), 5), ("Details".to_string(), 42)]);
    let details_ci = ov.rows.iter().position(|r| r.accept == "Details").unwrap();
    assert_eq!(ov.rows[details_ci].meta, RowMeta::GotoHeading { line: 42 });
    // A query tight enough to single out "Details" re-ranks the whole list.
    for c in "eta".chars() {
        ov.push(c);
    }
    assert!(
        ov.items.contains(&details_ci),
        "the Details row survives the query"
    );
    assert_eq!(
        ov.rows[details_ci].meta,
        RowMeta::GotoHeading { line: 42 },
        "the line stayed with its OWN row across refilter"
    );
    let pos = ov.items.iter().position(|&ci| ci == details_ci).unwrap();
    ov.selected = pos;
    assert_eq!(
        ov.selected_line(),
        Some(42),
        "selected_line resolves through the survived row"
    );
}

/// PRESERVATION LAW — Command palette appended SETTINGS rows keep
/// their key (accept) + current value (secondary) across a `refilter`, and
/// [`OverlayState::selected_setting_row`] still resolves the highlighted one
/// correctly once the query has re-ranked the list.
///
/// "Autosave" stands in for an UNCOVERED row (no `COVERED_BY` entry, so it
/// stays a `CommandSetting` row in this exact union rather than being
/// suppressed like a covered row is — "Keymap" itself is COVERED now, by
/// this item's own "Keymap…" catalog command, so it no longer appears here
/// at all; that suppression is what `covered_rows_are_excluded_from_the_
/// palette_on_both_platforms` in `settings/tests.rs` already proves).
#[test]
fn command_palette_settings_rows_keep_key_and_value_across_refilter() {
    let mut ov = OverlayState::new_command(
        crate::commands::visible_names(),
        crate::commands::visible_effective_bindings(&[], &[]),
        crate::commands::visible_hidden_mask(Default::default()),
    );
    ov.attach_settings_rows(
        crate::settings::palette_rows(),
        crate::settings::palette_value_cells(&Default::default()),
    );
    let ci = ov
        .rows
        .iter()
        .position(|r| r.accept == "Autosave")
        .unwrap();
    assert!(matches!(
        ov.rows[ci].meta,
        RowMeta::CommandSetting { id } if id == crate::settings::SettingId::Autosave
    ));
    let value_before = ov.rows[ci].secondary.clone();
    for c in "auto".chars() {
        ov.push(c);
    }
    assert!(ov.items.contains(&ci), "the Autosave row survives the query");
    assert_eq!(
        ov.rows[ci].secondary, value_before,
        "the value traveled with its OWN row"
    );
    ov.selected = ov.items.iter().position(|&i| i == ci).unwrap();
    let resolved = ov
        .selected_setting_row()
        .expect("the highlighted row resolves to a SettingRow");
    assert_eq!(resolved.name, "Autosave");
}

/// PRESERVATION LAW — History rows keep their restore `id` + `ts`
/// across a LENS switch (which rebuilds `items`/`item_sections` from
/// `rows`) AND a subsequent typed query.
#[test]
fn history_rows_keep_id_and_ts_across_lens_switch_and_query() {
    const DAY: u64 = 86_400_000;
    let now = 100 * DAY + 5_000;
    let session_start = 100 * DAY + 3_000;
    let row = |id: &str, ts: u64, which: &str| crate::history::TimelineRow {
        when: "x".to_string(),
        which: which.to_string(),
        counts: "+0 −0".to_string(),
        id: id.to_string(),
        timestamp: ts,
        pinned: false,
        name: None,
    };
    let rows = vec![
        row("a", 100 * DAY + 4_000, "fix engine"),
        row("b", 100 * DAY + 1_000, "edit notes"),
    ];
    let mut ov = OverlayState::new_history(rows, Some(now), Some(session_start));
    let a_ci = ov
        .rows
        .iter()
        .position(|r| matches!(&r.meta, RowMeta::History { id, .. } if id == "a"))
        .unwrap();
    ov.cycle_lens(1); // -> Session
    ov.cycle_lens(-1); // -> back to All
    for c in "fix".chars() {
        ov.push(c);
    }
    assert!(
        ov.items.contains(&a_ci),
        "row a survives the lens switch + query"
    );
    assert_eq!(
        ov.rows[a_ci].meta,
        RowMeta::History {
            id: "a".to_string(),
            ts: 100 * DAY + 4_000
        },
        "id + ts stayed with row a across the lens switch and the query"
    );
}

/// PRESERVATION LAW — Go-to FILE rows keep their OWN relative
/// "last edited" time; appended HEADING rows always read the constant
/// `"heading"` kind hint — never each other's cell, before or after a
/// reorder.
#[test]
fn goto_file_rows_keep_their_own_time_and_headings_read_heading() {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["a.md".to_string(), "b.md".to_string()],
        vec![],
        vec![],
    );
    ov.set_times(vec!["5m ago".to_string(), "2h ago".to_string()]);
    ov.attach_headings(vec![("Intro".to_string(), 1)]);
    let strings = ov.item_strings();
    let times = ov.item_times();
    let a_pos = strings.iter().position(|s| s == "a.md").unwrap();
    let b_pos = strings.iter().position(|s| s == "b.md").unwrap();
    let h_pos = strings.iter().position(|s| s.contains("Intro")).unwrap();
    assert_eq!(times[a_pos], "5m ago");
    assert_eq!(times[b_pos], "2h ago");
    assert_eq!(times[h_pos], "heading");
    // A query that isolates "a.md" re-ranks the list; its own time must follow.
    ov.push('a');
    let strings2 = ov.item_strings();
    let times2 = ov.item_times();
    let a_pos2 = strings2
        .iter()
        .position(|s| s == "a.md")
        .expect("a.md still matches \"a\"");
    assert_eq!(
        times2[a_pos2], "5m ago",
        "a.md's own time survives the reorder"
    );
}

/// A hidden command stays indexed in `rows` but never enters selectable `items`.
#[test]
fn command_hidden_rows_never_reach_items_but_stay_in_rows() {
    let names = crate::commands::visible_names();
    let n = names.len();
    assert!(n > 0, "the command catalog is non-empty");
    let mut hidden = vec![false; n];
    hidden[0] = true;
    let hidden_name = names[0].clone();
    let ov = OverlayState::new_command(names, vec![String::new(); n], hidden);
    assert_eq!(
        ov.rows.len(),
        n,
        "a hidden row is NOT removed from rows — only from items"
    );
    assert!(matches!(ov.rows[0].meta, RowMeta::CommandHidden));
    assert_eq!(
        ov.rows[0].accept, hidden_name,
        "index 0 still names the hidden command"
    );
    assert!(
        !ov.item_strings().iter().any(|s| s == &hidden_name),
        "a hidden row never appears in the SELECTABLE items"
    );
}
