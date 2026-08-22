//! THE SCENE-PLAN LAWS AGAINST THE REAL PIPELINE.
//!
//! The pure planner laws live beside the planner (`render/plan/tests.rs`); this
//! file is the device-level half — the three answers that used to be computed in
//! parallel places must be ONE planned object:
//!
//! * DRAWN — the shaped candidate line's own y, read back from the buffer the
//!   draw pass uploads (never rebuilt from arithmetic).
//! * INTERACTIVE — what `overlay_row_at` accepts at that same y.
//! * SIDECAR — what `overlay_window_report` tells the harness about the window.
//!
//! Swept over the WHOLE `OverlayKind` roster (a no-wildcard match, so a new
//! picker cannot dodge it), both layout families, both list styles, several
//! window geometries and both DPIs — the axes a single-representative law would
//! have missed. Plus the derivation laws the plan makes possible: it is never
//! retained across a resize / zoom / buffer swap, its size is O(visible) not
//! O(doc), and the retired loose-scalar row arithmetic cannot come back.
//!
//! This file's own `family()` classifier used to be a HAND-COPIED
//! match that disagreed with the production owner it was meant to describe:
//! it called `OverlayKind::Assets` `Grouped`, but `facets::scheme(Assets)` is
//! `None` ("the asset cleaner is a flat list — no lens strip"). That mislabel
//! was not vacuous — the headline law's own fixture (`overlay_view`) reads
//! `family()` to decide whether to populate a lens strip, so `Assets` was fed
//! a lens strip production never grants it, and `overlay_geometry` dutifully
//! took the GROUPED path for a kind that can never reach it live; the sweep
//! graded real rows the whole time, just along a code path `Assets` cannot
//! take outside this test, while never exercising its real FLAT path. Fixed
//! by deriving `family()` from `facets::scheme` directly (the shared
//! `overlay_height_clamp_law.rs` already did this correctly — the shape to
//! follow); the headline law also checks the REAL pipeline's `geom.theme`
//! against `facets::scheme` independently, so a future drift fails by name
//! instead of silently regrading the wrong path again.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::overlay::OverlayKind;
use crate::render::chrome::OverlayGeom;

/// How a picker kind lays its candidate area out. NOT a hand-copied match over
/// `OverlayKind` — the classification lesson. `overlay_geometry` decides FLAT vs
/// GROUPED by asking exactly one question, `!self.overlay_lens.is_empty()`,
/// which at sync time is populated iff `crate::facets::scheme(kind)` is
/// `Some`. A hand-maintained copy of that split (the shape this file shipped
/// with, and `Assets` drifted from) can silently disagree with the function it
/// is supposed to describe; deriving it here instead means a kind that changes
/// family in production changes it in the law automatically. Production's own
/// `facets::scheme` is the no-wildcard match a new `OverlayKind` must join
/// before it compiles (see its doc in `src/facets.rs`), so the sweep still
/// cannot silently skip a new kind — the exhaustiveness lives at the one real
/// owner instead of a second copy of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    /// The plain candidate window — one display line per visible item.
    Flat,
    /// The lens-strip card — an explicit display-line sequence whose section
    /// headers push item rows down (`header_rows == 2`).
    Grouped,
    /// The contextual popup anchored at a misspelled word: no query line at all
    /// (`header_rows == 0`).
    Contextual,
}

fn family(kind: OverlayKind) -> Family {
    if kind == OverlayKind::Spell {
        return Family::Contextual;
    }
    if crate::facets::scheme(kind).is_some() {
        return Family::Grouped;
    }
    Family::Flat
}

/// A realistic view for `kind`: enough rows to fill a window, a right column, and
/// — for the grouped family — the real lens strip plus per-row section labels, so
/// the plan carries genuine section HEADERS interleaved with item rows.
fn overlay_view(kind: OverlayKind, n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = (0..n).map(|i| format!("candidate row {i}")).collect();
    v.overlay_bindings = (0..n).map(|i| format!("C-{}", i % 10)).collect();
    v.overlay_selected = (n / 3).min(n.saturating_sub(1));
    v.overlay_hint = "type to filter".into();
    match family(kind) {
        Family::Grouped => {
            v.overlay_lens = vec![
                ("All".into(), true),
                ("File".into(), false),
                ("Edit".into(), false),
            ];
            // Three contiguous groups, so the plan carries three real headers at
            // three different positions.
            v.overlay_sections = (0..n)
                .map(|i| match i * 3 / n.max(1) {
                    0 => "Alpha".to_string(),
                    1 => "Beta".to_string(),
                    _ => "Gamma".to_string(),
                })
                .collect();
        }
        Family::Contextual => {
            v.overlay_spell = Some((0, 0, 5));
            v.overlay_items = (0..n.min(5)).map(|i| format!("suggest{i}")).collect();
            v.overlay_bindings = Vec::new();
            v.overlay_selected = 0;
            v.overlay_hint = String::new();
        }
        Family::Flat => {}
    }
    v
}

/// Grade every planned row of one rendered frame: DRAWN (the shaped line's own
/// top, read off the buffer the draw pass uploads) == PLANNED slot, and
/// INTERACTIVE (`overlay_row_at` at the slot's OWN span — never the card's
/// constant edges, which a staggering composition's row does not sit on) ==
/// that row's own item. Returns `(item rows, header lines)` graded.
///
/// THE DIAGONAL ARM'S OWN FIX: this used to probe the CARD's fixed
/// `card_x_span()` for every row alike; that is exactly the undisplaced span
/// `PlannedRow`'s own doc names as a past regression ("a staggered row was
/// clickable where it was not drawn"), so testing it here would have
/// reintroduced the bug this file exists to catch. Each row's own
/// `row_x_span` already carries its `dx`/`dw` — `0.0` on `Pane`/`Bars`, so
/// this is a no-op there and only a staggering composition's rows actually
/// exercise the difference.
fn grade_rows(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    probe: &super::overlay_probe::OverlayYProbe,
    ctx: &str,
) -> (usize, usize) {
    let (mut items, mut headers) = (0usize, 0usize);
    for row in plan.rows() {
        let drawn = *probe.primary.get(&row.display).unwrap_or_else(|| {
            panic!(
                "{ctx}: display row {} must have a shaped line — the plan claims {} \
                 candidate rows",
                row.display,
                plan.candidate_rows()
            )
        });
        assert!(
            (drawn - row.top).abs() < 0.75,
            "{ctx}: display row {} is DRAWN at y {drawn} but PLANNED at {} (slot height {})",
            row.display,
            row.top,
            row.height
        );
        let mid_y = row.top + row.height * 0.5;
        let (rx0, rx1) = plan.row_x_span(row.display).unwrap_or_else(|| {
            panic!(
                "{ctx}: display row {} must have a clickable x-span",
                row.display
            )
        });
        let mid_x = (rx0 + rx1) * 0.5;
        for x in [rx0, mid_x, rx1] {
            assert_eq!(
                p.overlay_row_at(x, mid_y),
                row.item,
                "{ctx}: display row {} draws item {:?} but the pointer at ({x}, {mid_y}) \
                 (row span [{rx0}, {rx1}]) resolves differently",
                row.display,
                row.item
            );
        }
        match row.item {
            Some(_) => items += 1,
            None => headers += 1,
        }
    }
    (items, headers)
}

