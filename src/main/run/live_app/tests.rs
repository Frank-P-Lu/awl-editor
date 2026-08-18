//! The live-`App` capture mode's own laws. Carved out of `live_app.rs` as a
//! sibling `tests.rs` so the production file stays under the size ceiling —
//! `code-health.py`'s `production()` exempts that exact filename precisely so
//! carving an inline `mod tests` out is a real remedy rather than moving lines
//! from one measured file to another.

use super::*;
use crate::capture::CaptureOpts;
use crate::config::Config;
use crate::settings::SettingId;
use crate::testscratch::ScratchDir;
use std::sync::Arc;

const CFG: &str = "/cfg/config.toml";

/// The chord that summons the Settings workspace in the convention this
/// pass is running under — resolved from the RUNNING convention rather than
/// hardcoded, because `native-gate.sh` runs the suite once per convention
/// and each pass must drive its own real binding.
fn open_settings_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-,",
        crate::convention::Convention::Linux => "C-,",
    }
}

fn new_document_chord() -> &'static str {
    match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-n",
        crate::convention::Convention::Linux => "C-n",
    }
}

/// The real chords a user walks in with: summon the workspace, `Tab` from
/// the navigation rail into the content pane, one `Down` per row of the
/// corpus, then `Enter` on the row. The row INDEX is derived from
/// `settings::visible_rows()`, so a corpus reorder re-aims this walk instead
/// of silently landing on a neighbour.
fn walk_to(id: SettingId) -> Vec<crate::keyspec::Chord> {
    let idx = crate::settings::visible_rows()
        .iter()
        .position(|r| r.id == id)
        .expect("the row is in the visible corpus");
    let downs = std::iter::repeat_n("Down", idx)
        .collect::<Vec<_>>()
        .join(" ");
    let spec = format!("{} Tab {downs} Enter", open_settings_chord());
    crate::keyspec::parse_chords(&spec).expect("the walk parses")
}

fn proj() -> std::path::PathBuf {
    std::path::PathBuf::from("/ws/proj")
}

/// A config with a real (sandbox) path, so a settings persist has somewhere
/// to land — the write is part of what the live door actually performs.
/// The live-`App` payload these tests drive, over the scratch config.
fn spec(keys: Vec<crate::keyspec::Chord>, root: Option<PathBuf>) -> LiveAppSpec {
    LiveAppSpec {
        file: None,
        keys,
        root,
        workspace: None,
        config: cfg(),
        canvas: None,
        dpi: None,
    }
}

fn cfg() -> Config {
    Config {
        path: std::path::PathBuf::from(CFG),
        ..Config::empty()
    }
}

/// Run `body` over a seeded `InMemoryFs`, so nothing in these laws — the App's
/// config persist included — can reach the real disk. The PNG + JSON still go
/// out through `std::fs`, the documented capture-deliverable bypass.
/// [`in_sandbox`] with one document already seeded — for the laws that need
/// the App to OPEN a real (sandbox) file and write it back.
fn in_sandbox_with<T>(doc: &std::path::Path, body: impl FnOnce() -> T) -> T {
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(doc, "# Probe\n\nSome prose.\n"),
    );
    crate::fs::with_fs(mem, body)
}

fn in_sandbox<T>(body: impl FnOnce() -> T) -> T {
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj"),
    );
    crate::fs::with_fs(mem, body)
}

fn sidecar(png: &std::path::Path) -> serde_json::Value {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("a sidecar");
    serde_json::from_str(&text).expect("valid sidecar JSON")
}

/// The selected row's VALUE cell, straight out of the sidecar's parallel
/// `overlay.bindings` array (`opts.rs`: a Settings row carries its readout
/// there as text).
fn selected_value(v: &serde_json::Value) -> String {
    let o = &v["overlay"];
    let i = o["selected_index"].as_u64().expect("a selected row") as usize;
    o["bindings"][i].as_str().expect("a value cell").to_string()
}

fn selected_name(v: &serde_json::Value) -> String {
    let o = &v["overlay"];
    let i = o["selected_index"].as_u64().expect("a selected row") as usize;
    o["items"][i].as_str().expect("a row label").to_string()
}

