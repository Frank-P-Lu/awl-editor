# Platform smoke checks

Read when verifying native menus or the wasm execution boundary.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

## Live menu-click smoke tier (macOS only, LOCAL runs — `scripts/smoke-menus.sh`)

A third verification tier, alongside the headless capture above and `cargo
test`: `scripts/smoke-menus.sh` builds a release `awl`, launches the REAL
windowed app against an isolated `/tmp` fixture, and uses macOS's
**"System Events" GUI scripting** (`osascript`) to click **every item in the
live native menu bar** — generated straight FROM the app itself
(`awl --print-menu-roster`, which prints `menu::roster()` verbatim), so the
script's click list can never drift from what `menu.rs` actually builds.
After each click it asserts the process is still alive, failing immediately
and naming the exact item if one ever kills the app — this is the tier that
caught the real muda menu-bar crash (a Rust-side use-after-free in
`menu::install`; see [native menu ownership](../platform.md)).

**What this covers that the headless harness above structurally cannot:**
real platform menu **dispatch** (`NSMenuItem` click → muda's ObjC
target/action → `MenuEvent` → the winit event loop → `App::handle_menu_event`)
and real **AppKit interaction** (the summoned About card's actual float-panel
render over the live frosted-blur backdrop, the native About/Quit label text
picking up "Awl", Window ▸ Minimize/Zoom genuinely acting on a real
`NSWindow`). The headless `--screenshot`/`--keys` path proves the
roster/routing **data** and the resolve **direction** (`menu.rs`'s own unit
tests) — it cannot construct or click a real `NSMenu` at all (confirmed:
building one off a test thread panics). This script is the other half.

**Requirements — LOCAL runs only, not CI:** macOS, plus **Accessibility
permission** for whatever process runs the script (System Settings ▸ Privacy
& Security ▸ Accessibility) so "System Events" is allowed to control other
apps' UI. No display attached means no menu bar to click, so this cannot run
in a headless CI runner — it is a human-machine, on-a-real-Mac tool.

**A hard-learned safety rule the script itself enforces:** it NEVER launches
its test instance under the shared `awl` process name — always a uniquely
named copy (`awl-smoke-$$`). Two processes sharing that exact name resolve
UNRELIABLY through the Accessibility API (confirmed empirically: `System
Events` returned the SAME window object — verified by moving it and watching
both "processes'" reported position move together — for two different PIDs
both named `awl`), so a naively-named test run risks silently operating on a
REAL, already-open awl instance instead of (or in addition to) its own
disposable one.

Usage:

```sh
scripts/smoke-menus.sh            # release build, full click-through
scripts/smoke-menus.sh --debug    # debug build instead
```

Exit 0 + `SMOKE RESULT: PASS` means every roster item was clicked and the
process stayed alive after each one. A slow/absent clean exit after the final
"Quit Awl" click is logged but NOT treated as a failure — this environment
has been observed to keep a launched `awl` busy even fully idle with zero
interaction (reproduced on an unmodified build with no menu clicks at all),
so it is not evidence of a menu-click regression; the script's own trap
hard-kills the test instance regardless, so the script always terminates.


## Web/wasm core smoke tier (`scripts/web-smoke.sh`)

The parallel tier for the OTHER platform edge: `scripts/web-smoke.sh` builds the
whole crate to `wasm32-unknown-unknown` (L1 — catches a native-only API rotting
the web build) and, when `wasm-bindgen-test-runner` is installed, runs
`src/websmoke.rs`'s `#[wasm_bindgen_test]`s through the node runner (L2 — proves
awl's platform-agnostic core actually RUNS in the wasm runtime). See WEB.md's
"Testing the web build" for install steps. Like the menu live-smoke tier, it
covers a seam the headless PNG/sidecar harness structurally cannot — but the
live browser PIXELS (WebGPU/WebGL2, touch, the rAF loop) still need a real
browser, the web build's own live-only boundary.

‼ **THE TWO DOORS SHARE THE AUTHORED 1.0 DEFAULT, BUT A GEOMETRY FIGURE IS STILL A
FIGURE AT ITS RESOLVED ZOOM.** Windowed launch, ordinary capture, and live-App capture
all resolve an absent override through `range::ZOOM.default`. A persisted `config.zoom`
or an explicit ordinary-capture `--zoom` still changes that axis. **So never compare a
geometry figure without recording `font.zoom`, and say which zoom any width you report
was taken at.** The live probe still uses a live-App reference because equal size does
not make the replay core an oracle for App-owned launch state.
⚠️ **Relatedly, a plain `--screenshot` is not hermetic** — it reads the host's own
`config.toml` unless `--config` is passed, so an un-configured replay measurement can
render a different world than the one you asked about. `--screenshot-app` is sandboxed.
