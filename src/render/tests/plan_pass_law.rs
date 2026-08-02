//! HOW MANY SCENE PLANS ONE OVERLAY FRAME IS ENTITLED TO, swept over the WHOLE
//! world roster.
//!
//! The plan-count witness lives in `--bench-suite`'s palette cell. That is a
//! hidden dev tool and is deliberately NOT in the native gate, so while it lived
//! only there the invariant it protects — "a consumer grew its own plan" — was
//! unguarded by every gate that actually runs. Worse, the suite pinned ONE world
//! and the witness had never been asked any other, so the count it asserted was
//! the count that one world happened to produce.
//!
//! THE AXIS THAT MATTERS IS THE CARD ANCHOR, not the list style. A frame's own
//! plan is built once and completed in place
//! (`OverlayRowPlan::complete_row_extent`), diagonal or not; the second plan that
//! genuinely exists belongs to the RIGHT-ANCHORED content-hug measurement, which
//! shapes a PROVISIONAL card inside `set_view` to learn how wide to hug — and
//! which the diagonal world Magpie (top-LEFT) does not run while the upright
//! poster world Cassowary (top-RIGHT) does. A law swept only over list styles
//! would have graded the wrong axis and passed.
//!
//! So this law runs every world, and asserts the plan count against
//! [`FramePasses`] — the SAME named-pass owner the bench cell settles against,
//! not a second copy of it.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;
use crate::render::benchsuite::{FramePasses, PlanWitness};

/// A summoned FLAT command palette with a real corpus of candidates — the same
/// shape the bench's palette cell opens.
fn palette_view(n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Command.title();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = n / 3;
    v.overlay_hint = "type to filter".into();
    v.overlay_window_rows = OverlayKind::Command.window_rows();
    v
}

/// THE HEADLINE — over every world, an overlay frame builds EXACTLY its named
/// planning passes and no more.
///
/// The oracle is independent of the counter it grades: `frames` is this test's
/// own loop bound and the pass names come from [`FramePasses::observe`], which
/// reads the hug MEASUREMENT'S OWN PRODUCT rather than the anchor predicate the
/// pipeline branches on. An extra plan anywhere in the frame — a consumer that
/// re-planned instead of reading the frame's plan — fails the sum by name.
#[test]
fn an_overlay_frame_builds_exactly_its_named_planning_passes() {
    let _g = crate::testlock::serial();
    const FRAMES: u64 = 4;
    const ITEMS: usize = 40;
    let (cw, ch) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(cw as f32, ch as f32) else {
        panic!("the plan-pass law requires a wgpu adapter");
    };
    let v = palette_view(ITEMS);

    let mut hugging: Vec<&'static str> = Vec::new();
    let mut plain: Vec<&'static str> = Vec::new();
    let mut diagonal: Vec<&'static str> = Vec::new();
    for (idx, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(idx);
        p.sync_theme();
        // One settled frame before the mark: the atlas warm-up and the first
        // shape are not the subject, and the witness counts plans, not frames.
        p.set_view(&v);
        p.prepare(&device, &queue, cw, ch).unwrap();

        let witness = PlanWitness::mark();
        for _ in 0..FRAMES {
            p.set_view(&v);
            p.prepare(&device, &queue, cw, ch).unwrap();
        }
        let passes = FramePasses::observe(&p);
        let (plans_per_frame, mean_rows) = witness
            .settle(FRAMES, &passes, v.overlay_window_rows, ITEMS as u64)
            .unwrap_or_else(|e| panic!("{}: {e}", world.name));
        assert!(
            mean_rows > 0,
            "{}: a drawn card must plan rows ({plans_per_frame} plans/frame)",
            world.name
        );
        match passes.content_hug {
            true => hugging.push(world.name),
            false => plain.push(world.name),
        }
        if matches!(world.render_caps.list_style, theme::ListStyle::Diagonal(_)) {
            diagonal.push(world.name);
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();

    // NON-VACUITY, per arm. An aggregate "some worlds passed" would go green on
    // a roster that had quietly stopped reaching one of the two shapes.
    assert!(
        !plain.is_empty() && !hugging.is_empty(),
        "the sweep must grade BOTH pass shapes — one-plan worlds {plain:?}, \
         content-hug worlds {hugging:?}"
    );
    assert!(
        diagonal.len() >= 2,
        "the sweep must include both mirrored diagonal compositions, got {diagonal:?}"
    );
}

/// THE MEASURED CLUSTER REACHES THE PLAN AT ALL — the device-tier half of the
/// completion.
///
/// `set_view` clears `diagonal_cluster` on every view push, so a frame's plan is
/// ALWAYS built with no measurement in hand and the completion is the only path
/// by which the measured attachment span reaches the plan. Its arithmetic is
/// pinned purely (`render/plan/tests.rs`'s completion law); what a device can
/// add is that the seam is wired — that
/// a real frame on a real diagonal world leaves a NON-INERT extent, and that the
/// planned row span genuinely CONTAINS the ink the cluster drew.
///
/// The oracle is the cluster's own drawn label/accessory positions, which are
/// what places glyphs — not the plan's arithmetic read back to itself.
#[test]
fn a_real_diagonal_frame_leaves_the_planned_span_around_its_drawn_ink() {
    let _g = crate::testlock::serial();
    let (cw, ch) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(cw as f32, ch as f32) else {
        panic!("the diagonal span law requires a wgpu adapter");
    };
    let v = palette_view(40);
    let mut graded = 0usize;
    for name in ["Mangrove", "Magpie"] {
        theme::set_active_by_name(name).expect("the diagonal worlds exist");
        p.sync_theme();
        p.set_view(&v);
        p.prepare(&device, &queue, cw, ch).unwrap();

        let geom = p.overlay_geometry(cw);
        let plan = p.overlay_row_plan(&geom);
        let cluster = p
            .diagonal_cluster_probe()
            .unwrap_or_else(|| panic!("{name}: a diagonal world must measure a cluster"));
        assert!(
            plan.rows().iter().any(|r| r.dx != 0.0 || r.dw != 0.0),
            "{name}: the law must not pass on an inert row extent"
        );
        let (x0, x1) = plan.card_x_span();
        for row in plan.rows() {
            let (left, right) = (x0 + row.dx, x1 + row.dw);
            assert!(
                left <= cluster.label_left(row.display) + 0.5,
                "{name}: display {} — the planned span starts at {left} but the drawn \
                 label starts at {}",
                row.display,
                cluster.label_left(row.display)
            );
            assert!(
                right + 0.5 >= cluster.accessory_right(row.display),
                "{name}: display {} — the planned span ends at {right} but the drawn \
                 accessory ends at {}",
                row.display,
                cluster.accessory_right(row.display)
            );
            graded += 1;
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    assert!(
        graded > 10,
        "the law must grade real planned rows, got {graded}"
    );
}
