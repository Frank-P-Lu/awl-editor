use super::headless_pipeline;
use crate::frame_clock::{Activity, FrameSample};
use std::time::Duration;

fn equal_steps(total: Duration, count: u32) -> Vec<Duration> {
    let nanos = total.as_nanos() as u64;
    let each = nanos / u64::from(count);
    let mut steps = vec![Duration::from_nanos(each); count as usize];
    *steps.last_mut().expect("positive step count") +=
        Duration::from_nanos(nanos - each * u64::from(count));
    steps
}

#[test]
fn one_wall_time_drives_every_real_owner_at_60hz_120hz_coarse_and_dropped_cadences() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping production cadence table: no wgpu adapter");
        return;
    }
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    for activity in Activity::ALL {
        let total = match activity {
            Activity::CaretPreview => Duration::from_millis(600),
            Activity::CopyPulse => Duration::from_millis(110),
            Activity::OverlayEntrance => Duration::from_millis(100),
            Activity::CaretMotion
            | Activity::OverlayBand
            | Activity::FoldChevrons
            | Activity::TravellingGround => Duration::from_millis(60),
        };
        let sixty_steps = ((total.as_secs_f64() * 60.0).ceil() as u32).max(1);
        let one_twenty_steps = ((total.as_secs_f64() * 120.0).ceil() as u32).max(1);
        let first_drop = total / 8;
        let second_drop = total / 4;
        let schedules = [
            ("60hz", equal_steps(total, sixty_steps)),
            ("120hz", equal_steps(total, one_twenty_steps)),
            ("coarse", equal_steps(total, 2)),
            (
                "dropped",
                vec![first_drop, second_drop, total - first_drop - second_drop],
            ),
        ];
        let mut outcomes = Vec::new();
        for (name, steps) in &schedules {
            let mut pipeline = headless_pipeline().expect("adapter checked above");
            assert!(pipeline.arm_activity_law(activity), "{activity:?} fixture");
            let armed_pose = pipeline.activity_law_pose(activity);
            let mut now = crate::clock::Instant::now();
            let mut active = crate::frame_clock::ActivitySet::empty();
            for elapsed in steps {
                now += *elapsed;
                active = pipeline.advance_frame(
                    FrameSample::injected(now, *elapsed),
                    activity == Activity::TravellingGround,
                );
            }
            assert!(
                active.contains(activity),
                "{activity:?} retired during {name}"
            );
            let pose = pipeline.activity_law_pose(activity);
            let movement_floor = if activity == Activity::CaretMotion {
                0.01
            } else {
                1e-5
            };
            assert!(
                (pose - armed_pose).abs() > movement_floor,
                "{activity:?}/{name}: cadence law was vacuous; production pose never left \
                 {armed_pose}"
            );
            outcomes.push((*name, pose));
        }
        let expected = outcomes[0].1;
        for (name, got) in &outcomes[1..] {
            let tolerance = if activity == Activity::CaretMotion {
                2.0
            } else {
                1e-4
            };
            assert!(
                (got - expected).abs() <= tolerance,
                "{activity:?}/{name}: equal presented time diverged: expected={expected}, got={got}"
            );
        }
    }
    crate::motion::set_reduced(saved);
}

#[test]
fn newly_armed_bounded_owner_starts_at_zero_delta_after_a_long_idle() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping idle-to-animation table: no wgpu adapter");
        return;
    }
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    let wall_start = crate::clock::Instant::now();
    for activity in Activity::ALL {
        let bounded = match activity {
            Activity::CaretMotion
            | Activity::CaretPreview
            | Activity::CopyPulse
            | Activity::OverlayEntrance
            | Activity::OverlayBand
            | Activity::FoldChevrons => true,
            Activity::TravellingGround => false,
        };
        if !bounded {
            continue;
        }
        let mut clock = crate::frame_clock::FrameClock::default();
        let idle = clock.sample(wall_start);
        clock.presented(idle, crate::frame_clock::ActivitySet::empty());
        let mut pipeline = headless_pipeline().expect("adapter checked above");
        assert!(pipeline.arm_activity_law(activity), "{activity:?} fixture");
        let armed_pose = pipeline.activity_law_pose(activity);
        let first = clock.sample(wall_start + Duration::from_secs(30));
        assert_eq!(first.elapsed, Duration::ZERO, "{activity:?}");
        let active = pipeline.advance_frame(first, false);
        assert!(active.contains(activity), "{activity:?} jumped to settled");
        assert_eq!(
            pipeline.activity_law_pose(activity),
            armed_pose,
            "{activity:?} moved on its first visible frame after idle"
        );
    }
    crate::motion::set_reduced(saved);
}

#[test]
fn copy_pulse_reports_one_active_to_idle_edge_at_its_authored_duration() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping frame-clock retirement table: no wgpu adapter");
        return;
    }
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    let mut pipeline = headless_pipeline().expect("adapter checked above");
    pipeline.copy_pulse();
    let mut now = crate::clock::Instant::now();
    let mut was_active = true;
    let mut retirements = 0;
    for elapsed in equal_steps(Duration::from_millis(440), 53) {
        now += elapsed;
        let active = pipeline
            .advance_frame(FrameSample::injected(now, elapsed), false)
            .contains(Activity::CopyPulse);
        retirements += usize::from(was_active && !active);
        was_active = active;
    }
    assert_eq!(retirements, 1, "a bounded animator retires exactly once");
    assert!(!was_active);
    assert_eq!(pipeline.copy_pulse_settle(), 1.0);
    crate::motion::set_reduced(saved);
}

#[test]
fn every_real_owner_reports_its_own_active_to_idle_retirement() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping production retirement table: no wgpu adapter");
        return;
    }
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    for activity in Activity::ALL {
        let mut pipeline = headless_pipeline().expect("adapter checked above");
        assert!(pipeline.arm_activity_law(activity), "{activity:?} fixture");
        let travelling = activity == Activity::TravellingGround;
        assert!(
            pipeline.active_activities(travelling).contains(activity),
            "{activity:?} did not enroll at its production owner"
        );
        assert!(
            !pipeline.retire_activity_law(activity).contains(activity),
            "{activity:?} did not retire through its production owner"
        );
    }
    crate::motion::set_reduced(saved);
}
