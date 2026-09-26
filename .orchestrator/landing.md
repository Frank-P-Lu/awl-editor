# Landing and CI

Read before integration, receipt assessment, pushes, or CI diagnosis.
[Orchestration guide](README.md) owns the board;
[verification policy](../docs/verification.md) owns check scope and evidence reuse.
Paths in commands are relative to the repo root.

## Integrate a bounded candidate

Inspect and compile each branch integration sequentially. Check struct construction
sites: a clean textual merge can omit a required field. Workers supply targeted
checks; request a worker full gate only for a stated integration risk. Run the
full native gate, wasm smoke, and required outcome captures on the combined
candidate. Profile-parity and pre-release checks retain their documented triggers
in [CAPTURE.md](../CAPTURE.md) and [RELEASING.md](../RELEASING.md).

Before the gate, stage only intended paths, commit, and confirm a clean candidate.
The receipt records HEAD, while Cargo reads the working tree; staged-but-uncommitted
code cannot be certified by a receipt naming the previous commit. Never change
HEAD or the tree while the gate runs. After failure, repair the failed slices
before repeating the full gate. Do not describe skipped or filtered tests as a
full native suite; only `scripts/native-gate.sh` issues that receipt.

For prose-only policy and queue edits, inspect the diff and links and cite the
prior validated commit without relabelling its receipt. Embedded docs and fixtures
remain test inputs. Native evidence is bounded to its actual hardware; local Metal
and software Vulkan do not cover hosted virtualised Metal. Report those limits.

## Code-health integration

`scripts/code-health.toml` is orchestrator-owned at merge time, never assigned to
one worker lane. Workers report required mark/exception changes and reasons; the
orchestrator applies them against the combined tree. Follow the current checker
and verification policy, not remembered thresholds. A reviewed mark raise needs a
cohesion reason; it cannot waive a hard safety limit or unrelated lint.

Read counts from the integrated files, not worker reports. Inspect all affected
marks together when multiple lanes touch one file. Target clippy exceptions by
file plus function, not repeated message text. Parse TOML after programmatic edits.
Stage new files and the manifest before code health; the scanner sees tracked
files while configuration is read from disk. Include the configuration in the
frozen candidate, and check status before relying on its receipt or pushing.

## Push and assess CI

Normally push after two or three landed items; routine board/docs edits ride that
train. Push immediately for a CI repair, correctness/data-loss fix, or requested
checkpoint once its checks pass. Inspect authorizing exit statuses before making
the separate push tool call. Keep the local stable toolchain aligned with CI
(`rustup check`). Tags and releases require explicit user authorization.

Check main's CI before and after a push. Do not supersede a run whose result is
needed to decide whether a repair worked, even with a documentation-only push.
Let one such run finish. Inspect the last successful sha as well as the latest
run; cancellation is not verification.

```sh
gh run list --branch main --limit 12 --json headSha,status,conclusion
gh run view <id> --json jobs
```

Use the workflow's declared gating jobs and their failed steps to interpret the
result. A roll-up `failure` or `cancelled` alone is not a diagnosis or push decision.

| Evidence | Action |
| --- | --- |
| A gating test/build failed | Record a top-priority `CI RED` item with run URL and first known bad commit; block unrelated integration and ship the repair first. |
| A newer push superseded a run | Coverage is absent; obtain a completed run. |
| A job ran near its `timeout-minutes` | Diagnose real cost or the ceiling; slower pushing does not fix a timeout. |
| Simultaneous cancellations, queued siblings, or `Service Unavailable` without a superseding push | Check GitHub Actions service status; hold further pushes during the outage, then retry. State “unverified remotely.” |
| A tolerated job failed in `Set up job` before tests | Classify infrastructure failure and retry; do not create a product `CI RED` item merely from its job name. |
| All gating jobs passed but a tolerated job timed out | Record gating results and the tolerated timeout separately; assess the workflow, not the cancelled roll-up alone. |

Job-level `continue-on-error` does not make setup failures or timeouts equivalent
to tolerated test failures. Keep the hosted macOS build/test and render-test arms
distinct as declared in the workflow. A green local train supplies neither Linux
nor hosted macOS evidence during a CI outage.

## Mutation and appearance evidence

A mutation proof establishes that the edit applied, the target built, and the
named test ran and failed for the intended reason. Assert replacement counts or
changed hashes; distinguish compilation failure from test failure. Mutate one
subject at a time, restore it, and report the actual assertion failure. If a
combination is needed, establish that the subjects are disjoint. A test that stays
green under its own regression is measuring something else.

Sweep parameters and inspect tests that pin a global off as well as those that
read it. Restore ambient state, not a compile-time platform guess. An audit finding
ends with the missing law. Re-prove loosened sub-assertions by mutation.

Appearance floors compare rendered quantities from the same frame rather than
byte equality against authored colors. Prefer relative distances or ratios that
share rasterizer effects. Record the tightest shipped value, defect value, and
threshold between them. If an absolute bound is unavoidable, reuse the law's
existing perceptual tolerance and prove that the defect still fails.
