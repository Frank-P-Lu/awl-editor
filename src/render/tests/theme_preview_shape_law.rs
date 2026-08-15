//! THEME-PREVIEW SHAPE REACH — the paintable prefix each picker movement shapes,
//! the supersedable tail a burst leaves behind, and the fully settled result.
//!
//! The theme-picker arrow step was profiled in `--release` on this machine's
//! Metal. The per-stage split of one preview hop is not evenly spread and it is
//! not ours: re-tinting every baked pipeline (`sync_theme_colors`) is 0.01 ms,
//! adopting the new face (`theme_font_adopt`, whose `set_size` is a no-op
//! because `text_wrap_width` is derived from a face-INDEPENDENT `char_width`)
//! is 0.00 ms, rebuilding every line's `AttrsList` is 0.8 ms on a 119-line
//! document and 8.1 ms on a 1896-line one — and the single
//! `buffer.shape_until_scroll` that follows is 17–20 ms, which is ~95% of the
//! reshape and roughly half the whole input-to-present step. A sampling profile
//! puts that time inside harfrust's own `shape_with_plan`, so it is real glyph
//! shaping rather than a pathology with a cheap cure.
//!
//! THE REACH IS WHY IT COSTS THAT. [`TextPipeline::full_shape_height`] budgets a
//! buffer tall enough for EVERY visual row, so one arrow re-shapes the whole
//! document while only a viewport's worth of rows can be drawn — on the
//! fixtures above, ~95% of the shaped rows are off-screen. That reach cannot
//! simply be dropped: an unshaped row carries NO geometry, and answering for it
//! from the estimate is the scroll-jump / wrong-image-height bug
//! `full_shape_height`'s own doc records.
//!
//! A preview arrow shapes what its frame can paint. A newer arrow may supersede
//! the remaining tail; once input rests, the final world's tail is shaped once.
//! Deliberate paced movement therefore settles each world, while a burst avoids
//! finishing off-screen work the user has already left behind.
//!
//! THIS FILE PINS BOTH HALVES, because either one alone is satisfiable by a
//! broken product:
//!
//! * at the moment of the PRESENT the shaped reach is bounded by the WINDOW, not
//!   by the document — otherwise the split bought nothing — and it still covers
//!   the whole band `rects::row_box_visible` lets a frame paint into, so the
//!   presented frame can never need a row that was not shaped;
//! * at the STEP BOUNDARY every row of the document is shaped again. This is the
//!   original whole-document assertion, unweakened and unmoved: a permanently
//!   narrow budget — the thing the reach exists to prevent — fails it exactly as
//!   it did before, and so does a tail completion that is deleted, no-op'd, or
//!   left to some later step.
//!
//! The oracle is a ROW COUNT, not a duration: it is deterministic, it holds on
//! any backend and at any profile, and it moves for exactly the reason the cost
//! moves. A timing assertion could do none of that.
//!
//! [`identical_settled_geometry_whichever_reach_the_step_took`] then pins the
//! other half of the bargain: splitting the step changes WHEN rows are shaped and
//! nothing else, so the settled document is the same document either way.
//!
//! That law is DIFFERENTIAL, and a differential law cannot see an error its two
//! arms SHARE. The row table's ORIGIN is exactly such an error: it is the same
//! origin on both sides of the comparison, so shifting it moves both arms
//! together and the comparison goes on reporting agreement — on a wrong answer.
//! [`row_geometry_keeps_the_documents_own_origin_at_every_scroll_depth`] closes
//! that with an ABSOLUTE oracle instead — arithmetic over a fixture whose row tops
//! are known without asking the renderer — swept across scroll depth, DPI, face
//! and reach. It is the law any future attempt to shape from the VIEWPORT rather
//! than from the document's first row has to satisfy, and the reason such an
//! attempt is not a small change: see that law's own docs.
//!
//! All three of those are satisfied by a split that has stopped BUYING anything:
//! they say what it must never break, not what it is for. The last law here,
//! [`a_preview_step_owes_a_tail_exactly_where_it_defers_rows`], holds it to its
//! purpose — and it is needed because the gap between the two is a UNITS
//! mismatch. The debt was decided by a HEIGHT compare against a budget that
//! deliberately over-estimates, while the saving is a ROW question, so every
//! arrow taken near the document's END declared a debt with no deferred rows
//! behind it and paid two whole-document relayouts for it, green above
//! throughout.

use super::{headless_pipeline, view};
use crate::render::{OFFSCREEN_CULL_MARGIN_ROWS, ShapeReach};

/// How many times taller than the viewport the fixture must be before this law
/// is allowed to claim it says anything about OFF-SCREEN rows. Set well under
/// the 16x the fixture below actually reaches, so the guard fails on a fixture
/// that stopped being tall rather than on ordinary metric drift.
const MIN_OFFSCREEN_MULTIPLE: f32 = 4.0;

