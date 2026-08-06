//! WIRING THE DIAGONAL MARKER'S TURN.
//!
//! The shared chevron owner and the static shape already exist
//! (`crate::selection::chevron_arms`); this routes `chrome::diagonal::
//! selected_chevron` through that owner at a PINNED-VERTEX parameterization,
//! adds the travel-direction source (`chrome::MarkerTravel`), and the
//! `step_diagonal_marker` OR-fold member.
//!
//! Three claims, each graded here: (1) turning the mark never lets its vertex
//! drift off the spine, whatever the turn; (2) the direction source reads a
//! WRAP as continuing whichever way was already travelling, not as reversing
//! because the raw index fell or rose sharply; (3) arriving via a Down move
//! and arriving via an Up move settle the marker at genuinely different,
//! mirrored angles — real pixels, both diagonal worlds — so Reduce Motion's
//! instant settle still carries the cue: a cue that exists only while an
//! animation plays is not a cue.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::render::chrome::MarkerTravel;
use crate::render::chrome::diagonal::{MARKER_TRAVEL_TILT_DEG, MARKER_TURN_MS, selected_chevron};

const WORLDS: [&str; 2] = ["Mangrove", "Magpie"];

/// A `spine_segment` triple's own two endpoints, recovered from `(center,
/// half, axis)` — `half[0]` is the segment's own half-LENGTH, so `center ∓
/// axis*half[0]` are exactly the two points it was built from. Mirrors the
/// identically-named helper in `marker_chevron_owner_item247.rs` and
/// `fold_chevron_direction_item248.rs`; kept local rather than shared per this
/// tree's own convention (each test module re-derives its own tiny probe
/// rather than importing one across files).
fn ends(seg: ([f32; 2], [f32; 2], [f32; 2])) -> ([f32; 2], [f32; 2]) {
    let (center, half, axis) = seg;
    let (dx, dy) = (axis[0] * half[0], axis[1] * half[0]);
    (
        [center[0] - dx, center[1] - dy],
        [center[0] + dx, center[1] + dy],
    )
}

