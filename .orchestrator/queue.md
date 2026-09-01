# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 529 — Nishiki-teki: audition a Japanese symbol cabinet, then give each adopted mark one honest purpose (user decision, 2026-08-29)

✅ COMPLETE — merged as `d8bb4b81`; ambient menu-bar restoration repaired in `1680c7f4`. Independent production audit accepted the derived-face roster and visual outcome; exact-main receipt: health pass, both conventions, forced menu-bar arm, 4,682 unit tests, 16 integration targets; web 16/16.

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

✅ COMPLETE — landed on main as merge `2d7b8090`; independent production
audit accepted with no findings. TableDims now takes footprint frost through
its own honest small insertion-card enrolment while the crisp/live-preview law
stays unchanged. Exact-main receipt: health pass, both conventions, forced
menu-bar arm, 4,686 unit tests, 16 integration targets; web 16/16.

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

✅ COMPLETE — landed on main as merge `c61b73a4`; production audit accepted
with no findings. The roster-derived Bars plate envelope now bounds upright
footprint frost for picker and pointer-menu geometries without changing hit
geometry. Exact combined-candidate gates are green: code health, 16 wasm smoke
tests, and full native receipt
`commit=c61b73a45b9d6e63d0acb1ffc19a29bd6d756819 health=pass:242s conventions=mac,linux scope=all-targets menubar=full:on unit_tests=4674 unit_shards=6 integration_targets=16`.
Subjective live softness remains a human taste check, not an engineering
blocker.

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

✅ COMPLETE — landed on main as merge `16508bf0`; independent production
audit accepted after deletion and width mutations proved the law non-vacuous.
All smart-punctuation substitutes now shape at their own advance, full content
size, and prose colour. Exact-main receipt: health pass, both conventions,
forced menu-bar arm, 4,690 unit tests, 16 integration targets; web 16/16.

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

✅ COMPLETE — landed on main as merge `ce74b6de`; independent production
audit accepted with no findings. Cassowary's active facet is now an outlined
docked tab whose open mouth joins the console card, while inactive labels
recede. The merge-train conflict was resolved without raising structural or
Clippy ceilings. Exact-main receipt: health pass, both conventions, forced
menu-bar arm, 4,690 unit tests, 16 integration targets; web 16/16.

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

✅ COMPLETE — landed on main as merge `ccc541a3`; independent production
audit accepted after five host-GPU vision-smoke captures and a conclusive
wrong-codepoint mutation of the derived-union law. All 20 worlds now wear
their approved Nishiki ornament trios at weight 500; composed runs, exact
64-codepoint enrolment, retained bullet pairs, fold choice, and the 18-set
reserve roster are law-pinned. Exact-main receipt: health pass, both
conventions, forced menu-bar arm, 4,693 unit tests, 16 integration targets;
web 16/16.

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
### 547 — CI RED: smart-punctuation full-body pixel law rejects Tawny ellipsis on Linux lavapipe

🟢 COMPLETE — second repair merged as `c403e4bc`; exact-main native receipt
`6501c7f4`, web smoke, and independent `gpt-5.6-terra` medium audit are green.
Hosted arbiter https://github.com/Frank-P-Lu/awl-editor/actions/runs/33461126973
then passed all four gating jobs, including Linux/lavapipe in both conventions.

The real defect was document-face shaping, not a backend tolerance: Tawny's
IBM Plex Mono ships at weight 300, and the old default-400 smart-punctuation
path fell through to a proportional face. `84152118` routes both reserved
advance and painted glyph through one document-family/weight/features owner.

The first repair landed on main as merge `1f07416f`; an independent
`gpt-5.6-terra` medium audit accepted its enrolment and mutations, and exact-main
native plus web gates were green locally. Hosted run
https://github.com/Frank-P-Lu/awl-editor/actions/runs/33453602151 then exposed a
different Tawny × EmDash result in both conventions: ornament ink is 20 px wide
versus a 14 px Unicode control (ratio 1.429), and the identical suffix begins 9
px farther right. Treat this new report as a product-defect hypothesis: determine
whether the ornament face is genuinely over-advancing or the control comparison
still collapses unlike shaping paths. Do not widen either absolute tolerance or
ratio band. Repair the product if the suffix displacement is real; otherwise
replace the faulty comparison with a same-frame outcome that still mutation-proves
deletion, undersizing, wrong ink and over-advance. Hosted lavapipe must turn green.

