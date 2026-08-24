# fold-mark glyph candidate survey

`sh captures/item-475-glyph-survey/shoot.sh` from the repo root regenerates
the gallery sheets and re-runs the direction-at-rest law.

This is a SURVEY round: no shipped rendering code changes here. The fold
chevron (`src/render/layers/fold_chevron.rs`) still draws its four rotated
quads exactly as before — nothing in this directory is wired into it. The
gallery exists so a human can pick a candidate glyph; the wiring is a
separate, later round.

Hermetic-adjacent: the sheets photograph shaped text through the real
bundled-font `FontSystem`, not a live document — there is no seeded tree or
ambient path involved, so nothing here can leak a real directory.

## What's shown

Each sheet is a candidates-by-(level × turn) grid: 6 rows (candidates) × 4
columns (H1 collapsed, H1 expanded, H3 collapsed, H3 expanded). Every cell
pairs the candidate mark with an upright "Heading" specimen in the world's
own display face, at the SAME font-size ladder rung the real heading text
uses (`markdown::heading_scale`, off `render::FONT_SIZE`) — so the taste call
can be made beside a heading, not on an isolated glyph.

| sheet | world | DPI |
| --- | --- | --- |
| `galah-light-1x.png` | Galah (light) | 1x |
| `galah-light-2x.png` | Galah (light) | 2x (Retina) |
| `bowerbird-dark-1x.png` | Bowerbird (dark) — the user's own reported world | 1x |
| `bowerbird-dark-2x.png` | Bowerbird (dark) | 2x (Retina) |

PNGs are scratch (untracked) — `shoot.sh` and this README are the checked-in
reproducer.

## Candidates (rows, top to bottom)

Every candidate has REAL glyph coverage in an ALREADY-BUNDLED, already-OFL
face — confirmed with a direct `ttf_parser` read over every `assets/fonts/
*.ttf` file (`face.glyph_index(ch)` + `face.glyph_bounding_box(gid)`
returning real, non-degenerate boxes), not assumed from Unicode charts. No
font asset changes, no new license surface: see "Licensing" below.

1. **EB Garamond, U+203A `›`** (SINGLE RIGHT-POINTING ANGLE QUOTATION
   MARK) — the pre-quad original, from the warm prose serif already bundled
   as several worlds' `Theme::font`.
2. **Iosevka, U+203A `›`** — the same codepoint from the mono/code face: a
   thinner, more geometric stroke than the serif reading, answering "reads
   fat" from a different face rather than a thinner quad.
3. **EB Garamond, U+3009 `〉`** (RIGHT ANGLE BRACKET) — the item's named
   "angle bracket family" lead. Real coverage in the SAME already-bundled EB
   Garamond — no CJK face or new composition needed to show it, even though
   the codepoint is CJK-flavored full-width punctuation.
4. **Iosevka, U+25B8 `▸`** (BLACK RIGHT-POINTING SMALL TRIANGLE) — found by
   inspecting coverage, not named in the brief: the classic disclosure
   triangle (Finder/macOS convention), a different visual language than the
   angle-quote family above.
5. **JetBrains Mono, U+276F `❯`** (HEAVY RIGHT-POINTING ANGLE QUOTATION MARK
   ORNAMENT) — a heavier weight in the same angle-quote family, from a third
   already-bundled mono face.
6. **EB Garamond, U+261E `☞`** (WHITE RIGHT POINTING INDEX) — the wildcard,
   added at the user's request for one wilder option: the MANICULE, the
   pointing hand scribes drew in manuscript margins to flag a passage.
   Same warm serif as candidate 1; points right at rest, so the
   quarter-turn grammar and the direction-at-rest law hold unchanged
   (fingertip tapers, cuff is the open end).

   A follow-up scan of the ten bundled CJK faces (same `ttf_parser`-class
   read, via fontTools) found no directional wildcard there: the JP/KR/SC
   faces carry ※ 〆 〒 ♪ (and NotoSansKR alone carries 〽 and 〠), but
   none of the manicule family, no shogi pieces, no arrows — nothing with
   a point that could survive the direction law. The vertical presentation
   forms remain absent from all 45 files, as already noted below.

## Leads investigated and dropped

