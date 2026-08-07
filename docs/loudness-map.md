# docs/loudness-map.md — the idle-loudness map (queue item 118)

Read this before re-opening item 118's territory, before proposing a world
rebalance, and before trusting a "the roster feels off" hunch — it separates
what is measured from what is scored, and it is the one place both live
together.

**"Idle loudness"** is how strongly a world asks for attention while the user
is simply writing in page mode: palette, typography, margin pattern, and
ambient motion count; summoned overlays do not. `1/5` is the quiet pole,
`3/5` is recognizable/alive but comfortable for hours, `5/5` is a
deliberately rare statement world.

## What this file is, and is not

Pixel/sidecar arithmetic proves **territory** (how much of the canvas the
ground occupies) and **contrast** (how hard the ground's marks push against
the page). It never claims the taste **score** — that is the user's call,
made freely, and a low score is not a defect to fix. This file records both
kinds of fact side by side so a future re-score is cheap, not so the
arithmetic can out-vote the human.

## The user's map — all twenty worlds, given directly (2026-08-06)

| 1/5 | 2/5 | 3/5 | 4/5 | 5/5 |
|---|---|---|---|---|
| Gumtree, Bilby, Mulga, Tawny, Mopoke, Currawong, Brolga, Wagtail | Potoroo, Saltpan, Bombora, Bowerbird, Galah, Magpie | Quokka, Paperbark | Mangrove, Cassowary | Firetail, Kite |

Distribution `8, 6, 2, 2, 2`, mean **2.20**.

**The target shape is DROPPED (user, 2026-08-06): *"i think we just drop the
target. it's fine, right now."*** The old `1, 7, 6, 4, 2` / mean 2.90 target
is retired — not amended, not restated as descriptive, not replaced. The
measured mean of 2.20 is accepted as awl's shape. **Do not re-derive a
target, and do not read the eight 1/5s as a deficit.** A world diverging from
what its measured column alone would predict is not a defect either — the
roster already carries one accepted case of exactly that (Mangrove measures
louder than Firetail on every static/motion column while scoring a step
below it; the inversion is closed on purpose, see below).

## Per-world arithmetic — Room capture, laptop arm (1600x1000, measure 70, page on)

Reproduce with `scripts/capture-loudness-118.sh` (writes
`gallery/item-118-loudness/<arm>/`, gitignored) then
`scripts/loudness-measure.py laptop`. Three other arms exist (`narrow`
1440x900, `wide` 2000x1250, `code` 2000x1250/measure 100). **Most worlds'
`g_edge`/`g_sd_lp` move by well under 1 unit across the three prose arms
(narrow/laptop/wide) — e.g. Saltpan's `g_edge` holds `0.441`-`0.444` and
Quokka's `g_sd_lp` holds `39.4`-`40.0` across all four including `code`. The
grounds whose PERIOD ties to viewport geometry move more: Kite's
`WarpedGrid` swings `g_sd_lp` 34.3 (code) to 39.4 (wide) and `g_edge`
0.058-0.072; Paperbark's `Deckle` similarly. That is real, not noise — a
wider window shows more lane repeats. The arm that matters most, separately:
`code`, because at `page_width_code = 100` a normal window's margins collapse
toward nothing, so a code-buffer capture measures something structurally
different from prose regardless of which ground a world carries. **State the
buffer/width with any figure — a loudness number without its geometry is not
a figure.**

`g_sdL` = CIE L\* standard deviation in the right margin (0-100 scale, the
headline ground-contrast number). `g_sd_lp` = the same luminance idea after a
4x4 box downsample — the ONE TO READ for perceived field contrast, since it
cancels dither/hairline structure below the eye's integration scale. `g_edge`
= fraction of horizontally adjacent margin pixel pairs crossing an 8-bit
luminance delta of 3 — busyness. Full metric definitions:
`scripts/loudness-measure.py`'s own module doc.

