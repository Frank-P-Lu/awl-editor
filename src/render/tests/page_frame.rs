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

/// THE PREMISE CHECK. Before the fix this table chose `doc_top` (the text
/// inset, `~16.0` at scale 1) as `top` for every UNSCROLLED case — the exact
/// gap the user saw as a plain strip between the canvas top and the frame.
/// The frame is a canvas-owned writing-surface boundary, so `top` must now be
/// `menubar_bottom + stroke_px` in every case, REGARDLESS of `doc_top` — the
/// "unscrolled with menu bar" and "DPI 2x" cases below both carry a `doc_top`
/// far from the expected answer specifically to prove that decoupling: if a
/// future edit reintroduces `doc_top` into the top computation, those two
/// cases stop matching their neighbours' formula and the whole table goes
/// red by name.
#[test]
fn page_frame_vertical_bounds_cover_short_tall_scrolled_and_menu_bar_cases() {
    use crate::render::layers::page_frame_vertical_bounds;

    let cases = [
        // name, doc_top, doc_height, menu_bottom, canvas_bottom, stroke_px, want_top, want_bottom
        ("empty", 16.0, 0.0, 0.0, 359.0, 2.0, 2.0, 359.0),
        ("one line", 16.0, 32.0, 0.0, 359.0, 2.0, 2.0, 359.0),
        ("short", 16.0, 64.0, 0.0, 359.0, 1.0, 1.0, 359.0),
        ("tall", 16.0, 3200.0, 0.0, 359.0, 2.0, 2.0, 359.0),
        // Scroll must not move the top: doc_top swings deeply negative and
        // the answer is unchanged from the unscrolled cases above.
        ("scrolled", -624.0, 3200.0, 0.0, 359.0, 2.0, 2.0, 359.0),
        (
            "scrolled below menu",
            -600.0,
            3200.0,
            24.0,
            359.0,
            1.0,
            25.0,
            359.0,
        ),
        // The literal Kite-regression shape: unscrolled, so doc_top (41 —
        // TEXT_TOP's inset past the bar) sits well ABOVE menu_bottom (24).
        // The old formula would have chosen 41; the frame must still meet
        // the bar's bottom edge, not the text's own inset.
        (
            "unscrolled with menu bar",
            41.0,
            64.0,
            24.0,
            359.0,
            1.0,
            25.0,
            359.0,
        ),
        // DPI 2x: every input doubles (Wagtail's 2px stroke becomes 4 device
        // px), and the answer scales identically — the bound has no
        // DPI-specific branch, only the caller's already-scaled inputs.
        (
            "dpi 2x with menu bar",
            32.0,
            128.0,
            48.0,
            719.0,
            4.0,
            52.0,
            719.0,
        ),
        // Degenerate: the menu bar's reserve alone fills the canvas, so the
        // frame collapses to zero height rather than painting past the
        // last legal row.
        (
            "bar fills the canvas",
            0.0,
            0.0,
            359.0,
            359.0,
            4.0,
            359.0,
            359.0,
        ),
    ];
    for (name, doc_top, doc_height, menu_bottom, canvas_bottom, stroke_px, want_top, want_bottom) in
        cases
    {
        assert_eq!(
            page_frame_vertical_bounds(doc_top, doc_height, menu_bottom, canvas_bottom, stroke_px),
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
    // `text_top` is the document's own first-row Y — the text inset the
    // frame must NOT chase. `frame_top` is the canvas-owned boundary
    // [`crate::render::layers::page_frame_vertical_bounds`] actually
    // returns (no menu bar here, so it collapses to the stroke weight
    // itself); the two are deliberately different quantities now.
    let text_top = p.doc_top().max(0.0);
    let weight = 2.0f32;
    let frame_top = p.menubar_reserve() + weight;
    assert!(
        frame_top < text_top,
        "sanity: the frame must meet the canvas top ({frame_top}) well above the text \
         inset ({text_top}), or this law cannot distinguish the fix from the bug"
    );

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
    let mid_y = (text_top + LINE_HEIGHT * 0.5) as i64;
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
    let top_band_y = (frame_top - weight * 0.5).floor() as i64;
    assert_eq!(
        at(mid_x, top_band_y),
        white,
        "the frame's TOP edge band must be the pure ladder white at ({mid_x}, {top_band_y})"
    );
    // THE PREMISE-CHECK PIXEL: no menu bar, so the frame must touch the
    // canvas's very first legal row — row 0 — exactly, not merely land
    // somewhere above the text. This is the pixel the top-gap regression
    // left unframed.
    assert_eq!(
        at(mid_x, 0),
        white,
        "the frame's top edge must touch the canvas's first legal row (row 0) with no \
         menu bar drawn"
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
    let empty_row_y = (text_top + LINE_HEIGHT * 2.5) as i64;
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
    let reserve_f = p.menubar_reserve();
    let reserve = reserve_f as i64;
    let menu_left_band = (p.column_left() - weight * 0.5).floor() as i64;
    let (texture, tview) = offscreen(&device, 500, 360);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl page-frame menu-bar encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    let menu_pixels = read_pixels(&device, &queue, &texture, 500, 360);
    // THE DISCRIMINATING PIXEL for the drawn-menu-bar case: the CENTER of the
    // frame's top stroke band, `menubar_reserve() + weight/2`, sampled on the
    // full-width TOP EDGE (mid-column, same convention as the no-menu-bar
    // check above) rather than at a rail's corner — the reserve itself can
    // be fractional (a non-integer bar height), so sampling its exact
    // (truncated) row can land in the sub-pixel sliver the stroke doesn't
    // cover; the center is always solidly inside the 2px band. This is the
    // strip that used to sit unframed between the bar's bottom edge and the
    // scrolled document's own (now-irrelevant) text inset.
    // A fractional bar height means the bar's own antialiased bottom edge
    // can bleed a sliver into this exact row (byte-exact would be a claim
    // about the rasterizer, per this repo's own recorded lesson) — so this
    // checks PRESENCE (near-white, not the flat ground/bar color) rather
    // than bit-exact equality.
    let frame_touch_row = (reserve_f + weight * 0.5).floor() as i64;
    let touch_pixel = menu_pixels[(frame_touch_row * 500 + mid_x) as usize];
    assert!(
        touch_pixel.iter().take(3).all(|&c| c >= 250),
        "the frame must touch the first legal row below the drawn menu bar (row \
         {frame_touch_row}, reserve {reserve_f}): got {touch_pixel:?}"
    );
    assert_eq!(
        menu_pixels[((reserve + 4) * 500 + menu_left_band) as usize],
        white
    );
    assert_ne!(
        menu_pixels[((reserve - 1) * 500 + menu_left_band) as usize],
        white,
        "the frame must never paint through the drawn menu bar (row {})",
        reserve - 1
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

/// THE GENERALIZED SWEEP, roster-derived rather than pinned to Kite/Wagtail
/// by name (`CLAUDE.md`'s "derive the enrolment from the roster" law): every
/// world whose `PageFrame` is `Line` — currently Kite (1px) and Wagtail
/// (2px), automatically including any future addition — must have its top
/// edge and both rails touch the first legal canvas row, swept over page
/// width, viewport height, scroll position, and DPI, with and without the
/// drawn menu bar.
///
/// The "never paints through the drawn menu bar" half is proven at the PURE
/// bounds level (`page_frame_vertical_bounds_cover_short_tall_scrolled_and_menu_bar_cases`,
/// `top - stroke_px == menubar_bottom` exactly, by construction) rather than
/// here: the frame draws before chrome, so a pixel sample above the bar only
/// proves the bar paints over any bleed, not that the frame stayed clear of
/// it.
#[test]
fn page_frame_top_meets_canvas_or_bar_across_line_worlds_and_configs() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(500.0, 600.0) else {
        eprintln!(
            "skipping page_frame_top_meets_canvas_or_bar_across_line_worlds_and_configs: \
             no wgpu adapter"
        );
        return;
    };
    let was_menu_bar_on = crate::menubar::menu_bar_on();

    let line_worlds: Vec<(&str, f32)> = theme::THEMES
        .iter()
        .filter_map(|w| match w.render_caps.page_frame {
            theme::PageFrame::Line { weight_px } => Some((w.name, weight_px.0)),
            theme::PageFrame::None => None,
        })
        .collect();
    assert!(
        line_worlds.len() >= 2,
        "sweep must enroll at least the two known PageFrame::Line worlds (Kite, Wagtail); \
         got {line_worlds:?} — an enrolment predicate that matches zero or one world proves nothing"
    );

    let measures = [20usize, 48usize];
    let heights = [300u32, 600u32];
    let scrolls = [0usize, 40usize];
    let dpis = [1.0f32, 1.5f32];
    let menu_bars = [false, true];
    let tall = (0..200).map(|_| "line").collect::<Vec<_>>().join("\n");

    // A boundary row straddling the frame and the drawn menu bar can carry a
    // small blend from whatever the bar draws at its own bottom edge — real,
    // measured values landed within 7 of the exact ink on a 1px stroke at a
    // fractional device-pixel offset. 12 stays far below the ~200-unit
    // distance from either the bar's or the page's own ground color, so it
    // cannot mistake "gap" for "touch".
    let near = |a: [u8; 4], b: [u8; 3]| {
        a.iter()
            .take(3)
            .zip(b.iter())
            .all(|(&x, &y)| (x as i32 - y as i32).abs() <= 12)
    };

    let mut cells = 0usize;
    for &(world_name, weight_logical) in &line_worlds {
        theme::set_active_by_name(world_name).unwrap();
        p.sync_theme();
        let ink = theme::page_frame_ink().rgb_bytes();
        for &measure in &measures {
            crate::page::set_measure(measure);
            crate::page::set_page_on(true);
            for &dpi in &dpis {
                p.set_dpi(dpi);
                for &menu_bar in &menu_bars {
                    crate::menubar::set_menu_bar_on(menu_bar);
                    for &h in &heights {
                        for &scroll_row in &scrolls {
                            let mut v = view(&tall, 0, 0);
                            if scroll_row > 0 {
                                v.scroll = ScrollPos::at_row(scroll_row);
                            }
                            p.set_view(&v);
                            p.prepare(&device, &queue, 500, h).unwrap();

                            let weight = p
                                .metrics
                                .px(crate::render::Logical(weight_logical))
                                .max(0.1);
                            let reserve = p.menubar_reserve();
                            let left = p.column_left();
                            let colw = p.column_width();
                            let mid_x = (left + colw * 0.5) as i64;
                            let left_band_x = (left - weight * 0.5).floor() as i64;
                            let right_band_x = (left + colw + weight * 0.5).floor() as i64;

                            let (texture, tview) = offscreen(&device, 500, h);
                            let mut encoder =
                                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                    label: Some("awl page-frame sweep encoder"),
                                });
                            p.render(&mut encoder, &tview).unwrap();
                            queue.submit(Some(encoder.finish()));
                            let pixels = read_pixels(&device, &queue, &texture, 500, h);

                            let label = format!(
                                "{world_name} measure={measure} h={h} scroll={scroll_row} \
                                 dpi={dpi} menu_bar={menu_bar}"
                            );
                            // A thin (1px) stroke at a fractional device-pixel
                            // offset can put its one fully-covered row on
                            // either side of a naive `reserve + weight/2`
                            // rounding, so scan a small window around the
                            // stroke's own analytic band rather than
                            // predicting the rasterizer's exact pixel-center
                            // rule. The window is deliberately TIGHT — a few
                            // pixels either side of the stroke — so the old
                            // top-gap regression (ink starting ~TEXT_TOP
                            // logical px further down) cannot satisfy it.
                            let scan_lo = (reserve.floor() as i64 - 1).max(0);
                            let scan_hi = ((reserve + weight).ceil() as i64 + 1).min(h as i64 - 1);
                            assert!(
                                scan_lo <= scan_hi,
                                "{label}: degenerate scan window [{scan_lo},{scan_hi}] for \
                                 reserve={reserve} weight={weight}"
                            );
                            for (what, x) in [
                                ("top edge", mid_x),
                                ("left rail", left_band_x),
                                ("right rail", right_band_x),
                            ] {
                                assert!(
                                    (0..500).contains(&x),
                                    "{label}: {what} sample x={x} fell off the 500px-wide canvas"
                                );
                                let first_ink_row = (scan_lo..=scan_hi)
                                    .find(|&y| near(pixels[(y * 500 + x) as usize], ink));
                                let Some(row) = first_ink_row else {
                                    panic!(
                                        "{label}: {what} never touches the first legal canvas \
                                         row — no ink found in rows {scan_lo}..={scan_hi} at \
                                         x={x} (reserve={reserve}, weight={weight})"
                                    );
                                };
                                assert!(
                                    (row as f32) <= reserve + weight + 1.0,
                                    "{label}: {what}'s first ink row ({row}) sits farther from \
                                     the canvas/bar top (reserve={reserve}) than the stroke's \
                                     own band — this is the shape of the old top-gap regression"
                                );
                            }
                            cells += 1;
                        }
                    }
                }
            }
        }
    }
    let expected_cells = line_worlds.len()
        * measures.len()
        * heights.len()
        * scrolls.len()
        * dpis.len()
        * menu_bars.len();
    assert_eq!(cells, expected_cells, "sweep must cover every cell");

    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(was_menu_bar_on);
}

/// THE IDENTITY LAW: the document's own first text row — `doc_top()` — must
/// not move by even one device pixel when the page-frame's stroke weight
/// differs (Kite 1px vs. Wagtail 2px), and must match the SAME formula
/// (`TEXT_TOP*scale + menubar_reserve() - scroll`) the frame bound no longer
/// participates in. If a future edit "fixed" the top-gap regression by
/// moving `doc_top` upward instead of correcting the frame's own bound, this
/// law is what would catch it: the frame bound's dependency on `doc_top` was
/// removed entirely by this fix, and this proves the removal did not become
/// a dependency running the other way.
#[test]
fn page_frame_stroke_weight_never_moves_the_first_text_row() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let Some((device, queue, mut p)) = headless_dqp(500.0, 360.0) else {
        eprintln!(
            "skipping page_frame_stroke_weight_never_moves_the_first_text_row: no wgpu adapter"
        );
        return;
    };
    let was_menu_bar_on = crate::menubar::menu_bar_on();
    crate::page::set_measure(24);
    crate::page::set_page_on(true);

    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);

    for &menu_bar in &[false, true] {
        crate::menubar::set_menu_bar_on(menu_bar);

        theme::set_active_by_name("Kite").unwrap(); // 1px frame
        p.sync_theme();
        p.prepare(&device, &queue, 500, 360).unwrap();
        let kite_doc_top = p.doc_top();

        theme::set_active_by_name("Wagtail").unwrap(); // 2px frame
        p.sync_theme();
        p.prepare(&device, &queue, 500, 360).unwrap();
        let wagtail_doc_top = p.doc_top();

        assert_eq!(
            kite_doc_top, wagtail_doc_top,
            "menu_bar={menu_bar}: the text row's own Y must not move when the frame's \
             stroke weight differs (1px vs 2px)"
        );
        let expected = p.metrics.px(TEXT_TOP) + p.menubar_reserve();
        assert_eq!(
            wagtail_doc_top, expected,
            "menu_bar={menu_bar}: doc_top stays TEXT_TOP*scale + menubar_reserve(), \
             independent of the page-frame top-gap fix"
        );
    }

    crate::menubar::set_menu_bar_on(was_menu_bar_on);
}
