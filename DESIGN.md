# DESIGN.md — how awl should feel

`PHILOSOPHY.md` holds the identity, priorities, and product boundary. This
document turns them into visual and interaction rules.

The implementation mechanisms live in `docs/render.md`, `docs/markdown.md`, and
`docs/fonts.md`. Concrete world laws live in `THEMES.md`.

## 1. The instrument

awl should feel like an instrument someone plays, not an application they
operate.

Its design vocabulary comes from four places:

- **Swiss and International Typographic Style:** grids, hierarchy, confident
  space, and structure that does not need ornament to explain itself.
- **Teenage Engineering:** a disciplined object with a playful soul; one
  dependable instrument containing several distinctive worlds.
- **N++ and good game feel:** immediate control, motion drawn as a mark, and
  responsiveness that makes restraint feel alive rather than sterile.
- **Persona:** proof that a functional core can support an authored, theatrical
  frame when the composition remains legible and intentional.

These are references, not costumes. Borrow their reasoning, not their surface
decoration.

## 2. Room and Frame

Every world composes two layers:

- **Room:** the writing itself—the page, prose, code, caret, selection, images,
  and tables.
- **Frame:** the ground, margins, orientation, overlays, workspaces, menus, and
  other chrome around the writing.

The Room is always excellent to read and edit. It may change typeface, palette,
and authored details with the world, but it cannot sacrifice text to spectacle.

The Frame carries most of a world's personality. It may be quiet, graphic,
animated, or loud. A strong world treats Room and Frame as one composition
without letting the frame obscure the work.

This is a heuristic, not an austerity law. Firetail may be theatrical; Wagtail
may reject hue; an image may contain colors awl does not control. The question
is not “is this calm?” It is “does this make a coherent and beautiful place to
write?”

### Worlds are complete environments

A world owns the visible properties that give the environment character:
typography, palette, ground, patterns, surface composition, chrome treatment,
caret treatment, and motion.

Changing worlds is a full audition. The real document and the real summoned
surface adopt the highlighted world together. Interaction state—query, selected
row, scroll, and focus—survives the crossing because it belongs to the user.

A deliberate selection may snap the surface into the destination world's
composition. Passive hover may preview color without moving the surface under
the pointer.

World personality is data through shared renderers. A world requiring identity
checks and private layout code is a design failure unless the shared model first
proves genuinely incapable of expressing the idea.

## 3. Typography and hierarchy

Typography is the foundation. Prose must remain readable before any pattern,
motion, or chrome earns attention.

Text uses authored roles rather than arbitrary styling. The common grammar is:

- content is fully present;
- markup and secondary information recede;
- metadata is available without competing;
- hierarchy comes from size, spacing, face, and limited authored weight.

### Ink roles

| Role | Use |
| --- | --- |
| `base_content` | Body prose, code, headings, and primary labels |
| `muted` | Markdown marks, comments, secondary labels, and quiet controls |
| `faint` | Orientation, counters, and metadata that should disappear from attention |
| `primary` | The world's principal interactive presence, normally the caret |
| `primary_content` | Legible ink drawn on `primary` |
| `selection_document` | Selected or matched content — the wash that covers text |
| `selection_ui` | The band under a selected row in a summoned surface. Derived as a value step off the surface ramp, so it is never a new hue by construction; a world may author it instead |
| `error` | Failure or destructive warning, never decoration |

These are semantic jobs, not fixed hues. Worlds author their own palettes and
may be monochrome or strongly colored. Functional colors keep their meaning.

### Size roles

The type scale is expressed as multipliers over body metrics:

| Role | Scale | Use |
| --- | ---: | --- |
| `TITLE` | 1.6× | H1 and document title |
| `SECTION` | 1.3× | H2 |
| `SUBHEAD` | 1.15× | H3 and below |
| `BODY` | 1.0× | Prose and code |
| `LABEL` | 0.8× | UI labels and metadata |

Worlds may give lower heading levels an authored bold face where the typeface
needs it. H1 spends size rather than bold weight.

Avoid scattered pixel sizes, invented emphasis colors, and synthetic weights.
Choose an existing role or extend the shared grammar deliberately.

