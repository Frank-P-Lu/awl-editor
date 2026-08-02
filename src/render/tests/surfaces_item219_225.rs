//! ITEMS 219 & 225 — TWO UNINTENDED SURFACES, THE SAME SHAPE OF MISTAKE.
//!
//! Both defects are a CHROME BAND sized from the row pitch (or from a card edge
//! that isn't there) instead of from the ink it backs, and both were reported
//! against the worlds where the mis-sized band happens to be most visible rather
//! than against the worlds that have it. So both laws sweep the whole roster:
//!
//! * **219** — a flat picker's query BEAT was folded into the query field's own
//!   line box. cosmic-text CENTRES a line's glyph run in its box, so the field's
//!   glyphs were drawn half a beat below the bar's own top pad and the bar opened
//!   a blank strip above them.
//! * **225** — a summoned WORKSPACE's `Bars` footer plate ran to the card's
//!   bottom edge. That is right for a card that hugs its content — the plate
//!   closes the card — but a workspace's card comes from the CANVAS, so the same
//!   rule painted a slab as tall as whatever vertical space the rows did not use.
//!
//! Every appearance claim here is arithmetic over real rendered pixels; the
//! query-ink oracle is DIFFERENTIAL (the same card shot twice, with and without
//! query text), so card texture, placards, backdrops and the world's own ground
//! cancel exactly and what is left is the field's own glyphs.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// Render the current frame offscreen and read it back.
fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item219 surfaces encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

/// The inclusive `y` band over which two frames of the SAME card differ, inside
/// `[x0, x1)` — with a query typed into one and not the other, that band IS the
/// query field's own drawn ink, with every world-specific surface cancelled.
fn differing_y_band(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    w: u32,
    h: u32,
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32,
) -> Option<(i32, i32)> {
    let mut top = i32::MAX;
    let mut bottom = i32::MIN;
    for y in y0.max(0)..y1.min(h as i32) {
        for x in x0.max(0)..x1.min(w as i32) {
            let i = y as usize * w as usize + x as usize;
            let (p, q) = (a[i], b[i]);
            let d = (0..3)
                .map(|k| (p[k] as i16 - q[k] as i16).unsigned_abs())
                .max()
                .unwrap_or(0);
            if d > 24 {
                top = top.min(y);
                bottom = bottom.max(y);
                break;
            }
        }
    }
    (top <= bottom).then_some((top, bottom))
}

