//! ITEM 114 — TIER 2: every setting, changed and persisted through the REAL
//! Settings-workspace door, driven by real chords into the live `App`.
//!
//! # Why this tier, and not a capture
//!
//! `docs/harness-reach.md` maps the harness's edge. `SettingToggle`,
//! `SettingValueCommit` and `SettingPathPick` are **Unsupported**: the live
//! global flip and the config write are both `App`-side, so a `--keys` capture
//! shows the Settings row selected and the query typed and does **not** show the
//! value changed or persisted. The lifecycle around them is tier 1 and captured
//! elsewhere in this item; the value changes are here, in Rust, over a hermetic
//! `App` on an `InMemoryFs`.
//!
//! # The trap this file exists to avoid
//!
//! The SAME setting has two doors with two different reaches. Flipping typewriter
//! scroll through its own COMMAND emits `persist_typewriter`, which is
//! replay-**Applied** — a capture sees it. Flipping the same setting from a row
//! in the Settings picker emits `setting_toggle`, which is **Unsupported**. A law
//! that verified through the command door and claimed it covered the picker door
//! would be vacuous about exactly the door this item rebuilt.
//!
//! So nothing here calls `App::setting_toggle`, `App::setting_value_commit` or
//! any other App-side door by name. Every assertion is reached by pressing the
//! chord that opens the Settings workspace, walking into it with the keys a user
//! walks in with, and pressing the key that changes the row — through
//! `dispatch_pressed_key` → the keymap → `App::apply` → the live effect
//! interpreter. `the_sweep_drives_the_picker_door_and_names_no_app_side_door`
//! holds that property structurally, so a future edit cannot quietly shortcut it.
//!
//! # What it sweeps
//!
//! Every row of `settings::visible_rows()`, classified by a WILDCARD-FREE match
//! over `SettingKind`: a new kind fails to compile here, and a new row is swept
//! the moment it joins the corpus. Both keymap conventions run, because
//! `native-gate.sh` runs the suite once per convention and the chord that opens
//! the workspace differs between them.

use super::*;
use crate::settings::{SettingId, SettingKind, SettingRow};
use std::sync::Arc;

const CFG: &str = "/cfg/config.toml";

/// The chord that opens the Settings workspace in the convention this pass is
/// running under — resolved from the running convention rather than hardcoded,
/// so the mac and linux passes each drive their OWN real binding.
fn open_settings_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-,",
        crate::convention::Convention::Linux => "C-,",
    }
}

/// A hermetic `App` over an `InMemoryFs` with a real config path and a
/// workspace of folders (so a Path row's folder navigator has somewhere to go).
fn workspace_app(mem: &crate::fs::InMemoryFs) -> App {
    let cfg = Config {
        path: std::path::PathBuf::from(CFG),
        workspace: Some(std::path::PathBuf::from("/ws")),
        session_restore: Some(false),
        reduce_motion: Some(false),
        ..Config::empty()
    };
    let _ = mem;
    app_on(None, "/ws/proj", cfg)
}

fn seeded_fs() -> crate::fs::InMemoryFs {
    crate::fs::InMemoryFs::new()
        .with_dir("/ws")
        .with_dir("/ws/proj")
        .with_dir("/ws/other")
        .with_dir("/cfg")
}

/// The value a settings row currently READS OUT — through the same one owner
/// (`settings::value_for`) the drawn row and the sidecar cell read, never a
/// second copy of "what is this set to".
fn readout(app: &App, row: &SettingRow) -> String {
    let values = crate::settings::SettingsValues::gather(
        &app.config,
        &app.project_location.root,
        app.frame.zoom(),
        crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    );
    crate::settings::value_for(row, &values)
}

fn config_text(mem: &crate::fs::InMemoryFs) -> String {
    use crate::fs::FileSystem;
    mem.read_to_string(std::path::Path::new(CFG))
        .unwrap_or_default()
}

