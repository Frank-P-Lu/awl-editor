# Accessibility — where awl stands (tier 2, 2026-08)

A calm, honest statement of what works today, what the known gap is, and where
the seams are for a future tier. No promised dates — this is a running note,
not a roadmap. See `CLAUDE.md`'s doc list for the rest of the contract docs
this one sits alongside.

## What works today

- **Fully keyboard-drivable.** Every command is reachable from the Cmd-P
  command palette by name — there is no mouse-only affordance anywhere in
  awl (`DESIGN.md` §6: keyboard-native with a bounded contextual pointer bridge,
  never a floating toolbar). The two-binding keymap (native ⌘ as the
  advertised layer, a quiet Emacs second slot) means the whole editor — open,
  edit, save, search, format, switch project, rebind keys, everything — is
  operable without ever touching a pointing device.
- **The picker footer names the keys THIS row has.** Every summoned card
  carries one dim line of control hints — for a sighted keyboard user, the
  only statement awl makes on screen about what a key will do (a screen reader
  now also hears it, as the dialog's description). So it is per-ROW wherever
  a row's keys differ from its picker's default: on a Settings row with a
  value rail (Zoom), where Left/Right step the value instead of switching
  category, the line reads `←/→ adjust` rather than `←/→ lens`. The rule is
  law-tested as an outcome — the key is driven for real on every Settings row
  and the hint must have named what happened — because a footer that says the
  wrong thing is worse than no footer: it invites a keyboard-only user to
  change a setting they were only trying to navigate past.
- **Zoom.** Cmd-=/Cmd–/Cmd-0 (and the Settings "Zoom" row) scale the whole
  document glyph-for-glyph, independent of the page column measure.
- **14 curated theme worlds, each contrast-law-tested.** Every world's ink
  ladder (`base_content` → `muted`) and role tints are checked by an
  automated law test for pairwise distinguishability and an amber guard (see
  THEMES.md) — a world can't ship with illegible or indistinguishable text.
  This is a contrast *floor*, not a dedicated high-contrast mode; there is no
  separate "increase contrast" toggle beyond picking a world.
- **Reduce Motion (this round).** A real accessibility preference, not a
  cosmetic toggle: `Settings → Editor → Reduce motion`, or the config key
  `reduce_motion = true`. When on, every juice animation (the caret's
  spring/glide, its squash-pop flinches and trailing streak, the copy-pulse
  selection brighten, the caret-style picker's choreographed preview loop)
  settles INSTANTLY to its exact final state instead of easing over time —
  same position, same color, same everything; motion-off is a pure time
  compression, never a feature change. Absent config means `auto`: awl reads
  the OS-level preference where one is reachable (macOS System Settings →
  Accessibility → Display → Reduce Motion; the web build's
  `prefers-reduced-motion` media query), consulted once at launch. Native
  Linux has no reliable cross-desktop accessibility API wired here yet, so
  `auto` reads as off there — the config key is the door: set
  `reduce_motion = true` by hand.
- **The window title names the document.** The OS window (and its title-bar
  text, which a screen reader's window list announces) always reads
  `awl - <path or "scratch"/"*scratch*"> [<world name>]` — never a bare
  "awl". This was true before this round; it is now driven by one shared,
  unit-tested function (`app::files::window_title`) instead of two
  independently-hand-written copies, so the very first frame after launch
  and every later open/switch/theme-change agree.

## Screen readers — what is wired, and what is unproven

**Native awl now publishes a real accessibility tree.** The editor still draws
its own text with wgpu straight onto a GPU surface — there is no `NSTextView`
and no platform text widget underneath any of it — so awl builds the tree
itself and hands it to [AccessKit](https://accesskit.dev/), the crate the Rust
GUI ecosystem has converged on for exactly this problem. AccessKit bridges it
to each OS's real API: NSAccessibility on macOS, AT-SPI2 on Linux, UIA on
Windows.

**One owner, three consumers.** `App::semantic_snapshot` folds the live
`Buffer`, the summoned-surface ladder, search state and the passive surfaces
into one renderer-independent `SemanticSnapshot`. Three things read it and
nothing re-derives it: the AccessKit tree, `awl --semantic-json`, and the
`semantic` field of a live-`App` capture sidecar. That is the point — two
parallel descriptions of one UI drift, and a screen reader is exactly where
that drift is invisible until it hurts someone.

### What the tree contains

| surface | what it exposes |
|---|---|
| The document | One multiline editable text node whose children are STABLE LINE RUNS — one text run per line, carrying that line's text including its newline — and the caret/selection as GRAPHEME offsets over the whole document: a combining sequence, a ZWJ family emoji or a flag is one position, never half of one. A selection that crosses a line break names two different runs, which is the ordinary case and round-trips both ways. Supports focus, set-selection, replace-selected-text and set-value. |
| Summoned pickers (all 19 kinds) | A dialog with its title and footer hint, its query field, and one option per visible row with its binding value and selected state. Row identity is keyed to the corpus, so filtering never renames a row under an assistive cursor. |
| Settings rows | The control each row actually is — check box, slider, text field or button — not a generic list option, with the actions that control really supports. |
| Find and replace | Both fields with their carets, the match-count description, and the case-sensitivity check box. |
| The format popover | One button per formatting command, with its on/off state. |
| Passive cards | About, Lifetime, Writing streaks, the stats HUD and the shortcut peek, announced line by line WITHOUT taking focus. Their text comes from `crate::card::content`, the same owner the renderer composes from, so what is heard is what is drawn. |
| Which-key | The `C-x` continuation panel, announced as informational rows. It teaches keys; it does not offer to press them. |
| The rendered menu bar | Menu titles with real expand/collapse, and an open dropdown's rows with real click. (This is the awl-drawn bar — Linux and web. macOS's native bar is already accessible on its own.) |
| Notices | The transient status line, as a live region. |

Exactly one node is focused at any moment, and it is the one the summoned-UI
ladder names. Passive surfaces never move it.

### Actions are real, not advertised

An action a node claims but that nothing performs is worse than an absent one,
because a screen reader offers it to the user and it silently does nothing. So
every advertised action is routed back through the ordinary `Action` /
`apply_transition` owners — the same path a keypress takes, with the same undo
and the same redraw — and a law sweeps every node of every surface and fails by
name on any action that is not. Nothing mutates the rope or an overlay from a
platform callback; requests arrive as winit user events and are applied on the
main loop.

### The honest limits

- **One real VoiceOver sitting has been run; the follow-up has not.** The first
  sitting (2026-08-02) is what found the "not responding" report above. Whether
  the fix holds — a full typing and navigation journey with no stall — needs a
  second sitting on an unlocked, foregrounded display, and that has not
  happened. No AT-SPI journey has been run at all. Everything else is
  verified by unit and law tests over the snapshot and its AccessKit
  projection — that the tree is correct and complete, that JSON and AccessKit
  say the same thing, that actions really fire. Whether a screen reader
  *reads it well* — announcement order, verbosity, live-region politeness — is
  unproven and stated here as unproven.
- **The web build has no accessibility tree.** AccessKit has no canvas or web
  adapter, so this round is native-only by construction. A browser story needs
  a DOM mirror behind the canvas, which is a separate round with a separate
  design; it is not a port of this one.
- **Announcement cost is gated, and incremental.** A frame with no assistive
  technology attached builds nothing at all — the only work is one integer
  compare. While one IS attached, the projection is retained between frames and
  updated in place: an ordinary keystroke re-reads the one line it touched and
  publishes two nodes (the changed run, and the document node whose selection
  moved), measured identical at 100, 1 000 and 20 000 lines. A gliding caret
  publishes nothing.

  This was a real defect, not a tidy-up. The first VoiceOver sitting
  (2026-08-02) found awl intermittently reported as **"not responding"** while
  editing: every redraw was cloning the whole rope, running UAX #29 over the
  entire document, projecting every node and republishing one monolithic
  document text run. AccessKit expects a full tree only at ACTIVATION and
  changed nodes afterwards, and awl's event-loop-proxy adapter — whose
  activation cannot answer synchronously — forced the full-tree form on every
  update. Item 218 replaced it with a synchronous mixed activation handler
  backed by a thread-safe parked tree, and changed-node updates from then on.
- **Reading order is the tree's order.** awl does not model spatial navigation,
  and there is no bounding-box geometry in the tree yet, so a screen reader's
  cursor-tracking and mouse-over modes have nothing to work with.

## Hold-gestures note

Two features in awl are **holds** — press-and-hold-to-peek, game-map style,
not a toggle:

- **The stats HUD** (Option-Cmd-I) — file-created date, session time, word
  count/reading time, percent through document — shows while held, vanishes
  on release.
- **The Cmd-P "peek"-style summon flow** for a picker preview (e.g. the
  caret-style picker's live choreographed demo) likewise only animates while
  its picker is open.

Neither hold gates information that is otherwise unreachable: every figure
the stats HUD shows is also derivable without holding anything (word count
and reading time render in the quiet bottom-right readout for any markdown
buffer; the file's path and the active theme are always in the window
title; session time is the one figure with no non-hold equivalent today, a
narrow, logged gap). A hold-only affordance is a genuine keyboard operation
(a single chord, held) rather than a mouse gesture, so it does not itself
block keyboard-only use — but a hold does require the physical ability to
keep a key depressed, which is worth naming honestly rather than assuming
away.

## Where this leaves tier 3

Tier 1 closed the two needs awl's architecture made cheap: Reduce Motion as a
render-side settle-instantly gate, and keyboard operability that was already
the design (`PHILOSOPHY.md` §4, `DESIGN.md` §6). Tier 2 — this round — closed
the expensive, architectural one: a real accessibility tree, with one owner
feeding both the platform and a headless agent.

What tier 3 is for, in order of how much it would matter:

1. **A real screen-reader sitting.** VoiceOver on macOS and Orca on Linux,
   driven by a person, on an unlocked display. Nothing below is worth designing
   before that says what actually reads badly. The first VoiceOver sitting
   happened and produced item 218; the confirming one is still owed. Note for
   whoever runs it: check `displaysleep` and the screensaver `idleTime` first —
   a display that sleeps mid-sitting silently invalidates it, as it did seven
   minutes into the 2026-08-02 attempt.
2. **Web.** A DOM mirror behind the canvas — the same snapshot, a different
   adapter.
3. **Geometry in the tree.** Bounding boxes, so cursor tracking and
   mouse-over reading work.
4. **Windows.** AccessKit's UIA adapter is already in the dependency tree; awl
   has no Windows build to put it in.
