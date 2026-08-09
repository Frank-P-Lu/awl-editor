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
   tree). Its `version.json` still reads `0.0.0 / prerelease` while `v0.9.0` is
   public. `FLY_API_TOKEN` is configured; trigger is `workflow_dispatch` only,
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
5. **Mulga and Magpie grounds want a visual judge** — see OWED.
6. **Kite's facet tag:** its doc calls it a "technical room" but its only facet
   is `voice: Modern`, and Technical belongs to Cassowary. Picker facets are
   curated and capped, so the change is yours.
7. **The narrow History comparison stage draws no footer** (`show_rows` false →
   `hint_rows` 0 at ~900×520 and below) — a deliberate discoverability hole;
   whether to spend vertical space there is taste.
8. **The macOS release arm** — Apple signing secrets, per `RELEASING.md` §1.
9. **Further tags and the site deploy** — your explicit word, every time.
10. **Does the debounce rip-out feel right (item 290, landed `51302d50`)?**
    Every step now reshapes (~33 ms each, two frames) instead of eight wrong-font
    steps and a 141–151 ms stall on the ninth; per-step cost unchanged, the 4×
    total is the cost of doing the work every step. Arrow quickly through the
    faceted theme picker: honest hitch, or worse than before? Recommendation:
    keep. Revert is `git revert 51302d50`; raising the window instead is refuted
    (300 ms → 348 ms settle, 400 → 459).
11. **The footer-reclaim row budget** (found during the CI RED fix `02d0ea23`,
    implemented, deliberately backed out): `avail_px` charges the hint row and
    blank separator a full `lh` each and never credits `overlay_footer_reclaim`,
    which draws them compact — 65 px unspent at zoom 3, a whole row. Crediting
    it changes shipped row counts on cards that already fit. The question is how
    many rows a card should show; the arithmetic is ready either way.

