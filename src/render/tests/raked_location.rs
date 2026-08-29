//! MAGPIE'S LOCATION CUE JOINS THE DIAGONAL LANGUAGE.
//!
//! **The premise, corrected before this file was written.** A `Diagonal`
//! world's location row carries no attachment inset (`RowSpan`'s `dx == 0`
//! at display 0), so the label already sits flush at the card's own text
//! column — there is no right-side indicator to mirror, and no mirroring
//! mechanism is built here. The two things genuinely missing are a SLANT
//! (the cue read upright, the one thing that made it look detached from the
//! world's diagonal line) and a GRADIENT (already a shared capability,
//! unused until now). Both ship as `theme::LocationStyle::Raked`, reusing
//! the shared rotated-label preparation owner wholesale — see
//! `render/rotated_location.rs`'s module doc for the shared shape, and
//! `render/chrome/diagonal.rs::location_axis_deg` for how the angle is
//! DERIVED from the spine's own step rather than pinned.
//!
//! Three families of law:
//!
//! - The angle is tied to the spine's own geometry, not a copied constant —
//!   graded PURELY here, and against the spine's own MEASURED (possibly
//!   narrow-card-clamped) step, never the authored one alone.
//! - No overlap with command rows — graded both purely (the shared
//!   `rotated_location_origin` formula's bottom-anchor guarantee, at
//!   Magpie's own non-quadrant angle) and over real pixels.
//! - The slant and the gradient actually reach the screen, at both DPI
//!   tiers — real GPU pixels, oracled differentially (`overlay_location:
//!   Some(label)` vs `None`), so ground, palette texture, and every other
//!   row cancel and what remains is the cue alone.

use super::super::*;
use super::{dither, headless_dqp, view};
use crate::render::chrome::diagonal::{active, location_axis_deg};
use crate::render::rotated_location::{ROTATED_LOCATION_HEADER_GAP_FRAC, rotated_location_origin};
use crate::rotated_label::geometry::label_axis_deg;

/// Magpie's own reference geometry: `ROW_STEP` (7px) over a representative
/// 32px row height, unscaled — the same numbers that first derived 77.66°,
/// kept here only as an independent regression witness, never as the
/// product's source of truth (that is `location_axis_deg` itself, fed the
/// REAL measured step at render time).
const REFERENCE_ROW_STEP: f32 = -7.0; // Ascending: negative, matches `DiagonalDirection::sign`
const REFERENCE_ROW_HEIGHT: f32 = 32.0;
const REFERENCE_ANGLE_DEG: f32 = 77.66;

// ---------------------------------------------------------------------------
// PURE — the angle formula itself, no GPU, no font system.
// ---------------------------------------------------------------------------

/// The formula matches an independently-written computation, at both signs,
/// and reproduces the reference angle first measured for this geometry —
/// without hardcoding that number as the source of truth (it is only
/// asserted here as a witness that this derivation and that measurement
/// agree).
#[test]
fn location_axis_deg_matches_an_independent_computation_and_the_reference_angle() {
    for (row_step, row_height) in [
        (REFERENCE_ROW_STEP, REFERENCE_ROW_HEIGHT),
        (-3.5, 16.0),  // same ratio, half scale — must reproduce the same angle
        (-14.0, 64.0), // same ratio, double scale
        (7.0, 32.0),   // Descending: positive step
        (-20.0, 32.0), // a steeper lean than Magpie's own
        (-0.5, 32.0),  // a much shallower lean, near-vertical
    ] {
        let lean = row_step.abs().atan2(row_height).to_degrees();
        let base = 90.0 - lean;
        let expected = if row_step <= 0.0 { base } else { 180.0 - base };
        let got = location_axis_deg(row_step, row_height);
        assert!(
            (got - expected).abs() < 1e-4,
            "row_step {row_step} row_height {row_height}: got {got}, expected {expected}"
        );
    }
    let magpie = location_axis_deg(REFERENCE_ROW_STEP, REFERENCE_ROW_HEIGHT);
    assert!(
        (magpie - REFERENCE_ANGLE_DEG).abs() < 0.01,
        "Magpie's reference geometry produced {magpie}°, expected close to {REFERENCE_ANGLE_DEG}°"
    );
}

/// A degenerate row (zero or negative height — never real, but a caller
/// should not divide by it) falls back to the vertical axis rather than
/// producing NaN or an infinite lean.
#[test]
fn location_axis_deg_is_finite_and_vertical_on_a_degenerate_row() {
    for row_height in [0.0, -1.0, f32::NAN] {
        let deg = location_axis_deg(-7.0, row_height);
        assert_eq!(
            deg, 90.0,
            "row_height {row_height}: expected the vertical fallback"
        );
    }
}

