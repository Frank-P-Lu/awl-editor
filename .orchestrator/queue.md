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

1. **The macOS release arm** — Apple signing secrets, per `RELEASING.md` §1.
2. **Further tags and site deploys** — your explicit word, every time.
3. **The AT-SPI journey (item 251)** needs a real Linux desktop session with
   Orca. This Mac and its headless/Linux CI arms cannot perform the human
   document-read, caret/selection, overlay, and editing-burst journey.
4. **The Linux drawn-menu Export click needs a real window/compositor.**
   `AWL_MENU_BAR_FORCE=on` reaches the production menu geometry and hit-test on
   this Mac (15 forced menu laws pass), but every hermetic `App` is deliberately
   GPU-less and `App::menubar_press` returns before hit-testing without the
   window-bound `Gpu`. Constructing that object requires a real winit window,
   display handle and wgpu surface; the live script has no pointer-press event.
   Close this on a Linux desktop with a real rendered-menu click, or after an
   explicitly approved live GUI harness gains press input plus observable state.
5. **Item 241's dense pointer/wheel cadence remains a live feel check.** The
   exact 4530x2756@2x headless case is now measured on the release build:
   1.43 s, 215,793,664-byte max RSS, page/outline/gutter all within the canvas
   and no visual clipping. A settled capture cannot establish interactive
   cadence; that last arm needs a human at the live window.
6. **The AppImage is now published for the first time, in v0.10.0, and NOBODY
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
7. **The export save panel wants your eye on macOS** (item 301) — an AppKit
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
  look itself is accepted. **Placement is decided:** each world chooses from a
  small shared roster of safe anchors; shared geometry owns collision avoidance
  and narrow-window fallback. Duration stays shared.

## Remaining work — handoff order (RE-DERIVED 2026-08-11, against the tree)

⚠️ **This section has gone stale five times, each time by editing the previous
list instead of re-checking the tree.** The fifth: item 345 sat on this board as
"ready to merge in one command, held for the user's word" while its merge
(`1e6fd7c8`, two parents, verified) was already an ancestor of `main` — the
word was given for a merge that had already happened. Re-derive from the tree,
every pass.

1. **395/401/404/406/413/418/419 — user decisions landed on the board; dispatchable.**
   Immediate language tag + toast, directory-shaped elision, keep both Back
   gestures while teaching `⌫`, name `esc close` on the unified Go-to surface,
   canonical zoom 1.0, remove the legacy wasm without rewriting history, add an
   openly licensed `⌫` to AwlMarks, unify file/heading/folder navigation under
   Go to, and reserve footer room before candidate rows. Bodies below.
2. **396 — the theme preview's win shrinks with scroll depth.** Recorded from
   388's own measurement, deliberately out of its scope. Body below. Not urgent —
   the shipped state is strictly better than before at every depth.
3. **211 — RUN, AND IT FOUND A LIVE DEFECT. ⚠️ The sitting is DONE; what it
   found is now the work.** Display unlocked, `caffeinate` holding it, and — the
   part that makes the result trustworthy — the lock was checked at BOTH ENDS and
   was clear at both, so this is not the silent zero-frame run the entry warned
   about. `MATRIX RESULT: DEFECT`, exit 1.
   **Two cells fail identically**, `mangrove-to-magpie-burst/settled` and
   `tawny-to-magpie-burst/settled`: `1/223 uniform blocks differ > 30.0`, block
   `(360,280)+40`, expected `rgb(251,251,250)` got `rgb(216,217,216)`, diff 35.
   Same block, same colours, both `-burst` cells, both at SETTLED — so a burst of
   theme previews leaves one block stale where a single hop does not. Every
   non-burst cell passed (commit, revert, early frames). Shots kept under the
   probe's work dir, which is `/tmp` and will not survive a restart — re-run
   `scripts/live-probe.sh --cells "mangrove-to-magpie-burst tawny-to-magpie-burst"`
   to regenerate. **Bisected: PRE-EXISTING, not 388** — the identical failure,
   same two cells and same block, reproduces at `cdb01a6f`, immediately before
   the same-step split merged. 388 is exonerated. Filed as 397.
