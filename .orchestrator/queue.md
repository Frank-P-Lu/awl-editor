# awl — live build queue

> Live execution state only — **open work, owed work, and what only the user can
> decide.** Nothing here describes something that already landed. Protocol,
> claiming, worktrees and execution hygiene live in `.orchestrator/README.md`.
>
> **TO RECOVER ANYTHING THIS FILE NO LONGER CARRIES:** every completion report,
> postmortem and closed decision through 2026-08-06 is in the board's own history
> at `git show 0dc30706:.orchestrator/queue.md`; `git log -p
> .orchestrator/queue.md` walks the rest, and `git log -S"<phrase>" --
> .orchestrator/queue.md` finds who removed a given line. ⚠️ **A compression once
> cleared an item's body while the item was still OPEN, and the reverse has also
> happened — summary lines survived a fix that had landed underneath them. Decide
> open-vs-done from the TREE and `git log --grep`, never from what this file says
> about itself.**

## ✅ CI RED — FIXED AND **CONFIRMED GREEN ON CI** 2026-08-07 (`196ad4ee`, merged `02d0ea23`; `linux (build + test)` **success** on `23783c12`). Kept because the LESSON is worth more than the fix.

**THE CAUSE: `menubar::MENU_BAR_ON` initialises to `false` on macOS and `true` on every other
platform.** The drawn bar takes **35.6px of vertical reserve** off every card's height budget
(`menubar_reserve()` → `card_y`). **Every render law in this repo has only ever run the macOS
half of that axis** — on the host that authors them — for its whole life.

⚠️ **AND `CLAUDE.md` LINE 64 ALREADY NAMED IT, INCLUDING THE REMEDY.** It documents
`menu_bar`'s default asymmetry and says *"reproduce that class locally by forcing the
initialiser's other branch"*. Doing exactly that produced **both failures with both messages
in 1.19 seconds.** The orchestrator's brief instead sent the lane after the shared GPU
adapter, `Bars`' outward-growing plate, and a cross-test global leak — **all three wrong, none
needing the adapter.** ‼ **The lesson for the next host-only failure: before hypothesising,
grep CLAUDE.md's tripwires for the platform axis. The answer was written down.**
⚠️ **And the orchestrator's own "4 rows vs 0" measurement was wrong for the same reason** — it
was taken with the menu bar HIDDEN. With it shown, fitting lines drop 9 → 7 and the grouped
family's `min_items: 0` takes the band to zero.

✅ **TWO REAL DEFECTS. THE FIRST WAS USER-FACING:** on Linux a picker could draw with **no
candidate rows at all** — `theme_overlay_geometry` charged a display row for every **section in
the list** rather than for the window it was about to draw, so at a trivially reachable 900×460
a 192px card showed zero items with 180px of canvas free below it. The second was the law
itself: the drawn-inset probe scanned a fraction of `card_h`, but `Bars` is `BarePlates` and
draws **no card plate**, so the slice reached past the last row into the world's own ground and
read a Firetail lava blob as a 1px inset.

🔵 **A THIRD DEFECT IS FOUND, IMPLEMENTED AND DELIBERATELY BACKED OUT — it is a TASTE CALL and
belongs to the user, not a CI lane.** `avail_px` charges the hint row and the blank separator a
full `lh` each and never credits `overlay_footer_reclaim`, which draws them compact — **65px at
zoom 3, a whole unspent row.** Crediting it changes shipped row counts on cards that already
fit, and collides with item 293's two-row contract and item 184's byte-identity guard. **The
question is how many rows a card should show; the arithmetic is ready either way.**

⚠️ **A FOURTH, WHICH THE CONTAINER FOUND AND CI NEVER SHOWED:** `hint_gap_item293`'s grouped
arm read `bare=2, hinted=0` on Linux — and **`2 - 0` is also `2`**, so it was **green because
its subject had collapsed.** Fixing the product turned it red. Every reading now carries a
presence floor. ‼ **This is the third law this week that was green over a collapsed subject.
A difference-of-two-readings needs a presence floor on EACH reading.**

**THE CONTAINER RECIPE — keep it; no local arm had ever seen this axis.** Ubuntu 24.04 +
`mesa-vulkan-drivers` + rustup, `CARGO_TARGET_DIR` on a volume. Two gotchas: **`-e
RUSTC_WRAPPER=` is required** (`.cargo/config.toml` pins sccache, absent in the image), and
**running it against a linked git WORKTREE fails two git-dependent laws** because the
worktree's `.git` is an absolute pointer into the host — use a plain checkout. Full recipe in
this commit's message and in the lane's report.


## 🔵 BLOCKED ON THE USER — nothing else can close these

⚠️ **This section has now been silently deleted TWICE** — once by an
orchestrator `git add -A` sweeping another tool's in-flight edit, once by a
worker's own commit despite its brief forbidding board writes. **After every
merge and every compression, verify this heading still exists.** If it is
missing, `git log -S"BLOCKED ON THE USER" -- .orchestrator/queue.md` finds who
took it.

