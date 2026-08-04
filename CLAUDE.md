# CLAUDE.md / AGENTS.md — working on awl-editor

> Lean core (halved 2026-07-22, queue item 17). Per-feature mechanism detail lives in `docs/` — each file names when to read it; read the matching doc before working in that area. Round-by-round history: `git log -p CLAUDE.md`. AGENTS.md is a symlink to this file; edit here.

awl is a calm, opinionated plain-text editor for prose and light code — Rust + wgpu + winit + glyphon. One core, two builds: native desktop (macOS Metal, Linux Vulkan) and browser (wasm32, WebGPU with a WebGL2 fallback). Native macOS ⌘ bindings are the advertised keymap, quietly doubled with Emacs/`mg` — both slots fire. A personal tool with a widened audience: people who aren't programmers but like computers, writing, novelty, and beauty.

Start with `PHILOSOPHY.md` — the identity, priorities, and product boundary. The other contract docs: DESIGN.md (the feel), THEMES.md (world laws), CAPTURE.md (headless verification), ARCHITECTURE.md (module map), WEB.md (browser build), RELEASING.md, ACCESSIBILITY.md.

## Direction

awl is a WYSIWYG editor on the Obsidian Live-Preview model: the file stays plain text, only the render becomes rich, and any line shows raw markdown while the caret is on it. The committed work is finishing that model — images inline, tables as real grids — through the markdown formatting commands. It is not a Word clone (no styled clipboard or general formatting toolbar) and carries no IDE machinery (LSP, multi-cursor, project tree). Contract: PHILOSOPHY.md §§1, 5.

## Build & test

```sh
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo build && scripts/native-gate.sh  # from the repo root; emits full-suite receipt
```

Incremental builds only (a clean rebuild is slow — no `cargo clean`). Edit in place, matching the file's own style. Judge feel in `--release`: dev frames are 10–20× slower, so perf claims are only honest there.

To RUN it on macOS use `scripts/dev-app.sh`, not `cargo run`: a bare binary has no bundle, so the menu bar reads `awl` and Stage Manager draws the generic tile — two surfaces macOS will not honor without one (docs/platform.md).

## Verify headlessly

```sh
cargo run -- --screenshot OUT.png [file]    # writes OUT.png + OUT.json sidecar
```

