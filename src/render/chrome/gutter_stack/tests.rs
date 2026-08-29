use super::*;
use crate::workingset::StackRow;

fn row(leaf: &str, parent: &str, active: bool) -> StackRow {
    StackRow {
        leaf: leaf.to_string(),
        parent: parent.to_string(),
        active,
        kind: crate::workingset::StackRowKind::File,
    }
}

/// A project HEADING row, the outer `active` field set the same way
/// [`crate::workingset::WorkingSet::expanded_rows`] now sets it — mirroring
/// [`crate::workingset::StackRowKind::Group`]'s own copy rather than
/// disagreeing with it.
fn group_row(leaf: &str, active: bool) -> StackRow {
    StackRow {
        leaf: leaf.to_string(),
        parent: String::new(),
        active,
        kind: crate::workingset::StackRowKind::Group { active },
    }
}

/// The three-file set every block-shape sweep below draws, with `active` naming
/// which slot is the reader's current file.
fn rows(active: usize) -> Vec<StackRow> {
    ["opening.md", "field-notes.md", "ledger.md"]
        .into_iter()
        .zip(["", "journal/", ""])
        .enumerate()
        .map(|(at, (leaf, parent))| row(leaf, parent, at == active))
        .collect()
}

fn layout_of(files: &[StackRow], changed: bool, project: bool, budget: usize) -> GutterLayout {
    GutterLayout {
        avail: 120.0,
        name: "opening.md".to_string(),
        project: if project {
            "notes".to_string()
        } else {
            String::new()
        },
        changed: if changed {
            "changed elsewhere".to_string()
        } else {
            String::new()
        },
        files: fit_rows(files, budget),
    }
}

/// THE LEAF SURVIVES, THE LOCATION YIELDS. Swept across the whole budget range
/// from "everything fits" down to the gutter's own hard floor, because the
/// interesting behaviour is entirely in the middle: a budget wide enough for the
/// name but not the path is exactly where a location-first fit would start
/// eating the filename.
#[test]
fn fit_rows_spend_the_budget_on_the_leaf_before_the_location() {
    let deep = row("final-draft.md", "research/sources/drafts/", true);
    for budget in crate::render::rowlayout::GUTTER_MIN_NAME_CHARS..40 {
        let fitted = fit_rows(std::slice::from_ref(&deep), budget);
        let line = &fitted[0];
        let leaf = &line.text[line.parent_byte..];
        assert!(
            !leaf.is_empty(),
            "budget {budget}: the filename was elided away entirely, leaving {:?}",
            line.text
        );
        assert!(
            line.text.chars().count() <= budget,
            "budget {budget}: row {:?} overflows its line",
            line.text
        );
        // A location that cannot be told the truth about is not drawn: the only
        // shapes allowed are the whole path, its elided form, or nothing.
        let parent = &line.text[..line.parent_byte];
        assert!(
            parent.is_empty() || parent == "research/sources/drafts/" || parent == "research/…/",
            "budget {budget}: invented a location {parent:?}"
        );
    }
    // And the deep path DOES get elided rather than dropped whenever there is
    // room for its elided form — otherwise this law would pass on a fit that
    // simply never draws a location at all.
    let mid = fit_rows(std::slice::from_ref(&deep), 29);
    assert_eq!(&mid[0].text[..mid[0].parent_byte], "research/…/");
}

/// A long nested label used to consume the whole character budget before the
/// always-shaped close run was appended. Cosmic text then wrapped that run onto
/// a new visual row, shifting every following glyph while the device-free row
/// planner (and its active plate) correctly stayed put.
#[test]
fn file_rows_reserve_the_close_lane_inside_their_one_line_budget() {
    for budget in crate::render::rowlayout::GUTTER_MIN_NAME_CHARS..40 {
        let fitted = fit_rows(
            &[row(
                "field-notes-with-a-long-name.md",
                "journal/research/",
                false,
            )],
            budget,
        );
        let occupied = fitted[0].text.chars().count() + super::CLOSE_MARK_TEXT.chars().count();
        assert!(
            occupied <= budget,
            "budget={budget}: label + stable close lane occupies {occupied} characters"
        );
    }

    // Prototype-only non-file rows carry no close lane, so their copy keeps the
    // whole budget rather than paying for an affordance they can never reveal.
    let more = crate::workingset::StackRow {
        leaf: "+ 12 more…".to_string(),
        kind: crate::workingset::StackRowKind::More { hidden: 12 },
        ..crate::workingset::StackRow::default()
    };
    let fitted = fit_rows(&[more], 8);
    assert_eq!(fitted[0].text.chars().count(), 8);
}

