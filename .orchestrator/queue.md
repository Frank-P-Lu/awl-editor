# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residuals 1–3 LANDED; Move rows sub-scope still open)

**Residual 3 LANDED 2026-08-25** (`d38cbe1e`, merge of `item-444-residual3-build`): the
overflow `+ N more…` row, the hold-still/minimal-slide resting window, and the
expanded scrollable/grouped panel — all three per the user's 2026-08-25 gallery
decisions (open order seeds the stack and activation never reorders it; the
window holds still when the newly active file is already visible and slides
the minimum otherwise; the expanded view reveals the active row on open and on
every re-activation but never fights the reader's own scroll). Render layer
needed zero changes; one shared `margin_rows(root)` now feeds both the live App
margin and the `--screenshot-app` capture fold. Full native-gate receipt
(mac+linux+menubar-full) and web-smoke both green. **Owed to a human, live-only
and structurally unreachable from any capture door** (same class as the
pre-existing switch/close click): the actual pointer press on the `+N more…`
row, a file row inside the expanded panel, and the wheel-over-panel gesture —
proven at every seam short of the literal window/pointer. Also owed: the
expanded panel's 8-row viewport size and click-away/Esc feel are judged from
the gallery's numbers, not yet confirmed live in a real window.

**Everything except the Move navigator is now LANDED on `main`.** Full sha
list: `git log --grep 'item 444'`; design history and every landed residual's
detail: `git log -p -- .orchestrator/queue.md`.

**Still open, NOT decided — the Move navigator sub-scope**, untouched by every
prototype pass so far and awaiting its own round (Residual 1's prototype
gallery is preserved untracked at `gallery/item-444-affordance-prototypes/`):

Move stays deliberately bounded to the source file's owning root. Its summoned
folders-only navigator says `move <filename>`, shows the current root-relative
destination, descends/ascends through folders, offers an explicit `New folder…`
row and a `Move here` action at every level, including whether those two rows
show permanently or contextually. A successful move keeps the stack slot
stable and updates its quiet parent path. No drag-to-move, bulk selection,
folder moves or cross-root moves in this item: those are file-manager machinery,
and a tiny contextual stack is the wrong place to imply them. Moving never
silently rewrites Markdown or incoming links; when the file contains relative
links/images, the completion feedback states that their paths may need review.

Verify, once built: context-menu Move and palette Move dispatch the same
action; moving a nested file keeps its stack slot and updates its relative
label, and never crosses the source root. Generated reference rows are
spot-checked against the dispatch they claim.

**Two smaller findings, still carried forward:**
- `gutter` in the sidecar (the single name/project fact) is stale
  *documentation* once the stack draws at N≥2 — its doc claims "exactly as
  drawn" but the pixels show a whole stack. Whoever next touches sidecar
  `buffers`/`gutter` reconciles the doc, or the field, deliberately.
- The scratch buffer is closeable (dirty scratch refuses like any other
  entry, the successor search skips it) but still has no *activation* door
  anywhere (`load_path` takes a path; `previous_path()` returns
  `Option<PathBuf>`) — a scratch row still silently swallows a switch click.

**Two facts worth knowing before the next residual touches this area:**
`Finish file` is now a misnomer for a command that CLOSES — renaming it
touches the palette, GUIDE, REFERENCE and the `finish_file` config key, so
only its *description* was corrected this round, not its name. And
`load_path` flushes autosave before parking, so under the default config a
parked entry is essentially never dirty — the parked-conflict path is mainly
reachable via `autosave = false`.

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
### 487 — Magpie theme-picker composition: query-to-list distance, frost boundaries through legible content, stranded chevron (USER-REPORTED 2026-08-25; diagnosed + candidate gallery LANDED, 🟠 AWAITING USER PICK)

Candidate gallery is built and landed at `captures/item-487-magpie-diagonal/`
(fixture, `shoot.sh`, `measure.py`, README with rationale) — the current
(broken) composition against candidate fixes for all three symptoms below,
across both worlds that actually use `ListStyle::Diagonal` (Magpie, Mangrove
— derived from the roster, not assumed) and both DPIs. Small prototype hooks
in `diagonal/gallery.rs` render the candidates through the real production
pipeline; nothing is wired into the shipped default — the user picks from
the gallery, same as items 444/475.

Repro: `--theme Magpie --keys "Cmd-p t h e m e Ret"` over any document
with a title. Three compounding symptoms — the blur MATH is exonerated
(kernel verified symmetric ±4-tap; rows above the footprint's top face
are byte-identical to the unfrosted frame), so this is boundary
PLACEMENT and composition, not shaders:

1. **Query-to-first-item distance.** The faceted card is wide (sized for
   the lens strip) and Magpie's ascending diagonal list right-aligns, so
   the FIRST item lands top-right — ~900px from the query caret at the
   card's left on a moderate window. The eye has nowhere to rest between
   typing and results; same defect family as Cassowary's dead band
   (landed with the 476-482 wave), one world at a time.
2. **Frost boundaries crossing legible content.** The footprint frost's
   feathered top face crosses the document's H1 (an alpha ramp through
   big glyphs reads as ink MELTING downward); its raking side face
   slices sentences mid-word (half-blurred, half-sharp lines); and over
   the empty left margin the film itself shows as a gray WEDGE bounded
   by the leaning face and the page column edge. On a wide window the
   band leaves most of the document crisply readable beside frosted
   fragments, which reads broken rather than defocused.
