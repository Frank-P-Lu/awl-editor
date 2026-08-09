//! A proportional caret remains a visible point of presence on
//! punctuation.  This probes the real shaped/raster geometry across the product
//! roster; the spawned PNG law in `tests/caret_punctuation_pixels.rs` checks the
//! resulting pixels.

use super::super::caret_body::{
    CARET_VISUAL_BODY_MIN_AREA, CARET_VISUAL_BODY_MIN_W, InkBox, caret_visual_body_dims,
};
use super::{headless_pipeline, view};
use crate::caret::CaretMode;
use crate::theme::{self, THEMES};

const PUNCTUATION: [char; 10] = [',', '.', '\'', ':', ';', '-', '(', '[', '—', '。'];

#[test]
fn proportional_punctuation_keeps_the_shared_horizontal_body_hug() {
    let _serial = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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
                    let px = p.metrics.scale;
                    let (w, _h) = super::super::caret::caret_visual_body_dims(ink, px);
                    assert!(
                        w >= CARET_VISUAL_BODY_MIN_W.px(px) - 1e-3,
                        "{} {mode:?} {ch:?}: width floor",
                        world.name
                    );
                    // Vertical sizing uses the row's measured
                    // x-height band.  This helper still owns only the
                    // horizontal ink hug/support-body floor; asserting its
                    // height here would bless the old punctuation-sized
                    // resting rectangle instead of the drawn caret.
                    if world.name == "Mopoke" && ch == ',' && mode == CaretMode::Block {
                        comma_was_floored = w > ink.width + 0.1;
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

/// The structural half of MONO IMMUNITY. `prepare_morph_body_or_empty`
/// (`caret_body.rs`)'s `needs_body` is `self.caret_anchor_ink_box().map(...).unwrap_or(false)`
/// — on a mono face `caret_anchor_ink_box` is unconditionally gated to `None`
/// (the mono/ligature/glyphless policy gate), so `needs_body` is
/// STRUCTURALLY always `false` there: the whole "draw a support body, knock
/// the covered glyph back" branch this item's bug lived in can never even be
/// entered on a mono world. That is a fact about the GATE, provable directly
/// at the unit seam with no GPU pixel read needed — and more reliable than
/// one, since a real glyph's raster shape (this test tried, in the sibling
/// pixel-level law `tests/caret_punctuation_color.rs`) varies enough
/// per punctuation mark that geometry/colour heuristics built to catch a
/// WRONG colour cannot also cleanly prove an ABSENT code path — this test is
/// that proof instead, swept over the full mono subset and the full
/// punctuation roster, both caret forms, no wildcard.
#[test]
fn mono_worlds_never_read_a_punctuation_ink_box() {
    let _serial = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
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

/// **THE AREA FLOOR SCALES AS THE SQUARE OF THE DISPLAY FACTOR — a property
/// the width-floor assertions above cannot see.** `w` is already clamped to
/// `CARET_VISUAL_BODY_MIN_W.px(px)` by `ink.width.max(...)` before the area
/// term ever runs, and that floor's own growth can only ever push `w` UP —
/// so a magnitude bug in the area's own scaling can hide entirely behind a
/// green width-floor check. Graded directly against [`Area::px2`], the one
/// function that carries the property: doubling the display factor must
/// QUADRUPLE the floor, not double it, which is what a length's linear
/// `.px()` would give if the area were mistyped into that family.
#[test]
fn the_area_floor_scales_by_the_square_of_the_display_factor() {
    let base = CARET_VISUAL_BODY_MIN_AREA.px2(1.0);
    assert!(base > 0.0, "an inert floor would make every ratio 0/0");
    for factor in [1.5f32, 2.0, 3.0] {
        let got = CARET_VISUAL_BODY_MIN_AREA.px2(factor);
        let want = base * factor * factor;
        let linear = base * factor;
        assert!(
            (got - want).abs() < 1e-3,
            "factor {factor}: Area::px2 gave {got}, the SQUARE relationship \
             wants {want} (a length's linear `.px()` would give {linear} \
             instead — exactly the under-scale a mistyped `Logical` would \
             produce)"
        );
    }
}

/// **THE AREA FLOOR IS GENUINELY LOAD-BEARING AT ORDINARY SCALE**, so the
/// property above is not proven of a term nothing ever reaches. At `px = 1`
/// a narrow, short ink box (well under both the width and height floors on
/// its own) still settles to exactly the authored floor.
#[test]
fn the_area_floor_engages_on_ink_under_every_floor_at_px_one() {
    let ink = InkBox {
        left: 0.0,
        top: 0.0,
        width: 2.0,
        height: 3.0,
    };
    let (w, h) = caret_visual_body_dims(ink, 1.0);
    let area = w * h;
    let want = CARET_VISUAL_BODY_MIN_AREA.px2(1.0);
    assert!(
        (area - want).abs() < 1e-3,
        "px 1: body area is {area}, the authored floor is {want} — the area \
         term did not engage for ink this small"
    );
}
