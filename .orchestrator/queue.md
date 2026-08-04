# awl — live build queue

> Live execution state only. Completed and superseded work is in git history
> (`git log -p .orchestrator/queue.md`). Protocol, claiming, worktrees, and
> execution hygiene live in `.orchestrator/README.md`.

## ⚠️ RELEASE IS NOT SHIPPABLE TODAY — item 226's dry run, 2026-08-03

**The release pipeline could not have built anything.** Landed at `3160e309`.
Found BEFORE a tag was attempted, which is the whole point of the item.

1. **The whole pipeline was dead.** `.cargo/config.toml` gained
   `rustc-wrapper = "sccache"` on 2026-07-30 (`d76eaaaa`) — **after** the last
   dry run on 2026-07-11. `ci.yml` installs sccache in all four jobs;
   `release.yml` in **none**. Reproduced in a clean container: `could not
   execute process sccache (never executed)`, exit 101, at `rustc -vV`. The
   first tag ever pushed would have failed to compile, publicly.
2. **`publish` could never publish.** The repo's default workflow permission is
   `read` and the job declared none. **Only a tag exercises that path, so no dry
   run could ever have found it** — it took reading the API.
3. **`deploy-web.yml` has the identical hole.** Found by the new law, not by
   looking. That law sweeps the axis a release-only check would have missed:
   every workflow JOB running cargo or trunk must install the wrapper first,
   across all workflow files — `ci.yml` was correct throughout, so a
   release-scoped check would have gone green.
4. **The tarball would have shipped with no `LICENSE`, silently.** Fonts
   (OFL 1.1) and Hunspell dictionaries (LGPL-2.1) are `include_bytes!`d into the
   binary; neither packaging path copied their audits while `RELEASING.md`
   asserted all five docs rode the artifacts — `cp X 2>/dev/null || true`
   swallowed it. No GPLv3 §6(d) source offer either. Now a **hard packaging
   failure**, mutation-proved.
5. **Portability, measured not estimated:** the binary requires `GLIBC_2.39`,
   proved by running the produced tarball on `debian:12`. **Excludes Debian 12,
   Ubuntu 22.04 LTS and RHEL 9.** Build base deliberately unchanged — a
   support-matrix decision; `RELEASING.md` §5 has the four options.

**Linux-only beta is structural now:** `mac` and `web` build on a dry run and
are SKIPPED on a tag, so an unsigned `.app` cannot reach a Release.

## 🔒 MEASURED 2026-08-04: THE DISPLAY IS LOCKED, SO EVERY LIVE ARM IS BLOCKED

**Checked, not assumed:** `ioreg -n Root -d1 -a | grep -A1 CGSSessionScreenIsLocked`
returns **`<true/>`**. `caffeinate` is already running (`pmset -g` shows
`sleep 0 (sleep prevented by caffeinate)`), and that is the trap worth naming:
**`caffeinate` prevents sleep and CANNOT unlock a locked screen.** Offering
caffeinate does not unblock a live sitting; only unlocking does. Screensaver
`idleTime` is still **300** and `disksleep` is 10.

**What this blocks right now, all of it needing one unlock rather than one
decision:** 118's world-loudness confirmation and its `--release` ambient
sitting; 211's unoccluded glide confirmation and its unreached sweep arms;
**207's real VoiceOver journey**; **218's VoiceOver sitting** (new this
wave); **241's one-run discriminator** — `AWL_THEME_FONT_DEBOUNCE_MS=0` from
Kite, which is the cheapest thing on this board and decides between three
candidate causes; and **244's `--release` judgement** of whether the new
companion breathe reads as a flash.

⚠️ **TWO CORRECTIONS TO THAT LIST, both made 2026-08-04, because a
misattributed blocker never gets cleared.** (1) **207's AT-SPI journey is NOT on
it** — AT-SPI2 is the *Linux* accessibility API (`ACCESSIBILITY.md:65`), so no
unlock of this Mac can reach it; it needs a Linux session with Orca and is now
**item 251**. Only 207's VoiceOver half belongs here. (2) **241's discriminator
is not a perception arm** — it is one run of `AWL_THEME_FONT_DEBOUNCE_MS=0` and
a HUD reading, so an AGENT can do it the moment the screen is awake; it sits in
this list only because it needs presented frames. The genuinely human arms are
118, 211, 218 and 244, of which only 118 has a head start (an independent agent
map, `1, 10, 3, 4, 1`, mean 2.68, to diff against rather than re-derive).

⚠️ **Do not let a lane "run" any of these under the lock.** `--live-script`
writes successful-looking `LIVE-PROBE shot … ok` lines while presenting **zero
frames** under a lock, and `live-probe.sh` only checks the lock in preflight —
which is exactly how the 2026-08-02 sitting was silently invalidated seven
minutes in. **Re-check the lock at BOTH ends of any live run.**

## 🔵 BLOCKED ON THE USER — nothing else can close these

⚠️ **This section has now been silently deleted TWICE** — once by an
orchestrator `git add -A` sweeping another tool's in-flight edit, once by the
item-204 worker's own commit `1127673d` despite its brief forbidding board
writes. **After every merge, verify this heading still exists.** If it is
missing, `git log -S"BLOCKED ON THE USER" -- .orchestrator/queue.md` finds who
took it.

1. **118 — the world-loudness map and the `--release` ambient sitting.** This
   blocks no development or integration; it is a pre-release taste check. The
   Done clause requires a USER-CONFIRMED map; pixel arithmetic may prove
   territory and contrast but never the taste score. An independent agent map
   exists (`1, 10, 3, 4, 1`, mean 2.68) to diff against rather than re-derive.
2. **The tag itself, and the site deploy.** Both are the user's explicit word,
   every time. See the release section above for what must be true first.
3. **The release support matrix (item 226 §5) — the glibc floor.** Measured: the
   binary needs `GLIBC_2.39`, which excludes Debian 12, Ubuntu 22.04 LTS and
   RHEL 9. `RELEASING.md` §5 has the four build bases and what each reaches.
   **Decide it together with item 227's AppImage**, which may make it moot for
   the friendly download while the tarball stays technical. Related and also
   yours: item 228 wants `0.9.0` in artifact names, which **cannot both hold**
   with the unversioned `/releases/latest/download/` URL the site hardcodes.

⚠️ **Before any live sitting: `displaysleep` is 10 and screensaver `idleTime`
is 300.** That is what silently invalidated the 2026-08-02 attempt seven minutes
in. Hold the display with `caffeinate -d -i -t <seconds>` and re-check the lock
at BOTH ends — `live-probe.sh` only checks in preflight, and `--live-script`
writes successful-looking `LIVE-PROBE shot … ok` lines while presenting zero
frames under a lock.

## Latest design decisions

**TWO USER CALLS — 2026-08-04, the rotating-mark session. Items 247 and 248.**
The session started from a real elevator's up/down chevron, which spins in 3D.
It was refined against the tree rather than accepted as a look, and the refining
is the part worth keeping: **(1)** the first proposed home — the fold chevron —
was rejected as the SITE for the spin because `chevron_revealed` puts the mark on
the caret's own row by construction, so the flourish would compete with the one
element DESIGN.md grants motion; **(2)** the user then identified the real site
from a screenshot, the diagonal palette marker, which turned out to already BE
two rotated quads on `prepare_rotated` with genuine up/down selection semantics —
eliminating the font, shader, tofu, contrast-floor and caret-collision hazards
the first siting carried, and reducing the work to animating endpoints. **The
call: 247 gets an authored symbol that turns (Mangrove and Magpie only); 248
animates the existing `›` on fold/unfold in ALL worlds.** 248 keeps the marker
glyph the user named, so its axis is the in-plane quarter turn `›`→`⌄` — the
only axis that works on a mark that is not left-right symmetric, and the one that
fixes the chevron's direction-blindness at the same time. ⚠️ Both items are
gated on the same rotatable stroked-mark primitive; whichever lands first owns
it. A third axis discussed and deliberately NOT queued: a `v → | → v` spin about
the vertical axis, which returns the mark to itself and would therefore read as
"acknowledged, nothing changed". It has no referent in awl today — zero-network
is a design invariant and nothing is ever loading — so it is parked rather than
invented a use for. Revisit only if a genuine indeterminate state appears.

**FOUR USER CALLS — 2026-08-03. All four were the open questions the 2026-08-03
queue review put to the user; each is recorded in its own item's body too.**

1. **238 — the rename is ADOPTED, `git remote` included.** "awl-editor is much
   better." So this is no longer a should-we: every shipped artifact names
   `Frank-P-Lu/awl-editor`, and the remote is repointed rather than left riding
   GitHub's redirect. ⚠️ **The item's law clause needs correcting before anyone
   writes it** — see 238's body: a grep-law over the bare token `awl-next` would
   be WRONG, because the local working directory is still `awl-next` and four
   Rust test fixtures legitimately use it as a sample project name. The law bans
   the old **repository URL**.
2. **222 / 131d — Magpie's labels: RIGHT-ALIGN the name text for ascending
   worlds, and MIRROR THE WHOLE CLUSTER, not just the rail.** This closes the
   taste call the 222+223 lane deliberately left open. The cluster is the unit
   that mirrors — label, gap and accessory together — so a short label on Magpie
   no longer leaves a gap between itself and its chord. It lands inside 131d's
   measured cluster rail, not as a patch on 222.
3. **230's residue — a separately-named `THROUGH VIEW` figure is DECLINED.**
   Closed on purpose, not deferred. Item 230 is complete as it stands: one owner,
   one text, drawn and announced agreeing. Do not re-propose it without a new
   reason — the recorded reason for declining is that the card earns its calm by
   carrying few figures, and "how far through what I can see" is a second answer
   to a question the reader already has one answer for.
4. **229 — CJK word count: COUNT IDEOGRAPHS AS TOKENS, and let the UNIT LABEL
   FOLLOW THE DOCUMENT'S DOMINANT SCRIPT** — "words" when the script spaces its
   words, "characters" when it does not. So the readout does not claim a word
   count for a script that has no spaced words; it changes what it is counting
   and says so. The mixed-script case now has a stated rule to satisfy rather
   than being left to fall out of the implementation.

**Item 116d — 2026-08-02, THE COMPOSITING CALL, and 116d is now UNBLOCKED.**
The comparison sits **ON the workspace surface**, not as a window through it.
The card stays one opaque surface — "A WORKSPACE IS ONE SURFACE" survives
intact — and the relocated document layer draws **after** the card into the
carved region, **without re-drawing its own ground**. The rejected arm was the
one the code itself had already found the defect in: a hole punched in the card
would show the *backdrop's* ground, because the ground punch is at the page
column and not at the region, and on a blur-eligible world `backdrop_blur()`
frosts the frame *around* the workspace — exactly where the region is not. So
"through" would have meant fixing two compositing bugs to reach a worse answer.
**116b's boundary law
`the_relocated_document_is_geometrically_placed_but_not_yet_composited` is now
the thing to delete and replace** with the containment-and-visibility law its
own message asks for. Painter's order gains a second document pass; the region
must be proven to contain it, in every world.

**Items 114 + 116d — 2026-08-02, the workspace Esc, settled once for BOTH
members as item 114 asked.** **One Esc always leaves.** Esc dismisses the
workspace from anywhere inside it; focus moves between the rail/timeline and
the content pane on `Tab`/`Shift-Tab` alone. A child audition summoned *out of*
a workspace (Settings → Theme picker) is a genuinely different rung and keeps
its own Esc-returns-to-parent behaviour. The rejected arm — Esc unwinds one
rung, so leaving History from the comparison takes two presses — was rejected
because the comparison is exactly where a reader spends their time, and Esc
would then mean two different things depending on where focus sits. **116d owes
History an explicit Back affordance in the footer**; 116c's `⇧↵`/`open_keep_version`
groundwork is already shaped for it.

**Item 132 / item 118 — 2026-08-02, the contradiction is RESOLVED in favour of
132.** **Kite is a 5/5.** The roster's target distribution is amended from
`1, 7, 7, 4, 1` (mean 2.85) to **`1, 7, 6, 4, 2` (mean 2.90)** — two deliberate
statement worlds, Firetail and Kite. Item 118's "the gap is the middle, not more
5s" was written before Kite was commissioned and is superseded on that one
clause only; the rest of its direction (calm bias, hover around 3, no
theme-park bell curve) stands. Recorded here because item 118's own audit found
these two user decisions could not both hold and asked for the call before Kite
was built rather than after.

## Ready — current user-visible wave

247. **Mangrove and Magpie: the diagonal selection marker becomes an authored
symbol that TURNS as the selection travels, and its turn direction says which
way you moved.** **Build:** `render/chrome/diagonal.rs::prepare_diagonal_spine`
draws the selected row's marker as two `crate::selection::spine_segment` calls —
a vertical tick at `spine_x` spanning the row, plus a horizontal connector out to
`label_left` (Descending) or `accessory_right` (Ascending) — which composes to a
`⊢`. Replace it with an authored mark built from the SAME primitive, and rotate
that mark about its own pivot while the selection travels: one way for down, the
other for up, settling to rest on arrival. A wrap (last row → first) takes the
long way round, so a wrap stops being silent. **The turn direction is
information that does not exist today** — in a long filtered list you currently
just appear somewhere else. **Scope:** These two worlds only. `ListStyle::Pane`
and `ListStyle::Bars` have no spine and are untouched, so this is world
expression, not universal grammar. `DiagonalDirection::sign()` is the existing
per-world dial and already returns ±1.0 — Mangrove's `\` and Magpie's `/` get
mirrored turns from it rather than from a second authored constant. **The
marker has no travel animator today:** `overlay_selection_rects` returns
`OverlaySelectionRects::default()` for `ListStyle::Diagonal` and the spine path
reads `vis.reads_selected(row.display)`, a bool — so the mark TELEPORTS while
Pane worlds get `VisualSelection::living()`'s interpolated `(from, to, t)` band.
Give Diagonal a travel phase **through that same `VisualSelection`**, never a
second animator: the Pane band's own doc explains that re-running an animator
lets the fill land on a different row from the ink shaped against it.
⚠️ **Traps, all pre-existing and all load-bearing here.** (a) A new dt-consuming
stepper MUST join `TextPipeline::advance`'s OR-fold — `motion.rs`'s own doc
warns that a fourth animator outside that seam is silently ungated by Reduce
Motion. (b) `narrowed_spine_corner_px` exists precisely to cap a segment's corner
radius by its half-length; a foreshortening arm that bypasses it will over-round
as it shortens. (c) `spine_segment` guards zero length — an arm passing through
zero must not yield a NaN axis. (d) Reduce Motion settles instantly to the SAME
final state, so the mark's RESTING orientation must carry everything the turn
says; if the direction cue only exists mid-flight, the animation is load-bearing
and `motion.rs`'s law is broken. **Done:** Moving the selection in Mangrove or
Magpie turns the marker in the direction of travel and settles it; a wrap is
visibly distinct from a single step; Reduce Motion shows the same settled marker
with no in-between frames; the other 18 worlds are byte-identical. **Verify:**
Sidecar for the marker's settled orientation and the selected index (state);
pixel arithmetic over `--screenshot` PNGs for the mark's geometry and its ink
contrast against Mangrove's and Magpie's own grounds (the sidecar is a state
oracle, not an appearance oracle). In-flight frames need `--screenshot-motion`;
an ordinary capture sees only the settled state. Sweep BOTH directions and the
wrap, in both worlds. **Routing:** Deep tier (Opus, high) — authoring the symbol
is taste work and the README routes taste above production. Follow with a Fable
visual-judge pass over real gallery captures.
✅ **SUB-DECISION RESOLVED — USER CALL 2026-08-04: THE SYMBOL IS THE STROKED
CHEVRON** (option (i) as recommended). Vertex on the spine, arms opening toward
the label. **The derivation is worth keeping, because it is arithmetic rather
than taste and it constrains any future revisit.** The drawable alphabet is
fixed by the primitive: `spine_segment` yields ONE rotated ROUNDED RECT, and
`set_corner`'s radius is clamped by the shader to `min(hsize.x, hsize.y)` — so
the vocabulary is straight strokes plus, at `length == thickness` with full
corner, dots. **No curves, no arcs, no fills, no glyphs.** The canvas is
~`10.0` logical px of connector reach at `3.0` weight against a row-tall tick —
about three strokes before it turns to mud. And the read-at-every-angle
requirement eliminates candidates ARITHMETICALLY, not by preference: **a plain
bar has 180° rotational symmetry, so half a turn is indistinguishable from no
turn** — which also kills the plus, cross, diamond, square and asterisk. The
chevron is the simplest mark with NO rotational symmetry, and it is directional
AT REST, which is what the Reduce Motion clause above actually requires.
**Rejected: the arrow** (shaft + two barbs) — most explicitly directional, but
three strokes at `3.0` weight inside `10.0` px puts each barb near 4px and it
muds at 1× DPI. **Rejected: tick + orbiting dot** — calmest, but a dot is too
weak a direction cue at this weight. **Rejected: `⊢` with the connector alone
swinging** — most conservative, but a horizontal connector says nothing at rest
and so fails the Reduce Motion clause unless the resting angle itself tilts.

248. **The fold chevron tells you which way it goes, and animates the change —
every world.** **Build:** `render/layers/fold_chevron.rs` is direction-blind
today: `FOLD_CHEVRON` is one const (`"\u{203A}"`, `›`) and
`fold_chevron_geometries` filters only on `fold::chevron_revealed` and
`line_ornament_visible`. Nothing reads fold state, so a collapsed heading and an
expanded one draw the IDENTICAL mark — the only collapsed signal is the separate
"… N lines" tail, which by construction only exists once you have already
folded. **Fix the information first:** `›` while the section is hidden, `⌄`
while it is showing. Then animate the quarter turn between them on fold/unfold.
⚠️ **glyphon 0.11 carries no transform of any kind** — `TextArea` exposes
`left/top/scale/bounds/default_color/custom_glyphs` and nothing else, so a
shaped run CANNOT rotate. The mark must leave the text pipeline. Build it from
`spine_segment` + `prepare_rotated` — two arms meeting at a vertex, rotated
about that vertex — the SAME primitive item 247 uses. Same behavior ⇒ same code:
whichever item lands first owns the shared rotatable-mark primitive and the
other consumes it; do not grow two. **This also retires three latent problems
the glyph carried:** `panel_attrs()` is the ACTIVE WORLD'S FACE
(`chrome/panel.rs:111` says so outright), so a functional affordance's SHAPE
currently varies across 20 worlds; `⌄`/`⌃` are the same tofu class that already
forced `panel.rs`'s `SYMBOL_FAMILY` escape hatch for ⌘/⌥; and an authored stroke
weight sidesteps the IBM Plex Mono weight-300 tripwire entirely. **Scope:** All
worlds — user decision. The direction information is universal grammar and never
varies; the turn's duration and character may be a per-world dial, the way the
ink already routes through `theme::fold_afford_chevron_ink()`.
⚠️ **THE CARET COLLISION, recorded so it is not rediscovered mid-build.**
`fold::chevron_revealed(line, cursor_line, hover_line)` reveals the mark when the
caret is ON that heading's row, or on hover. So on the KEYBOARD path the mark and
the caret are co-present BY CONSTRUCTION, a few characters apart in one row — and
the caret is the one thing DESIGN.md grants motion (`motion.rs`'s own doc calls
it "deliberately the one thing"). Value separation handles the static case
correctly; motion is not separated by value. **Recommended mechanism, not a
scope change:** animate on the POINTER path and snap on the KEYBOARD path. That
is DESIGN.md §"Motion follows importance" applied literally — keyboard folding is
the frequent path and gets zero delay, pointer folding is occasional and
exploratory and gets the expression — and it means the turn only ever plays when
it is the only moving thing on the row. If a live `--release` sitting says the
co-present animation reads fine, promote it; that call is the user's.
⚠️ **The contrast law is the non-vacuity proof.** `capture/tests/folds.rs`
already pins this mark's ink against the page ground it actually renders on — an
audit found Mangrove's chevron at ~1.5:1 and Firetail's tail at ~1.4:1,
effectively invisible, and fixed exactly those. A turning mark passes through its
THINNEST coverage mid-turn, which is precisely where it will fall back under the
floor. **Run the arithmetic at the thinnest frame, not only the settled ones,**
and prove non-vacuity by breaking the mark and watching the law go red. **Also:**
join `TextPipeline::advance`'s OR-fold (`motion.rs`), and make DIRECTION sidecar
STATE rather than an animation artifact — a capture with zero animation frames
must still report which way the chevron points, or Reduce Motion loses the
information the item exists to add. Check that the mark's row box is resolved
AFTER the fold's own `row_geom` invalidation: `fold_chevron_geometries` reads
`visual_rows(h.line).first()`, and folding changes row geometry. **Done:** A
heading's chevron says whether its section is hidden or showing, before the
first click; toggling turns it; Reduce Motion shows the correct settled
direction with no in-between frames; every world renders the same mark shape.
**Verify:** Sidecar for chevron direction per heading (state, and the Reduce
Motion arm); `--screenshot` pixel arithmetic for shape and for the contrast
sweep across the full world roster at the thinnest frame. `--keys` drives
`C-c C-f` through the real keymap. ⚠️ The pointer path is live-only — check
`docs/harness-reach.md` before promising a capture over hover state.
**Routing:** Production tier (Sonnet medium) for the implementation; the
contrast sweep is an audit and gets its own probe per the standing policy.

## Active claims — 2026-08-03 afternoon wave

**Dispatched 2026-08-03, four lanes. `main` CI is red on the KNOWN item-231
wedge and nothing else** — run `30778330964` (`69f379f7`): `web`, `linux` and
`mac live-probe` all green; `mac (build + test)` alone died as
`native-gate: ABORTED on its 1500s budget with mac_status=143 linux_status=143`.
That is 231's signature, not a new regression, so integration is not blocked —
but it is also exactly the uninformative red item 243 exists to end, which is
why 243 is in this wave. Local `main` is 6 board-only commits ahead of origin;
they ride this wave's first train.

