//! THE HEIGHT-CLAMP LAW.
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
//!
//! The GROUPED family's own arm of this clamp was still
//! incomplete at the zoom ceiling: a picker cycled onto a sectioned lens
//! carries extra fixed header overhead (the lens strip + real section
//! headers) this clamp did not shrink, and at zoom 3.0 on a short canvas that
//! overhead ALONE — before a single item is counted — could still exceed
//! `avail_px` (`card_h: 535.4` against `canvas_h: 460`, the command palette
//! on its File lens, confirmed present on unmodified pre-184 code via `git
//! stash`. `fit_item_rows` gained a
//! `min_items` parameter (its own doc): the FLAT family and the spell popup
//! keep `min_items: 1` (byte-identical), while the GROUPED family now passes
//! `min_items: 0` — when its fixed chrome overhead alone cannot fit, the card
//! answers with an empty candidate band rather than a forced row that
//! overruns the canvas. A no-op everywhere the floor did not bind, proven —
//! not merely asserted — by
//! `already_fitting_grouped_pickers_stay_byte_identical_across_the_floor_fix`.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Flat,
    Grouped,
    Contextual,
    /// The SUMMONED WORKSPACE family. Its box comes from the canvas rather than
    /// from a width cap, so its clamp is bounded by a
    /// different budget; it is swept here for the same reason the other three
    /// are, not exempted.
    Workspace,
}

/// Classify a kind by the SAME no-wildcard match production reads
/// (`facets::scheme`), not a second hand-maintained copy — seeing exactly why
/// this file exists in the first place (its module doc above).
fn family(kind: OverlayKind) -> Family {
    if kind == OverlayKind::Spell {
        return Family::Contextual;
    }
    // Asked BEFORE the faceting scheme, exactly as
    // `overlay_geometry` asks it: a workspace's rail IS its facet strip stood on
    // its end, so a kind that facets AND is a workspace is presented as the
    // latter. Classifying it as Grouped here would have this law measuring a
    // card production no longer draws.
    if kind.workspace_shape().is_some() {
        return Family::Workspace;
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
        Family::Workspace => {
            // The rail's data is the same lens strip; `overlay_workspace` is what
            // routes it to the workspace geometry, exactly as `sync_view` sets it.
            v.overlay_workspace = true;
            v.overlay_lens = vec![
                ("All".into(), true),
                ("Editor".into(), false),
                ("Appearance".into(), false),
            ];
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
        label: Some("awl overlay-height-clamp encoder"),
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

    let styles: [Option<theme::ListStyle>; 2] =
        [Some(theme::ListStyle::Pane), Some(theme::ListStyle::Bars)];
    // Harmless for the `Pane` arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));

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
    crate::render::set_bar_config_test_override(None);
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

/// THE ZOOM AXIS, against the three largest-corpus representatives: the theme
/// picker (the reported regression, a FLAT card whose per-kind cap is its
/// whole world roster), the command palette on its "All" home lens (an
/// already-clamped GROUPED card, its established sibling, in the state a
/// grouped picker spends the vast majority of its open time in), and — ITEM
/// 184 — the command palette CYCLED onto a sectioned lens, the shape whose
/// extra fixed header overhead (`sectioned: true`
/// below). The clamp arithmetic is identical code for every kind — only the
/// corpus and the per-kind cap differ — so stressing it here across the
/// documented zoom range (0.5..3.0) at both DPIs, on every logical canvas,
/// covers the axis the headline law (fixed at zoom 1.0) cannot: the
/// query-beat/row-pitch growth that only zoom drives.
///
/// The grouped-family arm closes the gap this test once carved out ("NOT SWEPT HERE"):
/// at the documented zoom ceiling on a short canvas the sectioned grouped
/// card's own fixed overhead (query line + lens strip + the section headers
/// its window carries) can still exceed `avail_px` before a single item is
/// counted — `fit_item_rows`'s `min_items: 0` floor for the grouped family
/// (see its doc) means the card answers with an EMPTY candidate band rather
/// than a forced row that overruns the canvas. `zero_rows_engaged` is the
/// NON-VACUITY floor for that arm: without it, this test could pass just as
/// well if the sectioned case never actually reached the pathological
/// corner.
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
    let mut zero_rows_engaged = 0usize;
    for (kind, sectioned) in [
        (OverlayKind::Theme, false),
        (OverlayKind::Command, false),
        (OverlayKind::Command, true),
    ] {
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for (lw, lh) in LOGICAL_CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for &zoom in &ZOOMS {
                    let mut v = overlay_view(kind, kind.window_rows() + 25, sectioned);
                    v.zoom = zoom;
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let geom = p.overlay_geometry(cw);
                    let plan = p.overlay_row_plan(&geom);
                    let (_t, _l, _s, card_h, canvas_h) = p
                        .overlay_window_report()
                        .expect("an open card must report a window");
                    checked += 1;
                    if plan.candidate_rows() == 0 {
                        zero_rows_engaged += 1;
                    }
                    assert!(
                        card_h <= canvas_h + 0.01,
                        "{kind:?} sectioned={sectioned} dpi={dpi} logical={lw}x{lh} zoom={zoom}: \
                         card_h {card_h} exceeds canvas_h {canvas_h}"
                    );
                }
            }
        }
    }
    p.set_dpi(1.0);
    assert!(
        checked >= 96,
        "the zoom sweep graded too few cells: {checked}"
    );
    assert!(
        zero_rows_engaged > 0,
        "the sectioned grouped arm never actually reached the pathological corner (empty \
         candidate band) — this law would pass just as well with `min_items: 0` \
         floor deleted"
    );
}

