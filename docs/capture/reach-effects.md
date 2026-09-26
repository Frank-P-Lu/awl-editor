# Replay effect reach

Read before selecting a replay capability or promising an effect outcome. Classifications depend on the driver and capability named below.

[Guide](../harness-reach.md) · Paths in code examples are relative to the repo root.

## Replay classifications

What a `--keys` capture does with each typed effect the shared core returns.
Generated from `replay::classify_for` / `replay::accept_class`, the production
owners; `replay::tests::the_harness_reach_map_matches_the_production_classifier`
fails if these rows drift from them, so this table cannot go stale.

- **Applied** — the replay performs it for real, or the effect's settled frame
  is byte-identical by contract. A capture witnesses it. Trust the sidecar.
- **Intercepted** — an external handoff (open a URL, trash a file, a browser
  download) is *observed and recorded* but not performed. The editor state is
  the same as live, so a capture is still trustworthy about everything else;
  the payload is available to a strict/storyboard trace.
- **Unsupported** — live-`App`-only work whose skip leaves the replay in a
  **different state than live**. `--strict-replay` aborts naming the effect;
  ordinary `--keys` warns on stderr and records it in `replay_skips`. **Do not
  ask for a capture oracle over one of these.** Drive it at tier 2 instead.

⚠️ **The three `notice_*` rows below read `Applied` and now mean it.** They were
`Applied` while the replay's interpreter *discarded* every notice — reported as
performed, drawn nowhere, and not recorded as a skip either, which is the worst of
the three classifications to be wrong about. An ordinary `--keys` capture now
latches the notice and photographs it, and the sidecar reports it as
`notice: { text, kind }`. A latched Toast never expires headlessly because there is
no clock, which matches a GPU-less live `App` (`App::set_toast_notice` arms no
deadline without a surface).

