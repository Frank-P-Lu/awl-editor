//! **THE FIND/REPLACE PANEL'S BORDERED CONTROLS ARE PRESENT AND VISIBLE, ACROSS
//! THE WHOLE WORLD ROSTER** — the reference chrome's "clear bordered fields...
//! distinct Replace/Replace all controls" is a pixel claim, not a geometry one:
//! a box can be correctly PLACED (proven by `render/plan/tests/panel_law.rs`'s
//! device-level hit-test sweep) while reading as invisible, exactly the
//! Wagtail tripwire CLAUDE.md records for a picker's selected-row band.
//!
//! **THE COMPANION-FLOOR SHAPE.** A field's FILL (`base_200`) is a value-step
//! off the surrounding card (`base_300`) on nineteen worlds, but on Wagtail
//! (the one-bit world) the two are IDENTICAL by design — the whole ramp
//! collapses to pure black/white there (`theme::derive::surface_step_band`'s own
//! early-return). A floor that only asked "does the fill differ from the card"
//! would need Wagtail excluded by NAME, which is exactly the enrolment trap
//! CLAUDE.md warns against. Instead the floor is DERIVED from each world's own
//! data: where `base_200 != base_300`, the FILL must carry real contrast against
//! the card; where they collapse, the BORDER token (`surface_selected`, the
//! same one every other summoned card's border already reads) must instead —
//! proving the compensating mechanism is real rather than assuming it.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

const W: u32 = 1200;
const H: u32 = 800;
/// A real, non-anti-aliased channel delta: two colors a viewer can tell apart
/// at a glance, not merely two f32s that differ in the ninth decimal.
const VISIBLE_DELTA: i32 = 12;

fn panel_view() -> ViewState {
    let mut v = view("hello world\nhello again\n", 0, 0);
    v.search_active = true;
    v.search_query = "hello".into();
    v.search_matches = vec![((0, 0), (0, 5)), ((1, 0), (1, 5))];
    v.search_current = Some(0);
    v.search_replace_active = true;
    v.search_replacement = "goodbye".into();
    v
}

fn pixel_at(pixels: &[[u8; 4]], width: u32, x: f32, y: f32) -> [u8; 4] {
    let xi = (x.round().max(0.0) as u32).min(width - 1);
    let yi = y.round().max(0.0) as u32;
    pixels[(yi * width + xi) as usize]
}

fn max_channel_delta(a: [u8; 4], b: [u8; 4]) -> i32 {
    (0..3)
        .map(|c| (a[c] as i32 - b[c] as i32).abs())
        .max()
        .unwrap()
}

/// **PRESENCE + CONTRAST, DERIVED PER WORLD, NEVER ASSUMED.** Every world must
/// actually DRAW the field/button boxes and the region separators (GPU instance
/// counts, so a reverted draw call goes red here, not just in geometry); then,
/// depending on whether THIS world's own `base_200`/`base_300` data collapse,
/// either the drawn FILL or the drawn/derived BORDER must clear a real,
/// non-anti-aliased pixel floor against the card ground.
#[test]
fn bordered_controls_are_drawn_and_visible_across_the_world_roster() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping bordered_controls_are_drawn_and_visible_across_the_world_roster: \
             no wgpu adapter"
        );
        return;
    };
    let ambient = theme::active_index();
    let mut enrolled = 0usize;
    let mut degenerate_worlds = 0usize;
    let mut worst_fill_delta = i32::MAX;

    for (i, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        p.sync_theme();
        p.set_view(&panel_view());
        p.prepare(&device, &queue, W, H).unwrap();

        assert!(
            p.panel_control_fill.instance_count() > 0,
            "{}: no field/button fills drawn",
            world.name
        );
        assert!(
            p.panel_control_border.instance_count() > 0,
            "{}: no field/button borders drawn",
            world.name
        );
        assert!(
            p.panel_rules.instance_count() > 0,
            "{}: no region separators drawn",
            world.name
        );

        let g = p
            .panel_geometry()
            .expect("an active search publishes geometry");
        let find_field = g
            .controls
            .iter()
            .find(|c| c.name == "find_field")
            .expect("find_field must be published");
        let [fx, fy, fw, fh] = find_field.rect;
        let [cx, cy, ..] = g.card;

        let pixels = pixeldiff::render_frame(&mut p, &device, &queue, W, H);
        let fill_px = pixel_at(&pixels, W, fx + fw * 0.5, fy + fh * 0.5);
        // The card's own pad corner: inside the card, on no shaped row, no
        // control ever drawn there — pure card ground.
        let card_px = pixel_at(&pixels, W, cx + 3.0, cy + 3.0);
        let fill_delta = max_channel_delta(fill_px, card_px);

        if world.base_200 != world.base_300 {
            assert!(
                fill_delta >= VISIBLE_DELTA,
                "{}: the field's fill {fill_px:?} is not visibly distinct from \
                 the card ground {card_px:?} (delta {fill_delta}, floor {VISIBLE_DELTA}) \
                 — base_200 differs from base_300 in this world's own data, so the \
                 fill should carry the distinction",
                world.name
            );
            worst_fill_delta = worst_fill_delta.min(fill_delta);
        } else {
            // THE DEGENERATE ARM: fill and card collapse to the same value by
            // design (Wagtail). The BORDER token must carry the distinction
            // instead — verified against the theme's own derived color, the
            // same one `theme::surface_selected()` every other summoned card's
            // border already reads, never a hardcoded literal here.
            degenerate_worlds += 1;
            let border_token = theme::surface_selected();
            let border_delta = max_channel_delta(
                [border_token.r, border_token.g, border_token.b, 255],
                [world.base_300.r, world.base_300.g, world.base_300.b, 255],
            );
            assert!(
                border_delta >= VISIBLE_DELTA,
                "{}: base_200 collapses onto base_300, and the border token \
                 {border_token:?} is not visibly distinct from the card ground \
                 {:?} either (delta {border_delta}) — the compensating mechanism \
                 this floor assumes is not actually present",
                world.name,
                world.base_300
            );
        }
        enrolled += 1;
    }
    theme::set_active(ambient);

    assert_eq!(
        enrolled,
        theme::THEMES.len(),
        "the sweep must reach every world"
    );
    assert!(
        degenerate_worlds >= 1,
        "the degenerate (base_200 == base_300) arm must be exercised by at \
         least one real world, or that branch is dead code no run has taken"
    );
    assert!(
        degenerate_worlds < theme::THEMES.len(),
        "the ordinary (base_200 != base_300) arm must ALSO be exercised — a \
         floor that only ever took the degenerate branch never actually graded \
         the fill contrast this test is named for"
    );
    assert!(
        worst_fill_delta != i32::MAX && worst_fill_delta >= VISIBLE_DELTA,
        "the tightest real fill-vs-card delta measured was {worst_fill_delta} \
         against a floor of {VISIBLE_DELTA}"
    );
}
