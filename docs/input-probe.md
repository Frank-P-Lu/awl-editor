# Live macOS keyboard evidence

A successful automation call proves that the driver accepted a request. It does
not prove that the intended awl field received it. Use a disposable document,
explicit root and config, isolated data directory, and a uniquely named release
app. Record the tested commit and bundle identity. Never target the shared `Awl`
name when another instance may be open.

Before each sequence, inspect the unlocked desktop and the target's current
accessibility state. Record the attempt identifier, wall-clock time, driver,
bundle/process identity, window, and focused control. If the driver cannot read
the desktop or target, stop that attempt and retain its error. Do not count it
as a delivered key. Keep physical attempts separately identified.

Launch the disposable app with `AWL_FLIGHT_RECORDER` pointing to a fresh log.
Use seeded public text only: existing key/action receipts include typed
characters. The recorder's header identifies the app PID and session start;
its lines use elapsed milliseconds. For each attempt:

1. Open Files, focus its query, and type `shared`.
2. Inspect the field, then send Cmd-A followed by Backspace.
3. Inspect again, then type `ab`. The query should contain exactly `ab` and
   the document should remain unchanged.
4. Retain the driver result, fresh accessibility state and screenshot, and
   the corresponding recorder interval. Repeat in another fresh session.

Read the first boundary that disagrees:

- `focus gained/lost` records awl's window focus, not the external driver's
  chosen target or macOS permission status.
- `winit KeyboardInput` records delivered keys plus modifier and composition
  state. Release events are not presses; named `consumed-by` receipts explain
  the preedit, lone-modifier, search-field and binding-recording guards.
- A macOS menu key equivalent can bypass winit's key-down path. Cmd-A normally
  produces `native-menu received`, followed by `input-state ... door=Menu`.
  Missing `KeyboardInput state=Pressed` alone is not a dropped Cmd-A.
- `input-state before/after` records the action door, document version and
  selection, and overlay kind, Files focus, query byte length, character caret
  and selection. These distinguish a delivered selection action from an
  unchanged field. `rejected-no-document` identifies the admission guard.
- `resolve` and `apply` receipts establish app handling; a fresh screenshot
  and successful present establish that the visible result caught up.

The recorder cannot establish the origin of a native menu action: a physical
shortcut, injected shortcut and menu click share that route. Correlate it with
the external attempt record. A headless `App::apply(..., Door::Menu)` law proves
field semantics but cannot prove OS delivery. `--live-script` similarly enters
the shared dispatch tail directly; it is useful for app/presentation checks,
not as evidence that external keyboard injection worked.
