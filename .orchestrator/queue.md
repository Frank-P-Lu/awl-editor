# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---

### 537 — footnote markers may wear the traditional reference ladder (user decision, 2026-09-01; sequenced AFTER 529 bundles the face)

⬜ DECIDED, READY — both product decisions landed (user, 2026-09-06): **(a)
per-document ladder scope** — the ladder follows first-reference order across
the whole document, matching today's numbering; awl has no pages, so per-page
recycling has nothing coherent to recycle on. **(b) The definition list
follows the option** — when the ladder is on, definitions wear the same mark
as their references; pairing them is the ladder's function. U+2016 ‖ coverage
in the adopted subset remains the lane's engineering verification, enrolled in
the glyph-presence law before landing.

DECIDED direction, from the user's own connection during 536's heritage
round: "the daggers were used for footnotes — we still have a chance to
use them, cuz we support footnotes." awl's footnote references already
paint their DISPLAY NUMBER as a painted ornament slot
(`footnote_number_slot` / the `FootnoteNumbers` ornament family,
docs/markdown.md — the same painted-substitute shape the bare-URL
ellipsis reuses), and display numbers already follow first-reference
order. This item adds a display OPTION (config + Settings row, default
staying numeric) that paints the TRADITIONAL REFERENCE LADDER instead:
* † ‡ § ‖ ¶, in that canonical order, doubling when exhausted (** ††
‡‡ …) per print tradition. Display-only, exactly like smart punctuation:
the file keeps `[^label]`; export unchanged (numeric) unless a later
item decides otherwise. The glyphs come from the symbol face — with
Nishiki adopted (529), † ‡ § ¶ are the celebrated cabinet's own
drawings, so the heritage is in SERVICE, not decoration: the daggers do
the same job they have done since the hand-press. Open sub-decisions
for the lane to put to the user before landing: (a) ladder scope —
per-document order (matching today's numbering) is the working
hypothesis; per-page recycling is print tradition but awl has no
pages; (b) whether the footnote DEFINITION list's markers follow the
same option; (c) ‖ DOUBLE VERTICAL LINE (U+2016) coverage in the
adopted subset must be verified and enrolled in the glyph-presence law.
Laws: ladder order pinned against the historical sequence; overflow
doubling; option off ⇒ byte-identical render to today.

---

### 577 — `Install sccache` costs 4m25s on every cold CI run because it builds from source (found by 566's step-timing, 2026-09-06)

`scripts/install-sccache.sh` builds sccache from source. It short-circuits when the pinned
version is already on PATH, so a warm run pays 0s and this was invisible until item 566
timed the first cold run in sixty: **4m25s**, the second-largest line in that job's
pre-suite budget. A prebuilt-tarball path would take ~4 minutes off every cold run, and
cold runs are now guaranteed to recur — rust-cache's key carries the rustc version, so
EVERY stable toolchain release produces one.

Not filed as a trivial swap: the script is shared with `release.yml`, so the blast radius
includes the release pipeline's permanently-unexercised `publish` job, and downloading a
prebuilt binary is a supply-chain and network-policy call rather than a build-speed one.
Decide the policy first (pin by digest? verify a checksum? keep source-build as the
fallback when the tarball 404s?), then implement.

Verify: a cold-cache CI run's `Install sccache` step drops to seconds; the release
workflow still installs the same pinned version by the same identity check.

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

### 588 — list-bullet pairs derive from each world's worn ornament set (carried out of 536's fold, 2026-08-30 decision)

Item 536 assigned all 20 worlds their Nishiki ornament trios (dash/star/underscore) and
recorded, as its own clause (c), that LIST-BULLET pairs were not covered by that pass: they
still carry the pre-Nishiki vocabulary while the trio beside them moved. The decision was a
"small follow-up taste round" deriving each world's bullet pair from the set it now wears —
Genjikō for Mulga, Moonfaces for Mopoke, Gambit for Currawong, and so on down the adopted
table in 536's own history (`git log -p -- .orchestrator/queue.md`).

Mechanism is unchanged and must stay unchanged: `theme::ornament::Ornaments` is per-world
const data. Derive the roster from `theme::worlds::THEMES` (`[Theme; 20]`, Cassowary
included) rather than a hand-list — a grep over `worlds.rs` alone has already produced a
wrong count of 19 once by missing Cassowary's own module.

Laws: every world's bullet pair is drawn from the same adopted union its trio is (enrol the
union from the roster, not a named member); no world keeps a bullet from the retired
vocabulary; the pair stays legible at prose size in both grounds. The visual outcome is a
taste call owed to the user — deliver a gallery capture across the roster, not an argument.

