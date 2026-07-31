//! ITEM 174 — THE SCENE-PLAN LAWS AGAINST THE REAL PIPELINE.
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
//! ITEM 185 — this file's own `family()` classifier used to be a HAND-COPIED
//! match that disagreed with the production owner it was meant to describe:
//! it called `OverlayKind::Assets` `Grouped`, but `facets::scheme(Assets)` is
//! `None` ("the asset cleaner is a flat list — no lens strip"). That mislabel
//! was not vacuous — the headline law's own fixture (`overlay_view`) reads
//! `family()` to decide whether to populate a lens strip, so `Assets` was fed
//! a lens strip production never grants it, and `overlay_geometry` dutifully
//! took the GROUPED path for a kind that can never reach it live; the sweep
//! graded real rows the whole time, just along a code path `Assets` cannot
//! take outside this test, while never exercising its real FLAT path. Fixed
//! by deriving `family()` from `facets::scheme` directly (item 181's own
//! `overlay_height_clamp_law.rs` already did this correctly — the shape to
//! follow); the headline law also checks the REAL pipeline's `geom.theme`
//! against `facets::scheme` independently, so a future drift fails by name
//! instead of silently regrading the wrong path again.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;
use crate::render::chrome::OverlayGeom;

/// How a picker kind lays its candidate area out. NOT a hand-copied match over
/// `OverlayKind` — item 185's own lesson. `overlay_geometry` decides FLAT vs
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
/// INTERACTIVE (`overlay_row_at` at the slot's centre and both card edges) ==
/// that row's own item. Returns `(item rows, header lines)` graded.
fn grade_rows(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    probe: &super::overlay_probe::OverlayYProbe,
    ctx: &str,
) -> (usize, usize) {
    let (x0, x1) = plan.card_x_span();
    let mid_x = (x0 + x1) * 0.5;
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
        for x in [x0, mid_x, x1] {
            assert_eq!(
                p.overlay_row_at(x, mid_y),
                row.item,
                "{ctx}: display row {} draws item {:?} but the pointer at ({x}, {mid_y}) \
                 resolves differently",
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

/// ITEM 185 — the REAL pipeline's own path must match production's OWN
/// classifier, `facets::scheme`, checked directly — never `fam` itself,
/// which would make this tautological (the fixture already built its state
/// FROM `fam`). Before item 185 this failed for `Assets`: the hand-copied
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

/// THE HEADLINE LAW. For every planned row of every picker kind, in both list
/// styles, at four window geometries and both DPIs: the SHAPED glyph line sits in
/// the planned slot, the pointer hit-test at that slot's own centre accepts that
/// row's own item, and the sidecar reports the planned window.
#[test]
fn drawn_hit_test_and_sidecar_agree_on_every_planned_row_for_every_overlay_kind() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping drawn_hit_test_and_sidecar_agree_on_every_planned_row: no wgpu adapter"
        );
        return;
    };

    let styles: [(&str, Option<theme::ListStyle>); 2] = [
        ("pane", Some(theme::ListStyle::Pane)),
        (
            "bars",
            Some(theme::ListStyle::Bars {
                radius: 6.0,
                gap: 8.0,
                grow_px: 24.0,
                extent: theme::BarExtent::FullWidth,
                coverage: theme::BarCoverage::All,
            }),
        ),
    ];
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
    // ITEM 185 — a non-vacuity floor PER FAMILY, not just an aggregate: an
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
                    let ctx = format!("{kind:?}/{fam:?} dpi={dpi} list={sname} canvas={cw}x{ch}");

                    assert_faceted_state_matches_production(&p, &geom, kind, fam, &ctx);
                    assert_sidecar_matches_plan(&p, &plan, &v, &ctx);

                    let (rows, headers) = grade_rows(&p, &plan, &probe, &ctx);
                    checked_rows += rows;
                    checked_headers += headers;
                    rows_by_family[fam_idx(fam)] += rows;
                    headers_by_family[fam_idx(fam)] += headers;
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    assert!(
        checked_rows > 500,
        "the sweep must actually grade hundreds of item rows, got {checked_rows}"
    );
    assert!(
        checked_headers > 0,
        "the sweep must include the grouped family's section HEADER lines (which accept \
         no click), got {checked_headers} — otherwise the header arm is vacuous"
    );

    // ITEM 185 — PER-FAMILY floors. An aggregate floor alone would have stayed
    // green even if a whole family's arm quietly went to zero (exactly what a
    // mis-classified kind can cause: its rows get counted toward the WRONG
    // family's bucket while its real family's bucket loses a contributor).
    for (name, idx) in [("Flat", 0), ("Grouped", 1), ("Contextual", 2)] {
        assert!(
            rows_by_family[idx] > 0,
            "the {name} family's own row arm graded zero rows — its part of the sweep is \
             vacuous, got {rows_by_family:?}"
        );
    }
    assert!(
        headers_by_family[fam_idx(Family::Grouped)] > 0,
        "the GROUPED family's own section-header lines were never graded — got \
         {headers_by_family:?}"
    );
    assert_eq!(
        headers_by_family[fam_idx(Family::Flat)] + headers_by_family[fam_idx(Family::Contextual)],
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

/// ITEM 174's ONE DELIBERATE OUTPUT CHANGE.
///
/// `content_rows` — how many display lines precede the footer — used to be
/// computed in one place as the grouped family's plan length, omitting the
/// empty-state NOTICE line the card height had already paid for. So a `Bars`
/// world's picker filtered to zero matches drew its footer PLATE over the "no
/// matches" row: the notice sat on a plated band a whole row above the footer's
/// own glyphs.
///
/// TWO ARMS, over the WHOLE shipping `Bars` roster (read off the theme data, so a
/// new Bars world joins the sweep):
///
/// 1. GEOMETRY, from the quads the emitter actually produces
///    (`overlay_bar_rects_probe`) — not from arithmetic and not from the sidecar:
///    no drawn row surface may overlap the notice row's own planned slot.
/// 2. APPEARANCE, from the frame's own pixels: the notice row's band must read as
///    plain card ground rather than as the footer's plate. Absolute luminance is
///    the only oracle available here (a differential against the plate-less card
///    is not sound — dropping the foot hint re-shapes the whole panel buffer), and
///    on a pale world the plate ink sits a couple of luminance steps off its own
///    ground, so the arm GRADES the worlds whose plate the oracle can genuinely
///    see (`plate_delta > VISIBLE_PLATE_LUMA`) and requires that set to be
///    non-empty. Arm 1 covers the rest.
#[test]
fn an_empty_states_notice_row_carries_no_footer_plate_on_any_bars_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping an_empty_states_notice_row_carries_no_footer_plate: no wgpu adapter");
        return;
    };

    let bars_worlds: Vec<&'static str> = theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.list_style, theme::ListStyle::Bars { .. }))
        .map(|t| t.name)
        .collect();
    assert!(
        bars_worlds.len() >= 4,
        "the Bars roster must be real, got {bars_worlds:?}"
    );

    let mut pixel_graded: Vec<&str> = Vec::new();
    for world in &bars_worlds {
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
        assert!(
            plates.iter().any(|r| (r[1] - footer_top).abs() < lh * 0.5),
            "{world}: no drawn plate sits at the planned footer top {footer_top:.1} \
             (plates {plates:?}) — arm 1 must be watching a real plate"
        );

        // ARM 2 — the pixels.
        let pixels = shoot(&device, &queue, &mut p);
        let bands = (geom.card_y, notice_top, footer_top, lh);
        if notice_reads_as_ground(&pixels, plan.card_x_span(), bands, world) {
            pixel_graded.push(world);
        }
    }
    crate::render::set_list_style_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        !pixel_graded.is_empty(),
        "at least one Bars world's plate must be visible enough to grade from pixels; \
         otherwise arm 2 is vacuous across the whole roster"
    );
}

