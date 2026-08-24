# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residuals 1–2 LANDED; residual 3 🟠 AWAITING USER CHOICE — prototype evidence landed)

Residual 3's gallery is built and landed at `captures/item-444-residual3/`
(fixture, `shoot.sh`, README with rationale) — collapsed/expanded/grouped
states across several worlds. The agent found the render layer needed zero
changes; the open question is a stateless-jitter UX call for the user to
make from the gallery, not further engineering.

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

### 475 — the fold mark becomes a font glyph: per-world symbols via rotated_label (USER DECISION 2026-08-23)

Symbol survey is built and landed at `captures/item-475-glyph-survey/`
(`shoot.sh`, README with candidate rationale, dropped leads with coverage
evidence, licensing notes). The agent checked candidate coverage by parsing
the actual bundled font files rather than trusting Unicode charts — the
item's own named glyph lead has zero real font coverage and was dropped.
🟠 AWAITING USER CHOICE — the symbol pick is next, from the gallery.

The fold chevron is four rotated quads (`selection::chevron_arms` →
`prepare_rotated`) because glyphon 0.11 has no transforms — and it reads
fat and world-generic (user, on Bowerbird). It WAS a real glyph (U+203A)
before the quarter-turn animation forced geometry. The mechanism that
reconciles the two now exists: `rotated_label/` rasterizes a shaped run
into an R8 coverage mask and rotates the quad, and is world-neutral by
construction — which symbol, which ink is caller-supplied theme data.

Decision: the fold mark becomes a font glyph, chosen PER WORLD as
`RenderCaps` data (`fold_afford` grows a mark spec — the bullets pattern,
one renderer, no world code paths), drawn through `rotated_label` so the
turn survives. Coverage home is `AwlMarks.ttf`, awl's own OFL-composed
symbol face: chosen symbols are composed IN (Noto Sans Symbols 2 / Noto
JP are already-licensed sources — user suggests Japanese forms are worth
mining: 〈 U+3008 family, vertical-form ﹀/︿, CJK single kakko), making
never-tofu a subsetting fact, not a per-world ladder gamble.

**Symbol picking comes FIRST and is the user's taste call**: survey
candidate glyphs by parsing the actual bundled faces (ask the fonts, not
the Unicode charts), then land a capture gallery — candidates × light/dark
worlds × H1/H3, collapsed and expanded — for the user to pick from. No
mark ships unpicked.

Constraints that carry over as the spec: the direction-at-rest law (a
collapsed and an expanded mark must differ with zero animation frames),
Reduce Motion's instant settle, and the full `fold_chevron_center.rs` law
set — ladder scaling per heading, pad clamp, hit-box enclosure, mixed
two-level batches — stays green or is consciously rewritten at the same
seams. Cheap fallback if the glyph route stalls on taste: `STROKE_CHARS`
(or a per-world `fold_afford` weight) thins today's quads in one line.

Related, landed for judgement (`render: fold chevron rides its heading's
ladder step`): the mark + gap now scale with the heading's Ladder J step.
🔵 OWED: user verdict on the scaled sizes and gap; reverting is one
commit.

---
### 476 — Cassowary palette dead band: ~3.5 rows of empty card between query and first item (USER-REPORTED 2026-08-24, live screenshot; reproduced headlessly)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

Every lens of the Cmd-P palette on Cassowary shows a blank band between
the query row and the first item, with the list reading bottom-anchored.
Measured (ordinary capture, `--theme Cassowary --keys "Cmd-p Right Right
Right"`; sidecar + code, not the report): three stacked charges.

1. **The vacated strip row (1.0 lh).** `theme_overlay_geometry` hardcodes
   `header_rows = 2` (`chrome/theme_picker.rs`), but under
   `FacetStyle::DockedTab` the strip draws ABOVE the card
   (`docked_facet_band`) and `push_docked_facet_areas` clips the in-card
   strip band out of the panel buffer — the row is billed and never drawn.
2. **A blank Location line (1.0 lh).** The plan's first display line is
   `PlanLine::Location`; the shaper draws it only when
   `location_style.draws_inline()` (`theme_picker/spans.rs`). Cassowary is
   `RotatedRail`, which draws the location on the edge rail — the planned
   line renders as nothing. The `draws_inline` gate lives at the shaper,
   not the planner.
