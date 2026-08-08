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
- 🔵 **A THIRD WIDTH-FAMILY QUESTION, and it is vertical rather than horizontal (item 347).** At the
  **authored zoom maximum (300%)** in the two smallest windows the app allows (464×288, 520×400), a
  workspace stage plans **no rows and draws no other region** — a card carrying no list at all. **7 cells,
  all inside bounds the product itself enforces**, so the corner is reachable. **Should a workspace keep a
  minimum card — one line of whichever region it is showing — or refuse to enter a stage with no room for
  one?** ⚠️ **This is neither 327's nor 342's:** 327 asks which lane yields inside a card that exists, 342
  asks whether the width cap should be font-aware, and this asks what happens when there is no room for a
  line at all. It is held by a two-sided ledger meanwhile, so whichever way you answer, the law reddens and
  its entries get deleted rather than quietly kept.
- 🔵 **THE FIND/REPLACE PANEL IS HALF ITS TUNED SIZE ON YOUR RETINA SCREEN (item 174's slice 3).**
  Its outer margin and inner pad are **raw device-pixel constants**, so the card's `y` and its text's `top`
  are **identical at 1× and 2×** while everything around them doubles. That is `CLAUDE.md`'s own recorded
  tripwire — *"chrome padding shipped at half its tuned size on every Retina display"* — **live in the
  product again, in a different surface.** ⚠️ **Not fixed, because it is layout policy and the 1× look is
  what was tuned**: scaling it makes the panel visibly roomier on every Retina display, which is your call
  in the same family as the card-width cap. ✅ **The law is already fix-tolerant** — it asserts agreement
  with the drawn rim at each scale independently, so it stays green the moment the pad is scaled.
  ⚠️ **Second, smaller:** the panel's card has **no clamp to the window** — its x reaches **−33 at 560px**,
  so it hangs off the left edge rather than sliding into view.
- 🔵 **THE MAGPIE MARK: THREE CANDIDATES AND A RECOMMENDATION, your call (item 346).** You said it
  *"needs to be thinner and more elegant"* — and **"thinner" was already answered before you asked**:
  131e cut the drawn ink from 136 px² to 60 at 1×, a 56% reduction. So what is left is **form**, and the
  captures stack the options top-to-bottom in `gallery/item-346/compare-magpie-1x-before-A-B-C-D.png`
  (and the 2× pair), with 6× crops of the mark itself.

  | | weight | reach | aperture | vertex | ink 1× | peak ΔE 1× |
  |---|---|---|---|---|---|---|
  | **before** (what you saw) | 3.00 | 5.0 | 1.00 | 98.5° | 136 px² | 89.0 |
  | **A** shipped today | 1.25 | 4.5 | 0.55 | 70.7° | 60 px² | 59.3 |
  | **B** slender *(recommended)* | 1.25 | 5.5 | 0.45 | **50.8°** | 64 px² | **65.6** |
  | **C** compact | 1.25 | 3.75 | 0.62 | 87.6° | 54 px² | 58.0 |
  | **D** thinner stroke | 1.00 | 4.5 | 0.55 | 70.7° | 53 px² | 54.6 |

  ‼ **USE THE NEW ARTIFACT, NOT THE OLD ONE.** The first comparison image could not do its job — a vision
  smoke could tell "before" from all four candidates but **not A from B from C.** Root cause: each
  candidate was cropped to **its own** bounding box at ~77×39 device px and upscaled 6×, so the vertex
  angle — the thing that actually differs — was sub-pixel. **Rebuilt with a FIXED crop window shared by all
  five, upscaled 20× nearest-neighbour, ordered by vertex angle DESCENDING so the comparison reads as a
  monotonic closing of the angle**, and annotated with each candidate's own numbers:
  **`gallery/item-346/compare-magpie-mark-1x-vertex-desc-before-C-A-D-B.png`** (and the `2x` twin), with
  **`compare-magpie-row-{1x,2x}-…`** showing the same five at **1:1 in their own row** — because a mark that
  reads well magnified can still vanish in place, and you are judging both *elegant* and *findable*.
  ✅ **Confirmed by looking: B and C are now trivially distinguishable** (50.8° visibly slender against
  87.6° visibly blunt) at both scales in both views. ⚠️ **A and D stay close on purpose** — only 0.25px of
  stroke separates them — which is itself the board's own argument for not picking D.
  ✅ **B, and the argument is read out of the product rather than from taste:** weight has bottomed out at
  1.25, so **D is the trap** — it is the only candidate that spends the remaining budget on stroke, and it
  is measurably the *weakest* at 1× (ΔE 54.6, lowest of the four) for a difference invisible at 2×. B
  instead closes the vertex from 70.7° to 50.8°, which against Bitter's slab-serif register reads as a
  **slender drawn reference mark rather than a UI arrow** — and it is simultaneously the **most present**
  thin candidate at 1× (ΔE 65.6, above even shipped A), because its arms lie nearer the pixel grid. That
  is the one direction where "thinner" and "still there" do not trade off.
  ⚠️ **C is the honest third option if B reads too wide** — smaller in both dimensions, but its vertex
  opens to 87.6°, back toward the blunt mark that drew your complaint. ✅ **Landing B is a one-line change
  to `DiagonalMark::HAIRLINE`, and Mangrove is provably untouched by it.**
- 🔵 **THE RIGHT-CLICK MENU NO LONGER DIMS YOUR DOCUMENT, AND THIS IS A SURFACE YOU OPEN
  CONSTANTLY (item 298).** `gallery/item-298/before-Tawny-menu.png` → `after-Tawny-menu.png`: before, the
  whole page frosted behind four rows under the pointer; after, **no frost at all**, because Tawny's card
  already backs its own rows. `before-Mangrove-menu.png` → `after-Mangrove-menu.png` is the other answer:
  Mangrove draws no panel and no plates, so the menu frosts **its own footprint only** — without that, its
  rows would interleave with your prose glyph-for-glyph. `reference-*-room-takeover.png` shows the same
  rows summoned to the middle of the room, which is the full takeover this is judged against.
  **Before/after differs on 79% of Mangrove's pixels and 76% of Tawny's**, so it is not subtle.
  ⚠️ **Revert is one line if it reads worse to you.** ⚠️ **And one member is now out of step:** the
  contextual **spell popup** still takes no frost on any world, including the three bare ones — so it has
  the interleaving problem the menu just stopped having. DESIGN §5 says it "recedes nothing" on purpose,
  so **whether that stays is your call**, not a defect I should quietly fix.
- 🔵 **THE CARET FIX IS BUILT AND WAITING ON ONE WORD FROM YOU — captures are in
  `gallery/item-345/` (item 345).** Open **`currawong_before_after.png`** and
  **`cassowary_before_after.png`**: each is a 2×2 grid (1× and 2× rows, before and after columns),
  cropped and upscaled around a real word with **a red line at the glyph cell's own right edge**, so the
  overhang and its absence are actually visible rather than a 2.4px change lost in a full window.
  **Measured:** 14.400 → 12.000 at 1×, 28.800 → 24.000 at 2×, on both worlds; every other world is
  untouched. ✅ **The branch `claude/item-345-caret-pitch` is ready to merge in one command** — fix, law
  and mutation proof all done, and the floor it removes was measured redundant on the empty line,
  end-of-line, a degenerate cell and a tab.
  ⚠️ **The question is only whether the narrower caret still reads as awl's one loud thing.** Say the
  word and it lands; say no and the branch is dropped. **The old width was 120% of the letter and
  overhung 2.4px into the next character**, so "leave it" is the one answer with a real cost.
  **Original framing: fixing it makes the caret NARROWER, which is why it is yours (item 345).** Those two worlds use **Iosevka**,
  whose glyph cell is **0.50 em** where the other three bundled monos sit at 0.60–0.62. A historical
  floor raises the block caret to a fixed width regardless of face, so it draws **14.400px over a
  12.000px cell — 120% of the letter, overhanging 2.4px into the next character at zoom 1.**
  ⚠️ **Every other mono world matches its cell exactly**, so this is those two worlds alone.
  🔵 **The fix removes the floor, which makes the caret on those worlds visibly THINNER** — correct
  against the glyph, but a change to the one thing in awl that is allowed to be loud. Worth seeing the
  before/after on both worlds at 1× and 2× before it lands. Nothing else changes.
- 🔵 **FOUR PUBLIC DOCS STATE THINGS THE CODE CONTRADICTS (item 344), and two are counts a reader
  will notice.** `GUIDE.md` says **"Nineteen worlds, one chord away"** and `ACCESSIBILITY.md` says
  **"14 curated theme worlds"** — the roster is **20**, and `WEB.md` already says 20, so the docs
  disagree with each other. `docs/platform.md` documents the git-editor door as **"Finish Buffer"** with
  a chord whose default was **retired**; the palette calls it **"Finish file"**. And `GUIDE.md`'s conceal
  description says reveal happens on the caret's line, omitting that **a selection also reveals** — a law
  already proves it does. ⚠️ **I verified all four against source myself**; the other 25 in that census
  are reported but unverified. **No taste call here — just telling you what a reader currently sees.**
- 🔵 **TWO WORLDS CUT OFF THE END OF THEIR OWN HELP LINE, AND EVERY HONEST FIX IS A TASTE CALL
  (item 342).** On **Potoroo and Firetail** the Keybindings card's hint clips: at 2× it reads
  `esc clos` and `esc clo` instead of `esc close`. `gallery/item-342/Potoroo-keybindings-1x.png` →
  `Potoroo-keybindings-2x-CLIPPED.png`, and the Firetail pair. Measured: the trailing margin collapses
  to **0.11×** of its 1× value and the final glyph ends on the column's scissor rather than its own
  terminal.
  ⚠️ **It is NOT a 2× bug — zoom 1.0 on a 1× display clips identically.** The card's width cap is
  `LogicalGrowOnly(520)`, and that family keeps its **device** width below scale 1, so **the shipped
  0.8 zoom is 25% roomier relative to its own text than anything a Retina user sees.** Same shape as
  item 321.
  🔵 **Four options, none mechanical, and the fourth may subsume the first:**
  1. **Widen the cap 520 → 545** logical (+4.8%) — measured to clear both; 540 does not. **Widens the
     calm card on every world at every zoom.**
  2. **Give the hint band a yield.** Rows already elide and the accessory column already yields —
     **the hint band is the one line in the card with no yield mechanism at all.** Eliding hides a
     discoverability segment; wrapping costs a row.
  3. **Leave it ledgered** — two worlds lose three characters of a help line at zoom ≥ 1.
  4. ⚠️ **Decide which scale tier the cap is TUNED at.** If the 0.8/1× look is the intended one, every
     Retina user is already seeing a tighter card than you designed — and fixing *that* may fix this.
  ⚠️ **This is upstream of the Settings width question already owed to you** (item 327): it changes the
  width that one has to divide. Worth deciding together.
- 🔵 **THE RIGHT-CLICK MENU'S GREYED-OUT LABELS WERE INVISIBLE, AND NOW ARE NOT (item 299).**
  `gallery/item-299/before-context-menu-Wagtail-selected0.png` → `after-…`. Before, the Cut row showed
  no "unavailable" at all while Paste wrongly showed one it had not earned, because every secondary
  label sat one row below its own. On Wagtail the misplaced ink measured **ΔE 0.0 from its ground —
  byte-identical, literally invisible.** After, Cut and Copy each read legibly and Paste and Select all
  correctly show nothing. Worth a glance to confirm "unavailable" reads as quiet rather than as broken.
- 🔵 **SIXTEEN TUNED QUANTITIES ARE HALF-SIZE ON YOUR RETINA SCREEN, AND FIXING THEM IS A TASTE
  CALL, NOT A UNIT ANNOTATION (item 338).** The spell squiggle's amplitude and period, the nit and
  table rule thicknesses, the inline-code pill's insets, the fence panel's inset, table cell padding
  and column gap, the pan bar, the I-beam width, the space-bar caret, the caret morph dilation, and the
  overlay's entrance drop are all multiplied by **zoom alone and never by DPI** — so they hold their
  *device* size as the display gets denser. **Measured exactly:** the code pill is 6.0000 short at 2×,
  which is precisely twice its own inset constant; the fence panel 16.0000, twice its own.
  ⚠️ **Nobody should just "fix" this**, which is why the lane that found it stopped: several of these
  carry *"TASTE TUNABLE, flagged for live review"* in their own doc comments, and doubling them at 2×
  changes sixteen deliberately-tuned appearances at once. **1× is unchanged either way.** What this
  wants is your eye on 1×/2× pairs per construct — and the answer may well be "yes, all sixteen",
  but it is yours. A ledger law holds them from both ends meanwhile, so none can be quietly forgotten
  or quietly added.
- 🔵 **THE SETTINGS WIDTH BUDGET DEPENDS ON YOUR FILESYSTEM PATH (item 327, measured live).**
  The "Project root" row shows its value as a **full, un-elided absolute path**, while other rows'
  labels elide (`"Page wid…(prose)"`). So a long checkout path can eat the shared accessory-column
  budget: with a ~62-char root the rail is **gone at every width from 640 to 1000 and present only at
  1200**; with a 2-char root it is **present from 800 up**. ⚠️ **This is upstream of the width question
  already owed to you** — any budget designed without eliding that value behaves differently on
  different machines, so "should that path elide like every other row's text?" may be the actual
  question. `gallery/item-334/pwp-*.png` (long root) against `pwp-shortroot-*.png`.
- 🔵 **THE SETTINGS CARD'S WIDTH BUDGET IS A DESIGN CALL, WITH NUMBERS (item 327).** As a
  `Range` setting's card narrows, `overlay_right_shown` drops the **entire accessory column at once**
  — the value text *and* the rail — because they are gated together. **Who should yield first: the
  row name, the value text, or the rail?** And **should the picker fall out of its faceted/diagonal
  composition SOONER**, before losing the whole accessory column, rather than after? At the narrowest
  reachable failure the column needs **480px against 319px available**; where the diagonal cluster is
  merely tight it is **412px against 366px**. `gallery/item-309/327-ordinary-640.png` shows the
  column gone; `327-ordinary-1200.png` shows it present. ⚠️ **One thing to fix before designing
  anything:** rail presence is **non-monotonic** — present at 740–870 and 930–1200, absent at 640–720
  *and again at 880–920*. That hole is unexplained, and a budget tuned against a clean boundary will
  not survive it.
- 🔵 **ONE RAIL USED TO WEAR ANOTHER'S HIGHLIGHT, AND THE FIX IS VISIBLE (item 309).**
  `gallery/item-309/309-crop-BEFORE-buggy-3rails-white.png` → `309-crop-AFTER-fixed-only-zoom-white.png`:
  before, three rails read as bright because a shared colour painted them all with whatever ink the
  **selected** rail earned; after, only the selected one does. Worth a glance to confirm the
  unselected rails now read as quiet rather than as disabled.
- 🔵 **TWO PLATES CHANGED APPEARANCE AND BOTH ARE DESIGN CALLS (items 308, 316).**
  1. **A RIM APPEARED under the footer hint** on the `Bars` worlds — `gallery/item-308/`
     `before-Cassowary-palette.png` → `after-…`, and the `@2x` pair. It exists because the plate
     measured **ΔE 1.91 from its own page, under the ≈2.3 JND** — invisible, so not a plate. The rim
     is the notice channel's own mechanism, which is why it clears **ΔE 54** rather than needing a
     louder fill. ⚠️ **A one-pixel rim is a visible new line in a calm design** — worth confirming it
     reads as an edge rather than as a box.
  2. **AN EMPTY CHIP DISAPPEARED** above "Switch project…" — `gallery/item-316/`
     `before-Cassowary-Files.png` → `after-…`, and `@2x`. Nothing else moved. It was a plate drawn
     under a row whose text composes off-card, exposed rather than caused by item 297.
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

131. 🟡 **131e's FIVE COMPOSITION PIXEL LAWS LANDED (merged 2026-08-08). ONE LINE OF ITS LIST
     REMAINS — do not mark 131e complete.**
     ✅ **Five laws, five separate oracles**, over both diagonal worlds × 1×/2× × both `MENU_BAR_ON` arms
     × four canvases × list shapes and scrolls. ‼ **The spine is located by SEARCH, not read out of the
     geometry that draws it** — a law asking "is there ink where the code says" cannot fail on a spine
     the same code draws wrongly. **Continuity walks >8000 interior scanlines**, which is the claim a
     geometry probe cannot make at all: the accessor returns two endpoints, and so would a per-row spine.
     The placard law sweeps **every `CardAnchor` exhaustively**, because the wordmark's corner is derived
     from it and a missed anchor is a missed corner.
     ⚠️ **THE SIXTH MUTATION IS THE ONE TO READ: at its first floor of 2px the law SURVIVED its own
     mutation** — zeroing the gap constant still left 32px of room. Caught only because the mutation ran
     **before** the law was believed. The floor is now **calibrated from the roster**: shipped tightest
     **57px**, defect **32px**, floor **40** with 17px above and 8px below, both numbers in the code.
     ✅ **Child-picker return: DONE and capturable** — Settings *entry* is a live-App `BufferEffect`, so
     the door is `--screenshot-app`; verified end to end that the parked parent's `selected_index: 9` is
     restored after `Escape` on both worlds. (`Escape` from the **category** region closes outright, which
     is correct, and is why a first attempt read as a failure.) ✅ **Dashboard captures: DONE**, 14 in
     `gallery/item-311/`. ✅ **Sidecar geometry: UNBLOCKED by 174's slice**, merged alongside.
     ✅ **REMAINDER CLOSED (merged 2026-08-08) — 131e's VERIFY-CLAUSE LIST IS NOW FULLY DONE.**
     ⚠️ **THE ORCHESTRATOR'S BRIEF WAS HALF WRONG AND THE LANE CORRECTED IT WITH MEASUREMENT.** I said key
     the comparison by `item`. That is right for FILTER and **false for SCROLL**, and the reason is in the
     code: `row.dx = base_dx + dx_step * display` is a pure function of the window **SLOT**, never of which
     item occupies it — the diagonal cascade is authored behaviour, not drift. A scroll moves every visible
     item to a new slot **by definition**, so an item-keyed equality claim across a real scroll would be
     **FALSE, not merely weak.** Verified empirically rather than argued: the lane's first mutation, keyed
     on the last visible item, **tripped the scroll law and left the filter law green** — because the
     filter scenario's visible window is provably unchanged while the scroll's last item legitimately
     differs. **So the filter law is item-keyed, the scroll law is slot-keyed, and the file says why.**
     ✅ Both drive **real chords through `ReplaySession`** rather than building a `ViewState`, and read the
     published rows off the sidecar JSON. The filter fixture is engineered so survivors provably keep
     their slots — 30 matching items first in corpus order, 150 sharing no characters with the query,
     relying on the scorer's documented tie-break by original index — so the law compares a genuinely
     unchanged window. Presence floors on both, including **slot 0's item asserted to differ after the
     scroll**, so the scroll is proven to have moved `top_idx`.
     🔴 **THE ENROLMENT FOUND A REAL BUG, which is why it was worth doing.** Adding the diagonal arm to the
     headline sweep **did not pass on its own**: `grade_rows` probed the pointer at the card's
     **undisplaced** span for every row regardless of style — exactly the regression `PlannedRow`'s own doc
     names, *a staggered row clickable where it is not drawn.* It now reads each row's own span through the
     accessor production already uses: a no-op on `Pane`/`Bars`, correct on `Diagonal`.
     ✅ **The gap is proven closed with the mutation the item asked for** — reverting `row_at` to the
     undisplaced span now reddens the sweep, naming a pointer outside published row 1's span that still
     selects its item. **Originally:** 🔵 **THE GENUINE REMAINDER: a landed law driving filter and scroll through REAL CHORDS.** The
     measurement exists and is clean — the spine's ink column is **x = 504 in all three Mangrove frames**
     (base 109 items, filtered to 8, scrolled 12 rows) and **x = 695 in all three Magpie frames**,
     identical to the pixel — **but the regression guard for that door does not.**
     **Original:** **131e IS PART-LANDED (merged 2026-08-06) — the MARK is done, the composition
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

     ⚠️ **THIS NOTE'S MECHANISM HALF WAS ALREADY FALSE WHEN IT WAS WRITTEN, AND IT MISLED A LATER
     DISPATCH (corrected 2026-08-08, see item 346).** It claims `render/chrome/diagonal.rs` carries a
     shared `SELECTED_SPINE_WEIGHT = Logical(3.0)` and that the theme layer holds no per-world marker
     weight. **Neither is true: `SELECTED_SPINE_WEIGHT` exists nowhere in the tree**, and `DiagonalMark
     { weight, reach, aperture }` has been `DiagonalSpine`'s field — `ListStyle::Diagonal`'s variant
     payload — since 131e landed it **the same day this note was written**, with Mangrove authoring
     `CRISP` and Magpie `HAIRLINE`. **The note sits two paragraphs after 131e's own landing note saying
     so.** The TASTE half stands and is now item 346's. **Original note:**
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

174. 🟡 **SLICES 1, 2 AND 3 LANDED (2026-08-08). THE ITEM STAYS OPEN — a multi-round refactor.**
     ✅ **SLICE 3: the find/replace panel's geometry at schema `/203`, and the family was chosen by
     MEASURING.** 42 test functions across 22 files both read pixels and locate something. ⚠️ **Ground and
     backgrounds are the numeric worst (13) and were correctly EXCLUDED** — band counts and dither density
     are appearance by nature and no plan owns a shader's ground. Once in-crate laws that can already call
     the owner were separated from **capture-level** laws that cannot, exactly **one** test answered a pure
     geometry question with a pixel walk and had no published fact to ask instead.
     ✅ **The row band had NO owner:** the forward step was inline in the caret placer and its inverse
     inline in the hit-test — two spellings of one rule. `PanelRowBands` is now that owner, **each arm the
     arithmetic its old caller used verbatim** so no float re-association could move a pixel, and **three
     call sites ask it.**
     🔴 **A MUTATION SURVIVED, AND THE LAW ONLY EXISTS IN ITS USEFUL FORM BECAUSE OF IT.** Making the
     pointer's inverse read a **20%-wider pitch** left the law green — **at 32px rows a 20% error still maps
     every band CENTRE to the right row.** The probes were rewritten from centres to **TRANSITIONS** (just
     inside each edge, and 1.5px past each edge must already be the neighbour), and the same mutation then
     reddened. ⚠️ **Another mutation failed to COMPILE first** and was caught by reading the build
     separately — the third silent-no-op shape.
     🔵 **TWO LIVE PRODUCT DEFECTS FOUND AND DELIBERATELY NOT FIXED (layout policy — see OWED).** The
     panel's outer margin and inner pad are **raw device-px constants**, so `card.y` and `text.top` are
     **identical at 1× and 2×** — `CLAUDE.md`'s own tripwire (a) live in the product, **half their tuned
     size on every Retina display.** And **the card has no clamp to the window**: its published `x` reaches
     **−33.242 at 560px**, so it hangs off the left edge.
     ✅ **The law is FIX-TOLERANT about the first** — exact double only where the quantity is
     metric-derived, direction elsewhere with the mechanism named, and agreement with the drawn rim
     **at each scale independently** — so it stays green if the pad is ever scaled.
     ✅ **Every published extent is graded against INK, not position alone** — slice 2's surviving-mutation
     lesson applied. `sidecar.rs` held at **871 exactly**, measured, because rustfmt splits a combined
     argument line back out. It also **corrected a doc error in place**: harness-reach.md still advertised a
     `selected` key slice 1 removed.
     **Slices 1 and 2 follow. SLICES 1 AND 2 LANDED (2026-08-08). THE ITEM STAYS OPEN — it is a multi-round refactor
     and these are two surface families of it.**
     ✅ **SLICE 2 published the accessory cluster's three lanes** (label, value, rail) at schema **/202**,
     each **`null` rather than a zero-width rect** when the frame drew nothing there — because "the column
     was yielded" and "the column is empty" are different facts, **and the width at which the pair turns
     `null` IS the measurement.**
     🔵 **AND IT ANSWERED ITEM 327's QUESTION — see that item.** No width policy changed:
     `rowlayout::fits`, `CARD_MAX_W` and the `overlay_right_shown` gating are byte-identical. **The slice
     makes 327 and 342 arithmetic; it does not answer them.**
     ⚠️ **THE LANE CROSSED THE `chrome/**` GUARD DELIBERATELY AND WAS RIGHT TO — the brief held a genuine
     contradiction.** "Extract ONE owner and route both through it" and "stop if you need a chrome
     accessor" cannot both be obeyed here, and the first is the rule that matters: writing that match in
     the report would have created **exactly the second copy slice 1 existed to prevent.** ✅ **The rule was
     spelled inline FOUR times** — in the seat the shaped names upload through, in the rail owner, in the
     surface list the frost measures from, and in the accessory upload. **All four now ask.** The chrome
     diff is call-site replacements only, **net −6 lines with no file growing**, isolated in its own commit.
     🔴 **ONE MUTATION SURVIVED AND THE LANE REPORTED IT.** Halving the published label width **in the
     SERIALIZER** passed both capture laws — a uniform factor survives a doubling relation untouched, and
     every internal-consistency check still held. **The general lesson: a lane pinned only by its ORIGIN
     accepts any uniform width scaling, at every scale.** Fixed by grading width against the ink.
     ✅ **An existing law reddened and was right:** the println audit counted a test file outside a
     `tests/` directory as runtime-reachable. **Not weakened and not added to its expected table** — the law
     moved to the correct side of that boundary, which also took it out of a production size ceiling it had
     no business being measured against.
     ✅ **Non-vacuity uses the shipped `window_rows` of 31, not the fixture's parked 12** — at 12 the column
     fits at every reachable width and the yielding state, the entire subject, goes ungraded.
     **Slice 1's own note follows. SLICE 1 LANDED (merged 2026-08-08) — the sidecar publishes planned ROW GEOMETRY at
     schema /201. THE ITEM ITSELF STAYS OPEN: this is one slice of a multi-round refactor.**
     ✅ **`overlay.window` gained a `band` block and a per-row rect list**, so a law can assert
     drawn↔hit-test↔sidecar agreement **without inferring anything from pixels** — the defect 174 names
     in its first sentence, and the thing item 131e was blocked on.
     ✅ **The constraint that mattered was reading ONE owner.** `row_x_span` is extracted so the span has
     a single spelling, read by both the pointer inverse and the published rect; **before this the
     expression existed once, inside `row_at`, and publishing it would have created the second copy 174
     exists to prevent.** The serializer performs no arithmetic at all.
     ✅ **Not on the frame path** — three callers, one per capture and two laws, zero hits in the pipeline
     or chrome; one rect per planned display line rather than per corpus item, so it inherits the
     planner's O(visible) bound; and no cache key, so no `buffer.version()` collision can serve a stale
     band.
     ✅ **The law grades the published rect against the two oracles that do NOT read it** — the shaped
     glyph's own y off the uploaded buffer, and whatever the pointer accepts, probed inside the span and
     1.5px outside each edge. **Comparing the report to the plan alone would have been a tautology.**
     ⚠️ **A FALSE LAW WAS CAUGHT IN DRAFT:** both laws first asserted rows are **contained** in the band.
     They are not — **the shipped Saltpan Settings card is staggered**, its `dw` stepping −7px per row and
     the selected row stepping 4px outward, so it publishes an `x` **left of `band_x`**. Containment
     would have been a law satisfied by the product only by luck of which world you sample.
     ⚠️ **AND THE LANE REPORTED A MUTATION ITS OWN LAW DOES NOT CATCH** rather than hiding it: reverting
     `row_at` to the undisplaced span leaves the sweep green, because the sweep forces `Pane`/`Bars`.
     Item 131's own staggered-row law catches it; **enrolling the diagonal arm is 131e's axis.**
     🔴 **TWO PRE-EXISTING ROSTER LAWS WENT RED ON THE COMBINED TREE, and both were right.** Item 164's
     transaction law caught the projection reading the **LOGICAL** selected row. The lane knew and
     documented the distinction — but the sidecar **already** publishes a selection as
     `window.sel_row`, resolved through the owner that also colours the band, so a second answer could
     only be the logical one and **the two disagree throughout every selection move.** A block whose
     purpose is making drawn-versus-published agreement assertable must not ship two selections that can
     disagree, **so the selection was REMOVED from the projection rather than allow-listed.** The second
     was the capture-source roster refusing to let a new `.rs` file ship unaudited.
     🔵 **131e's sidecar-geometry agreement is now UNBLOCKED**, and the handoff is precise: read
     `overlay.window.band` + `rows[]` as JSON (copy `capture/tests/plan_geometry.rs`) or
     `overlay_row_geometry()` in Rust. It directly answers orientation (`rows[i].x` monotone in `i`, sign
     giving the mirror), the label–control gap, placard/row non-overlap, and **"filtering and scrolling
     never make surviving rows jump horizontally"** — compare `rows[]` spans by `item` across two states,
     which was previously unmeasurable. ⚠️ **Two things its lane must know: the selected row legitimately
     steps OUTSIDE the band, so do not write a containment law; and the headline sweep forces
     `Pane`/`Bars`, so enrolling `ListStyle::Diagonal` is a small, well-defined addition and is 131e's.**
     **Original slice claim: the sidecar publishes planned ROW GEOMETRY.** Chosen because it is simultaneously 174's own verification clause
     (drawn↔hit-test↔sidecar identity) and **the one thing item 131e is blocked on.** The slice is
     explicitly NOT the planner 174 eventually wants: no widget tree, no scene framework, no duplicate
     CPU renderer, no per-frame document plan. **Original:** **Separate pure render planning from shaping/cache mechanics and GPU
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

273. ✅ **FULLY CLOSED (2026-08-08) — ALL SIX RESIDUALS ARE DONE.** (3), (5) and (6) merged
     2026-08-06; (2) 2026-08-07; **(1) the CLI flag roster 2026-08-08**; and **(4) had already
     landed 2026-08-07 under a branch named for the ITEM rather than the residual**, which is why
     this list read three-remaining for a day and got one residual dispatched twice. ⚠️ **That is the
     staleness class now recorded in `.orchestrator/README.md`: before dispatching a residual,
     `git log --grep` the item number — the board is a claim about the tree.**
     **Original header: RESIDUALS (3), (5) AND (6) ARE CLOSED (merged 2026-08-06). (1), (2) AND (4)
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
     ✅ **(4) WAS ALREADY DONE — LANDED 2026-08-07 UNDER A DIFFERENT DISPATCH, AND THIS BOARD STAYED
     STALE LONG ENOUGH THAT THE ORCHESTRATOR DISPATCHED IT A SECOND TIME.** `53082d49` and `96a18427`
     are both ancestors of `main`; the lane wrote nothing, correctly, and its worktree diff was empty.
     Verified independently: the `Reference` command exists in the `Tools` category beside Credits and
     Guide (`catalog/navigation.rs:381`), **no ellipsis** — opening a bundled document loads it straight
     into the buffer rather than summoning a surface — routed through the ONE owner
     `open_bundled_doc`, guarded by a source-scan law that fails if a fourth opener hand-rolls the
     write-and-load shape, replayable through the real `--keys` door (`harness-reach.md` lists
     `open_reference | Applied`), and **already documented in `REFERENCE.md` and `site/reference.html`**
     — the document documents its own door.
     ⚠️ **THE LESSON IS THE ORCHESTRATOR'S: a residual list goes stale when one residual lands under a
     branch named for the ITEM rather than the residual.** `claude/item-273-inapp-door` closed (4) while
     this list still read `🔵 BLOCKED`. **Before dispatching any residual, `git log --grep` the item
     number** — it costs one command and would have saved this round. Originally:** 🔵 **(4) IS BLOCKED,
     correctly reported rather than routed around:** an in-app
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

298. ✅ **LANDED (merged 2026-08-08) — AND THE ANSWER IS TWO ANSWERS, which is better than the item
     proposed.** Declining the full takeover is right: that arm is the defocus behind a card summoned to
     the middle of the room and answered with the keyboard, and **a four-row menu under the pointer,
     dismissed by the next click, never asks the document to stop being the subject.**
     ⚠️ **BUT `None` EVERYWHERE WOULD HAVE SHIPPED ITEM 294's EXACT DEFECT ONTO THE CONTEXT MENU.** On
     the three worlds whose composition draws **no panel and no plates**, a menu with no backdrop
     interleaves with the document **glyph for glyph** — a property of the COMPOSITION, not of which
     picker is open.
     ✅ **So the menu declines the takeover and the footprint arm's own roster predicate decides:**
     `None` on the **17** worlds that back their own rows (the item's off-switch, with a principled
     reason) and a **footprint** on the **3** that draw neither.
     ✅ **One predicate read by BOTH arms**, so they cannot both fire or both miss — and it deliberately
     avoids the crisp-backdrop predicate, whose documented meaning is *"this card previews live document
     state"*: **a context menu previews nothing.** The sidecar's `dim_overlay` was a **byte-identical
     second copy** of the same question and now delegates rather than agreeing by coincidence.
     ✅ **Measured through the real pipeline:** the page beyond the menu carried **343** document edges
     out of a million pixels, and now carries **75,895–111,232**.
     🔵 **One inconsistent member left, flagged not touched:** the contextual **spell popup** takes `None`
     on every world including the three bare ones, so it still has the interleaving problem the menu now
     avoids. That is a decided position in DESIGN §5 ("it recedes nothing") — **now the only member out
     of step.** **Original:** **A right-click menu should not frost the document.** ✅ **294 HAS ANSWERED THE
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

299. ✅ **LANDED (merged 2026-08-08) — THE FIX IS ONE TOKEN, and the diagnosis came first as the
     item demanded.** `right_bind_lines`: `header_rows.max(1)` → `header_rows`.
     ✅ **Per-row ink resolution was ALREADY CORRECT** — `shape_overlay_right` resolves each row's
     secondary ink from its own index. The defect was one row **upstream** and **geometric**: the
     `.max(1)` forced the first secondary label to lead with a blank line even where `header_rows` is
     **0** — true only for the right-click Context menu, the one kind combining no query row with a
     populated secondary column. **Every row's secondary text, and the ink resolved for it, landed one
     display row below its own:** row 0 drew nothing; row 1 wore row 0's ink on row 1's ground.
     **Byte-identical for every other kind**, where `header_rows` is always 1.
     ⚠️ **IT IS NOT ITEM 309's MECHANISM, and the distinction is worth keeping.** 309 was one shared
     `set_color` for every rail in a frame. Here the ink was resolved **correctly per row** and then
     dragged onto the **wrong row's ground**. **"Correct ink, wrong row", not "one ink for every
     row"** — a distinct class.
     ✅ **Illegibility measured against each row's own clean ground rather than by scanning:** Wagtail
     **ΔE 0.0 — byte-identical, literally invisible** — Cassowary **1.9** (under the 2.3 JND), Firetail
     **7.5**. ⚠️ **The premise was stale in its literal pair** (Paste is unconditionally enabled now, so
     "Copy and Paste both disabled" no longer matches the code) **but the item's own hypothesis was
     right** — which is why it said to test that first. Floors swept over the two reachable
     disabled-row states × roster × selected-row position × 1×/2×, presence paired with contrast and
     `graded == presence_graded` asserted so no cell can silently skip.
     **Original:** **Two rows in the same state draw their accessory in different inks.** Copy
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

302. ✅ **LANDED (merged 2026-08-08) — 24 CONTRADICTIONS FIXED, AND A FALSE INVARIANT WAS HIDING A
     LIVE RENDERING DEFECT. Original: Loose comments — a second pass, a different class from 275's.** 275 removed
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

     ✅ **THE CENSUS, and it is the transferable part:** 72,140 comment lines greped by **SHAPE** rather
     than by "wrong", a raw pool of ~1,600 narrowed to ~90 read **in depth against their owning code**,
     yielding 24 confirmed. ⚠️ **Three of this item's own five named instances were ALREADY TRUE and a
     fourth described a file item 290 deleted — so the item's own list was 40% stale, which is the
     defect it was filed about, applied to itself.**
     🔴 **THE HEADLINE — why the item said to hunt shape 4 first. `caret.rs` claimed "all THREE bundled
     monos share the `CHAR_WIDTH` 0.6-em pitch, so the floor is a no-op on real glyphs." There are FOUR
     and they do not share a pitch** — measured off the shipped `hmtx` through the same skrifa stack the
     pitch owner uses: Plex Mono and JetBrains **0.60 em**, Monaspace Xenon **0.62**, Iosevka **0.50**.
     `metrics.caret_w` is a fixed `CARET_W`, **face-independent**, so the mono arm floors the block caret
     **above** the glyph cell: **Currawong and Cassowary draw a 14.400px block over a 12.000px cell —
     120% of the glyph, a 2.4px overhang into the next character at zoom 1.** Now item **345**.
     ⚠️ **Verified independently at merge:** four distinct mono faces in the roster, and exactly those
     two worlds carry Iosevka.
     ⚠️ **The law that should have caught it pins TAWNY ALONE** — one hand-picked mono world, and one of
     the two faces that happen to be 0.6 em. That is the "hardcoded mono-face list" shape `CLAUDE.md`
     names, and **`facepitch.rs`'s own doc records the same list losing Iosevka once before.**
     ✅ **THE CAPTURE PIPELINE'S DESIGNED-AROUND INVARIANT, found as promised:** a markdown test claimed
     captures *"can never carry a notice"* when `CaptureOpts::notice` exists and a live-app law already
     photographs a toast — item 296 repaired the production doc and **missed this one.**
     ✅ **One law landed, not four.** `every_crisp_backdrop_kind_is_a_value_picker` extracts
     `keeps_backdrop_crisp` as one owner, replacing **two independent deciders — one of which keyed on
     the mode's own SPELLING in the capture door**, so the live `App` and a capture could disagree about
     the same kind. **A stale enumeration was the symptom; two deciders were the cause.** The lane
     flagged that as a deliberate deviation from "add laws only" rather than slipping it in.
     ✅ **A second drafted law was DELETED before landing** because the extraction left it no subject an
     existing law did not already own — a law without a subject being the exact failure this item exists
     to catch.
     ✅ **Numbers were DELETED rather than corrected** wherever the mechanism is roster-derived anyway:
     five stale world counts, a WGSL line count, and an audited-row count that was **both wrong and
     redundant with an assertion four lines below it.**
     🔵 **Two figures reported rather than guessed at**, per precedent: `spellunderline.rs`'s contrast
     sweep needs a real rendered-pixel sweep before anyone edits it, and `semantic/native.rs`'s
     `0.79/0.84 ms` is unpinned prose for that bench's owner. ⚠️ **The docs half of the census is item
     344** — that fan-out's report reached the orchestrator, not this lane.
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

308. ✅ **LANDED (merged 2026-08-08) — THE RIM EARNED ITS KEEP: ΔE 1.91 → 54.13.** Premise
     re-measured before anything was touched and it held exactly (Cassowary 1.91, Galah 5.67,
     Firetail 7.50). The footer plate now carries **the notice channel's own one-pixel rim**, and the
     floor it must clear is **that channel's `PLATE_PRESENCE_MIN` of ΔE 15, REUSED rather than
     re-derived** — both channels earn it the identical way, a value-stepped fill plus a rim.
     Swept 1×/2×: **Cassowary 54.13/58.47, Firetail 40.91/44.00, Galah 22.33/28.08**, with
     **Galah@1× the roster's tightest real value**, clearing the floor by half again.
     ✅ **THE GATE WAS NOT WIDENED** — this item's own law names widening as the dishonest repair.
     ✅ **Mutation-proven by emptying the rim:** Galah falls straight back to 5.67 and the law names
     it — *"the rim did not earn its keep here"*. The colour resolves fresh every frame with no
     `sync_theme_colors` entry, the notice channel's own reasoning, so a theme swap cannot serve last
     world's rim. **Original:** **CASSOWARY'S FOOTER PLATE IS ΔE 1.91 FROM ITS OWN PAGE — below the ≈2.3 JND.**
     Revealed by item 306: the old absolute-luma gate aborted on Firetail before
     Cassowary was ever graded, so one world's failure was hiding another's.
     ✅ **The recommended repair is a RIM, not a token change** — item 296's notice
     channel clears ΔE 15 precisely *because* it draws a one-pixel rim, and this footer
     plate has never had one. **Verify:** the plate's presence floor swept over the
     roster; do not widen the gate — `assert_plate_separation_is_not_vacuous`'s own
     failure message names widening as the dishonest repair. **Routing:** production
     tier, then the user's eye on the rim.

309. ✅ **LANDED (merged 2026-08-08) — and the LAW was harder than the fix.** `prepare_multicolor`
     already carries per-instance colour for the writing-streaks heatmap, so each rail computes its
     own ink from `on_band.contains(item)` and uploads through the **existing** mechanism rather than
     a second pipeline. The track was always uniform and is unchanged.
     ⚠️ **TWO EARLIER DRAFTS OF THE LAW WERE FALSE-POSITIVE, and both failure modes are reusable.**
     Grading a thumb against `theme::muted()` as an idealized constant fired on **Potoroo**, whose
     `Stripes` ground varies enough **within one row's height** to fool a single-frame pixel search.
     Grading against an **ADJACENT** row fired on several worlds at up to **ΔE 41 on a build that
     already had the fix** — adjacent-row elevation and shadow bleed is a real rendering fact, not
     this defect. The law that holds grades each rail **against its own two renders**
     (selected-elsewhere vs nothing-selected) and selects a row 2–3 away from both graded rails.
     ✅ **The ceiling was CALIBRATED, not guessed:** a roster survey under the reinstated bug reads
     **0.00** on every world where the flip cannot apply and **≥22.00** wherever it fires, so 6.0
     sits in the gap. **Mutation-proven:** *"Tawny: the NON-selected Zoom rail's thumb changed
     ([139,145,157] → [230,230,230], ΔE 32.04) purely because a DIFFERENT, non-adjacent row became
     selected"*. Mangrove byte-identical (`rail_thumb_over_fill()` is false there).
     **Original:** **`thumb_ink` IS ONE `set_color` FOR EVERY RAIL, SELECTED OR NOT.** Named by item
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

311. ✅ **CLOSED (merged 2026-08-08) — PREMISE FALSE. The flip is fine, and it makes the column MORE
     visible, not less.** On both diagonal worlds `muted` fails the 3.0 floor so the fallback fires — and
     on both it resolves to **`base_content`, not `base_100`**, so there is no invisible-ink case. Real
     pixels: the selected chord reads **ΔE 76.60** against its ground on Mangrove and **92.81** on Magpie,
     against 41.51 and 51.71 unselected — **the flip is worth +35 and +41.**
     ✅ **Two facts worth keeping.** The selected and unselected grounds are **byte-identical** on both
     worlds, which is the empirical confirmation that `Diagonal` really does emit no row fill — **so the
     premise the item reasoned FROM was true and the conclusion drawn from it was not.** That is what
     "measure before changing" buys.
     🔴 **THE REAL FINDING IS A LATENT HAZARD: the safety is an accident of both worlds' tokens, not a
     design.** The accessor returns `base_100` — the page itself — whenever the band's contrast against
     ground exceeds its contrast against content while `muted` fails the floor. **Magpie sits 1.82 against
     9.85**, so a Diagonal world authored into that corner would ship Firetail's thumb on its chord
     column and nothing would have said so. **That law is the deliverable**, over the whole roster with
     all four list families enrolled.
     ✅ **Its sharp arm is the interesting part:** when the selected column's ground is the SAME surface as
     an unselected column's, nothing was traded against and the flip may not cost a ΔE. **A slack
     tolerance cannot express that** — Galah's chord on its own `Bars` plate legitimately reads 32.7 where
     the same chord on page reads 47.6, so a global never-worse slack would need to be ~15 wide and would
     then tolerate most of the defect.
     ⚠️ **An oracle repair on the way:** the first version took the **most common** non-ground colour as
     the ink, which measures antialiasing — Paperbark's figure swung from **5.64 to 53.16 between two
     window heights of one world.** It now takes the furthest colour holding ≥4px of area.
     **Original:** **`Diagonal`'s SECONDARY FLIP IS PROBABLY WRONG BY ITEM 306's OWN ARGUMENT** — a
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

316. ✅ **LANDED (merged 2026-08-08) — THE ANSWER TO ITS OWN FIRST QUESTION WAS "NO", AND THE FIX
     WENT TO THE ROW-SURFACE OWNER RATHER THAN THE CUE.** Premise re-confirmed against an
     unmodified `main` binary first. Cause: `overlay_unselected_bar_rects` backed **every** item-less
     row, header or location, while a `PlanLine::Location`'s inline text is glyph-free wherever
     `LocationStyle::draws_inline()` is false — exactly Cassowary's off-card `RotatedRail`. It now
     reads **the same `draws_inline()` gate the shaper already reads**. Never a named-world check.
     ✅ **THE EXCLUSION IS EARNED BY MEASUREMENT, as the item demanded:** the production row-surface
     probe emits **ZERO** rects over the location row's own y-slot for the excluded case, swept over
     **roster × every faceting `OverlayKind` (derived from `facets::scheme`, not hardcoded) × both
     DPI tiers**, with non-vacuity asserted in **both** directions so the sweep cannot pass by seeing
     neither shape. Plus a real-pixel ground check (ΔE < 1 between the freed slot and the card ground
     above it).
     ✅ **Mutation-proven:** *"a glyph-free location row (slot y 197.4..232.6) still has 1 row
     surface(s) drawn over it — the empty chip item 316 was filed against"*.
     ✅ **THE RECORDED TRIPWIRE IS INTACT:** `overlay_prepare_bar_scrims`'s `backing == BarePlates`
     gate is untouched, as are `draws_row_plates()` and `overlay_selection_rects`.
     ⚠️ **HEADROOM WARNING:** `src/render/chrome/overlay_selection.rs` now sits at **exactly the
     500-line production ceiling**. It was not squeezed to fit — it grew 42 lines of real mechanism —
     but **the next change there has no room and must carve `mod tests` or a submodule first.**
     **Original:** **THE LOCATION ROW'S OWN BAR PLATE IS A VISIBLY EMPTY CHIP.** Exposed — **not
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

319. ✅ **CLOSED (merged 2026-08-08) — BOTH CITED INSTANCES DISSOLVED UNDER SOUND MEASUREMENT, AND
     NEITHER WAS A PRODUCT DEFECT. Recorded as "premise false, oracle repaired", not as fixed.**
     ⚠️ **Instance A is UNREACHABLE LIVE.** The footer tips ride **only** while the Keybindings overlay
     is open — `sync_discoverability`'s own comment says so — so no other flat picker ever grows a
     footer, and the cited Mangrove/Command combination cannot occur. Under the one combination that
     **can**, there is **no clip at the shipped default zoom on any world** (worst real tip 486.3px
     against 496px).
     ⚠️ **Instance B — item 318's "~42px of card ink outside `overlay_card_rect`" — DOES NOT REPRODUCE**
     against a production shaping accessor (`overlay_line_glyph_box`): **zero overflow** on either
     diagonal world at either scale. ✅ **AND ITEM 329 PREDICTED THIS EXACT FAILURE HOURS EARLIER.** 318
     found it by treating the card-ink mask as an **inclusion set**, and 329's audit documented that the
     mask is sound only as a **veto** and false inverted. **So that contract caught a stale finding on
     its first contact — the oracle was wrong, not the product.**
     ✅ **THE GUARD THE ITEM ACTUALLY WANTED NOW EXISTS**, swept over the **full roster** rather than
     narrowed to the diagonal worlds (Instance A's clip, when forced, reproduced on Tawny — a plain
     `Pane` world — so narrowing would have been wrong), every catalog tip, 1×/2×, and both menu-bar
     states at the shipped zoom.
     🔴 **AND IT IMMEDIATELY CAUGHT A FOURTH CLIP NOBODY KNEW ABOUT — now item 342.**
     🔵 **One residual, named rather than folded in:** at zoom **1.0** (not the shipped 0.8) Mangrove's
     plain hint line overflows the card's right edge by **~7.7 logical px**; Magpie and Paperbark stay
     clean. Direction-gated and zoom-gated. The likely mechanism — the clamp's width budget is
     advance-based while the hint's symbol glyphs may have wider cells — **needs its own check** and is
     the same shape as item 342. **Original:** **STILL OPEN — and item 318 added a SECOND, MEASURED INSTANCE: card ink exists ~42 logical px OUTSIDE
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

322. ✅ **LANDED (merged 2026-08-08) — 37 CONSTANTS, NOT ~30, AND THE PREMISE WAS TWO-THIRDS TRUE.**
     It correctly predicted `PAGE_TEXT_PAD_CHARS` as "at least one genuine miss". It was **wrong about
     the biggest group**: sixteen constants are not handled by a different, correct pipeline — they are
     handled by a **fourth one that is wrong**, now item **338**.
     ✅ **What landed:** `PAGE_TEXT_PAD_CHARS` into the `Chars` family that already existed; the two
     caret pads into `Logical`, where `px = m.caret_h / CARET_H` is **exactly** `Metrics::scale`
     because `with_dpi` built `caret_h` that way; three durations into a new **`Millis`** family; four
     genuinely dimensionless ratios declared with reasons; and eleven owner-resolved constants
     **excluded** because `Metrics::with_dpi` already multiplies each by `zoom * dpi` — typing those
     `Logical` would apply DPI **twice**, invisible at the one scale captures run at by default.
     ✅ **THE EXCLUSIONS ARE BY KIND WHEREVER THAT WAS ACHIEVABLE, which is the difference between a
     mechanism and a promise.** `Millis` has **no `Metrics::px` at all**, so the **compiler** refuses
     the pixel multiply (proven with `E0308`). The owner-resolved set is **DERIVED by parsing
     `Metrics::with_dpi`'s own body**, so the exclusion **expires** the day the owner stops multiplying
     one — proven by removing `FONT_SIZE`'s multiply and watching it become unclassified. ⚠️ **The
     ratio family is still a name list, and the lane said so** rather than claiming otherwise.
     ✅ **THE AUDIT WROTE THE LAW THAT WAS MISSING.** The newtype stops a length reaching a draw call
     *unmultiplied*; it does **not** stop a caller handing `px` the **wrong factor**, since `px` takes
     any `f32`. `no_length_is_resolved_against_zoom_alone` now sweeps every product source for that,
     because the caret's pads live outside the declaration files.
     ✅ **Evidence at the scale that matters: 22 shots per side** (3 worlds × 3 caret modes × 2 DPI plus
     palette and selection), rebuilt from the pre-change commit rather than stashed. **22/22 PNGs and
     22/22 sidecars byte-identical, eleven of them at dpi 2** — so nothing declared "already scaled
     elsewhere" moved where a doubled multiply would show. Ledger `render.rs` 2671 → **2705**.
     **Original:** **`src/render.rs`'s REMAINING ~30 CONSTANTS ARE UNCLASSIFIED, and item 315 declined to
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

327. 🔵 **THE QUESTION IS ANSWERED WITH NUMBERS AND IS NOW PURELY THE USER'S (item 174's slice 2,
     2026-08-08). NOTHING TECHNICAL REMAINS BLOCKING IT.**
     ✅ **WHO YIELDS FIRST: THE NAMES DO — and that is not what this item guessed.** Measured at
     `--theme Mangrove --root /p`, 27 planned rows, menu bar off (macOS's default). **The boundary is
     between 780 and 781 logical px:**

     | logical width | accessory column | widest label | widest value | rail | rail hit band |
     |---|---|---|---|---|---|
     | **780** | **yielded** | **215.42** | — (`null`) | — (`null`) | — |
     | **781** | granted | **156.67** | 94.32 | 69.63 | 93.57 |
     | 850+ | granted | 215.42 | 94.32 | 69.63 | 93.57 |

     ✅ **So at 781 the names ELIDE by 58.75px to buy the accessory column, and at 780 the column goes
     entirely and the names take all 215.42 back.** Between 790 and 850 the label grows in ~9.8px steps
     **while the value and rail never move at all** — **the accessory cluster is RIGID and the name lane
     absorbs every pixel until the whole column is dropped.** The rail reserves **79.42px** on top of the
     94.32 value.
     ✅ **The menu-bar arm moves the boundary, which confirms that axis is real:** bar **on** grants at 780
     with 26 rows; bar **off** yields at 780 with 27 — a row fewer is a narrower widest label is a looser
     budget. ✅ **Only the two diagonal worlds ever yield** in the swept 640–1200 band; the other 18 grant
     at every width, so the yield regime is a diagonal-cluster phenomenon at these widths.
     🔵 **WHAT IS LEFT IS ONLY TASTE, and the numbers frame it:** the rail costs ~79px and the value
     ~94px, against a name lane that gives up 59px before surrendering the lot. **Should the names keep
     eliding further so the rail survives narrower, should the rail yield before the value, or is dropping
     the whole column at 780 correct?** ⚠️ **Item 342 is upstream** — it asks whether the card's own width
     cap should be font-aware, which changes the width this divides. **Worth deciding together.**
     ⚠️ **The earlier premise corrections stand and are still worth reading. Originally:** ⚠️ **STILL OPEN, AND THE PREMISE NEEDED THREE CORRECTIONS — measured 2026-08-08, no code
     changed.** The defect is real but not where item 317 put it.
     ⚠️ **(a) 317's SPECIFIC CLAIM DID NOT REPRODUCE.** At 1200×800 with the current `SETTINGS`
     roster, Zoom's rail **is present** — via both the ordinary capture and `--screenshot-app`. The
     reproducible failure is at **narrow width**, not the shipped default canvas.
     ⚠️ **(b) THE LAW SWEEPS A STATE THE PRODUCT CANNOT REACH.** `settings_row_reach_law` includes
     `workspace=false`, but `workspace_shape()` returns `Some(RailOverRows)` **unconditionally** for
     `OverlayKind::Settings` (`overlay/workspace.rs:128`), so `overlay_workspace` is **always true**
     in the real product. The cited panic comes from a fixture-only cell. Under the one **reachable**
     state the defect still occurs, but with different numbers and a different mechanism.
     ✅ **(c) THE MECHANISM IS TWO REGIMES, NOT ONE.** `overlay_right_shown` gates the **entire
     accessory column together** — value text *and* rail — via `rowlayout::fits`. Moderate narrowing:
     the picker stays faceted and `shape_faceted`'s own `fits()` fails. Severe narrowing (640
     logical): **the facet strip ITSELF degrades away** (`geom.theme` flips false) and the flat
     shaper's independent `fits()` fails by far more — `needed=480.44` against `right_px=186.68`,
     short by **161.84px**.
     ✅ **THE BOUNDARY IS DATA-DEPENDENT, AND THIS IS THE MOST USEFUL THING ANYONE HAS LEARNED ABOUT
     THIS ITEM (measured live 2026-08-08, once item 334 made `--screenshot-app` honour a canvas).**
     **The "Project root" row's value is the FULL, UN-ELIDED absolute path** — unlike other rows'
     labels, which elide (`"Page wid…(prose)"`) — so **it can dominate the shared accessory-column
     budget on any real host with a long checkout path.** With the default root (a ~62-char absolute
     path) the rail is **absent at every sampled width 640–1000 and present only at 1200**; with a
     2-char root (`/p`) it is **present from 800 up with no gap through 1000**. ⚠️ **So the width
     budget has an input nobody was accounting for, and it is the user's own filesystem path.** Any
     budget designed without eliding that value will behave differently on different machines.
     ⚠️ **The non-monotonic hole did NOT reproduce at either root length** on a 20–80px sampling grid
     — **flagged UNRESOLVED, not disproven**: it may have been an artefact of the earlier sweep's root
     length, a different live row count (27), or a hole narrower than the grid.
     🔴 **The original claim, kept because it is not disproven: THE BOUNDARY MAY BE NON-MONOTONIC.** Measured at
     `world=Mangrove dpi=1 workspace=true`, PageWidthProse: rail **present** at 740–870 and
     930–1200, **absent** at 640–720 **and again at 880–920**. A real, reproducible gap between two
     present bands. The lane's hypothesis, undiagnosed: `name_px` is the widest label across **all**
     visible rows, so another row's label crossing an elision or wrap threshold transiently widens
     the column. **Diagnose this before designing the budget** — a fix tuned against a monotonic
     boundary will not hold across a hole in the middle of it.
     🔵 **THE PRODUCT QUESTION IS HANDED BACK WITH ITS NUMBERS, as the brief asked.** At the narrowest
     reachable failure the column needs **480px against 319px available (161px short)**; at the
     "diagonal cluster is merely tight" case, **412px against 366px (46px short)**. **Who yields
     first — the row name, the value text, or the rail — and should the picker fall out of its
     faceted/diagonal composition SOONER, before losing the whole accessory column rather than
     after?** Nothing in `overlay_shape.rs`, `rowlayout::fits` or the diagonal-cluster budget was
     touched. Captures: `gallery/item-309/327-*`.
     **Original:** **A `Range` SETTING LOSES ITS RAIL AT THE SHIPPED `window_rows = 31`.** Measured by item 317
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

