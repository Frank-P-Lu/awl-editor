# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 515 — one plate, one meaning: the working-set panel plates the current project AND the active file at once (user-confirmed confusing, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524 (sequenced with 518/520/521/522/524 on one branch, per the coordination note under 522)

Screenshot evidence on Kite: the expanded panel drew TWO purple plates —
the `notes/` group heading (plated because it is the current project,
`workingset/panel.rs` Group arm + `gutter_stack.rs::plate_rects`) and the
`scratch` file row (plated because it is the active buffer,
`workingset.rs::file_row`). The code itself names these as two different
questions ("which file" / "which project", `gutter_stack.rs` doc) but
answers both with the same treatment in the same column, and the gutter
block's own folder heading ALREADY names the current project directly above
the rows — so "you are in notes" is stated twice and the double plate reads
as two selections. User: "it's confusing as heck? like we already have
'notes' at the top no?"

DECIDED (user-confirmed 2026-08-29): **the plate means the active file,
and nothing else.** A group heading that is the current project keeps its
`active_ink` distinction but loses its plate; the project identity is
stated ONCE — by the gutter folder heading, or, once item 521 lands, by
the ink-marked group heading in the list (521 removes the separate gutter
line whenever group headings are drawn). The lane enumerates every `plate_rects`-family
consumer rather than patching the one arm the screenshot showed (the module
doc also names a "bottom identity" plate — same sweep, same one-meaning
rule judged against it). Cheap to revert (render-side row treatment; no
state change), so per the standing taste policy: land on main and await
feedback, revert cost stated in the commit.

