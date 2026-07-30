# docs/app-domains.md — the `App` ownership map

Read this before adding a field to `App`, before adding an `impl App` block in a
new module, and before moving state between subsystems. `ARCHITECTURE.md` is the
module map (which file holds what); this is the *state* map (which owner holds
which invariant). Queue item 172.

## The defect this map exists to close

`app.rs` declares `pub struct App` with 107 fields. Every one is private to
`crate::app`, which means every one of the ~24 `impl App` blocks under `src/app/`
can read and write all 107. The physical file split (item 56) made the code
navigable without making any invariant *owned*: a rule spread across four
modules is still spread across four modules after the split, it just reads
tidily. Two consequences show up in the census below:

- **Conjunction drift.** A rule expressed as `a && b.is_none() && c.is_none()`
  gets re-typed at each site. The summoned-layer precedence rule was written out
  by hand at five sites before this round.
- **Derived state by convention.** `root` implies `project` and `file_index`;
  nothing makes the implication hold, so `switch_project` re-derives two of the
  three and `reload_config` re-derives the fourth.

## Census method

Every count below is `self.<field>` / `app.<field>` occurrences under
`src/app.rs` + `src/app/**`, split into production and test files. Only that
subtree can see the fields at all (they are private to `crate::app`), so a
whole-crate grep over-counts badly: `src/main/run.rs`'s `ReplaySession` and
`render`'s `TextPipeline` carry same-named fields of their own (`overlay`,
`search`, `zoom`, `dpi`, `preedit`) that are unrelated state. Reproduce with:

```sh
grep -rn 'self\.<field>\b' src/app.rs src/app/
```

The scan must tolerate line-wrapped access (`&self\n    .cli_default_folder`) —
a contiguous `self.<field>` regex silently reports zero for a field that is only
ever read across a line break, which is how the first pass of this census
mis-declared `cli_default_folder` dead.

Totals: **1,310 production references** and **586 test references** inside
`src/app.rs` + `src/app/**`.

## The owners

Names follow queue item 172, with two corrections the census forced (see
"Where the item's premise was wrong").

| Owner | Fields | Prod refs | Reach (prod files) | Status |
| --- | --- | --- | --- | --- |
| `WorkspaceState` (summoned UI) | 3 | 99 | 14 | extracted — slice 1 |
| `PersistenceRuntime` (save feedback) | 5 | 23 | 6 | extracted — slice 2 |
| `DocumentSession` | 4 | 363 | 21 | mapped |
| `InputState` | 27 | 164 | 13 | mapped |
| `ProjectLocation` | 9 | 51 | 9 | mapped |
| `RenderRuntime` | 25 | 277 | 23 | mapped — held by item 174 |
| `FrameScheduler` | 12 | 123 | 12 | mapped |
| host / lifecycle (stays on `App`) | 22 | 210 | 19 | stays |

107 fields, 1,310 production references, no field in two owners and none
unassigned — the classification is exhaustive by construction, see below.

`src/app/tests/domains.rs` carries the same table as executable data with a
no-wildcard match over the owner roster, so a new `App` field fails the suite
until it is consciously assigned to an owner, and a new owner variant fails to
compile until it is described.

### `WorkspaceState` — the summoned-UI layer

`overlay` · `search` · `popover_open`

The three summoned surfaces. Not one field is independent: they form a
**precedence ladder** — a modal overlay outranks the find/replace panel, which
outranks the reveal-on-select format popover, which outranks the editor. Before
this round the ladder was five hand-written conjunctions — `sync_view`'s
popover gate, and in `app/input/mouse.rs` the Cmd-click link follow, the
popover-button press, the summon-on-release, and the overlay-before-search
`else if` in the press dispatch — plus five independent writers of
`self.overlay = None` / `self.search = None` / `self.popover_open = false`.

Highest dispersion per field in the whole struct: 33 production references per
field, spread over 14 files (`gpu` has more references but is one repeated
shape, see `RenderRuntime`).

**Item 173 replaced the `overlay` slot with `overlay::Journey`** — the closed
summoned-UI LIFECYCLE — and added the ladder's fourth rung, `Layer::Workspace`,
between `Search` and `Overlay`. The lifecycle deliberately does NOT live inside
`crate::app`: `WorkspaceState` is `pub(in crate::app)` and the headless `--keys`
replay (`main/run.rs::ReplaySession`) cannot see it, so an app-private lifecycle
would have forced the replay to keep a second copy — the exact defect this map
exists to close. `WorkspaceState` owns the one live instance and asks it for a
single closed fact (`overlay::Rung`), so the ladder reads the lifecycle instead
of re-deriving it. Six loose `OverlayState` fields retired into typed payloads:
`return_to` + `setting_path_key` → `Parked` + `Bind`, `original_theme` +
`original_caret` + `original_caret_was_auto` → `Audition`, and `diff_focus` →
the `Surface::WorkspaceDetail` stage (the field is storage; the lifecycle is the
sole writer, fenced by a law).

