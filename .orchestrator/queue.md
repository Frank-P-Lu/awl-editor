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

DECIDED (orchestrator, cheap to revert — one function): fold the current
root-relative destination into the card title itself rather than adding a
separate breadcrumb widget, extending `OverlayState::title()`
(`src/overlay/state.rs`) — the one existing owner that already builds
`"move welcome.md"` — to append the browse-relative folder when
`browse_dir` is `Some` and non-root (e.g. `"move welcome.md to notes/drafts/"`),
unchanged when at root. All three navigators (Move/Export/ProjectBrowse)
route through this one function already, so no forking. Revert cost: one
function in `overlay/state.rs`.

Item 507 (folder identity + current-item indication elsewhere) follows the
same convention once its surface is identified: the existing trailing-`/`
rule in `row_display` (`row.is_dir` / `RowMeta::GotoFolder`) for identity,
and the existing `active: bool` marker already carried by `StackRow` /
panel rows for current-item indication — reuse both rather than inventing a
third.

---
### 507 — an opened folder's row reads as a file and gets no current highlight (user-reported, 2026-08-27)

The user opened the `syntax` folder and, in the right-anchored list, its row
shows no folder identity (no trailing `/`, unlike Go-to's Folders-lens rows
which carry one by law) and no indication it is the current/open one: "it's
kinda confusing how its not highlighted? and it looks like a file when it's a
folder? (i guess cuz this isn't the root folder?)". Identify which surface
that list is (working set rail vs. a picker), verify the non-root guess, and
fix folder identity + current-item indication there. Same missing-orientation
family as item 504 — decide the presentation together so the two land as one
pattern, not two.

---
### 508 — truncated lists give no scroll or position indication (user-reported UX gap, 2026-08-27)

Long lists fold behind `+ N more…` (`workingset.rs`) and pickers window their
rows, but nothing tells the user where they are in the list or how much is
below: "there is no scroll bar so like how do i even know where my files
are... when you click the show more, it needs to show a scroll bar."
DECIDED (user-confirmed 2026-08-27): no literal scrollbar — a faint
positional count cue at the window's edges ("↑ 3 more" / "↓ 41 more"),
extending the existing `+ N more…` idiom; text-only, so it fits the
summoned-card personality and adds no interactive machinery. Scrolling
already works (picker wheel accumulation in `app/input/wheel.rs`, plus
arrow-key window sliding via `scroll_window`); the cue is orientation on top.
If direct manipulation is ever wanted, match the app's one existing
scrollbar-like object — the transient, thumb-proportioned table pan bar
(`markdown/tables.rs::table_pan_bar`) — rather than standing up persistent
chrome.

Scope (user-confirmed): STRUCTURAL, not per-surface. Derive the cue at the
one windowing owner (`scroll_window`'s `item_top`/`item_visible`/`n_items`),
so every windowed list — Go-to, command palette, theme picker, the
destination navigators, the expanded working-set panel — enrols for free and
a fitting list draws nothing. Two traps: sectioned cards (theme picker)
window DISPLAY LINES, but the cue counts hidden ITEMS, which the
plan/window split already distinguishes; and the resting stack's `+ N more…`
is an expand affordance, not a count — it stays, the cue lives in the
scrolling views only. Law: one sweep over the picker roster, no-wildcard
match, so a new picker cannot ship windowed rows without the cue.

Acceptance case (the user's own screenshot, 2026-08-27): the COMMAND
PALETTE's unfiltered All lens — a dozen-ish rows drawn from a much longer
command roster, ending at an ordinary row with nothing saying the list
continues. After the fix, that exact frame shows the below-window count; a
capture of the palette at the default window with the cue present and
arithmetic-correct (hidden = roster − visible) is the item's verify.

Second acceptance case, and the law's geometry axis (user's screenshot,
2026-08-27): the THEME PICKER in a SHORT window — eight world rows visible
of the full roster, nothing below the last row saying more exist. The cue
fires whenever the window clips the list, so the law sweeps window
geometries (tall-fits → no cue; short-clips → cue, arithmetic-correct) —
one geometry is the classic way this law would go green while blind.

---
### 511 — long bare URLs render as a raw multi-line wall (user-reported, 2026-08-27)

A pasted tracking-heavy URL wraps across seven lines of raw query string:
"isn't this kinda ugly?" On the Live-Preview model this is a conceal
candidate: a bare URL could display tamed while the caret is off its line
(e.g. domain + a quiet ellipsis, the full text returning on caret entry —
the same reveal mechanics `[text](url)` links already use, docs/markdown.md).
The file stays plain text throughout, per the product boundary.

DECIDED (orchestrator, cheap to revert — one new `ConcealKind` arm): tamed
form is scheme-stripped domain plus a quiet ellipsis, dropping path/query
entirely when concealed (`https://example.com/track?x=1&y=2` →
`example.com…`); full raw text reveals on caret entry or selection touch,
same as existing link conceal. Detect a bare URL as its own new
`ConcealKind::BareUrl` variant (`src/markdown/spans/kind.rs` +
`src/markdown/spans/detect.rs`) and route its reveal decision through the
existing line-scoped branch of `wysiwyg_reveals`
(`src/render/spans/conceal.rs`) rather than adding a second reveal rule.
Revert cost: remove the new `ConcealKind` variant, its detector arm, and its
`wysiwyg_reveals` match arm.

---
### 512 — working-set groups and folder history dedup by exact path spelling, and group labels carry leaf-only identity (measured, supersedes withdrawn 506, 2026-08-27)

The user reported "you can open the same folder twice??" (no steps captured).
Measured against the code rather than reproduced live: opening the same
folder twice through the SAME spelling cannot duplicate anything —
`recents::push` retains-out prior occurrences, `WorkingSet::open` matches by
key, `workingset/panel.rs::expanded_full` contains-checks roots. Two adjacent
gaps are certain from the code, either of which reads as "the same folder
twice":

(a) Group headings print `project::folder_name(root)` — the leaf alone — so
two different projects both named `notes` draw two identical,
indistinguishable headings. Any two same-named folders opened as projects in
one session hit this.

(b) Root identity is exact `PathBuf` equality everywhere (groups, recents
MRU), so alias spellings of ONE folder — macOS `/System/Volumes/Data`
firmlink vs `/Users/...`, symlinks, case-variant paths on case-insensitive
APFS — become two groups / two recent rows.

Fix shape: (a) disambiguate same-leaf group labels (quiet parent, like file
rows' `parent` label); (b) canonicalize at the one root-identity owner before
compare/store. Presentation of group rows lands with item 507's
folder-identity work. Laws at the unit seam with injected paths (the Recent
lens is tier-1-only, docs/harness-reach.md).

---
### 513 — contextual menus lose the teaching footer; summoned-surface material taxonomy decided (design session, 2026-08-29)

DECIDED (user-confirmed 2026-08-29), three parts. The design question was
"the format popover looks too different from the right-click menu — should
they be one material, and which?"; the answer that held after seeing the
command palette on Kite is a taxonomy, not a transplant:

(a) **The context menu keeps the palette's world list grammar — it is a
pocket palette.** One command system, one grammar, at every size: right-click
on a Ruled world draws the same hairline arrangement as the palette, just
fewer rows. Do NOT re-home `OverlayKind::Context` onto the float-panel
primitive; an earlier float recommendation was reversed on the user's
evidence (a plated context menu matches the tiny popover and clashes with
the big, frequent sibling).

(b) **The teaching footer goes on contextual menus — the build item.** Drop
the `type to filter ↵ choose esc close` line entirely for
`OverlayKind::Context` (the contextual arm already drops the query header;
this leaves pure rows). Filtering keeps WORKING silently wherever it works
today — the capability stays, the lesson goes; a right-click popup is
ambient idiom and needs no teaching. The palette's own footer is untouched
(it teaches non-ambient idioms: journeys, the workspace Back key). The
footer machinery feeds card-height math (`overlay_footer_lines` →
`overlay_card_h`), so the card must hug its rows after removal, not keep a
blank band.

(c) **The float material is reserved for content-preview surfaces** — the
format popover (and kin like the caret preview), whose material kinship is
with the page, not the chrome: its plate hosts real document styling
(highlight wash, code pill). It is deliberately NOT restyled toward list
grammar.

FLAGGED, no committed direction: the **spell popup** is now the stray — a
float that is a command list. Judge it (unjudged, not "well-loved") after
the de-footered context menu ships, side by side on Kite and a Pane world.

Coordination: item 509 (in progress) owns the same contextual presentation
(localized panel, no full-page scrim) and its branch touches the same arm —
build (b) after 509 lands, or fold it into that branch if the lane has not
shipped. Judge the footer removal on the absence-grammar worlds under 509's
chosen localized ground, not under today's full-page frost. Verify: the
footer-roster decision is pure policy (unit seam); the drawn card sweeps
target × world per 509's mechanism, plus the standing vision-smoke.

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
