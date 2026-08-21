//! THE HYBRID glide+snap OUTCOME ORACLE (the user's decision).
//!
//! The picker selection band ships the living-band MORPH glide. A SINGLE
//! deliberate selection move must GLIDE fully (the choreography plays and the
//! band settles exactly on the selected row). But under arrow-key AUTO-REPEAT
//! the glide (~110ms) is outrun: a new re-target arrives while the band is
//! still mid-flight. The former path CHAINED another lagging glide from
//! the in-flight position, so the drawn band trailed the logical selection —
//! "catches up every 2nd Down". The HYBRID (`TextPipeline::chase_or_snap`)
//! SNAPS on any in-flight re-target so `band == selection` at every step, while
//! a lone move still glides.
//!
//! These tests OBSERVE EVERY INTERMEDIATE MOVE (never only the settled index)
//! and express live CADENCE by feeding input timestamps as `advance(dt)` gaps
//! between re-targets — the same deterministic-motion instrument
//! `firetail_showcase.rs` drives. They cover BOTH band seams — Pane
//! (`living_band_phase`, the morph) and Bars (`overlay_band_drawn`, the slide)
//! — and the left/center/right card anchors through the real
//! `living_probe_geom` render path.
//!
//! Written FAILING-FIRST: the RAPID assertions fail on the chaining code (the
//! band trails the target) and pass once the snap arbiter lands; the SINGLE
//! assertions are the invariant that a lone move keeps gliding.

use super::super::*;
use super::{headless_pipeline, view};

use crate::render::livingband::{self, Choreo, MotionForce};

/// Snap/settle equality tolerance (device px). A snap is an EXACT float move
/// (`from == to == target`, a no-move morph rect), so this is generous slack.
const EPS: f32 = 0.05;
/// One glide duration in seconds (`OVERLAY_BAND_SLIDE_MS`), so a "rapid" gap is
/// a fraction of it and a "settle" gap is several times it.
const GLIDE_S: f32 = OVERLAY_BAND_SLIDE_MS.0 / 1000.0;

fn morph_force() -> MotionForce {
    MotionForce {
        choreo: Choreo::Morph,
        phase: None,
    }
}

/// The Pane band's DRAWN top this frame for selection `target` — the exact
/// value the fill quad rides (`living_band_phase` → `morph_band`). Advances the
/// re-target through the same seam `overlay_draw_card` uses.
fn pane_band_top(p: &mut TextPipeline, target: f32, lh: f32) -> f32 {
    let force = morph_force();
    let (from, to, t) = p.living_band_phase(force, target, lh);
    livingband::morph_band(from, to, lh, t, &force.choreo.params()).top
}

/// Arm a live pipeline with Reduce-Motion OFF (the only state where the band
/// animates at all), returning it plus the saved reduced-flag to restore.
fn armed() -> Option<(TextPipeline, bool)> {
    let _g = crate::testlock::serial();
    let mut p = headless_pipeline()?;
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    p.arm_live_juice();
    Some((p, saved))
}

// --- Pane (living-band MORPH) ------------------------------------------------

/// SINGLE cadence, Pane: a lone move GLIDES — the first frame draws at the OLD
/// row (not snapped), a mid-flight frame sits strictly between the two rows, and
/// the band settles EXACTLY on the selected row.
#[test]
fn pane_single_move_glides_through_the_middle_and_settles_on_selection() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping pane_single_move_glides: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let lh = 30.0f32;
    let row = |k: usize| 100.0 + k as f32 * lh;

    // Settle on row 0 (fresh overlay).
    assert!((pane_band_top(&mut p, row(0), lh) - row(0)).abs() < EPS);

    // Deliberate move 0 -> 1: the FIRST frame draws at the OLD row (gliding, not
    // snapped) — the whole point of "a single move glides".
    let start = pane_band_top(&mut p, row(1), lh);
    assert!(
        (start - row(0)).abs() < EPS,
        "glide starts at the previous row (got {start})"
    );
    assert!(
        (start - row(1)).abs() > lh * 0.5,
        "a single move must NOT snap to the target on frame 0 (got {start}, target {})",
        row(1)
    );

    // Mid-flight: strictly between the two rows (the morph is playing).
    p.advance(0.25 * GLIDE_S);
    let mid = pane_band_top(&mut p, row(1), lh);
    assert!(
        row(0) < mid && mid < row(1),
        "mid-glide sits between rows (got {mid})"
    );

    // Settles EXACTLY on the selected row.
    p.advance(3.0 * GLIDE_S);
    let settled = pane_band_top(&mut p, row(1), lh);
    assert!(
        (settled - row(1)).abs() < EPS,
        "the glide settles on the selection (got {settled})"
    );

    crate::motion::set_reduced(saved);
}

