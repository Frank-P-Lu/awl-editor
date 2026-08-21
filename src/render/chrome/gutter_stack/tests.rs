use super::*;
use crate::workingset::StackRow;

fn row(leaf: &str, parent: &str, active: bool) -> StackRow {
    StackRow {
        leaf: leaf.to_string(),
        parent: parent.to_string(),
        active,
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
    let mid = fit_rows(std::slice::from_ref(&deep), 26);
    assert_eq!(&mid[0].text[..mid[0].parent_byte], "research/…/");
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
        let fitted = fit_rows(&rows(active), 24);
        let spans = stack_spans(&fitted, None, ClosePrototype::Off);
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
    for close in [
        ClosePrototype::Off,
        ClosePrototype::OneStageAll,
        ClosePrototype::OneStageSiblings,
        ClosePrototype::TwoStageAll,
        ClosePrototype::TwoStageSiblings,
    ] {
        assert_eq!(
            stack_spans(&layout.files, None, close).len(),
            0,
            "{close:?}: an empty stack must not grow a reserved close lane"
        );
    }
    // And the block is exactly the two lines it has always been.
    assert_eq!(
        layout
            .lines()
            .iter()
            .map(|(_, kind)| *kind)
            .collect::<Vec<_>>(),
        vec![gutter::GutterLine::Name, gutter::GutterLine::Project]
    );
}

/// Every prototype is a color-only reveal over one already-shaped trailing
/// run, enrolled from the same row/zone geometry the click consumes.
#[test]
fn hover_close_prototypes_keep_label_geometry_fixed_and_enrol_the_truthful_row() {
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

        for prototype in [
            ClosePrototype::OneStageAll,
            ClosePrototype::OneStageSiblings,
            ClosePrototype::TwoStageAll,
            ClosePrototype::TwoStageSiblings,
        ] {
            let resting = stack_spans(&fitted, None, prototype);
            let over_row = stack_spans(&fitted, Some(switch), prototype);
            let over_zone = stack_spans(&fitted, Some(close), prototype);
            let text = |spans: &[(String, glyphon::Color)]| {
                spans.iter().map(|(s, _)| s.as_str()).collect::<String>()
            };
            assert_eq!(
                text(&resting),
                text(&over_row),
                "{prototype:?} row {row}: row-hover shifted the shaped label"
            );
            assert_eq!(
                text(&resting),
                text(&over_zone),
                "{prototype:?} row {row}: zone-hover shifted the shaped label"
            );
            assert_eq!(
                resting.iter().filter(|(s, _)| s.contains('×')).count(),
                fitted.len(),
                "{prototype:?}: every row reserves exactly one stable close run"
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
                "{prototype:?}: a mark leaked into the resting frame"
            );
            for at in 0..fitted.len() {
                let enrolled = at == row && !(row == 0 && !prototype.includes_active());
                assert_eq!(
                    row_marks[at].a() != 0,
                    enrolled,
                    "{prototype:?} row-hover enrollment disagrees at row {at}"
                );
                assert_eq!(
                    zone_marks[at].a() != 0,
                    enrolled,
                    "{prototype:?} zone-hover enrollment disagrees at row {at}"
                );
            }
            if row != 0 || prototype.includes_active() {
                if prototype.two_stage() {
                    assert_ne!(
                        row_marks[row].0, zone_marks[row].0,
                        "{prototype:?} row {row}: faint and full stages collapsed in Saltpan"
                    );
                } else {
                    assert_eq!(
                        row_marks[row].0, zone_marks[row].0,
                        "{prototype:?} row {row}: one-stage reveal changed inside the zone"
                    );
                }
            }
        }
    }
}

#[test]
fn folder_prototypes_change_only_the_multi_file_hierarchy() {
    let one = layout_of(&[], false, true, 24);
    let one_legacy = one.lines_with(FolderPrototype::Legacy);
    for prototype in [FolderPrototype::QuietBelow, FolderPrototype::HeadingAbove] {
        assert_eq!(
            one.lines_with(prototype),
            one_legacy,
            "{prototype:?}: a one-file block must keep its exact line order"
        );
    }

    let many = layout_of(&rows(0), false, true, 24);
    let below = many.lines_with(FolderPrototype::QuietBelow);
    let above = many.lines_with(FolderPrototype::HeadingAbove);
    assert!(matches!(
        below.last().unwrap().1,
        gutter::GutterLine::Project
    ));
    assert!(matches!(
        above.first().unwrap().1,
        gutter::GutterLine::Project
    ));
    assert_eq!(
        below
            .iter()
            .filter(|(_, k)| matches!(k, gutter::GutterLine::File(_)))
            .count(),
        3
    );
    assert_eq!(
        above
            .iter()
            .filter(|(_, k)| matches!(k, gutter::GutterLine::File(_)))
            .count(),
        3
    );
}

#[test]
fn prototype_environment_vocabulary_is_closed() {
    assert_eq!(
        parse_close_prototype(Some("one-all")),
        ClosePrototype::OneStageAll
    );
    assert_eq!(
        parse_close_prototype(Some("one-siblings")),
        ClosePrototype::OneStageSiblings
    );
    assert_eq!(
        parse_close_prototype(Some("two-all")),
        ClosePrototype::TwoStageAll
    );
    assert_eq!(
        parse_close_prototype(Some("two-siblings")),
        ClosePrototype::TwoStageSiblings
    );
    assert_eq!(parse_close_prototype(Some("typo")), ClosePrototype::Off);
    assert_eq!(
        parse_folder_prototype(Some("quiet-below")),
        FolderPrototype::QuietBelow
    );
    assert_eq!(
        parse_folder_prototype(Some("heading-above")),
        FolderPrototype::HeadingAbove
    );
    assert_eq!(
        parse_folder_prototype(Some("typo")),
        FolderPrototype::Legacy
    );
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
