//! Differential laws for retaining the document row partition across ordinary
//! text edits. The optimized pipeline is compared with a whole-buffer reshape;
//! edits that can move later rows must reject the retained path.

use super::super::*;
use super::{dither, headless_dqp, headless_pipeline, view};

fn assert_close(a: f32, b: f32, label: &str) {
    assert!((a - b).abs() <= 0.001, "{label}: {a} != {b}");
}

fn assert_same_geometry(a: &TextPipeline, b: &TextPipeline, lines: usize, label: &str) {
    assert_eq!(
        a.total_visual_rows(),
        b.total_visual_rows(),
        "{label}: rows"
    );
    assert_close(a.total_doc_height(), b.total_doc_height(), label);
    assert_eq!(a.caret_snapshot().1, b.caret_snapshot().1, "{label}: caret");
    for line in 0..lines {
        assert_close(
            a.row_geom.line_first_top(&a.buffer, &a.metrics, line),
            b.row_geom.line_first_top(&b.buffer, &b.metrics, line),
            label,
        );
        assert_close(
            a.row_geom.line_first_baseline(&a.buffer, &a.metrics, line),
            b.row_geom.line_first_baseline(&b.buffer, &b.metrics, line),
            label,
        );
        assert_close(
            a.row_geom.line_last_top(&a.buffer, &a.metrics, line),
            b.row_geom.line_last_top(&b.buffer, &b.metrics, line),
            label,
        );
        assert_close(
            a.row_geom.line_last_baseline(&a.buffer, &a.metrics, line),
            b.row_geom.line_last_baseline(&b.buffer, &b.metrics, line),
            label,
        );
        let ar = a.visual_rows(line);
        let br = b.visual_rows(line);
        assert_eq!(ar.len(), br.len(), "{label}: line {line} row count");
        for (ar, br) in ar.iter().zip(&br) {
            assert_close(ar.line_top, br.line_top, label);
            assert_close(ar.line_height, br.line_height, label);
            assert_eq!(ar.start_col, br.start_col, "{label}: row start");
            assert_eq!(ar.end_col, br.end_col, "{label}: row end");
            assert_eq!(ar.xs, br.xs, "{label}: glyph boundaries");
        }
    }
}

fn set_full_truth(p: &mut TextPipeline, text: &str, line: usize, col: usize) {
    p.set_text_full(text);
    p.set_view(&view(text, line, col));
}

#[test]
fn retained_geometry_matches_full_reshape_across_size_position_unicode_and_history() {
    let _guard = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let Some(mut incremental) = headless_pipeline() else {
        eprintln!("skipping retained_geometry_matches_full_reshape: no wgpu adapter");
        return;
    };
    let Some(mut full) = headless_pipeline() else {
        return;
    };
    incremental.enable_text_sync_profile();

    for &line_count in &[3usize, 257] {
        let mut lines: Vec<String> = (0..line_count)
            .map(|i| format!("line {i}: calm prose with café 東京 and 👩‍💻"))
            .collect();
        let seed = lines.join("\n");
        incremental.set_view(&view(&seed, 0, 0));
        for &line in &[0, line_count / 2, line_count - 1] {
            let before = lines[line].clone();
            lines[line].push('z');
            let edited = lines.join("\n");
            let col = lines[line].chars().count();
            incremental.set_view(&view(&edited, line, col));
            let work = incremental.text_sync_phases();
            assert_eq!(work.geometry_patch_hits, 1, "{line_count}/{line}: edit");
            assert_eq!(work.geometry_lines_patched, 1, "{line_count}/{line}: edit");
            assert_eq!(work.geometry_rows_patched, 1, "{line_count}/{line}: edit");
            assert!(
                work.geometry_index_probes <= 3 * (usize::BITS - line_count.leading_zeros()) as u64,
                "binary lookup stayed bounded"
            );
            set_full_truth(&mut full, &edited, line, col);
            assert_same_geometry(&incremental, &full, line_count, "edit");

            lines[line] = before;
            let undone = lines.join("\n");
            let undo_col = lines[line].chars().count();
            incremental.set_view(&view(&undone, line, undo_col));
            assert_eq!(
                incremental.text_sync_phases().geometry_patch_hits,
                1,
                "undo"
            );
            set_full_truth(&mut full, &undone, line, undo_col);
            assert_same_geometry(&incremental, &full, line_count, "undo");

            lines[line].push('z');
            let redone = lines.join("\n");
            incremental.set_view(&view(&redone, line, col));
            assert_eq!(
                incremental.text_sync_phases().geometry_patch_hits,
                1,
                "redo"
            );
            set_full_truth(&mut full, &redone, line, col);
            assert_same_geometry(&incremental, &full, line_count, "redo");
        }
    }
}

