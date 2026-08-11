//! **THE WORKSPACE'S BACK IS A KEY OF ITS OWN, AND `TAB` IS NOT TAUGHT AS ONE.**
//!
//! # The report this file answers
//!
//! `Tab` returning from the Settings content pane to its category rail is
//! strange, and it is *most* strange below `workspace_is_wide`, where the layout
//! stages: the rail you are "returning focus to" is not merely unfocused there,
//! it is off screen. `Tab` is a FOCUS key. A focus key reads as a Back only while
//! both regions are visible at once, and half of a workspace's reachable widths
//! do not show both.
//!
//! # What could not be changed, and why
//!
//! `Esc` is the obvious Back and it is spoken for. The user settled it once for
//! BOTH workspace members on 2026-08-02: ONE ESC ALWAYS LEAVES, from either
//! region, so that Esc cannot mean two different things depending on where focus
//! sits. That decision stands untouched here — `actions::tests::workspace_esc`
//! is its law and still passes — and this file is deliberately the *other* half
//! of the bill it left: with Esc leaving, the Back has to be named, and the key
//! it names has to be one whose ordinary meaning survives the staged layout.
//!
//! # What the Back is
//!
//! `⌫`, under exactly the rule awl's folder navigators already teach as `⌫ up`:
//! it belongs to the search field until the field is empty, and it goes up a
//! level the moment it is. `crate::overlay::workspace::BackKey` is the one owner
//! of that answer; the footer and the action seam both read it, so what is
//! advertised and what acts cannot come apart. `Tab`/`Shift-Tab` still cross —
//! nothing about the action model or the wide layout changed — they are simply
//! no longer the sentence the footer teaches.
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
use crate::overlay::workspace::BackKey;
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

/// Every real chord spec that reaches one [`BackKey`]. Two per key, because the
/// keyboard offers two of each, and a footer cell true for only the one an
/// author happened to try is a footer cell that is false for a user.
fn chords_for(back: BackKey) -> [&'static str; 2] {
    match back {
        BackKey::Erase => ["Backspace", "M-Backspace"],
        BackKey::Focus => ["Tab", "S-Tab"],
    }
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
fn the_advertised_back_key_walks_from_the_settings_content_pane_to_its_rail() {
    let mem = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial();

    let app = in_the_content_pane();
    let stood = card(&app, "in the content pane");
    let back = stood
        .detail_back()
        .expect("the content pane must have SOME Back — Esc leaves, it does not come back");
    assert_eq!(
        back,
        BackKey::Erase,
        "with an empty query the erase key is free, so ⌫ is the Back a user is taught. \
         The focus key is the fallback for a live query, not the default."
    );
    let hint = stood.foot_hint();
    assert!(
        advertises(&hint, back.glyph(), "back"),
        "the content pane must NAME its Back — awl's footer is its only statement of what a \
         key does, and there is no accessibility tree behind it. got {hint:?}"
    );

    for chord in chords_for(back) {
        let mut app = in_the_content_pane();
        app.press_spec_headless(chord)
            .unwrap_or_else(|e| panic!("{chord} parses: {e}"));
        let after = card(&app, &format!("after {chord}"));
        assert!(
            !after.detail_focus,
            "{chord}: the advertised `{} back` must land on the category rail",
            back.glyph()
        );
        assert_eq!(
            after.kind,
            OverlayKind::Settings,
            "{chord}: a Back comes back — it does not leave the workspace"
        );
    }
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
fn the_settings_content_pane_does_not_teach_tab_as_back() {
    let mem = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem));
    let _g = crate::testlock::serial();

    let app = in_the_content_pane();
    let hint = card(&app, "in the content pane").foot_hint();
    assert!(
        !advertises(&hint, BackKey::Focus.glyph(), "back"),
        "the Settings content pane still teaches `{} back`. Below workspace_is_wide the rail \
         it returns focus to is OFF SCREEN, so a focus key is being taught as a Back for a \
         region the user cannot see. got {hint:?}",
        BackKey::Focus.glyph()
    );
    let cells = hint.split(crate::overlay::HINT_SEP).count();
    assert!(
        cells <= 4,
        "the rows line grew to {cells} cells ({hint:?}) — a fifth overruns the card on a \
         narrow Bars world, so the Back is named by REPLACING a cell, never by adding one"
    );

    // AND TAB STILL CROSSES. The action model did not change: Tab is the focus
    // key and on a wide stage it is exactly what a user reaches for. What
    // changed is only what the footer TEACHES. Asserting this here is what keeps
    // the law above from being satisfiable by deleting the focus transfer.
    let mut app = in_the_content_pane();
    app.press_spec_headless("Tab").expect("Tab parses");
    assert!(
        !card(&app, "after Tab").detail_focus,
        "Tab must still cross between the two regions — the footer stopped naming it, the \
         keyboard did not stop offering it"
    );
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
fn a_live_query_keeps_the_erase_key_and_the_footer_follows_it_back() {
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
        typed.detail_back(),
        Some(BackKey::Focus),
        "with a live query the erase key belongs to the field, so the focus key is the \
         honest Back"
    );
    assert!(
        advertises(&typed.foot_hint(), BackKey::Focus.glyph(), "back"),
        "and the footer says so: {:?}",
        typed.foot_hint()
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

    // THE FOOTER HANDS THE CELL BACK the instant the field is empty…
    let drained = card(&app, "with the query drained");
    assert_eq!(
        drained.detail_back(),
        Some(BackKey::Erase),
        "an empty query releases the erase key"
    );
    assert!(
        advertises(&drained.foot_hint(), BackKey::Erase.glyph(), "back"),
        "…and the footer hands the cell back with it: {:?}",
        drained.foot_hint()
    );

    // …and the NEXT erase goes back, which is the whole grammar in one press.
    app.press_spec_headless("Backspace")
        .expect("Backspace parses");
    assert!(
        !card(&app, "after the fifth erase").detail_focus,
        "the erase after the last character is the Back — the same one press the folder \
         navigators spend to go up a level"
    );
}
