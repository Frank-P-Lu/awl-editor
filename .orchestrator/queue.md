# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 425 — Cassowary's rotated location reads as telemetry, not a second headline

Keep the rotated active-category cue, but subordinate it to the bold `COMMANDS`
placard. First audition: Cassowary's mono face (Iosevka Regular), uppercase,
lightly tracked, muted ink, and roughly 0.28× the placard's font size instead of
the current bold ⅔-scale echo. Format it as a technical locator such as
`03 / NAVIGATE`, with the one-based index derived from the active lens's real
position; a thin rule may join the index and label. The top chips remain the
interactive controls and the placard remains the only poster headline.

Express face/scale/ink/tracking/locator treatment as theme data through the
shared rotated-location path—no Cassowary branch and no fabricated index. Do not
insert literal spaces to fake tracking if the shaper cannot author it. Capture
before/after at both DPIs over every command lens, long labels, narrow/ordinary/
wide windows, and zooms that park the rail. Pixel laws prove hierarchy by size
and ink strength, truthful indexing, no clipping/collision, and presence where
room exists. Finish with a live taste verdict against the reported Cassowary
composition. Read `THEMES.md` and `docs/render.md`.

### 424 — themes choose the toast anchor

Add a small data-only toast-anchor roster to the theme model. Each world chooses
an authored anchor; shared geometry owns safe margins, overlay collisions, and a
narrow-window fallback. No world-specific positioning code. Keep the 2500 ms
lifetime shared pending the live verdict below.

Verify every world and anchor at narrow, ordinary, and wide geometries, both
DPIs, with document, picker, and workspace surfaces present. Pixel laws prove
the toast is visible, inside the canvas, and clear of active chrome. Read
`THEMES.md` and `docs/render.md`; add the new axis to the full surface roster.

### 423 — let outline metadata use the available margin before ellipsizing

Secondary/date lines in the left outline truncate while usable horizontal room
remains. Extend their text block roughly 30–50 px left where the margin permits,
keeping a 20–24 px minimum inset from the outline. Preserve the primary label,
outline boundary, and hierarchy; ellipsize only after the wider allowance is
consumed.

Verify narrow, ordinary, and wide windows with long metadata. Geometry laws pin
the minimum inset, prove the increased allowance, and retain ellipsis when space
is genuinely exhausted. Capture the reported shape for pixel confirmation.

### 422 — prose double-click uses linguistic words without changing code words

On macOS, prose/Markdown double-click and word-granularity drag use
NaturalLanguage's `NLTokenizer(.word)` for English, Japanese, and mixed text;
code buffers keep awl's editor-style `is_word_char`. The platform adapter owns
UTF-16 conversion and returns rope char ranges snapped to awl's UAX #29
boundaries. Linux/web keep today's English rule and select one extended grapheme
for an unspaced CJK run rather than swallowing the run through punctuation.

Verify prose vs code, apostrophes, hyphens, URLs, Markdown, Japanese compounds,
punctuation, emoji/combining clusters, and mixed scripts. macOS pins `構成` from
`大幅に構成が変わっており`; portable tests pin the CJK fallback and unchanged
English plus code `snake_case`. No dictionary or network lookup.

### 421 — make the cell caret contain full-square CJK ink

Over Japanese `構成`, the cell caret covers only the middle of `成`. Preserve the
stable Latin height, but derive one stable ideographic cell from the resolved CJK
face/script—no named glyph/world branch and no per-kanji ink-height jitter.
Mixed Latin/CJK transitions remain bounded.

Verify the pure geometry seam, then ordinary captures at both DPIs over the full
proportional roster and every cell-form caret. Pixel arithmetic proves the caret
is present and contains CJK ink with its authored pad; existing Latin-height laws
remain unchanged. Vision-smoke five worlds by locating the Japanese caret.

### 419 — reserve teaching-footer room before candidate rows

At `464x288`, zoom 1.4, menu bar on, Paperbark draws the Settings footer past
the card and the three Bars worlds omit it. Reserve the footer before allocating
visible candidate rows: show fewer rows when necessary, preserve each world's
row rhythm, never draw beyond the card, and never silently omit the footer.
Apply the same composition law to Rules, Bars, and narrow History.

Depends on 413 so footer metrics come from the bundled glyph. Replace the
two-sided defect ledgers with outcome laws over every world, both DPIs, the
minimum geometry, and ordinary controls.

### 418 — one Go-to surface for files, headings, and folders

