# Geometry, chrome, and semantic fields

Read for detailed field semantics. The checked schema header and field summary remain in CAPTURE.md.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

Field order is stable; consumers may parse positionally or by key.

**PATHS ARE HOME-RELATIVE.** Before a sidecar reaches disk the builder's `$HOME`
is rewritten to `~` throughout the artifact — `project.root`, `project.workspace`,
`project.default_folder`, `overlay.browse_dir`, `buffers.active`, `images[].path`
and the document text alike. A consumer that wants a real path expands `~` itself;
nothing else about the path changes, so the sidecar still says exactly which
directory it listed. This is not cosmetic: the repo is public, `/gallery` is
ignored precisely because captures are not meant to be tracked, and a lane does not
have to WRITE a path to publish one — it only has to capture. The rewrite is one
call in `capture::sidecar::write_sidecar`, the single writer every capture door
funnels through, so no block can opt out of it (`capture::redact`; laws in
`capture::tests::redact_law` and `run::tests::launch_context`). It does NOT reach
CONTENT read off the real filesystem: `overlay.items` for a picker pointed at a
real directory is that directory's own entry names, and the PNG beside it
photographs the same rows — take those against a seeded `--root` and an explicit
`--config`.

Schema `/203` adds **`search.panel`** — the summoned find/replace card's PLANNED
geometry, or `null` while the panel is down. **`/212` changed its shape**: the
bordered-chrome round (clear find/replace fields, a separate match/navigation
region, distinct `Replace`/`Replace all` controls) replaced the old terse `Aa`
toggle and key-hint line with real bordered controls, so the one `case_toggle`
span became a roster of named boxes.

| key | shape | what it is |
| --- | --- | --- |
| `card` | `{ x, y, w, h }` | the card's exterior rect, the one the float primitive rims and the one a press is accepted inside |
| `text` | `{ left, top }` | the ink origin inside it — where the shaped panel text was uploaded |
| `rows` | `[{ row, top, h }]` | one band per SHAPED row of the card: `0` the find field; `1` the replace field, once revealed; then the nav row (counter, step buttons, `Match case`); then, once replace is revealed, the actions row (`Replace` / `Replace all`) |
| `controls` | `[{ name, x, y, w, h }]` | every drawn field/button/checkbox box this frame, `name` one of `find_field`, `replace_field`, `nav_prev`, `nav_next`, `case_toggle`, `replace_button`, `replace_all_button` — present only for the ones the current row plan actually shaped (a plain find panel publishes no `replace_field`/`replace_button`/`replace_all_button`), each a CLICK TARGET seated on its own shaped glyphs, never a hardcoded pitch |

`row` indexes the CARD's own shaped lines, never a document row, and the bands are
the ones the pointer inverts: a press at a band's centre resolves to that field,
which is what makes the report gradeable against the hit-test rather than only
against itself. Which field is FOCUSED is deliberately absent — `search.editing_replacement`
already reports it, and a block whose purpose is making drawn-versus-published
agreement assertable must not ship two answers to one question.

The reported geometry is in physical pixels. The planner converts the logical
`PANEL_MARGIN` and `PANEL_PAD` through its panel metrics, and obtains row pitch
and controls from shaped text. Read the reported geometry at the requested DPI
and zoom; do not infer it from an older capture's constants.

The card is seated from the window's right edge and responds before that seating
would push it through the opposite edge: its value fields yield cells, the nav/
actions rows drop their trailing chord annotations, its exterior is capped to the
canvas between physical margins, and `x` is clamped. `search.panel` reports that
same responsive card, including every shaped row and control; it does not project
or repair the geometry after the frame draws.

**Why it is a geometry oracle and not a convenience.** Every figure is read off the
same `panel_layout` the draw sizes the card from and the pointer inverts, plus the
panel's one row-band owner (`render/plan/panel_report.rs`). Before it, the only law
asking a geometry question about this card — "can a long query widen it?" — answered
by walking the PNG inward from the window's right edge with a colour-distance
threshold, which cannot tell a card that moved from a rim that changed tone.

Schema `/202` adds three keys to each **`overlay.window.rows[]`** entry — the
row's ACCESSORY CLUSTER, reported part by part:

| key | shape | what it is |
| --- | --- | --- |
| `label` | `{ x, w }` | the row's NAME ink, seated where the frame drew it |
| `value` | `{ x, w }` | the row's accessory VALUE ink — a shortcut chord, a git tag, a setting's readout |
| `rail` | `{ x, w, hit_x, hit_w }` | a Range row's slider: the drawn track, then the more generous band a pointer is accepted in |

Each is `null` when the frame drew nothing there — a display line with no value, a
row that is not a Range, or a card that yielded its **whole** accessory column
because the row names and the value column together outgrew the text width.

⚠️ **`null` is a measurement, not a gap in the report.** A narrowing card drops
the value text and the rail *together*, so the width at which they turn `null` is
the width at which the accessory column was yielded — and the `label` lane's own
width on that same frame is what the names took instead. Those are the three
numbers the width-budget question needs. A `value` of width 0 would have said none
of them, which is why an undrawn lane is absent rather than empty.

The rail's two spans are separate because they differ on purpose: the drawn track
is what a reader sees, the hit band is what a press lands in, and the second is
wider so the control stays reachable. Both come off the one rail object the draw
path and the pointer hit-test already share, so a rail is clickable exactly where
it is drawn and the report cannot claim otherwise.

Schema `/201` adds **`overlay.window.band`** and **`overlay.window.rows`** — the
PLANNED geometry of a summoned picker's candidate band, one rect per display line.

`band` is `{ x, w, first_top, pitch, footer_top }`: the content band every row is
stepped in from, the band's first row top and row pitch, and the y where the
candidate area ends and the foot hint begins.

⚠️ **Neither `band` nor `rows` reports which row is selected, deliberately.**
`overlay.window.sel_row` already does, resolved through the owner that also colours
the band — so a second answer here could only be the plan's LOGICAL row, which is a
different fact (the line `Enter` activates) and disagrees with the drawn one for the
length of every selection move. Ask `sel_row` for the selection and these rects for
the geometry; the whole point of the block is that drawn-versus-published agreement
is assertable rather than ambiguous.

`rows` is in draw order, one entry per PLANNED DISPLAY LINE — never one per corpus
item, so a 40,000-row picker reports the dozen rows on screen:
`{ display, item, x, y, w, h }`. `item` indexes `overlay.items`, and is
`null` for a display line that carries no selectable item (the faceted card's
section headings and its secondary location line). `x .. x + w` is the row's own
INCLUSIVE pointer span, which on a staggered composition is narrower than `band`.

**These are PHYSICAL pixels**, the space the rest of the overlay geometry family
already speaks: the same space a pointer arrives in, the same space `card_h` /
`canvas_h` / `layout.rows[].top` are in, and the same space the PNG is in. A `WxH`
capture at `--capture-dpi N` is a `(W/N)x(H/N)` logical window, so every figure
here scales with N — a `y` that does NOT change between `--capture-dpi 1` and `2`
is the device-pixel bug this block exists to expose.

**Why it is a geometry oracle and not a convenience.** Every number is read off
the single scene plan (`render/plan/`) that the draw emitters and the pointer
hit-test read — `x` and `w` come from the plan's one row x-span accessor, the same
call `overlay_row_at` inverts. So the drawn rect, the clickable rect and the
reported rect are one object read three times, and a test can assert row geometry
without inferring anything from pixels. It remains a state oracle: a perfectly
reported row can still be drawn invisibly, which is an appearance claim and still
belongs to arithmetic over the PNG.

Schema `/200` adds the top-level **`notice`** block — `{ text, kind }` for the
calm notice on screen, or `null`. It exists because no capture door could see the
notice channel at all: `CaptureOpts` carried no slot for it, so a driven editor
that had genuinely raised `saved` produced a PNG **byte-identical** to one that
had raised nothing, while the same sidecar's `semantic` block (which reads the
`App` directly) announced that notice to a screen reader. `text` is the sentence
**exactly as drawn** — elided to the writing column's own budget when the column
is too narrow to hold it whole — on the same "as drawn" convention the `gutter`
block keeps. `kind` is `"toast"` (self-clearing) or `"sticky"` (held until its
owner clears it); it is a LIFETIME, not a severity, and it is what a reader needs
to know whether a notice's absence means it expired or was never raised. Both
drivers can report it: a live `App` off its frame state, an ordinary `--keys`
replay off the notice its own effect interpreter latches.

