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

11. **THE FOOTER-RECLAIM ROW BUDGET — found during the CI RED fix (`02d0ea23`), implemented,
   and deliberately backed out because it is a TASTE CALL.** `avail_px` charges the hint row
   and the blank separator a full `lh` each and never credits `overlay_footer_reclaim`, which
   draws them compact — **65px unspent at zoom 3, a whole row.** Crediting it changes shipped
   row counts on cards that already fit, and interacts with the hint-row two-row contract and
   the byte-identity guard. **The question is how many rows a card should show; the arithmetic
   is ready either way.**

## 🔵 OWED — live work that nothing above implies. Never cleared by a compression.
- 🔵 **FOUR THINGS ARE NOW ON `main` AWAITING YOUR EYE, all revertible in one commit or one line.**
  ✅ **The caret** (item 345) — no longer overhangs its glyph on **Currawong** and **Cassowary**; it is
  *narrower* there now, and the caret is this design's one accent. `scripts/dev-app.sh`, `Cmd-T` → Currawong,
  cursor mid-word; compare against Tawny, which is unchanged.
  ✅ **Magpie's mark** (item 346, candidate B) — vertex closed ~70.7° → ~50.8°, weight unchanged. `Cmd-T` →
  Magpie, then `Cmd-P` and arrow down a row. Then `Cmd-T` → Mangrove for the deliberate contrast.
  ✅ **The writing column at 2×** (item 338) — sixteen decorations were half their tuned size on every Retina
  display and now are not. **The squiggle is what reads instantly:** a tight thin ripple becomes a proper
  wave. `gallery/item-338/338-2x-before-after.png`. ✅ **The inconsistency this fix created is CLOSED (item 355)
  and was one device pixel:** the gap was exactly half its owed 2.0 at 2×, so the whole squiggle band sat one
  device px high (rows 157–178 → 158–179 on Tawny, height unchanged). Real and arithmetically exact, but
  **not worth a second look on its own** — it is the consistency the amplitude needed, not a visible change.
  ✅ **The card's width cap** (item 342) — 520 → 545, clearing the clipped help line on Potoroo and Firetail.
  The lane's read: it does **not** look over-wide; the extra 25px land as air after the hint and a looser
  label-to-chord gutter (~1.40:1 → ~1.47:1 against the card's height). **The gutter is what a critical eye
  will notice.** `gallery/item-338/342-shipped-look-*-zoom080-before-after.png`.
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

## Remaining work — handoff order (RE-DERIVED 2026-08-09, against the tree)

⚠️ **This section has gone stale four times, each time by editing the previous
list instead of re-checking the tree.** Every entry in the previous list was
verified landed via `git log --grep` before this re-derivation (292/293/299/303,
294/298, 305, 291, 296+300, 273's residuals, 302, 227, 131e+303 — all merged).

1. **368** — the three small merges. Quick, zero-risk, do first or fold into any lane's round.
2. **365** — the rename sweep BEFORE 372/373/374: it moves files that 372's citation stock,
   373's shard-balance hints, and 374's module paths all name. 366 folds into it.
3. **361 then 364** — ONE lane, sequenced, never a pair: both rewrite `pipeline_draw.rs::new`.
4. **362 and 363** — independent render refactors; 363 is identity-gated, so an outcome audit follows it.
5. **373 then 375** — shard the gate, then raise the lane ceiling and install the gate arbiter.
   **374** any time after 365; it directly raises 373's ceiling (both slow atoms sit in one shard).
6. **372** — the citation stock, after 365. Production tier; 1,700 judgement calls, not a sed script.
7. **357, 358, 369, 370, 359, 360, 371's lane-half** — independent, no ordering constraint among them.
8. **174** — multi-round refactor, continues by slices.
9. **231** — no live lead; its named next step is a macOS guest VM, a spend decision, not work to absorb.
10. **🔵 HUMAN / LIVE, none of which a lane can close** — see BLOCKED and OWED above. **251** is
   hardware-gated (a human at a Linux desktop with Orca). **327** and the landed taste calls
   (338/342/345/346, carried in OWED) close on the user's eye.

---

## Open items
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

359. 🔵 **OPEN — THE MIRROR IMAGE OF 355's AXIS: TWO CARD DIALS ARE dpi-CORRECT AND ZOOM-BLIND.**
     `CardShape::Chamfered { cut_px }` and `CardTexture::HalftoneDots { cell_px }` resolve as
     `* dpi.max(1.0)` — so they track a dense panel and **ignore the reader's zoom entirely**, where 355's family
     did the opposite. Found and deliberately **not** changed by 355's lane, recorded with their owner.
     ⚠️ **Why it is its own item and not a widening:** unlike every length 355 touched, **this one MOVES CARD
     GEOMETRY on several worlds at any zoom ≠ 1** — a chamfer cut and a halftone cell are visible form, not a
     sub-pixel gap. It is a taste call with a unit argument behind it, not a unit repair.
     ✅ **Build:** measure both at zoom 0.8/1.0/2.0 × dpi 1/2 per carrier world, capture before/after on the
     worlds that carry them, and bring the shipped-vs-repaired pair to the user rather than landing it — the
     chamfer is authored form on the worlds that chose it.

360. 🔵 **OPEN (small) — `Frost::feather_px` IS A DIAL THE PRODUCT DOES NOT HONOUR.**
     Every consumer reads the bare `lava::FROST_FEATHER_PX`; the field is written by world literals and read by
     nothing. **Either route the consumers through the field or delete it** — a theme capability that does not
     reach the renderer is worse than no capability, because a later round tunes it and measures no change.
     Found by 355's verdict census (it is the field whose `_px` name is a lie in the other direction).
     ✅ **Build:** grep the consumers, decide route-or-delete, and add the arm that makes an unread `RenderCaps`
     field fail the census rather than earn a verdict.
357. **GENERATE THE PUBLIC WORLD GALLERY FROM THE PRODUCT, SO ITS PICTURES AND ROSTER CANNOT DRIFT.**
     The four known public documentation contradictions were already fixed by item 344; do not repeat that
     census. This item is the remaining public-story work named in `ROADMAP.md`: render **every member of
     `theme::THEMES` over one canonical authored document**, through awl's real headless capture door, and
     publish the resulting gallery on the site's themes page rather than the front page. One script owns the
     regeneration command and deterministic ordering; the roster is derived from `theme::THEMES`, never
     copied into shell, HTML or prose. The canonical document exercises prose, headings, emphasis, a link,
     code, a list, a table and an inline image so each world's typography and authored treatments are visible
     over the same content. Generated assets carry no personal-machine paths and require zero network.
     **Done:** a clean regeneration is repeatable; adding, removing, renaming or reordering a world makes a
     stale-gallery law fail by that world's name; the themes page contains exactly the generated roster and
     locally resolvable images; a sample of generated entries is checked against the theme roster and capture
     sidecars rather than trusting the generator's own HTML; a five-shot affordance-locating vision smoke
     confirms the selected samples visibly show their named world and the document features needed to judge
     them. Run the relevant documentation/site checks and wasm smoke. **Do not deploy or publish** — that is
     an outward-facing action requiring separate authorization. **Routing:** production tier for the
     generator/laws; vision smoke for the rendered sample.

358. **BUILD A PERSISTENCE FAULT MATRIX OVER THE FILE LIFECYCLE, USING FAKES FOR PRECISE FAILURES AND
     REAL PROCESSES ONLY WHERE A FAKE CANNOT PROVE THE CLAIM.** The premise is partly satisfied already:
     `tests/fault_kill9.rs` kills the real binary inside `write_atomic`; `external_item204` covers external
     edits, both resolutions and relaunch recovery; live-App tests cover unwritable saves, autosave,
     rename/duplicate, corrupt scratch data and two-instance conflicts. **Begin with a census and do not
     rebuild those journeys.** The missing matrix is failure by PHASE and by OWNER: temporary-file write,
     final rename, parent folder renamed/removed, permission revoked or disk full while dirty, interrupted
     export preserving its prior target, real-editor kill during autosave followed by relaunch, and a large
     document save/relaunch with explicit time and memory bounds.

     **Tier 1 — deterministic fault injection:** extend the filesystem seam with a scripted failing backend
     that names the operation and ordinal where it fails. Sweep every durable owner (document manual save,
     autosave, scratch, recovery record, history, config/session, export) across write failure and rename
     failure. For user-owned files, assert the previous complete bytes remain, the edited buffer remains
     recoverable and dirty, and the UI reports a calm durable failure rather than silently acknowledging the
     version. For best-effort app metadata, assert the failure cannot block editing or corrupt a sibling
     store. Prove enrolment from the production owner roster rather than a hand-picked test list.

     **Tier 2 — bounded real-process journeys:** add only the cases the fake cannot witness: kill the real
     editor during autosave then relaunch and recover the newest acknowledged complete version; interrupt a
     replacement export and require either the old complete export or the new complete export, never a torn
     file; exercise a large manuscript through edit, save, kill/relaunch and reopen while reporting the
     fixture size, elapsed time and peak memory. Synchronize on observed write phases or write counts, never
     wall-clock luck. Every child runs under an isolated HOME/XDG/config tree and every scratch directory is
     parent-owned and cleaned on unwind.

     **Done:** the matrix report names every enrolled owner × failure phase and every deliberate exclusion;
     each law is mutation-proven by making the relevant owner falsely acknowledge or overwrite; the existing
     SIGKILL and external-change suites remain the single owners of their current claims; native both
     conventions and wasm gates pass, with POSIX-only process arms explicitly gated rather than pretended
     portable. This is safety work, not permission to redesign notices or recovery policy. **Routing:** deep
     tier, one owner end to end.

361. **PIPELINE TINT HAS TWO OWNERS AND NOTHING MAKES THEM AGREE.** Every baked GPU
     pipeline's colour is written twice: once as a constructor argument in
     `TextPipeline::new` (`src/render/pipeline_draw.rs:6` — a 593-line file that is almost
     entirely that one function) and again in `sync_theme_colors`
     (`src/render/pipeline_geometry.rs:27`). Measured across both files the same token
     expressions appear on both sides — `theme::surface_selected()` 6/6,
     `theme::primary()` 6/6, `float_shadow_srgba()` 5/5, `theme::base_200()` 6/5,
     `wash_rgba_bytes(SynKind::Comment)` once each — with no owner and no law over the
     construction half. `render/tests/selection_token_routing_law.rs` already proves the
     SYNC half for two tokens and its module doc names why a capture cannot help: **a
     headless capture builds its pipelines ONCE and never calls `sync_theme_colors`**, so a
     divergence between the halves repaints nothing any capture can see and reaches only a
     user who switches worlds while the app is running.
     ✅ **Build:** construct with placeholder tints and call `sync_theme_colors()` at the end
     of `new()`, so sync becomes the one owner and every future pipeline is born correct.
     ⚠️ **The two sides are NOT the same set, and the count asymmetry is LEGITIMATE** —
     `footer_plate_rim` and `overlay_spine_selected` are re-tinted per frame by
     `chrome/overlay_selection.rs` / `layers.rs`, and `render.rs:2250` says so in as many
     words. **Derive the enrolment from the fields `sync_theme_colors` actually writes**,
     never from "every pipeline". Note `new()` also sets dither/gradient state sync does not
     own (`pipeline_draw.rs:35,56,91,195,298`); sync overwrites only its own fields, and
     `dpi: 1.0` at construction makes `wagtail_stipple_cell_px(self.dpi)` agree with the
     hardcoded `(1.0)`.
     ✅ **Verify:** a law asserting each sync-owned pipeline's post-`new` colour equals its
     post-`sync` colour across the whole roster. `SelectionPipeline::test_color`
     (`src/selection.rs`) is the existing seam; the three other pipeline types
     (`caret/pipeline.rs:232`, `caret_glyph.rs:398`, `spellunderline.rs:221`) need the same
     read accessor. Mutation: skip the sync call, watch it go red by name. Plus a
     byte-identity capture sweep — **any diff is a pre-existing construction/sync
     divergence and is reported, not absorbed.** **Routing:** production tier.

362. **A 16-ARGUMENT POSITIONAL SIGNATURE WITH THREE CALL SITES, ELEVEN OF WHOSE ARGUMENTS
     ARE THE SAME DOCUMENT CONTEXT EVERY TIME.** `build_line_attrs`
     (`src/render/spans/layout.rs:121`, carrying its own `#[allow(clippy::too_many_arguments)]`)
     is the single shared recipe behind `set_text_incremental`, `restyle_all_lines` and
     `refresh_rule_conceal` — called at `src/render/text.rs:778`, `:937` and `:1115`. Eleven
     arguments are doc-level and repeated verbatim at all three sites (`base`,
     `base_font_size`, `base_line_height`, `md`, `md_spans`, `syn_spans`, `doc_lang`,
     `cjk_priority`, `fonts`, `cursor_byte`, `selection_touch`); only five vary per line
     (`line_text`, `line_doc_start`, `conceal_off_cursor`, `image_row_height`,
     `image_force`). ⚠️ **The filed count was 9; the tree says 11** — confirmed by reading
     all three call sites.
     ✅ **Build:** bundle the doc-level arguments into one `LineAttrsCtx` built once per call
     site and keep the per-line arguments explicit. The value is compiler help: a future
     twelfth doc-level input reaching only two of three sites is today a silent behaviour
     split, and the three-path no-drift contract the function's own doc claims is enforced
     by nothing but the author's care.
     ✅ **Verify:** behaviour-identity — a byte-identical capture sweep across the roster
     (output directory OUTSIDE any captured corpus), plus the existing markdown/conceal/image
     suites green. No new law is owed; if one is written, it belongs on the ctx's
     construction, not on the arity. **Routing:** production tier.

363. **THREE RENDER FUNCTIONS DO TWO JOBS EACH, AND EACH SECOND JOB IS A CLEAN LIFT.**
     Measured in the tree: `refresh_rule_conceal` (`src/render/text.rs:985`, 173 lines) ends
     with a self-contained image-force/row-height bookkeeping block at ~`:1049–1111` before
     it reaches its `build_line_attrs` call; `compute_image_layout` (`src/render/text.rs:414`,
     ~184 lines) is a find-spans pass glued to a size/force pass through a local `Found`
     struct at `:444`; `prepare_images` (`src/render/layers.rs:962`, ~211 lines, native arm)
     ends with a placeholder-LABEL tail at ~`:1095–1171` that shares nothing with the decode
     and quad work above it and lifts cleanly as `build_missing_placeholder_areas`.
     ✅ **Build:** three behaviour-identity extractions, each in its own reviewable commit so
     any one can be reverted alone. Do not merge them into one diff.
     ⚠️ **Per `CLAUDE.md`, an identity-gated refactor earns a follow-up OUTCOME audit** —
     byte-identity preserves pre-existing bugs, and the image/conceal neighbourhood is where
     this repo's stale-row and stale-height defects have clustered. Book the audit as part of
     the item, not as a nicety.
     ✅ **Verify:** byte-identical captures over the image/table/conceal fixtures at dpi 1
     and 2 (output outside the captured corpus), the markdown and images suites green, and
     the extracted helpers' names stating what they own rather than where they came from.
     **Routing:** production tier for the extractions; production tier for the outcome audit,
     dispatched separately so it does not read its own diff.

364. **THE 267-FIELD `TextPipeline` IS A DECLARED EXCEPTION; ITS CONSTRUCTOR'S DEFAULT TAIL
     IS NOT.** `TextPipeline::new`'s struct literal runs `src/render/pipeline_draw.rs:311–586`
     — 276 lines, of which **123 are trivial one-value defaults** (`None`, `false`,
     `Vec::new()`, `0.0`, `String::new()`). 69 of those lines carry an `overlay_` / `hud_` /
     `wk_` / `debug_` prefix. Grouping them into `#[derive(Default)]` sub-structs shrinks the
     constructor with no behaviour risk. ⚠️ **This shrinks the CONSTRUCTOR; it does not
     un-declare the struct's GPU-floor exception** (`scripts/code-health.toml`'s
     `src/render.rs` stanza), and the item must not be read as licence to touch that.
     ‼ **THE FILED "NEAR-ZERO RISK" IS TRUE OF BEHAVIOUR AND FALSE OF BLAST RADIUS, AND THE
     DIFFERENCE IS MEASURED.** A sub-struct renames every read site. `hud_` is 9 fields /
     ~77 mentions, `wk_` 5 / ~24, `debug_` 10 / ~39 — all tractable. **`overlay_` is 45
     fields and ~2257 mentions crate-wide**, which is a different item with a different cost.
     ✅ **Build:** do `hud_`, `wk_` and `debug_` as three separate commits and STOP. Report
     the constructor's new line count and an honest estimate for `overlay_`; whether that one
     is worth doing at all is a judgement the orchestrator makes on the reported number, not
     a foregone conclusion.
     ✅ **Verify:** compiles with no `..Default::default()` hiding a field the author meant to
     set (the sub-struct's `Default` is the ONE place a new field gets its inert value, the
     same discipline `ViewState::base()` carries), byte-identical captures, native both
     conventions. **Routing:** production tier.

365. **RETIRE THE INDEX-NAMED TEST FILE — 66 FILES WHOSE NAMES POINT AT A BOARD THAT NO LONGER
     CARRIES THEM. USER DECISION 2026-08-09: the exemption goes.**
     `git ls-files` finds **66** files matching `*item<N>*.rs` — **35,110 lines, 340 tests** — 57
     under `src/render/tests/`, the newest added 2026-08-08. `scripts/code-health.py` currently
     **protects** them: `TEST_FILENAME_ITEM_INDEX` (`:43`) + `is_index_named_test_file` (`:79`)
     exempt such a file's own citations from the comment-citation ratchet, and
     `check_index_named_test_files` (`:135`) validates the number. That carve-out was a real
     position, and it is now overruled.
     ✅ **THE ARGUMENT, AT THE MECHANISM LEVEL — four reasons, and the first is the rule's own:**
     • **The no-citation rule's justification is *"name the mechanism, so the comment stays true
     after the item is closed and the board is compressed."* A FILENAME is that argument's
     strongest case, not its exception** — it is read far more often than any comment it guards,
     and this very board pass compressed 59 closed bodies out of `queue.md`, so most of those 66
     numbers now resolve to nothing a reader can open.
     • **The pointer is redundant.** `git log -- <file>` and `git log --follow` reach the same
     archaeology, because the commits themselves cite the item. The filename buys nothing the
     history does not already carry.
     • **Naming by ticket caused real structural drift, measured in this tree:** `backgrounds_item86.rs`
     and `backgrounds_item89.rs` are one mechanism split into two files purely by landing order
     (item 366) — a mechanism name would have collided and forced the merge at write time.
     • **The defending check concedes its own limit.** `check_index_named_test_files`'s docstring
     says it *"cannot tell a wrong-but-real number from the right one (94 vs. 254 are both real
     items)"*. It catches a fabricated digit string and nothing else.
     ✅ **THE PLAN — three parts, three commits, in this order.**
     **(a) The rename sweep.** All 66 files get mechanism names. ‼ **The rename commit message
     carries an explicit old→new map**, so `git log --grep "item N"` still lands a reader on the
     file. Use `git mv` so `--follow` survives. Also updated: **58** `mod` lines in
     `src/render/tests/mod.rs` (plus the `mod` lists in `src/actions/tests`, `src/app/tests`,
     `src/theme/tests`), **25** cross-file `use super::<name>item<N>` imports (e.g.
     `backgrounds_item117.rs:4` → `backgrounds_item69`; `paperbark_retina_item201.rs:37–39` → three
     item-named siblings), and **21** production doc-pointers — `chrome/diagonal.rs:98`,
     `chrome/overlay_ink.rs:20–22`, `keymap.rs:29`, `render.rs:26`, `rotated_location.rs:89`,
     `quotecheck.rs:27`, `plan/overlay_rows.rs:247`, `geometry/page.rs:14`,
     `chrome/comparison.rs:28,90`, `theme/ground.rs:198`, `overlay/workspace.rs:105` and the rest.
     **(b) Flip the validator into a ratchet.** `check_index_named_test_files` stops validating
     numbers and starts **forbidding** any newly-added index-named test file, by the same
     newly-added-lines discipline the comment ratchet uses.
     **(c) Delete the exemption.** `is_index_named_test_citation` and `TEST_FILENAME_ITEM_INDEX`
     come out of the comment-citation ratchet, along with their self-test fixtures at
     `code-health.py:1433–1442,1497,1539`.
     ⚠️ **Scope boundary:** this item ends at filenames, module paths and the tooling. The
     grandfathered stock of in-body citations is **item 372**, which depends on this one.
     ✅ **Verify:** every test NAME unchanged (only module paths move) — diff the collected test
     list before and after; `code-health.sh` (not `code-health.py` — it carries the clippy arms
     the `.py` cannot see) green after `git add`; full native suite both conventions. Prove (b) and
     (c) non-vacuous by adding a throwaway `foo_item999.rs` and a throwaway `// item 1` comment
     line and watching each go red by name. **Routing:** repeatable tier for (a), production tier
     for (b) and (c); one owner end to end, since (a) is only safe if (b) lands with it.

