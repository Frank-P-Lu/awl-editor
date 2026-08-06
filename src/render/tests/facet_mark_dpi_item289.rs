//! ITEM 289 — `FacetStyle::Text` and `ChipVariant::Underline` drew their
//! active-lens mark's THICKNESS in raw device pixels while every other term of
//! the same rect (position, span) crossed the `Logical` -> `Metrics::px`
//! boundary. Item 242 named this class and gave it one rule — author in
//! logical units, multiply once at the boundary — and these two dials never
//! got the memo: on a 2x panel the mark's x/y/width doubled and its height
//! stayed put, so the mark rendered at half its tuned weight on every Retina
//! display. `--capture-dpi 1` is the one scale every ordinary capture runs at,
//! and it is the one scale at which the bug is invisible — so every claim here
//! sweeps 1x AND 2x explicitly.
//!
//! Two claims:
//!   1. THE DRAWN RESULT — the actual rendered ink band, measured in PNG-frame
//!      pixels (not a computed length), is roughly twice as thick at 2x as at
//!      1x on a canvas that is the SAME LOGICAL size at both tiers (the
//!      `chrome_pixel_space_item242` discipline — a canvas held at one
//!      physical size across DPI shrinks its LOGICAL content and reflows the
//!      strip, which is a different bug's shape than the one under test).
//!      Grading only `overlay_theme_underline`'s recorded rect would stay
//!      green through a draw call that quietly ignored it.
//!   2. THE ROSTER SWEEP — every world in `theme::THEMES` is queried for its
//!      OWN `render_caps.facet_style` (never a named world), sorted into the
//!      two enrolled arms (`Text`, `Chips(Underline)`) plus everything else,
//!      and the enrolled arms must show the doubling while an UNENROLLED arm
//!      (a style this item's diff never touches) is shown to already double
//!      correctly today — proof the fix is scoped to exactly its two dials,
//!      not a symptom the wider mechanism also needed. `Chips(Bracket)` draws
//!      no rect at all (its mark is a set of corner ticks, not a bar), so it
//!      is recorded but excluded from the rect-thickness arithmetic rather
//!      than forced through a check its own shape cannot answer.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

const LOGICAL: (f32, f32) = (1200.0, 800.0);

fn facet_view() -> ViewState {
    let scheme = crate::facets::scheme(crate::overlay::OverlayKind::Command)
        .expect("the command palette owns a facet roster to probe with");
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_items = (0..8).map(|i| format!("Command {i}")).collect();
    v.overlay_selected = 1;
    v.overlay_lens = scheme.strip_labels(0);
    v
}

/// A larger `zoom` for the DRAWN-pixel claim only: `Metrics::scale` is
/// `zoom * dpi`, so a bigger zoom draws every dial (including the 1.5-3.5px
/// marks under test) several times larger in device pixels, which shrinks
/// anti-aliasing's roughly-fixed-width halo relative to the quantity being
/// measured. The geometry claim needs no such boost — it reads the exact
/// value fed to the draw call — but a pixel-count estimator over a raw 1.5px
/// stroke is dominated by AA noise at zoom 1.
fn facet_view_zoomed() -> ViewState {
    let mut v = facet_view();
    v.zoom = 3.0;
    v
}

/// The active-lens mark rect the shaper recorded, at `dpi`, on a canvas whose
/// LOGICAL size is fixed at [`LOGICAL`] — so the strip's own responsive fold
/// makes the same layout decision at every tier and only the device
/// resolution changes. `None` if the surface drew no mark at all (a style
/// whose mark is not a rect, or a broken view). Returns the physical `(w, h)`
/// alongside so a caller can render the matching frame.
fn mark_rect(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    v: &ViewState,
    dpi: f32,
) -> (u32, u32, Option<[f32; 4]>) {
    let (w, h) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    p.set_view(v);
    let rect = p
        .prepare(device, queue, w, h)
        .ok()
        .and(p.overlay_theme_underline);
    (w, h, rect)
}

