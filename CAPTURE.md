# Capture guide and sidecar contract

Read this guide before producing capture evidence. [Capture reach](docs/harness-reach.md)
identifies which driver can exercise the requested behavior; [verification policy](docs/verification.md)
defines check scope. Read the focused reference for the mode or field you use.

## Quick start

Run from the repo root. On macOS, use the configured stable Rust toolchain:

```sh
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo run -- --screenshot OUT.png path/to/file.md
cargo run -- --screenshot OUT.png --keys "C-n C-n" path/to/file.md
```

The output is a PNG plus a JSON sidecar at the same stem. Omitting the input file
captures a scratch buffer. Ordinary replay uses the real keymap and shared core;
it never saves real files, but can read ambient config and skip App-only effects.
Use an explicit config and fixture root. Put output outside any listed corpus.

For App-owned state and isolated stores:

```sh
cargo run -- --screenshot-app OUT.png path/to/file.md --keys "SPEC"
cargo run -- --semantic-json path/to/file.md
```

`SPEC` is a placeholder for actual chords. The App driver is hermetic and executes
App effects; it does not exercise native windows or event-loop dispatch.
`--semantic-json` writes semantic state to stdout without a GPU or PNG.

## Select a reference

| Task | Read |
| --- | --- |
| Chord grammar, search/replace, strict replay and sandboxing | [Replay](docs/capture/replay.md) |
| App effects, recovery/history fixtures, semantic JSON | [Headless App](docs/capture/app.md) |
| Frame loops, deterministic motion, timelines, storyboards | [Motion](docs/capture/motion.md) |
| World galleries and review dashboard | [Visual review](docs/capture/review.md) |
| Determinism, coverage audits, debug/release differential | [Evidence](docs/capture/evidence.md) |
| Native menus and wasm smoke | [Platform checks](docs/capture/platform.md) |
| Geometry, chrome, and semantic field details | [Geometry fields](docs/capture/geometry-fields.md) |
| Layout, styles, typography, and their schema evolution | [Rendering fields](docs/capture/render-fields.md) |
| Picker, HUD, buffer, and navigation field details | [Interaction fields](docs/capture/interaction-fields.md) |
| Illustrative JSON | [Sidecar example](docs/capture/sidecar-example.md) |

## Interpret the evidence

The sidecar's state fields answer what state the editor holds. `layout` reports
geometry from the exact shaped frame. PNG arithmetic establishes visibility,
contrast, and perceptual distinction. Neither state nor geometry proves appearance.
Use `render/tests/pixeldiff.rs` for comparisons over actual rendered pixels.

Record driver, canvas, DPI, `font.zoom`, inputs, and backend. Captures are stable
for identical inputs on the same machine; fallback fonts and rasterizers can vary
across platforms. Compare matching configurations. A settled frame or deterministic
trajectory does not prove live cadence, compositor behavior, or taste.

Read replay warnings and `replay_skips`. Strict replay aborts on unbound chords,
dangling prefixes, Unsupported effects, or unavailable layout capability; it can
save only into its isolated scenario filesystem. Theme-picker navigation performs
real preview, so it can change the world requested by `--theme`.

## Paths are home-relative

The sidecar writer replaces the builder's home path with `~`; consumers expand it
when needed. This does not sanitize directory entry names or the PNG. Use seeded
roots and explicit config for public artifacts; do not capture private filenames.

## The sidecar JSON — schema `awl-capture/219` (`/220` timeline, `/221` held)

This header and the field table are read by the existing schema/enumeration laws.
Keep them here. `capture::SCHEMA_VERSION` and its source ledger own schema numbers.
Detailed field explanations and version notes are in the references above; mentions
of earlier schemas there do not reserve a current number.

