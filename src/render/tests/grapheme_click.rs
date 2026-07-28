//! CLICK-TO-CARET over multi-scalar grapheme clusters — the pointer half of the
//! cluster work, driven through the REAL shaped hit test
//! ([`TextPipeline::hit_test_scroll`] -> `col_in_run`) on every world, so a
//! PROPORTIONAL face's advances are what the assertions run on. A fixed-pitch
//! check would pass under either implementation and prove nothing.
//!
//! The measured defect these laws name: a shaper's glyph clusters are not UAX #29
//! clusters. Thai `ก` + `ำ` (U+0E33 SARA AM) is ONE cluster that every shipping
//! world's face shapes as TWO glyph groups, and a click in the middle of it
//! resolved to the char column BETWEEN the consonant and its vowel sign — a
//! caret position that does not exist on screen. Devanagari conjuncts split the
//! same way on IBM Plex Mono.

use super::super::*;
use super::{headless_pipeline, view};
use crate::grapheme::{CLUSTER_CORPUS, boundaries_of};

/// Every world, so no face's shaping is exempt. Returns `(name, is_mono)`.
fn worlds() -> Vec<(&'static str, bool)> {
    theme::THEMES
        .iter()
        .map(|t| (t.name, crate::render::facepitch::family_is_mono(t.font)))
        .collect()
}

/// The x range each of a line's clusters occupies, as
/// `(start_col, end_col, ink_left, ink_right)` — read from the SHAPED run's own
/// glyphs, so it is the ink the reader actually sees rather than a nominal cell
/// grid. Clusters of one char are skipped: they have no interior to land in.
fn cluster_inks(p: &TextPipeline, text: &str) -> Vec<(usize, usize, f32, f32)> {
    let bounds = boundaries_of(text);
    let byte_of = |col: usize| -> usize {
        text.char_indices()
            .nth(col)
            .map(|(b, _)| b)
            .unwrap_or(text.len())
    };
    let mut out = vec![];
    for w in bounds.windows(2) {
        let (s, e) = (w[0], w[1]);
        if e - s < 2 {
            continue;
        }
        let (first, last) = (byte_of(s), byte_of(e));
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for run in p.buffer.layout_runs() {
            if run.line_i != 0 {
                continue;
            }
            for g in run
                .glyphs
                .iter()
                .filter(|g| g.start >= first && g.end <= last)
            {
                lo = lo.min(g.x);
                hi = hi.max(g.x + g.w);
            }
        }
        if lo < hi {
            out.push((s, e, lo, hi));
        }
    }
    out
}