---

### 589 — Commands and shared transient chrome: clearer controls within each world's composition (user decision, 2026-09-07)

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

### 590 — Insert Link: a clear URL field with keyboard-first commit (user decision, 2026-09-07)

⬜ READY — queue only; coordinate shared chrome with 589.

**Decision.** Replace the empty imitation list in Insert Link with an obvious
destination field: readable `Link destination` label, `Paste or type a URL`
placeholder, immediate typing, Enter to commit and Esc to cancel. Keep a quiet
clickable commit affordance carrying its resolved binding. Preserve existing
URL prefill, selected-text wrapping, editing an existing link and undo behavior.

**Composition.** Keep the existing world/context placement policy, including
clamping on small windows; the generated below-paragraph location is illustrative,
not a new hardcoded rule. The user likes the relationship between Find and Link
chrome, with consistent borders/corners inside a world. Apply 589's world-specific
surface grammar and backing policy rather than shipping one generic rounded
dialog. Keep surrounding prose readable; no full-viewport blur merely to enter
a destination. Retain appropriate local separation where a world's composition
otherwise interleaves text with the document.

**Verify.** Read docs/markdown.md, docs/render.md and docs/harness-reach.md.
Test empty/prefilled/existing-link/selected-text paths, keyboard and pointer
commit/cancel, focus and document restoration. Native keyboard labels come
from the real keymap. Sweep composition families, anchors, narrow widths and
DPI 1/2; pixel-check label/field clarity and no clipping. Add the missing laws,
mutation-prove them, and include the standing vision smoke. Report final feel
as requiring the user's live eye, not as proven by image generation.

---

### 591 — Find/Replace: preferred bordered chrome, keyboard discoverability, existing top-right placement (user decision, 2026-09-07)

⬜ READY — queue only; coordinate with 589 and the focus-routing repair 585.

**Authoritative reference.** `references/find-replace-chrome.png` is the crop
the user explicitly preferred AFTER the keyboard-first remake. Preserve its
clear bordered fields, subtle lavender surface, thin separators, separate
match/navigation region and distinct Replace/Replace all controls where the
world uses bordered panels. The later borderless text-strip mockup is NOT the
selected chrome. Derive compatible corners from the world's shared treatment.

**Keep keyboard character.** Controls remain clickable, but visible shortcut
labels teach the existing behavior: replace/next, replace all, switch field,
close and match case. Source bindings from the active platform/keymap; no
hardcoded macOS glyphs on Linux. Distinguish labels, editable values, match
count and actions without tiny crowded hints. Find-only remains compact.
Keep the EXISTING top-right placement and safe inset, close below the title/menu
bar; the first mockup's large top gap was rejected. Do not adopt its enlarged
footprint blindly. Keep matches and surrounding prose readable.

**Verify.** Preserve search/replace semantics and focus, including 585's query
select-all law. Read docs/config.md, docs/render.md and harness-reach; exercise
both fields and actual command bindings. Capture empty/no-match/multiple-match,
find-only/replace and case states across world compositions, widths and DPI
1/2; assert bounds, shortcut/action correspondence and pixel legibility. Add
mutation-proven laws and the five-shot vision smoke. Theme identity remains
data through shared renderers, not one universal screenshot skin.

---

### 592 — Settings: compact label/value relationships and readable workspace hierarchy (user approval, 2026-09-07)

⬜ READY — queue only; coordinate shared chrome with 589.

**Approved direction.** The user strongly prefers the new Settings layout:
modest nearby title and identifiable search, category rail beside the active
category's controls, comfortable row spacing, and a bounded detail-column width
that keeps values close to labels. Extra window width becomes breathing room,
not a longer journey between a setting and its value. Remove the remote giant
SETTINGS label in favor of the integrated hierarchy. The reference's proportions
are the direction, not hardcoded pixel coordinates or replacement control semantics.

**Worlds / interaction.** Preserve the existing category/detail focus model,
selected-row control interaction, query, return path, exact editor restoration
and narrow staged presentation. Express rows, selection, corners and backing
through each world's Pane/Bars/Diagonal/Ruled vocabulary. Keep relevant key hints
near the active control, using real bindings. Start with a quiet opaque themed
workspace ground rather than ghost prose; retain frost only if it contributes
to that world's authored composition. No blanket removal of ambient effects.