Schema `/198` adds **`unit`** to the `readout` block and the `hud` block —
`"words"` or `"characters"`, alongside the count each block already carried.
Japanese and Chinese prose has no inter-word spaces, so the old whitespace
tokenizer saw a whole unspaced CJK document as ONE token; the readout now
counts an ideograph (Kana/Han/Bopomofo — Hangul spaces its own words and is
untouched) as a token of its own, and `unit` names which kind of token the
number beside it is. `"words"` for a script that spaces its words, else
`"characters"` — decided by a strict majority of the manuscript's own
characters that carry an unspaced script (a tie reads `"words"`); the
frontmatter `lang:` tag is deliberately NOT consulted for this — it is a
declared intent, not a report of what is actually written, and a body of
plain English tagged `lang: ja` must not read "characters". `unit` is `null`
exactly when `words`/`reading_min` are (a non-markdown or wordless buffer).
One owner, `crate::card::figures::{readout_figures, CountUnit}`, feeds both
blocks and the drawn HUD card's WORD COUNT row, so a capture's `readout.unit`
and `hud.unit` always agree with each other and with what the row draws.

Schema `/205` adds **`hud.selection`**. It is `null` for no selection or a
caret-only selection; otherwise it is `{ words, characters }` for the raw
selected buffer text. `characters` counts extended grapheme clusters, not
UTF-8 bytes or Unicode scalars. awl normalizes disk line endings into logical
`\n` internally, and a selected logical line break counts as one character, so
an LF file and a CRLF file report the same reader-facing selection count.
Concealed Markdown, folded lines and History previews do not change it: the
selection group is about the buffer bytes the writer selected, not the text a
particular frame happens to shape.

Vocabulary note (**no schema bump**; the shape is unchanged, only
the existing `reading_min` field gains a corrected VALUE, per `capture.rs`'s
"bump once per sidecar-SHAPE change" criterion): `/198`'s own landing left
`reading_min` computed at 200 units/minute for BOTH `unit`s — a WORDS-per-minute
figure applied unchanged to a CHARACTERS count, so a 5,500-character Japanese
manuscript read `28 min`, two to three times slower than published Japanese
silent-reading rates (roughly 400-600 characters/minute). `reading_min` is now
paced by the SAME `CountUnit` the block already carries
(`CountUnit::pace_per_minute`): 200 wpm for `"words"`, 500 cpm for
`"characters"` — the same 5,500-character document now reads `11 min`. A mixed
document takes its dominant script's pace outright, matching how `unit` itself
already resolves, rather than interpolating a blended rate.

Schema `/195` adds **`overlay.context_anchor`**, the physical-pixel click
anchor `[x, y]` for the awl-rendered contextual menu, or `null` for every
other summoned surface. The rendered `overlay.window` remains the clamped
appearance geometry; the anchor is the input/state oracle.

Schema `/197` adds two fields for the EXTERNAL-CHANGE CONFLICT surface:
**`gutter.changed`** and **`overlay.preview_view`**.

`gutter.changed` is the persistent `changed elsewhere` affordance beside the
filename — `true` exactly while the captured document is holding an unresolved
external change. It is chrome, not a notice, and that is the point: there is one
notice slot and a toast expiry clears it, so an unrelated "copied" can take the
conflict's line and leave nothing behind it. This field is true for as long as
the conflict is. Only a `driver: "live-app"` capture can ever set it — the
conflict is latched on the live `App`'s per-buffer disk baseline, and an
ordinary replay structurally has none (`run::CaptureSubject::changed_elsewhere`),
so every `"replay"` sidecar reports `false` and every pre-existing capture stays
byte-identical. Reaching the state at all needs `--seed-data` (see
`docs/harness-reach.md`); the affordance's APPEARANCE — that it reads stronger
than the filename beneath it — is asserted over the PNG's pixels, never from
this field.

`overlay.preview_view` names WHICH read-only view the comparison region is
showing: `"diff"`, `"mine"` or `"theirs"` (`overlay::ComparisonView::tag`), or
`null` when there is no comparison. It sits beside `preview_id`, which names the
SUBJECT — a restore id on a timeline, the conflicted file's path on a conflict
workspace. Two keys because they are two facts, and because the view is the only
one that tells three previews of ONE subject apart: a conflict workspace offers
Differences / Your version / Version on disk of the same file, and a sidecar
carrying the subject alone could not say which was on screen.