329. ✅ **CLOSED (merged 2026-08-08) — premise TRUE, oracle consolidated, contract ENFORCED; 0 of
     4 surviving callers wrong. NOT "fixed": no product changed**, and the diff is test files plus
     `docs/render.md`, so the shipped binary is unchanged by construction.
     ✅ **The premise's substance is now measured rather than remembered.** The "52 rows above the
     card" figure reproduces exactly at 1× on every enrolled world. ⚠️ **The three shadow counts do
     NOT reproduce** — 318's re-aim deleted the oracles, so their predicate is unrecoverable — but
     **the class does, with the same 100× separation and the same near-clean member**: Paperbark
     0–107 against Magpie 13001–40863. **A one-world check would still find none of it.** The
     reconstruction also moves **2.7× with menu-bar state alone**, which is why its floors are taken
     as a minimum over the sweep rather than at this host's default.
     ✅ **THERE WERE TWO BYTE-IDENTICAL DEFINITIONS**, each with its own duplicate `luma`, `step` and
     thresholds. Now one owner, `frost_card_ink.rs`.
     ⚠️ **THE PREDICTED HIGH ERROR RATE WAS REAL BUT ALREADY SPENT — both wrong readers were 318's
     own and 318 deleted them.** What the census found instead is the more interesting failure,
     because each of these PASSES today: **three callers consult a veto OUTSIDE its premise.**
     (a) the hue law, **repaired** — its region was the card's box, 2.3–5.4% of which the frost never
     reaches on a leaning world, so live page at the live page's exact colour pulled the "frosted"
     mean toward the "live" mean it is compared against: **the bound got EASIER the less of the box
     the frost covered.** It also derived the veto with a hardcoded `dpi = 1.0` while running at one
     scale; it now sweeps both. (b) the collar check — sound but removes **13.9% of the collar on
     Magpie at 1× and 0.0% on Mangrove at the same geometry**; left alone deliberately, since
     removing it would admit any ground difference between frames into a sharpness claim.
     (c) the parallelogram corner law — **sound but structurally DEGRADING, and no repair exists**
     (item 337).
     ✅ **The mechanism is a newtype whose `flags` field is private with exactly one reader,
     `vetoes(x, y)` — no `Index`, no `Deref`, no iterator, no length.** Both falsified oracles began
     by **enumerating** the flagged set, and a consumer can no longer reach it. The lane stated the
     honest limit instead of overselling: no type forbids `if !ink.vetoes(..)`; what it removes is the
     cheap path. A `visit()` combinator was considered and **rejected on cost** — unaffordable for the
     ramp law at 2×.
     ⚠️ **THE SECOND MUTATION IS A FINDING IN ITS OWN RIGHT:** raising the threshold until the mask
     stops flagging the world's ground would make it nearly trustworthy as an inclusion set **and
     useless as a veto**, because a glyph's anti-aliased skirt has small steps. **The veto's job is to
     over-exclude**, and the law now refuses that "improvement" by name.
     ✅ **Not claimed as a bug fixed:** the dilution was measurably a no-op at the roster's current
     rake (byte-identical Lab means with and without the gate). **A hazard closed** — the diluting set
     grows without bound with the shear, and a silent dilution is now a loud failure.
     **Original:** **294's `card_ink_mask` IS A VETO AND DOES NOT INVERT INTO AN INCLUSION SET — two frost
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

