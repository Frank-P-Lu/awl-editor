# Headless App and semantic capture

Read for App-owned state, seeded recovery/history, or accessibility snapshots.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

## Live-`App` capture (`--screenshot-app`) — the tier-2 oracle

Ordinary screenshot replay drives the **shared core**: real chords, the real keymap, the
real `apply_transition` — but no `App`. Effects only the live `App` can perform
are therefore classified **Unsupported** and skipped, and the sidecar reports the
skip rather than the state. `docs/harness-reach.md` is the map of exactly which.

`--screenshot-app OUT.png [file]` drives the **same `--keys` chord stream into a
real headless `App`** instead — `dispatch_pressed_key` → keymap resolve →
`App::apply` → the live effect interpreter, the code a physical keypress takes —
and then writes an **ordinary** PNG + sidecar from its state.

```sh
# Flip the Settings workspace's Keymap row and read the result from the sidecar.
cargo run -- --screenshot-app OUT.png --root /some/proj \
  --keys "s-, Tab Down Down ... Enter"
# OUT.json: "driver": "live-app", project.keymap_flavor: "emacs",
#           overlay.bindings[<row>]: "emacs", replay_skips: []
# The same spec through --screenshot: keymap_flavor stays "native" and
# replay_skips carries {"effect":"setting_toggle","action":"Newline"}.
```

**Same schema, same writer, same blocks.** There is one sidecar writer
(`capture::sidecar::write_sidecar`) and one per-frame fold
(`run::fold_capture_state`, shared with the storyboard stepper); this mode adds
neither. The only differences in the artifact are the top-level `driver` field
and the `semantic` tree, which only a live `App` can fold.

**`--capture-size`/`--capture-dpi` compose.** `LiveAppSpec` carries
the canvas + dpi flags onto the same `CaptureOpts` this door hands the ordinary
`capture_with` renderer, so the meaning is identical to every other capture
door: a `WxH` PHYSICAL canvas (default 1200x800) at `--capture-dpi N` (default
1.0) is a `(W/N)x(H/N)` LOGICAL window. This is the geometry axis the live
`App` door needs most — `docs/harness-reach.md`'s "Three asymmetries" section
covers the settings-picker width-budget defects only a narrow live-`App`
canvas can reach.

```sh
cargo run -- --screenshot-app OUT.png --root /some/proj --capture-size 640x800 \
  --keys "s-, Tab Down Down Down Down"
# OUT.json: "driver": "live-app", canvas: { "width": 640, "height": 800 },
#           overlay.window carries the REAL narrow-canvas Settings card, not
#           the 1200x800 default one.
```

The per-frame render OVERRIDE hooks stay refused, not silently dropped: `--sel`
/`--zoom`/`--scroll`/`--preedit`/`--search`/`--search-case`/`--search-replace`
and `--default-folder` all error with "not honored by the chosen capture mode"
rather than parsing and doing nothing — `LiveAppSpec` has no slot for any of
them, and for the first group that is deliberate (see [remaining limits](../harness-reach.md#remaining-limits)): the live App owns that state via real driving, and
an override would misrepresent the editor being photographed. `--root`/
`--workspace` were already threaded and stay so.

**Starting from state awl already had — `--seed-data DIR`.** The
sandbox is seeded from exactly the paths the command line names, and awl's own
data root was not one of them, so a mode whose whole premise is remembered state
— an unresolved-change record, a scratch stash, a session, a history log — had no
way in. `--seed-data DIR` recursively carries `DIR`'s files into the sandbox at
`fs::data_root()/<relative-path>`, where awl's own readers look for them —
including `history/<hash>.log`. The deterministic walk skips symlinks and
refuses more than 256 files or 4 MiB. Hermetic-door
only (`--screenshot-app`, `--semantic-json`, `--storyboard`, or `--screenshot
--keys --strict-replay`); naming it anywhere else is an error rather than a
silent no-op, because a run that named a store and did not get one would
photograph the wrong starting state.

```sh
# Start ALREADY CONFLICTED: the record says awl was holding `MINE` for draft.md,
# and the file on disk has since moved.
printf 'awl-unresolved-change 1\n%s\n%s' "$PWD/draft.md" "$MINE" > seed/unresolved-change.md
cargo run -- --screenshot-app OUT.png draft.md --seed-data seed \
  --keys "s-p R e v i e w Enter Down"
# OUT.json: gutter.changed: true, overlay.mode: "conflict",
#           overlay.preview_view: "mine", text: the user's own version.
```


## Semantic state without a GPU (`--semantic-json`)

```sh
cargo run -- --semantic-json [file] --keys "s-p t y p e"
```

Prints the same semantic tree the native AccessKit adapter projects and a
live-`App` sidecar embeds — roles, accessible names, values, the ONE focus
owner, relationships and the actions each node really supports — as pretty
JSON on stdout, with no window, no surface and no GPU. Hermetic on the same
`scenario` sandbox `--screenshot-app` uses, and it composes with `--keys`,
`--root`, `--workspace` and `--config`. It is a state door, not a capture
door: it writes no PNG, so it does not compose with `--screenshot*`.

This is how an agent reads awl's UI without a display. The field-by-field
shape is documented with the sidecar's `semantic` field below.

**Hermetic, and for a stronger reason than a strict replay.** A live `App`
*performs* the writes a replay only records — a settings persist, an autosave, a
save. So this mode is a scenario door: `args::parse_args` swaps the process fs to
the seeded `scenario` sandbox before the config loads, exactly as a storyboard
run does, and `App::new_headless_capture` deliberately constructs on that fs. The
PNG + JSON go out through `std::fs`, the documented sandbox bypass every capture
deliverable already uses.

**What it still cannot reach.** Tier 3: no window, no surface, no event loop
(`gpu` is `None`, so the harness renders the App's buffer through its own
offscreen pipeline, exactly as `--screenshot-frames` does). The
`&ActiveEventLoop` census in `app::tests::source_audit` is the exact list of what
that costs. Nothing here changes the sidecar-vs-appearance tripwire either: this
is still a **state** oracle, and appearance claims are still asserted over the
PNG's pixels.
