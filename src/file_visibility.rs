//! FILE VISIBILITY — item 77's ONE sticky picker-listing switch, replacing
//! the retired standalone "Show hidden files" toggle (`Action::ToggleHiddenFiles`,
//! Cmd-Shift-`.`, the per-overlay `OverlayState::show_hidden` flag).
//!
//! Two states:
//!   * **Text** (default) — the Browse file picker lists normal, non-hidden
//!     files awl can actually decode and edit (see [`crate::openable`]), plus
//!     every folder. Hidden entries (dotfiles) AND unsupported/binary files
//!     stay out of the listing.
//!   * **All** — reveals BOTH hidden entries and unsupported/binary files, so
//!     one switch answers "show me the complete project". An unsupported row
//!     stays VISIBLE for context but is never openable (see [`crate::openable`],
//!     the SEPARATE capability owner that decision belongs to).
//!
//! A process-global [`AtomicBool`] (DEFAULT OFF = Text), mirroring the
//! `page`/`spell`/`nits` sticky-toggle pattern: the Settings menu's "File
//! visibility" row, the sticky config pref, and `overlay::state::refilter`'s
//! display filter all read this ONE place. Reached through the Settings menu
//! only (no dedicated chord) — a picker-local ephemeral toggle was the old
//! shape being retired; this is a sticky, app-wide preference instead.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether "All" is active (dotfiles + unsupported files both revealed).
/// DEFAULT `false` (Text) — the calm, curated default a new install opens to.
static ALL_ON: AtomicBool = AtomicBool::new(false);

/// True when "All" visibility is active.
pub fn all_on() -> bool {
    ALL_ON.load(Ordering::Relaxed)
}

/// Set the live global directly (config load / capture `--config` seeding).
pub fn set_all_on(on: bool) {
    ALL_ON.store(on, Ordering::Relaxed);
}

/// Flip the global, returning the NEW value — the Settings menu's "File
/// visibility" toggle row rides the generic bool mechanism
/// (`App::setting_toggle`), which reads-then-writes itself, so this is
/// test-only sugar (offered for symmetry with `page::toggle`/`spell::toggle`).
#[allow(dead_code)]
pub fn toggle() -> bool {
    let next = !all_on();
    set_all_on(next);
    next
}

/// The setting row's calm VALUE-cell word: `"Text"` / `"All"`.
pub fn label() -> &'static str {
    if all_on() { "All" } else { "Text" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_text_and_toggles_both_ways() {
        let _g = crate::testlock::serial();
        let before = all_on();
        set_all_on(false);
        assert_eq!(label(), "Text");
        assert!(toggle(), "Text -> All");
        assert_eq!(label(), "All");
        assert!(all_on());
        assert!(!toggle(), "All -> Text");
        assert_eq!(label(), "Text");
        set_all_on(before); // never leak into a sibling test
    }
}