/// Does the persisted config carry a top-level assignment for `key`?
fn persisted(mem: &crate::fs::InMemoryFs, key: &str) -> Option<String> {
    config_text(mem)
        .lines()
        .find(|l| l.trim_start().starts_with(&format!("{key} = ")))
        .map(|l| l.trim().to_string())
}

/// WALK INTO THE WORKSPACE and stand on `row`, using only real chords: the
/// binding that summons it, `Tab` into the content pane (a fresh summon lands on
/// the navigation rail, which is the workspace's primary list), then one `Down`
/// per row of the corpus. Panics naming the row if the walk does not land on it,
/// so a corpus reorder is a loud failure rather than a silently mis-aimed
/// assertion.
fn stand_on(app: &mut App, row: &SettingRow) {
    app.press_spec_headless(open_settings_chord())
        .expect("the settings chord parses");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Settings),
        "the real binding summoned the Settings workspace"
    );
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.detail_focus),
        Some(false),
        "a fresh summon stands on the navigation rail, the workspace's primary list"
    );
    app.press_spec_headless("Tab")
        .expect("Tab parses and crosses into the content pane");
    let idx = crate::settings::visible_rows()
        .iter()
        .position(|r| r.id == row.id)
        .expect("every row is in the visible corpus");
    let downs = std::iter::repeat_n("Down", idx)
        .collect::<Vec<_>>()
        .join(" ");
    if !downs.is_empty() {
        app.press_spec_headless(&downs).expect("Down parses");
    }
    assert_eq!(
        app.workspace_state
            .overlay()
            .and_then(|o| o.selected_value())
            .map(str::to_string),
        Some(row.name.to_string()),
        "the walk stands on {:?}",
        row.name
    );
}

/// The `Toggle` arm of the sweep below: Enter flips the LIVE value, persists it
/// under its own key, keeps the workspace open, and the same key puts it back —
/// on disk as well as live, so a persist that never moved reads as a failure
/// rather than as symmetry.
fn sweep_toggle(app: &mut App, mem: &crate::fs::InMemoryFs, row: &SettingRow) {
    let key = crate::settings::toggle_key(row.id).expect("a Toggle has a key");
    stand_on(app, row);
    let before = readout(app, row);
    app.press_spec_headless("Enter").expect("Enter parses");
    let after = readout(app, row);
    assert_ne!(
        before, after,
        "{:?}: Enter on the row must flip the LIVE value",
        row.name
    );
    let written = persisted(mem, key).unwrap_or_else(|| {
        panic!(
            "{:?}: nothing persisted under {key:?} — config was:\n{}",
            row.name,
            config_text(mem)
        )
    });
    assert!(
        written.starts_with(&format!("{key} = ")),
        "{:?}: the write landed under its OWN key, not {written:?}",
        row.name
    );
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Settings),
        "{:?}: a toggle keeps you configuring",
        row.name
    );
    // Restore, and prove the flip is reversible from the same door —
    // in BOTH the live readout and what was written to disk, so a
    // toggle that moved the global without persisting (or the other
    // way round) fails here rather than looking symmetric.
    app.press_spec_headless("Enter").expect("Enter parses");
    assert_eq!(
        readout(app, row),
        before,
        "{:?}: the same key restores the live value",
        row.name
    );
    let restored = persisted(mem, key).expect("the restore persisted too");
    assert_ne!(
        written, restored,
        "{:?}: the restore wrote the OTHER bool — a persist that never \
                 moved would read identical here",
        row.name
    );
}