/// A document of `n` short, non-wrapping lines — one visual row each, at any
/// column width a page-mode window can produce — so the shaped-row COUNT is an
/// exact oracle rather than a wrap-dependent estimate.
fn tall_doc(n: usize) -> String {
    (0..n)
        .map(|i| format!("row {i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Everything this law derives from the fixture and the LIVE metrics, plus the
/// non-vacuity guards over them. Split out of the sweep because it is setup, not
/// law: nothing here asserts anything about a theme arrow.
struct Reach {
    /// Rows the window can show — derived from the live metrics rather than
    /// written down, so a zoom/DPI/menu-bar change moves every bound with it.
    viewport_rows: f32,
    /// How many viewports of document sit off-screen.
    multiple: f32,
    /// `rects::row_box_visible`'s own cull band, in rows.
    cull_rows: f32,
    /// The rows a frame is ALLOWED to paint into: the window plus the cull band
    /// on each side. The presented frame must have shaped at least these.
    paintable_rows: f32,
    /// The most a WINDOW-bounded reach ever needs: the viewport, the cull band,
    /// and one further window of slack that keeps the truncated document taller
    /// than the viewport so `max_scroll` cannot clamp the frame's own scroll.
    /// Anything materially past this is the document leaking back into the cost.
    window_bound: f32,
    /// The configuration every failure message reports (CLAUDE.md: a check runs in
    /// one configuration, and that configuration is itself an untested hypothesis).
    config: String,
}

impl Reach {
    fn measure(p: &crate::render::TextPipeline, lines: usize) -> Self {
        let viewport_rows = (p.window_h / p.metrics.line_height).max(1.0);
        let config = format!(
            "window {}x{} @dpi {} · line_height {:.1} · viewport {viewport_rows:.1} rows · \
             doc {lines} rows",
            p.window_w, p.window_h, p.dpi, p.metrics.line_height
        );
        let cull_rows = OFFSCREEN_CULL_MARGIN_ROWS.0;
        let me = Self {
            viewport_rows,
            multiple: lines as f32 / viewport_rows,
            cull_rows,
            paintable_rows: viewport_rows + 2.0 * cull_rows,
            window_bound: 3.0 * viewport_rows + 3.0 * cull_rows,
            config,
        };
        // NON-VACUITY: this law is a claim about rows the window CANNOT show. If
        // the fixture ever stops overflowing the viewport there are no such rows
        // and every assertion in the sweep would pass while proving nothing.
        assert!(
            me.multiple >= MIN_OFFSCREEN_MULTIPLE,
            "fixture no longer overflows the viewport ({:.1}x < {MIN_OFFSCREEN_MULTIPLE:.1}x): \
             this law would say nothing about off-screen rows — {}",
            me.multiple,
            me.config
        );
        assert!(
            me.window_bound < lines as f32,
            "the window bound ({:.0} rows) is not smaller than the document ({lines} rows): \
             the split cannot be observed on this fixture — {}",
            me.window_bound,
            me.config
        );
        me
    }

    /// THE PRESENT-TIME HALF of the law: the frame this arrow is about to hand the
    /// compositor has every row it is ALLOWED to paint, has no more than a
    /// window-bounded reach, and records the tail it still owes.
    fn assert_presentable_frame(
        &self,
        p: &crate::render::TextPipeline,
        from: &str,
        to: &str,
        lines: usize,
    ) {
        let (at, config) = (p.total_visual_rows(), &self.config);
        assert!(
            at as f32 >= self.paintable_rows,
            "{from} -> {to}: the frame about to be presented has {at} rows shaped but may \
             paint into {:.0} of them (window + {}-row cull margin either side) — a row \
             the renderer is allowed to draw was not shaped for it — {config}",
            self.paintable_rows,
            self.cull_rows
        );
        assert!(
            (at as f32) <= self.window_bound,
            "{from} -> {to}: the presented frame shaped {at} rows, past the {:.0} rows a \
             window-bounded reach needs ({:.1}x the viewport) — the preview step's cost \
             has gone back to tracking the DOCUMENT, which is the whole {lines}-row lag \
             this split exists to remove — {config}",
            self.window_bound,
            at as f32 / self.viewport_rows
        );
        assert!(
            p.shape_tail_owed(),
            "{from} -> {to}: {at} of {lines} rows are shaped but no tail is recorded as \
             owed — nothing would ever pay it — {config}"
        );
    }
}

/// A PACED sweep presents each world and lets its quiet settle finish before the
/// next movement.
///
/// The sweep is the picker's own: `overlay::build`'s `OverlayKind::Theme` arm
/// hands the picker `theme::THEMES` in order, so walking that roster in order IS
/// pressing Down through the card. The roster is read rather than transcribed,
/// so a twenty-first world is swept the day it is added and a world that starts
/// sharing its neighbour's face is handled by the `needs_theme_reshape` witness
/// below rather than by a name list.
#[test]
fn paced_theme_arrows_present_and_settle_every_world() {
    // The active WORLD is a process global and this law walks all of it, so take
    // the one process-wide guard — which also restores the world on the way out,
    // including the unwinding path. Declared before `p` so it outlives the
    // pipeline's GPU resources, not merely the calls that touch them
    // (CLAUDE.md's test-global rule: a `TextPipeline` dropped at the closing
    // brace still moves the shared device's counters).
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping paced_theme_arrows_present_and_settle_every_world: \
             no wgpu adapter"
        );
        return;
    };

    // The AMBIENT world, captured rather than named: the sweep below ends on
    // whatever `THEMES` ends on, and the guard requires the world it was entered
    // with. Reading it (rather than writing a default in) is the same rule that
    // keeps a `cfg!(target_os = ..)` restore from restoring the wrong value.
    let entered = crate::theme::active().name;

    const LINES: usize = 400;
    let text = tall_doc(LINES);
    p.set_view(&view(&text, 0, 0));
    let reach = Reach::measure(&p, LINES);
    let (multiple, config) = (reach.multiple, reach.config.clone());

    let mut reshaped_hops = 0usize;
    for world in crate::theme::THEMES {
        let from = crate::theme::active().name;
        crate::theme::set_active_by_name(world.name).unwrap_or_else(|| {
            panic!(
                "theme::THEMES names a world set_active_by_name rejects: {}",
                world.name
            )
        });

        // WITNESS THE WORK (CLAUDE.md: a bench/law that can silently measure
        // nothing is a defect). Ask BEFORE the switch whether this hop has a
        // reshape to do, then require the reshape counter to agree — so a hop
        // that quietly stopped reshaping fails here instead of sailing through
        // the row-count assertion on the PREVIOUS world's shaping.
        let owed = p.needs_theme_reshape();
        let before = p.reshape_count;

        // ---- the half of the step that runs BEFORE the frame is presented ----
        // Exactly what `App::retint_theme_preview` runs: colors, then the font
        // reshape at the PRESENTABLE reach. (The live App reaches the same
        // `sync_theme_font` through its timed door, which reads a clock this
        // headless path must not.)
        p.sync_theme_colors();
        p.sync_theme_font(ShapeReach::Presentable);

        let did = p.reshape_count > before;
        assert_eq!(
            owed,
            did,
            "{from} -> {}: needs_theme_reshape said {owed} but reshape_count {} \
             ({before} -> {}) — the hop and its witness disagree",
            world.name,
            if did { "moved" } else { "did not move" },
            p.reshape_count
        );
        if !did {
            // A hop with no reshape (only reachable when the sweep re-enters the
            // world it started in) narrows nothing and owes nothing.
            assert!(
                !p.shape_tail_owed(),
                "{from} -> {}: a hop that did not reshape still owes a tail",
                world.name
            );
            continue;
        }
        reshaped_hops += 1;

        // AT THE MOMENT OF THE PRESENT — the frame is drawable, and only just.
        reach.assert_presentable_frame(&p, from, world.name, LINES);

        // ---- the half that runs immediately AFTER that present, same step ----
        assert!(
            p.finish_shape_tail(),
            "{from} -> {}: the owed tail reported nothing to do — {config}",
            world.name
        );
        assert!(
            !p.shape_tail_owed(),
            "{from} -> {}: the tail was paid and is still owed",
            world.name
        );

        // THE LAW, UNCHANGED AND UNWEAKENED: the whole document is shaped at the
        // end of the step — every logical line holds a real, laid-out visual row,
        // including the ~95% of them below the fold. A reach narrowed
        // PERMANENTLY, or a tail completion that never runs, collapses this count
        // to roughly the window bound above.
        let rows = p.total_visual_rows();
        assert_eq!(
            rows, LINES,
            "{from} -> {}: one arrow's STEP ended with {rows} of {LINES} rows shaped \
             ({multiple:.1}x the viewport is off-screen and must still be shaped by the \
             time the next event is handled) — an unshaped tail carries no geometry at \
             all, which is the scroll-jump bug full_shape_height exists to prevent — \
             {config}",
            world.name
        );
    }

    // NON-VACUITY, second guard: the assertions above only say anything on a hop
    // that actually reshaped. The roster decides how many those are (every
    // consecutive pair currently differs in effective face); requiring all but the
    // one possible no-op re-entry keeps a roster that quietly stopped reshaping
    // from turning this whole sweep into a walk of `continue`s.
    assert!(
        reshaped_hops + 1 >= crate::theme::THEMES.len(),
        "only {reshaped_hops} of {} hops reshaped: this sweep asserted almost nothing — \
         {config}",
        crate::theme::THEMES.len()
    );

    // Hand the world back as it was found. The guard restores on the UNWINDING
    // path by itself, so a failure mid-sweep cannot poison another file; this
    // covers the path where every assertion held.
    crate::theme::set_active_by_name(entered);
}

/// A zero-gap burst replaces every intermediate off-screen tail and pays only
/// the final world's. The visible prefix still reshapes on every hop, so this is
/// coalescing rather than a preview debounce.
#[test]
fn zero_gap_theme_burst_coalesces_intermediate_tails_latest_selection_wins() {
    let _t = crate::testlock::serial();
    let entered = crate::theme::active().name;
    let start = crate::theme::THEMES.last().expect("theme roster").name;
    crate::theme::set_active_by_name(start);
    let Some(mut burst) = headless_pipeline() else {
        eprintln!("skipping zero_gap_theme_burst: no wgpu adapter");
        return;
    };
    let Some(mut whole) = headless_pipeline() else {
        return;
    };

    const LINES: usize = 400;
    let text = tall_doc(LINES);
    burst.set_view(&view(&text, 0, 0));
    whole.set_view(&view(&text, 0, 0));
    let reach = Reach::measure(&burst, LINES);
    let mut reshaped = 0usize;

    for world in crate::theme::THEMES {
        let from = crate::theme::active().name;
        crate::theme::set_active_by_name(world.name);
        let before = burst.reshape_count;
        burst.sync_theme_colors();
        burst.sync_theme_font(ShapeReach::Presentable);
        assert!(
            burst.reshape_count > before,
            "{from} -> {} did not reshape; the burst fixture stopped witnessing previews",
            world.name
        );
        reshaped += 1;
        reach.assert_presentable_frame(&burst, from, world.name, LINES);
        // Zero gap: deliberately do not finish here. The next world must be able
        // to replace this debt without first expanding the old one.
    }

    assert_eq!(
        reshaped,
        crate::theme::THEMES.len(),
        "every highlighted world must still shape its visible prefix"
    );
    assert!(burst.shape_tail_owed(), "the final world must own one tail");
    assert!(burst.finish_shape_tail(), "the final tail was not paid");
    assert!(
        !burst.finish_shape_tail(),
        "a burst produced more than one payable tail"
    );
    assert!(!burst.shape_tail_owed());

    // Final geometry is the same as one whole-document shape of the selected
    // world, including the off-screen rows the burst skipped along the way.
    whole.sync_theme();
    assert_eq!(burst.total_visual_rows(), whole.total_visual_rows());
    assert_eq!(
        burst.total_doc_height().to_bits(),
        whole.total_doc_height().to_bits()
    );
    for line in (0..LINES).step_by(37) {
        assert_eq!(
            burst
                .line_glyph_xs(line)
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>(),
            whole
                .line_glyph_xs(line)
                .iter()
                .map(|x| x.to_bits())
                .collect::<Vec<_>>(),
            "final selected world's line {line} differs from a whole shape"
        );
    }
    crate::theme::set_active_by_name(entered);
}

/// Commit keeps the previewed world and pays its tail; revert replaces the
/// preview with one whole shape of the original world. Neither may leave debt.
#[test]
fn theme_commit_and_revert_leave_the_document_fully_settled() {
    let _t = crate::testlock::serial();
    let entered = crate::theme::active().name;
    crate::theme::set_active_by_name("Mangrove");
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping theme_commit_and_revert_leave_the_document_fully_settled: no wgpu adapter"
        );
        return;
    };
    const LINES: usize = 400;
    p.set_view(&view(&tall_doc(LINES), 0, 0));

    crate::theme::set_active_by_name("Bombora");
    p.sync_theme_colors();
    p.sync_theme_font(ShapeReach::Presentable);
    assert!(p.shape_tail_owed(), "commit fixture did not create debt");
    p.sync_theme_font(ShapeReach::Whole); // commit: same active world, no-op
    assert!(p.finish_shape_tail(), "commit did not pay the preview tail");
    assert_eq!(p.total_visual_rows(), LINES);
    assert!(!p.shape_tail_owed());

    crate::theme::set_active_by_name("Galah");
    p.sync_theme_font(ShapeReach::Presentable);
    assert!(p.shape_tail_owed(), "revert fixture did not create debt");
    crate::theme::set_active_by_name("Bombora");
    p.sync_theme_colors();
    p.sync_theme_font(ShapeReach::Whole); // revert replaces the superseded tail
    assert!(
        !p.finish_shape_tail(),
        "revert paid the abandoned world's tail instead of replacing it"
    );
    assert_eq!(p.total_visual_rows(), LINES);
    assert!(!p.shape_tail_owed());
    crate::theme::set_active_by_name(entered);
}

