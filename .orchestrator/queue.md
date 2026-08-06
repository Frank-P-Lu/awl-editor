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

1. **THE SITE IS STALE AGAINST THE PUBLISHED RELEASE.** Measured 2026-08-06
   straight off the live host: `curl -s https://awl.computer/version.json` returns
   `{"version": "0.0.0", "prerelease": true}` — the "no tagged release yet"
   placeholder — so the live Check-for-Updates page tells visitors no release
   exists **while `v0.9.0` is public**. The landing page's `tar xzf` snippet is
   stale too. ⚠️ **`deploy-web.yml` cannot be run: NO repository secrets are
   configured at all** (`gh api …/actions/secrets` returns an empty list), so
   `FLY_API_TOKEN` is absent and the workflow fails on its first step — which is
   the deliberate design (RELEASING.md §2), not a bug. **One
   `gh secret set FLY_API_TOKEN` and one `gh workflow run deploy-web.yml` closes
   it**, and both are the user's to run. Once deployed, `version.json` picks up
   `0.9.0` with `prerelease: false` — the correct value, since that field means
   "a tag exists" rather than beta-vs-stable.
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

✅ **CLOSED HERE 2026-08-06 — item 118's direction call. THE TARGET SHAPE IS
DROPPED.** The user's words: *"i think we just drop the target. it's fine, right
now."* The roster's measured mean of **2.20** is accepted as awl's shape rather
than a shortfall against the aspirational 2.90, consistent with PHILOSOPHY's
calm bias. **`1, 7, 6, 4, 2` / mean 2.90 is retired**; it is not amended, not
restated, and not replaced by a descriptive one. Recorded in item 118's body as
well as here — **this item has already been answered twice by the user because a
decision recorded in one place was invisible in the place it gets read.**

## 🔵 OWED — live work that nothing above implies. Never cleared by a compression.

- **258 — a visual-judge pass over Mulga's new `Pinstripe` ground.** Six captures
  in the lane's worktree `gallery/item-258/` (gitignored). **The question for the
  judge is the one arithmetic cannot answer:** are fine vertical rules too
  fabric/corduroy for a literary slab-serif world, and is there enough separation
  from the other Pinstripe worlds given that ground has no per-world dials beyond
  palette. **Stated fallback if the verdict is "too technical": `Gradient`, one
  literal.**
- **260 — a visual-judge pass over Magpie's new `Bands` ground.** ⚠️ **The lane
  wrote its gallery shots to a temp dir; re-capture before judging rather than
  trusting `/tmp`.**
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

**Nothing is claimed and no lane is running.** Order:

1. **290** — the theme-font debounce rip-out. A user decision with its measurement
   already taken, and it deletes a constant that is rotting against a growing
   fixture. ⚠️ **It does not fix the reported theme-switch lag and must not be
   closed as though it had.**
2. **288 and 289** — small, already diagnosed, and both are user-visible-rule
   debt from the last wave. 289 moves fifteen worlds' 2× appearance, so it is the
   larger of the two and gates a second `Rules` carrier.
3. **131e** — selection and the full Verify clause; 131a–d are landed and the
   measured cluster rail exists in `render/chrome/diagonal.rs`.
4. **172's closure call.** Every domain in `docs/app-domains.md` reads
   "extracted" and the root is 20 fields — **the Done clause reads met.** Either
   close it deliberately with the census as its receipt, or name what remains.
   **Do not leave it open by default** — an item left open after its work landed
   is what wasted a dispatch on item 116 and what misfiled item 211 today.
5. **274's residual** — `overlay/tests.rs` (3433) and `buffer/tests.rs` (2241)
   are still monoliths against the ~500-line ceiling, and only
   `app_icon/tests.rs` carries a declared exception. The verbatim-move contract
   and the per-filter verification are in 274's body.
6. **273's six unbuilt residuals** — CLI flags have no roster to generate from,
   `Command` carries only `name`, `WORLDS.md`'s columns are hand-written, there
   is no in-app door, the site page is visually unreviewed, and the five-section
   structure was the lane's call rather than the user's.
7. **174** — one surface family migrated of every surface; the rest still own
   their geometry.
8. **227** — the AppImage. Nothing in the tree matches `AppImage`; it is
   unstarted and depends on 226, which is now complete.
9. **231** — a diagnosis item with **no live lead**; its shader-size hypothesis
   was falsified and nothing replaced it. Its named next step is a **macOS guest
   VM**, and this host has **no VM tooling installed** — a spend decision, not
   work to absorb.
10. **🔵 HUMAN / LIVE, none of which a lane can close** — see the BLOCKED and
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

131. **Give Mangrove and Magpie mirrored diagonal-line compositions across
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

172. **Decompose the 107-field `App` into owned state domains with narrow
     transition APIs.** **Build:** migrate fields and their invariants
     incrementally into explicit owners; each owner exposes domain transitions
     rather than public fields, and cross-domain work travels through typed
     outcomes/effects rather than back-references to `App`. Preserve the
     active-buffer whole-slot ownership law, fake-clock determinism, wasm gating,
     GPU recovery, and byte-identical behavior. Do not introduce a service
     locator, trait-per-method architecture, message bus, or flag-day rewrite.
     **Done:** `App` is lifecycle composition rather than the mutable home of
     every subsystem, and new workspace or persistence behavior has one obvious
     owner.

     🟢 **EVERY OWNER IS NOW EXTRACTED.** The root is **107 fields → 20**, and
     `docs/app-domains.md`'s table reads "extracted" for `WorkspaceState`,
     `PersistenceRuntime`, `DocumentSession`, `InputRuntime`,
     `ConfigurationRuntime`, `ProjectLocation`, `FrameRuntime` and `UsageLedger`,
     with 12 host/lifecycle fields staying on `App` deliberately. **The map is a
     deliverable in its own right** — `docs/app-domains.md`, with the same table
     as executable data in `src/app/tests/domains.rs`, exhaustive by construction.

     🔵 **SO THE STANDING QUESTION IS WHETHER THIS CLOSES.** Read the census as
     the receipt and close it, or name what remains — **do not leave it open by
     default.** Two facts a closer needs: the item **named two different domains
     `WorkspaceState`** and the name went to the summoned-UI meaning, with the
     project-folder domain becoming `ProjectLocation`; and **the one place
     byte-identity was consciously chosen over consistency** is
     `sync_cursor_icon`'s raw `popover_summon_bit()` read — documented, single
     call site, law-counted so a second consumer fails by name. **A byte-identity
     refactor preserves pre-existing bugs**, so if a second pair of eyes is spent
     anywhere on this work, that is the spot.

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

