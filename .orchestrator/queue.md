# awl — live build queue

> Live execution state only. Completed and superseded work is in git history
> (`git log -p .orchestrator/queue.md`). Protocol, claiming, worktrees, and
> execution hygiene live in `.orchestrator/README.md`.

## 🔵 BLOCKED ON THE USER — nothing else can close these

Consolidated 2026-08-02/03 so they are not scattered through the item bodies.
Every one has been taken as far as an agent can take it; what remains genuinely
needs a human. **None of them blocks any other work.**

**This remaining item needs an unlocked and FOREGROUNDED display.** ⚠️ The
machine's idle lock fired seven minutes into the 2026-08-02 sitting and
silently invalidated it — **disable the idle lock before the next one**, and
re-check the lock at BOTH ends of the run, because `live-probe.sh` only checks
it in preflight. Worse, `--live-script` forces a Prohibited, non-activating
window, so under a lock it writes successful-looking
`LIVE-PROBE shot … ok backend=window-server` lines while presenting **zero**
frames — a probe run can look like it worked and have photographed nothing.

1. **118 — the world-loudness map and the `--release` ambient sitting.** The
   item's own Done clause requires a USER-CONFIRMED map and states that pixel
   arithmetic may prove territory and contrast but never claims the taste
   score. An independent agent map exists (`1, 10, 3, 4, 1`, mean 2.68) to diff
   against rather than re-derive.
2. **207 — real VoiceOver and AT-SPI journeys.** Everything is verified at the
   snapshot and projection tier. Whether a screen reader *reads it well* —
   announcement order, verbosity, live-region politeness — is unproven and no
   test tier can stand in for it.
3. **131c — the chrome pixel-space decision.** Overlay chrome already mixes
   both spaces: row pitch scales with DPI while `BAR_SIDE_INSET`, the text hpad
   and `CARD_MAX_W` are raw device px. A diagonal pitch authored like its
   neighbours would be **physical by inheritance**, which is exactly what item
   186 exists to stop; making it logical would make it the first chrome
   quantity to declare its space, which either extends `ground_space` past
   `Background` or opens a sibling registry. **A design decision owed a human
   eye, not a line of code.** 131d and 131e are unclaimed behind it.
4. **211 — ✅ PRESENCE CONFIRMED ON A REAL SCREEN 2026-08-03; only FEEL and two
   arms remain.** Frames were photographed at last: a 10 s capture of the awl
   window alone, 110 frames, band top measured per frame by pixel arithmetic —
   one row per input over ~9–10 frames with the `out_back` overshoot and the
   two-row morph stretch mid-flight. **No input moved two rows; none snapped
   without a transition.** ~1,072 presented frames over 22 release launches,
   exactly 2 `Occluded` per launch (startup, before `occluded=false`) and zero
   after — the previous sitting's 10/10 `Occluded` was the lock and nothing
   else. **The defect was reproduced ON SCREEN and restored:** reverting
   `keep_gpu_loop_hot` to `stepped && frame_presented` parked the band on the
   row the selection left, then snapped two rows — distinct drawn band tops
   across four inputs, **2 mutated vs 34 fixed**, the user's report verbatim.
   17 sweep cells covered. 🔵 **Owed, and all three need a human:** whether the
   glide *reads* as calm (pacing was deliberately NOT characterised — the host
   ran at load 19→57 and the lane refused to offer its 16.7 ms intervals as
   evidence of smoothness); **1×**, since no 1× display was available; and
   **focus loss/regain and occlusion return**, which `--live-script`
   structurally cannot test — it forces a Prohibited, `AlwaysOnTop` window that
   can neither lose focus nor be occluded, so that arm needs a normal
   activating launch. **Scope note:** the Settings-as-workspace CATEGORY band
   moves one row per input and presents (253→297→340→384 px, verified by pixel
   arithmetic) but does not go through the living-band animator — it snaps,
   `prepare_highlight` never reports it, and 211's fix neither applies nor is
   needed there. Evidence in the gitignored `gallery/item-211-live-2026-08-03/`.