Schema `/196` adds the top-level **`semantic`** field, immediately after
`driver`: the exact renderer-independent semantic tree an assistive technology
is given, or `null`.

It is `null` for every `"replay"` capture, and an object for a `"live-app"`
one — the tree is a live-`App` fold (`App::semantic_snapshot`), and the shared
core has no surfaces, focus owner or action roster to fold. It is the SAME
value `awl --semantic-json` prints and the SAME value the native AccessKit
adapter projects, so a sidecar assertion about accessibility is an assertion
about what a screen reader actually receives, not about a model of it
(`semantic::native::tests::json_and_accesskit_are_projections_of_the_same_snapshot`).

The object is `{ schema, root_id, focus_id, nodes }`. `schema` is its own
version string, **`awl-semantic/3`** (the document is split into line
runs — see below), independent of the capture schema, because the semantic tree
is consumed on its own by `--semantic-json` as well. `focus_id`
names the ONE node with `focused: true`; that is an invariant, not a
convention, and it holds no matter how many passive surfaces are up. Each node
in `nodes` carries `id`, `role`, `name`, and only the properties that apply:
`value`, `description`, `children`, `controls`, `actions`, `selection`
(`{anchor, focus}`, in GRAPHEME offsets — never chars, never bytes),
`character_lengths`, `focusable`, `focused`, `editable`, `multiline`,
`selected`, `checked`, `expanded`. Absent means "does not apply"; the writer
omits empty and false-valued keys rather than spelling them out.

Ids are stable across edits AND across filtering: a picker row is keyed by its
CORPUS position, so typing a query narrows the visible rows without renaming
the survivors.

**The document is a sequence of LINE RUNS** (`awl-semantic/3`), not
one node holding the whole rope. The document node's `children` are
`document.run.<id>` nodes in reading order, one per line, each carrying that
line's text as its `value` — INCLUDING its trailing newline, so the runs
concatenate to the document byte for byte and the document node's
grapheme-offset `selection` is the sum of the run counts before it plus the
offset within. The ids come from `crate::semantic::runs::RunTable` and are
stable in the same sense a picker row's is: a run keeps its id through edits to
its own line and to every other, so a screen reader's cursor does not move when
a line is inserted above it. That is what lets the live adapter publish CHANGED
nodes after activation instead of the whole tree on every redraw. A sidecar
assertion about document text therefore reads the runs, not a single
`document.text` node — which no longer exists.

**Every passive surface a live-`App` capture DRAWS, it also announces.** The
summoned cards (About, Lifetime, Streaks, the stats HUD, the shortcut peek),
the which-key panel and the rendered menu bar each carry a node whenever the
PNG carries their pixels, and none when it does not. The cards' three document
figures — word count, frontmatter language, through-doc percent — are derived
by `crate::card::figures`, a pure owner over the document text that the
renderer and the fold both read, so the announced card is the drawn card rather
than a second description of it. Over the USER'S DOCUMENT, specifically: a
collapsed fold and a History preview each substitute something else for what
the renderer shapes, and both go through `ViewState::substitute_text`, which
keeps the document behind them — so folding a section does not change the word
count either surface reports, and a preview counts the manuscript rather than
the diff transcript on the page. Their LIVE-only figures (`hud.saved`, the
`lifetime` odometer, the `streaks` grid, the peek's learned rows, the About
card's update marker) read as their documented placeholders in a capture, in
the tree exactly as in the PNG — that determinism boundary is unchanged. The
roster is swept both directions, drawn ⇔ announced, by
`app::semantic::tests::passive_roster`.

Schema `/193` adds the top-level **`driver`** field, immediately after `schema`:
which TIER produced this sidecar. `"replay"` — the shared core
(`actions::apply_transition`), i.e. every `--screenshot` / `--keys` /
`--storyboard` / `--capture-timeline` / `--capture-held` / `--screenshot-frames`
capture. `"live-app"` — a real headless `App` (`--screenshot-app`; see
[App capture](app.md)). The distinction is load-bearing rather than cosmetic: two
sidecars can carry identical fields and mean different things, because a
`"replay"` capture owns no capability for the live-`App`-only effects it skips
(they appear in `replay_skips`) while a `"live-app"` capture performs them for
real and skips nothing. One owner: `capture::CaptureDriver`.