**Verify.** Read DESIGN.md, docs/render.md and harness-reach. Sweep category,
control kind, focus region, composition family, narrow/wide window, zoom and
DPI 1/2. Assert label/value proximity, usable controls, no clipping, correct
focus/selection and unchanged setting behavior. Validate appearance with pixels
and the standing vision smoke; add mutation-proven laws at shared seams. Final
theme-specific composition remains a live taste review.

---

### 595 — an `overlay_hover_stability_law` failure appeared on one gate arm, once, and could not be reproduced (found by 568/569's lane, 2026-09-07)

⬜ READY — small, but it is in the class this repo has been bitten by repeatedly.

`render::tests::overlay_hover_stability_law::a_deliberate_world_crossing_can_move_a_stationary_
pixels_hit_test_row` went red on the `linux` arm ONLY during one gate run, was **absent from
that arm's own `failures:` list**, was green on `mac` and `menubar-full` in the same run, and
was green on all three arms in the next. The lane could not reproduce it targeted and
recorded it as unexplained rather than asserting it benign, which is the right call.

Why it is worth a look rather than a shrug: it is a `render::` law reaching the shared test
GPU, which is exactly the order-sensitive class CLAUDE.md names. One device is one object
population and one set of wgpu-hal counters, and a test that merely borrows a handle mutates
them; the documented signature of an unguarded reach is a law that **passes alone, passes
unfiltered, and fails only under a filter** — never failing CI and always failing a developer.
Its disappearance from the arm's own failures list is itself a finding: a red that the
receipt's own summary did not carry.

**Ruled out 2026-09-07: this is NOT the hosted-mac wedge.** CI's tolerated `mac (render::tests)`
arm was checked in case the two were one phenomenon. They are not: that arm reports `562
passed; 588 failed` with passes and failures INTERLEAVED to the last second, which is ordinary
virtualised-Metal pixel divergence across about half the render suite, not a device loss and
not a single flaking law. 595's subject failed once on the LINUX arm of a local gate, passed
on the other two arms of the same run, and passed on all three next run. Different axis,
different shape.

Build: establish whether this law (and its neighbours in that file) take
`crate::testlock::serial()` and hold it for the LIFETIME OF THE RESOURCES rather than the
call — a `TextPipeline` dropped at the closing brace still moves the counters, so a lock a
helper takes and returns discharges nothing. Then either fix the enrolment or explain the
one-off. Laws: whatever is found, prove it by making the failure deterministic before
declaring it fixed.

---

### 596 — two small truths about the personal dictionary that its own docs get wrong (found by 568/569's lane, 2026-09-07)

⬜ READY — trivial, filed so they are not lost between a merge and a board compression.

(a) `REFERENCE.md` says the dictionary file "is read at startup only". It is also re-read when
the dictionary variant switches (`set_dictionary` → `load_user_dictionary`). A generated
reference stating a wrong answer with a roster behind it is the documented hazard — the fix is
the sentence, and the check is asking the property on both sides of the condition.

(b) `remove_word_from_dictionary_file` joins with `\n`, so a CRLF-edited word list is converted
to LF by a removal. Unreachable on awl's shipped platforms and therefore not urgent, but it
contradicts the file-preservation promise the same function otherwise keeps, and the rope's
whole CRLF discipline is "load normalizes, save restores".

---

### 597 — three inline-formatting cases that predate 586/587 and have no valid output today (found by that lane, 2026-09-07)

⬜ READY — small, and filed so they are not rediscovered as regressions of the fix that found
them. All three PRE-DATE 586/587 and none was introduced by it.

