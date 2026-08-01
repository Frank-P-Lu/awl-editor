# docs/harness-reach.md — how far the headless harness reaches

Read this **before writing a Verify clause that asks for a capture**, and before
believing a green `--keys` run about live-`App` state. CAPTURE.md is the
verification contract — what the harness does and what its sidecar means. This
is the map of its *edge*: which transitions a capture can witness, which need a
Rust-level driver, and which are live-only and say so. Queue item 183.

## The defect this map exists to close

Item 180 fixed a real user-visible bug (Switch-project left the Project picker
listing the old workspace's siblings). Its Verify clause said to drive
Switch-project with `--keys` and read the picker from the sidecar. That capture
could not exist: `--screenshot`/`--keys` never build an `App` at all, and every
door into the live one demanded an `&ActiveEventLoop` — a borrow that exists
only inside a running winit loop and cannot be constructed. The brief asked in
good faith for something structurally impossible, because nothing wrote down
where the harness stops.

CLAUDE.md's standing rule is *"the harness stays real: verified behavior is live
behavior — when a bug won't reproduce headlessly, extend the harness toward
reality rather than stubbing around it."* Item 183 did both halves: it extended
the harness where the cost was one narrowed borrow, and it wrote down the rest
rather than leaving a brief author to discover it the way item 180 did.

## The three tiers, and what each one can prove

| Tier | What runs | Driven by | Oracle |
| --- | --- | --- | --- |
| 1 — shared core | `actions::apply_transition` | `--keys`, `--storyboard`, `ReplaySession` | sidecar JSON + PNG pixels |
| 2 — live effect interpretation | `App::apply` → `apply_live_effect` / `apply_overlay_accept` | `App::press_chords_headless` — `--screenshot-app`, or `press_spec_headless` in a Rust test | **sidecar JSON + PNG pixels** (item 188), or Rust assertions on `App` state |
| 3 — window / surface / loop | the winit callbacks below | a real window only | live human, `--live-script`, `--soak-gpu` |

**Tier 1 is the tier CAPTURE.md documents.** A capture replays real chords
through the real keymap into the real `apply_transition`, and the sidecar is a
faithful state oracle for everything that transition owns: buffer, selection,
search, zoom, folds, the summoned-overlay `Journey`, page state, theme.

**Tier 2 is new as of item 183.** `App::apply` no longer takes an
`&ActiveEventLoop`; it takes `app::Exit`, the one capability it actually used
(`exit()`). The whole input-dispatch chain followed, so a test can now drive a
hermetic `App` from real chords — `App::press_spec_headless("s-S-p Backspace
Enter")` goes `dispatch_pressed_key` → keymap resolve → `apply` → the live
effect interpreter, the same code a physical keypress takes. Nothing stands in
for the live path there; it *is* the live path, minus the window.

**Tier 2 has a sidecar as of item 188 — `--screenshot-app`.** It was the half
item 183 left open: a real `App` was drivable but unphotographable, so every
live-`App`-only outcome had to be asserted in Rust rather than read from the one
oracle the rest of the project uses. `awl --screenshot-app OUT.png [file] --keys
"SPEC"` drives the SAME chord stream into a real headless `App` and writes an
ORDINARY PNG + sidecar from its state — same schema, same
`capture::sidecar::write_sidecar`, same blocks. The one difference in the
artifact is the top-level `driver` field: `"live-app"` here, `"replay"` for every
tier-1 capture. Mechanism: `main/run/live_app.rs` (the mode),
`app/capture_state.rs` (the `App`'s side of the fold + its capture constructor),
`run::fold_capture_state` (the ONE per-frame fold, shared with the storyboard
stepper), `run::project_info` (the ONE project-block builder, item 183).
CAPTURE.md's "Live-`App` capture" section is the contract.

Three properties worth knowing before you reach for it:

- **It is hermetic, unconditionally.** A live `App` PERFORMS the writes a replay
  only records, so the mode is a `crate::scenario` door: `args::parse_args` swaps
  the process fs to the seeded sandbox before the config loads, exactly as a
  storyboard run does.
- **It skips nothing.** `replay_skips` is empty by construction — there is no
  capability to lack, because the effect interpreter that runs is the live one.
  A live-`App` capture of a spec whose tier-1 twin warns on stderr is the point
  of the mode.
- **It does not reach tier 3.** No window, no surface, no event loop (`gpu` is
  `None`; the harness renders the App's buffer through its own offscreen
  pipeline, as `--screenshot-frames` does). And it is still a STATE oracle:
  appearance claims are asserted over the PNG's pixels, per CLAUDE.md's Wagtail
  tripwire, exactly as at tier 1.

Rust assertions on `App` state remain the purest seam for a sweep
(CLAUDE.md's unit > sidecar > capture ladder) — item 114's tier-2 settings sweep
is not retired by this. What is new is that a tier-2 claim can now be handed to a
sidecar oracle, a pixel oracle or a vision smoke when that is what the claim
needs.

## Tier 3 — the live-only census, exactly

Every function under `src/app.rs` + `src/app/**` whose signature takes an
`&ActiveEventLoop`. Twelve, in four files, and the list is not maintained by
hand: `app::tests::source_audit::the_active_event_loop_census_is_exact_and_the_input_chain_is_free_of_it`
scans the source and fails on any new one.

| Where | Functions | Why the loop is genuinely needed |
| --- | --- | --- |
| `app.rs` | `drive_gpu_soak` | `--soak-gpu` drives a real window |
| `app/gpu_recovery.rs` | `rebuild_gpu` | recreates the window-bound renderer |
| `app/lifecycle.rs` | `user_event`, `resumed`, `suspended`, `window_event`, `exiting`, `about_to_wait` | winit's own `ApplicationHandler` signatures |
| `app/window.rs` | `handle_gpu_fault`, `handle_gpu_frame_outcome`, `on_resized`, `on_redraw_requested` | rebuilds the surface, sets `ControlFlow` |

The same law asserts the **input-dispatch chain is empty** — `app/apply.rs`,
`app/input/keys.rs`, `app/input/mouse.rs`, `app/input/drags.rs`, `app/menu.rs`,
`app/probe.rs` may never take an `&ActiveEventLoop` again. One such parameter
re-blinds every transition reachable through it, in one line, and nothing else
in the suite would notice.

Two capabilities have already escaped this tier by being narrowed rather than
stubbed, and they are the pattern to copy: `app::Scheduler` (the
`about_to_wait` debounce/settle body, steppable under a `VirtualClock` — that
is what `--screenshot-frames` drives) and `app::Exit` (this item).

## Tier 2 — the effect table

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

<!-- reach-table:begin -->
| Effect | Class |
| --- | --- |
| `add_to_dictionary` | Unsupported |
| `check_for_updates` | Intercepted |
| `clipboard_paste_image` | Intercepted |
| `clipboard_write` | Intercepted |
| `copy_pulse` | Applied |
| `daemon_notify_finished` | Intercepted |
| `delete_squash` | Applied |
| `download_file` | Intercepted |
| `duplicate_note` | Unsupported |
| `edit_streak` | Applied |
| `export` | Intercepted |
| `finish_buffer` | Unsupported |
| `finish_save` | Unsupported |
| `flush_writing_streaks` | Applied |
| `follow_link` | Intercepted |
| `gulp` | Applied |
| `insert_date` | Applied |
| `jump_to_line` | Applied |
| `keep_version` | Unsupported |
| `last_buffer` | Unsupported |
| `line_land` | Applied |
| `new_document` | Applied |
| `none` | Applied |
| `notice_clear` | Applied |
| `notice_sticky` | Applied |
| `notice_toast` | Applied |
| `open_credits` | Applied |
| `open_guide` | Applied |
| `open_settings` | Applied |
| `overlay_accept:Assets` | Unsupported |
| `overlay_accept:Browse` | Unsupported |
| `overlay_accept:Caret` | Applied |
| `overlay_accept:CjkLang` | Applied |
| `overlay_accept:Command` | Unsupported |
| `overlay_accept:Date` | Applied |
| `overlay_accept:Dictionary` | Applied |
| `overlay_accept:Goto` | Applied |
| `overlay_accept:History` | Applied |
| `overlay_accept:InsertLink` | Unsupported |
| `overlay_accept:KeepName` | Unsupported |
| `overlay_accept:Keybindings` | Unsupported |
| `overlay_accept:MoveDest` | Unsupported |
| `overlay_accept:Project` | Applied |
| `overlay_accept:Rename` | Unsupported |
| `overlay_accept:Settings` | Unsupported |
| `overlay_accept:Spell` | Unsupported |
| `overlay_accept:Theme` | Applied |
| `persist_caret_mode` | Applied |
| `persist_menu_bar` | Applied |
| `persist_outline` | Applied |
| `persist_page_mode` | Applied |
| `persist_page_reset` | Applied |
| `persist_page_width` | Applied |
| `persist_spellcheck` | Applied |
| `persist_typewriter` | Applied |
| `persist_writing_nits` | Applied |
| `quit` | Unsupported |
| `rebind_commit` | Unsupported |
| `rebind_reset` | Unsupported |
| `recoil` | Applied |
| `redraw` | Applied |
| `rename_note_commit` | Unsupported |
| `report_problem` | Intercepted |
| `reshape` | Applied |
| `run_action` | Applied |
| `save` | Unsupported |
| `setting_path_pick` | Unsupported |
| `setting_range_step` | Applied |
| `setting_toggle` | Unsupported |
| `setting_value_commit` | Unsupported |
| `show_about` | Applied |
| `sync_view` | Applied |
| `trash_asset` | Intercepted |
| `type_impact` | Applied |
| `zoom_changed` | Applied |
<!-- reach-table:end -->

### Two asymmetries the table will not shout at you

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
way. One key stays Unsupported even under Isolated: `SettingToggle{key:
"keymap"}` needs a LIVE keymap rebuild so a later chord in the same replay
resolves against the new flavor, a capability no filesystem grant supplies —
the same reason `rebind_commit`/`rebind_reset` never promote either. That key
is precisely what item 188's `--screenshot-app` was proved on: the door with
no possible capability grant is the one where a live-`App` capture is the only
sidecar there will ever be.
`setting_range_step` was already Applied before this item, because the value
change itself already happened in the core.

**`overlay_accept:Project` is Applied, no residue (closed — item 189).** The
accepted root re-derives the sidecar's whole project block through one builder
(`run::project_info`), so a capture reports the new root *and* the new
workspace. `ReplaySession` used to hold its `root`, `workspace`, and file-index
`corpus` fixed for the session's whole lifetime, so a chord that ran **after**
the accept still read the launch root's tree: a `Cmd-O` following a
Switch-project listed the launch root's files even though the sidecar's own
accepted-location block was already correct. `ReplaySession::
resync_project_location` (`main/run/location.rs` — the module item 183
already carries the rest of this exact derivation in) is now the one owner
invoked the moment the accept fires — it rebuilds `corpus`
(`crate::index::build_index`) and re-resolves `workspace`
(`resolve_workspace`, against the SAME raw `--workspace` flag the constructor
used) before `root` itself moves, so a
chord applied after the accept sees the new tree exactly like live. Covered
end to end, both keymap conventions, by
`run::tests::keys_capture_switch_project_then_goto_lists_the_new_roots_files`;
the same-parent and no-parent (filesystem-root) edges item 180 named are swept
by `run::tests::resync_project_location_same_parent_switch_still_rebuilds_the_corpus`
and `run::tests::resync_project_location_no_parent_root_falls_back_to_itself_not_the_old_workspace`.

## What went wrong here once, so it does not again

The capture path carried its own hand-rolled copy of "what does this root
imply", and that copy still had item 180's bug after item 180 fixed the App's:
the Project-accept site re-derived `name`/`branch`/`dirty` from the accepted
root while carrying the **launch** root's `workspace` forward. Reproduced on a
real capture before the fix — `--keys "s-S-p Backspace Enter Enter Enter"` into
`/new-ws/proj-b` reported `root: /new-ws/proj-b` beside `workspace: /old-ws`.
An oracle that lies about the exact transition it is asked to witness is worse
than one that admits it cannot see.

Two laws now hold it: `app::files::tests::the_capture_sidecars_project_location_equals_the_live_apps`
pins the builder to the live `App`'s derivation across item 180's whole axis,
and `run::tests::every_capture_project_info_literal_is_accounted_for` accounts
for every construction site — because a parity test proves the builder is
right, never that every call site uses it, and the bug was in a call site.

## Item 114 landed on this map — what it actually used

The split below was written before item 114 started; this is what it built
against it, so a later reader can check the map against a real consumer.

- **Tier 1, as predicted.** The lifecycle laws are driven through
  `actions::apply_transition` and read the table's own vocabulary
  (`actions::tests::workspace_item114`); the presentation laws render a real
  pipeline and assert pixels (`render::tests::workspace_item114`). Nothing there
  needed a capability the map said did not exist.
- **Tier 2, as predicted, and it was the larger half.**
  `app::tests::workspace_item114` drives every `SettingId × SettingKind` by real
  chords into a hermetic `App` over an `InMemoryFs`. It carries its own
  anti-vacuity law, `the_sweep_drives_the_picker_door_and_names_no_app_side_door`,
  which forbids the sweep's source from naming `setting_toggle` /
  `setting_value_commit` / `setting_path_pick` / `range_persist` at all — the
  substitution the asymmetry below invites — and asserts a floor on how many
  chord specs it actually presses, so the ban cannot be satisfied by an empty
  file.
- **The asymmetry is now covered from both sides.**
  `the_settings_row_and_its_command_twin_reach_the_same_live_state` drives the
  same setting through its COMMAND (Applied, capturable) and through its Settings
  ROW (Unsupported, not), and asserts they land on the same live global and the
  same config key. That is what makes "the picker door works" a claim about the
  picker door rather than an inference from the command door's captures.
- **One deliberate, named hole.** `Report a Problem` is not driven live: it hands
  a `mailto:` URL to `App::follow_link`, which spawns the OS opener. It changes no
  editor state and no config; the row's dispatch is asserted at the core seam
  instead, and the sweep records the exclusion in its own coverage list rather
  than skipping it silently.

## For item 114 (Settings as the first summoned workspace)

Item 114's Verify clause asks for "deterministic capture/sidecar" oracles over
a workspace that lives on the live `App`. Read that clause against this map
before starting:

- **The lifecycle is tier 1 and fully capturable.** Item 173 deliberately put
  `overlay::Journey` in the shared core, not in the app-private
  `WorkspaceState`, precisely so `ReplaySession` would not need a second copy.
  Entry, focus transfer, child suspend/return, Back, exit and the parked-parent
  position all replay under `--keys` and land in the sidecar's overlay block.
  Every state/focus/back law item 114 wants is a real capture.
- **Setting *changes* were not, when item 114 was written.** `SettingToggle` /
  `SettingValueCommit` / `SettingPathPick` were Unsupported outright: a
  `--keys` capture would show the Settings row selected and the query typed,
  and would **not** show the value changed or persisted. "Preserve every
  setting's live apply, persistence, range, exact-entry, toggle and
  sub-picker behavior" was a tier-2 obligation. Item 114 landed against
  exactly that framing (`App::press_spec_headless` on a hermetic `App` over an
  `InMemoryFs`, asserting `App`/config state directly), and item 190
  (afterward) granted the trio Isolated-filesystem capability — see "Two
  asymmetries" above. That grant does not retire item 114's tier-2 laws (a
  hermetic `App` sweep is still the purest seam for "every `SettingId ×
  SettingKind`", and the anti-vacuity law there still bans the sweep from
  reaching the App-side doors directly); it opens a SECOND, narrower route —
  a strict/scenario capture over an isolated sandbox — for the specific claim
  "the picker door writes for real", with its own hermetic proof in
  `main/tests.rs`. Item 188 opens a THIRD, and the widest: `--screenshot-app`
  photographs the LIVE `App`'s own state, so a settings change is readable from
  an ordinary sidecar (`overlay.bindings[row]`, plus whatever block the setting
  feeds — `project.keymap_flavor` for the Keymap row). That route needs no
  capability grant at all, because nothing is being stood in for. The worked
  example is `run::live_app::tests::
  a_live_app_capture_photographs_a_keymap_flip_an_ordinary_capture_cannot_see`,
  which asserts BOTH sides: the live-`App` sidecar reports the flip, and the
  same spec through the ordinary `--keys` door still reports the old flavor and
  records the skip.
- **Theme and Caret audition are tier 1** (`overlay_accept:Theme` / `:Caret`
  are Applied — they set their process-global in the core), so the
  suspend-audition-return journey and the commit/revert parity are capturable
  end to end, including the returned-to position.
- **Layout, geometry and appearance stay tier 1.** Wide/narrow stages, the
  category rail, zoom and DPI are all rendered from `ViewState`; capture them
  normally, and assert appearance with pixel arithmetic, never the sidecar
  (CLAUDE.md's Wagtail tripwire).
- **The honest split to write into the brief:** state/focus/back/journey →
  capture + sidecar; every `SettingId × SettingKind` value change → tier-2 Rust
  laws; look and feel → capture pixels + a vision smoke.

## Left for a follow-up

Named here rather than quietly absorbed:

1. ~~**No sidecar from a live `App`.**~~ Closed by item 188 — `--screenshot-app`,
   see "Tier 2 has a sidecar" above. **What that item left, named rather than
   absorbed:** the mode captures ONE frame at the END of its chord stream, like
   `--screenshot` does. There is no per-step live-`App` film (the storyboard's
   `--storyboard` equivalent) and no `--strict-replay` analogue, because a live
   `App` has nothing to be strict about — it skips nothing. A mid-stream
   live-`App` trajectory would need the storyboard runner taught a second driver;
   nothing asks for it yet. The mode also drives the chord stream only —
   `--sel`/`--scroll`/`--search`/`--preedit`, the deterministic verification
   hooks that OVERRIDE replayed state, are deliberately not folded in, because on
   this door the App owns that state and an override would be the harness lying
   about the editor it is photographing.
2. ~~**`ReplaySession` re-scoping.**~~ Closed by item 189 — see
   `overlay_accept:Project`'s entry above. **Still open:** the storyboard
   runner (`main/story.rs::run_storyboard`) computes its own `capture::
   ProjectInfo` once before the run and folds the SAME value into every
   step's sidecar (`step_opts`), so a storyboard whose steps include a
   Switch-project would show the stale project block on every step after
   it — the identical defect shape, one call site removed. No storyboard
   fixture drives a Switch-project today, so this is undiagnosed rather than
   reproduced; flag it before writing one.
3. ~~**The Unsupported bucket is a work list, not a verdict.**~~ The settings
   trio's own grant closed by item 190 — see "Two asymmetries" above and
   `main/run/settings_effects.rs`. **Still true in general:** a row in the
   table above is Unsupported because the replay owns no capability for it,
   not because the behaviour is unobservable in principle — item 171's
   `FilesystemCapability` shows the shape (`save`/`finish_save`, then the
   settings trio) for granting one to a future effect that needs it.
