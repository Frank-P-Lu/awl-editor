//! Real-key coverage for Settings' three named focus recipients.
//!
//! The shared action laws own the pure route. These tests drive the live App
//! door and prove the same route and teaching survive keymap dispatch: Tab and
//! Shift-Tab visit Categories, Search, and Controls; typing hands attention to
//! Search; erasing never performs a hidden region transition; Esc closes.
//!
//! # Why THIS tier
//!
//! Because the report is about a KEYBOARD, and the only honest instrument for a
//! keyboard is the real one. Everything below goes through
//! `press_spec_headless` → `dispatch_pressed_key` → the real keymap →
//! `App::apply`, so the chord a user presses is the chord that runs. Both keymap
//! conventions are covered because `native-gate.sh` runs the suite once per
//! convention and every chord here is resolved from
//! `Convention::current()` rather than written out.
//!
//! The WIDTH axis is not here, and that is the point: neither the footer nor the
//! action seam takes a width, so wide and staged cannot disagree by
//! construction. `render::tests::workspace_back_width` is where that
//! construction is checked against real geometry on both sides of the
//! transition.

use super::*;
use crate::overlay::{OverlayKind, OverlayState};
use std::sync::Arc;

/// The chord that opens the Settings workspace in the convention this pass runs
/// under — resolved from the running convention, never hardcoded, so the mac and
/// linux passes each drive their OWN real binding.
fn open_settings_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-,",
        crate::convention::Convention::Linux => "C-,",
    }
}

fn seeded() -> crate::fs::InMemoryFs {
    crate::fs::InMemoryFs::new()
        .with_dir("/ws")
        .with_dir("/ws/proj")
        .with_dir("/cfg")
}

fn settings_app() -> App {
    app_on(
        None,
        "/ws/proj",
        Config {
            path: std::path::PathBuf::from("/cfg/config.toml"),
            workspace: Some(std::path::PathBuf::from("/ws")),
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..Config::empty()
        },
    )
}

/// The live card, or a panic naming the stage the walk expected to be on.
fn card(app: &App, what: &str) -> OverlayState {
    app.workspace_state
        .overlay()
        .unwrap_or_else(|| panic!("{what}: the Settings workspace must still be up"))
        .clone()
}

/// Walk in the way a user walks in: the real summon chord, then `→` off the
/// navigation rail into the content pane. Returns the App standing in the
/// content pane.
fn in_the_content_pane() -> App {
    let mut app = settings_app();
    app.press_spec_headless(open_settings_chord())
        .expect("the settings chord parses");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(OverlayKind::Settings),
        "the real binding summoned the Settings workspace"
    );
    assert!(
        !card(&app, "on summon").detail_focus,
        "a fresh summon stands on the navigation rail, the workspace's primary list"
    );
    app.press_spec_headless("Right")
        .expect("Right parses and enters the content pane");
    assert!(
        card(&app, "after →").detail_focus,
        "→ off the rail enters the content pane"
    );
    app
}

/// Does `hint` carry a cell reading exactly `glyph label`?
fn advertises(hint: &str, glyph: &str, label: &str) -> bool {
    hint.split(crate::overlay::HINT_SEP)
        .any(|cell| cell == format!("{glyph} {label}"))
}

/// **THE JOURNEY, BY REAL KEYS.** Summon, walk into the content pane, read the
/// footer, press the key the footer named, land back on the category rail.
///
/// The pressed key is READ OFF THE FOOTER's own owner rather than written here,
/// so this cannot pass by two literals agreeing with each other. Both chords for
/// that key are driven, and the workspace has to survive all of it — a Back that
/// closed the surface would be an exit, not a Back.
#[test]
fn the_advertised_route_walks_from_controls_to_categories_and_search() {
    let mem = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial();

    use crate::overlay::workspace::SettingsFocus;
    let mut app = in_the_content_pane();
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(SettingsFocus::Controls)
    );
    let hint = app.workspace_state.journey().foot_hint();
    assert!(advertises(&hint, "tab", "categories"), "{hint:?}");
    assert!(advertises(&hint, "⇧tab", "search"), "{hint:?}");

    app.press_spec_headless("Tab").expect("Tab parses");
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(SettingsFocus::Categories),
        "the forward route wraps from Controls to Categories"
    );
    app.press_spec_headless("S-Tab").expect("Shift-Tab parses");
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(SettingsFocus::Controls),
        "the reverse route wraps back to Controls"
    );
    app.press_spec_headless("S-Tab").expect("Shift-Tab parses");
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(SettingsFocus::Search),
        "the reverse route reaches Search"
    );
}

