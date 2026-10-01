# Verification policy

Read before planning checks, dispatching implementation, changing a source audit,
or deciding whether an existing result covers a later commit. This policy owns
verification scope and ordering; the orchestration guide owns dispatch mechanics.


## Three verification layers

Layer 1 is the editing loop. Run `scripts/verify.sh fast UNIT_FILTER...` with
explicit selectors for the owners changed by the diff. For example,
`scripts/verify.sh fast buffer:: actions::` checks formatting, compiles all native
targets, runs the existing diagnostic/substitution/view-policy ownership audits,
and runs the selected unit tests. `--lint` before the selectors adds strict
all-target/all-feature Clippy. This avoids the full health wrapper's process-budget
stress tests during every edit. An unmatched selector fails rather than reporting
an empty pass. Selectors are not automatic dependency analysis: inspect the diff
and include neighboring behavior and relevant convention/menu branches.

Integration targets need their real seam too: use, for example,
`cargo test --test fault_kill9` and `cargo test --test persistence_real_process`
when changing atomic writes or persistence. Keep Unicode, selection, undo, save
fidelity and data-loss regression checks in the editing loop for affected owners.
Reintroduce a repaired headline defect to prove its regression assertion fails.
Render changes retain the changed-axis outcome audit and about five real visual
smoke captures; neither a compiler check nor a state sidecar proves appearance.
Worker launches use the existing worker-build wrapper.

Layer 2 is the frozen combined candidate. Commit, freeze and run
`scripts/verify.sh full` once. It invokes the existing full native gate (including
complete code health and its tooling laws), wasm build/test compilation/Node smoke,
and eight-family debug/release parity, failing on any command error. The wasm
runner is required here so core runtime execution cannot be skipped. The final
HEAD and working tree must still be unchanged. This is one composed gate, not
permission to replace required coverage with filtered tests.

CI preserves the current required Linux, Mac non-render, wasm and Mac live-probe
jobs. Linux invokes native-gate.sh once; its redundant standalone complete health
invocation is removed. Web remains its own platform job, and push profile parity
remains blocking. The Mac rendering and AT-SPI jobs retain their existing declared
tolerances and visible gap reports. Local Metal/software Vulkan does not certify
hosted virtualized Metal. Do not run complete health immediately before the
native gate just to repeat it. Workers deliver targeted evidence; combine their
changes and gate the final candidate, not every worker branch by default.

Layer 3 is broader overnight and pre-release validation.
Extended verification runs daily at **03:17 UTC** on the default branch, or manually
through `extended-verification.yml` with mode overnight/pre-release. It is also
called by the nonpublishing and tagged release workflow before package jobs.
Both hosted Linux and macOS build release once, sweep the binary's complete world
roster at both DPI values using `scripts/verify.sh extended --bin
target/release/awl --jobs 1`, then run a longer 120-second live GPU launch.
Journey inputs use the host convention, including Linux Ctrl-End for document
end; the existing `AWL_CONVENTION_FORCE` fixture door reproduces either convention
locally. Typed text, Shift chords and every pixel/state assertion are preserved.
A finding or failed app contract fails that job; there is no new tolerated-failure
arm. The existing full unit/integration suites are not duplicated overnight.

The release workflow then validates its actual artifacts: the executable inside
the mounted Mac DMG, the extracted Linux tarball binary, and the AppImage's real
extract-and-run entry point each launch with synthetic isolated fixtures and
the built app's live presentation/recovery/memory verdict. The launcher is
CI-only because it opens a window. The app's isolated `--soak-gpu` mode owns
its synthetic state and rejects document, config and folder arguments; the helper
supplies only probe arguments and fresh process-data directories. This proves
live artifact presentation/recovery, not opening a supplied document. Package
metadata, signing policy, checksums, compression-size checks and the existing
release authorization remain in place.

The automation is not a human journey. Before a release, finish the real desktop
launch, native menus, physical IME/keyboard, browser clipboard and accessibility
journeys listed in RELEASING.md, WEB.md and ACCESSIBILITY.md. Do not grant automation
permissions, sign with new credentials, publish a release or deploy as part of
overnight checks. Existing artifact retention is seven days for synthetic journey
evidence; failed runs keep their available evidence too.

Commands and failures in this layer are defined by the existing check owners;
`scripts/test-verification-pipeline.py` proves dispatch, failure propagation,
nonempty targeted runs, frozen-candidate checks, workflow coverage wiring and
built-binary launch isolation. It runs from code health so wiring regressions
block the final gate. Gate-tool changes still require a real final-gate rehearsal.

## Matching Rust locally and in CI

