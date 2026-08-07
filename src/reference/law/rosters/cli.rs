//! THE COMMAND-LINE SECTION'S LAWS.
//!
//! Generating a table from a roster does not make it true — it moves the error
//! from transcription to SOURCING, and a generated table states its wrong answer
//! with a law behind it. The byte-diff in `super::super` only proves the section
//! equals itself regenerated; it cannot see whether a description's claim about
//! awl is correct. These laws read the claims back OUT of the roster and check
//! each against the code that owns the fact, asking every claim on both sides of
//! the condition that produced it.
//!
//! This file wrote itself out of a real defect: the hand-written `--help` string
//! stated `--measure`'s default as 80, and `page::DEFAULT_MEASURE` is 70 (with
//! 100 for code). One generation of that string into a public reference would
//! have shipped the wrong number with a drift law standing behind it.

use crate::args::flags::{FLAGS, FlagId, HelpBlock, Listing, lookup, name_of};
use crate::reference::Section;

/// Every flag lands in the section, in the sub-table its roster row selects.
/// BOTH SIDES of both fields: a `Shown` option must appear under Options and NOT
/// under the unlisted table, and a `Hidden` one the reverse — so a `listing` or
/// `block` field that had stopped steering anything fails here rather than
/// quietly grouping every flag together.
#[test]
fn every_flag_lands_in_the_sub_table_its_roster_row_selects() {
    let _g = crate::testlock::serial();
    let md = Section::Cli.markdown();
    let (modes, rest) = split_at_caption(&md, "### Options");
    let (options, unlisted) = split_at_caption(rest, "### Unlisted flags");

    let mut counts = [0usize; 3];
    for f in FLAGS {
        let row = format!("| `{}`", f.names.join(", "));
        let want = match (f.block, f.listing) {
            (HelpBlock::Modes, Listing::Shown) => {
                counts[0] += 1;
                (modes, "the capture-modes table")
            }
            (HelpBlock::Options, Listing::Shown) => {
                counts[1] += 1;
                (options, "the options table")
            }
            (_, Listing::Hidden) => {
                counts[2] += 1;
                (unlisted, "the unlisted table")
            }
        };
        assert!(
            want.0.contains(&row),
            "`{}` is missing from {} of REFERENCE.md's command-line section",
            f.name(),
            want.1
        );
        // The negative half: the row appears in exactly ONE sub-table.
        let elsewhere = [modes, options, unlisted]
            .iter()
            .filter(|part| part.contains(&row))
            .count();
        assert_eq!(
            elsewhere, 1,
            "`{}` appears in {elsewhere} sub-tables; each flag belongs to exactly one",
            f.name()
        );
    }
    assert!(
        counts.iter().all(|n| *n > 0),
        "every sub-table must be exercised, or this law is vacuous for one of them: \
         modes={}, options={}, unlisted={}",
        counts[0],
        counts[1],
        counts[2]
    );
}

/// Split a generated markdown section at a caption, returning the text before
/// and after it. Panics by name when the caption is gone — which is what a
/// renamed sub-table does.
fn split_at_caption<'a>(md: &'a str, caption: &str) -> (&'a str, &'a str) {
    let at = md.split_once(caption).unwrap_or_else(|| {
        panic!(
            "the generated command-line section carries no `{caption}` heading — a \
             sub-table was renamed and this law's enrolment no longer matches"
        )
    });
    (at.0, at.1)
}