/// **THE PUBLISHED GEOMETRY IS THE THIRD READING.** `overlay_row_geometry()` is
/// exactly what the capture sidecar serializes into `overlay.window.band` /
/// `overlay.window.rows` (`capture::plan_sidecar`), and it is graded here against
/// the two oracles that DO NOT READ IT: the shaped line's own drawn y, taken off
/// the buffer the draw pass uploads, and whatever `overlay_row_at` accepts.
///
/// That independence is the whole point. Asserting the report against the plan
/// alone would be a tautology — both sides read the same accessor, so a report
/// that grew its own arithmetic would still have to be compared to the arithmetic
/// it copied. Grading it against ink and against the pointer means a second
/// derivation cannot hide: a published rect one pixel off the drawn row, or a
/// published span wider than the span the pointer honours, fails by name.
///
/// The outside-the-span arm is gated on `item.is_some()` deliberately: a section
/// heading resolves to `None` both inside and outside its own rect, so requiring
/// a DIFFERENT answer outside would be unsatisfiable for it rather than strict.
/// Returns the number of published rows graded.
fn grade_published_geometry(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    probe: &super::overlay_probe::OverlayYProbe,
    ctx: &str,
) -> usize {
    let g = p
        .overlay_row_geometry()
        .unwrap_or_else(|| panic!("{ctx}: an open card must publish its row geometry"));
    assert_eq!(
        g.rows.len(),
        plan.candidate_rows(),
        "{ctx}: the sidecar publishes {} row rects for a planned band of {}",
        g.rows.len(),
        plan.candidate_rows()
    );
    assert_eq!(
        (g.first_top, g.pitch, g.footer_top),
        (plan.first_top(), plan.lh(), plan.footer_top()),
        "{ctx}: the published band header disagrees with the plan it is a projection of"
    );
    let mut graded = 0usize;
    for (i, rect) in g.rows.iter().enumerate() {
        assert_published_row_matches_its_band(&g, i, plan, ctx);
        grade_published_row_against_ink_and_pointer(p, probe, rect, ctx);
        graded += 1;
    }
    graded
}

/// The published band's own INTERNAL consistency, before either independent
/// oracle: the rect describes the display line it claims to, it is not
/// degenerate, it meets the content band, and it is contiguous with its
/// predecessor.
fn assert_published_row_matches_its_band(
    g: &crate::render::plan::OverlayRowGeometry,
    i: usize,
    plan: &crate::render::plan::OverlayRowPlan,
    ctx: &str,
) {
    let rect = g.rows[i];
    let planned = plan.rows()[i];
    assert_eq!(
        (rect.display, rect.item),
        (planned.display, planned.item),
        "{ctx}: published row {i} claims display {}/item {:?}, the plan says {}/{:?}",
        rect.display,
        rect.item,
        planned.display,
        planned.item
    );
    // A PRESENCE floor before any agreement claim: a report that published empty
    // rects would satisfy every comparison by having nothing to disagree with,
    // and would still tell a reader nothing.
    assert!(
        rect.w > 1.0 && rect.h > 1.0,
        "{ctx}: published row {} is a {}x{} rect — a degenerate rect makes every \
         agreement assertion vacuous",
        rect.display,
        rect.w,
        rect.h
    );
    // OVERLAP, deliberately not containment: a selected row on a staggering
    // composition steps OUTWARD past the content band's own edge by design (the
    // real Saltpan Settings card publishes `x` 4px left of `band.x`), so "inside
    // the band" is a false law rather than a strict one. The span's exactness is
    // pinned by the pointer probes instead, which fail in BOTH directions: a
    // published span narrower than the real one puts the outside probes inside
    // it, and a wider one puts an inside probe out.
    assert!(
        rect.x + rect.w > g.band_x && rect.x < g.band_x + g.band_w,
        "{ctx}: published row {} spans [{}, {}], which does not meet its own \
         content band [{}, {}] at all",
        rect.display,
        rect.x,
        rect.x + rect.w,
        g.band_x,
        g.band_x + g.band_w
    );
    if i > 0 {
        let prev = g.rows[i - 1];
        assert!(
            (rect.y - (prev.y + prev.h)).abs() < 0.01,
            "{ctx}: published rows {} and {} leave a gap: {} + {} != {}",
            prev.display,
            rect.display,
            prev.y,
            prev.h,
            rect.y
        );
    }
}

/// THE TWO INDEPENDENT ORACLES, against one published rect: the ink and the
/// pointer, neither of which reads the report.
fn grade_published_row_against_ink_and_pointer(
    p: &TextPipeline,
    probe: &super::overlay_probe::OverlayYProbe,
    rect: &crate::render::plan::PlannedRowRect,
    ctx: &str,
) {
    // DRAWN — the ink, not a second calculation.
    let drawn = *probe
        .primary
        .get(&rect.display)
        .unwrap_or_else(|| panic!("{ctx}: published row {} must have ink", rect.display));
    assert!(
        drawn >= rect.y - 0.75 && drawn < rect.y + rect.h,
        "{ctx}: published row {} reports the slot [{}, {}) but its glyph line is \
         DRAWN at {drawn}",
        rect.display,
        rect.y,
        rect.y + rect.h
    );
    // INTERACTIVE — inside the published span, at both edges and the middle.
    let mid_y = rect.y + rect.h * 0.5;
    for px in [rect.x + 0.5, rect.x + rect.w * 0.5, rect.x + rect.w - 0.5] {
        assert_eq!(
            p.overlay_row_at(px, mid_y),
            rect.item,
            "{ctx}: the pointer at ({px}, {mid_y}) is INSIDE published row {}'s \
             rect [{}, {}]x[{}, {}] but does not resolve to its item {:?}",
            rect.display,
            rect.x,
            rect.x + rect.w,
            rect.y,
            rect.y + rect.h,
            rect.item
        );
    }
    if rect.item.is_none() {
        return;
    }
    for px in [rect.x - 1.5, rect.x + rect.w + 1.5] {
        assert_ne!(
            p.overlay_row_at(px, mid_y),
            rect.item,
            "{ctx}: the pointer at ({px}, {mid_y}) is OUTSIDE published row {}'s \
             span [{}, {}] yet still selects its item — the published rect is \
             narrower than the one the pointer honours",
            rect.display,
            rect.x,
            rect.x + rect.w
        );
    }
    // No `selected` assertion here, deliberately: the published rect carries no
    // selection. `window.sel_row` already reports it from the owner that also
    // colours the band, and a second answer projected from the plan's LOGICAL row
    // would disagree with the drawn one for the length of every selection move —
    // which is the visual-selection transaction, and what this file's sibling law
    // `no_render_path_reads_the_logical_selected_row_outside_the_transaction`
    // refuses. The geometry is this projection's whole claim.
}

/// The REAL pipeline's own path must match production's OWN
/// classifier, `facets::scheme`, checked directly — never `fam` itself,
/// which would make this tautological (the fixture already built its state
/// FROM `fam`). This failed for `Assets` when the hand-copied
/// `family()` called it Grouped, so the fixture fed it a lens strip and
/// `overlay_geometry` dutifully took the grouped path (`geom.theme`), even
/// though `facets::scheme` says `Assets` should never facet at all.
fn assert_faceted_state_matches_production(
    p: &TextPipeline,
    geom: &OverlayGeom,
    kind: OverlayKind,
    fam: Family,
    ctx: &str,
) {
    let faceted = p.overlay_geom_is_faceted(geom);
    let should_facet = crate::facets::scheme(kind).is_some();
    assert_eq!(
        faceted,
        should_facet,
        "{ctx}: the real pipeline took the {} path for this kind, but \
         `facets::scheme({kind:?})` says it should{} facet — the law's \
         family() (currently reporting {fam:?}) has drifted from the \
         production owner it is supposed to describe",
        if faceted {
            "GROUPED"
        } else {
            "FLAT/CONTEXTUAL"
        },
        if should_facet { "" } else { " never" }
    );
}

