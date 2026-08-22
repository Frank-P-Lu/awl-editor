//! `ListStyle::Ruled` GRADUATED: the reach and the proof a shipped
//! row composition owes.
//!
//! The composition and its `Weight` selection are decided. What this file adds
//! is the part that made it a prototype: the FULL no-wildcard `OverlayKind`
//! row-surface sweep, the Settings WORKSPACE (both regions, both focus stages),
//! every `SettingId × SettingKind`, drawn↔hit-test↔sidecar agreement at 1× and
//! 2× DPI, and the pixel laws.
//!
//! # What each law is actually asking
//!
//! The style is defined by ABSENCE, so most of its claims are negative and a
//! negative claim goes vacuous easily. Every law here therefore carries its own
//! non-vacuity arm — a count of what it graded, or the positive control that
//! must FAIL the same predicate (a `Pane` world's filled band, a `Bars` world's
//! plates), so "no surface was found" can never be reported by a sweep that
//! rendered nothing.
//!
//! # The axes, and why these ones
//!
//! `OverlayKind::ALL` (no wildcard), both `RuleSelection` treatments, both DPIs,
//! and four canvases including one narrow enough to stage the workspace. DPI is
//! swept because a chrome length left in device pixels looks correct at exactly
//! one scale and this composition is nothing but chrome lengths; the canvas
//! roster is swept because the workspace's wide/narrow decision is a real
//! second geometry, not a smaller first one.
//!
//! # What is NOT here
//!
//! Live theme SWITCHING. `docs/harness-reach.md`: a capture (and a test that
//! renders through one freshly built pipeline) witnesses the state a pipeline
//! was BUILT with, never the state it was later re-seeded with. These laws build
//! their pipeline once and set the world through the same door production's
//! construction path does, so they see the composition; they do not see a
//! mid-session `Cmd-T` into a `Ruled` world.

use super::super::*;
use super::pixeldiff::render_frame;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};
use theme::RuleSelection::{Gutter, Weight};

/// The canvases every geometry sweep below runs. Two comfortably wide, one at
/// the workspace's staging threshold, one narrow enough to stage it — so no law
/// here is only true on a desktop.
const CANVASES: &[(u32, u32)] = &[(1400, 900), (1100, 760), (860, 900), (620, 820)];

/// Both treatments, always swept together: the fork is one decision in one
/// function and a law that graded only the shipped arm would let the other rot.
const MARKS: [theme::RuleSelection; 2] = [Weight, Gutter];

// ---------------------------------------------------------------------------
// Pixel helpers — the local copies this repo accepts per file (the same shape
// `list_surfaces.rs` / `syntax_roles.rs` carry).
// ---------------------------------------------------------------------------

fn avg(pixels: &[[u8; 4]], w: i64, h: i64, x: i64, y: i64, rw: i64, rh: i64) -> theme::Srgb {
    let (x0, y0) = (x.max(0), y.max(0));
    let (x1, y1) = ((x + rw).min(w), (y + rh).min(h));
    let mut s = [0u64; 3];
    let mut n = 0u64;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let p = pixels[(yy * w + xx) as usize];
            s[0] += p[0] as u64;
            s[1] += p[1] as u64;
            s[2] += p[2] as u64;
            n += 1;
        }
    }
    assert!(n > 0, "empty sample region");
    theme::Srgb::rgb((s[0] / n) as u8, (s[1] / n) as u8, (s[2] / n) as u8)
}

