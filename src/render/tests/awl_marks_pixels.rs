//! Real-pixel laws for every production door the Nishiki-derived Awl Marks
//! face currently reaches, plus the retained list-bullet treatment that shares
//! the rule renderer. Enrolment comes from the roster (`symbol-span`) or live
//! theme data, never a copied glyph or world list. Every contrast/size assertion
//! carries a presence companion: a deleted renderer cannot pass by leaving an
//! immaculate background behind.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::overlay::OverlayKind;

const W: u32 = 1200;
const H: u32 = 1000;
const BODY_CONTRAST_FLOOR: f32 = 2.4;
const QUIET_CONTRAST_FLOOR: f32 = 2.5;
const PRESENCE_CONTRAST_FLOOR: f32 = 1.5;

#[derive(Clone, Copy, Debug)]
struct InkStats {
    pixels: usize,
    strong_pixels: usize,
    width: i32,
    height: i32,
    max_contrast: f32,
}

fn linear(channel: u8) -> f32 {
    let value = channel as f32 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(pixel: [u8; 4]) -> f32 {
    0.2126 * linear(pixel[0]) + 0.7152 * linear(pixel[1]) + 0.0722 * linear(pixel[2])
}

fn contrast(a: [u8; 4], b: [u8; 4]) -> f32 {
    let (lo, hi) = {
        let (a, b) = (luminance(a), luminance(b));
        (a.min(b), a.max(b))
    };
    (hi + 0.05) / (lo + 0.05)
}

fn clamp_rect([x, y, w, h]: [f32; 4]) -> (i32, i32, i32, i32) {
    (
        x.floor().max(0.0) as i32,
        y.floor().max(0.0) as i32,
        (x + w).ceil().min(W as f32) as i32,
        (y + h).ceil().min(H as f32) as i32,
    )
}

fn diff_stats(with: &[[u8; 4]], without: &[[u8; 4]], rect: [f32; 4]) -> InkStats {
    let (x0, y0, x1, y1) = clamp_rect(rect);
    let (mut left, mut right, mut top, mut bottom) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
    let (mut pixels, mut strong_pixels, mut max_contrast) = (0usize, 0usize, 1.0_f32);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = y as usize * W as usize + x as usize;
            let a = with[index];
            let b = without[index];
            if a == b {
                continue;
            }
            pixels += 1;
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
            let ratio = contrast(a, b);
            max_contrast = max_contrast.max(ratio);
            strong_pixels += usize::from(ratio >= PRESENCE_CONTRAST_FLOOR);
        }
    }
    InkStats {
        pixels,
        strong_pixels,
        width: if pixels == 0 { 0 } else { right - left + 1 },
        height: if pixels == 0 { 0 } else { bottom - top + 1 },
        max_contrast,
    }
}

fn against_local_ground(frame: &[[u8; 4]], rect: [f32; 4]) -> InkStats {
    let (_x0, y0, x1, y1) = clamp_rect(rect);
    let sample_x = (x1 + 8).min(W as i32 - 1);
    let sample_y = ((y0 + y1) / 2).clamp(0, H as i32 - 1);
    let ground = frame[sample_y as usize * W as usize + sample_x as usize];
    let blank = vec![ground; frame.len()];
    diff_stats(frame, &blank, rect)
}

fn assert_legible(ctx: &str, dpi: f32, floor: f32, stats: InkStats) {
    assert!(
        stats.pixels >= (4.0 * dpi).ceil() as usize,
        "{ctx}: presence floor failed: only {} rendered pixels at {dpi}x",
        stats.pixels
    );
    assert!(
        stats.strong_pixels > 0,
        "{ctx}: no rendered pixel clears the {PRESENCE_CONTRAST_FLOOR}:1 presence floor; \
         stats={stats:?}"
    );
    let major = stats.width.max(stats.height) as f32 / dpi;
    let minor = stats.width.min(stats.height) as f32 / dpi;
    assert!(
        major >= 4.0 && minor >= 0.75,
        "{ctx}: rendered mark is too small at {dpi}x: {stats:?}"
    );
    assert!(
        stats.max_contrast >= floor,
        "{ctx}: peak rendered contrast {:.2}:1 is below {:.2}:1",
        stats.max_contrast,
        floor
    );
}

fn role_chars(role: &str) -> Vec<char> {
    marks::roster()
        .iter()
        .filter(|mark| mark.roles.contains(&role))
        .map(|mark| char::from_u32(mark.codepoint).expect("roster Unicode scalar"))
        .collect()
}