/// A GROUP HEADING RESERVES THE SAME CLOSE LANE A FILE ROW DOES — its own
/// mark closes the whole group, so a long project name must not be allowed to
/// wrap that run onto a second visual line the way an un-reserved file label
/// once did.
#[test]
fn group_headings_reserve_the_close_lane_inside_their_one_line_budget() {
    for budget in crate::render::rowlayout::GUTTER_MIN_NAME_CHARS..40 {
        let fitted = fit_rows(&[group_row("a-long-nested-project-folder/", true)], budget);
        let occupied = fitted[0].text.chars().count() + super::CLOSE_MARK_TEXT.chars().count();
        assert!(
            occupied <= budget,
            "budget={budget}: heading + stable close lane occupies {occupied} characters"
        );
    }
}

/// THE ACTIVE ROW'S NAME COMES FORWARD and every location stays quieter than the
/// name it qualifies. Asserted as INK IDENTITY between spans of one frame, never
/// against an authored constant, and swept over which row is active so the law
/// cannot pass by pinning slot 0.
#[test]
fn stack_spans_bring_only_the_active_name_forward() {
    // `stack_spans` now reads the process-global active theme (the plate's own
    // fill, to pick the active row's ink) — this held the guard's return value
    // in a discarded temporary before, an unguarded reader of a swappable
    // global (CLAUDE.md's testlock discipline) that happened to work only
    // because nothing else in this test ever wrote the theme.
    let _g = crate::testlock::serial();
    for active in 0..3 {
        let fitted = fit_rows(&rows(active), 27);
        let spans = stack_spans(&fitted, None);
        let name_inks: Vec<_> = spans
            .iter()
            .filter(|(text, _)| text.contains(".md"))
            .map(|(_, ink)| *ink)
            .collect();
        assert_eq!(name_inks.len(), 3, "one name span per row");
        let forward = name_inks[active];
        for (at, ink) in name_inks.iter().enumerate() {
            if at == active {
                continue;
            }
            assert_ne!(
                forward.0, ink.0,
                "active row {active} wears the same ink as sibling row {at}"
            );
        }
        // The nested row's location is quieter than its own name only where that
        // name is forward; everywhere else both are already the quiet ink.
        let location = spans
            .iter()
            .find(|(text, _)| text.ends_with("journal/"))
            .expect("the nested row draws its location");
        if active == 1 {
            assert_ne!(
                location.1.0, forward.0,
                "the active row's location must stay quieter than its name"
            );
        }
    }
}

