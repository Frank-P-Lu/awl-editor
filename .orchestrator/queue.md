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

✅ **CLOSED HERE 2026-08-06 — item 118's direction call. THE TARGET SHAPE IS
DROPPED.** The user's words: *"i think we just drop the target. it's fine, right
now."* The roster's measured mean of **2.20** is accepted as awl's shape rather
than a shortfall against the aspirational 2.90, consistent with PHILOSOPHY's
calm bias. **`1, 7, 6, 4, 2` / mean 2.90 is retired**; it is not amended, not
restated, and not replaced by a descriptive one. Recorded in item 118's body as
well as here — **this item has already been answered twice by the user because a
decision recorded in one place was invisible in the place it gets read.**

## 🔵 OWED — live work that nothing above implies. Never cleared by a compression.

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

✅ **GITHUB ACTIONS RECOVERED 2026-08-07 ("All Systems Operational") AND BOTH OWED
ITEMS ARE DISPATCHED.** CI is running on `main`, and the **AppImage release dry run —
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

118. **Pre-release world-loudness audit.** **Audit definition:** "idle loudness"
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
     🔵 **(2) IS ITS OWN ITEM, not a residual.** `Command` gaining a description
     means authoring 93 accurate one-liners under the docs-voice rule ("facts traced
     to verified sources") — larger than (3) and (5) combined.
     🔵 **(1) needs `main/args.rs` restructured** — 61 flags hand-parsed in one
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

293. 🟡 IN PROGRESS — claude, branch `claude/item-293-footer-separator`.
     **The overlay footer crowds the last row.** The hint line sits hard against
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

297. **Cassowary's rotated location label is too small and misplaced.** Today
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

301. **PART-LANDED (merged 2026-08-06) — the DESTINATION OWNER, the REVEAL and a LAW
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

307. 🟡 IN PROGRESS — claude, branch `claude/item-307-gutter-dpi`.
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

312. 🟡 IN PROGRESS — claude, branch `claude/item-312-feathered-frost`.
     **THE FOOTPRINT FROST'S EDGE IS A HARD RECTANGLE, AND UNDER A DIAGONAL LIST IT
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

313. **THE PICKER'S HINT LINE SITS FLUSH-LEFT UNDER A LEANING LIST.** "type to filter
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