Remote arbiter run
https://github.com/Frank-P-Lu/awl-editor/actions/runs/33443457917 is RED.
First observed bad main is `8545d9b0`; the failing law entered through item
545's merge `16508bf0`. Linux's full native suite fails in both keymap
conventions at exactly
`render::tests::smart_punct::pixels::smart_punct_ornament_is_full_body_glyph_in_content_ink_every_world`.
The only failing cell is Tawny × Ellipsis: source and rendered Unicode control
have the same 10 px width and the same body ink, but the source's body-colour
bounding box is 3 px tall while the control's is 1 px tall, exceeding a 1 px
height tolerance. Web and both gating macOS lanes are green; the tolerated
hosted-Metal render lane and AT-SPI lane remain separate known axes.

Treat the defect report as a hypothesis. First determine whether lavapipe has
exposed a real product geometry/ink defect or whether a one-raster-row Unicode
control makes bounding-box height an invalid backend oracle. Do not repair this
by tuning an absolute backend tolerance. If the product is sound, replace only
the false measurement with a same-frame relative outcome that still proves
full-size punctuation, body ink and non-vacuous presence over every world ×
`SmartPunctKind`; mutation-prove deletion, undersizing/width loss and wrong-ink
failures one at a time. Preserve the exact reported failure as diagnostic
evidence. Verify targeted laws in both keymap conventions, then the full native
gate and web suite; a production-tier independent audit reviews the premise,
enrolment and mutations. Remote Linux is the final lavapipe oracle.

---
### 548 — table text wears the PREVIOUS world's ink after a live theme switch (user report, 2026-09-01 — "tables are off on this theme"; ink identified by pixel arithmetic, mechanism still a hypothesis)

🟢 COMPLETE — `6a1d23c9` moved ordinary cells to live default ink and refreshes
inline cell attrs with the table cache; `fbcef6ac` aligned the inherited-ink law.
Independent `gpt-5.6-terra` medium audits accepted the exact candidate after
both mutations, the 20-world census, and five-shot vision smoke; native and web
gates are green on `fbcef6ac`.

Report: on a live Mangrove window the user's tables render near-invisible
while the surrounding prose is correct. Measured over the report's screenshot:
prose ink ≈ (219,233,228) — Mangrove `base_content` #D9E6E1 — but table cell
text paints at ≈ (16,18,22), which is **Magpie's** `base_content` #111317
within compression noise: a light world's near-black ink, DARKER than
Mangrove's page ground (17,38,35), roughly 1.1:1. Table glyphs are wearing
another world's content ink while prose wears the right one. This is not a
Mangrove palette defect: static headless captures of plain and rich GFM
tables in Mangrove at `--capture-dpi` 1 and 2 paint cell text in full
content ink.

That static cleanliness is expected, and docs/harness-reach.md names the
class exactly: a capture witnesses the state a pipeline was BUILT with,
never the state it was later RE-SEEDED with — the live `sync_theme_colors`
path from `app/apply.rs` is structurally unreachable by ordinary capture,
and a defect there "reaches only a user who changes worlds while the app is
running." Working hypothesis, to be verified before repair: the table-grid
text path (`prepare_table_grid` / `render/layers/table_grid.rs`, or
whatever actually owns table glyph tinting) is missed by the live re-tint,
or its cached rows are not invalidated on world change, so table text keeps
the prior (or a previewed) world's ink until some later reshape. The ink
identification stands regardless of which mechanism it turns out to be.

Grounding also settled most of a second visual in the report: the solid
block cells match the deliberate EMPTY-CELL plate rendering
(`table_empty_pipeline`, `theme::muted()` at alpha 30), reproduced in a
fully static capture over cells with no content — likely not part of this
defect, though whether the user's cells were in fact empty is unconfirmed,
and the measured block colour sits a little above both a fresh-Mangrove and
a stale-Magpie blend estimate. Audit their re-seed too:
`render/pipeline_geometry.rs` seeds that colour, and the lane determines
whether it runs on a live switch or only at construction — likewise every
other table-owned colour (x-ray, header rule, borders).