227. **Add a desktop-integrated AppImage as awl's friendly Linux download.**
     **Defect:** the tarball is appropriate for technical early adopters but is
     not a normal Linux desktop application: it has no launcher metadata or icon
     integration. **Build:** package awl as an x86_64 AppImage in the release
     workflow, alongside — not instead of — the tarball. Include the binary, a
     `.desktop` launcher entry, the canonical Linux PNG icon derived from the
     existing icon pipeline, licenses/credits, and only the runtime libraries that
     belong inside the package; **do not bundle GPU drivers.** Publish a checksum
     and stable release-asset name. **Done:** a user can download one file from
     GitHub Releases, mark it executable, launch awl, and receive correct desktop
     name/icon integration where the desktop supports it; the tarball remains
     available as fallback. **Verify:** AppImage structural validation; launch and
     headless smoke on representative Debian/Ubuntu and Fedora-like environments;
     Wayland and X11 launch checks; icon/desktop-entry law; GPU-adapter and
     file-open smoke; mutation proof removes launcher/icon packaging; release dry
     run uploads both Linux artifacts. **Routing:** production tier with a Linux
     visual/compatibility audit. ⚠️ **Item 226 is complete and the glibc floor is
     settled at 2.35, so the "decide it together with the support matrix" coupling
     has retired.** Confirmed unstarted: nothing in the tree matches `AppImage`.

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

273. **THE REFERENCE MANUAL — SIX RESIDUALS, named as unbuilt rather than implied
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
     **Scope:** the site page's presentation only — `REFERENCE.md`'s content and
     sectioning are settled above and do not move. **Verify:** the drift laws
     still hold across the restructure; every section reachable and linkable;
     the page reviewed on a real browser at desktop and narrow widths. ⚠️ **Zero
     network is a design invariant — no CDN, no webfont fetch, no script from
     off-host.** **Routing:** production tier.

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

274. **THE TEST MONOLITHS — two decomposed, THREE STILL STANDING.** `theme/tests`
     and `main/tests` are submodule dirs now (17 and 20 files). ⚠️ **Measured
     2026-08-06: `src/overlay/tests.rs` 3433, `src/app_icon/tests.rs` 2368,
     `src/buffer/tests.rs` 2241** against CLAUDE.md's *"~500 lines is a file's
     natural ceiling"*, and only `app_icon/tests.rs` carries a declared exception
     in `scripts/code-health.toml`. **The precedent is overwhelming:
     `src/render/tests/` is one hundred and six files.**

     **Build:** decompose each remaining monolith into a `tests/` submodule
     directory, **verbatim** — names and module paths unchanged, so
     `cargo test overlay::tests::foo` keeps working and no law's `--exact` filter
     breaks. Split by SUBJECT. **Scope:** test files only; production code
     untouched and byte-identical. **Not** a rewrite, **not** a chance to
     "improve" a test while moving it, **not** a place to delete a test that looks
     redundant — a verbatim move is auditable and a rewrite is not.

     ⚠️ **THE ONE THING THAT MAKES THIS RISKY RATHER THAN MECHANICAL:
     `crate::testlock::serial()` and the `cfg(test)` global writers.** A move that
     changes which tests share a file changes **nothing** about locking — but it
     changes which tests a developer runs together under a filter, and this repo
     has standing proof that a suite can pass alone, pass unfiltered, and fail
     only under one filter. **So the decomposition must be verified under the
     filters it creates.** **Done:** no `tests.rs` exceeds the ceiling without a
     declared, reasoned exception; every test name and module path is unchanged;
     `cargo test --bin awl` reports the **same count** before and after.
     **Verify:** the identical count is the primary oracle; then run each new
     module as its own filter as well as the full suite and a wide
     `--test-threads`. **Routing:** production tier — mechanical by design, and
     the value is entirely in it being boring.

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

288. **THREE IDENTIFIER-LEVEL CITATIONS — RECOMMENDED BY ITEM 287, DELIBERATELY
     NOT ACTED ON.** These are the same Conventions rule as 275/287, in the one
     place where fixing it is a *behaviour* change rather than a text edit: a test
     name is what `cargo test <substring>` filters on, and a filename is what
     `mod.rs` declares. **So each rename must move its declaration and any
     external filter with it, in one commit.** All three verified present
     2026-08-06:
     - `theme::tests::personality::bar_config_shipped_is_the_flip_round_hug_all_hybrid`
       → drop the round name; the doc's own language is "HUG-ALL HYBRID".
     - `theme::tests::fonts::mopoke_body_face_is_bitter_with_the_item_30_bullet_triple`
       → same shape.
     - `src/theme/tests/world_pin_item254.rs` → ⚠️ **this one is WRONG, not merely
       stale.** `git log -S "struct WorldPin"` puts the type's origin in **item
       94**; item 254 is the unrelated flaky-`alloc_bound_law` item. `item94` stays
       inside `code-health.py`'s `TEST_FILENAME_ITEM_INDEX` regex
       (`_item\d+[a-z]?\.rs$`), so no tooling change is needed — but the citation
       should point somewhere real, or be dropped for the mechanism.

     ⚠️ **THE FINDING WORTH KEEPING, BIGGER THAN THE THREE RENAMES:
     `TEST_FILENAME_ITEM_INDEX` BLESSES THE FORM OF A CITATION WITHOUT CHECKING
     THAT IT POINTS ANYWHERE REAL.** `world_pin_item254.rs` passed that exemption
     for its whole life while naming the wrong item; an exemption that
     pattern-matches `_item\d+\.rs` cannot tell 94 from 254. **If these filenames
     are kept as a convention, the exemption should verify the number against the
     board — otherwise it is a check that runs in one configuration and cannot see
     its own subject.** **Routing:** production tier; the renames are mechanical,
     the exemption question is a small design call.

289. **`FacetStyle::Text` AND `ChipVariant::Underline` DRAW THEIR MARK IN RAW
     DEVICE PIXELS, SO IT IS HALF-WEIGHT ON RETINA.** Measured by item 283's lane
     and deliberately not absorbed: the underline probes `[430.0, 153.5, 23.9,
     1.5]` at DPI 1 and `[860.0, 306.6, 47.8, **1.5**]` at DPI 2 — **every other
     term doubles and the thickness does not.** ⚠️ **This is item 242's
     chrome-pixel-space rule (author in logical units, multiply once at the
     boundary) with two dials that never got the memo**, and it is exactly the
     class CLAUDE.md's DPI tripwire names: every capture runs at
     `--capture-dpi 1`, the one scale at which it looks correct. **Scope: 14
     `FacetStyle::Text` worlds plus Wagtail's `Underline` chips.** It moves 15
     worlds' 2× appearance, which is why it is its own item and not a footnote.
     **Verify:** the mark's thickness scales with DPI like every neighbour, swept
     at 1× and 2× across the affected roster. **Routing:** production tier.
     ⚠️ **Confirm the premise with a capture before changing anything** — it is a
     lane's report, and a lane's report carries no privilege either.

