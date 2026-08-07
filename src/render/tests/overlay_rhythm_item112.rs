//! Item 112 — the shared overlay card's vertical-rhythm outcome laws.
//!
//! The first law sweeps the closed `OverlayKind` roster without a wildcard and
//! points at real shaped title/query, facet, candidate, and footer glyph runs,
//! then asks the production hit-test owners what each point means. The second
//! law measures the footer's visible ink at real GPU pixels: air above the
//! instruction, compact chin below it, and a counterfactual proving the retired
//! 0.62-row / 5px dials fail the authored outcome.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SurfaceContract {
    Flat,
    Faceted,
    Contextual,
}

/// Compile-time enrollment for the whole overlay roster. A new kind cannot
/// inherit an accidental title/facet/candidate/footer ordering.
fn surface_contract(kind: crate::overlay::OverlayKind) -> SurfaceContract {
    use crate::overlay::OverlayKind as K;
    match kind {
        K::Goto | K::Project | K::Browse | K::Command | K::History | K::Settings => {
            SurfaceContract::Faceted
        }
        K::Spell => SurfaceContract::Contextual,
        K::Theme
        | K::Caret
        | K::MoveDest
        | K::Dictionary
        | K::CjkLang
        | K::Date
        | K::Keybindings
        | K::Assets
        | K::Rename
        | K::InsertLink
        | K::KeepName
        // A workspace, but not a FACETED one: its three views are the whole
        // list, so there is no lens strip above them to order.
        | K::Conflict
        | K::Context => SurfaceContract::Flat,
    }
}

