# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

1. **Replay-session effect-family decomposition (439)** — 🟡 IN PROGRESS — replay worker
   (codex), branch `codex/queue-439-replay-effects`. Split
   `ReplaySession::apply_chord` into typed helpers for resolution, ordered effect
   interpretation, buffer switching, and trace classification without changing
   replay ordering or the shared action pipeline. Prove behavior with the existing
   replay/headless laws plus focused parity tests. Ready after item 437 to avoid
   overlapping `main/run` and storyboard integration.

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
