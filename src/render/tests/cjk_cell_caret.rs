//! The proportional cell caret over CJK: one resolved-face ideographic cell,
//! never the Latin typical-letter box and never the anchored kanji's own ink.

use super::super::*;
use super::{headless_dqp, headless_pipeline, pixeldiff, view};

const CJK: &str = "構成";
const W: u32 = 480;
const H: u32 = 420;

#[derive(Debug)]
struct PixelBounds {
    top: i32,
    bottom: i32,
    count: usize,
}

impl PixelBounds {
    fn height(&self) -> i32 {
        self.bottom - self.top + 1
    }
}

fn diff_bounds(a: &[[u8; 4]], b: &[[u8; 4]]) -> PixelBounds {
    let (mut top, mut bottom, mut count) = (i32::MAX, i32::MIN, 0usize);
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let i = y as usize * W as usize + x as usize;
            if a[i] != b[i] {
                top = top.min(y);
                bottom = bottom.max(y);
                count += 1;
            }
        }
    }
    PixelBounds { top, bottom, count }
}

fn clear_caret(p: &mut TextPipeline) {
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
}

fn proportional_worlds() -> impl Iterator<Item = &'static theme::Theme> {
    theme::THEMES
        .iter()
        .filter(|world| !facepitch::family_is_mono(world.font))
}

/// The device-free seam: typographic baseline shares place one exact em square
/// around the baseline, independent of glyph identity or display density.
#[test]
fn ideographic_cell_box_is_one_face_derived_em_at_every_scale() {
    for &font_size in &[12.0, 24.0, 48.0, 72.0] {
        let box_ = TextPipeline::ideographic_cell_box(font_size, (0.88, 0.12));
        assert!((box_.top - font_size * 0.88).abs() < 1e-4);
        assert!((box_.descent() - font_size * 0.12).abs() < 1e-4);
        assert!((box_.height - font_size).abs() < 1e-4);
    }
}

/// Every bundled CJK face enrolled by the font loader yields one sane full-em
/// cell. This proves the product roster, rather than a named Japanese face,
/// decides which metrics exist.
#[test]
fn every_bundled_cjk_face_enrolls_in_the_ideographic_cell_roster() {
    let mut enrolled = Vec::new();
    for bytes in facepitch::bundled_cjk_faces() {
        let family = facepitch::registered_family(bytes).expect("bundled CJK family");
        let (ascent, descent) = facepitch::ideographic_cell_em(&family)
            .unwrap_or_else(|| panic!("{family}: missing ideographic cell metrics"));
        assert!(
            ascent > 0.5 && descent >= 0.0,
            "{family}: {ascent}/{descent}"
        );
        assert!(
            (ascent + descent - 1.0).abs() < 1e-6,
            "{family}: ideographic cell must be exactly one em"
        );
        enrolled.push(family);
    }
    enrolled.sort();
    enrolled.dedup();
    assert_eq!(
        enrolled.len(),
        theme::EMBEDDED_CJK_FAMILIES.len(),
        "font-loader and theme CJK rosters must enroll the same families: {enrolled:?}"
    );
}

/// Geometry/pixel-adjacent law over the requested axes: full proportional
/// roster, both display densities, and both cell-form caret modes. The actual
/// swash ink for `成` must fit inside the cell with the authored pad, while `構`
/// and `成` share exactly one face-derived top/bottom (no per-kanji jitter).
#[test]
fn cjk_cell_forms_contain_kanji_ink_with_the_authored_pad_full_roster_both_dpis() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping CJK cell-caret law: no wgpu adapter");
        return;
    };

    let mut cells = 0usize;
    let mut tightest_pad = f32::MAX;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for world in proportional_worlds() {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for mode in [CaretMode::Block, CaretMode::Morph] {
                crate::caret::set_mode(mode);
                crate::caret::set_highlight_previous_character(mode == CaretMode::Morph);
                let mut edges = Vec::new();
                for anchor_col in 0..2 {
                    let cursor_col = match mode {
                        CaretMode::Block => anchor_col,
                        CaretMode::Morph => anchor_col + 1,
                        CaretMode::Ibeam => unreachable!(),
                    };
                    p.set_view(&view(CJK, 0, cursor_col));
                    p.settle_caret();
                    assert!(!p.caret_is_bar_form(), "{mode:?} must be a cell on CJK");

                    let raster = p.caret_anchor_raster_box().unwrap_or_else(|| {
                        panic!("{} d{dpi} {mode:?}: CJK anchor must rasterize", world.name)
                    });
                    let baseline = p.caret_baseline_y();
                    let ink_top = baseline - raster.top;
                    let ink_bottom = baseline + raster.descent();
                    let (cy, h) = p.caret_cell_vertical();
                    let top = cy - h * 0.5;
                    let bottom = cy + h * 0.5;
                    let pad = super::super::caret_body::CARET_BLOCK_INK_PAD.px(p.metrics.scale);
                    let top_pad = ink_top - top;
                    let bottom_pad = bottom - ink_bottom;
                    // Raster placement is integer-snapped; allow one device px
                    // while still requiring the authored logical pad on both edges.
                    let top_floor =
                        super::super::caret_body::CARET_BLOCK_TOP_PAD.px(p.metrics.scale) - dpi;
                    let floor = pad - dpi;
                    assert!(
                        top_pad >= top_floor && bottom_pad >= floor,
                        "{} d{dpi} {mode:?} col {anchor_col}: CJK ink \
                         {ink_top:.2}..{ink_bottom:.2} \
                         must sit inside caret {top:.2}..{bottom:.2} with authored pad {pad:.2}; \
                         got {top_pad:.2}/{bottom_pad:.2}",
                        world.name
                    );
                    tightest_pad = tightest_pad.min(top_pad.min(bottom_pad) / dpi);
                    edges.push((top, bottom));
                    cells += 1;
                }
                assert!(
                    edges
                        .iter()
                        .all(|(top, bottom)| top.is_finite() && bottom > top)
                );
            }
        }
    }
    assert_eq!(cells, proportional_worlds().count() * 2 * 2 * 2);
    eprintln!(
        "CJK cell caret: {cells} anchors across proportional roster × 2 DPI × \
         Block/Morph; tightest measured ink pad {tightest_pad:.2} logical px"
    );

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
}

