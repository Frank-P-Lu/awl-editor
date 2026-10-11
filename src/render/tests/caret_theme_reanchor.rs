//! Theme reflow must update the drawn caret, not only its geometry oracle.

use super::super::*;
use super::{headless_pipeline, view};

fn assert_anchored(p: &TextPipeline, label: &str) {
    let expected = p.caret_target_xy();
    let (position, target, _, animating) = p.caret_snapshot();
    for actual in [position, target] {
        assert!(
            (actual.0 - expected.0).abs() < 0.01 && (actual.1 - expected.1).abs() < 0.01,
            "{label}: drawn caret {actual:?}, shaped insertion cell {expected:?}"
        );
    }
    assert!(!animating, "{label}: theme reflow is not cursor navigation");
}

#[test]
fn theme_reflow_reanchors_drawn_caret_across_worlds_and_sync_paths() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(40);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping theme_reflow_reanchors_drawn_caret: no wgpu adapter");
        return;
    };
    let cases = [
        (
            "The quick brown fox with café 東京 and e\u{301}.",
            0,
            10,
            false,
        ),
        ("## Heading with wide letters WWW\n\nbody", 0, 16, true),
        (
            "A wrapping paragraph with many narrow letters iii and wider letters WWW \
             that crosses the writing column.\nCaret below the reflow.",
            1,
            12,
            false,
        ),
    ];
    let mut changed_geometry = 0;
    assert!(!theme::THEMES.is_empty());
    for dpi in [1.0, 2.0] {
        p.set_dpi(dpi);
        for width in [520.0, 1200.0] {
            p.set_size(width * dpi, 800.0 * dpi);
            for (mode, previous) in [
                (CaretMode::Ibeam, false),
                (CaretMode::Block, false),
                (CaretMode::Block, true),
            ] {
                crate::caret::set_mode(mode);
                crate::caret::set_highlight_previous_character(previous);
                for timed in [false, true] {
                    for (text, line, col, markdown) in cases {
                        theme::set_active_by_name("Tawny").unwrap();
                        p.sync_theme();
                        let mut v = view(text, line, col);
                        v.is_markdown = markdown;
                        v.selection = Some(((line, col - 2), (line, col)));
                        p.set_view(&v);
                        p.settle_caret();
                        for world in theme::THEMES {
                            theme::set_active_by_name(world.name).unwrap();
                            // Keyboard actions sync the view before the retint;
                            // no later set_view may repair the test's answer.
                            p.set_view(&v);
                            let old_target = p.caret_snapshot().1;
                            p.sync_theme_colors();
                            if timed {
                                p.sync_theme_font_timed(ShapeReach::Presentable);
                            } else {
                                p.sync_theme_font(ShapeReach::Whole);
                            }
                            let label = format!(
                                "{} dpi={dpi} width={width} mode={mode:?} \
                                 previous={previous} timed={timed} line={line}",
                                world.name
                            );
                            assert_anchored(&p, &label);
                            changed_geometry += usize::from(old_target != p.caret_target_xy());
                            assert_eq!((p.cursor_line, p.cursor_col), (line, col), "{label}");
                            assert_eq!(p.selection, v.selection, "{label}");
                            p.finish_shape_tail();
                            assert_anchored(&p, &label);
                        }
                    }
                }
            }
        }
    }
    assert!(
        changed_geometry > 0,
        "the sweep must exercise real caret reflow"
    );
}

#[test]
fn preview_burst_commit_and_revert_keep_caret_anchored_with_shape_tail() {
    let _guard = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(40);
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping preview_burst_caret_shape_tail: no wgpu adapter");
        return;
    };
    let text = "The quick brown fox\n".repeat(400);
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();
    let v = view(&text, 4, 10);
    p.set_view(&v);
    p.settle_caret();
    for world in ["Gumtree", "Bilby", "Quokka"] {
        theme::set_active_by_name(world).unwrap();
        p.set_view(&v);
        p.sync_theme_colors();
        p.sync_theme_font_timed(ShapeReach::Presentable);
        assert!(p.shape_tail_owed(), "{world}: fixture must leave a tail");
        assert_anchored(&p, world);
    }
    // Accepting the preview does not reshape again, but finishes its tail.
    p.sync_theme_font_timed(ShapeReach::Whole);
    assert!(p.finish_shape_tail());
    assert_anchored(&p, "commit");
    // Escape returns to the original world's geometry without moving text state.
    theme::set_active_by_name("Tawny").unwrap();
    p.set_view(&v);
    p.sync_theme_colors();
    p.sync_theme_font_timed(ShapeReach::Whole);
    assert!(!p.finish_shape_tail());
    assert_anchored(&p, "revert");
    assert_eq!((p.cursor_line, p.cursor_col), (4, 10));
}

#[test]
fn unchanged_theme_sync_preserves_navigation_motion() {
    let _guard = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping unchanged_theme_sync_preserves_navigation_motion: no wgpu adapter");
        return;
    };
    p.set_view(&view("The quick brown fox jumps over the lazy dog", 0, 0));
    p.sync_theme();
    p.inject_motion_demo();
    let before = p.caret_snapshot();
    assert!(before.3, "fixture must carry active navigation motion");
    assert!(!p.needs_theme_reshape());
    p.sync_theme();
    assert!(p.sync_theme_font_timed(ShapeReach::Whole).is_none());
    assert!(!p.finish_shape_tail());
    assert_eq!(p.caret_snapshot(), before);
}