/// The mark's DRAWN thickness in actual frame pixels, as a COVERAGE INTEGRAL
/// rather than a binary row count. A rasterizer's anti-aliasing kernel has a
/// roughly FIXED width in device pixels, so it adds a near-constant halo to
/// every stroke regardless of scale — negligible against a thick quantity
/// (item 242's card-edge inset, tens of px) but not against a stroke only a
/// few px thick, where a binary "does this row differ from ground" threshold
/// over- or under-counts the halo differently at each tier and never
/// converges on a clean 2x ratio. Summing each row's fractional COVERAGE
/// between the background color and a sampled full-ink color instead recovers
/// the stroke's true extent (the halo's partial-coverage rows contribute
/// less than 1 each, and their total is conserved by antialiasing almost
/// exactly), which is the standard antialiasing-invariant width estimator.
///
/// Sampled across several columns spanning the middle half of the rect's
/// width (never just one), so a single stray glyph pixel bleeding into one
/// column cannot skew the estimate. The background is read from BELOW the
/// mark at `dpi`-scaled clearance, never above it: the strip's own label sits
/// directly above the mark by design (`UNDERLINE_BASELINE_DROP` is a couple
/// of logical pixels), so a fixed device-pixel offset upward lands on the
/// label's own glyph ink at 1x and on clean ground at 2x — the same fixed
/// offset reading two different things at the two tiers this law compares.
/// Independent of the geometry claim on purpose — see the module doc's
/// non-vacuity note in `chrome_pixel_space_item242`.
fn drawn_thickness_px(frame: &[[u8; 4]], w: u32, h: u32, rect: [f32; 4], dpi: f32) -> Option<f32> {
    let n = 5usize;
    let lo = rect[0] + rect[2] * 0.25;
    let span = rect[2] * 0.5;
    let xs: Vec<u32> = (0..n)
        .map(|i| lo + span * (i as f32 / (n - 1) as f32))
        .filter(|x| *x >= 0.0 && (*x as u32) < w)
        .map(|x| x as u32)
        .collect();
    if xs.len() < n {
        return None;
    }
    let clear = 20.0 * dpi; // well clear of the mark, short of the next row
    let bg_y = ((rect[1] + rect[3] + clear) as u32).min(h.saturating_sub(1));
    // Tight around the recorded rect only: the strip's own label sits a few
    // px above it (`UNDERLINE_BASELINE_DROP`), close enough that a wider
    // window pulls in glyph ink whose per-column values vary (unlike the
    // mark's own uniform fill), corrupting the "darkest row" ink estimate
    // below.
    let y0 = (rect[1] - 2.0).max(0.0) as u32;
    let y1 = ((rect[1] + rect[3] + 4.0) as u32).min(h);
    if y1 <= y0 {
        return None;
    }
    let avg = |y: u32| -> [f32; 3] {
        let mut sum = [0.0f32; 3];
        for &x in &xs {
            let px = frame[(y * w + x) as usize];
            for c in 0..3 {
                sum[c] += px[c] as f32;
            }
        }
        [
            sum[0] / xs.len() as f32,
            sum[1] / xs.len() as f32,
            sum[2] / xs.len() as f32,
        ]
    };
    let bg = avg(bg_y);
    // Full-ink reference: the darkest (lowest-luminance) row sampled in the
    // scan window — the mark's own fill color, wherever it peaks.
    let ink = (y0..y1)
        .map(avg)
        .min_by(|a, b| (a[0] + a[1] + a[2]).total_cmp(&(b[0] + b[1] + b[2])))?;
    let denom: f32 = (0..3).map(|c| (ink[c] - bg[c]).powi(2)).sum();
    if denom < 1.0 {
        return None; // no usable contrast between ground and ink
    }
    let coverage = |y: u32| -> f32 {
        let px = avg(y);
        let dot: f32 = (0..3).map(|c| (px[c] - bg[c]) * (ink[c] - bg[c])).sum();
        (dot / denom).clamp(0.0, 1.0)
    };
    Some((y0..y1).map(coverage).sum())
}