1. **THE SITE IS STALE AGAINST THE PUBLISHED RELEASE — now ONE COMMAND, the
   user's to run:** `gh workflow run deploy-web.yml`.

   ⚠️ **RE-MEASURED 2026-08-06, and this item's own ORACLE WAS WRONG.** The host
   it named, `awl.computer`, is **NXDOMAIN** and appears **nowhere in the tree**
   except in this line — it was authored, not sourced. The live host is
   **`awl-editor.fly.dev`** (`site/fly.toml`: `app = "awl-editor"`). The
   *finding* reproduces exactly against the correct host, so the premise stands
   and only the instrument needed repair: `curl -s
   https://awl-editor.fly.dev/version.json` returns `{"version": "0.0.0",
   "prerelease": true}` — the "no tagged release yet" placeholder — while
   `v0.9.0` is public (`gh release list`, published 2026-08-06T02:04:33Z).

   ✅ **The secret half of this blocker is CLOSED.** `FLY_API_TOKEN` **is now
   configured** (`gh api …/actions/secrets` → `total_count: 1`, updated
   2026-08-06T13:44:31Z), and RELEASING.md §2 confirms it is the *only* secret
   `deploy-web.yml` checks (`.github/workflows/deploy-web.yml:43`). The board's
   "NO repository secrets are configured at all" is stale. Trigger is
   `workflow_dispatch` only — deliberate, never automatic.

   **The stale `tar xzf` snippet is the same deploy, not an edit.** The deployed
   landing page is 10445 B and carries **no version string at all**;
   `site/index.html` in the tree is 3749 B and already reads
   `tar xzf awl-0.9.0-linux-x86_64.tar.gz`. The two differ wholesale — the repo
   is the newer truth and the deploy is what carries it across.

   Once deployed, `version.json` picks up `0.9.0` with `prerelease: false` — the
   correct value, since that field means "a tag exists" rather than
   beta-vs-stable (item 228; `site/check.js`'s `checkState()`).
2. **KITE'S VEIL STRENGTH, and whether the crossing reads as intended.** One
   constant, `WARP_PAGE_VEIL = **0.13**` — ⚠️ **read out of the shader, not out
   of the lane's report, which said `0.20`. A figure owed to a taste call must be
   the one in the product.** Captures are in the lane's worktree at
   `gallery/item-268/` (`final/room/Kite.png`, `final/frame/Kite.png`, the
   two-tunnel `baseline/k-m66-d1.png`, rejected chrome variants in `chrome/`).
3. **WHICH WORLD ADOPTS `ListStyle::Rules` NEXT** — deliberately out of item
   283's scope. See item 283's handback below for what a second carrier needs
   first.
4. **ITEM 261's OPEN CALL: delete-outright vs a `cfg(test)` fixture.** The lane
   took the delete branch for `DeckleAnchor::Page`'s mutation witness and
   replaced the counterexample with a direct assertion that cannot go vacuous.
   **The user has not ruled between the two shapes. Reverting is re-adding one
   small shader function.**
5. **THE MULGA AND MAGPIE GROUNDS WANT A VISUAL JUDGE** — see the 🔵 OWED
   section; both are new grounds landed on arithmetic plus one taste call.
6. **KITE'S FACET TAG.** Its doc comment calls Kite a *"technical room"* but its
   only facet tag is `voice: Modern` — **Technical belongs to Cassowary.** The
   prose identity and the picker facet disagree. Changing it is a picker-facet
   decision (the bands are curated and capped), so it is the user's call.
7. **THE NARROW HISTORY COMPARISON STAGE DRAWS NO FOOTER** — `show_rows` false →
   `hint_rows` 0, so nothing teaches `tab back` / `esc close` at ~900×520 and
   below. A discoverability hole left deliberately when the workspace landed, and
   arguably a taste call about spending vertical space on a stage that has little.
   *(Its sibling gap — the narrow timeline column eliding mid-word on
   Mangrove/Magpie — is not a user call; it is owned by item 131e.)*
8. **The macOS release arm** — Apple signing secrets, per `RELEASING.md` §1.
9. **Further tags and the site deploy.** Both are the user's explicit word, every
   time.
10. **DOES THE DEBOUNCE RIP-OUT FEEL RIGHT? Item 290 is LANDED (`51302d50`) and
   this is the confirmation it owes — a look-and-agree, not a defect.** The board
   asked the lane to report back if removal regressed materially. It did, by
   **two orders of magnitude more than the board's own figure**, so the number is
   here rather than buried in a merge message.

   | 9-step burst, release | before | after |
   |---|---|---|
   | CLAUDE.md (119 lines) | 74.6 ms, **1** reshape | 297.9 ms, **9** reshapes |
   | 1896-line fixture | 82.9 ms, **1** reshape | 380.8 ms, **9** reshapes |

   Reshape counts are witnessed, not inferred. ⚠️ **The board's old "12.3 ms /
   n=1 for 8 inputs" carried item 291's bias signature verbatim** — 291 says a
   burst reports `n=1` for 8 inputs because `probe::mark_movement_input`
   overwrites a pending mark. `--bench-theme-burst` never touches that probe, so
   the figures above are clean and the old one was measuring the instrument.

   **Why it was landed rather than held:** per-step cost is **unchanged** at
   ~20–30 ms. The debounce never made a reshape cheaper — it **skipped** it, so
   eight of nine steps showed the WRONG FONT and the ninth paid a 141–151 ms
   stall. The 4× is the cost of actually doing the work on every step, spread
   evenly at ~33 ms (about two frames) instead of hidden behind a bimodal stall
   whose boundary sat inside human key cadence. That is the trade the item was
   opened to make, and PHILOSOPHY's calm bias prefers the even cost.

   **The question, and it is only answerable live:** arrow quickly through the
   faceted theme picker. Does each step feel like a small honest hitch with the
   right font, or does the burst feel worse than the old "colours move, fonts
   lag, then a freeze"? **Recommendation: keep it.** If it feels wrong the answer
   is one command — `git revert 51302d50` — not a rebuild, and the alternative of
   raising the window is already refuted (300 ms → 348 ms settle, 400 ms →
   459 ms).

✅ **CLOSED HERE 2026-08-06 — item 118's direction call. THE TARGET SHAPE IS
DROPPED.** The user's words: *"i think we just drop the target. it's fine, right
now."* The roster's measured mean of **2.20** is accepted as awl's shape rather
than a shortfall against the aspirational 2.90, consistent with PHILOSOPHY's
calm bias. **`1, 7, 6, 4, 2` / mean 2.90 is retired**; it is not amended, not
restated, and not replaced by a descriptive one. Recorded in item 118's body as
well as here — **this item has already been answered twice by the user because a
decision recorded in one place was invisible in the place it gets read.**

## 🔵 OWED — live work that nothing above implies. Never cleared by a compression.
- 🔵 **`--help` REALIGNED BY 7 WHITESPACE-ONLY LINES (item 273 residual 1) — kept, and one
  function to revert.** Generating `--help` from the roster meant picking one padding rule. Today's
  column is **eyeballed per line** (22 with six exceptions, 40 with two); the rule now is "pad to
  the column, else two spaces". **Order, grouping, wording and the documented flag set are
  unchanged** — the only differences are 7 lines' column positions. I kept it because a per-line
  fudge table is data existing solely to reproduce an inconsistency, but **it is your text and it is
  one `line()` function to put back.**
  ⚠️ **One line changed in FACT, not whitespace, and that one I would not revert:** `--measure` said
  *"default 80"* and the value is **70 for prose, 100 for code**.
- 🔵 **`Section::Cli`'s PLACEMENT IN THE REFERENCE IS A TASTE CALL (item 273 residual 1).**
  "Command line" is appended **last, after Markdown**, with three sub-tables — Capture modes (7),
  Options (31), Unlisted flags (23). The section is cheap to move and the "Unlisted flags" caption is
  equally open. Rendered and clicked at `#command-line`: 20 nav links, **0 dangling**, `<h2>` styles
  identically to its siblings, no horizontal overflow. `gallery/item-273r1/ref-cli.png`.
- 🔵 **THE EXPORT SAVE PANEL WANTS YOUR EYE ON macOS (item 301) — no test process can see it.**
  An AppKit modal is unobservable from a test (`MainThreadMarker::new()` returns `None` off the
  main thread), so the panel body is structural-by-construction and what IS tested is only that the
  door never opens without a real surface. **Three questions only you can answer:** does it open at
  the right folder, is the right name pre-filled, and does **Cancel** leave the document untouched?
  Try `Export as PDF…` from the File menu. ⚠️ **The Linux arm is separate and also unreached** —
  the in-app `ExportDest` card is the whole answer there and it is fully tested, but no macOS host
  clicks a drawn Linux menu bar.
- 🔵 **THE MENU BAR'S PADS DOUBLE ON A RETINA DISPLAY (item 323) — a deliberate appearance
  change I could not put to anyone, and no capture at `--capture-dpi 1` can show it.**
  `BAR_INSET_X` and `TITLE_PAD_X` going `Logical` means the bar's left inset and its two outer
  title bands grow at DPI > 1; `DROP_PAD_X`/`DROP_PAD_Y` do the same to the dropdown card. **1× is
  byte-identical.** The authored numbers are simply being honoured now — nothing was re-tuned —
  but the >1× look has never been seen by a human. ⚠️ **The drawn bar is the default OFF macOS**,
  so this wants a **Retina Linux or web** session, or `AWL_MENU_BAR_FORCE=on` locally. And the
  **macOS dropdown is live-only**: no local run opens it on a real `NSMenu` path.
- 🔵 **318's SHAPE IS THE ONE TO LOOK AT FIRST — it is your own call, delivered.**
  `gallery/item-318/before-Mangrove.png` → `after-Mangrove.png`, the world you photographed.
  Before, the blurred patch has vertical left and right edges; after, `ver. The`, `, while`, `is`
  go sharp at the top right and `words`, `in shape`, `beside` go sharp at the bottom left — **both
  raking edges translate with the spine.** `before-Magpie.png` → `after-Magpie.png` is the mirror.
  Pixel-verified: the difference is **exactly two triangles**, widening monotonically away from the
  card's vertical centre and mirrored between rake directions (Mangrove 31787px, Magpie 30436px).
  ⚠️ **ONE QUESTION LEFT, AND NOTHING WAITS ON IT: does a mirrored composition's QUERY FIELD mirror
  like its rows?** This is 313's terminus question one band over. The shape is a parallelogram on
  all three worlds either way. **The cost if you say yes:** Magpie's frost box currently extends
  **43.15 logical px past its card** on one face purely to seat the upright field — that widening
  disappears if the field mirrors. **The cost of the mechanism that would do it** is the one the
  lane refused unbriefed: the field moves +461px and right-aligns, so its `›` sigil would travel
  leftward as you type. **If you want the field mirrored, it needs its own design, not 313's.**
- 🔵 **MULGA WANTS A NUMBER FROM YOU (item 118).** Its `1/5` was scored against a ground that
  no longer exists — item 258 replaced `Starfield` with `Pinstripe`. Re-measured, deliberately not
  re-scored: it now sits **above every current 1/5 world** on two contrast columns and is
  **statistically identical to Cassowary (4/5)** on two others, while reading gentler than
  Cassowary does (a third its lightness swing, and no accent-coloured ink). **A fresh `1` does not
  match the arithmetic; `2`–`4` fits and the number is yours.**
  `gallery/item-118/mulga-neighbourhood/` puts it beside every `1/5` world plus Cassowary and
  Saltpan at one arm.
- 🔵 **GALAH IS ALREADY A TINNY BIT LOUDER (item 118) — shipped, not proposed.** `0.10 → 0.12`,
  the smallest step in the band you pinned that a capture can actually tell apart (`0.11` sits
  inside 8-bit quantization noise). **Worth a live `--theme Galah` glance** to confirm it reads as
  "a tinny bit" and not as a rebalance. `gallery/item-118/galah-density/`.
- 🔵 **THE NEAR-PAIR MERGE CALL IS YOURS (item 118), and one of its premises just died.**
  ROADMAP asks to merge the tightest near-pair; item 118 declined to execute it, because merging a
  world removes something a user may have chosen. ⚠️ **The Magpie/Saltpan pairing that call named
  is stale** — item 260 moved Magpie onto `Bands` the same day 258 moved Mulga. The genuinely
  tightest surviving pair is **Tawny/Mopoke** (same `Dots{edge:false}`, edge 0.0000 both, L* σ
  within 0.15). **Bilby/Brolga is a deliberate mirror per THEMES.md and is not a duplicate.**

- 🔵 **321 CHANGES THE MENU BAR ON EVERY NON-macOS HOST — the authored value, finally honoured.**
  The drawn bar (the default off macOS, and on the web) now holds a constant **35.6 logical px**
  at every zoom and DPI instead of visibly thinning as density rises: **+5 logical px at 2×, +6.7
  at 3×.** The card height budget below it shrinks by the same amount — **a few px, not the
  CI-RED's zero-row magnitude.** ⚠️ **At 1× and on macOS's default-off bar, byte-identical**, so
  no capture here can show it. Worth a look on a Retina Linux or web session.
- 🔵 **313's TERMINUS CALL — real but SMALL, and the measurement is the point.** At the hint's
  own row the spine has ENDED, so does the hint continue the lean past the terminus or sit at
  the terminal x? `gallery/item-313/continue-*.png` vs `terminus-*.png`. **Measured: the two
  answers are PIXEL-IDENTICAL on Mangrove** (the column clamp binds first there) **and 9px apart
  on Magpie.** `continue` is implemented, behind a one-word switch the laws *read* rather than
  restate, so flipping it is one token. Reasoning for `continue`: the rake is a property of the
  card, not a rule that stops at the last name — and at the terminal x the hint's left edge lands
  exactly on the last row's, reading as one more list row, which is the reading the separator
  exists to break.
- 🔵 **297's FOUR CALLS, and #1 is the genuinely debatable one.** Captures in
  `gallery/item-297/`.
  1. ⚠️ **PARK versus SHRINK.** Past **1.74× zoom on the widest card the cue DISAPPEARS**
     rather than shrinking — `case-zoom1.7-drawn.png` → `case-zoom1.75-parked.png`. That is
     deliberate: the size IS the composition, so a cue at some other fraction would be the
     small misplaced whisper this item removed, and the lens strip still shows `[Navigate]`
     in brackets. **But "gone" versus "smaller" is taste, and it is yours.**
  2. **The before/after on one frame:** `before-Cassowary-Files.png` →
     `after-Cassowary-Files.png`. A 15px muted "Files" at the card's border becomes a 505px
     run rising from just above `COMMANDS`, same margin, same ink. Longest case:
     `after-Cassowary-longest-This-folder.png`. Both tiers:
     `case-retina2x-2400x1600.png`.
  3. **Two loud marks now share the left margin** — the cue takes the wordmark's *full* ink,
     so hierarchy is by size alone with no value step. Deliberate; worth a look.
  4. **The gap** is `0.12 em` of the cue's own type over the placard's leading (≈36px of
     daylight at the reference canvas). One constant either way.
  ✅ `before-Magpie-Raked-unchanged.png` / `after-…` is a byte-identical pair — the receipt
  that refactoring the shared rotation path moved nothing on the other carrier.
- 🔵 **314 MOVED A VISIBLE LENGTH ON RETINA, and it is the intended value rather than a
  tuning choice.** The page's collapsed side pad and the outline rail's inset now sit at
  **16 logical px where they sat at 8** — doubled in physical terms on a 2× display, because
  they were being read unscaled. Nothing was re-tuned; the authored number is simply being
  honoured now. **Worth your eye on a Retina screen**, since no capture at `--capture-dpi 1`
  can show it and dpi-1 output is byte-identical (19/19 captures).
- 🔵 **312's TWO TASTE CALLS — the feather WIDTH and the LEAN, and no measurement settles
  either.** Captures in `gallery/item-312/`.
  1. ⚠️ **Is the defect gone?** Open `before-hard-edge-Mangrove.png` beside
     `after-Mangrove.png` — the same frame, shipped. Before, words break clean at the
     boundary (`the|` … `ver.`); after, they dissolve and the patch's silhouette rakes with
     the spine. **Orchestrator's own eye: yes, and the lean reads as intentional.**
  2. **The width.** `feather-14px-Mangrove.png` → `after-Mangrove.png` (28, shipped) →
     `feather-42px-Mangrove.png`. 14 sits at the law's arithmetic floor (a feather narrower
     than the blur's own 16 logical px reach reads hard); 42 pushes the skirt further onto
     the live page the picker exists to preview.
  3. **The lean.** `after-Mangrove.png` / `after-Magpie.png` (both directions) against
     `lean-off-feather-only-*.png` (upright box). This is also the honest way to judge the
     union's silhouette — **whether the leaning ears past the box read as intentional or as
     a smear.** `after-Paperbark.png` is the `Rules` arm: feathered and upright, which is
     what the roster-derived split produces without Paperbark being named anywhere.
- 🔵 **294's THREE TASTE CALLS, and one is a change to EVERY world at 2×.** Captures at
  `gallery/item-294/` (46 shots, copied out of `/tmp`).
  1. ⚠️ **THE DPI FIX CHANGES EVERY WORLD'S FULL-TAKEOVER FROST ON A RETINA DISPLAY, and
     no capture could ever have shown it.** The Gaussian's reach was a fixed count of
     quarter-res texels, so it was constant in DEVICE pixels — **the defocus a reader
     perceives was HALF strength at 2×.** Fixed by multiplying the authored logical reach
     by DPI once. `dpi 1` is byte-identical, so the suite cannot see it; at 2× it moves
     **224 249 of 960 000 pixels** on Gumtree's palette, toward the frost that overlay
     already showed at 1×. Compare `b2-Gumtree-palette.png` (before, backdrop words still
     readable) with `r2-Gumtree-palette.png` (after). **Isolatable to one function if you
     want it reverted separately from the footprint work.**
  2. **The footprint's rectangle has HARD EDGES that cut words mid-glyph** at the card's
     boundary — inherent to scoping, and it reads as a pane of frosted glass.
     Orchestrator's own eye on `caret-under.png`: clearly visible, and a legitimate design
     read rather than an artefact, but yours to accept.
  3. **A blurred spell squiggle becomes a soft red/pink band** inside the frost.
  ✅ **The caret card over its own caret reads WELL** — `caret-under.png`, Mangrove, caret
  inside the footprint: it survives as a soft orange glow under the rows rather than a
  competing mark, so the anchoring question the item raised needs no change.
- 🔵 **SHOULD EXPORTING BRING THE FINDER FORWARD AT ALL?** Item 301 wired
  `NSWorkspace activateFileViewerSelectingURLs:` after a successful export, gated on a
  real surface. **The lane flagged the product question rather than deciding it: the
  reveal TAKES FOCUS from the editor**, which is DESIGN's no-nagging-chrome boundary,
  and only a live look settles it. **If the answer is no, the honest alternative is a
  palette row, not an automatic reveal.** Live-only — no capture can photograph it, and
  none is claimed.
- 🔵 **DOES THE REFERENCE BELONG IN THE HELP MENU?** 273's residual (4) added a
  palette command ("Reference", no chord) and its lane **deliberately did not** add it
  to `src/menu.rs`'s `HELP_ITEMS` — the native macOS Help menu, also consumed by the
  drawn Linux menu bar — which today lists Guide and Credits beside Check for Updates
  and Report a Problem. That list is a **hand-curated subset with no coverage law**
  forcing every catalog command in (confirmed by the lane: no test requires it), so
  nothing is broken either way. **The question is taste: does a big cold reference
  table belong in a four-item menu next to a tutorial and a licence doc?** The lane
  was right not to decide it in a merge. One line either way.
- 🔵 **296/300's LOOK IS THE USER'S TO ACCEPT — four calls, none of them correctness.**
  Captures at `gallery/item-296-300/` (copied out of `/tmp`): `USER-toast.png` (a
  `saved` toast on a full page), `USER-sticky.png` (the Export refusal, held), and
  `final.png` (a three-world strip). The notice is now one plated LABEL line at the
  **top of the writing column**.
  1. **Top-of-column placement.** It clears the H1 in these shots but **will cover the
     first prose line on a document without a heading** — the unavoidable cost of
     putting it in the reading path when the margins are ~16px.
  2. **A square-cornered chip**, matching `menubar_bg` and DESIGN §1's Swiss
     vocabulary. A small radius is one constant away.
  3. **The kind ladder** — toast on `base_200`/`muted`, sticky on
     `base_300`/`base_content`, with a one-bit world inverting through
     `HighlightTreatment`. Whether a held refusal should be *this* much louder than an
     acknowledgement is theirs.
  4. **Plate padding** derived from the LABEL line height (`0.6×`/`0.22×`), so it holds
     at every zoom and DPI.
  **Live-only, flagged rather than claimed:** the 2500 ms toast lifetime only *feels*
  right in `--release` on a real window.
- 🔵 **131e/303's TWO TASTE CALLS, and the second is the interesting one.** Captures at
  `gallery/item-131e-303/` (gitignored, **copied out of `/tmp` deliberately** — the board
  already lost one lane's gallery that way). Show `before/Magpie.png` beside
  `Magpie-cmd.png`; the `*-mark-zoom.png` pair are 8x crops for the register comparison.
  1. **Magpie's hairline weight is `1.25`** (`reach 4.5`, `aperture 0.55`) — read out of
     `src/theme/diagonal.rs`, not out of a report. If it now reads too FAINT rather than
     too heavy, `weight` is the one dial and it moves without touching Mangrove. Worth a
     second look at Mangrove's `aperture: 1.0` too, now that there is something to
     compare it against.
  2. ⚠️ **THE MARK'S DISTANCE FROM ITS ROW'S TEXT IS A TRADEOFF, NOT A BUG.** It hangs
     off the cluster's BUDGET outer end rather than the accessory column's measured ink
     — correct under 131's own fixed-surface rule, since a content-derived anchor would
     make the mark jump on every filter and scroll. But on the many rows carrying no
     chord the mark sits alone in the outer margin: verified by eye at ~460px from its
     label on the Command palette. Findable (5/5 vision smoke) but a sweep of the eye
     rather than a glance. **The alternative breaks the fixed-surface rule, so this is a
     choose-your-tradeoff question.**
  3. The mark now SNAPS between rows with no glide — a deliberate no-op, see 303.
- ✅ **258 AND 260 ARE JUDGED AND CLOSED (2026-08-06).** Fresh captures at
  `gallery/owed-258-260/` (gitignored), judged at the visual-judge tier. **Verdicts:
  KEEP Mulga's `Pinstripe` — the `Gradient` fallback is DECLINED — and KEEP Magpie's
  `Bands` as rendered.**

  ⚠️ **The first pass was judged on a 16px margin SLIVER and said so**, naming a
  wide-margin capture as the one configuration that could overturn it — the default
  1200×800 capture puts the page nearly edge-to-edge, so the ground is barely
  present. Re-captured at 2000×900 with a 52-char column (~620px of ground per
  flank) and re-judged; **all three verdicts held under the harder geometry**, with
  continuity measured rather than assumed (same 9px period, same tones, so it is the
  same ground seen properly, not a retuning). **A ground question asked of an
  edge-to-edge capture is asked of almost no ground — set the geometry first.**

  **Why Pinstripe holds:** the shirting read requires crisp DARK rules on a light
  ground; Mulga is the opposite polarity at a quarter the contrast (ΔL 0.027), so ~70
  repeats per flank assemble as ribbed dark cloth around a lit page — buckram binding,
  not wardrobe. **Separation survives field scale** because the three carriers differ
  in MATERIAL, not tint: Mulga soft tone-on-tone ribbing, Saltpan polarity-flipped
  crisp tan-on-cream at ΔL 0.120 (laid paper), Cassowary phosphor-green hairlines on
  blue-black (scanline glass).

  🔵 **RECORDED AS A TRIGGER, NOT A TASK:** Mulga and Cassowary share the
  light-line-on-dark arm, and their separation rests on hue and room value rather
  than structure. **If a FOURTH `Pinstripe` carrier lands, or a picker audition ever
  reads Mulga↔Cassowary as a repaint, a Zigzag-style pitch/weight dial becomes owed —
  on Mulga first**, as the least structurally distinctive carrier. Nothing is owed
  today.

  **Why Bands holds:** its pitch is wider than the visible field, so even 2000px shows
  two soft ~25° boundaries and reads as overlapping paper planes, not stripes. ⚠️ **The
  angle is AUTHORED** — `Background::Bands { tones, angle }` — so "horizontal bands" in
  any earlier description was wrong, not the render. Subordination is not close: the
  ground's whole range is Δ25 gray, soft-edged, margins only.

  **Still live-only for both:** feel in motion and scroll. Nothing else is owed.
- **211 — an unoccluded LIVE GLIDE CONFIRMATION. ⚠️ THIS IS A LOOK-AND-AGREE,
  NOT A DEFECT: the every-other-input picker selection is FIXED** (`237f97d7`,
  merged `50d6b532`), and the board's old "thrice-reported picker defect" prose
  is stale and has cost a wrong call. **The break was a redraw-scheduling gap:**
  `App::on_redraw_requested` read `TextPipeline::advance` **before**
  `Gpu::redraw`, and the band's retarget happens inside `prepare`
  (`chase_or_snap`) — the only animator whose target is set at draw time — so on
  the frame a settled band was retargeted the pre-prepare answer was "nothing
  animating", the loop parked on Wait, the ease never got its second frame, and
  the next input's single dt drove `chase_or_snap`'s SNAP branch two rows.
  `chase_or_snap` now reports the re-zero and the loop reads it straight after
  `Gpu::redraw` returns; off-window the new term is a structural false, so no
  capture moved. **What is owed is only the photograph:** the display locked
  itself seven minutes into the sitting, so every present in the trace reads
  Occluded. **The state chain is CPU-side and occlusion-independent, so the
  diagnosis holds — but no frame was photographed and no video exists.** The
  instrument is `scripts/`'s live band sweep (`54c027e1`, `52885c6e`), written so
  it cannot look like success while photographing nothing.
- **284 — the live glide's feel**, and `MARKER_TRAVEL_TILT_DEG = 20°` —
  production-tier picks, not a taste-round decision the way the chevron's shape
  was. **And an honest gap: a wrap settles in the correct direction but its
  transient glide looks identical to an ordinary step**; whether a wrap deserves
  a distinct flourish is a live judgement.
- **242 — the formal affordance-locating vision smoke** over ~5 gallery shots.
  The lane did an eyes-on retina pass and reported it as such; the standing
  policy asks for the structured version. **Also named as a deliberate
  residual:** the declaration law covers authored `const`s and **not inline
  literals** — seven chrome pixel lengths remain physical inline
  (`gutter.rs:262,321,355,404`, `outline.rs:221,796`, `diagonal.rs:460-461`), all
  in margin chrome, none in the summoned-overlay families it measured.
  ✅ **CLOSED 2026-08-06 — THE RESIDUAL IS EMPTY, AND ONLY FIVE OF THE SEVEN WERE
  REAL.** `outline.rs:796` had drifted into a `#[cfg(test)]` module (a test FIXTURE
  literal, never shipped chrome — established by reading the whole non-test portion),
  and `diagonal.rs:460-461` was already resolved by items 303/131e. The five real ones
  — gutter's four bottom-anchored rects and outline's reserve band — were **merged into
  the existing `readout::CANVAS_INSET` rather than declared as five new constants**,
  because every one already claimed textual identity with it in a comment that nothing
  enforced. `Physical`, not promoted: usage was read rather than preferred, and every
  site compares against the device-pixel canvas dimension. **`CANVAS_INSET` is now the
  one owner across six call sites, so a promotion sweep moves all of them at once.**
  Proven by changing the constant and watching the captured PNG's hash move — which
  tests the WIRING, something a declaration law cannot do.
  ⚠️ **THE EIGHTH IS WHY THIS RESIDUAL EXISTED, AND THE LESSON OUTLIVES IT.** `readout.rs`'s `CANVAS_INSET` was a bare `8.0` repeated per anchor
  arm — invisible to this law for its whole life, because the law reads `const`s and
  those were literals. Item 296's lane merely NAMED it, and that alone made the law
  fire. It is declared `Physical` with its reason, recording what it already was.
  **Promoting it to `Logical` doubles the inset on Retina, which is almost certainly
  correct**, and it owes a 1×/2× sweep across every anchor arm plus the notice plate's
  clamp — a deliberate appearance change, not a merge fixup. The lesson for whoever
  closes this: **naming a literal is how you find these, so the cheapest way to shrink
  the residual is to name the remaining seven and let the law tell you.**
- **241 — the user's own window.** Every live number came from a 900×600 probe
  window; the 4530×2756 @2x window will show larger `atlas`/`acquire`. The
  mechanism is window-independent; the absolute after-numbers on that machine are
  unmeasured. Also untested live: a dense pointer/wheel sweep, which shares
  `retint_theme_preview`, so the rule applies but the cadence is unproven.
- **249 — a stated cost, measured not argued.** Nothing in `PendingWrites` pins a
  *view*, so the portable unit sees the buffer half of the pin and not the
  texture half: **a leak pinning only textures would be invisible.** That is the
  price of a unit that means the same thing on a backend whose texture counter
  runs backwards.
- **245 — one constant**, 200 wpm, the round conventional English figure.
- **263 — one check the lane deliberately DEFERRED rather than ran badly:** the
  construction-site document-seed mutation, held back to avoid contending with a
  running gate. It follows from the sync mutation but is inferred rather than
  measured. **Worth closing.**
- **273 — the site page is visually unreviewed.** Links pass and the CSS reuses
  `.credits-body`, but `site/reference.html` was never rendered, and the lane
  flagged that rather than claiming it.
- **271/283 — the graduated `Rules` style ships on ONE carrier world.** The
  second-carrier requirements are recorded in `theme/tests/personality.rs` beside
  Paperbark's entry (so the next author reads them there rather than here) and
  are summarised under item 283 below.

## Remaining work — handoff order (RE-DERIVED 2026-08-06, against the tree)

⚠️ **This section has gone stale three times, each time by editing the previous
list instead of re-checking the tree.** The rule, restated as an instruction:
**grep the tree for the thing the item promised.** Verified that way just now:

| claim | measured |
|---|---|
| `ListStyle` arms | **4** — `Pane`, `Diagonal(DiagonalDirection)`, `Bars`, `Rules(RuleSelection)` (`theme/model.rs`) |
| `Tunnel` arms | still **4** — `Fixed`, `PageScaled`, `MarginPlaced`, `Reversed` (`theme/ground.rs`), item 194's mutation arms intact |
| `Arrangement` / `LavaEdge` / `DeckleAnchor` | gone as columns; **4 surviving references, all prose or negative-assertion needles** |
| `Starfield` / `worlds_gallery` / `CASSOWARY_LIGHT` | **0 references** in `src/` and `shaders/` |
| `selection_document` / `selection_ui` | both exist (`theme/model.rs:538`, `:545`) |
| `POSTER_BARS` | **0 references** — the dials collapsed onto `BarConfig::SHIPPED` |
| `worker-build.sh` test budget | present — exports `CARGO_BUILD_JOBS` **and** `RUST_TEST_THREADS` |
| the repo rename | `git remote` is `awl-editor`; `src/repo_url_law.rs` bans the old repository reference (not the bare token) |
| `REFERENCE.md` + `site/reference.html` | both exist |
| test monoliths | `theme/tests` and `main/tests` are dirs; **`overlay/tests.rs` 3433, `app_icon/tests.rs` 2368, `buffer/tests.rs` 2241 remain** |
| `App` root | 107 fields → **20**; every owner in `docs/app-domains.md` reads "extracted" |
| `src/render/plan/` | overlay row family only (5 modules) |
| item 288's three identifiers | all three still present, verbatim |

✅ **BOTH ITEMS THAT WERE BLOCKED ALL OF 2026-08-06 ARE NOW SATISFIED (2026-08-07).**
- **CI on `main` is GREEN** — `completed/success` at `68953a89` and `31a3fafd`, read as the
  last SUCCESSFUL shas rather than the last runs.
- **Item 227's AppImage release dry run PASSED on real CI** (run `31134001680`), which is
  the one thing no worktree branch could ever prove, because a branch cannot push. Evidence
  read out of the run: `linux (release tar.gz)` succeeded having run **both** packaging
  steps; the structural law fired on the real artifact
  (`appdir OK: AppRun, dev.franklu.awl.desktop (+ Name/Exec/Icon/Type), dev.franklu.awl.png
  (root + hicolor), licences`); `appimagetool`'s pinned sha256 was **verified before use**;
  and `awl-…-linux-x86_64.AppImage` was cut and uploaded beside the tarball in one
  `awl-linux` artifact. ⚠️ **`publish GitHub Release` was SKIPPED, exactly as
  `RELEASING.md` documents** — so that job remains permanently unexercised until a real
  tag, and it still ships straight to the public.

**Historical:** Actions was in a major outage all of 2026-08-06. CI is running on `main`, and the **AppImage release dry run —
item 227's one outstanding verification, which no worktree branch could ever run — is
in flight** (`gh workflow run release.yml -f dry_run=true`, run `31134001680`; `plan`
already green, the `linux (release tar.gz)` job building). ⚠️ **Read its result before
believing item 227 is verified**, and remember `cancelled` is not a pass — check for the
last SUCCESSFUL sha, not the last run, and classify a red by its failed STEP names.

**Historical, kept because it explains the day's shape:** `main` spent 2026-08-06
locally receipted and remotely unverified through an Actions outage. Pushed deliberately: 15
commits on one local disk is the larger risk, the batch carries a full receipt on its
exact sha, a push cannot supersede a run that cannot start, and no tag is involved.
**OWED THE MOMENT ACTIONS RECOVERS:** re-run CI on `main` and read it — `cancelled`
is not a pass. Two axes stay uncovered until then: **CI's `linux` job is the only
real-Linux coverage** and **the hosted-mac jobs the only virtualised-GPU coverage**,
and a local receipt certifies neither. Also owed: `gh workflow run release.yml
-f dry_run=true` for item 227's AppImage, which no worktree branch could run.

**TEN ITEMS LANDED TODAY, NO LANE RUNNING.** Wave 1: 288, 295, 290, 304, 289, 291,
274 (+172 closed against its census). Wave 2: 227, 292, 305. Plus **258 and 260
judged and closed** in the OWED section.

⚠️ **SIX FALSE OR STALE PREMISES IN ONE DAY, EVERY ONE AUTHORED ON THIS BOARD OR BY
THE ORCHESTRATOR, AND EVERY ONE CAUGHT BY MEASUREMENT RATHER THAN BY REVIEW.** This
is the day's real finding, and it is a process fact, not a run of bad luck:
- **304** — "renders as NOTHING" had already been fixed a day *before* the item was
  written (`77db975e`). The report likely came from a binary predating the fix.
- **290** — "12.3 ms / n=1 for 8 inputs" *was* item 291's own documented instrument
  bias. A figure the board already knew to be untrustworthy became a premise.
- **289** — "Wagtail's `Underline` chips": Wagtail is `FacetStyle::Text`, **Magpie**
  is the sole carrier.
- **292** — the item called Kite a `Chips` world; Kite is `FacetStyle::Band`.
- **the orchestrator's own, twice** — "raise the file-size mark with a reason" is
  not generally possible (the ceiling is `min(old_size, mark)` against the frozen
  baseline, so a mark can only TIGHTEN); and a judge's brief said Magpie's `Bands`
  were horizontal when `Background::Bands` carries an authored `angle`.

**The rules this earns, and they belong in every brief:** state the premise-check
first; **quote a board figure with its instrument named**; derive an enrolment from
the roster rather than a name list; and **reproduce a user-reported defect against
HEAD before briefing it**, because the reporter's build may predate a fix.

⚠️ **THE ORDER BELOW IS STALE AS OF 2026-08-07 AND IS KEPT ONLY FOR ITS REASONING
ABOUT CONTENTION.** Checked against `git log --grep='^merge: item'`, not against this
file's own claims: **131e, 303, 292, 305, 296+300, 291, 273 (+ residual 4), 227, 306,
310, 294 have all MERGED** since it was written, and 301 PART-LANDED. What genuinely
survives from it, in order: **298** (unblocked the moment 294 landed, by 298's own
note), then **293**, then **299** — still one lane each, still never a parallel wave,
because `chrome/mod.rs` is what makes them contended. After those: 302 (wants a quiet
tree), 174, 231.

**NEW, USER-REPORTED 2026-08-07 — items 312 and 313**, the frosted footprint's hard
edge and the flush-left hint under a leaning list. They slot into the SAME chrome
sequence rather than beside it: **313 is downstream of 293** (same file, same line),
and **312 reads `chrome/diagonal.rs`, which 311 also names**. ⚠️ **312 and 298 both
change what the footprint frost is** — 298 asks whether a context menu should frost
the document at all, 312 changes the shape and edge of the frost it would draw — so
**298 first**, or the second lane rebuilds the first's answer.

🔵 **AND THE SAME SURFACE CAME BACK, from the user, later the same day: ITEM 318 IS NOW
USER-REPORTED AND USER-DECIDED.** The frost under a leaning list still reads as a
rectangle, and the user has called the shape — **it must read as a parallelogram**. That
resolves the "decide explicitly" 312 left open, so 318 stops being a cleanup behind 312
and becomes the user-visible half of it. **It inherits 312's whole sequencing** (298
first; one lane with the chrome cluster, never a parallel wave), and its principled route
runs through the query line's independent `left` — the sibling of 313, touching
`overlay_place_caret` and the field's clickable band.

Order for the next wave (as derived 2026-08-06; read the note above first):

1. **131e** — selection and the full Verify clause; 131a–d are landed and the
   measured cluster rail exists in `render/chrome/diagonal.rs`. ⚠️ It reaches
   `render/chrome/diagonal.rs`, which **303 also names as contended** — 131e and
   303 are one lane, sequenced, never a pair.
2. **THE CHROME-GEOMETRY CLUSTER — 292, 293, 299, 303. One lane each,
   SEQUENCED, never a parallel wave.** Verified against the tree rather than
   asserted: 292's `strip_gap()` and 293's `OverlayGeom::hint_rows` are **both
   declared in `render/chrome/mod.rs`** (`:163` and `:176`), 293 also reaches
   `overlay.rs:248-297` and `overlay_shape.rs:743`, 299's accessory ink sits in
   `overlay_draw.rs`/`overlay_rows.rs` with `src/context_menu.rs`, and 303 names
   `diagonal.rs`. **The file partition cannot separate them** — `chrome/mod.rs`
   alone makes 292 and 293 one lane, so pairing any two of these is the
   duplicated-geometry bill the protocol's §8 warns about.
3. **294 THEN 298**, in that order, per 298's own note: the footprint scoping may
   be what the context menu wants rather than an off-switch.
4. **305** — the two `spans.rs` files at ~2× the ceiling. Now the ONLY remaining
   file-size debt of its class: 274 cleared the last two test monoliths and
   `probe.rs` was carved this wave, so `markdown/spans.rs` (1061) and
   `render/spans.rs` (1140) stand alone. ⚠️ Unlike those, this is PRODUCTION code,
   so a new `spans/mod.rs` gets **no baseline grandfathering and fails outright
   over 500** — the split must land under the ceiling, not merely under a mark.
5. **291** — now a HARNESS item, not a product one: 290 dissolved its primary
   defect, leaving the `mark_movement_input` overwrite (which authored a wrong
   number on this board), the 5 s vanish, and one stale comment.
6. **296 with 300** — they may be one defect, and 300 says debug before
   redesigning.
7. **273's six unbuilt residuals** — CLI flags have no roster to generate from,
   `Command` carries only `name`, `WORLDS.md`'s columns are hand-written, there
   is no in-app door, the site page is visually unreviewed, and the five-section
   structure was the lane's call rather than the user's.
8. **302** — needs a quiet tree by its own clause (~1000-site blast radius), so
   it schedules when no chrome or render lane is running.
9. **174** — one surface family migrated of every surface; the rest still own
   their geometry.
10. **227** — the AppImage. Nothing in the tree matches `AppImage`; it is
   unstarted and depends on 226, which is now complete.
11. **231** — a diagnosis item with **no live lead**; its shader-size hypothesis
   was falsified and nothing replaced it. Its named next step is a **macOS guest
   VM**, and this host has **no VM tooling installed** — a spend decision, not
   work to absorb.
12. **🔵 HUMAN / LIVE, none of which a lane can close** — see the BLOCKED and
   OWED sections above. **251 is on that list and is hardware-gated**: it needs a
   human at a Linux desktop with Orca, and no unlock of this Mac reaches it.

---

## Open items

118. ✅ **LANDED (merged 2026-08-07) — the six proposals are dispositioned, Galah shipped, and
     the map is durable data. TWO THINGS ARE OWED TO THE USER AND NOTHING ELSE REMAINS.**
     🔵 **MULGA'S RE-SCORE IS THE USER'S** — re-measured, deliberately not re-scored. Item 258
     retired the `Starfield` its `1/5` described; it ships `Pinstripe` now, the same shader family
     as Saltpan and Cassowary. **A different instrument than the one the 1 was scored against.**
     Mulga sits **above every current 1/5 world** on `g_sdL` (3.63 vs ≈1.6) and `g_sd_lp` (4.3 vs
     ≈1.8–2.0), with `g_edge` **0.2222 and `g_sd_lp` 4.3 — statistically identical to
     Cassowary's**, a world scored `4/5`. Not the same read, though: its `g_sdL` is a THIRD of
     Cassowary's and its ink is not accent-coloured, so it reads as tone-on-tone ribbing rather
     than a hard ruled pinstripe. Stable across all four arms (±0.02). **A fresh `1` no longer
     matches the arithmetic; somewhere in `2`–`4` fits, and which number is the user's.**
     Captures: `gallery/item-118/mulga-neighbourhood/`.
     🔵 **GALAH'S NEW DENSITY IS SHIPPED, NOT PROPOSED** — `0.10 → 0.12`, the smallest step in
     the user's own pinned `0.12`–`0.16` band that a real capture can tell apart. **The rung below
     is what makes it a threshold rather than a pick:** at `0.11` the max right-margin luminance
     delta is 1.9 with **zero** pixels crossing the repo's `EDGE_DELTA=3` floor; at `0.12` it is
     3.7 with 0.18% crossing. Worth a live `--theme Galah` glance to confirm "a tinny bit" reads
     as intended rather than as a rebalance.
     ✅ **ITEM 108 MET ITS OWN DONE CONDITION AND STILL LEFT ITS SUBJECT IN THE QUIET BAND** —
     both its guarding laws pass, but roster-relative Gumtree remains the **second-faintest ground
     of twenty** by `g_sdL` (0.93). Doubling density took it from imperceptible to floor-clearing,
     not out of the quiet end. **This is why Galah's step was measured from scratch rather than
     reusing 108's recipe.**
     ✅ **THE NEAR-PAIR MERGE CALL GOES TO THE USER, NOT DONE** — merging a world removes
     something a user may have chosen. ⚠️ **And the pairing that call NAMED is itself stale in
     exactly Mulga's way:** item 260 moved Magpie off `Pinstripe` onto `Bands` **the same day** 258
     moved Mulga, so the inherited "edge 0.4444 on both" note describes a ground Magpie no longer
     ships. The note is retired.
     ✅ **THAT STALENESS CLASS NOW FAILS A TEST INSTEAD OF SURVIVING A ROUND.**
     `docs/loudness-map.md` carries the map and its arithmetic, and a drift anchor snapshots every
     world's ground byte-exactly, **naming the world that moved** — mutation-proven by retinting
     Mulga's `tint`. The Firetail/Mangrove inversion is recorded closed per the user.
     ⚠️ **TWO CORRECTIONS MADE AT MERGE, both worth reading.** The density sweep's doc claimed its
     enrolment was **derived** from `Background::density()`; the code re-listed four variants, and
     `density()` ends in `_ => 0.0` — so routing through it would let a new density-bearing variant
     answer "no density" and leave the sweep **silently**. Now an exhaustive match that fails to
     COMPILE. And **the lane lost its report to a background gate, so its mutation proof arrived as
     a claim with no panic text** — re-run at merge: reverting to `0.10` fails
     `galah_density_lands_in_the_pinned_up_a_tinny_bit_band` by name.
     **Original:** **Pre-release world-loudness audit.** **Audit definition:** "idle loudness"
     is how strongly a world asks for attention while the user is simply writing
     in page mode: palette, typography, margin pattern, and ambient motion count;
     summoned overlays do not. `1/5` is the quiet pole (Wagtail), `3/5` is
     recognizable/alive but comfortable for hours, and `5/5` is a deliberately
     rare statement world (Firetail, Kite). **This is a diagnostic distribution,
     never permission to turn up a world merely to fill a bin** — each world
     still earns its own identity. **Done:** the roster has a user-confirmed
     loudness map, its mean/distribution and outliers are explicit, near-duplicate
     intensity poles are named, and every proposed rebalance is either rejected
     on purpose or queued with a world-specific reason. **Pixel/sidecar
     arithmetic may prove territory and contrast but never claims the taste
     score.**

     ✅ **THE USER'S MAP — GIVEN DIRECTLY, all twenty worlds:**

     | 1/5 | 2/5 | 3/5 | 4/5 | 5/5 |
     |---|---|---|---|---|
     | Gumtree, Bilby, Mulga, Tawny, Mopoke, Currawong, Brolga, Wagtail | Potoroo, Saltpan, Bombora, Bowerbird, Galah, Magpie | Quokka, Paperbark | Mangrove, Cassowary | Firetail, Kite |

     **Final distribution `8, 6, 2, 2, 2`, mean 2.20.** ✅ **THE LIVE `--release`
     AMBIENT SITTING IS DONE: "the movement worlds are good"** — all six moving
     worlds confirmed in the live app, so the ambient scores are formed live and
     are no longer provisional.

     ✅ **THE DIRECTION CALL IS MADE (user, 2026-08-06): THE TARGET SHAPE IS
     DROPPED.** The user's words: *"i think we just drop the target. it's fine,
     right now."* **`1, 7, 6, 4, 2` / mean 2.90 is RETIRED — not amended, not
     restated as a descriptive shape, not replaced.** The roster's measured mean
     of **2.20** is accepted as awl's shape rather than a shortfall, consistent
     with PHILOSOPHY's calm bias: a roster that lands at 2.20 when its owner
     scores it freely is reporting that the target was aspirational. Every world
     was individually confirmed, both 5s sit exactly where the roster wants them,
     and this item's own rule forbids turning a world up merely to fill a bin.
     ⚠️ **Do not re-derive a target, and do not read the eight 1/5s as a deficit.**

     🔵 **WHAT REMAINS, and it is the whole of item 118's open work: the six
     standing proposals, plus one stale score.**
     **(a) Disposition the six standing proposals:** Galah's ground density
     (magnitude pinned by the user: *"up it a tinnyyy bit"*, so a small step off
     `0.10` in the `0.12`–`0.16` neighbourhood — **land the smallest value that
     reads as different in a real capture, and show the arithmetic for the one
     below it too**); re-verifying that item 108 actually met its Done condition
     (Gumtree still measures second-faintest at its shipped density, so **verify
     108 worked before repeating its recipe**); the recorded Firetail/Mangrove
     inversion; ROADMAP's "merge the tightest near-pair" call; recording the map
     as durable data so the next run diffs instead of re-deriving; and Mulga.
     **(b) ⚠️ MULGA'S SCORE OF 1 IS STALE** — item 258 replaced the ground it
     described. **Re-score it.**

     ⚠️ **STANDING RULES THIS ITEM SETS AND THAT KEEP GETTING RE-LITIGATED.**
     The user's map diverging from any measured column is **not a defect and
     carries no obligation** — the roster already carries a recorded, accepted
     divergence of exactly this kind (Mangrove measures louder than Firetail on
     every static and motion column while ranking a step below it). **Do not
     queue work to reconcile taste with measurement; that inverts the item.**
     **Firetail and Mangrove stay as they are** — the inversion proposal is
     CLOSED on purpose ("theyre fine as is"); do not re-propose without a new
     reason. **Near-duplicate poles, named as the item asked:** Tawny/Mopoke
     (tightest — same `Dots{edge:false}`, edge 0.0000 both, L\* σ within 0.15),
     Magpie/Saltpan (edge **0.4444 on both to four decimals**), Bilby/Brolga (a
     deliberate mirror per THEMES.md). **In a code buffer the map does not
     describe what ships** — at `page_width_code = 100` a 1600px window leaves a
     16px margin, the ground effectively vanishes and the roster's spread
     collapses toward palette alone.

131. **131e IS PART-LANDED (merged 2026-08-06) — the MARK is done, the composition
     sweep is not.** ✅ The per-world mark is now theme data: `ListStyle::Diagonal`
     carries `DiagonalSpine { direction, mark: DiagonalMark { weight, reach, aperture } }`
     as its variant payload, so a world **cannot** author an orientation without a mark
     and a non-diagonal world cannot carry mark data nothing reads — the
     never-half-apply guard is the COMPILER's, not a law's. Magpie authors HAIRLINE
     (1.25/4.5/0.55) against Bitter, Mangrove keeps CRISP (3.0/5.0/1.0) against
     JetBrains Mono; drawn ink 71px vs 151px where one shared mark gave 148 vs 151.
     ✅ **The `Choreo::TwoShape` question this item inherited BY NAME was already
     resolved in the tree and law-tested** (`twoshape_echo_uses_its_own_nearest_planned_row_span`
     — each shape reads `display_nearest` of its OWN drawn centre, never re-running an
     animator). Ratified, not rebuilt. **Do not re-dispatch it.**

     🔵 **WHAT 131e STILL OWES, named rather than implied complete:** live filter and
     scroll TRANSITIONS (only fixed offsets were swept), child-picker return (check
     `docs/harness-reach.md` before promising a capture), the composition-level pixel
     laws — orientation, line continuity, inset attachment band, fixed label-control
     gap, placard/row non-overlap — which are 131e's rather than 303's and each need
     their own oracle, sidecar geometry agreement (blocked: the sidecar publishes no
     row geometry, needing item 174's schema bump), and dashboard captures. Covered:
     480 graded cells over every `OverlayKind` x both worlds x 1x/2x x four canvases x
     four list shapes, every `SettingId x SettingKind`, hit-test agreement at zoom, and
     a 5/5 vision smoke. Original body:
          **Give Mangrove and Magpie mirrored diagonal-line compositions across
     contextual menus and the real Settings workspace.** 131a–d are LANDED;
     **131e is what remains: selection, and the full Verify clause.**

     **The composition, for 131e's reference.** **Mangrove** draws a continuous
     descending `\` spine with row clusters left-aligned on the RIGHT side;
     **Magpie** draws a continuous ascending `/` spine with clusters right-aligned
     on the LEFT. The line is mandatory in both — the striking read comes from
     the drawn division and triangular negative space, not merely staggered text.
     Never amber/primary: Mangrove uses a crisp tidal-teal line derived from its
     muted ink, Magpie a crisp graphite one. Resting weight is clearly visible but
     subordinate to text; **the selected row brightens and thickens only the local
     spine segment toward `base_content`, extends a short connector to the row,
     and steps the row outward by a few crisp pixels — no spring, pulse, or
     full-width selection bar.** Query/title/category/footer regions remain
     horizontal and stable. Filtering and scrolling sample a fixed
     surface-relative line at fixed row y positions, so content changes never make
     the spine or surviving rows jump horizontally.

     **131e's Verify clause, which is the reason it was never folded into
     131c/d:** full no-wildcard `OverlayKind` row-surface sweep plus every
     `SettingId × SettingKind`; simple/long labels, chords, values, toggles, text
     entry, sliders, empty/short/full/filtered/scrolled lists, category changes,
     child-picker return, and narrow/wide staging; drawn line/row/control ↔
     hit-test agreement at zoom and 1×/2× DPI; pixel laws for orientation, line
     continuity, inset attachment band, fixed label-control gap, local selected
     segment, placard/row non-overlap, non-primary ink, and no clipping; exact
     before/after identity for every non-assigned world; dashboard captures and
     affordance-locating vision smoke over Commands plus every Settings category
     in both worlds; native, both conventions, and wasm gates.

     ⚠️ **THE ONE OPEN COMPOSITION QUESTION 131e INHERITS BY NAME.**
     `Choreo::TwoShape`'s echo band can represent a **different row mid-glide**,
     and whose offset it inherits was left explicitly to 131e. The single-shape
     case is already fixed. **Do not re-run an animator to answer it** — the Pane
     band's own doc explains that re-running one lets the fill land on a different
     row from the ink shaped against it.

     🔵 **A LIVE USER CONSTRAINT ON THE SELECTED-ROW MARK, STILL UNADDRESSED —
     verified against the tree 2026-08-06.** From a real Magpie screenshot: *"it
     needs to be thinner and more elegant"*. **The real finding is that ONE glyph
     cannot serve both worlds:** Magpie's display face is `Bitter`, an editorial
     slab serif whose whole register is contradicted by a heavy geometric mark,
     while Mangrove is `JetBrains Mono`, a technical face where a crisp geometric
     mark is correct. **So the mark's WEIGHT and form belong in theme data beside
     the world's face.** Today they do not: `render/chrome/diagonal.rs` carries a
     shared `SELECTED_SPINE_WEIGHT = Logical(3.0)` and `src/theme/` holds no
     per-world marker weight at all. ⚠️ **Do not tune the single shared constant
     until Magpie looks right and call it done** — that is the shape this note
     exists to prevent. Magpie wants a hairline, high-contrast,
     typographically-sympathetic mark; Mangrove wants the crisp one it has.

     ⚠️ **131's own rule, which every consumer inherits: never ship a
     half-applied world.** Both worlds move in one commit or neither does.

174. **Separate pure render planning from shaping/cache mechanics and GPU
     execution.** **Defect:** `TextPipeline` and the render directory jointly own
     scene policy, document geometry, cache invalidation, hit-test inputs,
     sidecar-visible facts, GPU resources, and feature-specific drawing. Tests
     often have to infer planned geometry from pixels, while render-touching work
     can accidentally couple presentation rules to device state. **Build:** one
     deterministic scene/layout planner consuming `ViewState`, measured text
     inputs, theme capabilities and viewport data, emitting inspectable primitives
     plus interaction geometry. Shaping and cache ownership remain a distinct
     measured stage; GPU execution consumes the plan without deciding feature
     layout. Route drawing, hit-testing and sidecar geometry through the same
     planned objects, **migrating one coherent surface family at a time.**
     Preserve O(visible) frame work, buffer-identity cache keys, rowlayout
     ownership, deterministic capture, and exact output for migrated surfaces. Do
     not build a retained widget tree, general scene framework, duplicate CPU
     renderer, or allocate an entire document plan each frame. **Done:**
     presentation decisions are testable without a device; GPU code executes
     rather than invents layout; drawn and interactive geometry cannot drift
     through parallel calculations. **Verify:** plan-level geometry laws,
     drawn↔hit-test↔sidecar identity, buffer-swap/resize/zoom invalidation,
     allocation and reshape-count witnesses, exact before/after capture probes,
     release frame benchmarks, both conventions, full native, wasm/WebGL. Every
     render slice gets the standing vision smoke. **Routing:** deep owner with a
     production-tier outcome audit.

     🟢 **ONE FAMILY LANDED — the item remains OPEN.** `src/render/plan/` is
     device-free (shapes nothing, measures nothing, reads no clock) and
     `plan_overlay_rows` emits one `PlannedRow` per candidate display line plus
     the interaction geometry. **The forward `row -> y` arithmetic and its inverse
     are module-private, and `overlay_row_top`/`overlay_row_of`/`overlay_row_index`
     are DELETED from `render/chrome`: the bypass is compiler-enforced, not
     grep-enforced.**

     **Left for later slices, stated rather than implied:** document-content
     surfaces, search panel, HUD, gutter, outline, popover, whichkey and readout
     still own their geometry; the spell popup's anchoring is untouched (its rows
     are planned, its anchor is not); `overlay_secondary_top` /
     `overlay_split_bounds` / `overlay_strip_band` remain separate owners and
     **folding the strip band plus secondary column in is the natural next
     slice**; **no sidecar schema change** — publishing planned row rects would
     let a test assert row geometry with no device at all, but that is a schema
     bump plus a CAPTURE.md edit.

     ⚠️ **A measurement-honesty note the next slice inherits:** the first slice's
     release frames showed palette cells at median +8.1% while the **untouched**
     cells moved median −0.2% across a −7.0%…+22.5% range on the same run, with
     five workers building concurrently. The honest reading is *no
     palette-specific signal, confirmation owed on a quiet host*, and the bench
     baseline was deliberately **not** re-banked so contention noise is not frozen
     into every cell.

231. **Name the CAUSE of the hosted-macOS gate hang. The fix is a SECOND item,
     scoped only once the cause has a name.** ⚠️ **REFRAMED BY USER DECISION from
     "fix the hang" to "diagnose it" — and the reframe is the most important line
     in this item.** One fix has already been attempted and **failed**: the
     `src/gpu_cache.rs` round cut `render::tests::` GPU program builds
     **52,083 → 5,577 (9.3×)** and **the hang did not clear**. A second
     speculative fix would be worse than the first, because **the strongest
     remaining candidate is a SYMPTOM MASK**: `src/test_gpu.rs` holds a
     process-wide `OnceLock<(Device, Queue)>` "created once and never dropped",
     and recycling it would very likely turn CI green **without anyone learning
     what was exhausted** — destroying the only instrument that can currently see
     whether a user on a VM is affected.

     **Defect:** `main`'s `mac (render::tests)` job **HANGS, it does not fail** —
     exactly three tests (the runner's 3 vCPUs) park at the same instant and never
     move, and the `cargo`/`awl-…` orphans **survive SIGTERM** because they are
     parked in `poll(PollType::wait_indefinitely())`. Bisected over six sequential
     probes to **`8207e519`** (Kite, `Background::WarpedGrid`, +267 lines of
     `background.wgsl`), **both boundaries measured**.

     **ELIMINATED — do not re-derive; each was killed by measurement.**
     (a) **The shader:** 15 `backgrounds_item132`/`warp_tunnel` tests pass cleanly
     six minutes before the wedge, in two independent logs — **do not start by
     staring at it.** (b) **A single bad test:** the victim varies between runs,
     so the commit poisons the device rather than owning the hanging test.
     (c) **Concurrency:** `RUST_TEST_THREADS=1` **WEDGES**. (d) **A per-device
     resource:** the mac and linux conventions — two separate processes with two
     separate wgpu devices — stopped **within 10 MILLISECONDS** of each other, so
     **the contended resource is SYSTEM-WIDE: the VM's virtualised Metal stack
     itself.** (e) **Program-build volume:** the 9.3× cut did not clear it, and
     `--skip render::tests::` **COMPLETED** while building ~80,000 GPU programs in
     aggregate — those tests create AND DESTROY devices, forcing driver-side
     reclamation. **It is not how much you build — it is how much you pile on a
     device the driver never reclaims.** (f) **RAM:** steady at ~2.37 GB.
     (g) **Software adapters as a stand-in:** two independent lavapipe stacks ran
     `render::tests::` at both bisect boundaries and neither ever hung — a
     software rasteriser has no system-wide GPU resource for a cross-process wedge
     to exhaust, so it cannot reproduce this class even in principle.
     ☠️ **(h) The shader-source-size lead is DEAD.** HEAD carries the LARGEST
     `background.wgsl` of the three trees and got **2.6× FURTHER** in the
     container; fitting `budget/test = C + K·shader_bytes` across the boundaries
     needs a **negative** constant term. ⚠️ **That eliminates a hypothesis about
     the CONTAINER'S OOM, which is a PROXY — it says nothing about the hang.**

     **STILL UNKNOWN, AND THIS IS THE WHOLE ITEM:** WHICH resource in the
     virtualised Metal stack is exhausted. **FIRST DELIVERABLE — a LOCAL
     REPRODUCTION**, because without one every hypothesis costs a ~50-minute CI
     cycle. ⚠️ **The untried arm is a macOS GUEST VM on the Apple Silicon host**
     (Virtualization.framework — `tart`, or UTM): a macOS guest gets genuine
     **paravirtualised Metal**, the same class of stack as the hosted runner, and
     nothing local has ever exercised that axis. **No VM tooling is installed**,
     so the setup cost is real — state it rather than assuming it is free. **A
     negative here is a publishable result too.**

     **THE DECISION GATE — the item is not done without it.** Once the cause is
     named, answer: **is the PRODUCT exposed, or is this test-harness-only?** The
     asymmetry that decides it: the per-frame `create_shader_module` +
     `create_render_pipeline` churn exists **only in the test helpers**; the live
     app builds `BackgroundPipeline` once at construction and `prepare()`
     thereafter only uploads uniforms including the shader id. **But that rests on
     the churn hypothesis, which the 9.3× null result has WEAKENED** — if state
     accumulates from the WarpedGrid draw itself, or from allocations rather than
     programs, **a user on a VM IS exposed.**

     **Do NOT land a fix under this item.** Specifically: do not recycle or tear
     down the shared test device, and do not tune anything, until the cause has a
     name and the product/harness question has an answer. **If the diagnosis
     converges early and the fix then looks obvious, it still lands as a SEPARATE
     item so the causal claim and the change stay separately reviewable.**

     **Carry-forward facts a new owner would otherwise lose.** ⚠️ **wgpu 29.0.3:
     `wgpu::Device`'s `PartialEq` reports two separately requested, simultaneously
     live devices as EQUAL** (measured) — a device-keyed cache is impossible. A
     `cfg(test)` cache also **must not be thread-local**: libtest gives every test
     its own thread. **One law initially PASSED its own leak mutation** — drawing
     one world at a time lets each `prepare` overwrite the last; only building and
     preparing all twenty BEFORE any draws exposes it. ⚠️ **Two harness bugs, both
     of which scored a 60-minute hang as a PASS, both the same shape — an
     unfinished step wearing a finished step's field:** `gh` encodes an unfinished
     step as `conclusion:""` (never `null`), and a step killed by the job ceiling
     reports `status:"completed"` with `conclusion:"cancelled"`. **A harness
     reading a status field must enumerate what it accepts, never test for
     inequality.** ⚠️ **And a probe-integrity trap:** a cross-commit pass
     **silently scored the same binary twice** — both trees extracted within the
     same second, so Cargo's mtime fingerprint reused the other tree's artifacts.
     Use a target dir per tree plus a provenance assertion that fails on mismatch.

     **Done:** the exhausted resource is NAMED with a confirming measurement
     rather than a hypothesis; the product/harness question has an evidenced
     answer; and the fix is scoped as its own item. **Verify:** whatever names the
     cause must also PREDICT THE BOUNDARY — why `36707d06` survives and
     `8207e519` does not, why `--skip render::tests::` survives while doing more
     total GPU work, and why two processes on separate devices stop within 10 ms
     of each other. **Routing:** deep tier, one owner end to end. The rig is
     `scripts/oom-budget-container.sh`, labelled in its own header as a diagnostic
     reproducer and **not a gate**.

251. **Item 207's AT-SPI journey needs a LINUX machine.** **Defect:** the board's
     live-closure lists kept grouping "207's real VoiceOver / AT-SPI journeys"
     under *needs an unlocked display*. **That is true of the VoiceOver half and
     false of the AT-SPI half** — AT-SPI2 is the **Linux** accessibility API, so
     no amount of unlocking the dev Mac reaches it. Filed as its own item because
     **a blocker misattributed to the wrong cause never gets cleared.**
     `ACCESSIBILITY.md:110` states plainly that **no AT-SPI journey has been run
     at all**, and that honest-limits section must stay correct. **Build:** record
     what the journey requires — a Linux desktop session, Orca, the native build,
     and the same journeys the VoiceOver sitting runs: document read, caret and
     selection announcement, overlay summon/dismiss, and an editing burst.
     **Scope:** does NOT include shipping a fix for whatever it finds; a defect
     found here earns its own item. **Done:** either the journey has been run on a
     real Linux session and its findings recorded in `ACCESSIBILITY.md`, or the
     item stands parked with its hardware requirement stated. **Verify:** human
     journey; there is no headless stand-in, and AccessKit law tests already cover
     the projection, which is precisely the layer this item exists to look past.
     **Routing:** human, on Linux. ⚠️ **What unblocked here is the PROBE, not a
     defect:** the AT-SPI tree was correct all along, since AccessKit filters
     `Role::TextRun` from accessible children by design.

273. **RESIDUALS (3), (5) AND (6) ARE CLOSED (merged 2026-08-06). (1), (2) AND (4)
     REMAIN, each with a stated reason.**
     ✅ **(5) built:** `site/reference.html` is a docs page — sidebar navigation and
     anchored headings, **emitted from `Section::ALL` and `blocks()`**, with a nav
     link and its target heading sharing ONE `caption_id()` call. Three laws,
     including one that requires every nav `href` to resolve to a real `id=` in the
     page — the byte-diff alone cannot see that, since it compares the nav block
     only to itself regenerated. ⚠️ The nav is `div[role=navigation]`, **not**
     `<nav>`: `law/site.rs` sweeps every literal `<nav>` and demands identical
     destinations across pages, which is right for cross-page nav and wrong for an
     in-page TOC. Rendered and clicked in a real browser at three viewports.
     ✅ **(3) closed, and it found WORLDS.md WRONG IN TWO TABLES.** 21 stale cells
     across 11 of 20 worlds in the at-a-glance table (incl. Mopoke's Display naming
     an unrelated typeface), and **five errors in the background-membership table,
     one a week old**: Magpie under `Pinstripe` while carrying `Bands`; `Bands`
     calling itself DORMANT; `Gradient` claiming Galah, which has carried
     `Deckle`/`Fibres` since 2026-07-30; `Deckle` calling Fibres dormant; and `Dots`
     claiming Bowerbird, which carries **`Organic` — a variant with no row at all**.
     Both tables now carry roster-derived laws, and the Organic gap is made LOUD by
     a second law excusing exactly one pair. Every fix moved the DOCUMENT to the
     roster, never the reverse.
     ✅ **(6) decided earlier by the user — five sections kept.**
     🔵 **(4) IS BLOCKED, correctly reported rather than routed around:** an in-app
     door needs a new arm in `src/actions.rs` and `src/app/apply.rs`. Sequence it
     behind any lane holding those.
     ✅ **(2) CLOSED (merged 2026-08-07).** `Command` carries `description: Option<&'static str>` with **no `Default` impl**, so all 96 literals had to write the arm — the compiler enumerated the work instead of a law chasing it.
     🔵 **(2) IS ITS OWN ITEM, not a residual.** `Command` gaining a description
     means authoring 93 accurate one-liners under the docs-voice rule ("facts traced
     to verified sources") — larger than (3) and (5) combined.
     ✅ **(1) CLOSED (merged 2026-08-08) — THE ROSTER LANDED AND IT CAUGHT A WRONG NUMBER ON ITS
     WAY INTO A PUBLIC DOCUMENT.** One `flag_roster!` invocation derives both the `FlagId` enum and
     the `FLAGS` table from a single list; `lookup` is the one door, `take_operands` the one operand
     source, and dispatch is a **no-wildcard `match flag.id`** — so a new row **fails to COMPILE**
     until it is handled (proven with a throwaway row: `E0004` at `args.rs:143`). `--help` and the
     reference's CLI section generate from the same rows the parser dispatches on.
     ⚠️ **THE FIND, and it is exactly the hazard this repo records about generated documents:** the
     hand-written `--help` said `--measure … (default 80)`. The value is **70 for prose, 100 for
     code**. **Generating that string would have shipped a wrong number WITH A DRIFT LAW BEHIND
     IT** — caught by a spot-check that asked `page::DEFAULT_MEASURE` instead of trusting the text.
     `src/page.rs`'s module doc carried the same stale 80 and now names the consts rather than
     restating their values.
     ✅ **The premise was right about `parse_args` and incomplete about awl:** 61 arms, but the real
     surface is **64 flags** — `fn main` scans for three (`--print-menu-roster`, `--dump-menu-icon`,
     `--fault-write-loop`) and returns **before** `parse_args`. Those cannot be rows (each exits the
     process), so the boundary is named in `PRE_PARSE_FLAGS` with a law refusing any flag to be both
     a row and a pre-parse scan. **One flag, one parser.**
     ✅ **Operands are load-bearing, not decorative** — the loop consumes exactly what each row
     declares, so a wrong arity breaks PARSING rather than only misprinting a table, and all 40
     refusal messages stay byte-identical because each is stored verbatim. Documenting the 23
     unlisted flags is what makes a flag unable to land undocumented: whichever `Listing` it picks,
     the byte-diff law fails until regen. Eleven mutation proofs, two of them **re-proven after a
     clippy restructure and after rustfmt**, because a mutation that edits nothing reads green.
     ✅ **Ledger TIGHTENED both ways:** `args.rs` 1296 → **1127**, `parse_args` 809 → **616**.
     ✅ **Fixed at merge:** `--ground-audition`'s summary read *"item 121's A/B/C ground-audition
     manifest"* — a queue citation that had reached a **public** `REFERENCE.md` and the live site.
     Reworded and regenerated; **the regen diff is exactly that one string in both documents**,
     which is the check a generated document earns. — 61 flags hand-parsed in one
     `match`, in a file already carrying size and complexity exceptions.
     🔵 **OWED TO THE USER'S EYE:** the WORLDS.md correction makes several worlds
     visibly sparser (Mulga now shows Register alone, Tawny Register+Temp). The doc
     is now right; whether the "curated maximum of four per band" framing still
     reads well when it is this sparse is a taste call. Also the sidebar layout
     itself, on a public page.
     **THE REFERENCE MANUAL — SIX RESIDUALS, named as unbuilt rather than implied
     complete.** The mechanism ships: `REFERENCE.md` + `site/reference.html`,
     every table generated from awl's own rosters — commands (93, both conventions
     asked explicitly), synthetic chords, settings (31), config keys (31) with
     numeric bands, worlds (20), markdown constructs and conceal — **held by 17
     named drift laws**, with the site page **not a hand-mirror** but the same
     rows through an HTML emitter, so the two cannot disagree about a fact.
     **What follows is what it does not yet cover.**

     **🔵 THE SIX RESIDUALS.** (1) **CLI flags have no roster to generate from** —
     `main/args.rs` hand-parses 61 in one `match` and `--help` is one hand-written
     string, so that section needs the flag list lifted into data first;
     (2) **`Command` carries only `name`**, so the reference says what a command
     is called and bound to, never what it *does*; (3) **`WORLDS.md`'s
     Display/Mono/axis columns are still hand-written** and can drift — only
     membership is law-checked; (4) **no in-app door** (Guide and Credits have
     palette commands, the reference does not); (5) **the site page is visually
     unreviewed**; (6) **the five-section structure was the lane's call**, not the
     user's — re-sectioning is cheap since the marker pairs and `Section::ALL` are
     the only coupling.

     ✅ **(6) IS DECIDED 2026-08-06 — KEEP IT.** User: "273, i think that's
     fine." The five sections stand for `REFERENCE.md`; no re-sectioning.

     🔴 **(5) IS PROMOTED FROM "UNREVIEWED" TO A BUILD TASK, and it is the real
     remaining work.** User: *"it's not really friendly as a webpage? we should
     divide the sections up i think.... yknow, what a typical docs page looks
     like."* `site/reference.html` is today one long emitted scroll — the
     markdown's own shape pushed through an HTML emitter. **A reference someone
     browses in a browser needs the conventions of a docs site**: persistent
     section navigation, anchored and linkable headings, and the sections
     genuinely divided rather than stacked. Commands alone is ~43% of the
     document and is a single undifferentiated wall on the page.

     ⚠️ **THE CONSTRAINT THAT MAKES THIS NON-TRIVIAL, and it is this item's whole
     achievement: the site page is NOT a hand-mirror.** It is the same generated
     rows through an HTML emitter, held by the drift laws, "so the two cannot
     disagree about a fact." **Any restructuring must keep that property** — the
     navigation and division are emitted from `Section::ALL` and the same
     rosters, never hand-authored beside them, or the next roster change silently
     desynchronises the page from the manual. **A hand-written sidebar would
     forfeit the one guarantee this feature exists to provide.** Splitting into
     multiple PAGES is permitted only if the split is likewise generated.
     **METHOD — LOOK AT REAL DOCS SITES FIRST.** User, 2026-08-06: *"the agent
     should look at a couple of examples, eg bear, or typora, documentation, and
     generate the equivalent 'reference' section."* **Study how comparable
     writing apps structure a REFERENCE** — Bear's and Typora's docs are the two
     named, and one or two more in that register are welcome — and derive awl's
     equivalent from what those get right. **Do not invent a structure from first
     principles when the conventions already exist and readers already know
     them.** ⚠️ **Study the SHAPE, never the prose:** what earns a page of its
     own versus an anchor, how a long command table is broken up, where
     navigation lives, how a keyboard-shortcut roster is presented. Copy no text
     and no CSS.

     🔴 **THE GETTING-STARTED GUIDE IS THE USER'S AND IS OUT OF SCOPE.** User,
     same message: *"the getting started guide is something separate that i'll
     write."* **This lane writes NO onboarding prose, no tutorial, no "your first
     document", and does not restructure `GUIDE.md`.** It builds the reference's
     presentation only. If the examples suggest guide-shaped content, **report
     the observation and stop** — that is a note for the user, not work to
     absorb.

     ⚠️ **TWO DIFFERENT "ZERO NETWORK"s — do not conflate them.** Researching
     other docs sites over the web is ordinary lane work and is FINE. **awl's
     zero-network invariant is about the PRODUCT and the SHIPPED PAGE:** the
     deployed `site/reference.html` carries no CDN, no off-host webfont, no
     remote script, no analytics — every asset local, exactly as the site ships
     today.

     **Scope:** the site page's presentation only — `REFERENCE.md`'s content and
     sectioning are settled above and do not move. **Verify:** the drift laws
     still hold across the restructure; every section reachable and linkable;
     the page reviewed on a real browser at desktop and narrow widths; no
     off-host asset in the shipped output. **Routing:** production tier.

     ⚠️ **THE LESSON THE RESIDUALS SIT ON: GENERATION IS NOT SAFETY; IT MOVES THE
     ERROR FROM TRANSCRIPTION TO SOURCING.** The spot-check found three defects in
     its own first pass, all the same shape — generated from the wrong owner:
     `project_root` printed as a `config.toml` key the loader never reads; a Step
     column printing each band's MINIMUM because the readout formatter clamps
     first; and a reveal column asking `wysiwyg_reveals` ONCE with the caller's
     precomputed flag, inverting every line-scoped row. **Each closed with the law
     that catches it.**

     ⚠️ **A FLAGGED "DATA SMELL" THAT IS NOT ONE — CHECKED, SO NOBODY "FIXES" IT.**
     `theme/worlds.rs:142`'s `font: "Newsreader 16pt 16pt"` is **correct as
     written** — `render.rs:394` documents it as the actual registered family
     name, verified through fontdb, and `"Fraunces 9pt"` is the same shape.
     **Changing it would break Bilby's font resolution.**

     🔵 **THE SITE NAVIGATION QUESTION IS STILL OPEN.** `index.html` carries a TOP
     nav (`<header class="site-nav">`); `guide.html`, `credits.html` and
     `check.html` carry a FOOTER nav instead (`<nav class="foot-links">`) and have
     no top nav at all. **Both lists carry the same links, hand-duplicated across
     four files, already differing in link TEXT ("Try" vs "Try the editor") and in
     path style (`editor/` vs `/editor/`).** `site/llms.txt` is a THIRD
     enumeration. **Decide deliberately whether a round introduces one owner for
     the nav or accepts the duplication and adds to every copy — say which and
     why; do not silently do the second and leave the drift.**

283. **`ListStyle::Rules` GRADUATED — and handed back TWO THINGS, which are what
     remains open here.**

     **The brief's one design question dissolved on measurement.** The
     orchestrator asked the lane to decide the lens-strip tab pills, on the claim
     that a `Rules` theme picker leaves a bare strip. **The theme picker has no
     lens strip on any world** — retired by user decision, stated in
     `capture/modes.rs`, and a live `Cmd-T` capture carries an empty one. That was
     the **seventh** orchestrator-authored premise falsified in one session, and
     the pattern is unchanged: the brief described a surface without checking it
     existed. Where strips DO exist (file pickers, palette, History, Settings),
     `Rules` already answered in its own vocabulary — `FacetStyle::Text` marks the
     active lens with a hairline under its label, which is a rule like the ones
     arranging the list.

     🔵 **WHAT A SECOND CARRIER NEEDS** (all recorded in
     `theme/tests/personality.rs` beside Paperbark's entry, so the next author
     reads it there rather than here):
     - **A taste call on WHICH world** — deliberately out of 283's scope.
     - **A findability check on a DARK ground.** `rules_ink` uses `faint()` for
       hairlines and `base_content()` for the mark, so it is data-driven — but "a
       hairline at `faint()` is findable" is asserted **only on cream**.
     - **A `FacetStyle` that is not `Chips(FilledActive)`**, which would put a
       filled pill back on the strip. Paperbark is `Text`, so the interaction has
       never been posed and nothing currently forbids it.

     ⚠️ **AND `Rules` MUST NOT REACH A SECOND WORLD BEFORE 289 CLOSES** if that
     world's users are on Retina by default — the strip's mark is the style's own
     selection vocabulary, and a half-weight rule is a half-legible affordance.

293. ✅ **LANDED (merged 2026-08-07) — it was TWO defects, not one.** Not clipped:
     nothing was ever clipped because no space was ever reserved. **NOT COMPUTED** (no
     producer of `hint_rows` added a row for a gap) with a **NOT-DRAWN** twin (the
     shaper pushed one bare newline). Proven as two rather than one-described-twice by
     mutating each half alone and getting **distinct** symptoms.
     ⚠️ **THIS ITEM'S OWN CITATION WAS MISATTRIBUTED:** the *"footer.len() + 1 … a blank
     separator line"* comment belongs to `OverlayGeom::footer_rows` (the Keybindings-tips
     band, which already HAD its separator), not `hint_rows`. The row-list→hint boundary
     never carried the analogous term at all.
     ✅ **A pre-existing design law was retired and the reasoning was checked here before
     accepting it:** `overlay_rhythm_item112`'s "better balanced than the retired dials"
     compared a difference of differences whose counterfactual shifts BOTH sides by a
     constant unrelated to the new dial, so a third dial turns it into a coincidence
     (ties on Mangrove, inverts on Saltpan, no product change). **Both directional claims
     survive and the comparative became an ABSOLUTE bound** — stronger, not weaker.
     ✅ **A latent test bug surfaced that had been invisible its whole life:**
     `surfaces_item225` added a logical pad of 12.0 **without scaling by dpi**, exposed
     only once a correctly taller plate reached it at 2×.
     🔵 **Owed:** `OVERLAY_HINT_GAP_ROW = 0.45` was tuned against a compact-chin law, not
     by eye. And two disclosed coverage gaps, stated rather than hidden: one name-based
     `OverlayKind::Spell` exclusion, and the row-count law proven on three representative
     kinds rather than the roster. **Original:** **The overlay footer crowds the last row.** The hint line sits hard against
     the last candidate row. `OverlayGeom::hint_rows` documents itself as
     `footer.len() + 1` — "a blank separator line" — so a separator is already
     specified and none is drawn. Establish whether it is not computed, not
     drawn, or clipped before changing a constant.
     ⚠️ Seen on Kite (`Pane`) AND Cassowary (per-row plates), so the owner is the
     shared footer band, not a list style. Sweep the roster and every
     `OverlayKind`, including the context menu.
     **Verify:** the gap measurable on every world at 1×/2× on full, filtered,
     scrolled and empty lists — the empty-state notice row shares this band and
     has collided with the footer before. **Routing:** production tier.

294. ✅ **LANDED (merged 2026-08-06)** — a SCISSOR, not a rect uniform, so fragments
     outside the card are never written. `Frost = Full | Footprint(rect)` with
     `frost_mode` asking every full-takeover condition FIRST, so existing frosted frames
     are byte-identical by construction. Enrolment from the roster
     (`list_backing() != Card && !draws_row_plates()`), today Mangrove/Magpie/Paperbark.
     ✅ **The subtle half:** two consumers that mutate the DOCUMENT for the blur's sake
     key on `full_frost()`, not any frost — otherwise Mangrove's authored posterization
     would have been stripped from the live page OUTSIDE the card and its lava frozen
     where it is still visible. **Original:** **Blur the theme picker's own footprint.** On plateless worlds — `Diagonal`
     (Mangrove, Magpie) and `Rules` (Paperbark) — the document and the list
     interleave glyph-for-glyph. Frost is a property of the plate and those
     compositions draw none.
     **Build:** route the crisp pickers through `BlurBackdrop` with `DIM` at or
     near 0, scoped to the card's footprint so the surrounding page keeps the
     world's live colours. ⚠️ Footprint scoping is the work: `draw_backdrop`
     draws a fullscreen triangle, so this needs a scissor or rect uniform.
     `Pane` worlds are excluded — their plate already covers the document.
     **Scope:** both crisp pickers (`Theme | Caret`, `app/viewstate.rs:165`), no
     carve-outs — whatever the card covers is blurred, caret included. If a caret
     card over its own caret reads badly that is an ANCHORING question, not
     grounds for an exception. A general under-overlay scrim was rejected.
     **Verify:** no document glyph survives as text inside the footprint; hue
     unchanged outside it; swept 1×/2× on `Diagonal` and `Rules`; byte-identity
     for `Pane`. ⚠️ Laws in `render/tests/{hud,one_bit,outline}.rs` pin the
     current crisp behaviour — re-aim, don't delete. **Routing:** deep tier.

296. ✅ **LANDED with its pair (merged 2026-08-06).** Original body:
          **`ConvertLineEndings` is silent, and no capture can photograph a toast.**
     The action flips on-disk EOL with `Effect::None` — no notice ever — and is
     deliberately off the undo timeline, so a double-toggle is undetectable.
     Palette-only and unbound, which bounds severity.
     ⚠️ **The larger half:** `--screenshot-app` renders no toast for ANY action,
     cross-checked against `Cmd-S`. Every "the notice is set" claim in this tree
     is a sidecar claim. `prepare_notice`'s own comment asserts "every capture
     (which can never have a notice — autosave is live-only)" — false now, with
     ~10 callers including export, which IS reachable headlessly. The capture
     path was designed around an invariant later callers invalidated.
     **Build:** a notice naming the resulting convention; repair the capture path;
     fix the comment. **Verify:** a capture carries a toast. **Routing:**
     production tier.

297. ✅ **LANDED (merged 2026-08-07).** The cue is now a run turned 90° in the ROOM's own
     outer margin, rising from just above `COMMANDS` at exactly ⅔ its type size and in its
     ink — 505px of run where a 15px muted whisper sat at the card's border.
     ⚠️ **THE RESPONSIVE BOUND IS NOT `card_x`, and the first cut got it wrong** — under
     `Bars` the SELECTED row's plate grows OUTWARD past the card box (`grow_span`) and its
     scrim pads that again: **32 device px of card left of `card_x` at 1.8× zoom**, which
     put the cue two pixels from the plate. The bound asks the plate's own span owners at
     the growth animation's settled maximum.
     ⚠️ **THIS ITEM'S "LONGEST FACET" WAS WRONG, and so was the retired code's own
     comment:** both named workspace labels, and **a workspace plans no
     `PlanLine::Location` at all**, so the cue draws only on the four card-shaped faceting
     kinds. The longest a card can carry is "This folder". All 22 swept anyway.
     ✅ **The differential technique is worth reusing:** the reference frame is a **BLANK**
     location (datum present, text whitespace), not "no location" — which would make the
     planner emit the retired uppercase `Header`, a real glyph run. Everything else stays
     byte-identical, so the non-overlap arms can scan the **whole canvas** rather than a
     window that could hide a collision.
     ✅ Two mutations stayed green and **both produced a new law**; a third went red but
     only grazed its ceiling, exposing the ⅔ law as a tautology (it read the same constant
     the mutation changed), so a separate value law was added.
     ✅ Two PRE-EXISTING laws failed under the full filter and were repaired: one was a
     card-scoped ink oracle **reporting a weak cue while really measuring an empty line**.
     **Original:** **Cassowary's rotated location label is too small and misplaced.** Today
     `LocationStyle::RotatedRail` draws the facet name small, muted and flush with
     the card's left border. Target: rotated 90°, ~⅔ the Archivo Black `COMMANDS`
     placard, along the room's left edge, ABOVE the placard — a vertical
     companion to the wordmark at its own scale class.
     ⚠️ The type's doc comment specifies "small", "muted" and card-flush; all
     three are overturned, so re-author it in the same commit.
     **Scope:** Cassowary is the sole carrier (`worlds.rs:984`); `Raked` is out.
     Reuse `rotated_location.rs::prepare_rotated_location_label` — no second
     rotation path. **Responsive bound (the real risk):** size from the longest
     facet in the roster and the real edge territory, never the shortest; never
     overlap the card or placard, never clip, never fall back silently.
     **Verify:** every `OverlayKind` × facet name × narrow/wide/zoom at 1×/2×,
     with non-overlap and ⅔-relation laws; byte-identity for 19 worlds.
     **Routing:** deep tier.

298. **A right-click menu should not frost the document.** ✅ **294 HAS ANSWERED THE
     SEQUENCING QUESTION AND IT IS A ONE-LINE ROUTING CHANGE — with two caveats that
     change the shape.** Its `Frost::Footprint(rect)` gives proportional defocus, which
     is what this item asks for. But (a) **294's predicate is the wrong one to reuse**:
     it asks *"does the card back itself"*, while 298 asks *"is this a takeover"* — a
     context menu is `Pane`-backed on most worlds and would be excluded, so 298 adds
     its own arm to `frost_mode` keyed on the overlay KIND. And (b) it must first
     decide **whether a pointer-summoned menu frosts at all**, since on a `Pane` world
     its own plate already covers its footprint — **in which case `None` is right and
     this item really is the off-switch it first proposed, now with a principled
     reason rather than a hunch.** A four-row context menu
     takes the full-takeover treatment. `blur.rs` frames the effect as the
     defocus "behind a full-takeover overlay" and names the palette, go-to,
     outline, keybindings and spell — a pointer-summoned menu is none of those.
     **Build:** exclude the context menu from `BlurBackdrop` routing.
     ⚠️ Read with 294: they agree that defocus should be proportional to what the
     overlay covers. Sequence 294 first — its footprint scoping may be what this
     wants rather than an off-switch.
     **Verify:** document region byte-identical under a summoned context menu;
     full overlays keep their frost. **Routing:** production tier.

299. **Two rows in the same state draw their accessory in different inks.** Copy
     and Paste are both disabled, both render `unavailable`; Paste's is legible,
     Copy's is near-black on near-black.
     ⚠️ Diagnose, do not tune a colour. Cut is SELECTED and Copy is the row after
     it, so an accessory ink resolved from the wrong row's state would produce
     exactly this pair — a hypothesis to test, alongside a plain `faint()`/
     `muted()` split. Establish which row's state the ink is read from first. An
     accessory reading an adjacent row is a drawn/state disagreement that will not
     stay in this menu.
     **Verify:** a contrast floor for every disabled row's accessory against its
     own plate, swept over `OverlayKind` × **selected-row position** × world — the
     selection index is the axis the offset hypothesis lives on, and a
     selected-row-0 fixture could not see it. Contrast by pixel arithmetic.
     **Routing:** production tier.

300. ✅ **LANDED with its pair (merged 2026-08-06).** Original body:
          **The toast is never seen — debug before redesigning.** The user has never
     observed a channel with ~10 callers. Establish first whether it renders at
     all: does `set_toast_notice` reach `frame`; does `notice_readout_text`
     return it; does `prepare_notice` place it on-screen or park it; does the
     frame present. A redesign of something that never draws is wasted work, and
     the evidence points both ways — `TOAST_LIFETIME` 2500 ms, LABEL scale,
     `muted()` ink, bottom-centre of the writing column would all explain "never
     seen", and so would a real defect (296).
     **Then:** a notice a writer registers without it becoming nagging chrome.
     Position is the likeliest defect — bottom-centre is outside the reading eye's
     path. ⚠️ DESIGN gives motion and accent to the caret alone, so not a banner.
     ⚠️ A contrast floor is necessary and NOT sufficient — what shipped would pass
     one. Do not close this by adding a legibility law.
     Must reach the CLI/headless path too, which merges with 296's gap; they may
     be one defect. Prototype in awl via capture, never an HTML mockup; closes on
     the user's eye. **Routing:** deep tier, then the user.

     ⚠️ **RAISED IN PRIORITY BY 295, AND ITS SCOPE IS WIDER THAN ITS TITLE.**
     `NoticeState` has **two kinds**, and this item is written about one of them.
     `set_sticky_notice` sets `expires_at = None` — a sticky notice does **not**
     expire, so `TOAST_LIFETIME` cannot explain an unseen sticky one, while
     position, LABEL scale and `muted()` ink explain both. **Probe both kinds; a
     `Toast`-only diagnosis answers half the channel.**
     **295 now depends on this.** Its (a) fix routes a non-Markdown Export
     through `NoticeEffect::Sticky`, so **the user-visible half of a landed fix is
     gated here** — if a sticky notice does not draw, Export still fails silently
     for the user and only a unit test knows otherwise. `app/files/verbs.rs`
     already had two production sticky callers before that, so this was never a
     test-only channel.

301. ✅ **FULLY LANDED (merged 2026-08-08) — THE BLOCKED BUNDLE SHIPPED VIA ROUTE 1, AND THE
     CENSUS HAD GROWN 2.6× WHILE IT WAITED.** The premise held; the estimate was stale.
     **26 `E0004` sites, not 10** (`src/render/**` 6 → 7), every one a one-line arm and none
     needing a design decision — the errors are the no-wildcard roster style working. Route 2 was
     never considered.
     ✅ **`OverlayKind::ExportDest`:** title **"export to"** (the highlighted folder completes the
     sentence), hint `type to filter · ↵ export here · → open · ← up`, and a **folders-only empty
     state** because MoveDest's shared "no files here" is wrong for a folder list. Three navigation
     sites route through one new owner, `is_folder_destination()`, rather than each growing a
     second `== MoveDest` branch.
     ⚠️ **THE PART A NAIVE BUILD GETS WRONG: the format must survive the navigation.** The action
     chooses it, the accept reads it, and **`Journey::relevel` replaces the whole card at every
     descend and ascend.** It rides `OverlayState::export_format`, carried by one owner
     (`carry_level_payload_from`) — the same mechanism `Bind::Path` already uses for a Settings
     folder key. **Proven load-bearing:** deleting that call reddens ONLY the descend/ascend law
     while the other three export laws stay green.
     ✅ **`ExportTarget::at` is the one owner of how much of the path the notice speaks, as a
     RELATION rather than a per-arm flag** — with four routes to a destination a flag gets set by
     guess, and it reads wrong for the reachable case where the chosen folder is the one the
     document already lives in.
     ✅ **The modal is gated by the shipped reveal's own check, copied verbatim**, so
     `--screenshot-app` and every test take the identical write path and a surfaceless `App` cannot
     reach a main-thread modal — the cost the parked estimate missed. Verified live:
     `--screenshot-app` opens `mode: "export_dest"` under `driver: "live-app"` without hanging.
     ✅ **THE ELLIPSIS IS RESTORED, AND THE DECISION IS A TABLE RATHER THAN A PREFERENCE.** With it,
     the label is true on macOS (`NSSavePanel`) and Linux (the card) and false only on **web**,
     where the browser owns the download and awl opens nothing; without it, false on both platforms
     where a surface actually opens. **That residual is PINNED as a law asserting the set of
     platforms the one static label over-promises to is exactly `{Web}`** — red if web ever gains a
     destination surface, red if native ever loses one. `NATIVE_PANEL_IDS` gained the three export
     ids as data, routed through the roster's own `resolve` table.
     ✅ **And that law caught a second-order effect:** the catalog-divergence law enumerates rows
     whose menu label MUST differ from the catalog name, so the three Export ids had to be
     **removed** from it. **A law demanding a divergence that no longer exists is the mechanism
     working.** Seven mutation proofs, one at a time, each replacement asserted applied.
     🔵 **LIVE-ONLY, unreachable from this host:** the `NSSavePanel` itself (no test process can
     observe an AppKit modal; `MainThreadMarker::new()` returns `None` off the main thread, so the
     panel body is structural-by-construction) — **does it open at the right folder with the right
     name pre-filled, and does Cancel leave the document untouched?** And the drawn **Linux** menu
     bar actually firing `awl.export_word` into the card: the card, its navigation and its accept
     are all shared-core and tested here, but no macOS host clicks a Linux bar.
     **Original:** **PART-LANDED (merged 2026-08-06) — the DESTINATION OWNER, the REVEAL and a LAW
     REPAIR shipped; THE PANEL ITSELF IS BLOCKED.**

     🔴 **THE BLOCK, measured not guessed:** the `NSSavePanel`, the Linux save-role
     picker and the ellipsis flip are **one bundle**. An `OverlayKind::ExportDest`
     variant produces **10 `E0004` non-exhaustive-pattern errors, six in
     `src/render/**`** (`rowlayout.rs` + five render tests) — the repo's own
     no-wildcard roster style is what makes them errors. **It cannot ship half-done
     because `Routed::label` feeds BOTH menu bars** (`menu::roster()` drives the
     awl-drawn bar on Linux/web), so the ellipsis is a cross-platform promise on one
     static string: a macOS-only panel lies to exactly one platform whichever way the
     label goes — a fresh instance of the defect 295 fixed.

     🔵 **TWO UNBLOCK ROUTES, and this is the decision to make:**
     1. **Release the render hold** for six one-line match arms → a proper
        `OverlayKind::ExportDest` with its own title and hint. **Clean, no taste
        question.** ⚠️ Cheapest when no lane holds `render/**` — schedule it into a
        quiet tree rather than a wave.
     2. **No render edits at all:** reuse `OverlayKind::MoveDest` (already a
        folders-only destination navigator) with the export role on a new
        `OverlayState` field. **Cost: MoveDest's words must become role-neutral**
        ("move note" → "destination"), because a role-aware title is itself blocked
        in ≥3 held files — **a taste change to an existing verb's card.**
     **Recommendation: route 1.** It buys the right surface and asks nothing of the
     user; route 2 trades a held-file problem for a taste debt on an unrelated verb.

     ⚠️ **A THIRD COST THE PARKED ESTIMATE MISSED:** `runModal` runs on the process
     main thread, so a naive `Effect::Export` → panel makes `--screenshot-app` and
     `cargo test` **hang forever on a modal.** Any wiring gates on a real surface, as
     the reveal now does. (The panel call itself is ~12 lines of safe `objc2` fns — a
     structural twin of the shipping `pick_file_to_open` — so it is not the cost.)

     ✅ **`ellipsis_law` HAD A LIVE BLIND SPOT AND IT IS NOW CLOSED.** Its subject was
     `journey.card().is_some()` — an **in-app** card — but an `NSOpenPanel` opens in
     `App::handle_menu_event`, **above `apply_transition` entirely**. So `Browse
     files…` was passing **for the wrong reason** on macOS, and any row popping a
     modal with no ellipsis would have passed green. `NATIVE_PANEL_IDS` now declares
     that door as data with one predicate, and a second law requires every claimed id
     to sit on a row already promising a surface — **enrolment from the table, no
     `cfg!` in the law.** The flip is structural in both directions now.

     **Original:** **Export through the platform's own picker.** `mac_chrome::pick_file_to_open`
     already drives a real `NSOpenPanel`, wired at `app/menu.rs:56`, so the modal
     seam and its live-only caveat already ship — the parked save-dialog cost
     estimate should be re-derived, not inherited.
     **Build:** `NSSavePanel` on macOS defaulting to the document's folder and
     name; on Linux reuse the in-app browse picker in a save role — File → Open
     already routes there (`Action::OpenBrowse`), so the platform split exists.
     Reveal the file after writing, via `NSWorkspace` (already imported).
     ⚠️ Do NOT reach for an xdg portal: `rfd`'s Linux backend links GTK which this
     tree deliberately drops, a portal is a runtime service a self-contained
     tarball cannot depend on, and it would make export the only Linux verb using
     a system chooser.
     🔴 An in-app PDF preview is CLOSED, not deferred — a second document
     renderer. Do not re-propose.
     **Verify:** panel and reveal are live-only, flagged for human confirmation,
     never claimed from a capture; headless keeps its explicit path and stays
     byte-identical. **Routing:** deep tier.

302. **Loose comments — a second pass, a different class from 275's.** 275 removed
     narrated history; 287/288 removed citations. This is comments whose factual
     content contradicts the code. A history comment is noise; a loose one is read
     as truth and acted on.
     **Known instances:** `chrome/outline.rs` says "theme/caret/history pickers"
     when the crisp set is `Theme | Caret`; `prepare_notice`'s "can never have a
     notice"; `theme_font_debounce.rs`'s "12.0 ms on CLAUDE.md" against a fixture
     that grew 44%; `platform.md:88`'s "`~/notes` by default";
     `LocationStyle::RotatedRail`'s "small"/"muted"/card-flush.
     **Four shapes:** stale enumerations; stale "today" claims; baked
     measurements; and ⚠️ **invariants later code invalidated** — hunt these
     first, they are load-bearing, and the capture pipeline was DESIGNED around
     one of them.
     ✅ **The lever: a comment stating a checkable fact should be a LAW.** Prefer
     converting over rewording — a reworded comment rots on the same schedule.
     ⚠️ No grep for "wrong": read one at a time as 275 was; the list above is a
     shape guide, never a worklist. Prioritise comments claiming things about
     OTHER modules. Change no code except to add laws. Schedule against a quiet
     tree — same blast radius as 275's ~1000 sites.
     **Verify:** each new law mutation-proved. **Routing:** production tier.

303. ✅ **LANDED with 131e (merged 2026-08-06).** The mark mirrors to the row's
     outer edge off ONE `outward()` = `direction.sign()`, with the four pre-existing
     mirror multiplications routed through it; 284's rotation is fully removed,
     including both `DIMENSIONLESS` entries in item 242's count-asserting sweep. Five
     laws whose subject WAS the rotation are retired and six with a real subject
     replace them. 🔵 **The motion proposal is DECLINED with a reason:** a `Diagonal`
     world draws no selection band and ships `MotionJuice::CALM`, so there is no ease
     to ride and a glide means a new positional animator — the machinery the same
     commit removed. Original body:
          **The diagonal selection marker sits on the wrong side.** It belongs on the
     row's OUTER edge, away from the spine, mirroring between the two worlds:

     ```
     > item |          | item <
       item |          | item
        Magpie          Mangrove
     ```

     The cluster already mirrors (222/131d); the mark did not come with it.
     ✅ **The turn is dropped** — a plain upright `>` is the target. 284's
     rotation therefore loses its consumer and is REMOVED, not stranded: the
     `turn_deg` plumbing, the travel-direction source on `VisualSelection`, and
     the `step_*` term in `TextPipeline::advance`'s OR-fold. Retire its laws
     rather than deleting them blind.
     🔵 **Motion should stay — proposal, not decision:** let the mark ride the
     selection band's existing ease, gliding from the row it left to the row it
     reached. Direction becomes self-evident from the travel, which is what the
     rotation was for; it adds no machinery; and item 211's fix already
     guarantees a band ease gets its follow-up frame. Overshoot-and-settle is a
     cheap second option. Feel is live-only.
     **Scope:** the two `Diagonal` worlds; `Pane`/`Bars`/`Rules` do not move.
     **Verify:** the side derives from the same signed quantity that mirrors the
     cluster (`dx`/`dw` sign in the row planner), never a per-world branch;
     drawn↔hit-test agreement; 1×/2× across both worlds and every `OverlayKind`;
     byte-identity for 18 worlds. ⚠️ `diagonal.rs` is contended — check no claim
     before dispatch. **Routing:** deep tier.

306. ✅ **LANDED (merged 2026-08-06).** 19 of 20 worlds were affected, not a handful;
     Wagtail alone was unaffected because `#000000` is the curve's fixed point. Fixed at
     `to_wgpu`, renamed `to_wgpu_clear` — exactly one call site existed. Two laws: a
     256-value curve law and a drawn-frame `page_ground_law` using the page column's
     MODAL colour with a presence floor, because a probe pixel tuned to dodge lava,
     stars and stripe bands on twenty worlds today lands on one tomorrow. **Original:**
     **EVERY DARK WORLD'S PAGE GROUND DRAWS FAR LIGHTER THAN ITS AUTHORED
     `base_100`, AND THEMES.md IS THEREFORE WRONG ABOUT THE WHOLE GALLERY.** Found by
     item 296's lane while calibrating a plate against the page, and deliberately not
     fixed there — the blast radius is every world.

     **The mechanism, measured against real pixels on two worlds:** `LoadOp::Clear`'s
     colour is consumed as **LINEAR** by an sRGB-format target, while `to_wgpu()` hands
     it sRGB bytes straight through. So the page renders as the sRGB *encode* of the
     token rather than the token. **Currawong's authored `#060607` draws as `#2A2A2E`;
     Potoroo's `#1F0400` draws as `#622200`.** Light worlds are barely affected — the
     encode is near-identity up there — which is exactly why this survived.

     ⚠️ **This is why no fixed step off `base_100` can predict what a plate sits on**,
     and it is the reason 296's presence floor had to be perceptual ΔE rather than a
     luminance step. Any future work reasoning from an authored dark token is reasoning
     about a colour that is not on screen.

     **Verify:** the drawn page pixel equals the authored `base_100` for every world at
     1×/2×, swept over the roster — a law that would have failed on the day this
     shipped. ⚠️ **Expect the fix to CHANGE EVERY DARK WORLD'S APPEARANCE**, so it
     needs a visual-judge pass and the user's eye, not just arithmetic; several worlds
     were very likely tuned by eye *against the wrong ground*, which means the authored
     tokens themselves may now be wrong in the other direction. Read that risk before
     touching a token. **Routing:** deep tier, then the user.

307. ✅ **CLOSED 2026-08-07 — PREMISE FALSE, ORACLE REPAIRED. The gate was RIGHT.**
     `--capture-dpi N` makes a `WxH` DEVICE canvas mean a `(W/N)x(H/N)` **logical**
     window, so comparing two DPI tiers at the same device `--capture-size` compares two
     different logical windows — dpi 2 legitimately sees less page. **Grow the physical
     canvas in lockstep and the boundary is IDENTICAL at dpi 1, 2 and 3** (measure 70/71
     at 1200×800 logical, 49/50 at 900×700). Confirmed in the code too: `avail_chars =
     avail / label_char_w` carries `Metrics::scale` in **both** terms, so dpi cancels by
     construction. `gutter.rs` has zero diff.
     ✅ **The repair is a LAW, not a note** — dpi-invariance pinned at matched logical
     geometry over `measure 10..=100` × two windows × three tiers, with a non-vacuity
     clause requiring the boundary to be crossed. **That is what "oracle repaired"
     should always mean: the next reader measures instead of re-arguing.** Original:
     **THE GUTTER REPORTS `visible: false` AT DPI 2 WHERE IT IS VISIBLE AT DPI 1,
     at the same `--measure`.** Found by item 242's residual lane and **confirmed
     pre-existing** by stashing its own change, rebuilding and reproducing — so it is
     neither introduced nor fixed by that work, and it was flagged rather than chased.
     ⚠️ **Establish first whether this is a DEFECT or correct narrow-margin gating
     interacting with the scale factor** — the gutter has a legitimate visibility gate,
     and a capture at dpi 2 shows less logical content in the same canvas, so fewer
     margins may genuinely qualify. **Do not "fix" a gate that is right.**
     ⚠️ If it IS a defect it is invisible to every existing capture, because they all
     run at `--capture-dpi 1` — the same blind spot that hid items 289 and 292.
     **Verify:** the gutter's visibility derives from a logical quantity, swept at 1×/2×
     across the `--measure` range, with the boundary asserted on both sides.
     **Routing:** production tier.

308. **CASSOWARY'S FOOTER PLATE IS ΔE 1.91 FROM ITS OWN PAGE — below the ≈2.3 JND.**
     Revealed by item 306: the old absolute-luma gate aborted on Firetail before
     Cassowary was ever graded, so one world's failure was hiding another's.
     ✅ **The recommended repair is a RIM, not a token change** — item 296's notice
     channel clears ΔE 15 precisely *because* it draws a one-pixel rim, and this footer
     plate has never had one. **Verify:** the plate's presence floor swept over the
     roster; do not widen the gate — `assert_plate_separation_is_not_vacuous`'s own
     failure message names widening as the dishonest repair. **Routing:** production
     tier, then the user's eye on the rim.

309. **`thumb_ink` IS ONE `set_color` FOR EVERY RAIL, SELECTED OR NOT.** Named by item
     306 while fixing a different defect, and explicitly **not** the cause of that one —
     the lane retracted its own hypothesis after measuring (`on_band=[6]`: the failing
     rail WAS the selected one). So this is real, unmeasured, and independent. A rail
     that is not the selected row still takes the selected row's ink. **Build:** a second
     pipeline, with `overlay_bars`/`overlay_rows` as the precedent for the split.
     **Verify:** each rail's thumb clears a perceptual floor against **its own** ground,
     swept over the roster × selection state — the shape item 306's new
     `range_rail` law already uses. **Routing:** production tier.

310. ✅ **LANDED (merged 2026-08-06) — AND THERE WERE SIX, NOT FIVE.**
     `src/spellunderline.rs` carried a sixth whose own doc already said *"Identical to
     selection.rs"*. ⚠️ **The item's own byte-identity claim was FALSE as written:**
     calling the f64 owner and casting to `f32` mismatches a pure-`f32` evaluation of the
     same formula on **214 of 256 bytes, by up to 6 ULP**, because `powf` rounds
     differently at each width. Keeping the *caller's* width is not enough when the OWNER
     computes at another. Resolved by spelling the rule once in a `macro_rules!` body and
     instantiating it at **both** widths a real caller needs — one source, two honest
     numeric contracts. All 20 worlds' PNGs and sidecars byte-identical. **Original:**
     **FIVE COPIES OF THE sRGB EOTF, AND THE SCOPE IS ALREADY MEASURED.** `background.rs`,
     `lava.rs`, `render.rs`, `selection.rs`, `caret.rs`. Item 306's lane judged this a
     **bounded follow-up, not a wide refactor**, and gave the reason: each is one private
     per-channel loop with identical constants, differing only in float width and return
     shape (`[f32;4]`, `[f32;3]`, `Srgb→[f32;4]`), so routing them through
     `theme::srgb_channel_to_linear` is five mechanical edits **provable byte-identical
     by keeping each caller's own width.** It already removed a sixth copy and made the
     owner crate-visible, so the door is open. **Verify:** byte-identity per call site,
     plus a law with a no-wildcard match so a seventh copy cannot appear.
     **Routing:** production tier.

311. **`Diagonal`'s SECONDARY FLIP IS PROBABLY WRONG BY ITEM 306's OWN ARGUMENT** — a
     `Diagonal` world emits no row fill at all (`OverlaySelectionRects::default()`, by its
     own documented behaviour), so an ink chosen for a fill that is not under it can land
     on the page exactly as Firetail's thumb did. **Deliberately left alone by 306:**
     nothing is red, the appearance is unmeasured, and `chrome/diagonal.rs` was under
     concurrent change. ⚠️ **Measure before changing** — this is a hypothesis by analogy,
     and analogy is how three false premises reached this board this week.
     **Routing:** production tier.

312. ✅ **LANDED (merged 2026-08-07).** The extent stopped being a scissor: it arrives as a
     SHAPE (box, shear, feather) and rides in `fs_comp`'s alpha, with the scissor kept as a
     conservative bound so byte-identity beyond the frost holds **by construction on every
     backend** rather than resting on a blend round-trip against zero alpha.
     ⚠️ **A BARE PARALLELOGRAM WOULD HAVE MOVED THE DEFECT, NOT FIXED IT** — the card box
     IS the bounding box of the leaning rows, so its off-rake corners are exactly what the
     rows never reach, and they are **not empty**: the query line and foot hint are upright
     and flush left (~57px of hint over sharp document at the measured shear). The shape is
     therefore box **∪** sheared box, with the box named in code as a coverage FLOOR.
     ✅ **313 CAN DROP THAT FLOOR.** Once the hint and query lean with the rows, the union
     collapses to a true parallelogram — visible in `gallery/item-312/after-Mangrove.png`,
     where the hint sits upright at the foot. **That is 313's payoff beyond its own defect.**
     ⚠️ **AND 298, IF IT LANDS AFTER THIS, MUST CONSTRUCT A SHAPE, NOT A RECT:** an arm
     added to `frost_mode()` builds `Footprint { rect, shear }` and has to decide
     consciously whether a context menu leans. **Original:** **THE FOOTPRINT FROST'S EDGE IS A HARD RECTANGLE, AND UNDER A DIAGONAL LIST IT
     WANTS TO BE A FEATHERED PARALLELOGRAM.** User-reported against Mangrove's theme
     picker, with a screenshot: the frosted patch stops at a knife edge, and the world
     *already ships* a soft-edged version of the same idea a few hundred pixels away.

     **Premise-checked against the tree, and it holds by CONSTRUCTION rather than by
     mistuning.** `Frost::Footprint` is scoped with a **scissor rect**
     (`blur.rs:446` → `extent::scissor_px`); the composite target carries `blend: None`
     (`blur.rs:152`) and `fs_comp` returns alpha `1.0` (`shaders/blur.wgsl:87`). No value
     in that path can soften an edge. ⚠️ **`extent.rs`'s own module doc anticipated the
     alternative and dismissed it** — a rect uniform "would produce the same hard edge
     anyway". That is true of a rect uniform and FALSE of a feathered mask; the sentence
     is to be revised, not obeyed.

     ✅ **The user's "Mangrove already does this" names a real in-tree owner.** Mangrove
     is `Background::Lava`, whose field is masked `smoothstep(0, gap, …)` over
     `lava::MARGIN_GAP_PX` = **28.0** logical px at the column edge, and `lava_mask_2d`'s
     gutter carve is already the exact shape wanted — a bounded rect whose faces feather
     over `gap`. So the feather has a live precedent with an authored, tuned width, and
     the honest first question is whether the frost can borrow that quantity rather than
     author a second one.

     ⚠️ **THE ROSTER IS THREE WORLDS, NOT THE TWO IN THE REPORT.**
     `footprint_frost_applies` enrols on `!Card backing && !draws_row_plates`, which
     today is `Diagonal` **and** `Rules` — Mangrove, Magpie **and PAPERBARK**. The
     feather lands on all three. The parallelogram does NOT: shear is a `Diagonal`
     property, so Paperbark takes the soft edge and keeps its rectangle. **Derive both
     enrolments from the roster's own predicates; a name list would silently drop
     Paperbark and would not follow a world that changes list style.**

     **Build — the soft edge and the lean are ONE mechanism, not two.** Both need the
     extent to stop being a scissor: a footprint MASK in `fs_comp` (rect + shear +
     feather width through `U`), with alpha blending on the composite target.
     ✅ **`Frost::Full` can stay byte-identical without a second pipeline** — under
     `ALPHA_BLENDING` an alpha of exactly 1.0 is mathematically a replace — but that is
     an assertion to MAKE, not to assume. `Frost::Footprint([f32; 4])` grows the shear;
     the feather width is policy in `extent.rs`, not a per-call argument.

     ⚠️ **The lean must be READ, never re-authored.** `chrome/diagonal.rs` resolves the
     spine's per-row step (`ROW_STEP`) under a responsive bound
     (`TRAVEL_MAX_BAND_FRACTION`), so a cramped card gives up rake — a second copy of the
     constant would part company with the drawn spine at exactly the geometry a law
     forgets to sweep. Take the resolved composition.

     ⚠️ **`overlay_card_rect()` HAS A SECOND CONSUMER: pointer hit-testing**
     (`app/input/mouse.rs:359`, `:992`). Frost extent and hit region are one rect today.
     **Decide explicitly and say which in the commit** — recommended: the parallelogram
     is the FROST's extent alone, and the hit region stays the rect the rows occupy.

     **Verify:** the edge ramp measured across the footprint boundary in the PNG, swept
     over the enrolled roster **and at 1×/2×** — the reach is authored logical, so it
     must hold in logical px at both, which is precisely the class item 294 just fixed
     for the blur's own reach and which every `--capture-dpi 1` capture is blind to; the
     lean asserted against the DRAWN spine rather than a constant; and a **presence
     floor** beside the no-hard-edge law, because a feather that fades the whole
     footprint to nothing satisfies "no hard edge" perfectly (the floor-satisfied-by-
     deleting-its-subject trap).
     **Routing:** production tier, then the user's eye — feather width and lean are
     taste, and no capture settles either. Touches `render/blur*`, `shaders/blur.wgsl`,
     `pipeline_prepare.rs`, and READS `chrome/diagonal.rs` — **311 also names
     `diagonal.rs`; sequence, never pair.**

313. ✅ **LANDED (merged 2026-08-07) — and the mechanism already existed.** The item offered
     "its own run or its own buffer"; it is **neither.** `overlay_upload_text` already emits
     several `TextArea`s over the one `panel_buffer`, each clipped to a band with **its own
     `left`** — and the foot band was **already one of them**, never asked for a left of its
     own. **The emitter's diff is one token, zero net lines.** The whole tail moves as one
     block, because that band is chrome and not a list.
     ✅ The offset is READ, never authored: the lean from the rail's own `spine_step` (which
     carries the responsive yield), and **the hint's line found BY ITS TEXT** rather than by
     row-count arithmetic — the separator draws unconditionally while the row it is budgeted
     as can be dropped in the starvation degrade, so an index would drift.
     ⚠️ **The column clamp BINDS on Mangrove's theme picker** (359px of ink in a 496px
     column), so the band seats 40px short of the spine's line, right edge flush to the
     column. Better than flush-left by 137px, but not on the line. **Original:**
     **THE PICKER'S HINT LINE SITS FLUSH-LEFT UNDER A LEANING LIST.** ✅ **293 IS LANDED,
     SO THIS IS UNBLOCKED — and 293 changed NOTHING about the hint's shape:** it added
     lines, not a second run or buffer, so the hint is still in the same `panel_buffer`
     run with no independent x. **This item is neither easier nor harder than when it
     was written.**
     ✅ **AND IT NOW CARRIES A SECOND PAYOFF:** item 312 kept the card box as a coverage
     FLOOR for the footprint frost *because* the query line and this hint are upright
     and flush left — a bare parallelogram would have left ~57px of hint over sharp
     document. **Once the hint and query lean with the rows, that floor can be dropped
     and the frost becomes a true parallelogram.** See `gallery/item-312/after-Mangrove.png`. "type to filter
     ↵ keep esc revert" holds the card's left edge while every row above it rakes with
     the spine. Same user report and same screenshot as 312; separated because it is a
     different mechanism and a different contended file.

     **Measured:** the hint is pushed into the SAME `panel_buffer` rich-text run as the
     rows (`overlay_shape.rs:743` → `push_overlay_hint_spans`), so it inherits that
     buffer's left edge and has no independent x. Giving it a spine-derived offset means
     either its own run with an offset or its own buffer — **that shape choice is the
     item**, and it is why this is not a one-line move.

     ⚠️ **CONTENDS WITH 293 DIRECTLY — same file, same line.** 293 (the footer crowds
     the last row) already names `overlay.rs:248-297` and `overlay_shape.rs:743`. One
     lane, sequenced, **293 first**: a hint that is crowded is not fixed by moving it
     sideways.

     **Open design question the lane must put to the user rather than pick:** at the
     hint's own row the spine has ENDED — it spans the names above. So does the hint
     continue the lean past the spine's terminus, or sit at the terminal x? Two
     captures, the user's call.
     **Routing:** production tier, after 293, then the user.

