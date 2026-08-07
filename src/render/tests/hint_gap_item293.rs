//! ITEM 293 — THE FOOT HINT NO LONGER SITS FLUSH AGAINST THE LAST CANDIDATE
//! ROW (OR THE EMPTY-STATE NOTICE, WHICH SHARES ITS BAND).
//!
//! `OverlayGeom`'s own `footer_rows` doc names the pattern this defect broke:
//! the KEYBINDINGS footer band already reserved a blank row before its own
//! content (`footer.len() + 1`, `overlay_footer_lines`), but the hint's own
//! row (`hint_rows`) budgeted only the hint's own line — nothing separated it
//! from the row above. **This was a NOT-COMPUTED defect, not a not-drawn or a
//! clipped one**: `OverlayGeom::hint_rows` was always `0` or `1` — the hint's
//! own (shrunk) row — and no owner anywhere added a second row for the gap, so
//! the shaper (`push_overlay_hint_spans`) had only one `"\n"` to work with and
//! nothing to reserve past it even if it had used two. Established by reading
//! every producer of `hint_rows` (`overlay.rs`, `theme_picker.rs`,
//! `workspace.rs`) and the one shaper that draws it
//! (`overlay_shape.rs::push_overlay_hint_spans`) — none of the three families
//! ever budgeted a row for the gap, and the shaper emitted exactly one
//! newline, so there was nothing to clip: the space to hold a gap never
//! existed in the first place.
//!
//! `overlay_hint_gap_rows` (`chrome/mod.rs`) is now the ONE owner every
//! geometry family reserves the gap row through, and `push_overlay_hint_spans`
//! draws it with a second `"\n"`, sized to its own compact
//! `overlay_hint_gap_h` (deliberately smaller than a full row — see
//! `OVERLAY_HINT_GAP_ROW`'s doc — so the footer reads as one composed unit
//! rather than a wide gap floating over a pinned chin,
//! `overlay_rhythm_item112.rs`'s own law). This file is the device-level law
//! that the two agree: the gap the shaper DRAWS is a real, measurable
//! separation — never the near-zero gap the bug produced — over the whole
//! `OverlayKind` roster, every list style, both DPIs, and four candidate-list
//! shapes (full, filtered, scrolled, empty) — the empty-state notice shares
//! this band and has collided with the footer before (item 174).
//!
//! The oracle is read back out of the shaped `panel_buffer`
//! (`overlay_hint_gap_probe`, `overlay_probe.rs`), never re-derived from row
//! counts: `content_bottom` is `OverlayRowPlan::footer_top()`, the same y the
//! Bars footer plate and the footer-width probe already treat as "where the
//! content band ends", and the hint's own top/bottom come from the shaped
//! line whose text is the hint string verbatim. A law built from the same
//! arithmetic as the fix could stay green through a shared bug; this one
//! can't, because neither half of it re-derives `hint_rows`.

use super::super::*;
use super::{headless_dqp, headless_pipeline, view};
use crate::overlay::OverlayKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    Full,
    Filtered,
    Scrolled,
    Empty,
}

fn overlay_view(kind: OverlayKind, shape: Shape) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_hint = kind.hint();
    v.overlay_query = "co".into();
    v.overlay_query_caret = 2;
    let n = match shape {
        Shape::Full => 3,
        Shape::Filtered => 1,
        Shape::Scrolled => 40,
        Shape::Empty => 0,
    };
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = match shape {
        Shape::Scrolled => n - 1,
        _ => 0,
    };
    v.overlay_empty = (shape == Shape::Empty).then(|| kind.empty_corpus_message().to_string());
    if crate::facets::scheme(kind).is_some() {
        v.overlay_lens = vec![("All".into(), true), ("File".into(), false)];
        // CONTIGUOUS blocks, not alternating: a grouped card plans one header
        // per section RUN (`theme_plan`), so alternating sections would insert
        // a header before nearly every item and starve the window's own
        // budget on an unrelated axis (`total_headers`, not the hint).
        v.overlay_sections = (0..n)
            .map(|i| match i * 3 / n.max(1) {
                0 => "Alpha".to_string(),
                1 => "Beta".to_string(),
                _ => "Gamma".to_string(),
            })
            .collect();
    }
    v
}

