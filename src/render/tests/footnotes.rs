//! Footnote WYSIWYG geometry: exact reveal removes the composed numeral, the
//! source-owned slot survives wrapping, and every world scales it at both DPIs.

use super::super::*;
use super::{headless_pipeline, view};

const SOURCE: &str = concat!(
    "[^earlier]: defined first\n\n",
    "B[^β] A[^earlier] B again[^β].\n\n",
    "[^β]: unicode definition\n    continued line\n\n",
    "park caret here\n",
);

fn parked_view() -> ViewState {
    let mut state = view(SOURCE, 7, 0);
    state.is_markdown = true;
    state
}

#[test]
fn footnote_numbers_compose_off_line_and_exact_source_reveals_on_caret_or_selection() {
    let _guard = crate::testlock::serial();
    let Some(mut pipeline) = headless_pipeline() else {
        eprintln!("skipping footnote reveal law: no wgpu adapter");
        return;
    };
    pipeline.set_view(&parked_view());
    let marks = pipeline.footnote_marks();
    assert_eq!(
        marks.iter().map(|mark| mark.2).collect::<Vec<_>>(),
        [2, 1, 2, 1, 1],
        "source order is retained while numbers follow first reference"
    );
    assert!(
        pipeline.concealed_at(2, 1),
        "the reference label source is collapsed off its line"
    );

    let mut caret = parked_view();
    caret.cursor_line = 2;
    caret.cursor_col = 4;
    pipeline.set_view(&caret);
    assert_eq!(
        pipeline.footnote_marks().len(),
        2,
        "all three composed references on the active line yield to exact source"
    );
    assert!(!pipeline.concealed_at(2, 1));

    let mut selected = parked_view();
    selected.selection = Some(((4, 0), (5, 5)));
    pipeline.set_view(&selected);
    assert_eq!(
        pipeline.footnote_marks().len(),
        4,
        "the selected definition line reveals its marker while unrelated marks stay composed"
    );
    assert!(!pipeline.concealed_at(4, 0));
    assert!(
        pipeline.concealed_at(2, 1),
        "selection reveal remains line-scoped"
    );
}

#[test]
fn footnote_number_slot_tracks_the_wrapped_visual_row() {
    let _guard = crate::testlock::serial();
    let Some(mut pipeline) = headless_pipeline() else {
        eprintln!("skipping footnote wrap law: no wgpu adapter");
        return;
    };
    let source = concat!(
        "This deliberately long line wraps before its final footnote reference ",
        "because the measure is narrow[^wrap].\n\n",
        "[^wrap]: definition\n\npark\n",
    );
    let mut state = view(source, 4, 0);
    state.is_markdown = true;
    pipeline.set_size(360.0, 800.0);
    pipeline.set_view(&state);
    let marks = pipeline.footnote_marks();
    assert_eq!(marks.len(), 2);
    assert!(
        marks[0].0 > pipeline.doc_top() + pipeline.metrics.line_height * 0.5,
        "the inline number sits on the wrapped row containing its source cell: {marks:?}"
    );
    assert!(marks.iter().all(|mark| mark.3 > 0.0 && mark.3.is_finite()));
}

#[test]
fn every_world_scales_footnote_geometry_at_one_and_two_dpi() {
    let _guard = crate::testlock::serial();
    let Some(mut pipeline) = headless_pipeline() else {
        eprintln!("skipping footnote world × DPI law: no wgpu adapter");
        return;
    };
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        pipeline.sync_theme();
        let mut normalized = Vec::new();
        for dpi in [1.0f32, 2.0] {
            pipeline.set_dpi(dpi);
            pipeline.set_size(1200.0 * dpi, 800.0 * dpi);
            pipeline.set_view(&parked_view());
            let marks = pipeline.footnote_marks();
            assert_eq!(marks.len(), 5, "{} at {dpi}x: {marks:?}", world.name);
            assert!(marks.iter().all(|mark| {
                mark.0.is_finite() && mark.1.is_finite() && mark.3.is_finite() && mark.3 > 0.0
            }));
            normalized.push(
                marks
                    .iter()
                    .map(|mark| (mark.0 / dpi, mark.1 / dpi, mark.2, mark.3 / dpi))
                    .collect::<Vec<_>>(),
            );
        }
        for (one, two) in normalized[0].iter().zip(&normalized[1]) {
            assert_eq!(one.2, two.2, "{}: number identity changed", world.name);
            assert!((one.0 - two.0).abs() < 0.1, "{}: row top", world.name);
            assert!((one.1 - two.1).abs() < 0.1, "{}: left", world.name);
            assert!((one.3 - two.3).abs() < 0.1, "{}: slot", world.name);
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    pipeline.sync_theme();
    pipeline.set_dpi(1.0);
}
