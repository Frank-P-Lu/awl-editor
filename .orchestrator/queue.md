# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 529 — Nishiki-teki: audition a Japanese symbol cabinet, then give each adopted mark one honest purpose (user decision, 2026-08-29)

🟡 IN PROGRESS — gpt-5.6-sol high, branch codex/item-529-phase2 (phase 2: derived-face roster workflow, asset swap, and the decided single-source adoption; phase-1 taste gates are closed; the old item-529 worktree is read-only evidence because its temporary gallery commits diverged before the later decisions)

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

**BUNDLING STRATEGY (decided with the user, 2026-09-01):** the 12.3 MB
upstream face is NEVER bundled; the ONE derived subset face IS the
adoption mechanism. Union roster = the 34 chrome marks + 536's final
64-glyph ornament table + 537's reference ladder (⁎ † ‡ § ‖ ¶ — U+2016
verified in cmap) ≈ ~90 unique codepoints, tens of KB. The roster is a
checked-in file; a script regenerates the face from the upstream TTF
(recorded by sha in the ledger, kept out of git); later adoptions (525
start dress) are one-line roster additions. The derived face KEEPS the
family name "Awl Marks" — SYMBOL_FAMILY/ORNAMENT_MARKS untouched, pure
asset swap; OFL-clean (only "Nishiki-teki" is off-limits as a name) —
and NORMALISES OS/2 weight to 400 inside the face, killing the
weight-500 trap at the source instead of compensating per call site.
Document fallback: decided NO on the lane's own measurement (621
Japanese-relevant additions, all rare variants); revisit only on a real
tofu report. Once all twenty worlds wear Nishiki, the Garamond and
Junicode ornament REGISTERS are dead machinery — 536's build cuts them,
and Junicode-Ornaments.ttf is likely removable (lane greps consumers).
The § live-look in real chrome stays the user's one checkpoint after
the subset lands; the 12-circle web-loader face remains separate.

