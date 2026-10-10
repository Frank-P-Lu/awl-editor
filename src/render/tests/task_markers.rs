//! Task-list rich-preview laws: parsed membership, marker-slot geometry,
//! world/DPI appearance, and caret/selection reveal.

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};
use std::collections::BTreeSet;

const LOGICAL_W: u32 = 720;
const LOGICAL_H: u32 = 460;
type Pixel = [u8; 4];
type MarkerMask = BTreeSet<(i32, i32)>;
type MarkerMasks = Vec<MarkerMask>;
type Frame = Vec<Pixel>;

struct RenderTarget<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    width: u32,
    height: u32,
}
const DOC: &str = concat!(
    "- [ ] open task\n",
    "- [x] checked task\n",
    "- [] invalid stays text\n",
    "- outer\n",
    "  - middle\n",
    "    - inner\n",
    "  - [ ] nested open task with a deliberately long body that wraps onto another ",
    "visual row while the marker remains attached to its first row\n",
    "    - [x] nested checked task\n",
    "anchor\n",
);
const MIXED_LIST_DOC: &str = crate::embedded_docs::LIST_MARKERS_FIXTURE_MD;
const HANGING_LIST_DOC: &str = concat!(
    "- top-level ordinary marker with a deliberately long sentence that must wrap ",
    "and align every continuation with the first body glyph\n",
    "  Authored continuation paragraph with a deliberately long sentence that must ",
    "also wrap and share the owning item's first prose rail\n",
    "- [ ] top-level open task with a deliberately long sentence that must wrap and ",
    "align every continuation with the first body glyph\n",
    "  - nested ordinary marker with a deliberately long sentence that must wrap and ",
    "align every continuation with the first body glyph\n",
    "  - [x] nested checked task with a deliberately long sentence that must wrap and ",
    "align every continuation with the first body glyph\n",
    "    - deep ordinary marker with a deliberately long sentence that must wrap and ",
    "align every continuation with the first body glyph\n",
    "    - [ ] deep open task with a deliberately long sentence that must wrap and ",
    "align every continuation with the first body glyph\n",
    "anchor\n",
);

fn assert_mixed_list_geometry(p: &mut TextPipeline, world: &crate::theme::Theme, dpi: f32) {
    p.set_view(&view_md(
        MIXED_LIST_DOC,
        MIXED_LIST_DOC.lines().count() - 1,
        0,
    ));

    p.visible_row_gathers.set(0);
    let marks = p.list_marks();
    assert_eq!(
        p.visible_row_gathers.get(),
        0,
        "{} dpi {dpi}: list marks use retained local rows, not a full-frame row gather",
        world.name
    );
    assert_eq!(
        marks.len(),
        10,
        "{} dpi {dpi}: fixture enrollment",
        world.name
    );
    assert_eq!(
        marks.iter().map(|mark| mark.kind).collect::<Vec<_>>(),
        [
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Task(false),
            crate::render::rects::ListLineKind::Task(true),
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Task(false),
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Task(true),
            crate::render::rects::ListLineKind::Bullet,
        ],
        "{} dpi {dpi}: tasks replace bullets, invalid stays ordinary, ordered stays out",
        world.name
    );
    assert_eq!(
        [
            marks[3].glyph,
            marks[4].glyph,
            marks[6].glyph,
            marks[8].glyph
        ],
        ['☐', '🗹', '☐', '🗹'],
        "{} dpi {dpi}: every depth uses the shared Nishiki task pair",
        world.name
    );
    let top_body_xs = [
        p.line_glyph_xs(2)[2],
        p.line_glyph_xs(3)[2],
        p.line_glyph_xs(4)[2],
        p.line_glyph_xs(5)[6],
        p.line_glyph_xs(6)[6],
    ];
    let nested_body_xs = [p.line_glyph_xs(8)[4], p.line_glyph_xs(9)[8]];
    let level_two_body_xs = [p.line_glyph_xs(10)[6], p.line_glyph_xs(11)[10]];
    for (label, body_xs) in [
        ("top", top_body_xs.as_slice()),
        ("nested", nested_body_xs.as_slice()),
        ("level two", level_two_body_xs.as_slice()),
    ] {
        let first = body_xs[0];
        assert!(
            body_xs.iter().all(|x| (*x - first).abs() < 0.51),
            "{} dpi {dpi}: {label} ordinary/task bodies share one preview start: {body_xs:?}",
            world.name
        );
    }
    for mark in &marks {
        assert!(
            (mark.paint_width - mark.slot_width).abs() < 0.51,
            "{} dpi {dpi}: every list mark paints in the one measured body gap: {mark:?}",
            world.name
        );
    }
    assert!(
        p.visual_rows(8).len() > 1 && p.visual_rows(9).len() > 1,
        "{} dpi {dpi}: ordinary and task neighbors genuinely wrap",
        world.name
    );
}