366. **TWO FILES, ONE MECHANISM, SPLIT BY LANDING ORDER: THE ZIGZAG GROUND.**
     `src/render/tests/backgrounds_item86.rs` (308 lines) holds real-pixel proofs for
     `Background::Zigzag`; `src/render/tests/backgrounds_item89.rs` (1500 lines) describes
     itself in its own first sentence as *"the correctness repair of item 86's chevron margin
     ground"* and carries the field laws for the same shader. Same subject, same seam
     (`headless_dq`, `mark_field`), two files for no reason but the order they landed in —
     and 86 already imports from 69 while 89 imports from 69 and is imported by
     `backgrounds_item132.rs` and `paperbark_retina_item201.rs`. **Merge, don't align.**
     ✅ **Build:** one mechanism-named zigzag-background file; update the two `mod` lines in
     `src/render/tests/mod.rs` and the four external `use super::backgrounds_item89::…` /
     doc references (`backgrounds_item69.rs:10,786`, `backgrounds_item132.rs:21`,
     `backgrounds_item158.rs:28,1165`, `paperbark_retina_item201.rs:38`). Neither file
     carries a `code-health.toml` size mark today; the merged file will be ~1800 lines and
     test files are exempt from the production ceiling, so no mark is owed — **confirm that
     against the tool rather than against this sentence.**
     **Dependency:** 365 is now a decision, so **fold this into 365's rename sweep** — the two
     files are being renamed anyway and merging them at that moment costs one extra `git mv`.
     It is filed separately only so the merge is argued on its own evidence rather than riding
     in unexamined; it stands regardless of what the files end up called.
     ✅ **Verify:** the same 340-minus-nothing test names present, targeted
     `cargo test render::` green (it is the filter this repo's serial-guard failures hide
     behind), full suite at landing. **Routing:** repeatable tier.

