//! ITEM 253 — THE CURLY-QUOTE ORIENTATION ROSTER LAW. Sour Gummy shipped both
//! raised quote pairs with their outlines TRANSPOSED: `cmap[U+2018]` (opening
//! single) and `cmap[U+201C]` (opening double) pointed at the glyph that draws
//! the CLOSING "9" shape (a raised comma), and `cmap[U+2019]`/`cmap[U+201D]`
//! carried the mirrored opening "6" shape — identically across the Regular,
//! Bold and Black instances. `render/layers.rs`'s hanging blockquote pull-quote
//! mark asks for `U+201C` and was always correct; the bug was upstream, in
//! the font file, behind the right name. The fix (a `cmap`-only swap of both
//! pairs, run at build time via `fonttools`, never at load time — see
//! `assets/fonts/LICENSES.md`'s Sour Gummy provenance note) is unverifiable by
//! any test that only reads glyph NAMES, since `post`-table names never moved;
//! only geometry proves the outlines now sit behind the right codepoints.
//!
//! THE LAW below is the item's own "roster raster" turned permanent
//! ([`render::quotecheck`]'s doc comment has the geometric method): every face
//! in [`render::bundled_display_faces`] — never a hand-kept list, the SAME
//! roster `facepitch`'s laws sweep — must draw `U+2018`/`U+201C` heavy-BOTTOM
//! (the rotated opening "6") and `U+2019`/`U+201D` heavy-TOP (the raised-comma
//! closing "9"). It is GPU-free (reads font bytes + skrifa only), so it runs
//! on any box, and it is a NO-WILDCARD sweep: a new bundled face with this
//! exact upstream defect fails here by name, the day it ships, instead of
//! waiting for a screenshot to notice.
//!
//! MUTATION PROOF (recorded here, not re-run automatically — see the item's
//! report): reverting `assets/fonts/SourGummy-Regular.ttf`'s cmap to its
//! pre-fix mapping (re-running the SAME swap script a second time — the fix
//! is its own inverse) and re-running this exact test failed it by name:
//! `opening quote must be heavy-bottom: Sour Gummy U+2018 measured heavy-top`.
//! Re-applying the fix restored green. The law is not vacuous.

use crate::render::quotecheck::is_heavy_bottom;

/// The four codepoints item 253 is about, paired with which HALF of a
/// correctly mapped glyph's own bounding box must carry more ink. `true` =
/// heavy-bottom (opening quotes: the rotated "6"); `false` = heavy-top
/// (closing quotes: the raised-comma "9").
const EXPECTED_HEAVY_BOTTOM: &[(char, bool)] = &[
    ('\u{2018}', true),  // U+2018 LEFT SINGLE QUOTATION MARK (opening)
    ('\u{2019}', false), // U+2019 RIGHT SINGLE QUOTATION MARK (closing)
    ('\u{201C}', true),  // U+201C LEFT DOUBLE QUOTATION MARK (opening)
    ('\u{201D}', false), // U+201D RIGHT DOUBLE QUOTATION MARK (closing)
];

/// THE ROSTER SWEEP: every bundled display face — Latin display faces a
/// `Theme::font` can name, [`render::bundled_display_faces`] — draws all four
/// curly-quote marks with the typographically correct orientation. A face
/// missing one of the four codepoints fails here rather than being silently
/// skipped; every face in this roster is a Latin+punctuation prose face and
/// all fifteen carry all four (verified while writing this law).
#[test]
fn every_bundled_display_face_draws_curly_quotes_the_right_way_round() {
    let mut checked = 0usize;
    for (bytes, _pitch) in crate::render::bundled_display_faces() {
        let family = crate::render::facepitch::registered_family(bytes)
            .expect("every bundled display face registers a family name through fontdb");
        for &(ch, want_heavy_bottom) in EXPECTED_HEAVY_BOTTOM {
            let got = is_heavy_bottom(bytes, ch).unwrap_or_else(|| {
                panic!(
                    "{family}: no glyph for U+{:04X} — every bundled display face is expected \
                     to carry the curly-quote set",
                    ch as u32
                )
            });
            assert_eq!(
                got,
                want_heavy_bottom,
                "{family} U+{:04X}: expected heavy-{} ({}), measured heavy-{}",
                ch as u32,
                if want_heavy_bottom { "bottom" } else { "top" },
                if want_heavy_bottom {
                    "the rotated opening \"6\" shape"
                } else {
                    "the raised-comma closing \"9\" shape"
                },
                if got { "bottom" } else { "top" },
            );
            checked += 1;
        }
    }
    // Non-vacuity: the sweep actually visited every face × every codepoint,
    // not an empty iterator silently passing. 15 bundled display faces × 4
    // codepoints at the time this law was written; a roster change moves the
    // count, an empty roster would read 0 and this floor catches it.
    assert!(
        checked >= 4,
        "the roster sweep visited only {checked} (face, codepoint) pairs — \
         bundled_display_faces() looks empty"
    );
}

/// U+201A/U+201E (the LOW single/double quotes — `quotesinglbase`/
/// `quotedblbase`) sit correctly at the baseline sharing the comma's own
/// extents and were explicitly OUT OF SCOPE for the item's fix (touching them
/// would invent a second bug). This is the negative control: Sour Gummy's low
/// quotes measure heavy-TOP here too (matching their real ink concentrated
/// near the baseline, above their own descending tail) — the SAME orientation
/// as the (correct, untouched) closing marks, confirming the fix's `cmap`
/// swap never reached these two codepoints.
#[test]
fn low_quotes_are_untouched_by_the_fix() {
    let sourgummy = crate::render::FONT_THEME_FACES
        .iter()
        .map(|(bytes, _)| *bytes)
        .find(|bytes| {
            crate::render::facepitch::registered_family(bytes).as_deref() == Some("Sour Gummy")
        })
        .expect("Sour Gummy Regular is a bundled display face");
    for ch in ['\u{201A}', '\u{201E}'] {
        let got = is_heavy_bottom(sourgummy, ch);
        assert_eq!(
            got,
            Some(false),
            "U+{:04X} (a low quote, out of the item's fix scope) should still measure \
             heavy-top, matching its unmoved comma-derived shape",
            ch as u32
        );
    }
}
