# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 529 — Nishiki-teki: audition a Japanese symbol cabinet, then give each adopted mark one honest purpose (user decision, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-529 (phase 1 only: audit + gallery + Artifact publish, then STOP for the user's taste review before any asset lands)

"It's beautiful"; DECIDED: **Nishiki-teki is the first symbol face awl
should pursue. Character comes before byte count.** This is not a request
for a generic emoji font: the appeal is the Japanese-authored cabinet the
official 4.0.5 release carries — Genjikō/incense patterns, ARIB and Biblos
compatibility marks, lunar/Go/technical notation, early emoji compatibility,
and the stranger pictographs beyond Unicode. The upstream TTF is about
12.3 MB; audition the complete face rather than choosing a subset from names
alone. The downloaded upstream distribution and the font's own metadata both
declare SIL OFL 1.1 (its packaged OFL names no Reserved Font Name); record the
artifact and licence in the existing font ledger before any asset lands.
Do not claim editable design sources: none have been found yet.

The lane owns a GLYPH AUDIT before integration, rendered through awl's real
text pipeline rather than judged from a Unicode chart. Build a review gallery
across representative worlds, light/dark grounds, actual ornament/UI sizes,
and 1x/2x scale. Sample the WHOLE relevant cabinet by range, not a hand-picked
page of known beauties, and identify every sample by code point, glyph/range
name, provenance family, and whether it is standard Unicode or PUA. The
gallery is owed to the user for the taste call; report exact enrolment so a
missing range cannot make the audit vacuously pretty. DELIVERY (user ask,
2026-08-29): publish the gallery as a Claude Artifact page — the captures
embedded with their code-point/name/provenance labels — so the user can
flip through it in a browser rather than opening PNGs by hand. This does
not touch the no-web-artifacts convention: the glyphs themselves are still
rendered by awl's real pipeline via headless capture (never an HTML
re-rendering of the font); the artifact is only the viewing surface for
the taste review.

Classify the findings by an intended product role, with the present hypotheses
treated as questions the rendered evidence may overturn:

- **Thematic-break ornaments — strongest fit:** Genjikō and restrained
  geometric/lunar/technical marks can extend the existing ornament roster.
  They must remain legible at prose size, preserve the calm figure/ground
  hierarchy, and never impersonate a semantic control.
- **Per-world start dress — promising, separate application:** nominate marks
  that could serve item 525's later data-driven start-screen expression. This
  item audits and records them; it does not smuggle in a per-world renderer or
  pre-empt that design session.
- **Document fallback — standard Unicode only:** measure whether Nishiki fills
  real holes in the existing never-tofu ladder without disturbing ordinary
  Japanese text. PUA is never an ambient fallback and never silently changes
  the meaning of a user's file.
- **Semantic chrome — presumption against:** an unfamiliar mark is decoration,
  not a Save/Close/Warning icon. Admit one only if its meaning survives without
  a legend and it is clearer than AwlMarks' existing owner.
- **Insertion into documents — out by default:** inserting Nishiki PUA would
  make nominally plain text depend on awl's private font mapping. Standard
  Unicode symbols may motivate a later symbol-palette decision, not an
  unasked-for feature in this item.
- **Museum only:** vendor/service logos, historical compatibility brands,
  culturally specific religious/occult signs used as generic UI, and the
  elaborate shell/character surfaces may be fascinating gallery material but
  do not become product furniture merely because the font licence permits it.

Deliver a small, named adopted roster and a larger recorded reject/reserve
roster with reasons; "the font has thousands" is not a design system. Only
after that taste review does the lane bundle the upstream face (or an audited
subset if the selected roster makes that obviously better), route it through
one explicit symbol/ornament family rather than the general prose stack,
update `docs/fonts.md` and `assets/fonts/LICENSES.md`, and add laws for licence
enrolment, glyph presence/no-tofu, explicit-family routing, and the rendered
size/contrast of each actual use. Because adding a binary permanently grows
Git history even if reverted, the gallery/taste checkpoint precedes the asset
commit despite the standing land-easy-taste-changes policy.

**PHASE-1 TASTE REVIEW, first pass (user, live over the gallery, 2026-08-30)**
— strong-interest signals recorded verbatim for phase 2; this is a first
pass, not the final adopted roster, and the review is still open:

- **U+F58F–U+F59F (the snake):** "man this snake is cool!"
- **Playing-card symbols, especially Acorn and Bells** (the German suits):
  called out by name.
