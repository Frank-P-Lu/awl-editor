use super::*;

#[test]
fn field_is_strongest_at_a_blob_center_and_decays_with_distance() {
    let blobs = [[0.5f32, 0.5, 0.1, 1.0]];
    let vp = (1000.0, 800.0);
    let center = animated_center(0, 0.5, 0.5, 0.1, vp, 0.0);
    let center_px = (center.0 * vp.0, center.1 * vp.1);
    let at_center = metaball_field(center_px, vp, &blobs, 0.0);
    let near = metaball_field((center_px.0 + 40.0, center_px.1), vp, &blobs, 0.0);
    let far = metaball_field((center_px.0 + 400.0, center_px.1), vp, &blobs, 0.0);
    assert!(
        at_center > near,
        "field peaks at the center: {at_center} > {near}"
    );
    assert!(near > far, "field decays with distance: {near} > {far}");
    assert!(
        at_center <= 1.0 + 1e-4,
        "peak field ~= weight 1.0: {at_center}"
    );
    assert!(far < 0.01, "far field is negligible: {far}");
}

#[test]
fn two_near_blobs_sum_higher_than_one_between_them() {
    let one = [[0.40f32, 0.5, 0.1, 1.0]];
    let two = [[0.40f32, 0.5, 0.1, 1.0], [0.46, 0.5, 0.1, 1.0]];
    let vp = (1000.0, 800.0);
    let mid_px = (0.43 * vp.0, 0.5 * vp.1);
    let f_one = metaball_field(mid_px, vp, &one, 0.0);
    let f_two = metaball_field(mid_px, vp, &two, 0.0);
    assert!(
        f_two > f_one,
        "summed field is higher between two blobs: {f_two} > {f_one}"
    );
}

#[test]
fn animation_moves_a_blob_between_distinct_phases_but_is_bounded() {
    let base_cy = 0.5;
    let vp = (1000.0, 800.0);
    let a = animated_center(2, 0.05, base_cy, 0.05, vp, 0.0);
    let b = animated_center(2, 0.05, base_cy, 0.05, vp, 0.25);
    assert!(
        (a.1 - b.1).abs() > 1e-3,
        "phase 0 vs 0.25 move the blob: {a:?} {b:?}"
    );
    for phase in [0.0, 0.1, 0.37, 0.5, 0.83, 0.99, 1.25, 1.99] {
        let (_, cy) = animated_center(2, 0.05, base_cy, 0.05, vp, phase);
        assert!((cy - base_cy).abs() < 0.09, "bob stays bounded: {cy}");
    }
}

#[test]
fn backdrop_layout_is_the_twelve_body_field_with_no_page_geometry_input() {
    let vp = (1200.0, 800.0);
    assert_eq!(
        BACKDROP_BLOBS.len(),
        MAX_BLOBS,
        "the shipped field is the full twelve-body population"
    );
    for b in BACKDROP_BLOBS {
        assert!((0.0..=1.0).contains(&b[0]));
        assert!((0.0..=1.0).contains(&b[1]));
        assert!(
            b[2] * vp.1 >= 75.0,
            "backdrop blob is substantial at 1200×800 — still a real lamp, not a dot"
        );
    }
}

#[test]
fn lava_field_has_a_three_scale_population_and_a_firmer_mean_radius() {
    let mean = BACKDROP_BLOBS.iter().map(|b| b[2]).sum::<f32>() / BACKDROP_BLOBS.len() as f32;
    assert!(
        (mean - 0.131).abs() < 0.01,
        "mean radius {mean:.4} drifted off the approved firmer silhouette (target ~0.131, \
         about 10% below the prior eight-body field's ~0.146)"
    );
    let small = BACKDROP_BLOBS.iter().filter(|b| b[2] <= 0.105).count();
    let medium = BACKDROP_BLOBS
        .iter()
        .filter(|b| (0.120..0.150).contains(&b[2]))
        .count();
    let large = BACKDROP_BLOBS.iter().filter(|b| b[2] >= 0.155).count();
    assert!(
        small >= 3 && medium >= 5 && large >= 2,
        "the large/medium/satellite hierarchy vanished: {small} small, {medium} medium, \
         {large} large"
    );
}

