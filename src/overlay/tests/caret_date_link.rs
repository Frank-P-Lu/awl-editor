use super::*;

#[test]
fn caret_picker_lists_three_styles_navigates_and_maps_modes() {
    use crate::caret::CaretMode;
    // `new_caret` reads `crate::caret::is_auto()` at construction (for
    // `original_caret_was_auto`), so hold the caret global's lock and pin an
    // explicit override for the whole test — otherwise this races whatever
    // override another parallel test leaves behind.
    let _g = crate::testlock::serial();
    // SUMMON with Block active: the corpus is the three look labels in ALL order,
    // each row's "binding" column carrying its description.
    crate::caret::set_mode(CaretMode::Block);
    let ov = OverlayState::new_caret(CaretMode::Block);
    assert_eq!(ov.kind.as_str(), "caret");
    assert_eq!(ov.item_strings(), vec!["Block", "Morph", "I-beam"]);
    assert_eq!(
        ov.item_bindings(),
        vec![
            "rounded square + trailing underline",
            "takes the glyph silhouette",
            "an alive insertion bar",
        ]
    );
    // Opens highlighting the ACTIVE look, and `original_caret` remembers it.
    assert_eq!(ov.selected_value(), Some("Block"));
    assert_eq!(ov.selected_caret_mode(), Some(CaretMode::Block));
    // An explicit override was active at open — not auto.
    assert_eq!(
        ov.audition,
        crate::overlay::Audition::Caret {
            original: CaretMode::Block,
            was_auto: false
        }
    );
    // NAVIGATE down the list -> the selected look maps back via from_label.
    let mut ov = ov;
    ov.move_sel(1);
    assert_eq!(ov.selected_caret_mode(), Some(CaretMode::Morph));
    ov.move_sel(1);
    assert_eq!(ov.selected_caret_mode(), Some(CaretMode::Ibeam));
    // Opening with a non-Block look pre-selects THAT row.
    crate::caret::set_mode(CaretMode::Ibeam);
    let ov2 = OverlayState::new_caret(CaretMode::Ibeam);
    assert_eq!(ov2.selected_value(), Some("I-beam"));
    assert_eq!(
        ov2.audition,
        crate::overlay::Audition::Caret {
            original: CaretMode::Ibeam,
            was_auto: false
        }
    );
    // The hint leads with the universal jump cluster (move + type-to-filter) then
    // names ↵'s action; flat picker (no descend).
    assert_eq!(OverlayKind::Caret.hint(), "type to filter   \u{21B5} apply");
    // selected_caret_mode is None for a non-caret picker.
    let theme = OverlayState::new_theme(vec!["Tawny".into()], 0);
    assert_eq!(theme.selected_caret_mode(), None);

    // Restore.
    crate::caret::clear_override();
}

/// The DATE-FORMAT picker. `new_date` lists all five formats EACH
/// rendered with the given `today` as its PRIMARY text (what-you-see-is-what-
/// inserts), with the format NAME in the secondary column, pre-selects the
/// active format, and maps the selected CORPUS INDEX back to the format (the
/// accept path's resolution). Uses the fixed capture placeholder date so the
/// example strings are deterministic.
#[test]
fn date_picker_lists_five_examples_with_names_and_maps_by_index() {
    use crate::dateformat::DateFormat;
    let today = crate::dateformat::CAPTURE_PLACEHOLDER_YMD; // (2009, 3, 7)

    // SUMMON with DdMmYy active: the corpus is the five EXAMPLE DATES in ALL
    // order, each row's secondary column carrying the format's human name.
    let ov = OverlayState::new_date(DateFormat::DdMmYy, today);
    assert_eq!(ov.kind.as_str(), "date");
    assert_eq!(OverlayKind::Date.title(), "date format");
    assert_eq!(
        ov.item_strings(),
        vec![
            "07/03/09",
            "03/07/09",
            "2009-03-07",
            "2009/03/07",
            "7 March 2009"
        ],
        "each row is TODAY rendered in that format — what you see is what inserts"
    );
    assert_eq!(
        ov.item_bindings(),
        vec![
            "Day / Month / Year",
            "Month / Day / Year",
            "ISO 8601",
            "Year / Month / Day",
            "Day Month Year",
        ]
    );
    // Opens highlighting the ACTIVE format's row (its example date).
    assert_eq!(ov.selected_value(), Some("07/03/09"));
    // The selected CORPUS INDEX maps back to the format (the accept path).
    assert_eq!(
        ov.selected_corpus_index()
            .and_then(|i| DateFormat::ALL.get(i).copied()),
        Some(DateFormat::DdMmYy)
    );
    // NAVIGATE down: the row's example + its mapped format both advance.
    let mut ov = ov;
    ov.move_sel(2);
    assert_eq!(ov.selected_value(), Some("2009-03-07"));
    assert_eq!(
        ov.selected_corpus_index()
            .and_then(|i| DateFormat::ALL.get(i).copied()),
        Some(DateFormat::Iso)
    );
    // Opening with a different active format pre-selects THAT row.
    let ov2 = OverlayState::new_date(DateFormat::DMonthYyyy, today);
    assert_eq!(ov2.selected_value(), Some("7 March 2009"));
    // The hint mirrors Dictionary/Caret (no live preview to teach): move + filter, ↵ apply.
    assert_eq!(OverlayKind::Date.hint(), "type to filter   \u{21B5} apply");
}

