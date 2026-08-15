# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 440 — make theme preview latest-selection-wins

Keep the theme picker's full-document live preview; do not replace it with
preview cards. Rapid keyboard, pointer, or wheel navigation must not finish the
off-screen shaping tail for every intermediate world before handling the next
movement. Coalesce superseded work at the existing `retint_theme_preview` seam
so the newest highlighted theme wins, while deliberate slower navigation still
shows the complete live preview and commit/revert leaves the whole document
settled.

Premise measurement on real Apple Silicon Metal: the explicit debounce/rate
limiter is gone, but release-mode steps still occupy roughly 29–43 ms on the
short benchmark fixture and 40–48 ms on the long fixture because
`finish_shape_tail` completes before the event handler returns. Preserve the
immediate colour/background response and the first present; avoid background
threads, per-theme document caches, and a second preview renderer. Verify a
zero-gap burst reaches the final selection without shaping every intermediate
tail, a paced sweep still presents each world, commit/revert owes no tail, and
all keyboard/pointer/wheel doors share the one policy. Measure before/after in
release with `--bench-theme-burst`, including top/middle/end scroll positions.

### 441 — export Japanese text as glyphs, not placeholders

PDF export currently preserves Japanese in `ActualText` but visibly replaces
every unsupported scalar with `□`, because the closed PDF font roster contains
only Bitter and IBM Plex Mono. Enrol repository-owned, embedding-permitted
Japanese fallback faces in the PDF shaper/subsetter and select them by actual
glyph coverage without changing Latin typography. Define the bold and code
fallback behavior explicitly; do not consult system fonts or add runtime
network access.

Reproduce with the reported mixed Japanese/English note. Verify rendered PDF
pixels contain legible Japanese rather than placeholder boxes, extracted text
round-trips the original scalars, embedded subsets contain only required glyphs
plus dependencies, Latin-only output remains stable, and headings, wrapping,
links, tables, and page breaks remain sound. Render every page for visual QA.

## Needs a person, hardware, or release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
2. **AT-SPI journey (251)** — on a real Linux desktop with Orca, exercise
   document reading, caret/selection, overlays, and an editing burst.
3. **Linux drawn-menu Export click** — use a real window/compositor and confirm
   the rendered menu's Export action reaches its destination.
4. **Linux v0.10.0 artifacts** — launch both the tarball and AppImage on a real
   desktop; check launcher name/icon and the AppImage FUSE fallback.
5. **macOS Export as PDF panel (301)** — confirm initial folder/name and that
   Cancel leaves the document untouched.
6. **Live glide (284)** — judge the 20° travel tilt and whether wrapping needs a
   distinct flourish.
