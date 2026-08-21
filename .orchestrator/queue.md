# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

### 467 — one conditional frame clock owns live motion and the idle boundary (USER DECISION 2026-08-21; ready after 466)

Make awl's game-like behavior explicit without turning the editor into a
permanent 60 fps loop. One frame-domain state machine reduces every source of
work to exactly three host instructions: **Idle** (`Wait`, zero redraws),
**Deadline(earliest instant)** (`WaitUntil`, one wake for a debounce/ambient
tick/retry), or **Animating(active reasons)** (request the next display-synced
frame until every reason settles). Native follows the window's presentation
cadence; web follows winit's requestAnimationFrame path. Do not hard-code 60 Hz:
60/120 Hz and dropped frames sample the same elapsed time, and a hidden,
occluded, unfocused or failed-present surface must not spin.

This is a time/scheduling ownership refactor, not a renderer rewrite. Introduce
one injected-clock `FrameSample` (`now`, elapsed since the prior presented
sample) and an exhaustive activity roster for caret motion, caret preview,
copy pulse, overlay entrance/band, fold chevrons and the travelling Kite ground.
Each animator keeps its own curve, endpoints and accessibility semantics; it
samples the shared time and reports whether it remains active. Reduce Motion
still settles each effect at its existing final pose. A new animator must fail
a no-wildcard enrolment law until its wake, reduced-motion and pause behavior
are named.

Collapse the coordination glue into the one reducer:

1. Replace `PresentationState::last_frame: Option<Instant>`'s double duty as
   both a delta-time source and an implicit "the loop is hot" flag with explicit
   clock/presentation state. Keep a plainly named last-presented timestamp only
   if elapsed-time sampling needs it.
2. Remove the renderer's one-off `band_ease_started` field/take bridge and
   `keep_gpu_loop_hot(stepped, band_ease_started, frame_presented)`. The frame
   outcome carries one post-prepare activity set, so motion first discovered
   while resolving geometry cannot be invisible to the scheduler.
3. Fold `App::advance_travelling_ground` into the same animation roster. Theme
   data and the existing focus/motion/resize/blur eligibility rule still decide
   whether Kite travels; there is no world-identity scheduling branch.
4. Make which-key, peek, note/document autosave, zoom persistence,
   resize/move/crossing settle, low-rate lava/stars/waves ticks, toast expiry and
   GPU retry propose deadlines rather than setting control flow themselves.
   The reducer chooses animation over the earliest deadline, eliminating the
   scattered `last_frame().is_none()` guards and deadline-merging calls.
5. Route the debug panel's one settle-stamp redraw through the same draw-once
   demand and have the flight recorder name the active reasons and actual
   presented-frame interval. Theme-switch work phases and input-to-first-pixel
   latency remain separate measurements; add input-to-animation-settled timing
   rather than pretending the first present is the end of visible motion.

Do **not** convert sparse ambient ticks, autosaves or quiet-settle work into
per-frame clients, and do not add a background thread, timer wheel, document
layer cache or fixed-rate sleep. `request_frame` remains the ordinary one-shot
invalidation door; the clock only decides whether another frame is owed after
that draw. Preserve the headless no-clock rule for ordinary captures and use
the existing injected/virtual clock for deterministic multi-frame laws.

Stage the migration so the state reducer lands behind current behavior first,
then move the bounded animators, the post-prepare band report and Kite one at a
time; delete each old bridge only after its consumer is enrolled. Because this
is an identity-gated render refactor, follow it with an outcome audit rather
than treating byte identity as proof of correctness. Pure virtual-clock tables
cover Idle/Deadline/Animating, earliest-deadline selection, one-shot redraws,
60/120/coarse/dropped samples reaching the same wall-time pose, and every
activity becoming idle exactly once. Mutation checks remove one activity and
reintroduce a prepare-time zero epoch; both must fail. Laws prove a static
window requests no follow-up frame and a failed/occluded present parks for the
existing retry instead of polling. Settled captures remain byte-identical
across worlds, list styles and 1x/2x DPI; a motion film and release-mode live
flight cover caret, copy, fold, overlay and Kite journeys, followed by the
standing vision-smoke. Run the full native receipt and wasm smoke; real browser
RAF smoothness and live 60/120 Hz feel remain human confirmations. Read
`DESIGN.md` Motion/Performance, `ARCHITECTURE.md`, `docs/render.md`,
`docs/platform.md`, `docs/harness-reach.md` and `WEB.md` before implementation.

