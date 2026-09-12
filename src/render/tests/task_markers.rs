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
    assert_eq!(marks[0].glyphs, world.task_marker.glyphs(false));
    assert_eq!(marks[1].glyphs, world.task_marker.glyphs(true));
    assert_eq!(
        marks[2].glyphs,
        crate::theme::TaskMarkerGlyphs::Single(world.bullets.0)
    );
    assert_eq!(
        marks[4].glyphs,
        crate::theme::TaskMarkerGlyphs::Single(world.bullets.1),
        "{} dpi {dpi}: {marks:?}",
        world.name
    );
    assert_eq!(
        marks[5].glyphs,
        crate::theme::TaskMarkerGlyphs::Single(world.bullets.2)
    );
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
        assert!((mark.left - marker_x).abs() < 0.01);
        assert!(
            (xs[marker_col + 5] - xs[marker_col + 2]).abs() < 0.51,
            "{} dpi {dpi} line {line}: checkbox bytes collapse to zero advance",
            world.name,
        );
        assert!(
            xs[body_col] - xs[marker_col + 5] > 1.0,
            "{} dpi {dpi} line {line}: the source separator retains positive advance",
            world.name,
        );
        assert!(
            (body_x - (mark.left + mark.paint_width)).abs() < 0.51,
            "{} dpi {dpi} line {line}: retained separator follows the marker slot: \
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
    if matches!(world.task_marker, crate::theme::TaskMarkerStyle::Rounded) {
        assert!(
            masks[1].difference(&masks[0]).next().is_some(),
            "{} dpi {dpi}: the rounded checked overlay adds visible tick ink",
            world.name
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
        }
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn task_conceal_retains_its_separator_and_hit_tests_to_source_columns() {
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
    assert!(
        p.buffer.lines[0]
            .attrs_list()
            .get_span(5)
            .color_opt
            .is_none_or(|color| color.a() != 0),
        "the retained task separator stays transparent-free"
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
            xs[marker_col + 6] - xs[marker_col + 5] > 1.0,
            "line {line}: parser-owned separator keeps real advance: {xs:?}"
        );
    }

    let invalid = p.line_glyph_xs(2);
    assert!(
        invalid[5] - invalid[2] > 1.0,
        "invalid `[]` remains visible rather than joining task conceal: {invalid:?}"
    );

    let body_x = p.text_left() + p.line_glyph_xs(0)[6];
    let py = p.doc_top() + p.metrics.line_height * 0.5;
    let mut cols = BTreeSet::new();
    let mut px = p.text_left() - 2.0;
    while px <= body_x {
        let (line, col) = p.hit_test_scroll(px, py, crate::render::ScrollPos::default());
        assert_eq!(line, 0, "task-prefix click must stay on its own row");
        assert!(col <= DOC.lines().next().unwrap().chars().count());
        cols.insert(col);
        px += 0.5;
    }
    assert!(
        cols.len() > 1 && cols.iter().all(|col| *col <= 6),
        "task-prefix clicks map to valid source-prefix columns: {cols:?}"
    );

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
