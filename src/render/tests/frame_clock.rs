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
    let total = Duration::from_millis(60);
    let schedules = [
        ("60hz", equal_steps(total, 4)),
        ("120hz", equal_steps(total, 8)),
        ("coarse", equal_steps(total, 2)),
        (
            "dropped",
            vec![
                Duration::from_millis(8),
                Duration::from_millis(17),
                Duration::from_millis(35),
            ],
        ),
    ];
    for activity in Activity::ALL {
        let mut outcomes = Vec::new();
        for (name, steps) in &schedules {
            let mut pipeline = headless_pipeline().expect("adapter checked above");
            assert!(pipeline.arm_activity_law(activity), "{activity:?} fixture");
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
            outcomes.push((*name, pipeline.activity_law_pose(activity)));
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