/// **NOT A PINNED CONSTANT.** The angle moves with the ratio it is derived
/// from: steepening `row_step` leans the axis further from vertical, and the
/// two signed directions lean opposite ways — the property a hardcoded
/// 77.66° literal (or its bare negation) could not reproduce.
#[test]
fn location_axis_deg_is_sensitive_to_the_step_it_is_derived_from() {
    let shallow = location_axis_deg(-2.0, 32.0);
    let magpie = location_axis_deg(REFERENCE_ROW_STEP, REFERENCE_ROW_HEIGHT);
    let steep = location_axis_deg(-16.0, 32.0);
    assert!(
        shallow > magpie && magpie > steep,
        "shallow {shallow} magpie {magpie} steep {steep}: a bigger leftward step must lean \
         further from vertical (a smaller angle), not stay pinned"
    );
    let ascending = location_axis_deg(-7.0, 32.0);
    let descending = location_axis_deg(7.0, 32.0);
    assert!(
        (ascending - (180.0 - descending)).abs() < 1e-4,
        "ascending {ascending} and descending {descending} must be mirror images across \
         vertical (90°), not independently-signed numbers"
    );
}

/// **THE SHARED ORIGIN FORMULA'S GUARANTEES HOLD AT MAGPIE'S OWN ANGLE, NOT
/// ONLY AT 90°.** `rotated_location_origin` is the one owner both
/// `RotatedRail` and `Raked` route through (`render/rotated_location.rs`),
/// already proved flush/bottom-anchored at 90° elsewhere — this proves the
/// SAME formula at a near-vertical, non-quadrant angle, which is what stops
/// Magpie's cue from crowding the row beneath it. Magpie seats its run on the
/// row's own text column with no clearance, so the flush edge IS `flush_x`.
#[test]
fn raked_location_origin_is_flush_left_and_bottom_anchored_at_a_non_quadrant_angle() {
    let axis = label_axis_deg(REFERENCE_ANGLE_DEG);
    let fixtures: [(f32, f32, f32, [f32; 4]); 4] = [
        (152.0, 300.0, 32.0, [-1.0, -14.0, 22.0, 19.0]), // "Files"-shaped
        (22.0, 300.0, 32.0, [-1.0, -14.0, 58.0, 19.0]),  // "Navigate"-shaped
        (0.0, 0.0, 24.0, [-2.0, -12.0, 44.0, 17.0]),
        (733.5, 481.25, 40.4, [-1.0, -15.0, 30.0, 20.0]),
    ];
    for (flush_x, row_top, row_height, ink) in fixtures {
        let origin = rotated_location_origin(
            crate::render::rotated_location::FlushEdge::Left(flush_x),
            row_top + row_height,
            axis,
            ink,
        );
        let bounds = crate::rotated_label::geometry::label_bounds(origin, axis, ink);
        let left = bounds[0];
        let bottom = bounds[1] + bounds[3];
        assert!(
            (left - flush_x).abs() < 1e-3,
            "flush_x {flush_x}: left edge {left} is not flush (0 inset)"
        );
        assert!(
            (bottom - (row_top + row_height)).abs() < 1e-3,
            "row {row_top}/{row_height}: bottom edge {bottom} is not anchored to the row's \
             own bottom ({}) — an overrun here is an overlap with the row below",
            row_top + row_height
        );
    }
}

// ---------------------------------------------------------------------------
// REAL GEOMETRY — a live Magpie overlay, no pixel readback needed.
// ---------------------------------------------------------------------------

fn palette_view(lens: usize) -> ViewState {
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov = crate::overlay::OverlayState::new_command(
        names,
        crate::commands::effective_bindings(&[], &[], crate::keymap::KeymapFlavor::Native),
        hidden,
    );
    ov.set_facet_lens(lens);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Command.title().to_string();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_selected = ov.selected;
    v
}

fn files_lens() -> usize {
    crate::facets::scheme(crate::overlay::OverlayKind::Command)
        .expect("the command palette facets")
        .strip
        .iter()
        .position(|f| f.label == "Files")
        .expect("the command palette has a Files lens")
}

