//! ITEM 131 (the row-composition seam) — A ROW THAT STEPS SIDEWAYS IS CLICKABLE
//! EXACTLY WHERE IT IS DRAWN.
//!
//! Item 131 wants a third, theme-owned row composition whose rows are staggered
//! horizontally down a diagonal spine. The renderer could already stagger rows
//! before this file existed — the wild-menu slant probe
//! (`AWL_OVERLAY_SLANT_FORCE`) offsets each successive row's glyph origin and its
//! plate — but the offset was applied by the DRAW emitters alone. The pointer
//! hit-test (`OverlayRowPlan::row_at`) kept testing the card's own undisplaced
//! x-span, so a staggered row was clickable in a strip where nothing was drawn.
//! That is the exact failure DESIGN.md §8 forbids ("drawn geometry and hit-test
//! geometry have one owner. A surface that looks clickable must be clickable
//! where it is drawn at supported zoom and DPI"), and it is load-bearing for item
//! 131: every later law about the diagonal's attachment band, its selected
//! segment and its clusters is a claim about where a row *is*, and none of them
//! can mean anything while the draw and the pointer answer that separately.
//!
//! The offset is now planned, not drawn: `PlannedRow::dx` comes out of the same
//! planner pass as `top`/`height`, from one input step
//! (`TextPipeline::overlay_row_dx_step`), and every consumer — the per-row text
//! area, the bar plate, the Pane band, the selected bar, and `row_at` — reads
//! that one number. `dx == 0.0` for every shipping world, so this is a no-op
//! everywhere until a composition asks for it.
//!
//! ## The axis
//!
//! The staggering axis is swept against the FULL no-wildcard `OverlayKind`
//! roster, both list styles, four canvases and both DPIs — the same axes item
//! 174's own headline law sweeps, because the defect is not a property of one
//! picker. Two independent production owners are compared: the quads
//! `overlay_bar_rects_probe` actually emits (the DRAWN evidence) and
//! `overlay_row_at` (the INTERACTIVE evidence). They are not two readings of one
//! accessor — one comes from `slant_bar_span` inside the selection emitter, the
//! other from the planner's inverse.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;
use crate::render::overrides::SlantProbe;

/// The stagger the sweep drives. Large enough that a wrong x-test is unmissable
/// at every canvas in the sweep, small enough that the deepest row still holds
/// real width on the narrowest one (`overlay_shape_text` already taxes the
/// elision budget by the same total).
const STEP_PX: f32 = 9.0;

