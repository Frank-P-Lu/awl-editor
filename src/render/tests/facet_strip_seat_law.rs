//! THE ACTIVE-LENS MARK'S SEAT: `overlay_shape_theme` (`theme_picker.rs`)
//! computes every mark rect — underline, pill, tab, brackets, and the
//! ghost/tab-plate collections — from the strip's own shaped glyphs, and
//! `overlay_lens_at` (`docked_facet.rs`) hit-tests the same strip. Both used
//! to add the glyph's x to `geom.text_left` unconditionally, but the emitter
//! actually seats the head band's `TextArea` (query line + strip line) at
//! [`TextPipeline::overlay_head_left`] (`diagonal/offband.rs`) — the card's
//! text edge on an UPRIGHT world, but right-aligned to the text column on an
//! ASCENDING diagonal cluster (Magpie). `geom.text_left` is only that
//! function's own upright-world answer, not a second reading of it: on any
//! BANDED composition (a diagonal cluster, a split row lane) the two
//! diverge, and the mark drew — and the strip could be clicked — left of
//! where the label actually renders.
//!
//! The law below reads the label's shaped glyph span independently (off the
//! raw `panel_buffer` run, never through `overlay_shape_theme`'s own math)
//! and combines it with the ONE seat owner every consumer must now share,
//! then checks the PRODUCED mark rect and hit-test against that combination
//! — swept over every world in the roster, both upright and banded, so an
//! enrolment that only ever sees upright worlds (the shape of the original
//! miss) cannot pass here by construction.

use super::super::*;
use super::{headless_dqp, view};

const LOGICAL: (f32, f32) = (1200.0, 800.0);

/// A faceted Go-to-shaped card: several lenses, one active NOT at index 0 (so
/// a seat mistake shows up as a real x offset rather than a coincidental
/// zero), and enough candidate rows that a `Diagonal` world's row cluster —
/// and therefore its banding — actually resolves.
fn strip_view(active: usize) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "go to".to_string();
    v.overlay_items = vec![
        "src/first_document.md".into(),
        "src/second_document.md".into(),
        "src/third_document.md".into(),
    ];
    v.overlay_selected = 0;
    v.overlay_lens = ["All", "Files", "Headings", "Folders", "Recent"]
        .iter()
        .enumerate()
        .map(|(i, l)| (l.to_string(), i == active))
        .collect();
    v
}

/// The strip line's own byte ranges, one per label — the SAME concatenation
/// `overlay_shape_theme` builds (a leading `"\n"`, `chrome::strip_gap()`
/// between labels) so a label's glyphs can be found on `panel_buffer`'s
/// strip line. Bookkeeping only: which bytes belong to which label is
/// untouched by this item's fix, so re-deriving it here does not smuggle
/// the fix's own arithmetic into the oracle below.
fn strip_label_ranges(labels: &[(String, bool)]) -> Vec<std::ops::Range<usize>> {
    let mut text = String::from("\n");
    let mut ranges = Vec::with_capacity(labels.len());
    for (idx, (label, _)) in labels.iter().enumerate() {
        if idx > 0 {
            text.push_str(chrome::strip_gap());
        }
        let start = text.len();
        text.push_str(label);
        ranges.push(start..text.len());
    }
    ranges
}

/// A label's raw shaped glyph x-span on `panel_buffer`'s line 1 — buffer-local
/// (no seat added), `None` if it shaped no glyphs there.
fn glyph_span(buf: &GlyphBuffer, range: &std::ops::Range<usize>) -> Option<(f32, f32)> {
    let (a, b) = (range.start.saturating_sub(1), range.end.saturating_sub(1));
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    for run in buf.layout_runs() {
        if run.line_i != 1 {
            continue;
        }
        for g in run.glyphs.iter() {
            if g.start >= a && g.start < b {
                min_x = min_x.min(g.x);
                max_x = max_x.max(g.x + g.w);
            }
        }
    }
    (max_x > min_x).then_some((min_x, max_x))
}

/// Padded-pill styles (`Band`, the `Chips` variants other than `Underline`,
/// `DockedTab`) draw wider than the bare glyphs by design — a lateral pad a
/// few px either side. This is generous against every padding value those
/// skins actually use (`CHIP_HPAD` doubled, the bracket tick length) while
/// staying far tighter than a seat mistake, which on a banded world is the
/// row cluster's own ink width or more — asserted as a real, non-vacuous gap
/// below rather than assumed.
const PAD_SLACK: f32 = 24.0;

