//! The fold chevron's DIRECTION law: a collapsed heading and an
//! expanded one must draw genuinely DIFFERENT marks in a still frame, before
//! any animation exists. This is the defect's core (`fold_chevron.rs` used to
//! shape one fixed glyph, `\u{203A}`, regardless of fold state) and the exact
//! property CLAUDE.md's tripwire list warns a law can miss: "a law that
//! counts instances or measures extent cannot see the property it is named
//! for" — a selected mark can change shape completely while three
//! `instance_count() == 2` laws stay green, because a chevron is ALSO two
//! segments spanning the same box. Every law below grades the ANGLE or the
//! ink's own SHAPE — never a count — so a reintroduced direction-blind bug
//! cannot hide behind "the mechanism still fired".

use super::super::*;
use super::{headless_dqp, pixeldiff, view_md};
use crate::selection::chevron_arms;
use std::collections::BTreeSet;

const EPS: f32 = 0.01;

/// Reconstruct a `spine_segment` quad's own `(from, to)` endpoints from its
/// `(center, half, axis)` output — the inverse of the geometry
/// `chevron_arms` composes through (`half[0]` is the segment's own
/// half-LENGTH, so `center ∓ axis*half[0]` are exactly the two points it was
/// built from — see `selection::spine::spine_segment`'s own doc).
fn endpoints(seg: ([f32; 2], [f32; 2], [f32; 2])) -> ([f32; 2], [f32; 2]) {
    let (center, half, axis) = seg;
    let a = [center[0] - axis[0] * half[0], center[1] - axis[1] * half[0]];
    let b = [center[0] + axis[0] * half[0], center[1] + axis[1] * half[0]];
    (a, b)
}

/// Both `chevron_arms` segments are built `spine_segment(vertex, arm, …)`
/// — the SAME `vertex` for both — so their `endpoints().0` must agree exactly.
/// Asserting that agreement here is the direct pixel-level twin of
/// `chrome::diagonal::selected_chevron`'s own claim: "deriving the vertex FROM
/// the arm ends… makes the mirror structural".
fn vertex_of(arms: [([f32; 2], [f32; 2], [f32; 2]); 2]) -> [f32; 2] {
    let v0 = endpoints(arms[0]).0;
    let v1 = endpoints(arms[1]).0;
    assert!(
        (v0[0] - v1[0]).abs() < EPS && (v0[1] - v1[1]).abs() < EPS,
        "both arms of one chevron_arms call must meet at the SAME vertex: {v0:?} vs {v1:?}"
    );
    v0
}

/// THE PURE-GEOMETRY LAW — the item's core claim, graded on the ANGLE, not
/// the instance count (both states always draw exactly 2 segments; that is
/// precisely what a counting law cannot tell apart). At `turn=0°` (collapsed,
/// `›`) the shared vertex sits `reach` to the RIGHT of centre; at `turn=90°`
/// (expanded, `⌄`) it sits `reach` BELOW centre — never the same point.
#[test]
fn fold_chevron_arms_point_right_when_collapsed_and_down_when_expanded() {
    let center = [100.0, 50.0];
    let reach = 4.0;
    let spread = 3.0;
    let thickness = 1.5;

    let vertex_collapsed = vertex_of(chevron_arms(center, reach, spread, 0.0, thickness));
    let vertex_expanded = vertex_of(chevron_arms(center, reach, spread, 90.0, thickness));

    assert!(
        (vertex_collapsed[0] - (center[0] + reach)).abs() < EPS
            && (vertex_collapsed[1] - center[1]).abs() < EPS,
        "collapsed (›) vertex must sit `reach` to the RIGHT of centre, got {vertex_collapsed:?}"
    );
    assert!(
        (vertex_expanded[0] - center[0]).abs() < EPS
            && (vertex_expanded[1] - (center[1] + reach)).abs() < EPS,
        "expanded (⌄) vertex must sit `reach` BELOW centre, got {vertex_expanded:?}"
    );
    // Non-vacuous: the two vertices are NOT the same point — the exact
    // failure mode a direction-blind mark shows (one fixed shape regardless
    // of fold state).
    assert!(
        (vertex_collapsed[0] - vertex_expanded[0]).abs() > reach * 0.5
            || (vertex_collapsed[1] - vertex_expanded[1]).abs() > reach * 0.5,
        "collapsed and expanded must NOT share a vertex — that IS the item's own \
         regression: {vertex_collapsed:?} vs {vertex_expanded:?}"
    );
}