Law shape: at most one plated row per frame across the resting stack and
the expanded panel, and when one exists it is the active FILE row — swept
across worlds (Wagtail's page-inverse plate arm included), with non-vacuity
proven by re-plating a heading and watching it go red. Coordination: group-
row presentation is also touched by 507/512's folder-identity work — same
surface, keep the label conventions theirs and the plate rule this item's.

---
### 516 — closing the scratch shows a developer-register dead-end notice (user-reported, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-516

⌘W on the active scratch with unsaved text shows `save failed: no file
bound to this buffer (scratch)`. Mechanism, verified in code: the close
gate (`app/daemon.rs::try_save_finished_buffer`, reached from
`files/close.rs`) calls raw `document.save()`, and a true scratch bails in
`buffer/save.rs` with an anyhow string written for developers — which
lands VERBATIM in the sticky notice. Two defects in one cell:

(a) **Voice.** "buffer" is not a user word (user: "'buffer' is too
technical"), and the parenthetical reads as debug output. `close.rs`'s own
module doc states the principle this violates: "a notice describing a
state with no exit is a dead end." The PARKED-scratch arm already answers
in product voice ("scratch has unsaved text — open it before closing");
the ACTIVE-scratch arm is the gap. DECIDED: the refusal notice names the
exit in product voice — the route that already exists is ⌘S, which
promotes scratch into a real note (`verbs.rs::convert_scratch_and_save`,
decided behavior). Internal error strings may stay for logs but never
reach a notice untranslated; sweep the other `save failed: {e}` notice
sites for the same leak class.

(b) **Flow, DECIDED (user, 2026-08-29): closing scratch just closes it.**
Scratch is a PLACE, not a document ("we sorta have an empty screen right?
so maybe it's okay to close it") — and the mechanism already agrees: the
autosave engine stashes scratch's full text to the persistent stash
(`autosave.rs::stash_scratch_now`, idle/blur/quit, with its own history
ladder) and a bare relaunch restores it (`App::new`), so a close discards
NOTHING. The refusal existed only because the close gate routed scratch
through file-save machinery it was never subject to. Fix shape: on close
of the active scratch, flush the stash first (`stash_scratch_now` is the
existing owner — the close must not race the idle debounce), then dismiss
silently; the parked-scratch refusal arm ("open it before closing")
dismisses the same way. ⚠️ Precondition the lane verifies: an in-session
door BACK to scratch must exist once it is closed (the relaunch restore is
not enough — a closed scratch unreachable until restart is a trap); if no
summon door exists, build the smallest one or the close stays refused with
(a)'s voice fix. A stash write failure refuses the close in product voice
(the text's only copy is at stake — same rule as `save_parked`).

Law shape: a `--keys`/`--screenshot-app` journey closing a text-bearing
scratch asserts it closes with no notice and the stash holds the text;
resummon restores it; plus a voice law over user-facing notices (no
"buffer") at whatever seam the notice strings can be enumerated.

---
### 517 — Insert table: tables render as grids but nothing creates one (user request, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-517

"I think we want a table creation command." On-thesis: the committed
direction is finishing the live-preview model — "tables as real grids —
through the markdown formatting commands." Rendering exists
(`prepare_table_grid`, the table x-ray reveal, docs/markdown.md); creation
does not: no `InsertTable` action, so a writer must already know raw `|`
syntax to ever see the grid — exactly the knowledge the formatting
commands exist to remove (the popover's own law: no raw markdown in
chrome).

Shape: a catalog Action + palette entry ("Insert table…"), routed like
every formatting command (keys → Action → apply_transition, drivable by
`--keys`, visible in the sidecar). DECIDED (user, 2026-08-29): a creation
dialogue — a small summoned DIMENSION PICKER, keyboard-first with mouse
support. Form: a drawn mini-grid the arrows sculpt (`↑/↓` rows, `←/→`
columns) with the chosen `R × C` read out beside it, `↵` inserts, `Esc`
cancels; typed digits also accepted (a forgiving `3x4` / `3 4` parse);
the pointer picks by clicking a cell of the same drawn grid — one
geometry for arrows, readout, and clicks, so drawn and clickable cannot
disagree (the rowlayout discipline). Modest default (e.g. 3×2) so bare
`↵` is already useful. Insertion: header row + separator + body rows on
their own blank lines at the caret, caret landing in the first header
cell. Follow-ups deliberately OUT of this item: Tab walking cells inside
a table; popover/context-menu exposure (the popover roster is a locked
seven — a separate decision).

---
### 518 — the expanded panel's window can orphan file rows from their group heading (found decoding the user's screenshot, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524

`expanded_rows` (`workingset/panel.rs`) windows `expanded_full` with a
plain slice, so when the scroll starts mid-group the visible file rows
carry no group context — the user read three files from a DIFFERENT root
as belonging to "notes" because that group's own heading was scrolled off
above the window and the gutter's folder heading ("notes") sat directly
over them ("so why is there a second notes group? like i assumed
everything was in notes"). Fix shape: the standard sticky-heading answer —
when the window's first row is a File, pin its group's heading as the
window's first drawn row (costing one viewport slot, same as any drawn
heading), so every visible file row's group is nameable from the drawn
window alone. Law: sweep scroll positions over a multi-group set and
assert each visible File row's group heading is drawn in the same window;
non-vacuity via the pre-fix mid-group scroll. Related conventions: the
scroll/position-indication work that closed as item 508, and 507/512's
group-label rules — reuse, don't fork.

---
### 519 — the Go-to lens strip hugs the card's top edge (user-reported, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-519

Screenshot evidence on a dark mono world (split query card, amber
selection bar): inside the main card, the lens strip (`All Files Headings
Folders Recent`) sits tight under the card's top edge — visibly less
breath above it than the generous air below it before the section
heading, so the strip reads clipped against the rim ("there's not enough
visual padding above the all files navigate etc"). Probe first (a defect
report is a hypothesis): measure the strip band's top inset against the
card's other band pads across compositions — split query card vs
in-card query header, faceted vs flat — and against the same strip on
Kite (where the strip renders fine), to find WHICH composition arm loses
the pad and which owner should carry it (`overlay_head_left`/band pads
neighborhood; item 505's mark-seat delta is a reminder this area has
per-composition seats). Fix at the one owner, not per world. Law: pixel
arithmetic — the strip's ink-band top inset clears the same floor the
card's other bands get, swept worlds × compositions × 1x/2x; non-vacuity
by re-shrinking the pad.

---
### 520 — the expanded working-set panel gives no sign of overflow above (user-reported, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524

Screenshot on Potoroo: the expanded panel scrolled to its bottom shows a
plain first row — nothing says more items exist above the window ("i've
scrolled down to the bottom... there's no indication that there's more
items above"). The vocabulary already exists elsewhere: the resting
stack's own `+ N more…` row (`workingset.rs::stack_rows`) and the Go-to
card's `↓ N more` line — reuse that convention, don't invent a third
(same behavior ⇒ same code: one owner for "this list continues" if the
existing two can be merged). Shape: when `scroll > 0`, the window's first
slot carries `↑ N more`; when rows remain below, the last slot carries
`↓ N more` (verify whether the downward case is also missing here or
already handled). Coordinate with 518 (sticky group heading wants the
same top slot — decide the stacking: the overflow line and the pinned
heading must not fight for one row; likely overflow line first, heading
second, both costing viewport slots). Law: sweep scroll positions over a
set larger than the viewport and assert the indicator's presence/count at
both ends, pixel-verified on Potoroo (its stripes are a known pixel-
oracle trap — sample inside the drawn row band, not the ground).

---
### 521 — the gutter's standalone project label duplicates the drawn group headings (user decision, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524

Screenshot on Potoroo: the block reads `work` (the gutter folder heading)
directly above a list whose own headings — `notes/`, plated `work/` —
already carry the structure. User: "the first work... that's our current
project right, I think we get rid of that... notes and work look like
they are some headings anyways." DECIDED: **when the stack draws group
headings, the separate gutter folder-heading line is not drawn — the
headings ARE the structure, and the current project is the ink-marked
heading among them** (515's ink rule; the plate stays the active file's).
When NO group heading is visible — the single-file identity line, or a
resting stack showing only the active root's files — the gutter folder
heading remains the one project label. Net law: exactly ONE visible owner
of the project name at any time, never two. The heading/identity stacking
lives in `render/chrome/gutter.rs::lines` + `gutter_stack`; sweep both
shapes (resting/expanded) × single/multi-group × worlds. Cheap to revert
(render-side block composition): land on main per the standing taste
policy, revert cost stated in the commit.

---
### 522 — group headings are not closable; closing a group means closing its files (user request, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524

"I want to be able to close work..." — file rows already grow a hover ×
(`gutter_stack::CLOSE_MARK_TEXT`); a group heading offers nothing, so
retiring a whole project from the working set is one close per file. Shape:
the same hover × on a group heading closes every file in that group —
**as a fold of the ORDINARY per-file close**, each file through the
existing save/conflict gate (`files/close.rs`), stopping at the first
refusal with that file's own notice (never a new bulk-discard path; the
gate's guarantees are the product). A parked scratch in the group follows
516's rule (dismiss, stash intact). If the active file is in the closed
group, the existing successor logic decides what's next. Hit-test rides
the shared rowlayout geometry the file rows' × already uses — one owner,
no second close-lane mechanism. Laws: a multi-group journey closing a
clean group (all gone, working set intact elsewhere), a dirty-file group
(stops at the refusal, prior files closed, notice names the file), and
drawn-equals-clickable for the heading's × across worlds × both panel
shapes.

Coordination for 515/518/520/521/522: five open items now touch the
working-set stack's row plan and hit-test. Dispatch as one lane or a
strict sequence on one branch — parallel lanes here would merge-conflict
on every file they share.

---
### 523 — followable spans get the quiet underline (user decision, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-523

The URL taming (landed as item 511) removed the one thing that made a
bare URL self-identify — the scheme costume — so a tamed `r.goope.jp…`
reads as prose; a concealed `[text](url)` link has always had the same
problem. User: "they kinda don't look like links... make it like
underline or something." DECIDED: **one grammar for every followable
span** — named links and tamed bare URLs both draw a quiet baseline
underline. Form: a hairline in a muted step of the text's OWN ink —
structural, not hued, so the one-accent law holds; SOLID ink, never a
translucent wash (Wagtail's `decorative_wash: Off` bans low-alpha
treatments — the nit-underline already learned this). Distinct by
position and shape from the other line-marks: spell is wavy below,
strike is through the middle, link sits at the baseline. One owner for
the band geometry (the `strike_line_band` precedent), `Logical` units
(the DPI lesson). Lane decides with a recorded reason: whether the
underline persists or drops while the caret's line reveals raw markdown.
Laws: presence + geometry swept worlds × 1x/2x, a differential arm (non-
link text never underlined), and docs/markdown.md updated.

---
### 524 — the reserved close-mark lane dents the stack's right edge on file rows only (user-reported, 2026-08-29)

🟡 IN PROGRESS — claude, branch ws-stack-515-524

Screenshot on Potoroo (resting stack): `notes` (heading) ends flush
right while `fukushima-trip.md` stops visibly short of it — file rows
reserve the trailing `×` close lane at ALL times (`gutter_stack::
fit_rows` / `gutter.rs`, reserved so the name never reflows when the
hover × appears — sound reasoning), but headings don't, so the two row
kinds disagree about where the right edge IS and the ragged edge reads
as misalignment ("the spacing for the x button is always there? but it
looks so odd?"). Fix shape: UNIFORM reservation — every row kind in the
stack reserves the same trailing lane, which item 522 half-delivers
anyway (headings grow their own hover ×, so they need the lane too);
verify the single-file identity line (which also reserves, per
`gutter.rs`) joins the same edge. The no-reflow-on-hover property stays.
Law: every drawn row's right ink edge within one advance of every
other's, swept row kinds × both panel shapes × worlds; non-vacuity by
un-reserving one kind. Same lane as the 515/518/520/521/522 working-set
batch (this is the sixth item on that one surface).

---
### 525 — start screen: equal ink + chord hints now; per-world dress later (user decision, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-525

Today's start screen draws its two actions in different inks —
`New document` in `base_content`, `Go to` in `theme::muted()`
(`render/chrome/start.rs::prepare_start_surface`) — and the muted one
wears the universal disabled costume ("why are the two buttons
differently coloured?"). DECIDED: the minimal repair — both actions in
the SAME full ink, hierarchy carried by order alone, each with its chord
beside it in the established footer-hint grammar (`↵ New document ·
⌘O Go to` — quiet chord, full-ink verb) so they read as commands, not
buttons. Verify the drawn hit rects still match (`start_rows` is the
shared geometry). Cheap to revert (one function's ink + label shaping):
land on main per the standing taste policy.

FLAGGED for a later design session, deliberately not scoped here: the
user wants per-world start screens eventually ("each theme can have a
starting screen that suits them... stylise it later"). Constraint to
carry into that session: **no theme may need its own code path** — any
per-world start expression is authored RenderCaps-style DATA through the
one start renderer (the backgrounds already prove the pattern), never a
per-world start module.

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