/// The SETTLED document is byte-identical whichever reach its step took.
///
/// Splitting a preview step changes WHEN the off-screen rows are shaped; it must
/// change nothing about the document that results. Two pipelines walk the same
/// roster in the same order over the same fixture — one taking the split step
/// (`Presentable` + `finish_shape_tail`), one taking today's single whole-document
/// whole-document control — and at EVERY settled world their geometry is compared: the
/// visual-row count, the document height, every row's top and height, and every
/// line's real glyph X boundaries.
///
/// The glyph Xs are what make this more than a row count: they are read off the
/// shaped runs, so they differ between faces. Equality across the whole roster is
/// therefore also the statement that a split step never presents (or settles on)
/// one world's colors over another world's shaping — the outcome the split was
/// explicitly not allowed to buy.
#[test]
fn identical_settled_geometry_whichever_reach_the_step_took() {
    let _t = crate::testlock::serial();
    let Some(mut split) = headless_pipeline() else {
        eprintln!(
            "skipping identical_settled_geometry_whichever_reach_the_step_took: no wgpu adapter"
        );
        return;
    };
    let Some(mut whole) = headless_pipeline() else {
        return;
    };
    let entered = crate::theme::active().name;

    const LINES: usize = 400;
    let text = tall_doc(LINES);
    split.set_view(&view(&text, 0, 0));
    whole.set_view(&view(&text, 0, 0));

    let config = format!(
        "window {}x{} @dpi {} · line_height {:.1} · doc {LINES} rows",
        split.window_w, split.window_h, split.dpi, split.metrics.line_height
    );

    // Sample the glyph Xs rather than every line: a handful spread across the
    // document, deliberately including lines the split step's PRESENTED frame
    // could not have shaped (any line past the first couple of windows) as well as
    // ones it must have.
    let sampled: Vec<usize> = (0..LINES).step_by(37).collect();

    let mut differing_faces = 0usize;
    for world in crate::theme::THEMES {
        let from = crate::theme::active().name;
        crate::theme::set_active_by_name(world.name);

        // The split step, in order: preview reach, (present), tail.
        split.sync_theme_colors();
        split.sync_theme_font(ShapeReach::Presentable);
        split.finish_shape_tail();
        // Control: one whole-document reshape.
        whole.sync_theme();

        assert_eq!(
            split.total_visual_rows(),
            whole.total_visual_rows(),
            "{from} -> {}: split and whole steps settled on different visual-row \
             counts — {config}",
            world.name
        );
        assert_eq!(
            split.total_doc_height().to_bits(),
            whole.total_doc_height().to_bits(),
            "{from} -> {}: split {} vs whole {} document height — {config}",
            world.name,
            split.total_doc_height(),
            whole.total_doc_height()
        );
        for row in 0..whole.total_visual_rows() {
            assert_eq!(
                (
                    split.row_top_px(row).to_bits(),
                    split.row_height_px(row).to_bits()
                ),
                (
                    whole.row_top_px(row).to_bits(),
                    whole.row_height_px(row).to_bits()
                ),
                "{from} -> {}: row {row} geometry differs — split ({}, {}) vs whole ({}, {}) \
                 — {config}",
                world.name,
                split.row_top_px(row),
                split.row_height_px(row),
                whole.row_top_px(row),
                whole.row_height_px(row)
            );
        }
        let mut face_moved = false;
        for &line in &sampled {
            let a = split.line_glyph_xs(line);
            let b = whole.line_glyph_xs(line);
            assert_eq!(
                a.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                b.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
                "{from} -> {}: line {line}'s shaped glyph advances differ between a split \
                 step and a whole one — {config}",
                world.name
            );
            face_moved |= !a.is_empty();
        }
        if face_moved {
            differing_faces += 1;
        }
    }

    // NON-VACUITY: the glyph-X comparison says nothing about faces if the sampled
    // lines carry no glyphs at all.
    assert_eq!(
        differing_faces,
        crate::theme::THEMES.len(),
        "the sampled lines produced no glyph advances on {} of {} worlds — the \
         face half of this law swept nothing — {config}",
        crate::theme::THEMES.len() - differing_faces,
        crate::theme::THEMES.len()
    );

    crate::theme::set_active_by_name(entered);
}

