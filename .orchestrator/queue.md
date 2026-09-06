# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---

### 558 — single open file draws no active plate in the working-set gutter (user report, 2026-09-01 — "when you first open a file, it doesn't seem to be selected?"; behavior confirmed in every capture's own gutter)

🟡 CLAIMED 2026-09-07 — lane `item-558`, worktree `.claude/worktrees/item-558`. **DECIDED — plate it** (user, 2026-09-06, from the lane's side-by-side
captures: "sure plate it. this means more consistent right?"). The rule becomes
"the active file is ALWAYS plated" — no lone-file exception. Build: route the
single-file identity line through the same plate mechanism as multi-file active
rows (one owner, not a lookalike rect); law sweeps both cells (lone file, fresh
open among several) per the item's closing clause; the three prior items'
calm-when-alone reasoning is superseded by this decision. Investigation history:
merged `53c7b023`. Git history (items
444/469/515) confirms the unplated single-file identity line is
DELIBERATE — item 444's own commit: "THE ONE-FILE CONTRACT IS
STRUCTURAL, NOT A RESEMBLANCE... the plate pipeline is handed no rects,
not a stack of one that happens to agree"; item 469 chose `muted` ink
specifically because "a plate-less lone heading has nothing left to
differentiate against"; item 515 weighed and kept it plate-less. Still
production-tested today (`a_single_file_block_plates_nothing`). Per the
item's own framing: deliberate AND it has failed its reader once — a
taste call, not a bug, so no default behavior changed. Both candidates
captured headlessly for comparison: unplated (current, RGB ≈29-30,40-
41,21-22 at the identity row) vs a temporary plated patch (RGB ≈126,
140,103, matching the multi-file active-row treatment) — captures are
local-only (`/tmp/gutter-plate-compare/`, not committed); the lane
describes the pixel diff precisely in its report for the orchestrator
to relay. **Q: plate the single-file identity line to match multi-file
active rows, or keep it bare (the documented "calm when nothing to
distinguish" reasoning)?**

Separately verified (not a taste call): a freshly opened file among
several already-open files IS plated immediately — no bug, traced to
`WorkingSet::open` setting `active` unconditionally and `stack_rows`
re-deriving fresh on every call, no cache/debounce. Landed as a
mutation-proven law (`workingset::tests::a_freshly_opened_file_among_
several_is_active_immediately`). Exact-main receipt: health pass:266s,
both conventions, menubar=full:on, 4729 unit tests, 17 integration
targets.

With exactly one file open, the identity line shows the bare name — no
active-row plate — and the user reads that as "not selected." Two-file
state plates the active row (their own earlier screenshot). Confirmed as
current behavior incidentally in every capture this session (the
bottom-left gutter of each PNG shows the lone open file unplated).
Mechanism neighborhood: the single-file identity line is a separate door
from the stack (`render/chrome/gutter.rs` — "the identity line is EITHER
the lone filename or the working set's rows"; it already shares the
close-mark lane via one mechanism), and `gutter_stack::plate_rects` only
plates `file.active` STACK rows. Whether the unplated single file is a
deliberate calm-when-nothing-to-distinguish choice or an omission is for
the lane to establish from the tree and `git log` — and if deliberate, it
has failed its reader once, which is data: bring the finding plus a capture
of each candidate (plated vs not) back to the user for the taste call
rather than landing either silently. Also establish whether a freshly
OPENED file among several is plated immediately (the user said "first
open"; their screenshot is the single-file case — the multi-file
fresh-open cell is unverified). Law once decided: sweep both cells.

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

### 570 — blockquote pull-quote: the 66 gets its 99 (user report, 2026-09-06 — "the 66s must be followed with 99s… it kinda bothers me"; the blockquote ornament, not smart quotes)

🟢 MERGED as `d20cc09a`, EXACT-MAIN RECEIPT OWED; **placement is a taste call now put to the
user with captures** (see the owed section). Marks raised at merge off the merged tree —
geometry.rs 1340→1353, layers.rs 1383→1387, rects.rs 1788→1835, all far below their frozen
baselines. The lane could not raise them itself and correctly did not try: native-gate.sh's
own failure text says a lane does not edit `code-health.toml` in that mode.

**One correction this item owes its own text:** it claims the quote-orientation law "pins all
four curly codepoints heavy-bottom in every display face". It does not — `EXPECTED_HEAVY_BOTTOM`
pins `U+2018`/`U+201C` heavy-bottom and `U+2019`/`U+201D` heavy-**top**, which is what a closing
mark should be. The orchestrator relayed the wrong version into the lane's brief; the lane
checked the law rather than the brief and the shipped comment is right.

RESUMED HISTORY — lane `item-570`, worktree `.claude/worktrees/item-570`. The
first lane was cut off by a usage limit at the exact moment it began mutation proofs, leaving
its whole round as **staged, uncommitted changes with no commits on the branch**: 9 files,
~732 insertions including a new `render/tests/pull_quote_pair.rs`. So the implementation
exists and **nothing about it is proven** — no mutation, no gate, no receipt, and no report.

Its design, read out of the file it left rather than out of a claim about it: the closing mark
hangs in the writing column's RIGHT text-pad gutter (`geometry::pull_quote_right`) on the
block's LAST visual row, same display face, same layer scale, same `theme::faint` value as the
opening mark. Its law builds a DIFFERENTIAL pair per world — the document against itself with
the blockquote lines blanked, line count and row tops preserved — so the page ground and
whatever per-world pattern bleeds under the column cancel, which a same-image threshold cannot
do. It pairs presence with contrast for the reason the item demands.

The resuming lane is told to commit first so a second interruption cannot lose it, then to
verify that work rather than inherit it — the placement in that file's header is a claim, not
a finding.

The hanging pull-quote mark draws only the OPENING `“` — `QUOTE_MARK_GLYPH`
(src/render/layers.rs:64), shaped once in the world's display serif and hung
at `pull_quote_left` on each blockquote block's first line
(`TextPipeline::quote_marks`, ornaments.rs `QuoteOrnaments`). There is no
closing counterpart, so every quote in every world reads permanently
unclosed. Build the `”` (U+201D): same scale, same `theme::faint()` value,
same display-serif shaping, hung at each block's LAST line. Placement is
the one open design question the worker resolves and flags for live taste
confirmation: the typographically honest spot is trailing — mirrored into
the right margin at the block's end (or hanging after the last line's
text) — pick by capture comparison across a few worlds, not by argument.

