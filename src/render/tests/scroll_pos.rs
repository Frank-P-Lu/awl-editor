//! Continuous-scroll laws: packetisation must not change semantic geometry.

use super::super::*;
use super::{H, headless_dqp, headless_pipeline, pixeldiff, view, view_md};

#[test]
fn scroll_pos_fixed_point_has_exact_value_semantics() {
    let _serial = crate::testlock::serial();
    let a = ScrollPos { row: 3, px_q: 17 };
    assert_eq!(a, ScrollPos { row: 3, px_q: 17 });
    assert_ne!(a, ScrollPos { row: 3, px_q: 18 });
    assert_eq!(a.px(), 17.0 / 64.0);
}

#[test]
fn pixel_packets_are_incremental_reversible_and_match_a_batch() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let mut incremental = ScrollPos::default();
    for _ in 0..10 {
        incremental = p.scroll_by_px(incremental, 3.0, H);
    }
    let batch = p.scroll_by_px(ScrollPos::default(), 30.0, H);
    assert_eq!(
        incremental, batch,
        "packetisation must not change scroll state"
    );
    let reversed = p.scroll_by_px(incremental, -30.0, H);
    assert_eq!(
        reversed,
        ScrollPos::default(),
        "gesture reversal restores geometry"
    );
}

#[test]
fn subpixel_remainder_accumulates_and_carries_across_variable_rows() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    let text = "# Tall heading\nbody\n## Another heading\nbody\n".repeat(80);
    p.set_view(&view_md(&text, 0, 0));
    assert_ne!(
        p.row_height_px(0).round(),
        p.row_height_px(1).round(),
        "fixture must exercise variable-height adjacent rows"
    );

    let mut accumulated = ScrollPos::default();
    for _ in 0..17 {
        accumulated = p.scroll_by_px(accumulated, 1.0 / ScrollPos::SUBPX as f32, H);
    }
    assert_eq!(
        accumulated,
        ScrollPos { row: 0, px_q: 17 },
        "subpixel packets remain semantic until a rendered-pixel threshold"
    );
    assert_eq!(
        p.rendered_scroll_top_px(accumulated),
        p.rendered_scroll_top_px(ScrollPos::default()),
        "17/64px must not move settled raster geometry"
    );

    let first_span_q = (p.row_top_px(1) * ScrollPos::SUBPX as f32).round() as i32;
    let before = ScrollPos {
        row: 0,
        px_q: first_span_q - 9,
    };
    let carried = p.scroll_by_px(before, 18.0 / ScrollPos::SUBPX as f32, H);
    assert_eq!(
        carried,
        ScrollPos { row: 1, px_q: 9 },
        "the remainder carries through the real first-row boundary"
    );
    assert_eq!(
        p.scroll_by_px(carried, -18.0 / ScrollPos::SUBPX as f32, H),
        before,
        "adjacent-row carry is exactly reversible"
    );
}

#[test]
fn subpixel_semantics_do_not_change_settled_pixels_or_render_hash() {
    let _serial = crate::testlock::serial();
    let (device, queue, mut p) =
        headless_dqp(1200.0, H).expect("pixel scroll law requires a GPU adapter");
    let text = "visible raster witness abcdefghijklmnopqrstuvwxyz\n".repeat(80);
    let mut v = view(&text, 4, 7);
    v.scroll = ScrollPos::default();
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, H as u32)
        .expect("prepare 0:0");
    let zero_hash = p.blur_signature(1200, H as u32);
    let zero = pixeldiff::render_frame(&mut p, &device, &queue, 1200, H as u32);

    v.scroll = ScrollPos { row: 0, px_q: 17 };
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, H as u32)
        .expect("prepare 0:17");
    let subpixel_hash = p.blur_signature(1200, H as u32);
    let subpixel = pixeldiff::render_frame(&mut p, &device, &queue, 1200, H as u32);

    assert_ne!(
        ScrollPos::default(),
        v.scroll,
        "the two semantic states must genuinely differ"
    );
    assert!(
        zero.iter().any(|px| *px != zero[0]),
        "the rendered fixture must contain real ink, not a uniform vacuous frame"
    );
    assert_eq!(
        zero_hash, subpixel_hash,
        "a remainder below the settled-pixel threshold must not invalidate raster work"
    );
    pixeldiff::assert_identical(
        &zero,
        &subpixel,
        1200,
        H as i64,
        pixeldiff::Region::canvas(1200, H as i64),
        "semantic scroll 0:0 vs 0:17",
    );
}