/// SIDECAR == PLAN, independently of the derivation for the last arm: the
/// reported selected row must genuinely carry the selected ITEM, not just
/// agree by construction (both sides reading the same accessor would keep a
/// planner that forgot the grouped family's section headers in perfect
/// agreement while pointing at the wrong row).
fn assert_sidecar_matches_plan(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    v: &ViewState,
    ctx: &str,
) {
    assert!(
        plan.candidate_rows() > 0,
        "{ctx}: the card must plan at least one candidate row"
    );
    let (_top, lines, sel_row, _card_h, _canvas_h) = p
        .overlay_window_report()
        .unwrap_or_else(|| panic!("{ctx}: an open card must report a window"));
    assert_eq!(
        lines,
        plan.candidate_rows(),
        "{ctx}: the sidecar's `lines` must be the planned candidate band"
    );
    assert_eq!(
        Some(sel_row),
        plan.selected_display(),
        "{ctx}: the sidecar's `sel_row` must be the planned selected line"
    );
    assert!(
        sel_row < plan.candidate_rows(),
        "{ctx}: the planned selection must stay inside the planned window"
    );
    assert_eq!(
        plan.item_at(sel_row),
        Some(v.overlay_selected),
        "{ctx}: display line {sel_row} is reported as selected but carries item {:?}, not \
         the selected item {}",
        plan.item_at(sel_row),
        v.overlay_selected
    );
}

/// THE THREE LIST STYLES the headline sweep below forces, one at a time.
/// `Pane`/`Bars` alone left the staggering path unswept: `dx`/`dw` are zero
/// on both, so a mutation that reverted the pointer inverse to the card's own
/// undisplaced span (`PlannedRow`'s doc names this exact regression) left the
/// sweep green — a landed sidecar-geometry law caught and reported the gap
/// rather than hiding it. `Diagonal` is the one shipping style whose rows
/// carry a nonzero per-row `dx`/`dw`, so enrolling it is what turns the sweep
/// into a real test of the hit-test/draw agreement `PlannedRow::dx`/`dw`
/// exist for.
fn sweep_list_styles() -> [(&'static str, Option<theme::ListStyle>); 3] {
    [
        ("pane", Some(theme::ListStyle::Pane)),
        ("bars", Some(theme::ListStyle::Bars)),
        (
            "diagonal",
            Some(theme::ListStyle::Diagonal(
                theme::DiagonalSpine::descending(theme::DiagonalMark::CRISP),
            )),
        ),
    ]
}

/// THE HEADLINE LAW. For every planned row of every picker kind, in every
/// listed style, at four window geometries, both DPIs and BOTH MENU-BAR
/// STATES: the
/// SHAPED glyph line sits in the planned slot, the pointer hit-test at that
/// slot's own centre accepts that row's own item, and the sidecar reports the
/// planned window.
///
/// **THE MENU-BAR AXIS IS NOT DECORATION.** `menubar::MENU_BAR_ON` initialises
/// to `false` on macOS and `true` on every other platform, and the drawn bar
/// takes a vertical reserve off the top of every card's own height budget
/// (`menubar_reserve`, folded into `card_y`). So this sweep ran the macOS half
/// only, for its whole life, on the host that authors it — and its own
/// `candidate_rows() > 0` guard fired for the first time in CI's Linux job,
/// against a card the reserve had starved to an empty band.
#[test]
fn drawn_hit_test_and_sidecar_agree_on_every_planned_row_for_every_overlay_kind() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping drawn_hit_test_and_sidecar_agree_on_every_planned_row: no wgpu adapter"
        );
        return;
    };

    // The AMBIENT value, never `cfg!(target_os = ...)`: a `cfg!` inside a test
    // reports the host that COMPILED it rather than the branch the initialiser
    // actually took, so a restore written that way restores the wrong value
    // under any forcing of that initialiser.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    let styles = sweep_list_styles();
    // Harmless for the "pane" arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    // The whole geometry range, not one window: a roomy canvas, a narrow one that
    // forces the card into its edge-inset floor, a short one that clamps the
    // grouped family's own row cap, and a tall one. LOGICAL sizes — physical is
    // `logical * dpi` below, the same convention `overlay_height_clamp_law.rs`
    // uses and for the same reason: read literally as physical pixels, the
    // short 900x460 cell at dpi=2 is a logical ~450x230 window, BELOW the
    // app's own enforced minimum (`app::lifecycle`'s `MIN_COLS`/`MIN_LINES`,
    // ~464x288 logical) — not a window a live user can ever reach.
    let canvases: [(u32, u32); 4] = [(1200, 800), (700, 800), (900, 460), (1400, 1600)];

    let mut checked_rows = 0usize;
    let mut checked_headers = 0usize;
    // A non-vacuity floor PER FAMILY, not just an aggregate: an
    // aggregate floor is exactly how `Assets` drifted unnoticed (its rows kept
    // landing in the GROUPED bucket's total while the FLAT arm silently lost
    // its only faceted-in-error contributor). Counting each family separately
    // means a kind that silently stops reaching its own family's arm shows up
    // as that family's own count dropping, by name.
    let mut rows_by_family: [usize; 3] = [0, 0, 0]; // Flat, Grouped, Contextual
    let mut headers_by_family: [usize; 3] = [0, 0, 0];
    let fam_idx = |f: Family| -> usize {
        match f {
            Family::Flat => 0,
            Family::Grouped => 1,
            Family::Contextual => 2,
        }
    };
    let mut rows_by_bar = [0usize; 2];
    let mut published_by_bar = [0usize; 2];
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            for (sname, style) in styles {
                crate::render::set_list_style_test_override(style);
                for (lw, lh) in canvases {
                    let (cw, ch) = ((lw as f32 * dpi) as u32, (lh as f32 * dpi) as u32);
                    p.set_size(cw as f32, ch as f32);
                    for kind in OverlayKind::ALL {
                        let fam = family(kind);
                        let v = overlay_view(kind, 24);
                        p.set_view(&v);
                        p.prepare(&device, &queue, cw, ch).unwrap();

                        let geom = p.overlay_geometry(cw);
                        let plan = p.overlay_row_plan(&geom);
                        let probe = p.overlay_row_y_probe();
                        let ctx = format!(
                            "{kind:?}/{fam:?} dpi={dpi} list={sname} canvas={cw}x{ch} bar={bar}"
                        );

                        assert_faceted_state_matches_production(&p, &geom, kind, fam, &ctx);
                        assert_sidecar_matches_plan(&p, &plan, &v, &ctx);

                        published_by_bar[usize::from(bar)] +=
                            grade_published_geometry(&p, &plan, &probe, &ctx);

                        let (rows, headers) = grade_rows(&p, &plan, &probe, &ctx);
                        checked_rows += rows;
                        checked_headers += headers;
                        rows_by_family[fam_idx(fam)] += rows;
                        headers_by_family[fam_idx(fam)] += headers;
                        rows_by_bar[usize::from(bar)] += rows;
                    }
                }
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    assert_sweep_floors(
        checked_rows,
        checked_headers,
        rows_by_family,
        headers_by_family,
        rows_by_bar,
    );
    for (i, n) in published_by_bar.iter().enumerate() {
        assert!(
            *n > 200,
            "the sweep graded {n} PUBLISHED row rects with the menu bar {} — the \
             sidecar's own half of the three-way agreement is vacuous there \
             (both: {published_by_bar:?})",
            if i == 1 { "shown" } else { "hidden" }
        );
    }
    assert_eq!(
        published_by_bar.iter().sum::<usize>(),
        checked_rows + checked_headers,
        "every planned display line must also be a PUBLISHED one — the sidecar \
         cannot report a subset of the band the draw and the pointer share"
    );
}

