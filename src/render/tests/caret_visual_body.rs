//! ITEM 126 — a proportional caret remains a visible point of presence on
//! punctuation.  This probes the real shaped/raster geometry across the product
//! roster; the spawned PNG law in `tests/caret_punctuation_pixels.rs` checks the
//! resulting pixels.

use super::super::*;
use super::{headless_pipeline, view};
use crate::caret::CaretMode;
use crate::theme::{self, THEMES};

const PUNCTUATION: [char; 10] = [',', '.', '\'', ':', ';', '-', '(', '[', '—', '。'];

#[test]
fn proportional_punctuation_uses_the_shared_minimum_visual_body() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let _caret = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping proportional_punctuation_uses_the_shared_minimum_visual_body: no wgpu adapter"
        );
        return;
    };

    let mut proportional = 0usize;
    let mut comma_was_floored = false;
    for world in THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        if crate::caret::font_is_mono(p.shaped_font) {
            continue;
        }
        proportional += 1;
        for (dpi, zoom) in [(1.0, 0.75), (1.0, 1.5), (2.0, 0.75), (2.0, 1.5)] {
            p.set_dpi(dpi);
            for mode in [CaretMode::Block, CaretMode::Morph] {
                crate::caret::set_mode(mode);
                for ch in PUNCTUATION {
                    // Morph inhabits the character behind the insertion point.
                    let col = if mode == CaretMode::Morph { 2 } else { 1 };
                    let text = format!("a{ch}z");
                    let mut v = view(&text, 0, col);
                    v.zoom = zoom;
                    p.set_view(&v);
                    p.settle_caret();
                    let ink = p.caret_anchor_ink_box().unwrap_or_else(|| {
                        panic!(
                            "{} {mode:?} {ch:?}: real proportional punctuation ink",
                            world.name
                        )
                    });
                    let px = p.metrics.caret_h / CARET_H;
                    let (w, h) = super::super::caret::caret_visual_body_dims(ink, px);
                    assert!(
                        w >= CARET_VISUAL_BODY_MIN_W * px - 1e-3,
                        "{} {mode:?} {ch:?}: width floor",
                        world.name
                    );
                    assert!(
                        h >= CARET_VISUAL_BODY_MIN_H * px - 1e-3,
                        "{} {mode:?} {ch:?}: height floor",
                        world.name
                    );
                    assert!(
                        w * h >= CARET_VISUAL_BODY_MIN_AREA * px * px - 1e-2,
                        "{} {mode:?} {ch:?}: area floor",
                        world.name
                    );
                    if world.name == "Mopoke" && ch == ',' && mode == CaretMode::Block {
                        comma_was_floored =
                            w > ink.width + 0.1 && h > ink.height + 2.0 * CARET_INK_PAD * px + 0.1;
                    }
                }
            }
        }
    }
    assert!(
        proportional >= 11,
        "expected full proportional roster, got {proportional}"
    );
    assert!(
        comma_was_floored,
        "non-vacuity: the reported what, comma must activate the floor"
    );
}
