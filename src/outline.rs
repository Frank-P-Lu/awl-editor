//! The persistent MARGIN OUTLINE — the ambient table-of-contents that lingers
//! in the page margin so the document's structure stays oriented without a
//! summoned picker (PHILOSOPHY.md's "orientation lingers in two margin
//! surfaces" amendment). This module owns the process-global on/off flag; the
//! heading DATA rides the markdown parse the styling pass already pays for
//! (`markdown::headings_from_spans` → `TextPipeline::outline_headings`), and the
//! RENDER lands in a later phase.
//!
//! DEFAULT ON (flipped 2026-07-09 — a USER-DECIDED taste reversal of the
//! original opt-in-off call): the outline shipped opt-in because it was new,
//! unproven ambient chrome; having lived with it, the user's call is that the
//! orientation it gives is worth showing by default, like the other sticky
//! toggles (WYSIWYG / spellcheck / nits). A user config `outline = false`
//! still wins (the sticky-pref override reads the same either direction — see
//! [`crate::config::Config::outline_on`]).
//!
//!   * [`OUTLINE_ON`] — whether the margin outline is drawn (DEFAULT ON), a
//!     [`crate::toggle::Toggle`] — see that module for the shared mechanism.
//!   * [`outline_on`] / [`set_outline_on`] / [`toggle`] — the readers/writers.
//!
//! Set once at launch from the config sticky pref (`config::outline`, via
//! `Config::apply_sticky_globals`), flipped live by the "Toggle outline" command
//! (`Action::ToggleOutline`) and the settings menu. The render reads
//! [`outline_on`] each reshape, so a default `--screenshot` of a heading-free /
//! non-markdown / page-mode-off buffer stays byte-identical (the outline draws
//! nothing regardless of `on` when there's no heading to show); a markdown
//! buffer WITH headings under page mode now legitimately shows the outline in
//! a default capture, where it previously did not.

use crate::toggle::Toggle;

/// Whether the persistent margin outline is drawn. DEFAULT ON (see the module
/// doc's 2026-07-09 taste reversal) — the calm room shows the outline's quiet
/// orientation unless you turn it off (palette / `Cmd-Shift-O` / config
/// `outline = false`).
/// The value this flag carries on a fresh install, before any config or
/// settings write — the ONE owner of that fact, read both by the static
/// below and by the generated reference (`settings::toggle_default`).
pub(crate) const OUTLINE_DEFAULT: bool = true;
static OUTLINE_ON: Toggle = Toggle::new(OUTLINE_DEFAULT);

/// True when the margin outline is enabled (read by the renderer each reshape
/// + by the capture sidecar's `outline` block, so the two can never disagree).
pub fn outline_on() -> bool {
    OUTLINE_ON.on()
}

/// Set the outline on/off explicitly — the config sticky-pref launch-apply
/// (`Config::apply_sticky_globals`) and the settings-menu toggle.
pub fn set_outline_on(on: bool) {
    OUTLINE_ON.set(on);
}

/// Flip the outline and return the now-active state (the `Cmd-Shift-O` chord +
/// palette "Toggle outline").
pub fn toggle() -> bool {
    OUTLINE_ON.toggle()
}

/// **THE ONE RULE for "would THIS document give the margin outline rows to
/// draw"** — a markdown buffer carrying at least one heading, and nothing else.
///
/// Deliberately free of every ROOM-level fact ([`outline_on`], page mode, window
/// width): those are asked once, of the room, by the rail RESERVATION
/// (`render::geometry::TextPipeline::outline_wants_rail`). What is left is a
/// question about a DOCUMENT, and three surfaces have to ask it of three
/// different documents:
///
///   * the DRAW gate (`render::chrome::outline::outline_layout`) asks it of the
///     buffer on screen — no headings, no rows, whatever the room reserved;
///   * the reservation's ACTIVE half asks it of that same buffer;
///   * [`crate::buffers::BufferRegistry::park`] asks it of every OTHER open
///     buffer, so the reservation can be a fact about the WORKING SET rather
///     than about whichever file the reader happens to be looking at — which is
///     what stops a buffer switch from sliding the whole page sideways.
///
/// Spelled `is_markdown && has_heading` in three places, those three drift the
/// moment one of them grows a condition; asked here, a new caller inherits the
/// rule instead of re-deriving it.
pub fn document_wants_rail(is_markdown: bool, has_heading: bool) -> bool {
    is_markdown && has_heading
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_is_on_by_default_and_toggles() {
        let _g = crate::testlock::serial();
        set_outline_on(true);
        assert!(
            outline_on(),
            "the margin outline is ON by default (2026-07-09 taste flip)"
        );
        assert!(!toggle(), "toggle turns it off and reports the new state");
        assert!(!outline_on());
        assert!(toggle(), "toggle turns it back on");
        assert!(outline_on());
        set_outline_on(true);
    }
}
