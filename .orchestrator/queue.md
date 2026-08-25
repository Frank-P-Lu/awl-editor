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

**CLAIMED 2026-08-25 — building in worktree `item-444-move-navigator`.**

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
