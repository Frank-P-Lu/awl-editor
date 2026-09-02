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
//!     `Bind::Value`), it is a FLAT picker: the workspace's direct children,
//!     read one level deep and never deeper, MERGED with the recent-projects
//!     MRU, whose roots arrive as whole absolute paths at any depth
//!     (`build::recent`). Enter on a folder switches the root immediately (the
//!     synthetic accept-this-folder row — [`here_folder_label`] — picks the
//!     level's own directory), and there is no ascend affordance —
//!     Left/Right cycle the lens strip, not depth. Descended into from a
//!     Settings folder-VALUE row (`Bind::Path`), it keeps the full
//!     destination-navigator grammar: Enter on a folder DESCENDS into it and
//!     Backspace with an empty query ASCENDS, even above the workspace. Git
//!     folders carry a dim `git` tag in the row's secondary column either way.
//!   * `ProjectBrowse` — the FLAT picker's one door PAST that flatness, opened
//!     from its terminal `Browse for folder…` row: a folders-only walk of the
//!     workspace by absolute path with the destination navigators' grammar
//!     (`→` in, `←`/`⌫` out, `↵` takes the folder you stopped on), which is how
//!     a project nested deeper than a direct child is reached at all. It
//!     DESCENDS — the picker parks at its row and Esc comes back to it — and
//!     its accept emits the switch under `Project`, the one owner of "make this
//!     the root". The workspace is its floor: `browse_level` will not build a
//!     level outside it.
//!   * `Browse`  — ONE directory level at a time for the active root. Enter on a
//!     FOLDER descends (the list becomes that folder's children); Left/Backspace
//!     ASCENDS; Enter on a FILE opens it and closes. Git folders are marked. It
//!     is still summoned + transient — it vanishes on open/cancel, never a tree.

mod assets;
mod build;
mod capture;
pub(crate) mod comparison;
mod facet;
mod filter;
mod hint;
mod journey;
mod kind;
mod kind_composition;
mod link;
mod nav;
mod rename_edit;
mod roster;
mod row;
mod search;
mod semantic;
mod state;
mod table_dims;
pub(crate) mod workspace;

#[allow(unused_imports)] // HERE_LABEL / here_folder_label: read by the row-label laws
pub use build::{
    BuildCtx, HERE_ACCEPT, HERE_LABEL, SpellSuggestTarget, browse_level, build,
    elide_directory_path, elide_path, goto_folder_roster, here_folder_label, row_split,
};
// THE one question separating the switch-project roster's two routes, hoisted to
// the module surface because the law guarding the split lives outside it, and a
// law that re-derives "is this a remembered path" is a second owner of it.
pub(crate) use build::recent::is_remembered_root;
pub use capture::{Capture, CaptureStage, KeepEdit, LinkEdit, ValueEdit};
pub use comparison::{CONFLICT_ROWS, ComparisonRequest, ComparisonView, ConflictSubject};
#[allow(unused_imports)]
// used by overlay::tests (format_hint/HintAction directly; PIN_TAG below)
pub use hint::{ARROWS_LR, ARROWS_UD, HINT_SEP, HintAction, PIN_TAG, RANGE_LR_LABEL, format_hint};
#[allow(unused_imports)]
// the table's own vocabulary is consumed by the lifecycle law and workspace tests
pub use journey::{
    Audition, Beneath, Bind, Event, Journey, Landing, Parked, Resume, Rung, State, Surface,
    landing_of,
};
pub use kind::{AcceptDisposition, OverlayKind};
pub use link::LinkEditMode;
pub use rename_edit::RenameEdit;
#[allow(unused_imports)]
// OverlayRow/RowMeta/RowMetaTag: used by overlay tests and source-audit laws
pub use row::{OverlayRow, RangeCell, RowMeta, RowMetaTag, add_to_dictionary_label};
pub use state::{HugRoster, OverlayState};
#[allow(unused_imports)]
// DEFAULT_COLS/DEFAULT_ROWS/MIN_DIM: read only by test-only journeys
// (actions::tests::insert_table, main/tests::minibuffers), never by a
// non-test caller.
pub use table_dims::{DEFAULT_COLS, DEFAULT_ROWS, MAX_COLS, MAX_ROWS, MIN_DIM, TableDimsEdit};

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