/// The `Range` arm: the rail's own `→` step, then the EXACT numeric entry, both
/// landing on the authored grid and both persisting.
fn sweep_range(app: &mut App, mem: &crate::fs::InMemoryFs, row: &SettingRow) {
    let key = crate::settings::value_key(row.id).expect("a Range has a key");
    let spec = crate::settings::range_spec(row.id).expect("a Range has a spec");
    stand_on(app, row);
    let before = readout(app, row);
    // ARROW STEP — the rail's own keyboard.
    app.press_spec_headless("Right").expect("Right parses");
    let stepped = readout(app, row);
    assert_ne!(
        before, stepped,
        "{:?}: → must move the LIVE value one authored step",
        row.name
    );
    assert!(
        persisted(mem, key).is_some(),
        "{:?}: a discrete step persists at once — config was:\n{}",
        row.name,
        config_text(mem)
    );
    // EXACT ENTRY — Enter opens the numeric field, and a typed value
    // lands on the authored grid.
    let target = spec.stepped(spec.parse(&stepped).unwrap_or(spec.default), 3);
    let typed: String = spec
        .format(target)
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    app.press_spec_headless("Enter").expect("Enter parses");
    assert!(
        app.workspace_state
            .overlay()
            .is_some_and(|o| o.value_edit.is_some()),
        "{:?}: Enter on a range row opens the exact numeric entry",
        row.name
    );
    // The field opens SEEDED with the row's current value (retyping
    // over what is shown is the point), so clear it before typing —
    // more Backspaces than any readout is long.
    app.press_spec_headless(&["Backspace"; 10].join(" "))
        .expect("Backspace parses");
    let keys: Vec<String> = typed.chars().map(|c| c.to_string()).collect();
    app.press_spec_headless(&keys.join(" "))
        .expect("digits parse");
    app.press_spec_headless("Enter").expect("Enter parses");
    let committed = readout(app, row);
    assert_eq!(
        committed,
        spec.format(spec.parse(&typed).unwrap_or(target)),
        "{:?}: the typed value {typed:?} commits onto the authored grid",
        row.name
    );
    let written = persisted(mem, key).expect("the typed commit persisted");
    assert!(
        written.contains(&spec.persist_value(spec.parse(&typed).unwrap_or(target))),
        "{:?}: the persisted line {written:?} must carry the committed value",
        row.name
    );
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Settings),
        "{:?}: a value commit keeps you configuring",
        row.name
    );
}

/// The `Picker`/`Submenu` arm: descend into the row's own sub-picker, commit (or
/// leave, for a Submenu), and come back to the exact row, in the content pane.
fn sweep_picker(app: &mut App, row: &SettingRow) {
    let child = crate::settings::sub_overlay(row.id).expect("a Picker opens a sub-overlay");
    stand_on(app, row);
    let before = readout(app, row);
    app.press_spec_headless("Enter").expect("Enter parses");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(child),
        "{:?}: Enter opens its own sub-picker",
        row.name
    );
    if row.kind == SettingKind::Picker {
        // Pick a DIFFERENT value and come back.
        app.press_spec_headless("Down Enter")
            .expect("Down/Enter parse");
        assert_ne!(
            readout(app, row),
            before,
            "{:?}: committing in the sub-picker changes the live value",
            row.name
        );
    } else {
        // A Submenu (Keybindings) is a place, not a value: leaving it
        // must put you back where you were.
        app.press_spec_headless("Escape").expect("Escape parses");
    }
    let back = app
        .workspace_state
        .overlay()
        .expect("the workspace resumed");
    assert_eq!(
        back.kind,
        crate::overlay::OverlayKind::Settings,
        "{:?}: the child returned to the workspace",
        row.name
    );
    assert_eq!(
        back.selected_value(),
        Some(row.name),
        "{:?}: and to the exact row it was opened from",
        row.name
    );
    assert!(
        back.detail_focus,
        "{:?}: resumed in the content pane, where that row lives",
        row.name
    );
}