fn staggered_view(kind: OverlayKind, n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = (n / 3).min(n.saturating_sub(1));
    v.overlay_hint = "type to filter".into();
    if crate::facets::scheme(kind).is_some() {
        v.overlay_lens = vec![
            ("All".into(), true),
            ("File".into(), false),
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
    if kind == OverlayKind::Spell {
        v.overlay_spell = Some((0, 0, 5));
        v.overlay_items = (0..n.min(5)).map(|i| format!("suggest{i}")).collect();
        v.overlay_bindings = Vec::new();
        v.overlay_selected = 0;
        v.overlay_hint = String::new();
    }
    v
}

/// A `Bars` style whose plates cover EVERY row (`BarCoverage::All`) at the band's
/// full width, so each planned row has one drawn surface whose left edge is a
/// direct reading of that row's own composition origin.
const PLATED_BARS: theme::ListStyle = theme::ListStyle::Bars {
    radius: 6.0,
    gap: 8.0,
    grow_px: 24.0,
    extent: theme::BarExtent::FullWidth,
    coverage: theme::BarCoverage::All,
};

/// Grade one rendered card: every planned row's clickable span against its own
/// drawn composition, plus the drawn plates. Returns
/// `(rows graded, plates graded, deepest dx seen)`.
fn grade_staggered_card(
    p: &TextPipeline,
    cw: u32,
    plates: &[[f32; 4]],
    ctx: &str,
) -> (usize, usize, f32) {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let (x0, x1) = plan.card_x_span();
    let vis_sel = plan.selected_display();
    // The plate's own left edge against the row's step: a style insets its plates
    // from the band by its own constant, so what must hold is that the DRAWN
    // origin advances by exactly `dx` per row — the same number the pointer
    // boundary advances by.
    let mut plate_origins: Vec<(usize, f32)> = Vec::new();
    let (mut rows_graded, mut deepest) = (0usize, 0.0f32);
    for row in plan.rows() {
        assert!(
            (row.dx - STEP_PX * row.display as f32).abs() < 0.01,
            "{ctx}: display row {} planned dx {} for a {STEP_PX}px step",
            row.display,
            row.dx
        );
        deepest = deepest.max(row.dx);
        let mid_y = row.top + row.height * 0.5;
        // The row's own composition begins at `x0 + dx`. Anything left of that is
        // bare card on this row.
        if row.dx > 1.0 {
            for probe_x in [x0, x0 + row.dx * 0.5, x0 + row.dx - 0.5] {
                assert_eq!(
                    p.overlay_row_at(probe_x, mid_y),
                    None,
                    "{ctx}: display row {} is drawn starting at x {:.2} but a pointer at \
                     ({probe_x:.2}, {mid_y:.2}) — left of its own composition, over bare \
                     card — still selects it",
                    row.display,
                    x0 + row.dx
                );
            }
        }
        for probe_x in [x0 + row.dx + 0.5, (x0 + row.dx + x1) * 0.5, x1] {
            assert_eq!(
                p.overlay_row_at(probe_x, mid_y),
                row.item,
                "{ctx}: display row {} draws item {:?} but a pointer at ({probe_x:.2}, \
                 {mid_y:.2}) — inside its drawn composition — resolves differently",
                row.display,
                row.item
            );
        }
        rows_graded += 1;

        // THE DRAWN EVIDENCE, from a different owner: the plate the selection
        // emitter actually produced for this row.
        if row.item.is_none() || Some(row.display) == vis_sel {
            continue; // header lines and the grown selected plate
        }
        let want_top = row.top + 4.0; // gap * 0.5
        if let Some(plate) = plates
            .iter()
            .find(|r| (r[1] - want_top).abs() < 0.51 && r[3] < row.height + 0.51)
        {
            plate_origins.push((row.display, plate[0] - row.dx));
        }
    }
    if let Some(&(_, base)) = plate_origins.first() {
        for &(display, origin) in &plate_origins {
            assert!(
                (origin - base).abs() < 0.51,
                "{ctx}: display row {display}'s drawn plate does not advance by its own \
                 planned step — its origin minus dx is {origin:.2} where row {}'s is \
                 {base:.2}. The plate and the pointer boundary must move together.",
                plate_origins[0].0
            );
        }
    }
    (rows_graded, plate_origins.len(), deepest)
}

/// THE HEADLINE LAW. With a composition that steps every successive row `STEP_PX`
/// further in, on every `OverlayKind`, both list styles, four canvases and both
/// DPIs:
///
/// * a pointer INSIDE the row's own step — where the row is demonstrably not
///   drawn — must not select it;
/// * a pointer just past the step must;
/// * and (on the plated style) the drawn plate's own left edge must be that same
///   boundary, so the two owners agree by measurement rather than by both
///   reading one accessor.
#[test]
fn a_staggered_row_is_clickable_exactly_where_its_composition_is_drawn() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping a_staggered_row_is_clickable_exactly_where_it_is_drawn: no wgpu adapter"
        );
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true); // settle the entrance so `dx` is the full step

    let styles: [(&str, theme::ListStyle); 2] =
        [("pane", theme::ListStyle::Pane), ("bars", PLATED_BARS)];
    let canvases: [(u32, u32); 4] = [(1200, 800), (700, 800), (900, 460), (1400, 1600)];

    crate::render::set_slant_test_override(Some(SlantProbe {
        px_per_row: STEP_PX,
        italic: false,
    }));

    let mut graded_rows = 0usize;
    let mut graded_plates = 0usize;
    let mut deepest_dx = 0.0f32;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for (sname, style) in styles {
            crate::render::set_list_style_test_override(Some(style));
            for (lw, lh) in canvases {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for kind in OverlayKind::ALL {
                    let v = staggered_view(kind, 24);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let ctx = format!("{kind:?} dpi={dpi} list={sname} canvas={cw}x{ch}");
                    let plates = match sname {
                        "bars" => p.overlay_bar_rects_probe().1,
                        _ => Vec::new(),
                    };
                    let (rows, plated, deep) = grade_staggered_card(&p, cw, &plates, &ctx);
                    graded_rows += rows;
                    graded_plates += plated;
                    deepest_dx = deepest_dx.max(deep);
                }
            }
        }
    }
    crate::render::set_slant_test_override(None);
    crate::render::set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(
        graded_rows > 500,
        "the sweep must grade hundreds of staggered rows, got {graded_rows}"
    );
    assert!(
        graded_plates > 100,
        "the DRAWN-plate arm must actually find plates to compare against, got {graded_plates}"
    );
    assert!(
        deepest_dx > 4.0 * STEP_PX,
        "the sweep must reach genuinely deep rows or the x-test is never stressed, deepest \
         dx was {deepest_dx}"
    );
}

