# item 444 residual 3 — overflow/grouped working-set prototype gallery

`sh captures/item-444-residual3/shoot.sh` from the repo root regenerates the shots.

Hermetic: the sandbox is seeded from `fixture/` alone, through `--seed-tree`,
with an explicit `--config` and `--root` — never the ambient project or the
ambient config — so nothing here photographs a real directory. The PNGs and
their sidecars are scratch and are not committed; the fixture and the script
are, so the set survives the worktree that produced it (mirrors
`captures/item-444/README.md`'s own note).

## What this gallery is

Residual 3 of item 444 (`.orchestrator/queue.md`) is marked
**AWAITING USER CHOICE**: the resting-stack overflow windowing rule and the
cross-project grouped view need a screenshot audition before either becomes
a shipped feature. This gallery is that audition. It is **not** the shipped
feature — no production render or hit-test code changed to produce it.

## What this gallery does and does not demonstrate

The data model (`src/workingset/prototype.rs`: `PrototypeSpec::{Collapsed,
Expanded, Grouped}`, `RESTING_FILES = 5`, `EXPANDED_FILES = 8`) and its
capture-only door (`AWL_WORKING_SET_PROTOTYPE` env var, read only by
`PrototypeSpec::from_env()` under `--screenshot-app`, exposed in the sidecar's
`buffers.prototype` block) were already built and law-tested before this
round (`src/workingset/prototype/tests.rs`).

The open question going in was whether the **render layer** could draw those
projected rows at all — `render/plan/margin.rs`'s `plan_gutter_stack` was
known to lay out a plain bottom-anchored block with no scroll/viewport or
grouped-heading treatment. **It turns out it already can, with zero new
code.** The existing margin block (`render/chrome/gutter.rs`,
`render/chrome/gutter_stack.rs`) draws whatever `StackRow` list the working
set hands it — file rows, the one `More` row, `Group` heading rows — through
the SAME `plan_gutter_stack` bottom-anchored stack it always used, with
`gutter_stack::stack_spans` already inking a `Group{active}` heading in
`muted` and everything else in `faint`, and already withholding the trailing
close-mark lane from non-`File` rows. Piping the prototype's projected rows
through `opts.working_set` (already wired in `App::capture_opts`,
`src/app/capture_state.rs`) was enough to produce every shot below.

**What is genuinely NOT built, and this gallery does not claim otherwise:**
click/hit-test support for a `More` row, live scroll, or a cross-root
activate-on-click. `render/chrome/gutter_hit.rs`'s `stack_hit_from_plan`
still explicitly returns `None` for any row whose `StackRowKind` is not
`File` — a click on `+N more…` or a group heading does nothing in the live
app today, by design (the queue item asks for the screenshots first). Every
shot below is driven by setting the sealed env vars directly
(`AWL_WORKING_SET_PROTOTYPE[_SCROLL|_HOVER]`), never by a real pointer
gesture — the brief's own "whichever gets you real pixels fastest" clause.

## The fixture

`fixture/` is a two-root project tree: `workspace/notebook/` (ten files, one
nested under `journal/`) and its sibling `workspace/atlas/` (three files, one
nested under `src/`). No prior item-444 fixture had more than four files or
more than one root; this one exists so the prototype has a real >5-file,
multi-root working set to project instead of a hand-built struct.

`fixture/awl.toml` sets two things beyond the theme:

- `file_visibility = true`. **Load-bearing, not cosmetic.** The
  Switch-project picker's folder roster (`overlay::goto_folder_roster`) is
  filtered by `crate::index::is_hidden_entry` against the row's *absolute*
  path — and a worktree checked out under a dot-directory (this one lives
  under `.claude/worktrees/...`, and some CI temp dirs do too) has a
  dot-prefixed ancestor in every absolute path it can produce. Without this
  flag the picker silently loses every sibling folder row, `Cmd-Shift-P`
  shows only "Choose another folder…", and no cross-root shot is reachable
  from this checkout. Harmless when the checkout path has no dot ancestor.
- The ten/three file counts and the two root names (`notebook`, `atlas`) are
  otherwise arbitrary but deliberately distinct-prefixed
  (`opening/ledger/ideas/todo/draft/plan/review/archive/index/entry`) so a
  short fuzzy query in the Go-to picker's **Files** lens resolves each one
  unambiguously.

## The shots

The working set behind every shot is the same 13-file, two-root set: all ten
`notebook` files opened first (stable order), then `atlas`'s three files
reached through a real `Cmd-Shift-P` project switch. Only the *last*
reactivation step (and the theme, and the prototype mode/scroll) differs
between shots.

| shot | mode | what it shows |
| --- | --- | --- |
| `collapsed-opening-active.png` | collapsed | Resting stack: `notebook`'s first five files (`opening.md` active, at the top of its own group) + `+ 8 more…`. |
| `collapsed-entry-active.png` | collapsed | Same 13-file set, but the active file is `journal/entry.md` — the LAST slot in `notebook`'s group. The candidate windowing rule slides the five-row window to keep it visible (`plan.md … entry.md`) instead of always pinning the first five. |
| `collapsed-jitter.png` | collapsed | Same set again, activating `plan.md` right after `entry.md` — i.e. a file that was ALREADY the top row of the previous shot's window. See "A windowing finding" below: the row the reader was just looking at jumps to the opposite end of a four-slot-shifted window. |
| `collapsed-atlas-active.png` | collapsed | Switched to `atlas` and stopped there (3 files, no cross-root reactivation). Demonstrates the OTHER half of the rule: only the ACTIVE root's group shows at rest (3 rows, no overflow needed within this root), while `+ 10 more…` still counts every hidden buffer across BOTH roots — the one generic overflow row the queue item specifies, not a bug in the arithmetic. |
| `expanded-scroll0.png` | expanded | The 8-row scrollable window over `notebook`'s 10 files, unscrolled (`opening.md … archive.md`). |
| `expanded-scroll2.png` | expanded | The same window scrolled by 2 (`ideas.md … journal/entry.md`). Note the active file (`opening.md`) has scrolled out of view — the prototype's expanded mode does not clamp scroll to keep the active row visible the way the collapsed mode's window does; that is a real open question for the windowing rule, not a defect in this gallery. |
| `grouped-saltpan.png` | grouped | The cross-project view: every open file under BOTH roots, each headed by its root's name. `notebook`'s heading reads in the brighter `muted` ink (the active group); `atlas`'s reads `faint`. No truncation — all 13 rows draw, which is itself a data point: at this file count the grouped view already runs to 15 lines (2 headings + 13 files) with no scroll of its own. |
| `grouped-magpie.png` | grouped | The identical grouped view under the `Magpie` world (a light horizontal-band ground), for contrast against the default. |
| `collapsed-gumtree.png` | collapsed | The resting-stack view under the `Gumtree` world (a diagonal zigzag ground), for contrast against the default. |

## A windowing finding

The candidate rule in `prototype_collapsed` is **stateless**: it re-derives
the five-row window from nothing but the active file's index in the group
(`start = active_index.saturating_sub(4).min(max_start)`), never from the
PREVIOUS window. `collapsed-entry-active.png` and `collapsed-jitter.png`
both include `plan.md` — top row of the window in the first, bottom row of a
window that has shifted four slots down in the second — even though
`plan.md` was already visible on screen the whole time and nothing forced it
off. A window that instead preferred to hold still when the newly active
file is already inside it would show `plan.md` in the same row across both
shots. This is exactly the kind of behavior the queue item asks the user to
judge before it becomes the shipped rule, not a defect in this gallery.

## An incidental finding, not chased further

A `--theme Potoroo` probe of the same collapsed view (not included in the
committed gallery) showed an oversized gap between the active row's plate and
the row below it that the default (`Saltpan`) and `Magpie`/`Gumtree` shots
above do not have — plausibly a metrics interaction specific to Potoroo's
monospace chrome font. Worth a follow-up look if Potoroo is ever chosen for
this surface's own audit rotation; out of scope for this prototype pass.