/// THE CLICK LAW: sweeping the pointer across a line's full width, at every
/// world, no x resolves the caret to a position interior to a grapheme cluster —
/// asserted both on the render's own answer and on the composed document
/// resolution ([`crate::buffer::Buffer::hit_char`], which is what a real press
/// calls through `App::hit_test_char`).
///
/// Non-vacuity is asserted inside the sweep, not assumed: for every multi-char
/// cluster on the line, SOME x must resolve to its start and SOME x to its end,
/// so a hit test that answered a constant, or a sweep that stepped over the
/// cluster entirely, fails here rather than passing quietly.
#[test]
fn a_click_never_lands_inside_a_grapheme_cluster_on_any_world() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping a_click_never_lands_inside_a_grapheme_cluster_on_any_world: no wgpu");
        return;
    };
    let mut proportional = 0;
    for (world, mono) in worlds() {
        theme::set_active_by_name(world);
        p.sync_theme();
        if !mono {
            proportional += 1;
        }
        for (label, text) in CLUSTER_CORPUS.iter().copied() {
            let bounds = boundaries_of(text);
            let buf = crate::buffer::Buffer::from_str(&format!("{text}\n"));
            p.set_view(&view(&format!("{text}\n"), 1, 0));
            let py = p.doc_top() + p.metrics.line_height * 0.5;
            let left = p.text_left();
            let mut hit = std::collections::BTreeSet::new();
            for i in 0..600 {
                let px = left + i as f32 * 0.5;
                let (line, col) = p.hit_test_scroll(px, py, ScrollPos::default());
                assert!(
                    bounds.contains(&col),
                    "{world}/{label}: x=+{:.1} resolves to col {col}, inside a cluster of \
                     {text:?} (boundaries {bounds:?})",
                    i as f32 * 0.5
                );
                let idx = buf.hit_char(line, col);
                assert!(
                    bounds.contains(&idx),
                    "{world}/{label}: x=+{:.1} resolves to char {idx}, inside a cluster",
                    i as f32 * 0.5
                );
                hit.insert(col);
            }
            for (s, e, _, _) in cluster_inks(&p, text) {
                assert!(
                    hit.contains(&s) && hit.contains(&e),
                    "{world}/{label}: the sweep never reached both sides of the cluster \
                     ({s},{e}) — it proves nothing about it (reached {hit:?})"
                );
            }
        }
    }
    assert!(
        proportional >= 8,
        "the sweep must cover proportional faces, not only mono ones: {proportional}"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// THE SIDE LAW — what makes the snap NEAREST rather than merely legal: a
/// pointer in the LEFT half of a cluster's ink puts the caret at its START, and
/// one in the RIGHT half at its END. Probed at 20% / 40% and 60% / 80% of each
/// multi-char cluster's measured ink, on every world.
///
/// This is the assertion that fails on the reported bug: at 60–80% across the
/// Thai cluster the un-snapped walk answers the interior column, and at 20–40%
/// across a Devanagari conjunct on IBM Plex Mono it answers the interior column
/// too — and a snap that always went FORWARD would fail the left-half probes.
#[test]
fn a_click_in_a_clusters_left_half_lands_at_its_start_right_half_at_its_end() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping a_click_in_a_clusters_left_half_lands_at_its_start...: no wgpu");
        return;
    };
    let mut probed = 0;
    for (world, _) in worlds() {
        theme::set_active_by_name(world);
        p.sync_theme();
        for (label, text) in CLUSTER_CORPUS.iter().copied() {
            p.set_view(&view(&format!("{text}\n"), 1, 0));
            let py = p.doc_top() + p.metrics.line_height * 0.5;
            let left = p.text_left();
            for (s, e, lo, hi) in cluster_inks(&p, text) {
                for (frac, want, side) in [
                    (0.2, s, "start"),
                    (0.4, s, "start"),
                    (0.6, e, "end"),
                    (0.8, e, "end"),
                ] {
                    let x = lo + (hi - lo) * frac;
                    let (_line, col) = p.hit_test_scroll(left + x, py, ScrollPos::default());
                    assert_eq!(
                        col,
                        want,
                        "{world}/{label}: a click at {:.0}% of the cluster ({s},{e}) ink \
                         [{lo:.2},{hi:.2}] must land on its {side}",
                        frac * 100.0
                    );
                    probed += 1;
                }
            }
        }
    }
    assert!(
        probed > 500,
        "the side law must actually probe clusters on every world: {probed} probes"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// THE MONOTONICITY LAW: moving the pointer RIGHTWARD across a line never moves
/// the caret leftward. The axis nobody thinks to sweep — every individual answer
/// can be a legal cluster boundary while the sequence jumps back and forth, which
/// is exactly what a per-glyph rule did on a cluster shaped as several glyphs:
/// `a😀\u{200d}😀b` answered start, end, start, end as x advanced, so clicking the
/// right half of the emoji sequence put the caret BEFORE it.
#[test]
fn dragging_the_pointer_rightward_never_moves_the_caret_left() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping dragging_the_pointer_rightward_never_moves_the_caret_left: no wgpu");
        return;
    };
    for (world, _) in worlds() {
        theme::set_active_by_name(world);
        p.sync_theme();
        for (label, text) in CLUSTER_CORPUS.iter().copied() {
            p.set_view(&view(&format!("{text}\n"), 1, 0));
            let py = p.doc_top() + p.metrics.line_height * 0.5;
            let left = p.text_left();
            let mut last = 0usize;
            let mut last_x = 0.0f32;
            for i in 0..600 {
                let x = i as f32 * 0.5;
                let (_line, col) = p.hit_test_scroll(left + x, py, ScrollPos::default());
                assert!(
                    col >= last,
                    "{world}/{label}: x=+{x:.1} answers col {col} after x=+{last_x:.1} \
                     answered {last} — the caret moved LEFT as the pointer moved right \
                     in {text:?}"
                );
                last = col;
                last_x = x;
            }
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}
