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
  storyboard run does. The sandbox is seeded from exactly the paths the command
  line names — the CLI `file`, `--config`, and (item 204) `--seed-data DIR`, whose
  files land at awl's own `fs::data_root()` paths so a run can START from state
  awl already had. See "Left for a follow-up" #3.
- **It skips nothing.** `replay_skips` is empty by construction — there is no
  capability to lack, because the effect interpreter that runs is the live one.
  A live-`App` capture of a spec whose tier-1 twin warns on stderr is the point
  of the mode.
- **It does not reach tier 3.** No window, no surface, no event loop (`gpu` is
  `None`; the harness renders the App's buffer through its own offscreen
  pipeline, as `--screenshot-frames` does). And it is still a STATE oracle:
  appearance claims are asserted over the PNG's pixels, per CLAUDE.md's Wagtail
  tripwire, exactly as at tier 1.
- **`--capture-size`/`--capture-dpi` are honored (item 334 — closed a gap this
  map used to leave unrecorded).** Before, `LiveAppSpec` carried no canvas/dpi
  fields at all, so the flags parsed, validated as "honored" (`--screenshot-app`
  fell through to the plain-`--screenshot` bucket in the CLI's own hook-usage
  check), and were then silently dropped on the floor before the frame ever
  rendered — every geometry claim made against a `--screenshot-app` PNG before
  this closed was measuring the byte-stable 1200x800 default no matter what
  canvas the invocation asked for. `LiveAppSpec` now carries both, threaded onto
  the `CaptureOpts` `capture_live_app` hands the SAME `capture_with` path
  `--screenshot` renders through, so the meaning is identical on both doors: a
  narrower physical canvas re-wraps the SAME document into more visual rows
  (`layout.rows`), and a `WxH` device canvas at `--capture-dpi N` is the SAME
  logical `(W/N)x(H/N)` window as the unscaled canvas — proved by matching
  visual-row wrap counts between `1200x800 @1` and `2400x1600 @2`
  (`run::live_app::tests::a_live_app_capture_honors_capture_size_and_the_dpi_meaning_holds`).
  The per-frame render hooks (`--sel`/`--zoom`/`--scroll`/`--preedit`/
  `--search*`) and `--default-folder` stay REFUSED on this door rather than
  silently dropped — `LiveAppSpec` has no slot for any of them, and the first
  two exist because the live App owns that state via real driving; an override
  would misrepresent the editor being photographed. `--root`/`--workspace` were
  already threaded and stay so.

