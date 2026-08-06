//! LIFETIME STATS' App-side WIRING (native only — `cfg(not(target_arch =
//! "wasm32"))`, mirroring the daemon / session restore's own gate): the
//! TRACKING HOOKS the live `App` calls from its existing seams, plus the FLUSH
//! on the autosave triggers.
//!
//! The STATE is not here. `crate::stats` owns the pure store + injected-clock
//! helpers + the (de)serializer, and `app::usage::UsageLedger` owns
//! the live ledger — the odometer, its unflushed-changes stamp, and its
//! caret-travel anchor. What remains in this file is only the seam: the hooks
//! that have to reach the OTHER domains an odometer sample needs (the frame
//! clock and GPU pipeline, the document's cursor, the summoned overlay) before
//! handing a value to the owner.
//!
//! **The hooks (each takes the privacy gate as a [`super::usage::Recording`]
//! value from `ConfigurationRuntime` — the one reader of the toggle):**
//!  - [`Self::stats_note_keystroke`] — on the keyboard-input path
//!    (`on_keyboard_input`, past every filter).
//!  - [`Self::stats_track_caret`] — at the end of `sync_view` (the one live
//!    bridge every caret move passes through).
//!  - [`Self::stats_touch_file`] — from `load_path`, beside `push_recent_file`.
//!  - [`Self::stats_flush`] — the atomic write, on the SAME idle/blur/switch/
//!    quit triggers the autosave engine's own flush uses.
//!
//! **Determinism:** all four live ONLY on the live `App`; the headless capture
//! never constructs a `UsageLedger`, so a `--screenshot`/`--keys` capture is
//! STRUCTURALLY incapable of touching `stats.toml` — see
//! `main::run::tests::headless_replay_never_touches_the_stats_file`.

use super::*;

