use super::headless_pipeline;
use crate::frame_clock::{Activity, FrameSample};
use crate::render::{COPY_PULSE_MS, copy_pulse_ease};
use std::time::Duration;

fn equal_steps(total: Duration, count: u32) -> Vec<Duration> {
    let nanos = total.as_nanos() as u64;
    let each = nanos / u64::from(count);
    let mut steps = vec![Duration::from_nanos(each); count as usize];
    *steps.last_mut().expect("positive step count") +=
        Duration::from_nanos(nanos - each * u64::from(count));
    steps
}

fn pure_pulse_pose_after(steps: &[Duration]) -> (f32, bool) {
    let mut progress = 0.0f32;
    for elapsed in steps {
        progress = (progress + COPY_PULSE_MS.progress_per(elapsed.as_secs_f32())).min(1.0);
    }
    (copy_pulse_ease(progress), progress < 1.0)
}

#[test]
fn one_wall_time_has_one_pose_at_60hz_120hz_coarse_and_dropped_cadences() {
    let total = Duration::from_millis(110);
    let schedules = [
        ("60hz", equal_steps(total, 6)),
        ("120hz", equal_steps(total, 13)),
        ("coarse", equal_steps(total, 2)),
        (
            "dropped",
            vec![
                Duration::from_millis(8),
                Duration::from_millis(17),
                Duration::from_millis(61),
                Duration::from_millis(24),
            ],
        ),
    ];
    let expected = pure_pulse_pose_after(&schedules[0].1);
    assert!(expected.1, "110ms is deliberately a mid-flight pose");
    for (name, steps) in &schedules[1..] {
        let got = pure_pulse_pose_after(steps);
        assert!(
            (got.0 - expected.0).abs() < 1e-5,
            "{name}: equal wall time must mean equal pose: expected={expected:?} got={got:?}"
        );
        assert_eq!(got.1, expected.1, "{name}: activity state diverged");
    }
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
            .advance_frame(FrameSample { now, elapsed })
            .contains(Activity::CopyPulse);
        retirements += usize::from(was_active && !active);
        was_active = active;
    }
    assert_eq!(retirements, 1, "a bounded animator retires exactly once");
    assert!(!was_active);
    assert_eq!(pipeline.copy_pulse_settle(), 1.0);
    crate::motion::set_reduced(saved);
}