/// RAPID cadence, Pane: re-targets that arrive WHILE the band is still
/// mid-glide (auto-repeat) SNAP — `band == selection` at EVERY step, never a
/// trailing intermediate. FAILS on the chaining path (the band lags the row).
#[test]
fn pane_rapid_repeat_snaps_band_onto_selection_every_step() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping pane_rapid_repeat_snaps: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let lh = 30.0f32;
    let row = |k: usize| 100.0 + k as f32 * lh;

    // Settle on row 0, then PRIME a single glide (0 -> 1) and let it run only a
    // fraction of one glide duration — the band is now genuinely mid-flight.
    let _ = pane_band_top(&mut p, row(0), lh);
    p.advance(3.0 * GLIDE_S);
    let _ = pane_band_top(&mut p, row(0), lh);
    let _ = pane_band_top(&mut p, row(1), lh); // deliberate move -> glide begins
    p.advance(0.18 * GLIDE_S); // still mid-glide

    // Now auto-repeat: each Down lands within one glide duration of the last, so
    // every one must SNAP the band straight onto the fresh selection.
    for k in 2..=6 {
        let top = pane_band_top(&mut p, row(k), lh);
        assert!(
            (top - row(k)).abs() < EPS,
            "rapid step to row {k}: band must SNAP onto the selection ({}), got {top} \
             (chaining trails here)",
            row(k)
        );
        p.advance(0.18 * GLIDE_S); // next Down still within one glide duration
    }

    // Input goes quiet for a full glide: the NEXT move glides again (single
    // cadence resumes — the hybrid is not "always snap").
    p.advance(3.0 * GLIDE_S);
    let _ = pane_band_top(&mut p, row(6), lh);
    let resume = pane_band_top(&mut p, row(7), lh);
    assert!(
        (resume - row(6)).abs() < EPS,
        "after a quiet gap a fresh move GLIDES from the settled row again (got {resume})"
    );

    crate::motion::set_reduced(saved);
}

// --- Bars (ordinary slide via `overlay_band_drawn`) --------------------------

/// SINGLE cadence, Bars: the selected-bar slide glides from the previous row
/// and settles on the target (the Bars seam is `overlay_band_drawn`, armed
/// Slide — the same arbiter underneath).
#[test]
fn bars_single_move_glides_and_settles_on_selection() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping bars_single_move_glides: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    set_motion_test_override(Some(theme::MotionJuice {
        entrance: theme::OverlayEntrance::Instant,
        band: theme::BandResponse::Slide,
    }));

    // Settle at 100.
    let _ = p.overlay_band_drawn(100.0);
    p.advance(3.0 * GLIDE_S);
    assert!((p.overlay_band_drawn(100.0) - 100.0).abs() < EPS);

    // Move 100 -> 300: first frame at the old row, mid between, settles on 300.
    let start = p.overlay_band_drawn(300.0);
    assert!(
        (start - 100.0).abs() < EPS,
        "slide starts at the previous row (got {start})"
    );
    p.advance(0.25 * GLIDE_S);
    let mid = p.overlay_band_drawn(300.0);
    assert!(
        100.0 < mid && mid < 300.0,
        "mid-slide sits between rows (got {mid})"
    );
    p.advance(3.0 * GLIDE_S);
    assert!(
        (p.overlay_band_drawn(300.0) - 300.0).abs() < EPS,
        "slide settles on the target"
    );

    set_motion_test_override(None);
    crate::motion::set_reduced(saved);
}