/// EVERY NON-VACUITY FLOOR THE HEADLINE SWEEP OWES, in one place: the aggregate
/// row and header counts, the PER-FAMILY counts, and the PER-MENU-BAR-STATE
/// counts.
///
/// Each of the three is a different way for a sweep to grade nothing while
/// reporting a large total. An aggregate floor alone stayed green when a whole
/// family's arm went to zero — a mis-classified kind's rows land in the WRONG
/// family's bucket while its real family's bucket quietly loses its only
/// contributor. The menu-bar counts are the same failure along the newer axis:
/// the state a law never enters is the state it cannot grade, and one state
/// grading the whole roster would satisfy any total.
fn assert_sweep_floors(
    rows: usize,
    headers: usize,
    by_family: [usize; 3],
    headers_by_family: [usize; 3],
    by_bar: [usize; 2],
) {
    assert!(
        rows > 500,
        "the sweep must actually grade hundreds of item rows, got {rows}"
    );
    for (i, n) in by_bar.iter().enumerate() {
        assert!(
            *n > 200,
            "the sweep graded {n} item rows with the menu bar {} — its half of the \
             menu-bar axis is vacuous (both: {by_bar:?})",
            if i == 1 { "shown" } else { "hidden" }
        );
    }
    assert!(
        headers > 0,
        "the sweep must include the grouped family's section HEADER lines (which accept \
         no click), got {headers} — otherwise the header arm is vacuous"
    );
    for (name, idx) in [("Flat", 0), ("Grouped", 1), ("Contextual", 2)] {
        assert!(
            by_family[idx] > 0,
            "the {name} family's own row arm graded zero rows — its part of the sweep is \
             vacuous, got {by_family:?}"
        );
    }
    assert!(
        headers_by_family[1] > 0,
        "the GROUPED family's own section-header lines were never graded — got \
         {headers_by_family:?}"
    );
    assert_eq!(
        headers_by_family[0] + headers_by_family[2],
        0,
        "a FLAT or CONTEXTUAL kind graded a section-HEADER display line — only the \
         GROUPED family ever carries one, got {headers_by_family:?}"
    );
}

/// THE PLAN IS DERIVED, NEVER RETAINED. A resize, a zoom, and a buffer SWAP each
/// move the candidate band inside the very next frame, and the hit-test follows
/// in that same frame.
///
/// This is the invalidation class the plan deliberately has no cache for: there
/// is no plan key to go stale, so no `buffer.version()` collision can serve a
/// stale band (CLAUDE.md's cache-key tripwire). The law proves the absence is
/// real rather than asserted.
#[test]
fn the_plan_is_rederived_across_resize_zoom_and_a_buffer_swap() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_plan_is_rederived_across_resize_zoom_and_a_buffer_swap: no wgpu adapter"
        );
        return;
    };

    let mut v = overlay_view(OverlayKind::Keybindings, 30);

    // A helper that renders and returns (first_top, lh, the item under the
    // first row's own centre).
    let sample = |p: &mut TextPipeline, v: &ViewState, w: u32, h: u32| {
        p.set_size(w as f32, h as f32);
        p.set_view(v);
        p.prepare(&device, &queue, w, h).unwrap();
        let geom = p.overlay_geometry(w);
        let plan = p.overlay_row_plan(&geom);
        let first = plan.rows()[0];
        let (x0, x1) = plan.card_x_span();
        let hit = p.overlay_row_at((x0 + x1) * 0.5, first.top + first.height * 0.5);
        assert_eq!(
            hit, first.item,
            "the first planned row must always hit-test to its own item"
        );
        (plan.first_top(), plan.lh(), plan.candidate_rows(), x0)
    };

    let base = sample(&mut p, &v, 1200, 800);

    // RESIZE — a narrower canvas moves the card, so the planned band's x moves
    // and the hit-test follows in the same frame.
    let resized = sample(&mut p, &v, 640, 800);
    assert_ne!(
        base.3, resized.3,
        "a narrower canvas must move the planned card — otherwise this arm is vacuous"
    );

    // ZOOM — the row pitch is a measured metric, so a zoom must move both the
    // band's origin and its pitch.
    let mut zoomed = overlay_view(OverlayKind::Keybindings, 30);
    zoomed.zoom = 2.0;
    let z = sample(&mut p, &zoomed, 1200, 800);
    assert!(
        z.1 > base.1 * 1.5,
        "zoom 2.0 must widen the planned row pitch ({} -> {})",
        base.1,
        z.1
    );
    assert_ne!(
        z.0, base.0,
        "zoom must move the planned band's origin, not just its pitch"
    );

    // BUFFER SWAP — a different document under the SAME open picker, with the
    // picker's own row corpus changed too (the shape a real buffer swap takes:
    // versions restart at 0, so nothing may be keyed on them alone).
    v.text = "a totally different document\n".repeat(50);
    v.overlay_items = (0..7).map(|i| format!("other row {i}")).collect();
    v.overlay_bindings = (0..7).map(|i| format!("M-{i}")).collect();
    v.overlay_selected = 6;
    let swapped = sample(&mut p, &v, 1200, 800);
    assert_eq!(
        swapped.2, 7,
        "the swapped picker must plan its NEW seven rows, never the parked thirty"
    );
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);
    assert_eq!(
        plan.selected_display(),
        Some(6),
        "the swapped picker's selection must be planned against the new corpus"
    );
    assert_eq!(
        p.overlay_window_report().map(|r| r.1),
        Some(7),
        "the sidecar must report the swapped window, not a stale one"
    );

    p.set_dpi(1.0);
}

/// O(VISIBLE), NOT O(DOC), against the real pipeline: a picker over a large
/// corpus plans (and shapes, and uploads) only the rows on screen.
#[test]
fn a_huge_picker_corpus_still_plans_only_the_rows_on_screen() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping a_huge_picker_corpus_still_plans_only_the_rows_on_screen: no wgpu adapter"
        );
        return;
    };

    let mut planned = Vec::new();
    for n in [40usize, 4_000, 40_000] {
        let v = overlay_view(OverlayKind::Keybindings, n);
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        planned.push(plan.candidate_rows());
        assert!(
            plan.candidate_rows() <= 24,
            "a {n}-row picker planned {} rows — the plan must be bounded by the window, \
             never the corpus",
            plan.candidate_rows()
        );
        // And the plan's own claim matches what actually got shaped: the probe
        // maps one drawn line per planned row and nothing beyond.
        let probe = p.overlay_row_y_probe();
        assert_eq!(
            probe.primary.len(),
            plan.candidate_rows(),
            "a {n}-row picker shaped {} candidate lines for {} planned rows",
            probe.primary.len(),
            plan.candidate_rows()
        );
    }
    assert!(
        planned.windows(2).all(|w| w[0] == w[1]),
        "the planned row count must not grow with the corpus: {planned:?}"
    );
}

