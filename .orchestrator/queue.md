# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residuals 1–2 LANDED; residual 3 🟠 AWAITING USER CHOICE — prototype evidence landed)

**Everything except residual 3 is LANDED on `main`** — the capture door, the
`WorkingSet` module, resting-stack render, sidecar exposure +
click-to-switch, ⌘W/close-zone removal through one owner, the hover-reveal
close affordance with folder heading, and the honest zero-document state.
Full sha list: `git log --grep 'item 444'`; design history and the landed
residuals' detail: `git log -p -- .orchestrator/queue.md`.

**Two smaller findings, still carried forward:**
- `gutter` in the sidecar (the single name/project fact) is stale
  *documentation* once the stack draws at N≥2 — its doc claims "exactly as
  drawn" but the pixels show a whole stack. Whoever next touches sidecar
  `buffers`/`gutter` reconciles the doc, or the field, deliberately.
- The scratch buffer is closeable (dirty scratch refuses like any other
  entry, the successor search skips it) but still has no *activation* door
  anywhere (`load_path` takes a path; `previous_path()` returns
  `Option<PathBuf>`) — a scratch row still silently swallows a switch click.
  Feeds residual 2 (zero-document), which needs the same "no path yet"
  reasoning anyway.

**Two facts worth knowing before the next residual touches this area:**
`Finish file` is now a misnomer for a command that CLOSES — renaming it
touches the palette, GUIDE, REFERENCE and the `finish_file` config key, so
only its *description* was corrected this round, not its name. And
`load_path` flushes autosave before parking, so under the default config a
parked entry is essentially never dirty — the parked-conflict path is mainly
reachable via `autosave = false`.

**Open: residual 3 — overflow windowing, expanded/grouped cross-project
view, Move navigator. 🟠 AWAITING USER CHOICE.** The capture-only audition
and its laws are landed and full-gate green. No overflow interaction or Move
action was shipped. The remaining call is whether five rows plus one exact
count at rest expands into the grouped eight-row view, and whether Move
permanently shows `Move here` and `New folder…`. (Residual 1's prototype
gallery is preserved untracked at `gallery/item-444-affordance-prototypes/`.)

**QUEUED NEXT ACTION (not yet dispatched):** before either call above can be
made, build a throwaway prototype of the overflow row and its expanded view
— enough real rendering to demonstrate the windowing rule (which files sit
in the resting five, how the window holds steady as the active file
changes) and the cross-project grouping, captured to screenshots via
`--screenshot-app` against a seeded fixture (multiple roots, >5 open files),
mirroring `captures/item-444/shoot.sh`'s hermetic pattern. Move navigator is
a separate, still-untouched sub-scope — out of this prototype pass unless
the user asks for it too. Put the shots in front of the user before writing
any production windowing/grouping code.

Move stays deliberately bounded to the source file's owning root. Its summoned
folders-only navigator says `move <filename>`, shows the current root-relative
destination, descends/ascends through folders, offers an explicit `New folder…`
row and a `Move here` action at every level. A successful move keeps the stack
slot stable and updates its quiet parent path. No drag-to-move, bulk selection,
folder moves or cross-root moves in this item: those are file-manager machinery,
and a tiny contextual stack is the wrong place to imply them. Moving never
silently rewrites Markdown or incoming links; when the file contains relative
links/images, the completion feedback states that their paths may need review.

Visible overflow is bounded independently of the registry's safety cap.
PROTOTYPE five file rows plus a quiet `+ N more…` row. Accepting it EXPANDS the
same bottom-anchored stack UPWARD into a transient scrollable working-set view;
it does not detour through Go to, permanently lengthen the resting margin, or
turn the whole window into a sidebar. Wheel/trackpad motion over the expanded
list scrolls that list, Esc/click-away collapses it, and choosing a file returns
to the resting stack. The active file must always be represented in the visible
five; prototype the least-jittering stable-order window and put the >5 shots to
the user before fixing its exact windowing rule. Do not silently evict a sixth
file merely to make the drawing easy: the existing registry's
clean-LRU/never-dirty eviction is a memory safety bound, not a visible-stack
product rule.

PROTOTYPE the cross-project half in that SAME expanded view, taking the useful
part of Codex's sidebar grammar without adopting its permanent project-manager
shell: group retained open files under folder headings; mark the active file's
group clearly; keep only the active folder's group in the resting stack. The
ONE generic `+ N more…` row counts every hidden open buffer — same-root overflow
and other roots alike — and expands this grouped view; never add a parallel
`N files in other folders…` row. Clicking a file in another group atomically
restores its remembered project root AND activates its buffer, after which Go
to, New, Move, export and the resting stack all operate in that folder. This is
why the main folder remains meaningful: it is the ACTIVE group, not a claim that
no other folder may retain buffers. Do not show multiple groups persistently;
awl is not a project manager.