/// Locate the actual last shaped About line without copying its text metrics.
/// This stays beside the one appearance law that needs the sampling rectangle.
fn about_ornament_rect(p: &TextPipeline, width: u32, height: u32) -> Option<[f32; 4]> {
    if !crate::about::about_open() {
        return None;
    }
    let runs: Vec<_> = p.hud_buffer.layout_runs().collect();
    let ornament = runs.last()?;
    let block_h = runs
        .iter()
        .map(|run| run.line_top + run.line_height)
        .fold(0.0_f32, f32::max);
    let block_w = runs.iter().map(|run| run.line_w).fold(0.0_f32, f32::max);
    let m = p.metrics;
    let plan = crate::render::plan::plan_float_card(
        [width as f32, height as f32],
        [block_w, block_h],
        [m.char_width * 3.0, m.line_height * 0.9],
        m.px(TEXT_TOP),
    );
    Some([
        plan.text[0],
        plan.text[1] + ornament.line_top,
        ornament.line_w,
        ornament.line_height,
    ])
}

fn document_fixture(chars: &[char], blank: bool) -> String {
    let mut text = String::new();
    for &ch in chars {
        text.push(if blank { ' ' } else { ch });
        text.push('\n');
    }
    text.push('\n');
    text
}

fn document_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    chars: &[char],
    blank: bool,
) -> Vec<[u8; 4]> {
    let text = document_fixture(chars, blank);
    let mut v = view(&text, chars.len(), 0);
    v.is_markdown = true;
    p.set_view(&v);
    p.prepare(device, queue, W, H).unwrap();
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    pixeldiff::render_frame(p, device, queue, W, H)
}

