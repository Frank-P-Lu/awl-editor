# Galleries and visual review

Read when preparing a world gallery or review dashboard.

[Guide](../../CAPTURE.md) · Paths in code examples are relative to the repo root.

## The world gallery (`scripts/capture-worlds.sh`)

A second, roster-driven sibling to `scripts/capture.sh`: instead of sweeping
`samples/*.md` once each, it sweeps every CURRENT world once each, against
one shared canonical specimen (`scripts/world-gallery-specimen.md` — a
restrained-but-real document: one H1, two H2s, short prose, italic + bold +
inline code, a short list, a section break, no placeholders, nothing spell
would flag). For every world it renders two shots:

- **the Room** — the writing view itself, caret parked past the last
  heading (so the whole heading ladder sits fully WYSIWYG-concealed) at a
  fixed WIDE canvas (1600×1000) with page mode explicitly on and a narrower
  fixed measure (66) — generous margins on both sides for the page edges
  *and* the default persistent Outline rail; an 80ch page that reads
  edge-to-edge is exactly the failure this guards against.
- **the Frame** — the same origin, then the command palette summoned
  (`Cmd-P`) — one representative summoned overlay every world composes
  identically, exercising card/list/chrome personality.

```sh
scripts/capture-worlds.sh            # builds release, captures every world
scripts/capture-worlds.sh --debug    # same, using the debug build
```

Output lands under a **replaceable** gitignored run dir (wiped and rebuilt
every invocation — `/gallery` is already in `.gitignore`):

- `gallery/worlds/room/<World>.png` + `.json` — the Room, one pair per world
- `gallery/worlds/frame/<World>.png` + `.json` — the Frame, one pair per world
- `gallery/worlds/contact-light.png` + `.json` — labeled contact sheet, the light worlds
- `gallery/worlds/contact-dark.png` + `.json` — labeled contact sheet, the dark worlds

**The roster comes from the binary, never a hand-copied shell list.** The
script's only source of world names is `awl --list-worlds` (one name per
line), which prints `theme::world_names()` — the same one-owner function
`--help`'s theme line and the unknown-`--theme` error read (`src/theme/
worlds.rs`). Inserting or retiring a world in `theme::THEMES` changes every
one of them for free; there is nothing to keep in sync by hand. The contact
sheets don't hand-classify light/dark either — each world's bucket is read
straight off its own Room sidecar's `theme.mode`, the same field the script
already verified. The contact sheets are themselves ordinary awl captures:
a small markdown document embedding the just-written Room/Frame PNGs as
inline `![World — Room|WIDTH](room/World.png)` images under a `## World`
heading, rendered by awl's own image + text pipeline — no new external image
utility, no network, no OS automation.

The script **fails loudly** (`exit 1`, naming the offender) on: an empty
roster, a duplicate name in `--list-worlds`' output, a capture that errors
for a listed world (the binary rejecting the very name it just printed), a
written sidecar whose `theme.name` doesn't match the world it was asked to
render, an unrecognized `theme.mode`, or Room page/outline/margin geometry
that isn't the generous, non-edge-to-edge shape above. The separate law that
catches a world being newly **un-enrolled** (or added, or reordered) lives in
`tests/world_gallery_roster.rs` — a Rust integration test against the real
binary that hard-codes a 20-name roster snapshot on purpose, so a
`theme::THEMES` change fails `cargo test` loudly until a human consciously
updates it (mirroring `theme::tests::worlds_eleven_dark_nine_light`'s
existing hard-coded `20`).


## The visual review dashboard (`scripts/review.sh`)

`scripts/review.sh` is the human-facing review build over the capture and icon
oracles. It does not draw an HTML approximation of awl. It builds the real
binary, runs `scripts/capture-worlds.sh`, drives a code-owned roster of
important screens through `--screenshot --keys`, regenerates the current
shipped icon sheets through the offline icon renderer (without packing or
touching committed app assets), and writes one local entry point:

```sh
scripts/review.sh
open gallery/review/index.html
```

Use `scripts/review.sh --debug` for a faster iteration build. Output is a
replaceable gitignored directory: `gallery/review/index.html` plus relative
full-resolution PNG and JSON assets. The index provides section navigation,
world/surface/light-dark filtering, captions containing the exact
theme/canvas/key replay, sidecar links, and a concise boundary note for the
live-only review surfaces. It contains no network dependency. Missing captures,
world/sidecar/state mismatches, duplicate scene or DOM ids, capture/icon roster
drift, absent icon sheets, broken targets, and external URLs fail the build
rather than leaving a plausible stale card behind. The scene manifest names
each expected nested sidecar value, so a keystroke journey that rendered the
wrong Settings facet or failed to summon its surface cannot produce a green
dashboard merely because a PNG exists.

Clicking a screenshot opens its full-resolution asset in an offline modal
lightbox. `←`/`→` walk the currently visible filtered set; `Escape`, the close
control, or the backdrop dismiss it and return focus to the invoking card.
Modified clicks retain the image link's ordinary new-tab behavior.

The icon section uses one bounded card per world. Each card shows that world's
actual `Theme::icon_cursor` assignment rendered natively at
256/128/64/44/32/24 on both light and dark Dock surfaces. The icon exporter
retains the two full-roster shipped strips for dedicated overview work, but
they are not the dashboard's primary review unit.

The world half derives its roster from `awl --list-worlds`. The smaller
important-screen roster lives in `scripts/review/scenes.tsv`: one typed row per
fixture/theme/canvas/key sequence/capture mode/state expectation, so adding or
retiring a review surface is a single manifest edit. History is represented by
its honest first-run empty state; version comparison uses the capture harness's
deterministic read-only diff seam rather than reading or writing the user's
live history store. This dashboard replaces the old one-off pre-tag image
judgement: it is useful throughout development and should be regenerated again
at release preparation.