No new digit shortcuts. Once the working set can be grouped, partially hidden
and scrolled, “the third file” has no stable, obvious meaning across resting and
expanded states. ⌃Tab Last file stays exactly as shipped, `C-x b` may remain its
quiet Emacs alias, and Go to… remains the complete keyboard route.

Contract edits owned here: DESIGN §5's margin roster gains the stack as a
member with the outline's own license (may click-to-switch; orientation, not
management UI). PHILOSOPHY §1 is untouched — this is not a strip.

Harness reach for whatever ships from residual 3
(docs/harness-reach.md read for this clause): the working set is App-owned —
tier-1 captures classify it Unsupported — so every claim drives
`--screenshot-app`, hermetic against seeded roots only (the margin
photographs filenames, so ambient roots would leak paths). Verify, for the
still-unbuilt parts: overflow keeps the active file represented and its
`+ N more…` count exact; expanded scroll remains inside the working set;
cross-root activate restores the matching project/root before the frame and
the gutter never names the old root; context-menu Move and palette Move
dispatch the same action; moving a nested file keeps its stack slot and
updates its relative label, and never crosses the source root. Generated
reference rows are spot-checked against the dispatch they claim.

### 468 — Firetail palette edges regressed (USER-REPORTED 2026-08-22, live screenshot)

In Firetail, the command palette's row plates now show strange angular
notched/jagged edges on every label — the user's words: "all of the edges
look so weird". Hypothesis, unverified: the Cassowary docked-console corner
treatment (landed this round) leaks into Firetail's plates or applies
per-row where it should not — but the report is a hypothesis; first
reproduce with a `--screenshot` palette capture in Firetail and locate the
first bad commit before assigning a mechanism. Themes are data through one
renderer (RenderCaps): a per-world personality must be expressed as caps,
never a code path that bleeds across worlds. Standing policy applies twice:
user-reported bug → audit the neighborhood (palette plates across the full
world roster, pixel arithmetic per world), and render-touching → vision
smoke. Verify: a law pinning Firetail's plate edge treatment to its authored
caps, mutation-proven by re-enabling the leak; the audit ends by writing
whatever law let this ship silently.

### 469 — margin identity reading order flips between one file and the stack (USER-REPORTED 2026-08-22, live screenshots)

With one file open the bottom-left identity reads filename over folder
(`awl-start.md` above `notes`). At N≥2 the stack reads the opposite way:
folder heading on top, file rows beneath (`notes`, then `awl-start.md`,
then the active `awl-ramblings.md`). Same surface, inverted reading order
depending on state, and the folder line jumps sides of the filename when a
second file opens. The user finds it weird ("we didn't make the project
folder above the notes folder usually"). Options: (a) one-file case adopts
the stack's order — folder heading above the filename — making the N=1→N=2
transition pure row insertion; this consciously retires the N=1
byte-identical-to-today law (it was a construction guarantee, not a product
promise); (b) keep N=1 as shipped and accept the flip. Recommendation: (a),
for one consistent grammar; it is a small, cheap-to-revert render change,
so per the standing preference land it for judgement and say what reverting
costs. Verify: `--screenshot-app` captures at N=1 and N=2 across a few
worlds assert the folder line's position is the same in both states; the
retired identity law is replaced, not silently deleted.

Extension (USER 2026-08-22): once the one-file view adopts the stack
layout, the single row also carries the hover-close mark — the pointer
route to closing the last file, landing on the zero-document start surface
exactly as ⌘W already does. Pointer and keyboard dispatch the same close
owner (`App::close_buffer`), no new machinery; the lossless save/conflict
gate is unchanged. Verify: a `--screenshot-app` law closes the sole file
via the row's close zone and asserts the same zero-document state the ⌘W
law already pins.

### 471 — Cassowary console: square top, chamfered bottom, one corner-mask owner (USER DECISION 2026-08-22)

User-reported on the live app: the Cassowary console shows square artifacts
at its chamfered top corners — the new console layers (panel material,
placard bleed, scanlines, docked facet plates) hold independent opinions
about the corner; `overlay_material.rs` already sets the placard's chamfer
to 0.0 while the panel takes one card's mask. Quokka's palette proves the
shared `CardShape::Chamfered` pipeline clips fill correctly with one layer.

Decision: Cassowary's console goes SQUARE at the top corners (the docked
seam edge, where the facet strip lives) and keeps the chamfer at the
bottom (the free edge). Quokka stays all-four-corners, untouched.
Mechanism: `CardShape` grows a per-corner (or top/bottom) axis as theme
data — one renderer, no world-specific code path — and every console layer
clips through the single corner-mask owner; the bypass goes module-private.