/// EXACTLY ONE PLATE, ON THE ACTIVE ROW, IN EVERY BLOCK SHAPE.
///
/// The axis that matters is not which row is active but WHAT ELSE IS IN THE
/// BLOCK: the affordance line and the project line each appear and vanish
/// independently, so the active file's slot in the working set and its row index
/// in the drawn block are different numbers. A plate placed from the former
/// lands on the wrong line the moment a conflict raises the affordance — which
/// is why the plate is derived from `lines()` and this sweeps all four shapes.
#[test]
fn a_plate_marks_the_active_row_in_every_block_shape() {
    for changed in [false, true] {
        for project in [false, true] {
            for active in 0..3 {
                let files = rows(active);
                let layout = layout_of(&files, changed, project, 24);
                let lines = layout.lines();
                let plan = crate::render::plan::plan_gutter_stack(
                    300.0,
                    layout.avail,
                    12.0,
                    lines.len(),
                    8.0,
                    0.5,
                );
                let plates = plate_rects(&layout, &plan, 6.0, 2.0);
                let shape = format!("changed={changed} project={project} active={active}");
                assert_eq!(plates.len(), 1, "{shape}: expected exactly one plate");
                let at = lines
                    .iter()
                    .position(
                        |(_, kind)| matches!(kind, gutter::GutterLine::File(i) if *i == active),
                    )
                    .unwrap_or_else(|| panic!("{shape}: the active row is not in the block"));
                let band = plan.rows[at];
                let plate = plates[0];
                assert!(
                    plate[1] >= band[1] && plate[1] + plate[3] <= band[1] + band[3],
                    "{shape}: plate {plate:?} escapes its own row band {band:?}"
                );
                // Never meets the row above or below, at any block shape.
                for (other, neighbour) in plan.rows.iter().enumerate() {
                    if other == at {
                        continue;
                    }
                    assert!(
                        plate[1] + plate[3] <= neighbour[1]
                            || plate[1] >= neighbour[1] + neighbour[3],
                        "{shape}: plate {plate:?} overlaps row {other} at {neighbour:?}"
                    );
                }
                // Hugs the writing column on exactly the convention the gutter's
                // own frost pill already uses: the ink is right-aligned to the
                // box, and the treatment breathes one pad past it.
                assert!(
                    (plate[0] + plate[2] - (band[0] + band[2] + 2.0)).abs() < 0.01,
                    "{shape}: plate {plate:?} does not hug the column like the frost pill"
                );
                // The transparent close run still advances the right-aligned
                // label. A plate sized from visible label characters alone
                // leaves that many characters outside its fill; Wagtail then
                // draws black selected ink on the black page and loses them.
                let text_chars = lines[at].0.chars().count();
                let occupied_chars = text_chars + super::CLOSE_MARK_TEXT.chars().count();
                let expected_left =
                    (band[0] + band[2] - occupied_chars as f32 * 6.0 - 2.0).max(0.0);
                assert!(
                    (plate[0] - expected_left).abs() < 0.01,
                    "{shape}: plate {plate:?} leaves its reserved close lane outside the fill"
                );
            }
        }
    }
}

/// A BLOCK WITH NO WORKING SET HAS NO PLATE — the single-file margin's whole
/// contract, asserted at the geometry seam rather than trusted from the model.
#[test]
fn a_single_file_block_plates_nothing() {
    let layout = layout_of(&[], false, true, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    assert!(plate_rects(&layout, &plan, 6.0, 2.0).is_empty());
    assert_eq!(
        stack_spans(&layout.files, None).len(),
        0,
        "an empty stack must not grow a reserved close lane"
    );
    // And the block is exactly the two lines it has always been, in the same
    // top-to-bottom order the stack shape uses (`project_heads_only_the_
    // multi_file_hierarchy` pins the shared ordering; this law's own subject
    // is that a single file still plates nothing).
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|(_, kind)| *kind)
            .collect::<Vec<_>>(),
        vec![gutter::GutterLine::Project, gutter::GutterLine::Name]
    );
}

