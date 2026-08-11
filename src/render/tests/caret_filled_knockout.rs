//! Cassowary's Filled caret is a lit cell with the covered glyph knocked back in
//! the ground colour. Its knockout retains true source weight; ordinary Morph
//! keeps the deliberately bolder, hard-dilated silhouette.

use super::super::*;
use super::{headless_dqp, pixeldiff};

const W: u32 = 420;
const H: u32 = 240;
const _: () = assert!(CARET_MORPH_DILATE_PX.0 > CaretGlyphPipeline::FILLED_KNOCKOUT_DILATE_PX);

#[derive(Clone, Copy)]
struct Cell {
    name: &'static str,
    text: &'static str,
    col: usize,
    look: CaretMode,
    has_glyph: bool,
    code: bool,
}

const CELLS: [Cell; 8] = [
    Cell {
        name: "round",
        text: "o",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: false,
    },
    Cell {
        name: "stem",
        text: "l",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: false,
    },
    Cell {
        name: "bowl",
        text: "b",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: false,
    },
    Cell {
        name: "descender",
        text: "g",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: false,
    },
    Cell {
        name: "punctuation",
        text: "?",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: false,
    },
    Cell {
        name: "space",
        text: " ",
        col: 0,
        look: CaretMode::Block,
        has_glyph: false,
        code: false,
    },
    // `calt` substitutes shapes while retaining one glyph per fixed-pitch cell.
    Cell {
        name: "ligature",
        text: "=>",
        col: 0,
        look: CaretMode::Block,
        has_glyph: true,
        code: true,
    },
    // A REQUESTED Morph folds to Block WHOLESALE on this world — body, glyph mask
    // and anchor alike — so this cell must be indistinguishable from `round`, whose
    // text and column it deliberately repeats. It once declined the mask (and stepped
    // its anchor a column back) while painting as a Block: a hybrid belonging to
    // neither mode, which put the caret a column left of its insertion point on every
    // ink-caret world.
    Cell {
        name: "folded-morph",
        text: "o",
        col: 0,
        look: CaretMode::Morph,
        has_glyph: true,
        code: false,
    },
];

struct PreparedView<'a> {
    text: &'a str,
    line: usize,
    col: usize,
    look: CaretMode,
    zoom: f32,
    code: bool,
}

fn prepare(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    prepared: PreparedView<'_>,
) {
    crate::caret::set_mode(prepared.look);
    let mut v = super::view(prepared.text, prepared.line, prepared.col);
    v.zoom = prepared.zoom;
    if prepared.code {
        v.syn_lang = Some(crate::syntax::Lang::Rust);
    }
    p.set_view(&v);
    p.settle_caret();
    p.prepare(device, queue, W, H).unwrap();
}

fn render_glyph_only(
    glyph: &CaretGlyphPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Vec<[u8; 4]> {
    let (texture, view) = super::dither::offscreen(device, W, H);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("filled knockout source-mask encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("filled knockout source-mask pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        glyph.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, W, H)
}

fn differing(a: &[[u8; 4]], b: &[[u8; 4]], i: usize, floor: u8) -> bool {
    (0..3).any(|c| a[i][c].abs_diff(b[i][c]) >= floor)
}

fn near(mask: &[bool], x: i32, y: i32, radius: i32) -> bool {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let xx = x + dx;
            let yy = y + dy;
            if xx >= 0
                && yy >= 0
                && xx < W as i32
                && yy < H as i32
                && mask[(yy as u32 * W + xx as u32) as usize]
            {
                return true;
            }
        }
    }
    false
}

