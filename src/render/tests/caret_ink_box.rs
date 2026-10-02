//! Adaptive padded block bounds: measured ink, every world, both DPI tiers.
use super::super::*;
use super::{headless_pipeline, view};

#[test]
fn adaptive_caret_contains_every_resolved_grapheme_in_both_axes() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    let mut resolved = 0;
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        for world in theme::THEMES {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for text in [
                "x", "a", "H", "g", "Å", "W", "A\u{30a}", "漢", "。", "［", ",", "_",
            ] {
                p.set_view(&view(text, 0, 0));
                p.settle_caret();
                let Some(ink) = p.caret_anchor_ink_box() else {
                    continue;
                };
                let (cx, cy, w, h, radius, ..) = p.caret_geometry();
                let left = p.caret.pos.x + ink.left;
                let top = p.caret_baseline_y() - ink.top;
                let pad = p.metrics.scale;
                assert!(
                    cx - w * 0.5 <= left - pad + 0.01
                        && cx + w * 0.5 >= left + ink.width + pad - 0.01,
                    "{} dpi={dpi} {text}: width must contain complete ink plus margin",
                    world.name
                );
                assert!(
                    cy - h * 0.5 <= top - pad + 0.01
                        && cy + h * 0.5 >= top + ink.height + pad - 0.01,
                    "{} dpi={dpi} {text}: height must contain accents/descenders plus margin",
                    world.name
                );
                assert!(
                    radius >= (3.5 * pad).min(w * 0.5).min(h * 0.5) - 0.01,
                    "{} dpi={dpi} {text}: rounded body must remain visibly softer",
                    world.name
                );
                // Test the actual rounded boundary, not just its enclosing box.
                // The entire raster rectangle conservatively includes every serif,
                // accent and independently positioned combining mark.
                for x in [left, left + ink.width] {
                    for y in [top, top + ink.height] {
                        let qx = (x - cx).abs() - (w * 0.5 - radius);
                        let qy = (y - cy).abs() - (h * 0.5 - radius);
                        let distance =
                            qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
                        assert!(
                            distance <= -0.6 * pad,
                            "{} dpi={dpi} {text}: curved corner must contain complete ink \
                             with antialias margin: distance={distance} radius={radius}",
                            world.name
                        );
                    }
                }
                resolved += 1;
            }
        }
    }
    assert!(
        resolved > theme::THEMES.len() * 12,
        "resolved font roster must be exercised"
    );
}

#[test]
fn paperbark_caret_height_and_width_follow_actual_letter_ink() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        let mut sizes = Vec::new();
        for text in ["x", "a", "H", "g", "Å", "W"] {
            p.set_view(&view(text, 0, 0));
            p.settle_caret();
            let (_, _, w, h, ..) = p.caret_geometry();
            sizes.push((w, h));
        }
        assert!(
            sizes[0].1 < sizes[2].1,
            "x must be shorter than H at dpi={dpi}: {sizes:?}"
        );
        assert!(
            sizes[4].1 > sizes[2].1,
            "Å must include its ring at dpi={dpi}: {sizes:?}"
        );
        assert!(
            sizes[5].0 > sizes[0].0,
            "W must be wider than x at dpi={dpi}: {sizes:?}"
        );
    }
}

#[test]
fn heading_caret_tracks_shaped_font_size_in_mono_and_proportional_worlds() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    for world in ["Gumtree", "Tawny"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            p.set_dpi(dpi);
            let mut heights = Vec::new();
            for text in ["x", "### x", "## x", "# x"] {
                let mut v = view(text, 0, text.chars().count() - 1);
                v.is_markdown = true;
                p.set_view(&v);
                p.settle_caret();
                heights.push(p.caret_geometry().3);
            }
            assert!(
                heights.windows(2).all(|h| h[1] > h[0]),
                "{world} dpi={dpi}: {heights:?}"
            );
        }
    }
}

#[test]
fn moving_caret_streak_is_unaffected_by_the_ink_box() {
    let _t = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let _g = crate::testlock::serial();
    let _c = crate::testlock::serial();
    crate::caret::set_mode(CaretMode::Block);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping moving_caret_streak_is_unaffected_by_the_ink_box: no wgpu adapter");
        return;
    };
    let text = "alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta\ntheta\niota";

    for world in ["Gumtree", "Tawny"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        p.set_view(&view(text, 0, 0));

        // HORIZONTAL fast glide (settle ≈ 0): the deterministic mid-motion pose the
        // `--screenshot-motion` capture renders.
        p.inject_motion_demo();
        let (_cx, cy, w, h, _c, _ax, _ay) = p.caret_geometry();
        let s = p.caret.settle_factor();
        assert!(
            s < 0.2,
            "{world}: fixture must be genuinely mid-glide (s={s})"
        );
        assert!(
            w > h,
            "{world}: the motion pose must be long-and-thin: w={w} h={h}"
        );
        assert!(
            h < p.metrics.caret_block_h * 0.5,
            "{world}: the streak must stay thin — the ink box must not thicken it: h={h}"
        );
        let want_cy = p.caret.pos.y + p.metrics.caret_trail_drop;
        assert!(
            (cy - want_cy).abs() < 0.5,
            "{world}: the streak must run through the TEXT centre, NOT be pulled onto \
             the ink box: cy={cy} want={want_cy}"
        );

        // VERTICAL fast glide: same rule, other axis.
        p.inject_motion_demo_vertical();
        let (_cx, _cy, w_v, h_v, ..) = p.caret_geometry();
        assert!(
            w_v > h_v,
            "{world}: the vertical motion pose must stay long-and-thin: w={w_v} h={h_v}"
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}
