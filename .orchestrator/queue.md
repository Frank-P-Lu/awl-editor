# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 630 — bound document context and parsing work during prose edits (user request, 2026-09-08)

🟢 READY — lower priority than 629; queued only, not dispatched. Profile before choosing
scope; this is not authorization for a speculative parser or rope replacement.

Evidence: the combined manuscript benchmark reports about 0.973 ms in document-span
parsing and 0.661 ms in context preparation per edit. Context includes multiple calls,
so the entire figure must not be attributed to language evidence. Source confirms
`src/render/text.rs::set_text_incremental` calls `script::cjk_evidence(text)` and
`parse_doc_spans(text)` on every edit. `src/script/evidence.rs` scans all characters in an
English manuscript; `parse_doc_spans` passes the full document to the Markdown parser.
The editor already uses a rope. Avoidable work is in these derived representations.

Build: measure these owners separately and establish bytes/lines actually visited.
Choose the smallest measured improvement: retain per-line language evidence with an
exact document aggregate, and/or retain parsing with correct nonlocal invalidation.
A deleted last decisive CJK character must retract its evidence. Markdown fences,
frontmatter, references and other cross-line constructs must update affected suffixes;
unchanged text is not proof that derived styling is unchanged. Preserve preedit/IME,
code-language behavior, buffer identity, and settings invalidation. Keep a full path
for scopes that cannot yet be updated safely. Coordinate parsing/invalidation ownership
with 629 rather than creating separate caches of the same rule.

Done/Verify: isolated release before/after on size and edit-position ladders; work-count
and full-recompute equivalence laws cover insertion/deletion of decisive script evidence,
frontmatter, fences, Unicode, undo/redo, preedit, and buffer swaps. Mutation-prove the
headline law. Read `docs/fonts.md`, `docs/markdown.md`, and `docs/harness-reach.md` before
render verification; native gate, wasm, and a render outcome audit for changed styling.
Do not claim constant-time typing or an expected speedup without measurements.

---

### 628 — one shared chrome language for Find, Settings and the theme picker (user approval, 2026-09-08)

🟢 READY FOR COORDINATED PROTOTYPE — queued only, not dispatched. Implementation follows
user review of the composed prototype.

Problem: the user rejected the live Find strip as crowded and the live Settings
workspace as a huge surface with a tiny, tightly packed cluster and distant values.
They expected Find, Settings and the theme picker to share the approved bordered
form's family. Individual feature briefs reproduced controls but lost composition.
This item owns the coordinated correction; it is awl-rendered chrome, not a request
for OS-native widgets.

Coordination: supersedes conflicting visual choices in 591 (landed), 592 (reverted),
and 609 (landed). 589's shared-control work must use this language; retain its Commands
scope, but do not independently land overlapping visual changes ahead of this review.
592's separate visual implementation waits for this prototype; retain its correctness
findings and tests. 622/623's test-integrity repairs remain valid independently. Read
current tree/status before implementation; do not reapply a reverted branch wholesale.

Shared foundations (initial prototype measurements, not frozen final constants):
- Interface scale and display DPI control chrome, independently of document zoom.
  One UI face per world; labels/values share a readable size, titles about 1.25x and
  quieter hints about 0.85x. Respect font metrics and readable minimum sizes.
- Four-unit spacing rhythm: 8 within groups, 16 between groups, 20–24 panel padding.
  Controls start at 32 logical units high, growing for font metrics. One corner family
  per world, inner controls smaller-radius than their enclosing panel.
- Figure/ground by value; caret retains the accent. Clear focus outline/selection;
  distinguish selected, focused, pressed and disabled. Selection stays visible when
  keyboard focus moves away. Hints use actual platform/rebound keys for the focus.
- Start with opaque backing. Frost may express a world but cannot be necessary to
  read its controls; ambient background effects are outside this item's scope.

Shared components, one owner each: bounded text fields with persistent labels and
placeholders distinct from labels; bounded named buttons with optional shortcuts;
a checkbox plus label as one clickable group (no separate Aa beside Match case);
choice rows with nearby values and a choice affordance; consistent selectable list
rows; modest section headings with deliberate gaps and only useful dividers.
Route interactions through existing Action owners and expose real accessibility
roles/states/actions. Shared render components remain driven by theme data.