/// THE ONE DELIBERATE OUTPUT CHANGE.
///
/// `content_rows` — how many display lines precede the footer — used to be
/// computed in one place as the grouped family's plan length, omitting the
/// empty-state NOTICE line the card height had already paid for. So a bare-plate
/// world's picker filtered to zero matches drew its footer PLATE over the "no
/// matches" row: the notice sat on a plated band a whole row above the footer's
/// own glyphs.
///
/// THE ROSTER IS THE WORLDS THAT DRAW PLATES, WHICH IS NOT THE BARE-PLATE
/// ROSTER. `list_backing == BarePlates` is a claim about the CARD, not about
/// rows: Mangrove and Magpie are `ListStyle::Diagonal`, bare in that sense and
/// drawing no plate at all. A plate claim graded on them is a claim about
/// nothing, so the sweep asks `draws_row_plates` and arm 3 EARNS the exclusion
/// of the other two by measurement rather than by name.
///
/// THREE ARMS:
///
/// 1. GEOMETRY, from the quads the emitter actually produces
///    (`overlay_bar_rects_probe`) — not from arithmetic and not from the sidecar:
///    no drawn row surface may overlap the notice row's own planned slot.
/// 2. APPEARANCE, from the frame's own pixels: on `Bars`, the notice row's band
///    must read as plain card ground rather than as the footer's plate. Diagonal
///    authors ink through that ground, so its appearance is not a Bars oracle.
///    The oracle reads MEDIANS over the drawn plate's OWN COLUMNS — see
///    `notice_reads_as_ground` for why a card-wide mean measured plate width and
///    glyph coverage instead of plate visibility — and EVERY plate-drawing world
///    is graded, which the arm asserts rather than assumes.
/// 3. THE EXCLUSION, on the bare-plate worlds arms 1 and 2 do not reach: the
///    frame must emit NO row surface at all, so no plate can sit on the notice.
#[test]
fn an_empty_states_notice_row_carries_no_footer_plate_on_any_bare_plate_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping an_empty_states_notice_row_carries_no_footer_plate: no wgpu adapter");
        return;
    };

    let (plated, plateless): (Vec<&'static str>, Vec<&'static str>) = theme::THEMES
        .iter()
        .filter(|t| t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates)
        .map(|t| (t.name, t.render_caps.list_style.draws_row_plates()))
        .fold((Vec::new(), Vec::new()), |mut acc, (name, draws)| {
            match draws {
                true => acc.0.push(name),
                false => acc.1.push(name),
            }
            acc
        });
    assert_eq!(
        (plated.as_slice(), plateless.as_slice()),
        (
            ["Galah", "Firetail"].as_slice(),
            ["Mangrove", "Magpie", "Paperbark", "Kite"].as_slice()
        ),
        "the shipping bare-plate roster splits exactly this way — a new world joins \
         one arm or the other, never neither"
    );

    let mut pixel_graded: Vec<&str> = Vec::new();
    let mut measured_presence: Vec<(&str, f64)> = Vec::new();
    let mut plateless_graded: Vec<&str> = Vec::new();
    for world in plated.iter().chain(&plateless) {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();

        // A GROUPED (lens-strip) card filtered to zero matches: the notice line,
        // then the foot hint.
        let mut v = view("hello world\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = OverlayKind::Command.title();
        v.overlay_items = Vec::new();
        v.overlay_empty = Some("no matches".into());
        v.overlay_hint = "type to filter".into();
        v.overlay_lens = vec![("All".into(), false), ("Recent".into(), true)];
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();

        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        assert_eq!(
            plan.candidate_rows(),
            0,
            "{world}: a zero-match card plans no candidate rows"
        );
        assert_eq!(
            plan.content_rows(),
            1,
            "{world}: the empty-state NOTICE must occupy one content line"
        );
        let lh = plan.lh();
        let notice_top = plan.first_top();
        let footer_top = plan.footer_top();
        assert!(
            (footer_top - (notice_top + lh)).abs() < 0.01,
            "{world}: the footer must start exactly one row below the notice"
        );

        // ARM 3 — the exclusion, measured. A bare-plate world outside the plated
        // roster is excused from arms 1 and 2 for one reason only: it draws no
        // row surface at all, so nothing can be seated over the notice.
        if !plated.contains(world) {
            let surfaces = p.overlay_row_surfaces_probe();
            assert!(
                surfaces.is_empty(),
                "{world}: this world is excluded from the plate arms because it draws no \
                 row surface, but the frame emitted {surfaces:?}. Either it now draws \
                 plates — in which case it belongs in the plated roster and must be \
                 graded by arm 1 — or the exclusion is wrong."
            );
            plateless_graded.push(world);
            continue;
        }

        // ARM 1 — the emitted quads.
        let (sel, unsel) = p.overlay_bar_rects_probe();
        let plates: Vec<[f32; 4]> = sel.into_iter().chain(unsel).collect();
        assert!(
            !plates.is_empty(),
            "{world}: an empty Bars card must still emit its footer plate — otherwise \
             this arm is vacuous"
        );
        let notice_lo = notice_top + 1.0;
        let notice_hi = notice_top + lh - 1.0;
        let intruders: Vec<[f32; 4]> = plates
            .iter()
            .copied()
            .filter(|r| r[1] < notice_hi && r[1] + r[3] > notice_lo)
            .collect();
        assert!(
            intruders.is_empty(),
            "{world}: a drawn row surface overlaps the empty-state NOTICE row's own \
             planned slot (y {notice_lo:.1}..{notice_hi:.1}): {intruders:?}. The footer \
             plate is seated a row too high — the notice line the card height paid for \
             was left out of `content_rows`."
        );
        // …and the footer plate really is seated at the planned footer top.
        let footer_plate = *plates
            .iter()
            .find(|r| (r[1] - footer_top).abs() < lh * 0.5)
            .unwrap_or_else(|| {
                panic!(
                    "{world}: no drawn plate sits at the planned footer top {footer_top:.1} \
                     (plates {plates:?}) — arm 1 must be watching a real plate"
                )
            });

        // ARM 2 — the pixels, graded over that plate's own columns.
        let pixels = shoot(&device, &queue, &mut p, 1200, 800);
        let bands = (notice_top, footer_top, lh);
        if matches!(
            theme::active().render_caps.list_style,
            theme::ListStyle::Bars
        ) {
            let (presence, _lift) = notice_reads_as_ground(&pixels, footer_plate, bands, world);
            measured_presence.push((world, presence));
            pixel_graded.push(world);
        }
    }
    crate::render::set_list_style_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
    assert_plate_separation_is_not_vacuous(&measured_presence);
    // ENROLMENT IS THE PLATED ROSTER, DERIVED FROM THEME DATA — never a hardcoded
    // subset of it. A pixel oracle that grades "whichever worlds it happened to be
    // able to see" hardcodes a property of the GPU: this set was once literally
    // `["Firetail"]`, and the two worlds it omitted were omitted because a
    // card-wide mean could not see their plates, one of them straddling the
    // enrolment gate closely enough to flip between backends. The equality below
    // is what makes a narrowing fail by name instead of going quietly green.
    assert_eq!(
        pixel_graded, plated,
        "the Bars appearance arm must grade EVERY plate-drawing world"
    );
    assert_eq!(
        plateless_graded, plateless,
        "the exclusion arm must reach every plateless world"
    );
}