#[test]
fn live_app_first_new_document_asks_for_a_folder_before_creating_a_file() {
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-live-app-{}", std::process::id())));
    let png = dir.join("folder-choice.png");
    let json = in_sandbox(|| {
        capture_live_app(
            png.clone(),
            spec(
                crate::keyspec::parse_chords(new_document_chord()).unwrap(),
                Some(crate::fs::data_root()),
            ),
        )
        .unwrap();
        sidecar(&png)
    });
    assert_eq!(json["driver"].as_str(), Some("live-app"));
    assert_eq!(json["overlay"]["mode"].as_str(), Some("goto"));
}

/// THE PRIMARY LAW — the transition converted from
/// Rust-only to sidecar-provable.
///
/// Picking "Emacs" in the KEYMAP picker (Enter on the Settings "Keymap" row
/// descends into `OverlayKind::Keymap`; `KeymapFlavor::ALL` is `[Native,
/// Emacs]`, so one `Down` then `Enter` accepts Emacs) is the hardest case in
/// the live-only census, not a convenient one: `docs/harness-reach.md` lists
/// `overlay_accept:Keymap` as **Unsupported** under every filesystem
/// capability, because applying the picked flavor needs a LIVE keymap
/// rebuild no filesystem capability can supply. The transition is provable
/// only in Rust otherwise (`app::tests::files::
/// keymap_picker_accept_applies_persists_notifies_and_live_reapplies`,
/// asserting `App` + config state directly).
///
/// BOTH HALVES ARE ASSERTED IN ONE TEST, and the second is what makes the
/// first mean anything: the SAME chord spec is driven through the ordinary
/// `--keys` capture door, which must still report the OLD flavor and record
/// the skip. A live-App sidecar that agreed with the replay sidecar would
/// prove nothing had been reached at all.
///
/// ONLY MEANINGFUL ON `Convention::Linux` — the flavor is structurally inert
/// on `Convention::Mac` (`crate::keymap::KeymapFlavor`'s doc), so the "Keymap"
/// row is hidden from `visible_rows()` there entirely
/// (`settings::row_available_on`). `Convention::current()` is process-frozen
/// (`AWL_CONVENTION_FORCE` read once, memoized), so this test branches on the
/// ambient value rather than forcing one: `native-gate.sh`'s two-convention
/// run is what exercises BOTH this law (Linux pass) and the row's absence
/// (Mac pass, asserted in the `else` branch below) across the two runs.
#[test]
fn a_live_app_capture_photographs_a_keymap_pick_an_ordinary_capture_cannot_see() {
    let _g = crate::testlock::serial();
    if crate::convention::Convention::current() != crate::convention::Convention::Linux {
        // THE MACOS-INERT-HIDES-THE-ROW PROOF: on this convention there is
        // nothing to walk to at all — the settings row itself is gone.
        assert!(
            !crate::settings::visible_rows()
                .iter()
                .any(|r| r.id == SettingId::Keymap),
            "Keymap must be hidden from the Settings menu on Convention::Mac"
        );
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-replay-{}", std::process::id())),
    );
    let mut keys = walk_to(SettingId::Keymap);
    keys.extend(crate::keyspec::parse_chords("Down Enter").expect("the pick-Emacs tail parses"));

    // ── THE LIVE-`App` CAPTURE ────────────────────────────────────────
    let live = dir.join("live.png");
    let live_json = in_sandbox(|| {
        capture_live_app(live.clone(), spec(keys.clone(), Some(proj())))
            .expect("the live-App capture needs a GPU adapter");
        sidecar(&live)
    });
    assert_eq!(
        live_json["overlay"]["active"].as_bool(),
        Some(true),
        "the sidecar was folded AFTER the chords were driven — the walk \
         resumed the Settings card the picker was opened from"
    );
    assert_eq!(
        selected_name(&live_json),
        "Keymap",
        "the picker's accept resumed the Settings row it was opened from"
    );
    assert_eq!(
        live_json["driver"].as_str(),
        Some("live-app"),
        "the sidecar names the tier that produced it"
    );
    assert_eq!(
        live_json["project"]["keymap_flavor"].as_str(),
        Some("emacs"),
        "THE CONVERTED CLAIM: the live keymap pick is readable from the \
         sidecar's project block, not only from a Rust assertion"
    );
    assert_eq!(
        selected_value(&live_json),
        "Emacs",
        "and the row's own value cell agrees, in plain language — two \
         independent witnesses in one artifact"
    );
    assert!(
        live_json["replay_skips"]
            .as_array()
            .is_some_and(|a| a.is_empty()),
        "a live-App capture skips nothing: it performs the effect"
    );

    // ── THE SAME SPEC THROUGH THE ORDINARY `--keys` DOOR ──────────────
    // Anti-vacuity. This is the capture that could NOT witness the pick,
    // and it must still be unable to.
    let replay = dir.join("replay.png");
    let replay_json = in_sandbox(|| {
        super::super::capture_screenshot(
            replay.clone(),
            None,
            CaptureOpts::default(),
            keys,
            crate::keymap::KeymapState::new(),
            Some(proj()),
            None,
            std::path::PathBuf::from("/ws/notes"),
            cfg(),
            false,
        )
        .expect("the ordinary capture succeeds");
        sidecar(&replay)
    });
    assert_eq!(
        replay_json["driver"].as_str(),
        Some("replay"),
        "the ordinary door stamps the shared-core tier"
    );
    assert_eq!(
        selected_name(&replay_json),
        "Keymap",
        "the ordinary replay walked to the same row — the specs are \
         identical, and the journey resume is core-level (it happens \
         whether or not the accept effect itself gets applied)"
    );
    assert_eq!(
        replay_json["project"]["keymap_flavor"].as_str(),
        Some("native"),
        "the ordinary capture still cannot see the pick — if this ever reads \
         `emacs`, the live-App law above has stopped proving anything new"
    );
    assert_eq!(
        replay_json["replay_skips"],
        serde_json::json!([{ "effect": "overlay_accept", "action": "Newline" }]),
        "and it says so out loud rather than reporting stale state silently"
    );
}