/// THE ANIMATION BASIS — a partial turn lands the vertex on the expected
/// point along a continuous rotation, strictly between the two settled
/// angles, never snapped to either. The harness has no clock (CLAUDE.md: "the
/// headless path has no clock, animation, or randomness"), so this cannot
/// prove the live GLIDE looks right — only that the pure function driving it
/// is a genuine rotation and not, say, a hard cut at `turn >= 45`. The glide
/// itself is flagged for human confirmation in the round's report.
#[test]
fn fold_chevron_arms_turn_continuously_between_the_two_settled_angles() {
    let center = [0.0, 0.0];
    let reach = 10.0;
    let spread = 6.0;
    let thickness = 1.0;

    let mid = vertex_of(chevron_arms(center, reach, spread, 45.0, thickness));
    let theta = 45f32.to_radians();
    let expected = [reach * theta.cos(), reach * theta.sin()];
    assert!(
        (mid[0] - expected[0]).abs() < EPS && (mid[1] - expected[1]).abs() < EPS,
        "a 45° turn must land the vertex on the rotation's own expected point, got {mid:?}, \
         expected {expected:?}"
    );

    let collapsed = vertex_of(chevron_arms(center, reach, spread, 0.0, thickness));
    let expanded = vertex_of(chevron_arms(center, reach, spread, 90.0, thickness));
    assert!(
        (mid[0] - collapsed[0]).abs() > EPS || (mid[1] - collapsed[1]).abs() > EPS,
        "a mid-turn vertex must not equal the collapsed settled point"
    );
    assert!(
        (mid[0] - expanded[0]).abs() > EPS || (mid[1] - expanded[1]).abs() > EPS,
        "a mid-turn vertex must not equal the expanded settled point"
    );
}

/// THE DIRECTION SOURCE'S OWN EDGE CASE: a heading folded over an EMPTY
/// section (immediately followed by a sibling-or-shallower heading, hiding
/// ZERO lines) must still read collapsed. `fold_tails` alone cannot answer
/// this — [`crate::fold::fold_tails`] is gated on a nonzero hidden count — so
/// `TextPipeline::folded_headings` reads the real fold set instead
/// (`ViewState::folded_headings`'s own doc names this exact gap). Non-vacuous
/// against the naive "derive collapsed from fold_tails" implementation: that
/// reading would report this heading as EXPANDED (`fold_tails` is empty here).
#[test]
fn a_heading_folded_over_an_empty_section_still_reads_collapsed() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("no GPU adapter; skipping empty-section fold-direction law");
        return;
    };
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    // "# A" immediately followed by "# B": folding # A hides ZERO lines.
    let doc = "# A\n# B\nbody";
    let levels = crate::fold::heading_levels(doc, true);
    let folds: BTreeSet<usize> = [0].into_iter().collect();
    let tails = crate::fold::fold_tails(&levels, &folds);
    assert!(
        tails.is_empty(),
        "fixture self-check: the folded section really is empty"
    );
    let hidden = crate::fold::hidden_lines(&levels, &folds);

    let mut view = view_md(doc, 0, 0);
    crate::fold::apply_to_view(&mut view, &hidden, &tails, &folds);
    p.set_view(&view);

    let geoms = p.fold_chevron_geometries();
    assert_eq!(
        geoms.len(),
        1,
        "the caret sits on the (still-visible, never-hidden) folded heading"
    );
    assert!(
        geoms[0].collapsed,
        "an empty-section fold must still read COLLAPSED — fold_tails alone (empty \
         here) cannot answer this; folded_headings must"
    );
    crate::page::set_page_on(true);
}