- **116d (the flip slice)** — ⛔ **CLAIM WITHDRAWN: THE WORK WAS ALREADY DONE.**
  Dispatched off stale board text, returned without writing product code, which
  is the correct outcome. **The file hold is RELEASED** — `overlay_draw.rs`,
  `overlay_rows.rs` and `chrome/mod.rs` were never touched, so **174 is not
  blocked by this wave.** The one commit is docs-only and is merged at
  `1bb20071`; see the board-integrity note below, which is the real finding.
- **218** — ✅ **COMPLETE except its live sitting.** Merged `c282cedd`, pushed,
  CI green. Receipt
  `native-gate-receipt commit=441fdf141c87707706bf02c7864f8b0bfdba2e41 conventions=mac,linux scope=all-targets`,
  `web-smoke: OK`. ⚠️ **This entry sat stale at 🟡 IN PROGRESS for the rest of
  the session — the orchestrator merged the work and never wrote the landing
  note, and the USER caught it, not the board.** Same class as `16b4e8c2`'s 116d
  drop earlier today: **a merge is not done until its board entry says so.**
  **Measured, with the pre-change path timed live in the same run rather than
  remembered — per keystroke, release:**
  100 lines `0.175 → 0.005 ms`; 1 000 `1.789 → 0.009`; 10 000 `18.07 → 0.072`;
  **50 000 `93.33 → 0.476 ms`.** 93 ms/keystroke at 50k lines is the "awl is not
  responding". Runs rebuilt and nodes published are **flat at every size**, and
  the laws assert those counts rather than the clock.
  **Two things the item did not anticipate, both found by the owner:**
  incremental updates alone were not the fix — the snapshot *build* was also
  O(document) per frame, so the run table had to reach into `Buffer` at its
  three rope-mutation sites; and a third O(document) term hid in `fold_passive`,
  computing whole-document card figures every frame **even with no card open**.
  Ten laws, each watched failing by name, two of which catch a stale run being
  read aloud — swept over every offset and every ordered pair of run edges in a
  fixture with combining marks, a ZWJ family and flags across line breaks, plus
  a 2 000-step random walk. Semantic schema `awl-semantic/1 → /2`.
  **The wasm gate earned its keep exactly as the standing rule predicts:** the
  change looked native-only and broke the wasm build; nothing else would have
  caught it.
  🔵 **OWED — a real unlocked VoiceOver typing and navigation sitting.** No test
  tier stands in for it; `ACCESSIBILITY.md` records it as owed rather than done.
  Honest residual for whoever runs it: 0.476 ms at 50 000 lines still grows with
  the document, and whether that is perceptible to a VoiceOver user is
  unmeasurable without the sitting.
- **247 (the STATIC chevron mark only)** — 🟡 IN PROGRESS — claude (opus, high),
  **in the MAIN working tree, not a worktree** (this session is configured to
  work in place; recorded here because the standing protocol assumes a worktree
  and the divergence should not have to be inferred). Scope is deliberately the
  RESTING mark alone — replace the two-segment `⊢` with the decided stroked
  chevron in `render/chrome/diagonal.rs` and capture it at real size in both
  Mangrove and Magpie. **The turn, the travel phase and the `advance()` OR-fold
  are NOT in this slice** and 247 stays open after it: the symbol was chosen from
  a description, and the user should see pixels before motion is built on top.
  File hold: `render/chrome/diagonal.rs`.

⚠️ **BOARD-INTEGRITY DEFECT — this wave's first lane was spent on it, so it is
worth reading before the next compression.** `16b4e8c2` cleared finished items,
which the board's own header sanctions. But for 116d it **kept the superseded
intermediate bullet (`🟢 COMPOSITING ROUND LANDED; the flip is deliberately NOT
done`) and dropped the `✅ COMPLETE` entry that replaced it** — and then stated
in its own commit message that "231, 174 and 116d are open and kept verbatim."
116d was not open. Item 116's parent entry was likewise kept verbatim in its
pre-flip state, still reading "116d CANNOT flip `workspace_shape(History)`."
Claim `0a1dd593` was written off that text in good faith and was wrong.
**Verified against the tree, not the log:** `workspace_shape(History)` is
`TimelineOverComparison` at `src/overlay/workspace.rs:129`, and
`workspace_header_beat` is absent from `src/` entirely. The rule this breaks is
not "don't compress" — it is that **clearing an item must clear ALL of its
bullets, or the oldest survivor becomes the board's answer.**

⚠️ **THE RECEIPT GAP IS WIDER THAN THE NOTE BELOW SAYS — 116d joins it, and its
string is UNRECOVERABLE.** The compression deleted six `native-gate-receipt`
strings that appear in no commit message. Most are recoverable from
`git log -p .orchestrator/queue.md`; 116d's was **already truncated when written**
(`native-gate-receipt commit=86d73aa3… conventions=mac,linux scope=all-targets`),
so the full string is gone for good. `86d73aa3` is real and is a descendant of
116d's merge `a8eef4ee`, so a gate did cover a tree containing the work.
**Standing fix, cheap and permanent: put the receipt in the MERGE COMMIT
MESSAGE, not only on the board.** Every recurrence of this gap has the same
cause — the board is the only copy, and the board gets compressed.
- **243** — ✅ **COMPLETE.** Merged `1833757b`. `mac (build + test, minus
  render::tests)` gates with no `continue-on-error` at any level;
  `mac (render::tests)` carries it at the **job** key and names item 231 in its
  own job title, so the red is attributable from the workflow file alone.
  No Rust changed, so applicable arms only — code-health, actionlint, a YAML
  parse, and one targeted test. **No native tier claimed, correctly.**
  Clause 3's real sub-claim was proved the right way: the gating job's own step,
  run character-for-character, included and failed a broken `dateformat` test
  (2865 passed, 1 failed), so `--skip render::tests` demonstrably does not
  shield it — "cargo test fails" alone would not have proved "the job fails".
  ⚠️ **CLAUSE 1 IS OWED and can only be paid by the first `main` run after this
  lands**, because `ci.yml`'s push trigger is `branches: [main]` and worktree
  branches never push. **Check it on the next push.**
  ⚠️ **Consequence to absorb, not lose: NO hosted-mac arm prints a
  `native-gate-receipt` anymore.** `native-gate.sh` forbids filtered invocation
  by design, and both new jobs are filtered. Nothing parses that string from CI,
  so nothing breaks — but a human could previously read it as informal
  confirmation that the exact commit passed the full suite on virtualised Metal,
  and that no longer exists in any form. This is a **different** gap from the
  RECEIPT GAP below (which is about local pre-push receipts); it is a candidate
  for its own item, deliberately not absorbed silently.
  One brief correction from the lane, worth carrying: pushing a worktree branch
  runs **nothing** — `on.push.branches` is `[main]` only.
- **240** — ✅ **COMPLETE.** Merged `174f570d`; worktree removed. Receipt, and it
  is **in the merge commit message, not only here** —
  `native-gate-receipt commit=f0e8b46670e49979611278652bcf60454c1c0974 conventions=mac,linux scope=all-targets`,
  plus web smoke `16 passed`. The gate named `f0e8b466`, whose base differs from
  the merge candidate by two markdown files and **zero `.rs`**, so the native
  scope carries; that reasoning is recorded in the merge message rather than
  asserted here. The hand-kept 4-of-9 list is now one directory-driven sweep
  that reads `shaders/` and extracts entry points from each source's own
  `@vertex`/`@fragment` attributes, so a tenth shader **or an eleventh entry
  point** cannot be added without validation. **No live web defect: all 9
  shaders / 21 entry points pass GLSL ES 300** — the five uncovered ones
  (`blur`, `caret`, `caret_glyph`, `image`, `spellunderline`) are clean, so this
  is a coverage fix, not a defect fix. Mutation-proved **twice**: a real
  GLES-310-only construct (`pack4x8snorm`) in the previously-uncovered
  `caret.wgsl`, and a throwaway shader named in no list, which the sweep caught
  with no test-code change. The orchestrator re-ran the sweep on the merge
  candidate rather than taking the report on trust.
- **229** — ✅ **COMPLETE.** Merged `d8ae72c9`. Receipt
  `native-gate-receipt commit=e5c353775b72697959d7d9f5c6b3ee9b81418c0d conventions=mac,linux scope=all-targets`,
  `web-smoke: OK`, code-health and `cargo fmt --check` clean. A 5,500-character
  Japanese manuscript reported `1 word · 1 min`; it now reports
  `5500 characters · 28 min`.
  **Dominant script (decision a) = a strict majority of the BODY's own
  characters carrying an unspaced script** — Kana/Han/Bopomofo, **Hangul
  deliberately excluded** because Korean spaces its words. A tie, including an
  empty document, reads `Words`. **Both rejected alternatives are recorded in
  the doc comment rather than silently dropped:** a token-majority rule would
  label a document "characters" over one short inserted phrase, since every
  ideograph is its own token; and the frontmatter `lang:` tag is declared intent,
  not a report of what is written — the existing fixture tags `lang: ja` on
  space-separated English and must not read "characters". The counting rule runs
  regardless of the label, so the mixed pinned case goes **4 → 14** tokens while
  staying labeled "words", which is exactly what the user's decision asks for.
  **`SCHEMA_VERSION` 197 → 198** (decision c): `readout` and `hud` each gain a
  `unit` field, both fed by the one owner `card::figures::{readout_figures,
  CountUnit}` per item 230's seam. No second counter — decision (b) held.
  Mutation-proved twice, across the figures, HUD-row and sidecar-agreement laws.
  **Unanticipated and handled well:** `card/figures.rs` and `card/content.rs`
  crossed the 500-line production ceiling with no grandfathered escape, so each
  decomposed into a directory with a sibling `tests.rs`.
- **238** — ✅ **COMPLETE.** Merged `c1a9d5ce`. Receipt
  `native-gate-receipt commit=7c89c07e2f09350c7abb2d4efc3a47bff39b13db conventions=mac,linux scope=all-targets`
  (in the merge message too), web smoke `16 passed` / `web-smoke: OK`,
  `site-links.sh` green on 12 + 12. **The GPLv3 §6(d) source offer in every
  release tarball no longer depends on GitHub's redirect staying up** — that was
  the highest-stakes line in the item. 30 sites fixed, matching the survey floor
  exactly, including `site/check.js`'s `RELEASES_URL` and 8 in `site/llms.txt`
  that the item named nowhere and no HTML sweep would have caught.
  **Both traps were established, not concluded:** `ci-mac-bisect.sh` was fixed
  on its own unmerged branch (`584b4a7b`) and proved not to be a trap by copying
  the *original unfixed* file in and watching the law fail by name on all three
  lines; `site/editor/` was cleared by reading `deploy-web.yml`'s assemble step
  (`rm -rf` then fresh `trunk` output) and the `flyctl` working directory, so the
  live deployment never serves the stale bundle.
  ⚠️ **Its own code-health run caught a real defect before integration** — four
  doc comments citing "item 238", against the comments-aren't-history
  convention. Fixed at `1549a6fd`.
  ✅ **`git remote` IS REPOINTED** — the orchestrator did it before dispatch and
  verified it without a redirect (`git ls-remote --heads origin main` →
  `69f379f7`). It is config, not a commit, so it appears in no diff; the worker
  owns only the tracked-file surface and the law.

✅ **TRAIN PUSHED 2026-08-03/04: `69f379f7..3b354d3a`.** Green train on the exact
combined candidate —
`native-gate-receipt commit=3b354d3af731c50533bd898a92483bd3e3719e84 conventions=mac,linux scope=all-targets`,
0 failures, plus `web-smoke: OK` and code-health clean. **CI run `30825396088`
confirms item 243's split is LIVE**: the job list now reads
`mac (build + test, minus render::tests)` and
`mac (render::tests) — allowed failure, item 231`.

