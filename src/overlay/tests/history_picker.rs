use super::super::*;
use super::*;

#[test]
fn history_picker_lists_versions_navigates_and_carries_ids() {
    let mut ov = OverlayState::new_history(history_rows(), None, None);
    assert_eq!(ov.kind.as_str(), "history");
    // The top (newest) row is selected; its restore id is the accept value.
    assert_eq!(ov.selected_history_id(), Some("300"));
    // NAVIGATE down -> the selected id tracks the highlighted version.
    ov.move_sel(1);
    assert_eq!(ov.selected_history_id(), Some("200"));
    ov.move_sel(1);
    assert_eq!(ov.selected_history_id(), Some("100"));
    // No git / dir markers on the version rows.
    assert!(
        ov.item_strings()
            .iter()
            .all(|s| !s.contains('•') && !s.ends_with('/'))
    );
    // ITEM 116c: restore moved behind the deliberate SHIFT-HELD accept — bare
    // `↵` only opens the comparison (the same door `Tab` already is), and
    // `⇧↵` is the one restore door, footer-taught with its own glyph (Shift
    // reads the same on both conventions, so it needs no chord resolution).
    assert_eq!(
        OverlayKind::History.hint(),
        "type to filter   \u{21B5} compare   \u{21E7}\u{21B5} restore   \u{2190}/\u{2192} lens"
    );
    assert!(ov.foot_hint().contains("restore"));
}