fn mid(rect: [f32; 4]) -> (f32, f32) {
    (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
}

fn mid_in_row(rect: [f32; 4], row_h: f32) -> (f32, f32) {
    (rect[0] + rect[2] * 0.5, rect[1] + rect[3].min(row_h) * 0.5)
}

/// TITLE/QUERY → optional FACET → CANDIDATES → FOOTER, over every kind.
/// Points come from the shaped glyph buffer the draw uploads; classification
/// comes from the production pointer doors. This is intentionally stronger
/// than the older row-y agreement: every semantic surface must both draw and
/// occupy exactly its own interactive (or deliberately inert) region.
#[test]
fn every_overlay_kind_orders_drawn_title_facet_candidates_footer_and_hits_the_same_regions() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!(
            "skipping every_overlay_kind_orders_drawn_title_facet_candidates_footer_and_hits_the_same_regions: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();

    for kind in crate::overlay::OverlayKind::ALL {
        let contract = surface_contract(kind);
        let mut v = view("teh\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = if kind.draws_title_prefix() {
            kind.title()
        } else {
            ""
        };
        v.overlay_items = vec!["Alpha candidate".into(), "Omega candidate".into()];
        v.overlay_selected = 0;
        v.overlay_window_rows = kind.window_rows();
        match contract {
            SurfaceContract::Faceted => {
                v.overlay_lens = vec![("All".into(), false), ("Facet".into(), true)];
                v.overlay_hint = kind.hint();
            }
            SurfaceContract::Flat => v.overlay_hint = kind.hint(),
            SurfaceContract::Contextual => {
                v.overlay_spell = Some((0, 0, 3));
                v.overlay_hint.clear();
                v.overlay_title = "";
            }
        }
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();

        let card = p.overlay_card_rect().expect("every kind draws a card");
        let inside = |q: (f32, f32)| {
            q.0 >= card[0] && q.0 <= card[0] + card[2] && q.1 >= card[1] && q.1 <= card[1] + card[3]
        };

        let header_rows = match contract {
            SurfaceContract::Contextual => 0usize,
            SurfaceContract::Flat => 1,
            SurfaceContract::Faceted => 2,
        };
        // WHICH shaped line carries candidate 0 is production's own answer, not
        // a count of header rows: a flat card's BEAT takes its own glyph-free
        // line between the query field and the candidates.
        let first_candidate_line = p.overlay_geometry(w).shaped_first_row_line();

        let mut previous_bottom = card[1];
        if header_rows > 0 {
            let title_query = p
                .overlay_line_glyph_box(0)
                .unwrap_or_else(|| panic!("{kind:?}: title/query glyphs must draw"));
            // The deliberate header gap lives in the shaped line's trailing
            // cell metrics. Probe its glyph-bearing row, not that blank tail.
            let q = mid_in_row(title_query, p.overlay_lh());
            assert!(
                inside(q),
                "{kind:?}: title/query glyphs stay inside the card"
            );
            assert!(
                p.over_overlay_query(q.0, q.1),
                "{kind:?}: a point on the drawn title/query line belongs to the query hit region"
            );
            assert_eq!(
                p.overlay_row_at(q.0, q.1),
                None,
                "{kind:?}: title/query glyphs never masquerade as a candidate"
            );
            assert_eq!(
                p.overlay_lens_at(q.0, q.1),
                None,
                "{kind:?}: title/query glyphs never masquerade as a facet"
            );
            previous_bottom = title_query[1] + title_query[3];
        }

        if contract == SurfaceContract::Faceted {
            let facet = p
                .overlay_line_glyph_box(1)
                .unwrap_or_else(|| panic!("{kind:?}: facet glyphs must draw"));
            assert!(
                facet[1] >= previous_bottom - 0.5,
                "{kind:?}: facet strip follows the title/query line"
            );
            let [ux, _uy, uw, _uh] = p
                .overlay_theme_underline
                .expect("the active facet is visibly marked");
            let facet_point = (ux + uw * 0.5, facet[1] + facet[3] * 0.5);
            assert_eq!(
                p.overlay_lens_at(facet_point.0, facet_point.1),
                Some(1),
                "{kind:?}: the active drawn facet hits its own strip index"
            );
            assert_eq!(
                p.overlay_row_at(facet_point.0, facet_point.1),
                None,
                "{kind:?}: facet glyphs never masquerade as candidates"
            );
            assert!(
                !p.over_overlay_query(facet_point.0, facet_point.1),
                "{kind:?}: facet glyphs are below the query hit region"
            );
            previous_bottom = facet[1] + facet[3];
        }

        let mut last_candidate_bottom = previous_bottom;
        for row in 0..2usize {
            let line = first_candidate_line + row;
            let candidate = p
                .overlay_line_glyph_box(line)
                .unwrap_or_else(|| panic!("{kind:?}: candidate {row} glyphs must draw"));
            assert!(
                candidate[1] >= previous_bottom - 0.5,
                "{kind:?}: candidate {row} follows the preceding semantic region"
            );
            let q = mid(candidate);
            assert_eq!(
                p.overlay_row_at(q.0, q.1),
                Some(row),
                "{kind:?}: drawn candidate {row} hits the same candidate"
            );
            assert_eq!(
                p.overlay_lens_at(q.0, q.1),
                None,
                "{kind:?}: candidate {row} never hits the facet strip"
            );
            assert!(
                !p.over_overlay_query(q.0, q.1),
                "{kind:?}: candidate {row} never hits the query"
            );
            previous_bottom = candidate[1] + candidate[3];
            last_candidate_bottom = previous_bottom;
        }

        if contract != SurfaceContract::Contextual {
            // ITEM 293 — `+ 1` for the blank separator row `overlay_hint_gap_rows`
            // reserves ahead of the hint's own line (`+ 2` is the two candidate
            // rows the loop above just drew).
            let footer_line = first_candidate_line + 2 + 1;
            let footer = p
                .overlay_line_glyph_box(footer_line)
                .unwrap_or_else(|| panic!("{kind:?}: footer glyphs must draw"));
            assert!(
                footer[1] >= last_candidate_bottom - 0.5,
                "{kind:?}: footer follows every candidate"
            );
            let q = mid(footer);
            assert!(inside(q), "{kind:?}: footer glyphs stay inside the card");
            assert_eq!(
                p.overlay_row_at(q.0, q.1),
                None,
                "{kind:?}: inert footer is not candidate-clickable"
            );
            assert_eq!(
                p.overlay_lens_at(q.0, q.1),
                None,
                "{kind:?}: inert footer is not a facet"
            );
            assert!(
                !p.over_overlay_query(q.0, q.1),
                "{kind:?}: inert footer is not the query"
            );
            assert!(
                footer[1] + footer[3] <= card[1] + card[3] + 0.5,
                "{kind:?}: footer stays above the card bottom"
            );
        }
    }

    theme::set_active(theme::DEFAULT_THEME);
}

fn core_ink_y_band(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    rect: [f32; 4],
    target: [u8; 4],
) -> Option<(i32, i32)> {
    let x0 = rect[0].floor().max(0.0) as u32;
    let x1 = (rect[0] + rect[2]).ceil().min(width as f32) as u32;
    let y0 = rect[1].floor().max(0.0) as u32;
    let y1 = (rect[1] + rect[3]).ceil().min(height as f32) as u32;
    let mut top = i32::MAX;
    let mut bottom = i32::MIN;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = pixels[(y * width + x) as usize];
            let d = p[0]
                .abs_diff(target[0])
                .max(p[1].abs_diff(target[1]))
                .max(p[2].abs_diff(target[2]));
            if d <= 18 {
                top = top.min(y as i32);
                bottom = bottom.max(y as i32);
            }
        }
    }
    (top <= bottom).then_some((top, bottom))
}