/// Containment with `PAD_SLACK` room either side — the mark must reach at
/// least the label's own glyphs, and not overshoot by more than a pad.
fn contains_with_slack(mark: [f32; 4], lo: f32, hi: f32) -> bool {
    let (mx0, mx1) = (mark[0], mark[0] + mark[2]);
    mx0 <= lo + 0.5 && mx1 >= hi - 0.5 && mx0 >= lo - PAD_SLACK && mx1 <= hi + PAD_SLACK
}

/// **CLAIM 1 — THE MARK'S SEAT.** For every world in the roster, the active
/// lens's recorded mark rect sits under its own shaped glyphs at the seat the
/// emitter actually draws them at — `overlay_head_left`, not the card's plain
/// text edge. `Text` and `Chips(Underline)` (no lateral pad) are held to
/// exact alignment; the padded pill/tab skins to containment within
/// `PAD_SLACK`; `Chips(Bracket)` (ticks, no single rect) is checked against
/// its own ghost collection's outer bounds. Enrollment (upright vs. banded)
/// is read from [`TextPipeline::overlay_panel_bands`] — the same gate the
/// emitter itself uses to decide whether the strip needs a second seat at
/// all — never a named world, and both arms must be non-empty or the sweep
/// proves nothing about the axis it exists to cover.
#[test]
fn active_lens_mark_sits_under_the_active_labels_own_glyphs() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!("skipping the facet-strip seat law: no wgpu adapter");
        return;
    };
    let (w, h) = (LOGICAL.0 as u32, LOGICAL.1 as u32);
    let v = strip_view(2); // "Headings" active, matching the reported repro's shape

    let mut upright_worlds: Vec<&str> = Vec::new();
    let mut banded_worlds: Vec<&str> = Vec::new();
    let mut seat_deltas: Vec<(&str, f32)> = Vec::new();

    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).expect("theme roster name resolves");
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();

        let geom = p.overlay_geometry(w);
        let plan = p.overlay_row_plan(&geom);
        let banded = p.overlay_panel_bands(&geom, &plan).is_some();
        if banded {
            banded_worlds.push(world);
        } else {
            upright_worlds.push(world);
        }

        let seat = p.overlay_head_left(&geom, &plan);
        seat_deltas.push((world, seat - geom.text_left));

        let ranges = strip_label_ranges(&v.overlay_lens);
        let (label, _) = &v.overlay_lens[2];
        let range = &ranges[2];
        let Some((min_x, max_x)) = glyph_span(&p.panel_buffer, range) else {
            panic!("{world}: the active label {label:?} shaped no glyphs on the strip line");
        };
        let (want_lo, want_hi) = (seat + min_x, seat + max_x);

        let style = theme::active().render_caps.facet_style;
        match style {
            theme::FacetStyle::Text | theme::FacetStyle::Chips(theme::ChipVariant::Underline) => {
                let rect = p.overlay_theme_underline.unwrap_or_else(|| {
                    panic!("{world} ({style:?}, banded={banded}): no active-lens mark recorded")
                });
                let (mx0, mx1) = (rect[0], rect[0] + rect[2]);
                assert!(
                    (mx0 - want_lo).abs() <= 0.5 && (mx1 - want_hi).abs() <= 0.5,
                    "{world} ({style:?}, banded={banded}): the active mark spans \
                     [{mx0:.1}, {mx1:.1}] but {label:?}'s own shaped glyphs sit at \
                     [{want_lo:.1}, {want_hi:.1}] (seat {seat:.1} vs. the card's \
                     plain text edge {:.1}) — the mark drew off the label",
                    geom.text_left
                );
            }
            theme::FacetStyle::Chips(theme::ChipVariant::Bracket) => {
                assert!(
                    p.overlay_theme_underline.is_none(),
                    "{world}: Chips(Bracket) is a tick skin and should record no rect"
                );
                let ghosts = &p.overlay_theme_facet_ghosts;
                assert!(
                    !ghosts.is_empty(),
                    "{world} ({style:?}, banded={banded}): no corner ticks recorded for the \
                     active lens"
                );
                let gx0 = ghosts.iter().map(|r| r[0]).fold(f32::INFINITY, f32::min);
                let gx1 = ghosts
                    .iter()
                    .map(|r| r[0] + r[2])
                    .fold(f32::NEG_INFINITY, f32::max);
                assert!(
                    contains_with_slack([gx0, 0.0, gx1 - gx0, 0.0], want_lo, want_hi),
                    "{world} ({style:?}, banded={banded}): the corner ticks span \
                     [{gx0:.1}, {gx1:.1}] but {label:?}'s own shaped glyphs sit at \
                     [{want_lo:.1}, {want_hi:.1}] — the ticks framed the wrong seat"
                );
            }
            _ => {
                let rect = p.overlay_theme_underline.unwrap_or_else(|| {
                    panic!("{world} ({style:?}, banded={banded}): no active-lens mark recorded")
                });
                assert!(
                    contains_with_slack(rect, want_lo, want_hi),
                    "{world} ({style:?}, banded={banded}): the active mark is \
                     [{:.1}, {:.1}] but {label:?}'s own shaped glyphs sit at \
                     [{want_lo:.1}, {want_hi:.1}] — the mark did not sit under the label",
                    rect[0],
                    rect[0] + rect[2]
                );
            }
        }
    }

    assert!(
        !upright_worlds.is_empty(),
        "no upright world enrolled — the sweep proves nothing about that arm"
    );
    assert!(
        !banded_worlds.is_empty(),
        "no banded world enrolled (diagonal cluster / split row lane) — this is exactly the \
         axis the original bug lived on, and a sweep that never engages it cannot catch a \
         regression of it"
    );
    // NON-VACUITY OF THE BANDED ARM: at least one banded world's real seat must
    // actually differ from the card's plain text edge, or `PAD_SLACK`-bounded
    // containment above could pass by coincidence rather than by the seat being
    // read correctly.
    let max_delta = seat_deltas
        .iter()
        .map(|(_, d)| d.abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_delta > PAD_SLACK,
        "every enrolled world's seat ({seat_deltas:?}) sits within {PAD_SLACK}px of the card's \
         plain text edge — no world in the roster actually exercises the banded seat, so this \
         law cannot distinguish a correct read from `geom.text_left`"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}

