//! THE FLAG ROSTER — the one owner of "what command-line flags awl has".
//!
//! Every flag is one [`Flag`] row in [`roster`]: its spelling (plus any alias),
//! the operands it pulls off the argument stream, which `--help` block it
//! belongs to, whether `--help` prints it at all, and one sentence saying what
//! it does. Three consumers read that one table and nothing else:
//!
//! 1. [`super::parse_args`] — [`lookup`] is the ONLY place a `--…` token becomes
//!    a flag, and [`Flag::take_operands`] is the only place a flag's operands are
//!    pulled off the stream. The dispatch that follows is a no-wildcard match on
//!    [`FlagId`], so a new roster row fails to COMPILE until the parser reads it.
//! 2. [`help_text`] — `--help` is generated, not typed. Its wording, its order,
//!    and which flags it lists are all properties of the roster.
//! 3. `crate::reference::rows::cli` (test-only) — the reference's command-line
//!    section, held to this table by the reference drift laws. A new flag fails
//!    a named law until `scripts/regen-reference.sh` has run, so a flag cannot
//!    land undocumented even when `--help` deliberately stays quiet about it.
//!
//! WHY THE OPERANDS ARE DATA RATHER THAN `args.next()` PER ARM: the count and
//! the refusal text were once repeated in every arm, which made the usage line a
//! reader sees an independently-authored SECOND claim about the same fact. Here
//! the declared operand list is what the loop actually consumes, so a wrong
//! arity breaks parsing rather than only misprinting a document — the failure
//! mode a generated table needs, since a generated table states its wrong answer
//! with a law behind it.
//!
//! THE BROWSER BUILD HAS NO COMMAND LINE. `fn main` is the native entry;
//! `wasm_start` never calls [`super::parse_args`], so this roster is complete on
//! every target and only the arm BODIES that touch native-only modules carry a
//! `cfg`. Nothing here is per-OS.

use anyhow::Result;

/// One operand a flag pulls off the argument stream.
pub(crate) struct Operand {
    /// How a usage line spells it — a metavariable (`OUT.png`, `WxH`) or, where
    /// the shape needs showing, an example (`"0,16,50,150"`).
    pub(crate) meta: &'static str,
    /// The tail of the refusal when it is missing, so the message reads
    /// `<flag> <need>`. Empty for an optional operand, which cannot be missing.
    pub(crate) need: &'static str,
    /// An operand the flag reads IF the next argument LOOKS LIKE it. Never
    /// missing in the refusal sense — an absent or declined one falls back to
    /// the arm's own default. See [`Operand::wants`] for what "looks like"
    /// means; `numeric` narrows it further.
    pub(crate) optional: bool,
    /// An optional operand whose value must PARSE as a plain non-negative
    /// integer to be consumed. Declared for an operand like `--menu-open`'s
    /// index, where any non-numeric token is unambiguously not meant for this
    /// flag — unlike an optional PATH (`--pack-icns DIR`), where every string
    /// is a legitimate value and there is no content-based test that could
    /// tell a directory from a file argument, so that case stays ambiguous by
    /// construction and only declines a leading `-`.
    pub(crate) numeric: bool,
}

impl Operand {
    /// A required operand: absent, the flag is refused with `<flag> <need>`.
    pub(crate) const fn req(meta: &'static str, need: &'static str) -> Operand {
        Operand {
            meta,
            need,
            optional: false,
            numeric: false,
        }
    }

    /// An optional operand: absent, the arm supplies its own default. Declines
    /// only a token that starts with `-` (plainly a flag, never this
    /// operand's value) — see [`Operand::opt_numeric`] for a narrower operand
    /// that also declines a non-numeric token.
    pub(crate) const fn opt(meta: &'static str) -> Operand {
        Operand {
            meta,
            need: "",
            optional: true,
            numeric: false,
        }
    }

    /// An optional operand whose value must look like a plain integer to be
    /// consumed — so a following non-flag, non-numeric token (a file argument,
    /// most often) is left on the stream for the flag loop to read instead.
    pub(crate) const fn opt_numeric(meta: &'static str) -> Operand {
        Operand {
            meta,
            need: "",
            optional: true,
            numeric: true,
        }
    }

    /// Whether an optional operand should consume `next`, peeked off the
    /// stream without being removed from it yet. A token that starts with `-`
    /// is plainly a flag, never an operand value, so it is never consumed;
    /// a `numeric` operand additionally declines anything that does not parse
    /// as a plain non-negative integer. Never consulted for a required
    /// operand, which always takes whatever comes.
    fn wants(&self, next: Option<&str>) -> bool {
        match next {
            None => false,
            Some(tok) if tok.starts_with('-') => false,
            Some(tok) if self.numeric => tok.parse::<usize>().is_ok(),
            Some(_) => true,
        }
    }
}

/// Which `--help` block a flag belongs to, and how its usage line is spelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HelpBlock {
    /// A capture MODE: its own `awl <flag> <operands> [file]` line at the top.
    Modes,
    /// An option under `verification hooks (compose with --screenshot):`,
    /// indented two spaces.
    Options,
}

