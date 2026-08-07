# awl — reference

Every command, every settings row, every `config.toml` key, every world, and
every markdown construct awl renders.

**Nothing in this file is transcribed by hand.** Each table is built from the
same roster the running app reads — the command catalog, the settings registry,
`Config`'s own fields, the theme roster, the markdown span kinds — by
`src/reference/`. `src/reference/law.rs` rebuilds every table on each test run
and fails if the checked-in text differs by a byte. A table is changed by
changing the code it describes and then running `scripts/regen-reference.sh`.

`GUIDE.md` explains how awl's pieces fit together and remains a separate
document. This file does not replace it.

- [Commands](#commands)
- [Settings](#settings)
- [Configuration file](#configuration-file)
- [Worlds](#worlds)
- [Markdown](#markdown)
- [Command line](#command-line)

---

## Commands

Every command carries up to two chord slots. Slot 1 is the platform-native
binding; slot 2 is an Emacs-style binding. Both fire. A cell showing two chords
shows both slots.

The chords below are the defaults. `[keys]` in `config.toml` replaces them, per
command, and is documented under [Configuration file](#configuration-file).

An empty macOS or Linux cell means the command has no default chord on that
platform and is reached from the command palette. The palette lists every
command by name.

Commands are grouped by the same task categories the palette browses.

<!-- GENERATED:reference-commands:BEGIN -->
### Files

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Switch project… | Summon the project switcher, browsing to a different project folder. | `⌘⇧P` | `Ctrl+Shift+P` | Native, browser |
| Recent projects… | Open the project switcher focused on its Recent list. | — | — | Native |
| Browse files… | Summon the file browser for the current project folder. | — | — | Native, browser |
| Version history… | Summon the version timeline — git log if tracked, saved snapshots otherwise. | `⌘⇧H` | `Ctrl+Shift+H` | Native |
| Compare with version… | Open the read-only prose diff comparing the current buffer against a past version. | — | — | Native |
| Keep version… | Prompt for a name, then record the buffer text as a pinned history snapshot under it. | — | — | Native |
| New document | Start a new, empty document in the current project folder. | `⌘N` | `Ctrl+N` | Native, browser |
| Keep tutorial… | Mark the tutorial to be saved once a folder is chosen, opening the project switcher. | — | — | Native |
| Move… | Summon the destination browser to move the current file to another folder. | — | — | Native, browser |
| Rename note… | Open the rename prompt, seeded with the current file's name. | — | — | Native, browser |
| Duplicate note | Save a copy of the file beside it, deduplicated, and switch to editing the copy. | — | — | Native, browser |
| Finish file | Save the file, notify any daemon `--wait` client, and switch to the prior file. | `⌘W` | `Ctrl+W` | Native |
| Download file | Download the buffer's text as a file — the web export, since there is no real disk. | — | — | Browser |
| Export as Word… | Export the buffer to a `.docx` file; markdown buffers only. | — | — | Native, browser |
| Export as HTML… | Export the buffer to an `.html` file; markdown buffers only. | — | — | Native, browser |
| Export as PDF… | Export the buffer to a `.pdf` file; markdown buffers only, native builds only. | — | — | Native |
| Save | Save the buffer to disk. | `⌘S` | `Ctrl+S` | Native, browser |
| Review the change | Show an unresolved change: differences, your version, disk version. Changes nothing. | — | — | Native |
| Save your version | Settle an unresolved external change by writing the buffer over the file on disk. | — | — | Native |
| Use disk version | Settle an unresolved change by replacing the buffer with the disk file, as one edit. | — | — | Native |
| Quit | Quit the application. | `⌘Q` | `Ctrl+Q` | Native |

### Navigate

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Go to file… | Summon the fuzzy file finder for the current project. | `⌘O` | `Ctrl+O` | Native, browser |
| Go to heading… | Open the file finder pre-focused on the current document's headings. | — | — | Native, browser |
| Last file | Switch to the previously open file; a no-op with nothing to switch back to. | `⌃Tab` | `Ctrl+Tab` | Native, browser |
| Follow link | Open the caret's markdown link URL in the default browser, if there is one. | `C-c C-o` | — | Native, browser |
| Copy link destination | Copy the URL of the markdown link under the caret to the kill buffer. | — | — | Native, browser |
| Search forward | Open incremental search (prefilled from selection or last query), forward. | `⌘F · C-s` | `Ctrl+F` | Native, browser |
| Search backward | Open incremental search (prefilled from selection or last query), backward. | `⌘⇧F · C-r` | `Ctrl+Shift+F` | Native, browser |
| Find and replace… | Open the search panel with its replace row revealed. | `⌘R` | `Ctrl+R` | Native, browser |
| Forward word | Move the caret forward one word. | `⌥Right` | `Alt+Right` | Native, browser |
| Backward word | Move the caret backward one word. | `⌥Left` | `Alt+Left` | Native, browser |
| Line start | Move the caret to the start of the visual line (logical without an oracle). | `⌘Left · C-a` | `Home` | Native, browser |
| Line end | Move the caret to the end of the visual line (logical line without a layout oracle). | `⌘Right · C-e` | `End` | Native, browser |
| Document start | Move the caret to the start of the document. | `⌘Up` | `Ctrl+Home` | Native, browser |
| Document end | Move the caret to the end of the document. | `⌘Down` | `Ctrl+End` | Native, browser |
| Forward char | Move the caret forward one character. | `C-f` | — | Native, browser |
| Backward char | Move the caret backward one character. | `C-b` | — | Native, browser |
| Next line | Move the caret down one visual line, following soft wraps and a sticky goal column. | `C-n` | — | Native, browser |
| Previous line | Move the caret up one visual line, following soft wraps and a sticky goal column. | `C-p` | — | Native, browser |
| Delete word forward | Delete the word or punctuation run after the caret; a selection deletes instead. | — | — | Native, browser |
| Delete word backward | Delete the word or punctuation run before the caret; a selection deletes instead. | — | — | Native, browser |

### Format

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Align table | Re-pad the GFM table under the caret so its `\|` columns line up. | — | — | Native, browser |
| Insert Date | Insert today's date at the caret, in the configured date format. | `⌘⇧D · C-c .` | `Ctrl+Shift+D` | Native, browser |
| Blockquote | Toggle a `> ` blockquote prefix on the caret line or each line of the selection. | — | — | Native, browser |
| Bullet list | Toggle a `- ` bullet marker on the caret line or each line of the selection. | — | — | Native, browser |
| Numbered list | Toggle a numbered-list marker on the line or selection, renumbering sequentially. | — | — | Native, browser |
| Task list | Toggle a `- [ ] ` task checkbox on the caret line or each line of the selection. | `⌘⇧L` | `Ctrl+Shift+L` | Native, browser |
| Heading | Toggle a level-1 `# ` heading marker on the caret line. | — | — | Native, browser |
| Cycle heading | Cycle the caret line's heading level 1 → 2 → 3 → plain text. | — | — | Native, browser |
| Code block | Wrap the caret line or selection in a fenced code block, unwrapping if fenced. | — | — | Native, browser |
| Bold | Toggle `**bold**` markup around the selection or the word at the caret. | `⌘B` | `Ctrl+B` | Native, browser |
| Italic | Toggle `*italic*` markup around the selection or the word at the caret. | `⌘I` | `Ctrl+I` | Native, browser |
| Inline code | Toggle `` `inline code` `` markup around the selection or the word at the caret. | `⌘E` | `Ctrl+E` | Native, browser |
| Highlight | Toggle `==highlight==` markup around the selection or the word at the caret. | — | — | Native, browser |
| Strikethrough | Toggle `~~strikethrough~~` markup around the selection or the word at the caret. | — | — | Native, browser |
| Insert link… | Summon the URL prompt for a markdown link: wrap, edit, or insert a link at the caret. | `⌘K` | — | Native, browser |
| Undo | Undo the last edit group. | `⌘Z · C-/` | `Ctrl+Z · C-/` | Native, browser |
| Redo | Redo the last undone edit group. | `⌘⇧Z` | `Ctrl+Shift+Z` | Native, browser |
| Copy | Copy the selection to the kill buffer, leaving the text and clearing the mark. | `⌘C` | `Ctrl+C` | Native, browser |
| Cut | Cut the selection into the kill buffer and remove it from the buffer. | `⌘X · C-w` | `Ctrl+X` | Native, browser |
| Paste | Insert the OS clipboard's content — an image reference if it holds one, else text. | `⌘V · C-y` | `Ctrl+V · C-y` | Native, browser |
| Select all | Select the entire buffer. | `⌘A` | `Ctrl+A` | Native, browser |

### View

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Switch theme… | Summon the theme (world) picker. | `⌘T` | `Ctrl+T` | Native, browser |
| Toggle page mode | Toggle between the centered writing column and full window width. | — | — | Native, browser |
| Widen page | Widen the page column by one step. | — | — | Native, browser |
| Narrow page | Narrow the page column by one step. | — | — | Native, browser |
| Reset page width | Reset the page column to the buffer's default width, clearing any override. | — | — | Native, browser |
| Toggle debug | Toggle the debug overlay. | — | — | Native, browser |
| Toggle outline | Toggle the heading outline panel. | `⌘⇧O` | `Ctrl+Shift+O` | Native, browser |
| Fold section | Toggle collapse of the section under the caret; view state, not on the undo timeline. | `⌘⇧E · C-c C-f` | `Ctrl+Shift+E` | Native, browser |
| Collapse other sections | Collapse every markdown section except the one under the caret. | `⌘⇧M · C-c C-t` | `Ctrl+Shift+M` | Native, browser |
| Toggle typewriter scroll | Toggle keeping the caret vertically centered as you type. | — | — | Native, browser |
| Toggle menu bar | Toggle the menu bar's visibility. | — | — | Native, browser |
| Zoom in | Step the editor's zoom level up. | `⌘=` | `Ctrl+=` | Native, browser |
| Zoom out | Step the editor's zoom level down. | `⌘-` | `Ctrl+-` | Native, browser |
| Reset zoom | Reset the editor's zoom level to its default. | `⌘0` | `Ctrl+0` | Native, browser |

### Tools

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Spell suggestions… | Summon spelling suggestions for the misspelled word at the caret. | `⌘;` | `Ctrl+;` | Native, browser |
| Clean unused assets… | Summon the list of orphaned image files under the project, for moving to the trash. | — | — | Native |
| About | Show the About panel. | — | — | Native, browser |
| Credits | Open the bundled Credits document into the buffer. | — | — | Native, browser |
| Guide | Open the bundled Guide document into the buffer. | — | — | Native, browser |
| Reference | Open the bundled Reference document into the buffer. | — | — | Native, browser |
| Lifetime stats | Open the lifetime writing statistics panel. | — | — | Native |
| Writing streaks | Open the writing-streaks panel (per-day heatmap and cumulative total). | — | — | Native |
| Line endings… | Toggle the file's on-disk line ending between LF and CRLF; not on the undo timeline. | — | — | Native, browser |
| Report a Problem | Compose a `mailto:` bug report, attaching the newest crash log's path if one exists. | — | — | Native, browser |
| Check for Updates | Record a last-checked marker and open the site's version-check page in the browser. | — | — | Native |

### Settings

| Command | What it does | macOS | Linux | Builds |
|---|---|---|---|---|
| Caret style… | Summon the caret style picker. | — | — | Native, browser |
| Dictionary… | Summon the spelling dictionary picker. | — | — | Native, browser |
| Toggle spellcheck | Flip spellcheck on or off globally, silencing every squiggle when off. | — | — | Native, browser |
| Toggle caret style | Cycle to the next caret style. | — | — | Native, browser |
| Toggle writing nits | Toggle the writing-nits style underlines on or off. | — | — | Native, browser |
| Settings… | Summon the settings picker. | `⌘,` | `Ctrl+,` | Native, browser |
| Keybindings… | Summon the keybindings rebind menu. | — | — | Native, browser |

### Chords with no command

These two are matched by the keymap directly and cannot be rebound.

| Chord for | macOS | Linux |
|---|---|---|
| Command palette | `⌘P` | `Ctrl+P` |
| Held stats HUD | `⌘⌥I` | `Ctrl+Alt+I` |
<!-- GENERATED:reference-commands:END -->

---

## Settings

The Settings overlay edits the rows below. A row with a `config.toml` key
persists to that key on change; a row without one does not persist.

<!-- GENERATED:reference-settings:BEGIN -->
| Setting | Group | Control | config.toml key |
|---|---|---|---|
| Caret style | Editor | Opens a picker | — |
| Page mode | Editor | On/off | `page_mode` |
| Typewriter scroll | Editor | On/off | `typewriter_scroll` |
| Reduce motion | Editor | On/off | `reduce_motion` |
| Page width (prose) | Editor | Numeric rail | `page_width_prose` |
| Page width (code) | Editor | Numeric rail | `page_width_code` |
| Zoom | Editor | Numeric rail | `zoom` |
| Scroll sensitivity | Editor | Numeric rail | `scroll_sensitivity` |
| Date format | Editor | Opens a picker | — |
| Theme | Appearance | Opens a picker | — |
| WYSIWYG | Appearance | On/off | `wysiwyg` |
| Format popover | Appearance | On/off | `popover` |
| Inline images | Appearance | On/off | `inline_images` |
| Code ligatures | Appearance | On/off | `code_ligatures` |
| Outline | Appearance | On/off | `outline` |
| Menu bar | Appearance | On/off | `menu_bar` |
| Spellcheck | Writing | On/off | `spellcheck` |
| Dictionary | Writing | Opens a picker | — |
| Writing nits | Writing | On/off | `writing_nits` |
| Ambiguous CJK reads as | Writing | Opens a picker | — |
| Default folder | Files | Picks a folder | `default_folder` |
| Projects folder | Files | Picks a folder | `workspace` |
| Project root | Files | Picks a folder | — |
| File visibility | Files | On/off | `file_visibility` |
| Autosave | Files | On/off | `autosave` |
| Local history | Files | On/off | `history` |
| Session restore | Files | On/off | `session_restore` |
| Keymap | Keybindings | On/off | `keymap` |
| Keybindings | Keybindings | Opens a submenu | — |
| Report a Problem | Advanced | Runs a command | — |
| Edit config as text | Advanced | Runs a command | — |
<!-- GENERATED:reference-settings:END -->

---

## Configuration file

The config file is TOML at `$XDG_CONFIG_HOME/awl/config.toml`, or
`~/.config/awl/config.toml` when `XDG_CONFIG_HOME` is unset. awl opens it as
ordinary text; saving it reloads it live.

Precedence is command-line flag, then file, then default. An absent file and an
absent key behave identically.

<!-- GENERATED:reference-config:BEGIN -->
### Keys

An absent key takes the default below. A command-line flag overrides the file; the file overrides the default.

| Key | Value | Default |
|---|---|---|
| `default_folder` | path | — |
| `workspace` | path | — |
| `theme` | world name | `Saltpan` |
| `zoom` | percent | `100%` |
| `scroll_sensitivity` | percent | `100%` |
| `page_mode` | true \| false | `true` |
| `page_width_prose` | whole columns | `70` |
| `page_width_code` | whole columns | `100` |
| `caret_mode` | block \| morph \| ibeam | `morph` |
| `dictionary` | en_US \| en_GB \| en_AU | `en_US` |
| `writing_nits` | true \| false | `true` |
| `spellcheck` | true \| false | `true` |
| `history` | true \| false | `true` |
| `autosave` | true \| false | `true` |
| `wysiwyg` | true \| false | `true` |
| `popover` | true \| false | `true` |
| `inline_images` | true \| false | `true` |
| `code_ligatures` | true \| false | `true` |
| `cjk_priority` | list of language codes | — |
| `session_restore` | true \| false | `true` |
| `outline` | true \| false | `true` |
| `menu_bar` | true \| false | false on macOS, true elsewhere |
| `typewriter_scroll` | true \| false | `false` |
| `file_visibility` | true \| false | `false` |
| `stats` | true \| false | `true` |
| `reduce_motion` | true \| false | `false` |
| `ambient_motion` | true \| false | `true` |
| `keymap` | native \| emacs | `native` |
| `date_format` | ddmmyy \| mmddyy \| iso \| yyyymmdd \| dmonthyyyy | `ddmmyy` |
| `keys` | table of chord lists | — |
| `linux_keep_emacs` | list of chords | — |

### Numeric bands

A value outside the band is clamped to it, then snapped to the step.

| Key | Minimum | Maximum | Step | Default |
|---|---|---|---|---|
| `zoom` | `50%` | `300%` | `10%` | `100%` |
| `scroll_sensitivity` | `25%` | `400%` | `5%` | `100%` |
| `page_width_prose` | `20` | `200` | `1` | `70` |
| `page_width_code` | `20` | `200` | `1` | `100` |
<!-- GENERATED:reference-config:END -->

### Rebinding

`[keys]` maps a command's slug to its chords. A slug is the command's name
lowercased, spaces replaced with `_`, and any trailing `…` dropped — `Go to
file…` is `go_to_file`.

```toml
[keys]
save         = ["Cmd-S", "C-x C-s"]   # slot 1 native, slot 2 emacs
switch_theme = "Cmd-T"                # a single chord binds slot 1 only
```

A chord is written as `Modifier-Key`, modifiers joined by `-`: `Cmd`, `C`
(control), `M` (alt/meta), `S` (shift). A multi-chord sequence is separated by a
space, as in `C-x C-s`. On Linux, `Cmd` in slot 1 resolves to Ctrl.

`linux_keep_emacs` lists chords that keep their Emacs meaning on Linux where a
native binding would otherwise displace them.

---

## Worlds

A world is a complete visual environment: ground, ink ladder, display face, mono
face, section-break ornament, and background. `THEMES.md` states the laws a world
must satisfy; `WORLDS.md` describes each world's flavour.

<!-- GENERATED:reference-worlds:BEGIN -->
The default world is Saltpan. `--list-worlds` prints this roster; `--theme <World>` selects one for a single run.

| World | Ground | Display face | Mono face |
|---|---|---|---|
| Tawny | Dark | IBM Plex Mono | IBM Plex Mono |
| Mopoke | Dark | Bitter | IBM Plex Mono |
| Currawong | Dark | Iosevka | Iosevka |
| Potoroo | Dark | Monaspace Xenon | Monaspace Xenon |
| Gumtree | Light | Literata | Monaspace Xenon |
| Bilby | Light | Newsreader 16pt 16pt | Monaspace Xenon |
| Saltpan | Light | Fraunces 9pt | Monaspace Xenon |
| Quokka | Light | Sour Gummy | IBM Plex Mono |
| Bombora | Dark | EB Garamond | Monaspace Xenon |
| Bowerbird | Dark | IBM Plex Sans | JetBrains Mono |
| Mulga | Dark | Zilla Slab | Monaspace Xenon |
| Mangrove | Dark | JetBrains Mono | JetBrains Mono |
| Galah | Light | Figtree | IBM Plex Mono |
| Magpie | Light | Bitter | Monaspace Xenon |
| Brolga | Light | IBM Plex Sans | IBM Plex Mono |
| Wagtail | Dark | JetBrains Mono | JetBrains Mono |
| Firetail | Dark | Monaspace Xenon | Monaspace Xenon |
| Cassowary | Dark | Iosevka | Iosevka |
| Paperbark | Light | EB Garamond | Monaspace Xenon |
| Kite | Light | Fira Sans | JetBrains Mono |
<!-- GENERATED:reference-worlds:END -->

---

## Markdown

The file on disk stays plain text. awl renders it live and shows raw markdown on
whichever line the caret is on.

<!-- GENERATED:reference-markdown:BEGIN -->
### Constructs

The file stays plain text. Only the render changes.

| Construct | Written as |
|---|---|
| Heading, levels 1–6 | `# Heading` |
| Bold | `**bold**` |
| Italic | `*italic*` |
| Bold italic | `***both***` |
| Inline code and code blocks | `` `code` `` |
| Syntax highlighting in a fenced block | ```` ```rust ```` |
| Blockquote | `> quoted` |
| List, bulleted or numbered | `- item` |
| Link | `[text](target)` |
| Task list | `- [ ] task` |
| Highlight | `==highlight==` |
| Strikethrough | `~~struck~~` |
| Thematic break | `---` |
| Table | `\| a \| b \|` |
| Syntax characters of every construct above | ``# * ` > [ ] \|`` |

### What hides off the caret

With `wysiwyg = true`, the markup below hides while the caret and the selection are elsewhere.

| Construct | Hidden markup | Revealed by | Reveals in place |
|---|---|---|---|
| Heading | The leading `#` run | The line | Yes |
| Bold and italic | The `*` or `_` delimiters | The line | Yes |
| Inline code | The backticks | The line | Yes |
| Highlight | The `==` delimiters | The line | Yes |
| Strikethrough | The `~~` delimiters | The line | Yes |
| Fenced code block | Both fence lines and the info string | The whole block | Yes |
| Frontmatter | The whole `---` block | The whole block | Yes |
| Table | The whole source, replaced by a drawn grid | The whole block | No |
| Image | The whole `![alt](path)` source | The line | Yes |
| Link | The brackets and the target | The line | Yes |
| Blockquote | The `>` marker | The line | Yes |
<!-- GENERATED:reference-markdown:END -->

---

## Command line

The browser build has no command line; these are the native binary's flags. awl
takes a file to open and, for headless verification, a capture mode plus the
hooks that compose with it.

Every flag below is one row of the same roster the parser resolves arguments
through, so this table and `awl --help` cannot disagree about a flag's spelling,
its arguments, or what it does.

<!-- GENERATED:reference-cli:BEGIN -->
### Capture modes

At most one capture mode per run: awl refuses a second rather than silently preferring one.

| Flag | Takes | What it does |
|---|---|---|
| `--screenshot` | `OUT.png` | caret at rest (rounded square) |
| `--screenshot-motion` | `OUT.png` | caret mid-glide (centred trailing streak) |
| `--screenshot-motion-v` | `OUT.png` | caret mid-glide vertical (left-edge bar) |
| `--screenshot-motion-d` | `OUT.png` | caret mid-glide diagonal (slanted tracer) |
| `--screenshot-app` | `OUT.png` | drive --keys into a REAL headless App (hermetic) and capture ITS state — the only door that sees a live-App-only transition; sidecar carries driver: "live-app" |
| `--capture-timeline` | `"0,16,50,150" OUT.png` | deterministic timeline: step the caret glide by injected ms, frame per step (OUT.t<ms>.png) |
| `--capture-held` | `DIR "0,30,60,90" OUT.png` | deterministic HELD arrow (DIR=left\|right\|up\|down): re-target one char/line per step (held=true), frame per step with trail geometry |

### Options

| Flag | Takes | What it does |
|---|---|---|
| `--sel` | `L0:C0-L1:C1` | selection highlight from (l0,c0)..(l1,c1) |
| `--zoom` | `F` | zoom factor (0.5..3.0) |
| `--scroll` | `N[:Q]` | scroll to row N plus Q fixed 1/64px units |
| `--preedit` | `STR` | render STR as an IME preedit at the caret |
| `--search` | `STR` | open isearch panel for STR + highlight hits |
| `--search-case` | — | make --search case-sensitive |
| `--theme` | `NAME` | set the active color theme (Tawny, Mopoke, Currawong, Potoroo, Gumtree, Bilby, Saltpan, Quokka, Bombora, Bowerbird, Mulga, Mangrove, Galah, Magpie, Brolga, Wagtail, Firetail, Cassowary, Paperbark, Kite) |
| `--list-worlds` | — | print every theme name, one per line, then exit (the roster `--theme` accepts; see scripts/capture-worlds.sh) |
| `--icon-manifest` | — | print the app-icon export manifest as JSON (per world: icon palette tokens + display face + its logo-cursor; per face: the bundled font files), then exit — run from the repo root; see scripts/icons/ |
| `--ground-audition` | `W` | item 121's A/B/C ground-audition manifest, exit |
| `--pack-icns` | `[DIR]` | cut every world's rendered tiles (default assets/macos/candidates/tiles) into assets/macos/world/<World>.icns + the canonical assets/macos/Awl.icns, and regenerate src/app_icon/embedded.rs, then exit — run from the repo root AFTER scripts/export-icons.sh |
| `--export-linux-icon` | `OUT.png` | cut the 256px PNG out of the committed canonical assets/macos/Awl.icns, then exit — run from the repo root; see scripts/package-appimage.sh |
| `--caret-mode` | `MODE` | caret look: block, morph, ibeam, or auto (default: mono->block, proportional->morph) |
| `--capture-size` | `WxH` | physical canvas size for the capture (default 1200x800) |
| `--capture-dpi` | `N` | renderer scale factor (default 1.0); WxH at dpi N == (W/N)x(H/N) logical retina window |
| `--measure` | `N` | page-mode column width in chars (default 70 for prose, 100 for code; implies --page on) |
| `--page` | `on\|off` | page mode: centered column (on, default) vs edge-to-edge (off) |
| `--debug` | — | DEBUG: draw the dim top-left dev panel — frametime/zoom/viewport/cursor/theme/md+syn (OFF by default; frametime is a fixed placeholder in a headless capture) |
| `--hud` | — | summon the HELD stats HUD (live: hold Option-Cmd-I; clock/file-date fields are fixed placeholders in a capture) |
| `--menu-bar` | — | show the web/Linux MENU BAR (default on web/Linux, off on macOS which has the native bar); --menu-open N drops menu N's dropdown |
| `--peek` | — | summon the HOLD-⌘ shortcut peek (live: hold the convention's bare arming modifier — ⌘ on Mac, Ctrl on Linux — ~600ms; a capture shows the curated starter six) |
| `--streaks` | — | summon the WRITING STREAKS card (live: palette "Writing streaks"; a capture shows a fixed synthetic year + streak numbers) |
| `--whichkey` | — | summon the WHICH-KEY panel: the C-x prefix's follow-up keys (live: press C-x and pause ~500ms) |
| `--default-folder` | `DIR` | fallback active folder for a first launch with nothing remembered (default ~/notes) |
| `--config` | `PATH` | load settings from PATH (default ~/.config/awl/config.toml) |
| `--wait` | — | windowed editor only: single-instance daemon — hand `file` to an already-running awl and block until C-x # finishes it (EDITOR=awl --wait for git) |
| `--keys` | `"SPEC"` | replay emacs chords (e.g. "C-n C-n M->") then capture |
| `--seed-data` | `DIR` | seed awl's own DATA ROOT (the unresolved-change record, the scratch stash, session.toml, history) into a hermetic scenario sandbox from DIR's files — the only way a --screenshot-app run can START from state awl already had; refused outside a hermetic door |
| `--strict-replay` | — | with --screenshot --keys: abort (naming the offender) on an unbound chord, a live-only effect the replay can't perform, or a missing layout oracle; runs HERMETIC (an in-memory fs seeded from the named file + --config — a replayed save never touches the real file, the user's own config/notes/history are never read or written) |
| `--storyboard` | `TOML` | run a scenario storyboard (press/type/pause/run_for/expect steps — see scenarios/): strict + hermetic, emitting per-step PNG+JSON, deterministic film frames, a byte-stable trace.json, and (with ffmpeg on PATH) film.webm/film.mp4 |
| `--storyboard-out` | `DIR` | where the storyboard run's artifacts land (default: <storyboard>.run/ beside the .toml) |

### Unlisted flags

`awl --help` does not print these. They work like any other flag; they are benchmark, diagnostic and verification hooks rather than everyday arguments. Every benchmark opens no window.

| Flag | Takes | What it does |
|---|---|---|
| `--screenshot-frames` | `N OUT.png` | capture N successive settled frames of the real App scheduling body, stepped --frame-step-ms per frame |
| `--semantic-json` | — | print the headless App's accessibility tree as JSON instead of a PNG |
| `--help, -h` | — | print the usage summary above and exit |
| `--frame-step-ms` | `MS` | milliseconds of virtual clock between --screenshot-frames frames |
| `--search-replace` | — | open the search panel's labelled replace row — the fresh Cmd-R state |
| `--menu-open` | `[N]` | show the menu bar and drop menu N's dropdown (0 = the App menu) |
| `--lifetime` | — | summon the LIFETIME STATS card (live: the palette's "Lifetime stats"; a capture renders fixed placeholders rather than a live store) |
| `--root` | `DIR` | the active project root: scopes the go-to overlay and fills the sidecar's project block (default: the launch file's parent, else the working directory) |
| `--workspace` | `DIR` | workspace parent whose child directories are the switch-project candidates |
| `--live-script` | `"STEPS"` | drive the REAL windowed app through a live-probe step script; refused alongside any capture mode |
| `--live-shots` | `DIR` | where --live-script writes its shots (default: the system temp directory) |
| `--bench-typing` | — | time the per-keystroke update path on 100/1000/5000-line documents, whole-buffer reshape against incremental |
| `--bench-perf` | — | time the traced hot paths over the long fixtures under benches/fixtures |
| `--bench-frame` | — | per-stage frame profile of the live redraw sequence over the real repo docs |
| `--bench-theme-burst` | — | profile successive font-changing theme switches — the reshape and the first frame after each — cold and warm |
| `--bench-a11y` | — | time one keystroke's accessibility projection at 100 to 50 000 lines, the whole-snapshot path against the incremental one |
| `--bench-zoom-burst` | — | replay a rapid adjacent-level zoom burst: eager reflow against latest-wins coalescing |
| `--bench-frost` | — | profile the frost field's steady frames and its rebuilds, for both lava worlds |
| `--bench-caret` | — | record the caret glyph lookup's cost at the document top, middle and tail |
| `--bench-suite` | — | run the unified bench suite — corpus tiers by interaction scenarios — printing a table and writing bench.json beside the invocation |
| `--bench-baseline` | `PATH` | diff --bench-suite against a machine-keyed baseline, exiting nonzero on a >20% cell |
| `--soak-gpu` | — | run the bounded native window/surface robustness probe, isolated from the daemon, session, history and user config |
| `--soak-gpu-seconds` | `SECONDS` | how long --soak-gpu runs, in seconds (default 900) |
<!-- GENERATED:reference-cli:END -->