Surface compositions:
1. Find/Replace: compact form at the EXISTING top-right inset, starting 420–480 logical
   units wide and clamped to the window. Find field, optional Replace field, then
   count + previous/next + Match case group, optional Replace/Replace all actions,
   then quiet hints. Labels beside fields where roomy, above where narrow. Find-only
   shrinks vertically; never flatten all groups into one strip. Approved bordered
   reference: `references/find-replace-chrome.png` beside this board.
2. Settings: modest title and identifiable search; category rail around 140–180 units,
   gap 24, bounded detail column around 360–520. Labels left, controls in a consistent
   nearby column, rows initially 36–40 high. Extra window space surrounds useful
   content rather than separating labels/values. Full-workspace backing is allowed;
   composition still needs deliberate proportion. Narrow mode uses successive
   category/detail views and preserves search/focus/editor restoration.
3. Theme picker: stable chooser using the same fields/rows/surface family. Explicit
   stable panel composition, not the opening world's arbitrary list arrangement.
   Freeze position, width, UI face/size, row height/count, border geometry and
   selection treatment throughout preview. For the FIRST PROTOTYPE also freeze
   picker colors until dismissal; everything behind it continues live theme preview.
   Reopening adopts the chosen world's chrome styling. Preserve commit/cancel semantics.

World contract: grouping, hierarchy, behavior, label/value relationships, minimum
spacing/legibility, state meanings and responsive behavior are common. Worlds author
UI face, palette, corner/border character, plates/rules, state styling and optional
frost/motion. Plate/rule worlds retain identity while preserving recognizable controls
and grouping. Theme picker's stable composition is the explicit layout exception.

Phase 1 / review: show all THREE surfaces together in Kite and Mopoke, at normal and
narrow actual window sizes, including focused fields, selected rows and replacement
mode. Prototype in awl with headless captures per repo policy; no HTML artifacts.
Measure useful content, hierarchy and spacing, not merely panel/control presence.
User approves these coordinated compositions before Phase 2's shared implementation.
Do not call an image-generation approximation a product screenshot.

Phase 2 / done: implement reviewed measurements through shared owners; compare real
awl captures against the approved compositions before declaring completion. Read
`docs/render.md`, `docs/config.md`, `docs/harness-reach.md` and CAPTURE.md before
choosing probes. Sweep roster compositions, narrow/wide, UI scale, 1x/2x DPI, keyboard
and pointer routing, theme preview stability and cancel/restore. Assert real geometry
and relative rendered-pixel presence/legibility, with mutation proof and five-shot
vision smoke; appropriate native/wasm gates follow implementation. Headless captures
do not settle taste or live motion. This board-only decision claims no receipt.

---

### 615 — separate mouse dispatch, selection, surfaces, and scrolling (user request, 2026-09-08)

🟡 IN PROGRESS — Codex (codex), branch `codex/615-mouse`, worktree `.worktrees/615-mouse`.

`app/input/mouse.rs` combines document hit testing and selection, overlay navigation,
search/menu clicks, cursor feedback, and wheel routing in one oversized module.
Separate these responsibilities into focused mouse submodules, leaving the event
entry points and precedence visible in one dispatcher. Preserve gesture ordering,
selection/undo/fold semantics, redraw scheduling, and existing input-state ownership.
Shorten historical and repetitive commentary while retaining units and invariants.
No picker-input or animation-state redesign belongs in this item.

Verify: compare extracted method bodies against base, update source-law enrollment
for every moved consumer, and run targeted pointer/selection/scroll and ownership
laws. Independently audit outcomes and add a missing law if the audit finds a gap;
prove any new headline law with a compiling mutation. Then commit and run the full
native gate plus wasm smoke before landing. Pointer timing/feel remains live-only;
no keyboard capture is claimed to verify mouse events. No render geometry or styling
change is intended.

---

### 579 — awl renders ~9 fps on a pure software rasterizer, every world (measured by 566, 2026-09-06; predates 564)

