# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**10 open numbered tasks.** Ready: 641 and 637. Analysis complete: 642.
Dependencies/coordination: 638–640 and 589. Native prototype/candidate review:
628 and 582. User decision: 579.
Outstanding review of landed work and hardware checks are listed separately below.

### 641 — picker-specific construction inputs (user request, 2026-09-10)

🟢 READY — queue only, not dispatched. Behavior-preserving ownership refactor.

Build: replace the catch-all `overlay::BuildCtx` with focused input types so each
picker receives only the data it consumes. Start by mapping the current
`overlay/build/ctx.rs` fields to `overlay/build.rs` consumers and the live App,
replay, tests and benchmark construction sites. Choose typed constructors or a
request enum carrying per-picker inputs; avoid a picker kind plus an unrelated
payload, or another all-purpose bag hidden behind defaults. Share genuinely common
binding/config inputs through one small owner. Preserve the single construction
path shared by live and replay, including deliberate differences in recency,
history clocks, dictionary and filesystem data. Keep absent spell-target behavior,
row gates, ranking, labels, search budgets and navigable-explorer routing unchanged.

Done: production callers cannot supply irrelevant picker inputs or need to fill
unrelated fields with empty values. Remove redundant gathering and explanatory
bookkeeping where the new types make it unnecessary; retain comments about units,
lifetimes and real invariants. Do not introduce eager I/O, extra copying, new
picker behavior or a generic construction framework. No speedup is claimed without
release measurements at the affected work owner.

Coordination: separate from 615's mouse extraction. Integrate overlapping changes
serially with 637–639 and 628/589; rebase on their actual current state rather than
freezing the old Go-to shape or changing the approved UI contract. This refactor is
not an additional product-design approval gate.

Verify: read docs/render.md and docs/verification.md; enumerate every picker variant
with exhaustive handling, update all construction sites and source-law enrollment,
and compare observable construction results against base (rows/order, selection,
labels, gates, missing inputs and live/replay differences). Run targeted picker,
action and replay laws plus compiler checks for native and wasm consumers. Follow
any identity comparison with an outcome audit; repair missing laws for findings.
Use the verification policy's single integrated native/wasm gate, not duplicate
full worker gates. Read docs/harness-reach.md before choosing any render captures.

---

### 642 — animation state owns complete transitions (user request, 2026-09-10)

🟡 ANALYSIS COMPLETE — `animation_642_map` (Codex) mapped the exact private
overlay entrance/band boundary, live input-epoch bridge, scheduler boundary,
transition matrix, target laws and compiling close/reopen mutation. No code was
changed. Implementation remains serialized behind 637 and shared UI work.

Build: census related animation fields and their writers, starting with the
TextPipeline overlay entrance/selection-band fields in render.rs and their input,
prepare, advance and dismissal paths. Group fields by the animation whose invariant
they share, with private state and named shared transitions for start, retarget,
advance, settle and reset as applicable. Migrate every live, replay and test caller
so callers cannot reset the phase while leaving an old origin, target, pending
input epoch or active flag behind. Existing cohesive animation owners stay intact;
do not combine unrelated animations into one global state machine or generic engine.

Preserve authored curves, durations, retarget behavior, first-open/close behavior,
preview commit/revert and Reduce Motion endpoints. Preserve FrameSample's shared
presentation time, input-epoch accounting, pause/occlusion/failed-present semantics,
post-prepare activity reporting and clock-free capture deltas. This changes state
ownership, not animation feel or redraw policy. Reduce bookkeeping/comments only
where the type and transitions now express the invariant; retain meaningful units
and temporal contracts. Runtime performance claims require release before/after
measurements, not fewer fields or lines.

Coordination: separate from 615; serialize edits to shared callers with that branch
and the UI/picker work in 628/637–639. Reconcile against the current frame-clock and
theme-preview owners before implementation. No dependence on 641 unless the actual
call graph exposes one; do not require both refactors to land in one large change.

Verify: map animation state × transition × input sequence, including interrupted
retargets, repeated resets, close/reopen, buffer/world changes and Reduce Motion
mid-animation. Use injected time to compare poses and activity against base at
start, intermediate and settled samples; test stale pending-state removal and
prove a headline partial-reset regression law fails under a compiling mutation.
Run targeted motion/frame-clock/scheduling laws and an independent outcome audit;
read docs/render.md, docs/verification.md and docs/harness-reach.md before selecting
render probes. Apply the required render vision-smoke when rendering is touched,
then one integrated native/wasm gate. Deterministic state/pose checks do not verify
wall-clock feel; identify any remaining live timing checks honestly.

