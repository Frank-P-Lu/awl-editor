//! ITEM 181 — THE HEIGHT-CLAMP LAW.
//!
//! The defect: the GROUPED/faceted geometry (`theme_overlay_geometry`) divided
//! its own available pixels by the row pitch to bound its item window; the
//! FLAT geometry (`overlay_geometry`) capped its window only at a per-kind row
//! COUNT (`OverlayKind::window_rows`) that knows nothing about the canvas. A
//! flat picker whose count is its whole corpus — the theme picker,
//! `window_rows() == theme::THEMES.len()`, once its runtime lens strip retired
//! (making it flat, `docs/render.md`'s "Overlay personality" section) — drew a
//! card taller than the canvas at its own default size (`card_h: 934` against
//! `canvas_h: 800`, reproduced verbatim below before this law existed).
//!
//! `render::plan::fit_item_rows` is now the one clamp owner both families
//! read (see its doc in `render/plan/overlay_rows.rs`). This file is the
//! device-level proof, over the WHOLE `OverlayKind` roster (classified by the
//! SAME no-wildcard match production already uses, `facets::scheme` —
//! `overlay_plan_law.rs`'s own hand-copied `family()` classifier drifted from
//! it: it calls `Assets` grouped, but `facets::scheme(OverlayKind::Assets)` is
//! `None`, so production actually routes it through the FLAT path. Reusing the
//! real function instead of a second hand-maintained copy is exactly the
//! "same behavior ⇒ same code" a hand-copied classifier keeps failing at).
//!
//! TWO INDEPENDENT ORACLES per swept cell, plus a third pixel-level check:
//!  - STATE — the sidecar's own `card_h`/`canvas_h` (`overlay_window_report`).
//!  - GEOMETRY — the planner's OWN last row's bottom edge
//!    (`OverlayRowPlan::rows().last()`), derived by a DIFFERENT function
//!    (`plan_overlay_rows`'s private `row_top`) than the one `card_h` itself
//!    comes from (`TextPipeline::overlay_card_h`) — so the two cannot agree
//!    merely because they read the same arithmetic twice. Mutating either
//!    owner independently would move exactly one of the two numbers.
//!  - APPEARANCE — the rendered PNG's own pixels at the last row's baseline
//!    actually carry glyph ink, arithmetic over bytes rather than inferred
//!    from either of the above (CAPTURE.md's tripwire: the sidecar is a state
//!    oracle, not an appearance oracle — it once reported a selected row while
//!    that row rendered fully invisible).
//!
//! Swept over the whole `OverlayKind` roster, both list styles, four LOGICAL
//! window sizes at 1x/2x DPI (canvases are `logical * dpi` in PHYSICAL pixels,
//! so every cell is a genuinely reachable live window — never a physical
//! canvas that DPI happens to shrink below `app::lifecycle`'s own enforced
//! minimum, `MIN_COLS`x`MIN_LINES` — see the module doc below on the swept
//! bound), and four points across the documented zoom range (0.5..3.0).

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Flat,
    Grouped,
    Contextual,
}

/// Classify a kind by the SAME no-wildcard match production reads
/// (`facets::scheme`), not a second hand-maintained copy — seeing exactly why
/// this file exists in the first place (its module doc above).
fn family(kind: OverlayKind) -> Family {
    if kind == OverlayKind::Spell {
        return Family::Contextual;
    }
    if crate::facets::scheme(kind).is_some() {
        return Family::Grouped;
    }
    Family::Flat
}

