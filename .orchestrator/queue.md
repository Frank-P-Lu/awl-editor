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

🟡 IN PROGRESS — Claude (this session), branch `item-537`, worktree `.claude/worktrees/item-537`.

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

### 588 — list-bullet pairs derive from each world's worn ornament set (carried out of 536's fold, 2026-08-30 decision)

🟡 IN PROGRESS — Claude (this session), branch `item-588`, worktree `.claude/worktrees/item-588`.

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

### 592 — Settings: compact label/value relationships and readable workspace hierarchy (user approval, 2026-09-07; MERGED then REVERTED 2026-09-08)

🟡 IN PROGRESS — Claude (this session), branch `item-592`, worktree `.claude/worktrees/item-592`.
The branch is intact and the work is good; it comes back with one defect, not a rejection.

⬜ The merge train reverted it. `git bisect` between the last green receipt and HEAD names
`3a4c0ac5` — 592's own commit — as the first bad one, and it reproduces deterministically,
alone, single-threaded:

```
AWL_MENU_BAR_FORCE=on cargo test --bin awl -- --exact --test-threads 1 \
  render::tests::range_rail::the_rail_reads_against_its_ground_in_light_and_dark_worlds_real_pixels
```
```
Bombora (selected=true): the TRACK must paint something distinct from its ground
```

The law scans the rail's own row and samples its GROUND 14px past the rail's right end. Every
sample along the rail then matched that ground exactly, so no track ink was found at all. The
suspicion — for the lane to confirm or refute, NOT to inherit — is that clamping the content
pane to 72 chars moves what sits 14px past the rail: either the sample is no longer on the
card, or the rail's painted extent no longer matches the extent `overlay_range_scale` reports.
The second would be the same class of bug 591 was reverted for in the same wave.

**Why this is worth more than the fix.** 592's lane verified its ceiling LIVE, at five window
widths from 1200 to 3600, and watched the pane hold flat at 881.28px. That was real work and
it still missed this, because all five ran on this host's ambient menu-bar branch, which on
macOS is OFF. The `menubar-full` arm exists precisely because a macOS host never runs the
branch every Linux host and every CI run always take. A check runs in one configuration, and
the configuration is the hypothesis.

Keep everything else: merging the two drifted placard predicates into one owner
(`placard_style_applies`) is better than the item asked for and should come back with the fix.

---

### 603 — what should selecting inside a substituted transcript do? (named by 581's audit, 2026-09-07, and deliberately left unfixed)

🟡 IN PROGRESS — Claude (this session), branch `item-603`, worktree `.claude/worktrees/item-603`.

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

### 616 — table selection: neighbouring cells flicker while the band settles (user report, 2026-09-08)

⬜ READY. The user confirmed 551's whole-row band is what they want, and reported one defect on it in their own words: "the neighbouring cells kind of flicker, I don't like that." Cells beside the selected run change appearance transiently while the selection moves. Read the cause out of the tree before fixing — 551 landed in `f740749c` (`render/rects.rs`, `render/geometry.rs`, law in `render/tests/table_selection_band_law.rs`) with a follow-up in `db90497e`; find what animates or re-paints on the untouched cells (a band ease, the table x-ray's grid float, or a cache invalidation that re-shapes the row). **The user's fallback, stated plainly: if the flicker cannot be cut out on its own, remove the animation on the table band altogether.** A calm still band beats a lively one that flickers.

Laws: over a selection that grows one cell at a time, the pixels of every cell OUTSIDE the band are byte-identical frame to frame (sweep the roster, not one world); the band itself still paints whole rows. Prove non-vacuity by re-introducing the transient and watching the law go red. The feel is live-only; deliver a motion capture (`--screenshot-motion`) and flag the live look as owed.

Routing: worker Sonnet high (Claude) or `gpt-5.6-sol` high; outcome audit at the production tier.

---

### 620 — a law locates its subject by relative path, so it passes only from the crate root (found by 608's gate, 2026-09-08)