/// RAPID cadence, Bars: in-flight re-targets snap the selected bar onto the
/// fresh row every step. FAILS on the chaining path.
#[test]
fn bars_rapid_repeat_snaps_band_onto_selection_every_step() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping bars_rapid_repeat_snaps: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    set_motion_test_override(Some(theme::MotionJuice {
        entrance: theme::OverlayEntrance::Instant,
        band: theme::BandResponse::Slide,
    }));
    let row = |k: usize| 100.0 + k as f32 * 30.0;

    // Settle, then prime a glide and go mid-flight.
    let _ = p.overlay_band_drawn(row(0));
    p.advance(3.0 * GLIDE_S);
    let _ = p.overlay_band_drawn(row(0));
    let _ = p.overlay_band_drawn(row(1));
    p.advance(0.18 * GLIDE_S);

    for k in 2..=6 {
        let top = p.overlay_band_drawn(row(k));
        assert!(
            (top - row(k)).abs() < EPS,
            "rapid Bars step to row {k}: band must SNAP onto {} (got {top})",
            row(k)
        );
        p.advance(0.18 * GLIDE_S);
    }

    set_motion_test_override(None);
    crate::motion::set_reduced(saved);
}

// --- Anchors: left / center / right, through the real render path ------------

/// RAPID cadence across the THREE card anchors (TopLeft / TopCenter /
/// TopRight), driven through the real `living_probe_geom` seam so the band's
/// drawn rect is read exactly where the fill lands. For every anchor the band
/// SNAPS vertically onto the selected row under auto-repeat; the anchors are
/// genuinely distinct (the card's x differs), so the vertical tracking is
/// proven anchor-independent, not vacuous.
#[test]
fn rapid_snap_holds_under_left_center_right_anchors() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping rapid_snap_holds_under_anchors: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    // Force Pane (some worlds default to Bars) and the calm MORPH voice, so the
    // living-band probe path is the one under test regardless of active world.
    set_list_style_test_override(Some(theme::ListStyle::Pane));
    livingband::set_motion_test_override(Some(morph_force()));

    let items: Vec<String> = (0..8).map(|i| format!("item {i}")).collect();
    let set_sel = |p: &mut TextPipeline, k: usize| {
        let mut v = view("hello\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = "commands";
        v.overlay_items = items.clone();
        v.overlay_selected = k;
        p.set_view(&v);
    };
    // One "frame": set the selection, then read the band's drawn top + x and the
    // settled top of that display row (both from the ONE probe seam).
    let read = |p: &mut TextPipeline, k: usize| -> (f32, f32, f32) {
        set_sel(p, k);
        let geom = p.overlay_geometry(1200);
        let (_covered, target, first_top, lh, prim) = p.living_probe_geom(&geom);
        (prim[1], prim[0], first_top + target as f32 * lh)
    };

    let mut card_xs = Vec::new();
    for (label, anchor) in [
        ("left", theme::CardAnchor::TopLeft),
        ("center", theme::CardAnchor::TopCenter),
        ("right", theme::CardAnchor::TopRight),
    ] {
        set_card_anchor_test_override(Some(anchor));

        // Settle on row 0 for this anchor.
        let _ = read(&mut p, 0);
        p.advance(3.0 * GLIDE_S);
        let (_, card_x, _) = read(&mut p, 0);
        card_xs.push((label, card_x));

        // Prime a glide, go mid-flight, then auto-repeat.
        let _ = read(&mut p, 1);
        p.advance(0.18 * GLIDE_S);
        for k in 2..=5 {
            let (band_top, _x, settled_top) = read(&mut p, k);
            assert!(
                (band_top - settled_top).abs() < 1.0,
                "anchor {label}, rapid step to row {k}: band top must SNAP onto the row \
                 (band {band_top}, row {settled_top})"
            );
            p.advance(0.18 * GLIDE_S);
        }

        set_card_anchor_test_override(None);
    }

    // The three anchors are genuinely distinct placements (not a vacuous sweep):
    // the card's left edge steps rightward left < center < right.
    let x = |name: &str| card_xs.iter().find(|(l, _)| *l == name).unwrap().1;
    assert!(
        x("left") < x("center") && x("center") < x("right"),
        "anchors must place the card distinctly (left {}, center {}, right {})",
        x("left"),
        x("center"),
        x("right")
    );

    set_list_style_test_override(None);
    livingband::set_motion_test_override(None);
    crate::motion::set_reduced(saved);
}

