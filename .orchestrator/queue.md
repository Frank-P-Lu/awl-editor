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