/// THE OTHER HALF, and the one that protects every shipping world: with no
/// composition asking for a stagger, every planned row's `dx` is exactly zero and
/// `row_at` is the card's own undisplaced span, byte for byte. The per-row x-test
/// must not have quietly become a per-row x-test with a rounding error in it.
#[test]
fn an_upright_composition_plans_no_offset_and_keeps_the_cards_own_span() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping an_upright_composition_plans_no_offset: no wgpu adapter");
        return;
    };
    crate::render::set_slant_test_override(None);
    let mut graded = 0usize;
    for kind in OverlayKind::ALL {
        let v = staggered_view(kind, 24);
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        let (x0, x1) = plan.card_x_span();
        assert_eq!(
            p.overlay_row_dx_step(),
            0.0,
            "{kind:?}: an upright world must ask for no horizontal step"
        );
        for row in plan.rows() {
            assert_eq!(
                row.dx, 0.0,
                "{kind:?}: display row {} planned a nonzero dx",
                row.display
            );
            let mid_y = row.top + row.height * 0.5;
            for probe_x in [x0, (x0 + x1) * 0.5, x1] {
                assert_eq!(
                    p.overlay_row_at(probe_x, mid_y),
                    row.item,
                    "{kind:?}: display row {} must stay clickable across the whole card at \
                     x {probe_x}",
                    row.display
                );
            }
            graded += 1;
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded > 100, "the upright arm graded {graded} rows");
}

// --- The source law: one owner of a row's x -----------------------------------

/// `overlay_row_dx_step` is the composition's step, and it may be READ in exactly
/// two places: where it is defined, and the one plan-input construction site that
/// hands it to the planner. Anywhere else it would be a second row-x arithmetic —
/// the shape this item exists to retire — and `row_at` would have no way to agree
/// with it. The retired per-row accessor `overlay_slant_dx(` may not come back at
/// all.
#[test]
fn only_the_planner_derives_an_overlay_rows_horizontal_offset() {
    const STEP_OWNERS: &[&str] = &["pipeline_overlay.rs", "chrome/overlay.rs"];
    let render_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("render");
    let mut step_hits: Vec<String> = Vec::new();
    let mut retired_hits: Vec<String> = Vec::new();
    let mut files = Vec::new();
    collect(&render_root, &mut files);
    for path in files {
        let rel = path
            .strip_prefix(&render_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") || line.trim_start().starts_with("///") {
                continue; // prose, not code
            }
            if line.contains("overlay_row_dx_step(") {
                step_hits.push(format!("{rel}:{}", i + 1));
            }
            if line.contains("overlay_slant_dx(") {
                retired_hits.push(format!("{rel}:{}", i + 1));
            }
        }
    }
    let strays: Vec<&String> = step_hits
        .iter()
        .filter(|h| !STEP_OWNERS.iter().any(|o| h.starts_with(o)))
        .collect();
    assert!(
        strays.is_empty(),
        "only {STEP_OWNERS:?} may read the row composition's horizontal step. A consumer \
         that multiplies it by a row index is a parallel row-x calculation, and the pointer \
         hit-test cannot follow it. Read `PlannedRow::dx` / `OverlayRowPlan::row_dx` \
         instead. offending lines: {strays:?}"
    );
    assert!(
        retired_hits.is_empty(),
        "`overlay_slant_dx(` is the retired draw-only per-row offset that the hit-test \
         could not see; it must not return: {retired_hits:?}"
    );
    // NON-VACUOUS: the owners really do carry it.
    assert_eq!(
        step_hits.len(),
        2,
        "expected exactly the definition and the one plan-input site to name the step, \
         found {step_hits:?}"
    );
}

/// Every production `.rs` under `src/render/`, skipping the test tier.
fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                continue;
            }
            collect(&path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        out.push(path);
    }
}
