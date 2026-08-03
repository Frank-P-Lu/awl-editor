# awl — live build queue

> Live execution state only. Completed and superseded work is in git history
> (`git log -p .orchestrator/queue.md`). Protocol, claiming, worktrees, and
> execution hygiene live in `.orchestrator/README.md`.

## ⚠️ RELEASE IS NOT SHIPPABLE TODAY — item 226's dry run, 2026-08-03

**The release pipeline could not have built anything.** Landed at `3160e309`.
Found BEFORE a tag was attempted, which is the whole point of the item.

1. **The whole pipeline was dead.** `.cargo/config.toml` gained
   `rustc-wrapper = "sccache"` on 2026-07-30 (`d76eaaaa`) — **after** the last
   dry run on 2026-07-11. `ci.yml` installs sccache in all four jobs;
   `release.yml` in **none**. Reproduced in a clean container: `could not
   execute process sccache (never executed)`, exit 101, at `rustc -vV`. The
   first tag ever pushed would have failed to compile, publicly.
2. **`publish` could never publish.** The repo's default workflow permission is
   `read` and the job declared none. **Only a tag exercises that path, so no dry
   run could ever have found it** — it took reading the API.
3. **`deploy-web.yml` has the identical hole.** Found by the new law, not by
   looking. That law sweeps the axis a release-only check would have missed:
   every workflow JOB running cargo or trunk must install the wrapper first,
   across all workflow files — `ci.yml` was correct throughout, so a
   release-scoped check would have gone green.
4. **The tarball would have shipped with no `LICENSE`, silently.** Fonts
   (OFL 1.1) and Hunspell dictionaries (LGPL-2.1) are `include_bytes!`d into the
   binary; neither packaging path copied their audits while `RELEASING.md`
   asserted all five docs rode the artifacts — `cp X 2>/dev/null || true`
   swallowed it. No GPLv3 §6(d) source offer either. Now a **hard packaging
   failure**, mutation-proved.
5. **Portability, measured not estimated:** the binary requires `GLIBC_2.39`,
   proved by running the produced tarball on `debian:12`. **Excludes Debian 12,
   Ubuntu 22.04 LTS and RHEL 9.** Build base deliberately unchanged — a
   support-matrix decision; `RELEASING.md` §5 has the four options.

**Linux-only beta is structural now:** `mac` and `web` build on a dry run and
are SKIPPED on a tag, so an unsigned `.app` cannot reach a Release.

## 🔵 BLOCKED ON THE USER — nothing else can close these

⚠️ **This section has now been silently deleted TWICE** — once by an
orchestrator `git add -A` sweeping another tool's in-flight edit, once by the
item-204 worker's own commit `1127673d` despite its brief forbidding board
writes. **After every merge, verify this heading still exists.** If it is
missing, `git log -S"BLOCKED ON THE USER" -- .orchestrator/queue.md` finds who
took it.

1. **118 — the world-loudness map and the `--release` ambient sitting.** The
   Done clause requires a USER-CONFIRMED map; pixel arithmetic may prove
   territory and contrast but never the taste score. An independent agent map
   exists (`1, 10, 3, 4, 1`, mean 2.68) to diff against rather than re-derive.
2. **207 — real VoiceOver and AT-SPI journeys.** Verified at the snapshot and
   projection tier. Whether a screen reader *reads it well* — announcement
   order, verbosity, live-region politeness — is unproven and no test tier
   substitutes.
3. **131c — the chrome pixel-space decision.** Overlay chrome mixes both
   spaces: row pitch scales with DPI while `BAR_SIDE_INSET`, the text hpad and
   `CARD_MAX_W` are raw device px. A diagonal pitch authored like its neighbours
   would be **physical by inheritance**, which item 186 exists to stop; making
   it logical makes it the first chrome quantity to declare its space, either
   extending `ground_space` past `Background` or opening a sibling registry.
   **Owed a human eye, not a line of code.** 131d/131e are behind it.
4. **211 — three narrow arms.** Presence is ✅ confirmed on a real screen
   (2026-08-03) and the defect was reproduced live and restored. Still owed, all
   needing a human: whether the glide **reads as calm** (pacing deliberately
   uncharacterised — the host ran at load 19→57 and the lane refused to offer
   its 16.7 ms intervals as evidence); **1×**, no 1× display was available; and
   **focus loss/regain and occlusion return**, which `--live-script`
   structurally cannot test since it forces a Prohibited `AlwaysOnTop` window.
5. **The tag itself, and the site deploy.** Both are the user's explicit word,
   every time. See the release section above for what must be true first.

⚠️ **Before any live sitting: `displaysleep` is 10 and screensaver `idleTime`
is 300.** That is what silently invalidated the 2026-08-02 attempt seven minutes
in. Hold the display with `caffeinate -d -i -t <seconds>` and re-check the lock
at BOTH ends — `live-probe.sh` only checks in preflight, and `--live-script`
writes successful-looking `LIVE-PROBE shot … ok` lines while presenting zero
frames under a lock.

## Latest design decisions

**Item 116d — 2026-08-02, THE COMPOSITING CALL, and 116d is now UNBLOCKED.**
The comparison sits **ON the workspace surface**, not as a window through it.
The card stays one opaque surface — "A WORKSPACE IS ONE SURFACE" survives
intact — and the relocated document layer draws **after** the card into the
carved region, **without re-drawing its own ground**. The rejected arm was the
one the code itself had already found the defect in: a hole punched in the card
would show the *backdrop's* ground, because the ground punch is at the page
column and not at the region, and on a blur-eligible world `backdrop_blur()`
frosts the frame *around* the workspace — exactly where the region is not. So
"through" would have meant fixing two compositing bugs to reach a worse answer.
**116b's boundary law
`the_relocated_document_is_geometrically_placed_but_not_yet_composited` is now
the thing to delete and replace** with the containment-and-visibility law its
own message asks for. Painter's order gains a second document pass; the region
must be proven to contain it, in every world.

**Items 114 + 116d — 2026-08-02, the workspace Esc, settled once for BOTH
members as item 114 asked.** **One Esc always leaves.** Esc dismisses the
workspace from anywhere inside it; focus moves between the rail/timeline and
the content pane on `Tab`/`Shift-Tab` alone. A child audition summoned *out of*
a workspace (Settings → Theme picker) is a genuinely different rung and keeps
its own Esc-returns-to-parent behaviour. The rejected arm — Esc unwinds one
rung, so leaving History from the comparison takes two presses — was rejected
because the comparison is exactly where a reader spends their time, and Esc
would then mean two different things depending on where focus sits. **116d owes
History an explicit Back affordance in the footer**; 116c's `⇧↵`/`open_keep_version`
groundwork is already shaped for it.

**Item 132 / item 118 — 2026-08-02, the contradiction is RESOLVED in favour of
132.** **Kite is a 5/5.** The roster's target distribution is amended from
`1, 7, 7, 4, 1` (mean 2.85) to **`1, 7, 6, 4, 2` (mean 2.90)** — two deliberate
statement worlds, Firetail and Kite. Item 118's "the gap is the middle, not more
5s" was written before Kite was commissioned and is superseded on that one
clause only; the rest of its direction (calm bias, hover around 3, no
theme-park bell curve) stands. Recorded here because item 118's own audit found
these two user decisions could not both hold and asked for the call before Kite
was built rather than after.

**Item 213 — 2026-08-02:** The user approved the **3 px optical lift** for the
app-icon cursor across the complete world roster and all three cursor shapes.
The canonical assets were already regenerated at `f8d023e1`; item 213 is
complete.

## Latest integration receipt

**2026-08-02:** Item **131** is complete through final slice `3d5cbbbf` and
animation-transaction repair `eac5b4f7`. Item **172b**'s private input-runtime
substates are complete at `328bff0a`. Item **172c**'s `DocumentSession` landed
at `9e52f35e` with whole-slot cache ownership and mutation-proven
A→B→A→B→C→A restoration. Item **172d**'s single redraw door and typed scheduler
poll boundary landed at `85ac839e`. Item **172e** completed the one-owner
`FrameRuntime` at `5dda9de8`; its branch carried exact native, web, and
code-health receipts and is merged on `main`.