- **Vertical presentation forms U+FE3F `︿` / U+FE40 `﹀`** (the item's own
  named lead) **and their corner-bracket siblings U+FE41 / U+FE42: ZERO
  coverage in any of the 45 bundled `.ttf` files.** These cannot be shown
  this round without tofu. Showing them requires sourcing a NEW face (or
  compositing them into `AwlMarks.ttf` from an external Noto artifact, the
  same operation that built `AwlMarks.ttf`'s existing 34 codepoints) — that
  is next-round font-asset work, not a rendering choice, and is explicitly
  out of scope here.
- **CJK corner brackets U+300C `「` / U+300D `」`**: real coverage (Gowun
  Batang, Klee One, LXGW WenKai, Noto Sans/Serif JP/SC, Shippori Mincho, Zen
  Maru Gothic), but the SHAPE is a right-angle quotation bracket, not a
  wedge — it carries no directional "point" to rotate, so it was not
  rendered.
- **Modifier letter arrowheads U+02C3 `˃` / U+02C5 `˅`**: real coverage, but
  Iosevka-only and cap-height-anchored (modifier-letter class, sits near the
  x-height rather than centered on the mark's own box) — a real candidate,
  just a visually rougher fit; not carried into the gallery this round.
- **U+2304 DOWN ARROWHEAD `⌄` / U+2303 UP ARROWHEAD `⌃`**: `⌃` is already in
  `AwlMarks.ttf` (the modifier-key keycap glyph); `⌄` exists only in
  JetBrains Mono. A mismatched pair from two different sources was not
  pursued as a single-glyph rotation candidate.

## Font coverage table (the actual survey, `ttf_parser` reads)

| codepoint | glyph | bundled faces with real coverage |
| --- | --- | --- |
| U+203A | `›` | every bundled Latin display/mono face (EB Garamond, Iosevka, JetBrains Mono, IBM Plex Sans/Mono, Fira Sans, Bitter, Fraunces 9pt, Literata, Newsreader, Zilla Slab, Sour Gummy, Archivo Black, Abril Fatface, Monaspace Xenon, iA Writer Quattro S, Figtree) |
| U+3008/3009 | `〈`/`〉` | EB Garamond, Gowun Batang, Klee One, LXGW WenKai, Noto Sans JP/KR/SC, Noto Serif JP/SC, Shippori Mincho, Zen Maru Gothic |
| U+300A/300B | `《`/`》` | same set as U+3008/3009 |
| U+300C/300D | `「`/`」` | Gowun Batang, Klee One, LXGW WenKai, Noto Sans JP/KR/SC, Noto Serif JP/SC, Shippori Mincho, Zen Maru Gothic |
| U+FE3F/FE40 | `︿`/`﹀` | **none** |
| U+FE41/FE42 | (vertical corner brackets) | **none** |
| U+25B8 | `▸` | Iosevka, JetBrains Mono, Monaspace Xenon, Zilla Slab |
| U+25BE | `▾` | Iosevka, JetBrains Mono, Monaspace Xenon |
| U+25B6/25BC | `▶`/`▼` | Bitter, EB Garamond, Iosevka, JetBrains Mono, Monaspace Xenon, Zilla Slab (+ Klee One/Noto Sans JP/Noto Serif JP/Shippori Mincho for `▼` only) |
| U+02C3/02C5 | `˃`/`˅` | Iosevka only |
| U+276F | `❯` | JetBrains Mono, Monaspace Xenon |
| U+2304 | `⌄` | JetBrains Mono only |
| U+2303 | `⌃` | already in `AwlMarks.ttf` |
| U+232B | `⌫` | already in `AwlMarks.ttf` |

## Direction-at-rest, proved not assumed

`fold_mark_candidates_settle_in_opposite_directions`
(`src/render/tests/fold_mark_candidate_gallery.rs`) grades this on rendered
pixels for every candidate at both H1 and H3:

- An exact-transpose law: the collapsed and expanded ink boxes' width/height
  swap under the quarter turn (a hard geometric identity for any non-square
  glyph rotated 90°) — proves the turn genuinely rotates the mask rather
  than leaving it inert.
- A taper law: the expanded mark's horizontal ink SPAN narrows toward its
  BOTTOM edge and stays wide at its TOP edge — the actual vertex-points-down
  signature. (An earlier version of this law graded ink DENSITY top-half vs
  bottom-half and was wrong: a right-pointing wedge is physically wider —
  more ink area — at its open end than at its point, in EITHER orientation,
  so a density check does not distinguish direction. Span narrowing does.)

Both laws are non-vacuous against a broken sign: flipping `turn_deg`'s
`270.0` to `90.0` (the OTHER quarter turn) fails the taper law immediately —
verified by hand while writing it, not left as an assertion nobody has seen
fail.

## Licensing

Every rendered candidate comes from an already-bundled, already-attributed
OFL 1.1 face (`assets/fonts/LICENSES.md`): EB Garamond, Iosevka, JetBrains
Mono. Zero new font bytes, zero new license surface, nothing to add to that
file. The one flagged gap is the vertical-form lead above — showing it
requires new sourcing, and that sourcing (which upstream artifact, which
SHA-256, which license text) is unverified because it has not been fetched;
nothing about it is asserted here.

`AwlMarks.ttf` (awl's own composed symbol face, 34 codepoints — keycaps,
fleurons, ornaments) already exists in the tree from unrelated prior rounds
and carries none of these candidates. No mark ships unpicked; wiring a
choice into `fold_afford`/`RenderCaps` — and, if the vertical-form lead is
the pick, composing it into `AwlMarks.ttf` — is next-round work.