Flags compose: `--keys` (chord replay through the real keymap), `--theme <World>`, `--caret-mode`, `--measure`, `--screenshot-motion[-v|-d]`, `--screenshot-app` (the same chords into a real headless `App`), `--semantic-json` (that App's accessibility tree, no PNG), `--root/--workspace/--default-folder`, `--config`, `--debug`, `--hud`. Schema + semantics: CAPTURE.md. The schema number is one const, `capture::SCHEMA_VERSION`.

The sidecar is the source of truth for state; the PNG for geometry and appearance. The harness verifies state, geometry, colors, and deterministic single-frame trajectories; it cannot verify timing, feel over real time, or taste — flag those for live human confirmation rather than claiming them verified.

An ordinary capture drives the shared core, not the live `App` — so a transition the `App` owns (settings writes, buffer switching, config reload) is classified Unsupported and skipped. `--screenshot-app` (item 188) drives the same `--keys` chords into a REAL headless `App` and writes the same schema from its state, stamped `driver: "live-app"`; it is hermetic and skips nothing. `docs/harness-reach.md` maps the edge, per effect and per picker; check it before a brief promises a capture over live-`App` state (item 180 asked for one that could not exist).

Tripwire: the sidecar is a state oracle, not an appearance oracle — it once reported `selected_index: 2` while the row rendered fully invisible (Wagtail). Appearance claims (visible, distinct, legible) are asserted by arithmetic over the PNG's pixels.

## Spot-check audits (standing policy)

Audit agents run at the production tier: Sonnet medium on Claude or `gpt-5.6-terra` at `medium` on OpenAI. Probe form: enumerate state × surface × world, sampled along the changed axis, asserting the outcome per cell with pixel/sidecar arithmetic. Triggers: a new axis value (probe the full surface roster); an identity-gated refactor (follow with an outcome audit — byte-identity preserves pre-existing bugs); a user-reported bug (audit its neighborhood; bugs cluster); a degradation arm ships (probe the degraded state; its compensating mechanism gets named and law-tested); pre-tag (a journey sweep across worlds). Every render-touching round gets a vision-smoke: affordance-locating questions over ~5 gallery shots ("which row is selected?"), never "does this look fine?". An audit that finds something ends by writing the missing law test.

## Feature docs — read before working in the area

- `docs/config.md` — config.toml, `[keys]` rebinding, keymap flavors, Linux keep-list, retired defaults, page width.
- `docs/markdown.md` — span styling, heading ladder + variable row heights, conceal mechanics, formatting commands, Insert-link.
- `docs/fonts.md` — display faces, per-world mono, CJK ladder + never-tofu law, frontmatter lang, theme-preview debounce.
- `docs/syntax.md` — the Alabaster four-role philosophy, two-tier comments, `role_style_for`, spell scoping in code.
- `docs/render.md` — adaptive column, RenderCaps (themes as data), overlay/chrome personality, settings-in-palette, rowlayout.
- `docs/platform.md` — debug panel/HUD/copy pulse, autosave + history, line endings, daemon/EDITOR=awl, menu bar, session restore, updates, GPU faults + `--soak-gpu`.
- `docs/licensing.md` — GPL-3.0-only, asset licenses, generated third-party list, audit cadence + standing advisories.
- `docs/harness-reach.md` — how far the headless harness reaches: the three verification tiers, the live-only census, the per-effect capture table. **Read before writing a Verify clause that asks for a capture.**

## Tripwires (hard-won facts; mechanisms in the docs)

- wgpu macOS occlusion gate: a window without `NSWindowOcclusionStateVisible` returns Occluded before `nextDrawable()` → `acquires=0 presents=0`. It looks like a GPU bug; it is OS occlusion state. Soak windows must stay foregrounded. (platform)
- IBM Plex Mono ships as Weight 300; a default-400 request drops it and mono worlds fall through to proportional `.SF NS`. `mono_safe_weight()` compensates. (fonts)
- `C-k` stays kill-line on Linux in both flavors, so Insert-link has no default Linux binding; `C-c`/`C-x`/`C-v` stay native (Omarchy forwards Super+C/X/V as Ctrl). (config)
- Conceal reveal changes glyph advances, not just color: `refresh_rule_conceal` invalidates `row_geom` alongside its reshape. (markdown)
- Menu Quit/Edit items are routed through `App::apply`, never muda-predefined (predefined Quit bypasses teardown; predefined Edit selectors silently no-op on a wgpu NSView), and the returned `muda::Menu` stays alive for the app's lifetime — dropping it makes menu clicks use-after-free. (platform)
- AppKit objects free themselves out from under Rust ownership in two shapes, not one. A programmatically created `NSWindow` has `releasedWhenClosed` **YES** by default, so its close button frees the window while a `Retained` handle still points at it and the next summon is a use-after-free; set it NO and let one owner hold it (`mac_about`: a main-thread `thread_local` for the process lifetime). `NSButton`'s target is unretained — target a view's own window, which outlives it, rather than an object that may not. Same failure as the dropped `muda::Menu` above, different object. (platform)
- Capture gates: daemon, autosave/history, and session restore exist only on the live App; the headless path is structurally free of them. Git-managed files record no history snapshot — their timeline is `git log` alone. (platform)
- Cache-key discipline: anything keyed by `buffer.version()` also keys by buffer identity or clears on swap — versions restart at 0 per open, and the collision has already served a stale document once.
- Test-global locking: one process-wide reentrant guard, `crate::testlock::serial()`, taken by every test and every `cfg(test)` global writer — **and every READER of a swappable global** (item 101: an unguarded reader against a disciplined writer is the same race from the other end). Enforced, not conventional: `capture::sidecar::write_sidecar`, `fs::active`/`set_active`/`current_dir` assert the hold under `cfg(test)`, so an unguarded caller panics by name on its first run instead of flaking someone else's merge. The guard also RESTORES what it snapshots — world, page, spellcheck, and every forced render override — on the unwinding path too, so a fixture that sets a global and dies cannot poison a different test in a different file (item 233: a leaked `ListStyle::Bars` made an unrelated jump-hint law report a clip that was not one, green single-threaded and red under a wide `--test-threads`). Three rounds of ABBA deadlocks retired the ordered per-module locks; don't reintroduce them. (`config::ENV_LOCK` separately serializes env mutation.)
- Zero-network is a design invariant: awl never phones home or fetches at runtime; a future language pack is a file. License facts are never fabricated — the unverifiable gets flagged. (licensing)

## Engineering principles

- Same behavior ⇒ same code — merge, don't align. Extract one owner of the rule, route every consumer through it, make the bypass module-private, and add a law test with a no-wildcard match so a new member can't dodge the sweep.
- ~500 lines is a file's natural ceiling; past it, decompose into a submodule dir. Exceptions are declared (render.rs's GPU floor).
- Untested behavior doesn't exist. Test at the purest reachable seam (unit > sidecar > capture); live-only behavior is flagged for human confirmation.
- A law must fail on the bug it names — and the axis it sweeps is the one the author *didn't* think of. Four times in the 89–99 wave a green law hid a real defect by only checking imagined cases: one window geometry (89), a hardcoded mono-face list (97), hand-picked icon near-pairs (99, where the true closest pair was unpinned), a drag test that never advanced the clock (94). Sweep the axis (all geometries, the whole roster, the empirical worst case), and prove non-vacuity by breaking the product and watching the law go red — item 89's headline occupancy law survived the exact regression it was named for.
- The harness stays real: verified behavior is live behavior. When a bug won't reproduce headlessly, extend the harness toward reality rather than stubbing around it.
- Spend complexity where the product is. Editing edge-cases (graphemes, wrap, undo coalescing, CRLF, boundary motion) are the product — spend generously, test exhaustively. Infrastructure complexity is a smell: themes are data through one renderer, and a theme needing its own code path means the design is wrong. When cutting, cut machinery, never editing correctness.
- Perf is measured: `--bench-perf`, `--bench-frame`, `--bench-theme-burst`; record a before on base, and make the bench witness the work (assert a reshape count — one theme bench "measured" 5ms while nothing reshaped).