/// THE SOURCING LAW. Each of these descriptions states a NUMBER or a PATH as
/// awl's own default. The number is read back out of the roster's own text and
/// compared against the owner that actually supplies it, so a stale default
/// fails here instead of being published.
///
/// The pairing is deliberate: the needle proves the CLAIM is still being made
/// (a description reworded past its number fails the `contains` half), and the
/// owner proves the claim is TRUE. Either half alone can pass while the document
/// lies.
#[test]
fn every_default_a_flag_states_matches_the_owner_of_that_default() {
    let _g = crate::testlock::serial();
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let claims: Vec<(FlagId, String)> = vec![
        (
            FlagId::CaptureSize,
            format!(
                "default {}x{}",
                crate::capture::CANVAS_WIDTH,
                crate::capture::CANVAS_HEIGHT
            ),
        ),
        // The dpi a capture runs at with the flag absent — `opts.dpi` is
        // `Option<f32>` and every consumer resolves `None` through the same
        // `unwrap_or(1.0)`, which is the claim.
        (FlagId::CaptureDpi, "default 1.0".to_string()),
        (
            FlagId::Measure,
            format!(
                "default {} for prose, {} for code",
                crate::page::DEFAULT_MEASURE,
                crate::page::DEFAULT_MEASURE_CODE
            ),
        ),
        // One decimal deliberately: `f32`'s own `Display` renders 3.0 as `3`,
        // and a needle built that way would demand a number the description does
        // not print — the readout-formatter trap, one layer up from the band.
        (
            FlagId::Zoom,
            format!(
                "({:.1}..{:.1})",
                crate::range::ZOOM.min,
                crate::range::ZOOM.max
            ),
        ),
        (
            FlagId::SoakGpuSeconds,
            format!(
                "default {}",
                crate::soak_gpu::DEFAULT_DURATION.as_secs()
            ),
        ),
    ];
    for (id, needle) in claims {
        let f = FLAGS
            .iter()
            .find(|f| f.id == id)
            .unwrap_or_else(|| panic!("`FlagId::{id:?}` left the roster"));
        assert!(
            f.summary_text().contains(&needle),
            "`{}`'s description no longer states `{needle}`, which its owner in the tree \
             still says — either the owner changed and the description is now false, or the \
             description was reworded and this law's needle must follow it: {}",
            f.name(),
            f.summary_text()
        );
    }
    // The two PATH claims, from the resolvers that produce them rather than from
    // a retyped path. Both are `$HOME`-derived, so the law states its own
    // precondition instead of asserting against an absent variable.
    if let Some(home) = home {
        let folder = crate::args::resolve_default_folder(&None);
        assert_eq!(
            folder,
            home.join("notes"),
            "`resolve_default_folder` no longer answers ~/notes"
        );
        assert!(
            lookup("--default-folder")
                .expect("--default-folder is a flag")
                .summary_text()
                .contains("default ~/notes"),
            "--default-folder's description stopped naming the default the resolver returns"
        );
        // `config_path` reads `$AWL_CONFIG` and `$XDG_CONFIG_HOME` first, so the
        // documented path is only the HOME branch's answer — assert against that
        // branch's construction rather than calling the resolver under whatever
        // environment the test host happens to carry.
        let documented = home.join(".config").join("awl").join("config.toml");
        assert!(
            lookup("--config")
                .expect("--config is a flag")
                .summary_text()
                .contains("default ~/.config/awl/config.toml"),
            "--config's description stopped naming its documented default"
        );
        assert!(
            documented.ends_with("awl/config.toml"),
            "the documented config path shape changed"
        );
    }
}

/// `--measure`'s description claims it IMPLIES page mode. Asked on both sides of
/// that implication: page mode OFF, then the flag's own effect, then page mode
/// ON. A one-sided check would pass against a build where page mode was already
/// on for an unrelated reason.
#[test]
fn measure_turns_page_mode_on_as_its_description_claims() {
    let _g = crate::testlock::serial();
    assert!(
        lookup("--measure")
            .expect("--measure is a flag")
            .summary_text()
            .contains("implies --page on"),
        "--measure's description stopped claiming it implies page mode"
    );
    // Snapshot and restore by hand: `testlock::serial` reports a leaked page
    // global as a failure, and a law that mutates one owes the restore.
    let (was_on, was_measure) = (crate::page::page_on(), crate::page::measure());
    crate::page::set_page_on(false);
    assert!(!crate::page::page_on(), "the precondition is page mode OFF");
    // Exactly what the `FlagId::Measure` arm performs.
    crate::page::set_measure(55);
    crate::page::set_page_on(true);
    let (now_on, now_measure) = (crate::page::page_on(), crate::page::measure());
    crate::page::set_measure(was_measure);
    crate::page::set_page_on(was_on);
    assert!(
        now_on,
        "the --measure arm must leave page mode on, as the description promises"
    );
    assert_eq!(now_measure, 55);
}

/// `--menu-bar`'s description claims the default differs by platform. That is a
/// claim about TWO values, so both are read — from `menubar`'s own authored
/// consts, never from `cfg!`, since one checked-in document is generated on
/// macOS and verified on Linux.
#[test]
fn the_menu_bar_default_really_differs_by_platform_as_its_description_claims() {
    let _g = crate::testlock::serial();
    let text = lookup("--menu-bar")
        .expect("--menu-bar is a flag")
        .summary_text();
    assert!(
        text.contains("default on web/Linux, off on macOS"),
        "--menu-bar's description stopped stating the per-platform default: {text}"
    );
    assert!(
        crate::menubar::MENU_BAR_DEFAULT_OTHER,
        "the description says the bar is ON off-macOS"
    );
    assert!(
        !crate::menubar::MENU_BAR_DEFAULT_MACOS,
        "the description says the bar is OFF on macOS"
    );
    assert_ne!(
        crate::menubar::MENU_BAR_DEFAULT_MACOS,
        crate::menubar::MENU_BAR_DEFAULT_OTHER,
        "the description's whole content is that these two DIFFER"
    );
}