4. **HUMAN / LIVE — now small.** Seven blockers above, each needing a session,
   hardware, or a release-time word; the 30-item taste backlog closed by the
   bulk acceptance. ⚠️ 392 and 327 wait for dispatch until 388's quiet-host
   timing sitting completes — 388 was dispatched ALONE on purpose.
---

## Open items

### 423 — let outline metadata use the available margin before ellipsizing

User sighting: secondary/date lines in the left outline truncate while visibly usable
horizontal room remains. Preserve the outline boundary and its existing hierarchy, but
extend the metadata text block roughly 30–50 px farther left where the margin permits,
keeping a comfortable 20–24 px minimum inset from the outline. Ellipsize only after that
wider allowance is consumed; the primary label and overall outline composition stay put.

Verify the outline layout at representative narrow, ordinary and wide window geometries,
including long date/secondary strings. Geometry laws pin the minimum inset, prove that the
secondary lines receive more width than today without crossing the outline, and retain
ellipsis when the genuinely available width is exhausted. Capture the reported shape for
pixel confirmation that hierarchy and boundary are unchanged.

### 422 — prose double-click uses linguistic words without changing code words

User decision: on macOS, prose/Markdown double-click and word-granularity drag use
NaturalLanguage's `NLTokenizer(.word)` for English, Japanese and mixed text; code buffers
keep awl's editor-style `is_word_char` rule. Measured on the reported sentence, Apple's
tokenizer returns `大幅 / に / 構成 / が / 変わっ / て / おり`. Linux/web keep today's
English rule and, for an unspaced CJK run, select exactly one extended grapheme instead of
the whole run through punctuation. The platform adapter returns rope char ranges and owns
the UTF-16 conversion; every result still snaps to awl's UAX #29 boundaries.

Verify the shared policy over prose vs code, English apostrophes/hyphens/URLs/Markdown,
Japanese, four-kanji compounds, punctuation, emoji/combining clusters and mixed scripts.
macOS integration tests pin `構成`; portable tests pin the CJK grapheme fallback and prove
ordinary English plus code `snake_case` remain unchanged. No dictionary entry or network is
consulted: native Dictionary Look Up is a separate future action.

### 421 — the cell caret is Latin-height over full-square CJK ink

User sighting: over Japanese `構成`, the cell caret covers only the middle of `成`;
the glyph extends visibly above and below it. `caret_cell_vertical` deliberately uses one
typical-letter height per proportional `(face, row)`, but that Latin-stability rule is the
wrong metric for a full-square CJK anchor. Preserve the stable Latin height and derive one
stable ideographic cell from the resolved CJK face/script — no named-glyph or named-world
branch, and no per-kanji ink-height jitter. Mixed Latin/CJK transitions must remain bounded.

Verify at the pure geometry seam, then render Japanese + mixed-script fixtures through the
ordinary tier-1 capture at both DPIs over the full proportional roster and every cell-form
caret mode. Pixel arithmetic must prove the resting caret contains the CJK ink with its
authored pad and is present; the existing one-height Latin laws stay byte-for-byte green.
Vision-smoke five sampled worlds by asking which Japanese character carries the caret.

### 395 — three small palette/readout decisions — DECIDED

User decisions, ready for implementation:

- **393's palette row stays immediate and keeps no ellipsis.** After applying,
  show a brief confirmation toast naming the resulting document language.
- **327's directory elision keeps path identity.** Preserve at least one `/` and
  a recognizable portion of the final folder rather than returning a string that
  reads as a filename. File-row elision remains unchanged.
- **394 keeps both Back gestures and continues teaching `⌫ back`.** `←` remains
  the spatial shortcut from detail to rail; `⌫` remains the context-aware
  primary Back once the query is empty. Do not spend another footer cell on it.