314. ✅ **LANDED (merged 2026-08-07) — AND THIS ITEM'S OWN PREMISE WAS WRONG.** There is
     **no second role**: a 45-reader census found `TEXT_LEFT` is **logical everywhere**. The
     "subpixel-shimmer floor" duty does not exist at that call site — it belongs to
     `adaptive_column_left`'s closing `.floor()`, which floors whatever the policy returns,
     and `desired_left` is 244.96 and so fractional regardless. **Item 307 attributed the
     floor's job to the constant and the brief carried that forward.**
     ✅ **So the design call this item reserved for the user did not need them:** with the
     roles unanimous, splitting would create a constant with no members. The pad enrols in
     item 242's `Logical` family instead, whose newtype makes the bug **unrepresentable** —
     **the compiler then enumerated all 45 sites**, and
     `adaptive_column_left`'s `left_pad` PARAMETER is deleted in favour of `dpi`, so no
     caller can hand the policy an unscaled pad. **The bypass is closed rather than
     watched.**
     ⚠️ **WHY 307 COULD NOT SEE IT:** 307's law used `"hello world\n"`, so
     `outline_wants_rail()` was false and the policy was a passthrough. Every fixture here
     is HEADED. The configuration a check runs under is itself the hypothesis.
     ✅ **THREE MUTATIONS STAYED GREEN AND ARE FINDINGS, NOT GAPS:** the collapse floor is
     **double-guarded** (each site alone is compensated; only both together go red), and
     `page_min_margin`'s upper bound is **provably inert** — 0 of 3996 swept cells change if
     it is deleted. Both recorded in `docs/render.md` rather than left as green mysteries.
     ✅ **One law is documented as WEAKER than it looks:** drawn↔hit-test agreement survives
     every pad mutation *by design*, because both sides compose `text_left()`. Its doc says
     so instead of claiming a strength it lacks. **Original:** **`TEXT_LEFT` CARRIES TWO
     CONFLATED ROLES, AND THE ADAPTIVE COLUMN DRIFTS ACROSS
     DPI BECAUSE OF IT.** Found by item 307 while proving a *different* mechanism correct,
     and reported rather than widened into it.

     **Measured two ways.** `render/geometry.rs::adaptive_column_left_raw` adds the
     **un-scaled** `crate::render::TEXT_LEFT` (16.0) alongside dpi-scaled terms in
     `desired_left`/`min_left`, so **at matched logical geometry the outline's rail
     placement and the gutter's own `column_left()` move with DPI**: the gutter's
     visibility boundary flips a whole `--measure` step (1200×800 logical: dpi 1 → 75/76,
     dpi 2 and 3 → 76/77). Confirmed analytically —
     `desired_left_logical = 228.96 + 16/dpi`, matching the measured plateaus **244.96 /
     236.96 / 234.29** at dpi 1/2/3 — and empirically.

     ⚠️ **THIS IS NOT A UNIT SLIP AND MUST NOT BE FIXED AS ONE.** `TEXT_LEFT` is
     deliberately **physical** elsewhere: it is the subpixel-shimmer floor. Here the same
     constant is doing duty as a should-be-**logical** rail offset. **Splitting its two
     roles is a design decision** — do it before editing, and say which callers take which.
     ⚠️ **Blast radius:** `render/geometry.rs` is read by **caret, selection, hit-test and
     the drag handle**, so drawn↔hit-test agreement is part of the bar, not a bonus.

     **Verify:** the adaptive column's left edge is invariant across DPI at matched logical
     geometry, swept over the `--measure` range with the boundary asserted **on both
     sides** (a one-sided assertion passes on a gate that never turns on) — 307's new
     `gutter_visibility_boundary_is_dpi_invariant_at_matched_logical_geometry` is the shape
     to copy, and the experiment design is in 307's landing note. Plus drawn↔hit-test
     agreement at both tiers, and byte-identity at dpi 1 for anything not meant to move.
     **Routing:** production tier — but the role split is a design call, so surface it
     rather than picking silently.