/// The row table's ORIGIN is the DOCUMENT's first row, at every scroll depth and
/// after either reach — stated as ABSOLUTE arithmetic, not as a comparison
/// between two pipelines.
///
/// [`identical_settled_geometry_whichever_reach_the_step_took`] above is a
/// DIFFERENTIAL law: it asks whether a split step and a whole one agree. That is
/// the right question about the SPLIT, and the wrong one about the ORIGIN — any
/// error the two arms share passes it, reporting agreement on a wrong answer.
/// Shifting every `run.line_top` by one line height inside `RowGeom::ensure`
/// moves both arms identically and leaves it green.
///
/// This matters because the one remaining way to make a preview arrow cheaper at
/// DEPTH is to let cosmic-text's own `buffer.scroll` move, so `shape_until_scroll`
/// fills from the viewport instead of from the document's first row. Doing that
/// re-bases exactly what this law pins: `LayoutRunIter` starts at `scroll.line`
/// and measures `line_top` from `-scroll.vertical`, so every row above the scroll
/// leaves the table entirely and the survivors' tops are viewport-relative. The
/// damage does not announce itself — `row_top_px(0)` still answers `0.0`, which is
/// a plausible number for a different row.
///
/// So the oracle here is arithmetic over a fixture whose answer is known without
/// consulting the renderer: `tall_doc`'s lines each occupy exactly one visual row,
/// so row `r` sits at `r * line_height` and the document is `LINES * line_height`
/// tall — no matter where the viewport is, which face is active, or which reach
/// the last step took.
#[test]
fn row_geometry_keeps_the_documents_own_origin_at_every_scroll_depth() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping row_geometry_keeps_the_documents_own_origin_at_every_scroll_depth: \
             no wgpu adapter"
        );
        return;
    };
    let entered = crate::theme::active().name;

    const LINES: usize = 400;
    let text = tall_doc(LINES);

    // Both DPIs: a check runs in one configuration, and that configuration is
    // itself an untested hypothesis (CLAUDE.md). The row table is built from
    // device-pixel run tops, so the scale is an axis, not a detail.
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        // Walk faces too — the ORIGIN must be face-independent even though the
        // wrapped-row COUNT is not. `tall_doc` never wraps, so the arithmetic
        // below holds in every world and any face-dependence is a defect.
        for world in ["Mangrove", "Bombora", "Galah", "Tawny"] {
            crate::theme::set_active_by_name(world);
            let mut v = view(&text, 0, 0);
            p.set_view(&v);
            p.sync_theme();

            let line_h = p.metrics.line_height;
            let viewport = p.viewport_avail_px(p.window_h);
            let config = format!(
                "world {world} · dpi {dpi} · window {}x{} · line_height {line_h:.3} · \
                 doc {LINES} rows",
                p.window_w, p.window_h
            );
            // NON-VACUITY (a): a zero/degenerate line height makes every equality
            // below trivially true and the law would say nothing.
            assert!(
                line_h > 1.0,
                "line height {line_h} is degenerate — this law's arithmetic oracle \
                 would be vacuous — {config}"
            );
            // NON-VACUITY (b): with no scroll RANGE there is no depth axis, and the
            // one thing this law exists to catch is a depth-dependent origin.
            let doc_h = LINES as f32 * line_h;
            assert!(
                doc_h > 4.0 * viewport,
                "fixture is {doc_h:.0}px against a {viewport:.0}px viewport — there is \
                 no scroll depth to sweep — {config}"
            );

            // TOP / HALF / END, and after BOTH reaches at each depth: the reach is
            // what chooses the shaping budget, and a budget is the thing a
            // scroll-relative fill would be measured from.
            for depth in [0usize, LINES / 2, LINES - 1] {
                for reach in [ShapeReach::Whole, ShapeReach::Presentable] {
                    // Park the viewport, then take a real preview arrow FROM this
                    // depth — the sequence a user produces by scrolling and then
                    // holding Down in the theme picker.
                    v.scroll =
                        p.scroll_by_px(crate::render::ScrollPos::at_row(depth), 0.0, p.window_h);
                    p.set_view(&v);
                    let landed = v.scroll.row;
                    crate::theme::set_active_by_name(if world == "Bombora" {
                        "Mangrove"
                    } else {
                        "Bombora"
                    });
                    p.sync_theme_colors();
                    p.sync_theme_font(reach);
                    p.finish_shape_tail();

                    let at = format!("{config} · scroll row {landed} · reach {reach:?}");

                    assert_eq!(
                        p.total_visual_rows(),
                        LINES,
                        "the row table holds {} of the document's {LINES} rows — its \
                         domain is no longer the whole document — {at}",
                        p.total_visual_rows()
                    );
                    assert_eq!(
                        p.total_doc_height().to_bits(),
                        (LINES as f32 * line_h).to_bits(),
                        "document height {} where the fixture is exactly \
                         {LINES} x {line_h} = {} — {at}",
                        p.total_doc_height(),
                        LINES as f32 * line_h
                    );
                    // THE ORIGIN ITSELF. Row 0 is the DOCUMENT's first row and sits
                    // at 0.0 — not the first row the viewport can see.
                    for row in 0..LINES {
                        assert_eq!(
                            p.row_top_px(row).to_bits(),
                            (row as f32 * line_h).to_bits(),
                            "row {row} tops at {} where the document's own origin puts \
                             it at {} — the row table has been re-based — {at}",
                            p.row_top_px(row),
                            row as f32 * line_h
                        );
                    }
                    // Every LOGICAL line has real geometry, including the ones above
                    // the viewport: `UNSHAPED_LINE_TOP` here is a row that dropped out
                    // of the table, which every viewport cull silently accepts as
                    // "below everything" rather than reporting.
                    for line in 0..LINES {
                        assert!(
                            p.row_geom
                                .line_first_top(&p.buffer, &p.metrics, line)
                                .is_finite(),
                            "line {line} reports no geometry at all — it is not in the \
                             row table — {at}"
                        );
                    }
                    // The CLAMP reads the same table, and is what decides whether the
                    // frame presents at the scroll it was asked for.
                    assert_eq!(
                        p.max_scroll_rows(p.window_h),
                        LINES - crate::render::OVERSCROLL_KEEP_ROWS,
                        "max_scroll_rows collapsed to {} — the document the clamp sees \
                         is not the whole one — {at}",
                        p.max_scroll_rows(p.window_h)
                    );
                }
            }
        }
    }

    crate::theme::set_active_by_name(entered);
}