290. **RIP OUT THE THEME-FONT DEBOUNCE ENTIRELY.** **User decision, made against
     the measurement rather than in ignorance of it.** **Build:** delete
     `THEME_FONT_DEBOUNCE_DEFAULT_MS` (`src/app.rs:68`), the
     `AWL_THEME_FONT_DEBOUNCE_MS` override and its parser,
     `theme_font_reshape_decision` and the whole
     `src/app/theme_font_debounce.rs` module, `THEME_FONT_CHEAP_RESHAPE_MS`
     (`src/app.rs:105`) and its compile-time assert, and the deferred-settle path
     (`App::apply_deferred_theme_font`) once nothing calls it.
     `App::retint_theme_preview` reshapes on every preview step, unconditionally.

     **The case FOR: the mechanism is BIMODAL, and its mode boundary sits inside
     the range of natural human key cadence.** Measured (release, Apple Silicon
     Metal, 900×600 @2x, `--debug`, CLAUDE.md, 12× Down in the theme picker,
     display verified unlocked at both ends of 17 runs), arm sequence per cadence
     at the shipping 100 ms window (`I`=Immediate, `C`=Coalesce):

     | cadence | arms | stall |
     |---|---|---|
     | 60–95 ms | `ICCCCCCCCCCC` | 141–151 ms |
     | **100 ms** | **`IIIICCCC`** | **146, 138 ms** |
     | 105–250 ms | `IIIIIIII` | none |

     **A hard mode boundary at 100–105 ms with a ~4× latency gap across it** —
     immediate steps settle in 30–45 ms, a coalesced burst in 140–150 ms. At
     exactly 100 ms the flip lands mid-run: four normal steps, then a stall. That
     is the user's reported "hit it on the third item". **Uniform cost is
     predictable; a mode that flips on jitter is not, and that is the argument.**
     ⚠️ **The IRREGULARITY is inferred, not measured** — the harness robot has
     near-zero jitter so it flips once and stays; a human at ±30 ms around 100 ms
     would cross repeatedly within one run. No instrument here reproduces that.

     **A second, unexpected amplifier: coalescing MANUFACTURES work.** Only 8 of
     12 hops need a reshape at all at ≥100 ms (four are same-face/same-palette,
     ~2 ms each) — but during a coalesced burst **all 12** report needing one,
     because `shaped_font` goes stale while nothing reshapes.

     **Separately, its cost model has silently rotted:**
     `THEME_FONT_CHEAP_RESHAPE_MS` is anchored to a "12.0 ms on CLAUDE.md" figure
     whose fixture is CLAUDE.md itself, which has grown 44% (17,541 → 25,197
     bytes) and now measures 12.5–17.7 ms. **A constant calibrated against a
     live, growing fixture is a check that cannot see its own subject.** ⚠️ Note
     the gate never actually opens here: measured `last_reshape_cost` was
     **22–40 ms** in every run, 5–10× above it. The rot is real; it is not what
     drives the arm.

     ⚠️ **THE COST, WHICH THE LANE MUST MEASURE RATHER THAN ASSUME.** Removal
     regresses the BURST case — the one place the coalesce genuinely works.
     Measured today at the shipping window: a fast burst (no inter-key gaps)
     settles in **12.3 ms with n=1 reshape for 8 inputs.** With no debounce that
     becomes 8 synchronous reshapes queueing on the single main thread, which is
     precisely what item 37b's zero-default shipped and what item 202 was built to
     undo. **Record a before/after on the burst case explicitly. If it regresses
     materially, REPORT BACK rather than shipping it silently — the user accepted
     this trade in principle, not at an unmeasured magnitude.**

     ✅ **THIS ITEM DOES ADDRESS THE REPORTED LAG — an earlier draft of this body
     said it did not, and that was written off the "inert" reading now
     withdrawn.** The reported symptom IS the coalesced mode; removing the
     debounce leaves every step on the immediate arm at 30–45 ms and the stall
     mode ceases to exist. A residual ~30–45 ms per step remains and is
     reshape-bound (`sync_theme` 12.5–17.7 ms on CLAUDE.md, 23.7–28.8 ms on a
     1896-line fixture) — real, but uniform, and separate work.

     ⚠️ **RAISING THE WINDOW IS NOT THE ALTERNATIVE — measured and refuted.** At a
     deliberate 250 ms cadence, a 300 ms window gives arms `ICCCCCCCCCCC` and a
     **348 ms** settle; 400 ms gives **459 ms**. Raising it converts "every step
     immediate, uniform ~40 ms" into "nothing re-renders while you browse, then a
     one-third-of-a-second snap." **p50 movement-latency IMPROVES while the felt
     settle gets 8–10× worse**, because colour-only steps present fast and get
     sampled while stalled ones do not — the wrong statistic over a
     survivorship-biased sample. Do not re-propose it without new evidence.

     **Scope:** does NOT include raising the window, attacking reshape cost, or
     touching `themeswitch.rs`'s phase roster beyond whatever `SwitchPhase::Wait`
     becomes when it is structurally always zero — **name that decision rather
     than leaving a dead column.** **Done:** no debounce machinery remains, no
     constant is calibrated against a moving fixture, and the burst-case cost is
     measured and recorded rather than discovered later. **Verify:**
     `--bench-theme-burst` with its reshape-count witness (CLAUDE.md warns a theme
     bench once "measured" 5 ms while nothing reshaped — **assert the count**);
     before/after at both cadences; the isolated-step path unchanged. Feel is
     live-only and gets flagged for the user, never claimed. **Routing:**
     production tier.

