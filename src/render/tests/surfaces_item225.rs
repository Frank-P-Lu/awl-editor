//! ITEM 225 — A WORKSPACE'S OVERSIZED FOOTER PLATE.
//!
//! A `Bars` world backs its footer line with a plate. On a card that HUGS its
//! content the plate runs to the card's bottom edge, and that is right: the plate
//! closes the card and the two are the same line. A summoned WORKSPACE's card
//! comes from the CANVAS, so there is no bottom edge to close — the same rule
//! paints a slab as tall as whatever vertical space the rows did not use, hanging
//! below the footer's own glyphs with nothing in it. On Cassowary, whose plate
//! ink is very nearly black, that slab is the reported "oversized black
//! sub-settings bar"; on the two other PLATE-DRAWING worlds it is the same slab
//! in a paler ink, which is why this sweeps a roster and not the report.

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

/// **ITEM 225 — A WORKSPACE'S FOOTER PLATE ENDS WITH ITS FOOTER.**
///
/// A `Bars` world backs its footer line with a plate. On a card that HUGS its
/// content the plate runs to the card's bottom edge, and that is right: the plate
/// closes the card and the two are the same line. A summoned WORKSPACE's card
/// comes from the CANVAS, so there is no bottom edge to close — the same rule
/// paints a slab as tall as whatever vertical space the rows did not use, hanging
/// below the footer's own glyphs with nothing in it. On Cassowary, whose plate
/// ink is very nearly black, that slab is the reported "oversized black
/// sub-settings bar"; on the two other PLATE-DRAWING worlds it is the same slab
/// in a paler ink, which is why this sweeps a roster and not the report.
///
/// THE ROSTER IS THE WORLDS THAT DRAW PLATES, WHICH IS NOT THE BARE-PLATE
/// ROSTER. `list_backing == BarePlates` is a claim about the CARD — no panel, no
/// shadow, no border — and not about rows. Two of its five members, Mangrove and
/// Magpie, are `ListStyle::Diagonal`: bare in exactly that sense and yet drawing
/// no plate anywhere, so a plate claim graded on them is a claim about nothing
/// and could go green over a real defect. The sweep asks `draws_row_plates`, and
/// arm 3 below EARNS the other two worlds' exclusion by measurement.
///
/// THREE ARMS:
///
/// 1. GEOMETRY, from the quads the emitter actually produces: no plate a
///    workspace draws may be taller than one row of its own list — and the
///    footer's own plate must sit at the planned footer top, so the arm is
///    watching the plate it names.
/// 2. APPEARANCE, from the frame's own pixels: the band the retired rule would
///    have painted — reconstructed INLINE from that rule, never read back out of
///    the fix — must now read as workspace card ground.
/// 3. THE EXCLUSION, on the bare-plate worlds arms 1 and 2 do not reach: the
///    frame must emit NO row surface at all. A world that starts drawing one
///    fails here and has to join the plated roster above.
#[test]
fn a_workspace_footer_plate_ends_with_its_footer_on_every_bare_plate_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping a_workspace_footer_plate_ends_with_its_footer: no wgpu adapter");
        return;
    };
    let (plated, plateless): (Vec<&'static str>, Vec<&'static str>) = theme::THEMES
        .iter()
        .filter(|t| t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates)
        .map(|t| (t.name, t.render_caps.list_style.draws_row_plates()))
        .fold((Vec::new(), Vec::new()), |mut acc, (name, draws)| {
            match draws {
                true => acc.0.push(name),
                false => acc.1.push(name),
            }
            acc
        });
    assert_eq!(
        (plated.as_slice(), plateless.as_slice()),
        (
            ["Galah", "Firetail", "Cassowary"].as_slice(),
            ["Mangrove", "Magpie", "Paperbark"].as_slice()
        ),
        "the shipping bare-plate roster splits exactly this way — a new world joins \
         one arm or the other, never neither"
    );
    let mut pixel_graded: Vec<String> = Vec::new();
    let mut plateless_graded: Vec<String> = Vec::new();
    let mut retired_overrun = 0.0f32;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for world in plated.iter().chain(&plateless) {
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

            // --- ARM 3: THE EXCLUSION -------------------------------------
            // A bare-plate world outside the plated roster is excused from arms
            // 1 and 2 for one reason only: nothing it draws can be a SLAB — a
            // surface that outlives the footer and paints the space the rows did
            // not use. Two shapes satisfy that and both are measured here, from
            // the same production emitter and at the same fixture rather than
            // taken on trust: a `Diagonal` world emits no row quad at all, and a
            // `Rules` world emits only rules — none of them a fraction as tall
            // as a row, and none of them reaching below the list.
            if !plated.contains(world) {
                let surfaces = p.overlay_row_surfaces_probe();
                let slab = surfaces
                    .iter()
                    .find(|r| r[3] >= row_h * 0.5 || r[1] + r[3] > footer_top + row_h + 1.0);
                assert!(
                    slab.is_none(),
                    "{ctx}: this world is excluded from the footer-plate arms because \
                     nothing it draws can outlive its footer, but the frame emitted \
                     {slab:?} — either as tall as half a row ({row_h:.1}px) or reaching \
                     past the footer band at {footer_top:.1}. Either it now draws plates, \
                     in which case it belongs in the plated roster and must be graded by \
                     arms 1 and 2, or the exclusion is wrong."
                );
                plateless_graded.push(ctx);
                continue;
            }

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
            // ITEM 293 — the footer plate now backs TWO compact rows, not one:
            // the hint's own line, plus the blank separator ahead of it
            // (`overlay_hint_gap_rows`, at its own `overlay_hint_gap_h`).
            // `overlay_footer_reclaim(1, 1)` is the SAME arithmetic the plate's
            // own height (`overlay_selection.rs`'s `footer_band`) comes from, so
            // this ceiling can't drift from the fix it is grading.
            let max_footer_plate_h = 2.0 * row_h - p.overlay_footer_reclaim(1, 1);
            assert!(
                footer_plate[3] <= max_footer_plate_h + 0.5,
                "{ctx}: the workspace's footer plate is {:.1}px tall, taller than its own \
                 two compact rows ({max_footer_plate_h:.1}px) — it is painting the space \
                 the rows did not use, not backing the footer",
                footer_plate[3]
            );

            // THE RETIRED RULE, written out here rather than called: the plate ran
            // to `footer_top + footer_rows * lh + WORKSPACE_PAD`, bounded only by
            // a card bottom a workspace does not have. This fixture draws exactly
            // one footer line and the workspace pad is 12.0 LOGICAL px — scaled by
            // `dpi` (zoom is 1.0 throughout this fixture), a pre-existing gap in
            // this reconstruction item 293's own taller (still bounded) plate is
            // the first thing to reach: at dpi 1 the unscaled literal is within
            // rounding of the true value, so it never mattered until now.
            let retired_bottom = footer_top + row_h + 12.0 * dpi;
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
            let luma =
                |c: [u8; 4]| 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
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
    assert_eq!(
        plateless_graded.len(),
        plateless.len() * 2,
        "the exclusion arm must reach every plateless world at both DPIs, got \
         {plateless_graded:?}"
    );
    // ITEM 293 lowered this floor from 10.0: its own (still bounded) plate is
    // legitimately taller than the one this law was first written against —
    // it now backs TWO compact rows (the hint plus the blank separator ahead
    // of it) instead of one, so the margin against the retired unbounded rule
    // shrinks even though the fix is unchanged in kind. Still comfortably
    // non-trivial, not a rounding coincidence.
    assert!(
        retired_overrun > 5.0,
        "the retired rule's worst overrun across the sweep was only \
         {retired_overrun:.1}px — too small to be the reported bar"
    );
}