315. ✅ **LANDED (merged 2026-08-07) — and widening the law's scope paid off on its first
     run.** Every `TEXT_TOP` reader was genuinely logical, as 314 found for `TEXT_LEFT`. The
     reason it survived is now named: **the idiom `TEXT_TOP + menubar_reserve()` had SIX
     independent spellings**, so the bug persisted at half of them even after
     `menubar_reserve()` existed. One owner now: `TextPipeline::text_origin_top()`.
     ✅ **The declaration law's SCOPE was the defect.** Widening it past
     `src/render/chrome/` to `geometry.rs`/`geometry/**`/`scroll.rs` immediately caught
     **three more instances** — `PAGE_RESIZE_GRAB_PX`, `IMAGE_RESIZE_GRAB_PX`, `MIN_IMAGE_W`
     — plus a genuine live bug in `page_scroll_rows`, which was taking device height minus
     an unscaled pad. **Original:** **`TEXT_TOP` IS THE UNTOUCHED VERTICAL TWIN — THE SAME DEFECT ON THE OTHER AXIS —
     AND THE REASON BOTH SURVIVED IS THE DECLARATION LAW'S SCOPE.** Named by item 314's
     lane after it closed the horizontal half.

     **`TEXT_TOP` (16.0) is still an untyped `f32`, read unscaled** by `doc_top`,
     `visible_lines_z` and `scroll.rs` — the identical shape 314 just fixed for
     `TEXT_LEFT`, one axis over.

     ⚠️ **THE STRUCTURAL FINDING IS BIGGER THAN THE CONSTANT.** Item 242's unit-family
     declaration law — the one that forces a chrome length to declare `Logical`,
     `Physical`, `Chars` or `Rows` — **is scoped to `src/render/chrome/` only.** That is
     why both of these lived in `render/geometry.rs` and `render.rs` untouched for their
     whole lives, and it is the same class as the residual 242 already carries (the law
     reads authored `const`s and not inline literals). **Widening the law's scope is the
     durable half of this item**, and it will name others: expect the census to be longer
     than one constant.
     ✅ **The remedy is proven and cheap:** enrol in the `Logical` newtype family, which
     makes the bug unrepresentable and lets the compiler enumerate the call sites — 314
     did exactly that for 45 sites, and deleting the offending PARAMETER (rather than
     scaling it at each call) is what closed the bypass.

     **Verify:** vertical placement invariant across DPI at matched logical geometry, both
     boundary sides, with a **presence floor** beside it — 314's own law is the shape, and
     it records that invariance alone is satisfiable by deleting the pad since `0 × dpi` is
     perfectly invariant. Plus drawn↔hit-test at both tiers and byte-identity at 1×.
     ⚠️ **Fixtures must exercise the branch:** 307 missed the horizontal twin because its
     fixture had no headings, so the policy was a passthrough. **Read what your fixture
     actually reaches before trusting a green sweep.**
     **Routing:** production tier. ⚠️ Touches `render.rs`, `render/geometry.rs` and
     `scroll.rs`; sequence against anything holding those.