/// The same containment claim at the final rendered seam. Caret pixels are the
/// diff between a prepared frame and that exact frame with all caret pipelines
/// emptied; glyph ink is the caret-free `成` frame against the same world with an
/// empty row. Presence is asserted separately so an absent caret cannot satisfy
/// containment.
#[test]
fn rendered_cjk_cell_is_present_and_contains_kanji_ink_with_pad() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping rendered CJK cell law: no wgpu adapter");
        return;
    };

    let mut cells = 0usize;
    let mut tightest = i32::MAX;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for world in proportional_worlds() {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for mode in [CaretMode::Block, CaretMode::Morph] {
                crate::caret::set_mode(mode);
                crate::caret::set_highlight_previous_character(mode == CaretMode::Morph);
                let cursor_col = match mode {
                    CaretMode::Block => 0,
                    CaretMode::Morph => 1,
                    CaretMode::Ibeam => unreachable!(),
                };
                p.set_view(&view("成", 0, cursor_col));
                p.settle_caret();
                p.prepare(&device, &queue, W, H).unwrap();
                let with_caret = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                clear_caret(&mut p);
                let glyph_frame = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                let caret = diff_bounds(&with_caret, &glyph_frame);

                p.set_view(&view("", 0, 0));
                p.settle_caret();
                p.prepare(&device, &queue, W, H).unwrap();
                clear_caret(&mut p);
                let empty_frame = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
                let ink = diff_bounds(&glyph_frame, &empty_frame);

                assert!(
                    caret.count > 0 && caret.height() >= 8 * dpi as i32,
                    "{} d{dpi} {mode:?}: caret presence failed: {caret:?}",
                    world.name
                );
                assert!(
                    ink.count > 0 && ink.height() >= 8 * dpi as i32,
                    "{} d{dpi}: CJK fixture ink is absent: {ink:?}",
                    world.name
                );
                let top_pad = ink.top - caret.top;
                let bottom_pad = caret.bottom - ink.bottom;
                // Geometry above proves the authored logical pad. Final pixel
                // bounds are inclusive and both glyph and quad edges snap, so
                // retain that pad minus two device pixels at this raster seam.
                let required_top = (super::super::caret_body::CARET_BLOCK_TOP_PAD
                    .px(p.metrics.scale)
                    .floor() as i32
                    - 2)
                .max(0);
                let required = (super::super::caret_body::CARET_BLOCK_INK_PAD
                    .px(p.metrics.scale)
                    .floor() as i32
                    - 2)
                .max(0);
                assert!(
                    top_pad >= required_top && bottom_pad >= required,
                    "{} d{dpi} {mode:?}: rendered CJK ink {ink:?} must fit inside \
                     rendered caret {caret:?} with pad >= {required}px; got \
                     {top_pad}/{bottom_pad}px",
                    world.name
                );
                tightest = tightest.min(top_pad.min(bottom_pad));
                cells += 1;
            }
        }
    }
    assert_eq!(cells, proportional_worlds().count() * 2 * 2);
    eprintln!(
        "rendered CJK caret: {cells} proportional-world/DPI/form cells; \
         tightest device-pixel ink pad {tightest}px"
    );

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
}

