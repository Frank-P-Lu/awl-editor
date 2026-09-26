# Motion, timelines, and storyboards

Read for frame-loop, timeline, held-input, or multi-step capture.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

## The virtual-clock frame-loop capture (`--screenshot-frames`, Hidden)

```sh
cargo run -- --screenshot-frames 6 OUT.png [file]
# writes OUT.f000.png .. OUT.f005.png + one sidecar each, plus OUT.frames.json
```

A real headless `App` (`App::new_headless_scheduler`, hermetic, `gpu: None`)
drives its ACTUAL `about_to_wait_impl` scheduling body under a `VirtualClock`
stepped `--frame-step-ms` (default 100 ms) per frame, so a LIVE-ONLY
cross-frame behaviour — the which-key continuation panel summoning EXACTLY at
its 500 ms pause deadline — is inspectable from a deterministic sequence of
stills, the class a single settled `--screenshot` frame cannot show. The
document named on the command line is a STATIONARY backdrop the harness draws
itself (`capture::frames::capture_frames`, the same offscreen-pipeline shape
`--semantic-json` uses); the App renders nothing.

**`--capture-size`/`--capture-dpi` are honored.** Before, `Mode::ScreenshotFrames`
carried no canvas/dpi fields and fell through to the plain-`--screenshot` hook
bucket, so both flags parsed, validated as "honored", and were discarded
before the frame ever rendered — every invocation rendered the byte-stable
1200x800 default regardless of what was asked for. `capture_frames_async`
also never called `pipeline.set_dpi`, a second, independent instance of the
same shape one layer deeper: even a correctly-threaded dpi would have been a
no-op at the renderer. Both now follow the `--screenshot-app` path:
`Mode::ScreenshotFrames` carries `canvas`/`dpi`, threaded
onto the `CaptureOpts` the frame loop renders through, with the identical
meaning every other capture door gives them (a `WxH` physical canvas at
`--capture-dpi N` is the same logical `(W/N)x(H/N)` window) — proved by
matching visual-row wrap counts between `1200x800 @1` and `2400x1600 @2` on a
long-wrapping document, and by a narrower canvas producing genuinely more
reflow (`capture::tests::frames::
a_screenshot_frames_capture_honors_capture_size_and_the_dpi_meaning_holds`).

**Everything else this door drops is refused loudly, not silently.** The
document is loaded straight off disk with no `--keys` replay (the App's
buffer is a backdrop, not something chords mutate — `--keys` is refused
outright for this mode) and no project resolution at all, so the per-frame
render hooks (`--sel`/`--zoom`/`--scroll`/`--preedit`/`--search*`), `--root`,
`--workspace` and `--default-folder` all error "not honored by the chosen
capture mode" rather than parsing and doing nothing.


## Deterministic timeline capture (`--capture-timeline`)

A single frozen frame is great for *state*, but it can't show an animation's
**trajectory over time** — the caret glide, the trailing streak mid-glide, the
silhouette cross-fade brightening as it settles. `--capture-timeline` adds a
**deterministic timeline**: after a `--keys` replay sets up a NAVIGATION caret
move, it advances a VIRTUAL clock by a sequence of millisecond steps and writes a
frame at each step. The dt is **injected** (not a real clock), so the whole
sequence stays byte-deterministic — an agent can verify an animation's
*progression* (does the caret go origin → mid → settled, does the spring overshoot,
does the gap hold mid-glide) without a human eyeballing live motion.

```sh
cargo run -- --keys "C-e" --capture-timeline "0,16,50,150" OUT.png path/to/file.md
```

- The argument is a comma-separated list of **cumulative milliseconds since the
  move started**. The dt fed to step *i* is the delta `t[i]-t[i-1]`; the first
  entry `0` renders the pre-step frame (no advance).
- Each step writes `OUT.t<ms>.png` + `OUT.t<ms>.json` (suffix = the cumulative
  value), e.g. `OUT.t0.png`, `OUT.t16.png`, `OUT.t50.png`, `OUT.t150.png`.
- It composes with `--keys` / `--theme` / `--caret-mode` / `--root`. The `--keys`
  spec is **split**: every chord but the LAST sets up the origin; the LAST chord is
  the NAVIGATION move whose glide is captured. Use a **navigation** move (`C-e`,
  `M->`, `C-n`, …) — an EDIT move that crosses a row SNAPS (no glide), so it would
  show no trajectory.
- The caret spring is primed at the origin and started toward the destination, then
  `pipeline.advance(dt)` (the single virtual-clock seam shared with the live loop)
  steps the spring per entry. Because the real `step(dt)` runs, the trailing streak
  bridges correctly across fast glides — and it stays deterministic: **stepping the
  same sequence twice yields byte-identical PNGs + sidecars** (no real time, no RNG).

Per-step the sidecar gains a **`caret` block** (timeline frames are
`awl-capture/31`, held frames `awl-capture/32`) recording the spring snapshot so
the trajectory is machine-readable without eyeballing the PNG:

```json
"caret": { "t_ms": 50, "pos": { "x": 130.1, "y": 32 },
           "target": { "x": 164.0, "y": 32 }, "settle_factor": 0, "animating": true,
           "pop_scale": 1.0, "block": { "w": 9.6, "h": 32.0 },
           "trail": { "holding": true, "length": 28.0,
                      "tail": { "x": 110.0, "y": 32 }, "head": { "x": 138.0, "y": 32 } },
           "cosmetic_trail": { "present": true, "length": 28.0, "direction": "horizontal",
                               "held": true, "alpha": 1.0, "sweep": 1.0,
                               "tail": { "x": 110.0, "y": 32 }, "head": { "x": 138.0, "y": 32 } } }
```

