//! The SUMMONED, TRANSIENT navigation overlay (go-to file / switch project /
//! one-level browse).
//!
//! The overlay is NOT a sidebar/tree/tabs: it appears, is used, and VANISHES on
//! pick. While it is `Some`, typed chars edit the overlay QUERY (never the
//! buffer), Up/Down move the selection, Enter opens the highlighted item, and
//! Esc/C-g cancels. All of that is driven through `actions::apply_transition`, so the
//! `--keys` headless replay can open it, type to filter, move, and accept — the
//! whole flow stays agent-verifiable and serializable to the capture sidecar.
//!
//! Three kinds share the one card:
//!   * `Goto`    — the active project's flat file index (fuzzy jump).
//!   * `Project` — TWO features behind one card shape, disambiguated by
//!     [`journey::Bind`] rather than by kind (`OverlayState::foot_hint_scoped`'s
//!     doc has the full split). Plainly summoned (Switch Project — no bind, or
//!     `Bind::Value`), it is a FLAT picker over the workspace's direct children
//!     only: Enter on a folder switches the root immediately (the synthetic
//!     accept-this-folder row — [`here_folder_label`] — picks the level's own
//!     directory), and there is no ascend affordance —
//!     Left/Right cycle the lens strip, not depth. Descended into from a
//!     Settings folder-VALUE row (`Bind::Path`), it keeps the full
//!     destination-navigator grammar: Enter on a folder DESCENDS into it and
//!     Backspace with an empty query ASCENDS, even above the workspace. Git
//!     folders carry a dim `git` tag in the row's secondary column either way.
//!   * `Browse`  — ONE directory level at a time for the active root. Enter on a
//!     FOLDER descends (the list becomes that folder's children); Left/Backspace
//!     ASCENDS; Enter on a FILE opens it and closes. Git folders are marked. It
//!     is still summoned + transient — it vanishes on open/cancel, never a tree.

mod build;
mod capture;
pub(crate) mod comparison;
mod facet;
mod journey;
mod kind;
mod nav;
mod semantic;
mod state;
pub(crate) mod workspace;

#[allow(unused_imports)] // HERE_LABEL / here_folder_label: read by the row-label laws
pub use build::{
    BuildCtx, HERE_ACCEPT, HERE_LABEL, browse_level, build, elide_path, here_folder_label,
    row_split,
};
pub use capture::{Capture, CaptureStage, KeepEdit, LinkEdit, LinkEditMode, RenameEdit, ValueEdit};
pub use comparison::{CONFLICT_ROWS, ComparisonRequest, ComparisonView, ConflictSubject};
#[allow(unused_imports)]
// the table's own vocabulary is consumed by the lifecycle law and workspace tests
pub use journey::{
    Audition, Beneath, Bind, Event, Journey, Landing, Parked, Resume, Rung, State, Surface,
    landing_of,
};
#[allow(unused_imports)]
// used by overlay::tests (format_hint/HintAction directly; PIN_TAG below)
pub use kind::{
    ARROWS_LR, ARROWS_UD, AcceptDisposition, HINT_SEP, HintAction, OverlayKind, PIN_TAG,
    RANGE_LR_LABEL, format_hint,
};
#[allow(unused_imports)]
// OverlayRow/RowMeta/RowMetaTag: used by overlay tests and source-audit laws
pub use state::{OverlayRow, OverlayState, RangeCell, RowMeta, RowMetaTag};

fn command_hint_actions() -> Vec<HintAction> {
    vec![
        HintAction {
            glyph: "↵",
            label: "choose",
        },
        HintAction {
            glyph: ARROWS_LR,
            label: "category",
        },
        HintAction {
            glyph: "esc",
            label: "close",
        },
    ]
}

#[cfg(test)]
mod tests;
