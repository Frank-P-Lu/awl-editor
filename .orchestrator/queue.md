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

Everything left here needs a session, hardware, or a release-time word — the
taste-decision backlog itself was cleared by the 2026-08-11 bulk acceptance
(see Decided against).

1. **The site is stale against the published release — one command, yours:**
   `gh workflow run deploy-web.yml`. Live host is `awl-editor.fly.dev`
   (`site/fly.toml`; `awl.computer` is NXDOMAIN and appears nowhere in the
   tree). **Now genuinely stale, and measured:** the live `version.json` reads
   `{"version": "0.9.0", "prerelease": false}` while **v0.10.0 is public**, so
   Check for Updates cannot see the release that shipped. `version.json` comes
   from `git describe --tags` at deploy time, so one dispatch fixes it.
   (An earlier revision of this entry claimed `0.0.0 / prerelease`; that was
   false and was corrected in place before the tag.) `FLY_API_TOKEN` is
   configured; trigger is `workflow_dispatch` only, deliberately.
2. **The macOS release arm** — Apple signing secrets, per `RELEASING.md` §1.
3. **Further tags and site deploys** — your explicit word, every time.
4. **The AT-SPI journey (item 251)** needs a real Linux desktop session with
   Orca. This Mac and its headless/Linux CI arms cannot perform the human
   document-read, caret/selection, overlay, and editing-burst journey.
5. **The Linux drawn-menu Export click needs a real window/compositor.**
   `AWL_MENU_BAR_FORCE=on` reaches the production menu geometry and hit-test on
   this Mac (15 forced menu laws pass), but every hermetic `App` is deliberately
   GPU-less and `App::menubar_press` returns before hit-testing without the
   window-bound `Gpu`. Constructing that object requires a real winit window,
   display handle and wgpu surface; the live script has no pointer-press event.
   Close this on a Linux desktop with a real rendered-menu click, or after an
   explicitly approved live GUI harness gains press input plus observable state.
6. **Item 241's dense pointer/wheel cadence remains a live feel check.** The
   exact 4530x2756@2x headless case is now measured on the release build:
   1.43 s, 215,793,664-byte max RSS, page/outline/gutter all within the canvas
   and no visual clipping. A settled capture cannot establish interactive
   cadence; that last arm needs a human at the live window.
7. **The AppImage is now published for the first time, in v0.10.0, and NOBODY
   HAS LAUNCHED IT.** At the `v0.9.0` tag `release.yml` contained ZERO AppImage
   references — `scripts/package-appimage.sh` landed afterwards — so v0.9.0
   carried only the tarball. The v0.10.0 assets are verified by download:
   both `sha256sum`s check, the tarball carries RELEASING §4's full required set,
   the AppImage is a valid type-2 image (magic `41 49 02`), glibc floor 2.35.
   **What remains unverified is the only thing that matters to a user: RELEASING
   §5 step 7, launching BOTH artifacts on a real Linux desktop.** It cannot be
   performed from this Mac, so the AppImage's desktop-integration path — launcher
   name, icon, FUSE fallback — is live and exercised by nothing but its own build.
   The tarball is the documented fallback and is unaffected.
8. **The export save panel wants your eye on macOS** (item 301) — an AppKit
   modal is unobservable from any test. Right folder? Right pre-filled name?
   Cancel leaves the document untouched? Try `Export as PDF…`.

## 🔵 BLOCKED ON THE USER — visual verdicts and live journeys