/// **515: AN ACTIVE GROUP HEADING NEVER PLATES, EVEN WITH ITS OWN ACTIVE
/// FILE VISIBLE IN THE SAME WINDOW.** Superseded 507's law of the same
/// fixture shape (`an_active_group_heading_and_its_active_file_are_both_
/// plated`), which asserted the double-plate this item exists to remove:
/// a screenshot caught the expanded panel drawing two purple plates for one
/// project — the heading (current project) and the active file (current
/// document) — reading as two selections when only one answer, "which
/// file", owns a fill. `plate_rects`'s law above
/// (`a_plate_marks_the_active_row_in_every_block_shape`) only ever proved
/// "at most one" because its own fixture is File rows exclusively — the
/// resting stack's real shape, which never draws a Group row at all; this
/// sweeps the EXPANDED panel's real shape instead.
#[test]
fn an_active_group_heading_never_plates_only_its_active_file_does() {
    let files = vec![
        group_row("notes/", true),
        row("welcome.md", "", true),
        row("draft.md", "", false),
    ];
    let layout = layout_of(&files, false, false, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    let plates = plate_rects(&layout, &plan, 6.0, 2.0);
    assert_eq!(
        plates.len(),
        1,
        "only the active FILE may plate, never its group heading too: {plates:?}"
    );
}

/// A heading that is NOT the reader's current project draws no plate either
/// — and neither does the active project's own heading, only its active
/// file — while sitting beside another (inactive) project's heading and file.
/// `StackRow::active` combined with `StackRowKind::File` is the plate's
/// whole source of truth; a Group's own `active` field still drives its ink
/// ([`stack_spans`]) but never its fill.
#[test]
fn only_the_active_file_ever_plates_never_any_group_heading() {
    let files = vec![
        group_row("archive/", false),
        row("old.md", "", false),
        group_row("notes/", true),
        row("welcome.md", "", true),
    ];
    let layout = layout_of(&files, false, false, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    let plates = plate_rects(&layout, &plan, 6.0, 2.0);
    assert_eq!(
        plates.len(),
        1,
        "only the active file may plate: {plates:?}"
    );
}

/// **507: AN ACTIVE HEADING WEARS THE SAME ROUTED INK AN ACTIVE FILE DOES.**
/// Before this fix a heading's ink came from the ladder's plain `muted`
/// default, never [`theme::selected_row_secondary_ink`] — exactly the shape
/// this file's own tripwire names for File rows (a plate that fills at
/// page-inverse on Wagtail swallows unrouted ink). Now that a heading plates
/// too, it must be routed the same way.
#[test]
fn an_active_group_heading_wears_the_same_routed_ink_as_an_active_file() {
    let _g = crate::testlock::serial();
    let files = vec![group_row("notes/", true), row("welcome.md", "", true)];
    let fitted = fit_rows(&files, 24);
    let spans = stack_spans(&fitted, None);
    let heading_ink = spans
        .iter()
        .find(|(text, _)| text.contains("notes/"))
        .expect("the heading draws its own name span")
        .1;
    let file_ink = spans
        .iter()
        .find(|(text, _)| text.contains("welcome.md"))
        .expect("the file draws its own name span")
        .1;
    assert_eq!(
        heading_ink.0, file_ink.0,
        "an active heading and an active file must share the same routed ink"
    );
}

/// The production close mark is a one-stage color-only reveal over one
/// already-shaped trailing run, enrolled from the same row/zone geometry the
/// click consumes. The active document participates: "hovered" names the row
/// under the pointer, independently of which document is selected.
#[test]
fn hover_close_keeps_label_geometry_fixed_and_enrols_every_truthful_row() {
    let _g = crate::testlock::serial();
    crate::theme::set_active_by_name("Saltpan").expect("Saltpan is in the world roster");
    let fitted = fit_rows(&rows(0), 24);
    let layout = layout_of(&rows(0), true, true, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    let file_lines: Vec<_> = layout
        .lines()
        .iter()
        .enumerate()
        .filter_map(|(line, (_, kind))| match kind {
            gutter::GutterLine::File(row) => Some((line, *row)),
            _ => None,
        })
        .collect();
    assert_eq!(file_lines.len(), 3, "fixture enrols every file row");

    for (line, row) in file_lines {
        let band = plan.rows[line];
        let zone = close_zone(band);
        let switch = super::super::gutter_hit::stack_hit_from_plan(
            &layout,
            &plan,
            zone[0] - 1.0,
            band[1] + band[3] * 0.5,
        )
        .expect("row-hover point enrols");
        let close = super::super::gutter_hit::stack_hit_from_plan(
            &layout,
            &plan,
            zone[0] + 1.0,
            band[1] + band[3] * 0.5,
        )
        .expect("close-zone point enrols");
        assert_eq!(switch.row, row);
        assert!(!switch.is_close());
        assert_eq!(close.row, row);
        assert!(close.is_close());

        let resting = stack_spans(&fitted, None);
        let over_row = stack_spans(&fitted, Some(switch));
        let over_zone = stack_spans(&fitted, Some(close));
        let text = |spans: &[(String, glyphon::Color)]| {
            spans.iter().map(|(s, _)| s.as_str()).collect::<String>()
        };
        assert_eq!(
            text(&resting),
            text(&over_row),
            "row {row}: hover shifted the shaped label"
        );
        assert_eq!(
            text(&resting),
            text(&over_zone),
            "row {row}: zone shifted the shaped label"
        );
        assert_eq!(
            resting.iter().filter(|(s, _)| s.contains('×')).count(),
            fitted.len(),
            "every row reserves exactly one stable close run"
        );
        let marks = |spans: &[(String, glyphon::Color)]| {
            spans
                .iter()
                .filter(|(s, _)| s.contains('×'))
                .map(|(_, ink)| *ink)
                .collect::<Vec<_>>()
        };
        let rest_marks = marks(&resting);
        let row_marks = marks(&over_row);
        let zone_marks = marks(&over_zone);
        assert!(
            rest_marks.iter().all(|ink| ink.a() == 0),
            "a mark leaked into the resting frame"
        );
        for at in 0..fitted.len() {
            assert_eq!(
                row_marks[at].a() != 0,
                at == row,
                "row-hover enrollment disagrees at row {at}"
            );
            assert_eq!(
                zone_marks[at].a() != 0,
                at == row,
                "zone-hover enrollment disagrees at row {at}"
            );
        }
        assert_eq!(
            row_marks[row].0, zone_marks[row].0,
            "row {row}: one-stage reveal changed inside the zone"
        );
    }
}

/// A GROUP HEADING'S OWN CLOSE ZONE IS THE ONLY TARGET IT EVER OFFERS —
/// the switch half stays exactly as inert as it was before this row could
/// close anything (a press there is click-away, `App::gutter_stack_click`),
/// while a press on the reserved lane at its right edge enrols for `Close`
/// and names the SAME row the heading itself drew at. Mirrors the file-row
/// law above's own zone/switch split, but a heading has only one of the two
/// live — this is the law that would fail if the switch half were ever
/// wired up by accident, or if the close half silently stayed inert too.
#[test]
fn a_group_headings_switch_half_stays_inert_and_only_its_close_zone_enrols() {
    let files = vec![group_row("notes/", true), row("welcome.md", "", true)];
    let fitted = fit_rows(&files, 24);
    let layout = layout_of(&files, false, true, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    let (heading_line, heading_row) = layout
        .lines()
        .iter()
        .enumerate()
        .find_map(|(line, (_, kind))| match kind {
            gutter::GutterLine::File(row)
                if matches!(
                    layout.files[*row].kind,
                    crate::workingset::StackRowKind::Group { .. }
                ) =>
            {
                Some((line, *row))
            }
            _ => None,
        })
        .expect("the fixture draws exactly one heading");
    let band = plan.rows[heading_line];
    let zone = close_zone(band);
    let mid_y = band[1] + band[3] * 0.5;

    let switch =
        super::super::gutter_hit::stack_hit_from_plan(&layout, &plan, zone[0] - 1.0, mid_y);
    assert_eq!(
        switch, None,
        "a heading's switch half must stay inert — it is click-away, not a target"
    );

    let close = super::super::gutter_hit::stack_hit_from_plan(&layout, &plan, zone[0] + 1.0, mid_y)
        .expect("a heading's own close zone enrols");
    assert_eq!(
        close.row, heading_row,
        "the close hit must name the heading's own row"
    );
    assert!(close.is_close());
    assert!(matches!(
        close.kind,
        crate::workingset::StackRowKind::Group { .. }
    ));
    // The reservation this hit-test depends on: the heading's own mark is
    // shaped (even at zero alpha) so the zone geometry it is tested against
    // is the one the label was actually fitted around.
    assert!(
        fitted[heading_row].text.chars().count() + super::CLOSE_MARK_TEXT.chars().count() <= 24,
        "the heading's own fit did not reserve its close lane"
    );
}

/// THE FOLDER HEADING SITS ABOVE THE IDENTITY LINE IN BOTH SHAPES — one file
/// or a working set of many draw the SAME grammar, so opening a second file
/// inserts a row beneath what was already drawn rather than resorting the
/// block.
///
/// This replaces an earlier construction guarantee that pinned the one-file
/// block's bytes against a prior baseline: that guarantee was a side effect of
/// how the single-file path happened to be built, never a promise anything
/// read from it, and it held the two shapes to OPPOSITE orders — the heading
/// sat below the filename with one file, above it with two. The ordering this
/// law now pins is the real product contract (`GutterLine::lines`'s doc), and
/// `render::tests::gutter_stack_pixels`'s pixel-level law proves the N=1→N=2
/// transition is pure row insertion in the rendered block, not just in this
/// pure-data list.
#[test]
fn project_heads_only_the_multi_file_hierarchy() {
    let one = layout_of(&[], false, true, 24);
    assert_eq!(
        one.lines().iter().map(|(_, k)| *k).collect::<Vec<_>>(),
        vec![gutter::GutterLine::Project, gutter::GutterLine::Name],
        "the one-file block must head with the folder, exactly like the stack does"
    );

    let many = layout_of(&rows(0), false, true, 24);
    let above = many.lines();
    assert!(matches!(
        above.first().unwrap().1,
        gutter::GutterLine::Project
    ));
    assert_eq!(
        above
            .iter()
            .filter(|(_, k)| matches!(k, gutter::GutterLine::File(_)))
            .count(),
        3
    );

    // Swept over every shape the block draws (with/without the `changed
    // elsewhere` affordance, with/without a project to head), the heading is
    // either absent or first — it never sits after the identity line/rows.
    for changed in [false, true] {
        for project in [false, true] {
            for files in [Vec::new(), rows(0)] {
                let layout = layout_of(&files, changed, project, 24);
                let kinds: Vec<_> = layout.lines().iter().map(|(_, k)| *k).collect();
                if let Some(at) = kinds
                    .iter()
                    .position(|k| matches!(k, gutter::GutterLine::Project))
                {
                    let identity_at = kinds
                        .iter()
                        .position(|k| {
                            matches!(k, gutter::GutterLine::Name | gutter::GutterLine::File(_))
                        })
                        .unwrap_or(usize::MAX);
                    assert!(
                        at < identity_at,
                        "changed={changed} project={project} files_empty={}: project \
                         at {at} does not precede identity at {identity_at}: {kinds:?}",
                        files.is_empty()
                    );
                }
            }
        }
    }
}

/// **521: EXACTLY ONE VISIBLE OWNER OF THE PROJECT NAME.** The gutter's own
/// folder line is the project's one label whenever the stack draws no
/// heading of its own (a single-file identity, or a resting stack — which
/// never emits a `Group` row, [`crate::workingset::WorkingSet::stack_rows`]);
/// it vanishes the moment the stack DOES draw one, because that heading
/// already states which project this is (its own ink stays routed even
/// though it draws no plate) and a second label would repeat it. Swept over
/// both block shapes and both project-presence states, so the law cannot
/// pass by only ever exercising the case where the two rules happen to agree.
#[test]
fn the_folder_line_and_a_drawn_group_heading_never_both_own_the_project_name() {
    let heading_shapes: [(&str, Vec<StackRow>); 2] = [
        ("resting (no heading)", rows(0)),
        (
            "expanded (heading drawn)",
            vec![group_row("notes/", true), row("welcome.md", "", true)],
        ),
    ];
    for (shape, files) in &heading_shapes {
        for project in [true, false] {
            let layout = layout_of(files, false, project, 24);
            let lines = layout.lines();
            let project_lines = lines
                .iter()
                .filter(|(_, k)| matches!(k, gutter::GutterLine::Project))
                .count();
            let has_heading = shape.contains("heading drawn");
            let expected = usize::from(project && !has_heading);
            assert_eq!(
                project_lines, expected,
                "shape={shape} project={project}: expected {expected} folder line(s), \
                 found {project_lines} in {lines:?}"
            );
        }
    }
}

/// THE CLOSE ZONE IS ONLY AT THE RIGHT EDGE, AND THE REST OF THE ROW STILL
/// SWITCHES.
///
/// Swept over the margin widths a real window produces, because the two halves
/// fail at opposite ends: a zone measured as a FRACTION of the row would eat
/// half a narrow margin, and a zone pinned to an absolute px would vanish under
/// a wide one. Each width is probed on BOTH SIDES of its own boundary — the
/// boundary is derived from `close_zone`, so a law that moved with a broken
/// implementation is not what is being asserted; the invariants below are.
///
/// The invariants, in the order they can fail:
///   * the zone hugs the row's RIGHT edge exactly (that is the whole design —
///     it is the one x every right-aligned row shares);
///   * it never grows past the row (a full-row close target is a trap);
///   * it leaves the MAJORITY of the row switching, at every width above the
///     degenerate one — the asymmetry the design asks for;
///   * a point one pixel left of the boundary switches, and a point one pixel
///     right of it closes.
#[test]
fn only_the_rows_right_edge_closes_and_the_rest_of_it_switches() {
    let row_h = 12.0;
    // From a margin barely wider than the close square itself out to a wide one.
    for avail in [14.0_f32, 20.0, 48.0, 96.0, 120.0, 300.0] {
        for row_top in [0.0_f32, 37.5, 288.0] {
            let band = [0.0, row_top, avail, row_h];
            let zone = close_zone(band);
            let label = format!("avail={avail} top={row_top}");

            assert!(
                (zone[0] + zone[2] - (band[0] + band[2])).abs() < 0.001,
                "{label}: close zone {zone:?} does not end at the row's right edge {band:?}"
            );
            assert!(
                zone[2] <= band[2] + 0.001 && zone[2] > 0.0,
                "{label}: close zone width {} is not inside the row's {}",
                zone[2],
                band[2]
            );
            assert!(
                (zone[1] - band[1]).abs() < 0.001 && (zone[3] - band[3]).abs() < 0.001,
                "{label}: close zone {zone:?} does not share the row's own band vertically"
            );

            // PRESENCE, so this cannot pass by shrinking the zone to nothing:
            // the target is a full row square wherever the margin can hold one.
            if avail >= row_h {
                assert!(
                    (zone[2] - row_h).abs() < 0.001,
                    "{label}: close zone width {} is not the row square it claims to be",
                    zone[2]
                );
                assert!(
                    zone[0] > band[0],
                    "{label}: close zone {zone:?} swallowed the whole row"
                );
            }

            // Both sides of the boundary, one pixel apart.
            assert_eq!(
                row_intent(band, zone[0] - 1.0),
                RowIntent::Switch,
                "{label}: a pixel left of the close zone must still switch"
            );
            assert_eq!(
                row_intent(band, zone[0] + 1.0),
                RowIntent::Close,
                "{label}: a pixel inside the close zone must close"
            );
            // And the far left of the row — the empty margin a short name leaves
            // — is switching territory, not a dead patch.
            assert_eq!(
                row_intent(band, band[0] + 0.5),
                RowIntent::Switch,
                "{label}: the row's left end must switch"
            );
            assert_eq!(
                row_intent(band, band[0] + band[2] - 0.5),
                RowIntent::Close,
                "{label}: the row's extreme right must close"
            );
        }
    }
}

/// **THE DRAG INDICATOR IS CENTERED ON THE BOUNDARY it names**, swept over
/// every `file_row` in `0..=files.len()` and every block shape: `file_row: 0`
/// centers on the TOP edge of the first file row; every other `file_row`
/// centers on the BOTTOM edge of the row before it (so `file_row ==
/// files.len()` sits below the LAST row) — a straddling hairline
/// legitimately dips half its own thickness into the row(s) either side of
/// the boundary, so the invariant is the CENTER, not "never touches a row".
#[test]
fn drag_indicator_centers_on_the_boundary_above_the_named_file_row() {
    for changed in [false, true] {
        for project in [false, true] {
            let files = rows(1);
            let layout = layout_of(&files, changed, project, 24);
            let lines = layout.lines();
            let plan = crate::render::plan::plan_gutter_stack(
                300.0,
                layout.avail,
                12.0,
                lines.len(),
                8.0,
                0.5,
            );
            let shape = format!("changed={changed} project={project}");
            let offset = lines.len() - layout.files.len();
            for file_row in 0..=layout.files.len() {
                let rect = drag_indicator_rect(&layout, &plan, file_row, 2.0)
                    .unwrap_or_else(|| panic!("{shape} file_row={file_row}: must resolve"));
                let [x, y, w, h] = rect;
                assert_eq!(h, 2.0, "{shape} file_row={file_row}: fixed thickness");
                let center = y + h * 0.5;
                let (band, want_top) = if file_row == 0 {
                    (plan.rows[offset], true)
                } else {
                    (plan.rows[offset + file_row - 1], false)
                };
                let boundary = if want_top { band[1] } else { band[1] + band[3] };
                assert!(
                    (center - boundary).abs() < 0.01,
                    "{shape} file_row={file_row}: indicator {rect:?} centers on {center}, \
                     not the boundary {boundary} of band {band:?}"
                );
                // Hugs the SAME right edge every row's own band does.
                assert!(
                    (x + w - (band[0] + band[2])).abs() < 0.01,
                    "{shape} file_row={file_row}: indicator {rect:?} does not span row's \
                     own width {band:?}"
                );
            }
        }
    }
}

/// A BLOCK WITH NO WORKING SET (the single-file margin) never draws an
/// indicator — there is no slot list to straddle a boundary in.
#[test]
fn drag_indicator_is_absent_off_a_working_set() {
    let layout = layout_of(&[], false, true, 24);
    let plan = crate::render::plan::plan_gutter_stack(
        300.0,
        layout.avail,
        12.0,
        layout.lines().len(),
        8.0,
        0.5,
    );
    assert_eq!(drag_indicator_rect(&layout, &plan, 0, 2.0), None);
}
