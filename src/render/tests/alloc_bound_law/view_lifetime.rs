//! The view-only clause of the shared-device allocation law and its real-device
//! retained-handle mutation witness.

use super::*;

/// No queue operation in this workload can pin a view: `PendingWrites` holds
/// buffers and textures, and the workload never submits a render pass. Keep
/// this class-specific bound tight so one retained view per test cannot hide
/// inside the portable unit's two-whole-workloads of asynchronous slack. The
/// law still asks the live backend whether destruction settles this tightly;
/// this constant is not a claim inferred from a backend implementation.
const VIEW_ACCUMULATION_SLACK: i64 = 2;

/// The view-only half of the accumulation law, shared verbatim with its
/// retained-view mutation witness below.
pub(super) fn assert_views_do_not_accumulate(samples: &[gpu_alloc::GpuLive], subject: &str) {
    let first = samples
        .first()
        .expect("the theme roster is never empty")
        .texture_views;
    let last = samples
        .last()
        .expect("the theme roster is never empty")
        .texture_views;
    let peak = samples
        .iter()
        .map(|sample| sample.texture_views)
        .max()
        .expect("the theme roster is never empty");

    assert!(
        peak <= first + VIEW_ACCUMULATION_SLACK,
        "{subject} peaked at {peak} live texture views after starting at {first}; the \
         allocation law permits only {VIEW_ACCUMULATION_SLACK} views of scheduling slack. \
         A view-only pin must not hide inside the portable object's wider buffer-oriented \
         slack. Per-sample readings: {samples:?}",
    );
    assert!(
        last <= first + VIEW_ACCUMULATION_SLACK,
        "{subject} ended at {last} live texture views after starting at {first}; the \
         allocation law permits only {VIEW_ACCUMULATION_SLACK} views of scheduling slack. \
         The end of the sweep is the retained-view leak shape. Per-sample readings: \
         {samples:?}",
    );
}

/// A retained TextureView is a real mutation of the allocation law's subject,
/// not a synthetic counter value. On every backend whose live probe enrolls
/// `texture_views`, keep only one offscreen view per test-shaped workload and
/// require the exact class-specific assertion used by law 4 to reject it.
///
/// This is the lifetime gap in the portable sum's broad asynchronous slack:
/// early view-only growth can fit inside two whole workloads of buffer-oriented
/// slack. The view counter itself has no such limitation, so it gets its own
/// measured bound.
#[test]
fn retained_view_mutation_is_rejected_on_every_enrolled_backend() {
    let _g = crate::testlock::serial();
    let Some((device, queue, probe)) = instrumented() else {
        eprintln!(
            "skipping retained_view_mutation_is_rejected_on_every_enrolled_backend: \
             no wgpu adapter"
        );
        return;
    };
    if !probe.responds(Class::TextureViews) {
        eprintln!(
            "skipping retained-view mutation witness: this backend did not enroll \
             texture_views (probe: {probe})"
        );
        return;
    }
    assert!(
        probe.view_only_pin() > 0,
        "the live backend enrolled texture_views at creation but retaining only a view after \
         dropping its backing Texture moved the view counter by {} (probe: {probe}). This \
         oracle cannot witness the lifetime it claims to bound on this backend",
        probe.view_only_pin(),
    );

    let base = settled(&device, &queue);
    let mut retained_views = Vec::with_capacity(theme::THEMES.len());
    let mut samples = Vec::with_capacity(theme::THEMES.len());
    for _ in theme::THEMES {
        let (texture, view) = dither::offscreen(&device, W, H);
        drop(texture);
        retained_views.push(view);
        samples.push(gpu_alloc::live(&device).since(base));
    }
    assert_eq!(
        retained_views.len(),
        theme::THEMES.len(),
        "the mutation witness must retain exactly one view per workload"
    );

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_views_do_not_accumulate(&samples, "the retained-view mutation");
    }));
    assert!(
        result.is_err(),
        "the exact view-accumulation assertion used by law 4 accepted one retained view per \
         workload on an enrolled backend (probe: {probe}; readings: {samples:?})"
    );

    drop(retained_views);
    let after_drop = settled(&device, &queue).since(base).texture_views;
    assert!(
        after_drop <= VIEW_ACCUMULATION_SLACK,
        "dropping the mutation's only retained handles left {after_drop} extra live texture \
         views; the witness may be retaining some other owner and therefore would not prove \
         a view-only lifetime (probe: {probe})"
    );
}