/// **CLAIM 1 — THE DRAWN RESULT.** For both dials this item fixes, the
/// rendered ink band is about twice as many pixels tall on a 2x panel as it is
/// on a 1x one. This is the headline: it grades what a viewer actually sees,
/// not the rect the shaper merely recorded.
#[test]
fn the_marks_drawn_thickness_doubles_on_a_two_x_panel() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!("skipping the drawn facet-mark law: no wgpu adapter");
        return;
    };
    let v = facet_view_zoomed();
    for (label, style) in [
        ("FacetStyle::Text", theme::FacetStyle::Text),
        (
            "ChipVariant::Underline",
            theme::FacetStyle::Chips(theme::ChipVariant::Underline),
        ),
    ] {
        set_facet_style_test_override(Some(style));
        let mut got = Vec::new();
        for dpi in [1.0f32, 2.0f32] {
            let (w, h, rect) = mark_rect(&device, &queue, &mut p, &v, dpi);
            let rect = rect
                .unwrap_or_else(|| panic!("{label}: no active-lens mark recorded at dpi {dpi}"));
            let frame = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let px = drawn_thickness_px(&frame, w, h, rect, dpi).unwrap_or_else(|| {
                panic!("{label}: mark rect fell outside the frame at dpi {dpi}")
            });
            got.push(px);
        }
        p.set_dpi(1.0);
        let (t1, t2) = (got[0], got[1]);
        // NON-VACUITY: a mark that never draws would compare 0 against 0.
        assert!(
            t1 >= 1.0,
            "{label}: the drawn mark is only {t1}px thick at dpi 1 — comparing \
             it against dpi 2 would pass on a mark that isn't there"
        );
        let want = 2.0 * t1;
        let tol = 1.5 + 0.15 * want; // sub-pixel estimator, generous on a thin stroke
        assert!(
            (t2 - want).abs() <= tol,
            "{label}: the drawn mark measures {t1:.2}px thick at dpi 1 and \
             {t2:.2}px at dpi 2, where a mark whose thickness crossed the \
             pixel-space boundary would draw at {want:.2}px (tolerance \
             {tol:.2}). A thickness left in device pixels renders at half its \
             tuned weight on every Retina display."
        );
    }
    set_facet_style_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
    p.set_size(LOGICAL.0, LOGICAL.1);
}

