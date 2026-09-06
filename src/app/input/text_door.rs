//! **THE INSERTION-DOOR CENSUS: EVERY PRODUCTION PATH THAT EDITS THE FOCUSED
//! BUFFER'S TEXT, AND THE GATE EACH ONE ANSWERS TO.**
//!
//! # Why a census, and not three `if`s
//!
//! The read-only prose family (`OverlayState::shows_read_only_prose` — Version
//! History's timeline, the external-change conflict, the Credits viewer)
//! RELOCATES the one document layer into a workspace pane and substitutes a
//! transcript for the user's text. The buffer behind it is untouched and
//! invisible. A character that reaches it while a reader is looking at
//! CREDITS.md is a silent edit of a document nobody can see.
//!
//! The wall that stops that was written against THREE doors, and the lane that
//! wrote it found two of them (`ReplaceSelectedText`, `SetValue`) that nobody
//! had listed. That is the real defect: the class grows every time an input
//! capability lands, and a new door ships OPEN unless something forces it to
//! declare itself. A per-caller `if` is a rule each new caller has to remember,
//! and a roster of three is a rule that was already forgotten twice.
//!
//! # The seam
//!
//! [`TextDoor`] is the roster of EVERY such path, and every member states its
//! [`DoorGate`] in one wildcard-free match ([`TextDoor::gate`]) — so a new door
//! cannot compile until its author says whether the wall refuses it, the action
//! layer already covers it, or it is a NAMED exemption with a reason.
//!
//! The forcing function underneath is ownership, not discipline.
//! `DocumentSession`'s four raw text mutators are PRIVATE to the `document`
//! module; the only text edit reachable from anywhere else in `crate::app` is
//! [`App::write_document_text`], which takes a [`TextDoor`] as its first
//! argument. So a new door cannot reach the rope at all without naming a roster
//! member, and `app::tests::insertion_census` binds each member back to the one
//! source site that presses it.
//!
//! # The three gates
//!
//! * [`DoorGate::Wall`] — asks [`App::text_door_open`] before it writes. Every
//!   door that bypasses `App::apply` entirely and once asked only
//!   `has_active()`.
//! * [`DoorGate::ActionLevel`] — the shared action core. Its gate is one layer
//!   up and is not duplicated here: with a card or a summoned field up,
//!   `actions::intercept_action` consumes every action before a buffer verb can
//!   run, and that intercept sits at the ACTION level precisely because a macOS
//!   menu key equivalent, a menu click and a palette row's `Effect::RunAction`
//!   all arrive there without ever touching a keymap.
//! * [`DoorGate::Exempt`] — a path that must NOT be walled, carrying the reason.
//!   Two of them are the reading surfaces' own verbs: refusing those would break
//!   the very feature the surface exists for.
//!
//! # What this census deliberately does NOT cover
//!
//! WHOLE-SLOT replacement — opening a file, the clean-buffer reload after an
//! external change, starting a fresh document, session restore. Those swap the
//! `Entry` rather than editing a rope and take no undo step, and they need no
//! law of their own: `DocumentSession::active` is a PRIVATE field, so the
//! compiler already confines every one of them to the `document` module, where
//! each is a named transition (`app/tests/source_audit.rs` audits that module's
//! own internal discipline).

use crate::app::*;

