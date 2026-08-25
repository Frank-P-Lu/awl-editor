//! THE HEADER BAND, AGAINST THE REAL PIPELINE.
//!
//! The pure laws live beside the planner (`render/plan/tests.rs`); this file is
//! the device-level half. Its subject is the two display lines ABOVE the
//! candidate band — the query/title INPUT field every takeover picker draws, and
//! the grouped family's lens STRIP — and the three answers about them that used
//! to be computed in four separate `render/chrome` owners must be ONE planned
//! object:
//!
//! * DRAWN — the shaped line's own box, read back out of `panel_buffer`, the
//!   buffer the draw pass uploads. Never rebuilt from arithmetic.
//! * INTERACTIVE — the y-extent `over_overlay_query` / `overlay_lens_at` accept.
//! * DECORATIVE — where the query caret is centred and where the active-lens
//!   mark is centred, read back off the recorded quads.
//!
//! **EVERY ORACLE IS DERIVED INDEPENDENTLY OF THE ACCESSOR IT GRADES.** The
//! first family's own hard lesson was a device law whose oracle called the same
//! function as the code under test: it stayed green while pointing at the wrong
//! row. So "drawn" here means glyph metrics out of the shaped buffer, and
//! "interactive" means the production hit-test's own yes/no answer sampled at
//! real y values — not two reads of `plan.query_band()`.
//!
//! **THE DEFECT THIS FILE WAS WRITTEN AROUND.** `over_overlay_query` tested the
//! bare row pitch, `text_top .. text_top + lh`. On the FLAT family the query
//! BEAT inflates the query line ITSELF (`shape_overlay_names`'s `header_lh`), so
//! the field's box is `lh + header_gap` tall and cosmic-text half-leads its ink
//! LOW inside it: the pointer band ended above the caret and above the glyphs.
//! On the GROUPED family the beat inflates the lens STRIP instead, leaving the
//! query line a plain `lh` — so the same wrong reading was right there. That
//! asymmetry is why the sweep below runs BOTH families and asserts the pre-fix
//! formula's miss by name (`the_pre_plan_query_band_genuinely_missed_the_field`).

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::OverlayKind;

/// The same family split the plan law uses, derived from production's own
/// classifier rather than a hand-copied match, which can silently drift.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    Flat,
    Grouped,
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

fn overlay_view(kind: OverlayKind, n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title().to_string();
    v.overlay_query = "co".into();
    v.overlay_query_caret = 2;
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
            v.overlay_query = String::new();
            v.overlay_query_caret = 0;
            v.overlay_selected = 0;
            v.overlay_hint = String::new();
        }
        Family::Flat => {}
    }
    v
}

