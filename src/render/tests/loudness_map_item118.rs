//! ITEM 118 — the idle-loudness map's DRIFT ANCHOR.
//!
//! `docs/loudness-map.md` records the user's own 1-5 loudness score for
//! every world, alongside the territory/contrast arithmetic
//! `scripts/loudness-measure.py` reads off a real capture. Both are stale
//! the moment a world's ground changes underneath them — exactly the failure
//! this item was reopened to fix once already: item 258 replaced Mulga's
//! ground and the score sat unflagged for a full round because nothing
//! noticed the DATA had moved.
//!
//! This is the fix: a byte-exact snapshot of every world's
//! [`theme::Background`] (its `Debug` form, which is exhaustive over every
//! authored field — tones, dials, weave/tunnel arms) plus its
//! `has_ambient_tick()` flag, taken the moment `docs/loudness-map.md` was
//! last written. A future ground change — a retuned density, a new tone, a
//! swapped weave — makes the real roster's `Debug` string stop matching the
//! one recorded here, and this test fails BY WORLD NAME rather than staying
//! green while the doc quietly lies.
//!
//! **On a legitimate ground change:** update the snapshot below to the new
//! `Debug` string (this test's own failure message prints it — copy it
//! verbatim, do not hand-edit the format), then re-run
//! `scripts/capture-loudness-118.sh` + `scripts/loudness-measure.py` and
//! carry the new arithmetic into `docs/loudness-map.md`, and flag the
//! affected world's SCORE for the user to re-confirm — a changed ground is a
//! changed instrument, not a changed number to compute your way past.
//!
//! This file holds no taste judgement. It is pure data-identity, checked the
//! same way `personality_assignments_are_exactly_the_decided_table` checks
//! `RenderCaps` — a new world or a new field fails to compile/match until
//! this snapshot is consciously extended, so nothing can silently ride past.

use crate::theme;