3. **The query beat (1.55 lh).** `OVERLAY_QUERY_BEAT`
   (`chrome/overlay_policy.rs`) — air between plates on bare-plate worlds
   (Firetail is the control: every display line occupied), a hole inside
   one unified pane.

Fix direction: make `header_rows` facet-style-aware; gate the Location
line at the planner; taste-pass the beat for unified panes. Verify: pixel
+ sidecar arithmetic that the first item row sits within one beat of the
query row, swept across every lens and the DockedTab/RotatedRail roster
(derive enrolment from the roster), with a presence floor on the query row
itself; mutation — reinstate the flat `header_rows = 2` and watch it go
red.

---
### 477 — docked facets become real tabs on the console's top border (USER 2026-08-24; promotes item 471's named deferral)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

Item 471 deliberately parked "the reference mockup's tab-on-border" as a
separate taste call; the user has now made it: the category strip is
"meant to be tabs". Today `FacetStyle::DockedTab` seats the strip line
above the card with the active label on a filled box — it never reads as
a tab attached to the border.

Scope: the ACTIVE facet becomes a tab joined to the card's top edge — its
plate continuous with the card ground across the seam, no border stroke
through the tab's mouth; inactive facets stay quiet strip text. Depends
on item 471's single corner-mask owner landing first (the tabs sit on the
exact edge 471 squares); 471's other deferral, the left rail ticks, stays
parked — not promoted here.

Mechanism stays themes-as-data: the tab is drawn by the one docked-facet
owner reading theme tokens, no world-specific code path. Hit-test and
drawn geometry keep one owner (`overlay_lens_at` reads the same shaped
strip the dock moves).

Verify: pixel arithmetic at the seam — on the active facet the ink column
crossing the border row is continuous with the card ground; on inactive
facets and between tabs the border reads intact; swept across the
DockedTab roster and both a first and a non-first active facet (the dock
buffer once dropped the strip tail on exactly that axis). Cheap to
revert, so per standing preference: land for judgement.

---
### 478 — the palette query renders as a text field but is not one (UI-AFFORDANCE SWEEP 2026-08-24, code-verified)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

The overlay query line earns the I-beam (`cursor_shape.rs` via
`over_overlay_query`), but a click inside the card off a row is
deliberately swallowed (`app/input/mouse.rs`, overlay_click's
inside-the-card arm) — no caret placement, no selection, no drag-select.
And caret motion in the query is word-only: ForwardWord/BackwardWord
reach `query_word_right/left`, while ForwardChar/BackwardChar cycle the
lens on faceting pickers and LineStart/LineEnd jump the SELECTION to
first/last row (`actions/overlay_nav.rs`). Once a word-step lands
mid-query there is no char step and no End; the rename/link/keep/value
sub-editors already bind full char motion, so the mechanism exists.

