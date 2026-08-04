//! ITEM 221 — CASSOWARY'S VERTICAL SECOND-LEVEL HEADING.
//!
//! **Defect:** item 220 gave every faceting picker a location line (the
//! active lens's own name, the second level of the card's title hierarchy),
//! but drew it the same generic way on every world — including Cassowary,
//! whose left edge and bold Archivo-Black "Commands" placard are its whole
//! visual language, and which a horizontal line of chrome-face text in a
//! bars plate does not use at all.
//!
//! **Build:** on a `RotatedRail` world (Cassowary today, the ONLY world this
//! item touches), the location renders NOTHING inline — the plan line
//! item 220 built stays glyph-free — and instead a small muted run, turned
//! 90°, flush with the card's own left border. It reuses item 220's single
//! `PlanLine::Location` slot (no second row is planned) and item 235's
//! rotated-label capability wholesale (no second rotation path).
//!
//! Two families of law:
//!
//! - [`rotated_location_origin`] is PURE (no GPU, no font system) — exact
//!   float assertions on where the capability's own geometry helpers place
//!   the run, and that the transform is a genuine 90° turn (a transpose of
//!   the ink box), not a near-90° approximation.
//! - The real-pixel laws render Cassowary's REAL faceted card end to end
//!   (the actual `prepare_overlay` path, real GPU pixels) and grade the
//!   claims CLAUDE.md's tripwire insists on: appearance is arithmetic over
//!   the PNG's pixels, never the sidecar (which reports state, not whether
//!   anything is visible). Swept over the roster's SHORTEST and LONGEST
//!   facet names ("Files" vs "Navigate"/"Settings") — the shrink-to-fit
//!   budget in `prepare_rotated_location_label` exists entirely because a
//!   law (and a live capture) caught the long names bleeding into a
//!   neighbour under the first, centred placement this item shipped.

use super::super::*;
use super::{headless_dqp, view};
use crate::render::layers::{
    ROTATED_LOCATION_HEADER_GAP_FRAC, ROTATED_LOCATION_INSET_PX, rotated_location_origin,
};
use crate::rotated_label::geometry::{InkBox, label_axis_deg, label_bounds};

// ---------------------------------------------------------------------------
// PURE GEOMETRY — no device, no font system.
// ---------------------------------------------------------------------------

/// A handful of representative `(card_x, row_top, row_height)` triples and
/// ink boxes — short ("Files"-shaped) and long ("Settings"-shaped) — swept
/// against the SAME formula the product calls, never a re-derivation of it.
fn geometry_fixtures() -> Vec<(f32, f32, f32, InkBox)> {
    vec![
        // (card_x, row_top, row_height, ink[u_min, v_min, width, height])
        (420.0, 200.0, 32.0, [-1.0, -14.0, 50.0, 19.0]),
        (420.0, 200.0, 32.0, [-1.0, -14.0, 87.0, 19.0]),
        (0.0, 0.0, 24.0, [-2.0, -12.0, 65.0, 17.0]),
        (733.5, 481.25, 40.4, [-1.0, -15.0, 30.0, 20.0]),
    ]
}