// --- The frames the glide needs must actually be SCHEDULED -----------------
//
// The laws above fix the band's chase ARITHMETIC. Every one of them
// hand-drives the clock — `p.advance(dt)` between re-targets — which is exactly
// why they all stayed green over the every-other-input defect: the live loop only calls `advance`
// when a frame is scheduled, and on the frame that STARTS an ease nothing
// scheduled one. The band's target is set inside `prepare`, and
// `App::on_redraw_requested` reads `TextPipeline::advance` BEFORE `Gpu::redraw`
// runs `prepare`, so the pre-prepare answer is "settled" on precisely the frame
// an ease begins. The loop then parked on `ControlFlow::Wait`, the band stayed
// drawn at `overlay_band_from` (the row the selection LEFT) while the logical
// selection had already moved, and the next input's single `dt` put the band
// back in flight so `chase_or_snap` SNAPPED two rows at once: "the selection
// advances only every second input, with no transition".
//
// The post-prepare activity set closes that ordering, and these laws sweep the
// missing axis: not "where is the band drawn", but "did this prepare tell the
// loop it owes another frame".

/// The exact ordering the live loop uses: `advance` first (what
/// `on_redraw_requested` reads into `stepped`), then the retarget that `prepare`
/// performs, then the post-prepare activity report.
fn one_live_frame(
    p: &mut TextPipeline,
    dt: f32,
    retarget: impl FnOnce(&mut TextPipeline),
) -> (bool, bool) {
    let stepped = p.advance(dt);
    retarget(p);
    (
        stepped,
        p.active_activities()
            .contains(crate::frame_clock::Activity::OverlayBand),
    )
}