/// **THE FLOOR'S OTHER HALF — an empty candidate band must be FORCED.** The law
/// above proves the sectioned card never overruns its canvas; on its own, that
/// is satisfiable by a card that shows nothing at all. The grouped family's
/// `min_items: 0`
/// is licensed only where no row can fit; a card that plans zero rows while the
/// canvas beneath it still has room for a section header and an item row is not
/// degrading, it is mis-billing its own budget.
///
/// **THE DEFECT THIS NAMES:** the grouped family charged a header row for every
/// section in the LIST rather than for the window it was about to draw. At a
/// 900x460 canvas with the menu bar's vertical reserve taken out of the budget,
/// seven display lines fit, four went to the query line, the lens strip, the
/// hint and its separator, and three more were billed to sections the window
/// had no room to reach — leaving zero item rows in a card 192px tall inside a
/// 460px canvas, with 180px of it free below the card.
///
/// **SWEPT OVER THE MENU BAR** for the same reason `overlay_plan_law`'s
/// headline law is: `menubar::MENU_BAR_ON` starts `false` on macOS and `true`
/// everywhere else, so a sweep that never enters the second state cannot see a
/// starvation the reserve is what triggers.
///
/// **AND OVER THE LIST STYLE, because the ROW PITCH is the other half of the
/// starvation** — measured, not assumed. Written without this axis the law was
/// green under its own mutation: the ambient world's pitch is 27.2px at the
/// short canvas where `Bars` (whose `grow_px`/`gap` are part of `overlay_lh`)
/// gives 35.2px, and four rows still fit at the smaller pitch. A law about a
/// row budget has to sweep what sets the row height.
#[test]
fn an_empty_candidate_band_is_forced_by_the_canvas_never_by_its_own_header_bill() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping an_empty_candidate_band_is_forced_by_the_canvas: no wgpu adapter");
        return;
    };
    // The AMBIENT value, never `cfg!(target_os = ...)`: a `cfg!` here reports
    // the host that COMPILED the test, not the branch the initialiser took.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    // The same forced bar geometry `overlay_plan_law`'s headline law pins, so
    // the `Bars` arm carries a real plate gap and grow rather than whatever the
    // ambient world happens to author.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    let (mut checked, mut empty, mut tight) = (0usize, 0usize, 0usize);
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for style in [theme::ListStyle::Pane, theme::ListStyle::Bars] {
            crate::render::set_list_style_test_override(Some(style));
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                for (lw, lh) in LOGICAL_CANVASES {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for &zoom in &ZOOMS {
                        let mut v = overlay_view(OverlayKind::Command, 24, true);
                        v.zoom = zoom;
                        p.set_view(&v);
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let geom = p.overlay_geometry(cw);
                        let plan = p.overlay_row_plan(&geom);
                        let ctx = format!(
                            "Command/sectioned dpi={dpi} logical={lw}x{lh} zoom={zoom} \
                         menu_bar={bar} list={style:?}"
                        );
                        checked += 1;
                        let rows = plan.candidate_rows();
                        if rows <= 2 {
                            tight += 1;
                        }
                        if rows > 0 {
                            continue;
                        }
                        empty += 1;
                        // The canvas left under the drawn card, LESS the bottom
                        // margin the card contractually never spends, must not fit
                        // the two display lines an empty band is missing — one
                        // section header and one item row.
                        let free =
                            ch as f32 - (geom.card_y + geom.card_h) - p.overlay_card_margin();
                        let need = 2.0 * p.overlay_lh();
                        assert!(
                            free < need,
                            "{ctx}: the card plans ZERO candidate rows while {free}px of spendable \
                         canvas sits free below it — a section header plus an item row need \
                         {need}px, so this band was emptied by its own header bill, not by the \
                         canvas"
                        );
                    }
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    p.set_dpi(1.0);
    assert!(checked >= 128, "the sweep graded too few cells: {checked}");
    // NON-VACUITY, both ends. Without an empty band the assertion body never
    // runs at all; without a TIGHT-but-populated cell the sweep only ever
    // grades roomy cards, which are not where the floor lives.
    assert!(
        empty > 0,
        "no swept cell ever reached an empty candidate band, so the floor above was never \
         evaluated — this law would pass with the degradation arm deleted"
    );
    assert!(
        tight > 0,
        "no swept cell landed within two candidate rows of empty, so the sweep never came \
         near the floor it grades"
    );
    eprintln!(
        "empty-band floor: {checked} cells, {empty} empty bands, {tight} at two rows or fewer"
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
         this is the height-clamp regression itself, reproduced verbatim"
    );
    assert!(sel < lines, "the selected world must stay on screen");
}

/// THE GROUPED-PALETTE REGRESSION: the command palette
/// cycled onto a sectioned lens, 900x460, zoom 3.0 — used to report `card_h:
/// 535.4` against `canvas_h: 460` verbatim (measured on the unmodified
/// prior code. After the `min_items: 0` floor for the grouped
/// family, the same view reports `card_h: 372.2`, an EMPTY candidate band
/// (`plan.candidate_rows() == 0`) rather than the one forced, overrunning row
/// the old `min_items: 1` floor demanded. This is the smallest possible
/// non-vacuity witness for this item — reverting the grouped `min_items` to
/// `1` reproduces the exact reported failure, watched red in the report.
#[test]
fn the_900x460_zoom3_sectioned_command_case_now_fits() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_900x460_zoom3_sectioned_command_case_now_fits: no wgpu adapter");
        return;
    };
    let mut v = overlay_view(
        OverlayKind::Command,
        OverlayKind::Command.window_rows() + 25,
        true,
    );
    v.zoom = 3.0;
    p.set_size(900.0, 460.0);
    p.set_view(&v);
    p.prepare(&device, &queue, 900, 460).unwrap();
    let geom = p.overlay_geometry(900);
    let plan = p.overlay_row_plan(&geom);
    let (_top, lines, sel_row, card_h, canvas_h) = p
        .overlay_window_report()
        .expect("the command palette must report a window");
    assert!(
        card_h <= canvas_h,
        "the sectioned command palette's card_h {card_h} exceeds its canvas_h {canvas_h} at \
         900x460/zoom3.0 — this is the grouped-palette regression itself, reproduced verbatim"
    );
    assert_eq!(
        lines, 0,
        "this exact cell's own fixed chrome overhead (query line + strip + the section \
         header its one surviving item would need) already exceeds `avail_px` before any \
         item is counted — an empty band is the correct, honest answer here, not a forced \
         row that overruns the canvas; got {lines} lines"
    );
    assert_eq!(
        plan.candidate_rows(),
        0,
        "the planner's own row count must agree with the sidecar's `lines`"
    );
    assert_eq!(
        sel_row, 0,
        "no row is selectable, so the sidecar reports the default 0"
    );
    p.set_dpi(1.0);
}

