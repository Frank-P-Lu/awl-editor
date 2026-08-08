# docs/app-domains.md — the `App` ownership map

Read this before adding a field to `App`, before adding an `impl App` block in a
new module, and before moving state between subsystems. `ARCHITECTURE.md` is the
module map (which file holds what); this is the *state* map (which owner holds
which invariant). Queue item 172.

## The defect this map exists to close

The initial census found `pub struct App` with 107 fields; the extracted
owners have reduced the root to 20. Every root field is private to `crate::app`,
which means every one of the ~24 `impl App` blocks under `src/app/` can still
read and write it. The physical file split (item 56) made the code
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

The totals this census recorded are deliberately not restated here. They were a
snapshot of a moving tree, and the sums no longer reconciled with the per-owner
table's own columns — `app/tests/domains.rs` is the live ratchet (every root
`App` field classified to exactly one owner, the field count monotonically
down), and it is the only figure that means anything at any later date. Re-run
the grep above for a current number rather than trusting one written here.

## The owners

Names follow queue item 172, with two corrections the census forced (see
"Where the item's premise was wrong").

| Owner | Fields | Prod refs | Reach (prod files) | Status |
| --- | --- | --- | --- | --- |
| `WorkspaceState` (summoned UI) | 3 | 99 | 14 | extracted — slice 1 |
| `PersistenceRuntime` (save feedback) | 5 | 23 | 6 | extracted — slice 2 |
| `DocumentSession` | 4 | 363 | 21 | extracted — slice 5 |
| `InputRuntime` | 27 | 164 | 13 | extracted — slice 4 |
| `ConfigurationRuntime` | 4 | 87 | 16 | extracted — slice 3 |
| `ProjectLocation` | 6 | 51 | 9 | extracted — slice 3 |
| `FrameRuntime` | 38 | 400 | 23 | extracted |
| `UsageLedger` | 9 | 67 | 2 | extracted — slice 6 |
| host / lifecycle (stays on `App`) | 12 | 54 | 8 | stays |

The Fields and Prod refs columns are the census-time measurement and are not
maintained; what is maintained is the property — no field in two owners, none
unassigned — which `app/tests/domains.rs` asserts on every run.

The `UsageLedger` and host/lifecycle rows were re-measured at slice 6 with the
reproduce command above (`(self|app).<field>` over non-test files under
`src/app.rs` + `src/app/**`), which is stricter than the original pass — read
those two rows against each other, not against the older rows.

`src/app/tests/domains.rs` carries the same table as executable data with a
no-wildcard match over the owner roster, so a new `App` field fails the suite
until it is consciously assigned to an owner, and a new owner variant fails to
compile until it is described.

### `WorkspaceState` — the summoned-UI layer

⚠️ **THE FIELD LISTS UNDER EACH OWNER BELOW ARE THE NAMES EACH FIELD CARRIED ON
ROOT `App`, before extraction** — not the names its owning struct uses today.
`app/tests/domains.rs::retired_field_names` is that list's one owner and pins it
by name, so a re-added `overlay` fails the suite; the current struct field is
whatever its own definition says. Three have been renamed since:
`overlay` → `journey` (item 173's closed lifecycle, described below),
`popover_open` → `popover_summoned`, and `autosave_saved_version` →
`note_saved_version`. Reading a list here as a struct's field list will send you
looking for a field that is not there.

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
shape, see `FrameRuntime`).

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

Extracted as `app/document.rs::DocumentSession`. The active
`Entry<BufferExtra>`, background registry, last-buffer target, and shared spell
checker are private to that owner. Park/activate moves the whole entry; session
restore inserts through the same owner. Consumers receive immutable document
and cache projections plus named edit, cache, and persistence transitions. The
one mutable `Buffer` loan is fenced to `app/apply.rs`'s shared action core by a
source law. A full A→B→A→B→C→A law compares every `BufferExtra` member, including
the version-keyed text/spell caches and the pending autosave stamp.

### `InputRuntime`

`keymap` · `mods` · `prefix_pending_at` · `whichkey_shown` · `hud_key` ·
`hud_mods` · `peek_arm` · `peek_armed_at` · `pointer_hide` · `cursor_px` ·
`dragging` · `drag_press_px` · `drag_armed` · `page_resizing` ·
`page_resize_edge` · `page_resize_anchor` · `image_resizing` · `range_drag` ·
`cursor_icon` · `drag_granularity` · `last_click_time` · `last_click_px` ·
`click_count` · `scroll_px_accum` · `preedit` · `ime_enabled` ·
`scroll_sensitivity`