---

### 637 — implement the folder-to-file-to-writing journey (user approval, 2026-09-10)

🟡 IN PROGRESS — `navigation_637` (Codex), branch `codex/637-files`.
Coordinate renderer changes with 628; preserve its shared visual review requirement
rather than reopening this approved navigation direction.

Build: dedicated Files / Recent navigation in the existing summoned-surface
system. Files with an empty query shows immediate subfolders and files; folder
rows have a folder affordance and trailing disclosure and descend inside the
same browser without changing the writing root. Breadcrumbs/Up expose the current
location. Typing searches names/paths throughout the explicitly named root,
including descendants; show root-relative paths for results and duplicate names.
Clearing search restores the browse location. Recent contains recently opened
files in the named root, not folder-switch destinations. Preserve useful selection
and scroll state through focus changes; never reorder the open-document stack by MRU.

Open folder/Change folder is explicit root selection: macOS keeps its system chooser,
Linux its internal chooser. Success immediately shows that root's Files contents;
cancellation/failure preserves valid prior context. Folder browsing, root selection,
and file activation must have separate state transitions. Loading/partial search,
no matches, truly empty, unsupported-only, and unavailable/permission-denied are
distinct states. Use short existing-surface status/empty treatments, not large
cards, duplicate New actions, or success copy telling an empty folder to choose a file.

Connect home, Welcome's entry point, and the bottom-left folder heading to the same
Files owner. Keep the stack's placement, stable order, active mark, close/drag
behavior, and per-file remembered roots; it lists only open documents. A folder
heading needs a discoverable affordance and matching hit target. Choosing a file
dismisses Files and opens/activates its existing buffer, preserving edits/undo and
position. No arbitrary file is opened merely because a folder was chosen.

New document and its actual bound shortcut name the destination. While searching
or viewing Recent, the destination is explicitly the root; while browsing it is
the displayed directory. Home without a root creates recoverable unsaved work.
Preserve existing files' save paths and autosave/recovery guarantees. Implement
the focus/arrow/Tab rules from 636, including action-level menu interception.
Align menu, palette, home, hints and user docs on Files terminology while keeping
intentional legacy config keys/aliases functional. Keep heading/line actions reachable.

Verify: pure transition laws first, then seeded --screenshot-app journeys with
--seed-tree and explicit config: home → folder → nested file → edit → another
file → original buffer; root A/B switch and cancellation; search/clear; duplicate
names; long paths; empty/missing/read-only roots; New document through key AND menu
Action. State proofs include root, buffer identity, save destination, undo, selection
and scroll. Native chooser and pointer timing need real-window checks; do not claim
headless keys exercise AppKit key equivalents. Capture real geometry/pixel presence
across worlds, narrow/wide and DPI 1/2 with five-shot vision smoke and mutation-proven
headline laws. Targeted checks here; integrated gate belongs to 640.

---

### 638 — one location-navigation owner for Open, Move, Save As and Export (user approval, 2026-09-10)

🟡 DEPENDS ON 636 and 637 — queue only, not dispatched. Integrate overlapping
navigation/render ownership sequentially, not as independent browser rewrites.

Build: reuse the agreed folder-row, location/breadcrumb, scope, focus, search and
back-navigation rules wherever awl chooses a location. Inspect existing MoveDest,
ExportDest, Browse, ProjectBrowse and save/copy platform routes first; converge
their common location behavior at one owner. Keep platform-native dialogs where
they already own the interaction. The operation's typed purpose owns the final
action and side effects: Open file, Move here, Save here, Export here. Do not make
a navigation-row activation move/save/export before explicit commitment.

Make source document and destination unambiguous. Preserve rename/overwrite
confirmation, read-only handling, extension/format policy, recovery and original
file identity for copy/export. Retain operation-specific constraints rather than
forcing every chooser into file-opening semantics. No new filesystem manager,
general-purpose dialog framework, or public filesystem writes for demo purposes.

