# awl — live build queue

> Live execution state only — **open work, owed work, and what only the user can
> decide.** Nothing here describes something that already landed. Protocol,
> claiming, worktrees and execution hygiene live in `.orchestrator/README.md`.
>
> **Entries are BRIEFS, not essays (user decision 2026-08-09):** headline, the
> load-bearing numbers and paths, dependencies, verify, routing. History,
> premise-correction stories and closure reports live in git — `git log -p
> .orchestrator/queue.md` walks all of it; every completion report through
> 2026-08-06 is at `git show 0dc30706:.orchestrator/queue.md`; the pre-terse
> bodies of everything below are at `git show 87dc0450:.orchestrator/queue.md`;
> `git log -S"<phrase>" -- .orchestrator/queue.md` finds who removed a line.
> ⚠️ A compression once cleared an OPEN item's body, and summary lines have
> survived fixes that landed underneath them. **Decide open-vs-done from the
> TREE and `git log --grep`, never from what this file says about itself.**

## 🔵 BLOCKED ON THE USER — nothing else can close these

⚠️ This section has been silently deleted twice (once by `git add -A`, once by a
worker committing against its brief). After every merge and compression, verify
this heading exists; `git log -S"BLOCKED ON THE USER"` finds who took it.

1. **The site is stale against the published release — one command, yours:**
   `gh workflow run deploy-web.yml`. Live host is `awl-editor.fly.dev`
   (`site/fly.toml`; `awl.computer` is NXDOMAIN and appears nowhere in the
   tree). ⚠️ **This entry's own figure was measured false and is corrected in
   place:** the live `version.json` reads `{"version": "0.9.0", "prerelease":
   false}`, not the `0.0.0 / prerelease` claimed here — the site was redeployed
   after v0.9.0. It is stale only against whatever tag ships next, because
   `version.json` comes from `git describe --tags` at deploy time.
   `FLY_API_TOKEN` is configured; trigger is `workflow_dispatch` only,
   deliberately.
2. **Kite's veil strength** — `WARP_PAGE_VEIL = 0.13` (read from the shader; a
   lane's report once said 0.20). Captures: the lane's worktree,
   `gallery/item-268/`.
3. **Which world adopts `ListStyle::Rules` next** — second-carrier requirements
   are recorded in `theme/tests/personality.rs` beside Paperbark's entry; see
   item 283.
4. **Item 261's open call: delete-outright vs a `cfg(test)` fixture** for
   `DeckleAnchor::Page`'s mutation witness. The delete branch was taken;
   reverting is re-adding one small shader function.
5. **Mulga and Magpie grounds want a visual judge** — see the visual/live
   blocked section below.
6. **Kite's facet tag:** its doc calls it a "technical room" but its only facet
   is `voice: Modern`, and Technical belongs to Cassowary. Picker facets are
   curated and capped, so the change is yours.
7. **The narrow History comparison stage draws no footer** (`show_rows` false →
   `hint_rows` 0 at ~900×520 and below) — a deliberate discoverability hole;
   whether to spend vertical space there is taste.
8. **The macOS release arm** — Apple signing secrets, per `RELEASING.md` §1.
9. **Further tags and the site deploy** — your explicit word, every time.
10. **The footer-reclaim row budget** (found during the CI RED fix `02d0ea23`,
    implemented, deliberately backed out): `avail_px` charges the hint row and
    blank separator a full `lh` each and never credits `overlay_footer_reclaim`,
    which draws them compact — 65 px unspent at zoom 3, a whole row. Crediting
    it changes shipped row counts on cards that already fit. The question is how
    many rows a card should show; the arithmetic is ready either way.
11. **Should the selected diagonal mark travel with the selection band?** Today
    it snaps directly to the destination row while the band eases there. The
    alternative makes the mark ride that same ease on the two Diagonal worlds.
    This is a live feel decision no settled capture can make. Recommendation:
    prototype the shared-ease arm only if the current snap reads detached; keep
    the instant mark if it reads as the destination indicator rather than part
    of the moving band.
