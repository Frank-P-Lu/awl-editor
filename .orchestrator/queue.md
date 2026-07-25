# awl — live build queue

> Live execution state only. Completed and superseded work is in git history
> (`git log -p .orchestrator/queue.md`). Protocol, claiming, worktrees, and
> execution hygiene live in `.orchestrator/README.md`.

## Ready — current user-visible wave

89. **Item-86 zigzag-field correctness follow-up — tile Quokka and Gumtree’s page-margin zigzags across the field.** Reopen item 86’s light-world ground as a correctness repair: the current shader repeats teeth along one wandering chevron line but does not tile lines vertically across the margin field, while the previous tests proved only that some pattern pixels existed. Replace that isolated stroke with a genuinely repeated Mario-like zigzag field so every substantial page-margin region visibly contains material rather than large blank areas. Keep the writing column untouched. At an ordinary window height, show roughly three broad visible zigzag rows, with Quokka tighter and more playful and Gumtree broader and quieter. Preserve both worlds’ cards, typography, and palettes, and leave Bombora unchanged; do not open a new taste round unless real captures expose a genuine design fork. Add real-pixel occupancy laws that sample multiple vertical and horizontal cells, assert exclusion from the writing column and deterministic rendering, generate representative Quokka and Gumtree captures at multiple sizes, run the standing vision smoke and frame-cost check, then pass the full native and wasm gates. **Unclaimed; correctness follow-up only.**

## Ready — shared ownership and performance

90. **Continuous document scrolling — replace row quantization with one shared semantic scroll position.** **Build:** Replace awl’s row-quantized document scrolling with one continuous scroll position shared by mouse/trackpad scrolling and keyboard caret-follow. **Scope:** The current failure is that trackpad pixels are accumulated into 16px thresholds and then rounded to whole visual-row changes, while caret-follow can also position the viewport only at visual-row boundaries; because one inline-image row may be hundreds of pixels tall, either input can discard the entire image in one jump. Introduce one semantic scroll position consisting of a top visual-row anchor plus a pixel offset within that row; feed real trackpad deltas into it, translate wheel notches and Page Up/Down into pixel distances, and make caret-follow move only enough to reveal the caret rectangle. Route hit-testing, zoom anchoring, typewriter mode, selection auto-scroll, session/buffer restoration, rendering, and headless capture through that owner. Picker lists remain deliberately row-based; preserve item 82’s intersection-aware image culling. **Done:** Tall images and other variable-height blocks move smoothly and progressively under small mouse deltas; pressing Down past an image scrolls only enough to reveal the new caret line; reversing direction restores the same geometry without jumps; ordinary wrapped text, headings, tables, folds, page mode, and buffer switching remain aligned. **Verify:** Extend the capture seam to express and report intra-row scroll; prove that batched and incremental pixel deltas reach the same position, that Down/Up across a viewport-tall image is minimal and reversible, that an image remains drawn until its actual bottom exits, and that caret/hit-test geometry agrees throughout. Audit tall images, tall tables, headings, wrapping, zoom/DPI, resize, selection drag, typewriter mode, folds, session restore, native wheel and trackpad paths, and WebGL; measure release scrolling with O(visible) work, then pass the full native and wasm gates. **Unclaimed; high-risk scroll-state/ownership migration — requires the qualifying architecture/ownership planning pass before implementation under the current routing rules.**

## Timed — not blocked

20. **Pre-tag taste pass.** At the user’s explicit tag/release start, the implementation/release owner generates one current world screenshot export, then Fable judges only those images for per-world bullets, squiggle size/baseline including Bilby, dash padding, and Saltpan font outcomes; Fable never implements or edits. Ordinary pushes do not trigger it.

24. **Release-adjacent user-facing docs refresh.** After the current user-visible wave settles and before release preparation, update GUIDE, welcome/tour, and site guide for the current product, chords, and features. Matter-of-fact voice; facts verified. Site copy may change; deployment remains separately user-gated.

## Parked — explicit gate or future design

- **Export save-dialog scope:** macOS + Linux, one live-only cross-platform seam; capture uses an explicit path. Decided, not scheduled.
- **Per-world living-band choreography:** audition TwoShape/Slam/Soft against Morph; live feel is the oracle. Needs a design session.
- **Per-world copy-pulse differentiation:** possible future motion tweak; needs a design session.
- **Site deployment:** only on the user’s explicit word.

## Monitoring — non-blocking

- **Hands-on checks still useful:** Dawn/Bilby world feel; writer-diff panel/Tab + zoom readout; phantom image resize handle; upward scrolling past images in release; right-click Add-to-dictionary summon; 2px Wagtail stipple taste.
- **Live-only follow-ups from the 76–88 wave (harness can't drive them):** 80 — Find/Replace scroll smoothness over real typing; 81 — heading-chevron mouse-press→toggle wiring (no GPU App in unit tests; geometry+toggle proven headlessly); 85 — theme-picker felt input→present lag (latency-probe ms numbers are live-only); 87 — Bombora drift speed / counter-motion / calmness over real seconds.
- **GPU memory:** no action unless the 6 GB symptom recurs; then probe the live surface with the window foregrounded.

## Release blockers and reminders

- App icon.
- Dictionary/font/license notices plus code copyright/NOTICE review.
- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user’s explicit word. A dry run may precede them.