⬜ READY — small, reproduced in one command, and it is the configuration principle in its
purest form: the law is correct and the working directory it runs in is the untested
hypothesis.

`app::semantic::tests::semantic_snapshot_has_no_ungated_frame_side_caller`
(`src/app/semantic/tests/mod.rs`) walks the tree from `PathBuf::from("src")` — a RELATIVE
path — so it asserts over the crate only when the test process happens to start in the crate
root. Run the same built binary from anywhere else and it panics `src is readable: NotFound`.
Reproduced directly: green from the worktree, red from `/tmp`, same binary, same commit.

Two defects, not one. **(a) The subject is located by cwd** rather than by
`env!("CARGO_MANIFEST_DIR")`, which is fixed at compile time and is what every other
source-walking law in this tree should be checked against too — sweep for siblings, because
this one was found by accident. **(b) The failure misnames itself.** The
`.expect("src is readable")` fires for EVERY directory the walk pops, so whichever directory
actually failed, the message says "src". A law that cannot name what broke costs a reader the
diagnosis it was written to give.

It went red once, during a gate run while the orchestrator was deleting fifteen worktree
`target/` trees and the host was at load 100 with 9 GiB of swap in use. That is the trigger,
not the cause — a law anchored at the manifest directory would not have noticed.

**The sweep is already done — this is the census, so the lane spends its round fixing rather
than finding.** Four sites assume the crate root is the working directory, and two of them say
so in a comment without ever asserting it, which is the tell:

| site | shape |
|---|---|
| `src/app/semantic/tests/mod.rs:111` | `PathBuf::from("src")` — the walk that fired |
| `src/app/semantic/tests/mod.rs:165` | `read_to_string("src/render/chrome/hud.rs")` — same file, same defect |
| `src/app_icon/tests.rs:24` | `fn root() -> PathBuf { PathBuf::from(".") }`, commented *"Tests run with CWD == the crate root"* |
| `src/icon_manifest.rs:295` | the same comment, *"Tests run with CWD == the crate root."* |

The correct pattern is already in this tree and needs no invention:
`src/module_map_law.rs:82` uses `PathBuf::from(env!("CARGO_MANIFEST_DIR"))`, which is fixed at
COMPILE time and therefore cannot be moved by a runner, a shard, or a shell. `println_audit`,
`macos_identity_law`, `embedded_docs_law` and `roster_claim_law` all read the same way.

Build: anchor all four at `CARGO_MANIFEST_DIR`; make the expect name the directory it was
actually reading rather than always saying "src". Delete the two comments — an assumption
stated in prose and asserted nowhere is the thing being retired, and leaving the comment
beside a fixed call site would preserve the wrong idea.
Law: the walk resolves identically from a different working directory — assert it by running
the check function with the process cwd changed, so the law fails if someone reintroduces a
relative root.

Routing: worker Sonnet medium; no audit tier needed — the fix is mechanical and the sweep is
the valuable half.

---

### 621 — the cpu-spin law asserts an absolute 50% floor, so a busy host cannot earn a receipt (blocked a merge train, 2026-09-08)

⬜ READY, and it is currently BLOCKING: while several lanes build, no gate on this machine can
issue a receipt at all, including a lane's own.

`scripts/test-native-gate.sh`'s cpu-spin probe launches a fixture that deliberately spins, then
asserts the vitals heartbeat saw a tracked process peak at `>= 50` percent of a core. The
comment beside the floor explains where 50 came from: `ps -o time=` quantises to whole seconds
on Linux, so a 3s window can under-read a pegged process by about a third. That is a
MEASUREMENT correction, and it is correct as far as it goes.

What it does not contemplate is a host where the fixture cannot GET a core. On a ten-core
machine running seven concurrent lane builds the spinner peaked at 33.3%, and the law failed —
truthfully reporting that the busiest tracked process was not pegged, which was simply the
fact. The law is not wrong about what it measured; its unstated precondition is an idle host,
and this fleet's whole design is to not have one.

