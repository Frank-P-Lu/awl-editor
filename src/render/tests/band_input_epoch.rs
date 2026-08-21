//! Input-anchored timing laws for the theme picker's selection band.
//!
//! The live preview does synchronous shaping before `prepare` can resolve the
//! selected row. These laws inject one virtual `now` at input and another at
//! prepare, proving that the work interval consumes the authored 110 ms rather
//! than starting a fresh 110 ms tail. Both shared band seams are swept: Pane's
//! living morph and the ordinary sliding-band override.

use super::super::*;
use super::headless_pipeline;
use crate::clock::Clock;
use crate::render::livingband::{self, Choreo, MotionForce};

const EPS: f32 = 0.001;
const LH: f32 = 30.0;

#[derive(Clone, Copy, Debug)]
enum Seam {
    Pane,
    Sliding,
}

impl Seam {
    const ALL: [Self; 2] = [Self::Pane, Self::Sliding];
}

fn morph_force() -> MotionForce {
    MotionForce {
        choreo: Choreo::Morph,
        phase: None,
    }
}

fn target(k: usize) -> f32 {
    100.0 + k as f32 * LH
}

fn phase_and_top(p: &mut TextPipeline, seam: Seam, target: f32) -> (f32, f32) {
    match seam {
        Seam::Pane => {
            let force = morph_force();
            let (from, to, phase) = p.living_band_phase(force, target, LH);
            let top = livingband::morph_band(from, to, LH, phase, &force.choreo.params()).top;
            (phase, top)
        }
        Seam::Sliding => {
            let top = p.overlay_band_drawn(target);
            (p.overlay_band_t, top)
        }
    }
}

fn arm_seam(p: &mut TextPipeline, seam: Seam) {
    p.arm_live_juice();
    set_motion_test_override(match seam {
        Seam::Pane => None,
        Seam::Sliding => Some(theme::MotionJuice {
            entrance: theme::OverlayEntrance::Instant,
            band: theme::BandResponse::Slide,
        }),
    });
}

fn at(clock: &crate::clock::VirtualClock, p: &mut TextPipeline, elapsed_ms: u64) {
    clock.advance_ms(elapsed_ms);
    p.begin_overlay_frame(clock.now());
}

#[test]
fn render_work_consumes_the_authored_band_budget_across_both_seams() {
    let _g = crate::testlock::serial();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(false);
    let mut phase_failures = Vec::new();

    for seam in Seam::ALL {
        for delay_ms in [0u64, 40, 80, 110, 150] {
            let mut p = headless_pipeline().expect("shared test adapter");
            arm_seam(&mut p, seam);
            let clock = crate::clock::VirtualClock::new();
            p.begin_overlay_frame(clock.now());
            let (_, fresh) = phase_and_top(&mut p, seam, target(0));
            assert!(
                (fresh - target(0)).abs() < EPS,
                "{seam:?}: first open settles"
            );

            p.stamp_overlay_movement(clock.now());
            at(&clock, &mut p, delay_ms);
            let (phase, top) = phase_and_top(&mut p, seam, target(1));
            let expected = (delay_ms as f32 / OVERLAY_BAND_SLIDE_MS.0).min(1.0);
            if (phase - expected).abs() >= EPS {
                phase_failures.push(format!(
                    "{seam:?}, delay={delay_ms}ms: expected {expected:.3}, got {phase:.3}"
                ));
                continue;
            }
            if delay_ms >= OVERLAY_BAND_SLIDE_MS.0 as u64 {
                assert!(
                    (top - target(1)).abs() < EPS,
                    "{seam:?}, delay={delay_ms}ms: work consumed the whole budget; \
                     the band is settled"
                );
                assert!(
                    !p.active_activities()
                        .contains(crate::frame_clock::Activity::OverlayBand),
                    "a phase already settled at first prepare owes no decorative follow-up frame"
                );
            }
        }
    }

    assert!(
        phase_failures.is_empty(),
        "prepare must inherit the input phase in every delay cell:\n{}",
        phase_failures.join("\n")
    );

    set_motion_test_override(None);
    crate::motion::set_reduced(saved_reduced);
}