/// A realistic view for `kind`: a corpus bigger than any shipped picker's own
/// window cap (`kind.window_rows()`), with the REAL per-kind cap set (not the
/// `ViewState::base()` default of 12) — the theme picker's own 19-world cap is
/// the shape this law exists to sweep, and a fixture that quietly re-defaulted
/// it to 12 would never reproduce the regression.
///
/// `sectioned` chooses, for a GROUPED kind only, between the picker's default
/// summon state (the "All" home lens — no sections, `facet_lens == 0`,
/// `crate::overlay::facet`'s own "All is HOME" convention) and a lens the user
/// has cycled to (real section headers). A grouped picker spends the vast
/// majority of its open time on "All"; `sectioned` lets a caller choose which
/// shape it wants rather than always paying the extra header overhead.
fn overlay_view(kind: OverlayKind, n: usize, sectioned: bool) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_window_rows = kind.window_rows();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = n.saturating_sub(1);
    v.overlay_hint = "type to filter".into();
    match family(kind) {
        Family::Grouped if sectioned => {
            v.overlay_lens = vec![
                ("All".into(), false),
                ("File".into(), true),
                ("Edit".into(), false),
            ];
            v.overlay_sections = (0..n)
                .map(|i| match i * 3 / n.max(1) {
                    0 => "Alpha".to_string(),
                    1 => "Beta".to_string(),
                    _ => "Gamma".to_string(),
                })
                .collect();
        }
        Family::Grouped => {
            // The "All" home: every grouped picker's own summon default, no
            // section headers — the shape most of a grouped picker's open time
            // is actually spent in.
            v.overlay_lens = vec![("All".into(), true), ("File".into(), false)];
        }
        Family::Contextual => {
            let cap = kind.window_rows().min(n).max(1);
            v.overlay_spell = Some((0, 0, 5));
            v.overlay_items = (0..cap).map(|i| format!("suggest{i}")).collect();
            v.overlay_bindings = Vec::new();
            v.overlay_selected = cap - 1;
            v.overlay_hint = String::new();
        }
        Family::Flat => {}
    }
    v
}

/// Render offscreen and read the pixels back (the same small readback dance
/// `overlay_plan_law.rs`'s own `shoot` uses).
fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item181 height-clamp encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

/// Mean luminance of the pixel rows `[y0, y1)` across x `[x0, x1)`. `None` for
/// a degenerate (empty or off-canvas) band.
fn mean_luma(
    pixels: &[[u8; 4]],
    canvas_w: u32,
    canvas_h: u32,
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
) -> Option<f64> {
    let (xa, xb) = (x0.max(0.0) as u32, (x1.min(canvas_w as f32)) as u32);
    let (ya, yb) = (y0.max(0.0) as u32, (y1.min(canvas_h as f32)) as u32);
    if xb <= xa || yb <= ya {
        return None;
    }
    let mut sum = 0.0f64;
    let mut n = 0u32;
    for y in ya..yb {
        for x in xa..xb {
            let px = pixels[(y * canvas_w + x) as usize];
            sum += 0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64;
            n += 1;
        }
    }
    (n > 0).then_some(sum / n as f64)
}

/// LOGICAL window sizes (pre-DPI) the sweep uses — the same roster
/// `overlay_plan_law.rs` already swears by (a roomy canvas, a narrow one that
/// forces the edge-inset floor, a short one that stresses the height clamp,
/// and a tall one), all comfortably above `app::lifecycle`'s own enforced
/// minimum window (`MIN_COLS * CHAR_WIDTH + 2*TEXT_LEFT` x
/// `MIN_LINES * LINE_HEIGHT + 2*TEXT_TOP` = 464x288 logical) at either DPI.
const LOGICAL_CANVASES: [(u32, u32); 4] = [(1200, 800), (700, 800), (900, 460), (1400, 1600)];

const ZOOMS: [f32; 4] = [0.5, 1.0, 2.0, 3.0];