This is the configuration principle again, one turn further in: the law states its
measurement correction in a comment and never states the precondition that the correction
assumes. Compare 620 — same shape, different axis (working directory there, host load here).

Build, and the choice matters. The weak fix is a bigger tolerance, which only moves the load at
which this recurs. Two better ones:

- **Assert the relationship, not the absolute.** What the law actually wants to know is whether
  the heartbeat's reported percentage TRACKS the fixture's real CPU consumption. Compare the
  heartbeat's figure against the fixture's own cumulative CPU time over the same window and
  require them to agree within a band. That is true on an idle host and on a loaded one, and it
  still fails if the heartbeat stops seeing the process — which is the defect the law exists to
  catch, and the one a raised tolerance would start hiding.
- **Or detect contention and skip LOUDLY**, naming the load average and saying which law did not
  run — never silently, and never by passing.

Whichever is chosen, the probe must print the configuration it ran in — load average and core
count — so a reader can tell a real regression from a busy afternoon without re-running it.

Non-vacuity is the interesting half: prove the new form still goes red when the heartbeat
genuinely loses sight of a spinning process, which is the original defect (a receipt run once
reported `tracked_procs=0` and `0.6%` while two test binaries burned a core each).

Routing: worker Sonnet high — the fix is small, the oracle design is not.

---

### 622 — the theme-picker chrome pin is a swappable global outside testlock's one field list (found reviewing 609's merge, 2026-09-08)

⬜ READY — small, and it is the leak class this repo has already paid for once.

609 added `PICKER_CHROME_PIN`, a thread-local world index that six `effective_*` resolvers
read instead of `theme::active()`. The design is right and the module comment argues
correctly that a thread-local can skip the lock: it isolates `cargo test`'s parallel worker
threads from each other, which is the property the process-global guard exists to supply.

The gap is not the lock — it is the RESTORE. `testlock`'s anti-leak design works because its
snapshot, its leak audit and its restore share ONE field list by construction, so a global
cannot be added to the snapshot and forgotten by the audit. This pin is in none of the three.
A test that pins and then panics before unpinning leaks the pin to the next test on that
worker thread, and the next test reads a `list_style`, `chrome_face` or `location_style` it
did not choose.

That is exactly the shape of the leaked `ListStyle::Bars` that once made an unrelated
jump-hint law report a clip that was not one — green single-threaded, red under a wide
`--test-threads`, and blamed on the wrong law for a while. The pin's own law does unpin and
re-pin around its mutation arm, but on the HAPPY path only; an assertion failing between those
two calls leaves the pin set.

Build: bring the pin under the same discipline as the other swappable globals — either into
`testlock`'s shared field list so the snapshot, the audit and the restore all see it, or
behind a guard object whose `Drop` restores it on the unwinding path too. Prefer the shared
list: a second mechanism here is how the first one drifted.

Law: a test that pins and then panics must not leave the pin set for the next test on that
thread. Prove non-vacuity by removing the restore and watching it go red — and run it under a
wide `--test-threads`, because that is the configuration where this class shows up at all.

**A SECOND GLOBAL OF THE SAME CLASS, found by 591's repair lane and not by looking for it.**
`search::LAST_QUERY` is a process-global that `actions::motion::start_search` — the real
`C-s`/`OpenReplace` production path — prefills from, and `testlock`'s guard does not snapshot
or restore it either. Tests that neither hold the lock nor call `clear_last_query()` inherit
the previous test's query: two laws asserting `Some("h")` got `Some("alphah")`. Those tests
were repaired to match the convention their siblings in the same file already followed, but
the structural fix belongs here, with the pin.

That makes this item about the LIST rather than about one global. Two were found in one day,
neither by searching for them, which is the shape of a class rather than a pair — so the
useful deliverable is a way for a new swappable global to be caught at birth, not two more
entries typed into a table. Sweep for others while you are here and report what you find.

Routing: worker Sonnet medium.

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