/// **CLAIM 2 — THE HIT-TEST.** `overlay_lens_at` must resolve a click to the
/// SAME label the mark law above just verified is drawn there — every label
/// on the strip, not only the active one, and on both upright and banded
/// worlds. A click at each label's own shaped midpoint (seat + glyph
/// midpoint, read independently as above) must resolve to that label's own
/// strip index; the skin can never disagree with where a label is drawn.
#[test]
fn overlay_lens_at_hit_tests_the_same_seat_the_mark_draws_at() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!("skipping the facet-strip hit-test seat law: no wgpu adapter");
        return;
    };
    let (w, h) = (LOGICAL.0 as u32, LOGICAL.1 as u32);
    let v = strip_view(2); // "Headings" active

    let mut banded_worlds: Vec<&str> = Vec::new();
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).expect("theme roster name resolves");
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();

        let geom = p.overlay_geometry(w);
        let plan = p.overlay_row_plan(&geom);
        let banded = p.overlay_panel_bands(&geom, &plan).is_some();
        if banded {
            banded_worlds.push(world);
        }
        let docked = p.docked_facet_band(&geom, &plan);
        let seat = if docked.is_some() {
            geom.text_left
        } else {
            p.overlay_head_left(&geom, &plan)
        };
        let strip = docked
            .or_else(|| plan.strip_band())
            .unwrap_or_else(|| panic!("{world}: a faceted card plans a strip line"));
        let py = strip.center();

        let ranges = strip_label_ranges(&v.overlay_lens);
        for (idx, (label, _active)) in v.overlay_lens.iter().enumerate() {
            let Some((min_x, max_x)) = glyph_span(&p.panel_buffer, &ranges[idx]) else {
                panic!("{world}: label {label:?} (index {idx}) shaped no glyphs");
            };
            let px = seat + (min_x + max_x) * 0.5;
            assert_eq!(
                p.overlay_lens_at(px, py),
                Some(idx),
                "{world} (banded={banded}): a click at the shaped midpoint of \
                 {label:?} (index {idx}), x={px:.1}, resolved to a different lens \
                 than the one drawn there"
            );
        }
    }
    assert!(
        !banded_worlds.is_empty(),
        "no banded world enrolled — the hit-test sweep never engaged the axis the original \
         bug lived on"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
}