### 466 — the theme picker's living band counts its 110 ms from input, not from first prepare (USER-REPORTED BUG 2026-08-21; 🟡 IN PROGRESS — item-466-theme-input-epoch (codex), branch codex/item-466-theme-input-epoch)

Fix the paced keyboard case the existing burst benchmark and debug-pane
`theme worst` number do not describe. With ordinary motion enabled, a theme
move performs synchronous font/document reshaping and atlas work before the
new row can present; only inside that frame's `prepare` does the living band
discover its target and reset its ~110 ms ease. A 30–70 ms theme frame therefore
makes one key press occupy roughly 140–180 ms visually. At the user's ordinary
~150 ms Up/Down cadence the next press can arrive while the prior band still
claims to be in flight, crossing `chase_or_snap`'s glide/snap boundary and
reading as a pause followed by a catch-up. Reduce Motion is lag-free because it
removes precisely this post-render animation tail. The logical selection and
theme preview are already current; do not defer either or remove the living
band.

Stamp a theme-picker movement at the real input/apply seam and carry that epoch
to the renderer without introducing a wall clock into ordinary capture. When
prepare first resolves the selected row's geometry, derive the band phase from
`now - movement_at`, not zero from the prepare instant. Expensive work therefore
uses up the animation budget: a quick frame shows most of the glide, a frame
that consumes 110 ms or more shows the band settled, and no work makes the
effect longer than authored. On a genuine rapid retarget, first sample the old
pose at the same `now`, then preserve the current latest-selection-wins snap
policy; logical selection, hit testing and Enter always target the newest row.
Freshly opened overlays still begin settled, pointer and wheel movement use the
same epoch owner, and Reduce Motion remains an immediate final pose.

Extend the flight recorder rather than the debug pane's theme transaction
headline: retain raw key/apply/prepare/present stamps and add the band phase plus
an input-to-band-settled line. `theme latest/worst` continues to end when the
new themed frame first presents; label or document that boundary rather than
quietly folding decorative tail time into font/reshape/atlas phases. A paced
release-mode run at approximately 150 ms per Up/Down is the human acceptance:
each row responds continuously with motion on and remains instant with Reduce
Motion, with no pause/catch-up rhythm.

At the pure time seam, sweep render delay 0/40/80/110/150 ms and input cadence
60/100/150/220 ms across Pane's living band and the ordinary sliding-band
override. Assert that settlement is at most 110 ms after input, that delay never
adds a second 110 ms, and that a superseding move cannot animate toward a stale
row. Cover keyboard, wheel and pointer, world crossings, first-open and Reduce
Motion. Mutation-proof the law by re-anchoring the epoch at prepare and require
the 40/80 ms cases to fail. Ordinary settled captures remain byte-identical;
the existing motion-frame harness proves intermediate geometry, while actual
wall-clock feel is reported only from the live release build. Read
`docs/fonts.md`, `docs/render.md`, `docs/platform.md` and
`docs/harness-reach.md` before implementation. Item 467 may later replace the
bridge machinery, but this fix lands independently and must not wait for that
refactor.

### 465 — Cassowary's summoned chrome becomes one submerged operations console (USER DECISION 2026-08-21; ready to build)

Revamp Cassowary's command palette around the simplified composition chosen in
the design session: one dark right-anchored console, the active category docked
into its top edge, the command rows and keyboard footer inside that same pane,
a quiet index beside it, and the oversized `COMMANDS` placard deliberately
cropped by the bottom of the viewport. Preserve Cassowary's green phosphor on
black glass, Iosevka face, filled CRT caret, blurred document backdrop and red
reserved for errors. This is a stronger Frame around the same command task;
commands, filtering, category movement, acceptance, dismissal, accessibility
and row hit-testing do not change.

The composition must extend the shared chrome grammar, not create a Cassowary
renderer. Every new choice gets an inert default in `RenderCaps`, one shared
geometry/draw owner, and a neutral name another world can select without code
changes or a Cassowary identity check. Reuse the existing `Pane`, shaped facet
labels, row layout, card elevation and chamfered card shape wherever they
already express the result. A capability does not need a second production
world on day one; reusability means it can be assigned to a synthetic or future
world entirely through data. Do not distort another world merely to prove the
abstraction.

