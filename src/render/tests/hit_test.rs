//! Live pointer hit testing must read the shaped run's real advances. A
//! fixed-width approximation is invalid for the proportional display faces awl
//! ships, even when it happens to agree on a short string.

use super::super::*;
use super::{headless_pipeline, view};

#[test]
fn proportional_hit_test_uses_shaped_advances_not_the_nominal_pitch() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::world("Mopoke").expect("Mopoke is a shipped world");
    assert!(
        !crate::render::facepitch::family_is_mono(theme::active().font),
        "Mopoke must remain a proportional-world witness"
    );
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping proportional_hit_test_uses_shaped_advances_not_the_nominal_pitch: no wgpu"
        );
        return;
    };
    p.sync_theme();
    let text = "iiimmmwww";
    p.set_view(&view(text, 0, 0));

    let xs = p.line_glyph_xs(0);
    let cells = text.chars().count();
    assert_eq!(xs.len(), cells + 1, "one shaped boundary per source column");
    let py = p.doc_top() + p.metrics.line_height * 0.5;
    let left = p.text_left();
    let mut disagreement = None;
    for col in 0..cells {
        let px = left + xs[col] + (xs[col + 1] - xs[col]) * 0.25;
        let fixed_pitch_col = ((px - left) / p.metrics.char_width).round() as usize;
        let (_, live_col) = p.hit_test_scroll(px, py, ScrollPos::default());
        assert_eq!(
            live_col, col,
            "Mopoke {text:?}: a click in shaped cell {col} must select that cell"
        );
        if fixed_pitch_col != live_col {
            disagreement = Some((col, live_col, fixed_pitch_col, xs[col + 1] - xs[col]));
            break;
        }
    }
    let Some((cell, live, fixed, advance)) = disagreement else {
        panic!(
            "Mopoke {text:?} unexpectedly agreed with fixed pitch {} across every shaped cell; \
             the proportional witness no longer proves the live seam",
            p.metrics.char_width
        );
    };
    assert_ne!(
        live, fixed,
        "Mopoke {text:?} cell {cell} advance {advance} must disagree with nominal pitch {}",
        p.metrics.char_width
    );
}
