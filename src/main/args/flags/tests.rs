//! THE ROSTER'S LAWS. The point of each one is the axis the generator collapses:
//! a table that prints `--help` and a reference section states its wrong answer
//! with a law behind it, so every claim the roster makes is asked on BOTH sides
//! of the condition that produced it.

use super::*;

/// The runtime half of the roster/enum bijection. The compile-time half is
/// `super::super::parse_args`'s no-wildcard match on [`FlagId`] — a variant with
/// no arm does not build. This is the other direction: a variant with no ROW.
#[test]
fn every_id_appears_exactly_once_in_the_roster() {
    for id in FlagId::ALL {
        let rows = FLAGS.iter().filter(|f| f.id == *id).count();
        assert_eq!(
            rows, 1,
            "`FlagId::{id:?}` has {rows} rows in the flag roster; it must have exactly one"
        );
        // Reached deliberately: `name_of` panics by name for a missing row, and
        // this is where that panic must fire rather than in a user's shell.
        assert!(name_of(*id).starts_with('-'));
    }
    assert_eq!(
        FLAGS.len(),
        FlagId::ALL.len(),
        "the roster and the id list came from one macro invocation and cannot differ in length"
    );
}

/// Every spelling is a real flag spelling, and no two flags answer to the same
/// token — the failure that would make `lookup` silently prefer whichever row
/// comes first.
#[test]
fn every_spelling_is_distinct_and_dash_prefixed() {
    let mut seen: Vec<&str> = Vec::new();
    for f in FLAGS {
        assert!(
            f.name().starts_with("--"),
            "`{}`'s canonical spelling must be a long flag",
            f.name()
        );
        for n in f.names {
            assert!(n.starts_with('-'), "`{n}` is not a flag spelling");
            assert!(
                !seen.contains(n),
                "two roster rows answer to `{n}` — `lookup` would silently pick one"
            );
            seen.push(n);
        }
    }
}

/// THE RECOGNITION LAW, both sides. Every roster spelling resolves to its own
/// row; a token the roster does not carry resolves to nothing. The near-misses
/// matter more than the hits: a `starts_with`-shaped lookup would accept
/// `--screenshot-nonsense` as `--screenshot`, and a `contains`-shaped one would
/// accept `x--debug`.
#[test]
fn lookup_accepts_every_spelling_and_only_those() {
    for f in FLAGS {
        for n in f.names {
            assert_eq!(
                lookup(n).map(|g| g.id),
                Some(f.id),
                "`{n}` does not resolve to its own roster row"
            );
        }
    }
    for miss in [
        "--screenshot-nonsense",
        "x--debug",
        "-debug",
        "--Debug",
        "--",
        "",
        "file.md",
        "-x",
    ] {
        assert!(
            lookup(miss).is_none(),
            "`{miss}` is not a flag but `lookup` claimed one"
        );
    }
}

/// `-h` is the roster's only alias, and it reaches exactly the same row as
/// `--help`. The negative half is the axis a single assertion collapses: no
/// OTHER flag has quietly acquired an alias, so a document printing one
/// canonical spelling per row is telling the whole truth.
#[test]
fn help_is_the_only_flag_with_an_alias() {
    assert_eq!(lookup("-h").map(|f| f.id), Some(FlagId::Help));
    assert_eq!(lookup("--help").map(|f| f.id), Some(FlagId::Help));
    let aliased: Vec<&str> = FLAGS
        .iter()
        .filter(|f| f.names.len() > 1)
        .map(|f| f.name())
        .collect();
    assert_eq!(
        aliased,
        vec!["--help"],
        "a second alias appeared; the reference prints one spelling per row and must \
         start printing both"
    );
}