/// THE FOOTER PLATE CLEARS THE NOTICE CHANNEL'S OWN PRESENCE FLOOR, NOW THAT
/// IT HAS THE SAME RIM.
///
/// Before this law the footer plate had no independent presence claim at all —
/// `an_empty_states_notice_row_carries_no_footer_plate_on_any_bare_plate_world`'s
/// own `PLATE_DISCRIMINABLE_MIN` (ΔE 1.5) only asks "can arm 2's oracle tell the
/// plate from ground at all", which Cassowary's un-rimmed plate cleared at ΔE
/// 1.91 while sitting under the ≈2.3 JND — a plate nobody could see, passing a
/// floor that was never a legibility claim. This law asks the real question,
/// against the SAME floor the calm notice's own rim already clears
/// (`notice::PLATE_PRESENCE_MIN`, ΔE 15) — reused rather than re-derived, because
/// both channels earn it the identical way: a value-stepped fill plus a
/// one-pixel rim off the ink ladder.
///
/// The technique mirrors `notice.rs`'s own presence floor exactly: the PAGE
/// reference is "what was actually there" — the same rect, read back from a
/// render of the identical view with the overlay closed — rather than a nominal
/// token value, so a page whose real on-screen colour differs from its authored
/// constant (dither, gradient, dimming) cannot flatter the measurement.
#[test]
fn footer_plate_clears_the_notice_channels_presence_floor_on_every_bars_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping footer_plate_clears_the_notice_channels_presence_floor: no wgpu adapter"
        );
        return;
    };

    // Enrolment derived from the roster, not a named world — asserted equal
    // to the plate-drawing roster the sibling law already established, so the
    // two laws cannot silently enroll different sets.
    let plated: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| {
            t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates
                && t.render_caps.list_style.draws_row_plates()
        })
        .map(|t| t.name)
        .collect();
    assert_eq!(
        plated,
        ["Galah", "Firetail"],
        "the plate-drawing `Bars` roster moved — this law's enrolment must move with it"
    );

    // The SAME logical room at two device scales (`rotated_rail.rs`'s
    // own tier shape) — a device-pixel bug in a one-px rim is exactly the
    // class `--capture-dpi 1` alone cannot see.
    const TIERS: [(u32, u32, f32); 2] = [(1200, 800, 1.0), (2400, 1600, 2.0)];

    let entry_world = theme::active_index();
    let mut worst: Vec<(String, f64)> = Vec::new();
    for world in &plated {
        theme::set_active_by_name(world).unwrap();
        for &(cw, ch, dpi) in &TIERS {
            p.sync_theme();
            p.set_size(cw as f32, ch as f32);
            p.set_dpi(dpi);

            let mut v = view("hello world\n", 0, 0);
            v.overlay_active = true;
            v.overlay_title = OverlayKind::Command.title();
            v.overlay_items = vec!["Go to file...".into(), "Switch project...".into()];
            v.overlay_hint = "type to filter".into();
            p.set_view(&v);
            p.prepare(&device, &queue, cw, ch).unwrap();

            let geom = p.overlay_geometry(cw);
            let plan = p.overlay_row_plan(&geom);
            let footer_top = plan.footer_top();
            let lh = plan.lh();

            let (sel, unsel) = p.overlay_bar_rects_probe();
            let plates: Vec<[f32; 4]> = sel.into_iter().chain(unsel).collect();
            let plate = *plates
                .iter()
                .find(|r| (r[1] - footer_top).abs() < lh * 0.5)
                .unwrap_or_else(|| {
                    panic!(
                        "{world}@{dpi}: no drawn plate sits at the planned footer top \
                         {footer_top:.1} (plates {plates:?}) — this law must watch a real plate"
                    )
                });
            let shot_with = shoot(&device, &queue, &mut p, cw, ch);

            // THE PAGE REFERENCE: this exact rect, rendered with the overlay
            // closed — "what was actually there", `notice.rs`'s own technique.
            // Document layout does not move when the overlay opens, so the
            // rect maps to the same real page pixels either way.
            let plain = view("hello world\n", 0, 0);
            p.set_view(&plain);
            p.prepare(&device, &queue, cw, ch).unwrap();
            let shot_plain = shoot(&device, &queue, &mut p, cw, ch);
            // Restore the overlay view before the next iteration reads it.
            p.set_view(&v);

            let pad = (plate[2].min(plate[3]) * 0.15).max(3.0);
            let (ix0, iy0, ix1, iy1) = (
                plate[0] + pad,
                plate[1] + pad,
                plate[0] + plate[2] - pad,
                plate[1] + plate[3] - pad,
            );
            let fill = median_of(&shot_with, ix0, iy0, ix1, iy1, cw, ch);
            let rim = median_ring(&shot_with, plate, 1.0, cw, ch);
            let page = median_of(&shot_plain, ix0, iy0, ix1, iy1, cw, ch);
            let presence = pixeldiff::delta_e(rim, page).max(pixeldiff::delta_e(fill, page));
            worst.push((format!("{world}@{dpi}"), presence));
            assert!(
                presence >= super::notice::PLATE_PRESENCE_MIN,
                "{world}@{dpi}: the footer plate (page {page:?}, fill {fill:?}, rim {rim:?}) \
                 sits ΔE {presence:.2} from its own page, under the notice channel's own ΔE \
                 {} presence floor — the rim did not earn its keep here",
                super::notice::PLATE_PRESENCE_MIN
            );
            eprintln!(
                "{world}@{dpi}: footer-plate presence ΔE {presence:.2} (floor {})",
                super::notice::PLATE_PRESENCE_MIN
            );
        }
    }
    theme::set_active(entry_world);
    let (tightest_world, tightest) = worst
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).expect("no NaN ΔE"))
        .expect("the plated roster is non-empty");
    eprintln!(
        "footer-plate presence: tightest is {tightest_world} at ΔE {tightest:.2} \
         (floor {})",
        super::notice::PLATE_PRESENCE_MIN
    );
}

/// The MEDIAN colour over a rect, in device px — robust to the minority of
/// pixels a glyph or a rounded corner contributes (see `notice.rs`'s own doc
/// for why a median rather than a mean).
fn median_of(pixels: &[[u8; 4]], x0: f32, y0: f32, x1: f32, y1: f32, w: u32, h: u32) -> [u8; 4] {
    let luma = |px: [u8; 4]| -> f64 {
        0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
    };
    let (a, b) = (
        y0.ceil().max(0.0) as u32,
        y1.floor().min(h as f32 - 1.0) as u32,
    );
    let (c, d) = (
        x0.ceil().max(0.0) as u32,
        x1.floor().min(w as f32 - 1.0) as u32,
    );
    let mut v: Vec<[u8; 4]> = (a..b)
        .flat_map(|y| (c..d).map(move |x| pixels[(y * w + x) as usize]))
        .collect();
    assert!(
        !v.is_empty(),
        "empty sample band x {x0:.1}..{x1:.1} y {y0:.1}..{y1:.1}"
    );
    v.sort_by(|p, q| luma(*p).partial_cmp(&luma(*q)).expect("no NaN luminance"));
    v[v.len() / 2]
}

/// The MEDIAN colour of the ring `plate` grown by `grow` on every side, MINUS
/// `plate` itself — the rim's own solid core, however it was drawn. Not tied to
/// the specific one-pixel-outset convention `footer_plate_rim` happens to use:
/// this samples whatever occupies the grown band, so a law reading it fails on
/// an absent or mis-sized rim instead of assuming its geometry.
fn median_ring(pixels: &[[u8; 4]], plate: [f32; 4], grow: f32, w: u32, h: u32) -> [u8; 4] {
    let luma = |px: [u8; 4]| -> f64 {
        0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
    };
    let [x, y, pw, ph] = plate;
    let (ox0, oy0, ox1, oy1) = (x - grow, y - grow, x + pw + grow, y + ph + grow);
    let (a, b) = (oy0.floor().max(0.0) as u32, oy1.ceil().min(h as f32) as u32);
    let (c, d) = (ox0.floor().max(0.0) as u32, ox1.ceil().min(w as f32) as u32);
    let mut v: Vec<[u8; 4]> = Vec::new();
    for yy in a..b {
        for xx in c..d {
            let (fx, fy) = (xx as f32 + 0.5, yy as f32 + 0.5);
            let inside_fill = fx > x && fx < x + pw && fy > y && fy < y + ph;
            if !inside_fill {
                v.push(pixels[(yy * w + xx) as usize]);
            }
        }
    }
    assert!(
        !v.is_empty(),
        "empty rim ring for plate {plate:?}, grow {grow}"
    );
    v.sort_by(|p, q| luma(*p).partial_cmp(&luma(*q)).expect("no NaN luminance"));
    v[v.len() / 2]
}

/// Render the current frame offscreen and read it back.
fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl empty-card encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