fn chrome_view(chars: &[char], blank: bool) -> ViewState {
    let mut v = view("body\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Keybindings.title().to_string();
    v.overlay_items = chars
        .iter()
        .enumerate()
        .map(|(index, _)| format!("mark {index}"))
        .collect();
    v.overlay_bindings = chars
        .iter()
        .map(|&ch| {
            if blank {
                " ".to_string()
            } else {
                ch.to_string()
            }
        })
        .collect();
    v.overlay_hint = OverlayKind::Keybindings.hint();
    v.overlay_window_rows = OverlayKind::Keybindings.window_rows();
    v
}

fn chrome_hint_view(ch: char, blank: bool) -> ViewState {
    let mut v = chrome_view(&[ch], true);
    v.overlay_hint = if blank {
        " ".to_string()
    } else {
        ch.to_string()
    };
    v
}

/// Generic prose, standard-size chrome columns, and compact teaching hints are
/// the three size/ink treatments reached by `symbol-span`. They sweep every
/// role member in every world. Adding a roster row enrolls it automatically;
/// removing a rendered glyph makes its paired frame go empty and fail presence.
#[test]
fn every_symbol_span_is_legible_at_document_and_chrome_sizes() {
    let _guard = crate::testlock::serial();
    let chars = role_chars("symbol-span");
    assert!(
        !chars.is_empty(),
        "the symbol-span consumer enrolled nothing"
    );
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Awl Marks symbol-span pixel law: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    let mut graded = Vec::new();
    let dpis = [1.0_f32, 2.0];
    let consumers = ["document", "chrome", "chrome-hint"];

    for dpi in dpis {
        p.set_dpi(dpi);
        for (world_index, world) in theme::THEMES.iter().enumerate() {
            theme::set_active(world_index);
            p.sync_theme();

            for &ch in &chars {
                let with = document_frame(&mut p, &device, &queue, &[ch], false);
                let rect = [
                    p.text_left() - 2.0 * dpi,
                    p.doc_top() + p.row_top_px(0),
                    p.metrics.line_height * 3.0,
                    p.row_height_px(0),
                ];
                let without = document_frame(&mut p, &device, &queue, &[ch], true);
                assert_legible(
                    &format!("{} document U+{:04X}", world.name, ch as u32),
                    dpi,
                    BODY_CONTRAST_FLOOR,
                    diff_stats(&with, &without, rect),
                );
                graded.push(("document", world.name, dpi.to_bits(), ch));
            }

            for group in chars.chunks(1) {
                p.set_view(&chrome_view(group, false));
                p.prepare(&device, &queue, W, H).unwrap();
                let probe = p.overlay_row_y_probe();
                let card = p.overlay_card_rect().expect("Keybindings draws a card");
                let rects: Vec<_> = (0..group.len())
                    .map(|row| {
                        let top = probe.secondary.get(&row).copied().unwrap_or_else(|| {
                            panic!("{} chrome row {row} shaped no binding", world.name)
                        });
                        // Mirrored/diagonal worlds may seat the secondary on the
                        // opposite side. The paired frame keeps the primary and
                        // row treatment byte-identical, so the full card width
                        // still isolates only this binding glyph without a
                        // hand-copied alignment classifier.
                        [card[0], top, card[2], probe.lh]
                    })
                    .collect();
                let with = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                p.set_view(&chrome_view(group, true));
                p.prepare(&device, &queue, W, H).unwrap();
                let without = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                for (&ch, rect) in group.iter().zip(rects) {
                    assert_legible(
                        &format!("{} chrome U+{:04X}", world.name, ch as u32),
                        dpi,
                        QUIET_CONTRAST_FLOOR,
                        diff_stats(&with, &without, rect),
                    );
                    graded.push(("chrome", world.name, dpi.to_bits(), ch));
                }
            }

            for &ch in &chars {
                p.set_view(&chrome_hint_view(ch, false));
                p.prepare(&device, &queue, W, H).unwrap();
                let card = p.overlay_card_rect().expect("hint draws in a card");
                let (_, top, bottom) = p
                    .overlay_hint_gap_probe(W)
                    .unwrap_or_else(|| panic!("{} shaped no teaching hint", world.name));
                let rect = [card[0], top, card[2], bottom - top];
                let with = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                p.set_view(&chrome_hint_view(ch, true));
                p.prepare(&device, &queue, W, H).unwrap();
                let without = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                assert_legible(
                    &format!("{} chrome hint U+{:04X}", world.name, ch as u32),
                    dpi,
                    QUIET_CONTRAST_FLOOR,
                    diff_stats(&with, &without, rect),
                );
                graded.push(("chrome-hint", world.name, dpi.to_bits(), ch));
            }
        }
    }
    assert_eq!(
        graded.len(),
        chars.len() * theme::THEMES.len() * dpis.len() * consumers.len(),
        "every roster-derived symbol is graded through every size/ink treatment"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
}

fn ornament_fixture() -> ViewState {
    let text = "---\n\n***\n\n___\n\n- a\n  - b\n    - c\n\n";
    let mut v = view(text, 10, 0);
    v.is_markdown = true;
    v
}

#[test]
fn every_rule_ornament_shapes_as_one_fitted_nishiki_run_at_both_dpis() {
    let _guard = crate::testlock::serial();
    assert_eq!(
        ORNAMENT_WEIGHT.0, 500,
        "ornaments request Nishiki at weight 500"
    );
    let Some((_, _, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping ornament run geometry law: no wgpu adapter");
        return;
    };
    let mut widths = std::collections::BTreeMap::new();
    let mut composed = 0usize;
    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        for (world_index, world) in theme::THEMES.iter().enumerate() {
            theme::set_active(world_index);
            p.sync_theme();
            p.set_view(&ornament_fixture());
            let probes = p.rule_run_shape_probe();
            assert_eq!(
                probes.len(),
                3,
                "{} shapes one run per rule syntax",
                world.name
            );
            for probe in probes {
                let text = probe.text;
                let layout_runs = probe.layout_runs;
                let glyphs = probe.glyphs;
                let width = probe.width;
                let faces = probe.faces;
                let components = text.chars().count();
                assert_eq!(
                    layout_runs, 1,
                    "{} {text:?} split into layout runs",
                    world.name
                );
                if components == 1 {
                    assert_eq!(glyphs, 1, "{} {text:?} did not shape one glyph", world.name);
                } else {
                    assert!(
                        (2..=components).contains(&glyphs),
                        "{} {text:?} produced an empty/foreign composed shape: \
                         components={components} glyphs={glyphs}",
                        world.name
                    );
                }
                assert!(
                    width.is_finite() && width > 0.0 && width <= p.text_wrap_width(),
                    "{} {text:?} has unfitted width {width}",
                    world.name
                );
                assert!(
                    faces.iter().all(|(family, weight)| {
                        family == theme::ORNAMENT_NISHIKI && *weight == 400
                    }),
                    "{} {text:?} escaped the bundled Nishiki-derived face: {faces:?}",
                    world.name
                );
                if components > 1 {
                    composed += 1;
                }
                let key = (world.name, text);
                if let Some(one_x) = widths.insert(key.clone(), width) {
                    let ratio = width / one_x;
                    assert!(
                        (1.98..=2.02).contains(&ratio),
                        "{} {:?} width must scale with dpi: 1x={one_x} 2x={width} ratio={ratio}",
                        key.0,
                        key.1
                    );
                }
            }
        }
    }
    assert_eq!(
        composed, 6,
        "three approved composed runs must be exercised at each of two DPIs"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}

fn run_codepoints(run: &str) -> String {
    run.chars()
        .map(|ch| format!("U+{:04X}", ch as u32))
        .collect::<Vec<_>>()
        .join("+")
}

/// Rule ornaments and list bullets are separate render sizes and positions.
/// The world roster itself decides enrolment; the frame is then redrawn with
/// only the ornament renderer emptied, making each diff the actual glyph ink.
#[test]
fn every_rule_ornament_and_existing_bullet_is_legible_at_its_real_size() {
    let _guard = crate::testlock::serial();
    let worlds: Vec<_> = theme::THEMES
        .iter()
        .enumerate()
        .filter(|(_, world)| world.ornament_face == theme::ORNAMENT_NISHIKI)
        .collect();
    assert!(
        !worlds.is_empty(),
        "no world currently consumes Awl Marks ornaments"
    );
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping rule ornament and bullet pixel law: no wgpu adapter");
        return;
    };
    crate::page::set_page_on(true);
    let mut graded_rules = 0usize;
    let mut graded_bullets = 0usize;
    let mut expected_uses = 0;
    for dpi in [1.0_f32, 2.0] {
        p.set_dpi(dpi);
        for &(world_index, world) in &worlds {
            theme::set_active(world_index);
            p.sync_theme();
            p.set_view(&ornament_fixture());
            p.prepare(&device, &queue, W, H).unwrap();
            let rules = p.rule_marks();
            let bullets = p.bullet_marks();
            assert_eq!(
                rules.iter().map(|&(_, ch)| ch).collect::<Vec<_>>(),
                vec![
                    world.ornaments.dash,
                    world.ornaments.star,
                    world.ornaments.underscore
                ],
                "{}: rendered rule enrolment drifted from theme consumers",
                world.name
            );
            assert_eq!(
                bullets.iter().map(|&(_, _, ch)| ch).collect::<Vec<_>>(),
                vec![world.bullets.0, world.bullets.1, world.bullets.2],
                "{}: rendered bullet enrolment drifted from theme consumers",
                world.name
            );
            expected_uses += rules.len() + bullets.len();
            let with = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
            p.md_enabled = false;
            p.prepare_ornaments(&device, &queue, W, H).unwrap();
            let without = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
            p.md_enabled = true;

            for (top, ch) in rules {
                let rect = [
                    p.text_left(),
                    top,
                    p.page_geometry().3,
                    p.metrics.line_height * world.ornament_scale,
                ];
                assert_legible(
                    &format!("{} rule {}", world.name, run_codepoints(ch)),
                    dpi,
                    QUIET_CONTRAST_FLOOR,
                    diff_stats(&with, &without, rect),
                );
                graded_rules += 1;
            }
            for (top, left, ch) in bullets {
                let rect = [left, top, p.metrics.char_width * 2.0, p.metrics.line_height];
                assert_legible(
                    &format!("{} bullet U+{:04X}", world.name, ch as u32),
                    dpi,
                    QUIET_CONTRAST_FLOOR,
                    diff_stats(&with, &without, rect),
                );
                graded_bullets += 1;
            }
        }
    }
    assert_eq!(
        graded_rules + graded_bullets,
        expected_uses,
        "every theme-derived rule and bullet consumer must be graded"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}

/// The About card uses the world's dash at body size and body ink, a distinct
/// production treatment from the larger muted rule. Its real last shaped line
/// supplies the sampling box; nearby card ground supplies the contrast pair.
#[test]
fn every_live_awl_marks_about_end_mark_is_present_sized_and_legible() {
    let _guard = crate::testlock::serial();
    let worlds: Vec<_> = theme::THEMES
        .iter()
        .enumerate()
        .filter(|(_, world)| world.ornament_face == theme::ORNAMENT_NISHIKI)
        .collect();
    assert!(
        !worlds.is_empty(),
        "no About card consumes an Awl Marks dash"
    );
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping Awl Marks About pixel law: no wgpu adapter");
        return;
    };
    crate::about::set_open(true);
    p.set_view(&view("body\n", 0, 0));
    let mut graded = Vec::new();
    let dpis = [1.0_f32, 2.0];
    for dpi in dpis {
        p.set_dpi(dpi);
        for &(world_index, world) in &worlds {
            theme::set_active(world_index);
            p.sync_theme();
            p.prepare(&device, &queue, W, H).unwrap();
            let rect = about_ornament_rect(&p, W, H)
                .unwrap_or_else(|| panic!("{} shaped no About end mark", world.name));
            assert!(
                rect[2] > 0.0,
                "{}: About end mark has zero advance",
                world.name
            );
            let frame = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
            assert_legible(
                &format!(
                    "{} About {}",
                    world.name,
                    run_codepoints(world.ornaments.dash)
                ),
                dpi,
                BODY_CONTRAST_FLOOR,
                against_local_ground(&frame, rect),
            );
            graded.push((world.name, dpi.to_bits(), world.ornaments.dash));
        }
    }
    crate::about::set_open(false);
    assert_eq!(
        graded.len(),
        worlds.len() * dpis.len(),
        "every live Awl Marks About consumer is graded at both DPIs"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}