enum_with_all! {
/// EVERY production path that inserts or replaces text in the FOCUSED buffer.
///
/// Ordered by gate: the action core, then the doors behind the wall, then the
/// named exemptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum TextDoor {
    /// The shared action core (`actions::apply_transition`, reached through the
    /// one mutable-buffer loan in `app/apply.rs`) — every chord, menu item,
    /// context-menu row, palette row and paste.
    ActionCore,
    /// The platform IME's COMMITTED composition — `WindowEvent::Ime(Commit)`,
    /// the finalized text of a CJK/dead-key composition. It never resolves
    /// through the keymap, so the action intercept never sees it.
    Ime,
    /// An assistive technology replacing the document's SELECTED text
    /// (`SemanticRequest::ReplaceSelectedText` on the document node).
    AssistiveReplaceSelection,
    /// An assistive technology setting the document's WHOLE value
    /// (`SemanticRequest::SetValue` on the document node) — a replace of every
    /// character, which is the same wall for a larger blast radius.
    AssistiveSetValue,
    /// The inline-image drag-resize write-back: a MOUSE gesture stamping the
    /// settled `|NNN` width hint into the image's alt text on button release.
    /// The census's own find — a door with no keyboard, no keymap and no
    /// assistive request, which is exactly why the three-door roster missed it.
    ImageWidthDrag,
    /// "Insert Date" — the live half of `Effect::InsertDate`, which reads the
    /// real wall clock and inserts at the caret. The effect is produced by the
    /// action core, but the INSERT happens here, after that gate, so the door is
    /// walled in its own right rather than trusting a gate one call away.
    InsertDate,
    /// Version History's own RESTORE verb, pressed from the timeline that is
    /// itself a read-only prose surface.
    HistoryRestore,
    /// The external-change conflict's "use disk version", pressed from the
    /// conflict card, which is itself a read-only prose surface.
    ConflictTakeTheirs,
    /// Relaunch recovery putting the user's rescued text back as a document
    /// BECOMES active — startup and every open.
    RelaunchRecoveryAdopt,
    /// `--bench-a11y`'s typing arm, against its own headless `App`.
    AccessibilityBench,
    /// The hidden persistence fault-probe subprocess (`--persistence-probe`),
    /// seeding a payload into a killable headless `App`.
    PersistenceFaultProbe,
    /// The capture harness's own `--keys` replay. It is the item's own worked
    /// example of a named exemption, and the only member that never touches
    /// `DocumentSession` at all.
    HeadlessReplay,
}
}

/// Where one census member lives in the source tree.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) struct DoorSite {
    /// The ONE production file that names this door, relative to `src/`.
    pub(in crate::app) file: &'static str,
    /// Whether it reaches the rope through [`App::write_document_text`].
    pub(in crate::app) through_the_door: bool,
}

/// What stops a door from writing into a document the reader cannot see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum DoorGate {
    /// Asks [`App::text_door_open`] immediately before it writes.
    Wall,
    /// Gated one layer up, at the ACTION level, by `actions::intercept_action`.
    ActionLevel,
    /// Deliberately not walled. The string is the REASON, and it is required:
    /// an unnamed exemption is the bug this census exists to prevent.
    Exempt(&'static str),
}

impl TextDoor {
    /// **THE ONE WILDCARD-FREE MATCH.** A new door does not compile until its
    /// author answers here.
    ///
    /// Written out member by member rather than collapsed to a default, because
    /// the answer genuinely differs across the roster and the next door's answer
    /// is not knowable from the last one's.
    pub(in crate::app) fn gate(self) -> DoorGate {
        match self {
            TextDoor::ActionCore => DoorGate::ActionLevel,
            TextDoor::Ime
            | TextDoor::AssistiveReplaceSelection
            | TextDoor::AssistiveSetValue
            | TextDoor::ImageWidthDrag
            | TextDoor::InsertDate => DoorGate::Wall,
            TextDoor::HistoryRestore => DoorGate::Exempt(
                "the reading surface's OWN verb — Version History IS a read-only prose \
                 surface, and restoring a version is the single thing its timeline is \
                 for. Walling it would refuse the feature rather than protect it.",
            ),
            TextDoor::ConflictTakeTheirs => DoorGate::Exempt(
                "the reading surface's OWN verb — the external-change conflict IS a \
                 read-only prose surface, and \"use disk version\" is one of the two \
                 resolutions it exists to offer.",
            ),
            TextDoor::RelaunchRecoveryAdopt => DoorGate::Exempt(
                "not an input surface: it runs as a document BECOMES active (startup, \
                 every open), before any card can be up, and it restores text the user \
                 already had rather than adding any.",
            ),
            TextDoor::AccessibilityBench => DoorGate::Exempt(
                "`--bench-a11y` drives its own headless `App` with no window, no \
                 overlay and no user — there is no reading surface for it to write \
                 behind.",
            ),
            TextDoor::PersistenceFaultProbe => DoorGate::Exempt(
                "a hidden diagnostic subprocess seeding a payload into a killable \
                 headless `App`; it opens no surface and is never reachable from the \
                 running editor.",
            ),
            TextDoor::HeadlessReplay => DoorGate::Exempt(
                "structurally outside `App`: the capture harness's replay session owns \
                 its OWN `Buffer` and never touches `DocumentSession`, so there is no \
                 focused document behind a surface for it to reach.",
            ),
        }
    }