/// Grades one swept cell of `no_card_exceeds_its_canvas_for_any_overlay_kind`:
/// the STATE oracle (`card_h <= canvas_h`), the GEOMETRY oracle (the planner's
/// OWN last row bottom — a different function than the one `card_h` came
/// from), and the APPEARANCE oracle (real pixel ink at the last candidate
/// row's baseline). Returns `(clamp engaged here, appearance oracle graded
/// something)` for the caller's non-vacuity floors.
fn grade_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    kind: OverlayKind,
    n: usize,
    cw: u32,
    ch: u32,
) -> (bool, bool) {
    let v = overlay_view(kind, n, true);
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).unwrap();
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let ctx = format!("{kind:?} n={n} canvas={cw}x{ch}");

    let (_top, _lines, _sel, card_h, canvas_h) = p
        .overlay_window_report()
        .unwrap_or_else(|| panic!("{ctx}: an open card must report a window"));
    assert!(
        card_h <= canvas_h + 0.01,
        "{ctx}: card_h {card_h} exceeds canvas_h {canvas_h}"
    );

    if let Some(last) = plan.rows().last() {
        assert!(
            last.bottom() <= canvas_h + 0.01,
            "{ctx}: the planner's own last row bottom {} exceeds canvas_h {canvas_h} even \
             though the sidecar claims card_h {card_h} fits — the two owners disagree",
            last.bottom()
        );
    }

    let engaged = n > kind.window_rows()
        && plan.rows().iter().filter(|r| r.item.is_some()).count() < kind.window_rows().min(n);

    let mut ink_ok = false;
    if let Some(last_item_row) = plan.rows().iter().rev().find(|r| r.item.is_some()) {
        let pixels = shoot(device, queue, p, cw, ch);
        let x0 = geom.text_left;
        let x1 = geom.text_left + geom.text_w;
        let ground = mean_luma(
            &pixels,
            cw,
            ch,
            x0,
            x1,
            geom.card_y + 1.0,
            geom.card_y + 9.0,
        );
        let pad = (last_item_row.height * 0.2).max(1.0);
        let row_band = mean_luma(
            &pixels,
            cw,
            ch,
            x0,
            x1,
            last_item_row.top + pad,
            last_item_row.bottom() - pad,
        );
        if let (Some(g), Some(r)) = (ground, row_band) {
            ink_ok = true;
            assert!(
                (r - g).abs() > 6.0,
                "{ctx}: the last candidate row (display {}) shows no ink distinguishable \
                 from the card's own ground (ground {g:.2}, row {r:.2}) — a row the \
                 state/geometry oracles call ON-CANVAS may still be blank or clipped in \
                 the actual pixels",
                last_item_row.display
            );
        }
    }
    (engaged, ink_ok)
}

