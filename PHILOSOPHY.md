# PHILOSOPHY.md — why awl is the way it is

This is the root document. It says what awl is, what it values, and how to
decide when good ideas pull in different directions.

The other contract documents carry the detail:

- `DESIGN.md` turns the philosophy into visual and interaction rules.
- `THEMES.md` defines the worlds and their shared laws.
- `ARCHITECTURE.md` describes the system that keeps one editor coherent.
- `CAPTURE.md` defines what can be verified headlessly.
- `WEB.md` describes the deliberately narrower browser build.
- `ACCESSIBILITY.md` records current guarantees and future work honestly.

Git holds the history. This file states the product as it is understood today.

## The thesis

> awl is a calm, beautiful, and fun writing environment for prose and light
> code. It is a WYSIWYG plain-text editor with a playful caret.

awl is for people who want to create Markdown files, especially people who
enjoy writing and typing on keyboards.

The priorities are deliberate:

1. **Fun**
2. **Beauty**
3. **Calm**

Calm matters, but it is not a ceiling. Some worlds are quiet; some are loud.
What matters is that the experience is coherent, the writing remains excellent,
and expression is earned.

## 1. One thing at a time

awl keeps attention on the task that currently matters.

While writing, the document owns the screen. There is no permanent file tree,
tab strip, formatting toolbar, or status dashboard competing with it. Brief
choices appear in context and disappear when made. A sustained task such as
Settings or Version History may take over the workspace completely, then return
the writer to exactly where they were.

One task may contain several coordinated regions. Settings can place categories
beside controls; History can place a timeline beside a comparison. Those regions
serve one purpose together. awl does not show multiple editable documents at
once in v1.

### WYSIWYG plain text

The file stays ordinary Markdown. Only the rendering becomes rich.

Away from the caret, the document reads as finished writing. Wherever the caret
enters, the underlying Markdown becomes directly editable. Headings, emphasis,
links, code, images, tables, and later Markdown constructs all follow the same
direction: show what the document says until the writer arrives to edit how it
is written.

This is not a hidden document model. There is no proprietary format and no
conversion boundary. The source is always the file, portable to any other text
editor.

### A complete home for writing

awl can be the complete home for a folder of Markdown without becoming a project
manager. It includes the simple file operations, navigation, search, and version
history needed to sustain writing. The filesystem stays real and understandable;
awl does not replace it with an application database.

The opening experience follows the same rule. First launch opens one real
Markdown file that is both welcome and tutorial. The user learns awl by reading
and editing inside the actual editor, not by completing a modal tour.

## 2. Beauty is coherence

Typography, readable prose, generous spacing, and clear hierarchy are the floor.
Every surface should feel composed rather than accumulated.

Beauty does not require visual silence. awl may be colorful, animated,
theatrical, or strange. The test is whether those choices form a convincing
environment for writing. Flash alone is a gimmick; flash in service of a
beautiful editor can be the point.

### Worlds refresh the writer

Themes in awl are worlds, not swatches. Changing worlds should make a familiar
instrument feel fresh without changing how it works. This renewal is central to
awl: when writing starts to feel stale, the room can change around it.

Each world earns its place by being distinct and coherent across typography,
palette, layout, motion, patterns, and chrome. It must remain good to read and
edit in after the first impression. A world may be deliberately intense; not
every world has to be the one someone uses all day.

The authored roster targets twenty worlds. Quality remains the gate: a count
never justifies a weak addition.

Most customization remains curated. A custom world may eventually let the user
assemble colors, faces, patterns, and other existing ingredients from awl's
wardrobe. A public theme format and theme sharing may make sense later, but the
world system is too young to freeze that design in v1.

## 3. Fun lives under the hands

The main source of fun is the experience itself. awl reacts to the writer. It is
tactile, immediate, and alive.

The caret is the main character, but it is not the only thing allowed to move.
Worlds and summoned surfaces may animate when motion expresses their character
or responds to the user. Nothing should move merely to delay an action or demand
attention without earning it.

### Performance is part of the personality