/// The depths a preview arrow is taken from. Both ENDS of the curve are the
/// claim — the top, where the split must still pay for itself, and the last row,
/// where it must cost nothing — and the middle is swept because the crossover
/// between them is where a units error hides: it produces a plausible-looking
/// curve everywhere except at the end.
fn preview_depths(lines: usize) -> [usize; 7] {
    [
        0,
        lines / 8,
        lines / 4,
        lines / 2,
        3 * lines / 4,
        7 * lines / 8,
        lines - 1,
    ]
}

/// ONE preview arrow of [`a_preview_step_owes_a_tail_exactly_where_it_defers_rows`],
/// taken from `depth` in a document already settled on the departure world:
/// scroll there, hop to `to` at the [`ShapeReach::Presentable`] reach, and assert
/// the whole of that law over the one arrow. Returns whether it DEFERRED rows, so
/// the sweep can close its own non-vacuity over the curve.
///
/// Split out of the sweep because it is the law's body rather than its axes —
/// everything here is per-arrow and reads none of the sweep's counters.
fn preview_arrow_defers(
    p: &mut crate::render::TextPipeline,
    v: &mut crate::render::ViewState,
    to: &str,
    depth: usize,
    lines: usize,
    config: &str,
) -> bool {
    // Park the viewport, then take a real preview arrow from it.
    v.scroll = p.scroll_by_px(crate::render::ScrollPos::at_row(depth), 0.0, p.window_h);
    p.set_view(v);
    let at = format!("{config} · scroll row {}/{lines}", v.scroll.row);

    // WITNESS THE WORK: a hop that quietly stopped reshaping would sail through
    // every assertion below on the previous world's shaping. Asked AFTER the world
    // moves — `needs_theme_reshape` compares the shaped face against the ACTIVE one.
    crate::theme::set_active_by_name(to);
    let must = p.needs_theme_reshape();
    let before = p.reshape_count;
    p.sync_theme_colors();
    p.sync_theme_font(ShapeReach::Presentable);
    assert!(
        must && p.reshape_count > before,
        "the arrow did not reshape (needs_theme_reshape {must}, reshape_count \
         {before} -> {}) — {at}",
        p.reshape_count
    );

    // The two halves of the step, read at the two moments that matter.
    let owed = p.shape_tail_owed();
    let at_present = p.total_visual_rows();
    p.finish_shape_tail();
    let settled = p.total_visual_rows();
    let deferred = at_present < settled;

    // THE LAW.
    assert_eq!(
        owed,
        deferred,
        "the step {} a tail while shaping {at_present} of {settled} rows before its \
         present: a debt is a claim that rows were DEFERRED, and paying one for rows \
         that were already shaped costs two whole-document relayouts for nothing — {at}",
        if owed { "owed" } else { "owed no" }
    );
    // Unweakened and restated AT DEPTH, which is where this law's own change lives:
    // the whole-document sweep above only ever runs at scroll row 0.
    assert_eq!(
        settled, lines,
        "the step ended with {settled} of {lines} rows shaped — an unshaped tail \
         carries no geometry at all — {at}"
    );
    assert!(
        !p.shape_tail_owed(),
        "the step ended still owing a tail — {at}"
    );

    // THE TWO ENDS OF THE CURVE, which are the shipped claim itself.
    if depth == 0 {
        assert!(
            deferred,
            "at the document top the arrow deferred nothing ({at_present} of {settled} \
             rows shaped before the present) — the split is the whole-document step \
             under another name — {at}"
        );
    }
    if depth == lines - 1 {
        assert!(
            !deferred && !owed,
            "at the document's last row the arrow deferred {} rows and owed {owed}: \
             the paintable band already reaches the last row here, so there is nothing \
             to defer and a debt is pure cost — {at}",
            settled - at_present
        );
    }
    deferred
}