fn redmean(a: theme::Srgb, b: theme::Srgb) -> f32 {
    let rbar = (a.r as f32 + b.r as f32) * 0.5;
    let dr = a.r as f32 - b.r as f32;
    let dg = a.g as f32 - b.g as f32;
    let db = a.b as f32 - b.b as f32;
    ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
        .sqrt()
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// A card of `kind` with `n` real candidate rows, built the way `sync_view`
/// builds one: lenses only where the kind actually facets, the spell popup's own
/// contextual shape where that is the kind. The selected row is deliberately in
/// the MIDDLE of the list, so a selected row always has a neighbour above AND
/// below and the "who owns this boundary" arm is never vacuous.
fn rules_view(kind: OverlayKind, n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\nthird line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = n / 2;
    v.overlay_hint = "type to filter".into();
    if crate::facets::scheme(kind).is_some() {
        v.overlay_lens = vec![
            ("All".into(), true),
            ("File".into(), false),
            ("Edit".into(), false),
        ];
    }
    if let Some(shape) = kind.workspace_shape() {
        v.overlay_workspace = true;
        v.overlay_rows_primary = shape.rows_are_primary();
        // A rail needs entries, and a faceting scheme is what supplies them —
        // a kind whose scheme is `None` would draw no rail at all.
        if v.overlay_lens.is_empty() {
            v.overlay_lens = vec![("All".into(), true), ("Editor".into(), false)];
        }
        // THE ROWS STAGE HOLDS FOCUS. A workspace has two regions and only one
        // of them carries its mark at full presence; these laws grade the ROW
        // list, so the row list is the region that must be live. The other
        // stage — where the pane's mark is dimmed and the RAIL's is the live one
        // — is swept by `the_workspace_rail_is_ruled_...`, which drives both.
        v.overlay_detail_focus = true;
    }
    if kind == OverlayKind::Spell {
        v.overlay_spell = Some((0, 0, 5));
        v.overlay_items = (0..n.min(5)).map(|i| format!("suggest{i}")).collect();
        v.overlay_bindings = Vec::new();
        v.overlay_selected = v.overlay_items.len() / 2;
        v.overlay_hint = String::new();
        v.overlay_lens = Vec::new();
        v.overlay_workspace = false;
    }
    v
}

/// The CONTENT rows a ruled list arranges — the same filter the production
/// emitter applies, so a law counts the boundaries the frame counted.
fn content_rows(
    plan: &crate::render::plan::OverlayRowPlan,
) -> Vec<crate::render::plan::PlannedRow> {
    plan.rows()
        .iter()
        .filter(|r| r.item.is_some())
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// LAW 1 — the full `OverlayKind` sweep: a rule is a rule, never a surface
// ---------------------------------------------------------------------------

/// GRADE ONE RENDERED `Ruled` CARD against the whole composition: the two
/// authored weights, the no-scrim/no-pane claim, boundary continuity, the two
/// spans, and that the selection is marked exactly once. `None` when this cell
/// draws no list at all (a staged workspace region, an empty popup); otherwise
/// `(rules graded, a selection was marked)`.
///
/// Extracted rather than inlined because three laws below need the same reading
/// of a frame and a fourth grades one setting row at a time — one owner of "what
/// does this frame's row list look like" beats four that could drift.
fn grade_ruled_card(
    p: &mut TextPipeline,
    cw: u32,
    mark: theme::RuleSelection,
    ctx: &str,
) -> Option<(usize, bool)> {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let rows = content_rows(&plan);
    if rows.is_empty() {
        return None;
    }
    let pitch = plan.lh();
    let (hair, heavy) = p.rule_weights();
    let quads = p.overlay_row_surfaces_probe();

    // NO SCRIM, NO PANE. Both are objects; this style draws none.
    assert_eq!(
        p.panel_card.instance_count(),
        0,
        "{ctx}: a rule carries NO scrim — padding one out on every side is exactly how it \
         becomes the plate this style refuses"
    );
    assert_eq!(
        p.float_card.instance_count(),
        0,
        "{ctx}: a Ruled world floats no pane — enclosure is the one thing the style refuses"
    );

    for q in &quads {
        assert!(
            (q[3] - hair).abs() < 0.01 || (q[3] - heavy).abs() < 0.01,
            "{ctx}: quad {q:?} is {}px tall — a `Ruled` list emits only its two authored \
             weights (hairline {hair}, selected {heavy})",
            q[3]
        );
        assert!(
            q[3] < pitch * 0.5,
            "{ctx}: quad {q:?} approaches the row pitch {pitch} — a row-tall quad IS a filled \
             band, whatever it is called"
        );
    }

    // EVERY INTERIOR BOUNDARY CARRIES EXACTLY ONE RULE. The boundary is the row
    // slot's own edge and a rule straddles it, so a rule "covers" a boundary
    // when the boundary falls inside its own band.
    let covering = |y: f32| -> usize {
        quads
            .iter()
            .filter(|q| y >= q[1] - 0.51 && y <= q[1] + q[3] + 0.51)
            .count()
    };
    let mut boundaries = vec![rows[0].top];
    boundaries.extend(rows.iter().map(|r| r.bottom()));
    for &y in &boundaries {
        assert_eq!(
            covering(y),
            1,
            "{ctx}: boundary y={y} is covered by {} rules — every boundary between two rows is \
             drawn exactly once (a hairline laid under a heavy rule is two marks pretending to \
             be one)",
            covering(y)
        );
    }

    // THE TWO SPANS. A hairline runs the text measure; a `Weight` selection's
    // heavy rule runs the full band, and that difference in REACH is half of
    // what `Weight` says.
    let (measure_x, measure_w) = (geom.text_left, geom.text_w.max(1.0));
    let (band_x, band_w) = (geom.band_x_probe(), geom.band_w_probe().max(1.0));
    for q in &quads {
        match ((q[3] - heavy).abs() < 0.01, mark) {
            (true, Weight) => assert!(
                (q[0] - band_x).abs() < 0.51 && (q[2] - band_w).abs() < 0.51,
                "{ctx}: a selected rule {q:?} must run the full band [{band_x}, {band_w}] — \
                 running out past the text measure is what makes the mark visible without \
                 filling anything"
            ),
            // A `Gutter` mark is a short segment hanging BESIDE the measure, so
            // it is heavy and deliberately short.
            (true, Gutter) => assert!(
                q[0] + q[2] <= measure_x + 0.51,
                "{ctx}: a Gutter mark {q:?} must hang in the gutter, left of the text measure \
                 at x={measure_x}"
            ),
            (false, _) => assert!(
                (q[0] - measure_x).abs() < 0.51 && (q[2] - measure_w).abs() < 0.51,
                "{ctx}: hairline {q:?} must run the text measure [{measure_x}, {measure_w}] so \
                 a rule and the label above it start and stop together"
            ),
        }
    }

    // THE SELECTION IS MARKED AT ALL, and marked once.
    let heavies = quads.iter().filter(|q| (q[3] - heavy).abs() < 0.01).count();
    let want = match mark {
        Weight => 2, // the two rules bounding the selected row
        Gutter => 1, // one segment in the gutter
    };
    let marked = plan.selected_display().is_some();
    if marked {
        assert_eq!(
            heavies, want,
            "{ctx}: a selected row must carry exactly {want} heavy rule(s), got {heavies}"
        );
    }
    Some((quads.len(), marked))
}

/// THE MARK IS ON THE SELECTED ROW'S OWN SLOT — the arm that turns "one mark of
/// the right weight" into "the right row is marked". Shared by the picker sweep
/// and the settings sweep so the two cannot drift about what bounding means.
fn assert_mark_sits_on_the_selected_row(
    p: &mut TextPipeline,
    cw: u32,
    mark: theme::RuleSelection,
    ctx: &str,
) {
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let Some(sel) = plan.selected_display() else {
        return;
    };
    let rows = content_rows(&plan);
    let Some(slot) = rows.iter().find(|r| r.display == sel) else {
        return;
    };
    let (_, heavy) = p.rule_weights();
    let mut marks: Vec<[f32; 4]> = p
        .overlay_row_surfaces_probe()
        .into_iter()
        .filter(|q| (q[3] - heavy).abs() < 0.01)
        .collect();
    marks.sort_by(|a, b| a[1].total_cmp(&b[1]));
    match mark {
        Weight => {
            assert_eq!(marks.len(), 2, "{ctx}: two bounding rules");
            let top = marks[0][1] + marks[0][3] * 0.5;
            let bot = marks[1][1] + marks[1][3] * 0.5;
            assert!(
                (top - slot.top).abs() < 0.51 && (bot - slot.bottom()).abs() < 0.51,
                "{ctx}: the rules bound [{top}, {bot}], the row is [{}, {}] — the mark and the \
                 row must be the same slot",
                slot.top,
                slot.bottom()
            );
        }
        Gutter => {
            assert_eq!(marks.len(), 1, "{ctx}: one gutter segment");
            let cy = marks[0][1] + marks[0][3] * 0.5;
            assert!(
                cy > slot.top - 0.51 && cy < slot.bottom() + 0.51,
                "{ctx}: the gutter mark at y={cy} must hang beside its own row [{}, {}]",
                slot.top,
                slot.bottom()
            );
        }
    }
}

/// THE HEADLINE SWEEP. On every `OverlayKind`, both treatments, both DPIs and
/// four canvases, every quad a `Ruled` frame emits for its row list is a RULE:
///
/// * its height is exactly one of the composition's two authored weights,
/// * it never approaches a row's own height (a row-tall quad IS a filled band,
///   whatever it is called),
/// * it carries no scrim (`panel_card`) and floats no pane (`float_card`),
/// * every interior boundary between two consecutive content rows carries
///   EXACTLY ONE rule — no gap, and never a hairline laid under a heavy rule,
/// * and (`Weight`) the selected row's two bounding rules run the full band
///   while every hairline runs only the text measure.
///
/// The axis this sweeps that the composition's author did not: `Gutter` as well
/// as the shipped `Weight`, DPI 2 as well as 1, and the workspace canvases where
/// the band is a PANE rather than a card.
#[test]
fn every_overlay_kinds_rules_are_rules_and_never_a_surface() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping every_overlay_kinds_rules_are_rules_and_never_a_surface: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true); // settle the entrance: no travelling band

    let mut graded_cards = 0usize;
    let mut graded_rules = 0usize;
    let mut graded_selected = 0usize;
    for mark in MARKS {
        theme::set_active_by_name("Paperbark").unwrap();
        p.sync_theme();
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for kind in OverlayKind::ALL {
                    let v = rules_view(kind, 12);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let ctx = format!("{kind:?} mark={mark:?} dpi={dpi} canvas={cw}x{ch}");
                    let Some((rules, selected)) = grade_ruled_card(&mut p, cw, mark, &ctx) else {
                        continue; // a stage that draws no list draws no rules
                    };
                    graded_rules += rules;
                    graded_selected += usize::from(selected);
                    graded_cards += 1;
                }
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    // NON-VACUITY, three ways: the sweep really rendered every kind on every
    // cell, the cards really emitted rules, and a selection really was marked.
    let cells = MARKS.len() * 2 * CANVASES.len() * OverlayKind::ALL.len();
    assert!(
        graded_cards * 10 >= cells * 9,
        "the sweep must grade essentially every cell — {graded_cards} of {cells}"
    );
    assert!(
        graded_rules > 2000,
        "the sweep must grade thousands of real rules, got {graded_rules}"
    );
    assert!(
        graded_selected > 200,
        "the sweep must grade hundreds of real selections, got {graded_selected}"
    );
}

// ---------------------------------------------------------------------------
// LAW 2 — drawn ↔ hit-test ↔ sidecar, at both DPIs
// ---------------------------------------------------------------------------

/// DESIGN.md §8: drawn geometry and hit-test geometry have one owner. Under
/// `Weight` the mark is a PAIR OF BOUNDARIES rather than a filled row, so "where
/// the row is" is stated by two rules with nothing between them — which is
/// the same shape a staggered row's own hit-test wore, from the other side. A
/// pointer inside the band the two heavy rules bound must resolve to the row the
/// SIDECAR reports selected; a pointer just outside must resolve to its
/// neighbour, not to it.
///
/// Three independent owners are compared, never one accessor read twice: the
/// DRAWN rules (`overlay_row_surfaces_probe`, out of the selection emitter), the
/// INTERACTIVE answer (`overlay_row_at`, out of the planner's inverse), and the
/// STATE answer (`ViewState::overlay_selected`, which is what the sidecar
/// serialises).
#[test]
fn the_row_a_rules_selection_bounds_is_the_row_the_pointer_and_the_sidecar_agree_on() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the_row_a_rules_selection_bounds...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();
    set_list_style_test_override(Some(theme::ListStyle::Ruled(Weight)));

    let mut graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        for &(lw, lh) in CANVASES {
            let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for kind in OverlayKind::ALL {
                let v = rules_view(kind, 12);
                let state_selected = v.overlay_selected;
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{kind:?} dpi={dpi} canvas={cw}x{ch}");
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let rows = content_rows(&plan);
                let Some(sel_disp) = plan.selected_display() else {
                    continue;
                };
                let Some(sel_row) = rows.iter().find(|r| r.display == sel_disp) else {
                    continue;
                };
                let (_, heavy) = p.rule_weights();
                let mut marks: Vec<[f32; 4]> = p
                    .overlay_row_surfaces_probe()
                    .into_iter()
                    .filter(|q| (q[3] - heavy).abs() < 0.01)
                    .collect();
                marks.sort_by(|a, b| a[1].total_cmp(&b[1]));
                assert_eq!(
                    marks.len(),
                    2,
                    "{ctx}: `Weight` marks a row with exactly its two bounding rules"
                );

                // THE DRAWN CLAIM: the band between the two rules' own centres.
                let top = marks[0][1] + marks[0][3] * 0.5;
                let bottom = marks[1][1] + marks[1][3] * 0.5;
                assert!(
                    (top - sel_row.top).abs() < 0.51 && (bottom - sel_row.bottom()).abs() < 0.51,
                    "{ctx}: the rules bound [{top}, {bottom}] but the planned row is \
                     [{}, {}] — the mark and the row must be the same slot",
                    sel_row.top,
                    sel_row.bottom()
                );

                // THE INTERACTIVE CLAIM: a pointer between them selects that row.
                let mid_x = geom.text_left + geom.text_w * 0.5;
                for t in [0.15f32, 0.5, 0.85] {
                    let y = top + (bottom - top) * t;
                    assert_eq!(
                        p.overlay_row_at(mid_x, y),
                        sel_row.item,
                        "{ctx}: a pointer at ({mid_x:.1}, {y:.1}) — inside the band the two \
                         drawn rules bound — must resolve to the row they bound"
                    );
                }
                // …and just OUTSIDE them it does not. A boundary shared with a
                // neighbour resolves to the neighbour, never to nothing and
                // never to both.
                for (probe_y, side) in [(top - 1.5, "above"), (bottom + 1.5, "below")] {
                    if let Some(hit) = p.overlay_row_at(mid_x, probe_y) {
                        assert_ne!(
                            Some(hit),
                            sel_row.item,
                            "{ctx}: a pointer {side} the selected row's own bounding rule still \
                             resolves to it — the mark claims more than it draws"
                        );
                    }
                }

                // THE STATE CLAIM: the item the rules bound is the one the
                // sidecar reports, mapped through the same window the plan used.
                assert_eq!(
                    sel_row.item,
                    Some(state_selected),
                    "{ctx}: the rules bound item {:?} while the state (and so the sidecar) \
                     reports {state_selected}",
                    sel_row.item
                );
                graded += 1;
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 100,
        "the agreement sweep must reach every kind on every cell, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 3 — the row's interior is plain ground, in real pixels
// ---------------------------------------------------------------------------

/// THE ONE THING NEITHER TREATMENT IS ALLOWED TO BECOME, asserted over pixels
/// rather than over the rects that intended to draw them (CLAUDE.md's Wagtail
/// tripwire — the sidecar once reported `selected_index: 2` while the row
/// rendered fully invisible).
///
/// The measurement is a DIFFERENCE OF TWO FRAMES over one row, not a comparison
/// of two rows in one frame: row `k` is rendered once while it is selected and
/// once while a distant row is, and the pixels of its own interior — everything
/// strictly between its two boundaries, across the whole band — are compared.
/// Reading two different rows in one frame cannot work here: they carry
/// different labels, so their pixels differ for reasons that have nothing to do
/// with the selection. This way the ONLY thing that changed is whether the row
/// is selected, and the fraction of its interior that moved is exactly how much
/// of the row the mark filled.
///
/// A `Ruled` selection may move a sliver — the `Gutter` segment genuinely hangs
/// inside the row's vertical extent, in the gutter column — but never a BAND.
/// NON-VACUITY IS THE POSITIVE CONTROL: the identical measurement on a `Pane`
/// world must fill essentially the whole interior, so a probe that sampled the
/// wrong strip (or a frame that rendered nothing) cannot pass this by accident.
#[test]
fn a_rules_selection_fills_none_of_its_rows_interior_and_a_panes_fills_it_all() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping a_rules_selection_fills_none_of_its_rows_interior...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    /// What FRACTION of row `k`'s own interior changes when the selection moves
    /// onto it, and how many pixels that interior holds (so a zero fraction over
    /// an empty region can never read as a pass).
    fn interior_fill(
        p: &mut TextPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        w: u32,
        h: u32,
        k: usize,
    ) -> (f32, usize) {
        let render_with = |p: &mut TextPipeline, sel: usize| {
            let mut v = rules_view(OverlayKind::Command, 12);
            v.overlay_selected = sel;
            p.set_view(&v);
            p.prepare(device, queue, w, h).unwrap();
            render_frame(p, device, queue, w, h)
        };
        // FRAME B first, so the geometry read afterwards is row `k` SELECTED —
        // the frame whose mark this law is about.
        let far = if k >= 6 { 0 } else { 11 };
        let b = render_with(p, far);
        let a = render_with(p, k);
        let geom = p.overlay_geometry(w);
        let plan = p.overlay_row_plan(&geom);
        let slot = content_rows(&plan)
            .into_iter()
            .find(|r| r.item == Some(k))
            .expect("row k is windowed");
        // Strictly INSIDE the row: past the heaviest rule either boundary can
        // carry, so a boundary's own ink is never counted as interior fill.
        let (_, heavy) = p.rule_weights();
        let inset = heavy + 1.0;
        let (y0, y1) = ((slot.top + inset) as i64, (slot.bottom() - inset) as i64);
        let (x0, x1) = (
            geom.band_x_probe() as i64,
            (geom.band_x_probe() + geom.band_w_probe()) as i64,
        );
        let (mut moved, mut total) = (0usize, 0usize);
        for yy in y0.max(0)..y1.min(h as i64) {
            for xx in x0.max(0)..x1.min(w as i64) {
                let i = (yy * w as i64 + xx) as usize;
                total += 1;
                if (0..3).any(|c| a[i][c] != b[i][c]) {
                    moved += 1;
                }
            }
        }
        (moved as f32 / total.max(1) as f32, total)
    }

    let mut fills = Vec::new();
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for k in [2usize, 6, 9] {
            let (frac, total) = interior_fill(&mut p, &device, &queue, w, h, k);
            assert!(
                total > 5000,
                "Ruled({mark:?}) row {k}: the interior sample must be a real region, got \
                 {total} px — a law over nothing passes over anything"
            );
            assert!(
                frac < 0.05,
                "Ruled({mark:?}) row {k}: selecting it repainted {:.1}% of its own interior \
                 ({total} px sampled) — the row's interior must stay plain ground. A filled \
                 band is `Pane`'s answer, and borrowing it makes this a restyle of that.",
                frac * 100.0
            );
            fills.push(frac);
        }
    }

    // THE POSITIVE CONTROL. The same measurement, the same fixture, the same
    // sample window — on the style whose answer IS a filled band.
    set_list_style_test_override(Some(theme::ListStyle::Pane));
    let (pane, pane_total) = interior_fill(&mut p, &device, &queue, w, h, 6);
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        pane > 0.9,
        "NON-VACUITY: the identical measurement on a `Pane` world must find its filled band \
         over essentially the whole interior (got {:.1}% of {pane_total} px); if it does not, \
         this law is measuring the wrong region and its `Ruled` arms ({fills:?}) prove nothing",
        pane * 100.0
    );
}

