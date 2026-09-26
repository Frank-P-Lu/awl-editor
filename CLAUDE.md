# Working on awl

AGENTS.md is a symlink to this file; edit CLAUDE.md. Keep this entry point concise:
repository-wide rules belong here, mechanisms in the linked docs, history in git,
and work status in `.orchestrator/queue.md`.

## Product

Start with [PHILOSOPHY.md](PHILOSOPHY.md). awl is a calm, opinionated plain-text
editor for prose and light code, built with Rust, wgpu, winit, and glyphon. Native
and browser builds share one core. Markdown stays plain text; rendering becomes
rich and the caret reveals editable source. Preserve editing correctness, Unicode,
undo, save fidelity, and immediate response. No Word-style document model, styled
clipboard, general formatting toolbar, or IDE machinery. Runtime is offline:
no telemetry, asset fetching, or required service.

Follow [DESIGN.md](DESIGN.md): the caret is the accent, hierarchy uses value,
choices use summoned surfaces, and syntax has four roles. Prototype design in awl
with real captures; do not build HTML mockups. Themes are data through one renderer;
a theme-specific code path calls for design review.

## Read before working in an area

Read the matching documents before editing; do not load the entire list for every task.

| Work | Required reference |
| --- | --- |
| Checks, tests, audits, or implementation dispatch | [docs/verification.md](docs/verification.md) |
| Capture plans or evidence | [docs/harness-reach.md](docs/harness-reach.md), [CAPTURE.md](CAPTURE.md) |
| Architecture or App ownership | [ARCHITECTURE.md](ARCHITECTURE.md), [docs/app-domains.md](docs/app-domains.md) |
| Configuration, bindings, page width | [docs/config.md](docs/config.md) |
| Markdown styling, conceal, formatting, links | [docs/markdown.md](docs/markdown.md) |
| Fonts, CJK, fallback | [docs/fonts.md](docs/fonts.md) |
| Syntax or spell scoping | [docs/syntax.md](docs/syntax.md) |
| Layout, chrome, pickers, render state | [docs/render.md](docs/render.md) |
| Worlds | [THEMES.md](THEMES.md) |
| Persistence, menus, sessions, GPU faults, live probes | [docs/platform.md](docs/platform.md) |
| Browser build | [WEB.md](WEB.md) |
| Release pipeline or publishing | [RELEASING.md](RELEASING.md) |
| Accessibility | [ACCESSIBILITY.md](ACCESSIBILITY.md) |
| Licensing or dependencies | [docs/licensing.md](docs/licensing.md) |
| Queue orchestration or design-session decisions | [.orchestrator/README.md](.orchestrator/README.md) |

## Build and verification

Use incremental builds; never `cargo clean`. Match the surrounding code's style.
On macOS, run the bundled app with `scripts/dev-app.sh`. Measure performance and
judge feel in release builds, with before/after measurements witnessing real work.

```sh
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo build
scripts/native-gate.sh
scripts/web-smoke.sh
```

Choose scope through `docs/verification.md`: cheap checks, targeted tests and
required outcome audits, then one full gate on the frozen integrated candidate.
Workers normally deliver targeted evidence. Only a `scripts/native-gate.sh` receipt
authorizes the phrase “full native suite”; filtered runs and binary unit tests do
not. State tested commit, configuration, skips, and hardware limits. Local Metal
and software adapters do not establish hosted virtualised-Metal behavior.

For a headless capture:

```sh
cargo run -- --screenshot OUT.png [file]
```

Read the capture references before selecting a driver. Ordinary captures drive the
shared core; `--screenshot-app` drives a real headless App. Neither proves every
live interaction. Sidecars prove state; PNG pixel arithmetic proves appearance.
Timing, live feel, and taste require human confirmation. Extend the harness toward
real behavior when needed. Render changes require the outcome audits and visual
smoke defined in the verification policy.

## Engineering rules

- Give each behavior one owner, route all consumers through it, and make bypasses
  private. Use exhaustive law tests so new roster members cannot escape coverage.
- Test at the purest real seam. Confirm a reported defect and its measurement;
  prove headline regression laws fail when the bug is reintroduced. Sweep the
  changed configuration axis, including relevant environment and platform branches.
- Preserve source audits until replacements prove the same guarantees. Size
  thresholds trigger design review, not automatic extraction; record justified
  exceptions through the existing code-health mechanism.
- Input flows through keys → `keymap.rs` Actions → `actions::apply_transition`.
  Keep interactions replayable and observable. Native menu actions also need
  action-level interception; a key-only guard cannot protect them.
- Keep per-frame work O(visible). Picker rows use `render/rowlayout`. Cache keys
  using `buffer.version()` also need buffer identity or invalidation on swap.
- New `ViewState` fields get inert defaults in `ViewState::base()`; keep `sync_view`
  exhaustive so adding a field forces a render decision.
- Every test and every test-global reader/writer holds `crate::testlock::serial()`;
  keep the guard alive through shared GPU resource destruction. Details and
  environment-locking rules are in the verification policy.
- Comments explain mechanisms, not incident history, queue items, rounds, or shas.
  User-facing docs use short, factual prose with verified sources; PHILOSOPHY and
  DESIGN keep their personal register. Never invent license facts.
- Public files and captures contain no personal-machine paths or private-note
  references. Seed file-picker captures with an explicit `--root` and `--config`;
  sidecar redaction does not sanitize photographed rows. The user's private notes
  may be read, never written.

## Branches & pushing

Development happens on local `main`; verify the remote default with
`git remote show origin`. Before every commit, check `pwd`, the current branch,
and the staged diff. Stage explicit paths only, never `git add -u` or `-A`.
Preserve unrelated work. Never stash or reset the index during an uncommitted merge;
verify a merge commit has two parents.

Never commit or mutate the candidate while your gate runs. Commit, freeze, gate,
then push. Add new files before `code-health.sh`, which checks tracked files.
A green native receipt plus wasm checks authorizes a push. Prose-only policy/queue
changes use diff/link checks and cite the prior validated commit without relabelling
its receipt. Check in-flight CI before pushing; a newer push cancels it. Missing
`conclusion` means inspect `status` and wait when required.

Tags and releases require the user's explicit word every time. Release dry runs
skip `publish`; changes there need self-diagnosing failures and cannot claim
rehearsal coverage. Worktree branches never push. Worktree agents first run
`git merge --ff-only main` and report if it cannot fast-forward. Integrate branches
sequentially, inspect struct construction sites, and gate the combined candidate.
Hand back genuine product/taste conflicts.

Queue orchestrators read `.orchestrator/README.md` before each dispatch wave.
Claim and commit before implementation; use the claimed worktree. Only orchestrators
write the board; workers report shas and outcomes. Choose explicit model/effort by
role; inheritance needs a deliberate statement in the brief. Design decisions land
as self-contained queue items with an `orchestrator: decisions` commit.

## Worktree lifecycle

Create agent-owned worktrees through `scripts/worktree.py create` under the main
checkout's `.worktrees/<task>-<purpose>/`, including detached baselines and one-off
experiments. Register tool-managed exceptions through its `state` command. Keep
lifecycle and review locations current; never infer retirement from age or a
completed turn. Use `scripts/sweep.sh --list` and `--retired` for inventory and
preview. Only explicitly retired, stopped, preserved work is eligible for removal.
See [docs/worktree-lifecycle.md](docs/worktree-lifecycle.md) for the procedure.
