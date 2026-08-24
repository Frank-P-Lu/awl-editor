# symbol atlas — item 486 shopping catalog

`sh gallery/item-486-symbol-atlas/shoot.sh` from the repo root regenerates
`symbol-atlas.html` (scratch, untracked — see `.gitignore`'s `/gallery/*`
exception carved out for this directory's `shoot.sh`/`README.md` only).

This is a SURVEY round: no shipped rendering code changes here, and nothing
here decides which mark ships anywhere in the product. The generator is
`src/render/tests/symbol_atlas_gallery.rs`, an `#[ignore]`d unit test gated a
second way on `AWL_SYMBOL_ATLAS_OUT` (unset = total no-op, so no gate,
filtered or unfiltered, ever writes gallery files) — the same pattern
`captures/item-475-glyph-survey/fold_mark_candidate_gallery.rs` established,
adapted for an HTML deliverable instead of PNG sheets.

## What it is

A shopping catalog for future marks (fold glyphs, ornaments, bullets, world
flourishes) built entirely from fonts awl **already bundles** — nothing
picked from this page carries a new-font cost. For every `assets/fonts/*.ttf`
file, the generator reads that face's own `cmap` table directly via
`ttf_parser` (a fontTools-style binary read — `Subtable::codepoints`, unioned
across every subtable the face carries), filters to a curated set of
"symbol-ish" Unicode blocks (see below), and for each surviving codepoint
pulls the glyph's own name from the face's `post`/CFF glyph-name table
(`ttf_parser::Face::glyph_name`) where present — **never invented**. Unnamed
glyphs show as `(unnamed)` rather than a guessed label.

Reading the `cmap` directly, not a Unicode chart, matters concretely:
`Junicode-Ornaments.ttf` maps 48 codepoints, and **43 of the 48 (90%)** sit
in the Private Use Area (`U+E000`-`U+F8FF`) — measured directly off the
generated page's own codepoint list, not eyeballed. A chart-range approach
would have found almost nothing there, since PUA codepoints carry no
chart-assigned meaning at all. The font's own `post` table still names
most of them (`uniE270` etc. — real embedded names, not the chart's silence).

The page groups by face, then by Unicode block, and for every glyph cell
lists the roster of every OTHER bundled face that also carries that exact
codepoint (checked directly per face via `glyph_index`, not derived from the
block filter, so cross-face membership is exact even where the filter's
`AwlMarks.ttf` exemption below widens one face's own scope beyond the
others').

Rendering is via the **browser**, not the real awl pipeline: each face
contributing at least one in-scope glyph is embedded whole as a base64
`@font-face` `data:` URI (a plain hand-rolled base64 encoder — no new crate
dependency for a single-use, `#[ignore]`d survey tool). The item's own text
accepts this for a browse-only inventory; it is explicitly **not** a
replacement for the real pipeline (`FontSystem`/`SwashCache`/GPU) for any
actual taste call — see "Known limitation" below for why that distinction is
not academic here.

## The Unicode block filter (labels, not gates)

`SYMBOL_BLOCKS` in the generator is a LABEL/GROUPING filter over codepoints a
face's cmap already reported — it never decides whether a face "should"
cover something. In scope: General Punctuation, Letterlike Symbols, Arrows
(+ Supplemental A/B), Miscellaneous Technical, Control Pictures, OCR,
Enclosed Alphanumerics, Geometric Shapes, Miscellaneous Symbols, Dingbats,
Miscellaneous Symbols and Arrows, CJK Symbols and Punctuation, Enclosed CJK
Letters and Months, CJK Compatibility, Vertical Forms, CJK Compatibility
Forms, Small Form Variants, and the Private Use Area. Deliberately out:
Basic Latin/Latin-1 (plain ASCII), Superscripts/Subscripts, Currency
Symbols, Number Forms, pure Mathematical Operators, Box Drawing/Block
Elements, Braille, and the bulk CJK Unified Ideographs/Kana/Hangul blocks —
real coverage, but not ornament material, and the ideograph blocks alone
would drown the catalog in tens of thousands of rows per CJK face.

**One block was tried and dropped after direct measurement: Halfwidth and
Fullwidth Forms (U+FF00-FFEF).** Including it inflated Noto Sans JP from 65
to 157 in-scope codepoints — measured, not assumed — because 85-95% of that
block in every bundled CJK face is fullwidth *duplicates of plain ASCII
letters and digits* (Ａ-Ｚ, ０-９), not ornament material. Dropping it
brought Shippori Mincho to exactly 56 in-scope codepoints, matching this
item's own pre-round estimate precisely — strong independent confirmation
the final block list, not the discarded one, is the right filter.
`AwlMarks.ttf` (awl's own composed symbol face) is exempt from the block
filter entirely: every codepoint it maps is already, by construction, a
symbol/keycap/ornament glyph.

`Face::glyph_index` is queried per candidate codepoint mainly to resolve a
glyph ID for `glyph_name` and to confirm the codepoint actually maps to a
real (non-`.notdef`) glyph — `codepoints()` itself can report codepoints a
subtable defines but resolves to glyph 0, per its own doc.

## Measured inventory (confirmed/corrected against the item's own numbers)

All 45 bundled `.ttf` files carry at least one in-scope codepoint; **1082
distinct in-scope codepoints** total across the whole roster.

| face | in-scope codepoints | vs. the item's pre-round estimate |
| --- | --- | --- |
| `AwlMarks.ttf` | **34** | matches exactly |
| `Junicode-Ornaments.ttf` | **48** | matches exactly |
| `ShipporiMincho-Regular.ttf` | **56** | matches exactly |
| `NotoSansJP-Regular.ttf` | **65** | estimate said "~94" — **measured lower**; see Halfwidth/Fullwidth note above for why the estimate likely ran high |
| `NotoSerifJP-Regular.ttf` | **65** | same correction as Noto Sans JP |
| `NotoSansSC-Regular.ttf` / `NotoSerifSC-Regular.ttf` | **111** each | not separately estimated by the item |
| `NotoSansKR-Regular.ttf` | **64** | not separately estimated |
| `Iosevka-Regular.ttf` / `-Bold.ttf` | **364** each | the single widest-coverage Latin/mono face — not estimated, and see "Known limitation" below |
| `MonaspaceXenon-Regular.ttf` / `-Bold.ttf` | **734** each | the widest coverage of any bundled face (Nerd-Font-style icon set lives in its own PUA range) |
| every other Latin display/mono face | 13-61 each | "scattered dingbat/ornament coverage," confirmed — full per-face table is in the generated page itself |

Per the round's own methodology note (a defect/estimate report is a
hypothesis, and the measurement that produced it is part of what needs
checking): the "~94 each" CJK figure in the item's own text does not survive
a direct `cmap` read once fullwidth-Latin duplicates are correctly excluded
— the real number is 65 for the Noto JP pair. The other three pre-supplied
figures (AwlMarks 34, Junicode 48, Shippori ~56) all measured exactly right,
which is the independent evidence that the correction above is the filter
bug, not a new one.

## Known limitation (measured, not theoretical) — Iosevka renders wrong in Chromium

Loaded the generated page in a real Chromium instance (Playwright) and read
`document.fonts` after load: **`Iosevka-Bold.ttf` and `Iosevka-Regular.ttf`
both fail to decode**, logging `OTS parsing error: glyf: Bad glyph flag
(101), bit 6 must be set to zero for flag 1` before `Failed to decode
downloaded font`. `ttf_parser` (this generator's own reader) parses both
files without complaint, and the bytes embedded are byte-identical to what
`render.rs` already `include_bytes!`s into the shipping binary — so this is
Chromium's OTS font sanitizer rejecting a glyph-flag bit pattern in
Iosevka's `glyf` table (very plausibly the `OVERLAP_SIMPLE` flag some
font-build toolchains set, which predates or exceeds OTS's own
understanding), not a defect in the font file or in this tool's read of it.

**The failure is not visually obvious.** Chromium does not draw tofu for the
two Iosevka sections — it silently falls back to the system default font.
Many of Iosevka's punctuation-block glyphs (hyphen, en/em dash, double
vertical line, etc.) then render as a *plausible-looking but wrong* shape:
the system font's interpretation, not Iosevka's own. A screenshot of the
Iosevka-Bold section confirmed this directly (blank cells for whitespace
codepoints, substituted-font glyphs for visible ones — never Iosevka's
actual glyph). **Do not judge an Iosevka candidate's shape from this page.**
Re-render it through the item-475 gallery machinery (the real
`FontSystem`/`SwashCache`/GPU path, which does not run Chromium's sanitizer
and is what the app itself uses) before any taste call touches an Iosevka
glyph. Every other one of the 45 faces loaded and rendered correctly per the
same `document.fonts` check.

## Owed to whoever publishes this next

The generated `symbol-atlas.html` is **~49MB** — every face contributing at
least one in-scope glyph is embedded whole (raw `.ttf` bytes, base64), and
several bundled CJK faces are multi-megabyte on their own (`KleeOne-Regular.ttf`
alone is 4.7MB). That is fine for local browsing but **exceeds a 16MB
publish cap** some downstream tooling enforces — this generator makes no
attempt to fit under one, since nothing in the item's own text asked for
that. Getting it under a cap needs one of: per-face glyph subsetting (keep
only the in-scope codepoints' outlines, not the whole font), splitting into
one page per face, or dropping the heaviest whole-font CJK embeds in favor
of static PNG crops. None of that is done here.

Also worth knowing before judging any "empty" cell: a handful of the General
Punctuation entries are genuinely invisible characters (various
fixed/variable-width SPACE codepoints, e.g. `U+2000`-`U+200A`, and
zero-width format controls like `U+200B`/`U+200C`/`U+200D`) — a blank glyph
cell for one of those is CORRECT, not a rendering failure. It is only a
concern when it coincides with the Iosevka rows above.

## Leads investigated and dropped

- **Rendering through the real awl pipeline instead of the browser**: ruled
  out for THIS deliverable by the item's own text (a browse-only inventory
  may use `@font-face`), but the Iosevka finding above is exactly the risk
  that constraint carries, made concrete rather than theoretical.
- **Full Halfwidth and Fullwidth Forms block**: tried, measured, dropped —
  see above. Kept out of the generator entirely rather than left in as a
  disabled option, since a chart-derived "this block sounds relevant" guess
  is precisely the failure mode this item's own text warns against ("never
  Unicode chart ranges").
- **Deriving `roster` from the per-face scoped map instead of a direct
  `glyph_index` re-check per face**: the scoped map would have been
  cheaper, but it under-reports the roster for any codepoint that entered
  the catalog only via `AwlMarks.ttf`'s block-filter exemption — a regular
  face could still carry that exact codepoint without it being "in scope"
  by ITS OWN filter pass. The direct re-check (45 faces × ~1100 codepoints,
  trivial cost) is exact regardless of which face first admitted a
  codepoint.
- **A pulled-in `base64` crate dependency**: not added. A ~20-line
  hand-rolled encoder is simpler to review than a new `Cargo.toml` entry for
  a single-use `data:` URI encoder in an `#[ignore]`d survey tool, and adds
  zero build-graph or license-audit surface (`docs/licensing.md`'s
  `scripts/audit.sh` inventory stays untouched).

## Licensing

Every embedded face is already-bundled and already-attributed OFL 1.1
(`assets/fonts/LICENSES.md`: "Every bundled face is distributed under the
SIL Open Font License, Version 1.1" — confirmed 45-file roster, matching
`assets/fonts/*.ttf`'s own count). Zero new font bytes, zero new license
surface: this tool only re-encodes bytes awl already ships, as base64, into
a scratch HTML file that is never committed.
