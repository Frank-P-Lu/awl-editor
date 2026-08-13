//! THE WORKSPACE RESERVES ITS TEACHING FOOTER BEFORE CANDIDATE ROWS.
//!
//! A fixed-height workspace used the flat picker's one-candidate floor. At the
//! enforced 464×288 minimum, zoom 1.4, and a visible menu bar, that floor spent
//! room the footer already needed: Paperbark's Rules rhythm seated the footer
//! below the card and all three Bars worlds shaped no footer run at all. History
//! shared the same allocator but had not been included in the exception ledger.
//!
//! This is an outcome law, not another ledger. It sweeps every world, both DPIs,
//! the reported minimum, and two ordinary controls. Settings and narrow History
//! go through the same workspace composition. Every footer must have shaped ink,
//! fit horizontally and vertically inside its card, and reach real pixels. Every
//! candidate row must remain inside the card at the world's own unchanged pitch.
//! The minimum is required to spend fewer rows than the roomy control in Rules,
//! Bars, and History, proving that the footer was protected by composition rather
//! than by a coincidental font or geometry change.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::workspace_back_width::{card_in_content, content_view, enrolled};
use super::{comparison_view, headless_dqp};
use crate::overlay::OverlayKind;

const GLYPH_STEP: f32 = 24.0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Surface {
    Settings,
    History,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Geometry {
    Minimum,
    Roomy,
    Base,
}

#[derive(Clone, Copy)]
struct Cell {
    geometry: Geometry,
    w: u32,
    h: u32,
    zoom: f32,
    menu_bar: bool,
}

const CELLS: [Cell; 3] = [
    Cell {
        geometry: Geometry::Minimum,
        w: 464,
        h: 288,
        zoom: 1.4,
        menu_bar: true,
    },
    Cell {
        geometry: Geometry::Roomy,
        w: 1200,
        h: 800,
        zoom: 1.4,
        menu_bar: true,
    },
    Cell {
        geometry: Geometry::Base,
        w: 1200,
        h: 800,
        zoom: 1.0,
        menu_bar: false,
    },
];

fn surface_view(surface: Surface) -> ViewState {
    match surface {
        Surface::Settings => {
            let kind = enrolled()
                .into_iter()
                .next()
                .expect("a content-row workspace enrolls");
            content_view(&card_in_content(kind))
        }
        Surface::History => {
            let mut v = comparison_view("# Compared draft\n\nChanged prose.\n", 0, 0);
            v.overlay_items = (0..12)
                .map(|i| format!("{i} hr ago · edited a section"))
                .collect();
            v.overlay_bindings = (0..12).map(|i| format!("+{i} −{i}")).collect();
            v.overlay_hint = crate::overlay::format_hint(&OverlayKind::History.rail_hint_actions());
            v.overlay_window_rows = OverlayKind::History.window_rows();
            v
        }
    }
}

fn luma(pixel: [u8; 4]) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}

#[derive(Clone, Copy)]
struct FooterSeat {
    card: [f32; 4],
    top: f32,
    bottom: f32,
}

/// Count columns carrying a local glyph edge inside the shaped footer line.
/// The scan stays two pixels inside the line box and card sides, excluding the
/// footer plate and card boundaries so deleting the text cannot satisfy it.
fn footer_ink_columns(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &TextPipeline,
    width: u32,
    height: u32,
    seat: FooterSeat,
) -> usize {
    let (texture, view) = offscreen(device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl workspace footer outcome"),
    });
    p.render(&mut encoder, &view).unwrap();
    queue.submit(Some(encoder.finish()));
    let pixels = read_pixels(device, queue, &texture, width, height);
    let x0 = (seat.card[0] + 4.0).ceil().max(1.0) as usize;
    let x1 = (seat.card[0] + seat.card[2] - 4.0)
        .floor()
        .min(width as f32 - 2.0) as usize;
    let y0 = (seat.top + 2.0).ceil().max(1.0) as usize;
    let y1 = (seat.bottom - 2.0).floor().min(height as f32 - 2.0) as usize;
    (x0..x1)
        .filter(|&x| {
            (y0..y1).any(|y| {
                let i = y * width as usize + x;
                (luma(pixels[i]) - luma(pixels[i + 1])).abs() > GLYPH_STEP
                    || (luma(pixels[i]) - luma(pixels[i + width as usize])).abs() > GLYPH_STEP
            })
        })
        .count()
}

