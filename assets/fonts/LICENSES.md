# bundled font licenses

awl bundles the font faces in this directory into the application binary at
build time (`assets/fonts/`, loaded by `render.rs`). **Every bundled face is
distributed under the SIL Open Font License, Version 1.1** — with the one
documented exception called out below. (The former `AwlSymbols.ttf` — the app's
only non-OFL asset, a DejaVu/Bitstream-Vera derivative — has been RETIRED and
rebuilt as the all-OFL `AwlMarks.ttf`; the Bitstream dependency is gone.) This
file is the index; the full OFL 1.1 text travels with the fonts as
[`OFL.txt`](./OFL.txt), and the per-foundry copyright lines are preserved in the
`OFL-*.txt` files for the CJK faces.

Each row's copyright holder is taken from the face's **own embedded `name`
table** (nameID 0 = copyright; nameID 7 = trademark for the one documented
Monaspace Xenon exception in the table below) — the authoritative ground
truth, not an assumption, and confirmed present for **all 45** bundled `.ttf`
files by a direct `name`-table read (item 255, 2026-08-04;
`render::tests::font_licence_item255` holds this permanently — see below).
The `license` column is taken from the same embedded fields (nameID 13/14)
where the face carries them, and otherwise from the upstream project's own
published grant, the same pattern the exception note below spells out in
full for iA Writer Quattro S.

> This file covers the bundled FONTS only. It makes **no claim** about the
> license of awl's own source code, which is a separate, still-undecided matter.

## ⚠ one exception to flag (not embedded — read before assuming)

