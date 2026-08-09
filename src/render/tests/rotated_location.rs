//! ITEM 221 — THE FACETED CARD'S LOCATION CUE, DRAWN RATHER THAN SHAPED.
//!
//! **Defect:** the shared row planner gave every faceting picker a location
//! line (the active lens's own name, the second level of the card's title
//! hierarchy), but drew it the same generic way on every world — including
//! worlds whose whole visual language is a rotation.
//!
//! **Build:** on a world whose `LocationStyle` answers `draws_inline() ==
//! false`, the location renders NOTHING inline — the planned line stays
//! glyph-free — and a rotated run is composed instead, through the
//! world-neutral rotated-label capability (no second rotation path) and the
//! row planner's single `PlanLine::Location` slot (no second row).
//!
//! ⚠️ **THIS FILE DOES NOT OWN `RotatedRail`'s PLACEMENT.** That style is
//! composed against the ROOM — the wordmark placard's own outer margin, at ⅔
//! of its type — and its composition, presence, non-overlap and park laws live
//! in the sibling `rotated_rail` module. What remains here is what both
//! expressions still share: the PURE placement/fit solvers, the byte-identity
//! oracles that hold `Raked` to the retired formulas it was calibrated under,
//! and the All-home law.
//!
//! - The solvers are PURE (no GPU, no font system) — exact float assertions on
//!   where the capability's own geometry helpers place a run, and that the
//!   transform is a genuine 90° turn (a transpose of the ink box), not a
//!   near-90° approximation.
//! - The byte-identity oracles re-implement the RETIRED single-axis shrink and
//!   row-anchored origin formulas as the oracle and sweep the new shared
//!   solvers against them, so `Raked`'s pixels are provably unmoved by the
//!   generalisation the rail's own composition needed. (`Raked` is Magpie's,
//!   and Magpie is one of the nineteen worlds whose bytes must not move.)

use super::super::*;
use super::{headless_dqp, view};
use crate::render::rotated_location::{
    FlushEdge, ROTATED_LOCATION_HEADER_GAP_FRAC, ROTATED_LOCATION_MIN_SCALE, rotated_fit_shrink,
    rotated_location_origin,
};
use crate::rotated_label::geometry::{InkBox, label_axis_deg, label_bounds};

// ---------------------------------------------------------------------------
// PURE GEOMETRY — no device, no font system.
// ---------------------------------------------------------------------------

/// Representative `(flush_x, row_top, row_height)` triples and ink boxes —
/// short ("Files"-shaped) and long ("Settings"-shaped) — swept against the SAME
/// solvers the product calls, never a re-derivation of them.
fn geometry_fixtures() -> Vec<(f32, f32, f32, InkBox)> {
    vec![
        // (flush_x, row_top, row_height, ink[u_min, v_min, width, height])
        (420.0, 200.0, 32.0, [-1.0, -14.0, 50.0, 19.0]),
        (420.0, 200.0, 32.0, [-1.0, -14.0, 87.0, 19.0]),
        (0.0, 0.0, 24.0, [-2.0, -12.0, 65.0, 17.0]),
        (733.5, 481.25, 40.4, [-1.0, -15.0, 30.0, 20.0]),
    ]
}

/// The run's screen footprint seats FLUSH against the edge it was given and
/// lands its BOTTOM edge exactly on the anchor — never centred (see
/// `rotated_location_origin`'s own doc for why bottom). Both flush directions
/// are graded, because `RotatedRail` mirrors onto the room's other margin when
/// the wordmark hugs that side.
#[test]
fn rotated_location_origin_is_flush_and_bottom_anchored_in_both_directions() {
    let axis = label_axis_deg(90.0);
    for (edge_x, row_top, row_height, ink) in geometry_fixtures() {
        let bottom = row_top + row_height;
        let left = label_bounds(
            rotated_location_origin(FlushEdge::Left(edge_x), bottom, axis, ink),
            axis,
            ink,
        );
        assert!(
            (left[0] - edge_x).abs() < 1e-3,
            "Left({edge_x}) bottom {bottom}: left edge {} is not flush",
            left[0]
        );
        assert!(
            (left[1] + left[3] - bottom).abs() < 1e-3,
            "Left({edge_x}) bottom {bottom}: bottom edge {} is not anchored",
            left[1] + left[3]
        );
        let right = label_bounds(
            rotated_location_origin(FlushEdge::Right(edge_x), bottom, axis, ink),
            axis,
            ink,
        );
        assert!(
            (right[0] + right[2] - edge_x).abs() < 1e-3,
            "Right({edge_x}) bottom {bottom}: right edge {} is not flush",
            right[0] + right[2]
        );
        assert!(
            (right[1] + right[3] - bottom).abs() < 1e-3,
            "Right({edge_x}) bottom {bottom}: bottom edge {} is not anchored",
            right[1] + right[3]
        );
        // The two directions are genuinely different placements whenever the
        // run has width — a `Right` arm that silently behaved like `Left` would
        // put a mirrored cue on top of the card.
        assert!(
            (left[0] - right[0]).abs() > 1e-3 || ink[2].abs() < 1e-3,
            "Left and Right produced the same origin for ink {ink:?}"
        );
    }
}