/// ARM 2 of the empty-state law: does the NOTICE row's band read as plain card
/// ground rather than as the footer's plate?
///
/// THE STATISTIC IS A MEDIAN OVER THE DRAWN PLATE'S OWN COLUMNS, and both halves
/// of that phrase are load-bearing.
///
/// * **The plate's columns, not the card's.** A plate HUGS its row's content, so
///   it covers a fraction of the card that varies from ~20% (a wide card, a short
///   hint) to ~80% (a narrow right-rail card). Averaging across the whole card
///   therefore measures *how wide the plate is* as much as how visible it is —
///   and it dilutes the plate's own contrast by that same fraction.
/// * **A median, not a mean.** Inside the plate's columns the flat fill is the
///   strict majority of pixels and glyph ink is the minority, so the median
///   returns the FILL COLOUR ITSELF, exactly. A mean instead returns
///   `coverage × contrast` summed over every surface in the band, which mixes
///   three quantities that have nothing to do with each other: the plate's
///   contrast, the plate's width, and the row's own glyph coverage. Two of those
///   are set by the shaper and the card geometry, and the third by the palette.
///
/// That mixing is not academic. Under a card-wide mean the notice row's lift is
/// `ink_coverage × ink_contrast`, and `ink_coverage` scales as `1 / card_width`:
/// the same "no matches" string reads as a whisper on a 600px card and as a band
/// on a 178px one, purely because the divisor changed. On the footer row the same
/// mean subtracts the plate's darkening from the hint's ink lift — two comparable
/// terms — so the enrolment number is a RESIDUAL, and a small shift in either
/// input moves it by a large fraction of itself. That is how enrolment came to
/// depend on which GPU rasterized the glyphs.
///
/// `bands` is `(notice_top, footer_top, lh)`; `plate` is the footer plate quad
/// the emitter really produced (arm 1's own evidence), which supplies the
/// columns. THERE IS NO "COULD NOT SEE IT" ANSWER: this used to hand one back and
/// the caller counted it as an excuse, which is how a graded roster of one world
/// came to look deliberate. Visibility is now an assertion like any other.
fn notice_reads_as_ground(
    pixels: &[[u8; 4]],
    plate: [f32; 4],
    bands: (f32, f32, f32),
    world: &str,
) -> (f64, f64) {
    let (notice_top, footer_top, lh) = bands;
    let x_lo = (plate[0] + 2.0).max(0.0) as u32;
    let x_hi = (plate[0] + plate[2] - 2.0).min(1199.0) as u32;
    let at = |x: u32, y: u32| -> [u8; 4] { pixels[(y * 1200 + x) as usize] };
    let luma = |px: [u8; 4]| -> f64 {
        0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
    };
    // Sorted BY LUMINANCE but carrying the PIXEL, so the median below is the
    // fill's own colour rather than one projection of it — the whole point of
    // taking a median here (see this function's doc) survives the move to a
    // perceptual oracle only if all three axes come with it.
    let samples = |y0: f32, y1: f32| -> Vec<[u8; 4]> {
        let (a, b) = (y0.ceil().max(0.0) as u32, y1.floor().min(800.0) as u32);
        let mut v: Vec<[u8; 4]> = (a..b)
            .flat_map(|y| (x_lo..x_hi).map(move |x| (x, y)))
            .map(|(x, y)| at(x, y))
            .collect();
        v.sort_by(|p, q| luma(*p).partial_cmp(&luma(*q)).expect("no NaN luminance"));
        v
    };
    let median = |y0: f32, y1: f32| -> [u8; 4] {
        let v = samples(y0, y1);
        assert!(!v.is_empty(), "{world}: empty sample band {y0:.1}..{y1:.1}");
        v[v.len() / 2]
    };
    let pad = (lh * 0.25).max(4.0);

    // THE GROUND REFERENCE is the gap directly above the first content row —
    // adjacent to the band being graded, so no card-wide wash can tilt it, and
    // above every row surface, so no plate can. The card's own TOP PAD is the
    // wrong reference: two worlds shade their card's top edge, which biases it
    // away from the ground that actually surrounds the notice row.
    //
    // That the gap IS plain ground is asserted rather than assumed: a header
    // block that grew into it, or a world that washed it, would otherwise
    // silently rebase every number below.
    let (g0, g1) = (notice_top - pad - 2.0, notice_top - 2.0);
    let gap = samples(g0, g1);
    let ground = gap[gap.len() / 2];
    let flat = gap
        .iter()
        .filter(|p| (luma(**p) - luma(ground)).abs() <= 1.0)
        .count() as f64
        / gap.len() as f64;
    assert!(
        flat > 0.98,
        "{world}: the ground reference strip (y {g0:.1}..{g1:.1}, the gap above the \
         notice row) is not plain card ground — only {:.1}% of it sits within 1 luma \
         of its median {ground:?}. Something now draws there, so every number this \
         oracle derives from it is rebased.",
        flat * 100.0
    );

    let notice = median(notice_top + pad, notice_top + lh - pad);
    let footer = median(footer_top + pad, footer_top + lh - pad);
    let plate_presence = pixeldiff::delta_e(footer, ground);
    let notice_lift = pixeldiff::delta_e(notice, ground);
    assert!(
        plate_presence > PLATE_DISCRIMINABLE_MIN,
        "{world}: the footer plate's own fill is indistinguishable from card ground \
         (ground {ground:?}, plate {footer:?}; ΔE {plate_presence:.2} ≤ \
         {PLATE_DISCRIMINABLE_MIN}), so arm 2 cannot see the surface it grades. Either \
         this world's palette changed or the plate stopped drawing — ENROLMENT MUST \
         NOT SILENTLY NARROW, and it must never depend on which GPU ran the test."
    );
    assert!(
        notice_lift <= NOTICE_IS_GROUND_MAX,
        "{world}: the empty-state NOTICE row reads as a plated band rather than plain \
         card ground (ground {ground:?}, notice {notice:?}, plate {footer:?}; notice \
         lift ΔE {notice_lift:.2} > {NOTICE_IS_GROUND_MAX}, against the plate's own \
         ΔE {plate_presence:.2}). The footer plate is seated a row too high — the \
         notice line the card height paid for was left out of `content_rows`."
    );
    // Reported every run: a floor whose headroom a reader has to take on trust is
    // a floor nobody can tell has stopped discriminating.
    eprintln!(
        "{world}: footer-plate presence ΔE {plate_presence:.2} (discriminability \
         floor {PLATE_DISCRIMINABLE_MIN}), notice-row lift ΔE {notice_lift:.2} \
         (ceiling {NOTICE_IS_GROUND_MAX})"
    );
    (plate_presence, notice_lift)
}

/// **NON-VACUITY OF ARM 2, PROVED AGAINST THE ROSTER RATHER THAN AGAINST TWO
/// LITERALS.** The arm's claim is "the notice row sits within ΔE
/// [`NOTICE_IS_GROUND_MAX`] of ground", and that only means something on a world
/// where a plate seated one row too high would actually EXCEED the ceiling. So
/// every enrolled world's own MEASURED plate presence has to clear it with margin;
/// otherwise the arm passes on a world whose plate is too close to its ground to be
/// detected by the very statistic that grades it.
///
/// Written as a named check rather than inline because it is a claim about the
/// LAW, not about the frame — and because the two floors it relates are the whole
/// reason the shared luminance constant had to be split.
fn assert_plate_separation_is_not_vacuous(measured: &[(&str, f64)]) {
    let (tightest, tightest_de) = measured
        .iter()
        .copied()
        .min_by(|a, b| a.1.partial_cmp(&b.1).expect("no NaN ΔE"))
        .expect("the plated roster is non-empty");
    assert!(
        tightest_de > NOTICE_IS_GROUND_MAX * 1.5,
        "the roster's tightest footer-plate presence is {tightest} at ΔE \
         {tightest_de:.2}, which does not clear the reads-as-ground ceiling (ΔE \
         {NOTICE_IS_GROUND_MAX}) by half again. On that world a plate seated a row \
         too high would measure inside the ceiling, so arm 2's claim is vacuous \
         there — lower the ceiling or give the plate a rim, never widen the gate. \
         Measured: {measured:?}"
    );
}