**The `ambient` column is `Theme::has_ambient_tick()`, not the sidecar's own
`page.ambient.style` field.** That sidecar field read `"none"` for every
world but Currawong in a plain single-frame capture — including Bombora,
Bowerbird, Mangrove, Firetail and Kite, all of which genuinely move — a real
instrument limit worth naming rather than trusting silently: a frozen t=0
frame does not surface most worlds' ambient style through that key.
`has_ambient_tick()` is the product's own dispatch predicate
(`scripts/capture-ambient-118.sh` reads the same one) and is trustworthy here.

| World | Score | Ground | Ambient tick | g_sdL | g_sd_lp | g_edge |
|---|---|---|---|---|---|---|
| Wagtail | 1 | gradient (flat) | no | 0.00 | 0.0 | 0.0000 |
| Galah | 2 | deckle (fibres) | no | 0.33 | 4.8 | 0.0067 |
| Gumtree | 1 | zigzag | no | 0.93 | 21.3 | 0.0201 |
| Bilby | 1 | gradient | no | 1.14 | 25.6 | 0.0000 |
| Currawong | 1 | gradient | yes (stars) | 1.21 | 1.9 | 0.0011 |
| Mopoke | 1 | dots | no | 1.45 | 1.8 | 0.0000 |
| Brolga | 1 | gradient | no | 1.56 | 32.5 | 0.0000 |
| Tawny | 1 | dots | no | 1.60 | 2.0 | 0.0000 |
| Bowerbird | 2 | organic | yes | 1.74 | 2.2 | 0.0000 |
| Potoroo | 2 | stripes | no | 2.10 | 5.0 | 0.0022 |
| Quokka | 3 | zigzag | no | 2.11 | 39.7 | 0.0371 |
| Magpie | 2 | bands | no | 2.12 | 48.2 | 0.0045 |
| Saltpan | 2 | pinstripe | no | 2.40 | 35.0 | 0.4444 |
| Paperbark | 3 | deckle (strata) | no | 2.51 | 45.6 | 0.1298 |
| Kite | 5 | warped-grid | yes | 2.61 | 36.6 | 0.0657 |
| Mulga | **1 (STALE — see below)** | pinstripe | no | 3.63 | 4.3 | 0.2222 |
| Firetail | 5 | lava | yes | 4.90 | 7.0 | 0.0000 |
| Cassowary | 4 | pinstripe | no | 5.68 | 4.3 | 0.2222 |
| Bombora | 2 | waves | yes | 8.06 | 15.8 | 0.0000 |
| Mangrove | 4 | lava | yes | 10.70 | 16.5 | 0.0682 |

Sorted by `g_sdL`, Galah (0.33) is the faintest *ground-bearing* world on the
whole roster (Wagtail carries no ground at all); Gumtree (0.93) is the
second-faintest.

## Mulga's score is STALE — re-measured, not re-scored

Item 258 replaced Mulga's ground: the old `Background::Starfield` (which the
`1/5` score describes) is retired entirely, and Mulga now ships
`Background::Pinstripe` — the same shader family as Saltpan and Cassowary, a
crisp light-line-on-dark rule rather than a scattered star field. **This is a
different instrument than the one the `1` was scored against; nothing here
assigns a new number.**

Measured at the laptop arm (1600x1000, measure 70) and stable across all
four arms (narrow/wide/code within ±0.02 on every column below — the
Pinstripe rule's period does not scale with margin width the way a sparse
scattered field might):

| | Mulga | nearest 1/5 neighbour | nearest 2/5+ neighbour |
|---|---|---|---|
| g_sdL | **3.63** | Tawny/Brolga ≈1.6 (2.3x lower) | Cassowary 5.68 (4/5) |
| g_edge | **0.2222** | every 1/5 world ≈0.0000-0.02 | Cassowary 0.2222 (identical) |
| g_sd_lp | **4.3** | Tawny/Mopoke ≈1.8-2.0 (2.2x lower) | Cassowary 4.3 (identical) |
| step (page/ground luminance gap) | 7.6 | — | Cassowary 5.4 |

