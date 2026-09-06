//! THE SELECTION-QUAD CONTENT-CLIP LAW, RE-AIMED FOR THE RELOCATED VIEWPORT.
//!
//! Every quad that rides the SAME translucent-quad family as the document
//! selection wash — the search-match highlight (both via `range_rects`), the
//! IME preedit underline, and the caret — must stay inside
//! `TextPipeline::content_clip()`: the writing column horizontally (always),
//! narrowed vertically to whatever region the document layer is actually
//! drawing in (see `rects.rs::content_clip`'s doc). A drag that clamps its
//! hit-test to the page's own left edge, or a comparison scrolled so a
//! selected/composing/caret row leaves the region, both resolve through that
//! ONE owner — bounding PAINT only, never the SELECTABLE range itself (the
//! document positions above are untouched by any of this).
//!
//! **WHAT THE RELOCATION CHANGED, AND WHAT IT DID NOT.** The vertical bound
//! came from the diff-as-preview panel's own card rect: the single case where
//! the document drew somewhere other than the whole canvas. That composition —
//! a card drawn AROUND the page column while the transcript rendered inside it
//! — is gone, replaced by a real relocated viewport
//! (`TextPipeline::comparison_viewport`), which is now `doc_clip_band`'s owner.
//! The LAW is untouched: the same four emitters, the same one owner, the same
//! by-name sweep. Only the scenario that narrows the band is different, and it
//! is a stronger one — the region now moves horizontally as well as
//! vertically, so the X arm of the clip is genuinely exercised for the first
//! time instead of being a no-op the page column satisfied by construction.
//!
//! NO-WILDCARD: each quad-emitting method below is called BY NAME — a future
//! emitter that skips the clip has to dodge an explicit line here, not a
//! generic loop that would silently pass it by.
//!
//! NON-VACUOUS: before the shared clip, `range_rects` (feeding both `selection_rects`
//! and `search_match_rects`), `preedit_rects`, and the caret's own gate in
//! `prepare_caret_layer` never read the clip at all for the WIDTH axis, and
//! `range_rects`/`preedit_rects` never read it on EITHER axis — a
//! selection/search/preedit row scrolled past the document region's own bottom
//! edge painted straight over everything below it. Reverting the
//! `self.clip_rects_to_band(rects)` calls this file exercises (see `rects.rs`)
//! reproduces that: the tests below fail immediately, because every fixture
//! here is DERIVED from the live band so that the rows it selects genuinely
//! straddle and pass it.

use super::super::*;
use super::{comparison_view, headless_dqp, headless_pipeline, view};

/// The first document line whose visual row begins strictly BELOW `y` — derived
/// from the pipeline's own row geometry, so a fixture can never quietly stop
/// straddling the band when the workspace's own metrics move.
fn first_row_below(p: &TextPipeline, y: f32, lines: usize) -> usize {
    let doc_top = p.doc_top();
    (0..lines)
        .find(|&l| doc_top + p.visual_rows(l)[0].line_top > y)
        .expect("the fixture must be tall enough to reach past the band")
}

/// A plain (non-markdown, non-wrapping) doc with `n` short numbered lines —
/// uniform `line_height` rows at zero scroll, so row `k`'s screen top is
/// `TEXT_TOP + k*line_height`, a predictable geometry for the clip-boundary
/// arithmetic below.
fn tall_doc(n: usize) -> String {
    (0..n).map(|i| format!("line {i}\n")).collect()
}

/// Assert every `[x,y,w,h]` rect in `rects` lies within `(x0,y0,x1,y1)` (a
/// small epsilon for float rounding) — the shared assertion body every test
/// below calls, so "what counts as escaping the clip" can't drift between
/// the selection / search / preedit cases.
fn assert_all_within(rects: &[[f32; 4]], clip: (f32, f32, f32, f32), what: &str) {
    let (x0, y0, x1, y1) = clip;
    for r in rects {
        assert!(
            r[0] >= x0 - 1e-2
                && r[0] + r[2] <= x1 + 1e-2
                && r[1] >= y0 - 1e-2
                && r[1] + r[3] <= y1 + 1e-2,
            "{what} paints outside the content clip: rect={r:?} clip={clip:?}"
        );
    }
}