Repro and laws: `--screenshot-app` reaches this transition (theme accept is
tier 1, `overlay_accept:Theme` = Applied): start `--theme Magpie` on a
table document, drive the real theme picker to accept Mangrove, then assert
by PNG arithmetic that table cell ink equals the NEW world's content ink —
a contrast floor against the page ground paired with a presence floor, so
the law cannot be satisfied by deleting its subject. Add the
pipeline-reading law harness-reach.md prescribes for this axis: after a
live `sync_theme_colors`, table pipeline colours match the new world,
swept across the roster rather than one hand-picked pair, with a
non-vacuity guard that the values being distinguished actually differ
somewhere in the roster. Mutation-prove by restoring construction-only
seeding on the table path and watching both laws go red. Standing policy —
a user-reported bug gets its neighborhood audited: census every pipeline
colour seeded at construction for the same live-switch miss; tables are
unlikely to be the only tenant.

---
### 549 — stale empty-cell plates survive a buffer swap: the no-tables early return never clears `table_empty_pipeline` (user report, 2026-09-01 — "switching tabs leaves the table in place?"; mechanism identified by reading, exact residue shape matches)

Report: after switching from a buffer containing a table with empty cells
to a different buffer with no tables, the empty-cell plates keep drawing at
their old screen positions over the new document's text. The residue is
plates ONLY — no stale cell text, no stale header rule — and that shape is
exactly what the code predicts: `prepare_table_grid`'s no-tables early
return (`render/layers/table_grid.rs`, the `blocks.is_empty()` arm)
re-prepares `table_rule_pipeline` and the glyphon table renderer with empty
slices but returns WITHOUT touching `table_empty_pipeline`, whose previous
instances keep drawing every frame (`render/pipeline_layers.rs` draws it
unconditionally). Treat that as a strong hypothesis with the usual
verification duty, but the asymmetry is right there in the arm.

Replay note, so no lane burns a round the way this grounding nearly did: a
`--screenshot-app` journey (table buffer → New document via palette, also
a Last-file round trip) captures CLEAN, structurally — per
`main/run/live_app.rs`'s own module doc the App renders nothing during
chord replay; one final frame goes through the harness's own offscreen
pipeline (`capture::capture_with`), so the table buffer's plates are never
written into any pipeline for the swap to leave behind. This is the harness-reach lesson
("a capture witnesses the state a pipeline was BUILT with") wearing a new
face: the residue needs a frame HISTORY. The purest seam is a unit law on
one `TextPipeline`: prepare a table-bearing document (assert plate
instances present — the presence half), then re-prepare with a plain
document and assert the empty-plate instance count is zero; mutation-prove
by deleting the fix. Sweep the OTHER doors into the same early return —
the arm also fires when WYSIWYG or markdown is toggled OFF, so a toggle
over a table document must shed its plates the same way — and audit the
table-owned pipelines as one roster (rules+pan bar, x-ray, empty plates,
cell text) so every member is cleared on every door, not just the two the
early return already covers. Item 548's neighborhood audit (pipeline
colours seeded at construction) and this item (pipeline instances not
cleared on empty prepare) are two tenants of the same street: table
layers sitting outside an invalidation path the rest of the render walks.
A Goto-corpus aside from the grounding, for whoever owns the harness next:
a headless `--screenshot-app` with `--root` seeded showed only the open
file in the Goto Files lens (`b.md` sibling never listed, "no matches" on
query `b`), which smells like the file index racing the single-frame
capture — worth a look before anyone writes a Verify clause that drives a
swap through Goto.

---
### 550 — working-set close lane: the active plate overhangs its label by one character too many (user taste report, 2026-09-01 — "the highlight bar is SLIGHTLY too long, on the left side… i think the x button should fit when you hover over it, but it's just a tad too long"; geometry measured, mechanism identified)

Report: the active row's plate in the working-set stack extends left of the
label's first glyph by a gap the user reads (correctly) as the reserve for
the hover-revealed close mark — and finds a touch too long. Measured on the
user's 2× screenshot in the label's own character units (char width taken
from the sibling row's 17-char name, label width cross-checks to within
1px): the left overhang is **3.17 chars**; the right overhang is under one.
That 3 + pad is exactly what the code draws: `render/chrome/gutter_stack.rs`
shapes `CLOSE_MARK_TEXT = "×  "` — × plus TWO spaces — as the always-present
leading run on every row, `plate_rects` measures the plate's ink as label +
those 3 chars, and `plate_rect` adds `pad_x`. On hover the × occupies the
first char and one space separates it from the label; the second space is
the surplus the user is seeing.

