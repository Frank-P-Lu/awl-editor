# ARCHITECTURE.md — awl's module map

How the pieces fit. For the identity and product boundary see `PHILOSOPHY.md`;
for the feel see `DESIGN.md`; for how to verify headlessly see `CAPTURE.md`.
This doc is the wiring.

awl is a single Rust binary (crate `awl`): Rust + wgpu (2D only) + winit.
mac = Metal, linux = Vulkan. It is a keyboard-native prose-writing instrument
with native and selectable keymaps (see `PHILOSOPHY.md` §4).

## The input → action → apply spine

The one path everything flows through:

```
key event ──▶ keymap ──▶ Action ──▶ apply_transition ──▶ state + typed Effects
 (winit       keymap.rs            actions.rs             │
  or --keys)                                             ├─ live interpreter
                                                        └─ headless interpreter
```

- A keystroke (a live winit event, or a chord from `--keys`) resolves to a single
  `Action`.
- `Action` is the editor's command vocabulary — motions, edits, region ops, view
  ops, file ops. It is the seam between "what key was pressed" and "what the
  editor does."
- `apply_transition` is the GPU-/winit-/clipboard-/filesystem-free seam. It
  mutates editor state and returns a closed, ordered `Transition` of typed
  requests for persistence, clipboard, buffer, daemon, notice, and render work.
  Live and headless callers interpret that same transition explicitly.

## Modules

Several of the larger modules are **directory modules**: the original `foo.rs`
stays as the module root (it keeps `mod foo;` in `main.rs`) and declares
`mod <topic>;` submodules that live in a sibling `foo/` directory — the same
precedent that split `render.rs` into `render/{caret,chrome,geometry,…}.rs`.
The split is a pure file-relocation (items lifted verbatim, visibility widened to
`pub(crate)`/`pub(super)` only where a sibling needs them, re-exported by bare
name); behavior is byte-identical. Submodules are listed under each root below.

**Entry / control**
- `main.rs` — entry point + CLI. Parses `Mode` (interactive window vs. headless
  `--screenshot` / `--screenshot-motion[-v]`, with optional `--keys`). For
  headless modes it loads the buffer, `replay_keys`, then hands off to capture.
  → `main/`: `args` (CLI / `Mode` parsing + folder resolution), `run` (the
  interactive + headless run paths). `run/` splits the replay session by
  responsibility: `chord` (search-first resolution + the depth-first action
  worklist), `effect_interpreter` (ordered non-typed effects), `buffers`
  (whole-buffer park/switch), `trace` (classification + trace/warning/skip
  bookkeeping), `settings_effects`, `location`, and `capture_fold`.
- `app.rs` — the winit `ApplicationHandler`: window + event loop, composes the
  owned runtime handles, mouse handling, and the live transition interpreter (persistence,
  clipboard mirroring, GPU-measured page sizing, animation/redraw scheduling).
  → `app/`: `gpu` (device/surface setup), `files` (open/save/project glue),
  `document` (`DocumentSession` — the active whole slot, background registry,
  previous-buffer target, spell checker, and private `BufferExtra` caches),
  `viewstate` (view sync + paging), `input` (mouse/key event handling), `apply`
  (the `App::apply` wrapper around `apply_transition` + live effects), `daemon`
  (the App-side half of the single-instance daemon below), `workspace`
  (`WorkspaceState` — the summoned-UI layer LADDER: overlay > workspace > search
  > popover > editor, with private fields and named transitions; the fourth rung
  reads `overlay::Journey`), `persistence`
  (`PersistenceRuntime` — the app-global save ledger: the fresh-document
  autosave debounce+version pair, the save-feedback clocks, the title dirty
  cache), `frame` (`FrameRuntime` — private GPU/surface lifecycle,
  presentation ledger, injected-clock deadlines, notice lifetime, and the
  typed idle-poll boundary), `usage` (`UsageLedger` — the two private
  local-usage records: the lifetime odometer + silent command ledger and the
  writing-streaks day map, with their sampling anchors, one privacy gate, and
  a `dirtying` submodule that makes a record and its unflushed-changes stamp
  one value; `stats.rs`/`streaks.rs` keep only the App-side wiring).
  `App` is being decomposed into owned state domains (queue item 172): read
  `docs/app-domains.md` — the ownership map — before adding a field to `App` or
  an `impl App` block in a new module. Its single `InputRuntime` handle contains
  private `KeyboardInput` and `PointerInput` substates; only `app/input/`
  projects them, while sibling domains use named transitions and typed snapshots.
  `app/tests/domains.rs` is the gate: every
  root `App` field is classified to exactly one owner, and the field count is a
  ratchet that may only go down.
