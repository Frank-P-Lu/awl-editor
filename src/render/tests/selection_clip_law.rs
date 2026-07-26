//! ITEM 84 — THE SELECTION-QUAD CONTENT-CLIP LAW.
//!
//! Every quad that rides the SAME translucent-quad family as the document
//! selection wash — the search-match highlight (both via `range_rects`), the
//! IME preedit underline, and the caret — must stay inside
//! `TextPipeline::content_clip()`: the writing column horizontally (always),
//! narrowed to the diff-preview panel's own inset band vertically while a
//! preview is up (see `rects.rs::content_clip`'s doc). A drag that clamps its
//! hit-test to the page's own left edge, or a diff transcript scrolled so a
//! selected/composing/caret row leaves the card, both resolve through that
//! ONE owner — bounding PAINT only, never the SELECTABLE range itself (the
//! document positions above are untouched by any of this).
//!
//! NO-WILDCARD: each quad-emitting method below is called BY NAME — a future
//! emitter that skips the clip has to dodge an explicit line here, not a
//! generic loop that would silently pass it by.
//!
//! NON-VACUOUS: before this round, `range_rects` (feeding both
//! `selection_rects` and `search_match_rects`), `preedit_rects`, and the
//! caret's own gate in `prepare_caret_layer` never read the clip at all for
//! the WIDTH axis, and `range_rects`/`preedit_rects` never read it on EITHER
//! axis — a selection/search/preedit row scrolled past the diff panel's own
//! bottom edge painted straight over the margin below the card. Reverting the
//! `self.clip_rects_to_band(rects)` calls this file exercises (see
//! `rects.rs`) reproduces that: the tests below fail immediately, because the
//! very rows this scenario selects are engineered to sit past the band.

use super::super::*;
use super::{headless_dqp, headless_pipeline, view};

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
fn selection_and_search_rects_never_paint_past_the_diff_panel_band() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping selection_and_search_rects_never_paint_past_the_diff_panel_band: no wgpu adapter"
        );
        return;
    };
    // Shrink the canvas so the diff panel's content band ends MID-ROW (not on
    // a row boundary) — a genuine PARTIAL trim, not just a whole-row drop.
    p.set_size(1200.0, 716.0);

    let text = tall_doc(30);
    let mut v = view(&text, 18, 0);
    v.diff_panel = true;
    // Seven full lines: the first few sit inside the band, the rest cross or
    // fall entirely past its bottom edge.
    v.selection = Some(((18, 0), (24, 0)));
    v.search_matches = vec![((20, 0), (21, 0))];
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
fn preedit_rect_never_paints_past_the_diff_panel_band() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping preedit_rect_never_paints_past_the_diff_panel_band: no wgpu adapter");
        return;
    };
    let text = tall_doc(40);
    // Line 30's row top sits at TEXT_TOP(16) + 30*LINE_HEIGHT(32) == 976, well
    // past an ordinary 800-tall canvas's diff band — no `set_size` override
    // needed here.
    let mut v = view(&text, 30, 4);
    v.diff_panel = true;
    v.preedit = "ab".to_string();
    p.set_view(&v);

    let clip = p.content_clip();
    // NON-VACUOUS: the composing row's own top is independently past the
    // clip's bottom edge, so an UNCLIPPED preedit rect would have to escape —
    // this is not a scenario that happens to already sit in-band.
    let doc_top = p.doc_top();
    let row_top = doc_top + p.visual_rows(30)[0].line_top;
    assert!(
        row_top > clip.3,
        "precondition: line 30's row (top={row_top}) must sit past the clip bottom ({}) \
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
        "a preedit underline whose whole row sits past the diff panel band must be dropped \
         outright, not trimmed to a sliver: {rects:?}"
    );
}

/// The caret quad is SELECTION-ADJACENT geometry sharing the same "quads
/// don't clip to `TextBounds`" problem (see `prepare_caret_layer`'s own doc)
/// — reads the SAME `content_clip` owner, proven here over a real `prepare()`
/// pass (not just the geometry function) so the actual uploaded instance
/// count is what's asserted, mirroring `one_bit.rs`'s `is_drawn()` seam.
#[test]
fn caret_parks_when_its_row_scrolls_past_the_diff_panel_band() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping caret_parks_when_its_row_scrolls_past_the_diff_panel_band: no wgpu adapter"
        );
        return;
    };
    crate::caret::set_mode(crate::caret::CaretMode::Block);

    let text = tall_doc(40);
    // In-band control: the caret on line 1 draws normally.
    let mut v_in = view(&text, 1, 0);
    v_in.diff_panel = true;
    p.set_view(&v_in);
    p.settle_caret();
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        p.caret_pipeline.is_drawn(),
        "precondition: an ordinary in-band caret still draws while the diff panel is up"
    );

    // Out-of-band: line 30's row (top ~976) sits well past an 800-tall
    // canvas's content band.
    let mut v_out = view(&text, 30, 0);
    v_out.diff_panel = true;
    p.set_view(&v_out);
    p.settle_caret();
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(
        !p.caret_pipeline.is_drawn(),
        "a caret whose row scrolled past the diff panel's band must park (never paint over \
         the card's rim / the margin below it)"
    );
    assert!(
        !p.caret_glyph_pipeline.is_drawn(),
        "the morph glyph-silhouette sibling must park too — the SAME clip, not a second one"
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
