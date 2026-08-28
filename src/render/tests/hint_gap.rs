//! THE FOOT HINT NO LONGER SITS FLUSH AGAINST THE LAST CANDIDATE
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
//! `overlay_rhythm.rs`'s own law). This file is the device-level law
//! that the two agree: the gap the shaper DRAWS is a real, measurable
//! separation — never the near-zero gap the bug produced — over the whole
//! `OverlayKind` roster, every list style, both DPIs, and four candidate-list
//! shapes (full, filtered, scrolled, empty) — the empty-state notice shares
//! this band and has collided with the footer before.
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
    v.overlay_title = kind.title().to_string();
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
    // Reproduce the product's presentation discriminator. The spell picker is
    // a word-anchored popup, not a card with a foot band; enrolment below asks
    // this state rather than naming the kind again.
    v.overlay_spell = (kind == OverlayKind::Spell).then_some((0, 0, 3));
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
    // THE CLAIM — a real, DELIBERATELY compact row separates them (the shared
    // `overlay_hint_gap_h`, smaller than a full row by design — see
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
/// flat/grouped shapers reach; the word-anchored popup is excluded by its
/// product presentation state because it structurally never draws a foot
/// hint), both list styles, both DPIs, and full/empty candidate
/// shapes (the empty-state notice shares this band — the known
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
    let mut footerless_graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((lw as f32 * dpi) as u32, (lh_win as f32 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for (sname, style) in styles {
            crate::render::set_list_style_test_override(style);
            for kind in OverlayKind::ALL {
                for shape in [Shape::Full, Shape::Empty] {
                    let v = overlay_view(kind, shape);
                    if v.overlay_spell.is_some() {
                        assert!(
                            !v.overlay_hint.is_empty(),
                            "{kind:?}: the excluded popup must carry a real hint; \
                             otherwise the exclusion is vacuous"
                        );
                        p.set_view(&v);
                        assert!(
                            p.overlay_hint_gap_probe(cw).is_none(),
                            "{kind:?}: a word-anchored popup unexpectedly enrolled \
                             in the card foot-band law"
                        );
                        continue;
                    }
                    // THE SECOND (POLICY, not structural) EXCLUSION: a kind
                    // whose product hint is the empty string — today only the
                    // pointer-anchored context menu, which draws pure rows
                    // with no teaching line at all (`OverlayKind::hint_actions`).
                    // Unlike Spell it reaches the ordinary flat geometry
                    // family; it just has nothing to reserve a gap ahead of.
                    if v.overlay_hint.is_empty() {
                        p.set_view(&v);
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        assert!(
                            p.overlay_hint_gap_probe(cw).is_none(),
                            "{kind:?}: a kind with an empty product hint drew a \
                             hint line anyway"
                        );
                        footerless_graded += 1;
                        continue;
                    }
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
    assert!(
        footerless_graded > 4,
        "the empty-product-hint exclusion (the context menu's dropped \
         teaching line) must actually be reached, got {footerless_graded}"
    );
}

/// FILTERED and SCROLLED shapes, plus the WORKSPACE family (Settings' rail
/// and timeline stages), swept over the world roster — a targeted companion
/// to the headline sweep rather than a repeat of its full breadth.
///
/// `Goto` stands in for `Context` here on purpose: this law's whole subject
/// is a REAL hint's gap surviving filtering/scrolling, and the pocket
/// palette carries no hint to have a gap ahead of (the headline sweep above
/// already proves that emptiness holds under Full/Empty; a dedicated law —
/// `context_menu_card_hugs_its_rows_with_no_hint_reserved` in
/// `frost_context.rs` — proves the same absence over the real anchored
/// geometry, world by world).
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
            for kind in [OverlayKind::Command, OverlayKind::Goto, OverlayKind::Theme] {
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
///
/// **RUN IN BOTH MENU-BAR STATES.** The drawn menu bar takes a vertical reserve
/// off the top of the card's own budget (`menubar_reserve`, folded into
/// `card_y`), so this law's whole subject — how many rows that budget buys —
/// differs between the two. `MENU_BAR_ON` starts hidden on macOS and shown
/// everywhere else, and with the bar shown the GROUPED arm below was satisfied
/// by its candidate band collapsing to ZERO: `2 - 0` is also `2`. A row-cost
/// law must never be satisfiable by having no rows to cost, so each reading
/// carries its own presence floor.
#[test]
fn a_nonempty_hint_enrols_exactly_one_separator_row() {
    assert_eq!(
        super::super::chrome::overlay_hint_gap_rows(0),
        0,
        "an absent hint must not reserve a separator"
    );
    for hint_rows in 1..=4 {
        assert_eq!(
            super::super::chrome::overlay_hint_gap_rows(hint_rows),
            hint_rows,
            "{hint_rows} hint rows must enrol the same number of separator rows"
        );
    }
}

#[test]
fn a_hint_reserves_compact_pixels_before_workspace_rows_and_two_slots_on_cards() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_hint_reserves_exactly_two_rows_from_the_candidate_window: no wgpu adapter"
        );
        return;
    };
    // The AMBIENT value, never `cfg!(target_os = ...)`: a `cfg!` here reports
    // the host that COMPILED the test, not the branch the initialiser took.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        // A SHORT canvas, deliberately: `OverlayKind::window_rows()` is a per-kind
        // FIXED cap (12 for both kinds below), and on the default 800px-tall
        // canvas that fixed cap — not the pixel budget the hint's rows come out
        // of — is what binds, so the hint's cost would not show up in the visible
        // count at all. Shrinking the canvas makes the pixel budget the binding
        // constraint, which is the constraint `overlay_hint_gap_rows` actually
        // feeds.
        p.set_size(1200.0, 460.0);

        let visible_rows = |p: &mut TextPipeline, v: &ViewState| -> usize {
            p.set_view(v);
            let n = p.overlay_window_report().expect("overlay open").1;
            // PRESENCE FLOOR, per reading: `bare - hinted == 2` is satisfied by a
            // band that collapsed to nothing as readily as by the right difference.
            assert!(
                n > 0,
                "menu_bar={bar}: the card must show candidate rows for their count to \
             mean anything — a row-cost law cannot be graded on an empty band"
            );
            n
        };

        let mut enrolled = 0usize;
        let mut excluded_popups = 0usize;
        let mut excluded_footerless = 0usize;
        for kind in OverlayKind::ALL {
            let mut v = overlay_view(kind, Shape::Scrolled);
            if v.overlay_spell.is_some() {
                excluded_popups += 1;
                continue;
            }
            // A kind whose product hint is the empty string — today only the
            // pointer-anchored context menu — has no "bare vs hinted" row
            // cost to measure: there is nothing to clear before the `bare`
            // reading, so the two readings would trivially agree at cost 0
            // rather than proving anything about the reservation.
            if v.overlay_hint.is_empty() {
                excluded_footerless += 1;
                continue;
            }
            if let Some(shape) = kind.workspace_shape() {
                v.overlay_workspace = true;
                v.overlay_rows_primary = shape.rows_are_primary();
            }
            // Section headers are a coupled row cost, not the hint's cost. Keep
            // the real grouped family but ask it on its section-free home lens.
            v.overlay_sections.clear();
            let hint = v.overlay_hint.clone();
            assert!(
                !hint.is_empty(),
                "{kind:?}: an enrolled surface must carry a real product hint"
            );
            v.overlay_hint.clear();
            let bare = visible_rows(&mut p, &v);
            v.overlay_hint = hint;
            let hinted = visible_rows(&mut p, &v);
            let cost = bare.saturating_sub(hinted);
            if kind.workspace_shape().is_some() {
                assert!(
                    (1..=2).contains(&cost),
                    "{kind:?} menu_bar={bar}: a fixed-height workspace reserves the compact \
                     separator + teaching line before candidates, which must cost one or two \
                     whole candidate pitches at this control (bare={bare}, hinted={hinted})"
                );
            } else {
                assert_eq!(
                    cost, 2,
                    "{kind:?} menu_bar={bar}: a floating card still budgets the hint and its \
                     separator as exactly two candidate slots (bare={bare}, hinted={hinted})"
                );
            }
            enrolled += 1;
        }
        assert_eq!(
            enrolled + excluded_popups + excluded_footerless,
            OverlayKind::ALL.len(),
            "every roster member must be enrolled or excluded by its presentation"
        );
        assert!(
            enrolled > 10 && excluded_popups > 0 && excluded_footerless > 0,
            "the roster sweep and both its exclusions (word-anchored popup, \
             empty-hint policy) must be non-vacuous"
        );
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}
