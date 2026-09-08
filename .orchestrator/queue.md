# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

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

### 600 — `--all-worktrees`: guard it (user decision, 2026-09-08)

⬜ DECIDED, READY: **guard it.** The user chose the recommendation below in one word. Build the refusal — `--all-worktrees` exits non-zero, naming the pid or the process, while the native-gate arbiter marker names a live pid or any `cargo`/`rustc` runs — and a law that a live marker makes the flag refuse. Nothing else in this item is open.

🔵 **(b) LANDED and receipted in `19c4e2fc`. (a) is a decision the lane deliberately did not
take.** The floors are now derived from measurement — `MINIMUM_BYTES` unchanged at 24 GiB
because it is a capacity floor, `HEALTHY_BYTES` down to a derived 27, and every receipt now
reports what recovery reclaimed.

What remains is one question with a measured cost on both sides.

**What the mode costs.** A fleet-wide sweep empties `deps` and `.fingerprint` while leaving
`incremental` intact — measured, and reproduced in a control. Fourteen of sixteen worktrees on
this host currently sit in that state, holding about **90 GiB of `target/` that backs no
build**, each owing a full cold rebuild if resumed. A law stops any tracked script or workflow
passing the flag; nothing stops a person typing it mid-wave, and 593 already showed what a
sweep reaching a live sibling does.

**What it buys.** One command instead of forty-one, at a moment when the fleet is genuinely
idle — and it rarely is: two lanes were live while the measurement ran.

**The lane's recommendation, which the orchestrator endorses: keep the mode and put a check
where the operator's judgement currently is** — refuse `--all-worktrees` while the native-gate
arbiter marker names a live pid, or while any `cargo`/`rustc` runs. About ten lines, and it
turns "the operator knows nothing is building" from an assumption into an assertion. Deleting
the mode is second-best and does not touch the larger `incremental` number (item 612). The
status quo, where the safety is a habit, is worst.

---

### 603 — what should selecting inside a substituted transcript do? (named by 581's audit, 2026-09-07, and deliberately left unfixed)

⬜ DECIDED, READY (user, 2026-09-07): **select within the transcript.** A selection asked for inside a substituted transcript selects that transcript's text — the first of the three options below, the one that needs a transcript-side offset map. The action stays advertised; it is never scoped to nothing. The user's own words: it should select what you selected.

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

### 605 — the close-mark zone and both plates are placed by a char-count estimate, and a proportional face puts them left of the × (user-reported with a screenshot, 2026-09-07)

🟡 IN PROGRESS — Claude (this session), branch `item-605-617`, worktree `.claude/worktrees/item-605-617`; landed together with 617 as the board directs.

⬜ READY — a user-reported bug, so audit its neighbourhood: the active-row plate shares the
estimate and the same drift.

Reported with a screenshot: in a right-aligned stack over a proportional face, hovering a
15-character name lit a plate a full plate-width to the LEFT of the ×, and the active-row
plate ran past the × on the same side. Cause, read out of the tree: `close_hover_plate_rect`
and `close_zone` (`render/chrome/gutter_stack.rs`) derive the ink's left edge as
`right − (chars + 2) × label_char_w`, with `label_char_w = CHAR_WIDTH × LABEL` — the fixed
nominal advance in `render.rs`, never the shaped label's width. Right alignment pins the real
right edge, so in a proportional face the estimate overshoots left by the per-glyph shortfall
summed over the name; the drift grows with name length and with how narrow the face runs.
`plate_rects` uses the same estimate. Because the hover plate and the hit-test are
deliberately ONE rect, this is a hit bug, not a cosmetic one: on a long name a click on the ×
glyph itself lands in Switch, and the lit box off to the left is the place that would close.

Why the law missed it: `the_lone_row_close_mark_reveals_on_real_pixels_only_over_the_hovered_zone`
(`render/tests/gutter_stack_pixels.rs`) sweeps two name lengths in Saltpan only, and pads the
mark lane 6px to tolerate "estimate/shaping slop on a proportional face" — the face axis was
never swept, and the pad was set under the slop it was meant to expose.