367. **THE SIDECAR IS PARSEABLE JSON AND FOUR TEST FILES SCAN IT AS A STRING.**
     `src/capture/tests/panels.rs` carries 20 `.contains(` assertions against rendered prose
     — `:71` pins the literal `"still · frame — ms · worst —\nkey→px — ms\nredraws —"`, and
     others pin whole `"frame_ms": null, "worst_ms": null, …` runs including their interior
     spacing. `src/capture/tests/schema_chrome.rs` has 24. One debug-panel wording change or
     one serializer spacing change breaks ~20 scattered literals that were never about
     wording. `serde_json` is already a dependency (`Cargo.toml:109`), and
     `src/capture/tests/mod.rs:137`'s `num_after(json, anchor, key)` is the existing helper —
     itself a string scanner, so it is part of the subject rather than the fix.
     ✅ **Build:** one parse-then-assert-typed-fields helper in `src/capture/tests/mod.rs`,
     replacing both the `.contains` literals and `num_after`'s call sites. Keep the assertions
     asserting the same FACTS — a typed read of a field that was never checked is scope creep,
     and a typed read that quietly drops a check is a law going vacuous.
     ✅ **Then the bigger payoff, same treatment:** `src/export/tests.rs` (**57**
     `.contains`, not the filed ~50) and `src/export/pdf/tests.rs` (**58**, not ~47) — the two
     most string-pinned files in the repo. These are a separate commit and may be a separate
     round; export output is not the capture sidecar and needs its own parse seam.
     ✅ **Verify:** each converted assertion is proven non-vacuous by breaking the field it
     reads and watching it go red by name — a string `.contains` that becomes a typed read of
     the wrong field passes for the wrong reason, which is exactly what this item is trying to
     stop. **Routing:** production tier.

