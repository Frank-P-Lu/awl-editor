# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 504 — destination navigators show no current-folder indication while browsing (found by item 444's Move-navigator verification, 2026-08-26)

Move, Export, and ProjectBrowse all navigate `browse_dir` internally but never
render it: the card title stays static (e.g. `move welcome.md`) with no
breadcrumb after descending into a subfolder, confirmed by real
`--screenshot-app` pixels on Move. Item 444's decided UX said the Move
navigator "shows the current root-relative destination" — this is the gap
against that line, left unbuilt there because a Move-only fix would violate
"same behavior ⇒ same code" (this is shared, pre-existing infrastructure, not
something Move's build introduced).

Fix shape (not yet decided): a breadcrumb rendered beside the title, or the
current folder folded into the row label — one owner all three navigators
route through, matching the pattern the mechanism already uses elsewhere.
Small enough to scope as its own item; needs a taste call on presentation
before building.

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