The largest domain by field count (27), but locality is near-perfect: 14 of the
27 are touched by exactly one file and `app/input/` accounts for 135 of the 164
production references. One `InputRuntime` handle owns two private coherent
substates. `KeyboardInput` owns key resolution, modifiers, prefix/which-key,
HUD/peek, and IME composition. `PointerInput` owns visibility and cursor shape,
press→drag→release state, resize gestures, click cadence, and wheel sensitivity
and accumulation. Only `app/input/` children may project those substates;
scheduler, settings, view, headless-press, and window-lifecycle consumers use
named observations or transitions on `InputRuntime`.

The only genuine cross-domain leak was `cursor_px`, formerly read by `apply.rs`;
`sync_overlay_after_core` now accepts a typed `RestingPointer` value snapshot,
so overlay resync cannot retain or mutate the live pointer state. Text-drag
release and the next press baseline are likewise one pointer transition: release
retires both `dragging` and the sticky slop arm, and a new press snapshots its
own position before it can arm.

### `ConfigurationRuntime` — persisted settings and startup policy

`config` · `default_folder` · `cli_workspace` · `cli_default_folder`

The runtime configuration has a different responsibility from the current
project: it owns user settings, explicit CLI precedence, and the first-run
default-folder fallback. Its typed `LocationPolicy` is the only configuration
fact the live project location receives. A config reload returns that policy as
a `ReloadOutcome`, so keymap/settings application cannot quietly leave the
project location on its prior workspace rule.

### `ProjectLocation` — "where am I working"

`root` · `project` · `file_index` · `workspace_root` · `recent_projects` ·
`recent_files`

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

The CLI inputs and `default_folder` live in `ConfigurationRuntime`, not in this
owner: first-run policy must never be mistaken for the running project's root.

### `FrameRuntime`

`gpu` · `recovery_window` · `gpu_lifecycle` · `gpu_retry_at` ·
`gpu_timeout_streak` · `gpu_pending` · `present_sync_on` · `present_sync_valid` ·
`dpi` · `zoom` · `zoom_reflow` · `zoom_anchor` · `theme_font_at` ·
`theme_font_last_reshape_at` · `theme_switch_at` · `theme_settle` ·
`theme_switches` · `caret_edit_streaks` ·
`caret_held` · `caret_impact` · `caret_recoil` · `frame_costs` · `debug_still` ·
`redraw_count` · `last_latency_ms` · `input_stamp`

`clock` · `last_frame` · `lava_tick_at` · `resize_settle_at` ·
`move_settle_at` · `crossing_settle_at` · `crossing_teardown_pending` ·
`focused` · `notice` · `notice_kind` · `notice_expires_at` · `zoom_persist_at`

One `app::frame::FrameRuntime` owns the GPU/surface lifecycle, presentation
bookkeeping, render-affecting feedback, deadlines, the injected clock, and the
notice lifetime. These are one lifecycle: input arms work, the idle poll settles
it, and a presented frame retires it. Theme and crossing stamps remain beside
the effects they schedule.

`frame/poll.rs` accepts copyable input, document, and configuration scheduling
snapshots and returns a fixed `PollOutcome`: redraw, reshape, persist zoom,
expire notice, retry, and the next deadline. It is not a message bus.
`frame/surface.rs` privately owns recovery, timeout, retry, and present-sync
state. GPU replacement invalidates the present-sync shadow before equality may
elide the platform write. `frame/presentation.rs` holds the private presentation
ledger.

The root exposes one `frame` handle. The API-width law in
`app/tests/domains.rs` prevents one-field accessors from recreating the old
field bag; lifecycle changes use named transitions and read-only facts cross as
typed snapshots. Raw GPU loans remain for render-pipeline consumers until the
planner/execution split is complete.

### `UsageLedger` — the two private local-usage records

`stats` · `stats_origin` · `stats_last_input_ms` · `stats_last_caret_xy` ·
`stats_last_cursor` · `stats_dirty` · `streaks` · `streaks_baseline` ·
`streaks_dirty`