316. **THE LOCATION ROW'S OWN BAR PLATE IS A VISIBLY EMPTY CHIP.** Exposed — **not
     introduced** — by item 297: it is **byte-identical in that item's before and after
     shots** (visible at ≈470,205 in `gallery/item-297/after-Cassowary-Files.png`), and the
     retired 15px whisper simply camouflaged it. Now that the cue composes off-card, the
     plate it used to sit on draws with nothing in it.

     ⚠️ **Establish the owner before changing anything, and note the neighbourhood is a
     recorded tripwire.** `overlay_prepare_bar_scrims`'s gate reads `backing == BarePlates`
     and **is correct as-is** (see this board's TRIPWIRE section — do not "fix" it to
     `draws_row_plates()`); `ListStyle::draws_row_plates()` is the one owner of whether a
     style backs its rows, `overlay_selection_rects` the one place a style becomes row
     surfaces. **Earn any exclusion by measurement** — the frame must emit no row surface at
     all for the excluded case, at the same fixture and DPIs — rather than by a name list.

     **The question to answer first:** should a location row that plans a glyph-free line
     get a plate at all? If not, the fix is in whoever decides a row's surface, not in the
     cue. **Verify:** no plate is emitted for a glyph-free location row, swept over the
     roster × `OverlayKind` × 1×/2×, with byte-identity everywhere a plate legitimately
     belongs. **Routing:** production tier.

