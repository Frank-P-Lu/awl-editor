# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 508 — truncated lists give no scroll or position indication (user-reported UX gap, 2026-08-27)

🟡 IN PROGRESS — claude, branch item-508

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
### 513 — the spell popup's material is unjudged (residual of the 2026-08-29 design session)

Parts (a) (context menu keeps palette grammar) and (c) (float material
reserved for content-preview surfaces) are pure decisions with nothing to
build, and part (b) (teaching footer dropped from contextual menus) has
landed on `main`. The one open piece: the **spell popup** is the taxonomy's
stray — a float that is actually a command list, unjudged rather than
"well-loved." Its precondition is now satisfied (the de-footered context
menu shipped). Judge it side by side with the context menu on Kite and a
Pane world, and either re-home it onto the pocket-palette grammar or record
why it stays a float.

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