/// Grade ONE rendered card's header band: the query field's box against the
/// shaped run, the pointer, the caret; then the lens strip against the shaped
/// strip line and `overlay_lens_at`. Split out of the sweep so each concern
/// reads as one claim rather than one long scroll.
fn grade_header_band(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    geom: &crate::render::chrome::OverlayGeom,
    pr: &super::overlay_probe::OverlayYProbe,
    fam: Family,
    ctx: &str,
    tally: (&mut usize, &mut usize),
) {
    let (fields, strips) = tally;
    let (x0, x1) = plan.card_x_span();
    let mid_x = (x0 + x1) * 0.5;
    // WHICH kinds have a field is asserted BEFORE the branch, not
    // inferred from it: a `let else` that only checks the kind on
    // the `None` arm cannot see a contextual popup that GREW one
    // (it would sail into the main arm, where every consumer reads
    // the same plan and so agrees with it perfectly).
    assert_eq!(
        plan.query_band().is_some(),
        fam != Family::Contextual,
        "{ctx}: exactly the takeover pickers draw a query line — a \
         contextual popup has no field to centre a caret in, hit-test, \
         or split a surface at"
    );
    let Some(field) = plan.query_band() else {
        // A contextual popup plans no field, so nothing may
        // accept a pointer as one, anywhere down the card.
        let card = geom.card_probe();
        let mut y = card[1];
        while y < card[1] + card[3] {
            assert!(
                !p.over_overlay_query(mid_x, y),
                "{ctx}: a card with no query line accepted a pointer at \
                 y={y} as if it had one"
            );
            y += plan.lh() * 0.25;
        }
        *fields += 1;
        return;
    };

    // --- DRAWN == PLANNED -------------------------------------
    // `query_line_top`/`query_line_height` are read off the shaped
    // run the draw pass uploads, not off the plan.
    assert!(
        (pr.query_line_top - field.top).abs() < 0.75,
        "{ctx}: the query line is DRAWN at {} but PLANNED at {}",
        pr.query_line_top,
        field.top
    );
    assert!(
        (pr.query_line_height - field.height).abs() < 0.75,
        "{ctx}: the query line is DRAWN {} tall but PLANNED {} — the \
         beat belongs to the field's own box",
        pr.query_line_height,
        field.height
    );

    // --- INTERACTIVE == DRAWN ---------------------------------
    // Sampled through the production hit-test's own yes/no answer.
    for (label, y, want) in [
        ("just above the field", field.top - 1.0, false),
        ("the field's top edge", field.top + 0.1, true),
        ("the field's centre", field.center(), true),
        ("just inside the bottom", field.bottom() - 0.1, true),
        ("the field's bottom edge", field.bottom() + 0.1, false),
    ] {
        assert_eq!(
            p.over_overlay_query(mid_x, y),
            want,
            "{ctx}: the pointer at {label} (y={y}) must{} read as the \
             query field — planned box [{}, {}]",
            if want { "" } else { " not" },
            field.top,
            field.bottom()
        );
    }
    // …and off the CARD horizontally it is never the field. Probed against the
    // card's own left edge, not the content band's: a workspace's band starts
    // after its navigation rail, and the field's x-bound is the card (which this
    // slice deliberately left alone — its subject is the band's Y).
    assert!(
        !p.over_overlay_query(geom.card_probe()[0] - 40.0, field.center()),
        "{ctx}: a pointer left of the card is not on the field"
    );

    // --- THE INK IS INSIDE THE BAND ---------------------------
    // THE DEFECT'S OWN SHAPE, graded from the shaped BASELINE (a
    // glyph fact, not arithmetic): the row the query's ink sits on
    // must be inside the band the pointer accepts. Before this
    // family the baseline sat up to 30px BELOW the accepted band.
    assert!(
        p.over_overlay_query(mid_x, pr.query_baseline),
        "{ctx}: the query's own shaped baseline (y={}) is not inside \
         the band the pointer accepts as the field — the I-beam is \
         somewhere the text is not",
        pr.query_baseline
    );

    // --- THE CARET IS CENTRED IN ITS OWN FIELD ----------------
    assert!(
        field.contains(pr.caret_center),
        "{ctx}: the caret centre {} is outside the field box [{}, {}]",
        pr.caret_center,
        field.top,
        field.bottom()
    );
    assert!(
        p.over_overlay_query(mid_x, pr.caret_center),
        "{ctx}: the pointer does not accept the field where its own \
         caret is drawn"
    );
    *fields += 1;

    grade_lens_strip(p, plan, pr, fam, ctx, field, strips);
}

/// The GROUPED family's lens strip: its planned box against the shaped strip
/// line, and `overlay_lens_at` against that same box's edges.
fn grade_lens_strip(
    p: &TextPipeline,
    plan: &crate::render::plan::OverlayRowPlan,
    pr: &super::overlay_probe::OverlayYProbe,
    fam: Family,
    ctx: &str,
    field: crate::render::plan::PlannedHeader,
    strips: &mut usize,
) {
    let (x0, x1) = plan.card_x_span();
    let mid_x = (x0 + x1) * 0.5;
    // --- THE LENS STRIP ---------------------------------------
    let Some(strip) = plan.strip_band() else {
        assert_ne!(
            fam,
            Family::Grouped,
            "{ctx}: a grouped card must plan a lens strip"
        );
        assert!(
            p.overlay_lens_at(mid_x, field.center()).is_none(),
            "{ctx}: a card with no strip may not answer a lens hit"
        );
        return;
    };
    assert_eq!(fam, Family::Grouped, "{ctx}: only a grouped card strips");
    let drawn_top = pr
        .strip_line_top
        .unwrap_or_else(|| panic!("{ctx}: a grouped card shapes line 1"));
    let drawn_bottom = pr.strip_line_bottom.unwrap();
    assert!(
        (drawn_top - strip.top).abs() < 0.75 && (drawn_bottom - strip.bottom()).abs() < 0.75,
        "{ctx}: the strip is DRAWN [{drawn_top}, {drawn_bottom}] but \
         PLANNED [{}, {}]",
        strip.top,
        strip.bottom()
    );
    // The strip box is BELOW the query field and they abut.
    assert!(
        (strip.top - field.bottom()).abs() < 1e-3,
        "{ctx}: the strip must start where the query field ends \
         ({} vs {})",
        strip.top,
        field.bottom()
    );

    // INTERACTIVE: find an x the lens hit-test genuinely claims at
    // the planned strip centre, then prove the SAME x is refused
    // just outside the planned box in both directions.
    let mut lens_x = None;
    let mut x = x0;
    while x < x1 {
        if p.overlay_lens_at(x, strip.center()).is_some() {
            lens_x = Some(x);
            break;
        }
        x += 2.0;
    }
    if let Some(lx) = lens_x {
        *strips += 1;
        assert!(
            p.overlay_lens_at(lx, strip.top + 0.1).is_some(),
            "{ctx}: the strip's own top edge must accept a lens click"
        );
        assert!(
            p.overlay_lens_at(lx, strip.bottom() - 0.1).is_some(),
            "{ctx}: just inside the strip's bottom must accept a lens click"
        );
        assert!(
            p.overlay_lens_at(lx, strip.top - 1.0).is_none(),
            "{ctx}: above the planned strip box is not the strip"
        );
        assert!(
            p.overlay_lens_at(lx, strip.bottom() + 0.1).is_none(),
            "{ctx}: below the planned strip box is not the strip"
        );
    }
}