291. **THE THEME-SETTLE INSTRUMENT CANNOT SEE THE STALLS IT EXISTS TO MEASURE.**
     **Defect, measured not argued:** a **Coalesce** step never creates a
     transaction at all. `retint_theme_preview`'s Coalesce arm calls only
     `arm_theme_font(now)`, so `sync_theme_font_measured` — the SOLE creator of
     `ThemeSettleInFlight` — is never reached; only the eventual
     `apply_deferred_theme_font` makes one, timed from `theme_switch_at()`, i.e.
     **the last input's stamp**. Measured at 60–90 ms cadence: **12 reshaping
     inputs produce 2 recorded transactions.** Ten of twelve steps are invisible
     to `theme latest`, `theme worst` and the whole breakdown. **The user
     reported this from the product before it was found in the source** — "the
     lag isn't captured by theme worst values."

     ⚠️ **`MIN_PHASE_COVERAGE`/`unaccounted` CANNOT CATCH THIS, BY CONSTRUCTION.**
     The floor guards transactions that were RECORDED; an absent transaction
     cannot show a shortfall. `themeswitch.rs`'s own module doc presents that
     floor as the guarantee that future blind spots self-report. **It does not
     cover the blind spot that drops the transaction.** This is the repo's
     standing lesson in its purest form: the check ran in the one configuration
     that could not see its subject.

     **Three qualifications, so this is not overstated:** the transaction that IS
     recorded shows ~147 ms, so the readout is blind to the stall's FREQUENCY,
     not its magnitude; it still under-reports the felt total, because eight
     arrows over 700 ms with no re-render feel like 700 ms while the readout says
     147 (it measures from the LAST input); and `SWITCH_WINDOW` is 5 s with
     `settle_lines` returning an empty vec when empty, so **the lines vanish
     entirely 5 s after the last switch** — a user who feels lag and then looks
     finds nothing.

     ⚠️ **THE SAME SURVIVORSHIP BIAS IS IN THE HARNESS.**
     `probe::mark_movement_input` OVERWRITES a still-pending mark by design, so a
     zero-gap burst reports `n=1` for 8 inputs. Every p50 taken through it is
     conditioned on "steps that got a present" — which is how a diagnosis lane
     concluded the debounce was inert. **Fix both or neither; a repaired readout
     over a biased probe still lies.**

     **Build:** record a transaction per preview STEP, not per completed settle,
     or surface a dropped/superseded count beside the headline. **Done:** the
     number a user sees moves when the product stalls. **Verify:** a synthetic
     burst at 60–95 ms cadence reports 12 of 12, not 2 of 12; mutation-prove by
     restoring the drop and watching the law go red. ⚠️ **Sequence AFTER 290 if
     290 lands** — with no debounce there is no Coalesce arm, which removes this
     defect's cause but NOT the harness bias or the 5 s vanish. **Routing:**
     production tier.

292. **KITE'S ACTIVE LENS CHIP COLLIDES WITH THE CARD'S TOP EDGE.**
     **User-reported with screenshot 2026-08-06.** In Kite's command palette the
     filled `All` chip's plate runs flush into the top edge of the strip band,
     with no breathing room above it — the highlight reads as clipped rather than
     seated. **Diagnose before fixing:** `strip_gap()`
     (`src/render/chrome/mod.rs:163`) is HORIZONTAL only (`CHIP_STRIP_GAP` vs
     `STRIP_GAP`), so it is not the owner; the vertical inset between the strip
     band's top and the chip plate is. ⚠️ **Verify the premise before building —
     confirm from the drawn pixels which quantity is short**, and check whether
     it is Kite-specific or true of every `FacetStyle::Chips` world (Kite is the
     reporter, not necessarily the only carrier). **Scope:** vertical seating of
     the chip within its band; does NOT include the chip's horizontal gaps or
     item 289's unscaled underline. **Verify:** the plate's top inset scales with
     DPI like its neighbours, swept 1×/2× across every `Chips` world;
     byte-identity for non-`Chips` worlds. **Routing:** production tier.

293. **THE OVERLAY FOOTER CROWDS THE LAST ROW — A PANE-WIDE FIX, NOT KITE'S.**
     **User-reported with screenshot 2026-08-06, and the user explicitly scoped
     it: "that should be a pane world specific fix (eg fix for all panes)."** The
     grey hint line (`type to filter · ↵ choose · ←/→ category · esc close`) sits
     hard against the last candidate row with no separating space, so system text
     and content text read as one block. **Diagnose before fixing:**
     `OverlayGeom::hint_rows` documents itself as `footer.len() + 1`, "a blank
     separator line between the hint and the band", and the card is said to grow
     by exactly that — **so a separator is already specified and the screenshot
     shows none. Establish whether it is not computed, not drawn, or being
     consumed by a clip** before changing any constant. ⚠️ **SCOPE BROADENED
     2026-08-06, SAME DAY: this was first scoped `Pane`-wide on the user's
     instruction, then observed AGAIN on Cassowary's right-click menu — a
     per-row-plate composition, not `Pane`.** Two compositions showing the
     identical symptom means the owner is the shared footer/band geometry, not a
     list style. **Sweep the whole roster and every `OverlayKind`, including the
     context menu**; the `Pane` framing was the first sighting, not the boundary.
     **Verify:** the gap exists and is measurable in pixels on every world at
     1×/2× DPI, on full, filtered, scrolled and empty lists —
     the empty-state notice row shares this band and has produced a footer/plate
     collision before. **Routing:** production tier.