Rust assertions on `App` state remain the purest seam for a sweep
(CLAUDE.md's unit > sidecar > capture ladder) — item 114's tier-2 settings sweep
is not retired by this. What is new is that a tier-2 claim can now be handed to a
sidecar oracle, a pixel oracle or a vision smoke when that is what the claim
needs.

### A PICKER ROW'S GEOMETRY IS NOW A SIDECAR FACT, ON BOTH DOORS

`overlay.window.band` and `overlay.window.rows` (schema `/201`) publish the
PLANNED rect of every candidate display line — `{ display, item, x, y, w, h,
selected }` in physical pixels. Both doors carry it, because both write through
the one `capture::sidecar::write_sidecar`.

**What this changes for a Verify clause.** "The selected row's band sits at the
right y", "row 3 is clickable exactly where it is drawn", "the rows keep a fixed
pitch under a filter" and "a row's control does not overlap its label" used to be
answerable only by measuring the PNG — an appearance oracle answering a geometry
question, which cannot distinguish *drawn in the wrong place* from *drawn
correctly in a colour this world happens to hide*. Ask the sidecar for the
geometry and the PNG for the appearance.

**Where the line still is.** This is a STATE oracle, and the standing tripwire
applies unchanged: a perfectly reported rect can be drawn invisibly. Visible,
distinct and legible remain claims about pixels. The rect is what the plan
DECIDED — the same object the draw emitters and `overlay_row_at` read, so it
cannot disagree with them, but it says nothing about a downstream emitter that
declines to draw at all.

## Tier 3 — the live-only census, exactly

Every function under `src/app.rs` + `src/app/**` whose signature takes an
`&ActiveEventLoop`. Fourteen, in five files, and the list is not maintained by
hand: `app::tests::source_audit::the_active_event_loop_census_is_exact_and_the_input_chain_is_free_of_it`
scans the source and fails on any new one.

| Where | Functions | Why the loop is genuinely needed |
| --- | --- | --- |
| `app.rs` | `drive_gpu_soak` | `--soak-gpu` drives a real window |
| `app/frame/accessibility.rs` | `FrameRuntime::install_accessibility`, `AccessibilityRuntime::install` | the AccessKit adapter binds the real loop + window, and must do so BEFORE the window becomes visible (macOS caches a newly ordered-in window's accessibility parent) |
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

**`--capture-size`/`--capture-dpi` are honored (item 339 — the identical gap
item 334 closed on `--screenshot-app`, found by that item's own lane while
diagnosing and left alone until now).** Before, `Mode::ScreenshotFrames`
carried no canvas/dpi fields at all and fell into the CLI's plain-`--screenshot`
hook bucket, so both flags parsed, validated as "honored", and were then
rendered through a bare `CaptureOpts::default()` regardless of what was typed —
every N-frame capture was the byte-stable 1200x800 default no matter the
canvas asked for. Worse, `capture::capture_frames_async` itself never called
`pipeline.set_dpi`, so even a correctly-threaded `--capture-dpi` would have
been a no-op at the renderer — a second, independent instance of the same
"accepted and ignored" shape one layer deeper. Both are fixed: `Mode::
ScreenshotFrames` now carries `canvas`/`dpi`, `CaptureKind::ScreenshotFrames`
gives the door its own accurate hook list (canvas + dpi honored; the per-frame
render hooks, `--root`, `--workspace`, `--default-folder` and `--keys` all
refuse loudly — this door's document is a stationary backdrop loaded straight
off disk, with no replay and no project resolution at all, so none of those
has anywhere to land), and `capture_frames_async` calls `set_dpi` exactly
where `capture_async` does. Proved by measuring geometry, mirroring item 334's
form: a document with one long wrapping line renders 12 visual rows at
1200x800, 30 at 640x800 (genuinely more reflow, not a relabelled number), and
12 at 2400x1600 @ dpi 2.0 — identical to the 1200x800 @ dpi-1 baseline, the
documented dpi meaning holding on this door too. Reverting the fix (either
half) reproduces the exact original collapse: every canvas reports 1200x800
regardless of the flags typed.

**Every `Mode::*` capture door, and whether it carries canvas/dpi (item 339's
own audit, so a third gap of this shape does not go unlisted):**

| Door | Canvas/dpi? | How |
| --- | --- | --- |
| `Screenshot` | Yes | `CaptureOpts.canvas`/`.dpi` |
| `ScreenshotMotion`/`-Vertical`/`-Diagonal` | No | refused loudly (`CaptureKind::Motion`), never silently dropped |
| `ScreenshotFrames` | Yes (item 339) | own `canvas`/`dpi` fields → `CaptureOpts` |
| `ScreenshotApp`/`SemanticJson` | Yes / N/A | `LiveAppSpec.canvas`/`.dpi` (item 334); `SemanticJson` renders no PNG, so its own spec always carries `None` and the CLI refuses the flag combination before either mode is built |
| `CaptureTimeline`/`CaptureHeld` | Yes | own `canvas`/`dpi` fields |
| `Storyboard` | No | refused outright — a storyboard run sets no `out`, so it resolves to `CaptureKind::Windowed`, which already refuses both flags |
| `Windowed` | No | a real OS window; both flags refused for the same reason |
| Every `Bench*` mode, `SoakGpu` | No | same refusal as `Storyboard` (none of these set `out` either); each bench's own fixed internal canvas is documented on the `Mode` variant, not driven by these flags |

No third silently-discarding door was found: every other mode either threads
the flags for real or is refused by the existing `unused_hooks` classification
because it never sets `out` in the first place.

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

⚠️ **The three `notice_*` rows below read `Applied` and now mean it.** They were
`Applied` while the replay's interpreter *discarded* every notice — reported as
performed, drawn nowhere, and not recorded as a skip either, which is the worst of
the three classifications to be wrong about. An ordinary `--keys` capture now
latches the notice and photographs it, and the sidecar reports it as
`notice: { text, kind }`. A latched Toast never expires headlessly because there is
no clock, which matches a GPU-less live `App` (`App::set_toast_notice` arms no
deadline without a surface).

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
| `open_reference` | Applied |
| `open_settings` | Applied |
| `overlay_accept:Assets` | Unsupported |
| `overlay_accept:Browse` | Unsupported |
| `overlay_accept:Caret` | Applied |
| `overlay_accept:CjkLang` | Applied |
| `overlay_accept:Command` | Unsupported |
| `overlay_accept:Conflict` | Unsupported |
| `overlay_accept:Context` | Unsupported |
| `overlay_accept:Date` | Applied |
| `overlay_accept:Dictionary` | Applied |
| `overlay_accept:ExportDest` | Unsupported |
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
| `resolve_keep_mine` | Unsupported |
| `resolve_take_theirs` | Unsupported |
| `review_external_change` | Unsupported |
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

**History's COMPARISON inverts the usual tier ordering: the ORDINARY capture
reaches it and `--screenshot-app` cannot.** Everywhere else the live-`App` door
is the wider one, so this reads backwards and has already been briefed
backwards once (item 116d, 2026-08-03). `overlay_accept:History` is Applied, and
the timeline is reachable by `--keys` alone; but the COMPARISON only renders
when `selected_history_id()` resolves, which needs a real history STORE. The two
doors get one from opposite directions:

- **Ordinary `--keys` / `--screenshot` reads the ambient data root**, so
  pointing `XDG_DATA_HOME` at a prepared store gives a capture the full
  timeline *and* comparison. This is the working route, and it is what found two
  of item 116d's six defects. Note it must be the PLAIN door: `args::parse_args`
  computes `hermetic = strict_replay || storyboard || live_app || semantic_json`,
  so adding `--strict-replay` swaps in the sandbox and loses the store along
  with the other three modes.
- **`--screenshot-app` cannot get one at all.** The mode is hermetic by
  construction, and its only data-root seed slot, `--seed-data DIR`
  (`scenario::data_root_seeds`), is **flat** — it `read`s each direntry and
  writes it at `data_root().join(file_name)`, so a directory silently yields
  nothing. The history store is one level DOWN — `history::store::history_root()`
  is `data_root().join("history")` and a file's log is
  `<history_root>/<fnv1a>.log` — so no `--seed-data` layout can place a log
  where `history::list` looks.

So follow-up #3's "every consumer of the data root puts a plain file directly
under it" is not true, and `history` is the counter-example. Teaching
`data_root_seeds` to recurse would close the gap; nothing has needed it yet,
and it is named here rather than absorbed. Until then, a Verify clause wanting
a *live-`App`* sidecar over a History comparison is asking for something that
does not exist — use the seeded ordinary capture, or assert at tier 2 in Rust
(`app::tests::history`).

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
3. ~~**No capture tier reaches an EXTERNAL-CHANGE CONFLICT.**~~ **Closed by item
   204 slice 2 — `--seed-data DIR`.** The measurement slice 1 recorded here was
   right and is kept, because it is what the fix was designed against.

   The conflict is latched on the live `App`'s per-buffer disk baseline, so
   **tier 1 cannot hold it at all** — that has not changed and will not. Tier 2
   had two possible ways in and both were shut:

   - **Raise one during the run.** Still impossible by construction: the change
     has to come from OUTSIDE awl, and a capture drives chords only. There is no
     step at which another writer touches the file. This one is not fixable and
     was not fixed.
   - **Start already conflicted, from the recovery record.** This is the one that
     opened. `--screenshot-app` is a `crate::scenario` door whose sandbox was
     seeded by `scenario::cli_seeds` from exactly TWO paths — the CLI `file` and
     `--config`. Nothing wrote awl's data root, so `recovery::read()` found
     nothing: driven for real, a record at `$XDG_DATA_HOME/awl/unresolved-change.md`
     beside a diverging file photographed the DISK text and no conflict. The
     store was not merely unseeded, it was **unseedable through the door**.

   **What opened it** is a THIRD seed slot of the same shape as the two that
   were already there — `--seed-data DIR` (`scenario::data_root_seeds`), whose
   files are carried into the sandbox at awl's own `fs::data_root()` paths, so
   `recovery::read()`, `fs::scratch_stash_path()` and `session.toml` find them
   where they look. It is a narrowing, not a stub: the harness NAMES the store
   on the command line rather than reading the machine's real one, so a
   capture's starting state is written down in its own invocation and a
   developer's remembered session can never leak into it. Flat — entries one
   directory deep, directories skipped — which covers every consumer this item
   needed (`recovery`, the scratch stash, `session.toml`) but **not every
   consumer there is**: `history::store::history_root()` is
   `data_root().join("history")`, a SUBDIRECTORY, so the history store is
   structurally unseedable through this slot. See
   "Three asymmetries" above; that is why a History comparison is the one
   surface an ordinary capture photographs and `--screenshot-app` does not.
   Refused outside a hermetic
   door rather than silently ignored, since a run that named a store and did not
   get one would photograph the wrong starting state.

   ```sh
   awl --screenshot-app OUT.png DOC.md --seed-data DIR --keys "SPEC"
   ```

   **What the oracle can now say.** ⚠️ **The chrome `notice` DOES have a sidecar
   field now** — `notice: { text, kind }`, schema `/200` — and the sentence this
   paragraph used to carry ("a notice is transient and a single slot, so it was
   the wrong thing to build a state oracle on") was the invariant that left every
   capture door structurally unable to photograph a channel with ~ten production
   callers. A driven editor that had raised `saved` produced a PNG byte-identical
   to one that had raised nothing. The transience argument was about the LIVE
   clock; a headless capture has none, so a notice latched during a replay simply
   stays. Beside it, item 204 slice 2 added the PERSISTENT affordance's own field,
   **`gutter.changed`** (schema `/197`), plus **`overlay.preview_view`** — the
   `ComparisonView` tag, which is the one fact that tells three previews of ONE
   subject apart. So a live-`App` capture of a conflict reports: the affordance
   is up, which surface is open, which of its three views, and the previewed
   prose itself. Appearance claims about the affordance stay pixel arithmetic
   (`render::tests::chrome_overlay::the_changed_elsewhere_affordance_reports_grows_and_reads_stronger_than_the_name`),
   per the Wagtail tripwire.

   Proved on the REAL binary, both ways round, in
   `tests/seed_data_slot_item204.rs`: with the slot the capture starts
   conflicted; without it, the identical command still photographs the disk text
   and no conflict.

4. ~~**The Unsupported bucket is a work list, not a verdict.**~~ The settings
   trio's own grant closed by item 190 — see "Two asymmetries" above and
   `main/run/settings_effects.rs`. **Still true in general:** a row in the
   table above is Unsupported because the replay owns no capability for it,
   not because the behaviour is unobservable in principle — item 171's
   `FilesystemCapability` shows the shape (`save`/`finish_save`, then the
   settings trio) for granting one to a future effect that needs it.