## Conventions

- Input path: keys → `keymap.rs` (`Action`) → `actions.rs::apply_transition`. Every interaction stays drivable by `--keys` and visible in the sidecar.
- Determinism: the headless path has no clock, animation, or randomness; live-only animation captures its settled state.
- Per-frame work is O(visible), not O(doc) — follow the proto-cache shape in `render/rects.rs`.
- Picker rows go through `render/rowlayout` (docs/render.md), never hand-placed.
- Comments state what the code can't say about itself. Not history ("BEFORE this existed…"), not design memos, not a changelog — that belongs in the commit message, which is where `git log -p` will find it. Never cite a queue item, round, or sha without verifying it exists.
- Design discipline (DESIGN.md): one accent — the caret; figure/ground by value; summoned overlays over persistent chrome; four syntax roles, no rainbow.
- No web artifacts: design ideas are prototyped in awl via headless capture, not HTML mockups.
- The repo is public: tracked files carry no personal-machine paths or private-notes references — `$HOME` over `/Users/<name>`, "the repo root" over absolute paths, "the user's notes (private)" over their location.
- A new `ViewState` field gets an inert default in `ViewState::base()`; the one exhaustive construction site is `sync_view`, which must fail to compile on the new field — forcing a conscious render decision.
- When replay is clean but the user still sees it, hunt the live-only classes: stale caches across buffer swaps, missing invalidation on resize/page-drag, redraw-scheduling gaps.

## Docs voice (user-set)

User-facing docs (CREDITS, GUIDE, welcome/tour, site pages) are matter-of-fact: tables and short declarative sentences, facts traced to verified sources, no filler. PHILOSOPHY/DESIGN keep their personal register.