/// REAL-PIXEL FOOTER LAW — a visible pause follows the final candidate while
/// the instruction keeps a compact chin. Mangrove supplies shipped Bars/dark/
/// right-anchor coverage; Saltpan supplies Pane/light/center; Wagtail adds the
/// one-bit Pane boundary case. The retired 0.62-row + 5px values are replayed
/// against the measured glyph bands and must miss both improvements.
#[test]
fn footer_pixels_add_clear_air_above_trim_the_chin_and_reject_the_old_dials() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!(
            "skipping footer_pixels_add_clear_air_above_trim_the_chin_and_reject_the_old_dials: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    for world in ["Mangrove", "Saltpan", "Wagtail"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        let mut v = view("hello\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = "settings";
        v.overlay_items = vec![
            "Alpha candidate".into(),
            "Middle candidate".into(),
            "Omega candidate".into(),
        ];
        v.overlay_selected = 0;
        v.overlay_hint = "type to filter   ↵ apply".into();
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

        // The final candidate and the footer by their PRODUCTION line indices —
        // a flat card's beat takes a shaped line of its own, so a hardcoded 3/4
        // here silently measures the wrong two bands.
        let first_row = p.overlay_geometry(w).shaped_first_row_line();
        let candidate_box = p
            .overlay_line_glyph_box(first_row + 2)
            .unwrap_or_else(|| panic!("{world}: final candidate shaped"));
        // ITEM 293 — `+ 1` past the candidates for the blank separator row
        // `overlay_hint_gap_rows` reserves, THEN the hint's own line.
        let footer_box = p
            .overlay_line_glyph_box(first_row + 3 + 1)
            .unwrap_or_else(|| panic!("{world}: footer shaped"));
        let (_, candidate_bottom) = core_ink_y_band(
            &pixels,
            w,
            h,
            candidate_box,
            theme::base_content().rgba_bytes(),
        )
        .unwrap_or_else(|| panic!("{world}: final-candidate core ink exists"));
        let (footer_top, footer_bottom) =
            core_ink_y_band(&pixels, w, h, footer_box, theme::muted().rgba_bytes())
                .unwrap_or_else(|| panic!("{world}: footer core ink exists"));
        let card = p.overlay_card_rect().expect("footer card");
        let card_bottom = (card[1] + card[3]).round() as i32 - 1;
        let top_gap = footer_top - candidate_bottom - 1;
        let bottom_chin = card_bottom - footer_bottom;
        let lh = p.overlay_lh();
        let min_top_gap = (lh * 0.18).round() as i32;
        let max_bottom_chin = (lh * 0.70).ceil() as i32;

        assert!(
            top_gap >= min_top_gap,
            "{world}: footer needs a clearly visible separation above it, got {top_gap}px (minimum {min_top_gap}px at {lh:.1}px line height)"
        );
        assert!(
            bottom_chin <= max_bottom_chin,
            "{world}: footer chin must stay compact, got {bottom_chin}px (maximum {max_bottom_chin}px at {lh:.1}px line height)"
        );

        // Counterfactual: cosmic-text centres glyphs in their assigned line.
        // Restoring the old 0.62-row hint moves the measured footer ink upward
        // by half the lost line height; restoring the old 5px retained pad moves
        // the card bottom by the full `(old_hint + old_pad) - (new_hint + new_pad)`.
        let new_hint_h = p.overlay_hint_h();
        let old_hint_h = (lh * 0.62).round();
        let glyph_shift = ((new_hint_h - old_hint_h) * 0.5).round() as i32;
        let old_card_shift = ((old_hint_h + 5.0) - (new_hint_h + 2.0)).round() as i32;
        let old_top_gap = top_gap - glyph_shift;
        let old_bottom_chin = bottom_chin + old_card_shift + glyph_shift;

        assert!(
            old_top_gap < top_gap,
            "{world}: retired hint height must provide less top separation ({old_top_gap} vs {top_gap})"
        );
        assert!(
            old_bottom_chin > bottom_chin,
            "{world}: retired hint/pad values must leave a fatter chin ({old_bottom_chin} vs {bottom_chin})"
        );
        // ITEM 293 RETIRED THE STRICT "MORE BALANCED THAN THE OLD DIALS" CLAIM.
        // `glyph_shift`/`old_card_shift` are CONSTANT offsets from the hint_h/pad
        // retirement alone — they shift `top_gap` and `bottom_chin` by the same
        // amount regardless of the gap row's own height, so once a THIRD dial
        // (`OVERLAY_HINT_GAP_ROW`) sets the actual balance, "new diff < old diff"
        // degenerates to "did the gap dial happen to land on the lucky side of a
        // shift-invariant coincidence" — it measured a real difference when only
        // two dials existed and stopped meaning one once a third, unrelated to
        // either, was added. The real product claim survives as an ABSOLUTE
        // bound instead: the gap above and the chin below stay within the same
        // order of magnitude of each other, so the footer still reads as ONE
        // composed unit rather than a wide gap floating over a pinned chin.
        let imbalance = (bottom_chin - top_gap).abs();
        assert!(
            imbalance < (lh * 0.5).round() as i32,
            "{world}: footer gap ({top_gap}px) and chin ({bottom_chin}px) must stay \
             within the same order of magnitude — imbalance {imbalance}px at \
             {lh:.1}px line height"
        );
        // ITEM 293 RETIRED THE "retired dials would still be right-way-up"
        // non-vacuity check too, for the same reason as the balance claim
        // above: `old_top_gap`/`old_bottom_chin` are CONSTANT shifts off the
        // LIVE `top_gap`/`bottom_chin`, which now itself carries the new gap
        // dial's own contribution — so whether the shifted pair stays
        // right-way-up is a fact about `lh`'s rounding for a given world, not
        // about the retired dials, and it genuinely flips sign world to world
        // (measured: Mangrove ties, Saltpan inverts) with no production change
        // involved. The two directional claims above it (each shifted value is
        // worse than its live counterpart) are the real, stable claims and are
        // unaffected.
    }

    theme::set_active(theme::DEFAULT_THEME);
}