/// **THE FORMER SURPRISE, PINNED OUT.** A freshly entered content pane must not
/// teach `tab back`.
///
/// This is the assertion the user's report earns directly, and it is separate
/// from the one above on purpose: a change that added `⌫ back` while leaving
/// `tab back` beside it would satisfy the journey law and ship the very sentence
/// that was reported as strange.
///
/// It also floors the CELL COUNT. The rows line already runs to four cells and a
/// fifth overruns the card on a narrow `Bars` world, so "name the Back" is
/// answered by REPLACING the focus cell rather than by adding beside it, and a
/// regression that quietly re-grew the line would be a legibility defect on a
/// real world rather than a wording one.
#[test]
fn the_settings_controls_hint_names_every_focus_destination() {
    let mem = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial();

    let app = in_the_content_pane();
    let hint = app.workspace_state.journey().foot_hint();
    assert!(
        !hint.contains(" back"),
        "Settings teaches destinations, not an ambiguous Back: {hint:?}"
    );
    let cells = hint.split(crate::overlay::HINT_SEP).count();
    assert!(
        cells <= 6,
        "the controls line grew unexpectedly to {cells} cells: {hint:?}"
    );
    assert!(advertises(&hint, "tab", "categories"), "{hint:?}");
    assert!(advertises(&hint, "⇧tab", "search"), "{hint:?}");
    assert!(advertises(&hint, "esc", "close"), "{hint:?}");
}

/// **THE ERASE KEY IS THE QUERY'S FIRST, AND THE BACK ONLY WHEN THE QUERY IS
/// DONE WITH IT** — and the footer says so at every step, by real keys.
///
/// This is the rule the folder navigators already teach (`⌫ up` on Browse /
/// Switch project / Move to… / Export to…), and it is the reason the Back cell
/// is DERIVED rather than authored: while a filter is live the erase key is
/// busy, so the honest Back is the focus key, and the footer has to say the
/// other thing for exactly as long as that lasts. A static cell would be a lie
/// for part of every filtered journey.
#[test]
fn typing_and_erasing_stay_in_search_until_focus_is_moved_explicitly() {
    let mem = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial();

    let mut app = in_the_content_pane();
    app.press_spec_headless("z o o m").expect("typing parses");
    let typed = card(&app, "with a query typed");
    assert_eq!(
        typed.query.text(),
        "zoom",
        "the real keys reached the query"
    );
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(crate::overlay::workspace::SettingsFocus::Search),
        "typing hands Controls to Search"
    );

    // FOUR ERASES DRAIN THE QUERY AND CHANGE NOTHING ELSE — the field's own
    // work, still the field's.
    for n in 1..=4 {
        app.press_spec_headless("Backspace")
            .expect("Backspace parses");
        let now = card(&app, "mid-drain");
        assert!(
            now.detail_focus,
            "erase {n} of 4 must still be editing the query, not navigating"
        );
        assert_eq!(now.query.text().chars().count(), 4 - n, "erase {n} of 4");
    }

    // An extra erase is still a query edit, never an implicit region change.
    app.press_spec_headless("Backspace")
        .expect("Backspace parses");
    assert_eq!(
        app.workspace_state.journey().settings_focus(),
        Some(crate::overlay::workspace::SettingsFocus::Search)
    );
    assert!(card(&app, "after the fifth erase").query.text().is_empty());
    let hint = app.workspace_state.journey().foot_hint();
    assert!(advertises(&hint, "⇧tab", "categories"), "{hint:?}");
}