/// THE STEPPING MECHANISM, on INJECTED dt — the deterministic half of the
/// animation. `advance(dt)` takes an injected delta, not a real clock (exactly
/// the mechanism `--capture-timeline`/`--capture-held` drive for the caret
/// spring — `capture/animated.rs`'s own doc, and
/// `caret::tests::spring_settle::timeline_injected_dt_progresses_and_is_deterministic`
/// is this test's direct sibling), so STEPPING it is exactly as deterministic
/// as any other pure function: what the harness genuinely cannot reach is the
/// real-time FEEL, not the stepping arithmetic. Primes the turn map at the
/// pre-toggle settled state (mirroring the caret spring's own "first target
/// snaps, no glide" contract — `CaretAnim::set_target`'s doc), THEN toggles
/// the fold and steps a virtual clock: the fraction must progress
/// MONOTONICALLY from 0 toward 1, land strictly BETWEEN the two at an early
/// step (never a hard cut), and settle EXACTLY at 1.0 once enough virtual time
/// has passed. The GLIDE'S real-time feel stays flagged for human
/// confirmation — this proves the math it rides is sound.
#[test]
fn fold_chevron_turn_progresses_on_injected_dt_and_settles_exactly() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("no GPU adapter; skipping fold-chevron injected-dt stepping law");
        return;
    };
    let _g = crate::testlock::serial();
    assert!(
        !crate::motion::reduced(),
        "fixture self-check: reduce-motion must be off for this law to see a glide"
    );
    crate::page::set_page_on(true);
    let text = "# A\na1\na2\n# B\nb1";

    // Start COLLAPSED (fold # A) — caret on the heading so its chevron summons.
    let levels = crate::fold::heading_levels(text, true);
    let folds: BTreeSet<usize> = [0].into_iter().collect();
    let hidden = crate::fold::hidden_lines(&levels, &folds);
    let tails = crate::fold::fold_tails(&levels, &folds);
    let mut collapsed_view = view_md(text, 0, 0);
    crate::fold::apply_to_view(&mut collapsed_view, &hidden, &tails, &folds);
    p.set_view(&collapsed_view);
    // PRIME: the first `advance` call seeds the turn map at the CURRENT
    // (collapsed) settled target with no glide.
    p.advance(0.0);
    assert_eq!(
        p.fold_chevron_turn_fraction(0, true),
        0.0,
        "primed collapsed state must read the exact settled › (turn fraction 0.0)"
    );

    // TOGGLE: unfold # A. Its target flips to 1.0 (⌄); the turn map already
    // holds a PRE-toggle entry at 0.0, so this is a genuine glide, not a
    // second prime.
    let unfolded_view = view_md(text, 0, 0);
    p.set_view(&unfolded_view);

    let steps_ms: [u32; 4] = [16, 50, 150, 400];
    let mut prev_ms = 0u32;
    let mut fractions = Vec::new();
    for &t in &steps_ms {
        let dt = (t.saturating_sub(prev_ms)) as f32 / 1000.0;
        prev_ms = t;
        p.advance(dt);
        fractions.push(p.fold_chevron_turn_fraction(0, false));
    }

    for w in fractions.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-6,
            "the turn fraction must progress MONOTONICALLY toward the new target: {fractions:?}"
        );
    }
    assert!(
        fractions[0] > 0.0 && fractions[0] < 1.0,
        "an early step must be MID-GLIDE, neither the old nor the new settled value: {:?}",
        fractions[0]
    );
    let last = *fractions.last().unwrap();
    assert!(
        (last - 1.0).abs() < 1e-3,
        "late enough into the glide the mark must SETTLE exactly at 1.0 (⌄): {last}"
    );
    crate::page::set_page_on(true);
}

