//! THE PAGE-FRAME PIXEL LAW (`theme::PageFrame`, the personality-assignment
//! round's graduated capability) — the render-side half of the theme-side
//! `theme::tests::page_frame_ink_is_the_ladder_and_assigned_weights_are_real`:
//! the ASSIGNED state must be PIXEL-PROVABLE (frame pixels genuinely drawn,
//! in-bounds, in the world's own ladder ink — never inferred from an
//! instance count, the Wagtail-invisible-row lesson), and the None state
//! must genuinely upload NOTHING (zero instances IS the outcome there: an
//! empty instance buffer cannot draw). Wagtail — the first and only
//! assignment (2px, its ladder white) — is the fixture; a default-caps
//! world is the byte-identity control.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};

// --- the AWL_PAGE_FRAME_FORCE grammar (pure) — the probe that survives the
// --- AWL_PAGE_BORDER graduation, reshaped to force the CAPABILITY only.

#[test]
fn parse_page_frame_force_accepts_none_and_positive_weights() {
    assert_eq!(parse_page_frame_force("none"), Some(theme::PageFrame::None));
    assert_eq!(
        parse_page_frame_force("None"),
        Some(theme::PageFrame::None),
        "case-insensitive"
    );
    assert_eq!(
        parse_page_frame_force("2"),
        Some(theme::PageFrame::Line {
            weight_px: crate::render::Logical(2.0)
        })
    );
    assert_eq!(
        parse_page_frame_force(" 1.5 "),
        Some(theme::PageFrame::Line {
            weight_px: crate::render::Logical(1.5)
        }),
        "whitespace-tolerant"
    );
}

#[test]
fn parse_page_frame_force_rejects_garbage() {
    for bad in ["", "wat", "0", "-2", "inf", "NaN", "2px"] {
        assert_eq!(
            parse_page_frame_force(bad),
            None,
            "expected None for {bad:?}"
        );
    }
}

#[test]
fn page_frame_vertical_bounds_cover_short_tall_scrolled_and_menu_bar_cases() {
    use crate::render::layers::page_frame_vertical_bounds;

    let cases = [
        ("empty", 16.0, 0.0, 0.0, 359.0, 16.0, 359.0),
        ("one line", 16.0, 32.0, 0.0, 359.0, 16.0, 359.0),
        ("short", 16.0, 64.0, 0.0, 359.0, 16.0, 359.0),
        ("tall", 16.0, 3200.0, 0.0, 359.0, 16.0, 359.0),
        ("scrolled", -624.0, 3200.0, 0.0, 359.0, 0.0, 359.0),
        (
            "scrolled below menu",
            -600.0,
            3200.0,
            24.0,
            359.0,
            24.0,
            359.0,
        ),
    ];
    for (name, doc_top, doc_height, menu_bottom, canvas_bottom, want_top, want_bottom) in cases {
        assert_eq!(
            page_frame_vertical_bounds(doc_top, doc_height, menu_bottom, canvas_bottom),
            (want_top, want_bottom),
            "{name} frame bounds"
        );
    }
    assert_ne!(16.0 + 32.0, 359.0, "sanity: old one-line bottom must fail");
}