**Scope change (user, 2026-09-08, see 617):** the hover PLATE is being retired — the × will flip colour instead — so this item owns the ZONE and the active-row plate only. The hit-test still needs the shaped edge; a click on the × glyph must close. Sequence 617 after this, or land both in one lane.

Fix: ONE owner reads the ink's left edge off the shaped `gutter_buffer`'s layout run (the
row's first glyph x, or right edge minus `line_w`), consumed by the zone, the hover plate, the
active plate and the hit-test alike; the char-count estimate survives only for the BUDGET
(`avail_chars`), where a count is the right question. Laws: enrol the whole world roster
(derived from `THEMES`, not a named world) × both name lengths; assert the zone's left edge
against the ×'s real first-glyph x within an antialiasing tolerance; retire the 6px pad, or
justify it against a measured maximum; prove non-vacuity by restoring the estimate under one
proportional face and watching the law go red. Name the world in the failure message.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production
tier. Read docs/render.md (rowlayout) and this file's test-lock tripwire before touching the
pixel law — it renders on the shared device.

---

### 606 — 570's closing 99 moves to B: after the last line's own text (user decision, 2026-09-07)

⬜ DECIDED, READY. The user saw the A/B captures (Paperbark and Bowerbird, one-line and
multi-line) and chose B. Per-world was asked and declined: where a closing mark sits is a
typographic rule, not a world identity — one answer, twenty worlds. Reopen only if a live
look across the roster disagrees.

What to build: a per-mark x on `QuoteOrnaments` (`render/layers/ornaments.rs`) — the "about
20 lines" 570's lane costed when it prototyped B as a capture rather than landing it. The 99
hangs one gap after the last visual row's shaped ink, on that row's own baseline. Two things
the prototype captures show and this item must fix rather than inherit: (a) on the multi-line
case the 99 rode above the row and read as belonging to the row above — anchor it to the last
row's baseline the way the 66 is anchored to the first row's. **The user said this in their own
words on seeing the captures: "some of the 99s look a tad too tall, it should be closer to the
baseline, just a little bit"** — so the vertical placement is a taste target, not just a
geometry fix, and the lane should offer two or three drops as captures rather than pick one. The user then showed a reference (a pull-quote in chat, not on disk): the 66 hangs in the left margin with its top near the first line's cap height; the 99 follows the last word after a gap of about half an em, with its ink sitting between that line's x-height and cap height — a little above the baseline, never above the line's own top. That is the target;
(b) at the widest wrap the
trailing 99 must yield inside the column rather than escape past the text edge — clamp,
never overflow. The 66 stays where it is.

Laws: 99's x = last-row ink right + gap, on every world and at narrow and wide wrap; its y
band overlaps the last row's band and no other row's; a presence floor on the glyph's ink so
a mark that failed to paint cannot pass. Deliver A-vs-B captures across the roster for the
live eye; the feel is owed to the user, not proven by capture.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; visual judge at the production tier.

---

### 607 — follow gestures: middle-click under both Linux flavors, and the gestures rebindable (user decision, 2026-09-07)

⬜ DECIDED, READY. Two calls 576 left one line from the user, both now taken the other way.

**(a) Middle-click follows under Linux `native` as well as `emacs`.** It collided with nothing
under `native` and was flavor-gated only to keep the platform convention plain; the user
wants it present. Ctrl-click stays on Linux under both flavors; ⌘-click on Mac stays;
Ctrl-click stays absent on macOS, where the OS spends it as the secondary click.

**(b) The gestures become `[keys]`-rebindable.** This is the decision 576 said was worth taking
before the grammar had users; it is taken. Rebinding a mouse chord means a second chord
grammar: extend `keyspec::parse_chord` (or a sibling owner beside it) to spell `click`,
`middle-click` and `right-click` with the same modifier prefixes keys use (`C-click`,
`M-click`, `s-click`), routed through `keymap::platform::active_follow_gestures` as the ONE
selection point, listed on every label surface the key bindings already reach, and
documented in docs/config.md. The keep-list stays untouched: a mouse chord remains outside
it by construction, and the law that says so stays.

Laws: every default gesture in the roster round-trips through the parser; a `[keys] follow =
"…"` line replaces the defaults per platform and the label surfaces report it; a chord the
grammar cannot spell keeps the default and prints a note naming the line, the same shape a
bad key chord already gets. Keep the deferred `#heading-anchor` no-op deferred.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production tier.

---

### 612 — `target/debug/incremental` has no owner and no reclaimer, and it is the biggest disk lever here (measured by 600's lane, 2026-09-07)

🟡 IN PROGRESS — Claude (this session), branch `item-612-613`, worktree `.claude/worktrees/item-612-613`; one lane with 613.

⬜ READY — the largest single disk fact on this host, and nothing in the fleet addresses it.

`cargo sweep` cannot touch `target/debug/incremental` at any threshold — measured directly,
124 KiB before and after a sweep that emptied `deps` and `.fingerprint`. That directory is
**61–69% of every `target/` on this machine**: 16.5 of 25.4 GiB in the root checkout, 5.2 of
7.5 and 6.1 of 10.0 in lanes.

After a sweep it is **pure dead weight** — the `deps` it belonged to are gone, so it backs
nothing. The fleet currently holds about 90 GiB in that state across 14 worktrees.

Build: decide who owns it and on what rule. The obvious candidates are an age-based prune the
preflight can actually perform, or `CARGO_INCREMENTAL=0` for lane builds (which trades rebuild
speed for space and should be measured, not assumed). Whatever is chosen, the preflight's
`SWEEP_YIELD_BYTES` is derived from what recovery can actually reclaim and must be re-derived
if this door starts reclaiming too.

---

### 613 — `test-native-gate.sh`'s free-oracle hardcodes 40 GiB, an undeclared coupling to the healthy floor (found by 600's lane, 2026-09-07)

🟡 IN PROGRESS — Claude (this session), branch `item-612-613`, same lane as 612.

⬜ READY — small, and it is the "a check runs in one configuration" hazard in miniature.

The probe's fake free-space oracle returns a hardcoded 40 GiB. That is above the healthy floor
today (27 GiB, and 32 before), so the probes exercise the no-recovery path. **If anyone ever
raises `HEALTHY_BYTES` past 40 GiB, every native-gate probe silently starts taking the preflight
lock and running the sweep path** — changing what those laws test without a single one of them
going red.

Build: derive the oracle's value from the floor it is meant to sit above, or assert the
relationship so the coupling fails loudly instead of silently. Law: the oracle's value must
exceed the healthy floor by a stated margin, and the law fails if the floor is raised past it.

---

### 608 — a selected bullet row draws its depth ornament AND its revealed raw `-` (user-reported with a screenshot, 2026-09-07)

🟡 IN PROGRESS — Claude (this session), branch `item-608`, worktree `.claude/worktrees/item-608`.

⬜ READY — small, reproduced headlessly, and the neighbourhood is already audited: the bullet
ornament is the ONE painted-ornament family that never learned the selection reveal.

Reported as "when selected, the 2nd-level indent changes shape": in a nested list, selecting
across a child row shows two markers on it — the depth glyph where it always sits and, just
below and left of it, the raw `-` the selection reveal restored. Reproduced in Bombora and
Bowerbird with `--keys "C-n C-n S-Down S-Down"` over a four-line list; the marker lane's ink
on the selected child row widens from the ornament's 7px to the full 25px of a depth-0 lane,
and the zoom shows the glyph stacked over the dash. The depth-0 rows double too, but there the
ornament sits on top of the dash and hides it, which is why the child row is the one a reader
notices.

Cause, read out of the tree: the line-attrs owner (`render/spans/layout.rs`) conceals the raw
marker only when `conceal_off_cursor && !line_selected`, exactly as its comment promises — "on
the caret's own line, or any selected line, the raw markup reveals and NO ORNAMENT IS DRAWN".
The painter does not keep that promise: `bullet_marks` (`render/rects.rs`) skips `li ==
self.cursor_line` and nothing else, while its siblings `rule_marks`, `footnote_marks` and
`bare_url_marks` all filter through `selection_touch_bytes`/`selection_touches` as well. The
selection reveal was widened to the legacy bullet CONCEAL and never to the bullet ORNAMENT.

Fix: route `bullet_marks` through the same `selection_touch_bytes`/`selection_touches` owner
the other three read — one filter, not a fourth reading of the overlap test — so the ornament
set and the conceal set are the same set by construction. Laws: extend the existing bullet
depth/reveal law so that a selection touching a bullet row (caret elsewhere) yields no glyph
for that row in `bullet_glyphs()` while `bullet_marker_concealed` reads false for it; sweep
depth 0 and depth 1 and a selection that touches the row without the caret's line moving
(the `refresh_rule_conceal` skip-gate tripwire in docs/markdown.md); prove non-vacuity by
restoring the caret-only skip and watching it go red. A pixel companion: the marker lane's
ink on a selected child row is the dash's alone, no wider than the unselected caret-row lane.

Routing: worker Sonnet medium (Claude) or `gpt-5.6-sol` medium; outcome audit at the
production tier.

---

### 609 — the theme picker keeps ONE chrome while the document behind it previews each world (user decision from reader feedback, 2026-09-07)

⬜ DECIDED, READY — coordinate with 589 (shared transient chrome): this item is the one
surface 589's "each world's authored composition" rule does NOT apply to, by decision.

Reader feedback, relayed by the user: "the theme switcher should not jump all over the
place — it made my boyfriend dizzy". Reproduced with `--keys "Cmd-T C-n…"` from Tawny: every
arrow re-composes the LIST ITSELF into the previewed world's chrome, because
`sync_theme_colors` switches `theme::active()` per arrow and the picker reads its
composition from there like every other overlay. Across a few arrows the list is a plain
pane at the column's left, then a descending spine on the left with a THEMES placard
(Mangrove), then chips on the left with a paged "↑ 1 more / ↓ 7 more" window (Galah), then an
ascending spine on the RIGHT (Magpie), then a ruled list top-right (Kite) — moving corners,
changing face, row pitch, list style and how many rows are visible, all while the reader is
trying to hold the selection with their eyes.

**Decision.** The theme picker gets a FIXED chrome for the life of the summon: one simple
list in one place, the Find/Replace-box grammar the user already prefers
(`references/find-replace-chrome.png` beside this board), while everything BEHIND it — page,
prose, margins, ground — previews the world live as today. The list's own surface colours
may follow the previewed world (that is the preview) but its composition, anchor, face,
row pitch, page window and selection treatment do not. Frost stays `Footprint`.

Mechanism, not a per-world code path: the picker's chrome reads a PINNED `RenderCaps` /
composition captured at summon (or a dedicated `ListStyle::Pane`-shaped constant) rather
than `theme::active().render_caps` per frame — one seam, named, with every other overlay
still reading the live caps. `effective_list_style()` is where the picker currently asks; do
not special-case inside the compositions.

Laws: across a full arrow sweep of the roster the picker's card rect, anchor, list style,
row pitch and visible-row window are identical frame to frame (sidecar + pixel bbox of the
card), while the page ground behind it changes on every arrow (presence: the frames DO
differ outside the card); prove non-vacuity by restoring the live-caps read and watching the
rect law go red on the first non-Pane world. Standing five-shot vision smoke.

Routing: worker `gpt-5.6-sol` high or Sonnet high; visual judge at the production tier.
Feel is owed to the user's live eye — and to the reader who got dizzy.

---

### 610 — an untagged Chinese note renders as a patchwork of Japanese and Chinese faces; the Han tiebreak grows an evidence tier and Settings gets Auto (reader feedback + user decision, 2026-09-07)

⬜ DECIDED, READY.

Reader feedback: "Chinese is kinda weird… Simplified is correct in Bowerbird but not right in
other themes". Measured: the note is untagged, so every Han run resolves through
`script::doc_lang_for`, which hands it `cjk_priority.first()` — `ja` by default and in the
user's own config — so the run shapes in the world's JAPANESE face. Every bundled Japanese
subset (Zen Maru Gothic, Noto Sans/Serif JP, Klee One, Shippori Mincho) carries the same
6 356-character JIS set, which LACKS the simplified-only characters: of the sentence
这是简体中文的一段测试文字骨头直角与其内外开关门说话车站, the JP faces hold
是体中文的一段文字骨直角与其内外站 and none of 这简测试头开关门说话车. So the shared
characters draw in the Japanese face with Japanese forms (骨 直 与 内 differ visibly) and the
rest fall through glyph-by-glyph to whichever face has them (Noto Sans SC bundled, PingFang
on macOS) — two faces interleaved inside one sentence. Bowerbird only LOOKS right because
Zen Maru Gothic and Noto Sans SC happen to be close in weight and roundness.

The gap is structural: `dominant_cjk` (the "intelligent" tier) feeds only the palette's Tag
document language command, and for a pure-Han note it answers `Han`, never zh-vs-ja — so
RENDERING has two tiers (tag, then the settings ladder) where the user believed it had three.

**Decision (user's model).** Resolution is tag → evidence in the text → setting, and the
setting's default is **Auto**. The evidence tier is DOCUMENT-scoped (one language per note,
never a flip mid-sentence), in-memory and per-open, edits nothing — `dominant_cjk`'s own
contract, widened to answer the question it currently ducks:

1. any kana anywhere → `Ja`;
2. any SIMPLIFIED-ONLY character (这 们 说 没 …: in GB 2312, in neither JIS X 0208 nor Big5)
   → `ZhHans`;
3. any TRADITIONAL-ONLY character (這 們 說 …) → `ZhHant`;
4. hangul → `Ko`;
5. nothing decisive (shared characters only) → the setting: **Auto = the current default
   ladder** (`ja, zh-Hans, zh-Hant, ko`); an explicit ladder replaces step 5 ONLY. It never
   overrides steps 1–4 — a "Japanese" setting forcing a note full of 这 into a face that has no
   这 is the patchwork again.

A kanji-only Japanese title stays Japanese (no simplified-only character in it); a Chinese
note written wholly in shared characters is the one miss, and no real paragraph of simplified
Chinese manages it. **The known miss is a MIXED note** — Japanese and Chinese paragraphs in one
file — which resolves Japanese as a whole (kana wins the document) and leaves its Chinese
paragraph in today's patchwork; a tag cannot rescue it either, being one answer per file.
Per-paragraph scoping is the fix for that shape and is deliberately NOT in this item (rare,
and adjacent lines flipping face is its own taste round). The character tables are GENERATED from Unicode's Unihan data and checked
in (`script::han_class` or a sibling) — never derived from which font happens to be bundled,
so the rule cannot move when a subset does. A tagged document is unchanged: the tag still
wins, and the Tag command now writes what the evidence tier already concluded.

**Settings + config.** The "CJK priority" row gains an **Auto** value and defaults to it;
config accepts `cjk_priority = "auto"` alongside the explicit list (an absent key is Auto;
today's default list written out explicitly is honoured as an explicit ladder and reads the
same as Auto). `frontmatter::cjk_priority()` stays the ONE owner the Settings row, the CJK
picker and the render ladder all read.

Laws: the untagged test sentence resolves `ZhHans` in every world (enrol the roster from
`THEMES`); the same sentence with one kana appended resolves `Ja`; its traditional twin
resolves `ZhHant`; a shared-characters-only note follows the setting, and an explicit
`ja`-first ladder does NOT override a simplified-only hit; a `lang:` tag beats all of it;
prove non-vacuity by restoring `cjk_priority.first()` and watching the first law go red.
Pixel companion: the sentence in one world shapes in ONE family end to end (sidecar
`font.scripts` plus a per-run family read). The generated tables get a law against the
Unihan source they were cut from, and a spot-check of a sample of entries against the
code (the generated-document tripwire in CLAUDE.md). Update docs/fonts.md's ladder and
docs/config.md's `cjk_priority`.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production
tier.

---

### 611 — "Open in Awl" from the Finder: declare document types and accept the open-documents event (user request, 2026-09-07)

⬜ READY — engineering, two halves, both required; the second is the one that is easy to
skip and then nothing opens.

The user asked: "finder: open in awl, like right-click on a file and add this option? how do
we do this?" Today `scripts/package-macos.sh` writes an Info.plist with NO
`CFBundleDocumentTypes`, so the Finder's Open With menu never lists Awl and it cannot be made
the default for `.md`; and the app has no handler for the open-documents Apple Event
(`application:openURLs:` / `openFiles:` — `grep -rn openFiles src` is empty), so even
`open -a Awl note.md` launches the app without the file. The only live door is the daemon's
`open <path>` socket line, which the CLI uses.

(1) **Declare the types** in the plist: `CFBundleDocumentTypes` for `net.daringfireball.markdown`,
`public.plain-text`, `public.text` (and the `.txt`/`.md`/`.markdown` extensions as
`CFBundleTypeExtensions` for pre-UTType consumers), role Editor, `LSHandlerRank Alternate`
so Awl is OFFERED without stealing the default. Once declared, right-click ▸ Open With ▸ Awl
appears for every text file, and "Change All…" makes it the default. Keep the MAS arm's
entitlements in mind: the sandboxed build needs `com.apple.security.files.user-selected.read-write`
already present for a picker-chosen file; verify Finder-opened files are covered by the same
entitlement (they are, as user-selected).

(2) **Accept the event.** winit 0.30 owns the `NSApplicationDelegate` and forwards no
open-documents event, so install a handler on the delegate winit creates (objc2 subclass or
method addition on the existing delegate class, the way `mac_chrome` already reaches AppKit)
that routes each URL into the SAME `DaemonEvent::OpenPath` the socket door already posts via
`EventLoopProxy` — one open path, never a second. Handle BOTH cases: app already running
(event arrives on the live loop) and cold launch (the event arrives before the window exists;
queue it and drain after the first frame, the same shape session restore uses). Honour the
existing single-instance daemon: a second Finder open must not spawn a second process.

A Finder context-menu item that reads literally "Open in Awl" is a Finder Sync extension or a
user-installed Quick Action, neither of which awl ships; Open With is the platform's own
answer and is what the item delivers. Record that in docs/platform.md.

Laws: the plist declares each type by name (a test parses the generated plist); the
open-path route is one owner (grep-law: no second path from AppKit into `App`); the
cold-launch queue drains exactly once. Live: Open With from the Finder on a running and on a
quit Awl, and `open -a Awl file.md` — live-only by nature, flagged for the user.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production
tier.

### 616 — table selection: neighbouring cells flicker while the band settles (user report, 2026-09-08)

⬜ READY. The user confirmed 551's whole-row band is what they want, and reported one defect on it in their own words: "the neighbouring cells kind of flicker, I don't like that." Cells beside the selected run change appearance transiently while the selection moves. Read the cause out of the tree before fixing — 551 landed in `f740749c` (`render/rects.rs`, `render/geometry.rs`, law in `render/tests/table_selection_band_law.rs`) with a follow-up in `db90497e`; find what animates or re-paints on the untouched cells (a band ease, the table x-ray's grid float, or a cache invalidation that re-shapes the row). **The user's fallback, stated plainly: if the flicker cannot be cut out on its own, remove the animation on the table band altogether.** A calm still band beats a lively one that flickers.

Laws: over a selection that grows one cell at a time, the pixels of every cell OUTSIDE the band are byte-identical frame to frame (sweep the roster, not one world); the band itself still paints whole rows. Prove non-vacuity by re-introducing the transient and watching the law go red. The feel is live-only; deliver a motion capture (`--screenshot-motion`) and flag the live look as owed.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production tier.

---

### 617 — the × hover box goes; the mark flips colour instead (user decision, 2026-09-08)

🟡 IN PROGRESS — Claude (this session), branch `item-605-617`, same lane as 605.

⬜ DECIDED, READY — sequenced after 605, or landed in the same lane.

The user kept the hand cursor (559's open question, now closed) and rejected the hover plate: "the box that shows up when you hover the × is kind of offensive; get rid of the box and just flip the colour of the ×." So on hover the × swaps to a second palette colour and no rect is drawn. The hit zone stays exactly where 605 puts it (shaped ink edge); only the paint changes. Pick the flipped colour from the world's existing roster through one owner rather than a new constant per world — the caret accent is the obvious candidate, and DESIGN.md's one-accent rule is the reason to prefer it. The user asked "we have enough colours in the palette to do this, right?" — answer that in the lane's report with the pair actually used, per world, and a contrast figure against the ground.

Laws: hovered × ink differs from resting × ink on every world (presence floor on both); no plate pixels paint on hover (the plate law from 558/559 inverts, not deletes); the active-row plate is untouched. Retire the plate-geometry laws that only 605's hover plate needed.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; vision smoke over five worlds asking "which × is hovered?".

---

### 618 — Gumtree's dash ornament about 15% smaller (user taste call on 561, 2026-09-08)

⬜ DECIDED, READY. The user saw 561 live in Gumtree: the snake reads proportionate but "a tad too tall — make it 15% smaller, maybe." Gumtree's `ornament_scale` is `4.648` in `theme/worlds.rs`; the target is about `3.95`. **Tripwire from 561, still true:** star and underscore share that one dial with dash, so a plain scale change shrinks all three. Decide in the lane whether the three should move together (simplest; check star and underscore in Gumtree after) or dash gets its own factor — prefer the shared move unless a capture shows the other two going too small, and say which in the report. Update the equalisation law in `theme::tests::ornament` so it does not re-equalise the value back up. Deliver before/after captures of `---`, `***` and `___` in Gumtree at the default geometry.

Routing: worker Sonnet medium; one capture round, no audit beyond the law.

---

### 619 — the no-document screen says which folder is open (user report, 2026-09-08)

⬜ READY. The user opened a folder of markdown files and saw the same blank screen as before, and read it as "nothing happened" — the folder HAD opened (the picker lists it), but the screen gave no sign. That is a real gap in the first-run/no-document state, not a bug in opening. Read the current state out of the tree first: `app/lifecycle.rs` builds the title through `window_title_no_document`, and `firstrun::is_first_run` decides the welcome; find what the empty document surface paints when `root` is set but `file` is not (the empty-state notice law in `render/plan/tests.rs` is the seam).

What to build: when a root is set and no file is open, the empty surface names the folder (its last path segment, home-relative) and the one action that follows — the Go-to-file chord rendered through `keytoken`, so the glyph is right per convention. Same owner for the title bar's no-document text. Keep it to one dim line in the composition, not a panel. Nothing changes when neither root nor file is set.

Laws: sidecar reports the folder name in the empty state when a root is set (drive it with `--root`); the line is absent with no root; the chord glyph matches the active convention; pixel presence floor on the line in both grounds. Capture against a seeded `--root`, never the ambient one (public repo; see Conventions).

Routing: worker Sonnet high; vision smoke: "what folder is open?" over three worlds.

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

These items have MERGED and left the build queue. Each one still owes the user an answer or
a live look, which landing does not discharge. Full context is in
`git log -p -- .orchestrator/queue.md`.

**584 — new-document VoiceOver (merged `27aa13fa`).** 583's pause-then-type journey was confirmed live by the user on 2026-09-08 ("seems to work"); only 584's VoiceOver listening test remains, and the user said they will do it later.
The display was locked at both ends of the lane's round — `CGSSessionScreenIsLocked` read
`<true/>` before it started and again after the gate launched — so it did not run the app and
claimed no live evidence, which is the correct call: a locked display fails SILENTLY and
writes successful-looking probe lines while presenting zero frames. Still owed to a human:
583's pause-then-type journey in a real window (the autosave clock does not exist in ordinary
capture), and 584's VoiceOver listening test. Stated plainly because the ceiling matters:
584's laws prove what awl PUBLISHED to the AccessKit adapter at the one door every update goes
through. They cannot prove the OS received it, or that VoiceOver announces it.

**558 — the lone file's plate (merged `6c888d5c`). LIVE LOOK NOT OBTAINED.** The display was
locked at both ends of that lane's round, so it ran headless captures only and claimed no live
evidence. The plate is capture-verified in Mulga at RGB 126,140,103 over a 2447-pixel bbox,
matching the candidate you chose from. What a capture cannot tell you is whether the newly
plated lone file reads as calm or as busy in ordinary use — that is the whole reason 444, 469
and 515 left it bare, and it is the one thing worth a live glance now that the decision has
gone the other way.

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
