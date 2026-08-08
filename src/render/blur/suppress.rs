//! THE FROST, SUPPRESSED — a test-only door with exactly one purpose: giving a law two
//! frames whose ONLY difference is the summoned card's own drawing.
//!
//! THE CARD'S INK CANNOT BE ISOLATED FROM A FROSTED FRAME, and `frost_card_ink` exists
//! because of it: outside the shape's reach the same frame shows the world's live ground at
//! full sharpness, so the derived oracle's flagged set is a superset of the card's drawing
//! whose surplus is the WORLD's, and it does not invert into "where the card is". An
//! open-versus-closed difference does not rescue it either — wherever the frost lands, that
//! difference carries `blur(ground) − ground`, and that is precisely the region a
//! completeness claim has to read.
//!
//! Turn the frost off and the confound is identically zero. Two frames of the same
//! document at the same size on the same world, one with the picker up and one without,
//! share their ground and their document exactly, so the residue between them is the
//! card's drawing and nothing else. That is a positive oracle for "where the card IS",
//! built by removing the term that made the negative one negative — not by inverting a
//! veto.
//!
//! It is NOT carried by `testlock::serial()`, so a law that sets it restores it on the way
//! out — the same discipline the menu-bar arm follows in the same test family. There is no
//! such thing as this flag in a ship build: the whole module is `cfg(test)`, and
//! `frost_mode`'s branch on it compiles away with it.

use std::sync::atomic::{AtomicBool, Ordering};

static SUPPRESSED: AtomicBool = AtomicBool::new(false);

/// Is the frost suppressed for this process right now? Read once per `frost_mode`.
pub(crate) fn frost_suppressed() -> bool {
    SUPPRESSED.load(Ordering::Relaxed)
}

/// Suppress (or restore) the frost. The caller holds `testlock::serial()` and restores
/// `false` on every exit path, including the unwinding one.
pub(crate) fn set_frost_suppressed(on: bool) {
    SUPPRESSED.store(on, Ordering::Relaxed);
}