12. **Hosted-mac Metal diagnosis (item 231)** needs a macOS guest VM with
    paravirtualised Metal. No VM tooling is installed; creating that rig is a
    real storage/time spend. The next engineering step begins only after that
    spend is approved. A negative reproduction is publishable; no speculative
    product fix lands under the diagnosis item.
13. **The AT-SPI journey (item 251)** needs a real Linux desktop session with
    Orca. This Mac and its headless/Linux CI arms cannot perform the human
    document-read, caret/selection, overlay, and editing-burst journey.
14. **The Settings workspace's 880 px two-column transition (item 327).** The
    previously unexplained non-monotonic rail hole is diagnosed on the current
    reachable Settings state: with the long Project-root fixture, the focused
    narrow pane has no accessory at 640–740, carries it at 760–860, then
    `workspace_is_wide` introduces the category rail at 880 (⚠️ **that number is
    a property of THIS fixture, not of the product** — item 387 measured the
    same flip between 1070 and 1075 px on the default fixture at
    `--capture-dpi 1`, because `workspace_is_wide` is derived from display-face
    metrics × zoom, not a constant. Do not write 880 into a law) and shrinks the
    diagonal row-cluster budget from 514 to 339 px. The value/Range rail drops
    through 940 and returns at 960 (419 px). Choose whether to delay two-column
    mode until the content pane preserves its accessory, or keep the 880
    transition and choose which of row name, value, or rail yields first.
    Separately, “Project root” is a full un-elided machine path, so the boundary
    varies by machine. Recommendation: elide the path like other row text and
    delay two-column mode until the accessory survives; staging one region a
    little longer is calmer than showing both regions with controls missing.
15. **The Linux drawn-menu Export click needs a real window/compositor.**
    `AWL_MENU_BAR_FORCE=on` reaches the production menu geometry and hit-test on
    this Mac (15 forced menu laws pass), but every hermetic `App` is deliberately
    GPU-less and `App::menubar_press` returns before hit-testing without the
    window-bound `Gpu`. Constructing that object requires a real winit window,
    display handle and wgpu surface; the live script has no pointer-press event.
    Close this on a Linux desktop with a real rendered-menu click, or after an
    explicitly approved live GUI harness gains press input plus observable state.
16. **Item 211's unoccluded live-glide photograph needs an unlocked display.**
    The existing live-band sweep is the correct instrument and refuses false
    success, but the required end-of-run lock check currently reports
    `CGSSessionScreenIsLocked = true`. `caffeinate` cannot unlock it. Run the
    sitting only after the display is unlocked and can remain foregrounded.
17. **Item 241's dense pointer/wheel cadence remains a live feel check.** The
    exact 4530x2756@2x headless case is now measured on the release build:
    1.43 s, 215,793,664-byte max RSS, page/outline/gutter all within the canvas
    and no visual clipping. A settled capture cannot establish interactive
    cadence; that last arm needs a human at the live window.

## 🔵 BLOCKED ON THE USER — visual verdicts and live journeys

The mechanical evidence for these is complete. What remains is an aesthetic
choice, a live-feel judgement, or hardware/session access an agent does not have.

- 🔵 **387's Back cell while a filter is live.** In Settings, `⌫` belongs to the
  query field as soon as a character is typed, so the footer's Back cell has to
  say something else for that stretch. **Shipped: fall back to `tab back` while
  the query is live**, because the stage must always name a Back that is TRUE.
  The alternative is to drop the cell entirely while typing — then `tab back`
  never appears at all, but part of every filtered journey advertises no Back.
  Reverting to that is one line: `BackKey::Focus`'s arm in
  `OverlayState::detail_back` returns `None`, plus one assertion.
- 🔵 **386's two glances.** Potoroo with the rail UNFOCUSED reads 3.07:1, the
  roster's tightest cell and just over the project's own 3.0 bar — not a defect,
  but worth an eye. And the dimmed unfocused rail plate is a declared
  degradation at ΔE 4.49 worst case, below the 2.3 JND by design: confirm it
  still reads as "a mark, insisting less" rather than as nothing.