368. **THREE SMALL "MERGE, DON'T ALIGN" VIOLATIONS, ONE OWNER, THREE COMMITS.**
     ✅ **(a) `Config::empty()` and `Config::load()` carry byte-identical 34-field struct
     literals** — `src/config/model.rs:39` (`path: PathBuf::new()`) and `:149` (`path`),
     differing in that one line and nothing else. Every new setting must be hand-added to
     both, and the compiler catches a MISSING field, never a misaligned one. `load` builds
     from `Self::empty()` and sets `path`.
     ✅ **(b) `src/main/args.rs:948–1245`** is ~298 lines of embedded `#[cfg(test)]` unit
     tests in a file whose own module already has the convention: `src/main/args/flags/tests.rs`
     exists. Move them to `src/main/args/tests.rs`. The file's `code-health.toml` mark is 1245
     against a frozen baseline of 1615, so this is a mark TIGHTENING to ~947 — cheap, and the
     mark edit is the orchestrator's at merge time, not the lane's.
     ✅ **(c) `fixture_opts()` is copied THREE times, not two** — `src/capture/tests/panels.rs:11`,
     `src/capture/tests/pickers_faceted.rs:13` and `src/render/tests/date_picker_ink.rs:32`,
     each an identical 3-line alias for `CaptureOpts::default()`. ⚠️ **And the alias is not
     even the dominant spelling in its own file:** `panels.rs` calls `CaptureOpts::default()`
     directly 40 times against 9 uses of the helper. **Deleting the helper is the merge**;
     hoisting it into `capture/tests/mod.rs` keeps a wrapper that earns nothing and cannot
     reach the `render/tests` copy anyway.
     ✅ **Verify:** (a) a law that a new `Config` field cannot be added to one constructor and
     not the other — the simplest form is that `load` on an absent path equals `empty()` with
     the path set, mutation-proven by desyncing one field; (b) and (c) are name-only moves,
     proven by the suite. **Routing:** repeatable tier for (b) and (c); production tier for
     (a)'s law.

