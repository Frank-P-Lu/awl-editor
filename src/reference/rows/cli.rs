//! The command-line section's rows, read from the flag roster
//! (`crate::args::flags::FLAGS`) — the same table `parse_args` resolves every
//! argument through and `--help` prints from.
//!
//! Nothing here decides what a flag is called, what it takes, or what it does.
//! What IS authored here is the presentation: three sub-tables, and the fact
//! that the third one exists at all. `--help` prints only the `Shown` rows;
//! documenting the `Hidden` ones anyway is what makes a new flag unable to land
//! undocumented — `super::super::law`'s byte-diff fails until the document has
//! been regenerated, whichever listing the new flag chose.

use crate::args::flags::{self, Flag, HelpBlock, Listing};

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// Every command-line flag: the capture modes, the options `--help` lists, and
/// the ones it does not.
pub(crate) fn cli() -> Vec<Block> {
    vec![
        block(
            Some("Capture modes"),
            Some(
                "At most one capture mode per run: awl refuses a second rather than \
                 silently preferring one.",
            ),
            table(|f| f.block == HelpBlock::Modes && f.listing == Listing::Shown),
        ),
        block(
            Some("Options"),
            None,
            table(|f| f.block == HelpBlock::Options && f.listing == Listing::Shown),
        ),
        block(
            Some("Unlisted flags"),
            Some(
                "`awl --help` does not print these. They work like any other flag; they \
                 are benchmark, diagnostic and verification hooks rather than everyday \
                 arguments. Every benchmark opens no window.",
            ),
            table(|f| f.listing == Listing::Hidden),
        ),
    ]
}

/// One sub-table: the roster's own order, its own spellings, its own operand
/// list, its own sentence.
fn table(keep: impl Fn(&Flag) -> bool) -> Table {
    let mut t = Table::new(&["Flag", "Takes", "What it does"]);
    for f in flags::FLAGS.iter().filter(|f| keep(f)) {
        t.push(vec![
            // Every spelling, canonical first — so an alias is documented
            // wherever it exists rather than only where someone remembered.
            Cell::code(f.names.join(", ")),
            Cell::code_or_dash(&f.operand_usage()),
            Cell::text(f.summary_text()),
        ]);
    }
    t
}