(a) A document backtick immediately OUTSIDE the selection — `` x`y ``, select `y` — has no
valid output without editing text the user did not select. The honest answers are a refusal
or a widened edit, and which one is a product decision, not an implementation detail.

(b) `` **`y`** `` — a payload that is entirely a code span — cannot be recognised by any span
oracle, because awl emits no prose span when no `Event::Text` survives inside. So the toggle
cannot tell "already bold" from "not bold" here. The fix is a different oracle, not a
different threshold.

(c) `==` cannot contain a backtick at all: `push_highlight_spans` sees one text event.

Build: decide (a) deliberately — refuse or widen — and give (b) an oracle that does not depend
on a surviving text event. Laws: each case asserted through the real parser, and each proven
non-vacuous by restoring today's behaviour and watching it go red.

---

### 598 — a summoned surface now swallows ⌘Q and ⌘S, and the picker card always did (found by 585's lane, 2026-09-07)

⬜ READY — small, but it is a question about intent rather than a bug with an obvious answer.

585 gave the find/replace panel the same action-level gate the picker card has always had, so
the panel now consumes every Edit-menu verb while it is up. It also consumes **⌘Q and ⌘S**,
because that is what the card does and making the panel disagree would have been a SECOND
policy — the lane inherited the existing contract rather than inventing a third one, which was
the right call for its own round and is the wrong place to settle this.

The question this exposes: **should a summoned surface block Quit and Save at all?** A picker
that swallows ⌘Q is plausibly a pre-existing bug that nobody noticed because nobody tried it
with a picker up. Reverting is one `matches!` carve-out in `search::keys::intercept_action`,
and whatever is decided applies to BOTH surfaces or the two drift apart again.

Laws: whichever way it goes, the card and the panel must agree by construction rather than by
coincidence — one owner, swept over the surface roster, so a third summoned surface cannot
pick a third answer.

---

### 600 — 593's narrowing has two sharp edges left, both named by the lane that made them (2026-09-07)

⬜ READY — small, and both are consequences of a fix that was correct.

(a) **`--all-worktrees` is a loaded gun with a safety.** The fleet-wide sweep survives behind
an explicit flag because the maintenance use is real, and a law now stops any tracked script
or workflow from passing it. Nothing stops a human or an agent typing it during a wave.
Deleting the mode outright is a one-line follow-up; the trade is losing a convenience that
reclaimed 26 GB → 131 GB this session against keeping an edge that can corrupt four lanes.
Decide it deliberately rather than leaving it as an accident of sequencing.

(b) **The disk preflight's floors were tuned for the OLD reach.** `HEALTHY_BYTES` 32 GiB and
`MINIMUM_BYTES` 24 GiB assumed a sweep that could reclaim across the whole fleet; one
worktree's stale artifacts often will not clear that gap. So `insufficient space after
sweep-1d` will fire more often. That is the honest failure the design already prefers over a
corrupted sibling build — but the numbers were never measured under four-lane pressure, and
now they need to be.

---

### 601 — `code-health.sh` reaps its own caller's process group, and the workaround is prose every lane must remember (found by 593's lane, 2026-09-07)

⬜ READY — a real fix waiting inside a probe that already exists.

`code-health.sh` group-kills in a way that reaps the process group of whatever launched it.
The `set -m` + subshell workaround is documented in `.orchestrator/README.md` and lanes do use
it — but a documented workaround is a rule every lane has to remember, and **forgetting it
presents as a SILENT LANE rather than as an error**, which is the worst failure shape
available: nothing to read, nothing to grep, and no signal that the round even ended.

The fix belongs where the behaviour is already understood — `test-native-gate.sh`'s group-kill
probe knows how to retire descendants without reaching its own caller. Laws: a
`code-health.sh` launched from a shell leaves that shell's siblings alive, proven by planting
one and requiring it to survive; and the law must fail if the group kill is widened back.

---

### 602 — `Srgb::to_glyphon()` silently drops alpha, so a translucent text colour renders opaque (found by 570's lane while mutating, 2026-09-07)

⬜ READY — small, and it is a product fact rather than a test artifact.

While mutation-proving 570, the lane faded a mark by setting `Srgb { a: 8, .. }` and the law
stayed GREEN. The law was not at fault: **`Srgb::to_glyphon()` calls `Color::rgb`, which drops
the alpha channel entirely**, so the fade never reached the renderer at all. The mutation was
re-done as a colour blend toward the ground and fired correctly.

Why this is worth an item rather than a note: every caller that sets an alpha on a text colour
is silently getting an opaque one, and nothing says so. Either alpha is meaningful for glyph
colour — in which case this is a bug and the conversion should carry it — or it is not, in
which case the type should not accept a value it discards. **Establish which before changing
anything**, since a roster of callers may be relying on today's behaviour without knowing it.

Laws: whichever way it goes, a colour whose alpha is set must either reach the renderer with
that alpha or fail to compile. Prove non-vacuity by rendering two colours differing only in
alpha and requiring the frames to differ (or the code not to build).

---

### 603 — what should selecting inside a substituted transcript do? (named by 581's audit, 2026-09-07, and deliberately left unfixed)

⬜ READY — a product decision first, a fix second. Do not treat it as a bug report.

581 closed the accessibility leak: while History, Conflict or Credits substitutes a
transcript for the pixels, the tree now describes what the reader can see rather than the
hidden buffer. One door was named and left open rather than quietly widened.

`SemanticRequest::SetTextSelection` on the document node still maps grapheme offsets against
the REAL buffer regardless of read-only prose. It cannot simply be walled: an existing law,
`every_advertised_action_drives_a_real_transition`, requires it to keep working because all
three surfaces advertise `SetTextSelection` as a reading affordance — and a reading surface
that advertises an action it refuses is worse than one that does not advertise it.

So the question is genuinely a product one: **when an assistive technology asks to select
text inside a substituted transcript, what should happen?** Plausible answers — select within
the transcript (needs a transcript-side offset map), advertise the action but scope it to
nothing, or stop advertising it on read-only prose (which changes what a screen-reader user
is told the surface can do). Each has a different cost to the reader, and none is obviously
right.

Laws: whichever is chosen, the three surfaces must agree by construction with enrolment
derived from `shows_read_only_prose` rather than named, and the advertise/refuse pairing must
be law-pinned so a surface cannot advertise what it will not do.

---

### 604 — three band consumers 572 fixed but did not grade, and one inflation site it did not sweep (named by 572's own lane, 2026-09-07)

⬜ READY — small, and it exists because the lane said plainly where its own sweep stopped
rather than letting the enrolment guard imply a completeness it did not have.

572 made one owner of the caret-band scale, so every consumer got the fix. Its grading law
`every_caret_band_consumer_grew_by_the_size_rung_alone` grades **five** of them — selection
band, find-match wash, code pill, strike fraction, spell gap. The **nit underline** and the
**x-ray table-row band** read the same owner, are fixed by it, and are graded by nothing and
explained by nothing; the item's own text named "spell/nit underlines" and "table x-ray rows".
The link underline is honestly pinned as structurally absent from a heading row (pulldown
stamps a heading's link text `Heading`, not `LinkText`) with an assertion saying so — that one
is answered, not missing.

The 8-call-site enrolment guard forces NEW consumers into the sweep. It does not retroactively
enrol these two, which is exactly the gap a call-site count cannot see.

Also unswept: the **thematic-break `ornament_scale` row**. Its module doc argues the room is
dropped on reveal, and that argument is asserted rather than law-tested — item 571 fixed the
reveal, and nothing pins the selection band and underlines on that row.

And one enrolment that is derived but not pinned to a number: the mono-world band law asserts
only `graded > 0` rather than an exact cell count, the single enrolment in that file without
one. A sweep that silently shrinks to one world would pass it.

Laws: grade the two ungraded consumers on the same axis as the other five; sweep the
thematic-break row against every caret-adjacent treatment the way 572 swept the heading rung;
give the mono law an exact count derived from its own filtered roster.

## A lane-facing note: three lanes lost a gate cycle to the same law

`roster_claim_law::no_source_comment_types_the_world_roster_size` reddened **three separate
lanes** in one session — 570's (`pull_quote_pair.rs` typed "twenty worlds" in the module doc of
a file whose whole subject is deriving enrolment from the roster), 558's ("nineteen/twenty
worlds" in comments), and 572's (two sites, one of them a failure message reading "thirteen of
the twenty worlds"). In 572's case the previous lane's commit was **already red** against a law
that has been on `main` since 2026-08-26 and is an ancestor of that commit.

The law is right and is doing its job. The cost is discovery: it is a unit test a **filtered**
`cargo test` never reaches, so a lane meets it only at the full gate, after the work is done —
and writing "the twenty worlds" in prose is the natural way to describe a sweep. Every lane
that hit it was writing a comment ABOUT deriving enrolment from the roster.

**So a brief that asks a lane to sweep the world roster should say this outright:** describe
the roster by asking it, never by typing its size, in comments and failure messages alike. That
costs a sentence and saves a gate cycle, and gate cycles on this host are ~15 minutes each.

## Owed to the user — landed work awaiting a live eye

These items have MERGED and left the build queue. Each one still owes the user an answer or
a live look, which landing does not discharge. Full context is in
`git log -p -- .orchestrator/queue.md`.

**568 — personal spell suggestions (gated on `item-568-569`, merge pending).** Two decisions in
`src/spell/personal.rs` are the user's to confirm, quoted from the source rather than from a
commit message: `pub(super) const MAX_DISTANCE: usize = 2;` and the ranking rule that "a
personal near-miss is the user's OWN vocabulary, added deliberately, so it must never lose a
slot to a bundled guess". **The risk worth naming:** the fix offers a personal word only when
the typed word is already flagged misspelled AND within 2 edits. If what the user actually did
was type a PREFIX — `Zorb` for `Zorbling`, four edits away — this does not reach it, and the
standing "we don't need autocomplete" decision makes that deliberate. Worth asking before 568
is called closed.

**586/587 — inline formatting (merged `945ceff1`).** Two calls the lane made and flagged
rather than buried, both read out of the tree:

- **A taste call, landed, one line to revert** per this board's standing preference.
  `==highlight==` has no flanking rule of its own — measured, `==hello world ==` really does
  highlight — so trimming its edge whitespace is taste, not grammar. It is currently
  `InlineKind::Highlight => Grammar::Prose("==")` in `src/actions/format/inline.rs`; giving it
  its own grammar arm restores the old behaviour. Reverting is one line.
- **Code spans do not pad edge spaces**, though CommonMark strips a symmetric pair. awl styles
  the SOURCE bytes, so padding would show you `"  x  "` for a selected `" x "`. The cost is
  that a foreign renderer reads `` ` x ` `` as `x`. The backtick case IS padded, because there
  the alternative is no span at all.

**583/584 — new-document behaviour (merged `27aa13fa`). LIVE CONFIRMATION DID NOT HAPPEN.**
The display was locked at both ends of the lane's round — `CGSSessionScreenIsLocked` read
`<true/>` before it started and again after the gate launched — so it did not run the app and
claimed no live evidence, which is the correct call: a locked display fails SILENTLY and
writes successful-looking probe lines while presenting zero frames. Still owed to a human:
583's pause-then-type journey in a real window (the autosave clock does not exist in ordinary
capture), and 584's VoiceOver listening test. Stated plainly because the ceiling matters:
584's laws prove what awl PUBLISHED to the AccessKit adapter at the one door every update goes
through. They cannot prove the OS received it, or that VoiceOver announces it.

**585 — Find's edit verbs (merged `92b1b13a`). LIVE CONFIRMATION NOT OBTAINED.** The display
was locked (`CGSSessionScreenIsLocked = true`), so the visible ⌘A-then-typing journey the item
asks for was not run and no live evidence is claimed. Owed to a human.

**570 — where the closing 99 hangs (merged, landed as A).** *The closing 99 currently hangs in
the writing column's right gutter, mirroring the 66, so the pair brackets the column — and on
a short quote the 99 sits a long way from the words it closes. Should it instead hang
immediately after the last line's own text?* Captures sent to the user, A over B, in Bowerbird
and Paperbark, for both a multi-line and a one-line quote. A is symmetric and never collides
with text at any wrap width; B closes a one-line quote unmistakably but breaks the pair's
symmetry (66 outside the text, 99 inside it) and on the multi-line case its ink rides above
the row top and reads as belonging to the row above. Lane's recommendation and the
orchestrator's: keep A. Reverting to B is NOT one line — it needs a per-mark x on
`QuoteOrnaments`, about 20 lines — which is why B was prototyped rather than landed alongside.

**576 — the Linux follow gesture, answered in the form the user asked it.** They asked "what
should it be on linux…??? like for each keymap?" The answer, landed for their judgement:
**macOS ⌘-click under both flavors** (the flavor is structurally inert on Mac); **Linux
`native`: Ctrl-click**; **Linux `emacs`: Ctrl-click plus middle-click**. Ctrl-click is what
every editor and browser on Linux already does, and because it is a MOUSE chord it steps on
none of the `C-c`/`C-v`/`C-x` rules that make the two Linux keymaps differ at all — which is
why it can be the same under both and a Linux user need not learn two answers. Middle-click is
the extra one for emacs hands because mouse-2 is the traditional follow gesture there and awl
implements no X11 primary-selection paste to collide with. Ctrl-click is deliberately absent
on macOS, where the OS spends it as the secondary click.

Two things the user may want to overrule, each one line: **middle-click is emacs-only** (it
collides with nothing under `native` either, and was flavor-gated only so the plain platform
convention stays plain), and **the gestures are fixed rather than `[keys]`-rebindable**
(rebinding a mouse chord means inventing a second chord grammar, which is a decision worth
taking now rather than after that grammar has users). Deferred and recorded, not smuggled in: a
bare `#heading-anchor` resolves to a calm no-op.

