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

**✅ Move navigator sub-scope DECIDED 2026-08-25 — READY TO BUILD** (three
user answers; Residual 1's prototype gallery is preserved untracked at
`gallery/item-444-affordance-prototypes/`):

- **Action rows are CONTEXTUAL:** `Move here` is always visible (the
  primary verb — the navigator was summoned to move), but `New folder…`
  appears only when the typed query matches no existing folder — the
  quiet create-on-unmatched-name picker idiom; the folder name is
  already typed when the row appears.
- **`New folder…` creates AND moves in one stroke** — no descend-and-
  confirm ceremony; the completion notice names the full new path.
- **Keyboard grammar confirmed:** Enter on a folder row descends,
  Backspace/Left at an empty query ascends, Enter on `Move here`
  commits.

The bounded scope below stands as previously specified:

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

**✅ PICKED 2026-08-25 — READY TO BUILD.** The user approved the survey
("actually great") and DELEGATED the per-world assignment; the recorded
mapping keys off the ornament-face registers (`theme/ornament.rs`), a
roster that already classifies every world's flavor — derived, not a
hand list, so a new world inherits a mark from its register:

- **ORNAMENT_GARAMOND register** (true literary serifs — Bilby,
  Bombora): EB Garamond `›` (U+203A), candidate 1.
- **ORNAMENT_JUNICODE register** (antique/expressive — Gumtree,
  Saltpan, Magpie, Mopoke, Mulga): EB Garamond `☞` (U+261E), the
  manicule, candidate 6 — the manuscript-margin hand in the antique
  register.
- **ORNAMENT_MARKS register** (modern/technical/geometric — the rest,
  mono worlds included): Iosevka `▸` (U+25B8), candidate 4.