fn assert_wrapped_item_geometry(
    p: &mut TextPipeline,
    world: &crate::theme::Theme,
    dpi: f32,
    line_i: usize,
) {
    let line_text = HANGING_LIST_DOC.lines().nth(line_i).unwrap();
    let item = crate::markdown::list_item(line_text).expect("fixture list item");
    let body_col = item.content
        + usize::from(
            line_text[item.content..].starts_with("[ ] ")
                || line_text[item.content..].starts_with("[x] "),
        ) * 4;
    let rows = p.visual_rows(line_i);
    assert!(
        rows.len() > 1,
        "{} dpi {dpi} line {line_i}: fixture must genuinely wrap",
        world.name
    );
    let body_x = rows[0].xs[body_col];
    for (row_i, row) in rows.iter().enumerate().skip(1) {
        let continuation_x = row.xs[row.start_col];
        assert!(
            (continuation_x - body_x).abs() < 0.51,
            concat!("{} dpi {} line {} row {}: ", "body {} != continuation {}"),
            world.name,
            dpi,
            line_i,
            row_i,
            body_x,
            continuation_x
        );

        let visible_col = (row.start_col..row.end_col)
            .find(|col| row.xs[col + 1] - row.xs[*col] > 0.51)
            .expect("continuation has a visible glyph");
        let next_col = visible_col + 1;
        let visible_x = row.xs[visible_col];
        let next_x = row.xs[next_col];
        let px = p.text_left() + visible_x + (next_x - visible_x) * 0.25;
        let py = p.doc_top() + row.line_top + row.line_height * 0.5;
        let (hit_line, hit_col) = p.hit_test_scroll(px, py, crate::render::ScrollPos::default());
        assert_eq!(hit_line, line_i, "pointer stays on continuation paragraph");
        assert!(
            (visible_col..=next_col).contains(&hit_col),
            concat!(
                "{} dpi {} line {} row {}: ",
                "pointer col {}, expected {}..={}"
            ),
            world.name,
            dpi,
            line_i,
            row_i,
            hit_col,
            visible_col,
            next_col,
        );
        let caret_x = p.col_x_and_advance(line_i, hit_col).0;
        assert!(
            (caret_x - row.xs[hit_col]).abs() < 0.51,
            "{} dpi {dpi} line {line_i} row {row_i}: caret {caret_x} != row {}",
            world.name,
            row.xs[hit_col]
        );

        let rects = p.range_rects((line_i, visible_col), (line_i, next_col));
        let rect = rects
            .iter()
            .find(|rect| rect[1] <= py && py <= rect[1] + rect[3])
            .unwrap_or_else(|| {
                panic!(
                    concat!(
                        "{} dpi {} line {} row {}: ",
                        "range {}..{} has no rect at y={}; {:?}"
                    ),
                    world.name, dpi, line_i, row_i, visible_col, next_col, py, rects
                )
            });
        assert!(
            (rect[0] - (p.text_left() + continuation_x)).abs() < 0.51,
            "{} dpi {dpi} line {line_i} row {row_i}: range x {} != continuation x {}",
            world.name,
            rect[0],
            p.text_left() + continuation_x
        );
    }
}

#[test]
fn mixed_list_markers_share_one_preview_slot_and_body_start() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping mixed list-marker geometry law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let world = crate::theme::active();
    assert_mixed_list_geometry(&mut p, &world, 1.0);
}