#[test]
fn partial_single_line_shape_rejects_before_materializing_replacement_rows() {
    let _guard = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping partial_single_line_shape_rejects_before_materializing: no wgpu adapter"
        );
        return;
    };
    p.enable_text_sync_profile();

    let seed = "calm prose words ".repeat(5_000);
    p.set_view(&view(&seed, 0, 0));
    p.total_visual_rows();
    let shaped_rows = p.buffer.lines[0]
        .layout_opt()
        .map(Vec::len)
        .expect("the huge line is shaped");
    let mut probes = 0;
    let cached_rows = p
        .row_geom
        .cached_line_row_count(0, &mut probes)
        .expect("the presented row partition is cached");
    assert!(
        shaped_rows > cached_rows,
        "fixture must expose partial presentation: shaped={shaped_rows}, cached={cached_rows}"
    );

    let edited = format!("z{seed}");
    p.set_view(&view(&edited, 0, 1));
    let work = p.text_sync_phases();
    assert_eq!(work.geometry_patch_hits, 0, "partial partition must reject");
    assert_eq!(
        work.geometry_rows_materialized, 0,
        "row-count mismatch must reject before replacement rows are built"
    );
}

#[test]
fn retained_geometry_rejects_edits_that_can_move_other_rows() {
    let _guard = crate::testlock::serial();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    let _page = crate::page::PagePin::snapshot();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    crate::theme::set_active_by_name("Gumtree").expect("Gumtree exists");
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping retained_geometry_rejects_edits_that_can_move_other_rows: no adapter");
        return;
    };
    p.enable_text_sync_profile();

    p.set_view(&view("one\ntwo\nthree", 1, 3));
    p.set_view(&view("one\ntwo\nnew\nthree", 2, 3));
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "line insertion"
    );

    let mut markdown = view("before\nnot a fence\nafter", 1, 3);
    markdown.is_markdown = true;
    p.set_view(&markdown);
    markdown.text = "before\n```\nafter".to_string();
    p.set_view(&markdown);
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "fence boundary"
    );

    let mut table = view(
        "| Name | Note |\n| --- | --- |\n| wren | a short note |",
        2,
        10,
    );
    table.is_markdown = true;
    p.set_view(&table);
    table.text = "| Name | Note |\n| --- | --- |\n| wren | a much wider table note |".to_string();
    p.set_view(&table);
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "table structure"
    );

    let mut heading = view("plain heading\nbody", 0, 3);
    heading.is_markdown = true;
    p.set_view(&heading);
    heading.text = "# Tall heading\nbody".to_string();
    p.set_view(&heading);
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "changed row height"
    );

    let mut frontmatter = view("---\nlang: en\n---\n東京 prose", 1, 8);
    frontmatter.is_markdown = true;
    p.set_view(&frontmatter);
    frontmatter.text = "---\nlang: ja\n---\n東京 prose".to_string();
    p.set_view(&frontmatter);
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "document language scope"
    );

    let mut wrapped = "x".repeat(8);
    p.set_view(&view(&wrapped, 0, wrapped.len()));
    let rows_before = p.visual_rows(0).len();
    let mut crossed = false;
    for _ in 0..500 {
        wrapped.push('x');
        p.set_view(&view(&wrapped, 0, wrapped.len()));
        let rows = p.visual_rows(0).len();
        if rows != rows_before {
            assert_eq!(p.text_sync_phases().geometry_patch_hits, 0, "wrap boundary");
            crossed = true;
            break;
        }
    }
    assert!(crossed, "fixture must cross a wrap boundary");

    let seed = "short line\n".repeat(40);
    p.set_view(&view(&seed, 20, 2));
    let old_width = p.buffer.size().0;
    crate::page::set_measure(crate::page::DEFAULT_MEASURE_CODE);
    let edited = seed.replacen("short line", "short line!", 1);
    p.set_view(&view(&edited, 0, 11));
    assert_ne!(
        p.buffer.size().0,
        old_width,
        "page measure changes wrap width"
    );
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        0,
        "page width change"
    );

    let previous_generation = p.row_geom.generation();
    crate::theme::set_active_by_name("Tawny").expect("Tawny exists");
    p.sync_theme();
    assert!(
        p.row_geom.generation() > previous_generation,
        "font/world invalidates"
    );
    let reshapes = p.reshape_count;
    p.set_view(&view(&edited, 10, 3));
    assert_eq!(
        p.reshape_count, reshapes,
        "cursor movement does not reshape"
    );

    let edited_again = edited.replacen("short line!", "short line!?", 1);
    p.set_view(&view(&edited_again, 0, 12));
    assert_eq!(
        p.text_sync_phases().geometry_patch_hits,
        1,
        "post-world edit"
    );
}

