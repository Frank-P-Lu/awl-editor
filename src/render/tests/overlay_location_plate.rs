//! A LOCATION ROW THAT PLANS NO GLYPHS GETS NO PLATE EITHER.
//!
//! **Defect (exposed, not introduced, by the rotated-rail location cue):** a
//! `Bars` card's own LOCATION line (`PlanLine::Location`, the second-level
//! heading above the candidate rows) used to shape real inline text on every
//! world. Once `LocationStyle::RotatedRail` (Cassowary) moved that text
//! off-card into the room's own margin, the line stayed glyph-free — but the
//! per-row plate loop in
//! `overlay_selection.rs::overlay_unselected_bar_rects` kept backing EVERY
//! `item.is_none()` row (header or location) with a plate regardless, so the
//! location row drew a visibly empty rounded chip.
//!
//! **The owner:** `overlay_unselected_bar_rects` now reads the SAME gate the
//! shaper itself reads (`LocationStyle::draws_inline()`) before pushing a
//! plate for a location row — never a named-world check, so a future
//! non-inline style loses its chip for free and a future inline one keeps it.
//!
//! **Two arms, over the plate-drawing roster × every faceting `OverlayKind` ×
//! both DPI tiers:**
//!
//! - **THE QUAD ARM** — the one production row-surface owner,
//!   `overlay_row_surfaces_probe`, must emit NO rect over the location row's
//!   own Y-slot on a non-inline world (earned by measurement, not a name
//!   list), and MUST emit one there on an inline world — the non-vacuity half,
//!   proving the row legitimately gets a plate when nothing excludes it.
//! - **THE PIXEL ARM** — real GPU pixels. On the excluded world the location
//!   row's own slot reads as plain card ground (matches the gap directly
//!   above it, the same "ground" this tree's other presence floors use); on
//!   an included world it does not.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::overlay::OverlayKind;

const TIERS: [(u32, u32, f32); 2] = [(1200, 800, 1.0), (2400, 1600, 2.0)];

/// The plate-drawing `Bars` roster — the same derivation `overlay_plan_law`
/// uses, asserted equal so the two files cannot silently enroll different
/// sets.
fn plated_roster() -> Vec<&'static str> {
    theme::THEMES
        .iter()
        .filter(|t| {
            t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates
                && t.render_caps.list_style.draws_row_plates()
        })
        .map(|t| t.name)
        .collect()
}

/// A faceted card of `kind` at its first place-naming lens, real content, one
/// location line — `rotated_rail.rs`'s own `faceted_view` shape.
fn faceted_view(kind: OverlayKind) -> Option<ViewState> {
    let scheme = crate::facets::scheme(kind)?;
    let lens = (1..scheme.strip.len()).find(|&i| scheme.location(i).is_some())?;
    let label = scheme.location(lens)?;
    let items: Vec<String> = match kind {
        OverlayKind::Command => crate::commands::names(),
        _ => (0..8).map(|k| format!("item-{k}.md")).collect(),
    };
    let n = items.len();
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = items;
    v.overlay_bindings = vec![String::new(); n];
    v.overlay_lens = scheme.strip_labels(lens);
    v.overlay_sections = vec![label.to_string(); n];
    v.overlay_location = Some(label.to_string());
    v.overlay_selected = 0;
    Some(v)
}

fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl location-plate encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