/// The first prose glyph and every continuation row share one measured inset,
/// including nested bullets and tasks. The same row x-grid must answer pointer,
/// caret and range geometry; a paint-only translation would fail at least one
/// of these assertions.
#[test]
fn hanging_list_rows_share_body_pointer_caret_and_range_geometry() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping hanging list geometry law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    let entry_world = crate::theme::active().name;

    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        // Tall enough that range geometry's production visible-band cull keeps
        // all six deliberately wrapped paragraphs in this coordinate audit.
        p.set_size(520.0 * dpi, 1_600.0 * dpi);
        for world in crate::theme::THEMES.iter() {
            crate::theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            p.set_view(&view_md(HANGING_LIST_DOC, 7, 0));

            for line_i in [0usize, 2, 3, 4, 5, 6] {
                assert_wrapped_item_geometry(&mut p, world, dpi, line_i);
            }

            let parent_body_x = p.visual_rows(0)[0].xs[2];
            let continuation_rows = p.visual_rows(1);
            assert!(
                continuation_rows.len() > 1,
                "{} dpi {dpi}: authored continuation fixture must genuinely wrap",
                world.name
            );
            for (row_i, row) in continuation_rows.iter().enumerate() {
                let col = if row_i == 0 { 2 } else { row.start_col };
                assert!(
                    (row.xs[col] - parent_body_x).abs() < 0.51,
                    concat!(
                        "{} dpi {}: authored continuation row {} x={} ",
                        "!= parent body {}"
                    ),
                    world.name,
                    dpi,
                    row_i,
                    row.xs[col],
                    parent_body_x
                );
            }
            assert_eq!(
                p.buffer.lines[1].text(),
                HANGING_LIST_DOC.lines().nth(1).unwrap(),
                "layout must never mutate authored continuation bytes"
            );
        }
    }
    crate::theme::set_active_by_name(entry_world).unwrap();
}

#[test]
fn list_shaped_code_keeps_literal_indent_and_never_gets_a_hanging_inset() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping list-shaped code ownership law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let text = "```text\n  - literal code\n```\nanchor\n";
    p.set_view(&view_md(text, 3, 0));
    assert!(
        p.list_marks().is_empty(),
        "code lookalike is not a rich list item"
    );
    assert_eq!(p.buffer.lines[1].hanging_inset(), 0.0);
    let xs = p.line_glyph_xs(1);
    assert!(
        xs[2] - xs[0] > p.metrics.char_width,
        "literal code indentation must keep its visible advance: {xs:?}"
    );
}

#[test]
fn retained_list_line_loses_conceal_and_inset_when_an_inserted_fence_owns_it() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping retained list ownership law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_view(&view_md("  - item\nanchor\n", 1, 0));
    assert!(p.buffer.lines[0].hanging_inset() > 0.0);
    assert!(p.visual_rows(0)[0].xs[4] - p.visual_rows(0)[0].xs[0] < 0.51);

    let fenced = "```\n  - item\nanchor\n";
    p.set_view(&view_md(fenced, 2, 0));
    assert!(p.list_marks().is_empty());
    assert_eq!(p.buffer.lines[1].text(), "  - item");
    assert_eq!(p.buffer.lines[1].hanging_inset(), 0.0);
    let xs = p.line_glyph_xs(1);
    assert!(
        xs[4] - xs[0] > p.metrics.char_width * 2.5,
        "fenced literal keeps its source indent and marker advances: {xs:?}"
    );
}

#[test]
fn authored_list_continuation_reveals_for_caret_and_either_selection_direction() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping authored continuation reveal law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_size(520.0, 1_200.0);

    let hidden = view_md(HANGING_LIST_DOC, 7, 0);
    p.set_view(&hidden);
    assert!(p.buffer.lines[1].hanging_inset() > 0.0);
    assert!(
        p.visual_rows(1)[0].xs[2] - p.visual_rows(1)[0].xs[0] < 0.51,
        "preview collapses only the parser-owned source indentation"
    );

    let mut caret = view_md(HANGING_LIST_DOC, 1, 3);
    p.set_view(&caret);
    assert_eq!(p.buffer.lines[1].hanging_inset(), 0.0);
    assert!(p.visual_rows(1)[0].xs[2] - p.visual_rows(1)[0].xs[0] > 1.0);

    for selection in [Some(((1, 3), (1, 8))), Some(((1, 8), (1, 3)))] {
        caret.cursor_line = 7;
        caret.cursor_col = 0;
        caret.selection = selection;
        p.set_view(&caret);
        assert_eq!(
            p.buffer.lines[1].hanging_inset(),
            0.0,
            "either selection direction reveals raw continuation layout"
        );
        assert!(p.visual_rows(1)[0].xs[2] - p.visual_rows(1)[0].xs[0] > 1.0);
    }

    p.set_view(&hidden);
    assert!(p.buffer.lines[1].hanging_inset() > 0.0);
    assert_eq!(
        p.buffer.lines[1].text(),
        HANGING_LIST_DOC.lines().nth(1).unwrap(),
        "reveal/reconceal leaves source bytes unchanged"
    );
}

