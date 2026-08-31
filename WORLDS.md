# WORLDS.md — the theme worlds, in plain flavour

This is the **human** reference for awl's theme worlds: what each one *feels* like,
where it sits on each picker axis, and which faces it wears. For the technical
*contract* — the measurable laws a world must satisfy and the tests that enforce
them — see **THEMES.md**. For the *why* of the ink/one-accent discipline, see
**DESIGN.md**.

**The anchor is the flavour sentence.** Each world gets one short sentence naming
its feel. That sentence is the north star: every other choice — ground colour,
display face, code mono, section-break ornament, where it lands on Time /
Register / Voice / Temperature — only has to *make sense when read against the
sentence*. If a choice fights the sentence, the choice is wrong (or the sentence
is). Cohesion is "does this all agree with one line of prose," nothing fancier.

---

## The worlds at a glance

| World          | Ground                      | Margin background | Display             | Mono            | Ornament (`---`/`***`/`___`)                  | Time  | Register | Voice     | Temp    |
| -------------- | --------------------------- | ----------------- | ------------------- | --------------- | --------------------------------------------- | ----- | -------- | --------- | ------- |
| **Gumtree**    | pale eucalyptus-green       | Zigzag             | Literata            | Monaspace Xenon | Riverbank · long snake / fish / snail          | Day   | —        | Literary  | Cool    |
| **Bilby**      | palest rose-gold dawn       | Gradient           | Newsreader          | Monaspace Xenon | Hanami · cherry blossom / blossom / tulip      | Dawn  | Refined  | —         | —       |
| **Magpie**     | paper-white, high-contrast  | Pinstripe          | Bitter              | Monaspace Xenon | Asterism · ⁂ ⁑ ✱                               | Day   | —        | Literary  | Neutral |
| **Saltpan**    | warm ecru salt-flat         | Pinstripe          | Fraunces            | Monaspace Xenon | Scriptorium · coronis / dotted diple / ancora  | Dawn  | Refined  | —         | —       |
| **Quokka**     | warm peach reef             | Zigzag             | Sour Gummy          | IBM Plex Mono   | Tavern · acorns / bells / leaves               | Dawn  | Everyday | Modern    | Warm    |
| **Galah**      | dusty-pink                  | Gradient           | Figtree             | IBM Plex Mono   | Cardtable · ♠ ♥ ♣                              | Dawn  | —        | Modern    | Warm    |
| **Potoroo**    | dark burnt-orange           | Stripes            | Monaspace Xenon     | Monaspace Xenon | Undergrowth · mushroom / clover / shamrock     | Dusk  | Humble   | Technical | Warm    |
| **Mopoke**     | warm charcoal               | Dots               | Bitter              | IBM Plex Mono   | Moonfaces · full / first-quarter / new         | Dusk  | Humble   | —         | —       |
| **Bombora**    | dark violet                 | Waves              | EB Garamond         | Monaspace Xenon | Arabesque · joined white / joined black / scroll | Night | Refined  | Literary  | —       |
| **Mulga**      | blackish-olive              | Pinstripe          | Zilla Slab          | Monaspace Xenon | Genjikō · 1–5 / pattern / pattern              | —     | Everyday | —         | —       |
| **Bowerbird**  | midnight-navy               | Dots               | IBM Plex Sans       | JetBrains Mono  | Wish · shooting / glowing / sparkles           | Night | Everyday | Modern    | Cool    |
| **Brolga**     | pale sky-blue               | Gradient           | IBM Plex Sans       | IBM Plex Mono   | Dovecote · right / olive-branch / left dove    | Day   | —        | —         | Cool    |
| **Mangrove**   | dark tidal-teal             | Lava · dithered    | JetBrains Mono      | JetBrains Mono  | Spirals · round / angular / conical             | —     | —        | Technical | Cool    |
| **Tawny**      | warm-grey                   | Dots               | IBM Plex Mono       | IBM Plex Mono   | Autumn · maple / fluttering / fallen leaf      | —     | Humble   | —         | Neutral |
| **Currawong**  | near-pure-black OLED        | Gradient + stars   | Iosevka             | Iosevka         | Gambit · ♘ ♕ ♙                                 | Night | —        | Technical | Neutral |
| **Wagtail**    | near-black, zero-saturation | Gradient           | JetBrains Mono      | JetBrains Mono  | Songbook · ♩ ♪ ♬                               | Dusk  | —        | —         | —       |
| **Firetail**   | deep oxblood-charcoal       | Lava · smooth      | Monaspace Xenon     | Monaspace Xenon | Stars · ✦ ✶ ✧                                  | —     | —        | —         | Warm    |
| **Cassowary**  | near-black glass            | Pinstripe          | Iosevka             | Iosevka         | Splatter · splash / black / centerless         | Night | —        | Technical | —       |
| **Paperbark**  | palest cream, honey layers  | Deckle · strata    | EB Garamond         | Monaspace Xenon | Fleurons · ❧ ☙ ❦                               | Day   | Refined  | Literary  | —       |
| **Kite**       | near-white pale lavender    | Warped grid        | Fira Sans           | JetBrains Mono  | Solar · ☀ ☼ 🌞                                 | —     | —        | Modern    | —       |

