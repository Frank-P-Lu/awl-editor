# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 504 — destination navigators show no current-folder indication while browsing (found by item 444's Move-navigator verification, 2026-08-26)

Move, Export, and ProjectBrowse all navigate `browse_dir` internally but never
render it: the card title stays static (e.g. `move welcome.md`) with no
breadcrumb after descending into a subfolder, confirmed by real
`--screenshot-app` pixels on Move. Item 444's decided UX said the Move
navigator "shows the current root-relative destination" — this is the gap
against that line, left unbuilt there because a Move-only fix would violate
"same behavior ⇒ same code" (this is shared, pre-existing infrastructure, not
something Move's build introduced).

Fix shape (not yet decided): a breadcrumb rendered beside the title, or the
current folder folded into the row label — one owner all three navigators
route through, matching the pattern the mechanism already uses elsewhere.
Small enough to scope as its own item; needs a taste call on presentation
before building.

---
## Needs specific hardware

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