| field          | meaning |
|----------------|---------|
| `schema`       | sidecar format version; bump if the shape changes |
| `semantic`     | ACCESSIBILITY (schema `/196`): the renderer-independent semantic tree, or `null` on a `"replay"` capture. The same value `--semantic-json` prints and the native AccessKit adapter projects — see the narrative above for its shape, its grapheme-offset selections, and the card-content gap a pipeline-less live-`App` capture has |
| `canvas`       | render target size in pixels |
| `font`         | active theme's chosen font family plus `zoom`, effective `size`, and effective `line_height` used for layout. `size` and `line_height` are physical pixels after zoom × capture DPI, in the same coordinate space as `text_origin`, `page.column`, and the PNG; therefore `top + n * font.line_height` addresses the correct ordinary row at every zoom. `zoom` is the clamped user zoom factor (DPI remains separately reported by `canvas.dpi` when explicitly set). `cjk` = `{ family, bundled }` — the world's resolved Japanese fallback face (bundled Noto Serif/Sans JP first, system Hiragino/Noto-CJK trailing — see the Japanese-bundle-round schema `/86` note above), or `null` if neither is present; `scripts` = `{ ja, zh_hans, zh_hant, ko }` (i18n round, schema `/92`) — `cjk`'s shape for all four non-Latin scripts; `ja` **is** `cjk` (one snapshot, same bytes), the other three may be `null` (no bundled asset yet, machine-dependent) |
| `theme`        | active color world: `name`, `font_family`, `mode` (light/dark), `base100`, `primary` (hex) |
| `caret_mode`   | effective caret look (`"block"`/`"morph"`/`"ibeam"`) |
| `dictionary`   | active spell-check dictionary variant (`"en_US"`/`"en_GB"`/`"en_AU"`); default `en_US`. Set via `--config` (`dictionary = "en_AU"`) or the Dictionary picker (Cmd-P → "Dictionary") |
| `spellcheck`   | GLOBAL spell-check on/off; default `true`. `false` silences every squiggle (prose and scoped code strings/comments alike) and makes the spell-suggest picker a no-op. Set via `--config` (`spellcheck = false`) or the "Toggle Spellcheck" palette command |
| `date_format`  | INSERT DATE (schema `/178`): `{ format, example }` — the active `crate::dateformat::DateFormat`'s persisted slug (`"ddmmyy"`/`"mmddyy"`/`"iso"`/`"yyyymmdd"`/`"dmonthyyyy"`; default `"ddmmyy"`) and that format rendered against the FIXED placeholder civil date (2009-03-07 — a headless capture has no clock, so "today" is always this same date). Set via `--config` (`date_format = "iso"`) or the Settings menu's "Date format" cycling row. `example` for the default is `"07/03/09"` |
| `text_origin`  | top-left pixel of the first glyph row (`left` = the page column left, centered in page mode; `16.0` edge-to-edge) |
| `document`     | `{ active, start_actions, start_folder, start_goto_chord }`: whether a real document exists. With none, the actions are exactly `["New document", "Go to"]`; otherwise `[]` |
| `page`         | PAGE MODE: `null` with no active document; otherwise `on` (centered column vs edge-to-edge), `measure` (column width in chars), `class` (schema `/98`: `"prose"`/`"code"` — which sticky measure, `page_width_prose`/`page_width_code`, is in effect for this document; see `crate::page::PageClass`), `column.{left,width}` (px), `background` (the active world's margin shader — a tagged `{kind, ...}` object, e.g. `{kind:"gradient", from, to, dir}`, `{kind:"dots", from, to, dir, tint, edge}`, `{kind:"bands", tones:[c0,c1,c2], angle}` (Gumtree), `{kind:"waves", tones:[c0,c1,c2]}` (Bombora), or `{kind:"deckle", ground, layer, deckle, weave, period_px, wander_px, density, static}` (Paperbark — `weave` is the theme-owned profile, `"strata"` on Paperbark and `"fibres"` on Galah; the `anchor` key was removed in `/199` when that dial collapsed to its viewport arm), or `{kind:"organic", tones:[c0,c1,c2], scale_px, density, phase}` (Bowerbird — the `arrangement` key was removed in `/199` for the same reason; the ground draws the crisp collected-treasure field and nothing else), or `{kind:"warped-grid", ground, minor, major, tunnel, spacing_px, density, forward_cells}` (Kite — `tunnel` is `"fixed"`; `"page-scaled"`, `"margin-placed"`, and `"reversed"` are mutation arms)) |
| `focus`        | FOCUS MODE: `mode` (`off`/`paragraph`/`sentence`) + `active_start`/`active_end` (char offsets of the full-ink unit, `null` when off) |
| `wysiwyg`      | WYSIWYG conceal: `{ on, concealed }`. `on` mirrors the sticky `wysiwyg` config pref (default `true`). `concealed` is `[start_byte, end_byte, "kind"]` ranges the renderer drew transparent THIS frame — `"heading"`/`"emphasis"`/`"code"`/`"highlight"` (LINE-scoped: revealed only on the caret's own line OR a line the active selection touches) or `"fence"`/`"frontmatter"` (BLOCK-scoped: revealed only with the caret anywhere inside the block, or the selection touching any line inside it — a frontmatter block reuses the `fence` rule verbatim, see schema `/92`; selection reveal, 2026-07-22, no schema bump — see `render::spans::wysiwyg_reveals`). `"table"` (schema `/163`-ish, see the `tables` narrative above) NEVER leaves `concealed` in place — a selected/caret-touched table row instead swaps to the `xray` float mechanism; `tables[].revealed` and the render-only `xray` state are the ones to check for a table. Empty when `on` is false or nothing is concealed this frame |
| `doc_lang`     | i18n round (schema `/92`): the document's own frontmatter `lang:` tag (`"ja"`/`"zh-Hans"`/`"zh-Hant"`/`"ko"`/`"en"`), or `null` for an untagged/non-markdown document |
| `md_spans`     | MARKDOWN STYLING: array of `[start_byte, end_byte, "tag"]` styled spans (`markup`/`h1`..`h6`/`bold`/`italic`/`bold_italic`/`code`/`quote`/`list_marker`/`link_text`/`task_open`/`task_checked`/`task_done`/`rule`/`highlight`); empty for non-`.md` buffers. A frontmatter block's span also reports plain `"markup"` here (the conceal STATE lives in `wysiwyg` instead — see above). UNCHANGED by the WYSIWYG round — a concealable span still reports its ordinary tag here regardless of the caret |
| `syn_lang`     | SYNTAX HIGHLIGHTING: the DETECTED code language name (`"rust"`, `"go"`, …) or `null` for a non-CODE buffer; agrees with `syn_spans` (`null` ⇔ empty) |
| `syn_spans`    | SYNTAX HIGHLIGHTING: array of `[start_byte, end_byte, "tag"]` Alabaster role spans (`comment`/`string`/`constant`/`definition`); empty for non-CODE buffers (`.env`/`.md`/`.txt`/unknown). Mutually exclusive with `md_spans` |
| `readout`      | QUIET word/character-count readout: `{ words, reading_min, unit }` (reading_min = ceil(words/200), min 1; `unit` is `"words"` or `"characters"`; see the schema `/198` narrative above), or `null` for a non-markdown / wordless buffer. NO LONGER drawn (moved to the held HUD); kept as the HUD's source |
| `gutter`       | PAGE-MODE GUTTER: `{ visible, name, project, changed }` — the left-margin orientation label: the folder heading (muted, LABEL size) over the filename — or, once more than one file is open, a directly scrollable working-set stack in the filename's place (including files remembered under another root) — the heading always on top so a second file opening only inserts a row below it. `visible` is true only when drawn (page mode + a name + a margin past the hard floor, `render::rowlayout::GUTTER_MIN_NAME_CHARS`); `name` and `project` are each **exactly as drawn** — independently fit to ONE line, middle-elided (extension preserved) only once the margin can't hold that line whole (`render::rowlayout::gutter_plan`/`fit_primary`, the same door the picker rows use). Neither line yields to the other from width pressure; `project` is `""` only when there is genuinely no project to show. `changed` (schema `/197`) is the persistent `changed elsewhere` affordance — `true` only on a `driver: "live-app"` capture whose document holds an unresolved external change, in which case the block draws an additional line above the folder heading in a stronger ink |
| `notice`       | THE CALM NOTICE (schema `/200`): `{ text, kind }`, or `null` when nothing is showing. `text` is the sentence exactly as drawn (elided to the column's budget on a narrow canvas — `render::rowlayout::fit_primary_end_to_px`, the same pixel-truth door the margin outline uses); `kind` is `"toast"` or `"sticky"`. Read off the PIPELINE, not off the fold's input, so the block cannot claim a message the PNG does not carry — and `null` on a frame that YIELDS the notice (a relocated read-only comparison) even though one is set. Drawn as one plated LABEL line at the top of the writing column: fill `base_200`/`base_300` by kind, a one-pixel rim `muted`/`base_content`, text through `theme::selected_row_ink`; a true one-bit world inverts the sticky arm because it has no value step to spend |
| `dim_overlay`  | `true` when a FULL-takeover overlay dims the document behind it (the scrim); `false` for the search SPLIT panel / no overlay (DESIGN §5) |
| `debug`        | DEBUG panel (renamed from the old `fps` counter): `{ enabled, text, frame_ms, worst_ms, budget_ms, key_px_ms, redraws, still, autosave_state, autosave_since_s }`. OFF by default (empty `text` → byte-identical). `text` is the full stacked readout; `frame_ms`/`worst_ms`/`budget_ms`/`key_px_ms`/`redraws`/`still` are the machine-readable perf triad (all `null` + `still: true` in a capture — no clock runs headlessly). `autosave_state` (`"off"`/`"held"`/`"saved"`, else `null`) + `autosave_since_s` (whole seconds since the last successful autosave write, else `null`) mirror the panel's `autosave …` line, fed EXCLUSIVELY through `App::autosave_flush`'s one door — both `null` in every capture (the engine is structurally live-App-only) |
| `hud`          | HELD STATS HUD: `{ held, words, reading_min, unit, percent, lang, selection }`. `held` is the summon state (false by default → byte-identical); `words`/`reading_min`/`unit` null for non-markdown (`unit` is `"words"`/`"characters"`, schema `/198` — see the narrative above; always agrees with the top-level `readout` block, one owner); `percent` = cursor %-through-doc; `lang` (schema `/92`) is the document's frontmatter language; `selection` (schema `/205`) is null or raw selected-text `{ words, characters }`, with extended-grapheme characters and one logical line break per selected newline. Every figure is a pure function of the USER'S DOCUMENT + cursor/selection — the whole buffer, never the shaped page — so a collapsed fold or an open History preview leaves them unmoved (`readout` likewise). It follows that `lang` and the top-level `doc_lang` can differ: `doc_lang` is the SHAPED text's language, which is what the per-script font ladder must follow, and a diff transcript carries no frontmatter. No clock, fully capture-safe |
| `about`        | SUMMONED ABOUT CARD (schema `/99`): `{ open }`. `false` by default (byte-identical); `true` after the palette "About" command (or the macOS menu bar's App ▸ "About Awl") opens it. Shares the HUD's float-card pipeline (`about.rs` + `render/chrome.rs::prepare_hud`) rather than owning a parallel one |
| `line_count`   | total logical lines in the buffer |
| `scroll_lines` | top visual-row anchor (0 on load; retained for row-oriented diagnostics) |
| `scroll_px` | semantic offset within `scroll_lines`, reported in pixels |
| `scroll_top_px` | rendered document offset in pixels; this is the geometry the PNG obeys |
| `cursor`       | caret position, 0-based line and column (in chars) |
| `selection`    | the active selection region, or `null` when there is none |
| `text`         | the complete buffer contents (JSON-escaped) |
| `first_lines`  | the first up-to-12 logical lines, in order, for quick checks |
| `layout`       | SHAPED-FRAME LAYOUT oracle (schema `/187`): `{ rows, caret, selection }`. Rows are in draw order and carry raw `content`, source `line`, half-open `start_col`/`end_col`, absolute physical-pixel `xs` boundaries, `top`, and shaped `height`. `caret.row` and each selection segment's `row` index directly into that array. Borrowed from the exact sealed frame partition; never recomputed. It proves geometry, not pixel visibility or contrast |
| `search`       | isearch + find/replace state: `query`, `active`, `case_sensitive`, `hit_count`, `current`, `replace_active` (replace field revealed), `replacement` (replace text), plus `panel` — the card's PLANNED geometry (schema `/203`, see the narrative above), `null` while the panel is down |
| `focused_field` | Live-App field projection: `field`, rendered `text`, char-index `caret`, `selection`, transient `preedit` range, and prepared physical `candidate_rect`. Null without a focused field. Session identities are omitted. Rust App-event laws drive IME; CLI chords do not synthesize physical composition. |
| `project`      | active project (`--root`), fields `root`/`name`/`branch`/`dirty`/`default_folder`/`workspace`/`keymap_flavor` (`branch`, `default_folder`, `workspace` may be null); `null` when no project. The three path fields are HOME-RELATIVE (`~/…`, see "Paths are home-relative" above) — expand `~` if you need a real path |
| `overlay`      | summoned nav overlay: `active`, `mode` (`goto`/`switch`/`project_browse`/`browse`/`theme`/`caret`/`dictionary`/`cjk_lang`/`date`/`keymap`/`move`/`command`/`spell`/`keybindings`/`history`/`conflict`/`credits`/`settings`/`assets`/`rename`/`insert_link`/`keep_version`/`context`/`export_dest`/`table_dims`/`search_folder`/`user_words`), `query`, `query_caret` (the field's own char-index caret — schema `/209`'s own note, above, has the mid-query motion rule), `settings_focus` (`"categories"`/`"search"`/`"controls"` for Settings, else null), `selected_index`, `browse_dir` (the level shown: root-relative for `browse`/`move`, ABSOLUTE for `switch` and the `project_browse` navigator — home-relative `~/…` when it falls under `$HOME` — else null), `items` (dirs trailing `/`, a git child tagged `"git"` in the secondary column rather than bulleted; `switch` pins the accept-this-folder row on top, reading `use this folder — <name>`; command names for `command`; the three variant labels for `dictionary`; native/emacs labels for `keymap`), `bindings` (command-palette key chords parallel to `items`; the caret/dictionary/keymap pickers' one-line descriptions; else `[]`) |
| `buffers`      | MULTI-BUFFER registry + the VISIBLE WORKING SET: `{ open, active, files, active_index }`. `open` = how many buffers are currently open (the active one + everything backgrounded); `active` = the active buffer's path, `"scratch"`, `"untitled"` for an unnamed fresh buffer, or `null` with no document. A plain `--screenshot` always reports `open: 1`. `files` = one full root-relative label per drawn stack row, in stable open order (`[]` whenever the margin draws no stack — a single open file, the zero-document state, or any capture door with no `App` to ask); simultaneous fresh rows read `untitled`, `untitled 2`, … only as needed to distinguish them. `active_index` = which row is the reader's current file, or `null` |
| `replay_skips` | permissive `--keys` truthfulness record, always an array. Each skipped live-App-only effect is `{ effect, action }` in replay order: `effect` is the stable snake_case effect name and `action` is the resolved originating action name. Empty for a capture with no skipped effect. `--strict-replay` aborts before writing an artifact on any such effect, so it never emits a partial list. |

Schema `/218` adds `overlay.theme_actions`: physical Switch and Cancel hit
rectangles on the Theme chooser, null elsewhere. Schema `/219` adds the
`focused_field` projection described above. These report prepared rendering;
physical OS composition and candidate-window delivery remain live checks.