#[test]
fn table_and_fold_views_keep_nonzero_remainders_on_production_scroll_paths() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("surface-scroll laws require a GPU adapter");

    let table_doc = "| left | right |\n| --- | --- |\n| cell | value |\n\n".repeat(80);
    p.set_view(&view_md(&table_doc, 0, 0));
    let table_scroll = p.scroll_by_px(ScrollPos { row: 2, px_q: 17 }, 0.0, H);
    assert_eq!(
        table_scroll.px_q, 17,
        "a shaped table view retains the semantic within-row remainder"
    );
    let _ = p.hit_test_scroll(p.text_left() + 8.0, TEXT_TOP + 8.0, table_scroll);

    let folded_doc = (0..80)
        .map(|i| format!("# Section {i}\nbody {i}\n"))
        .collect::<String>();
    let levels = crate::fold::heading_levels(&folded_doc, true);
    let collapsed = [0usize, 20, 40, 60].into_iter().collect();
    let hidden = crate::fold::hidden_lines(&levels, &collapsed);
    let tails = crate::fold::fold_tails(&levels, &collapsed);
    let mut folded = view_md(&folded_doc, 0, 0);
    crate::fold::apply_to_view(&mut folded, &hidden, &tails);
    p.set_view(&folded);
    let fold_scroll =
        p.scroll_to_show_row_pos(p.visual_row_of(10, 0), ScrollPos { row: 3, px_q: 17 }, H);
    assert!(
        fold_scroll.px_q >= 0,
        "folded cursor-follow returns a canonical semantic remainder"
    );
    assert!(
        fold_scroll.px_q != 0 || fold_scroll.row != 3,
        "the folded production path must make a real scroll decision"
    );
}

#[test]
fn caret_row_box_reveal_is_minimal_when_already_visible() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 10, 0));
    let scroll = ScrollPos { row: 5, px_q: 13 };
    assert_eq!(p.scroll_to_show_row_pos(10, scroll, H), scroll);
}

#[test]
fn caret_follow_uses_the_settled_pixel_at_the_viewport_boundary() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("caret-follow law requires a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let scroll = ScrollPos { row: 0, px_q: 17 };
    assert_eq!(p.scroll_top_px(scroll), 17.0 / 64.0);
    assert_eq!(p.rendered_scroll_top_px(scroll), 0.0);
    assert_eq!(
        p.scroll_to_show_row_pos(0, scroll, H),
        scroll,
        "row 0 is flush with the actual rendered viewport top; semantic-only \
         visibility math would incorrectly reset its accumulated remainder"
    );
}

#[test]
fn huge_pixel_delta_is_bounded_to_the_document_end() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let end = p.scroll_by_px(ScrollPos::default(), 1_000_000_000.0, H);
    assert!(end.row <= p.max_scroll_rows(H));
    assert!(end.px_q >= 0);
}

#[test]
fn oversized_intra_row_input_normalizes_to_its_containing_row() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let normalized = p.scroll_by_px(ScrollPos { row: 0, px_q: 3200 }, 0.0, H);
    assert!(normalized.row > 0, "0:3200 must carry into later rows");
    assert!(normalized.px_q >= 0);
    assert!(
        normalized.px() < p.row_height_px(normalized.row),
        "offset must remain strictly inside its containing row"
    );
    assert_eq!(
        p.scroll_top_px(normalized),
        3200.0 / ScrollPos::SUBPX as f32,
        "normalization preserves the requested document coordinate"
    );
}

#[test]
fn every_pixel_packet_result_is_canonical() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let mut pos = ScrollPos::default();
    for delta in [3.0, 49.75, -7.25, 10_000.0, -312.5] {
        pos = p.scroll_by_px(pos, delta, H);
        assert!(pos.px_q >= 0, "{delta}: nonnegative offset");
        assert!(
            pos.px() < p.row_height_px(pos.row),
            "{delta}: offset lies within its containing row"
        );
    }
}

#[test]
fn production_scroll_has_one_semantic_owner_and_one_normalizer() {
    let owner_sources = [
        include_str!("../../app/document.rs"),
        include_str!("../viewstate_def.rs"),
        include_str!("../../render.rs"),
        include_str!("../pipeline_geometry.rs"),
    ];
    for source in owner_sources {
        assert!(
            !source.contains("pub scroll_lines"),
            "a production row mirror bypasses ScrollPos"
        );
    }

    let geometry = include_str!("../scroll.rs");
    assert_eq!(
        geometry.matches("fn scroll_pos_at_q(").count(),
        1,
        "fixed-point normalization has exactly one owner"
    );
    assert_eq!(
        geometry.matches("nearest_row(").count(),
        0,
        "nearest-row projection must never resolve semantic scroll"
    );
    let incremental = geometry
        .split("pub fn scroll_by_px(")
        .nth(1)
        .expect("production incremental owner")
        .split("/// Minimally reveal")
        .next()
        .expect("incremental owner body");
    assert!(
        !incremental.contains("scroll_pos_at_q"),
        "incremental wheel scroll must carry adjacent rows, not reconstruct an absolute coordinate"
    );

    let old_geometry = include_str!("../geometry.rs");
    for mirror in [
        "fn scroll_to_show_row(",
        "fn scroll_to_center_row(",
        "fn char_screen_top(",
        "fn zoom_anchor_scroll(",
    ] {
        assert!(
            !old_geometry.contains(mirror),
            "geometry.rs still contains the retired row-only mirror {mirror}"
        );
    }
}