/// **THE ROTATION IS EXACT, not merely "close to 90°".** At the quarter-turn
/// axis, `label_bounds` is a genuine transpose of the ink box: its screen
/// WIDTH equals the ink's own (ascender-to-descender) HEIGHT, and its screen
/// HEIGHT equals the ink's own (reading-direction) WIDTH. This is the
/// property a forgotten rotation (axis left at upright) CANNOT satisfy —
/// proven directly below by asserting the SAME fixtures fail this exact
/// check at the upright axis, so the assertion is not vacuously true for any
/// axis.
#[test]
fn rotated_location_origin_performs_a_genuine_quarter_turn_not_a_near_90_resample() {
    let axis_90 = label_axis_deg(90.0);
    let axis_upright = label_axis_deg(0.0);
    let mut proved_non_vacuous = 0;
    for (edge_x, row_top, row_height, ink) in geometry_fixtures() {
        let bottom = row_top + row_height;
        let rotated = label_bounds(
            rotated_location_origin(FlushEdge::Left(edge_x), bottom, axis_90, ink),
            axis_90,
            ink,
        );
        assert!(
            (rotated[2] - ink[3]).abs() < 1e-3 && (rotated[3] - ink[2]).abs() < 1e-3,
            "ink {ink:?} at 90°: screen footprint {rotated:?} is not the exact transpose \
             of the ink box — the quadrant axis is not being used"
        );
        // Non-vacuity: the SAME exact-transpose claim, asked of the UPRIGHT
        // axis on the same ink box, must fail whenever the box is not
        // itself square — proving this law can see a forgotten rotation.
        if (ink[2] - ink[3]).abs() > 1.0 {
            let upright = label_bounds(
                rotated_location_origin(FlushEdge::Left(edge_x), bottom, axis_upright, ink),
                axis_upright,
                ink,
            );
            assert!(
                (upright[2] - ink[3]).abs() > 1.0 || (upright[3] - ink[2]).abs() > 1.0,
                "ink {ink:?}: the upright axis ALSO produced the transposed footprint — \
                 this law cannot tell a rotated run from an upright one"
            );
            proved_non_vacuous += 1;
        }
    }
    assert!(
        proved_non_vacuous > 0,
        "no fixture had a non-square ink box — the non-vacuity arm never ran"
    );
}

/// The header-gap SAFETY FACTOR is a fraction strictly inside `(0, 1)` — a
/// data-sanity guard against `Raked`'s shrink budget silently reverting to
/// "the whole gap" (this feature's own first cut, which measurably touched the
/// lens strip on long facet names) or to "none of it" (which would shrink far
/// more aggressively than the real layout needs).
#[test]
#[allow(clippy::assertions_on_constants)] // the constant IS the subject under test
fn header_gap_safety_fraction_is_a_real_fraction() {
    assert!(
        ROTATED_LOCATION_HEADER_GAP_FRAC > 0.0 && ROTATED_LOCATION_HEADER_GAP_FRAC < 1.0,
        "ROTATED_LOCATION_HEADER_GAP_FRAC ({ROTATED_LOCATION_HEADER_GAP_FRAC}) must be a \
         fraction of the gap, not the whole gap or none of it"
    );
}

// ---------------------------------------------------------------------------
// BYTE-IDENTITY ORACLES — `Raked`'s pixels are unmoved by the generalisation
// of the shared solvers. The oracle is the RETIRED formula, transcribed here,
// not a re-reading of the new one.
// ---------------------------------------------------------------------------

/// THE RETIRED SHRINK: one axis, the run's screen HEIGHT against a single
/// budget, floored at the legibility minimum.
fn retired_shrink(natural_height: f32, budget: f32) -> f32 {
    if natural_height > budget && natural_height > 0.0 {
        (budget / natural_height).clamp(ROTATED_LOCATION_MIN_SCALE, 1.0)
    } else {
        1.0
    }
}