impl App {
    /// Record ONE keyboard press into the odometer. `printable` is whether the
    /// press resolved to an `Action::InsertChar` (a real character written).
    /// The session clock is read through the ONE time owner (the same clock
    /// the ledger's origin was stamped from), so a deterministic clock would
    /// govern the active-writing odometer too.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_note_keystroke(&mut self, printable: bool) {
        let recording = self.config.usage_recording();
        let now = self.frame.now();
        self.usage.note_keystroke(recording, now, printable);
    }

    /// Sample the caret and accumulate its DOCUMENT-space travel. Called at the
    /// end of `sync_view`, once the pipeline's caret target reflects this sync's
    /// cursor. The sample is handed over as a THUNK: it reads the GPU pipeline
    /// and queries the rope, and neither is paid for with tracking off. It
    /// yields `None` when the GPU is not up yet (nothing to read a caret
    /// position from).
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_track_caret(&mut self) {
        let recording = self.config.usage_recording();
        let frame = &self.frame;
        let document = &self.document;
        self.usage.track_caret(recording, || {
            let gpu = frame.gpu()?;
            Some((
                gpu.pipeline.caret_doc_xy(),
                document.buffer().cursor_line_col(),
            ))
        });
    }

    /// Record a file OPEN into the distinct-files set (deduped). Called from
    /// `load_path`, the same door the recent-files MRU rides.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_touch_file(&mut self, path: PathBuf) {
        let recording = self.config.usage_recording();
        self.usage.touch_file(recording, path);
    }

    /// Drop the caret-travel anchor across a BUFFER SWAP (file open / new
    /// note), so the first caret sample in the new document re-anchors instead
    /// of counting the jump between two documents' incomparable coordinate
    /// spaces as travel.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_reset_caret_anchor(&mut self) {
        self.usage.reset_caret_anchor();
    }

    /// Push the LIFETIME-ODOMETER snapshot into the pipeline for the held HUD's
    /// odometer rows. Called every `sync_view` — the field is cheap to hold and
    /// only read when the HUD is summoned. With the odometer OFF the ledger
    /// yields `None`, so the rows honestly read as the `"—"` placeholder rather
    /// than a misleading row of zeros. This is the LIVE-ONLY seam that keeps a
    /// `--hud` capture (which never calls `sync_view`) showing placeholders.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_sync_hud(&mut self) {
        // Snapshot BEFORE borrowing the GPU (both read `self`) — the shape
        // `streaks_sync_card` already uses.
        let snapshot = self.usage.hud_snapshot(self.config.usage_recording());
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_hud_stats(snapshot);
        }
    }

    /// Push the DISCOVERABILITY surfaces' content into the pipeline every `sync_view`
    /// (LIVE-ONLY, native-only): the HOLD-⌘ peek's personalized rows and the Keybindings
    /// footer's top-3 tips, both derived from the SILENT USAGE LEDGER's graduation
    /// ranking (the SAME query, top-6 for the peek / top-3 for the footer). A headless
    /// capture never calls this, so the pipeline's peek rows stay EMPTY (→ the curated
    /// STARTER SIX renders, deterministic) and the footer tips stay empty (→ the footer
    /// is hidden, a Keybindings capture byte-identical). Cheap: the ledger is catalog-
    /// sized, so ranking it per sync is negligible (like `stats_sync_hud`).
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn sync_discoverability(&mut self) {
        let peek_rows = self.peek_rows_from_ledger();
        // The footer tips ride ONLY while the Keybindings overlay is open (so no OTHER
        // flat picker ever grows a footer); empty otherwise → the footer hides.
        let tips = if self.overlay_is_keybindings() {
            self.keybinding_tips_from_ledger()
        } else {
            Vec::new()
        };
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_peek_rows(peek_rows);
            gpu.pipeline.set_keybindings_tips(tips);
        }
    }

    /// The HOLD-⌘ peek's personalized rows from the ledger. Empty on a fresh
    /// install → the pipeline falls back to the starter six.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn peek_rows_from_ledger(&self) -> Vec<crate::peek::PeekRow> {
        self.usage.peek_rows()
    }

    /// The Keybindings footer's "your top 3" tip lines from the ledger.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn keybinding_tips_from_ledger(&self) -> Vec<String> {
        self.usage.keybinding_tips()
    }

    /// Whether the currently-summoned overlay (if any) is the Keybindings rebind menu —
    /// the gate for pushing the footer tips (the footer belongs to that one picker).
    #[cfg(not(target_arch = "wasm32"))]
    fn overlay_is_keybindings(&self) -> bool {
        self.workspace_state
            .overlay()
            .map(|o| o.kind == crate::overlay::OverlayKind::Keybindings)
            .unwrap_or(false)
    }

    /// Record ONE command dispatch into the SILENT USAGE LEDGER, attributed to the
    /// `door` it came through (chord / palette / menu). Called at the TOP of
    /// [`Self::apply`] — the ONE seam every door funnels through (a keyboard chord, the
    /// palette's `Effect::RunAction` re-dispatch, and the macOS menu handler all reach
    /// `apply`), so all three attribute here without a parallel path.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn ledger_note_dispatch(
        &mut self,
        action: &crate::keymap::Action,
        door: crate::stats::Door,
    ) {
        let recording = self.config.usage_recording();
        self.usage.note_dispatch(recording, action, door);
    }

    /// Flush the odometer to disk ATOMICALLY, on the SAME idle/blur/switch/quit
    /// triggers the autosave engine's own flush uses. A no-op when the feature
    /// is off OR nothing has changed since the last flush, so a quiet blur/quit
    /// writes nothing.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn stats_flush(&mut self) {
        let recording = self.config.usage_recording();
        self.usage.flush_odometer(recording);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn keystrokes_and_chars_accrue_then_flush_round_trips() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // Three presses: two printable inserts, one motion.
            app.stats_note_keystroke(true);
            app.stats_note_keystroke(true);
            app.stats_note_keystroke(false);
            assert_eq!(app.usage.odometer().keystrokes, 3);
            assert_eq!(
                app.usage.odometer().chars_typed,
                2,
                "only the printable presses count as chars"
            );
            assert!(
                app.usage.odometer_dirty(),
                "increments mark the store dirty"
            );

            app.stats_flush();
            assert!(!app.usage.odometer_dirty(), "flush clears the dirty flag");
            let saved = crate::stats::load(&crate::stats::stats_path());
            assert_eq!(saved.keystrokes, 3);
            assert_eq!(saved.chars_typed, 2);
        });
    }

    #[test]
    fn touch_file_records_distinct_opens_and_dedupes() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.stats_touch_file(PathBuf::from("/n/a.md"));
            app.stats_touch_file(PathBuf::from("/n/b.md"));
            app.stats_touch_file(PathBuf::from("/n/a.md")); // a re-open
            assert_eq!(
                app.usage.odometer().files_touched_count(),
                2,
                "distinct count, not open count"
            );
        });
    }

    /// A re-open of an already-seen path reports [`Changed::No`] from the
    /// store's own dedupe, so it must not re-dirty the record — otherwise a
    /// quiet session that merely re-opened a file would write `stats.toml` on
    /// every later idle tick. This is the arm the `Dirtying` door exists to
    /// keep honest: a blanket "any record call dirties" would pass every other
    /// test in this file and fail only here.
    #[test]
    fn a_deduped_reopen_does_not_dirty_the_record() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.stats_touch_file(PathBuf::from("/n/a.md"));
            app.stats_flush();
            assert!(!app.usage.odometer_dirty(), "flushed clean");
            app.stats_touch_file(PathBuf::from("/n/a.md"));
            assert!(
                !app.usage.odometer_dirty(),
                "a deduped re-open changes nothing and must not re-dirty the record"
            );
        });
    }

    #[test]
    fn flush_is_a_no_op_when_nothing_changed() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // No increments yet: a flush must not even create the file.
            app.stats_flush();
            assert!(
                crate::fs::active()
                    .read(&crate::stats::stats_path())
                    .is_err(),
                "a clean flush writes nothing"
            );
        });
    }

    #[test]
    fn kill_switch_off_tracks_nothing_and_never_writes() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let cfg = Config {
                stats: Some(false),
                ..Config::empty()
            };
            let mut app = App::new(None, PathBuf::from("/n"), None, None, cfg);
            app.stats_note_keystroke(true);
            app.stats_touch_file(PathBuf::from("/n/a.md"));
            assert_eq!(app.usage.odometer().keystrokes, 0, "off: no tracking");
            assert!(!app.usage.odometer_dirty());
            app.stats_flush();
            assert!(
                crate::fs::active()
                    .read(&crate::stats::stats_path())
                    .is_err(),
                "off: never writes stats.toml"
            );
        });
    }

    #[test]
    fn ledger_attributes_doors_by_the_dispatched_action_and_round_trips() {
        use crate::keymap::Action;
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // A catalog command dispatched through each of the three doors.
            app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Chord);
            app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Chord);
            app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Palette);
            app.ledger_note_dispatch(&Action::OpenThemeMenu, crate::stats::Door::Menu);
            // A NON-catalog action (arrow motion / self-insert) keys no row — the hot
            // path — and a CATALOG-listed navigation motion (rebindable since
            // 2026-07-10) is gated out by `is_motion` just the same: cursor travel
            // never keys a ledger row nor dirties the store.
            app.ledger_note_dispatch(&Action::ForwardChar, crate::stats::Door::Chord);
            app.ledger_note_dispatch(&Action::InsertChar('z'), crate::stats::Door::Chord);
            app.ledger_note_dispatch(&Action::ForwardWord, crate::stats::Door::Chord);
            app.ledger_note_dispatch(&Action::LineStart, crate::stats::Door::Chord);

            let goto = app.usage.odometer().command_counts("go_to_file");
            assert_eq!((goto.chord, goto.palette, goto.menu), (2, 1, 0));
            let theme = app.usage.odometer().command_counts("switch_theme");
            assert_eq!((theme.chord, theme.palette, theme.menu), (0, 0, 1));
            assert_eq!(
                app.usage.odometer().command_usage.len(),
                2,
                "only catalog commands keyed rows"
            );
            assert!(
                app.usage.odometer_dirty(),
                "a recorded dispatch marks the store dirty"
            );

            // Persists into (and reloads from) the SAME stats.toml as the odometer.
            let expected = app.usage.odometer().command_usage.clone();
            app.stats_flush();
            let saved = crate::stats::load(&crate::stats::stats_path());
            assert_eq!(saved.command_usage, expected);
        });
    }

    #[test]
    fn ledger_graduation_candidates_wire_through_the_real_catalog() {
        use crate::keymap::Action;
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // Reached repeatedly via the palette; Go to file… HAS a native chord (Cmd-O).
            for _ in 0..4 {
                app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Palette);
            }
            // Keep version is palette-only (no native chord) — must be excluded even
            // though it is the most-used slow-door command here.
            for _ in 0..9 {
                app.ledger_note_dispatch(&Action::KeepVersion, crate::stats::Door::Palette);
            }
            // The candidate query wired through the catalog's own `has_native_chord`.
            let cands = app
                .usage
                .odometer()
                .graduation_candidates(crate::commands::has_native_chord, 5);
            let slugs: Vec<&str> = cands.iter().map(|(s, _)| s.as_str()).collect();
            assert_eq!(slugs, vec!["go_to_file"], "chordless Keep version excluded");
            assert!(
                !app.usage.odometer().is_graduated("go_to_file"),
                "not yet graduated on slow-door use"
            );

            // Now learn the Cmd-O chord GRADUATION_N times: it drops off the candidates.
            for _ in 0..crate::stats::GRADUATION_N {
                app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Chord);
            }
            assert!(
                app.usage.odometer().is_graduated("go_to_file"),
                "chord in the fingers now"
            );
            assert!(
                app.usage
                    .odometer()
                    .graduation_candidates(crate::commands::has_native_chord, 5)
                    .is_empty(),
                "a graduated command is no longer a candidate"
            );
        });
    }

    #[test]
    fn discoverability_surfaces_rank_slow_door_use_from_a_fake_ledger() {
        use crate::keymap::Action;
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // A fake ledger: three native-chord commands reached via slow doors, ranked
            // by slow-door count (Go to file 4 > Switch theme 2 > Version history 1).
            for _ in 0..4 {
                app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Palette);
            }
            app.ledger_note_dispatch(&Action::OpenThemeMenu, crate::stats::Door::Palette);
            app.ledger_note_dispatch(&Action::OpenThemeMenu, crate::stats::Door::Menu);
            app.ledger_note_dispatch(&Action::OpenHistory, crate::stats::Door::Palette);
            // Keep version is palette-only (no native chord) → never a peek/footer row
            // even though it's reached via a slow door.
            for _ in 0..9 {
                app.ledger_note_dispatch(&Action::KeepVersion, crate::stats::Door::Palette);
            }

            // CONVENTION-PARAMETRIC expected chord labels: `peek_row_for_slug`/
            // `UsageLedger::peek_rows` resolve each chord through
            // `commands::resolved_native_label(c, Convention::current())` — Mac ⌘
            // glyphs on `Convention::Mac`, Linux word labels (`"Ctrl+O"`) on
            // `Convention::Linux` (see `convention.rs`'s doc + the AWL_CONVENTION_FORCE
            // dev knob CI's linux runner exercises via the real `cfg(target_os)` path).
            // Computing the expectation through the SAME resolver — rather than a
            // hardcoded mac-only literal — keeps this law true on EITHER convention,
            // never just whichever one happens to be ambient.
            let label_for = |action: &Action| -> String {
                let c = crate::commands::COMMANDS
                    .iter()
                    .find(|c| c.action == *action)
                    .unwrap();
                crate::commands::resolved_native_label(c, crate::convention::Convention::current())
            };
            let goto_chord = label_for(&Action::OpenGoto);
            let theme_chord = label_for(&Action::OpenThemeMenu);
            let history_chord = label_for(&Action::OpenHistory);

            // The PEEK rows: chord+name, ranked, chordless Keep version excluded.
            let peek = app.peek_rows_from_ledger();
            let names: Vec<&str> = peek.iter().map(|r| r.name.as_str()).collect();
            assert_eq!(names, vec!["Go to file", "Switch theme", "Version history"]);
            assert_eq!(peek[0].chord, goto_chord);
            assert_eq!(peek[1].chord, theme_chord);

            // The FOOTER tips: the SAME ranking, top 3, as "<chord>  <name>" one-liners.
            let tips = app.keybinding_tips_from_ledger();
            assert_eq!(
                tips,
                vec![
                    format!("{goto_chord}  Go to file"),
                    format!("{theme_chord}  Switch theme"),
                    format!("{history_chord}  Version history"),
                ]
            );
        });
    }

    #[test]
    fn discoverability_surfaces_are_empty_on_a_fresh_ledger() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // Nothing tracked yet → no personalized rows / tips. The pipeline then falls
            // back to the curated starter six for the peek, and the footer hides.
            assert!(
                app.peek_rows_from_ledger().is_empty(),
                "fresh ledger: no peek rows"
            );
            assert!(
                app.keybinding_tips_from_ledger().is_empty(),
                "fresh ledger: no footer tips"
            );
        });
    }

    #[test]
    fn ledger_off_records_no_command_usage() {
        use crate::keymap::Action;
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let cfg = Config {
                stats: Some(false),
                ..Config::empty()
            };
            let mut app = App::new(None, PathBuf::from("/n"), None, None, cfg);
            app.ledger_note_dispatch(&Action::OpenGoto, crate::stats::Door::Chord);
            assert!(
                app.usage.odometer().command_usage.is_empty(),
                "off: the ledger stays empty"
            );
            assert!(!app.usage.odometer_dirty());
        });
    }
}
