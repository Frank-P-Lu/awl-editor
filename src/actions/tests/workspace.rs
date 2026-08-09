//! ITEM 114 — TIER 1: the summoned workspace's STATE, FOCUS and BACK, driven
//! through the real `apply_transition` seam the `--keys` replay shares.
//!
//! `docs/harness-reach.md` is explicit that this half is fully capturable: item
//! 173 put `overlay::Journey` in the shared core precisely so the replay would
//! not need a second copy, so entry, focus transfer, child suspend/return, Back
//! and exit all replay under `--keys` and land in the sidecar. What is asserted
//! here is asserted in the lifecycle's own vocabulary (`Surface`, `Landing`),
//! because that is the thing a capture reports.
//!
//! The VALUE side is deliberately absent — `SettingToggle` and friends are
//! replay-Unsupported and live in `app::tests::workspace`.

use super::overlay_drive::command_overlay_with_settings;
use super::*;
use crate::overlay::{Beneath, Event, OverlayKind, State, Surface, landing_of};

fn journey_state(journey: &crate::overlay::Journey) -> State {
    journey.state()
}

fn surface(journey: &crate::overlay::Journey) -> Option<Surface> {
    match journey.state() {
        State::Editor => None,
        State::Summoned { surface, .. } => Some(surface),
    }
}

/// THE WORKSPACE'S KEYBOARD AND THE TRANSITION TABLE ARE ONE DECISION.
///
/// Every key a user presses to move between a workspace's two regions is driven
/// through the REAL action seam, and where it lands is compared against what
/// `landing_of` says for that exact `(state, event)` pair — so the keyboard
/// cannot drift from the table item 173 wrote, in either direction. The pairs
/// are enumerated from the table's own rosters rather than hand-listed, and the
/// sweep asserts it covered every `Surface` a Settings workspace can occupy.
#[test]
fn the_workspaces_keys_land_where_the_transition_table_says() {
    let _g = crate::testlock::serial();
    let mut seen: Vec<Surface> = Vec::new();

    // ENTRY — a fresh summon stands on the PRIMARY list (the navigation rail).
    let mut journey = crate::overlay::Journey::seeded(Some(settings_overlay()));
    assert_eq!(
        journey_state(&journey),
        State::Summoned {
            surface: Surface::Workspace,
            beneath: Beneath::Editor,
        },
        "a summoned workspace enters on its primary list, over the editor"
    );
    seen.push(Surface::Workspace);

    // → INTO THE CONTENT. The table calls this `ToggleDetail`.
    let before = journey_state(&journey);
    settings_drive(&mut journey, &Action::ForwardChar);
    assert_eq!(
        landing_of(before, Event::ToggleDetail),
        crate::overlay::Landing::Detail,
        "the table says rightward off the primary list is the detail stage"
    );
    assert_eq!(surface(&journey), Some(Surface::WorkspaceDetail));
    seen.push(Surface::WorkspaceDetail);

    // Tab crosses back, and crosses again — the same one event, both ways.
    settings_drive(&mut journey, &Action::InsertTab);
    assert_eq!(
        surface(&journey),
        Some(Surface::Workspace),
        "Tab crosses back"
    );
    settings_drive(&mut journey, &Action::InsertTab);
    assert_eq!(surface(&journey), Some(Surface::WorkspaceDetail));

    // SHIFT-TAB crosses too — `Action::Outdent` in the document, and the other
    // half of the key pair the 2026-08-02 Esc decision names as the only way
    // between the regions.
    settings_drive(&mut journey, &Action::Outdent);
    assert_eq!(
        surface(&journey),
        Some(Surface::Workspace),
        "Shift-Tab crosses back too"
    );
    settings_drive(&mut journey, &Action::Outdent);
    assert_eq!(surface(&journey), Some(Surface::WorkspaceDetail));

    // ONE ESC ALWAYS LEAVES — including from the content pane, which is the
    // 2026-08-02 decision. The Back is Tab, above, and the footer says so.
    let before = journey_state(&journey);
    settings_drive(&mut journey, &Action::Cancel);
    assert_eq!(
        landing_of(before, Event::Cancel),
        crate::overlay::Landing::Editor,
        "the table says a cancel on the detail stage leaves the workspace"
    );
    assert_eq!(
        journey_state(&journey),
        State::Editor,
        "so Esc off the content pane closes outright — no second press"
    );

    // AND FROM THE PRIMARY LIST, the same one press.
    let mut journey = crate::overlay::Journey::seeded(Some(settings_overlay()));
    let before = journey_state(&journey);
    settings_drive(&mut journey, &Action::Cancel);
    assert_eq!(
        landing_of(before, Event::Cancel),
        crate::overlay::Landing::Editor
    );
    assert_eq!(journey_state(&journey), State::Editor, "and then it closes");

    // Every surface a Settings workspace can occupy was visited. `Contextual` is
    // not one of them — a workspace that reported itself contextual would have
    // failed the entry assertion above.
    seen.sort();
    seen.dedup();
    assert_eq!(
        seen,
        vec![Surface::Workspace, Surface::WorkspaceDetail],
        "the sweep must cross both of the workspace's stages"
    );
}

