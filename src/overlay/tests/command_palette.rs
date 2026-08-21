use super::*;

/// A minimal [`BuildCtx`] with every field empty/None — the tests that only
/// care about ONE input fill just that one.
fn empty_build_ctx<'a>(config_keys: &'a [(String, Vec<String>)]) -> BuildCtx<'a> {
    BuildCtx {
        goto_corpus: Vec::new(),
        goto_open: Vec::new(),
        goto_recent: Vec::new(),
        goto_times: Vec::new(),
        config_keys,
        config_linux_keep: &[],
        goto_headings: Vec::new(),
        goto_line_count: 0,
        goto_folders: Vec::new(),
        goto_recent_folders: Vec::new(),
        spell_target: None,
        history_entries: Vec::new(),
        history_now: None,
        history_session_start: None,
        settings_values: Default::default(),
        assets: Vec::new(),
        row_gates: Default::default(),
    }
}

/// FINISH BUFFER GATING: the palette row list excludes "Finish file"
/// with no daemon `--wait` client waiting, and re-includes it — dispatching
/// correctly — the instant one is. Built through the REAL `overlay::build` seam
/// (the same one the live App and the headless replay both call), so this is the
/// purest reachable seam short of a live daemon round-trip (flagged for human
/// confirmation in the report — the daemon itself is structurally live-only).
#[test]
fn command_palette_hides_finish_buffer_without_a_waiter_and_shows_it_with_one() {
    // NO waiter (the default `BuildCtx`, matching headless capture / a fresh
    // live App with nothing waiting): "Finish file" is absent from what's
    // rankable/selectable...
    let ctx_idle = BuildCtx {
        row_gates: Default::default(),
        ..empty_build_ctx(&[])
    };
    let ov_idle = crate::overlay::build(OverlayKind::Command, &ctx_idle)
        .expect("the Command palette always summons");
    assert!(
        !ov_idle.item_strings().contains(&"Finish file".to_string()),
        "Finish file must be hidden from the palette with no active daemon waiter"
    );
    // ...but the underlying corpus itself is UNTOUCHED (only what's shown/
    // selectable shrinks) — the row-index math `commands::visible_action_of`
    // relies on for every OTHER command stays valid.
    assert!(
        ov_idle.accepts().contains(&"Finish file"),
        "hiding a row must not shrink the underlying corpus (index-stability)"
    );
    assert_eq!(
        ov_idle.rows.len(),
        crate::commands::visible_names().len() + crate::settings::palette_names().len(),
        "corpus stays exactly commands::visible() + the settings union, unshrunk"
    );

    // A waiter IS active: the row reappears...
    let ctx_waiting = BuildCtx {
        row_gates: crate::commands::RowGates {
            has_waiter: true,
            ..Default::default()
        },
        ..empty_build_ctx(&[])
    };
    let mut ov_waiting = crate::overlay::build(OverlayKind::Command, &ctx_waiting)
        .expect("the Command palette always summons");
    assert!(
        ov_waiting
            .item_strings()
            .contains(&"Finish file".to_string()),
        "Finish file must show while a daemon waiter is active"
    );
    // ...and selecting it resolves through the SAME `commands::visible_action_of`
    // seam the real palette Enter/accept uses (`actions::overlay_nav`) — proving
    // DISPATCH stays unchanged: a shown Finish file row still runs the real
    // `Action::FinishBuffer`.
    ov_waiting.query = crate::textbox::TextBox::seeded("Finish file");
    ov_waiting.refilter();
    let idx = ov_waiting
        .selected_corpus_index()
        .expect("the exact name must fuzzy-match its own row");
    assert_eq!(
        crate::commands::visible_action_of(idx),
        crate::keymap::Action::FinishBuffer
    );
}

#[test]
fn command_palette_lists_names_with_parallel_bindings() {
    let names = vec![
        "Go to file".to_string(),
        "Switch theme".to_string(),
        "Save".to_string(),
    ];
    let binds = vec![
        "C-x C-f".to_string(),
        "C-x t".to_string(),
        "C-x C-s".to_string(),
    ];
    let mut ov = OverlayState::new_command(names.clone(), binds.clone(), vec![false; names.len()]);
    assert_eq!(ov.kind.as_str(), "command");
    // Empty query: rows are the names in order, bindings stay parallel.
    assert_eq!(ov.item_strings(), names);
    assert_eq!(ov.item_bindings(), binds);
    // Fuzzy filter narrows to "Switch theme" and keeps its binding aligned.
    ov.push('t');
    ov.push('h');
    ov.push('e');
    assert_eq!(ov.selected_value(), Some("Switch theme"));
    assert_eq!(
        ov.item_bindings().first().map(|s| s.as_str()),
        Some("C-x t")
    );
}