/// Whether `--help` prints a flag. Both values are documented in the reference —
/// the roster is the whole flag surface either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Listing {
    /// `--help` prints it.
    Shown,
    /// `--help` does not print it: a benchmark, diagnostic or verification hook
    /// (and `--help` itself, which has never listed its own flag).
    Hidden,
}

/// One flag.
pub(crate) struct Flag {
    pub(crate) id: FlagId,
    /// Every spelling that reaches this flag; `names[0]` is the canonical one a
    /// document prints.
    pub(crate) names: &'static [&'static str],
    pub(crate) block: HelpBlock,
    pub(crate) listing: Listing,
    pub(crate) operands: &'static [Operand],
    /// One sentence, matter-of-fact. `{worlds}` expands to the live world
    /// roster — see [`Flag::summary_text`].
    pub(crate) summary: &'static str,
}

/// Declare the whole roster once. The [`FlagId`] enum and the [`FLAGS`] table
/// come from the SAME list, so there is no second list for a maintainer to keep
/// aligned — the `enum_with_all!` move, one payload wider.
macro_rules! flag_roster {
    ( $(
        $id:ident : $names:expr, $block:ident, $listing:ident, $ops:expr, $summary:expr
    );+ $(;)? ) => {
        /// One variant per roster row. `super::parse_args` matches on this with
        /// NO WILDCARD, so a new row fails to compile until the parser reads it.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum FlagId { $($id,)+ }

        impl FlagId {
            /// The sweep roster: every declared id, in roster order. Read by the
            /// roster laws rather than by the parser, which reaches its ids
            /// through [`lookup`] — the same shape as `enum_with_all!`'s `ALL`.
            #[allow(dead_code)]
            pub(crate) const ALL: &'static [FlagId] = &[ $(FlagId::$id,)+ ];
        }

        /// THE ROSTER. Order is `--help`'s order within each block.
        pub(crate) const FLAGS: &[Flag] = &[ $(
            Flag {
                id: FlagId::$id,
                names: $names,
                block: HelpBlock::$block,
                listing: Listing::$listing,
                operands: $ops,
                summary: $summary,
            },
        )+ ];
    };
}

#[path = "flags/roster.rs"]
mod roster;

pub(crate) use roster::{FLAGS, FlagId};

/// THE ROSTER'S BOUNDARY, stated rather than left silent. `fn main` intercepts
/// these hidden diagnostics with a bare `env::args()` scan and returns
/// BEFORE [`super::parse_args`] runs, so they never reach a roster row and a row
/// could not serve them: each one exits the process, and two exist only on
/// macOS. They stay out of the roster deliberately — but a flag added the same
/// way and ALSO given a roster row would leave one flag with two parsers, which
/// is what the roster laws' pre-parse check exists to notice.
#[cfg(test)]
pub(crate) const PRE_PARSE_FLAGS: &[&str] = &[
    "--print-menu-roster",
    "--dump-menu-icon",
    "--fault-write-loop",
    "--persistence-fault-probe",
];

/// The flag a command-line token names, or `None` for anything that is not a
/// flag spelling. THE ONE DOOR: `super::parse_args` never compares an argument
/// against a literal, so a flag the roster does not carry cannot be parsed and a
/// flag it does carry cannot be missed.
pub(crate) fn lookup(arg: &str) -> Option<&'static Flag> {
    FLAGS.iter().find(|f| f.names.contains(&arg))
}

/// The canonical spelling of one roster id — [`lookup`] read backwards, for the
/// few places that must ask about a PARTICULAR flag rather than parse an unknown
/// one, so those places carry no literal of their own either. Total by
/// construction: `tests::every_id_appears_exactly_once_in_the_roster` sweeps
/// `FlagId::ALL` through it.
pub(crate) fn name_of(id: FlagId) -> &'static str {
    FLAGS
        .iter()
        .find(|f| f.id == id)
        .map(Flag::name)
        .unwrap_or_else(|| panic!("`FlagId::{id:?}` has no row in the flag roster"))
}