1. **`iAWriterQuattroS-Regular.ttf` embeds NO license statement.** Its name
   table carries only `Copyright 2017 IBM Corp. and iA Inc. All rights
   reserved.` with an *empty* license-description field and *no* license URL —
   so the font file itself does not assert OFL. The upstream project
   (Information Architects, [github.com/iaolo/iA-Fonts](https://github.com/iaolo/iA-Fonts))
   publishes the iA Writer families under the **SIL OFL 1.1** (see that repo's
   `LICENSE.md`); we rely on that upstream grant. Recorded as OFL 1.1 by
   upstream provenance, with the caveat that the embedded table is silent.

## per-face table

| file | family | copyright holder (nameID 0) | reserved font name | license | source |
|------|--------|------------------------------|--------------------|---------|--------|
| `Bitter-Regular.ttf` | Bitter | The Bitter Project Authors | — | SIL OFL 1.1 | github.com/solmatas/BitterPro |
| `EBGaramond-Regular.ttf` | EB Garamond | The EB Garamond Project Authors | — | SIL OFL 1.1 | github.com/octaviopardo/EBGaramond12 |
| `Figtree-Regular.ttf` | Figtree | The Figtree Project Authors | — | SIL OFL 1.1 | github.com/erikdkennedy/figtree |
| `FiraSans-Regular.ttf` | Fira Sans | The Mozilla Foundation and Telefonica S.A. | — | SIL OFL 1.1 | github.com/google/fonts/tree/main/ofl/firasans |
| `Iosevka-Regular.ttf` | Iosevka | Renzhi Li (aka. Belleve Invis) | — | SIL OFL 1.1 | github.com/be5invis/Iosevka |
| `Junicode-Ornaments.ttf` | Junicode | Peter S. Baker | — | SIL OFL 1.1 | github.com/psb1558/Junicode-font |
| `Fraunces9pt-Regular.ttf` | Fraunces 9pt | The Fraunces Project Authors | — | SIL OFL 1.1 | github.com/undercasetype/Fraunces |
| `iAWriterQuattroS-Regular.ttf` | iA Writer Quattro S | IBM Corp. and iA Inc. | — | SIL OFL 1.1 (upstream — not embedded, see note ②) | github.com/iaolo/iA-Fonts |
| `IBMPlexMono-Light.ttf` | IBM Plex Mono | IBM Corp. | IBM Plex (trademark) | SIL OFL 1.1 | github.com/IBM/plex |
| `IBMPlexSans-Regular.ttf` | IBM Plex Sans | IBM Corp. | — | SIL OFL 1.1 | github.com/IBM/plex |
| `JetBrainsMono.ttf` | JetBrains Mono | The JetBrains Mono Project Authors | — | SIL OFL 1.1 | github.com/JetBrains/JetBrainsMono |
| `Literata-Regular.ttf` | Literata | The Literata Project Authors | — | SIL OFL 1.1 | github.com/googlefonts/literata |
| `MonaspaceXenon-Regular.ttf` | Monaspace Xenon | GitHub, Inc. | Monaspace, Monaspace Xenon (+ Argon/Neon/Radon/Krypton) | SIL OFL 1.1 | github.com/githubnext/monaspace |
| `Newsreader-Regular.ttf` | Newsreader | The Newsreader Project Authors | — | SIL OFL 1.1 | github.com/productiontype/Newsreader |
| `ZillaSlab-Regular.ttf` | Zilla Slab | The Mozilla Foundation | Zilla (trademark) | SIL OFL 1.1 | github.com/typotheque/zilla-slab |
| `LXGWWenKai-Regular.ttf` | LXGW WenKai | LXGW; The Klee Project Authors | 霞鹜 / 霞鶩 / 落霞孤鹜 / 落霞孤鶩 / LXGW | SIL OFL 1.1 (+ additional permission — see `OFL-LXGWWenKai.txt`) | github.com/lxgw/LxgwWenKai |
| `NotoSansJP-Regular.ttf` | Noto Sans JP | Adobe | Source | SIL OFL 1.1 | github.com/notofonts / Google Fonts |
| `NotoSansKR-Regular.ttf` | Noto Sans KR | Adobe | Source | SIL OFL 1.1 | github.com/notofonts / Google Fonts |
| `NotoSansSC-Regular.ttf` | Noto Sans SC | Adobe | Source | SIL OFL 1.1 | github.com/notofonts / Google Fonts |
| `NotoSerifJP-Regular.ttf` | Noto Serif JP | Adobe | — | SIL OFL 1.1 | github.com/notofonts / Google Fonts |
| `NotoSerifSC-Regular.ttf` | Noto Serif SC | Adobe | — | SIL OFL 1.1 | github.com/notofonts / Google Fonts |
| `ShipporiMincho-Regular.ttf` | Shippori Mincho | The Shippori Mincho Project Authors | — | SIL OFL 1.1 | github.com/fontdasu/ShipporiMincho / Google Fonts (static Regular, subset to JIS X 0208) |
| `ZenMaruGothic-Regular.ttf` | Zen Maru Gothic | The Zen Maru Gothic Authors | Zen | SIL OFL 1.1 | github.com/googlefonts/zen-marugothic / Google Fonts (static Regular, subset to JIS X 0208) |
| `KleeOne-Regular.ttf` | Klee One | The Klee Project Authors (Fontworks) | Klee | SIL OFL 1.1 | github.com/fontworks-fonts/Klee / Google Fonts (static Regular, subset to JIS X 0208) |
| `GowunBatang-Regular.ttf` | Gowun Batang | The Gowun Batang Project Authors (Yanghee Ryu) | — | SIL OFL 1.1 | github.com/yangheeryu/Gowun-Batang / Google Fonts (static Regular, subset to KS X 1001) |
| `Bitter-Bold.ttf` | Bitter | The Bitter Project Authors | — | SIL OFL 1.1 | github.com/solmatas/BitterPro (variable, instanced `wght=700`) |
| `EBGaramond-Bold.ttf` | EB Garamond | The EB Garamond Project Authors | — | SIL OFL 1.1 | github.com/octaviopardo/EBGaramond12 (variable, instanced `wght=700`) |
| `Figtree-Bold.ttf` | Figtree | The Figtree Project Authors | — | SIL OFL 1.1 | github.com/google/fonts (ofl/figtree, variable, instanced `wght=700`) |
| `FiraSans-Bold.ttf` | Fira Sans | The Mozilla Foundation and Telefonica S.A. | — | SIL OFL 1.1 | github.com/google/fonts (ofl/firasans, static Bold) |
| `Fraunces9pt-Bold.ttf` | Fraunces 9pt | The Fraunces Project Authors | — | SIL OFL 1.1 | github.com/undercasetype/Fraunces (variable, instanced `wght=700 opsz=9 SOFT=0 WONK=0`) |
| `iAWriterQuattroS-Bold.ttf` | iA Writer Quattro S | IBM Corp. and iA Inc. | — | SIL OFL 1.1 (upstream — not embedded, see note ②) | github.com/iaolo/iA-Fonts (static Bold) |
| `IBMPlexSans-Bold.ttf` | IBM Plex Sans | IBM Corp. | — | SIL OFL 1.1 | github.com/IBM/plex (static Bold) |
| `Literata-Bold.ttf` | Literata | The Literata Project Authors | — | SIL OFL 1.1 | github.com/googlefonts/literata (variable, instanced `wght=700 opsz=12`) |
| `Newsreader-Bold.ttf` | Newsreader | The Newsreader Project Authors | — | SIL OFL 1.1 | github.com/productiontype/Newsreader (variable, instanced `wght=700 opsz=16`) |
| `ZillaSlab-Bold.ttf` | Zilla Slab | The Mozilla Foundation | Zilla (trademark) | SIL OFL 1.1 | github.com/typotheque/zilla-slab (static Bold) |
| `IBMPlexMono-Bold.ttf` | IBM Plex Mono | IBM Corp. | IBM Plex (trademark) | SIL OFL 1.1 | github.com/IBM/plex (Google Fonts ofl/ibmplexmono, static Bold) |
| `JetBrainsMono-Bold.ttf` | JetBrains Mono | The JetBrains Mono Project Authors | JetBrains Mono (trademark) | SIL OFL 1.1 | github.com/JetBrains/JetBrainsMono (Google Fonts ofl/jetbrainsmono, variable, instanced `wght=700`) |
| `Iosevka-Bold.ttf` | Iosevka | Renzhi Li (aka. Belleve Invis) | — | SIL OFL 1.1 | github.com/be5invis/Iosevka (PkgTTF-Iosevka, static Bold) |
| `MonaspaceXenon-Bold.ttf` | Monaspace Xenon | GitHub, Inc. (copyright in name ID 7) | Monaspace, Monaspace Xenon (+ Argon/Neon/Radon/Krypton) | SIL OFL 1.1 | github.com/githubnext/monaspace (variable, instanced `wght=700 wdth=100 slnt=0`) |
| `ArchivoBlack-Regular.ttf` | Archivo Black | The Archivo Black Project Authors | — | SIL OFL 1.1 | github.com/Omnibus-Type/ArchivoBlack (Google Fonts ofl/archivoblack, static Black, subset to Latin + punctuation) |
| `AbrilFatface-Regular.ttf` | Abril Fatface | TypeTogether (www.type-together.com) | Abril, Abril Fatface | SIL OFL 1.1 | github.com/google/fonts (ofl/abrilfatface, static Regular, subset to Latin + punctuation) |
| `SourGummy-Regular.ttf` | Sour Gummy | The Sour Gummy Project Authors | — | SIL OFL 1.1 | github.com/eifetx/Sour-Gummy-Fonts (Google Fonts ofl/sourgummy, variable `wdth,wght`, instanced `wght=400 wdth=100`, Latin+punctuation subset) |
| `SourGummy-Bold.ttf` | Sour Gummy | The Sour Gummy Project Authors | — | SIL OFL 1.1 | github.com/eifetx/Sour-Gummy-Fonts (instanced `wght=700 wdth=100`, same subset — item 70's real Bold companion) |
| `SourGummy-Black.ttf` | Sour Gummy | The Sour Gummy Project Authors | — | SIL OFL 1.1 | github.com/eifetx/Sour-Gummy-Fonts (instanced `wght=900 wdth=100`, same subset — item 70's bundled 900 heavy candidate, see below) |
| `AwlMarks.ttf` | Awl Marks | EB Garamond Project Authors; Noto Sans Symbols 2 source Copyright 2017 Google Inc., distributed by The Noto Project Authors; Iosevka (Renzhi Li); Junicode (Peter S. Baker) | — | SIL OFL 1.1 (composed from OFL sources — see note below) | github.com/octaviopardo/EBGaramond12; github.com/notofonts/noto-fonts (Noto Sans Symbols 2); github.com/be5invis/Iosevka; github.com/psb1558/Junicode-font |

**Item 132 (Kite, the twentieth world) added NO font asset.** Its display face is
the Fira Sans Regular + Bold pair above, which was already embedded, licensed and
attributed — Quokka's face until item 70 moved it to Sour Gummy, and the roster's
last registered-but-unassigned display family since. Both rows' license and
copyright holder are unchanged and still read from the files' own `name` tables;
no reserved font name is involved. Shipped weight, for the asset-size audit:
`FiraSans-Regular.ttf` 274,832 bytes, `FiraSans-Bold.ttf` 191,756 bytes — both
already counted in every previous release.

Most faces here are single-weight instances (Regular, except IBM Plex Mono which
ships Light/300); the CJK faces are subset to their target script's code-point
range (JIS X 0208 / GB 2312 / KS X 1001), and the text/ornament/chrome faces
(Fira Sans, Iosevka, Bitter, Junicode, Archivo Black, Abril Fatface) are subset
to their needed code-point ranges, as described in `CLAUDE.md`. Subsetting and single-weight instancing are
permitted modifications under the OFL (the fonts remain OFL, unsold by
themselves, reserved names untouched).

The **`*-Bold.ttf`** faces (Bitter, EB Garamond, Figtree, Fira Sans, Fraunces 9pt,
iA Writer Quattro S, IBM Plex Sans, Literata, Newsreader, Zilla Slab — plus the
four monospace display faces IBM Plex Mono, JetBrains Mono, Monaspace Xenon,
Iosevka) are the 700-weight companions to every bundled display face, so
`**bold**` renders as real bold instead of falling into cosmic-text's monospace
fallback (the `weight_diff == 0` trap — a Regular-only family drops the 700
request during fallback filtering and it lands in the ugly `.SF NS` / mono
substitute). Each was sourced exactly like the CJK faces: the static upstream
Bold where one ships (Fira Sans, IBM Plex Sans, Zilla Slab, iA Writer Quattro S,
IBM Plex Mono, Iosevka), else instanced from the OFL variable source at
`wght=700` (`fonttools varLib.instancer`, pinning the Regular's optical size —
Literata `opsz=12`, Newsreader `opsz=16`, Fraunces `opsz=9` — and, for Monaspace
Xenon, its width/slant axes to the Regular's `wdth=100 slnt=0`; JetBrains Mono
has a lone `wght` axis), then subset to the SAME code-point set as its Regular.
Each registers under the IDENTICAL family name its Regular uses (the `name`
table's family/subfamily forced to `<family>`/`Bold`, usWeightClass 700, the
bold `fsSelection`/`macStyle` bits set). IBM Plex Mono is the notable pair: awl
ships its Regular as the Light/300 weight (`mono_safe_weight`), but its Bold is
the genuine upstream 700, so a `**bold**` span jumps Light→Bold for clear
emphasis. `Fraunces9pt-Bold.ttf` covers 624 of its Regular's 637 code-points: 13
rare transliteration/combining marks (Ṅ Ṡ Ṧ Ṩ Ẏ + combining hook/ring-above,
dot-below) are absent from the upstream Fraunces variable source itself, so no
`wght=700` instance can carry them; every other bold matches its Regular's
coverage exactly.

The **`ArchivoBlack-Regular.ttf`** and **`AbrilFatface-Regular.ttf`** faces are
the CHROME-VOICES round's two curated overlay-CHROME voices — a distinctive face
for a world's summoned-overlay chrome only (placard wordmark / inline title
prefix / lens-strip labels), never a list row, the query line, or the writing
column. **Archivo Black** is the LOUD voice (Firetail's pick): a single heavy
Omnibus-Type display weight whose `OS/2.usWeightClass` is 400 — verified in the
downloaded file's own `name`/`OS/2` tables — so a plain `Weight::NORMAL` request
matches it (no `mono_safe_weight` exception, the opposite corner of the IBM Plex
Light trap). **Abril Fatface** is the REFINED voice: a high-contrast TypeTogether
Didone display Regular (usWeightClass 400) with embedded Reserved Font Names
"Abril" and "Abril Fatface" (preserved through the subset). Both were fetched
from `google/fonts` `ofl/`, their OFL-1.1 grant re-read from each file's own name
table (nameID 0 copyright + nameID 13 license, unchanged by the subset), and
subset with `pyftsubset` to Latin (U+0020–017F) + typographic/code punctuation.
Abril is registered but assigned to no world's data yet (gallery-only, pending
the user's veto pass); Firetail alone names Archivo Black on its `chrome_face`.

**`SourGummy-{Regular,Bold,Black}.ttf` provenance (item 70, Quokka's
printed-card round):** Sour Gummy is a bouncy, gummy-lettered display face —
Quokka's new Latin display face (`Theme::font`), replacing Fira Sans; IBM Plex
Mono (code) and the Klee One/LXGW WenKai CJK companions are unchanged. Fetched
from `google/fonts` `ofl/sourgummy` (upstream github.com/eifetx/Sour-Gummy-Fonts),
a variable font on two axes (`wdth` 100-125, `wght` 100-900), its OFL 1.1 grant
re-read from the downloaded file's own name table (nameID 0 copyright + nameID 13
license, unchanged by instancing/subsetting — never assumed). Following the
established instance/subset path (`fonttools varLib.instancer --update-name-table`
pinning `wdth=100`, then `pyftsubset`): THREE static instances at `wght=400`
(Regular, Quokka's prose face), `wght=700` (Bold, subfamily "Bold",
`usWeightClass 700`, the bold `fsSelection`/`macStyle` bits set — the REAL
`**bold**`/heading companion `FONT_THEME_BOLD_FACES` registers under the
IDENTICAL family name "Sour Gummy"), and `wght=900` (Black, subfamily "Black",
`usWeightClass 900` — the round's REQUIRED second heavy candidate, a genuine
instance at that weight, not relabelled 700 metadata). All three share the exact
same 335-codepoint subset — Latin (U+0020-017F) + a fuller typographic
punctuation set matching the OTHER bundled prose faces (Literata/Newsreader's
own coverage: modifier letters, dead-key combining diacritics, en/em dash,
curly + low-9 quotes, dagger/double-dagger, bullet, ellipsis, primes,
guillemets, fraction slash, euro, trademark, up/down arrows, minus, division
slash) — broader than the narrower ArchivoBlack/AbrilFatface CHROME-only
punctuation set above, since Sour Gummy shapes real document prose/headings,
not just overlay chrome. DOCUMENTED GAP (mirrors the `Fraunces9pt-Bold.ttf`
precedent): 21 of the requested codepoints are absent from the upstream Sour
Gummy source itself (currency ¤, soft hyphen, plus-minus, micro sign, a few
rare Latin Extended-A letters, modifier apostrophes, two rare combining marks,
thin/zero-width space, primes, up/down arrows) — no instance can carry them;
every other requested codepoint is present, identically, across all three
weights. Native+wasm size delta: +157,220 bytes (0.150 MB) — the three
~52 KB subset instances (a variable-font source with two axes, subset before
instancing would be smaller still, but the established path instances first
so each weight's own hinting/outlines stay faithful to that named instance).
Normal operation resolves `**bold**`/headings to the 700 file
(`weight_diff == 0`); the bundled 900 file stays addressable-but-unselected
unless the dev-only `AWL_SOURGUMMY_HEAVY_FORCE=900` knob (mirrors
`AWL_CJK_FORCE`'s "total no-op unless set" contract — see `render.rs`'s
`apply_sourgummy_heavy_force`) prunes the 700 face so the same request falls
through to the 900 file instead — the mechanism the round's 700-vs-900 taste
captures were shot through.

**DEVIATION (item 253, all three instances): the upstream `cmap` shipped both
raised quote pairs transposed, and the fix touches `cmap` only.** The `glyf`
outlines were never wrong — `quoteleft`/`quoteright` and
`quotedblleft`/`quotedblright` each carry one correct shape (the rotated
opening "6" and the raised-comma closing "9") — but upstream's `cmap` pointed
`U+2018`/`U+201C` (opening single/double) at the CLOSING "9" outline and
`U+2019`/`U+201D` (closing) at the mirrored opening "6", identically across
Regular/Bold/Black. This is not one of the instancing/subsetting steps
described above; it is a separate, independent repair, recorded here so a
future re-fetch of `ofl/sourgummy` does not silently reintroduce it. The fix
swaps ONLY the four `cmap` entries (`U+2018`↔`U+2019`, `U+201C`↔`U+201D`) in
every subtable that carries them, via `fonttools` (`fontTools.ttLib.TTFont`,
mutating each subtable's decompiled `cmap` dict once — subtables sharing a
dict object after decompile must be swapped only once — then clearing
`subtable.data` to force a recompile, since `cmap_format_4.compile()`
otherwise returns its cached raw bytes verbatim and silently ignores the
mutation). `glyf`, `hmtx`, `name`, `post`, `OS/2`, `GDEF`/`GPOS`/`GSUB`,
`STAT`, `gasp`, `prep`, `loca` and `maxp` are byte-identical before and after,
per-glyph, in all three files; `head` differs only in the mandatory whole-font
`checkSumAdjustment` (`modified` was pinned via `recalcTimestamp=False` to
avoid a gratuitous timestamp bump for a four-codepoint change). The remap is
metrically free — `U+2018`/`U+2019` share one advance and `U+201C`/`U+201D`
share another, within each weight (207/207 and 384/384 at Regular, 239/239
and 454/454 at Bold, 261/261 and 501/501 at Black) — because the advance
travels with the glyph, not the codepoint, so swapping which codepoint names
which glyph moves no line-breaking geometry. `U+201A`/`U+201E` (the low
single/double quotes, which sit correctly at the baseline sharing the comma's
own extents) are untouched. `render::tests::quote_orientation_item253` is the
permanent roster law over every bundled display face
(`render::bundled_display_faces`) asserting the orientation this note
describes, so the next face shipping this exact upstream defect fails on
arrival. The defect is upstream at
`github.com/eifetx/Sour-Gummy-Fonts` (via `google/fonts` `ofl/sourgummy`) and
is worth filing there; awl's zero-network invariant makes that a person's
action, not the app's.

**`AwlMarks.ttf` provenance (composed from OFL sources):** the rebuilt symbol /
ornament face is a hand-merged subset — decomposed glyph outlines copied from
four SIL-OFL faces (all UPM 1000, so metrics align) into one face renamed to the
private family "Awl Marks": the fleurons + reference marks (⌃ § † ‡ • ◦ ❧ ❦ ☙)
from **EB Garamond**, the remaining modifier keycaps + fleurons (⌘ ⌥ ⇧ ▪ ❡ ❥)
from **Noto Sans Symbols 2**, the ↵/⇥ key-hint keycaps from **Iosevka**, and the
asterism ⁂ from **Junicode**. Each source is OFL 1.1; the merged face carries an
honest multi-source copyright + OFL grant in its own `name` table (nameID 0/13).
It replaces the retired DejaVu/Bitstream-derived `AwlSymbols.ttf` (cmap parity
confirmed — the identical 18 codepoints), removing the app's only non-OFL asset.

The face was later EXPANDED (additive only — every prior glyph outline + metric
is byte-identical, a strict cmap superset) with 15 more symbols decomposed from
**Noto Sans Symbols 2** (the same OFL source, same UPM 1000, the same
decompose-and-merge operation), for the per-world section-break ornament re-pick
— stars (✦ ✧ ✶ ✷ ✴ ❂ ⭑ · U+2726 2727 2736 2737 2734 2742 2B51), florets
(✿ ❀ ❁ ✽ · U+273F 2740 2741 273D), and geometric marks (❖ ◈ ⬥ ◆ · U+2756 25C8
2B25 25C6). The family name ("Awl Marks") is unchanged, so the merged face now
carries 33 codepoints (the original 18 + these 15).

U+232B ERASE TO THE LEFT was then added from the official
**Noto Sans Symbols 2 Regular v2.003** artifact at
`notofonts/noto-fonts/hinted/ttf/NotoSansSymbols2/NotoSansSymbols2-Regular.ttf`.
The exact source artifact has SHA-256
`630846d528dbe4c4981370a4d0a9475a1fd1491a129bb411f8e157cdb5de13c6`;
its embedded name table records `Copyright 2017 Google Inc. All Rights Reserved.`
and the SIL OFL 1.1 grant. The repository's `LICENSE` has SHA-256
`0dab92d0544f7b233403f14b84a663bdbfa746982eda629e7f4f9ffe1b036feb`,
names The Noto Project Authors, and contains the full SIL OFL 1.1 text. The
simple outline was copied without scaling (both faces use UPM 1000), preserving
the source horizontal metrics: advance 1197, left side bearing 51. AwlMarks'
private family name is unchanged, and the face now carries 34 codepoints.

## license texts in this directory

- [`OFL.txt`](./OFL.txt) — the full SIL Open Font License 1.1, canonical
  text, applying to every OFL face above.
- `OFL-NotoSansJP.txt`, `OFL-NotoSansKR.txt`, `OFL-NotoSansSC.txt`,
  `OFL-NotoSerifJP.txt`, `OFL-NotoSerifSC.txt`, `OFL-LXGWWenKai.txt`,
  `OFL-GowunBatang.txt` — the per-foundry OFL copies as shipped upstream (kept
  verbatim; LXGW's carries an additional-permission clause).

**Where OFL §2's copyright notice actually lives, and why that satisfies it
(item 255).** OFL 1.1 §2 permits redistribution "provided that each copy
contains the above copyright notice and this license," and says explicitly
that these "can be included either as stand-alone text files, human-readable
headers or in the appropriate machine-readable metadata fields within text
or binary files as long as those fields can be easily viewed by the user."
awl ships the stand-alone-text-file form: `OFL.txt` above is the license
text, and this file's per-face table is the copyright notice, naming the
holder for every bundled face once per distributed package — not once per
font file. **That satisfies §2 on its own**, independent of what any single
`.ttf`'s own `name` table carries. A prior version of this sentence
conflated the two mechanisms, claiming the copyright notice's location WAS
each face's embedded `name` table, which put the compliance story on a
foundation this document had never actually re-checked against the shipped
files.

**A 2026-08-04 audit (item 255) found that foundation sound anyway, once
measured correctly.** A first pass reported only 7 of 45 bundled faces
carrying a copyright string, using `strings`/`strings -e b` over the raw
file bytes — but macOS's `strings` has no `-e` flag, so the UTF-16BE pass
silently produced nothing for every file, and 3 of the 7 ASCII hits it did
get were `post`-table glyph names like `copyright.sc` (a small-caps variant
name), not `name`-table records at all. A direct read of every face's
`name` table via `ttf_parser::Face::names()` — the same table `fontdb` and
`skrifa` decode at load time — found the opposite: **all 45 bundled `.ttf`
files carry a real copyright-bearing record**, nameID 0 for 43 of them and
nameID 7 for Monaspace Xenon's two instances (already flagged in the
per-face table above). Nothing needs restoring in the build pipeline.
[`render::tests::font_licence_item255`](../../src/render/tests/font_licence_item255.rs)
holds both facts permanently — every bundled `.ttf`, read from the directory
rather than a hand-kept list, has a `LICENSES.md` row AND a decodable
copyright/trademark record — so a face added without an entry, or a future
re-subset that strips the record, fails it by name instead of waiting for
another manual audit to (mis-)measure it.