**Cleared by the 2026-08-11 bulk acceptance — shipped state stands on every
landed taste item** (see Decided against; the removed list with each item's
revert handle is in that decision's commit, `git log -S"bulk acceptance" --
.orchestrator/queue.md`). Flag anything that reads wrong live; do not re-ask
item by item. Still open, because only real time at the live window answers:

- **284 — the live glide's feel** and `MARKER_TRAVEL_TILT_DEG = 20°`; plus
  whether a wrap's transient (indistinguishable from an ordinary step) deserves
  a distinct flourish. Live judgement.
- **296/300 — the 2500 ms toast lifetime** only *feels* right live; the notice's
  look itself is accepted.

## Remaining work — handoff order (RE-DERIVED 2026-08-11, against the tree)

⚠️ **This section has gone stale five times, each time by editing the previous
list instead of re-checking the tree.** The fifth: item 345 sat on this board as
"ready to merge in one command, held for the user's word" while its merge
(`1e6fd7c8`, two parents, verified) was already an ancestor of `main` — the
word was given for a merge that had already happened. Re-derive from the tree,
every pass.

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
3. **392 — DECIDED 2026-08-11: the forcing arm.** Make `AWL_MENU_BAR_FORCE=on` a
   standing pre-push arm. Body below.
4. **327 — DECIDED 2026-08-11: elide and delay.** Settings two-column: elide the
   Project-root path, delay two-column mode until the accessory survives. Body
   below.
5. **393 — a Han run renders Japanese while the user's Han tiebreak says
   Chinese.** User report, live macOS. Premise-check the resolution ladder
   before touching code. Body below.
6. **394 — Settings arrowing: Right enters, Left does not return.** User
   report, live macOS. Bugs cluster — 387/389 changed Back/Backspace handling
   in this exact neighborhood last wave. Body below.
7. **211 — the live-glide photograph. ⚠️ RE-BLOCKED, AND THE FLIP IS THE
   LESSON.** The lock was CLEAR when this was queued and read
   `CGSSessionScreenIsLocked = true` about fifteen minutes later, before
   dispatch. A sitting launched on the first reading would now be writing
   successful-looking `LIVE-PROBE shot … ok` lines while presenting zero frames.
   **This is why the check belongs at BOTH ENDS and why a preflight-only check
   is not a check.** `caffeinate` holds a display awake and cannot unlock one.
   Dispatch when the screen is unlocked AND someone can keep it that way; the
   window is small, top-left, non-activating and never steals keyboard focus.
8. **HUMAN / LIVE — now small.** Eight blockers above, each needing a session,
   hardware, or a release-time word; the 30-item taste backlog closed by the
   bulk acceptance. ⚠️ 392 and 327 wait for dispatch until 388's quiet-host
   timing sitting completes — 388 was dispatched ALONE on purpose.
---

## Open items

### 388 — Theme-picker arrowing: measured, and the fix is a DESIGN FORK for the user

🔵 **BLOCKED ON THE USER — a design decision, not a missing measurement.**
Law landed at `3a8da981` on `claude/item-388-theme-preview-lag`; **no product
code changed**, so settled output is identical by construction.

Measured in `--release` on a quiet host (load 4.5–7.8, no `rustc`): a 9-hop
burst costs **282–291 ms on a 119-line document and 357–363 ms on an 1896-line
one** — ~32 and ~40 ms per arrow, 2–3 frames each at 60 Hz. The reported ~33 ms
is right per STEP but wrong about the reshape: `sync_theme` is 20/26 ms of it
and the frame after is another 10–17 ms.

**Dominant stage: `buffer.shape_until_scroll`, ~95% of the reshape.** Two
premises died on measurement: **font adopt is FREE** (`text_wrap_width` derives
from a face-independent `char_width`, so a world hop never rewraps and
`set_size` is a no-op — there is no wasted double-layout), and the cost is **not
per-line** (119 lines pays 20 ms, 1896 pays 17 ms; it tracks glyphs and span
fragmentation). A `sample` profile puts it inside harfrust's own
`shape_with_plan` — real glyph shaping, no cheap cure.

**The removable work is reach, not per-glyph cost.** `full_shape_height` budgets
every visual row, so one arrow shapes the whole document while only a viewport
can be drawn — and all 19 consecutive pairs in `THEMES` differ in face, so every
arrow pays. Clamping to the window measured 282→113 ms and 357→212 ms.

**Why it was not landed:** that reach is deliberate. An unshaped tail falls back
to `RowGeom`'s ESTIMATED line height — the scroll-jump bug `full_shape_height`'s
own doc records — and it feeds `max_scroll`. Narrowing needs either re-shaping
at settle (**the catch-up stall the brief ruled out**) or permanently relaxing a
document-wide invariant (**a correctness decision, not a timing fix**).

**The fork, and the lane's recommendation:** shape the visible window, present,
then finish the tail **within the same step** before the next event is handled.
Input-to-present ~30 ms → ~13 ms; total work unchanged, nothing deferred past
the step, no arrow ever shows a wrong font, document fully shaped at every step
boundary. It is an App-level change and was correctly not attempted in a round
that could not law and mutation-prove it.

**Two questions owed:** (1) does ~32–40 ms per arrow read as laggy or merely
heavy? (2) same-step completion as above, or leave it? ⚠️ Note the law that
landed PINS today's whole-document reach, so whoever takes the fork must update
it deliberately — its mutation shows 25 of 400 rows shaped when the budget is
narrowed to the viewport.
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

### 392 — make `AWL_MENU_BAR_FORCE=on` a standing pre-push arm

**DECIDED 2026-08-11 (user):** the forcing arm, not the widened filter.
Background: `native-gate.sh` runs its menubar arms over tests matching
`menubar|menu_bar`; three laws in two days shipped blind to this axis — two in
`metric_scale`/`caret_filled_knockout`, one in `workspace_back_width` — each
passing a full local gate, each caught by CI instead. `menubar::platform_default`
is `false` on macOS and `true` everywhere else, so an unpinned law measures a
different product locally than in CI; about eleven more tests sit outside the
name filter. Costs, measured: the forcing arm reproduced all three failures in
about a second each; the widened filter costs two more full suites per gate
(~4 min each) and is rejected. Build: a standing arm in `native-gate.sh` that
runs the affected filters under `AWL_MENU_BAR_FORCE=on`, named in the receipt
so a reader sees what was and wasn't covered. Verify: prove non-vacuity by
reverting one of the three caught fixes locally and watching the arm go red.

### 327 — Settings two-column: elide the path, delay the transition

**DECIDED 2026-08-11 (user):** elide "Project root" like other row text, and
delay two-column mode until the content pane preserves its accessory — staging
one region a little longer is calmer than showing both regions with controls
missing. Diagnosed mechanics: with the long Project-root fixture the focused
narrow pane has no accessory at 640–740, carries it at 760–860, then
`workspace_is_wide` introduces the category rail at 880 and shrinks the
diagonal row-cluster budget from 514 to 339 px; the value/Range rail drops
through 940 and returns at 960 (419 px). ⚠️ **The 880 is a property of THAT
fixture, not of the product** — item 387 measured the same flip between 1070
and 1075 px on the default fixture at `--capture-dpi 1`, because
`workspace_is_wide` is derived from display-face metrics × zoom, not a
constant. Do not write 880 into a law; derive the boundary from the flip
itself. Verify: sweep widths across both fixtures at 1× and 2×, asserting the
accessory is present whenever two-column mode is active and the elided path
never overflows its cell; the law names which fixture and boundary it enrolled.

### 393 — a Han run renders Japanese while the user's Han tiebreak says Chinese

User report (2026-08-11, live macOS): with the ambiguous-Han setting on
Chinese, a document mixing Japanese sentences with the bare-Han heading 你好
still renders 你好 with a Japanese face. **The report is a hypothesis — find
which stage of the ladder decided before touching code.** The documented order
(docs/fonts.md, `script.rs`): frontmatter `lang:` → the run's own script →
`cjk_priority` Han tiebreak → Latin floor. Two mechanisms can make the observed
behavior CORRECT-but-surprising: write-back-once stamps `lang:` frontmatter
into untagged markdown CJK docs on open (a Ja-dominant doc gets `lang: ja`,
which outranks the tiebreak forever after, even scrolled out of view), and a
doc opened before the setting changed keeps its stamp. Reproduce with a fixture
of this doc shape; read the sidecar's `doc_lang` and per-run
`font.scripts`/`font.cjk`. Also verify what the user's Settings knob actually
writes: is the ambiguous-Han control wired to the `cjk_priority` key the loader
reads, or is it the generated-reference class of defect (a key the loader never
consults)? Outcomes and routes: (a) a stamped `lang: ja` wins → mechanism
correct; the open question (does an explicit user tiebreak outrank a
machine-stamped tag, or should the stamp be made visible?) goes back to the
user as a design brief, not a silent fix; (b) the knob writes an unread key →
real defect, fix at the loader seam; (c) `cjk_priority` is consulted and still
resolves ja → real defect in `script.rs`. Either defect ends with a law
driving the same doc fixture on both sides of the tiebreak and requiring the
resolved family pair to differ (the generator-collapse rule: probe both sides
of the condition).

### 394 — Settings arrowing: Right enters, Left does not return

User report (2026-08-11, live macOS): in the Settings workspace, Right moves
focus inward but Left does not come back. Premise-check with real `--keys` on
the live-app driver (`--screenshot-app`), then sweep — do not hand-pick one
state: for every reachable Settings state (row list, category rail, value/Range
rail, two-column and narrow, query filtered and not), drive Right then Left and
read focus from the sidecar. The law: wherever Right moves focus, Left from the
destination returns to the origin, or the state names why not; it must fail
while the asymmetry exists. Neighborhood: items 387/389 changed Back/Backspace
ownership (`BackKey`, `OverlayState::detail_back`) in the same wave — check
whether the Left intercept was narrowed by the same edit before inventing a new
seam. Fix stays on the keymap → `Action` → `apply_transition` route so `--keys`
drives it and the sidecar sees it.

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

- **The 2026-08-11 bulk acceptance — shipped state stands on every taste item
  then open.** ~30 verdicts closed as accepted, not deferred: Kite's veil
  (0.13) and facet tag, 261's delete branch, the narrow-History footer hole,
  the footer-reclaim row budget, the diagonal mark's instant snap, the spell
  popup's no-frost, 301's Finder-forward reveal, and items 131e/174/242/273/
  294/296–300/303/308/309/312–318/321/323/338/342/345–347/359/386/387. Each
  stays revertible on sight; the removed list with per-item revert handles and
  gallery paths is in this decision's commit — `git log -S"bulk acceptance" --
  .orchestrator/queue.md`. Do not re-open without a live sighting that reads
  wrong.
- **231's guest-VM spend — DECLINED 2026-08-11.** The project rides free
  open-source GitHub runners for this axis; `mac (render::tests)` stays
  tolerated red, pinned by name in `ci.yml`. Revisit only if the axis starts
  gating a shipping artifact. A negative reproduction remains publishable if
  the rig ever exists.
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

- **`ListStyle::Rules` second carrier** — requirements recorded beside
  Paperbark's entry in `theme/tests/personality.rs` (item 283); adopt when a
  world earns it, not on a date.
- **A pitch/weight dial for the `Pinstripe` arm** (from the closed 258/260
  judging): Mulga and Cassowary share the light-line-on-dark arm, separated by
  hue and room value, not structure. Owed only if a FOURTH carrier lands or an
  audition ever reads Mulga↔Cassowary as a repaint — Mulga first. Nothing owed
  today.
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
