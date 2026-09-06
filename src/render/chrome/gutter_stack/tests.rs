use super::*;
use crate::workingset::StackRow;

/// WCAG relative luminance / contrast ratio — a SCRATCH measurement for the
/// plated-ink presence floor below, spelled here rather than reached for in
/// `theme::derive` (where it is private, and deliberately so: the shipped
/// rule is [`theme::selected_row_secondary_ink`] itself, and a law that
/// borrowed the very helper that function makes its decision with could only
/// ever restate it).
fn rel_luminance(c: theme::Srgb) -> f32 {
    let lin = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

fn contrast_ratio(a: theme::Srgb, b: theme::Srgb) -> f32 {
    let (ya, yb) = (rel_luminance(a), rel_luminance(b));
    let (hi, lo) = if ya > yb { (ya, yb) } else { (yb, ya) };
    (hi + 0.05) / (lo + 0.05)
}

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

/// **LAW 1 (data-level pin): `fit_rows` RECLAIMS THE FULL BUDGET FOR THE
/// LABEL.** Before option A the trailing close lane was docked out of every
/// row's budget (`budget - CLOSE_MARK_TEXT.chars().count()`), so even a name
/// long enough to want the whole line was capped three characters short of
/// it — the bug this item exists to close ("every name sits ~3 chars short
/// of the stack's right edge"). The mark is now a LEADING span shaped on top
/// of whatever `fit_rows` returns (`stack_spans`), so a sufficiently long
/// candidate must be able to spend the ENTIRE budget on the label alone.
/// Swept across the whole budget range and the `StackRowKind` roster, so no
/// kind can silently keep the old docked ceiling.
#[test]
fn fit_rows_reclaims_the_full_budget_for_the_label() {
    let long_leaf = "a-name-long-enough-to-fill-every-budget-this-law-tries.md";
    let kinds = [
        crate::workingset::StackRowKind::File,
        crate::workingset::StackRowKind::More { hidden: 3 },
        crate::workingset::StackRowKind::Group { active: false },
        crate::workingset::StackRowKind::Overflow {
            up: true,
            hidden: 3,
        },
    ];
    for budget in crate::render::rowlayout::GUTTER_MIN_NAME_CHARS..40 {
        for &kind in &kinds {
            let row = crate::workingset::StackRow {
                leaf: long_leaf.to_string(),
                kind,
                ..crate::workingset::StackRow::default()
            };
            let fitted = fit_rows(std::slice::from_ref(&row), budget);
            assert_eq!(
                fitted[0].text.chars().count(),
                budget,
                "budget={budget} kind={kind:?}: the label stopped {} chars short of the \
                 full budget — a close lane is still being docked from it",
                budget - fitted[0].text.chars().count()
            );
        }
    }
}

/// **EVERY ROW KIND SHAPES THE SAME LEADING MARK, UNIFORMLY** — not sampled
/// per kind, but proved as one shared shape: whatever `stack_spans` draws for
/// a row, the very first characters of that row's own shaped line (after its
/// row-separating newline, on every row but the first) are the close mark's
/// text, for every member of `StackRowKind`. A `More`/`Overflow` row can
/// never reveal it (`stack_hit_from_plan`'s own enrolment keeps `hover` from
/// ever naming one), but it still shapes the identical leading run every
/// other kind does — a uniform ragged-edge growth, never one that only grows
/// where the mark happens to be revealable.
#[test]
fn every_row_kind_shapes_the_same_leading_mark_uniformly() {
    let kinds = [
        crate::workingset::StackRowKind::File,
        crate::workingset::StackRowKind::More { hidden: 3 },
        crate::workingset::StackRowKind::Group { active: false },
        crate::workingset::StackRowKind::Overflow {
            up: true,
            hidden: 3,
        },
    ];
    for (at, &kind) in kinds.iter().enumerate() {
        let mut rows = vec![row("opening.md", "", true)];
        rows.push(crate::workingset::StackRow {
            leaf: "second.md".to_string(),
            kind,
            ..crate::workingset::StackRow::default()
        });
        let fitted = fit_rows(&rows, 24);
        let spans = stack_spans(&fitted, None);
        // Row 0's own first span carries no leading "\n" (`stack_spans`'
        // own doc); row `at`'s (index 1 here) does.
        let first_span = &spans[0].0;
        assert!(
            first_span.starts_with(super::CLOSE_MARK_TEXT),
            "row 0: shaped line {first_span:?} does not lead with the close mark"
        );
        let second_span = spans
            .iter()
            .find(|(text, _)| text.starts_with('\n'))
            .unwrap_or_else(|| panic!("kind={kind:?}: no second row was shaped at all"));
        assert_eq!(
            &second_span.0[1..1 + super::CLOSE_MARK_TEXT.len()],
            super::CLOSE_MARK_TEXT,
            "kind={kind:?} (roster index {at}): the row's own shaped line does not lead \
             with the close mark right after its row-separating newline"
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

/// **A BLOCK WITH NO WORKING SET PLATES ITS IDENTITY LINE — the active file
/// is ALWAYS plated, and the geometry is the SAME `plate_rects` answer a
/// multi-file active row gets.**
///
/// SUPERSEDES `a_single_file_block_plates_nothing`, which asserted the
/// opposite by name and was correct against a recorded decision the user has
/// since overridden ("sure plate it. this means more consistent right?"): a
/// lone open file read as "not selected" because the one mark that says
/// "this is the file you are editing" was the one thing the block withheld
/// exactly when there was nothing else to compare against.
///
/// It is not enough that SOME rect appears — the old law's real content was
/// that the block's shape and the plate's shape agree, so this asserts the
/// identity line's plate is byte-for-byte the rect the SAME line would get
/// as a stack row: same band, same ink measurement, same pad. Anything else
/// would be a lookalike rect drawn beside the mechanism rather than by it.
#[test]
fn a_single_file_block_plates_its_own_identity_line() {
    // Swept over the block shapes the affordance and the project line make,
    // because the identity line's own row INDEX moves with them — the same
    // axis `a_plate_marks_the_active_row_in_every_block_shape` sweeps, and
    // the reason a plate is derived from `lines()` rather than from a count.
    for changed in [false, true] {
        for project in [false, true] {
            let layout = layout_of(&[], changed, project, 24);
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
            let plates = plate_rects(&layout, &plan, 6.0, 2.0);
            assert_eq!(
                plates.len(),
                1,
                "{shape}: the lone open file is the active file and must be plated"
            );
            let at = lines
                .iter()
                .position(|(_, kind)| matches!(kind, gutter::GutterLine::Name))
                .unwrap_or_else(|| panic!("{shape}: no identity line in the block"));
            let band = plan.rows[at];
            let plate = plates[0];
            assert!(
                plate[1] >= band[1] && plate[1] + plate[3] <= band[1] + band[3],
                "{shape}: plate {plate:?} escapes the identity line's own band {band:?}"
            );
            // THE SAME MECHANISM, NOT A RESEMBLANCE: the rect a stack row of
            // the identical text would be handed, computed by the same
            // production function through the File arm, must equal this one
            // exactly. A second code path that merely looked right would
            // drift here on the first change to `plate_rect`'s padding.
            let as_row = layout_of(&[row(&layout.name, "", true)], changed, project, 24);
            let row_lines = as_row.lines();
            let row_plan = crate::render::plan::plan_gutter_stack(
                300.0,
                as_row.avail,
                12.0,
                row_lines.len(),
                8.0,
                0.5,
            );
            let row_plates = plate_rects(&as_row, &row_plan, 6.0, 2.0);
            assert_eq!(
                row_plates.len(),
                1,
                "{shape}: the one-row stack fixture must plate exactly once"
            );
            assert_eq!(
                plate, row_plates[0],
                "{shape}: the identity line's plate {plate:?} is not the rect the same \
                 name gets as a stack row {:?} — two plate mechanisms, not one",
                row_plates[0]
            );
            // The block is still exactly the lines it has always been, in the
            // same top-to-bottom order the stack shape uses: this item
            // changed what the identity line WEARS, never where it sits.
            let mut expect = Vec::new();
            if changed {
                expect.push(gutter::GutterLine::Changed);
            }
            if project {
                expect.push(gutter::GutterLine::Project);
            }
            expect.push(gutter::GutterLine::Name);
            assert_eq!(
                lines.iter().map(|(_, kind)| *kind).collect::<Vec<_>>(),
                expect,
                "{shape}: the block's own line order moved"
            );
            assert_eq!(
                stack_spans(&layout.files, None).len(),
                0,
                "{shape}: an empty stack must not grow a reserved close lane"
            );
        }
    }
}

/// **NO LINE BUT AN ACTIVE FILE EVER PLATES — enrolled over the whole
/// [`gutter::GutterLine`] roster, so the identity line's new fill cannot
/// spread to the block's other lines.**
///
/// The companion to the law above: plating the identity line means the
/// `Name` arm of `plate_rects`' own match now returns a rect, and the arms
/// either side of it (`Project`, `Changed`) must still return nothing. The
/// match carries no wildcard, so a future line kind fails to compile rather
/// than inheriting an answer; this asserts the answer the existing kinds
/// get, at a block shape that draws every one of them at once.
#[test]
fn only_the_active_file_line_plates_across_the_whole_line_roster() {
    let files = vec![row("opening.md", "", false), row("ledger.md", "", true)];
    let layout = layout_of(&files, true, true, 24);
    let lines = layout.lines();
    let plan =
        crate::render::plan::plan_gutter_stack(300.0, layout.avail, 12.0, lines.len(), 8.0, 0.5);
    // The fixture really does draw every non-File line kind, or the sweep
    // below proves nothing about them.
    for kind in [gutter::GutterLine::Changed, gutter::GutterLine::Project] {
        assert!(
            lines.iter().any(|(_, k)| *k == kind),
            "the fixture never drew {kind:?} — this law would sweep past it"
        );
    }
    let plates = plate_rects(&layout, &plan, 6.0, 2.0);
    assert_eq!(plates.len(), 1, "exactly one plate: {plates:?}");
    let plated_band = plates[0];
    for (at, (_, kind)) in lines.iter().enumerate() {
        let band = plan.rows[at];
        let covered =
            plated_band[1] < band[1] + band[3] && plated_band[1] + plated_band[3] > band[1];
        let should = matches!(kind, gutter::GutterLine::File(i) if layout.files[*i].active);
        assert_eq!(
            covered,
            should,
            "line {at} ({kind:?}) is {}plated and should {}be",
            if covered { "" } else { "not " },
            if should { "" } else { "not " }
        );
    }
}

/// **THE LONE IDENTITY LINE'S NAME WEARS THE PLATED-ROW INK, NOT THE
/// MARGIN'S PLAIN `muted` — one owner, swept over the whole world roster.**
///
/// The ink `muted` was chosen for this line *because* it had no plate to
/// differentiate against; with a fill behind it that reasoning no longer
/// holds on its own, and the value must come from the same
/// [`theme::selected_row_secondary_ink`] routing a plated stack row's does.
/// In many worlds that routing returns `muted` unchanged (the ink survives
/// its own justification) — which is exactly why this is asserted as
/// ROUTING rather than as a value: a world where `muted` collapses into
/// `surface_selected` must get the pole instead, and a hardcoded `muted`
/// would be green everywhere else and invisible exactly there.
///
/// Enrolment is the roster itself (`theme::THEMES`), and the sweep asserts
/// the identity line's ink EQUALS an active stack row's in every world —
/// the two shapes cannot drift.
///
/// ⚠️ THIS LAW OWNS THE OWNER, NOT THE WIRING. It proves
/// [`active_row_ink`] answers the same thing for both shapes; it cannot see
/// whether `prepare_gutter` actually SPENDS that answer on the identity
/// line, because that seam needs a device. The pixel law
/// (`render/tests/gutter_stack_pixels.rs`'s
/// `the_active_file_is_plated_alone_and_among_several_on_every_world`) is
/// what goes red when the identity line's span is handed plain `muted`
/// again — verified by mutation, not assumed.
#[test]
fn the_identity_lines_ink_is_the_same_routed_ink_an_active_stack_row_wears() {
    let _g = crate::testlock::serial();
    let _pin = theme::WorldPin::snapshot();
    let mut judged = Vec::new();
    let mut routed_away = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let name = world.name;
        let identity = active_row_ink();
        let stack = stack_spans(&fit_rows(&[row("opening.md", "", true)], 24), None);
        let row_ink = stack
            .iter()
            .find(|(text, _)| text.contains("opening.md"))
            .expect("the active row draws its own name span")
            .1;
        assert_eq!(
            identity.0, row_ink.0,
            "{name}: the identity line's ink and an active stack row's disagree — \
             two ink owners, not one"
        );
        // PRESENCE, not just agreement: an ink equal to its own plate would
        // satisfy the equality above while rendering nothing. The routed ink
        // must genuinely separate from the fill it sits on — measured as a
        // WCAG ratio here, and again on real pixels by
        // `render/tests/gutter_stack_pixels.rs`'s lone-file law.
        let fill = theme::surface_selected();
        let ink = theme::selected_row_secondary_ink(fill);
        let cr = contrast_ratio(fill, ink);
        assert!(
            cr >= 1.6,
            "{name}: the plated ink {ink:?} reads at {cr:.2} against its own plate \
             {fill:?} — the identity line's name would vanish into the fill this item \
             just put behind it"
        );
        if ink != theme::muted() {
            routed_away.push(name);
        }
        judged.push(name);
    }
    assert_eq!(
        judged.len(),
        theme::THEMES.len(),
        "only {} of {} worlds were judged: {judged:?}",
        judged.len(),
        theme::THEMES.len()
    );
    // NON-VACUITY OF THE ROUTING ITSELF: at least one world must actually
    // take the fallback, or this law would pass identically against a
    // hardcoded `muted` and prove nothing about the mechanism it names.
    assert!(
        !routed_away.is_empty(),
        "no world in the roster routes the plated ink away from `muted` — the routing \
         is untested and a hardcoded `muted` would pass this law"
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
/// already-shaped LEADING run, enrolled from the same row/zone geometry the
/// click consumes. The active document participates: "hovered" names the row
/// under the pointer, independently of which document is selected.
///
/// This is also LAW 2 (reveal changes ink only) at the pure-data seam: the
/// whole shaped label text is byte-identical across resting/row-hover/
/// zone-hover, so a hover can only ever be a per-row color flip on the mark's
/// own run, never a reflow of anything else on the line.
#[test]
fn hover_close_keeps_label_geometry_fixed_and_enrols_every_truthful_row() {
    let _g = crate::testlock::serial();
    crate::theme::set_active_by_name("Saltpan").expect("Saltpan is in the world roster");
    // Small enough that even the fixture's longest row (the nested
    // "journal/field-notes.md" parent+leaf, plus the mark's own three
    // characters) keeps its ink leading edge inside the 120px band —
    // `avail`/`budget` here were picked independently of any real char
    // width, so this only has to stay consistent with itself.
    let label_char_w = 4.0;
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
        let text_w = (fitted[row].text.chars().count() + super::CLOSE_MARK_TEXT.chars().count())
            as f32
            * label_char_w;
        let mark_w = super::CLOSE_MARK_TEXT.chars().count() as f32 * label_char_w;
        let zone = close_zone(band, text_w, mark_w);
        let y = band[1] + band[3] * 0.5;
        let hit_at = |px: f32| {
            super::super::gutter_hit::stack_hit_from_plan(&layout, &plan, label_char_w, px, y)
        };
        let switch = hit_at(zone[0] - 1.0).expect("row-hover point enrols");
        let close = hit_at(zone[0] + 1.0).expect("close-zone point enrols");
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
/// while a press on its own leading-ink lane enrols for `Close` and names the
/// SAME row the heading itself drew at. Mirrors the file-row law above's own
/// zone/switch split, but a heading has only one of the two live — this is
/// the law that would fail if the switch half were ever wired up by
/// accident, or if the close half silently stayed inert too.
#[test]
fn a_group_headings_switch_half_stays_inert_and_only_its_close_zone_enrols() {
    let label_char_w = 6.0;
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
    let text_w = (fitted[heading_row].text.chars().count() + super::CLOSE_MARK_TEXT.chars().count())
        as f32
        * label_char_w;
    let mark_w = super::CLOSE_MARK_TEXT.chars().count() as f32 * label_char_w;
    let zone = close_zone(band, text_w, mark_w);
    let mid_y = band[1] + band[3] * 0.5;

    let switch = super::super::gutter_hit::stack_hit_from_plan(
        &layout,
        &plan,
        label_char_w,
        zone[0] - 1.0,
        mid_y,
    );
    assert_eq!(
        switch, None,
        "a heading's switch half must stay inert — it is click-away, not a target"
    );

    let close = super::super::gutter_hit::stack_hit_from_plan(
        &layout,
        &plan,
        label_char_w,
        zone[0] + 1.0,
        mid_y,
    )
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
/// **LAW 3 (hit-zone/ink agreement), at the pure-geometry seam.** Swept over
/// the margin widths a real window produces, a RANGE of ink widths from an
/// empty name up to one that fills the whole row (the axis the trailing
/// design never had to sweep, since its own edge never moved), AND a RANGE
/// of `mark_w` values standing in for the roster of real per-face mark
/// widths (`CLOSE_ZONE_ROWS`'s own doc: a narrow-aspect face's 2-char lane
/// can sit under a full row-height square, a wide-aspect face's over it): a
/// right-aligned row's ink leading edge moves with the name's own length, so
/// `close_zone` has to track it rather than a fixed x, and a law that only
/// ever probed one width (of either axis) could pass while the anchor or
/// the cap silently drifted at every OTHER value. Each width is probed on
/// BOTH SIDES of its own boundary — the boundary is derived from
/// `close_zone`, so a law that moved with a broken implementation is not
/// what is being asserted; the invariants below are.
///
/// The invariants, in the order they can fail:
///   * the zone hugs the row's own ink LEADING edge exactly — `avail -
///     text_w`, never a fixed x (that is the whole design: it is the mark's
///     own position, and the mark's position moves with the name);
///   * it never escapes the row (a target outside the band is unreachable);
///   * wherever the ink's leading edge leaves room for both a full
///     row-height square AND the mark's own full lane before the row's own
///     right edge, the zone is `min(row_h, mark_w)` — the SMALLER of the
///     two, never shrunk further just because a name happens to be short,
///     and never wider than the mark's own drawn lane either (559's own
///     reason: a wider zone would highlight text that is not the ×);
///   * a point one pixel left of the boundary switches, and a point one
///     pixel right of it closes;
///   * the row's own extreme right — where the NAME's ink actually sits —
///     switches, the mirror image of the trailing design's own right-edge
///     close (this is the assertion that would have caught landing the
///     mirror backwards: it is INVERTED from what the trailing law asserted
///     at the same point).
#[test]
fn close_zone_hugs_the_rows_own_leading_ink_edge_and_the_rest_switches() {
    let row_h = 12.0;
    // Narrower than the row square, exactly matched, and wider than it — the
    // three shapes the measured mono roster's real pitch produces relative
    // to `CLOSE_ZONE_ROWS`.
    for mark_w in [row_h * 0.5, row_h, row_h * 2.0] {
        // From a margin barely wider than the close square itself out to a wide one.
        for avail in [14.0_f32, 20.0, 48.0, 96.0, 120.0, 300.0] {
            for row_top in [0.0_f32, 37.5, 288.0] {
                let band = [0.0, row_top, avail, row_h];
                // A RANGE of name widths: empty, shorter than the zone itself,
                // exactly one zone, half the row, and maximal (fills the row
                // entirely — the case a fixed-x zone could never have modeled).
                // Never narrower than `mark_w` — a real caller's `text_w` always
                // includes the mark's own chars, so it can never be smaller
                // than the mark's own lane.
                for text_w in [mark_w, row_h.max(mark_w), avail * 0.5, avail] {
                    if text_w > avail {
                        continue;
                    }
                    let zone = close_zone(band, text_w, mark_w);
                    let ink_left = (avail - text_w).max(0.0);
                    let label =
                        format!("avail={avail} top={row_top} text_w={text_w} mark_w={mark_w}");

                    assert!(
                        (zone[0] - ink_left).abs() < 0.001,
                        "{label}: close zone {zone:?} does not hug the row's own ink leading \
                         edge {ink_left}"
                    );
                    assert!(
                        zone[0] + zone[2] <= band[0] + band[2] + 0.001,
                        "{label}: close zone {zone:?} escapes the row's own band {band:?}"
                    );
                    assert!(
                        zone[2] >= 0.0,
                        "{label}: close zone width {} went negative",
                        zone[2]
                    );
                    assert!(
                        (zone[1] - band[1]).abs() < 0.001 && (zone[3] - band[3]).abs() < 0.001,
                        "{label}: close zone {zone:?} does not share the row's own band \
                         vertically"
                    );
                    // NEVER WIDER THAN THE MARK'S OWN LANE — 559's own
                    // containment concern: a zone (and the hover plate drawn
                    // from it) reaching past the mark's own drawn characters
                    // would highlight/accept clicks over the label's ink.
                    assert!(
                        zone[2] <= mark_w + 0.001,
                        "{label}: close zone width {} reaches past the mark's own lane {mark_w}",
                        zone[2]
                    );

                    // PRESENCE, so this cannot pass by shrinking the zone to
                    // nothing: the target is the smaller of a full row square
                    // and the mark's own lane, wherever there is room for
                    // both between the ink's leading edge and the row's own
                    // right edge.
                    let want = row_h.min(mark_w);
                    if avail - ink_left >= want {
                        assert!(
                            (zone[2] - want).abs() < 0.001,
                            "{label}: close zone width {} is not min(row_h, mark_w)={want}",
                            zone[2]
                        );
                    }

                    if zone[2] > 0.0 {
                        // Both sides of the boundary, one pixel apart.
                        assert_eq!(
                            row_intent(band, text_w, mark_w, zone[0] - 1.0),
                            RowIntent::Switch,
                            "{label}: a pixel left of the close zone must still switch"
                        );
                        assert_eq!(
                            row_intent(band, text_w, mark_w, zone[0] + 1.0),
                            RowIntent::Close,
                            "{label}: a pixel inside the close zone must close"
                        );
                    }
                    // The row's own extreme right is where the NAME's ink sits —
                    // switching territory now, the mirror of the trailing
                    // design's own right-edge close.
                    if zone[0] + zone[2] < band[0] + band[2] - 0.5 {
                        assert_eq!(
                            row_intent(band, text_w, mark_w, band[0] + band[2] - 0.5),
                            RowIntent::Switch,
                            "{label}: the row's extreme right (the name's own ink) must switch"
                        );
                    }
                    // A MAXIMAL name (fills the whole row) pushes the mark's
                    // own leading edge all the way to the row's own leading
                    // edge — the far left of the band now closes, the mirror
                    // image of the trailing design's far-right close, as long
                    // as the mark's own lane reaches that far too.
                    if text_w >= avail - 0.001 && avail >= mark_w {
                        assert_eq!(
                            row_intent(band, text_w, mark_w, band[0] + 0.5),
                            RowIntent::Close,
                            "{label}: a maximal-width name's own mark sits at the row's \
                             leading edge — the far left must close"
                        );
                    }
                }
            }
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

/// **550: THE REVEALED × KEEPS A REAL BREATH BEFORE THE LABEL, AND ITS OWN
/// CLICK ZONE NEVER REACHES PAST THE MARK'S OWN LANE — MEASURED FROM EVERY
/// BUNDLED MONO FACE'S OWN FILE, NEVER A HARDCODED ASPECT TABLE.**
///
/// The roster is DERIVED, not named: every face
/// [`crate::render::bundled_display_faces`] ships that
/// [`crate::render::facepitch::mono_pitch_em`] measures as genuinely
/// fixed-pitch (`facepitch.rs`'s own "ask the font, not the docs" mechanism —
/// the same one the caret's mono/proportional split rides) enrols, so a newly
/// bundled mono face is swept the moment it ships, with nothing to remember.
/// `char_w` per face is that measurement times the LABEL-scale font size —
/// the SAME quantity `panel_attrs`'s shaping and `prepare_gutter`'s
/// `label_char_w` both mean to approximate, but measured off the real glyph
/// table instead of the one nominal `Metrics::char_width` constant every
/// world shares regardless of its own display face. (A genuinely mono face's
/// OWN advance is the same for every glyph by construction — the definition
/// [`facepitch::measure_pitch`] checks — so this is also `×`'s own advance,
/// not merely a representative probe glyph's.)
///
/// TWO invariants, swept over that roster and (for the second) over the
/// gutter's own ink-width range from the empty-name floor up past a row's own
/// height — the axis CLOSE_ZONE_ROWS's own doc names ("a narrow-aspect mono
/// is where a 2-char lane would first lose the square"):
///
/// * **PRESENCE FLOOR — the reveal keeps a real breath, not just a nonzero
///   one.** `CLOSE_MARK_TEXT`'s own doc promises "`×` followed by a single
///   breath before the label"; asserted here as at least half a character's
///   width of gap between the × glyph's own ink and the mark lane's trailing
///   edge, on every face. **This is the law CLAUDE.md's mutation-proof
///   convention wants**: shrinking [`CLOSE_MARK_TEXT`] to `"×"` alone (no
///   trailing space) leaves ZERO breath on every face — the exact regression
///   the two-space predecessor over-corrected and this item's one-space fix
///   targets — and this floor catches it on the first face it checks,
///   independent of window width or any particular name.
/// * **ZONE ⊆ LANE — the close zone this face's row draws never reaches
///   past the mark's own two-character lane.** [`close_zone`]'s own upper
///   bound is `min(row_h, mark_w, available_ink)`; asserted directly here
///   with each face's REAL `mark_w`, which is where the roster sweep earns
///   its keep — Iosevka's measured pitch (the narrowest bundled mono face)
///   is the one where `2 × char_w` first sits BELOW `row_h`, so its own zone
///   is capped by the lane rather than the row square, a case a single
///   wide-aspect face's screenhot could never surface.
#[test]
fn close_mark_keeps_a_real_breath_and_its_zone_never_escapes_the_lane_across_every_mono_face() {
    let m = crate::render::Metrics::new(1.0);
    let label = crate::markdown::type_scale::LABEL;
    let font_size_px = m.font_size * label;
    let row_h = m.line_height * label;
    let mark_chars = super::CLOSE_MARK_TEXT.chars().count();

    let mut found = Vec::new();
    for (bytes, _declared) in crate::render::bundled_display_faces() {
        let Some(pitch_em) = crate::render::facepitch::mono_pitch_em(bytes) else {
            continue; // not a genuinely fixed-pitch bundled face — nothing to sweep
        };
        let family = crate::render::facepitch::registered_family(bytes)
            .unwrap_or_else(|| "<unregistered>".to_string());
        found.push(family.clone());

        let char_w = pitch_em * font_size_px;
        let mark_w = mark_chars as f32 * char_w;
        let glyph_w = char_w; // × occupies exactly one cell on a true mono face

        let breath = mark_w - glyph_w;
        assert!(
            breath >= 0.5 * char_w - 0.001,
            "{family}: the revealed × keeps only {breath}px of breath before the label \
             (needs >= {}px, half a char at this face's own {char_w}px pitch) — \
             CLOSE_MARK_TEXT's own reserve no longer covers a real one-space breath",
            0.5 * char_w
        );

        for chars in [0usize, rowlayout::GUTTER_MIN_NAME_CHARS, 12, 40] {
            let text_w = (chars + mark_chars) as f32 * char_w;
            let band = [0.0, 0.0, text_w.max(row_h) + 40.0, row_h];
            let zone = close_zone(band, text_w, mark_w);
            assert!(
                zone[2] <= mark_w + 0.001,
                "{family}: chars={chars}: close zone width {} reaches past the mark's own \
                 {mark_chars}-char lane ({mark_w}px at this face's {char_w}px pitch) — a hover \
                 plate drawn from this rect would highlight the label's own first glyph",
                zone[2]
            );
        }
    }
    assert!(
        found.len() >= 3,
        "the bundled-display-face roster derivation found only {} genuinely mono face(s) \
         ({found:?}) — too few to call this a roster sweep",
        found.len()
    );
}