#[test]
fn lava_ambient_drift_reads_mainly_vertical() {
    // The horizontal excursion of every body over the full loop stays well
    // under half its own vertical excursion, in real pixels — the reduced
    // `LAVA_HORIZONTAL_SWAY` firming the silhouette so ambient drift reads
    // mainly vertical. The bound sits comfortably above the shipped worst
    // case (~0.32) but below what a reverted full-strength (1.0) sway would
    // produce (~0.61) — the axis a wide-sway regression trips first.
    for &(w, h) in &[(1200.0_f32, 800.0), (700.0, 1200.0)] {
        for (i, b) in BACKDROP_BLOBS.iter().enumerate() {
            let (mut min_x, mut max_x, mut min_y, mut max_y) =
                (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for step in 0..200 {
                let phase = step as f32 * LAVA_LOOP_CYCLES / 200.0;
                let (cx, cy) = animated_center(i, b[0], b[1], b[2], (w, h), phase);
                let (px, py) = (cx * w, cy * h);
                min_x = min_x.min(px);
                max_x = max_x.max(px);
                min_y = min_y.min(py);
                max_y = max_y.max(py);
            }
            let (range_x, range_y) = (max_x - min_x, max_y - min_y);
            assert!(
                range_x < 0.5 * range_y,
                "{w}x{h} body {i}: horizontal excursion {range_x:.2}px is not comfortably \
                 below vertical {range_y:.2}px (ratio {:.3}) — sway no longer reads contained",
                range_x / range_y
            );
        }
    }
}

#[test]
fn lava_field_contributes_to_both_margins_across_the_full_geometry_sweep() {
    // A body contributes when its animated centre (± its own nominal radius)
    // reaches past the writing column's edge — a geometry law, not a
    // sidecar proxy for pixels. The sweep spans short/tall AND narrow/wide
    // windows, the full authored page-width range (`range::PAGE_WIDTH_PROSE`
    // is 20..200 columns), and 1x/2x device DPI — the axis a single
    // hand-picked geometry would miss (CLAUDE.md's headline lesson). Uses
    // the SAME column formula the live app does (`render::column_left_for`/
    // `column_width_for`), never a parallel computation.
    const WINDOWS: [(f32, f32); 8] = [
        (600.0, 900.0),   // small, narrow, tall
        (700.0, 1200.0),  // narrow, very tall
        (900.0, 500.0),   // wide, short
        (1200.0, 800.0),  // baseline
        (1600.0, 900.0),  // laptop widescreen
        (2560.0, 1440.0), // wide desktop
        (3840.0, 1200.0), // ultra-wide, short relative to width
        (5120.0, 2756.0), // large HiDPI desktop
    ];
    const MEASURES: [usize; 6] = [20, 45, 70, 100, 140, 200];
    const PHASES: [f32; 4] = [0.0, 0.5, 1.0, 1.5];

    let mut contributed = [false; MAX_BLOBS];
    for &(w, h) in &WINDOWS {
        for &measure in &MEASURES {
            for &dpi in &[1.0f32, 2.0] {
                let char_width = crate::render::Metrics::with_dpi(1.0, dpi).char_width;
                let col_left = crate::render::column_left_for(w, char_width, true, measure);
                let col_right =
                    col_left + crate::render::column_width_for(w, char_width, true, measure);
                // Degenerate windows where the measure eats the whole width
                // leave no margin to fill — a floor of `column_width_for`'s
                // own formula, not a lava-population bug; those combos are
                // skipped for the "margin is non-empty" bar but still let
                // every body register its reach below.
                let has_left_margin = col_left > MARGIN_GAP_PX * 2.0;
                let has_right_margin = (w - col_right) > MARGIN_GAP_PX * 2.0;
                for &phase in &PHASES {
                    let mut left_n = 0;
                    let mut right_n = 0;
                    for (i, b) in BACKDROP_BLOBS.iter().enumerate() {
                        let (cx, _) = animated_center(i, b[0], b[1], b[2], (w, h), phase);
                        let cx_px = cx * w;
                        let r_px = b[2] * h;
                        if (cx_px - r_px) < col_left {
                            left_n += 1;
                            contributed[i] = true;
                        }
                        if (cx_px + r_px) > col_right {
                            right_n += 1;
                            contributed[i] = true;
                        }
                    }
                    assert!(
                        !has_left_margin || left_n >= 1,
                        "{w}x{h} measure={measure} dpi={dpi} phase={phase}: left margin is \
                         empty (col_left={col_left:.1})"
                    );
                    assert!(
                        !has_right_margin || right_n >= 1,
                        "{w}x{h} measure={measure} dpi={dpi} phase={phase}: right margin is \
                         empty (col_right={col_right:.1}, window={w})"
                    );
                }
            }
        }
    }
    assert!(
        contributed.iter().all(|&c| c),
        "some body never reaches a margin anywhere in the swept geometry: {contributed:?}"
    );
}

#[test]
fn lava_margin_field_never_reads_as_empty_or_a_solid_wall() {
    // Field-VALUE law (not geometric overlap), using the SHARED edge-blend
    // thresholds the shader itself renders through (`FROST_THRESHOLD` /
    // `FROST_EDGE_WIDTH` mirror `shaders/lava.wgsl`'s `THRESHOLD` /
    // `EDGE_WIDTH`, reused for both the plain lamp and the frost path):
    // across the swept geometry, a real margin samples both a genuinely LIT
    // point (field >= FROST_THRESHOLD, blended in) and a genuinely DARK
    // point (field below `FROST_THRESHOLD - FROST_EDGE_WIDTH`, where
    // `edge_t` is exactly 0 — the rendered pixel is bit-for-bit flat
    // ground) — the twelve bodies read as a field breathing over ground,
    // never a flat sheet of brightness nor a totally dark band.
    const WINDOWS: [(f32, f32); 6] = [
        (700.0, 1200.0),
        (900.0, 500.0),
        (1200.0, 800.0),
        (1600.0, 900.0),
        (2560.0, 1440.0),
        (5120.0, 2756.0),
    ];
    const MEASURES: [usize; 4] = [20, 70, 100, 200];
    const PHASES: [f32; 3] = [0.0, 0.7, 1.4];

    fn margin_extent(x0: f32, x1: f32, viewport: (f32, f32), phase: f32) -> (f32, f32) {
        let (w, h) = viewport;
        let mut min_f = f32::MAX;
        let mut max_f = f32::MIN;
        // Coarse grid — the "ground shows somewhere" floor.
        for xi in 0..12 {
            let x = x0 + (x1 - x0) * (xi as f32 + 0.5) / 12.0;
            for &yf in &[0.08_f32, 0.30, 0.50, 0.70, 0.92] {
                let f = metaball_field((x, yf * h), viewport, &BACKDROP_BLOBS, phase);
                min_f = min_f.min(f);
                max_f = max_f.max(f);
            }
        }
        // Every animated centre landing inside this margin band — the
        // guaranteed peak the coarse grid alone could straddle.
        for (i, b) in BACKDROP_BLOBS.iter().enumerate() {
            let (cx, cy) = animated_center(i, b[0], b[1], b[2], viewport, phase);
            let (px, py) = (cx * w, cy * h);
            if px >= x0 && px <= x1 {
                let f = metaball_field((px, py), viewport, &BACKDROP_BLOBS, phase);
                min_f = min_f.min(f);
                max_f = max_f.max(f);
            }
        }
        (min_f, max_f)
    }

    for &(w, h) in &WINDOWS {
        for &measure in &MEASURES {
            for &dpi in &[1.0f32, 2.0] {
                let char_width = crate::render::Metrics::with_dpi(1.0, dpi).char_width;
                let col_left = crate::render::column_left_for(w, char_width, true, measure);
                let col_right =
                    col_left + crate::render::column_width_for(w, char_width, true, measure);
                for &phase in &PHASES {
                    if col_left > 200.0 {
                        let (min_f, max_f) = margin_extent(0.0, col_left, (w, h), phase);
                        assert!(
                            max_f >= FROST_THRESHOLD,
                            "{w}x{h} measure={measure} dpi={dpi} phase={phase}: left margin \
                             never lights up (max {max_f:.3})"
                        );
                        assert!(
                            min_f < FROST_THRESHOLD - FROST_EDGE_WIDTH,
                            "{w}x{h} measure={measure} dpi={dpi} phase={phase}: left margin \
                             never shows ground (min {min_f:.3})"
                        );
                    }
                    if (w - col_right) > 200.0 {
                        let (min_f, max_f) = margin_extent(col_right, w, (w, h), phase);
                        assert!(
                            max_f >= FROST_THRESHOLD,
                            "{w}x{h} measure={measure} dpi={dpi} phase={phase}: right margin \
                             never lights up (max {max_f:.3})"
                        );
                        assert!(
                            min_f < FROST_THRESHOLD - FROST_EDGE_WIDTH,
                            "{w}x{h} measure={measure} dpi={dpi} phase={phase}: right margin \
                             never shows ground (min {min_f:.3})"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn page_width_only_occludes_or_reveals_the_same_backdrop_field() {
    let vp = (1200.0, 800.0);
    let px = (250.0, 400.0);
    let field = metaball_field(px, vp, &BACKDROP_BLOBS, 0.0);
    assert!(
        field > 0.5,
        "the immutable backdrop has visible lava at the probe: {field}"
    );
    assert!(column_mask(px.0, 300.0, 900.0, MARGIN_GAP_PX) > 0.0);
    assert_eq!(column_mask(px.0, 200.0, 1000.0, MARGIN_GAP_PX), 0.0);
    assert_eq!(field, metaball_field(px, vp, &BACKDROP_BLOBS, 0.0));
}

#[test]
fn backdrop_continues_behind_the_page_while_the_page_stays_flat() {
    let vp = (1200.0, 800.0);
    let b = BACKDROP_BLOBS[11]; // authored under the ordinary page footprint
    let center = animated_center(11, b[0], b[1], b[2], vp, 0.0);
    let px = (center.0 * vp.0, center.1 * vp.1);
    assert!(metaball_field(px, vp, &BACKDROP_BLOBS, 0.0) >= b[3]);
    assert_eq!(column_mask(px.0, 300.0, 900.0, MARGIN_GAP_PX), 0.0);
}

#[test]
fn column_mask_is_zero_inside_the_column_and_full_in_the_margin() {
    let (col_left, col_right, gap) = (300.0, 900.0, 28.0);
    assert_eq!(column_mask(600.0, col_left, col_right, gap), 0.0);
    assert_eq!(
        column_mask(col_left, col_left, col_right, gap),
        0.0,
        "0 AT the edge"
    );
    assert_eq!(
        column_mask(col_right, col_left, col_right, gap),
        0.0,
        "0 AT the far edge"
    );
    assert!((column_mask(col_left - gap, col_left, col_right, gap) - 1.0).abs() < 1e-4);
    assert!((column_mask(col_right + gap, col_left, col_right, gap) - 1.0).abs() < 1e-4);
    assert_eq!(column_mask(20.0, col_left, col_right, gap), 1.0);
}

#[test]
fn rail_carve_flattens_the_left_margin_and_keeps_the_right_byte_identical() {
    let (col_left, col_right, gap) = (300.0, 900.0, MARGIN_GAP_PX);
    for x in [
        0.0, 20.0, 150.0, 272.0, 285.0, 300.0, 600.0, 900.0, 914.0, 928.0, 1100.0,
    ] {
        assert_eq!(
            lava_mask(x, col_left, col_right, gap, false),
            column_mask(x, col_left, col_right, gap),
            "rail off is the plain column mask at x={x}"
        );
    }
    for x in [0.0, 5.0, 20.0, 150.0, 271.9, 285.0, 299.0] {
        assert_eq!(
            lava_mask(x, col_left, col_right, gap, true),
            0.0,
            "the rail band holds no lava at x={x}"
        );
    }
    assert_eq!(column_mask(150.0, col_left, col_right, gap), 1.0);
    assert_eq!(lava_mask(600.0, col_left, col_right, gap, true), 0.0);
    for x in [900.0, 910.0, 914.0, 928.0, 1000.0, 1199.0] {
        assert_eq!(
            lava_mask(x, col_left, col_right, gap, true),
            column_mask(x, col_left, col_right, gap),
            "the right margin keeps the lamp untouched at x={x}"
        );
    }
}

#[test]
fn rail_carve_moves_the_glow_distance_off_the_left_edge() {
    let (col_left, col_right) = (300.0, 900.0);
    for x in [301.0, 330.0, 355.0] {
        let carved = rail_dist_outside(x, col_left, col_right, true);
        assert!(
            carved < -100.0,
            "left-edge glow is structurally unreachable when carved: x={x} dist={carved}"
        );
        let plain = rail_dist_outside(x, col_left, col_right, false);
        assert!(plain > -60.0 && plain < 0.0, "uncarved x={x} dist={plain}");
    }
    for x in [850.0, 875.0, 899.0] {
        assert_eq!(
            rail_dist_outside(x, col_left, col_right, true),
            rail_dist_outside(x, col_left, col_right, false),
            "right-edge glow distance unchanged at x={x}"
        );
    }
}

#[test]
fn gutter_corner_carve_zeroes_only_its_bounds_and_keeps_both_margins() {
    let (col_left, col_right, gap) = (300.0, 900.0, MARGIN_GAP_PX);
    let rect = [0.0, 820.0, 260.0, 1000.0];
    for &(x, y) in &[
        (20.0, 900.0),
        (150.0, 400.0),
        (600.0, 500.0),
        (1000.0, 900.0),
    ] {
        assert_eq!(
            lava_mask_2d(x, y, col_left, col_right, gap, false, None),
            column_mask(x, col_left, col_right, gap),
            "gutter None is the plain column mask at ({x},{y})"
        );
    }
    for &(x, y) in &[(20.0, 970.0), (120.0, 900.0), (200.0, 860.0)] {
        assert_eq!(column_mask(x, col_left, col_right, gap), 1.0);
        assert_eq!(
            lava_mask_2d(x, y, col_left, col_right, gap, false, Some(rect)),
            0.0,
            "the gutter corner band holds no lava at ({x},{y})"
        );
    }
    for &(x, y) in &[(20.0, 200.0), (150.0, 400.0), (120.0, 600.0)] {
        assert_eq!(
            lava_mask_2d(x, y, col_left, col_right, gap, false, Some(rect)),
            column_mask(x, col_left, col_right, gap),
            "the left margin above the gutter band keeps its lamp at ({x},{y})"
        );
    }
    for &(x, y) in &[(950.0, 900.0), (1000.0, 970.0), (1100.0, 500.0)] {
        assert_eq!(
            lava_mask_2d(x, y, col_left, col_right, gap, false, Some(rect)),
            column_mask(x, col_left, col_right, gap),
            "the right margin keeps its lamp beside a gutter corner carve at ({x},{y})"
        );
    }
}

#[test]
fn gutter_corner_dist_outside_is_a_box_signed_distance() {
    let rect = [0.0, 820.0, 260.0, 1000.0];
    assert!(gutter_corner_dist_outside(120.0, 900.0, rect) < 0.0);
    assert!((gutter_corner_dist_outside(270.0, 900.0, rect) - 10.0).abs() < 1e-4);
    assert!((gutter_corner_dist_outside(120.0, 800.0, rect) - 20.0).abs() < 1e-4);
    let (col_left, col_right, gap) = (300.0, 900.0, MARGIN_GAP_PX);
    let above = 820.0 - gap - 1.0;
    assert!(
        (lava_mask_2d(120.0, above, col_left, col_right, gap, false, Some(rect))
            - column_mask(120.0, col_left, col_right, gap))
        .abs()
            < 1e-4,
        "a full gap above the corner band the lamp is back to full"
    );
}

#[test]
fn column_mask_ramps_monotonically_across_the_feather() {
    let (col_left, col_right, gap) = (300.0, 900.0, 40.0);
    let mut prev = column_mask(col_left, col_left, col_right, gap);
    for k in 1..=40 {
        let x = col_left - k as f32; // stepping out into the left margin
        let m = column_mask(x, col_left, col_right, gap);
        assert!(
            m >= prev - 1e-6,
            "mask ramps monotonically at x={x}: {m} >= {prev}"
        );
        prev = m;
    }
    assert!(
        (prev - 1.0).abs() < 1e-4,
        "settled at full strength: {prev}"
    );
}

#[test]
fn lava_ticks_only_when_active_ambient_on_not_reduced_and_focused() {
    assert!(
        lava_should_tick(true, true, false, true, false),
        "all conditions met → tick"
    );
    assert!(
        !lava_should_tick(false, true, false, true, false),
        "non-lava world never ticks"
    );
    assert!(
        !lava_should_tick(true, false, false, true, false),
        "ambient_motion off → no tick"
    );
    assert!(
        !lava_should_tick(true, true, true, true, false),
        "reduce motion → no tick"
    );
    assert!(
        !lava_should_tick(true, true, false, false, false),
        "blurred → paused, no tick"
    );
    assert!(
        !lava_should_tick(true, true, false, true, true),
        "resize, move, or blur pause holds phase"
    );
}

#[test]
fn any_transient_live_interaction_pauses_the_lamp() {
    assert!(
        !lava_paused(false, false, false),
        "truly idle: the lamp may drift"
    );
    assert!(
        lava_paused(true, false, false),
        "a live RESIZE stream holds it"
    );
    assert!(
        lava_paused(false, true, false),
        "a live MOVE stream holds it"
    );
    assert!(
        lava_paused(false, false, true),
        "a blur-eligible overlay (frost) holds it"
    );
    assert!(lava_paused(true, true, false));
}

#[test]
fn field_viewport_holds_settled_geometry_until_explicit_snap() {
    let mut settled = [1200.0, 800.0];
    assert_eq!(field_viewport([1320.0, 840.0], settled), settled);
    assert_eq!(
        field_viewport([1400.0, 900.0], settled),
        settled,
        "successive resize ticks keep the same field"
    );
    settled = [1400.0, 900.0];
    assert_eq!(
        field_viewport([1400.0, 900.0], settled),
        [1400.0, 900.0],
        "settle snaps exactly once to the final viewport"
    );
    assert_eq!(
        field_viewport([1400.0, 900.0], [0.0, 0.0]),
        [1400.0, 900.0],
        "first frame falls back to live geometry"
    );
}

#[test]
fn blur_capture_relaxes_only_the_lava_posterization_invariant() {
    assert!(dither_for_blur(true, false), "live Mangrove stays dithered");
    assert!(!dither_for_blur(true, true), "frost source is smooth");
    assert!(!dither_for_blur(false, false), "Firetail stays smooth");
    assert!(!dither_for_blur(false, true), "blur never invents dither");
}

#[test]
fn env_override_wins_then_reduced_freeze_then_stored() {
    assert_eq!(lava_phase_for(0.7, false, Some(0.35)), 0.35);
    assert_eq!(lava_phase_for(0.7, true, Some(0.35)), 0.35);
    assert_eq!(lava_phase_for(0.7, true, None), LAVA_FROZEN_PHASE);
    assert_eq!(lava_phase_for(0.7, false, None), 0.7);
}

#[test]
fn capture_default_phase_is_frozen_t0() {
    assert_eq!(lava_phase_for(LAVA_FROZEN_PHASE, false, None), 0.0);
    assert_eq!(LAVA_FROZEN_PHASE, 0.0);
}

#[test]
fn advance_phase_moves_forward_and_wraps_over_the_full_field_period() {
    let p = advance_phase(0.0, 1.0);
    assert!(
        p > 0.0 && p < LAVA_LOOP_CYCLES,
        "one second advances within a cycle: {p}"
    );
    let w = advance_phase(1.999, 1.0);
    assert!(
        (0.0..LAVA_LOOP_CYCLES).contains(&w),
        "wrapped into the two-cycle interval: {w}"
    );
    assert!(advance_phase(0.1, 0.5) > 0.1);
}

#[test]
fn two_cycle_endpoint_is_seamless_for_every_blob_center() {
    let vp = (1200.0, 800.0);
    for (i, b) in BACKDROP_BLOBS.iter().enumerate() {
        for start in [0.0, 0.17, 0.63, 1.21] {
            let a = animated_center(i, b[0], b[1], b[2], vp, start);
            let z = animated_center(i, b[0], b[1], b[2], vp, start + LAVA_LOOP_CYCLES);
            assert!(
                (a.0 - z.0).abs() < 1e-6 && (a.1 - z.1).abs() < 1e-6,
                "blob {i} does not meet its two-cycle endpoint from {start}: {a:?} vs {z:?}"
            );
        }
    }
    let b = BACKDROP_BLOBS[1];
    let at_zero = animated_center(1, b[0], b[1], b[2], vp, 0.0);
    let at_one = animated_center(1, b[0], b[1], b[2], vp, 1.0);
    assert!((at_zero.0 - at_one.0).abs() > 1e-4);

    for px in [
        (24.0, 40.0),
        (160.0, 400.0),
        (600.0, 300.0),
        (1140.0, 720.0),
    ] {
        let a = metaball_field(px, vp, &BACKDROP_BLOBS, 0.0);
        let z = metaball_field(px, vp, &BACKDROP_BLOBS, LAVA_LOOP_CYCLES);
        assert!(
            (a - z).abs() < 1e-6,
            "metaball field does not meet its two-cycle endpoint at {px:?}: {a} vs {z}"
        );
    }
}

#[test]
fn delayed_ambient_ticks_advance_at_most_one_fixed_step() {
    assert_eq!(ambient_tick_dt(LAVA_TICK_SECONDS), LAVA_TICK_SECONDS);
    assert_eq!(ambient_tick_dt(8.0), LAVA_TICK_SECONDS);
    assert_eq!(ambient_tick_dt(-1.0), 0.0);

    let ordinary = advance_phase(0.4, LAVA_TICK_SECONDS);
    let delayed = advance_phase(0.4, 8.0);
    assert_eq!(
        delayed, ordinary,
        "an eight-second event-loop stall must advance exactly one ambient tick, never catch up"
    );
    assert!((ordinary - 0.4 - LAVA_TICK_SECONDS * LAVA_SPEED).abs() < 1e-6);
}

/// The gallery knob's grammar. The `<edge>` token (`hard` | `glow`) went with
/// the dial it set — and it is REJECTED now rather than accepted-and-ignored,
/// which is the half worth law-testing: a stale `AWL_LAVA=warm:0.5:hard` in
/// someone's shell must fail loudly instead of quietly rendering the one
/// treatment while appearing to ask for the other.
#[test]
fn parse_spec_reads_palette_phase_and_dither_and_rejects_the_retired_edge_token() {
    let (bg, phase) = parse_spec("deepsea:0.35:dither").unwrap();
    assert_eq!(phase, 0.35);
    match bg {
        Background::Lava { dithered, .. } => assert!(dithered),
        _ => panic!("expected a Lava background"),
    }
    let (bg2, _) = parse_spec("warm:0.0").unwrap();
    match bg2 {
        Background::Lava { dithered, .. } => assert!(!dithered),
        _ => panic!("expected a Lava background"),
    }
    assert!(
        parse_spec("warm:0.5:hard").is_none(),
        "the retired `hard` edge token must be rejected, not silently ignored"
    );
    assert!(
        parse_spec("deepsea:0.35:glow:dither").is_none(),
        "the retired `glow` edge token must be rejected, not silently ignored"
    );
    assert!(parse_spec("nope:0.0").is_none());
    assert!(parse_spec("warm:notanumber").is_none());
    assert!(parse_spec("warm:0.0:bogus").is_none());
}

#[test]
fn frost_is_the_shipped_default() {
    assert!(
        std::hint::black_box(FROST_RAIL_DEFAULT),
        "the user's pick — frost ships"
    );
    assert!(frost_on(), "frost is on by default (no gallery knob)");
}

#[test]
fn frost_dimensions_scale_with_zoom_and_device_dpi() {
    for zoom in [0.8_f32, 1.0, 1.25] {
        for logical in [FROST_BLUR_PX, FROST_FEATHER_PX, FROST_PILL_PAD_X] {
            let one = frost_px(logical, zoom, 1.0);
            let two = frost_px(logical, zoom, 2.0);
            assert!(
                (two - 2.0 * one).abs() < f32::EPSILON,
                "logical Frost dimension {logical} at zoom {zoom}: 2× physical {two} \
                 must be exactly twice 1× {one}"
            );
        }
    }
}

#[test]
fn frost_field_softens_the_smooth_field() {
    let blobs = [[0.5f32, 0.5, 0.1, 1.0]];
    let vp = (1000.0, 800.0);
    let center = animated_center(0, 0.5, 0.5, 0.1, vp, 0.0);
    let cpx = (center.0 * vp.0, center.1 * vp.1);
    let raw = metaball_field(cpx, vp, &blobs, 0.0);
    let soft = frost_field(cpx, vp, &blobs, 0.0, FROST_BLUR_PX);
    assert!(
        soft > 0.0 && soft < raw,
        "blurred peak sits below the raw peak: {soft} < {raw}"
    );
    let far = frost_field((cpx.0 + 400.0, cpx.1), vp, &blobs, 0.0, FROST_BLUR_PX);
    assert!(far < 0.01, "bare ground stays dark under the blur: {far}");
}

#[test]
fn frost_seed_bump_is_one_on_the_ink_and_decays_to_zero_by_a_radius() {
    let seed = [100.0f32, 300.0, 230.0, 40.0];
    assert!(
        (frost_seed_bump(200.0, 230.0, seed) - 1.0).abs() < 1e-6,
        "1 on the ink"
    );
    assert!(
        (frost_seed_bump(100.0, 230.0, seed) - 1.0).abs() < 1e-6,
        "1 at the run's left end"
    );
    assert_eq!(
        frost_seed_bump(300.0 + 41.0, 230.0, seed),
        0.0,
        "0 past a radius right"
    );
    assert_eq!(
        frost_seed_bump(200.0, 230.0 + 41.0, seed),
        0.0,
        "0 past a radius up"
    );
    let mut prev = 1.0;
    for k in 0..=42 {
        let x = 300.0 + k as f32;
        let b = frost_seed_bump(x, 230.0, seed);
        assert!(
            b <= prev + 1e-6,
            "bump decays monotonically at x={x}: {b} <= {prev}"
        );
        prev = b;
    }
    assert!(
        (frost_seed_bump(200.0, 230.0 - 20.0, seed) - frost_seed_bump(200.0, 230.0 + 20.0, seed))
            .abs()
            < 1e-6,
        "the halo is symmetric above/below the ink"
    );
}

#[test]
fn frost_coverage_merges_nearby_seeds_and_splits_far_ones() {
    let r = 40.0f32;
    let near = [[100.0f32, 100.0, 200.0, r], [150.0, 150.0, 200.0, r]];
    assert!(
        frost_coverage(125.0, 200.0, &near) > 0.5,
        "nearby seeds bridge into one island"
    );
    let far = [[100.0f32, 100.0, 200.0, r], [240.0, 240.0, 200.0, r]];
    assert!(
        frost_coverage(170.0, 200.0, &far) < 0.5,
        "far seeds leave a live gap between islands"
    );
    assert!(
        frost_coverage(100.0, 200.0, &far) > 0.5,
        "each far seed still frosts its own core"
    );
    let stacked = [[100.0f32, 200.0, 200.0, r], [100.0, 200.0, 250.0, r]];
    assert!(
        frost_coverage(150.0, 225.0, &stacked) > 0.5,
        "vertically-close rows merge organically"
    );
}

#[test]
fn frost_coverage_frosts_the_ink_and_empty_is_inert() {
    let a = [10.0f32, 40.0, 20.0, 40.0];
    let b = [200.0f32, 240.0, 220.0, 40.0];
    assert!(
        frost_coverage(25.0, 20.0, &[a, b]) > 0.999,
        "over seed A's ink → frosted"
    );
    assert!(
        frost_coverage(220.0, 220.0, &[a, b]) > 0.999,
        "over seed B's ink → frosted"
    );
    assert_eq!(
        frost_coverage(1000.0, 1000.0, &[a, b]),
        0.0,
        "far from every seed the lamp is live"
    );
    assert_eq!(
        frost_coverage(25.0, 20.0, &[]),
        0.0,
        "an empty seed list frosts nothing"
    );
}

#[test]
fn frost_pixel_dims_toward_ground_and_stays_bounded() {
    let ground = Srgb {
        r: 0x17,
        g: 0x09,
        b: 0x0c,
        a: 0xff,
    };
    let lo = Srgb {
        r: 0x24,
        g: 0x0c,
        b: 0x14,
        a: 0xff,
    };
    let hi = Srgb {
        r: 0x52,
        g: 0x18,
        b: 0x2c,
        a: 0xff,
    };
    let dark = frost_pixel(0.0, ground, lo, hi, FROST_DIM);
    assert_eq!(
        (dark.r, dark.g, dark.b),
        (ground.r, ground.g, ground.b),
        "no blob → flat ground"
    );
    let bright = frost_pixel(1.0, ground, lo, hi, FROST_DIM);
    let lerp = |a: u8, b: u8, t: f32| (a as f32 + (b as f32 - a as f32) * t).round() as i32;
    let bound = (
        lerp(hi.r, ground.r, FROST_DIM),
        lerp(hi.g, ground.g, FROST_DIM),
        lerp(hi.b, ground.b, FROST_DIM),
    );
    assert_eq!(
        (bright.r as i32, bright.g as i32, bright.b as i32),
        bound,
        "saturated frost == the worst bound"
    );
    assert!(
        (bright.r as i32) < hi.r as i32,
        "the value dim pulls the pill back toward ground"
    );
}