**Composition pass:**

1. Promote the existing unified pane/footer composition from its development
   override into authored theme data. Cassowary selects a card-backed pane with
   the keyboard hints inside the same exterior boundary as the rows; other
   worlds retain their current composition byte-for-byte.
2. Add a shared **docked active facet** treatment. The active category reads as
   a tab joined to the pane's top border; inactive categories remain quiet
   labels on the same navigation line. The shaped label span remains the one
   hit target and keyboard focus geometry. `Files` appears once, in the active
   tab — no second `FILES` subtitle on the side or inside the pane.
3. Add an index-only locator grammar and let Cassowary show the real selected
   category index as two digits (`02`) beside the pane. It is derived from
   state, not decorative telemetry. The active tab owns the category name, so
   the locator does not repeat it.
4. Add authored placard bleed/placement rather than applying a draw-time
   offset. Cassowary's bottom placard may intentionally leave the viewport,
   producing the submerged `COMMANDS` crop while its reported geometry, the
   rotated locator and narrow-window degradation all agree with what is drawn.
   Keep the existing chamfered card for this pass; exact asymmetric notches are
   not required.

**Material pass, after the composition is visible:** add a reusable static
scanline/raster material for summoned chrome. Cassowary applies it sparingly to
the console and placard at one absolute-canvas phase so rows do not shimmer as
selection moves. It remains legible at 1x and 2x DPI, schedules no idle redraw,
contains no clock or randomness on the headless path, and has an honest reduced
motion/transparency degradation where the existing accessibility contract
requires one. Prefer a bounded overlay/material pass over a private duplicate
text renderer. Tune the shipped strength from captures and a live release-mode
taste check; presence must be visible under pixel arithmetic without making
body text or shortcuts harder to read.

**Deliberate exclusions:** no glow/bloom pass, animated flicker, rolling bars,
noise, chromatic aberration, bespoke asymmetric corner masks, outer targeting
rails, hazard stripes, fake measurements, or per-row `+`/`-` ticks. A later
world may justify a general capability independently, but none is smuggled into
this item as Cassowary costume. The document Room remains untouched; the raster
material belongs to summoned chrome, not prose.

Verification follows the shared-renderer claim as seriously as the Cassowary
appearance. Pure laws cover the new cap defaults and the full surface roster,
prove the index formatter's exact state-derived output, keep facet drawing and
hit spans in agreement, and sweep placard/pane geometry across narrow and wide
canvases. A source/roster law rejects world-identity branches, and applying each
new capability to a synthetic theme proves the renderer is data-driven; all
non-Cassowary worlds remain byte-identical under inert defaults. Tier-1
captures cover Cassowary's Commands palette at `All`, `Files`, a moved row, a
typed filter and empty results across representative canvas sizes and 1x/2x
DPI. Read geometry from the sidecar and assert tab connection, selection-band
presence, scanline presence/phase and shortcut legibility from PNG pixels.
Because each cap adds a rendering-axis value, run the standing surface-roster
audit and vision-smoke: which category is active, which command is selected,
and whether every footer hint and shortcut remains readable. Measure the
material pass in release mode and prove a static frame does not create
continuous redraw. The final taste call is the placard crop depth and scanline
strength; reverting either is theme data, while removing the shared capability
is not. Read `docs/render.md`, `docs/harness-reach.md`, `THEMES.md` and
`ACCESSIBILITY.md` before implementation.

### 462 — footnotes join the WYSIWYG Markdown model (USER DECISION 2026-08-21; ready to build)

Support the widely used Markdown footnote extension: an inline reference
`[^label]` paired with a definition `[^label]: text` (including continued
indented lines). This syntax is supported in practice by GitHub and many prose
Markdown tools, but it is **not core CommonMark and is not in the formal GFM
spec**; document that portability honestly rather than calling it universal.
Labels are identifiers, not display numbers: rendering numbers references by
first appearance while preserving the authored label and source bytes.

