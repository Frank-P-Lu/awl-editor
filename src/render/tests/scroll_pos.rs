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
    let Some(mut p) = headless_pipeline() else {
        return;
    };
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
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    p.set_view(&view(&"ordinary row\n".repeat(200), 10, 0));
    let scroll = ScrollPos { row: 5, px_q: 13 };
    assert_eq!(p.scroll_to_show_row_pos(10, scroll, H), scroll);
}

#[test]
fn huge_pixel_delta_is_bounded_to_the_document_end() {
    let _serial = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    p.set_view(&view(&"ordinary row\n".repeat(200), 0, 0));
    let end = p.scroll_by_px(ScrollPos::default(), 1_000_000_000.0, H);
    assert!(end.row <= p.max_scroll_rows(H));
    assert!(end.px_q >= 0);
}