#[test]
fn full_delay_by_input_cadence_sweep_never_targets_a_superseded_row() {
    let _g = crate::testlock::serial();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(false);

    for seam in Seam::ALL {
        for delay_ms in [0u64, 40, 80, 110, 150] {
            for cadence_ms in [60u64, 100, 150, 220] {
                let mut p = headless_pipeline().expect("shared test adapter");
                arm_seam(&mut p, seam);
                let clock = crate::clock::VirtualClock::new();
                p.begin_overlay_frame(clock.now());
                let _ = phase_and_top(&mut p, seam, target(0));

                // First move at t=0. If the next input outruns its render, it
                // supersedes that pending target before prepare ever sees it.
                p.stamp_overlay_movement(clock.now());
                if cadence_ms < delay_ms {
                    clock.advance_ms(cadence_ms);
                    p.stamp_overlay_movement(clock.now());
                    clock.advance_ms(delay_ms);
                    p.begin_overlay_frame(clock.now());
                    let (phase, top) = phase_and_top(&mut p, seam, target(2));
                    assert!(
                        (top - target(1)).abs() > LH * 0.5,
                        "{seam:?}, delay={delay_ms}, cadence={cadence_ms}: \
                         no frame may animate toward stale row 1"
                    );
                    if phase >= 1.0 || p.overlay_band_pending_snap {
                        assert!(
                            (top - target(2)).abs() < EPS,
                            "{seam:?}: latest-selection-wins must draw the superseding row"
                        );
                    }
                } else {
                    at(&clock, &mut p, delay_ms);
                    let _ = phase_and_top(&mut p, seam, target(1));
                    clock.advance_ms(cadence_ms - delay_ms);
                    p.stamp_overlay_movement(clock.now());
                    at(&clock, &mut p, delay_ms);
                    let _ = phase_and_top(&mut p, seam, target(2));
                    assert_eq!(
                        p.overlay_band_last,
                        Some(target(2)),
                        "{seam:?}, delay={delay_ms}, cadence={cadence_ms}: \
                         the animator may start at the old pose, but its destination is \
                         always the newest row"
                    );
                }

                // Regardless of when prepare occurred, 110 ms from the latest
                // input is a hard settlement ceiling, never prepare + 110 ms.
                if p.overlay_band_t < 1.0 {
                    let remain = ((1.0 - p.overlay_band_t) * OVERLAY_BAND_SLIDE_MS.0).ceil() as u64;
                    at(&clock, &mut p, remain);
                    let (_, top) = phase_and_top(&mut p, seam, target(2));
                    assert!(
                        (top - target(2)).abs() < EPS,
                        "{seam:?}: settled on newest row"
                    );
                }
            }
        }
    }

    set_motion_test_override(None);
    crate::motion::set_reduced(saved_reduced);
}

#[test]
fn first_open_world_crossings_and_reduce_motion_keep_their_existing_policy() {
    let _g = crate::testlock::serial();
    let saved_reduced = crate::motion::reduced();
    let clock = crate::clock::VirtualClock::new();
    let mut p = headless_pipeline().expect("shared test adapter");
    arm_seam(&mut p, Seam::Pane);
    crate::motion::set_reduced(false);

    // Even a defensive stamp before first geometry cannot invent a source row.
    p.stamp_overlay_movement(clock.now());
    p.begin_overlay_frame(clock.now());
    let (_, top) = phase_and_top(&mut p, Seam::Pane, target(0));
    assert!(
        (top - target(0)).abs() < EPS,
        "freshly opened overlays begin settled"
    );

    // Pane -> Bars uses the same epoch owner; the ordinary sliding override is
    // neither restarted at prepare nor given a stale Pane-only destination.
    p.stamp_overlay_movement(clock.now());
    at(&clock, &mut p, 80);
    arm_seam(&mut p, Seam::Sliding);
    let (phase, _) = phase_and_top(&mut p, Seam::Sliding, target(1));
    assert!(
        (phase - 80.0 / 110.0).abs() < EPS,
        "world crossing keeps input epoch"
    );
    assert!(
        p.active_activities()
            .contains(crate::frame_clock::Activity::OverlayBand),
        "the post-prepare activity report carries the crossing band"
    );

    // Bars -> Pane under Reduce Motion is still an immediate final pose and
    // does not claim a follow-up frame.
    crate::motion::set_reduced(true);
    p.stamp_overlay_movement(clock.now());
    at(&clock, &mut p, 0);
    arm_seam(&mut p, Seam::Pane);
    let (phase, top) = phase_and_top(&mut p, Seam::Pane, target(2));
    assert_eq!(phase, 1.0);
    assert!((top - target(2)).abs() < EPS);
    assert!(
        !p.active_activities()
            .contains(crate::frame_clock::Activity::OverlayBand)
    );

    set_motion_test_override(None);
    crate::motion::set_reduced(saved_reduced);
}
