# Rendering field reference

Read for layout, style, and typography field details. Versioned entries describe schema evolution; use CAPTURE.md for the current schema reservation.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

**Where the authoritative history lives.** The append-only per-round history
table is the doc comment above `capture::SCHEMA_VERSION` in `src/capture.rs` —
that is the one place the number and its per-bump rationale are maintained (the
one-const rule; never hand-copy the number). The detailed prose entries below
document the earlier rounds; rather than transcribe every bump since `/99`, this
is the CURRENT-STATE summary of what the sidecar carries now.

Vocabulary note (STRIKETHROUGH render round — **no schema bump**; the shape is
unchanged, only existing STRING fields gain a value, per `capture.rs`'s
"bump once per sidecar-SHAPE change" criterion): `md_spans` gains the
`"strikethrough"` content tag for GFM `~~struck~~` (its `~~` delimiters stay
ordinary `"markup"`, like every other syntax character; exactly-two tildes only
— a single `~x~` is inert, the `==` exactly-two precedent), and
`wysiwyg.concealed` gains the LINE-scoped `"strikethrough"` marker-conceal tag.
The struck text renders in the world's `muted` ink with a drawn strike line
(`render::spans::strike_line_band`/`strike_ink`, the ONE owner the format
popover's `S` button shares); the writer's-diff transcript serializes deletions
as real `~~…~~` now (the combining-stroke `\u{0336}` mechanism is retired). The
`popover.buttons` labels changed VALUE (not shape) to the self-demonstrating
roster `B I A code S H link`.

Schema `/187` adds the top-level **`layout`** block, the shaped-frame layout
oracle: `{ rows, caret, selection }`. `rows` is every shaped visual row in
document order. Each row is `{ index, content, line, start_col, end_col,
xs, top, height }`: raw source content for that wrapped span; its source logical
line and half-open character-column span; absolute physical-pixel x boundaries
for `start_col..=end_col`; and the absolute row top and shaped height. `caret`
is `null` or `{ row, line, col, x }`, using the caret's real wrap affinity.
`selection` is a list of row-local `{ row, start_col, end_col, x0, x1,
to_next_line }` segments. The pipeline seam returns no report before a frame is
prepared; after preparation it borrows the renderer-owned sealed row partition
in place. This is layout, not appearance: a perfectly reported row can still be
transparent or illegible in the PNG.

Schema `/169` (`/170` timeline, `/171` held) is the state after the rounds
between `/99` and here. Blocks/fields added or changed since the `/99` About
card, each detailed in `capture.rs`'s table: `hud.eol` (line-endings, `/100`)
and `hud.saved` (`/165`); `font.ornament` (`/103`); `overlay.show_hidden`
(hide-dotfiles, `/106`), `overlay.empty` (`/115`), `overlay.git` (`/118`),
`overlay.window` (bounded scroll-window, `/121`), `overlay.return_to`
(breadcrumb, `/154`), `overlay.title` (self-announcement, `/167`); `md_spans`
GFM table tags (`/109`); the theme-picker `swatches` array added (`/112`) then
DROPPED (`/124`); new top-level blocks `tables` (`/127`), `images` (`/130`),
`outline` (`/133`, with `ancestors` at `/139`), and `lifetime` (`/142`, split
out of `hud`); the `focus` block REMOVED (`/145`); new overlay modes `assets`
(`/148`) and `cjk_lang` (`/157`); the held `peek` block (`/151`); the `menubar`
block (`/160`-`/162`); the `xray` block + `tables.revealed` meaning (`/163`-
`/165`); `project.keymap_flavor` (`/164`); `about.checked` (Check for Updates,
`/166`); `page.background`'s `lava` arm (`/168`); `about.pending_crash`
(passive crash recovery, `/169`); `page.background`'s `bands`/`waves` arms
(Gumtree grass-bands + Bombora wave-tiers, `/181`); and
`page.background`'s `zigzag` arm (the repeating-chevron ground — Quokka
and Gumtree both moved to a repeating chevron ground, `/183`);
`page.background`'s `deckle` arm (Paperbark — `{kind, ground, layer,
deckle, weave, anchor, period_px, wander_px, density, static}`, where `weave` is the
theme-owned profile `"strata"`/`"fibres"`, `/189`); `page.background`'s
`organic` arm gaining `arrangement` (the ground's own theme-owned
profile, `/190`; Bowerbird's literal later became `"finds"`, the
crisp three-object collected-treasure field — `"masses"`, the original rounded
cut-paper field, rode on as reusable infrastructure carried by no world).
**Both of those keys, and `lava`'s `edge`, are GONE as of `/199`** — each named
a dial that had collapsed to one arm, so the key reported a constant; the
grounds themselves are unchanged and every world's PNG is byte-identical
across that bump. Note the asymmetry with `/198`'s own predecessor: a key
REMOVAL is a shape change because a reader keying on it breaks, while merely
narrowing a value space is not;
`overlay.workspace` + the `overlay.diff_focus` → `overlay.detail_focus` rename
(`/191`): `overlay.workspace` is `true` when the
summoned surface is drawn as a WORKSPACE — it takes the viewport, carries a
navigation rail, and leaves the document as a quiet backdrop — rather than as a
contextual card. When it is true, `overlay.lens_strip` IS that rail (the same
`[label, active]` pairs, presented as a column instead of a strip) and
`overlay.sections` is empty, because the rail names the active category once
instead of repeating it as a header over the only bucket it lets through. The
focus field was renamed because it was never diff-specific: it has always meant
"the summoned surface's DETAIL stage holds the keyboard", which for History is
its comparison and for Settings is the content pane beside the rail (the
diff-panel card the History detail stage used to be dressed as is retired; no
document-layer surface reads this field any more, and
`capture::tests::panels::history_preview_renders_the_transcript_as_the_document_in_every_world`
asserts a `detail_focus` flip moves no pixel of it); and
`page.background`'s `warped-grid` arm (Kite — `{kind, ground, minor, major,
tunnel, spacing_px, density, forward_cells}`, where `tunnel` is the theme-owned
profile and `forward_cells` is `0` in an ordinary headless capture because
nothing there ticks the clock; a STATE
oracle only — how the field LOOKS is asserted over the PNG, `/194`); and
`overlay.ranges` (Settings RANGE ROWS — a per-row array parallel to
`overlay.items`, `null` on an ordinary row and a 0..1 RAIL FRACTION on a range
row, so a `--keys`-driven rail step is assertable beside the value TEXT the
row's `bindings` cell already carried; empty for every mode but `settings`,
`/184`); and the metric-scale correction (`/185`), where
`font.size`/`font.line_height` became effective physical-pixel metrics and
`font.zoom` was added. Every one of these bumps preserved byte-identical
DEFAULT captures apart from the named field — see the table.

Schema `/99` (was `/98`; timeline `/100`, held `/101`) is the **SUMMONED
ABOUT CARD** (`about.rs` + `menu.rs`'s routed About item, which replaced
muda's predefined About dialog): a top-level `about` block, `{ "open": bool
}` — `false` by default (byte-identical capture), `true` after the palette
"About" command (or `--keys` replaying it) opens it. See [native menu ownership](../platform.md) for why About moved off muda's predefined item (a real
use-after-free fix in `menu::install`, unrelated to About specifically, plus
a separate taste upgrade to an in-app card).

Schema `/98` (was `/95`; timeline `/99`, held `/100`) is the **PROSE/CODE
PAGE-WIDTH SPLIT**: the 70-char measure is a PROSE number, and a recognized
code file now reads its OWN sticky measure (`page_width_code` in config,
default 100 — rustfmt's own `max_width`) instead of sharing the prose one
(`page_width_prose`, default 70 — the retired single `page_width` key's
successor). The `page` block gains **`class`** (`"prose"`/`"code"` —
`render::TextPipeline::page_class`, delegating to the SAME classifier
`Buffer::page_class` uses: a recognized code language is `"code"`; markdown,
the no-path scratch/note surface, or an unrecognized plain-text file is
`"prose"`), so a reviewer can assert which sticky measure is in effect
directly from the sidecar rather than re-deriving it from `syn_lang`. Every
other `page` field is unchanged in shape; a document that was implicitly
"prose" under the old single `page_width` key renders **byte-identically**
(same default measure — `70` — with `class: "prose"` newly reported).

Schema `/95` (was `/92`; timeline `/96`, held `/97`) **fixes** the `gutter`
block to always agree with the pixels — the gutter-elision bug: at a narrow
(but real) page-mode margin, the bottom-left orientation label used to lay the
RAW filename straight into a fixed-width box and let cosmic-text word-wrap it,
so a long name read as `"DESIG"` / `"N.md"` on two lines while the fixed-height
box silently clipped the `project` line out from underneath it — yet
`gutter.name`/`gutter.project` kept reporting the un-drawn raw strings. Same
shape (`{ visible, name, project }`), corrected meaning: BOTH `name` and
`project` are **exactly as drawn** — the new shared owner
(`render::rowlayout::gutter_plan` + `fit_primary`, the SAME middle-ellipsis,
extension-preserving elision door the picker rows already used) fits EACH of
them to **one line independently**, middle-eliding a line (keeping its
extension when it has one) only once the margin genuinely can't hold it
whole. A taste pass settled the two lines' relationship (still under this
same `/95` — the correction landed before this shape ever shipped): **neither
line yields to the other from width pressure** — a long filename elides while
the project keeps showing right alongside it, and vice versa; `project` is
`""` only when there is genuinely no project to report (never as a forced
yield to protect the filename). Below a hard floor (`GUTTER_MIN_NAME_CHARS`,
~6 chars of margin) the whole gutter hides rather than draw a stub.
**Unaffected** at any margin wide enough to hold both lines whole — every
existing wide-window capture is byte-identical; only a genuinely narrow
margin (the bug's own reproduction, `--capture-size` + `--measure`, or the
live app's page-column drag) sees a different `name`/`project` value than
before, and now it's the CORRECT one.

Schema `/92` (timeline `/93`, held `/94`) is the **i18n round**: multilingual
docs (Latin, ja, zh-Hans, zh-Hant, ko) get per-world per-script typography.
Two additive sidecar changes:

- A top-level **`doc_lang`** field: the document's own frontmatter `lang:` tag
  (`"ja"`/`"zh-Hans"`/`"zh-Hant"`/`"ko"`/`"en"`), or `null` for an untagged or
  non-markdown document. Pure function of the currently-shaped text (re-derived
  every reshape via `crate::frontmatter::detect`) — assert it directly after a
  `--keys` edit that types a frontmatter block, or after opening a fixture that
  already carries one.
- **`font.scripts`** — `font.cjk`'s `{ family, bundled }|null` shape
  generalized to the four non-Latin scripts this round adds ladders for:
  `{ "ja": {...}|null, "zh_hans": {...}|null, "zh_hant": {...}|null, "ko":
  {...}|null }`. `scripts.ja` **is** `font.cjk` — one
  `render::ScriptFontReports` snapshot per sidecar, read through one renderer,
  so the two are the same bytes by construction rather than two resolutions
  that happen to agree (they once didn't under a concurrent
  theme flip) — and is non-`null` in every normal build (bundled Noto
  Serif/Sans JP). `zh_hans`/`zh_hant`/`ko` ship **no bundled asset** this round
  (a v1 taste call — PingFang SC/TC, Apple SD Gothic Neo, falling back to Noto
  Sans CJK SC/TC/KR on Linux), so those three are genuinely machine-dependent:
  `null` is the documented degenerate case on a box with none of those
  installed, not a bug.

A frontmatter block itself is invisible to `md_spans`/word-count/spell/nits by
DESIGN (metadata, not manuscript — see the `wysiwyg`/`md_spans` note below and
`crate::markdown::frontmatter_end`); it renders as dim `Markup` and obeys the
SAME block-scoped WYSIWYG conceal a fenced code block does (`wysiwyg.concealed`
reports it tagged `"frontmatter"`, revealed only when the caret sits anywhere
inside the block — reuses the `Fence` seam verbatim, no new machinery). The
held stats HUD also gains a `lang` field (`hud.lang`) — deterministic, so it's
capture-safe like every other HUD figure. It reads the DOCUMENT's frontmatter,
which is the shaped text's own on every ordinary frame; see the sidecar table's
`hud` row for the two states where the shaped text is a substitute and this
field parts company with `doc_lang`.

Config gains `cjk_priority` (a TOML array of BCP 47 tags, default `["ja",
"zh-Hans", "zh-Hant", "ko"]`): the tiebreak ladder for an AMBIGUOUS Han-only
run/document (kana/hangul/bopomofo are unambiguous and never consult it). It
drives both the explicit "Tag document language" palette command (which stamps
a `lang:` frontmatter block in as one normal undoable edit — the ONLY door that
writes one; opening a document never does) and the per-run render resolution
ladder.

⚠️ **NO CAPTURE CAN WITNESS THAT LADDER, `--config` or not** — measured, and
the reverse of what this paragraph used to claim. Both doors paint through
`capture::capture_with`, whose `ViewState` comes from `ViewState::base()` and
pins `DEFAULT_CJK_PRIORITY`; nothing under `src/capture/` assigns
`cjk_priority`, and `App::sync_view` — the one site that reads
`Config::cjk_priority_or_default` — never runs without a GPU. On a bare-Han
fixture under `--screenshot-app`, `ja`-first / `zh-Hans`-first / `ko`-first
`--config` ladders all produce a BYTE-IDENTICAL PNG, while the same document's
own frontmatter tag (`lang: ja` vs `lang: ko`, same byte length) changes it —
so the oracle is sensitive and the ladder is simply absent.
`font.scripts`/`font.cjk` cannot report it either: they are the WORLD's
per-`FontId` family roster, invariant in the ladder by construction. What a
capture CAN prove is the READOUT — `--semantic-json --keys "Cmd-,"` shows the
Settings row following `--config`. See `docs/harness-reach.md`.

The `cjk_priority` SETTING gained a Han-ambiguity EVIDENCE tier ahead of it
(`script::cjk_evidence` — kana/simplified-only/traditional-only/hangul
detected straight from the document text, folded in via
`script::effective_cjk_priority`) and Settings gained an **Auto** value for
the setting itself (`frontmatter::cjk_priority_is_auto`, config
`cjk_priority = "auto"`). The unreachability above is unchanged for the
SETTING (still pinned/config-blind exactly as measured) — but the evidence
tier is a different axis: it is a pure function of the buffer TEXT, computed
inside `TextPipeline::set_text_incremental` with no `Config`/`ViewState`
involvement at all, so an ORDINARY `--screenshot` (no `--config`, no
`--screenshot-app`) exercises it fully. A bare-Han fixture carrying a
simplified-only character now renders in the bundled Simplified-Chinese face
even under the capture pipeline's pinned `ja`-first default, which is the
regression this tier fixes made visible to the harness for the first time.

Schema `/89` (timeline `/90`, held `/91`) adds a top-level **`buffers`** block
for the MULTI-BUFFER CORE (N open buffers, exactly one active, switching
preserves everything — see ARCHITECTURE.md): `{ "open": N, "active":
"path-or-scratch" }`. `open` counts every currently-open buffer (the active one
+ anything backgrounded — see `crate::buffers::BufferRegistry`); `active`
names the active buffer's identity: its absolute path, or the literal string
`"scratch"` for the singleton pathless writing surface, or `"untitled"` for a
fresh Cmd-N buffer awaiting its first successful naming save. A plain `--screenshot` (no
`--keys`, or a `--keys` spec that never opens a second file) always reports
`open: 1` — **byte-identical single-buffer behavior**, the schema bump is
additive only. Drive the multi-buffer case with `--keys` chaining two Go-to-
file (`C-x C-f`) accepts around an edit — e.g. open `a.txt`, type, `C-x C-f`
to `b.txt`, type, `C-x C-f` back to `a.txt` — and the final capture's `text` /
`cursor` reflect A's PRESERVED edit + cursor (not a fresh disk re-read), while
`buffers.open` stays at the count of everything still open (the launch
scratch + A + B) and `buffers.active` names A again. This exercises the SAME
`crate::buffers::BufferRegistry` the live App uses to make "opening a file
that's already open switches to its live buffer" true, wired inline inside
`main/run.rs`'s `replay_keys` so it composes across an entire `--keys` run
(`run::tests::replay_keys_goto_a_then_b_then_a_preserves_edits_and_cursor`).
Tab-strip/selector UI, session restore, and cross-process buffer sharing are
explicitly OUT of this round (state model only, no chrome).

Schema `/204` widens that block with the VISIBLE WORKING SET the margin's
bottom identity draws: `files` is one label per drawn stack row, in STABLE OPEN
ORDER, and `active_index` names which row is the current file (`null` when
there is no stack). A row's label is its file's full ROOT-RELATIVE path
(`journal/field-notes.md`) — a state label, not the width-elided text the
margin drew, which stays `gutter`'s business. `files` is `[]` whenever no stack
is drawn: a single open file, and every capture door with no live `App` to ask.
Because the working set is App-owned, the multi-file case is reachable only
through `--screenshot-app` (`docs/harness-reach.md`), seeded with `--seed-tree`
so a hermetic sandbox holds a real project to open a second file from. This is
the oracle for the order contract: switching files moves `active_index` and
must leave `files` untouched — the stack is stable-order, not MRU.

Schema `/209` adds **`overlay.query_caret`**: the query field's own
CHAR-index caret (`crate::textbox::TextBox::caret`). It equals `query`'s own
length — the field's RESTING position — after ordinary typing, a click on the
query line, or a drag; a click/drag that lands short of the end, or
`ForwardWord`/`BackwardWord` stopping mid-query, leaves it there instead. That
resting-vs-mid-query fact is also the switch a picker's `ForwardChar` /
`BackwardChar` / `LineStart` / `LineEnd` read: AT the end they keep their
list-nav overloads (lens cycle, folder descend/ascend, row jump); anywhere
else they fall through to the field's own char motion / Home-End, and
reaching the end again (an End, or a char-step that lands there) restores the
list-nav reading on the very next keypress.

Schema `/210` adds **`overlay.window.cue_above`** / **`overlay.window.cue_below`**:
the faint positional COUNT CUE ("↑ 3 more" / "↓ 41 more") that
draws when a candidate window clips the corpus. Each is `null` when nothing
is hidden past that edge, else the ITEM count — never a display-line count: a
sectioned card (the theme picker) windows DISPLAY LINES (headers + item
rows), but the cue counts hidden ITEMS, so `cue_below` plus `n_items` minus
`top` minus the ITEM count in the drawn window (not `lines`, which bills
section headers too) is the arithmetic to check, not a bare subtraction
against `lines`. Derived at the one windowing owner (`scroll_window`'s own
`(top, visible)` pair, read through `window_edge_counts`,
`render/chrome/mod.rs`) shared by every candidate window — flat, grouped, and
the summoned workspace — so a picker that never clips reports both `null`.

Schema `/211` adds **`overlay.asset_preview`**: the Asset Cleaner's live
preview panel's own PLANNED rect `{ x, y, w, h }`, beside the picker's card at
the same `card_y`/`card_h` — or `null` off that picker, or on a canvas with no
genuine room for it beside the card. Read straight off the SAME
`TextPipeline::asset_preview_rect` the panel's background, thumbnail and
can't-decode text all draw from, so a Verify clause can sample the panel's
pixels at an exact coordinate instead of estimating its bounds from private
layout constants. Which orphan is selected is already sidecar-visible via
`overlay.items[overlay.selected_index]` / `overlay.bindings[overlay.
selected_index]` (name / size), so the CONTENT of the panel — does it show
that orphan's own image, does a can't-decode file show its honest statement —
stays an appearance claim over the PNG's pixels (the capture guide's state/appearance distinction), never the sidecar's to answer.

Schema `/207` adds top-level **`document`**: `{ active, start_actions }`.
Ordinary frames report `active: true` and no start actions. After a tier-2
live-App capture closes the last document it reports `active: false` and
exactly `["New document", "Go to"]`; `page` and `buffers.active` are `null`,
`buffers.open` is `0`, and the semantic tree contains the matching two buttons
instead of a document node. The renderer receives no hidden scratch state: the
transport buffer used by the offscreen capture is suppressed by this same fact,
so `text_origin`, `cursor`, `text`, and `layout` are also `null`, `line_count`
is `0`, and `first_lines` is empty.

Schema `/86` (timeline `/87`, held `/88`) adds a top-level **`wysiwyg`** block
for the WYSIWYG amendment ("if the caret is on that line, show the actual
markdown; otherwise show the preview" — see PHILOSOPHY.md): `{ "on": bool,
"concealed": [[start_byte, end_byte, "kind"], ...] }`. `on` mirrors the sticky
config pref (`wysiwyg`, default `true`; `false` reproduces the pre-round
always-visible markup byte-identically — no conceal, no inline-code pill, no
fenced-block panel). `concealed` lists exactly the ranges the renderer drew
TRANSPARENT this settled frame — a heading's leading `#`, a bold/italic
delimiter run, an inline code span's backticks, or a `==highlight==` delimiter
pair, tagged `"heading"`/`"emphasis"`/`"code"`/`"highlight"` respectively —
each **LINE-scoped**: revealed (absent from `concealed`) only when the caret
sits on that exact line. A FENCED code block's marker lines (the info-string
line + the closing fence) report tag `"fence"` and are **BLOCK-scoped**:
revealed only when the caret is ANYWHERE inside the whole block, never per
individual line (so stepping through a multi-line block's body doesn't flicker
the fence markers); the block's BODY lines never appear in `concealed`
regardless of caret position (they carry their own `code`/`syn_spans`/
`code_<lang>_<role>` coloring, never blanked). `md_spans` itself is **UNCHANGED**
by this round — a concealable span still reports its ordinary `"markup"` (or
`"code"`) tag there; `wysiwyg.concealed` is the additive, separate report of
which of those ranges are *currently* invisible. Drive it with `--keys` moving
the cursor onto/off a heading or fenced-block line and diff `wysiwyg.concealed`
between the two captures (`markdown::tests`/`render::tests`/`capture::tests`
cover the per-kind conceal-on/off, the caret-enters-line reveal, the fenced
block's whole-block reveal, and `wysiwyg = false` byte-identity).

The fenced-code PANEL (a quiet value-step `base_200` background spanning the
whole block, fence lines AND body, ALWAYS present once `wysiwyg.on` is true —
independent of the caret; only the marker TEXT concealment is caret-gated) and
the inline-code PILL (the same value-step tint, a small overhang behind an
inline `` `code` `` span) are GPU geometry, not part of the JSON — verify their
PRESENCE indirectly via `wysiwyg.on` + `md_spans`/`syn_lang` (a fenced/inline
code span exists) and confirm the pixels visually from the PNG; the exact quad
placement is a render-test concern (`render::tests`), not a sidecar field.

Schema `/86` (timeline `/87`, held `/88`) also adds **`font.cjk`** — the Japanese-
bundle round (see `theme.rs`'s `CJK_MINCHO`/`CJK_GOTHIC` doc + CLAUDE.md): awl
now embeds Noto Serif JP + Noto Sans JP (Google Fonts, OFL, JIS X 0208-subset;
`render::FONT_CJK_FACES`) and lists them FIRST in the per-world CJK candidate
list, ahead of the system Hiragino/Noto-CJK fallback. `font.cjk` reports the
active world's *resolved* candidate — `{ "family": "Noto Serif JP" | "Noto Sans
JP" | a system face name, "bundled": true|false }` — or `null` in the
contrived case where NEITHER a bundled nor a system candidate is present. Since
the bundled face is always registered in a normal build, `font.cjk` is
non-`null` in every default capture and, critically, **machine-independent**:
a JP fixture rendered under any world resolves to the SAME bundled family on
every machine, with no dependency on which system CJK fonts happen to be
installed (the property the harness could not previously assert — see
`capture::tests::i18n_fixtures::japanese_fixture_resolves_bundled_cjk_face_deterministically`,
the first JP-rendering capture test). Bundling is TASTE-GATED, not yet the
final call: Hiragino/system stays as a trailing candidate until a live
eyeball-call between the two (see `gallery/jp-compare/` — Bombora/Currawong ×
Hiragino/Noto, produced via the dev-only `AWL_CJK_FORCE=system|bundled` env
knob, not a shipped flag). The Chinese round bundled zh-Hans (Noto Serif/Sans
SC + a characterful LXGW WenKai override for the Klee worlds) and ko (Noto
Sans KR) the same way — `font.scripts.zh_hans`/`.ko` report the same
`{family, bundled}` shape, and `AWL_CJK_FORCE` gained a third value (`floor`,
pruning just the characterful WenKai) to produce the analogous
`gallery/zh-worlds/` A/B/C captures. See THEMES.md for the world-by-world
assignment table.

Schema `/80` (timeline `/81`, held `/82`) adds **`highlight`** to the `md_spans`
tag vocabulary for the de-facto `==marked==` convention (Obsidian/Typora/iA —
NOT CommonMark, which has no `==` construct). A markdown buffer's
`==marked text==` reports the inner text as a `"highlight"` span and its `==`
delimiters as ordinary dim `"markup"` spans, exactly like every other syntax
character. RENDER: the marked text keeps FULL content ink (a no-op transform in
`md_attrs`, like `Heading`) with a warm wash quad drawn BEHIND it — reusing the
SAME wash pipeline + tint as the prose-comment wash (`role_style_for`'s
`Comment` arm; `rects.rs::ensure_wash_protos` routes `MdKind::Highlight` spans
into that identical bucket, one warm-wash owner rather than a third
pipeline/shader). A single `=` is deliberately meaningless (rejected — prose
like `x = y` must never match): only an ISOLATED run of EXACTLY TWO `=`
qualifies as a delimiter, so a bare `=`, a `===`, and an adjacent `====` all
stay inert literal text (`markdown::equals_runs`). Delimiters pair up greedily
two at a time; an unpaired trailing `==` is left as plain text (no crash, no
span — the "unclosed `==`" case), and a candidate pair separated by a `\n` is
rejected too (NO CROSS-LINE SPANS — a soft-wrapped paragraph already arrives as
separate `Text` events split at the break, so this mostly guards a defensive
edge the parser doesn't otherwise produce). `==` inside inline code or a fenced/
indented code block is ignored (inline code arrives via a separate event
entirely; a code-block body is explicitly skipped). A CODE buffer's `a == b`
comparison never risks matching in the first place — `markdown::spans` is only
ever invoked on an `is_markdown` buffer (`render/text.rs`'s `md_enabled` gate),
so a `.rs` file's `==` never reaches this module at all. Drive it with a `.md`
buffer containing `==marked text==` and assert `md_spans` carries `"highlight"`
(`capture::tests::schema_chrome::markdown_highlight_tag_present_in_sidecar`); the wash pixels
are covered at the render-test layer instead of a PNG diff
(`render::tests::washes::markdown_highlight_inherits_wash_and_code_buffers_never_match`).

Schema `/77` (timeline `/78`, held `/79`) adds **`silhouette`** to the
top-level `caret_preview` block (the caret-style picker's floating preview
panel; see [interaction fields](interaction-fields.md)) — whether the MORPH glyph-silhouette pipeline actually
painted THIS frame (settled on a real inhabited glyph while Morph is the
highlighted look; `false` for Block/I-beam, or for a Morph moment with no
glyph to light / still in fast motion, where the preview falls back to the
same thin bar / streak the block pipeline draws). Fixes a bug where the
picker's demo caret NEVER fed the glyph-silhouette pipeline at all — it always
drew a permanent thin bar for Morph, so the one place a user chooses the look
never actually demonstrated it. The preview now runs its OWN
`CaretGlyphPipeline` instance (never the document's — the two may prepare and
draw in the same frame while a crisp caret picker sits over the live
document) through the same settled-glyph / glyphless-bar / fast-motion-streak
three-way dispatch the document caret uses. Drive it with
`--keys "Cmd-P C a r e t Enter Down"` (opens the palette, filters to "Caret
style", opens the picker, arrows down to Morph) and assert
`caret_preview.silhouette == true` on the settled capture (the sample line
ends `"...morph"`, so the anchor — one char back of the insertion point — is
a real letter, `"h"`).

Schema `/74` (timeline `/75`, held `/76`) adds a top-level **`spellcheck`**
boolean — the GLOBAL spell-check on/off (default `true`), reported alongside
`dictionary`. Toggle it live via the "Toggle Spellcheck" palette command, or
set it once via `--config` (`spellcheck = false`). OFF silences EVERY
squiggle — prose and the scoped code-string/comment check alike (see the
STRING PROSE GATE below) — and `misspelled`/the squiggle geometry go empty
regardless of what the buffer contains, so `spellcheck` is the field to assert
rather than inferring "off" from an empty squiggle list (a clean document with
zero typos would look the same). The SAME round also GATES a code buffer's
`Str` spans on a small prose heuristic (`spell::looks_like_prose_string`,
mirroring `syntax::looks_like_code`'s shape): a STRING squiggles only when its
content reads as prose (2+ space-separated word-shaped tokens) — a bare
single-token string (`"struct"`, `"en_AU"`, a format specifier, a CSS
selector) never does, fixing bare code-vocabulary strings squiggling inside
`syn_lang`-detected buffers. `syn_spans`/`md_spans` are unaffected (this only
narrows which words `misspelled` reports on top of them).

Schema `/73` (timeline `/74`, held `/75`) adds the AUTOSAVE-ENGINE line to the
opt-in `debug` panel + block: a quiet `autosave …` line stamped EXCLUSIVELY
through `App::autosave_flush`'s one door (+ its clobber-guard sub-paths
`autosave_doc_now` / `stash_scratch_now`), so it can never say anything the
engine did not just do — user request: "add 'autosaved' or some indication to
the debug menu". Live it reads `autosave saved · Ns ago` (the engine wrote
successfully `N` whole seconds ago this session), `autosave on` (enabled, not
held, nothing written yet this session), `autosave held — disk changed` (the
CLOBBER GUARD is currently blocking a write — mirrors the existing calm
bottom-center notice), or `autosave off` (`autosave = false` in config). The
`debug` block gains two machine-readable fields alongside the existing perf
ones: `autosave_state` (`"off"` / `"held"` / `"saved"`, else `null`) and
`autosave_since_s` (whole seconds since the last successful engine write, else
`null`). Like the perf triad, the ENTIRE autosave line is live-App-only — the
engine is structurally unreachable from a headless capture (see
`headless_replay_never_arms_autosave_or_stashes_scratch`), so a `--debug`
capture always renders the FIXED, numberless placeholder `"autosave —"` and
both new fields are `null`, keeping the block byte-stable across machines. A
default (`--debug` absent) capture is unaffected — the panel draws nothing.
Note the panel schedules ZERO frames either way (the debug-panel-v2 contract):
the "Ns ago" figure only advances on whatever frame the editor draws anyway
(an edit, a spell-debounce repaint, …), not on its own timer — a LIVE-ONLY feel
(the number visibly climbing while you watch) that the harness cannot verify.

Schema `awl-capture/67` (was `/64`; timeline `/68`, held `/69`) adds
`overlay.preview_id` for the HISTORY TIMELINE's live preview: while the History
picker is open, the highlighted row's VERSION is previewed **in the document
itself** — the top-level `text` (and the whole rendered frame: scroll math,
cursor clamped into the previewed rows, buffer-indexed spans cleared) reports
THAT version's content, and `preview_id` names its restore id, so "arrowing the
rows shows that version" is assertable headlessly. `null` for every other
overlay mode, the empty-state row, and a plain `--screenshot` (whose PNG stays
byte-identical). The same bump reworked the History rows to answer WHEN + WHICH:
`overlay.items` compose `"{when} · {which}"` (the relative label — clock-suffixed
`" HH:MM"` exactly when siblings share a label — then the git COMMIT SUBJECT or
an awl snapshot's auto-description, e.g. `edited "Two flows, one engine"`), and
`overlay.bindings` carry the faint `"+N −M"` changed-counts. Drive it with
`--keys "Cmd-S-h C-n"` (open + arrow: `text` == that version, `preview_id` set);
`Esc` closes with the buffer untouched; `RET` restores undoably. The History
backdrop is CRISP (no frosted blur) — the document IS the preview.

The SAME `/67` bump also adds the TWO-TIER COMMENT tag to the syntax role vocabulary: `syn_spans` may now carry
**`comment_code`** alongside `comment` — a comment whose body reads as
COMMENTED-OUT CODE (the central `syntax::looks_like_code` heuristic,
default-to-prose) is reported as `comment_code` and renders in the muted grey,
while a PROSE comment keeps the `comment` tag and renders PROMINENT (full
content ink + the per-world comment wash). Markdown fenced spans gain the same
tier through the shared seam: `md_spans` may report `code_<lang>_comment_code`
(e.g. `code_rust_comment_code`) next to the existing `code_<lang>_comment`.
The role COLORS the tags map to are now derived by `role_style_for`
(`render/spans/colors.rs`) — quiet per-world hue tints + low-alpha background washes;
same tags, new pixels, law-tested per world.

Schema `/70` (timeline `/71`, held `/72`) adds a top-level **`dictionary`**
field — the active spell-check dictionary variant (`"en_US"` / `"en_GB"` /
`"en_AU"`, `config::dictionary_name`), reported alongside `caret_mode`. `en_US`
is the built-in default (an absent config `dictionary` key, or none at all,
keeps a plain `--screenshot` byte-identical). Switch it live via the summoned
