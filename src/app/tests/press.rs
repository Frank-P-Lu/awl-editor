//! The HEADLESS PRESS DOOR — the tier-2 driver `docs/harness-reach.md` maps.
//!
//! Test-only, and test-only is the point: it drives the REAL live `App`, not a
//! stand-in for it.

use super::*;

/// THE HEADLESS PRESS DOOR — the THIRD caller of
/// `App::dispatch_pressed_key`, beside the real
/// `WindowEvent::KeyboardInput` path and the `--live-script` probe.
///
/// It exists because the live `App` used to be undrivable off-window at all:
/// every door into it wanted an `&ActiveEventLoop`, which can only be borrowed
/// from inside a running winit loop. A whole class of transitions — everything
/// `App::apply` interprets, including `switch_project` → `set_root` and
/// `reload_config` — therefore had no headless driver, so no test and no
/// capture could reach it (queue items 180, 183). Narrowing that borrow to the
/// one capability it actually used (`app::schedule::Exit`) is what opened this door; the map
/// of what it did and did not reach is `docs/harness-reach.md`.
///
/// The three lines mirror `app/probe.rs`'s chord arm exactly, for the same
/// reason it gives: a parsed chord carries its own modifier state (a physical
/// press would have delivered it as `ModifiersChanged` first) and is already
/// un-composed, so `raw` and `bare` coincide. This is deliberately NOT a second
/// input pipeline — it feeds the one the window feeds, so a headless press and
/// a physical press are the same code by construction.
impl App {
    pub(crate) fn press_chord_headless(
        &mut self,
        chord: &crate::keyspec::Chord,
        exit: &dyn schedule::Exit,
    ) {
        self.mods = chord.mods;
        self.dispatch_pressed_key(exit, chord.key.clone(), chord.key.clone(), false);
        self.mods = winit::event::Modifiers::default();
    }

    /// Parse a `--keys` spec and drive every chord in it through the real press
    /// pipeline above. Returns whether any of them asked the loop to exit — the
    /// live `event_loop.exit()` a `schedule::RecordingExit` stands in for.
    /// Errors only on a structurally invalid token, exactly like `--keys`.
    pub(crate) fn press_spec_headless(&mut self, spec: &str) -> anyhow::Result<bool> {
        let chords = crate::keyspec::parse_chords(spec)?;
        let exit = crate::app::schedule::RecordingExit::new();
        for chord in &chords {
            self.press_chord_headless(chord, &exit);
        }
        Ok(exit.exit_requested())
    }
}