fn assert_ligature_substitution(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text: &str,
    cell: Cell,
    zoom: f32,
    what: &str,
) {
    let ligature_pixels = pixeldiff::render_frame(p, device, queue, W, H);
    crate::render::set_code_ligatures_on(false);
    let perturb = format!("{text} ");
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text: &perturb,
            line: 0,
            col: cell.col,
            look: cell.look,
            zoom,
            code: false,
        },
    );
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text,
            line: 0,
            col: cell.col,
            look: cell.look,
            zoom,
            code: true,
        },
    );
    let plain_pixels = pixeldiff::render_frame(p, device, queue, W, H);
    let substitution = pixeldiff::diff_region(
        &ligature_pixels,
        &plain_pixels,
        W as i64,
        H as i64,
        pixeldiff::Region::canvas(W as i64, H as i64),
    );
    assert!(
        substitution.differing >= 2 && substitution.max_channel_delta >= 5,
        "{what}: code-ligature forcing must visibly substitute Iosevka's => glyphs; \
         got {substitution:?}"
    );

    crate::render::set_code_ligatures_on(true);
    let perturb = format!("{text}  ");
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text: &perturb,
            line: 0,
            col: cell.col,
            look: cell.look,
            zoom,
            code: false,
        },
    );
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text,
            line: 0,
            col: cell.col,
            look: cell.look,
            zoom,
            code: true,
        },
    );
}

fn assert_true_weight_mask(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    actual: &[[u8; 4]],
    what: &str,
) {
    // A block-only diff isolates the knockout without assuming world colours.
    p.caret_glyph_pipeline.clear();
    let block_only = pixeldiff::render_frame(p, device, queue, W, H);

    // Independently render the same uploaded source mask white on black with
    // zero dilation: this is the true-weight pixel oracle.
    let (from_box, to_box, morph_t) = p.caret_glyph_geometry();
    p.caret_glyph_pipeline.set_color([255, 255, 255]);
    p.caret_glyph_pipeline.prepare(
        device,
        queue,
        W,
        H,
        p.caret_mask_from.as_ref(),
        from_box,
        p.caret_mask_to.as_ref(),
        to_box,
        morph_t,
        1.0,
        0.0,
    );
    let source = render_glyph_only(&p.caret_glyph_pipeline, device, queue);

    let n = (W * H) as usize;
    let source_mask: Vec<bool> = (0..n)
        .map(|i| source[i][0] >= 16 || source[i][1] >= 16 || source[i][2] >= 16)
        .collect();
    let knockout_mask: Vec<bool> = (0..n)
        .map(|i| differing(actual, &block_only, i, 2))
        .collect();
    let source_n = source_mask.iter().filter(|&&on| on).count();
    let knockout_n = knockout_mask.iter().filter(|&&on| on).count();
    assert!(
        source_n >= 4 && knockout_n >= 4,
        "{what}: empty mask populations"
    );

    let mut knockout_outside_source = 0usize;
    let mut source_without_knockout = 0usize;
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let i = (y as u32 * W + x as u32) as usize;
            if knockout_mask[i] && !near(&source_mask, x, y, 1) {
                knockout_outside_source += 1;
            }
            if source_mask[i] && !near(&knockout_mask, x, y, 1) {
                source_without_knockout += 1;
            }
        }
    }
    assert_eq!(
        knockout_outside_source, 0,
        "{what}: knockout expanded beyond the source glyph's one-pixel AA allowance \
         (source={source_n}, knockout={knockout_n}, outside={knockout_outside_source})"
    );
    assert_eq!(
        source_without_knockout, 0,
        "{what}: source glyph lost from the Filled cell \
         (source={source_n}, knockout={knockout_n}, missing={source_without_knockout})"
    );
}

fn exercise_cell(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cell: Cell,
    zoom: f32,
    dpi: f32,
) -> bool {
    let text = format!("{}\npark", cell.text);
    let what = format!("cell={} zoom={zoom} dpi={dpi}", cell.name);

    // Parking on row 1 is the appearance baseline for the cell body's presence.
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text: &text,
            line: 1,
            col: 0,
            look: CaretMode::Block,
            zoom,
            code: cell.code,
        },
    );
    let parked = pixeldiff::render_frame(p, device, queue, W, H);
    prepare(
        p,
        device,
        queue,
        PreparedView {
            text: &text,
            line: 0,
            col: cell.col,
            look: cell.look,
            zoom,
            code: cell.code,
        },
    );
    if cell.name == "ligature" {
        assert_ligature_substitution(p, device, queue, &text, cell, zoom, &what);
    }

    let (cx, cy, cw, ch, ..) = p.caret_geometry();
    let actual = pixeldiff::render_frame(p, device, queue, W, H);
    let body = pixeldiff::diff_region(
        &actual,
        &parked,
        W as i64,
        H as i64,
        pixeldiff::Region::new(cx - cw, cy - ch, cw * 2.0, ch * 2.0),
    );
    assert!(
        body.differing >= 8 && body.max_channel_delta >= 20,
        "{what}: the filled cell must remain visibly present, got {body:?}"
    );

    let drew = p.caret_glyph_pipeline.is_drawn();
    assert_eq!(drew, cell.has_glyph, "{what}: glyph-mask enrolment drift");
    if drew {
        assert_true_weight_mask(p, device, queue, &actual, &what);
    }
    drew
}