/// THE ASSIGNED HALF, over real GPU output: Wagtail's 2px frame draws PURE
/// LADDER WHITE (`page_frame_ink` = `base_content` = `#FFFFFF`, exactly —
/// the hard-edged dither-1.0 fill has no antialiased fringe, so the one-bit
/// law needs no tolerance here) at the expected coordinates: straddling the
/// writing column's left and right edges and its top edge, strictly INSIDE
/// the canvas, with flat pure-black ground further out in the margin and on
/// the page itself. Then THE ABSENT HALF: a default-caps world prepared
/// through the same path uploads ZERO frame rects (structurally nothing to
/// draw — the byte-identity guarantee for the fifteen None worlds).
#[test]
fn wagtail_page_frame_draws_pure_ladder_white_in_bounds_and_none_worlds_draw_none() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(500.0, 360.0) else {
        eprintln!(
            "skipping wagtail_page_frame_draws_pure_ladder_white_in_bounds_and_none_worlds_draw_none: no wgpu adapter"
        );
        return;
    };
    let was_menu_bar_on = crate::menubar::menu_bar_on();
    crate::page::set_measure(24);
    crate::page::set_page_on(true);
    crate::menubar::set_menu_bar_on(false);

    theme::set_active_by_name("Wagtail").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);
    p.prepare(&device, &queue, 500, 360).unwrap();

    // The frame reads the SAME geometry owners the renderer does.
    let left = p.column_left();
    let colw = p.column_width();
    let right = left + colw;
    let top = p.doc_top().max(0.0);
    let weight = 2.0f32;

    let (texture, tview) = offscreen(&device, 500, 360);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl page-frame encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let pixels = read_pixels(&device, &queue, &texture, 500, 360);
    let at = |x: i64, y: i64| -> [u8; 4] { pixels[(y * 500 + x) as usize] };
    let white = [255u8, 255, 255, 255];
    let black = [0u8, 0, 0, 255];

    // Sample the CENTER of each 2px edge band, on a row safely inside the
    // document's vertical extent (mid first line — the frame spans the doc).
    let mid_y = (top + LINE_HEIGHT * 0.5) as i64;
    let left_band_x = (left - weight * 0.5).floor() as i64;
    let right_band_x = (right + weight * 0.5).floor() as i64;
    assert_eq!(
        at(left_band_x, mid_y),
        white,
        "the frame's LEFT edge band must be the pure ladder white at ({left_band_x}, {mid_y})"
    );
    assert_eq!(
        at(right_band_x, mid_y),
        white,
        "the frame's RIGHT edge band must be the pure ladder white at ({right_band_x}, {mid_y})"
    );
    // The TOP edge band, sampled mid-column (no glyph sits above the doc top).
    let mid_x = (left + colw * 0.5) as i64;
    let top_band_y = (top - weight * 0.5).floor() as i64;
    assert_eq!(
        at(mid_x, top_band_y),
        white,
        "the frame's TOP edge band must be the pure ladder white at ({mid_x}, {top_band_y})"
    );
    let bottom_band_y = 359;
    assert_eq!(
        at(mid_x, bottom_band_y),
        white,
        "short document frame reaches canvas bottom"
    );
    // IN-BOUNDS: every sampled band coordinate is strictly on-canvas (the
    // samples above would have panicked on an out-of-range index otherwise —
    // assert it explicitly so the law reads).
    for (x, y) in [
        (left_band_x, mid_y),
        (right_band_x, mid_y),
        (mid_x, top_band_y),
        (mid_x, bottom_band_y),
    ] {
        assert!(
            (0..500).contains(&x) && (0..360).contains(&y),
            "frame sample ({x}, {y}) fell off the canvas — the frame must draw in-bounds"
        );
    }
    // FIGURE/GROUND stays flat around the frame: pure black just outside in
    // the margin, and pure black just inside on the page (below the text
    // lines — line 2 is empty, so no glyph interferes).
    let margin_x = (left - weight - 4.0) as i64;
    let inside_x = (left + weight + 4.0) as i64;
    let empty_row_y = (top + LINE_HEIGHT * 2.5) as i64;
    assert_eq!(
        at(margin_x.max(0), mid_y),
        black,
        "the margin just OUTSIDE the frame stays the flat pure-black ground"
    );
    assert_eq!(
        at(inside_x, empty_row_y),
        black,
        "the page just INSIDE the frame stays the flat pure-black ground"
    );

    crate::page::set_page_on(false);
    p.set_view(&v);
    p.prepare(&device, &queue, 500, 360).unwrap();
    let off_mid_x = (p.column_left() + p.column_width() * 0.5) as i64;
    let off_left_band_x = (p.column_left() - weight * 0.5).floor() as i64;
    let (texture, tview) = offscreen(&device, 500, 360);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl page-frame page-off encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let off_pixels = read_pixels(&device, &queue, &texture, 500, 360);
    assert_eq!(
        p.page_frame_pipeline.instance_count(),
        0,
        "page-off uploads no page-frame rects"
    );
    assert_eq!(
        off_pixels[(359 * 500 + off_mid_x) as usize],
        black,
        "page-off has no former bottom page-frame stroke"
    );
    assert_eq!(
        off_pixels[(160 * 500 + off_left_band_x) as usize],
        black,
        "page-off has no former left page-frame stroke"
    );

    crate::page::set_page_on(true);
    let tall = (0..100).map(|_| "line").collect::<Vec<_>>().join("\n");
    let mut scrolled = view(&tall, 0, 0);
    scrolled.scroll = ScrollPos::at_row(40);
    p.set_view(&scrolled);
    p.prepare(&device, &queue, 500, 360).unwrap();
    let scroll_left_band = (p.column_left() - weight * 0.5).floor() as i64;
    let (texture, tview) = offscreen(&device, 500, 360);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl page-frame scrolled encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let scrolled_pixels = read_pixels(&device, &queue, &texture, 500, 360);
    assert_eq!(
        scrolled_pixels[(4 * 500 + scroll_left_band) as usize],
        white
    );
    assert_eq!(
        scrolled_pixels[(359 * 500 + scroll_left_band) as usize],
        white
    );

    crate::menubar::set_menu_bar_on(true);
    p.set_view(&scrolled);
    p.prepare(&device, &queue, 500, 360).unwrap();
    let reserve = p.menubar_reserve() as i64;
    let menu_left_band = (p.column_left() - weight * 0.5).floor() as i64;
    let (texture, tview) = offscreen(&device, 500, 360);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl page-frame menu-bar encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let menu_pixels = read_pixels(&device, &queue, &texture, 500, 360);
    assert_eq!(
        menu_pixels[((reserve + 4) * 500 + menu_left_band) as usize],
        white
    );
    assert_ne!(
        menu_pixels[((reserve - 1) * 500 + menu_left_band) as usize],
        white
    );
    crate::menubar::set_menu_bar_on(false);

    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        p.set_view(&v);
        p.prepare(&device, &queue, 500, 360).unwrap();
        let expected = match world.render_caps.page_frame {
            theme::PageFrame::None => 0,
            theme::PageFrame::Line { .. } => 4,
        };
        assert_eq!(
            p.page_frame_pipeline.instance_count(),
            expected,
            "{}: state follows capability",
            world.name
        );
    }

    crate::page::set_page_on(false);
    for world in theme::THEMES.iter() {
        theme::set_active_by_name(world.name).unwrap();
        p.sync_theme();
        p.set_view(&v);
        p.prepare(&device, &queue, 500, 360).unwrap();
        assert_eq!(
            p.page_frame_pipeline.instance_count(),
            0,
            "{}: page-off uploads no page-frame rects regardless of capability",
            world.name
        );
    }

    crate::menubar::set_menu_bar_on(was_menu_bar_on);
}
