use super::*;

// The RESPONSIVE page column: `min(measure_px, window - 2*margin)`, centered, with
// the margin collapsing from the generous `page_min_margin` to the small
// `PAGE_MIN_PAD` as the measure crowds the width. These exercise the pure formula
// (no GPU, no page globals) across the WIDE / NARROW / transition regimes.
const CW: f32 = CHAR_WIDTH; // 14.4

#[test]
fn wide_window_seats_centered_column_at_measure() {
    let measure_px = 40.0 * CW; // 576
    let w = column_width_for(1200.0, CW, true, 40);
    let left = column_left_for(1200.0, CW, true, 40);
    assert!(
        (w - measure_px).abs() < 1e-3,
        "wide: column == measure, got {w}"
    );
    assert!(
        (left - (1200.0 - measure_px) * 0.5).abs() < 1e-3,
        "wide: centered, got {left}"
    );
    assert!(
        left > page_min_margin(1200.0) - 1e-3,
        "wide leftover >= generous margin"
    );
}

#[test]
fn narrow_window_fills_minus_small_pad() {
    for &win in &[300.0_f32, 400.0, 700.0] {
        let w = column_width_for(win, CW, true, 80); // 80-char measure ~1152px >> win
        let left = column_left_for(win, CW, true, 80);
        assert!(
            (w - (win - 2.0 * PAGE_MIN_PAD)).abs() < 1e-3,
            "narrow {win}: fills minus pad, got {w}"
        );
        assert!(
            (left - PAGE_MIN_PAD).abs() < 1e-3,
            "narrow {win}: left at small pad, got {left}"
        );
        assert!(
            w + 2.0 * left <= win + 1e-3,
            "narrow {win}: never overflows"
        );
    }
}

#[test]
fn column_is_monotonic_and_never_overflows_across_a_resize() {
    let measure_px = 80.0 * CW;
    let mut prev = 0.0_f32;
    let mut w = 200.0;
    while w <= 2600.0 {
        let col = column_width_for(w, CW, true, 80);
        let left = column_left_for(w, CW, true, 80);
        assert!(
            col >= prev - 1e-3,
            "column must not shrink as window grows (w={w})"
        );
        assert!(
            col <= measure_px + 1e-3,
            "column never exceeds the measure (w={w})"
        );
        assert!(
            left >= PAGE_MIN_PAD - 1e-3,
            "always at least the small pad (w={w})"
        );
        assert!(
            col + 2.0 * left <= w + 1e-2,
            "never overflows the window (w={w})"
        );
        prev = col;
        w += 50.0;
    }
    assert!((column_width_for(2600.0, CW, true, 80) - measure_px).abs() < 1e-3);
}

#[test]
fn wide_capture_is_byte_identical_to_the_old_cap() {
    let measure_px = 40.0 * CW; // 576
    assert!((column_width_for(1200.0, CW, true, 40) - measure_px).abs() < 1e-3);
    assert!((column_left_for(1200.0, CW, true, 40) - (1200.0 - measure_px) * 0.5).abs() < 1e-3);
}

#[test]
fn page_off_is_edge_to_edge_unaffected() {
    assert!((column_left_for(1200.0, CW, false, 80) - NONPAGE_INSET).abs() < 1e-3);
    assert!(
        (column_width_for(1200.0, CW, false, 80) - (1200.0 - 2.0 * NONPAGE_INSET)).abs() < 1e-3
    );
    assert!(std::hint::black_box(NONPAGE_INSET) > PAGE_MIN_PAD);
}

fn outline_pref_px() -> f32 {
    rowlayout::OUTLINE_PREFERRED_CHARS as f32 * CW * crate::markdown::type_scale::LABEL
}
fn outline_min_px() -> f32 {
    rowlayout::OUTLINE_MIN_CHARS as f32 * CW * crate::markdown::type_scale::LABEL
}
fn margin_gap() -> f32 {
    CW * crate::render::chrome::MARGIN_COLUMN_GAP_CHARS.0
}
const ADAPTIVE_LEFT_PAD: f32 = TEXT_LEFT;