## 🔵 OWED — live work that nothing above implies. Never cleared by a compression.

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
  (CLAUDE.md's own tripwire, live again). Not fixed because the 1× look is what
  was tuned; the law is fix-tolerant if you say scale it. Also: the card has no
  clamp to the window — `x` reaches −33 at 560 px width.
- 🔵 **The right-click menu no longer dims your document** (item 298): carded
  worlds get no frost at all; bare worlds frost the menu's own footprint only.
  Before/after differs on 76–79% of pixels; `gallery/item-298/`. Revert is one
  line. Out of step now: the **spell popup** still takes no frost anywhere
  (DESIGN §5 says it "recedes nothing" on purpose) — whether that stands is
  yours.
- 🔵 **Four public docs state things the code contradicts** (item 344, all four
  verified): GUIDE says 19 worlds, ACCESSIBILITY says 14 — the roster is 20 and
  WEB.md already says so; platform.md documents "Finish Buffer" with a retired
  chord (palette says "Finish file"); GUIDE omits that a selection also reveals
  conceal. 25 further census entries reported but unverified.
- 🔵 **Right-click menu's greyed-out labels** (item 299): "unavailable" used to
  sit one row below its own row (ΔE 0.0 — invisible); now correct. A glance to
  confirm it reads as quiet, not broken. `gallery/item-299/`.
- 🔵 **Settings width budget** (item 327 — the open item below carries the
  question): two supporting facts for the decision. The "Project root" value
  never elides, so the rail's presence depends on the user's checkout-path
  length (`gallery/item-334/pwp-*.png`); and rail presence is **non-monotonic**
  (absent at 640–720 *and* 880–920, present between) — diagnose the hole before
  designing a budget.
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
  Cancel leaves the document untouched? Try `Export as PDF…`. The Linux arm
  (drawn menu bar firing export) is separately unreached.
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
- **211 — the unoccluded live-glide photograph.** The defect is FIXED
  (`237f97d7`); what is owed is only the photo — the display locked mid-sitting
  and every present read Occluded. Instrument: `scripts/`' live band sweep
  (`54c027e1`), built so it cannot look like success while photographing
  nothing.
- **284 — the live glide's feel** and `MARKER_TRAVEL_TILT_DEG = 20°`; plus
  whether a wrap's transient (indistinguishable from an ordinary step) deserves
  a distinct flourish. Live judgement.
- **242 — residuals:** the formal affordance-locating vision smoke (an eyes-on
  pass happened; the structured version is owed), and `readout::CANVAS_INSET`
  is declared `Physical` with its reason — promoting it to `Logical` doubles
  the inset on Retina (almost certainly correct) and owes a 1×/2× sweep across
  every anchor arm plus the notice plate's clamp. Naming inline literals is how
  the remaining ones get found.
- **241 —** live numbers came from a 900×600 probe window; the user's 4530×2756
  @2x window is unmeasured, and the dense pointer/wheel cadence is unproven.
- **249 —** stated cost: nothing in `PendingWrites` pins a *view*, so a leak
  pinning only textures is invisible to the portable unit.
- **245 —** one constant: 200 wpm.
- **263 —** the construction-site document-seed mutation was deferred (gate
  contention), inferred rather than measured. Worth closing.
- **273 —** `site/reference.html` is visually unreviewed (links pass, CSS
  reuses `.credits-body`).
- **271/283 —** `Rules` ships on one carrier; second-carrier requirements live
  in `theme/tests/personality.rs`.

## Remaining work — handoff order (RE-DERIVED 2026-08-09, against the tree)

⚠️ **This section has gone stale four times, each time by editing the previous
list instead of re-checking the tree.** Every entry in the previous list was
verified landed via `git log --grep` before this re-derivation (292/293/299/303,
294/298, 305, 291, 296+300, 273's residuals, 302, 227, 131e+303 — all merged).

1. **362 and 363** — independent render refactors; 363 is identity-gated, so an outcome audit follows it.
2. **373 then 375** — shard the gate, then raise the lane ceiling and install the gate arbiter.
   **374** any time after 365; it directly raises 373's ceiling (both slow atoms sit in one shard).
3. **372** — the citation stock, after 365. Production tier; 1,700 judgement calls, not a sed script.
4. **357, 358, 369, 370, 359, 360, 371's lane-half** — independent, no ordering constraint among them.
5. **174** — multi-round refactor, continues by slices.
6. **231** — no live lead; its named next step is a macOS guest VM, a spend decision, not work to absorb.
7. **🔵 HUMAN / LIVE, none of which a lane can close** — see BLOCKED and OWED above. **251** is
   hardware-gated (a human at a Linux desktop with Orca). **327** and the landed taste calls
   (338/342/345/346, carried in OWED) close on the user's eye.

---

## Open items

174. 🟡 **Separate pure render planning from shaping/caches and GPU execution — multi-round;
     slices 1–3 landed 2026-08-08, the item stays open.** Landed so far: `src/render/plan/`
     (device-free; `plan_overlay_rows` emits `PlannedRow`s; the forward/inverse row arithmetic
     is module-private and the chrome bypasses are deleted); the sidecar publishes row geometry
     (schema /201), the accessory cluster's three lanes (/202, `null` ≠ zero-width — the width
     where the pair turns `null` IS the measurement), and the find/replace panel's geometry
     (/203, with `PanelRowBands` as the one owner of the row-band step and its inverse).
     **Left, stated:** document-content surfaces, search panel, HUD, gutter, outline, popover,
     whichkey and readout still own their geometry; the spell popup's anchor is unplanned;
     `overlay_secondary_top`/`overlay_split_bounds`/`overlay_strip_band` are separate owners —
     folding the strip band + secondary column in is the natural next slice.
     **Constraints that hold for every slice:** O(visible), buffer-identity cache keys,
     rowlayout ownership, deterministic capture, exact output for migrated surfaces; no
     retained widget tree, no general scene framework, no duplicate CPU renderer, no per-frame
     document plan. Lessons the next slice inherits: probe TRANSITIONS not centres (a
     20%-wider-pitch mutation survives centre probes at 32 px rows); grade published extents
     against INK, not origin (a uniform width scaling survives origin-only laws); don't write
     containment laws over staggered rows (Saltpan publishes `x` left of `band_x`); the first
     slice's bench delta read as contention noise, so confirmation is owed on a quiet host.
     **Routing:** deep owner with a production-tier outcome audit.

231. **Name the CAUSE of the hosted-macOS gate hang; the fix is a SECOND item.** Reframed by
     user decision from "fix" to "diagnose". `mac (render::tests)` HANGS (never fails): exactly
     3 tests (the runner's vCPUs) park simultaneously in `poll(wait_indefinitely)`, surviving
     SIGTERM. Bisected to `8207e519` (Kite, `Background::WarpedGrid`), both boundaries measured.
     **Eliminated by measurement — do not re-derive:** the shader itself; a single bad test
     (victim varies); concurrency (`RUST_TEST_THREADS=1` wedges); a per-device resource (mac
     and linux conventions on separate devices stopped within 10 ms — the contended resource
     is the VM's virtualised Metal stack, system-wide); program-build volume (a 9.3× cut did
     not clear it; `--skip render::tests::` completes while building MORE programs — but on
     churned, destroyed devices); RAM (steady 2.37 GB); software adapters (two lavapipe stacks
     never hang — no system-wide GPU resource to exhaust); shader-source size (HEAD carries the
     largest wgsl and gets 2.6× further).
     **Unknown, and the whole item: WHICH resource the virtualised Metal stack exhausts.**
     First deliverable: a LOCAL reproduction — the untried arm is a macOS guest VM on this
     host (tart/UTM, paravirtualised Metal; no VM tooling installed, a real spend). A negative
     is publishable too.
     **Decision gate (the item is not done without it):** is the PRODUCT exposed, or is this
     harness-only? The live app builds `BackgroundPipeline` once; the per-frame program churn
     is test-helper-only — but that rests on the churn hypothesis, which the 9.3× null result
     weakened. **Do not land a fix under this item** — recycling the shared test device would
     turn CI green without anyone learning what was exhausted.
     Carry-forwards: wgpu 29.0.3 `Device::PartialEq` reports two live devices EQUAL (no
     device-keyed cache); a `cfg(test)` cache must not be thread-local (libtest = thread per
     test); one leak law only fires if all twenty worlds build+prepare BEFORE any draws; `gh`
     encodes unfinished steps as `conclusion:""` and ceiling-killed steps as
     `completed/cancelled` — enumerate accepted values, never test inequality; cross-commit
     probes need a target dir per tree plus a provenance assertion (same-second extraction let
     Cargo reuse the other tree's artifacts).
     **Done:** the resource is NAMED with a confirming measurement; the product/harness
     question has an evidenced answer; the fix is scoped as its own item. **Verify:** the
     diagnosis must PREDICT the boundary (why `36707d06` survives, why `--skip render::tests::`
     survives doing more work, why two processes stop within 10 ms). **Routing:** deep tier.
     Rig: `scripts/oom-budget-container.sh` (diagnostic, not a gate).

251. **Item 207's AT-SPI journey needs a LINUX machine** — AT-SPI2 is the Linux accessibility
     API; no unlock of this Mac reaches it, and `ACCESSIBILITY.md:110` says honestly that no
     AT-SPI journey has ever run. **Build:** a real Linux desktop session + Orca + the native
     build, running the VoiceOver sitting's journeys (document read, caret/selection
     announcement, overlay summon/dismiss, editing burst); findings recorded in
     ACCESSIBILITY.md. Defects found earn their own items. **Done:** run and recorded, or
     parked with the hardware requirement stated. **Routing:** human, on Linux. (The AT-SPI
     tree itself was verified correct — AccessKit filters `Role::TextRun` by design.)

283. **`ListStyle::Rules` graduated — two handbacks remain.** (1) A second carrier world:
     taste call on WHICH world (deliberately out of scope here); needs a findability check for
     `faint()` hairlines on a DARK ground (asserted only on cream today) and a `FacetStyle`
     decision that doesn't put a filled pill back on the strip (Paperbark is `Text`; nothing
     forbids `Chips(FilledActive)`). Requirements recorded in `theme/tests/personality.rs`
     beside Paperbark. (2) ⚠️ `Rules` must not reach a second world whose users default to
     Retina before item 289 closes — a half-weight rule is a half-legible affordance.

327. 🔵 **The Settings accessory-column width budget is a design call, with numbers.** As a
     `Range` setting's card narrows, `overlay_right_shown` drops the entire accessory column
     at once — value text AND rail, gated together. **Who yields first: the row name, the
     value, or the rail? And should the picker fall out of its faceted/diagonal composition
     SOONER, before losing the whole column?** Numbers: narrowest reachable failure needs
     480 px against 319 available; the merely-tight diagonal case 412 vs 366. The names elide
     58.75 px before the column drops whole (boundary at 780/781 logical, menu-bar arm moves
     it by one row); only the two diagonal worlds ever yield in the 640–1200 band.
     `gallery/item-309/327-ordinary-{640,1200}.png`. ⚠️ Before designing: (a) the boundary is
     **non-monotonic** — rail absent at 640–720 *and* 880–920, present between; diagnose the
     hole first; (b) the "Project root" value never elides, so the budget depends on the
     user's checkout-path length — eliding it may be the actual fix; (c) item 342's cap-tier
     question is upstream (it changes the width this divides) — decide together.
     **Routing:** production tier, then the user.

357. **Generate the public world gallery from the product, so pictures and roster cannot
     drift.** Render every member of `theme::THEMES` over one canonical authored document
     (prose, headings, emphasis, link, code, list, table, inline image) through the real
     headless capture door; publish on the site's themes page. One script owns regeneration
     and ordering; the roster derives from `theme::THEMES`, never copied into shell/HTML.
     No personal paths, zero network. **Done:** regeneration repeatable; a stale-gallery law
     fails by world name on add/remove/rename/reorder; generated entries spot-checked against
     the roster and sidecars rather than the generator's own HTML; five-shot vision smoke on
     samples. **Do not deploy** — outward-facing, separate authorization. **Routing:**
     production tier + vision smoke.

358. **Persistence fault matrix over the file lifecycle** — fakes for precise failures, real
     processes only where a fake cannot prove the claim. Existing coverage stands
     (`tests/fault_kill9.rs`, `external_item204`, live-App save/autosave/conflict suites) —
     census first, do not rebuild those journeys. The missing matrix is failure by PHASE ×
     OWNER: tmp-write, final rename, parent renamed/removed, permission/disk-full while
     dirty, interrupted export, kill-during-autosave + relaunch, large-doc bounds.
     **Tier 1:** scripted failing filesystem backend naming operation + ordinal; sweep every
     durable owner (manual save, autosave, scratch, recovery, history, config/session,
     export); user files keep previous complete bytes + recoverable dirty buffer + calm
     durable failure; app metadata never blocks editing or corrupts a sibling. Enrolment from
     the production owner roster. **Tier 2:** only what fakes can't witness — kill real
     editor during autosave + relaunch; interrupted replacement export yields old-complete or
     new-complete, never torn; large-manuscript journey with size/time/memory reported.
     Synchronize on observed writes, never wall-clock. Isolated HOME/XDG per child.
     **Done:** matrix report names every owner × phase and every exclusion; each law
     mutation-proven; POSIX-only arms gated, not pretended portable. **Routing:** deep tier.

359. 🔵 **Two card dials are dpi-correct and zoom-blind** (mirror of 355's axis):
     `CardShape::Chamfered{cut_px}` and `CardTexture::HalftoneDots{cell_px}` resolve as
     `* dpi.max(1.0)` — they ignore zoom entirely, and unlike 355's lengths they move visible
     card FORM on carrier worlds at any zoom ≠ 1. A taste call with a unit argument, not a
     unit repair. **Build:** measure both at zoom 0.8/1.0/2.0 × dpi 1/2 per carrier, capture
     before/after, bring the pair to the user rather than landing it.

360. 🔵 **`Frost::feather_px` is a dial the product does not honour** — every consumer reads
     bare `lava::FROST_FEATHER_PX`; the field is written by world literals and read by
     nothing. Route the consumers through the field or delete it, and add the census arm that
     makes an unread `RenderCaps` field fail rather than earn a verdict.

362. **A 16-argument positional signature, three call sites, eleven arguments identical.**
     `build_line_attrs` (`src/render/spans/layout.rs:121`) is the shared recipe behind
     `set_text_incremental`/`restyle_all_lines`/`refresh_rule_conceal`
     (`src/render/text.rs:778,:937,:1115`); 11 doc-level args repeat verbatim (measured — the
     filed count was 9), 5 vary per line. **Build:** one `LineAttrsCtx` built per call site;
     per-line args stay explicit. The value is compiler help: a twelfth doc-level input
     reaching two of three sites is today a silent behaviour split. **Verify:**
     byte-identical capture sweep (output outside any captured corpus) + existing
     markdown/conceal/image suites. **Routing:** production tier.

363. **Three render functions do two jobs each; each second job lifts cleanly.**
     `refresh_rule_conceal` (`text.rs:985`, 173 ln — image-force bookkeeping at ~:1049–1111);
     `compute_image_layout` (`text.rs:414` — find-spans glued to size/force via `Found`);
     `prepare_images` (`layers.rs:962`, native — placeholder-label tail at ~:1095–1171 →
     `build_missing_placeholder_areas`). **Build:** three behaviour-identity extractions,
     three separate commits. ⚠️ Identity-gated refactor ⇒ book the follow-up OUTCOME audit
     (byte-identity preserves pre-existing bugs; stale-row/height defects cluster here).
     **Verify:** byte-identical captures over image/table/conceal fixtures at dpi 1+2;
     helpers named for what they own. **Routing:** production tier; audit dispatched
     separately so it doesn't read its own diff.

367. **The sidecar is parseable JSON and four test files scan it as a string.**
     `capture/tests/panels.rs`: 20 `.contains(` against rendered prose (`:71` pins a literal
     panel-text run); `schema_chrome.rs`: 24. One wording or serializer-spacing change breaks
     ~20 literals that were never about wording. `serde_json` is already a dep; the existing
     helper `num_after` (`capture/tests/mod.rs:137`) is itself a string scanner — part of the
     subject. **Build:** one parse-then-assert-typed helper in `capture/tests/mod.rs`,
     replacing the literals and `num_after` call sites; assert the same FACTS (a typed read
     of an unchecked field is scope creep; a dropped check is a law going vacuous). Then the
     bigger payoff, separate commit/round: `export/tests.rs` (**57** `.contains`) and
     `export/pdf/tests.rs` (**58**) — export needs its own parse seam. **Verify:** each
     converted assertion proven non-vacuous by breaking the field it reads. **Routing:**
     production tier.

369. **Clean the theme data model before the custom-world composer makes it a public
     contract.** Census every `Theme`/`RenderCaps` capability × adopting worlds; zero/one
     adopter is a classification prompt, not an auto-delete (Rules, Diagonal, chamfer,
     ambient stars, background kinds stay data at one wearer; tiny corrective geometry and
     no-variation fields go to the shared renderer owner). Audit `selection_ui` (delete only
     if the derived answer covers every consumer), fold lifts, Firetail `icon_ground`,
     Cassowary `pane_split`, zero-variation frost/motion fields. `spell_underline_gap` is
     resolved (355) — excluded. Replace Wagtail-shaped switches with one general arbitrary
     TWO-COLOUR resolver (palette-role swap, not `1 - dst`; inverse selection and inverse
     block caret independently selectable); all colours in the token section; the resolved
     renderer has no world-name branches. **Done:** a roster law reports every zero/single-
     adopter capability by name and fails on an unclassified new field; each removal/
     promotion has a consumer census + mutation-proven law; the two-colour path proven with
     a non-black/white pair; worlds pixel-identical except separately approved corrections;
     THEMES.md + docs/render.md updated. **Dependency:** before the composer. **Routing:**
     deep tier.

370. **Trim Magpie's left parallelogram by recomposing the selected mark, not by lying about
     the frost footprint.** The user's screenshot shows the selected `>` far left of the `/`
     spine, forcing a broad softened parallelogram; item 343 proved the footprint is already
     tight — the left extent is live mark ink. Bring the mark inward, then let
     `footprint_narrow` derive the shorter left face from surfaces actually drawn. No
     Magpie-only crop, no clipping the mark, no new per-world placement field while 369
     removes that class. **Done:** before/after left footprint in logical px at the
     screenshot's stress shape; everything inside footprint+feather at 1×/2×; selected row
     locatable in a five-shot vision smoke; Mangrove byte-identical or changed only via an
     explicitly accepted shared rule; footprint + diagonal/frost suites green. Reversible
     taste change. **Routing:** production tier + vision smoke.

371. **Residuals harvested from the 2026-08-09 compression** — five live threads whose
     parent bodies went to history (`git log --grep` by parent number reaches the full
     bodies). Independent; take any one alone.
     - (293) `OVERLAY_HINT_GAP_ROW = 0.45` was tuned against a law, never judged by eye; its
       laws also disclosed a name-based `OverlayKind::Spell` exclusion and a three-kinds-not-
       roster row-count law — both enrolment shapes this repo has been bitten by.
     - (301) File the `NSSavePanel` + drawn-Linux-menu export paths in
       `docs/harness-reach.md`'s live-only census — the filing is the deliverable.
     - (303) Proposal, not decision: let the selected mark ride the selection band's ease
       between rows (two `Diagonal` worlds only). Feel is live-only; closes on a human.
     - (319) At zoom 1.0 (not the shipped 0.8) Mangrove's plain hint overflows the card's
       right edge ~7.7 logical px; advance-based budget vs wider symbol cells.
       `foot_band_no_clip_item319.rs:48` names the residual and is positioned to grow the arm.
     - (349) The pairwise-distinctness floor: comparison-set candidates are graded against
       the page but nothing asserts a minimum ΔE BETWEEN adjacent candidates at judging size
       — a comparison set can pass every floor and be useless as a comparison. Generalises
       to any capture produced to settle a taste call.
     **Routing:** production tier for the two laws; repeatable for the doc filing; the two
     feel questions are the user's.

372. **Retire the whole queue-citation stock, not just the filenames — 1,700 lines across
     ~348 tracked files. USER DECISION 2026-08-09.** The comment ratchet only ever governed
     newly-ADDED lines; everything before `08856553` (2026-08-04) is grandfathered. Measured
     (`\bitem[ _]?\d+\b` over `git ls-files`, excluding `.orchestrator/` and code-health's
     own 228 machinery lines): Rust 1,250 lines/295 files (1,121 comments; 78 in string
     literals); Markdown 248/17; scripts 95/28; shaders 64/3; workflows 39/2; toml/sh 4/3.
     Identifiers are bounded: 64 module names (all covered by 365's renames) + 21 test fn
     names; zero types/consts.
     ‼ **Exclusions, enumerated by name in the brief:** grep-laws whose SUBJECT is the number
     (`retired_item_76_identifiers_leave_no_trace_in_source`, `retired_item_76_needles`) — a
     blanket rename guts them. ‼ **A number deleted is not a comment fixed:** `// ITEM 105 —`
     must gain a real mechanism description or be dropped; `// —` is the failure mode, and no
     line-count check sees it.
     **Build, phased and separately revertible:** (1) Rust comment bodies; (2) the 21 test fn
     names minus exclusions; (3) shaders/scripts/workflows; (4) markdown — CLAUDE.md and
     contract docs are the user's prose: propose, don't apply. Widen the ratchet per phase.
     **Dependency:** 365 first. **Verify:** ratchet proven non-vacuous per shape; no test
     name changes beyond the 21 deliberate ones; `code-health.sh` green after `git add`;
     full native + wasm (phase 3 touches shaders). **Routing:** production tier — 1,700
     judgement calls; the repeatable tier will produce 1,700 comments with a hole where the
     number was.

373. **Shard the gate's test execution across processes — 214 s → 52 s measured — and make
     the partition prove its own completeness.** Measured 2026-08-09 on this 10-core host:
     `cargo test --bin awl` runs at ~1× parallelism (`testlock::serial()`,
     `src/testlock/mod.rs:210`) — 214.4 s wall at ~100% CPU (3992 passed/17 ignored). The
     same binary as N concurrent PROCESSES with disjoint filters works (own lock, own GPU
     device per process): count-balanced 4-way only 1.61×; **duration-balanced 6-way 52.2 s,
     4.10×, 458% CPU**, pass/ignored sums exact in every configuration, zero filter-induced
     failures anywhere.
     ✅ **The non-negotiable:** a provably-complete partition is not a "filtered invocation" —
     but only if the proof RUNS. At run time, assert per-shard `--list` counts sum exactly to
     the binary's full `--list` (4009 today); fail loudly on drift.
     ‼ Libtest filters are SUBSTRING matches with real collisions here (`markdown::`/
     `theme::`/`overlay::` match inside `render::tests::`; `run::` matches `firstrun::`) —
     use full trailing-`::` prefixes + explicit `--skip` lists, as the winning composition
     does. ‼ `native-gate.sh` runs FOUR concurrent passes (two conventions × two menu-bar
     arms, `:589–609`), so 6 shards ⇒ **14 concurrent GPU-holding processes** — that is what
     the shard-count knob is sized against. And `mac_command`/`linux_command` are bare
     `cargo test` (`:467–468`) including the 13 integration binaries: **sharding must not
     narrow the gate to `--bin awl`**; the completeness assertion covers the sharded binary
     only.
     **Build:** derive the partition from `--list` at run time with the measured composition
     as balance hints only (a verified static list still needs a human per drift); locate the
     binary via `cargo test --no-run --message-format=json`, never a literal hash path;
     env knob dialling shards to 1 for multi-worktree waves. The measured composition is
     EPHEMERAL (`/tmp/awl-final/{R1..R4,C,D}.sh`) — capture or regenerate it in the first
     commit. R4 carries both slow atoms (≥37 s of the 52.2 s wall) — item 374 raises this
     item's ceiling; judge together.
     **Verify:** count-sum assertion proven non-vacuous (delete a prefix, watch the gate
     refuse by name); pass/ignored equal to an unsharded baseline on the same commit;
     `scripts/test-native-gate.sh` green — its CPU-heartbeat law flakes under exactly this
     load, so a red heartbeat is contention first, rerun alone. Report new wall time and
     process count. **Routing:** deep tier — this edits the script that issues the receipt;
     the failure mode is a green gate that tested less than it claimed.

374. **Two test atoms cost 37 of the sharded suite's 52 seconds.**
     `render::tests::diagonal_pixel_composition::` = 22.51 s / 5 tests;
     `render::tests::frost_context_item298::` = 14.51 s / 3+1 — more wall time than ~380
     neighbours combined, hidden in the 104-submodule long tail, both in shard R4 (they set
     373's floor). **Question:** what makes each test cost seconds (per-cell device/pipeline
     work? oversized offscreen targets? frames rebuilt per assertion?) and can it come out
     WITHOUT weakening the law. ‼ The cost is the law's own sweep (both are roster × DPI
     sweeps rendering real frames — `diagonal_pixel_composition.rs:63` iterates
     `theme::THEMES` × `[1.0, 2.0]`; `frost_context_item298.rs:125` sweeps worlds), and
     CLAUDE.md demands that breadth — **narrowing enrolment is the satisfiable-by-deleting-
     its-subject failure wearing a stopwatch.** Legitimate targets are per-cell overhead
     only. "The cost is the coverage" is a real closing answer; it tells 373 its floor.
     ⚠️ Figures are one host/one configuration — re-measure before and after, same machine.
     ⚠️ 365 renames `frost_context_item298.rs` — sequence 365 first or accept a fix-up pass.
     **Verify:** mutation-prove ≥1 law per module after the change; enrolment cell counts
     equal before/after; `cargo test render::` green under the filter. **Routing:**
     production tier.

375. **Raise the lane ceiling to 6–8, put a queue in front of the gate, and partition lanes
     by MECHANISM, not file.** ⚠️ Gated on 373 — short gates are the enabler. (User decision
     2026-08-09.) The four-lane ceiling is gate contention, not editing: each gate runs four
     full-suite passes each pinned to ~1 core by `testlock::serial()`, so four lanes gating
     = ~16 sustained test cores on ten (the measured load-69.79 wave). With 373 a gate is
     ~1–2 min and gates stagger. Three parts:
     **(a)** README ceiling 6–8 lanes; `worker-build.sh` budget stays; disk ~5 GB/worktree ⇒
     eight lanes ≈ 40 GB+, governed by the existing disk-preflight section — cite it.
     **(b)** Gate ARBITER: extend `.orchestrator/native-gate.marker` from advisory to a
     blocking queue — at most 1–2 gates (lane or train) at once, each at full shard width;
     marker names holder PID/sha/start. Replaces "wait for the wave to quiesce"; also
     structurally retires the CPU-heartbeat false-red class (that self-test fails from
     gate-collision load alone).
     **(c)** README §8 at MECHANISM granularity: keep "two lanes on one mechanism are one
     lane"; stop serializing on hub files (`keymap.rs` `Action` enum, `commands/catalog/*`,
     `mod` lists, `code-health.toml`, `assets/keymap-defaults.toml`) whose edits are
     append-shaped and merge cleanly. Evidence: ~5 files/action and ~10/setting means every
     feature crosses hubs, so file partitioning transitively serializes nearly everything;
     and the partition's recorded bill — a lane duplicating a chrome geometry around a held
     file — is the "same behavior ⇒ same code" violation it was meant to prevent. The serial
     merge train, re-gating every landing on the exact combined candidate, stays the
     collision catcher.
     **Verify:** README states ceiling/arbiter/rule with the measured numbers; arbiter
     demonstrated live (second gate queues, naming the holder); heartbeat-flake note
     repointed at the arbiter. **Routing:** production tier — protocol prose + one small
     scripts change.

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
