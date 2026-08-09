//! Construction and live switching share one tint owner. A capture creates a
//! pipeline only once, so only this direct pipeline reading can see a mismatch.

use super::super::*;
use super::headless_pipeline;

#[test]
fn every_sync_owned_pipeline_is_tinted_the_same_after_construction_and_a_live_sync() {
    let _g = crate::testlock::serial();

    for (i, t) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        let Some(mut p) = headless_pipeline() else {
            eprintln!("skipping pipeline-tint construction law: no wgpu adapter");
            return;
        };

        let constructed = sync_owned_tints(&p);
        p.sync_theme_colors();
        assert_eq!(
            constructed,
            sync_owned_tints(&p),
            "{}: construction must use sync_theme_colors as its sole tint owner",
            t.name
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
}

/// Exactly the fields `sync_theme_colors` writes. Per-frame owners such as the
/// footer rim and selected spine are deliberately absent.
fn sync_owned_tints(p: &TextPipeline) -> Vec<(&'static str, Vec<f32>)> {
    let mut tints = sync_owned_surface_tints(p);
    tints.extend(sync_owned_overlay_tints(p));
    tints.extend(sync_owned_annotation_tints(p));
    tints
}

fn sync_owned_surface_tints(p: &TextPipeline) -> Vec<(&'static str, Vec<f32>)> {
    vec![
        ("caret_pipeline", p.caret_pipeline.test_color().to_vec()),
        (
            "caret_trail_pipeline",
            p.caret_trail_pipeline.test_color().to_vec(),
        ),
        (
            "caret_glyph_pipeline",
            p.caret_glyph_pipeline.test_color().to_vec(),
        ),
        (
            "selection_pipeline",
            p.selection_pipeline.test_color().to_vec(),
        ),
        ("match_pipeline", p.match_pipeline.test_color().to_vec()),
        (
            "wash_comment_pipeline",
            p.wash_comment_pipeline.test_color().to_vec(),
        ),
        (
            "wash_string_pipeline",
            p.wash_string_pipeline.test_color().to_vec(),
        ),
        (
            "wash_highlight_pipeline",
            p.wash_highlight_pipeline.test_color().to_vec(),
        ),
        (
            "fence_panel_pipeline",
            p.fence_panel_pipeline.test_color().to_vec(),
        ),
        (
            "code_pill_pipeline",
            p.code_pill_pipeline.test_color().to_vec(),
        ),
        (
            "image_placeholder_pipeline",
            p.image_placeholder_pipeline.test_color().to_vec(),
        ),
        (
            "image_scrim_pipeline",
            p.image_scrim_pipeline.test_color().to_vec(),
        ),
        (
            "table_rule_pipeline",
            p.table_rule_pipeline.test_color().to_vec(),
        ),
        (
            "fold_chevron_pipeline",
            p.fold_chevron_pipeline.test_color().to_vec(),
        ),
        ("panel_card", p.panel_card.test_color().to_vec()),
        ("panel_shadow", p.panel_shadow.test_color().to_vec()),
        ("panel_border", p.panel_border.test_color().to_vec()),
        ("hud_shadow", p.hud_shadow.test_color().to_vec()),
        ("hud_border", p.hud_border.test_color().to_vec()),
        ("hud_card", p.hud_card.test_color().to_vec()),
        ("wk_shadow", p.wk_shadow.test_color().to_vec()),
        ("wk_border", p.wk_border.test_color().to_vec()),
        ("wk_card", p.wk_card.test_color().to_vec()),
        ("popover_wash", p.popover_wash.test_color().to_vec()),
        ("popover_hl_wash", p.popover_hl_wash.test_color().to_vec()),
        ("popover_strike", p.popover_strike.test_color().to_vec()),
        ("menubar_bg", p.menubar_bg.test_color().to_vec()),
        ("menubar_hi", p.menubar_hi.test_color().to_vec()),
        ("menu_drop_shadow", p.menu_drop_shadow.test_color().to_vec()),
        ("menu_drop_border", p.menu_drop_border.test_color().to_vec()),
        ("menu_drop_card", p.menu_drop_card.test_color().to_vec()),
        ("menu_drop_sep", p.menu_drop_sep.test_color().to_vec()),
        ("panel_caret", p.panel_caret.test_color().to_vec()),
        (
            "caret_preview_pipeline",
            p.caret_preview_pipeline.test_color().to_vec(),
        ),
        (
            "caret_preview_glyph_pipeline",
            p.caret_preview_glyph_pipeline.test_color().to_vec(),
        ),
        ("float_shadow", p.float_shadow.test_color().to_vec()),
        ("float_border", p.float_border.test_color().to_vec()),
        ("float_card", p.float_card.test_color().to_vec()),
    ]
}

fn sync_owned_overlay_tints(p: &TextPipeline) -> Vec<(&'static str, Vec<f32>)> {
    vec![
        ("overlay_rows", p.overlay_rows.test_color().to_vec()),
        ("overlay_bars", p.overlay_bars.test_color().to_vec()),
        ("overlay_spine", p.overlay_spine.test_color().to_vec()),
        ("overlay_cross", p.overlay_cross.test_color().to_vec()),
        (
            "overlay_range_track",
            p.overlay_range_track.test_color().to_vec(),
        ),
        (
            "overlay_range_thumb",
            p.overlay_range_thumb.test_color().to_vec(),
        ),
        (
            "overlay_lens_underline",
            p.overlay_lens_underline.test_color().to_vec(),
        ),
    ]
}

fn sync_owned_annotation_tints(p: &TextPipeline) -> Vec<(&'static str, Vec<f32>)> {
    vec![
        ("spell_pipeline", p.spell_pipeline.test_color().to_vec()),
        ("nit_pipeline", p.nit_pipeline.test_color().to_vec()),
        ("strike_pipeline", p.strike_pipeline.test_color().to_vec()),
        (
            "link_underline_pipeline",
            p.link_underline_pipeline.test_color().to_vec(),
        ),
        (
            "page_frame_pipeline",
            p.page_frame_pipeline.test_color().to_vec(),
        ),
        ("placard_stipple", p.placard_stipple.test_color().to_vec()),
    ]
}