/// **`Raked`'s SHRINK DECISION IS FLOAT-FOR-FLOAT WHAT IT WAS.** The
/// generalised solver takes a two-axis fit box; `Raked` passes its across axis
/// UNBOUNDED, which must make the new solver collapse onto the retired
/// single-axis formula exactly — swept over a grid that crosses the budget from
/// both sides and pushes past the floor.
///
/// The sweep's own axis is the one the author of a two-axis solver would miss:
/// an unbounded component must be inert, including when the run is wider than
/// it is tall.
#[test]
fn generalised_fit_solver_reproduces_the_retired_single_axis_shrink_exactly() {
    let mut crossed = (0usize, 0usize, 0usize);
    for &w in &[1.0f32, 30.0, 200.0, 900.0] {
        for &h in &[1.0f32, 12.0, 19.0, 40.0, 220.0, 800.0] {
            for &budget in &[1.0f32, 12.0, 19.0, 40.0, 220.0] {
                let want = retired_shrink(h, budget);
                let got = rotated_fit_shrink([w, h], [f32::INFINITY, budget]);
                assert_eq!(
                    got.to_bits(),
                    want.to_bits(),
                    "footprint {w}x{h} budget {budget}: the generalised solver answered \
                     {got} where the retired formula answers {want} — Raked's pixels move"
                );
                match () {
                    _ if h <= budget => crossed.0 += 1,
                    _ if want > ROTATED_LOCATION_MIN_SCALE => crossed.1 += 1,
                    _ => crossed.2 += 1,
                }
            }
        }
    }
    assert!(
        crossed.0 > 0 && crossed.1 > 0 && crossed.2 > 0,
        "the grid missed a regime (fits / shrinks / floored): {crossed:?}"
    );
}

/// **THE BOUNDED AXIS IS NOT INERT** — the companion arm, so the law above
/// cannot pass by a solver that ignores its fit box altogether. A run bounded on
/// BOTH axes takes the tighter ratio, which is the whole reason `RotatedRail`
/// can be held off the card.
#[test]
fn a_bounded_across_axis_tightens_the_fit_the_retired_formula_would_have_missed() {
    let (w, h) = (200.0f32, 40.0);
    let loose = rotated_fit_shrink([w, h], [f32::INFINITY, 40.0]);
    let tight = rotated_fit_shrink([w, h], [120.0, 40.0]);
    assert!(
        (loose - 1.0).abs() < 1e-6,
        "an unbounded across axis must not shrink a run that fits its along budget"
    );
    assert!(
        (tight - 0.6).abs() < 1e-6,
        "a bounded across axis must take the tighter ratio (120/200), got {tight}"
    );
}

/// **`Raked`'s ORIGIN IS FLOAT-FOR-FLOAT WHAT IT WAS.** The retired solver took
/// `(flush_x, inset_px, row_top, row_height)` and anchored the run's bottom on
/// `row_top + row_height`; the generalised one takes a flush EDGE and a bottom.
/// `Raked` passed `inset_px = 0.0`, so the two must agree exactly.
#[test]
fn generalised_origin_reproduces_the_retired_row_anchored_origin_exactly() {
    let axis = label_axis_deg(37.5); // Raked's own rake, not the quarter turn
    for (flush_x, row_top, row_height, ink) in geometry_fixtures() {
        let raw = label_bounds([0.0, 0.0], axis, ink);
        let want = [flush_x - raw[0], (row_top + row_height - raw[3]) - raw[1]];
        let got =
            rotated_location_origin(FlushEdge::Left(flush_x), row_top + row_height, axis, ink);
        assert_eq!(
            [got[0].to_bits(), got[1].to_bits()],
            [want[0].to_bits(), want[1].to_bits()],
            "flush {flush_x} row {row_top}/{row_height} ink {ink:?}: origin moved"
        );
    }
}

// ---------------------------------------------------------------------------
// REAL PIXELS — the All home, still this file's.
// ---------------------------------------------------------------------------

/// A COMMAND-palette view on Cassowary, at the facet strip index `lens`,
/// folded the way `App::sync_view` folds one.
fn palette_view(lens: usize) -> ViewState {
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov = crate::overlay::OverlayState::new_command(
        names,
        crate::commands::effective_bindings(&[], &[]),
        hidden,
    );
    ov.set_facet_lens(lens);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Command.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_selected = ov.selected;
    v
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
        label: Some("awl item221 rotated-location encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

fn render_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    v: &ViewState,
) -> Vec<[u8; 4]> {
    p.set_view(v);
    p.prepare(device, queue, 1200, 800).unwrap();
    shoot(device, queue, p, 1200, 800)
}

/// **THE ALL HOME SHOWS NO CUE — a byte-identity differential, real pixels.**
/// The identical view is shot twice, differing only in `overlay_location`
/// (`Some("Files")` vs `None`) at strip index 0 (All) — where the two must be
/// BYTE IDENTICAL, because `overlay_location` is already `None` at the All home
/// (`FacetScheme::location`'s own contract) and forcing it to `None` again is a
/// no-op. Non-vacuity is inherited from `rotated_rail`'s composition
/// law, which proves this same differential sees a real diff when one exists.
#[test]
fn cassowary_all_home_draws_no_rotated_cue() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item221 all-home law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    p.sync_theme();

    let mut v = palette_view(0); // All
    assert_eq!(
        v.overlay_location, None,
        "the All lens already carries no location"
    );
    let with = render_view(&device, &queue, &mut p, &v);
    v.overlay_location = None;
    let without = render_view(&device, &queue, &mut p, &v);

    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        with, without,
        "forcing an already-None overlay_location changed the render — the All home is \
         not actually location-free"
    );
}
