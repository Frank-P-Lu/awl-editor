//! **THE ONE WALL EVERY TEXT-INSERTION DOOR OUTSIDE THE SHARED ACTION CORE
//! HITS.**
//!
//! # Why a wall exists at all
//!
//! The read-only prose family (`OverlayState::shows_read_only_prose` — Version
//! History's timeline, the external-change conflict, the Credits viewer)
//! RELOCATES the one document layer into a workspace pane and substitutes a
//! transcript for the user's text. The buffer behind it is untouched and
//! invisible. A character that reaches it while a reader is looking at
//! CREDITS.md is a silent edit of a document nobody can see.
//!
//! The KEYMAP door is already shut, structurally and for every overlay: with a
//! card up, `actions::intercept_action` consumes every action before
//! `apply_buffer_action` runs, so no chord can reach the rope. Every OTHER door
//! bypasses `App::apply` entirely and asked only `has_active()` — the platform
//! IME's committed composition, and the two assistive-technology requests that
//! write document text. Those are the doors this module owns.
//!
//! # Why a typed door rather than a per-caller check
//!
//! A per-caller `if` is a rule each new caller has to remember. [`TextDoor`] is
//! the roster of doors, [`App::text_door_open`] answers for all of them in one
//! wildcard-free match, and `app::tests::read_only_surface`'s sweep drives
//! [`TextDoor::ALL`] with a wildcard-free match of its own — so a fourth door
//! cannot be added without saying, in production, whether a read-only surface
//! refuses it, and cannot be added without the sweep failing to compile until it
//! is actually driven.

use crate::app::*;

/// EVERY door through which text reaches the active document from OUTSIDE the
/// shared action core (`actions::apply_transition`).
///
/// The core's own door is deliberately absent: it is shut for every summoned
/// overlay one layer up, by `actions::intercept_action`, and duplicating that
/// here would be a second owner of a rule that already has one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum TextDoor {
    /// The platform IME's COMMITTED composition — `WindowEvent::Ime(Commit)`,
    /// the finalized text of a CJK/dead-key composition. It never resolves
    /// through the keymap, so the overlay intercept never sees it.
    Ime,
    /// An assistive technology replacing the document's SELECTED text
    /// (`SemanticRequest::ReplaceSelectedText` on the document node).
    AssistiveReplaceSelection,
    /// An assistive technology setting the document's WHOLE value
    /// (`SemanticRequest::SetValue` on the document node) — a replace of every
    /// character, which is the same wall for a larger blast radius.
    AssistiveSetValue,
}

impl TextDoor {
    /// The roster, for the sweep. A door absent from here is a door no law
    /// drives.
    #[cfg(test)]
    pub(in crate::app) const ALL: [TextDoor; 3] = [
        TextDoor::Ime,
        TextDoor::AssistiveReplaceSelection,
        TextDoor::AssistiveSetValue,
    ];

    /// **DOES A READ-ONLY PRESENTATION REFUSE THIS DOOR?** Wildcard-free, so a
    /// new door must answer before it compiles.
    ///
    /// All three answer `true`, and the match is written out rather than
    /// collapsed to a constant precisely because the next door might not: a
    /// future paste-into-a-comparison-search would be a door that reaches
    /// something other than the rope.
    fn refused_by_read_only(self) -> bool {
        match self {
            TextDoor::Ime | TextDoor::AssistiveReplaceSelection | TextDoor::AssistiveSetValue => {
                true
            }
        }
    }
}

impl App {
    /// **THE ONE WALL.** Whether `door` may write to the active document right
    /// now.
    ///
    /// Two conditions, and the second is this item's whole subject:
    ///
    ///   * there IS an active document (the pre-existing `has_active()` gate
    ///     every one of these doors already carried); and
    ///   * the document is not being PRESENTED read-only — the family predicate
    ///     `OverlayState::shows_read_only_prose`, derived from the comparison
    ///     roster rather than pinned to a named kind, so a fourth read-only
    ///     surface is walled the day it compiles.
    ///
    /// Refusal is SILENT (DESIGN's calm): no notice, no beep, no recoil. A
    /// reader who types on a reading surface has done nothing wrong, and a
    /// scolding surface is the opposite of what this one is for.
    pub(in crate::app) fn text_door_open(&self, door: TextDoor) -> bool {
        if !self.document.has_active() {
            return false;
        }
        !(door.refused_by_read_only() && self.presents_read_only_prose())
    }

    /// The family membership, asked of the open card. Its own owner
    /// (`OverlayState::shows_read_only_prose`) derives it from the comparison
    /// roster; this is only the App's route to it, so the wall above and the
    /// caret layer's twin cannot read two different rosters.
    pub(in crate::app) fn presents_read_only_prose(&self) -> bool {
        self.workspace_state
            .overlay()
            .is_some_and(crate::overlay::OverlayState::shows_read_only_prose)
    }
}
