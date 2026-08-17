//! src/credits.rs — the embedded CREDITS.md text, `include_str!`'d at build
//! time (zero network, mirroring every other bundled asset in this app).
//!
//! Summoned via Cmd-P → "Credits" (`Action::OpenCredits`, see `commands.rs` +
//! `actions.rs`), which opens a summoned, scrollable, READ-ONLY viewer
//! (`OverlayKind::Credits`, `overlay/comparison.rs`'s `OverlayState::new_credits`)
//! over the document — never a buffer swap. The active buffer's path and
//! version are untouched by opening, scrolling or dismissing it: this text is
//! pushed straight into the relocated document viewport
//! (`comparison::prose_for`), the same mechanism the History/Conflict
//! comparison panes use to show read-only prose without touching the buffer.
//! There is no on-disk refresh copy and nothing for autosave to clobber,
//! because there is no buffer here to autosave in the first place.

/// The full text of the repo's `CREDITS.md`, embedded at compile time. The
/// `include_str!` path itself lives in the ONE owner, `crate::embedded_docs`
/// (a doc move is a one-line edit there); this re-export keeps
/// `credits::CREDITS_MD` as the cohesive public name every consumer imports.
pub use crate::embedded_docs::CREDITS_MD;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_text_is_nonempty_and_mentions_the_license_door() {
        assert!(!CREDITS_MD.is_empty());
        assert!(CREDITS_MD.contains("GPL-3.0"));
        assert!(CREDITS_MD.contains("THIRD-PARTY-LICENSES"));
    }
}