Grounding: the quote-orientation law (render/tests/quote_orientation.rs)
already pins all four curly codepoints heavy-bottom in every display face,
so no new font risk; the never-tofu law covers the glyph. Multi-block
documents get one pair per block (quote_marks already walks blocks).

NOT in scope, recorded so nobody conflates: straight→curly smart-quote
substitution (typing or render) stays declined per the standing user
decision — this item closes the ornament's pair, it does not educate quotes.

Laws: every world's blockquote capture shows BOTH marks by pixel presence
(a presence floor, not just a contrast ratio — the mark must exist to
pass); open and close share value/scale by arithmetic; a one-line
blockquote still shows a legible pair (the degenerate case where first
line == last line); no mark on non-blockquote lines.

---

### 572 — class audit: decorative geometry vs the caret — every row or advance inflated for an ornament, probed against what caret/selection/highlight inherit (user decision, 2026-09-06 — "we should fix this class of bugs yeah?? like ornament + cursor")

Item 571 (rule row's ornament room swallowing the block caret) is one
member of a class, and 545 (smart-punct conceal's giant reserved slot) is
another — per standing policy, the neighborhood gets audited because bugs
cluster. The class: any mechanism that grows a ROW's height or a run's
ADVANCES for a decorative replacement, cross-examined against every
caret-adjacent treatment drawn from that geometry (block caret, beam
caret, selection band, spell/nit underlines, link underline, find-match
wash).

Enumerate the inflation sites from the code rather than from this list —
known members to seed the sweep, not bound it: the thematic-break
`ornament_scale` row (571 fixes the reveal; the audit checks selection
bands and underlines on it too), the heading ladder's DECOUPLED row growth
(row grows "beyond what its font size alone needs" — does the block caret
span the decoupled extra, or the em box? a big caret on big text is
coherent, a caret taller than the heading's own glyphs is not), inline
image rows (absolute line-height, same decoupling shape), the smart-punct
reserved slot (545 — what does a selection or block caret over the slot
look like before/after that fix), zero-width concealed spans (caret x and
selection band at a collapsed span's boundary), and table x-ray rows.

Probe form per policy: state (caret-on / selection-touch / off) × surface
(each caret/highlight treatment) × world (sampled across the roster,
including one dark world and one with a large equalized `ornament_scale`,
e.g. the 4.6× member), asserting per cell with sidecar geometry AND pixel
arithmetic — a highlight's drawn box against the row's glyph ink box, not
just against row metrics (the sidecar is a state oracle, not an appearance
oracle). Production audit tier. An audit that finds something ends by
writing the missing law; the deliverable is the cell table plus laws, with
each defect it finds either fixed in-item when small or queued as its own
scoped item when not.

---

### 576 — follow a link: right-click shows "go to", modifier-click opens (user decision, 2026-09-06 — "right click on a link should show 'go to'; and i guess command click should open it too? (what should it be on linux…??? like for each keymap?)")

🟢 BUILT AND COMMITTED (`a29d8a1b`), GATE OUTSTANDING — lane `item-576`, worktree
`.claude/worktrees/item-576`, based on `135f9a5c`.

**THIS ITEM'S HEADLINE PREMISE WAS FALSE, and the lane checked before building.** "Nothing
follows them yet" is wrong three ways: `Action::FollowLink` already existed with a `C-c C-o`
chord, a palette row and a catalog entry; `App::follow_link` already spawned
`open`/`xdg-open`/`window.open`; and right-click already summoned a card whose first row was
"Follow link", with ⌘-click already following on macOS. That is the fourth
orchestrator-authored premise this board has watched dissolve on first measurement, and it
closes as **premise largely false, three real gaps found and fixed** rather than as "built the
affordance".

The three real gaps, all narrower and sharper than the item: **(1) Linux had no follow gesture
at all** — `SUPER` is ⌘ on Mac and the compositor's key on Linux, so a Linux hand had no
pointer follow whatsoever, which is exactly the user's own question. **(2) Bare URLs wore the
underline and followed nothing** — `link_at` runs pulldown without the autolink extension, so a
plain `https://…` in prose is no `Tag::Link`, and every door answered `None` under a hairline
the render had already drawn. **(3) A relative link was handed to the OS opener** — `open
./notes/plan.md` gives a `.md` file to whatever the desktop owns that extension, instead of
opening it in awl.

**The item's own load-bearing premise HELD and is now a law rather than an argument.**
`Config::effective_linux_keep` composes strings that every consumer parses through
`keyspec::parse_chord` into a `(Key, ModifiersState)` pair, and a mouse button has no spelling
in that grammar. `the_linux_keep_list_holds_only_key_chords_so_no_mouse_chord_can_collide`
drives the composed keep-list under both flavors plus a user entry, requires every member to
parse as a key chord, and requires `Ctrl-Click`/`Mouse-2`/`Middle-Click`/`Cmd-Click` to parse
as none.

Mouse gestures deliberately do NOT go into `active_seed_tables`: its entries are `(&str,
Action)` chord specs that `parse_binding` consumes and `seeded_chords_for` prints, so a
`"Mouse-2"` string would fail the first and print a fake key chord in the second. Instead the
shared gate `linux_emacs_layer(convention, flavor)` was extracted and BOTH selection points
call it, with a law asserting over the whole grid that the key layers and the pointer gesture
enrol identically. `keymap::follows_link` is the one predicate the press path and the hover
cursor both ask, so the pointing hand and the click cannot disagree.

⚠️ **Merge note:** this lane touched `src/render/rects.rs` (one match arm, net −2 lines),
which item 570 also grew. Re-derive that mark off the merged tree rather than from either
branch's number.

The followable-span grammar already marks every followable span with one
underline (named links and tamed bare URLs, one owner) — but nothing
follows them. Build the follow affordance:

- **Modifier-click opens.** macOS: ⌘-click (decided). Linux: recommend
  Ctrl-click under BOTH flavors — the platform convention in every editor
  and browser, and a mouse chord, so it collides with none of the C-c/C-x
  text-chord rules (the keep-list machinery governs key chords, not mouse
  chords) — plus middle-click (mouse-2) as the emacs flavor's own follow
  gesture, seeded Linux-only like the Meta layer, inert on Mac and under
  native. Route the gesture through the keymap/platform seed-table seam so
  label surfaces and dispatch read one source; decide there whether the
  mouse chord is `[keys]`-rebindable or fixed (recommend fixed v1).
- **Right-click on a followable span** shows the go-to affordance. awl has
  no context menus by design — keep it summoned and minimal (a one-row
  card in the overlay family, or the pointer affordance family from the
  hover work), not a native NSMenu grafted onto the wgpu view (see the
  muda tripwire). Label: "Go to <destination>" with the tamed authority,
  never the raw URL flood.
- **What opens where:** web URLs hand off to the system opener
  (`open`/`xdg-open` — an outward action at the user's explicit gesture,
  not a runtime fetch; the zero-network invariant is about awl phoning
  home, and this is the EDITOR-daemon shape of OS integration). A RELATIVE
  path to a local file follows IN awl — the Live-Preview model's own move
  (a vault of notes linking each other). Heading anchors and footnote
  jumps: deferred, recorded.
- Verify: the follow gesture produces a typed effect carrying the resolved
  destination, asserted in the sidecar through both chord replay and the
  real-App driver; the actual launch is the live-only tail (harness-reach:
  flag it, do not stub the effect layer around it). Laws: modifier-click
  on a plain word produces nothing; the same click on each followable kind
  resolves the destination the underline grammar says it has; Linux seeds
  appear under the right flavors and the Mac binding under none of them.

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

### 580 — insertion-door census: every path that can mutate the focused buffer is enrolled at one seam (follow-up to 575's lane report, 2026-09-06)

⬜ READY (blocked on 575 merging — builds on its wall and `text_door.rs`)

Evidence: 575's lane found TWO insertion doors nobody had listed (assistive
`ReplaceSelectedText` and `SetValue`) beside the briefed one (`Ime::Commit`). The class
grows every time a new input capability lands (menu Edit actions, Linux middle-click paste,
future dictation, daemon/EDITOR writes, drag-drop text if ever), and today a new door
ships OPEN by default: nothing forces it through 575's wall, so the next one repeats this
bug on whatever modal surface exists then.

Build: enumerate every code path that inserts or replaces text in the focused buffer from
an input surface, and route each through the one wall 575 built (or record it as a NAMED
exemption with a reason — e.g. the capture harness's own replay). Then the census law, per
"same behavior ⇒ same code": a wildcard-free match over the door roster at the seam, so a
NEW door fails to enrol until it declares wall-routed or exempt. The law must fail on the
bug it names: prove non-vacuity by re-opening one existing door (locally, uncommitted) and
watching it go red. Pure unit seam — no capture needed; harness-reach is not in play.

Worker: engineering tier. Worktree per protocol; claim before code.

---

### 581 — substitution-leak audit: what else reports about the hidden document while History/Conflict/Credits is up? (follow-up to 575's lane report, 2026-09-06)

⬜ READY (after 575 merges; audit-tier per standing policy)

Evidence: 575's rail-query leak — typing on Credits filtered the HIDDEN document's rows
and reported "no matches" about a file the user couldn't see. Bugs cluster: the leak class
is "a query/filter/status surface answering about the underlying buffer while a
`TimelineOverComparison` substitution is showing something else." The rail is fixed; its
siblings are unaudited.

Probe form (state × surface × world, per the spot-check policy): the three roster overlays
(History, Conflict, Credits — derive the set from `shows_read_only_prose`, never name it)
× every query/status surface that reads the focused buffer — find & replace, spell
navigation/suggestions, outline/jump surfaces, palette state lines, the debug HUD's
buffer readouts — sampled across ≥2 worlds. Assert per cell with sidecar/pixel arithmetic
via `--screenshot-app` (check docs/harness-reach.md per effect before promising a capture;
flag any live-only cell for human confirmation instead of claiming it). An audit that
finds something ends by writing the missing law; if the audit finds nothing, it still ends
by writing the law that pins the clean state, enrolment derived from the roster.

Worker: audit tier — Sonnet medium on Claude or `gpt-5.6-terra` medium on OpenAI.

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

### 599 — the TextDoor doors still reach the document behind a summoned panel (measured by 585's lane, 2026-09-07; pinned, not fixed)

⬜ READY — the boundary is now visible and law-pinned; this item decides where it should be.

585 closed the ACTION door into the document behind a summoned field. It measured — did not
assume — that the `TextDoor` doors remain open: an IME commit and two assistive writes still
reach the parked document while the panel is up. That is the same boundary `read_only_surface`
already pinned for the picker card, so the two surfaces agree today; a law goes red if that
silently changes.

Build: decide whether a summoned text-entry surface should own those doors too. It is one
decision across both surfaces, not two — and it interacts with 580's insertion-door census,
which is the item that enumerates every path that can mutate the focused buffer at one seam.
Sequence this AFTER 580 or fold it into that census, rather than closing the same door twice
in two shapes.

Laws: whatever is decided, the enrolment comes from the door roster rather than a hand-list,
and the law names what enrolled — 585's own sweep found only 2 of 7 surfaces ever leaked, so
a law that assumes uniform behaviour across surfaces would be wrong in both directions.

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

**Second train, `72e922e1`** — covers 583/584 and 585, taken with HEAD verified unmoved across
the run:

```
native-gate-health status=ok elapsed_seconds=251 mode=real
native-gate-receipt commit=72e922e1400def84b1e4983893548186955fc7f5 health=pass:251s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=4917 unit_shards=6
  integration_targets=18
```
plus `web-smoke: OK`.

**Second train's CI: green.** Run 34050443205 on `c3d26d08` passed all four gating jobs —
`mac (build + test, minus render::tests)`, `web`, `linux (build + test)` and `mac live-probe` —
with only the pinned tolerated `atspi` and `mac (render::tests)` red.

**First train, `5d4819e3`** — covered 571/573, 567, 568/569 and 586/587:
`health=pass:271s conventions=mac,linux scope=all-targets menubar=full:on unit_tests=4903
unit_shards=6 integration_targets=18`, plus `web-smoke: OK`. Pushed as `a7ad4c68`; **CI run
34047161907 passed all four gating jobs**, including the hosted-mac pair — the two reds are
the pinned tolerated `atspi` and `mac (render::tests)`. That hosted arm is the only one that
has ever seen the virtualised-GPU axis, so it is the half of the verification no local receipt
can supply.

⚠️ **Hardware bound, restated because a green receipt is exactly when it gets forgotten:** a
local receipt certifies the dev host's real Apple Silicon Metal. A wedge once stayed green
here while red on hosted macOS for ~140 commits, and CI's lavapipe job stayed green through
that entire streak, so a software adapter is not a stand-in for that axis.

⚠️ **Neither receipt covers a live journey.** Three items merged this wave with their live
confirmation explicitly NOT obtained, because the display was locked; they are in the owed
section, not silently absorbed into these receipts.

## The new gate arms have now run somewhere other than here

CLAUDE.md's standing question of any green check is not only "does this law sweep the right
axis" but **"has this check ever run anywhere but here"** — the other DPI, the other backend,
the other entry point, the other filter. For 593/594/578's three new `code-health.sh` arms
that question is now answered rather than assumed.

CI run 34056753311 on `135f9a5c` passed all four gating jobs, and its **Linux** job ran the new
arms twice — once as the standalone health step, once inside the native full suite:

```
code-health: self-test clean
test-sweep: SKIPPED law 4 (cargo-sweep not installed on this host).
test-sweep: sweep.sh deletes only inside its caller's worktree
test-pycache-guards: 3 by-path loaders leave no scripts/__pycache__
```

Two things worth keeping. Laws 1–3 and the self-test really do run on a second platform, so
they are not dev-host-only. And **law 4 announced its own absence instead of passing
silently** — its lane predicted exactly this (CI installs no cargo-sweep) and designed the skip
to be loud and self-describing. A law that skips quietly reads identical to a law that passed,
and this board has been bitten by that shape more than once.

## Scripts-only merges claim no receipt — and this one says so

`e4d2cf71` (593 + 594 + 578) changed eight files: seven under `scripts/` and
`.orchestrator/README.md`. **No Rust, no shader, no Cargo manifest, no CI workflow** —
verified by diffing the merge's own name list, not assumed from the subject lines. CLAUDE.md
is explicit that such a change claims no receipt and must say so, so this one does.

What DOES stand behind it, both stronger than required: the lane's full native gate on its own
branch tip (`native-gate-receipt commit=c2370fff health=pass:498s conventions=mac,linux
scope=all-targets menubar=full:on unit_tests=4917 unit_shards=6 integration_targets=18`), and
an orchestrator run of `code-health.sh` on the MERGED tree confirming the three new arms
actually execute there — `self-test clean`, `sweep.sh deletes only inside its caller's
worktree`, `3 by-path loaders leave no scripts/__pycache__` — with the clippy-exception count
and ratchet baseline unmoved. A law that is wired but does not run is the failure this repo has
recorded most often; item 567 deleted five laws that nothing ran.

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