Candidates 2 and 5 unused. The mapping is per-world `RenderCaps` data,
so any world re-assigns in one line; final taste sign-off rides the
BUILT result in real worlds (flag a gallery re-shoot of the wired marks
for the user's glance before merge). The 🔵 ladder-step verdict is
settled: the user twice approved the sheets, which show the scaled
sizes at rest ("the fold marks look good", "survey is actually great")
— reverting the scaling stays one commit if live feel disagrees.

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
### 487 — Magpie theme-picker composition: query-to-list distance, frost boundaries through legible content, stranded chevron (USER-REPORTED 2026-08-25; ✅ USER PICKED 2026-08-25 — READY TO BUILD)

**THE PICKS (user, 2026-08-25, from the audition artifact):**

1. **`frost-top0` — ADOPT.** The Mangrove frame decided it: the blur
   stays a selective band and the title stops melting. Ship the
   pivot-compensated top-seat as the gallery built it.
2. **`frost-full` — REJECTED, on principle:** "the whole point of
   selective blur is so that you can see the themes underneath." No
   full-canvas frost on Diagonal worlds; do not re-audition it.
3. **`query-right` — ADOPT.** The user's own diagnosis: Magpie's
   fundamental problem is that everything right-aligns EXCEPT the
   caret, so the caret joins the composition. Carry `offband.rs`'s
   recorded objection (an input's sigil travels as the user types)
   into the build as a LIVE-FEEL question flagged for human
   confirmation on the real window — the pick stands unless typing
   feel disproves it, and a static capture cannot answer it either
   way.
4. **`chevron-short` — ADOPT**, at `mark_span`'s own layer
   (`cluster.rs`) per the one-owner note, not the gallery's
   `prepare_diagonal_spine` override.
5. **Refinement riding the adoption (user):** the frosted footprint
   should hug the diagonal list more tightly — "slightly more just
   underneath the diagonal themes scroll" — i.e. the footprint tracks
   the roster's own drawn extent rather than sprawling, so more of the
   theme preview stays sharp around the band. Bounded: a footprint
   geometry change within the same owner the gallery already touched,
   auditioned at the same seams if it needs its own taste check.

Build = promote the picked candidates from their `AWL_DIAGONAL_GALLERY_*`
env gates to the shipped default (deleting the gates and the rejected
`frost-full` hook), plus the item's Ready-once-picked verify below.

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
### 494 — fence-line detection gets one owner: spellcheck and the code-block toggle miss `~~~` fences (found by the 2026-08-25 duplication census, verified)

**CLAIMED 2026-08-25 — building in worktree `item-494-fence-line-owner`.**

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
### 500 — comment audit: stale roster counts, history narration, restatement (USER 2026-08-25, after a measured density audit)

Measured (tokei): 81,596 comment lines against 254,962 code lines — 32%
comment-to-code, ~70% of it doc comments.

**Standing rule (USER 2026-08-25, deliberately aggressive): the burden
of proof is on the COMMENT, not the deleter.** This tree is
AI-authored end to end, so a comment's existence is no evidence anyone
found it worth writing — the default for prose that cannot justify
itself is deletion. A comment survives only as one of three things:
(a) a constraint the code cannot express, VERIFIED against the tree
during this pass (tripwire-class entries carry git-log receipts —
those count as verification); (b) recorded decision provenance (the
user's calls — the one irreplaceable class); (c) a bare
cross-reference to the fact's living owner. Narration of what the
code does, mechanism essays recoverable by reading the code,
restatement, and manufactured gravitas all go. Two asymmetries make
this safe: git preserves every deleted sentence, and a wrongly-KEPT
unverified comment now costs more than a wrongly-deleted one — untrue
prose is what misleads the next cold-start agent. Doc comments whose
API-contract half is load-bearing keep the contract sentence and lose
the essay.

Three verified defect classes ride inside that rule, in priority
order:

1. **Stale facts.** The world roster is 19–20; live comments assert
   "all fifteen shipped worlds" (`src/render.rs:1825`, `:1852`),
   "fifteen of the sixteen" (`src/render/layers.rs:242`), "the sixteen
   worlds" (`src/theme/ornament.rs:5`). Actively false documentation.
   Fix each with roster-relative phrasing ("every `AmbientStyle::None`
   world"), never a fresh literal count; then extend the
   `doc_counts_law` mechanism to source comments — a law that finds any
   spelled-out `<numeral>-world` claim in `src/` and requires it to
   match `THEMES.len()` (or simply bans the pattern in favor of
   roster-relative phrasing). Sweep the same class for other rosters
   (face counts, arm counts) while the law is being written.
2. **History narration.** ~80 genuine sites of "used to be…"/"before
   this existed"/"an earlier version…" narration (grep inventory:
   `used to be|used to call|before this existed|an earlier version|a
   prior version|byte-identical to before`) — the convention sends
   history to commit messages. EXEMPT: a law file recounting why its
   own assertion is shaped a certain way (e.g. `range_rail.rs`,
   `nits.rs`, `alloc_bound_law.rs`) — that is a constraint the code
   cannot state, the highest-value comment class in the tree. Judge
   each site: does deleting the sentence lose a constraint, or only a
   changelog?
3. **Restatement and narration.** In the top comment-share files (12
   files exceed 55% — worst: `theme/ornament.rs` 75%, `capture/opts.rs`
   64%), the same fact is often stated 2–3× (module doc, item doc,
   field doc), padded with narration. Under the standing rule: every
   surviving fact stated ONCE at its owning declaration,
   cross-referenced elsewhere; narration deleted outright. Start with
   the 12-file roster, then sweep outward as budget allows — the rule
   applies tree-wide, the roster is just the richest seam.

Verify: the new count-claim law goes red on the pre-fix tree (proves it
catches the three known stale sites), green after; the history-grep
inventory is empty post-pass except documented exemptions; the landing
note reports lines removed and the kept-comment verification ledger (a
sample of ~20 kept class-(a) comments with the tree evidence that
verified each). A second reader spot-checks a sample of DELETIONS
against the survival test — the check is that nothing deleted was an
unrestated verified constraint, judged from the diff. Comment-only
change claims no receipt beyond the law's own test run.
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
