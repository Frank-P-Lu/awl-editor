# Window and input reach

Read for window, surface, native input, and runtime-only boundaries. Headless App state does not imply platform dispatch coverage.

[Guide](../harness-reach.md) · Paths in code examples are relative to the repo root.

## Tier 3 — the live-only census, exactly

Every function under `src/app.rs` + `src/app/**` whose signature takes an
`&ActiveEventLoop`. Fourteen, in five files, and the list is not maintained by
hand: `app::tests::source_audit::the_active_event_loop_census_is_exact_and_the_input_chain_is_free_of_it`
scans the source and fails on any new one.

| Where | Functions | Why the loop is genuinely needed |
| --- | --- | --- |
| `app.rs` | `drive_gpu_soak` | `--soak-gpu` drives a real window |
| `app/frame/accessibility.rs` | `FrameRuntime::install_accessibility`, `AccessibilityRuntime::install` | the AccessKit adapter binds the real loop + window, and must do so BEFORE the window becomes visible (macOS caches a newly ordered-in window's accessibility parent) |
| `app/gpu_recovery.rs` | `rebuild_gpu` | recreates the window-bound renderer |
| `app/lifecycle.rs` | `user_event`, `resumed`, `suspended`, `window_event`, `exiting`, `about_to_wait` | winit's own `ApplicationHandler` signatures |
| `app/window.rs` | `handle_gpu_fault`, `handle_gpu_frame_outcome`, `on_resized`, `on_redraw_requested` | rebuilds the surface, sets `ControlFlow` |

### Surface- and platform-only export doors

Two export paths sit at tier 3 without taking `&ActiveEventLoop`, so they are
not members of the fourteen-function signature census above:

| Platform / door | Owner | Why no headless oracle reaches it |
| --- | --- | --- |
| macOS File-menu Export | `App::handle_menu_event` → `run_native_panel` → `export_via_platform_panel` | Opens a real modal `NSSavePanel`. The surface gate deliberately refuses a GPU-less `App`; otherwise `--screenshot-app` and tests would block their main thread waiting for a human. Chords and command-palette export remain tier-2-drivable because they use the in-app destination navigator instead. |
| Linux drawn-menu Export | `App::menubar_press` → `App::apply` | The click door starts with the live renderer's menu hit-test. macOS never draws this menu, so this platform arm cannot be exercised on the design host; a headless state capture can prove the routed export transition, but not that a real Linux menu click reaches it. |

The macOS arm needs a person to accept or cancel the platform panel. The Linux
arm needs a real Linux window and pointer click. Neither may be claimed from a
`--keys` or `--screenshot-app` capture.

### An IME COMMIT has zero `--keys` vocabulary either, and has its own tier-2 door

`--keys` chords replay through the real keymap, and a COMMITTED IME composition
(`WindowEvent::Ime(Ime::Commit)` — the finalized text of a CJK or dead-key
composition) never touches the keymap at all. There is no chord that spells it
on any capture door, tier 1 or tier 2, so for a long time nothing headless could
drive the one insertion path that bypasses `App::apply`.

`App::commit_ime_headless` (`app/press.rs`, beside `press_spec_headless`) is that
door. It is a NARROWING rather than a stand-in: it hands the same `winit` event
to the same `App::on_ime` that `lifecycle.rs`'s `WindowEvent::Ime` arm hands it
to, so a headless commit and a physical composition are the same code path minus
the platform input method.

**What it does NOT open.** There is still no `--keys` token and no sidecar for
it — `--screenshot-app` drives chords only, so a capture cannot photograph a
frame an IME commit produced. A Verify clause about IME behaviour is a Rust
assertion on `App` state (`app::tests::read_only_surface` is the worked example),
not a capture. Composition PREEDIT is a different matter and has its own
deterministic render hook, `--preedit`.

### A macOS MENU KEY EQUIVALENT is not a key, and no capture door can spell it

AppKit answers a menu item's key equivalent in `performKeyEquivalent:` against
the main menu **before** the key window ever sees the event. So on macOS ⌘A is
not a `winit` key at all: it fires Edit ▸ Select all, and
`App::handle_menu_event` routes that id into `App::apply` as an `Action`. A
menu or context-menu CLICK and a palette row's `Effect::RunAction` arrive the
same way — an `Action` with no keystroke behind it.

Every capture door enters through the key drivers instead. `--keys` (tier 1)
and `--screenshot-app` (tier 2) both replay chords through the real keymap, and
a summoned surface's key guard (`search::keys::intercept`) consumes those before
`apply_transition` — so a capture can never observe what an action-only door
does while such a surface is up. That gap is exactly where ⌘A-in-Find selected
the whole document for the life of the panel.

**The tier-2 door.** Drive `App::apply(action, …, Door::Menu)` — or the real
`App::handle_menu_event(id, …)` where the id → action resolution is itself part
of the claim — on a live headless `App`, and read the buffer and the surface's
own field back. `app::tests::summoned_field_actions` is the worked example; its
verb roster comes from `menu::edit_menu_actions()`, which is the Edit menu's own
rows, i.e. precisely the set that gets real key equivalents.

**What it does NOT open.** There is no `--keys` token and no sidecar field for a
fired menu item, so a Verify clause about "⌘A while the panel is up" is a Rust
assertion on `App` state, never a capture — and the PNG half of such a claim
(does the field's selection band actually paint?) is a render-tier pixel test
over `ViewState`, `render::tests::panel_field_selection`.

### Mouse-drag gestures have zero `--keys` vocabulary

`--keys` chords replay through the real keymap (`keymap.rs` -> `Action`), and
the keymap carries no mouse events at all — a click, a drag, or a held
edge-scroll cannot be spelled as a chord on any capture door, tier 1 or tier
2. So the *pixel geometry* a drag gesture computes (a hit-test, an
overshoot-derived rate, the scroll step it advances) is provable at the
`TextPipeline` level with a real test-GPU device (`render/tests/`) — the
device gives real shaped-glyph geometry without a window — but the gesture
ITSELF (a live `CursorMoved` stream, and the `about_to_wait` re-arm that keeps
a held drag scrolling once the pointer stops moving) can only be driven by a
real window and pointer. Scroll-on-drag (`App::step_drag_scroll`,
`app/input/mouse/document.rs`) follows this shape exactly: the overshoot-to-rate curve
and the composed overshoot -> scroll -> hit-test tick are pipeline laws
(`render::tests::drag_scroll`); the App wiring that drives them from a real
drag is live-only, flagged for human confirmation like every other pointer
gesture on this map.

**The working-set row DRAG follows this shape exactly, one layer up:**
`App::gutter_stack_click`/`on_row_drag`/`end_row_drag` (`app/input/gutter.rs`)
arm, track and settle a press-and-drag reorder of a margin stack row, but the
gesture ITSELF — the press, the `CursorMoved` stream past the drag-arm slop,
the release — has no `--keys` chord and is live-only, same as every other
pointer gesture on this map. What IS provable headlessly, at the purest reachable
seam, is everything the gesture computes: the reorder state machine
(`WorkingSet::reorder_in_group`, `WorkingSet::reorder_target`, `WorkingSet::
resting_row_index`) is pure and exhaustively unit-tested in
`workingset/tests.rs`, and the App-side "given a recognized drag-and-drop of
row A to position B" seam (`App::gutter_stack_row_drop`) is GPU-free and driven
directly by row index in `app/input/gutter/tests.rs`, mirroring the existing
click-resolution laws. A Verify clause may assert the POST-DROP order through
`App::capture_opts().working_set` (the same fold `--screenshot-app` writes into
its sidecar) after driving `gutter_stack_row_drop` directly — it may not ask
for a `--screenshot-app` capture of the drag GESTURE itself, which cannot exist.

**The FOLLOW gesture (modifier-click / middle-click on a followable span) is the
same shape, and its outward tail is a second live-only layer on top.** Three
distinct claims, three different tiers, and a Verify clause must not ask the
wrong one for the wrong thing:

1. **Which mouse chord follows, per convention (and per `[keys] follow`
   override)** — pure (`keymap::platform::active_follow_gestures` /
   `follows_link`), swept over the whole `Convention x button x modifier`
   grid — the roster is no longer flavor-gated (middle-click follows under
   Linux `native` and `emacs` alike) — plus the override's own parse/replace
   contract, in `keymap/tests.rs`. No capture door is involved or needed.
2. **What a followable span resolves to, and which typed effect carries it** —
   pure (`markdown::follow::followable_at`, `actions::follow::follow_effect`),
   enrolled from the underline grammar's own predicate
   (`MdKind::is_followable`) in `markdown/follow/tests.rs` and
   `actions/follow/tests.rs`.
3. **The follow ITSELF, end to end, through a capture door** — reachable, but
   only through the KEYBOARD door onto the same seam, and only for the LOCAL
   arm. `Effect::OpenPathAtLine` is Applied at both tiers, so following a
   relative link is readable from `buffers.active` under `--keys` AND
   `--screenshot-app`
   (`run::live_app::tests::following_a_relative_link_lands_the_destination_in_both_drivers_sidecars`).
   `Effect::FollowLink` is **Intercepted** — the `open`/`xdg-open` spawn is the
   live-only tail, and a `--screenshot-app` capture that followed an external
   URL would spawn a real browser, which is exactly what that classification
   exists to prevent. Assert the external arm at the effect seam; never ask a
   capture for it.

The PRESS that starts any of this — a real `MouseInput` with modifiers held, the
hover that turns the pointer into a hand — has no chord on any door and is
live-only, flagged for human confirmation like every other pointer gesture here.

### The switch-project Recent lens is EMPTY at every capture door, on purpose

The recent-projects MRU is live-only persisted state, and the headless path
feeds `overlay::browse_level` an empty list (`main/run/chord.rs`) as a determinism
gate — a capture whose rows depended on which projects this machine had opened
would not be byte-stable. `--screenshot-app` does **not** widen this: it is the
one live-only fact a live `App` in a capture still does not carry, because the
gate sits in the level builder both doors share, not in the `App`.

So the lens's whole enrolment — a remembered root becoming its own whole-path
row, the MRU ordering, the level-/home-relative label, the refusal of a root
that no longer names a directory — is **tier 1 only**, asserted at the unit seam
with the MRU injected (`overlay::tests::project`, `actions::tests::pickers_nav`).
**A Verify clause must not ask for a capture of a POPULATED Recent lens: no such
artifact can exist.** What a capture can witness is the lens with nothing in it —
the strip, the landing on All, the empty-state copy.

### The personal dictionary's rows are EMPTY at the tier-1 door

`ReplaySession` builds its own `SpellChecker` (`run.rs`), and that constructor's
own doc says the personal dictionary starts EMPTY and the caller loads it via
`set_user_words` — a call **only the live `App` makes**
(`App::load_user_dictionary`). So the "Personal dictionary…" picker's
summon-time gather in `main/run/chord.rs` questions a checker that has never
been told a word, and a plain `--keys` capture photographs an empty list no matter
what `dictionary.txt` holds. The picker's own SUMMON is real at tier 1 — the
palette row resolves and the card opens; only its ROWS cannot exist there.

Unlike the switch-project Recent lens above, **`--screenshot-app` DOES widen
this**: a live `App` loads the file at startup and every add/forget keeps the
two in step, so the tier-2 door photographs real rows.

**A Verify clause must not ask for a tier-1 capture of a POPULATED personal
dictionary: no such artifact can exist.** Ask for `--screenshot-app`, and ask
for it against an explicit `--config` and `--root` — these rows are the
operator's own added words, and this repo is public.

Leaving the tier-1 gather empty is a decision rather than an oversight, for the
same determinism-and-privacy reason the Recent lens is gated: a replay that read
the ambient `dictionary.txt` would put whoever ran it into the capture.
`capture::tests::personal_dictionary_journey` holds the ceiling where it is, and
goes red the day someone moves it.

### The `cjk_priority` Han tiebreak is tier 3 for the RENDER, tier 2 for the READOUT

Measured, not inferred. The config key reaches the App: a `--semantic-json
--keys "Cmd-,"` capture reads Settings' "Ambiguous CJK reads as" as `Japanese`
under a `ja`-first `--config` and `Simplified Chinese` under a `zh-Hans`-first
one, so the loader, the sticky-global seed and the row's readout are all wired
and tier-2 provable.

**The RENDERED FACE is not.** Both capture doors paint through
`capture::capture_with`, whose `ViewState` comes from `ViewState::base()` — and
that pins `frontmatter::DEFAULT_CJK_PRIORITY`; nothing under `src/capture/`
assigns `cjk_priority` at all. `App::sync_view` (the one construction site that
DOES read `Config::cjk_priority_or_default`) early-returns without a GPU, so no
test in this tree ever builds a `ViewState` carrying a configured ladder either.
Consequence, confirmed by pixels on a bare-Han fixture under `--screenshot-app`:

| Varied | PNG |
| --- | --- |
| the document's own frontmatter `lang:` tag (`ja` vs `ko`, same byte length) | **differs** — ladder step (a) is live, and the pixel oracle is sensitive |
| `--config cjk_priority` (`ja`-first vs `zh-Hans`-first vs `ko`-first), untagged | **byte-identical** — ladder step (c) never reaches the capture pipeline |

So a Verify clause may not ask a capture to witness the Han tiebreak. Assert
step (c) at the pure seam (`script::resolve_font_id`,
`render::spans::add_script_spans`); the config→render leg is live-only and
needs human confirmation. Threading the ladder into the capture pipeline would
move this to tier 2, and is a real follow-up.

**The DOCUMENT-scoped evidence tier sitting ahead of step (c)
(`script::cjk_evidence`, folded in by `script::effective_cjk_priority`) is on
the OTHER side of this exact boundary — tier 1, not tier 3.** It is a pure
function of the buffer's own text, read straight from the `text: &str`
`TextPipeline::set_text_incremental` already holds, with no `Config`/
`ViewState` hop at all — so an ORDINARY `--screenshot` (no `--config`, no
`--screenshot-app`) exercises it in full, including its interaction with the
pinned `DEFAULT_CJK_PRIORITY` default the table above describes: an untagged,
bare-Han fixture carrying a simplified-only character (e.g. 这) resolves the
bundled Simplified-Chinese face in the PNG even though the pinned ladder is
`ja`-first, and a mutation that removes the evidence fold (reverting to
`cjk_priority.first()` alone) turns that same fixture back to the Japanese
face — a real, capture-provable regression law
(`capture::tests::i18n_fixtures`). What stays tier-3-only is exactly what the
table above already says: an EXPLICIT, non-default `cjk_priority` setting
still never reaches a rendered pixel through either capture door.

The same law asserts the **input-dispatch chain is empty** — `app/apply.rs`,
`app/input/keys.rs`, `app/input/mouse.rs` and its children, `app/input/drags.rs`,
`app/menu.rs`, `app/probe.rs` may never take an `&ActiveEventLoop` again. One such parameter
re-blinds every transition reachable through it, in one line, and nothing else
in the suite would notice.

Two capabilities have already escaped this tier by being narrowed rather than
stubbed, and they are the pattern to copy: `app::Scheduler` (the
`about_to_wait` debounce/settle body, steppable under a `VirtualClock` — that
is what `--screenshot-frames` drives) and `app::schedule::Exit` (this item).

**`--capture-size`/`--capture-dpi` are honored (item 339 — the identical gap
item 334 closed on `--screenshot-app`, found by that item's own lane while
diagnosing and left alone until now).** Before, `Mode::ScreenshotFrames`
carried no canvas/dpi fields at all and fell into the CLI's plain-`--screenshot`
hook bucket, so both flags parsed, validated as "honored", and were then
rendered through a bare `CaptureOpts::default()` regardless of what was typed —
every N-frame capture was the byte-stable 1200x800 default no matter the
canvas asked for. Worse, `capture::capture_frames_async` itself never called
`pipeline.set_dpi`, so even a correctly-threaded `--capture-dpi` would have
been a no-op at the renderer — a second, independent instance of the same
"accepted and ignored" shape one layer deeper. Both are fixed: `Mode::
ScreenshotFrames` now carries `canvas`/`dpi`, `CaptureKind::ScreenshotFrames`
gives the door its own accurate hook list (canvas + dpi honored; the per-frame
render hooks, `--root`, `--workspace`, `--default-folder` and `--keys` all
refuse loudly — this door's document is a stationary backdrop loaded straight
off disk, with no replay and no project resolution at all, so none of those
has anywhere to land), and `capture_frames_async` calls `set_dpi` exactly
where `capture_async` does. Proved by measuring geometry, mirroring item 334's
form: a document with one long wrapping line renders 12 visual rows at
1200x800, 30 at 640x800 (genuinely more reflow, not a relabelled number), and
12 at 2400x1600 @ dpi 2.0 — identical to the 1200x800 @ dpi-1 baseline, the
documented dpi meaning holding on this door too. Reverting the fix (either
half) reproduces the exact original collapse: every canvas reports 1200x800
regardless of the flags typed.

**Every `Mode::*` capture door, and whether it carries canvas/dpi (item 339's
own audit, so a third gap of this shape does not go unlisted):**

| Door | Canvas/dpi? | How |
| --- | --- | --- |
| `Screenshot` | Yes | `CaptureOpts.canvas`/`.dpi` |
| `ScreenshotMotion`/`-Vertical`/`-Diagonal` | No | refused loudly (`CaptureKind::Motion`), never silently dropped |
| `ScreenshotFrames` | Yes (item 339) | own `canvas`/`dpi` fields → `CaptureOpts` |
| `ScreenshotApp`/`SemanticJson` | Yes / N/A | `LiveAppSpec.canvas`/`.dpi` (item 334); `SemanticJson` renders no PNG, so its own spec always carries `None` and the CLI refuses the flag combination before either mode is built |
| `CaptureTimeline`/`CaptureHeld` | Yes | own `canvas`/`dpi` fields |
| `Storyboard` | No | refused outright — a storyboard run sets no `out`, so it resolves to `CaptureKind::Windowed`, which already refuses both flags |
| `Windowed` | No | a real OS window; both flags refused for the same reason |
| Every `Bench*` mode, `SoakGpu` | No | same refusal as `Storyboard` (none of these set `out` either); each bench's own fixed internal canvas is documented on the `Mode` variant, not driven by these flags |

No third silently-discarding door was found: every other mode either threads
the flags for real or is refused by the existing `unused_hooks` classification
because it never sets `out` in the first place.