/// The Date picker's row content must NOT get the muted-directory/
/// content-filename figure/ground split (`OverlayKind::row_path_splits`):
/// three of the five example dates (`DD/MM/YY`, `MM/DD/YY`, `YYYY/MM/DD`) use
/// `/` as a DATE separator, and `row_split` would otherwise mistake it for a
/// path boundary and mute part of the date's own glyphs. Only `InsertLink`'s
/// row content is a genuine URL/path — exhaustive over every kind so a future
/// variant must consciously decide (no-wildcard match in the fn under test).
#[test]
fn only_insert_link_rows_get_the_path_figure_ground_split() {
    for kind in OverlayKind::ALL {
        let expect = matches!(kind, OverlayKind::InsertLink);
        assert_eq!(
            kind.row_path_splits(),
            expect,
            "{kind:?}.row_path_splits() should be {expect}"
        );
    }
    // The concrete regression: every example date this picker shows contains a
    // literal `/` in three of its five formats, yet must render splitless.
    assert!(!OverlayKind::Date.row_path_splits());
    let today = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
    let ov = OverlayState::new_date(crate::dateformat::DateFormat::DdMmYy, today);
    assert!(
        ov.item_strings().iter().any(|s| s.contains('/')),
        "at least one example date must contain '/' for this law to bite"
    );
}

/// `original_caret_was_auto`: the field the Caret-style picker's auto-aware
/// Cancel relies on (see `actions::overlay_nav`'s Cancel arm). It reads the
/// LIVE `crate::caret::is_auto()` global at construction, independent of
/// whatever concrete `active` mode is passed in (the two real call sites keep
/// the two in step by always passing `crate::caret::mode()`).
#[test]
fn caret_picker_captures_whether_it_opened_while_auto() {
    use crate::caret::CaretMode;
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();

    // AUTO: no override set — a mono world resolves Block.
    crate::caret::clear_override();
    crate::theme::set_active_by_name("Tawny").unwrap();
    assert_eq!(crate::caret::mode(), CaretMode::Block);
    let ov = OverlayState::new_caret(crate::caret::mode());
    assert_eq!(
        ov.audition,
        crate::overlay::Audition::Caret {
            original: CaretMode::Block,
            was_auto: true
        },
        "records the RESOLVED look, flagged as auto's resolution not a pin"
    );

    // EXPLICIT: an actual pin, even one that resolves to the exact same
    // concrete mode, is NOT auto.
    crate::caret::set_mode(CaretMode::Block);
    let ov2 = OverlayState::new_caret(crate::caret::mode());
    assert_eq!(
        ov2.audition,
        crate::overlay::Audition::Caret {
            original: CaretMode::Block,
            was_auto: false
        },
        "an explicit pin is never reported as auto"
    );

    // Restore.
    crate::caret::clear_override();
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}