294. **THE THEME PICKER IS UNREADABLE OVER A PLATELESS WORLD — BLUR ITS OWN
     FOOTPRINT, AND ONLY ITS FOOTPRINT.** **User-reported with screenshot
     2026-08-06 (Magpie): the document's prose and the world names interleave
     glyph-for-glyph — "the world loudness map" through "Magpie", "pretty good?"
     through "Kite" — two readable layers in one place, which DESIGN.md forbids.**

     **Why it happens, and it is a deliberate decision rather than an oversight.**
     `render/blur.rs`'s module doc: *"the THEME PICKER and the CARET-STYLE PICKER
     stay CRISP (no backdrop at all) — their whole job is showing the live theme
     colours"*; `pipeline_prepare.rs:82` names the frost as what *"would defeat
     the theme picker's crisp live-color preview."* That holds for worlds with a
     plate. It fails for **plateless compositions** — `Diagonal` (Mangrove,
     Magpie) and now `Rules` (Paperbark) — because **frost is a property of the
     plate, and those compositions deliberately draw none.** The picker is also
     the ONE overlay whose backdrop is chosen by the row under the cursor rather
     than by the user, so it drags a reader through worlds they never picked.

     ✅ **THE EXCEPTION IS OVER-BROAD, AND THE SOURCE SAYS SO.** The blur
     **already preserves hue** — "a defocus, not a desaturation — the whole
     point". The only thing that shifts the palette is `DIM = 0.16`, documented
     as "0 = pure blur, no recede". **The exception was protecting the colour
     preview from `DIM` and discarded the hue-safe blur with it.**

     **Build (the user's shape, given directly):** route the theme picker through
     `BlurBackdrop` with **`DIM` at or near 0**, and **scope the composite to the
     picker card's own footprint** so the surrounding page stays crisp and the
     world's live colours remain judgeable — which is the whole reason the
     exception existed. ⚠️ **The footprint scoping is the real work:**
     `draw_backdrop` currently draws a **fullscreen triangle**, so this needs a
     scissor rect or a rect uniform, not a flag. **`Pane` worlds are explicitly
     unaffected** (user's words) — their plate already covers the document, so
     blurring under it is invisible and wasteful; gate on the plateless
     compositions rather than paying for it everywhere.

     ✅ **SCOPE DECIDED 2026-08-06: BOTH CRISP PICKERS, AND NO CARVE-OUTS.**
     User: *"we're only blurring the area UNDER the theme picker... so if it's
     over the caret then it should be blurred too."* **The footprint is the
     whole rule** — whatever the card covers is blurred, including the caret if
     the card happens to sit over it, and the caret picker takes the identical
     treatment rather than staying excepted. **Do not carve the caret out of the
     blurred region** and do not special-case the two pickers against each other.
     ⚠️ **One consequence to watch live rather than design around:** a caret
     picker card positioned over the caret will blur the very caret it previews.
     If that reads badly in use it is an ANCHORING question — where the card
     opens relative to the caret — **not a reason to reintroduce a carve-out.**
     ⚠️ **THE EXCEPTION IS `Theme | Caret` AND NOTHING ELSE**
     (`src/app/viewstate.rs:165`) — an earlier draft of this item said the
     HISTORY picker was also in it, on the strength of
     `render/chrome/outline.rs`'s prose "the CRISP theme/caret/history pickers".
     **That prose is loose and the code is not:** History is deliberately
     excluded, with its reason stated at the assignment — its comparison is
     composited inside the workspace's own content region, so what sits behind
     the card is the user's untouched document, a quiet backdrop DESIGN.md §5
     says recedes. **Nothing about History is open here.** Does NOT include a
     general scrim: an earlier proposal to dim the document under every summoned
     overlay was **rejected by the user** as unnecessary — do not re-propose it.

     **Done:** on every plateless world the picker's own text is the only
     readable text within its footprint, while the page outside it still shows
     the previewed world's real colours. **Verify:** pixel arithmetic over the
     footprint proving no document glyph survives as text, plus a hue check
     outside it proving the preview is untouched; swept across `Diagonal` and
     `Rules` worlds at 1×/2× DPI; byte-identity for every `Pane` world.
     ⚠️ **Laws pin the current crisp behaviour in at least `render/tests/hud.rs`,
     `one_bit.rs` and `outline.rs` — they must be RE-AIMED, not deleted**, and
     one of them exists to stop the HUD forcing a frost that defeats the preview.
     **Routing:** deep tier — it touches a GPU composite path and a design
     decision with a written rationale.

295. **EXPORT IS A BROKEN BUTTON — THREE COMPOUNDING FAILURES IN ONE THREE-ITEM
     MENU SECTION.** **User-reported: "file -> export as pdf... doesn't DO
     anything???? like this is a usability nightmare."** An audit confirmed the
     feature works AND that the user's words are literally accurate for their
     input. **Three independent defects, ranked:**

     **(a) 🔴 On a NON-MARKDOWN buffer it is a total no-op.** `src/actions.rs:369`
     — `Action::ExportWord | Action::ExportHtml => Effect::None` behind an
     `is_markdown()` gate, and `ExportPdf` carries the identical gate. No write,
     no notice, nothing dispatched. **Reproduced on the shipped binary**, not
     inferred: the palette matched and ran the command (`overlay.items:
     ['Export as PDF…']`), the overlay closed as if something executed, the
     sidecar notice was `""`, no file appeared. **And the menu row is built
     `enabled: true` unconditionally** (`src/menu/native.rs:16-27`) —
     `set_markdown_enabled` greys only the Markdown submenu — so nothing warns
     the user. `.txt` and light code are in scope per PHILOSOPHY, so this is a
     reachable everyday path.

     **(b) 🔴 The ellipsis lies, and this is the PROXIMATE cause of the report.**
     Every other ellipsis row in the File menu — Browse files…, Switch project…,
     Recent projects…, Rename file…, Move file…, Version history… — opens a
     further surface. Save and Duplicate file carry no ellipsis and complete
     immediately. **Export as PDF…/Word…/HTML… are the ONLY ellipsis items in the
     entire menu that complete immediately with no surface at all.** The label
     trains the user to wait for a panel that is never coming. Verified by
     enumerating every row's actual dispatch.

     **(c) 🟠 Destination surprise on the unsaved/unconfigured path.**
     `export_document` writes a sibling of a saved file, else into
     `project_location.root` — which `resolve_launch_context`
     (`src/main/run/location.rs:36-70`) falls back to `crate::fs::data_root()` =
     `~/.local/share/awl`, an app-internal dot-hidden directory Finder does not
     show. A first-time user exporting the Welcome document or a scratch note
     cannot browse to the result. ⚠️ **`docs/platform.md:88` glosses this
     fallback as "`~/notes` by default", which is wrong for the unconfigured
     case — fix the doc in the same pass.** The toast names the filename only,
     never the path.

     **Scope:** this item does NOT decide the parked save-dialog question — (a)
     and (b) are defects regardless of how that lands. **Done:** the menu row
     tells the truth about what it will do, and no invocation completes with the
     user unable to tell whether anything happened. **Verify:** a capture over a
     non-Markdown buffer proving either a disabled row or an explicit notice; the
     ellipsis convention asserted across the whole `FILE_ITEMS` roster by a law,
     so a future row cannot dodge it. **Routing:** production tier.

296. **`ConvertLineEndings` IS A PURER SILENT SUCCESS THAN EXPORT.**
     `Action::ConvertLineEndings` (`src/keymap.rs:118`) flips the file's on-disk
     EOL convention with **`Effect::None`** — no notice at any time
     (`src/actions.rs:158-161`; the unit test's own comment: "convert is a plain
     metadata flip, no effect"). It is also deliberately NOT on the undo timeline
     (the settled VS Code model), **so a double-toggle is undetectable**: the
     user cannot see that it happened, cannot see what it did, and cannot undo
     it. **Severity is bounded by reach, not by shape** — palette-only, unbound
     by default, power-user name. **Build:** a notice naming the resulting
     convention. **Verify:** sidecar assertion that the notice is set; the EOL
     metadata itself is already covered. **Routing:** production tier.

     ⚠️ **A HARNESS GAP FOUND WHILE AUDITING THIS, and it is the more important
     half.** The audit tried to pixel-prove the export toast illegible via
     `--screenshot-app` and got **no notice text in the frame at all** — then
     cross-checked against `Cmd-S`, which also sets a toast, and got the
     identical empty result. **The offscreen capture pipeline does not render
     transient toast chrome for ANY action.** So every "the notice is set" claim
     in this tree is a SIDECAR claim, and no capture has ever proven a toast is
     visible to a human. That is CLAUDE.md's own tripwire — the sidecar is a
     state oracle, not an appearance oracle — sitting unexercised over a whole
     feedback channel. **Perceptibility of toasts is therefore UNVERIFIED, not
     verified-good; treat it as live-only until the harness reaches it.**

     ✅ **THE CAUSE IS NAMED, AND IT IS A STALE ASSUMPTION IN A COMMENT.**
     `prepare_notice` (`src/render/chrome/readout.rs`) documents itself as "one
     quiet LABEL-sized line in the muted ink at the BOTTOM-CENTER of the writing
     column (today: the autosave external-change guard's 'changed elsewhere')"
     and states that "an EMPTY notice parks it off-screen, so **every capture
     (which can never have a notice — autosave is live-only)** stays
     byte-identical." **That parenthetical was true when autosave was the sole
     caller and is FALSE NOW:** `set_toast_notice` has ~10 callers including
     `exported {shown}`, `downloaded {name}`, `saved your version`, `reloaded —
     changed elsewhere` and `graphics recovered` — and export IS reachable
     headlessly. **The capture pipeline was designed around an invariant that
     later callers quietly invalidated, and the comment still asserts it.** So
     the harness gap is not an oversight in the capture path; it is a live
     assumption that stopped being true. Fix the comment in the same pass, and
     treat "no capture can have one" as a claim to re-verify rather than inherit.

     ⚠️ **CORROBORATING EVIDENCE, worth more than any measurement here: the USER
     DID NOT KNOW awl HAD TOASTS AT ALL** ("we had toasts...??"), having shipped
     the feature. A feedback channel its own author has never noticed is not a
     feedback channel. **This is the strongest argument that the fix is a design
     change to notice presentation, not merely a harness repair** — but the
     design call is the user's, so this item stops at naming it.

297. **CASSOWARY'S ROTATED LOCATION LABEL IS TOO SMALL AND IN THE WRONG PLACE.**
     **User design decision 2026-08-06, with screenshot.** Today
     `LocationStyle::RotatedRail` draws the active facet name ("Tools") as, in
     its own words, "a small, muted run turned 90° and seated flush with the
     card's own left border — a subordinate vertical counterpart to a loud
     primary title". **The user's verdict on that reading: "this is wrong."** It
     currently sits tucked against the list as a tiny sub-heading and reads as
     debris beside the card rather than as a second title.

     **The intent, in the user's words:** *"really big, on its side, but like
     above the 'commands' title... like 2/3 it's size, but along the left edge."*
     So: **rotated 90°, sized at ~⅔ of the Archivo Black `COMMANDS` placard,
     along the ROOM's left edge, positioned ABOVE the placard** — a vertical
     companion to the wordmark, at the wordmark's own scale class, not a label
     hugging the card. The two become one typographic composition reading up the
     left edge and across the bottom.

     ⚠️ **THE DOC COMMENT IS PART OF THE CHANGE.** `LocationStyle::RotatedRail`'s
     own definition (`src/theme/model.rs`) specifies "small", "muted" and "flush
     with the card's own left border" — all three are being overturned, so the
     comment is re-authored in the same commit or the next reader implements the
     old design from the type. **Scope:** Cassowary is the SOLE carrier
     (`src/theme/worlds.rs:984`), so no other world moves; `Raked` (the Diagonal
     worlds' sibling treatment) is NOT in scope. Reuse
     `render/rotated_location.rs::prepare_rotated_location_label` — the
     world-neutral rotated-label capability — rather than adding a second
     rotation path; the type's own comment already forbids that.

     **Responsive bound, and it is the real risk:** at ⅔ of a placard sized for
     the room, a long facet name ("Navigate", "Settings", "Recent") on a short
     window will collide with the card, the placard, or the window top. **Size
     from the real available edge territory and the longest facet in the roster,
     not from the shortest** — the same discipline item 131 states for its
     diagonal. Never overlap the card, never clip, never silently fall back to
     the old small treatment. **Done:** the location reads as a second title at a
     glance, and every facet name in the roster is fully legible at every
     supported window size. **Verify:** every `OverlayKind` × every facet name ×
     narrow/wide/zoom at 1×/2× DPI, with pixel laws for non-overlap against both
     the card and the placard, and for the ⅔ size relation holding as the placard
     scales; byte-identity for all nineteen non-Cassowary worlds; affordance-
     locating vision smoke. **Routing:** deep tier — it is a typographic
     composition, not a constant.

298. **A RIGHT-CLICK MENU SHOULD NOT FROST THE DOCUMENT.** **User decision
     2026-08-06, with screenshot (Cassowary): "when you right click, we shouldn't
     show blur. it's a bit excessive."** A four-row Cut/Copy/Paste/Select-all
     context menu currently takes the full-takeover treatment — the whole page
     defocuses behind a menu occupying a fraction of it. **The blur is for a
     FULL-TAKEOVER overlay** (`render/blur.rs`'s own framing: "the cached, cheap
     defocus behind a full-takeover overlay", naming the palette, go-to, outline,
     keybindings and spell). **A pointer-summoned context menu is not one** — it
     is transient, small, and anchored to where the user clicked.

     **Build:** exclude the context menu from `BlurBackdrop` routing, the same
     way the theme and caret pickers are excluded today. ⚠️ **This is the exact
     opposite direction to item 294 and the two must be read together:** 294 adds
     a FOOTPRINT-scoped blur under the theme picker; this REMOVES the full-page
     blur under the context menu. They agree on the underlying principle — the
     defocus should be proportional to what the overlay actually covers — and a
     lane taking either one should state that principle rather than treating them
     as contradictory. **Sequence 294 first if both are live**, since it
     establishes footprint scoping that this item may want rather than a bare
     off-switch. **Verify:** byte-identity of the document region for a summoned
     context menu across the roster; the full overlays keep their frost.
     **Routing:** production tier.

299. **TWO ROWS IN THE SAME STATE DRAW THEIR ACCESSORY IN DIFFERENT INKS, AND ONE
     IS ILLEGIBLE.** **User-reported with screenshot 2026-08-06 (Cassowary
     context menu): "notice how the 'unavailable' next to Copy is... invisible?
     that's a bug."** Copy and Paste are both disabled and both render the
     accessory text `unavailable` on the same plate — **Paste's reads as legible
     green, Copy's is near-black on near-black and effectively invisible.**
     Identical state, identical string, different ink. One of them is wrong.

     ⚠️ **DIAGNOSE BEFORE FIXING — do not tune a colour.** A one-row offset is a
     live hypothesis worth testing first (Cut is the SELECTED row and Copy is the
     row immediately after it, so an accessory ink resolved from the wrong row's
     selection state would produce exactly this pair), but so is a plain
     `faint()`/`muted()` split keyed on something that differs between the two
     rows. **Establish which row's state the accessory ink is actually read from**
     before changing anything. ⚠️ **The neighbouring possibility is the more
     serious one:** if an accessory resolves its ink from an adjacent row, that
     is a drawn/state disagreement of the same family this repo has already been
     bitten by, and it will not be confined to this menu.

     **Verify:** a law asserting every disabled row's accessory meets a contrast
     floor against its own plate, swept over every `OverlayKind` × selected-row
     position × world — **the sweep is the selection index, because that is the
     axis the offset hypothesis lives on**, and a fixture testing only a
     selected-row-0 menu would be structurally unable to see it. Contrast is
     asserted by arithmetic over the PNG's pixels, never by reading the token.
     **Routing:** production tier.

300. **THE TOAST IS INVISIBLE IN PRACTICE — REDESIGN THE NOTICE.** **User
     decision 2026-08-06: "i've never seen the toast lol."** The author of the
     product has never once noticed a feedback channel with ~10 callers
     (`exported …`, `downloaded …`, `saved your version`, `reloaded — changed
     elsewhere`, `graphics recovered`). **That is the finding; no measurement
     improves on it.** Today `prepare_notice` draws "one quiet LABEL-sized line
     in the muted ink at the BOTTOM-CENTER of the writing column" for
     `TOAST_LIFETIME = 2500 ms` — sub-body size, muted ink, bottom-centre, gone
     in two and a half seconds.

     🔴 **THE PREMISE CHANGED 2026-08-06 — DEBUG BEFORE REDESIGNING.** User: *"i
     havent even seen it. i think there's a bug that's preventing it from
     showing. we should debug and fix this."* **This item was written as a taste
     problem (a notice too quiet to notice) and must now first answer a factual
     one: does the toast render AT ALL on the live path?** ⚠️ **Do not open with
     a redesign.** Establish, in this order: (1) does `set_toast_notice` reach
     `frame`; (2) does `notice_readout_text()` return it; (3) does
     `prepare_notice` place it on-screen rather than parking it off; (4) does the
     frame carrying it actually present. **A redesign of something that never
     draws is wasted work, and the evidence genuinely points both ways** — the
     ink/size/position/dwell are all quiet enough to explain "never seen", AND
     `--screenshot-app` renders no toast for any action (item 296), which is
     equally consistent with a real defect on a shared path. **Whichever it is,
     say so plainly** — "premise false, oracle repaired" and "fixed" read
     identically on a board six weeks later and only one means the product
     changed.

     ✅ **AND IT MUST REACH THE CLI/HEADLESS PATH TOO** — user, same message:
     "make it show up in the cli commands too." **This merges with item 296's
     harness gap:** no capture has ever photographed a toast, and the comment
     asserting none ever could is itself false. A notice the harness cannot see
     is a notice no law can hold. **Sequence 296's repair with this item's
     debugging rather than separately** — they may well be the same defect, and
     if they are, that is the finding.

     **Build (only once the above is answered):** a notice a writer actually
     registers without it becoming chrome that nags. **The tension is real and is the whole design problem:**
     DESIGN.md gives motion to the caret alone and favours summoned overlays over
     persistent chrome, so the answer is NOT a bouncing banner. Candidate axes to
     weigh — dwell time (2500 ms is short for a line you were not looking at),
     ink (muted is the quietest register the palette has), size (LABEL is below
     body), and position (bottom-centre of the writing column is outside the
     reading eye's path, which is arguably the actual defect). ⚠️ **Prototype in
     awl via headless capture, never as an HTML mockup**, and put candidates in
     front of the user — this is a taste call and the item closes on their word,
     not on a contrast ratio.

     ⚠️ **A CONTRAST FLOOR IS NECESSARY AND NOT SUFFICIENT.** A notice can pass
     every arithmetic check and still go unseen for 2500 ms in a place nobody
     looks — which is exactly what shipped. **Do not close this by adding a
     legibility law.** **Depends on 296's harness gap** — until a capture can
     photograph a toast at all, no candidate can be judged on pixels, so 296's
     repair sequences first or this item runs blind. **Routing:** deep tier, then
     the user's eye.

301. **EXPORT SHOULD USE THE SYSTEM SAVE PANEL — AND THE SEAM IS ALREADY HALF
     BUILT.** **User decision 2026-08-06: "surely people want to see what it
     looks like right? and also select where it goes? i wonder if we should use a
     system export for this. like we're already doing this for opening a file."**

     ✅ **THE USER'S PREMISE IS CORRECT, VERIFIED:** `mac_chrome::pick_file_to_open`
     (`src/mac_chrome.rs:46`) already drives a real `NSOpenPanel`, wired into
     File → Open at `src/app/menu.rs:56`, and `src/mas.rs:321` describes its own
     folder picker as "`pick_file_to_open`'s exact shape, folder-only". **So the
     objc2/AppKit modal seam, its live-only harness caveat and its unit-testable
     split all already ship.** `NSSavePanel` is the sibling of a pattern in the
     tree today. ⚠️ **This materially weakens the parked rationale.** The board
     parks "Export save-dialog scope: macOS + Linux, one live-only cross-platform
     seam" as decided-not-scheduled — but **half that seam is built and shipping**,
     and the cost estimate the parking rested on should be re-derived rather than
     inherited. **Unpark it or re-park it deliberately; do not let it sit on a
     premise that has changed.**

     ✅ **AND THE LINUX HALF IS ALREADY ANSWERED — THERE IS NO CROSS-PLATFORM
     SEAM TO BUILD.** Verified: on Linux, File → Open does NOT use a system
     dialog at all — `src/app/menu.rs:49` routes `awl.open` to
     `Action::OpenBrowse`, awl's OWN in-app browser, and redirects to
     `NSOpenPanel` on macOS alone as "the macOS convention". **So the platform
     split already exists and is deliberate, and export should mirror it:
     `NSSavePanel` on macOS, awl's own picker on Linux.**

     ⚠️ **DO NOT REACH FOR AN XDG PORTAL.** `org.freedesktop.portal.FileChooser`
     over D-Bus is the modern Linux mechanism and is the wrong answer here on
     three counts: `rfd`'s default Linux backend links **GTK**, which this tree
     deliberately avoids (Cargo.toml drops muda's `gtk`/`libxdo` defaults on
     purpose); a portal is a **runtime service**, so on a minimal WM without one
     installed the dialog fails outright, which a self-contained tarball cannot
     accept; and it would make export the ONLY verb on Linux using a system
     chooser while Open uses awl's. **Recorded so it is not proposed as the
     obvious answer** — it is the obvious answer, and it is wrong for this tree.

     **Build:** route `Effect::Export` through `NSSavePanel` on macOS, defaulting
     to the document's own folder and name; on Linux reuse the existing in-app
     browse picker in a save role. `--screenshot`/`--keys` keep taking an
     explicit path, exactly as the open picker is already bypassed headlessly.
     **This subsumes item 295(c)** — a chosen destination cannot be a surprise —
     but NOT 295(a) or (b), which are defects regardless.

     ✅ **BOTH OPEN QUESTIONS ARE DECIDED BY THE USER, 2026-08-06 — this item
     carries no remaining design calls.**
     **(1) "See what it looks like" means REVEAL THE FILE AFTER EXPORT** ("yeah
     exactly"), using `NSWorkspace`, which `src/mac_chrome.rs` already imports.
     🔴 **An in-app preview of the rendered PDF is CLOSED, not deferred** — it
     would be a second document renderer, which CLAUDE.md's "infrastructure
     complexity is a smell" forbids and which awl has deliberately avoided
     elsewhere. **Do not re-propose it as a follow-up.**
     **(2) The platform split is confirmed** ("sounds good"): `NSSavePanel` on
     macOS, awl's own in-app picker on Linux. No portal, no GTK, no new seam.

     **Done:** a user chooses where the file goes, and can see the file
     afterwards without knowing where awl would have put it. Together with the
     save panel this retires 295(c) entirely — a chosen destination cannot be a
     surprise, and a revealed file cannot be lost. **Verify:** the panel and the
     reveal are BOTH live-only — flagged for human confirmation, never claimed
     from a capture, since `NSWorkspace` and a modal are exactly the AppKit
     chrome `mac_chrome.rs` already documents as beyond the harness. The headless
     path keeps its explicit `--screenshot`/`--keys` route and stays
     deterministic and byte-identical. **Routing:** deep tier.

302. **LOOSE COMMENTS — A SECOND PASS, AND A DIFFERENT DEFECT CLASS FROM 275's.**
     **User-requested 2026-08-06 after a comment misled the orchestrator into
     relaying a false open decision.** Item 275 removed comments that narrated
     HISTORY; 287/288 removed CITATIONS. **This is neither: it is comments whose
     factual content has drifted from, or was never precise about, the code they
     describe.** A history comment is merely noise. **A loose comment is read as
     truth and acted on.**

     **FIVE INSTANCES, ALL FOUND BY ACCIDENT IN ONE DAY, which is the argument
     for looking on purpose:**
     - `render/chrome/outline.rs` says "the CRISP theme/caret/history pickers".
       **The crisp set is `Theme | Caret`** (`app/viewstate.rs:165`); History is
       deliberately excluded WITH a stated reason. The prose is simply wrong, and
       it cost a wrong decision relayed to the user.
     - `render/chrome/readout.rs`'s `prepare_notice`: "(today: the autosave
       external-change guard's…)" and "every capture (which can never have a
       notice — autosave is live-only)". **~10 callers now, and export is
       reachable headlessly.**
     - `app/theme_font_debounce.rs`: "12.0 ms on CLAUDE.md" — **a measurement
       against a fixture that has since grown 44%.**
     - `docs/platform.md:88`: the location fallback glossed as "`~/notes` by
       default", **wrong for the unconfigured case**.
     - `theme/model.rs`'s `LocationStyle::RotatedRail`: "small", "muted", "flush
       with the card's own left border" — **all three overturned by item 297.**

     **FOUR SHAPES, and the last is the dangerous one.** (a) **stale
     enumerations** — a comment lists members and the roster moved; (b) **stale
     "today" claims** — "X is the only caller" and it is not; (c) **baked
     measurements** — a number measured once against something that moves;
     (d) **invariants later code invalidated** — "every capture can never have a
     notice". ⚠️ **(d) is not bad prose, it is a LOAD-BEARING assumption: the
     capture pipeline was DESIGNED around that sentence, and the design outlived
     its truth.** Hunt (d) first.

     ✅ **THE LEVER, AND IT IS THE POINT OF THE ITEM: A COMMENT THAT STATES A
     CHECKABLE FACT SHOULD BE A LAW, NOT A COMMENT.** "The crisp set is
     Theme|Caret", "no capture carries a notice", "this is the only caller" are
     all assertions a test can hold and prose cannot. **Prefer converting such a
     comment into a law over rewording it** — a reworded comment rots again on
     the same schedule; a law fails the day it stops being true. Where a fact is
     genuinely not checkable, say so in the comment rather than stating it flatly.

     ⚠️ **METHOD — THERE IS NO GREP FOR "WRONG".** This must be READ, one comment
     at a time, exactly as 275 was; the five above are a scale estimate and a
     shape guide, **never a worklist**. Prioritise comments that make claims
     about OTHER modules (a comment describing its own three lines rarely
     misleads; one describing a roster, a caller set or an invariant elsewhere
     is how all five of these went wrong). ⚠️ **Do not change code, except to add
     laws.** ⚠️ **Schedule against a quiet tree** — 275 touched ~1000 sites and
     conflicted with everything; this pass has the same blast radius.

     **Done:** no comment asserts a roster, a caller set or an invariant that the
     code contradicts, and the checkable ones are held by tests. **Verify:** each
     new law mutation-proved by breaking the fact and watching it go red — a law
     asserting a comment is worthless if it passes either way. **Routing:**
     production tier, read one at a time.

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
