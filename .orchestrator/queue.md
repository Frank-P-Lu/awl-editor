# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residuals 1–2 LANDED; residual 3 windowing + scroll DECIDED USER 2026-08-25, ready to build; Move rows question still open)

Residual 3's gallery is built and landed at `captures/item-444-residual3/`
(fixture, `shoot.sh`, README with rationale) — collapsed/expanded/grouped
states across several worlds. The agent found the render layer needed zero
changes. The user judged the gallery 2026-08-25 and decided both UX
questions it posed (recorded in "Residual 3 decisions" below).

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

**Residual 3 decisions (USER 2026-08-25, from the gallery):**
- **Ordering law:** open order seeds the stack; activating a file NEVER
  reorders it. (Already the built behavior; now pinned as the user's rule,
  and item 484's drag-to-reorder makes a drag the ONE order-changing
  gesture.)
- **Resting window rule — hold still, slide minimally.** The gallery's
  stateless candidate (window re-derived from the active index alone) was
  rejected on its own `collapsed-jitter.png` evidence: activating a file
  already visible slid the window four slots. The shipped rule is
  stateful: when the newly active file is already inside the visible
  window, the window does not move; when it is outside, the window slides
  the minimum distance that reveals it.
- **Expanded scroll — reveal on open, never clamp during scroll.** The
  browser-tab convention: the expanded view opens scrolled so the active
  row is visible, and any activation re-reveals it, but a user's own
  wheel/trackpad scroll is never fought — the active row may scroll out
  and nothing pulls it back.

**Ready to build:** overflow row interaction (`+ N more…` click/hit-test),
the expanded scrollable view, and the grouped cross-project view, per the
spec paragraphs below under the three decisions above. **Still open, NOT
decided:** the Move navigator sub-scope (including whether it permanently
shows `Move here` and `New folder…`) — untouched by the prototype pass and
awaiting its own round. (Residual 1's prototype gallery is preserved
untracked at `gallery/item-444-affordance-prototypes/`.)

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
five; the windowing and scroll rules are the DECIDED ones above (hold-still
minimal-slide window; reveal-on-open, unclamped scroll). Do not silently evict a sixth
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
`+ N more…` count exact; the hold-still law — activating a file already
inside the visible window leaves every drawn row in place (the exact
`collapsed-jitter.png` sequence, asserted by sidecar row identity), and
activating one outside slides the window by the minimum; the expanded view
opens with the active row visible, a scripted scroll moves it off-screen
with no clamp, and a subsequent activation re-reveals it; expanded scroll
remains inside the working set;
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
### 483 — hover grammar for clickable chrome (DECIDED: option (c), USER 2026-08-25 — ready to build)

**USER DECISION 2026-08-25: option (c)** — hover feedback only where the
ambiguity is real: the format popover's buttons (which one a click fires)
and the inline-image resize handles (that they exist at all). Every other
clickable surface stays still; hover elsewhere remains an overlay-only
gesture. The decision becomes a DESIGN.md sentence, and the popover's
tile-to-the-edges hit regions are re-judged as part of the popover half.
Verify: the popover's hovered button carries a visible acknowledgement
asserted by pixel arithmetic (presence floor, both themes' worlds
sampled), the non-hovered buttons do not; an image border hover draws its
handle affordance; and a sweep asserts the OTHER seven rosters' surfaces
draw NO new hover state (the calm is the law too).

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

---
### 484 — drag-to-reorder the working-set stack rows (USER 2026-08-25)

The user wants the open-file rows in the margin stack reorderable by
pointer drag, the way browser tabs reorder. This makes the stack's order
USER-OWNED, which locks in item 444 residual 3's ordering law from the
other side: open order seeds the list, activation NEVER reorders, and
now a drag is the ONE gesture that changes order — so no windowing or
activation rule may shuffle rows behind the user's back.

Scope: press-and-drag on a `File` stack row lifts it and drops it at a
new position within its own root's group, in both the resting stack and
the (once shipped) expanded view; a quiet insertion indicator marks the
drop slot while dragging. In-group only — a row never drags across a
group heading (cross-root movement stays the Switch-project /
activation route, and moving the FILE between folders stays item 444's
Move navigator; this item moves rows, not files). The reordered
sequence is the same order every consumer reads — resting window,
expanded view, grouped view, session restore — one owner, no parallel
list. Drag vs click disambiguation follows the platform threshold (a
sloppy click must still switch); the close-zone press is not a drag
handle.

Depends on 444 residual 3's windowing choice only for which rows are
VISIBLE — the reorder mechanism itself does not. Working set is
App-owned, so verification drives `--screenshot-app` against a seeded
multi-file fixture (`captures/item-444-residual3/fixture` pattern):
sidecar asserts the post-drop order, its persistence across a
restart-with-session-restore, and that a drag attempt across a heading
lands clamped inside the group. Reorder feel over real time is
live-only — flag for human confirmation.

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