#[test]
fn lazy_list_continuation_reconciles_inset_for_caret_and_either_selection_direction() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping lazy continuation reveal law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    let text = "- parent\nlazy continuation\nafter\n";
    let hidden = view_md(text, 2, 0);
    p.set_view(&hidden);
    assert!(p.buffer.lines[1].hanging_inset() > 0.0);

    let mut reveal = view_md(text, 1, 3);
    p.set_view(&reveal);
    assert_eq!(p.buffer.lines[1].hanging_inset(), 0.0);

    for selection in [Some(((1, 0), (1, 4))), Some(((1, 4), (1, 0)))] {
        reveal.cursor_line = 2;
        reveal.cursor_col = 0;
        reveal.selection = selection;
        p.set_view(&reveal);
        assert_eq!(
            p.buffer.lines[1].hanging_inset(),
            0.0,
            "either selection direction reveals a zero-indent continuation"
        );
    }

    p.set_view(&hidden);
    assert!(p.buffer.lines[1].hanging_inset() > 0.0);
    assert_eq!(p.buffer.lines[1].text(), "lazy continuation");
}

fn isolated_marker_masks(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    marks: &[crate::render::rects::ListMark],
) -> (MarkerMasks, Frame) {
    p.prepare(device, queue, width, height).unwrap();
    let with = pixeldiff::render_frame(p, device, queue, width, height);
    p.md_enabled = false;
    p.prepare_ornaments(device, queue, width, height).unwrap();
    let without = pixeldiff::render_frame(p, device, queue, width, height);
    p.md_enabled = true;

    let masks = marks
        .iter()
        .map(|mark| {
            let origin_x = mark.left.floor() as i32;
            let x0 = (mark.left - p.metrics.char_width).floor().max(0.0) as i32;
            let x1 = (mark.left + mark.slot_width + p.metrics.char_width)
                .ceil()
                .min(width as f32) as i32;
            let y0 = mark.top.floor().max(0.0) as i32;
            let y1 = (mark.top + p.metrics.line_height).ceil().min(height as f32) as i32;
            let mut mask = BTreeSet::new();
            for y in y0..y1 {
                for x in x0..x1 {
                    let idx = (y as u32 * width + x as u32) as usize;
                    if with[idx] != without[idx] {
                        mask.insert((x - origin_x, y - y0));
                    }
                }
            }
            mask
        })
        .collect();
    (masks, with)
}

fn first_ink_column(
    pixels: &[Pixel],
    width: u32,
    height: u32,
    rect: [i32; 4],
    ground: Pixel,
) -> Option<i32> {
    let [x0, y0, x1, y1] = rect;
    (x0.max(0)..x1.min(width as i32)).find(|&x| {
        (y0.max(0)..y1.min(height as i32)).any(|y| {
            let pixel = pixels[(y as u32 * width + x as u32) as usize];
            (0..3)
                .any(|channel| (i16::from(pixel[channel]) - i16::from(ground[channel])).abs() > 18)
        })
    })
}

fn ink_row_bounds(
    pixels: &[Pixel],
    width: u32,
    height: u32,
    rect: [i32; 4],
    ground: Pixel,
) -> Option<(i32, i32)> {
    let [x0, y0, x1, y1] = rect;
    let mut rows = (y0.max(0)..y1.min(height as i32)).filter(|&y| {
        (x0.max(0)..x1.min(width as i32)).any(|x| {
            let pixel = pixels[(y as u32 * width + x as u32) as usize];
            (0..3)
                .any(|channel| (i16::from(pixel[channel]) - i16::from(ground[channel])).abs() > 18)
        })
    });
    let first = rows.next()?;
    Some((first, rows.next_back().unwrap_or(first)))
}

fn assert_marker_enrolment(
    world: &crate::theme::Theme,
    dpi: f32,
    marks: &[crate::render::rects::ListMark],
) {
    assert_eq!(
        marks.len(),
        8,
        "{} dpi {dpi}: one mark per list row",
        world.name
    );
    let kinds: Vec<_> = marks.iter().map(|mark| mark.kind).collect();
    assert_eq!(
        kinds,
        [
            crate::render::rects::ListLineKind::Task(false),
            crate::render::rects::ListLineKind::Task(true),
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Bullet,
            crate::render::rects::ListLineKind::Task(false),
            crate::render::rects::ListLineKind::Task(true),
        ],
        "{} dpi {dpi}: valid tasks replace bullets; `- []` remains an ordinary bullet",
        world.name
    );
    assert_eq!(marks[0].glyph, crate::theme::task_marker(false));
    assert_eq!(marks[1].glyph, crate::theme::task_marker(true));
    assert_eq!(marks[2].glyph, world.bullets.0);
    assert_eq!(
        marks[4].glyph, world.bullets.1,
        "{} dpi {dpi}: {marks:?}",
        world.name
    );
    assert_eq!(marks[5].glyph, world.bullets.2);
}