331. ✅ **LANDED (merged 2026-08-08).** The three descriptions lead with the step item 301 added —
     *"Choose a folder, then export as `.docx`; markdown buffers only."* Regenerated through
     `scripts/regen-reference.sh`, and **the diff is exactly those three rows in both documents**:
     twelve changed lines, three rows before and after, two files. Nothing else moved.
     🔵 **ONE ASYMMETRY THE LANE FLAGGED RATHER THAN SILENTLY ACCEPTING, and it is now item 336:**
     Word and HTML are available on **web** too, where the browser owns the download and there is no
     folder to choose — so *"Choose a folder"* is literally true on native and an approximation on
     web. **Same shape as the ellipsis item 301 pinned with a law**, but the description carries no
     such law. **Original:** **THE THREE EXPORT COMMANDS' CATALOG DESCRIPTIONS NO LONGER MENTION THAT YOU CHOOSE A FOLDER.**
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

332. ✅ **LANDED (merged 2026-08-08) — IT TOOK TWO RULES, AND THE BRIEF'S PREDICTION HELD.**
     Declining any token starting with `-` does **not** fix the reported bug, because `file.md` does
     not start with `-`. So the second rule is an explicit per-row property rather than a heuristic:
     **`Operand::opt_numeric`**, an optional operand whose value must parse as a plain `usize`.
     `--menu-open` takes it.
     ✅ **`--pack-icns`'s `DIR` deliberately does NOT.** A path is legitimately any non-flag string,
     so it is **ambiguous by construction** and the lane declined to invent directory-name sniffing.
     **That residual is pinned by its own law**, so a future "fix" cannot quietly make `DIR`
     heuristic and reintroduce the ambiguity elsewhere. The rule lives on the operand **declaration
     as data**, not as a `flag.id == MenuOpen` special case in the parse loop — which is what the
     roster exists for.
     ✅ **VERIFIED AT THE BINARY, not only in unit tests:** before, `--screenshot --menu-open FILE`
     produced a sidecar with **no `buffers.active` at all** — the file was silently eaten; after, it
     is the file. The mutation was then re-applied to the **fixed** binary and rebuilt, to confirm the
     repro was not a static-reading inference. Both laws watched failing by name.
     ⚠️ **One invocation genuinely changes meaning** — `awl --menu-open file.md` used to open an
     untitled buffer and now opens the file. That is the bug being fixed rather than a behaviour
     anyone could have relied on, but it is product-visible. `--pack-icns` changes no reachable
     invocation: it only stops eating a following flag, and it exits before any file would open.
     No refusal message moved. **Original:** **`--menu-open` AND `--pack-icns` SWALLOW THE NEXT ARGUMENT UNCONDITIONALLY, so
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