See the [checked effect table](../harness-reach.md#replay-effect-table).


`finish_buffer` remains Unsupported in the ordinary tier-1 replay above. The
tier-2 `--screenshot-app` door performs its save/notify/close effects for real;
when it closes the last file, the PNG and sidecar capture the resulting
zero-document start surface (`document.active: false`, `page: null`, and the
two semantic start buttons).

### ⚠️ A CAPTURE BUILDS ITS PIPELINES ONCE, SO BYTE-IDENTITY CANNOT SEE A LIVE THEME SWITCH

**Measured, not reasoned:** a token routed to the wrong pipeline *in the live
theme-switch path* moved **zero of 120 capture files** — twenty worlds × three
surfaces, PNG and sidecar. The mechanism is structural rather than a gap
someone forgot to close: `sync_theme_colors` is reached from
`app/apply.rs` (the live switch) and from pipeline **construction**, and a
capture only ever hits construction. So the colour half of a theme switch is
never exercised, and a defect there **repaints nothing any capture can see** —
it reaches only a user who changes worlds while the app is running.

**What this costs you:** byte-identity across the whole roster is the strongest
oracle this repo has for a refactor, and it is **blind to this one axis**. A
rename or re-route that touches pipeline colour seeding needs a law that reads
**the pipelines' own colours after a sync**, not a capture diff. Give such a
law a **non-vacuity guard** — that the two values being distinguished actually
differ somewhere in the roster — or it passes on a tree where they happen to
coincide.

**Do not generalise this into "captures prove nothing".** They proved the other
119 things in that same sweep. The rule is narrower and worth stating exactly:
*a capture witnesses the state a pipeline was BUILT with, never the state it
was later RE-SEEDED with.*

### Three asymmetries the table will not shout at you

**The same setting has two doors with two different reaches — narrower now,
closed by item 190.** Flipping typewriter scroll through its own command
emits `persist_typewriter` (**Applied** — the global flip happens in the
shared core, so an ORDINARY capture sees it, no capability needed).
Flipping the *same* setting from a row in the Settings picker emits
`setting_toggle`: still **Unsupported** for the table above (which classifies
under `FilesystemCapability::None`, the ordinary `--keys` door — the live
global flip and the config write are both `App`-side, so an ordinary replay
cannot honestly perform them), but **Applied** under `FilesystemCapability::
Isolated` (`main/run/settings_effects.rs`, the item-171 shape: only a strict/
scenario capture ever owns that capability, per `ReplayPolicy::isolated`).
`setting_value_commit` and `setting_path_pick` are promoted the identical
way. Keymap is no longer part of this trio at all — the "Keymap" row is a
`SettingKind::Picker` now (a catalog command + sub-overlay, the Caret-style
shape), so its accept emits `overlay_accept:Keymap`, not `setting_toggle`.
That accept stays Unsupported under EVERY capability, Isolated included (see
the table above): applying the picked flavor needs a LIVE keymap rebuild so
a later chord in the same replay resolves against the new flavor, a
capability no filesystem grant supplies — the same reason `rebind_commit`/
`rebind_reset` never promote either. Keymap is precisely what item 188's
`--screenshot-app` was proved on: the door with no possible capability grant
is the one where a live-`App` capture is the only sidecar there will ever be.
`setting_range_step` was already Applied before this item, because the value
change itself already happened in the core.

**`overlay_accept:Project` is Applied, no residue.** An accepted Project
navigator root re-derives the sidecar's whole project block through one builder
(`run::project_info`), so a capture reports the new root *and* the new
workspace. The Settings `Project root` path row reaches the same owner through
an isolated replay's Applied `setting_path_pick`; this is the strict-storyboard
door. The unified Files card deliberately does not emit either effect: its
`Change folder…` row requests the live-only native chooser and remains
Unsupported at tier 1.

`ReplaySession` once held its `root`, `workspace`, and file-index `corpus` fixed
for the session's whole lifetime, so a chord that ran **after** either applied
root effect still read the launch root's tree. `ReplaySession::
resync_project_location` (`main/run/location.rs`) is now the one owner invoked
by both applied routes — it rebuilds `corpus`
(`crate::index::build_index`) and re-resolves `workspace`
(`resolve_workspace`, against the SAME raw `--workspace` flag the constructor
used) before `root` itself moves, so a
chord applied after the accept sees the new tree exactly like live. Covered
end to end, both keymap conventions, by
`run::tests::keys_capture_switch_project_then_goto_lists_the_new_roots_files`;
the same-parent and no-parent (filesystem-root) edges item 180 named are swept
by `run::tests::resync_project_location_same_parent_switch_still_rebuilds_the_corpus`
and `run::tests::resync_project_location_no_parent_root_falls_back_to_itself_not_the_old_workspace`.

**`overlay_accept:ProjectBrowse` is Unsupported because nothing ever emits it.**
The switch-project picker's `Browse for folder…` door opens its own navigator
kind, but that navigator's accept is emitted **as `Project`** — one owner of
"make this the root", whichever door reached it — so the whole door journey
rides the Applied row above and reports the new root exactly like a direct
switch. The full journey (door → descend → switch → a `Cmd-O` in the re-rooted
session) is covered end to end, both keymap conventions, by
`run::tests::keys_capture_browse_door_reaches_a_nested_project_and_returns`,
which also reads the navigator's own mid-journey state out of the sidecar
(`mode: "project_browse"`, `return_to: "switch"`, its `browse_dir` per level).

**History's COMPARISON is reachable through both capture doors, with different
fixture seams.** `overlay_accept:History` is Applied and the timeline is
reachable by `--keys`; the comparison renders when `selected_history_id()`
resolves against a history store.

- **Ordinary `--keys` / `--screenshot` reads the ambient data root**, so
  pointing `XDG_DATA_HOME` at a prepared store gives a capture the full
  timeline *and* comparison. This is the working route, and it is what found two
  of item 116d's six defects. Note it must be the PLAIN door: `args::parse_args`
  computes `hermetic = strict_replay || storyboard || live_app || semantic_json`,
  so adding `--strict-replay` swaps in the sandbox and loses the store along
  with the other three modes.
- **`--screenshot-app` reads an explicit hermetic store.** `--seed-data DIR`
  recursively carries that directory into `data_root()` while preserving
  relative paths, so a fixture at `DIR/history/<fnv1a>.log` lands where
  `history::list` reads it. The walk is deterministic, bounded to the same
  256-file / 4 MiB ceiling as `--seed-tree`, and does not follow symlinks.
  `tests/seed_data_slot.rs::a_nested_seeded_history_log_supports_compare_and_cancel`
  proves the real parse → sandbox → App → keymap → sidecar chain, including
  Enter comparison and Esc cancellation without a source write.

Use the live-App door for a hermetic acceptance capture. The ordinary ambient
route remains useful when the test is deliberately exercising a real prepared
store; do not combine it with strict replay.

## Ownership and acceptance laws

Project reporting uses `run::project_info` at every construction site, and each
storyboard step uses `ReplaySession::current_project_info` through the shared fold.
The laws `the_capture_sidecars_project_location_equals_the_live_apps` and
`every_capture_project_info_literal_is_accounted_for` protect both the builder's
answer and its callers. A correct builder alone does not prove every caller uses it.

The Settings acceptance sweep drives real chords across `SettingId × SettingKind`.
`the_sweep_drives_the_picker_door_and_names_no_app_side_door` forbids direct calls
to setting effect handlers and asserts a minimum number of driven specs.
`the_settings_row_and_its_command_twin_reach_the_same_live_state` checks both doors'
global state and persisted key. Keep those laws even when another capture driver
can photograph an individual case. `Report a Problem` is excluded from live
dispatch because it spawns the OS mail opener; assert its dispatch at the core
seam and record the exclusion explicitly.
