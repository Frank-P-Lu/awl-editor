//! ITEM 126 — a proportional caret remains a visible point of presence on
//! punctuation.  This probes the real shaped/raster geometry across the product
//! roster; the spawned PNG law in `tests/caret_punctuation_pixels.rs` checks the
//! resulting pixels.

use super::super::caret_body::{
    CARET_VISUAL_BODY_MIN_AREA, CARET_VISUAL_BODY_MIN_H, CARET_VISUAL_BODY_MIN_W,
};
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

/// ITEM 200 — the structural half of MONO IMMUNITY. `prepare_morph_body_or_empty`
/// (`caret_body.rs`)'s `needs_body` is `self.caret_anchor_ink_box().map(...).unwrap_or(false)`
/// — on a mono face `caret_anchor_ink_box` is unconditionally gated to `None`
/// (item 91's mono/ligature/glyphless policy gate), so `needs_body` is
/// STRUCTURALLY always `false` there: the whole "draw a support body, knock
/// the covered glyph back" branch this item's bug lived in can never even be
/// entered on a mono world. That is a fact about the GATE, provable directly
/// at the unit seam with no GPU pixel read needed — and more reliable than
/// one, since a real glyph's raster shape (this test tried, in the sibling
/// pixel-level law `tests/caret_punctuation_color_item200.rs`) varies enough
/// per punctuation mark that geometry/colour heuristics built to catch a
/// WRONG colour cannot also cleanly prove an ABSENT code path — this test is
/// that proof instead, swept over the full mono subset and the full
/// punctuation roster, both caret forms, no wildcard.
#[test]
fn mono_worlds_never_read_a_punctuation_ink_box() {
    let _serial = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let _caret = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping mono_worlds_never_read_a_punctuation_ink_box: no wgpu adapter");
        return;
    };

    let mut mono = 0usize;
    for world in THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        if !crate::caret::font_is_mono(p.shaped_font) {
            continue;
        }
        mono += 1;
        for mode in [CaretMode::Block, CaretMode::Morph] {
            crate::caret::set_mode(mode);
            for ch in PUNCTUATION {
                let col = if mode == CaretMode::Morph { 2 } else { 1 };
                let text = format!("a{ch}z");
                let v = view(&text, 0, col);
                p.set_view(&v);
                p.settle_caret();
                assert!(
                    p.caret_anchor_ink_box().is_none(),
                    "{} {mode:?} {ch:?}: a mono world must never read a punctuation ink box — \
                     if this fails, `needs_body` can trigger on mono and this item's mono \
                     immunity claim no longer holds",
                    world.name
                );
            }
        }
    }
    assert!(mono >= 5, "expected the full mono subset, got {mono}");

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}
