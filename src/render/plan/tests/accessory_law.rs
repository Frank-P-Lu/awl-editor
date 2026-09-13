//! **THE ACCESSORY CLUSTER'S PUBLISHED LANES, GRADED AGAINST THE INK AND THE
//! POINTER** — the device-level companion to [`crate::render::plan::accessory_lane`], over the
//! real pipeline and the whole world roster. The sibling laws in `super` assert the planner's
//! arithmetic with no device at all; this file asks whether the number the sidecar
//! PUBLISHES describes the lane the frame actually DREW and the rail a press
//! actually lands in.
//!
//! Grading the report against the plan it was projected from would be a
//! tautology, so each lane is graded against something that never reads it:
//! [`grade_label`] against the shaped glyphs' own ink off the buffer the frame
//! uploaded and the seat the draw handed glyphon; [`grade_value`] against the same
//! for the accessory buffer, plus the name lane it grows toward; [`grade_rail`]
//! against the value lane it is pinned to by `rowlayout::rail_accessory_width`,
//! and against the POINTER itself.
//!
//! **PRESENCE IS GRADED, NOT ONLY POSITION**, because a floor over a width is
//! satisfied by publishing no lane at all. The sweep counts what it enrolled —
//! labels, values, rails, both accessory mirrors, and the YIELDED cells where the
//! card dropped its whole accessory column — and fails if any population is empty.
//! The swept widths straddle that boundary on purpose; without a crossing the law
//! would grade the comfortable regime twice.
//!
//! **THE MENU BAR IS AN AXIS HERE.** Its reserve costs the card a whole row, and a
//! different planned row count is a different widest label, hence a different
//! accessory budget — the granted/yielded boundary moves with the arm.
//!
//! Finally [`assert_budget_returns_to_the_names`] asserts the one thing separating
//! the lanes was for: yielding the column hands its budget back to the names, and
//! somewhere on the roster the names genuinely pay for it. Its reference is the
//! same card geometry shaped without an accessory — not the widest swept card,
//! whose capped Settings composition can still retain the accessory.

use super::super::{Lane, PlannedRowRect, RailLane};
use crate::overlay::{OverlayKind, OverlayState};
use crate::render::TextPipeline;
use crate::render::chrome::{OverlaySpanInks, VisualSelection};
use crate::render::rowlayout::{ColumnFlow, rail_accessory_width};
use crate::render::tests::{
    SETTINGS_VIEW_PARKED_WINDOW_ROWS, headless_dqp, settings_overlay_view, view,
};
use crate::theme;

/// The Settings corpus is the only one that carries all three lanes at once — a
/// name, a value readout, and a rail on its Range rows — so it is the surface
/// this law grades. Built through the same production wiring `overlay::build`'s
/// Settings arm uses.
fn settings_state() -> OverlayState {
    let vals = crate::settings::SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.4,
        scroll_sensitivity: 1.0,
        default_folder: "/n".into(),
        workspace: "/w".into(),
        project_root: "/p".into(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    };
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov
}

/// Shape the same planned rows through the production shaper after removing
/// only their accessory data. The resolved card/workspace geometry and diagonal
/// cluster stay fixed, so any name elision that remains belongs to the card
/// itself rather than to the accessory budget.
fn labels_without_accessory(
    p: &mut TextPipeline,
    geom: &crate::render::chrome::OverlayGeom,
    plan: &super::super::OverlayRowPlan,
) -> std::collections::BTreeMap<usize, f32> {
    let bindings = std::mem::take(&mut p.overlay_bindings);
    let ranges = std::mem::take(&mut p.overlay_ranges);
    let right_shown = p.overlay_right_shown;
    let chrome = crate::render::overlay_chrome_theme();
    let shown = p.overlay_shape_text(
        geom,
        plan,
        OverlaySpanInks {
            ink: chrome.base_content.to_glyphon(),
            muted: chrome.muted.to_glyphon(),
            selected: None,
        },
        &VisualSelection::default(),
        true,
    );
    let labels = p.overlay_row_primary_px(geom);
    p.overlay_bindings = bindings;
    p.overlay_ranges = ranges;
    p.overlay_right_shown = right_shown;
    assert!(!shown, "a no-accessory reference cannot grant an accessory");
    labels
}

