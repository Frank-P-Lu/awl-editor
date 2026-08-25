//! CLI argument parsing and capture-mode selection.

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::capture::{self, CaptureOpts};
use crate::config::{self, Config};
use crate::keymap::KeymapState;
use crate::{caret, debug, hud, keyspec, lifetime, page, theme, whichkey};

// THE FLAG ROSTER — the one owner of "what flags exist". `parse_args` below
// compares no argument against a literal: every `--…` token resolves through
// `flags::lookup`, every operand comes off the stream through
// `Flag::take_operands`, and the dispatch is a no-wildcard match on `FlagId`, so
// a roster row with no arm fails to compile. `--help` and the reference's
// command-line section are both generated from the same table — which is why the
// module is `pub(crate)`: `crate::reference::rows::cli` reads the roster, and
// reading it is the whole point. Nothing outside PARSES with it; `lookup` and
// `take_operands` have exactly one caller, the loop in `parse::parse_flag_loop`.
#[path = "args/flags.rs"]
pub(crate) mod flags;
#[path = "args/modes.rs"]
mod modes;
#[path = "args/parse.rs"]
mod parse;
#[path = "args/parsers.rs"]
mod parsers;
use flags::FlagId;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use modes::LiveAppSpec;
pub(crate) use modes::Mode;
pub(crate) use parse::parse_args;
use parsers::*;

/// Resolve the DEFAULT-FOLDER CANDIDATE: explicit `--default-folder`, else
/// `~/notes` (`$HOME/notes`), else `./notes` if HOME is unset.
///
/// The candidate is a first-run launch fallback only when the CLI or config
/// explicitly supplied the setting. An unconfigured launch uses awl's data
/// root; `run::location::resolve_launch_context` owns and tests that gate.
pub(crate) fn resolve_default_folder(default_folder: &Option<PathBuf>) -> PathBuf {
    if let Some(n) = default_folder {
        return n.clone();
    }
    match crate::fs::home_dir() {
        Some(home) => home.join("notes"),
        None => PathBuf::from("notes"),
    }
}

#[cfg(test)]
#[path = "args/tests.rs"]
mod tests;