3. **The selected-row chevron** (`diagonal.rs::selected_chevron`) sits
   at the window's far-left edge, a full window-width from the
   right-anchored labels it points at — it reads as stranded debris over
   the frosted band.

**Ready once a candidate is picked:** verify pixel arithmetic that no
feather face crosses document glyph rows sharper than the shipped
floor, the query-to-first-item distance bounded, swept across the
Diagonal-world roster, both DPIs.

---
### 491 — the native gate grows a health arm: code-health.sh becomes structurally unskippable (USER 2026-08-25, after repeated red-main repair rounds; renumbered from a colliding 488 — that number was already spent by item 485's query_drag finding)

`scripts/native-gate.sh` contains no reference to code-health;
`scripts/code-health.sh` carries fmt, full clippy `-D warnings`,
cargo-machete, and `code-health.py`'s ratchets (size marks + frozen
baseline, stale exceptions, item-number citation bans). So a lane holds
a full green receipt while its diff is health-red, and the failure
surfaces post-merge on `main` — measured cost this round: several
red-main repairs, including genuine shrinks where marks were capped by
the frozen baseline rather than raisable. README's landing list already
names code health; nothing structural enforces it. The fix is the gate,
not another instruction.

Mechanism: native-gate.sh runs `code-health.sh` as a named arm BEFORE
acquiring the full-width arbiter slot (health is CPU-only; the
serialized GPU window must not grow), on the commit the receipt names,
and the receipt gains a `health=` line — no receipt without it. On
failure the arm's output must carry the policy: a RAISABLE mark failure
says "report the number; the orchestrator raises code-health.toml at
merge" (lanes may not edit the toml), distinct from a hard-ceiling
failure (500-line/baseline — shrink is the only remedy);
`code-health.py`'s messages already distinguish the tiers, so this is
labeling, not new analysis. The gate also asserts `git status --short`
clean before starting — closing the documented dirty-tree-receipt and
unstaged-toml hazards in the same move. CI is untouched (linux already
runs the script); this closes the LOCAL lane gap only.

Verify: a receipt from the new gate names the health arm; a seeded
worktree with a health-red diff gets no receipt and the mark-vs-ceiling
wording matches the failure class (drive both classes); the arbiter
marker's hold window is measured unchanged; `test-native-gate.sh`'s own
law set gains the arm. Record the added wall-clock in the landing note —
worker briefs' timeout guidance depends on it.

### 492 — one list-marker grammar: Enter-continuation routes through `markdown::list_item` (found by the 2026-08-25 duplication census, verified)