/// THE HEADLINE STILL-FRAME LAW, on real rendered pixels rather than only the
/// pure formula above. Isolates the mark exactly as
/// `fold_chevron_center.rs` does (differenced against a no-hover REST
/// frame, so only the mark's own ink survives), then measures each state's
/// own ink BOUNDING BOX. `fold_chevron.rs`'s `REACH_CHARS`/`SPREAD_CHARS` are
/// deliberately unequal, so the collapsed mark's box reads WIDER than tall and
/// the expanded mark's reads TALLER than wide — an aspect-ratio INVERSION,
/// not merely "some pixel changed". A same-shape regression collapses both
/// ratios toward the same side of 1.0; this is the law watched red against
/// exactly that mutation (see the round's report for the panic text).
#[test]
fn fold_chevron_ink_bbox_flips_aspect_between_collapsed_and_expanded() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    crate::page::set_page_on(true);
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item-248 fold-chevron aspect law: no GPU adapter");
        return;
    };
    crate::theme::set_active_by_name("Wagtail").unwrap();
    p.sync_theme();

    // Two sibling sections, no soft-wrap — the same fixture `folds.rs` uses.
    let text = "# A\na1\na2\n# B\nb1";

    let bbox_for = |p: &mut TextPipeline, collapsed: bool| -> (i64, i64, i64, i64) {
        // Caret parked away from the heading in BOTH states (line 4), so only
        // HOVER toggles the chevron — the same isolation the hover laws
        // in `folds.rs` use, so WYSIWYG conceal never contaminates the diff.
        let mut rest = view_md(text, 4, 0);
        if collapsed {
            let levels = crate::fold::heading_levels(text, true);
            let folds: BTreeSet<usize> = [0].into_iter().collect();
            let hidden = crate::fold::hidden_lines(&levels, &folds);
            let tails = crate::fold::fold_tails(&levels, &folds);
            crate::fold::apply_to_view(&mut rest, &hidden, &tails, &folds);
        }
        p.set_view(&rest);
        p.set_hover_line(None);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let before = pixeldiff::render_frame(p, &device, &queue, 1200, 800);

        p.set_hover_line(Some(0));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geoms = p.fold_chevron_geometries();
        assert_eq!(
            geoms.len(),
            1,
            "hovering the heading must summon exactly one mark"
        );
        let geom = geoms[0];
        assert_eq!(
            geom.collapsed, collapsed,
            "fixture self-check: the summoned mark's own collapsed bit must match \
             what this closure asked for"
        );
        let after = pixeldiff::render_frame(p, &device, &queue, 1200, 800);

        let x0 = p.column_left().floor() as i64;
        let x1 = p.text_left().ceil() as i64;
        let y0 = (geom.row_top - geom.row_height).floor() as i64;
        let y1 = (geom.row_top + geom.row_height * 2.0).ceil() as i64;
        let (mut minx, mut miny, mut maxx, mut maxy) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
        for y in y0.max(0)..y1.min(800) {
            for x in x0.max(0)..x1.min(1200) {
                let i = (y * 1200 + x) as usize;
                if before[i] != after[i] {
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                    miny = miny.min(y);
                    maxy = maxy.max(y);
                }
            }
        }
        assert!(
            maxx >= minx && maxy >= miny,
            "the {} chevron must paint SOME pixel in its own margin lane",
            if collapsed { "collapsed" } else { "expanded" }
        );
        (minx, miny, maxx, maxy)
    };

    let (l0, t0, r0, b0) = bbox_for(&mut p, true);
    let (l1, t1, r1, b1) = bbox_for(&mut p, false);
    let w0 = (r0 - l0 + 1) as f32;
    let h0 = (b0 - t0 + 1) as f32;
    let w1 = (r1 - l1 + 1) as f32;
    let h1 = (b1 - t1 + 1) as f32;
    let ratio_collapsed = w0 / h0;
    let ratio_expanded = w1 / h1;
    assert!(
        ratio_collapsed > 1.05,
        "collapsed (›) ink must read WIDER than tall: bbox {w0}x{h0}px, ratio {ratio_collapsed:.3}"
    );
    assert!(
        ratio_expanded < 0.95,
        "expanded (⌄) ink must read TALLER than wide: bbox {w1}x{h1}px, ratio {ratio_expanded:.3}"
    );
    crate::page::set_page_on(true);
}