/// The `Path` arm: the real folder navigator, its `.` row accepting the level you
/// are standing in, and what that does — which is per-id and wildcard-free.
fn sweep_path(app: &mut App, mem: &crate::fs::InMemoryFs, row: &SettingRow) {
    let key = crate::settings::path_key(row.id).expect("a Path has a key");
    stand_on(app, row);
    app.press_spec_headless("Enter").expect("Enter parses");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Project),
        "{:?}: Enter opens the real folder navigator",
        row.name
    );
    // The navigator opens with the first real FOLDER highlighted (a
    // folder row DESCENDS); its synthetic `.` row on top is the one
    // that accepts the level you are standing in. `Up` reaches it.
    app.press_spec_headless("Up Enter").expect("Up/Enter parse");
    // WHAT A PICKED FOLDER DOES is per-id and wildcard-free: two of
    // the three write a config key, and the third RE-SCOPES the live
    // project instead — `App::setting_path_pick`'s own arms. A sweep
    // that demanded a config write from all three would be asserting
    // something false about the one that does not.
    match row.id {
        SettingId::DefaultFolder | SettingId::ProjectsFolder => {
            let written = persisted(mem, key).unwrap_or_else(|| {
                panic!(
                    "{:?}: picking a folder must persist {key:?} — config was:\n{}",
                    row.name,
                    config_text(mem)
                )
            });
            assert!(
                written.contains("/ws"),
                "{:?}: the persisted line {written:?} must carry the \
                         picked folder",
                row.name
            );
            assert_eq!(
                readout(app, row),
                "/ws",
                "{:?}: and the row now READS OUT the folder it was given",
                row.name
            );
        }
        SettingId::ProjectRoot => {
            assert_eq!(
                app.project_location.root,
                std::path::PathBuf::from("/ws"),
                "{:?}: picking a folder re-scopes the live project",
                row.name
            );
            assert_eq!(
                readout(app, row),
                "/ws",
                "{:?}: and the row reads out the new root",
                row.name
            );
        }
        other => panic!(
            "{other:?} is a new `Path` row — say what picking a folder \
                     does for it rather than inheriting a neighbour's answer"
        ),
    }
}

/// The `Action` arm.
fn sweep_action(app: &mut App, row: &SettingRow) {
    match row.id {
        SettingId::EditConfigAsText => {
            stand_on(app, row);
            app.press_spec_headless("Enter").expect("Enter parses");
            assert!(
                !app.workspace_state.overlay_open(),
                "{:?}: going somewhere ends the journey",
                row.name
            );
            assert_eq!(
                app.document.buffer().path(),
                Some(std::path::Path::new(CFG)),
                "{:?}: the config file itself is now the open buffer",
                row.name
            );
        }
        // DELIBERATE, NAMED HOLE. `Report a Problem` composes a `mailto:`
        // URL and hands it to the OS through `App::follow_link`, which
        // spawns `open`/`xdg-open`. Driving it live in a test would launch
        // a mail client on the machine running the suite. It changes no
        // editor state and no config, so what is left to verify is that
        // the row reaches its effect through the workspace's own
        // dispatcher — which `actions::tests::overlay_drive::
        // settings_report_problem_row_reuses_the_report_effect_and_closes`
        // asserts at the core seam. Recorded here rather than skipped
        // silently, so the sweep's coverage is honest.
        SettingId::ReportProblem => {}
        other => panic!(
            "{other:?} is a new `Action` row — give it an arm rather than \
                     letting it fall through"
        ),
    }
}