fn task_body_positions(
    p: &TextPipeline,
    world: &crate::theme::Theme,
    dpi: f32,
    marks: &[crate::render::rects::ListMark],
) -> [f32; 4] {
    let mut body_xs = [0.0; 4];
    for (index, (line, marker_col, mark)) in [
        (0usize, 0usize, &marks[0]),
        (1, 0, &marks[1]),
        (6, 2, &marks[6]),
        (7, 4, &marks[7]),
    ]
    .into_iter()
    .enumerate()
    {
        let body_col = marker_col + 6;
        let xs = p.line_glyph_xs(line);
        let marker_x = p.text_left() + xs[marker_col];
        let body_x = p.text_left() + xs[body_col];
        assert!(
            (marker_x - body_x).abs() < 0.51,
            "{} dpi {dpi} line {line}: concealed source prefix collapses onto the body rail",
            world.name
        );
        assert!(
            (xs[marker_col + 5] - xs[marker_col + 2]).abs() < 0.51,
            "{} dpi {dpi} line {line}: checkbox bytes collapse to zero advance",
            world.name,
        );
        assert!(
            (xs[body_col] - xs[marker_col + 2]).abs() < 0.51,
            "{} dpi {dpi} line {line}: checkbox and task-only separator collapse together",
            world.name,
        );
        assert!(
            (body_x - (mark.left + mark.paint_width)).abs() < 0.51,
            "{} dpi {dpi} line {line}: task body follows the shared marker slot: \
             left={} width={} body={body_x}",
            world.name,
            mark.left,
            mark.paint_width
        );
        body_xs[index] = body_x;
    }
    body_xs
}

fn assert_task_marker_pixels(
    p: &mut TextPipeline,
    target: RenderTarget<'_>,
    world: &crate::theme::Theme,
    dpi: f32,
    marks: &[crate::render::rects::ListMark],
    body_xs: [f32; 4],
) {
    let (masks, frame) = isolated_marker_masks(
        p,
        target.device,
        target.queue,
        target.width,
        target.height,
        marks,
    );
    assert!(
        masks[0].len() >= 4 && masks[1].len() >= 4,
        "{} dpi {dpi}: both task states paint real marker pixels: open={} checked={}",
        world.name,
        masks[0].len(),
        masks[1].len()
    );
    assert_ne!(
        masks[0], masks[1],
        "{} dpi {dpi}: open and checked task pixels must differ",
        world.name
    );
    assert_marker_seating(p, &target, world, marks, &masks, &frame, body_xs);
    for (state, mark, mask, body_x) in [
        ("open", &marks[0], &masks[0], body_xs[0]),
        ("checked", &marks[1], &masks[1], body_xs[1]),
    ] {
        let left = mask.iter().map(|(x, _)| *x).min().unwrap();
        let right = mask.iter().map(|(x, _)| *x).max().unwrap();
        assert!(
            left >= 0 && right < mark.paint_width.ceil() as i32,
            "{} dpi {dpi}: {state} marker ink [{left},{right}] must stay inside its \
             source-derived {:.1}px paint slot",
            world.name,
            mark.paint_width
        );
        let row_y0 = mark.top.floor() as i32;
        let row_y1 = (mark.top + p.metrics.line_height).ceil() as i32;
        let body_ink = first_ink_column(
            &frame,
            target.width,
            target.height,
            [
                body_x.floor() as i32,
                row_y0,
                (body_x + 120.0 * dpi).ceil() as i32,
                row_y1,
            ],
            world.base_100.rgba_bytes(),
        )
        .expect("task body paints real ink");
        let marker_right = mark.left.floor() as i32 + right;
        assert!(
            body_ink > marker_right + 1,
            "{} dpi {dpi}: {state} marker ink [{left},{right}] at x={}..={marker_right}, \
             body ink starts at x={body_ink}; a blank pixel column must separate them",
            world.name,
            mark.left.floor() as i32 + left,
        );
    }
}