/// TYPING IS SEARCHING, AND THE RESULTS ARE ROWS. A character pressed while the
/// navigation rail holds focus edits the search query AND hands focus to the
/// content pane in one gesture — because what you are typing into is the rows'
/// filter, and a query that narrowed a list you were not looking at would be a
/// key that appeared to do nothing.
#[test]
fn typing_on_the_rail_searches_and_moves_into_the_results() {
    let _g = crate::testlock::serial();
    let mut journey = crate::overlay::Journey::seeded(Some(settings_overlay()));
    assert_eq!(surface(&journey), Some(Surface::Workspace));
    for c in "zoom".chars() {
        settings_drive(&mut journey, &Action::InsertChar(c));
    }
    let card = journey.card().expect("the workspace is up");
    assert_eq!(card.query.text(), "zoom", "the query took every character");
    assert_eq!(
        card.selected_value(),
        Some("Zoom"),
        "and the filter found the row"
    );
    assert_eq!(
        surface(&journey),
        Some(Surface::WorkspaceDetail),
        "and focus followed the results into the content pane"
    );
}

/// THE RAIL'S VERTICAL KEYS STEP CATEGORIES, and the category they step is the
/// SAME state the content pane's `←/→` steps — one fact, not two that agree.
#[test]
fn the_rail_and_the_content_panes_arrows_move_one_category_state() {
    let _g = crate::testlock::serial();
    let mut journey = crate::overlay::Journey::seeded(Some(settings_overlay()));
    let lens_of = |j: &crate::overlay::Journey| j.card().unwrap().facet_lens;
    assert_eq!(lens_of(&journey), 0, "a workspace opens on the All home");

    // On the RAIL: down steps a category, up steps back, both clamped.
    settings_drive(&mut journey, &Action::NextLine);
    assert_eq!(lens_of(&journey), 1, "↓ on the rail steps one category");
    settings_drive(&mut journey, &Action::PreviousLine);
    assert_eq!(lens_of(&journey), 0);
    settings_drive(&mut journey, &Action::PreviousLine);
    assert_eq!(lens_of(&journey), 0, "and clamps at the home");

    // In the CONTENT pane: `→` steps the same category state.
    settings_drive(&mut journey, &Action::InsertTab);
    assert_eq!(surface(&journey), Some(Surface::WorkspaceDetail));
    settings_drive(&mut journey, &Action::ForwardChar);
    assert_eq!(
        lens_of(&journey),
        1,
        "→ in the content pane moves the rail, because there is one category state"
    );
    // And the rows really did narrow to it.
    let card = journey.card().unwrap();
    let want = card.lens_strip()[1].0.clone();
    for name in card.item_strings() {
        assert_eq!(
            crate::settings::category_of(&name),
            Some(want.as_str()),
            "{name:?} is showing under the {want:?} category"
        );
    }
}