Measured on the full roster at 2910x1720 @2x, `--release`, median `queue.submit +
device.poll` over 300 timed frames, under `llvmpipe (LLVM 15.0.6, 128 bits)`: **82-184 ms
per frame for every world** — Wagtail 82.2 at the fast end, Saltpan 184.3 at the slow. The
same binary on this host's Metal renders Kite in 1.310 ms, so lavapipe is ~84x slower
across the board. This is a property of the whole render, not of any one ground, and it
predates item 564.

It is recorded rather than actioned because nobody has established that it MATTERS: a
Linux user on real hardware has a real GPU, and the software path is what a VM, a
remote-desktop session, or a machine with no working Vulkan driver falls back to. The
question the Linux release wants answered is whether that fallback is a supported
configuration or a documented non-target.

⚠️ Do not repair this by measuring on Metal — no local gate sees the axis, and the number
above is from one arm64 container with Mesa 22.3.6, not from CI's x86_64 lavapipe. Any
claim about "software rendering performance" needs its configuration stated, per the
standing rule that a check runs in one configuration and that configuration is itself an
untested hypothesis.

**Shape of the work, so a lane does not start tuning.** The FIRST deliverable is a profile,
not a patch: where do those 82-184 ms actually go, per world, on the software path? Until that
exists, every optimisation is a guess, and this codebase's own history says a bench that does
not witness the work will happily measure nothing — one theme bench "measured" 5 ms while no
reshape happened at all. So make the profile witness the frame, and report the breakdown
before proposing a change.

**"Documented non-target" is a legitimate answer and may be the right one.** An 84x gap that
is uniform across every world is not a hot spot; it is the cost of the whole render meeting a
rasteriser with no GPU under it. If the profile says that, the honest outcome is a
RELEASING.md/WEB.md sentence naming software rendering as unsupported and saying what a user
sees when they land on it — not a speculative optimisation pass. That is a product call and
belongs to the user; bring them the profile and the two options rather than a patch.

Whatever is measured, state the configuration in the same breath as the number: which
rasteriser, which Mesa, which architecture. The figure above is one arm64 container with Mesa
22.3.6 and is NOT CI's x86_64 lavapipe.

---

### 582 — Kite tunnel visual correction: restore the approved bending, folded 3D surface (user report + decision, 2026-09-06)

⬜ READY — corrective follow-up to 564. Queue only; not dispatched.

**Outcome.** The user rejected the delivered appearance: regular concentric
circles and straight spokes, unlike the approved organic tunnel prototype.
Restore the prototype's depth-dependent bending, section rotation, and folded
silhouette in the shared background renderer. Matching parameter labels is not
visual parity. Item 564's engineering completion does not constitute acceptance
of its appearance; this item owns the correction and renewed visual sign-off.

**Evidence and first check.** Source comparison finds the saved prototype
projects individual 3D tube points with depth-dependent centre displacement,
radius and roll. Current `shaders/background.wgsl` instead uses a single polar
axis, adds a bounded shift to a logarithmic ring coordinate, tapers folds back
to circles outside a depth band, and derives rails directly from `theta`.
These are substantive geometric differences. First reproduce with a fresh
native capture and explicit config; record build and motion policy so an old
running binary cannot be mistaken for current source. The user's screenshot
contains private prose: do not copy it into tracked fixtures or reports.

**Reference.** Existing approved standalone study (read-only reference, no new
web artifact): `$HOME/.codex/visualizations/2026/08/18/01a01547-26ee-7cf3-be19-203fc0c69a13/living-tunnel-study.html`,
especially `tunnelPoint`, its sampling loops, and motion update. Read the actual
file before implementing. Preserve reference evidence outside the repo; capture
only seeded public prose for any tracked native comparison. The core mapping is
recorded here so the brief remains useful if that local artifact is unavailable:

```text
turn = theta + worldZ * twist
pulse = 1 + .075*sin(worldZ*1.25) + .035*sin(worldZ*2.7 + theta*2)
radius = max(.46, 1 + fold*(.46*cos(3*turn)
                          + .18*sin(5*turn - worldZ*.35))) * pulse
pathX = .22*sin(worldZ*.48) + .07*sin(worldZ*1.17)
pathY = .17*cos(worldZ*.39) - .06*sin(worldZ*.91)
angle = theta + .12*sin(worldZ*.31) + spin
x3 = pathX + radius*cos(angle)
y3 = pathY + radius*sin(angle)
scale = min(width,height)*.72 / z
p = clamp((z-.72)/(10.8-.72), 0, 1)
bend = p*p*(3-2*p)
centre = viewportCentre + (vanishingPoint-viewportCentre)*bend
screenPoint = centre + (x3,y3)*scale
```