⚠️ **A SHARED-WORKING-TREE HAZARD, found by causing it.** Commit `47b9e40f`
carries a message about the heartbeat's Linux branches and ALSO silently
deleted items 211, 207 and 131c from this section. The mechanism: **two
orchestrators edit this file in the SAME working tree, so `git add -A` sweeps
whatever the other one has in flight into your commit under your message.**
`.orchestrator/README.md` §5 already says to reread at HEAD and diff before
committing — that is necessary but NOT sufficient, because the other tool's
edit can land between your read and your commit. **Stage `queue.md`
deliberately and read `git diff --cached` before every board commit; never
`git add -A` in the shared tree.**

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

## CI RED — the mac gate HANGS; observability landed, cause still open

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
`0.0`. ✅ **Its Linux branches are now CONFIRMED WORKING** — they were shipped
written-and-reasoned but never executed (no Linux host locally), and CI's linux
job in run `30754462965` is their first real exercise: `load1=2.13 cpu_count=4
tracked_procs=8 tracked_cpu_pct=91.3 busiest=[rustc:6107=91.3]`, rising to
`358.3` under multithreaded rustc (100 = one core, so >100 is correct), with
each convention's last line captured. No `unavailable`, no `none`, no confident
zero. The one caveat that shipped unproven is closed.

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

