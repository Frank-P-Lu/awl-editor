//! THE STRAY BLANK BAND ABOVE A PICKER'S CONTENT.
//!
//! A flat picker's query BEAT was folded into the query field's own line box.
//! cosmic-text CENTRES a line's glyph run in its box, so the field's glyphs were
//! drawn half a beat below the bar's own top pad and the bar opened a blank strip
//! above them. Reported against five worlds; present, identically, on all twenty
//! — which is why this sweeps the roster and not the report.
//!
//! The appearance claim is arithmetic over real rendered pixels, and the oracle
//! is DIFFERENTIAL: the same card is shot twice, with and without query text, so
//! card texture, placards, backdrops, the world's own ground and the dither all
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
        label: Some("awl query-field surfaces encoder"),
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
    (w, h): (u32, u32),
    (x0, x1): (i32, i32),
    (y0, y1): (i32, i32),
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

/// **THE QUERY FIELD'S INK RIDES ITS OWN LINE, ON EVERY WORLD.**
///
/// The claim is stated against the ONE thing a bar's composition depends on: the
/// field's drawn ink must centre where a plain row's ink would, one half-pitch
/// below the card's own `text_top`. A beat folded into the field's box does not
/// sit below the field — it is split around it, and the half above is the blank
/// strip this law forbids.
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
                v.overlay_title = OverlayKind::Theme.title().to_string();
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
                    (cw, ch),
                    (cx0.round() as i32, cx1.round() as i32),
                    (geom.card_y.round() as i32, plan.first_top().round() as i32),
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