Follow awl's one WYSIWYG rule. Away from the caret, references render as quiet
superscripts and definitions as composed footnote prose; the caret/selection
reveals the exact Markdown needed to edit the affected line. Repeated
references, definitions before references, missing definitions, duplicate
labels and Unicode labels degrade to legible editable source without losing or
inventing text. Add an **Insert footnote** formatting command that creates one
reference/definition pair with a collision-free label and leaves a useful
caret position; the feature remains fully usable by typing syntax manually.
Reference activation jumps to its definition through the shared jump/fold
reveal path, with a return route only if it can remain calm and deterministic.

Export HTML/PDF/Word must preserve the footnote meaning rather than emitting
concealed source or dropping definitions. Test parsing/conceal/edit boundaries
at the purest seam, round-trip exact bytes, and sweep malformed/reordered/
repeated/multiline cases. Render laws cover reference/definition geometry,
selection reveal, variable wrapping and every world × DPI; read
`docs/markdown.md` and `docs/harness-reach.md` before implementation.

### 459 — complete the ordinary-file vocabulary: Trash, Save a Copy, reveal/path, Go to line (USER DECISION 2026-08-18; slices 2–4 LANDED; slice 1 waits for item 444 residual 2)

🟢 LANDED — slice 3 (Reveal in File Manager + Copy File Path), `cdd5a1bc`
🟢 LANDED — slice 4 (Go to Line), `0d95045d`
🟢 LANDED — slice 2 (Save a Copy), via the merge recorded by
`git log --grep 'item 459'`

**Slice 2 landed on `main`.** Save a Copy uses one folder-then-filename
journey with truthful `save a copy to` / `save a copy as` labels. It writes
the source document's exact disk bytes without changing its path, working-set
identity, history, caret or undo state. The no-clobber path is atomic against a
destination created after preflight; the ordinary macOS save panel retains its
explicit overwrite-confirmation contract.