fn assert_marker_seating(
    p: &TextPipeline,
    target: &RenderTarget<'_>,
    world: &crate::theme::Theme,
    marks: &[crate::render::rects::ListMark],
    masks: &[MarkerMask],
    frame: &[Pixel],
    body_xs: [f32; 4],
) {
    let dpi = p.dpi;
    let body_by_mark = [
        body_xs[0],
        body_xs[1],
        p.text_left() + p.line_glyph_xs(2)[2],
        p.text_left() + p.line_glyph_xs(3)[2],
        p.text_left() + p.line_glyph_xs(4)[4],
        p.text_left() + p.line_glyph_xs(5)[6],
        body_xs[2],
        body_xs[3],
    ];
    for ((mark, mask), body_x) in marks.iter().zip(masks).zip(body_by_mark) {
        let marker_right = mask.iter().map(|(x, _)| *x).max().unwrap();
        let marker_top = mask.iter().map(|(_, y)| *y).min().unwrap();
        let marker_bottom = mask.iter().map(|(_, y)| *y).max().unwrap();
        let row_y0 = mark.top.floor() as i32;
        let row_y1 = (mark.top + p.metrics.line_height).ceil() as i32;
        let (body_top, body_bottom) = ink_row_bounds(
            frame,
            target.width,
            target.height,
            [
                body_x.floor() as i32,
                row_y0,
                (body_x + 120.0 * dpi) as i32,
                row_y1,
            ],
            world.base_100.rgba_bytes(),
        )
        .expect("list body paints real ink");
        let marker_center_y = row_y0 as f32 + (marker_top + marker_bottom) as f32 * 0.5;
        let body_center_y = (body_top + body_bottom) as f32 * 0.5;
        assert!(
            (marker_center_y - body_center_y).abs() <= p.metrics.line_height * 0.20,
            "{} dpi {dpi}: {:?} vertical seat: marker={marker_center_y} body={body_center_y}",
            world.name,
            mark.kind
        );
        let body_ink = first_ink_column(
            frame,
            target.width,
            target.height,
            [
                body_x.floor() as i32,
                row_y0,
                (body_x + 120.0 * dpi) as i32,
                row_y1,
            ],
            world.base_100.rgba_bytes(),
        )
        .expect("list body paints real ink");
        let marker_ink_right = mark.left.floor() as i32 + marker_right;
        let gap = body_ink - marker_ink_right - 1;
        assert!(
            gap as f32 >= p.metrics.font_size * 0.30,
            "{} dpi {dpi}: {:?} marker-to-prose gap {gap}px is below 0.30em ({:.1}px); \
             marker right={marker_ink_right}, body ink={body_ink}",
            world.name,
            mark.kind,
            p.metrics.font_size * 0.30,
        );
    }
}