- **U+F5BC–U+F5FF (Assorted):** the moons here noted with interest.
- **U+F620–U+F62B (Circle, PUA, 12 marks):** "can we somehow use this as
  the loading bar on the web???" — record as a NEW candidate application:
  the web build's load indicator cycling the 12 circle marks (or the moon
  phases) while the wasm downloads. Two mechanics to settle before adopting:
  (a) the loader runs BEFORE awl's fonts arrive, so the marks would need a
  tiny subsetted face (12 glyphs, a few KB) inlined into the loader page —
  a deliberate, named exception to "glyphs render through awl's pipeline",
  confined to the shipped loader and bundled offline (zero-network holds);
  (b) PUA is acceptable here because the loader is awl's own chrome, never
  document content — the PUA prohibitions above are about user files.
- **U+F007B–U+F00FF and U+F0100–U+F01AB (Plane-15 Assorted):** "so many
  cool things here too!"

**SECOND PASS — the AwlMarks-replacement sub-audit (user verdicts over a
dedicated before/after artifact, 2026-08-30).** Measured first: Nishiki's
cmap covers ALL 34 codepoints AwlMarks carries, every pair rendered through
the real stack (survey generator, family/weight parameterised; resolved
family verified per cell, 0 blank, 0 foreign), so full replacement is one
single-source subset — no glyph welding, and the four-source hand-weld
retires. Production reaches the face through `render::SYMBOL_FAMILY` /
`theme::ornament::ORNAMENT_MARKS`, so adoption is an asset swap. Mind
Nishiki's `usWeightClass = 500` in the derived face. User verdicts:
**26 of 34 marks go to Nishiki outright**; keep-current: U+2318 ⌘,
U+2765 ❥, U+232B ⌫; unsure: U+00A7 §, U+2734 ✴, U+2740 ❀; unmarked:
U+2042 ⁂, U+273D ✽. DIRECTION (user): where the standard-codepoint
drawing is weak, **remap a better mark from elsewhere in the cabinet** in
the derived subset face (chrome is awl's own; a PUA-sourced drawing behind
a standard codepoint changes nothing for documents). Nishiki ships no
GSUB stylistic alternates, so alternates are other codepoints' drawings:
a name-hunt found candidates for all six weak marks (shield/endless/
four-petalled knots for ⌘, squared backspace/delete symbols for ⌫,
pilcrows/paragraphos/tengwar section marks for §, plus heart/star/
florette families), rendered and added to the same artifact for the
user's picks — final remap roster still open, awaiting those picks.

**THIRD PASS — closing decisions (user, 2026-08-30):** **⌘ U+2318 goes to
Nishiki after all** — the modifier keys must stay recognisable as the Mac
keys, so no cabinet remap for them; the user's call is to accept the same
sign in Nishiki's different hand ("bite the bullet"). **§ U+00A7 takes
Nishiki's drawing, tentatively accepted** — the user notes it reads like a
currency sign (euro-adjacent); flag it for one live look in real chrome
before final. **⌫ U+232B remains the one open semantic mark** — alternate
pick from the artifact still pending; if none convinces, it stays the
single welded-in exception in the derived face. **❥ U+2765, ✴ U+2734,
❀ U+2740 (and the unmarked ⁂ U+2042, ✽ U+273D) are no longer individual
verdicts** — they dissolve into the per-world ornament-set design pass
(item 536): their product role is per-world ornament data, so the real
decision is which world draws which set, not one global glyph.

---
### 533 — right-anchored hug width is re-measured per lens/filter view, so Kite's Go-to jumps sideways on every lens switch (user report, 2026-08-30)

User report with screenshots, mechanism verified in code before filing.
On Kite, summoning Go-to and stepping lenses (Folders → Headings) slides
the whole card — lens strip, query, rows, footer — hundreds of px
sideways. "Very disorienting."

MECHANISM (verified): Kite and Mangrove are the only
`CardAnchor::TopRight` worlds, the one anchor where `mirrors_growth()`
is true (`theme/model.rs`). For them `overlay_desired_w`
(`render/chrome/overlay.rs`) hugs `overlay_content_w`, which
`set_view` re-measures every frame via `measure_overlay_content_w`
(`render/chrome/roster.rs`) over the CURRENT `overlay_items` — the
lens-bucketed, query-filtered list `OverlayState::refilter` hands the
renderer. Switching lens swaps that roster (long folder paths vs the
bare "no headings yet" empty state), the hug width changes, and with
the right edge pinned the LEFT edge — where all the chrome ink lives —
translates. The same structure means TYPING A FILTER narrows the same
roster and resizes the card per keystroke on these two worlds (derived
from `refilter`'s shape; lane confirms live). The command palette runs
the identical infrastructure and differs only in content variance: its
lenses share one command corpus, so the widest row barely moves.

`measure_overlay_content_w`'s own doc already names the principle —
"a hug width is a property of the picker's CONTENT, and the scroll
position is not content" (that pass fixed the scroll-position flavour
of this exact defect). Extension this item lands: **the lens and the
filter are not content either — the SUMMON's corpus is.** Grow-only
per session is NOT sufficient: `Action::OpenOutline` (`actions.rs`)
summons Go-to pre-lensed onto Headings, so a session can start narrow
and would still jump once on the first step to Files/Folders.