/// THE SETTLED-BAND LAW, across both band seams. A move that arrives on a SETTLED band is
/// invisible to the `advance` that ran before `prepare` — that is the whole
/// defect — so the retarget must report itself, or the ease it just started
/// never gets a second frame.
#[test]
fn a_move_onto_a_settled_band_reports_the_ease_advance_could_not_see() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping settled_band_reports_the_ease: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let lh = 30.0f32;
    let row = |k: usize| 100.0 + k as f32 * lh;
    let dt = 1.0 / 60.0;

    // --- Pane (living-band morph) -------------------------------------------
    // Fresh overlay: settles, and must NOT claim a frame is owed.
    let (_, fresh) = one_live_frame(&mut p, dt, |p| {
        let _ = pane_band_top(p, row(0), lh);
    });
    assert!(!fresh, "a fresh overlay SETTLES the band; it owes no frame");
    let (stepped, started) = one_live_frame(&mut p, dt, |p| {
        let _ = pane_band_top(p, row(1), lh);
    });
    assert!(
        !stepped,
        "NON-VACUITY: the pre-prepare `advance` must report SETTLED here — if it \
         ever sees this move on its own, this law is proving nothing"
    );
    assert!(
        started,
        "Pane: a move onto a settled band must report the ease `advance` could \
         not see (item 211 — otherwise the loop parks and the band freezes on \
         the row the selection left)"
    );
    // The ease is now in flight, so the ORDINARY term carries it home and the
    // bridge must go quiet — it reports a START, never a continuation.
    let (stepped, started) = one_live_frame(&mut p, dt, |p| {
        let _ = pane_band_top(p, row(1), lh);
    });
    assert!(stepped, "mid-glide, `advance` itself keeps the loop hot");
    assert!(
        !started,
        "no new move: the bridge reports a START, not a continuation (it must \
         not manufacture a perpetual hot loop)"
    );

    // In-flight move (auto-repeat): `chase_or_snap`'s SNAP branch re-zeroes the
    // timer too, so it owes a frame just as much as a glide start does.
    let (_, started) = one_live_frame(&mut p, dt, |p| {
        let _ = pane_band_top(p, row(4), lh);
    });
    assert!(
        started,
        "Pane: an in-flight SNAP re-zeroes the ease and owes a frame"
    );

    // --- Bars (ordinary slide) ----------------------------------------------
    set_motion_test_override(Some(theme::MotionJuice {
        entrance: theme::OverlayEntrance::Instant,
        band: theme::BandResponse::Slide,
    }));
    let mut q = match headless_pipeline() {
        Some(q) => q,
        None => {
            set_motion_test_override(None);
            crate::motion::set_reduced(saved);
            return;
        }
    };
    q.arm_live_juice();
    let (_, fresh) = one_live_frame(&mut q, dt, |q| {
        let _ = q.overlay_band_drawn(row(0));
    });
    assert!(!fresh, "Bars: a fresh overlay settles and owes no frame");
    // Let it rest a full glide so the next move is a SETTLED-cadence glide.
    q.advance(3.0 * GLIDE_S);
    let _ = q.overlay_band_drawn(row(0));
    let (stepped, started) = one_live_frame(&mut q, dt, |q| {
        let _ = q.overlay_band_drawn(row(1));
    });
    assert!(
        !stepped,
        "NON-VACUITY (Bars): the pre-prepare advance reports settled"
    );
    assert!(
        started,
        "Bars: a move onto a settled slide must report the ease `advance` could not see"
    );
    set_motion_test_override(None);

    crate::motion::set_reduced(saved);
}

/// The accessibility + determinism half. Reduce Motion may SNAP the selection
/// but must never suppress the state change — and it must also not schedule a
/// frame for an ease it refused to play. An UNARMED pipeline (every headless
/// capture, bench and test that does not call `arm_live_juice`) is the same
/// story by construction: it never reaches `chase_or_snap` at all, so the live
/// loop's second animation term is a constant `false` off-window and the
/// byte-identity gates stand.
#[test]
fn reduce_motion_and_unarmed_pipelines_owe_no_frame_but_still_move_the_band() {
    let Some((mut p, saved)) = armed() else {
        eprintln!("skipping reduce_motion_owes_no_frame: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let lh = 30.0f32;
    let row = |k: usize| 100.0 + k as f32 * lh;
    let dt = 1.0 / 60.0;

    crate::motion::set_reduced(true);
    let _ = one_live_frame(&mut p, dt, |p| {
        let _ = pane_band_top(p, row(0), lh);
    });
    for k in 1..=4 {
        let (_, started) = one_live_frame(&mut p, dt, |p| {
            let _ = pane_band_top(p, row(k), lh);
        });
        assert!(
            !started,
            "Reduce Motion plays no ease, so it owes no follow-up frame (row {k})"
        );
        // …but the band is ON the selection immediately — the state change is
        // never suppressed, only its transition.
        let top = pane_band_top(&mut p, row(k), lh);
        assert!(
            (top - row(k)).abs() < EPS,
            "Reduce Motion SNAPS the band onto row {k} (got {top}, want {})",
            row(k)
        );
    }
    crate::motion::set_reduced(false);

    let Some(mut u) = headless_pipeline() else {
        crate::motion::set_reduced(saved);
        return;
    };
    // Deliberately NOT `arm_live_juice()` — the capture/bench shape.
    for k in 0..=4 {
        let (_, started) = one_live_frame(&mut u, dt, |u| {
            let _ = pane_band_top(u, row(k), lh);
        });
        assert!(
            !started,
            "an unarmed pipeline never eases, so it can never ask the live loop \
             for a frame (row {k})"
        );
    }

    crate::motion::set_reduced(saved);
}
