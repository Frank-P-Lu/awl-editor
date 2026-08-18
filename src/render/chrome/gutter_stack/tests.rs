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
        let spans = stack_spans(&fitted);
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
    assert_eq!(stack_spans(&layout.files).len(), 0);
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