333. ✅ **CLOSED (2026-08-08) — PREMISE FALSE, RECORDED SO IT IS NOT RE-RAISED: the site's analytics beacon is
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

334. ✅ **LANDED (merged 2026-08-08) — HONOURED, not refused, and the defect was FOUR TIMES WIDER
     than filed.** `LiveAppSpec` carries canvas and dpi and hands them to the same `CaptureOpts` the
     shared renderer takes, so the meaning is identical on both doors.
     ✅ **Proven by MEASURING GEOMETRY, not by echoing a flag:** a long line wraps to **12** visual
     rows at 1200×800, **20** at 640×800 (genuinely more reflow, not a relabelled number), and **12**
     at 2400×1600 dpi 2.0 — identical to the 1200×800 dpi-1 baseline, which is the documented dpi
     meaning holding on this door. Mutation-proven by reverting the two assignments while keeping the
     bindings alive.
     ⚠️ **THE DIAGNOSIS FOUND THE SAME BUG SEVEN MORE TIMES.** `--screenshot-app` fell through to the
     plain-`Screenshot` bucket in the hook classifier, so **`--sel`, `--zoom`, `--scroll`,
     `--preedit`, `--search`, `--search-replace` and `--default-folder` were ALL classified as
     honoured for that door and silently discarded** — none ever reached `LiveAppSpec`.
     `CaptureKind::ScreenshotApp` now has its own accurate hook list: canvas, dpi, root and workspace
     honoured, the rest refusing loudly. **A door that shares another door's bucket inherits promises
     it does not keep.**
     ✅ `docs/harness-reach.md` **and** `CAPTURE.md` both record the fix, the dpi meaning and what is
     still refused — harness-reach.md being the file briefs are told to read before promising a
     capture, so a gap there propagates. **Original:** 🔴 **`--screenshot-app` SILENTLY IGNORES `--capture-size` AND `--capture-dpi`.** Found by item
     327's lane when the width it needed to measure was unreachable through that door.
     `Mode::ScreenshotApp`'s `LiveAppSpec` carries **no `canvas`/`dpi` fields**, so the canvas is
     hard-coded to **1200×800** and both flags are accepted and discarded without a word.
     ⚠️ **A flag that is accepted and ignored is worse than one that is refused** — the lane spent a
     round believing it had measured at 640 when it had not, and any future live-`App` claim about
     geometry is suspect until this is fixed. This is the "a check runs in one configuration, and
     that configuration is itself an untested hypothesis" failure with the configuration silently
     overridden.
     ✅ **Build:** carry canvas and dpi on `LiveAppSpec` and honour them, **or** refuse the
     combination loudly. Prefer honouring: the live-`App` door is the only one that reaches
     transitions the ordinary path classifies Unsupported, so it needs the geometry axis most.
     ⚠️ **`docs/harness-reach.md` does not record this gap** — it must, whichever way the fix goes.
     ⚠️ Note the flag roster now declares operands as data with laws behind it, so express any
     refusal there rather than as a special case. **Routing:** production tier.

335. ✅ **LANDED (merged 2026-08-08) — and the lane DID NOT STOP AT GREEN, which is the part worth
     keeping.** The swept set now derives from `workspace_shape()` instead of enumerating a boolean,
     so the unreachable `workspace=false` cell cannot be graded at all.
     ⚠️ **Taken alone the corrected law passes EVERYWHERE — exactly the "you removed its subject"
     warning.** The lane diagnosed rather than assumed: a **separate, already-documented and
     deliberately parked** simplification in the same helper — `overlay_window_rows` left at the flat
     default of **12** rather than the shipped **31** — independently shields it. Proven with a
     throwaway patch that reproduces item 327's real defect at the one now-reachable cell, then
     reverted. **The subject is intact; a different fixture is hiding it, and that one is parked on
     327's own open product question.**
     ✅ **The neighbour sweep found ~40 sites with the `[false, true]` shape** and confirmed the rest
     are genuinely independent interaction axes rather than values an owner already decides — so the
     pattern is not systemic. **Original:** **`settings_row_reach_law` SWEEPS A STATE `OverlayKind::Settings` CANNOT REACH.** Found by item
     327's lane. The law's axis includes `workspace=false`, while `workspace_shape()` returns
     `Some(RailOverRows)` **unconditionally** for that kind, so `overlay_workspace` is always true in
     the product. **The cell that produced 327's cited panic is fixture-only.**
     ⚠️ **This is the mirror of the enrolment failures already recorded** — those swept nothing;
     this sweeps something that does not exist, which is just as misleading because **it produces
     panics about impossible states and sends a lane chasing them.** A law's axis must be the
     product's reachable state space.
     ✅ **Build:** derive the swept states from the owner (`workspace_shape()`) rather than
     enumerating a boolean, so an unreachable combination cannot be graded. ⚠️ **Check the
     neighbours** — if this law enumerates one boolean it does not own, others likely do too, and the
     fix is the derivation rather than deleting one cell. **Verify:** the law still fails on 327's
     real, reachable defect at narrow width after the axis is corrected. **Routing:** production tier.

336. ✅ **LANDED (merged 2026-08-08) — REWORDED, NOT PINNED, and the reasoning is the transferable
     part.** Both honest options were live; the lane took the cheaper one because **a wording exists
     that is accurate everywhere without going vague**, so a second law-maintenance surface mirroring
     `ellipsis_law` would be machinery kept forever to describe a wording problem a better sentence
     dissolves. The bar for pinning — *"no wording is both accurate and useful"* — was not met.
     ✅ Word and HTML now read **"Export as `.docx`; markdown buffers only, folder chosen on native."**
     — the destination step item 331 added is **kept and correctly conditioned**, not retreated from,
     which is what distinguishes this from the vague-dodge. `ExportPdf` is unchanged and needed no
     change: it is `native_only`, so it never ships where its own claim fails.
     ✅ **Regen diff exactly 8 lines** — two description cells, before and after, across both documents.
     `editing.rs` holds at **498** against its hard 500 ceiling, unchanged, because the new wording
     collapsed both fields to single lines under rustfmt. The label-divergence law and both ellipsis
     laws were run and are unaffected. **Original:** **A COMMAND'S DESCRIPTION CAN OVER-PROMISE A PLATFORM AND NOTHING CHECKS IT — the mechanism
     exists for LABELS only.** Found by item 331's lane, which flagged it rather than inheriting the
     precedent blindly. Item 301 established that a static string promising a surface must be true
     per platform, and pinned it: `ellipsis_law` asserts the set of platforms the Export **label**
     over-promises to is exactly `{Web}`. The new **descriptions** say *"Choose a folder"* and carry
     the identical over-promise **with no law at all** — Word and HTML ship on web, where the browser
     owns the download and no folder is chosen.
     ✅ **Build:** extend the existing law's subject from labels to descriptions rather than writing a
     second mechanism — `export_picks_destination(platform)` is already the pure, platform-parameterised
     predicate, so the same enrolment works. ⚠️ **Decide deliberately whether the fix is a LAW or a
     WORDING change**: a per-platform description is not currently expressible, so honest options are
     to pin the known divergence exactly as the ellipsis is pinned, or to word the description so it
     is true everywhere. **The second is cheaper and may be better** — say which and why.
     ⚠️ **Docs voice applies** (matter-of-fact, no filler) and this text is user-facing in the palette.
     Any change regenerates `REFERENCE.md`; **verify the regen diff moves only what the change
     explains.** **Routing:** production tier.