### Layout

The writing column is adaptive, stable, and generous. It stays centered when
the frame is symmetric and moves only when real occupied territory requires it.
Margin surfaces borrow leftover space; they do not steal width from prose or
reflow the document when toggled.

Spacing carries hierarchy as seriously as type. Prefer a clear interval and
confident negative space over boxes, separators, and labels explaining every
relationship.

## 4. Presence and response

The caret is the clearest point of presence. It occupies the character rather
than hiding in the seam between characters, never blinks, and makes motion
visible through its body and trail.

The caret is a main character, not a monopoly on expression. Other elements may
react or animate when motion expresses a world or acknowledges an action.

### Motion follows importance

The more often an action occurs, the less animation may delay it.

- Typing, caret movement, selection, and scrolling complete immediately.
- The caret reaches its destination at full speed; squash, stretch, overshoot,
  and the trailing mark describe the movement after the input has landed.
- Menu navigation and other occasional choices may use visible transitions,
  provided they remain responsive.
- World changes and ambient grounds may carry longer motion because they are
  environmental rather than input-critical.

No animation may hold the result hostage. If an effect makes the editor feel
slower, redesign the effect.

### Performance is visible design

Moving worlds target fluid 60-frame-per-second presentation on moderate
hardware. Static worlds redraw only when something changes. Idle work should be
negligible.

Prefer event-driven motion, cached geometry, bounded fields, downsampled effects,
and simple compositing. Do not remove character reflexively to save cost; find
the inexpensive version of the intended effect and measure it in release mode.

Motion settings and current accessibility behavior are specified in
`ACCESSIBILITY.md` and the relevant feature docs. Do not claim a broader
guarantee than the implementation provides.

## 5. Surfaces and attention

One primary task owns the screen.

### The rule that decides which surface

**Overlays split attention for a brief contextual choice. Summoned workspaces
relocate it for sustained work.**

That is the whole test, and it is about the task rather than the amount of
content. A choice you make *while* doing something else keeps the document in
view, because you still need it — so it is an overlay, however many rows it has.
A task you go *into*, work in, and come back from should not be read through a
card with your unfinished sentence showing behind it — so it takes the viewport,
leaves the document as a quiet backdrop, and returns you to the exact editor
state when you leave.

A workspace is still summoned. Relocating attention buys the viewport and a
second coordinated region; it does not buy permanence. The roster is deliberately
short — Settings and Version History — and a picker does not graduate onto it
because it grew.

### Contextual overlays

An overlay belongs to the current writing action. It keeps the document visible
because the user still needs that context. Examples include Commands, Goto,
Find/Replace, Theme, Caret, spelling, links, and formatting.

Overlays:

- are summoned and transient;
- keep a stable relationship to the document or their authored world rail;
- share row measurement, focus, navigation, and hit-testing primitives;
- use the world's surface composition rather than feature-specific decoration;
- dismiss without disturbing document state.

### Sustained workspaces

A workspace relocates attention for a sustained task. Settings and Version
History may occupy the viewport rather than competing with a readable document
behind a small card.

A workspace uses two coordinated regions: a primary navigation list and the
content it governs—categories beside controls, or a timeline beside a
comparison. They are one task, so they share one search, one selection grammar,
and one back path:

- the primary list is where a workspace opens, and where a single `Esc` leaves
  for the editor;
- moving into the content is a focus transfer, and `Esc` there is a *back* to
  the primary list, never a close;
- exactly one region is live at a time, and the difference is expressed by
  value—the same marker at less presence—rather than by a second decoration.

Wide windows show both regions at once. Narrow windows stage them sequentially,
with the same back path, because the alternative is compressing both into
illegibility. Width is presentation: it decides what is drawn and never what a
key means.

Closing the workspace returns to the exact editor state. A workspace is still
summoned; it is not a persistent application shell, page router, tab system, or
permission to migrate every picker.

### Depth and separation

Prefer figure/ground, value, spacing, and composition over generic elevation
effects. The shared neutral ramp provides the default depth grammar:

- `base_100`: deepest plane;
- `base_200`: raised or differentiated plane;
- `base_300`: focused or foreground plane.