/// **THE MEASURED STEP, NOT THE AUTHORED ONE, IS WHAT THE CUE FOLLOWS.** On an
/// ordinary window the diagonal spine affords its authored `ROW_STEP`
/// outright; on a card too narrow for it, `spine_travel`'s
/// `TRAVEL_MAX_BAND_FRACTION` yield flattens the spine — and if the location
/// cue read the authored constant instead of that same measured value, it
/// would lean MORE than the line beside it, drifting apart from it exactly in
/// the one regime this matters. Both widths are real, prepared frames; no
/// number here is asserted from the formula alone.
#[test]
fn location_axis_deg_follows_the_spines_measured_clamp_not_the_authored_constant() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping clamp-tracking law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Magpie").expect("Magpie ships");
    let files = files_lens();

    let measure = |p: &mut TextPipeline, w: u32| -> (f32, f32, f32) {
        p.sync_theme();
        let v = palette_view(files);
        p.set_view(&v);
        p.prepare(&device, &queue, w, 800).unwrap();
        let geom = p.overlay_geometry(w);
        let plan = p.overlay_row_plan(&geom);
        let composition = active(p).expect("Magpie carries a diagonal composition");
        let probe = p
            .diagonal_cluster_probe()
            .expect("a faceted Magpie card measures a diagonal cluster");
        let loc_display = geom
            .plan_labels_probe()
            .iter()
            .position(|s| s.starts_with("loc:"))
            .expect("the Files lens plans a location line");
        let row_height = plan.rows()[loc_display].height;
        (composition.row_step, probe.spine_step(), row_height)
    };

    let (authored_wide, measured_wide, rh_wide) = measure(&mut p, 1200);
    let (authored_narrow, measured_narrow, rh_narrow) = measure(&mut p, 220);
    theme::set_active(theme::DEFAULT_THEME);

    assert_eq!(
        authored_wide, authored_narrow,
        "the AUTHORED step is a theme constant and must not vary with window width"
    );
    assert!(
        (measured_wide - authored_wide).abs() < 1e-3,
        "an ordinary 1200px window must afford the authored step outright: measured \
         {measured_wide}, authored {authored_wide}"
    );
    assert!(
        measured_narrow.abs() < authored_wide.abs() - 1.0,
        "a 220px window must actually engage the narrow-card clamp (measured \
         {measured_narrow}, authored {authored_wide}) — otherwise this law tests nothing"
    );

    let axis_wide = location_axis_deg(measured_wide, rh_wide);
    let axis_narrow_measured = location_axis_deg(measured_narrow, rh_narrow);
    let axis_narrow_if_authored_were_used = location_axis_deg(authored_narrow, rh_narrow);
    assert!(
        (axis_narrow_measured - 90.0).abs() < (axis_wide - 90.0).abs(),
        "the clamped spine is flatter (closer to vertical), so the cue derived from the \
         MEASURED step must lean less than the wide-window cue: narrow {axis_narrow_measured}, \
         wide {axis_wide}"
    );
    assert!(
        (axis_narrow_measured - axis_narrow_if_authored_were_used).abs() > 0.5,
        "reading the authored constant instead of the measured step would have produced a \
         visibly different angle here ({axis_narrow_if_authored_were_used} vs \
         {axis_narrow_measured}) — the two sources must actually disagree for this law to \
         prove anything"
    );
}

// ---------------------------------------------------------------------------
// REAL PIXELS — the actual `prepare_overlay` path, both DPI tiers.
// ---------------------------------------------------------------------------

fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl raked-location encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    dither::read_pixels(device, queue, &texture, w, h)
}