337. ✅ **CLOSED (merged 2026-08-08) — PREMISE FALSE AS A DECAY, and the reason is a good one: ITEM
     343 ALREADY REMOVED IT.** 343 narrowed the frost's box after this was filed, which **grew** the
     unfrosted region, so the cited subject of 1716–14263 px now measures **10,825–99,740** — 7× larger.
     Measured per cell across all eight leaning cells: **the veto removes 0.0% of the graded region**,
     and with **no document at all** the old field counts **zero** edges there. **So the erosion this
     item names measures zero** — while the mechanism stays real elsewhere (the veto flags
     63,312–516,272 px frame-wide).
     ✅ **The re-aim landed anyway and removes the hazard rather than the symptom.** The region is now
     **arithmetic over the shape with no pixel term**, so `measured` cannot erode; the field is a
     **residue in which card ink and world ground both cancel by construction**; and a card-free claim
     grades every surface `overlay_drawn_surfaces` declares, **stated before the shear branch** so the
     upright `Rules` member is graded too. 343's `full <= 2` reasoning is kept verbatim.
     🔴 **A FINDING WORTH CARRYING BEYOND BOTH ITEMS: `overlay_drawn_surfaces` IS NOT A PER-PIXEL INK
     MAP.** The spine is declared as its **two end caps** — deliberately, since its bbox un-shears wider
     than the whole narrowing — so a first version using the declared boxes as a **pixel exclusion** let
     **186 px of a 3px diagonal stroke** through on Mangrove. **That contract is now recorded at the
     definition**, which is where 329's own contract lives.
     ⚠️ **One mutation did NOT fire the new clause and the lane said so** rather than claiming it did:
     the shipped box is derived from the same list, so dropping a term leaves the rest inside it, and a
     pre-existing clause caught it. **That limit is now stated where the clause is defined.**
     **Original:** **THE PARALLELOGRAM CORNER LAW'S SUBJECT SHRINKS AS A WORLD'S GROUND GETS BUSIER, and nothing
     inside the frost tests can fix it.** Reported by item 329's audit and correctly not acted on.
     `frost_parallelogram_item318.rs`'s corner law measures **sharp document** in exactly the region
     where the card-ink veto **cannot distinguish card ink from world ground** — the card genuinely
     draws rows and its ring in those off-rake corners, and no threshold separates them there. So the
     law is sound (it biases DOWN, fail-safe, guarded by `measured > 500`) but **degrades silently**:
     a busier ground leaves it less to measure. Today it survives on 1716–14263 px with 195–462 edges.
     ⚠️ **This is the "law satisfiable by deleting its own subject" family, one step removed** — the
     subject is not deleted by a code change but eroded by a THEME change, so it can decay without
     anyone touching the law or the product it grades.
     ✅ **Build:** it needs an ink oracle the **chrome path itself declares** — an enumeration of what
     the card draws in its off-rake corners — rather than one inferred from pixels. That is outside
     the frost tests' partition, which is why 329 stopped here. ⚠️ **Do not "fix" it by lowering the
     `measured > 500` guard**; that guard is the only thing making the erosion visible.
     **Routing:** production tier.