Verify: a roster-derived test enumerates every location consumer and its commit
verb. Exercise browse/back/search/cancel and valid/invalid destination per operation
in hermetic fixtures. Cancellation must perform no file operation; commit changes
only the promised files. Mutation proof must catch a bypass or premature commit.
Check Mac native boundaries and Linux fallback separately, and report live-only
coverage honestly. Use shared chrome checks and targeted operation tests; 640 owns
the integrated gate.

---

### 639 — UI coherence across Commands, search, Settings, previews and History (user approval, 2026-09-10)

🟡 COORDINATE WITH 637 and 628/589 — queue only, not dispatched. The 636
interaction contract is now in DESIGN.md and docs/render.md; use 637 as the native
navigation reference rather than launching another shared-chrome rewrite.

Build: apply context → query/views → choices → actions across the actual surface
roster, preserving task-specific composition. Commands remains actions, the outline
remains document navigation, Settings remains behavior, History remains changes
to the named document. Make scope visible when an action depends on it: Search in
Writing, Move September.md to…, Export September.md, History of September.md.
Use readable names and disambiguating paths without repeating the same location
throughout a panel. Document search and folder-content search retain distinct scope;
filename/path search does not silently become content search.

Use 636's common focus/action grammar, not identical key meanings in hidden focus
states. Row selection, keyboard focus and pointer hover remain visibly different.
Footers/buttons announce the actual action and real rebound/platform keys. Unify
labels and routing across keyboard, menu, context menu and palette through existing
Action owners. Preserve focus/editor restoration and accessible roles/states.

Explicitly distinguish selection from preview and commitment: Files opens only on
accept; world/caret movement can preview, accept keeps and cancel restores; History
selection can compare without restoring document bytes. Preserve existing immediate
Settings semantics unless a separately approved decision changes them. Esc is not
a blanket rollback of already committed settings or a successful root selection.

Visual ownership stays with 628/589: shared hierarchy, controls, nearby label/value
relationships, meaningful states and legibility within each world's face, palette,
placement, material and motion. Keep Settings/History's sustained-workspace structure;
do not shrink them to the Files picker or copy the HTML mock's warm skin everywhere.

Verify: enumerate surface × focus region × action × scope/preview state from the
production roster. Test keys and direct Actions, canceled versus committed changes,
search scoping, narrow-layout focus and restoration. Reuse shared native render
probes with selected/focused/hovered states and the standing world/DPI/pixel checks;
five-shot vision smoke asks concrete affordance questions. Findings get missing law
tests. Do not duplicate 628's prototype work or relabel its pending review as passed.

---

### 640 — integrated navigation/coherence acceptance and documentation (user approval, 2026-09-10)

🟡 DEPENDS ON 637–639 and their relevant 628/589 integration — queue only, not dispatched.

Build: review the combined native experience as one journey, remove obsolete parallel
entry points/contradictory teaching, and update the existing contracts, GUIDE,
Welcome/tour, keybinding reference and accessibility documentation to verified behavior.
Keep stable user config compatibility deliberate; no silent binding migrations.
The approved study is a design reference, not a shipped-behavior or performance receipt.

Acceptance journey: no-document/Welcome → choose writing root → Files → enter
subfolder → search/clear → open/edit → switch open documents → reopen/cancel → New
document via button and actual shortcut → Move/Save a Copy/Export in hermetic paths →
folder-content search → Settings → world preview/cancel → History compare/cancel.
Include two roots, duplicate names, unsaved edits, unavailable/empty folders, narrow
windows, and focus moved away from a still-selected row. Verify that path ownership,
buffer identity and save behavior remain understandable throughout. Include a Mac
menu-key-equivalent journey and real Linux fallback coverage; headless keys alone
do not certify either. Preserve the screen-lock checks at both ends of live runs.

Verify: follow docs/verification.md: cheap/targeted checks in each owning item, outcome
audits and mutation proofs on the integrated candidate, then one full native gate
and web smoke after commit/freeze. Use seeded captures, world/geometry/DPI coverage,
pixel presence and legibility, and the standing five-shot vision smoke. Validate
supported web behavior without claiming desktop chooser parity. Distinguish live
feel/taste still owed to the user from mechanically proven state and geometry.
Do not dispatch or claim implementation merely because this acceptance work is queued.

---

### 628 — one shared chrome language for Find, Settings and the theme picker (user approval, 2026-09-08)

🟡 IMPLEMENTATION ANALYSIS IN PROGRESS — `chrome_628_map` (Codex), read-only
against current `main`, the approved direction and native prototype. The prototype
exists on `item-628` at `e37a13ba` and is not integrated. This is not implementation
acceptance; native edits remain serialized behind 637.