Frequent actions must complete immediately. Typing, caret movement, selection,
and scrolling cannot wait for animation. Motion follows the result instead of
slowing it down: the caret reaches its destination at full speed and its trail
carries the gesture.

Less frequent interactions, such as moving through a menu or changing worlds,
may use more visible transitions. They still must not feel sluggish.

When a world is moving, the target is a fluid 60-frame-per-second experience on
moderate hardware. Static worlds do not need to redraw like games, and idle work
should be negligible. Effects earn their cost; the editor never spends
performance merely to prove that it can.

Small surprises, distinctive worlds, and playful summoned surfaces support the
fun. They do not compensate for latency. A fast editor is fun in the direct,
physical sense that the machine feels connected to the hands.

## 4. Opinionated by default

awl makes strong choices and works beautifully before the user configures
anything. When workflows genuinely differ, a setting is justified. Uncertainty
alone is not a reason to add a preference.

Most of awl cannot be customized. Someone who wants a fundamentally different
editor can fork it. There is no plugin or extension system.

### Keyboard-native, not keyboard-exclusive

Typing on keyboards is part of the audience and part of the pleasure.

Native platform shortcuts receive first-class design, documentation, and
testing. awl also ships selectable, platform-aware keymaps; Emacs is the first
alternative, not assumed knowledge. Users may rebind individual actions, but
customization does not excuse incoherent defaults.

Pointer users are welcome. Common contextual actions may be clicked, and the
selection popover meets mouse users halfway while teaching the keyboard
experience. awl does not build advanced mouse-first workflows or grow a
desktop-publishing interface beside the keyboard one.

### Editing rigor without purity theatre

Text correctness is part of the product. Unicode, selections, undo, wrapping,
line endings, save fidelity, and predictable movement deserve serious
engineering attention. Data loss is never an acceptable simplification.

Rigor is still proportional. awl aims to be better than mainstream editors at
the details writers encounter, not to double its complexity for every
theoretical edge case. Spend generously where correctness affects real writing;
measure obscure completeness against its product cost.

### One core, adapted honestly

macOS is the design center. Linux is a close second. Windows should work, but it
does not yet receive the same promise of native polish.

Desktop is the main product. The web build exists for convenient light editing
and shares the editing core where the platform permits. Full desktop parity is
not a v1 promise, though the browser build may grow into more later.

Accessibility follows the same honesty. awl already supports parts of an
accessible experience and intends to improve after v1. Current guarantees belong
in `ACCESSIBILITY.md`; aspirations must not be described as shipped behavior.

## 5. The boundaries protect the product

awl cares about writing Markdown.

It does not care about Word-style document formatting. It has no proprietary
layout model, styled clipboard, or general formatting toolbar. Contextual
Markdown actions are welcome because they help people write Markdown.

It supports light code editing, not software-development machinery. There is no
LSP, symbol graph, build system, background project intelligence, multi-cursor
model, or IDE plugin ecosystem.

It assumes offline, single-player use, and that assumption will not change. The
complete base editor works locally without an account, telemetry, runtime asset
fetches, or a required service. Real-time collaborative editing is outside its
direction.

awl is open source forever. A Pro edition is possible but unsettled. If it ever
exists, it may add things that naturally belong there—sync or optional asset
packs, for example—but it cannot remove capabilities from or hollow out the
open-source base editor.

## 6. How to decide

When a new idea appears, ask:

1. Does it help someone create Markdown?
2. Does it make the experience more fun, more beautiful, or meaningfully calmer?
3. Does it preserve the speed of frequent actions?
4. Does it form one coherent task, or add permanent competing furniture?
5. Is its expression earned by the writing experience, or is it novelty alone?
6. Does it belong in an opinionated core, or is it machinery for another kind of
   product?
7. Can it remain honest about files, platforms, accessibility, and verification?

If the idea passes, build it as one product rule with one owner. If it needs a
theme-specific mechanism, a second editing model, or a parallel interaction
system, the design is probably not finished.

The success test is modest and demanding. Someone opens awl and says:

> What? This looks nice. This is interesting.

Then they begin writing.
