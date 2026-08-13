# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 421 — make the cell caret contain full-square CJK ink

🟡 IN PROGRESS — queue-421 (codex), branch codex/queue-421-cjk-cell-caret

Over Japanese `構成`, the cell caret covers only the middle of `成`. Preserve the
stable Latin height, but derive one stable ideographic cell from the resolved CJK
face/script—no named glyph/world branch and no per-kanji ink-height jitter.
Mixed Latin/CJK transitions remain bounded.

Verify the pure geometry seam, then ordinary captures at both DPIs over the full
proportional roster and every cell-form caret. Pixel arithmetic proves the caret
is present and contains CJK ink with its authored pad; existing Latin-height laws
remain unchanged. Vision-smoke five worlds by locating the Japanese caret.

### 419 — reserve teaching-footer room before candidate rows

🟡 IN PROGRESS — queue-419 (codex), branch codex/queue-419-footer-reserve

At `464x288`, zoom 1.4, menu bar on, Paperbark draws the Settings footer past
the card and the three Bars worlds omit it. Reserve the footer before allocating
visible candidate rows: show fewer rows when necessary, preserve each world's
row rhythm, never draw beyond the card, and never silently omit the footer.
Apply the same composition law to Rules, Bars, and narrow History.

Depends on 413 so footer metrics come from the bundled glyph. Replace the
two-sided defect ledgers with outcome laws over every world, both DPIs, the
minimum geometry, and ordinary controls.

### 418 — one Go-to surface for files, headings, and folders

🟡 IN PROGRESS — queue-418 (codex), branch codex/queue-418-unified-goto

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
9. **Zoom 1.0 real-window probe (404)** — unlock the macOS display, then run the
   real presentation/acquire reference check and judge the 100% default live.