/// **A CAPTURE CARRIES A TOAST**, and the sidecar's two answers
/// about it agree.
///
/// The defect this closes was not "the notice looked wrong": no capture door
/// could SEE the notice channel at all. `CaptureOpts` had no slot for it, so a
/// real headless `App` that had genuinely raised `saved` produced a PNG
/// **byte-identical** to one that had raised nothing — while the SAME
/// sidecar's `semantic` block, which reads the `App` directly, announced that
/// notice to a screen reader. One artifact, two answers. Every "the notice is
/// set" claim in this tree was a sidecar claim, never a photographed one.
///
/// So this law asserts both halves, and the SECOND is what makes the first
/// mean something:
///
///   1. the two PNGs differ — the notice is on the frame;
///   2. the top-level `notice` block and the `semantic` tree's own status node
///      report the SAME sentence, so the artifact cannot say one thing to a
///      reader and another to an assistive client.
///
/// A byte-identity assertion is the exact shape of the bug, which is why it is
/// spelled that way rather than as "the notice block is non-null".
#[test]
fn a_live_app_capture_photographs_the_toast_the_semantic_tree_announces() {
    let _g = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-notice-{}", std::process::id())));
    // The document lives in the HERMETIC sandbox, not on the real disk: the
    // App reads and (on save) WRITES through `crate::fs::active()`, and the
    // whole point of a save here is that it really happens.
    let doc = std::path::PathBuf::from("/ws/proj/probe.md");

    // A real Cmd-S (or its Emacs twin) through the real keymap: `manual_save`
    // raises the `saved` toast as its only user-visible result.
    let save = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-s",
        crate::convention::Convention::Linux => "C-x C-s",
    };
    let quiet = dir.join("quiet.png");
    let noticed = dir.join("noticed.png");
    let (quiet_json, noticed_json) = in_sandbox_with(&doc, || {
        capture_live_app(
            quiet.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: Vec::new(),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("the live-App capture needs a GPU adapter");
        capture_live_app(
            noticed.clone(),
            LiveAppSpec {
                file: Some(doc.clone()),
                keys: crate::keyspec::parse_chords(save).expect("the save chord parses"),
                root: Some(proj()),
                workspace: None,
                config: cfg(),
                canvas: None,
                dpi: None,
            },
        )
        .expect("the live-App capture needs a GPU adapter");
        (sidecar(&quiet), sidecar(&noticed))
    });

    assert_eq!(
        noticed_json["notice"]["text"].as_str(),
        Some("saved"),
        "the sidecar reports the notice the App raised"
    );
    assert_eq!(
        noticed_json["notice"]["kind"].as_str(),
        Some("toast"),
        "and its KIND — a lifetime, which is what distinguishes it from a \
         held sticky notice"
    );
    assert!(
        quiet_json["notice"].is_null(),
        "a capture that raised no notice reports none"
    );

    // THE DEFECT, spelled as itself.
    let a = std::fs::read(&quiet).expect("read the quiet PNG");
    let b = std::fs::read(&noticed).expect("read the noticed PNG");
    assert_ne!(
        a, b,
        "a capture carrying a toast must NOT be byte-identical to one \
         without — that identity is item 296, and it held for every notice \
         this channel ever raised"
    );

    // THE TWO ANSWERS AGREE.
    let announced = noticed_json["semantic"]["nodes"]
        .as_array()
        .expect("a live-App sidecar carries a semantic tree")
        .iter()
        .find(|n| n["role"].as_str() == Some("status"))
        .and_then(|n| n["name"].as_str().map(str::to_string));
    assert_eq!(
        announced.as_deref(),
        noticed_json["notice"]["text"].as_str(),
        "the `notice` block and the `semantic` tree's status node must carry \
         the SAME sentence — they were free to disagree, and did"
    );
}