`markdown::list_item` (`src/markdown/spans/detect.rs:147`) is the documented
SHARED list-detection primitive — the Tab/Shift-Tab indent gate calls it
(`src/actions/edit.rs:188`). But `smart_newline_for` (`src/actions/edit.rs:235`),
the Enter-continuation, hand-rolls the same bullet/ordered grammar instead of
calling it, and the copies PROVABLY diverge: `list_item`'s indent loop accepts
spaces only, `smart_newline_for`'s accepts spaces or tabs. So on a tab-indented
list line, Enter continues the list while Tab-to-indent silently no-ops —
two commands disagreeing about whether the same line is a list item. And
`list_item`'s own doc comment claims a shared-primitive status that is false
at this call site.

Fix: decide the tab contract once (recommend: `list_item` accepts tabs in its
indent skip, matching CommonMark's tab-as-indent), then route
`smart_newline_for`'s marker detection through `list_item` — the checkbox
suffix and blockquote handling stay layered on top in `edit.rs`. Delete the
duplicate grammar.

Verify: a parity law over a line corpus (tab-indented bullets/ordered/task
items, near-misses like bare `-` and `12 monkeys`) asserting the two
consumers agree on is-list for every line; a `--keys` test driving Tab on a
tab-indented list line asserting it indents rather than soft-tabs. Prove the
law non-vacuous by re-introducing the spaces-only loop and watching it go red.

---
### 493 — retire the stray sRGB EOTF copy in `theme::derive`, and harden the law that missed it (found by the 2026-08-25 duplication census, verified)

`src/theme/derive.rs:169-179` (`rel_lum`/`contrast_ratio` — live runtime
inputs to `placard_stipple_density` and `selected_row_ink`) reimplements the
sRGB EOTF locally with breakpoint `0.03928` (the pre-2017 WCAG draft value)
instead of routing through the canonical owner
`theme::color::srgb_channel_to_linear_f32` (breakpoint `0.04045`). The
dedicated anti-copy law `srgb_eotf_law.rs` scans for the literal text
`"0.04045"` (its `NEEDLE` const) — so a copy that retypes the WRONG constant
is structurally invisible to it, a blind spot the law's own doc comment
already concedes.

Fix: replace `rel_lum`'s inner linearize with
`srgb_channel_to_linear_f32`, delete the local curve; then widen the law's
needle set so a retyped breakpoint is caught too (scan for `0.03928`
outright, plus the curve's other constants `12.92`/`1.055`/`2.4` co-occurring
outside the owner — pick the formulation that stays false-positive-free
against the existing tree).

Verify: the law goes red on the pre-fix tree (proves the hardened needle
catches this exact instance), green after; the affected runtime outputs
(stipple density, selected-row ink picks) are compared before/after across
the world roster — any world whose pick CHANGES is named in the landing note
for a taste glance, since the old curve was numerically wrong but shipped.

---
### 494 — fence-line detection gets one owner: spellcheck and the code-block toggle miss `~~~` fences (found by the 2026-08-25 duplication census, verified)

`markdown::spans::detect::fence_line_lang` (`src/markdown/spans/detect.rs:8`)
is the authoritative fence recognizer: up to 3 leading indent spaces, a run
of 3+ backticks OR tildes. Two production sites substitute a naive
`trim_start().starts_with("```")`: `src/spell.rs:477` (the fence-skip toggle
in `misspelled_spans`) and `src/actions/format.rs:154` (`is_fence`, the
code-block-toggle's already-fenced judgment). Both miss `~~~` fences the
renderer treats as real — so spellcheck squiggles inside a tilde-fenced
block's body, and the toggle misjudges a tilde-fenced selection. Neither
copy honors the 3-space indent allowance either.

Fix: factor the prefix logic out of `fence_line_lang` into a small
`is_fence_line(&str) -> bool` in the same module (one owner, the lang gate
layered on top), route both call sites through it, delete the local checks.

Verify: a parity law asserting `is_fence_line` agrees with
`fence_line_lang`'s recognition domain over a corpus (backtick, tilde,
indented, short-run, mid-line); a spell unit test with a `~~~` fence
asserting its body produces zero misspelled spans; a format test toggling a
tilde-fenced selection. Red-first on the pre-fix tree for the tilde cases.