✅ **`main` IS GREEN AND STAYED GREEN THROUGH A RED AND A REVERT.** Second
confirmed green: run `30848110830` (`37098cb4`, item 245 plus the 239 revert) —
all four gating arms success, `mac (render::tests)` failure and tolerated. The
split has now correctly reported three distinct states in one session: a real
regression (the clippy arms), a bad new law (239's oracle), and green.

✅✅ **ITEM 243 IS COMPLETE — CLAUSE 2 PAID, AND `main` IS GREEN FOR THE FIRST
TIME IN THE RECORDED WINDOW.** Run `30838810157` (`76903fc1`) concluded
**`success`** with `mac (render::tests)` at **`failure`** and all four gating
arms green. **A tolerated wedge failure now leaves the workflow green, which is
the whole thing the split was for.** Checked rather than assumed:
`gh run list --branch main --limit 25 --json headSha,conclusion` returns
**exactly one** success, and it is this run — so after the ~140-commit red
streak and everything since, this is `main`'s first green.
**Both of item 243's owed clauses are now closed by real runs, not by parse:**
clause 1 on `30825396088` (the gating arm passing on a hosted runner) and
clause 2 here. **When item 231 lands, `mac (render::tests)` goes green and is
promoted to gating with no further decision needed** — that is why the shape
only had to be decided once.

🔴 **243's VERIFY CLAUSE 2 FAILED ON ITS FIRST REAL RUN — repaired at
`da70df93`, and the mechanism is worth knowing repo-wide.** Run `30825396088`
concluded **`cancelled`** with **every gating arm green**:
`mac (build + test, minus render::tests)`, `linux`, `web` and `mac live-probe`
all success; only the tolerated `mac (render::tests)` was non-success, at
`cancelled` after 60m33s. **`continue-on-error: true` tolerates a job that
FAILS. A step or job exceeding `timeout-minutes` is CANCELLED, not failed, and
cancellation propagates to the run's conclusion regardless.** So `main` still
did not read green — the exact uninformative signal the split existed to end.
**This is precisely why the 243 lane refused to claim clause 2 from a YAML parse
and named it owed; the parse was right about the shape and the shape was
insufficient.** Fix: `scripts/ci-wedge-budget.sh` bounds the hang INSIDE the
step and exits non-zero, an ordinary failure `continue-on-error` does tolerate;
`timeout-minutes` stays as a backstop above it for the runner-loses-comms case
(`actions/runner-images#13882`) that no in-process watchdog survives. The test
filter is passed FROM the workflow so the scope stays readable there — 243's own
promise — and `mac-split-audit` keeps grading the real invocation; that audit
caught the first attempt at exactly this and was re-proved to still bite.
Watchdog proved in isolation (137 on overrun, 0 otherwise) since the hang only
reproduces on hosted macOS. **Clause 2 end-to-end is owed to `da70df93`'s run.**

⚠️ **TO ITEM 247's OWNER — your commit `60266f9d` carries 21 lines of BOARD text
you did not write, and this paragraph is the third recurrence of that class.**
The merge-train session had this CI-RED note **staged but not committed** in the
main working tree when you branched to `claude/item-247-chevron-prototype`; a
`git checkout -b` carries staged changes across, so your commit swept it. **No
harm done and nothing was lost** — it was recovered from your commit and
re-applied here, which is why you are reading it. **Two asks:** drop the
`.orchestrator/queue.md` hunk from `60266f9d` before landing 247, or expect a
conflict against this text; and going forward, `git add` explicit paths in a
shared tree rather than `-A`/`-u`. Thank you for moving 247 onto a branch — that
resolves the collision the previous paragraph proposed, and the main tree is
free for the train again.

✅ **243's VERIFY CLAUSE 1 IS PAID, and this is the wave's headline result.**
On run `30825396088`, `mac (build + test, minus render::tests)` completed
**success** on a hosted runner. **That is the first hosted-mac arm to pass and
gate in ~140 commits.** `linux`, `web` and `mac live-probe` are green alongside
it. The tolerated `mac (render::tests)` was still running when the gating arm
finished — expected; it may burn its 90-minute ceiling, and that is by design,
not a new signal. **What this buys, starting now: real virtualised-Metal
coverage over ~95% of the suite, and a red that names its own cause from the
workflow file alone.** Clause 2 end-to-end — that the tolerated job's red does
not fail the workflow — is settled by that run's final conclusion; the YAML
shape was already proved by parse (job-level `continue-on-error`, gating job
carrying none at any level).

⚠️ **THE SAME ANTI-PATTERN APPEARED TWICE IN ONE WAVE, in different costumes —
this is the wave's most reusable finding.** A hand-kept roster standing in for a
derived one. Item **240** removed it from the shader sweep (a hand-kept list
covering 4 of 9 shaders → a directory-driven sweep). Item **238's new law then
reintroduced it**: `no_tracked_file_spells_the_old_repository_reference` walked
the filesystem behind a hand-kept `SKIP_DIRS` list that has to be held in sync
with `.gitignore` by hand. It went red on combined `main` over
`.playwright-mcp/` — gitignored browser snapshots holding pre-rename URLs no
shipped artifact has ever contained. **The asymmetry is the instructive part: CI
checks out a clean tree and would have passed, so only developers with local
debris would ever have seen it, each rediscovering it separately.** Repaired at
`3b354d3a` with `git ls-files -z`, which is definitionally the set the law's own
name and panic message claim and cannot drift. Mutation-proved both directions —
red by name at `README.md:73` on reintroduction, and green with the untracked
snapshots still present on disk, which is the actual regression. A non-vacuity
assert was added on the enumeration, because a broken listing would have
filtered every offender out and passed silently. **When a law says "tracked",
ask git.**

- **237** — ✅ **COMPLETE.** Merged `72d08422`. Receipt
  `native-gate-receipt commit=459911d86031d8a01a3c340210a43fdcac406b52 conventions=mac,linux scope=all-targets`,
  `web-smoke: OK`, code-health and fmt clean. Arm 2 **re-aimed, not deleted**:
  it now reads `geom.text_left` against the left edge of what
  `overlay_bar_rects_probe` actually emitted, so it fails on item 234's defect
  directly **and** on a decoupling of `bar_hug_span`'s left edge from
  `bar_full_span`'s — which arm 1 structurally cannot see. Arm 1 untouched; the
  item-236 scrim gate confirmed untouched too.
  ⚠️ **THE MOST INSTRUCTIVE PART: the owner's FIRST redesign was the same class
  of defect it was sent to fix.** Non-tautological, but blind to the bug it
  named — it graded the drawn plate's placement against itself, never reading
  `geom.text_left`, and **passed clean with item 234's original mutation live.**
  What exposed it was isolating arm 1 and re-running the mutation. **That is the
  generalisable technique: to test whether arm N is real, neuter the other arms
  and re-run the original defect.** A law suite can hide a dead arm behind a
  live one indefinitely otherwise.
  A real product feature nearly became a false positive: taking the minimum x
  across plates picked up the SELECTED plate, whose left edge
  `overlay_selected_bar_rects` mirrors by `grow_px` on `TopRight`/
  `mirrors_growth` worlds (Cassowary, Firetail). Restricted to grow-immune
  unselected/footer plates.
  **Brief correction worth carrying:** the phrasing "the oracle must read the
  DRAWN text" pointed at the plate, and reading the plate ALONE is itself the
  trap. The oracle has to read the text's own position.

- **244** — ✅ **COMPLETE except its live sitting.** Merged `0d0a8b1b`. Receipt
  `native-gate-receipt commit=5d17b676471646be9380eed0fb2b1209c150c6a9 conventions=mac,linux scope=all-targets`,
  `web-smoke: OK` (16 passed), **`bash scripts/code-health.sh`** clean, fmt
  clean. The field translation is **deleted outright**, both terms; the
  companion breathe reuses `stars.rs`'s envelope verbatim so the integer-cycles
  law covers the new motion and the pop cannot return through a different
  variable. The vacuous law is **deleted, not edited** — verified absent by grep
  and by a 0-test run.
  **Numbers, because this item's Verify clause is unusually falsifiable:**
  wrap pair **0 of 960000 pixels differ** across the wrap and **2413 differ
  mid-cycle**, so the field is provably still AND the probe is provably not
  vacuous. Byte-identity **19/20 PNG** (only Bowerbird differs) and **20/20
  sidecars**.
  ⚠️ **Two pieces of method worth stealing.** It mutation-proved **one arm at a
  time** — item 237's technique, applied unprompted a few hours after 237
  established it — and the replacement law **names the consumer it caught**
  (`Bowerbird (organic companion breathe): … 15 channel levels`;
  `Bombora (waves): … 33 channel levels`) rather than merely going red. And it
  **caught its own confound**: a naive same-worktree byte-identity comparison
  reported a spurious `dirty` sidecar diff, so it re-measured from a
  matched-basename worktree at the parent commit, because the gutter renders the
  project name and a different basename changes the pixels.
  ⚠️ **It also cherry-picked `f8121f45` into its branch** rather than leaving the
  red for integration — its base predated the clippy repair, so its own
  `code-health.sh` inherited main's failure. Merged without conflict.
  🔵 **OWED, and nothing above implies it:** the `--release` sitting — whether
  the pop is gone by eye, the amplitude (`ORGANIC_BREATHE_AMOUNT = 1.2`, tuned
  to a ~17-level peak channel swing after a first value proved sub-perceptible),
  and whether it reads as a flash. Both constants are marked taste-tunable in
  the shader. **The display is locked; no sitting was attempted.**

🔴🔴 **CI RED 2026-08-04, CAUSED BY THIS SESSION'S OWN BRIEFS. Repaired at
`f8121f45`, pushed. READ THIS BEFORE WRITING ANOTHER BRIEF.**

Run `30830772892` failed **both** gating arms — `mac (build + test, minus
render::tests)` and `linux` — at the step "Rust code health", on six clippy
errors from items 218 and 229: three test-only helpers reported never-used
(`stats`, `accessibility_stats` in `app/frame/accessibility.rs`,
`incremental_tree_update` in `semantic/native.rs`), three
`doc_lazy_continuation` in `card/figures/mod.rs`, and one `map_clone` in
`semantic/native/tests.rs`.

‼ **THE CAUSE IS THE WRONG GATE COMMAND, AND IT IS THE ORCHESTRATOR'S FAULT.
CI runs `bash scripts/code-health.sh`. Every brief in this wave — and every
check the orchestrator ran itself — used `python3 scripts/code-health.py`.**
The `.py` is real but **NARROWER**: it carries the structural and Clippy
ratchets and genuinely caught two defects today (a 104-column line, four
queue-item citations in comments). The `.sh` additionally runs the clippy arms,
**including the mac-only cfg arm**, which is where all six of these lived.
**Two lanes reported "code-health clean" in good faith while running the wrong
entry point, because the brief told them to.** An earlier note in this section
says the briefs "omitted code-health" — that was only half the story, and the
half that followed was worse, because naming the wrong command reads as
coverage. **THE PRE-LANDING COMMAND IS `bash scripts/code-health.sh`. Say that,
not the `.py`.**

Repair: the three helpers are genuinely test-only (verified by grep — their only
callers are `tests.rs` files), so they are `cfg(test)`-gated rather than
silenced with `allow(dead_code)`. That stranded `ProjectionStats` as an unused
import twice, so its re-export is split rather than deleted —
`SemanticProjection` still has non-test consumers and `ProjectionStats` no
longer does. Reproduced with the `.sh` before fixing and verified with it;
`native-gate-receipt commit=f8121f4503d82dd62c726a62667bde0adfa35744 conventions=mac,linux scope=all-targets`,
`web-smoke: OK`.

✅ **WORTH SAYING PLAINLY: ITEM 243's SPLIT IS WHAT CAUGHT THIS, on its second
real run.** Before the split these six errors would have landed inside a job
that was already red for item 231's wedge, and nothing would have distinguished
them from the known hang. That distinction is the entire rationale of the item,
and it paid within hours.

✅ **AND THE WEDGE REPAIR (`da70df93`) IS PROVEN AT THE JOB LEVEL:** in run
`30830772892`, `mac (render::tests)` concluded **`failure`**, not `cancelled`,
after 64m05s — the 2×1500 s budget plus build, as designed. `ci-wedge-budget.sh`
converts the hang into an ordinary failure exactly as intended. **What is still
owed is one run where the gating arms PASS and a wedge failure alone leaves the
workflow green** — that is run `30836476858` on `f8121f45`, in flight.

- **245** — ✅ **COMPLETE.** Merged `5635b5e2`. Receipt
  `native-gate-receipt commit=3763064e76aab3a10c3e93dd7b3f1bef80d026c9 conventions=mac,linux scope=all-targets`,
  `web-smoke: OK`, `bash scripts/code-health.sh` clean, fmt clean. The 5,500-
  character Japanese fixture now reads **11 min** — it was 28 after item 229 and
  a flat 1 before it.
  **The rate is LANDED, not parked: 500 characters/minute**, the round midpoint
  of the published ~400–600 cpm Japanese silent-reading range, chosen the way
  200 wpm is the round conventional English figure. 🔵 **It is one constant,
  `CJK_CHARS_PER_MINUTE`, and it is a taste-checkable number — if the user wants
  a different pace it is a one-line change, not a rebuild.**
  **The structural part is better than the number:** `markdown::reading_time_min`
  now **takes the pace as an argument** instead of hardcoding `READING_WPM`, so
  no caller can silently apply the wrong rate — the defect cannot recur through
  a second call site. The pace lives on `CountUnit::pace_per_minute`, beside the
  enum, and `readout_figures` just asks the unit; item 229/215's single seam was
  not reopened. Mixed documents take the dominant script's pace outright, so
  `dominant_unit`'s strict-majority rule decides label and pace in one edit and
  its no-flicker guarantee covers the pace for free.
  **`SCHEMA_VERSION` checked and deliberately NOT moved** (stays 198):
  `reading_min` is an existing field gaining a corrected value, not a new shape,
  which is `capture.rs`'s own stated bump criterion.

⚠️ **THE 245 LANE INDEPENDENTLY HIT BOTH OF THE ORCHESTRATOR'S OWN MISTAKES
FROM TODAY, ONE OF THEM DESPITE AN EXPLICIT WARNING. That makes them process
defects rather than individual lapses, and they belong in the brief template.**
- **It committed while its own gate ran** — "commit before any wait" taken
  literally, mid-suite — and `native-gate.sh` refused the receipt exactly as it
  refused this session's (`start=d0c1dfc3… end=3763064e`). A clean full run
  thrown away, for the second time today, by two different actors. **The rule
  has to be stated as an ORDER, not a duty: commit BEFORE launching the gate,
  never during it.** "Commit before pausing" and "do not commit during the gate"
  read as compatible right up until the gate is the thing you are pausing on.
- **It hit a variant of the self-matching `pgrep` trap the brief had warned it
  about.** It correctly used the bracket form `pgrep -f '[n]ative-gate\.sh'` —
  and still matched itself, because its own wait loop contained
  `echo "native-gate.sh finished"`, an UNBRACKETED occurrence elsewhere on the
  same command line. **So the bracket trick is not sufficient; any occurrence of
  the literal string anywhere in the watcher's command line defeats it.** It
  recovered by switching to `kill -0 <pid>` on the known launch PID, which has
  no such failure mode. **Prefer the PID.**

- **239** — ⚠️ **PARTLY REVERTED. Its FINDINGS stand; its ORACLE did not.** The oracle landed `52b1b313` and was reverted `b2f27143` after CI went red — all three `alloc_bound_law` tests failed in the **linux** job under both conventions, on the exact portability the design chose itself on. **Requeued as item 249**, which carries the full evidence and everything worth keeping. **The findings below need no re-measurement and must not be re-derived.** Original entry: **its headline is a NEGATIVE RESULT that is the
  deliverable rather than a shortfall.** Merged `52b1b313`. Receipt
  `native-gate-receipt commit=45fee36af80270fa54f1fb024a69e79fd58bc8b8 conventions=mac,linux scope=all-targets`,
  `web-smoke: OK`, `bash scripts/code-health.sh` clean, fmt clean.
  **The portable counter does NOT reproduce item 232's container split.**
  Objects per test: good `36707d06` **242.4**, bad `8207e519` **243.3** — a ratio
  of **1.0039** where the container's own was **1.244**. Wrong order of
  magnitude, not a weak signal. **So wgpu object allocation is not what the
  4 GiB container was exhausting**, and item 231's named residual suspects
  accumulate badly but do not accumulate *differently* between the two trees.
  Guarded against item 232's own error: separate worktrees per boundary, with a
  compile-time provenance stamp asserted identical across samples.
  **Mechanism correction for item 231's entry:** the suspect class is misnamed.
  It is not "reclaimed only on poll" — `Queue::write_buffer`/`write_texture`
  park an `Arc` in `PendingWrites`, drained in exactly one place, `pre_submit`.
  A caller that stages writes and never submits pins them **however often it
  polls**, and most render tests `prepare()` and return. **This matters twice: a
  fix aimed at polling would not have worked, and the live app submits every
  frame, so on this resource class the product is not exposed and the harness
  is.** Bound: an empty submit plus a non-blocking poll in `test_gpu::arrive` —
  no roster of tests, no roster of resources. Live objects went from a 160,201
  climb to `kept` median 7 / p90 231.
  Oracle counts objects, not bytes, and that is measured: `buffers`/`textures`/
  `texture_views` are maintained by metal, vulkan, gles **and** dx12, while
  `buffer_memory`/`texture_memory` are vulkan and dx12 only — a byte-valued law
  would read **flat zero on two of awl's four backends**.
  ⚠️ **The third law is the one that keeps the other two honest:** drop wgpu's
  `counters` feature and it fails **by name**, instead of every counter silently
  reading zero and both allocation laws going vacuous forever. That is the
  anti-vacuity arm this board keeps asking for, written without being asked.
  ⚠️ **Item 237's trap caught one of the author's own, in this file.** The first
  workload submitted a pass, so law 3 stayed **GREEN** under its own mutation —
  `Queue::submit` maintains the device on its way out, so the workload was doing
  the reclaiming its law existed to check. Both hazards are documented in the
  workload's doc comment.
  ⚠️ **Known limit, reported rather than hidden:** a constant bounded pool (30
  extra buffers held in a thread-local each call *replaces*) leaves all three
  laws green. These laws bound GROWTH, not absolute ceiling.

🔵 **ITEM 231 — A CHEAP, FALSIFIABLE LEAD FROM THE 239 LANE. Two ratios, and the
lane flagged it as correlation rather than dressing it as a finding.**
`background.wgsl` is **72,896** bytes at `8207e519` against **58,687** at
`36707d06` — ratio **1.2421**. The container's own test-count ratio, 199/160, is
**1.2437**. They differ by **0.13%**. Both boundaries predate `gpu_cache.rs`, so
every `TextPipeline::new` translated that whole file through naga and then the
backend compiler, and under lavapipe the result is LLVM IR and machine code held
in the llvmpipe context. **If the OOM budget is dominated by per-translation
shader-compilation memory, it would be spent in inverse proportion to shader
source size — which is what those two numbers say.**
**The falsification is ONE container run:** `gpu_cache` cut program builds 9.3×
and landed after both boundaries, so if this is right, HEAD under the same 4 GiB
ceiling should get dramatically further than 199, or not OOM at all. This does
**not** contradict 231's elimination (e) — that cut failed to clear the *hang*,
and the OOM is explicitly a different failure mode. ⚠️ **Two data points and a
coincidence-grade match. Nothing here shows that bounding allocation growth
would prevent the hosted-mac hang, and no commit, comment or law name in the
239 branch implies item 231 is fixed or explained.**

✅ **THE CI RED IS REPAIRED, CONFIRMED ON THE AXIS THAT FOUND IT.** Run
`30836476858` (`f8121f45`) came back with **`mac (build + test, minus
render::tests)` = success and `linux` = success**, plus `web` and
`mac live-probe` green. That run's own conclusion reads `cancelled`, but for a
reason that is not a verdict on the code: **this session pushed into it**, and
`ci.yml`'s concurrency group cancels in progress, so the wedge job was killed
mid-budget rather than timing out. Item 243's clause 2 therefore rides run
`30838810157` (`76903fc1`).

⚠️ **A `pgrep -f` WAIT THAT MATCHES ITSELF — new trap, cost one lane an
unbounded stall, worth a line in every future brief.** The item-239 lane armed
`until ! pgrep -f 'native-gate.sh'; do sleep …; done`. **The watcher's own shell
command line contains the string `native-gate.sh`, so `pgrep -f` matches the
watcher, the condition is permanently true, and the loop can never exit.** Two
processes matched `native-gate` on this host and both were that watcher; no gate
and no cargo were running at all. Use the bracket trick the README already uses
for `ps aux | grep "[c]argo"` — `pgrep -f '[n]ative-gate\.sh'` — or `pgrep -x
cargo`, or just watch the log stop growing. **And the deeper point this proves
concretely: a wait that never terminates is indistinguishable from a wait that
terminates and is never noticed, because nothing wakes a worker but the
orchestrator. Neither is a wake-up source.**

⚠️ **TWO ORCHESTRATOR MISTAKES ON 2026-08-04, BOTH ALREADY WRITTEN DOWN
SOMEWHERE AND BOTH MADE ANYWAY. Recorded so the next session does better than
re-read them.**

**1. Committed a board note WHILE the merge-train gate ran, and threw away a
full native run.** `native-gate.sh` refused correctly:
`native-gate: HEAD changed while the suite ran (start=0d0a8b1b… end=76903fc1…);
no receipt issued`. Every test had passed, both conventions, zero failures —
and none of it counted. This is the README's own §Gates rule, and the identical
incident it already records from 2026-07-31. **The failure mode is sequencing:
folding a landing note in right after a merge is correct BETWEEN gates and wrong
DURING one.** Re-run gave
`native-gate-receipt commit=76903fc1dcd13a1755eb55677bc504b554e1c87d conventions=mac,linux scope=all-targets`.
The gate catching its own invalidation is the only reason the loss was visible
rather than a receipt naming a tree that had moved underneath it.

**2. Pushed while a CI run was still in flight, cancelling the exact
verification that was owed.** `ci.yml`'s concurrency group is
`cancel-in-progress: true`, so pushing `76903fc1` killed run `30836476858`
mid-flight — the run that was going to pay item 243's clause 2. Nothing is lost
permanently (the next run does the same job) but ~65 minutes are. ⚠️ **THIS IS
NOW STRUCTURALLY WORSE THAN IT USED TO BE AND THE BOARD SHOULD SAY SO: since
`da70df93`, a full CI cycle is ~65 MINUTES**, because the tolerated wedge burns
its 2×1500 s budget before failing. **So the README's "let one run finish before
pushing the next" is no longer a politeness — it is a 65-minute window in which
any push destroys the evidence.** Batch the train, or accept that clause-2-style
end-to-end CI evidence needs a quiet window nobody pushes into.

⚠️ **A BRIEF DEFECT THIS WAVE PAID FOR — fix it in the next brief template.**
Items 240 and 243 were each green alone and **red together**: 240's new file
carried a 104-column line and was not `rustfmt`-clean, and combined `main`
failed `code-health` the moment 243's merge landed on top of it. Neither lane
was at fault — **the orchestrator's briefs named `native-gate.sh` and
`web-smoke.sh` but never `code-health`**, so a lane could run everything it was
asked and still land a health failure. The README already says the pre-landing
set is *code health, native gate, wasm smoke*; the briefs dropped the first one.
Repaired in the same commit as this note (`cargo fmt` plus a wrapped panic
line); the sweep still passes and the health ratchets are clean. **This is also
the README's "two branches each green alone can be red together" warning
arriving in its most boring possible form — a line length.**

⚠️⚠️ **TWO ORCHESTRATORS ARE LIVE RIGHT NOW AND ONE IS WRITING THE MAIN WORKING
TREE. Read this before your next edit.** Item 247's claim says it works **in the
main working tree, not a worktree**. The other session (this one) uses that same
tree as the **merge train**: it merges lane branches there, runs
`scripts/native-gate.sh` there, and pushes from there. Those two uses are not
compatible without a rule, because `native-gate.sh` refuses a receipt if HEAD
moves under it, and a gate run against a dirty tree silently certifies the other
session's uncommitted edits.

**The rule, proposed by the merge-train session and adopted unless 247's owner
objects on this board: 247 keeps the main tree for SOURCE edits; the merge train
moves its gate to `awl-next-worktrees/train-gate`.** Until that move lands, the
train session will check `git status --short` immediately before every gate and
abort rather than gate a tree it does not own. It has already happened once
harmlessly: `41cc4bc0` (247's own claim, board-only) landed between this
session's gate at `c282cedd` and its push, so the pushed tree differs from the
gated one by `.orchestrator/queue.md` alone — **zero `.rs`, so the receipt's
native scope carries.** Recorded rather than glossed, because the next such
overlap may not be board-only.

**Evidence the board discipline is holding so far:** `23a47790` (item 244) and
`ef17eeab` (247/248) both landed on `main` between this session's commits and
both survived intact, because every board write here has been a targeted edit
rather than a wholesale rewrite. Keep doing that. Rule 5 exists for exactly this
hour.

**Two housekeeping facts for whoever integrates.**
`awl-next-worktrees/item-232-scratch` is a leftover directory that
`git worktree list` does not know about — inspect before deleting, it was not
touched by this wave. And the RECEIPT GAP below is still open.

**Overnight results, newest first.**

⚠️ **RECEIPT GAP — items 232, 235 and 236, whose completion entries have been cleared as history.**
None of the three merge commits (`5bc771ca`, `df630ad9`, `ef6f87ca`) records a
`native-gate-receipt` string, so by this repo's own rule — *the receipt is the
only authorization to call the native tier "full native suite"* — **their native
scope is UNVERIFIED.** A gate demonstrably ran for 235 (it is what caught
`gpu_cache_law` at "found 9, expected 8", which a targeted run structurally
could not reach), and 236 paid a health debt only a health run surfaces, so this
is very likely a recording failure rather than a skipped gate. **Re-issue the
three receipts on the merged tree, or leave them recorded as unverified.** Do
not retro-fit a receipt string from memory.

- **231** — 🔴 OPEN, and **REFRAMED to a diagnosis item** by user decision
  2026-08-03: name the cause first, then decide who owns the fix. The full
  evidence — what is eliminated, what is still unknown, the local-repro plan and
  the carry-forward traps — now lives in **item 231's own entry**, which is
  authoritative. The `src/gpu_cache.rs` round landed (`52,083 → 5,577` program
  builds, 9.3×) and **did not clear the hang** (run `30770296246`); its receipt is
  `native-gate-receipt commit=3e3db0c6… conventions=mac,linux scope=all-targets`.
- **174** — 🟢 SECOND FAMILY LANDED, item remains OPEN. Merged to `main`;
  worktree removed. `PlannedHeader` owns the overlay header band, with the query
  beat folded into the LAST header line's box exactly as the shaper folds it into
  that line's glyph metrics. **Deleted from `render/chrome`, not banned by law:**
  `overlay_secondary_top`, `overlay_split_bounds`, `overlay_strip_band`,
  `overlay_query_center` — the previous slice's standard, met again. ⚠️ **It
  found a shipping pointer defect of exactly the predicted class:** `over_overlay_query`
  tested the bare row pitch, but on the FLAT family the beat inflates the query
  line itself, so at the shipping default the field draws `[64.0, 133.2]` while
  the pointer band ended at `91.2` — **the I-beam sat in empty air ABOVE the
  query text and the text itself took the plain arrow.** 13 of 19 `OverlayKind`s,
  Settings-as-workspace included; worst case 33.0px at 2×. The GROUPED family was
  right *by accident* (its beat inflates the lens strip instead), which is how a
  parallel calculation survives review — it agrees on the arm somebody looked at.
  Identity 840/840 PNG and 840/840 sidecar, zero differing; the cursor icon is
  the only changed output. **Left explicitly:** `workspace_header_beat` did not
  merge — its consumer is reached ~45× a frame through the four relocated
  document owners, so planning inside it would trade one parallel calculation for
  45 plans a frame; a law fails by name if they drift. Follow-up: item **217**.
- **116d** — 🟢 COMPOSITING ROUND LANDED; the flip is deliberately NOT done.
  Merged to `main`; worktree removed. **The owner stopped at a clean boundary
  and that is the correct outcome** — the comparison can now be SEEN, and
  `workspace_shape(History)` is still `None`, which is now safe to change.
  `draw_document_layers` splits into `draw_document_ground` (background, lava,
  stars, page frame — the quiet frame, never relocated) and
  `draw_document_content`; on a comparison frame the content is submitted AFTER
  `draw_overlay_card` into the carved region without re-drawing its ground, and
  the ordinary frame concatenates the two in their original order so **every
  non-comparison frame is byte-identical by construction**. The blur path now
  captures the ground alone while a comparison is up (116b's frosted-ghost
  defect), and `blur_signature` hashes the comparison flag — otherwise Settings'
  workspace and History's sign identically at one scroll and keep the wrong
  frost. `clip_text_bounds` is the glyph twin of item 84's quad clip. The old
  boundary law is deleted as its own message asked and replaced by four
  containment-and-visibility laws. **The Esc decision landed for BOTH members**;
  `Shift-Tab` was not wired at all and had to be added, and `GUIDE.md` plus
  `site/guide.html` both promised "Esc there is a *back* to the rail" and were
  fixed. Captures 1320 files: 1080 byte-identical, 240 differing — all the
  `settings-detail` probes, one sidecar field, `esc back` → `tab back`.
  ⚠️ **The vision smoke found a defect the whole green suite missed:** the
  five-cell workspace footer ran off the card on Firetail at 900×520. The
  pre-existing no-clip law measured the FLAT card at one canvas over three
  worlds and was structurally blind to the workspace; the missing law is now
  written over the whole roster at four canvases and both stages.
  **Premise corrections worth keeping:** 116c's `⇧↵`/`open_keep_version` is the
  *restore* and the *keep* prompt, NOT groundwork for the Back affordance — the
  orchestrator's brief said otherwise and was wrong; and History's Back was
  already half-present (`foot_hint`'s `detail_focus` branch said `tab back`) —
  the debt was **Settings'**, whose line said `esc back`, the reverse of how the
  brief framed it. `workspace_header_beat` was deliberately left unfolded: it is
  a fourth copy of a ONE-LINE header today, and the moment the lens moves to the
  header `header_rows` becomes 2 and that copy becomes **wrong** rather than
  merely duplicated — so folding it belongs inside the lens-to-header slice.
  **The restore notice is CALLED but not implemented:** restore emits one calm
  notice naming the version and the undo (`restored "2 hr ago" · ⌘Z to undo`);
  Esc emits none, because it undoes a view substitution and a toast confirming a
  no-op is the nagging DESIGN's calm bias forbids. It belongs with the restore
  journey the flip owns.
  **Left for the next slice:** flip History to `TimelineOverComparison`; move the
  lens to the header (inside `PlannedHeader`, folding `workspace_header_beat` in
  as part of it); reuse the ordinary candidate-row hit-test for the timeline;
  deep-link `Version history…` and `Compare with version…`; implement the restore
  notice; and the tier-2 hermetic work. ⚠️ **One harness fact for that slice:**
  `--keys` captures reach History's timeline but **not** its comparison — a
  headless capture has no history store, so `selected_history_id()` is `None`
  and the focus transfer declines. The comparison's capture-tier probes need
  `--screenshot-app` or a seeded store.

## Remaining work — handoff order (RE-DERIVED 2026-08-03 afternoon)

⚠️ **The 2026-08-02 list below this one is STALE and is retained only as
history — do not dispatch from it.** Verified against the tree: its #1 (116d)
is landed, its #2 (204) is landed **both slices**, its #3's named remainder
(`workspace_header_beat`) is already folded and gone from `src/`, and of its #4
the 217 and 215 members are landed. Their `✅ COMPLETE` entries were cleared
into history by `16b4e8c2`, which is sanctioned; what was not sanctioned is
that the handoff list itself was never re-derived afterward, so it kept
pointing at finished work. **A compression that clears completions must
re-derive this section in the same commit.**

**Live order, after this afternoon's wave:**

1. **218 — dispatched this wave.** The VoiceOver stall is user-reported and is
   the only open item that degrades a shipping accessibility path.
2. **243, 240, 238 — dispatched this wave.** 243 first among them at
   integration: it is what ends the uninformative `main` red.
3. **174 — RE-SCOPED 2026-08-04 by survey, not by the stale text. Open, not
   blocked.** Its named remainder `workspace_header_beat` is **gone** — folded
   into `plan::header_band_height` by the 116d flip slice — so the 2026-08-02
   handoff pointed at work that no longer exists.
   **What the planner actually owns today** (`src/render/plan/`, five modules):
   `overlay_header` (`PlannedHeader`, `beat_stands_alone`, `header_band_height`),
   `overlay_rows` (+`plan_witness`), `overlay_row_plan`, and `row_extent`
   (`RowExtent`, `ClusterExtent`, `RowSpan`). That is **two families migrated —
   the overlay row family and the header band** — against the item's goal of
   every surface.
   **What still owns its own geometry, from a grep for hand-computed bounds in
   `render/chrome/`:** `mod.rs`, `gutter.rs`, `overlay_selection.rs`,
   `preview.rs`, `overlay.rs`, `workspace.rs`. That list is the candidate set
   for the next slice — **it is a survey, not a plan.** ⚠️ **Pick the next
   family by measurement rather than by that ordering**, and pick it against a
   live check of `render/chrome/`, because this item's remainder has now gone
   stale twice.
   ⚠️ **Scheduling constraint, live right now:** `overlay_selection.rs` and
   `diagonal.rs` are item **247**'s working set (branch
   `claude/item-247-chevron-prototype`). Do not dispatch a 174 slice over those
   two until 247 lands. `gutter.rs` and `preview.rs` are clear of it.
4. **Then, unblocked and unclaimed:** 222/131d (Magpie's mirrored cluster, user
   decision made), 221 and 224 (both were blocked on 235's capability, which
   landed), 241, 242, and **249** (unblocked 2026-08-04 — see the evidence-branch
   decision at the foot of this board; **250** retires the exception it needed).
   ⚠️ **Corrected 2026-08-04:** this line previously also named 237, 229 and 239.
   **237 and 229 are COMPLETE** (`72d08422`, `d8ae72c9`) and **239's oracle was
   REVERTED and requeued as 249** — dispatching from the old text would have
   re-done finished work, which is the exact defect the header of this section
   complains about, recurring one wave later.
5. **Human/live closures, all needing an unlocked and FOREGROUNDED display:**
   118's world-loudness confirmation and its `--release` ambient sitting; 211's
   unoccluded confirmation and its unreached sweep arms; **207's real VoiceOver
   journey** — its **AT-SPI half is item 251 and needs a LINUX machine, not an
   unlock**; and now **218's own final Done clause**, which no test tier
   can stand in for. ⚠️ `displaysleep` is 10 and screensaver `idleTime` is 300 —
   this silently invalidated the 2026-08-02 sitting seven minutes in. Hold the
   display with `caffeinate -d -i -t <seconds>` and re-check the lock at BOTH
   ends; `live-probe.sh` only checks in preflight.

**Item 116's parent entry should now be closed outright** — it is kept verbatim
in a pre-flip state that is no longer true. Left open deliberately rather than
edited mid-wave, because it is a long entry and rewriting it while three lanes
run risks exactly the drop this section is about.

## Remaining work — STALE, 2026-08-02, retained as history only

1. **116d — dispatch first; it is UNBLOCKED and everything behind it waits.**
   116a–c are landed and both of its owed decisions are now made: the comparison
   sits ON the workspace surface, and one Esc always leaves. Do the compositing
   round FIRST — delete `the_relocated_document_is_geometrically_placed_but_not_yet_composited`
   and replace it with the containment-and-visibility law its own message asks
   for — then flip `workspace_shape(History)` to `TimelineOverComparison`, add
   the footer Back affordance, move the lens to the header, deep-link
   `Version history…` / `Compare with version…`, and run the split Verify sweep
   (tier 1 replays `overlay_accept:History`; tier 2 owns the store, git, the
   pruned ladder, renamed timelines, `KeepVersion` and the restore's disk read).
   116a's handoff still applies: reuse the ordinary candidate-row hit-test for
   the timeline rather than extending the rail functions — `geom.rail` is `None`
   whenever rows are primary. ⚠️ **It writes `render/chrome/overlay_draw.rs`,
   `overlay_rows.rs` and `chrome/mod.rs`, which item 174's next family also
   writes — do not run the two concurrently.**
2. **Then 204.** Unblocked the moment 116d's composited general read-only payload
   exists. Preserve the one editable buffer; add disk fingerprinting (mtime plus
   length cannot detect the required same-time/same-size rewrite), the recovery
   record, three read-only conflict views, gated-action resolution, and align
   Guide/welcome/site prose.
3. **174's next family**, whenever it is not racing 116d. `workspace_header_beat`
   is the named remainder but is its own slice — its consumer is reached ~45× a
   frame through the four relocated document owners.
4. **Follow-ups queued by this wave:** item **218** first (the newly live
   VoiceOver path can stall while typing), then **217** (`--bench-suite`'s
   plan-count witness vs the diagonal re-plan; not urgent, and must not be fixed
   by weakening the witness) and **215** (extract word count / language / percent
   into pure owners so a live-App capture carries card semantics).
5. **Human/live closures, all needing an unlocked and FOREGROUNDED display:**
   118's world-loudness confirmation and its `--release` ambient sitting; 211's
   one unoccluded confirmation that the fixed build presents the glide, plus its
   unreached sweep arms; and item 207's real VoiceOver / AT-SPI journeys, which
   no test tier can stand in for. ⚠️ **The machine's idle lock fired seven
   minutes into the 2026-08-02 sitting and silently invalidated it** — disable
   the idle lock before the next one, and re-check the lock at BOTH ends of the
   run, because `live-probe.sh` only checks it in preflight.

Items 131, 172, 207 and 213 are complete; 174 has two families landed and stays
open. Older historical prose below is retained but the receipts above are
authoritative.

After each landed item: update this board, exact combined-main code-health +
web + native receipt, push `main`, remove only the completed worktree, and run
`scripts/sweep.sh 1`. Tags/releases and site deployment still require explicit
user authorization.

116. **Move Version History into the shared summoned workspace — timeline and prose diff become one readable task, never three competing layers.** **Build:** Preserve the existing history store, git backend, pruning, facets, descriptions, kept versions, and prose-diff engine, but replace the current History overlay/diff-as-preview composition with item 114’s workspace. On wide windows, show a narrow timeline/navigation pane beside a large read-only comparison pane; moving through versions updates the comparison immediately. On narrow windows, show the timeline first and enter the comparison as a second stage with an explicit return path. The current editor is backdrop/state, not a third readable layer. **Scope:** Keep local loose-file snapshots and git-managed history behind the same UI with only a quiet source label. Preserve independent diff scrolling and a clear focus transfer between timeline and comparison. `Esc` leaves the current buffer byte-for-byte unchanged; restore must be a deliberate, footer-taught action rather than bare `Enter`, and remains undoable. `Version history…` and `Compare with version…` deep-link into the same workspace at the appropriate focus; `Keep version…` retains its brief naming prompt and returns coherently. Remove the old History overlay, document-under-card preview composition, and feature-specific diff-panel dressing only when their last consumer is gone—retain the generic prose-diff machinery and do not strand parallel disabled paths. **Done:** A user can answer “which version, what changed, and do I want it back?” without overlapping titles, hidden prose, or ambiguity about whether the editor is active; the same flow works for local and git history; exiting is a true no-op and restoring is deliberate and undoable. **Verify:** Timeline→live comparison→focus/scroll→back/exit/restore journeys for local snapshots, named/pinned versions, git commits, empty history, renamed files, and pruned ladders; narrow/wide/zoom/DPI captures across representative light/dark, Pane/Bars, and Wagtail worlds; pixel laws proving timeline and comparison never overlap and the original document does not remain a competing readable layer; restore undo law; capture/replay parity; dashboard vision smoke; native, both conventions, and wasm gates. **Depends on the completed item 114 Settings workspace (landed `60477e7c`); user design decision 2026-07-26.** ⚠️ **DECOMPOSED 2026-07-31 after inventory — the owner stopped at a clean boundary rather than half-land it, which is the correct outcome and the brief's stated escape hatch.** It began the content-model change, then reverted deliberately: the first edit flips History's shell predicate on, and committing that without the content is precisely the empty workspace item 114 forbids. **This is not "big like 114" — it is four independent 114-sized changes, three of which were invisible from the item text.** **(1) The comparison has no renderer.** awl has exactly one prose renderer — the document layer. The transcript is markdown from `prosediff::render_markdown_blocks`, so relocating it into the content pane means giving `column_left()`, `column_width()`, `doc_top()` and `doc_clip_band()` — the four owners every document consumer routes through, ~45 call sites across `rects.rs`, `layers.rs`, `text.rs`, `geometry.rs`, `scroll.rs` — a viewport override, then gating every margin-orientation surface composed off them. Item 114 added a third geometry family beside two others and never touched the document layer; this is a second structural change of the same size in the most load-bearing geometry in the tree. A second prose renderer inside the overlay pane is the "infrastructure complexity is a smell" CLAUDE.md forbids. **(2) The removal is wider than the build,** and the item's hedge resolves toward caution: the diff-panel dressing's last consumer really is History (`scripts/review.sh` sets `opts.diff` but never `opts.preview_text`, the only thing lighting `vstate.diff_panel`), so `diff_panel`, `diff_panel_rect`, `prepare_diff_panel` and three pipelines all go — but item 84's `doc_clip_band` must **survive and be re-owned** by the comparison viewport, and its law files re-aimed rather than deleted. **(3) ~60 History test functions across ~25 files assert the CARD presentation** — each a judgment call, not a rename. **(4) Restore needs a new input primitive:** `CompareVersion`/`KeepVersion` have no default chord, and "deliberate, footer-taught, not bare Enter" cannot be a named chord because `HintAction.glyph` is `&'static str` while a chord glyph is convention-dependent — so the footer-honest option is a shift-held accept delegating to `Newline` in the editor. **Two premise corrections.** `workspace_shell()` as a **bool is insufficient**: 114's shell puts facet labels in the rail and rows in the pane, but History wants the timeline as the primary list, so flipping the bool yields the wrong composition. It must become a shape — and **DESIGN.md §5 already sanctions exactly this** ("categories beside controls, or a timeline beside a comparison"), so it is a reading 114 deferred, not an invention. Separately, **two "Done" clauses are already true and cost nothing**: Esc leaves the buffer byte-identical (the transcript is a view substitution, never a buffer write) and restore is already undoable (`App::restore_history` goes through one atomic `Buffer::set_text`). Neither emits a notice, which is worth deciding — a silent document replacement is the one place a toast earns its keep. **THE DECOMPOSITION, in dependency order.** **116a — the shape:** `workspace_shell()` becomes `workspace_shape() -> Option<WorkspaceShape>` (`RailOverRows` | `TimelineOverComparison`) with `rows_are_primary()` as the single fact geometry/keyboard/hints reduce to; History still returns `None`, so nothing is presented. Tier 1, fully capturable, lands green and changes no pixel. **116b — the relocated document viewport:** `comparison_viewport` as the one owner, read by the four geometry owners; margin surfaces gated; `diff_panel` and its pipelines structurally removed; the clip/wash/panel laws re-aimed. **This is the risky half and deserves to fail alone.** ✅ **116c LANDED — merge `7ea5cd78`** (`f3da0d07`, `1254a7b9`). `Action::AcceptAlternate` (Shift+Enter) resolved directly in `KeymapState::resolve_named`, needing no catalog chord because **Shift reads identically on both conventions** — proved by a Mac×Linux × native×emacs sweep rather than asserted, plus a law confirming it is absent from the Linux keep-list. **The delegation is the literal same code path, not a copy:** `apply_buffer_action`'s arm is now `Action::Newline | Action::AcceptAlternate`, and byte-identity is proved over `Buffer::disk_bytes()` across **ten** smart-newline shapes — bullet/numbered/task continuation, the empty-item provenance flag across four mixed step-orders, blockquote continue/end, bare-indent carry, non-markdown bypass, active-selection override — not the plain-prose case anyone would have thought to check. `history_intercept` folded into `workspace_intercept`, routed through 116a's `rows_are_primary()` rather than a kind branch; `overlay_nav.rs`'s mark **lowered** to 768 as it went. `KeepVersion` now descends through `overlay::Journey` rather than entering over a card. **Honest about reachability:** the descend branch is not reachable through today's live dispatch (the palette closes itself first), so it was proved by a direct unit test "rather than a fictional re-dispatch". ⚠️ **Merge-train note:** the lane reported "`code-health.py`: clean" — the python arm alone, not `code-health.sh` with its clippy pass — and the candidate failed `clippy::type_complexity` on its new fixture tuple. Third lane this run whose branch-level health claim did not survive the train, and the same defect class the run has been about: a check whose stated scope exceeds what it ran. Fixed with a `type Fixture` alias. **116d inherits:** the intercept is ready for `TimelineOverComparison`, `⇧↵` and `open_keep_version` are ready for a real in-workspace hint, and `workspace_shape(History)` is still `None`, waiting on 116b's compositing question. an alternate-accept action delegating to `Newline` in the editor with a byte-identity law, plus shape-aware intercept. **116d — the flip and the journeys:** History becomes `TimelineOverComparison`, deep links, the lens moved to the header, and the full Verify sweep. **Verification split, written against `docs/harness-reach.md`:** tier 1 covers entry, focus transfer, Back, exit, parked-parent position, timeline selection, lens cycling, staging, zoom/DPI and every pixel law — `overlay_accept:History` is Applied, so the restore journey replays. Tier 2 covers anything touching the store or git: snapshot recording, the pruned ladder, renamed-file timelines, `KeepVersion` (Unsupported) and the restore's disk read. **The item's Verify clause reads as though the whole thing were capturable; it is not, and asking for a sidecar over `KeepVersion` would repeat item 180's mistake.** **Owed a human, and it compounds item 114's open question:** from the comparison the first `Esc` is a Back to the timeline, so leaving History from the content region takes two — the same interaction decision 114 flagged, and it should be settled once for both members before 116d lands. ✅ **116a LANDED — merge `6202205c`** (`bff81da9`): `workspace_shape() -> Option<WorkspaceShape>` with `rows_are_primary()` as the one fact, `TimelineOverComparison` defined but routed nowhere, Settings byte-identical (identical PNG sha256; the only sidecar delta was `project.dirty` from the stash procedure). Its mutation broke **three pre-existing item-114 laws**, proving the re-route is real rather than inert, and a grep-law bans matching the shape enum outside its defining file. **Handoff worth keeping:** the geometry seam is already in place, so 116b's comparison viewport just reads the content rect; 116d's timeline hit-test should reuse the ordinary candidate-row hit-test rather than extend the rail functions, since `geom.rail` is `None` whenever rows are primary; and **`chrome/workspace.rs` is at 497/500 with no mark escape** — it postdates the frozen baseline, so 116b must extract a submodule before adding to it. ✅ **116b LANDED — merge `350aed68`** (`80527ae0`). `TextPipeline::comparison_viewport()` is the one owner; `column_left`/`column_width`/`doc_top`/`doc_clip_band` read it and everything downstream follows without knowing. Extracting `workspace_regions()` first took `chrome/workspace.rs` from 497 to **479**, off the ceiling 116a warned about. The bypass is named and enumerated in `render/geometry/page.rs` with a law pinning its consumers to that file plus exactly two fallback arms. **98 captures byte-identical**, PNG and sidecar, verified three times. `diff_panel` and its three pipelines removed after the owner verified the last-consumer reading itself; item 84's `doc_clip_band` **survived and was re-owned**, its laws re-aimed rather than deleted, and its X arm is genuinely exercised for the first time. **Mutation-proofing found two fixtures that would have gone quiet instead of red** — one searched for its straddling canvas by asking the very clip under test whether it had trimmed anything, the other graded a band its transcript never inked; both now derive independently with non-vacuity floors. **No mark raised; seven lowered.** ⚠️ **The boundary it stopped at, pinned as a law rather than absorbed:** the relocation moved the document's geometry but **not its place in painter's order**, so the workspace card still draws over it — opaque hides it, translucent ghosts it, and a blur-eligible world frosts the document into the frame around the region. `the_relocated_document_is_geometrically_placed_but_not_yet_composited` asserts both halves over the whole roster and its own message tells 116d to delete and replace it. **116d CANNOT flip `workspace_shape(History)` before that compositing round — it would present an invisible comparison.** The open design question is whether the comparison sits *on* the workspace surface or is a window *through* it.

131. **Give Mangrove and Magpie mirrored diagonal-line compositions across contextual menus and the real Settings workspace.** **Build:** Add one reusable theme-owned diagonal row composition through the shared rowlayout/surface machinery, then assign its two authored orientations: **Mangrove** draws a continuous descending `\` spine, with row clusters left-aligned on the RIGHT side; **Magpie** draws a continuous ascending `/` spine, with row clusters right-aligned on the LEFT side. The line is mandatory in both—the striking read comes from the drawn division and triangular negative space, not merely staggered text. It may visually bleed toward the surface corners, while row attachment points occupy an inset middle band so the first/last rows retain usable width. **Line treatment:** never amber/primary. Mangrove uses a crisp tidal-teal line derived from its muted ink; Magpie uses a crisp graphite line from its muted ink. Resting weight is clearly visible but subordinate to text; the selected row brightens and thickens only the local spine segment toward `base_content`, extends a short connector to the row, and steps the row outward by a few crisp pixels—no spring, pulse, or full-width selection bar. Existing bottom-left Mangrove stipple and bottom-right Magpie ghost placards occupy the opposite empty triangle rather than colliding with the rows. **Controls are in scope, not a fallback:** model each row as a measured `label + fixed gap + accessory/control` cluster. Reserve consistent label/accessory extents across the visible set so shortcuts, values, toggles, checkmarks, exact-entry fields, and Range sliders trace a stable parallel rail with honest spacing; anchor the whole cluster to the spine instead of independently nudging text. Query/title/category navigation/footer regions remain horizontal and stable—the diagonal owns the candidate/setting rows, not every glyph on the surface. Filtering and scrolling sample a fixed surface-relative line at fixed row y positions, so content changes never make the spine or surviving rows jump horizontally. **Surface reach:** enroll every contextual overlay’s row section that currently consumes theme list-style data, not Commands alone; non-row panels keep their existing geometry. After item 115 removes the old Settings overlay, apply the SAME diagonal owner to the Settings workspace’s main setting list for Mangrove/Magpie, while its category rail, search/title shell, child Theme/Caret auditions, and narrow-stage navigation remain item 115’s workspace behavior. The empty main-pane triangle may carry the existing active category/title typography, but never a duplicate decorative label. **Responsive bound:** size the slope from the real widest visible cluster and available side territory; widen within the existing surface limit first, then reduce horizontal travel while preserving a visibly diagonal direction. Never overlap, clip, shrink type/controls, introduce horizontal scrolling, or silently fall back to Pane/Bars. **Scope:** This is a third data-driven row composition, not Mangrove/Magpie branches and not permission to diagonalize the document, workspace shell, History comparison, native Mac menu, or web/Linux persistent menu bar. Other worlds and their Pane/Bars results remain byte-identical. Item 112 owns the shared overlay rhythm first; items 114/115 own the workspace and Settings migration first. **Done:** Mangrove reads like a tidal instrument panel and Magpie like an editorial spread in both brief menus and sustained Settings, with a visible line, deliberate negative space, readable controls, obvious selection, and no loss of keyboard/pointer usability at any supported width. **Verify:** Full no-wildcard `OverlayKind` row-surface sweep plus every `SettingId × SettingKind`; simple/long labels, chords, values, toggles, text entry, sliders, empty/short/full/filtered/scrolled lists, category changes, child-picker return, and narrow/wide staging; drawn line/row/control ↔ hit-test agreement at zoom and 1×/2× DPI; pixel laws for orientation, line continuity, inset attachment band, fixed label-control gap, local selected segment, placard/row non-overlap, non-primary ink, and no clipping; exact before/after identity for every non-assigned world; dashboard captures and affordance-locating vision smoke over Commands plus every Settings category in both worlds; native, both conventions, and wasm gates. **Depends on 112 (landed, `fa64a3a4`) + 114 (pending) — **there is no item 115**; it was folded into 114 by `d726c4bb`, so the references to "item 115" in this item's body mean 114. The one real blocker is 114, which LANDED `60477e7c`. Ambitious user design decision 2026-07-27.** ⚠️ **DECOMPOSED 2026-08-01 into 131a–e after its seam turned out to be missing — but the seam fix itself LANDED and closed a live defect.** ✅ **SEAM LANDED — merge `dbf33714`** (`6df426a5`). **The defect it found, which was already shipping:** the renderer could stagger overlay rows — the offset lived in the draw emitters — but `OverlayRowPlan::row_at` kept testing the card's **undisplaced** x-span. A staggered row was clickable across a strip where nothing was drawn, and the deeper the row the wider the lie. Draw and pointer answered "where is this row" separately, which DESIGN.md §8 forbids, and every law item 131 wants to write is a claim about where a row *is*. `PlannedRow::dx` is now the one owner, planned in `plan_overlay_rows` exactly as `top` is planned from `lh`; the text area, bar plate, Pane band, selected bar and `row_at` all read it. `dx == 0.0` for every shipping world, proved byte-identical across **19 worlds × 5 surfaces = 95 captures**, hashing PNG *and* sidecar, 95/95. A source law bans any third file from re-deriving row x. **Four premise corrections.** (a) The mirror needs a **signed two-sided extent** — Mangrove's `\` steps rows right (left edge moves), Magpie's `/` with right-aligned clusters steps left (right edge moves); one `dx` cannot express both, and generalising speculatively would have been the dormant path the item forbids. (b) **The spine has no primitive** — every overlay quad pipeline is axis-aligned — but `caret.wgsl` already carries a rotation axis, so the cheap route is an inert `axis` on the selection instance, not a new pipeline class. (c) **Neither world authors its placard corner**; both are `PlacardCorner::Auto`, derived from their anchors, so the "opposite empty triangle" requirement needs the derivation to learn about the spine — a change the item does not budget for. (d) **Item 186's registry is keyed on `Background` variants and a row composition is chrome, not a ground**, so there is no slot — and the real finding is that **overlay chrome already mixes both spaces**: row pitch scales with DPI while `BAR_SIDE_INSET`, the text hpad and `CARD_MAX_W` are raw device px. A diagonal pitch authored like its neighbours would be **physical by inheritance**, exactly what item 186 exists to stop. Making it logical would make it the first chrome quantity to declare its space, which either extends `ground_space` past `Background` or opens a sibling registry — **a design decision owed a human eye, not a line of code.** **Good news it found:** the Settings workspace comes free — `workspace_geometry` builds an ordinary `OverlayGeom` and the row planner reads its band, so one owner reaches contextual menus and the workspace through one path. `workspace_shape`/`workspace_geometry` were not touched, so item 116a's lane stayed clear. **Decomposition:** **131a** the two-sided span (`dx` → `[left, right]`, small now the seam exists); **131b** the spine primitive (inert `axis` on `SelectionPipeline` + a rotated rounded-rect emitter, byte-identical for all 15 existing consumers); **131c** the composition owner (`ListStyle::Diagonal`, both worlds in one commit since the item forbids a half-applied world, **including the logical/physical decision in (d)**); **131d** the measured cluster rail (where the `SettingId × SettingKind` sweep bites); **131e** selection and the full Verify clause. 131a and 131b are small enough to pair; 131c–e should not be attempted in one pass. ✅ **131a+131b LANDED — merge `dbf33714`..`f4340960`** (`307308d2`, `31d9a7b4`). `PlannedRow` gained `dw` (a width delta) beside `dx`, chosen over `[left_inset, right_inset]` because every consumer already manipulates `(x, width)` pairs; one signed `dx_per_row` still drives both mirrors, split by sign in the planner alone. `SelInstance`/`selection.wgsl` gained an inert `axis` mirroring `caret.wgsl`, plus `prepare_rotated` and the pure `spine_segment`/`narrowed_spine_corner_px` helpers — **no non-test consumer**, `src/theme/` zero diff, no world touched. Byte-identity proved across **190 files** (95 captures × PNG + sidecar). **A pre-existing defect on the SHIPPED DEFAULT path was found and half-fixed:** `overlay_pane_selection`'s living-band branch (`Choreo::Morph`, the default — not an opt-in probe) called `living_band_rects` and drew the result verbatim, **never reading any row's offset**; only the sibling non-living branch read `dx`. A real drawn/hit-test disagreement, invisible only because nothing had ever planned a nonzero offset. The single-shape case is fixed; **`Choreo::TwoShape`'s echo band can represent a different row mid-glide, and whose offset it inherits is a composition question left explicitly to 131e.** The owner also flagged, unprompted, that its rotation law is single-function parametric rather than cross-owner — "flagging that distinction rather than overstating it". Merge-train note: the lane's branch passed health carrying unformatted code that happened to fit `selection.rs`'s 768 mark; rustfmt on the candidate tripped it at 771, and the spine helpers were **extracted** to `selection/spine.rs` (mark down to 731) rather than raised. 131c is BLOCKED on the user's chrome pixel-space decision in finding (d). 131d–e unclaimed.** ✅ **THE (d) BLOCKER IS DISCHARGED BY ITEM 242, NOT BY A DECISION HERE — 2026-08-03.** The user chose the toolkit answer: author chrome in logical units and multiply once at the boundary, rather than open a registry. Item 242 owns it and **sequences before 131c**; once chrome has a default, the diagonal declares nothing special and 131c is ordinary work. 242's measurement also **corrects finding (d) on one point**: `CARD_MAX_W` is not raw device px but a grow-only hybrid (`scale.max(1.0)` in `overlay_desired_w`), and `overlay_lh` — the quantity (d) called logical — is itself a dpi-scaled term plus two raw ones. **222/131d's parked label-alignment taste call is unaffected and still owed.**

172. **Decompose the 107-field `App` into owned state domains with narrow transition APIs.** **Defect:** Physical file splits have kept individual implementation areas navigable, but more than twenty modules still extend `impl App` and can reach the whole live application state. Invariants for documents, input, workspaces, rendering, scheduling, and persistence therefore remain coupled by convention. **Build:** After item 171 establishes the effect boundary, migrate fields and their invariants incrementally into explicit owners: `DocumentSession`, `InputState`, `WorkspaceState`, `RenderRuntime`, `FrameScheduler`, and `PersistenceRuntime` (names may change if the ownership map proves a better cut). Each owner exposes domain transitions rather than public fields; cross-domain work travels through typed outcomes/effects, not back-references to `App`. Preserve the active-buffer whole-slot ownership law, fake-clock determinism, wasm gating, GPU recovery, and byte-identical behavior. Do not introduce a service locator, trait-per-method architecture, message bus, or flag-day rewrite; land coherent vertical slices with compile-time removal of the old field access. **Done:** `App` is lifecycle composition rather than the mutable home of every subsystem, and new workspace or persistence behavior has one obvious owner. **Verify:** First commit an ownership map and call-site census; add structural gates against direct cross-domain field access and growth of root `App`; run focused state/cache identity laws after each slice, then both conventions, full native, wasm, and the live scheduling/GPU probes. **Depends on 171 (landed). Routing:** deep owner plus independent ownership audit. **User-requested code-health work 2026-07-29.** 🟢 **TWO SLICES LANDED — merge `4821c63a`** (`73db7850` map/census, `ec94c743` WorkspaceState, `4399ebf0` PersistenceRuntime, `d56e8bb7` ratchets). **The item remains OPEN**: four domains are mapped and gated but not extracted.

**The ownership map is a deliverable in its own right** — `docs/app-domains.md`, with the same table as executable data in `src/app/tests/domains.rs`. 1,310 production and 586 test references classified, exhaustive by construction (no field in two owners, none unassigned). Remaining, with production references: `DocumentSession` 4 fields/363 refs, `RenderRuntime` 25/277 (**held for item 174**), `InputState` 27/164, `FrameScheduler` 12/123, `ProjectLocation` 9/51, and 22 host/lifecycle fields that stay on `App`. `config` (88 refs, 16 files, one writer) is the best remaining single-field slice.

**The map's own argument against the obvious next cut, worth keeping:** `gpu`'s 160 references across 23 files — the most dispersed field in the struct — are dominated by `gpu.window.request_redraw()`, which is a *scheduling verb wearing a render field*. Taking `RenderRuntime` first would pull 22 files into item 174's blast radius for no invariant. Relatedly, the item's `RenderRuntime`/`FrameScheduler` boundary is **not a real boundary**: for `theme_font_at`, `theme_switch_at`, `theme_settle`, `crossing_settle_at` the *stamp* is scheduling and the *effect* is rendering. The map assigns them to `RenderRuntime` and says so rather than pretending the line is clean; item 174 should move them and record the decision in the classification gate.

**Also worth knowing: `InputState` is the item's largest named owner and its lowest-value one.** 14 of its 27 fields are touched by exactly one file and `app/input/` accounts for 135 of 164 references; extracting it satisfies the item's letter and buys a struct rename. Its only genuine cross-domain leak is `cursor_px`, read at two sites in `apply.rs` — cheaper to pass the position than to move 27 fields. The map argues it should probably never be extracted as a 27-field struct.

⚠️ **THE ITEM NAMED TWO DIFFERENT DOMAINS `WorkspaceState`, and they share no fields.** This item lists it beside `DocumentSession`/`PersistenceRuntime` and closes on "new **workspace** or **persistence** behavior", which reads as the project-folder domain; item 173 says "in item 172's `WorkspaceState`, define one closed lifecycle for editor, brief contextual overlay, sustained summoned workspace, and suspended child audition", which is the summoned-UI domain. **The name went to 173's meaning** — 173 is the downstream consumer and the critical path to 114 — and the project-folder domain is now `ProjectLocation`, with `App::workspace` renamed `workspace_root` so the two cannot be misread for each other. **This item's Done clause is therefore satisfied for persistence and NOT YET for project-folder behavior.** Recorded rather than quietly counted as met.

**The one place byte-identity was consciously chosen over consistency** is `sync_cursor_icon`'s raw `popover_summon_bit()` read: documented, single call site, and law-counted so a second consumer fails by name. A byte-identity refactor preserves pre-existing bugs, so if a second pair of eyes is spent anywhere on this branch, that is the spot.

174. **Separate pure render planning from shaping/cache mechanics and GPU execution.** **Defect:** `TextPipeline` and the render directory jointly own scene policy, document geometry, cache invalidation, hit-test inputs, sidecar-visible facts, GPU resources, and feature-specific drawing. Tests often have to infer planned geometry from pixels, while render-touching work can accidentally couple presentation rules to device state. **Build:** Introduce one deterministic scene/layout planner that consumes `ViewState`, measured text inputs, theme capabilities, and viewport data and emits inspectable primitives plus interaction geometry. Shaping and cache ownership remain a distinct measured stage; GPU execution consumes the plan without deciding feature layout. Route drawing, hit-testing, and sidecar geometry through the same planned objects, migrating one coherent surface family at a time. Preserve O(visible) frame work, buffer-identity cache keys, rowlayout ownership, deterministic capture, and exact output for migrated surfaces. Do not build a retained widget tree, general scene framework, duplicate CPU renderer, or allocate an entire document plan each frame. **Done:** presentation decisions are testable without a device; GPU code executes rather than invents layout; drawn and interactive geometry cannot drift through parallel calculations. **Verify:** Plan-level geometry laws, drawn↔hit-test↔sidecar identity, buffer-swap/resize/zoom invalidation, allocation and reshape-count witnesses, exact before/after capture probes across representative worlds and surfaces, release frame benchmarks, both conventions, full native, wasm/WebGL. Every render slice gets the standing vision smoke. **Independent of 171–173; schedule away from item 114’s overlapping render files. Routing:** deep owner (`gpt-5.6-sol` high) with production-tier outcome audit. **User-requested code-health work 2026-07-29.** 🟢 **FIRST FAMILY LANDED — branch `claude/item-174-render-plan`** (`a10944a8`, `5d97e140`, `3ef6d85d`, `15e3eede`, `a1674d1e`, `629eb937`). **The item remains OPEN**: one surface family is migrated, the rest still own their geometry.

**`src/render/plan/` is device-free** — shapes nothing, measures nothing, reads no clock — and `plan_overlay_rows` emits one `PlannedRow` per candidate display line plus the interaction geometry. **The forward `row -> y` arithmetic and its inverse are module-private, and `overlay_row_top`/`overlay_row_of`/`overlay_row_index` are DELETED from `render/chrome`: the bypass is compiler-enforced, not grep-enforced.** Routed through the one plan: hit-test, range rails (draw and hit), the visual-selection band target and coverage grid, Pane band, Bars plates, chord plates, footer plate, slant clip bands, shaped-row item mapping, and `overlay_window_report`. Item 164's arrangement is preserved and re-pointed, not weakened — the planner is now the sole owner of `selected_display()` and `overlay_row_at` deliberately stays outside the transaction.

**Why this family, from the census rather than the brief:** `overlay_row_top` was a free function taking five loose scalars, re-assembled at **eight** call sites, and `overlay_draw.rs` then abandoned it and re-derived `first_top + k*lh` inline. "How many candidate lines does this card have" was written out **five** times, and the fifth added `+ empty.is_some()` in only one arm — a live defect. "Which line is logically selected" had item 164's owner **and a second, differently-clamped copy inside the test y-probe**, i.e. the oracle itself was a parallel calculation.

**Identity: 470 capture probes** (19 worlds × 10 overlay surfaces, plus 8 geometry axes over 7 worlds). **34 PNGs differ and every one is the deliberate fix; zero other PNG or sidecar bytes changed**, re-verified three times, with after-vs-after 0/470.

**The deliberate output change is a real defect the migration exposed.** `content_rows` omitted the empty-state notice line the card height had already paid for, so a `Bars` world's picker filtered to zero matches drew its **footer plate over the "no matches" row**. The 34 differing cells are exactly the two zero-match states on the five shipping Bars worlds plus geometry variants — measured on Galah as 19,295 px in one band `y 242..300`, one row height, nothing else.

**Two things mutation proof found that reading did not.** (a) `livingband::covered_rows` was stepping its own `first_top + k*lh` grid one module over from the plate with the same bug; the first source law's sentinels could not see a consumer *stepping off the band origin*, and adding that sentinel lit it up immediately. (b) **The device law's `sel_row` arm was tautological** — sidecar and plan both read `selected_display()`, so a planner that forgets grouped headers kept them in perfect agreement while pointing at the wrong row, and it was *watched staying green*. An independent oracle (the reported line must carry `overlay_selected`'s item) made it fail by name. This is the strongest instance yet of the repo's own lesson: agreement between two readers of one function proves nothing.

**Witnesses:** `plans_per_frame = 1` and `plan_rows_per_plan = 12` for a **106-item** palette — one `Vec<PlannedRow>`, 384 bytes per overlay frame, with the bench asserting *exactly* one plan (zero means measuring nothing, more than one means a consumer grew its own). O(visible) proven at 200,000 items. Reshape counts unchanged.

⚠️ **Two measurement honesty notes.** Release frames showed palette cells at median +8.1%, but the **untouched** cells moved median −0.2% across a −7.0%…+22.5% range on the same run — the whole S tier moved on every scenario including ones this change cannot touch, with five workers building concurrently. 0.17 ms is implausible for 384 bytes plus a ≤24-row scan, so the honest reading is *no palette-specific signal, confirmation owed on a quiet host*. Relatedly the **bench baseline was deliberately not re-banked**: re-banking tonight would freeze contention noise into every cell.

**Left for later slices, stated rather than implied:** document-content surfaces, search panel, HUD, gutter, outline, popover, whichkey and readout still own their geometry; the spell popup's anchoring is untouched (its rows are planned, its anchor is not); `overlay_secondary_top`/`overlay_split_bounds`/`overlay_strip_band` remain separate owners and folding the strip band plus secondary column in is the natural next slice; **no sidecar schema change** — publishing planned row rects would let a test assert row geometry with no device at all, but that is a schema bump plus a CAPTURE.md edit.

**Two premise corrections worth keeping.** (a) The brief said overlay rows "already have `render/rowlayout` as a partial owner" — `rowlayout` owns *horizontal* cell layout (primary/secondary budget, elision, rails) and had nothing to do with the vertical row-Y duplication where all the drift lived. They are orthogonal owners and the planner correctly did not subsume it. (b) The theme picker **retired its lens strip** (user decision 2026-07-15) and is now flat, so the grouped/sections path is exercised by the **command palette** under a non-All lens. A brief reaching for the theme picker to test grouped behaviour would have tested a flat card.

**Follow-up routed:** item 181.

118. **Pre-release world-loudness audit — repeat the 1–5 idle Room/Frame check before release.** **Audit definition:** “idle loudness” is how strongly a world asks for attention while the user is simply writing in page mode: palette, typography, margin pattern, and ambient motion count; summoned overlays do not. `1/5` is the quiet pole (Wagtail), `3/5` is recognizable/alive but comfortable for hours, and `5/5` is a deliberately rare statement world (Firetail). **Baseline from the 2026-07-26 design session, eighteen shipping worlds:** `1/5: 1`, `2/5: 10`, `3/5: 2`, `4/5: 4`, `5/5: 1` (mean 2.67). Bowerbird item 117 intentionally changes that to `1, 9, 3, 4, 1` (mean 2.72). **Direction for a twenty-world roster:** hover around 3 with awl’s calm bias; the provisional healthy shape is `1, 7, 7, 4, 1` (mean 2.85), not a symmetric theme-park bell curve. The current gap is the middle, not more 5s. This is a diagnostic distribution, never permission to turn up a world merely to fill a bin—each world still earns its own identity. Bilby/Brolga/Tawny/Saltpan are valuable quiet anchors; Galah and Mulga are candidates to inspect, not pre-decided promotions. **Run:** Immediately before release preparation, review every shipping world’s current Room in item 20’s dashboard at representative page widths, then observe every ambient world live in `--release`; independently assign 1–5 before looking at the baseline, reconcile the roster together, and record any chosen changes as separate concrete queue items. Include any nineteenth/twentieth worlds that have graduated by then; do not graduate Cassowary Light or any other candidate merely to hit twenty. **Done:** The final roster has a user-confirmed loudness map, its mean/distribution and outliers are explicit, near-duplicate intensity poles are named, and every proposed rebalance is either rejected on purpose or queued with a world-specific reason. **Verify:** Affordance-locating vision smoke over all Room captures; live-only confirmation of speed/calmness for Lava, Currawong stars, Bombora waves, Bowerbird cutouts, and any later ambient world. Pixel/sidecar arithmetic may prove territory and contrast but never claims the taste score. **Timed before release preparation; user design decision 2026-07-26. ✅ **PREPARATION LANDED — merge on `main`** (scripts only; no Rust, shader, Cargo or CI file touched, so **no full-native-suite receipt is claimed** — code-health and web-smoke clean). 🔵 **STILL AWAITING THE USER: the confirmed map, and the live `--release` sitting.** **The lane scored all nineteen worlds before opening anything baseline-related, and that search became the finding: the 2026-07-26 per-world map was NEVER WRITTEN DOWN.** Only the histogram survives — `queue.md` and its history, ROADMAP, THEMES, PHILOSOPHY and the design-session commits were all searched. **Independent map: `1, 10, 3, 4, 1`, mean 2.68.** Excluding Paperbark (which post-dates the baseline): `1, 10, 2, 4, 1`, **mean 2.67 — the baseline's exact aggregate**, arrived at blind. Corroboration: both stated anchors match (Wagtail 1, Firetail 5); the four 4/5s recoverable by inference from item 117 are **exactly the four it independently chose**; and **Bowerbird scored 3**, the first independent confirmation that item 191's `Finds` swap plus tuning hit its target rather than overshooting. **One unresolvable disagreement:** the baseline's second 3/5 is one of Currawong/Gumtree/Magpie/Mopoke and no record says which. ⚠️ **A CONTRADICTION BETWEEN TWO USER DECISIONS, found here and material to the pending Kite call:** item 132 commissions Kite as a **second 5/5**, while item 118's own target shape `1,7,7,4,1` carries **one** 5 and states *"the gap is the middle, not more 5s"*. Both cannot hold — either the shape becomes `1,7,6,4,2` (mean 2.90) or Kite is not a 5. **This needs answering as part of the Kite decision, not after it.** ⚠️ **The live `--release` observation was NOT run and was explicitly not claimed: the screen is LOCKED** (`CGSSessionScreenIsLocked: true`, checked at both ends of the session with the same probe `live-probe.sh` uses). Under the occlusion tripwire a live window presents zero frames, so a sitting now would produce a false result — item 113's unlocked session no longer holds. **Every ambient world's score is provisional on that check.** Offered instead, honestly bounded: deterministic phase trajectories converted to real seconds via the product's own `LAVA_SPEED` — fraction of right margin moved past 3 L\* at ten seconds is **Mangrove 0.351 · Firetail 0.344 · Bowerbird 0.077 · Bombora 0.027 · Currawong 0.002**, proving trajectory only, never cadence or calmness. **Premise corrections:** item 20's dashboard has **no width parameter** (fixed 1600×1000), so "representative page widths" required a new sweep; **item 132 calls Kite the nineteenth world when Paperbark already is** (Kite would be twentieth); and **in a code buffer the map does not describe what ships** — at `page_width_code = 100` a 1600px window leaves a 16px margin, the ground effectively vanishes and the roster's spread collapses toward palette alone. **Two metric repairs recorded because they nearly produced false findings:** linear luminance called every dark world's ground flat while the captures plainly showed shapes (gamma, not a finding) — CIE L\* is now the headline column; and `ink_cr` took a *percentile of the column*, whose area changes with the measure while the ink does not, so it moved between arms for one palette (Paperbark 10.75 vs 4.42) until a fixed extreme-pixel count made it arm-invariant. **Vision smoke 19/19 on every affordance except inline code, which is not locatable in Wagtail** — pixel-checked as byte-identical background, which is the 1-bit law's own sanctioned answer (THEMES.md) and therefore declared behaviour rather than a defect, though the affordance genuinely does not exist there. **Near-duplicate poles, named as the item asked:** Tawny/Mopoke (tightest — same `Dots{edge:false}`, edge 0.0000 both, L\* σ within 0.15), Magpie/Saltpan (edge **0.4444 on both to four decimals**), Bilby/Brolga (deliberate mirror per THEMES.md), and **Firetail/Mangrove inverted** — Mangrove measures louder on every static and motion column while ranking a step lower. **Six proposals, all labelled as proposals:** Galah's ground density (item 108's Gumtree precedent; the cheapest 2→3), re-verifying item 108 actually met its Done condition (Gumtree measures second-faintest at its shipped density), rejecting Mulga's promotion on purpose and recording why, resolving the Firetail/Mangrove inversion, making ROADMAP's "merge the tightest near-pair" call on the now-named pair, and **recording the confirmed map as durable data so the next run diffs instead of re-deriving four scores by inference.** Captures in `gallery/worlds/`, `gallery/item-118-loudness/` and `gallery/item-118-ambient/`. Dispatched now because the roster is momentarily STABLE at nineteen worlds — Bowerbird's `Finds` and item 186's ground space both landed, and Kite is held off main — so a distribution taken now describes what actually ships. Only the preparation is dispatchable: the item's Done requires a USER-CONFIRMED map and states that pixel arithmetic may prove territory and contrast but never claims the taste score. The lane produces captures, a vision smoke, and its own independent 1–5 proposal; the confirmation and any rebalance remain the user's.**

## Parked — explicit gate or future design

- **Export save-dialog scope:** macOS + Linux, one live-only cross-platform seam; capture uses an explicit path. Decided, not scheduled.
- **Per-world living-band choreography:** audition TwoShape/Slam/Soft against Morph; live feel is the oracle. Needs a design session.
- **Per-world copy-pulse differentiation:** possible future motion tweak; needs a design session.
- **Site deployment:** only on the user’s explicit word.

## Monitoring — non-blocking

- **Hands-on checks still useful:** writer-diff panel/Tab + zoom readout; heading-chevron mouse-press→toggle wiring; theme-picker felt input→present lag; Bombora drift speed / counter-motion / calmness over real seconds.
- **GPU memory:** no action unless the 6 GB symptom recurs; then probe the live surface with the window foregrounded.

## Release blockers and reminders

- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user’s explicit word. A dry run may precede them.
215. **Give the live-App capture the card semantics it currently cannot carry.** **Defect:** item 207's passive-surface fold takes its content rather than deriving it, because the render pipeline is the only holder of the live figures — word count, frontmatter language, and through-doc percent. So a `--screenshot-app` capture, which has no pipeline, writes a sidecar whose `semantic` has NO node for a card that is plainly DRAWN in the PNG. Which-key and the menu bar have no such dependency and do appear, so the gap is silent and partial rather than obvious. **Build:** extract those three figures into pure owners over `&str` (plus the buffer facts they already have), so both the renderer and the semantic fold read one owner. **The forbidden alternative, named so nobody re-derives it:** having the `App` recompute word count / language / percent for the snapshot would be a second description of the same fact, which is the exact drift item 207 exists to prevent. **Done:** a live-App capture's `semantic` contains a node for every card its PNG draws, and CAPTURE.md's honesty note about the gap is deleted rather than reworded. **Verify:** a no-wildcard sweep over the card roster asserting PNG-drawn ⇔ node-present; grapheme and CJK word counts through the extracted owner; mutation proof that removing a card from the fold fails by name. **Routing:** production tier. **Follow-up from item 207, 2026-08-02.** ⚠️ **ITEM 230 HAS SINCE LANDED IN THIS EXACT SEAM AND CHANGES THE JOB — read it before starting.** `ViewState::substitute_text` is now the ONE door that replaces shaped text, recording the document *and the caret's place in it*, and `TextPipeline::figure_source()` is the single seam saying which text the figures are over. **So the extraction this item asks for must route through those owners, not around them.** ⚠️ **And 230 proved this item's own gather is load-bearing on MORE than it claims:** `DocFigures::of` derives all three figures from ONE text and ONE caret, which is what makes it structurally impossible to take one figure from the owner and another by hand — **215's gather constrains WHAT QUESTIONS CAN BE ASKED SEPARATELY, not merely how they are filled.** Do not relax it to make the extraction easier. **Three figures, not two:** LANGUAGE vanishes under a History preview (a transcript carries no frontmatter), and the blast radius reaches past the card into the sidecar's `readout` block and `wordcount_text`. **One contract break already made deliberately:** `hud.lang` no longer mirrors top-level `doc_lang` — the latter is the SHAPED text's language, which the per-script font ladder must follow. **A named gap this item inherits:** the live `sync_view` preview substitution still has no behavioural law, because `hermetic()` has no GPU so `sync_view` returns early; closing it properly needs a `--screenshot-app`-driven harness. **Sequencing:** item 229 also rewrites `card::figures` (ideograph counting plus a unit label). Whoever goes second inherits the other's owner — do not run them concurrently.

218. **Make native screen-reader editing incremental so VoiceOver never reports awl as unresponsive.** **Defect:** The first real VoiceOver sitting found that editing basically works and that spoken characters/words are ordinary VoiceOver typing echo, but VoiceOver intermittently says “awl is not responding.” Inspection names a credible hot path: while AT is attached, every redraw clones the whole rope, runs UAX #29 over the entire document, projects every semantic node, includes `Tree` metadata, and republishes one monolithic document `TextRun`. AccessKit explicitly expects full trees only at activation and recommends changed-node updates afterward; awl uses the event-loop-only adapter path whose asynchronous activation forces the full-tree form. **Build:** use a synchronous mixed/direct activation handler backed by a thread-safe latest snapshot, then retain native projection state and emit atomic incremental `TreeUpdate`s. Represent document text as stable line or paragraph runs so an ordinary edit updates only affected runs, parent children when structure changes, selection, and focus. Keep `SemanticSnapshot` the one semantic owner; do not create a second document model, manually announce keystrokes, override VoiceOver typing echo, or run App transitions from an accessibility callback. **Done:** typing, deleting, selection, navigation, paste, undo, and surface changes remain correctly announced without stalls on small or large documents. **Verify:** latency/allocation witnesses across document sizes prove a one-character edit does not scan or publish the whole document; full-tree-only-on-activation and changed-node laws; Unicode/grapheme and multiline-selection round trips across run boundaries; mutation proofs restore the monolithic/full-tree paths; real unlocked VoiceOver typing and navigation sitting with no “not responding” report. **Routing:** deep accessibility/performance owner with a production-tier outcome audit. **User-reported and researched against Apple + AccessKit primary documentation, 2026-08-02.**

229. **A Japanese or Chinese manuscript's WORD COUNT is meaningless.** **Defect:** `card::figures::word_count` is `split_whitespace` over the manuscript body, and Japanese and Chinese put no spaces between words. Measured, not assumed: `今日はいい天気ですね。` — 11 characters — reports **1 word**, and `"今日はいい天気ですね。".repeat(500)` — **5,500 characters** — still reports `1 word · 1 min`. **The divergence is script-specific, not "CJK"-wide, and that is the assumption most worth pinning:** Korean is fine (`오늘 날씨가 좋네요` → 3, it spaces its words), mixed text is undercounted by its CJK half (`The title is 今日は…` → 4), and an ideographic space `U+3000` IS Unicode whitespace and does split. Graphemes already hold — ZWJ families, regional-indicator flags and decomposed `é` each stay one token. **Build:** give the readout a script-aware count through the ONE owner `src/card/figures.rs` — do not add a second counter beside it, which is the drift item 215 exists to prevent. A character/ideograph count for unspaced scripts is the conventional answer; whether the readout says "words" for such a document is a product decision, not a mechanical one. **Scope:** the count feeds the HUD readout and the semantic snapshot through one owner, so both move together or neither does. **Verify:** the pinned table above as a regression floor; a mixed-script document; `U+3000`; the grapheme cases unchanged; the sidecar and the drawn readout agreeing. **Found by item 215's measurement 2026-08-03, pinned rather than changed because changing the figure is a product call with sidecar consequences.** ✅ **USER DECISION 2026-08-03, so the product call is made and this is now buildable: COUNT IDEOGRAPHS AS TOKENS, and let the UNIT LABEL FOLLOW THE DOCUMENT'S DOMINANT SCRIPT** — the readout says **"words"** for a script that spaces its words and **"characters"** for one that does not. It does not claim a word count for a script that has none; it changes what it counts and renames the unit to match. **What that decision still leaves the owner to settle, named so it is not discovered late:** (a) **"dominant script" needs a definition and a threshold** — the pinned mixed case `The title is 今日は…` counts 4 today and is a real document shape, so decide whether dominance is a majority of counted tokens, a majority of characters, or the frontmatter `lang` (`docs/fonts.md`) when present, and pin the tie; (b) the label is a **second** thing the figure now carries, so `card::figures` returns a unit alongside the number and **both the drawn readout and the semantic snapshot must take it from that one owner** — item 215 exists to stop exactly the second description this invites, and item 230 has already routed both sides through one owner, so do not reopen that seam; (c) the sidecar's `readout`/`wordcount_text` change shape for such a document, which is a **CAPTURE.md-visible** change and may be a schema bump — check `capture::SCHEMA_VERSION` rather than assuming. **Verify additionally:** the pinned table with its expected units, a document that flips dominance across an edit (the label must follow, and must not flicker on a single character), and the `U+3000` and grapheme cases unchanged. ⚠️ **Renumbered from 227 on 2026-08-03: two orchestrators minted 227/228 independently within minutes. Theirs (AppImage, `v0.9.0`) were already cited in `RELEASING.md`, so these moved instead.**

231. **Name the CAUSE of the hosted-macOS gate hang. The fix is a SECOND item, scoped only once the cause has a name.** ⚠️ **REFRAMED BY USER DECISION 2026-08-03, from "fix the hang" to "diagnose it" — and the reframe is the most important line in this item.** One fix has already been attempted and **failed**: the `src/gpu_cache.rs` round cut `render::tests::` GPU program builds **52,083 → 5,577 (9.3×)**, `TextPipeline::new` 44 ms → 23 ms, `cargo test --bin awl` 133.8 s → 116.6 s, 3616 passing either side — and **the hang did not clear** (run `30770296246`). A second speculative fix would be worse than the first, because **the strongest remaining candidate is a SYMPTOM MASK**: `src/test_gpu.rs:27` holds a process-wide `OnceLock<(Device, Queue)>` whose own doc says it is "created once and never dropped", and recycling or periodically tearing it down would very likely turn CI green **without anyone learning what was exhausted**. The product's exposure is still an open question (see the ownership gate below), so a harness fix that greens the board **destroys the only instrument that can currently see whether a user on a VM is affected.** Diagnose, name the cause, THEN decide who owns the fix. **Defect:** `main`'s `mac (build + test)` job has been red for ~140 commits; `linux`, `web` and `mac live-probe` pass on every red run. It **HANGS, it does not fail** — exactly three tests (the runner's 3 vCPUs, i.e. every libtest worker thread) park at the same instant and never move, the job dies at its ceiling or the VM dies, and the `cargo`/`awl-…` orphans **survive SIGTERM** because they are parked in `poll(PollType::wait_indefinitely())`. Bisected over six sequential probes to **`8207e519`** ("item 194: one camera, one projected cylinder, cropped at the page"), **both boundaries measured** — parent `36707d06` GOOD, `8207e519` BAD, no re-run contradicting a first reading. That commit takes `THEMES` 19 → 20, adding `KITE` with `Background::WarpedGrid` and **+267 lines of `background.wgsl`**; the tests that wedge are roster sweeps. **ELIMINATED — do not re-derive; each was killed by measurement.** (a) **The shader:** 15 `backgrounds_item132`/`warp_tunnel` tests pass cleanly **six minutes before** the wedge, in two independent logs; no unbounded loop in the WGSL and `warpgrid.rs` touches no wgpu at all — **do not start by staring at it.** (b) **A single bad test:** the victim varies between runs (`scroll_pos` in one log, `split_pane`/`stars` in another), so the commit poisons the device rather than owning the hanging test. (c) **Concurrency:** `RUST_TEST_THREADS=1` **WEDGES**. (d) **A per-device resource:** the mac and linux conventions — **two separate processes with two separate wgpu devices** — stopped **within 10 MILLISECONDS** of each other and never moved, so **the contended resource is SYSTEM-WIDE: the VM's virtualised Metal stack itself.** (e) **Program-build volume:** the 9.3× cut did not clear it, and `--skip render::tests::` **COMPLETED** — 2860 tests per convention in 110 s while standing up its own device per test and building **~80,000 GPU programs in aggregate**, i.e. far MORE total GPU work. Those tests create AND DESTROY devices, forcing driver-side reclamation, where `render::tests::` piles transient resources onto one device never torn down. **It is not how much you build — it is how much you pile on a device the driver never reclaims.** (f) **RAM:** `free_bytes` steady at ~2.37 GB. (g) **Software adapters as a stand-in:** two independent lavapipe stacks ran `render::tests::` at both bisect boundaries and neither ever hung (item 232) — a software rasteriser has no system-wide GPU resource for a cross-process wedge to exhaust, so it cannot reproduce this class even in principle. **STILL UNKNOWN, AND THIS IS THE WHOLE ITEM:** WHICH resource in the virtualised Metal stack is exhausted. "Cumulative exhaustion of a driver-internal table — compiled-pipeline slots, or allocations wgpu only reclaims on poll" remains a **labelled hypothesis with no confirming measurement.** **FIRST DELIVERABLE — a LOCAL REPRODUCTION**, because without one every hypothesis costs a ~50-minute CI cycle and the last one that looked excellent was wrong. ⚠️ **The untried arm is a macOS GUEST VM on the Apple Silicon host** (Virtualization.framework — `tart`, or UTM). **Item 232's negative result does not apply to it:** that measured *software rasterisers on Linux*, whereas a macOS guest gets genuine **paravirtualised Metal**, the same class of stack as the hosted runner, and nothing local has ever exercised that axis. Measured preconditions 2026-08-03: **179 GiB free** on the dev host and **no VM tooling installed** (`tart`, `utm`, `qemu-system-aarch64` all absent), so the setup cost is real — state it rather than assuming it is free. **A negative here is a publishable result too**, and either way it directly feeds item 232. **SECOND — instrument the resource class, using the fast local oracle that already exists.** Item **239** measured that under a fixed 4 GiB ceiling at `--test-threads=1`, `render::tests::` walks RSS **monotonically to an OOM kill**, commit-correlated (good parent reaches test **199** twice, `8207e519` reaches **160** twice, alternated to control for drift) — **~4 local minutes against a 50-minute CI cycle.** ⚠️ **Carry 239's own caveat: every container death was a prompt SIGKILL with `OOMKilled=true`, NEVER the hosted runner's park-forever-with-memory-flat, so this is a DIFFERENT failure mode and bounding the growth is NOT proven to prevent the hang.** It is a fast proxy for *a* leak, not evidence about *the* hang. **The suspects, in the lane's own order of promise:** the per-call `glyphon::Cache` + `TextAtlas` (`render/tests/mod.rs:140,155`, `images.rs:598`, `chrome_panels.rs:1703,1750,1802`) and every `offscreen()` texture and readback buffer — **allocations, not programs**, which wgpu reclaims only on poll; the residual ~5,577 program builds are dominated by the direct `BackgroundPipeline`/`SelectionPipeline` helpers (~1,800 calls, 55 call sites). **THE DECISION GATE — what "then decide" means, and the item is not done without it.** Once the cause is named, answer: **is the PRODUCT exposed, or is this test-harness-only?** The asymmetry that decides it: the per-frame `create_shader_module` + `create_render_pipeline` churn exists **only in the test helpers** (3 sites); the live app builds `BackgroundPipeline` **once** at construction (`pipeline_draw.rs:32`) and `prepare()` thereafter only uploads uniforms *including the shader id*, so switching themes never rebuilds the pipeline and a user pays one compile per launch. **But that rests on the churn hypothesis, which the 9.3× null result has now WEAKENED** — if state accumulates from the WarpedGrid draw itself, or from allocations rather than programs, **a user on a VM IS exposed** and the fix belongs to the product, not the harness. Only after that answer does a fix get scoped. **Do NOT land a fix under this item.** Specifically: do not recycle or tear down the shared test device, do not bound the allocation growth (that is item 239's scope and it is explicitly not a fix for this), and do not tune anything, until the cause has a name and the product/harness question has an answer. **If the diagnosis converges early and the fix then looks obvious, it still lands as a SEPARATE item so the causal claim and the change stay separately reviewable.** **Carry-forward facts a new owner would otherwise lose.** ⚠️ **wgpu 29.0.3: `wgpu::Device`'s `PartialEq` reports two separately requested, simultaneously live devices as EQUAL** (measured) — a device-keyed cache is therefore impossible; the first draft trusted it and 648/3616 tests died with `BindGroupLayout does not exist`. A `cfg(test)` cache also **must not be thread-local**: libtest gives every test its own thread, which left builds at 86,061 — no change at all. **One law initially PASSED its own leak mutation**: drawing one world at a time lets each `prepare` overwrite the last, and only building and preparing all twenty BEFORE any draws exposes it. **Tooling:** probe driver at `~/.awl-item231/probe.sh` (outside the repo); `scripts/ci-mac-bisect.sh` on branch `claude/ci-mac-bisect` (`c336cc1a`, never pushed) carries `probe`/`verdict`/`next`/`cleanup`. ⚠️ **Two harness bugs, both of which scored a 60-minute hang as a PASS, both the same shape — an unfinished step wearing a finished step's field:** `gh` encodes an unfinished step as `conclusion:""` (never `null`), and a step killed by the job ceiling reports `status:"completed"` with `conclusion:"cancelled"`. **A harness reading a status field must enumerate what it accepts, never test for inequality.** ⚠️ **And a probe-integrity trap from item 232's lane:** a cross-commit pass **silently scored the same binary twice** — both trees extracted within the same second, so Cargo's mtime fingerprint reused the other tree's artifacts. Use a target dir per tree plus a provenance assertion that fails on mismatch. **Done:** the exhausted resource is NAMED with a confirming measurement rather than a hypothesis; the product/harness question has an evidenced answer; and the fix is scoped as its own item. **Verify:** whatever names the cause must also PREDICT THE BOUNDARY — it has to explain why `36707d06` survives and `8207e519` does not, why `--skip render::tests::` survives while doing more total GPU work, and why two processes on separate devices stop within 10 ms of each other. **Routing:** deep tier, one owner end to end. **Reframed by user decision 2026-08-03; the earlier "shared wgpu device wedges" mechanism is FALSIFIED and must not be carried forward.**

237. **Item 234's first law has a CONSTANT arm that cannot fail on any product change.** **Defect:** found by the item-236 lane while sweeping for fabricated-geometry laws, and left deliberately rather than taken mid-item. `a_workspace_rows_text_sits_inside_its_own_plate_on_every_world`'s **arm 2** computes `overrun` from `bar_full_span(band_x, band_w)`, which is pure `(band_x + 8, band_w - 16)` — **so the overrun is `BAR_SIDE_INSET` identically in every cell.** No world, width, lens or DPI enters it. Per cell it asserts `8.0 > 1.0`; at the end it asserts `max(8.0, 4.0) == 8.0`. **It cannot fail on any product change except editing the constant itself.** Arm 1 is what actually sweeps and is sound. The law's doc also promised a third arm the body does not have (the phantom bullet is already removed). **Decide, then act:** this is the shape item 217 faced when its device law stayed green under its own no-op mutation, and the precedent there was to **delete rather than ship vacuous**. Either delete arm 2, or re-aim it at something the product can actually change — if the intent was "the plate's inset is what the drawn text respects", the oracle must read the DRAWN text, not re-derive the same constant. **Do not simply leave it**: a law that always passes is worse than no law, because it reads as coverage. **Verify:** whatever replaces it must fail by name on item 234's original defect — text sitting `BAR_SIDE_INSET` outside its plate at either edge — and be mutation-proved on a plate-drawing world. **Found 2026-08-03; arm 1 is unaffected and still guards the item.** ⚠️ **CARRY THIS FROM ITEM 236, which fixed the sibling class and is the reason to look here at all: `overlay_prepare_bar_scrims`'s gate reads `backing == BarePlates` — the SAME card-vs-row substitution, in SHIPPING code — and it is CORRECT AS-IS. Do not "fix" it to `draws_row_plates()`.** That scrim pass is the only thing that clears `panel_card` on a bare-plate world, so gating it out would let a stale instance survive into a Diagonal frame. It looks exactly like the defect this item is about and is not one; 236 caught it as a near-regression and recorded it rather than shipping it. **Also from 236, and directly useful here:** `ListStyle::draws_row_plates()` now exists as the one owner of "does this style back its rows with plates", `overlay_selection_rects` is the one place a list style becomes row surfaces, and `overlay_bar_rects_probe` **refuses** on a plateless world — so a replacement arm has a real oracle to grade against and cannot fabricate one. **Prefer 236's pattern for the exclusion too:** earn it by measurement (the frame must emit no row surface at all on the excluded world, at the same fixture and DPIs) rather than by a name list, so a world that starts drawing plates fails instead of dodging the sweep.

238. **The GitHub rename left stale URLs in shipped artifacts — including the GPLv3 source offer.** **Defect:** the repository is now `Frank-P-Lu/awl-editor`; `git remote` still says `awl-next.git` and every push works **through GitHub's redirect**, which is exactly what makes this latent rather than loud (verified: `gh api repos/Frank-P-Lu/awl-next -q .full_name` returns `Frank-P-Lu/awl-editor`). **The surface is wider than tooling.** ⚠️ **`scripts/package-linux.sh:126` is the GPLv3 §6(d) SOURCE OFFER** — a licence obligation pointing at a name that survives only by redirect, and item 226 added it to every release tarball. Also `src/mac_about/facts.rs:47` (`GITHUB_URL`, shipped in the About window), `README.md:73`, **16 links across `site/index.html`, `site/check.html`, `site/credits.html`, `site/guide.html`**, and `scripts/ci-mac-bisect.sh` lines 129/184/187 on the unmerged branch `claude/ci-mac-bisect`. **Build:** one owner for the repository URL wherever a shipped artifact names it, and a law that fails if a tracked file spells the old name. Decide deliberately whether `git remote` is repointed — a redirect that works is not a reason to keep a wrong name in a licence notice. **Verify:** grep-law over tracked files; the tarball's source offer resolves without redirect; the About window's link resolves; `scripts/site-links.sh` still passes. **Touches Rust and the site, so it needs the full native gate and web smoke. Found by the item-232 lane 2026-08-03; none of it was in that lane's diff.** ✅ **USER DECISION 2026-08-03: ADOPT `awl-editor` everywhere, and REPOINT `git remote`** — "awl-editor is much better." The deliberate-decision clause is answered; a redirect that works is not a reason to keep a wrong name in a licence notice. ⚠️ **THE ITEM'S OWN LAW CLAUSE IS WRONG AS WRITTEN, corrected here by survey before anyone builds it.** "A law that fails if a tracked file spells the old name" would fail on legitimate content: **the local working directory is still `awl-next`**, and `src/render/rowlayout.rs:238` — **inside `#[cfg(test)] mod tests`, verified** — plus `src/capture/tests/schema_chrome.rs:188`, `src/render/tests/chrome_overlay.rs:510` and `src/render/framebench.rs:103` all use `"awl-next"` as a **sample project name in a fixture**, which is realistic test data precisely because it is the real directory. **The law must ban the old repository URL (`github.com/Frank-P-Lu/awl-next`), not the bare token** — and if the token is ever banned outright, those fixtures are the thing to rename first, deliberately. ⚠️ **The surface is also LARGER than the item states.** Measured: not "16 links across `site/*.html`" but **18** (`check.html` 4, `credits.html` 8, `guide.html` 2, `index.html` 4) — **plus `site/check.js:47`'s `RELEASES_URL` and 8 in `site/llms.txt`**, which the item names nowhere and which no HTML sweep would catch, for **27 in `site/` alone**. Two more outside it: **`CLAUDE.md:1`** and **`run-linux.sh:2`** name the repo in prose. ⚠️ **And one is a BUILT artifact:** `site/editor/awl-347842567538f209_bg.wasm` contains the string twice, so it only clears on a wasm rebuild — a grep-law scoped to tracked files must either exclude built artifacts by path or the web build must be run before the law can go green. **`scripts/ci-mac-bisect.sh` lines 129/184/187 are on the unmerged branch `claude/ci-mac-bisect` and cannot be fixed from `main`** — either land that branch first or fix it there and record which. **One owner is achievable in Rust only** (`src/mac_about/facts.rs::GITHUB_URL` already is one); shell, HTML, JS and txt cannot share it, so **the cross-language enforcement IS the law** — say so rather than implying a single constant reaches all 30-odd sites.

239. **Bound the render suite's allocation growth, and give it a portable oracle.** **Defect:** item 232 measured, under a fixed 4 GiB container ceiling at `--test-threads=1`, that `render::tests::` RSS climbs **monotonically to an OOM kill** — and how far it gets is commit-correlated: the good parent `36707d06` reaches test **199** twice, the bad `8207e519` reaches test **160** twice, alternated good/bad/good/bad to control for drift. **The bad tree spends the same budget 20% sooner.** These are item 231's named residual suspects — per-call `glyphon::Cache` + `TextAtlas`, and every `offscreen()` texture and readback buffer, which wgpu reclaims only on poll — made visible in **four local minutes** rather than a 50-minute CI cycle. ⚠️ **This does NOT show that bounding the growth would prevent the hosted-mac hang**, and nothing yet does; treat it as a strong lead, not a fix. ⚠️ **RSS is not a portable oracle** — the same suite peaks at 448 MiB on the dev host's Metal, so a law written against RSS would measure the container and not the product. **Build:** a wgpu-side allocation counter that travels across backends, then bound what the suite accumulates. **Verify:** the counter reproduces the 199-vs-160 split without a container; a law that fails when the suite grows its per-test allocation; mutation proof. **Follow-up to items 231 and 232, 2026-08-03.**

240. **Five of nine shaders have NO offline WebGL2 validation.** **Defect:** `src/render/tests/webgl_shader_validation.rs` validates only **4 of 9** shaders against GLSL ES 300. `blur.wgsl`, `caret.wgsl`, `caret_glyph.wgsl`, `image.wgsl` and `spellunderline.wgsl` have **none** — their WGSL is only ever validated at native runtime against Metal and Vulkan, **so a construct the GLSL-ES backend rejects would reach the browser fallback unseen.** awl is one core and two builds, and the WebGL2 fallback is a shipping target; a shader that natively compiles and web-side does not is exactly the failure this file exists to prevent, and it is currently guarding fewer than half of them. **Found by the item-235 lane**, which added `rotated_label.wgsl` (both stages, both pass) and noticed the roster it was joining. **Build:** validate every shader in `shaders/`, driven off the directory rather than a hand-kept list, so a tenth shader cannot be added without one. **Verify:** the sweep fails by name when a shader is added and not validated — mutation-prove it by adding a construct GLSL ES 300 rejects and watching it go red; confirm all nine currently pass, and if any does **not**, that is a live web defect and outranks the sweep. **~40 lines. Found 2026-08-03.**

241. **A theme switch takes ~100 ms to settle while doing ~2 ms of work — and the instrument built to name the dominant cost cannot see it.** **Defect:** user-reported live on 2026-08-03 with a `--debug` HUD photograph, switching **from Kite to Mulga**: `theme latest 103.6 ms · theme worst 117.2 ms`, on an otherwise healthy frame (`frame 3.9 ms · worst 10.3`, `key→px 6.0 ms`, 4530×2756 @2.0x). **The breakdown line is the finding.** `src/themeswitch.rs` exists so "the dominant cost NAMES ITSELF instead of being guessed", and for that same worst transaction it reads `font 0.0 · reshape 0.1 · rowgeom 0.0 · atlas 1.8 · present 0.2` — **2.1 ms across all five phases, 1.8% of 117.2 ms.** Whatever costs the other ~115 ms is outside every phase the instrument covers. **This is the symptom item 202's repair round was supposed to have closed:** `docs/fonts.md` records the flat-100 ms landing as punishing an isolated step at "settle ~124ms, reopening the exact 'felt theme-switch freeze' item 37b's own commit was about", which is what the leading-edge-plus-trailing-coalesce rule was built to fix. **Diagnose before fixing — three candidates, and the item must not assume one.** (a) **The debounce is being paid on a step that should be immediate.** `THEME_FONT_DEBOUNCE_DEFAULT_MS` is **100**, and 103.6 − 2.1 ≈ **101.5**; if `theme_font_reshape_decision` returns `Coalesce` here, the transaction is ~98% deliberate waiting. (b) **The classification is correct by its own letter and wrong for the felt act.** Arrowing through the theme picker genuinely IS a burst — every step lands within `window` of the last reshape — so every step coalesces and every step costs the full trailing settle, even though `--bench-theme-burst`'s reshape cost is 10–35 ms and the measured reshape here was **0.1 ms**. The rule's cost model was calibrated when an isolated reshape cost ~39 ms; a world pair needing no font change has nothing to coalesce and the leading-edge test should reflect the work, not the clock. (c) **The residual is not the debounce at all and the instrument is simply blind.** Nothing between input and the five phases is timed, **including anything Kite's `Background::WarpedGrid` does** — and the user's report is specifically about leaving Kite. ⚠️ **The board's item-231 claim that "switching themes does not rebuild the pipeline" was made about the CI wedge and has never been verified against this readout; do not carry it in as an established fact.** **The cheap discriminator already exists and costs one live run:** `AWL_THEME_FONT_DEBOUNCE_MS=0` is item 202's own A/B escape hatch. If the settle collapses to single-digit ms, it is (a)/(b) and the fix is in the scheduling rule; if it stays ~100 ms, it is (c) and **the first deliverable is extending `SwitchPhases` to cover the gap**, because a five-phase breakdown that accounts for 1.8% of its own headline is worse than none. Run it from Kite and from a quiet world to test the Kite-specific claim, and on both an isolated commit and a stepped burst. **Build:** whichever the discriminator names — do not tune the constant before the chain names the break, and do not close this by widening the debounce's A/B override into a user setting. If it is the scheduling rule, the leading-edge test should key on whether real reshape work is pending rather than on elapsed time alone; `sync_theme_font_timed`'s own early `None` ("no reshape work — nothing to time") is evidence the path can already tell. **Scope:** the readout is DEBUG-mode and LIVE-ONLY by construction (`settle_lines` returns empty for `None`, the only value a capture holds), so no headless probe can close this alone and none should be faked. Preserve item 202's burst coalescing — an N-step run must not go back to N reshapes, which is the regression 37b's zero-window landing caused. **Done:** committing a theme change settles in a time proportional to the work it actually does, the breakdown line accounts for the bulk of its own headline number, and a rapid picker run still coalesces. **Verify:** live `--release` before/after on the reported pair and at least one font-changing pair, recorded from the same HUD; the movement-latency distribution (`docs/render.md`, `--live-script`'s `latency` step) across an isolated commit and a 30 ms-apart burst, so the burst arm cannot regress unnoticed; a unit law over `theme_font_reshape_decision` for whatever new input it gains; a `SwitchPhases` law that fails when recorded phases fall below a stated fraction of the transaction, so this blind spot cannot silently reopen; `--bench-theme-burst` with its reshape-count witness intact (CLAUDE.md: one theme bench "measured" 5ms while nothing reshaped). **Needs an unlocked, foregrounded display — see the idle-lock warning at the top of this board.** **Routing:** production tier with deep live-render review. **User-reported with a HUD photograph 2026-08-03.**

242. **Chrome has no default pixel space, so every hand-authored constant is an independent coin flip — and the boundary that fixes it already exists.** **Defect:** awl draws in device pixels and nothing else; "logical" here means only "multiplied by `dpi` on its way in". The text and caret families already pass through ONE boundary — `Metrics::with_dpi` (`src/render.rs:226`) scales 13 base constants by `s = zoom * dpi`. **Chrome was never enrolled, and its constants are mixed three different ways.** Measured, not assumed: (a) `overlay_text_hpad` (`src/render/chrome/overlay.rs:91`) returns `BAR_SIDE_INSET + BAR_TEXT_PAD` (8.0 + 13.0) for `Bars` and a bare `12.0` for `Pane`/`Diagonal`, multiplied by nothing — **truly physical**; (b) `CARD_MAX_W` is **NOT** raw, as the 131 finding recorded, but a **GROW-ONLY HYBRID** — `overlay_desired_w` multiplies it by `overlay_pixel_scale()` under `scale.max(1.0)`, so it is physical at scale ≤ 1 and logical above, deliberately, to fix a zoom-blind card collapse (documented at `chrome/overlay.rs:124`); (c) **`overlay_lh` is itself mixed** — `metrics.line_height * effective_overlay_scale()` (dpi-scaled) **+** `effective_overlay_leading()` **+** `overlay_row_gap()` (both raw, the latter a theme-authored `ListStyle::Bars { gap }`). So the one quantity the tree treats as logical already drifts out of proportion across displays, today, on shipping worlds. The `_LOGICAL` suffixes sitting in `src/render/chrome/` (`ROW_STEP_LOGICAL`, `SPINE_WEIGHT_LOGICAL`, `SPINE_CORNER_LOGICAL`, `ATTACHMENT_BAND_INSET_LOGICAL`, `SELECTED_OUTWARD_LOGICAL`, `SELECTED_SPINE_WEIGHT_LOGICAL`, `CLUSTER_CONNECTOR_LOGICAL`) are the 131 lane reaching for exactly this by naming convention — the right instinct with an unenforceable mechanism, because **a suffix is not a type**. ⚠️ **A live appearance bug is implied and must be established FIRST, before any migration:** `chrome/overlay.rs:124` says the caps are "tuned for the 1:1 capture canvas". If that is also true of the raw insets, chrome padding renders at **half its tuned physical size on every Retina display**, including the dev machine — structurally invisible because `opts.dpi.unwrap_or(1.0)` (`src/capture/opts.rs:305`) means every capture, law and gallery shot runs at the one scale where the bug cannot exist. Capture the palette at `--capture-dpi 2` and compare the padding-to-text ratio against 1×; record the answer either way, because a negative result is what licenses treating this as pure hygiene. **Build:** enroll chrome in the EXISTING boundary. **Do not extend `theme::ground_space` and do not build a chrome sibling of it** — item 186's per-variant declaration table is right for GROUNDS, where physical is genuinely common (a stipple or dither must land on the physical grid or it moirés, hence `wagtail_stipple_cell_px(dpi)`); chrome wants the opposite default, and copying 186's shape would import the wrong answer with more machinery. Give chrome one scaled owner with **logical as the default and physical as the annotated exception**, enforced by the compiler or a no-wildcard law rather than by a name — a bare `f32` length reaching a draw call without passing the owner should fail by name. **Classify before migrating: only one of chrome's four unit families moves.** Device-px lengths (`BAR_SIDE_INSET`, `BAR_TEXT_PAD`, `HPAD`, `VPAD`, `PLACARD_INSET`, `CHIP_HPAD`/`CHIP_VPAD`, `CARD_EDGE_INSET_FLOOR`, `ANCHOR_GAP`) migrate; character units (`RAIL_GAP_CHARS`, `MIN_PANE_CHARS`, `MARGIN_COLUMN_GAP_CHARS`) and row units (`OVERLAY_HINT_ROW`, `OUTLINE_GAP_ROWS`) are already correct by construction and must not be double-scaled; pure ratios (`OVERLAY_UI_SCALE`, `WORKSPACE_MARGIN_FRAC`, `TIMELINE_MAX_FRAC`, `PLACARD_SIZE_STEP`, the alphas) must NOT scale and the law must not force them. `CARD_MAX_W` gets an **honest third classification** — a worker who records it as plain physical will "fix" it and reintroduce the zoom-blind collapse its own comment describes. **Glyphs still rasterize at device resolution:** shaping at logical size and scaling the raster blurs text, which is why every solution to this problem sizes the backing store separately from layout. `Metrics` keeps handing glyphon physical sizes; only the LAYOUT side gains a logical view, and that seam stays inside `Metrics`. **Scope:** chrome only. There remains exactly ONE coordinate system — device px — and this decides which constants are multiplied on the way into it; it is not a coordinate-system rewrite. The document/caret/text families already pass the boundary; `src/theme/` ground space is untouched; other worlds' output stays byte-identical at the capture scale. **Done:** a new chrome length cannot be authored in the wrong space without a compile error or a named law failure; the four unit families are recorded where the constants live; and the DPI tier a capture certifies is stated accurately — `--capture-dpi 1` is the identity path and is evidence about nothing else. **Verify:** byte-identity at dpi 1.0 across the full surface roster — the 19-world × 5-surface, 95-capture PNG+sidecar hash 131a/b already ran (190 files) — since the multiply is the identity there and every existing law must be untouched. **The value is entirely in laws that do not exist yet:** sweep `dpi ∈ {1.0, 2.0}` and assert that every migrated quantity holds its ratio to `line_height`, that the ratio family does NOT scale, and that `overlay_lh`'s three terms scale together. Mutation proof per family — break one constant's enrollment, watch the sweep go red by name, paste the panic. A Retina taste pass across the world roster once padding doubles, plus affordance-locating vision smoke over the palette at 2×, because this DOES change the Retina look and that is the point. **This retires item 131c's blocker instead of answering it** — 131c is BLOCKED on the chrome pixel-space decision in item 131's finding (d); with a default in place the diagonal authors its numbers like every other quantity and no design call is owed. **Sequence BEFORE 131c**; 131d/e follow. **Routing:** deep tier, one owner end to end — a classification and typing decision, not a mechanical sweep. **User design decision 2026-08-03, from the 131c chrome-space discussion; the measurements above were taken during it.**

243. **Split the hosted-macOS CI job so the arm that PASSES gates today, and the one that hangs is tolerated by name.** ⚠️ **This is the resolution of item 232, and it is NOT that item's recommendation — the lane recommended C (declare the whole hosted-mac job the arm and gate on it); the USER CHOSE THE SPLIT on 2026-08-03. Do not read C as the decision.** **Why C was rejected:** gating on a red job blocks `main` until item 231 is fixed, and 231 is now a DIAGNOSIS item with no promised fix date. **Why A and B were rejected (item 232's measurement, already landed at `5bc771ca`):** A — a local container with a software adapter — cost a 1.67 GB image and ~14 GiB of Docker VM disk that did not return, for **zero coverage of the target axis**, because no portable software rasteriser reproduces the wedge (two independent lavapipe stacks ran `render::tests::` at both bisect boundaries and neither ever hung); B — a slow CI job — **already exists** as the `linux` job, which has run that exact arm green through the whole ~140-commit red streak. **Build:** split `mac (build + test)` into two jobs. **`mac (build + test, minus render::tests)` becomes GATING immediately** — item 231's discriminating probes measured this arm **COMPLETING**, 2860 tests per convention in 110 s, while standing up its own device per test and building ~80,000 GPU programs in aggregate, so it passes today at no cost. **`mac (render::tests)` becomes a separate ALLOWED-FAILURE job, pinned by name to item 231** in the workflow file itself, not only on this board. **What it buys:** real virtualised-GPU signal over ~95% of the suite starting now, the tolerated red shrinks from "the whole mac job" to one named subset with an open item behind it, and **`main` is not blocked**. When 231 lands, the second job goes green and is promoted to gating — no further decision needed, which is why this shape needs deciding only once. **The rationale, because it is the actual lesson of the streak:** the failure was never that a job was red — it was that a red job carried no information anyone consumed, so nothing distinguished "the known wedge" from "a new regression". The split restores that distinction mechanically. **Also in scope, and the other half of item 232's Done clause:** state the tier a receipt certifies accurately wherever receipts are described — `CLAUDE.md`, `.orchestrator/README.md`, `RELEASING.md` — namely *"sound on the hardware the receipts run on, with virtualised-GPU behaviour untested by any local gate"*. `CLAUDE.md` already carries a version of this sentence; make the three agree. **Scope:** CI configuration and docs. Do **not** add a local software-adapter arm (measured negative, twice) and do **not** make every developer's local gate slower — item 232 refused that explicitly. **Done:** a green `main` means the non-render mac arm passed on virtualised Metal; a red `render::tests` mac job is attributable to item 231 by name from the workflow file alone; and no doc claims a receipt covers an axis it has never exercised. **Verify:** the gating job passes on a hosted runner; the allowed-failure job's red does not fail the workflow; a deliberately broken test in the gating half DOES fail it (mutation proof — an allowed-failure misconfiguration that silently tolerates everything is the obvious way to get this wrong); the three docs say the same thing. **Routing:** production tier. **User decision 2026-08-03, resolving item 232.**

244. **Bowerbird's ground POPS every ~67 seconds. Cut the drift entirely; give the companions a value breathe instead.** ✅ **USER DESIGN DECISION 2026-08-04 — the product call is MADE and this is buildable.** Three parts, and the third holds whatever happens to the first two. **(1) The field translation goes.** **(2) It is replaced by a per-element VALUE breathe on the COMPANION role.** **(3) The law fix is UNCONDITIONAL** — it is a defect on its own terms, not a consequence of the redesign. **Defect, measured not inferred.** The Organic ground's vertical drift is DISCONTINUOUS at the shared clock's wrap. `render/layers.rs:88` feeds Organic its own drift inline — `waves_render_phase() * TAU / LAVA_LOOP_CYCLES` — so `g.drift` runs `0 → TAU` across one loop. `shaders/background.wgsl:723-726`'s `organic_rgb` then takes **`sin(g.drift)`** for X and **`cos(g.drift * 0.73)`** for Y. Across the wrap X is `sin(TAU)=0 → sin(0)=0`, **continuous**; Y is `cos(0.73·TAU) = −0.125333 → cos(0) = +1.000000`, a **1.125333 discontinuity in normalised units**. Bowerbird ships `scale_px: 195.0`, so Y amplitude is `max(195·0.10, 9.0) = 19.5 px` and **the whole field snaps ~21.9 px vertically in ONE FRAME, every ~67 s** (`LAVA_LOOP_CYCLES = 2.0` at the shipped rate). **The house rule it dodged is already in writing:** `src/background/waves.rs` states for Waves that `WAVE_DRIFT_CYCLES` is an INTEGER so *"the drift ... meets its own endpoint exactly where the clock wraps — seamless, no pop"*, citing the twinkling-stars' "integer cycles per ambient loop" law (THEMES.md). **`0.73` is not an integer**, and Organic is the one ambient consumer that breaks it. ⚠️ **THE LAW NAMED FOR THIS BUG IS VACUOUS — the sharpest instance of that class yet found here, and it is why the pop shipped.** `render/tests/backgrounds_item117.rs:65`'s `organic_phase_moves_and_wraps_without_a_catchup_jump` is GREEN while the pop ships, for three compounding reasons: (a) it calls **`waves_drift_radians`, which Organic never uses** — Organic computes its drift inline at `render/layers.rs:88` — so it guards the wrong owner entirely; (b) it never applies the **`0.73`**, the single term that breaks; and (c) what remains, `sin`/`cos` of `0.0` versus `TAU`, **asserts the 2π-periodicity of trigonometry rather than any property of awl — it cannot fail for ANY value of ANY constant in this repo.** **WHY THE TRANSLATION GOES, and it is a world argument rather than a motion-is-bad argument.** Every other ambient ground earns its motion from its subject: Bombora is a **sea** and seas move; Currawong is a **star field** and stars twinkle; Kite is a **travelling grid** and travel is the point. Bowerbird's ground is `Finds` — *"the crisp COLLECTED-TREASURE grammar: one large anchor, one smaller companion offset across its edge, and one tiny cut-out"*. **A bower is an ARRANGEMENT: objects deliberately placed and then left alone.** Drift does not decorate that, it disturbs it. ⚠️ **And the motion is PERCEPTIBLE — user-confirmed 2026-08-04, which forecloses the "too slow to matter" defence.** Perceptible persistent motion in the GROUND competes with the caret, which DESIGN.md names as the one accent. So the field stops moving; the ground keeps a life of its own by a different means. **WHY THE COMPANION ROLE — and what was considered and rejected, so it is not re-litigated.** `organic_finds_rgb` draws three elements per cell, each with a seeded kind, radius and rotation: `kind_a` the ANCHOR, `kind_b` the COMPANION, `kind_c` the CUT-OUT. Selecting by **role** rather than by shape kind is the whole point: **every cell has exactly one companion, so the alive elements are evenly distributed BY CONSTRUCTION** — one per cell, no clumping — while remaining an authored grammar ("in Bowerbird, the companions breathe") rather than a random sprinkle. **Rejected: selection by SHAPE KIND** (the user's first sketch, "all triangles") — kinds are seeded per element, so they clump, leaving one region twitchy and another dead; and **triangles are the HIGHEST-salience of the three shapes** (sharp corners, strong directionality), so animating them stacks two attention-getters exactly where the caret must stay the accent. If shape-kind selection is ever revisited, it should be CIRCLES — make the alive thing the calm thing. **Rejected: the CUT-OUT role** — semantically the prettiest (negative space breathing) but they are the smallest elements and it risks falling below perceptible, which is the one thing this round must not do. **Viable fallback, not chosen: a seeded ~20% subset of any kind** — atmosphere rather than grammar; take it if the companion rule reads as too regular in the live sitting. **Build.** Delete the `drift` vec2 from `organic_rgb` outright — both terms, so the field no longer translates at all and `render/layers.rs`'s Organic arm stops computing one. The breathe is a **`mix` between two of the world's EXISTING three tones** — no new palette data, no per-world authoring, so any future Organic world inherits it free and "themes are data" stays intact. The envelope **reuses `stars.rs:185`'s shape** — `(rate * phase / LAVA_LOOP_CYCLES + offset).fract()` with an INTEGER rate and a **seeded per-element phase offset** so neighbours are desynchronised. Reusing that owner is what makes the integer-cycles law cover the new motion automatically, so the pop cannot return through a different variable. Amplitude is a FRACTION of the tone gap — taste-tunable, flagged for live review, and it must never read as a flash. **Scope:** Organic is Bowerbird's ground alone today. Bombora's waves, the stars and WarpedGrid stay untouched and **byte-identical** — this is not permission to retune any of them, and the shared clock keeps its current contract. **The law, owed regardless of the design:** delete `organic_phase_moves_and_wraps_without_a_catchup_jump` and replace it with one that sweeps **EVERY ambient consumer's drift-to-shader term across the wrap** — the axis the dead one missed — asserting continuity in the value the **SHADER** actually evaluates, never in a Rust helper the shader may not call. **Mutation proof: reintroduce a non-integer rate and watch it go red BY NAME**, and paste the panic. **Verify:** a capture pair at the phases either side of the wrap (`AWL_WAVES_PHASE` reaches a real mid-drift composition) proving the field **does not move at all** between them; a phase sweep showing the breathe is visible and that no two neighbouring companions are in phase; **byte-identity for the other 19 worlds**, PNG and sidecar; and a live `--release` sitting, because the pop was found by eye and its replacement can only be judged by eye. **Routing:** production tier, with the live sitting owned by the user. **User-reported and designed 2026-08-04; the arithmetic and the vacuous law were measured the same day.**

221. **Make Cassowary’s active Files category cue a vertical secondary heading.** **Defect:** the generic Files treatment does not use Cassowary’s left edge and strong Commands heading to establish its intended two-level hierarchy. **Build:** when Files is active in Cassowary, render “Files” as a smaller secondary-colour counterpart to the bold Commands heading, rotated 90 degrees and aligned flush with the far-left border; show none under All. Reuse the shared hierarchy data from item 220, with Cassowary’s expression supplied as theme data rather than a new palette code path. **Done:** Cassowary presents primary Commands plus a legible, subordinate vertical Files cue without crowding commands. **Verify:** Cassowary All/Files captures at representative canvas sizes and scale factors; geometry/contrast laws for left-edge placement, rotation, and non-overlap; mutation proof removes the cue; visual review confirms hierarchy. **Depends on item 220. Routing:** production tier with visual-judge review. **User design decision 2026-08-02.** 🟢 **UNBLOCKED — item 235 landed the capability (`df630ad9`), and 221 is now theme data on top of it.** **The 90° case costs nothing:** the rotation is a **lossless texel transpose** at the quadrant angles — ink 1.0000, mae 0.0000, contrast 1.0000 at 0/90/180/270°, **both DPIs**, pixel-exact against an ideal rigid rotation. So the "legible at 1× and 2×" clause is already measured for this item's exact angle; do not re-derive it. **Both premises the 220 lane corrected still apply here:** the shared datum is 220's single `overlay_location`, expressed as a `RenderCaps` variant (the same shape `Background`/`CardTexture`/`FacetStyle`/`TitleStyle::Placard` already have), and **the location inherits the section header's existing planned slot — no second header line is needed.** **The cue is not interactive** (it is read, not pressed), but `label_hit` exists and is law-tested if that ever changes; it derives from the run's own rotated frame rather than axis-aligned bounds, which over-claim at a slant and would steal presses from neighbouring rows.

224. **Redesign Magpie’s command-palette location indicator as a mirrored diagonal cue.** **Defect:** Magpie’s new right-side location indicator looks poor and does not belong to the world’s diagonal visual language. **Build:** prefer the indicator on the left. If a layout places it on the right, mirror its form. In either position, give it a slant and gradient matching Magpie’s diagonal line, while preserving legibility, palette hit targets, and command geometry. **Done:** the indicator clearly communicates location and reads as part of Magpie rather than a detached marker. **Verify:** Magpie captures across left/right layout conditions, canvas sizes, and scale factors; geometry laws prove mirroring and no overlap; gradient/angle law ties it to the diagonal line; mutation proof restores the unmirrored right-side form; visual-judge review. **Routing:** production tier with visual-judge review. **User design decision 2026-08-02.** ⚠️ **THIS ITEM'S PREMISE DOES NOT HOLD AS WRITTEN — measured by the 220 lane, and it shrinks the item substantially.** **After item 220, Magpie's cue is ALREADY on the LEFT**, riding the diagonal cluster's own row stagger. **There is no right-side indicator to mirror.** The "prefer the indicator on the left / if a layout places it on the right, mirror its form" clause is therefore already satisfied, and **the only things genuinely missing are the SLANT and the GRADIENT.** Build those; do not build a mirroring mechanism for a case that does not occur. 🟢 **UNBLOCKED — item 235 landed the capability (`df630ad9`).** **The gradient is already there:** 235 added a baseline gradient as a flagged scope addition — one instance field plus one `mix()` — specifically so this item would not have to. ⚠️ **Magpie's 77.66° is a DERIVATION, not a spec** — 235 computed it to have something to test; **this item's author picks the real number**, and the capability sweeps every angle regardless. Expect a resample at a non-quadrant angle: the round trip measures contrast 0.65 at 1× and 0.83 at 2×, with worst-case departure from an ideal bilinear rigid rotation of **0.0046** of full coverage — **that softening is the price of rotating a raster, not ink the pipeline loses**, and supersampling was deliberately declined because it would buy at 1× what 2× already gives while making letterforms stop matching their upright siblings. **Legibility here must be judged against that measured floor, not against an upright control.** ⚠️ **Do not confuse this with 222/131d's parked taste call** — right-aligning ascending worlds' name text is item 131d's, and it is adjacent to but not part of this cue.

226. **Prepare awl’s first GitHub Release around the existing Linux tarball.** **Build:** retain `awl-linux-x86_64.tar.gz` as the technical Linux download, make it discoverable from the release/download surfaces with concise unpack-and-run guidance, and attach a checksum manifest for every downloadable release artifact. Exercise the current release workflow as a dry run, including the release-profile parity gate and Linux packaging path; diagnose any publication, provenance, or archive-layout failure before a tag is considered. Configure the first public beta to publish **Linux only**; do not attach unsigned macOS or web artifacts. **Scope:** this is release preparation, not authority to tag or publish. A public tag/release remains an explicit user decision; macOS waits for Apple signing and notarization. **Done:** a dry run yields an inspectable Linux tarball and checksums, the release page has an unambiguous technical install path, and the release checklist names the still-required public-release and mac-signing decisions. **Verify:** unpack the produced archive in a clean Linux environment, run a headless smoke and launch check, verify checksums, and confirm the dry run created no tag or Release. **Routing:** production tier. **User design decision 2026-08-03.**

227. **Add a desktop-integrated AppImage as awl’s friendly Linux download.** **Defect:** the tarball is appropriate for technical early adopters but is not a normal Linux desktop application: it has no launcher metadata or icon integration. **Build:** package awl as an x86_64 AppImage in the release workflow, alongside—not instead of—the tarball. Include the binary, a `.desktop` launcher entry, the canonical Linux PNG icon derived from the existing icon pipeline, licenses/credits, and only the runtime libraries that belong inside the package; do not bundle GPU drivers. Publish a checksum and stable release-asset name. **Done:** a user can download one file from GitHub Releases, mark it executable, launch awl, and receive correct desktop name/icon integration where the desktop supports it; the tarball remains available as fallback. **Verify:** AppImage structural validation; launch and headless smoke on representative Debian/Ubuntu and Fedora-like environments; Wayland and X11 launch checks; icon/desktop-entry law; GPU-adapter and file-open smoke; mutation proof removes launcher/icon packaging; release dry run uploads both Linux artifacts. **Depends on item 226. Routing:** production tier with a Linux visual/compatibility audit. **User design decision 2026-08-03.**

228. **Version the first public beta as `v0.9.0`, then launch `v1.0.0`.** **Decision:** the first internet-facing Awl release is a public beta named **Awl 0.9.0 — Public Beta**. The GitHub Release is marked prerelease, but the version itself carries no `-beta` suffix. Patch releases (`v0.9.1`, etc.) are for launch-blocking fixes and polish; `v1.0.0` is the official launch once the core install-and-writing journey is ready. **Build:** update `Cargo.toml` and all version-bearing release surfaces together only when the release-preparation work is green and the user authorizes the public tag. **Done:** the package version, tag, GitHub Release title/status, downloadable artifact names, and release notes tell one coherent pre-1.0 story. **Verify:** release dry run names artifacts with `0.9.0`; version/source law finds no stale `0.1.0`; release checklist distinguishes the prerelease from the later `v1.0.0` launch. **User design decision 2026-08-03.**

245. **A CJK manuscript's READING TIME is now wrong in the other direction.** **Defect:** item 229 (landed `d8ae72c9`) fixed the count and the unit but left the pace at `ceil(count / 200)` for BOTH units, so a 5,500-character Japanese document reports **`5500 characters · 28 min`**. 200 units/minute is a WORDS-per-minute figure; published Japanese reading rates are roughly 400–600 characters/minute, so the readout is **two to three times too slow** for exactly the documents item 229 set out to serve. The number moved from meaningless (`1 word · 1 min`) to merely wrong, which is progress but is not done. ⚠️ **The item-229 owner declined to fix this deliberately and was RIGHT to** — the brief did not ask for a reading pace and inventing one silently would have buried a product decision inside an implementation round. It is recorded here because a declined scope question that goes unwritten is indistinguishable from an oversight. **Build:** the pace becomes a property of the unit, taken from the one owner `card::figures` that already returns `CountUnit` — not a second constant beside it, and not a per-world or per-language table. **Decide and pin:** the characters-per-minute figure, and whether a mixed document interpolates or takes its dominant script's pace outright (recommend: dominant script's pace outright, matching how the unit label already resolves, so one rule explains both). **Verify:** item 229's pinned regression table extended with expected minutes per row; the mixed-dominance edit case, since the pace must follow the label and must not flicker; the sidecar and drawn readout agreeing, through the one owner. **Routing:** production tier — but the chosen rate is a product call, so if the owner is not confident, park the number as a 🔵 and land the mechanism with today's value. **Found by the item-229 lane 2026-08-04 and flagged rather than absorbed.**

246. ✅ **RESOLVED 2026-08-04 by option (a), landed in `RELEASING.md` — accept the loss in writing.** The receipt section now states that neither mac job prints a `native-gate-receipt`, that `native-gate.sh`'s refusal to run filtered is exactly what makes its receipt mean anything, and that nothing was built to replace the signal — so no reader goes hunting for one that is gone. It directs the reader to the two jobs' own conclusions, which are individually meaningful, that being the point of the split. Option (b), a synthesised combined statement, was declined on the recorded ground that it would re-bundle what item 243 deliberately unbundled and would have to misstate scope to call itself a receipt. Implemented by the orchestrator directly rather than dispatched: the brief would have cost more than the change. **Original entry follows.** **No hosted-mac CI arm prints a `native-gate-receipt` anymore, and nothing replaced the signal.** **Defect:** item 243 (landed `1833757b`) split `mac (build + test)` into a gating filtered arm and a tolerated `render::tests` arm. `scripts/native-gate.sh` **refuses any filtered invocation by construction** — that is its contract and the reason its receipt means what it means — so neither new job can call it, and neither prints a receipt. **This is intended and is NOT a regression to undo**: a filtered run was never entitled to claim the full suite, and the item-243 lane both chose this deliberately and flagged it rather than absorbing it. **What was actually lost:** before the split, a human reading CI's mac job log could see `native-gate-receipt commit=… conventions=mac,linux scope=all-targets` and take it as informal confirmation that *that exact commit* passed the full suite **on hosted virtualised Metal** — the one axis no local gate reaches. That confirmation now exists in no form. ⚠️ **This is a DIFFERENT gap from the board's RECEIPT GAP**, which is about local pre-push receipts going unrecorded in commit messages; do not merge the two, they have different owners and different fixes. **Nothing currently consumes the CI string** — `scripts/test-native-gate.sh` tests the script itself and `code-health.py`'s audits check the script's shape and the `linux` job's use of it — so nothing is broken today. **Decide, then act:** either (a) accept the loss and say so in `RELEASING.md` where receipts are described, so no reader looks for a signal that is gone; or (b) have the two mac jobs jointly emit one honest combined statement naming what each half covered, which is **not** a `native-gate-receipt` and must not be spelled like one. **Recommend (a)** — the split's whole value is that each arm's conclusion is now individually meaningful, and a synthesised receipt re-bundles what was just deliberately unbundled. **Do not** relax `native-gate.sh`'s no-filtering contract to make a receipt reachable; that contract is load-bearing. **Found by the item-243 lane 2026-08-03/04.**

✅ **ITEM 249 IS UNBLOCKED — USER DECISION 2026-08-04: THE EVIDENCE BRANCH IS
ALLOWED, AND `CLAUDE.md` NOW SAYS SO.** The recommendation below was taken. A
branch may be pushed **solely to collect CI evidence on an arm no local gate can
reach**, opened as a PR for that purpose alone, never merged from, and **deleted
from the remote once the run is recorded** — the run and its logs outlive the
branch, which is what makes the deletion safe. The rule in `CLAUDE.md`
("Worktree branches never push") was absolute and is what actually blocked this;
a board note could not have cleared it, because a worker reads `CLAUDE.md` and
stops. ⚠️ **The exception is TEMPORARY and self-retiring — item 250 removes it**
once 249's evidence is in hand. Two mechanics for the lane: pushing the branch
alone runs **nothing** (`ci.yml`'s push trigger is `branches: [main]`), so the
PR is what fires the `linux` job; and `workflow_dispatch` is not a push-free
route, since it needs the ref on the remote too. **Original note follows.**

⚠️ **ITEM 249 WAS BLOCKED ON A RULE, NOT ON EFFORT.** 249 requires its oracle be
**proved on lavapipe before it lands**, since validating only on this host's
Metal is the exact gap that produced it. There are only two routes and **both were
closed:**
- **A local lavapipe container.** Item 232 measured this and refused it: a
  1.67 GB image and ~14 GiB of Docker VM disk that did not come back, for zero
  coverage of item 231's target axis. That refusal was about item 231's *hang*,
  not about allocation counting, so it is arguably not binding here — **but
  re-opening it is a decision, not an assumption a lane should make.**
- **CI on a branch.** `ci.yml`'s triggers are `push: branches: [main]`,
  **`pull_request`**, and `workflow_dispatch`. So a branch CAN reach CI's linux
  job by opening a PR — the item-243 lane's "pushing a worktree branch runs
  nothing" is true only of the `push` trigger and does **not** rule this out.
  ⚠️ **But it requires pushing a worktree branch, which `CLAUDE.md` forbids
  outright: "Worktree branches never push."** `workflow_dispatch` has the same
  problem — it needs the ref on the remote.

**Recommendation: allow a worktree branch to be pushed for the express purpose
of collecting CI evidence, on a branch that is never merged from.** It is the
cheap route, it uses the arm that already exists, and it is exactly how the
lavapipe axis becomes checkable before landing rather than after. The
alternative — keep the rule and pay for a container — costs disk that item 232
already measured as not returning. **Either way this is the user's call; 249
should not be dispatched until it is made, because a lane cannot honestly
satisfy its Verify clause otherwise.**

249. **Item 239's portable allocation oracle was NOT portable — redesign the oracle, keep the findings.** ⚠️ **This supersedes item 239's Build clause. 239's substantive findings are NOT in question and must not be re-derived; only the oracle is.** **Defect:** the oracle landed at `52b1b313` and was reverted at `b2f27143` because `main` went red — run `30844149209`, all three `alloc_bound_law` tests FAILED in the **linux** job under **both** conventions while every other arm passed. The law's own message named the cause: `creating one wgpu texture moved wgpu-hal's live texture count by 0 instead of 1 (before: -5 objects (buffers 3, textures -8, views 0); after: -4 ...)`. **Two separate facts in that line, and the second is the stranger one.** The delta is **0**, so on CI's backend creating a texture does not move the counter at all; and the absolute reading is **NEGATIVE** (`textures -8`), which no create/destroy accounting should ever produce. ⚠️ **The failure is precisely the axis the design chose itself on.** 239 picked counts over bytes because `buffers`/`textures`/`texture_views` are maintained by metal, vulkan, gles **and** dx12 while `buffer_memory`/`texture_memory` are vulkan and dx12 only — read off wgpu-hal 29.0.3 source. It was then validated only on this host's real Metal, and lavapipe falsified it on first contact. **A portable oracle that is not portable is worse than none, because the bound it guards reads as covered.** **Build:** decide what a cross-backend allocation oracle can actually assert, with **CI's lavapipe as a first-class target rather than an afterthought** — the negative absolute count needs explaining before any law is written on top of it, because a counter that can go negative cannot support a bound in either direction. Candidates worth weighing: read counters only as *deltas within one owned device* rather than as absolutes; use a wgpu-level rather than hal-level accounting point; or accept that the oracle is backend-specific and gate the law by backend, which is honest but forfeits the item's original purpose. **Verify:** whatever lands must be **proved on lavapipe before it lands** — a local container or CI-on-a-branch, not the dev host's Metal alone, since that is the exact gap that produced this item. Mutation proof as always, plus a non-vacuity arm equivalent to 239's third law (drop the `counters` feature and fail by name), which was well-designed and should survive the redesign. **Carry forward, all still true and none needing re-measurement:** the counter does **not** reproduce item 232's 199-vs-160 split (242.4 vs 243.3 objects per test, 0.4% against the container's 24%), so wgpu object allocation is not what the 4 GiB ceiling was exhausting; the suspects are pinned by `PendingWrites` until a **submit**, not by a poll, so a poll-targeted fix would not have worked and the live app which submits every frame is not exposed; and `background.wgsl`'s size ratio across the bisect boundary (1.2421) matches the container's test-count ratio (1.2437) to 0.13%, which one container run at HEAD would falsify. **The bound itself** — an empty submit plus a non-blocking poll in `test_gpu::arrive` — went out with the revert and is worth restoring **with** whatever law can honestly guard it. **Routing:** deep tier. **Found by CI 2026-08-04; the reverted work is in `52b1b313` for reference.**

250. **Restore the absolute "worktree branches never push" rule once item 249 has its lavapipe evidence.** **Defect:** by user decision 2026-08-04, `CLAUDE.md`'s push rule carries a temporary exception permitting an evidence-only branch, because item 249's oracle must be proved on an arm no local gate reaches and CI-on-a-branch was the cheap route. **An exception that outlives its reason becomes a standing loophole** — the rule's value is that it is absolute and needs no judgement call at 2am, and every reader after today will find a paragraph inviting them to weigh whether their push qualifies. **Build:** delete the exception paragraph from `CLAUDE.md`'s "Branches & pushing" section and restore the plain sentence, once 249 has landed with its evidence recorded. **Keep what was learned, in one sentence rather than a carve-out:** that CI's `linux` job is reachable from a branch only via `pull_request`, that the `push` trigger is `branches: [main]` and fires on nothing else, and that a run's logs outlive a deleted branch — those are facts about the repo worth keeping wherever the rule lives, and none of them require permission to be granted in advance. **Also confirm the evidence branch was actually deleted from the remote** (`git ls-remote --heads origin` names no leftover), since the deletion is half of what made the exception safe to grant. **Done:** `CLAUDE.md` states the rule absolutely again, no remote branch survives from 249's run, and the CI-reachability facts are recorded without a standing exemption. **Verify:** grep `CLAUDE.md` for the exception text and find nothing; `git ls-remote --heads origin` is clean of the evidence branch. **Depends on item 249. Routing:** orchestrator-direct — this is a documentation edit, and briefing it would cost more than doing it. **User decision 2026-08-04, made in the same breath as the exception itself, which is the right time to schedule an undo.**

251. **Item 207's AT-SPI journey needs a LINUX machine, and has been queued behind a macOS screen lock instead.** **Defect:** the board's live-closure list groups "207's real VoiceOver / AT-SPI journeys" under *needs an unlocked and FOREGROUNDED display*, alongside 118, 211, 218 and 244. **That is true of the VoiceOver half and false of the AT-SPI half.** AT-SPI2 is the **Linux** accessibility API (`ACCESSIBILITY.md:65` — NSAccessibility on macOS, AT-SPI2 on Linux, UIA on Windows), so no amount of unlocking the dev Mac reaches it; it needs a Linux desktop session with Orca. Filed as its own item because a blocker misattributed to the wrong cause **never gets cleared** — the arm will keep appearing in every "one unlock closes these" list, and each unlock will keep leaving it open, with nobody noticing that the list was wrong rather than the sitting. `ACCESSIBILITY.md:110` already states plainly that **no AT-SPI journey has been run at all**, so the honest limits section is correct today and must stay correct. **Build:** nothing to build until the hardware exists — this item's first job is to record what the journey requires (a Linux desktop session, Orca, the native build, and the same journeys the VoiceOver sitting runs: document read, caret and selection announcement, overlay summon/dismiss, and an editing burst that would surface the item-218 stall class on the other adapter). **Scope:** does NOT include shipping a fix for whatever it finds; a defect found here earns its own item, exactly as item 218 did from the first VoiceOver sitting. **Done:** either the journey has been run on a real Linux session and its findings recorded in `ACCESSIBILITY.md`'s honest-limits section, or the item stands parked with its hardware requirement stated — and in the meantime the board's live-closure list no longer implies an unlock will close it. **Verify:** human journey; there is no headless stand-in, and AccessKit law tests already cover the projection, which is precisely the layer this item exists to look past. **Routing:** human, on Linux. **Split out of item 207 on 2026-08-04 after the live-closure list was found to conflate two different blockers.**
