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

fn assert_header_ink(ctx: &str, pixels: &[[u8; 4]], width: u32, g: &plan::PanelGeometry, dpi: f32) {
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
}

fn assert_caption_marks(
    p: &TextPipeline,
    ctx: &str,
    pixels: &[[u8; 4]],
    width: u32,
    g: &plan::PanelGeometry,
    dpi: f32,
) {
    let ground = rgba(theme::THEMES[theme::active_index()].base_300);
    let field = control(g, "find_field");
    for spec in &p.panel_control_marks.spans {
        let control_name = match spec.kind {
            chrome::ControlMarkKind::Checkbox(_) => "case_toggle",
            chrome::ControlMarkKind::Disclosure(_) => "reveal_replace",
        };
        let target = control(g, control_name);
        let (left, right) = p
            .panel_span_x(spec.slot.row, spec.slot.byte_start, spec.slot.byte_end)
            .unwrap();
        let mark = ink_bounds(
            pixels,
            width,
            [
                g.text_left + left - dpi,
                target[1],
                right - left + 2.0 * dpi,
                target[3],
            ],
            spec.color,
            ground,
        );
        let (left, right) = p
            .panel_span_x(
                spec.caption.row,
                spec.caption.byte_start,
                spec.caption.byte_end,
            )
            .unwrap();
        let caption = ink_bounds(
            pixels,
            width,
            [g.text_left + left, target[1], right - left, target[3]],
            spec.color,
            ground,
        );
        assert!(
            (mark[0] - field[0]).abs() <= 3.0 * dpi,
            "{ctx}: {control_name} mark must join the shared left edge: {mark:?} / {field:?}"
        );
        let error = ((mark[1] + mark[3]) - (caption[1] + caption[3])) * 0.5;
        assert!(
            error.abs() <= 1.5 * dpi,
            "{ctx}: {control_name} mark and caption must share visible center: \
             {mark:?} / {caption:?}"
        );
        let (w, h) = (mark[2] - mark[0], mark[3] - mark[1]);
        match spec.kind {
            chrome::ControlMarkKind::Checkbox(_) => assert!(
                w.max(h) >= 11.0 * dpi - 1.0,
                "{ctx}: checkbox ink is too small: {mark:?}"
            ),
            chrome::ControlMarkKind::Disclosure(expanded) => {
                assert!(
                    w.max(h) >= 11.0 * dpi - 1.0,
                    "{ctx}: disclosure ink is too small: {mark:?}"
                );
                assert!(
                    if expanded { w > h } else { h > w },
                    "{ctx}: disclosure must point down when expanded and right when collapsed: \
                     {mark:?}"
                );
            }
        }
    }
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
    let ambient_bar = crate::menubar::menu_bar_on();
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for (i, world) in theme::THEMES.iter().enumerate() {
            theme::set_active(i);
            p.sync_theme();
            for dpi in [1.0, 2.0] {
                let (w, h) = ((464.0 * dpi) as u32, (288.0 * dpi) as u32);
                p.set_dpi(dpi);
                p.set_size(w as f32, h as f32);
                for checked in [false, true] {
                    for expanded in [false, true] {
                        v.search_case_sensitive = checked;
                        v.search_replace_active = expanded;
                        p.set_view(&v);
                        p.prepare(&device, &queue, w, h).unwrap();
                        let g = p.panel_geometry().unwrap();
                        let ctx = format!(
                            "{}@{dpi}x checked={checked} expanded={expanded}",
                            world.name
                        );
                        assert!(
                            g.card[1] >= p.menubar_reserve()
                                && g.card[1] + g.card[3] <= h as f32 + 0.01,
                            "{ctx}: card exceeds minimum canvas: {:?}, menubar={} rows={:?}",
                            g.card,
                            p.menubar_reserve(),
                            g.rows
                        );
                        let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                        assert_header_ink(&ctx, &pixels, w, &g, dpi);
                        assert_caption_marks(&p, &ctx, &pixels, w, &g, dpi);
                        if expanded {
                            assert_action_ink(&ctx, &pixels, w, &g, dpi);
                        }
                        enrolled += 1;
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    assert_eq!(enrolled, theme::THEMES.len() * 16);
}