---
### 495 — writing streaks count the words the readout shows (found by the 2026-08-25 duplication census, verified)

`streaks_current_words` (`src/app/streaks.rs:52`) counts via
`markdown::word_count` — plain `split_whitespace().count()` — while its own
comment claims it is "the same `markdown::word_count` the readout / held HUD
use". False: the visible readout goes through `card::figures::words_readout`
→ the CJK-aware `figures::word_count` (`src/card/figures/mod.rs:256`,
`count_tokens(manuscript(..))`, with a pinned CJK regression floor). For
CJK-majority prose the readout reports hundreds of words while the streak
ledger accrues single digits — the streak system silently undercounts
exactly the sessions it exists to honor, and the comment actively misleads
the next reader.

Fix: point `streaks_current_words` at the figures owner (export a shared
alias if visibility needs it), and note that `manuscript()` also strips
frontmatter — a deliberate improvement for streak purposes (typing
frontmatter isn't prose), name it in the landing note. Correct the comment.

Verify: a regression test with a CJK fixture asserting the streak delta
equals the readout's count for the same text; a plain-English fixture
asserting the switch changes nothing there (whitespace tokenization and
count_tokens agree on spaced prose — assert it rather than assume it).

---
### 496 — law enrolment repairs: three sweeps that a roster change silently narrows (found by the 2026-08-25 test-quality census, verified)

Three enrolment holes, same family, one item:

1. `dormant_profile_arms` (`src/render/tests/ground_space.rs:134`) derives
   its Deckle/Zigzag/Organic arms via `if let` on NAMED worlds (Paperbark,
   Quokka, Bowerbird), silently pushing nothing on a mismatch — and Deckle
   and Zigzag each have TWO wearers (`worlds.rs:641/966`, `39/232`), so the
   file's variant-level completeness law stays green if the named wearer
   changes ground while the sibling still covers the variant: the arm
   vanishes from the sweep with zero failure. The adjacent
   `dormant_edge_dots` was already rewritten for exactly this (find_map
   over `THEMES` + `.expect`); apply the same shape to all three arms.
2. Three files hardcode the diagonal pair — `cluster_mirror.rs:44`,
   `settings_row_reach_law.rs:88`, `plan_pass_law.rs:133` — with no tether
   to the live roster. Only `diagonal_composition.rs:211` derives-and-pins;
   a third `ListStyle::Diagonal` world would trip that one file while the
   other three silently keep grading two worlds forever. Extract one shared
   `diagonal_worlds()` helper derived from `THEMES` (asserting today's pair
   exactly, so growth is a conscious edit in ONE place) and route all four
   files through it.
3. Lower confidence, re-verify before building:
   `theme_preview_shape_law.rs:614` states a face-independence claim in
   roster-wide terms but samples 4 worlds covering ~4 of ~14 distinct
   display faces — no mono face is swept. Derive the sample from the
   distinct-`font` roster instead (one world per face suffices).

Verify: each repaired enrolment names what enrolled in its failure message;
prove non-vacuity by breaking one enrolled member per repair and watching
the specific law go red.

---
### 497 — decompose `parse_args` (a 941-line function) and `on_mouse_wheel`'s phases (found by the 2026-08-25 structure census, verified)

`src/main/args.rs::parse_args` is 941 lines of the file's 974 — the largest
function in the tree by ~3× — with already-commented phase seams: the
argument-token loop, CLI validation, hermetic scenario filesystem setup,
config load, sticky-preference resolution, capture-opts assembly, final
`Mode` construction. The code-health ratchet tracks the FILE's line delta
and already concedes it "needs a focused follow-up"; this is that follow-up.
Fix shape: thread the ~20 local mutables through a small phase-context
struct so each commented phase becomes a named function (the shape
`main/run.rs` already uses). Second, smaller arm: `on_mouse_wheel`
(`src/app/input/mouse.rs:983`, 150 lines) extracts its five comment-
delimited early-return phases (`try_working_set_panel_scroll`,
`try_horizontal_table_pan`, `try_overlay_wheel_route`, `try_zoom_wheel`)
into private helpers, leaving a short dispatcher — the same shape the file's
`on_press`/`on_drag` already use.

Behavior-identical refactor, both arms — which triggers the standing
outcome-audit policy (byte-identity preserves pre-existing bugs): follow
with a spot-check probe over the changed axis, not just the suite.

Verify: full native gate green; the args arm additionally replays a sample
of real CLI invocations (each capture flag family, `--keys`, seeded roots)
asserting identical `Mode`/sidecar output pre/post; wheel arm replays the
wheel-routing tests plus one capture per routed surface. code-health size
marks for both files RATCHET DOWN in the same change, not merely hold.

---
### 498 — retire the judged working-set prototype machinery, keep the Move-rows audition (found by the 2026-08-25 structure census, verified)

`src/workingset/prototype.rs:28-218` — `PrototypeSpec`
(Collapsed/Expanded/Grouped), `prototype_view`, the three builders,
`PrototypeReport`, wiring at `src/app/capture_state.rs:98`, plus its
163-line test file — was the audition scaffolding for item 444's gallery
rounds. The decisions it existed to inform are now SHIPPED: residual 3
landed real `stack_rows` (`src/workingset.rs:431`) and `expanded_rows`
(`src/workingset/panel.rs:142`, which itself documents inheriting the
Grouped prototype's order). The audition path now duplicates shipped logic
behind an inert `AWL_WORKING_SET_PROTOTYPE` env var with no live decision
left. The gallery-audition idiom as a CLASS stays blessed (item 487's
diagonal gallery is live and untouched); this one specific audition is
spent.

`prototype_move_rows`/`prototype_move_from_env` (lines 17-26 and the
`capture_state.rs:103` arm) STAY — item 444's Move-navigator sub-scope is
still open and that audition is its instrument.

Fix: delete the spent types/builders/tests and the `from_env` wiring arm;
keep Move. Check `docs/` and `captures/` for references to the retired env
var and reconcile.

Verify: full gate green; `grep -r AWL_WORKING_SET_PROTOTYPE` finds only the
Move audition's surface (or the var splits so the retired name is gone);
the shipped stack/panel test coverage is confirmed to not route through any
deleted helper before deletion, not after.

---
### 499 — hardening two panic-on-missing-state seams: PDF `finish()` and `range_apply_live` (found by the 2026-08-25 correctness census; NO reproduced defect — defense in depth)

Neither is a demonstrated bug; both are single-point invariants a future
change reopens with no compiler signal. Board-verified reachability status
is part of the item so nobody re-litigates it.

1. `src/export/pdf/writer.rs:303` `finish()` unwraps every reserved object
   slot (`.expect("planned PDF object")`) — a reserve-without-write slip on
   any document shape panics the app mid-export, a user-facing live path.
   The allocator graph was NOT fully traced; treat "unreachable" as
   unproven. Fix: `finish` returns `Result`, export surfaces the error as
   the existing notice mechanism does for save failures — export fails
   gracefully, never crashes. While there: `range_spec(id).unwrap()` at
   `range_settings.rs:75` rides the same review.
2. `range_apply_live` (`src/app/files/range_settings.rs:88`) calls the
   panicking `document.buffer()` accessor in the zero-document state's
   reach. Empirically verified NOT currently reachable — three independent
   guards (`reject_without_document`, the mouse-path twin, the stricter
   menu variant) each block it, confirmed by driving the real binary. The
   hardening: route the document-dependent branch through `buffer_opt()`
   and no-op on `None`, so the invariant stops depending on three
   independently-maintained call sites agreeing forever.

Verify: a unit test driving `finish` with a deliberately unwritten slot
asserts an error (not a panic) and a shown notice; the zero-document
settings path gets a `--screenshot-app` law asserting no panic and no-op
behavior; suite green.

---
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
