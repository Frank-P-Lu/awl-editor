//! ITEM 188 — THE LIVE-`App` CAPTURE MODE (`--screenshot-app OUT.png [file]`):
//! a real headless [`App`], driven by real chords, photographed by the ORDINARY
//! sidecar writer.
//!
//! # What this mode is for
//!
//! `docs/harness-reach.md` splits the harness into three tiers. Tier 1 (a
//! `--keys` capture) replays the SHARED CORE, and every effect only the live
//! `App` can perform is classified Unsupported and skipped — so a Settings
//! write, a buffer finish, a keymap-flavor flip is executed nowhere and reported
//! as a `replay_skips` entry rather than as state. Tier 2 (a real `App` driven
//! by `App::press_chords_headless`) performs them for real, but until this mode
//! existed it had no sidecar at all: the outcome had to be asserted in Rust, off
//! the one oracle the rest of the project reads.
//!
//! This mode is tier 2 WITH tier 1's oracle. The App is real, the chords are
//! real, the effects actually happen; the artifact is an ordinary
//! `awl-capture/N` PNG + JSON, distinguished only by its `driver: "live-app"`
//! field.
//!
//! # Three things it deliberately does not do
//!
//! **It grows no second serializer.** The frame is rendered by
//! [`capture::capture_with`] — the same single-frame path `--screenshot` takes
//! — which writes through the one `capture::sidecar::write_sidecar`. The
//! `CaptureOpts` are built by `App::capture_opts`, which routes the project
//! block through `run::project_info` and everything else through
//! `run::fold_capture_state`, the fold the storyboard stepper shares.
//!
//! **It emits no second schema.** One const, one header, one `/N` row.
//!
//! **It touches no real file.** The App PERFORMS its writes here, so the mode is
//! a SCENARIO door: `args::parse_args` swaps the process fs to the seeded
//! hermetic sandbox before the config loads (the storyboard's treatment), and
//! `App::new_headless_capture` deliberately constructs on THAT fs. The PNG +
//! JSON go out through `std::fs`, the documented sandbox bypass every capture
//! deliverable already uses.
//!
//! # What it still does not reach
//!
//! Tier 3. There is no window, no surface and no event loop, so `gpu` is `None`
//! and the App renders nothing itself — the harness renders the App's buffer
//! through its own offscreen pipeline, exactly as `--screenshot-frames` does.
//! The `&ActiveEventLoop` census in `app::tests::source_audit` is the exact list
//! of what that costs.

use std::path::PathBuf;

use anyhow::Result;

use crate::app::App;
use crate::capture;
use crate::config::Config;