FIX SPEC: measure the hug once per summon over the UNLENSED, UNFILTERED
display roster — Go-to's All home is verified a superset of every lens
(`index.rs`: files, headings, and authored folders "in one fuzzy-ranked
list"), so the union is the All view's display strings, plus each lens's
empty-state line, plus the secondary column labels. Plumb it per the
`ViewState` convention (inert default in `base()`, `sync_view` fails to
compile on the new field). Memoize per (corpus identity, metrics) — the
`roster_memo` pattern in `roster.rs` is the shape; re-measure on
zoom/DPI/corpus change, never per frame; the 2×-cap short-circuit in
`measure_roster_primary_px` already bounds cost on huge rosters.

TASTE TRADEOFF, named so the veto is cheap: stability beats tightest
hug. An OpenOutline summon over a headingless doc gets a card as wide
as the widest file path where today it hugs one short line. The cheap
alternative — drop the hug for faceted pickers and always take the cap —
loses because it rewrites Kite/Mangrove's authored composition in every
session, not just the mixed-lens ones. Revert cost of the chosen fix:
one commit.

VERIFY: (a) unit law at the measure seam — fixed corpus, hug width
invariant across lens index and filter string, enrolment derived from
the ROSTER by `card_anchor.mirrors_growth()` (never "Kite" by name),
non-vacuity proven by flipping back to the per-view measure and watching
it go red; (b) capture pair for the outcome — same seeded `--root` and
explicit `--config`, `--theme Kite`, two `--keys` runs ending on
different lenses, card left edge equal by pixel arithmetic
(`overlay_accept:Goto` is Applied in docs/harness-reach.md; lens
stepping is core-drivable ←/→). Do not ask for a populated
switch-project Recent lens (harness-reach names it impossible).
Rust-touching, so the item claims a full gate receipt.

---
### 534 — uncapped first-line filename derivation makes a long-first-line note unsaveable: "save failed: File name too long (os error 63)" (user report, 2026-08-30)

User screenshot: a fresh note in the notes folder shows the sticky
"save failed: File name too long (os error 63)" over prose whose
paragraphs are single logical lines. ENAMETOOLONG mechanism verified by
arithmetic, not reproduced live: `note_stem` (`buffer/notes.rs`) slugs
the ENTIRE first non-empty line with no length cap, `Buffer::save_owned`
(`buffer/save.rs`) binds `<slug>.md`, and macOS NAME_MAX is 255 bytes
per component. One paragraph visible in the very screenshot (the
"527 + 528" summary line, 285 chars) slugs to 269 bytes — 272 with
`.md` — so any note whose first non-empty line is a prose paragraph
of roughly ≥250 chars can never be saved under its derived name. The
lane's first step is the standing premise check: reproduce with a real
tempdir on the real disk (`InMemoryFs` enforces no NAME_MAX, so the
in-memory seam CANNOT witness this failure), and confirm which save
door raised the visible sticky (manual save / close-flush; the exact
triggering line sits above the screenshot's viewport, so the repro is
constructed, not transcribed).

FIX: cap the derived stem in `note_stem` — the ONE owner every caller
already routes through (first autosave naming, `convert_scratch_and_save`,
web export via `display_name`) — truncating the slug at a dash/word
boundary under a taste budget well below the FS limit (something like
60–80 chars; nobody wants a 250-char filename), byte-aware so a CJK
first line (3 bytes/char) also lands under budget, never ending in a
trailing dash. Headroom arithmetic in the brief, not re-derived:
the atomic-write sibling adds `.{name}.awl-tmp` (10 bytes,
`fs/paths.rs`), the corrupt quarantine adds ~37, and `unique_path`'s
collision suffix a few more — all must fit inside NAME_MAX at the cap.
`unique_path` already disambiguates two notes truncating to the same
stem. `display_name`/export naming inherit the cap through the same
owner — no second rule.

SECOND DELIVERABLE, same neighbourhood (bugs cluster): `autosave_note`
(`app/files/autosave.rs`) swallows the save error silently —
`if let Ok(()) = … {}` with no else — so an unnamed note whose naming
save fails just KEEPS NOT SAVING with zero signal until a manual save
or close-flush finally surfaces the sticky. With the cap the
too-long class vanishes, but disk-full / read-only-dir failures keep
the same silent shape. Decide and land a calm surfacing (a sticky on
the first failure, not a per-debounce nag), or record explicitly why
autosave failure stays silent.

VERIFY: unit laws at the `note_stem` seam (cap honoured; ASCII + CJK +
no-alphanumeric sweeps; dash-boundary + no-trailing-dash), plus a
REAL-DISK tempdir law: a note whose first line is ≥300 chars saves
successfully and its filename length is under the cap — non-vacuity by
reverting the cap and watching that law go red with ENAMETOOLONG.
Rust-touching: full gate receipt.

---
### 535 — sticky notices abandon the writing-column-top slot for the world's toast axis (user decision, 2026-08-30, reverses a documented composition call)

User, looking at the save-failed sticky squatting top-center over the
text: "center top is an annoying place for the toast… we should move it
to somewhere less intrusive." That placement is currently AUTHORED, not
accidental — docs/render.md: "Sticky notices keep the
writing-column-top composition; only self-clearing toasts use the world
axis" — and the mechanism matches: `notice_toast_plan`
(`render/chrome/readout/toast.rs`) plans ONLY `NoticeKind::Toast`,
so `prepare_notice` (`render/chrome/readout.rs`) falls back to
`CornerAnchor::TopCenter` for a sticky. On Kite the transient "saved"
toast already goes to the authored TopRight while the held "save
failed" sits dead-center in the reading line. This item RECORDS the
reversal: stickies route through the SAME authored `toast_anchor` +
`plan_toast` collision/narrow-fallback planner toasts use (merge, don't
align — one placement owner, the `Sticky` gate in `notice_toast_plan`
becomes the one-line diff). Sticky keeps its own plate inks
(`notice_plate_inks` is untouched — lifetime stays expressed by value).

Watch the axes the old composition was carrying: the narrow-canvas
BottomCenter fallback and the picker/outline/workspace collision roster
must hold for stickies too (they come free through `plan_toast`), and
the 1080-cell + 360-cell placement laws in `render/tests/notice.rs`
enrol sticky alongside toast rather than staying toast-only — sweep the
NoticeKind axis, don't imagine it. Update the docs/render.md sentence
in the same commit so the doc and the code state the same rule.
Revert cost: one line (the kind gate) plus the doc sentence — say so in
the commit. Full gate receipt.

---
### 532 — keymap/platform.rs: the seed-table doc comments still describe the Meta-only world

Comment-only truth fix in `src/keymap/platform.rs`, outdated by the
classic-chords round. `active_seed_tables`' doc says "Today just
`LINUX_EMACS_META_SEED`" while the body returns both tables, and one
sentence got garbled in that round's edit ("a seeded layer added for a
future seeded layer (a classic-chords C-x table, say)…") — rewrite it to
describe the two-table present plainly. Sweep the neighbouring seed-table
doc comments in the same file for sibling claims the round outdated (e.g.
`seeded_chords_for`'s "At most one entry today" — verify against the
tables before keeping or cutting it). No behaviour change intended; it is
still a Rust-file edit, so it claims a full gate receipt, and the file
carries a frozen code-health comment baseline — keep the rewrite tight
enough not to trip the ratchet (two prior merges needed a follow-up
"tighten comments back under baseline" commit; don't earn a third).
Cheap enough to piggyback on the next keymap-touching item if one is
dispatched first.

---
### 536 — per-world ornament sets from the full Nishiki cabinet (user decision, 2026-08-30; sequenced AFTER 529 bundles the face)

DECIDED direction, user's words paraphrased with their consent to file:
with the whole cabinet bundled "we can use whatever glyphs we want", so
run a design pass over the ornament assignments of ALL 19 worlds
(GUMTREE…KITE — derive the roster from `theme::worlds`, never a
hand-list). Curate **on the order of ten named ornament SETS** — each a
full `Ornaments` trio (dash/star/underscore) plus the list-bullet pair —
drawn from Nishiki's standard AND PUA cabinets (PUA is fine here: ornament
chrome, never document content), then assign sets to worlds so overlap is
minimal but sets still repeat (~10 sets over 19 worlds ≈ each set worn by
about two worlds). Explicitly NOT per-world bespoke machinery ("we're not
doing theme-specific stuff just yet"): the mechanism is unchanged —
`theme::ornament::Ornaments` is per-world const data and stays exactly
that; this pass only re-picks the data with a vastly richer palette.
Candidate sets that fit worlds' existing temperaments: Genjikō incense
patterns, the moons (U+F5BC–F5FF neighbourhood), the snake range
(F58F–F59F), German playing-card suits (Acorn/Bells), lunar/Go/technical
notation, plus the existing star/floret/geometric vocabulary in Nishiki's
hand. ❥ ✴ ❀ ⁂ ✽ from the AwlMarks verdicts land here rather than as
global picks. Delivery mirrors the marks audition: sets rendered through
the real pipeline at true ornament sizes on each candidate world's real
ground, published as an Artifact taste sheet for the user's set-to-world
assignment before any data lands. Laws follow the existing ornament laws:
legibility at prose size, figure/ground calm, per-world size/contrast
sweep derived from the world roster (no wildcard), and the subset-face
glyph-presence law regenerated from the adopted union so a set's glyph
cannot silently vanish from the derived face.

---
## Needs specific hardware

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