*(20 worlds — the authored target. The names are Australian fauna, flora, and landscape — flavour, not taxonomy. Wagtail and Firetail are statement-world mirrors; Brolga is the cool light pole; Cassowary is the dark-technical statement, a NERV terminal; Paperbark is the material world, the only one whose ground is a handmade sheet; Kite closes the roster as Firetail's light counterpart, the only world whose ground is travelling — see below.)*

Kite leaves **Time**, **Register** and **Temperature** untagged on purpose. It is
plainly a cool, refined, daylit world, but the picker's Cool, Refined, Day, Dawn
and Night bands are all already at their curated maximum of four, so it headlines
the one lens where it reads clearest and stays out of the crowd — the same
curation rule Paperbark follows for Warm.

Paperbark leaves **Temperature** untagged on purpose. It is plainly a warm
world, but the picker's Warm band already carries its curated maximum of four
(Potoroo, Quokka, Galah, Firetail); a fifth entry would trade a curated band for
a crowd, so this world headlines Time / Register / Voice instead.

---

## The margin backgrounds

Page mode keeps the writing column flat and paints the world background only in
the margins. The pattern is structure, not content: it stays behind the page,
uses the world's own quiet palette, and never spends the caret accent.

| Background | What it draws | Shipping worlds |
| ---------- | ------------- | --------------- |
| **Gradient** | A directional colour blend with no built-in marks. | Bilby, Currawong, Brolga, Wagtail |
| **Dots** | A regular grid of small round dots over a gradient. All shipping dot worlds currently use the uniform form; the available page-edge proximity form is unassigned. | Mopoke, Tawny |
| **Pinstripe** | Fine parallel print/ledger lines over a gradient. | Saltpan, Cassowary, Mulga |
| **Stripes** | A diagonal striped band concentrated at the page boundary and dissolved outward into the margin. | Potoroo |
| **Lava** | A slow metaball field in the margins. Reduce Motion can still it; deterministic captures use a fixed phase. Firetail is smooth wine; Mangrove is dithered deep-sea blue. | Firetail, Mangrove |
| **Bands** | Exactly three large, tone-on-tone diagonal bands spanning the WHOLE margin field — cut-paper grass, not a repeating stripe-tile. Static; the ONLY colors are the world's own ground-ladder rungs. | Magpie |
| **Zigzag** | A TILED FIELD of repeating chevron ("V") rows over a gradient — a whisper mark like Dots/Pinstripe, not a final-color field like Bands/Waves. The chevron repeats both along its travel direction and across it, and consecutive rows abut by construction, so every part of a margin carries rows at any window size or shape. Four per-world dials (tooth wavelength, peak excursion — which also sets the row pitch — travel angle, an extra coverage multiplier) keep the two shipping worlds from reading as a recolor of one asset: Quokka is tight/steep/bold, Gumtree is broad/open/quiet. | Quokka, Gumtree |
| **Deckle** | Quasi-random CONTOUR LANES of handmade paper, seeded per lane and torn by a fixed two-tone wander so no lane is a ruled line. One theme-owned `weave` picks the profile: **Strata** indexes the lanes on DISTANCE FROM THE PAGE COLUMN, so the contours gather around the writing page and mirror across it, each lane filled at its own seeded tone with a torn tint on its boundary; **Fibres** indexes them on screen `y` and draws thin translucent strokes with seeded dropouts plus a sparser diagonal vein family. Three dials — lane pitch, wander amplitude, one coverage multiplier — and `density: 0.0` collapses either profile to a flat ground exactly, which is the differential oracle its pixel laws measure against. Entirely static. | Paperbark (Strata), Galah (Fibres) |
| **Warped grid** | A straight projected tube on ONE room-fixed axis — the room's own centre, so the vanishing point sits behind the page and each margin carries one flank. The page cannot rescale, flatten, bend or reposition the field; it veils it, and the tube's major rings cross the column faintly so the two flanks read as one scene. Rings travel steadily outward; every fifth line is stronger, the minor rung retires in narrow margins, and both lattices fade before the far end becomes unresolved. | Kite |
| **Waves** | Exactly three stacked, non-overlapping shallow wave tiers — wide scalloped crests, horizontally phase-offset so they layer instead of gridding. Static; the world's own ground-ladder rungs. | Bombora |

Currawong's base margin background is **Gradient**. Its slowly appearing and
dying stars are a separate ambient Frame layer (`AmbientStyle::Stars`), which
composes over any ground; the at-a-glance table writes “Gradient + stars” so the
visible result is not misleading. It is now the only star mechanism awl carries
— the separate static scattered-star GROUND that Bombora and later Mulga wore
was retired outright once its last world moved off it.

---

## Each world

> **Every world carries one distinct, story-led trio from the Nishiki cabinet.**
> The three slots follow markdown syntax order (`---` / `***` / `___`); the dash
> also closes the About card. A joined snake or arabesque pair is shaped as one
> run. All marks come from the roster-derived **Awl Marks** subset, requested at
> weight 500. List bullets keep their existing pairs pending their own fitting round.

### Gumtree
**A pale eucalyptus-green reading room, calm and cool in clear daylight.**
Literata's easygoing book-serif on cool green paper; Shippori Mincho for Japanese; Monaspace Xenon for code.
Its margins carry a broad, quiet eucalyptus zigzag field — shallow,
near-horizontal chevron rows in the ground's own ladder, about six of them down an ordinary
window, replacing the room's original grass-bands field.
Day · Refined · Literary · Cool.

### Bilby
**First light on the desert — the palest rose-gold page, the night's violet still in the ink.**
Newsreader's editorial serif on a pale pre-sunrise horizon: warm rose-gold ground planes,
a cool violet-grey ink ladder (dawn's complementary structure), one sunrise-gold caret,
and a 1px ink hairline framing the writing column — the light pole's page frame,
Wagtail's 2px white frame mirrored. The bilby is a dawn-active desert marsupial;
its world is dawn itself. Its ground planes are deliberately pale and less peach
(composition, caret, and ink untouched), so the room stays distinct from
Quokka's own peach reef. Shippori Mincho for
Japanese; Monaspace Xenon for code.
Dawn · Refined · Literary · Warm.

### Magpie
**A paper-white, high-contrast page — sharp black on white.**
Bitter's sharp, high-contrast slab on bright paper; Monaspace Xenon for code.
Day · Everyday · Literary · Neutral.

### Saltpan
**A warm ecru salt-flat at first light — old-style and airy.**
Fraunces' characterful old-style serif on warm sand; Monaspace Xenon for code.
Dawn · Refined · Literary · Warm.

### Quokka
**A sunlit peach reef — friendly, warm, modern, deliberately playful.**
Sour Gummy's bouncy display face on warm peach; Klee One for Japanese; IBM Plex Mono for code.
Its summoned cards are a printed-card statement — a crisp 45° chamfered
silhouette and a small rotated halftone-dot texture, strongest at the card's
right decorative edge and rolling off before the left content column. Its
margins carry a tight, bold repeating zigzag field — the same
ground as Gumtree's, dialled 2.5x tighter and much steeper, so roughly twice as many
chevron rows cross the margin — replacing the room's original dot grid.
Dawn · Everyday · Modern · Warm.

### Galah
**A dusty-pink reading room at dawn — warm and friendly.**
Figtree's soft humanist sans on rose; Zen Maru Gothic for Japanese; IBM Plex Mono for code.
Dawn · Everyday · Modern · Warm.

### Potoroo
**A burnt-orange burrow at dusk — warm, dim, all monospace.**
Monaspace Xenon as both page and code face; a rust-dark room.
Dusk · Humble · Technical · Warm.

### Firetail
**A deep oxblood-charcoal lamp — wine-dark ground with one ember-gold spark.**
Monaspace Xenon as both page and code face; smooth wine lava drifts in the margins, redder than violet Bombora and deliberately clear of Potoroo's orange-rust den.
Warm.

### Mopoke
**A cosy warm-charcoal room after dark — utilitarian and soft.**
iA Writer Quattro S (duospaced) on warm charcoal; Klee One for Japanese; IBM Plex Mono for code.
Dusk · Humble · Modern · Warm.

### Bombora
**A violet-dark swell over a submerged reef — classical and literary.**
EB Garamond's Renaissance serif on deep violet; Shippori Mincho for Japanese; Monaspace Xenon for code.
Night · Refined · Literary · Cool.

### Mulga
**A blackish-olive night in the arid acacia scrub — slab-sturdy.**
Zilla Slab on dark olive; Monaspace Xenon for code.
Night · Everyday · Literary · Cool.

### Bowerbird
**A glossy blue-black bower — crisp and technical.**
IBM Plex Sans on midnight navy; Zen Maru Gothic for Japanese; the crisp JetBrains Mono for code.
Night · Everyday · Modern · Cool.

### Mangrove
**A dark tidal-teal den — cool and rooted.**
JetBrains Mono as both page and code face; a teal-dark room.
Night · Humble · Technical · Cool.

### Tawny
**A warm-grey nocturne — plain and neutral as a frogmouth.**
IBM Plex Mono as both page and code face; near-neutral warm grey.
Night · Humble · Technical · Neutral.

### Currawong
**Near-pure-black OLED — stark, true, a coder's den.**
Iosevka as both page and code face; narrow, mechanical, true-black ground.
Night · Humble · Technical · Neutral.

### Wagtail
**A near-black room with zero saturation anywhere — the caret included.**
JetBrains Mono as both page and code face; a plain grey ladder, top to bottom.
Wagtail is awl's ONE deliberate exception to "one warm thing" (`DESIGN.md`
§3's logged amendment) — every other world keeps an amber caret; this one
keeps none. The caret's identity rides on VALUE alone (pure white — the
brightest thing in the room, by construction) and MOTION (the spring juice
is still its and only its own) instead of hue. Named for the Willie
Wagtail, a fearless black-and-white bird that's active at dawn and dusk —
Dusk.

### Brolga
**A clear cool sky after rain — pale sky-blue, washed clean, one red-crown spark.**
IBM Plex Sans on a pale periwinkle sky-blue page, a deep cool slate-navy ink; Noto Sans JP for Japanese; the humanist IBM Plex Mono for code.
Brolga is the cool light pole: a clean
cool SANS on blue where Gumtree (the only other cool light world) is a green
SERIF, so it reads as its own thing, not Gumtree's sibling. The brolga is a tall
grey-blue wetland crane with a vivid red crown; its world is the pale blue of a
clear sky reflected in still shallow water, and its one warm living thing is the
crane's red crown — a coral-vermilion caret.
Day · Cool.

### Paperbark
**A sheet of handmade paper in a daylit studio — deckled cream layers gathered around the page, bark-brown ink, one vermilion mark.**
EB Garamond's Renaissance serif on the palest cream; Shippori Mincho for Japanese; Monaspace Xenon for code.
Its margins are the world: nested deckled contours in cream and pale honey, laid *around* the writing page rather than behind it — the field is a function of the distance to the page edge, so the layers mirror across the column and gather toward it as you widen the window. Each lane takes its own seeded tone and its boundary carries a torn deckle tint, so the sheet reads as pressed layers rather than a ruled pattern. Static: nothing moves, no raking light, no second material mode. The writing page itself stays flat and opaque; the one accent is a coral-vermilion caret. The paperbark is a eucalypt whose trunk sheds in pale papery layers you can peel and write on — the world is that bark read as a sheet.
Day · Refined · Literary.

### Kite
**A near-white mineral page gliding through a cool straight-grid tunnel — indigo geometry in the margins, one vermilion eye.**
Fira Sans's screen-engineered humanist sans on pale lavender; Noto Sans JP for Japanese; JetBrains Mono for code.
Kite is the deliberate LIGHT counterpart to Firetail: cool rather than warm, geometric rather than organic, crisp rather than liquid, directional rather than drifting. One straight indigo-and-graphite tube at a fixed room scale fills the room, its rings travelling continuously toward the reader and its vanishing point hidden behind the writing itself; the margins carry its two flanks at full strength and the page carries the major rings alone, veiled, so the arcs stay continuous across the column. The chrome answers Firetail corner for corner: a small Figtree wordmark bottom-right against Firetail's bottom-left poster, a top-right card against its top-left, a hairline page frame and a filled facet band. Losing focus pauses it in place and resumes without catching up; ambient motion off and Reduce Motion freeze it to one composed still. The single accent is a hot vermilion caret — the kite's red eye.
Modern.

### Cassowary
**A NERV operations terminal after dark — green phosphor data on black glass, a lit block cursor in that same phosphor, red only when something is wrong.**
Iosevka as both page and code face, the narrow mechanical terminal-readout font; Noto Sans JP for Japanese; the summoned command overlay goes loud in Archivo Black. Cassowary is the dark-technical statement world (an Evangelion wink). Where every other chromatic world spends its one accent on an amber caret, Cassowary spends it on the terminal's own phosphor GREEN: the caret is the ink's own colour, drawn as an authentic CRT block cursor — a lit green cell with the letter under it knocked out in the black-glass ground. Red is held back for the alert channel alone (the spell-squiggle, a warning-crimson selection). The writing page stays a calm green terminal; the drama is transient, appearing only when you summon a command. The cassowary is a glossy-black, red-wattled, blue-green-necked living dinosaur — the black-ground / green-data / red-warning palette is the bird's own colouring.
Night · Technical.

---

## The fonts we ship

One line of flavour each. (All bundled, all OFL — the Awl Marks symbol set is
composed from OFL sources too; full attribution in `assets/fonts/LICENSES.md`.)

**Weights.** Every face ships **Regular (400)**; the 10 proportional display
faces *also* ship a **Bold (700)** companion (instanced + subset from the same
OFL sources) so inline `**bold**` renders as real bold in the world's own face —
not the system-mono fallback it used to trip. The monospace faces stay
Regular-only (code rarely bolds); *italic* is synthesized (a slant of the
Regular) on every face; and headings deliberately use size, not weight.

### Display serifs
- **Literata** — a warm, faintly bookish reading serif drawn for long-form screen text (Google's e-book face).
- **Newsreader** — a lively editorial serif with old-style warmth, built for reading on screen.
- **Fraunces** — a characterful "old-style" display serif with soft-serif wobble and literary swagger.
- **EB Garamond** — a faithful revival of Claude Garamond's Renaissance serif: classical, elegant, and (uniquely here) carrying real fleurons. *(Worn at both value poles — dark Bombora's violet and light Paperbark's cream.)*
- **Zilla Slab** — Mozilla's sturdy, friendly slab-serif; utilitarian with a bit of shoulder. *(Now Mulga's alone.)*
- **Bitter** — a sharp, higher-contrast screen slab: crisper and more incisive than Zilla, cut for high-contrast pages.

### Display sans
- **IBM Plex Sans** — IBM's neutral humanist workhorse: clear, unfussy, corporate-calm. *(awl's cool-sans face, worn at both value poles — dark Bowerbird's midnight navy and light Brolga's pale sky.)*
- **Fira Sans** — Mozilla's screen-engineered humanist sans: low-contrast, upright, and cut for interface legibility rather than warmth on the page. *(Kite's display face — the roster's last registered-but-unassigned face, and the one whose `l` is a bare stem.)*
- **Sour Gummy** — a bouncy, gummy-lettered display face with real playful character. *(Quokka's own pick — its printed-card identity: a chamfered card silhouette + a rotated halftone-dot texture pair with the face.)*
- **Figtree** — a soft, rounded geometric sans with a friendly contemporary warmth.
- **iA Writer Quattro S** — a duospaced writing face (proportional look, monospace rhythm) tuned for calm drafting.

### Monospace (code)
- **Monaspace Xenon** — GitHub's slab-serif monospace: a code grid with literary, typewriter warmth.
- **IBM Plex Mono** — the monospace kin of Plex Sans: warm, humanist, easy on the eyes.
- **JetBrains Mono** — a crisp, tall coding monospace engineered for long editor hours.
- **Iosevka** — a narrow, mechanical, characterful coding mono: tight and precise, a literal coder's face.

### CJK (per-script, per-world)
- **Noto Serif JP** — Japanese mincho (serif): brushed and formal, for a literary Japanese page — the neutral floor for the display-serif worlds that keep it (Saltpan, Mulga, Magpie).
- **Shippori Mincho** — a bookish, characterful Japanese mincho: the per-world pick for awl's true book-serif worlds (Gumtree, Bilby, Bombora, Paperbark).
- **Noto Sans JP** — Japanese gothic (sans): even, modern, clean kana and kanji — the neutral floor for the mono/sans worlds that keep it (Potoroo, Tawny, Currawong, Mangrove, Firetail, Brolga).
- **Zen Maru Gothic** — a rounded, warm Japanese gothic: the per-world pick for awl's rounded humanist-sans worlds (Galah, Bowerbird).
- **Klee One** — a brush kaisho Japanese face with real calligraphic character: the per-world pick for the two Klee worlds (Mopoke, Quokka), pairing with LXGW WenKai's matching Chinese brush.
- **Noto Serif SC** — Simplified-Chinese Song/serif: the classic printed-book hanzi shape.
- **Noto Sans SC** — Simplified-Chinese Hei/sans: even geometric strokes, screen-clean.
- **Noto Sans KR** — Korean gothic (sans): clean modern Hangul, one face for every world.
- **LXGW WenKai** — a calligraphic Klee-style Chinese face: tapered brush strokes with real character.

### Symbols
- **Awl Marks** — awl's private, roster-derived subset of Nishiki-teki. It carries the chrome marks, the exact 64-codepoint world-ornament union, and the reserved reference ladder without exposing the upstream face as document fallback. See `assets/fonts/AwlMarks.roster.tsv` and `assets/fonts/LICENSES.md`.

### Ornament face
Every section-break trio uses **Awl Marks**, the Nishiki-derived subset. The
world table above is the assignment roster. EB Garamond and Junicode remain
temporarily registered only for existing list bullets; the bullet-pair fitting
round will decide those replacements rather than inferring them here.