/// One byte-identity scenario: a kind, whether it is cycled to a sectioned
/// lens, its canvas, and its zoom.
struct Scenario {
    kind: OverlayKind,
    sectioned: bool,
    canvas: (u32, u32),
    zoom: f32,
}

/// A GEOMETRY fingerprint of one grouped-family scenario: the card's own
/// height/x-span, the planned row count, and its sidecar report.
type Fingerprint = (
    f32,
    (f32, f32),
    usize,
    Option<(usize, usize, usize, f32, f32)>,
);

/// Fingerprints `s`. Shared by the byte-identity law below. NOT a pixel
/// hash: this file's `Cache`/`TextPipeline` ride a process-wide SHARED GPU
/// device (`test_gpu::shared_device_queue`), and the glyph atlas it packs
/// into carries real history from whichever OTHER tests happened to run
/// earlier in the same binary — measured directly (`git stash` before/after,
/// both in isolation and inside a full `render::tests` run): the geometry
/// tuple below was IDENTICAL in every combination, while an exact hash of
/// the rendered pixels moved with the atlas's own packing history alone,
/// with no change to this law's code. That is exactly why the shared
/// appearance oracle (`no_card_exceeds_its_canvas_for_any_overlay_kind`'s
/// `ink_ok`) grades a LOCAL relative-luminance delta rather than a global
/// exact hash — the same precedent applies here.
fn fingerprint(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    s: &Scenario,
) -> Fingerprint {
    let (cw, ch) = s.canvas;
    let mut v = overlay_view(s.kind, s.kind.window_rows() + 25, s.sectioned);
    v.zoom = s.zoom;
    p.set_size(cw as f32, ch as f32);
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).unwrap();
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let report = p.overlay_window_report();
    // Still renders a real frame — a smoke check that the draw path does not
    // panic for any of these scenarios, even though the pixels aren't
    // compared byte-for-byte (see the doc above).
    let _pixels = shoot(device, queue, p, cw, ch);
    (geom.card_h, plan.card_x_span(), plan.rows().len(), report)
}

