//! Continuous-scroll laws: packetisation must not change semantic geometry.

use super::super::*;
use super::{H, headless_pipeline, view};

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
fn caret_row_box_reveal_is_minimal_when_already_visible() {
    let _serial = crate::testlock::serial();
    let mut p = headless_pipeline().expect("semantic-scroll laws require a GPU adapter");
    p.set_view(&view(&"ordinary row\n".repeat(200), 10, 0));
    let scroll = ScrollPos { row: 5, px_q: 13 };
    assert_eq!(p.scroll_to_show_row_pos(10, scroll, H), scroll);
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
        include_str!("../../app/files/active.rs"),
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
}