### `PersistenceRuntime` — save feedback and the autosave debounce

`autosave_dirty_at` · `autosave_saved_version` · `autosave_last_ok` ·
`last_saved_ok` · `title_dirty`

The App-global half of saving. (The per-buffer half — `doc_saved_version`,
`scratch_saved_version`, `disk_mtime`, `scratch_mtime`, `doc_autosave_at` —
correctly lives in `files::BufferExtra`, travelling with the active slot;
item 56 got that cut right.) The invariant nothing held: **an armed debounce
stamp and a stale saved-version must be retired together.** Six copies of two
rules over two fields that only make sense as one ledger: the "is a write
owed" comparison at `viewstate.rs`'s arming check, `is_document_dirty`'s fresh
branch and `flush_note`'s skip check; the "record the version and disarm"
pairing at `autosave_note`+`flush_note`, `convert_scratch_and_save` and
`start_fresh_document`. Plus two more in the engine, where
`autosave_last_ok`/`last_saved_ok` were stamped as separate statements at both
`Ok` arms and had to stay in lockstep.

`autosave_saved_version` is keyed by `buffer.version()` with no buffer identity
— exactly CLAUDE.md's cache-key tripwire, and versions restart at 0 per open.
It IS safe today, and the argument is worth recording because it is not
obvious: a buffer becomes "unnamed fresh" only through `Buffer::start_fresh_doc`
(one caller, which resets the ledger in the same breath) or
`Buffer::set_note_dir` (one caller, which records a write immediately). That is
a two-call-site argument, not an invariant — so `PersistenceRuntime`'s reset
clears the version as well as the timer, and its law sweeps the version values
that collide.

### `DocumentSession`

`active` · `buffer_registry` · `prev_file` · `spell`

363 production references, 21 files — the largest domain by reach, and the one
already partly owned: `active` is a whole-slot `Entry<BufferExtra>` whose sole
constructor/destructor is `files/active.rs`, enforced by a source-audit law.
What is *not* owned is the reading: 21 modules reach `self.active.buffer` and
`self.active.extra.<field>` directly, deliberately (the module doc argues a
whole-self borrow through an accessor would reintroduce the friction the slot
design avoids). A future slice should narrow `extra` field-by-field rather than
wrap `active`; `prev_file` (the last-buffer toggle target) has two writers and
belongs with the registry.

### `InputState`

`keymap` · `mods` · `prefix_pending_at` · `whichkey_shown` · `hud_key` ·
`hud_mods` · `peek_arm` · `peek_armed_at` · `pointer_hide` · `cursor_px` ·
`dragging` · `drag_press_px` · `drag_armed` · `page_resizing` ·
`page_resize_edge` · `page_resize_anchor` · `image_resizing` · `range_drag` ·
`cursor_icon` · `drag_granularity` · `last_click_time` · `last_click_px` ·
`click_count` · `scroll_px_accum` · `preedit` · `ime_enabled` ·
`scroll_sensitivity`