#[test]
fn history_picker_groups_by_session_and_today_with_injected_now() {
    const DAY: u64 = 86_400_000;
    let now = 100 * DAY + 5_000;
    let session_start = 100 * DAY + 3_000; // this session began mid-day 100
    let row = |id: &str, ts: u64| crate::history::TimelineRow {
        when: "x".to_string(),
        which: String::new(),
        counts: "+0 −0".to_string(),
        id: id.to_string(),
        timestamp: ts,
        pinned: false,
        name: None,
    };
    let rows = vec![
        row("a", 100 * DAY + 4_000), // today AND in this session
        row("b", 100 * DAY + 1_000), // today, but before this session started
        row("c", 99 * DAY + 1_000),  // yesterday
    ];
    let mut ov = OverlayState::new_history(rows, Some(now), Some(session_start));
    // Lands on All (every version); strip is All-first.
    assert_eq!(ov.active_facet_id(), Some("all"));
    assert_eq!(ov.items.len(), 3);
    // → Session lens: only "a" (at/after session start).
    ov.cycle_lens(1);
    assert_eq!(ov.active_facet_id(), Some("session"));
    let history_id = |ov: &OverlayState, ci: usize| match &ov.rows[ci].meta {
        RowMeta::History { id, .. } => id.clone(),
        _ => panic!("history row must carry RowMeta::History"),
    };
    let session_ids: Vec<String> = ov.items.iter().map(|&ci| history_id(&ov, ci)).collect();
    assert_eq!(session_ids, vec!["a".to_string()]);
    assert!(ov.item_sections().iter().all(|s| s == "Session"));
    // → Today lens: "a" and "b" (same calendar day), never yesterday's "c".
    ov.cycle_lens(1);
    assert_eq!(ov.active_facet_id(), Some("today"));
    let today_ids: Vec<String> = ov.items.iter().map(|&ci| history_id(&ov, ci)).collect();
    assert_eq!(today_ids, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn history_time_lenses_are_inert_headless_no_clock() {
    // With no reference clock (the headless capture path), Session/Today group
    // NOTHING — the determinism gate — so those lenses show an empty list.
    let mut ov = OverlayState::new_history(history_rows(), None, None);
    ov.cycle_lens(1); // Session
    assert_eq!(ov.active_facet_id(), Some("session"));
    assert!(ov.items.is_empty(), "Session inert with no clock");
    ov.cycle_lens(1); // Today
    assert_eq!(ov.active_facet_id(), Some("today"));
    assert!(ov.items.is_empty(), "Today inert with no clock");
}

#[test]
fn history_rows_show_when_dot_which_and_counts_ride_the_faint_column() {
    // The MAIN column composes "when · which" (the bare when for an empty
    // which); the faint right column carries the "+N −M" changed-counts —
    // the existing binding-column pattern, zero new layout.
    let ov = OverlayState::new_history(history_rows(), None, None);
    assert_eq!(
        ov.item_strings(),
        vec![
            "just now · fix: the engine",
            "2 min ago · edited \"Two flows\"",
            "1 hr ago",
        ]
    );
    assert_eq!(ov.item_bindings(), vec!["+0 −0", "+0 −1", "+1 −2"]);
    // The composed corpus is what the fuzzy filter matches, so a SUBJECT
    // query finds its version (a free win of the composition).
    let mut ov = OverlayState::new_history(history_rows(), None, None);
    for c in "engine".chars() {
        ov.push(c);
    }
    assert_eq!(ov.item_strings().len(), 1);
    assert_eq!(ov.selected_history_id(), Some("300"));
}

#[test]
fn history_picker_marks_a_pinned_version_in_the_secondary_column() {
    // THE CONSCIOUS MARK: a KEPT (pinned) version wears the calm "pinned" tag
    // AHEAD of its changed-count in the faint secondary column (`item_bindings`
    // — the exact source the sidecar's `overlay.bindings` folds from), while an
    // un-pinned version stays bare. The count is never dropped for the tag.
    let mk = |id: &str, pinned: bool| crate::history::TimelineRow {
        when: "just now".to_string(),
        which: String::new(),
        counts: "+0 −1".to_string(),
        id: id.to_string(),
        timestamp: id.parse().unwrap_or(0),
        pinned,
        name: None,
    };
    let ov = OverlayState::new_history(vec![mk("2", true), mk("1", false)], None, None);
    let binds = ov.item_bindings();
    assert!(
        binds[0].contains(PIN_TAG),
        "the pinned row is marked: {:?}",
        binds[0]
    );
    assert!(
        binds[0].contains("+0 −1"),
        "and keeps its changed-count: {:?}",
        binds[0]
    );
    assert!(
        !binds[1].contains(PIN_TAG),
        "an un-pinned row stays bare: {:?}",
        binds[1]
    );
}

#[test]
fn history_picker_named_row_shows_name_primary_and_demotes_the_timestamp() {
    // NAMED SAVE POINTS: a named row's PRIMARY cell is the NAME itself (the
    // fuzzy corpus too — typing the name finds it), with the timestamp DEMOTED
    // beside the changed-count in the faint secondary column ("when · +N −M").
    // The redundant "pinned" tag is dropped for a named row (the name IS the
    // conscious mark); an unnamed sibling — pinned or not — keeps the exact
    // pre-name shape. Same corpus/bindings columns, no new layout path.
    let mk = |id: &str, pinned: bool, name: Option<&str>| crate::history::TimelineRow {
        when: "2 hr ago".to_string(),
        which: "edited \"Title\"".to_string(),
        counts: "+3 −1".to_string(),
        id: id.to_string(),
        timestamp: id.parse().unwrap_or(0),
        pinned,
        name: name.map(str::to_string),
    };
    let ov = OverlayState::new_history(
        vec![
            mk("3", true, Some("draft A")),
            mk("2", true, None),
            mk("1", false, None),
        ],
        None,
        None,
    );
    // Primary cells: name for the named row, "when · which" for the rest.
    assert_eq!(ov.rows[0].accept, "draft A", "the name IS the primary cell");
    assert_eq!(
        ov.rows[1].accept, "2 hr ago · edited \"Title\"",
        "unnamed rows unchanged"
    );
    // Secondary cells: timestamp demoted for the named row; pin tag only on the
    // unnamed pinned row.
    let binds = ov.item_bindings();
    assert_eq!(
        binds[0], "2 hr ago · +3 −1",
        "timestamp + count demoted to secondary"
    );
    assert!(
        !binds[0].contains(PIN_TAG),
        "no redundant pin tag on a named row"
    );
    assert_eq!(
        binds[1],
        format!("{PIN_TAG} · +3 −1"),
        "unnamed pinned row keeps its tag"
    );
    assert_eq!(binds[2], "+3 −1", "plain row untouched");
    // The restore ids stay parallel — Enter/Tab on a named row reach id "3".
    let ids: Vec<&str> = ov
        .rows
        .iter()
        .map(|r| match &r.meta {
            RowMeta::History { id, .. } => id.as_str(),
            _ => panic!("history row must carry RowMeta::History"),
        })
        .collect();
    assert_eq!(ids, vec!["3", "2", "1"]);
    // Typing the NAME finds the named row (it rides the fuzzy corpus).
    let mut ov2 = ov.clone();
    for c in "draft".chars() {
        ov2.push(c);
    }
    assert_eq!(ov2.item_strings().len(), 1, "the name is fuzzy-findable");
    assert_eq!(ov2.selected_history_id(), Some("3"));
}

#[test]
fn history_picker_empty_state_shows_calm_row_and_no_op_accept() {
    // No versions -> an empty-corpus picker that summons but lists nothing; the
    // SHARED empty-state owner supplies the calm "no history yet" message row,
    // and every accept path already no-ops on an empty item list.
    let ov = OverlayState::new_history(Vec::new(), None, None);
    assert_eq!(ov.kind.as_str(), "history");
    assert!(
        ov.item_strings().is_empty(),
        "empty corpus lists no real rows"
    );
    assert_eq!(
        ov.empty_notice().as_deref(),
        Some("no history yet"),
        "the shared empty-state supplies History's calm message"
    );
    assert_eq!(
        ov.selected_history_id(),
        None,
        "nothing to restore on empty"
    );
}