/// Grade one prepared frame's hint gap: the drawn gap is close to a full row
/// (never near-zero, the pre-fix shape), and both halves of the claim are
/// genuinely present (a "gap exists" reading must not be satisfiable by
/// either side going missing — the floor-satisfied-by-deleting-its-subject
/// trap CLAUDE.md names).
fn grade_hint_gap(p: &TextPipeline, width: u32, lh: f32, ctx: &str, graded: &mut usize) {
    let Some((content_bottom, hint_top, hint_bottom)) = p.overlay_hint_gap_probe(width) else {
        panic!("{ctx}: this fixture always sets a non-empty hint, but none was drawn");
    };
    // NOT COMPUTED, the second way a gap can go missing even once the shaper
    // draws it: the CARD must actually have grown to hold the extra row, or
    // the hint's own (correctly gapped) line is drawn past the card's own
    // bottom edge — clipped by whatever the draw pass scissors to, or simply
    // bleeding past the card's border. Geometry and the shaped buffer are two
    // independent owners (`overlay_hint_gap_rows` vs `push_overlay_hint_spans`)
    // and a law reading only one of them can stay green while the other lags.
    let geom = p.overlay_geometry(width);
    let [_cx, card_y, _cw, card_h] = geom.card_probe();
    assert!(
        hint_bottom <= card_y + card_h + 0.75,
        "{ctx}: the hint's drawn line [{hint_top}, {hint_bottom}] runs past \
         the card's own bottom edge ({}) — the shaper drew the gap but the \
         geometry never grew the card to hold it",
        card_y + card_h
    );
    // PRESENCE FLOOR — both sides of the gap are real, non-collapsed content.
    assert!(
        hint_bottom > hint_top + 1.0,
        "{ctx}: the hint's own drawn line has near-zero height ({} .. {}) — a \
         gap here would be satisfied by the hint disappearing, not by it \
         moving down",
        hint_top,
        hint_bottom
    );
    assert!(
        content_bottom > 0.0,
        "{ctx}: the content band's own bottom ({content_bottom}) reads as \
         collapsed"
    );
    // THE CLAIM — a real, DELIBERATELY compact row separates them (item 293's
    // own `overlay_hint_gap_h`, smaller than a full row by design — see
    // `OVERLAY_HINT_GAP_ROW`'s doc), not the near-zero gap the bug produced
    // (single `\n`, i.e. ordinary line spacing only).
    let gap = hint_top - content_bottom;
    let gap_h = p.overlay_hint_gap_h();
    assert!(
        gap >= gap_h * 0.6,
        "{ctx}: the hint sits only {gap}px below the content band (the \
         separator's own row is {gap_h}px, row pitch {lh}) — the blank \
         separator row is missing or too small; the pre-fix shape put this \
         at ~0"
    );
    assert!(
        gap <= lh,
        "{ctx}: the hint sits {gap}px below the content band (row pitch \
         {lh}) — more than a full reserved row, so something else grew"
    );
    *graded += 1;
}