/// ALREADY-FITTING GROUPED PICKERS STAY BYTE-IDENTICAL, PROVEN not
/// asserted: five representative scenarios — the command palette on "All"
/// and cycled to a sectioned lens, a narrow canvas, a tall canvas at a small
/// zoom, and a short canvas at zoom 1.0 (all comfortably clear of the
/// pathological corner the test above pins) — fingerprinted (card geometry,
/// planned row count, sidecar report) on the UNMODIFIED pre-184 code via
/// `git stash` (production files only — `overlay.rs`, `overlay_clamp.rs`,
/// `theme_picker.rs`, `plan/overlay_rows.rs`, `plan/tests.rs` — keeping this
/// test itself) and pinned here verbatim; re-measured with the fix restored:
/// identical, both times. `fit_item_rows`'s new `min_items: 0` path for the
/// grouped family is a no-op wherever the floor does not bind
/// (`saturating_sub` alone already clears 1 whenever `fit_lines >
/// overhead_rows`), so every one of these must still match exactly — a
/// mutation that touched the ordinary (non-floor) arithmetic would move one
/// of these numbers even though it has nothing to do with the floor this
/// item changed.
///
/// The hint's own gap row is one
/// more row of overhead every one of these scenarios now carries, so
/// `card_h` grows where the window had slack (`Command`, `Project`) and the
/// visible row count drops by exactly one where it did not (`Goto`,
/// `Browse`) — this test's own claim (the FLOOR fix stays a no-op here) is
/// untouched; only the baseline it diffs against moved.
#[test]
fn already_fitting_grouped_pickers_stay_byte_identical_across_the_floor_fix() {
    let _g = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping already_fitting_grouped_pickers_stay_byte_identical_across_the_floor_fix: \
             no wgpu adapter"
        );
        return;
    };
    // This law is about the floor fix's arithmetic being byte-identical, not the
    // menu bar — pin the bar off so the card geometry doesn't shift under a
    // platform where the bar defaults on (`_misc_restore` above already restores
    // whatever this found).
    crate::menubar::set_menu_bar_on(false);
    let cases: [(Scenario, Fingerprint); 5] = [
        (
            Scenario {
                kind: OverlayKind::Command,
                sectioned: false,
                canvas: (1200, 800),
                zoom: 1.0,
            },
            (
                485.800_02,
                (300.0, 900.0),
                12,
                Some((25, 12, 11, 485.800_02, 800.0)),
            ),
        ),
        (
            Scenario {
                kind: OverlayKind::Command,
                sectioned: true,
                canvas: (1200, 800),
                zoom: 1.0,
            },
            (513.0, (300.0, 900.0), 13, Some((25, 13, 12, 513.0, 800.0))),
        ),
        (
            Scenario {
                kind: OverlayKind::Goto,
                sectioned: false,
                canvas: (700, 800),
                zoom: 2.0,
            },
            // THE CELL THE CHROME PIXEL-SPACE ROUND MOVED MOST. Three logical
            // lengths bind here at zoom 2 and every one of them used to be a
            // physical number sitting beside doubled text. The card's edge-inset
            // FLOOR resolves to 20px rather than 10 (the span narrows 10..690 ->
            // 20..680); the grouped card's own pad and its drop from the canvas
            // top resolve to 24 and 80 rather than 12 and 40, which is 84px less
            // vertical room, so the height clamp seats six item rows where it
            // used to seat eight (694.0 -> 609.2). At 200% zoom a card whose
            // padding stayed at 100% was the defect, not the baseline.
            // The hint's own gap row costs this cell one visible
            // candidate row (6 -> 5): the window was already binding on
            // `avail_px` here, so the extra row of overhead is absorbed by
            // showing one fewer item, exactly as any other overhead addition
            // already would be. `card_h` ALSO drops (609.2 -> 578.8): the
            // gap row is shorter than a full row (`OVERLAY_HINT_GAP_ROW`),
            // and reclaiming that slack now outweighs `avail_px`'s own
            // clamp, so the card is content-derived here rather than
            // window-clamped. A widened `OVERLAY_HINT_GAP_ROW` (the footer
            // legibility fix) grows the same gap row further still
            // (578.8 -> 589.8): the row count this cell fits does not move
            // again, only how much of the compact row's own slack survives
            // the reclaim.
            (589.8, (20.0, 680.0), 5, Some((32, 5, 4, 589.8, 800.0))),
        ),
        // This cell used to be `History`, which is no longer a GROUPED picker
        // either: it is presented as a summoned workspace, so its numbers here
        // would be measuring a card production does not draw — the same reason
        // the `Settings` cell below was retired. `Project` is the
        // grouped kind that was not otherwise covered, held at the same tall-canvas
        // shape the History cell was chosen for.
        (
            Scenario {
                kind: OverlayKind::Project,
                sectioned: true,
                canvas: (1400, 1600),
                zoom: 0.5,
            },
            // The SUB-1 companion: at zoom 0.5 the same lengths resolve DOWN,
            // so the card is 13px shorter (261 -> 248) at the same thirteen
            // rows — the footer's 2px pad reclaims one pixel more, and the
            // grouped card's own 12px pad is 6. Enrolling chrome in `zoom * dpi`
            // moves it at every scale away from 1, not only on retina; the
            // card_x span is untouched because the edge floor never binds here.
            (
                257.0,
                (400.0, 1000.0),
                13,
                Some((25, 13, 12, 257.0, 1600.0)),
            ),
        ),
        // This fifth cell used to be `Settings`, which is no longer a
        // GROUPED picker: it is presented as a summoned workspace, whose box
        // comes from the canvas rather than from a width cap, so its numbers
        // here would be measuring a card production does not draw. `Browse` is
        // the grouped kind that was not otherwise covered, held at the same
        // short-canvas shape the Settings cell was chosen for.
        (
            Scenario {
                kind: OverlayKind::Browse,
                sectioned: true,
                canvas: (900, 460),
                zoom: 1.0,
            },
            // Same absorption shape as the `Goto` cell above: one
            // fewer visible row (7 -> 6), and `card_h` also drops (331.8 ->
            // 316.6) for the same content-derived-not-window-clamped reason;
            // the widened gap row grows it again (316.6 -> 322.6).
            (322.6, (150.0, 750.0), 6, Some((32, 6, 5, 322.6, 460.0))),
        ),
    ];
    // EVERY cell is reported, not just the first to move: a pinned-fingerprint
    // law that stops at the first mismatch tells a reader one number when the
    // change in front of them moved three, and each round then costs a rerun to
    // learn the next one.
    let mut moved = Vec::new();
    for (scenario, expected) in &cases {
        let got = fingerprint(&device, &queue, &mut p, scenario);
        if got != *expected {
            moved.push(format!(
                "{:?} sectioned={} {:?} zoom={}: (card_h, card_x_span, plan_len, report) \
                 {expected:?} -> {got:?}",
                scenario.kind, scenario.sectioned, scenario.canvas, scenario.zoom
            ));
        }
    }
    assert!(
        moved.is_empty(),
        "already-fitting grouped scenarios changed — the grouped-family floor fix must be \
         a no-op here:\n{}",
        moved.join("\n")
    );
    p.set_dpi(1.0);
}
