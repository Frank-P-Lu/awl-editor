# Interaction field reference

Read for picker, HUD, buffer, and navigation details. Versioned entries describe schema evolution; use CAPTURE.md for the current schema reservation.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

**Dictionary picker** (Cmd-P → "Dictionary"; `overlay.mode == "dictionary"`) —
UNLIKE the theme/caret pickers it has **no live preview** as the selection
moves (a dictionary re-parse is a real one-time cost, not a per-keystroke one),
so `dictionary` only changes on `Enter`, never on a bare `--keys "... Down"`.
Drive it with `--keys "Cmd-P d i c t Enter Down Down Enter"` (opens the palette,
filters to "Dictionary", opens the picker, selects "English (Australia)",
commits) and assert `dictionary == "en_AU"`; a `--config` file with
`dictionary = "en_AU"` produces the same effective variant with no flags at
all (`apply_sticky_globals`, mirroring `theme`/`caret_mode`).

**CAPTURE IS STRUCTURALLY FREE OF REMEMBERED CONTEXT.** The one
active-folder-context owner is the live App's session (native-only,
`app/session.rs`) — a headless capture never constructs an `App`, so it never
reads or writes it (the capture-gate law). A **bare** capture — no `file`
argument AND no explicit `--root` flag — resolves `project.root` to **cwd**,
always, regardless of what a `--config` file's `default_folder` key names —
`default_folder` is a WINDOWED-launch-only, first-run fallback
(see the launch-precedence law in `docs/platform.md`), never consulted by a
capture. An explicit `--root` still wins outright; supplying a `file` argument
keeps deriving from that file's own directory, unaffected. Verify with a
seeded config: `cargo run -- --config /path/cfg.toml --screenshot OUT.png`
(no file, no `--root`, run from a known cwd) and assert `project.root` equals
that cwd, NOT the config's `default_folder`.

Schema `awl-capture/40` (was `/37`; timeline `/41`, held `/42`) adds the top-level
`hud` block for the SUMMONED-WHILE-HELD stats HUD — a calm centered metadata panel
shown WHILE a key is HELD (default **Cmd-I**, rebindable as `stats_hud`) and dismissed
on release (the game-map "hold to peek" affordance). It is `{ "held": bool,
"file_created": "...", "session": "...", "words": N|null, "reading_min": M|null,
"percent": P }`. `held` is the summon state — `false` on a default `--screenshot` (so
the scrim/card/text draw nothing and the frame is **byte-identical**), `true` under the
`--hud` flag or a `--keys "Cmd-I"` replay (the SETTLED held render: a dim scrim + a
`base_300` card carrying the stats). The figures mirror the rendered panel with the
SAME placeholder rules so the sidecar agrees with the pixels: `file_created` is the
file's `YYYY-MM-DD` created date LIVE, or `"unsaved"` for a scratch buffer, or the fixed
placeholder `"—"` for a saved file in a CAPTURE (the capture never reads a file's date,
so the sidecar stays byte-stable across machines); `session` is the live elapsed time
LIVE, the fixed `"—"` placeholder in a capture (no clock — like the fps counter);
`words`/`reading_min` are the markdown word-count + reading-time (`null` for a
non-markdown buffer, which OMITS that stat); and `percent` is the cursor's deterministic
%-through-doc (shown in a capture). So the only fields that ever carry a live value are
clock / filesystem ones, and those are always placeholdered in a capture.