New axis value ⇒ standing-policy probe across the full overlay surface
roster (palette, placard, query bar, toast…) in both authoring worlds, plus
the render-touching vision smoke. Verify: pixel arithmetic over the corner
triangles — top corners match the panel ground exactly where square, bottom
corners show the cut — mutation-proven by re-squaring one layer and by
desyncing one layer's mask; Quokka byte-identical before/after.

Deliberately out of scope: the reference mockup's tab-on-border and left
rail ticks — a separate taste call, easier once the mask has one owner.

### 472 — bare-plate picker legibility: footer gap magnitude (USER-REPORTED 2026-08-22 on Firetail, live screenshot)

Backdrop half LANDED: `Bars` (Firetail, Galah, Kite) now enrols in the
footprint frost alongside `Diagonal`/`Ruled` — `blur::footprint_frost_applies`
no longer excludes it, `TextPipeline::overlay_drawn_surfaces` contributes the
row band's full width (a plate hugs its own label, narrower than the row's
own clickable span, so the declared surface is the BAND, matching the term
`Ruled`'s rule spans already contribute — not the per-row hug list). Three
pixel-arithmetic oracles (`frost_context`, `frost_footprint`'s edge and hue
laws) were blind to `Bars` plates/scrims authored close to the world's own
ground (their edge gradient in the empty-document frame can sit under
`CardInk`'s derived `INK_GRADIENT`); `TextPipeline::overlay_row_ink_probe`
(the production ink owner) plus a shared `row_ink_vetoes` dilation is now
the geometric backstop, mirrored in `frost_context::Pair` and
`frosted_and_live_mean_lab`. Verified: headless captures on Firetail (dark)
and Galah (light) show legible plates over a genuinely blurred document —
no sharp interleaved text — confirmed against Paperbark (`Ruled`, already
enrolled, unchanged). The predicate change also enrols the pointer-anchored
CONTEXT MENU on `Bars` worlds (`frost_context` now proves it there too),
which the original report didn't name but the same legibility rationale
covers. `Kite` is a sixth `BarePlates` world the item predates; the
roster-derived sweeps (`enrolled_worlds()`) already cover it.

**Still owed — the footer gap.** Confirmed present on ALL THREE captures
above (Firetail, Galah, Paperbark): the hint strip sits flush under the
last row, and the document's display-size heading (`THEMES`) reaches the
hint strip's own line. The backdrop fix does not move it — it is a
magnitude question on `OVERLAY_HINT_GAP_ROW` (`render/chrome/overlay_policy.rs`),
shared by every overlay family on every world, so any bump is a separate,
narrowly reviewable commit. Verify: a minimum gap between the last row and
the hint strip swept across all `BarePlates` worlds and row counts, plus a
legibility floor for row labels over the busiest ground (display-heading
behind the rows), mutation-proven both ways.

### 473 — clippy debt: the slice worth paying (USER 2026-08-22)

Ledger measured 2026-08-22: 92 recorded exceptions (84 `too_many_lines` —
50 of them test-file law tables, which stay; 8 `cognitive_complexity`) and
96 inline allows (70 `too_many_arguments`, 14 `type_complexity`). NOT a
bulk lint-silencing pass — the valuable work is three bounded cuts:

1. **`too_many_arguments` where params are same-typed and adjacent** (the
   70 allows cluster in render chrome, caret/pipeline, main/run): a
   transposed `f32, f32, f32` argument compiles clean and renders wrong,
   so this is a real defect class, not cosmetics. Where ≥3 functions pass
   the same parameter tail, extract one context struct and route them
   through it (same behavior ⇒ same code); leave the deliberate mirrors
   ("mirrors capture_screenshot's own surface") alone.
2. **The 14 `type_complexity` allows** — named type aliases, mechanical.
3. **Read the 8 `cognitive_complexity` functions** and decompose only
   where a seam is honest; an exception with a good reason stays.

Constraint: behavior-identical refactors only, and identity is proven —
targeted tests per touched seam, full gate at landing; any site whose
refactor would change an observable is reported, not "fixed". Exceptions
retired as their sites are fixed; `code-health.toml` edits are
orchestrator-owned at merge. Staleness needs no sweep — the health script
already flags stale exception messages by name.

## Needs specific hardware

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
