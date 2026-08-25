# item 487 — Magpie/Mangrove diagonal picker composition candidate gallery

`sh captures/item-487-magpie-diagonal/shoot.sh` from the repo root regenerates
the gallery and re-runs `measure.py`'s pixel-arithmetic checks.

Hermetic: the sandbox is seeded from `fixture/` alone, through an explicit
`--config` and `--root` — never the ambient project or the ambient config —
so nothing here photographs a real directory. The PNGs and their sidecars are
scratch and are not committed; the fixture, `shoot.sh` and `measure.py` are,
so the set survives the worktree that produced it (mirrors
`captures/item-444-residual3/README.md`'s own note).

## What this gallery is

Item 487 names three compounding symptoms in the Diagonal-world picker
composition — query-to-first-item distance, frost boundaries crossing
legible content, and a stranded selected-row chevron — already reproduced
and diagnosed headlessly before this round. This gallery is the **candidate
audition** the item asks for, following the item-444/item-475 pattern: for
each symptom, the CURRENT (broken) state and one or more candidate fixes,
across the worlds that actually use this composition, at both DPIs. **It is
not the shipped fix** — no candidate is the new default; the user picks.

## Which worlds this composition touches (derived, not assumed)

`grep -n "ListStyle::Diagonal" src/theme/worlds.rs src/theme/worlds/*.rs`
finds exactly two: **Mangrove** (`DiagonalSpine::descending`, the CRISP mark)
and **Magpie** (`DiagonalSpine::ascending`, the HAIRLINE mark). Both are in
this gallery — not Magpie alone, and not a third one that isn't there. (Item
487's own text also names Cassowary's "dead band" as the same defect
*family*; Cassowary's `list_style` is `ListStyle::Pane`, a different
composition entirely, fixed separately in the 476–482 wave — it does not
enrol here and isn't in this gallery.)

## The fixture

`fixture/workspace/doc.md`: one document with a real H1 title, two paragraphs
of prose under it (long enough for a raking side face to cross mid-word), and
a second heading + paragraph so the document has real depth. `fixture/awl.toml`
sets `theme = "Magpie"` (each shot's own `--theme world` overrides it) and
`file_visibility = true` (load-bearing under a dot-prefixed worktree path —
see `captures/item-444-residual3/README.md`'s own note on why).

The repro chord is the item's own: `Cmd-p t h e m e Ret` — the command
palette, filtered to "Switch theme", accepted — which lands on the **theme
picker itself** (`mode: "theme"`, an unfiltered flat list of all 20 worlds).
That picker inherits the ACTIVE theme's own `ListStyle`, so under `--theme
Magpie`/`--theme Mangrove` it renders in that world's own Diagonal
composition — the theme picker turns out to be the cleanest reproduction
surface named by the item, not a separate contrivance.

## What is genuinely NOT built, and this gallery does not claim otherwise

No shipped rendering code's DEFAULT behavior changed. Every candidate lives
behind its own `AWL_DIAGONAL_GALLERY_*` env var, read in exactly one place
each (`src/render/chrome/diagonal/gallery.rs`'s own module doc), resolving to
`None`/`false` — and therefore the SAME code path the shipped composition
already draws through — on every ordinary run, including every "current"
shot in this gallery (which sets none of these vars). The one behavior
change reachable without an env var is a pure refactor with no default
change: `overlay_query_caret_box` and `overlay_panel_bands`'s head band now
read their left seat through the new `overlay_head_left` owner instead of
inlining `geom.text_left` — which still returns `geom.text_left` whenever
`AWL_DIAGONAL_GALLERY_QUERY` is unset (see `offband.rs`'s own doc on
`overlay_head_left`). `cargo test --bin awl diagonal` (27 tests),
`footprint` (16 tests), `overlay_` (177 tests) and `query_caret` (4 tests)
all still pass unforced.

The query-right candidate is a **placement mock**, not a finished feature:
`offband.rs`'s own design comment says right-aligning the query field was
rejected on purpose — *"a query FIELD is an input, and right-aligning one on
a mirrored composition would make its sigil travel as the user types."* A
static capture cannot show that travel, and this gallery does not pretend
otherwise; the tradeoff the comment names is real and unresolved by this
candidate. Judge the still frame; the travel-while-typing cost is a live
question for whoever picks this direction.

## Candidates, by symptom

### 1. Query-to-first-item distance

- **`*-frost-full-*`** — full-canvas frost instead of the card's own
  footprint (`AWL_DIAGONAL_GALLERY_FROST=full`). The whole page recedes a
  value together, so the empty band between the query caret and the
  right-anchored first item stops reading as "dead space over sharp
  document" — it's all one frosted field. See the `wide-*-1x` pair below for
  the item's own motivating case (a wide window).
- **`*-query-right-*`** — the query header right-aligned against the card's
  own text column (`AWL_DIAGONAL_GALLERY_QUERY=right`), instead of seated at
  the card's left text edge. Measured (`measure.py`, XOR-isolated caret
  position — see its own module doc): on Magpie 1x the caret moves from
  **x=156.5 to x=667.0** while the first item's own label sits at
  **x=557.8** — the raw caret-to-first-item distance falls from **401px to
  109px**. At 2x: **749px → 272px**. The caret now overshoots the first
  item slightly (the card's own text column runs a little past the row
  content) rather than landing short of it — a real side effect, not hidden
  in this report.

### 2. Frost boundaries crossing legible content

- **`*-frost-top0-*`** — the footprint's top face seated at the canvas top
  (`AWL_DIAGONAL_GALLERY_FROST=top0`) instead of the card's own top edge, so
  the H1 sits entirely inside the frosted box rather than straddled by its
  boundary. **Pivot-compensated**: `gallery::seat_top_above_first_line`'s own
  doc explains why a naive rect edit would silently slide the raking side
  faces too (the parallelogram un-shears about the box's own vertical
  centre, and moving the top edge moves that centre) — the candidate shifts
  `x` by `shear * Δcy` to hold every side face at its original canvas
  position. Verified (not assumed, and swept across the roster rather than
  spot-checked on one cell): `measure.py`'s `frost_diff` diffs the current
  and candidate PNGs at the same `>10`-per-channel threshold it counts by
  (raw `Image.getbbox()` is the wrong tool here — the changed footprint
  shape feeds back into the ordered-posterization dither signature
  `pipeline_prepare.rs` documents, and that sub-perceptual noise floor
  spans nearly the whole canvas; see the function's own doc), then
  `frost_diff_stays_above_row_band` checks the thresholded bbox's bottom
  edge against the shot's own `overlay.window.band.first_top`. All four
  world×DPI cells pass: `magpie-1x (192,7,730,49)`, `magpie-2x
  (62,0,1171,98)`, `mangrove-1x (488,7,919,50)`, `mangrove-2x
  (451,18,1025,100)` — every one CONTAINED above its own row band, so the
  candidate's diff never reaches the rows or the document body below them;
  the side faces held still on both worlds and both DPIs, not just the one
  cell first inspected by eye.
  Visually (reproducible from the PNGs themselves): Magpie's "A Document
  With A Real T‸itle" reads as a melting half-legible ghost in **current**;
  in **top0** the same span reads as a clean, uniform blur — no straddling
  boundary. Mangrove's H1 wraps ("A Document" / "With A Real Title") and
  the word boundary crossing is even clearer: **current** shows "A
  Document" sharp bleeding into a melting "With A Real Title"; **top0**
  shows "A Document" sharp and "With A Real Title" uniformly, fully
  blurred — the boundary moved to fall between words instead of through
  glyphs.
- The item's other two placement complaints under symptom 2 — the raking
  side face slicing mid-sentence, and the gray wedge over the empty left
  margin — are visible in every `*-current-*` shot (the raking spine crosses
  the fixture's own prose paragraphs) but this round did not audition a
  separate candidate for the side face's own placement; `frost-full`
  removes it by removing the footprint shape entirely, which is the
  candidate on offer for that half of symptom 2.

### 3. Stranded selected-row chevron

- **`*-chevron-short-*`** — the vertex seated at the row's own measured NAME
  ink end (+ the composition's own mark gap) instead of the far edge of the
  row's whole reserved cluster width (`AWL_DIAGONAL_GALLERY_CHEVRON=short`;
  implemented in `prepare_diagonal_spine`, reading
  `overlay_row_primary_px` — the SAME shaped-buffer measurement
  `overlay_panel_bands` already uses for label placement, never a second
  shaping pass). Measured (`measure.py`'s `chevron_report`): on Magpie 1x
  the isolated ink run at `(181, 192)` — 262–343px from the selected row's
  own label box — is present in **current** and **absent** in **short**;
  in its place, `short`'s label run gains **13.5px of extra fused ink**
  directly abutting the label (`label_fused_extra_px`), i.e. the chevron now
  touches what it's marking rather than sitting in an unrelated blank column.
  At 2x the same far run vanishes the same way, but `label_fused_extra_px`
  reads `0.0` there — that row's mark seats close enough to touch the
  label's antialiasing without adding measurable extra width, not a sign
  the candidate did less at 2x. Read the disappearance of the far run, not
  the fusion number, as the both-DPI confirmation. Mangrove shows the same
  effect visually (`mangrove-chevron-short-1x.png`:
  `Mangrove<` — the chevron immediately right of the label it was
  ~330px away from in `mangrove-current-1x.png`).

## The shots

Per world (Magpie, Mangrove) × DPI (1x, 2x): `current`, `frost-full`,
`frost-top0`, `chevron-short`, `query-right` — 20 shots. Plus one WIDE-window
pair (Magpie, 1x, `--capture-size 1600x900`): `wide-current` and
`wide-frost-full`, for the item's own "a wide window leaves most of the
document crisply readable beside frosted fragments" case, which a
1200-wide capture is too narrow to show. 22 PNGs total, matching item-475's
1x/2x convention.

Every shot's sidecar reports `driver: "replay"` (an ordinary `--screenshot`
capture reaches the theme picker fine; no live-App-only transition is
needed here) and `replay_skips: []` — the `Cmd-p … Ret` chord replays
cleanly start to finish.

## What `measure.py` checks, and its one named gap

Chevron and caret measurements are **Magpie-only**: Mangrove's
`Background::Pinstripe` ground is a fine dot texture, and the flat
color-distance threshold that correctly isolates ink from Magpie's flat/banded
ground cannot tell "ink" from "background dot" on Mangrove without a
texture-aware oracle this throwaway script does not build. Mangrove still
gets the DPI-and-texture-agnostic `frost_top0_diff` check (plain pixel
difference, unaffected by the ground texture) and its own gallery shots for
a human's visual read — the "Mangrove `<`" adjacency and the H1
word-boundary claims above are read directly off the committed-script-
reproducible PNGs, not asserted by `measure.py`.

## Recommendation (the author's own taste read — the user decides)

- **Frost top-seat (`frost-top0`)**: recommend adopting this shape of fix (a
  pivot-compensated rect extension) for the shipped composition — it
  resolves the H1-melting complaint with no visible cost in either world,
  and the pixel diff confirms it changes nothing outside the head band.
- **Chevron reach (`chevron-short`)**: recommend adopting the underlying
  idea (anchor the mark to measured name ink, not the reserved cluster
  width) — worth the shipping round doing it at `mark_span`'s own layer
  (`cluster.rs`) rather than this round's `prepare_diagonal_spine`-level
  override, per the "one owner" principle, once picked.
- **Query-to-first-item**: no clear winner between `frost-full` and
  `query-right` — they're different tradeoffs (full-frost changes the
  picker's whole visual register on every Diagonal-world card; query-right
  keeps the crisp-preview footprint but reopens the sigil-travel question
  `offband.rs` deliberately closed). Both are real, both are shown; this is
  the taste call most worth putting to the user directly.
