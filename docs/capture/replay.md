# Replay and strict scenarios

Read when scripting keys, search/replace, or isolated filesystem replay.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

## How to invoke a capture (non-interactively)

The cargo invocation must be prefixed with the toolchain PATH on this machine:

```sh
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
```

Single file → PNG + sidecar:


```sh
cargo run -- --screenshot OUT.png path/to/file.md
# writes OUT.png and OUT.json (sidecar derived by replacing the extension)
```

Scratch (empty) buffer:

```sh
cargo run -- --screenshot OUT.png
```

### Scripting input before the frame (`--keys`)

`--keys "<spec>"` replays a sequence of keystrokes through the **real keymap**
(`KeymapState::resolve` → `Action` → `apply_transition`) against the loaded buffer
*before* the single frame is captured, so the PNG + sidecar reflect post-replay
state. It composes with `--screenshot` and the motion variants:

```sh
cargo run -- --screenshot OUT.png --keys "C-n C-n M->" path/to/file.md
```

Spec grammar — space-separated emacs chords:

- Modifier prefixes: `C-` (Ctrl), `M-` (Meta/Alt), `S-` (Shift), `s-` (Super/Cmd).
- Named keys: `Left Right Up Down Home End Enter Tab Backspace Delete Space Esc`.
- Bare/shifted printable chars self-insert (`a`, `Z`, `<`, `>`).
- `C-x` two-chord prefixes compose: `"C-x C-s"` → save.
- `S-` on a motion chord is select-intent, exactly like a live held Shift:
  `--keys "S-Right S-Right"` leaves a two-char selection (sidecar `selection`),
  and the next unshifted motion collapses it. `C-Space` still sets the sticky
  emacs mark. Shift-PageDown/PageUp deliberately do not extend (documented
  non-movers), and `M-<`/`M->`'s Shift stays incidental (pure motion).

Because replay drives the same keymap + `apply_transition` seam as live editing, a
capture exercises the real edit logic — not a parallel mock. The visual-line
LAYOUT ORACLE (the offscreen-shaped pipeline wrapped motion consults) is
RE-SHAPED from the current buffer / zoom / page-measure state before every
action (`capture::OraclePipeline::refresh` — one seam), mirroring the live
window's between-keystrokes re-sync — so an edit that re-wraps a line, a
replayed zoom change, or a Goto buffer switch (and the sticky-measure re-apply
riding it) never leaves a later motion on stale wrap geometry.

**Caveats — know these before trusting a replay:**

- **Ordinary replay never saves.** Save and Finish become typed persistence
  requests, recorded as skipped in permissive replay because the ordinary
  headless interpreter owns no filesystem-write capability. Opening an absent
  config also leaves it absent. Under `--strict-replay`, the caller explicitly
  grants an isolated in-memory filesystem capability, so Save lands only in
  that sandbox and the real file keeps every byte.
- **Unbound chords are silent no-ops** (e.g. `C-Q` → `Ignore`, dropped); only
  structurally invalid tokens (e.g. `frobnicate`) error. (Under
  `--strict-replay` an unbound chord aborts instead — see below.)
- **The theme (and caret-style) picker's LIVE PREVIEW is real, not skipped
  headlessly.** `--keys "Cmd-T C-n"` opens "Switch theme…" and moves the
  selection down — same as live, that PREVIEWS the next world onto the
  process-global active theme (`preview_overlay`), so the captured frame
  renders in the NEXT world, not whatever `--theme` you passed. This is
  expected behavior (the whole point of `--keys` is driving the real seam),
  not a bug — but a scenario author who wants a SPECIFIC world should pin it
  with `--theme` and avoid keying through the theme picker at all, rather
  than trying to land on a world by counting `C-n`/`C-p` presses.

### Search/replace is fully drivable (the isearch-input gap is retired)

With isearch active, the live window routes EVERY key to the search surface
before the keymap ever sees it. The replay loop now runs the SAME code — one
shared interception seam, `search::keys::intercept`, consumed by both the live
guard (`app/input/keys.rs`) and the replay guard (`main/run/chord.rs`) — so a
`--keys` spec drives the panel's whole operation set exactly like live typing:

- **Query typing + Backspace** — `--keys "C-s h i"` searches for `hi` (the
  buffer text is untouched; the caret lands on the current match).