**558 — the lone file's plate (merged `6c888d5c`). LIVE LOOK NOT OBTAINED.** The display was
locked at both ends of that lane's round, so it ran headless captures only and claimed no live
evidence. The plate is capture-verified in Mulga at RGB 126,140,103 over a 2447-pixel bbox,
matching the candidate you chose from. What a capture cannot tell you is whether the newly
plated lone file reads as calm or as busy in ordinary use — that is the whole reason 444, 469
and 515 left it bare, and it is the one thing worth a live glance now that the decision has
gone the other way.

**572 — four visible changes, none of them seen live (branch gated, merge pending).** The lane
read these out of the tree rather than out of a commit message, and ran no live probe:

- **A new authored constant nobody has looked at.** `FOOTNOTE_NUMBER_GAP = 0.10`
  (`render/spans/conceal/substitutes.rs`) is the gap after a painted footnote number as a
  fraction of the body row — 3.2px at Tawny's line height, visible in every world. The retired
  formula's implicit gap VARIED across the roster; this is uniform. Its doc claims it sits
  inside the retired spread of 0.76–4.63px and that was confirmed against the tree, but it is
  still a taste default chosen by arithmetic rather than by eye.
- **Every band on a heading row is now shorter** — 13% on `#`, 21% on `##`, 25% on `###`.
  Selection, find-match wash, code pill, strike, link/spell/nit underlines, mono caret,
  insertion bar. This is the fix, and it is the kind of change that is right in the numbers and
  still wants a glance. Revert is one line in `render/geometry/caret_band.rs`.