**Build / scope.** Preserve the resulting projected surface, including curved
longitudinal ribs, displaced sections, and visible folds/overlap where the
reference has them. One continuous tube can have a centreline that bends with
depth: continuity does not require every section to share one screen centre.
Choose a bounded reusable rendering mechanism appropriate to that geometry;
do not force it into the existing closed-form polar approximation if that loses
the shape. Keep theme choices as data, one motion owner, and native/browser
support. No Kite-name branch, separate per-margin tunnels, or general scene
editor. Revisit laws that enforce straight radial rails or concentricity: those
properties describe the rejected implementation, not the product contract.

Retain the approved light lavender/mineral palette, calm central page, fold .34,
twist .72, forward drift .05, and 58 ribs as visual reference settings. Any
necessary parameter/unit conversion must preserve appearance and be explained.
Retain random corner targets with no immediate repeat, 15-second dwell,
12-second smooth transit, and very slow section roll (study spin rate
`twist*.035` per second). Retain subtle convergence haze without orb/crosshair.
Handle dense distant lines with antialiasing and depth/contrast fading while
preserving the near-field surface; flattening the tube is not the fallback.
Reduce Motion and Ambient-off use a deterministic authored static folded pose;
ordinary pause/lost focus freeze the current pose. Static mode must keep the
same characteristic geometry. No new user-facing controls are needed.

**Done / verify.** Read THEMES.md, docs/render.md, CAPTURE.md and
docs/harness-reach.md before implementation. Produce matched reference/native
views at recorded viewport, page width, pose and settings: top-right and
bottom-left dwell, a transit midpoint, and the motion-safe pose; include both
1200×800 and 1600×1000 and DPI 1/2. Native captures use an explicit hermetic
fixture/config. Use the existing deterministic motion seam where it reaches
these states; extend the shared seam if necessary rather than inventing a
parallel renderer for tests. Sidecars verify state; pixels verify visible
curvature, fold presence, page legibility and continuity across page masking.
Compare projected landmarks/curves against the reference geometry with an
explicit tolerance; a nonzero fold uniform alone proves nothing. Add a law
that fails on the current concentric-ring/straight-spoke approximation and
prove it fails after the mutation builds and runs. Sweep narrow/wide pages,
corner/transit poses, and static/moving states for crowding and aliasing.

Perform the standing vision smoke over about five real gallery shots with
concrete questions about curved ribs and folded sections; obtain a visual
judge's reference comparison before declaring visual parity. Keep real-time
comfort and final resemblance acceptance explicitly OWED to the user; present
the comparison images, not just test receipts. Measure release rendering cost
before/after, preserve bounded wake/freeze behavior, and run native gate and
web smoke for the implementation. A passed engineering gate cannot close the
user's visual rejection by itself.

Worker routing when dispatched: `gpt-5.6-sol` high for geometry/implementation;
visual judge `gpt-5.6-sol` xhigh; outcome audit `gpt-5.6-terra` medium. Follow the
board's claim/worktree protocol and integrate serially with overlapping shader
work. This brief authorizes the correction, not unrelated background redesigns.

---

### 589 — Commands and shared transient chrome: clearer controls within each world's composition (user decision, 2026-09-07)

Coordination update (2026-09-08): **628 owns the shared visual specification and
prototype review; follow its sequencing before overlapping visual implementation.**

⬜ READY — queue only; not dispatched. Shared design foundation for 590–592;
integrate overlapping renderer work serially.

**Decision.** Improve Commands' query/result hierarchy and spacing while keeping
the document readable outside the summoned surface. Integrate its title with
its controls rather than letting a remote oversized label dominate the task.
Preserve categories, bindings, keyboard selection and each world's authored
placement; the generated upper-right mockup is not a universal anchor.