Mulga's ground no longer measures like the rest of the `1/5` band on any of
the three contrast columns — it sits **above every current `1/5` world** on
`g_sdL` and `g_sd_lp`, and its `g_edge`/`g_sd_lp` are **statistically
identical to Cassowary's**, a world scored `4/5`. The two aren't the same
read, though: Mulga's `g_sdL` (3.63) is a third of Cassowary's (5.68) — a
gentler lightness swing on the same crisp-line structure — and Mulga's ink is
not accent-colored the way Cassowary's caret-matched ink is (item 258's own
landing note: Mulga sits at a quarter Saltpan's polarity/contrast, reading as
tone-on-tone ribbing rather than a hard ruled pinstripe). **Owed to the
user:** a fresh `1` no longer matches this arithmetic; a re-score belongs
somewhere in the `2`-`4` neighbourhood, but which number is not this file's
call. Captures: `gallery/item-118/mulga-neighbourhood/` (Mulga alongside
every `1/5` world plus Cassowary/Saltpan, all at the laptop arm).

## The six standing proposals — dispositions

1. **Galah's ground density — SHIPPED.** `0.10` → `0.12` (`src/theme/worlds.rs`).
   The user pinned the magnitude ("up it a tinnyyy bit", `0.12`-`0.16`);
   `0.12` is not an arbitrary pick inside that band — it is the smallest
   rung that reads as different in a real capture. Measured against the
   shipped `0.10` render (1600x1000, measure 70), right-margin luminance
   delta:

   | density | max Δ (8-bit luma) | % margin px crossing EDGE_DELTA=3 |
   |---|---|---|
   | 0.11 (one rung below the pinned band) | 1.9 | 0.00% |
   | **0.12 (shipped)** | **3.7** | **0.18%** |
   | 0.13 | 5.6 | 0.87% |
   | 0.14 | 7.3 | 0.99% |
   | 0.16 | 10.9 | 2.54% |

   `0.11` never clears the repo's own perceptibility floor
   (`EDGE_DELTA = 3`, `scripts/loudness-measure.py`); `0.12` is the first
   rung that does — that pair is the threshold evidence, not just a
   monotonic ramp. Captures: `gallery/item-118/galah-density/`
   (`d010`/`d011`/`d012`/`d013`/`d014`/`d016`, all Room at 1600x1000/measure
   70). Guarded by `density_bearing_worlds_show_a_material_gap_between_full_and_half_density`
   and `galah_density_lands_in_the_pinned_up_a_tinny_bit_band`
   (`src/render/tests/backgrounds_item158.rs`) — the first sweeps every
   density-bearing world (Gumtree, Quokka, Bowerbird, Paperbark, Galah,
   Kite), the second pins Galah's own band. Even at `0.12`, Galah's `g_sdL`
   (0.33) stays the faintest *ground-bearing* reading on the roster — this
   is a whisper, not a rebalance.

2. **Item 108 (Gumtree chevron visibility) — MET its Done condition, with a
   caveat worth stating plainly.** The shipped density bump (`0.20` →
   `0.40`) is guarded by
   `gumtree_zigzag_is_visibly_present_across_dashboard_geometries` (peak mark
   deviation ≥18 at two dashboard geometries) and
   `gumtree_visibility_floor_rejects_the_imperceptible_density_mutation`
   (the retired `0.20` value stays below that floor) —
   both pass today (`cargo test --bin awl -- backgrounds_item86::gumtree_`).
   **But roster-relative, the fix did not move Gumtree out of the quiet
   end:** it remains the **second-faintest ground on the entire twenty-world
   roster** by `g_sdL` (0.93 — only Galah's 0.33 is fainter, excluding
   Wagtail's null ground). Doubling density took Gumtree from imperceptible
   to a real, floor-clearing mark, not out of the quiet band. **This is the
   reason Galah's own step (#1 above) was measured from scratch rather than
   reusing Gumtree's 2x-the-density recipe** — the two grounds (Zigzag vs.
   Deckle/Fibres) respond differently, and 108's own magnitude is not a
   precedent for how big a step another world's dial needs.