/// Render the current frame offscreen and read it back.
fn shoot(device: &wgpu::Device, queue: &wgpu::Queue, p: &mut TextPipeline) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, 1200, 800);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item174 empty-card encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, 1200, 800)
}

/// ARM 2 of the empty-state law: does the NOTICE row's band read as plain card
/// ground rather than as the footer's plate? `bands` is `(card_y, notice_top,
/// footer_top, lh)`. Returns whether this world's plate was visible enough for the
/// absolute oracle to grade at all; a `false` here means arm 1 carries the world.
fn notice_reads_as_ground(
    pixels: &[[u8; 4]],
    card_x: (f32, f32),
    bands: (f32, f32, f32, f32),
    world: &str,
) -> bool {
    /// The luminance gap, against the card's own top-pad ground, at which the
    /// footer plate becomes visible to an absolute pixel oracle. Below it a pale
    /// world's plate is a whisper and only arm 1 can speak.
    const VISIBLE_PLATE_LUMA: f64 = 15.0;

    let (card_y, notice_top, footer_top, lh) = bands;
    let x_lo = (card_x.0 + 2.0).max(0.0) as u32;
    let x_hi = (card_x.1 - 2.0).min(1199.0) as u32;
    let mean_luma = |y0: f32, y1: f32| -> f64 {
        let (a, b) = (y0.ceil().max(0.0) as u32, y1.floor().min(800.0) as u32);
        let (mut sum, mut n) = (0.0f64, 0u32);
        for y in a..b {
            for x in x_lo..x_hi {
                let px = pixels[(y * 1200 + x) as usize];
                sum += 0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64;
                n += 1;
            }
        }
        sum / n.max(1) as f64
    };
    // The card's own TOP PAD: pure ground, no glyph ink, on every world.
    let ground = mean_luma(card_y + 1.0, card_y + 10.0);
    let pad = (lh * 0.25).max(4.0);
    let notice = mean_luma(notice_top + pad, notice_top + lh - pad);
    let footer = mean_luma(footer_top + pad, footer_top + lh - pad);
    let plate_delta = (footer - ground).abs();
    let notice_delta = (notice - ground).abs();
    if plate_delta <= VISIBLE_PLATE_LUMA {
        return false;
    }
    assert!(
        notice_delta * 3.0 < plate_delta,
        "{world}: the empty-state NOTICE row reads as a plated footer band rather than \
         plain card ground (ground {ground:.2}, notice {notice:.2}, footer {footer:.2}; \
         notice delta {notice_delta:.2} vs plate delta {plate_delta:.2})"
    );
    true
}