**Shared scope.** Consistency is WITHIN each world, not one skin across worlds.
Find, Link and Commands share that world's surface colours, border weight,
corner rules, field grammar and spacing. Nested controls derive compatible
corners; do not invent feature-specific radii. Preserve Pane, Bars, Diagonal
and Ruled compositions: no compulsory rounded enclosing panel for plate/rule
worlds. Carry relevant improvements through sibling pickers/prompts via shared
owners. The user's preferred Find/Replace chrome is preserved in
`references/find-replace-chrome.png` beside this board; it guides bordered
surfaces, not every world's visual identity.

**Separation / verify.** Brief choices should retain readable surrounding prose.
Use the world's backing, retaining local frost where needed; this is NOT a
global blur-off instruction. Unbacked text must not overlap document ink.
Read DESIGN.md, docs/render.md and docs/harness-reach.md. Audit the actual
surface × composition × placement roster, narrow/wide and DPI 1/2; assert
geometry/state and pixel legibility, add mutation-proven laws and the standing
five-shot vision smoke. Keep anchor stability and keyboard behavior intact.

---

## Two orchestrators share this board — renumber yourself, never the other

A second orchestrator session works this board. On 2026-09-07 both queued items in the same
window and **both used 605 and 606**, because this session appended by number without
re-reading a board that had moved under it. Theirs landed first (`35177829`); mine were the
duplicates and mine were renumbered to 612 and 613, along with the one cross-reference that
pointed at the wrong 605.

The rule that prevents it: **re-read the board immediately before choosing an item number, and
if two numbers collide, the LATER writer renumbers.** Their commit is the tiebreak, not
seniority and not who noticed. Fix the cross-references in the same commit — a renumber that
leaves a stale pointer is worse than the collision, because the pointer still resolves to a
real item and reads as deliberate.

Related and cheaper: this session also spent several turns listing questions the user had
ALREADY answered through the other session — 603, 568, 570's placement, 576's gestures and
572's four taste calls were all decided in `35177829` while this one was mid-wave. Read the
board's own Owed section before telling the user what they owe you.

## GATED AND PUSHED — and the CI run nobody read

Resolved. `main` is receipted and pushed through `8f7c628f`, covering 596, 597, 598, 602, 595
and 604. The exact-main gate ran with HEAD verified unmoved end to end:

```
native-gate-receipt commit=8f7c628f health=pass:280s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=4979 unit_shards=6 integration_targets=18
```
plus `web-smoke: OK`. The two-writer worry that held this back was overstated: the gate
happened to catch a quiet window, and the user's answer was to push.

⚠️ **What actually cost something was not the missing receipt — it was a CI run this session
never opened.** The push before it (`437e6280`, carrying 600/601) failed the GATING
`linux (build + test)` job, and this session recorded the wave as landed without reading the
result. **A push is not finished when it succeeds; it is finished when its run is read.** The
concurrency setting makes that worse, not better: pushing over an in-flight run cancels it, so
a run can vanish without ever having been looked at.

The failure itself is recorded under the ambient-environment entry in CLAUDE.md's principles
and fixed in `d57f1d93`. In one line: `disk-preflight.sh` answers a CI environment on a branch
of its own, every hosted runner exports `CI`, and `test-disk-preflight.sh` let its fleet laws
inherit that variable — so each half of the suite ran in exactly one place and never the
other. Green here, red there, for the whole life of the law.

## Owed to the user — landed work awaiting a live eye

**588 — Brolga has no working bullet triple, and this is a real choice, not a defect.** The
lane swept `bullet_scale` from 0.55 to 0.95 and found NO value where all three Dovecote
members clear the 2.5:1 contrast floor AND stay clear of the following text: under about 0.68
at least one dove falls below the floor, and by about 0.75 — where contrast finally clears —
the widest dove fills the fixed-width bullet box edge to edge. Brolga therefore keeps the
plain `•◦▪`, documented in the world, beside the scale constant, and in the law's exception
list.

The two ways out are both mechanism changes and neither is this item's: **a different single
glyph for Brolga**, or **a wider bullet box**. A twenty-world gallery and contact sheet are at
`/tmp/claude-588-gallery/` for the taste pass — the harness proved legibility, distinctness
and derivation, and none of those is taste.

