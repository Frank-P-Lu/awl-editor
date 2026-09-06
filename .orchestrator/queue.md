# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---

### 558 — single open file draws no active plate in the working-set gutter (user report, 2026-09-01 — "when you first open a file, it doesn't seem to be selected?"; behavior confirmed in every capture's own gutter)

⬜ DECIDED, READY — **plate it** (user, 2026-09-06, from the lane's side-by-side
captures: "sure plate it. this means more consistent right?"). The rule becomes
"the active file is ALWAYS plated" — no lone-file exception. Build: route the
single-file identity line through the same plate mechanism as multi-file active
rows (one owner, not a lookalike rect); law sweeps both cells (lone file, fresh
open among several) per the item's closing clause; the three prior items'
calm-when-alone reasoning is superseded by this decision. Investigation history:
merged `53c7b023`. Git history (items
444/469/515) confirms the unplated single-file identity line is
DELIBERATE — item 444's own commit: "THE ONE-FILE CONTRACT IS
STRUCTURAL, NOT A RESEMBLANCE... the plate pipeline is handed no rects,
not a stack of one that happens to agree"; item 469 chose `muted` ink
specifically because "a plate-less lone heading has nothing left to
differentiate against"; item 515 weighed and kept it plate-less. Still
production-tested today (`a_single_file_block_plates_nothing`). Per the
item's own framing: deliberate AND it has failed its reader once — a
taste call, not a bug, so no default behavior changed. Both candidates
captured headlessly for comparison: unplated (current, RGB ≈29-30,40-
41,21-22 at the identity row) vs a temporary plated patch (RGB ≈126,
140,103, matching the multi-file active-row treatment) — captures are
local-only (`/tmp/gutter-plate-compare/`, not committed); the lane
describes the pixel diff precisely in its report for the orchestrator
to relay. **Q: plate the single-file identity line to match multi-file
active rows, or keep it bare (the documented "calm when nothing to
distinguish" reasoning)?**

Separately verified (not a taste call): a freshly opened file among
several already-open files IS plated immediately — no bug, traced to
`WorkingSet::open` setting `active` unconditionally and `stack_rows`
re-deriving fresh on every call, no cache/debounce. Landed as a
mutation-proven law (`workingset::tests::a_freshly_opened_file_among_
several_is_active_immediately`). Exact-main receipt: health pass:266s,
both conventions, menubar=full:on, 4729 unit tests, 17 integration
targets.

With exactly one file open, the identity line shows the bare name — no
active-row plate — and the user reads that as "not selected." Two-file
state plates the active row (their own earlier screenshot). Confirmed as
current behavior incidentally in every capture this session (the
bottom-left gutter of each PNG shows the lone open file unplated).
Mechanism neighborhood: the single-file identity line is a separate door
from the stack (`render/chrome/gutter.rs` — "the identity line is EITHER
the lone filename or the working set's rows"; it already shares the
close-mark lane via one mechanism), and `gutter_stack::plate_rects` only
plates `file.active` STACK rows. Whether the unplated single file is a
deliberate calm-when-nothing-to-distinguish choice or an omission is for
the lane to establish from the tree and `git log` — and if deliberate, it
has failed its reader once, which is data: bring the finding plus a capture
of each candidate (plated vs not) back to the user for the taste call
rather than landing either silently. Also establish whether a freshly
OPENED file among several is plated immediately (the user said "first
open"; their screenshot is the single-file case — the multi-file
fresh-open cell is unverified). Law once decided: sweep both cells.

---

### 537 — footnote markers may wear the traditional reference ladder (user decision, 2026-09-01; sequenced AFTER 529 bundles the face)

⬜ DECIDED, READY — both product decisions landed (user, 2026-09-06): **(a)
per-document ladder scope** — the ladder follows first-reference order across
the whole document, matching today's numbering; awl has no pages, so per-page
recycling has nothing coherent to recycle on. **(b) The definition list
follows the option** — when the ladder is on, definitions wear the same mark
as their references; pairing them is the ladder's function. U+2016 ‖ coverage
in the adopted subset remains the lane's engineering verification, enrolled in
the glyph-presence law before landing.

DECIDED direction, from the user's own connection during 536's heritage
round: "the daggers were used for footnotes — we still have a chance to
use them, cuz we support footnotes." awl's footnote references already
paint their DISPLAY NUMBER as a painted ornament slot
(`footnote_number_slot` / the `FootnoteNumbers` ornament family,
docs/markdown.md — the same painted-substitute shape the bare-URL
ellipsis reuses), and display numbers already follow first-reference
order. This item adds a display OPTION (config + Settings row, default
staying numeric) that paints the TRADITIONAL REFERENCE LADDER instead:
* † ‡ § ‖ ¶, in that canonical order, doubling when exhausted (** ††
‡‡ …) per print tradition. Display-only, exactly like smart punctuation:
the file keeps `[^label]`; export unchanged (numeric) unless a later
item decides otherwise. The glyphs come from the symbol face — with
Nishiki adopted (529), † ‡ § ¶ are the celebrated cabinet's own
drawings, so the heritage is in SERVICE, not decoration: the daggers do
the same job they have done since the hand-press. Open sub-decisions
for the lane to put to the user before landing: (a) ladder scope —
per-document order (matching today's numbering) is the working
hypothesis; per-page recycling is print tradition but awl has no
pages; (b) whether the footnote DEFINITION list's markers follow the
same option; (c) ‖ DOUBLE VERTICAL LINE (U+2016) coverage in the
adopted subset must be verified and enrolled in the glyph-presence law.
Laws: ladder order pinned against the historical sequence; overflow
doubling; option off ⇒ byte-identical render to today.

---

### 567 — scripts/ cruft sweep: delete six concluded-investigation scripts and five unwired shell laws (user decision, 2026-09-06 — "unused stuff is kinda cruft right… i think i lean delete too")

🟢 VERIFIED AND GATED, MERGE PENDING — lane `item-567`, worktree
`.codex/worktrees/item-544-frost-box`, tip `7c99639a`.

**The premise held exactly, and this time it was re-measured rather than inherited.** The
lane swept the pre-sweep tree with `git grep -F` after first proving that tool reaches
hidden paths, and separately ruled out the two ways a basename sweep can miss a live
reference — glob-shaped paths under `scripts/`, and directory enumeration inside
`code-health.py` (which filters a tracked-file list and holds no script roster). All eleven
deletions have zero references outside their own bodies except the three the brief already
named: `capture-ground-space.sh` and `capture-overlay-header-identity.sh` at
`docs/render.md:54`/`:164` (the brief said 50/160; the file has since shifted four lines) and
`test-sccache.sh` at `src/version_law.rs:15`.

`8a2b5d48` was NOT the tip — `2c41d06a` was, a rebase of it with two hunk-header offsets and
otherwise an identical patch. Content is right: exactly the eleven deletions, no more and no
fewer, all four riders landed, and every named keep still present (both CI budget twins
confirmed live in `ci.yml`).

One defect found and fixed: dropping the clause from `version_law.rs`'s module doc left the
paragraph unwrapped — a 41-column orphan and one line four columns past the 76-column
ceiling the rest of the file holds. `7c99639a`, wrap only, text unchanged.

Receipt: `native-gate-receipt commit=7c99639a health=pass:323s conventions=mac,linux
scope=all-targets menubar=full:on unit_tests=4853 unit_shards=6 integration_targets=17`,
plus `web-smoke: OK`. Both came on retry, for the reason item 593 now records.

⚠️ **The receipt's base is `1bb0e751`, not today's main.** The branch is 2 ahead / 8 behind
and cannot be fast-forwarded, so the merge train re-gates the combined candidate rather than
carrying this receipt across. The divergence is provably benign — the sweep's 15 files and
the 26 files main has changed since are disjoint — but benign is not gated.

**Two corrections this item's own brief got wrong, recorded so they die here:** (a) the brief
says the only wired shell law is `test-native-gate.sh`; there are TWO — `linux-deps-law.sh`
is wired at `code-health.sh:70`. The deletions are unaffected. (b) hygiene rider (a)'s stated
motive is false; see item 594.

A full audit of `scripts/` (78 tracked files) found the tree largely
load-bearing — CI-invoked, gate-invoked, or documented entry points — with
exactly eleven deletions and two hygiene riders. Git history keeps everything;
no dangling references may survive the sweep.

**Delete, zero references anywhere:**
`capture-overlay-chrome-dpi-matrix.sh`, `capture-paperbark-wallpaper.sh`,
`capture-warp-motion.sh` (evidence captures from concluded investigations) and
`ci-mac-bisect.sh` (the hosted-mac freeze bisect harness; the wedge it hunted
is still the tolerated-red pair in ci.yml, and `git log` keeps the tool if the
hunt reopens).

**Delete, past-tense doc citations only:** `capture-ground-space.sh`
(docs/render.md:50) and `capture-overlay-header-identity.sh`
(docs/render.md:160). Each is cited once as the evidence sheet of a finished
investigation — reword those clauses so no deleted path stays referenced; the
evidence lives in history.

**Delete, the five unwired shell laws:** `test-sweep.sh`,
`test-disk-preflight.sh`, `test-reap-orphaned-gates.sh`,
`test-worker-build.sh`, `test-sccache.sh`. Nothing runs them — the only wired
shell law is `test-native-gate.sh` (code-health.sh:73). Measured 2026-09-06:
two of the five are ALREADY red, and both reds are law-rot on healthy
subjects, not regressions — `test-sccache.sh` pins the literal `5G` while
`.cargo/config.toml` moved to `10G`; `test-worker-build.sh`'s
RUST_TEST_THREADS owner-whitelist predates native-gate.sh/code-health.py and
its `rg --hidden` sweeps `.git/lost-found`. Wiring instead was costed and
declined: ~40s per gate (the reap law alone runs 30s) plus repairing both
rotted laws, to guard orchestrator plumbing rather than the product. Recorded
decision: an unwired law is worse than none — it rots silently and then cries
wolf; if one of these subjects earns a law again, it gets wired into
`code-health.sh` at birth, like `test-native-gate.sh`.

**Reference cleanup that makes this Rust-touching:** `src/version_law.rs:15`'s
module doc names `test-sccache.sh`'s fixture crate among the legitimate
`0.1.0`s — drop that clause. Full receipt required.

**Hygiene riders:** (a) `code-health.sh`'s `python3 scripts/code-health.py`
invocation gains `PYTHONDONTWRITEBYTECODE=1` (parity with
`test-native-gate.sh`; kills the recurring `scripts/__pycache__/`). (b)
RELEASING.md gains one line naming `scripts/pretag-journeys.py` as the
pre-tag journey-sweep instrument — CLAUDE.md's pre-tag policy demands the
sweep, and no doc tells anyone the tool exists.

**Explicitly kept (asked and answered, do not re-audit):** the
`ci-atspi-budget.sh`/`ci-wedge-budget.sh` near-twins (both live in ci.yml;
merging is churn), the ground-contrast and ambient-motion capture/measure
pairs (`ground-contrast-measure.py` is cited by
`render/tests/deckle_ground.rs`; ambient motion just shipped), the hero
image trio (site asset pipeline, site/README.md), and `pretag-journeys.py`
(documented by rider b).

Verify: after the sweep, `rg -l <basename>` for each deleted file returns no
tracked hits; `scripts/code-health.sh` passes; full native-gate receipt (a
Rust doc comment moved).

---

### 568 — spell suggestions never offer the user's own dictionary words (user report, 2026-09-06 — "add to dictionary, but this new word doesn't really show up in the autocomplete?" — the ⌘-; spell picker is meant; explicitly NOT a completion feature: "we don't need autocomplete")

🟢 VERIFIED AND GATED, MERGE PENDING — lane `item-568-569`, worktree
`.codex/worktrees/item-543-tabledims-frost`, tip `5795ebeb`. **The board was stale and the tree
was right:** fully implemented as `969a1f88`, every Build and Laws clause satisfied.

The premise survives and is one-sided, which is why the bug was invisible: `check` consults
`user_words` (spell.rs:271) while pre-commit `suggest` was `self.dict.suggest(word, &mut out)`
alone, and nothing ever inserts a user word into `self.dict`. New `spell/personal.rs` runs a
bounded Levenshtein scan over `user_words`, recases to the typed shape, and `merge_ahead`
places personal hits AHEAD of bundled ones — load-bearing, because the Spell card truncates
after assembly. 12 laws, enrolled from `DictVariant::ALL` rather than a named member.

Mutation-proven by the lane, red observed each time: reinstating the bundled-only `suggest`
reddens **9 of 12**; ranking personal behind bundled reddens the ranking law and the
post-truncation law.

The personal dictionary is check-only. `SpellChecker::check` consults
`user_words` (src/spell.rs:261), but `suggest`/`suggest_at`
(src/spell.rs:364) consult `self.dict` — the bundled Hunspell dictionary —
alone, so a typo one letter off a word the user added is never offered that
word as a correction. The add path, storage
(`~/.config/awl/dictionary.txt`, one word per line, loaded lowercased via
`set_user_words`) and cache invalidation are all sound; only the suggest
side is blind.

Build: merge near-miss matches from `user_words` into the suggestion list.
The set is small (a personal dictionary), so a simple bounded edit-distance
scan over it is enough — no need to teach spellbook anything. Decisions the
worker owns: ranking (a personal word at comparable distance should not
lose to a dict word — it is the user's own vocabulary) and case shape
(stored words are lowercase; follow the typed word's capitalization the way
Hunspell suggestions do). Note the picker truncates to
`OverlayKind::MAX_SUGGESTIONS` after assembly (overlay/state.rs:892) — merge
before the cut.

Laws: add a word, misspell it nearby ⇒ the added word appears in
suggestions (through the real `suggest` path, not a fixture predicate); a
distant added word does not appear; capitalized typo ⇒ capitalized
suggestion; both the chord picker and the palette "Spell suggestions…" row
reach the merged list (one owner, no second suggest path).

---

### 569 — personal dictionary picker: list the words, remove per row (user decision, 2026-09-06 — "richer listing words… something very simple")

🟢 VERIFIED AND GATED, MERGE PENDING — same lane, worktree and tip as 568. **This half was not
unstarted either:** committed as `5eb70b93`, every clause satisfied. `OverlayKind::UserWords` on
the Asset Cleaner's grammar, `Effect::ForgetUserWord`, a file rewrite preserving `#` comments,
blank lines and order, and the two per-row destructive accepts merged into ONE owner
(`row_retiring_accept`) rather than two adjacent early-returns saying the same thing. Docs in
GUIDE/REFERENCE/CAPTURE/harness-reach.

Mutation-proven, red observed: a rewrite that drops comments/blanks reddens three laws; skipping
the in-memory removal reddens two; a `remove_user_word_row` that reports success without removing
reddens the row-retirement law.

**Capture clause honoured, checked independently.** No PNG or JSON is committed and no `/Users/`
appears anywhere in the branch diff. The lane reproduced the capture itself against an explicit
`--config` and a seeded `--root`, writing outside the tree: `driver: "live-app"`, `mode:
"user_words"`, `replay_skips: []`, rows gathered from the live checker. Row legibility asserted by
its own pixel arithmetic — ink 162 against a uniform ground of 191.7. The lane reports its figures
differ from the commit message's because it ran a different world, and confirms only "real ink",
not the commit's specific numbers. That is the right way round: a figure owed to a human is read
out of the product, never out of the report that landed it.

`~/.config/awl/dictionary.txt` is user-facing and completely undocumented —
no UI shows its contents and no doc says it exists; removal is hand-edit
only (src/app/files/dictionary.rs:90, deliberate v1). Build a summoned
picker, very simple: a palette row (working name "Personal dictionary…")
listing the loaded words, with accept-on-row REMOVING that word — the
"Clean unused assets…" picker is the exact structural precedent
(overlay/assets.rs: rows from a summon-time scan, per-row destructive
accept that keeps the picker open via `remove_asset_row`, effect applied in
app/apply.rs). Removal rewrites the file preserving what the append path
already preserves — hand-edited comments, blank lines, order — minus the
removed line, through `fs::write_atomic`, then invalidates spell caches and
respells (same tail as `add_to_dictionary`).

Empty state says where words come from ("Add '…' to dictionary" in the
spell picker). Enrolment: `OverlayKind` roster, the closed
`COMMAND_TASK_CATEGORIES` table and the commands catalog (the coverage law
holds the row red until classified). Rider: document the file and the
picker in GUIDE/REFERENCE (regen-reference.sh if the roster feeds it).

Laws: picker lists exactly the loaded words; removing one respells the
buffer (a squiggle returns); the file keeps its hand-edited comments across
a removal; capture the picker via `--screenshot-app` against an explicit
`--config` (never the ambient one — overlay rows photograph real content).

---

### 570 — blockquote pull-quote: the 66 gets its 99 (user report, 2026-09-06 — "the 66s must be followed with 99s… it kinda bothers me"; the blockquote ornament, not smart quotes)

The hanging pull-quote mark draws only the OPENING `“` — `QUOTE_MARK_GLYPH`
(src/render/layers.rs:64), shaped once in the world's display serif and hung
at `pull_quote_left` on each blockquote block's first line
(`TextPipeline::quote_marks`, ornaments.rs `QuoteOrnaments`). There is no
closing counterpart, so every quote in every world reads permanently
unclosed. Build the `”` (U+201D): same scale, same `theme::faint()` value,
same display-serif shaping, hung at each block's LAST line. Placement is
the one open design question the worker resolves and flags for live taste
confirmation: the typographically honest spot is trailing — mirrored into
the right margin at the block's end (or hanging after the last line's
text) — pick by capture comparison across a few worlds, not by argument.

Grounding: the quote-orientation law (render/tests/quote_orientation.rs)
already pins all four curly codepoints heavy-bottom in every display face,
so no new font risk; the never-tofu law covers the glyph. Multi-block
documents get one pair per block (quote_marks already walks blocks).

NOT in scope, recorded so nobody conflates: straight→curly smart-quote
substitution (typing or render) stays declined per the standing user
decision — this item closes the ornament's pair, it does not educate quotes.

Laws: every world's blockquote capture shows BOTH marks by pixel presence
(a presence floor, not just a contrast ratio — the mark must exist to
pass); open and close share value/scale by arithmetic; a one-line
blockquote still shows a legible pair (the degenerate case where first
line == last line); no mark on non-blockquote lines.

---

### 571 — revealed thematic break keeps its ornament-sized row: giant raw `***` and a caret slab 2.2× too tall (user report, 2026-09-06 — "the way the cursor highlights the * is all wrong… like its wayy too tall?"; reproduced headlessly, first try)

🟢 MERGED, EXACT-MAIN RECEIPT OWED — landed on main as merge `a7846f7a` (`6b1a98a4` + fmt
`0d765228`). The receipt quoted below is the merge-train candidate's, taken at `ceb5c252`;
main has since taken 574, 575, 542 and 560's hint card, so no receipt describes the tree as
it now stands. **First attempt 2026-09-07 produced NO RECEIPT and is void on two independent grounds, neither
of them a defect in the tree:** HEAD moved under the run (`3cf7061b` → `fdf7139d`, a
concurrent board-only commit), and the host was at load 43 on ten cores with three lane
builds live, so `code-health.sh`'s `cpu-spin` probe — which asserts that a deliberately
spinning fixture peaks a tracked core — could not see its own spinner. That is this repo's
standing hazard in its purest form: the law was fine and the CONFIGURATION it ran under was
the untested hypothesis. The receipt is re-taken on a quiet host once the lanes are off it,
and until then no one may describe main as gated.
`build_line_attrs` hoists the reveal predicate above the ornament-scale decision, so a
revealed rule line drops the ornament's room entirely: raw markers at body size, body-height
row, body-size caret. Four laws (2 sidecar geometry, 2 real-pixel), each single-mutation
proven. Merge-train receipt on the exact combined candidate (566 + 571/573 + the mark raise):
`native-gate-receipt commit=ceb5c252 health=pass:239s conventions=mac,linux scope=all-targets
menubar=full:on unit_tests=4835 unit_shards=6 integration_targets=17`. `rects.rs`'s size mark
rose 1755 → 1775 with the reason recorded (frozen baseline 2238, so a raise below the
baseline, not a ceiling breach).

Measured on Saltpan (`--theme Saltpan`, caret onto a `***` line): a body row
is 32px; the thematic-break row is 70.4px — exactly `ornament_scale` 2.2× —
caret on or off. On reveal the raw `***` shapes at 2.2× body size (33px
advances vs ~14px body) and the block caret takes the full row: a ~33×70px
primary-ink slab dwarfing the small high-hung asterisk ink. Same numbers
for `---`; reproduces in every world (each carries `ornament_scale` ≥
~2.16) — Saltpan is simply the default.

Mechanism: deliberate machinery with an unconsidered corner.
`md_line_scale` (src/render/spans/layout.rs:40) grows a confirmed
thematic-break line to the world's `ornament_scale` so the row fits the
centered break fleuron `prepare_ornaments` draws off-caret, in lockstep by
design. But it is a whole-line font+row scale, so the caret-entry reveal
inherits the ornament's size for the raw markup, and the block caret
inherits the 70px row. The ornament needed the room; the revealed text
never did.

DECIDED (user, 2026-09-06 — "yeah exactly no need for ornament room"):
while the rule line is revealed (caret OR selection touches it — the one
`wysiwyg_reveals` rule), the ornament is not drawn, so the line drops the
ornament scale entirely: raw `***`/`---`/`___` at body size, row at body
height, caret at body caret size. Line-local reflow on reveal is already
the conceal system's accepted cost, and `refresh_rule_conceal` already
invalidates `row_geom` on exactly this transition — thread the reveal
state into the `md_line_scale` decision through its confirmed-rule gate's
call sites rather than adding a second scale owner. Mind the skip-gate
tripwire: the reveal set already keys on caret line AND selection
(`last_conceal_selection`); the scale must key on the same state or a
selection-only reveal serves a stale tall row.

Laws: caret on a `***` line ⇒ that row's sidecar height equals the body
row height and revealed advances equal body advances (sweep `***`/`---`/
`___`); caret off ⇒ height returns to `ornament_scale` × body and the
ornament draws (presence floor by pixel arithmetic, not just geometry);
selection touching the line without the caret ⇒ same reveal metrics (the
skip-gate axis); a `---` inside a fenced code block stays body-height
throughout (the confirmed_rule gate must keep holding). Verify across a
second world beside Saltpan so the scale is read from the roster, not
pinned.

---

### 572 — class audit: decorative geometry vs the caret — every row or advance inflated for an ornament, probed against what caret/selection/highlight inherit (user decision, 2026-09-06 — "we should fix this class of bugs yeah?? like ornament + cursor")

Item 571 (rule row's ornament room swallowing the block caret) is one
member of a class, and 545 (smart-punct conceal's giant reserved slot) is
another — per standing policy, the neighborhood gets audited because bugs
cluster. The class: any mechanism that grows a ROW's height or a run's
ADVANCES for a decorative replacement, cross-examined against every
caret-adjacent treatment drawn from that geometry (block caret, beam
caret, selection band, spell/nit underlines, link underline, find-match
wash).

Enumerate the inflation sites from the code rather than from this list —
known members to seed the sweep, not bound it: the thematic-break
`ornament_scale` row (571 fixes the reveal; the audit checks selection
bands and underlines on it too), the heading ladder's DECOUPLED row growth
(row grows "beyond what its font size alone needs" — does the block caret
span the decoupled extra, or the em box? a big caret on big text is
coherent, a caret taller than the heading's own glyphs is not), inline
image rows (absolute line-height, same decoupling shape), the smart-punct
reserved slot (545 — what does a selection or block caret over the slot
look like before/after that fix), zero-width concealed spans (caret x and
selection band at a collapsed span's boundary), and table x-ray rows.

Probe form per policy: state (caret-on / selection-touch / off) × surface
(each caret/highlight treatment) × world (sampled across the roster,
including one dark world and one with a large equalized `ornament_scale`,
e.g. the 4.6× member), asserting per cell with sidecar geometry AND pixel
arithmetic — a highlight's drawn box against the row's glyph ink box, not
just against row metrics (the sidecar is a state oracle, not an appearance
oracle). Production audit tier. An audit that finds something ends by
writing the missing law; the deliverable is the cell table plus laws, with
each defect it finds either fixed in-item when small or queued as its own
scoped item when not.

---

### 573 — selection-revealed rule line double-draws: raw `***` AND the break fleuron together (found by 572's first probe, 2026-09-06; reproduced headlessly by pixel arithmetic)

🟢 MERGED, RECEIPT OWED — landed with 571 as merge `a7846f7a`. `rule_lines` now drops every
line the selection touches, not just the caret's own, so the fleuron stops drawing over the
revealed raw markers. Both layers read one owner (`selection_touch_bytes` /
`selection_touches`) rather than re-deriving the reveal state, which is what let them
disagree. Merge-train receipt taken at `ceb5c252`; the exact-main receipt for today's HEAD — see 571.

A selection that touches a thematic-break line (caret elsewhere) reveals
the raw markers — correct since the 2026-07-22 selection-reveal widening —
but the centered break fleuron still draws on top of the row.
Caret-on-the-line hides it correctly. Evidence, Saltpan: off-state fleuron
ink at x 586–613 with column left 139.2; in a selection-touch capture the
adaptive column sat 80px right (left 219.2) and an identical-width ink run
appears at exactly 666–693 inside the revealed row, alongside the 2.2×
raw `***`. Mechanism: the selection-reveal decision (`wysiwyg_reveals` +
`selection_touch_bytes`) was threaded into span/line decisions and the
table x-ray, but `prepare_ornaments`' hide-while-revealed gate still keys
on the caret line alone.

Build: thread the same selection-touch state into the ornament layer's
draw decision — one reveal rule, one owner, no per-layer re-derivation.
Sequenced WITH 571 (the row-scale fix must key on the identical state or
the two layers disagree again from the other side).

Laws: selection touching a rule line ⇒ zero fleuron ink in that row's band
(pixel arithmetic, swept across the adaptive column's positions — the
probe that caught this moved the column); caret-on keeps hiding it;
selection cleared ⇒ fleuron returns (presence floor, the ornament must
exist off-reveal for the absence assertion to be non-vacuous); a fenced
`---` body line draws no fleuron in any state.

---

### 576 — follow a link: right-click shows "go to", modifier-click opens (user decision, 2026-09-06 — "right click on a link should show 'go to'; and i guess command click should open it too? (what should it be on linux…??? like for each keymap?)")

The followable-span grammar already marks every followable span with one
underline (named links and tamed bare URLs, one owner) — but nothing
follows them. Build the follow affordance:

- **Modifier-click opens.** macOS: ⌘-click (decided). Linux: recommend
  Ctrl-click under BOTH flavors — the platform convention in every editor
  and browser, and a mouse chord, so it collides with none of the C-c/C-x
  text-chord rules (the keep-list machinery governs key chords, not mouse
  chords) — plus middle-click (mouse-2) as the emacs flavor's own follow
  gesture, seeded Linux-only like the Meta layer, inert on Mac and under
  native. Route the gesture through the keymap/platform seed-table seam so
  label surfaces and dispatch read one source; decide there whether the
  mouse chord is `[keys]`-rebindable or fixed (recommend fixed v1).
- **Right-click on a followable span** shows the go-to affordance. awl has
  no context menus by design — keep it summoned and minimal (a one-row
  card in the overlay family, or the pointer affordance family from the
  hover work), not a native NSMenu grafted onto the wgpu view (see the
  muda tripwire). Label: "Go to <destination>" with the tamed authority,
  never the raw URL flood.
- **What opens where:** web URLs hand off to the system opener
  (`open`/`xdg-open` — an outward action at the user's explicit gesture,
  not a runtime fetch; the zero-network invariant is about awl phoning
  home, and this is the EDITOR-daemon shape of OS integration). A RELATIVE
  path to a local file follows IN awl — the Live-Preview model's own move
  (a vault of notes linking each other). Heading anchors and footnote
  jumps: deferred, recorded.
- Verify: the follow gesture produces a typed effect carrying the resolved
  destination, asserted in the sidecar through both chord replay and the
  real-App driver; the actual launch is the live-only tail (harness-reach:
  flag it, do not stub the effect layer around it). Laws: modifier-click
  on a plain word produces nothing; the same click on each followable kind
  resolves the destination the underline grammar says it has; Linux seeds
  appear under the right flavors and the Mac binding under none of them.

---

### 577 — `Install sccache` costs 4m25s on every cold CI run because it builds from source (found by 566's step-timing, 2026-09-06)

`scripts/install-sccache.sh` builds sccache from source. It short-circuits when the pinned
version is already on PATH, so a warm run pays 0s and this was invisible until item 566
timed the first cold run in sixty: **4m25s**, the second-largest line in that job's
pre-suite budget. A prebuilt-tarball path would take ~4 minutes off every cold run, and
cold runs are now guaranteed to recur — rust-cache's key carries the rustc version, so
EVERY stable toolchain release produces one.

Not filed as a trivial swap: the script is shared with `release.yml`, so the blast radius
includes the release pipeline's permanently-unexercised `publish` job, and downloading a
prebuilt binary is a supply-chain and network-policy call rather than a build-speed one.
Decide the policy first (pin by digest? verify a checksum? keep source-build as the
fallback when the tarball 404s?), then implement.

Verify: a cold-cache CI run's `Install sccache` step drops to seconds; the release
workflow still installs the same pinned version by the same identity check.

---

### 578 — `code-health.py --self-test` has been failing on main, unnoticed, because no gate runs it (found by 566, 2026-09-06)

`code-health.sh` never invokes `python3 scripts/code-health.py --self-test`, so the
self-test drifted red on `main` without anyone seeing it: six requirements had been added
to `native_gate_audit` over time without updating its script fixture (receipt shape,
`git status --short`, `gate_health_command`, and the three menu-bar arms). Item 566's lane
repaired the fixture — `code-health: self-test clean` now, including its two new
mutations — but **left the wiring alone deliberately, because wiring it changes what the
gate runs, and that is an orchestrator/user call.**

This is the same shape as 567's recorded decision that an unwired law is worse than none:
it rots silently and then cries wolf. The difference is that this law is not a candidate
for deletion — it guards the ratchet script that guards everything else.

DECISION OWED: wire `--self-test` into `code-health.sh` (cost: its runtime on every gate,
measure it first) or accept that it is a manual instrument and say so in the script's own
header. Do not leave the third state, which is what it has been in.

---

### 579 — awl renders ~9 fps on a pure software rasterizer, every world (measured by 566, 2026-09-06; predates 564)

Measured on the full roster at 2910x1720 @2x, `--release`, median `queue.submit +
device.poll` over 300 timed frames, under `llvmpipe (LLVM 15.0.6, 128 bits)`: **82-184 ms
per frame for every world** — Wagtail 82.2 at the fast end, Saltpan 184.3 at the slow. The
same binary on this host's Metal renders Kite in 1.310 ms, so lavapipe is ~84x slower
across the board. This is a property of the whole render, not of any one ground, and it
predates item 564.

It is recorded rather than actioned because nobody has established that it MATTERS: a
Linux user on real hardware has a real GPU, and the software path is what a VM, a
remote-desktop session, or a machine with no working Vulkan driver falls back to. The
question the Linux release wants answered is whether that fallback is a supported
configuration or a documented non-target.

⚠️ Do not repair this by measuring on Metal — no local gate sees the axis, and the number
above is from one arm64 container with Mesa 22.3.6, not from CI's x86_64 lavapipe. Any
claim about "software rendering performance" needs its configuration stated, per the
standing rule that a check runs in one configuration and that configuration is itself an
untested hypothesis.

---

### 580 — insertion-door census: every path that can mutate the focused buffer is enrolled at one seam (follow-up to 575's lane report, 2026-09-06)

⬜ READY (blocked on 575 merging — builds on its wall and `text_door.rs`)

Evidence: 575's lane found TWO insertion doors nobody had listed (assistive
`ReplaceSelectedText` and `SetValue`) beside the briefed one (`Ime::Commit`). The class
grows every time a new input capability lands (menu Edit actions, Linux middle-click paste,
future dictation, daemon/EDITOR writes, drag-drop text if ever), and today a new door
ships OPEN by default: nothing forces it through 575's wall, so the next one repeats this
bug on whatever modal surface exists then.

Build: enumerate every code path that inserts or replaces text in the focused buffer from
an input surface, and route each through the one wall 575 built (or record it as a NAMED
exemption with a reason — e.g. the capture harness's own replay). Then the census law, per
"same behavior ⇒ same code": a wildcard-free match over the door roster at the seam, so a
NEW door fails to enrol until it declares wall-routed or exempt. The law must fail on the
bug it names: prove non-vacuity by re-opening one existing door (locally, uncommitted) and
watching it go red. Pure unit seam — no capture needed; harness-reach is not in play.

Worker: engineering tier. Worktree per protocol; claim before code.

---

### 581 — substitution-leak audit: what else reports about the hidden document while History/Conflict/Credits is up? (follow-up to 575's lane report, 2026-09-06)

⬜ READY (after 575 merges; audit-tier per standing policy)

Evidence: 575's rail-query leak — typing on Credits filtered the HIDDEN document's rows
and reported "no matches" about a file the user couldn't see. Bugs cluster: the leak class
is "a query/filter/status surface answering about the underlying buffer while a
`TimelineOverComparison` substitution is showing something else." The rail is fixed; its
siblings are unaudited.

Probe form (state × surface × world, per the spot-check policy): the three roster overlays
(History, Conflict, Credits — derive the set from `shows_read_only_prose`, never name it)
× every query/status surface that reads the focused buffer — find & replace, spell
navigation/suggestions, outline/jump surfaces, palette state lines, the debug HUD's
buffer readouts — sampled across ≥2 worlds. Assert per cell with sidecar/pixel arithmetic
via `--screenshot-app` (check docs/harness-reach.md per effect before promising a capture;
flag any live-only cell for human confirmation instead of claiming it). An audit that
finds something ends by writing the missing law; if the audit finds nothing, it still ends
by writing the law that pins the clean state, enrolment derived from the roster.

Worker: audit tier — Sonnet medium on Claude or `gpt-5.6-terra` medium on OpenAI.

---

### 582 — Kite tunnel visual correction: restore the approved bending, folded 3D surface (user report + decision, 2026-09-06)

⬜ READY — corrective follow-up to 564. Queue only; not dispatched.

**Outcome.** The user rejected the delivered appearance: regular concentric
circles and straight spokes, unlike the approved organic tunnel prototype.
Restore the prototype's depth-dependent bending, section rotation, and folded
silhouette in the shared background renderer. Matching parameter labels is not
visual parity. Item 564's engineering completion does not constitute acceptance
of its appearance; this item owns the correction and renewed visual sign-off.

**Evidence and first check.** Source comparison finds the saved prototype
projects individual 3D tube points with depth-dependent centre displacement,
radius and roll. Current `shaders/background.wgsl` instead uses a single polar
axis, adds a bounded shift to a logarithmic ring coordinate, tapers folds back
to circles outside a depth band, and derives rails directly from `theta`.
These are substantive geometric differences. First reproduce with a fresh
native capture and explicit config; record build and motion policy so an old
running binary cannot be mistaken for current source. The user's screenshot
contains private prose: do not copy it into tracked fixtures or reports.

**Reference.** Existing approved standalone study (read-only reference, no new
web artifact): `$HOME/.codex/visualizations/2026/08/18/01a01547-26ee-7cf3-be19-203fc0c69a13/living-tunnel-study.html`,
especially `tunnelPoint`, its sampling loops, and motion update. Read the actual
file before implementing. Preserve reference evidence outside the repo; capture
only seeded public prose for any tracked native comparison. The core mapping is
recorded here so the brief remains useful if that local artifact is unavailable:

```text
turn = theta + worldZ * twist
pulse = 1 + .075*sin(worldZ*1.25) + .035*sin(worldZ*2.7 + theta*2)
radius = max(.46, 1 + fold*(.46*cos(3*turn)
                          + .18*sin(5*turn - worldZ*.35))) * pulse
pathX = .22*sin(worldZ*.48) + .07*sin(worldZ*1.17)
pathY = .17*cos(worldZ*.39) - .06*sin(worldZ*.91)
angle = theta + .12*sin(worldZ*.31) + spin
x3 = pathX + radius*cos(angle)
y3 = pathY + radius*sin(angle)
scale = min(width,height)*.72 / z
p = clamp((z-.72)/(10.8-.72), 0, 1)
bend = p*p*(3-2*p)
centre = viewportCentre + (vanishingPoint-viewportCentre)*bend
screenPoint = centre + (x3,y3)*scale
```

**Build / scope.** Preserve the resulting projected surface, including curved
longitudinal ribs, displaced sections, and visible folds/overlap where the
reference has them. One continuous tube can have a centreline that bends with
depth: continuity does not require every section to share one screen centre.
Choose a bounded reusable rendering mechanism appropriate to that geometry;
do not force it into the existing closed-form polar approximation if that loses
the shape. Keep theme choices as data, one motion owner, and native/browser
support. No Kite-name branch, separate per-margin tunnels, or general scene
editor. Revisit laws that enforce straight radial rails or concentricity: those
properties describe the rejected implementation, not the product contract.

Retain the approved light lavender/mineral palette, calm central page, fold .34,
twist .72, forward drift .05, and 58 ribs as visual reference settings. Any
necessary parameter/unit conversion must preserve appearance and be explained.
Retain random corner targets with no immediate repeat, 15-second dwell,
12-second smooth transit, and very slow section roll (study spin rate
`twist*.035` per second). Retain subtle convergence haze without orb/crosshair.
Handle dense distant lines with antialiasing and depth/contrast fading while
preserving the near-field surface; flattening the tube is not the fallback.
Reduce Motion and Ambient-off use a deterministic authored static folded pose;
ordinary pause/lost focus freeze the current pose. Static mode must keep the
same characteristic geometry. No new user-facing controls are needed.

**Done / verify.** Read THEMES.md, docs/render.md, CAPTURE.md and
docs/harness-reach.md before implementation. Produce matched reference/native
views at recorded viewport, page width, pose and settings: top-right and
bottom-left dwell, a transit midpoint, and the motion-safe pose; include both
1200×800 and 1600×1000 and DPI 1/2. Native captures use an explicit hermetic
fixture/config. Use the existing deterministic motion seam where it reaches
these states; extend the shared seam if necessary rather than inventing a
parallel renderer for tests. Sidecars verify state; pixels verify visible
curvature, fold presence, page legibility and continuity across page masking.
Compare projected landmarks/curves against the reference geometry with an
explicit tolerance; a nonzero fold uniform alone proves nothing. Add a law
that fails on the current concentric-ring/straight-spoke approximation and
prove it fails after the mutation builds and runs. Sweep narrow/wide pages,
corner/transit poses, and static/moving states for crowding and aliasing.

Perform the standing vision smoke over about five real gallery shots with
concrete questions about curved ribs and folded sections; obtain a visual
judge's reference comparison before declaring visual parity. Keep real-time
comfort and final resemblance acceptance explicitly OWED to the user; present
the comparison images, not just test receipts. Measure release rendering cost
before/after, preserve bounded wake/freeze behavior, and run native gate and
web smoke for the implementation. A passed engineering gate cannot close the
user's visual rejection by itself.

Worker routing when dispatched: `gpt-5.6-sol` high for geometry/implementation;
visual judge `gpt-5.6-sol` xhigh; outcome audit `gpt-5.6-terra` medium. Follow the
board's claim/worktree protocol and integrate serially with overlapping shader
work. This brief authorizes the correction, not unrelated background redesigns.

---

### 583 — blank new document raises an autosave failure (live bug hunt, 2026-09-06)

⬜ READY — user requested queueing; not dispatched.

**Observed / reproduce.** In the running macOS app, Kite, create a document
through File → New document or ⌘N and pause without typing. The notice reads
“autosave failed: empty note: nothing to save yet — changes remain in editor”.
Reproduced on two new notes; the warning was also visible during subsequent
typing. Explicit ⌘S after adding prose reported saved. This establishes a
false failure for an ordinary empty state, not data loss or a failed disk write.
The running binary's revision was not recorded; recheck on the current build.

**Build / verify.** Treat a genuinely empty unnamed note as a quiet no-write
state while preserving real save-error notices. Read docs/platform.md. At the
live-App scheduling/persistence seam, sweep empty, whitespace-only, first prose,
delete-back-to-empty and a real filesystem failure. Assert no file litter and
no failure notice for the empty cases, successful persistence after typing,
and retained errors for genuine failures. Add a law that fails on the reported
empty-note behavior; prove the mutation builds and the test runs. Confirm the
pause/typing journey in a real window; ordinary capture has no autosave clock.

---

### 584 — document accessibility text goes stale after creating a new document (live bug hunt, 2026-09-06)

⬜ READY — user requested queueing; not dispatched.

**Observed / reproduce.** With an existing prose file open, File → New document
changed the window/document name to untitled and showed a blank page, but the
native accessibility readout still returned the old file's text. After typing
new prose, full accessibility reads returned no document value despite the
visible text. Selecting the visible sentence through accessibility failed with
“Could not find the requested text”. Query fields continued reporting values.

**Scope / premise check.** Evidence came through computer-use accessibility
reads, not a VoiceOver listening test. Later window-control errors were excluded
from the finding. Reproduce with synthetic A/B documents on a recorded current
build, and distinguish stale adapter data from tool caching before changing
product code. Read ACCESSIBILITY.md and the semantic/native bridge owners;
zero accessible TextRun children is expected and is not this defect.

**Verify.** Exercise new/open/switch/edit at colliding buffer versions, checking
document text and selection through the native text interface against the actual
buffer. Add a law at the failing publication/cache seam, mutation-proven, and
confirm the native journey. A correct semantic snapshot alone cannot prove the
OS received it. If tooling caused the mismatch, close as premise false with
the repaired oracle rather than claiming a product fix.

---

### 585 — ⌘A in Find selects the underlying document instead of the query (live bug hunt, 2026-09-06)

⬜ READY — user requested queueing; not dispatched.

**Observed / reproduce.** Open Find with a nonempty query, focus its field,
press ⌘A, then type replacement text. With query `beta`, typing `alpha`
produced `betaalpha`; another ⌘A followed by `Z` produced `betaalphaZ`.
The screenshot showed the entire underlying document selected while the
accessibility focus remained on Find. Reproduced twice in the running macOS
app, Kite; record the build when repeating it.

**Build / scope.** Select-all and subsequent editing must belong to the focused
query, preserving the parked document selection. Read docs/config.md and the
input-routing owners. Audit the neighboring Find/Replace fields and summoned
text-entry surfaces for select-all, cut/copy/paste, deletion and undo escaping
to the document. Derive the surface roster; do not count variants of this
routing defect as separate fixes. Preserve document editing when no field is up.

**Verify.** Drive the real keymap at the purest shared/live-App seam and assert
query value, query selection and unchanged document text/selection. Include
both keymap conventions and both find fields. Add the missing focus-routing
law and prove it fails with the bypass restored; repeat the visible ⌘A/typing
journey after the repair.

---

### 586 — Bold wraps trailing whitespace and produces literal Markdown (live bug hunt, 2026-09-06)

🟡 CLAIMED 2026-09-07 — lane `item-586-587`, worktree `.claude/worktrees/item-586-587`,
branched off `3cf7061b`. Bundled with 587 by that item's own instruction: they are one seam,
and splitting them across two lanes buys a handoff bug rather than parallelism. The lane
reproduces before implementing.

**Observed / reproduce.** Paste a document with a body line `hello world `
(one trailing space). Select that line's contents using line-start then
⌘⇧Right, press ⌘B, and move down. The line becomes `**hello world **` and
displays literal asterisks with ordinary text. The identical control without
the trailing space renders bold. Both arms were exercised in the running
macOS app, Kite. Preserve exact whitespace in the fixture.

**Build / scope.** Formatting a selection with edge whitespace must produce
valid emphasis while retaining the original whitespace outside the delimiters.
Read docs/markdown.md and the shared formatting owner. Audit sibling inline
formatters according to their own grammar, including leading/trailing spaces,
tabs, whitespace-only selections, line boundaries and reversed selections;
do not impose emphasis rules on code spans indiscriminately.

**Verify.** Assert exact resulting text, parsed emphasis coverage of the intended
content, selection/caret behavior, toggle and undo round trips. Use the existing
Markdown parser rather than marker presence as the oracle. Add a failing law
for the trailing-space case and mutation-prove it. Confirm the off-caret live
preview; apply standing pixel/vision checks if the implementation touches rendering.

---

### 587 — Inline Code fails to protect backticks inside the selection (live bug hunt, 2026-09-06)

🟡 CLAIMED 2026-09-07 with 586 — same lane, worktree and branch (`item-586-587`), which is
what this item's own coordination clause asks for: shared formatter edits integrate serially
under one owner.

**Observed / reproduce.** Paste a body line containing a space-separated `a`,
the literal source `` `tick` ``, and `b` (whole line: ``a `tick` b``). Select
the entire line and press ⌘E, then move down. The preview shows two separate
code fragments around `a` and `b`, with `tick` outside them. Moving back onto
the line exposes single-backtick outer delimiters colliding with the existing
inner pair. Observed in the running macOS app, Kite; reproduce on a recorded
current build before implementation.

**Build / verify.** Read docs/markdown.md. The shared inline-code formatter must
choose a delimiter run and any grammar-required padding that preserve the
selected literal content as one code span. Sweep internal backtick-run lengths,
backticks at either edge, spaces, empty selections, existing spans and toggle/
undo paths. Parse the generated Markdown and assert exactly one code span with
the intended literal payload; marker counting alone is insufficient. Add the
missing law and prove it fails when single delimiters are restored. Confirm
the live preview and route every formatting door through the same owner.

---

### 588 — list-bullet pairs derive from each world's worn ornament set (carried out of 536's fold, 2026-08-30 decision)

Item 536 assigned all 20 worlds their Nishiki ornament trios (dash/star/underscore) and
recorded, as its own clause (c), that LIST-BULLET pairs were not covered by that pass: they
still carry the pre-Nishiki vocabulary while the trio beside them moved. The decision was a
"small follow-up taste round" deriving each world's bullet pair from the set it now wears —
Genjikō for Mulga, Moonfaces for Mopoke, Gambit for Currawong, and so on down the adopted
table in 536's own history (`git log -p -- .orchestrator/queue.md`).

Mechanism is unchanged and must stay unchanged: `theme::ornament::Ornaments` is per-world
const data. Derive the roster from `theme::worlds::THEMES` (`[Theme; 20]`, Cassowary
included) rather than a hand-list — a grep over `worlds.rs` alone has already produced a
wrong count of 19 once by missing Cassowary's own module.

Laws: every world's bullet pair is drawn from the same adopted union its trio is (enrol the
union from the roster, not a named member); no world keeps a bullet from the retired
vocabulary; the pair stays legible at prose size in both grounds. The visual outcome is a
taste call owed to the user — deliver a gallery capture across the roster, not an argument.

### 589 — Commands and shared transient chrome: clearer controls within each world's composition (user decision, 2026-09-07)

⬜ READY — queue only; not dispatched. Shared design foundation for 590–592;
integrate overlapping renderer work serially.

**Decision.** Improve Commands' query/result hierarchy and spacing while keeping
the document readable outside the summoned surface. Integrate its title with
its controls rather than letting a remote oversized label dominate the task.
Preserve categories, bindings, keyboard selection and each world's authored
placement; the generated upper-right mockup is not a universal anchor.

**Shared scope.** Consistency is WITHIN each world, not one skin across worlds.
Find, Link and Commands share that world's surface colours, border weight,
corner rules, field grammar and spacing. Nested controls derive compatible
corners; do not invent feature-specific radii. Preserve Pane, Bars, Diagonal
and Ruled compositions: no compulsory rounded enclosing panel for plate/rule
worlds. Carry relevant improvements through sibling pickers/prompts via shared
owners. The user's preferred Find/Replace chrome is preserved in
`references/find-replace-chrome.png` beside this board; it guides bordered
surfaces, not every world's visual identity.

**Separation / verify.** Brief choices should retain readable surrounding prose.
Use the world's backing, retaining local frost where needed; this is NOT a
global blur-off instruction. Unbacked text must not overlap document ink.
Read DESIGN.md, docs/render.md and docs/harness-reach.md. Audit the actual
surface × composition × placement roster, narrow/wide and DPI 1/2; assert
geometry/state and pixel legibility, add mutation-proven laws and the standing
five-shot vision smoke. Keep anchor stability and keyboard behavior intact.

### 590 — Insert Link: a clear URL field with keyboard-first commit (user decision, 2026-09-07)

⬜ READY — queue only; coordinate shared chrome with 589.

**Decision.** Replace the empty imitation list in Insert Link with an obvious
destination field: readable `Link destination` label, `Paste or type a URL`
placeholder, immediate typing, Enter to commit and Esc to cancel. Keep a quiet
clickable commit affordance carrying its resolved binding. Preserve existing
URL prefill, selected-text wrapping, editing an existing link and undo behavior.

**Composition.** Keep the existing world/context placement policy, including
clamping on small windows; the generated below-paragraph location is illustrative,
not a new hardcoded rule. The user likes the relationship between Find and Link
chrome, with consistent borders/corners inside a world. Apply 589's world-specific
surface grammar and backing policy rather than shipping one generic rounded
dialog. Keep surrounding prose readable; no full-viewport blur merely to enter
a destination. Retain appropriate local separation where a world's composition
otherwise interleaves text with the document.

**Verify.** Read docs/markdown.md, docs/render.md and docs/harness-reach.md.
Test empty/prefilled/existing-link/selected-text paths, keyboard and pointer
commit/cancel, focus and document restoration. Native keyboard labels come
from the real keymap. Sweep composition families, anchors, narrow widths and
DPI 1/2; pixel-check label/field clarity and no clipping. Add the missing laws,
mutation-prove them, and include the standing vision smoke. Report final feel
as requiring the user's live eye, not as proven by image generation.

### 591 — Find/Replace: preferred bordered chrome, keyboard discoverability, existing top-right placement (user decision, 2026-09-07)

⬜ READY — queue only; coordinate with 589 and the focus-routing repair 585.

**Authoritative reference.** `references/find-replace-chrome.png` is the crop
the user explicitly preferred AFTER the keyboard-first remake. Preserve its
clear bordered fields, subtle lavender surface, thin separators, separate
match/navigation region and distinct Replace/Replace all controls where the
world uses bordered panels. The later borderless text-strip mockup is NOT the
selected chrome. Derive compatible corners from the world's shared treatment.

**Keep keyboard character.** Controls remain clickable, but visible shortcut
labels teach the existing behavior: replace/next, replace all, switch field,
close and match case. Source bindings from the active platform/keymap; no
hardcoded macOS glyphs on Linux. Distinguish labels, editable values, match
count and actions without tiny crowded hints. Find-only remains compact.
Keep the EXISTING top-right placement and safe inset, close below the title/menu
bar; the first mockup's large top gap was rejected. Do not adopt its enlarged
footprint blindly. Keep matches and surrounding prose readable.

**Verify.** Preserve search/replace semantics and focus, including 585's query
select-all law. Read docs/config.md, docs/render.md and harness-reach; exercise
both fields and actual command bindings. Capture empty/no-match/multiple-match,
find-only/replace and case states across world compositions, widths and DPI
1/2; assert bounds, shortcut/action correspondence and pixel legibility. Add
mutation-proven laws and the five-shot vision smoke. Theme identity remains
data through shared renderers, not one universal screenshot skin.

### 592 — Settings: compact label/value relationships and readable workspace hierarchy (user approval, 2026-09-07)

⬜ READY — queue only; coordinate shared chrome with 589.

**Approved direction.** The user strongly prefers the new Settings layout:
modest nearby title and identifiable search, category rail beside the active
category's controls, comfortable row spacing, and a bounded detail-column width
that keeps values close to labels. Extra window width becomes breathing room,
not a longer journey between a setting and its value. Remove the remote giant
SETTINGS label in favor of the integrated hierarchy. The reference's proportions
are the direction, not hardcoded pixel coordinates or replacement control semantics.

**Worlds / interaction.** Preserve the existing category/detail focus model,
selected-row control interaction, query, return path, exact editor restoration
and narrow staged presentation. Express rows, selection, corners and backing
through each world's Pane/Bars/Diagonal/Ruled vocabulary. Keep relevant key hints
near the active control, using real bindings. Start with a quiet opaque themed
workspace ground rather than ghost prose; retain frost only if it contributes
to that world's authored composition. No blanket removal of ambient effects.

**Verify.** Read DESIGN.md, docs/render.md and harness-reach. Sweep category,
control kind, focus region, composition family, narrow/wide window, zoom and
DPI 1/2. Assert label/value proximity, usable controls, no clipping, correct
focus/selection and unchanged setting behavior. Validate appearance with pixels
and the standing vision smoke; add mutation-proven laws at shared seams. Final
theme-specific composition remains a live taste review.

### 593 — `sweep.sh 1` prunes every worktree's `target/`, including the ones being built in (found by 567's lane, 2026-09-07; measured, not inferred)

⬜ READY — high priority: this defect corrupts concurrent lanes, which is the exact
configuration the orchestration layer is built to run.

`scripts/sweep.sh` is the disk preflight's sole deletion owner and
`.orchestrator/worker-build.sh` fires the preflight on EVERY lane command. The sweep
traverses and prunes every worktree's `target/` — not only the caller's — so a lane that
merely starts a build deletes fingerprint files out from under a sibling lane's live
compile. The victim dies on `failed to write …/.fingerprint/<crate>/invoked.timestamp`
(ENOENT), which reads as a broken build rather than as another process's deletion.

**Measured, not argued.** 567's lane instrumented one attempt with a `ps` sampler: a
`sweep.sh 1` launched from `.claude/worktrees/item-586-587` ran 00:51:33–00:52:03 while the
lane's own build compiled, and that build died on `harfrust-…/invoked.timestamp` immediately
after. The cost that round was six runs — the native gate went green on attempt 2 and
web-smoke on attempt 7.

The trigger is a low-disk condition, so the failure is bursty and looks flaky: while free
space sits under the preflight's floor, every lane command fires a sweep, so an active wave
has its lanes reliably corrupting each other. One honest early refusal was also observed:
`disk-preflight: insufficient space after sweep-1d; free_bytes=25336659968
minimum_bytes=25769803776`.

Mitigated but NOT fixed 2026-09-07: the orchestrator reclaimed the stale `target/` trees of
every worktree with no live build (26 GB free → 131 GB), which stops the trigger firing
today. That is headroom, not a repair — the next wave that fills the disk reproduces it.

Build: the sweep must never delete inside a worktree other than its caller's, or must take a
lock every builder respects. Prefer the narrow fix. Laws: a sweep launched from worktree A
leaves worktree B's `target/` untouched; the law fails when the traversal is widened back to
all worktrees. Note the deleted `test-sweep.sh` was one of 567's eleven unwired laws — this
subject has just earned a law again, so per that item's own recorded decision it gets wired
into `code-health.sh` at birth rather than left to rot.

---

### 594 — the recurring `scripts/__pycache__/` has two unguarded creators, and the guard that was supposed to stop it cannot (found by 567's lane, 2026-09-07; premise of 567's rider (a) falsified)

⬜ READY — small, and it closes a false premise rather than leaving it in the tree.

Item 567's hygiene rider (a) added `PYTHONDONTWRITEBYTECODE=1` to `code-health.sh`'s
`python3 scripts/code-health.py` invocation to kill the recurring `scripts/__pycache__/`.
The flag is harmless and worth keeping as parity, but **its stated motive is false and the
directory will keep coming back:** `code-health.py` runs as `__main__` and imports no local
module, so CPython can never write a `.pyc` for it. Confirmed empirically — a full
`code-health.sh` run produced no `scripts/__pycache__`.

The real creators are the `importlib.util.spec_from_file_location` loaders at
`scripts/test-native-test-shards.py:14`, `scripts/ground-contrast-measure.py:91`,
`scripts/ambient-motion-measure.py:35`, and `scripts/test-native-gate.sh:60`.
`test-native-gate.sh` already guards its two (lines 6 and 55). The two `*-measure.py`
scripts are unguarded, and both are on 567's explicit keep-list — invoked from
`render/tests/deckle_ground.rs` and the `capture-*.sh` pair.

Build: guard the two unguarded loaders at their own call sites. Law: after a run of each
consumer, no `scripts/__pycache__` exists; prove it non-vacuous by removing a guard and
watching the directory come back. Rider: correct rider (a)'s comment where it states the
motive, so the next reader is not taught the wrong mechanism.

### 595 — an `overlay_hover_stability_law` failure appeared on one gate arm, once, and could not be reproduced (found by 568/569's lane, 2026-09-07)

⬜ READY — small, but it is in the class this repo has been bitten by repeatedly.

`render::tests::overlay_hover_stability_law::a_deliberate_world_crossing_can_move_a_stationary_
pixels_hit_test_row` went red on the `linux` arm ONLY during one gate run, was **absent from
that arm's own `failures:` list**, was green on `mac` and `menubar-full` in the same run, and
was green on all three arms in the next. The lane could not reproduce it targeted and
recorded it as unexplained rather than asserting it benign, which is the right call.

Why it is worth a look rather than a shrug: it is a `render::` law reaching the shared test
GPU, which is exactly the order-sensitive class CLAUDE.md names. One device is one object
population and one set of wgpu-hal counters, and a test that merely borrows a handle mutates
them; the documented signature of an unguarded reach is a law that **passes alone, passes
unfiltered, and fails only under a filter** — never failing CI and always failing a developer.
Its disappearance from the arm's own failures list is itself a finding: a red that the
receipt's own summary did not carry.

Build: establish whether this law (and its neighbours in that file) take
`crate::testlock::serial()` and hold it for the LIFETIME OF THE RESOURCES rather than the
call — a `TextPipeline` dropped at the closing brace still moves the counters, so a lock a
helper takes and returns discharges nothing. Then either fix the enrolment or explain the
one-off. Laws: whatever is found, prove it by making the failure deterministic before
declaring it fixed.

---

### 596 — two small truths about the personal dictionary that its own docs get wrong (found by 568/569's lane, 2026-09-07)

⬜ READY — trivial, filed so they are not lost between a merge and a board compression.

(a) `REFERENCE.md` says the dictionary file "is read at startup only". It is also re-read when
the dictionary variant switches (`set_dictionary` → `load_user_dictionary`). A generated
reference stating a wrong answer with a roster behind it is the documented hazard — the fix is
the sentence, and the check is asking the property on both sides of the condition.

(b) `remove_word_from_dictionary_file` joins with `\n`, so a CRLF-edited word list is converted
to LF by a removal. Unreachable on awl's shipped platforms and therefore not urgent, but it
contradicts the file-preservation promise the same function otherwise keeps, and the rope's
whole CRLF discipline is "load normalizes, save restores".

## Owed to the user — landed work awaiting a live eye

These items have MERGED and left the build queue. Each one still owes the user an answer or
a live look, which landing does not discharge. Full context is in
`git log -p -- .orchestrator/queue.md`.

**568 — personal spell suggestions (gated on `item-568-569`, merge pending).** Two decisions in
`src/spell/personal.rs` are the user's to confirm, quoted from the source rather than from a
commit message: `pub(super) const MAX_DISTANCE: usize = 2;` and the ranking rule that "a
personal near-miss is the user's OWN vocabulary, added deliberately, so it must never lose a
slot to a bundled guess". **The risk worth naming:** the fix offers a personal word only when
the typed word is already flagged misspelled AND within 2 edits. If what the user actually did
was type a PREFIX — `Zorb` for `Zorbling`, four edits away — this does not reach it, and the
standing "we don't need autocomplete" decision makes that deliberate. Worth asking before 568
is called closed.

**551 — table selection band (merged `f740749c`, follow-up `db90497e`).** The band now paints
whole rows. If a spreadsheet-style cell-wise selection is what you actually wanted, say so —
that alternative was flagged, never built.

**553 — folder-wide search (merged `277c3717`, follow-ups `e076ddd8`/`104fb174`).** The match
highlight's real-pixel legibility is live-only and unverified. Also flagged, not hidden:
grouping does not use the lens-strip header mechanism (a deliberate scope call); a CRLF
source file's matched line keeps a cosmetic trailing `\r`; and the corpus is summon-time
only, like Assets and Go to — a file edited on disk while the picker stays open is not
re-read until the next summon.

**559 — close mark hover (merged with 550 as `347eba64`).** Keep the existing hand cursor, or
switch the whole row to arrow-plus-hover-only to match the cited convention? Hover is
pointer-only and undrivable by `--keys`/`--screenshot-app`, so the resting geometry is
capture-verified but the feel and the cursor question are yours.

**561 — ornament scale equalized upward (merged `5f90cb6d`, follow-ups `1b22a1c1`/`fd2f5894`).**
Gumtree's dash is a 4-glyph snake run, so equalizing its height also grew its width (~119px →
~252px against a 1008px column); it reads proportionate in capture, unconfirmed live.
Unmeasured: star and underscore share one `ornament_scale` dial with dash, so they grew
proportionally without being checked against their own ink-to-em ratios.

**564 — Kite's living warped-grid tunnel (merged `c3c3032e`, cleanup `002f09fe`; pushed).**
Live human sign-off is owed for the several-minute drift and contortion feel — the harness
verifies single-frame trajectories and the motion-safe still, not wall-clock feel over
minutes. Also owed: at the default 1200×800 capture geometry the roaming vanishing point can
land closer to the page edge than at the 1600×1000 geometry the pixel laws sweep, so it is
worth a live look at whether the convergence ever reads as landing inside the page itself at
common window sizes rather than staying a margin phenomenon. Item 582 (open, above) revises
this ground's geometry and inherits the same sign-off.

## Watch — verification that only a future run can supply

**566 — the CI wiring's own oracle (repair merged `964bba03`).** The next `linux (build +
test)` run's `native-gate-env` line must read `budget_source=deadline`, not
`budget_source=none`. Nothing local can test the `$GITHUB_ENV` hop, so this stays open as a
thing to read off the next run rather than as work to do.

## Needs specific hardware

🔴 BLOCKED — these journeys require physical environments unavailable to the current orchestration host.

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

🔴 BLOCKED — release work requires the user's explicit release word and Apple signing secrets.

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