## Active claims — 2026-08-02 wave

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
- **211** — ✅ COMPLETE; user confirmed one-row glides in Commands, Settings,
  and Themes on the fixed release build, 2026-08-03. Merged to
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
- **204** — 🟡 IN PROGRESS — claude (deep owner; data loss, so routed above the
  item's stated production tier), branch `claude/item-204-external-changes`,
  worktree `../awl-next-worktrees/item-204-external-changes`. **UNBLOCKED by
  116d's flip:** its three views map directly onto the typed `ComparisonView`
  (`Differences` | `Mine` | `Theirs`), so it adds a producer beside
  `history::comparison_prose` rather than the one-off renderer its premise audit
  forbade. Briefed that mtime + length CANNOT detect the same-time/same-size
  rewrite its own Verify clause requires — a byte fingerprint or retained
  baseline is mandatory — and that the prohibitions (no watcher service, no
  second editable buffer, no side-by-side renderer, no auto-merge, no duplicate
  user file) are as binding as the requirements.
- **116d** — ✅ COMPLETE. Merged and pushed; worktree removed. Receipt
  `native-gate-receipt commit=86d73aa3… conventions=mac,linux scope=all-targets`.
  History is the timeline/comparison workspace: the flip, the lens in the header,
  both deep links, the restore notice, and the payload generalisation.
  **`workspace_header_beat` was FOLDED** into `plan::header_band_height` — the
  previous owner left it deliberately, reasoning that it was a fourth copy of a
  ONE-LINE header and would become *wrong* rather than merely duplicated once
  `header_rows` became 2, which is exactly what happened. The row band IS the
  timeline column, so the ordinary candidate-row hit-test is the timeline
  hit-test. **Six defects the new laws found, each red before green:** the
  capture path drew the OTHER SHAPE because a sidecar carries a mode and not a
  shape, so replay never set `overlay_rows_primary` while every unit law stayed
  green; an empty timeline relocated the LIVE DOCUMENT into the comparison's
  place — the third readable layer item 116 exists to remove; Mangrove/Magpie's
  selected-row overhang had nowhere to go on a workspace; two footer clips, one
  from a LABEL-scaled measurement under-measuring Potoroo by 1.2px; a latent
  item-114 `Bars` footer plate 450px tall; and the narrow timeline stage frosted
  the PARKED TRANSCRIPT into the backdrop, caught by vision smoke on Firetail at
  900×520. ⚠️ **TWO ORCHESTRATOR PREMISES WERE WRONG, both material:** (a) an
  ordinary `--keys` `--screenshot` DOES reach the comparison fully with a store
  seeded under `XDG_DATA_HOME`, and it is **`--screenshot-app` that cannot** —
  its hermetic FS has no store; the brief claimed the reverse, and the seeded
  ordinary capture is what found two of the six defects. (b)
  `the_workspace_beat_still_agrees_with_the_planned_query_box` did **not** fail
  when broken — it compared the band to the *query box*, which is
  `lh + header_gap` on BOTH shapes, so a two-line header slid straight past it;
  re-aimed to sweep both shapes with a non-vacuity floor. **Left explicitly:**
  the narrow COMPARISON stage draws no footer (`show_rows` false → `hint_rows` 0),
  so nothing teaches `tab back` / `esc close` at ~900×520 and below — a
  discoverability hole rather than a trap, since you reach it by `Tab` from a
  timeline that did teach both and one Esc always leaves; and on Mangrove/Magpie
  the narrow timeline column now elides mid-word, which item 131e owns.
- **116d (compositing round)** — 🟢 LANDED; the flip is deliberately NOT done.
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

## Remaining work — handoff order (2026-08-03, overnight wave)

**In flight:** the mac-CI bisect (probes on `ci-probe/mac`), **204**, **216**,
**217**. That is the four-worker budget; nothing else may start until one
returns.

**⚠️ COLLISION MAP — items 219–225 are NOT independent and must not be
dispatched as a batch.** Most of them write the same overlay/chrome files, and
two are explicitly coupled. Dispatch in these groups, one group at a time:

1. **220 + 221 together, one owner.** 221 says outright that it "reuse[s] the
   shared hierarchy data from item 220, with Cassowary's expression supplied as
   theme data rather than a new palette code path" — splitting them buys a
   handoff bug and nothing else.
2. **219 and 225 as one owner.** Both are "an unintended surface appears"
   defects (a blank band above the theme picker in five worlds; a large black
   rectangle under Cassowary Settings' sub-settings). Both are most likely a
   shared layout owner rather than world-specific dead space, and 225 forbids
   masking it by overlaying another rectangle. One owner is likely to find one
   cause.
3. **223 alone** (Mangrove palette keybindings — routes labels through the
   shared palette presentation owner, no local special case).
4. **222 + 224 together** — both are Mangrove/Magpie diagonal-language work and
   both touch the diagonal composition items 131a/b own. **Check item 131e's
   scope before starting:** 131e owns selection composition and the
   `Choreo::TwoShape` echo-band question, and 222's "anchor the gradient to its
   intended stationary surface" is adjacent to it.

**Then, in order:** **215** (extract word count / language / percent into pure
owners so a live-App capture carries card semantics — it touches
`src/card/content.rs`, which item 207 created, so keep it away from anything
else in `app/semantic/`); **174's next family**; and **131d/131e**, which are
behind the 131c decision now recorded in the user-blocked section.

**226 (first GitHub Release) is DISPATCHABLE ONLY AS A DRY RUN.** Its own text
asks to "exercise the current release workflow as a dry run … diagnose any
publication, provenance, or archive-layout failure before a tag is considered."
That much is agent work. **The tag and the release itself require the user's
explicit word, every time** — see `.orchestrator/README.md` §Gates and
`CLAUDE.md`. Do not create a tag, do not publish, and do not deploy the site.

**Standing merge-train procedure, learned the hard way 2026-08-02:** run the
train gate from a DETACHED WORKTREE pinned at the candidate commit
(`git worktree add --detach ../awl-next-worktrees/train-gate <sha>`). A second
orchestrator commits board updates to `main` continuously, and
`native-gate.sh` correctly refuses a receipt if HEAD moves under it — two full
native runs were thrown away this way in one evening before the pinned worktree
fixed it. The pinned tree's HEAD cannot be moved by anything on `main`.

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