/// Compare the composited knockout to the glyph pipeline's own zero-dilation
/// source mask. A one-device-pixel allowance admits only raster/composite AA
/// quantisation; restoring Morph's 2-logical-pixel dilation creates a real ring
/// outside that allowance and fails by name. The source mask and the Filled
/// frame are both real GPU pixels, never a geometry or instance-count proxy.
#[test]
fn cassowary_filled_knockout_keeps_source_weight_across_cells_zoom_and_dpi() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Filled knockout pixel law: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Cassowary").unwrap();
    p.sync_theme();
    let saved_ligatures = crate::render::code_ligatures_on();
    crate::render::set_code_ligatures_on(true);

    assert_eq!(CaretGlyphPipeline::FILLED_KNOCKOUT_DILATE_PX, 0.0);

    let mut glyph_cells = 0usize;
    let mut glyphless_cells = 0usize;
    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        for zoom in [0.8_f32, 1.0, 2.0] {
            for cell in CELLS {
                if exercise_cell(&mut p, &device, &queue, cell, zoom, dpi) {
                    glyph_cells += 1;
                } else {
                    glyphless_cells += 1;
                }
            }
        }
    }
    assert_eq!(
        glyph_cells,
        7 * 3 * 2,
        "seven inhabited classes x zoom x dpi"
    );
    assert_eq!(
        glyphless_cells,
        3 * 2,
        "space census drift: the space is the ONLY glyphless cell"
    );

    p.set_dpi(1.0);
    crate::render::set_code_ligatures_on(saved_ligatures);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// Ordinary Morph is re-uploaded through an independent explicit reference call
/// using `CARET_MORPH_DILATE_PX`; the whole frame must remain byte-identical at
/// every requested zoom/DPI pair. This catches an over-broad "make every glyph
/// mask true-weight" repair even if Cassowary itself looks correct.
#[test]
fn ordinary_morph_keeps_its_dilated_pixels_byte_identical() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping ordinary Morph identity law: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();

    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        for zoom in [0.8_f32, 1.0, 2.0] {
            let text = "xo\npark";
            // Morph inhabits the character immediately before the insertion point.
            prepare(
                &mut p,
                &device,
                &queue,
                PreparedView {
                    text,
                    line: 0,
                    col: 2,
                    look: CaretMode::Morph,
                    zoom,
                    code: false,
                },
            );
            assert!(
                p.caret_glyph_pipeline.is_drawn(),
                "zoom={zoom} dpi={dpi}: fixture mask"
            );
            let actual = pixeldiff::render_frame(&mut p, &device, &queue, W, H);

            let (from_box, to_box, morph_t) = p.caret_glyph_geometry();
            p.caret_glyph_pipeline
                .set_color(theme::primary().rgb_bytes());
            p.caret_glyph_pipeline.prepare(
                &device,
                &queue,
                W,
                H,
                p.caret_mask_from.as_ref(),
                from_box,
                p.caret_mask_to.as_ref(),
                to_box,
                morph_t,
                1.0,
                p.metrics.px(CARET_MORPH_DILATE_PX),
            );
            let reference = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
            assert_eq!(
                actual, reference,
                "zoom={zoom} dpi={dpi}: ordinary Morph bytes changed"
            );
        }
    }

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}
