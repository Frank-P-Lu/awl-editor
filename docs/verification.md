# Verification policy

Read before planning checks, dispatching implementation, changing a source audit,
or deciding whether an existing result covers a later commit. This policy owns
verification scope and ordering; the orchestration guide owns dispatch mechanics.

## Order the work by cost

1. Inspect the diff and identify the behavior, platforms, and source audits it touches.
2. Run formatting, relevant source audits, compiler/lint checks, and targeted tests
   before expensive GPU sweeps. Register new benchmark modules and fixtures here.
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

Automatic dependency-aware receipt reuse is not implemented. Until it is, executable,
test, asset, Cargo, shader, and CI changes require the final full gate. The current
gate's commit-freeze rule still applies; never change its tree or HEAD mid-run.

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