Review update (2026-09-10): the user likes the saved Settings layout and explicitly
asked to continue Find/Replace and theme-picker refinement on the interactive design
site alongside it. Preserve the bounded Settings label/value column and category
rail. The site extends the approved Files study with separated find/navigation/
replacement groups and stable theme preview; the Switch/Cancel refinement below supersedes Keep/Cancel. This HTML
review is explicitly user-authorized for this study; it does not replace the native
composition, accessibility, world/DPI checks or final taste review below.
Reference: https://awl-files-reconsidered.s84fzrm6tq.chatgpt.site/ (study 05; earlier study 04 reviewed the initial composition).

Approved theme-picker refinement (2026-09-11; study 05): use **Themes** in
user-facing labels, not Worlds. Keep the same top-right placement, size and layout
across all themes at a given viewport/UI scale, including when reopened under a
different theme. Narrow windows use one shared centered layout. This is stronger
than merely freezing the opening theme's anchor during a preview. Select to preview;
**Switch** / Enter commits and closes, **Cancel** / Esc restores the prior theme.
Keep the quieter, opaque bordered chrome and hold its appearance steady during
preview. Remove “Preview on the page”, the explanatory chooser subtitle, palette
blurbs and redundant Keep wording. Remove background/partial blur from these
reviewed picker surfaces; keep the document sharp behind them. This does not remove
unrelated authored theme backgrounds or motion. Use actual effective shortcut labels.

Verify the placement contract across opening themes as well as preview destinations,
normal/narrow viewports and UI scale/DPI. Check Switch/Cancel through keyboard,
pointer and direct Actions, with focus restored to the invoking surface (including
Settings). The approved direction is queued for native implementation; it does not
mark the existing native prototype integrated or its outstanding checks passed.

The native prototype report flags an existing workspace-width law as failing and
leaves theme composition untouched. Re-measure against current main before integration;
prototype captures are neither a gate receipt nor evidence that the running app has
these changes. Continue through 636/639's interaction contract without duplicating
this item's visual ownership. No native implementation dispatched by this review.

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
   stable panel composition and cross-theme placement per the approved refinement above.
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

### 579 — software rendering: profiled, and the answer is a product call (investigation complete, 2026-09-09)

🟢 **PROFILED — awaiting the user's decision. No code change is proposed and none should be
until this is answered.**

The profile settles the engineering question. On the cited configuration — arm64, Debian 12,
Mesa 22.3.6, `llvmpipe (LLVM 15.0.6, 128 bits)`, `PHYSICAL_DEVICE_TYPE_CPU`, reproduced in a
pre-existing rig rather than a new one — every sample at every contention level has the same
shape:

```
   queue.submit + device.poll |  81.296 ms | 98.9%
   22 other CPU-side stages   |   ~0.7 ms  | <1%
   TOTAL (median frame)       |  82.164 ms
```

**There is no hot spot in awl's code.** Text shaping, layout, ornaments, table grid, chrome,
spell squiggles and render encode together cost under 1ms per frame, under 1.5% of the frame
even in the cleanest run. Over 99% sits inside Mesa's own rasterisation of already-encoded draw
calls. Document size barely matters — 1943 lines and 124 lines cost nearly the same — which is
the O(visible) principle holding.