Against DESIGN.md's "a surface that looks clickable must be clickable
where it is drawn". Small design decision inside the item: what char
keys and Home/End do while the caret sits mid-query vs at the end (the
list-nav overloads are load-bearing and stay when the caret is at rest
at the end — state the chosen rule in the item's landing note).

Verify: `--keys` replay laws — click-to-place via the pointer path,
char motion round-trips, Home/End behaviour on both sides of the chosen
rule; sidecar asserts query caret position (extend the schema if it is
not yet reported). Mutation: re-swallow the query click and watch the
placement law go red.

---
### 479 — hover and cursor truth go stale when content moves under a stationary pointer (UI-AFFORDANCE SWEEP 2026-08-24, code-verified)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

`sync_cursor_icon`, `update_fold_hover` and `resolve_gutter_stack_hover`
are driven by `CursorMoved` plus hand-patched doors (modifiers, overlay
transitions, drag ends). `on_mouse_wheel` ends with no resync — so wheel
scroll, wheel zoom and keyboard scroll leave the OS cursor shape and the
revealed fold chevron pinned to what WAS under the pointer until the
mouse next moves. Lifecycle half: `on_cursor_left` clears the stack
hover but not the chevron hover (its own doc already states the
principle), so a revealed chevron stays lit in an unfocused window;
`on_focus_lost` touches neither hover.

Fix direction is structural, not a fourth door-patch: recompute
pointer-derived state at one frame-level seam whenever geometry or
scroll changed under an unmoved pointer, and clear both hovers on
cursor-left/focus-loss. Watch the cost: the resync must stay O(visible)
and must not fire per frame of an animation without need.

Verify: unit laws at the seam — after a synthetic scroll with an unmoved
pointer, the recomputed `CursorContext` differs where the geometry says
it must; fold/stack hover cleared after cursor-left and focus-loss.
Stale-icon feel over real time is live-only — flag for human
confirmation. Mutation: drop the wheel-path resync, watch the law go
red.

---
### 480 — no autoscroll when a selection drag reaches the viewport edge (UI-AFFORDANCE SWEEP 2026-08-24, code-verified absence)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

Dragging a text selection to the window's edge stops at one screenful:
`on_drag` → `hit_test_scroll` clamps to the visible band, and no
scroll-on-drag exists anywhere (three comments explicitly deny it).
Mouse selection beyond a screen requires the keyboard. Editing
edge-cases are the product — this is the most consequential everyday gap
the sweep found.

Mechanism: while a range drag is live and the pointer sits beyond the
column's top/bottom band, scroll at a rate derived from overshoot and
extend the selection to the hit-test under the moving content — through
the existing drag owner, not a parallel path. The headless path has no
clock, so the rate lives behind the same seam live animation already
uses; the settled state (selection extended to the scrolled-to point)
stays deterministic.

Verify: unit laws on the overshoot→rate function and on
selection-extends-under-scroll at the drag owner's seam;
`--screenshot-motion` for the deterministic settled frame. The feel of
the rate over real time is live-only — flag for human confirmation, per
harness-reach discipline. Mutation: clamp the rate to zero and watch the
extension law go red.

---
### 482 — modal summoned cards own the next key and click, but not the wheel; the streaks second page has no pointer route (UI-AFFORDANCE SWEEP 2026-08-24, code-verified)

🟡 IN PROGRESS — claude, branch (dispatched via Agent isolation:worktree, sha to follow)

`dismiss_summoned_card` is consulted from exactly one input path, the
mouse-button press. `on_mouse_wheel` has no card guard — with
About/Lifetime/Streaks up, the wheel scrolls the DOCUMENT behind the
card. Separately, the streaks card's cumulative page is reachable only
by ←/→ intercept, nothing drawn advertises it, and any click — including
on the chart — dismisses the card instead of paging.

Decision folded in: does the wheel DISMISS the card (matching the
any-key/any-click grammar) or is it swallowed? Recommendation: swallow —
a wheel is not an intent the way a press is, and scrolling the hidden
document moves state the user cannot see. For streaks: draw the ←/→
hint on the card and give a click on the card's own surface the paging
(or at minimum non-dismissing) role; click outside still dismisses.

Verify: `--screenshot-app` laws — wheel with a card up leaves the
document scroll byte-identical in the sidecar; streaks paging reachable
by pointer; the hint present by pixel arithmetic (presence floor, not
just contrast). Mutation: remove the wheel guard and watch the
scroll-identity law go red.

---
### 483 — hover grammar for clickable chrome (USER DECISION NEEDED; UI-AFFORDANCE SWEEP 2026-08-24)

🔵 PARKED — needs the user's taste call before any code.

Exactly three surfaces draw hover state today: overlay rows, the fold
chevron, the working-set stack. Seven more are clickable with a
hit-test and NO hover visual: margin outline rows, workspace rail
entries, settings range rails, find/replace panel cells, start-screen
action rows, the drawn (web/Linux) menu bar, format popover buttons.
Sharpest cases: the format popover's hit regions tile the WHOLE card —
padding and inter-button gaps fire the nearest button
(`render/plan/popover.rs::hit`) with no feedback about which — and
inline-image resize handles are invisible (the OS cursor is the entire
affordance), so image resizability is undiscoverable.

The question: which chrome acknowledges the pointer, and how quietly?
Options: (a) hover is an overlay-only gesture, persistent chrome stays
still (today's de-facto rule, minus the chevron/stack exceptions);
(b) one quiet grammar everywhere clickable — the existing hover
treatment class extended to the roster above; (c) grammar only where
ambiguity is real: popover buttons (which one fires) and image handles
(that they exist), rest stays calm. Recommendation: (c) — it fixes the
two genuine ambiguities without making the chrome restless.

Whichever wins becomes a DESIGN.md sentence, and the popover's
tile-to-the-edges hit regions get re-judged under it.

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
