//! THEME-PREVIEW SHAPE REACH — how much of the document ONE theme-picker arrow
//! re-shapes, WHEN inside its own step it does so, and therefore where that step's
//! dominant cost lives.
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
//! So the reach is not narrowed — it is SPLIT ACROSS ONE STEP
//! ([`crate::render::ShapeReach`]). A preview arrow shapes what the frame it is
//! about to present can paint, that frame presents, and the off-screen tail is
//! shaped immediately afterwards, inside the same event handler, before any
//! further input can be delivered. Total work per step is unchanged; only its
//! order within the step moved.
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
}

/// EVERY arrow of a full theme-picker sweep presents a frame shaped only as far
/// as that frame can paint, and leaves the WHOLE document shaped by the end of
/// its own step.
///
/// The sweep is the picker's own: `overlay::build`'s `OverlayKind::Theme` arm
/// hands the picker `theme::THEMES` in order, so walking that roster in order IS
/// pressing Down through the card. The roster is read rather than transcribed,
/// so a twenty-first world is swept the day it is added and a world that starts
/// sharing its neighbour's face is handled by the `needs_theme_reshape` witness
/// below rather than by a name list.
#[test]
fn every_theme_arrow_shapes_the_whole_document_by_the_end_of_its_step() {
    // The active WORLD is a process global and this law walks all of it, so take
    // the one process-wide guard — which also restores the world on the way out,
    // including the unwinding path. Declared before `p` so it outlives the
    // pipeline's GPU resources, not merely the calls that touch them
    // (CLAUDE.md's test-global rule: a `TextPipeline` dropped at the closing
    // brace still moves the shared device's counters).
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping every_theme_arrow_shapes_the_whole_document_by_the_end_of_its_step: \
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
    let Reach {
        viewport_rows,
        multiple,
        cull_rows,
        paintable_rows,
        window_bound,
        config,
    } = Reach::measure(&p, LINES);

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
        let at_present = p.total_visual_rows();
        assert!(
            at_present as f32 >= paintable_rows,
            "{from} -> {}: the frame about to be presented has {at_present} rows shaped \
             but may paint into {paintable_rows:.0} of them (window + \
             {cull_rows}-row cull margin either side) — a row the \
             renderer is allowed to draw was not shaped for it — {config}",
            world.name
        );
        assert!(
            (at_present as f32) <= window_bound,
            "{from} -> {}: the presented frame shaped {at_present} rows, past the \
             {window_bound:.0} rows a window-bounded reach needs ({:.1}x the viewport) — \
             the preview step's cost has gone back to tracking the DOCUMENT, which is \
             the whole {LINES}-row lag this split exists to remove — {config}",
            world.name,
            at_present as f32 / viewport_rows
        );
        assert!(
            p.shape_tail_owed(),
            "{from} -> {}: {at_present} of {LINES} rows are shaped but no tail is \
             recorded as owed — nothing would ever pay it — {config}",
            world.name
        );

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

/// The SETTLED document is byte-identical whichever reach its step took.
///
/// Splitting a preview step changes WHEN the off-screen rows are shaped; it must
/// change nothing about the document that results. Two pipelines walk the same
/// roster in the same order over the same fixture — one taking the split step
/// (`Presentable` + `finish_shape_tail`), one taking today's single whole-document
/// step — and at EVERY settled world their whole shaped geometry is compared: the
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
        // Today's step: one whole-document reshape.
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
