# awl docs — Licensing, credits & supply chain

> Read before touching licenses, bundled assets, dependencies, `cargo audit` findings, or release paperwork. Moved verbatim out of CLAUDE.md 2026-07-22 (queue item 17); earlier round history: `git log -p CLAUDE.md`.

## Licensing & credits

- awl's code is **GPL-3.0-only** (`Cargo.toml`, flippable to `-or-later` — sole copyright holder's call); `NOTICE` states copyright. Bundled-asset licenses live beside them (`assets/fonts/LICENSES.md` all-OFL, `assets/dict/LICENSES.md` — `en_GB` LGPL-2.1, `en_US`/`en_AU` no in-file statement, recorded as an open gap). **Never fabricate a license fact — flag what's unverifiable.**
- `THIRD-PARTY-LICENSES.md` is generated (`cargo about generate about.hbs -o …`), never hand-edited; `about.toml` is its license-inventory owner. `deny.toml` independently blocks license drift across the full product graph using the resolved GPL-compatible license choices; it does not make claims about bundled assets. `CREDITS.md` (warm, PHILOSOPHY voice) is `include_str!`'d; Cmd-P → "Credits" opens it (`App::open_credits` refreshes a real on-disk copy first — a path-less buffer reads as scratch to autosave and would clobber the stash).

## Supply chain

- **Run `scripts/audit.sh` each merge-train day** (install exactly `cargo install cargo-deny --version 0.20.2 --locked`). It is the sole dependency-policy gate: RustSec advisories and yanks, licenses, duplicate crate versions, and registry/git sources. It checks the macOS, Linux, and wasm targets with all features, including the macOS `mas` flavor. Semver-compatible fix → `cargo update -p <crate>` (minimal, never major; `wgpu` stays exact-pinned) + the targeted test slice. No non-major path → record one narrow advisory ID, risk note, and removal condition in `deny.toml`; cargo-deny fails an ignore that no longer matches the graph.
- Standing accepted findings (no non-major path): RUSTSEC-2026-0194/0195 (`quick-xml` 0.39.4, gated behind a `winit 0.30` bump via `wayland-scanner`; parsed XML is the build-time Wayland protocol spec, not attacker input) and RUSTSEC-2026-0192 (`ttf-parser`, unmaintained, no patch short of a font-parser swap). Their exact rationales/removal conditions live only in `deny.toml`; re-check when an upstream `winit`/`cosmic-text` release picks up the fixes.
- **The zero-network property is a design invariant, not an accident.** awl never phones home, never fetches at runtime (no telemetry, no remote font/dict/theme download, no update checker — see Check for Updates). Any future language pack is a file dropped into `fs::data_root()` or bundled at build time. `cargo deny`/`update`/`install` are build-time tooling and don't compromise this.