These items have MERGED and left the build queue. Each one still owes the user an answer or
a live look, which landing does not discharge. Full context is in
`git log -p -- .orchestrator/queue.md`.

**584 / 626 — live macOS text access confirmed 2026-09-08; 626 closed as
“premise false, oracle repaired”.** On running build `dbd4d2f5c415` (contains
`27aa13fa`), the user launched a trusted, read-only native AX recorder from Terminal;
CUA drove the actual app. Successful `AXStringForRange` / character-count responses
followed original document (12) → seeded alpha (39, including café and newlines) →
beta (40, distinct text) → alpha (39) → New (0, empty). A subsequent test keystroke
and undo also appeared as 1 → 0; returning to the original document restored its
exact initial 12-character text. No stale document text reproduced at the OS seam.
The missing AXValue is expected for this multiline AccessKit node; CUA's inability
to read/select through its own helper was not proof of a product defect. No product
fix or native-suite receipt is claimed. VoiceOver speech/announcement quality is
UNTESTED and explicitly deferred by the user, who asked to skip that sitting; do
not keep asking for it. This bounded switch/New check does not claim an exhaustive
selection-write, Unicode, or reactivation audit. 583's pause-then-type journey was
separately confirmed by the user on 2026-09-08.

**553 — folder-wide search (merged `277c3717`, follow-ups `e076ddd8`/`104fb174`).** The match
highlight's real-pixel legibility is live-only and unverified. Also flagged, not hidden:
grouping does not use the lens-strip header mechanism (a deliberate scope call); a CRLF
source file's matched line keeps a cosmetic trailing `\r`; and the corpus is summon-time
only, like Assets and Go to — a file edited on disk while the picker stays open is not
re-read until the next summon.

**561 — answered 2026-09-08: proportionate, a little tall; 618 takes it down ~15%.** Original note kept for the star/underscore caveat, which 618 inherits. Ornament scale equalized upward (merged `5f90cb6d`, follow-ups `1b22a1c1`/`fd2f5894`).**
Gumtree's dash is a 4-glyph snake run, so equalizing its height also grew its width (~119px →
~252px against a 1008px column); it reads proportionate in capture, unconfirmed live.
Unmeasured: star and underscore share one `ornament_scale` dial with dash, so they grew
proportionally without being checked against their own ink-to-em ratios.

**564 — Kite's living warped-grid tunnel (merged `c3c3032e`, cleanup `002f09fe`; pushed). REJECTED AGAIN LIVE, 2026-09-08:** the user looked and said "Kite is still really wrong" — nothing has changed since 582 was written, because 582 is queued and NOT dispatched. The next dispatch wave should take 582 ahead of new taste work; the user asked whether it was finished and the answer is no.
Live human sign-off is owed for the several-minute drift and contortion feel — the harness
verifies single-frame trajectories and the motion-safe still, not wall-clock feel over
minutes. Also owed: at the default 1200×800 capture geometry the roaming vanishing point can
land closer to the page edge than at the 1600×1000 geometry the pixel laws sweep, so it is
worth a live look at whether the convergence ever reads as landing inside the page itself at
common window sizes rather than staying a margin phenomenon. Item 582 (open, above) revises
this ground's geometry and inherits the same sign-off.

---

## Green train — the exact-main receipts

**Eleventh train, `98254e7a` — PUSHED as `115aad53..98254e7a`.** Covers 631 and 629.

```
native-gate-receipt commit=98254e7a health=pass:277s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5108 unit_shards=6 integration_targets=18
```
plus `web-smoke: OK`.

629's numbers, on a host verified quiet before each run: typing_live median 7.960ms to 6.859ms
on the 50,029-word plain-prose tier (p90 8.412 to 6.910), and 10.064ms to 8.758ms on heavy
markdown. Tiers the change cannot help are flat within noise. The counters tie those to work:
one line retokenized per keystroke on the prose tier, and exactly the document's own line count
per keystroke on the ineligible code tier.