317. ✅ **LANDED (merged 2026-08-07) — and the boolean instrument is now EXHAUSTED.** The whole
     suite is green under `MENU_BAR_ON`'s non-macOS arm (3867 = baseline), so the lane built a
     **sharper** one: bar on *and* `menubar_reserve()` **tripled**, separating laws that SEE the
     reserve from laws blind to it — 14 of 3867 see it, and **nine failed with a presence-floor
     message naming the collapsed subject**, which is the CI RED's lesson having taken. Two were
     measuring the wrong thing (a law hand-rolling its own viewport spent 18px of a 21px tolerance
     on its own error — **the product was correct**). ⚠️ **The probe was BLIND to two laws because
     an earlier arm of the same roster pinned the toggle off** — their configuration was a property
     of ITERATION ORDER. ✅ **And the census has ONE DOOR:** `MENU_BAR_ON` is the only
     platform-forked sticky default, and `src/render/` non-test has **zero** `target_os`.
     **Original:**
     **HOW MANY OTHER LAWS ARE BLIND TO THE `menu_bar` AXIS? SWEEP THEM UNDER THE
     FORCING.** The CI RED above was one platform default — `MENU_BAR_ON` is `false` on macOS
     and `true` everywhere else — costing **35.6px of every card's height budget**, and it hid
     **a picker that drew zero candidate rows on Linux.** Two laws were repaired and a handful
     gained the axis. **The open question is how many others never see it.**

     ✅ **The method is already proven and takes seconds, not a container:** force
     `MENU_BAR_ON`'s other branch in `src/menubar.rs` and run the affected filters.
     CLAUDE.md's own words — *"a fix that passes unforced has not been tested"*.
     **Build:** run the **whole** suite under the forcing, list every test that changes
     behaviour or fails, and for each decide whether it (a) genuinely needs the axis swept, (b)
     is legitimately platform-scoped, or (c) is measuring the wrong thing entirely — which is
     what the drawn-inset probe turned out to be.
     ⚠️ **Expect the third category.** A law written on a host where a reserve is always zero
     can encode that zero into its own geometry without anyone noticing, and this is the second
     time this week a probe was found measuring the canvas instead of its subject.
     ⚠️ **Do not add the axis mechanically to everything** — a law that gains a second axis it
     does not need doubles its cost for nothing. The deliverable is the CENSUS and the
     judgement, not a bulk edit.
     ✅ **And the same question applies to every other platform-forked default**, not just this
     one: the durable version of this item is a list of them and which laws sweep each.
     **Routing:** production tier. ⚠️ Touches `src/render/tests/**` broadly — schedule against
     a quiet tree.