Rename `Go to file…` to `Go to…` and fold the existing Goto and Project corpora
into one typed destination list with lenses
`All · Files · Headings · Folders · Recent`. Files open, headings jump, folders
switch the active writing folder, Recent combines recent files and folders, and
All searches every known destination. Folder rows retain clear path identity;
user-facing copy says **folder**, not project.

All entry points share this overlay and accept seam: `⌘O` opens All, `⌘⇧P`
opens Folders, and the heading context action opens Headings. Retire the public
`Switch project…`, `Recent projects…`, `Browse files…`, and palette
`Go to heading…` entries; contextual heading wording may remain. The footer
includes `esc close`, yielding explanatory prose such as `type to filter` first
when width is tight.

Expose direct `Open file…` and `Open folder…` actions using the
platform-appropriate chooser. Open folder starts at the configured workspace and
switches to the accepted folder. A `Choose another folder…` fallback in the
Folders lens opens that chooser directly—never a second `ProjectBrowse` stage.

Verify catalog, File/context menus, generated GUIDE, default/rebound chords,
macOS/Linux/web rosters, all lenses and empty states, fuzzy ranking, mixed recent
ordering, typed accept effects, chooser cancel/accept, session/workspace
re-scope, sidecar semantics, and removal of public project/browse wording. Read
`docs/config.md`, `docs/platform.md`, `docs/render.md`, and
`docs/harness-reach.md` before splitting capture from live verification.

### 413 — add an openly licensed `⌫` to AwlMarks

U+232B `⌫`, used throughout footer Back cells, exists in no bundled face and
currently falls through to system fonts with host-dependent metrics. Add it to
`assets/fonts/AwlMarks.ttf`. The glyph must be original or sourced under a
licence compatible with GPL-3.0-only; record authorship/source and licence with
the font assets.

Verify every bundled-face/world roster resolves the footer through AwlMarks with
no system fallback, tofu, or host-dependent advance. Read `docs/fonts.md` and
`docs/licensing.md`.

### 406 — remove the obsolete tracked wasm without rewriting history

Remove `site/editor/awl-347842567538f209_bg.wasm`: the unused 43 MB artifact
contains 456 baked `/Users/frank/…` Cargo paths. Do not rewrite Git history; the
strings are paths, not secrets. The deploy already builds a fresh remapped wasm
through `scripts/with-remap.sh`.

Verify the site assembles and deploys without the tracked artifact and that the
fresh bundle contains no home path. No release or deployment is authorized by
this item.

### 404 — make zoom 1.0 the single default

Windowed launch uses `app::INITIAL_ZOOM = 0.8`; capture and the settings range
default to 1.0. Make 1.0 authoritative for launch, capture, and configuration
through one owner; remove divergent copies.

Re-baseline affected pixel expectations deliberately. Verify live launch,
ordinary captures, both DPIs, zoom controls/readout, persisted overrides, and
the live-probe reference geometry. Read `docs/render.md` and
`docs/harness-reach.md`.

### 395 — language confirmation toast and directory-shaped elision

- Keep `Tag document language` immediate and without an ellipsis. After it
  applies, show a brief toast naming the result, such as
  `Document language: Japanese`, through item 424's shared toast placement.
- Directory readouts must retain path identity when truncated: preserve at least
  one `/` and a recognizable part of the final folder. File-row elision remains
  unchanged.

Verify command dispatch, document metadata, undo/save behavior, toast text, and
long/narrow directory paths including a leaf that alone exceeds the allowance.
The existing `←` and context-aware `⌫ back` behavior is accepted and needs no
change.

## Needs a person, hardware, or release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
2. **AT-SPI journey (251)** — on a real Linux desktop with Orca, exercise
   document reading, caret/selection, overlays, and an editing burst.
3. **Linux drawn-menu Export click** — use a real window/compositor and confirm
   the rendered menu's Export action reaches its destination.
4. **Linux v0.10.0 artifacts** — launch both the tarball and AppImage on a real
   desktop; check launcher name/icon and the AppImage FUSE fallback.
5. **Dense pointer/wheel feel (241)** — judge live cadence. The settled
   4530x2756@2x release capture already fits without clipping.
6. **macOS Export as PDF panel (301)** — confirm initial folder/name and that
   Cancel leaves the document untouched.
7. **Live glide (284)** — judge the 20° travel tilt and whether wrapping needs a
   distinct flourish.
8. **Toast duration (296/300)** — judge the shared 2500 ms lifetime live after
   item 424 lands.