631's own measurement deflated its tool honestly — preflight is 290s against the 284s the gate
already spends on the same `code-health.sh`, so it is not a lighter check, only that signal
isolable from the GPU and full-suite work after it. Its gate rehearsal ran concurrently with
629's test suite and incidentally validated the cpu-spin oracle rewritten today: ground truth
92.0% of a core against a heartbeat reading 102.3%.


**Tenth train, `d2a45d5e` — PUSHED as `4059591c..d2a45d5e`, 17 commits.** Covers 537, 632, 588.

```
native-gate-receipt commit=d2a45d5e health=pass:284s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5100 unit_shards=6 integration_targets=18
```
plus `web-smoke: OK`.

Two failures the train caught that no worker gate would have, both worth naming because they
argue for where the full suite belongs.

**537's sixteenth settings toggle** turned two roster sweeps red — sweeps whose own message is
"the toggle roster changed size, update this sweep deliberately". They live in
`app::tests::files` and `actions::tests::overlay_drive`, and no reasonable reading of a
footnote-ladder diff selects those modules. A census reached from an unrelated module is
exactly what only a whole-suite run finds.

**607's follow-override note** (previous train) had no fate in the println audit — a
whole-tree census, invisible to any branch-local run by construction.

Under the retired policy both would have been found by a per-branch gate at the cost of one
serialized full gate per lane; under the current one they were found once, at the point where
the tree they share actually exists.


**Ninth train, `4059591c` — PUSHED as `07959082..4059591c`, 18 commits.**

```
native-gate-receipt commit=4059591c health=pass:294s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5089 unit_shards=6 integration_targets=18
```
plus `web-smoke: OK`. Covers 623, 607, the pointer-roster extraction, 627 and 616.

First train gated under the new verification policy: one gate on the integrated candidate
rather than a full gate per worker branch. Five lanes delivered targeted evidence instead, and
the single gate found the one thing they could not — 607's `[keys] follow` note had no fate in
the println audit, which is a whole-tree census no branch-local run reaches.

The eighth train's CI (run 34249325854) passed all four gating jobs, including both hosted-mac
arms — the only place the virtualised-GPU axis is ever exercised.


**Eighth train, `07959082` — PUSHED as `127ab8b1..07959082`, 103 commits.**

```
native-gate-receipt commit=07959082 health=pass:303s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5074 unit_shards=6 integration_targets=18
```
plus `web-smoke: OK`, HEAD verified unmoved across the run.

This one is unusual in what it covers. The push had been blocked for hours on a token scope —
577's merge touches `.github/workflows/ci.yml` and the session's OAuth token carried no
`workflow` scope — so the backlog grew to 103 commits, of which roughly fifty were the peer
session's merges carrying no receipt of their own. The last receipt recorded on this board
before it was the seventh train at `984a9975`. So this receipt is the first end-to-end
verification that stretch has had.

A machine reboot in the middle destroyed every earlier gate log in `/tmp`, which is why the
seventh train's evidence cannot be re-read: a receipt is a claim about a commit, and the
commit survives, but the transcript proving it does not. Worth knowing before anyone tries to
audit an older train from its log.


**Seventh train, `984a9975`** — covers 608, 600, 577, 611, 618, 590, 609, 606, 605+617 and
619, HEAD verified unmoved across the run.

```
native-gate-receipt commit=984a9975 health=pass:345s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5007 unit_shards=6 integration_targets=18
```