Schema `awl-capture/37` (was `/36`; timeline `/38`, held `/39`) adds two top-level
fields for the chrome TYPE-SYSTEM pass: a `gutter` block and a `dim_overlay`
boolean. `gutter` is the page-mode ORIENTATION GUTTER (a quiet stacked label in the
LEFT margin — the filename in MUTED ink over the project in FAINT ink, both at the
smaller LABEL size): `{ "visible": bool, "name": "...", "project": "..." }`.
`visible` is `true` EXACTLY when the gutter is drawn — page mode ON, a buffer name,
and a wide-enough margin — so it agrees with the pixels; HIDDEN (edge-to-edge / no
name / narrow margin) is `{ "visible": false, "name": "", "project": "" }`, keeping
a non-page capture stable. `name` is the buffer's display name — the bound file's
name for a saved file, or the derived `<slug>.md` / `"scratch"` placeholder for an
unsaved note. `dim_overlay` is `true` when a FULL-takeover overlay (command palette,
go-to, theme picker, keybindings, spell picker, …) is up and the document is DIMMED
behind it by the translucent scrim, and `false` for the search SPLIT panel / no
overlay — the doc stays bright there (DESIGN §5). The same bump REMOVED the
always-on bottom-right word-count readout from the rendered chrome (it moves into the
held HUD); the `readout` block stays in the sidecar (a pure function of the text,
the HUD's source).

Schema `awl-capture/33` (was `/30`; timeline `/34`, held `/35`) extends the
`overlay` block with the REBIND MENU (`keybindings` mode): a `notice` string (a
transient conflict / "saved …" / "reset …" line) and a `capture` sub-block, `null`
unless a rebind capture is in progress. While capturing, `capture` is
`{ "command", "stage", "chord_mode", "captured", "prompt" }` — `stage` is
`"choose"` (KEY vs CHORD) / `"recording"` / `"confirm"`, `chord_mode` is true for a
multi-press sequence, and `captured` is the combos pressed so far (each a canonical
chord spec). Both fields are absent (`notice: ""`, `capture: null`) for every other
overlay mode, so the baseline overlay block is unchanged.

Schema `awl-capture/27` (was `/24`; timeline `/28`, held `/29`) adds the
`syn_spans` block (SYNTAX HIGHLIGHTING — the Alabaster four-role code styling). It
is an array of `[start_byte, end_byte, "tag"]` triples over the document `text`,
one per styled span the capture rendered — `tag` is one of `comment`, `string`,
`constant`, `definition` (the ONLY four roles awl colors; everything else stays
the default ink). The array is **empty for a non-CODE buffer** (gated by
`Buffer::syntax_lang` → `syntax::Lang::from_path`, which excludes `.env`, `.md`/
`.markdown`, `.txt`, and any unrecognized/scratch buffer), so a `.md`/`.txt`
capture is byte-stable. Markdown and syntax are mutually exclusive, so at most one
of `md_spans` / `syn_spans` is ever non-empty. Deterministic (a pure function of
the text + language). Present on every path. Example assertion: a Rust `// foo`
line yields a `comment` span over the comment, and `fn bar` yields a `definition`
span over `bar`. Every `syntax::Lang` variant carries a real lexer;
`Lang::from_extension` is the gate, and an unrecognized extension yields no spans
at all rather than a stubbed language. The companion **`syn_lang`** field reports the DETECTED language
name (`"rust"`, `"go"`, …) — or `null` for a non-CODE buffer — so the sidecar says
WHICH language produced the `syn_spans` rather than leaving it implicit; it is
gated by the same `Buffer::syntax_lang` so `syn_lang` and `syn_spans` always agree
(`null` ⇔ empty array).

Schema `awl-capture/24` (was `/21`; timeline `/25`, held `/26`) adds two FIND +
REPLACE fields to the `search` block: `replace_active` (`true` once the replace
field has been revealed on the search panel — a MODE of the same card, bound to
Cmd-Option-F / Tab) and `replacement` (the replace field's text). `--keys`
replays set `replace_active` headlessly: `s-M-f` (Cmd-Option-F) or `Cmd-r`
opens the panel straight into replace mode, OR — with a panel already open — a
single bare `Tab` toggles the replace field on, mirroring the live single-key
affordance. (Historical note: at this schema's introduction the replacement
could not be typed in a replay — the "isearch-input gap" — so it stayed `""`;
the shared search-key seam later retired that gap, and a replay now fills
`replacement`/`editing_replacement` exactly like live typing — see
"Search/replace is fully drivable" above.) Both are present
on every path (`false` / `""` for a non-search capture), so a plain `--screenshot`
stays byte-stable apart from the two new keys.

Schema `awl-capture/21` (was `/18`; timeline `/22`, held `/23`) adds the `readout`
block (the QUIET word-count / reading-time readout) and three new `md_spans` tags
for task lists + rules. The `readout` is `{ "words": N, "reading_min": M }` — the
exact figures the bottom-right readout shows (`M` = `ceil(words / 200)`, floored at
1) — or `null` when nothing is drawn (a non-markdown OR wordless buffer), so a plain
non-markdown capture stays byte-stable. Present on every path; pure function of the
text. New `md_spans` tags: `task_open` (an unchecked `[ ]` checkbox — rides full ink,
present), `task_checked` (a checked `[x]` checkbox — dim), `task_done` (a CHECKED
task's body text — dim, so the line recedes), and `rule` (a `---`/`***`/`___`
thematic-break line — dim; the renderer also draws a thin centered rule quad over the
row). A setext `---` heading underline is NOT a `rule`.

Schema `awl-capture/30` (was `/27`; timeline `/31`, held `/32`) adds the `fps`
block for the opt-in DEBUG frame counter: `{ "enabled": bool, "text": "<string>" }`.
The counter is **OFF by default**, so a plain `--screenshot` is `{ "enabled": false,
"text": "" }` and BYTE-IDENTICAL (nothing is drawn). Enable it with the `--fps`
flag (or drive `--keys "C-x r"` / the palette "Toggle FPS") — the capture has no
clock, so `text` is then the FIXED, numberless placeholder `"fps · — ms"` (a real
`<n> fps · <ms> ms` reading only ever appears in a live window). `text` is exactly
what the dim top-left corner draws, so the toggle is assertable from the sidecar
without eyeballing the PNG.

Schema `awl-capture/18` (was `/17`; timeline `/19`, held `/20`) adds the `md_spans`
block (MARKDOWN STYLING). It is an array of `[start_byte, end_byte, "tag"]` triples
over the document `text`, one per styled span the capture rendered — `tag` is one of
`markup` (syntax that recedes to the dim ink), `h1`..`h6`, `bold`, `italic`,
`bold_italic`, `code`, `quote`, `list_marker`, `link_text` (plus the task/rule tags
above as of `/21`). The array is **empty for a non-markdown buffer** (gated by the
`.md`/`.markdown` extension), so a `.rs`/`.txt` capture is byte-stable. Deterministic
(a pure function of the text). Present on every path. Example assertion: a `# Title`
line yields a `markup` span over `# ` and an `h1` span over `Title`.

Schema `awl-capture/17` (was `/8`; timeline `/18`, held `/19`) adds `notes_root` +
`workspace` to the `project` block — the EFFECTIVE config folders (flag > config >
default), so a `--config <path>` launch's configured folders are verifiable with no
flags. Both are JSON `null` on the timeline/held paths. The CONFIG SYSTEM
(`config.rs`) also surfaces rebinds in `overlay.bindings` for the command palette.

Schema `awl-capture/9` is emitted ONLY by `--capture-timeline` frames: it appends
the per-step `caret` block (`t_ms`, animated `pos`, `target`, `settle_factor`,
`animating`) documented under "Deterministic timeline capture" above. Every other
capture path (including a plain `--screenshot`) stays at `/8` with no `caret`
block, so its sidecar is byte-unchanged.

Schema `awl-capture/8` (was `/7`) adds the `focus` block (FOCUS MODE: the iA-Writer
dim-everything-but-here render). `focus.mode` is `off` | `paragraph` | `sentence`;
`active_start` / `active_end` are the CHAR offsets of the active unit rendered at
full ink (the rest dimmed), or `null` when focus is `off`. The capture renders the
SETTLED state (active full, surroundings dim) — the brighten/dim crossfade is
live-only, so the frame stays deterministic and has no clock. Focus is set
headlessly with `--focus off|paragraph|sentence` (or driven via `--keys "C-x d"`,
which cycles Off → Paragraph → Sentence); the `C-x d` chord / "Focus mode" palette
entry cycle it live. `focus.mode` is `off` (range `null`) for a plain
`--screenshot`, so the baseline shape is stable. The `focus` block was added to BOTH
the plain (`/7`→`/8`) and timeline (`/8`→`/9`) paths in lockstep, keeping the two
sidecar shapes distinct.

Schema `awl-capture/7` (was `/6`) adds the `page` block (PAGE MODE: the centered,
measure-capped writing column + the active world's margin gradient) and makes
`text_origin.left` TRUTHFUL — it now reports the actual column left (centered in
page mode), not the fixed `16.0` const. `page.on` is `true` by default; the column
`left`/`width` are pixels, and `gradient` carries the world's margin `from`/`to`
hexes + `dir` vector. Page mode is set headlessly with `--page on|off` and the
column width with `--measure N` (chars; implies `--page on`); the `C-x w` chord /
"Toggle page mode" palette entry flip it live. At the default `--measure 80` the
column is ~1152px on the 1200px canvas (tiny margins); use a NARROW measure (e.g.
`--measure 40` → 576px column, ~312px margins each side) to make the gradient
margins clearly visible in a capture.

Schema `awl-capture/6` (was `/5`) adds the `overlay.bindings` array — the COMMAND
PALETTE's per-row key-chord labels, parallel to `items` (empty `[]` for every
other mode). Schema `/5` (was `/4`) added the `project` block (the active project
root resolved from `--root`: `root`, `name`, `branch`, `dirty` — all read-only)
and the `overlay` block (the summoned navigation overlay: `active`, `mode`,
`query`, `selected_index`, `browse_dir`, `items`, `bindings`). `project` is `null`
and `overlay.active` is `false` for a plain `--screenshot`, so the baseline is
unchanged. A `--keys` replay can open the overlay, type to filter, move the
selection (`Down`/`C-n`), and `Enter` to act — all reflected here, so the whole
flow is verifiable from the sidecar.

The modes walked in detail below are the ones this document teaches; `mode`'s
full value set is in the sidecar-field table, owned by `OverlayKind::as_str` and
held to it by `capture::tests::capture_md_drift`. All ride the one transient card:

* `goto` (`C-x C-f`) — the active project's flat file index; `Enter` opens the
  highlighted file.
* `switch` (`C-x p`) — the FLAT project picker. It shows ONE directory level, the
  `--workspace` dir, so `browse_dir` is that absolute directory for the whole life
  of the card (never `null` while open). `items` lists its child FOLDERS only (all with a
  trailing `/`; a git child carries a `"git"` SECONDARY-column tag, never a name
  bullet), with a synthetic accept-this-folder row PINNED
  at the top — it carries `"."` as its accept string and READS as `use this
  folder — <name>`, naming the directory `browse_dir` points at — meaning "use
  THIS folder as the project root" and the `Browse for
  folder…` DOOR row pinned at the BOTTOM. The initial selection lands on the first
  real folder. `Enter` SELECTS the highlighted folder as the new root — it does NOT
  descend (set_root → re-index, recompute branch/dirty) and closes; the new root
  shows in the sidecar `project` block. `Left`/`Right` cycle the lens strip (All /
  Recent), not depth, and `Backspace` with an empty query is INERT: this card
  cannot leave the workspace. Anything deeper is the door below. A faint hint line
  at the card foot spells the model out: `type to filter  ↵ select  ←/→ lens`
  (mirrored in the sidecar `overlay.hint`).
* `project_browse` — the DOOR the `switch` picker's last row opens (`↵` on `Browse
  for folder…`), for a project that is not a direct workspace child. A folders-only
  navigator walking by ABSOLUTE path with the destination-picker grammar: `Right` /
  `C-f` DESCENDS into the highlighted folder, `Left` / `C-b` / `Backspace` ASCENDS,
  and `Enter` switches to the folder you stopped on (falling back to the level
  itself). The workspace is its FLOOR — an ascend at the top level stands still.
  It is a DESCEND, so `overlay.return_to` reads `"switch"` while it is up and
  `Escape` resumes the picker on its door row rather than closing to the document.
  Hint: `type to filter  ↵ switch here  → open  ← up`.
* `browse` (`C-x j`) — ONE directory level of the active root at a time.
  `browse_dir` is the root-relative level shown (`null` = the root). `items` lists
  directories first (each with a trailing `/`, git repos also `• `-marked) then
  files. `Right` or `Enter` on a folder DESCENDS (the list becomes that folder's
  children, `browse_dir` updates); `Left` or `Backspace` ASCENDS one level;
  `Enter` on a file opens it and closes. It is summoned + transient — it vanishes
  on open/cancel, never a tree.
* `theme` (`C-x t`) — the twenty worlds, fuzzy-filterable with live preview.
* `command` (`Cmd-P` / `s-p`) — the COMMAND PALETTE: a fuzzy search over every
  named command. `items` are the command display names (in catalog order) and the
  parallel `bindings` array gives each command's current key chord (shown dim,
  right-aligned beside the name in the card). `Enter` RUNS the selected command via
  its `Action` — so e.g. `s-p g o Enter` closes the palette and the `goto` overlay
  opens (the next captured `overlay.mode` is `goto`), `s-p` then a theme query +
  `Enter` opens the `theme` picker, and `Save`/`Quit` run directly. The catalog
  lives in `commands.rs` and is the seam the native-rebinding registry uses.
* `keybindings` (`Cmd-P` → "Keybindings") — the GAME-STYLE REBIND MENU: the same
  command list + `bindings` column as the palette, but `Enter` on a command starts a
  CAPTURE instead of running it. The capture flows through `overlay.capture` (see the
  schema note above): `Enter` → `choose` (KEY vs CHORD; `Up`/`Down` toggle, `Enter`
  picks) → `recording` (KEY finishes on the first press, CHORD collects up to the
  keymap's 2-deep limit then `Enter` finishes). A PLAIN-key press is `--keys`-drivable
  through the capture (`s-p k e y b RET u n d o RET RET q` rebinds Undo → `q`); a
  MODIFIED chord (`C-t` / `M-f`) is recorded LIVE in the window (a chord-level
  interception before keymap resolution — needs human confirmation). `Delete` on a
  command RESETS it to default; the captured binding is written to a `[keys]` SLOT
  (max 2, newest first), saved to `config.toml`, and live-reloaded (`overlay.notice`
  reflects the result; a CONFLICT moves the capture to `confirm` and warns before
  committing — live only). `Esc` cancels a capture / closes the menu.
* `move` (`C-x m`) — the MOVE-DESTINATION picker for the current file: the
  browse navigator over the **active folder** (the SAME root `browse`
  walks; no separate notes-root concept), listing FOLDERS only.
  `Right` DESCENDS into the highlighted folder, `Left` / `Backspace` ASCENDS,
  `Enter` ACCEPTS the
  destination — the highlighted folder, or, when the typed `query` matches no
  listed folder, a NEW folder of that name to create. `browse_dir` tracks the
  level (active-folder-relative; `null` = the active folder itself). The actual mkdir + move is
  applied live in the windowed app (App-only, so a `--keys` capture stays
  byte-deterministic and never mutates fixtures); the picker itself is fully
  drivable + verifiable here.

In every navigable explorer (`browse`/`move`/`export_dest`/`project_browse`)
`Backspace` doubles as "go to PARENT": with a non-empty fuzzy `query` it pops a
char (preserving the filter), and with an empty `query` it ASCENDS one level
exactly like `Left`. The flat `switch` picker is NOT one of them — it walks
nowhere, so its `Backspace` is inert on an empty query.
`browse_dir` is `null` for the `goto`/`theme`/`command` modes (and for the
`browse`/`move` ROOT level); for `switch` and `project_browse` it is the absolute
directory currently shown. `bindings` is `[]` for every mode except `command` and `keybindings`. The `C-x b`
last-buffer toggle and Cmd-N new-document swap are editor actions, not
overlays, so they leave no `overlay` trace — their effect shows in `text` /
`project` (after Cmd-N the buffer is a fresh, unnamed document IN THE SAME
active folder — no project change; the filename is derived from
its first line ONCE, on the first material save).

Schema `awl-capture/3` (was `/2`) adds the `theme` block describing the active
color world the frame was rendered with, and `font.family` reports that world's
display font (see `--theme` in `main.rs` and the twenty worlds in `theme/worlds.rs`).
Per-theme font switching is now **LIVE**: the document is actually shaped and
rendered in the world's face (mono / serif / sans / slab) via
`Family::Name(theme.font)` — not just recorded — so `font.family` /
`theme.font_family` name the family the rendered glyphs are really drawn with.
Proportional faces are fully supported: the caret tracks each glyph's real shaped
advance (no fixed mono cell), so it sits correctly over the glyph on every world.
(Historical, from the eight-world era.) Those worlds mapped onto five distinct faces: Tawny + Potoroo → IBM Plex Mono,
Gumtree + Saltpan → Literata, Bilby + Bombora → Newsreader, Quokka → IBM Plex
Sans, Mulga → Zilla Slab. (Historical note, schema `/40`-era: Tawny was the
DEFAULT world then, IBM Plex Mono, so a bare capture opened on awl's mono "home"
look. As of 2026-07-11 the DEFAULT is **Saltpan** — a warm light world, Fraunces
9pt serif — awl's first impression now; see `theme::DEFAULT_THEME`'s own doc
comment. Tawny stays one theme-cycle away and its own worked example below is
unchanged, since it's illustrating the SHAPE of the sidecar, not today's launch
world.)