fn luma(c: [u8; 4]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

/// One `(dpi,)` cell of the real-pixel laws below: renders Magpie's Files
/// lens with and without a location, both at 1200x800, and returns the set
/// of differing pixels plus the row band they are graded against.
///
/// **THE CONFOUND, FOUND AND ROUTED AROUND.** Forcing `overlay_location` to
/// `None` does not make the row glyph-free on every world — `theme_plan`
/// (`render/chrome/theme_picker.rs`) falls back to `PlanLine::Header`, which
/// draws INLINE unconditionally regardless of `LocationStyle` (the retired
/// uppercase header reappears — a deliberate non-vacuity arm elsewhere, not a
/// bug here). That header is real, expected ink, confined to `[row.top,
/// row.bottom]` — it is what a plain differential would compare the cue
/// against. A cue seated flush with a CARD BORDER, off in its own gutter
/// column, sidesteps this by position (the header never draws there); Magpie's
/// cue sits in the SAME column as the header (its own inline placement is
/// unmoved), so there is no gutter to isolate by position. What separates
/// them instead is REACH: the header is one line tall and cannot leave
/// `[row.top, row.bottom]`; Magpie's cue is BOTTOM-anchored at `row.bottom`
/// (proven exactly by the pure law above) and grows upward past `row.top`
/// whenever it is genuinely turned — the one thing an upright run confined
/// to its own row height structurally cannot do. Both laws below grade the
/// ABOVE-ROW band alone for exactly that reason.
struct Grade {
    diff: Vec<(i64, i64)>,
    row_top: f32,
    row_bottom: f32,
    budget_top: f32,
    flush_x: f32,
    row_height: f32,
}

impl Grade {
    /// Diff pixels strictly above the row's own line box — unreachable by
    /// either the header this diff would otherwise show or an upright
    /// single-line run, reachable only by a cue that genuinely climbs.
    fn above_row(&self, margin: f32) -> Vec<(i64, i64)> {
        self.diff
            .iter()
            .copied()
            .filter(|&(_, y)| (y as f32) < self.row_top - margin)
            .collect()
    }

    /// Diff pixels within a thin band flush with the row's own bottom — past
    /// where the header (which stopped short of `row_bottom` in every
    /// measured case) reaches, so this is cue-only regardless of the header
    /// confound above it.
    fn at_bottom(&self, margin: f32) -> Vec<(i64, i64)> {
        self.diff
            .iter()
            .copied()
            .filter(|&(_, y)| (y as f32) >= self.row_bottom - margin)
            .collect()
    }
}

fn grade(device: &wgpu::Device, queue: &wgpu::Queue, p: &mut TextPipeline, dpi: f32) -> Grade {
    p.sync_theme();
    p.set_dpi(dpi);
    let files = files_lens();
    let mut v = palette_view(files);
    assert!(
        v.overlay_location.is_some(),
        "the Files lens must carry a location"
    );

    p.set_view(&v);
    p.prepare(device, queue, 1200, 800).unwrap();
    let geom = p.overlay_geometry(1200);
    assert!(p.overlay_geom_is_faceted(&geom), "not a faceted card");
    let plan = p.overlay_row_plan(&geom);
    let loc_display = geom
        .plan_labels_probe()
        .iter()
        .position(|s| s.starts_with("loc:"))
        .expect("a planned location line");
    let row = plan.rows()[loc_display];
    let budget = row.height + geom.header_gap.max(0.0) * ROTATED_LOCATION_HEADER_GAP_FRAC;
    let flush_x = geom.text_left + row.dx;

    let with_loc = shoot(device, queue, p, 1200, 800);
    v.overlay_location = None;
    p.set_view(&v);
    p.prepare(device, queue, 1200, 800).unwrap();
    let without_loc = shoot(device, queue, p, 1200, 800);

    let (w, h) = (1200i64, 800i64);
    let mut diff = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if with_loc[i] != without_loc[i] {
                diff.push((x, y));
            }
        }
    }
    Grade {
        diff,
        row_top: row.top,
        row_bottom: row.bottom(),
        budget_top: row.bottom() - budget,
        flush_x,
        row_height: row.height,
    }
}

/// **PRESENT, FLUSH, GENUINELY CLIMBING PAST THE ROW'S OWN LINE BOX (the
/// slant), AND NEVER BELOW THE ROW'S OWN BOTTOM (no overlap with the command
/// row beneath it) — real pixels, both DPI tiers.** `above_row` is the
/// discriminator that catches a reverted slant: an upright run of the same
/// glyphs is confined to `[row.top, row.bottom]`, so it leaves NO ink above
/// `row.top` at all, whatever its width. A real near-vertical run does, by
/// construction (`rotated_location_origin`'s bottom anchor plus a run taller
/// than one row).
#[test]
fn magpie_raked_location_cue_is_present_flush_climbs_and_never_crowds_the_row_below() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping slant/geometry pixel law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Magpie").expect("Magpie ships");

    let mut graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        let g = grade(&device, &queue, &mut p, dpi);
        assert!(
            !g.diff.is_empty(),
            "{dpi}x: no differing pixel at all — the cue did not draw"
        );
        let margin = (g.row_height * 0.15).max(2.0);

        let above = g.above_row(margin);
        assert!(
            !above.is_empty(),
            "{dpi}x: no cue ink strictly above the row's own line box (row_top {}) — an \
             upright run (or the retired header this diff would otherwise reproduce) is \
             structurally confined to its own row and can never leave ink here; only a \
             genuine slant does",
            g.row_top
        );
        let min_y_above = above.iter().map(|(_, y)| *y).min().unwrap();
        assert!(
            g.row_top - min_y_above as f32 > margin,
            "{dpi}x: the cue's climb above row_top ({}) is only {:.1}px, no more than \
             anti-aliasing noise — this is not a meaningful slant",
            g.row_top,
            g.row_top - min_y_above as f32
        );
        assert!(
            min_y_above as f32 >= g.budget_top - margin,
            "{dpi}x: cue ink reaches y={min_y_above}, above its own shrink-to-fit ceiling ({}) \
             — it grew into the lens strip's own slack",
            g.budget_top
        );

        // FLUSH is checked at the run's BOTTOM (its pen origin, exactly what
        // `rotated_location_origin` places), not in the above-row band: a
        // near-vertical run leaning right as it climbs drifts AWAY from
        // `flush_x` toward the top by construction (its own reading axis has
        // a positive x component) — flush is a property of the run's start,
        // never its far end. A generous margin (well past the header's own
        // measured reach, which stops short of `row_bottom` in every case)
        // keeps this reading cue-only despite the header confound.
        let flush_margin = (g.row_height * 0.5).max(6.0);
        let bottom = g.at_bottom(flush_margin);
        assert!(
            !bottom.is_empty(),
            "{dpi}x: no cue ink flush with the row's own bottom"
        );
        let min_x_bottom = bottom.iter().map(|(x, _)| *x).min().unwrap();
        assert!(
            (min_x_bottom as f32 - g.flush_x).abs() <= 4.0,
            "{dpi}x: leftmost cue ink near the row's own bottom sits at x={min_x_bottom}, not \
             flush with text_left+dx={}",
            g.flush_x
        );

        let max_y_all = g.diff.iter().map(|(_, y)| *y).max().unwrap();
        assert!(
            (max_y_all as f32) <= g.row_bottom + margin,
            "{dpi}x: diff ink reaches y={max_y_all}, past the row's own bottom ({}) — the cue \
             (or its substitute) crowds the command row beneath it",
            g.row_bottom
        );
        graded += 1;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(graded, 2, "the DPI sweep moved");
}