    /// **WHERE THIS DOOR LIVES, AND WHETHER IT COMES THROUGH THE DOOR
    /// FUNCTION.** Wildcard-free for the same reason [`Self::gate`] is: a new
    /// door has to say where it lives, and `app::tests::insertion_census` then
    /// checks that it really does live there, that nothing else names it, and
    /// that the set of files calling [`App::write_document_text`] is exactly the
    /// set this roster claims.
    ///
    /// Two doors do not call that function and are named in a COMMENT at their
    /// site instead — the action core, whose rule lives one layer up in
    /// `actions::intercept_action`, and the replay session, which is outside
    /// `crate::app` and cannot even reference this type. Naming them anyway is
    /// the point: an unnamed exemption is the bug this census exists to prevent.
    #[cfg(test)]
    pub(in crate::app) fn site(self) -> DoorSite {
        let through = |file| DoorSite {
            file,
            through_the_door: true,
        };
        let named_only = |file| DoorSite {
            file,
            through_the_door: false,
        };
        match self {
            TextDoor::ActionCore => named_only("app/document.rs"),
            TextDoor::HeadlessReplay => named_only("main/run/effect_interpreter.rs"),
            TextDoor::Ime => through("app/input/ime.rs"),
            TextDoor::AssistiveReplaceSelection | TextDoor::AssistiveSetValue => {
                through("app/semantic/requests.rs")
            }
            TextDoor::ImageWidthDrag | TextDoor::HistoryRestore => through("app/files/verbs.rs"),
            TextDoor::InsertDate => through("app/files/settings.rs"),
            TextDoor::ConflictTakeTheirs | TextDoor::RelaunchRecoveryAdopt => {
                through("app/files/external.rs")
            }
            TextDoor::AccessibilityBench => through("app/semantic/bench.rs"),
            TextDoor::PersistenceFaultProbe => through("app/persistence/fault_probe.rs"),
        }
    }

    /// **DOES A READ-ONLY PRESENTATION REFUSE THIS DOOR?** Derived from
    /// [`Self::gate`] rather than re-spelled, so the wall, the census and the
    /// sweeps cannot answer differently. [`DoorGate`] itself stays inside this
    /// module: a consumer asks the roster a question, never re-decides it.
    pub(in crate::app) fn is_walled(self) -> bool {
        matches!(self.gate(), DoorGate::Wall)
    }

    /// This door's exemption reason, if it has one — `None` for a door some gate
    /// stops. The predicate the sweeps derive their subject from, and the string
    /// `app::tests::insertion_census` requires to be a real sentence.
    #[cfg(test)]
    pub(in crate::app) fn exemption_reason(self) -> Option<&'static str> {
        match self.gate() {
            DoorGate::Exempt(reason) => Some(reason),
            DoorGate::Wall | DoorGate::ActionLevel => None,
        }
    }
}

/// The shape of one text edit, as the census's door takes it. Wildcard-free at
/// its single dispatch point (`DocumentSession::apply_text_edit`), so a new edit
/// shape is a conscious addition rather than a fifth raw mutator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum TextEdit<'a> {
    /// One character at the caret — the typing shape, which coalesces into the
    /// open undo group.
    Char(char),
    /// A string at the caret, replacing any selection, sealed on both sides.
    Insert(&'a str),
    /// A char-indexed range replaced by `text`.
    ReplaceRange {
        start: usize,
        end: usize,
        text: &'a str,
    },
    /// The WHOLE document replaced, as one undoable edit.
    Whole(&'a str),
}

impl App {
    /// **THE ONE PRODUCTION TEXT-MUTATION DOOR.** Every path in the census that
    /// touches `DocumentSession` comes through here, naming itself.
    ///
    /// Returns whether the edit was performed, so a caller that must report
    /// (an assistive request's handled flag) can.
    ///
    /// It is not a policy of its own: it asks [`Self::text_door_open`], which
    /// asks `door.gate()`. An exempt door still gets the `has_active()` check
    /// every one of these paths already carried.
    pub(in crate::app) fn write_document_text(
        &mut self,
        door: TextDoor,
        edit: TextEdit<'_>,
    ) -> bool {
        if !self.text_door_open(door) {
            return false;
        }
        self.document.apply_text_edit(edit);
        true
    }

    /// **THE ONE WALL.** Whether `door` may write to the active document right
    /// now.
    ///
    /// Two conditions, and the second is this wall's whole subject:
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
        !(door.is_walled() && self.presents_read_only_prose())
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