3. **Firetail/Mangrove inversion — CLOSED on purpose, recorded, not
   re-litigated.** The user: *"theyre fine as is."* Mangrove measures louder
   than Firetail on every static and motion column in this file's own table
   (`g_sdL` 10.70 vs 4.90, `g_sd_lp` 16.5 vs 7.0) while scoring a step below
   it (4 vs 5) — a divergence between measurement and taste that the item's
   own standing rule says carries no obligation to reconcile.

4. **ROADMAP's "merge the tightest near-pair" call — NOT executed, put to
   the user.** The named near-pairs, re-verified against the live roster
   this round:
   - **Tawny/Mopoke** — tightest. Both `Dots{edge:false}`, both `g_edge
     0.0000` to four decimals, L\* σ within 0.15 (1.60 vs 1.45 in this
     file's table — inside the noise this instrument carries between two
     genuinely-close Dots worlds).
   - **Bilby/Brolga** — a deliberate mirror per THEMES.md, not a duplicate;
     not a merge candidate.
   - **Magpie/Saltpan — this pairing is ITSELF STALE, the same failure
     class as Mulga's score.** The board's inherited note claims "edge
     0.4444 on both to four decimals". That was true when Magpie shipped
     `Background::Pinstripe` — but item 260 (closed the same day as item
     258, commit `43bd92e9`) moved Magpie to `Background::Bands`. Today
     Magpie's `g_edge` measures **0.0045**, not 0.4444 — Saltpan alone
     carries 0.4444. The two worlds are no longer even the same ground
     family (Pinstripe vs. Bands), so this specific near-pair no longer
     exists. Flagged here rather than silently repeated; not re-derived
     into a replacement pairing, since that would be inventing new work
     this item was not asked for. **A merge is a product decision that
     removes something a user may have deliberately chosen — it is hers to
     make, not this item's to execute.**

5. **Recorded as durable data.** `docs/loudness-map.md` (this file) holds
   the score map and the arithmetic; `src/render/tests/loudness_map_item118.rs`
   holds a byte-exact snapshot of every world's `Background` data (`Debug`
   form) plus its `has_ambient_tick()` flag, asserted against the live
   roster on every test run. **A future ground change fails that test BY
   WORLD NAME** rather than leaving a score to go stale unnoticed the way
   Mulga's did for a full round — the failure message hands back the exact
   new snapshot line to paste in, and says to re-measure, update this doc,
   and flag the affected world's score for re-confirmation. Mutation-proven:
   temporarily retinting Mulga's Pinstripe `tint` field produced
   ```
   thread '...loudness_map_snapshot_matches_the_live_roster_or_names_what_drifted' panicked:
   assertion `left == right` failed: Mulga's ground has changed since docs/loudness-map.md
   was last written — its loudness score is now STALE, exactly the way item 258 left
   Mulga's for a full round. ...
   ```
   then reverted clean.

6. **Mulga — see the dedicated section above.** Re-measured, not re-scored;
   the arithmetic sits ready for the user's next look.

## What is owed to the user's eye or score right now

- **Mulga's score.** A fresh `1` no longer matches its measured ground; the
  arithmetic above (and `gallery/item-118/mulga-neighbourhood/`) is staged
  for a one-look re-score.
- **Galah's new density (`0.12`)** is shipped, not merely proposed — worth a
  live glance (`--theme Galah`) to confirm "a tinny bit" reads as intended
  rather than as a rebalance.
- **The Magpie/Saltpan near-pair note is retired as stale** (see #4) — no
  action owed beyond having flagged it; the drift anchor (#5) means this
  class of staleness gets caught automatically from here on.