/// **ITEM 219 — THE QUERY FIELD'S INK RIDES ITS OWN LINE, ON EVERY WORLD.**
///
/// The claim is stated against the ONE thing a bar's composition depends on: the
/// field's drawn ink must centre where a plain row's ink would, one half-pitch
/// below the card's own `text_top`. A beat folded into the field's box does not
/// sit below the field — it is split around it, and the half above is the blank
/// strip item 219 names.
///
/// THE ORACLE IS REAL PIXELS AND IT IS DIFFERENTIAL: the same picker is shot
/// with an empty query and with one typed, and the rows that differ are the
/// field's own glyphs. Ground, gradient, card texture, placard wordmark and
/// dither cancel exactly — the arithmetic sees the ink and nothing else.
///
/// SWEPT over the whole world roster × two canvases × 1x/2x DPI, because the
/// displacement is `header_gap / 2` and therefore scales with the row pitch: a
/// law that graded one world at one size is exactly the law that would have gone
/// green on this for as long as the fold has existed.
#[test]
fn a_flat_pickers_query_ink_centres_on_its_own_line_in_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_flat_pickers_query_ink_centres_on_its_own_line: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    let mut worst = 0.0f32;
    let mut worst_ctx = String::new();
    let mut worst_folded = 0.0f32;

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for (lw, lh_px) in [(1200.0f32, 800.0f32), (900.0, 620.0)] {
            let (cw, ch) = ((lw * dpi) as u32, (lh_px * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for world in crate::theme::world_names() {
                theme::set_active_by_name(world).unwrap();
                p.sync_theme();

                let mut v = view("hello world\n", 0, 0);
                v.overlay_active = true;
                v.overlay_title = OverlayKind::Theme.title();
                v.overlay_items = vec!["Alpha".into(), "Beta".into(), "Gamma".into()];
                v.overlay_selected = 0;
                v.overlay_hint = OverlayKind::Theme.hint();
                v.overlay_window_rows = OverlayKind::Theme.window_rows();

                // FRAME A — no query typed.
                v.overlay_query = String::new();
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let row_h = plan.lh();
                let (cx0, cx1) = plan.card_x_span();
                let frame_a = shoot(&device, &queue, &mut p, cw, ch);

                // FRAME B — the SAME card with query text in the field.
                v.overlay_query = "MMMMMM".into();
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let frame_b = shoot(&device, &queue, &mut p, cw, ch);

                let ctx = format!("{world} {lw}x{lh_px}@{dpi}");
                let band = differing_y_band(
                    &frame_a,
                    &frame_b,
                    cw,
                    ch,
                    cx0.round() as i32,
                    cx1.round() as i32,
                    geom.card_y.round() as i32,
                    plan.first_top().round() as i32,
                );
                let (ink_top, ink_bottom) = band.unwrap_or_else(|| {
                    panic!(
                        "{ctx}: typing into the query changed no pixel between the card top \
                         and the first candidate row — the differential oracle is blind and \
                         this cell would grade nothing"
                    )
                });
                let ink_center = (ink_top + ink_bottom) as f32 * 0.5;

                // --- THE CLAIM: the ink rides its own line's centre ------------
                let want = geom.text_top + row_h * 0.5;
                let off = (ink_center - want).abs();
                let bound = row_h * 0.25;
                assert!(
                    off <= bound,
                    "{ctx}: the query's drawn ink centres at {ink_center:.1}, but its own \
                     line box centres at {want:.1} ({off:.1}px off, bound {bound:.1}px) — \
                     the beat has been folded into the field again and the bar carries a \
                     blank strip above its own text"
                );
                if off > worst {
                    worst = off;
                    worst_ctx = ctx.clone();
                }

                // --- NON-VACUITY: the retired fold, written out here -----------
                // A field inflated to `lh + header_gap` centres its glyph run at
                // the midpoint of THAT box; the displacement is the distance this
                // law would then have to accept.
                let folded_center = geom.text_top + (row_h + p.overlay_header_gap()) * 0.5;
                worst_folded = worst_folded.max((folded_center - want).abs());

                // --- AND THE BAR CLOSES BELOW IT ------------------------------
                // On a world that really draws two surfaces, the field's ink must
                // be wholly inside the upper one, with comparable air above and
                // below: that is the composition the blank strip broke.
                let fills = p.overlay_pane_fills_probe();
                if fills.len() == 2 {
                    let bar = fills[0];
                    let (bar_top, bar_bottom) = (bar[1], bar[1] + bar[3]);
                    assert!(
                        ink_top as f32 > bar_top && (ink_bottom as f32) < bar_bottom,
                        "{ctx}: the query's ink [{ink_top}, {ink_bottom}] is not inside its \
                         own drawn bar [{bar_top:.1}, {bar_bottom:.1}]"
                    );
                    let air_above = ink_top as f32 - bar_top;
                    let air_below = bar_bottom - ink_bottom as f32;
                    assert!(
                        (air_above - air_below).abs() <= row_h * 0.35,
                        "{ctx}: the query bar is lopsided — {air_above:.1}px of air above \
                         its text and {air_below:.1}px below (bound {:.1}px)",
                        row_h * 0.35
                    );
                }
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        graded,
        crate::theme::THEMES.len() * 4,
        "every world must be graded at every canvas and DPI cell"
    );
    assert!(
        worst_folded > 12.0,
        "the retired fold's own displacement across the sweep was only \
         {worst_folded:.1}px — too small to be the reported blank band; the fixture \
         has stopped reproducing it"
    );
    assert!(
        worst < worst_folded * 0.5,
        "the graded worst case ({worst:.1}px, {worst_ctx}) is not comfortably clear of \
         the retired fold's own {worst_folded:.1}px — this law is not discriminating"
    );
}

/// **ITEM 225 — A WORKSPACE'S FOOTER PLATE ENDS WITH ITS FOOTER.**
///
/// A `Bars` world backs its footer line with a plate. On a card that HUGS its
/// content the plate runs to the card's bottom edge, and that is right: the plate
/// closes the card and the two are the same line. A summoned WORKSPACE's card
/// comes from the CANVAS, so there is no bottom edge to close — the same rule
/// paints a slab as tall as whatever vertical space the rows did not use, hanging
/// below the footer's own glyphs with nothing in it. On Cassowary, whose plate
/// ink is very nearly black, that slab is the reported "oversized black
/// sub-settings bar"; on the four other bare-plate worlds it is the same slab in
/// a paler ink, which is why this sweeps the roster and not the report.
///
/// TWO ARMS, over the WHOLE shipping bare-plate roster:
///
/// 1. GEOMETRY, from the quads the emitter actually produces: no plate a
///    workspace draws may be taller than one row of its own list — and the
///    footer's own plate must sit at the planned footer top, so the arm is
///    watching the plate it names.
/// 2. APPEARANCE, from the frame's own pixels: the band the retired rule would
///    have painted — reconstructed INLINE from that rule, never read back out of
///    the fix — must now read as workspace card ground.
#[test]
fn a_workspace_footer_plate_ends_with_its_footer_on_every_bare_plate_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_workspace_footer_plate_ends_with_its_footer: no wgpu adapter");
        return;
    };
    let bare_plate_worlds: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates)
        .map(|t| t.name)
        .collect();
    assert_eq!(
        bare_plate_worlds,
        ["Mangrove", "Galah", "Magpie", "Firetail", "Cassowary"],
        "the bare-plate law must follow the exact shipping roster"
    );

    let mut pixel_graded: Vec<String> = Vec::new();
    let mut retired_overrun = 0.0f32;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for world in &bare_plate_worlds {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();

            // A SETTINGS workspace with a SHORT list, so the card's canvas-sized
            // box leaves a great deal of room the rows do not use — the exact
            // condition under which the retired rule painted its slab.
            let mut v = view("hello world\n", 0, 0);
            v.overlay_active = true;
            v.overlay_workspace = true;
            v.overlay_rows_primary = false;
            v.overlay_title = OverlayKind::Settings.title();
            v.overlay_items = vec!["Alpha".into(), "Beta".into()];
            v.overlay_selected = 0;
            v.overlay_hint = "↑/↓ category   ↵ settings   esc close".into();
            v.overlay_lens = vec![("All".into(), true), ("Editor".into(), false)];
            p.set_view(&v);
            p.prepare(&device, &queue, cw, ch).unwrap();

            let geom = p.overlay_geometry(cw);
            let plan = p.overlay_row_plan(&geom);
            let row_h = plan.lh();
            let footer_top = plan.footer_top();
            let ctx = format!("{world}@{dpi}");
            assert!(
                geom.card_y + geom.card_h > footer_top + 4.0 * row_h,
                "{ctx}: the fixture must leave real unused space below the footer, or \
                 neither arm can see the defect"
            );

            // --- ARM 1: THE EMITTED QUADS ---------------------------------
            let (sel, unsel) = p.overlay_bar_rects_probe();
            let plates: Vec<[f32; 4]> = sel.into_iter().chain(unsel).collect();
            let footer_plate = plates
                .iter()
                .copied()
                .find(|r| (r[1] - footer_top).abs() < row_h * 0.5)
                .unwrap_or_else(|| {
                    panic!(
                        "{ctx}: no drawn plate sits at the planned footer top \
                         {footer_top:.1} (plates {plates:?}) — arm 1 must be watching a \
                         real plate"
                    )
                });
            assert!(
                footer_plate[3] <= row_h,
                "{ctx}: the workspace's footer plate is {:.1}px tall, taller than one row \
                 of its own list ({row_h:.1}px) — it is painting the space the rows did \
                 not use, not backing the footer",
                footer_plate[3]
            );

            // THE RETIRED RULE, written out here rather than called: the plate
            // ran to `footer_top + rows * lh + WORKSPACE_PAD`, bounded only by a
            // card bottom a workspace does not have.
            let retired_bottom = footer_top
                + (geom.hint_rows_probe() + geom.footer_rows_probe()) as f32 * row_h
                + super::super::chrome::WORKSPACE_PAD_PROBE;
            let plate_bottom = footer_plate[1] + footer_plate[3];
            retired_overrun = retired_overrun.max(retired_bottom - plate_bottom);
            assert!(
                retired_bottom > plate_bottom + 2.0,
                "{ctx}: the retired rule would have ended at {retired_bottom:.1} and the \
                 fix ends at {plate_bottom:.1} — this cell no longer reproduces the defect"
            );

            // --- ARM 2: THE PIXELS ----------------------------------------
            // The band the retired rule would have painted must now read as the
            // workspace's own ground. Reference ground is taken from the same
            // band, just RIGHT of the plate (the plates hug their labels), so the
            // comparison is within one horizontal line of the same surface.
            let pixels = shoot(&device, &queue, &mut p, cw, ch);
            let at = |x: i32, y: i32| -> [u8; 4] {
                pixels[(y.clamp(0, ch as i32 - 1)) as usize * cw as usize
                    + (x.clamp(0, cw as i32 - 1)) as usize]
            };
            let luma = |c: [u8; 4]| {
                0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
            };
            let inside_x = (footer_plate[0] + footer_plate[2] * 0.5).round() as i32;
            let right_x = (footer_plate[0] + footer_plate[2] + 8.0 * dpi).round() as i32;
            let plate_y = (footer_plate[1] + footer_plate[3] * 0.5).round() as i32;
            let plate_luma = luma(at(inside_x, plate_y));
            let ground_luma = luma(at(right_x, plate_y));
            // Only grade worlds whose plate an absolute oracle can genuinely see.
            if (plate_luma - ground_luma).abs() < 6.0 {
                continue;
            }
            let probe_y = ((plate_bottom + retired_bottom) * 0.5).round() as i32;
            let below = luma(at(inside_x, probe_y));
            let below_ground = luma(at(right_x, probe_y));
            assert!(
                (below - below_ground).abs() < (plate_luma - ground_luma).abs() * 0.35,
                "{ctx}: the band the retired rule painted (y={probe_y}) still reads as \
                 plate, not as workspace ground — luma {below:.1} against the ground's \
                 {below_ground:.1}, with the plate itself {plate_luma:.1} against \
                 {ground_luma:.1}"
            );
            pixel_graded.push(ctx);
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        pixel_graded.len() >= 4,
        "the appearance arm graded only {pixel_graded:?} — too few visible plates for \
         the pixels to carry any world"
    );
    assert!(
        retired_overrun > 10.0,
        "the retired rule's worst overrun across the sweep was only \
         {retired_overrun:.1}px — too small to be the reported bar"
    );
}