- 🔵 **The AppImage has never been published, and v0.10.0 will be its first
  public appearance.** At the `v0.9.0` tag `release.yml` contained ZERO AppImage
  references — `scripts/package-appimage.sh` landed afterwards — so v0.9.0's
  Release carries only the tarball and `SHA256SUMS`. RELEASING §5 step 7 (launch
  BOTH artifacts on a real Linux desktop) cannot be performed from this Mac, so
  the AppImage's desktop-integration path — launcher name, icon, FUSE fallback —
  ships unexercised by anything but its own build. The tarball is the documented
  fallback and is unaffected.

- 🔵 **Four things on `main` awaiting your eye, each revertible in one commit or
  line:**
  - **The caret** (item 345) — no longer overhangs its glyph on Currawong and
    Cassowary (was 120% of the letter, 2.4 px into the next character; now
    matches the cell: 14.400 → 12.000 at 1×). It is *narrower* there now, and
    the caret is the design's one accent. Branch `claude/item-345-caret-pitch`
    is ready to merge in one command; `gallery/item-345/*_before_after.png` has
    2×2 grids with the glyph-cell edge marked. Say the word and it lands.
  - **Magpie's mark** (item 346, candidate B landed) — vertex ~70.7° → ~50.8°,
    weight 1.25 unchanged, Mangrove untouched. If B reads too wide, C (87.6°,
    smaller both ways) is the honest third; D (thinner stroke) is the trap —
    weakest presence at 1× for a difference invisible at 2×. Fixed-crop
    comparison: `gallery/item-346/compare-magpie-mark-1x-vertex-desc-before-C-A-D-B.png`
    (+2× twin, + `compare-magpie-row-*` at 1:1). One line in
    `src/theme/diagonal.rs` either way.
  - **The writing column at 2×** (item 338) — sixteen decorations (squiggle,
    pills, table pads, rules, caret widths, entrance drop…) were half their
    tuned size on every Retina display; the squiggle reads instantly.
    `gallery/item-338/338-2x-before-after.png`. 1× is unchanged. Several carry
    "TASTE TUNABLE" doc comments, so the answer may be "yes, all sixteen" — but
    it is yours, per construct. A two-sided ledger law holds them meanwhile.
  - **The card's width cap** (item 342) — 520 → 545, clearing the clipped help
    line (`esc clos`/`esc clo`) on Potoroo and Firetail; 540 does not clear it.
    The extra 25 px land as air after the hint and a looser label-to-chord
    gutter (~1.40:1 → ~1.47:1) — the gutter is what a critical eye will notice.
    `gallery/item-338/342-shipped-look-*-zoom080-before-after.png`. Open
    sub-question, upstream of 327: which scale tier is the cap TUNED at?
    `LogicalGrowOnly` keeps device width below scale 1, so the shipped 0.8 zoom
    is 25% roomier than anything a Retina user sees.
- 🔵 **Zoom-300% minimum card** (item 347): in the two smallest windows the app
  allows, a workspace stage plans no rows and draws no other region — a card
  carrying no list, 7 reachable cells. Keep a one-line minimum card, or refuse
  to enter the stage? Held by a two-sided ledger meanwhile.