#[test]
fn adaptive_wide_window_is_byte_identical_to_symmetric() {
    let left = adaptive_column_left(
        1200.0,
        CW,
        true,
        40,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    let symmetric = column_left_for(1200.0, CW, true, 40);
    assert_eq!(left, symmetric, "wide: adaptive placement changes nothing");
}

#[test]
fn adaptive_outline_not_wanted_never_shifts_even_when_narrow() {
    let left = adaptive_column_left(
        900.0,
        CW,
        true,
        40,
        false,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    let symmetric = column_left_for(900.0, CW, true, 40);
    assert_eq!(left, symmetric);
}

#[test]
fn adaptive_page_off_never_shifts() {
    let left = adaptive_column_left(
        900.0,
        CW,
        false,
        40,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    assert_eq!(left, NONPAGE_INSET);
}

#[test]
fn adaptive_narrow_window_shifts_right_and_grants_the_full_preferred_rail() {
    let win = 900.0;
    let measure = 40usize;
    let symmetric = column_left_for(win, CW, true, measure);
    let width = column_width_for(win, CW, true, measure);
    let pref = outline_pref_px();
    let min = outline_min_px();
    let gap = margin_gap();
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        pref,
        min,
        gap,
        ADAPTIVE_LEFT_PAD,
    );
    assert!(
        left > symmetric,
        "narrow: column shifts right, got {left} vs symmetric {symmetric}"
    );
    let avail = (left - gap) - ADAPTIVE_LEFT_PAD;
    assert!(
        (avail - pref).abs() < 1.0,
        "narrow: outline granted its full preferred rail (within the whole-pixel snap), avail={avail} pref={pref}"
    );
    assert_eq!(
        left,
        (pref + gap + ADAPTIVE_LEFT_PAD).floor(),
        "narrow: the granted left is exactly the snapped desired_left"
    );
    let total_margin = win - width;
    let right_margin = total_margin - left;
    assert!(
        right_margin >= RIGHT_MARGIN_BREATH - 1e-3,
        "narrow: right margin keeps its breathing floor, got {right_margin}"
    );
}

#[test]
fn adaptive_narrow_shift_caps_at_the_right_margin_breathing_floor() {
    let win = 800.0;
    let measure = 40usize;
    let width = column_width_for(win, CW, true, measure);
    let total_margin = win - width;
    let symmetric = column_left_for(win, CW, true, measure);
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    assert!(
        left > symmetric,
        "still shifts right from the symmetric position"
    );
    let right_margin = total_margin - left;
    assert!(
        (right_margin - RIGHT_MARGIN_BREATH).abs() < 0.5,
        "capped exactly at the breathing floor, got {right_margin}"
    );
    let avail = (left - margin_gap()) - ADAPTIVE_LEFT_PAD;
    assert!(
        avail < outline_pref_px() - 1.0,
        "granted rail is LESS than the full preference (capped by the floor), avail={avail}"
    );
    assert!(
        (avail / (CW * crate::markdown::type_scale::LABEL)).floor()
            >= rowlayout::OUTLINE_MIN_CHARS as f32,
        "but still comfortably above the hard hide floor"
    );
}

#[test]
fn adaptive_narrowest_window_recenters_instead_of_overshooting_the_right_margin() {
    let win = 300.0;
    let measure = 80usize; // way more than fits at 300px
    let symmetric = column_left_for(win, CW, true, measure);
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    assert_eq!(
        left, symmetric,
        "narrowest: no shift possible, column re-centers exactly"
    );
}

#[test]
fn adaptive_no_payoff_shift_recenters_instead_of_shifting_for_a_hidden_outline() {
    let win = 1100.0;
    let measure = 70usize;
    let symmetric = column_left_for(win, CW, true, measure);
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    assert_eq!(
        left, symmetric,
        "a shift that can't clear the outline's own minimum rail must not happen at all"
    );
    let width = column_width_for(win, CW, true, measure);
    let total_margin = win - width;
    let old_max_left = (total_margin - RIGHT_MARGIN_BREATH).max(0.0);
    assert!(
        old_max_left > symmetric,
        "fixture: the old formula would have shifted"
    );
    let old_avail = (old_max_left - margin_gap()) - ADAPTIVE_LEFT_PAD;
    let label_char_w = CW * crate::markdown::type_scale::LABEL;
    let old_avail_chars = (old_avail / label_char_w).floor().max(0.0) as usize;
    assert!(
        old_avail_chars < rowlayout::OUTLINE_MIN_CHARS,
        "fixture: the old shift would still leave the outline below its hide floor"
    );
}

#[test]
fn adaptive_threshold_boundary_resolves_to_wide_not_narrow() {
    let pref = outline_pref_px();
    let min = outline_min_px();
    let gap = margin_gap();
    let desired_left = pref + gap + ADAPTIVE_LEFT_PAD;
    let measure = 40usize;
    let measure_px = measure as f32 * CW;
    let win = measure_px + 2.0 * desired_left;
    let symmetric = column_left_for(win, CW, true, measure);
    assert!(
        (symmetric - desired_left).abs() < 1.0,
        "fixture: symmetric lands at desired_left, got {symmetric} vs {desired_left}"
    );
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        pref,
        min,
        gap,
        ADAPTIVE_LEFT_PAD,
    );
    assert!(
        (left - symmetric.floor()).abs() < 1e-3,
        "boundary resolves to WIDE (no shift) at the exact threshold: left={left} symmetric={symmetric}"
    );
}

#[test]
fn adaptive_never_shrinks_the_column_only_moves_where_it_sits() {
    for &(win, measure) in &[(1200.0_f32, 40usize), (900.0, 40), (800.0, 40), (300.0, 80)] {
        let width = column_width_for(win, CW, true, measure);
        let left = adaptive_column_left(
            win,
            CW,
            true,
            measure,
            true,
            outline_pref_px(),
            outline_min_px(),
            margin_gap(),
            ADAPTIVE_LEFT_PAD,
        );
        assert!(
            left + width <= win + 1e-2,
            "shifted column must still fit the window (win={win} measure={measure}): left={left} width={width}"
        );
    }
}

#[test]
fn adaptive_entry_ramp_is_continuous_no_more_46px_jump() {
    let pref = outline_pref_px();
    let min = outline_min_px();
    let gap = margin_gap();
    let mut prev: Option<f32> = None;
    for w in 1090..=1170 {
        let left = adaptive_column_left(
            w as f32,
            CW,
            true,
            70,
            true,
            pref,
            min,
            gap,
            ADAPTIVE_LEFT_PAD,
        );
        if let Some(p) = prev {
            let step = left - p;
            assert!(
                step >= -1e-3,
                "width {w}px: column_left decreased ({p} -> {left})"
            );
            assert!(
                step <= 20.0,
                "width {w}px: column_left jumped {step}px in a single pixel of resize ({p} -> {left}) — the jitter bug"
            );
        }
        prev = Some(left);
    }
}

#[test]
fn adaptive_ramp_still_recenters_well_outside_the_ramp_band() {
    let win = 1100.0;
    let measure = 70usize;
    let symmetric = column_left_for(win, CW, true, measure);
    let left = adaptive_column_left(
        win,
        CW,
        true,
        measure,
        true,
        outline_pref_px(),
        outline_min_px(),
        margin_gap(),
        ADAPTIVE_LEFT_PAD,
    );
    assert_eq!(
        left, symmetric,
        "well outside the ramp band: still a bare recenter, no partial shift"
    );
}

#[test]
fn adaptive_left_snaps_to_whole_physical_pixels_across_a_1px_sweep() {
    let pref = outline_pref_px();
    let min = outline_min_px();
    let gap = margin_gap();
    for wants in [false, true] {
        let mut prev: Option<f32> = None;
        for w in 1000..=1400u32 {
            let left = adaptive_column_left(
                w as f32,
                CW,
                true,
                70,
                wants,
                pref,
                min,
                gap,
                ADAPTIVE_LEFT_PAD,
            );
            assert_eq!(
                left,
                left.floor(),
                "width {w} (wants={wants}): left must be a whole physical pixel, got {left}"
            );
            if let (Some(p), false) = (prev, wants) {
                let step = left - p;
                assert!(
                    step == 0.0 || step == 1.0,
                    "width {w}: symmetric-regime left must step exactly 0 or 1 whole px per width px, got {step}"
                );
            }
            prev = Some(left);
        }
    }
}

#[test]
fn page_column_advance_strips_zoom_keeps_dpi() {
    for &dpi in &[1.0_f32, 2.0] {
        let base = CW * dpi;
        for &zoom in &[0.5_f32, 1.0, 1.6, 2.5, 3.0] {
            let live = CW * zoom * dpi; // == metrics.char_width
            let adv = page_column_advance(live, zoom);
            assert!(
                (adv - base).abs() < 1e-3,
                "zoom={zoom} dpi={dpi}: advance must be zoom-free"
            );
        }
    }
    // Zoom 1.0 (the deterministic capture path) is an exact identity.
    assert!((page_column_advance(CW, 1.0) - CW).abs() < 1e-6);
}

#[test]
fn zooming_in_keeps_column_and_margins_constant_gutter_stays() {
    let window = 1200.0;
    let measure = 40; // narrow measure -> generous, clearly-present margins
    let base_adv = page_column_advance(CW, 1.0);
    let ref_w = column_width_for(window, base_adv, true, measure);
    let ref_left = column_left_for(window, base_adv, true, measure);
    assert!(
        ref_left > PAGE_MIN_PAD + 1.0,
        "fixture must have a visible margin/gutter"
    );
    for &zoom in &[0.5_f32, 1.0, 1.6, 2.5, 3.0] {
        let live = CW * zoom; // metrics.char_width at this zoom (dpi 1.0)
        let adv = page_column_advance(live, zoom);
        let w = column_width_for(window, adv, true, measure);
        let left = column_left_for(window, adv, true, measure);
        assert!(
            (w - ref_w).abs() < 1e-3,
            "zoom={zoom}: column px must not change (got {w}, want {ref_w})"
        );
        assert!(
            (left - ref_left).abs() < 1e-3,
            "zoom={zoom}: left margin must not change"
        );
        let right = window - left - w;
        let ref_right = window - ref_left - ref_w;
        assert!(
            (right - ref_right).abs() < 1e-3,
            "zoom={zoom}: right margin must not change"
        );
    }
}

#[test]
fn hover_zone_arms_only_within_grab_px_of_an_edge() {
    let measure_px = 40.0 * CW; // 576
    let left = (1200.0 - measure_px) * 0.5; // 312
    let tol = PAGE_RESIZE_GRAB_PX;
    assert_eq!(
        page_boundary_hit(left, left, measure_px, tol),
        Some(ResizeEdge::Left)
    );
    assert_eq!(
        page_boundary_hit(left + tol - 0.5, left, measure_px, tol),
        Some(ResizeEdge::Left)
    );
    assert_eq!(
        page_boundary_hit(left + tol + 2.0, left, measure_px, tol),
        None
    );
    let right = left + measure_px; // 888
    assert_eq!(
        page_boundary_hit(right - 1.0, left, measure_px, tol),
        Some(ResizeEdge::Right)
    );
    assert_eq!(page_boundary_hit(600.0, left, measure_px, tol), None);
}

#[test]
fn resize_affordance_arms_at_both_drawn_edges_in_every_page_on_cell() {
    // THE LOCKOUT LAW (bug, 2026-07-15): in page mode the resize affordance must
    // arm at BOTH drawn column edges for every measure × window — ESPECIALLY the
    // collapsed cells (column pinned at the PAGE_MIN_PAD margins) where the old
    // `left <= PAGE_MIN_PAD + 1.0 → None` guard killed the affordance and locked the
    // user out of dragging a widened-past-capacity column back inward. Drives the
    // ONE arming owner `page_resize_edge_hit` against the DRAWN geometry
    // (`column_left_for`/`column_width_for`), so a reintroduced collapse-guard fails
    // here. Pure — no GPU, no page globals.
    let tol = PAGE_RESIZE_GRAB_PX;
    let adv = CW; // zoom-stripped page-column advance
    let mut saw_collapsed = false;
    for &measure in &[20usize, 40, 70, 100, 140] {
        for &window in &[600.0f32, 900.0, 1200.0, 2400.0] {
            let left = column_left_for(window, adv, true, measure);
            let width = column_width_for(window, adv, true, measure);
            let right = left + width;
            let cell = format!("measure={measure} window={window}");

            assert_eq!(
                page_resize_edge_hit(true, left, width, left, tol),
                Some(ResizeEdge::Left),
                "{cell}: left edge must arm",
            );
            assert_eq!(
                page_resize_edge_hit(true, left, width, right, tol),
                Some(ResizeEdge::Right),
                "{cell}: right edge must arm",
            );
            assert!(
                page_resize_edge_hit(true, left, width, left + tol - 0.5, tol).is_some(),
                "{cell}: just inside the left edge must arm",
            );
            assert!(
                page_resize_edge_hit(true, left, width, right - (tol - 0.5), tol).is_some(),
                "{cell}: just inside the right edge must arm",
            );

            assert_eq!(
                page_resize_edge_hit(false, left, width, left, tol),
                None,
                "{cell}: page off must not arm (left)",
            );
            assert_eq!(
                page_resize_edge_hit(false, left, width, right, tol),
                None,
                "{cell}: page off must not arm (right)",
            );

            if left <= PAGE_MIN_PAD + 1.0 {
                saw_collapsed = true;
                assert!(
                    page_resize_edge_hit(true, left, width, left, tol).is_some()
                        && page_resize_edge_hit(true, left, width, right, tol).is_some(),
                    "{cell}: COLLAPSED column must keep both edges grabbable (the lockout fix)",
                );
            }
        }
    }
    assert!(
        saw_collapsed,
        "grid must include collapsed cells or it can't prove the lockout fix",
    );
}

#[test]
fn in_writing_column_is_true_inside_and_on_both_edges_false_outside() {
    let measure_px = 40.0 * CW; // 576
    let left = (1200.0 - measure_px) * 0.5; // 312
    let right = left + measure_px; // 888
    assert!(
        in_writing_column(left, left, measure_px),
        "exactly on the left edge counts as inside"
    );
    assert!(
        in_writing_column(right, left, measure_px),
        "exactly on the right edge counts as inside"
    );
    assert!(
        in_writing_column(600.0, left, measure_px),
        "dead center is inside"
    );
    assert!(
        !in_writing_column(left - 1.0, left, measure_px),
        "just past the left margin is outside"
    );
    assert!(
        !in_writing_column(right + 1.0, left, measure_px),
        "just past the right margin is outside"
    );
}

#[test]
fn image_handle_hit_arms_the_right_zone_per_edge_and_corner() {
    let rect = [100.0_f32, 50.0, 300.0, 200.0];
    let tol = IMAGE_RESIZE_GRAB_PX;
    assert_eq!(
        image_handle_hit((100.0, 50.0), rect, tol),
        Some(ImageHandle::TopLeft)
    );
    assert_eq!(
        image_handle_hit((400.0, 50.0), rect, tol),
        Some(ImageHandle::TopRight)
    );
    assert_eq!(
        image_handle_hit((100.0, 250.0), rect, tol),
        Some(ImageHandle::BottomLeft)
    );
    assert_eq!(
        image_handle_hit((400.0, 250.0), rect, tol),
        Some(ImageHandle::BottomRight)
    );
    assert_eq!(
        image_handle_hit((100.0, 150.0), rect, tol),
        Some(ImageHandle::Left)
    );
    assert_eq!(
        image_handle_hit((400.0, 150.0), rect, tol),
        Some(ImageHandle::Right)
    );
    assert_eq!(
        image_handle_hit((250.0, 50.0), rect, tol),
        Some(ImageHandle::Top)
    );
    assert_eq!(
        image_handle_hit((250.0, 250.0), rect, tol),
        Some(ImageHandle::Bottom)
    );
    assert_eq!(
        image_handle_hit((400.0 - tol + 1.0, 250.0 - tol + 1.0), rect, tol),
        Some(ImageHandle::BottomRight)
    );
    assert_eq!(image_handle_hit((250.0, 150.0), rect, tol), None, "center");
    assert_eq!(
        image_handle_hit((100.0, 50.0 - tol - 5.0), rect, tol),
        None,
        "above the top-left, off both"
    );
    assert_eq!(
        image_handle_hit((1000.0, 1000.0), rect, tol),
        None,
        "far outside"
    );
}

#[test]
fn image_resize_width_drives_per_handle_clamped_to_min_and_wrap() {
    let rect = [100.0_f32, 50.0, 300.0, 200.0];
    let (wrap, min) = (500.0_f32, MIN_IMAGE_W);
    let w = |h: ImageHandle, p: (f32, f32)| image_resize_width(h, rect, p, wrap, min, 0.0);
    assert!((w(ImageHandle::Right, (350.0, 150.0)) - 250.0).abs() < 1e-3);
    assert!((w(ImageHandle::Left, (200.0, 150.0)) - 200.0).abs() < 1e-3);
    assert!((w(ImageHandle::Bottom, (250.0, 150.0)) - 150.0).abs() < 1e-3);
    assert!((w(ImageHandle::Top, (250.0, 150.0)) - 150.0).abs() < 1e-3);
    assert!((w(ImageHandle::BottomRight, (100.0 + 150.0, 50.0 + 100.0)) - 150.0).abs() < 1e-3);
    assert!((w(ImageHandle::BottomRight, (400.0, 250.0)) - 300.0).abs() < 1e-3);
    assert!((w(ImageHandle::TopLeft, (100.0, 50.0)) - 300.0).abs() < 1e-3);
    assert!((w(ImageHandle::TopRight, (400.0, 50.0)) - 300.0).abs() < 1e-3);
    assert!((w(ImageHandle::BottomLeft, (100.0, 250.0)) - 300.0).abs() < 1e-3);
    assert!(
        w(ImageHandle::TopLeft, (60.0, 20.0)) > 300.0,
        "TopLeft out widens"
    );
    assert!(
        w(ImageHandle::TopLeft, (250.0, 150.0)) < 300.0,
        "TopLeft toward center narrows"
    );
    // Clamps: dragging way out clamps to wrap; way in clamps up to the floor.
    assert!((w(ImageHandle::Right, (5000.0, 150.0)) - wrap).abs() < 1e-3);
    assert!((w(ImageHandle::Right, (100.0, 150.0)) - min).abs() < 1e-3);
    // A degenerate wrap below the floor never inverts the clamp band.
    assert!(
        (image_resize_width(ImageHandle::Right, rect, (350.0, 150.0), 10.0, min, 0.0) - min).abs()
            < 1e-3
    );
}

/// The viewport-height half of the clamp: a drag can never grow an image
/// taller than `max_h`, even when the wrap width would otherwise allow it.
#[test]
fn image_resize_width_caps_at_the_viewport_height_ceiling() {
    let rect = [100.0_f32, 50.0, 300.0, 200.0];
    let (wrap, min) = (800.0_f32, MIN_IMAGE_W);
    let max_h = 150.0_f32;
    let w = image_resize_width(ImageHandle::Right, rect, (5000.0, 150.0), wrap, min, max_h);
    assert!((w - 225.0).abs() < 1e-3, "capped to height ceiling: {w}");
    // A max_h of 0 (unknown window height) disables the height half entirely —
    // dragging way out clamps to `wrap` instead.
    let w2 = image_resize_width(ImageHandle::Right, rect, (5000.0, 150.0), wrap, min, 0.0);
    assert!((w2 - wrap).abs() < 1e-3, "max_h<=0 disables the cap: {w2}");
    let w3 = image_resize_width(ImageHandle::Right, rect, (100.0, 150.0), wrap, min, max_h);
    assert!(
        (w3 - min).abs() < 1e-3,
        "floor still wins under a tight height cap: {w3}"
    );
}

#[test]
fn page_drag_measure_is_monotonic_across_the_rail_hide_boundary() {
    let window = 1800.0;
    let pref = outline_pref_px();
    let min = outline_min_px();
    let gap = margin_gap();

    let rendered_right = |m: usize| {
        adaptive_column_left(window, CW, true, m, true, pref, min, gap, ADAPTIVE_LEFT_PAD)
            + column_width_for(window, CW, true, m)
    };
    let cliffs = (crate::page::MIN_MEASURE + 1..=crate::page::MAX_MEASURE)
        .any(|m| rendered_right(m) < rendered_right(m - 1));
    assert!(
        cliffs,
        "fixture must span the rail-hide cliff or it can't reproduce the bug"
    );

    let start = 100usize;
    let anchor = adaptive_column_left(
        window,
        CW,
        true,
        start,
        true,
        pref,
        min,
        gap,
        ADAPTIVE_LEFT_PAD,
    );

    let mut prev = page_resize_measure_anchored(CW, 1700.0, anchor, ResizeEdge::Right);
    let first = prev;
    for px in 1700..=1799 {
        let m = page_resize_measure_anchored(CW, px as f32, anchor, ResizeEdge::Right);
        assert!(
            m >= prev,
            "rightward drag must never shrink the measure: at pointer {px} got {m} after {prev}",
        );
        prev = m;
    }
    assert!(
        prev > first,
        "the sweep must climb, not sit pinned (got {first}..{prev})"
    );
    let right_anchor = 2000.0;
    let mut lprev = page_resize_measure_anchored(CW, 1900.0, right_anchor, ResizeEdge::Left);
    for px in (1400..=1900).rev() {
        let m = page_resize_measure_anchored(CW, px as f32, right_anchor, ResizeEdge::Left);
        assert!(
            m >= lprev,
            "leftward drag of the left edge must never shrink the measure"
        );
        lprev = m;
    }
}

#[test]
fn page_drag_maps_one_advance_to_one_measure_not_two() {
    let start = 40usize;
    let left_anchor = 100.0;
    let at_press = left_anchor + start as f32 * CW; // the rendered right edge for `start`
    assert_eq!(
        page_resize_measure_anchored(CW, at_press, left_anchor, ResizeEdge::Right),
        start,
        "pressing the rendered edge must not snap the measure",
    );
    assert_eq!(
        page_resize_measure_anchored(CW, at_press + CW, left_anchor, ResizeEdge::Right),
        start + 1,
        "one advance of pointer travel is exactly one char",
    );
    let right_anchor = 2000.0;
    let left_press = right_anchor - start as f32 * CW;
    assert_eq!(
        page_resize_measure_anchored(CW, left_press, right_anchor, ResizeEdge::Left),
        start,
    );
    assert_eq!(
        page_resize_measure_anchored(CW, left_press - CW, right_anchor, ResizeEdge::Left),
        start + 1,
        "the left edge tracks 1:1 too (widen by dragging further from the anchor)",
    );
}

#[test]
fn page_drag_is_symmetric_and_zoom_independent() {
    for &zoom in &[0.5_f32, 1.0, 2.0] {
        let adv = page_column_advance(CW * zoom, zoom); // == CW at dpi 1.0
        let left_anchor = 100.0;
        let right_anchor = 2000.0;
        let dist = 40.0 * CW; // 40 chars of travel from the anchor
        let m_right =
            page_resize_measure_anchored(adv, left_anchor + dist, left_anchor, ResizeEdge::Right);
        let m_left =
            page_resize_measure_anchored(adv, right_anchor - dist, right_anchor, ResizeEdge::Left);
        assert_eq!(m_right, 40, "zoom={zoom}: 40 chars of travel -> 40 chars");
        assert_eq!(
            m_left, m_right,
            "zoom={zoom}: left/right mirror to the same measure"
        );
        let wider = page_resize_measure_anchored(
            adv,
            left_anchor + dist + 200.0,
            left_anchor,
            ResizeEdge::Right,
        );
        let narrower = page_resize_measure_anchored(
            adv,
            left_anchor + dist - 200.0,
            left_anchor,
            ResizeEdge::Right,
        );
        assert!(
            wider > m_right && narrower < m_right,
            "zoom={zoom}: out widens, in narrows"
        );
    }
}

#[test]
fn page_drag_clamps_to_the_settable_band() {
    let anchor = 100.0;
    assert_eq!(
        page_resize_measure_anchored(CW, 100_000.0, anchor, ResizeEdge::Right),
        crate::page::MAX_MEASURE,
    );
    assert_eq!(
        page_resize_measure_anchored(CW, anchor, anchor, ResizeEdge::Right),
        crate::page::MIN_MEASURE,
    );
    assert_eq!(
        page_resize_measure_anchored(CW, anchor - 500.0, anchor, ResizeEdge::Right),
        crate::page::MIN_MEASURE,
    );
    assert_eq!(
        page_resize_measure_anchored(0.0, 100_000.0, anchor, ResizeEdge::Right),
        crate::page::MIN_MEASURE,
    );
}

#[test]
fn narrow_window_still_collapses_edge_to_edge_at_any_zoom() {
    let window = 360.0; // 40-char measure ~576px >> window -> collapse
    for &zoom in &[0.5_f32, 1.0, 1.6, 3.0] {
        let adv = page_column_advance(CW * zoom, zoom);
        let w = column_width_for(window, adv, true, 40);
        let left = column_left_for(window, adv, true, 40);
        assert!(
            (w - (window - 2.0 * PAGE_MIN_PAD)).abs() < 1e-3,
            "zoom={zoom}: fills minus pad"
        );
        assert!(
            (left - PAGE_MIN_PAD).abs() < 1e-3,
            "zoom={zoom}: collapses to the small pad"
        );
    }
}
