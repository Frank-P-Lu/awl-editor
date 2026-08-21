//! The conditional frame clock shared by the live host and every animator.
//!
//! A frame is either quiet, waiting for one deadline, or presenting until a
//! closed set of visible activities settles.  The renderer reports activities;
//! the host reduces them with sparse deadlines.  Nothing in this module owns a
//! timer, thread, or fixed refresh rate.

use crate::clock::Instant;
use std::time::Duration;

/// What Reduce Motion does to an enrolled activity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReducedMotion {
    Settle,
}

/// What happens to an enrolled activity while presentation is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PauseBehavior {
    ParkUntilPresented,
}

// One declaration owns the enum, names, reduced-motion behavior, and pause
// behavior. Adding an animator anywhere else is impossible; adding it here
// cannot compile without naming every policy the scheduler needs.
macro_rules! activity_roster {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(u8)]
        pub(crate) enum Activity {
            $($variant),+
        }

        impl Activity {
            pub(crate) const ALL: [Self; activity_roster!(@count $($variant),+)] = [
                $(Self::$variant),+
            ];

            pub(crate) const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            pub(crate) const fn reduced_motion(self) -> ReducedMotion {
                match self {
                    $(Self::$variant => ReducedMotion::Settle),+
                }
            }

            pub(crate) const fn pause_behavior(self) -> PauseBehavior {
                match self {
                    $(Self::$variant => PauseBehavior::ParkUntilPresented),+
                }
            }
        }
    };
    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(activity_roster!(@unit $variant)),+])
    };
    (@unit $variant:ident) => { () };
}

activity_roster! {
    CaretMotion => "caret-motion",
    CaretPreview => "caret-preview",
    CopyPulse => "copy-pulse",
    OverlayEntrance => "overlay-entrance",
    OverlayBand => "overlay-band",
    FoldChevrons => "fold-chevrons",
    TravellingGround => "travelling-ground",
}

/// A compact, copyable set of the activities still visible after prepare.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ActivitySet(u16);

impl ActivitySet {
    pub(crate) const fn empty() -> Self {
        Self(0)
    }

    pub(crate) const fn one(activity: Activity) -> Self {
        Self(1 << activity as u8)
    }

    pub(crate) fn insert(&mut self, activity: Activity) {
        self.0 |= Self::one(activity).0;
    }

    pub(crate) const fn contains(self, activity: Activity) -> bool {
        self.0 & Self::one(activity).0 != 0
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) fn iter(self) -> impl Iterator<Item = Activity> {
        Activity::ALL
            .into_iter()
            .filter(move |activity| self.contains(*activity))
    }

    pub(crate) fn names(self) -> String {
        self.iter()
            .map(Activity::name)
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// One monotonic sample shared by every activity in a presented frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameSample {
    /// Clock-owned visible time. It advances only from a successful-present
    /// baseline and therefore remains fixed across parked wall-clock gaps.
    pub(crate) now: Instant,
    pub(crate) elapsed: Duration,
    wall_now: Instant,
}

impl FrameSample {
    pub(crate) fn elapsed_secs(self) -> f32 {
        self.elapsed.as_secs_f32()
    }

    #[cfg(test)]
    pub(crate) fn injected(now: Instant, elapsed: Duration) -> Self {
        Self {
            now,
            elapsed,
            wall_now: now,
        }
    }
}

/// The only three instructions the frame domain gives its host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Directive {
    Idle,
    Deadline(Instant),
    Animating(ActivitySet),
}

/// Sparse owners only propose wakes. The reducer keeps the earliest one.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Deadlines(Option<Instant>);

impl Deadlines {
    pub(crate) fn propose(&mut self, proposed: Option<Instant>) {
        if let Some(proposed) = proposed {
            self.0 = Some(self.0.map_or(proposed, |current| current.min(proposed)));
        }
    }

    pub(crate) const fn earliest(self) -> Option<Instant> {
        self.0
    }
}

/// Presentation-clock state. `last_presented` is only a delta source; whether
/// the loop is active is represented explicitly by `activities`.
#[derive(Debug, Default)]
pub(crate) struct FrameClock {
    last_wall_presented: Option<Instant>,
    visible_now: Option<Instant>,
    activities: ActivitySet,
    draw_once: bool,
}

impl FrameClock {
    pub(crate) fn sample(&self, now: Instant) -> FrameSample {
        let wall_elapsed = self
            .last_wall_presented
            .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        FrameSample {
            now: self
                .visible_now
                .map_or(now, |visible| visible + wall_elapsed),
            elapsed: if self.activities.is_empty() {
                Duration::ZERO
            } else {
                wall_elapsed
            },
            wall_now: now,
        }
    }

    pub(crate) fn presented(&mut self, sample: FrameSample, activities: ActivitySet) {
        debug_assert!(
            activities
                .iter()
                .all(|activity| activity.reduced_motion() == ReducedMotion::Settle)
        );
        self.last_wall_presented = Some(sample.wall_now);
        self.visible_now = Some(sample.now);
        self.activities = activities;
    }

    /// Failed/occluded/unfocused presentation pauses elapsed-time sampling and
    /// parks every activity. The renderer retains its pose for the next wake.
    pub(crate) fn park(&mut self) {
        debug_assert!(
            self.activities
                .iter()
                .all(|activity| { activity.pause_behavior() == PauseBehavior::ParkUntilPresented })
        );
        self.last_wall_presented = None;
        self.activities = ActivitySet::empty();
    }