**The item's own per-world spread did NOT reproduce.** The cited 82→184ms range across worlds
became four worlds within 2ms of each other at ~83ms, including the originally cited fastest
and slowest. The lane could not tell whether the original spread is contention-sensitive or
whether its own best window (load ~45-60, never this host's ~5 idle) flattened real differences
toward a floor. Named as an open gap rather than resolved.

⚠️ The orchestrator told that lane it owned the measurement window and then ran a gate on top
of it. The order-of-magnitude finding survives that easily — the signal is 99% against 1% — but
the per-world question is exactly the kind a contended host destroys, and it should be re-asked
on a genuinely idle machine before anyone concludes the spread was imaginary.

**Not measurable from here:** CI's x86_64 lavapipe. This host is arm64, and a qemu-emulated
x86_64 container would add emulation overhead indistinguishable from driver cost. Every number
above is the arm64/Mesa-22.3.6 axis only.

**THE DECISION, which is the user's:**

1. **Documented non-target.** A line in RELEASING.md/WEB.md naming software rendering as
   unsupported, stating what a person actually sees — roughly 5-12 fps at this canvas size,
   usable for reading and light editing, visibly laggy while typing or scrolling — and pointing
   at a working GPU driver. No code.
2. **Supportable with named work.** A software-adapter-detected degraded mode: smaller internal
   canvas, simplified backgrounds, fewer glyphs shaped. New mechanism, scoped as future work.

The lane leans to (1) and so does this board: closing an 80-180ms gap needs a different render
strategy, not a fix, and "more machinery for one degraded case" is the direction PHILOSOPHY.md
leans away from. But a Linux user on a VM, a remote desktop, or a machine with no working
Vulkan driver lands here, so it is a product-shape question and not an engineering one.

Leftover: Docker volumes `awl579-cargo-registry` and `awl579-target` hold the built arm64 rig
for a clean re-measurement without repaying the build.

### 582 — Kite tunnel visual correction: restore the approved bending, folded 3D surface (user report + decision, 2026-09-06)

🟡 REPAIR IN PROGRESS — `kite_582_repair` (Codex), based on current `main`.
The `item-582` candidate was rejected by independent visual review on 2026-09-11
and remains unintegrated. It improved rail curvature and retained clean haze,
legibility, static character and DPI behavior, but did not reach the approved
projected geometry: measured native section-radius variation remained about
4–5% versus the reference's 42–45%, its centreline bend was incorrectly scaled
by the fold amplitude, section roll had no equivalent owner, and the transit
pose still read as a concentric target. Repair must preserve the successful
veil/far-core/antialiasing work while replacing the log-ring approximation with
geometry that carries section centre, full multiplicative radius, roll and
fixed-theta longitudinal curvature directly. Fresh judge artifacts are in the
untracked scratch directory `/private/tmp/item582-judge`; they are evidence,
not product inputs or a gate receipt. Live motion parity and human acceptance
remain owed after a mechanically acceptable repair.

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

## Outstanding review of landed work

These are follow-ups, not additional unimplemented build tasks. Completed work and
past verification reports remain in `git log -p -- .orchestrator/queue.md`.

- **588 — Brolga bullet taste decision.** The current plain `•◦▪` remains the
  fallback: no tested Dovecote scale satisfied both contrast and the fixed-width
  box. A different glyph or wider box would be a separate mechanism decision.
  The twenty-theme gallery still needs a taste review; legibility checks alone
  do not supply it.
- **553 — folder-search highlight review.** Real-pixel match-highlight legibility
  remains unverified. Retain the known boundaries: results use summon-time disk
  contents, grouping differs from lens headers, and CRLF matches can retain a
  cosmetic trailing carriage return. Review with 639/640 rather than treating
  the original implementation as unfinished.
- **561 / 618 — ornament follow-up.** The requested ~15% reduction is merged
  (`760f4f43`, merge `b3e8d2aa`). Outstanding: live proportions and the inherited
  star/underscore ink-to-em check; those share the dash's scale dial. Do not
  requeue the already-landed size reduction.

Kite's unresolved appearance and live motion review belong to **582** above;
there is no separate 564 build item. Its review must include convergence near
page edges at common window sizes as well as several-minute motion comfort.
For current accessibility acceptance and deferred work, use `ACCESSIBILITY.md`;
the resolved 584/626 investigation does not require another confirmation sitting.

## Latest recorded verification

The latest recorded native/wasm baseline is **`4e225355`** (633 and the aggregate
perf documentation), already on main:

```text
native-gate-receipt commit=4e225355 health=pass:304s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5123 unit_shards=6 integration_targets=18
web-smoke: OK
```

This is the original baseline receipt, not verification of later code. Subsequent
queue/policy-only commits use diff/link checks under `docs/verification.md`.
Older receipts, resolved CI investigations and completed train summaries are in Git
history. Local hardware receipts do not establish hosted-GPU or live-journey results;
check remote status before a future push rather than inheriting old push warnings.

## Needs specific hardware

These remain unverified on the orchestration host; honor the current scope and
release policy in `ACCESSIBILITY.md` and `RELEASING.md`.

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst (post-v1 per
   `ACCESSIBILITY.md`).
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Release authority

Signing/notarisation setup is complete; it is not an open setup task. Every new
tag/release still requires the user's explicit instruction per `RELEASING.md`.