### 401 — name `esc close` in the Go-to-file footer — DECIDED

Found by item 398's vision smoke and **pre-existing, not this wave's**: the one
affordance question its method could not answer from pixels alone. The Command
palette's footer names `esc close`; Go-to-file's names only
`type to filter ↵ open ←/→ lens`. So "what key returns from here?" is
unanswerable from that surface. `src/overlay/kind.rs`:
`OverlayKind::Goto => vec![enter("open"), key(ARROWS_LR, "lens")]`, unchanged
since at least the item-204 shape — 391 and 394 touched Project and Settings
footers only.

**Decision:** add `esc close` to Go-to-file for parity with the Command palette.
At the minimum width, explanatory prose such as `type to filter` yields before
an actual key affordance. Implement this in item 418's unified Go-to surface,
not as a competing edit to the retiring standalone shape.

### 404 — make zoom 1.0 the single default — DECIDED

Raised by item 397 and **not touched by it**, because the blast radius is every
pixel law in the tree. `app::INITIAL_ZOOM` is **0.8** for a windowed launch; a
replay capture takes `opts.zoom.unwrap_or(1.0)`; `range::ZOOM.default` is a
**third** copy at 1.0. So every headless capture in the suite renders the
document at a size no user sees, and the two doors disagree by 25% — which is
exactly how 397's probe came to grade a live window against a picture 25% larger.

**Decision:** 1.0 is authoritative for launch, capture and configuration. One
owner supplies the value; remove the divergent copies. Re-baseline affected
pixel expectations deliberately.

### 406 — remove the obsolete tracked wasm; keep history — DECIDED

🔵 **LIVE, PUBLIC, AND ONLY HALF FIXABLE BY ME.** Found by item 402, verified
independently: `site/editor/awl-347842567538f209_bg.wasm` is tracked, 43 MB, and
`strings` finds **456 `/Users/frank/…`** occurrences — cargo registry paths baked
into debug info.

The build seam was already fixed: `scripts/with-remap.sh` exists and
`deploy-web.yml` runs `trunk build` through it every deploy. **This bundle
predates the wrapper and was never rebuilt.** The commit that landed the wrapper
claims "no home path baked into shipped binary"; the artifact in the tree says
otherwise. It is also unused — RELEASING calls the checked-in copy legacy and the
deploy assembles a fresh build over it.

**Decision:** remove the unused bundle from the current tree. Do not rewrite Git
history: the strings are a username and Cargo paths, not secrets. Verify the
deploy continues assembling a fresh remapped build without the tracked artifact.

### 413 — add an openly licensed `⌫` to AwlMarks — DECIDED

Found by item 408 while diagnosing a CI red; the diagnosis IS the finding.
**U+232B `⌫` — the Back cell's own glyph, drawn in footers across the product —
is carried by ZERO faces in `assets/fonts`.** Checked with fontTools over every
bundled face: the arrows appear in 32 of them, `↵` in 5 including `AwlMarks.ttf`,
and the erase mark in none. `render.rs` builds its DB with `FontSystem::new()`,
which loads SYSTEM fonts, so that advance came from Apple Symbols here and
DejaVu Sans on Ubuntu — a 2.92% width difference that reddened a gating job.

⚠️ This sits directly against the **never-tofu law**: awl ships its own faces so a
document cannot render as boxes, then draws its own chrome in a glyph it does not
ship. On a Linux desktop with no symbol font in the fallback chain the Back cell
is a tofu box — untested, because every host that has run this has had one.

**Decision:** add U+232B to `AwlMarks.ttf`. The glyph must be original or sourced
under a licence compatible with awl's GPL-3.0-only distribution; record its
authorship/source and licence alongside the font assets. Verify every bundled
font roster resolves the footer through AwlMarks without system-font fallback.

### 418 — one Go-to surface for files, headings and folders — DECIDED

