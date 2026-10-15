//! Followable text keeps its underline distance when another run moves the baseline.
use super::super::*;
use super::{headless_pipeline, view_md};

fn gaps(p: &TextPipeline, line: usize, start: usize, end: usize) -> Vec<f32> {
    let baselines: Vec<_> = p
        .buffer
        .layout_runs()
        .filter(|run| {
            run.line_i == line && run.glyphs.iter().any(|g| g.start < end && g.end > start)
        })
        .map(|run| p.doc_top() + run.line_y)
        .collect();
    let underlines = p.link_underlines();
    assert_eq!(
        underlines.len(),
        baselines.len(),
        "one underline per linked row"
    );
    underlines
        .iter()
        .zip(baselines)
        .map(|(s, baseline)| s.y + s.h * 0.5 - baseline)
        .collect()
}

#[test]
fn wrapped_link_underlines_keep_the_unwrapped_baseline_gap() {
    let _guard = crate::testlock::serial();
    let previous_world = theme::active_index();
    let previous_page = (crate::page::page_on(), crate::page::measure());
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping link baseline law: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    let label = "minimum ".repeat(24);
    let mut cases = 0;
    for world in theme::THEMES {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            p.set_dpi(dpi);
            p.set_size(1200.0 * dpi, 900.0 * dpi);
            for measure in [40, 55] {
                crate::page::set_measure(measure);
                p.set_view(&view_md("[minimum](https://example.com)\n\nrest", 2, 0));
                let expected = gaps(&p, 0, 1, 8)[0];
                let [_, top, _, height] = p.range_rects((0, 1), (0, 8))[0];
                let (old_y, _, _) = spans::link_underline_band(top, height, p.metrics.scale);
                assert!(
                    (p.link_underlines()[0].y - old_y).abs() < 0.001,
                    "ordinary unwrapped placement must be preserved: {}",
                    world.name
                );
                for bare in [false, true] {
                    let linked = if bare {
                        format!("https://example.com/{}", "minimum/".repeat(24))
                    } else {
                        format!("[{label}](https://example.com)")
                    };
                    let text = format!("- Parent\n  - {linked}\n\nrest");
                    let start = if bare { 4 } else { 5 };
                    let end = if bare {
                        4 + linked.len()
                    } else {
                        5 + label.len()
                    };
                    // Moving the caret and selection also exercises cache invalidation.
                    for state in [0, 1, 2] {
                        let mut v = view_md(&text, if state == 1 { 1 } else { 3 }, 0);
                        if state == 2 {
                            v.selection = Some(((1, 0), (1, 2)));
                        }
                        p.set_view(&v);
                        let actual = gaps(&p, 1, start, end);
                        assert!(!actual.is_empty());
                        if !bare || state != 0 {
                            assert!(actual.len() > 1, "fixture must wrap");
                        }
                        for (row, gap) in actual.iter().enumerate() {
                            assert!(
                                (gap - expected).abs() < 0.05 * dpi,
                                "{} dpi={dpi} measure={measure} state={state} bare={bare} \
                                 row={row}: gap={gap}, unwrapped={expected}",
                                world.name
                            );
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, theme::THEMES.len() * 24);
    eprintln!(
        "verified {cases} link states across {} worlds",
        theme::THEMES.len()
    );
    theme::set_active(previous_world);
    crate::page::set_page_on(previous_page.0);
    crate::page::set_measure(previous_page.1);
}