/// A preview step declares a shaping DEBT exactly where it DEFERS rows — the two
/// are the same question, and it is a question about ROWS.
///
/// [`every_theme_arrow_shapes_the_whole_document_by_the_end_of_its_step`] above
/// pins what the split must never break. This law pins what it must never stop
/// buying, and it exists because the two are separated by a UNITS mismatch that
/// leaves every assertion in that law green.
///
/// The saving is bought by leaving rows unshaped: shaping only what the frame can
/// paint is cheaper than shaping the document exactly to the extent that some of
/// the document is not shaped. Ask instead whether the paint budget is shorter
/// than [`TextPipeline::full_shape_height`] — a HEIGHT compare against a budget
/// that deliberately over-estimates (~8 wrapped rows per logical line, plus every
/// reserved image height) — and the answer is `true` in a region where the saving
/// is exactly zero: at a scroll near the document's END the paintable band already
/// reaches the last row while sitting far under the over-estimate. A step there
/// declares a debt, pays a whole-document `set_size` to narrow and another to
/// restore (cosmic-text relayouts every laid-out line on any height change), and
/// defers nothing. Measured on a 1948-row fixture across the twenty-world roster
/// it was 20 debts, 0 deferred rows, and a per-step regression against the
/// unsplit step it replaced.
///
/// So the oracle here is the pair (`shape_tail_owed`, `rows shaped at the present
/// < rows shaped at the step boundary`), and the law is that they agree — swept
/// across scroll DEPTH, which is the axis the units mismatch is invisible on
/// anywhere but its far end, and across DPI and FACE, because the budget is built
/// out of `line_height` and the crossover depth moves with it.
///
/// THE FIXTURE CARRIES THE PREMISE. `presentable_reach_height` decides whether to
/// narrow BEFORE the reshape, from the last fully-shaped pass's own document
/// height, so the biconditional is exact only where a reshape does not change that
/// height. `tall_doc` never wraps and carries no heading or image row, so its
/// height is `LINES * line_height` in every world — asserted below rather than
/// assumed, so this law reports the death of its own premise instead of quietly
/// becoming a coin flip.
#[test]
fn a_preview_step_owes_a_tail_exactly_where_it_defers_rows() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping a_preview_step_owes_a_tail_exactly_where_it_defers_rows: no wgpu adapter"
        );
        return;
    };
    let entered = crate::theme::active().name;

    const LINES: usize = 400;
    let text = tall_doc(LINES);

    // The ARRIVAL face is swept as well as the departure one: the budget is built
    // from the metrics, and a world's face is what a preview arrow changes.
    const HOPS: [(&str, &str); 4] = [
        ("Mangrove", "Bombora"),
        ("Bombora", "Galah"),
        ("Galah", "Tawny"),
        ("Tawny", "Mangrove"),
    ];

    // How the curve came out, counted rather than assumed — the two ends are
    // asserted per sample below, and these close the sweep's own non-vacuity.
    let (mut deferring, mut whole_reach) = (0usize, 0usize);

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for (from, to) in HOPS {
            let mut v = view(&text, 0, 0);
            crate::theme::set_active_by_name(from);
            p.set_view(&v);
            // Settle onto the departure world at the WHOLE reach, so the arrow below
            // starts from a fully shaped document — the state a real picker arrow is
            // always taken from, and the state whose row table the decision reads.
            p.sync_theme();

            let line_h = p.metrics.line_height;
            let reach = Reach::measure(&p, LINES);
            let config = format!("{} · dpi {dpi} · {from} -> {to}", reach.config);

            // THE PREMISE, asserted: on this fixture the document's own height is a
            // property of the metrics alone, so "the budget covers the document"
            // means the same thing before and after the reshape.
            assert_eq!(
                p.total_doc_height().to_bits(),
                (LINES as f32 * line_h).to_bits(),
                "the fixture is {} tall where {LINES} non-wrapping rows of {line_h:.3} \
                 make {} — this law's biconditional rests on that height being \
                 face-independent — {config}",
                p.total_doc_height(),
                LINES as f32 * line_h
            );

            for depth in preview_depths(LINES) {
                // Back to the departure world, settled and WHOLE — the previous
                // depth's arrow left the pipeline in `to`, and a hop to the world it
                // is already in reshapes nothing.
                crate::theme::set_active_by_name(from);
                p.sync_theme();
                if preview_arrow_defers(&mut p, &mut v, to, depth, LINES, &config) {
                    deferring += 1;
                } else {
                    whole_reach += 1;
                }
            }
        }
    }

    // NON-VACUITY over the sweep as a whole: this law is a claim about a CURVE, and
    // a sweep that landed entirely on one side of it would satisfy every assertion
    // above while pinning only one of the two states.
    let samples = 2 * HOPS.len() * preview_depths(LINES).len();
    assert_eq!(
        deferring + whole_reach,
        samples,
        "{} of {samples} arrows were counted",
        deferring + whole_reach
    );
    assert!(
        deferring > 0 && whole_reach > 0,
        "the depth sweep produced {deferring} deferring arrows and {whole_reach} \
         whole-reach ones — it never crossed the curve this law is about"
    );

    crate::theme::set_active_by_name(entered);
}