#[test]
fn selection_and_search_rects_never_paint_past_the_comparison_viewport() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping selection_and_search_rects_never_paint_past_the_viewport: no adapter");
        return;
    };
    // SEARCH for a canvas whose region bottom lands MID-BAND rather than in the
    // leading between two rows — a genuine PARTIAL trim, not just a whole-row
    // drop. Derived rather than hardcoded: the region's own top moves with the
    // workspace's header beat, so a fixed height stops straddling the moment any
    // overlay metric is retuned (as when this law moved off the
    // diff panel's fixed 8px inset).
    let text = tall_doc(60);
    let mut v = comparison_view(&text, 0, 0);
    // The acceptance test is pure ROW GEOMETRY, deliberately not the output of
    // the clip this law is about: a fixture chosen by asking `selection_rects`
    // whether it trimmed anything would go quiet — not red — the day the clip
    // was reverted.
    let mut chosen = None;
    for h in (690..=790).step_by(2) {
        p.set_size(1200.0, h as f32);
        p.set_view(&v);
        let Some((_, band_bottom)) = p.doc_clip_band() else {
            continue;
        };
        let last_in = first_row_below(&p, band_bottom, 60).saturating_sub(1);
        let row_top = p.doc_top() + p.visual_rows(last_in)[0].line_top;
        let band_top = row_top + (p.metrics.line_height - p.metrics.caret_h) * 0.5;
        // The region's bottom must fall strictly INSIDE that row's own selection
        // band, so clipping it is a partial trim rather than a whole-row drop.
        if band_bottom > band_top + 1.0 && band_bottom < band_top + p.metrics.caret_h - 1.0 {
            chosen = Some((h, last_in.saturating_sub(1)));
            break;
        }
    }
    let (canvas_h, straddle) = chosen.expect(
        "no swept canvas height put the comparison region's bottom edge mid-band — the \
         partial-trim arm of this law cannot be exercised",
    );
    p.set_size(1200.0, canvas_h as f32);
    v.cursor_line = straddle;
    v.selection = Some(((straddle, 0), (straddle + 6, 0)));
    // The match sits on the FIRST selected row, which the sweep above proved is
    // still inside the region — so its own arm asserts a clipped-but-painted
    // rect rather than an empty vector.
    v.search_matches = vec![((straddle, 0), (straddle, 4))];
    p.set_view(&v);

    let clip = p.content_clip();
    let caret_h = p.metrics.caret_h;

    let sel = p.selection_rects();
    assert!(
        !sel.is_empty(),
        "precondition: some selected rows still paint"
    );
    assert_all_within(&sel, clip, "a selection rect");
    // NON-VACUOUS, part 1: not every one of the 7 selected rows survives —
    // some are dropped outright by the band.
    assert!(
        sel.len() < 7,
        "precondition: the band must actually drop some of the 7 selected rows, got {} rects: {sel:?}",
        sel.len()
    );
    // NON-VACUOUS, part 0: the clip's X arm is a REAL constraint here, not the
    // page column's by-construction no-op — the relocated region's left edge
    // sits well inside the canvas and away from the page column's own.
    assert!(
        clip.0 > 1.0 && clip.2 < 1199.0,
        "precondition: the relocated region must be genuinely inset, got x {}..{}",
        clip.0,
        clip.2
    );
    // NON-VACUOUS, part 2: at least one surviving rect is a genuine PARTIAL
    // trim (shorter than the untrimmed caret-height band), proving the clip
    // engaged mid-row rather than only ever dropping whole rows.
    assert!(
        sel.iter().any(|r| r[3] < caret_h - 1.0),
        "precondition: at least one surviving rect must be trimmed shorter than a full \
         caret-height band ({caret_h}): {sel:?}"
    );

    let matches = p.search_match_rects();
    assert!(
        !matches.is_empty(),
        "precondition: the search match still paints"
    );
    assert_all_within(&matches, clip, "a search-match rect");
}

#[test]
fn preedit_rect_never_paints_past_the_comparison_viewport() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping preedit_rect_never_paints_past_the_comparison_viewport: no wgpu adapter"
        );
        return;
    };
    let text = tall_doc(60);
    let mut v = comparison_view(&text, 0, 0);
    p.set_view(&v);
    let (_, band_bottom) = p
        .doc_clip_band()
        .expect("precondition: the comparison viewport narrows the document band");
    // A composing row whose own top is past the region's bottom edge.
    let past = first_row_below(&p, band_bottom, 60) + 1;
    v.cursor_line = past;
    v.cursor_col = 4;
    v.preedit = "ab".to_string();
    p.set_view(&v);

    let clip = p.content_clip();
    // NON-VACUOUS: the composing row's own top is independently past the
    // clip's bottom edge, so an UNCLIPPED preedit rect would have to escape —
    // this is not a scenario that happens to already sit in-band.
    let doc_top = p.doc_top();
    let row_top = doc_top + p.visual_rows(past)[0].line_top;
    assert!(
        row_top > clip.3,
        "precondition: line {past}'s row (top={row_top}) must sit past the clip bottom ({}) \
         for this test to mean anything",
        clip.3
    );

    let rects = p.preedit_rects();
    // The row is entirely past the band, so every rect the (still real,
    // still-emitted-then-clipped) geometry would have produced is dropped —
    // never partially escaping over the card's bottom rim.
    assert_all_within(&rects, clip, "a preedit underline rect");
    assert!(
        rects.is_empty(),
        "a preedit underline whose whole row sits past the comparison viewport must be \
         dropped outright, not trimmed to a sliver: {rects:?}"
    );
}