/// Switching between Latin's deliberately shorter typical-letter cell and the
/// full ideographic cell is a bounded face/script transition, not a per-glyph
/// jump. Both sides use their production anchors and every proportional world,
/// display density, and cell-form caret participates.
#[test]
fn mixed_latin_cjk_cell_transitions_stay_bounded() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping mixed Latin/CJK transition law: no wgpu adapter");
        return;
    };

    let text = "a構成a";
    let mut saw_real_step = false;
    let mut cells = 0usize;
    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for world in proportional_worlds() {
            theme::set_active_by_name(world.name).unwrap();
            p.sync_theme();
            for mode in [CaretMode::Block, CaretMode::Morph] {
                crate::caret::set_mode(mode);
                crate::caret::set_highlight_previous_character(mode == CaretMode::Morph);
                let mut edges = Vec::new();
                for anchor_col in 0..text.chars().count() {
                    let cursor_col = match mode {
                        CaretMode::Block => anchor_col,
                        CaretMode::Morph => anchor_col + 1,
                        CaretMode::Ibeam => unreachable!(),
                    };
                    p.set_view(&view(text, 0, cursor_col));
                    p.settle_caret();
                    let (cy, h) = p.caret_cell_vertical();
                    edges.push((cy - h * 0.5, cy + h * 0.5));
                }
                let bound = p.metrics.line_height * 0.75;
                for pair in edges.windows(2) {
                    let top_step = (pair[1].0 - pair[0].0).abs();
                    let bottom_step = (pair[1].1 - pair[0].1).abs();
                    assert!(
                        top_step <= bound && bottom_step <= bound,
                        "{} d{dpi} {mode:?}: mixed Latin/CJK edge step \
                         {top_step:.2}/{bottom_step:.2}px exceeds quarter-row \
                         bound {bound:.2}: {edges:?}",
                        world.name
                    );
                    saw_real_step |= top_step > dpi || bottom_step > dpi;
                    cells += 1;
                }
            }
        }
    }
    assert!(
        saw_real_step,
        "fixture must exercise two genuinely different cells"
    );
    assert_eq!(cells, proportional_worlds().count() * 2 * 2 * 3);

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
}