Direction (the taste verdict is already given): trim the lane by one space
so the revealed × just fits — likely `CLOSE_MARK_TEXT = "× "` — keeping the
file's own invariants intact. Two couplings the lane must hold: (a) the
single-file identity line (`super::gutter`) shapes the SAME lane through the
same const — both surfaces move together or neither; (b) `close_zone` is an
h×h square anchored at the ink's leading edge, and the file's own tripwire
("a revealed × must never draw routed ink outside its own fill") wants a law,
not a spot check: assert × ink ⊆ plate fill AND click zone ⊆ plate fill,
swept across the per-world mono FACE roster rather than one face — the
zone's width is the row height while the lane's is char-width-times-two, so
a narrow-aspect mono is where a 2-char lane would first lose the square
(the axis the measured screenshot, one face at one size, cannot cover).
Mutation-prove by shrinking the lane to "×" alone and watching the
containment law go red. Plate/zone geometry is pure functions — unit seam;
the hover reveal itself is pointer state no `--keys` capture drives
(docs/harness-reach.md before promising otherwise), so the resting plate
width is the capturable half and the hover feel is the user's live check.

---
### 551 — selecting across a table paints margin slivers, not a band: the selection wash collapses against the concealed source (user report, 2026-09-01 — "the table doesn't really… select properly?"; reproduced headlessly, first try)

Report: extend a selection through a GFM table and the selected rows show
no selection band — just a thin vertical sliver at each row's left margin —
while the rows themselves reveal their raw aligned source, drawn with no
wash behind it. Reproduced deterministically at dpi 1, Firetail, over a
prose/table/prose file: `--keys "Down Down S-Down S-Down S-Down"` lands
`selection` at line 3 col 2 → line 5 col 2 in the sidecar, and the PNG
shows per-row band slivers 5–8px wide hugging the left margin while the
revealed row ink spans ~380px beside them. That is the user's screenshot
exactly (their formatting popover was up, so the app agreed a selection
existed — only the paint is missing).

The mechanism is half-named in the tree already: `render/rects.rs` (the
wash builder, "A GFM table renders as a drawn GRID…") documents that a span
inside the concealed-to-zero-width table source "would collapse to a thin,
full-row-height sliver at the left margin" and CARVES OUT inline-code and
highlight washes for exactly that reason. The selection band looks like the
member of that family that never got a treatment — it still measures the
concealed advances, while the selection-touched rows are simultaneously
REVEALED and drawn from the table path's aligned layout, so the band's
geometry and the visible ink disagree. Note the carve-out precedent is
skip-the-wash; a selection cannot be skipped — it has to be REBUILT against
the revealed run's real advances (or the grid's cell geometry, if the lane
and user land on cell-wise selection paint), so this is a design choice to
put to the user with a capture, not silently. Sweep the axis: endpoints
inside cells, spans covering header and divider rows, partial first/last
lines, WYSIWYG toggled off (raw mode should already band normally — assert
it as the control cell), and the wrapped tall-row case the wash comment
says made the sliver visible. Law with the presence floor: per selected
table row, band width ≥ the selected revealed ink's width, never the
sliver; prove non-vacuity by reverting. Fully headless — the repro line
above is the Verify seed; run one second world alongside Firetail so the
enrolment isn't a single palette's property. Same street as 548/549/550:
table surfaces sitting outside an invariant the rest of the renderer walks
(there, invalidation; here, the reveal/wash contract).

---
### 552 — `~~` fuses into one wide tilde in prose: Monaspace ships its tilde ligatures behind `rlig`, which awl never disables (user report, 2026-09-01 — "why does ~~ turn into a big tilda? doesn't this break our rule?"; yes — mechanism verified in the font's own GSUB)

Report: typing `~~` in a Monaspace world renders ONE wide swung tilde
where two characters sit in the file (reproduced headlessly, Firetail,
`alpha ~~ beta` and an unpaired `~~word`; a single `~` stays small). The
user is right that this breaks the stated rule: `render/text.rs
font_features` promises prose "standard + contextual ligatures ON
(fi/fl)…`calt` OFF unconditionally", and its own doc claims Monaspace is
"ligature-free either way". Both clauses miss the door the font actually
uses: read from `assets/fonts/MonaspaceXenon-Regular.ttf`'s GSUB, the
tilde family (`~~`, `<~`, `~>`, `!~`, `-~`, `=~`, `<~>`, `~~>`, `<~~`) is
registered under `dlig` (disabled, good) AND reachable again through a
chain-context lookup owned by **`rlig` — Required Ligatures** — a feature
shapers apply unconditionally (it exists for scripts whose ligation is
mandatory) and that no feature set in the tree touches (`rlig` appears
nowhere in src). So the prose set's fi/fl-only contract is satisfied at
`calt`/`dlig` and lost at `rlig`.