338. 🔴 **SIXTEEN CONSTANTS IN THE WRITING COLUMN HOLD THEIR DEVICE SIZE AS THE DISPLAY GETS DENSER —
     item 242's headline defect, found in a fourth pipeline nobody had audited.** Measured by item
     322, which correctly declined to "fix" it. `metrics.zoom` is the clamped user zoom;
     `metrics.scale` is `zoom * dpi`. **Sixteen read sites multiply by `.zoom` ALONE and never meet
     `dpi`**: `SPELL_AMP`, `SPELL_PERIOD`, `SPELL_THICKNESS`, `NIT_THICKNESS`, `PREEDIT_UNDERLINE_H`,
     `CODE_PILL_INSET_X/_Y`, `FENCE_PANEL_INSET_X`, `TABLE_CELL_PAD_X`, `TABLE_COL_GAP`,
     `TABLE_RULE_THICKNESS`, `TABLE_PAN_BAR_THICKNESS`, `IBEAM_W`, `CARET_SPACE_BAR_W`,
     `CARET_MORPH_DILATE_PX`, plus `OVERLAY_ENTRANCE_DROP_PX` which is scaled by **nothing at all**.
     ✅ **MEASURED, NOT INFERRED**, at matched logical geometry: the code pill is **6.0000 short at
     dpi 2 — exactly 2 × `CODE_PILL_INSET_X`** — and the fence panel **16.0000 short, exactly
     2 × `FENCE_PANEL_INSET_X`**. Exact to the byte of the constant.
     ⚠️ **THIS IS A PRODUCT DECISION WITH A HUMAN IN IT, WHICH IS WHY 322 PARKED IT.** Resolving them
     through `Metrics::px` changes what **every Retina display draws for sixteen separately tuned
     taste quantities**, several carrying *"TASTE TUNABLE, flagged for live review"* in their own doc
     comments. **Declaring them `Physical` would be worse** — it asserts the device grid is the right
     reference for a spell squiggle's amplitude. ✅ 1× is unchanged either way, and
     `no_length_is_resolved_against_zoom_alone` already guards the repair from keeping the wrong
     factor. ✅ **A ledger law holds them meanwhile, graded from BOTH ends** (set equality against the
     file's own leftover set), so a new one cannot join by being DPI-blind and a repaired one cannot
     stay by being forgotten.
     ✅ **Build: a visual judge on 1×/2× pairs, world × construct**, then land the sixteen in one pass.
     ⚠️ **Four cannot be typed without editing `chrome/**`** (`CODE_PILL_INSET_X` is read completely
     raw at `chrome/popover.rs`; `IBEAM_W`, `CARET_SPACE_BAR_W`, `CARET_MORPH_DILATE_PX` at
     `chrome/preview.rs`). **Routing:** production tier, then the user's eye.

339. ✅ **LANDED (merged 2026-08-08) — WIDER THAN FILED, and there was a SECOND instance one layer
     down.** `Mode::ScreenshotFrames` had **no classifier arm at all**, so it fell into the
     plain-`Screenshot` bucket whose list treats **every** hook as honoured: canvas, dpi, **`--keys`**,
     the per-frame render hooks, `--root`, `--workspace` and `--default-folder` were all accepted and
     discarded. ⚠️ **`--keys` is the one this door over-promised beyond `--screenshot-app`**, and it
     needed its own refusal because it is **not in the `SuppliedHooks` table**: every other
     out-producing mode threads keys, and this is the first that structurally **cannot** — the document
     is a stationary backdrop for the App's scheduling loop, never a replay.
     ⚠️ **AND THE SAME SHAPE EXISTED BELOW THE CLI:** `capture_frames_async` **never called
     `set_dpi`**, so even a correctly-threaded `--capture-dpi` would have been a no-op **at the
     renderer**. Fixing the plumbing alone would have left the flag still ignored.
     ✅ **Measured on the real binary in BOTH directions.** Before: 1200×800, 640×800 and 2400×1600@2
     all reported canvas 1200×800 with identical wrap counts. After: **12** wrapped rows at 1200×800,
     **30** at 640×800, **12** at 2400×1600@2 — matching the dpi-1 baseline, the documented meaning.
     ✅ **`docs/harness-reach.md` now enumerates EVERY `Mode::*` capture door and whether it honours
     canvas and dpi**, so "which doors can I ask this of" has one answer in the file briefs must read.
     **There is no third silently-discarding door.** **Original:** **`--screenshot-frames` HAS ITEM 334's DEFECT, UNFIXED.** Found by 334's lane while diagnosing and
     deliberately left alone. `Mode::ScreenshotFrames` has no canvas/dpi fields and the identical
     plain-`Screenshot` bucket fallthrough, so it too accepts geometry flags and discards them
     silently. It is a **Hidden** flag, which is why it is lower priority — and also why nobody would
     notice. ✅ **The fix is now mechanical: copy `CaptureKind::ScreenshotApp`'s shape** — its own hook
     list, canvas and dpi carried on the spec. ⚠️ **Verify by measuring geometry, not by asserting a
     field was set** — 334's proof was that a narrower canvas produced genuinely more reflow.
     **Routing:** production tier.

340. ✅ **LANDED (merged 2026-08-08) — one owner, one parked note, and the note was UNDERCOUNTING
     its own blast radius.** `render/tests/mod.rs` holds `SETTINGS_VIEW_PARKED_WINDOW_ROWS` and
     `settings_overlay_view`; both files keep a one-line wrapper, so their nine call sites are
     untouched. ✅ **The 12 was NOT un-parked** — that would force item 327's open product question —
     and the note survives at the constant's single site rather than being deleted.
     ⚠️ **Mutation-proving the dedup found the old comment wrong:** setting the parked value to the
     shipped 31 reddens **THREE** range-rail laws, not the two it claimed (more laws were added since
     it was written), **plus** `settings_row_reach_law` with item 327's exact predicted panic text. A
     parked value's blast radius grows silently while its note stays still.
     **Original:** **`settings_view()` IS DUPLICATED IN TWO TEST FILES, BOTH CARRYING THE SAME PARKED NOTE.** Found
     by item 335's lane. `settings_row_reach_law.rs` and `range_rail.rs` each declare their own copy,
     and each hardcodes `overlay_window_rows` to `ViewState::base()`'s default of **12** instead of the
     shipped **31** — the simplification that currently shields item 327's real defect from its own
     law. ⚠️ **Two copies of a fixture mean two places to un-park it, and 335 proved the parked value
     is load-bearing**: correcting it reproduces 327's defect. ✅ **Build:** one owner for the fixture.
     ⚠️ **Do NOT un-park the 12 as part of the dedup** — that forces item 327's still-open product
     question. Make it a parameter with the parked default named at one site, so un-parking later is a
     one-line change in one place. **Routing:** production tier.

341. ✅ **LANDED (merged 2026-08-08) — and the widening exposed a TEST THAT COULD NOT SEE ITS OWN
     BUG, which is the fifth time a law's scope has been the defect.**
     ✅ **The census was exactly the three named** — the first time this class did not find more than
     briefed. It also read the four sibling caret files, found nothing needing reclassification, and
     **deliberately did NOT add them to the swept set**: the parser matches line SHAPE rather than scope,
     so a function-local const in a partition-forbidden file would register as an unfixable offender,
     forcing either a red gate or a dishonest exclusion for code it never verified. **Reported rather
     than quietly done** — and they can be folded in cheaply later.
     ✅ **`_MIN_AREA` IS AN AREA, so it scales as the SQUARE of the linear factor.** The code already knew
     that implicitly, so it was not a live bug — but **typing it `Logical` and resolving it linearly was
     exactly the briefed hazard: correct at `px = 1`, where `1×1` equals `1²` and the two are
     indistinguishable, and silently under-scaled everywhere else.** It gets a new by-kind family,
     **`Area`, with ONLY a `px2` door and no linear arithmetic**, so a future misuse cannot resolve it
     through the wrong power. The declaration law recognises it by type with its own non-vacuity floor.
     🔴 **THE FIND: the pre-existing unit test could not see a magnitude bug in the area term AT ALL.**
     The width is clamped to the width floor **before** the area term runs, so area growth can only push
     the width up, never down — a wrong area formula never fails that check. **Proven rather than argued:
     mutating `px2` back to linear left all 221 caret tests green.** The missing law is now written and
     mutation-proven before being believed. ✅ **Its companion stayed green under the same mutation,
     correctly** — `px = 1` is exactly where linear and quadratic coincide, **which is why this would have
     been invisible at the one scale captures run at by default.**
     ✅ **Evidence at the scale that matters:** two release binaries, one built **from the base commit
     rather than stashed**, 24 cells across three proportional worlds × two caret modes × 1×/2× × two
     zooms, caret landed on a comma to engage both floors. **48 of 48 files byte-identical, including all
     eight dpi-2 cells.**
     ✅ A clippy ceiling was cleared by **extracting an owner rather than asking for an exception**, so no
     ledger number is owed. **Original:** **A SCOPE RESIDUAL ONE DIRECTORY OUT, the same shape that hid `TEXT_LEFT`/`TEXT_TOP`.** Found by
     item 322. `src/render/caret_body.rs` declares `CARET_VISUAL_BODY_MIN_W`, `_MIN_H` and `_MIN_AREA`
     as bare `f32` — multiplied by the correct recovered scale, so **not a defect today**, but outside
     the declaration law's swept set. ⚠️ **A law's scope has now been the defect FOUR times** (items
     315, 323, 322 and this), and each time widening it enumerated real work. ✅ **Build:** widen the
     sweep to the caret files and classify what it enumerates, one family at a time, naming the
     pipeline that already scales each. **Routing:** production tier.

342. ⚠️ **MEASURED, ORACLE REPAIRED, NO FIX — THE REMAINING QUESTION IS A TASTE CALL AND IT IS THE
     USER'S (2026-08-08).** The clip is real and confirmed; **both the board's hypothesis and the
     orchestrator's were wrong about why.**
     ⚠️ **THE SHAPED-EXTENT THEORY IS FALSE, MEASURED.** The content side **is** advances
     (`overlay_footer_content_px` reduces by `run.line_w`), but the budget side is **a bare constant
     with no font term at all** — `CARD_MAX_W = LogicalGrowOnly(520.0)` clamped to the window. Unioning
     every hint glyph's real swash placement against the advance total across the roster: the two agree
     within **±1.1px** and **the ink is usually the NARROWER of the pair** (Potoroo 803.2 advances
     against **802.0** of ink). **Measuring extents would make every number worse.** The mismatch is
     font-versus-fixed-cap.
     🔴 **THE DEEPER FINDING, and it is item 321's exact shape: THE CAP IS INCOHERENT ABOUT ITS OWN
     REFERENCE TIER.** `LogicalGrowOnly::px` is `self.0 * scale.max(1.0)`, so **below scale 1 the cap
     keeps its DEVICE width while the text shrinks.** At the shipped default (zoom 0.8, dpi 1) the card
     runs **25% roomier relative to its own text** than at any scale ≥ 1 — where the fit becomes a pure
     ratio, **1.0121 at scale 1.0, 1.6 AND 2.0 to four decimals.** So **zoom 1.0 on a 1× display clips
     identically**; 2× is merely how it is reachable at the shipped zoom. Proven by deleting the clamp
     and watching the same ratio appear at 0.8/1×. ⚠️ **The menu-bar arm is NOT a gate** — both arms
     measure identical ratios, since nothing in the width budget reads the reserve (that part of the
     item's premise was wrong, harmlessly).
     ⚠️ **TWO PREMISE CORRECTIONS, ONE OF THEM THE LANE'S OWN.** Widening the sweep *appeared* to find
     the palette's `Command` hint clipping on all five monospace-chrome worlds **and to reproduce item
     319's sibling residual to 0.1px on the verbatim same hint string.** Both dissolved: **the command
     palette FACETS**, so `overlay_geometry` routes it to the wider `CARD_MAX_W_FACETED` cap and the
     flat column was never its budget. Driven through each kind's real owner, the overflow set is
     **exactly Potoroo and Firetail × Keybindings**. **So 342 is confirmed and 319's residual is
     PREMISE-FALSE** — measured against an owner that hint never rides.
     ✅ **319's font exclusion is DELETED, not adjusted, and removal was proven load-bearing** (emptying
     the ledger reddens the law with exactly the board's number). In its place a **two-sided ratchet
     rather than an exclusion**: every cell graded, and a pair overflows **iff** ledgered at the pinned
     ratio — so a new clipping world fails, a ledgered pair that stops clipping fails, and a drifted
     ratio fails. Sweep: 21 worlds × 18 carded kinds × 1×/2× × both bar arms × zoom {0.8, 1.0} =
     **2880 graded cells**, workspace kinds excluded by their owner. Firetail's deficit is **+38.4px**
     against Potoroo's **+9.6** because a `Bars` world's text hpad differs from `Pane`'s — which is why
     the ledger is keyed by **world**, not by face.
     ✅ **Six mutations, each with its match count asserted and each showing a `test result:` line. Two
     BRACKET the number a decision needs: raising the cap to 545 clears both worlds; 540 does not.**
     🔵 **THE QUESTION IS THE USER'S AND IT IS DIFFERENT FROM 327's** — 327 asks who yields *inside* the
     accessory column; this asks whether the card's **own chrome** gets a yield at all, and whether the
     width cap should be font-aware. **But it is UPSTREAM of 327**, because it changes the width 327 has
     to divide. **Original:** 🔴 **POTOROO AND FIRETAIL CLIP THEIR KEYBINDINGS HINT AT 2×, MENU BAR OFF — WHICH IS macOS'S OWN
     DEFAULT, so this is live on the dev platform.** Found by item 319's new sweep on its first run,
     with no tip needed: the hint alone measures **803.2px** against each world's own (different)
     column budget. Clean at 1×, which is why every capture missed it.
     ✅ **THE DETERMINING PROPERTY IS MEASURED, NOT GUESSED: both are the worlds whose CHROME font is
     `"Monaspace Xenon"`** — and **nine other worlds with a monospace chrome font do NOT clip**, so it
     is that face's wider advance, not monospace generally. 319's law excludes them **by that measured
     property rather than by a name list**, with a non-vacuity count (`excluded == 8` = 2 worlds × 2 dpi
     × 2 bar states), so the exclusion cannot silently widen.
     ⚠️ **This is a font-metrics-versus-fixed-budget question**, which is why 319 flagged it instead of
     absorbing it. It is the **same shape as 319's own zoom-1.0 residual** (Mangrove's hint overflowing
     by ~7.7px): a width budget computed from advances against text whose glyph cells are wider.
     **Consider fixing both together, and check whether the budget should measure shaped extents rather
     than advances.**
     ⚠️ **Fixing it by shrinking the hint text is the dishonest repair** — the hint is the discoverability
     affordance. **Verify** across the roster × 1×/2× × both menu-bar arms at the shipped zoom, and
     **remove 319's exclusion as part of the fix** so its non-vacuity count fails if the exclusion is
     left behind. **Routing:** production tier.

343. ✅ **LANDED (merged 2026-08-08) — route (a): the frost's box is the DRAWN SURFACES, and the
     cross-section goes 576 → 476.4 logical with 0.00 slack per side plus the feather.**
     ⚠️ **THE ASYMMETRY PREMISE WAS FALSE AND IT WAS THE ORCHESTRATOR'S.** The item predicted 40 left /
     177 right, the 177 being "149 of dead card width past even the widest line". **That width is not
     dead: the selected row's chevron mark stands at the card's outer edge** (x 1036.5–1049.5 against a
     card ending at 1060.0), and `overlay_line_glyph_box` is blind to it **for exactly the reason the
     item says it is blind to rules — it is a rect, not a shaped run.** Measured in the shape's own
     un-sheared frame (the only frame where "left" and "right" are properties of the box) **the true old
     slack was 49.3 / 50.3 — near symmetric.** So the fix is symmetric because the defect was, and the
     warning about a symmetric trim leaving the larger half was warning against nothing.
     ✅ **NO MASK WAS INVERTED** — the constraint items 329 and 319 each paid a round to learn. Every term
     comes from a production owner: rules from a new `x_reach()` beside `rules_ink`, rails from the same
     `overlay_rails` the pointer hit-test reads, text from an `overlay_panel_bands` **the emitter now
     loops over**, so the seat glyphon is handed and the seat the frost measures are one object.
     ✅ **Two shape decisions worth reusing.** The spine is **two END CAPS, not a bbox** — a bbox
     un-shears to `weight + |shear|·h` = **78.5px, larger than the entire narrowing**, and since the
     shape is convex and the un-shear affine both caps contain the segment exactly. And the mark is asked
     of **every** row: item **164's transaction law** caught the first version reading the logical
     selected display row from a module not allowed to, and the rewrite makes the box
     **selection-independent**, so a selection move can no longer invalidate the cached backdrop.
     ✅ **The tightness allowance is 2.0 logical px, bounded from BOTH ends rather than picked** — above by
     shaper float noise and the advance-versus-cell gap, below by the composition's smallest authored
     horizontal length of **7px**, so any reintroduced structural slack moves the excess by ≥7 and cannot
     hide under 2. Measured excess **0.00 at every cell**, because the box is derived rather than compared.
     ✅ **318's coverage floor untouched and green at 1.0000.** Its **corner** law was re-aimed with sound
     reasoning: "exactly two corners short" held only while the box was bounded below by the card's, and
     a narrowed box leaves a corner short for a second reason. **What survives at any width is the
     property that separates the shapes** — a union contains the card's box so all four corners are
     frosted, a parallelogram holds at most two — so it is now `full <= 2`.
     ✅ **Byte-identity measured against a real build of `main`'s source, not asserted:** Wagtail, Galah,
     Quokka, Tawny, Bilby, Kite and **Paperbark** (enrolled, `Rules`, shear 0) identical, both
     `Frost::Full` surfaces identical, Mangrove and Magpie differ by design with **byte-identical
     sidecars**. `blur/extent.rs` was **carved** into a new `blur/narrow.rs` rather than grown.
     ⚠️ **TWO OF ITS LAWS WERE RED ON THE COMBINED TREE AND ITS OWN GATE NEVER RAN, so neither had been
     seen.** (a) The containment law **demanded the impossible in one swept cell**: at shear −0.3 the
     fixture's corner (212,110) is not inside the **card's own** sheared shape either (that span is
     [264.5, 784.5]), so a shrink-only narrowing cannot contain a point that was never frosted. Re-aimed
     to "a corner the un-narrowed shape covered stays covered", **with its own non-vacuity count** —
     skipping uncovered corners would otherwise let a shape covering nothing pass perfectly. (b) The
     **toggle law** caught `suppress.rs`'s raw `static AtomicBool`; allow-listed with the reason that
     **there is no shipped state to make sticky** — `mod suppress` is `cfg(test)` and `frost_mode`'s
     branch carries the same attribute, both verified rather than taken from the module doc. ⚠️ **The
     allow-list is keyed by the scanner's PATH, not the basename** — a `"suppress.rs"` key left the law
     red, which is the allow-list working.
     🔵 **Route (a) is now EXHAUSTED, and that is the honest limit:** the 476 that remains is real ink —
     the upright head band (122, flush left), the foot hint (359), and the chevron at the far right.
     **Narrowing further means moving the card, the mark or the foot, which is item 342's live question
     with the user.** **Original:** 🔴 **THE FROSTED FOOTPRINT IS ROUGHLY TWICE AS WIDE AS ANYTHING DRAWN INSIDE IT — user-reported
     against Mangrove's theme picker, with a screenshot: *"there's a bit too much blur on the left and
     right sides."* The silhouette is now right (312/313/318 landed); its WIDTH is not.**

     ✅ **MEASURED AT THE PRODUCTION SEAM, NOT READ OFF THE SCREENSHOT** — `overlay_card_rect` +
     `frost_mode()` + `overlay_line_glyph_box` over the typed-query theme picker at 1200×900 / 2400×1800,
     and every figure below is **identical in logical px at 1× and 2×**, so this is a width defect and
     NOT another instance of the DPI class item 294 fixed:

     | world | card box (logical) | widest drawn LINE | frost box | silhouette bound |
     |---|---|---|---|---|
     | Mangrove | 520.0 × 452.6 | **359.0** (the foot hint) | 520.0 (no widening) | **706.9** |
     | Magpie | 520.0 × 452.6 | **241.0** | 563.2 (widened LEFT 43.2 by `footprint_box`) | **750.0** |
     | Paperbark | 520.0 | **201.0** | 520.0 (shear 0) | **576.0** |

     The candidate rows themselves measure **61.2–110.2** logical wide on Mangrove and the query line
     122.4; only the foot hint reaches 359. **So at any single row the parallelogram's cross-section is
     `card_w + 2 × feather` = 576 logical over a row carrying at most 110 logical of ink.**

     ⚠️ **THE SLACK IS ASYMMETRIC, AND THE SCREENSHOT CANNOT SHOW THAT** because the lean redistributes
     it: **40 logical px on the left** (12 text hpad + 28 feather) against **177 on the right** (149 of
     dead card width past even the widest line, + 28 feather). A fix tuned to "reduce the padding on
     either side" symmetrically would leave the larger half in place.

     **THE CAUSE IS THE CARD'S LAYOUT BOX, NOT THE FEATHER.** The frost's box is `overlay_card_rect`
     (widened only where `footprint_box` must seat upright chrome), and `card_w` comes from
     `overlay_desired_w(CARD_MAX_W…)` — a **fixed desired width clamped to the window**, with no
     relation to how wide the shaped rows actually are. On a world that draws no plate and no card
     backing, that width is invisible right up until a frost is scoped to it.
     ⚠️ **REDUCING THE FEATHER IS THE WRONG LEVER AND IS FLOORED BY LAW.**
     `the_footprint_feather_is_at_least_the_blur_it_edges` requires it to clear the Gaussian's own 16
     logical px reach, and the shipped 28 is only 12 above that floor — spending the whole margin buys
     back 24 of the 217 logical px of slack per row and reintroduces the hard edge item 312 removed.

     🔵 **THE FORK IS ALREADY SETTLED BY MEASUREMENT, so the lane does not get to choose wrong.** On
     `Rules` the card genuinely DOES draw at its full width: `overlay_rules` emits every rule at
     `(band_x, band_w)`, and `band_w() == card_w` for every non-workspace picker. **So a frost narrowed
     to the GLYPH ink would strand Paperbark's rules over sharp document** — and `overlay_line_glyph_box`
     is blind to it, because rules are rects and it reads shaped runs. On `Diagonal` nothing is drawn
     full-width (the spine is a rail; the rows and both chrome bands are shaped runs). **The box must
     therefore be derived from the DRAWN SURFACES — glyph runs ∪ rules ∪ rails — never from the glyphs
     alone and never from the layout box.**

     **Two routes; (a) is recommended.** **(a) Narrow the FROST's box** to the drawn surfaces' own union,
     leaving the card's layout box and its pointer hit region exactly as they are — which is already the
     established split (item 312 decided the frost's extent and the hit region are separate quantities,
     and `footprint_box` already moves one without the other). **(b) Narrow the CARD** on a plateless
     world — a bigger change that moves row elision, the footer's yield and the hit-test, and collides
     with 342's live hint-budget work.

     **Verify — the two laws must bound the box from BOTH sides, and neither is sufficient alone.**
     `frost_parallelogram_item318`'s ink-coverage floor is what catches an over-narrow box and it **stays,
     unweakened**; add a TIGHTNESS law from the other end — the frost's box exceeds the drawn surfaces'
     union by no more than the feather plus a stated allowance — because a coverage floor alone is
     satisfied perfectly by the 520px box that prompted this item. ⚠️ **The tightness law's surface union
     must come from a production owner, not from `CardInk`** — that oracle is a VETO and does not invert
     (items 329 and 319 both burned a round on exactly that). Sweep the enrolled roster from
     `footprint_frost_applies` (never a name list — Paperbark is the shear-0 member that keeps this
     honest) × 1×/2× × both `MENU_BAR_ON` arms.
     **Routing:** production tier, then the user's eye — the final width is taste and no capture settles
     it. Touches `render/blur/extent.rs`, `pipeline_prepare.rs`; READS `chrome/overlay_rules.rs` and
     `chrome/diagonal/`. **337 grades the same silhouette and 342 is live in the hint's width budget —
     sequence, never pair.**

344. ✅ **LANDED (merged 2026-08-08) — and the best of the four public fixes made the number
     UNSTATEABLE rather than correct.** `GUIDE.md` now writes **`{{count:worlds}} worlds, one chord
     away`** — a new token kind beside `{{key:}}`/`{{cmd:}}`, answered by `theme::THEMES.len()` at open
     time, so **the digits are not in the file.** `ACCESSIBILITY.md`'s "14 curated theme worlds" became
     **"Every curated theme world"**, because the load-bearing word was always *every* — the count was
     deleted, not corrected. `platform.md` and `GUIDE.md`'s conceal description were reworded.
     ✅ **TWO MORE PUBLIC SURFACES THE CENSUS NEVER NAMED, found while fixing those:** `site/guide.html`
     carried the same wrong count and the same caret-only paragraph, and **`REFERENCE.md` +
     `site/reference.html` printed the retired `C-x #` chord — sourced from the flag roster, so `--help`
     printed it too.** Fixed at the source and regenerated; that diff moved **exactly two lines**.
     ⚠️ **THE CENSUS I HANDED OVER WAS WRONG IN PLACES, AND THE LANE CHECKED RATHER THAN INHERITED.**
     Its claim that two owners' field names *"do not exist in source at all"* is **FALSE** — they exist
     deliberately, in a test pinning the names each field carried **before extraction**. The real defect
     is ambiguity: the doc lists pre-extraction names without saying so, so the fix **labels** them
     rather than rewriting them, since correcting them into current struct fields would duplicate a
     ratchet that already pins them.
     ⚠️ **AND A CLAIM IN THE ORCHESTRATOR'S OWN BRIEF WAS FALSE:** I told the lane THEMES.md already had
     roster-derived laws for its membership tables. **Those laws are over WORLDS.md. THEMES.md was not
     even embedded** — it is now, with four laws over it.
     ⚠️ **Several instances were WORSE than reported:** ten sidecar modes missing not nine, seven CJK
     worlds not five, eight wrong THEMES.md rows not seven — and one of those **named a world that
     carries no frame at all** while missing the one that does. One entry was **already correct**:
     WEB.md's "20 worlds" was right, and the contradiction was that two other docs disagreed with it.
     ✅ **Twelve laws in four files, eleven mutations**, each with its match count asserted, its build
     confirmed and a `test result:` line. ⚠️ **One mutation SURVIVED and the lane did not believe it** —
     declaring a decomposed module as a directory passed, because most decomposed modules are **both** a
     file and a directory. The module doc had **overclaimed** what the arm decides; it was corrected and
     the arm re-proved against a module with no sibling directory.
     ✅ **THREE DEFECTS IN ITS OWN LAWS, each caught by a GUARD rather than by re-reading:** a harvest
     that read "20 worlds" as stating no count, **a cell splitter that swept one row of fourteen while
     reporting clean** (the enrolment-predicate shape), and a Values check looking for a bare variant
     tick where the column ticks a payload shape. All three are recorded in the code as tripwires.
     ✅ **Both gate failures on the way were real**, and the second is the standing lesson in miniature:
     **web-smoke caught laws that could not compile in the wasm test binary** because their embedded
     sources are gated `not(wasm32)` — which the native gate is structurally unable to see.
     ✅ `sidecar.rs` came back to its mark **exactly** rather than asking for a raise: the doc comment
     that grew it moved to the law that reads the writer.
     🔵 **One content decision left, not a drift fix:** whether `docs/app-domains.md` should also list
     each owner's CURRENT struct fields. That is a question about which names a reader should see.
     **Original:** 🔴 **THE DOC CENSUS IS 29 CONFIRMED CONTRADICTIONS ACROSS NINE FILES, AND FOUR ARE IN PUBLIC,
     USER-FACING DOCS.** Produced by item 302's own fan-out audit; **the orchestrator independently
     re-verified the four public ones against source** and they hold. This is filed separately because
     the census is far larger than 302's brief anticipated (it named five instances) and **must not be
     lost if that lane cannot finish all of it.**
     🔴 **VERIFIED BY THE ORCHESTRATOR, and these are the ones a reader sees:**
     `pub const THEMES: [Theme; 20]` — but **`GUIDE.md:292` says "Nineteen worlds, one chord away"** and
     **`ACCESSIBILITY.md:30` says "14 curated theme worlds, each contrast-law-tested"** (WEB.md:137
     correctly says 20, so the docs contradict each other). **`docs/platform.md:40` documents the git
     door as `"Finish Buffer"` (`C-x #`)** when the palette label is **`"Finish file"`** and that chord
     default is **retired** (`Cmd-W` now). `GUIDE.md:303` says conceal reveals *"except on the line your
     caret is on"* — **a selection touch also reveals**, which `wysiwyg_reveals_selection_widens_every_kind`
     proves and `docs/markdown.md` documents.
     ⚠️ **The rest, reported by the audit and NOT independently re-verified by the orchestrator** (so
     treat each as a hypothesis to check, per the standing rule): `CAPTURE.md:1332` claims only
     `rust`+`python` syntax when `Lang` has **20** real variants; `CAPTURE.md:1434/1593` undercount
     `OverlayKind` (**21** variants) and omit nine real modes from the sidecar's `mode` enum;
     `CAPTURE.md:1592` omits three fields the sidecar writer actually emits; `docs/render.md:27` says
     "16 worlds"; `docs/render.md:98-99` says `History` returns `None` from `workspace_shape()` when it
     now returns `Some(TimelineOverComparison)` — **shape 4, an invariant later code invalidated**;
     `RELEASING.md:345` says `ubuntu-latest` when the workflow pins `ubuntu-22.04` **and the same file's
     line 334 records that as resolved**; `ARCHITECTURE.md` lists two directory modules as flat files and
     misstates `daemon.rs`'s cfg gate (it is also compiled out under `mas`); `THEMES.md` carries **nine**
     stale deviation rows plus a JA ladder table missing 5 of 20 worlds; and **`docs/app-domains.md` has
     four owners whose field enumerations no longer match their structs** — two owners' field *names* do
     not exist in source at all, and its own totals no longer reconcile (1,308 against a stated 1,310).
     ✅ **THE LEVER IS 302's AND IT APPLIES HERE: a doc stating a checkable fact should be a LAW.** A
     world count, a variant count, a field enumeration and a sidecar field list are all derivable —
     `reference.rs`'s drift laws are the working precedent, and `THEMES.md` already has roster-derived
     laws for its membership tables. **Prefer generating or law-checking over rewording; a reworded
     number rots on the same schedule.** ⚠️ **But heed the generated-document hazard:** generating from a
     roster moves the error from transcription to SOURCING, and this repo has shipped three such errors
     in one pass. **Spot-check generated entries against the code they claim to describe.**
     ⚠️ **The four public ones are the priority** — they are what a user or a screen-reader user reads.
     **Routing:** production tier; sequence behind item 302, which owns the same lever.

345. 🟡 **PREPPED AND HELD ON A BRANCH — `claude/item-345-caret-pitch`, ready to merge in one
     command, AWAITING THE USER'S WORD (2026-08-08). Not merged, deliberately: the orchestrator
     committed to showing the captures first.**
     ✅ **Premise reproduced through the real render pipeline before anything changed** — Currawong's
     `caret_block_w()` returned **14.4 against the face's own advance of 12.0**, and Iosevka is still the
     display face of exactly those two worlds.
     ✅ **THE FLOOR IS GENUINELY REDUNDANT, MEASURED RATHER THAN INHERITED** — the whole safety argument
     for removing it, so the lane was told not to take item 302's word. Empty line **14.4**, end-of-line
     **14.4**, degenerate collapsed cell **14.4**, and **tab 84px** (the font's own wide glyph, where the
     floor was already a no-op). All four rescue paths return `char_width` **regardless of face**, because
     `CARET_W == CHAR_WIDTH` by definition — **the floor and the degenerate-cell rescue were always the
     same number.** No case regresses.
     ✅ **The fix drops the mono arm entirely**, so the block width is the real shaped advance
     unconditionally, as the proportional arm always was. **Per-shot, from sidecars:** 14.400→12.000 at
     1× and 28.800→24.000 at 2× on both worlds — exact doubling, no rounding artefact.
     ✅ **The law derives BOTH sides independently** (`mono_pitch_em` reads `hmtx ÷ units_per_em` via
     skrifa, nothing `render::caret` computes) and sweeps `Pitch::Mono` from the roster with a presence
     floor. Mutation (reinstating the floor) reddens it **by name**; the other three faces were then
     isolated under the identical mutation and all measured **diff = 0**, confirming the floor is a true
     no-op for Plex Mono, JetBrains and Monaspace Xenon exactly as the arithmetic predicts.
     ✅ **The Tawny-only predecessor was corrected rather than deleted**, and now says outright that its
     pitch happens to equal the cell so it alone could never have caught this — **which is what let this
     ship.** No other law reddened.
     🔵 **One number for the merge:** `src/render/caret.rs` 977 → **972** (a decrease). **Original:**
     🔴 **THE BLOCK CARET IS 120% OF THE GLYPH IT SITS ON, ON TWO SHIPPED WORLDS — a 2.4px overhang
     into the next character at zoom 1.** Found by item 302 while repairing a comment that asserted the
     opposite, and **the numbers are measured, not inferred.** `caret_block_w`'s mono arm keeps a
     historical `.max(caret_w)` floor, and `metrics.caret_w` is a fixed `CARET_W` that is
     **face-INDEPENDENT** — so on any mono face narrower than 0.60 em the floor raises the block **past
     the cell it is supposed to fit.** Measured off the shipped `hmtx`: Plex Mono **0.60**, JetBrains
     **0.60**, Monaspace Xenon **0.62**, **Iosevka 0.50** — and `CURRAWONG` and `CASSOWARY` carry
     Iosevka, drawing **block=14.400 over cell=12.000** while every other mono world matches exactly.
     ⚠️ **THE EXISTING LAW PINS TAWNY ALONE** — one hand-picked mono world, which happens to be one of
     the two 0.6-em faces. `facepitch.rs`'s own doc records **the same hardcoded face list losing Iosevka
     once before**, so this is the second time that shape has cost this repo something.
     🔵 **IT IS A TASTE CALL AND THE LANE CORRECTLY DID NOT DECIDE IT.** The candidate fix is dropping
     `.max(caret_w)` so the mono arm becomes the proportional arm — which also makes the floor redundant,
     since `col_x_and_advance` already rescues a degenerate cell. **But it NARROWS a shipped world's
     caret**, and the caret is this design's one accent. **Put the before/after to the user on Currawong
     and Cassowary at 1× and 2×.**
     ✅ **The law is written and mutation-ready** (item 302 reverted it rather than landing red): it
     sweeps `bundled_display_faces()` filtered to declared `Pitch::Mono` — **roster-derived enrolment,
     presence floor ≥ 2, and BOTH sides derived** (the caret's pitch from `CHAR_WIDTH / FONT_SIZE`, the
     face's from its own `hmtx` over its own `units_per_em`). **It failed correctly on first contact,
     which is how the defect surfaced.** ⚠️ **Land the law WITH the fix, never before it** — and do not
     narrow the law to the faces that pass. **Routing:** production tier, then the user's eye.

346. 🟡 **THE MECHANISM WAS ALREADY DONE — premise false, and the premise was the ORCHESTRATOR's.
     A REAL LAW LANDED ANYWAY, AND THE TASTE CALL IS THE USER'S (2026-08-08).**
     ⚠️ **`SELECTED_SPINE_WEIGHT` EXISTS NOWHERE IN THE TREE** — grep finds it twice, both times in item
     131's own note — and the mark's weight has been per-world theme data since 131e landed it. Verified
     independently: both commits are ancestors of `main`. ‼ **The brief quoted 131e's landing note saying
     the mark landed and then quoted a note TWO PARAGRAPHS LATER in the same item saying it had not.**
     That contradiction was visible without any grep, and 131's note is now corrected in place.
     ✅ **The brief's other half was right and is what landed: the DRAWN presence floor did not exist, and
     its absence was real.** One law counted bytes that **MOVED**, which a lane shifted by one level
     satisfies; another graded only the **ORDER** of two worlds' ink, which two marks scaled together
     toward nothing preserve. **Both are the satisfiable-by-deleting-its-subject shape.**
     🔴 **THE LANE'S FIRST VERSION OF THE FLOOR SURVIVED ITS OWN MUTATION, and the reason generalises:**
     at an eighth of the shipped stroke **a sub-pixel quad is not absent — the SDF spreads it**, so the
     wash still peaked at ΔE 28 against an absolute floor picked by halving the shipped numbers. **An
     absolute ΔE on a rendered cell was the wrong UNIT as much as the wrong number.**
     ✅ **The floor that holds is RELATIVE AND LOCAL:** for the darkest cell in the lane, how far it moved
     from **its own unselected ground** as a fraction of the distance to the ink the mark is painted in,
     plus a count of cells past half-way. Shipped Magpie reads **0.67 at 1× against a 0.5 floor** with 10
     covered cells against 4; the wash reads **0.30 with zero.** Both quantities are distances between two
     readings of the same cell, so **no byte is compared to a theme constant** and the law survives a
     backend that rounds differently.
     ✅ **Mangrove byte-identical across all five candidate builds at both scales**, proven by rebuilt
     binaries and matching hashes rather than asserted. ✅ **No theme number changed** — Magpie still ships
     `1.25/4.5/0.55`. **Original claim: A LIVE USER CONSTRAINT, LIFTED OUT
     OF ITEM 131's BODY WHERE IT SAT UNADDRESSED: the Magpie selected-row mark *"needs to be thinner and
     more elegant"* — the user's own words about a real screenshot.** Item 131e's lane flagged it rather
     than fixing it unasked or silently omitting it, which was right; it is filed here because the
     mechanism is clear and the precedent is already in the tree.
     ✅ **THE DESIGN FINDING IS ALREADY MADE: one glyph cannot serve both worlds.** Magpie's display face
     is **`Bitter`**, an editorial slab serif whose register a heavy geometric mark contradicts; Mangrove's
     is **`JetBrains Mono`**, where a crisp geometric mark is correct. **So the mark's weight and form
     belong in theme data beside the world's face** — and today they do not: `chrome/diagonal.rs` carries a
     shared `SELECTED_SPINE_WEIGHT = Logical(3.0)` and the theme layer holds no per-world selected-marker
     weight at all.
     ‼ **THE NOTE'S OWN WARNING IS THE FAILURE MODE: do NOT tune the single shared constant until Magpie
     looks right and call it done.** That makes Mangrove wrong — the same defect one world over.
     ✅ **The precedent to copy is 131e's row mark**, which lives in `ListStyle::Diagonal`'s **variant
     payload** (`DiagonalSpine { direction, mark: DiagonalMark { weight, reach, aperture } }`) so a world
     cannot author an orientation without a mark — **the never-half-apply guard is the COMPILER's, not a
     law's.** ⚠️ **Check whether `DiagonalMark` should simply GAIN the selected weight** rather than growing
     a second parallel structure: two structures describing one world's mark is the two-owners-agreeing-by-
     coincidence defect this board carries several instances of.
     ⚠️ **Verify:** Mangrove **byte-identical** (measured against a baseline binary, never `git stash`); a
     roster-derived law that the weight is per-world data rather than a shared constant; and **a PRESENCE
     floor beside it, because "thinner" is satisfied perfectly by a mark that has vanished** — this repo
     has shipped exactly that shape. **Sweep 1×/2×: a 1.25px hairline at dpi 1 is the case to watch.**
     🔵 **CLOSES ON THE USER'S EYE.** Two or three candidate weights, not a sweep of ten, with the
     one to ship named and argued. **Routing:** deep tier, then the user.
347. ✅ **CLOSED (merged 2026-08-08) — PREMISE FALSE: THE ~860 WAS TWO CAPTURE DOORS DISAGREEING ABOUT
     ZOOM, not a property of the card.**
     ✅ **The card does NOT draw nothing.** At narrow widths it draws its category rail, a full-card
     selection band, and a foot hint naming the way forward. The "vanishing" is the **documented narrow
     STAGING regime**: for a rows-in-pane workspace the rows live in the content pane, so at the summon
     stage they are correctly not planned — the mechanism's own comment says a narrow window stages them.
     **One more chord reaches the rows at every reachable size:** 28 rows at 500px, **6 rows at 464×288,
     the app's own enforced minimum window.** There is no width at which the list is unreachable.
     🔴 **THE 860 WAS THE ORDINARY DOOR'S ZOOM.** That door pins **1.0** as a byte-stable baseline; the
     live app launches at **0.8**, and nothing said so. **Match the axis and the two doors agree to 1px on
     every world swept.** The threshold is also **per-world, not one number** — at the shipped launch zoom
     it spans **670–739** across the roster (Bombora narrowest, Firetail widest).
     ✅ **Two axes checked and found NOT to matter here, worth recording because they mattered elsewhere:**
     the **menu-bar arm does not move this boundary at all** (unlike item 327's, because nothing in the
     wide test reads the reserve), and **1×/2× are identical in logical px on both doors.**
     ⚠️ **A plain `--screenshot` is also not hermetic** — a first run picked up the dev host's own config
     and rendered a different world, so any un-configured replay measurement of this was host-dependent.
     **Both facts are now in `docs/harness-reach.md` and `CAPTURE.md`.**
     🔴 **BUT A GENUINELY BLANK CARD EXISTS, AND NOT WHERE THIS ITEM PUT IT — now the third product
     question (see OWED).** 7 cells where a stage plans no rows **and** draws no other region, all at the
     **authored zoom maximum** in the two smallest windows. Both bounds are the product's own, so the
     corner is reachable. **Pinned as a two-sided ledger**, so whichever floor lands the law reddens and
     the entries are deleted rather than kept.
     ✅ **The law's presence floor is the clause to reuse: at the same window, zoom and scale the OTHER
     stage must have rows.** Without it, *"no rows here"* is satisfied perfectly by rows that exist on no
     stage at any width. **Original:** 🔴 **THE SETTINGS CARD PLANS ZERO ROWS BELOW ~860 LOGICAL PX ON ITS SHIPPED SHAPE — a narrow-window
     cliff, not a yield.** Found by item 174's slice 2 while measuring the accessory budget, and reported
     with the code untouched. On the ordinary capture path with `workspace: true` — **the reachable arm,
     since `workspace_shape()` answers `Some(RailOverRows)` unconditionally for Settings** — the card
     plans `rows: []` and draws **no card at all** below roughly 860 logical px, reappearing at 900 with 20
     rows. With `workspace: false` it plans 20 rows down to 700, **but no summon can reach that state.**
     ⚠️ **This is ADJACENT TO item 327 and is NOT the same defect:** 327 is about which lane yields inside a
     card that still exists. This is the card vanishing. It is also **why slice 2's yield law grades a
     value-less picker rather than a narrowed one.**
     ⚠️ **Check the premise before building** — it is one lane's measurement on one door, and item 335
     already found this law family grading a state the product cannot reach. **Establish what a real
     narrow window does through `--screenshot-app`, since Settings entry is a live-App effect.**
     ✅ **Build:** find where the row budget goes to zero and decide whether the floor is a minimum card or
     a refusal to summon. ⚠️ **A card that silently draws nothing is worse than either** — if the honest
     answer is a product decision about minimum width, **hand it back with the numbers rather than picking.**
     ✅ Slice 2 also repaired a neighbouring fixture that hardcoded `workspace: false`, folding a state no
     summon can reach; it now derives it from `workspace_shape()`. **Routing:** production tier.
348. ✅ **LANDED (merged 2026-08-08) — one owner, six files each 13 lines smaller, and a law against a
     seventh.** Six copies confirmed independently rather than trusted from the brief's grep; none had
     already gone.
     ✅ **THE FIELD-BY-FIELD DIFF WAS THE PART WORTH DOING FIRST.** All twelve fields matched byte-for-byte
     across all six **except `zoom` and `scroll_sensitivity`**, which two files deliberately vary — the rail
     sweep parameterises both, the row-reach probe parameterises zoom alone. **Those stayed parameters
     rather than being forced to agreement**, which is precisely what a dedup that changed a fixture's
     behaviour would have done silently.
     🔴 **AND THE SHARED CONSTANTS ARE NOT INERT — proven, not assumed.** Lengthening `project_root` in the
     new owner reddens `settings_row_reach_law` **by name**, which is exactly item 327's documented
     mechanism: **the project root's own length moves the accessory-column budget.** So a dedup that had
     normalised one file's root would have changed which width its card yields at. That mutation is also
     what proves the consumers read the new owner rather than a leftover copy.
     ⚠️ **The anti-seventh-copy law's first draft FALSE-POSITIVED on the six wrappers' own signatures** —
     a wrapper's return type textually contains the needle it scans for. Caught before landing and
     discriminated on the declaration form. Enrolment walks the test tree, never a name list.
     ✅ **35 tests green before and after, counts stated per file** (36 with the new law), every gate exit 0,
     and **no `code-health.toml` entry references any touched file.**
     ⚠️ **CORRECTION (item 351's audit): "35 green before and after" is true and MISLEADING.** The fixture
     parks `overlay_window_rows: 12` where the product sets **31**, and the in-tree note records that
     correcting it turns `settings_row_reach_law` **RED** — so **part of that green is a configuration the
     product never runs in.** The dedup was right to preserve it (a dedup must not change behaviour) and the
     parking is right to stay (un-parking forces item 327's open question), **but the count read as
     reassurance about laws that are partly vacuous, and now says so.**
     **Original:** **SIX COPIES OF THE
     `SettingsValues` TEST FIXTURE.** Reported by item 347's lane while working on something else, and
     deliberately left because two lanes were live: `workspace_item114.rs`, `workspace_plate_item234.rs`,
     `range_rail.rs`, `settings_row_reach_law.rs`, `marker_side_item303.rs`, `rules_composition_item283.rs`.
     ✅ **Item 340 is the template** — it deduplicated the SIBLING fixture (`settings_view`), put the owner
     in `render/tests/mod.rs` with the one parked value named at a single site, and left a one-line wrapper
     in each consumer so no call site moved.
     ‼ **AND ITEM 340's TRAP APPLIES DIRECTLY: a dedup that changes a fixture's behaviour is a SILENT TEST
     CHANGE.** When 340 mutation-proved its own dedup it found the old note **undercounted its blast radius
     by a law**, because laws had been added since it was written. ⚠️ **So the six may not be identical, and
     a difference may be load-bearing** — item 327 measured that the **project root's own length moves the
     accessory-column budget**, so silently normalising one file's `project_root` could change which width
     its card yields at. **Diff field by field and report differences BEFORE merging them; parameterise
     what must differ rather than forcing agreement.**
     ✅ **Verify:** every affected law green before and after with the counts stated per file, the dedup
     itself mutation-proven so the consumers are shown to read the new owner rather than a leftover copy,
     and **a no-wildcard law so a SEVENTH copy cannot appear** — item 310's sRGB-EOTF dedup is that shape
     (it removed five, found a sixth, and left a law).
     **Routing:** production tier.

349. ✅ **RAN (2026-08-08) — the session's render work is VISUALLY CORRECT, and the audit found ONE real
     defect: in the artifact meant to let the user DECIDE.**
     ✅ **Eight shots opened with the image reader — confirmed explicitly, not inferred from filenames** (a
     report written from filenames certifies something nobody saw). Every affordance was located: the
     frost's footprint has a **visible diagonal cut** with sharp text beyond it and the lean follows the
     rows' own staggered edge; **Tawny's document is fully sharp behind the right-click menu with zero blur
     anywhere in the frame**, while Mangrove's is frosted **only inside its footprint** — exactly the
     two-answer routing item 298 built; **"unavailable" is legible on Cut AND Copy** and absent on Paste and
     Select all, including inside the selection band (it measured ΔE 0.0 — invisible — before item 299);
     the footer rim reads as a **one-pixel edge, not a box**, with the fill unchanged; **nothing** is drawn
     above "Switch project…"; the selected rail is identifiable by band and thumb while the others read
     **quiet rather than disabled**; and the caret's right edge **lands exactly on the cell's own edge** in
     both "after" panels where the "before" panels visibly overhang.
     ✅ **ONE ACCENT PER FRAME HOLDS.** No frame showed two hues competing. Cassowary's selected fill, giant
     background glyph, footer rim and headings are **one green family** reading as the world's tint rather
     than a second accent; Wagtail is fully monochrome. ⚠️ **Honest caveat recorded: two shots are tight
     crops without the caret in view**, so they confirm "no second accent in the visible region", not the
     whole frame.
     🔴 **THE FINDING, and it is about a deliverable to the USER rather than the product:
     `compare-magpie-1x-before-A-B-C-D.png` cannot do its one job.** The auditor could tell "before" from
     all four candidates but **could not tell A from B from C with any confidence** at the size they render
     — and that file exists solely so a human can choose between them. **The vertex angle is what differs
     most (98.5° / 70.7° / 50.8° / 87.6°) and a vertex angle is invisible in a small crop.** Being rebuilt.
     🔵 **The law it named, unwritten as an audit should leave it: a PAIRWISE-DISTINCTNESS floor for a
     comparison set.** Today each candidate is graded against the page — its presence floor — but nothing
     asserts a minimum ΔE **between adjacent candidates at the size a human judges them**, so **a comparison
     set can pass every individual floor while being useless as a comparison.** That generalises past this
     item: any capture produced to settle a taste call is subject to it.
     **Original claim: THE VISION SMOKE THIS SESSION OWED AND NEVER RAN.** `CLAUDE.md`'s standing policy: *"Every render-touching round gets a vision-smoke:
     affordance-locating questions over ~5 gallery shots ('which row is selected?'), never 'does this look
     fine?'."* **This session landed frost narrowing, a plate rim, an empty-chip removal, per-rail ink, a
     right-click-menu routing change, a caret width change and a per-world mark — and no vision smoke.**
     ⚠️ **The policy's form is the point:** every question must have a **locatable answer the auditor can be
     WRONG about**, because *"does this look fine"* is answerable by a picture of anything — and this repo
     has a recorded case of a sidecar reporting `selected_index: 2` while the row rendered **fully
     invisible**. ✅ **"Cannot locate" is the finding**, not a failure to report.
     ✅ **Also asked, because it is this session's specific risk: is there exactly ONE accent per frame?** A
     round that added a rim, a mark and a rail treatment could easily have produced a second thing competing
     for the eye, and **nobody has looked.**
     ⚠️ **Read-only: the auditor edits nothing.** An audit that finds something **names** the missing law
     without writing it. **Routing:** production tier.
350. ✅ **LANDED (merged 2026-08-08) — AND THE HONEST ANSWER WAS NOT A LAW.** The lane took the
     capture-side oracle over a standing law and **argued it from evidence rather than defaulting**: the four
     candidates a vision smoke could not tell apart have **no registry, enum or file to enrol** — `gallery/`
     is gitignored, no branch or script survives, and the three commits that landed touch one test file and
     **never construct or name A/B/C/D.** The set was built by a throwaway script in a worktree that no longer
     exists. ‼ **So a test over an invented registry would have been "a law satisfiable by deleting its own
     subject" one level removed — grading data nobody declared.**
     ✅ **The oracle takes candidates ALREADY COMPOSITED into one shared frame at one zoom and dpi**, never
     cropped to their own bounding boxes and rescaled independently — **that renormalisation is the exact
     mechanism that erased the vertex-angle signal in the real artifact.** A wrong-sized buffer is **refused
     rather than skipped**, which makes the own-bbox-crop defect impossible to pass by accident.
     ✅ **Its companion floor is the SET'S OWN SIZE**, asserted first, so *"every adjacent pair is distinct"*
     cannot pass vacuously on a singleton. **Both floors are borrowed with their reasons rather than
     invented** — the covered-pixel count mirrors the selected mark's own floor (a thin stroke must not pass
     on population alone) and the peak sits past the JND at the margin this tree's other ΔE floors use.
     🔴 **THE SCALE CLAIM IS PROVEN, NOT ASSERTED, and that is what makes it more than a tidy helper.** A real
     ~26° edge-angle difference clears both floors at **40×40**, still clears them minified to **4×4** and
     barely at **3×3**, and **the SAME floor constants correctly REFUSE the SAME real difference box-averaged
     to 2×2.** **The verdict flips with frame size alone** — so *"measure at the artifact's own scale"* is
     demonstrated rather than sloganeered.
     ✅ Three mutations, each with its panic captured. ⚠️ `code-health` caught **four queue-item citations in
     the lane's own first draft**, fixed in a separate commit rather than argued for.
     **Original:** **A COMPARISON SET NEEDS A PAIRWISE-DISTINCTNESS FLOOR, AND NO CAPTURE PRODUCED FOR A TASTE CALL HAS
     ONE.** Named by item 349's vision smoke after it could not tell three of four candidates apart in the
     artifact built for exactly that choice.
     ⚠️ **The gap is structural, not a one-off:** every floor this repo has for a rendered treatment grades
     it **against its own ground** — presence, contrast, coverage. **Nothing grades one candidate against
     ANOTHER.** So a comparison set can pass every individual presence floor and still be useless, which is
     the "law satisfiable while its purpose fails" shape one level up: the laws were about the product and
     the artifact was about the decision.
     ✅ **Build:** a floor asserting a minimum perceptual separation between adjacent members of a
     comparison set **at the size the set is rendered at** — the size is load-bearing, since these
     candidates differ mostly in vertex angle, which a small crop destroys. ⚠️ **Derive the set from
     wherever candidates are declared rather than a name list**, and **pair it with the obvious companion:
     a set of one member is trivially distinct**, so assert the set's own size too.
     ⚠️ **Scope it honestly** — this only pays for artifacts that exist to settle a decision, not for every
     capture. If the honest answer is that it belongs in a capture helper rather than a law, say so.
     **Routing:** production tier.
351. ✅ **RAN (2026-08-08) — AND IT FOUND A FALSE CHECKABLE CLAIM THIS SESSION ITSELF LANDED, plus the
     defect item 302 exists to delete still live one function away.** Read-only; ran four commands and a
     standalone f32 probe, and **said plainly what it did not audit.**
     🔴 **(A) `m.caret_h / CARET_H` IS NOT `Metrics::scale`, and items 322 and 341 both assert in landed
     comments that it is.** Measured in real Rust f32 over awl's own zoom grid × dpi: **8 of 156 pairs
     mismatch by 1 ULP, every one at dpi 1.5 or 3.0.** ‼ **`(28·s)/28` is not an f32 round-trip — and dpi 1
     and 2 are EXACT for all 26 grid zooms, which is exactly why 322's 22 shots and 341's 48 files were
     silent.** dpi 1.5 is reachable through fractional Wayland scaling. ⚠️ **Magnitude ≤1.4e-6 px: NOT a
     rendering defect, and the audit refused to present it as one.** It is a **false checkable claim** —
     item 302's own subject — in two production docs, three production sites and ~5 test helpers. **Item 352.**
     🔴 **(B) The string-vs-type pair 302 merged is still live on the same two kinds:** `capture/modes.rs`
     gates `caret_preview` on **`o.mode == "caret"`** while `app/viewstate.rs` gates the same field on
     **`o.kind == OverlayKind::Caret`**. **Item 353.**
     🔴 **(C) THE LANDED LAW IS WEAKER THAN ITS OWN DOC.** The doc says the frost exemption is earned by
     *previewing live document state*; the law asserts only **crisp ⊆ `ValuePick`**. The real live-preview
     owner ends in **`_ => {}`**, so **a new live-previewing kind can inherit frost, blur its own preview, and
     leave every law green.** **Item 353.**
     ✅ **(302's merge itself is SOUND, proven symbolically rather than sampled:** the string decider and the
     type decider are identical for **every** string including unknown ones, since lookup is by `as_str` over
     `ALL` and injectivity is already lawed. **The note was correctly hedged.**)
     ✅ **(E) 310 has a seventh EOTF copy — in WGSL — and that is HONEST**, because the law's own doc
     discloses `.wgsl` is outside its reach; constants verified matching and the f64/f32 pair is bounded four
     orders of magnitude below visibility. **310's note is accurate including its self-correction.**
     🔴 **(G) A PRESERVED VACUITY NO BOARD NOTE CARRIES, INCLUDING MINE.** The shared settings fixture parks
     `overlay_window_rows: 12` where the product sets **31**, and the in-tree note records that correcting it
     turns `settings_row_reach_law` **RED**. So a family of settings-card laws is green **only in a
     configuration the product never runs in** — `CLAUDE.md`'s *"the configuration is itself an untested
     hypothesis"*, live. Inherited from 340 and correctly preserved by 348, **but item 348's note says "35
     tests green before and after", which reads as reassurance about laws that are partly vacuous.**
     ⚠️ **Corrected on 348's note.** It stays parked because un-parking it forces item 327's open question.
     ✅ **It named four laws and wrote none**, as a read-only audit should. ⚠️ **And it listed what it did NOT
     audit: item 343 entirely, 174's slice 2, most of 322's ~33 constants, and it reproduced NO byte-identity
     claim** — every identity figure on this board is still only as good as its lane's report.
     **Original claim: THE OUTCOME AUDIT THE STANDING POLICY OWES AFTER AN
     IDENTITY-GATED REFACTOR, AND THIS SESSION NEVER RAN — the second such omission found after saying the
     queue was empty.** `CLAUDE.md` names the trigger and the reason in one line: *"an identity-gated
     refactor (follow with an outcome audit — **byte-identity preserves pre-existing bugs**)."*
     ⚠️ **At least SEVEN refactors this session argued their correctness from byte-identity** — items 302,
     310, 322, 341, 343, 348 and 174's three slices — **and nobody asked the follow-up.** Byte-identity
     proves *"I changed nothing"*; it says nothing about whether what was preserved was right.
     ✅ **The question is not "did the refactor work".** An extraction takes N spellings and makes one
     authoritative, so **if the spellings disagreed, byte-identity on the sampled captures only proves the
     sample missed the disagreement.** The two highest-risk cases are named for the auditor: **302 merged a
     decider keyed on a mode's own STRING with one keyed on a type** — those cannot agree on an unknown
     string — and **174's slice 3 merged a forward step with its own inverse**, which agree only if the
     rounding is symmetric.
     ⚠️ **Also asked: did the chosen owner inherit a bug from the spelling it was taken from?** Item 310's
     own note records that a naive f64→f32 cast mismatched **214 of 256 bytes by up to 6 ULP**, so "each
     caller keeps its own width" is a choice that can preserve the wrong width. **And: was the identity
     sample capable of seeing a difference at all** — a claim "byte-identical at dpi 1" proves little for a
     quantity that only differs at dpi ≠ 1, which is a recorded failure here.
     ✅ **The brief requires ranking by disagreement risk and auditing the top two or three PROPERLY, saying
     plainly which were not audited** — a shallow pass over seven is worth less than a deep pass over two,
     and a clean bill of health from an audit that looked at nothing is the worst outcome. **Read-only.**
     **Routing:** deep tier.
352. ✅ **LANDED (merged 2026-08-08) — the three sites read the stored `scale`, and MY OWN BRIEF WAS WRONG
     THREE TIMES, each corrected by measurement.**
     ⚠️ **NINE mismatching pairs, not eight** — dpi **1.75 at zoom 2.1** was missing, so my claim that *"every
     one of them is at dpi 1.5 or 3.0"* is **false**; 1.75 is exactly as reachable on fractional scaling, and
     the law now sweeps 1.25 and 1.75 for that reason. ⚠️ **Worst magnitude 1.9e-6 px, not 1.4e-6** (f32
     rounding of the ×3 on top of the scale delta). **Still a five-hundred-thousandth of a device pixel, and
     the lane declined to call it a rendering defect** just as the audit did.
     ‼ **AND ONE PREMISE CHANGED THE DELIVERABLE: the numeric law I asked for CANNOT EXIST.**
     `caret_h / CARET_H == m.scale` is a property of `Metrics`' own f32 arithmetic and **does not observe the
     call sites at all**, so routing them through the field cannot change its value — written literally it
     would be **red forever**. The lane split it correctly: **the fail-then-pass law is a SOURCE SCAN**, which
     observes the real subject (which spelling production uses), and the numeric law lands beside it as a
     **permanent measurement**. ✅ **Both halves of that measurement are asserted, because either alone goes
     vacuous** — *"some mismatch"* survives the exact pairs becoming inexact, and *"1 and 2 are exact"*
     survives the division becoming a perfect round trip and the scan becoming ceremony.
     🔴 **ITS SECOND MUTATION IS THE ONE TO REUSE: shrinking the sweep to dpi 1 and 2 — the two factors every
     capture uses, and this item's entire blind spot — makes the numeric law REFUSE TO RUN**, reporting that
     the scan beside it would be guarding nothing. **That is the "has this check ever run anywhere but here"
     arm built into the law itself.**
     ⚠️ **And its FIRST mutation caught its own tooling before it caught the law:** an 8-space needle was a
     substring of the 16-space line, so the edit never applied and the run printed **`ok. 3 passed` —
     indistinguishable from a law that survived.** Only the asserted match count revealed it.
     ✅ **None of the three sites is inside `caret_block_w`**, so the held caret branch is untouched.
     ✅ **Byte-identity across 144 files, 36 of 72 cells at dpi 2**, aggregate md5 equal — and **explicitly NOT
     claimed at dpi 1.25/1.5/1.75/3**, the factors where the number legitimately moves by 1 ULP.
     **Original:** 🔴 **THREE PRODUCTION SITES RE-DERIVE A SCALE
     THE `Metrics` OWNER ALREADY STORES, AND TWO LANDED COMMENTS SAY THAT IS EXACT. IT IS NOT.**
     `m.caret_h / CARET_H` mismatches `m.scale` by **1 ULP at 8 of 156 (zoom, dpi) pairs**, every one at
     **dpi 1.5 or 3.0** — measured in real f32 over awl's own quantised zoom grid. ‼ **dpi 1 and 2 are exact,
     which is why the identity evidence that shipped with items 322 and 341 could not see it.**
     ⚠️ **Magnitude ≤1.4e-6 px — NOT a rendering defect. Do not present it as one.** It is a false checkable
     claim of exactly the class item 302 deletes, now in two production docs and three call sites.
     ✅ **Build:** route the three sites through `m.scale` — making the claim TRUE rather than deleting it —
     then the law the audit named (`caret_h / CARET_H == m.scale` bit-exactly over the zoom grid × dpi
     {1, 1.5, 2, 3}), which **fails today at 8 cells and passes the moment the sites are fixed.** ‼ **Write
     the law first, watch it fail, then fix: that order IS the proof.** Plus a source scan in item 310's shape
     so a fourth site cannot reappear. **Routing:** deep tier.

353. ✅ **LANDED (merged 2026-08-08) — the crisp exemption is now an equivalence THE PRODUCT PROVES, not
     two hand-written lists agreeing.**
     ✅ **`previews_live_document` is a wildcard-free per-kind match, and the preview owner OPENS by asking
     it** — so an arm added there alone is **INERT until the kind declares itself.** ‼ **The gate, not the
     arms, is the enrolment, and that is what closes the `_ => {}` hole a subset law could not see.** A fourth
     spelling of the same membership is gone.
     🔴 **THE SECOND LAW IS THE PRIZE: it calls the REAL preview function once per kind, steps a card's
     highlight, and asserts the set of kinds that ACTUALLY MOVED the running editor equals the declared
     set.** Both directions are named in its message — a kind claiming an audition it never performs **takes
     a crisp backdrop for nothing**, and a kind auditioning from behind a frosted card **blurs its own
     subject.** ✅ **Plus a distinction floor so the predicate cannot collapse into the accept disposition**,
     which is exactly how three row-previewing pickers would silently become crisp.
     ✅ **Those three kinds were VERIFIED rather than assumed** to sit on the non-previewing side — their own
     build comments say *"nothing previews on move"* and *"the example dates ARE the preview"* — and the
     byte-identity proof confirms **no frost moved.**
     ✅ **The string-versus-type pair that survived 302 is routed through the type at both capture sites, and
     nothing needed widening**: five sibling sites in the same function already used that idiom.
     ✅ **The stale prose enumeration is CONVERTED, not reworded** — the membership list is deleted outright,
     because after the equivalence law the predicate names itself and there is nothing left to rot.
     ✅ **And the ungraded arm is graded:** the panel caret's y against **three owners that never read the
     placer** (the published band's centre, glyphon's own line top, and the pointer's inverse), swept over
     both field arms × both row counts × both dpi — **with the dpi axis PROVED rather than assumed** by
     requiring the pitch to actually differ. The two pre-existing panel laws stayed green under its mutation,
     which confirms nothing else graded it.
     🔴 **A METHODOLOGICAL FINDING WORTH REUSING: its byte-identity run first reported DIFFERS, and it was
     the FIXTURE rather than the product** — the go-to picker lists its own folder, and the captures were
     being written into it, so the listing grew between arms. **A before/after capture whose output directory
     sits inside the captured corpus measures the harness.**
     ⚠️ **`src/capture/modes.rs` now sits at 583 = its frozen baseline EXACTLY, so it has zero headroom** —
     the next change there must carve. **Original:** 🔴 **ITEM 302's DEFECT IS STILL LIVE ONE
     FUNCTION AWAY, AND ITS LAW IS WEAKER THAN ITS OWN DOC.**
     ⚠️ **(B)** `capture/modes.rs` gates `caret_preview` on the mode **STRING** while `app/viewstate.rs` gates
     the same field on the **TYPE** — the identical cross-door pair 302's commit message calls *"where two
     doors drift"*, on the same two kinds.
     🔴 **(C) THE MAIN DELIVERABLE.** `keeps_backdrop_crisp`'s doc says the exemption is earned by *previewing
     live document state*; the law asserts only **crisp ⊆ `ValuePick`**, and the real live-preview owner ends
     in **`_ => {}`**. **So a new live-previewing kind can inherit frost, blur its own preview, and leave every
     law green.** Extract a wildcard-free `previews_live_document()` that both the preview owner and its
     duplicate `matches!` route through, and assert the **IFF** the doc claims.
     ⚠️ **Three `ValuePick` kinds preview INSIDE their own rows** (*"the example dates ARE the preview"*) and
     therefore **want** frost — the new predicate must put them on the non-previewing side or three pickers'
     frost changes. **(D)** a stale prose enumeration of the crisp set remains in `pipeline_prepare.rs`.
     **(F)** `PanelRowBands::center()` is ungraded — **the panel caret's Y is asserted nowhere.**
     **Routing:** deep tier.
354. 🚧 CLAIMED (worktree item-354-oracle-scale, production tier) **FIVE CARET TEST ORACLES STILL RECOVER THE DISPLAY SCALE BY DIVISION, one ULP from the factor the
     product now uses.** Reported by item 352's lane, which fixed the three PRODUCTION sites and left these
     because `src/render/tests/**` was a sibling lane's — the right call, and it recommended folding them into
     whichever lane holds that directory next.
     ⚠️ **The reason this is not stylistic:** these are **oracles for the production geometry**, and since 352
     they compute their expected value from a factor **one ULP away from the one the product used.** That is an
     **oracle/subject divergence**, invisible today only because every one of them runs at dpi 1 or 2 — **the
     same blindness item 352 was filed about.** If a caret oracle is ever swept at dpi 1.5 (and it should be),
     a bit-exact comparison fails for a reason that is not the product's.
     ✅ **Build:** five one-line edits to read `metrics.scale` — `caret_ink_box.rs:59,300,316`,
     `caret_transition_item105.rs:87`, `caret_visual_body.rs:56`. ⚠️ **352's source-scan law deliberately
     EXCLUDES test paths, and its doc says in as many words that the exclusion is a scope boundary rather than
     an endorsement** — so consider whether the scan should widen to tests once these are fixed, which would
     stop a sixth appearing. **Routing:** production tier.
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