- **Find next / previous** — `C-s`/`C-r`, `Down`/`Up`, `Cmd-F`/`Cmd-S-F`,
  `Cmd-G`/`Cmd-S-G` while the panel is open (the Emacs two-press wrap and its
  caret recoil included; the recoil itself is live-only animation).
- **Case toggle** — `M-c` (sidecar `search.case_sensitive`).
- **Replace-field editing** — `Tab` reveals/flips the field, `Cmd-R` focuses
  it, then plain chars/`Space`/`Backspace` edit the replacement (sidecar
  `search.replacement` + `search.editing_replacement`).
- **Replace-one / replace-all** — `Enter` (replace mode) swaps the current
  match and advances; `s-Enter` (Cmd-Enter) swaps every match in one edit.
- **Accept / abort** — `Enter` (plain find) closes leaving the cursor on the
  match; `Esc`/`C-g` closes AND restores the origin cursor, exactly like live
  (the old headless `Cancel` close that skipped the origin-restore is gone).

Worked example (fold visible in the sidecar `search` block + `text`):

```sh
cargo run -- --screenshot OUT.png --keys "C-s l i n e Tab r o w Enter" notes.md
# OUT.json: search.query "line", replace_active true, replacement "row",
# editing_replacement true, and text shows ONE "line" already swapped to "row".
```

To make this possible, `--keys` parsing now stops at the CHORD level
(`keyspec::parse_chords`); each chord resolves through the real keymap INSIDE
the replay loop (`keyspec::ChordResolver`), after the search guard has had its
chance to consume it — the exact ordering of live key dispatch. A consumed
chord is never judged "unbound" (`M-c` is a case toggle inside the panel, a
silent no-op outside it).

### Strict replay (`--strict-replay`, opt-in)

`--screenshot --keys "SPEC" --strict-replay` turns the permissive replay into a
truthfulness gate (the scenario-runner default the harness phases build on —
`src/replay.rs` is the one owner). Every `actions::Effect` is classified
**Applied** (performed for real headlessly), **Intercepted** (an external
handoff — URL open, mailto, Trash, download — observed and recorded, payload
included, deliberately not performed), or **Unsupported** (live-App-only work
whose skip would diverge the session from live). The classification is a
no-wildcard match, so a future `Effect` variant fails to compile until
classified. Strict mode aborts, naming the exact offender, on: an unbound
chord or dangling prefix sequence (at replay time, via
`keyspec::ChordResolver` — it moved out of parse time when the search guard
made "was this chord for the keymap at all" replay-state-dependent; a chord
the open search panel consumes is never judged unbound), any Unsupported
effect, or a missing layout oracle (no GPU adapter — motion would
silently fall back to logical lines). A spec that crosses no such seam renders
byte-identically to the permissive run. The plain `--keys` path stays
permissive but now WARNS on stderr when it crosses an Unsupported or
Intercepted seam; intercepted handoffs are recorded in the replay result under
both modes (the future trace file's seam).

**Hermetic by default (the scenario filesystem).** A strict run swaps the
process filesystem seam (`fs::active()`) to an in-memory SANDBOX before its
config even loads — `src/scenario.rs` is the one owner, with exactly one
production door (`args::parse_args`'s strict arm). The sandbox is seeded from
exactly the inputs the command line names: the launch file's bytes and an
explicitly-passed config (`--config` / `$AWL_CONFIG`). Everything downstream
routes through it — the config load, the buffer open, the `.git` probe (so the
read-only `git` subprocesses never spawn), the index walk, a replayed save, a
History read, a Settings open. A strict scenario therefore NEVER reads the
user's implicit `~/.config/awl/config.toml` (an un-seeded path degrades to
pure defaults) and NEVER writes any real file besides the PNG + JSON it was
asked to produce: a replayed `Cmd-S` lands in the sandbox and the real file
keeps every byte. External handoffs stay observed-not-performed (Intercepted,
above), so a scenario run's only real side effects are its own artifacts.
`tests/hermetic_canary.rs` proves the whole contract on the real binary: a
save-bearing strict scenario under a canary HOME/XDG leaves the entire canary
tree byte-identical, while the legacy leg (no `--strict-replay`) still reads
the user's config off the real disk — hermeticity is the SCENARIO default,
never a regression of the one-off permissive harness. Storyboards (the later
phase) seed MORE files — fixtures, config, history — through the same
`scenario::build_sandbox` door, not a new seam.
