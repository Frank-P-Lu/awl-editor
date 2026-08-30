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