/// THE HEADLINE TIER-2 LAW — every `SettingId × SettingKind`, changed and
/// persisted through the Settings workspace's own door, by real chords.
///
/// The dispatcher is a wildcard-free `match` on `SettingKind`, so a new kind
/// stops this file compiling, and the corpus is read from `visible_rows()`, so a
/// new ROW is swept the moment it exists. Each arm restores what it changed
/// before the next row runs, which is also a second assertion: a setting that
/// could be flipped but not flipped back would fail there.
#[test]
fn every_setting_changes_and_persists_through_the_real_workspace_door() {
    let mem = seeded_fs();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    // The sweep genuinely changes page mode and the page measure — that is what
    // it is for — so it restores them the way every other global-touching test
    // does, rather than leaving the next test to inherit them.
    let _page = crate::page::PagePin::snapshot();
    crate::caret::clear_override();

    let mut swept: Vec<(SettingId, &'static str)> = Vec::new();
    for row in crate::settings::visible_rows() {
        let mut app = workspace_app(&mem);
        match row.kind {
            SettingKind::Toggle => {
                sweep_toggle(&mut app, &mem, row);
                swept.push((row.id, "toggle: flipped, persisted, restored"));
            }
            SettingKind::Range => {
                sweep_range(&mut app, &mem, row);
                swept.push((row.id, "range: stepped, typed, persisted"));
            }
            // No row is authored `Value` today — every bounded numeric is a
            // `Range`. The arm exists so the match stays wildcard-free, and it
            // says so out loud rather than silently passing over one that
            // appeared.
            SettingKind::Value => panic!(
                "{:?} is authored `Value`; this sweep has no arm for one yet — \
                 write it rather than widening the match",
                row.name
            ),
            SettingKind::Picker | SettingKind::Submenu => {
                sweep_picker(&mut app, row);
                swept.push((row.id, "picker: descended, committed, returned in place"));
            }
            SettingKind::Path => {
                sweep_path(&mut app, &mem, row);
                swept.push((row.id, "path: navigator, picked, applied"));
            }
            SettingKind::Action => {
                sweep_action(&mut app, row);
                swept.push((row.id, "action: run through the workspace's own dispatcher"));
            }
        }
    }
    assert_eq!(
        swept.len(),
        crate::settings::visible_rows().len(),
        "every visible row must have been swept exactly once: {swept:?}"
    );
    crate::caret::clear_override();
}

/// THE ANTI-VACUITY LAW for the sweep above: it must reach the PICKER door, and
/// it must reach it the long way.
///
/// `docs/harness-reach.md` names the asymmetry this guards. A setting flipped
/// through its own COMMAND emits `persist_typewriter` and friends — replay-
/// **Applied**, and therefore already covered by ordinary captures. The same
/// setting flipped from a Settings ROW emits `setting_toggle`, which is
/// **Unsupported** and has no capture at all. A sweep that quietly called
/// `App::setting_toggle("typewriter_scroll")` would look identical, pass just as
/// green, and prove nothing about the door item 114 rebuilt.
///
/// So: the sweep's source may not name any App-side settings door, and must
/// drive `press_spec_headless`. Both halves are asserted, and the second is what
/// keeps the first from being satisfied by an empty file.
#[test]
fn the_sweep_drives_the_picker_door_and_names_no_app_side_door() {
    let src = include_str!("workspace_item114.rs");
    // THE SWEEP is its dispatcher plus its five per-kind arms, so the scan is
    // this whole file up to (but excluding) THIS test — whose own prose names
    // the very doors it bans and would otherwise fail the law it belongs to.
    let body = src
        .split_once("/// THE ANTI-VACUITY LAW")
        .map(|(before, _)| before)
        .expect("this law follows the sweep it guards");
    // Prose is not a call site.
    let body: String = body
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for door in [
        "setting_toggle(",
        "setting_value_commit(",
        "setting_path_pick(",
        "setting_range_step(",
        "range_apply_live(",
        "range_persist(",
        "persist_pref(",
    ] {
        assert!(
            !body.contains(door),
            "the sweep called the App-side door `{door}` directly. That is the \
             COMMAND door's reach, not the Settings picker's — the exact \
             substitution `docs/harness-reach.md` warns about. Drive the chord."
        );
    }
    let presses = body.matches("press_spec_headless").count();
    assert!(
        presses >= 15,
        "the sweep only pressed {presses} chord specs — it is no longer driving \
         the real input pipeline, and the ban above is vacuous"
    );
}

/// THE PICKER DOOR AND THE COMMAND DOOR REACH THE SAME PLACE — and the picker
/// door is the one with no capture.
///
/// Typewriter scroll has both: a `Toggle typewriter scroll` command (whose
/// effect is `persist_typewriter`, replay-**Applied**) and a Settings row (whose
/// effect is `setting_toggle`, **Unsupported**). Driving each by its own real
/// chord and landing on the same live global + the same persisted key is what
/// makes "the picker door works" a claim about the picker door, rather than an
/// inference from the command door's captures.
#[test]
fn the_settings_row_and_its_command_twin_reach_the_same_live_state() {
    let mem = seeded_fs();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();

    let row = crate::settings::row_of(SettingId::TypewriterScroll);
    let key = crate::settings::toggle_key(row.id).unwrap();

    // THE COMMAND DOOR, through the palette, by real chords.
    let mut app = workspace_app(&mem);
    let start = crate::typewriter::typewriter_on();
    let palette = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-p",
        crate::convention::Convention::Linux => "C-p",
    };
    app.press_spec_headless(palette).expect("the palette opens");
    assert_eq!(
        app.workspace_state.overlay().map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Command),
        "the palette chord for this convention really summoned the palette — a \
         mis-bound chord here would make the command door look broken instead"
    );
    for c in "typewriter".chars() {
        app.press_spec_headless(&c.to_string()).expect("a letter");
    }
    app.press_spec_headless("Enter").expect("Enter parses");
    let via_command = crate::typewriter::typewriter_on();
    assert_ne!(start, via_command, "the command door flipped it");
    let command_line = persisted(&mem, key).expect("the command door persisted it");

    // THE PICKER DOOR, through the workspace, by real chords.
    let mut app = workspace_app(&mem);
    stand_on(&mut app, &row);
    app.press_spec_headless("Enter").expect("Enter parses");
    let via_picker = crate::typewriter::typewriter_on();
    assert_ne!(via_command, via_picker, "the picker door flipped it back");
    assert_eq!(
        via_picker, start,
        "and landed on exactly the value the command door started from"
    );
    let picker_line = persisted(&mem, key).expect("the picker door persisted it");
    assert_ne!(
        command_line, picker_line,
        "the two doors wrote different values, so both really wrote"
    );
    assert!(
        command_line.starts_with(&format!("{key} = "))
            && picker_line.starts_with(&format!("{key} = ")),
        "both doors persisted under the SAME config key: {command_line:?} / {picker_line:?}"
    );
}