#[test]
fn command_picker_lands_on_all_then_groups_every_task_category_and_recent() {
    let names = crate::commands::names();
    let binds = crate::commands::effective_bindings(&[], &[]);
    let hidden = vec![false; names.len()];
    let mut ov = OverlayState::new_command(names, binds, hidden);
    // Lands on the flat All home; the strip is All-first.
    assert_eq!(
        ov.active_facet_id(),
        Some("all"),
        "opens on the flat All landing"
    );
    assert_eq!(
        ov.lens_strip().first().map(|(l, _)| l.clone()),
        Some("All".to_string())
    );
    assert!(
        ov.item_sections().iter().all(|s| s.is_empty()),
        "All never groups"
    );
    for (idx, category) in crate::commands::TaskCategory::ALL.into_iter().enumerate() {
        ov.set_facet_lens(idx + 1);
        assert_eq!(
            ov.active_facet_id(),
            Some(category.label().to_ascii_lowercase().as_str())
        );
        assert!(
            !ov.items.is_empty(),
            "{} category is non-empty",
            category.label()
        );
        for (row, &ci) in ov.items.iter().enumerate() {
            assert_eq!(ov.item_sections()[row], category.label());
            assert_eq!(
                crate::commands::task_category_of(&ov.rows[ci].accept),
                Some(category)
            );
        }
    }
    ov.set_facet_lens(1);
    assert!(
        ov.item_strings().iter().any(|s| s == "Save"),
        "Save is a Files command"
    );
    // The Recent lens (strip index 7) reads the recency vec: seed one, see it group.
    let undo = ov.rows.iter().position(|r| r.accept == "Undo").unwrap();
    ov.recent = vec![undo];
    ov.set_facet_lens(7);
    assert_eq!(ov.active_facet_id(), Some("recent"));
    assert_eq!(
        ov.item_strings(),
        vec!["Undo".to_string()],
        "only the recent command"
    );
    assert!(ov.item_sections().iter().all(|s| s == "Recent"));
}

#[test]
fn command_search_is_global_from_every_category_and_returns_to_that_category() {
    let names = crate::commands::names();
    let binds = crate::commands::effective_bindings(&[], &[]);
    let hidden = vec![false; names.len()];
    for lens in 0..8 {
        let mut ov = OverlayState::new_command(names.clone(), binds.clone(), hidden.clone());
        ov.set_facet_lens(lens);
        for ch in "bold".chars() {
            ov.push(ch);
        }
        assert!(
            ov.item_strings().iter().any(|row| row == "Bold"),
            "exact search reaches Bold from lens {lens}: {:?}",
            ov.item_strings()
        );
        for _ in 0..4 {
            ov.pop();
        }
        assert_eq!(
            ov.facet_lens, lens,
            "clearing search preserves browse category"
        );
        if lens != 0 {
            assert!(
                ov.item_strings().iter().all(|name| {
                    lens == 7
                        || crate::commands::task_category_of(name)
                            == Some(crate::commands::TaskCategory::ALL[lens - 1])
                        || crate::settings::palette_rows()
                            .iter()
                            .any(|row| lens == 6 && row.name == name)
                }),
                "clearing returns to lens {lens}"
            );
        }
    }
}

#[test]
fn every_command_union_row_has_exactly_one_non_all_browse_route() {
    let names = crate::commands::visible_names();
    let binds = crate::commands::visible_effective_bindings(&[], &[]);
    let hidden = vec![false; names.len()];
    let mut ov = OverlayState::new_command(names, binds, hidden);
    let settings = crate::settings::palette_rows();
    ov.attach_settings_rows(settings.clone(), vec![String::new(); settings.len()]);

    let mut visits = vec![0usize; ov.rows.len()];
    for lens in 1..=6 {
        ov.set_facet_lens(lens);
        for &corpus_i in &ov.items {
            visits[corpus_i] += 1;
        }
    }
    for (row, count) in ov.rows.iter().zip(visits) {
        assert_eq!(
            count, 1,
            "Commands union row {:?} must have exactly one non-All browse route",
            row.accept
        );
    }
}