- 🔵 **The find/replace panel** (item 174 slice 3): outer margin and inner pad
  are raw device-px constants — half their tuned size on every Retina display
  (CLAUDE.md's own tripwire, live again). Not promoted because the 1× look is
  what was tuned; the law is fix-tolerant if you say scale it. The objective
  narrow-window defect is closed: the card now caps to the available width,
  wraps its complete teaching copy, and stays inside both canvas edges.
- 🔵 **The right-click menu no longer dims your document** (item 298): carded
  worlds get no frost at all; bare worlds frost the menu's own footprint only.
  Before/after differs on 76–79% of pixels; `gallery/item-298/`. Revert is one
  line. Out of step now: the **spell popup** still takes no frost anywhere
  (DESIGN §5 says it "recedes nothing" on purpose) — whether that stands is
  yours.
- 🔵 **Right-click menu's greyed-out labels** (item 299): "unavailable" used to
  sit one row below its own row (ΔE 0.0 — invisible); now correct. A glance to
  confirm it reads as quiet, not broken. `gallery/item-299/`.
- 🔵 **Quokka's card texture at non-1× zoom** (item 359): keep the shipped
  DPI-only 11 px chamfer / 8 px dot cell as stable printed detail, or scale both
  with editor zoom so the card remains one geometric form with its type? At
  zoom 2 the zoom-aware candidate is 22/16 logical px and changes 19.6–22.9%
  of the canvas; at zoom 0.8 it is 8.8/6.4 and changes 8.4–13.2%. Both remain
  legible. Recommendation: keep DPI-only unless the larger, more decorative
  2× texture is specifically wanted. Comparison sheet:
  `/tmp/awl-item-359/comparison-current-vs-zoom-aware.png`.
- 🔵 **One rail wore another's highlight — fixed** (item 309): only the selected
  rail brightens now. Glance: `gallery/item-309/309-crop-{BEFORE,AFTER}-*.png`.
- 🔵 **Two plate changes, both design calls** (items 308, 316): a 1-px rim
  appeared under the footer hint on `Bars` worlds (the old plate was ΔE 1.91
  from the page — invisible); an empty chip above "Switch project…" disappeared.
  `gallery/item-308/`, `gallery/item-316/`, each with @2x pairs.
- 🔵 **`--help` realigned by 7 whitespace-only lines** (item 273 r1) — one
  padding rule instead of an eyeballed-per-line table; one `line()` function to
  revert. One factual fix kept regardless: `--measure` said "default 80", the
  value is 70 prose / 100 code.
- 🔵 **`Section::Cli` placement in the reference** (item 273 r1): appended last,
  after Markdown; caption "Unlisted flags" equally open. Rendered clean, 0
  dangling links. `gallery/item-273r1/ref-cli.png`. Cheap to move.
- 🔵 **The export save panel wants your eye on macOS** (item 301) — an AppKit
  modal is unobservable from any test. Right folder? Right pre-filled name?
  Cancel leaves the document untouched? Try `Export as PDF…`. The separately
  diagnosed Linux rendered-menu click is recorded under BLOCKED.
- 🔵 **Should exporting bring the Finder forward at all?** (item 301) The reveal
  takes focus — DESIGN's no-nagging boundary. If no, the honest alternative is
  a palette row, not an automatic reveal. Live-only.
- 🔵 **Does the Reference belong in the Help menu?** (item 273 r4) `HELP_ITEMS`
  is a hand-curated four-item list with no coverage law. One line either way.
- 🔵 **The menu bar's pads double on Retina** (item 323): `BAR_INSET_X`,
  `TITLE_PAD_X`, `DROP_PAD_*` went `Logical`; 1× byte-identical; the >1× look
  has never been seen by a human. Needs Retina Linux/web or
  `AWL_MENU_BAR_FORCE=on`; the macOS dropdown is live-only.
- 🔵 **Non-macOS menu bar height** (item 321): now a constant 35.6 logical px at
  every zoom/DPI instead of thinning as density rises (+5 px at 2×). Byte-
  identical at 1× and on macOS default; worth a Retina Linux/web look.
- 🔵 **313's terminus call:** does the hint continue the spine's lean past the
  terminus or sit at the terminal x? Measured pixel-identical on Mangrove, 9 px
  apart on Magpie. `continue` is implemented behind a one-word switch; the
  argument for it: at terminal x the hint reads as one more list row.
  `gallery/item-313/{continue,terminus}-*.png`.
- 🔵 **318's raking frost edges** — landed; the one open question: should a
  mirrored composition's QUERY FIELD mirror like its rows? If yes, Magpie's
  43.15 px frost overhang disappears but the field right-aligns and its `›`
  travels as you type — it needs its own design, not 313's.
  `gallery/item-318/{before,after}-{Mangrove,Magpie}.png`.
- 🔵 **297's four calls** (`gallery/item-297/`): (1) past 1.74× zoom the
  navigate cue DISAPPEARS rather than shrinking — deliberate (the size is the
  composition), but gone-vs-smaller is yours; (2) before/after pairs; (3) the
  cue takes the wordmark's full ink — hierarchy by size alone; (4) the gap is
  one constant (0.12 em).
- 🔵 **314 moved a visible length on Retina:** page collapsed side pad and
  outline rail inset now honour their authored 16 logical px (were reading as
  8 device). dpi-1 byte-identical (19/19); wants a Retina glance.
- 🔵 **312's feather width and lean** (`gallery/item-312/`): shipped 28 px
  feather vs 14 (law's floor) and 42 (skirt onto the page); lean-on vs upright.
  Orchestrator's eye: the defect (words breaking clean at a hard edge) is gone
  and the lean reads intentional — yours to accept.
- 🔵 **294's three calls** (`gallery/item-294/`): (1) the full-takeover frost
  was HALF strength on Retina (fixed reach was device-px); fixed — moves 224k
  of 960k pixels on Gumtree at 2×, isolatable to one function if you want it
  reverted; (2) the footprint's hard edges cut words mid-glyph — reads as a
  pane of frosted glass; (3) a blurred squiggle becomes a soft red band inside
  frost. The caret-under-card case reads well and needs no change.
- 🔵 **296/300's notice look** (`gallery/item-296-300/`): one plated LABEL line
  at the top of the writing column. Calls: placement (covers the first prose
  line on heading-less docs), square corners, the toast/sticky loudness ladder,
  padding ratios. The 2500 ms toast lifetime only *feels* right live.
- 🔵 **131e/303's two taste calls** (`gallery/item-131e-303/`): Magpie hairline
  now weight 1.25 — too faint? one dial, Mangrove untouched. And the mark hangs
  at the cluster's budget outer end (fixed-surface rule), so on chord-less rows
  it sits ~460 px from its label — the alternative anchor breaks the
  fixed-surface rule; choose the tradeoff.
- 🔵 **Trigger, not task** (from the closed 258/260 judging): Mulga and
  Cassowary share the light-line-on-dark arm, separated by hue and room value,
  not structure. If a FOURTH `Pinstripe` carrier lands, or an audition ever
  reads Mulga↔Cassowary as a repaint, a Zigzag-style pitch/weight dial becomes
  owed — Mulga first. Nothing owed today.
- **284 — the live glide's feel** and `MARKER_TRAVEL_TILT_DEG = 20°`; plus
  whether a wrap's transient (indistinguishable from an ordinary step) deserves
  a distinct flourish. Live judgement.
- **242 — `readout::CANVAS_INSET` remains a visual choice.** The formal
  five-shot affordance smoke and the 1×/2× anchor/menu/margin/outline sweeps are
  complete; every affordance was locatable and all objective geometry laws
  passed. The inset is declared `Physical`; promoting it to `Logical` doubles
  it on Retina and is now purely an appearance decision.
- **271/283 —** `Rules` ships on one carrier; second-carrier requirements live
  in `theme/tests/personality.rs`.

## Remaining work — handoff order (RE-DERIVED 2026-08-09, against the tree)

⚠️ **This section has gone stale four times, each time by editing the previous
list instead of re-checking the tree.** Every entry in the previous list was
verified landed via `git log --grep` before this re-derivation (292/293/299/303,
294/298, 305, 291, 296+300, 273's residuals, 302, 227, 131e+303 — all merged).

1. **388 — Theme-picker arrowing is still visibly laggy.** User reports the landed
   debounce removal did not make Up/Down browsing feel responsive. The mechanism
   pays a measured ~33 ms full font/span reshape on every differing-font preview;
   the mechanism before it accumulated a 141–151 ms ninth-step stall. Reproduce in
   `--release` with movement-to-present receipts over the full 20-world arrow sweep
   and a long Markdown document; attribute font adopt, reshape, row geometry, atlas
   and present costs; then remove or amortize the dominant work while every arrow
   still previews the destination world's truthful font and colors. A delayed
   wrong-font sequence or a later catch-up stall is not a fix. **Dispatch ALONE —
   its deliverable is timing, and a receipt taken beside other lanes measures the
   wave, not the mechanism.**
2. **391 — the picker footer teaches `⌫ up` where Backspace is now inert.** Body below.
3. **392 — the menu-bar axis is covered by a NAME FILTER, and it has now cost a CI
   red.** `native-gate.sh` runs its menubar arms over tests matching
   `menubar|menu_bar`, and its own comment records a census counting 14 tests that
   actually see a tripled reserve. Both laws CI just caught were outside that
   filter, and about eleven more remain. The trade is gate wall-clock against
   finding this class locally: widen the filter (two more full suites per gate), or
   make `AWL_MENU_BAR_FORCE=on` a standing pre-push arm — it reproduced both
   failures in under a second where the lavapipe container cost ten minutes.
   Recommendation: the forcing arm, because it is nearly free and this class has
   now escaped a local gate twice. 🔵 User decision.
6. **🔵 HUMAN / LIVE after 386–390.** **231** needs the approved macOS guest-VM
   spend; **251** needs a human at a Linux desktop with Orca. **327** and the landed
   taste calls (338/342/345/346, carried in the visual/live blocked section) close
   on the user's eye.

---

## Open items

### 388 — Theme-picker arrowing is still visibly laggy

🟡 IN PROGRESS — claude (deep), branch `claude/item-388-theme-preview-lag`,
dispatched ALONE on a quiet host: its deliverable is movement-to-present timing
in `--release`, and four concurrent lanes at the gate phase measured load
average 69.79 here, so a receipt taken beside them measures the wave. Execute
from handoff item 1 above.

### 391 — The picker footer advertises `⌫ up` where Backspace is now inert

Surfaced by item 389, out of its scope and deliberately not fixed there. The
flat Switch-Project picker no longer ascends, but `overlay/kind.rs`'s
`kind_actions` still prints `⌫ up` for `OverlayKind::Project`. The text is
KIND-level and shared with the Settings folder-value picker (`Bind::Path`),
which genuinely does still ascend — so the two instances need distinguishing
before the hint can differ, and the `OverlayState`/`Journey` split does not
currently carry that context from picker to hint. Note item 387 landed a
`BackKey` owner for the workspace footer's Back cell in the same wave; check
whether that owner is the right seam to extend rather than inventing a second
one. Verify by driving both pickers with real `--keys` and reading each footer
from the sidecar; a law must fail if either picker's hint names a key that its
own intercept does not honour.

## ⚠️ TRIPWIRE — ONE SHIPPING GATE THAT LOOKS EXACTLY LIKE A DEFECT AND IS NOT

`overlay_prepare_bar_scrims`'s gate reads `backing == BarePlates` — the same
card-vs-row substitution a whole item was written to remove — **and it is
CORRECT AS-IS. Do not "fix" it to `draws_row_plates()`.** That scrim pass is the
only thing that clears `panel_card` on a bare-plate world, so gating it out
would let a stale instance survive into a `Diagonal` frame. It was caught once
as a near-regression and recorded rather than shipped.

Related owners, so a replacement law has a real oracle rather than a fabricated
one: `ListStyle::draws_row_plates()` is the one owner of "does this style back
its rows with plates", `overlay_selection_rects` is the one place a list style
becomes row surfaces, and `overlay_bar_rects_probe` **refuses** on a plateless
world. **Earn an exclusion by measurement — the frame must emit no row surface
at all on the excluded world, at the same fixture and DPIs — rather than by a
name list, so a world that starts drawing plates fails instead of dodging the
sweep.**

## Decided against — do not re-propose without a new reason

- **Reusing one mutable render pipeline across the diagonal/frost roster sweeps to shorten
  their 23.34 s + 15.84 s cost.** The optimization changed per-cell pixel measurements,
  proving cross-cell state changed the law's subject. Fresh-pipeline isolation stays. Both
  laws mutation-failed over their full enrolled cells and `cargo test --bin awl render::`
  passed 1105/1105; item 373 must budget this measured coverage floor.
- **A separately-named `THROUGH VIEW` figure on the writer's card.** Closed on
  purpose, not deferred. The recorded reason: the card earns its calm by carrying
  few figures, and "how far through what I can see" is a second answer to a
  question the reader already has one answer for.
- **A closing pull-quote mark.** Blockquote text is already dim for the block's
  whole extent, so the end is legible without a second glyph, and a closing mark
  has no honest anchor — the last line's right edge is ragged, so it would float
  at an arbitrary x or hang in a margin that holds nothing. Hanging pull-quote
  marks are conventionally single.
- **A pointer/keyboard split on the fold chevron's turn.** The brief recommended
  animating on the POINTER path and SNAPPING on the KEYBOARD path, because
  `chevron_revealed` puts the mark on the caret's own row by construction. **That
  split was never implemented, the co-present animation reads fine live
  ("the chevrons are great"), and it is PROMOTED as shipped.** `FOLD_CHEVRON_TURN_MS
  = 140.0` stands. Do not build the split.
- **A fifth `ListStyle` shaped as a grid or tile layout** (a palette is a linear
  scan, tiles fight it, and it is IDE-shaped), **a stacked or overlapping deck**
  (hurts scanability), or **numbered quick-select rows** (a feature, not a style,
  and a structural device must encode something true).
- **A warm tutorial voice anywhere in the reference.** The split is the user's
  own: the reference is cold, **the tutorial is the user's to write.** A lane must
  not draft one, must not "warm up" the reference, and must not leave placeholder
  tutorial prose for the user to fill in.
- **A restated loudness target (item 118).** The user dropped the target shape — *"i think
  we just drop the target. it's fine, right now."* The roster's measured mean **2.20** is
  accepted as awl's shape; `1, 7, 6, 4, 2` / mean 2.90 is retired — not amended, not replaced
  by a descriptive one. **This question has already been answered twice** because the decision
  was recorded where it was not read; do not re-ask it.

## Parked — explicit gate or future design

- **Export save-dialog scope:** macOS + Linux, one live-only cross-platform seam;
  capture uses an explicit path. Decided, not scheduled.
- **Per-world living-band choreography:** audition TwoShape/Slam/Soft against
  Morph; live feel is the oracle. Needs a design session.
- **Per-world copy-pulse differentiation:** possible future motion tweak; needs a
  design session.
- **Site deployment:** only on the user's explicit word.
- **Kite's stereo idea, recorded and NOT queued.** Stereoscopy needs the two
  views SUPERIMPOSED and fused by the viewer's brain; here they are side by side
  with an opaque page between them, so nothing fuses — and an interocular offset
  is precisely what reintroduces the "two tunnels" read. **Do not build stereo.
  Do not silently drop it either:** if a future round wants depth *between* the
  margins, the honest lever is FOV/perspective strength on a single shared
  camera.
- **A rotating mark about the vertical axis (`v → | → v`)**, deliberately not
  queued: it returns the mark to itself and would read as "acknowledged, nothing
  changed". It has no referent in awl today — zero-network is a design invariant
  and nothing is ever loading. Revisit only if a genuine indeterminate state
  appears.

## Monitoring — non-blocking

- **Hands-on checks still useful:** writer-diff panel/Tab + zoom readout;
  heading-chevron mouse-press→toggle wiring; theme-picker felt input→present lag;
  Bombora drift speed / counter-motion / calmness over real seconds.
- **GPU memory:** no action unless the 6 GB symptom recurs; then probe the live
  surface with the window foregrounded.
- **The `atspi` and `mac (render::tests)` CI arms are tolerated red by design**,
  pinned by name in `ci.yml` to items 257 and 231. `atspi` was deliberately NOT
  promoted to gating when 257 closed: **the repaired probe's first instrument is
  CI itself, and promoting an arm on a probe nobody has watched run is how a
  green comes to mean nothing.** Promote it after it runs green on `main` for a
  stretch, as a conscious decision.
- **Linux-container repro recipe for lavapipe CI reds:** in commit `196ad4ee`'s message
  (Ubuntu 24.04 + `mesa-vulkan-drivers` + rustup; `-e RUSTC_WRAPPER=` required; a plain
  checkout, never a linked worktree — its `.git` points into the host).

## Release blockers and reminders

- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user's explicit word. A dry run may precede them.
- **Exactly one `native-gate-receipt` appeared in one 30-commit stretch.** The
  standing fix — **put the receipt in the MERGE COMMIT** — is not being followed
  reliably, and the tree once carried an unverified accessibility fix on `main`
  as a result. The process gap is the finding, not the code.