The reported symptom was `workspace = ~`: Switch project opens on 27 folders
while `Browse for folder…` lands below its 12-row window. The decision removes
the extra stage rather than pinning that terminal row.

**One primary surface:** rename `Go to file…` to `Go to…`; fold the existing
Goto and Project corpora into one typed destination list with lenses
`All · Files · Headings · Folders · Recent`. File rows open, heading rows jump,
and folder rows switch the active writing folder. `Recent` combines recent files
and folders; `All` searches every known destination. Folder rows keep clear path
identity. User-facing copy says **folder**, never project; internal names may stay.

**Entry points, not competing systems:** `⌘O` opens All, `⌘⇧P` opens Folders,
and the heading context action opens Headings. These routes share one overlay and
accept seam. Retire the separate public `Switch project…`, `Recent projects…`,
`Browse files…`, and palette `Go to heading…` entries; the contextual heading
wording may remain because it is a local shortcut into the shared surface.

**Opening is separate from going:** expose direct `Open file…` and `Open folder…`
actions. Each immediately opens the platform-appropriate chooser; Open folder
starts at the configured workspace and switches to the accepted folder. The
Folders lens may carry a visible `Choose another folder…` fallback, but it opens
that same chooser directly — never `ProjectBrowse` as a second in-app stage.
This also ends the current macOS inconsistency where File-menu `Browse files…`
opens an AppKit panel while the same catalog command opens an in-app hierarchy.

Verify the command catalog, File menu, context menus, GUIDE generation, default
and rebound chords, macOS/Linux/web platform rosters, all five lenses and empty
states, mixed fuzzy ranking, recent-file/folder order, typed-row accept effects,
chooser cancel/accept, session/workspace re-scope, sidecar semantics, and the
absence of any remaining public project/browse wording. Read `docs/config.md`,
`docs/platform.md`, `docs/render.md`, and `docs/harness-reach.md` before writing
the capture/live Verify split.

### 419 — reserve footer room before candidate rows — DECIDED

Found by item 414's sweep, which was the first to cross the world axis on this
geometry, and **ledgered rather than fixed** — the item was a law, not a repair.

At `464x288 logical, zoom=1.4, menu_bar=on` — the app's own enforced minimum
window — one geometry has three outcomes decided by the world's `list_style` row
gap: `Pane` and `Diagonal` (16 worlds) fit at demand **0.9849**; **`Rules`
(Paperbark) reaches 1.2433 and is drawn past the card's bottom edge**; **`Bars`
(3 worlds) never lays the footer out at all.** Nine spilled cells and thirteen
starved ones are now pinned, and **six of the nine and eleven of the thirteen are
on `bars`/`rules` cells no previous law could see.**

⚠️ Not a host difference and not new — it has presumably shipped for as long as
those pitches have. What was new is that anything looked. The ledgers are
two-sided, so a repair reddens them and must delete its entries deliberately.

**Decision:** reserve the teaching footer before allocating visible candidate
rows. When space is tight, show fewer Settings rows; preserve each world's row
rhythm, never draw beyond the card, and do not silently omit the footer. Apply
the same composition law to Rules, Bars and the narrow History shape; replace
the two-sided defect ledgers with outcome laws.

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
- **Switch-project spatial redesign (two-column Miller / frame-carried
  breadcrumb), raised in the 2026-08-12 design session and NOT settled.** The
  question: whether the browse fallback deserves Finder-style columns, or the
  Frame should carry orientation (DESIGN §2 names orientation as the Frame's
  job), or item 410's row is enough. Recorded constraints for whoever picks it
  up: `←/→` is the LENS rail on every picker and Miller wants it for
  ascend/descend — breaking the shared grammar on one picker is item 401's
  "two pickers disagree" defect by construction; the card's width cap and the
  minimum window already price out a second column of real rows; and
  type-to-filter, not spatial browsing, is the picker's primary gesture.
  Re-audition after 409+410 land; live feel is the oracle.
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