/// The live-`App` capture must write through the ORDINARY schema — one
/// const, one writer, one shape. A parallel serializer would show up here as
/// a schema string that is not the plain one, or as a sidecar missing a block
/// every other capture carries.
#[test]
fn the_live_app_sidecar_is_the_ordinary_schema_and_shape() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-schema-{}", std::process::id())),
    );
    let live = dir.join("plain.png");
    let live_json = in_sandbox(|| {
        capture_live_app(live.clone(), spec(Vec::new(), Some(proj())))
            .expect("the live-App capture needs a GPU adapter");
        sidecar(&live)
    });
    assert_eq!(
        live_json["schema"].as_str(),
        Some(crate::capture::schema_plain().as_str()),
        "the plain single-frame schema, from the one const"
    );
    // A SPOT-CHECK ACROSS THE WHOLE SHAPE, not just the blocks this mode
    // touches: the fold hands `capture_with` an ordinary `CaptureOpts`, so
    // every block a `--screenshot` carries must be present here too.
    for block in [
        "canvas",
        "font",
        "theme",
        "caret_mode",
        "page",
        "wysiwyg",
        "layout",
        "hud",
        "search",
        "project",
        "overlay",
        "buffers",
        "replay_skips",
        "cursor",
        "text",
        "md_spans",
    ] {
        assert!(
            live_json.get(block).is_some(),
            "the live-App sidecar is missing `{block}` — it is not the \
             ordinary schema, which means something grew a second serializer"
        );
    }
    assert_eq!(
        live_json["overlay"]["active"].as_bool(),
        Some(false),
        "no chords were pressed, so the shared no-overlay literal is emitted \
         — the same one every card-less capture carries"
    );
}