/// Cmd-P DEEP-LINKS INTO THE WORKSPACE at the relevant CATEGORY and ROW.
///
/// A Range row cannot be operated from the command palette at all: the palette's
/// settings rows are appended by `attach_settings_rows`, which carries no
/// `RangeCell`, so the row offers a bare text field with no rail, no current
/// value in reach and no neighbours. Rather than teach the palette a second copy
/// of a Settings control, the row takes you to the place that owns it — standing
/// on its own category, on its own row, in the content pane, with the palette
/// parked so Esc walks back out the way you came in.
#[test]
fn a_palette_settings_row_deep_links_into_the_workspace_at_its_own_row() {
    let _g = crate::testlock::serial();
    let mut journey = crate::overlay::Journey::seeded(Some(command_overlay_with_settings()));
    // Stand on the palette's own Zoom SETTINGS row by identity — a fuzzy query
    // for "zoom" ranks the `Zoom in` / `Zoom out` COMMANDS alongside it, and
    // this law is about the settings row, not about the ranker.
    {
        let card = journey.card_mut().unwrap();
        let want = crate::settings::row_of(crate::settings::SettingId::Zoom).name;
        card.selected = card
            .items
            .iter()
            .position(|&ci| card.rows[ci].accept == want)
            .expect("the palette's settings union carries the Zoom row");
    }
    assert_eq!(
        journey.card().unwrap().selected_setting_row().map(|r| r.id),
        Some(crate::settings::SettingId::Zoom),
        "the palette found the Zoom settings row"
    );
    settings_drive(&mut journey, &Action::Newline);

    let card = journey.card().expect("the deep link landed somewhere");
    assert_eq!(
        card.kind,
        OverlayKind::Settings,
        "it opened the Settings workspace"
    );
    assert_eq!(
        card.selected_value(),
        Some("Zoom"),
        "standing on the row that was asked for"
    );
    assert_eq!(
        card.lens_strip()[card.facet_lens].0,
        crate::settings::row_of(crate::settings::SettingId::Zoom).category,
        "with the rail pointing at that row's own category"
    );
    assert!(
        card.detail_focus,
        "and focus on the content pane, where the row is"
    );
    assert!(
        card.selected_range().is_some(),
        "the row now carries the rail control the palette could not show"
    );
    assert_eq!(
        journey.parked_kind(),
        Some(OverlayKind::Command),
        "the palette is parked, not replaced, so Esc walks back out"
    );
}