/// THE ARITY LAW, both sides of `Operand::optional`. A required operand's
/// absence refuses the flag with the roster's own `need` text; an optional one's
/// absence is fine. The pair MUST differ per flag — that is what makes the
/// declared operand list load-bearing rather than decorative, and it is the same
/// check that pins the refusal messages the parser has always printed.
#[test]
fn a_missing_operand_is_refused_exactly_when_the_roster_says_required() {
    let mut required = 0usize;
    let mut optional = 0usize;
    for f in FLAGS {
        let empty: Vec<String> = Vec::new();
        let got = f.take_operands(&mut empty.into_iter());
        match f.operands.first() {
            Some(op) if !op.optional => {
                required += 1;
                let Err(e) = got else {
                    panic!(
                        "`{}` declares a required operand but an empty stream satisfied it",
                        f.name()
                    )
                };
                let msg = e.to_string();
                assert_eq!(
                    msg,
                    format!("{} {}", f.name(), op.need),
                    "`{}`'s refusal must read as the roster spells it",
                    f.name()
                );
            }
            Some(_) => {
                optional += 1;
                let ops = got.expect("an optional operand may be absent");
                assert!(
                    ops.opt(0).is_none(),
                    "`{}` reported an operand nothing supplied",
                    f.name()
                );
            }
            None => {
                got.expect("a flag with no operands cannot be short of one");
            }
        }
    }
    assert!(
        required > 0 && optional > 0,
        "both sides of `optional` must be exercised (required={required}, optional={optional}) \
         — a roster with only one kind makes this law vacuous"
    );
}

/// The operands a flag declares are the operands it CONSUMES, in order — the
/// property that lets a usage line be generated instead of authored. The
/// zero-operand side of the pair is the one that matters: it must leave the
/// following argument on the stream, which is how a file argument survives.
#[test]
fn a_flag_consumes_exactly_the_operands_it_declares() {
    for f in FLAGS {
        let supplied: Vec<String> = (0..f.operands.len())
            .map(|i| format!("op{i}"))
            .chain(["trailing.md".to_string()])
            .collect();
        let mut stream = supplied.into_iter();
        let ops = f
            .take_operands(&mut stream)
            .expect("every declared operand was supplied");
        for i in 0..f.operands.len() {
            assert_eq!(ops.req(i), format!("op{i}"), "`{}` operand {i}", f.name());
        }
        assert_eq!(
            stream.next().as_deref(),
            Some("trailing.md"),
            "`{}` consumed one argument too many — a file argument after it would vanish",
            f.name()
        );
    }
}

/// `--help` prints the `Shown` rows and NOT the `Hidden` ones. Both sides, by
/// spelling: a listing bit that did nothing would pass a one-sided check.
#[test]
fn help_prints_exactly_the_shown_flags() {
    let _g = crate::testlock::serial();
    let text = help_text();
    let mut shown = 0usize;
    let mut hidden = 0usize;
    for f in FLAGS {
        // Match the rendered PREFIX, not the bare name: `--menu-open` appears
        // inside `--menu-bar`'s sentence and `--search` inside `--search-case`,
        // so a substring test on the name alone answers the wrong question.
        let want = match f.block {
            HelpBlock::Modes => mode_prefix(f),
            HelpBlock::Options => option_prefix(f),
        };
        let printed = text.lines().any(|l| l.starts_with(&want));
        match f.listing {
            Listing::Shown => {
                shown += 1;
                assert!(
                    printed,
                    "`{}` is Shown but --help carries no `{want}` line",
                    f.name()
                );
            }
            Listing::Hidden => {
                hidden += 1;
                assert!(
                    !printed,
                    "`{}` is Hidden but --help gives it a `{want}` line",
                    f.name()
                );
            }
        }
    }
    assert!(
        shown > 0 && hidden > 0,
        "both listings must be exercised (shown={shown}, hidden={hidden})"
    );
}