/// (world name, `format!("{:?}", background)`, `has_ambient_tick()`) —
/// captured 2026-08-07 against the roster shipped at the time
/// `docs/loudness-map.md` was written (Galah's density freshly at `0.12`,
/// Mulga on its item-258 `Pinstripe` ground). Order matches `theme::THEMES`;
/// completeness is enforced below rather than assumed.
const SNAPSHOT: &[(&str, &str, bool)] = &[
    (
        "Tawny",
        "Dots { from: Srgb { r: 22, g: 24, b: 29, a: 255 }, to: Srgb { r: 32, g: 34, b: 40, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 44, g: 47, b: 55, a: 255 }, edge: false }",
        false,
    ),
    (
        "Mopoke",
        "Dots { from: Srgb { r: 27, g: 24, b: 20, a: 255 }, to: Srgb { r: 37, g: 33, b: 27, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 51, g: 45, b: 36, a: 255 }, edge: false }",
        false,
    ),
    (
        "Currawong",
        "Gradient { from: Srgb { r: 6, g: 6, b: 7, a: 255 }, to: Srgb { r: 14, g: 15, b: 17, a: 255 }, dir: (0.0, 1.0) }",
        true,
    ),
    (
        "Potoroo",
        "Stripes { from: Srgb { r: 31, g: 4, b: 0, a: 255 }, to: Srgb { r: 86, g: 40, b: 0, a: 255 }, band: Srgb { r: 107, g: 58, b: 18, a: 255 }, angle: 0.6 }",
        false,
    ),
    (
        "Gumtree",
        "Zigzag { from: Srgb { r: 228, g: 248, b: 226, a: 255 }, to: Srgb { r: 207, g: 243, b: 204, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 183, g: 239, b: 180, a: 255 }, period_px: 170.0, amplitude_px: 60.0, angle: 0.26, density: 0.4, banded: false }",
        false,
    ),
    (
        "Bilby",
        "Gradient { from: Srgb { r: 251, g: 237, b: 230, a: 255 }, to: Srgb { r: 243, g: 225, b: 214, a: 255 }, dir: (0.0, 1.0) }",
        false,
    ),
    (
        "Saltpan",
        "Pinstripe { from: Srgb { r: 251, g: 243, b: 222, a: 255 }, to: Srgb { r: 242, g: 230, b: 199, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 217, g: 199, b: 155, a: 255 } }",
        false,
    ),
    (
        "Quokka",
        "Zigzag { from: Srgb { r: 255, g: 223, b: 207, a: 255 }, to: Srgb { r: 255, g: 210, b: 189, a: 255 }, dir: (0.7, 0.7), tint: Srgb { r: 224, g: 174, b: 146, a: 255 }, period_px: 100.0, amplitude_px: 24.0, angle: 0.0, density: 0.6, banded: true }",
        false,
    ),
    (
        "Bombora",
        "Waves { tones: [Srgb { r: 21, g: 10, b: 44, a: 255 }, Srgb { r: 36, g: 21, b: 64, a: 255 }, Srgb { r: 60, g: 54, b: 84, a: 255 }] }",
        true,
    ),
    (
        "Bowerbird",
        "Organic { tones: [Srgb { r: 12, g: 20, b: 38, a: 255 }, Srgb { r: 19, g: 29, b: 51, a: 255 }, Srgb { r: 31, g: 44, b: 73, a: 255 }], scale_px: 195.0, density: 0.46 }",
        true,
    ),
    (
        "Mulga",
        "Pinstripe { from: Srgb { r: 22, g: 31, b: 15, a: 255 }, to: Srgb { r: 30, g: 41, b: 22, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 62, g: 74, b: 49, a: 255 } }",
        false,
    ),
    (
        "Mangrove",
        "Lava { ground: Srgb { r: 17, g: 39, b: 35, a: 255 }, blob_lo: Srgb { r: 23, g: 35, b: 43, a: 255 }, blob_hi: Srgb { r: 34, g: 60, b: 79, a: 255 }, dithered: true }",
        true,
    ),
    (
        "Galah",
        "Deckle { ground: Srgb { r: 248, g: 224, b: 230, a: 255 }, layer: Srgb { r: 241, g: 207, b: 217, a: 255 }, deckle: Srgb { r: 169, g: 146, b: 152, a: 255 }, weave: Fibres, period_px: 64.0, wander_px: 8.0, density: 0.12 }",
        false,
    ),
    (
        "Magpie",
        "Bands { tones: [Srgb { r: 251, g: 251, b: 250, a: 255 }, Srgb { r: 241, g: 241, b: 239, a: 255 }, Srgb { r: 228, g: 228, b: 225, a: 255 }], angle: 0.62 }",
        false,
    ),
    (
        "Brolga",
        "Gradient { from: Srgb { r: 220, g: 230, b: 248, a: 255 }, to: Srgb { r: 199, g: 215, b: 242, a: 255 }, dir: (0.0, 1.0) }",
        false,
    ),
    (
        "Wagtail",
        "Gradient { from: Srgb { r: 0, g: 0, b: 0, a: 255 }, to: Srgb { r: 0, g: 0, b: 0, a: 255 }, dir: (0.0, 1.0) }",
        false,
    ),
    (
        "Firetail",
        "Lava { ground: Srgb { r: 23, g: 9, b: 12, a: 255 }, blob_lo: Srgb { r: 36, g: 12, b: 20, a: 255 }, blob_hi: Srgb { r: 82, g: 24, b: 44, a: 255 }, dithered: false }",
        true,
    ),
    (
        "Cassowary",
        "Pinstripe { from: Srgb { r: 5, g: 5, b: 6, a: 255 }, to: Srgb { r: 11, g: 12, b: 13, a: 255 }, dir: (0.0, 1.0), tint: Srgb { r: 30, g: 74, b: 50, a: 255 } }",
        false,
    ),
    (
        "Paperbark",
        "Deckle { ground: Srgb { r: 240, g: 223, b: 186, a: 255 }, layer: Srgb { r: 216, g: 183, b: 122, a: 255 }, deckle: Srgb { r: 159, g: 105, b: 55, a: 255 }, weave: Strata, period_px: 47.0, wander_px: 6.5, density: 0.2 }",
        false,
    ),
    (
        "Kite",
        "WarpedGrid { ground: Srgb { r: 229, g: 222, b: 243, a: 255 }, minor: Srgb { r: 169, g: 162, b: 200, a: 255 }, major: Srgb { r: 70, g: 64, b: 110, a: 255 }, tunnel: Fixed, spacing_px: 30.0, density: 0.62 }",
        true,
    ),
];

#[test]
fn loudness_map_snapshot_matches_the_live_roster_or_names_what_drifted() {
    assert_eq!(
        SNAPSHOT.len(),
        theme::THEMES.len(),
        "the snapshot covers {} worlds but the live roster carries {} — a world was \
         enrolled or retired without updating this anchor (and docs/loudness-map.md)",
        SNAPSHOT.len(),
        theme::THEMES.len()
    );
    for t in theme::THEMES {
        let (_, expected_bg, expected_tick) = SNAPSHOT
            .iter()
            .find(|(name, ..)| *name == t.name)
            .unwrap_or_else(|| {
                panic!(
                    "{:?} has no entry in this snapshot — a world was enrolled without \
                     updating docs/loudness-map.md's drift anchor",
                    t.name
                )
            });
        let got_bg = format!("{:?}", t.background);
        assert_eq!(
            &got_bg, expected_bg,
            "{}'s ground has changed since docs/loudness-map.md was last written — its \
             loudness score is now STALE, exactly the way item 258 left Mulga's for a full \
             round. Re-measure with scripts/capture-loudness-118.sh + \
             scripts/loudness-measure.py, carry the new arithmetic into the doc, flag the \
             score for the user to re-confirm, and paste this new snapshot line:\n  ({:?}, {:?}, {}),",
            t.name, t.name, got_bg, t.has_ambient_tick()
        );
        assert_eq!(
            t.has_ambient_tick(),
            *expected_tick,
            "{}'s ambient-tick capability has changed since docs/loudness-map.md was last \
             written — item 118 counts ambient motion as loudness, so this world's score is \
             now stale too",
            t.name
        );
    }
}