- **Tamed bare URLs get visibly tighter** — the "…" slot narrows by up to 14.76px on Mulga.
  Footnote slots move in both directions.
- **A table cell holding a bare URL now shows its raw source** where it used to collapse into
  a hole nothing painted into.

**551 — table selection band (merged `f740749c`, follow-up `db90497e`).** The band now paints
whole rows. If a spreadsheet-style cell-wise selection is what you actually wanted, say so —
that alternative was flagged, never built.

**553 — folder-wide search (merged `277c3717`, follow-ups `e076ddd8`/`104fb174`).** The match
highlight's real-pixel legibility is live-only and unverified. Also flagged, not hidden:
grouping does not use the lens-strip header mechanism (a deliberate scope call); a CRLF
source file's matched line keeps a cosmetic trailing `\r`; and the corpus is summon-time
only, like Assets and Go to — a file edited on disk while the picker stays open is not
re-read until the next summon.

**559 — close mark hover (merged with 550 as `347eba64`).** Keep the existing hand cursor, or
switch the whole row to arrow-plus-hover-only to match the cited convention? Hover is
pointer-only and undrivable by `--keys`/`--screenshot-app`, so the resting geometry is
capture-verified but the feel and the cursor question are yours.

**561 — ornament scale equalized upward (merged `5f90cb6d`, follow-ups `1b22a1c1`/`fd2f5894`).**
Gumtree's dash is a 4-glyph snake run, so equalizing its height also grew its width (~119px →
~252px against a 1008px column); it reads proportionate in capture, unconfirmed live.
Unmeasured: star and underscore share one `ornament_scale` dial with dash, so they grew
proportionally without being checked against their own ink-to-em ratios.

**564 — Kite's living warped-grid tunnel (merged `c3c3032e`, cleanup `002f09fe`; pushed).**
Live human sign-off is owed for the several-minute drift and contortion feel — the harness
verifies single-frame trajectories and the motion-safe still, not wall-clock feel over
minutes. Also owed: at the default 1200×800 capture geometry the roaming vanishing point can
land closer to the page edge than at the 1600×1000 geometry the pixel laws sweep, so it is
worth a live look at whether the convergence ever reads as landing inside the page itself at
common window sizes rather than staying a margin phenomenon. Item 582 (open, above) revises
this ground's geometry and inherits the same sign-off.

---

## Green train — the exact-main receipts

**Fifth train, `a7076b32`** — covers 572, HEAD verified unmoved across the run:

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

🔴 BLOCKED — release work requires the user's explicit release word and Apple signing secrets.

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