/// THE HEADLINE LAW. Over the WHOLE `OverlayKind` roster (all three families),
/// both list styles, four canvases and both DPIs: the query field's PLANNED box
/// is the box the shaper DREW, is the band the pointer ACCEPTS, and is the box
/// the caret is centred in — and the grouped family's lens strip likewise.
#[test]
fn the_drawn_query_field_the_pointer_band_and_the_caret_are_one_planned_box() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_drawn_query_field_...: no wgpu adapter");
        return;
    };

    let styles: [(&str, Option<theme::ListStyle>); 2] = [
        ("pane", Some(theme::ListStyle::Pane)),
        ("bars", Some(theme::ListStyle::Bars)),
    ];
    // Harmless for the "pane" arm above (nothing reads it when the resolved
    // style isn't `Bars`); set once rather than threading a second array.
    crate::render::set_bar_config_test_override(Some(theme::BarConfig {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: theme::BarExtent::FullWidth,
        coverage: theme::BarCoverage::All,
    }));
    // LOGICAL canvases; physical is `logical * dpi`. Same convention (and same
    // reason) as `overlay_plan_law.rs`.
    let canvases: [(u32, u32); 4] = [(1200, 800), (700, 800), (900, 460), (1400, 1600)];

    // Per-family non-vacuity counters: an aggregate floor is exactly how a whole
    // family's arm can quietly go to zero.
    let mut fields_by_family: [usize; 3] = [0, 0, 0];
    let mut strips_graded = 0usize;
    let fam_idx = |f: Family| match f {
        Family::Flat => 0,
        Family::Grouped => 1,
        Family::Contextual => 2,
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
                    let pr = p.overlay_row_y_probe();
                    let ctx = format!("{kind:?}/{fam:?} dpi={dpi} list={sname} canvas={cw}x{ch}");

                    grade_header_band(
                        &p,
                        &plan,
                        &geom,
                        &pr,
                        fam,
                        &ctx,
                        (&mut fields_by_family[fam_idx(fam)], &mut strips_graded),
                    );
                }
            }
        }
    }
    crate::render::set_list_style_test_override(None);
    crate::render::set_bar_config_test_override(None);
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);

    for (name, idx) in [("Flat", 0), ("Grouped", 1), ("Contextual", 2)] {
        assert!(
            fields_by_family[idx] > 0,
            "the {name} family's own arm graded nothing — it is vacuous, got \
             {fields_by_family:?}"
        );
    }
    assert!(
        strips_graded > 0,
        "the lens-strip pointer arm never found a claimable label — it is vacuous"
    );
}

