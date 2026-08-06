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

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Switch project… | `⌘⇧P` | `Ctrl+Shift+P` | Native, browser |
| Recent projects… | — | — | Native |
| Browse files… | — | — | Native, browser |
| Version history… | `⌘⇧H` | `Ctrl+Shift+H` | Native |
| Compare with version… | — | — | Native |
| Keep version… | — | — | Native |
| New document | `⌘N` | `Ctrl+N` | Native, browser |
| Keep tutorial… | — | — | Native |
| Move… | — | — | Native, browser |
| Rename note… | — | — | Native, browser |
| Duplicate note | — | — | Native, browser |
| Finish file | `⌘W` | `Ctrl+W` | Native |
| Download file | — | — | Browser |
| Export as Word… | — | — | Native, browser |
| Export as HTML… | — | — | Native, browser |
| Export as PDF… | — | — | Native |
| Save | `⌘S` | `Ctrl+S` | Native, browser |
| Review the change | — | — | Native |
| Save your version | — | — | Native |
| Use disk version | — | — | Native |
| Quit | `⌘Q` | `Ctrl+Q` | Native |

### Navigate

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Go to file… | `⌘O` | `Ctrl+O` | Native, browser |
| Go to heading… | — | — | Native, browser |
| Last file | `⌃Tab` | `Ctrl+Tab` | Native, browser |
| Follow link | `C-c C-o` | — | Native, browser |
| Copy link destination | — | — | Native, browser |
| Search forward | `⌘F · C-s` | `Ctrl+F` | Native, browser |
| Search backward | `⌘⇧F · C-r` | `Ctrl+Shift+F` | Native, browser |
| Find and replace… | `⌘R` | `Ctrl+R` | Native, browser |
| Forward word | `⌥Right` | `Alt+Right` | Native, browser |
| Backward word | `⌥Left` | `Alt+Left` | Native, browser |
| Line start | `⌘Left · C-a` | `Home` | Native, browser |
| Line end | `⌘Right · C-e` | `End` | Native, browser |
| Document start | `⌘Up` | `Ctrl+Home` | Native, browser |
| Document end | `⌘Down` | `Ctrl+End` | Native, browser |
| Forward char | `C-f` | — | Native, browser |
| Backward char | `C-b` | — | Native, browser |
| Next line | `C-n` | — | Native, browser |
| Previous line | `C-p` | — | Native, browser |
| Delete word forward | — | — | Native, browser |
| Delete word backward | — | — | Native, browser |

### Format

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Align table | — | — | Native, browser |
| Insert Date | `⌘⇧D · C-c .` | `Ctrl+Shift+D` | Native, browser |
| Blockquote | — | — | Native, browser |
| Bullet list | — | — | Native, browser |
| Numbered list | — | — | Native, browser |
| Task list | `⌘⇧L` | `Ctrl+Shift+L` | Native, browser |
| Heading | — | — | Native, browser |
| Cycle heading | — | — | Native, browser |
| Code block | — | — | Native, browser |
| Bold | `⌘B` | `Ctrl+B` | Native, browser |
| Italic | `⌘I` | `Ctrl+I` | Native, browser |
| Inline code | `⌘E` | `Ctrl+E` | Native, browser |
| Highlight | — | — | Native, browser |
| Strikethrough | — | — | Native, browser |
| Insert link… | `⌘K` | — | Native, browser |
| Undo | `⌘Z · C-/` | `Ctrl+Z · C-/` | Native, browser |
| Redo | `⌘⇧Z` | `Ctrl+Shift+Z` | Native, browser |
| Copy | `⌘C` | `Ctrl+C` | Native, browser |
| Cut | `⌘X · C-w` | `Ctrl+X` | Native, browser |
| Paste | `⌘V · C-y` | `Ctrl+V · C-y` | Native, browser |
| Select all | `⌘A` | `Ctrl+A` | Native, browser |

### View

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Switch theme… | `⌘T` | `Ctrl+T` | Native, browser |
| Toggle page mode | — | — | Native, browser |
| Widen page | — | — | Native, browser |
| Narrow page | — | — | Native, browser |
| Reset page width | — | — | Native, browser |
| Toggle debug | — | — | Native, browser |
| Toggle outline | `⌘⇧O` | `Ctrl+Shift+O` | Native, browser |
| Fold section | `⌘⇧E · C-c C-f` | `Ctrl+Shift+E` | Native, browser |
| Collapse other sections | `⌘⇧M · C-c C-t` | `Ctrl+Shift+M` | Native, browser |
| Toggle typewriter scroll | — | — | Native, browser |
| Toggle menu bar | — | — | Native, browser |
| Zoom in | `⌘=` | `Ctrl+=` | Native, browser |
| Zoom out | `⌘-` | `Ctrl+-` | Native, browser |
| Reset zoom | `⌘0` | `Ctrl+0` | Native, browser |

### Tools

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Spell suggestions… | `⌘;` | `Ctrl+;` | Native, browser |
| Clean unused assets… | — | — | Native |
| About | — | — | Native, browser |
| Credits | — | — | Native, browser |
| Guide | — | — | Native, browser |
| Reference | — | — | Native, browser |
| Lifetime stats | — | — | Native |
| Writing streaks | — | — | Native |
| Line endings… | — | — | Native, browser |
| Report a Problem | — | — | Native, browser |
| Check for Updates | — | — | Native |

### Settings

| Command | macOS | Linux | Builds |
|---|---|---|---|
| Caret style… | — | — | Native, browser |
| Dictionary… | — | — | Native, browser |
| Toggle spellcheck | — | — | Native, browser |
| Toggle caret style | — | — | Native, browser |
| Toggle writing nits | — | — | Native, browser |
| Settings… | `⌘,` | `Ctrl+,` | Native, browser |
| Keybindings… | — | — | Native, browser |

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