#[derive(Clone, Copy, Default)]
struct RowPair {
    minimum: usize,
    roomy: usize,
}

#[derive(Default)]
struct LawTally {
    rows: std::collections::BTreeMap<(Surface, usize, u32), RowPair>,
    graded: usize,
    pixel_cells: usize,
}

struct FooterProof {
    card: [f32; 4],
    top: f32,
    bottom: f32,
}

struct TestRender<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    pipeline: &'a mut TextPipeline,
}

fn prove_card_and_footer(
    pipeline: &TextPipeline,
    v: &ViewState,
    surface: Surface,
    cell: Cell,
    width: u32,
    height: u32,
    ctx: &str,
) -> FooterProof {
    let card = pipeline.workspace_geometry(width).card_probe();
    assert!(
        card[0] >= 0.0
            && card[1] >= 0.0
            && card[0] + card[2] <= width as f32 + 0.5
            && card[1] + card[3] <= height as f32 + 0.5,
        "{ctx}: workspace card {card:?} leaves its canvas {width}x{height}"
    );
    if surface == Surface::History && cell.geometry == Geometry::Minimum {
        assert!(
            !pipeline.workspace_is_wide(width),
            "{ctx}: minimum History must exercise its narrow stage"
        );
    }
    let Some((content_bottom, top, bottom)) = pipeline.overlay_hint_gap_probe(width) else {
        panic!("{ctx}: the teaching footer was omitted")
    };
    let hint_line = pipeline
        .overlay_hint_line()
        .expect("the probe found the same footer line");
    let drawn_hint = pipeline.panel_buffer.lines[hint_line].text();
    assert!(
        v.overlay_hint.ends_with(drawn_hint),
        "{ctx}: footer cell order changed: authored {:?}, drawn {drawn_hint:?}",
        v.overlay_hint
    );
    let required = match surface {
        Surface::Settings => "back",
        Surface::History => "close",
    };
    assert!(
        drawn_hint.contains(required),
        "{ctx}: width yield omitted final `{required}` from {drawn_hint:?}"
    );
    assert!(
        bottom <= card[1] + card[3] + 0.5,
        "{ctx}: footer {top:.1}..{bottom:.1} exceeds card bottom {:.1}",
        card[1] + card[3]
    );
    assert!(
        top >= content_bottom,
        "{ctx}: footer {top:.1} begins before content ends at {content_bottom:.1}"
    );
    let (footer_width, text_width) = pipeline.overlay_footer_fit_probe(width);
    assert!(
        footer_width > 1.0 && footer_width <= text_width + 0.5,
        "{ctx}: footer shapes {footer_width:.1}px into {text_width:.1}px and clips"
    );
    FooterProof { card, top, bottom }
}

fn prove_row_rhythm(
    pipeline: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    proof: &FooterProof,
    ctx: &str,
) {
    let pitch = pipeline.overlay_lh();
    for (display, row) in plan.rows().iter().enumerate() {
        assert_eq!(
            row.height.to_bits(),
            pitch.to_bits(),
            "{ctx}: row {display} changed the world's {pitch:.1}px rhythm"
        );
        assert!(
            row.top >= proof.card[1] && row.bottom() <= proof.top + 0.5,
            "{ctx}: row {display} at {:.1}..{:.1} crosses footer at {:.1}",
            row.top,
            row.bottom(),
            proof.top
        );
        if let Some(next) = plan.rows().get(display + 1) {
            assert!(
                ((next.top - row.top) - pitch).abs() <= 0.01,
                "{ctx}: rows {display}/{} advance {:.2}px, not {pitch:.2}px",
                display + 1,
                next.top - row.top
            );
        }
    }
}