369. **CLEAN THE THEME DATA MODEL BEFORE THE CUSTOM-WORLD COMPOSER MAKES IT A PUBLIC CONTRACT.** Begin
     with a generated census of every `Theme`/`RenderCaps` capability and its adopting worlds. Zero or one
     adopter is a classification prompt, **not** an automatic deletion rule: coherent reusable treatments
     such as Rules, Diagonal, chamfer, ambient stars and background kinds remain data even when today's
     roster has one wearer; tiny corrective geometry and fields with no real per-world variation belong to
     their shared renderer owner instead. Audit `selection_ui` and delete it only if the derived selection
     answer covers every consumer; audit fold lifts, Firetail's `icon_ground`, Cassowary's `pane_split`, and
     zero-variation frost/motion fields for derivation or promotion into a genuinely reusable treatment.
     `spell_underline_gap` is already resolved by item 355 and is excluded rather than rediscovered.

     Replace Wagtail-shaped compatibility switches with one general **arbitrary two-colour** resolver: the
     authored ground and ink can be any two colours, not only black/white, and inverse selection/block-caret
     compositing swaps those palette roles rather than applying mathematical `1 - dst`. Keep inverse
     selection and inverse block caret independently selectable for ordinary palettes. Colours—including
     every treatment's soft/strong rungs—live in the colour-token section; no colour literal hides beside a
     float or geometry parameter. The resolved renderer has no world-name branches.

     **Done:** a roster law reports every zero-/single-adopter capability by adopter name and fails when a
     new field escapes classification; each removal or promotion has a consumer census and a mutation-proven
     law; the arbitrary two-colour path is proved with a non-black/non-white pair and preserves readable
     syntax, selection and caret states; retained one-adopter treatments are explicitly classified as
     reusable visual vocabulary; existing worlds remain pixel-identical except for separately approved,
     named corrections. Update `THEMES.md` and `docs/render.md` so the composer can expose stable semantic
     controls rather than renderer internals. Native both conventions and wasm gates pass. **Dependency:**
     complete before the custom-world composer. **Routing:** deep tier, one owner end to end.

370. **TRIM MAGPIE'S LEFT PARALLELOGRAM BY RECOMPOSING THE SELECTED MARK, NOT BY LYING ABOUT THE FROST
     FOOTPRINT.** The user's live screenshot shows the selected-row `>` far to the left of Magpie's `/`
     spine, forcing a broad softened parallelogram. Item 343 proved the current footprint is already tight:
     the remaining left extent is live mark ink, not padding. Bring that mark inward (and adjust its relation
     to the row/cluster if needed), then let `footprint_narrow` derive the shorter left face from the surfaces
     actually drawn. Do not add a Magpie-only crop, clip the mark, strand it on sharp document ink, or add a
     new low-level per-world placement field while item 369 is removing that class of exception.

     **Done:** capture the current Magpie overlay at the screenshot's broad/narrow stress shape, audition one
     tighter composition, and record the before/after left footprint in logical pixels; every mark, spine,
     label, shortcut and footer surface remains inside the hard footprint and feather support at 1x and 2x;
     the selected row remains immediately locatable in a five-shot vision smoke; Mangrove's mirrored
     composition is either byte-identical or changed only through an explicitly accepted shared rule. The
     footprint tightness/coverage laws and targeted diagonal/frost suites pass. This is a small reversible
     taste change: revert restores the old mark placement and footprint with no file/config migration.
     **Routing:** production tier plus vision smoke.