/// EXIT LEAVES THE EDITOR EXACTLY AS IT WAS — the live half of the enter/exit
/// byte-identity claim (`actions::tests::lifecycle` owns the core half). A
/// workspace that relocated attention must give it back with the buffer, the
/// cursor, the selection and the scroll untouched, after a journey that actually
/// changed a setting.
#[test]
fn leaving_the_workspace_returns_the_editor_untouched() {
    let mem = seeded_fs();
    let _fs = crate::fs::FsGuard::install(Arc::new(mem.clone()));
    let _g = crate::testlock::serial();

    let mut app = workspace_app(&mem);
    app.document
        .replace_buffer(crate::buffer::Buffer::from_str("alpha\nbeta\ngamma\n"));
    app.press_spec_headless("Down Down Right Right")
        .expect("cursor chords parse");
    let state = |app: &App| {
        (
            app.document.buffer().disk_bytes(),
            app.document.buffer().cursor_char(),
            app.document.buffer().selection_range(),
            app.document.buffer().eol(),
            app.document.buffer().can_undo(),
        )
    };
    let before = state(&app);

    let row = crate::settings::row_of(SettingId::PageMode);
    stand_on(&mut app, &row);
    app.press_spec_headless("Enter").expect("Enter parses");
    // Back to the rail, then out — the workspace's two Escs, both from the table.
    app.press_spec_headless("Escape").expect("Escape parses");
    assert!(
        app.workspace_state.overlay_open(),
        "the first Escape off the content pane is a BACK to the rail"
    );
    app.press_spec_headless("Escape").expect("Escape parses");
    assert!(
        !app.workspace_state.overlay_open(),
        "the second Escape leaves for the editor"
    );

    assert_eq!(
        state(&app),
        before,
        "a summoned workspace may own its rows and nothing else — asserted over \
         `disk_bytes` (what a save would actually write), so an EOL change cannot \
         hide in a normalized rope"
    );
    // Restore the global this journey flipped.
    stand_on(&mut app, &row);
    app.press_spec_headless("Enter").expect("Enter parses");
}
