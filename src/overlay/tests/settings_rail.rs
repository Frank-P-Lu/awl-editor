use super::super::*;
use super::*;

// ── ITEM 94 — the RANGE CELL column (the row model half) ─────────────────────

fn settings_values(zoom: f32) -> crate::settings::SettingsValues {
    crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom,
        scroll_sensitivity: 1.0,
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

fn settings_with_rails(zoom: f32) -> OverlayState {
    let vals = settings_values(zoom);
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        vec![],
        vec![],
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov
}

/// The rail column is parallel to rows and derived through each range spec.
#[test]
fn item_range_fracs_are_parallel_to_the_rows_and_derived_from_the_spec() {
    let _g = crate::testlock::serial();
    let ov = settings_with_rails(1.4);
    let fracs = ov.item_range_fracs();
    let names = ov.item_strings();
    assert_eq!(
        fracs.len(),
        names.len(),
        "the rail column is parallel to the rows"
    );
    let visible_rows = crate::settings::visible_rows();
    for (name, frac) in names.iter().zip(&fracs) {
        let row = visible_rows
            .iter()
            .find(|row| row.name == name)
            .expect("visible overlay name belongs to a setting");
        if let Some(spec) = crate::settings::range_spec(row.id) {
            let f = frac.expect("every Range row carries a rail");
            let value = crate::settings::range_value(row.id, &settings_values(1.4)).unwrap();
            assert!(
                (f - spec.frac_of(value)).abs() < 1e-6,
                "the thumb is the spec's fraction"
            );
        } else {
            assert!(frac.is_none(), "{name} must carry no rail");
        }
    }
    // The row's own secondary TEXT and its rail agree (one gathered instant).
    let zi = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .unwrap();
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    assert_eq!(ov.item_bindings()[zi], spec.format(1.4));
    assert_eq!(ov.range_of_item(zi).unwrap().step, spec.step_of(1.4));
}

/// A picker without range rows reports an empty rail column.
#[test]
fn a_picker_without_range_rows_reports_an_empty_rail_column() {
    let _g = crate::testlock::serial();
    let ov = OverlayState::new(OverlayKind::Goto, corpus(), vec![], vec![]);
    assert!(ov.item_range_fracs().is_empty());
    assert!(ov.range_of_item(0).is_none());
    assert!(ov.selected_range().is_none());

    let mut settings = settings_with_rails(1.0);
    assert!(!settings.item_range_fracs().is_empty());
    settings.set_range_cells(Vec::new());
    assert!(
        settings.item_range_fracs().is_empty(),
        "an empty cell list clears every rail rather than leaving a stale one"
    );
}

/// `set_selected_range` moves the selected step and readout together.
#[test]
fn set_selected_range_moves_the_selected_rows_step_and_readout_together() {
    let _g = crate::testlock::serial();
    let spec = crate::settings::range_spec(crate::settings::SettingId::Zoom).unwrap();
    let mut ov = settings_with_rails(1.0);
    let zi = ov
        .items
        .iter()
        .position(|&i| ov.rows[i].accept == "Zoom")
        .unwrap();
    ov.selected = zi;
    let others: Vec<String> = ov.item_bindings();

    let next = spec.stepped(1.0, 3);
    ov.set_selected_range(spec.step_of(next), spec.format(next));
    assert_eq!(ov.selected_range().unwrap().step, spec.step_of(next));
    assert_eq!(ov.item_bindings()[zi], spec.format(next));
    assert!((ov.item_range_fracs()[zi].unwrap() - spec.frac_of(next)).abs() < 1e-6);
    // Every OTHER row's cell is untouched.
    for (i, (before, after)) in others.iter().zip(ov.item_bindings()).enumerate() {
        if i != zi {
            assert_eq!(*before, after, "row {i}'s cell must not move");
        }
    }
    // On a NON-range row it is a calm no-op (no panic, no stray write).
    ov.selected = 0;
    let cell0 = ov.item_bindings()[0].clone();
    ov.set_selected_range(0, "nope".into());
    assert_eq!(ov.item_bindings()[0], cell0);
}

/// THE FOOT LINE FOLLOWS THE SELECTION, and its wording is pinned here. Every
/// authored rail row reads `adjust`; every ordinary row reads `category` (item
/// 114 — on a workspace the lens IS the navigation rail's category). The rest of
/// The sweep is derived from the complete settings registry, so adding a Range row
/// cannot inherit a stale neighbour assumption. (The keys-vs-hint OUTCOME sweep is
/// `actions::tests::overlay_drive::the_foot_hint_names_what_left_right_actually_do_on_every_settings_row`.)
#[test]
fn the_settings_foot_hint_says_adjust_only_while_a_rail_row_is_selected() {
    let _g = crate::testlock::serial();
    // ITEM 114 — this law is about the ROWS' keys, and the rows are the
    // workspace's DETAIL stage. Focus is moved through the LIFECYCLE (the one
    // writer of that bit), not by assignment, which is also why the card is held
    // inside a `Journey` here.
    let mut journey = crate::overlay::Journey::seeded(Some(settings_with_rails(1.0)));
    journey.toggle_detail();
    let ov = journey.card_mut().unwrap();
    let visible = crate::settings::visible_rows();
    assert_eq!(
        visible.len(),
        ov.items.len(),
        "law sweeps every visible row"
    );
    for (selected, row) in visible.iter().enumerate() {
        let ov = journey.card_mut().unwrap();
        ov.selected = selected;
        let expected = if row.kind == crate::settings::SettingKind::Range {
            OverlayKind::Settings.range_row_hint()
        } else {
            OverlayKind::Settings.hint()
        };
        assert_eq!(
            ov.foot_hint(),
            expected,
            "{:?} advertises exactly its authored interaction kind",
            row.id
        );
    }
    assert!(
        visible
            .iter()
            .any(|row| row.id == crate::settings::SettingId::ScrollSensitivity),
        "the exhaustive sweep includes the new Scroll sensitivity row"
    );
    // The two variants differ in EXACTLY the ←/→ cell.
    let plain_line = OverlayKind::Settings.hint();
    let ranged_line = OverlayKind::Settings.range_row_hint();
    let plain: Vec<&str> = plain_line.split(HINT_SEP).collect();
    let ranged: Vec<&str> = ranged_line.split(HINT_SEP).collect();
    assert_eq!(plain.len(), ranged.len(), "the range variant adds no cells");
    for (a, b) in plain.iter().zip(&ranged) {
        if a != b {
            assert_eq!(*a, "\u{2190}/\u{2192} category");
            assert_eq!(*b, "\u{2190}/\u{2192} adjust");
        }
    }
}