/// A `--capture-size`/`--capture-dpi` combination reaching this door must
/// change the RENDERED GEOMETRY, not merely echo the requested numbers back
/// into the `canvas` block — a law asserting only the latter is satisfiable
/// by a door that reports the field and then ignores it. So this drives one
/// long single-logical-line document through three renders and reads the
/// visual-row WRAP COUNT for that line straight out of the `layout` oracle:
///
/// 1. a narrower PHYSICAL canvas at dpi 1 must wrap the same line into MORE
///    visual rows than a wider one — real reflow, not an echoed number;
/// 2. a device canvas at dpi 2 that is exactly DOUBLE a dpi-1 canvas must
///    wrap identically to it — the documented `--capture-dpi` meaning
///    (`WxH` device @ dpi N == `(W/N)x(H/N)` logical) holding on this door
///    exactly as it does on the ordinary `--screenshot` one.
#[test]
fn a_live_app_capture_honors_capture_size_and_the_dpi_meaning_holds() {
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-live-app-canvas-{}", std::process::id())),
    );
    let doc = std::path::PathBuf::from("/ws/proj/long.md");
    // One long logical line with plentiful word-wrap points, long enough to
    // wrap several times even at the wide canvas.
    let long_line = "supercalifragilistic ".repeat(60);
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws")
            .with_dir("/ws/proj")
            .with_file(&doc, &long_line),
    );

    let wide_out = dir.join("wide.png");
    let narrow_out = dir.join("narrow.png");
    let dpi2_out = dir.join("dpi2.png");
    let spec_at = |canvas: Option<(u32, u32)>, dpi: Option<f32>| LiveAppSpec {
        file: Some(doc.clone()),
        keys: Vec::new(),
        root: Some(proj()),
        workspace: None,
        config: cfg(),
        canvas,
        dpi,
    };
    let (wide_json, narrow_json, dpi2_json) = crate::fs::with_fs(mem, || {
        capture_live_app(wide_out.clone(), spec_at(Some((1200, 800)), None))
            .expect("the live-App capture needs a GPU adapter");
        capture_live_app(narrow_out.clone(), spec_at(Some((640, 800)), None))
            .expect("the live-App capture needs a GPU adapter");
        // Double the wide canvas AND the dpi: same LOGICAL window (1200x800).
        capture_live_app(dpi2_out.clone(), spec_at(Some((2400, 1600)), Some(2.0)))
            .expect("the live-App capture needs a GPU adapter");
        (sidecar(&wide_out), sidecar(&narrow_out), sidecar(&dpi2_out))
    });

    // The field is genuinely threaded (not left at the 1200x800 default) —
    // necessary, but per the doc comment above, not sufficient on its own.
    assert_eq!(wide_json["canvas"]["width"].as_u64(), Some(1200));
    assert_eq!(narrow_json["canvas"]["width"].as_u64(), Some(640));
    assert_eq!(dpi2_json["canvas"]["width"].as_u64(), Some(2400));
    assert_eq!(dpi2_json["canvas"]["dpi"].as_f64(), Some(2.0));

    let wraps_for_line0 = |v: &serde_json::Value| -> usize {
        v["layout"]["rows"]
            .as_array()
            .expect("a live-App sidecar carries a layout block")
            .iter()
            .filter(|row| row["line"].as_u64() == Some(0))
            .count()
    };
    let wide_wraps = wraps_for_line0(&wide_json);
    let narrow_wraps = wraps_for_line0(&narrow_json);
    let dpi2_wraps = wraps_for_line0(&dpi2_json);

    assert!(
        wide_wraps >= 2,
        "the fixture line must wrap at least twice at 1200px wide for this \
         law to be meaningful (got {wide_wraps})"
    );
    assert!(
        narrow_wraps > wide_wraps,
        "a narrower --capture-size must wrap the SAME line into MORE visual \
         rows ({narrow_wraps} at 640px vs {wide_wraps} at 1200px) — this is \
         the geometry the flag is supposed to change, not just the number \
         echoed into the canvas block"
    );
    assert_eq!(
        dpi2_wraps, wide_wraps,
        "2400x1600 @ dpi 2.0 is the SAME logical 1200x800 window as \
         1200x800 @ dpi 1.0, so the two must wrap identically — the \
         documented --capture-dpi meaning (WxH device @ dpi N == \
         (W/N)x(H/N) logical) holding on the live-App door exactly as it \
         does on the ordinary --screenshot one"
    );
}

