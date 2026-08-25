//! **THE DOCKED TAB'S MOUTH.** `FacetStyle::DockedTab` seats a strip line
//! above the card with the active label on its own plate — this law grades
//! whether that plate reads as a TAB joined to the card rather than a chip
//! floating a stroke's-width above it. Companion to `facet_strip_air.rs`'s
//! connection law, which grades the OTHER default (the strip sitting inside
//! the split lower surface); DockedTab deliberately moves its strip above the
//! pane and earns its own seam law here.
//!
//! Two floors, read from real pixels rather than the sidecar (a rect can be
//! reported correctly while nothing paints it — CAPTURE.md's own Wagtail
//! tripwire): on the ACTIVE facet's own column, at the row where the card's
//! border ring lives, no border-colored pixel survives (the MOUTH) and the
//! column really is painted the card's own ground (a PRESENCE floor a
//! deleted/blank tab could not also satisfy). On every OTHER column of that
//! same row — inactive facets, the gaps between them — the border reads
//! intact.
//!
//! Enrolment is derived from the roster (`render_caps.facet_style ==
//! FacetStyle::DockedTab`), never a named world, and the sweep covers every
//! position in the real Command-palette facet roster — not just the FIRST
//! one. The item this law exists for names that exact axis: a prior bug in
//! this same strip (the shaped-text buffer, not this plate) dropped its tail
//! on exactly a non-first active facet while a first-facet-only check stayed
//! green.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

fn command_view(active: usize) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands".to_string();
    v.overlay_items = (0..8).map(|i| format!("Command {i}")).collect();
    v.overlay_selected = 0;
    v.overlay_lens = crate::commands::COMMAND_FACETS.strip_labels(active);
    v
}

/// Every shipped world that draws the docked-active-facet TAB treatment.
/// Derived from the roster: a future world adopting `FacetStyle::DockedTab`
/// is swept here for free, and one that drops it stops being graded for
/// free, with no name to update either way.
fn docked_tab_roster() -> Vec<&'static theme::Theme> {
    theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.facet_style, theme::FacetStyle::DockedTab))
        .collect()
}

fn close_to(a: [u8; 4], b: [u8; 4], tol: u8) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| x.abs_diff(*y) <= tol)
}

/// One cell's identity, carried only to name a failure and to size the pixel
/// buffer being read.
struct SeamCell<'a> {
    world: &'a str,
    dpi: f32,
    active: usize,
    w: u32,
    h: u32,
}