Scope is wider than prose: Monaspace Xenon is the DISPLAY face of Potoroo
and Firetail (whole documents) and the MONO of many worlds (inline code
and code blocks), and the CODE arm for unsafe monos disables
`calt`/`rclt`/`ccmp` but not `rlig` — so the fusion fires in code too,
where two source chars sharing one glyph is exactly the non-uniform
`line_glyph_xs` break the pitch probe exists to prevent. The lane should
(a) enumerate what else rides that `rlig` chain in both Monaspace weights
(the GSUB walk that found this filtered to tilde-composed ligatures only),
(b) decide the disable's scope — per-face rather than blanket, since
`rlig` is genuinely required for e.g. Arabic through fallback faces and a
global off is a different bug, and (c) re-derive the "ligature-free either
way" claim from the font rather than re-asserting it. Laws: prose `~~`
shapes as two glyphs with two advances on every Monaspace world (sweep the
roster, not a named world); the code-buffer caret grid law extended by a
`~~`-bearing line; strikethrough's own raw reveal (`~~struck~~` on the
caret line) shows four distinct tildes. Mutation-prove by re-enabling.
Headlessly verifiable end to end — the repro capture above is the seed.

---
### 553 — search across the folder: full-text search as a summoned surface (user decision, 2026-09-01)

DECIDED from the feature-gap review. PHILOSOPHY §1 promises "the simple
file operations, navigation, search, and version history needed to
sustain writing," and search currently stops at the buffer: ⌘F/⌘R are
in-document, and Go to… matches names/headings/recents, never content —
"where did I write about X" has no answer inside awl. Build full-text
search over the active folder: type a query, see matching lines grouped
by file with the match highlighted, Enter opens that file at the match
through the same door any open uses. Constraints: a summoned surface on
the existing picker conventions (`render/rowlayout`, docs/render.md),
keyboard-first, palette entry "Search in folder…"; no default chord is
decided here — ⌘⇧F is `search_backward`, so any binding is a separate
taste call and palette-only is acceptable to ship. Reuse `src/index.rs`'s
folder enumeration (same gitignore conventions); scanning is O(folder)
and stays off the frame path — work happens on summon/keystroke against
a bounded budget, never per-frame. Zero network. Native first; web where
the platform permits. Verify: unit tests over the matcher/grouping seam
(case folding and Unicode decisions recorded in the tests), and a
`--keys` sidecar journey — summon, type, Enter, sidecar shows the landed
file and caret; read docs/harness-reach.md before promising more. NOT
this item: regex UI, saved searches, replace-across-files.

---
### 554 — drag-and-drop onto the window: a text file opens, an image lands in the document (user decision, 2026-09-01)

winit's `DroppedFile` event is unhandled today — dropping anything on
the window does nothing. DECIDED semantics, in the user's words: a
markdown/text file is the "same thing as open file, with the file" —
route the dropped path through the exact door `Open file…` uses, so
working-set, recents, and session semantics come free (same behavior ⇒
same code, one owner). A dropped image "should put the image in the
file": reuse the paste-image pipeline — the image is copied into
`assets/` beside the document and a reference inserted at the caret as
one undoable edit (docs/markdown.md). The classify-and-route step is
the only new code; the image arm calls the same owner paste-image
uses, never a second implementation. Sub-decisions the lane settles
with recorded defaults: multiple files dropped (working hypothesis:
text files open into the working set in drop order; images insert
sequentially); an image dropped on the scratch/no-path buffer follows
whatever rule paste-image already has there. Native-only where
paste-image is native-only; web drag-drop is out of scope. Verify: the
OS gesture never flows through `--keys` — unit-test the
classification/routing seam directly (path in, decision out), and flag
the physical drop itself for live human confirmation per
docs/harness-reach.md.

---
### 555 — sentence motion: forward/back, select, and delete by sentence (user decision, 2026-09-01)