/// **THE QUERY BAR HOLDS THE QUERY FIELD AND NOTHING ELSE**, and a beat folded
/// into that field genuinely displaces its own glyphs.
///
/// The claim, over the whole roster: the field's planned box ends exactly one
/// breathe above the split seam, so the upper surface closes below the bar's own
/// line and the ink sits inside it with a top pad and a bottom pad. The
/// NON-VACUITY half reconstructs the folded-beat box INLINE
/// (`text_top .. text_top + lh + header_gap`) — never read back out of the fix —
/// and asserts that on the FLAT family that box would run PAST the seam, so the
/// bar could not close above the gap and the glyphs it centres would land most
/// of a beat below the bar's own top pad. The GROUPED family is asserted
/// UNAFFECTED by the change, which is the point: the two arms disagreed, and the
/// arm nobody looked at is the one that was wrong.
///
/// Swept over the whole world roster and both DPIs, because the displacement
/// scales with the row pitch and no world may be quietly exempt.
#[test]
fn the_query_bar_closes_below_the_field_and_the_folded_beat_would_overrun_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_query_bar_closes_below_the_field...: no adapter");
        return;
    };
    let mut worst_flat_overrun = 0.0f32;
    let mut flat_cells = 0usize;
    let mut grouped_cells = 0usize;

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for world in crate::theme::world_names() {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            for kind in [OverlayKind::Keybindings, OverlayKind::Command] {
                let fam = family(kind);
                let v = overlay_view(kind, 24);
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let pr = p.overlay_row_y_probe();
                let field = plan.query_band().expect("both kinds draw a query line");
                let (seam_top, seam_bottom) = plan
                    .split_bounds()
                    .expect("a card with a header and a beat splits");
                let ctx = format!("{world}/{kind:?}/{fam:?} dpi={dpi}");

                // --- THE CLAIM -----------------------------------------------
                // The bar closes below the field, and the field's own ink is
                // inside the bar with room on both sides of it.
                assert!(
                    field.bottom() <= seam_top + 0.01,
                    "{ctx}: the query field's box [{}, {}] runs past the surface seam \
                     at {seam_top} — the bar cannot close above the gap it belongs to",
                    field.top,
                    field.bottom()
                );
                assert!(
                    pr.query_baseline < seam_top && pr.caret_center < seam_top,
                    "{ctx}: the field's shaped baseline ({}) / caret ({}) fall at or \
                     below the seam ({seam_top}) — its own ink is drawn onto the gap",
                    pr.query_baseline,
                    pr.caret_center
                );

                // --- NON-VACUITY: THE FOLDED-BEAT BOX, WRITTEN OUT HERE -------
                let folded_bottom = geom.text_top + plan.lh() + p.overlay_header_gap();
                match fam {
                    Family::Flat => {
                        flat_cells += 1;
                        assert!(
                            folded_bottom > seam_bottom,
                            "{ctx}: the fixture must reproduce the reported defect — a \
                             field folded to {folded_bottom} has to run past the seam's \
                             own end ({seam_bottom}) for this law to mean anything"
                        );
                        // Its glyph run would centre HERE, most of a beat below
                        // the bar's own top pad.
                        let folded_center = (geom.text_top + folded_bottom) * 0.5;
                        worst_flat_overrun = worst_flat_overrun.max(folded_center - field.center());
                    }
                    Family::Grouped => {
                        grouped_cells += 1;
                        // The grouped card's own beat still rides its STRIP, so
                        // its field and its seam are untouched: the folded box IS
                        // the strip's box here — UNLESS the active `FacetStyle`
                        // docks that strip outside the card (`docked_facet_band`),
                        // in which case the strip's own box charges no `lh` of
                        // its own and the folded box is the field's alone.
                        let strip = plan.strip_band().expect("a grouped card draws a strip");
                        let docked = matches!(
                            crate::render::effective_facet_style(),
                            theme::FacetStyle::DockedTab
                        );
                        let want = if docked {
                            folded_bottom
                        } else {
                            folded_bottom + plan.lh()
                        };
                        assert!(
                            (strip.bottom() - want).abs() < 0.75,
                            "{ctx}: a grouped card's beat must still inflate its strip \
                             (docked={docked}) — strip box [{}, {}], wanted bottom {want}",
                            strip.top,
                            strip.bottom()
                        );
                    }
                    Family::Contextual => unreachable!("neither kind is contextual"),
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        flat_cells > 20 && grouped_cells > 20,
        "both arms must be swept"
    );
    assert!(
        worst_flat_overrun > 5.0,
        "the folded beat's worst glyph displacement across the roster was only \
         {worst_flat_overrun}px — too small to be the reported blank band; the \
         fixture has stopped reproducing it"
    );
}

/// THE LAST MEMBER OF THIS FAMILY TO MERGE — and the reason it had to (item
/// 116d).
///
/// `comparison_viewport` answers "how far below `text_top` does a workspace's
/// content begin" from the resolved `WorkspaceFrame`: the same fit that can
/// yield header chrome under height pressure. It feeds those resolved rows and
/// beat into the plan module's own `header_band_height`, rather than carrying a
/// second cached header-band answer on `TextPipeline`.
///
/// The law is that the arithmetic OWNER it calls and the row PLAN the pixels
/// came from agree — over the world roster, both DPIs, and **both workspace
/// shapes**, because the one-line shape alone is exactly the sweep that would
/// have gone green over this bug.
#[test]
fn the_workspace_header_band_still_agrees_with_the_planned_header_boxes() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_workspace_header_band_still_agrees_...: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    let mut two_line_cells = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for world in crate::theme::world_names() {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            // BOTH SHAPES. `rows_primary` is `sync_view`'s own projection of
            // `WorkspaceShape::rows_are_primary`, so driving it here drives the
            // production seam: `false` is Settings' rail-over-rows (one header
            // line), `true` is History's timeline-over-comparison (two — the
            // search line and the lens strip that has nowhere else to live).
            for rows_primary in [false, true] {
                let mut v = overlay_view(OverlayKind::Settings, 12);
                v.overlay_workspace = true;
                v.overlay_rows_primary = rows_primary;
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let geom = p.overlay_geometry(cw);
                let plan = p.overlay_row_plan(&geom);
                let heads = plan.header_lines();
                let field = plan
                    .query_band()
                    .expect("a workspace draws its own search line");
                let last = *heads.last().expect("a workspace has a header band");
                // THE ORACLE IS THE PLANNED BOXES, not the same expression again:
                // the band's run is the last planned box's bottom PLUS whatever
                // beat stands on its own after it (`beat_line` — a one-line
                // workspace puts the beat after its search line rather than inside
                // it), measured back to `text_top`, which is where the first
                // candidate row begins.
                let planned = last.bottom() + plan.beat_line().unwrap_or(0.0) - field.top;
                let frame = p.workspace_frame(cw);
                let band = crate::render::plan::header_band_height(
                    frame.fit.header_rows,
                    p.overlay_lh(),
                    frame.fit.header_gap,
                );
                assert!(
                    (band - planned).abs() < 1e-3,
                    "{world} dpi={dpi} rows_primary={rows_primary}: \
                     resolved workspace header band ({band}) has drifted from the planned header \
                     band ({planned}, {} lines) — the comparison pane would be seated off \
                     the line the workspace's own rows begin on",
                    heads.len()
                );
                assert!(
                    (plan.first_top() - (field.top + planned)).abs() < 1e-3,
                    "{world} dpi={dpi} rows_primary={rows_primary}: the header band must \
                     close exactly on the candidate band's first row"
                );
                assert_eq!(
                    heads.len(),
                    1 + usize::from(rows_primary),
                    "{world} dpi={dpi}: a timeline workspace carries its lens as a SECOND \
                     header line; a rail workspace carries it in the rail"
                );
                if rows_primary {
                    two_line_cells += 1;
                }
                // …and the workspace's own search line gets the SAME drawn/pointer/
                // caret grading the takeover cards get. The headline sweep cannot
                // reach it: its fixture leaves `overlay_workspace` off, so Settings
                // is graded there as the GROUPED card it becomes when it is NOT
                // relocated.
                let pr = p.overlay_row_y_probe();
                let ctx = format!("{world} dpi={dpi} workspace rows_primary={rows_primary}");
                let (mut fields, mut strips) = (0usize, 0usize);
                grade_header_band(
                    &p,
                    &plan,
                    &geom,
                    &pr,
                    match rows_primary {
                        true => Family::Grouped,
                        false => Family::Flat,
                    },
                    &ctx,
                    (&mut fields, &mut strips),
                );
                assert_eq!(fields, 1, "{ctx}: the workspace's field must be graded");
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 40,
        "the roster sweep must actually run, got {graded}"
    );
    // NON-VACUITY on the axis that broke: a sweep that never reached the TWO-line
    // header would have gone green over the very drift this law is named for.
    assert!(
        two_line_cells > 20,
        "the sweep must reach the two-line header shape, got {two_line_cells}"
    );
}