/// THE HEADLINE LAW — every `OverlayKind`, both list styles, every logical
/// canvas at 1x/2x DPI (physical = logical * dpi, so the window is always a
/// genuinely reachable one), at zoom 1.0. A separate, deeper sweep below adds
/// the zoom axis against the two largest-corpus representatives (the theme
/// picker's own 19-row cap is the shape that regressed; a big Grouped picker
/// is its already-clamped sibling), since the clamp arithmetic itself is
/// identical code regardless of kind — only the corpus size and the per-kind
/// cap differ.
#[test]
fn no_card_exceeds_its_canvas_for_any_overlay_kind() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping no_card_exceeds_its_canvas_for_any_overlay_kind: no wgpu adapter");
        return;
    };

    let styles: [Option<theme::ListStyle>; 2] = [
        Some(theme::ListStyle::Pane),
        Some(theme::ListStyle::Bars {
            radius: 6.0,
            gap: 8.0,
            grow_px: 24.0,
            extent: theme::BarExtent::FullWidth,
            coverage: theme::BarCoverage::All,
        }),
    ];

    let mut clamp_engaged = 0usize; // NON-VACUITY: the clamp must actually bind somewhere.
    let mut ink_checked = 0usize;
    for style in styles {
        crate::render::set_list_style_test_override(style);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for (lw, lh) in LOGICAL_CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for kind in OverlayKind::ALL {
                    let n = kind.window_rows() + 25; // well past every per-kind cap
                    let (engaged, ink_ok) = grade_cell(&device, &queue, &mut p, kind, n, cw, ch);
                    clamp_engaged += engaged as usize;
                    ink_checked += ink_ok as usize;
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    p.set_dpi(1.0);

    assert!(
        clamp_engaged > 0,
        "the height clamp never actually engaged anywhere in the sweep — this law would \
         pass just as well with the clamp deleted"
    );
    assert!(
        ink_checked > 100,
        "the appearance oracle graded too few rows ({ink_checked}) to mean anything"
    );
}

/// THE ZOOM AXIS, against the two largest-corpus representatives: the theme
/// picker (the reported regression, a FLAT card whose per-kind cap is its
/// whole world roster) and the command palette on its "All" home lens (an
/// already-clamped GROUPED card, its established sibling, in the state a
/// grouped picker spends the vast majority of its open time in). The clamp
/// arithmetic is identical code for every kind — only the corpus and the
/// per-kind cap differ — so stressing it here across the documented zoom
/// range (0.5..3.0) at both DPIs, on every logical canvas, covers the axis the
/// headline law (fixed at zoom 1.0) cannot: the query-beat/row-pitch growth
/// that only zoom drives.
///
/// NOT SWEPT HERE: a grouped picker deliberately CYCLED onto a sectioned lens
/// (`sectioned: true`) carries extra fixed header overhead this item's clamp
/// does not touch, and at the documented zoom ceiling combined with a short
/// canvas that overhead alone can still exceed the canvas — a real,
/// reproducible, PRE-EXISTING gap (present on `main` before this item, in the
/// grouped family's own already-established clamp, not introduced by it) in
/// the chrome-overhead sizing itself, not the item-row count this clamp
/// owns. Out of scope here; flagged in the landing report rather than folded
/// silently into this law's assertion.
#[test]
fn no_card_exceeds_its_canvas_across_the_documented_zoom_range() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping no_card_exceeds_its_canvas_across_the_documented_zoom_range: no wgpu adapter"
        );
        return;
    };

    let mut checked = 0usize;
    for kind in [OverlayKind::Theme, OverlayKind::Command] {
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for (lw, lh) in LOGICAL_CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for &zoom in &ZOOMS {
                    let mut v = overlay_view(kind, kind.window_rows() + 25, false);
                    v.zoom = zoom;
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let (_t, _l, _s, card_h, canvas_h) = p
                        .overlay_window_report()
                        .expect("an open card must report a window");
                    checked += 1;
                    assert!(
                        card_h <= canvas_h + 0.01,
                        "{kind:?} dpi={dpi} logical={lw}x{lh} zoom={zoom}: card_h {card_h} \
                         exceeds canvas_h {canvas_h}"
                    );
                }
            }
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 64,
        "the zoom sweep graded too few cells: {checked}"
    );
}

/// THE EXACT REPORTED REGRESSION, pinned by name and by number: the theme
/// picker at 1200x800 (dpi 1.0), at the SAME sticky zoom (1.5) the reporting
/// capture's own config carried, used to report `card_h: 934.00006` against
/// `canvas_h: 800` verbatim (19 world rows, no canvas awareness at all in the
/// flat path). This is the smallest possible non-vacuity witness — deleting
/// `fit_item_rows`'s clamp (or routing the flat path around it) reproduces
/// this exact failure, watched red in the report.
#[test]
fn the_reported_theme_picker_regression_stays_fixed() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_reported_theme_picker_regression_stays_fixed: no wgpu adapter");
        return;
    };
    let mut v = overlay_view(OverlayKind::Theme, crate::theme::THEMES.len(), false);
    v.zoom = 1.5;
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let (_top, lines, sel, card_h, canvas_h) = p
        .overlay_window_report()
        .expect("the theme picker must report a window");
    assert!(
        card_h <= canvas_h,
        "the theme picker's default card_h {card_h} exceeds its canvas_h {canvas_h} — \
         this is the item 181 regression itself, reproduced verbatim"
    );
    assert!(sel < lines, "the selected world must stay on screen");
}