// ---------------------------------------------------------------------------
// LAW 4 — the mark is findable, and it is not the accent
// ---------------------------------------------------------------------------

/// A MARK NOBODY CAN SEE IS NOT A MARK (the Wagtail lesson), and a mark in the
/// accent colour breaks DESIGN.md §3's one-accent rule — the caret is the only
/// accent awl has. Both halves are measured on real pixels: the heavy rule's own
/// band must be a clear value step from the ground beside it, and the ink it
/// draws with must not be the world's `primary`.
///
/// Swept over both treatments and both DPIs, because a rule floored at one
/// device pixel and a rule three logical pixels thick are different measurements
/// at 2× and only one of them was tuned.
#[test]
fn the_rules_mark_is_a_findable_step_of_page_ink_and_never_the_accent() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the_rules_mark_is_a_findable_step...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();
    let accent = theme::primary();
    let ink = theme::base_content();
    // The mark is the page's own content ink, never a second accent. Asserted at
    // the token level so the claim survives a world whose accent happens to be
    // near its ink.
    assert!(
        redmean(ink, accent) > 40.0,
        "the carrier world's ink {ink:?} and accent {accent:?} must be distinguishable, or \
         the no-accent arm below is vacuous"
    );

    let mut graded = 0usize;
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1400.0 * dpi) as u32, (900.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            // Three surfaces, not one: a flat picker, a faceted one whose strip
            // pushes the list down, and the workspace, whose band is a PANE.
            for kind in [
                OverlayKind::Command,
                OverlayKind::Goto,
                OverlayKind::Settings,
            ] {
                let v = rules_view(kind, 12);
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let (_, heavy) = p.rule_weights();
                let marks: Vec<[f32; 4]> = p
                    .overlay_row_surfaces_probe()
                    .into_iter()
                    .filter(|q| (q[3] - heavy).abs() < 0.01)
                    .collect();
                assert!(
                    !marks.is_empty(),
                    "{kind:?} mark={mark:?} dpi={dpi}: no heavy rule drawn"
                );
                let px = render_frame(&mut p, &device, &queue, cw, ch);
                for q in &marks {
                    // The rule's own middle third, and the ground a row's pitch
                    // away from it at the same x — so the comparison is the rule
                    // against what it is drawn ON, not against a neighbouring rule.
                    let sx = (q[0] + q[2] * 0.35) as i64;
                    let sw = (q[2] * 0.3).max(2.0) as i64;
                    let rule_px = avg(
                        &px,
                        cw as i64,
                        ch as i64,
                        sx,
                        (q[1] + q[3] * 0.5) as i64,
                        sw,
                        1,
                    );
                    let ground = avg(
                        &px,
                        cw as i64,
                        ch as i64,
                        sx,
                        (q[1] + plan.lh() * 0.45) as i64,
                        sw,
                        1,
                    );
                    let step = redmean(rule_px, ground);
                    assert!(
                        step >= 25.0,
                        "{kind:?} mark={mark:?} dpi={dpi}: the selection rule {q:?} reads \
                     {rule_px:?} against ground {ground:?} — only redmean {step:.1}. A mark \
                     nobody can find is not a mark."
                    );
                    let to_accent = redmean(rule_px, accent);
                    let to_ink = redmean(rule_px, ink);
                    assert!(
                        to_ink < to_accent,
                        "{kind:?} mark={mark:?} dpi={dpi}: the rule reads {rule_px:?}, nearer \
                     the accent {accent:?} (redmean {to_accent:.1}) than the page ink \
                     {ink:?} ({to_ink:.1}) — one accent, and it is the caret (DESIGN §3)"
                    );
                    graded += 1;
                }
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    // `Weight` marks with two rules, `Gutter` with one: 3 surfaces x 2 DPIs x
    // (2 + 1) = 18 marks, and every one of them graded.
    assert!(
        graded >= 18,
        "the pixel sweep must grade every mark, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 5 — nothing is drawn outside the band it belongs to
// ---------------------------------------------------------------------------

/// A rule that runs out past the text measure is the point of `Weight`, and it
/// is one arithmetic slip from running out past the CARD. Every quad a `Ruled`
/// frame emits must lie inside the band it belongs to — horizontally within the
/// card/pane (the `Gutter` mark included, which hangs left of the measure but
/// still inside the band) and vertically within the card's own box.
#[test]
fn no_rule_is_drawn_outside_the_card_that_owns_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping no_rule_is_drawn_outside_the_card_that_owns_it: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();
    let mut graded = 0usize;
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for kind in OverlayKind::ALL {
                    let v = rules_view(kind, 12);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let geom = p.overlay_geometry(cw);
                    let ctx = format!("{kind:?} mark={mark:?} dpi={dpi} canvas={cw}x{ch}");
                    let (bx, bw) = (geom.band_x_probe(), geom.band_w_probe());
                    let [_, cy, _, chh] = geom.card_probe();
                    for q in p.overlay_row_surfaces_probe() {
                        assert!(
                            q[0] >= bx - 0.51 && q[0] + q[2] <= bx + bw + 0.51,
                            "{ctx}: rule {q:?} runs outside its band [{bx}, {bw}]"
                        );
                        assert!(
                            q[1] >= cy - 0.51 && q[1] + q[3] <= cy + chh + 0.51,
                            "{ctx}: rule {q:?} runs outside the card's vertical box \
                             [{cy}, {chh}]"
                        );
                        graded += 1;
                    }
                }
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 2000,
        "the clip sweep must grade thousands of rules, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 6 — the workspace RAIL is ruled, not banded
// ---------------------------------------------------------------------------

fn settings_values() -> crate::settings::SettingsValues {
    super::settings_values(1.0, 1.0)
}

/// A REAL Settings workspace, standing on `lens`, with `detail` deciding which
/// region holds focus — built exactly as `overlay::build`'s Settings arm builds
/// one, and focused through the LIFECYCLE rather than by assignment.
fn settings_card(lens: usize, detail: bool) -> OverlayState {
    let vals = settings_values();
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_secondaries(crate::settings::visible_value_cells(&vals));
    ov.set_range_cells(crate::settings::visible_range_cells(&vals));
    ov.set_facet_lens(lens);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    if detail {
        journey.toggle_detail();
    }
    journey.card().expect("the card is up").clone()
}

/// Fold a workspace card into a `ViewState` the way `App::sync_view` does.
fn settings_view(ov: &OverlayState, selected: usize) -> ViewState {
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Settings.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_ranges = ov.item_range_fracs();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    v.overlay_sections = ov.item_sections();
    v.overlay_hint = ov.foot_hint();
    v.overlay_selected = selected.min(ov.item_strings().len().saturating_sub(1));
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = ov.window_rows();
    v
}

/// GRADE ONE RENDERED `Ruled` RAIL. Every quad is one of the two authored
/// weights and none approaches a rail row's height; a `Weight` selection bounds
/// the active entry with exactly two rules running the full column while every
/// hairline stops at the label measure; and — the arm that matters most — what
/// the FRAME UPLOADED matches what the composition owner says, because a law
/// that only read `workspace_rail_rule_ink` stays green while the draw path
/// stops calling it. That is measured rather than assumed: this file's rail arm
/// did exactly that against a mutation reverting the rail to its filled band.
fn grade_ruled_rail(
    p: &mut TextPipeline,
    rail: &[([f32; 4], bool)],
    mark: theme::RuleSelection,
    ctx: &str,
) {
    let row_h = rail[0].0[3];
    let (ghosts, quads) = p.workspace_rail_rule_ink(mark);
    let (hair, heavy) = p.rule_weights();
    for q in quads.iter().chain(ghosts.iter()) {
        assert!(
            (q[3] - hair).abs() < 0.01 || (q[3] - heavy).abs() < 0.01,
            "{ctx}: rail quad {q:?} is neither of the two authored rule weights ({hair}, \
             {heavy}) — the rail's mark must be made of the list's own substance"
        );
        assert!(
            q[3] < row_h * 0.5,
            "{ctx}: rail quad {q:?} approaches a rail row's height {row_h} — a filled band is \
             the one thing this style refuses, and the rail is not exempt"
        );
    }
    if let (Some(&(rect, _)), Weight) = (rail.iter().find(|(_, a)| *a), mark) {
        assert_eq!(
            quads.len(),
            2,
            "{ctx}: the active rail entry is bounded by exactly its two rules"
        );
        let mut ys: Vec<f32> = quads.iter().map(|q| q[1] + q[3] * 0.5).collect();
        ys.sort_by(f32::total_cmp);
        assert!(
            (ys[0] - rect[1]).abs() < 0.51 && (ys[1] - (rect[1] + rect[3])).abs() < 0.51,
            "{ctx}: the rail's rules bound {ys:?} but its active entry is [{}, {}]",
            rect[1],
            rect[1] + rect[3]
        );
        // THE RUN-OUT: the mark reaches the full column while a hairline stops
        // at the label measure, exactly as the pane's do.
        for q in &quads {
            assert!(
                (q[2] - rect[2]).abs() < 0.51,
                "{ctx}: the rail's heavy rule {q:?} must run the full column (w {})",
                rect[2]
            );
        }
        for g in &ghosts {
            assert!(
                g[2] < rect[2] - 0.51,
                "{ctx}: a rail hairline {g:?} must stop at the label measure, short of the \
                 column (w {})",
                rect[2]
            );
        }
    }
    assert_eq!(
        p.overlay_lens_underline.instance_count() as usize,
        quads.len(),
        "{ctx}: the frame uploaded {} selection quads for a rail the composition says has {} — \
         the draw path is not reading the rules owner",
        p.overlay_lens_underline.instance_count(),
        quads.len()
    );
    assert_eq!(
        p.overlay_facet_ghost.instance_count() as usize,
        ghosts.len(),
        "{ctx}: the frame uploaded {} hairlines for a rail the composition says has {}",
        p.overlay_facet_ghost.instance_count(),
        ghosts.len()
    );
}

/// THE HALF-APPLIED WORLD THIS ITEM EXISTS TO CLOSE.
///
/// A summoned workspace has TWO regions that both keep a selection, and the
/// rail's mark took the world's selected-row band unconditionally — a filled
/// band, which is the one thing this style refuses. Paperbark shipped with its
/// content pane arranged by rules and the rail beside it wearing a plate.
///
/// A rail IS a list, so the style that says how a list is arranged says how this
/// one is: on `Ruled` the rail's entries are separated by hairlines and its
/// active entry is bounded by two heavy rules running the rail's full column —
/// the very rect the band occupied. On every other style the rail keeps its
/// band, and that arm is the non-vacuity control: it proves this law is looking
/// at the rail at all.
#[test]
fn the_workspace_rail_is_ruled_on_a_rules_world_and_banded_on_every_other() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping the_workspace_rail_is_ruled...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let mut graded_rules = 0usize;
    let mut graded_banded = 0usize;
    // Every style, no wildcard: a fifth arm must decide what its rail does.
    let styles: [theme::ListStyle; 5] = [
        theme::ListStyle::Ruled(Weight),
        theme::ListStyle::Ruled(Gutter),
        theme::ListStyle::Pane,
        theme::ListStyle::Bars,
        theme::ListStyle::Diagonal(theme::DiagonalSpine::descending(theme::DiagonalMark::CRISP)),
    ];
    for style in styles {
        set_list_style_test_override(Some(style));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in CANVASES {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for lens in [0usize, 2, 5] {
                    for detail in [false, true] {
                        let ov = settings_card(lens, detail);
                        let v = settings_view(&ov, 3);
                        p.set_view(&v);
                        p.prepare(&device, &queue, cw, ch).unwrap();
                        let ctx = format!(
                            "{style:?} dpi={dpi} canvas={cw}x{ch} lens={lens} detail={detail}"
                        );
                        let rail = p.workspace_rail_rows_probe();
                        if rail.is_empty() {
                            continue; // the narrow stage showing its rows
                        }
                        match style {
                            theme::ListStyle::Ruled(mark) => {
                                grade_ruled_rail(&mut p, &rail, mark, &ctx);
                                graded_rules += 1;
                            }
                            // THE CONTROL. Every other style's rail keeps the
                            // filled band it always had — one rect, a full rail
                            // row tall.
                            theme::ListStyle::Pane
                            | theme::ListStyle::Bars
                            | theme::ListStyle::Diagonal(_) => {
                                if rail.iter().any(|(_, a)| *a) {
                                    let row_h = rail[0].0[3];
                                    let band = p.workspace_rail_mark().unwrap_or_else(|| {
                                        panic!("{ctx}: a banded rail marks its active entry")
                                    });
                                    assert!(
                                        (band[3] - row_h).abs() < 0.51,
                                        "{ctx}: the banded rail's mark {band:?} must be a full \
                                         rail row tall ({row_h})"
                                    );
                                    assert_eq!(
                                        p.overlay_lens_underline.instance_count(),
                                        1,
                                        "{ctx}: a banded rail uploads exactly its one band"
                                    );
                                    graded_banded += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded_rules > 20 && graded_banded > 20,
        "both arms must be reached — {graded_rules} ruled, {graded_banded} banded"
    );
}

/// THE RAIL'S OWN PIXELS. The law above reads the quads the frame uploaded,
/// which is one owner away from what a viewer sees; this one reads the frame.
///
/// A filled band covers its whole entry, INCLUDING the glyph-free slack the rail
/// column reserves past its longest label — `overlay_text_hpad` on each side, by
/// the column's own measurement. So: sample that slack strip inside the ACTIVE
/// entry, and the same strip inside an INACTIVE one, in one frame. On `Ruled`
/// they must read the same ground; on every banded style they must not. No
/// glyph can fall in that strip by construction, so the comparison is about the
/// mark alone.
#[test]
fn the_active_rail_entry_reads_as_plain_ground_on_a_rules_world_and_as_a_band_elsewhere() {
    let _g = crate::testlock::serial();
    let (w, h) = (1400u32, 900u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_active_rail_entry_reads_as_plain_ground...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    /// The redmean between the active rail entry's glyph-free slack and an
    /// inactive entry's, in one rendered frame.
    fn slack_delta(
        p: &mut TextPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        w: u32,
        h: u32,
    ) -> f32 {
        let ov = settings_card(0, false);
        let v = settings_view(&ov, 3);
        p.set_view(&v);
        p.prepare(device, queue, w, h).unwrap();
        let rail = p.workspace_rail_rows_probe();
        assert!(rail.len() >= 2, "the rail must have entries to compare");
        let active = rail.iter().find(|(_, a)| *a).expect("an active entry").0;
        let idle = rail
            .iter()
            .find(|(r, a)| !*a && (r[1] - active[1]).abs() > active[3] * 0.5)
            .expect("an inactive entry")
            .0;
        // The slack the column's own measurement reserves past its longest
        // label — a production number, never a guessed fraction of the width.
        let hpad = p.overlay_text_hpad();
        let x = (active[0] + active[2] - 2.0 * hpad + 3.0) as i64;
        let sw = (2.0 * hpad - 6.0).max(2.0) as i64;
        let inset = (active[3] * 0.3) as i64;
        let sh = (active[3] as i64 - 2 * inset).max(1);
        let px = render_frame(p, device, queue, w, h);
        let a = avg(&px, w as i64, h as i64, x, active[1] as i64 + inset, sw, sh);
        let b = avg(&px, w as i64, h as i64, x, idle[1] as i64 + inset, sw, sh);
        redmean(a, b)
    }

    let mut ruled = Vec::new();
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        let d = slack_delta(&mut p, &device, &queue, w, h);
        assert!(
            d <= 4.0,
            "Ruled({mark:?}): the active rail entry's slack reads redmean {d:.1} from an \\
             inactive one's — the rail's active entry must be plain ground with rules at its \\
             boundaries, not a filled band"
        );
        ruled.push(d);
    }
    // THE CONTROL, through the identical measurement on the styles whose rail
    // genuinely IS a band.
    let mut banded = Vec::new();
    for style in [theme::ListStyle::Pane, theme::ListStyle::Bars] {
        set_list_style_test_override(Some(style));
        banded.push(slack_delta(&mut p, &device, &queue, w, h));
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    for (style, d) in [("Pane", banded[0]), ("Bars", banded[1])] {
        assert!(
            d > 20.0,
            "NON-VACUITY: {style}'s rail band must be visible in the very same strip \\
             (redmean {d:.1}); if it is not, this law samples the wrong pixels and its \\
             `Ruled` arms ({ruled:?}) prove nothing"
        );
    }
}

// ---------------------------------------------------------------------------
// LAW 7 — every `SettingId × SettingKind`
// ---------------------------------------------------------------------------

/// EVERY SETTING ROW, RULED. `SettingKind` spans a toggle, a picker, a range
/// with a drawn rail, a path, a submenu and an action — six different row
/// CONTENTS through one row planner — and the composition has to hold for all of
/// them. A range row is the one that could genuinely differ: it draws a rail
/// (`overlay_range_track`/`_thumb`) INSIDE the row, and a law that only graded
/// toggles would never see a rule colliding with it.
///
/// Coverage is asserted against the registry itself rather than against a count:
/// every `SettingId` in `settings::SETTINGS` must have been graded, and every
/// `SettingKind` must have been reached, so a new setting cannot slip in
/// un-swept.
#[test]
fn every_setting_id_and_kind_is_ruled_in_the_settings_workspace() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping every_setting_id_and_kind_is_ruled...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let names: Vec<String> = crate::settings::visible_names();
    // `visible_names` is the drawn corpus in registry order, so a row's index is
    // its registry row — the one mapping this law needs, taken from the same
    // owner the picker takes it from.
    let registry: Vec<&crate::settings::SettingRow> = crate::settings::SETTINGS
        .iter()
        .filter(|r| names.iter().any(|n| n == r.name))
        .collect();
    assert_eq!(
        registry.len(),
        names.len(),
        "the visible corpus and the registry must line up 1:1 or the coverage claim below is \
         about the wrong rows"
    );

    let mut seen_ids = std::collections::BTreeSet::new();
    let mut seen_kinds = std::collections::BTreeSet::new();
    let mut graded = 0usize;
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for &(lw, lh) in &[(1400u32, 900u32), (620, 820)] {
                let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                p.set_size(cw as f32, ch as f32);
                for (idx, row) in registry.iter().enumerate() {
                    // The rows stage holds focus, so the pane's mark is at full
                    // strength and the row under test is genuinely selected.
                    let ov = settings_card(0, true);
                    let v = settings_view(&ov, idx);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let ctx = format!(
                        "{:?} ({:?}) mark={mark:?} dpi={dpi} {cw}x{ch}",
                        row.id, row.kind
                    );
                    // The WHOLE composition on this row, through the same reader
                    // every other law uses — weights, no scrim, boundary
                    // continuity, both spans, one mark.
                    let graded_card = grade_ruled_card(&mut p, cw, mark, &ctx);
                    let Some((_, marked)) = graded_card else {
                        continue;
                    };
                    assert!(marked, "{ctx}: the row under test must be the selected one");
                    // …and the mark sits on ITS OWN slot. This is the claim a
                    // RANGE row could break: its rail is drawn from a different
                    // owner at the same y, so a mark that took the rail's band
                    // instead of the row's would still be one mark of the right
                    // weight.
                    assert_mark_sits_on_the_selected_row(&mut p, cw, mark, &ctx);
                    seen_ids.insert(format!("{:?}", row.id));
                    seen_kinds.insert(format!("{:?}", row.kind));
                    graded += 1;
                }
            }
        }
    }
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    let want_ids: std::collections::BTreeSet<String> =
        registry.iter().map(|r| format!("{:?}", r.id)).collect();
    let want_kinds: std::collections::BTreeSet<String> =
        registry.iter().map(|r| format!("{:?}", r.kind)).collect();
    assert_eq!(
        seen_ids,
        want_ids,
        "every SettingId in the visible registry must have been graded — missing {:?}",
        want_ids.difference(&seen_ids).collect::<Vec<_>>()
    );
    assert_eq!(
        seen_kinds,
        want_kinds,
        "every SettingKind must have been reached — missing {:?}",
        want_kinds.difference(&seen_kinds).collect::<Vec<_>>()
    );
    assert!(
        graded > 200,
        "the sweep must grade every row on every cell, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 8 — the lens strip: a rule marks the active tab, and no pill is a plate
// ---------------------------------------------------------------------------

/// THE TAB DECISION, PINNED.
///
/// A tab PILL is a plate, so this style draws none — which leaves the question
/// the graduation had to answer: what says which tab is active? The answer is
/// the style's own vocabulary, and it was already there: `FacetStyle::Text`
/// marks the active lens with a hairline UNDER its label, which is a rule. The
/// strip is not bare; it is ruled, exactly like the list below it.
///
/// This law holds both halves at once, on every faceting `OverlayKind`: zero tab
/// plates, and exactly one active-lens mark whose height is hairline-class
/// rather than row-class. The `Bars` arm is the non-vacuity control — it draws a
/// pill per tab, so a law that found none everywhere would be measuring nothing.
#[test]
fn a_rules_lens_strip_marks_its_active_tab_with_a_rule_and_draws_no_pill() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping a_rules_lens_strip_marks_its_active_tab...: no adapter");
        return;
    };
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let faceting: Vec<OverlayKind> = OverlayKind::ALL
        .iter()
        .copied()
        .filter(|k| crate::facets::scheme(*k).is_some() && k.workspace_shape().is_none())
        .collect();
    assert!(
        faceting.len() >= 4,
        "the strip law needs the real faceting roster, got {faceting:?}"
    );

    let mut graded = 0usize;
    for mark in MARKS {
        set_list_style_test_override(Some(theme::ListStyle::Ruled(mark)));
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((1400.0 * dpi) as u32, (900.0 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for &kind in &faceting {
                let v = rules_view(kind, 12);
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{kind:?} mark={mark:?} dpi={dpi}");
                assert!(
                    p.overlay_strip_tab_plates.is_empty(),
                    "{ctx}: a tab pill is a plate and this style draws none"
                );
                let strip = p
                    .overlay_theme_underline
                    .unwrap_or_else(|| panic!("{ctx}: the active lens must be marked"));
                let lh = p.overlay_lh();
                assert!(
                    strip[3] < lh * 0.25,
                    "{ctx}: the active-lens mark {strip:?} is {}px tall against a row pitch of \
                     {lh} — a mark that thick is a pill, not a rule",
                    strip[3]
                );
                // AND IT IS VISIBLE, in real pixels, at BOTH scales. This is the
                // arm that makes the tab decision an evidence-backed one rather
                // than a structural claim: the strip's mark is `FacetStyle`'s
                // underline, whose height is a raw device-pixel `1.5` that does
                // NOT scale (measured: identical at DPI 1 and DPI 2 while every
                // other term doubles). A rule that thin could have vanished into
                // the ground on a Retina frame; it does not, and this says so by
                // arithmetic rather than by reading the constant.
                let px = render_frame(&mut p, &device, &queue, cw, ch);
                let sx = (strip[0] + strip[2] * 0.3) as i64;
                let sw = (strip[2] * 0.4).max(2.0) as i64;
                let on = avg(&px, cw as i64, ch as i64, sx, strip[1] as i64, sw, 1);
                let off = avg(
                    &px,
                    cw as i64,
                    ch as i64,
                    sx,
                    (strip[1] + strip[3] + 3.0) as i64,
                    sw,
                    1,
                );
                let step = redmean(on, off);
                assert!(
                    step >= 20.0,
                    "{ctx}: the active-lens underline {strip:?} reads {on:?} against the ground \
                     just below it {off:?} — only redmean {step:.1}. If the strip's one mark is \
                     invisible then a `Ruled` strip really is bare, and the tab question is \
                     still open."
                );
                assert!(
                    p.overlay_theme_facet_ghosts.is_empty(),
                    "{ctx}: an inactive tab gets no ghost pill either"
                );
                graded += 1;
            }
        }
    }
    // THE CONTROL: the same strip on a `Bars` world DOES draw a pill per tab, so
    // the emptiness above is a property of `Ruled` rather than of the fixture.
    set_list_style_test_override(Some(theme::ListStyle::Bars));
    p.set_dpi(1.0);
    p.set_size(1400.0, 900.0);
    let v = rules_view(OverlayKind::Command, 12);
    p.set_view(&v);
    p.prepare(&device, &queue, 1400, 900).unwrap();
    let pills = p.overlay_strip_tab_plates.len();
    set_list_style_test_override(None);
    crate::motion::set_reduced(saved_reduced);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        pills >= 3,
        "NON-VACUITY: a `Bars` world must draw a pill per tab (got {pills}); if it does not, \
         the `Ruled` arms above prove nothing"
    );
    assert!(
        graded >= 16,
        "the strip sweep must reach every faceting kind, got {graded}"
    );
}

// ---------------------------------------------------------------------------
// LAW 9 — the composition owner is shared, not copied
// ---------------------------------------------------------------------------

/// SAME BEHAVIOUR ⇒ SAME CODE. The picker rows and the workspace rail are the
/// same composition over different bands, and they came out of one function
/// precisely so a change to one cannot leave the other behind. This law asserts
/// that at the pure seam: handed the same rows and the same spans, the owner
/// produces the same rules regardless of which consumer asked — and that its
/// `Weight` arm really does REPLACE the two hairlines it covers rather than lay
/// a heavy rule over them.
#[test]
fn the_rules_owner_replaces_the_boundaries_it_claims_rather_than_covering_them() {
    let _g = crate::testlock::serial();
    use crate::render::chrome::overlay_rules::{RuleRow, RuleSpans, rules_ink};
    let spans = RuleSpans {
        hair: 1.0,
        heavy: 3.0,
        measure: (100.0, 200.0),
        band: (80.0, 240.0),
        mark: (13.0, 9.0),
    };
    let rows: Vec<RuleRow> = (0..4)
        .map(|i| RuleRow {
            top: 10.0 + i as f32 * 20.0,
            bottom: 30.0 + i as f32 * 20.0,
            selected: i == 1,
        })
        .collect();

    let (hair, heavy) = rules_ink(&rows, Weight, &spans);
    // Five boundaries in a four-row list; the selected row claims two of them,
    // so three hairlines remain and neither claimed boundary is drawn twice.
    assert_eq!(heavy.len(), 2, "the selected row's two bounding rules");
    assert_eq!(hair.len(), 3, "the three boundaries it did not claim");
    let claimed = [rows[1].top, rows[1].bottom];
    for h in &hair {
        let y = h[1] + h[3] * 0.5;
        assert!(
            claimed.iter().all(|c| (c - y).abs() > 0.01),
            "hairline at {y} is laid under a heavy rule — `Weight` REPLACES a boundary"
        );
        assert_eq!(
            (h[0], h[2]),
            spans.measure,
            "a hairline runs the text measure"
        );
    }
    for q in &heavy {
        assert_eq!((q[0], q[2]), spans.band, "a heavy rule runs the full band");
    }

    // ADJACENT SELECTED ROWS (what a live glide reads) share their boundary
    // once: three rules, not four.
    let glide: Vec<RuleRow> = (0..4)
        .map(|i| RuleRow {
            top: 10.0 + i as f32 * 20.0,
            bottom: 30.0 + i as f32 * 20.0,
            selected: i == 1 || i == 2,
        })
        .collect();
    let (_, heavy2) = rules_ink(&glide, Weight, &spans);
    assert_eq!(
        heavy2.len(),
        3,
        "two adjacent selected rows share their common boundary, which is drawn once"
    );

    // The `Gutter` arm touches no boundary at all: every hairline survives, and
    // the mark hangs left of the measure.
    let (hair_g, heavy_g) = rules_ink(&rows, Gutter, &spans);
    assert_eq!(hair_g.len(), 5, "Gutter leaves every boundary alone");
    assert_eq!(heavy_g.len(), 1, "one segment beside the selected row");
    assert!(
        heavy_g[0][0] + heavy_g[0][2] <= spans.measure.0,
        "the gutter mark hangs left of the text measure"
    );
}