/// CLAIM 1 — THE VERTEX IS PINNED. `selected_chevron`'s vertex sits on the
/// spine at `(spine_x, (top+bottom)/2)` REGARDLESS of `turn_deg`: the marker
/// is anchored to a line it may not leave, so turning it must rotate the arms
/// about that fixed point rather than sliding the whole mark along the spine.
///
/// Swept across row origins, heights, spine abscissae, both reach signs (the
/// two worlds' mirrored clusters) and a turn range spanning a half-plus turn —
/// not just the two settled tilts — because the pin must hold at every angle a
/// live glide passes through, not only at the two angles that ship.
///
/// Non-vacuity: the SAME `(reach, spread, turn)` fed to the raw
/// `chevron_arms` primitive WITHOUT the pinned-centre derivation — i.e. a
/// centre fixed at the turn-0 midpoint — is also checked, and at every
/// nonzero turn its vertex is shown to have moved OFF the spine. That is the
/// exact defect the pinning derivation exists to prevent, so a law that
/// cannot see it prove anything.
#[test]
fn selected_chevron_pins_its_vertex_to_the_spine_at_every_turn() {
    const EPS: f32 = 1e-3;
    let mut cases = 0;
    let mut naive_drift_seen = false;
    for top in [0.0_f32, 137.5, 1024.0] {
        for height in [12.0_f32, 27.5, 88.0] {
            for spine_x in [0.0_f32, 64.0, 933.25] {
                // BOTH signs: a Descending world reaches right, an Ascending
                // one left.
                for reach in [-40.0_f32, -10.0, 10.0, 40.0] {
                    for turn_deg in [-179.0_f32, -90.0, -20.0, 0.0, 20.0, 90.0, 179.0] {
                        let (t, b) = (top + 2.0, top + height - 2.0);
                        let arm_x = spine_x + reach;
                        let want = [spine_x, (t + b) * 0.5];
                        let ctx = format!(
                            "top {top} height {height} spine_x {spine_x} reach {reach} \
                             turn {turn_deg}"
                        );

                        let pinned = selected_chevron(spine_x, arm_x, t, b, 3.0, turn_deg);
                        for (name, seg) in [("upper", pinned[0]), ("lower", pinned[1])] {
                            let vertex = ends(seg).0;
                            assert!(
                                (vertex[0] - want[0]).abs() < EPS
                                    && (vertex[1] - want[1]).abs() < EPS,
                                "{ctx}: the {name} arm's vertex must stay pinned at \
                                 {want:?}, got {vertex:?} — the marker left the spine"
                            );
                        }

                        // NON-VACUITY: the naive (unpinned) centre, held fixed at the
                        // turn-0 midpoint, drifts the vertex off the spine for any
                        // nonzero turn — proving the pin is load-bearing, not a
                        // no-op restatement of the same arithmetic.
                        let naive_reach = (spine_x - arm_x) * 0.5;
                        let naive_spread = (t - b) * 0.5;
                        let naive_center = [(spine_x + arm_x) * 0.5, (t + b) * 0.5];
                        let naive = crate::selection::chevron_arms(
                            naive_center,
                            naive_reach,
                            naive_spread,
                            turn_deg,
                            3.0,
                        );
                        let naive_vertex = ends(naive[0]).0;
                        let drift = ((naive_vertex[0] - want[0]).powi(2)
                            + (naive_vertex[1] - want[1]).powi(2))
                        .sqrt();
                        if turn_deg != 0.0 && reach.abs() > EPS {
                            assert!(
                                drift > EPS,
                                "{ctx}: the UNPINNED centre must drift the vertex off the \
                                 spine at a nonzero turn (drift {drift:.6}px) — if it does \
                                 not, this law cannot tell a pinned mark from a wandering one"
                            );
                            naive_drift_seen = true;
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        cases,
        3 * 3 * 3 * 4 * 7,
        "the sweep must not silently shrink"
    );
    assert!(
        naive_drift_seen,
        "the non-vacuity arm never ran — every case had a zero turn or a zero reach"
    );
}

/// CLAIM 2 — THE WRAP-AWARE DIRECTION SOURCE.
///
/// `MarkerTravel::of` is pure arithmetic (no device, no clock), so its whole
/// domain is swept directly: every ordinary one-step move in both directions
/// across several list sizes, the two degenerate no-travel cases, and the
/// headline claim — a wrap reads as the direction that CONTINUES, not the one
/// the raw index delta alone implies. `total == 2` is deliberately excluded
/// from the generic one-step sweep: with two rows the two directions are the
/// same physical move (`forward == backward == 1`), so the tie-break
/// (`forward <= backward` picks `Down`) is the CORRECT, defined answer rather
/// than a case this law can grade as "the other direction".
#[test]
fn marker_travel_of_reads_the_wrap_aware_direction_and_a_no_op_as_none() {
    assert_eq!(
        MarkerTravel::of(2, 2, 6),
        None,
        "a no-op move (prev == next) must report no travel"
    );
    assert_eq!(
        MarkerTravel::of(0, 0, 0),
        None,
        "a degenerate empty list must report no travel"
    );

    let mut cases = 0;
    for total in [3usize, 4, 6, 24] {
        for prev in 0..total {
            let next_down = (prev + 1) % total;
            assert_eq!(
                MarkerTravel::of(prev, next_down, total),
                Some(MarkerTravel::Down),
                "total {total}: {prev} -> {next_down} (one step forward) must read Down"
            );
            let next_up = (prev + total - 1) % total;
            assert_eq!(
                MarkerTravel::of(prev, next_up, total),
                Some(MarkerTravel::Up),
                "total {total}: {prev} -> {next_up} (one step back) must read Up"
            );
            cases += 2;
        }
    }
    assert_eq!(
        cases,
        (3 + 4 + 6 + 24) * 2,
        "the sweep must not silently shrink"
    );

    // THE WRAP — a wrap continues whichever direction was already travelling.
    assert_eq!(
        MarkerTravel::of(5, 0, 6),
        Some(MarkerTravel::Down),
        "last -> first (a Down wrap) must still read Down, not Up — CLAUDE.md's own \
         brief: \"a wrap … takes the long way round\""
    );
    assert_eq!(
        MarkerTravel::of(0, 5, 6),
        Some(MarkerTravel::Up),
        "first -> last (an Up wrap) must still read Up, not Down"
    );

    // NON-VACUITY: the tie total==2 really does resolve to the documented
    // Down default rather than something this law would misreport as a pass.
    assert_eq!(MarkerTravel::of(0, 1, 2), Some(MarkerTravel::Down));
    assert_eq!(MarkerTravel::of(1, 0, 2), Some(MarkerTravel::Down));
}

/// A picker whose rows are all on screen at once (`overlay_window_rows` covers
/// every item), so a display index never means anything but the item's own
/// position — no scroll, no elision, no wrap ambiguity to control for.
fn nav_view(items: usize, selected: usize) -> ViewState {
    let mut v = view("doc\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "go to";
    v.overlay_items = (0..items).map(|i| format!("item {i}")).collect();
    v.overlay_selected = selected;
    v.overlay_window_rows = items;
    v
}

/// CLAIM 3, PART A — DOWN AND UP SETTLE AT MIRRORED, DISTINCT ANGLES, and the
/// two worlds mirror EACH OTHER on the same travel — verifying that `arm_x`
/// (`cluster.label_anchor`) is one expression, no per-world match, so this
/// code need not (and does not) branch on world identity anywhere of its own:
/// [`MarkerTravel::sign`] × `MARKER_TRAVEL_TILT_DEG` × `DiagonalDirection::sign`
/// is the WHOLE formula
/// (`resolve_diagonal_marker_travel`), and this law asserts the two worlds'
/// outputs are exact negatives of each other for the identical travel.
///
/// A fresh pipeline settles turn 0.0 with NO glide (`!juice_live`), so this is
/// exactly what a headless capture renders — no clock, no animation, matching
/// CLAUDE.md's determinism law.
#[test]
fn diagonal_marker_settles_at_mirrored_tilts_for_down_and_up_travel() {
    let _g = crate::testlock::serial();
    let Some((device, queue, _)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping diagonal_marker_settles_at_mirrored_tilts: no wgpu adapter");
        return;
    };

    // Indexed to WORLDS — avoids matching the world's NAME (a `&str`, whose
    // match would need a wildcard arm to be exhaustive) to route a value.
    let mut down_by_world = [0.0_f32; WORLDS.len()];

    for (world_ix, world) in WORLDS.into_iter().enumerate() {
        // FRESH pipelines per world: `diagonal_marker_row` is per-pipeline
        // travel memory, and a shared pipeline carried across worlds would
        // still remember the PREVIOUS world's last-selected row, making a
        // "fresh open" no longer fresh — the exact bug this exists to avoid
        // for a real user re-opening an overlay.
        let (mut p_down, mut p_up) = (
            headless_dqp(1200.0, 800.0).unwrap().2,
            headless_dqp(1200.0, 800.0).unwrap().2,
        );
        let _pin = theme::WorldPin::world(world).expect("both diagonal worlds ship");
        p_down.sync_theme();
        p_up.sync_theme();

        // p_down: settle fresh on row 2 (turn 0.0, no history) with a REAL
        // `prepare` (establishes shaping/metrics), then arrive at row 3 via a
        // single Down step — read through ONE direct `resolve_visual_selection`
        // call, never a second, since a second call this same frame would see
        // its OWN first call's write and report no travel (`prev == next`).
        p_down.set_view(&nav_view(6, 2));
        p_down.prepare(&device, &queue, 1200, 800).unwrap();
        assert_eq!(
            p_down.diagonal_marker_turn_deg(),
            0.0,
            "{world}: a freshly opened card must settle un-turned"
        );
        p_down.set_view(&nav_view(6, 3));
        let down_geom = p_down.overlay_geometry(1200);
        let down_plan = p_down.overlay_row_plan(&down_geom);
        assert_eq!(
            p_down
                .resolve_visual_selection(&down_geom, &down_plan)
                .travel(),
            Some(MarkerTravel::Down),
            "{world}: the transaction's own travel source must report Down for 2 -> 3"
        );
        let turn_down = p_down.diagonal_marker_turn_deg();

        // p_up: settle fresh on row 4, then arrive at the SAME row 3 via a
        // single Up step.
        p_up.set_view(&nav_view(6, 4));
        p_up.prepare(&device, &queue, 1200, 800).unwrap();
        p_up.set_view(&nav_view(6, 3));
        let up_geom = p_up.overlay_geometry(1200);
        let up_plan = p_up.overlay_row_plan(&up_geom);
        assert_eq!(
            p_up.resolve_visual_selection(&up_geom, &up_plan).travel(),
            Some(MarkerTravel::Up),
            "{world}: the transaction's own travel source must report Up for 4 -> 3"
        );
        let turn_up = p_up.diagonal_marker_turn_deg();

        assert!(
            turn_down.abs() > 1.0,
            "{world}: a Down arrival must settle at a real nonzero tilt, got {turn_down}"
        );
        assert!(
            (turn_down + turn_up).abs() < 1e-4,
            "{world}: Down ({turn_down}) and Up ({turn_up}) must settle at MIRRORED \
             angles (sum must be ~0), or Reduce Motion's instant settle cannot tell \
             the two directions apart"
        );
        assert!(
            (turn_down.abs() - MARKER_TRAVEL_TILT_DEG).abs() < 1e-4,
            "{world}: the settled tilt's magnitude must be the authored \
             MARKER_TRAVEL_TILT_DEG ({MARKER_TRAVEL_TILT_DEG}), got {}",
            turn_down.abs()
        );

        down_by_world[world_ix] = turn_down;
    }

    let (mangrove_down, magpie_down) = (down_by_world[0], down_by_world[1]);
    assert!(
        (mangrove_down + magpie_down).abs() < 1e-4,
        "the SAME travel (Down) must settle at OPPOSITE angles on the two mirrored \
         worlds — Mangrove {mangrove_down}, Magpie {magpie_down} — proving the turn \
         is mirrored from DiagonalDirection::sign(), the same dial the cluster \
         itself already mirrors on"
    );

    theme::set_active(theme::DEFAULT_THEME);
}

/// CLAIM 3, PART B — REAL PIXELS: the Down-settled and Up-settled marks for
/// the SAME selected row are genuinely different ink, on both diagonal
/// worlds. Isolates the marker's own local box (read off
/// `diagonal_cluster_probe` + the row plan — the SAME geometry the draw used,
/// never a re-derivation) and counts pixels that differ between the two
/// captures inside it.
#[test]
fn diagonal_marker_ink_differs_between_arriving_from_above_and_below() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p_down)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping diagonal_marker_ink_differs: no wgpu adapter");
        return;
    };
    let Some((_, _, mut p_up)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping diagonal_marker_ink_differs: no wgpu adapter");
        return;
    };

    for world in WORLDS {
        let _pin = theme::WorldPin::world(world).expect("both diagonal worlds ship");
        p_down.sync_theme();
        p_up.sync_theme();

        p_down.set_view(&nav_view(6, 2));
        p_down.prepare(&device, &queue, 1200, 800).unwrap();
        p_down.set_view(&nav_view(6, 3));
        p_down.prepare(&device, &queue, 1200, 800).unwrap();
        let frame_down = pixeldiff::render_frame(&mut p_down, &device, &queue, 1200, 800);

        p_up.set_view(&nav_view(6, 4));
        p_up.prepare(&device, &queue, 1200, 800).unwrap();
        p_up.set_view(&nav_view(6, 3));
        p_up.prepare(&device, &queue, 1200, 800).unwrap();
        let frame_up = pixeldiff::render_frame(&mut p_up, &device, &queue, 1200, 800);

        // The marker's own local box for display row 3: the cluster's SPINE
        // end (the vertex's home column) out to its arm end, spanning the
        // row's own inset top/bottom — the exact territory `selected_chevron`
        // draws into, read off the same probes the draw used.
        let geom = p_down.overlay_geometry(1200);
        let plan = p_down.overlay_row_plan(&geom);
        let row = plan.rows()[3];
        let probe = p_down
            .diagonal_cluster_probe()
            .unwrap_or_else(|| panic!("{world}: a diagonal world measures a cluster"));
        let spine_x = probe.spine_x(3);
        let arm_x = probe.label_anchor(3);
        let pad = 6.0_f32;
        let x0 = (spine_x.min(arm_x) - pad).max(0.0) as i64;
        let x1 = (spine_x.max(arm_x) + pad).min(1200.0) as i64;
        let y0 = row.top.floor().max(0.0) as i64;
        let y1 = row.bottom().ceil().min(800.0) as i64;

        let mut differ = 0usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * 1200 + x) as usize;
                if frame_down[i] != frame_up[i] {
                    differ += 1;
                }
            }
        }
        assert!(
            differ > 20,
            "{world}: the Down-settled and Up-settled marks for the SAME selected \
             row must paint genuinely different ink in the marker's own box \
             (x {x0}..{x1}, y {y0}..{y1}) — got {differ} differing px. A cue that \
             reads the same at rest regardless of travel direction is no cue at \
             all."
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
}

/// THE STEPPING MECHANISM, on INJECTED dt — the deterministic half of the
/// glide, mirroring `fold_chevron_direction_item248`'s own sibling law
/// exactly (`fold_chevron_turn_progresses_on_injected_dt_and_settles_exactly`).
/// `advance(dt)` takes an injected delta, not a real clock, so stepping it is
/// exactly as deterministic as any other pure function: what the harness
/// genuinely cannot reach is the real-time GLIDE's FEEL, flagged for human
/// confirmation, not the stepping arithmetic itself.
#[test]
fn diagonal_marker_turn_progresses_on_injected_dt_and_settles_exactly() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping diagonal_marker_turn_progresses: no wgpu adapter");
        return;
    };
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    p.arm_live_juice();
    let _pin = theme::WorldPin::world("Mangrove").expect("Mangrove ships");
    p.sync_theme();

    // Settle fresh on row 2 (turn 0.0), then arrive at row 3 via a Down step —
    // this RETARGETS `diagonal_marker_target` but the animator has not yet
    // been stepped, so `diagonal_marker_turn` (the eased value) starts at 0.0
    // while the target sits at the settled tilt.
    p.set_view(&nav_view(6, 2));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    p.set_view(&nav_view(6, 3));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let target = MARKER_TRAVEL_TILT_DEG; // Mangrove Descending, Down: +sign*+sign.

    let steps_ms: [u32; 4] = [16, 30, 60, 400];
    let mut prev_ms = 0u32;
    let mut turns = Vec::new();
    for &t in &steps_ms {
        let dt = (t.saturating_sub(prev_ms)) as f32 / 1000.0;
        prev_ms = t;
        p.advance(dt);
        turns.push(p.diagonal_marker_turn_deg());
    }

    for w in turns.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-6,
            "the turn must progress MONOTONICALLY toward the target: {turns:?}"
        );
    }
    assert!(
        turns[0] > 0.0 && turns[0] < target,
        "an early step must be MID-GLIDE, neither the old nor the new settled \
         value: {:?} (target {target})",
        turns[0]
    );
    let last = *turns.last().unwrap();
    assert!(
        (last - target).abs() < 1e-3,
        "late enough into the glide the mark must SETTLE exactly at {target}: {last}"
    );
    // The glide covers at most one full tilt swing (`2 * MARKER_TRAVEL_TILT_DEG`)
    // in `MARKER_TURN_MS`; four steps summing past that duration is enough to
    // guarantee settlement, so a law that fails to settle here is a real bug,
    // not an under-run.
    assert!(
        (steps_ms[3] as f32) > MARKER_TURN_MS,
        "fixture self-check: the sweep must run longer than one full turn"
    );

    theme::set_active(theme::DEFAULT_THEME);
    crate::motion::set_reduced(saved);
}