`rust-toolchain.toml` owns the exact project version (currently Rust 1.99.0),
Clippy/rustfmt components and wasm target. Standard rustup proxies honor the
file. Direct links to `toolchains/stable.../bin` bypass it; use the scoped
`scripts/project-rust.sh cargo ...` wrapper to select the actual pinned binaries
even with those links. The wrapper uses `rustup run`, retaining the official
linker-library environment as well as the compiler selection. `verify.sh`
activates this helper automatically; other build helpers preserve its PATH. Activation affects only the command
process, leaving global links and the default unchanged. CI, extended verification, release and web build
workflows activate the same file through `.github/actions/project-rust` and
print the selected compiler. Release's extra Mac architecture targets remain.
The separate nightly coverage instrument retains its existing nightly pin.

After changing the project pin, run `rustup show active-toolchain` inside the
repo to install/sync its official toolchain, then verify
`scripts/project-rust.sh rustc --version` and
`rustup target list --installed`. On a Mac, also install the pinned version's
`x86_64-unknown-linux-gnu` target to retain the cross-platform Clippy check.
Run targeted checks and `scripts/verify.sh full` on the frozen upgrade candidate,
then require the exact hosted checks before accepting the upgrade. Do not change
`rustup default`; other repositories keep their existing defaults.

## Order the work by cost

1. Inspect the diff and identify the behavior, platforms, and source audits it touches.
2. Run formatting, relevant source audits, compiler/lint checks, and targeted tests
   before expensive GPU sweeps. Register new benchmark modules and fixtures here.
   `scripts/preflight.sh` composes exactly this step's existing check owners
   (`scripts/code-health.sh` plus the benchmark-output/view-construction
   ownership audits) into one standalone, GPU-free command; its pass is
   targeted evidence, not a full receipt.
3. Run the required outcome audits and release measurements once the candidate is
   stable. Measurements run without competing builds. Tests that skip their GPU
   subject are reported as skipped, never as evidence that the behavior passed.
4. Integrate branches sequentially into one deliberate candidate. Commit and freeze
   it, then run `scripts/native-gate.sh` and `scripts/web-smoke.sh`. The native gate
   already runs `code-health.sh`; do not run that complete wrapper immediately before
   it just to repeat the same checks. Its cheaper constituent checks are useful earlier.

Workers deliver targeted verification, outcome evidence, and known gaps. A full gate
on every worker branch followed by another on the same integrated change is not the
default. Request a worker full gate only when an identified integration risk warrants
it. This scope rule also applies when an older queue brief mechanically lists both.
An explicit user requirement for additional verification still takes precedence.

The ignored `app::tests::summoned_field_actions::browser_paste::
browser_paste_bulk_release_measurement` test measures a 45,056-byte Unicode
clipboard fixture through all seven `TextField` roster members in release mode.
Its sibling law counts actual splices, refilters, search recomputations, previews
and field mirrors; timing is observational, never the correctness oracle. Neither
test covers the separate Table Dimensions grid or proves browser/operating-system
clipboard delivery.

The merge train may collect related changes into a bounded candidate, inspecting and
compiling each merge, then gate that candidate once. It does not push or describe the
candidate as verified until the full required checks pass. After a failure, diagnose
all available failures and run the repaired slices before another full gate. Do not
rerun a known failing candidate unchanged except to investigate a suspected flake.

## State what a result proves

Record the command, tested commit, scope, configuration, outcome, and material skips.
For GPU evidence, name the backend/hardware class; local Metal does not prove hosted
virtualised Metal. Keep the existing convention, menu-bar, wasm, and applicable
cross-backend coverage. A filtered test run is targeted evidence, not a full receipt.

A check's reusable evidence must include its inputs: source and assets, tests and
fixtures, build configuration, toolchain, environment branches, and relevant hardware.
A change to any of these invalidates the affected result. Unknown dependencies mean
rerun, not guessed independence. Changing a test invalidates that test's prior result;
a bookkeeping repair cannot turn a failed full run into a passing receipt.

A conservative, explicit-registry reuse mechanism (`scripts/verify_cache.py`) exists
for a small, named set of checks: it reuses a cached PASS only when every declared
input — tracked source/tests/config, the command, the toolchain, the hardware class,
and the check's own declared environment branches — hashes identical, never reuses a
cached failure, and is not wired into the gate path. It does not amount to
dependency-aware receipt reuse for the gate as a whole. Until a replacement proves
equivalent coverage, executable, test, asset, Cargo, shader, and CI changes require the
final full gate. The current gate's commit-freeze rule still applies; never change its
tree or HEAD mid-run.

After a successful gate, a prose-only policy or queue edit does not require rerunning
Rust/GPU tests. Inspect the exact diff, check relevant links, and cite the original
validated commit plus the documentation-only commit; do not relabel the old receipt.
Embedded product documentation, generated fixtures, and scripts are inputs, even when
they look like documentation. Run the checks their consumers require. Gate-tooling
changes need the tool's own laws and a real gate rehearsal, not just a prose review.

## Prefer rules about behavior and ownership