/// **THE CARET PARKS FOR A RELOCATED COMPARISON AT EVERY SCROLL POSITION.**
///
/// The caret quad is SELECTION-ADJACENT geometry sharing the same "quads don't
/// clip to `TextBounds`" problem (see `prepare_caret_layer`'s own doc), and this
/// used to assert the narrower half of that: a caret whose row had scrolled PAST
/// the comparison viewport parks on the shared `content_clip`, with an IN-BAND
/// control that still drew.
///
/// That control no longer exists. A relocated document is READ-ONLY prose and
/// draws no caret at any scroll position — the caret is awl's one accent and
/// means "you can write here", and every insertion door into a reading surface
/// is walled. So the clip's caret arm has no subject left inside a comparison,
/// and the law is re-aimed at the stronger rule that swallowed it. The clip
/// itself keeps real subjects in this file (the selection band and the preedit
/// underline above), and the static no-wildcard law below still pins the caret
/// path to the same owner.
///
/// The old in-band control becomes the PRESENCE COMPANION, one step out: the
/// SAME document at the SAME caret line with no comparison up must draw, or
/// "parked" would be equally true of a renderer that stopped drawing carets.
#[test]
fn caret_parks_when_its_row_scrolls_past_the_comparison_viewport() {
    let _g = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping caret_parks_when_its_row_scrolls_past_the_viewport: no adapter");
        return;
    };
    crate::caret::set_mode(crate::caret::CaretMode::Block);

    let text = tall_doc(60);
    // PRESENCE: the same document, the same caret line, no comparison up.
    p.set_view(&view(&text, 1, 0));
    p.settle_caret();
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        p.caret_pipeline.is_drawn(),
        "precondition: an ordinary document at this caret line must draw a caret, or \
         every parked reading below is satisfied by a renderer that draws none"
    );

    // IN-BAND, relocated: the caret's row sits inside the region and it still
    // must not draw — the reading surface refuses text, so it shows no accent.
    p.set_view(&comparison_view(&text, 1, 0));
    p.settle_caret();
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        !p.caret_pipeline.is_drawn(),
        "a relocated comparison is read-only prose: no caret, even on a row well \
         inside the region"
    );
    let (_, band_bottom) = p
        .doc_clip_band()
        .expect("precondition: the comparison viewport narrows the document band");
    let past = first_row_below(&p, band_bottom, 60) + 1;

    // Out-of-band: a row past the relocated region's own bottom edge. Parked
    // for the same reason and, before it, by the shared clip.
    p.set_view(&comparison_view(&text, past, 0));
    p.settle_caret();
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        !p.caret_pipeline.is_drawn(),
        "a caret whose row scrolled past the comparison viewport must park (never paint \
         over the workspace around it)"
    );
    assert!(
        !p.caret_glyph_pipeline.is_drawn(),
        "the morph glyph-silhouette sibling must park too — the SAME rule, not a second one"
    );

    crate::caret::set_mode(crate::caret::CaretMode::Block);
    theme::set_active(theme::DEFAULT_THEME);
}

/// STATIC "NO-WILDCARD" LAW: every quad-emitting function this file's runtime
/// tests exercise must have its OWN body call the shared clip owner —
/// enumerated BY NAME below (no glob/wildcard scan that a new emitter could
/// dodge by never being added to a list). Mirrors `theme_caps_law.rs`'s
/// grep-law shape, scoped to just the handful of named functions rather than
/// a whole-directory scan, since "is this ONE function routed through the
/// ONE owner" is a much narrower claim than "does ANY line anywhere match a
/// banned pattern".
#[test]
fn every_selection_adjacent_emitter_routes_through_the_shared_clip() {
    let rects_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render/rects.rs"),
    )
    .expect("src/render/rects.rs must exist");
    let layers_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render/layers.rs"),
    )
    .expect("src/render/layers.rs must exist");

    // `range_rects` is the ONE body `selection_rects` and `search_match_rects`
    // both funnel through — checking it once covers both by construction
    // (see `range_rects`'s own doc: "Shared by `selection_rects` and
    // `search_match_rects`").
    let checks: &[(&str, &str, &str)] = &[
        ("rects.rs", "fn range_rects(", &rects_src),
        ("rects.rs", "fn preedit_rects(", &rects_src),
        ("layers.rs", "fn prepare_caret_layer(", &layers_src),
    ];
    for &(file, sig, src) in checks {
        let start = src
            .find(sig)
            .unwrap_or_else(|| panic!("{file}: missing `{sig}`"));
        // The function body: from the signature to the next top-level `\n    pub`
        // (the next sibling method at the same one-tab indent) or EOF — good
        // enough to scope the search to just this one function's own body.
        let rest = &src[start..];
        let end = rest[sig.len()..]
            .find("\n    pub")
            .map(|i| i + sig.len())
            .unwrap_or(rest.len());
        let body = &rest[..end];
        assert!(
            body.contains("clip_rects_to_band(") || body.contains("content_clip("),
            "{file}: `{sig}` must route through the shared content clip \
             (`clip_rects_to_band`/`content_clip`) — item 84's law"
        );
    }
}