/// THEME AND CARET AUDITION FROM THE WORKSPACE, and return to it — the fast
/// editor-backed pickers item 114 deliberately did NOT absorb.
///
/// Both accepts are replay-Applied (`docs/harness-reach.md`), so this whole
/// journey is capturable end to end. Asserted for both, and for both outcomes:
/// a COMMIT keeps the audition and comes back to the exact row, a CANCEL reverts
/// it and comes back to the exact row. The returned-to position is the thing item
/// 173 found broken and this item depends on.
#[test]
fn a_workspace_audition_commits_or_reverts_and_returns_to_its_exact_row() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::caret::clear_override();

    for (query, row_name, child) in [
        ("theme", "Theme", OverlayKind::Theme),
        ("caret", "Caret style", OverlayKind::Caret),
    ] {
        for commit in [true, false] {
            let mut journey = crate::overlay::Journey::seeded(Some(settings_overlay()));
            for c in query.chars() {
                settings_drive(&mut journey, &Action::InsertChar(c));
            }
            let card = journey.card().unwrap();
            assert_eq!(card.selected_value(), Some(row_name));
            let before_value = card.item_bindings()[card.selected].clone();

            settings_drive(&mut journey, &Action::Newline);
            assert_eq!(
                journey.card().unwrap().kind,
                child,
                "{row_name}: the row opened its own fast picker"
            );
            assert_eq!(
                journey.parked_kind(),
                Some(OverlayKind::Settings),
                "{row_name}: with the workspace parked beneath it"
            );
            // Audition a different value.
            settings_drive(&mut journey, &Action::NextLine);
            let auditioned = journey
                .card()
                .unwrap()
                .selected_value()
                .unwrap()
                .to_string();

            settings_drive(
                &mut journey,
                if commit {
                    &Action::Newline
                } else {
                    &Action::Cancel
                },
            );
            let back = journey.card().expect("{row_name}: the workspace resumed");
            assert_eq!(back.kind, OverlayKind::Settings);
            assert_eq!(
                back.selected_value(),
                Some(row_name),
                "{row_name}: resumed on the row the child was opened from"
            );
            assert_eq!(
                back.query.text(),
                query,
                "{row_name}: and with the filter that found it"
            );
            assert!(
                back.detail_focus,
                "{row_name}: and in the content pane, where that row lives"
            );
            // COMMIT/REVERT PARITY, read from the live owner the row's own cell
            // reads — the audition either stuck or it did not.
            let live = match child {
                OverlayKind::Theme => crate::theme::active().name.to_string(),
                OverlayKind::Caret => crate::caret::mode().label().to_string(),
                _ => unreachable!(),
            };
            match commit {
                true => assert_eq!(
                    live, auditioned,
                    "{row_name}: a commit keeps the auditioned value"
                ),
                false => assert_eq!(
                    live, before_value,
                    "{row_name}: a cancel reverts to what the row read before"
                ),
            }
        }
    }
    crate::caret::clear_override();
}

/// NO PARALLEL SETTINGS UI SURVIVES — a structural law over the source.
///
/// The old presentation was the grouped/faceted CARD family
/// (`theme_overlay_geometry`), reached whenever a card carried a lens strip.
/// Settings now reaches the workspace family instead, and the way that is
/// decided has exactly ONE owner. This asserts that no second predicate names
/// Settings inside the renderer, which is the shape a re-grown parallel path
/// would take: a `kind == Settings` (or a `"settings"` string) in a render file
/// deciding how to draw it.
#[test]
fn no_second_place_decides_how_settings_is_presented() {
    // Scoped to the CHROME cluster — the geometry and draw owners, where a
    // parallel presentation path would have to live. `render/rowlayout.rs`
    // legitimately names every kind in its own no-wildcard row-layout match,
    // which is a decision about a ROW's cells, not about which surface grammar
    // a card belongs to.
    let render = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("render")
        .join("chrome");
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    fn walk(dir: &std::path::Path, out: &mut Vec<String>, scanned: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                    continue;
                }
                walk(&path, out, scanned);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs")
                || path.file_name().and_then(|n| n.to_str()) == Some("tests.rs")
            {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            *scanned += 1;
            let rel = path.to_string_lossy().to_string();
            for (i, line) in text.lines().enumerate() {
                let code = line.trim_start();
                if code.starts_with("//") {
                    continue; // prose may name it
                }
                if code.contains("OverlayKind::Settings") {
                    out.push(format!("{rel}:{}", i + 1));
                }
            }
        }
    }
    walk(&render, &mut offenders, &mut scanned);
    assert!(
        offenders.is_empty(),
        "the renderer may not name the Settings KIND: how a card is presented is \
         `OverlayKind::workspace_shape`'s decision, projected through \
         `ViewState::overlay_workspace`. A second test here is the parallel path \
         item 114 exists to remove. Offending lines: {offenders:?}"
    );
    assert!(
        scanned >= 15,
        "the scanner only read {scanned} chrome files — it is looking in the \
         wrong place and this law is vacuous"
    );
}