#[test]
fn every_world_and_dpi_paints_distinct_task_state_clear_of_the_body() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1440.0, 920.0) else {
        eprintln!("skipping task marker world/DPI pixels: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);

    for &dpi in &[1.0f32, 2.0] {
        let width = (LOGICAL_W as f32 * dpi) as u32;
        let height = (LOGICAL_H as f32 * dpi) as u32;
        p.set_size(width as f32, height as f32);
        p.set_dpi(dpi);
        for world in crate::theme::THEMES.iter() {
            crate::theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            p.set_view(&view_md(DOC, 8, 0));

            let marks = p.list_marks();
            assert_marker_enrolment(world, dpi, &marks);
            let body_xs = task_body_positions(&p, world, dpi, &marks);
            assert!(
                p.visual_rows(6).len() > 1,
                "{} dpi {dpi}: the nested task fixture genuinely wraps",
                world.name
            );
            assert_task_marker_pixels(
                &mut p,
                RenderTarget {
                    device: &device,
                    queue: &queue,
                    width,
                    height,
                },
                world,
                dpi,
                &marks,
                body_xs,
            );
            assert_mixed_list_geometry(&mut p, world, dpi);
        }
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

fn hit_row(p: &TextPipeline, line: usize, row_index: usize, x: f32) -> (usize, usize) {
    let row = &p.visual_rows(line)[row_index];
    let py = p.doc_top() + row.line_top + row.line_height * 0.5;
    p.hit_test_scroll(x, py, crate::render::ScrollPos::default())
}

fn assert_raw_task_prefix_hits(p: &TextPipeline, line: usize, reveal: &str) {
    let raw = p.line_glyph_xs(line);
    for col in 0..6 {
        let px = p.text_left() + raw[col] + (raw[col + 1] - raw[col]) * 0.25;
        assert_eq!(
            hit_row(p, line, 0, px),
            (line, col),
            "{reveal} returns task-prefix hit testing to raw source column {col}"
        );
    }
}

#[test]
fn task_conceal_collapses_its_separator_and_hit_tests_to_source_columns() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping task conceal and hit-test law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_view(&view_md(DOC, 8, 0));

    for byte in [0usize, 2, 3, 4] {
        let attrs = p.buffer.lines[0].attrs_list().get_span(byte);
        assert_eq!(
            attrs.color_opt.map(|color| color.a()),
            Some(0),
            "off-caret task source byte {byte} must be transparent beneath its one state marker"
        );
    }
    assert_eq!(
        p.buffer.lines[0]
            .attrs_list()
            .get_span(5)
            .color_opt
            .map(|color| color.a()),
        Some(0),
        "the task-only separator conceals with the checkbox"
    );

    for line in [0usize, 1, 6, 7] {
        let marker_col = if line < 2 {
            0
        } else if line == 6 {
            2
        } else {
            4
        };
        let xs = p.line_glyph_xs(line);
        assert!(
            (xs[marker_col + 5] - xs[marker_col + 2]).abs() < 0.51,
            "line {line}: `[ ]`/`[x]` source collapses off-caret: {xs:?}"
        );
        assert!(
            (xs[marker_col + 6] - xs[marker_col + 5]).abs() < 0.51,
            "line {line}: parser-owned task separator collapses off-caret: {xs:?}"
        );
    }

    let invalid = p.line_glyph_xs(2);
    assert!(
        invalid[5] - invalid[2] > 1.0,
        "invalid `[]` remains visible rather than joining task conceal: {invalid:?}"
    );

    let list_lines = p.list_lines_snapshot();
    let marks = p.list_marks();
    for (line, expected) in [
        (0usize, [0usize, 0, 3, 6]),
        (3usize, [0usize, 0, 1, 2]),
        (6usize, [0usize, 2, 5, 8]),
    ] {
        let item_index = list_lines
            .iter()
            .position(|item| item.line == line)
            .expect("fixture list line enrolled");
        let mark = marks[item_index];
        let points = [
            p.text_left(),
            mark.left,
            mark.left + mark.slot_width * 0.5,
            mark.left + mark.slot_width,
        ];
        for (point, want) in points.into_iter().zip(expected) {
            assert_eq!(
                hit_row(&p, line, 0, point),
                (line, want),
                "line {line}: preview indent/marker/body edge owns source column {want}"
            );
        }
    }
    let wrapped = p.visual_rows(6);
    assert!(wrapped.len() > 1, "nested task fixture genuinely wraps");
    let second = &wrapped[1];
    let nested_index = list_lines
        .iter()
        .position(|item| item.line == 6)
        .expect("wrapped task enrolled");
    let nested_mark = marks[nested_index];
    let second_py = p.doc_top() + second.line_top + second.line_height * 0.5;
    assert_eq!(
        p.hit_test_scroll(
            nested_mark.left + nested_mark.slot_width * 0.5,
            second_py,
            crate::render::ScrollPos::default(),
        ),
        (6, second.start_col),
        "the marker rail belongs only to row one; the same x on row two maps to its wrapped start"
    );
}

#[test]
fn task_prefix_hits_return_to_raw_source_for_caret_and_selection_reveal() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping raw task-prefix hit-test law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    p.set_view(&view_md(DOC, 0, 0));
    let revealed = p.line_glyph_xs(0);
    assert!(
        revealed[5] - revealed[2] > 1.0,
        "caret reveal restores the raw checkbox's full advance: {revealed:?}"
    );
    for byte in [0usize, 2, 3, 4] {
        assert!(
            p.buffer.lines[0]
                .attrs_list()
                .get_span(byte)
                .color_opt
                .is_none_or(|color| color.a() != 0),
            "caret reveal restores raw task byte {byte}"
        );
    }
    assert_raw_task_prefix_hits(&p, 0, "caret reveal");

    let mut selected = view_md(DOC, 8, 0);
    selected.selection = Some(((1, 0), (1, 18)));
    p.set_view(&selected);
    let selected_xs = p.line_glyph_xs(1);
    assert!(
        selected_xs[5] - selected_xs[2] > 1.0,
        "selection reveal restores the raw checkbox's full advance: {selected_xs:?}"
    );
    for byte in [0usize, 2, 3, 4] {
        assert!(
            p.buffer.lines[1]
                .attrs_list()
                .get_span(byte)
                .color_opt
                .is_none_or(|color| color.a() != 0),
            "selection reveal restores raw task byte {byte}"
        );
    }
    assert_raw_task_prefix_hits(&p, 1, "forward-selection reveal");

    selected.selection = Some(((1, 6), (1, 0)));
    p.set_view(&selected);
    assert_eq!(
        p.buffer.lines[1].hanging_inset(),
        0.0,
        "reverse selection also reveals raw task geometry"
    );
    assert_raw_task_prefix_hits(&p, 1, "reverse-selection reveal");
}

#[test]
fn list_prefix_hit_mapping_preserves_rtl_and_mixed_script_rows() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping bidi list-prefix hit-test law: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    p.sync_theme();
    const BIDI_DOC: &str = "  - שלום עולם\n- English שלום\nanchor\n";
    p.set_view(&view_md(BIDI_DOC, 2, 0));
    let rtl_row = &p.visual_rows(0)[0];
    let rtl_body_x = rtl_row.xs[4];
    let (prose_x, prose_col, guard_x, guard_col) = {
        let run = p
            .buffer
            .layout_runs()
            .find(|run| run.line_i == 0)
            .expect("RTL list row shaped");
        assert!(run.rtl, "Hebrew list paragraph exercises the RTL fallback");
        let (prose_x, prose_col) = run
            .glyphs
            .iter()
            .filter(|glyph| glyph.start >= 4 && glyph.w > 1.0)
            .map(|glyph| glyph.x + glyph.w * 0.5)
            .map(|target_x| (target_x, p.col_in_run(&run, target_x)))
            .next()
            .expect("RTL prose exposes a visible shaped glyph after its list prefix");
        let (guard_x, guard_col) = (0..=16)
            .map(|step| rtl_body_x.max(0.0) * step as f32 / 16.0)
            .find_map(|target_x| {
                let native = p.col_in_run(&run, target_x);
                let forced = p.concealed_list_prefix_hit_col(0, true, true, target_x);
                (forced.is_some() && forced != Some(native)).then_some((target_x, native))
            })
            .expect("forcing LTR projection across RTL changes a native prefix-rail hit");
        (prose_x, prose_col, guard_x, guard_col)
    };
    assert_eq!(
        p.concealed_list_prefix_hit_col(0, true, false, guard_x),
        None,
        "RTL paragraphs explicitly decline the LTR prefix projection"
    );
    let rtl_py = p.doc_top() + rtl_row.line_top + rtl_row.line_height * 0.5;
    for (target_x, expected, label) in [
        (prose_x, prose_col, "visible prose"),
        (guard_x, guard_col, "prefix rail"),
    ] {
        assert_eq!(
            p.hit_test_scroll(
                p.text_left() + target_x,
                rtl_py,
                crate::render::ScrollPos::default(),
            ),
            (0, expected),
            "RTL list {label} retains native shaped hit testing"
        );
    }

    let mixed_index = p
        .list_lines_snapshot()
        .iter()
        .position(|item| item.line == 1)
        .expect("mixed-script LTR row enrolled");
    let mixed_mark = p.list_marks()[mixed_index];
    assert_eq!(
        hit_row(&p, 1, 0, mixed_mark.left + mixed_mark.slot_width * 0.5,),
        (1, 1),
        "mixed-script LTR list marker keeps normalized prefix mapping"
    );
}

#[test]
fn task_marker_reveal_transitions_remove_only_the_touched_ornament_and_keep_source() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping task marker reveal transitions: no wgpu adapter");
        return;
    };
    crate::markdown::set_wysiwyg_on(true);

    p.set_view(&view_md(DOC, 8, 0));
    assert_eq!(
        p.list_marks().len(),
        8,
        "settled preview enrolls every list row"
    );

    p.set_view(&view_md(DOC, 0, 0));
    assert_eq!(
        p.list_marks().len(),
        7,
        "caret reveal removes the open-task ornament"
    );
    assert_eq!(
        p.buffer.lines[0].text(),
        "- [ ] open task",
        "source remains byte-exact"
    );

    let mut selected = view_md(DOC, 8, 0);
    selected.selection = Some(((1, 0), (1, 18)));
    p.set_view(&selected);
    assert_eq!(
        p.list_marks().len(),
        7,
        "selection reveal removes the checked-task ornament"
    );
    assert_eq!(
        p.buffer.lines[1].text(),
        "- [x] checked task",
        "checked source remains byte-exact"
    );

    p.set_view(&view_md(DOC, 8, 0));
    assert_eq!(
        p.list_marks().len(),
        8,
        "clearing selection restores the marker without moving the caret line"
    );
}