#[test]
fn conceal_cursor_change_invalidates_then_matches_fresh_geometry() {
    let _guard = crate::testlock::serial();
    let _misc = crate::testlock::misc::TogglesRestore::capture();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut reused) = headless_pipeline() else {
        eprintln!("skipping conceal_cursor_change_invalidates: no wgpu adapter");
        return;
    };
    let Some(mut fresh) = headless_pipeline() else {
        return;
    };
    let text = "*emphasis* and plain text\nsecond line";
    let mut on_markup = view(text, 0, 2);
    on_markup.is_markdown = true;
    reused.set_view(&on_markup);
    let generation = reused.row_geom.generation();
    let reshapes = reused.reshape_count;

    let mut off_markup = view(text, 1, 2);
    off_markup.is_markdown = true;
    reused.set_view(&off_markup);
    assert_eq!(
        reused.reshape_count, reshapes,
        "cursor move is not a text reshape"
    );
    assert!(
        reused.row_geom.generation() > generation,
        "revealing/concealing glyphs invalidates retained geometry"
    );
    fresh.set_view(&off_markup);
    assert_same_geometry(&reused, &fresh, 2, "conceal cursor move");
}

fn render_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    width: u32,
    height: u32,
) -> Vec<[u8; 4]> {
    p.settle_caret();
    p.prepare(device, queue, width, height).unwrap();
    let (texture, target) = dither::offscreen(device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("retained geometry differential"),
    });
    p.render(&mut encoder, &target).unwrap();
    queue.submit(Some(encoder.finish()));
    dither::read_pixels(device, queue, &texture, width, height)
}

#[test]
fn retained_geometry_renders_identically_to_a_full_reshape() {
    let _guard = crate::testlock::serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut incremental)) = headless_dqp(800.0, 600.0) else {
        eprintln!("skipping retained_geometry_renders_identically: no wgpu adapter");
        return;
    };
    let Some((_device2, _queue2, mut full)) = headless_dqp(800.0, 600.0) else {
        return;
    };
    incremental.enable_text_sync_profile();
    let seed = (0..90)
        .map(|i| format!("line {i}: a quiet sentence with café"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut seed_view = view(&seed, 45, 4);
    seed_view.scroll = ScrollPos::at_row(38);
    incremental.set_view(&seed_view);
    let seed_pixels = render_pixels(&device, &queue, &mut incremental, 800, 600);
    let edited = seed.replacen("line 45:", "line 45: z", 1);
    let mut edited_view = view(&edited, 45, 10);
    edited_view.scroll = ScrollPos::at_row(38);
    incremental.set_view(&edited_view);
    assert_eq!(incremental.text_sync_phases().geometry_patch_hits, 1);
    full.set_text_full(&edited);
    full.set_view(&edited_view);

    let incremental_pixels = render_pixels(&device, &queue, &mut incremental, 800, 600);
    let full_pixels = render_pixels(&device, &queue, &mut full, 800, 600);
    assert_ne!(
        seed_pixels, incremental_pixels,
        "visible edited glyphs must change pixels"
    );
    assert_eq!(
        incremental_pixels, full_pixels,
        "retained and full frames differ"
    );
}

#[test]
fn empty_row_patch_is_rejected_without_mutating_the_cache() {
    let _guard = crate::testlock::serial();
    let rows = &mut [];
    let geom = rowgeom::RowGeom::new();
    let generation = geom.generation();
    assert!(geom.patch_lines_if_stable(rows).is_none());
    assert_eq!(geom.generation(), generation);

    let Some(mut populated) = headless_pipeline() else {
        eprintln!("skipping populated empty-row patch arm: no wgpu adapter");
        return;
    };
    populated.set_view(&view("one\ntwo", 0, 0));
    let generation = populated.row_geom.generation();
    let patches = &mut [(0, Vec::new())];
    assert!(populated.row_geom.patch_lines_if_stable(patches).is_none());
    assert_eq!(populated.row_geom.generation(), generation);
}