⚠️ **NOT PUSHED, and not for any reason in the tree.** GitHub refuses the push because 577's
merge touches `.github/workflows/ci.yml` and this session's token carries no `workflow` scope.
That is a credential scope on the user's own account: `gh auth refresh -h github.com -s
workflow`. Nearly fifty commits wait behind it.

Two items were REVERTED out of this train rather than shipped: 591 (a sidecar publishing a
card rim 560 units from where it draws, plus a summoned-layer bypass in mouse.rs) and 592 (a
menu-bar-axis regression in the range rail). Both branches are intact and both are back with
their lanes. **Both were merged un-gated** because this orchestrator told every lane to skip
the full gate so the train could hold the capacity=1 arbiter. That protocol bought arbiter
time and cost two defects reaching main and two gate cycles removing them. It is not obviously
a bad trade at eleven lanes, but it is a trade, and it should be made deliberately rather than
inherited.


**Sixth train, `8f7c628f`** — covers 596, 597, 598, 602, 595 and 604, HEAD verified unmoved
across the run. `health=pass:280s unit_tests=4979`, web-smoke OK. **CI run 34173463828 was
cancelled by the push that followed it** (`concurrency.cancel-in-progress`), so this train's
own CI verdict does not exist; `d57f1d93` on top of it carries the linux fix and is the run to
read. Recorded rather than quietly inherited from the local receipt.

⚠️ **The train before this one, `437e6280`, was pushed and its CI never read. It was RED** on
`linux (build + test)`. See the section above.


**Fifth train, `a7076b32`** — covers 572, HEAD verified unmoved across the run. Pushed as
`2ce630d5`; **CI run 34076734681 passed all four gating jobs** (39 min wall, the linux job the
long pole at 38m54s — in line with the ~37-minute warm baseline 566 established).


```
native-gate-receipt commit=a7076b323c8ac462399edbb71789b7269ad85887 health=pass:247s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=4964 unit_shards=6
  integration_targets=18
```
plus `web-smoke: OK`. No mark raised — the branch LOWERED `render/geometry.rs` to 1323 after
`caret_band` moved into its own module.

**Fourth train, `0e195574`** — covers 580 and 581, HEAD verified unmoved across the run.
Pushed as `297ed802`; **CI run 34072883296 passed all four gating jobs.**


```
native-gate-receipt commit=0e19557466c341138fbc5e7d87295f4e00947020 health=pass:249s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=4953 unit_shards=6
  integration_targets=18
```
plus `web-smoke: OK`. No marks raised — 580's census closed a bypass by making four mutators
module-private, and 581 split `projection.rs` at the ceiling rather than asking for room.

**Third train, `555fa5d6`** — covered 570, 558 and 576. `health=pass:254s unit_tests=4946`,
web-smoke OK. Pushed as `afda18f4`; CI run 34062997740 passed all four gating jobs.

**Second train, `72e922e1`** — covered 583/584 and 585. `health=pass:251s unit_tests=4917`,
web-smoke OK. Pushed as `c3d26d08`; CI run 34050443205 passed all four gating jobs.

**First train, `5d4819e3`** — covered 571/573, 567, 568/569 and 586/587. `health=pass:271s
unit_tests=4903`, web-smoke OK. Pushed as `a7ad4c68`; CI run 34047161907 passed all four
gating jobs, including the hosted-mac pair — the only arm that has ever seen the
virtualised-GPU axis, and therefore the half of the verification no local receipt supplies.

⚠️ **Hardware bound, restated because a green receipt is exactly when it gets forgotten:** a
local receipt certifies the dev host's real Apple Silicon Metal. A wedge once stayed green
here while red on hosted macOS for ~140 commits, and CI's lavapipe job stayed green through
that entire streak, so a software adapter is not a stand-in for that axis.

⚠️ **No receipt covers a live journey.** Five items merged this wave with live confirmation
explicitly NOT obtained, because the display was locked; they are in the owed section rather
than silently absorbed into a green line.

## Watch — verification that only a future run can supply

**566's oracle: ANSWERED 2026-09-07, and the wiring works.** The item asked whether the linux
job's `native-gate-env` line would read `budget_source=deadline` rather than
`budget_source=none`, because nothing local can test the `$GITHUB_ENV` hop. Read out of run
34039686854's own linux log:

```
native-gate-env cpus=4 mem_bytes=16766414848 conventions=2 test_threads=2
  budget_seconds=3686 budget_source=deadline deadline_epoch=1788709583
```

`budget_source=deadline`, a real 61-minute budget, and `linux (build + test)` green. The
runner death clock is armed, so an over-run now ends as a readable FAILURE instead of a
cancellation that verifies nothing and discards the cold `target/`. Nothing further is owed
here; 566 is closed.

## Needs specific hardware

🔴 BLOCKED — these journeys require physical environments unavailable to the current orchestration host.

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

🔴 BLOCKED on the user's release word only. The user reports (2026-09-08) that Apple signing and notarisation are set up and have shipped a release already — tags `v0.9.0` through `v0.12.0` exist — so the secrets line is retired. What remains: every tag waits for the user's explicit word.