/// Drive `keys` into a real headless [`App`] rooted at `root`, then capture the
/// resulting editor state to `out` + its sidecar. Returns after both artifacts
/// are written.
pub(crate) fn capture_live_app(
    out: PathBuf,
    file: Option<PathBuf>,
    keys: Vec<crate::keyspec::Chord>,
    root: Option<PathBuf>,
    workspace: Option<PathBuf>,
    config: Config,
) -> Result<()> {
    // Same root precedence as every other capture door — the EXPLICIT `--root`
    // or the launch file's own directory, never a remembered session (the
    // capture-gate law).
    let active_root = super::resolve_root(&root, &file);
    let mut app = App::new_headless_capture(file, active_root, workspace, config);
    // The chords go through `dispatch_pressed_key` -> keymap resolve -> `apply`
    // -> the live effect interpreter: the same code a physical keypress takes.
    // An exit request is state, not an error — a spec ending in Quit still
    // photographs the editor it left behind.
    let _exit_requested = app.press_chords_headless(&keys);
    let opts = app.capture_opts();
    capture::capture_with(&out, crate::run::CaptureSubject::buffer(&app), &opts)?;
    println!("wrote {} (+ sidecar .json)", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureOpts;
    use crate::settings::SettingId;
    use crate::testscratch::ScratchDir;
    use std::sync::Arc;

    const CFG: &str = "/cfg/config.toml";

    /// The chord that summons the Settings workspace in the convention this
    /// pass is running under — resolved from the RUNNING convention rather than
    /// hardcoded, because `native-gate.sh` runs the suite once per convention
    /// and each pass must drive its own real binding (item 114's rule).
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
    fn cfg() -> Config {
        Config {
            path: std::path::PathBuf::from(CFG),
            ..Config::empty()
        }
    }

    /// Run `body` over a seeded `InMemoryFs`, so nothing in these laws — the App's
    /// config persist included — can reach the real disk. The PNG + JSON still go
    /// out through `std::fs`, the documented capture-deliverable bypass.
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
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl-item208-live-app-{}", std::process::id())),
        );
        let png = dir.join("folder-choice.png");
        let json = in_sandbox(|| {
            capture_live_app(
                png.clone(),
                None,
                crate::keyspec::parse_chords(new_document_chord()).unwrap(),
                Some(crate::fs::data_root()),
                None,
                cfg(),
            )
            .unwrap();
            sidecar(&png)
        });
        assert_eq!(json["driver"].as_str(), Some("live-app"));
        assert_eq!(json["overlay"]["mode"].as_str(), Some("switch"));
    }

    /// QUEUE ITEM 188, THE PRIMARY LAW — the transition converted from
    /// Rust-only to sidecar-provable.
    ///
    /// Flipping the KEYMAP row of the Settings workspace is the hardest case in
    /// the live-only census, not a convenient one: `docs/harness-reach.md` lists
    /// `setting_toggle` as **Unsupported** for an ordinary `--keys` capture, and
    /// names `SettingToggle{key: "keymap"}` as the ONE key that stays Unsupported
    /// even under item 190's `FilesystemCapability::Isolated` grant, because it
    /// needs a LIVE keymap rebuild no filesystem capability can supply. Before
    /// this item it was provable only in Rust (`app::tests::workspace_item114`'s
    /// tier-2 sweep, asserting `App` + config state directly).
    ///
    /// BOTH HALVES ARE ASSERTED IN ONE TEST, and the second is what makes the
    /// first mean anything: the SAME chord spec is driven through the ordinary
    /// `--keys` capture door, which must still report the OLD flavor and record
    /// the skip. A live-App sidecar that agreed with the replay sidecar would
    /// prove nothing had been reached at all.
    #[test]
    fn a_live_app_capture_photographs_a_keymap_flip_an_ordinary_capture_cannot_see() {
        let _g = crate::testlock::serial();
        let dir = ScratchDir::new(
            std::env::temp_dir().join(format!("awl-item188-live-app-{}", std::process::id())),
        );
        let keys = walk_to(SettingId::Keymap);

        // ── THE LIVE-`App` CAPTURE ────────────────────────────────────────
        let live = dir.join("live.png");
        let live_json = in_sandbox(|| {
            capture_live_app(live.clone(), None, keys.clone(), Some(proj()), None, cfg())
                .expect("the live-App capture needs a GPU adapter");
            sidecar(&live)
        });
        assert_eq!(
            live_json["overlay"]["active"].as_bool(),
            Some(true),
            "the sidecar was folded AFTER the chords were driven — a card the \
             walk summoned must be up in it"
        );
        assert_eq!(
            selected_name(&live_json),
            "Keymap",
            "the walk stood on the Keymap row"
        );
        assert_eq!(
            live_json["driver"].as_str(),
            Some("live-app"),
            "the sidecar names the tier that produced it"
        );
        assert_eq!(
            live_json["project"]["keymap_flavor"].as_str(),
            Some("emacs"),
            "THE CONVERTED CLAIM: the live keymap flip is readable from the \
             sidecar's project block, not only from a Rust assertion"
        );
        assert_eq!(
            selected_value(&live_json),
            "emacs",
            "and the row's own value cell agrees — two independent witnesses in \
             one artifact"
        );
        assert!(
            live_json["replay_skips"]
                .as_array()
                .is_some_and(|a| a.is_empty()),
            "a live-App capture skips nothing: it performs the effect"
        );

        // ── THE SAME SPEC THROUGH THE ORDINARY `--keys` DOOR ──────────────
        // Anti-vacuity. This is the capture that could NOT witness the flip,
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
            "the ordinary replay walked to the same row — the specs are identical"
        );
        assert_eq!(
            replay_json["project"]["keymap_flavor"].as_str(),
            Some("native"),
            "the ordinary capture still cannot see the flip — if this ever reads \
             `emacs`, the live-App law above has stopped proving anything new"
        );
        assert_eq!(
            replay_json["replay_skips"],
            serde_json::json!([{ "effect": "setting_toggle", "action": "Newline" }]),
            "and it says so out loud rather than reporting stale state silently"
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
            std::env::temp_dir().join(format!("awl-item188-schema-{}", std::process::id())),
        );
        let live = dir.join("plain.png");
        let live_json = in_sandbox(|| {
            capture_live_app(live.clone(), None, Vec::new(), Some(proj()), None, cfg())
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
}