371. **RESIDUALS HARVESTED OUT OF THE 2026-08-09 COMPRESSION — five live threads that would
     otherwise have left the board with their parent bodies.**
     This board pass compressed 59 closed item bodies to history (`git log -p .orchestrator/queue.md`
     has every one of them in full). Five carried a `🔵` that was neither answered nor mirrored in
     the BLOCKED / OWED sections, so they are restated here. **Each is independent; a lane may take
     one without the others.** The parent number is given so `git log --grep` reaches the full body.
     • **(from 293) `OVERLAY_HINT_GAP_ROW = 0.45` was tuned against a compact-chin law, never judged
     by eye** — a live look is owed before anyone treats the constant as settled. Its laws also
     disclosed two coverage gaps: a **name-based** `OverlayKind::Spell` exclusion (the enrolment
     shape this repo has been bitten by), and a row-count law proven on three representative kinds
     rather than the roster.
     • **(from 301) The `NSSavePanel` is live-only and unreachable from this host** — no test
     process can observe an AppKit modal (`MainThreadMarker::new()` returns `None` off the main
     thread). Does it open at the right folder with the right name pre-filled, and does Cancel leave
     the document untouched? Same for the drawn **Linux** menu bar actually firing `awl.export_word`.
     **These belong in `docs/harness-reach.md`'s live-only census, which is where a brief author
     looks** — filing them there is the deliverable, not re-proving them.
     • **(from 303) The selected mark's MOTION is a proposal, not a decision.** Let the mark ride the
     selection band's existing ease, gliding from the row it left to the row it reached, so direction
     becomes self-evident from the travel. No new machinery. Scope: the two `Diagonal` worlds only.
     **Feel is live-only**, so this closes on a human, not a law.
     • **(from 319) At zoom 1.0 — not the shipped 0.8 — Mangrove's plain hint line overflows the
     card's right edge by ~7.7 logical px.** Magpie and Paperbark stay clean; direction- and
     zoom-gated. Likely mechanism: the clamp's width budget is advance-based while the hint's symbol
     glyphs have wider cells. `render/tests/foot_band_no_clip_item319.rs:48` names the residual in
     its own module doc, so the law is already positioned to grow the arm.
     • **(from 349) The PAIRWISE-DISTINCTNESS floor the vision-smoke audit named and deliberately
     left unwritten.** Each candidate in a comparison set is graded against the page — its presence
     floor — but nothing asserts a minimum ΔE **between adjacent candidates at the size a human
     judges them**, so a comparison set can pass every individual floor and still be useless as a
     comparison. **This generalises past its parent: any capture produced to settle a taste call is
     subject to it**, and this repo has already shipped one comparison artifact a vision smoke could
     not read.
     **Routing:** per bullet — production tier for the two laws, the doc filing is bounded enough for
     repeatable tier, and the two feel questions are the user's.

372. **RETIRE THE WHOLE QUEUE-CITATION STOCK, NOT JUST THE FILENAMES — 1,700 lines across ~348
     tracked files. USER DECISION 2026-08-09.** `CLAUDE.md` forbids citing queue items, rounds or
     shas in code; **the comment-citation ratchet only ever governed newly-ADDED lines**, so
     everything written before it landed (`08856553`, 2026-08-04) is grandfathered and still present.
     ✅ **MEASURED IN THIS TREE, not estimated** — `\bitem[ _]?\d+\b`, case-insensitive, over
     `git ls-files`, excluding `.orchestrator/` (the board is allowed to cite itself) and excluding
     `scripts/code-health.py` + `code-health.toml`'s own **228** lines of regexes, docstrings,
     self-test fixtures and `reason` prose, which are the checker's machinery rather than citations:
     | surface | lines | files |
     |---|---|---|
     | Rust | **1,250** | 295 |
     | Markdown (docs, contract docs, site) | 248 | 17 |
     | `scripts/` | 95 | 28 |
     | `shaders/*.wgsl` | 64 | 3 |
     | `.github/workflows/*.yml` | 39 | 2 |
     | `.toml` / `.sh` | 4 | 3 |
     | **total** | **1,700** | **~348** |
     Of the Rust lines, **1,121 are comment lines** and 153 are non-comment (78 of those inside
     string literals). **Identifiers are a small, bounded population and were counted separately:**
     64 distinct item-bearing Rust identifiers, **all 64 of them module names of the files item 365
     renames**, plus **21 test function names** (e.g. `smart_newline_no_guess_provenance_law_item_78`,
     `item_106_pointer_replay_seam_…`). Zero const/struct/type names.
     ‼ **ONE POPULATION IS NOT A CITATION AND MUST NOT BE SWEPT: a grep-law whose SUBJECT is the
     number.** `retired_item_76_identifiers_leave_no_trace_in_source` and `retired_item_76_needles`
     assert that a retired identifier family is absent from the tree; the digits are the assertion,
     not a pointer. **Enumerate this class first and exclude it by name in the brief**, because a
     blanket rename silently guts the law.
     ‼ **AND A NUMBER DELETED IS NOT A COMMENT FIXED.** `CLAUDE.md`'s rule is *"comments state what
     the code can't say about itself … name the mechanism"*. A comment reading `// ITEM 105 —` and
     nothing else must **gain a real description or be dropped entirely**; leaving `// —` is the
     failure mode this item exists to prevent, and it is invisible to any line-count check.
     ✅ **Build, phased so each phase is revertible alone:** (1) Rust comment bodies, the largest
     population; (2) the 21 test fn names, minus the grep-law exclusions; (3) shaders, scripts,
     workflows; (4) markdown — ⚠️ **`CLAUDE.md` and the contract docs are prose the user owns; propose
     rewrites rather than applying them.** After each phase, widen the ratchet so that surface can
     never regrow the stock.
     **Dependency:** item 365 first — it owns the filenames, module paths and the tooling flip, and
     doing comments before names would rewrite the same lines twice.
     ✅ **Verify:** the ratchet, widened per phase, is proven non-vacuous by re-adding one citation of
     each shape and watching it go red by name; no test name changes except the 21 deliberate ones,
     diffed against the collected list; `code-health.sh` green after `git add`; full native suite
     both conventions plus wasm, because phase 3 touches shaders. **Routing:** production tier — this
     is 1,700 judgement calls about what each comment was trying to say, not a `sed` script, and the
     repeatable tier will produce 1,700 comments with a hole where the number was.