/// **ONE PERCEPTUAL SCALE, TWO FLOORS, AND THE SEPARATION BETWEEN THEM PROVED
/// AGAINST THE ROSTER RATHER THAN ASSUMED.**
///
/// These were ONE absolute 8-bit luminance constant (`VISIBLE_PLATE_LUMA = 7.0`),
/// and it was the wrong unit twice over. A `|ΔY|` gap collapses in the dark and is
/// luminance-ONLY, so it calls a plate that differs from its page in hue or chroma
/// invisible — `pixeldiff::delta_e`'s doc records the Potoroo case where that
/// demanded a product change to a plainly legible surface. And its recorded
/// "roster clears the gate by 2.5×–5×" margin was measured against a page that was
/// never on screen: the frame's `LoadOp::Clear` handed raw sRGB bytes to an sRGB
/// target, so every dark world's ground drew tens of bytes too light.
///
/// The conversion moved TWO verdicts, in OPPOSITE directions, which is what a
/// better unit is supposed to do. Firetail's plate sits 6.06 luma from its true
/// page — under the old gate — but **ΔE 7.50**, comfortably visible. Cassowary's
/// sits **ΔE 1.91**, under the classic ΔE ≈ 2.3 just-noticeable difference, and the
/// luminance gate never reached it because Firetail aborted the run first.
///
/// ⚠️ **[`PLATE_DISCRIMINABLE_MIN`] IS NOT A LEGIBILITY FLOOR AND MUST NOT BE READ
/// AS ONE.** It answers "can arm 2's claim discriminate at all" — a question about
/// the ORACLE, not about a reader's eye. Whether Cassowary's ΔE 1.91 footer plate
/// is visible ENOUGH is a separate product question, escalated as a finding rather
/// than settled by a constant here; this file must not certify it either way. The
/// legibility floor for a plate that HAS one is `render::tests::notice`'s own
/// `PLATE_PRESENCE_MIN` (ΔE 15), which the notice channel clears because it also
/// draws a RIM — the mechanism this footer plate has never had.
///
/// [`NOTICE_IS_GROUND_MAX`] is the arm's REAL claim: the notice row reads as plain
/// card ground, i.e. is not plated. Splitting the constant made it STRICTER — it
/// used to be bounded by whatever the visibility gate happened to need, and every
/// enrolled world measures ΔE 0.00 against a ceiling of 1.0.
///
/// The separation is not a matter of one literal being written smaller than the
/// other: the caller asserts every enrolled world's MEASURED plate presence clears
/// the ceiling with margin, so "a plated notice row would be caught" is proved
/// against the shipping roster on every run rather than inferred from two numbers.
const PLATE_DISCRIMINABLE_MIN: f64 = 1.5;
const NOTICE_IS_GROUND_MAX: f64 = 1.0;

// --- The retired-arithmetic source law ---------------------------------------

/// The sentinel TERMS of the row-Y arithmetic moved into the planner.
/// A production line under `src/render/` that contains one is re-deriving a row's
/// position from loose scalars — the shape that let the draw path, the hit-test,
/// the band and the footer plate each carry their own copy.
const RETIRED_TERMS: &[&str] = &[
    "overlay_row_top(",
    "overlay_row_of(",
    "overlay_row_index(",
    "header_rows as f32",
    // The HEADER band's four retired owners. Each is DELETED from
    // `render/chrome`, so the compiler is the primary guard and these are the
    // belt-and-braces half: reintroducing one by name fails here instead of
    // shipping a fifth answer to "where is the query line".
    "overlay_secondary_top(",
    "overlay_split_bounds(",
    "overlay_strip_band(",
    "overlay_query_center(",
    // …and the ARITHMETIC shape those four carried: adding the card's loose
    // `header_gap` to something to place or size a header line. The planner owns
    // `text_top + i*lh` and the beat's home in the LAST header line's box; a
    // consumer that reaches for `geom.header_gap` in a sum is re-deriving it.
    //
    // DISCLOSED LIMIT: this catches the sum written against `geom`, which is how
    // every retired copy was written. A copy that first binds the gap to a local
    // and sums THAT is invisible to it — which is why the DELETION, not this
    // sweep, is the real enforcement. The last member of the family outside it,
    // `comparison.rs`'s workspace header band, has MERGED: its consumer is
    // called ~45 times a frame and still cannot afford a plan, but it now calls
    // the planner's own `header_band_height` rather than re-summing, and
    // `overlay_header_band_law.rs`'s
    // `the_workspace_header_band_still_agrees_with_the_planned_header_boxes`
    // grades it against the planned boxes over both workspace shapes.
    "+ geom.header_gap",
    "geom.header_gap +",
];

/// The OTHER shape of the same mistake: stepping off the plan's own band origin by
/// a row COUNT instead of reading the row's slot. `plan.first_top() + k as f32 *
/// plan.lh()` is arithmetically the answer today and drifts the moment a row's
/// height stops being uniform — and the plate that used to be seated this way was
/// a row too high for two years. Production may name `first_top` (the band's top
/// edge is a legitimate clip bound) but never multiply off it.
fn steps_off_the_band_origin(line: &str) -> bool {
    line.contains("first_top") && line.contains("as f32")
}

/// The ONLY place the arithmetic may live — the planner's own files.
///
/// `plan/overlay_header.rs` is here because `header_band_height` is the one
/// owner of "how far below `text_top` the candidate band begins", and both the
/// planner's `row_top` and the workspace's relocated document viewport now read
/// it instead of each summing `header_rows * lh + header_gap` for themselves. It
/// is planner arithmetic living in a planner file, which is exactly what this
/// list is for.
const ARITHMETIC_OWNERS: &[&str] = &[
    "plan/overlay_rows.rs",
    "plan/overlay_row_plan.rs",
    "plan/overlay_header.rs",
];

fn re_derives_a_row_y(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false; // prose, not code
    }
    RETIRED_TERMS.iter().any(|t| line.contains(t)) || steps_off_the_band_origin(line)
}

fn scan(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<(String, usize)>) {
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
            scan(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // A sibling `tests.rs` is the test tier, not a render path.
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (i, line) in text.lines().enumerate() {
            if re_derives_a_row_y(line) {
                out.push((rel.clone(), i + 1));
            }
        }
    }
}

#[test]
fn only_the_planner_derives_an_overlay_row_position() {
    let render_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("render");
    let mut hits = Vec::new();
    scan(&render_root, &render_root, &mut hits);
    let strays: Vec<String> = hits
        .iter()
        .filter(|(f, _)| !ARITHMETIC_OWNERS.contains(&f.as_str()))
        .map(|(f, l)| format!("  {f}:{l}"))
        .collect();
    assert!(
        strays.is_empty(),
        "only `{ARITHMETIC_OWNERS:?}` may derive an overlay candidate row's position. A \
         consumer that computes its own row y is a parallel calculation: the drawn band, \
         the click that lands in it and the sidecar's report of it will agree until the \
         day one of them is edited. Read the row off `OverlayRowPlan` instead. \
         offending lines:\n{}",
        strays.join("\n")
    );

    // NON-VACUOUS: the owner really does carry the arithmetic, so a refactor that
    // moved it elsewhere trips this law instead of silently emptying it.
    for owner in ARITHMETIC_OWNERS {
        assert!(
            hits.iter().any(|(f, _)| f == owner),
            "`{owner}` must carry planner arithmetic"
        );
    }
}

#[test]
fn the_source_scanner_reads_code_and_skips_prose() {
    assert!(re_derives_a_row_y(
        "    text_top + header_rows as f32 * lh + header_gap + row as f32 * lh"
    ));
    assert!(re_derives_a_row_y(
        "        let t = overlay_row_top(a, b, c, d, e);"
    ));
    assert!(!re_derives_a_row_y(
        "/// folds it in through [`overlay_row_top`] — a prose reference"
    ));
    assert!(!re_derives_a_row_y("// header_rows as f32 in a note"));
    assert!(!re_derives_a_row_y("        let top = plan.row_top(k);"));
    assert!(re_derives_a_row_y(
        "        let t = plan.first_top() + k as f32 * plan.lh();"
    ));
    assert!(!re_derives_a_row_y(
        "                    bounds: clip(0.0, plan.first_top()),"
    ));
}
