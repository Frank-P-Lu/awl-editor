# Capture evidence and determinism

Read when asserting geometry, pixels, profile parity, or coverage.
[Capture guide](../../CAPTURE.md) · Paths in commands are repo-relative.

## Determinism

A single-frame capture is byte-stable for identical inputs on the same machine.
The default canvas is 1200×800; `--capture-size`, DPI, zoom, config, and theme are
resolved inputs. The texture format is `Rgba8UnormSrgb`, single-sampled, and cleared
to the active world's `base_100`. Read effective font and layout metrics from the
sidecar rather than assuming a fixed origin or cell pitch.

A freshly loaded file starts at line 0, column 0. [Key replay](replay.md) applies
real transitions before the frame. Single-frame capture has no live clock, blink,
or animation progression; the chosen caret and ground are rendered deterministically.
Use the [motion modes](motion.md) for explicit trajectories.

Developer-only pose overrides are memoized at startup and inert when unset:

| Override | Subject |
| --- | --- |
| `AWL_LAVA=<palette>:<phase>[:edge][:dither]` | Lava lamp |
| `AWL_STARS_PHASE=<cycles>` | Twinkling stars |
| `AWL_WAVES_PHASE=<cycles>` | Bombora waves and Bowerbird value-breathe |
| `AWL_WARP_PHASE=still\|settled\|start\|wrap\|<seconds>` | Kite travel |

These are not user settings and do not appear in help. Embedded faces give stable
shaping inputs; system fallback for missing glyphs may differ by OS. State is a
separate claim from geometry and pixels: do not use fallback-dependent `layout.xs`
as a cross-platform golden.

## Read the right oracle

Check that artifacts exist, content and cursor match the scenario, and `layout.rows`
reports the expected wraps, advances, heights, and selections. Compare duplicate
runs on the same configuration when asserting determinism.

The sidecar proves state. Its `layout` borrows the sealed shaped-frame partition;
it does not reshape, estimate pitch, or reconstruct advances. The PNG proves
appearance. A correctly selected row with correct geometry may still be invisible.
Assert visibility, contrast, and distinction over pixels, using
`render/tests/pixeldiff.rs::assert_perceptibly_different` or `assert_identical` with
an appropriate region and floor. Instance counts or computed colors prove intent,
not the final image. Follow the verification policy's backend and mutation rules.

A rebuilt frame cannot establish live swap-cache correctness, resize invalidation,
or redraw scheduling. The compositor's behavior between submitted frames—stretch,
tear, dropped/coalesced frames during rapid resizing—is a separate live boundary.
A correct mid-motion PNG does not prove every frame was presented. Keep timing,
feel, and taste subject to live human confirmation.

## Debug/release sidecar differential

`scripts/release-profile-gate.sh` builds both profiles and compares strict,
deterministic captures across state-bearing action families (`Buffer`, `Viewport`,
`Format`, `Overlay`, `Session`, `View`, `Align`, `Export`). The debug state is the
differential oracle; pairs must be byte-identical. Run once on combined main before
a push train and before a tag, not at every worktree landing. This is headless;
platform presentation retains its [separate checks](platform.md).

## Periodic missing-law coverage audit

Before a risky implementation wave and before release preparation, run
`scripts/coverage.sh`. It pins cargo-llvm-cov and a branch-capable nightly and writes
ignored coverage artifacts: HTML, LLVM JSON, commit/tool/config provenance, and
changed-code triage. Ordinary push CI does not run or upload this manual audit;
there is no repository-wide percentage target.

Uncovered changed paths are a reading list. Add a law only for a real behavior
contract. Covered code does not prove pixels, compositor behavior, or feel.
`coverage/provenance.txt` names exclusions: the native process entry point and
direct AppKit/window-server boundary. Keep their live evidence obligations.