Extracted as `app/usage.rs::UsageLedger`. **This map previously argued these
nine fields were already single-owner by locality and would gain nothing from a
wrapper.** The locality claim is true and still is — all six `stats_*` fields
are touched only by `app/stats.rs` and all three `streaks_*` only by
`app/streaks.rs`, 67 production references over exactly those two files — but
locality was the wrong measurement. This item's defect is
*invariants coupled by convention*, and three of them crossed the two modules:

- **One privacy kill switch, eight readers.** Both records hang off the single
  `stats` config toggle. It was re-read at eight sites across the two files, so
  a new tracking hook that forgot its `if` would record private usage with the
  toggle off — reachable by one missing line, catchable only by review. The
  toggle now has exactly ONE reader anywhere under `src/app/`
  (`ConfigurationRuntime::usage_recording`), which hands every transition a
  typed `Recording` value, and `the_usage_privacy_gate_has_exactly_one_reader`
  counts it.
- **A record and its unflushed-changes stamp, paired by hand five times.** The
  private `usage::dirtying` module makes the pairing structural: outside it
  neither store is reachable as `&mut` and neither flag is writable at all, so a
  recording path CANNOT forget its stamp. Its one door takes the store's own
  `Changed` verdict, so a deduped re-open still writes nothing on a quiet quit.
  `the_usage_records_have_exactly_one_dirty_stamping_site` fences the inside of
  that module, which the type system cannot.
- **One flush cadence and one buffer-swap anchor rule**, spelled out at ten
  flush call sites across six files and five anchor sites across three.

**The asymmetry, recorded rather than "fixed".** Three sites flush the streaks
record ALONE (`load_path`, `start_fresh_document`, the card-summon effect). That
is correct, not drift: the streaks delta is BUFFER-SCOPED — words now minus
words at the last sample of *this* document — so it must be sampled before the
active buffer changes, while the odometer's counters are app-global and carry no
such deadline. Two named transitions, not one merged flush.

### Host / lifecycle — stays on `App`

`clipboard` · `clipboard_last_written` · `soak` · `soak_recovery_pending` ·
`soak_passed` · `probe_ready` · `daemon_socket_path` · `wait_conns` ·
`menu_proxy` · `_menu_bar` · `restored_window` · `pending_crash`

These are process/OS handles and one-shot startup handoffs — `App` genuinely is
their lifecycle. `_menu_bar` exists only to not be dropped. The `stats`/
`streaks` groups left this bucket in slice 6; see `UsageLedger` above for why
the locality argument that kept them here was measuring the wrong thing.
Configuration is no longer host lifecycle state:
`ConfigurationRuntime` owns it with the startup location policy that gives its
`workspace` and `default_folder` settings meaning.

## The other half: REACH

Fields are one axis. The other is how much of the live application any given
piece of code *can* read, which the field extractions did not change at all:
44 `impl App` blocks across 38 files, before and after, every one of them able
to touch all 20 root fields because they are private to `crate::app` and to
nothing smaller.

### The census that has to come first

A block is the wrong unit — it is a container, not a caller, and a block
holding twenty single-domain methods is not god-object reach. Measured per
METHOD (`self.<handle>` over non-test files under `src/app.rs` + `src/app/**`,
one row per `fn` inside an `impl App`):

| Domains one method touches | Methods | Share |
| --- | --- | --- |
| 7 | 1 | 0.3% |
| 6 | 1 | 0.3% |
| 5 | 1 | 0.3% |
| 4 | 12 | 4% |
| 3 | 37 | 12% |
| 2 | 71 | 24% |
| 1 | 127 | 42% |
| 0 | 51 | 17% |

301 methods; **15 of them (5%) touch four or more domains**, and the two worst
are the coordinators that are supposed to — `sync_view` (7) is the one
exhaustive `ViewState` construction site, `run_action_core` (6) is the shared
action core. Two thirds of all methods touch one domain or none.

So the shape of the remaining defect is not "coordinators reach too far". It is
that a method touching ONE domain still has the same reach as `sync_view`,
because `&mut App` is the only handle anyone is offered. The cut that pays is
**narrowing the handle a subsystem receives**, not moving another field.

### Which subsystem, by domain closure

Grouping those methods by subsystem and taking the union of domains each group
touches — its *closure*, the narrowest handle that could serve it:

| Subsystem | Methods | Closure |
| --- | --- | --- |
| `app/files/` | 86 | 8 (all of them) |
| `app/input/` | 58 | 5 |
| `app/apply*` | 32 | 7 |
| `app/semantic/` | 32 | **3** |
| `app/window.rs` | 19 | 5 |
| `app/stats.rs` | 11 | 5 |
| `app/schedule.rs` | 9 | 5 |
| `app/viewstate*` | 8 | 7 |

`app/semantic/` is the outlier: the largest subsystem whose closure is narrow
enough that a typed handle is a boundary rather than a rename.

### `SemanticView` — the fold's narrow borrow

Building the accessibility tree is a READ over the document and the summoned-UI
ladder. It took `&App`, so it could equally read the clipboard, the daemon
socket, the usage ledger and the GPU.

`App::semantic_view` is now the one place the whole `App` is read on the fold
side. It hands down a `SemanticView` holding two borrows — `DocumentSession`
and `WorkspaceState` — and three VALUE snapshots the frame already had to
compute: the summoned card as finished `CardContent`, the which-key rows, and
the notice text. Values rather than borrows for the same reason
`InputRuntime`'s `RestingPointer` is one: a fold that held a live handle could
retain it, and the narrowing would be a habit rather than a boundary.

`surfaces.rs`, `passive.rs` and the retained `projection.rs` — the whole
tree-building path — can no longer NAME `App`. `mod.rs` keeps the live doors
and the narrowing constructor; `requests.rs` keeps the `App` because every arm
of it genuinely ends in another owner's verb (`App::apply`, `sync_view`,
`request_frame`), which is mutation, not fold.

Two things have to stay true for that to be a boundary, and Rust expresses
neither — `SemanticView` and `App` are both nameable from anywhere under
`crate::app`. `semantic_fold_reads_only_the_narrow_view` in
`app/tests/semantic_reach.rs` asserts both, with a wildcard-free roster swept
against the directory: a fold file may not name `App` as an identifier, and the
view's field roster is pinned so it cannot grow a handle to a fourth domain
without a conscious edit.

The block count barely moves and is the wrong headline: two `impl App` blocks
go (`surfaces.rs`, `passive.rs`) and the narrowing constructor adds one back in
`view.rs`, so 44 becomes 43. The load-bearing number is the file one. Counting
the five fold/request files and setting aside `bench.rs` (a `--bench-a11y`
harness that builds its own `App` to measure against), **the files that can
reach root `App` state fall from 5 to 3** — `mod.rs`, `requests.rs`, and the
63-line `view.rs` — and the 789 lines of fold beneath them can now read only
what the view names.

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
   rule. The initial map correctly prioritized `WorkspaceState`; the later
   `InputRuntime` extraction is deliberately a locality-preserving handle plus
   the typed pointer snapshot, not a second interaction system.
3. **Locality is not ownership.** This map's own first pass used file locality
   to argue the odometer and streaks fields needed no owner (see `UsageLedger`).
   Locality answers "who touches this field"; the item's defect is "who holds
   this rule", and the privacy gate — one toggle, seven readers, two modules —
   was dispersed by exactly the measure the map was not taking. A field group
   that lives in one file can still hold a rule that does not.
4. **`app/files/` was the wrong first reach cut.** The `UsageLedger` slice
   closed by naming it: ten `impl App` blocks that are really
   `DocumentSession` + `PersistenceRuntime` + `ProjectLocation` collaboration,
   convertible into one `FileOps` owner. Measured, that triple covers **30 of
   the 71** domain-touching methods under `app/files/`; the other 41 need a
   fourth domain, and `ConfigurationRuntime` (21 methods) is reached there more
   often than `PersistenceRuntime` (18) or `ProjectLocation` (14). A `FileOps`
   wide enough to serve the subsystem needs seven of the eight handles, which
   is `&mut App` with a different name — `app/files/` has the WIDEST closure of
   any subsystem, not a narrow one. It is a real decomposition target on the
   dispersion axis this map already measures; it is not a reach target.
5. **Render and scheduling are one frame lifecycle.** Theme/crossing stamps,
   present-sync claims, GPU recovery, and redraw retirement cross the proposed
   render/scheduler line. `FrameRuntime` therefore owns the lifecycle as one
   domain; render planning remains separately owned by the planner work.