awl is prose-first but its motion grammar stops at words — no sentence
verb exists anywhere in the tree. Add sentence forward/backward motion,
shift-extension (enrolled in `is_motion` so selection extension arrives
by the existing rule, and the documented non-movers test is updated
deliberately), and delete to sentence end/start. The boundary rule is
the product here — editing edge-cases get generous spend: prefer UAX #29
sentence segmentation (the `unicode-segmentation` crate already in the
grapheme path carries it) over a hand-rolled terminator heuristic, since
abbreviations ("e.g.", "Dr.") are exactly the axis a hand rule misses;
whatever rule ships, record it and sweep it in tests. Bindings: the
Emacs slots take the classic M-a / M-e / M-k — noting the Meta layer
seeds Linux-only under the `emacs` flavor and is inert on Mac
(docs/config.md); there is no macOS-native convention, so the native
slots may ship empty with palette entries ("Sentence forward" etc.),
one `[keys]` line away for anyone who wants chords. Verify: exhaustive
unit tests at the motion seam — terminators, closing quotes and parens
after the period, ellipsis including the smart-punct `...` run, CJK 。,
buffer ends, a sentence spanning soft wraps — plus a `--keys` sidecar
journey.

---
### 556 — move line/selection up and down (user decision, 2026-09-01)

No such action exists. The prose meaning is "reorder list items and
paragraphs": one command swaps the caret's logical line — or every line
a selection touches, moved as one block — with its neighbor, caret and
selection riding the moved text, the whole move one sealed undo group
(mutation-prove: two moves undo as two steps, a block move as one).
Edges to sweep: first/last line (no-op, recorded), the final line
without a trailing `\n`, a block move against buffer ends, sticky goal
column unaffected, and wrapped text — this is a LOGICAL-line move,
visual rows never reorder independently. Inside a table a line swap is
a source edit like any other; row-leave re-pad (the landed 542
mechanism) owns re-alignment — add one test proving a moved table row
re-pads rather than corrupting the grid. Numbered lists: the existing
renumbering rule owns fixing ordinals; prove it fires by test rather
than re-implementing. Bindings: ⌥↑ / ⌥↓ are free in the defaults
(checked 2026-09-01) and are the cross-editor convention; macOS text
fields use them for paragraph motion, which awl does not implement, so
taking them is a deliberate taste call — land it and name the revert
cost per the standing policy. The Emacs slot has no exact classic
(transpose-lines differs); seed the same chords or leave empty.
Verify: unit tests at the edit seam plus a `--keys` sidecar journey.

---
### 553 — single open file draws no active plate in the working-set gutter (user report, 2026-09-01 — "when you first open a file, it doesn't seem to be selected?"; behavior confirmed in every capture's own gutter)

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
### 554 — close mark wants real hover feedback, and the row wants a pointer cursor decision (user report, 2026-09-01 — "when your mouse is over the X button it should be highlighted or something… the whole thing is a pointer right… how do tabs do this?")

Today's affordance ladder (`render/chrome/gutter_stack.rs`, the reveal
logic): the × is invisible until the pointer is over its ROW, then drawn
`muted`, and brightens to the selected-row secondary ink only when the
pointer is inside the close ZONE itself. The user's report says that
ladder's top rung is too quiet — asks for a visible highlight on the ×
under the pointer (a plate/ring behind the glyph, not just an ink shift),
and raises the CURSOR question: the whole row is clickable (switch) plus
the destructive close zone, and nothing changes the pointer shape
(`cursor_shape.rs` is the owner to check — what does the gutter request
today?). Their own answer to "how do tabs do this": browser/editor tabs
show a hover plate on the × and do NOT use the hand cursor — arrow plus
hover-state is the convention. Lane: audition a close-zone hover plate
(zone rect already exists — `close_zone` — so the plate is the same rect
the hit-test owns, keeping drawn-vs-accepted from drifting), keep the
arrow cursor unless the user asks otherwise, and bring a capture pair to
the user. Hover is pointer state no `--keys` capture drives; the plate
geometry laws sit at the unit seam (zone ⊆ plate fill, 550's family), and
the hover look itself is the user's live check. Coordinate with 550 (the
lane trims the same geometry) — one lane may take both.

---
### 555 — theme picker rhythm: dead rows between the query head and the first row, and a top-heavy oversized hint card (user taste report, 2026-09-01 — "from where the caret is until the first item there's a lot of spacing… kind of weird; the bottom instruction box has too much padding vertically")

