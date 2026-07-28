//! Laws for the capture layout oracle's shaped-frame ownership.

use super::super::*;
use super::headless_dqp;

fn prepare(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) {
    p.prepare(device, queue, width, height)
        .expect("frame prepares");
}

#[test]
fn layout_report_reads_proportional_shaped_reality_and_locates_state() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(false);
    crate::theme::set_active_by_name("Gumtree").unwrap(); // proportional Literata
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping layout_report_reads_proportional_shaped_reality: no wgpu");
        return;
    };
    let mut v = super::view("iiiiWWWW narrow and wide", 0, 5);
    v.selection = Some(((0, 1), (0, 7)));
    p.set_view(&v);
    assert!(
        p.layout_report().is_none(),
        "an unprepared pipeline must not assemble a report on demand"
    );
    prepare(&mut p, &device, &queue, 1200, 800);
    let report = p.layout_report().expect("prepared frame is reportable");
    let row = &report.rows[0];
    assert_eq!(row.content, "iiiiWWWW narrow and wide");
    assert_eq!((row.logical_line, row.start_col, row.end_col), (0, 0, 24));
    assert_eq!(row.xs.len(), row.end_col - row.start_col + 1);
    let i_advance = row.xs[1] - row.xs[0];
    let w_advance = row.xs[5] - row.xs[4];
    assert!(
        (i_advance - w_advance).abs() > 2.0,
        "the witness must disagree with fixed pitch: i={i_advance}, W={w_advance}"
    );
    assert!(
        (row.xs[4] - row.xs[0] - 4.0 * p.metrics.char_width).abs() > 2.0,
        "the reported boundary must be shaped Literata geometry, not col × fixed pitch"
    );
    let caret = report.caret.as_ref().expect("caret located in rows");
    assert_eq!((caret.row, caret.logical_line, caret.col), (0, 0, 5));
    assert_eq!(caret.x, row.xs[5]);
    assert_eq!(report.selection.len(), 1);
    let selection = &report.selection[0];
    assert_eq!(
        (selection.row, selection.start_col, selection.end_col),
        (0, 1, 7)
    );
    assert_eq!((selection.x0, selection.x1), (row.xs[1], row.xs[7]));
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn layout_report_moves_rows_when_the_real_wrap_width_changes() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::theme::set_active_by_name("Gumtree").unwrap();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping layout_report_moves_rows_when_wrap_width_changes: no wgpu");
        return;
    };
    let text = "proportional wrapping witness ".repeat(16);
    crate::page::set_measure(60);
    p.set_view(&super::view(&text, 0, 0));
    prepare(&mut p, &device, &queue, 1200, 800);
    let wide = p.layout_report().unwrap();

    crate::page::set_measure(20);
    prepare(&mut p, &device, &queue, 1200, 800);
    let narrow = p.layout_report().unwrap();
    assert!(
        narrow.rows.len() > wide.rows.len(),
        "narrowing the drawn column must increase visual rows: wide={} narrow={}",
        wide.rows.len(),
        narrow.rows.len()
    );
    assert_ne!(
        wide.rows[0].end_col, narrow.rows[0].end_col,
        "the first shaped wrap boundary must move with the real width"
    );
    assert!(
        narrow.rows.windows(2).all(|pair| pair[1].top > pair[0].top),
        "reported rows stay in shaped top-to-bottom order"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn layout_report_borrows_the_sealed_frame_without_assembly_or_row_clones() {
    let _g = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(false);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping layout_report_borrows_sealed_frame: no wgpu");
        return;
    };
    p.set_view(&super::view(
        "one shaped line\nand a second proportional line",
        1,
        8,
    ));
    prepare(&mut p, &device, &queue, 1200, 800);

    geometry::reset_glyph_x_assembly_count();
    rowgeom::reset_layout_report_ownership_counts();
    let report = p.layout_report().expect("sealed frame report");
    assert_eq!(report.rows.len(), 2, "non-vacuous multi-row witness");
    assert_eq!(
        geometry::glyph_x_assembly_count(),
        0,
        "the report must never rebuild glyph x geometry"
    );
    assert_eq!(
        rowgeom::visual_row_clone_count(),
        0,
        "the report must borrow frame rows rather than clone VisualRows"
    );
    assert_eq!(
        rowgeom::report_row_borrow_count(),
        1,
        "one report is one in-place borrow of the drawn frame partition"
    );
}
