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
use crate::args::LiveAppSpec;
use crate::capture;

/// Drive `keys` into a real headless [`App`] rooted at `root`, then capture the
/// resulting editor state to `out` + its sidecar. Returns after both artifacts
/// are written.
pub(crate) fn capture_live_app(out: PathBuf, spec: LiveAppSpec) -> Result<()> {
    let LiveAppSpec {
        file,
        keys,
        root,
        workspace,
        config,
    } = spec;
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

/// Print the same semantic snapshot the AccessKit adapter and live-App
/// sidecar consume, after driving optional real keymap actions headlessly.
pub(crate) fn print_semantic_json(spec: LiveAppSpec) -> Result<()> {
    let LiveAppSpec {
        file,
        keys,
        root,
        workspace,
        config,
    } = spec;
    let active_root = super::resolve_root(&root, &file);
    let mut app = App::new_headless_capture(file, active_root, workspace, config);
    let _exit_requested = app.press_chords_headless(&keys);
    println!(
        "{}",
        serde_json::to_string_pretty(&app.semantic_snapshot())?
    );
    Ok(())
}

#[cfg(test)]
#[path = "live_app/tests.rs"]
mod tests;
