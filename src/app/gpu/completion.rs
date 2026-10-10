//! Pending uploads belong to a submission even when no image is presented.
use super::*;

pub(super) enum SurfaceFollowup {
    None,
    Reconfigure,
    Recreate,
}

fn priority(kind: GpuFaultKind) -> u8 {
    match kind {
        GpuFaultKind::DeviceLost => 0,
        GpuFaultKind::SurfaceRecoveryFailed => 1,
        GpuFaultKind::Internal => 2,
        GpuFaultKind::OutOfMemory => 3,
        GpuFaultKind::Validation => 4,
    }
}

fn merge_faults(mut outcome: GpuFrameOutcome, faults: Vec<GpuFault>) -> GpuFrameOutcome {
    for fault in faults {
        let replace = match &outcome {
            GpuFrameOutcome::Fault(existing) => priority(fault.kind) <= priority(existing.kind),
            _ => true,
        };
        if replace {
            outcome = GpuFrameOutcome::Fault(fault);
        }
    }
    outcome
}

/// No acquired image may remain alive at this boundary. Submit reports faults
/// through the existing inbox, not its returned SubmissionIndex. This drains
/// pending writes on healthy devices; it neither waits nor bounds stalled work.
fn complete_uploads(
    outcome: GpuFrameOutcome,
    mut faults: impl FnMut() -> Vec<GpuFault>,
    submit: impl FnOnce(),
) -> GpuFrameOutcome {
    if matches!(outcome, GpuFrameOutcome::Presented(_)) {
        return outcome;
    }
    let outcome = merge_faults(outcome, faults());
    if matches!(&outcome, GpuFrameOutcome::Fault(f) if f.kind != GpuFaultKind::Validation) {
        return outcome;
    }
    submit();
    merge_faults(outcome, faults())
}

fn follow_surface(
    outcome: GpuFrameOutcome,
    follow: impl FnOnce() -> Result<(), GpuFault>,
) -> GpuFrameOutcome {
    if matches!(outcome, GpuFrameOutcome::Skipped(_))
        && let Err(fault) = follow()
    {
        return GpuFrameOutcome::Fault(fault);
    }
    outcome
}

impl Gpu {
    pub(super) fn finish_unpresented(
        &mut self,
        outcome: GpuFrameOutcome,
        activities: PreparedActivities,
        followup: SurfaceFollowup,
    ) -> PreparedFrame {
        let outcome = complete_uploads(
            outcome,
            || self.take_faults(),
            || {
                self.queue.submit([]);
            },
        );
        let outcome = follow_surface(outcome, || match followup {
            SurfaceFollowup::None => Ok(()),
            SurfaceFollowup::Reconfigure => {
                self.surface.configure(&self.device, &self.config);
                Ok(())
            }
            SurfaceFollowup::Recreate => self.recover_surface(),
        });
        PreparedFrame {
            outcome: merge_faults(outcome, self.take_faults()),
            activities,
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
