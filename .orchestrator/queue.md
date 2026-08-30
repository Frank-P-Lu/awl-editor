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
before final. **⌫ U+232B goes to Nishiki too (user, follow-up)** — so
ALL 34 marks resolve to Nishiki and the derived face is a pure
single-source subset with zero welded-in exceptions. **❥ U+2765, ✴ U+2734,
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
of roughly ≥250 chars can never be saved under its derived name.
PREMISE CONFIRMED BY THE USER (2026-08-30): "yeah, it was a long
paragraph" — the document's true first non-empty line, above the
screenshot's viewport, was a prose paragraph; the fix direction is
ratified ("your fix makes sense"). The lane still reproduces with a
real tempdir on the real disk before fixing (`InMemoryFs` enforces no
NAME_MAX, so the in-memory seam CANNOT witness this failure).

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

RATIFIED + SHARPENED (user, 2026-08-30): "having two different
locations … is just overkill and kind of bizarre — we need to clean
that up in either case." The end state is ONE notice location per
world. To be precise about the present shape (the user read it as two
locations authored in the theme): themes author exactly ONE anchor
today (`toast_anchor`); the sticky's top-center was a hardcoded global
composition rule outside theme data. This item deletes that second
rule, so afterwards one authored anchor governs every notice.

The user also raised: should placement even vary per world, or be one
centralized location/setting? RECOMMENDATION RECORDED, awaiting the
user's word (default: keep per-world, no setting): per-world placement
is the product's existing grammar — worlds already relocate the
placard, the card, and the facet strip as authored composition
(Kite's TopRight deliberately mirrors Firetail's TopLeft), and a toast
is glanceable/transient, where within-world coherence matters more
than cross-world muscle memory. A `[ui]` config override is machinery
awl doesn't need until a real complaint arrives; if one does, it is a
small additive follow-up, not a redesign. Should the user instead
choose one global location, that is theme-data removal (`toast_anchor`
retires like the theme picker's lens strip did) — a different, bigger
item; do not start it on this brief.

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
### 537 — a failed scratch→note conversion destroys the scratch identity, silently drops the buffer on the next switch, and leaves a dead "scratch" row (user report, 2026-08-30; "sounds like an architectural code smell — we should fix this")

User symptom (screenshot): the working-set stack shows a "scratch" row
that does NOTHING when clicked, while the sibling file rows switch
fine. Every link of the causal chain is verified in code; the entry
premise (the failed ⌘S from item 534's screenshot happened in the TRUE
scratch, not a ⌘N note) is the one unverified link — the user has been
asked, and the lane re-confirms by reproduction either way.

THE CHAIN: (1) ⌘S in the true scratch routes to
`convert_scratch_and_save` (`app/files/verbs.rs`) →
`Buffer::save_into_folder` (`buffer/save.rs`), which calls
`set_note_dir` — flipping `is_unnamed_fresh()` true (it is literally
`note_dir.is_some()`, `buffer.rs`) — BEFORE attempting the fallible
save, with no rollback on `Err`. Item 534's uncapped filename made that
save fail, so the buffer was left mutated: scratch identity destroyed.
(2) `BufferKey::of` (`buffers.rs`) returns `None` for an unnamed-fresh
buffer, so `park_active` (`app/document.rs`) CANNOT park it — its else
arm keeps it active. (3) The next switch (`open_path`) then assigns
`self.active = Some(new entry)` over it — the buffer and every edit
since the last scratch stash flush are DROPPED, no gate, no notice.
(4) The working set's `Scratch` row (enrolled at launch) outlives the
entry; clicking it → `activate_open_buffer`'s no-path arm →
`activate_key` → `take_parked(Scratch)` → `None` → silent `false`.
Dead row. Recovery that still works: the "Open scratch" palette command
(`close_facts(Scratch)` is `None`, so it falls through to the stash
restore) — told to the user.

FIX, three layers, ordered:
- (a) HEAD OF CHAIN — naming becomes TRANSACTIONAL: `save_into_folder`
  derives the path and attempts the write FIRST, committing
  `note_dir`/path binding only on success (or rolls back `note_dir` on
  `Err`). A failed save must leave the buffer byte-identical in
  identity to before the ⌘S.
- (b) THE LAW, named: **an active buffer holding user text is never
  replaced except by park (reversible), a gated close, or an explicit
  refusal.** Today `park_active`'s unparkable arm + every
  `self.active = Some(…)` assignment after it violates this silently.
  Non-vacuity: break `park_active` and watch it go red.
- (c) DEAD-ROW FALLBACK: `activate_open_buffer`'s no-path arm
  (`app/files/open.rs`) gets a fallback for `BufferKey::Scratch` —
  fall through to `open_scratch`'s stash-restore door instead of
  returning silently. This also covers the LATENT second route into
  the same state: the registry's clean-LRU eviction (`buffers.rs
  park`) does not exempt `Scratch`, and an evicted pathed file
  re-reads from disk while an evicted scratch has no such fallback.

Item 538 proposes retiring the ghost class outright; if that direction
is taken it subsumes (b)/(c) — land (a) regardless. Repro needs NO
fault injection: a real tempdir plus a ≥300-char first line reproduces
the exact failure natively (InMemoryFs cannot — 534 records why).
Cross-ref 534: its cap removes THIS trigger, but any save error
(read-only dir, disk full) re-enters the chain, so 537 is not
subsumed by it. Full gate receipt.

---
### 538 — unnamed-fresh documents are ghosts: no key, no row, no park — give them first-class identity (user direction, 2026-08-30: "architectural code smell")

The architecture underneath 537's chain, and a second user-reported
symptom of the same gap (screenshot): **⌘N shows "• scratch" in the
title, but the working-set stack draws no row for the new document and
keeps HIGHLIGHTING the previous file as active** — the stack is lying
about which document the user is looking at. Verified mechanism:
`start_fresh_document` (`app/document.rs`) enrols no working-set row
(one-shot naming: no identity until named — a recorded decision,
`app/document/cache.rs` doc), and nothing clears `working.active`, so
`file_row`'s `active: self.active == Some(at)` still marks the old
row. A third confusion in the same family: an unnamed fresh NOTE's
display name is the string "scratch" (`app/files/mod.rs`), colliding
with the true scratch's identity in the title bar and stack.

DIRECTION to design and land: an unnamed-fresh document gets a
first-class `BufferKey` (e.g. a session-unique `Fresh(n)`), so it
parks, enrols a working-set row, switches, and closes like every other
buffer; the one-shot naming save then RE-KEYS the entry to its derived
path (the registry re-key precedent: `park` already replaces same-key
entries, and history has `rename`). This retires the ghost class:
537's silent-drop hole and dead-row arm become structurally
impossible, ⌘N's row appears immediately (labelled distinctly from
true scratch — "untitled"? the lane proposes, the user picks the
word), and the stale-active highlight cannot happen because
activation goes through the same `set_active` door as everything
else. Sweep the consumers that assume "path-less = scratch":
`BufferKey::of`, `close_facts`, the stash machinery
(`scratch_saved_version`), daemon gate (`app/daemon.rs`), session
restore, and the capture sidecar's buffers block
(`capture/buffers_sidecar.rs`). The working set already models
path-less rows (`OpenFile.path: Option`), so the row half is small;
the registry/naming half is the design work. Laws: every route that
creates a document — open, ⌘N, scratch restore, daemon — lands
exactly one working-set row whose active mark agrees with
`active_key`, swept across the route roster, no wildcard.
Full gate receipt.

---
### 539 — working-set stack: move the hover-revealed close mark to the LEADING side, so names sit flush against the page edge (user taste direction, 2026-08-30 — RECOMMENDED option A, awaiting the user's word)

Context: the hover-reveal-with-reserved-lane the user asked for already
ships (`stack_spans` shapes a trailing `"  ×"` on every row, alpha 0
until row-hover — names never move). The residual itch is the
reservation itself: the uniform trailing lane (`fit_rows`'
`label_budget = budget − 3`) parks every name ~3 chars short of the
stack's right edge, so the block never actually hugs the writing
column it right-aligns toward.

The user proposed two options; A is recommended:
- **A (recommended): the mark moves to the LEADING side.** Names
  right-align FLUSH to the stack's edge (reclaiming the lane), and the
  `×` shapes as a leading span at alpha 0 — in right-aligned layout a
  leading span grows the line LEFTWARD into the ragged edge's already
  empty space, so revealing it still changes ink only, and nothing
  ever moves. Trade named honestly: the mark no longer sits in one
  vertical column (its x follows each name's leading edge), so
  serially closing several rows means a small horizontal chase — at a
  resting stack of ≤4 rows this is negligible, and macOS tab close
  buttons sit leading-side, so the position reads native.
- **B (named, not recommended): keep the mark trailing but push it
  further right**, letting names right-align flush with the mark
  beyond them. Rejected because that space is the seam awl works to
  keep calm: the active-row plate ends one pad past the box and the
  frost halos hug the column the same way (`plate_rect`'s right-edge
  invariant) — a mark there crowds the page boundary and collides
  with the plate/frost conventions.

Mechanics for the lane: `fit_rows` stops docking the close lane from
the label budget but must still keep the LEADING mark from clipping at
the canvas edge on maximal-width rows (the mark may yield there — it
is hover ink, not identity). `stack_spans` moves the always-shaped
mark span from trailing to leading per row (alpha-flip mechanism
unchanged; More/Overflow rows keep shaping it un-revealable, same as
today). The close HIT ZONE flips from the right edge to the leading
edge, derived from the same row plan the draw uses
(`gutter_hit::stack_hit_from_plan`) so click and ink cannot disagree.
The single-file identity line rides the same door (`gutter.rs`'s
row-0 reveal) and gets the same flip. The active-row plate derives
from shaped `text_w`, which now includes the leading mark — decide
whether the plate should cover the mark region or only the label, and
law-test whichever is chosen. Laws: flush-right alignment (every
name's right edge equals the stack edge, swept across row counts and
name widths), reveal-changes-ink-only (geometry byte-identical
hover vs not), and the hit-zone/ink agreement law. Cheap to revert
(one commit, the trailing layout is `git log`'s to restore); per the
standing land-easy-taste policy this can land for judgement once the
user confirms A. Full gate receipt.

---
### 540 — Insert Table dimension picker: the hint clips mid-word ("Esc canc"), and the card's placement/backing needs a judged pass (user report, 2026-08-30; feature itself: "AWESOME!!")

User verdict on item 517's picker: "insert table is AWESOME!!" — the
feature holds; this is polish on its card. Two defects, one verified
and one to assess.

VERIFIED — THE HINT CLIPS: `table_dims_overlay_geometry`
(`render/chrome/table_dims.rs`) sizes the card to the GRID alone
(`desired_w = grid_w + 2·pad`, ~253 logical) and hands the hint
`text_w = card_w − 2·pad`, so "8 × 8 table  ↵ insert  Esc cancel"
runs out of column and clips mid-word to "Esc canc" — a raw clip, not
an elision. The main card already encodes the lesson this fourth
geometry arm missed: `measure_overlay_content_w` (`roster.rs`)
includes the card's CHROME LINES (query/lens/footer) in the content
measure precisely so no chrome line outruns the card. FIX: the dims
card's desired width is the max of the grid extent and the hint's
SHAPED width (plus pads); the grid centres horizontally in whatever
card results. The narrow-window yield (`hint_yielding_explanation`)
stays what it is — a genuine-window-constraint fallback, not a patch
for a self-inflicted width. Law: across zoom/DPI cells, the shaped
hint width fits inside `text_w` (non-vacuous: revert the max and
watch the 1× cell go red), plus a pixel assertion that the final
glyph column of the hint row carries ink inside the card bounds.

TO ASSESS, NOT ASSUME — "sort of overall in a weird position?"
(user, tentative): the card follows the standard summon placement
(frozen world `CardAnchor`, `CARD_TOP_DROP`), so on Kite it should
sit the top-right rail like every picker — but on a
plateless-backing world (Kite's `Ruled`, and the `Bars`/`Diagonal`
family) this card draws NO organizing ink at all: no rows for rules
or bars to structure, so a dense ink grid + one hint line float bare
over the frosted page. Run the standing vision-smoke: capture the
picker across the world roster and judge whether the plateless
members need a guaranteed backing for THIS card (a card whose content
is a drawn grid arguably always earns a plate/border, the way the
spell popup always carries its float panel — the "organizing absence"
of Ruled has nothing to organize here). Separately, put the
PLACEMENT QUESTION to the user with captures rather than deciding it:
(a) keep the world-anchor takeover placement (consistent with every
summon), or (b) anchor at the CARET like the contextual spell popup
(`CONTEXT_ANCHOR_DROP` precedent) — an insertion picker pointing at
its insertion point is the Word/Docs-dropdown intuition the user may
be reaching for. Record their pick on this item before moving the
card. Full gate receipt.

---
### 541 — table grid: the header-separator rule draws through a revealed row's source (user report, 2026-08-30)

Verified to the line. The per-row reveal contract
(`render/layers/table_grid.rs`, "grid and source never share a row's
pixels") is enforced for CELLS — the cell loop skips the revealed
x-ray row (`meta.revealed && xray_lines.contains(&doc_line)`,
line ~92) — but the ONE faint header-separator rule is pushed
UNCONDITIONALLY at `sep_doc_line` (lines ~126–137). Put the caret on
the separator row and its raw `|-|-|-|` source floats over the band
with the rule drawn straight through it (user screenshot). FIX: the
rule gets the same guard the cells have — suppressed when the
revealed row IS the separator's doc line. Sweep the sibling quads
while there: the pan bar (pushed in the same function) against a
revealed LAST row, same question. Law at the placement seam: with the
caret on each row of a table in turn, no rule/pan-bar rect intersects
the revealed row's band (non-vacuous: revert the guard, red on the
separator row). One-line-plus-law scale; per the land-easy policy
this can land for judgement. Full gate receipt.

---
### 542 — table EDITING is all raw-source friction: the low-hanging UX basket (user report, 2026-08-30 — "kinda awful to edit")

**FIRST WAVE DECIDED (user, 2026-08-30): fruits (1) Tab/Shift-Tab
cell hop and (2) Enter scaffold-row are greenlit — ready to
dispatch.** (3)/(4) stay parked pending their own word; (5) remains
deferred to its own design session. Land the first wave alongside
item 543 (empty cells render as nearly nothing), or a Tab-walk
through a fresh insert hops invisible cells.

The render half of tables is landed (grid, per-row reveal, dimension
picker — "AWESOME!!"); the EDITING half is still bare raw-source: the
caret's row drops to `| aa | bb |` and every pipe, pad, and cell hop
is hand-typed. Tables-as-real-grids is committed direction
(CLAUDE.md §Direction), so this friction is on-mission to remove.
Fruits ranked by leverage over cost — the user picks which to
greenlight; (1) and (2) are the recommended first wave:

1. **Tab / Shift-Tab = next / previous cell** while the caret is in a
   table: jump to the next cell's content start, wrapping across
   rows; Tab on the LAST cell appends a fresh scaffold row (the
   Obsidian/Typora convention). Pure caret/edit motion at the buffer
   seam, drivable by `--keys`, and the existing per-row reveal
   follows the caret for free. The single biggest ergonomic win.
   (Check the existing Tab binding's table context carefully — Tab
   currently indents/inserts; the table context must win only INSIDE
   a table block, and the law sweeps both contexts.)
2. **Enter inside a table = insert a scaffold row below** (`| | | |`
   matching the column count), never a mid-cell line split that
   breaks the table shape. Escape hatch stays: a literal split is
   still reachable (Shift-Enter or at-block-edge semantics — lane
   proposes, user confirms).
3. **Auto-align on row-leave**: the shipped `align_table` re-pad runs
   automatically (debounced, or when the caret leaves the table/row)
   so the source stays Prettier-shaped without summoning the command.
   Mind undo coalescing (the re-pad is its own sealed group) and
   caret preservation across the re-pad.
4. **Row/column verbs in the palette**: Insert row above/below,
   insert column left/right, delete row/column — source splices over
   the existing row/cell parser (`markdown/tables.rs`), gated to
   caret-in-table exactly like `AlignTable`'s availability gate.
5. NOT this item (the big arc): editing cells IN the grid without
   dropping to source. That is the "tables as real grids" destination
   and earns its own design session; nothing in 1–4 pre-empts it, and
   all four survive it (they are source-level operations the grid
   editor would also need).

Every fruit is exhaustively testable at the buffer seam (editing
edge-cases are the product — spend generously per CLAUDE.md), plus
`--keys` journey captures for Tab-walks and Enter-rows. Full gate
receipt per landing wave.

---
### 543 — an all-empty table renders as almost nothing: empty grid cells need a faint display-only affordance (user report, 2026-08-30)

Verified: `build_table` (`markdown/tables.rs`) emits ALL-EMPTY cells
(`| |` rows), and the WYSIWYG grid draws only cell TEXT plus the one
faint separator rule — no cell borders. So the dimension picker's own
fresh insert (item 517, the best-reviewed feature on this board)
lands as a single faint horizontal line floating in space: the
user's "an empty table doesn't really render the | | |". The first
minute of the flagship flow shows nearly nothing.

FIX, display-only (the file stays plain text; nothing is inserted
into cells): an EMPTY cell draws a faint placeholder affordance in
the grid — and the treatment is already designed one module over:
the dimension picker's empty cells draw `muted()` at low alpha
(`table_dims.rs`'s own comment records WHY: an opaque `base_200`
wash vanished bytewise on Wagtail's card, while translucent muted
over whatever ground is visible on every world by construction).
Reuse that reasoning for the document grid's empty cells (wash, or a
short faint underline — lane renders both, user picks from captures).
A cell that gains content drops its ghost individually; a table with
every cell filled is byte-identical to today's render. Also give
empty COLUMNS a minimum layout width (`compute_table_layout`) so an
8×8 empty table has real extent rather than collapsing to padding.
Laws: pixel presence of the affordance over every world (the
vision-smoke question "how many columns does this empty table
have?"), the filled-table byte-identity, and the per-cell drop as
content arrives. Sequenced WITH item 542's first wave — Tab-hopping
a fresh insert must land on visible cells. Full gate receipt.

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

🟡 IN PROGRESS — design-session Claude (this is the DESIGN PASS only: curate sets, render on real grounds, Artifact fitting-room for the user's set-to-world assignment; no product data lands from this claim)

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