Measured on the user's 2× screenshot (unclamped list, all 20 worlds
fit): the query head's caret dot ends at y≈106 and the first row's ink
starts at y≈240 — ≈135px of nothing, about TWO full row pitches (row
pitch ≈62px). The hint card at bottom is ≈107px inside its borders for
one ≈24px text line — 51px pad above the text, 30px below: oversized and
top-heavy, and the asymmetry reads as a mistake rather than a choice. In
the 1200×800 captures the same gap holds the "↑ N more" clamp marker, so
a starting hypothesis for the lane: the head gap is a reserved marker row
(plus its leading) that stays reserved even when nothing is clamped, and
the fix is collapsing it in the unclamped state — but derive the actual
owner from `render/rowlayout` / the overlay geometry rather than trusting
this reading. Direction offered to the user (they asked for a better
idea): head-to-list gap of one row leading when unclamped, marker row
materializing only when it has something to say; hint text vertically
centered in a card one row tall plus symmetric padding, matching the
picker rows' own rhythm. Geometry is fully headless-verifiable (this
item's numbers came from pixel arithmetic); the final feel is the user's
live sign-off. Blur/frost complaints from the same session are NOT this
item: the bounding-box-over-L-shape frost and the squiggle-over-frost
layering question belong to the 543/544 street — check with the user
before opening that follow-up.

---
### 556 — world ornaments render at inconsistent sizes across worlds; scale the small sets up to the big ones (user decision, 2026-09-01 — "for some worlds it's a bit small… bigger is actually better here; make sure the smaller ones are scaled up to match the bigger ones")

Two user screenshots of the SAME document in two worlds: a chess-piece
ornament set (dark world) drawing noticeably smaller, relative to the
body text beside it, than a bar-glyph set (dark-green world) — rough
normalization against each shot's own char width puts the chess set at
~2.6–2.9 char widths tall against the bars' ~3.6 (approximate: measured
from screenshots at unknown zooms, normalized by their own text; the lane
re-measures headlessly). The user's direction is explicit and is the
item: EQUALIZE UPWARD — the bigger rendering is the target, the smaller
sets scale up to match it. Owner: the per-world ornament cabinet (529/536
neighborhood). Law shape: capture one fixed document across the FULL
world roster, measure each world's ornament ink height normalized to that
world's own body char width by pixel arithmetic, and pin the spread —
every world within a tolerance band of the roster target, derived from
the roster rather than a named pair, naming the offender in the failure
message. Watch the axis the glyphs hide: different sets have different
intrinsic ink-to-em ratios (a chess knight fills its box differently than
solid bars), so the law measures INK EXTENT, not font size requested.
Final size is the user's live sign-off.

---
### 557 — Insert Table dims grid: hover should live-resize the selection, and the pointer question again (user decision, 2026-09-01 — "it's not really mouse friendly… as you hover over it it should resize along to where your cursor is… cursor should be pointer I think"; animation explicitly deferred: "maybe a bit too much for now")

Verified in the tree: the dims grid answers CLICKS only —
`app/input/mouse.rs` maps a press through `table_dims_cell_at` to
`table_dims_pick`, and no pointer-Moved path touches the dims selection;
hovering the grid changes nothing until you click. The ask: pointer over
cell (r, c) previews r×c live — the standard Notion/Word grid gesture —
with the keyboard path (arrows already work) staying authoritative and
the two never fighting (hover updates the same one selection state the
arrows move; no second shadow state). Also set the pointer cursor over
the grid (`cursor_shape.rs` is the owner; note 554 records the user's
own observation that tabs DON'T hand-cursor — the two items should land
one consistent cursor policy for clickable chrome, so coordinate).
Animation is DEFERRED by the user's own words — do not add it. Verify:
hover is pointer state outside `--keys` reach; the hit-mapping
(`table_dims_cell_at`) and the selection-update seam are unit-law
territory (hover at cell ⇒ selection equals cell, swept over the grid's
corners and the card's padding edges where off-by-one lives); the live
feel is the user's check. The card itself is 543/544 frost territory —
this item does not touch the frost.

---
### 537 — footnote markers may wear the traditional reference ladder (user decision, 2026-09-01; sequenced AFTER 529 bundles the face)

🔴 BLOCKED — needs the user's product decisions on per-document versus recycled scope and whether definition-list markers follow the display option. U+2016 coverage remains an engineering verification, not a user decision.

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