/// ACCESSIBILITY TIER 1 — REDUCE MOTION settles the marker's turn INSTANTLY:
/// same final angle as the unarmed/headless path, zero glide frames. Mirrors
/// `step_copy_pulse`'s and `step_fold_chevrons`'s own gate exactly.
#[test]
fn diagonal_marker_turn_settles_instantly_under_reduce_motion() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping diagonal_marker_turn_settles_instantly: no wgpu adapter");
        return;
    };
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(true);
    p.arm_live_juice();
    let _pin = theme::WorldPin::world("Mangrove").expect("Mangrove ships");
    p.sync_theme();

    p.set_view(&nav_view(6, 2));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    p.set_view(&nav_view(6, 3));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let before_advance = p.diagonal_marker_turn_deg();
    p.advance(0.001);
    let after_advance = p.diagonal_marker_turn_deg();

    assert!(
        (before_advance - MARKER_TRAVEL_TILT_DEG).abs() < 1e-4,
        "Reduce Motion must settle the marker's turn to the target on the VERY \
         FIRST read, with no glide frame in between: got {before_advance}"
    );
    assert!(
        (after_advance - before_advance).abs() < 1e-6,
        "advancing the clock under Reduce Motion must not move the settled turn: \
         {before_advance} -> {after_advance}"
    );

    theme::set_active(theme::DEFAULT_THEME);
    crate::motion::set_reduced(saved);
}
