//! src/reference_doc.rs — the embedded `REFERENCE.md` text, `include_str!`'d
//! at build time (zero network, mirroring `guide.rs`/`credits.rs`'s exact
//! pattern).
//!
//! Summoned via Cmd-P → "Reference" (`Action::OpenReference` /
//! `Effect::OpenReference`, see `commands.rs` + `actions.rs`), which opens
//! this text into the buffer exactly like Credits opens `CREDITS.md` and
//! Guide opens `GUIDE.md` — see `App::open_reference` (`app/files/open.rs`,
//! the shared `open_bundled_doc` owner all three route through) for why it is
//! written to a real on-disk path (under `fs::data_root()`, refreshed to the
//! embedded text on every open) rather than left path-less: a path-less
//! buffer is indistinguishable from the SCRATCH surface to the autosave
//! engine (`App::autosave_flush`'s `buffer.path().is_none()` arm stashes it as
//! scratch), which would silently clobber the user's real scratch stash the
//! next time autosave flushes. Routing through a real path keeps Reference an
//! ordinary, harmlessly-editable buffer instead.
//!
//! UNLIKE `guide.rs`, there is no per-open rendering step: `REFERENCE.md`
//! carries no `{{key:slug}}` chord tokens (its command table already lists
//! both the mac and Linux chord under explicit columns, generated for both
//! conventions at once — see `src/reference/rows/commands.rs`), so the
//! embedded text opens verbatim, like `credits.rs`.
//!
//! `src/reference.rs` (test-only) is the DIFFERENT module that GENERATES
//! `REFERENCE.md`'s tables from awl's live rosters and holds them to the tree
//! with drift laws — nothing there ships in a binary. This module is the
//! runtime companion: the one door that makes the generated document
//! reachable from inside the running app, exactly as `guide.rs`/`credits.rs`
//! already do for their own documents.

/// The full text of the repo's `REFERENCE.md`, embedded at compile time. The
/// `include_str!` path itself lives in the ONE owner, `crate::embedded_docs`
/// (a doc move is a one-line edit there); this re-export keeps
/// `reference_doc::REFERENCE_MD` as the cohesive public name every consumer
/// imports.
pub use crate::embedded_docs::REFERENCE_MD;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_text_is_nonempty_and_carries_the_commands_table() {
        assert!(!REFERENCE_MD.is_empty());
        assert!(REFERENCE_MD.contains("## Commands"));
        assert!(REFERENCE_MD.contains("Nothing in this file is transcribed by hand."));
    }
}