impl Flag {
    /// The canonical spelling — what a document prints and what a refusal names.
    pub(crate) fn name(&self) -> &'static str {
        self.names[0]
    }

    /// Pull this flag's declared operands off the argument stream. A required
    /// operand that is absent is refused as `<flag> <need>`; an optional one is
    /// consumed only when the next token [`Operand::wants`] it, so a token
    /// that plainly belongs to something else — another flag, or (for a
    /// `numeric` operand) a file argument — is left on the stream for the
    /// parse loop to read next. The caller peeks first, so nothing is removed
    /// from the stream until it is known to belong to this operand.
    pub(crate) fn take_operands(
        &'static self,
        args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    ) -> Result<Operands> {
        let mut vals = Vec::with_capacity(self.operands.len());
        for op in self.operands {
            if op.optional {
                if op.wants(args.peek().map(String::as_str)) {
                    vals.push(args.next().expect("just peeked Some"));
                    continue;
                }
                // Declined (absent, a flag, or — for a numeric operand — the
                // wrong shape) ends the list here, same as the old "stop at
                // the first missing one": no roster row currently declares an
                // operand after an optional one.
                break;
            }
            match args.next() {
                Some(v) => vals.push(v),
                None => anyhow::bail!("{} {}", self.name(), op.need),
            }
        }
        Ok(Operands { flag: self, vals })
    }

    /// How a usage line spells this flag's operands: `OUT.png`, `[DIR]`,
    /// `DIR "0,30,60,90" OUT.png`. Empty for a flag that takes none.
    pub(crate) fn operand_usage(&self) -> String {
        self.operands
            .iter()
            .map(|op| {
                if op.optional {
                    format!("[{}]", op.meta)
                } else {
                    op.meta.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The summary with its one substitution applied. `{worlds}` expands to the
    /// live theme roster, so `--theme`'s help line and its reference row read the
    /// SAME list the flag itself accepts rather than a copy that can drift.
    pub(crate) fn summary_text(&self) -> String {
        if self.summary.contains(WORLDS_TOKEN) {
            return self
                .summary
                .replace(WORLDS_TOKEN, &crate::theme::world_names().join(", "));
        }
        self.summary.to_string()
    }
}

/// The one substitution [`Flag::summary_text`] performs.
const WORLDS_TOKEN: &str = "{worlds}";

/// One flag's operands, as the argument stream supplied them. Carries the flag
/// so a mismatch between what an arm reads and what the roster declares fails
/// LOUDLY and by name instead of as a bare index panic.
pub(crate) struct Operands {
    flag: &'static Flag,
    vals: Vec<String>,
}

impl Operands {
    /// The Nth declared operand, which is present by construction: a required
    /// operand's absence already refused the flag in [`Flag::take_operands`].
    pub(crate) fn req(&self, n: usize) -> &str {
        self.vals.get(n).map(String::as_str).unwrap_or_else(|| {
            panic!(
                "`{}` reads operand {n} but its roster row declares {} — the arm and \
                 the roster disagree about this flag's arity",
                self.flag.name(),
                self.flag.operands.len()
            )
        })
    }

    /// The Nth operand when the roster declares it OPTIONAL: `None` when the
    /// argument stream ran out, so the arm applies its own default.
    pub(crate) fn opt(&self, n: usize) -> Option<&str> {
        self.vals.get(n).map(String::as_str)
    }
}

/// The first line: awl's own invocation, which names no flag.
const USAGE_LINE: &str = "awl [file]";

/// The heading over the [`HelpBlock::Options`] block.
const OPTIONS_HEADING: &str = "verification hooks (compose with --screenshot):";

/// Where a description starts on a [`HelpBlock::Modes`] line, and on a
/// [`HelpBlock::Options`] line. A prefix at or past its column takes the minimum
/// gap instead, so a long flag pushes its own description right rather than
/// shifting the whole block.
const MODE_COLUMN: usize = 40;
const OPTION_COLUMN: usize = 22;
const MIN_GAP: usize = 2;

/// `--help`, generated. Every listed flag, in roster order, under its own
/// block's heading — so the roster is the only thing that decides what a reader
/// of `--help` sees.
pub(crate) fn help_text() -> String {
    let mut out = String::from(USAGE_LINE);
    for f in shown(HelpBlock::Modes) {
        out.push('\n');
        out.push_str(&line(&mode_prefix(f), &f.summary_text(), MODE_COLUMN));
    }
    out.push_str("\n\n");
    out.push_str(OPTIONS_HEADING);
    for f in shown(HelpBlock::Options) {
        out.push('\n');
        out.push_str(&line(&option_prefix(f), &f.summary_text(), OPTION_COLUMN));
    }
    out
}

/// Every listed flag in one block, in roster order.
fn shown(block: HelpBlock) -> impl Iterator<Item = &'static Flag> {
    FLAGS
        .iter()
        .filter(move |f| f.block == block && f.listing == Listing::Shown)
}

/// `awl --capture-held DIR "0,30,60,90" OUT.png [file]` — a capture mode's
/// whole invocation, since that is what a reader retypes.
fn mode_prefix(f: &Flag) -> String {
    let ops = f.operand_usage();
    if ops.is_empty() {
        format!("awl {} [file]", f.name())
    } else {
        format!("awl {} {ops} [file]", f.name())
    }
}

/// `  --capture-size WxH` — an option, indented under the block heading.
fn option_prefix(f: &Flag) -> String {
    let ops = f.operand_usage();
    if ops.is_empty() {
        format!("  {}", f.name())
    } else {
        format!("  {} {ops}", f.name())
    }
}

fn line(prefix: &str, summary: &str, column: usize) -> String {
    let len = prefix.chars().count();
    let pad = if len < column { column - len } else { MIN_GAP };
    format!("{prefix}{}{summary}", " ".repeat(pad))
}

#[cfg(test)]
#[path = "flags/tests.rs"]
mod tests;