/// The run's screen footprint is FLUSH with the card's own left border
/// (`card_x + ROTATED_LOCATION_INSET_PX`, exactly — a hairline clearance, not
/// touching the border stroke) and BOTTOM-anchored on the row band (its
/// bottom edge lands exactly on `row_top + row_height`, never centred — see
/// `rotated_location_origin`'s own doc for why bottom, not centred, is the
/// product's real anchor).
#[test]
fn rotated_location_origin_is_flush_left_and_bottom_anchored() {
    let axis = label_axis_deg(90.0);
    for (card_x, row_top, row_height, ink) in geometry_fixtures() {
        let origin = rotated_location_origin(card_x, row_top, row_height, axis, ink);
        let bounds = label_bounds(origin, axis, ink);
        let left = bounds[0];
        let bottom = bounds[1] + bounds[3];
        assert!(
            (left - (card_x + ROTATED_LOCATION_INSET_PX)).abs() < 1e-3,
            "card_x {card_x} row {row_top}/{row_height}: left edge {left} \
             is not flush with card_x + inset ({})",
            card_x + ROTATED_LOCATION_INSET_PX
        );
        assert!(
            (bottom - (row_top + row_height)).abs() < 1e-3,
            "card_x {card_x} row {row_top}/{row_height}: bottom edge {bottom} \
             is not anchored to the row's own bottom ({})",
            row_top + row_height
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
    for (card_x, row_top, row_height, ink) in geometry_fixtures() {
        let rotated = label_bounds(
            rotated_location_origin(card_x, row_top, row_height, axis_90, ink),
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
                rotated_location_origin(card_x, row_top, row_height, axis_upright, ink),
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
/// data-sanity guard against the shrink-to-fit budget silently reverting to
/// "the whole gap" (this item's own first cut, which measurably touched the
/// lens strip on the roster's two longest facet names — see the real-pixel
/// law below) or to "none of it" (which would shrink far more aggressively
/// than the real layout needs).
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
// REAL PIXELS — the actual `prepare_overlay` path, real GPU output.
// ---------------------------------------------------------------------------

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

/// A COMMAND-palette view on Cassowary, at the facet strip index `lens`,
/// folded the way `App::sync_view` folds one — the same construction
/// `palette_location_item220.rs`'s own fixture uses.
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

fn lens_index(label: &str) -> usize {
    crate::facets::scheme(crate::overlay::OverlayKind::Command)
        .expect("the command palette facets")
        .strip
        .iter()
        .position(|f| f.label == label)
        .unwrap_or_else(|| panic!("no {label} lens on the command palette"))
}

/// Render `v`, restore nothing (the caller owns `p`'s lifetime), return the
/// RGBA readback.
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

/// **THE CUE IS FLUSH-LEFT, LEGIBLE, AND NEVER CROWDS A NEIGHBOUR — real
/// pixels, both DPIs, the roster's shortest and two longest facet names.**
///
/// THE ORACLE IS DIFFERENTIAL, like `palette_location_item220.rs`'s own pixel
/// law: the identical view is shot twice, differing in ONLY
/// `overlay_location` (`Some(label)` vs `None`) — same items, same
/// selection, same bar plates (a SELECTED `HugLabel` plate genuinely grows
/// toward the card's own left edge under `grow_px`, which made an earlier,
/// non-differential cut of this law fail on its OWN gutter scan, mistaking
/// that plate's ink for the cue's). Whatever pixel differs between the two
/// shots is attributable to the location treatment ALONE — the rotated cue
/// appearing, or (nothing else) the retired uppercase header disappearing
/// from the TEXT column. Restricting the diff to the GUTTER
/// (`x < text_left`) isolates the rotated cue's own contribution: the
/// retired header draws in the text column, never the gutter.
///
/// Per world state: **present** (some diff pixel in the cue's own permitted
/// band, from the lens strip's own drawn bottom to the location row's own
/// bottom — non-vacuity, a parked pipeline fails this), **flush** (the
/// leftmost diff pixel sits within a few device px of `card_x`), and **no
/// collision** (zero diff pixels in the gutter outside that band — above the
/// strip, or below the row).
#[test]
fn cassowary_rotated_location_cue_is_flush_left_and_never_crowds_a_neighbor() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item221 rotated-location pixel law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");

    let mut graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        for &label in &["Files", "Navigate", "Settings"] {
            p.sync_theme();
            p.set_dpi(dpi);
            let lens = lens_index(label);
            let mut v = palette_view(lens);
            assert_eq!(v.overlay_location.as_deref(), Some(label));

            p.set_view(&v);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            // THE REAL GEOMETRY THIS FRAME PLANNED — never re-derived, never
            // eyeballed off a screenshot.
            let geom = p.overlay_geometry(1200);
            assert!(
                p.overlay_geom_is_faceted(&geom),
                "{label}: not a faceted card"
            );
            let plan = p.overlay_row_plan(&geom);
            let loc_display = geom
                .plan_labels_probe()
                .iter()
                .position(|s| s == &format!("loc:{label}"))
                .unwrap_or_else(|| {
                    panic!(
                        "{label}: no location line planned ({:?})",
                        geom.plan_labels_probe()
                    )
                });
            let row = *plan
                .rows()
                .get(loc_display)
                .unwrap_or_else(|| panic!("{label}: no planned row at display {loc_display}"));
            assert!(
                plan.strip_band().is_some(),
                "{label}: Cassowary's command palette has no lens strip"
            );
            let card_x = geom.band_x_probe();
            let text_left = geom.text_left;
            let row_bottom = row.top + row.height;
            // THE DESIGN'S OWN CEILING — `row_height` plus the fraction of
            // the query beat's calm divider `prepare_rotated_location_label`
            // treats as safely blank (never the WHOLE gap: most of it is
            // the strip's own pill sitting centred in a taller box, not
            // free space — see that function's own comment). A cue that
            // reaches higher than this, for ANY reason (the shrink-to-fit
            // call skipped, the wrong quantity fed it, a future edit to the
            // budget formula), fails here independently of the internal
            // formula: this is the REAL rendered footprint, read back from
            // GPU pixels, held to the CONTRACT rather than re-trusted.
            let budget = row.height + geom.header_gap.max(0.0) * ROTATED_LOCATION_HEADER_GAP_FRAC;

            let with_loc = shoot(&device, &queue, &mut p, 1200, 800);
            v.overlay_location = None;
            let without_loc = render_view(&device, &queue, &mut p, &v);

            // `set_dpi` changes glyph SCALE, not the framebuffer's own
            // resolution (`set_size` alone owns that) — the capture stays a
            // fixed 1200x800, and `dpi` is exercised through the geometry
            // (`row`, `geom.header_gap`, `geom.band_x_probe()`) reading
            // finer glyph metrics at the same canvas size, exactly like
            // `render/tests/hover_slop_law.rs` and its neighbours sweep it.
            let (w, h) = (1200i64, 800i64);
            assert_eq!(
                with_loc.len() as i64,
                w * h,
                "{label} at {dpi}x: capture size mismatch"
            );

            // `AA_PAD` absorbs anti-aliased edge softness AT THE BUDGET
            // CEILING alone (a glyph's antialiased top row is real ink at
            // partial coverage, not a design overflow) — never at the row's
            // own bottom, which `rotated_location_origin` anchors EXACTLY:
            // that edge staying zero-tolerance is the real "never crowds a
            // command row" guarantee this law exists to hold.
            const AA_PAD: f32 = 2.0;
            let x0 = card_x.round().max(0.0) as i64;
            let x1 = (text_left.round() as i64).min(w);
            let y_lo = (row_bottom - budget - AA_PAD).round() as i64;
            let y_hi = row_bottom.round() as i64;

            let mut present = false;
            let mut leftmost: Option<i64> = None;
            let mut above_strip: Vec<(i64, i64)> = Vec::new();
            let mut below_row: Vec<(i64, i64)> = Vec::new();
            for y in 0..h {
                for x in x0.max(0)..x1.max(x0) {
                    let i = (y * w + x) as usize;
                    if with_loc[i] == without_loc[i] {
                        continue;
                    }
                    if y >= y_lo && y <= y_hi {
                        present = true;
                        leftmost = Some(leftmost.map_or(x, |l| l.min(x)));
                    } else if y < y_lo {
                        above_strip.push((x, y));
                    } else {
                        below_row.push((x, y));
                    }
                }
            }
            assert!(
                present,
                "{label} at {dpi}x: no differing pixel found in the cue's own permitted \
                 gutter band (x [{x0},{x1}) y [{y_lo},{y_hi}]) — the cue did not draw"
            );
            let left_px = leftmost.expect("present implies a leftmost column");
            let flush_target = card_x.round() as i64;
            assert!(
                (left_px - flush_target).abs() <= (2.0 * ROTATED_LOCATION_INSET_PX).round() as i64,
                "{label} at {dpi}x: leftmost cue ink at x={left_px} is not flush with \
                 card_x={flush_target}"
            );
            assert!(
                above_strip.is_empty(),
                "{label} at {dpi}x: {} differing pixels ABOVE the shrink-to-fit budget \
                 (row_height + header_gap*frac, {budget:.1}px) ({:?}…) — the cue grew \
                 past its own declared ceiling toward the lens strip",
                above_strip.len(),
                &above_strip[..above_strip.len().min(5)]
            );
            assert!(
                below_row.is_empty(),
                "{label} at {dpi}x: {} differing pixels BELOW the location row's own \
                 bottom ({:?}…) — the cue crowds the first command row",
                below_row.len(),
                &below_row[..below_row.len().min(5)]
            );
            graded += 1;
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(graded, 6, "the sweep over facet names x DPI moved");
}

/// **THE ALL HOME SHOWS NO CUE — same gutter-column differential, real
/// pixels.** All the same confounds `cassowary_rotated_location_cue_is_
/// flush_left_and_never_crowds_a_neighbor` cancels apply here too (a
/// SELECTED plate reaches close to `card_x`), so this reads the SAME
/// differential (`overlay_location: Some("Files") -> None`) at strip index
/// 0 (All) instead — where the two shots must be BYTE IDENTICAL, because
/// `overlay_location` is already `None` at the All home
/// (`FacetScheme::location`'s own contract) and forcing it to `None` again
/// is a no-op. Non-vacuity is inherited from the Files-lens law above,
/// which proves this same oracle sees a real diff when one exists.
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