- `t_ms` — the cumulative virtual-clock time this frame renders.
- `pos` — the ANIMATED caret pixel position (where it is drawn THIS step). Across a
  glide this progresses monotonically from the origin toward `target`.
- `target` — the true (settled) cursor pixel position the spring is gliding to.
- `settle_factor` — the [0,1] shape morph: ~0 mid-glide (caret collapsed to the
  trailing underline streak), → 1 as it arrives and re-forms the resting square.
- `animating` — `true` while the spring has not yet snapped to rest.
- `pop_scale` — the cosmetic squash-pop scale applied to the caret block on arrival.
- `block` — the drawn caret block size `{ w, h }` (h scales up over a big heading).
- `trail` — the drawn POSITION streak geometry (`holding`, `length`, `tail`, `head`).
  Present ONLY on the held path (`awl-capture/32`); the timeline path omits it.
- `cosmetic_trail` — the cosmetic | streak, on BOTH timeline and held frames:
  `present`, `length`, `direction`, `held`, `alpha`, `sweep`, `tail`, `head`.

So an agent asserts e.g. `pos.x` strictly increases t0→t150 and `settle_factor`
rises toward 1, proving the glide progressed origin → mid → settled. The plain
`--screenshot` path emits no `caret` block and stays schema `awl-capture/30`.


## Storyboards (`--storyboard`) — scenario runs with a film

A **storyboard** is a checked-in TOML file (`scenarios/*.toml`) that drives one
whole scenario end-to-end — typing, search, selection, pickers — through the
SAME strict replay session a `--strict-replay` capture uses, rendering as it
goes:

```sh
cargo run -- --storyboard scenarios/demo.toml            # outputs → scenarios/demo.run/
cargo run -- --storyboard scenarios/demo.toml --storyboard-out /tmp/run
```

Steps (exactly one key per `[[step]]`; parsing is STRICT — a typo'd key or a
garbled chord aborts at parse time, never silently skips):

- `press = "<chord spec>"` — replay chords through the real keymap (the
  `--keys` grammar; the open search panel consumes them first, exactly live).
- `type = "text"` — literal typing, each char the chord a spec would spell
  (`keyspec::text_chords`; whitespace via the named `Space`/`Enter`/`Tab`).
- `pause = ms` / `run_for = ms` — advance the VIRTUAL clock in fixed 20 ms
  frame steps (`capture::FRAME_MS`), one film frame per tick — this is where a
  caret glide or settle actually plays out on film.
- `expect` (a sub-table) — assert basic state: `cursor = [line, col]`,
  `overlay = "command"|"none"|…`, `search_active`, `search_query`,
  `selection`, `text_contains`. A failed expectation aborts the run.

Header keys: `name` (defaults to the file stem), `file` (the document to open,
relative to the storyboard's own directory; absent = scratch), `theme` (a
world name).

One run emits, into `--storyboard-out` (default `<board>.run/` beside the
TOML, gitignored):

- `step-NNN.png` + `step-NNN.json` — one frame + ordinary plain-schema sidecar
  per action step (an `expect` step renders nothing). The PNG is a byte-copy
  of the step's LAST film frame — a step artifact can never diverge from the
  film.
- `frames/frame-NNNNN.png` — every film frame, ALWAYS retained (the
  byte-deterministic deliverable).
- `trace.json` (`awl-trace/1`) — every chord's resolved action + effect
  classification (`applied` / `intercepted` / `unsupported`, plus the
  keymap-free `search_input` / `prefix` outcomes), every assertion outcome,
  and the `abort` record (if any) with the exact error text stderr shows.
- `film.webm` + `film.mp4` — encoded FROM the frames by a local `ffmpeg` when
  one is on PATH (`-bitexact`, 50 fps). No/broken ffmpeg only skips the encode
  (a note is printed; the frames remain) — never fails the run.

**Guarantees.** A storyboard run is STRICT (an unbound chord, a dangling
prefix, an Unsupported live-only effect, or a failed `expect` aborts, naming
the offender in both stderr and the partial trace — the film never silently
crosses a seam the headless driver does not implement) and HERMETIC (the
in-memory sandbox from `src/scenario.rs`, seeded with the storyboard's own
`file` + an explicit `--config`; the only real writes are the artifacts
above). Repeated runs of the same storyboard produce a **byte-identical
`trace.json` and frames** (`tests/storyboard_film.rs` pins this on the real
binary, along with the abort fixture `scenarios/abort-unsupported.toml`).
Each rendered step derives its project block from the replay session's current
root, so a Project root path pick through Settings changes later sidecars
without rewriting earlier ones (`scenarios/storyboard-project-fold.toml` pins
the hermetic two-root case). Files' Change folder row keeps its live-only native
chooser boundary and is not a strict-storyboard route.
Determinism is only claimed for trace + frames — the encoded films depend on
the local ffmpeg build. The film is deterministic VISUAL REVIEW of motion on
the virtual clock, NOT a claim about real compositor cadence (that stays a
live-only boundary, as above).

All bundled fixtures at once (the canonical command):

```sh
scripts/capture.sh           # builds release, renders every samples/*.md to gallery/
scripts/capture.sh --debug   # same, using the debug build
```

`scripts/capture.sh` writes, for each `samples/NAME.md`:

- `gallery/NAME.png`  — 1200×800 RGBA, one deterministic frame
- `gallery/NAME.json` — the render-state sidecar described below