/// One cell's worth of pixel arithmetic at the seam: the active facet's own
/// column carries no border pixel and real ground ink (the mouth), while
/// every other column inside the card still shows the border (intact
/// elsewhere).
fn assert_seam(cell: &SeamCell, card: [f32; 4], underline: [f32; 4], pixels: &[[u8; 4]]) {
    let SeamCell {
        world,
        dpi,
        active,
        w,
        h,
    } = *cell;
    let border = theme::surface_selected().rgba_bytes();
    let ground = theme::pane_surface(crate::render::effective_card_elevation()).rgba_bytes();

    // THE SEAM BAND: a few device rows immediately above the card's own top
    // edge, where the card's 1-device-px border ring (`chrome::
    // FLOAT_BORDER_RING_PX`, deliberately NOT dpi-scaled) and its
    // antialiased feather live. A band rather than one exact row so the
    // probe cannot miss the ring by a rounding disagreement with the
    // rasterizer.
    let y0 = (card[1] - 4.0).round().max(0.0) as i64;
    let y1 = card[1].round().max(y0 as f32 + 1.0) as i64;
    let px = |x: i64, y: i64| -> [u8; 4] {
        let xi = x.clamp(0, w as i64 - 1);
        let yi = y.clamp(0, h as i64 - 1);
        pixels[(yi * w as i64 + xi) as usize]
    };
    let any_border = |x: i64| (y0..y1).any(|y| close_to(px(x, y), border, 24));
    let any_ground = |x: i64| (y0..y1).all(|y| close_to(px(x, y), ground, 12));

    let tab_x0 = underline[0].round() as i64;
    let tab_x1 = (underline[0] + underline[2]).round() as i64;
    let card_x0 = card[0].round() as i64;
    let card_x1 = (card[0] + card[2]).round() as i64;
    assert!(
        tab_x1 > tab_x0,
        "{world}: active facet {active} has no width"
    );

    // THE MOUTH: no border-colored pixel crosses the active facet's own
    // column anywhere in the seam band.
    let mouth_border = (tab_x0..tab_x1).filter(|&x| any_border(x)).count();
    assert_eq!(
        mouth_border, 0,
        "{world}@{dpi}x: active facet {active} (x {tab_x0}..{tab_x1}) shows \
         {mouth_border} border-colored columns crossing its own seam band \
         y {y0}..{y1} — the tab's mouth is not continuous with the card"
    );

    // PRESENCE: the mouth reads as real card-ground ink for its whole band,
    // not as a gap a deleted tab would satisfy just as well.
    let mouth_ground = (tab_x0..tab_x1).filter(|&x| any_ground(x)).count();
    assert!(
        mouth_ground as i64 > (tab_x1 - tab_x0) / 2,
        "{world}@{dpi}x: active facet {active} shows only {mouth_ground}/{} \
         card-ground columns at its seam — the presence floor a blank tab \
         would also pass",
        tab_x1 - tab_x0
    );

    // INTACT ELSEWHERE: outside the active facet's own column but still
    // inside the card, the ordinary border ring still draws — inactive
    // facets and the gaps between tabs are untouched.
    let mut elsewhere_border = 0i64;
    let mut elsewhere_total = 0i64;
    for x in card_x0..card_x1 {
        if x >= tab_x0 && x < tab_x1 {
            continue;
        }
        elsewhere_total += 1;
        if any_border(x) {
            elsewhere_border += 1;
        }
    }
    assert!(
        elsewhere_border * 4 > elsewhere_total,
        "{world}@{dpi}x: active facet {active} — the border elsewhere on the \
         seam band reads broken ({elsewhere_border}/{elsewhere_total} columns)"
    );
}

#[test]
fn the_active_facets_plate_joins_the_card_ground_with_no_border_in_its_mouth() {
    let _guard = crate::testlock::serial();

    let roster = docked_tab_roster();
    assert!(
        !roster.is_empty(),
        "no world carries FacetStyle::DockedTab — this law would run vacuously"
    );

    let n_facets = crate::commands::COMMAND_FACETS.strip_labels(0).len();
    assert!(
        n_facets >= 3,
        "need at least a first (index 0) and a non-first active facet to sweep the axis"
    );

    let mut cells = 0usize;
    for world in &roster {
        let _pin = theme::WorldPin::world(world.name)
            .unwrap_or_else(|| panic!("{} is in the authored roster", world.name));

        for dpi in [1.0f32, 2.0f32] {
            let (logical_w, logical_h) = (1200.0f32, 800.0f32);
            let (w, h) = ((logical_w * dpi) as u32, (logical_h * dpi) as u32);
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!(
                    "skipping the_active_facets_plate_joins_the_card_ground: no wgpu adapter"
                );
                return;
            };
            p.set_dpi(dpi);

            for active in 0..n_facets {
                let v = command_view(active);
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();

                let card = p
                    .overlay_card_rect()
                    .unwrap_or_else(|| panic!("{}: DockedTab draws a Pane card", world.name));
                let underline = p.overlay_theme_underline.unwrap_or_else(|| {
                    panic!("{}: active facet {active} draws no tab plate", world.name)
                });

                let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                let cell = SeamCell {
                    world: world.name,
                    dpi,
                    active,
                    w,
                    h,
                };
                assert_seam(&cell, card, underline, &pixels);

                cells += 1;
            }
        }
    }
    assert_eq!(
        cells,
        roster.len() * n_facets * 2,
        "every DockedTab world x every facet position x both DPI tiers must be \
         graded — first (index 0) and at least one non-first facet are both \
         in this sweep"
    );
}
