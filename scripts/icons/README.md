# Per-world app icons — the offline exporter

Pre-rendered macOS app icons, one per world, generated ahead of time and
committed. **No ordinary build runs any of this**: `cargo build`, `cargo test`
and the shipping app never invoke a browser and never invoke awl's own wgpu
renderer. They only ever read PNGs that this pipeline already produced.

## The lockup

`aw` plus a plain lowercase `l` in the world's display face — one inline run of
text, so the same font size and the same baseline are true by construction, and
ordinary web shaping owns the letterspacing. Behind the `l` sits a deliberately
**fake** logo cursor: a rounded box that is never awl's live Block/Morph/I-beam
caret renderer and never moves.

Three cursor presets, and only three:

| preset | shape | geometry |
|---|---|---|
| `block` | rectangular slab | grows past the glyph; small em radius |
| `pill` | capsule | grows past the glyph; radius = half the box |
| `narrow` | super-narrow pill | sits *inside* the glyph's advance; capsule |

Colors come from the world's real theme tokens and nowhere else: `base_100`
ground, `base_content` for `aw`, `primary` for the cursor, `primary_content` for
the `l`. Ambient (lava/stars) grounds flatten to `base_100` — an icon is one
still frame.

## Run it

```sh
scripts/export-icons.sh              # manifest -> pages -> PNGs -> pixel checks
scripts/export-icons.sh --check      # ... and re-render, comparing sha256s
scripts/export-icons.sh --only tiles-128   # one page, for a tuning loop
```

Three programs, each runnable alone:

| step | program | what it owns |
|---|---|---|
| 1 | `cargo run -- --icon-manifest` (`src/icon_manifest.rs`) | the palette + face facts, derived from `theme::THEMES` and the `assets/fonts/*.ttf` name tables |
| 2 | `scripts/icons/build.mjs` | HTML pages + `fonts.css` (fonts inlined as base64 `data:` URLs) |
| 3 | `scripts/icons/render.mjs` | one pinned Chromium over CDP, writing every PNG |
| 4 | `scripts/icons/verify.py` | pixel arithmetic over the result |

Requires Node 22+ (global `fetch` + `WebSocket`; there are **no npm
dependencies** and no `node_modules`) and the pinned Chromium revision already
present locally. The script never downloads a browser — point
`AWL_ICON_CHROMIUM` at a binary if yours lives elsewhere.

## The rules the code enforces

- **One owner for palette + font facts.** Nothing here restates a hex or a font
  file name. `--icon-manifest` reads them out of `theme::THEMES` (through the
  same `Srgb::hex` the capture sidecar uses) and out of the font files
  themselves via fontdb — so the export follows a retuned world automatically,
  and IBM Plex Mono is declared at the Weight **300** the file actually is
  rather than a fabricated 400 (`docs/fonts.md`).
- **No per-world branches.** A world contributes four colors and a family name.
  Optical tuning lives in `tuning.json`, keyed by FONT FAMILY, so the five
  shared faces carry exactly one tuning across both their worlds; `build.mjs`
  refuses a key that names a world, a key outside `allowed`, or a value outside
  `bounds`.
- **Bounded tuning.** Per face: radius, padding (as percentages of the glyph's
  own inline box) and weight (which bundled FILE, regular or bold). The
  wordmark's size and the squircle corner are global — a face needing its own
  size would mean the lockup is wrong.
- **No glyph coordinates.** Every number is relative to the `l`'s own advance
  and font-derived content area, which is why one rule fits a wide mono `l` and
  a tight serif `l`.
- **Zero network.** Local font files, `file://` pages, a local browser launched
  with networking disabled.
- **Determinism is a gate, not a hope.** `--check` re-renders everything into a
  scratch tree and diffs sha256s. It has already earned its keep: three tiles in
  a page's last column rasterized their corner antialiasing ±3/255 differently
  between runs, which is why `render.mjs` grows the viewport past the document
  instead of capturing beyond it.

## Output

`assets/macos/candidates/`

- `gallery/overview-{dark,light}.png` — all 54 candidates at 128px
- `gallery/sizes-<preset>-{dark,light}.png` — every world down the size roster,
  each size rendered natively rather than scaled from the master
- `gallery/dock-<preset>-{dark,light}.png` — a literal Dock row at 56 / 44 / 24
- `gallery/world-<World>.png` — one world, three presets, both surfaces
- `tiles/<World>-<preset>-<size>.png` — the raw candidates, 16…1024
- `legibility.txt` — how far down each candidate keeps its knocked-out `l`

Committed: the galleries, the 1024 masters and the legibility ladder. The
intermediate tile sizes are regenerable in about a minute and are not tracked.

## Not yet done (follow-ups)

Choosing a preset per world, packaging the canonical `Awl.icns`
(`scripts/package-macos.sh` already wires one in if `assets/macos/Awl.icns`
exists — see RELEASING.md's Icon TODO), and swapping the live Dock image on
sticky-theme restore and on a COMMITTED theme change. Linux launcher and the web
favicon are later rounds still.