318. ✅ **LANDED (merged 2026-08-07) — THE FROST IS A PARALLELOGRAM ON ALL THREE ENROLLED
     WORLDS, AND THE ROUTE WAS NEITHER OF THE TWO THE BRIEF OFFERED.** `footprint_dist_outside`
     was `upright.min(leaning)`, a union whose upright term was the card's whole box, so a
     parallelogram was impossible by construction at any shear. The lane BUILT the principled
     route (an independent `left` for the query line, 313's mechanism) and **backed it out**: that
     mechanism seats the field via the same `ColumnFlow` mirror the rows use, moving it **+461
     logical px** and right-aligning it — a text input whose `›` sigil travels leftward as the
     user types. An unbriefed product change to an input field, correctly refused. Instead the
     coverage duty left the MASK and moved into the SHAPE'S OWN WIDTH: the mask asks the sheared
     box alone and `footprint_box` widens the footprint's **rect** until the parallelogram contains
     the card's upright chrome. **Widening a parallelogram leaves a parallelogram**, so the
     silhouette pays nothing, no chrome moves, and `overlay_card_rect` and the pointer hit region
     are untouched. The clickable band never needed to move — that briefed premise was wrong too,
     because the clamp keeps the field inside the band the hit-test already accepts.
     ⚠️ **TWO OF THE THREE BRIEFED PREMISES WERE FALSE AND BOTH WERE THE ORCHESTRATOR'S.** The query
     band is not above the frosted region — it sits **12 logical px INSIDE the card's top edge**,
     so the floor's last named subject was inside the shape and the floor had not retired on its
     own. And the "+12 descending" figure was **the card's own TOP EDGE**, a vertical clearance
     that says nothing about the rake; the real horizontal clearances are **69.62/84.34** for the
     foot and **43.15 outside** on ascending. `docs/render.md` repaired. Premise (a) held —
     Mangrove confirmed from the roster, not the image.
     ✅ **HOW A PARALLELOGRAM WAS DISTINGUISHED FROM A RECTANGLE NUMERICALLY**, which is the part
     worth reusing: (1) both faces translate by the same `shear×(py−cy)` so the span's WIDTH is
     constant — the union moved one face at a time, so its span WIDENED away from the centre row;
     (2) frosted area falls short of its own bounding box by the two triangular corners
     `|shear|·h²` — zero for a rectangle, and zero for the union. In pixels: exactly two of the
     CARD's own corners short of full frost with real sharp document showing through, against
     **exactly zero** under the union. Presence floors kept: frosted area still `w×h` within 2%,
     head-band coverage measured **1.0000** against a **0.9** floor, zero document edges behind
     the band. 1×/2× and **both menu-bar arms** — the bar's 35.6px reserve moves the band's y
     from 64.0 to 99.6, so that arm is not decorative.
     ⚠️ **THE CORNER LAW WAS GREEN UNDER ITS OWN MUTATION ON FIRST WRITE** — aimed at the SHAPE's
     bounding box, whose off-rake corners the union's ears reach identically. Re-aimed at the
     CARD's box (a union always contains it) it goes red: *"this shape leaves []. ZERO of them is
     the rectangle the user photographed"*.
     ✅ **THREE DEFECTS FOUND BEYOND THE BRIEF.** 312's ramp law was **profiling the union's face
     rather than the drawn one**, off by up to `|shear|·h/2` on half of every leaning card —
     `footprint_face_x` is now the single owner (and the ten lines that bought back a tightened
     clippy exception). 294's `card_ink_mask` **is a veto and does not invert into an inclusion
     set**: its "blur of a blank page" premise holds only where the frost reaches, so it reported
     card ink 52 rows above the card, and intersecting it selects ground structure under the
     card's SHADOW (4004px Mangrove, 19584 Magpie, 8 Paperbark). Both oracles falsified on first
     run. Card ink ~42px outside `overlay_card_rect` on the diagonal worlds is **pre-existing and
     unchanged here** — filed to item 319.
     Byte-identical: Paperbark (enrolled, `Rules`, shear 0), Wagtail, Galah, Quokka. Mangrove and
     Magpie differ by design and their **sidecars are identical** — appearance, not state.
     `Frost::Full` holds via `the_full_frosts_composite_is_destination_independent`.
     **Original:** **THE FROST STILL READS AS A RECTANGLE
     UNDER A LEANING LIST, AND THE USER HAS CALLED THE SHAPE: IT MUST READ AS A
     PARALLELOGRAM.** A live screenshot of the theme picker over a `Diagonal` world: the
     frosted patch is an upright box while every row, the spine and the foot inside it rake.
     The user's words: *"the blur was not achieved… you can see how it's kinda like a square
     right? that's wrong… it should be like a parallelogram."* **312's open question — which
     of the two routes below, decided explicitly — is now ANSWERED BY THE USER'S TASTE, not
     by a lane's convenience.** The shape is the deliverable; the floor is whatever survives
     under it.

     **THE CAUSE IS STRUCTURAL, READ OUT OF THE CODE RATHER THAN THE PIXELS** (so this
     premise needs no reproduction to be believed, only the *magnitudes* do):
     `blur::extent::footprint_dist_outside` is `upright.min(leaning)` — a UNION whose upright
     term is the card's **whole box**. The card box is the bounding box of the leaning rows,
     so the union always CONTAINS the full rectangle and the shear can only add two overhang
     corners. **A parallelogram silhouette is impossible by construction today**, at any
     shear, on any world. The lean is real and is doing exactly what it was built to do; it
     is invisible because the floor drawn beside it is larger.

     **312's COVERAGE FLOOR CAN BE NARROWED — its remaining subject is ONE BAND ON ONE
     WORLD, measured.** Item 313 leaned the foot band, so the floor no longer covers it: the
     band now sits **+66 logical px inside** the leaning term on Mangrove and **+83 on
     Magpie**. But the **QUERY LINE on Magpie sits −71 px OUTSIDE** it, and Paperbark's shear
     is 0 so the parallelogram *is* the box there.

     ⚠️ **SO ON THE DESCENDING WORLD THE FLOOR ALREADY PROTECTS NOTHING — every drawn band is
     inside the leaning term (query +12, foot +66), and the rectangle the user photographed
     is pure floor with no remaining subject.** If that holds when re-measured, the
     descending world can read as a true parallelogram *without* the sibling item, and only
     the ascending world waits on the query line's independent `left`.

     ✅ **Two routes. The user's call makes the principled one the target, not the fallback:**
     give the query line the same independent `left` item 313 gave the foot — which **also
     has to move the amber query caret (`overlay_place_caret`) and the field's clickable
     band**, so it is a sibling item rather than a footnote, and once it lands the floor has
     no subject on either diagonal world and the shape is a parallelogram outright. The cheap
     route — union the parallelogram with the **header band's rect** instead of the whole box
     — is a strict shrink with no new mechanism, but ⚠️ **it leaves an upright cap across the
     card's full width at the head of the shape, so it does NOT by itself deliver the read
     the user asked for.** Ship it only as an interim, say so in the commit, and keep this
     item open behind it.

     🔵 **THREE PREMISES THE LANE CHECKS BEFORE BUILDING — two are the orchestrator's own
     reading of a JPEG and carry no privilege** (the board's own rule; six false premises in
     one day were all authored above the lane). **(a)** The world in the shot was identified
     from a lava ground and rows stepping RIGHT as they descend — i.e. **descending,
     Mangrove**. Confirm from the roster, not from the image; if it is the ascending world
     the −71px query line is live and the sibling item gates everything. **(b)** In the shot
     the query prompt and its amber caret appear to sit **ABOVE the frosted region's top
     edge entirely** — if the query line is outside the footprint rect vertically, the
     floor's last named subject may not be inside the shape at all, and the floor retires
     without the sibling item on **both** worlds. Measure the header band's rect against
     `overlay_card_rect`; do not assume either way. **(c)** Re-measure the ±66/±83/−71/+12
     figures against HEAD before relying on them — they are item 313's, taken before whatever
     has merged since.

     **Verify:** the shape law must state the POSITIVE claim the user asked for — the
     silhouette's two off-rake corners of the card box are **NOT** frosted at a real shear —
     beside the floor's existing negative one, that no drawn chrome sits over sharp document.
     ⚠️ **Both halves are needed and neither is sufficient:** a corner-exclusion law alone is
     satisfied perfectly by frosting nothing (312's own satisfied-by-fading trap, and the
     board's third green-over-a-collapsed-subject law this week), so it carries a **presence
     floor** — the row cluster's own band is fully frosted — and the chrome-coverage floor is
     narrowed, never deleted. Sweep the enrolled worlds from the roster (`Diagonal` both
     directions **and** `Rules`, whose shear is 0 and whose rectangle must stay
     byte-identical) × 1×/2×, and **under the `MENU_BAR_ON` forcing**, since the card's
     height budget — and therefore the rake the rail resolves — moves in that axis.
     **`docs/render.md`'s item-312 bullet and `blur/extent.rs`'s "THE UNION IS THE POINT"
     doc both argue for the floor in the present tense and must be rewritten with it**, not
     left contradicting the code. **Routing:** production tier, then the user for the live
     `--release` look — the final read is taste and no capture claims it.

319. **STILL OPEN — and item 318 added a SECOND, MEASURED INSTANCE: card ink exists ~42 logical px OUTSIDE
     `overlay_card_rect` on both diagonal worlds** (found at `(1102, 344)`, mask 0.03 **both before
     and after** 318 — so it is pre-existing and 318 changed nothing about it). That makes this
     item's subject two independent overflows of the same card, not one, and both were found by
     laws written for something else. **THE FOOT BAND'S INK ALREADY EXCEEDS THE CARD'S TEXT COLUMN AND IS CLIPPED — pre-existing,
     and a no-clip law does not cover those cells.** Found by item 313 while leaning it, and
     deliberately left exactly where it was found. Measured: **Mangrove/Command at 1200px is
     434px of ink in a 496px column** *including* the Keybindings tips, and Mangrove's theme
     hint leaves only 137px of room.
     ⚠️ **`overlay_footer_fit_probe`'s no-clip law apparently does not reach these world × kind
     pairs** — so establish what its enrolment actually covers **before** changing any geometry;
     the law's gap may be the whole finding. **Verify:** no foot-band run is clipped, swept over
     the roster × `OverlayKind` × 1×/2× **and under the `MENU_BAR_ON` forcing**, since the bar's
     reserve is in this axis. **Routing:** production tier.

320. ✅ **LANDED with 317.** `window_rows()` diverges from the flat `12` for exactly four kinds, so
     most of the 36 fixtures missing the field are correct **by accident**. Three laws changed
     verdict once set — **and the cause is a live product defect, not a fixture bug** (see 327), so
     the folds were backed out rather than landing a red suite, with the measurement recorded in
     code at each site. **Original:**
     **A LATENT TEST-FIXTURE BUG THAT MAKES HEIGHT-BUDGET SWEEPS LIE.**
     `ViewState::overlay_window_rows` left at its **default pins every kind to 12 rows**, so a
     law that believes it is varying the card's height budget is not varying anything. The
     capture path sets it from `OverlayKind::window_rows()`; **a law must too.**
     ⚠️ This is the same class as the CI RED above — a sweep whose own configuration is the
     untested part — and it is cheap to audit: grep every fixture that constructs a `ViewState`
     and check whether it sets that field. **Expect several.** **Verify:** each affected law
     re-run with the field set, and any that changes verdict named. **Routing:** production tier.

321. ✅ **LANDED (merged 2026-08-07).** The bar's LOGICAL height was 35.60 at 1×, 32.27 at
     1.5×, 30.60 at 2× and 28.93 at 3× — **the padding shrank as the display got denser.**
     Now 35.60 at every tier; **1× is byte-identical.** The census found ONE role across two
     call sites, so nothing to split, and the fix **closes the bypass**: `BAR_PAD_Y` is a
     `Logical` and `bar_height` takes a required `scale`, so — a `Logical` having no
     arithmetic but `.px(scale)` — no caller can pass an unscaled pad at all.
     ✅ It **reported a block rather than routing around one**: widening the declaration law
     to `src/menubar.rs` means editing a file the census lane held. **Original:**
     **`menubar.rs`'s `bar_height` MIXES A SCALED ARGUMENT WITH AN UNSCALED CONSTANT — the
     same defect shape items 314 and 315 just closed, one file over.**
     `bar_height(line_height) = line_height + 2.0 * BAR_PAD_Y`, where the argument is scaled and
     `BAR_PAD_Y` (5.0) is not. Found by item 315's census and left alone because `menubar.rs` was
     outside its partition.

     ⚠️ **This one is load-bearing in a way the others were not: the menu bar's reserve is the
     axis that caused this repo's gating CI RED** (a picker drawing zero candidate rows on Linux,
     because `MENU_BAR_ON` is `true` off macOS and its 35.6px comes off every card's budget).
     **So a wrong `bar_height` is wrong on every non-macOS host, in the quantity that starves
     cards.**
     ✅ **Item 315's own law is already written not to be fooled by this** — it reads
     `menubar_reserve()`'s **live value** rather than a hand-rolled formula — so fixing this
     will not silently invalidate it.
     **Verify:** the bar's height is invariant across DPI at matched logical geometry, and the
     card budget that derives from it likewise, **under the `MENU_BAR_ON` forcing** (the bar is
     hidden by default on the dev host, so an unforced run cannot see its own subject).
     **Routing:** production tier.

322. **`src/render.rs`'s REMAINING ~30 CONSTANTS ARE UNCLASSIFIED, and item 315 declined to
     guess — correctly.** Widening the declaration law to `render.rs` itself is the last step of
     items 242/314/315, but that file declares families the lane did not audit: caret and font
     sizes that already flow through `Metrics::with_dpi`'s own multiply (a different, correct
     pipeline), animation durations in **milliseconds**, raw alpha and lightness values — and
     **at least one genuine miss, `PAGE_TEXT_PAD_CHARS`, which wants `Chars`.**
     ⚠️ **The reason not to rush it is CLAUDE.md's own:** *a generated document states its wrong
     answer with a law behind it.* A constant mis-declared `Logical` gets silently multiplied by
     DPI forever, with a law asserting it is right. **Classify one family at a time, and for each
     say which pipeline already scales it.**
     **Verify:** the widened sweep green with every constant declared, plus byte-identity at 1×
     for anything whose family you assert is already handled elsewhere — that is the claim most
     likely to be wrong. **Routing:** production tier.

323. ✅ **LANDED (merged 2026-08-07) — THE CENSUS FOUND SIX, NOT FOUR, AND TWO ARE LEGITIMATELY
     PHYSICAL. The law's scope was the defect a third time.** Adding `src/menubar.rs` to item 242's
     swept set enumerated `BAR_INSET_X`, `TITLE_PAD_X`, `DROP_PAD_X`, `DROP_PAD_Y` **plus
     `EDGE_BLEED_PX` and `FLUSH_EPS`**, which no census above the lane had named.
     ✅ **Logical (4), each with the usage evidence that decided it:** `BAR_INSET_X` (the x the
     first title's glyphs draw from, added to offsets shaped at device metrics); `TITLE_PAD_X`
     (only the two OUTER band edges — every interior edge is a midpoint between device-scaled glyph
     extents and widens with DPI on its own, so a device-fixed pad shrinks the outer bands relative
     to the interior ones); `DROP_PAD_X` and `DROP_PAD_Y` (card width/height off `m.char_width` and
     a `Rows`-derived pitch, already-enrolled siblings in the same expressions).
     ⚠️ **PHYSICAL (2), declared with their reasons — the case the brief warned about, and it was
     real.** `EDGE_BLEED_PX` pushes a flush rect off-canvas far enough to hide what the
     **rasterizer** feathers, and both quantities are device-fixed: the fill shader antialiases
     with `smoothstep(-1.0, 1.0, d)` (~1px each side, framebuffer space) and `CORNER_RADIUS: 2.5`
     is uploaded once at construction, **never multiplied**. Scaling it grows overdraw on Retina
     while the thing it hides stays put. `FLUSH_EPS` is a half-**device**-pixel tolerance on "is
     this edge on the boundary pixel", not breathing room — scaled, a rect 1.4 device px clear on a
     3× display would start counting as flush, which is a different rect rather than a
     better-tuned one.
     ✅ **The law grading `EDGE_BLEED_PX` reads `CORNER_RADIUS` out of `src/selection.rs` rather
     than restating it, so the Physical classification's PREMISE is graded** — raising the radius to
     6.0 fails it by name. A presence floor sits beside the invariance check (zeroing
     `TITLE_PAD_X` fails: *"invariance alone is satisfied by a deleted pad"*), and the dropdown
     hit-test law now sweeps four DPI tiers instead of asserting at 1×. `drop_inner_origin` is the
     one owner of the card pad, read by both the drawn row grid and the hit-test. **Original:** **FOUR MORE BARE `f32` PADS IN `menubar.rs`, SAME SHAPE AS 321 — and adding that file to
     the declaration law's sweep is the fix that finds them.** `BAR_INSET_X`, `TITLE_PAD_X`,
     `DROP_PAD_X` and `DROP_PAD_Y` are still untyped, and they are **added to device-scaled glyph
     positions** in `chrome/menubar.rs` and `chrome/menubar/dropdown.rs`. Found by item 321's
     census, **unverified and deliberately untouched** — it had closed `BAR_PAD_Y` and would not
     absorb four more on a hunch.

     ✅ **Do the law first, then the constants.** `src/menubar.rs` is outside item 242's swept set
     (`chrome/**` + `geometry.rs` + `geometry/**` + `scroll.rs`, widened by item 315). Adding one
     path makes the compiler and the law enumerate the work — 315 widened the scope and caught
     three instances on the first run, and **a law's scope has now been the defect twice.**
     ⚠️ **Verify each is genuinely logical before scaling it** — 314 expected two roles and found
     one across 45 readers, but a pad added to a *shaped glyph position* may legitimately be
     physical, and getting that backwards multiplies it by DPI forever with a law asserting it is
     right. **Classify from usage, one at a time.**
     ⚠️ **The dropdown is a live-only surface on macOS** and the drawn bar is the non-macOS
     default, so **run under the `MENU_BAR_ON` forcing** — an unforced run cannot see the subject.
     **Routing:** production tier.

324. ✅ **LANDED (merged 2026-08-07) — THE AXIS THAT CAUSED THIS REPO'S GATING CI RED IS NOW SWEPT BY
     THE GATE.** `AWL_MENU_BAR_FORCE=on|off`, copied from `Convention::current()`: memoized
     `OnceLock` env read, a pure `classify_force` beside it so every input shape is unit-swept,
     inert unless set. **The knob moves the DEFAULT**, so `set_menu_bar_on`/`toggle` keep working —
     what an ambient-restoring fixture needs. `platform_default(is_macos)` is split out as a pure
     fn, which is what makes both arms gradable from one host.
     ✅ **Proven past the tests:** `--screenshot`'s sidecar reports `menubar.shown` `false`
     unforced, `true` under `on`, `false` under `off`; frame row 2 moves `(17,39,35)` →
     `(24,52,46)`.
     ✅ **MEASURED COST OF THE GATE ARM: ~1 SECOND, AND NOT ON THE CRITICAL PATH.** The arms ran at
     elapsed **10–12 s** against conventions ending at **308 s**; all four share the target dir, so
     Cargo's lock means only the first compiles. Either arm's failure suppresses the receipt with
     both statuses preserved.
     ⚠️ **WHAT THE ARM DOES NOT COVER, SAID PLAINLY:** it filters on test *names* containing
     `menubar`/`menu_bar` — **31 of 3889** — and a whole-suite census under a tripled reserve found
     **14 tests that observe the reserve without saying so in their name.** `--bin awl` means no
     integration targets. The receipt's own text is unchanged, so a filtered arm widens no claim.
     ✅ **`ci.yml` DELIBERATELY UNTOUCHED, and the reasoning holds:** CI's `linux` job calls
     `native-gate.sh`, so both arms already run there on real Linux + lavapipe, and the local gate
     runs both on real Metal. The one uncovered cell is **forced-ON render tests on virtualised
     Metal**, which sits inside `mac (render::tests)` — tolerated red by design. Adding the filter
     to the **gating** mac job would either gate on tolerated-red tests (8 of the 31 are
     `render::tests`) or need a second skip list that would drift, on a job that has run its
     30-minute ceiling out before.
     ✅ **The missing law was written**, reading the forcing each `cargo` invocation actually
     received: `on` and `off` exactly once each, and the canary plus both conventions with it
     **unset** — a convention that inherited a forcing would make the axis a property of the
     convention arms and sweep one branch twice. **Original:** **GIVE `MENU_BAR_ON` A FORCING KNOB AND A GATE ARM — this is the durable fix for the whole
     class, and the model already exists in the tree.** Item 317's closing words:
     *"today the only way to sweep the axis is editing a source file — which is why nobody ever
     did."*

     ✅ **Copy `Convention::current()` exactly.** It is **fully swept** via an
     `AWL_CONVENTION_FORCE` env knob **plus both arms run in `native-gate.sh` every gate**. Give
     `MENU_BAR_ON`'s initialiser an `AWL_MENU_BAR_FORCE` equivalent and add the arm. **Then the
     axis that caused this repo's gating CI RED is swept by the gate instead of by whoever
     remembers.**
     ⚠️ **Capture the AMBIENT value, never `cfg!`** — inside a test `cfg!(target_os = …)` reflects
     the host that COMPILED it, not the branch the value took, which is exactly the defect item 325
     is about.
     ⚠️ **Cost check before adding the arm:** `native-gate.sh` already runs two conventions
     concurrently; a third dimension multiplies, and CI's `linux` job has a `timeout-minutes`
     ceiling that has been hit before (a timed-out job reports as `cancelled`). **Measure the added
     wall-clock and say it** — sweeping one representative filter under the forcing may be the
     honest trade rather than the whole suite.
     **Routing:** production tier.

325. ✅ **LANDED (merged 2026-08-07) — the tautology is gone and BOTH halves fail on their own bug.**
     `Config::menu_bar_on` now defers to `menubar::menu_bar_default()`. The law is split: (a) an
     absent key gives the OWNER's answer, graded by the forcing arms; (b) the owner's two platform
     arms are **distinct, and which way round**, asked through `platform_default`'s explicit
     `is_macos` argument **so both are readable from a macOS host**.
     ✅ **The mutation this law had never survived, run: flipping `MENU_BAR_DEFAULT_OTHER` to
     `false` now fails** with *"the menu bar's two platform defaults must DIFFER — a bar that
     defaults the same way everywhere makes every `menu_bar` law on this host a claim about
     nothing, which is how a picker drawing zero candidate rows on Linux reached a gating CI run"*.
     ⚠️ **And restoring the accessor's own `cfg!` is GREEN UNFORCED, red only under the forcing** —
     that is the old tautology's exact blind spot, now covered. **Original:** **`src/config/tests.rs` ASSERTS A `cfg!` AGAINST THE IDENTICAL `cfg!` — a tautology that
     cannot fail on any host.** Found by item 317's census.
     `cfg.menu_bar_on() == cfg!(not(target_os = "macos"))` at `:596` and `:602`, while the subject
     is `config/model.rs:104`'s `self.menu_bar.unwrap_or(cfg!(not(target_os = "macos")))`.
     ⚠️ **Neither side reads the actual owner** — `menubar::MENU_BAR_DEFAULT_MACOS`/`_OTHER`. **Flip
     `MENU_BAR_DEFAULT_OTHER` to `false` and `MENU_BAR_ON` changes while this law stays green.** Its
     comment claims to check *"the platform default (ON web/Linux, OFF macOS)"*, which is precisely
     what it does not do.
     **Build:** route both the accessor and the law through the named consts, so the law grades the
     owner. **Verify:** mutation-prove by flipping the const and watching it go red — that is the
     whole point, and it is the mutation this law has never survived. **Routing:** production tier.

326. ✅ **LANDED (merged 2026-08-07) — one owner, and the law refuses a new bypass rather than
     trusting visibility.** Owner: **`TextPipeline::menubar_reserve`**, holding the `menu_bar_on()`
     gate and the LABEL-scaled line height; `chrome/menubar.rs` draws at `self.menubar_reserve()`
     inside the branch where the bar is known on, **so the reserve IS the strip**.
     `menubar::bar_height` narrows `pub` → `pub(crate)`. Consumers routed through it: the document
     inset, the pipeline `hit_test`, the scroll viewport, `rects.rs`'s `card_y`, `layers.rs`,
     `overlay.rs`, `overlay_shape.rs`, `rotated_location.rs`, `debug_text.rs`, `readout.rs`, the
     capture sidecar — and now the drawn strip.
     ✅ **Visibility alone would NOT refuse a bypass** (any `src/render` file could still call it),
     so a law sweeps every non-test `.rs` under `src/` with a **no-wildcard** match, allow-listing
     exactly one file with its reason. **Non-vacuous in three directions, all three watched
     failing:** the drawn strip re-spelling `bar_height` itself, a stale allow-list entry (*"remove
     the entry rather than leaving a closed bypass looking open"*), and an empty walk.
     **Original:** **THE MENU BAR'S HEIGHT HAS TWO OWNERS, agreeing only by coincidence.**
     `render/geometry.rs:711` (the reserve) and `render/chrome/menubar.rs:82` (the drawn strip) each
     independently spell `bar_height(metrics.line_height * type_scale::LABEL)`. Item 317's ×3 probe
     drove a wedge between them and `chrome_panels::menu_bar_left_and_right_columns…` immediately
     read the drawn bar as non-ground.
     ⚠️ **Same shape `geometry.rs`'s own doc records for `TEXT_TOP + menubar_reserve()`, which had
     SIX spellings and is why item 315's bug survived at half of them.** One owner, every consumer
     routed through it, and a law with a no-wildcard match — the standing rule.
     ✅ Item 321 already made `bar_height` take a required `scale`, so the two call sites now agree
     on units; **this item is about them agreeing by CONSTRUCTION rather than by both remembering to
     call the same function.** **Routing:** production tier.

327. **A `Range` SETTING LOSES ITS RAIL AT THE SHIPPED `window_rows = 31`.** Measured by item 317
     at 1200×800: the Settings card draws **22 candidate display lines in a 718.8px card**, the
     selected Zoom row **is planned and drawn** (`sel_row = 6 < lines = 22`), and **`overlay_rails`
     emits no rail for it** — the wider drawn set grows the diagonal cluster's label/value columns
     until `rail_geom` cannot seat a rail in what remains.
     ⚠️ **This is the SHIPPED configuration, not a fixture artefact:** `sync_view` and both capture
     paths set 31. It was invisible because fixtures left `ViewState::overlay_window_rows` at its
     flat default of 12. `settings_row_reach_law` fails identically at
     `world=Mangrove dpi=1 logical_width=640 setting=PageWidthProse`.
     🔵 **A live-`App` confirmation is OWED before this is treated as fact** —
     `overlay_accept:Settings` is `Unsupported` on the ordinary capture path
     (`docs/harness-reach.md`), so it needs `--screenshot-app`. **Treat 317's measurement as a
     hypothesis until then.**
     ⚠️ It is a product/taste call about the accessory cluster's width budget — items 318/319's
     neighbourhood — so **sequence with them and put the budget question to the user.**
     **Routing:** production tier, then the user.

328. ✅ **LANDED (merged 2026-08-08) — and the real fix was a SECOND accessor, not a changed
     signature.** `density()` is exhaustive now, so a new density-bearing variant must be
     classified before it compiles. ✅ **The return type stays `f32`, argued from the callers:**
     `background_desc` and its production-parity test mirror both flatten every variant into a
     shader uniform, following the same inert-zero convention `profile_mode`/`tunnel_mode` already
     use on sibling fields; the other five sites ask `density()` on already-known-`Zigzag` worlds
     and do direct arithmetic. `Option` would have added the struct's only `Option` field plus
     `unwrap_or` at both flattening sites for nothing.
     ✅ **`bears_density()` is the actual answer to "the enrolment is more truthful than its
     accessor",** because **`density() == 0.0` cannot answer "does this ground carry the field at
     all"** — a density-bearing world can author its dial to zero, and item 118's presence floor
     depends on telling those apart. The private copy the density sweep carried is gone; it asks
     the owner, so enrolment and accessor share one exhaustiveness discipline and cannot drift.
     ✅ **The compile-time claim was PROVEN, not asserted:** a throwaway variant was added, `cargo
     check` failed at exactly the two arms, and it was removed. **Original:** **`Background::density()` IS AN OWNER THAT ANSWERS "NO DENSITY" FOR A GROUND THAT HAS ONE.**
     `theme/ground.rs:365` ends in `_ => 0.0`, so a new density-bearing `Background` variant
     compiles, ships, and reads as density **zero** through the one accessor every consumer asks.
     Found by item 118 while building the density sweep: the sweep's enrolment could not route
     through this owner *because* of the wildcard, and had to spell an exhaustive match of its own
     to stay honest — **the enrolment is now more truthful than the accessor it was supposed to
     derive from**, which is the wrong way round.
     ✅ **Build:** make the arms exhaustive (return `Option<f32>`, or a `match` with no wildcard so
     a new variant fails to compile). `with_half_density` in `render/tests/backgrounds_item158.rs`
     is the shape to copy — it is already exhaustive for exactly this reason.
     ⚠️ **`density()`'s callers may rely on the `0.0` for non-density grounds**, so this is a
     read-every-caller change, not a signature flip. Grep before deciding the return type.
     **Verify:** a law with a no-wildcard match, and check the sweep in
     `backgrounds_item158.rs` can then be simplified to ask the owner. **Routing:** production tier.

329. **294's `card_ink_mask` IS A VETO AND DOES NOT INVERT INTO AN INCLUSION SET — two frost
     oracles were falsified on first contact.** Measured by item 318. Its "a blur of a blank page"
     premise holds only **where the frost actually reaches**, so as an inclusion set it reported
     card ink **52 rows above the card**; and intersecting it with an open-vs-closed difference
     then selects ground structure under the card's **SHADOW** (4004px Mangrove, 19584 Magpie, 8
     Paperbark — note the third is nearly clean, so a one-world check would have missed it).
     318's own laws were re-aimed at the production owner's declared box instead. **This item is
     the remaining audit:** find every other reader that treats this mask as "where the card is"
     rather than "where the card is not", since two of the first two examined were wrong.
     ⚠️ **This is an ORACLE defect, not a product defect** — nothing a user sees is wrong, so the
     close condition is "every caller checked and the mask's contract documented at its
     definition", not a pixel change. **Routing:** production tier.

330. ✅ **LANDED (merged 2026-08-08) — ⚠️ THE ITEM'S PREMISE WAS WRONG ABOUT THE CAUSE, AND THE
     PREMISE WAS MINE. THE PROBE WAS NOT HANGING — IT NEVER RAN.** Per-step timestamps from the
     API: `Install system dependencies` took **21m41s** against a same-day neighbour's **14s** for
     the identical step, and the probe step is recorded **`skipped`**, because `cargo build` was
     still compiling when the 30-minute ceiling cancelled the job. **A rare `apt-get` stall against
     the package mirror consumed the entire budget before the thing I suspected got a chance to
     run.** (That the mirror is *why* apt stalled is inference; every timestamp above is read from
     real job data.)
     ✅ **AND THE RATE WAS MEASURED RATHER THAN ASSUMED: 1 occurrence in ~37 runs on `main`**, not
     every run as the item implied. Every other `cancelled` conclusion in that population is a
     **supersede**, and every genuine probe failure completes in 8–15 minutes. **So the probe's own
     watchdog was left alone** — no run in the sample shows it firing, and there is no evidence to
     act on. Item 257 still owns the failure.
     ✅ **Both `apt-get` calls are now bounded**, turning a repeat stall into an ordinary tolerated
     `failure` within minutes instead of a job-dominating `cancelled` at the ceiling.
     ⚠️ **FIXED AT MERGE, and this is the reusable part: the wrapper was `timeout 5m sudo apt-get`,
     which puts `timeout` OUTSIDE `sudo`.** `timeout` kills by signalling the child it forked;
     outside `sudo` that child is the setuid binary running as **root**, and an unprivileged
     `timeout` signalling a root process gets **EPERM** — the timer fires, the kill fails, and the
     wrapper waits exactly as long as the stall it was added to bound. Reordered to
     `sudo timeout 5m apt-get`. **On a path that fires once in ~37 runs, a wrapper that cannot fire
     is worse than none: it reads as protected for months, then does nothing on the one run that
     needed it.** **Original:** **`atspi` NOW TIMES OUT RATHER THAN FAILING FAST, AND IT MAKES A GREEN TRAIN READ
     `cancelled`.** Measured 2026-08-07 on `ba292f75`: `atspi` ran **30m20s** into its
     `timeout-minutes: 30`, and because a **timed-out** job cancels the run's conclusion even under
     `continue-on-error`, the run reported `cancelled` while **all four gating jobs succeeded**.
     It used to fail fast. ⚠️ **The orchestration hazard is recorded in `.orchestrator/README.md`
     (read per-job conclusions, never the roll-up), so this item is the COST, not the confusion:**
     a tolerated-red job burning 30 minutes of every run is 30 minutes CI cannot spend on the
     `linux` job that is the only real-Linux arm, and `linux` has hit its own ceiling before.
     ✅ **Build:** either make the probe fail fast again (it is asserting on a bridge that is
     absent, which should be answerable in seconds) or lower its `timeout-minutes` so it cannot
     dominate a run. **Do not simply delete the job** — item 252's bridge liveness is the thing it
     exists to watch. ⚠️ Note item 257 already owns the *failure*; this is about its DURATION.
     **Routing:** production tier.

331. **THE THREE EXPORT COMMANDS' CATALOG DESCRIPTIONS NO LONGER MENTION THAT YOU CHOOSE A FOLDER.**
     Reported by item 301's lane rather than ridden into its bundle, which was the right call.
     *"Export the buffer to a `.docx` file; markdown buffers only"* is still **true** — it is
     simply no longer the whole story now that export asks WHERE first. ⚠️ **The cost is why this
     is its own item:** updating them requires regenerating `REFERENCE.md`, which
     `src/reference.rs`'s drift laws hold to the tree, so a one-word edit lands as a doc
     regeneration.
     ⚠️ **Sequence behind item 273's residual (1)** — that lane is restructuring the flag roster and
     the reference emitters, and this touches the same generated document.
     ⚠️ **Docs voice applies** (matter-of-fact, facts traced to verified sources), and the
     description is user-facing text in the palette. **Verify:** the regenerated `REFERENCE.md`
     diff is exactly the three descriptions and nothing else — a generated document that moves more
     than the change explains is the sourcing hazard, not a formatting nuisance.
     **Routing:** production tier.

332. **`--menu-open` AND `--pack-icns` SWALLOW THE NEXT ARGUMENT UNCONDITIONALLY, so
     `awl --menu-open file.md` SILENTLY EATS THE FILE.** Found by item 273's flag-roster lane and
     **deliberately preserved byte-identical** rather than ridden into that bundle — the right call,
     because fixing it changes behaviour a user could be relying on.
     ⚠️ **Both flags declare an OPTIONAL operand and then take the next token whatever it is**, so
     there is no way to pass a file alongside either. The roster now makes the arity explicit and
     law-tested, which is what makes this reportable at all.
     ✅ **Build:** an optional operand should decline a token that looks like a path or another flag
     — but "looks like" is the whole design question, and `--pack-icns DIR` legitimately takes a
     path. **Prefer the rule that a following token starting with `-` is never consumed**, and
     decide the path case explicitly rather than by heuristic.
     ⚠️ **This is a product change to CLI behaviour.** State the before/after for both flags and
     which invocations change meaning. **Routing:** production tier, then the user if any real
     invocation changes.

333. ⚠️ **PREMISE FALSE, RECORDED SO IT IS NOT RE-RAISED: the site's analytics beacon is
     DELIBERATE, DOCUMENTED AND THE USER'S OWN.** Item 273's lane flagged `//gc.zgo.at/count.js` in
     `site/reference.html` as sitting oddly beside a "no analytics" note. It is a **cookieless
     GoatCounter** beacon with its own section in `site/README.md` naming the dashboard, present on
     all eight site pages and on `main` since the reference page landed. ✅ **awl's zero-network
     invariant governs the APP BINARY — never phoning home, never fetching at runtime — not the
     marketing website**, and conflating the two would have produced a "fix" that removed something
     the user configured on purpose.
     🔵 **One real residual, small:** `site/README.md` says the beacon *"lives in three places
     (keep them in sync)"* and it is now on **eight**. The doc is stale, not the beacon.
     **Routing:** production tier — a doc correction, ideally with the count derived rather than
     restated.

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

## Release blockers and reminders

- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user's explicit word. A dry run may precede them.
- **Exactly one `native-gate-receipt` appeared in one 30-commit stretch.** The
  standing fix — **put the receipt in the MERGE COMMIT** — is not being followed
  reliably, and the tree once carried an unverified accessibility fix on `main`
  as a result. The process gap is the finding, not the code.