Worlds may express shared surfaces through authored Pane, Bars, border, binary,
or other data-driven compositions. Borders, rules, and patterns are valid when
they belong to the world; a default drop shadow added merely to make something
look like a card is not.

### Persistent orientation

Persistent chrome is narrowly bounded.

In page mode, the left margin answers **where am I?**:

- Outline at the top: position in the document;
- filename and folder identity at the bottom: position in the filesystem.

The right margin answers **how much?**:

- word count and reading time at the bottom.

Margin surfaces:

- hug the writing column rather than the window edge;
- use quiet label treatment;
- hide when space is insufficient;
- never change the prose column's geometry;
- remain orientation, not permanent management UI.

The Outline may click-to-jump. It is not a resizable or focusable file-tree
substitute.

When the last document closes, the Room is absent rather than replaced by a
fake scratch page. The Frame keeps the remembered folder context and offers a
centered, calm start surface with exactly two actions: **New document** and
**Go to**. There is no caret, page, filename, outline, count, or document node
until one of those actions creates or opens a real document. A first launch is
unchanged: it still begins with the ordinary scratch document.

Web and Linux may show a slim persistent menu bar because those environments
otherwise provide no discoverable application menu. macOS uses its native menu
bar and never draws the substitute.

## 6. Keyboard and pointer

Keyboard interaction is the primary grammar. Surfaces teach important shortcuts
through concise footer hints and which-key guidance rather than assuming prior
knowledge.

Pointer interaction is real, not grudging:

- click to place the caret or choose a row;
- drag to select text, resize images, and adjust range controls;
- use contextual menus;
- use the selection-triggered Markdown formatting popover.

The formatting popover is a bounded bridge for pointer users. Its controls fire
the same actions as keymaps and Commands. It appears from a mouse selection,
dismisses with that context, and does not grow into a persistent toolbar.

Do not create separate keyboard and pointer products. They share actions,
state, geometry, and validation.

Hover feedback is drawn only where which control fires, or that a control
exists at all, is genuinely ambiguous — the format popover's buttons (its hit
regions tile edge to edge) and inline-image resize handles (the OS cursor is
otherwise the whole affordance). Every other clickable surface stays visually
still under hover; the pointing-hand cursor alone is the acknowledgement.

## 7. Rich content

WYSIWYG content should feel native to the page rather than embedded as a foreign
widget.

- Markdown marks recede away from the caret and reveal for editing.
- Images default to fit the column. Awl does not recolor user images or surround
  them with ornamental frames. Missing-image and resize affordances remain
  subordinate to the content.
- Tables use real grid geometry with legible cells and restrained structure.
- Code blocks remain readable as code without turning into miniature IDEs.
- Formatting controls edit Markdown and remain undoable.

The rich render never changes file ownership: source Markdown is always one
caret placement away.

## 8. Responsive behavior

Narrow layouts simplify or stage; they do not miniaturize.

When space contracts:

1. preserve readable type and honest control sizes;
2. remove decorative travel or peripheral detail;
3. hide optional margin orientation;
4. stage multi-region workspaces with a back path;
5. reduce a world's composition within its authored bounds;
6. never overlap, clip, introduce horizontal scrolling, or silently change the
   interaction model.

Drawn geometry and hit-test geometry have one owner. A surface that looks
clickable must be clickable where it is drawn at supported zoom and DPI.

## 9. Designing something new

Before adding a surface or visual mechanism, answer:

- What single task owns attention?
- Is this Room or Frame, and does its expression belong there?
- Which shared text, surface, row, focus, and input primitives already own the
  behavior?
- Does motion preserve the speed of the action?
- Does the result remain legible in every enrolled world?
- What happens narrow, zoomed, at high DPI, and with current accessibility
  settings?
- Can the real result be captured and inspected deterministically?
- Is the feature still interesting after the novelty wears off?

Then verify the rendered result, not only the state that intended to draw it.
Use real captures for geometry and appearance, sidecars for state, and live
release-mode judgment for motion and feel.

The final visual test is simple: awl should look good enough to interrupt
expectation, interesting enough to invite exploration, and responsive enough
that the user begins writing instead of admiring the interface from a distance.