The largest domain by field count (27) and the **lowest-value** to extract:
locality is already near-perfect. 14 of the 27 are touched by exactly one file
(8 only by `app/input/mouse.rs`, 4 only by `keys.rs`, 2 only by `drags.rs`), and
`app/input/` accounts for 135 of the 164 production references. Extracting this
buys a struct rename, not an owned invariant. The one genuine cross-domain leak
is `cursor_px`, read by `apply.rs`
(`sync_overlay_after_core` re-arms the overlay's hover baseline from the
pointer's resting position) — a two-site coupling, better fixed by passing the
position than by moving 27 fields.

### `ProjectLocation` — "where am I working"

`root` · `project` · `file_index` · `workspace_root` · `recent_projects` ·
`recent_files` · `default_folder` · `cli_workspace` · `cli_default_folder`

The real derived-state domain: `project` and `file_index` are *functions of*
`root`, and `workspace_root` is a function of `(cli_workspace, config.workspace,
root)`. Today `App::set_root` re-derives `project` and calls
`rescan_file_index`, while `App::reload_config` re-derives `workspace_root` — so
the two re-derivation sites disagree about what `root` implies. **Confirmed
consequence:** `resolve_workspace` falls back to `root.parent()` when neither
the CLI flag nor `config.workspace` names one, so after a Switch-project into a
tree with a different parent, `workspace_root` still points at the OLD parent
until something calls `reload_config`. The Project picker (`C-x p`) browses
`workspace_root`, so it lists the previous workspace's siblings. Not fixed here
— the fix changes behavior, and this round is identity-preserving; it wants its
own queue item alongside the `ProjectLocation` slice, whose `set_root`
transition is the one obvious place for it.

`cli_default_folder` and `cli_workspace` have exactly one read each, both in
`reload_config` — the CLI override correctly wins over the reloaded config.

### `RenderRuntime` — held by item 174

`gpu` · `recovery_window` · `gpu_lifecycle` · `gpu_retry_at` ·
`gpu_timeout_streak` · `gpu_pending` · `present_sync_on` · `present_sync_valid` ·
`dpi` · `zoom` · `zoom_reflow` · `zoom_anchor` · `theme_font_at` ·
`theme_switch_at` · `theme_settle` · `theme_switches` · `caret_edit_streaks` ·
`caret_held` · `caret_impact` · `caret_recoil` · `frame_costs` · `debug_still` ·
`redraw_count` · `last_latency_ms` · `input_stamp`

`gpu` alone is 160 production references across 23 files — the single most
dispersed field in the struct — but nearly every one is the same shape:
`if let Some(gpu) = self.gpu.as_ref() { gpu.window.request_redraw() }`, i.e. a
*redraw request*, not GPU state. That is a `FrameScheduler` verb hiding inside a
`RenderRuntime` field. Item 174 owns the render-planning restructure; this
domain waits for it rather than racing it.

### `FrameScheduler`

`clock` · `last_frame` · `lava_tick_at` · `resize_settle_at` ·
`move_settle_at` · `crossing_settle_at` · `crossing_teardown_pending` ·
`focused` · `notice` · `notice_kind` · `notice_expires_at` · `zoom_persist_at`

Every debounce/settle deadline plus the notice's own expiry. `theme_font_at`,
`theme_switch_at` and `theme_settle` are the boundary cases: the *stamp* is
scheduling and the *effect* is rendering. The map assigns them to
`RenderRuntime` because their consumers (`retint_theme_preview`,
`retint_theme_now`) are render transitions; item 174's cut may move them, and
the classification gate is where that decision gets recorded.

`clock` is already owned in the way this item wants every field owned: one
`Box<dyn Clock>`, a grep law (`app/clock_law.rs`) fencing the module against a
raw `Instant::now()`, and a `VirtualClock` swap behind the same field. It is the
template — the difference between `clock` and `notice` is not the number of call
sites, it is that `clock` has a law and `notice` has a convention.

### Host / lifecycle — stays on `App`

`clipboard` · `clipboard_last_written` · `soak` · `soak_recovery_pending` ·
`soak_passed` · `probe_ready` · `daemon_socket_path` · `wait_conns` ·
`menu_proxy` · `_menu_bar` · `config` · `restored_window` · `pending_crash` ·
`stats`-group · `streaks`-group

These are process/OS handles and one-shot startup handoffs — `App` genuinely is
their lifecycle. `_menu_bar` exists only to not be dropped. `stats`/`streaks`
are already single-owner by locality (all 6 `stats_*` fields are touched by
`app/stats.rs` alone; all 3 `streaks_*` by `app/streaks.rs` alone) and gain
nothing from a wrapper. `config` is the exception worth a later slice: 87
production references over 16 files, and `reload_config` is the one writer.

## Where the item's premise was wrong

1. **`WorkspaceState` names two different domains.** Item 172 lists it beside
   `DocumentSession`/`PersistenceRuntime` and closes with "new *workspace* or
   *persistence* behavior has one obvious owner", which reads as the
   project-folder domain. Item 173 says "in item 172's `WorkspaceState`, define
   one closed lifecycle for editor, brief contextual overlay, sustained summoned
   workspace, and suspended child audition" — the *summoned-UI* domain. They are
   disjoint sets of fields. This round gives `WorkspaceState` to item 173's
   meaning (it is the critical path to items 114/116) and names the other domain
   `ProjectLocation`. The old `App::workspace` field is renamed
   `workspace_root` so `self.workspace_root` and `self.workspace_state` can never
   be misread for each other.
2. **"More than twenty modules can reach the whole live application state" is
   true but not the whole defect.** Reach is not the cost; *dispersion per
   invariant* is. `app/input/mouse.rs` reaches 40 fields and is fine, because 14
   of them are its own and the rest are read once. `overlay` is reached by 13
   files and is not fine, because five of them re-derive the same precedence
   rule. Extracting the 27-field `InputState` would satisfy the item's letter
   and buy nothing; extracting the 3-field `WorkspaceState` retires five copies
   of a rule.
3. **`RenderRuntime` is mostly not a render domain.** `gpu`'s 146 references
   are dominated by "request a redraw", which belongs to scheduling. A
   `RenderRuntime` extraction that moved `gpu` without first separating the
   redraw verb would drag 22 files into item 174's blast radius for no
   invariant.
