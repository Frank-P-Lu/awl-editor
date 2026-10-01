//! Optical alignment is graded from rendered ink, across every world and DPI.
//! Geometry alone cannot prove the words look centered inside their buttons.
use super::super::*;
use super::{headless_dqp, pixeldiff, view};

fn control(g: &plan::PanelGeometry, name: &str) -> [f32; 4] {
    g.controls.iter().find(|c| c.name == name).unwrap().rect
}

fn rgba(c: theme::Srgb) -> [u8; 4] {
    [c.r, c.g, c.b, 255]
}

/// A perceptual foreground mask avoids assuming light or dark world colors.
fn ink_bounds(
    pixels: &[[u8; 4]],
    width: u32,
    rect: [f32; 4],
    ink: [u8; 4],
    ground: [u8; 4],
) -> [f32; 4] {
    let [x, y, w, h] = rect;
    let height = pixels.len() as u32 / width;
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    let contrast = pixeldiff::delta_e(ink, ground);
    assert!(contrast > 2.3, "ink must be perceptible against its ground");
    for py in (y.floor().max(0.0) as u32)..((y + h).ceil() as u32).min(height) {
        for px in (x.floor().max(0.0) as u32)..((x + w).ceil() as u32).min(width) {
            let color = pixels[(py * width + px) as usize];
            if pixeldiff::delta_e(color, ink) < contrast * 0.55 {
                left = left.min(px);
                top = top.min(py);
                right = right.max(px + 1);
                bottom = bottom.max(py + 1);
            }
        }
    }
    assert!(
        right > left && bottom > top,
        "no visible glyph ink in {rect:?}"
    );
    [left as f32, top as f32, right as f32, bottom as f32]
}

fn assert_header_ink(
    p: &TextPipeline,
    ctx: &str,
    pixels: &[[u8; 4]],
    width: u32,
    g: &plan::PanelGeometry,
    dpi: f32,
) {
    let muted = rgba(theme::muted());
    let ground = rgba(theme::THEMES[theme::active_index()].base_300);
    let field = control(g, "find_field");
    let header = ink_bounds(
        pixels,
        width,
        [field[0] - 2.0 * dpi, g.text_top, 60.0 * dpi, 36.0 * dpi],
        muted,
        ground,
    );
    assert!(
        (header[0] - field[0]).abs() <= 2.0 * dpi,
        "{ctx}: Find ink must start at the field edge: {header:?} / {field:?}"
    );
    let close = control(g, "close");
    let mark = ink_bounds(pixels, width, close, muted, ground);
    let shift = (mark[0] + mark[2]) * 0.5 - (close[0] + close[2] * 0.5);
    assert!(
        (3.0 * dpi..=6.0 * dpi).contains(&shift),
        "{ctx}: close ink should move right inside its target: {mark:?} / {close:?}"
    );
    assert!(close[2] >= 32.0 * dpi - 0.01 && close[3] >= 32.0 * dpi - 0.01);
    let reveal = control(g, "reveal_replace");
    let span = p.panel_control_spans.reveal.unwrap();
    let (x0, x1) = p
        .panel_span_x(span.row, span.byte_start, span.byte_start + "▾".len())
        .unwrap();
    let arrow = ink_bounds(
        pixels,
        width,
        [
            g.text_left + x0 - dpi,
            reveal[1],
            x1 - x0 + 2.0 * dpi,
            reveal[3],
        ],
        muted,
        ground,
    );
    assert!(
        arrow[3] - arrow[1] >= 6.5 * dpi - 0.01,
        "{ctx}: disclosure must have a larger visible chevron: {arrow:?}"
    );
}

fn assert_action_ink(ctx: &str, pixels: &[[u8; 4]], width: u32, g: &plan::PanelGeometry, dpi: f32) {
    let replacement = control(g, "replace_field");
    for name in ["replace_button", "replace_all_button"] {
        let [x, y, w, h] = control(g, name);
        let inset = 3.0 * dpi;
        let bbox = ink_bounds(
            pixels,
            width,
            [x + inset, y + inset, w - 2.0 * inset, h - 2.0 * inset],
            rgba(theme::base_content()),
            rgba(theme::THEMES[theme::active_index()].base_200),
        );
        let center = (bbox[1] + bbox[3]) * 0.5;
        let rect = [x, y, w, h];
        assert!(
            (center - (y + h * 0.5)).abs() <= 1.5 * dpi,
            "{ctx}: {name} words must be optically centered: {bbox:?} / {rect:?}"
        );
        assert!(
            y - (replacement[1] + replacement[3]) >= 8.0 * dpi - 0.51,
            "{ctx}: visible air is required below replacement before {name}"
        );
    }
}

#[test]
fn find_replace_optical_alignment_uses_visible_ink_in_every_world() {
    let _lock = crate::testlock::serial();
    let _world = theme::WorldPin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(464.0, 288.0) else {
        eprintln!("skipping panel optical law: no wgpu adapter");
        return;
    };
    let mut v = view("hello hello\n", 0, 0);
    v.search_active = true;
    v.search_query = "hello".into();
    v.search_matches = vec![((0, 0), (0, 5)), ((0, 6), (0, 11))];
    v.search_current = Some(0);
    v.search_replace_active = true;
    v.search_replacement = "goodbye".into();
    let mut enrolled = 0;
    for (i, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        p.sync_theme();
        for dpi in [1.0, 2.0] {
            let (w, h) = ((464.0 * dpi) as u32, (288.0 * dpi) as u32);
            p.set_dpi(dpi);
            p.set_size(w as f32, h as f32);
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let g = p.panel_geometry().unwrap();
            let ctx = format!("{}@{dpi}x", world.name);
            assert!(
                g.card[1] + g.card[3] <= h as f32,
                "{ctx}: card exceeds minimum canvas"
            );
            let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            assert_header_ink(&p, &ctx, &pixels, w, &g, dpi);
            assert_action_ink(&ctx, &pixels, w, &g, dpi);
            enrolled += 1;
        }
    }
    assert_eq!(enrolled, theme::THEMES.len() * 2);
}