// --- The retired-arithmetic source law ---------------------------------------

/// The sentinel TERMS of the row-Y arithmetic item 174 moved into the planner.
/// A production line under `src/render/` that contains one is re-deriving a row's
/// position from loose scalars — the shape that let the draw path, the hit-test,
/// the band and the footer plate each carry their own copy.
const RETIRED_TERMS: &[&str] = &[
    "overlay_row_top(",
    "overlay_row_of(",
    "overlay_row_index(",
    "header_rows as f32",
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

/// The ONLY place the arithmetic may live.
const ARITHMETIC_OWNER: &str = "plan/overlay_rows.rs";

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
        .filter(|(f, _)| f != ARITHMETIC_OWNER)
        .map(|(f, l)| format!("  {f}:{l}"))
        .collect();
    assert!(
        strays.is_empty(),
        "only `{ARITHMETIC_OWNER}` may derive an overlay candidate row's position. A \
         consumer that computes its own row y is a parallel calculation: the drawn band, \
         the click that lands in it and the sidecar's report of it will agree until the \
         day one of them is edited. Read the row off `OverlayRowPlan` instead. \
         offending lines:\n{}",
        strays.join("\n")
    );

    // NON-VACUOUS: the owner really does carry the arithmetic, so a refactor that
    // moved it elsewhere trips this law instead of silently emptying it.
    assert!(
        hits.iter().any(|(f, _)| f == ARITHMETIC_OWNER),
        "`{ARITHMETIC_OWNER}` must actually contain the row arithmetic — this law is \
         scanning for something that no longer exists"
    );
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