**ROSTER WORKFLOW (user requirement, 2026-09-01: "we want to make it
easy to add/remove new symbols in the future").** Confirmed with the
user: the upstream face is NOT bundled, ever — and the add/remove path
is a first-class deliverable of the build, not an afterthought:
(a) ONE tracked roster file (beside the font asset) lists every adopted
glyph — codepoint, name, role, source range — and editing THAT file is
the entire act of adopting or retiring a symbol; (b) ONE script
regenerates `AwlMarks.ttf` from the roster + the upstream TTF (passed
by path, never fetched — zero-network; its sha256 verified against the
ledger's recorded value before subsetting), normalising weight to 400
and writing the family name + OFL metadata; (c) the glyph-presence law
derives its enrolment FROM the roster file, never a hand list, so a new
line is law-covered the moment it lands; (d) a drift law pins the
bundled face's cmap EXACTLY equal to the roster — a glyph in the face
but not the roster is as red as the reverse; (e) consumers (ornament
tables, chrome constants, the footnote ladder) reference glyphs the
roster can see, so a removal fails loudly at test time rather than
rendering tofu at runtime. Acceptance: adding one symbol = one roster
line + one script run + one small commit, with every law green and no
other file touched by hand.

**JAPANESE-TEXT QUESTION (user ask, measured + auditioned, 2026-09-01):**
"should we use nishiki for one of the japanese fonts?" Audited rather
than assumed — the first time the face's LETTERS went through the
pipeline (the census had excluded letter scripts): complete kana
(86/86 hiragana, 90/90 katakana, 0 blank, all resolving to Nishiki-teki)
plus four running sentences beside all three bundled Japanese faces
(Klee One / Noto Sans JP / Noto Serif JP), published as a dedicated Kana
Audition artifact. Finding: the hand is a chunky rounded DISPLAY voice —
poster, not page. DECLINED for the prose/never-tofu ladder on four
grounds: display register; single weight 500 (no bold companion);
partial kanji (6,083 of the unified block); and the measured gap it
would close is 621 rare variants the ladder already declines by policy.
RESERVED as a candidate DISPLAY voice (start-screen dress for a
JP-flavoured world, theme-preview line, headings-as-dress) — one honest
purpose, never ambient — pending the user's read of the audition.

**THIRD PASS — closing decisions (user, 2026-08-30):** **⌘ U+2318 goes to
Nishiki after all** — the modifier keys must stay recognisable as the Mac
keys, so no cabinet remap for them; the user's call is to accept the same
sign in Nishiki's different hand ("bite the bullet"). **§ U+00A7 takes
Nishiki's drawing, tentatively accepted** — the user notes it reads like a
currency sign (euro-adjacent); flag it for one live look in real chrome
before final. **⌫ U+232B goes to Nishiki too (user, follow-up)** — so
ALL 34 marks resolve to Nishiki and the derived face is a pure
single-source subset with zero welded-in exceptions. **❥ U+2765, ✴ U+2734,
❀ U+2740 (and the unmarked ⁂ U+2042, ✽ U+273D) are no longer individual
verdicts** — they dissolve into the per-world ornament-set design pass
(item 536): their product role is per-world ornament data, so the real
decision is which world draws which set, not one global glyph.

---
### 542 — table editing: row/column palette verbs (user report, 2026-08-30 — "kinda awful to edit"; fruit 1 landed, this is fruit 2)

🔴 BLOCKED — the remaining row/column palette verbs need the user's greenlight

Three waves have landed: Tab/Shift-Tab moves between cells and wraps
across rows, Tab on the final cell appends a scaffold row, Enter
inside a table inserts a matching scaffold row, and (2026-08-31)
`align_table`'s re-pad now fires automatically on row-leave instead
of requiring the command by hand — undo-isolated (always its own
sealed group) and caret-preserving (a logical cell+offset pair,
`markdown/table_caret.rs`, invariant under the padding-only rewrite).
The greenlit 2026-09-01 remedy is landed too: Tab/Shift-Tab selects
the next non-empty cell's trimmed contents so typing replaces it,
while an empty/scaffold cell receives a bare caret immediately after
its opening pipe. First/last cells, append-row, reverse motion,
selection replacement, and a real `--keys` sidecar journey are
covered; code-health, wasm smoke, and the full native suite are green
on the merged candidate.

**Remaining fruit, not yet greenlit — needs the user's word:**
Row/column verbs in the palette: Insert row above/below, insert
column left/right, delete row/column — source splices over the
existing row/cell parser (`markdown/tables.rs`), gated to
caret-in-table exactly like `AlignTable`'s availability gate.

NOT this item (the big arc): editing cells IN the grid without
dropping to source. That is the "tables as real grids" destination
and earns its own design session; the remaining source-level
operations survive it and would also serve the grid editor.

---
### 543 — TableDims picker: frost only the card's footprint, never the whole page (user decision, 2026-09-01)

The user likes the picker; the frost extent is the complaint, in their
words: when the picker is active "there shouldn't be any blur — or
rather there should only be blur underneath the picker." Today the
dimension picker takes `blur::Frost::Full` and defocuses the entire
document for a card the size of a postage stamp.

**The mechanism already exists — this is an enrolment decision, not new
plumbing.** The footprint arm (`blur::Frost::Footprint`: mask in the
composite's alpha, feathered skirt, optional shear) already serves the
theme picker, the caret picker, the pointer-anchored context menu, and
the Diagonal spell popup. The one owner of the decision is
`TextPipeline::frost_mode` via `overlay_declines_takeover`, and the
constraint the lane must respect: the crisp set is
`OverlayKind::keeps_backdrop_crisp`, LAW-PINNED EQUAL to
`previews_live_document` over `OverlayKind::ALL`. TableDims does NOT
preview the live document, so do not buy the exemption by lying in that
predicate — give it its own honest reason to decline the takeover (the
same shape as the contextual menu's: a small caret-anchored insertion
card is not a takeover; the insertion point it serves is on the page it
would otherwise blur) as a new door beside `overlay_crisp` and the
contextual arm — the equality law stays untouched, and the new reason
gets its own roster law naming which kinds it enrols.
On plated compositions `footprint_frost_applies` already answers "no
frost at all," which is right — TableDims draws its own plate there
(verified by capture on the default world); the bare-composition arm
gets the footprint. `--keys "Cmd-P i n s e r t Space t a b Enter"`
reaches the open picker headlessly (sidecar `overlay.mode:
"table_dims"`), both drivers.

**Scope fence:** the user generalized — "we use blur in this way in a
lot of places, the theme picker comes to mind" — but the theme and
caret pickers ALREADY footprint, and the genuinely modal takeovers
(palette, go-to, outline, keybindings, spell list) keep `Frost::Full`
by design (the card is the subject there). The lane audits the
full-frost roster and REPORTS which members are small-card takeovers
like TableDims; it flips only TableDims without a further user call.

**Settled by the user (2026-09-01): the caret drawing over the frost is
fine** — "it's just like you shouldn't frost the whole page, it should
just frost the picker bit." So no taste question remains here; the item
is purely the frost-extent enrolment. (For the record: the user's live
screenshot showed the caret's whole table row crisp over the frost,
which headless replay of the same state — both drivers — does not
produce; the user approves the live feel, so no work is owed on it
unless the lane trips over a stale-frost class while in there.)

Verify: footprint-vs-full is unit-testable at `frost_mode`'s seam;
pixel arithmetic over captures for "page crisp outside the skirt,
frosted under the card" on a bare world; the existing frost laws sweep
the new enrolment. Full gate receipt.

---
### 544 — footprint frost's box far exceeds the drawn card on upright plate-hugging compositions (user report, 2026-09-01 — "this bounding box is way too big"; reproduced headlessly)

🟡 IN PROGRESS — gpt-5.6-sol high, branch codex/item-544-frost-box

Two user screenshots, both Firetail: the theme picker with a
band-shaped blur patch hanging well to the right of its narrow row
plates, blurring document mid-glyph nowhere near the card; and the
pointer-anchored two-row menu ("Go to folders… / Open file…") with a
frosted rectangle several times the menu's size, offset up-and-right
of it. The user's word: the blur "looks weird" and the box is "way
too big."

**Reproduced deterministically, so no live hunt is needed:**
`--theme Firetail --keys "Cmd-T"` over a ten-line prose file. Measured
on that capture (1200×800, dpi 1): the sidecar's `overlay.window.band`
is x 127.5, w 545, and the frosted region matches THAT band (right
edge ≈672+feather), while the drawn surfaces — label-hugging plates —
span only ≈133..286 plus the foot-hint card ≈133..530. The frost also
reaches the query head's row near the canvas top, far above the first
row plate. Wagtail is fine for the 1-bit reason (`Backdrop::Flat`
forgoes frost entirely); a Firetail-class world is the exhibit.

**Where to look, held as pointers rather than a verdict:** the
narrowing machinery exists and is documented as exactly this fix —
`blur/narrow.rs` (`footprint_narrow`, X-only, plus the diagonal-only
bottom trim) fed by `chrome/overlay_ink.rs::overlay_drawn_surfaces`,
composed in `pipeline_prepare.rs::footprint_drawn_box`. It
demonstrably does not bite on Firetail's upright `ListStyle::Bars`
composition. Note two of its own confessions: the seat-top step's
comment says the prior frost item's "own audition never reached an
upright world", and term (4) of `overlay_drawn_surfaces` pushes a
FULL-BAND surface whenever `overlay_rule_spans` answers — either the
narrowing never runs for this composition, a band-wide term defeats
it, or the box it starts from is not the one measured here. The lane
establishes which with the same two-frame diff used to file this.

The fix must sweep the axis the original work didn't: every
composition in the roster (Pane/Card, Bars, Ruled, Diagonal, plates
that hug vs fill), derived from the theme roster rather than named
worlds, asserting per world that the frosted region's bounding box
stays within the DRAWN card's box plus feather — with the presence
floor the frost laws already know (a frost that vanishes entirely
also satisfies a too-big bound). The pointer-anchored menu is a
second geometry family and gets its own probe cell. Full gate
receipt.

---
### 545 — smart-punct conceal reserves a giant slot: off-caret `...` leaves an ~7-char hole in the line (user report, 2026-09-01 — "moving down!!! there's that giant space being created?"; reproduced headlessly)

🟡 IN PROGRESS — gpt-5.6-sol high, branch codex/item-545-smart-punct

Caret leaves a line containing a literal `...` and the concealed
render opens a huge blank run where the dots were; the caret coming
back reveals the raw line and the hole vanishes — so ordinary
Down-arrow motion visibly reflows the paragraph the user just left.
The user then spotted the trigger themselves: "oh wait it comes from
a ... misrendering i think." Correct.

**Reproduced with a one-line fixture, no wrap involved:**
`short line but also... there is more`, caret moved off the line
(`--keys "Down Down"`, Firetail, dpi 1). Renders as
`also…⟨~7-char gap⟩there is more`. Two visible defects in one
mechanism, `ConcealKind::SmartPunct`'s painted-substitute slot
(`markdown/conceal.rs`; render side `render/spans/conceal.rs`, the
`SmartPunct` arm near line 474):

- **The reserved slot's advance is far wider than the substitute
  glyph** — the span neither collapses to the `…` glyph's own width
  nor keeps the literal's 3-char width; the hole reads as ~7 chars.
  On wrapped lines (the user's screenshots) the oversized slot also
  moves the wrap points, which is why leaving the line visibly
  reflows it.
- **The substitute paints small and grey** — the `…` renders well
  below body size and in dim ink. USER CALL (2026-09-01): "should we
  render it the same colour as the text? ... in grey is a bit
  distracting" — so the substitute renders at full content size and
  the text's own colour, which is also what the kind's own doc
  already promises (real sentence punctuation, carved out of the
  dim-markup rule).

The lane sweeps the WHOLE `SmartPunctKind` roster, not the reported
case: `--`, `---`, and `...` each probed on-caret (literal bytes,
plain prose) and off-caret (substitute at the substitute's own
advance), short line and wrapped line, and asserts the off-caret
row's total advance shift is within one glyph of
(substitute − literal). The law must go red on today's tree before
the fix lands. The dash arm is CONFIRMED broken the same way, not
just likely (user screenshot, 2026-09-01: "the gap is kinda long?
like after the em dash", source line verbatim
`A long sentence--- after all, we all need one.` — the `---` run
renders `—` plus roughly two spare characters of slot, in the same
dim ink; i.e. the slot keeps the LITERAL's 3-char advance where the
substitute is 1). Use that line as a law fixture. Note the measured
`...` hole was ~7 chars against a 3-char literal, so the two arms
may overshoot by different arithmetic — measure each, don't assume
one constant. All three runs share the fix and the law. Full gate
receipt.

---
### 546 — Cassowary's docked palette tabs: a design pass on the DockedTab facet (user taste report, 2026-09-01 — "the tabs for cassowary don't look super convincing lol")

The palette's category strip on Cassowary. The mechanism is
`FacetStyle::DockedTab` (`theme/model.rs`), whose own contract is
"the active label is a tab JOINED to the card's top border; inactive
labels remain unplated text" — and the join is exactly what the
rendered frame fails to sell: the active label sits as a solid
filled block butted against the card's UNBROKEN top border, so
nothing visually connects tab to card, and the inactive labels read
as a loose menu-bar of bold words rather than the same strip's
other tabs.

A taste pass, not a bug: the composition lives in the Cassowary
console chrome (`render/tests/cassowary_console.rs` and
`docked_facet_gap_law.rs` pin today's geometry; docs/render.md's
RotatedRail note records that the docked active tab owns the
category label). Directions to audition — rendered through real
captures, judged against the CRT-console character, never an HTML
mock: (a) make the join REAL: break the card's top border under the
active tab so tab and card share one continuous outline (the folder-
tab idiom the name promises); (b) consider an OUTLINED tab over a
filled block, matching the card's own hairline-and-fill grammar;
(c) recede the inactive labels a value so the active one carries the
strip. Per the land-easy-taste policy, land the best candidate on
main for the user's judgement and say what reverting costs; existing
docked-facet laws move WITH the design, updated in the same commit.
Scope: Cassowary's DockedTab only — the other facet styles (Text,
Band, Chips) have no report against them.

---
### 536 — per-world ornament sets from the full Nishiki cabinet (user decision, 2026-08-30; sequenced AFTER 529 bundles the face)

✅ DESIGN PASS COMPLETE (user approved the arrangement, 2026-09-01: "they look good! lets queue this!") — build phase UNCLAIMED, sequenced after 529 bundles the face. The FINAL block at the end of this item is the decided roster; the v-notes above it are the fitting history.

DECIDED direction, user's words paraphrased with their consent to file:
with the whole cabinet bundled "we can use whatever glyphs we want", so
run a design pass over the ornament assignments of ALL 20 worlds
(`theme::worlds::THEMES` is `[Theme; 20]`, Cassowary included — derive
the roster from THAT array, never a hand-list; a grep over `worlds.rs`
alone missed Cassowary's own module and produced a wrong count of 19,
already once). Curate **on the order of ten named ornament SETS** — each a
full `Ornaments` trio (dash/star/underscore) plus the list-bullet pair —
drawn from Nishiki's standard AND PUA cabinets (PUA is fine here: ornament
chrome, never document content), then assign sets to worlds so overlap is
minimal but sets still repeat (~10 sets over 20 worlds ≈ each set worn by
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

**DESIGN PASS DELIVERED (2026-08-30):** the fitting-room Artifact is
published and awaits the user's arrangement. Ten sets curated and rendered
(0 blank, 0 foreign, all resolving to Nishiki-teki): Genjikō (F500/F50C/
F51B), Lunar (F640/F648/F650), Riverbank — upstream's JUSTIFYING SNAKE
trunk/head/tail, drawn to JOIN into one creature, a fact a later break
style could exploit — (F592/F593/F591), Tavern acorns/bells/leaves
(F5B0/F5B3/F5B1), Arabesque (F814/F818/F827), Stars (2726/2736/2727),
Florets (273F/2741/2740), Geometrics (2756/25C8/2B25), Fleurons
(2767/2619/2766), Trigrams (2630/2632/2637). The circle dozen stays
reserved for the web loader. Every stage is real: ground bands cropped
from per-world --capture-dpi 2 captures at the rule row's own sidecar
geometry (a heading-only twin capture supplies the clean band), ink
sampled from each world's actual rendered ornament, glyphs rendered at
the world's own scale tier. A proposed assignment ships in the page
(sets worn by ~2 worlds each); the user reassigns live and copies the
arrangement out. Data lands only after that arrangement returns.

**FITTING ROOM v2 (user feedback round, 2026-08-31):** the snake JOINS —
tail+trunk+head shaped as one run through the real pipeline compose into
a single continuous creature with no seam, and the trunk repeats for
length; the arabesque wave segments join into swags the same way.
Riverbank's proposal is now: `---` the whole joined snake, `***` the head
alone, `___` the tail. SHIPPING a joined break means `Ornaments` fields
widen from `char` to a string — a small, conscious model change the lane
must own, not assume. Genjikō opens on the user-requested 1 2 3 4 5
ground pattern (U+F501). Bombora re-proposed to Arabesque (user: moons/
circles don't fit; waves suit the reef world), Bilby to Fleurons. NEW
eleventh set from the user-flagged U+FEFD3 block: Dovecote (right dove
FEFE1 / olive-branch dove FEFE3 / left dove FEFE2), proposed for Brolga.
Same block also holds acorn fleurons, compass manicules, and palm
branches — recorded as reserve. U+FF000–FF0B7 browsed with the user:
largely museum under the standing clause (religious signs, brands,
pictographs), a few ornament-viable bursts/spirals left in reserve.
Arrangement still pending the user's copy-out.

**FITTING ROOM v3 (user push-back round, 2026-08-31):** three user calls,
all in. (a) Snake parts retired — "if you have the full snake why would
you have part of the snake?" — Riverbank is now THREE LENGTHS of one
whole creature: `___` hatchling (tail+head, joins seamlessly with no
trunk), `---` snake, `***` grand snake. (b) Trigrams cut on principle,
not visuals: the user asked for a compelling non-visual reason, and the
honest answer is 529's own museum clause (divinatory signs as generic
UI) argues against them — replaced by MANICULES (U+261E/261D/261C), the
printer's own five-century margin mark, proposed for Wagtail/Currawong.
(c) Lunar replaced by MOONFACES (U+1F31D/1F31B/1F31A, the moon with a
face) per "we can replace the lunar one" + the ask for something playful
from the browsed blocks; Mopoke wears it. NEW twelfth set SPLATTER
(U+FF04E/FF04F/FF04D — ink splashes), proposed for Cassowary's metaball
world. Genjikō's 1 2 3 4 5 (U+F501) confirmed present since v2 — the
user was on a cached v1. Twelve sets, arrangement still pending.

**FITTING ROOM v4 (2026-08-31):** user: one snake only. Riverbank is now
a bestiary at three speeds — `---` the long joined snake (SNAKE4), `***`
a butterfly (U+1F98B), `___` a snail (U+1F40C). Arabesque takes the
user's one-joined-plus-two shape: the joined white swag (F814+F815+F816)
for `---`, a white scroll (U+F81C) and black wave (U+F827) for the
others — the old F818 slot was WAVE-1 mirrored, user caught it.
Manicules swapped out on request; replacement is TALLY (U+F59B/F59D/
F59A, Japanese box-tally strokes — a break that counts sections, the
non-visual reason the trigrams lacked), proposed for Wagtail/Currawong.
Moonfaces confirmed great by the user. Caslon foundry ornaments
(FEFD6/FEFD8/FEFD9 chain/cartouche/rosette) rendered and RESERVED —
lovely but ornate for the stark worlds. Twelve sets, arrangement pending.

**FITTING ROOM v5 — the expansion shelf (2026-08-31):** user asked for
~20 more sets to browse. Riverbank's butterfly (user: doesn't fit)
became a FISH (U+1F41F) — snake/fish/snail, all river creatures.
Arabesque's joined runs tightened to the user's exact pairs: F814+F815
white for `---`, F827+F828 black for `***`, scroll U+F81C for `___`.
Twenty-two extra sets added to the shelf, every glyph pipeline-rendered
and family-verified (60 cells × 3 tiers, 0 blank, 0 foreign): Lunar,
Manicules and Caslon return as options; new: Genjikō II, Keizuko, Palms,
Acorns, Hearts, Snow, Wish, Spirals, Cardtable, Gambit, Songbook,
Tallybars, Heraldry, Autumn, Hanami, Undergrowth, Solar, Harbour,
Asterism. 34 sets total; proposal unchanged (extras sit unworn). The
page's masks were deduplicated into shared CSS classes, so the tripled
shelf SHRANK the artifact (3.9→2.0 MB). Cabinet gaps recorded: no frog,
no turtle anywhere in the face. Arrangement still pending.

**FITTING ROOM v6 — story-led proposal from the user's full shelf review
(2026-09-01).** The user graded all 34 sets. Loved: moonfaces, riverbank,
splatter, wish, spirals, undergrowth, solar, songbook, dovecote; nice:
tavern, arabesque, florets, cardtable, gambit, autumn, hanami, palms;
fine-not-exceptional: stars, geometrics, fleurons, harbour; benched:
lunar (again), genjiko2/keizuko (redundant with genjikō), acorns and
snow — with a stated CURATION PRINCIPLE worth keeping: a trio should be
three drawings, not one drawing rotated three ways. Tally benched as
culturally alien to Western eyes despite its counting rationale.
ASTERISM promoted on the user's type-history argument (awl celebrates
fonts; ⁂ is print's own centuries-old section-break mark — the one set
whose job description matches the slot). New proposal pairs worlds with
sets by STORY, near-unique (only arabesque worn twice): Gumtree
riverbank, Potoroo undergrowth, Bilby hanami, Saltpan+Bombora arabesque,
Quokka tavern, Mulga genjikō, Tawny autumn, Mopoke moonfaces, Bowerbird
wish, Currawong gambit, Mangrove harbour, Galah cardtable, Magpie
asterism, Brolga dovecote, Wagtail songbook (a songbird's world),
Firetail stars, Paperbark fleurons, Kite solar, Cassowary splatter.
Awaiting the user's confirmation or edits via the page's copy-out.

**FITTING ROOM v7 — the typographic-heritage shelf (2026-09-01).** User:
"we should celebrate typography where we can" — four heritage sets added,
all pipeline-rendered and verified (12 cells × 3 tiers, 0 blank/foreign):
REFERENCE MARKS (†/‡/※ — the Western footnote ladder plus Japan's
komejirushi, honouring the font's own tradition), RUBRICATION (¶/⁋/§ —
the pilcrow was in Nishiki's cmap though outside the census's rendered
set), SCRIPTORIUM (coronis + two paragraphoi — shelved with an honest
blurb: the most ancient section-dividers are mostly plain strokes and
fail the user's three-drawings rule), CURIOSITIES (‽/⁊/dotted obelos).
Separately the user flagged HARBOUR as internally incoherent (anchor and
sailboat are illustrations, the helm a diagram — three registers) — 
benched; Mangrove re-proposed to SPIRALS (tidal eddies, a loved set).
38 sets on the shelf. Arrangement still pending the user's copy-out.

**v8:** Scriptorium's twin plain-line paragraphoi (user: "too similar")
replaced by Alexandria's actual margin kit — coronis U+2E0E, dotted
diple U+2E16 (Aristarchus's mark), downwards ancora U+2E14 — three
distinct drawings, rendered and verified. The daggers conversation also
produced item 537 (footnote reference ladder).

**v9:** user chose to WEAR one heritage set — Saltpan takes SCRIPTORIUM
(the parchment world in Alexandria's own margin marks), which also
retires the proposal's last doubled set: all twenty worlds now dress
uniquely. Curiosities stays shelved (Currawong noted as its natural
wearer if ever wanted). Arrangement pending the user's copy-out.

**v10:** Spirals' mirrored left/right pair (user's rotation objection
again — correctly) rebuilt as three IDEAS of a spiral: round U+FF041,
angular U+FF053 (square coil), conical U+FF052. Candidates auditioned
and rejected: loose spiral (too close to round), cyclone (too busy),
curly loops (read as owl eyes). Mangrove wears the corrected set.

**FINAL — THE DECIDED ARRANGEMENT (user approved, 2026-09-01).** Trio
order is (dash `---`, star `***`, underscore `___`). Composed runs are
'+'-joined codepoints shaped as ONE text run — so `Ornaments` fields
MUST widen from `char` to a string; that model change is now required,
not optional. All glyphs are Nishiki-teki, requested at weight 500.

| world | set | dash / star / underscore |
|---|---|---|
| Gumtree | Riverbank | F591+F592+F592+F593 (long snake) / 1F41F fish / 1F40C snail |
| Potoroo | Undergrowth | 1F344 mushroom / 1F340 four-leaf clover / 2618 shamrock |
| Bilby | Hanami | 1F338 cherry blossom / 1F33C blossom / 1F337 tulip |
| Saltpan | Scriptorium | 2E0E coronis / 2E16 dotted diple / 2E14 downwards ancora |
| Quokka | Tavern | F5B0 acorns / F5B3 bells / F5B1 leaves |
| Bombora | Arabesque | F814+F815 white pair / F827+F828 black pair / F81C white scroll |
| Mulga | Genjikō | F501 (1 2 3 4 5) / F500 / F51B |
| Tawny | Autumn | 1F341 maple / 1F343 fluttering / 1F342 fallen |
| Mopoke | Moonfaces | 1F31D full / 1F31B first-quarter / 1F31A new |
| Bowerbird | Wish | 1F320 shooting star / 1F31F glowing star / 2728 sparkles |
| Currawong | Gambit | 2658 knight / 2655 queen / 2659 pawn |
| Mangrove | Spirals | FF041 round / FF053 angular / FF052 conical |
| Galah | Cardtable | 2660 spade / 2665 heart / 2663 club |
| Magpie | Asterism | 2042 asterism / 2051 two vertical / 2731 heavy asterisk |
| Brolga | Dovecote | FEFE1 dove right / FEFE3 olive-branch dove / FEFE2 dove left |
| Wagtail | Songbook | 2669 quarter / 266A eighth / 266C beamed sixteenths |
| Firetail | Stars | 2726 black four-point / 2736 six-point / 2727 white four-point |
| Paperbark | Fleurons | 2767 / 2619 / 2766 |
| Kite | Solar | 2600 black sun / 263C white sun / 1F31E sun with face |
| Cassowary | Splatter | FF04E splash / FF04F black splash / FF04D centerless splash |

Adopted glyph union for the derived subset face: 64 codepoints (the
table's singles plus composed components F591/F592/F593, F814/F815,
F827/F828) — enrol every one in the glyph-presence law. Build notes for
the lane: (a) `ornament_scale` tiers stay per-world as today; (b) each
world's `ornament_face` moves to the Nishiki register — a FOURTH
`OrnamentRegister` variant, which by design breaks `fold_mark_for`'s
exhaustive match until the lane consciously picks the Nishiki fold
mark; (c) LIST-BULLET pairs were NOT covered by this pass — a small
follow-up taste round derives each world's bullet pair from its worn
set; (d) the 18 unworn shelf sets and the benched sets land in the
recorded reserve roster with the reasons already noted above; (e) the
fitting-room artifact is the visual reference for all of this.

---
### 537 — footnote markers may wear the traditional reference ladder (user decision, 2026-09-01; sequenced AFTER 529 bundles the face)

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