/// The two blocks render differently, and the difference is the roster's, not
/// the renderer's guess: a mode is a whole `awl … [file]` invocation, an option
/// is an indented line under the heading. Asked on both sides so a block field
/// that had stopped mattering would fail here.
#[test]
fn a_mode_renders_as_an_invocation_and_an_option_as_an_indented_line() {
    let _g = crate::testlock::serial();
    let text = help_text();
    assert!(
        text.starts_with(USAGE_LINE),
        "the first line names awl itself"
    );
    assert!(
        text.contains(OPTIONS_HEADING),
        "the options block keeps its heading"
    );
    let heading_at = text
        .find(OPTIONS_HEADING)
        .expect("the options heading is present");
    for f in FLAGS.iter().filter(|f| f.listing == Listing::Shown) {
        let want = match f.block {
            HelpBlock::Modes => format!("awl {}", f.name()),
            HelpBlock::Options => format!("  {}", f.name()),
        };
        let at = text
            .find(&want)
            .unwrap_or_else(|| panic!("--help has no `{want}` line for `{}`", f.name()));
        match f.block {
            HelpBlock::Modes => assert!(
                at < heading_at,
                "`{}` is a capture mode but its line sits under the options heading",
                f.name()
            ),
            HelpBlock::Options => assert!(
                at > heading_at,
                "`{}` is an option but its line sits above the options heading",
                f.name()
            ),
        }
    }
}

/// Every description is a real sentence fragment, and no row leaves a
/// substitution token unexpanded — the failure a typo'd `{world}` would cause,
/// which would print a literal brace to a user.
#[test]
fn every_summary_is_meaningful_and_fully_substituted() {
    let _g = crate::testlock::serial();
    for f in FLAGS {
        let s = f.summary_text();
        assert!(
            !s.trim().is_empty(),
            "`{}` carries no description",
            f.name()
        );
        assert_eq!(s.trim(), s, "`{}`'s description is not trimmed", f.name());
        assert!(
            !s.contains('{') && !s.contains('}'),
            "`{}`'s description still carries an unexpanded token: {s}",
            f.name()
        );
        assert_ne!(
            s.trim_start_matches("--"),
            f.name().trim_start_matches("--"),
            "`{}`'s description only restates its own name",
            f.name()
        );
    }
}

/// THE SUBSTITUTION LAW, both sides. `--theme`'s description names the live
/// world roster; every other row passes through untouched. Without the negative
/// half a `replace` that fired on the wrong rows would still look right.
#[test]
fn the_worlds_token_expands_only_where_the_roster_writes_it() {
    let _g = crate::testlock::serial();
    let theme = lookup("--theme").expect("--theme is a flag");
    assert!(
        theme.summary.contains(WORLDS_TOKEN),
        "--theme's raw description is expected to carry the token"
    );
    let expanded = theme.summary_text();
    assert!(
        !expanded.contains(WORLDS_TOKEN),
        "the token survived expansion: {expanded}"
    );
    for name in crate::theme::world_names() {
        assert!(
            expanded.contains(&name),
            "the expanded --theme description omits the world `{name}`"
        );
    }
    for f in FLAGS.iter().filter(|f| f.id != FlagId::Theme) {
        assert_eq!(
            f.summary_text(),
            f.summary,
            "`{}` has no token, so its description must pass through unchanged",
            f.name()
        );
    }
}

/// Optional operands print with brackets, required ones without. The pair is the
/// whole point: a usage column that bracketed everything (or nothing) would read
/// plausibly and be wrong about every row.
#[test]
fn operand_usage_brackets_exactly_the_optional_operands() {
    assert_eq!(
        lookup("--pack-icns")
            .expect("--pack-icns is a flag")
            .operand_usage(),
        "[DIR]"
    );
    assert_eq!(
        lookup("--export-linux-icon")
            .expect("--export-linux-icon is a flag")
            .operand_usage(),
        "OUT.png"
    );
    assert_eq!(
        lookup("--capture-held")
            .expect("--capture-held is a flag")
            .operand_usage(),
        "DIR \"0,30,60,90\" OUT.png"
    );
    assert_eq!(
        lookup("--debug")
            .expect("--debug is a flag")
            .operand_usage(),
        "",
        "a flag with no operands prints no operands"
    );
    for f in FLAGS {
        let usage = f.operand_usage();
        for op in f.operands {
            let want = if op.optional {
                format!("[{}]", op.meta)
            } else {
                op.meta.to_string()
            };
            assert!(
                usage.contains(&want),
                "`{}`'s usage `{usage}` does not spell operand `{}` as `{want}`",
                f.name(),
                op.meta
            );
        }
    }
}