373. **SHARD THE GATE'S TEST EXECUTION ACROSS PROCESSES — 214s → 52s MEASURED — AND MAKE THE
     PARTITION PROVE ITS OWN COMPLETENESS.** Measured on this 10-core host 2026-08-09: the full
     `cargo test --bin awl` suite runs at **~1× parallelism** because `testlock::serial()`
     (`src/testlock/mod.rs:210`) serializes every test in-process. Baseline **214.4s wall at ~100%
     CPU** (3992 passed / 17 ignored). Running the SAME binary as N concurrent PROCESSES with
     disjoint libtest filters works — each process gets its own lock and its own GPU device.
     Count-balanced 4-way gave only **1.61×** (one shard carried 133s); **duration-balanced 6-way
     gave 52.2s wall, 4.10×, 458% CPU**, with pass/ignored sums matching baseline exactly in every
     configuration and **zero filter-induced failures across baseline, 2-, 4- and 6-way** — the
     historical passes-unfiltered/fails-filtered class did not fire once.
     ✅ **THE NON-NEGOTIABLE, AND IT IS WHAT KEEPS THE RECEIPT HONEST.** `CLAUDE.md` says a filtered
     Cargo invocation is *"targeted tests"* and only the gate's receipt may say *"full native
     suite"*. **A provably-complete partition is not a filtered invocation — but only if the proof
     runs.** So at run time the runner asserts the per-shard `--list` counts **sum exactly** to the
     binary's own full `--list` count (**4009** today = 3992 + 17), and fails loudly on drift. A
     hardcoded prefix list silently rots the first time a module is added or renamed, and a shard
     set that quietly stopped covering a module would still print green.
     ‼ **LIBTEST FILTERS ARE SUBSTRING MATCHES AND THIS CODEBASE HAS REAL COLLISIONS — verified in
     the tree:** `render/tests/mod.rs` declares `mod markdown` (`:116`), `mod theme` (`:199`) and
     `mod chrome_overlay` (`:40`), so a bare `markdown::` / `theme::` / `overlay::` filter also
     matches inside `render::tests::`; and `src/main.rs` declares both `mod run` (`:42`) and
     `mod firstrun` (`:81`), so `run::` matches `firstrun::`. **Filter sets use full trailing-`::`
     prefixes plus explicit `--skip` lists**, which is exactly what the winning composition does.
     ⚠️ **THE BRIEF'S OWN PREMISE NEEDED CORRECTING IN TWO PLACES, BOTH CHECKED AGAINST THE SCRIPT.**
     (a) `native-gate.sh` does not run **two** concurrent passes — it runs **four**: `mac_pid`,
     `linux_pid`, `menubar_on_pid`, `menubar_off_pid`, all launched by `gate_launch` and waited
     together (`scripts/native-gate.sh:589–609`). `gate_conventions=2` is the CPU-budget divisor,
     not the process count — the same ×2 the orchestrator README already had to correct in its own
     build-job arithmetic. **So 6 shards × 2 conventions + 2 menu-bar arms = 14 concurrent test
     processes, each holding its own GPU device**, and that is the real subject of the shard-count
     knob. (b) `mac_command`/`linux_command` are bare `cargo test` (`:467–468`), which also builds
     and runs the **13 integration binaries** under `tests/`. The 4009 count is `--bin awl` ALONE.
     **Sharding must not narrow the gate from `cargo test` to `cargo test --bin awl`** — the
     integration binaries stay, and the completeness assertion covers the sharded binary only.
     ✅ **Build:** derive the partition from `--list` at run time, with the measured composition as
     **balance hints only** — a static list that is merely verified will still need a human every
     time it drifts, while a derived one self-heals and the assertion catches the derivation. Locate
     the test binary via `cargo test --no-run --message-format=json`, never a literal path: the
     experiment's scripts hardcode `target/debug/deps/awl-871e1acdf7c52cbe` and that hash moves on
     rebuild. Add an env knob dialling shard count to 1 for orchestrator multi-worktree waves
     (aggregate GPU memory), defaulting to the wrapper-friendly value.
     ⚠️ **The measured composition is EPHEMERAL and is this experiment's whole yield:**
     `/tmp/awl-final/{R1,R2,R3,R4,C,D}.sh` on this machine. **Capture or regenerate it in the FIRST
     commit.** Shape: R1–R4 split `render::` by measured duration (~34/~37/~52/~27 module prefixes);
     **C** is nine non-render prefixes — `app:: syntax:: actions:: run:: overlay:: buffer::
     markdown:: capture:: theme::` with `--skip render:: --skip firstrun::`; **D** is the remainder,
     expressed as skips of all of the above plus the 20 `run::tests::*` submodules C absorbs.
     ‼ **R4 CARRIES BOTH SLOW ATOMS** (`diagonal_pixel_composition`, `frost_context_item298`), so
     **≥37s of the 52.2s wall is two modules** — item 374 is what raises this item's ceiling, and
     the two should be judged together.
     ✅ **Verify:** the count-sum assertion proven non-vacuous by deleting one prefix from a shard
     and watching the gate refuse by name; pass/ignored sums equal to an unsharded baseline on the
     same commit; `scripts/test-native-gate.sh` (829 lines, the gate's own self-test) green —
     ⚠️ **and its CPU-heartbeat law flakes under exactly the load this item creates**, so a red
     heartbeat is classified as contention and rerun alone before it is attributed to the diff.
     Report the new wall time and the concurrent-process count. **Routing:** deep tier — this edits
     the one script that issues the receipt, and the failure mode is a green gate that tested less
     than it claimed.

374. **TWO TEST ATOMS COST 37 OF THE SHARDED SUITE'S 52 SECONDS, AND NOTHING ABOUT THEM LOOKS
     EXPENSIVE FROM THE OUTSIDE.** The duration survey behind item 373 found
     `render::tests::diagonal_pixel_composition::` at **22.51s for 5 tests** (~4.5s each) and
     `render::tests::frost_context_item298::` at **14.51s for 3 tests + 1 ignored**. Together ~37s —
     **more wall time than roughly 380 of their neighbours combined**, hidden in the long tail of
     104 small `render::tests` submodules where nothing draws attention to them. Both landed in the
     same shard (R4), so after duration balancing **these two set the sharded suite's floor**: fixing
     them raises 373's ceiling with no lock or device-pool redesign.
     ✅ **The question the item answers:** what makes each test cost seconds — repeated per-test
     device/pipeline construction, oversized offscreen targets, or many full frames per test — and
     can that cost come out **without weakening the law**.
     ‼ **AND THAT LAST CLAUSE IS THE WHOLE RISK, BECAUSE THE COST IS THE LAW'S OWN SWEEP.** Read in
     the tree: both are roster × DPI sweeps rendering real frames per cell —
     `diagonal_pixel_composition.rs:63` iterates `theme::THEMES` and each of its four tests then
     loops `for dpi in [1.0, 2.0]` (`:396`, `:490`, `:619`, `:893`); `frost_context_item298.rs`
     sweeps worlds through `sweep()` (`:125`) across four tests. **`CLAUDE.md` demands exactly this
     breadth** — *"the axis it sweeps is the one the author didn't think of… sweep the roster, the
     whole geometry range"* — so **narrowing the enrolment is not an optimisation, it is this
     board's recorded satisfiable-by-deleting-its-subject failure wearing a stopwatch.** Legitimate
     targets are per-cell overhead: frames built once and reused across assertions, offscreen
     targets sized to the region actually read, a device/pipeline hoisted out of the per-cell loop.
     If the honest answer is "the cost is the coverage", **say so and close it as premise-answered**
     — that is a real outcome and it tells 373 its floor.
     ⚠️ **The figures are one host, one configuration** — this board's standing tripwire. **Re-measure
     before and after on the same machine**, and report both numbers plus the method, because a
     timing claim taken from a report rather than from a run is how this repo has been wrong before.
     ⚠️ **Ordering:** item 365 renames `frost_context_item298.rs` to a mechanism name, which
     invalidates this item's module path and 373's balance hints on the same day it lands.
     **Sequence 365 before either, or accept one mechanical fix-up pass across both.**
     ✅ **Verify:** every assertion in both modules still fires — mutation-prove at least one law per
     module after the change, since a faster test that stopped seeing its subject is the exact
     failure this item could cause; enrolment counts (worlds × DPI cells graded) reported before and
     after and **required to be equal**; `cargo test render::` green under the filter, which is the
     invocation this repo's serial-guard defects hide behind. **Routing:** production tier.

375. **RAISE THE LANE CEILING TO SIX–EIGHT, PUT A QUEUE IN FRONT OF THE GATE, AND PARTITION
     LANES BY MECHANISM, NOT FILE.** ⚠️ **Gated on 373 — short gates are the enabler; do not
     start this before the sharded gate lands.** (User decision 2026-08-09.)
     **The four-lane ceiling is a GATE-CONTENTION ceiling, not an editing one.** Each gate runs
     four full-suite passes (two conventions × both menu-bar arms), each pinned to ~one core by
     `testlock::serial()`, so four lanes gating together schedule ~16 sustained test cores on
     ten — the measured load-69.79 wave. With 373 a gate is ~1–2 min and gates stagger instead
     of colliding. Three parts:
     **(a) README ceiling: 6–8 lanes.** `worker-build.sh`'s per-lane budget stays. Disk note:
     ~5 GB of `target/` per worktree, so eight lanes ≈ 40 GB+ — the disk-preflight section
     already governs that axis; cite it rather than restating it.
     **(b) A gate ARBITER.** Extend the `.orchestrator/native-gate.marker` seam from advisory
     to a blocking queue: at most one or two gates (lane or train) run at once, each at full
     shard width; the marker names holder PID, sha, and start time so a blocked gate can say
     who it waits on. This replaces the soft "wait for the wave to quiesce" rule with a
     mechanism, and structurally retires the CPU-heartbeat false-red class the README
     documents — that self-test fails from gate-collision load alone.
     **(c) README §8 partition rule: MECHANISM granularity.** Keep the core — two lanes on one
     mechanism are one lane. Stop serializing on shared HUB files (`keymap.rs`'s `Action` enum,
     `commands/catalog/*`, `mod` lists, `code-health.toml`, `assets/keymap-defaults.toml`)
     whose edits are append-shaped and whose stanzas the README itself records as merging
     cleanly, with a hold's real cost being "a lane blocked behind another lane's grant".
     Evidence for the change: the measured touch-point counts (~5 files per action, ~10 per
     setting) mean every feature crosses hubs, so file partitioning transitively serializes
     nearly everything; and the partition's recorded bill — a lane DUPLICATED a chrome geometry
     because another lane held the file it should have called into — is the "same behavior ⇒
     same code" violation the partition was meant to prevent. The serial one-at-a-time merge
     train, re-gating every landing on the exact combined candidate, stays the collision
     catcher and is unchanged.
     **Verify:** README states the new ceiling, the arbiter, and the mechanism rule with the
     measured numbers above; the arbiter is demonstrated live (start a gate, start a second,
     watch it queue naming the holder); the heartbeat-flake note is repointed at the arbiter.
     **Routing:** production tier — protocol prose plus one small scripts change.

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