/// THE HEADLINE LAW. Over the whole `OverlayKind` roster (every family the
/// flat/grouped shapers reach — `Spell` is excluded, it structurally never
/// draws a hint), both list styles, both DPIs, and full/empty candidate
/// shapes (the empty-state notice shares this band — item 174's own
/// collision).
#[test]
fn the_hint_sits_a_full_row_below_the_content_band() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_hint_sits_a_full_row_below_the_content_band: no wgpu adapter");
        return;
    };
    let styles: [(&str, Option<theme::ListStyle>); 2] = [
        ("pane", Some(theme::ListStyle::Pane)),
        ("bars", Some(theme::ListStyle::Bars)),
    ];
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    let (lw, lh_win) = (1200u32, 800u32);
    let mut graded = 0usize;
    let mut empty_graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((lw as f32 * dpi) as u32, (lh_win as f32 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for (sname, style) in styles {
            crate::render::set_list_style_test_override(style);
            for kind in OverlayKind::ALL {
                if kind == OverlayKind::Spell {
                    continue;
                }
                for shape in [Shape::Full, Shape::Empty] {
                    let v = overlay_view(kind, shape);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let lh = p.overlay_lh();
                    let ctx = format!("{kind:?} dpi={dpi} list={sname} shape={shape:?}");
                    grade_hint_gap(&p, cw, lh, &ctx, &mut graded);
                    if shape == Shape::Empty {
                        empty_graded += 1;
                    }
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded > 100, "the sweep must actually run, got {graded}");
    assert!(
        empty_graded > 20,
        "the empty-state notice cell must be reached — item 174's own \
         collision — got {empty_graded}"
    );
}

/// FILTERED and SCROLLED shapes, plus the WORKSPACE family (Settings' rail
/// and timeline stages), swept over the world roster — a targeted companion
/// to the headline sweep rather than a repeat of its full breadth.
#[test]
fn the_hint_gap_holds_when_filtered_scrolled_or_in_a_workspace() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_hint_gap_holds_when_filtered_scrolled_or_in_a_workspace: no wgpu adapter"
        );
        return;
    };
    let mut graded = 0usize;
    let mut scrolled_graded = 0usize;
    let mut workspace_graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for world in crate::theme::world_names() {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for kind in [
                OverlayKind::Command,
                OverlayKind::Context,
                OverlayKind::Theme,
            ] {
                for shape in [Shape::Filtered, Shape::Scrolled] {
                    let v = overlay_view(kind, shape);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let lh = p.overlay_lh();
                    let ctx = format!("{world}/{kind:?} dpi={dpi} shape={shape:?}");
                    grade_hint_gap(&p, cw, lh, &ctx, &mut graded);
                    if shape == Shape::Scrolled {
                        scrolled_graded += 1;
                    }
                }
            }
            // THE WORKSPACE FAMILY — Settings' two stages (`rows_primary`
            // false/true is the rail-over-rows / timeline-over-comparison
            // split `the_workspace_header_band_...` law already exercises).
            for rows_primary in [false, true] {
                let mut v = overlay_view(OverlayKind::Settings, Shape::Scrolled);
                v.overlay_workspace = true;
                v.overlay_rows_primary = rows_primary;
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let lh = p.overlay_lh();
                let ctx = format!("{world}/workspace rows_primary={rows_primary} dpi={dpi}");
                grade_hint_gap(&p, cw, lh, &ctx, &mut graded);
                workspace_graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded > 60, "the sweep must actually run, got {graded}");
    assert!(
        scrolled_graded > 20,
        "the scrolled shape must be reached, got {scrolled_graded}"
    );
    assert!(
        workspace_graded > 20,
        "the workspace family must be reached, got {workspace_graded}"
    );
}

/// THE ROW-COUNT PROOF, independent of the shaped glyphs. A hint reserves
/// exactly TWO rows out of an overfull candidate window — its own (shrunk)
/// line, plus the blank separator ahead of it — over the FLAT, GROUPED and
/// WORKSPACE families alike. This is the one law in the file that reads
/// `overlay_hint_gap_rows`'s effect directly (`plan.candidate_rows()`, the
/// windowed row count every family's `chrome_rows` bounds) rather than
/// through the shaped buffer: the WORKSPACE family's card is CANVAS-sized
/// (`regions.card`, never content-derived — `workspace.rs`'s own doc), so on
/// a generously large canvas the drawn-gap law above cannot see a missing
/// reservation there — the extra unbudgeted row simply still fits inside a
/// card that was never going to hug it. Proven against exactly that gap by
/// mutation (`hint_gap_rows = 0` in `workspace.rs` passes the drawn-gap law
/// above but fails this one).
#[test]
fn a_hint_reserves_exactly_two_rows_from_the_candidate_window() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_hint_reserves_exactly_two_rows_from_the_candidate_window: no wgpu adapter"
        );
        return;
    };
    // A SHORT canvas, deliberately: `OverlayKind::window_rows()` is a per-kind
    // FIXED cap (12 for both kinds below), and on the default 800px-tall
    // canvas that fixed cap — not the pixel budget the hint's rows come out
    // of — is what binds, so the hint's cost would not show up in the visible
    // count at all. Shrinking the canvas makes the pixel budget the binding
    // constraint, which is the constraint `overlay_hint_gap_rows` actually
    // feeds.
    p.set_size(1200.0, 350.0);

    let visible_rows = |p: &mut TextPipeline, v: &ViewState| -> usize {
        p.set_view(v);
        p.overlay_window_report().expect("overlay open").1
    };

    // FLAT — `Caret` carries no facet scheme (`facets::scheme`), so
    // `overlay_view` leaves `overlay_lens` empty and the dispatcher takes the
    // flat path.
    let mut flat = overlay_view(OverlayKind::Caret, Shape::Scrolled);
    flat.overlay_hint = String::new();
    let flat_bare = visible_rows(&mut p, &flat);
    flat.overlay_hint = OverlayKind::Caret.hint();
    let flat_hinted = visible_rows(&mut p, &flat);
    assert_eq!(
        flat_bare.saturating_sub(flat_hinted),
        2,
        "flat: a hint must cost exactly 2 rows of the candidate window \
         (bare={flat_bare}, hinted={flat_hinted})"
    );

    // GROUPED — `Command` carries a real facet scheme, so `overlay_view` sets
    // `overlay_lens` and the dispatcher takes the grouped (faceted) path —
    // the same card the very first screenshot of this item showed crowded.
    let mut grouped = overlay_view(OverlayKind::Command, Shape::Scrolled);
    grouped.overlay_hint = String::new();
    let grouped_bare = visible_rows(&mut p, &grouped);
    grouped.overlay_hint = OverlayKind::Command.hint();
    let grouped_hinted = visible_rows(&mut p, &grouped);
    assert_eq!(
        grouped_bare.saturating_sub(grouped_hinted),
        2,
        "grouped: a hint must cost exactly 2 rows of the candidate window \
         (bare={grouped_bare}, hinted={grouped_hinted})"
    );

    // WORKSPACE — Settings' rail-over-rows stage, canvas-sized card.
    let mut ws = overlay_view(OverlayKind::Settings, Shape::Scrolled);
    ws.overlay_workspace = true;
    ws.overlay_rows_primary = false;
    ws.overlay_hint = String::new();
    let ws_bare = visible_rows(&mut p, &ws);
    ws.overlay_hint = OverlayKind::Settings.hint();
    let ws_hinted = visible_rows(&mut p, &ws);
    assert_eq!(
        ws_bare.saturating_sub(ws_hinted),
        2,
        "workspace: a hint must cost exactly 2 rows of the candidate window \
         (bare={ws_bare}, hinted={ws_hinted})"
    );
}