/// What the sweep enrolled, so a green run can be shown to have graded something.
#[derive(Default)]
struct Enrolled {
    labels: usize,
    values: usize,
    rails: usize,
    yielded_cells: usize,
    granted_cells: usize,
    yielded_worlds: std::collections::BTreeSet<&'static str>,
    granted_worlds: std::collections::BTreeSet<&'static str>,
    flows: Vec<ColumnFlow>,
    cells: Vec<Cell>,
}

/// One swept cell's NAME lanes and whether that cell got its accessory column —
/// the pair the budget relation below is asserted over.
struct Cell {
    world: &'static str,
    bar: bool,
    logical_width: u32,
    granted: bool,
    labels: Vec<(usize, f32, f32)>,
}

/// A controlled flat-card crossing which makes BOTH sides of the budget rule
/// observable. Settings' current roster genuinely yields on narrow cells, but
/// no granted cell elides a name: treating a different visible row at another
/// width as "payment" was the stale oracle this repair removes.
fn assert_budget_mechanism_crosses(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
) {
    let ambient_world = theme::active_index();
    theme::set_active_by_name("Tawny").expect("the reported world remains enrolled");
    p.sync_theme();
    let mut cell = |logical_width| {
        p.set_dpi(1.0);
        p.set_size(logical_width as f32, 600.0);
        let mut v = view("hello\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = "Budget probe".into();
        v.overlay_items = vec!["W".repeat(48)];
        v.overlay_bindings = vec!["i".repeat(20)];
        v.overlay_window_rows = 1;
        p.set_view(&v);
        p.prepare(device, queue, logical_width, 600).unwrap();
        let geom = p.overlay_geometry(logical_width);
        let plan = p.overlay_row_plan(&geom);
        let row = p.overlay_row_geometry().unwrap().rows[0];
        let width = row.lanes.label.expect("the probe draws its name").w;
        let reference = labels_without_accessory(p, &geom, &plan)[&row.display];
        (row.lanes.value.is_some(), width, reference)
    };
    let returned = cell(280);
    let paid = cell(524);
    theme::set_active(ambient_world);
    p.sync_theme();
    assert!(
        !returned.0 && (returned.1 - returned.2).abs() < 0.5,
        "the yielded control did not return its budget: {returned:?}"
    );
    assert!(
        paid.0 && paid.1 < paid.2 - 0.5,
        "the granted control did not make its name pay: {paid:?}"
    );
    eprintln!("accessory budget counterfactual: paid {paid:?}; returned {returned:?}");
}

#[test]
#[allow(clippy::too_many_lines)]
fn published_row_lanes_match_the_drawn_ink_and_the_clickable_rail() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping published_row_lanes law: no wgpu adapter");
        return;
    };
    let mut e = Enrolled::default();
    // The enrolment is the ROSTER, not a named world: the accessory mirror is a
    // per-world answer, and pinning this to two worlds is how a sweep stops
    // covering the axis it exists for the day a world changes composition.
    // The AMBIENT values, captured rather than assumed: a `cfg!`-derived default
    // read inside a test describes the host that compiled it, not the branch the
    // process actually took, so restoring to a named constant restores the wrong
    // thing under any forcing of either knob.
    let ambient_bar = crate::menubar::menu_bar_on();
    let ambient_world = theme::active_index();
    for world in theme::THEMES.iter().map(|t| t.name) {
        theme::set_active_by_name(world).expect("roster world resolves");
        p.sync_theme();
        for &bar in &[true, false] {
            crate::menubar::set_menu_bar_on(bar);
            // These widths STRADDLE the accessory column's yield boundary on the
            // shipped roster, which is what makes `yielded_cells` non-zero and the
            // yielding state graded rather than only the comfortable one.
            for &logical_width in &[464u32, 520, 640, 1200] {
                for &dpi in &[1.0f32, 2.0] {
                    let width = (logical_width as f32 * dpi).round() as u32;
                    let height = (800.0 * dpi).round() as u32;
                    p.set_dpi(dpi);
                    p.set_size(width as f32, height as f32);
                    let ov = settings_state();
                    // **THE SHIPPED ROW COUNT, NOT THE FIXTURE DEFAULT.** The
                    // shared Settings fixture leaves `overlay_window_rows` at an
                    // inert 12; `sync_view` and both capture paths set
                    // `ov.window_rows()`, and the difference is the whole
                    // question — a wider drawn set means a wider widest label,
                    // which means a tighter accessory budget. At 12 the column
                    // fits at every width this sweep could reach, so the yielding
                    // state would have gone ungraded.
                    let mut v = settings_overlay_view(&ov, ov.window_rows());
                    debug_assert!(ov.window_rows() > SETTINGS_VIEW_PARKED_WINDOW_ROWS);
                    // Settings' own `workspace_shape` answers unconditionally, so
                    // this is the ONE reachable arm rather than a swept pair.
                    let ws = OverlayKind::Settings.workspace_shape().is_some();
                    v.overlay_workspace = ws;
                    v.overlay_detail_focus = ws;
                    p.set_view(&v);
                    p.prepare(&device, &queue, width, height).unwrap();

                    let ctx = format!("world={world} bar={bar} w={logical_width} dpi={dpi}");
                    let geom = p.overlay_geometry(width);
                    let plan = p.overlay_row_plan(&geom);
                    let report = p
                        .overlay_row_geometry()
                        .unwrap_or_else(|| panic!("{ctx}: a summoned card reports geometry"));
                    assert_eq!(
                        report.rows.len(),
                        plan.rows().len(),
                        "{ctx}: one published entry per planned display line"
                    );
                    let flow = p.overlay_accessory_flow();
                    if !e.flows.contains(&flow) {
                        e.flows.push(flow);
                    }
                    let lh = p.overlay_lh();
                    let bands = p.overlay_panel_bands(&geom, &plan);
                    let first_line = geom.shaped_first_row_line();

                    let any_value = report.rows.iter().any(|r| r.lanes.value.is_some());
                    if any_value {
                        e.granted_cells += 1;
                        e.granted_worlds.insert(world);
                    } else {
                        e.yielded_cells += 1;
                        e.yielded_worlds.insert(world);
                    }
                    // A card may grant its value column and STILL seat no rail:
                    // the rail needs its own room inside what the column left. The
                    // implication that DOES hold is graded per row in `grade_rail`.
                    for row in &report.rows {
                        let k = row.display;
                        let mid_y = row.y + row.h * 0.5;
                        if let Some(label) = row.lanes.label {
                            e.labels += 1;
                            let seat = bands.as_ref().map_or(geom.text_left, |b| b[k + 1].left);
                            grade_label(&p, &ctx, k, label, seat, first_line + k);
                        }
                        if let Some(value) = row.lanes.value {
                            e.values += 1;
                            grade_value(&p, &ctx, k, value, row.lanes.label, geom.header_rows + k);
                        }
                        if let Some(rail) = row.lanes.rail {
                            e.rails += 1;
                            grade_rail(&p, &ctx, row, rail, mid_y, lh, flow);
                        }
                    }
                    // Banked for the budget relation, at ONE scale only: every
                    // figure doubles, so mixing them compares 1x against 2x.
                    // Shape the SAME plan and geometry without its accessory
                    // after grading the frame (the shaper reuses the draw buffer).
                    // This is the real budget counterfactual; the widest swept
                    // card is not one now that the Settings composition is capped.
                    if dpi == 1.0 {
                        let drawn: std::collections::BTreeMap<usize, f32> = report
                            .rows
                            .iter()
                            .filter_map(|row| row.lanes.label.map(|label| (row.display, label.w)))
                            .collect();
                        let without_accessory = labels_without_accessory(&mut p, &geom, &plan);
                        assert!(
                            !drawn.is_empty(),
                            "{ctx}: the drawn Settings plan has no name ink"
                        );
                        let labels: Vec<(usize, f32, f32)> = drawn
                            .iter()
                            .map(|(&display, &width)| {
                                let Some(&reference) = without_accessory.get(&display) else {
                                    panic!("{ctx}: no-accessory shaping omitted row {display}");
                                };
                                (display, width, reference)
                            })
                            .collect();
                        assert!(
                            labels.iter().all(|(_, _, reference)| *reference > 0.0),
                            "{ctx}: the independently shaped no-accessory Settings plan has no ink"
                        );
                        e.cells.push(Cell {
                            world,
                            bar,
                            logical_width,
                            granted: any_value,
                            labels,
                        });
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(ambient_world);

    // ── NON-VACUITY: every population this law claims to grade was reached ──
    assert!(
        e.labels > 100 && e.values > 100 && e.rails > 20,
        "the sweep graded {} labels, {} values, {} rails — a lane population this \
         small means the enrolment stopped matching, not that the product changed",
        e.labels,
        e.values,
        e.rails
    );
    // The same world must be seen on BOTH sides of the boundary, not merely one
    // world yielding somewhere and a different one granting elsewhere: only the
    // crossing proves the swept widths actually bracket the gate.
    assert!(
        e.yielded_worlds
            .iter()
            .any(|w| e.granted_worlds.contains(w)),
        "no world was seen on both sides of the accessory column's gate — \
         yielded {:?}, granted {:?}. Without a crossing, the narrow arm and the \
         wide arm could each be grading a different, comfortable regime.",
        e.yielded_worlds,
        e.granted_worlds
    );
    assert!(
        e.yielded_cells > 0 && e.granted_cells > 0,
        "the swept widths must STRADDLE the accessory column's yield boundary: \
         {} cells granted the column and {} yielded it. All-granted means the \
         narrow arm stopped being narrow and the interesting state is ungraded.",
        e.granted_cells,
        e.yielded_cells
    );
    assert_eq!(
        e.flows.len(),
        2,
        "both accessory mirrors must be reached across the roster, saw {:?} — a \
         one-sided sweep grades one arm of a rule whose whole shape is the mirror",
        e.flows
    );
    assert_budget_returns_to_the_names(&e.cells);
    assert_budget_mechanism_crosses(&device, &queue, &mut p);
}

/// **WHAT THE PUBLISHED LANES SAY ABOUT THE WIDTH BUDGET**, asserted per
/// (world, menu-bar) run rather than per cell, because the claim is a RELATION
/// between two regimes of the same card.
///
/// Only runs observed on BOTH sides enter this relation. A composition that
/// never draws an accessory still has its label lane graded above, but a narrow
/// label there is ordinary card pressure, not budget returned by this gate.
///
/// **Yielding the accessory column returns its budget to the names.** On a
///    cell that reports no value lane, the widest name must match the same plan
///    and geometry shaped without an accessory. If a future budget yielded the
///    column and kept accessory-driven elision anyway, the card would have given
///    up its readouts and bought nothing, and this goes red.
///
/// The controlled companion above proves separately that some granted name
/// really pays and that a yielded name really gets the budget back. Settings'
/// current visible roster does not exercise the first mechanism; requiring it
/// here is how the old widest-card oracle mistook a different row for elision.
fn assert_budget_returns_to_the_names(cells: &[Cell]) {
    let mut returned = 0usize;
    let crossing_runs: std::collections::BTreeSet<(&str, bool)> = cells
        .iter()
        .filter_map(|run| {
            let same_run = |c: &&Cell| c.world == run.world && c.bar == run.bar;
            let crosses = cells.iter().filter(same_run).any(|c| c.granted)
                && cells.iter().filter(same_run).any(|c| !c.granted);
            crosses.then_some((run.world, run.bar))
        })
        .collect();
    for run in cells {
        if !crossing_runs.contains(&(run.world, run.bar)) {
            continue;
        }
        if !run.granted {
            returned += 1;
            for (display, width, reference) in &run.labels {
                assert!(
                    (*width - *reference).abs() < 0.5,
                    "world={} bar={} w={} row {display}: the accessory column was YIELDED and \
                     the name still measures {width} against its same-geometry no-accessory \
                     {reference} — giving up the readouts must hand their budget back to the names",
                    run.world,
                    run.bar,
                    run.logical_width
                );
            }
        }
    }
    assert!(
        !crossing_runs.is_empty() && returned > 0,
        "the Settings budget relation graded {} same-world/menu runs and {returned} \
         yielded cells. Zero of either population makes the relation vacuous: without a \
         crossing run there is no same-composition comparison, and without a yielded cell \
         there is no returned budget to grade.",
        crossing_runs.len()
    );
}

/// **THE NAME LANE vs THE SHAPED GLYPHS' OWN INK.** `seat` is the DRAW's own band
/// origin for this row — never the report's `label.x` — so the two answers arrive
/// by different routes and can disagree.
fn grade_label(p: &TextPipeline, ctx: &str, k: usize, label: Lane, seat: f32, line_i: usize) {
    assert!(
        label.w > 0.0,
        "{ctx} row {k}: a reported lane is never empty"
    );
    let (l, r) = glyph_ink(&p.panel_buffer, line_i)
        .unwrap_or_else(|| panic!("{ctx} row {k}: a reported label lane must have shaped ink"));
    let (ink_l, ink_r) = (seat + l, seat + r);
    // OVERLAP, never containment either way: an advance width and a swash's real
    // ink disagree by a pixel or so in BOTH directions across this roster, and a
    // staggered row legitimately publishes an x outside its own band.
    assert!(
        ink_r > label.x - 1.5 && ink_l < label.x + label.w + 1.5,
        "{ctx} row {k}: published label [{}, {}] does not meet its drawn ink \
         [{ink_l}, {ink_r}]",
        label.x,
        label.x + label.w
    );
    assert!(
        (ink_l - label.x).abs() < 2.0,
        "{ctx} row {k}: published label origin {} is not where the frame seated \
         the ink ({ink_l})",
        label.x
    );
    // AND THE WIDTH. Pinned only by its origin, a lane's width scaled by any
    // uniform factor stays green — including at both capture scales, since a
    // factor survives the doubling relation untouched.
    assert!(
        (label.w - (ink_r - ink_l)).abs() < 2.0,
        "{ctx} row {k}: published label width {} is not the width of the ink it \
         claims ({})",
        label.w,
        ink_r - ink_l
    );
}

/// **THE VALUE LANE**, against the accessory buffer's own ink width — which is
/// seat-independent, so it holds whichever end the column hangs on — and against
/// the name lane it grows toward.
fn grade_value(
    p: &TextPipeline,
    ctx: &str,
    k: usize,
    value: Lane,
    label: Option<Lane>,
    line_i: usize,
) {
    assert!(
        value.w > 0.0,
        "{ctx} row {k}: a reported lane is never empty"
    );
    let (l, r) = glyph_ink(&p.panel_bind_buffer, line_i)
        .unwrap_or_else(|| panic!("{ctx} row {k}: a reported value lane must have shaped ink"));
    assert!(
        (value.w - (r - l)).abs() < 2.0,
        "{ctx} row {k}: published value width {} is not the width of the accessory \
         ink it claims ({})",
        value.w,
        r - l
    );
    if let Some(label) = label {
        let (a, b) = (label.x, label.x + label.w);
        let (c, d) = (value.x, value.x + value.w);
        assert!(
            b <= c + 0.01 || d <= a + 0.01,
            "{ctx} row {k}: the name lane [{a}, {b}] and the value lane [{c}, {d}] \
             overlap — a row's two ends grow toward each other and must never meet"
        );
    }
}

/// **THE RAIL**, against the value lane it annotates and against the POINTER.
///
/// The rail hangs one fixed accessory gap inward of the value text off the same
/// anchor, whichever way the cluster mirrors, so the two published edges are
/// pinned to each other through the owner the hit-test reads. Then the pointer
/// itself: pressed at the published track's own midpoint, and 1.5px outside the
/// published hit band, which must be the whole band a press is accepted in.
fn grade_rail(
    p: &TextPipeline,
    ctx: &str,
    row: &PlannedRowRect,
    rail: RailLane,
    mid_y: f32,
    lh: f32,
    flow: ColumnFlow,
) {
    let k = row.display;
    let value = row.lanes.value.unwrap_or_else(|| {
        panic!(
            "{ctx} row {k}: a rail without a value lane is a control with no \
             readout — the two are gated together"
        )
    });
    let gap = rail_accessory_width(lh) - rail.w;
    assert!(
        gap > 0.0,
        "{ctx} row {k}: the accessory reservation must exceed the rail it reserves \
         for (rail {} of {})",
        rail.w,
        rail_accessory_width(lh)
    );
    let (rail_inner, value_edge) = match flow {
        ColumnFlow::Leftward => (rail.x + rail.w, value.x),
        ColumnFlow::Rightward => (rail.x, value.x + value.w),
    };
    let seen = (value_edge - rail_inner).abs();
    assert!(
        (seen - gap).abs() < 0.05,
        "{ctx} row {k}: the rail's inner edge {rail_inner} must sit exactly one \
         accessory gap ({gap}) from the value lane's edge {value_edge}, got {seen}"
    );
    if let Some(label) = row.lanes.label {
        assert!(
            rail.x >= label.x + label.w - 0.01 || rail.x + rail.w <= label.x + 0.01,
            "{ctx} row {k}: rail [{}, {}] crosses the name lane [{}, {}]",
            rail.x,
            rail.x + rail.w,
            label.x,
            label.x + label.w
        );
    }
    assert!(
        rail.hit_x < rail.x + 0.01
            && rail.hit_x + rail.hit_w > rail.x + rail.w - 0.01
            && rail.hit_w > rail.w,
        "{ctx} row {k}: the hit band [{}, {}] must strictly contain the drawn \
         track [{}, {}]",
        rail.hit_x,
        rail.hit_x + rail.hit_w,
        rail.x,
        rail.x + rail.w
    );
    let item = row
        .item
        .unwrap_or_else(|| panic!("{ctx} row {k}: a rail row always carries an item"));
    let cx = rail.x + rail.w * 0.5;
    assert_eq!(
        p.overlay_range_at(cx, mid_y).map(|(i, _)| i),
        Some(item),
        "{ctx} row {k}: a press at the published rail's own midpoint ({cx}, \
         {mid_y}) must adjust that row's range"
    );
    for px in [rail.hit_x - 1.5, rail.hit_x + rail.hit_w + 1.5] {
        assert_ne!(
            p.overlay_range_at(px, mid_y).map(|(i, _)| i),
            Some(item),
            "{ctx} row {k}: a press 1.5px outside the published hit band ({px}) \
             must not adjust that row's range"
        );
    }
    // The published track inverts back through the owner the DRAG reads, so an
    // edge press reads as an edge fraction.
    let x1 = rail.x + rail.w;
    let (lo, hi) = (
        crate::render::rail_frac_at(rail.x, rail.x, x1),
        crate::render::rail_frac_at(x1, rail.x, x1),
    );
    assert!(
        lo < 0.001 && hi > 0.999,
        "{ctx} row {k}: the published track must invert to its own full fraction \
         range, got {lo}..{hi}"
    );
}

/// The min/max x of the GLYPHS on shaped line `line_i`, in the buffer's own
/// coordinates — the ink itself, never the line's advance width, which includes
/// trailing space the frame draws nothing for.
fn glyph_ink(buffer: &glyphon::Buffer, line_i: usize) -> Option<(f32, f32)> {
    for run in buffer.layout_runs() {
        if run.line_i != line_i || run.glyphs.is_empty() {
            continue;
        }
        let l = run.glyphs.iter().map(|g| g.x).fold(f32::INFINITY, f32::min);
        let r = run
            .glyphs
            .iter()
            .map(|g| g.x + g.w)
            .fold(f32::NEG_INFINITY, f32::max);
        return (l.is_finite() && r > l).then_some((l, r));
    }
    None
}