fn grade_cell(
    render: &mut TestRender<'_>,
    world_index: usize,
    surface: Surface,
    cell: Cell,
    dpi: f32,
    tally: &mut LawTally,
) {
    let pipeline = &mut render.pipeline;
    crate::menubar::set_menu_bar_on(cell.menu_bar);
    let (width, height) = ((cell.w as f32 * dpi) as u32, (cell.h as f32 * dpi) as u32);
    pipeline.set_dpi(dpi);
    pipeline.set_size(width as f32, height as f32);
    let mut v = surface_view(surface);
    v.zoom = cell.zoom;
    pipeline.set_view(&v);
    pipeline
        .prepare(render.device, render.queue, width, height)
        .unwrap();
    let world = &crate::theme::THEMES[world_index];
    let ctx = format!(
        "{} {surface:?} {:?} {}x{} zoom={} dpi={dpi} menu_bar={}",
        world.name, cell.geometry, cell.w, cell.h, cell.zoom, cell.menu_bar
    );
    let geom = pipeline.workspace_geometry(width);
    let plan = pipeline.overlay_row_plan(&geom);
    let proof = prove_card_and_footer(pipeline, &v, surface, cell, width, height, &ctx);
    prove_row_rhythm(pipeline, &plan, &proof, &ctx);
    let inked = footer_ink_columns(
        render.device,
        render.queue,
        pipeline,
        width,
        height,
        FooterSeat {
            card: proof.card,
            top: proof.top,
            bottom: proof.bottom,
        },
    );
    assert!(
        inked >= 8,
        "{ctx}: only {inked} footer columns carry a glyph edge"
    );
    tally.pixel_cells += 1;
    let pair = tally
        .rows
        .entry((surface, world_index, dpi.to_bits()))
        .or_default();
    match cell.geometry {
        Geometry::Minimum => pair.minimum = plan.candidate_rows(),
        Geometry::Roomy => pair.roomy = plan.candidate_rows(),
        Geometry::Base => {}
    }
    tally.graded += 1;
}

fn prove_yield_enrolment(tally: &LawTally) -> (usize, usize, usize) {
    let (mut rules, mut bars, mut history) = (0, 0, 0);
    for ((surface, world_index, _dpi), pair) in &tally.rows {
        let world = &crate::theme::THEMES[*world_index];
        assert!(
            pair.roomy > 0,
            "{} {surface:?}: roomy control plans no candidates",
            world.name
        );
        if pair.minimum >= pair.roomy {
            continue;
        }
        match (surface, world.render_caps.list_style) {
            (Surface::History, _) => history += 1,
            (_, theme::ListStyle::Rules(_)) => rules += 1,
            (_, theme::ListStyle::Bars) => bars += 1,
            _ => {}
        }
    }
    assert!(
        rules >= 2 && bars >= 6 && history >= crate::theme::THEMES.len() * 2,
        "minimum must yield Rules, Bars and History: rules={rules}, bars={bars}, history={history}"
    );
    (rules, bars, history)
}

#[test]
fn every_workspace_footer_is_reserved_before_rows_and_drawn_inside_its_card() {
    let _guard = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping workspace footer reserve law: no wgpu adapter");
        return;
    }
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let world_pin = crate::theme::WorldPin::snapshot();
    let Some((device, queue, mut pipeline)) = headless_dqp(64.0, 64.0) else {
        return;
    };
    let mut tally = LawTally::default();

    for world_index in 0..crate::theme::THEMES.len() {
        crate::theme::set_active(world_index);
        pipeline.sync_theme();
        for surface in [Surface::Settings, Surface::History] {
            for cell in CELLS {
                for dpi in [1.0f32, 2.0] {
                    grade_cell(
                        &mut TestRender {
                            device: &device,
                            queue: &queue,
                            pipeline: &mut pipeline,
                        },
                        world_index,
                        surface,
                        cell,
                        dpi,
                        &mut tally,
                    );
                }
            }
        }
    }
    let (rules, bars, history) = prove_yield_enrolment(&tally);
    assert_eq!(
        tally.graded,
        crate::theme::THEMES.len() * 2 * CELLS.len() * 2,
        "every world, surface, geometry and DPI must enroll"
    );
    assert_eq!(
        tally.pixel_cells, tally.graded,
        "every graded footer must reach pixels"
    );

    drop(world_pin);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    let outcomes = tally.graded;
    let pixels = tally.pixel_cells;
    eprintln!(
        "workspace footer reserve: {outcomes} outcomes and {pixels} pixel cells; \
         reductions rules={rules}, bars={bars}, history={history}; \
         ambient menu bar {ambient_menu_bar}"
    );
}