/// **CLAIM 2 — THE ROSTER SWEEP.** Enrollment is derived from
/// `theme::THEMES`' own `render_caps.facet_style`, never a named world, so a
/// future world that picks up `Text` or `Chips(Underline)` is swept the day it
/// lands rather than the day someone remembers to add it here. Every enrolled
/// world must show the doubling; the unenrolled `Chips(Hairline)` /
/// `Chips(FilledActive)` / `Band` arm — this item's diff never touches those
/// branches — is checked too, to prove the fix is scoped: they must ALREADY
/// scale correctly, both before and after this change. `Chips(Bracket)` draws
/// corner ticks, not a rect, so it is recorded and excluded from the
/// arithmetic rather than forced through a check its own shape can't answer.
#[test]
fn every_enrolled_world_scales_its_facet_mark_by_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!("skipping the facet-mark roster sweep: no wgpu adapter");
        return;
    };
    let v = facet_view();
    let mut text_worlds: Vec<&str> = Vec::new();
    let mut underline_worlds: Vec<&str> = Vec::new();
    let mut other_worlds: Vec<&str> = Vec::new();
    let mut no_rect_worlds: Vec<&str> = Vec::new();
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).expect("theme roster name resolves");
        p.sync_theme();
        let style = theme::active().render_caps.facet_style;
        let (_, _, rect1) = mark_rect(&device, &queue, &mut p, &v, 1.0);
        let (_, _, rect2) = mark_rect(&device, &queue, &mut p, &v, 2.0);
        p.set_dpi(1.0);
        let Some(rect1) = rect1 else {
            // A style whose mark is not a rect (`Chips(Bracket)`'s corner
            // ticks). Every `Logical`-typed tick length already crosses the
            // pixel-space boundary (enforced roster-wide by
            // `chrome_pixel_space_item242`'s declaration sweep), so there is
            // nothing this law can additionally grade here — record it and
            // move on rather than fail a check the style structurally can't
            // answer.
            no_rect_worlds.push(world);
            continue;
        };
        let rect2 =
            rect2.unwrap_or_else(|| panic!("{world}: drew a mark rect at dpi 1 but not dpi 2"));
        let (t1, t2) = (rect1[3], rect2[3]);
        assert!(
            t1 >= 1.0,
            "{world} ({style:?}): the recorded mark thickness is only {t1}px \
             at dpi 1 — a zero-thickness mark would pass any doubling check"
        );
        match style {
            theme::FacetStyle::Text => {
                text_worlds.push(world);
                assert!(
                    (t2 - 2.0 * t1).abs() <= 0.05,
                    "{world}: FacetStyle::Text mark thickness is {t1}px at dpi \
                     1 and {t2}px at dpi 2 — every other term of this rect \
                     doubles, and the thickness must too"
                );
                // BYTE-IDENTITY AT THE CAPTURE DPI: the tuned dpi-1 look this
                // item must not disturb.
                assert!(
                    (t1 - 1.5).abs() < 1e-4,
                    "{world}: the dpi-1 Text mark thickness moved from its \
                     tuned 1.5px to {t1}px — this item must not change the \
                     look every ordinary --capture-dpi 1 capture already saw"
                );
            }
            theme::FacetStyle::Chips(theme::ChipVariant::Underline) => {
                underline_worlds.push(world);
                assert!(
                    (t2 - 2.0 * t1).abs() <= 0.05,
                    "{world}: ChipVariant::Underline mark thickness is {t1}px \
                     at dpi 1 and {t2}px at dpi 2 — every other term of this \
                     rect doubles, and the thickness must too"
                );
                assert!(
                    (t1 - 3.5).abs() < 1e-4,
                    "{world}: the dpi-1 Underline chip thickness moved from \
                     its tuned 3.5px to {t1}px — this item must not change \
                     the look every ordinary --capture-dpi 1 capture already \
                     saw"
                );
            }
            _ => {
                other_worlds.push(world);
                // OUT OF SCOPE, PROVEN SCOPED: this item's diff never touches
                // Band or the other two rect-drawing Chips variants, and
                // their thickness is `chip_h` — already derived from
                // `metrics.line_height`, so it already crosses the
                // pixel-space boundary. It must therefore ALREADY double,
                // with or without this item's fix.
                assert!(
                    (t2 - 2.0 * t1).abs() <= 0.05,
                    "{world} ({style:?}): an OUT-OF-SCOPE facet mark measures \
                     {t1}px at dpi 1 and {t2}px at dpi 2 — expected it to \
                     already scale correctly (this item's diff never touches \
                     this style's branch); a mismatch here means the roster \
                     enrollment above is wrong, not that this style needs \
                     the same fix"
                );
            }
        }
    }
    // NAME WHAT ENROLLED, and refuse to pass on a sweep that found nothing —
    // the item's own count is 14 `FacetStyle::Text` worlds; derived here from
    // the live roster rather than pinned to that number's source.
    assert!(
        text_worlds.len() >= 14,
        "enrolled FacetStyle::Text worlds ({}): {text_worlds:?} — expected at \
         least 14 per item 289's own count",
        text_worlds.len()
    );
    assert!(
        !underline_worlds.is_empty(),
        "no world enrolled ChipVariant::Underline — the roster sweep found \
         nothing to grade for the item's second dial"
    );
    assert!(
        !other_worlds.is_empty(),
        "no world enrolled a rect-drawing out-of-scope facet style — the \
         scoping half of this law never ran"
    );
    eprintln!(
        "facet mark dpi sweep: Text={text_worlds:?} Underline={underline_worlds:?} \
         other={other_worlds:?} no_rect={no_rect_worlds:?}"
    );
    theme::set_active(theme::DEFAULT_THEME);
    p.set_dpi(1.0);
    p.set_size(LOGICAL.0, LOGICAL.1);
}
