use super::*;
use std::cell::{Cell, RefCell};

fn fault(kind: GpuFaultKind) -> GpuFault {
    GpuFault {
        kind,
        message: format!("test {kind:?}"),
    }
}

#[test]
fn healthy_skips_and_classified_validation_submit_once_unhealthy_faults_do_not() {
    let _guard = crate::testlock::serial();
    for skip in [
        GpuFrameSkip::Timeout,
        GpuFrameSkip::Occluded,
        GpuFrameSkip::PrepareFailed,
        GpuFrameSkip::SurfaceReconfigured,
        GpuFrameSkip::SurfaceRecreated,
    ] {
        let submits = Cell::new(0);
        let outcome = complete_uploads(GpuFrameOutcome::Skipped(skip), Vec::new, || {
            submits.set(submits.get() + 1)
        });
        assert!(matches!(outcome, GpuFrameOutcome::Skipped(s) if s == skip));
        assert_eq!(submits.get(), 1);
    }
    for kind in [
        GpuFaultKind::DeviceLost,
        GpuFaultKind::Internal,
        GpuFaultKind::SurfaceRecoveryFailed,
        GpuFaultKind::OutOfMemory,
        GpuFaultKind::Validation,
    ] {
        for queued in [false, true] {
            let submits = Cell::new(0);
            let mut inbox = Some(fault(kind));
            let outcome = if queued {
                GpuFrameOutcome::Skipped(GpuFrameSkip::PrepareFailed)
            } else {
                GpuFrameOutcome::Fault(inbox.take().unwrap())
            };
            let outcome = complete_uploads(
                outcome,
                || inbox.take().into_iter().collect(),
                || submits.set(submits.get() + 1),
            );
            assert!(matches!(outcome, GpuFrameOutcome::Fault(f) if f.kind == kind));
            assert_eq!(submits.get(), usize::from(kind == GpuFaultKind::Validation));
        }
    }
    complete_uploads(
        GpuFrameOutcome::Presented(None),
        || panic!("present has its own fault owner"),
        || panic!("present already submitted"),
    );
}

#[test]
fn submission_fault_priority_blocks_deferred_surface_actions() {
    let _guard = crate::testlock::serial();
    for kind in [
        GpuFaultKind::DeviceLost,
        GpuFaultKind::Internal,
        GpuFaultKind::SurfaceRecoveryFailed,
        GpuFaultKind::OutOfMemory,
        GpuFaultKind::Validation,
    ] {
        let inbox = RefCell::new(Vec::new());
        let outcome = complete_uploads(
            GpuFrameOutcome::Skipped(GpuFrameSkip::SurfaceReconfigured),
            || std::mem::take(&mut *inbox.borrow_mut()),
            || {
                inbox.borrow_mut().push(fault(GpuFaultKind::Validation));
                inbox.borrow_mut().push(fault(kind));
            },
        );
        let outcome = follow_surface(outcome, || panic!("fault must suppress configure/recreate"));
        assert!(matches!(outcome, GpuFrameOutcome::Fault(f) if f.kind == kind));
    }
    for high in [
        GpuFaultKind::DeviceLost,
        GpuFaultKind::SurfaceRecoveryFailed,
        GpuFaultKind::Internal,
        GpuFaultKind::OutOfMemory,
    ] {
        let outcome = merge_faults(
            GpuFrameOutcome::Fault(fault(high)),
            vec![fault(GpuFaultKind::Validation)],
        );
        assert!(matches!(outcome, GpuFrameOutcome::Fault(f) if f.kind == high));
    }
    let precise = GpuFault {
        kind: GpuFaultKind::Validation,
        message: "actual acquisition error".into(),
    };
    let outcome = merge_faults(
        GpuFrameOutcome::Fault(fault(GpuFaultKind::Validation)),
        vec![precise],
    );
    assert!(
        matches!(outcome, GpuFrameOutcome::Fault(f) if f.message == "actual acquisition error")
    );
    let outcome = follow_surface(
        GpuFrameOutcome::Skipped(GpuFrameSkip::SurfaceRecreated),
        || Err(fault(GpuFaultKind::SurfaceRecoveryFailed)),
    );
    match outcome {
        GpuFrameOutcome::Fault(f) => assert_eq!(f.kind, GpuFaultKind::SurfaceRecoveryFailed),
        _ => panic!("surface recovery fault disappeared"),
    }
}

#[test]
fn acquired_image_drops_before_flush_and_configuration_or_submission_fault() {
    let _guard = crate::testlock::serial();
    struct Image<'a>(&'a Cell<bool>);
    impl Drop for Image<'_> {
        fn drop(&mut self) {
            self.0.set(false);
        }
    }
    for fail_submit in [false, true] {
        let alive = Cell::new(true);
        let events = RefCell::new(Vec::new());
        let inbox = RefCell::new(Vec::new());
        let outcome = super::super::acquire::reconfigure_after_discard(Image(&alive), || {
            let outcome = complete_uploads(
                GpuFrameOutcome::Skipped(GpuFrameSkip::SurfaceReconfigured),
                || std::mem::take(&mut *inbox.borrow_mut()),
                || {
                    assert!(!alive.get());
                    events.borrow_mut().push("submit");
                    if fail_submit {
                        inbox.borrow_mut().push(fault(GpuFaultKind::DeviceLost));
                    }
                },
            );
            follow_surface(outcome, || {
                assert!(!alive.get());
                events.borrow_mut().push("configure");
                Ok(())
            })
        });
        assert_eq!(
            *events.borrow(),
            if fail_submit {
                vec!["submit"]
            } else {
                vec!["submit", "configure"]
            }
        );
        assert!(matches!(outcome, GpuFrameOutcome::Fault(_)) == fail_submit);
    }
}

#[cfg(target_os = "macos")]
pub(in crate::app::gpu) mod metal;