/// **ITEM 450, THE DECIDED HALF OF ITEM 444's OPEN NOTE: the bottom
/// identity's folder line always names the ACTIVE FILE's own folder, never
/// the nominally "active project."** `switch_project` (`s-S-p`/`C-S-p`,
/// `Action::OpenProject`) moves `project_location.root` with no document
/// opened or activated, so the gutter's project line must keep naming the
/// root the OPEN file remembers — while every destination default (New
/// document, Go to, Move, export — represented here by the sidecar's own
/// `project.root`, the one field they all read) keeps following the switch.
/// Reverting the fix in `App::sync_view` makes this go red on `gutter`
/// alone, never on `project`: the two claims are independent by
/// construction, and BOTH are asserted in one frame so neither drifts
/// unnoticed.
///
/// Swept across the WHOLE world roster at two DPIs (not one hand-picked
/// world/scale) because the gutter's project line is a rendered, elided
/// margin string (`rowlayout::fit_primary`, gated on `avail_chars` — a
/// function of the label's own font metrics) — CLAUDE.md's tripwire that a
/// check validated on one scale alone has shipped real DPI-dependent chrome
/// bugs applies here as much as anywhere else. The companion PRESENCE floor
/// (`same_root`) rules out a formatter that always shows nothing: it must
/// name `notes` too, not merely differ from `archive`.
#[test]
fn switch_project_alone_names_the_open_files_folder_while_the_dispatch_root_follows() {
    let _g = crate::testlock::serial();
    let mem = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir("/cfg")
            .with_dir("/ws/notes")
            .with_dir("/ws/archive")
            .with_file("/ws/notes/index.md", "index\n"),
    );
    let open_project = match crate::convention::Convention::current() {
        crate::convention::Convention::Mac => "s-S-p",
        crate::convention::Convention::Linux => "C-S-p",
    };
    // The picker's folders facet opens on the workspace row itself; one
    // `Down` reaches its first alphabetical child (`archive`, ahead of
    // `notes`), `Enter` accepts it — Switch project to `archive`, nothing
    // else.
    let switch_keys = crate::keyspec::parse_chords(&format!("{open_project} Down Enter"))
        .expect("the switch-project walk parses");

    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl-switch-project-identity-{}",
        std::process::id()
    )));

    // `Config` carries no `Clone`, so a fresh one is built per capture
    // rather than shared across the two calls a cell needs. The canvas
    // doubles WITH the dpi (2400x1600 @ 2.0, matching
    // `a_live_app_capture_honors_capture_size_and_the_dpi_meaning_holds`'s
    // own pairing) so the LOGICAL page stays 1200x800 at both scales — a real
    // Retina display, not an artificially narrowed logical page that would
    // suppress the gutter's project line for a reason unrelated to this fix.
    let spec_for = |world: &str, dpi: f32, keys: Vec<crate::keyspec::Chord>| LiveAppSpec {
        file: Some(PathBuf::from("/ws/notes/index.md")),
        keys,
        root: Some(PathBuf::from("/ws/notes")),
        workspace: Some(PathBuf::from("/ws")),
        config: Config {
            theme: Some(world.to_string()),
            ..cfg()
        },
        canvas: if dpi > 1.0 {
            Some((2400, 1600))
        } else {
            None
        },
        dpi: Some(dpi),
    };

    let mut checked = 0usize;
    crate::fs::with_fs(mem, || {
        for world in crate::theme::world_names() {
            for dpi in [1.0f32, 2.0f32] {
                // PRESENCE FLOOR: the ordinary same-root case, no switch at
                // all — the label must still name `notes`, not merely
                // differ from `archive`.
                let same_out = dir.join(format!("same-{world}-{dpi}.png"));
                let same = capture_live_app(same_out.clone(), spec_for(world, dpi, Vec::new()))
                    .map(|()| sidecar(&same_out))
                    .expect("the live-App capture needs a GPU adapter");
                assert_eq!(
                    same["gutter"]["project"].as_str(),
                    Some("notes"),
                    "world={world} dpi={dpi}: the ordinary same-root case \
                     must NAME its folder"
                );

                // THE DECISION: Switch project to `archive`, nothing else.
                let out = dir.join(format!("switch-{world}-{dpi}.png"));
                let v = capture_live_app(out.clone(), spec_for(world, dpi, switch_keys.clone()))
                    .map(|()| sidecar(&out))
                    .expect("the live-App capture needs a GPU adapter");
                assert_eq!(
                    v["gutter"]["project"].as_str(),
                    Some("notes"),
                    "world={world} dpi={dpi}: the identity must keep naming \
                     the OPEN file's own folder, never the switched-to \
                     project — non-vacuous by construction (this read \
                     `archive` before the fix)"
                );
                // The name line is elided independently at some world/DPI
                // combinations (`rowlayout::fit_primary`, unrelated to this
                // item's fix) — checked for PRESENCE and its preserved
                // extension, not byte-exact text.
                let drawn_name = v["gutter"]["name"].as_str().unwrap_or_default();
                assert!(
                    drawn_name.ends_with(".md") && !drawn_name.is_empty(),
                    "world={world} dpi={dpi}: the open file itself never \
                     changed, got {drawn_name:?}"
                );
                assert_eq!(
                    v["project"]["root"].as_str(),
                    Some("/ws/archive"),
                    "world={world} dpi={dpi}: the DISPATCH root (New \
                     document / Go to / Move / export's destination) must \
                     still follow Switch project — item 450 does not \
                     re-sync the two halves, it only stops the label lying"
                );
                checked += 1;
            }
        }
    });
    assert_eq!(
        checked,
        crate::theme::world_names().len() * 2,
        "the roster x DPI sweep lost cells"
    );
}