/// `--caret-mode`'s description claims the `auto` default is font-derived:
/// mono to block, proportional to morph. Asked on both sides of the font
/// condition — the pair MUST differ, which is the only thing that makes the
/// sentence mean anything.
#[test]
fn the_caret_mode_auto_default_really_follows_the_face_as_its_description_claims() {
    let _g = crate::testlock::serial();
    let text = lookup("--caret-mode")
        .expect("--caret-mode is a flag")
        .summary_text();
    assert!(
        text.contains("mono->block, proportional->morph"),
        "--caret-mode's description stopped stating the font-derived default: {text}"
    );
    // Enrolment from the roster, not from a named world: whichever worlds
    // actually carry a mono and a proportional display face.
    let mono = crate::theme::THEMES
        .iter()
        .find(|t| crate::caret::font_is_mono(t.font))
        .expect("some world wears a mono display face");
    let prop = crate::theme::THEMES
        .iter()
        .find(|t| !crate::caret::font_is_mono(t.font))
        .expect("some world wears a proportional display face");
    // The override is never touched: `default_mode` is the font-derived answer
    // the `auto` arm falls back to, which is what the description claims.
    let was = crate::theme::active().name;
    crate::theme::set_active_by_name(mono.name).expect("a roster world");
    let on_mono = crate::caret::default_mode();
    crate::theme::set_active_by_name(prop.name).expect("a roster world");
    let on_prop = crate::caret::default_mode();
    crate::theme::set_active_by_name(was).expect("the world this law entered under");
    assert_eq!(
        on_mono,
        crate::caret::CaretMode::Block,
        "`{}` wears the mono face `{}` and must default to block",
        mono.name,
        mono.font
    );
    assert_eq!(
        on_prop,
        crate::caret::CaretMode::Morph,
        "`{}` wears the proportional face `{}` and must default to morph",
        prop.name,
        prop.font
    );
    assert_ne!(on_mono, on_prop, "the claim is that the two DIFFER");
}

/// `--list-worlds` claims it prints the roster `--theme` accepts. Both sides:
/// every name it would print is a name `--theme` resolves, and a name it would
/// not print is one `--theme` refuses.
#[test]
fn list_worlds_prints_exactly_the_roster_theme_accepts_as_its_description_claims() {
    let _g = crate::testlock::serial();
    assert!(
        lookup("--list-worlds")
            .expect("--list-worlds is a flag")
            .summary_text()
            .contains("the roster `--theme` accepts"),
        "--list-worlds' description stopped claiming it prints --theme's roster"
    );
    let names = crate::theme::world_names();
    assert_eq!(
        names.len(),
        crate::theme::THEMES.len(),
        "`world_names` must be the whole roster"
    );
    let was = crate::theme::active().name;
    let refused: Vec<&str> = names
        .iter()
        .filter(|n| crate::theme::set_active_by_name(n).is_none())
        .copied()
        .collect();
    let bogus = crate::theme::set_active_by_name("Notaworld");
    crate::theme::set_active_by_name(was).expect("the world this law entered under");
    assert!(
        refused.is_empty(),
        "--list-worlds would print {refused:?}, which --theme refuses"
    );
    assert!(
        bogus.is_none(),
        "--theme accepts a name --list-worlds would never print"
    );
}

/// `--semantic-json`'s description says it prints JSON INSTEAD OF a PNG, and
/// `name_of` is how the parser asks about that flag without a literal of its
/// own. If the spelling ever changes, the hermetic-run predicate in `parse_args`
/// and this description must move together — this is the law that notices.
#[test]
fn the_semantic_json_spelling_the_parser_asks_for_is_the_roster_s_own() {
    let _g = crate::testlock::serial();
    assert_eq!(name_of(FlagId::SemanticJson), "--semantic-json");
    assert_eq!(
        lookup(name_of(FlagId::SemanticJson)).map(|f| f.id),
        Some(FlagId::SemanticJson),
        "the name the parser pushes onto `capture_modes` must resolve back to the same row"
    );
}