/// **THE GRADIENT ACTUALLY REACHES THE SCREEN, tied to the spine's own two
/// authored tones.** Compares average colour in the ABOVE-ROW band (cue-only,
/// see [`Grade`]'s own doc for why) against the band flush with the row's own
/// bottom (also cue-only — the header stops short of it in every measured
/// case). A flat single colour would read identically in both; a real
/// gradient along the baseline does not.
#[test]
fn magpie_raked_location_cue_carries_a_real_two_tone_gradient() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping gradient pixel law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Magpie").expect("Magpie ships");

    let mut graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        let g = grade(&device, &queue, &mut p, dpi);
        assert!(
            !g.diff.is_empty(),
            "{dpi}x: no differing pixel — nothing to sample"
        );
        let margin = (g.row_height * 0.15).max(2.0);
        let above = g.above_row(margin);
        let bottom = g.at_bottom(margin);
        assert!(
            !above.is_empty(),
            "{dpi}x: no above-row band to sample — see the climb law"
        );
        assert!(!bottom.is_empty(), "{dpi}x: no at-bottom band to sample");

        // Re-shoot the "with" frame alone to sample real composited colour
        // (the diff only carries coordinates).
        p.sync_theme();
        p.set_dpi(dpi);
        let files = files_lens();
        let v = palette_view(files);
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let with_loc = shoot(&device, &queue, &mut p, 1200, 800);

        let avg_luma = |pts: &[(i64, i64)]| -> f32 {
            let sum: f32 = pts
                .iter()
                .map(|&(x, y)| luma(with_loc[(y * 1200 + x) as usize]))
                .sum();
            sum / pts.len() as f32
        };
        let top_luma = avg_luma(&above);
        let bottom_luma = avg_luma(&bottom);
        assert!(
            (top_luma - bottom_luma).abs() > 3.0,
            "{dpi}x: above-row luma {top_luma:.1} and at-bottom luma {bottom_luma:.1} are \
             indistinguishable — a flat single colour would read exactly this way"
        );
        graded += 1;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(graded, 2, "the DPI sweep moved");
}

/// **THE ALL HOME SHOWS NO CUE** — same differential, at strip index 0, where
/// `overlay_location` is already `None` (`FacetScheme::location`'s own
/// contract), so forcing it to `None` again must be a byte-identical no-op.
#[test]
fn magpie_all_home_draws_no_raked_cue() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping all-home law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Magpie").expect("Magpie ships");
    p.sync_theme();

    let mut v = palette_view(0); // All
    assert_eq!(
        v.overlay_location, None,
        "the All lens already carries no location"
    );
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let with = shoot(&device, &queue, &mut p, 1200, 800);
    v.overlay_location = None;
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let without = shoot(&device, &queue, &mut p, 1200, 800);

    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        with, without,
        "forcing an already-None overlay_location changed the render — the All home is not \
         actually cue-free"
    );
}