- `daemon.rs` — the SINGLE-INSTANCE DAEMON (native only, and not in the
  sandboxed store build:
  `cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))`): a Unix domain socket beside the scratch
  stash (`fs::data_root().join("awl.sock")`). Owns the bind-or-handoff startup
  dance (`startup`/`bind_or_connect` — the stale-socket truth table), the
  dumb newline-delimited wire protocol (`format_open`/`parse_open`/
  `format_done`), and the accept-loop thread (`spawn_accept_thread`) that
  posts a `DaemonEvent` into the live winit event loop via
  `EventLoopProxy::send_event`. `app/daemon.rs` reacts to that event
  (`App::handle_daemon_event` → `load_path` + raise the window), and owns
  `Action::FinishBuffer` (the catalog's "Finish file", default Cmd-W) — save,
  notify any daemon `--wait` client, switch to the previous buffer. Lives
  ONLY on the live App's startup path (`app::run`), never on any headless
  `--screenshot`/`--bench-*` mode — see `daemon.rs`'s module doc for the full
  capture-gate argument and CLAUDE.md's Daemon section for the doors.

**Editor core (renderer-agnostic logic)**
- `actions.rs` — `ActionCtx` + `apply_transition`: the sole shared
  state-transition seam and closed typed-effect vocabulary (above). Every
  caller receives the complete ordered transition; focused tests select its
  primary effect explicitly.
  → `actions/`: `edit` (markdown smart-Enter), `flinch` (caret-feedback
  triggers), `motion` (oracle-aware motions + page scroll + search open),
  `overlay_nav` (modal overlay intercept + browse-path + live preview), `rebind`
  (the game-style rebind-menu key handling).
- `keymap.rs` — the stable input façade: `KeymapState::resolve(key, mods) → Action`;
  table-driven, including the `C-x` prefix.
  → `keymap/`: `action` (the `Action` vocabulary + exhaustive `is_motion` /
  `is_edit` classifications), `state` (prefix/override maps + catalog seeding),
  `resolve` (the single stateful resolver), `platform` (Linux collision/keep
  policy + keymap flavors), `binding` (chord parsing and key canonicalization).
- `keyspec.rs` — `parse_keys("C-n M-> …") → Vec<Action>`: parses emacs key-spec
  strings by driving the *real* keymap. The headless analog of typing; powers
  `--keys`.
- `fs.rs` — the one `FileSystem` trait and public filesystem façade.
  → `fs/`: `native` (real disk), `memory` (tests + strict replay sandbox),
  `web` (localStorage + first-load seeds), `paths` (atomic write + shared
  path derivation), `active` (the swappable backend + guarded CWD globals).
- `buffer.rs` — the document: a ropey rope, edit ops, cursor, undo/redo grouping,
  mark/anchor primitives.
  → `buffer/`: `edit`, `selection`, `motion`, `undo`, `focus`, `notes`, `tests`.
- `buffers.rs` — the MULTI-BUFFER REGISTRY: `BufferKey` (a buffer's stable
  identity — a path, or the one `Scratch` sentinel) + `BufferRegistry<T>` (the
  MRU-ordered, capped park/take store for every BACKGROUNDED buffer) +
  `Entry<T>` (a buffer plus its opaque per-buffer payload — the SAME type the
  live App's `DocumentSession` owned slot uses), shared verbatim by the live
  `App` (`app/document.rs`'s private `BufferExtra` payload) and the headless `--keys`
  replay (`main/run.rs`'s `replay_keys`, payload `()`) — one owner of "open a
  file that's already open switches to its live buffer," never two aligned
  copies. `DocumentSession` owns the live ACTIVE half and performs a whole-slot
  `mem::replace`/assignment on park/activate, never a field-by-field
  snapshot/restore; the replay's `buffer` local is its own, unchanged.
- `selection.rs` — the selection / region model (C-Space mark, kill/copy, drag).
- `rotated_label/` — THE ROTATED LABEL: the one way awl can draw text at an
  axis other than upright. glyphon 0.11 has no transform (`TextArea` is
  left/top/scale/bounds/colour, `CustomGlyph` is left/top/width/height), so a
  world that wants a turned or slanted cue cannot express it through the
  document layer at all. One short shaped run is composed on the CPU into a
  single R8 coverage image (`mask.rs`, the same swash cache glyphon rasterises
  from) and painted through one quad rotated onto a unit axis — the axis
  rotation `shaders/caret.wgsl` already performs, applied to a glyph mask. Its
  frame (`geometry.rs`) is pure: axis, quad, tight bounds and a hit test in the
  run's own coordinates. It reads no theme, and it composes exactly ONE layout
  run, so it cannot grow into a second prose renderer; the document layer stays
  the one. See docs/render.md.
- `range.rs` — the RANGE SPEC owner (item 94): one typed description of a bounded,
  stepped setting (`min`/`max`/`step`/`default`, a display unit, and a linear or
  logarithmic rail mapping) plus every derivation from it — quantization, the step
  grid, both directions of the rail mapping, keyboard stepping, the readout, the
  exact-entry parse, and the persisted RHS. `settings::range_spec` maps a settings
  row to its spec; `render::clamp_zoom` delegates here. Keyboard, pointer, render,
  sidecar and persistence all route through it, so no input path computes a
  parallel value. See docs/render.md.
- `overlay/journey/` — THE SUMMONED-UI LIFECYCLE (`Journey`): one closed state
  set (`Surface × Beneath`), one closed event set, and one wildcard-free table
  (`journey/table.rs`) saying where every Esc/Back/accept lands. Owns the
  suspend/return of a sustained workspace into a child audition — the parked
  parent's exact return position (`journey/parked.rs`: `Parked`/`Resume`), the
  child's typed write-back (`Bind`), and the revert payload a cancel undoes
  (`Audition`). Lives in the shared core, not in `crate::app`, so the headless
  `--keys` replay reaches the identical transitions; `app/workspace` owns the
  one live instance and derives its ladder rung from it. Scoped to Settings and
  Version History — it is not a route stack. See `docs/app-domains.md`.
- `search/` — incremental search (isearch) state + match finding.
  → `keys` (the chord layer over an active search), `semantic`.
- `spell.rs` / `spellunderline.rs` — spellcheck (spellbook) + underline data.

**Rendering / presentation**
- `render.rs` — all wgpu drawing: glyph atlas + shaping (glyphon), buffer text,
  the caret block, selection highlights, spell underlines, and the isearch panel
  card. The big file (still the largest in the tree).
  → `render/`: `plan` (the deterministic, device-free SCENE PLANNER — drawing,
    hit-testing and the sidecar read its planned objects instead of each deriving
    geometry of their own; see docs/render.md), `caret`, `chrome` (status strip /
    HUD card / readout), `geometry`,
  `rowgeom` (per-row geometry table for variable heading heights), `spans`
  (md/CJK/syntax/focus `AttrsList` layering), `text`, `focus`, `rects`, `layers`,
  `facepitch` (is a bundled family monospaced? — measured from each face's own
  advance widths; the caret's mono/proportional fork reads it).
- `caret.rs` — caret position + its springy motion/glide animation (the "streak"
  / motion work).
  → `caret/`: `spring`, `morph`, `juice`, `preview`, `pipeline`, `tests`.
- `theme/` — the world rosters and every palette token derived from them
  (BASE_* greys, the single amber accent).
  → `worlds` (the `THEMES` roster), `model` (`Theme`/`RenderCaps` and their
    variant types), `color`, `derive`, `ground`/`ground_space`/`icon_ground`,
    `cjk`, `diagonal`, `ornament`.

**Verification**
- `capture.rs` — headless one-frame capture: render to an offscreen texture, read
  back pixels → PNG + JSON sidecar (`awl-capture/2`). The agent-facing contract;
  see `CAPTURE.md`.
  → `capture/`: `opts` (capture options), `modes` (the capture entry paths),
  `gpu` (offscreen device/readback), `animated` (motion-frame capture), `oracle`,
  `sidecar` (the JSON sidecar emitter), `tests`.
- `bench.rs` — microbenchmarks.

## Two flows, one engine

1. **Live:** winit event → `app.rs` → `keymap::resolve` → `Action` →
   `actions::apply_transition` → the live effect interpreter → `render.rs`.
2. **Headless verify:** `--keys "spec"` → `keyspec::parse_keys` → `Vec<Action>` →
   `replay_keys` / `apply_transition` (same seam) → the headless effect
   interpreter → `capture.rs` renders one
   deterministic frame → PNG + sidecar.

Because both flows share `keymap` + `apply_transition`, a headless capture
exercises the real edit logic rather than a mock. Search-panel chords also use
one renderer-independent interception seam in both flows.

Ordinary headless replay owns no filesystem-write capability: Save and Finish
are recorded/skipped without touching disk, opening an absent config does not
create it, clipboard/daemon handoffs are intercepted, and render requests
settle without a window. Strict scenarios and storyboards explicitly receive
an isolated in-memory filesystem capability, so their typed Save requests can
land in the sandbox. GPU-measured paging remains live-only; oracle-less
headless tests use their documented fixed page size.