## Branches & pushing

Development happens on local `main` (it may run ahead of origin; `git remote show origin` is the default-branch truth). A green train — a receipt from `scripts/native-gate.sh`, wasm — authorizes a push; CI minutes are a non-concern (user rule 2026-07-15). The receipt is the only authorization to call the native tier “full native suite”: `cargo test --bin awl` is binary unit tests and any filtered invocation is targeted tests; counts never prove scope. **And the receipt is hardware-bounded: it certifies "sound on the hardware the receipts run on, with virtualised-GPU behaviour untested by any local gate."** The gate uses whatever GPU the host has — on the dev machine, real Apple Silicon Metal — and item 231's wedge was green there while red on hosted macOS's virtualised Metal for ~140 commits. A software adapter is not a stand-in for that axis (measured, item 232): CI's `linux` job runs this same gate against Mesa lavapipe and stayed green through the whole streak, and a local lavapipe container never once hung at either bisect boundary. The only arm that has ever seen this axis is CI's hosted-mac jobs. Since item 243 (user decision 2026-08-03, resolving item 232's parked question), that arm is split: `mac (build + test, minus render::tests)` GATES `main` directly, and `mac (render::tests)` is tolerated red, pinned by name to item 231 in the workflow file itself. Tags and releases wait for the user's explicit word, every time. Worktree branches never push — with ONE **temporary** exception, by user decision 2026-08-04 (item 249), which **item 250 exists to remove**: a branch MAY be pushed **solely to collect CI evidence on an arm no local gate can reach** (lavapipe, hosted macOS). It is opened as a PR for that purpose alone, is never merged from, and the remote branch is deleted once the run is recorded — the run and its logs outlive the branch, which is the point. This paragraph reverts to the absolute rule the moment 249's evidence is in hand; if you are reading it after item 250 closed, it should not be here. Pushing the branch by itself runs nothing: `ci.yml`'s push trigger is `branches: [main]`, so the PR is what fires it, and `workflow_dispatch` needs the ref on the remote too. Everything else about the rule stands — an evidence branch is not an integration route, and a worktree agent verifies its base with `git merge --ff-only main` and stops to report if it won't fast-forward. The merge train integrates one branch at a time, gated on the full suite; for structs with per-call-site initializers, grep the construction sites before declaring a merge done (git merges a missing field cleanly and fails to compile later). A genuine product/taste conflict is grounds to abort and hand back.

## Open decisions & known divergences (do not re-discover)

- **CRLF / lone-CR / U+2028 (resolved — the VS Code model, docs/platform.md):** the rope is always pure `\n`; load normalizes, save restores via `Buffer::disk_bytes`; a lone `\r`/NEL/LS/PS is content. EOL is document metadata, not on the undo timeline.
- **History ownership (settled):** a git-managed file's timeline is `git log` alone. Loose files snapshot on every save, pruned by the aged ladder. Autosave still writes git files (writing ≠ version-meddling).
- **Shift-PageDown/PageUp** deliberately do not extend a selection (documented non-movers in the `is_motion` test); promoting them is a conscious follow-up, not a bug.
- **Shared orchestration board:** build queues, dependencies, and status live in `.orchestrator/queue.md` — the one tool-neutral source of truth (ROADMAP.md is product direction). A queue orchestrator reads `.orchestrator/README.md` before every dispatch wave. Its protocol: claim on the board and commit before writing code; work in a worktree named on the claim line; board writes are orchestrator-only — workers report shas + outcomes. Every dispatched worker gets an explicit model and effort chosen by role; never silently inherit the orchestrator's defaults. Inheritance is allowed only when the brief deliberately records that the worker needs the same model and effort.
- **Design-session flow (2026-07-22, README §Design sessions):** brainstormed decisions land as self-contained queue items committed with an `orchestrator: decisions` subject; git is the log — no decisions file. The user's notes (private, outside the repo) are the user's space: agents read there, never write.