#[test]
fn a_glyph_free_location_row_draws_no_plate_and_an_inline_one_still_does() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_glyph_free_location_row_draws_no_plate: no wgpu adapter");
        return;
    };

    let plated = plated_roster();
    assert_eq!(
        plated,
        ["Galah", "Firetail", "Cassowary"],
        "the plate-drawing `Bars` roster moved — this law's enrolment must move with it"
    );

    let entry_world = theme::active_index();
    let mut cells_checked = 0usize;
    let mut inline_seen = false;
    let mut off_card_seen = false;
    for world in &plated {
        theme::set_active_by_name(world).unwrap();
        let location_inline = theme::active().render_caps.location_style.draws_inline();
        for kind in OverlayKind::ALL {
            let Some(v) = faceted_view(kind) else {
                continue;
            };
            for &(w, h, scale) in &TIERS {
                p.sync_theme();
                p.set_view(&v);
                p.set_size(w as f32, h as f32);
                p.set_dpi(scale);
                p.prepare(&device, &queue, w, h).unwrap();

                let geom = p.overlay_geometry(w);
                let Some(display) = geom.plan_location_row_display() else {
                    continue; // this kind's real geometry plans no location line (e.g. a workspace)
                };
                let plan = p.overlay_row_plan(&geom);
                let row = *plan
                    .rows()
                    .iter()
                    .find(|r| r.display == display)
                    .unwrap_or_else(|| {
                        panic!("{world}/{kind:?}@{scale}: no planned row at display {display}")
                    });
                cells_checked += 1;

                // ARM 1 — THE QUADS. Y-only: rows stack without vertical
                // overlap by construction, so a rect sharing this row's Y-slot
                // is a claim about THIS row regardless of its X span.
                let surfaces = p.overlay_row_surfaces_probe();
                let (lo, hi) = (row.top + 1.0, row.bottom() - 1.0);
                let overlapping = surfaces
                    .iter()
                    .filter(|r| r[1] < hi && r[1] + r[3] > lo)
                    .count();
                if location_inline {
                    inline_seen = true;
                    assert!(
                        overlapping > 0,
                        "{world}/{kind:?}@{scale}: an INLINE location row drew no plate at \
                         all (slot y {lo:.1}..{hi:.1}) — this arm is vacuous unless a real \
                         world still draws one here"
                    );
                } else {
                    off_card_seen = true;
                    assert_eq!(
                        overlapping, 0,
                        "{world}/{kind:?}@{scale}: a glyph-free location row (slot y \
                         {lo:.1}..{hi:.1}) still has {overlapping} row surface(s) drawn over \
                         it — the empty chip item 316 was filed against"
                    );

                    // ARM 2 — THE PIXELS, on the excluded world only (the only
                    // case where "reads as ground" is the claim).
                    let pixels = shoot(&device, &queue, &mut p, w, h);
                    assert_row_reads_as_ground(
                        &pixels,
                        &plan,
                        row,
                        w,
                        h,
                        &format!("{world}/{kind:?}@{scale}"),
                    );
                }
            }
        }
    }
    theme::set_active(entry_world);
    assert!(
        cells_checked > 0,
        "the sweep reached no faceted card at all"
    );
    assert!(
        inline_seen,
        "no INLINE-style world was ever swept — arm 1's non-vacuity is untested"
    );
    assert!(
        off_card_seen,
        "no non-inline world was ever swept — the defect's own case is untested"
    );
}

/// ARM 2's own claim: `row`'s slot reads as plain card ground rather than as
/// a plate. Reference: the gap directly above the row's own slot — nothing
/// else draws there, so it is genuine card ground.
fn assert_row_reads_as_ground(
    pixels: &[[u8; 4]],
    plan: &crate::render::plan::OverlayRowPlan,
    row: crate::render::plan::PlannedRow,
    w: u32,
    h: u32,
    ctx: &str,
) {
    let (x0, x1) = plan.card_x_span();
    let pad = (row.height * 0.2).max(2.0);
    let ground = median_of(
        pixels,
        x0 + 4.0,
        row.top - pad - 4.0,
        x1 - 4.0,
        row.top - 2.0,
        w,
        h,
    );
    let slot = median_of(
        pixels,
        x0 + 4.0,
        row.top + pad,
        x1 - 4.0,
        row.bottom() - pad,
        w,
        h,
    );
    let presence = pixeldiff::delta_e(slot, ground);
    assert!(
        presence < 1.0,
        "{ctx}: the location row's own slot (colour {slot:?}) reads ΔE {presence:.2} from the \
         card ground just above it (colour {ground:?}) — something still draws there"
    );
}

/// A rect's median colour, in device px — robust to the minority of pixels a
/// glyph or a rounded corner contributes (`overlay_plan_law`'s own doc has the
/// full reasoning for median over mean here).
fn median_of(pixels: &[[u8; 4]], x0: f32, y0: f32, x1: f32, y1: f32, w: u32, h: u32) -> [u8; 4] {
    let luma = |px: [u8; 4]| -> f64 {
        0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
    };
    let (a, b) = (
        y0.ceil().max(0.0) as u32,
        y1.floor().min(h as f32 - 1.0) as u32,
    );
    let (c, d) = (
        x0.ceil().max(0.0) as u32,
        x1.floor().min(w as f32 - 1.0) as u32,
    );
    let mut v: Vec<[u8; 4]> = (a..b)
        .flat_map(|y| (c..d).map(move |x| pixels[(y * w + x) as usize]))
        .collect();
    assert!(
        !v.is_empty(),
        "empty sample band x {x0:.1}..{x1:.1} y {y0:.1}..{y1:.1}"
    );
    v.sort_by(|p, q| luma(*p).partial_cmp(&luma(*q)).expect("no NaN luminance"));
    v[v.len() / 2]
}