New source audits check the rule: application diagnostics use the notice owner,
benchmark diagnostics stay in benchmark modules, and substituted document views use
the substitution owner. Prefer type/module boundaries or syntax-aware checks over
exact print counts, line counts, or assignment spellings. Enroll new modules by their
structural role and test that the forbidden case is detected.

Existing count-based audits remain enforced until replaced. When touching one, assess
whether a small structural replacement removes the recurring maintenance cost without
weakening its subject. Keep a narrow reviewed registration when replacement is a
separate task; do not grow a generic exemption or disable the audit to get green.

File and function size thresholds trigger design review. Around 500 file lines or
100 function lines, look for a coherent owner worth extracting. A longer exhaustive
constructor, transition, or law can be clearer than several helpers with shared state.
Record that judgment in the existing code-health exception mechanism. A reviewed
size increase needs a concrete cohesion reason, not user permission. Do not extract
solely to hit a number, silently raise a hard safety limit, or waive unrelated lints.
The current checker still requires its exact manifest entries until tooling changes.

## Preserve the strong guarantees

Editing correctness, data preservation, Unicode, undo, invalidation, and accurate
rendering remain product requirements. Test at the purest seam that exercises the
real behavior, with meaningful configuration coverage. Headline regression laws must
fail when the named regression is deliberately reintroduced and compiled. Appearance
claims still need pixel evidence and the required visual review.

Performance claims require release before/after measurements and counters at the actual
work owner. Report remaining full-document work and latency boundaries honestly.
Infrastructure should make these guarantees easier to establish, not replace them
with procedural counts. Releases and tags still require explicit user authorization.

## Outcome audits

Audit agents use Sonnet medium on Claude or `gpt-5.6-terra` at `medium` on OpenAI.
Enumerate state × surface × world along the changed axis, asserting outcomes per
cell using pixels and sidecars. Required triggers:

- A new axis value: probe the full surface roster.
- An identity-gated refactor: audit outcomes; byte identity preserves existing bugs.
- A user-reported bug: audit neighboring states and surfaces.
- A degradation arm: probe that state, name its compensation, and law-test it.
- Pre-tag: sweep journeys across worlds.

Every render-touching round gets a visual smoke over about five gallery shots,
using affordance-locating questions such as “which row is selected?” An audit that
finds a defect ends with the missing law test.

## Test globals and GPU evidence

Every test and every `cfg(test)` global reader/writer takes the one process-wide
reentrant `crate::testlock::serial()` guard. Do not reintroduce ordered per-module
locks. `config::ENV_LOCK` separately serializes environment mutation. The guard
restores snapshotted state, including forced render overrides, even during unwind.
`capture::sidecar::write_sidecar`, filesystem globals, and `test_gpu::arrive` assert
ownership in tests.

The shared GPU is mutable global state even when a caller only borrows handles.
The guard must outlive the resources, including their destructors; a helper-local
lock cannot protect returned resources. Use `test_gpu::adapter_present()` for an
adapter skip check before taking the guard. A privately created device is outside
this shared-device rule, but cannot supply another device's counter baseline.
Investigate filtered runs as well as full runs when diagnosing counter races.

Restore captured ambient values, not values inferred from `cfg!(target_os = …)`.
The native gate's `menubar-full` arm runs every binary unit test under the opposite
`AWL_MENU_BAR_FORCE` branch and records `menubar=full:<branch>`. A name filter is
not a substitute. Set environment variables explicitly in script laws and sweep
both CI and non-CI behavior; inherited environment is an uncontrolled test input.

GPU allocation laws use `gpu_alloc::probe` on the actual device and assert only
classes whose counters respond. In wgpu-hal 29.0.3, Vulkan texture counters can
walk negative because normal creation does not increment them; `CoreCounters` is
empty. Prove cross-backend claims on a second backend. Software Vulkan does not
stand in for hosted virtualised Metal. The hosted macOS build/test job gates main;
the separate `render::tests` job is tolerated red as declared in the workflow.

## Laws must detect their subject

Derive enrollment from the roster, assert that it is nonempty and independent of
configuration, and name enrolled cases in failures. Check that the assertion would
reject the actual broken state. Appearance treatments need a presence floor as
well as a contrast floor: deleting a selection wash must not improve its score.
Sweep relevant DPI, geometry, backend, entry point, test filter, and environment
branches. A law's configuration is part of its claim.

Validate defect reports against the product and the measurement tool's actual
behavior, including dependency filtering and host tool differences. If the premise
is false, report “premise false, oracle repaired,” not “fixed.” Read values for
human taste decisions from the product rather than an agent's report. Spot-check
generated references against the code they describe, probing both sides of any
condition the generator collapsed.

Performance evidence uses release `--bench-perf`, `--bench-frame`, or
`--bench-theme-burst` as appropriate, with a base measurement and counters proving
the intended work ran. Headless paths remain deterministic, without clock,
animation, or randomness; live animation captures its settled state. When replay
passes but live behavior fails, check buffer-swap caches, resize/page-drag
invalidation, and redraw scheduling.