    pub(crate) fn directive(&self, deadlines: Deadlines) -> Directive {
        if !self.activities.is_empty() {
            Directive::Animating(self.activities)
        } else if let Some(deadline) = deadlines.earliest() {
            Directive::Deadline(deadline)
        } else {
            Directive::Idle
        }
    }

    pub(crate) fn demand_draw_once(&mut self) {
        self.draw_once = true;
    }

    pub(crate) fn take_draw_once(&mut self) -> bool {
        std::mem::take(&mut self.draw_once)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_names_wake_reduced_motion_and_pause_without_a_wildcard() {
        for activity in Activity::ALL {
            let (name, reduced, paused) = match activity {
                Activity::CaretMotion => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::CaretPreview => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::CopyPulse => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::OverlayEntrance => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::OverlayBand => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::FoldChevrons => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
                Activity::TravellingGround => (
                    activity.name(),
                    activity.reduced_motion(),
                    activity.pause_behavior(),
                ),
            };
            assert!(!name.is_empty());
            assert_eq!(reduced, ReducedMotion::Settle);
            assert_eq!(paused, PauseBehavior::ParkUntilPresented);
        }
    }

    #[test]
    fn reducer_prefers_animation_then_the_earliest_deadline_then_idle() {
        let base = Instant::now();
        let mut clock = FrameClock::default();
        assert_eq!(clock.directive(Deadlines::default()), Directive::Idle);
        let mut deadlines = Deadlines::default();
        deadlines.propose(Some(base + Duration::from_secs(2)));
        assert_eq!(
            clock.directive(deadlines),
            Directive::Deadline(base + Duration::from_secs(2))
        );
        let sample = clock.sample(base);
        clock.presented(sample, ActivitySet::one(Activity::CopyPulse));
        assert_eq!(
            clock.directive({
                let mut deadlines = Deadlines::default();
                deadlines.propose(Some(base + Duration::from_millis(1)));
                deadlines
            }),
            Directive::Animating(ActivitySet::one(Activity::CopyPulse))
        );
    }

    #[test]
    fn deadline_reduction_chooses_the_earliest_proposal() {
        let base = Instant::now();
        let mut deadlines = Deadlines::default();
        deadlines.propose(Some(base + Duration::from_millis(100)));
        deadlines.propose(None);
        deadlines.propose(Some(base + Duration::from_millis(40)));
        deadlines.propose(Some(base + Duration::from_millis(70)));
        assert_eq!(
            FrameClock::default().directive(deadlines),
            Directive::Deadline(base + Duration::from_millis(40))
        );
    }

    #[test]
    fn only_presented_samples_advance_and_a_parked_surface_restarts_at_zero() {
        let base = Instant::now();
        let mut clock = FrameClock::default();
        let first = clock.sample(base);
        assert_eq!(first.elapsed, Duration::ZERO);
        clock.presented(first, ActivitySet::empty());
        let idle = clock.sample(base + Duration::from_millis(8));
        assert_eq!(idle.elapsed, Duration::ZERO);
        assert_eq!(idle.now, base + Duration::from_millis(8));
        clock.presented(idle, ActivitySet::one(Activity::CaretMotion));
        assert_eq!(
            clock.sample(base + Duration::from_millis(16)).elapsed,
            Duration::from_millis(8)
        );
        clock.park();
        let resumed = clock.sample(base + Duration::from_secs(10));
        assert_eq!(
            (resumed.elapsed, resumed.now),
            (Duration::ZERO, base + Duration::from_millis(8)),
            "an occluded interval must not fast-forward visible motion"
        );
    }

    #[test]
    fn parked_animation_yields_to_the_retry_deadline() {
        let base = Instant::now();
        let mut clock = FrameClock::default();
        clock.presented(clock.sample(base), ActivitySet::one(Activity::CaretMotion));
        clock.park();
        let retry = base + Duration::from_millis(100);
        let mut deadlines = Deadlines::default();
        deadlines.propose(Some(retry));
        assert_eq!(clock.directive(deadlines), Directive::Deadline(retry));
    }

    #[test]
    fn draw_once_is_one_edge_not_a_loop_state() {
        let mut clock = FrameClock::default();
        assert!(!clock.take_draw_once());
        clock.demand_draw_once();
        assert!(clock.take_draw_once());
        assert!(!clock.take_draw_once());
        assert_eq!(clock.directive(Deadlines::default()), Directive::Idle);
    }

    #[test]
    fn every_activity_becomes_idle_exactly_once_when_retired() {
        let base = Instant::now();
        for activity in Activity::ALL {
            let mut clock = FrameClock::default();
            clock.presented(clock.sample(base), ActivitySet::one(activity));
            assert!(matches!(
                clock.directive(Deadlines::default()),
                Directive::Animating(_)
            ));
            clock.presented(
                clock.sample(base + Duration::from_millis(16)),
                ActivitySet::empty(),
            );
            assert_eq!(
                clock.directive(Deadlines::default()),
                Directive::Idle,
                "{activity:?}"
            );
            assert_eq!(
                clock.directive(Deadlines::default()),
                Directive::Idle,
                "{activity:?}"
            );
        }
    }
}
