//! THEME-PREVIEW SHAPE REACH — how much of the document ONE theme-picker arrow
//! re-shapes, and therefore where that step's dominant cost lives.
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
//! fixtures above, ~95% of the shaped rows are off-screen. That reach is
//! deliberate (an unshaped tail falls back to `RowGeom`'s ESTIMATED line height,
//! which is the scroll-jump / wrong-image-height bug `full_shape_height`'s own
//! doc records), so this law does not argue with it. It PINS it, and it states
//! the off-screen multiple in its failure message, so that narrowing the reach
//! to buy back the 17–20 ms is a conscious, named decision that has to answer
//! for the geometry it stops computing — rather than a silent edit that reads
//! green here and shows up as a scroll jump on a long document.
//!
//! The oracle is a ROW COUNT, not a duration: it is deterministic, it holds on
//! any backend and at any profile, and it moves for exactly the reason the cost
//! moves. A timing assertion could do none of that.

use super::{headless_pipeline, view};

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

/// EVERY arrow of a full theme-picker sweep re-shapes EVERY row of the
/// document, not just the rows the window can show.
///
/// The sweep is the picker's own: `overlay::build`'s `OverlayKind::Theme` arm
/// hands the picker `theme::THEMES` in order, so walking that roster in order IS
/// pressing Down through the card. The roster is read rather than transcribed,
/// so a twenty-first world is swept the day it is added and a world that starts
/// sharing its neighbour's face is handled by the `needs_theme_reshape` witness
/// below rather than by a name list.
#[test]
fn every_theme_arrow_shapes_the_whole_document_not_just_the_viewport() {
    // The active WORLD is a process global and this law walks all of it, so take
    // the one process-wide guard — which also restores the world on the way out,
    // including the unwinding path. Declared before `p` so it outlives the
    // pipeline's GPU resources, not merely the calls that touch them
    // (CLAUDE.md's test-global rule: a `TextPipeline` dropped at the closing
    // brace still moves the shared device's counters).
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping every_theme_arrow_shapes_the_whole_document_not_just_the_viewport: \
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

    // REPORT THE CONFIGURATION THIS RAN IN (CLAUDE.md: a check runs in one
    // configuration, and that configuration is itself an untested hypothesis).
    // The viewport's row capacity is derived from the live metrics rather than
    // written down, so a zoom/DPI/menu-bar change moves the guard with it.
    let viewport_rows = (p.window_h / p.metrics.line_height).max(1.0);
    let config = format!(
        "window {}x{} @dpi {} · line_height {:.1} · viewport {:.1} rows · doc {LINES} rows",
        p.window_w, p.window_h, p.dpi, p.metrics.line_height, viewport_rows
    );

    // NON-VACUITY: this law is a claim about rows the window CANNOT show. If the
    // fixture ever stops overflowing the viewport there are no such rows and
    // every assertion below would pass while proving nothing.
    let multiple = LINES as f32 / viewport_rows;
    assert!(
        multiple >= MIN_OFFSCREEN_MULTIPLE,
        "fixture no longer overflows the viewport ({multiple:.1}x < \
         {MIN_OFFSCREEN_MULTIPLE:.1}x): this law would say nothing about off-screen rows — {config}"
    );

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
        p.sync_theme();
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

        // THE LAW: the whole document is shaped after the hop — every logical
        // line holds a real, laid-out visual row, including the ~95% of them
        // below the fold. A reach narrowed to the viewport collapses this count
        // to roughly `viewport_rows`.
        let rows = p.total_visual_rows();
        assert_eq!(
            rows, LINES,
            "{from} -> {}: one arrow left {rows} of {LINES} rows shaped \
             ({:.1}x the viewport is off-screen and must still be shaped) — \
             an unshaped tail carries RowGeom's ESTIMATED height, which is the \
             scroll-jump bug full_shape_height exists to prevent — {config}",
            world.name, multiple
        );
    }

    // Hand the world back as it was found. The guard restores on the UNWINDING
    // path by itself, so a failure mid-sweep cannot poison another file; this
    // covers the path where every assertion held.
    crate::theme::set_active_by_name(entered);
}