**Slice 4 landed on `main` (`0d95045d`).** Go to Line is a numeric row inside
the existing unified Go to… overlay, not a parallel navigation system: it
carries the destination buffer's line count (`OverlayState::goto_line_count`
/ `attach_line_jump`), refreshes its single fixed row live from the typed
query on every keystroke (`refilter`'s `sync_goto_line_row`), and its own
accept-path gate (`selected_is_line_jump`) is the numeric sibling of the
existing `selected_is_heading` gate. Both resolve through the SAME shared
jump owner (`Effect::JumpToLine`) the Headings lens already uses, so caret
placement, fold reveal and follow-scroll are not reimplemented. First,
middle, last and out-of-range lines, wrapped text, Unicode and a folded
destination are swept through `--keys` with the sidecar proving caret and
scroll state (`src/capture/tests/goto_line_jump.rs`).

**A real lens-leak bug surfaced and got fixed along the way** (found by the
lane while writing the lens-scoping law, not briefed): the generic "Files"
bucket predicate (`!heading && !is_dir`) accidentally also claimed the
line-jump row, since it wasn't gated on anything but those two flags.
`filter.rs`'s `retain_visible_rows` now scopes the row to `facet_lens == 0`
explicitly, mirroring how `GotoHeading` is already scoped to its own lens —
watched the law go red on the exact bug, then green after the fix; the
accept-path wiring itself was separately mutation-tested by disabling it and
confirming both the unit and capture-level laws go red on that regression.
`overlay_nav.rs`, `app/apply.rs`, `overlay/nav.rs`, `overlay/state.rs` and
`render/benchsuite/scenarios.rs` crossed their frozen code-health high-water
marks with this slice's own additions; each raise verified against the
merged tree.

**Item 459 now has only slice 1 (Trash) open.** Slice 1 has an explicit dependency
(item 444 residual 2's zero-document state) for its final-document case.

**Slice 3 landed on `main` (`cdd5a1bc`).** `CopyFilePath` and
`RevealInFileManager` join the palette and item 444's shared filename context
menu, both gated off (`PaletteGates::named_file`) for an unnamed scratch
document rather than fabricating a location. Reveal reuses the export
door's own `App::reveal_path` gate — no second live-only implementation —
with an explicit `#[cfg(not(target_arch = "wasm32"))]`/wasm-stub split so
the match stays exhaustive on the browser target, which never surfaces the
command at all (native-only in the catalog). `Effect::RevealInFileManager`
is recorded (intercepted, never performed) by `--keys` replay, the same
external-handoff shape `Export`/`FollowLink` already use. Copy File Path
puts the absolute native path on the system clipboard, asserted by exact
text.

Two real bugs the lane caught before landing: a non-hermetic clipboard test
that read the real host's scratch clipboard stash instead of an injected
fake, and a wasm build break in the `RevealInFileManager` effect arm (the
match needed its browser-side stub, not just the native gate). Both fixed
and reverified. `actions.rs`, `app/apply.rs`, `commands.rs` and `replay.rs`
crossed their frozen code-health high-water marks with this slice's own
additions; each raise verified against the merged tree, not the branch's
pre-rebase number.

awl is a complete home for ordinary Markdown files, and the current file verbs
stop one step short of that promise. Add the five agreed capabilities below as
one coherent filesystem-completeness pass. They are independently shippable;
do not hold the small platform/path and navigation slices behind the larger
destructive-file lifecycle work.

1. **Move file to Trash.** Add the missing recoverable sibling of Rename,
   Move and Duplicate. Use the OS Trash abstraction — never permanent deletion
   and never a home-grown trash folder. The command targets the named buffer
   when invoked from item 444's working-set context menu and the active document
   from the palette; both doors dispatch one action. Run the existing
   save/external-change/conflict gate first. A dirty or conflicted document is
   never discarded or overwritten. A successful trash closes/removes that
   buffer and selects the same successor the close owner would. **Dependency:**
   trashing the final open document waits for item 444 residual 2's honest
   zero-document state; until then, either land only the non-final case with a
   truthful refusal for the last document or sequence the full slice after that
   residual. Confirmation is owed only where the document is dirty/conflicted;
   an ordinary clean file goes directly to the recoverable OS Trash.

2. **Save a Copy…, not Save As….** Write the current Markdown bytes to a
   user-chosen destination while preserving the current document's path,
   working-set identity, history ownership, autosave target, cursor and undo
   timeline. This is the useful Markdown "export" behavior without inventing
   Export Markdown or ambiguously overlapping Rename/Move. No-clobber and
   overwrite confirmation follow the platform save-panel contract. A copy is a
   snapshot, not a newly active document and not a second live buffer unless the
   user opens it later.

3. **Reveal in File Manager + Copy File Path.** Add both to the palette and
   item 444's shared filename context menu. On macOS the visible label may read
   **Reveal in Finder**; elsewhere use the platform-neutral file-manager label.
   Reuse the existing live-only reveal owner currently exercised by exports,
   generalized to any document path. Copy File Path puts the absolute native
   path on the system clipboard. Both commands are gated off for an unnamed
   scratch document rather than fabricating a location. They perform no file
   mutation and never change the active folder.

4. **Go to Line….** Add a line-number route inside the existing unified **Go
   to…** task rather than a parallel top-level navigation system. Accept a
   one-based line number, clamp or clearly refuse out-of-range input, move the
   caret through the shared jump owner, reveal any enclosing fold, and scroll
   the destination into view. Preserve the existing Headings lens; this is its
   numeric companion for long prose and light code.

Product boundary: no permanent file tree, bulk deletion, folder deletion,
cross-root file management, Export Markdown command, or second document model.
The filesystem stays real and understandable. The context menu is a discovery
door onto the same catalog actions, never a second implementation.

Verify each slice at its purest reachable seam. Trash uses an injected fake
Trash backend and sweeps active/parked × clean/dirty/conflicted ×
last/non-last, proving a failed Trash leaves both disk and working set intact.
Save a Copy drives a live headless `App` over `InMemoryFs` and asserts the
destination bytes while every source identity/state field remains unchanged.
Reveal is live-only and must remain suppressed on a headless surface, matching
the export law; path-copy asserts exact native clipboard text and the unnamed
gate. Go to Line sweeps first/middle/last/out-of-range lines, wrapped text,
Unicode and a folded destination through `--keys`, with the sidecar proving
caret and scroll state. Read `docs/platform.md` and `docs/harness-reach.md`
before implementing or promising captures; render-touching slices receive the
standing vision-smoke and DPI/world audit required by policy.

### 444 — the working set becomes visible: a margin buffer stack (USER DECISION 2026-08-16; residual 1 LANDED; residual 2 🟡 IN PROGRESS — item-444-zero-document (codex), branch codex/item-444-zero-document)

**Landed on `main`** (full sha list in `git log --grep 'item 444'`): the
`--seed-tree` capture door; the `WorkingSet` module; the cross-root ownership
fix; the resting-stack render (N=1 byte-identical by construction, 40/40
proven non-vacuously); sidecar exposure + click-to-switch (`buffers` gains
`files[]`/`active_index`, `SCHEMA_VERSION` 203→204, through the same
`App::load_path` door every picker/daemon handoff shares); and now **⌘W as
the one true removal owner** (`9213f16d`). `App::close_buffer(key)`
(`src/app/files/close.rs`) is reached by both ⌘W and a stack row's close
zone, resolving to the active entry or a parked one without the caller
choosing. Three pieces that didn't exist before now do, all law-tested with
captured mutation panics: `DocumentSession::entry_unsaved` (save, generalized
off any entry, not just the active one), a parked-entry conflict gate that
**refuses** rather than latching into the single unresolved-conflict slot
(pointing that slot at a parked path would let a later "Save your version"
overwrite the wrong document), and `notify_close_waiters(key)` (daemon
notify, generalized off the active buffer). Closing the last file saves and
notifies but removes nothing — that's residual 1 below, not a bug.

**New gap this landing opens, sequencing decided 2026-08-18 (merge now,
affordance next):** the stack row's right-edge close zone is now WIRED to
`close_buffer` — always lossless (saves first, refuses on conflict) — but
still draws NOTHING. No chrome hover-tracking exists anywhere in the margin
outline (it's click-only too), so a click near a row's right edge now
silently closes that file instead of switching to it, with no visual cue
distinguishing the zone. Landed anyway on the user's explicit call, with the
hover-reveal × treatment owed as the next residual, not deferred indefinitely.

**Two smaller findings, still carried forward:**
- `gutter` in the sidecar (the single name/project fact) is stale
  *documentation* once the stack draws at N≥2 — its doc claims "exactly as
  drawn" but the pixels show a whole stack. Whoever next touches sidecar
  `buffers`/`gutter` reconciles the doc, or the field, deliberately.
- The scratch buffer is closeable (dirty scratch refuses like any other
  entry, the successor search skips it) but still has no *activation* door
  anywhere (`load_path` takes a path; `previous_path()` returns
  `Option<PathBuf>`) — a scratch row still silently swallows a switch click.
  Feeds residual 2 (zero-document), which needs the same "no path yet"
  reasoning anyway.

**Two facts worth knowing before the next residual touches this area:**
`Finish file` is now a misnomer for a command that CLOSES — renaming it
touches the palette, GUIDE, REFERENCE and the `finish_file` config key, so
only its *description* was corrected this round, not its name. And
`load_path` flushes autosave before parking, so under the default config a
parked entry is essentially never dirty — the parked-conflict path is mainly
reachable via `autosave = false`.

**Residual, in the order the landed work sets up:**

1. **Hover-reveal close affordance + folder-line distinction (USER-REPORTED
   2026-08-18 on the live app). LANDED.** The user chose a folder heading
   above the file rows and a one-stage close mark on whichever row is under
   the pointer, including the current document. The mark is transparent at
   rest in a stable pre-shaped lane, so labels do not shift. This landed via
   the merge recorded by `git log --grep 'item 444'`. (b) row cursor and (d)
   Wagtail legibility landed earlier (`f8f3fb4c`): stack rows now earn the pointing hand via the
   same `CursorContext` roster/no-wildcard law the outline rows already use
   (`gutter_stack_hit`, no parallel hit-test), and the active row's label
   ink routes through `theme::selected_row_secondary_ink`/`surface_selected`
   — the same "ink over a filled plate inverts" mechanism `one_bit.rs`
   already proves for the picker/toast — so Wagtail's selected file is
   legible again, presence + legibility floors both mutation-proven
   independently. The one-file case remains byte-identical.

2. **Zero-document state** — the largest remaining piece; `DocumentSession`
   would need an optional active slot, and every subsystem this item names
   (renderer, actions, autosave, session, title, accessibility tree,
   sidecar) needs an honest `no active document` representation.
3. **Overflow windowing, expanded/grouped cross-project view, Move
   navigator** — deliberately last: this item's own text asks for captures
   judged by the user before any of these get built. The resting stack now
   renders, switches and closes, so a >5-file / cross-project capture set can
   finally be produced — but the exact windowing/grouping rule still needs
   the user's eyes on that set before it's built, not guessed ahead of it.

Design session 2026-08-16. The user works between a couple of files and wants
tabs' affordance without tabs: ⌃Tab covers two files but not three, and a
purely keyboard-summoned working set was judged too invisible for the widened
audience — for a non-programmer, tabs are intuitive because they are visible
and clickable. A persistent tab strip stays out (PHILOSOPHY §1 names it). The
shape chosen instead lives inside DESIGN §5's existing margin grammar: the
bottom-left identity (filename + folder — "position in the filesystem") WIDENS
into a quiet stack of the open files — current file on a soft Arc-like selected
row, the others dimmer, click to switch. Not a new chrome region; an existing
orientation surface deepens by a line or two, and with one file open it is
byte-identical to today. The user chose bottom over above-the-outline, and chose
the visibly row-shaped Arc-like treatment over bare labels floating in the
margin.

The stack is PROJECT-SCOPED, not one global pile of arbitrary paths. The active
folder remains load-bearing: it owns Go to's recursive file corpus, New
document, Move and export destinations, git identity, and the session's folder
context. All files below that root may coexist in the stack, including files in
different subfolders; a nested file reads by its shortest unambiguous
ROOT-RELATIVE path (`journal/field-notes.md`), not by a leaf name that throws
away the location. Switching folders through Go to changes which stack is
visible, while each folder's working set stays parked and returns when that
folder becomes active again — the Arc-Space shape, without a permanent space
switcher. Opening or activating a buffer from another root must restore that
buffer's remembered project context in the same transition: the current
registry can preserve cross-root buffers while `load_path` leaves the old root
active, which would make the document and bottom folder identity disagree.
Close that seam here; never present it as intentional mixed-root behavior.

Nested files render as a FLAT working set, never a miniature tree. The root
heading names the active folder (`notes`); each row shows the filename and, when
it lives below the root, its root-relative parent in quieter ink
(`journal/ field-notes.md`). Preserve the leaf and the nearest useful location
when space is tight (`research/…/ final-draft.md`). No indentation, disclosure
arrows or expandable folders: those marks promise a file tree this surface does
not provide.

What keeps it tabs-but-not-tabs: no reordering and no drag; hides under space
pressure exactly as the outline does; inherits every §5 margin law (hug the
column, quiet label treatment, never change the prose column's geometry).
STABLE OPEN ORDER within a project, not MRU — reordering on every switch would
jitter the margin while the pointer is reaching for a row.

Closing is part of the feature, not an assumed affordance. Today ⌘W's
"Finish file" saves, notifies any daemon waiter, and switches to the previous
buffer but leaves the finished buffer parked; that is not close. Give the
working set one true removal owner: ⌘W closes the active file after the same
lossless save/conflict gate and still notifies a waiter, and a row exposes the
same action through a quiet hover-only close target (never a persistent column
of crosses). The close target appears when the pointer enters the row's RIGHT
close zone; the rest of the row remains the larger switch target, so merely
moving through the list does not fill it with controls. Closing an inactive row
closes that named buffer without first activating it; a dirty or conflicted
buffer is never discarded.

The stack makes the existing file verbs discoverable without becoming their
implementation. Secondary-click a row to open the SAME filename context menu
the bottom identity already owns: Rename file…, Move file…, Duplicate file,
Version history…, plus Close file. Pointer and keyboard routes dispatch the
same catalog actions; Cmd-P → Move file… opens the identical destination
navigator. A row action targets the named buffer rather than silently switching
documents merely to make an active-buffer-only function convenient.

Move stays deliberately bounded to the source file's owning root. Its summoned
folders-only navigator says `move <filename>`, shows the current root-relative
destination, descends/ascends through folders, offers an explicit `New folder…`
row and a `Move here` action at every level. A successful move keeps the stack
slot stable and updates its quiet parent path. No drag-to-move, bulk selection,
folder moves or cross-root moves in this item: those are file-manager machinery,
and a tiny contextual stack is the wrong place to imply them. Moving never
silently rewrites Markdown or incoming links; when the file contains relative
links/images, the completion feedback states that their paths may need review.

Closing the LAST file enters a real ZERO-DOCUMENT state, not a fake unnamed
buffer and not a closed application window. Today `DocumentSession` always owns
one active `Entry`, so this is product machinery rather than empty-state copy:
the renderer, actions, autosave, session, title, accessibility tree and sidecar
must all represent `no active document` honestly. The world remains; the page
surface disappears because drawing blank paper would imply an unsaved file. A
small calm start surface offers exactly `New document` and `Go to…`; the active
folder remains remembered, so either route has an unambiguous context. First
launch still opens the authored Welcome document — zero-document is reached by
an explicit close, never used as a replacement tour.

Visible overflow is bounded independently of the registry's safety cap.
PROTOTYPE five file rows plus a quiet `+ N more…` row. Accepting it EXPANDS the
same bottom-anchored stack UPWARD into a transient scrollable working-set view;
it does not detour through Go to, permanently lengthen the resting margin, or
turn the whole window into a sidebar. Wheel/trackpad motion over the expanded
list scrolls that list, Esc/click-away collapses it, and choosing a file returns
to the resting stack. The active file must always be represented in the visible
five; prototype the least-jittering stable-order window and put the >5 shots to
the user before fixing its exact windowing rule. Do not silently evict a sixth
file merely to make the drawing easy: the existing registry's
clean-LRU/never-dirty eviction is a memory safety bound, not a visible-stack
product rule.

PROTOTYPE the cross-project half in that SAME expanded view, taking the useful
part of Codex's sidebar grammar without adopting its permanent project-manager
shell: group retained open files under folder headings; mark the active file's
group clearly; keep only the active folder's group in the resting stack. The
ONE generic `+ N more…` row counts every hidden open buffer — same-root overflow
and other roots alike — and expands this grouped view; never add a parallel
`N files in other folders…` row. Clicking a file in another group atomically
restores its remembered project root AND activates its buffer, after which Go
to, New, Move, export and the resting stack all operate in that folder. This is
why the main folder remains meaningful: it is the ACTIVE group, not a claim that
no other folder may retain buffers. Do not show multiple groups persistently;
awl is not a project manager.

No new digit shortcuts. Once the working set can be grouped, partially hidden
and scrolled, “the third file” has no stable, obvious meaning across resting and
expanded states. ⌃Tab Last file stays exactly as shipped, `C-x b` may remain its
quiet Emacs alias, and Go to… remains the complete keyboard route.

Contract edits owned here: DESIGN §5's margin roster gains the stack as a
member with the outline's own license (may click-to-switch; orientation, not
management UI). PHILOSOPHY §1 is untouched — this is not a strip.

Capture-prototype FIRST, judgement before machinery: gallery shots across a
few worlds × {one root with nested files, only one file in the active root,
more than five files, zero documents}, bottom-left, including right-edge
hover-close, collapsed overflow, expanded scrolling and grouped cross-project
states, put to the user before laws or bindings land. Harness reach
(docs/harness-reach.md read for this clause):
`last_buffer` and the multi-file working set are App-owned — tier-1 captures
classify them Unsupported — so the prototype and every switching claim drive
`--screenshot-app`, which skips nothing and is hermetic (sandbox seeded from
the named CLI paths); that hermeticity also satisfies the path-leak rule,
because the margin photographs filenames — shots run against seeded roots
only, never the ambient ones. Verify: the sidecar gains the working set (open
files + active index) through the one redacting writer; a `--screenshot-app`
law opens A, opens B, accepts A's stack row and asserts the active buffer
changed AND the stack's drawn order did not. A pixel law asserts that the
active row is
distinguishable from its dimmed siblings, with a companion presence floor
(a stack faded to the page must fail, not pass happier); cross-root activate
restores the matching project/root before the frame and the gutter never names
the old root; nested same-root files render distinct relative labels; close
removes exactly its target and never loses a dirty/conflicted buffer; overflow
keeps the active file represented and its `+ N more…` count exact; expanded
scroll remains inside the working set; closing the last file produces no active
buffer, no page surface and exactly the two start actions without changing the
remembered root; the one-file case before close is byte-identical to today's
margin across the world roster; context-menu Move and palette Move dispatch the
same action; moving a nested file keeps its stack slot and updates its relative
label, and never crosses the source root. Generated reference rows are
spot-checked against the dispatch they claim.

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