**2026-08-01, the current train.** Items **201** (Paperbark's Deckle density on
Retina) and **200** (the caret's punctuation knockback colour) are landed on
`main` in that order. The combined candidate is
`a4f35a43c64a8ea20de3d0e67b35467ce21722e7`, receipt
`native-gate-receipt commit=a4f35a43… conventions=mac,linux scope=all-targets`,
web/wasm 16/16, code health clean. Item 200's own receipt predated item 201, and
its laws measure Paperbark pixels as the Kite analogue, so all 8 were re-run on
the combined tree (8 passed) before the train gate rather than trusting two
receipts taken on different bases. CI is green on `f47028d0` under the raised
mac ceiling.

## CI RED — THE MECHANISM IS NOW KNOWN: the shared wgpu device wedges

**Probe 3 of the bisect left a log, and it is the most informative artifact in
this investigation.** From run `30756807172`:

```
16:56:43  ...split_pane::split_draws_two_surfaces_unified_draws_one ... ok
16:57:43  split_pane::split_shows_ground_across_the_gap_and_no_glyph_escapes  has been running for over 60 seconds
16:57:43  split_pane::split_stays_valid_narrow_and_empty                      has been running for over 60 seconds
16:57:43  stars::currawong_star_field_is_dpi_invariant_in_logical_space       has been running for over 60 seconds
17:46:15  ##[error]The operation was canceled.
```

**Exactly THREE tests — the runner's three vCPUs, i.e. EVERY libtest worker
thread — wedge at the same instant and never move again for 49 minutes.** Left
behind: `Terminate orphan process: awl-69f18644050` and `cargo`, unkillable.

**This is not one bad test. It is the SHARED WGPU DEVICE wedging**, after which
every test that touches it parks forever in `read_pixels`'
`poll(PollType::wait_indefinitely())` — which is also why the process survives
SIGTERM. Control: three `app_icon` tests tripped the same 60-second warning
earlier at 16:47 and **recovered**, so slow-but-alive is normal on this runner
and the final three are categorically different.

⚠️ **The victim varies; the wedge is constant.** Here it is `split_pane` and
`stars`; in run `30732589551` it was `scroll_pos`, at a different point —
**none of them item 194's warp tests.** So the culprit commit **poisons the
device rather than owning the hanging test**, and no amount of staring at the
hung test name will find it. Also: this window's `native-gate.sh` runs
conventions **sequentially**, so this is a single-process hang and the
two-convention concurrency at HEAD is NOT required to produce it.

**The duration spread is explained and is NOT evidence against determinism.**
Probes 1–2 ran 58–59 min because the VM died and GitHub reaped it; probe 3 ran
75 min because the VM SURVIVED and the job ceiling fired — which is exactly why
probe 3 left a log (runner alive → post steps ran → HTTP 200). Same hang,
different killer.

⚠️ **The oracle mis-scored probe 3 as GOOD and would have sent the bisect to
the wrong commit.** `gh` reported `gate step: completed/cancelled`, and a
`status != "completed"` test scored a 64-minute hang as a pass — the same class
as the earlier `conclusion:""` trap, an unfinished step wearing a finished
step's field. Fixed at `c336cc1a`: the test now allow-lists the **conclusion**
(`success`/`failure` GOOD; `cancelled`/`timed_out` BAD; anything unrecognised
is INVALID and scored by hand rather than guessed), re-validated across all six
runs on record. **Two oracle bugs of the same shape in one investigation — a
harness that reads a status field must enumerate what it accepts, never test
for inequality.**

**✅ CONVERGED: the first bad commit is `8207e519`** — "item 194: one camera,
one projected cylinder, cropped at the page". Six probes, every boundary
measured:

| probe | commit | run | verdict |
|---|---|---|---|
| baseline | `7bca59d6` | `30686231377` | GOOD, gate 1014 s, log 200 |
| 1 | `c5b8399e` | `30752286816` | BAD, 3543 s, log 404 |
| 2 | `10cd49e0` | `30754550500` | BAD, 3482 s, log 404 |
| 3 | `d1e997b9` | `30756807172` | BAD, 3853 s, log 200 (ceiling) |
| 4 | `94211bb6` | `30759720562` | BAD, 3242 s, log 404 |
| 5 | **`8207e519`** | `30761792967` | **BAD**, 3423 s, log 404 |
| 6 | `36707d06` (`8207e519^`) | `30763999999` | **GOOD**, 19m03s, step `completed/success` |

**BOTH BOUNDARIES ARE MEASURED, NOT INFERRED.** The parent runs clean in 19
minutes against `8207e519`'s 57 — a real `success` conclusion on the one side
that carries no corroboration. No re-run contradicted a first reading.

**What is SUPPORTED by evidence.** `8207e519` takes `THEMES` from 19 to 20,
adding `KITE` with `Background::WarpedGrid` and **+267 lines of
`background.wgsl`**; 42 test files reference `THEMES`. The tests that wedge are
**roster sweeps** — `split_pane.rs` carries "sweep EVERY shipped world" over
`theme::THEMES.iter()`. All three libtest workers (3 vCPUs) park at the same
instant and **the victim differs between runs**, so the shared device wedges
rather than any specific test. `BackgroundPipeline::new` calls
`create_shader_module` on the whole grown file **plus** `create_render_pipeline`
— and the **test** helpers do this **once per rendered frame** (3 sites) while
the **live app does it exactly once** (`pipeline_draw.rs:32`, stored as a
field).

⚠️ **THE WARPED-GRID SHADER EXECUTES FINE — this is not a non-terminating
shader.** In probe 3's log, 15 `backgrounds_item132`/`warp_tunnel` tests passed
cleanly at 16:51:43–16:51:58, **six minutes before** the wedge at 16:57:43; same
story in `30732589551`. Combined with no unbounded loop in the WGSL and
`warpgrid.rs` touching no wgpu at all, the obvious candidate is ruled out.

**HYPOTHESIS, labelled as such:** cumulative exhaustion of a driver-internal
resource in the virtualised Metal stack — compiled-pipeline slots or
shader-compiler memory — under the added compile churn, after which submits
never retire and every `read_pixels` parks in `poll(wait_indefinitely())`. It
fits the varying victim, the simultaneous three-thread park, and the unkillable
`awl-…`/`cargo` orphans that survive SIGTERM. **The vitals cut only one way:**
`free_bytes` steady at ~2.37 GB rules out RAM exhaustion but says nothing about
a driver-internal table.

**PRODUCT OR CI? The lane's read is CI/test-harness-amplified, NOT a product
defect — with a caveat, and the distinction decides who owns the fix.** The
per-frame pipeline rebuild that supplies the churn exists **only in the test
helpers**. The live app builds `BackgroundPipeline` once at construction and
`prepare()` thereafter merely uploads uniforms *including the shader id*, so
**switching themes does not rebuild the pipeline**. A user selecting Kite on a
VM pays one `background.wgsl` compile per launch and then ordinary draws, and
the shader is shown to execute correctly. **Caveat:** that rests on the churn
hypothesis being right; if the true cause is the WarpedGrid draw accumulating
device state, the product IS exposed on a VM. Lower on the evidence, not
excludable from here.

**Determinism: settled.** Five BADs across five distinct trees, four of them independent
commits, all reproducing the same hang; no re-run has contradicted a first
reading. Probe 6 was the first genuine re-measurement, deliberately on the GOOD
side, and it came back GOOD — so the bisect stands.

🔵 **THE FIX IS NOT WRITTEN AND WANTS ITS OWN ITEM.** Two things a fix-owner
needs and could easily lose in a summary: **the shader is exonerated by
evidence, not argument** (15 `backgrounds_item132`/`warp_tunnel` tests passed
cleanly six minutes before the wedge, in two independent logs — do not start by
staring at the WGSL); and **the churn exists only in the test helpers**, which
is the asymmetry deciding whether this is a product bug at all.

⚠️ **This orchestrator wrote that "the tree the receipts were certifying was
sound" and the bisect owner correctly REFUSED to sign it.** Two reasons, both
better than the reassurance: the CI-only conclusion rests on an unconfirmed
hypothesis, so "not a product regression" is a best read and not an established
fact; and **the receipts have a structural blind spot** — `native-gate.sh` runs
on the dev host's real Apple Silicon Metal and **nothing local exercises a
virtualised GPU**, so their greenness never was evidence about this axis. The
honest statement is "sound on the hardware the receipts run on, with
virtualised-GPU behaviour untested by any local gate". That gap is item **232**.

**Superseded bisect state:****Superseded bisect state:** window `7bca59d6` GOOD .. `d1e997b9` BAD. Real candidates are
just three — `8207e519` (37 files, shader), `94211bb6` (6, shader), `c325fdad`
(4, no shader). `bbb3c2f7` and `d1e997b9` touch no code and cannot be
first-bad. Probes 1–3 all BAD; probe 4 (`94211bb6`) running. **A GOOD reading
at the boundary is the dangerous one** — a BAD is corroborated by duration and
the 404/ceiling, a GOOD is corroborated by nothing but completing — so the
confirmation re-run is owed hardest there.

## CI RED — earlier rounds: the mac gate HANGS; observability landed

**`main` has had no successful CI run since `7bca59d6` (2026-08-01 05:38).**
Only `mac (build + test)` fails; `linux`, `web` and `mac live-probe` pass on
every red run.

⚠️ **TWO diagnoses have now been made and BOTH were wrong. Read that before
proposing a third.** The first was memory starvation — falsified by
measurement (peak RSS 448–667 MB over a 1/3/10 `--test-threads` sweep, wall
time FLAT because `testlock::serial` already serialises the global-touching
tests; and in the real environment, vitals held 2.371–2.386 GB free with
`swap_used_bytes=0` across 34 consecutive heartbeats). The second was
compile-bound, **and it was this orchestrator's error**: `Compiling libc /
proc-macro2 / quote / syn` was read out of the one surviving log and attributed
to step 8, but those lines are in **step 5** — `Install sccache` is a real
`cargo install`. Step 7's `cargo build` took **12 seconds** on a warm cache and
step 8's compile phase is **113 s, 4.6% of the step**. Check timestamps against
step boundaries before attributing a log line to a step.

**What step 8 actually does is HANG.** After `04:45:56` there are 34 heartbeats
and not one other line. The last thing printed is libtest's unterminated prefix
with no result:
`test render::tests::scroll_pos::subpixel_semantics_do_not_change_settled_pixels_or_render_hash ... `
— a test that builds a real wgpu device and reads a frame back, while the
sibling convention does the same on the same **virtualised Metal** device.
**n=1. A lead, not a diagnosis.**

**`a972eafc` ("speed up the full native gate") is VERIFIED and must NOT be
reverted.** Two failures with the identical signature predate it, and on hosted
hardware it is a win rather than a tax: linux gate step, successful runs only,
**n=16 median 1272 s before (1019–2076) vs n=7 median 916 s after (707–937)** —
~28% faster. Its compile cost on the mac runner is the 113 s above. An
orchestrator message proposing to revert it for CI was wrong and was correctly
refused by the lane with this evidence.

**Why six failures produced only one log — four budget defects, all fixed**
(`0f779a81`): the budget armed *after* the canary, so the compile phase had no
watchdog; it killed two **pids** rather than two **process groups**, and a
surviving child holds the step's stdout open, which is exactly why a step
concludes `null` with no uploaded log; it exited ~5 s before its own KILL
escalation; and it timed from gate start while steps 1–7 varied from 96 s to
9m51s across runs, so a gate-relative budget expires at an unpredictable point
on the clock that kills the job. Now: every phase is its own process group
under `set -m`; each convention's output flows through an in-gate filter that
labels every line, stamps phase boundaries, and ignores TERM so it drains the
unterminated line naming the hung test; and the budget takes whichever of a
1500 s gate clock or a job-anchored 2100 s deadline (`AWL_NATIVE_GATE_DEADLINE_EPOCH`)
comes first. Runner losses were at job-minute 53/55/56/62 and the last green
mac job took 26m51s end to end, so job-minute 35 sits 18 min inside the
earliest loss and 8 past the green run. `timeout-minutes` is untouched at 75 —
the runner dies well inside it, so the ceiling was never the mechanism.

⚠️ **THE IN-GATE BUDGET DOES NOT FIRE ON THE REAL RUNNER — proved, not
suspected.** Run `30746762499` wired everything correctly: the "Runner death
clock" step ran at 11:56:18 (deadline 12:31:18) and step 9 started 11:58:23
(gate budget 12:23:23). **The job died at 12:49:18 — 26 minutes after the
earlier of the two should have ended it.** Step 9 `null`, both Post steps
`null`, log 404. **The step never concluded, so the gate never exited, so the
budget never fired**, despite 9/9 runtime mutation proofs locally. Log
availability tracks "did the step conclude", which is why `30732589551` — the
one run whose budget did fire — is the only one that ever produced a log.

Two candidates remain and the board cannot separate them: **(a)** the watchdog
is broken on the runner (starved, killed with the group it should outlive, or
blocked on whatever the suite is blocked on); or **(b)** the whole VM freezes
before the budget and GitHub's timestamp is only when it gave up reaping — in
which case **no in-process watchdog can ever work and the self-abort strategy
is unsound**.

**The discriminator landed** (`timeout-minutes: 40` on the mac gate STEP,
enforced by the runner agent rather than by our shell; job-level 75 and the
in-gate budget both kept): a log from the step timeout means the watchdog is
broken; nothing at all means the VM freezes and we stop trying to self-abort.
**Unproven until a real mac run — if the VM is frozen, a step timeout is as
dead as our watchdog.**

**The heartbeat now separates deadlock from livelock**: `load1`, `cpu_count`,
per-tracked-process CPU and the busiest pid, plus CPU time beside elapsed in
the abort report. It is a **delta of `ps -o time=`, not `pcpu`** — pcpu is a
lifetime average on Linux and a decayed one on macOS, and the CI shape (3.5 min
hot, then 35 min hung) reads ~9% on one and ~0% on the other. Unparseable
prints `unavailable`; zero tracked processes prints `none`; never a confident
`0.0`. ⚠️ **Its Linux branches are written and reasoned but NEVER EXECUTED** —
no Linux host here; CI's linux job is their first real test.

**Two more laws that were vacuous when written, both caught by mutating the
REAL script rather than the synthetic self-test** — the running tally for this
CI work is now five: the audit pinned `load1=`/`cpu_count=` names that the
*abort report* also carries, so the heartbeat's copy could be deleted or
commented out entirely with the audit clean; and the probe itself reported
`tracked_procs=0` while both suites burned a core, because Cargo had moved to
integration binaries younger than the sample window — indistinguishable from an
idle machine, and exactly the confident-zero class that shipped once already.

**The user's standing decision (2026-08-02): coverage is NOT cut.** Both
conventions stay. If the suite genuinely cannot fit the runner, `main` stays
red and we say so — narrowing the mac job's scope is not an acceptable fix.

**Next, in order:** read the next mac failure, which will now name the phase,
convention, target and hung test; add a per-test watchdog so one hung test
fails by name instead of taking the runner down; and only as a last resort
bisect `7bca59d6..edc89757` (~50 commits, including two vectorized-search
commits confined to `src/search/mod.rs` + `src/buffer.rs`) — each mac run costs
~50 minutes and nothing reproduces locally, where the suite passes in 182 s.

### The superseded diagnosis, kept because the correction is the lesson### The superseded diagnosis, kept because the correction is the lesson

**`main` has had no successful CI run since `7bca59d6` (2026-08-01 05:38).**
130 commits have landed since. Eight consecutive runs are `failure` or
`cancelled`, and the previous orchestrator's train notes claim green from local
receipts alone — no one checked the remote, which is exactly the check
`.orchestrator/README.md` §Gates makes mandatory before AND after a push.

**The failure is one job and one cause.** `linux`, `web` and `mac live-probe`
pass on every red run. Only `mac (build + test)` fails, always in step 8
(`scripts/native-gate.sh`), always with a `null` step conclusion and the
GitHub annotation *"The hosted runner lost communication with the server.
Anything in your workflow that terminates the runner process, starves it for
CPU/Memory, or blocks its network access can cause this error."* Confirmed
identical on runs `30727349406` (04bef696, 55m), `30721529191` (9111aed4, 56m)
and `30715372469` (eea3118a, 62m). Job logs 404 — the runner dies before
uploading them, which is itself evidence of a hard kill rather than a test
failure.

**This is NOT the timeout class item 196 diagnosed.** `timeout-minutes` is 75
and all three deaths happened at 55–62 minutes, well inside it. Raising the
ceiling again will not help; the runner is being starved, not guillotined.
`scripts/native-gate.sh` sets no parallelism or memory bound at all — no
`--test-threads`, no `CARGO_BUILD_JOBS` — so on a hosted macOS VM it runs the
whole GPU/glyphon suite at host-adaptive width under both conventions.

~~Claimed — claude (deep), branch `claude/ci-red-mac-runner`.~~ Landed; the
worktree is removed. **The standing lesson: a local green train says nothing
about the remote, and this board carried "CI is green" for 130 commits on the
strength of local receipts alone.** Check `gh run list --branch main` before
AND after every push, and check the last *successful* sha rather than the last
run.

## Ready — current user-visible wave

## Active claims — 2026-08-02/03 wave

**Overnight results, newest first.**

- **220 + 234** — ✅ COMPLETE. **221 + 224 argued and DEFERRED to item 235.**
  Receipt `native-gate-receipt commit=c2cfac75… conventions=mac,linux
  scope=all-targets`, 3627 passed.
  **The theme-data constraint HOLDS — it is not what blocks 221/224.** Both
  expressions are a `RenderCaps` variant over 220's single `overlay_location`
  datum, the same shape `Background`/`CardTexture`/`FacetStyle`/
  `TitleStyle::Placard` already have. **A capability gap blocks them:** glyphon
  0.11 has **no transform, no rotation, no skew anywhere in the crate**, and the
  tree's only rotation turns rounded-rect QUADS, not glyphs. See item **235** —
  world-neutral, and both become data on top of it.
  ⚠️ **FIVE BRIEF PREMISES WERE WRONG.** **220 is universal, not the Files
  filter's problem:** every non-All lens of every faceting picker groups into
  exactly ONE section whose label is character-for-character the lens's own —
  6 pickers, 22 lenses, 20 worlds; that line was never a section header
  anywhere. **234 is not a right-edge defect:** the row box was the bare band,
  so text sat `BAR_SIDE_INSET` (8.00 px, measured) **outside its plate at BOTH
  edges**, and the LEFT half — the first glyph of every row label cut by its
  plate — was the more widespread symptom **nobody had reported**. **224's
  premise does not hold at all:** after 220, Magpie's cue is already on the
  LEFT, riding the diagonal cluster's own row stagger; there is no right-side
  indicator to mirror, only the slant and gradient are missing. **No second
  header line was needed** — the location inherits the section header's existing
  planned slot, which is why all 1200 sidecars are byte-identical. And **a
  latent parallel calculation surfaced:** the timeline column measured its
  footer by shaping the raw string in one face while the drawn footer is
  symbol-split, under-measuring by 1.3 px on Mopoke; two unspent pads hid it.
  **Captures per-surface, as this class requires:** 840 PNGs byte-identical
  across 14 surfaces, 360 differing across exactly the six the two items touch,
  **all 1200 sidecars byte-identical**.
  ⚠️ **The gate caught TWO defects in the lane's OWN laws that filtered runs had
  shown green:** a roster that included two plate-less worlds, and an oracle
  that scavenged "ink near the plate" — which **cannot attribute a pixel to a
  glyph**. The rewrite compares the emitter's own quads against the column edge
  and upgraded the sweep from one row of one lens to every `SettingId` of every
  category × three widths × every plated world. Follow-up: item **236**.
- **222 + 223** — 🟡 IN PROGRESS — claude (production), branch
  `claude/items-222-223-mangrove`. Both briefed with the lesson 219/225 just
  taught: **assume universal until measured otherwise.** For 223 especially,
  "Mangrove omits shortcuts other themes show" may mean the SHARED owner drops
  them under a condition Mangrove happens to meet — and if the labels are
  ELIDED rather than omitted, that is a rowlayout budget question, not a missing
  feature. 222 is warned off pre-empting item 131e, which still owns selection
  composition on the diagonal machinery.

- **231** — 🔴 STILL OPEN. **The fix did NOT clear the hang, and the probes
  FALSIFIED the recorded diagnosis.** Receipt for the landed improvement:
  `native-gate-receipt commit=3e3db0c6… conventions=mac,linux scope=all-targets`.
  **Four discriminating arms over one grafted workflow** (run `30766842071`):
  control **WEDGED**; `RUST_TEST_THREADS=1` **WEDGED** — so **it does not need
  concurrency**, which kills the "three workers race" reading; `render::tests::`
  **WEDGED**; `--skip render::tests::` **COMPLETED**, 2860 tests per convention
  in 110 s *while standing up its own device per test and building ~80,000 GPU
  programs in aggregate*. **So it is not how many programs a process builds — it
  is how many against ONE long-lived device.**
  ⚠️ **THE "SHARED WGPU DEVICE WEDGES" DIAGNOSIS IS DEAD.** In the post-fix log
  the mac and linux conventions — **two separate processes with two separate
  wgpu devices** — stopped **within 10 MILLISECONDS** of each other and never
  moved. They share no device. **The contended resource is SYSTEM-WIDE: the VM's
  virtualised Metal stack itself.** That also explains why the no-render arm
  survives while doing far more total GPU work — its tests create AND DESTROY
  devices, forcing driver-side reclamation, where `render::tests::` piles
  transient resources onto one device never torn down.
  **The landed work still earns its place** (`src/gpu_cache.rs`): `render::tests::`
  GPU program builds **52,083 → 5,577**, `TextPipeline::new` 44 ms → 23 ms,
  `cargo test --bin awl` 133.8 s → 116.6 s, 3616 passing either side. Only
  objects that are pure code are shared; everything writable stays per-instance.
  **A 9.3× cut in churn did not clear it** (run `30770296246`).
  ⚠️ **A DANGEROUS wgpu FACT, measured (29.0.3): `wgpu::Device`'s `PartialEq`
  reports two separately requested, simultaneously live devices as EQUAL.** A
  device-keyed cache is therefore impossible — the first draft trusted it and
  648/3616 tests died with `BindGroupLayout does not exist`. Identity is a
  property of the CALL SHAPE instead. The cache also **must not be
  thread-local**: libtest gives every test its own thread, which left builds at
  86,061, i.e. no change at all.
  **One law initially PASSED its own leak mutation** — drawing one world at a
  time lets each `prepare` overwrite the last; only building and preparing all
  twenty BEFORE any draws exposes it.
  **Where to look next, per the lane:** the residual ~5,577 builds are dominated
  by the direct `BackgroundPipeline`/`SelectionPipeline` helpers (~1,800 calls,
  55 call sites) — but more promising given the system-wide finding, the
  per-call `glyphon::Cache` + `TextAtlas` and every `offscreen()` texture and
  readback buffer, which are **allocations, not programs**, and which wgpu only
  reclaims on poll. Probe driver at `~/.awl-item231/probe.sh`.
  **Item 232's virtualised-GPU arm would have caught all of this locally in
  minutes rather than 50 per cycle** — that item is now the higher-value one.
  ⚠️ **The merge shipped two wasm warnings the branch could not have seen**
  (`OnceLock` native-only, `scoped` whose caller is `None` on wasm); fixed at
  `3e3db0c6`, and an over-gate of `Mutex` broke the wasm build in between —
  the wasm `programs()` is an `unreachable!` stub that still names it.

**⚠️ THE GITHUB REPO HAS BEEN RENAMED to `Frank-P-Lu/awl-editor`.** `git remote`
still says `awl-next.git` and every push tonight succeeded through GitHub's
redirect, but tooling that reconstructs URLs by hand will point at the old name
— `scripts/ci-mac-bisect.sh` already does.
- **219 + 225** — ✅ COMPLETE. Receipt `native-gate-receipt commit=d7709def…
  conventions=mac,linux scope=all-targets`.
  ⚠️ **NEITHER DEFECT WAS WORLD-SPECIFIC, and that inverts what both items
  asked for — a per-world nudge would have left FIFTEEN WORLDS BROKEN.**
  **219 is on all twenty worlds**, byte for byte identically: the query beat was
  folded into the last header line's box, and cosmic-text **centres** a line's
  glyph run in its box, so on the flat family — whose one header line IS the
  query field — the glyphs drew `header_gap/2` low. Measured at 1200×800: query
  ink centres at **98.0** where its own line box centres at **77.6** — a
  **20.4 px blank strip at the top of every takeover picker**.
  Mopoke/Currawong/Gumtree/Bilby/Bowerbird were simply the five clicked through.
  **225 is on all five bare-plate worlds:** the `Bars` footer plate ran to the
  card's bottom edge — right for a card that hugs its content, wrong for a
  workspace whose card comes from the canvas. **Cassowary's plate ink is
  `base_100` `#050506`, which is why it reads BLACK there; on Galah it is pale
  pink and invisible.** 52 px of plate for 19 px of ink.
  **Fixes, one sentence each:** the query field's box is always exactly one
  line, `plan::beat_stands_alone` owning whether the beat rides the last header
  line's glyph metrics or closes the band as its own glyph-free line —
  `split_bounds` collapsed from two arms to one and the seam hangs from the
  field's own bottom edge, byte-identical arithmetic on the grouped family; and
  the workspace footer takes its own band via `overlay_footer_reclaim`, the
  existing owner the card height already reads. No overlay rectangle, no
  per-world anything, rowlayout untouched.
  **A law was DELETED rather than repaired**, per item 217's precedent: item
  174's `the_pre_plan_query_band_genuinely_missed_the_field` reconstructed the
  RETIRED formula, which after 219 is the CORRECT one on both families, so it
  could no longer fail on anything. 174's user-visible outcome is still held by
  its own headline law, which passed unmodified.
  **A constraint in the brief was unsatisfiable:** "other worlds must remain
  byte-identical" cannot hold for a correct fix here. What IS byte-identical is
  every other **surface** — 840 probes, **840/840 sidecars identical**, 480/840
  PNGs identical, all 360 differing the same deliberate class (the six
  one-header-line surfaces × 20 worlds × 3 canvases). **State the constraint
  per-surface in future briefs of this class.** Follow-up: item **234**.
- **233** — ✅ COMPLETE. Receipt `native-gate-receipt commit=e5d520d2…
  conventions=mac,linux scope=all-targets`. `SerialGuard` now snapshots
  `render::overrides::pins()` on entry and restores on exit beside
  world/page/spellcheck — **including on the UNWINDING path**, which is the part
  a reset at the end of a test body cannot buy. A dirty exit that did not panic
  still fails by name, listing every knob that changed.
  ⚠️ **THE BRIEF'S PREMISE WAS WRONG and the lane explained the leak anyway.**
  `list_surfaces.rs:909` is NOT the only site with that value: its helper
  `bars(6.0, 8.0, 24.0)` expands to exactly the longhand in
  `chrome_overlay.rs`'s `list_style_override_reader_writer_are_serialized`,
  **whose worker thread exits its `serial()` window without resetting** and
  whose main thread released `outer` still holding `Some(Pane)`. The forced
  value survived both windows and the next thread through the mutex read it.
  **A grep for the literal struct misses the helper call and vice versa** —
  which is exactly why it read as a single site. Fixed at the source too, and
  the new guard found it on its FIRST run, the only failure in a 3619-test
  parallel sweep.
  **An ELEVENTH forced knob was found:** the living-band motion probe lived in
  `livingband.rs` with **no write assert at all** — the one genuinely unpoliced
  override, invisible to an item that named ten. Now under the same snapshot,
  its writer asserting the hold, with an alias keeping its nine call sites
  untouched. `leaked_knobs` destructures both sides exhaustively, so a new
  `RenderOverrides` field must be named there or the build breaks.
  **The proof was built on the axis that matters:** 3619 tests green at
  `--test-threads=16` and again at 24. Under the mutation the headline test's
  first assertion **passed spuriously** because the sibling law had already
  poisoned it — the defect reproducing inside its own proof.
- **230** — ✅ COMPLETE. Receipt `native-gate-receipt commit=a37d741f…
  conventions=mac,linux scope=all-targets`. `ViewState::substitute_text` is the
  ONE door that replaces shaped text, recording the document *and the caret's
  place in it*; both seams route through it and `TextPipeline::figure_source()`
  is the single seam saying which text the figures are over. `doc_source: None`
  on every ordinary frame is byte-identical.
  ⚠️ **THE BRIEF'S CENTRAL SPLIT WAS WRONG.** I said fixing `WORD COUNT` alone
  was a complete outcome with `THROUGH DOC` left as a possible product call.
  **That option did not structurally exist:** `DocFigures::of` derives all three
  figures from ONE text and ONE caret, and item 215 gathered them precisely so a
  caller cannot take one from the owner and another by hand. **215's gather is
  load-bearing on WHAT QUESTIONS CAN BE ASKED SEPARATELY, not merely on how they
  are filled** — worth remembering when briefing anything near it. `THROUGH DOC`
  derives from the CARET, not the scroll position, so it is not a scrollbar; the
  old value jumped when the reader folded a section the caret was not in.
  **Three figures were affected, not two** — LANGUAGE vanished under a History
  preview, a transcript carrying no frontmatter — **and the blast radius reached
  past the card**: the sidecar's `readout` block and `wordcount_text`, the calm
  corner notice's feeder, were wrong in the same two states. **`hud_report()`
  was ALREADY mixing owners** in exactly the way 215 forbids, agreeing only
  because both paths happened to read the shaped text. One deliberate contract
  break: `hud.lang` no longer mirrors top-level `doc_lang` — the latter is the
  SHAPED text's language, which the per-script font ladder must follow.
  🔵 **Parked for the user:** should the card gain a separately-named
  `THROUGH VIEW` figure? That is the only shape in which "how far through what I
  can see" can exist without reintroducing the drift. Cheap to add, needs no
  rework. **Known gap, named:** the live `sync_view` preview substitution has no
  behavioural law — `hermetic()` has no GPU so `sync_view` returns early — and
  its guard is the source-scan roster law, which mutation proof showed does
  fire. Closing it properly needs a `--screenshot-app`-driven harness.
  ⚠️ **The merge hit the hazard `CLAUDE.md` names:** item 204 slice 2 added
  `preview_view` to `OverlayInfo`, 230 added a fixture constructing one, **git
  merged both cleanly and the tree did not compile.** Filled with `Some("diff")`
  rather than the `None` its siblings use, because a History preview IS a
  comparison and the real path emits `request.map(|r| r.view.tag())` — `None`
  would contradict the `preview_text` the fixture sets. Six further mark hunks
  reconciled by MEASURING: `render.rs` 2477, `pipeline_draw.rs` 547,
  `pipeline_geometry.rs` 607, each **+1 over both branches**.

- **204** — ✅ COMPLETE, both slices. Receipt `native-gate-receipt commit=8d5565c5…
  conventions=mac,linux scope=all-targets`, 3637 passed per convention.
  `comparison::prose_for` is the ONE dispatch — History's producer where the
  store is, the conflict's beside the two texts it reads, both consumers routed
  through it, no second renderer and no parallel cache. `--seed-data` closes the
  harness gap slice 1 measured: the affordance and all three previews are
  reachable at capture tier 2, proved on the real binary **both ways round**,
  with a canary that a bait record in the real `XDG_DATA_HOME` is neither read
  nor written. Schema `/197` adds `gutter.changed` and `overlay.preview_view`;
  **a `notice` sidecar field was deliberately NOT added** — a single transient
  slot cleared by any toast expiry is the wrong thing to build a state oracle
  on, which is the very gap this slice replaces. ⚠️ **A real defect from the
  roster audit:** the wheel-region predicate asked
  `o.kind == OverlayKind::History` — a fact about one SURFACE rather than the
  SHAPE both share — so wheeling over a conflict's manuscript flipped its three
  rows instead of scrolling the prose. All five hand-pinned "exactly N"
  assertions were decided consciously and **two were made STRONGER**, spelling
  their members out by name instead of counting, so a fourth must still be
  argued. **Vision smoke called the affordance "subtle" and arithmetic overruled
  it** — a clean three-step luminance ladder, 29.7 / 110.7 / 163.3 / 251: the
  Wagtail tripwire running the other way, where the eye was wrong and the pixels
  decided. **Premise corrections:** THREE hand-kept arrays omitted `Date`, not
  two, all now derived from `OverlayKind::ALL`; and Esc takes TWO presses from a
  palette-launched workspace, because the first returns to the parked palette —
  the standing grammar every palette-opened picker has. Follow-up: item **233**.

- **217** — ✅ COMPLETE. `--bench-suite` runs green again. **BOTH of the brief's
  premises were wrong, and they pointed at two different plans.** (a) The second
  plan was NOT the diagonal's: `prepare_overlay` rebuilt the plan
  UNCONDITIONALLY after `resolve_diagonal_cluster`, so **Saltpan — the default
  world, upright, with no cluster at all — paid a full second plan every overlay
  frame.** That is what the bench was hitting, and it is eliminated: the frame
  builds ONE plan and completes its measured half in place through the same
  `apply_row_extent` a fresh build runs. (b) There IS a genuine second plan and
  it belongs to **item 51's right-anchored content-hug measurement**, which
  shapes a provisional card to learn how wide to hug — Cassowary (right-anchored,
  upright) shows 2, Magpie (left-anchored, diagonal) shows 1. **THE AXIS NOBODY
  HAD SWEPT WAS THE CARD ANCHOR, NOT THE LIST STYLE.** So it is *named* rather
  than numbered: `FramePasses` names each pass and asserts the sum, a third plan
  fails by name, and its oracle is the measurement's own cached hug width rather
  than `overlay_right_anchored()`, so it cannot agree with the code by
  construction. Two witnesses were pointed at the wrong surface — the palette's
  row witness read `overlay_rows`, the flat/poster band quad a diagonal world
  deliberately never fills, and the theme scenario restored the posture world
  instead of `DEFAULT_THEME`. Because `--bench-suite` is outside the native
  gate, `render/tests/plan_pass_law.rs` re-runs the same witness under
  `cargo test` across the roster. **An honest negative, kept out of the tree:** a
  first device-tier completion law compared the opening frame's pixels to the
  settled frame's and **stayed green under the no-op mutation**, because on a
  diagonal world the drawn ink is placed by `cluster.label_left(...)` and not
  `row.dx` — the plan's extent drives hit-test and clip, not the glyphs. Deleted
  rather than shipped vacuous. Two marks moved, **neither raised**:
  `chrome/overlay.rs` lowered 738→735, and a `too_many_lines` exception lowered
  110→109 after extracting `pin_posture`.
- **216** — ✅ COMPLETE. The mark-audit skip is closed; see the CI section above.
- **211** — ✅ Presence confirmed on a real screen; see the blocked section for
  the three arms still owed.

- **207** — ✅ COMPLETE. Merged to `main`; worktree removed. Native awl has one
  semantic UI owner: `SemanticSnapshot` feeds the AccessKit tree, `--semantic-json`
  and the live-App sidecar from one description. **The orchestrator's trap call
  held** — card text was being composed inside the renderer, so it moved to
  `src/card/content.rs` (`CardInputs` → `open_card` → `CardContent::spans`) and
  `hud.rs` went 539 → 489 lines composing no card text of its own, under a source
  law. ⚠️ **The branch had NEVER been gated:** code health was red — two files
  over the 500-line ceiling, nine stale Clippy exceptions, four failing
  native-gate census laws — repaired by decomposition rather than line-golf.
  **Two real stage-1 defects the new laws found and inspection had not:** the
  overlay query node advertised `SemanticAction::Focus` with nothing routing it
  (every arm now returns `handled`), and `schema` was `&'static str`, which serde
  can only fill from a `&'static` input — **the JSON handed to an agent could not
  be parsed back at all.** Two of the lane's own laws were vacuous and mutation
  proof is what found them. 55 new crates, all permissive; `cargo deny check bans`
  needed four narrow skips for the AT-SPI stack's forked `syn`/`toml_edit`/
  `toml_datetime`/`winnow`, each with a removal condition. 🔵 **Real VoiceOver and
  AT-SPI journeys were NOT run and are NOT claimed** — everything is proven at
  the snapshot and projection tier; whether a screen reader *reads it well*
  (announcement order, verbosity, live-region politeness) is unproven and needs a
  human at an unlocked display. Web accessibility remains a separate DOM-backed
  round. Follow-up: item **215**.
- **174** — 🟢 SECOND FAMILY LANDED, item remains OPEN. Merged to `main`;
  worktree removed. `PlannedHeader` owns the overlay header band, with the query
  beat folded into the LAST header line's box exactly as the shaper folds it into
  that line's glyph metrics. **Deleted from `render/chrome`, not banned by law:**
  `overlay_secondary_top`, `overlay_split_bounds`, `overlay_strip_band`,
  `overlay_query_center` — the previous slice's standard, met again. ⚠️ **It
  found a shipping pointer defect of exactly the predicted class:** `over_overlay_query`
  tested the bare row pitch, but on the FLAT family the beat inflates the query
  line itself, so at the shipping default the field draws `[64.0, 133.2]` while
  the pointer band ended at `91.2` — **the I-beam sat in empty air ABOVE the
  query text and the text itself took the plain arrow.** 13 of 19 `OverlayKind`s,
  Settings-as-workspace included; worst case 33.0px at 2×. The GROUPED family was
  right *by accident* (its beat inflates the lens strip instead), which is how a
  parallel calculation survives review — it agrees on the arm somebody looked at.
  Identity 840/840 PNG and 840/840 sidecar, zero differing; the cursor icon is
  the only changed output. **Left explicitly:** `workspace_header_beat` did not
  merge — its consumer is reached ~45× a frame through the four relocated
  document owners, so planning inside it would trade one parallel calculation for
  45 plans a frame; a law fails by name if they drift. Follow-up: item **217**.
- **211** — 🟢 DIAGNOSED AND FIXED; one live confirmation still OWED. Merged to
  `main`; worktree removed. **It was never a lost input.** `advance(dt)` runs
  BEFORE `Gpu::redraw()`, and `prepare` — inside that call — is where the
  selection band is retargeted. The band is the ONE animator whose target is set
  at draw time; every other spring is armed at the apply seam where the next
  `advance` sees it. So on the frame a settled band is retargeted the pre-prepare
  answer is "nothing animating", the loop parks on `Wait` and requests no
  follow-up frame; the ease never gets a second frame, the band stays drawn on
  the row the selection LEFT, and the next input's single `dt` puts it back in
  flight so `chase_or_snap` takes its SNAP branch to the freshest row — two
  inputs, one jump of two rows, no transition. **Shipping on the default path:**
  `awl_living_band()` defaults to `Morph`; `arm_live_juice`'s "no world ships
  one" was true of `MotionJuice` and false of the living band, corrected in
  place. **Why items 104 and 106 stayed green through three sightings:** their
  laws hand-drive `p.advance(dt)` between retargets, which is precisely what the
  live loop was failing to do. Trace: 7 inputs, indices 1..6 exactly +1 each,
  none doubled or lost, `hover_took_selection` 0. 🔵 **OWED, and it needs the
  user:** the display auto-locked at 12:55:42 JST seven minutes into the sitting,
  so all 10 presents read `Occluded` — the diagnosis is CPU-side and
  occlusion-independent, but **no frame was photographed, there is no 60 fps
  video, and the sweep arms (held-repeat, pointer parked above/on/below, scrolled
  and fresh windows, focus/occlusion return, Settings and the other picker kinds,
  Bars worlds, 1×/2×) were never reached.** ⚠️ **Two harness facts worth more
  than the item:** `--live-script` forces a Prohibited, non-activating window, so
  under any lock it writes successful-looking `LIVE-PROBE shot … ok` lines while
  presenting ZERO frames — a probe run can look like it worked and have
  photographed nothing; and `live-probe.sh` tests the lock only in preflight,
  never at the end, so it would have passed at 12:52 three minutes before the
  lock landed.
- **116d** — 🟢 COMPOSITING ROUND LANDED; the flip is deliberately NOT done.
  Merged to `main`; worktree removed. **The owner stopped at a clean boundary
  and that is the correct outcome** — the comparison can now be SEEN, and
  `workspace_shape(History)` is still `None`, which is now safe to change.
  `draw_document_layers` splits into `draw_document_ground` (background, lava,
  stars, page frame — the quiet frame, never relocated) and
  `draw_document_content`; on a comparison frame the content is submitted AFTER
  `draw_overlay_card` into the carved region without re-drawing its ground, and
  the ordinary frame concatenates the two in their original order so **every
  non-comparison frame is byte-identical by construction**. The blur path now
  captures the ground alone while a comparison is up (116b's frosted-ghost
  defect), and `blur_signature` hashes the comparison flag — otherwise Settings'
  workspace and History's sign identically at one scroll and keep the wrong
  frost. `clip_text_bounds` is the glyph twin of item 84's quad clip. The old
  boundary law is deleted as its own message asked and replaced by four
  containment-and-visibility laws. **The Esc decision landed for BOTH members**;
  `Shift-Tab` was not wired at all and had to be added, and `GUIDE.md` plus
  `site/guide.html` both promised "Esc there is a *back* to the rail" and were
  fixed. Captures 1320 files: 1080 byte-identical, 240 differing — all the
  `settings-detail` probes, one sidecar field, `esc back` → `tab back`.
  ⚠️ **The vision smoke found a defect the whole green suite missed:** the
  five-cell workspace footer ran off the card on Firetail at 900×520. The
  pre-existing no-clip law measured the FLAT card at one canvas over three
  worlds and was structurally blind to the workspace; the missing law is now
  written over the whole roster at four canvases and both stages.
  **Premise corrections worth keeping:** 116c's `⇧↵`/`open_keep_version` is the
  *restore* and the *keep* prompt, NOT groundwork for the Back affordance — the
  orchestrator's brief said otherwise and was wrong; and History's Back was
  already half-present (`foot_hint`'s `detail_focus` branch said `tab back`) —
  the debt was **Settings'**, whose line said `esc back`, the reverse of how the
  brief framed it. `workspace_header_beat` was deliberately left unfolded: it is
  a fourth copy of a ONE-LINE header today, and the moment the lens moves to the
  header `header_rows` becomes 2 and that copy becomes **wrong** rather than
  merely duplicated — so folding it belongs inside the lens-to-header slice.
  **The restore notice is CALLED but not implemented:** restore emits one calm
  notice naming the version and the undo (`restored "2 hr ago" · ⌘Z to undo`);
  Esc emits none, because it undoes a view substitution and a toast confirming a
  no-op is the nagging DESIGN's calm bias forbids. It belongs with the restore
  journey the flip owns.
  **Left for the next slice:** flip History to `TimelineOverComparison`; move the
  lens to the header (inside `PlannedHeader`, folding `workspace_header_beat` in
  as part of it); reuse the ordinary candidate-row hit-test for the timeline;
  deep-link `Version history…` and `Compare with version…`; implement the restore
  notice; and the tier-2 hermetic work. ⚠️ **One harness fact for that slice:**
  `--keys` captures reach History's timeline but **not** its comparison — a
  headless capture has no history store, so `selected_history_id()` is `None`
  and the focus transfer declines. The comparison's capture-tier probes need
  `--screenshot-app` or a seeded store.

## Remaining work — handoff order (2026-08-02, after the evening wave)

1. **116d — dispatch first; it is UNBLOCKED and everything behind it waits.**
   116a–c are landed and both of its owed decisions are now made: the comparison
   sits ON the workspace surface, and one Esc always leaves. Do the compositing
   round FIRST — delete `the_relocated_document_is_geometrically_placed_but_not_yet_composited`
   and replace it with the containment-and-visibility law its own message asks
   for — then flip `workspace_shape(History)` to `TimelineOverComparison`, add
   the footer Back affordance, move the lens to the header, deep-link
   `Version history…` / `Compare with version…`, and run the split Verify sweep
   (tier 1 replays `overlay_accept:History`; tier 2 owns the store, git, the
   pruned ladder, renamed timelines, `KeepVersion` and the restore's disk read).
   116a's handoff still applies: reuse the ordinary candidate-row hit-test for
   the timeline rather than extending the rail functions — `geom.rail` is `None`
   whenever rows are primary. ⚠️ **It writes `render/chrome/overlay_draw.rs`,
   `overlay_rows.rs` and `chrome/mod.rs`, which item 174's next family also
   writes — do not run the two concurrently.**
2. **Then 204.** Unblocked the moment 116d's composited general read-only payload
   exists. Preserve the one editable buffer; add disk fingerprinting (mtime plus
   length cannot detect the required same-time/same-size rewrite), the recovery
   record, three read-only conflict views, gated-action resolution, and align
   Guide/welcome/site prose.
3. **174's next family**, whenever it is not racing 116d. `workspace_header_beat`
   is the named remainder but is its own slice — its consumer is reached ~45× a
   frame through the four relocated document owners.
4. **Follow-ups queued by this wave:** item **218** first (the newly live
   VoiceOver path can stall while typing), then **217** (`--bench-suite`'s
   plan-count witness vs the diagonal re-plan; not urgent, and must not be fixed
   by weakening the witness) and **215** (extract word count / language / percent
   into pure owners so a live-App capture carries card semantics).
5. **Human/live closures, all needing an unlocked and FOREGROUNDED display:**
   118's world-loudness confirmation and its `--release` ambient sitting; 211's
   one unoccluded confirmation that the fixed build presents the glide, plus its
   unreached sweep arms; and item 207's real VoiceOver / AT-SPI journeys, which
   no test tier can stand in for. ⚠️ **The machine's idle lock fired seven
   minutes into the 2026-08-02 sitting and silently invalidated it** — disable
   the idle lock before the next one, and re-check the lock at BOTH ends of the
   run, because `live-probe.sh` only checks it in preflight.

Items 131, 172, 207 and 213 are complete; 174 has two families landed and stays
open. Older historical prose below is retained but the receipts above are
authoritative.

After each landed item: update this board, exact combined-main code-health +
web + native receipt, push `main`, remove only the completed worktree, and run
`scripts/sweep.sh 1`. Tags/releases and site deployment still require explicit
user authorization.

116. **Move Version History into the shared summoned workspace — timeline and prose diff become one readable task, never three competing layers.** **Build:** Preserve the existing history store, git backend, pruning, facets, descriptions, kept versions, and prose-diff engine, but replace the current History overlay/diff-as-preview composition with item 114’s workspace. On wide windows, show a narrow timeline/navigation pane beside a large read-only comparison pane; moving through versions updates the comparison immediately. On narrow windows, show the timeline first and enter the comparison as a second stage with an explicit return path. The current editor is backdrop/state, not a third readable layer. **Scope:** Keep local loose-file snapshots and git-managed history behind the same UI with only a quiet source label. Preserve independent diff scrolling and a clear focus transfer between timeline and comparison. `Esc` leaves the current buffer byte-for-byte unchanged; restore must be a deliberate, footer-taught action rather than bare `Enter`, and remains undoable. `Version history…` and `Compare with version…` deep-link into the same workspace at the appropriate focus; `Keep version…` retains its brief naming prompt and returns coherently. Remove the old History overlay, document-under-card preview composition, and feature-specific diff-panel dressing only when their last consumer is gone—retain the generic prose-diff machinery and do not strand parallel disabled paths. **Done:** A user can answer “which version, what changed, and do I want it back?” without overlapping titles, hidden prose, or ambiguity about whether the editor is active; the same flow works for local and git history; exiting is a true no-op and restoring is deliberate and undoable. **Verify:** Timeline→live comparison→focus/scroll→back/exit/restore journeys for local snapshots, named/pinned versions, git commits, empty history, renamed files, and pruned ladders; narrow/wide/zoom/DPI captures across representative light/dark, Pane/Bars, and Wagtail worlds; pixel laws proving timeline and comparison never overlap and the original document does not remain a competing readable layer; restore undo law; capture/replay parity; dashboard vision smoke; native, both conventions, and wasm gates. **Depends on the completed item 114 Settings workspace (landed `60477e7c`); user design decision 2026-07-26.** ⚠️ **DECOMPOSED 2026-07-31 after inventory — the owner stopped at a clean boundary rather than half-land it, which is the correct outcome and the brief's stated escape hatch.** It began the content-model change, then reverted deliberately: the first edit flips History's shell predicate on, and committing that without the content is precisely the empty workspace item 114 forbids. **This is not "big like 114" — it is four independent 114-sized changes, three of which were invisible from the item text.** **(1) The comparison has no renderer.** awl has exactly one prose renderer — the document layer. The transcript is markdown from `prosediff::render_markdown_blocks`, so relocating it into the content pane means giving `column_left()`, `column_width()`, `doc_top()` and `doc_clip_band()` — the four owners every document consumer routes through, ~45 call sites across `rects.rs`, `layers.rs`, `text.rs`, `geometry.rs`, `scroll.rs` — a viewport override, then gating every margin-orientation surface composed off them. Item 114 added a third geometry family beside two others and never touched the document layer; this is a second structural change of the same size in the most load-bearing geometry in the tree. A second prose renderer inside the overlay pane is the "infrastructure complexity is a smell" CLAUDE.md forbids. **(2) The removal is wider than the build,** and the item's hedge resolves toward caution: the diff-panel dressing's last consumer really is History (`scripts/review.sh` sets `opts.diff` but never `opts.preview_text`, the only thing lighting `vstate.diff_panel`), so `diff_panel`, `diff_panel_rect`, `prepare_diff_panel` and three pipelines all go — but item 84's `doc_clip_band` must **survive and be re-owned** by the comparison viewport, and its law files re-aimed rather than deleted. **(3) ~60 History test functions across ~25 files assert the CARD presentation** — each a judgment call, not a rename. **(4) Restore needs a new input primitive:** `CompareVersion`/`KeepVersion` have no default chord, and "deliberate, footer-taught, not bare Enter" cannot be a named chord because `HintAction.glyph` is `&'static str` while a chord glyph is convention-dependent — so the footer-honest option is a shift-held accept delegating to `Newline` in the editor. **Two premise corrections.** `workspace_shell()` as a **bool is insufficient**: 114's shell puts facet labels in the rail and rows in the pane, but History wants the timeline as the primary list, so flipping the bool yields the wrong composition. It must become a shape — and **DESIGN.md §5 already sanctions exactly this** ("categories beside controls, or a timeline beside a comparison"), so it is a reading 114 deferred, not an invention. Separately, **two "Done" clauses are already true and cost nothing**: Esc leaves the buffer byte-identical (the transcript is a view substitution, never a buffer write) and restore is already undoable (`App::restore_history` goes through one atomic `Buffer::set_text`). Neither emits a notice, which is worth deciding — a silent document replacement is the one place a toast earns its keep. **THE DECOMPOSITION, in dependency order.** **116a — the shape:** `workspace_shell()` becomes `workspace_shape() -> Option<WorkspaceShape>` (`RailOverRows` | `TimelineOverComparison`) with `rows_are_primary()` as the single fact geometry/keyboard/hints reduce to; History still returns `None`, so nothing is presented. Tier 1, fully capturable, lands green and changes no pixel. **116b — the relocated document viewport:** `comparison_viewport` as the one owner, read by the four geometry owners; margin surfaces gated; `diff_panel` and its pipelines structurally removed; the clip/wash/panel laws re-aimed. **This is the risky half and deserves to fail alone.** ✅ **116c LANDED — merge `7ea5cd78`** (`f3da0d07`, `1254a7b9`). `Action::AcceptAlternate` (Shift+Enter) resolved directly in `KeymapState::resolve_named`, needing no catalog chord because **Shift reads identically on both conventions** — proved by a Mac×Linux × native×emacs sweep rather than asserted, plus a law confirming it is absent from the Linux keep-list. **The delegation is the literal same code path, not a copy:** `apply_buffer_action`'s arm is now `Action::Newline | Action::AcceptAlternate`, and byte-identity is proved over `Buffer::disk_bytes()` across **ten** smart-newline shapes — bullet/numbered/task continuation, the empty-item provenance flag across four mixed step-orders, blockquote continue/end, bare-indent carry, non-markdown bypass, active-selection override — not the plain-prose case anyone would have thought to check. `history_intercept` folded into `workspace_intercept`, routed through 116a's `rows_are_primary()` rather than a kind branch; `overlay_nav.rs`'s mark **lowered** to 768 as it went. `KeepVersion` now descends through `overlay::Journey` rather than entering over a card. **Honest about reachability:** the descend branch is not reachable through today's live dispatch (the palette closes itself first), so it was proved by a direct unit test "rather than a fictional re-dispatch". ⚠️ **Merge-train note:** the lane reported "`code-health.py`: clean" — the python arm alone, not `code-health.sh` with its clippy pass — and the candidate failed `clippy::type_complexity` on its new fixture tuple. Third lane this run whose branch-level health claim did not survive the train, and the same defect class the run has been about: a check whose stated scope exceeds what it ran. Fixed with a `type Fixture` alias. **116d inherits:** the intercept is ready for `TimelineOverComparison`, `⇧↵` and `open_keep_version` are ready for a real in-workspace hint, and `workspace_shape(History)` is still `None`, waiting on 116b's compositing question. an alternate-accept action delegating to `Newline` in the editor with a byte-identity law, plus shape-aware intercept. **116d — the flip and the journeys:** History becomes `TimelineOverComparison`, deep links, the lens moved to the header, and the full Verify sweep. **Verification split, written against `docs/harness-reach.md`:** tier 1 covers entry, focus transfer, Back, exit, parked-parent position, timeline selection, lens cycling, staging, zoom/DPI and every pixel law — `overlay_accept:History` is Applied, so the restore journey replays. Tier 2 covers anything touching the store or git: snapshot recording, the pruned ladder, renamed-file timelines, `KeepVersion` (Unsupported) and the restore's disk read. **The item's Verify clause reads as though the whole thing were capturable; it is not, and asking for a sidecar over `KeepVersion` would repeat item 180's mistake.** **Owed a human, and it compounds item 114's open question:** from the comparison the first `Esc` is a Back to the timeline, so leaving History from the content region takes two — the same interaction decision 114 flagged, and it should be settled once for both members before 116d lands. ✅ **116a LANDED — merge `6202205c`** (`bff81da9`): `workspace_shape() -> Option<WorkspaceShape>` with `rows_are_primary()` as the one fact, `TimelineOverComparison` defined but routed nowhere, Settings byte-identical (identical PNG sha256; the only sidecar delta was `project.dirty` from the stash procedure). Its mutation broke **three pre-existing item-114 laws**, proving the re-route is real rather than inert, and a grep-law bans matching the shape enum outside its defining file. **Handoff worth keeping:** the geometry seam is already in place, so 116b's comparison viewport just reads the content rect; 116d's timeline hit-test should reuse the ordinary candidate-row hit-test rather than extend the rail functions, since `geom.rail` is `None` whenever rows are primary; and **`chrome/workspace.rs` is at 497/500 with no mark escape** — it postdates the frozen baseline, so 116b must extract a submodule before adding to it. ✅ **116b LANDED — merge `350aed68`** (`80527ae0`). `TextPipeline::comparison_viewport()` is the one owner; `column_left`/`column_width`/`doc_top`/`doc_clip_band` read it and everything downstream follows without knowing. Extracting `workspace_regions()` first took `chrome/workspace.rs` from 497 to **479**, off the ceiling 116a warned about. The bypass is named and enumerated in `render/geometry/page.rs` with a law pinning its consumers to that file plus exactly two fallback arms. **98 captures byte-identical**, PNG and sidecar, verified three times. `diff_panel` and its three pipelines removed after the owner verified the last-consumer reading itself; item 84's `doc_clip_band` **survived and was re-owned**, its laws re-aimed rather than deleted, and its X arm is genuinely exercised for the first time. **Mutation-proofing found two fixtures that would have gone quiet instead of red** — one searched for its straddling canvas by asking the very clip under test whether it had trimmed anything, the other graded a band its transcript never inked; both now derive independently with non-vacuity floors. **No mark raised; seven lowered.** ⚠️ **The boundary it stopped at, pinned as a law rather than absorbed:** the relocation moved the document's geometry but **not its place in painter's order**, so the workspace card still draws over it — opaque hides it, translucent ghosts it, and a blur-eligible world frosts the document into the frame around the region. `the_relocated_document_is_geometrically_placed_but_not_yet_composited` asserts both halves over the whole roster and its own message tells 116d to delete and replace it. **116d CANNOT flip `workspace_shape(History)` before that compositing round — it would present an invisible comparison.** The open design question is whether the comparison sits *on* the workspace surface or is a window *through* it.

131. **Give Mangrove and Magpie mirrored diagonal-line compositions across contextual menus and the real Settings workspace.** **Build:** Add one reusable theme-owned diagonal row composition through the shared rowlayout/surface machinery, then assign its two authored orientations: **Mangrove** draws a continuous descending `\` spine, with row clusters left-aligned on the RIGHT side; **Magpie** draws a continuous ascending `/` spine, with row clusters right-aligned on the LEFT side. The line is mandatory in both—the striking read comes from the drawn division and triangular negative space, not merely staggered text. It may visually bleed toward the surface corners, while row attachment points occupy an inset middle band so the first/last rows retain usable width. **Line treatment:** never amber/primary. Mangrove uses a crisp tidal-teal line derived from its muted ink; Magpie uses a crisp graphite line from its muted ink. Resting weight is clearly visible but subordinate to text; the selected row brightens and thickens only the local spine segment toward `base_content`, extends a short connector to the row, and steps the row outward by a few crisp pixels—no spring, pulse, or full-width selection bar. Existing bottom-left Mangrove stipple and bottom-right Magpie ghost placards occupy the opposite empty triangle rather than colliding with the rows. **Controls are in scope, not a fallback:** model each row as a measured `label + fixed gap + accessory/control` cluster. Reserve consistent label/accessory extents across the visible set so shortcuts, values, toggles, checkmarks, exact-entry fields, and Range sliders trace a stable parallel rail with honest spacing; anchor the whole cluster to the spine instead of independently nudging text. Query/title/category navigation/footer regions remain horizontal and stable—the diagonal owns the candidate/setting rows, not every glyph on the surface. Filtering and scrolling sample a fixed surface-relative line at fixed row y positions, so content changes never make the spine or surviving rows jump horizontally. **Surface reach:** enroll every contextual overlay’s row section that currently consumes theme list-style data, not Commands alone; non-row panels keep their existing geometry. After item 115 removes the old Settings overlay, apply the SAME diagonal owner to the Settings workspace’s main setting list for Mangrove/Magpie, while its category rail, search/title shell, child Theme/Caret auditions, and narrow-stage navigation remain item 115’s workspace behavior. The empty main-pane triangle may carry the existing active category/title typography, but never a duplicate decorative label. **Responsive bound:** size the slope from the real widest visible cluster and available side territory; widen within the existing surface limit first, then reduce horizontal travel while preserving a visibly diagonal direction. Never overlap, clip, shrink type/controls, introduce horizontal scrolling, or silently fall back to Pane/Bars. **Scope:** This is a third data-driven row composition, not Mangrove/Magpie branches and not permission to diagonalize the document, workspace shell, History comparison, native Mac menu, or web/Linux persistent menu bar. Other worlds and their Pane/Bars results remain byte-identical. Item 112 owns the shared overlay rhythm first; items 114/115 own the workspace and Settings migration first. **Done:** Mangrove reads like a tidal instrument panel and Magpie like an editorial spread in both brief menus and sustained Settings, with a visible line, deliberate negative space, readable controls, obvious selection, and no loss of keyboard/pointer usability at any supported width. **Verify:** Full no-wildcard `OverlayKind` row-surface sweep plus every `SettingId × SettingKind`; simple/long labels, chords, values, toggles, text entry, sliders, empty/short/full/filtered/scrolled lists, category changes, child-picker return, and narrow/wide staging; drawn line/row/control ↔ hit-test agreement at zoom and 1×/2× DPI; pixel laws for orientation, line continuity, inset attachment band, fixed label-control gap, local selected segment, placard/row non-overlap, non-primary ink, and no clipping; exact before/after identity for every non-assigned world; dashboard captures and affordance-locating vision smoke over Commands plus every Settings category in both worlds; native, both conventions, and wasm gates. **Depends on 112 (landed, `fa64a3a4`) + 114 (pending) — **there is no item 115**; it was folded into 114 by `d726c4bb`, so the references to "item 115" in this item's body mean 114. The one real blocker is 114, which LANDED `60477e7c`. Ambitious user design decision 2026-07-27.** ⚠️ **DECOMPOSED 2026-08-01 into 131a–e after its seam turned out to be missing — but the seam fix itself LANDED and closed a live defect.** ✅ **SEAM LANDED — merge `dbf33714`** (`6df426a5`). **The defect it found, which was already shipping:** the renderer could stagger overlay rows — the offset lived in the draw emitters — but `OverlayRowPlan::row_at` kept testing the card's **undisplaced** x-span. A staggered row was clickable across a strip where nothing was drawn, and the deeper the row the wider the lie. Draw and pointer answered "where is this row" separately, which DESIGN.md §8 forbids, and every law item 131 wants to write is a claim about where a row *is*. `PlannedRow::dx` is now the one owner, planned in `plan_overlay_rows` exactly as `top` is planned from `lh`; the text area, bar plate, Pane band, selected bar and `row_at` all read it. `dx == 0.0` for every shipping world, proved byte-identical across **19 worlds × 5 surfaces = 95 captures**, hashing PNG *and* sidecar, 95/95. A source law bans any third file from re-deriving row x. **Four premise corrections.** (a) The mirror needs a **signed two-sided extent** — Mangrove's `\` steps rows right (left edge moves), Magpie's `/` with right-aligned clusters steps left (right edge moves); one `dx` cannot express both, and generalising speculatively would have been the dormant path the item forbids. (b) **The spine has no primitive** — every overlay quad pipeline is axis-aligned — but `caret.wgsl` already carries a rotation axis, so the cheap route is an inert `axis` on the selection instance, not a new pipeline class. (c) **Neither world authors its placard corner**; both are `PlacardCorner::Auto`, derived from their anchors, so the "opposite empty triangle" requirement needs the derivation to learn about the spine — a change the item does not budget for. (d) **Item 186's registry is keyed on `Background` variants and a row composition is chrome, not a ground**, so there is no slot — and the real finding is that **overlay chrome already mixes both spaces**: row pitch scales with DPI while `BAR_SIDE_INSET`, the text hpad and `CARD_MAX_W` are raw device px. A diagonal pitch authored like its neighbours would be **physical by inheritance**, exactly what item 186 exists to stop. Making it logical would make it the first chrome quantity to declare its space, which either extends `ground_space` past `Background` or opens a sibling registry — **a design decision owed a human eye, not a line of code.** **Good news it found:** the Settings workspace comes free — `workspace_geometry` builds an ordinary `OverlayGeom` and the row planner reads its band, so one owner reaches contextual menus and the workspace through one path. `workspace_shape`/`workspace_geometry` were not touched, so item 116a's lane stayed clear. **Decomposition:** **131a** the two-sided span (`dx` → `[left, right]`, small now the seam exists); **131b** the spine primitive (inert `axis` on `SelectionPipeline` + a rotated rounded-rect emitter, byte-identical for all 15 existing consumers); **131c** the composition owner (`ListStyle::Diagonal`, both worlds in one commit since the item forbids a half-applied world, **including the logical/physical decision in (d)**); **131d** the measured cluster rail (where the `SettingId × SettingKind` sweep bites); **131e** selection and the full Verify clause. 131a and 131b are small enough to pair; 131c–e should not be attempted in one pass. ✅ **131a+131b LANDED — merge `dbf33714`..`f4340960`** (`307308d2`, `31d9a7b4`). `PlannedRow` gained `dw` (a width delta) beside `dx`, chosen over `[left_inset, right_inset]` because every consumer already manipulates `(x, width)` pairs; one signed `dx_per_row` still drives both mirrors, split by sign in the planner alone. `SelInstance`/`selection.wgsl` gained an inert `axis` mirroring `caret.wgsl`, plus `prepare_rotated` and the pure `spine_segment`/`narrowed_spine_corner_px` helpers — **no non-test consumer**, `src/theme/` zero diff, no world touched. Byte-identity proved across **190 files** (95 captures × PNG + sidecar). **A pre-existing defect on the SHIPPED DEFAULT path was found and half-fixed:** `overlay_pane_selection`'s living-band branch (`Choreo::Morph`, the default — not an opt-in probe) called `living_band_rects` and drew the result verbatim, **never reading any row's offset**; only the sibling non-living branch read `dx`. A real drawn/hit-test disagreement, invisible only because nothing had ever planned a nonzero offset. The single-shape case is fixed; **`Choreo::TwoShape`'s echo band can represent a different row mid-glide, and whose offset it inherits is a composition question left explicitly to 131e.** The owner also flagged, unprompted, that its rotation law is single-function parametric rather than cross-owner — "flagging that distinction rather than overstating it". Merge-train note: the lane's branch passed health carrying unformatted code that happened to fit `selection.rs`'s 768 mark; rustfmt on the candidate tripped it at 771, and the spine helpers were **extracted** to `selection/spine.rs` (mark down to 731) rather than raised. 131c is BLOCKED on the user's chrome pixel-space decision in finding (d). 131d–e unclaimed.**

172. **Decompose the 107-field `App` into owned state domains with narrow transition APIs.** **Defect:** Physical file splits have kept individual implementation areas navigable, but more than twenty modules still extend `impl App` and can reach the whole live application state. Invariants for documents, input, workspaces, rendering, scheduling, and persistence therefore remain coupled by convention. **Build:** After item 171 establishes the effect boundary, migrate fields and their invariants incrementally into explicit owners: `DocumentSession`, `InputState`, `WorkspaceState`, `RenderRuntime`, `FrameScheduler`, and `PersistenceRuntime` (names may change if the ownership map proves a better cut). Each owner exposes domain transitions rather than public fields; cross-domain work travels through typed outcomes/effects, not back-references to `App`. Preserve the active-buffer whole-slot ownership law, fake-clock determinism, wasm gating, GPU recovery, and byte-identical behavior. Do not introduce a service locator, trait-per-method architecture, message bus, or flag-day rewrite; land coherent vertical slices with compile-time removal of the old field access. **Done:** `App` is lifecycle composition rather than the mutable home of every subsystem, and new workspace or persistence behavior has one obvious owner. **Verify:** First commit an ownership map and call-site census; add structural gates against direct cross-domain field access and growth of root `App`; run focused state/cache identity laws after each slice, then both conventions, full native, wasm, and the live scheduling/GPU probes. **Depends on 171 (landed). Routing:** deep owner plus independent ownership audit. **User-requested code-health work 2026-07-29.** 🟢 **TWO SLICES LANDED — merge `4821c63a`** (`73db7850` map/census, `ec94c743` WorkspaceState, `4399ebf0` PersistenceRuntime, `d56e8bb7` ratchets). **The item remains OPEN**: four domains are mapped and gated but not extracted.

**The ownership map is a deliverable in its own right** — `docs/app-domains.md`, with the same table as executable data in `src/app/tests/domains.rs`. 1,310 production and 586 test references classified, exhaustive by construction (no field in two owners, none unassigned). Remaining, with production references: `DocumentSession` 4 fields/363 refs, `RenderRuntime` 25/277 (**held for item 174**), `InputState` 27/164, `FrameScheduler` 12/123, `ProjectLocation` 9/51, and 22 host/lifecycle fields that stay on `App`. `config` (88 refs, 16 files, one writer) is the best remaining single-field slice.

**The map's own argument against the obvious next cut, worth keeping:** `gpu`'s 160 references across 23 files — the most dispersed field in the struct — are dominated by `gpu.window.request_redraw()`, which is a *scheduling verb wearing a render field*. Taking `RenderRuntime` first would pull 22 files into item 174's blast radius for no invariant. Relatedly, the item's `RenderRuntime`/`FrameScheduler` boundary is **not a real boundary**: for `theme_font_at`, `theme_switch_at`, `theme_settle`, `crossing_settle_at` the *stamp* is scheduling and the *effect* is rendering. The map assigns them to `RenderRuntime` and says so rather than pretending the line is clean; item 174 should move them and record the decision in the classification gate.

**Also worth knowing: `InputState` is the item's largest named owner and its lowest-value one.** 14 of its 27 fields are touched by exactly one file and `app/input/` accounts for 135 of 164 references; extracting it satisfies the item's letter and buys a struct rename. Its only genuine cross-domain leak is `cursor_px`, read at two sites in `apply.rs` — cheaper to pass the position than to move 27 fields. The map argues it should probably never be extracted as a 27-field struct.

⚠️ **THE ITEM NAMED TWO DIFFERENT DOMAINS `WorkspaceState`, and they share no fields.** This item lists it beside `DocumentSession`/`PersistenceRuntime` and closes on "new **workspace** or **persistence** behavior", which reads as the project-folder domain; item 173 says "in item 172's `WorkspaceState`, define one closed lifecycle for editor, brief contextual overlay, sustained summoned workspace, and suspended child audition", which is the summoned-UI domain. **The name went to 173's meaning** — 173 is the downstream consumer and the critical path to 114 — and the project-folder domain is now `ProjectLocation`, with `App::workspace` renamed `workspace_root` so the two cannot be misread for each other. **This item's Done clause is therefore satisfied for persistence and NOT YET for project-folder behavior.** Recorded rather than quietly counted as met.

**The one place byte-identity was consciously chosen over consistency** is `sync_cursor_icon`'s raw `popover_summon_bit()` read: documented, single call site, and law-counted so a second consumer fails by name. A byte-identity refactor preserves pre-existing bugs, so if a second pair of eyes is spent anywhere on this branch, that is the spot.

174. **Separate pure render planning from shaping/cache mechanics and GPU execution.** **Defect:** `TextPipeline` and the render directory jointly own scene policy, document geometry, cache invalidation, hit-test inputs, sidecar-visible facts, GPU resources, and feature-specific drawing. Tests often have to infer planned geometry from pixels, while render-touching work can accidentally couple presentation rules to device state. **Build:** Introduce one deterministic scene/layout planner that consumes `ViewState`, measured text inputs, theme capabilities, and viewport data and emits inspectable primitives plus interaction geometry. Shaping and cache ownership remain a distinct measured stage; GPU execution consumes the plan without deciding feature layout. Route drawing, hit-testing, and sidecar geometry through the same planned objects, migrating one coherent surface family at a time. Preserve O(visible) frame work, buffer-identity cache keys, rowlayout ownership, deterministic capture, and exact output for migrated surfaces. Do not build a retained widget tree, general scene framework, duplicate CPU renderer, or allocate an entire document plan each frame. **Done:** presentation decisions are testable without a device; GPU code executes rather than invents layout; drawn and interactive geometry cannot drift through parallel calculations. **Verify:** Plan-level geometry laws, drawn↔hit-test↔sidecar identity, buffer-swap/resize/zoom invalidation, allocation and reshape-count witnesses, exact before/after capture probes across representative worlds and surfaces, release frame benchmarks, both conventions, full native, wasm/WebGL. Every render slice gets the standing vision smoke. **Independent of 171–173; schedule away from item 114’s overlapping render files. Routing:** deep owner (`gpt-5.6-sol` high) with production-tier outcome audit. **User-requested code-health work 2026-07-29.** 🟢 **FIRST FAMILY LANDED — branch `claude/item-174-render-plan`** (`a10944a8`, `5d97e140`, `3ef6d85d`, `15e3eede`, `a1674d1e`, `629eb937`). **The item remains OPEN**: one surface family is migrated, the rest still own their geometry.

**`src/render/plan/` is device-free** — shapes nothing, measures nothing, reads no clock — and `plan_overlay_rows` emits one `PlannedRow` per candidate display line plus the interaction geometry. **The forward `row -> y` arithmetic and its inverse are module-private, and `overlay_row_top`/`overlay_row_of`/`overlay_row_index` are DELETED from `render/chrome`: the bypass is compiler-enforced, not grep-enforced.** Routed through the one plan: hit-test, range rails (draw and hit), the visual-selection band target and coverage grid, Pane band, Bars plates, chord plates, footer plate, slant clip bands, shaped-row item mapping, and `overlay_window_report`. Item 164's arrangement is preserved and re-pointed, not weakened — the planner is now the sole owner of `selected_display()` and `overlay_row_at` deliberately stays outside the transaction.

**Why this family, from the census rather than the brief:** `overlay_row_top` was a free function taking five loose scalars, re-assembled at **eight** call sites, and `overlay_draw.rs` then abandoned it and re-derived `first_top + k*lh` inline. "How many candidate lines does this card have" was written out **five** times, and the fifth added `+ empty.is_some()` in only one arm — a live defect. "Which line is logically selected" had item 164's owner **and a second, differently-clamped copy inside the test y-probe**, i.e. the oracle itself was a parallel calculation.

**Identity: 470 capture probes** (19 worlds × 10 overlay surfaces, plus 8 geometry axes over 7 worlds). **34 PNGs differ and every one is the deliberate fix; zero other PNG or sidecar bytes changed**, re-verified three times, with after-vs-after 0/470.

**The deliberate output change is a real defect the migration exposed.** `content_rows` omitted the empty-state notice line the card height had already paid for, so a `Bars` world's picker filtered to zero matches drew its **footer plate over the "no matches" row**. The 34 differing cells are exactly the two zero-match states on the five shipping Bars worlds plus geometry variants — measured on Galah as 19,295 px in one band `y 242..300`, one row height, nothing else.

**Two things mutation proof found that reading did not.** (a) `livingband::covered_rows` was stepping its own `first_top + k*lh` grid one module over from the plate with the same bug; the first source law's sentinels could not see a consumer *stepping off the band origin*, and adding that sentinel lit it up immediately. (b) **The device law's `sel_row` arm was tautological** — sidecar and plan both read `selected_display()`, so a planner that forgets grouped headers kept them in perfect agreement while pointing at the wrong row, and it was *watched staying green*. An independent oracle (the reported line must carry `overlay_selected`'s item) made it fail by name. This is the strongest instance yet of the repo's own lesson: agreement between two readers of one function proves nothing.

**Witnesses:** `plans_per_frame = 1` and `plan_rows_per_plan = 12` for a **106-item** palette — one `Vec<PlannedRow>`, 384 bytes per overlay frame, with the bench asserting *exactly* one plan (zero means measuring nothing, more than one means a consumer grew its own). O(visible) proven at 200,000 items. Reshape counts unchanged.

⚠️ **Two measurement honesty notes.** Release frames showed palette cells at median +8.1%, but the **untouched** cells moved median −0.2% across a −7.0%…+22.5% range on the same run — the whole S tier moved on every scenario including ones this change cannot touch, with five workers building concurrently. 0.17 ms is implausible for 384 bytes plus a ≤24-row scan, so the honest reading is *no palette-specific signal, confirmation owed on a quiet host*. Relatedly the **bench baseline was deliberately not re-banked**: re-banking tonight would freeze contention noise into every cell.

**Left for later slices, stated rather than implied:** document-content surfaces, search panel, HUD, gutter, outline, popover, whichkey and readout still own their geometry; the spell popup's anchoring is untouched (its rows are planned, its anchor is not); `overlay_secondary_top`/`overlay_split_bounds`/`overlay_strip_band` remain separate owners and folding the strip band plus secondary column in is the natural next slice; **no sidecar schema change** — publishing planned row rects would let a test assert row geometry with no device at all, but that is a schema bump plus a CAPTURE.md edit.

**Two premise corrections worth keeping.** (a) The brief said overlay rows "already have `render/rowlayout` as a partial owner" — `rowlayout` owns *horizontal* cell layout (primary/secondary budget, elision, rails) and had nothing to do with the vertical row-Y duplication where all the drift lived. They are orthogonal owners and the planner correctly did not subsume it. (b) The theme picker **retired its lens strip** (user decision 2026-07-15) and is now flat, so the grouped/sections path is exercised by the **command palette** under a non-All lens. A brief reaching for the theme picker to test grouped behaviour would have tested a flat card.

**Follow-up routed:** item 181.

118. **Pre-release world-loudness audit — repeat the 1–5 idle Room/Frame check before release.** **Audit definition:** “idle loudness” is how strongly a world asks for attention while the user is simply writing in page mode: palette, typography, margin pattern, and ambient motion count; summoned overlays do not. `1/5` is the quiet pole (Wagtail), `3/5` is recognizable/alive but comfortable for hours, and `5/5` is a deliberately rare statement world (Firetail). **Baseline from the 2026-07-26 design session, eighteen shipping worlds:** `1/5: 1`, `2/5: 10`, `3/5: 2`, `4/5: 4`, `5/5: 1` (mean 2.67). Bowerbird item 117 intentionally changes that to `1, 9, 3, 4, 1` (mean 2.72). **Direction for a twenty-world roster:** hover around 3 with awl’s calm bias; the provisional healthy shape is `1, 7, 7, 4, 1` (mean 2.85), not a symmetric theme-park bell curve. The current gap is the middle, not more 5s. This is a diagnostic distribution, never permission to turn up a world merely to fill a bin—each world still earns its own identity. Bilby/Brolga/Tawny/Saltpan are valuable quiet anchors; Galah and Mulga are candidates to inspect, not pre-decided promotions. **Run:** Immediately before release preparation, review every shipping world’s current Room in item 20’s dashboard at representative page widths, then observe every ambient world live in `--release`; independently assign 1–5 before looking at the baseline, reconcile the roster together, and record any chosen changes as separate concrete queue items. Include any nineteenth/twentieth worlds that have graduated by then; do not graduate Cassowary Light or any other candidate merely to hit twenty. **Done:** The final roster has a user-confirmed loudness map, its mean/distribution and outliers are explicit, near-duplicate intensity poles are named, and every proposed rebalance is either rejected on purpose or queued with a world-specific reason. **Verify:** Affordance-locating vision smoke over all Room captures; live-only confirmation of speed/calmness for Lava, Currawong stars, Bombora waves, Bowerbird cutouts, and any later ambient world. Pixel/sidecar arithmetic may prove territory and contrast but never claims the taste score. **Timed before release preparation; user design decision 2026-07-26. ✅ **PREPARATION LANDED — merge on `main`** (scripts only; no Rust, shader, Cargo or CI file touched, so **no full-native-suite receipt is claimed** — code-health and web-smoke clean). 🔵 **STILL AWAITING THE USER: the confirmed map, and the live `--release` sitting.** **The lane scored all nineteen worlds before opening anything baseline-related, and that search became the finding: the 2026-07-26 per-world map was NEVER WRITTEN DOWN.** Only the histogram survives — `queue.md` and its history, ROADMAP, THEMES, PHILOSOPHY and the design-session commits were all searched. **Independent map: `1, 10, 3, 4, 1`, mean 2.68.** Excluding Paperbark (which post-dates the baseline): `1, 10, 2, 4, 1`, **mean 2.67 — the baseline's exact aggregate**, arrived at blind. Corroboration: both stated anchors match (Wagtail 1, Firetail 5); the four 4/5s recoverable by inference from item 117 are **exactly the four it independently chose**; and **Bowerbird scored 3**, the first independent confirmation that item 191's `Finds` swap plus tuning hit its target rather than overshooting. **One unresolvable disagreement:** the baseline's second 3/5 is one of Currawong/Gumtree/Magpie/Mopoke and no record says which. ⚠️ **A CONTRADICTION BETWEEN TWO USER DECISIONS, found here and material to the pending Kite call:** item 132 commissions Kite as a **second 5/5**, while item 118's own target shape `1,7,7,4,1` carries **one** 5 and states *"the gap is the middle, not more 5s"*. Both cannot hold — either the shape becomes `1,7,6,4,2` (mean 2.90) or Kite is not a 5. **This needs answering as part of the Kite decision, not after it.** ⚠️ **The live `--release` observation was NOT run and was explicitly not claimed: the screen is LOCKED** (`CGSSessionScreenIsLocked: true`, checked at both ends of the session with the same probe `live-probe.sh` uses). Under the occlusion tripwire a live window presents zero frames, so a sitting now would produce a false result — item 113's unlocked session no longer holds. **Every ambient world's score is provisional on that check.** Offered instead, honestly bounded: deterministic phase trajectories converted to real seconds via the product's own `LAVA_SPEED` — fraction of right margin moved past 3 L\* at ten seconds is **Mangrove 0.351 · Firetail 0.344 · Bowerbird 0.077 · Bombora 0.027 · Currawong 0.002**, proving trajectory only, never cadence or calmness. **Premise corrections:** item 20's dashboard has **no width parameter** (fixed 1600×1000), so "representative page widths" required a new sweep; **item 132 calls Kite the nineteenth world when Paperbark already is** (Kite would be twentieth); and **in a code buffer the map does not describe what ships** — at `page_width_code = 100` a 1600px window leaves a 16px margin, the ground effectively vanishes and the roster's spread collapses toward palette alone. **Two metric repairs recorded because they nearly produced false findings:** linear luminance called every dark world's ground flat while the captures plainly showed shapes (gamma, not a finding) — CIE L\* is now the headline column; and `ink_cr` took a *percentile of the column*, whose area changes with the measure while the ink does not, so it moved between arms for one palette (Paperbark 10.75 vs 4.42) until a fixed extreme-pixel count made it arm-invariant. **Vision smoke 19/19 on every affordance except inline code, which is not locatable in Wagtail** — pixel-checked as byte-identical background, which is the 1-bit law's own sanctioned answer (THEMES.md) and therefore declared behaviour rather than a defect, though the affordance genuinely does not exist there. **Near-duplicate poles, named as the item asked:** Tawny/Mopoke (tightest — same `Dots{edge:false}`, edge 0.0000 both, L\* σ within 0.15), Magpie/Saltpan (edge **0.4444 on both to four decimals**), Bilby/Brolga (deliberate mirror per THEMES.md), and **Firetail/Mangrove inverted** — Mangrove measures louder on every static and motion column while ranking a step lower. **Six proposals, all labelled as proposals:** Galah's ground density (item 108's Gumtree precedent; the cheapest 2→3), re-verifying item 108 actually met its Done condition (Gumtree measures second-faintest at its shipped density), rejecting Mulga's promotion on purpose and recording why, resolving the Firetail/Mangrove inversion, making ROADMAP's "merge the tightest near-pair" call on the now-named pair, and **recording the confirmed map as durable data so the next run diffs instead of re-deriving four scores by inference.** Captures in `gallery/worlds/`, `gallery/item-118-loudness/` and `gallery/item-118-ambient/`. Dispatched now because the roster is momentarily STABLE at nineteen worlds — Bowerbird's `Finds` and item 186's ground space both landed, and Kite is held off main — so a distribution taken now describes what actually ships. Only the preparation is dispatchable: the item's Done requires a USER-CONFIRMED map and states that pixel arithmetic may prove territory and contrast but never claims the taste score. The lane produces captures, a vision smoke, and its own independent 1–5 proposal; the confirmation and any rebalance remain the user's.**

## Parked — explicit gate or future design

- **Export save-dialog scope:** macOS + Linux, one live-only cross-platform seam; capture uses an explicit path. Decided, not scheduled.
- **Per-world living-band choreography:** audition TwoShape/Slam/Soft against Morph; live feel is the oracle. Needs a design session.
- **Per-world copy-pulse differentiation:** possible future motion tweak; needs a design session.
- **Site deployment:** only on the user’s explicit word.

## Monitoring — non-blocking

- **Hands-on checks still useful:** writer-diff panel/Tab + zoom readout; heading-chevron mouse-press→toggle wiring; theme-picker felt input→present lag; Bombora drift speed / counter-motion / calmness over real seconds.
- **GPU memory:** no action unless the 6 GB symptom recurs; then probe the live surface with the window foregrounded.

## Release blockers and reminders

- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user’s explicit word. A dry run may precede them.
204. **Resolve external file changes without silent overwrite or a second document model.** **Defect:** awl checks disk metadata only while flushing a dirty buffer, then holds autosave behind a sticky notice whose promised “reopen for theirs” path does not exist; clean buffers never refresh, manual Save force-overwrites the external version, and session restore records no held content. **Build:** extend the existing clobber guard at focus return, buffer activation, and every persistence/identity boundary. A clean buffer reloads from disk with cursor/scroll preserved. When both sides changed, keep the awl buffer as the sole editable document, stop writes to the original path, and atomically maintain one recovery record under awl’s data root using the scratch-stash pattern. Show a persistent `changed elsewhere` affordance beside the filename. Its conflict overlay reuses History’s preview substitution to show, one at a time, **Differences**, **Your version**, and **Version on disk**; previews are read-only and never replace the buffer. Esc returns to editing unresolved; Save, switch, rename/move, Finish File, and Quit route back through resolution. Explicit **Save your version** rechecks disk before writing; **Use disk version** becomes one undoable replacement. No filesystem watcher service, second editable buffer, side-by-side renderer, auto-merge, duplicate user file, or general recovery/versioning system. Update `GUIDE.md`, the welcome document, and any generated site copy in the same change: none may keep promising that manual Save force-overwrites or advertise a conflict action that does not exist. **Done:** neither version can be silently lost, unresolved state survives a crash/relaunch, and every user-facing statement of the save/conflict model matches the shipped paths. **Verify:** pure content/stat truth tables including same-mtime/same-length changes, deletion and repeated external writes; tier-2 hermetic `App` sweeps for every gated action and relaunch recovery; `--screenshot-app` state/pixel captures for indicator, three previews, Esc return and both resolutions; token/source laws pinning the Guide, welcome, and generated site to the live save vocabulary; new-surface roster audit plus vision smoke; mutation proofs restore the false reopen path, forced Save clobber, missing recovery write, and obsolete Guide promise. **Routing:** production owner (`gpt-5.6-terra` medium) with deep independent review for the data-loss/state-ownership seam. **User design decision 2026-08-01.** 🔵 **BLOCKED ON 116d — premise audit 2026-08-01.** History's geometry seam exists, but its comparison is deliberately not composited and `History` still has no workspace shape; the only current preview substitution is a selected-history-row cache and cannot carry typed ours/disk/diff payloads. Building 204 now would require the prohibited one-off renderer. Resume after History's timeline/comparison workspace owns compositing and a general read-only payload. The implementation must also add a byte fingerprint or retained baseline, because mtime plus length cannot detect the required same-time/same-size rewrite.

207. **Give native awl one semantic UI owner feeding both AccessKit and agent JSON.** **Defect:** the custom wgpu UI has no platform accessibility tree, so VoiceOver/AT-SPI cannot read the document, caret, selection or summoned surfaces; meanwhile the capture sidecar describes internal/render state but not roles, accessible names, focus ownership or valid actions. Building those as two parallel descriptions would guarantee drift. **Build:** introduce a renderer-independent, serializable `SemanticSnapshot` derived from the live `Buffer`, active surface/Journey, search and settings state. Stable semantic IDs, roles, names, values, selection, focus, relationships and supported actions live here once. Add two adapters: native `accesskit_winit` tree updates and `awl --semantic-json`; embed the same snapshot in live-app capture sidecars without replacing their geometry/pixel or replay fields. Route AccessKit action requests back through the winit event loop and existing `Action`/buffer transition owners—never mutate the rope or overlays from a callback and never create a shadow text model. Stage delivery: exact raw-Markdown multiline editing first; then every summoned-surface kind, controls and important notices. Web accessibility is a separate DOM-backed follow-up because AccessKit has no canvas/web adapter today. **Done:** with the display obscured, a VoiceOver user can read, navigate, select, edit, undo/save and operate Commands/Settings; an agent reads the byte-equivalent semantic state headlessly without a GPU. **Verify:** Unicode/grapheme selection round-trips; stable IDs across edits/filtering; one focus owner; no-wildcard surface roster; every advertised action drives the real transition; animation-only frames emit no update; JSON↔AccessKit parity laws; real VoiceOver and Linux AT-SPI journeys. **Routing:** deep architecture owner (`gpt-5.6-sol` high), staged commits with independent accessibility review. **User-approved 2026-08-01.**

211. **Picker selection intermittently appears to advance only every second input, with no transition.** **Defect:** the user reports the alternating-row failure again in the live Commands palette on 2026-08-01 (Quokka screenshot: Switch project selected) and adds that the selection animation sometimes does not play. This is the third report of the shape: Firetail on 2026-07-17, Settings/Mopoke on 2026-07-26 (item 104), now Commands/Quokka. Item 104's exhaustive logical-step and pixel/hit-test laws stayed green and found no fix; item 106 later guarded keyboard selection from stationary-pointer hover. Their green state does not exonerate the live-only seam. **Diagnose before fixing:** instrument one navigation input from winit receipt through `App::apply`, `OverlayState.selected`, redraw request, prepared highlight endpoint, animation scheduling, acquired frame and present. Determine whether the missing visible step is a dropped/repeated input, stationary-pointer takeover, state advancing twice, stale render state, or a redraw/present gap; do not tune animation constants until that chain names the break. Sweep tap, held-repeat and rapid alternating Up/Down; pointer outside/parked above/on/below the row; freshly opened and scrolled windows; focus/occlusion return; Commands, Settings and every picker kind; representative Pane/Bars worlds at 1×/2×. **Done:** every accepted navigation input produces exactly one reachable selection and a presented visual response; Reduce Motion may snap but never suppress the state change. **Verify:** add a live-App event→present trace assertion plus the missing law at the failing owner; retain item 104/106 laws; mutation proof recreates the diagnosed lost/every-other frame; real release run records inputs, selected indices, requested/acquired/presented frames and a 60 fps video or frame sequence. Headless settled captures cannot close this item alone. **Routing:** production tier with deep live-render review. **User-reported with screenshot 2026-08-01.**


215. **Give the live-App capture the card semantics it currently cannot carry.** **Defect:** item 207's passive-surface fold takes its content rather than deriving it, because the render pipeline is the only holder of the live figures — word count, frontmatter language, and through-doc percent. So a `--screenshot-app` capture, which has no pipeline, writes a sidecar whose `semantic` has NO node for a card that is plainly DRAWN in the PNG. Which-key and the menu bar have no such dependency and do appear, so the gap is silent and partial rather than obvious. **Build:** extract those three figures into pure owners over `&str` (plus the buffer facts they already have), so both the renderer and the semantic fold read one owner. **The forbidden alternative, named so nobody re-derives it:** having the `App` recompute word count / language / percent for the snapshot would be a second description of the same fact, which is the exact drift item 207 exists to prevent. **Done:** a live-App capture's `semantic` contains a node for every card its PNG draws, and CAPTURE.md's honesty note about the gap is deleted rather than reworded. **Verify:** a no-wildcard sweep over the card roster asserting PNG-drawn ⇔ node-present; grapheme and CJK word counts through the extracted owner; mutation proof that removing a card from the fold fails by name. **Routing:** production tier. **Follow-up from item 207, 2026-08-02.**

216. **The file-size-mark raise audit skips at both places drift actually appears.** **Defect:** `scripts/code-health.sh` disables its raise audit whenever HEAD is already `main`'s commit, announcing *"the audit only has force on a worktree branch checked before landing."* That is true, and it means the audit never runs on **CI's push-to-`main` job** or on **the merge train's post-merge candidate** — the two places where CROSS-BRANCH mark drift is created and is visible nowhere else. A branch computes its marks against the tree it was cut from; two branches cut from the same base each land green and their combined tree can be over a mark that neither could see. Item 207 demonstrated it: cut from `73d35118`, it carried marks describing a tree that items 174 and 211 had already changed, and the combined `app.rs` matched neither side's number. **This was caught by accident** — health happened to be run while the merge was still uncommitted, so HEAD was still the pre-merge commit and the audit fired. That is not a procedure. **Build:** give the audit a real baseline at the merge candidate instead of skipping — the merge's first parent is exactly the prior `main`, so `HEAD^1` is the comparison the skip claims does not exist. Keep the skip only where it is genuinely true (a plain push whose parent is its own predecessor is still a valid diff; the degenerate case is a root commit). Do not weaken the audit to make it pass. **Done:** a combined tree that exceeds a mark fails at the merge train and in CI, not by luck. **Verify:** a two-branch fixture cut from one base, each green alone and over a mark together, fails by name at the merge; the existing worktree-branch behaviour is unchanged; mutation proof restores the unconditional skip and watches the fixture go green when it should be red. **Routing:** production tier. **Found at the 2026-08-02 merge train.**

217. **`--bench-suite` is broken on `main`, and the witness is arguing with a real re-plan.** **Defect:** the suite aborts with `the scene planner must run exactly once per timed frame (10 plans over 5)`. `prepare_overlay` builds the row plan TWICE — once before `resolve_diagonal_cluster` and once after, because the cluster changes the plan's `dx`/`dw` — while item 174's first-family witness asserts exactly one plan per frame. Both sides are defensible, which is why this is an item and not a patch: the witness exists so that "a consumer grew its own plan" fails loudly, and it is doing its job; the second plan is genuine work the diagonal composition requires. **Decide between:** rebuilding once *after* the cluster resolves (one plan, but the cluster then needs its inputs before the plan exists), or teaching the witness the real per-frame count with the diagonal arm named, so a THIRD plan still fails. Do not simply raise the number. **Scope:** `--bench-suite` is a hidden dev tool, is not in the native gate, and CI is unaffected — this is not urgent and must not be fixed by weakening the witness. **Verify:** the suite runs green on a diagonal world and a non-diagonal one; the witness still fails by name when a consumer adds a plan; the O(visible) and reshape-count witnesses are unchanged. **Found by item 174's second family, 2026-08-02, identically on base and branch — it is pre-existing, not that slice's doing.**

218. **Make native screen-reader editing incremental so VoiceOver never reports awl as unresponsive.** **Defect:** The first real VoiceOver sitting found that editing basically works and that spoken characters/words are ordinary VoiceOver typing echo, but VoiceOver intermittently says “awl is not responding.” Inspection names a credible hot path: while AT is attached, every redraw clones the whole rope, runs UAX #29 over the entire document, projects every semantic node, includes `Tree` metadata, and republishes one monolithic document `TextRun`. AccessKit explicitly expects full trees only at activation and recommends changed-node updates afterward; awl uses the event-loop-only adapter path whose asynchronous activation forces the full-tree form. **Build:** use a synchronous mixed/direct activation handler backed by a thread-safe latest snapshot, then retain native projection state and emit atomic incremental `TreeUpdate`s. Represent document text as stable line or paragraph runs so an ordinary edit updates only affected runs, parent children when structure changes, selection, and focus. Keep `SemanticSnapshot` the one semantic owner; do not create a second document model, manually announce keystrokes, override VoiceOver typing echo, or run App transitions from an accessibility callback. **Done:** typing, deleting, selection, navigation, paste, undo, and surface changes remain correctly announced without stalls on small or large documents. **Verify:** latency/allocation witnesses across document sizes prove a one-character edit does not scan or publish the whole document; full-tree-only-on-activation and changed-node laws; Unicode/grapheme and multiline-selection round trips across run boundaries; mutation proofs restore the monolithic/full-tree paths; real unlocked VoiceOver typing and navigation sitting with no “not responding” report. **Routing:** deep accessibility/performance owner with a production-tier outcome audit. **User-reported and researched against Apple + AccessKit primary documentation, 2026-08-02.**

229. **A Japanese or Chinese manuscript's WORD COUNT is meaningless.** **Defect:** `card::figures::word_count` is `split_whitespace` over the manuscript body, and Japanese and Chinese put no spaces between words. Measured, not assumed: `今日はいい天気ですね。` — 11 characters — reports **1 word**, and `"今日はいい天気ですね。".repeat(500)` — **5,500 characters** — still reports `1 word · 1 min`. **The divergence is script-specific, not "CJK"-wide, and that is the assumption most worth pinning:** Korean is fine (`오늘 날씨가 좋네요` → 3, it spaces its words), mixed text is undercounted by its CJK half (`The title is 今日は…` → 4), and an ideographic space `U+3000` IS Unicode whitespace and does split. Graphemes already hold — ZWJ families, regional-indicator flags and decomposed `é` each stay one token. **Build:** give the readout a script-aware count through the ONE owner `src/card/figures.rs` — do not add a second counter beside it, which is the drift item 215 exists to prevent. A character/ideograph count for unspaced scripts is the conventional answer; whether the readout says "words" for such a document is a product decision, not a mechanical one. **Scope:** the count feeds the HUD readout and the semantic snapshot through one owner, so both move together or neither does. **Verify:** the pinned table above as a regression floor; a mixed-script document; `U+3000`; the grapheme cases unchanged; the sidecar and the drawn readout agreeing. **Found by item 215's measurement 2026-08-03, pinned rather than changed because changing the figure is a product call with sidecar consequences.** ⚠️ **Renumbered from 227 on 2026-08-03: two orchestrators minted 227/228 independently within minutes. Theirs (AppImage, `v0.9.0`) were already cited in `RELEASING.md`, so these moved instead.**

230. **Under a fold, the drawn word count and the announced one disagree.** **Defect:** the renderer derives its figures from the SHAPED text, and `fold::apply_to_view` filters that down to visible lines under a fold — and replaces it entirely with the diff transcript under a History preview. So with a section folded, the DRAWN `WORD COUNT` and `THROUGH DOC` are over the visible document while the ANNOUNCED ones are over the whole buffer. **Pre-existing, not introduced by item 215** — the old `word_count` summed the same filtered pipeline lines — and **the semantic side's answer is the more correct one**. **The question is a product call, which is why this is an item:** should folding a section change the document's word count? Almost certainly not, but the counter-argument is that THROUGH DOC is a position within what you can currently see. Decide, then make both sides read the one owner over the one text. **Verify:** a document with a folded section and a History preview open, asserting drawn ⇔ announced for both figures; mutation proof that filtering the input to the owner fails by name. **Found by item 215, deliberately left open rather than absorbed 2026-08-03.**

231. **Fix the hosted-macOS gate hang introduced by `8207e519`.** **Defect:** `main`'s `mac (build + test)` job has been red for ~140 commits. Bisected to **`8207e519`** ("item 194: one camera, one projected cylinder, cropped at the page") with **both boundaries measured** — parent `36707d06` GOOD in 1092 s, `8207e519` BAD — over six sequential probes, deterministic, no re-run contradicting a first reading. **Evidenced mechanism:** the commit takes `THEMES` from 19 to 20, adding `KITE` with `Background::WarpedGrid` and +267 lines of `background.wgsl`; 42 test files reference `THEMES`, and the tests that wedge are **roster sweeps** (`split_pane.rs` sweeps EVERY shipped world). All three libtest workers — the runner's 3 vCPUs — park at the same instant, **the victim differs between runs** (`scroll_pos` in one log, `split_pane`/`stars` in another), and the orphans survive SIGTERM: the **shared wgpu device wedges** and every later `read_pixels` parks in `poll(PollType::wait_indefinitely())`. ⚠️ **THE SHADER IS EXONERATED BY EVIDENCE, NOT ARGUMENT — do not start by staring at the WGSL.** 15 `backgrounds_item132`/`warp_tunnel` tests passed cleanly **six minutes before** the wedge, in two independent logs; there is no unbounded loop in it and `warpgrid.rs` touches no wgpu at all. **Hypothesis, explicitly unconfirmed:** cumulative exhaustion of a driver-internal resource in the virtualised Metal stack — compiled-pipeline slots or shader-compiler memory — after which submits never retire. `free_bytes` steady at ~2.37 GB rules out RAM exhaustion but says nothing about a driver-internal table. **The asymmetry that decides ownership:** the per-frame `create_shader_module` + `create_render_pipeline` churn exists **only in the test helpers** (3 sites); the live app builds `BackgroundPipeline` **once** at construction (`pipeline_draw.rs:32`) and `prepare()` thereafter only uploads uniforms *including the shader id*, so switching themes never rebuilds the pipeline and a user pays one compile per launch. **So this reads as test-harness-amplified rather than a product defect — but that rests on the churn hypothesis being right; if device state accumulates from the WarpedGrid draw itself, a user on a VM is exposed.** **The cheapest next discriminators, ~10 minutes each and already supported by the harness:** a hosted-mac run at `--test-threads=1` (does the wedge need concurrency?) and one restricted to `render::tests::` (does it localise?). **Tooling:** `scripts/ci-mac-bisect.sh` on branch `claude/ci-mac-bisect` (`c336cc1a`, never pushed) has `probe`/`verdict`/`next`/`cleanup`. **Two harness bugs to carry, both of which scored a 60-minute hang as a PASS and both the same shape — an unfinished step wearing a finished step's field:** `gh` encodes an unfinished step as `conclusion:""` (never `null`), and a step killed by the job ceiling reports `status:"completed"` with `conclusion:"cancelled"`. **A harness reading a status field must enumerate what it accepts, never test for inequality.** **Diagnosis complete 2026-08-03; the fix is unstarted.**

232. **No local gate exercises a virtualised GPU, and that blind spot just cost ~140 commits of red CI.** **Defect:** `scripts/native-gate.sh` runs on the dev host's real Apple Silicon Metal. Nothing in the local gate set exercises a virtualised GPU, so a receipt's greenness never was evidence about that axis — and item 231's defect is green on real Metal and red on virtualised Metal. **This is not a harness artifact of no consequence:** it left unkillable processes and took down whole VMs, and it went unnoticed for 140 commits because the only signal that could see it was the CI job everyone had stopped believing. **The honest statement of what a receipt certifies today is "sound on the hardware the receipts run on, with virtualised-GPU behaviour untested by any local gate."** **Build:** decide whether awl wants a virtualised-GPU arm at all, and if so where — a container with a software adapter (lavapipe/SwiftShader) in the local gate, a CI job that is allowed to be slow, or an explicit declaration that the hosted-mac CI job IS that arm and must therefore be treated as gating rather than tolerated red. **Do not answer it by making the local gate slower for every developer without deciding that consciously.** **Done:** the tier a receipt certifies is stated accurately wherever receipts are described (`CLAUDE.md`, `.orchestrator/README.md`, `RELEASING.md`), and either an arm covers the axis or its absence is a recorded decision. **Verify:** the chosen arm reproduces item 231's wedge before its fix and passes after. **Routing:** deep tier — this is a testing-strategy decision, not a script change. **Found by item 231's bisect owner, correcting this orchestrator's claim that "the tree the receipts were certifying was sound" 2026-08-03.**

233. **`SerialGuard` restores globals but does not police the render overrides.** **Defect:** found by item 204 slice 2, whose extra test only changed the scheduling and exposed a pre-existing hole. `jump_hint_is_present_and_never_clips_for_every_kind` built its pipeline **before** taking the serial guard and never pinned the list style it measures against, so a leaked `Bars { gap: 8.0, FullWidth }` override made it report a clip that was not one — **green single-threaded, red in a wide parallel run.** The reader's end is closed (guard hoisted, style pinned). **The producer is not:** `list_surfaces.rs:909` is the only site with that exact value and it *does* reset, so the leak path is unexplained. **The real gap: `SerialGuard` restores world, page and spellcheck but leaves the render overrides unpoliced**, so any test that sets one and dies — or resets on a path an early `?` skips — poisons whatever runs next, and the victim is a *different* test in a *different* file. Same shape as the CI wedge item 231 diagnosed: a shared resource corrupted by one actor, failing somewhere unrelated. **Build:** bring the render overrides under the same restore discipline as the other globals; prefer making the leak impossible over finding the leaker. **Verify:** a fixture that sets an override and panics must not affect the next test; the whole suite green under a wide `--test-threads`; mutation proof that removing the restore fails by name. ⚠️ **A green single-threaded run proves nothing here** — the defect only appears under parallelism, the axis a local reproduction is least likely to sweep. **Found 2026-08-03; the leak path is a named unknown, not a solved problem.**

234. **Cassowary's Settings clips its own value plates.** **Defect:** in the Settings workspace on Cassowary, the "Block" value plate cuts the final `k` at the panel's right edge. **Confirmed PRE-EXISTING on the base binary** by the item 219/225 lane, which found it while capturing and deliberately did not widen its scope. **Same class as items 220/221's palette work** — a value/accessory plate measured against the wrong right bound — so it may share an owner with them and should be looked at together rather than patched alone. **Build:** find the one owner of the value plate's right bound in the workspace's content pane and make it read the pane's real extent; do not special-case Cassowary and do not shrink the type. **Verify:** the full `SettingId × SettingKind` sweep at the widths where it bites, across the roster rather than the world it was noticed on — items 219 and 225 were both reported as world-specific and both turned out to be universal, so **assume universal until measured otherwise**; exact before/after captures with every unaffected surface byte-identical; a pixel law that fails on the clipped glyph. **Found 2026-08-03 by the 219/225 lane, not fixed.**

235. **Give awl a rotated glyph run, so a world can slant or turn a label.** **Defect:** items **221** (Cassowary's `Files` as a 90°-rotated flush-left secondary heading) and **224** (Magpie's location cue given the world's slant and gradient) are both blocked, and **not by the design.** The item-220 lane proved the theme-data constraint holds — both expressions are a `RenderCaps` variant over 220's single `overlay_location` datum, exactly the shape `Background`, `CardTexture`, `FacetStyle` and `TitleStyle::Placard` already have, and neither needs a palette code path. **What blocks them is a capability gap:** awl draws text only through glyphon 0.11, whose `TextArea` exposes `left/top/scale/bounds/default_color/custom_glyphs` and whose `CustomGlyph` is `left/top/width/height` — **no transform, no rotation, no skew anywhere in the crate.** The only rotation in the tree is `SelectionPipeline::prepare_rotated`, which rotates **rounded-rect quads, not glyphs**. **Build:** a rotated glyph-run pipeline — the shape already exists in the tree as a mask cache (`caret_glyph.rs`) plus the axis rotation `shaders/caret.wgsl` performs. **It is world-neutral: build it once and both 221 and 224 become theme data.** **Scope:** this is a text-rendering capability, not a palette feature. It must not become a second prose renderer — CLAUDE.md's "infrastructure complexity is a smell" applies, and the document layer stays the one prose renderer. **Done:** a label can be drawn at an arbitrary axis through one owner, legibly at 1× and 2×, and 221/224 land as data on top of it. **Verify:** rotation at the axes both worlds need, at both DPIs; legibility measured rather than eyeballed; hit-test agreement if the rotated run is ever interactive; exact identity for every surface that draws no rotated text. **Blocks 221 and 224. Found by the item-220 lane 2026-08-03, which correctly deferred rather than branching the palette per world.**

236. **Item 225's footer-plate law grades two worlds' fabricated plates.** **Defect:** found by the item-220 lane while working nearby, and worth a second look because it is a law defect rather than a product one. `a_workspace_footer_plate_ends_with_its_footer_on_every_bare_plate_world` sweeps the `list_backing == BarePlates` roster, which has five members — but **two of them, Mangrove and Magpie, are `ListStyle::Diagonal` and draw no plate at all.** `overlay_bar_rects_probe` *synthesizes* rects for them, so the law's quad arm grades fabricated geometry. **Its pixel arm self-skips, so the law is not lying about pixels** — but a law that grades a shape the product never draws proves nothing on those two cells and could go green over a real defect there. **"Bare-plate roster" is not "plate-drawing roster", and the two have been used interchangeably.** **Build:** give the law the roster it actually means, and check whether any sibling law makes the same substitution. **Verify:** the law still fails on item 225's original defect; mutation proof on a plate-drawing world; the two diagonal worlds either excluded by name with a reason or covered by a law that grades what they really draw. **Found 2026-08-03.**

213. **Optically center the logo cursor inside every app icon.** ✅ **COMPLETE — `f8d023e1`; user approved the 3 px lift on 2026-08-02.** The canonical macOS icon, complete world roster, paired favicon assets, and all Block/Pill/Narrow galleries were regenerated together; exporter/container laws and raster-clearance sweeps passed before review.

219. **Remove the stray blank band above the theme picker across affected worlds.** **Defect:** the top-right theme picker shows an unwanted empty strip above its content in Mopoke, Currawong, Gumtree, Bilby, and Bowerbird. The same control must not acquire world-specific vertical dead space. **Build:** locate the shared picker/header layout owner and remove the erroneous gap without hand-tuning individual worlds or changing the rowlayout contract. **Done:** the picker begins at its intended top boundary in every world, while its selected row, query, keyboard navigation, and dismissal geometry remain correct. **Verify:** roster sweep of theme-picker captures across all worlds, with pixel/geometry law proving no unexplained blank header band; mutation proof restores the gap; vision smoke locates the selected row. **Routing:** production tier. **User-reported in a 2026-08-02 design session.**

220. **Give the command palette a deliberate two-level location hierarchy instead of a duplicate Files title.** **Defect:** the Files filter currently repeats “Files” in a small, low-contrast label, while the palette needs to distinguish the primary content heading (“Commands”) from the active category. **Build:** retain Commands as the prominent primary heading; show no secondary category label for All; when Files is active, replace the duplicated label with a clear secondary location treatment positioned with the command list. Keep hierarchy, contrast, and spacing coherent without changing command order or key behaviour. **Done:** a reader can tell both the current content level and active category at a glance, with no repeated title. **Verify:** All and Files captures across the full world roster; sidecar/replay coverage of filter selection; pixel/semantic geometry laws for heading order and contrast; mutation proof restores the duplicate; vision smoke identifies the active category. **Routing:** production tier with visual-judge review. **User design decision 2026-08-02.**

221. **Make Cassowary’s active Files category cue a vertical secondary heading.** **Defect:** the generic Files treatment does not use Cassowary’s left edge and strong Commands heading to establish its intended two-level hierarchy. **Build:** when Files is active in Cassowary, render “Files” as a smaller secondary-colour counterpart to the bold Commands heading, rotated 90 degrees and aligned flush with the far-left border; show none under All. Reuse the shared hierarchy data from item 220, with Cassowary’s expression supplied as theme data rather than a new palette code path. **Done:** Cassowary presents primary Commands plus a legible, subordinate vertical Files cue without crowding commands. **Verify:** Cassowary All/Files captures at representative canvas sizes and scale factors; geometry/contrast laws for left-edge placement, rotation, and non-overlap; mutation proof removes the cue; visual review confirms hierarchy. **Depends on item 220. Routing:** production tier with visual-judge review. **User design decision 2026-08-02.**

222. **Keep Mangrove’s diagonal-line gradient fixed while its item list scrolls.** **Defect:** scrolling the command list shifts the diagonal gradient, producing a conspicuous, jarring motion unrelated to the list movement. **Build:** anchor the gradient to its intended stationary surface while list rows scroll independently; preserve the world’s diagonal treatment and all scroll/input behaviour. **Done:** only the list content moves during scrolling; the diagonal gradient remains visually fixed. **Verify:** deterministic scroll trajectory captures for Mangrove plus a comparable non-diagonal control world; pixel arithmetic across frames proves the gradient does not translate; mutation proof restores scroll-coupled movement; live release confirmation of feel. **Routing:** production tier. **User-reported in a 2026-08-02 design session.**

223. **Restore Mangrove command-palette keybindings.** **Defect:** Mangrove’s Command-P palette omits visible keyboard shortcuts that comparable themes show. **Build:** route keybinding labels through the shared palette presentation owner so Mangrove preserves the same keyboard affordances, with world-appropriate contrast and no local special case. **Done:** every command that exposes a shortcut elsewhere exposes the same shortcut in Mangrove, legibly and in the correct row. **Verify:** command-palette shortcut roster sweep across all worlds, with no-wildcard sidecar/pixel law for label presence and row association; mutation proof hides a Mangrove label; visual smoke reads shortcuts. **Routing:** production tier. **User-reported in a 2026-08-02 design session.**

224. **Redesign Magpie’s command-palette location indicator as a mirrored diagonal cue.** **Defect:** Magpie’s new right-side location indicator looks poor and does not belong to the world’s diagonal visual language. **Build:** prefer the indicator on the left. If a layout places it on the right, mirror its form. In either position, give it a slant and gradient matching Magpie’s diagonal line, while preserving legibility, palette hit targets, and command geometry. **Done:** the indicator clearly communicates location and reads as part of Magpie rather than a detached marker. **Verify:** Magpie captures across left/right layout conditions, canvas sizes, and scale factors; geometry laws prove mirroring and no overlap; gradient/angle law ties it to the diagonal line; mutation proof restores the unmirrored right-side form; visual-judge review. **Routing:** production tier with visual-judge review. **User design decision 2026-08-02.**

225. **Remove Cassowary Settings’ oversized black sub-settings bar.** **Defect:** Cassowary’s Settings panel renders a large black rectangle beneath the sub-settings content. **Build:** identify the shared settings/workspace layout or Cassowary render-cap cause and eliminate the unintended surface without masking it by overlaying another rectangle. Preserve Settings categories, focus, scrolling, and detail controls. **Done:** the panel ends at its intended content boundary with no black bar, and other worlds remain unchanged. **Verify:** Settings captures across the full roster and all Cassowary sub-settings states; pixel/geometry law for the panel’s bottom boundary; mutation proof recreates the black bar; vision smoke confirms controls remain visible. **Routing:** production tier. **User-reported in a 2026-08-02 design session.**

226. **Prepare awl’s first GitHub Release around the existing Linux tarball.** **Build:** retain `awl-linux-x86_64.tar.gz` as the technical Linux download, make it discoverable from the release/download surfaces with concise unpack-and-run guidance, and attach a checksum manifest for every downloadable release artifact. Exercise the current release workflow as a dry run, including the release-profile parity gate and Linux packaging path; diagnose any publication, provenance, or archive-layout failure before a tag is considered. Configure the first public beta to publish **Linux only**; do not attach unsigned macOS or web artifacts. **Scope:** this is release preparation, not authority to tag or publish. A public tag/release remains an explicit user decision; macOS waits for Apple signing and notarization. **Done:** a dry run yields an inspectable Linux tarball and checksums, the release page has an unambiguous technical install path, and the release checklist names the still-required public-release and mac-signing decisions. **Verify:** unpack the produced archive in a clean Linux environment, run a headless smoke and launch check, verify checksums, and confirm the dry run created no tag or Release. **Routing:** production tier. **User design decision 2026-08-03.**

227. **Add a desktop-integrated AppImage as awl’s friendly Linux download.** **Defect:** the tarball is appropriate for technical early adopters but is not a normal Linux desktop application: it has no launcher metadata or icon integration. **Build:** package awl as an x86_64 AppImage in the release workflow, alongside—not instead of—the tarball. Include the binary, a `.desktop` launcher entry, the canonical Linux PNG icon derived from the existing icon pipeline, licenses/credits, and only the runtime libraries that belong inside the package; do not bundle GPU drivers. Publish a checksum and stable release-asset name. **Done:** a user can download one file from GitHub Releases, mark it executable, launch awl, and receive correct desktop name/icon integration where the desktop supports it; the tarball remains available as fallback. **Verify:** AppImage structural validation; launch and headless smoke on representative Debian/Ubuntu and Fedora-like environments; Wayland and X11 launch checks; icon/desktop-entry law; GPU-adapter and file-open smoke; mutation proof removes launcher/icon packaging; release dry run uploads both Linux artifacts. **Depends on item 226. Routing:** production tier with a Linux visual/compatibility audit. **User design decision 2026-08-03.**

228. **Version the first public beta as `v0.9.0`, then launch `v1.0.0`.** **Decision:** the first internet-facing Awl release is a public beta named **Awl 0.9.0 — Public Beta**. The GitHub Release is marked prerelease, but the version itself carries no `-beta` suffix. Patch releases (`v0.9.1`, etc.) are for launch-blocking fixes and polish; `v1.0.0` is the official launch once the core install-and-writing journey is ready. **Build:** update `Cargo.toml` and all version-bearing release surfaces together only when the release-preparation work is green and the user authorizes the public tag. **Done:** the package version, tag, GitHub Release title/status, downloadable artifact names, and release notes tell one coherent pre-1.0 story. **Verify:** release dry run names artifacts with `0.9.0`; version/source law finds no stale `0.1.0`; release checklist distinguishes the prerelease from the later `v1.0.0` launch. **User design decision 2026-08-03.**