type BracketMeasurement = (&'static str, f32, f32, f32, f32, f32, f32);

fn measure_japanese_bracket_cell(
    p: &mut TextPipeline,
    world: &'static str,
    expected_family: &str,
    zoom: f32,
    dpi: f32,
) -> BracketMeasurement {
    let text = "の「文」へ";
    let mut focused = view(text, 0, 1);
    focused.zoom = zoom;
    p.set_view(&focused);
    p.settle_caret();

    let xs = p.line_glyph_xs(0);
    let advance = xs[2] - xs[1];
    let key = p
        .cursor_glyph_key_at(0, 1)
        .unwrap_or_else(|| panic!("{world} z{zoom} d{dpi}: bracket glyph"));
    let actual_family = p
        .font_system
        .db()
        .face(key.font_id)
        .expect("bracket glyph face")
        .families[0]
        .0
        .clone();
    assert_eq!(actual_family, expected_family, "{world} z{zoom} d{dpi}");
    assert!(
        p.caret_anchor_ideographic_cell().is_some(),
        "{world} z{zoom} d{dpi}: U+300C must use the CJK cell"
    );
    let font_size = p.metrics.font_size;
    assert!(
        advance >= font_size * 0.9 && advance <= font_size * 1.1,
        concat!(
            "{} z{:.1} d{:.0}: bracket advance {:.2} is not one ",
            "ideographic cell around font size {:.2}"
        ),
        world,
        zoom,
        dpi,
        advance,
        font_size
    );

    let bracket_ink = p
        .caret_anchor_raster_box()
        .unwrap_or_else(|| panic!("{world} z{zoom} d{dpi}: bracket raster"));
    let (caret_cx, _, caret_w, _, _, _, _) = p.caret_geometry();
    let caret_left = caret_cx - caret_w * 0.5;
    let bracket_cell_left = p.text_left() + xs[1];
    let leading = bracket_ink.left;
    let trailing = advance - bracket_ink.left - bracket_ink.width;

    let (ime_x, ime_y, ime_w, ime_h) = p.caret_pixel_rect();
    assert!((ime_x - bracket_cell_left).abs() < 1e-3);
    assert!((ime_w - advance).abs() < 1e-3);
    let (_, hit_col) = p.hit_test_scroll(
        bracket_cell_left + advance * 0.25,
        ime_y + ime_h * 0.5,
        ScrollPos::default(),
    );
    assert_eq!(hit_col, 1, "{world} z{zoom} d{dpi}: bracket midpoint hit");

    let mut selected = view(text, 0, 2);
    selected.zoom = zoom;
    selected.selection = Some(((0, 1), (0, 2)));
    p.set_view(&selected);
    let selection = p.selection_rects();
    assert_eq!(
        selection.len(),
        1,
        "{world} z{zoom} d{dpi}: one selection row"
    );
    assert!((selection[0][0] - bracket_cell_left).abs() < 1e-3);
    assert!((selection[0][2] - advance).abs() < 1e-3);

    let mut composing = view("の文」へ", 0, 1);
    composing.zoom = zoom;
    composing.preedit = "「".to_string();
    p.set_view(&composing);
    let preedit = p.preedit_rects();
    assert_eq!(preedit.len(), 1, "{world} z{zoom} d{dpi}: one preedit row");
    assert!((preedit[0][0] - bracket_cell_left).abs() < 1e-3);
    assert!((preedit[0][2] - advance).abs() < 1e-3);

    let mut unfocused = view(text, 0, 0);
    unfocused.zoom = zoom;
    p.set_view(&unfocused);
    let unfocused_xs = p.line_glyph_xs(0);
    assert_eq!(
        unfocused_xs.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        xs.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
        "{world} z{zoom} d{dpi}: focus must not reshape punctuation"
    );
    let kana_ink = p
        .caret_anchor_raster_box()
        .unwrap_or_else(|| panic!("{world} z{zoom} d{dpi}: kana raster"));
    let kana_ink_right = p.text_left() + xs[0] + kana_ink.left + kana_ink.width;
    let visible_gap = caret_left - kana_ink_right;
    (world, zoom, dpi, advance, leading, trailing, visible_gap)
}

fn assert_wrapped_japanese_bracket_geometry(p: &mut TextPipeline) {
    p.set_dpi(1.0);
    p.set_size(320.0, 420.0);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();
    let wrapped = "日本語の「文」を静かに書く。".repeat(12);
    let source = wrapped.as_bytes().to_vec();
    let bracket_cols = wrapped
        .chars()
        .enumerate()
        .filter_map(|(col, ch)| (ch == '「').then_some(col));
    let mut saw_later_row = false;
    for col in bracket_cols {
        p.set_view(&view(&wrapped, 0, col));
        let rows = p.visual_rows(0);
        assert!(rows.len() > 1, "narrow fixture must wrap");
        let row = pick_row(&rows, col);
        let cell_x = p.text_left() + row.xs[col];
        let advance = row.xs[col + 1] - row.xs[col];
        let (ime_x, ime_y, ime_w, ime_h) = p.caret_pixel_rect();
        assert!((ime_x - cell_x).abs() < 1e-3);
        assert!((ime_w - advance).abs() < 1e-3);
        let (_, hit_col) = p.hit_test_scroll(
            cell_x + advance * 0.25,
            ime_y + ime_h * 0.5,
            ScrollPos::default(),
        );
        assert_eq!(hit_col, col, "wrapped U+300C at source column {col}");
        saw_later_row |= row.line_top > 0.0;
    }
    assert!(
        saw_later_row,
        "fixture must exercise punctuation after a wrap"
    );
    assert_eq!(
        wrapped.as_bytes(),
        source,
        "reflow never changes source bytes"
    );
}

/// The reported `の「` seam, measured on all five bundled Japanese families at
/// three zooms and both display densities. Shaping, caret, hit, selection, IME,
/// focus, and wrapped-line geometry must agree on the same one-em cell.
#[test]
fn japanese_opening_bracket_face_cell_and_horizontal_geometry_agree_across_matrix() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping Japanese punctuation geometry matrix: no wgpu adapter");
        return;
    };
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
    let worlds = [
        ("Paperbark", "Shippori Mincho"),
        ("Saltpan", "Noto Serif JP"),
        ("Currawong", "Noto Sans JP"),
        ("Galah", "Zen Maru Gothic"),
        ("Mopoke", "Klee One"),
    ];
    let mut cells = 0usize;
    let mut measured = Vec::new();

    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for &(world, expected_family) in &worlds {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for &zoom in &[0.8f32, 1.0, 1.6] {
                measured.push(measure_japanese_bracket_cell(
                    &mut p,
                    world,
                    expected_family,
                    zoom,
                    dpi,
                ));
                cells += 1;
            }
        }
    }

    assert_eq!(cells, worlds.len() * 3 * 2);
    for row in measured {
        eprintln!(
            concat!(
                "U+300C world={} zoom={:.1} dpi={:.0} advance={:.2}px ",
                "leading={:.2}px trailing={:.2}px kana-ink-to-caret={:.2}px"
            ),
            row.0, row.1, row.2, row.3, row.4, row.5, row.6
        );
    }
    assert_wrapped_japanese_bracket_geometry(&mut p);

    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
    crate::caret::set_highlight_previous_character(false);
}
